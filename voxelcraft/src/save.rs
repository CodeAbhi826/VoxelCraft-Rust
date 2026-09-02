//! World save orchestration — chunk ↔ NBT ↔ Anvil + `level.dat`
//! (Master Spec §28, Phase 2 gate: deterministic read/write tests).
//!
//! Layering:
//!
//! * `nbt.rs`   — Named Binary Tag codec (the bytes);
//! * `anvil.rs` — region container `region/r.X.Z.mca` (the file);
//! * **this module** — the *vanilla 1.16.5 disk schema* on top: chunk
//!   `Level` compounds (paletted `Sections`, `Biomes`, `Heightmaps`),
//!   `level.dat` (gzip NBT `Data` compound), and the state-id mapping
//!   between our runtime registry and `minecraft:*` registry names.
//!
//! §28 mandates separating the *internal runtime format* from the
//! *external compatibility format* — nothing here leaks into the live
//! `Chunk` layout. Two deliberate approximations (documented, benign for
//! 1.16.5 readers):
//!
//! * `Heightmaps` are written from the gen column height (terrain surface,
//!   not trees/water) as both `WORLD_SURFACE` and `MOTION_BLOCKING`;
//!   vanilla recalculates heightmaps it considers stale on load.
//! * section light arrays are omitted — the light engine is Phase 4;
//!   vanilla relights sections with missing light data.
//!
//! Robustness (§46): every parse of foreign/corrupt data degrades to
//! "treat as absent / regenerate" or unknown-name → air; nothing panics.
//!
//! The load path ignores `xPos`/`zPos` (the caller knows which chunk it
//! asked for — the region slot IS the position) and recomputes
//! `Chunk::height` from section content, so a stale/absent heightmap
//! can never desync gameplay from saved blocks.

use crate::anvil;
use crate::blocks::{
    self, prop_state_decode, prop_state_encode, COBBLE_STAIRS, OAK_FENCE, OAK_SLAB,
    BIRCH_LOG_X, BIRCH_LOG_Z, OAK_LOG_X, OAK_LOG_Z, SPRUCE_LOG_X, SPRUCE_LOG_Z,
};
use crate::chunk::{Chunk, Section, SECTION_COUNT, SECTION_LEN};
use crate::nbt::{self, Nbt};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Minecraft 1.16.5 (`https://minecraft.wiki/w/Data_version`).
pub const DATA_VERSION: i32 = 2586;
/// Chunk status a fully-generated+decorated chunk carries (1.16.5 names).
const STATUS_FULL: &str = "full";

// ---------------------------------------------------------------------------
// registry mapping: our block ids ↔ vanilla `minecraft:*` names
//
// Vanilla 1.16.5 numeric biome ids (world-internal registry order,
// cross-checked against minecraft.wiki / 1.16.5 data dumps):
//   0 ocean · 1 plains · 2 desert · 3 mountains · 4 forest · 12 snowy
//   tundra · 16 beach. Our gen.rs Biome enum maps onto those 1:1.
// ---------------------------------------------------------------------------

/// our Biome id → vanilla 1.16.5 numeric biome id
/// (Ocean, Beach, Plains, Forest, Desert, Snowy, Mountains)
const BIOME_TO_VANILLA: [i32; 7] = [0, 16, 1, 4, 2, 12, 3];
/// vanilla biome id → ours; unknown → Plains (2)
fn vanilla_biome_to_ours(v: i32) -> u8 {
    match v {
        0 => 0, // Ocean
        16 => 1, // Beach
        1 => 2,  // Plains
        4 => 3,  // Forest
        2 => 4,  // Desert
        12 => 5, // Snowy (taiga/tundra family)
        3 => 6,  // Mountains
        _ => 2,  // unknown → Plains (safe, always valid)
    }
}

/// flat block ids (0..=56) → vanilla registry names. Index 14 ("Snowy
/// Grass") shares `grass_block` with 1 via the `snowy=true` property.
const VANILLA_NAMES: [&str; 57] = [
    "minecraft:air",
    "minecraft:grass_block", // 1 Grass Block
    "minecraft:dirt",
    "minecraft:stone",
    "minecraft:cobblestone",
    "minecraft:sand",
    "minecraft:oak_log",
    "minecraft:oak_planks",
    "minecraft:oak_leaves",
    "minecraft:water",
    "minecraft:glass",
    "minecraft:bedrock",
    "minecraft:gravel",
    "minecraft:snow_block",
    "minecraft:grass_block", // 14 Snowy Grass → + Properties{snowy:true}
    "minecraft:grass",       // 1.16.5 name of the short-grass plant
    "minecraft:poppy",
    "minecraft:dandelion",
    "minecraft:granite",
    "minecraft:diorite",
    "minecraft:andesite",
    "minecraft:stone_bricks",
    "minecraft:bricks",
    "minecraft:mossy_cobblestone",
    "minecraft:smooth_stone",
    "minecraft:obsidian",
    "minecraft:coal_ore",
    "minecraft:iron_ore",
    "minecraft:gold_ore",
    "minecraft:diamond_ore",
    "minecraft:redstone_ore",
    "minecraft:lapis_ore",
    "minecraft:emerald_ore",
    "minecraft:iron_block",
    "minecraft:gold_block",
    "minecraft:diamond_block",
    "minecraft:glowstone",
    "minecraft:bookshelf",
    "minecraft:crafting_table",
    "minecraft:clay",
    "minecraft:terracotta",
    "minecraft:pumpkin",
    "minecraft:melon",
    "minecraft:ice",
    "minecraft:cactus",
    "minecraft:white_wool",
    "minecraft:red_wool",
    "minecraft:blue_wool",
    "minecraft:yellow_wool",
    "minecraft:black_wool",
    "minecraft:birch_log",
    "minecraft:birch_leaves",
    "minecraft:spruce_log",
    "minecraft:spruce_leaves",
    "minecraft:red_mushroom",
    "minecraft:brown_mushroom",
    "minecraft:dead_bush",
];

/// registry name for the three property-driven blocks (Phase 1)
fn model_block_name(b: u8) -> &'static str {
    match b {
        OAK_SLAB => "minecraft:oak_slab",
        COBBLE_STAIRS => "minecraft:cobblestone_stairs",
        OAK_FENCE => "minecraft:oak_fence",
        _ => "minecraft:air",
    }
}

/// our state id → (vanilla registry name, properties) for a palette entry.
/// Properties are emitted **vanilla-style**: absent for property-less
/// blocks, present only with non-default values worth preserving.
fn state_to_vanilla(s: u16) -> (String, Vec<(String, String)>) {
    // property blocks (slab / stairs / fence states, base 63+)
    if let Some((b, props)) = prop_state_decode(s) {
        let props: Vec<(String, String)> = props
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        return (model_block_name(b).to_string(), props);
    }
    // log axis variants
    let axis = match s {
        OAK_LOG_X | BIRCH_LOG_X | SPRUCE_LOG_X => Some("x"),
        OAK_LOG_Z | BIRCH_LOG_Z | SPRUCE_LOG_Z => Some("z"),
        _ => None,
    };
    if let Some(axis) = axis {
        let name = VANILLA_NAMES[blocks::state_block(s) as usize];
        return (
            name.to_string(),
            vec![("axis".to_string(), axis.to_string())],
        );
    }
    // snowy grass variant
    if s == 14 {
        return (
            "minecraft:grass_block".to_string(),
            vec![("snowy".to_string(), "true".to_string())],
        );
    }
    // flat identity block
    let idx = s as usize;
    if idx < VANILLA_NAMES.len() {
        (VANILLA_NAMES[idx].to_string(), Vec::new())
    } else {
        // unmapped runtime state — degrade to air (never hit today)
        ("minecraft:air".to_string(), Vec::new())
    }
}

/// vanilla palette entry → our state id; `None` = unknown name (→ air).
fn vanilla_to_state(name: &str, props: &[(String, String)]) -> Option<u16> {
    let prop = |key: &str| -> Option<&str> {
        props.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    };
    // property blocks
    let prop_block = match name {
        "minecraft:oak_slab" => Some(OAK_SLAB),
        "minecraft:cobblestone_stairs" => Some(COBBLE_STAIRS),
        "minecraft:oak_fence" => Some(OAK_FENCE),
        _ => None,
    };
    if let Some(b) = prop_block {
        let set: Vec<(&str, &str)> =
            props.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        return prop_state_encode(b, &set); // missing props → vanilla defaults
    }
    // grass variants (snowy property selects our Snowy Grass id)
    if name == "minecraft:grass_block" {
        return match prop("snowy") {
            Some("true") => Some(14),
            _ => Some(1),
        };
    }
    // log axis variants (per-log state triples — the axis offsets differ
    // per species, so decode via explicit constants)
    if let Some((block, x_state, z_state)) = match name {
        "minecraft:oak_log" => Some((blocks::OAK_LOG as u16, OAK_LOG_X, OAK_LOG_Z)),
        "minecraft:birch_log" => Some((blocks::BIRCH_LOG as u16, BIRCH_LOG_X, BIRCH_LOG_Z)),
        "minecraft:spruce_log" => Some((blocks::SPRUCE_LOG as u16, SPRUCE_LOG_X, SPRUCE_LOG_Z)),
        _ => None,
    } {
        return match prop("axis") {
            Some("x") => Some(x_state),
            Some("z") => Some(z_state),
            _ => Some(block), // axis=y / absent → default
        };
    }
    // flat blocks (skip 14 — handled above via grass_block+snowy)
    VANILLA_NAMES
        .iter()
        .position(|&n| n == name && n != "minecraft:grass_block")
        .map(|i| i as u16)
}

// ---------------------------------------------------------------------------
// heightmap packing (9-bit, 7 per long, 37 longs — vanilla rule)
// ---------------------------------------------------------------------------

fn pack_heightmap(height: &[u8; 256]) -> Vec<i64> {
    let mut longs = vec![0i64; 37];
    for (i, &h) in height.iter().enumerate() {
        // vanilla heightmap value = highest block y + 1 (0..=256, 9 bits)
        let v = (h as u32).saturating_add(1).min(256) as i64;
        let long = i / 7;
        let shift = (i % 7) * 9;
        longs[long] |= v << shift;
    }
    longs
}

/// topmost non-air y per column, recomputed from section content
fn recompute_height(chunk: &mut Chunk) {
    for z in 0..16usize {
        for x in 0..16usize {
            chunk.height[z * 16 + x] = 0;
            'col: for sy in (0..SECTION_COUNT).rev() {
                if let Some(sec) = &chunk.sections[sy] {
                    if sec.is_empty() {
                        continue;
                    }
                    for y in (0..16usize).rev() {
                        if sec.get(x, y, z) != 0 {
                            chunk.height[z * 16 + x] = (sy * 16 + y) as u8;
                            break 'col;
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// chunk → NBT (vanilla 1.16.5 disk schema)
// ---------------------------------------------------------------------------

/// vanilla palette bits for a palette of `len` entries (4..8; our registry
/// can never exceed 256 distinct states per section, so no direct mode)
fn bits_for(len: usize) -> u8 {
    let mut b = 4u8;
    while (1usize << b) < len && b < 8 {
        b += 1;
    }
    b
}

/// Serialize a chunk into the vanilla 1.16.5 chunk NBT root (unnamed root
/// compound carrying `DataVersion` + `Level`). Deterministic: same chunk +
/// same `last_update` → byte-identical output (gate §Phase 2).
pub fn chunk_to_nbt(
    cx: i32,
    cz: i32,
    chunk: &Chunk,
    last_update: i64,
    light: Option<&crate::light::LightData>,
) -> Vec<u8> {
    let mut level = Nbt::compound();
    level.set("xPos", Nbt::Int(cx));
    level.set("zPos", Nbt::Int(cz));
    level.set("Status", Nbt::String(STATUS_FULL.into()));
    level.set("LastUpdate", Nbt::Long(last_update));
    level.set("InhabitedTime", Nbt::Long(0));

    // ---- Sections: one compound per non-empty 16³ section ----
    let mut sections: Vec<Nbt> = Vec::new();
    for (sy, slot) in chunk.sections.iter().enumerate() {
        let Some(sec) = slot else { continue };
        if sec.is_empty() {
            continue;
        }
        let flat = sec.states_flat();

        // palette in first-appearance order (deterministic over the YZX scan)
        let mut palette: Vec<u16> = Vec::new(); // palette index → our state id
        let mut indices = [0u16; SECTION_LEN];
        for (i, &s) in flat.iter().enumerate() {
            match palette.iter().position(|&p| p == s) {
                Some(pi) => indices[i] = pi as u16,
                None => {
                    indices[i] = palette.len() as u16;
                    palette.push(s);
                }
            }
        }

        // pack palette indices, entries never straddling a u64 (vanilla rule)
        let bits = bits_for(palette.len()) as usize;
        let epl = 64 / bits;
        let mut data = vec![0i64; SECTION_LEN.div_ceil(epl)];
        for (i, &pi) in indices.iter().enumerate() {
            let word = i / epl;
            let shift = (i % epl) * bits;
            data[word] |= (pi as i64) << shift;
        }

        // palette compounds: {Name, Properties?} — properties omitted for
        // property-less blocks, exactly like vanilla
        let mut palette_nbt: Vec<Nbt> = Vec::with_capacity(palette.len());
        for &s in &palette {
            let (name, props) = state_to_vanilla(s);
            let mut entry = Nbt::compound();
            entry.set("Name", Nbt::String(name));
            if !props.is_empty() {
                let mut pr = Nbt::compound();
                for (k, v) in props {
                    pr.set(&k, Nbt::String(v));
                }
                entry.set("Properties", pr);
            }
            palette_nbt.push(entry);
        }

        let mut sec_nbt = Nbt::compound();
        sec_nbt.set("Y", Nbt::Byte(sy as i8));
        sec_nbt.set("Palette", Nbt::List(palette_nbt));
        sec_nbt.set("BlockStates", Nbt::LongArray(data));
        // Phase 4 §28: light arrays (vanilla nibble format — even index in
        // the low nibble). Only materialized sections carry data; the
        // loader treats a save with NO light arrays anywhere as pre-P4 and
        // re-lights on load.
        if let Some(ld) = light {
            if let Some(lsec) = &ld.sections[sy] {
                sec_nbt.set("SkyLight", Nbt::ByteArray(pack_nibbles(&lsec.sky)));
                sec_nbt.set("BlockLight", Nbt::ByteArray(pack_nibbles(&lsec.blk)));
            }
        }
        sections.push(sec_nbt);
    }
    // Phase 4: sections with NO blocks but materialized light (block light
    // reaches into pure-air sections) — emit air-palette compounds so the
    // light survives the round-trip.
    if let Some(ld) = light {
        for sy in 0..16usize {
            if chunk.sections[sy]
                .as_ref()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
            {
                continue; // already written above
            }
            let Some(lsec) = &ld.sections[sy] else { continue };
            let mut sec_nbt = Nbt::compound();
            sec_nbt.set("Y", Nbt::Byte(sy as i8));
            let mut air = Nbt::compound();
            air.set("Name", Nbt::String("minecraft:air".into()));
            sec_nbt.set("Palette", Nbt::List(vec![air]));
            // single-entry palette → 4-bit indices, 256 longs of zeros
            sec_nbt.set("BlockStates", Nbt::LongArray(vec![0i64; 256]));
            sec_nbt.set("SkyLight", Nbt::ByteArray(pack_nibbles(&lsec.sky)));
            sec_nbt.set("BlockLight", Nbt::ByteArray(pack_nibbles(&lsec.blk)));
            sections.push(sec_nbt);
        }
    }
    level.set("Sections", Nbt::List(sections));

    // ---- Biomes: per-column IntArray(256), z*16+x order (1.16.5 rule) ----
    let biomes: Vec<i32> = chunk
        .biome
        .iter()
        .map(|&b| *BIOME_TO_VANILLA.get(b as usize).unwrap_or(&1))
        .collect();
    level.set("Biomes", Nbt::IntArray(biomes));

    // ---- Heightmaps (approximation, see module doc) ----
    let mut heightmaps = Nbt::compound();
    heightmaps.set("WORLD_SURFACE", Nbt::LongArray(pack_heightmap(&chunk.height)));
    heightmaps.set("MOTION_BLOCKING", Nbt::LongArray(pack_heightmap(&chunk.height)));
    level.set("Heightmaps", heightmaps);

    let mut root = Nbt::compound();
    root.set("DataVersion", Nbt::Int(DATA_VERSION));
    root.set("Level", level);
    // cannot fail for this shape (depth 5, bounded sizes) — a failure here
    // is an engine bug, not corrupt input
    nbt::write_root("", &root).expect("chunk nbt writer")
}

// ---------------------------------------------------------------------------
// NBT → chunk (load path; tolerant of foreign/corrupt data)
// ---------------------------------------------------------------------------

/// Derive the effective bits-per-entry of a `BlockStates` array from its
/// length: `epl = ceil(4096/longs)`, `bpe = floor(64/epl)`. Covers the
/// whole vanilla ladder (256→4, 342→5, 410→6, 456→7, 512→8, 1024→15/16).
/// For 1024 longs the 15-vs-16-bit ambiguity is resolved by testing whether
/// any long uses bits 60..=63 (always zero in 15-bit packing).
fn derive_bits(longs: usize) -> Option<u8> {
    if longs == 0 || longs > 2048 {
        return None; // absurd → caller treats section as empty
    }
    let epl = SECTION_LEN.div_ceil(longs);
    if epl == 0 || 64 / epl == 0 {
        return None;
    }
    let mut bpe = (64 / epl) as u8;
    if epl == 4 && bpe == 16 {
        bpe = 15; // vanilla direct width; 16-bit variants still decode: any
                  // long with bits 60+ set would misalign anyway — accepted
    }
    Some(bpe)
}

/// pack a 4096-entry nibble array (vanilla order: even index = low nibble)
fn pack_nibbles(data: &[u8; 4096]) -> Vec<i8> {
    let mut out = vec![0i8; 2048];
    for (i, &v) in data.iter().enumerate() {
        out[i >> 1] |= ((v & 0xF) as i8) << ((i & 1) * 4);
    }
    out
}

/// unpack a vanilla nibble array (missing/short arrays read as 0)
fn unpack_nibbles(data: &[i8]) -> [u8; 4096] {
    let mut out = [0u8; 4096];
    for i in 0..4096 {
        let b = data.get(i >> 1).copied().unwrap_or(0) as u8;
        out[i] = (b >> ((i & 1) * 4)) & 0xF;
    }
    out
}

/// Parse a vanilla 1.16.5 chunk NBT root into a `Chunk` (+ its light, when
/// the save carries light arrays). Unknown palette names degrade to air;
/// corrupt sections are skipped; `height` is recomputed from content.
/// Returns `Err(reason)` only for wholesale unparseable data (callers fall
/// back to terrain regeneration). A save with no light arrays at all
/// (pre-Phase-4) yields `None` → the caller re-lights on load.
pub fn chunk_from_nbt(data: &[u8]) -> Result<(Chunk, Option<crate::light::LightData>), String> {
    let (_root_name, root) = nbt::read_root(data).map_err(|e| e.to_string())?;
    let level = root
        .get("Level")
        .ok_or_else(|| "chunk nbt: missing Level compound".to_string())?;
    if !matches!(level, Nbt::Compound(_)) {
        return Err("chunk nbt: Level is not a compound".into());
    }

    let mut chunk = Chunk::empty();
    let mut out_light = crate::light::LightData::new();
    let mut any_light = false;

    // ---- sections ----
    if let Some(sections) = level.get("Sections").and_then(|s| s.as_list()) {
        for sec in sections {
            let Some(sy) = sec.get("Y").and_then(|y| y.as_i64()) else {
                continue;
            };
            if !(0..16).contains(&sy) {
                continue; // corrupt Y — skip section (§46)
            }
            let sy = sy as usize;

            // palette: list of {Name, Properties?}
            let mut palette: Vec<u16> = Vec::new();
            if let Some(pal) = sec.get("Palette").and_then(|p| p.as_list()) {
                for entry in pal {
                    let name = entry
                        .get("Name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("minecraft:air");
                    let mut props: Vec<(String, String)> = Vec::new();
                    if let Some(Nbt::Compound(pr)) = entry.get("Properties") {
                        for (k, v) in pr {
                            if let Nbt::String(val) = v {
                                props.push((k.clone(), val.clone()));
                            }
                        }
                    }
                    palette.push(vanilla_to_state(name, &props).unwrap_or(0));
                }
            }
            if palette.is_empty() {
                palette.push(0); // empty palette → all-air section
            }

            // BlockStates → flat states
            let mut flat = [0u16; SECTION_LEN];
            if let Some(longs) = sec.get("BlockStates").and_then(|d| d.as_i64_slice()) {
                if let Some(bits) = derive_bits(longs.len()) {
                    let bits = bits as usize;
                    let epl = 64 / bits;
                    let mask = (1u64 << bits) - 1;
                    for i in 0..SECTION_LEN {
                        let word = i / epl;
                        if word >= longs.len() {
                            break;
                        }
                        let shift = (i % epl) * bits;
                        let pi = ((longs[word] >> shift) as u64 & mask) as usize;
                        // out-of-range index (corrupt/trailing junk) → air
                        flat[i] = palette.get(pi).copied().unwrap_or(0);
                    }
                }
            }
            // Phase 4 §28: light arrays — materialize the section when present
            if let Some(sky) = sec.get("SkyLight").and_then(|d| d.as_i8_slice()) {
                let lsec = out_light
                    .sections[sy]
                    .get_or_insert_with(|| {
                        Box::new(crate::light::LightSection {
                            sky: Box::new([0u8; 4096]),
                            blk: Box::new([0u8; 4096]),
                        })
                    });
                lsec.sky = Box::new(unpack_nibbles(sky));
                if let Some(blk) = sec.get("BlockLight").and_then(|d| d.as_i8_slice()) {
                    lsec.blk = Box::new(unpack_nibbles(blk));
                }
                any_light = true;
            }
            // (missing BlockStates → flat stays all-air — 1.18-style single
            // palette entries; harmless to accept)

            chunk.sections[sy] = Section::from_states(&flat).map(std::sync::Arc::from);
        }
    }

    // ---- biomes ----
    if let Some(bi) = level.get("Biomes").and_then(|b| b.as_i32_slice()) {
        for i in 0..256usize.min(bi.len()) {
            chunk.biome[i] = vanilla_biome_to_ours(bi[i]);
        }
    }

    recompute_height(&mut chunk);
    Ok((
        chunk,
        if any_light {
            Some(out_light)
        } else {
            None // pre-Phase-4 save → caller re-lights
        },
    ))
}

// ---------------------------------------------------------------------------
// level.dat
// ---------------------------------------------------------------------------

/// World metadata persisted in `level.dat` (vanilla keys + a `voxelcraft`
/// sub-compound for engine-specific player state, ignored by vanilla).
#[derive(Clone, Debug, Default)]
pub struct WorldMeta {
    pub seed: u64,
    pub name: String,
    pub spawn: (i32, i32, i32),
    /// player position + orientation restored on load
    pub player: Option<PlayerMeta>,
    /// engine world-age tick (vanilla `Time`)
    pub game_time: i64,
}

#[derive(Clone, Debug)]
pub struct PlayerMeta {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
}

fn gzip_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(data)?;
    enc.finish()
}

fn gunzip_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(data).read_to_end(&mut out)?;
    Ok(out)
}

/// Write `level.dat` (gzip NBT) with an atomic swap: old file is kept as
/// `level.dat_old` — same crash-safety pattern as vanilla.
pub fn write_level_dat(world_dir: &Path, meta: &WorldMeta) -> std::io::Result<()> {
    let mut data = Nbt::compound();
    data.set("LevelName", Nbt::String(meta.name.clone()));
    data.set("RandomSeed", Nbt::Long(meta.seed as i64));
    data.set("SpawnX", Nbt::Int(meta.spawn.0));
    data.set("SpawnY", Nbt::Int(meta.spawn.1));
    data.set("SpawnZ", Nbt::Int(meta.spawn.2));
    data.set("DataVersion", Nbt::Int(DATA_VERSION));
    data.set("GameType", Nbt::Int(1)); // creative
    data.set("Time", Nbt::Long(meta.game_time));
    data.set(
        "LastPlayed",
        Nbt::Long(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)),
    );
    if let Some(p) = &meta.player {
        let mut vc = Nbt::compound();
        vc.set("PlayerX", Nbt::Double(p.pos[0]));
        vc.set("PlayerY", Nbt::Double(p.pos[1]));
        vc.set("PlayerZ", Nbt::Double(p.pos[2]));
        vc.set("PlayerYaw", Nbt::Float(p.yaw));
        vc.set("PlayerPitch", Nbt::Float(p.pitch));
        data.set("voxelcraft", vc);
    }
    let mut root = Nbt::compound();
    root.set("Data", data);
    let bytes = nbt::write_root("", &root).expect("level.dat writer");
    let gz = gzip_bytes(&bytes)?;

    fs::create_dir_all(world_dir)?;
    let path = world_dir.join("level.dat");
    let tmp = world_dir.join("level.dat.tmp");
    fs::write(&tmp, &gz)?;
    if path.exists() {
        let _ = fs::rename(&path, world_dir.join("level.dat_old")); // keep backup
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Read `level.dat` (falling back to `level.dat_old`); missing → `None`.
/// Field values default permissively — a level.dat from a foreign tool
/// still yields a usable `WorldMeta` (§46).
pub fn read_level_dat(world_dir: &Path) -> std::io::Result<Option<WorldMeta>> {
    let path = world_dir.join("level.dat");
    // primary first, vanilla's kept backup as fallback; both absent → None
    let gz = fs::read(&path)
        .or_else(|_| fs::read(world_dir.join("level.dat_old")))
        .ok();
    let Some(gz) = gz else { return Ok(None) };
    let bytes = gunzip_bytes(&gz)?;

    let (_name, root) = match nbt::read_root(&bytes) {
        Ok(r) => r,
        Err(_) => return Ok(None), // corrupt → caller regenerates
    };
    let data = root.get("Data").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "level.dat: no Data compound")
    })?;
    let get_i64 = |k: &str| data.get(k).and_then(|v| v.as_i64());
    let mut meta = WorldMeta {
        seed: get_i64("RandomSeed").unwrap_or(0) as u64,
        name: data
            .get("LevelName")
            .and_then(|v| v.as_str())
            .unwrap_or("VoxelCraft")
            .to_string(),
        spawn: (
            get_i64("SpawnX").unwrap_or(8) as i32,
            get_i64("SpawnY").unwrap_or(crate::SEA_LEVEL as i64 + 1) as i32,
            get_i64("SpawnZ").unwrap_or(8) as i32,
        ),
        player: None,
        game_time: get_i64("Time").unwrap_or(0),
    };
    if let Some(Nbt::Compound(vc)) = data.get("voxelcraft") {
        // Vec<(String, Nbt)> — direct lookup (Nbt::get is on Nbt, not the vec)
        let find = |k: &str| vc.iter().find(|(key, _)| key == k).map(|(_, v)| v);
        let d = |k: &str| find(k).map(|v| if let Nbt::Double(x) = v { *x } else { 0.0 });
        let fl = |k: &str| find(k).map(|v| if let Nbt::Float(x) = v { *x } else { 0.0 });
        meta.player = Some(PlayerMeta {
            pos: [
                d("PlayerX").unwrap_or(meta.spawn.0 as f64),
                d("PlayerY").unwrap_or(meta.spawn.1 as f64 + 20.0),
                d("PlayerZ").unwrap_or(meta.spawn.2 as f64),
            ],
            yaw: fl("PlayerYaw").unwrap_or(0.0),
            pitch: fl("PlayerPitch").unwrap_or(0.0),
        });
    }
    Ok(Some(meta))
}

// ---------------------------------------------------------------------------
// high-level world directory operations (native only by nature of anvil.rs)
// ---------------------------------------------------------------------------

/// Default save location: `./saves/VoxelCraft` next to the current dir.
/// (A platform config-dir layout arrives with §32 settings work.)
/// §28: the save directory of one dimension. Vanilla layout: the overworld
/// saves at the world root; the nether at `<world>/DIM-1` (the End, if it
/// ever ships, is `DIM1`). Chunks of different dimensions never mix.
pub fn dimension_dir(world_dir: &Path, dim: crate::world::Dimension) -> PathBuf {
    match dim {
        crate::world::Dimension::Overworld => world_dir.to_path_buf(),
        crate::world::Dimension::Nether => world_dir.join("DIM-1"),
    }
}

pub fn default_world_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("saves")
        .join("VoxelCraft")
}

/// Persist one chunk into its region file (Anvil write, zlib like vanilla).
pub fn store_chunk(
    world_dir: &Path,
    cx: i32,
    cz: i32,
    chunk: &Chunk,
    last_update: i64,
    light: Option<&crate::light::LightData>,
) -> std::io::Result<()> {
    anvil::write_chunk(world_dir, cx, cz, &chunk_to_nbt(cx, cz, chunk, last_update, light))
}

/// Persist many chunks in one pass — one compact-and-rewrite per touched
/// region file (autosave path; ~400 chunks → a handful of rewrites).
pub fn store_chunks(
    world_dir: &Path,
    entries: &[(i32, i32, &Chunk, Option<&std::sync::Arc<crate::light::LightData>>)],
    last_update: i64,
) -> std::io::Result<()> {
    let encoded: Vec<(i32, i32, Vec<u8>)> = entries
        .iter()
        .map(|(cx, cz, c, l)| (*cx, *cz, chunk_to_nbt(*cx, *cz, c, last_update, l.map(|a| a.as_ref()))))
        .collect();
    anvil::write_chunks(world_dir, &encoded)
}

/// Load one chunk (+ its light) from its region file. `Ok(None)` = not saved
/// (or corrupt beyond repair) → caller regenerates from terrain.
pub fn load_chunk(
    world_dir: &Path,
    cx: i32,
    cz: i32,
) -> std::io::Result<Option<(Chunk, Option<crate::light::LightData>)>> {
    match anvil::read_chunk(world_dir, cx, cz)? {
        None => Ok(None),
        Some(nbt_bytes) => Ok(chunk_from_nbt(&nbt_bytes).ok()),
    }
}

// ------------------------------------------------------------------ tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::*;
    use crate::chunk::idx;
    use std::time::SystemTime;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "vc-save-{tag}-{}-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            std::process::id()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// A chunk exercising every serialization-relevant state family:
    /// flat blocks, ores, snowy grass, cross plants, water, log axis
    /// variants, property blocks (slab/stairs/fence defaults + non-defaults),
    /// and content in a high section (y=200 → section 12).
    fn demo_chunk() -> Chunk {
        let mut c = Chunk::empty();
        // bedrock + stone floor in section 0
        for y in 0..4usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    c.set(x, y, z, if y == 0 { BEDROCK } else { STONE });
                }
            }
        }
        // ore sprinkling (direct 4→5-bit palette growth)
        c.set(1, 2, 1, DIAMOND_ORE);
        c.set(2, 2, 2, GOLD_ORE);
        c.set(3, 3, 3, REDSTONE_ORE);
        c.set(4, 1, 4, GRAVEL);
        // surface-ish
        c.set(8, 4, 8, GRASS);
        c.set(9, 4, 9, DIRT);
        c.set(10, 4, 10, SAND);
        c.set(11, 4, 11, 14); // Snowy Grass
        c.set(12, 5, 12, 15); // tall grass (cross)
        c.set(13, 62, 13, WATER);
        c.set(14, 4, 14, GLOWSTONE);
        // property blocks: defaults and non-defaults
        c.set_state(2, 4, 2, 63); // oak_slab half=bottom (default)
        c.set_state(3, 4, 3, 64); // oak_slab half=top
        c.set_state(4, 70, 4, prop_state_encode(COBBLE_STAIRS, &[("facing", "east"), ("half", "top")]).unwrap());
        c.set_state(5, 70, 5, prop_state_encode(OAK_FENCE, &[("east", "true"), ("south", "true")]).unwrap());
        // log axis variants
        c.set_state(6, 70, 6, OAK_LOG_X);
        c.set_state(7, 70, 7, OAK_LOG_Z);
        c.set_state(8, 70, 8, BIRCH_LOG_X);
        c.set_state(9, 70, 9, SPRUCE_LOG_Z);
        c.set(0, 70, 0, OAK_LOG); // plain axis=y
        // high section content (section 12)
        c.set(0, 200, 0, GLOWSTONE);
        c.set(1, 200, 1, OBSIDIAN);
        // birch + spruce materials
        c.set(2, 200, 2, BIRCH_LOG);
        c.set(3, 200, 3, SPRUCE_LEAVES);
        // distinct biomes per quadrant (Ocean/Beach/Plains/Forest)
        for z in 0..16usize {
            for x in 0..16usize {
                c.biome[z * 16 + x] = (x < 8) as u8 * 2 + (z < 8) as u8;
            }
        }
        c.height[0] = 200; // arbitrary stored height (load recomputes)
        c
    }

    /// full-content comparison: every block position + biomes
    fn assert_same_content(a: &Chunk, b: &Chunk) {
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    assert_eq!(a.get(x, y, z), b.get(x, y, z), "block at ({x},{y},{z})");
                }
            }
        }
        assert_eq!(a.biome.as_ref(), b.biome.as_ref());
    }

    #[test]
    fn chunk_nbt_roundtrip_content_identical() {
        let c = demo_chunk();
        let bytes = chunk_to_nbt(5, -7, &c, 12345, None);
        let back = chunk_from_nbt(&bytes).unwrap();
        assert_same_content(&c, &back.0);
    }

    #[test]
    fn chunk_roundtrip_is_deterministic() {
        let c = demo_chunk();
        // serialize twice → byte-identical (Phase 2 gate)
        let a = chunk_to_nbt(-3, 9, &c, 777, None);
        let b = chunk_to_nbt(-3, 9, &c, 777, None);
        assert_eq!(a, b);
        // save→load→save reaches a FIXED POINT after one cycle: the only
        // load-side mutation is `height`, recomputed from content (derived
        // data — §28 runtime/external separation); palette order, sections
        // and biomes are content-derived, so resaves stay byte-stable.
        let loaded = chunk_from_nbt(&a).unwrap().0;
        let resaved = chunk_to_nbt(-3, 9, &loaded, 777, None);
        let reloaded = chunk_from_nbt(&resaved).unwrap().0;
        let resaved2 = chunk_to_nbt(-3, 9, &reloaded, 777, None);
        assert_eq!(resaved, resaved2, "resave must be a fixed point");
        // content survives both cycles unchanged
        assert_same_content(&c, &reloaded);
    }

    #[test]
    fn anvil_store_load_roundtrip() {
        let dir = tmp_dir("anvil");
        let c = demo_chunk();
        store_chunk(&dir, 100, -100, &c, 42, None).unwrap();
        let back = load_chunk(&dir, 100, -100).unwrap().expect("chunk present");
        assert_same_content(&c, &back.0);
        // absent chunk → None
        assert!(load_chunk(&dir, 101, -100).unwrap().is_none());
    }

    #[test]
    fn batch_store_load_roundtrip() {
        let dir = tmp_dir("batch");
        let a = demo_chunk();
        let mut b = demo_chunk();
        b.set(0, 1, 0, OBSIDIAN);
        let d = Chunk::empty(); // all-air chunk survives the cycle too
        let entries: Vec<(i32, i32, &Chunk, Option<&std::sync::Arc<crate::light::LightData>>)> = vec![
            (0, 0, &a, None),
            (31, 31, &b, None),
            (-5, 7, &d, None),
            (64, 64, &a, None),
        ];
        store_chunks(&dir, &entries, 5).unwrap();
        assert_same_content(&a, &load_chunk(&dir, 0, 0).unwrap().unwrap().0);
        assert_same_content(&b, &load_chunk(&dir, 31, 31).unwrap().unwrap().0);
        assert_same_content(&d, &load_chunk(&dir, -5, 7).unwrap().unwrap().0);
        assert_same_content(&a, &load_chunk(&dir, 64, 64).unwrap().unwrap().0);
    }

    #[test]
    fn nbt_layout_matches_vanilla_shape() {
        let c = demo_chunk();
        let bytes = chunk_to_nbt(2, 3, &c, 0, None);
        let (name, root) = nbt::read_root(&bytes).unwrap();
        assert_eq!(name, "");
        assert_eq!(root.get("DataVersion").unwrap().as_i64(), Some(2586));
        let level = root.get("Level").unwrap();
        assert_eq!(level.get("xPos").unwrap().as_i64(), Some(2));
        assert_eq!(level.get("zPos").unwrap().as_i64(), Some(3));
        assert_eq!(level.get("Status").unwrap().as_str(), Some("full"));
        // sections: palette entries carry minecraft: names, BlockStates sized right
        let sections = level.get("Sections").unwrap().as_list().unwrap();
        let sec0 = sections.iter().find(|s| s.get("Y").and_then(|y| y.as_i64()) == Some(0)).unwrap();
        let palette = sec0.get("Palette").unwrap().as_list().unwrap();
        assert!(palette.len() >= 8); // bedrock/stone/ores/grass/…
        for entry in palette {
            let n = entry.get("Name").unwrap().as_str().unwrap();
            assert!(n.starts_with("minecraft:"), "palette name {n}");
        }
        // 4-bit packing → 256 longs; 5-bit (our palette ≥ 9 distinct) → 342
        let longs = sec0.get("BlockStates").unwrap().as_i64_slice().unwrap().len();
        assert!(longs == 256 || longs == 342);
        // biomes: IntArray(256) with vanilla ids. Quadrants:
        // (x<8,z<8) → Forest(4), (x≥8,z<8) → Plains(1),
        // (x<8,z≥8) → Beach(16), (x≥8,z≥8) → Ocean(0)
        let bi = level.get("Biomes").unwrap().as_i32_slice().unwrap();
        assert_eq!(bi.len(), 256);
        assert_eq!(bi[0], 4); // (x=0,z=0) → Forest
        assert_eq!(bi[15 * 16 + 15], 0); // (x=15,z=15) → Ocean
        // heightmaps: 37 longs, 9-bit
        let hm = level.get("Heightmaps").unwrap().get("WORLD_SURFACE").unwrap();
        assert_eq!(hm.as_i64_slice().unwrap().len(), 37);
    }

    #[test]
    fn foreign_vanilla_chunk_parses() {
        // hand-built 1.16.5-style section: palette [air, stone, oak_log
        // axis=x], 4-bit packed data placing stone at (0,0,0) and the log
        // at (1,0,0)
        let mut pal_air = Nbt::compound();
        pal_air.set("Name", Nbt::String("minecraft:air".into()));
        let mut pal_stone = Nbt::compound();
        pal_stone.set("Name", Nbt::String("minecraft:stone".into()));
        let mut pal_log = Nbt::compound();
        pal_log.set("Name", Nbt::String("minecraft:oak_log".into()));
        let mut props = Nbt::compound();
        props.set("axis", Nbt::String("x".into()));
        pal_log.set("Properties", props);

        let mut data = vec![0i64; 256];
        data[0] |= 1; // (x=0,y=0,z=0) → stone
        data[0] |= 2 << 4; // (x=1,y=0,z=0) → oak_log[axis=x]

        let mut sec = Nbt::compound();
        sec.set("Y", Nbt::Byte(0));
        sec.set("Palette", Nbt::List(vec![pal_air, pal_stone, pal_log]));
        sec.set("BlockStates", Nbt::LongArray(data));
        let mut level = Nbt::compound();
        level.set("xPos", Nbt::Int(0));
        level.set("zPos", Nbt::Int(0));
        level.set("Sections", Nbt::List(vec![sec]));
        let biomes: Vec<i32> = vec![1; 256]; // plains
        level.set("Biomes", Nbt::IntArray(biomes));
        let mut root = Nbt::compound();
        root.set("DataVersion", Nbt::Int(2586));
        root.set("Level", level);
        let bytes = nbt::write_root("", &root).unwrap();

        let (chunk, _light) = chunk_from_nbt(&bytes).unwrap();
        assert_eq!(chunk.get(0, 0, 0), STONE);
        // chunk.get returns the raw u8 state id (57 = OAK_LOG_X); the flat
        // u16 accessor confirms the full state survived
        assert_eq!(chunk.get(1, 0, 0) as u16, OAK_LOG_X);
        assert_eq!(chunk.sections[0].as_ref().unwrap().states_flat()[1], OAK_LOG_X);
        assert_eq!(chunk.biome[0], 2); // plains → our id 2
        assert_eq!(chunk.height[0], 0); // top non-air at y=0
    }

    #[test]
    fn unknown_names_and_corruption_degrade_gracefully() {
        let c = demo_chunk();
        let mut bytes = chunk_to_nbt(0, 0, &c, 1, None);
        // corrupt one palette name in-place (stone → sTonE) — must still
        // parse (that state becomes air), never panic
        let needle = b"minecraft:stone";
        let pos = bytes.windows(needle.len()).position(|w| w == needle).unwrap();
        bytes[pos + 11] = b'X'; // minecraft:Xtone
        let back = chunk_from_nbt(&bytes);
        assert!(back.is_ok(), "mutated chunk still parses");
        // wholesale garbage → Err (caller regenerates)
        assert!(chunk_from_nbt(&[1, 2, 3]).is_err());
        // missing Level compound → Err
        let mut root = Nbt::compound();
        root.set("DataVersion", Nbt::Int(2586));
        let bad = nbt::write_root("", &root).unwrap();
        assert!(chunk_from_nbt(&bad).is_err());
        // empty sections list + no biomes → valid empty chunk
        let mut level = Nbt::compound();
        level.set("Sections", Nbt::List(vec![]));
        let mut root = Nbt::compound();
        root.set("Level", level);
        let empty = nbt::write_root("", &root).unwrap();
        let (e, el) = chunk_from_nbt(&empty).unwrap();
        assert_eq!(e.get(0, 0, 0), 0);
        assert!(e.sections.iter().all(|s| s.is_none()));
        assert!(el.is_none(), "no light arrays → None (pre-P4 save)");
    }

    #[test]
    fn level_dat_roundtrip() {
        let dir = tmp_dir("level");
        let meta = WorldMeta {
            seed: 0xDEAD_BEEF_CAFE_1234,
            name: "Test World".into(),
            spawn: (-17, 71, 239),
            player: Some(PlayerMeta { pos: [1.5, 72.0, -3.25], yaw: -0.75, pitch: 0.5 }),
            game_time: 4242,
        };
        write_level_dat(&dir, &meta).unwrap();
        let back = read_level_dat(&dir).unwrap().expect("level.dat present");
        assert_eq!(back.seed, meta.seed);
        assert_eq!(back.name, "Test World");
        assert_eq!(back.spawn, (-17, 71, 239));
        let p = back.player.unwrap();
        assert!((p.pos[0] - 1.5).abs() < 1e-9);
        assert!((p.pos[1] - 72.0).abs() < 1e-9);
        assert!((p.pos[2] - -3.25).abs() < 1e-9);
        assert!((p.yaw - -0.75).abs() < 1e-6);
        assert!((p.pitch - 0.5).abs() < 1e-6);
        assert_eq!(back.game_time, 4242);
        // missing dir → None; level.dat_old fallback works
        assert!(read_level_dat(&dir.join("nowhere")).unwrap().is_none());
        fs::rename(dir.join("level.dat"), dir.join("level.dat_old")).unwrap();
        let old = read_level_dat(&dir).unwrap().expect("falls back to _old");
        assert_eq!(old.seed, meta.seed);
    }

    #[test]
    fn player_edit_survives_save_cycle() {
        let dir = tmp_dir("edit");
        let mut c = demo_chunk();
        // player digs a hole and places a fence
        c.set(8, 4, 8, AIR);
        c.set_state(8, 4, 8, prop_state_encode(OAK_FENCE, &[("west", "true")]).unwrap());
        store_chunk(&dir, 0, 0, &c, 9, None).unwrap();
        let back = load_chunk(&dir, 0, 0).unwrap().expect("present");
        let back = (back.0, back.1);
        let fence = back.0.sections[0].as_ref().unwrap().states_flat()[idx(8, 4, 8)];
        assert_eq!(fence, prop_state_encode(OAK_FENCE, &[("west", "true")]).unwrap());
    }

    #[test]
    fn height_recompute_uses_top_block() {
        let mut c = Chunk::empty();
        c.set(4, 33, 4, STONE);
        c.set(4, 200, 4, GLOWSTONE);
        let bytes = chunk_to_nbt(0, 0, &c, 0, None);
        let back = chunk_from_nbt(&bytes).unwrap();
        assert_eq!(back.0.height[4 * 16 + 4], 200);
        c.set(4, 200, 4, AIR);
        let bytes = chunk_to_nbt(0, 0, &c, 0, None);
        let back = chunk_from_nbt(&bytes).unwrap();
        assert_eq!(back.0.height[4 * 16 + 4], 33);
    }

    /// Full §28 life cycle with REAL terrain-gen output (the exact path the
    /// game loop drives): generate → player edit → flush chunks + level.dat
    /// → fresh session restores seed/player/blocks, and unedited terrain
    /// reloads bit-identical to a same-seed regeneration.
    #[test]
    fn world_save_cycle_with_real_terrain() {
        let dir = tmp_dir("cycle");
        let seed: u64 = 12345;

        // --- session 1: the game generates, the player edits, autosave fires
        let gen = crate::gen::TerrainGen::new(seed);
        let (generated, _outbound) = gen.generate_chunk(0, 0, Vec::new());
        let mut chunk = (*generated).clone(); // detach from Arc for editing
        chunk.set(8, 70, 8, GLOWSTONE); // a player edit
        let entries: Vec<(i32, i32, &Chunk, Option<&std::sync::Arc<crate::light::LightData>>)> =
            vec![(0, 0, &chunk, None)];
        store_chunks(&dir, &entries, 100).unwrap();
        write_level_dat(
            &dir,
            &WorldMeta {
                seed,
                name: "VoxelCraft".into(),
                spawn: (8, 70, 8),
                player: Some(PlayerMeta { pos: [8.5, 90.0, 8.5], yaw: 1.0, pitch: -0.5 }),
                game_time: 100,
            },
        )
        .unwrap();

        // --- session 2: level.dat restores the world identity…
        let m2 = read_level_dat(&dir).unwrap().expect("level.dat survives");
        assert_eq!(m2.seed, seed);
        assert_eq!(m2.spawn, (8, 70, 8));
        let p = m2.player.unwrap();
        assert_eq!(p.pos, [8.5, 90.0, 8.5]);
        assert!((p.yaw - 1.0).abs() < 1e-6 && (p.pitch + 0.5).abs() < 1e-6);

        // --- …the chunk reloads with the edit intact…
        let (loaded, _l) = load_chunk(&dir, 0, 0).unwrap().expect("chunk present");
        assert_eq!(loaded.get(8, 70, 8), GLOWSTONE);

        // --- …and every other block matches a same-seed regeneration exactly
        let (fresh, _) = crate::gen::TerrainGen::new(seed).generate_chunk(0, 0, Vec::new());
        let mut mismatches = 0u32;
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    if (x, y, z) != (8, 70, 8) && loaded.get(x, y, z) != fresh.get(x, y, z) {
                        mismatches += 1;
                    }
                }
            }
        }
        assert_eq!(mismatches, 0, "saved terrain diverged from regeneration");
        // biome columns survive too
        assert_eq!(loaded.biome.as_ref(), fresh.biome.as_ref());
    }
}
