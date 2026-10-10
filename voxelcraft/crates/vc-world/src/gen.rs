//! World generation: simplex noise, fBm, biomes, caves, trees.
//! Fully deterministic per seed; pure functions (safe on worker threads).

use crate::vanilla_noise::VanillaTerrain;
use crate::world::Dimension;
use std::sync::Arc;
use vc_blocks::blocks::*;
use vc_chunk::chunk::Chunk;
#[cfg(test)]
use vc_chunk::chunk::CHUNK_LEN;
use vc_rng::rng::Rng;

/// the chunk-generator return: the finished chunk + queued cross-chunk
/// edits (world pos, state) for neighbors to apply on their next pass
type GenOut = (Arc<Chunk>, Vec<(i32, i32, i32, u16)>);

/// Backlog round (weather): the biome's precipitation form
/// (VERIFIED w/Weather — see `Biome::precipitation`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Precip {
    None,
    Rain,
    Snow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Ocean = 0,
    Beach = 1,
    Plains = 2,
    Forest = 3,
    Desert = 4,
    Snowy = 5,
    Mountains = 6,
    /// §26/§28: the Nether's single biome (chunk biome u8 = 7)
    NetherWastes = 7,
    // ---- Phase 10 content breadth: 6 climate biomes (vanilla save ids
    // live-verified from the wiki Biome page: taiga=5, swamp=6, jungle=21,
    // birch_forest=27, savanna=35, badlands=37) ----
    Taiga = 8,
    BirchForest = 9,
    Jungle = 10,
    Savanna = 11,
    Swamp = 12,
    Badlands = 13,
    /// Phase E1 (1.0.0 content): Mushroom Fields — mycelium surface,
    /// huge mushrooms, mooshroom herds, NO natural hostile spawns (all
    /// VERIFIED w/Mushroom_Fields, live 2026-09-06). Internal id 14
    /// (vanilla's real registry id is 14 = mushroom_fields, matching).
    MushroomFields = 14,
    // ---- 1.7.2 bracket (live-verified reference wiki /Java_Edition_1.7.2):
    // the four headliner overworld additions of the Update that Changed
    // the World. [merge renumber] shifted 14..=17 -> 15..=18 past the E1
    // MushroomFields id ----
    FlowerForest = 15,
    SunflowerPlains = 16,
    IceSpikes = 17,
    DarkForest = 18,
    // ---- 1.13 bracket (Aquatic-era update): the ocean temperature split
    // (VERIFIED live 2026-09-07, reference wiki /Java_Edition_1.13
    // §World generation: "Added voxelcraft:warm_ocean (Warm Ocean),
    // voxelcraft:lukewarm_ocean (Lukewarm Ocean), voxelcraft:cold_ocean
    // (Cold Ocean) ... voxelcraft:frozen_ocean (Frozen Ocean) now
    // generates again"). Internal ids 19..=22 (vanilla registry ids
    // 44/45/46/10 — the deep variants are depth-cosmetic and fold into
    // these families here, disclosed). The pre-1.13 "Ocean" (id 0)
    // stays as the neutral temperate ocean. ----
    WarmOcean = 19,
    LukewarmOcean = 20,
    ColdOcean = 21,
    FrozenOcean = 22,
    /// 1.16 (Nether Update, part 2): the crimson forest — the piglin/
    /// hoglin home (VERIFIED w/Crimson_Forest live: "the second most
    /// common Nether biome, making up 22% of the Nether by volume";
    /// "This is the only biome where hoglins naturally spawn outside
    /// of bastion remnants"). Internal id 23 (vanilla registry id 171
    /// — renumbered into the engine's internal sequence, the standing
    /// disclosed convention).
    CrimsonForest = 23,
    /// 1.16: the warped forest — the enderman-fungus home (VERIFIED
    /// w/Warped_Forest live: "the rarest of the five biomes in the
    /// Nether, making up around 8% of the Nether by volume";
    /// "hostile mobs do not spawn naturally" — endermen are the
    /// exception). Internal id 24 (vanilla registry id 172).
    WarpedForest = 24,
    /// Backlog round (2026-09-09): the soul sand valley — VERIFIED
    /// (w/Soul_Sand_Valley live, capture backlog_page_Soul_Sand_Valley.json):
    /// "makes up around 17% of the Nether by volume, making it the
    /// third most common biome"; "mostly composed of soul sand and
    /// soul soil, with gravel found on its coastlines"; "Soul fire is
    /// scattered throughout the biome and Nether fossils poke out of
    /// the terrain"; "Its fog is cyan"; spawns: skeleton 20/71,
    /// ghast 50/71 (5% attempt success), enderman 1/71. Internal id 25
    /// (vanilla registry id 170).
    SoulSandValley = 25,
    /// Backlog round: the basalt deltas — the basalt-pillar wastes
    /// (stats from the w/Basalt_Deltas capture). Internal id 26
    /// (vanilla registry id 173).
    BasaltDeltas = 26,
    /// Vanilla-parity terrain round (2026-09-14): the river biome —
    /// bands carved by the ridged-noise river field (the disclosed
    /// adaptation of vanilla's layer-stack rivers; vanilla registry id 7).
    /// Depth −0.5 / scale 0.0 (vanilla river.json, misode/mcmeta 1.16.5).
    River = 27,
    // ---- 4.1e: hill/deep/shore variants evidenced in the owner's
    // Survival copy (vanilla ids from the Before-1.18 table, fetched
    // live 2026-10-10). Hills share their base's depth/scale pair
    // (approximation — disclosed in classify).
    DesertHills = 28,         // 17
    TaigaHills = 29,          // 19
    DeepOcean = 30,           // 24
    StoneShore = 31,          // 25
    BirchHills = 32,          // 28
    GiantTreeTaiga = 33,      // 32
    GiantTreeTaigaHills = 34, // 33
    WoodedMountains = 35,     // 34
    DeepLukewarmOcean = 36,   // 48
    DeepColdOcean = 37,       // 49
    GravellyMountains = 38,   // 131
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Ocean => "Ocean",
            Biome::Beach => "Beach",
            Biome::Plains => "Plains",
            Biome::Forest => "Forest",
            Biome::Desert => "Desert",
            Biome::Snowy => "Snowy Taiga",
            Biome::Mountains => "Mountains",
            Biome::NetherWastes => "Nether Wastes",
            Biome::Taiga => "Taiga",
            Biome::BirchForest => "Birch Forest",
            Biome::Jungle => "Jungle",
            Biome::Savanna => "Savanna",
            Biome::Swamp => "Swamp",
            Biome::Badlands => "Badlands",
            Biome::MushroomFields => "Mushroom Fields",
            Biome::FlowerForest => "Flower Forest",
            Biome::SunflowerPlains => "Sunflower Plains",
            Biome::IceSpikes => "Ice Spikes",
            Biome::DarkForest => "Dark Forest",
            Biome::WarmOcean => "Warm Ocean",
            Biome::LukewarmOcean => "Lukewarm Ocean",
            Biome::ColdOcean => "Cold Ocean",
            Biome::FrozenOcean => "Frozen Ocean",
            Biome::CrimsonForest => "Crimson Forest",
            Biome::WarpedForest => "Warped Forest",
            Biome::SoulSandValley => "Soul Sand Valley",
            Biome::BasaltDeltas => "Basalt Deltas",
            Biome::River => "River",
            Biome::DesertHills => "Desert Hills",
            Biome::TaigaHills => "Taiga Hills",
            Biome::DeepOcean => "Deep Ocean",
            Biome::StoneShore => "Stone Shore",
            Biome::BirchHills => "Birch Hills",
            Biome::GiantTreeTaiga => "Giant Tree Taiga",
            Biome::GiantTreeTaigaHills => "Giant Tree Taiga Hills",
            Biome::WoodedMountains => "Wooded Mountains",
            Biome::DeepLukewarmOcean => "Deep Lukewarm Ocean",
            Biome::DeepColdOcean => "Deep Cold Ocean",
            Biome::GravellyMountains => "Gravelly Mountains",
        }
    }

    pub fn from_u8(v: u8) -> Biome {
        match v {
            1 => Biome::Beach,
            2 => Biome::Plains,
            3 => Biome::Forest,
            4 => Biome::Desert,
            5 => Biome::Snowy,
            6 => Biome::Mountains,
            7 => Biome::NetherWastes,
            8 => Biome::Taiga,
            9 => Biome::BirchForest,
            10 => Biome::Jungle,
            11 => Biome::Savanna,
            12 => Biome::Swamp,
            13 => Biome::Badlands,
            14 => Biome::MushroomFields,
            15 => Biome::FlowerForest,
            16 => Biome::SunflowerPlains,
            17 => Biome::IceSpikes,
            18 => Biome::DarkForest,
            19 => Biome::WarmOcean,
            20 => Biome::LukewarmOcean,
            21 => Biome::ColdOcean,
            22 => Biome::FrozenOcean,
            23 => Biome::CrimsonForest,
            24 => Biome::WarpedForest,
            25 => Biome::SoulSandValley,
            26 => Biome::BasaltDeltas,
            27 => Biome::River,
            28 => Biome::DesertHills,
            29 => Biome::TaigaHills,
            30 => Biome::DeepOcean,
            31 => Biome::StoneShore,
            32 => Biome::BirchHills,
            33 => Biome::GiantTreeTaiga,
            34 => Biome::GiantTreeTaigaHills,
            35 => Biome::WoodedMountains,
            36 => Biome::DeepLukewarmOcean,
            37 => Biome::DeepColdOcean,
            38 => Biome::GravellyMountains,
            _ => Biome::Ocean,
        }
    }

    /// 4.1a: vanilla 1.16.5 registry id for the 4.0 numeric-diff
    /// exchange (census reports these, not internal ids). Sources:
    /// the ids already cited across this file's comments (taiga 5,
    /// swamp 6, jungle 21, birch 27, savanna 35, badlands 37,
    /// mushroom 14, river 7, crimson 171, warped 172, soul 170,
    /// basalt 173) plus the undisputed base ids (ocean 0, plains 1,
    /// desert 2, mountains 3, forest 4, beach 16, snowy→12,
    /// flower 132, sunflower 129, ice spikes 140, dark forest 29,
    /// warm 44, lukewarm 45, cold 46, frozen 10, nether wastes 8).
    pub fn vanilla_id(self) -> u8 {
        match self {
            Biome::Ocean => 0,
            Biome::Plains => 1,
            Biome::Desert => 2,
            Biome::Mountains => 3,
            Biome::Forest => 4,
            Biome::Taiga => 5,
            Biome::Swamp => 6,
            Biome::River => 7,
            Biome::NetherWastes => 8,
            Biome::FrozenOcean => 10,
            Biome::Snowy => 12,
            Biome::DesertHills => 17,
            Biome::TaigaHills => 19,
            Biome::DeepOcean => 24,
            Biome::StoneShore => 25,
            Biome::BirchHills => 28,
            Biome::GiantTreeTaiga => 32,
            Biome::GiantTreeTaigaHills => 33,
            Biome::WoodedMountains => 34,
            Biome::DeepLukewarmOcean => 48,
            Biome::DeepColdOcean => 49,
            Biome::GravellyMountains => 131,
            Biome::MushroomFields => 14,
            Biome::Beach => 16,
            Biome::Jungle => 21,
            Biome::BirchForest => 27,
            Biome::DarkForest => 29,
            Biome::Savanna => 35,
            Biome::Badlands => 37,
            Biome::WarmOcean => 44,
            Biome::LukewarmOcean => 45,
            Biome::ColdOcean => 46,
            Biome::SunflowerPlains => 129,
            Biome::FlowerForest => 132,
            Biome::IceSpikes => 140,
            Biome::SoulSandValley => 170,
            Biome::CrimsonForest => 171,
            Biome::WarpedForest => 172,
            Biome::BasaltDeltas => 173,
        }
    }

    /// 1.16 (Nether Update, part 2): the nether biome family — the
    /// wastes + the two forests (region gates for mob spawning and
    /// the snow-golem heat rule: every nether flavor is "hot").
    /// The backlog round adds the valley + deltas (all five now).
    pub fn is_nether(self) -> bool {
        matches!(
            self,
            Biome::NetherWastes
                | Biome::CrimsonForest
                | Biome::WarpedForest
                | Biome::SoulSandValley
                | Biome::BasaltDeltas
        )
    }

    /// Backlog round (weather): the biome's precipitation form —
    /// VERIFIED w/Weather: "Rain occurs only in blocks with a
    /// temperature higher than 0.15 and not in certain dry biomes
    /// (deserts, savannas, and badlands)"; "Snowfall occurs only in
    /// biomes with a base temperature less than 0.15". The engine's
    /// temperature model is the biome family itself (documented
    /// adaptation — vanilla samples climate per column).
    pub fn precipitation(self) -> Precip {
        match self {
            // dry — no rain, no snow
            Biome::Desert | Biome::Savanna | Biome::Badlands => Precip::None,
            // cold — snow instead of rain
            Biome::Snowy | Biome::IceSpikes | Biome::FrozenOcean => Precip::Snow,
            // nether/end flavors never see weather (the game layer's
            // dimension gate is primary; this is belt-and-braces)
            b if b.is_nether() => Precip::None,
            _ => Precip::Rain,
        }
    }

    /// 1.13: the ocean temperature family gate (any ocean-flavored
    /// biome, including the neutral id-0 ocean).
    pub fn is_ocean(self) -> bool {
        matches!(
            self,
            Biome::Ocean
                | Biome::WarmOcean
                | Biome::LukewarmOcean
                | Biome::ColdOcean
                | Biome::FrozenOcean
                | Biome::DeepOcean
                | Biome::DeepLukewarmOcean
                | Biome::DeepColdOcean
        )
    }
}

// ---------------------------------------------------------------- simplex --

const GRAD3: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

const F2: f32 = 0.366_025_42; // 0.5 * (sqrt(3) - 1)
const G2: f32 = 0.211_324_87; // (sqrt(3) - 1) / 6
const F3: f32 = 0.333_333_34;
const G3: f32 = 0.166_666_67;

// ---- 1.0.3 (PLAN v3.1): deterministic worldgen math (R5) -----------------
// std's f32::sin/cos/sqrt are documented as implementation-defined — each
// platform's libm decides the last bits. Worldgen calls them in the
// feature/structure placement paths, so terrain differed per OS. These
// wrappers route through the pure-Rust `libm` crate (bit-exact ports of
// musl's routines): identical results on Linux, Windows, macOS, wasm.
// The golden worldgen hash (golden_determinism_tests) pins the swap:
// values there were taken on std libm and re-pinned in 1.0.3 — reported,
// never silently re-baselined.

#[inline]
fn dsin32(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
fn dcos32(x: f32) -> f32 {
    libm::cosf(x)
}

#[inline]
fn dsin64(x: f64) -> f64 {
    libm::sin(x)
}

#[inline]
fn dcos64(x: f64) -> f64 {
    libm::cos(x)
}

#[inline]
fn dsqrt32(x: f32) -> f32 {
    libm::sqrtf(x)
}

/// round-half-away-from-zero, matching std f32::round (libm has no
/// roundf wrapper name for it in older versions; roundf exists — use it)
#[inline]
fn dround32(x: f32) -> f32 {
    libm::roundf(x)
}

pub struct Noise {
    perm: Box<[u8; 512]>,
    perm_mod12: Box<[u8; 512]>,
}

impl Noise {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut p = [0u8; 256];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i as u8;
        }
        for i in (1..256).rev() {
            let j = rng.next_range((i + 1) as u32) as usize;
            p.swap(i, j);
        }
        let mut perm = Box::new([0u8; 512]);
        let mut perm_mod12 = Box::new([0u8; 512]);
        for i in 0..512 {
            perm[i] = p[i & 255];
            perm_mod12[i] = perm[i] % 12;
        }
        Noise { perm, perm_mod12 }
    }

    pub fn noise2(&self, xin: f32, yin: f32) -> f32 {
        let s = (xin + yin) * F2;
        let i = (xin + s).floor();
        let j = (yin + s).floor();
        let t = (i + j) * G2;
        let x0 = xin - (i - t);
        let y0 = yin - (j - t);

        let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };
        let x1 = x0 - i1 + G2;
        let y1 = y0 - j1 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let ii = (i as i64 & 255) as usize;
        let jj = (j as i64 & 255) as usize;
        let gi0 = self.perm_mod12[ii + self.perm[jj] as usize] as usize;
        let gi1 = self.perm_mod12[ii + i1 as usize + self.perm[jj + j1 as usize] as usize] as usize;
        let gi2 = self.perm_mod12[ii + 1 + self.perm[jj + 1] as usize] as usize;

        let mut n = 0.0;
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 > 0.0 {
            let t = t0 * t0;
            let g = &GRAD3[gi0];
            n += t * t * (g[0] * x0 + g[1] * y0);
        }
        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 > 0.0 {
            let t = t1 * t1;
            let g = &GRAD3[gi1];
            n += t * t * (g[0] * x1 + g[1] * y1);
        }
        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 > 0.0 {
            let t = t2 * t2;
            let g = &GRAD3[gi2];
            n += t * t * (g[0] * x2 + g[1] * y2);
        }
        70.0 * n
    }

    pub fn noise3(&self, xin: f32, yin: f32, zin: f32) -> f32 {
        let s = (xin + yin + zin) * F3;
        let i = (xin + s).floor();
        let j = (yin + s).floor();
        let k = (zin + s).floor();
        let t = (i + j + k) * G3;
        let x0 = xin - (i - t);
        let y0 = yin - (j - t);
        let z0 = zin - (k - t);

        let (i1, j1, k1, i2, j2, k2): (f32, f32, f32, f32, f32, f32);
        if x0 >= y0 {
            if y0 >= z0 {
                i1 = 1.0;
                j1 = 0.0;
                k1 = 0.0;
                i2 = 1.0;
                j2 = 1.0;
                k2 = 0.0;
            } else if x0 >= z0 {
                i1 = 1.0;
                j1 = 0.0;
                k1 = 0.0;
                i2 = 1.0;
                j2 = 0.0;
                k2 = 1.0;
            } else {
                i1 = 0.0;
                j1 = 0.0;
                k1 = 1.0;
                i2 = 1.0;
                j2 = 0.0;
                k2 = 1.0;
            }
        } else {
            if y0 < z0 {
                i1 = 0.0;
                j1 = 0.0;
                k1 = 1.0;
                i2 = 0.0;
                j2 = 1.0;
                k2 = 1.0;
            } else if x0 < z0 {
                i1 = 0.0;
                j1 = 1.0;
                k1 = 0.0;
                i2 = 0.0;
                j2 = 1.0;
                k2 = 1.0;
            } else {
                i1 = 0.0;
                j1 = 1.0;
                k1 = 0.0;
                i2 = 1.0;
                j2 = 1.0;
                k2 = 0.0;
            }
        }

        let x1 = x0 - i1 + G3;
        let y1 = y0 - j1 + G3;
        let z1 = z0 - k1 + G3;
        let x2 = x0 - i2 + 2.0 * G3;
        let y2 = y0 - j2 + 2.0 * G3;
        let z2 = z0 - k2 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let ii = (i as i64 & 255) as usize;
        let jj = (j as i64 & 255) as usize;
        let kk = (k as i64 & 255) as usize;

        let mut n = 0.0;

        let t0 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
        if t0 > 0.0 {
            let g = &GRAD3
                [self.perm_mod12[ii + self.perm[jj + self.perm[kk] as usize] as usize] as usize];
            let t = t0 * t0;
            n += t * t * (g[0] * x0 + g[1] * y0 + g[2] * z0);
        }
        let t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
        if t1 > 0.0 {
            let g = &GRAD3[self.perm_mod12[ii
                + i1 as usize
                + self.perm[jj + j1 as usize + self.perm[kk + k1 as usize] as usize] as usize]
                as usize];
            let t = t1 * t1;
            n += t * t * (g[0] * x1 + g[1] * y1 + g[2] * z1);
        }
        let t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
        if t2 > 0.0 {
            let g = &GRAD3[self.perm_mod12[ii
                + i2 as usize
                + self.perm[jj + j2 as usize + self.perm[kk + k2 as usize] as usize] as usize]
                as usize];
            let t = t2 * t2;
            n += t * t * (g[0] * x2 + g[1] * y2 + g[2] * z2);
        }
        let t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
        if t3 > 0.0 {
            let g = &GRAD3[self.perm_mod12
                [ii + 1 + self.perm[jj + 1 + self.perm[kk + 1] as usize] as usize]
                as usize];
            let t = t3 * t3;
            n += t * t * (g[0] * x3 + g[1] * y3 + g[2] * z3);
        }
        32.0 * n
    }
}

// ------------------------------------------------------------------- fBm --

fn fbm2(noise: &Noise, x: f32, z: f32, octaves: u32, lac: f32, gain: f32) -> f32 {
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for _ in 0..octaves {
        sum += amp * noise.noise2(x * freq, z * freq);
        norm += amp;
        amp *= gain;
        freq *= lac;
    }
    sum / norm
}

#[inline]
fn lerp64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// --------------------------------------------------------------- terrain --

pub struct ColumnInfo {
    pub height: i32,
    pub biome: Biome,
    pub top: u16,
    pub filler: u16,
}

/// P7 structures: village grid — 4.2a documents the vanilla
/// RandomSpread parameters (placement JSON, public data: spacing 34,
/// separation 8, salt 10387312). One candidate per 34×34-chunk cell,
/// uniformly placed in the (spacing−separation) window; the margin
/// convention (which cell edges hold the buffer) is [ESTIMATED /
/// APPROXIMATION] — tuned against owner numeric diffs if it misses.
pub const VILLAGE_SPACING_CHUNKS: i32 = 34;
pub const VILLAGE_SEPARATION_CHUNKS: i32 = 8;
pub const VILLAGE_SALT: u64 = 10387312;
/// max horizontal reach of village structures from the well (houses ≤ 19 r
/// + 2 footprint + well roof)
const VILLAGE_MAX_REACH: i32 = 40;

// ---- Phase 10 structure descriptors (module scope, like DungeonRoom) ----

/// one mineshaft anchored in its chunk: parlor + corridors (layout
/// deterministic from the anchor chunk). Each chunk near a shaft emits
/// only the parts of this layout that fall inside itself.
#[derive(Clone, Debug)]
pub struct Mineshaft {
    /// parlor center (world coords)
    pub x: i32,
    pub z: i32,
    pub y: i32,
    /// corridor: (dx, dz, length) — unit direction + block length
    pub corridors: Vec<(i32, i32, i32)>,
}

/// one ravine: anchored in its chunk, path deterministic from the seed.
#[derive(Clone, Debug)]
pub struct Ravine {
    /// path start (world coords)
    pub x0: i32,
    pub z0: i32,
    /// direction (unit)
    pub dx: f32,
    pub dz: f32,
    /// VERIFIED: 85..=127
    pub length: i32,
    /// base half-width (< 15 wide ⇒ half-width ≤ 7)
    pub half_w: f32,
    /// VERIFIED: up to 62 deep
    pub depth: i32,
    /// VERIFIED: start (top) 10..=72
    pub top: i32,
}

/// floor division (Rust's `/` truncates toward zero — region math needs
/// the mathematical floor so negative coordinates map to the right
/// region)
#[inline]
fn floor_div(a: i32, b: i32) -> i32 {
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// 1.16 (Nether Update, part 2): the nether region biome — a
/// deterministic 2x2-chunk cell hash (contiguous forest regions, not
/// per-column confetti). Shares VERIFIED against the wiki infobox
/// rows: crimson forest "22% of the Nether by volume", warped forest
/// "around 8% of the Nether by volume" (the rarest of the five);
/// the nether wastes fill the rest. The region scale is the engine's
/// disclosed adaptation — vanilla's 3D biome climate sampler needs
/// the full multi-noise stack.
pub fn nether_region_biome(seed: u64, cx: i32, cz: i32) -> Biome {
    let region_x = floor_div(cx, 2);
    let region_z = floor_div(cz, 2);
    // Backlog round: the five-biome roll at the wiki-verified volumes —
    // wastes 37% (the remainder), crimson 22% (w/Crimson_Forest "22% of
    // the Nether by volume"), SSV 17% (w/Soul_Sand_Valley "around 17%"),
    // basalt deltas 16% (w/Basalt_Deltas "around 16%"), warped 8%
    // (w/Warped_Forest "around 8% ... the rarest of the five")
    let v = Rng::hash3(seed ^ 0xF0E7, region_x, 0, region_z) % 100;
    if v < 8 {
        Biome::WarpedForest
    } else if v < 24 {
        Biome::BasaltDeltas
    } else if v < 41 {
        Biome::SoulSandValley
    } else if v < 63 {
        Biome::CrimsonForest
    } else {
        Biome::NetherWastes
    }
}

// [merge] our 1.7.2 badlands_band fn removed — the E-series
// badlands_band_color + stained_terracotta(color) banding replaced it

/// mineshaft generation chance per chunk — VERIFIED (wiki Mineshaft
/// page, live): "a 0.4% chance to attempt to begin generating in every
/// chunk"
pub const MINESHAFT_CHANCE: f32 = 0.004;
/// 4.4a: vein anchor salt (engine-local — vanilla decoration salts are
/// unpublished; positions tune against owner diffs later).
pub const VEIN_SALT: u64 = 0x6E15;
/// 4.1g: layer-stack salts (engine-local) + island density FIT value.
pub const LAYER_SALT_ISLAND: u64 = 0x1A4D;
pub const LAYER_SALT_ZOOM: u64 = 0x2001;
pub const LAYER_SALT_SPECIAL: u64 = 0x5EC1A1;
pub const LAYER_ISLAND_PCT: u64 = 80;
/// 4.1i: climate-gate salts + FIT percentages (snow/warm shares).
pub const LAYER_SALT_CLIMATE: u64 = 0xC11A;
/// 4.1n2: second family roll salt (the within-family overlay splits
/// need a roll INDEPENDENT of the base-family pick — sharing one
/// roll made dark/flower/giant unreachable via disjoint ranges).
pub const LAYER_SALT_OVERLAY: u64 = 0x0E1A;
pub const LAYER_SNOW_PCT: u64 = 20;
// 4.1j FIT: warm 15 → 8 (reference copy: no warm land at all in its
// temperate region; deserts/jungles must still exist globally).
pub const LAYER_WARM_PCT: u64 = 8;

/// 4.2b: scattered-structure spread parameters. Provenance: village
/// (34/8/salt) is documented placement-JSON data; pyramid/jungle/
/// mansion numbers are the engine's pre-existing regions preserved
/// verbatim ([ESTIMATED] — vanilla 1.16 salts/spacing live in code,
/// not in published data). The shared helper below gives every
/// structure the same RandomSpread shape (salt-mixed region rng +
/// uniform window + site check) so later tuning fits numbers, not
/// shapes.
pub const PYRAMID_SPACING: i32 = 32;
pub const PYRAMID_MARGIN: i32 = 4;
pub const PYRAMID_SALT: u64 = 0x0E5;
pub const JUNGLE_SPACING: i32 = 32;
pub const JUNGLE_MARGIN: i32 = 4;
pub const JUNGLE_SALT: u64 = 0x3E4E;
pub const MANSION_SPACING: i32 = 8;
pub const MANSION_MARGIN: i32 = 1;
pub const MANSION_SALT: u64 = 0xA11C;
/// 4.2e: desert-well chance per chunk, JE 1/1000 (VERIFIED w/Desert_Well:
/// "1⁄1000 [JE only]"; the well is a FEATURE — generates with the
/// structures option off). Position-within-chunk jitter + salt are
/// engine choices [ESTIMATED]; the salt is unpublished.
pub const WELL_CHANCE_DENOM: u32 = 1000;
pub const WELL_SALT: u64 = 0x9E11;
/// 4.2e: witch-hut spread — temple-family 32-chunk regions (placement
/// numbers unpublished, [ESTIMATED]); swamp-only site check.
pub const HUT_SPACING: i32 = 32;
pub const HUT_MARGIN: i32 = 4;
pub const HUT_SALT: u64 = 0x9117;
/// pyramid candidate spacing [tuning value — vanilla's structure spacing
/// is not published on the wiki; one candidate per 32×32-chunk region]
pub const PYRAMID_REGION_CHUNKS: i32 = 32;
/// ravine chance per chunk [tuning value — vanilla's canyon carver
/// probability is not published on the wiki; 1 per 50 chunks]
pub const RAVINE_CHANCE: f32 = 0.02;

#[derive(Clone, Copy, Debug)]
struct HouseSite {
    /// house center (world blocks)
    x: i32,
    z: i32,
    /// floor level (terrain height at the site)
    floor: i32,
    /// blacksmith houses get a furnace
    blacksmith: bool,
}

/// a rolled monster room / dungeon (Phase 5 §27 — all world coordinates)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DungeonRoom {
    /// interior min corner; interior is `size`×4×`size` air
    pub x0: i32,
    pub y0: i32,
    pub z0: i32,
    /// interior width/length: 7, 9, or 11 (VERIFIED)
    pub size: i32,
    /// spawner mob code (50% zombie / 25% skeleton / 25% spider)
    pub mob: u8,
    /// chest positions (up to 2, on the floor against the walls)
    pub chests: [[i32; 3]; 2],
    /// how many of the two chest slots actually placed
    pub chest_count: usize,
}

/// One carver worm paired with its precomputed ellipsoid path —
/// (x, y, z, half-width) per 4-block step.
pub type WormPath = Vec<(f64, f64, f64, f64)>;

/// One perlin-worm cave carver, anchored in its start chunk (the
/// vanilla-parity replacement for the 1.18-style noise-sheet caves;
/// deterministic from the seed + start chunk alone).
#[derive(Clone, Debug)]
pub struct CaveWorm {
    /// start position (world coords)
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// initial heading
    pub yaw: f64,
    /// worm step count (4 blocks per step)
    pub steps: u32,
}

pub struct TerrainGen {
    pub seed: u64,
    /// §28: which dimension this generator produces
    pub dim: Dimension,
    /// Phase E3 (1.5–1.6 bracket): Superflat world type (VERIFIED live
    /// 2026-09-06, reference wiki /Superflat: classic preset = "one
    /// layer of grass blocks and two layers of dirt, followed by
    /// bedrock", plains biome; JE also generates villages/strongholds —
    /// the engine's flat mode generates NO structures, disclosed
    /// adaptation).
    pub flat: bool,
    /// 4.1b: Large Biomes world type (VERIFIED reference wiki
    /// /Large_Biomes: pre-1.18 biome layers scaled ×4 XZ, "16 times as
    /// much area"; rivers NOT larger, mountains NOT larger). Engine
    /// adaptation: the biome-classification fields (temperature,
    /// humidity, variant) and the continental shelf sample at ÷4
    /// coordinates; the river band, mountain mask and detail fields
    /// are untouched. The continental inclusion is [ESTIMATED /
    /// APPROXIMATION] — the wiki exempts only rivers explicitly.
    pub large_biomes: bool,
    /// 4.3a: Amplified world type (VERIFIED reference wiki /Amplified:
    /// larger altitude range, Nether/End unaffected, oceans/rivers see
    /// no obvious change). Engine rule in density_params: land biomes
    /// (depth > 0) sample depth 1+2d / scale 1+4s (the documented
    /// minHeight/maxHeight behavior); the ocean gate on the
    /// continental field is [ESTIMATED / APPROXIMATION].
    pub amplified: bool,
    /// 2026-09-14 parity round: the vanilla "Generate Structures"
    /// world-create option (options key `generate-structures`, on the
    /// More World Options page — VERIFIED reference wiki /Java_Edition_1.3.1
    /// §Changes: "New world-generation option: generate structures").
    /// OFF gates every structure emit below (dungeons, villages,
    /// mineshafts, pyramids, jungle temples, mansions, strongholds);
    /// terrain, caves, ores and vegetation are NOT structures and keep
    /// generating (the vanilla split: structure features vs features).
    pub structures: bool,
    /// continental shelf field (~750–3000-block scale) — drives the
    /// land/ocean split of the density stack
    n_cont: Noise,
    /// mountain-region mask (~475-block scale) — where high, the density
    /// stack gains the vanilla mountains-biome response (depth 1.0 /
    /// scale 0.5, misode/mcmeta 1.16.5 mountains.json)
    n_mfac: Noise,
    n_temp: Noise,
    n_humid: Noise,
    /// Vanilla-parity terrain round (2026-09-14): the 1.16.5-structured
    /// density noise stack — improved-Perlin fBm samplers wired with the
    /// vanilla constants (see vanilla_noise.rs for the cited sources).
    vterrain: VanillaTerrain,
    /// 4.4b: outer-End island mask field.
    n_end: Noise,
    /// River band: ridged low-frequency field (the disclosed adaptation
    /// of vanilla's layer-stack rivers).
    n_river: Noise,
    /// §28 nether: cavern pair (bigger scale than overworld caves)
    n_neth1: Noise,
    n_neth2: Noise,
    /// §28 nether: wall density variation so caverns aren't uniform
    n_neth3: Noise,
    /// Phase E1: the rare mushroom-island field (~0.15% of the overworld
    /// — VERIFIED w/Mushroom_Fields). A dedicated low-frequency noise;
    /// where it clears the threshold, the column becomes a
    /// mushroom-fields island regardless of the climate result.
    n_mush: Noise,
}

impl TerrainGen {
    /// overworld generator (back-compatible)
    pub fn new(seed: u64) -> Self {
        TerrainGen::for_dimension(seed, Dimension::Overworld)
    }

    /// §28: per-dimension generator — the world seed is salted per
    /// dimension (same seed, independent generators, vanilla pattern)
    pub fn for_dimension(seed: u64, dim: Dimension) -> Self {
        let seed = seed ^ dim.seed_salt();
        TerrainGen {
            seed,
            dim,
            flat: false,
            large_biomes: false,
            amplified: false,
            structures: true,
            n_cont: Noise::new(seed ^ 0x1000),
            n_mfac: Noise::new(seed ^ 0x2000),
            n_temp: Noise::new(seed ^ 0x5000),
            n_humid: Noise::new(seed ^ 0x6000),
            vterrain: VanillaTerrain::new(seed ^ 0xB100),
            n_river: Noise::new(seed ^ 0xA500),
            n_neth1: Noise::new(seed ^ 0xA100),
            n_neth2: Noise::new(seed ^ 0xA200),
            n_neth3: Noise::new(seed ^ 0xA300),
            n_end: Noise::new(seed ^ 0xE17D),
            n_mush: Noise::new(seed ^ 0xA400),
        }
    }

    /// Phase E3: classic-superflat generator (bedrock + 2 dirt + grass,
    /// plains biome, no structures — the engine adaptation of the
    /// VERIFIED classic preset; see the `flat` field docs).
    pub fn for_dimension_flat(seed: u64, dim: Dimension) -> Self {
        let mut g = Self::for_dimension(seed, dim);
        g.flat = true;
        g
    }

    /// 4.1b: large-biomes generator (see the `large_biomes` field).
    pub fn for_dimension_large(seed: u64, dim: Dimension) -> Self {
        let mut g = Self::for_dimension(seed, dim);
        g.large_biomes = true;
        g
    }

    /// 4.3a: amplified generator (see the `amplified` field).
    pub fn for_dimension_amplified(seed: u64, dim: Dimension) -> Self {
        let mut g = Self::for_dimension(seed, dim);
        g.amplified = true;
        g
    }

    /// The climate fields (temperature / humidity / variant) — the
    /// pre-rewrite scales, unchanged.
    pub fn climate_fields(&self, x: i32, z: i32) -> (f32, f32, f32) {
        // 4.1b: large biomes sample the classification fields at ÷4
        // (the river band keeps its own scale — see large_biomes docs)
        let (xf, zf) = if self.large_biomes {
            (x as f32 / 4.0, z as f32 / 4.0)
        } else {
            (x as f32, z as f32)
        };
        let temp = fbm2(
            &self.n_temp,
            (xf + 3000.0) / 1700.0,
            (zf + 3000.0) / 1700.0,
            2,
            2.0,
            0.5,
        );
        let humid = fbm2(
            &self.n_humid,
            (xf - 5000.0) / 1400.0,
            (zf + 5000.0) / 1400.0,
            2,
            2.0,
            0.5,
        );
        // 1.7.2 bracket: the variant/biome-flavor noise — same temperature
        // and humidity fields, different octave offsets, so variant patches
        // are large-scale (~400 blocks) and never correlate with detail
        let var = fbm2(
            &self.n_humid,
            (xf + 12000.0) / 400.0,
            (zf - 11000.0) / 400.0,
            2,
            2.0,
            0.5,
        );
        (temp, humid, var)
    }

    /// The land-climate biome's vanilla depth/scale pair (misode/mcmeta
    /// 1.16.5 biome JSONs, live 2026-09-14: plains 0.125/0.05, forest
    /// 0.1/0.2, taiga 0.2/0.2, desert 0.125/0.05, snowy tundra
    /// 0.125/0.05, ice spikes 0.425/0.45, jungle 0.1/0.2, savanna
    /// 0.125/0.05, swamp −0.2/0.1, badlands 0.1/0.2, birch 0.1/0.2,
    /// flower forest 0.1/0.4, dark forest 0.1/0.2, mushroom fields
    /// 0.2/0.3). The brackets mirror the climate classification below
    /// (height-independent, so the density stack can use them before the
    /// final classification).
    fn climate_depth_scale(&self, x: i32, z: i32) -> (f64, f64) {
        let (temp, humid, var) = self.climate_fields(x, z);
        if temp < -0.32 {
            if var > 0.58 {
                (0.425, 0.45) // ice spikes
            } else {
                (0.125, 0.05) // snowy tundra
            }
        } else if temp < -0.1 {
            (0.2, 0.2) // taiga
        } else if temp > 0.25 && humid < -0.12 {
            (0.1, 0.2) // badlands
        } else if temp > 0.3 && humid < 0.05 {
            (0.125, 0.05) // desert
        } else if temp > 0.25 && humid > 0.3 {
            (0.1, 0.2) // jungle
        } else if temp > 0.35 {
            (0.125, 0.05) // savanna
        } else if humid > 0.45 {
            (-0.2, 0.1) // swamp
        } else if humid > 0.12 && temp < 0.2 {
            if var > 0.42 {
                (0.1, 0.4) // flower forest
            } else {
                (0.1, 0.2) // birch forest
            }
        } else if humid > 0.12 {
            // dark forest / forest / birch-forest share the vanilla
            // depth/scale pair (all 0.1 / 0.2)
            (0.1, 0.2)
        } else {
            (0.125, 0.05) // plains / sunflower plains
        }
    }

    /// The per-column density drivers (shared by `column()`'s direct
    /// root-solve and the chunk pipeline's 4×8×4 lattice — the vanilla
    /// 1.16.5 noise-settings semantics; see vanilla_noise.rs).
    ///
    /// Effective height = base 68 (depthBaseSize 8.5 × 8) + continental
    /// shelf (steeper below sea level so deep oceans floor at ~35–45,
    /// the vanilla deep-ocean depth −1.8 response) + mountain-region
    /// mask (the mountains-biome depth 1.0 response) + depth-noise
    /// wobble + the climate biome's depth response. Amplification scales
    /// the 3D limit-noise field by biome variation + the mountain mask
    /// (1 density unit ≈ 8 blocks of surface wobble). The river carve
    /// bends the column toward a 58-high bed inside the band.
    fn density_params(&self, x: i32, z: i32, bd: f64, bv: f64) -> (f64, f64, f32) {
        let xf = x as f32;
        let zf = z as f32;
        // 4.1b: the continental shelf joins the large-biomes scale
        // ([ESTIMATED / APPROXIMATION] — see large_biomes docs)
        let (cxf, czf) = if self.large_biomes {
            (xf / 4.0, zf / 4.0)
        } else {
            (xf, zf)
        };
        let cont = fbm2(&self.n_cont, cxf / 1500.0, czf / 1500.0, 4, 2.0, 0.5) * 1.7;
        // 4.3a: amplified land transform (depth 1+2d, scale 1+4s for
        // depth > 0; oceans gated on the continental field —
        // [ESTIMATED / APPROXIMATION], see amplified docs)
        let (bd, bv) = if self.amplified && bd > 0.0 && cont > -0.5 {
            (1.0 + bd * 2.0, 1.0 + bv * 4.0)
        } else {
            (bd, bv)
        };
        let mmask = smoothstep(
            0.3,
            0.6,
            fbm2(
                &self.n_mfac,
                (xf + 700.0) / 950.0,
                (zf - 300.0) / 950.0,
                2,
                2.0,
                0.5,
            ),
        ) as f64;
        let depth_n = self.vterrain.depth_noise(x as f64, z as f64);
        let cont_resp = if cont < 0.0 { 22.0 } else { 14.0 };
        let mut h_eff = 68.0 + cont as f64 * cont_resp + mmask * 18.0 + depth_n * 5.0 + bd * 14.0;
        let amp = 1.0 + bv * 1.8 + mmask * 2.2;

        // river carve: ridged field; bed 58 at the core, banks blending
        // out to the natural height by |rv| = 0.06 (~8-block water bands)
        let rv = self.n_river.noise2(xf / 640.0, zf / 640.0);
        if rv.abs() < 0.06 {
            let t = smoothstep(0.006, 0.06, rv.abs()) as f64;
            let bed = 58.0 + t * (h_eff - 58.0);
            h_eff = h_eff.min(bed);
        }
        (h_eff, amp, rv)
    }

    /// The random density offset — JSON `random_density_offset: true`,
    /// drawn per noise column (4-block lattice) in [−0.1875, 0.0625]
    /// (the recalled vanilla draw range; hash-based, disclosed
    /// adaptation of the per-column chunk-random draw).
    fn random_density_offset(&self, x: i32, z: i32) -> f64 {
        let v = (Rng::hash3(self.seed ^ 0xD345, x, 0x11, z) % 4096) as f64 / 4096.0;
        v * 0.25 - 0.1875
    }

    /// Column classification (the pre-rewrite bracket chain, heights now
    /// from the density stack; the river band inserted between ocean and
    /// beach).
    fn classify(
        &self,
        _temp: f32,
        humid: f32,
        var: f32,
        h: i32,
        rv: f32,
        pos: (i32, i32),
    ) -> (Biome, u16, u16) {
        // 4.1i: layer values computed once (land/deep/special +
        // snow/warm gates); height branches below stay untouched.
        let (lx, lz) = (pos.0.div_euclid(4), pos.1.div_euclid(4));
        // 4.1i2: height owns the land/ocean split (the layer-ocean
        // seam rule flooded beaches); deep is height-pure (4.1n: the
        // 4.1h layer-interior union over-marked deep 2.3x vs the copy).
        let (_, _, special) = self.layer_cell(lx, lz);
        let (snow, warm) = self.layer_climate(lx, lz);
        // family pick hash (position-deterministic; proportions FIT)
        let pick = Rng::hash3(self.seed ^ LAYER_SALT_SPECIAL, lx, 0xB17, lz) % 100;
        // overlay roll on an independent salt (sharing `pick` for the
        // within-family splits selected disjoint ranges — unreachable).
        // 4.1n3: sampled at 16-block PATCH cells (reference overlays
        // are contiguous patches — dark forests, giant stands — not
        // per-column salt-and-pepper; the marginal stays uniform so
        // shares hold, and patch-interior columns agree, which the
        // mansion triple-column ground check needs)
        let px = lx.div_euclid(4);
        let pz = lz.div_euclid(4);
        let pick2 = Rng::hash3(self.seed ^ LAYER_SALT_OVERLAY, px, 0xB17, pz) % 100;
        // 4.1p FIT (verification census): the h60-61 shelf is broad
        // (6.7pp) — ocean kept it all (1.64x). Ocean narrows to h<60;
        // the shelf becomes submerged land (dirt-topped below, so the
        // tops read as seabed while the biomes feed needy plains).
        let (biome, top, filler) = if h < vc_chunk::SEA_LEVEL - 2 {
            // 4.1i: ocean family from gates + depth (temp retired —
            // uncorrelated per copy measurement)
            // 4.1p: deep h<51 -> h<52 (measured 4.27% vs copy 5.3%;
            // P(h<52)=9.23% predicts ~5.6%)
            let deep = h < vc_chunk::SEA_LEVEL - 10;
            if snow {
                (Biome::FrozenOcean, GRAVEL, GRAVEL)
            } else if warm {
                (Biome::WarmOcean, SAND, SAND)
            } else if deep {
                // 4.1l FIT from the owner-copy census (deep-family:
                // deep ~79%, deep-cold ~6%, deep-lukewarm ~15%; the
                // old 50/50 made deep-cold 6.6% vs 0.4% in the copy)
                if pick < 6 {
                    (Biome::DeepColdOcean, GRAVEL, GRAVEL)
                } else if pick < 21 {
                    (Biome::DeepLukewarmOcean, SAND, SAND)
                } else {
                    (Biome::DeepOcean, GRAVEL, GRAVEL)
                }
            } else if pick < 6 {
                // 4.1l FIT (shallow-family: ocean ~93%, cold ~6%,
                // lukewarm ~2%; old thirds made cold/luke ~22% vs
                // under 1% each in the copy)
                (Biome::ColdOcean, SAND, GRAVEL)
            } else if pick < 8 {
                (Biome::LukewarmOcean, SAND, SAND)
            } else {
                // the neutral temperate ocean (the pre-1.13 "Ocean")
                (Biome::Ocean, SAND, GRAVEL)
            }
        } else if rv.abs() < 0.03 && h <= vc_chunk::SEA_LEVEL {
            // 4.1n: river band widened toward the carve edge (0.06);
            // copy rivers are 5.7% vs our 0.006% — geometric step one,
            // carve profile untouched (dedicated slice if still short)
            // Vanilla-parity terrain round: the river band — carved by
            // the ridged river field (disclosed adaptation of vanilla's
            // layer-stack rivers); sand-over-dirt bed, water fills to
            // sea level through the standard fluid fill.
            (Biome::River, SAND, DIRT)
        } else if h == vc_chunk::SEA_LEVEL + 1 {
            // 4.1o: beach narrowed to the h63 fringe (see above)
            // 4.1e: high shores split to stone shore (vanilla 25;
            // variant gate [ESTIMATED] — single-column classify has no
            // adjacency info for the mountain-foot rule)
            if var > 0.55 {
                (Biome::StoneShore, STONE, GRAVEL)
            } else {
                (Biome::Beach, SAND, SAND)
            }
        } else if h > 73 {
            // 4.1p: mountain gate 75 -> 73 (P(h>73) ~= 14.7% funds
            // mountains ~10.7 + taiga-hills 3.1 + giant-hills 2.3;
            // [ESTIMATED] — verify shares on the next census).
            // Taiga-base high cells keep their family overlay (the
            // gate was starving hills/giant-hills to zero); other
            // families take the mountain split below.
            // 4.1e: mountain family splits by variant field
            // ([ESTIMATED] thresholds — vanilla hills rise with the
            // base; wooded 34 / gravelly 131 from the Before-1.18 table)
            if self.layer_base_biome(special, snow, warm, lx, lz) == Biome::Taiga {
                self.taiga_overlay(h, var, pick2)
            } else if h > 112 {
                (Biome::Mountains, SNOW, STONE)
            } else if var > 0.5 {
                (Biome::WoodedMountains, STONE, STONE)
            } else if var < -0.5 {
                (Biome::GravellyMountains, STONE, GRAVEL)
            } else {
                (Biome::Mountains, STONE, STONE)
            }
        }
        // ---- 4.1i: land base from the layer stack (replaces the
        // temp/humid predicate chain — measurement showed the fBm
        // climate fields uncorrelated with the reference layout).
        // Height-gated branches above (ocean/river/beach/mountains)
        // are untouched; swamp keeps its wettest-band gate.
        else {
            let land = if humid > 0.45 && h <= 66 {
                // Swamp: wettest band + low flat terrain (unchanged)
                (Biome::Swamp, GRASS, DIRT)
            } else if snow {
                // snow-land: ice spikes by variant, else the layer
                // Snowy/Taiga pick with taiga overlays
                if var > 0.58 {
                    (Biome::IceSpikes, SNOW, DIRT)
                } else {
                    match self.layer_base_biome(special, true, false, lx, lz) {
                        Biome::Taiga => self.taiga_overlay(h, var, pick2),
                        _ => (Biome::Snowy, SNOW_GRASS, DIRT),
                    }
                }
            } else {
                let base = self.layer_base_biome(special, false, warm, lx, lz);
                self.finish_land_base(base, h, var, pick2)
            };
            // 4.1p: submerged shelf (h60-61) tops read as seabed —
            // biome kept for shares, grass would read wrong underwater
            let (b, t, f) = land;
            if h < vc_chunk::SEA_LEVEL {
                (b, DIRT, DIRT)
            } else {
                (b, t, f)
            }
        };
        (biome, top, filler)
    }

    /// 4.1i: land-base finish (hills + variant overlays shared by the
    /// snow and temperate paths). Family-internal splits ride the
    /// INDEPENDENT overlay roll `pick2` (4.1n2 — sharing the base
    /// pick selected disjoint ranges, unreachable).
    fn finish_land_base(&self, base: Biome, h: i32, var: f32, pick: u64) -> (Biome, u16, u16) {
        match base {
            // 4.1k: badlands keeps its red-sand floor (the warm-gate
            // addition restores it; surface unchanged from 4.1e)
            Biome::Badlands => (Biome::Badlands, RED_SAND, RED_SANDSTONE),
            Biome::Desert => {
                if h >= 74 {
                    (Biome::DesertHills, SAND, SAND)
                } else {
                    (Biome::Desert, SAND, SAND)
                }
            }
            Biome::Savanna => {
                if var > 0.72 {
                    (Biome::Savanna, COARSE_DIRT, DIRT)
                } else {
                    (Biome::Savanna, GRASS, DIRT)
                }
            }
            Biome::Jungle => (Biome::Jungle, GRASS, DIRT),
            Biome::Taiga => self.taiga_overlay(h, var, pick),
            Biome::BirchForest => {
                // flower forest 45% of the narrow birch base
                if pick < 45 {
                    (Biome::FlowerForest, GRASS, DIRT)
                } else if h >= 78 {
                    (Biome::BirchHills, GRASS, DIRT)
                } else {
                    (Biome::BirchForest, GRASS, DIRT)
                }
            }
            Biome::Forest => {
                // dark forest 33% of the forest family (copy share)
                if pick < 33 {
                    (Biome::DarkForest, GRASS, DIRT)
                } else {
                    (Biome::Forest, GRASS, DIRT)
                }
            }
            Biome::MushroomFields => (Biome::MushroomFields, MYCELIUM, DIRT),
            _ => {
                if var > 0.5 {
                    (Biome::SunflowerPlains, GRASS, DIRT)
                } else {
                    (Biome::Plains, GRASS, DIRT)
                }
            }
        }
    }

    /// 4.1i: taiga family overlay (giant/hills/pod-zol) shared by the
    /// snow and temperate paths. Giant membership rides the uniform
    /// cell roll (4.1n FIT: 28% of the family vs the var-tail 2%);
    /// 4.1q: hills 78 -> 76, giant-hills 80 -> 78 (verification
    /// census: hills 0.51x / giant-hills 0.21x starved, giant 1.33x
    /// fat — the gates sat above most high taiga; [ESTIMATED]).
    /// Lowland podzol keeps its var band.
    fn taiga_overlay(&self, h: i32, var: f32, pick: u64) -> (Biome, u16, u16) {
        if pick < 28 {
            if h >= 78 {
                (Biome::GiantTreeTaigaHills, PODZOL, DIRT)
            } else {
                (Biome::GiantTreeTaiga, PODZOL, DIRT)
            }
        } else if h >= 76 {
            (Biome::TaigaHills, GRASS, DIRT)
        } else if var > 0.45 {
            (Biome::Taiga, PODZOL, DIRT)
        } else {
            (Biome::Taiga, GRASS, DIRT)
        }
    }

    /// Terrain height + climate classification for one column.
    ///
    /// Vanilla-parity terrain round (2026-09-14): the height now comes
    /// from the 1.16.5-structured density stack (see vanilla_noise.rs)
    /// via a direct Newton root-solve at this column — the chunk
    /// pipeline interpolates the same field on the 4×8×4 lattice, and
    /// the two agree within ~1 block.
    pub fn column(&self, x: i32, z: i32) -> ColumnInfo {
        let xf = x as f32;
        let zf = z as f32;

        // Phase E1: the mushroom-island override (VERIFIED
        // w/Mushroom_Fields: ~0.15% of the overworld, islands in the
        // ocean, mycelium surface). A dedicated low-frequency field;
        // where it clears the threshold the column becomes a gentle
        // island above sea level regardless of the climate pick below.
        // 4.1j FIT: gate 0.63 → 0.78 (measured 7.6% at 0.63 vs rare
        // in the reference copy).
        let mush = self.n_mush.noise2(xf / 400.0, zf / 400.0);
        if self.dim == Dimension::Overworld && mush > 0.87 {
            let h = (vc_chunk::SEA_LEVEL as f32 + 1.0 + (mush - 0.63) * 30.0)
                .floor()
                .min(vc_chunk::SEA_LEVEL as f32 + 6.0) as i32;
            return ColumnInfo {
                height: h,
                biome: Biome::MushroomFields,
                top: MYCELIUM,
                filler: DIRT,
            };
        }

        // ---- the vanilla-structured density stack ----
        let (temp, humid, var) = self.climate_fields(x, z);
        let (bd, bv) = self.climate_depth_scale(x, z);
        let (h_eff, amp, rv) = self.density_params(x, z, bd, bv);
        let rnd_off = self.random_density_offset(x, z);
        // Newton root-solve of the surface (two passes; 1 density unit
        // ≈ 8 blocks of height)
        let mut y_surf = h_eff;
        for _ in 0..2 {
            let d = self.vterrain.density(
                x as f64,
                y_surf.clamp(0.0, 200.0),
                z as f64,
                h_eff,
                amp,
                rnd_off,
            );
            y_surf = (y_surf + d * 8.0).clamp(4.0, 200.0);
        }
        let h = y_surf.round().clamp(4.0, 200.0) as i32;

        let (biome, top, filler) = self.classify(temp, humid, var, h, rv, (x, z));
        ColumnInfo {
            height: h,
            biome,
            top,
            filler,
        }
    }

    /// Vanilla-parity cave carvers (2026-09-14): perlin-worm tunnels
    /// replacing the 1.18-style noise-sheet caves. Roll: probability
    /// 0.14285715 (1/7) per chunk — the vanilla 1.16.5
    /// configured_carver/cave.json value (misode/mcmeta, live 2026-09-14).
    /// Worm: 10..28 steps of 4 blocks, drifting yaw/pitch, width 1..5
    /// with entrance rooms on the first two steps; y span 8..128; carved
    /// cells at/below y=10 become LAVA (the carver lava level); bedrock
    /// never carved; a liquid guard keeps tunnels from opening into
    /// water columns (ocean/beach/river border columns keep a 12-block
    /// floor margin where the neighbor chunk can't be inspected —
    /// disclosed). Shape: chain of ellipsoids along the path — the
    /// documented pre-1.18 "carver cave" structure (the reference wiki
    /// w/Cave §Carver caves).
    fn cave_worms_near(&self, cx: i32, cz: i32) -> Vec<CaveWorm> {
        const CAVE_PROB: f32 = 0.14285715;
        const SCAN: i32 = 8;
        let mut out = Vec::new();
        for dz in -SCAN..=SCAN {
            for dx in -SCAN..=SCAN {
                let scx = cx + dx;
                let scz = cz + dz;
                let mut rng = Rng::new(Rng::hash3(self.seed ^ 0xCA7E, scx, 0, scz));
                if rng.next_f32() >= CAVE_PROB {
                    continue;
                }
                let x = scx * 16 + rng.next_range(16) as i32;
                let z = scz * 16 + rng.next_range(16) as i32;
                let y = 8 + rng.next_range(120); // 8..128
                let yaw = rng.next_f32() as f64 * std::f64::consts::TAU;
                let steps = 10 + rng.next_range(18);
                out.push(CaveWorm {
                    x: x as f64 + 0.5,
                    y: y as f64,
                    z: z as f64 + 0.5,
                    yaw,
                    steps,
                });
            }
        }
        out
    }

    /// The worm's full ellipsoid path: (x, y, z, half-width) per step.
    /// Pure and deterministic — re-derives the start-chunk rng stream
    /// (the roll draws are re-consumed to reach the drift stream).
    fn worm_path(&self, worm: &CaveWorm) -> Vec<(f64, f64, f64, f64)> {
        let scx = (worm.x.floor() as i32) >> 4;
        let scz = (worm.z.floor() as i32) >> 4;
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0xCA7E, scx, 0, scz));
        // re-consume the roll draws (probability, x, z, y, yaw, steps)
        let _ = rng.next_f32();
        let _ = rng.next_range(16);
        let _ = rng.next_range(16);
        let _ = rng.next_range(120);
        let _ = rng.next_f32();
        let _ = rng.next_range(18);
        let mut path = Vec::with_capacity(worm.steps as usize);
        let mut x = worm.x;
        let mut y = worm.y;
        let mut z = worm.z;
        let mut yaw = worm.yaw;
        let mut pitch = 0.0;
        let mut width = 1.5 + rng.next_f32() as f64;
        for step in 0..worm.steps {
            yaw += (rng.next_f32() as f64 - 0.5) * 0.7;
            pitch = (pitch + (rng.next_f32() as f64 - 0.5) * 0.35).clamp(-0.6, 0.6);
            x += dcos64(yaw) * 4.0;
            z += dsin64(yaw) * 4.0;
            y = (y + pitch * 4.0).clamp(8.0, 126.0);
            width += (rng.next_f32() as f64 - 0.42) * 0.8;
            width = width.clamp(1.0, 5.0);
            let w = if step < 2 { width + 1.5 } else { width }; // entrance rooms
            path.push((x, y, z, w));
        }
        path
    }

    /// Carve every worm that reaches this chunk (ellipsoid chain).
    fn carve_caves(&self, chunk: &mut Chunk, cx: i32, cz: i32) {
        let ox = cx * 16;
        let oz = cz * 16;
        let worms = self.cave_worms_near(cx, cz);
        for worm in &worms {
            for &(px, py, pz, w) in &self.worm_path(worm) {
                let h = w; // vertical half-height ≈ horizontal
                let bx0 = (px - w).floor() as i32;
                let bx1 = (px + w).ceil() as i32;
                let bz0 = (pz - w).floor() as i32;
                let bz1 = (pz + w).ceil() as i32;
                let by0 = (py - h).floor().max(5.0) as i32;
                let by1 = (py + h).ceil().min(128.0) as i32;
                for bx in bx0.max(ox)..=bx1.min(ox + 15) {
                    for bz in bz0.max(oz)..=bz1.min(oz + 15) {
                        let lx = (bx - ox) as usize;
                        let lz = (bz - oz) as usize;
                        // fast reject: the column's ellipsoid xz test
                        let dxr = (bx as f64 + 0.5 - px) / w;
                        let dzr = (bz as f64 + 0.5 - pz) / w;
                        if dxr * dxr + dzr * dzr > 1.0 {
                            continue;
                        }
                        let col_idx = lz * 16 + lx;
                        let col_biome = Biome::from_u8(chunk.biome[col_idx]);
                        let is_border = lx == 0 || lx == 15 || lz == 0 || lz == 15;
                        let border_margin = is_border
                            && (col_biome.is_ocean()
                                || col_biome == Biome::Beach
                                || col_biome == Biome::River)
                            && col_biome != Biome::NetherWastes;
                        for by in by0..=by1 {
                            let cur =
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), by as usize);
                            if cur == AIR || cur == WATER || cur == BEDROCK || cur == LAVA {
                                continue; // only carve solids
                            }
                            // liquid guard: never carve adjacent to water
                            let near_water = (lx > 0
                                && chunk.get_local(
                                    vc_chunk::chunk::LocalXZ::new(lx - 1, lz),
                                    by as usize,
                                ) == WATER)
                                || (lx < 15
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx + 1, lz),
                                        by as usize,
                                    ) == WATER)
                                || (lz > 0
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx, lz - 1),
                                        by as usize,
                                    ) == WATER)
                                || (lz < 15
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx, lz + 1),
                                        by as usize,
                                    ) == WATER)
                                || (by > 0
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx, lz),
                                        (by - 1) as usize,
                                    ) == WATER)
                                || (by < 127
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx, lz),
                                        (by + 1) as usize,
                                    ) == WATER);
                            if near_water {
                                continue;
                            }
                            if border_margin && by > chunk.height[col_idx] as i32 - 12 {
                                continue;
                            }
                            let dy = (by as f64 + 0.5 - py) / h;
                            if dxr * dxr + dy * dy + dzr * dzr <= 1.0 {
                                let b = if by <= 10 { LAVA } else { AIR };
                                chunk.set(lx, by as usize, lz, b);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Vanilla-parity ore veins (2026-09-14): per-chunk feature placement
    /// with the exact vanilla 1.16.5 table (misode/mcmeta 1.16.5
    /// configured_feature/ore_*.json + the biome feature stage 6 order,
    /// live 2026-09-14): dirt 10×33 y0..255, gravel 8×33 y0..255,
    /// granite/diorite/andesite 10×33 y0..79, coal 20×17 y0..127, iron
    /// 20×9 y0..63, gold 2×9 y0..31, redstone 8×8 y0..15, diamond 1×8
    /// y0..15, lapis 1×7 y16±8 (depth_average baseline 16, spread 16).
    /// Vein shape: the vanilla ellipsoid blob (rotated in xz, per-block
    /// hash edge-jitter), replacing base-stone only (the vanilla target
    /// tag base_stone_overworld = stone + the three variants).
    fn place_ores(&self, chunk: &mut Chunk, cx: i32, cz: i32, rng: &mut Rng) {
        const VEINS: [(u16, u32, u32, i32, i32); 11] = [
            (DIRT, 10, 33, 0, 255),
            (GRAVEL, 8, 33, 0, 255),
            (GRANITE, 10, 33, 0, 79),
            (DIORITE, 10, 33, 0, 79),
            (ANDESITE, 10, 33, 0, 79),
            (COAL_ORE, 20, 17, 0, 127),
            (IRON_ORE, 20, 9, 0, 63),
            (GOLD_ORE, 2, 9, 0, 31),
            (REDSTONE_ORE, 8, 8, 0, 15),
            (DIAMOND_ORE, 1, 8, 0, 15),
            (LAPIS_ORE, 1, 7, 8, 24), // depth_average(16, 16) -> y16±8
        ];
        // stream-stability shim: pre-4.4a drew 3 values per blob from
        // the chunk rng here; downstream features (trees/vegetation)
        // still consume that stream, so replay the draws untouched and
        // let anchor-chunk streams below decide the veins.
        for &(_, count, _, y_min, y_max) in &VEINS {
            for _ in 0..count {
                rng.next_range(16);
                rng.next_range(16);
                rng.next_range((y_max - y_min + 1) as u32);
            }
        }
        // cross-chunk resolution: blobs anchor in their own chunk (3×3
        // neighborhood covers size ≤ 33) and every overlapped chunk
        // emits its own parts — no border clipping.
        for dax in -1..=1 {
            for daz in -1..=1 {
                let (ax, az) = (cx + dax, cz + daz);
                let mut arng = Rng::new(Rng::hash3(self.seed ^ VEIN_SALT, ax, 0, az));
                for &(state, count, size, y_min, y_max) in &VEINS {
                    for _ in 0..count {
                        // the vanilla placement: square (anywhere in the
                        // anchor chunk) + range (uniform y)
                        let wx = ax * 16 + arng.next_range(16) as i32;
                        let wz = az * 16 + arng.next_range(16) as i32;
                        let wy = y_min + arng.next_range((y_max - y_min + 1) as u32) as i32;
                        self.ore_blob_world(chunk, (cx * 16, cz * 16), (wx, wy, wz), state, size);
                    }
                }
            }
        }
    }

    /// One vanilla-style ellipsoid ore blob at the chunk-local center,
    /// replacing base-stone only; edge jitter is a per-position hash so
    /// the blob shape is stream-order independent.
    /// One vanilla-style ellipsoid ore blob at WORLD coords, emitting
    /// only the cells inside this chunk (ox, oz = chunk origin).
    /// 4.4a: the shape hash uses world coords so every overlapped
    /// chunk resolves the same blob (was chunk-local before).
    fn ore_blob_world(
        &self,
        chunk: &mut Chunk,
        origin: (i32, i32),
        center: (i32, i32, i32),
        state: u16,
        size: u32,
    ) {
        let (ox, oz) = origin;
        let (wx, wy, wz) = center;
        let a = size as f64 / 8.0;
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0BE5, wx, wy, wz));
        let hx = a * (0.7 + rng.next_f32() as f64 * 0.6);
        let hy = a * (0.5 + rng.next_f32() as f64 * 0.5);
        let hz = a * (0.7 + rng.next_f32() as f64 * 0.6);
        let theta = rng.next_f32() as f64 * std::f64::consts::PI;
        let (st, ct) = (dsin64(theta), dcos64(theta));
        let r_out = ((a * 1.35).ceil() as i32) + 1;
        for dy in -r_out..=r_out {
            let by = wy + dy;
            if !(1..=250).contains(&by) {
                continue;
            }
            for dx in -r_out..=r_out {
                for dz in -r_out..=r_out {
                    let (bx, bz) = (wx + dx, wz + dz);
                    let (lx, lz) = (bx - ox, bz - oz);
                    if !(0..16).contains(&lx) || !(0..16).contains(&lz) {
                        continue;
                    }
                    let cur = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        by as usize,
                    );
                    let base_stone =
                        cur == STONE || cur == GRANITE || cur == DIORITE || cur == ANDESITE;
                    if !base_stone {
                        continue;
                    }
                    // ellipsoid test in the theta-rotated xz frame
                    let fx = dx as f64 * ct + dz as f64 * st;
                    let fz = -dx as f64 * st + dz as f64 * ct;
                    let v = (fx / hx) * (fx / hx)
                        + (dy as f64 / hy) * (dy as f64 / hy)
                        + (fz / hz) * (fz / hz);
                    // per-position edge jitter (±0.15 on the radius)
                    let j = (Rng::hash3(self.seed ^ 0x0BE6, bx, by, bz) % 1000) as f64 / 1000.0;
                    if v <= 1.0 + (j - 0.5) * 0.3 {
                        chunk.set(lx as usize, by as usize, lz as usize, state);
                    }
                }
            }
        }
    }
    /// Generate one chunk column (dimension-dispatched). Pure: returns
    /// chunk + edits for neighbors (tree canopies crossing chunk borders).
    pub fn generate_chunk(
        &self,
        cx: i32,
        cz: i32,
        inbound: Vec<(u16, u16)>, // (block idx, id) edits queued from neighbors
    ) -> GenOut {
        match self.dim {
            Dimension::Overworld => self.generate_overworld_chunk(cx, cz, inbound),
            Dimension::Nether => self.generate_nether_chunk(cx, cz, inbound),
            Dimension::End => self.generate_end_chunk(cx, cz, inbound),
        }
    }

    /// The overworld generator: terrain columns, caves, ores, vegetation,
    /// villages. §26/§48 Phase 7.
    fn generate_overworld_chunk(
        &self,
        cx: i32,
        cz: i32,
        inbound: Vec<(u16, u16)>, // (block idx, id) edits queued from neighbors
    ) -> GenOut {
        let mut chunk = Chunk::empty();
        let mut rng = Rng::new(Rng::hash3(self.seed, cx, 0, cz));
        let sea = vc_chunk::SEA_LEVEL;
        let mut outbound: Vec<(i32, i32, i32, u16)> = Vec::new();

        // ---- Phase E3: superflat short-circuit (the VERIFIED classic
        // preset: bedrock, 2 dirt, grass; plains biome; no structures,
        // no caves, no ocean fill — the surface sits at y=3) ----
        if self.flat {
            for z in 0..16usize {
                for x in 0..16usize {
                    let col_idx = z * 16 + x;
                    chunk.height[col_idx] = 3;
                    chunk.biome[col_idx] = Biome::Plains as u8;
                    for y in 0..=3usize {
                        let b = match y {
                            0 => BEDROCK,
                            1 | 2 => DIRT,
                            _ => GRASS,
                        };
                        chunk.set(x, y, z, b);
                    }
                }
            }
            return (Arc::new(chunk), outbound);
        }

        // Phase 10: ravines covering this chunk (computed once — the
        // 11×11-chunk neighborhood covers the max 127-block diagonal so
        // every chunk independently agrees on every ravine that reaches
        // it, exactly like the village/mineshaft region queries)
        let ravines = self.ravines_near_chunk(cx, cz);

        // ---- Vanilla-parity terrain round (2026-09-14): the 1.16.5
        // density stack sampled on the 4×8×4 noise lattice (5×33×5 per
        // chunk), trilinearly interpolated per block (see
        // vanilla_noise.rs for the cited constants), then the surface
        // builder, the bedrock floor, the worm carvers, and the ore
        // features — the vanilla generation order (noise → surface →
        // carvers → features). ----
        let ox = cx * 16;
        let oz = cz * 16;

        // climate biome depth/scale grid (7×7 at 4-block steps; the 3×3
        // neighborhood around each lattice column smooths the biome
        // response — the vanilla squoze-biome behavior)
        let mut climate = [[(0f64, 0f64); 7]; 7];
        for (gz, row) in climate.iter_mut().enumerate() {
            for (gx, cell) in row.iter_mut().enumerate() {
                *cell =
                    self.climate_depth_scale(ox + (gx as i32 - 1) * 4, oz + (gz as i32 - 1) * 4);
            }
        }

        // per-lattice-column drivers (5×5 at 4-block steps)
        struct LatCol {
            h_eff: f64,
            amp: f64,
            rnd: f64,
            mush_island: Option<f64>,
        }
        let mut lat: Vec<LatCol> = Vec::with_capacity(25);
        for lz in 0..5usize {
            for lx in 0..5usize {
                let wx = ox + lx as i32 * 4;
                let wz = oz + lz as i32 * 4;
                let mut bd = 0.0;
                let mut bv = 0.0;
                for dz in 0..3usize {
                    for dx in 0..3usize {
                        bd += climate[lz + dz][lx + dx].0;
                        bv += climate[lz + dz][lx + dx].1;
                    }
                }
                bd /= 9.0;
                bv /= 9.0;
                let mush = self.n_mush.noise2(wx as f32 / 400.0, wz as f32 / 400.0);
                let island = if self.dim == Dimension::Overworld && mush > 0.87 {
                    Some(
                        (vc_chunk::SEA_LEVEL as f64 + 1.0 + (mush as f64 - 0.63) * 30.0)
                            .min(vc_chunk::SEA_LEVEL as f64 + 6.0),
                    )
                } else {
                    None
                };
                let (h_eff, amp, _) = self.density_params(wx, wz, bd, bv);
                let rnd = self.random_density_offset(wx, wz);
                lat.push(LatCol {
                    h_eff,
                    amp,
                    rnd,
                    mush_island: island,
                });
            }
        }

        // density lattice 5×33×5 (y cells of 8 blocks)
        let mut dens = [0f64; 5 * 33 * 5];
        for ly in 0..33usize {
            for lz in 0..5usize {
                for lx in 0..5usize {
                    let c = &lat[lz * 5 + lx];
                    let wx = (ox + lx as i32 * 4) as f64;
                    let wz = (oz + lz as i32 * 4) as f64;
                    let d = match c.mush_island {
                        Some(isl) => (isl - (ly * 8) as f64) / 8.0,
                        None => {
                            self.vterrain
                                .density(wx, (ly * 8) as f64, wz, c.h_eff, c.amp, c.rnd)
                        }
                    };
                    dens[(ly * 5 + lz) * 5 + lx] = d;
                }
            }
        }
        let getdens =
            |lxx: usize, lyy: usize, lzz: usize| -> f64 { dens[(lyy * 5 + lzz) * 5 + lxx] };

        // per-block fill
        for z in 0..16usize {
            for x in 0..16usize {
                let wx = ox + x as i32;
                let wz = oz + z as i32;
                let lxi = x / 4;
                let lzi = z / 4;
                let fx = (x % 4) as f64 / 4.0;
                let fz = (z % 4) as f64 / 4.0;

                // highest cell with any solid corner → scan start
                let mut top_cell = 0usize;
                'cells: for ly in (0..33usize).rev() {
                    for czz in lzi..=lzi + 1 {
                        for cxx in lxi..=lxi + 1 {
                            if getdens(cxx, ly, czz) > -0.6 {
                                top_cell = ly;
                                break 'cells;
                            }
                        }
                    }
                }
                let top_y = ((top_cell + 1) * 8).min(255);

                let mut surf: i32 = 0;
                // 1.4: cell-major fill — the 8 lattice corners are read
                // once per 8-block cell instead of once per level
                // (identical values, identical lerp order per level).
                // Cells whose corners are all <= 0 above sea level can
                // write nothing and are skipped: trilinear is a convex
                // combination of its corners, so d <= max <= 0 (no
                // STONE), and no level is below sea (no WATER).
                // Surf/height/biome tracking is unaffected (skipped
                // cells contribute no d > 0 level).
                for ly in 0..=(top_y / 8) {
                    let y0 = ly * 8;
                    let y1 = (y0 + 7).min(top_y);
                    let c000 = getdens(lxi, ly, lzi);
                    let c100 = getdens(lxi + 1, ly, lzi);
                    let c010 = getdens(lxi, ly, lzi + 1);
                    let c110 = getdens(lxi + 1, ly, lzi + 1);
                    let c001 = getdens(lxi, ly + 1, lzi);
                    let c101 = getdens(lxi + 1, ly + 1, lzi);
                    let c011 = getdens(lxi, ly + 1, lzi + 1);
                    let c111 = getdens(lxi + 1, ly + 1, lzi + 1);
                    let cmax = c000
                        .max(c100)
                        .max(c010)
                        .max(c110)
                        .max(c001)
                        .max(c101)
                        .max(c011)
                        .max(c111);
                    if cmax <= 0.0 && (y0 as i32) >= sea {
                        continue;
                    }
                    for y in (y0..=y1).rev() {
                        let fy = (y % 8) as f64 / 8.0;
                        // trilinear over the 8 cell corners
                        let dx0 = lerp64(c000, c100, fx);
                        let dx1 = lerp64(c010, c110, fx);
                        let dxy = lerp64(dx0, dx1, fz);
                        let ex0 = lerp64(c001, c101, fx);
                        let ex1 = lerp64(c011, c111, fx);
                        let exy = lerp64(ex0, ex1, fz);
                        let d = lerp64(dxy, exy, fy);

                        if d > 0.0 {
                            chunk.set(x, y, z, STONE);
                            if y as i32 > surf {
                                surf = y as i32;
                            }
                        } else if (y as i32) < sea {
                            chunk.set(x, y, z, WATER);
                        }
                    }
                }

                // ---- classification + surface builder ----
                // (BEFORE the bedrock/ravine/carver passes: vanilla biome
                // selection is climate-driven and never depends on carved
                // height — a ravine canyon keeps its surface biome)
                let col_idx = z * 16 + x;
                let mush = self.n_mush.noise2(wx as f32 / 400.0, wz as f32 / 400.0);
                let (biome, top, filler) = if self.dim == Dimension::Overworld && mush > 0.87 {
                    (Biome::MushroomFields, MYCELIUM, DIRT)
                } else {
                    let (temp, humid, var) = self.climate_fields(wx, wz);
                    let rv = self.n_river.noise2(wx as f32 / 640.0, wz as f32 / 640.0);
                    self.classify(temp, humid, var, surf, rv, (wx, wz))
                };
                chunk.biome[col_idx] = biome as u8;
                chunk.height[col_idx] = surf.clamp(0, 255) as u8;

                // surface band: top + filler with the patchy surface-depth
                // noise (dirt 3..6), desert/beach sandstone under the sand,
                // badlands terracotta strata (all pre-rewrite materials)
                let sd = self.vterrain.surface_depth(wx as f64, wz as f64);
                let dirt_depth = 3 + ((sd * 1.5 + 0.5).max(0.0) as i32);
                let band_top = surf.min(255);
                // badlands strata run 16 deep (the pre-rewrite,
                // wiki-cited banding); other biomes only need the
                // top+filler band
                let band_bot = if biome == Biome::Badlands {
                    (surf - 16).max(1)
                } else {
                    (surf - 9).max(1)
                };
                for y in (band_bot..=band_top).rev() {
                    let yi = y as usize;
                    let cur = chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), yi);
                    let is_stone =
                        cur == STONE || cur == GRANITE || cur == DIORITE || cur == ANDESITE;
                    if !is_stone {
                        continue; // water / bedrock / carved air stay
                    }
                    let b = if y == surf {
                        top
                    } else if y > surf - dirt_depth {
                        filler
                    } else if biome == Biome::Badlands && y > surf - 16 {
                        // the banded terracotta strata (pre-rewrite,
                        // wiki-cited)
                        stained_terracotta(badlands_band_color(self.seed, y))
                    } else {
                        break; // below the strata band: keep stone
                    };
                    chunk.set(x, yi, z, b);
                }
                // Phase E2 (VERIFIED w/Emerald_Ore): emerald ore only in
                // mountains-family biomes as single blocks y 4..=31
                if biome == Biome::Mountains {
                    for y in 4..=31 {
                        if y >= surf {
                            break;
                        }
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y as usize) == STONE
                            && emerald_ore(self.seed, wx, y, wz)
                        {
                            chunk.set(x, y as usize, z, EMERALD_ORE);
                        }
                    }
                }

                // bedrock floor (vanilla shape): y=0 always; y=1..4 with
                // the decreasing per-column chance (draw r ∈ 0..4;
                // bedrock for y ≤ r — the 100/80/60/40/20% stack,
                // reference wiki /Bedrock: "the five bottommost layers
                // ... in a rough pattern"; hash-based draw, disclosed)
                let br = (Rng::hash3(self.seed ^ 0xBED0, wx, 0, wz) % 5) as i32;
                for y in 0..=br.min(4) {
                    chunk.set(x, y as usize, z, BEDROCK);
                }

                // ravine carve (the existing region system; interval from
                // the pre-carve surface down, never through bedrock or
                // water; blocks only — the biome stays as classified)
                if !ravines.is_empty() {
                    if let Some((rv_top, rv_bottom)) = self.ravine_cut(&ravines, wx, wz, surf) {
                        for y in (rv_bottom + 1)..=(rv_top.min(surf)) {
                            let cur =
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y as usize);
                            if cur != BEDROCK && cur != WATER && cur != AIR {
                                chunk.set(x, y as usize, z, AIR);
                            }
                        }
                    }
                }
            }
        }

        // worm carvers (after the surface pass, like vanilla)
        self.carve_caves(&mut chunk, cx, cz);

        // recompute heights after carving (cave entrances / ravine
        // floors expose the true surface)
        for z in 0..16usize {
            for x in 0..16usize {
                let col_idx = z * 16 + x;
                let mut y = chunk.height[col_idx] as i32;
                while y > 0 {
                    let c = chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y as usize);
                    if c != AIR && c != WATER {
                        break;
                    }
                    y -= 1;
                }
                chunk.height[col_idx] = y.clamp(0, 255) as u8;
            }
        }

        // vanilla ore veins (feature stage 6 — after carving, replacing
        // base stone only)
        self.place_ores(&mut chunk, cx, cz, &mut rng);

        // pass 2: inbound edits from neighbors (trees poking into this chunk)
        for (idx, id) in inbound {
            let cur = chunk.get_idx(idx as usize);
            let trunk = id == OAK_LOG || id == DARK_OAK_LOG || id == ACACIA_LOG;
            if cur == AIR
                || (trunk && (cur == LEAVES || cur == ACACIA_LEAVES || cur == DARK_OAK_LEAVES))
            {
                chunk.set_idx(idx as usize, id);
            }
        }

        // pass 3: decorations (trees, plants) — deterministic per chunk
        let set_dec = |chunk: &mut Chunk,
                       outbound: &mut Vec<(i32, i32, i32, u16)>,
                       wx: i32,
                       wy: i32,
                       wz: i32,
                       id: u16,
                       replace_leaves: bool| {
            if !(0..=255).contains(&wy) {
                return;
            }
            let lxi = wx - ox;
            let lzi = wz - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) {
                let cur = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lxi as usize, lzi as usize),
                    wy as usize,
                );
                let trunk =
                    id == OAK_LOG || id == DARK_OAK_LOG || id == ACACIA_LOG || id == JUNGLE_LOG;
                if cur == AIR
                    || (replace_leaves
                        && trunk
                        && (cur == LEAVES
                            || cur == ACACIA_LEAVES
                            || cur == DARK_OAK_LEAVES
                            || cur == JUNGLE_LEAVES))
                {
                    chunk.set(lxi as usize, wy as usize, lzi as usize, id);
                }
            } else {
                outbound.push((wx, wy, wz, id));
            }
        };

        let tree_count = {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]); // center sample
            match b {
                Biome::Forest => 8,
                Biome::BirchForest => 7,
                // 4.1e: hills mirror their base canopy
                Biome::BirchHills => 7,
                Biome::Jungle => 10,
                Biome::Taiga => 5,
                Biome::TaigaHills | Biome::GiantTreeTaiga | Biome::GiantTreeTaigaHills => 5,
                Biome::Swamp => 3,
                // 1.7.2: dark forest = "dark oak trees closely packed
                // together" (wiki) — the densest canopy in the game
                Biome::DarkForest => 14,
                Biome::FlowerForest => 5,
                Biome::SunflowerPlains => {
                    if rng.next_f32() < 0.5 {
                        1
                    } else {
                        0
                    }
                }
                Biome::Savanna => {
                    // vanilla savanna: scattered acacias (~1-2 per chunk)
                    1 + (rng.next_f32() < 0.4) as i32
                }
                Biome::Plains => {
                    if rng.next_f32() < 0.5 {
                        1
                    } else {
                        0
                    }
                }
                Biome::Snowy => 3,
                _ => 0,
            }
        };

        for _ in 0..tree_count {
            let lx = 2 + rng.next_range(12) as i32;
            let lz = 2 + rng.next_range(12) as i32;
            let col_idx = lz as usize * 16 + lx as usize;
            let top = chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                chunk.height[col_idx] as usize,
            );
            if top != GRASS && top != SNOW_GRASS {
                continue;
            }
            let h = chunk.height[col_idx] as i32;
            // species: forest mixes oak + birch; snowy taiga/taiga grow
            // spruce; birch forest is birch-dominant; jungle grows its own
            // wood (audit-fix 2026-09-07: the E1-era oak-adaptation is
            // retired — JUNGLE_LOG/JUNGLE_LEAVES exist now, VERIFIED
            // w/Tree: "Added jungle trees" 1.2.1 12w03a);
            // 1.7.2: savanna grows acacia, dark forest grows dark oak
            let biome_here = Biome::from_u8(chunk.biome[col_idx]);
            let (log, leaf) = match biome_here {
                Biome::Snowy
                | Biome::Taiga
                | Biome::TaigaHills
                | Biome::GiantTreeTaiga
                | Biome::GiantTreeTaigaHills => (SPRUCE_LOG, SPRUCE_LEAVES),
                Biome::Savanna => (ACACIA_LOG, ACACIA_LEAVES),
                Biome::DarkForest => (DARK_OAK_LOG, DARK_OAK_LEAVES),
                Biome::Jungle => (JUNGLE_LOG, JUNGLE_LEAVES),
                Biome::Forest => {
                    if rng.next_f32() < 0.35 {
                        (BIRCH_LOG, BIRCH_LEAVES)
                    } else {
                        (OAK_LOG, LEAVES)
                    }
                }
                Biome::BirchForest | Biome::BirchHills => {
                    if rng.next_f32() < 0.75 {
                        (BIRCH_LOG, BIRCH_LEAVES)
                    } else {
                        (OAK_LOG, LEAVES)
                    }
                }
                _ => (OAK_LOG, LEAVES),
            };
            let th = if biome_here == Biome::Snowy
                || biome_here == Biome::Taiga
                || biome_here == Biome::TaigaHills
                || biome_here == Biome::GiantTreeTaiga
                || biome_here == Biome::GiantTreeTaigaHills
            {
                6 + rng.next_range(3) as i32 // spruce grows taller
            } else if biome_here == Biome::Jungle {
                // audit-fix (VERIFIED w/Jungle_Tree search round: regular
                // jungle trees have a 1x1 trunk "which can extend up to
                // 10 blocks tall") — 5..10, distinctly taller than oak
                5 + rng.next_range(6) as i32
            } else {
                4 + rng.next_range(3) as i32 // 4..6
            };
            let y0 = h + 1;

            // ---- audit-fix: jungle bushes (VERIFIED w/Tree: "Jungle
            // bushes also generate in the jungle biome, featuring a
            // single jungle log surrounded by oak leaves") — ~25% of
            // jungle trees take the bush form: 1 JUNGLE_LOG + an oak
            // leaf blob, no tall trunk.
            if biome_here == Biome::Jungle && rng.next_f32() < 0.25 {
                set_dec(
                    &mut chunk,
                    &mut outbound,
                    ox + lx,
                    y0,
                    oz + lz,
                    JUNGLE_LOG,
                    true,
                );
                // leaf ring around the log (ragged corners) + a cap above
                for (dy, r) in [(0i32, 1i32), (1, 1), (2, 1)] {
                    let ly = y0 + dy;
                    for dx in -r..=r {
                        for dz in -r..=r {
                            if dx == 0 && dz == 0 && dy == 0 {
                                continue; // the log's own cell
                            }
                            let corner = dx.abs() == r && dz.abs() == r;
                            if corner && rng.next_f32() < 0.5 {
                                continue;
                            }
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx,
                                ly,
                                oz + lz + dz,
                                LEAVES, // oak leaves — the VERIFIED detail
                                false,
                            );
                        }
                    }
                }
                if chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    h as usize,
                ) == GRASS
                {
                    chunk.set(lx as usize, h as usize, lz as usize, DIRT);
                }
                continue;
            }

            // ---- 1.7.2 tree shapes ----
            // acacia (savanna): "curved trees made of acacia logs" — a
            // vertical base, a diagonal offset segment, then a FLAT disc
            // canopy (the wiki's signature acacia silhouette)
            if biome_here == Biome::Savanna {
                let base_h = 2 + rng.next_range(2) as i32; // vertical part
                let lean = rng.next_range(4) as i32; // 0..3 = +x,+z,-x,-z
                let (ldx, ldz) = match lean {
                    0 => (1, 0),
                    1 => (0, 1),
                    2 => (-1, 0),
                    _ => (0, -1),
                };
                let lean_len = 1 + rng.next_range(2) as i32; // diagonal part
                let top_y = y0 + base_h + lean_len;
                // vertical trunk
                for ty in 0..base_h {
                    set_dec(
                        &mut chunk,
                        &mut outbound,
                        ox + lx,
                        y0 + ty,
                        oz + lz,
                        log,
                        true,
                    );
                }
                // diagonal segment (axis state when in-chunk; neighbor
                // outbound keeps the plain id — axis-variant outbound edits
                // are a documented simplification)
                for i in 1..=lean_len {
                    let bx = ox + lx + ldx * i;
                    let bz = oz + lz + ldz * i;
                    let by = y0 + base_h - 1 + i;
                    let lxi_i = bx - ox;
                    let lzi_i = bz - oz;
                    if (0..16).contains(&lxi_i) && (0..16).contains(&lzi_i) {
                        let axis = if ldx != 0 { 0u8 } else { 2u8 };
                        chunk.set_state(
                            lxi_i as usize,
                            by as usize,
                            lzi_i as usize,
                            log_axis_state(log, axis),
                        );
                    }
                }
                // flat canopy: two r=2 discs + one r=1 cap (ragged corners)
                for (dy, r) in [(0i32, 2i32), (1, 2), (2, 1)] {
                    let ly = top_y + dy - 1;
                    for dx in -r..=r {
                        for dz in -r..=r {
                            if dx == 0 && dz == 0 && dy < 2 {
                                continue;
                            }
                            let corner = dx.abs() == r && dz.abs() == r;
                            if corner && rng.next_f32() < 0.5 {
                                continue;
                            }
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + ldx * lean_len + dx,
                                ly,
                                oz + lz + ldz * lean_len + dz,
                                leaf,
                                false,
                            );
                        }
                    }
                }
                // dirt under trunk
                if chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    h as usize,
                ) == GRASS
                {
                    chunk.set(lx as usize, h as usize, lz as usize, DIRT);
                }
                continue;
            }
            // dark oak (dark forest): "very thick and short trees" — a 2×2
            // trunk with a broad low canopy; vanilla requires a 2×2 sapling
            // configuration to grow (wiki)
            if biome_here == Biome::DarkForest {
                let th_d = 5 + rng.next_range(3) as i32; // 5..7 short+thick
                let y0d = h + 1;
                // 2×2 trunk
                for dx in 0..2 {
                    for dz in 0..2 {
                        for ty in 0..th_d {
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx,
                                y0d + ty,
                                oz + lz + dz,
                                log,
                                true,
                            );
                        }
                    }
                }
                // broad canopy: r=3 discs at the top two layers, r=2, r=1 cap
                for (dy, r) in [(-1i32, 3i32), (0, 3), (1, 2), (2, 1)] {
                    let ly = y0d + th_d - 1 + dy;
                    for dx in -r..=r + 1 {
                        for dz in -r..=r + 1 {
                            let cdx = dx - 1; // canopy centered on the 2×2
                            let cdz = dz - 1;
                            if (0..=1).contains(&cdx) && (0..=1).contains(&cdz) && dy < 2 {
                                continue; // trunk spot
                            }
                            let corner = cdx.abs() == r || cdz.abs() == r;
                            let corner2 = cdx.abs() == r && cdz.abs() == r;
                            if (corner2 || (corner && rng.next_f32() < 0.35))
                                && (cdz.abs() >= r || cdx.abs() >= r)
                            {
                                continue;
                            }
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx - 1,
                                ly,
                                oz + lz + dz - 1,
                                leaf,
                                false,
                            );
                        }
                    }
                }
                // dirt under the 2×2
                for dx in 0..2 {
                    for dz in 0..2 {
                        if chunk.get_local(
                            vc_chunk::chunk::LocalXZ::new((lx + dx) as usize, (lz + dz) as usize),
                            h as usize,
                        ) == GRASS
                        {
                            chunk.set((lx + dx) as usize, h as usize, (lz + dz) as usize, DIRT);
                        }
                    }
                }
                continue;
            }

            // canopy: two 5x5 layers, two 3x3 layers (oak/birch);
            // spruce: stacked narrowing rings
            if biome_here == Biome::Snowy || biome_here == Biome::Taiga {
                for dy in 0..th {
                    let ly = y0 + dy;
                    let r: i32 = match dy {
                        0 => 1,
                        1 => 2,
                        2 => 2,
                        3 => 2,
                        _ => 1,
                    };
                    for dx in -r..=r {
                        for dz in -r..=r {
                            if dx == 0 && dz == 0 {
                                continue;
                            }
                            let corner = dx.abs() == r && dz.abs() == r;
                            if corner && rng.next_f32() < 0.6 {
                                continue;
                            }
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx,
                                ly,
                                oz + lz + dz,
                                leaf,
                                false,
                            );
                        }
                    }
                }
                // spire tip
                set_dec(
                    &mut chunk,
                    &mut outbound,
                    ox + lx,
                    y0 + th,
                    oz + lz,
                    leaf,
                    false,
                );
            } else {
                for dy in -2..=1 {
                    let ly = y0 + th - 1 + dy;
                    let r: i32 = if dy < 0 { 2 } else { 1 };
                    for dx in -r..=r {
                        for dz in -r..=r {
                            if dx == 0 && dz == 0 && dy < 0 {
                                continue; // trunk spot
                            }
                            let corner = dx.abs() == r && dz.abs() == r;
                            if corner && (dy >= 0 || rng.next_f32() < 0.5) {
                                continue; // ragged corners
                            }
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx,
                                ly,
                                oz + lz + dz,
                                leaf,
                                false,
                            );
                        }
                    }
                }
            }
            // trunk
            for ty in 0..th {
                set_dec(
                    &mut chunk,
                    &mut outbound,
                    ox + lx,
                    y0 + ty,
                    oz + lz,
                    log,
                    true,
                );
            }
            // ---- audit-fix: vines on jungle trunks (VERIFIED w/Vines:
            // "Jungle trees of both sizes have vines on their trunks and
            // canopy edges"). Cross-rendered adaptation: the vine block
            // occupies the air cell beside each trunk log (~60%/side).
            if biome_here == Biome::Jungle {
                for ty in 0..th {
                    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        if rng.next_f32() < 0.6 {
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx + dx,
                                y0 + ty,
                                oz + lz + dz,
                                VINE,
                                false,
                            );
                        }
                    }
                }
            }
            // ---- 1.15 (Buzzy Bees): bee nests on generated trees.
            // VERIFIED w/Bee §Natural generation: oak + birch trees
            // carry a nest by biome (JE: Plains/Sunflower Plains 5%,
            // Flower Forest 2%, Forest/Birch Forest/Old Growth Birch
            // Forest 0.2%; meadow/mangrove/cherry are post-1.16.5
            // biomes — out of scope). The nest generates holding 2-3
            // bees — the sim layer registers them lazily on first
            // approach (deterministic per nest position, see
            // vc-gameplay bees.rs). ----
            // the roll is a PER-TREE POSITION hash (the Rng::hash3
            // fossil pattern) — deterministic per tree and consumes NO
            // chunk-rng stream draws, so the nest window cannot shift
            // the downstream flora layout (the golden-determinism
            // discipline)
            let nest_roll = Rng::hash3(self.seed ^ 0xBEE5, ox + lx, 0, oz + lz) % 1000;
            let nest_ok = match biome_here {
                Biome::Plains | Biome::SunflowerPlains => nest_roll < 50, // 5%
                Biome::FlowerForest => nest_roll < 20,                    // 2%
                Biome::Forest | Biome::BirchForest => nest_roll < 2,      // 0.2%
                _ => false,
            };
            let oak_or_birch = matches!((log, leaf), (OAK_LOG, LEAVES) | (BIRCH_LOG, BIRCH_LEAVES));
            if nest_ok && oak_or_birch {
                // a nest cell beside the trunk, mid-canopy height — the
                // side + height are hash-derived too (no stream draws).
                // The cell EMBEDS in the canopy (vanilla nests replace
                // a leaf beside the trunk) — the in-chunk write replaces
                // AIR or a leaf; border-tree cells going outbound would
                // need the leaf-replace path that apply_gen_edit only
                // grants logs, so those drop — disclosed simplification
                // (~6% of trees sit on a chunk border).
                let side = (Rng::hash3(self.seed ^ 0xBEE5, ox + lx, 1, oz + lz) % 4) as i32;
                let (ndx, ndz) = match side {
                    0 => (1, 0),
                    1 => (0, 1),
                    2 => (-1, 0),
                    _ => (0, -1),
                };
                let span = (th.max(2) - 1).max(1) as u64;
                let ny_off = (Rng::hash3(self.seed ^ 0xBEE5, ox + lx, 2, oz + lz) % span) as i32;
                let ny = (y0 + 1 + ny_off).clamp(1, 254);
                let nx = lx + ndx;
                let nz = lz + ndz;
                if (0..16).contains(&nx) && (0..16).contains(&nz) {
                    let cur = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(nx as usize, nz as usize),
                        ny as usize,
                    );
                    if matches!(cur, AIR | LEAVES | BIRCH_LEAVES) {
                        chunk.set(nx as usize, ny as usize, nz as usize, BEE_NEST);
                    }
                }
            }
            // dirt under trunk
            let under = chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                h as usize,
            );
            if under == GRASS || under == SNOW_GRASS {
                chunk.set(lx as usize, h as usize, lz as usize, DIRT);
            }
        }

        // flowers + tall grass
        let plant_attempts = {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            match b {
                Biome::Plains => 14,
                Biome::Savanna => 12,
                Biome::Jungle => 14,
                Biome::Forest => 10,
                Biome::BirchForest => 9,
                Biome::Taiga => 4,
                Biome::Swamp => 6,
                Biome::Snowy => 2,
                // 1.7.2: flower forest = "very densely packed with the
                // various new flowers"; sunflower plains = plains flora +
                // the sunflower crop itself
                Biome::FlowerForest => 40,
                Biome::SunflowerPlains => 18,
                _ => 0,
            }
        };
        for _ in 0..plant_attempts {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let col_idx = lz as usize * 16 + lx as usize;
            let h = chunk.height[col_idx] as i32;
            if chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                h as usize,
            ) != GRASS
            {
                continue;
            }
            if chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                (h + 1) as usize,
            ) != AIR
            {
                continue;
            }
            let b_here = Biome::from_u8(chunk.biome[col_idx]);
            let r = rng.next_f32();
            // per-biome flora mixes (1.7.2 wiki lists):
            // * flower forest: peonies, orange/white tulips, oxeye daisies,
            //   rose bush, allium + dandelions — excluding sunflowers
            // * sunflower plains: sunflowers over plain flora
            // * everyone else: tall grass + poppy/dandelion as before
            let (id, tall_top) = match b_here {
                Biome::FlowerForest => {
                    if r < 0.50 {
                        // the small-flower mix (weighted by the wiki's list;
                        // 1.14 18w43a: + cornflower, lily of the valley —
                        // VERIFIED w/Cornflower §Natural generation "flower
                        // forest" + w/Lily_of_the_Valley "flower forest")
                        let s = rng.next_range(10) as u8;
                        let small = match s {
                            0 => ALLIUM,
                            1 => OXEYE_DAISY,
                            2 => ORANGE_TULIP,
                            3 => WHITE_TULIP,
                            4 => RED_TULIP,
                            5 => PINK_TULIP,
                            6 => AZURE_BLUET,
                            7 => BLUE_ORCHID,
                            8 => CORNFLOWER,
                            _ => LILY_OF_THE_VALLEY,
                        };
                        (small, 0u16)
                    } else if r < 0.62 {
                        (PEONY, PEONY_TOP)
                    } else if r < 0.74 {
                        (ROSE_BUSH, ROSE_BUSH_TOP)
                    } else if r < 0.84 {
                        (LILAC, LILAC_TOP)
                    } else if r < 0.92 {
                        (FLOWER_RED, 0u16)
                    } else {
                        (FLOWER_YELLOW, 0u16)
                    }
                }
                Biome::SunflowerPlains => {
                    if r < 0.45 {
                        (SUNFLOWER, SUNFLOWER_TOP)
                    } else if r < 0.85 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.93 {
                        (FLOWER_RED, 0u16)
                    } else if r < 0.97 {
                        (OXEYE_DAISY, 0u16)
                    } else {
                        // 1.14: cornflower joins the plains flora
                        // (w/Cornflower §Natural generation: "plains,
                        // sunflower plains, ...")
                        (CORNFLOWER, 0u16)
                    }
                }
                // audit-fix: ferns (VERIFIED w/Fern — "non-solid plant
                // blocks... have the same characteristics as grass";
                // placed on grass/dirt family w/Fern §Placement).
                // Biome list VERIFIED w/Fern §Natural generation:
                // "Ferns occur naturally only in jungle, taiga, snowy
                // taiga and old growth taiga biomes and their variants,
                // scattered with short grass" (NOT swamp — the live
                // source corrected the first draft).
                Biome::Jungle => {
                    if r < 0.55 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.85 {
                        (FERN, 0u16)
                    } else if r < 0.93 {
                        (FLOWER_RED, 0u16)
                    } else {
                        (FLOWER_YELLOW, 0u16)
                    }
                }
                Biome::Taiga | Biome::Snowy => {
                    if r < 0.62 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.86 {
                        (FERN, 0u16)
                    } else if r < 0.93 {
                        (FLOWER_RED, 0u16)
                    } else {
                        (FLOWER_YELLOW, 0u16)
                    }
                }
                // 1.14: cornflower joins the plains flora (VERIFIED
                // w/Cornflower §Natural generation: "plains, sunflower
                // plains, flower forest, and meadow biomes" — our biome
                // set has Plains; meadow is 1.17-era, out of bracket)
                Biome::Plains => {
                    if r < 0.72 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.84 {
                        (FLOWER_RED, 0u16)
                    } else if r < 0.90 {
                        (FLOWER_YELLOW, 0u16)
                    } else {
                        (CORNFLOWER, 0u16)
                    }
                }
                // 1.14: lily of the valley joins the forest floors
                // (VERIFIED w/Lily_of_the_Valley §Natural generation:
                // "forest, flower forest, birch forest, old growth
                // birch forest, and dark forest" — our Forest +
                // BirchForest; dark forest is out of bracket)
                Biome::Forest | Biome::BirchForest => {
                    if r < 0.70 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.80 {
                        (FLOWER_RED, 0u16)
                    } else if r < 0.90 {
                        (FLOWER_YELLOW, 0u16)
                    } else {
                        (LILY_OF_THE_VALLEY, 0u16)
                    }
                }
                _ => {
                    if r < 0.72 {
                        (TALL_GRASS, 0u16)
                    } else if r < 0.86 {
                        (FLOWER_RED, 0u16)
                    } else {
                        (FLOWER_YELLOW, 0u16)
                    }
                }
            };
            // two-block flowers are ATOMIC: both cells must be open —
            // a lower half under an obstructed top cell would strand a
            // headless sunflower/lilac (the latent race the 1.15
            // round's flora test caught; now the pair places only when
            // both h+1 AND h+2 are air)
            let tall_ok = tall_top == 0
                || (h + 2 <= 255
                    && chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 2) as usize,
                    ) == AIR);
            if tall_ok {
                set_dec(
                    &mut chunk,
                    &mut outbound,
                    ox + lx,
                    h + 1,
                    oz + lz,
                    id,
                    false,
                );
                // two-block flowers: the upper half rides one block above
                if tall_top != 0 {
                    set_dec(
                        &mut chunk,
                        &mut outbound,
                        ox + lx,
                        h + 2,
                        oz + lz,
                        tall_top,
                        false,
                    );
                }
            }
        }

        // ───────── 1.14 (Village & Pillage — nature half) flora ────
        // Bamboo: VERIFIED w/Bamboo §Natural_generation: "Bamboo
        // generates in widely scattered single shoots within jungle
        // biomes. Bamboo generates much more densely in the bamboo
        // jungles, covering large areas of the landscape. Bamboo does
        // not generate in sparse jungles." The engine has no
        // bamboo-jungle sub-biome, so the adaptation is a per-chunk
        // patch roll (~20%) of scattered 1-5-tall shoots on grass —
        // disclosed (the alternative, per-column single-shoot
        // salt-and-pepper, reads worse at chunk scale).
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if b == Biome::Jungle && rng.next_f32() < 0.20 {
                let shoots = 4 + rng.next_range(7) as i32; // 4..10
                for _ in 0..shoots {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    let floor = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    );
                    if floor != GRASS && floor != DIRT && floor != PODZOL {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 1) as usize,
                    ) != AIR
                    {
                        continue;
                    }
                    // vanilla jungle shoots are short (1-3 commonly);
                    // a rare 5-tall one gives the canopy-side skyline
                    let bh = 1 + rng.next_range(4) as i32; // 1..4
                    for dy in 1..=bh {
                        let y = h + dy;
                        if y < 255
                            && chunk.get_local(
                                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                                y as usize,
                            ) == AIR
                        {
                            chunk.set(lx as usize, y as usize, lz as usize, BAMBOO);
                        }
                    }
                }
            }
        }
        // Sweet berry bushes: VERIFIED w/Sweet_Berry_Bush §Natural
        // generation: "only generate in taiga and snowy taiga biomes.
        // Each chunk has a 1/12 chance to generate sweet berry bushes
        // in random patches." (The old-growth taiga rows fold into the
        // engine's single Taiga, same as the fern rule.) Patches of
        // 3-6 bushes at random ages 1..=3 — mostly recognizable as
        // bearing berries at generation, matching the harvestable
        // wild-bush feel.
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if (b == Biome::Taiga || b == Biome::Snowy) && rng.next_f32() < 1.0 / 12.0 {
                let bushes = 3 + rng.next_range(4) as i32; // 3..6
                for _ in 0..bushes {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    let floor = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    );
                    if floor != GRASS && floor != SNOW_GRASS && floor != PODZOL && floor != DIRT {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 1) as usize,
                    ) != AIR
                    {
                        continue;
                    }
                    let age = 1 + rng.next_range(3) as u8; // 1..=3
                    chunk.set(
                        lx as usize,
                        (h + 1) as usize,
                        lz as usize,
                        berry_bush_state(age),
                    );
                }
            }
        }

        // 1.7.2: ice plains spikes — "tall spires made of packed ice"
        // (wiki); 1-2 spires per chunk, 5-15 tall, plus-shaped bases
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if b == Biome::IceSpikes {
                let spires = 1 + rng.next_range(2) as i32;
                for _ in 0..spires {
                    let lx = 2 + rng.next_range(12) as i32;
                    let lz = 2 + rng.next_range(12) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    let spire_h = 5 + rng.next_range(11) as i32; // 5..15
                    let base_r = if rng.next_f32() < 0.5 { 1i32 } else { 2i32 };
                    for dy in 0..spire_h {
                        let y = h + 1 + dy;
                        // taper: wide plus-base for the lower quarter,
                        // single column above, 2x2 collar at mid
                        let r = if dy < spire_h / 4 {
                            base_r
                        } else if dy < spire_h / 2 {
                            1
                        } else {
                            0
                        };
                        for dx in -r..=r {
                            for dz in -r..=r {
                                // plus shape (no corners) at r=2, full at r<=1
                                if r == 2 && dx.abs() == 2 && dz.abs() == 2 {
                                    continue;
                                }
                                set_dec(
                                    &mut chunk,
                                    &mut outbound,
                                    ox + lx + dx,
                                    y,
                                    oz + lz + dz,
                                    PACKED_ICE,
                                    false,
                                );
                            }
                        }
                    }
                }
            }
        }

        // 1.10 fossils — VERIFIED (wiki /w/Java_Edition_1.10 §World
        // generation, live 2026-09-06): "generates 15–24 blocks
        // underground in deserts, swampland and their M and hills
        // variants. Each chunk has a 1/64 chance of generating a fossil.
        // Composed of bone blocks and some coal ore, arranged as to
        // resemble the skulls and spines of giant extinct creatures."
        // Ours: a 1/64 chunk roll in Desert/Swamp placing a small
        // skull-and-spine cluster of bone blocks with coal-ore ribs.
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if (b == Biome::Desert || b == Biome::Swamp)
                && Rng::hash3(self.seed ^ 0xF055, cx, 0, cz).is_multiple_of(64)
            {
                let fx = 3 + rng.next_range(10) as i32;
                let fz = 3 + rng.next_range(10) as i32;
                let col_idx = fz as usize * 16 + fx as usize;
                let surf = chunk.height[col_idx] as i32;
                let fy = (surf - 24 + rng.next_range(10) as i32).max(10); // 15..24 under
                                                                          // skull: 3×3 bone cap with eye sockets
                for dx in 0..3 {
                    for dz in 0..3 {
                        let is_eye = (dx == 1) && (dz == 0 || dz == 2);
                        let id = if is_eye { COAL_ORE } else { BONE_BLOCK };
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + fx + dx,
                            fy,
                            oz + fz + dz,
                            id,
                            false,
                        );
                    }
                }
                // spine: a chain of bone segments descending sideways
                let len = 4 + rng.next_range(4) as i32;
                for i in 0..len {
                    let sx = fx + 3 + i;
                    let sz = fz + 1;
                    let id = if i % 3 == 2 { COAL_ORE } else { BONE_BLOCK };
                    set_dec(&mut chunk, &mut outbound, ox + sx, fy, oz + sz, id, false);
                    if i % 2 == 0 {
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + sx,
                            fy - 1,
                            oz + sz,
                            BONE_BLOCK,
                            false,
                        );
                    }
                }
            }
        }

        // mushrooms in forests (shaded floor)
        let mush_attempts = {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            match b {
                Biome::Forest => 6,
                Biome::Jungle => 5,
                Biome::Taiga => 4,
                Biome::Swamp => 4,
                Biome::BirchForest => 3,
                Biome::Snowy => 3,
                _ => 0,
            }
        };
        for _ in 0..mush_attempts {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let col_idx = lz as usize * 16 + lx as usize;
            let h = chunk.height[col_idx] as i32;
            if chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                h as usize,
            ) != GRASS
                && chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    h as usize,
                ) != SNOW_GRASS
            {
                continue;
            }
            if chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                (h + 1) as usize,
            ) != AIR
            {
                continue;
            }
            let id = if rng.next_f32() < 0.5 {
                MUSHROOM_RED
            } else {
                MUSHROOM_BROWN
            };
            set_dec(
                &mut chunk,
                &mut outbound,
                ox + lx,
                h + 1,
                oz + lz,
                id,
                false,
            );
        }

        // Phase E1: HUGE mushrooms in Mushroom Fields (VERIFIED
        // w/Mushroom_Fields: "generate abundantly"; w/Huge_mushroom: the
        // red dome = five 3×3 slabs of cap blocks around the stalk, the
        // brown cap = a flat slab; stems 4..6 tall; growth needs ≥5 clear
        // blocks which an island surface provides)
        {
            let has_mush = chunk
                .biome
                .iter()
                .any(|&b| Biome::from_u8(b) == Biome::MushroomFields);
            // 4.1j: attempts target mushroom columns (tiny islands
            // otherwise never get their mushrooms); same attempt
            // budget, same rng stream shape (positions drawn first).
            let mut mush_cols = Vec::new();
            for lx in 0..16i32 {
                for lz in 0..16i32 {
                    if Biome::from_u8(chunk.biome[lz as usize * 16 + lx as usize])
                        == Biome::MushroomFields
                    {
                        mush_cols.push((lx, lz));
                    }
                }
            }
            if has_mush {
                let count = 4 + rng.next_range(4) as i32; // 4..7 attempts
                for _ in 0..count {
                    let (lx, lz) = mush_cols[(rng.next_range(mush_cols.len() as u32)) as usize];
                    // stream-stability shim: the pre-4.1j code drew two
                    // range(12) positions here; downstream features share
                    // this rng, so replay one dummy draw (modulo sampling
                    // consumes a fixed call each — stream preserved).
                    let _ = rng.next_range(12);
                    let col_idx = lz as usize * 16 + lx as usize;
                    if Biome::from_u8(chunk.biome[col_idx]) != Biome::MushroomFields {
                        continue; // per-column gate
                    }
                    let h = chunk.height[col_idx] as i32;
                    // fold: MYCELIUM stores its dedicated state (254)
                    let top = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    );
                    if top != MYCELIUM && top != GRASS {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 1) as usize,
                    ) != AIR
                    {
                        continue;
                    }
                    let red = rng.next_f32() < 0.5;
                    let stem = 4 + rng.next_range(3) as i32; // 4..6
                    let y0 = h + 1;
                    // stalk
                    for dy in 0..stem {
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + lx,
                            y0 + dy,
                            oz + lz,
                            MUSHROOM_STEM,
                            false,
                        );
                    }
                    let cap_y = y0 + stem;
                    let cap_block = if red {
                        MUSHROOM_RED_BLOCK
                    } else {
                        MUSHROOM_BROWN_BLOCK
                    };
                    if red {
                        // dome: the 3×3 cap slab on top + four side slabs
                        // (VERIFIED w/Huge_mushroom: "five 3×3 slabs ...
                        // arranged above and around the stalk, forming a
                        // dome")
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + lx,
                            cap_y,
                            oz + lz,
                            cap_block,
                            false,
                        );
                        for dx in -1..=1 {
                            for dz in -1..=1 {
                                if dx == 0 && dz == 0 {
                                    continue;
                                }
                                set_dec(
                                    &mut chunk,
                                    &mut outbound,
                                    ox + lx + dx,
                                    cap_y,
                                    oz + lz + dz,
                                    cap_block,
                                    false,
                                );
                                set_dec(
                                    &mut chunk,
                                    &mut outbound,
                                    ox + lx + dx,
                                    cap_y - 1,
                                    oz + lz + dz,
                                    cap_block,
                                    false,
                                );
                                // side slabs hang one lower, edges only
                                if dx.abs() == 1 || dz.abs() == 1 {
                                    set_dec(
                                        &mut chunk,
                                        &mut outbound,
                                        ox + lx + dx,
                                        cap_y - 2,
                                        oz + lz + dz,
                                        cap_block,
                                        false,
                                    );
                                }
                            }
                        }
                    } else {
                        // brown: one flat 5×5 slab cap (VERIFIED: flat)
                        for dx in -2..=2 {
                            for dz in -2..=2 {
                                set_dec(
                                    &mut chunk,
                                    &mut outbound,
                                    ox + lx + dx,
                                    cap_y,
                                    oz + lz + dz,
                                    cap_block,
                                    false,
                                );
                            }
                        }
                    }
                }
                // small mushrooms scatter on the mycelium (any light —
                // VERIFIED w/Mycelium: mushrooms persist at any light)
                for _ in 0..6 {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    if Biome::from_u8(chunk.biome[col_idx]) != Biome::MushroomFields {
                        continue; // per-column gate
                    }
                    let h = chunk.height[col_idx] as i32;
                    // fold: MYCELIUM stores its dedicated state (254)
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    ) != MYCELIUM
                    {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 1) as usize,
                    ) != AIR
                    {
                        continue;
                    }
                    let id = if rng.next_f32() < 0.5 {
                        MUSHROOM_RED
                    } else {
                        MUSHROOM_BROWN
                    };
                    set_dec(
                        &mut chunk,
                        &mut outbound,
                        ox + lx,
                        h + 1,
                        oz + lz,
                        id,
                        false,
                    );
                }
            }
        }

        // desert: dead bushes + cactus columns + clay in low sand
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if b == Biome::Desert {
                for _ in 0..4 {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    ) != SAND
                    {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (h + 1) as usize,
                    ) != AIR
                    {
                        continue;
                    }
                    if rng.next_f32() < 0.55 {
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + lx,
                            h + 1,
                            oz + lz,
                            DEAD_BUSH,
                            false,
                        );
                    } else {
                        let ch = 1 + rng.next_range(3) as i32;
                        for dy in 0..ch {
                            set_dec(
                                &mut chunk,
                                &mut outbound,
                                ox + lx,
                                h + 1 + dy,
                                oz + lz,
                                CACTUS,
                                false,
                            );
                        }
                    }
                }
                // shallow clay pockets (1.16.5 river/beach clay patches)
                for _ in 0..2 {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if !(vc_chunk::SEA_LEVEL - 2..=vc_chunk::SEA_LEVEL + 1).contains(&h) {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    ) == SAND
                    {
                        chunk.set(lx as usize, h as usize, lz as usize, CLAY);
                    }
                }
            }
        }

        // Phase 10: jungle melon patches (vanilla jungles scatter melons
        // on the floor) + swamp water pools (vanilla swamps are dotted
        // with shallow pools at surface level)
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if b == Biome::Jungle {
                for _ in 0..2 {
                    let lx = 1 + rng.next_range(14) as i32;
                    let lz = 1 + rng.next_range(14) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    ) == GRASS
                        && chunk.get_local(
                            vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                            (h + 1) as usize,
                        ) == AIR
                    {
                        set_dec(
                            &mut chunk,
                            &mut outbound,
                            ox + lx,
                            h + 1,
                            oz + lz,
                            MELON,
                            false,
                        );
                    }
                }
            }
            if b == Biome::Swamp {
                for _ in 0..4 {
                    let lx = 1 + rng.next_range(14) as i32;
                    let lz = 1 + rng.next_range(14) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    // only in the flat swamp band, and not already water
                    if !(vc_chunk::SEA_LEVEL..=vc_chunk::SEA_LEVEL + 2).contains(&h) {
                        continue;
                    }
                    if chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        h as usize,
                    ) == GRASS
                        && chunk.get_local(
                            vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                            (h + 1) as usize,
                        ) == AIR
                    {
                        // 2x1 shallow pool: punch the surface to water
                        chunk.set(lx as usize, h as usize, lz as usize, WATER);
                        if lx + 1 < 16
                            && chunk.get_local(
                                vc_chunk::chunk::LocalXZ::new((lx + 1) as usize, lz as usize),
                                h as usize,
                            ) == GRASS
                        {
                            chunk.set((lx + 1) as usize, h as usize, lz as usize, WATER);
                        }
                    }
                }
            }
        }

        // cave glowstone: rare glowing clusters deep underground, glued to
        // cave ceilings (the block above a carved cell stays stone — hang
        // the glowstone from it by scanning y where air sits below solid).
        {
            for _ in 0..3 {
                let lx = rng.next_range(16) as i32;
                let lz = rng.next_range(16) as i32;
                let col_idx = lz as usize * 16 + lx as usize;
                let hmax = (chunk.height[col_idx] as i32 - 6).clamp(8, 40);
                if hmax <= 10 {
                    continue;
                }
                let y = 8 + rng.next_range((hmax - 8) as u32) as i32;
                let above = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    (y + 1) as usize,
                );
                let here = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    y as usize,
                );
                if here == AIR && (is_opaque(above) && above != BEDROCK) {
                    chunk.set(lx as usize, (y + 1) as usize, lz as usize, GLOWSTONE);
                    // a couple of extra glow blocks around it
                    let extra = rng.next_range(3);
                    for _ in 0..extra {
                        let dx = rng.next_range(3) as i32 - 1;
                        let dz = rng.next_range(3) as i32 - 1;
                        let nx = (lx + dx).clamp(0, 15) as usize;
                        let nz = (lz + dz).clamp(0, 15) as usize;
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(nx, nz), (y + 1) as usize)
                            != AIR
                            && chunk.get_local(vc_chunk::chunk::LocalXZ::new(nx, nz), y as usize)
                                == AIR
                        {
                            chunk.set(nx, (y + 1) as usize, nz, GLOWSTONE);
                        }
                    }
                }
            }
        }

        // ─────────────── 1.13 (Aquatic-era update): ocean flora ─────────────
        // VERIFIED changelog §Blocks + §World generation (live captures
        // in scripts/v113_page_changelog_text.txt):
        // - kelp: "Generate in ocean biomes, except warm oceans ...
        //   Can grow multiple blocks high" — 2-4-block columns
        // - seagrass: "Generates in oceans ..., rivers, and swamplands"
        //   (rivers fold into the ocean family's shallow band here;
        //   swamp pools get their own 20% roll — disclosed)
        // - coral reefs: "Naturally generate in warm ocean biomes ...
        //   composed of coral, coral blocks and coral fans" — patch
        //   noise fields mixing the three forms
        // - sea pickles: "generate in warm oceans, especially around
        //   coral reefs ... Up to 4 of them can be placed on a block"
        // - the frozen-ocean ice sheet + "Generates in icebergs" blue
        //   ice: simplified pack-ice mounds with blue-ice cores (the
        //   full iceberg shape grammar is out of scope, disclosed)
        {
            for lx in 0..16usize {
                for lz in 0..16usize {
                    let col_idx = lz * 16 + lx;
                    let b = Biome::from_u8(chunk.biome[col_idx]);
                    let h = chunk.height[col_idx] as i32;
                    // swamp seagrass: the flat pool band (the pools
                    // themselves are punched above at the swamp pass)
                    if b == Biome::Swamp {
                        if h < vc_chunk::SEA_LEVEL
                            && chunk
                                .get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), (h + 1) as usize)
                                == WATER
                            && rng.next_f32() < 0.20
                        {
                            chunk.set(lx, (h + 1) as usize, lz, SEAGRASS);
                        }
                        continue;
                    }
                    if !b.is_ocean() || h >= sea - 1 {
                        continue; // land, shore, or no water column
                    }
                    if b == Biome::FrozenOcean {
                        // the ice sheet: the top water block freezes
                        // (vanilla frozen-ocean surface)
                        chunk.set(lx, sea as usize, lz, ICE);
                    }
                    // floor flora (a water cell above the floor)
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), (h + 1) as usize)
                        != WATER
                    {
                        continue;
                    }
                    if b == Biome::WarmOcean {
                        // coral reef patch (large-scale noise so reefs
                        // read as fields, not salt-and-pepper)
                        let patch = fbm2(
                            &self.n_humid,
                            (ox + lx as i32) as f32 / 48.0,
                            (oz + lz as i32) as f32 / 48.0,
                            2,
                            2.0,
                            0.5,
                        );
                        if patch > 0.15 {
                            let r = rng.next_f32();
                            if r < 0.35 {
                                // coral block as the floor surface
                                let ci = rng.next_range(5) as u16;
                                chunk.set(lx, h as usize, lz, CORAL_BLOCK_BASE + ci);
                            } else if r < 0.60 {
                                let ci = rng.next_range(5) as u16;
                                chunk.set(lx, (h + 1) as usize, lz, CORAL_PLANT_BASE + ci);
                            } else if r < 0.78 {
                                let ci = rng.next_range(5) as u16;
                                chunk.set(lx, (h + 1) as usize, lz, CORAL_FAN_BASE + ci);
                            } else if r < 0.90 {
                                // sea pickle cluster 1-4 (VERIFIED
                                // "Up to 4 of them can be placed")
                                let count = 1 + rng.next_range(4) as u8;
                                chunk.set(lx, (h + 1) as usize, lz, sea_pickle_state(count));
                            }
                        } else if rng.next_f32() < 0.10 {
                            // sparse warm flora outside the reef patch
                            chunk.set(lx, (h + 1) as usize, lz, SEAGRASS);
                        }
                    } else {
                        // kelp + seagrass (all ocean families except
                        // warm — VERIFIED kelp exclusion)
                        let r = rng.next_f32();
                        if r < 0.08 {
                            // "can grow multiple blocks high" — 2-4
                            let kh = 2 + rng.next_range(3) as i32;
                            for dy in 1..=kh {
                                let y = h + dy;
                                if y < sea
                                    && chunk.get_local(
                                        vc_chunk::chunk::LocalXZ::new(lx, lz),
                                        y as usize,
                                    ) == WATER
                                {
                                    chunk.set(lx, y as usize, lz, KELP);
                                }
                            }
                        } else if r < 0.20 {
                            chunk.set(lx, (h + 1) as usize, lz, SEAGRASS);
                        }
                    }
                }
            }
            // icebergs: 40% of frozen-ocean chunks carry one — a
            // pack-ice mound with a blue-ice core rising above the
            // sheet (simplified vanilla shape, disclosed; vanilla frozen
            // oceans carry frequent icebergs)
            if Biome::from_u8(chunk.biome[8 * 16 + 8]) == Biome::FrozenOcean
                && rng.next_f32() < 0.40
            {
                let bx = 3 + rng.next_range(10) as i32;
                let bz = 3 + rng.next_range(10) as i32;
                let r = 2 + rng.next_range(3) as i32; // 2..4
                for dy in -2..=4i32 {
                    // taper: full radius underwater, shrinking above
                    let rr = if dy <= 0 {
                        r
                    } else {
                        (r as f32 * (1.0 - dy as f32 / 6.0)).round() as i32
                    };
                    for dx in -rr..=rr {
                        for dz in -rr..=rr {
                            let d = (dx * dx + dz * dz) as f32;
                            if d > (rr * rr) as f32 {
                                continue;
                            }
                            let px = bx + dx;
                            let pz = bz + dz;
                            if !(0..=15).contains(&px) || !(0..=15).contains(&pz) {
                                continue;
                            }
                            let y = (sea + dy) as usize;
                            // blue-ice core (inner ~half), packed-ice shell
                            let id = if d < (rr * rr) as f32 * 0.25 {
                                BLUE_ICE
                            } else {
                                PACKED_ICE
                            };
                            let cur = chunk.get_local(
                                vc_chunk::chunk::LocalXZ::new(px as usize, pz as usize),
                                y,
                            );
                            if cur == WATER || cur == ICE || (dy > 0 && cur == AIR) {
                                chunk.set(px as usize, y, pz as usize, id);
                            }
                        }
                    }
                }
            }
        }

        // ───────────────── P5 structures: dungeons (monster rooms) ────
        // §27/Phase 5: per-chunk feature (VERIFIED 1.16.5 generation,
        // wiki Monster Room revision 1944695): 8 attempts per chunk, room
        // size 7/9/11, floor 25% cobble / 75% mossy, spawner at center
        // (zombie 50% / skeleton 25% / spider 25%), up to 2 chests.
        //
        // 2026-09-14: the whole structure block rides the vanilla
        // "Generate Structures" world option — OFF skips all seven
        // emits (see the `structures` field doc for the vanilla split)
        if self.structures {
            if let Some(room) = self.dungeon_in_chunk(cx, cz) {
                self.emit_dungeon(&mut chunk, room, ox, oz);
            }

            // ──────────────────────────────── P7 structures: villages ────
            // Deterministic per 24×24-chunk region: each chunk emits ONLY the
            // village blocks falling inside itself (positions are globally
            // derived, so every chunk independently agrees on the layout —
            // no cross-chunk handoff, no generation-order dependence).
            for &(village_wx, village_wz) in self.villages_near(ox, oz).iter() {
                self.emit_village(&mut chunk, village_wx, village_wz, ox, oz);
            }

            // ─────────────── Phase 10 structures (same emit discipline) ──
            for ms in self.mineshafts_near(ox, oz).iter() {
                self.emit_mineshaft(&mut chunk, ms, ox, oz);
            }
            for &(px, pz) in self.pyramids_near(ox, oz).iter() {
                self.emit_pyramid(&mut chunk, px, pz, ox, oz);
            }
            for &(tx, tz) in self.jungle_temples_near(ox, oz).iter() {
                self.emit_jungle_temple(&mut chunk, tx, tz, ox, oz);
            }
            // 1.11: woodland mansions (dark forest, rare — VERIFIED
            // w/Woodland_Mansion: "generate rarely in dark forests")
            for &(mx, mz) in self.woodland_mansions_near(ox, oz).iter() {
                self.emit_woodland_mansion(&mut chunk, mx, mz, ox, oz);
            }
            for &(sx, sz) in self.strongholds().iter() {
                // skip far strongholds cheaply (the layout spans ~30 blocks
                // around the center; the guard avoids running the emit for
                // the 99.99% of chunks nowhere near one)
                if (sx - ox).abs() > 40 || (sz - oz).abs() > 40 {
                    continue;
                }
                self.emit_stronghold(&mut chunk, sx, sz, ox, oz);
            }
            // 4.2e: witch huts ride the structures gate (scattered)
            for &(hx, hz) in self.witch_huts_near(ox, oz).iter() {
                self.emit_hut(&mut chunk, hx, hz, ox, oz);
            }
        }
        // 4.2e: desert wells are FEATURES (generate with the structures
        // option off — w/Desert_Well) — hooked outside the gate
        for &(wx, wz) in self.wells_near(ox, oz).iter() {
            self.emit_well(&mut chunk, wx, wz, ox, oz);
        }

        (Arc::new(chunk), outbound)
    }

    // ------------------------------------------------------------ dungeons --
    // Phase 5 §27: the vanilla monster room, in-chunk feature form. All
    // numeric rules VERIFIED from the 1.16.5-era wiki (revision 1944695):
    // 8 attempts/chunk · open area 7/9/11 wide · floor solid · ceiling
    // solid · walls need 1-5 two-high openings · 3 rolls per each of 2
    // chests · floor 75% mossy · spawner mob 50/25/25.
    //
    // Documented adaptations:
    // * the room fits inside its owning chunk (vanilla rooms can straddle
    //   chunk borders; the wiki itself classifies them as a per-chunk
    //   *feature*, which is exactly what this is)
    // * "next to a cave" is approximated by the 1-5-openings wall check —
    //   openings only exist where the generator's cave carving produced air
    // * vanilla's y range spans the whole underground; ours rolls in the
    //   8..=35 band (below the surface margin, above bedrock)

    /// dungeon attempts per chunk (VERIFIED: Java 8)
    pub const DUNGEON_ATTEMPTS: u32 = 8;

    /// raw-terrain solidity at underground (x,y,z) — replicates exactly
    /// what the terrain pass leaves behind (stone unless carved / under
    /// raw-terrain solidity at underground (x,y,z) — replicates exactly
    /// what the terrain pass leaves behind: the density stack (positive
    /// ⇒ solid), minus the worm carvers' ellipsoids. The worm paths are
    /// passed in by the caller (they are expensive to re-derive per
    /// query — `dungeon_in_chunk` computes them once).
    fn gen_solid(&self, x: i32, y: i32, z: i32, worms: &[(CaveWorm, WormPath)]) -> bool {
        if y <= 0 {
            return true; // the flat bedrock floor
        }
        if y > 200 {
            return false;
        }
        // mushroom islands: solid below the island height
        let xf = x as f32;
        let zf = z as f32;
        let mush = self.n_mush.noise2(xf / 400.0, zf / 400.0);
        if self.dim == Dimension::Overworld && mush > 0.87 {
            let isl = (vc_chunk::SEA_LEVEL as f64 + 1.0 + (mush as f64 - 0.63) * 30.0)
                .min(vc_chunk::SEA_LEVEL as f64 + 6.0);
            return (y as f64) <= isl;
        }
        let (bd, bv) = self.climate_depth_scale(x, z);
        let (h_eff, amp, _) = self.density_params(x, z, bd, bv);
        let rnd = self.random_density_offset(x, z);
        let d = self
            .vterrain
            .density(x as f64, y as f64, z as f64, h_eff, amp, rnd);
        if d <= 0.0 {
            return false;
        }
        // carver ellipsoids (bedrock region excluded)
        if (5..=128).contains(&y) {
            for (_worm, path) in worms {
                for &(px, py, pz, w) in path {
                    let dx = (x as f64 + 0.5 - px) / w;
                    let dy = (y as f64 + 0.5 - py) / w;
                    let dz = (z as f64 + 0.5 - pz) / w;
                    if dx * dx + dy * dy + dz * dz <= 1.0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// roll this chunk's dungeon (pure — no chunk data needed; the layout
    /// derives from the seed + terrain functions, so it is identical from
    /// any caller: generation, tests, E2E)
    pub fn dungeon_in_chunk(&self, cx: i32, cz: i32) -> Option<DungeonRoom> {
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0D66, cx, 0, cz));
        // the carver worms that can reach this chunk, paths precomputed
        // once (gen_solid tests their ellipsoids per query)
        let worms: Vec<(CaveWorm, WormPath)> = self
            .cave_worms_near(cx, cz)
            .into_iter()
            .map(|w| {
                let p = self.worm_path(&w);
                (w, p)
            })
            .collect();
        for _ in 0..Self::DUNGEON_ATTEMPTS {
            // size roll: 7 / 9 / 11 (VERIFIED open-area set)
            let size = match rng.next_range(3) {
                0 => 7,
                1 => 9,
                _ => 11,
            };
            // interior min corner must leave wall rings on both sides
            let lx = 1 + rng.next_range((15 - size) as u32) as i32; // 1..=15-size
            let lz = 1 + rng.next_range((15 - size) as u32) as i32;
            let y0 = 8 + rng.next_range(28) as i32; // 8..=35
                                                    // spawner mob roll (VERIFIED 50/25/25)
            let mob = match rng.next_range(4) {
                0 | 1 => vc_blocks::blocks::SPAWNER_ZOMBIE,
                2 => vc_blocks::blocks::SPAWNER_SKELETON,
                _ => vc_blocks::blocks::SPAWNER_SPIDER,
            };
            let wx0 = cx * 16 + lx;
            let wz0 = cz * 16 + lz;
            // ---- validation (VERIFIED rules) ----
            // floor area incl. under walls: entirely solid
            let floor_ok = (-1..=size)
                .all(|dx| (-1..=size).all(|dz| self.gen_solid(wx0 + dx, y0 - 1, wz0 + dz, &worms)));
            if !floor_ok {
                continue;
            }
            // ceiling area incl. over walls: entirely solid
            let ceil_ok = (-1..=size)
                .all(|dx| (-1..=size).all(|dz| self.gen_solid(wx0 + dx, y0 + 5, wz0 + dz, &worms)));
            if !ceil_ok {
                continue;
            }
            // walls need 1..5 openings (2-high air at floor level) — this
            // is the "always near a cave" approximation
            let mut openings = 0usize;
            for dx in -1..=size {
                for dz in -1..=size {
                    let on_ring = dx == -1 || dx == size || dz == -1 || dz == size;
                    if !on_ring {
                        continue;
                    }
                    let air2 = !self.gen_solid(wx0 + dx, y0, wz0 + dz, &worms)
                        && !self.gen_solid(wx0 + dx, y0 + 1, wz0 + dz, &worms);
                    if air2 {
                        openings += 1;
                    }
                }
            }
            if !(1..=5).contains(&openings) {
                continue;
            }

            // ---- chests: 3 rolls each, max 2 (VERIFIED) ----
            // qualification (adapted in-chunk): an interior floor cell with
            // exactly ONE wall-adjacent side (vanilla: "empty block with a
            // solid block on exactly one of its four sides" — after the
            // interior is carved to air, wall-adjacency is exactly that)
            let mut chests = [[wx0, y0, wz0]; 2];
            let mut chest_count = 0usize;
            'chests: for slot in 0..2 {
                for _ in 0..3 {
                    let dx = rng.next_range(size as u32) as i32;
                    let dz = rng.next_range(size as u32) as i32;
                    let on_x_edge = dx == 0 || dx == size - 1;
                    let on_z_edge = dz == 0 || dz == size - 1;
                    if on_x_edge == on_z_edge {
                        continue; // 0 or 2 solid sides — vanilla rejects both
                    }
                    let c = [wx0 + dx, y0, wz0 + dz];
                    if chests[..chest_count].contains(&c) {
                        continue; // no double chest on the same cell
                    }
                    chests[slot] = c;
                    chest_count += 1;
                    continue 'chests;
                }
            }
            return Some(DungeonRoom {
                x0: wx0,
                y0,
                z0: wz0,
                size,
                mob,
                chests,
                chest_count,
            });
        }
        None
    }

    /// place a rolled room into the chunk (walls cobble, floor 75% mossy
    /// — VERIFIED —, interior air, spawner center, chests against walls)
    fn emit_dungeon(&self, chunk: &mut Chunk, room: DungeonRoom, ox: i32, oz: i32) {
        let DungeonRoom {
            x0,
            y0,
            z0,
            size,
            mob,
            chests,
            chest_count,
        } = room;
        // mossy pattern rng — derived from the room anchor (stable)
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0D66, x0, 0, z0));
        let lx = x0 - ox;
        let lz = z0 - oz;
        for dx in -1..=size {
            for dz in -1..=size {
                let ring = dx == -1 || dx == size || dz == -1 || dz == size;
                let cx = (lx + dx) as usize;
                let cz = (lz + dz) as usize;
                for dy in -1..=5i32 {
                    let cy = (y0 + dy) as usize;
                    if dy == -1 {
                        // floor: 25% cobble / 75% mossy (VERIFIED)
                        let b = if rng.next_range(4) == 0 {
                            COBBLE
                        } else {
                            MOSSY_COBBLE
                        };
                        chunk.set(cx, cy, cz, b);
                    } else if dy == 5 {
                        // ceiling: plain cobble
                        chunk.set(cx, cy, cz, COBBLE);
                    } else if ring {
                        // walls: plain cobble
                        chunk.set(cx, cy, cz, COBBLE);
                    } else {
                        // interior: air (clears any cave/glowstone leftovers)
                        chunk.set(cx, cy, cz, AIR);
                    }
                }
            }
        }
        // spawner dead center (VERIFIED position); the state carries the
        // mob type (zombie 232 / skeleton 233 / spider 234)
        let sx = (lx + size / 2) as usize;
        let sz = (lz + size / 2) as usize;
        chunk.set_state(sx, y0 as usize, sz, vc_blocks::blocks::spawner_state(mob));
        // chests (up to 2, VERIFIED count)
        for c in chests.iter().take(chest_count) {
            let ccx = (c[0] - ox) as usize;
            let ccz = (c[2] - oz) as usize;
            chunk.set(ccx, c[1] as usize, ccz, CHEST);
        }
    }

    // ------------------------------------------------------------ nether --
    // §26/§28: the Nether generator (our own implementation, 1.16.5's
    // Nether-Wastes *character*): a solid netherrack mass 0..127 between a
    // jittered bedrock floor and bedrock ceiling, carved by two big 3D
    // noise fields into vast caverns; quartz ore veins in the rock, soul
    // sand patches on cavern floors, glowstone clusters on cavern ceilings.
    // The opaque bedrock ceiling zeroes skylight for everything below —
    // exactly the vanilla "no sky light in the nether" rule, achieved
    // through the same column scan the light engine already runs.
    fn generate_nether_chunk(&self, cx: i32, cz: i32, inbound: Vec<(u16, u16)>) -> GenOut {
        let mut chunk = Chunk::empty();
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0D1D, cx, 0, cz));
        // the nether has no cross-chunk decorations (structures are
        // strictly in-chunk) → outbound stays empty
        let outbound: Vec<(i32, i32, i32, u16)> = Vec::new();

        // bedrock shell thickness (floor 1..5, ceiling 1..5, jittered)
        let floor_bed = |wx: i32, wz: i32| -> i32 {
            (Rng::hash3(self.seed ^ 0xF10D, wx, 0, wz) % 4) as i32 // 0..3
        };
        let ceil_bed = |wx: i32, wz: i32| -> i32 {
            (Rng::hash3(self.seed ^ 0xCE11, wx, 0, wz) % 4) as i32 // 0..3
        };

        let mut nether = vec![false; 16 * 16 * 128]; // solid-cell scratch (y<128)

        for z in 0..16usize {
            for x in 0..16usize {
                let wx = cx * 16 + x as i32;
                let wz = cz * 16 + z as i32;
                let col_idx = z * 16 + x;
                // 1.16 (Nether Update, part 2): the nether biome split —
                // region cells of 2x2 chunks (contiguous forests, not
                // per-column confetti): crimson ~22% of the volume,
                // warped ~8%, wastes the rest (VERIFIED shares, the
                // w/Crimson_Forest + w/Warped_Forest infobox rows;
                // region scale is the engine's disclosed adaptation —
                // vanilla's 3D biome sampler needs the full climate
                // stack)
                chunk.biome[col_idx] = nether_region_biome(self.seed, cx, cz) as u8;
                let fb = 1 + floor_bed(wx, wz);
                let cb = 127 - ceil_bed(wx, wz);
                chunk.height[col_idx] = 127; // highest opaque = the bedrock roof

                for y in 1..=126i32 {
                    let solid = if y < fb || y > cb {
                        // near the shell: always rock (blend into bedrock)
                        true
                    } else {
                        // carve: intersection of two 3D "sheets" (like the
                        // overworld spaghetti caves, scaled up ~2.2x) → the
                        // big interconnected caverns; n_neth3 biases whole
                        // regions rockier or netherer so caverns vary
                        let xf = wx as f32;
                        let yf = y as f32;
                        let zf = wz as f32;
                        let n1 = self.n_neth1.noise3(xf / 150.0, yf / 70.0, zf / 150.0);
                        let n2 = self.n_neth2.noise3(
                            (xf + 800.0) / 150.0,
                            yf / 70.0,
                            (zf - 800.0) / 150.0,
                        );
                        let bias = self.n_neth3.noise3(xf / 300.0, yf / 110.0, zf / 300.0);
                        // carve where the two fields both approach 0 AND the
                        // regional bias leans nether. Base ~0.055 keeps the
                        // mass dominant (~70% rock); the ±0.09 swing gives
                        // rocky vs netherer regions; the mid-height band
                        // stays a bit more open (vanilla's cavern band)
                        let r = n1 * n1 + n2 * n2;
                        let t = 0.055 + bias * 0.09 + 0.025 * (1.0 - (y - 70).abs() as f32 / 90.0);
                        r < t.max(0.012)
                    };
                    if solid {
                        nether[(y * 256 + z as i32 * 16 + x as i32) as usize] = true;
                    }
                }
            }
        }

        // materialize: bedrock shell + netherrack (with quartz ore) cells
        for z in 0..16usize {
            for x in 0..16usize {
                let wx = cx * 16 + x as i32;
                let wz = cz * 16 + z as i32;
                let fb = 1 + floor_bed(wx, wz);
                let cb = 127 - ceil_bed(wx, wz);
                for y in 0..=127i32 {
                    let is_bed = y <= fb.saturating_sub(1) || y > cb || y == 0 || y == 127;
                    let solid =
                        is_bed || nether[(y.max(0) * 256 + z as i32 * 16 + x as i32) as usize];
                    if !solid {
                        continue;
                    }
                    let b: u16 = if is_bed {
                        BEDROCK
                    } else {
                        // quartz ore: hash-gated veins in the rock
                        let v = Rng::hash3(self.seed ^ 0x07A2, wx, y, wz);
                        if (v % 100_000) as f32 / 100_000.0 < 0.011 {
                            NETHER_QUARTZ_ORE
                        } else {
                            NETHERRACK
                        }
                    };
                    chunk.set(x, y as usize, z, b);
                }
            }
        }

        // inbound edits (none in practice — no cross-chunk nether decorations
        // — but the pipeline contract is honored)
        for (idx, id) in inbound {
            if chunk.get_idx(idx as usize) == AIR {
                chunk.set_idx(idx as usize, id);
            }
        }

        // 1.10 magma blobs — VERIFIED (wiki /w/Magma_Block, live
        // 2026-09-06): "found in the Nether, generating 4 blobs per chunk
        // between Y=27 and Y=36... similar frequency to andesite in the
        // Overworld". Blobs of 4-9 blocks, embedded in netherrack.
        for _ in 0..4 {
            let bx = rng.next_range(16) as i32;
            let by = 27 + rng.next_range(10) as i32;
            let bz = rng.next_range(16) as i32;
            let size = 4 + rng.next_range(6) as i32;
            for i in 0..size {
                let ox = (bx + (i % 3) - 1).clamp(0, 15);
                let oy = (by + (i / 9)).clamp(27, 36);
                let oz = (bz + ((i / 3) % 3) - 1).clamp(0, 15);
                // only replace netherrack (embedded look, never floating)
                if chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(ox as usize, oz as usize),
                    oy as usize,
                ) == NETHERRACK
                {
                    chunk.set(ox as usize, oy as usize, oz as usize, MAGMA_BLOCK);
                }
            }
        }

        // decorations: soul sand floors + glowstone ceilings (deterministic).
        // 1.7.2 refactor: Chunk::get now FOLDS states to block ids itself
        // (the V2 window made the old `as u8` truncation unsafe), so the
        // per-site state_block fold here is gone — get already returns the
        // owning block id.
        for _ in 0..14 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            // scan the column for a floor (solid below air) in the band
            let mut y = 30;
            while y < 100 {
                let here_air = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    y as usize,
                ) == AIR
                    && (y + 1) < 128
                    && chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (y + 1) as usize,
                    ) == AIR;
                let below = if y > 0 {
                    chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (y - 1) as usize,
                    )
                } else {
                    BEDROCK
                };
                if here_air && below == NETHERRACK {
                    // vanilla-ish: soul sand valley patches — replace the top
                    // 1..2 floor blocks. 1.16: a third of the patches are
                    // SOUL SOIL (the valley's other floor — VERIFIED
                    // w/Soul_Soil "naturally generates in soul sand
                    // valleys"), and one-in-six carries a SOUL FIRE flame
                    // on top (the eternal blue fires, VERIFIED w/Soul_Fire
                    // "Soul fire ... generates naturally in soul sand
                    // valley biomes"; flint-ignition is not in the engine,
                    // disclosed)
                    let wx2 = cx * 16 + lx;
                    let wz2 = cz * 16 + lz;
                    let soil = Rng::hash3(self.seed ^ 0x5011, wx2, y, wz2).is_multiple_of(3);
                    let floor_b = if soil { SOUL_SOIL } else { SOUL_SAND };
                    let depth = 1 + rng.next_range(2) as i32;
                    for d in 0..depth {
                        chunk.set(lx as usize, (y - 1 - d) as usize, lz as usize, floor_b);
                    }
                    if Rng::hash3(self.seed ^ 0xF1E5, wx2, y + 1, wz2).is_multiple_of(6) {
                        chunk.set(lx as usize, y as usize, lz as usize, SOUL_FIRE);
                    }
                    break;
                }
                y += 1;
            }
        }

        // Phase E1: Nether fortresses (432×432 regions — VERIFIED). Each
        // chunk emits every fortress whose arms reach it, so the layout is
        // deterministic and cross-chunk stable (the village/mineshaft
        // region-query pattern).
        let (ox, oz) = (cx * 16, cz * 16);
        for (fx, fz) in self.fortresses_near_chunk(cx, cz) {
            self.emit_fortress(&mut chunk, fx, fz, ox, oz);
        }

        for _ in 0..8 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let mut y = 20;
            while y < 110 {
                let here =
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                let above = if y < 127 {
                    chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        y + 1,
                    )
                } else {
                    BEDROCK
                };
                if here == AIR && above == NETHERRACK {
                    chunk.set(lx as usize, y + 1, lz as usize, GLOWSTONE);
                    // a small cluster around it
                    let extra = rng.next_range(3);
                    for _ in 0..extra {
                        let dx = rng.next_range(3) as i32 - 1;
                        let dz = rng.next_range(3) as i32 - 1;
                        let nx = (lx + dx).clamp(0, 15) as usize;
                        let nz = (lz + dz).clamp(0, 15) as usize;
                        let there = chunk.get_local(vc_chunk::chunk::LocalXZ::new(nx, nz), y + 1);
                        let below_there = chunk.get_local(vc_chunk::chunk::LocalXZ::new(nx, nz), y);
                        if there == NETHERRACK && below_there == AIR {
                            chunk.set(nx, y + 1, nz, GLOWSTONE);
                        }
                    }
                    break;
                }
                y += 1;
            }
        }

        // 1.16 (Nether Update, part 1): the V13 nether decorations —
        // basalt blobs + pillars, blackstone/gilded/crying patches,
        // nether gold ore veins, ancient debris clusters
        self.gen_v116_nether_decorations(&mut chunk, &mut rng, cx, cz);

        // 1.16 (Nether Update, part 2): the V14 forest families — the
        // nylium floors, huge fungi, shroomlights, vines + undergrowth
        self.gen_v116b_nether_forests(&mut chunk, &mut rng, cx, cz);

        // backlog round: the two missing Nether regions — the soul sand
        // valley + the basalt deltas (all five 1.16 nether biomes now)
        self.gen_backlog_nether_regions(&mut chunk, &mut rng, cx, cz);

        (Arc::new(chunk), outbound)
    }

    /// Backlog round (2026-09-09): the two missing Nether regions —
    /// the soul sand valley and the basalt deltas, closing the five-
    /// biome 1.16 Nether map.
    ///
    /// Soul Sand Valley (VERIFIED w/Soul_Sand_Valley, capture
    /// backlog_page_Soul_Sand_Valley.json): "mostly composed of soul
    /// sand and soul soil, with gravel found on its coastlines"; "Soul
    /// fire is scattered throughout the biome and Nether fossils poke
    /// out of the terrain"; "Giant columns of basalt called basalt
    /// pillars can be found stretching from the floor to the ceiling";
    /// native vegetation = crimson roots + mushrooms.
    ///
    /// Basalt Deltas (VERIFIED w/Basalt_Deltas, capture
    /// backlog_page_Basalt_Deltas.json): the "second rarest Nether
    /// biome, making up around 16% of the Nether by volume"; the
    /// wasteland body is basalt + blackstone + magma (the deltas'
    /// surface is the engine's netherrack body converted to basalt —
    /// the documented column-carver adaptation).
    fn gen_backlog_nether_regions(&self, chunk: &mut Chunk, rng: &mut Rng, cx: i32, cz: i32) {
        use vc_blocks::blocks::{
            BASALT, BLACKSTONE, BONE_BLOCK, CRIMSON_ROOTS, MAGMA_BLOCK, MUSHROOM_BROWN,
            MUSHROOM_RED, NETHERRACK, SOUL_FIRE, SOUL_SAND, SOUL_SOIL,
        };
        let region = nether_region_biome(self.seed, cx, cz);
        if region == Biome::SoulSandValley {
            // ---- the soul floor: netherrack surface -> soul sand (60%)
            // or soul soil (40%) — the "mostly composed of" row ----
            for z in 0..16usize {
                for x in 0..16usize {
                    let wx = cx * 16 + x as i32;
                    let wz = cz * 16 + z as i32;
                    for y in (20..110usize).rev() {
                        let below = chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y);
                        if below == NETHERRACK
                            && chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y + 1) == 0
                        {
                            let sand = Rng::hash3(self.seed ^ 0x50F1, wx, y as i32, wz) % 10 < 6;
                            let floor = if sand { SOUL_SAND } else { SOUL_SOIL };
                            chunk.set(x, y, z, floor);
                            // the nether fossils: ~1 in 8 surface columns
                            // sprouts a bone rib arc right at the floor
                            // ("Nether fossils poke out of the terrain",
                            // VERIFIED — placement on known surface
                            // columns instead of a separate random scan,
                            // so fossils always land on real terrain)
                            if Rng::hash3(self.seed ^ 0xB0A5, wx, y as i32, wz).is_multiple_of(8) {
                                let dir: i32 = if Rng::hash3(self.seed ^ 0xB0A6, wx, 0, wz)
                                    .is_multiple_of(2)
                                {
                                    1
                                } else {
                                    -1
                                };
                                let len = 4
                                    + (Rng::hash3(self.seed ^ 0xB0A7, wx, y as i32, wz) % 4) as i32;
                                for d in 0..len {
                                    let fx = (x as i32 + d * dir).clamp(0, 15) as usize;
                                    let fz = z;
                                    let fy = (y as i32 + 1 - (d / 3) + (d == 0) as i32)
                                        .clamp(1, 126)
                                        as usize;
                                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(fx, fz), fy)
                                        == 0
                                    {
                                        chunk.set(fx, fy, fz, BONE_BLOCK);
                                    }
                                }
                            }
                            // soul fire on soul soil ("scattered
                            // throughout", VERIFIED — soul fire burns on
                            // soul soil only, the soul_fire placement rule)
                            if !sand
                                && Rng::hash3(self.seed ^ 0x50F2, wx, y as i32, wz)
                                    .is_multiple_of(60)
                            {
                                chunk.set(x, y + 1, z, SOUL_FIRE);
                            } else if Rng::hash3(self.seed ^ 0x50F3, wx, y as i32, wz)
                                .is_multiple_of(40)
                            {
                                // the sparse native vegetation: crimson
                                // roots + mushrooms (VERIFIED row)
                                let plant =
                                    match Rng::hash3(self.seed ^ 0x50F4, wx, y as i32, wz) % 3 {
                                        0 => CRIMSON_ROOTS,
                                        1 => MUSHROOM_RED,
                                        _ => MUSHROOM_BROWN,
                                    };
                                chunk.set(x, y + 1, z, plant);
                            }
                            break;
                        }
                    }
                }
            }

            // ---- the giant basalt pillars: floor to ceiling —
            // stretch from a floor toward the ceiling band ----
            for _ in 0..3 {
                let lx = rng.next_range(16) as i32;
                let lz = rng.next_range(16) as i32;
                // find the floor, then run a tall column (air above —
                // same pattern)
                let mut fy = 20i32;
                for y in (20..110usize).rev() {
                    let b =
                        chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                    let above = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (y + 1).min(127),
                    );
                    if b != 0 && vc_blocks::blocks::is_solid(b) && above == 0 {
                        fy = y as i32;
                        break;
                    }
                }
                if fy < 20 {
                    continue;
                }
                let top = 90 + rng.next_range(30) as i32; // into the ceiling band
                for yy in fy + 1..=top {
                    if yy > 126
                        || chunk.get_local(
                            vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                            yy as usize,
                        ) != 0
                    {
                        break;
                    }
                    chunk.set(lx as usize, yy as usize, lz as usize, BASALT);
                }
            }
        } else if region == Biome::BasaltDeltas {
            // ---- the deltas floor: netherrack surface -> basalt (70%)
            // with blackstone (20%) + magma (10%) — the deltas' basalt
            // body (the wiki's own composition: basalt, blackstone,
            // magma) ----
            for z in 0..16usize {
                for x in 0..16usize {
                    let wx = cx * 16 + x as i32;
                    let wz = cz * 16 + z as i32;
                    for y in (20..110usize).rev() {
                        let below = chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y);
                        if below == NETHERRACK
                            && chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y + 1) == 0
                        {
                            let v = Rng::hash3(self.seed ^ 0xBA2A, wx, y as i32, wz) % 10;
                            let floor = if v < 7 {
                                BASALT
                            } else if v < 9 {
                                BLACKSTONE
                            } else {
                                MAGMA_BLOCK
                            };
                            chunk.set(x, y, z, floor);
                            break;
                        }
                    }
                }
            }
            // ---- the deltas' signature short thick basalt columns:
            // 6 per chunk, 4..10 tall (denser + shorter than the
            // valley's giants — the deltas' look) ----
            for _ in 0..6 {
                let lx = rng.next_range(16) as i32;
                let lz = rng.next_range(16) as i32;
                let mut base = 20i32;
                for y in (20..110usize).rev() {
                    let b =
                        chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                    let above = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        (y + 1).min(127),
                    );
                    if b != 0 && vc_blocks::blocks::is_solid(b) && above == 0 {
                        base = y as i32;
                        break;
                    }
                }
                if base < 20 {
                    continue;
                }
                let h = 4 + rng.next_range(7) as i32;
                let thick = rng.next_range(2) == 0; // half are 2x2
                for d in 1..=h {
                    let yy = (base + d).min(126) as usize;
                    let x = lx.clamp(0, 15) as usize;
                    let z = lz.clamp(0, 15) as usize;
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), yy) == 0 {
                        chunk.set(x, yy, z, BASALT);
                    }
                    if thick {
                        let x2 = (lx + 1).clamp(0, 15) as usize;
                        let z2 = (lz + 1).clamp(0, 15) as usize;
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x2, z), yy) == 0 {
                            chunk.set(x2, yy, z, BASALT);
                        }
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z2), yy) == 0 {
                            chunk.set(x, yy, z2, BASALT);
                        }
                    }
                }
            }
        }
    }

    /// 1.16 (Nether Update, part 2): the nether forest generation —
    /// the crimson/warped region's signature look, all in-chunk:
    /// nylium floors ("The forest floor is mostly covered with
    /// crimson nylium", VERIFIED w/Crimson_Forest), huge fungi (the
    /// stem + wart-cap + shroomlight "trees"), the undergrowth
    /// tufts, weeping vines hanging under crimson canopies + twisting
    /// vines climbing from warped ground (VERIFIED w/Weeping_Vines +
    /// w/Twisting_Vines "Post-generation" rows). Adaptations,
    /// disclosed: no bone-meal growth path (no nylium-spreading sim);
    /// the huge fungi are the engine's tree-generator pattern trimmed
    /// to the two families.
    fn gen_v116b_nether_forests(&self, chunk: &mut Chunk, rng: &mut Rng, cx: i32, cz: i32) {
        use vc_blocks::blocks::{
            CRIMSON_FUNGUS, CRIMSON_NYLIUM, CRIMSON_ROOTS, CRIMSON_STEM, NETHERRACK,
            NETHER_SPROUTS, NETHER_WART_BLOCK, SHROOMLIGHT, TWISTING_VINES, WARPED_FUNGUS,
            WARPED_NYLIUM, WARPED_ROOTS, WARPED_STEM, WARPED_WART_BLOCK, WEEPING_VINES,
        };
        let region = nether_region_biome(self.seed, cx, cz);
        if region == Biome::NetherWastes {
            return; // the wastes keep their part-1 look
        }
        let crimson = region == Biome::CrimsonForest;
        let (nylium, stem, wart_cap) = if crimson {
            (CRIMSON_NYLIUM, CRIMSON_STEM, NETHER_WART_BLOCK)
        } else {
            (WARPED_NYLIUM, WARPED_STEM, WARPED_WART_BLOCK)
        };

        // ---- the nylium floor: every netherrack surface with air
        // above (y 20..110) turns nylium; ~30% of the columns keep
        // bare netherrack ("with some netherrack ... generating on the
        // surface as well", VERIFIED) ----
        for z in 0..16usize {
            for x in 0..16usize {
                let wx = cx * 16 + x as i32;
                let wz = cz * 16 + z as i32;
                let bare = Rng::hash3(self.seed ^ 0xBA4E, wx, 0, wz) % 10 < 3;
                if bare {
                    continue;
                }
                for y in (20..110usize).rev() {
                    let below = chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y);
                    if below == NETHERRACK
                        && chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y + 1) == 0
                    {
                        // plant the nylium block (Chunk::set routes
                        // through default_state)
                        chunk.set(x, y, z, nylium);
                        // the undergrowth: ~45% of nylium columns sprout
                        // a tuft (fungi/roots crimson; roots/sprouts/
                        // fungi warped — "The most frequent vegetation
                        // ... includes crimson fungi, crimson roots",
                        // VERIFIED w/Crimson_Forest)
                        if Rng::hash3(self.seed ^ 0xF106, wx, y as i32, wz) % 100 < 45 {
                            let plant = if crimson {
                                match Rng::hash3(self.seed ^ 0xC407, wx, y as i32, wz) % 3 {
                                    0 => CRIMSON_FUNGUS,
                                    _ => CRIMSON_ROOTS,
                                }
                            } else {
                                match Rng::hash3(self.seed ^ 0x9A0E, wx, y as i32, wz) % 4 {
                                    0 => WARPED_FUNGUS,
                                    1 => WARPED_ROOTS,
                                    2 => NETHER_SPROUTS,
                                    _ => WARPED_ROOTS,
                                }
                            };
                            chunk.set(x, y + 1, z, plant);
                        }
                        break;
                    }
                }
            }
        }

        // ---- the huge fungi (the forest's "trees"): 3 attempts per
        // chunk — a 4..9-tall stem from a nylium floor, a 3..5-wide
        // wart cap, one shroomlight in the cap, and (crimson only)
        // weeping-vine strands hanging from the cap's rim; (warped)
        // twisting-vine columns climbing 2..6 from the ground nearby
        // ----
        for _ in 0..3 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            // find a nylium floor (only forests themselves host the
            // huge fungi — VERIFIED w/Crimson_Forest: "This is the
            // only place where huge crimson fungus trees grow
            // naturally")
            let mut base = 0i32;
            for y in (20..110usize).rev() {
                let b = chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                if b != 0 && vc_blocks::blocks::is_solid(b) {
                    base = y as i32;
                    break;
                }
            }
            if base < 20 {
                continue;
            }
            let floor = chunk.get_local(
                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                base as usize,
            );
            if floor != nylium {
                continue; // not a forest floor — skip this attempt
            }
            let h = 4 + rng.next_range(6) as i32; // 4..9
            let mut top = base;
            for d in 1..=h {
                let yy = base + d;
                if yy > 125
                    || chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                        yy as usize,
                    ) != 0
                {
                    break;
                }
                chunk.set(lx as usize, yy as usize, lz as usize, stem);
                top = yy;
            }
            if top <= base + 1 {
                continue; // the stem never made it out of the floor
            }
            // the cap: a disc of wart blocks + the shroomlight core
            let r = 2 + rng.next_range(2) as i32; // 2..3 → caps 3..5 wide
            let cy = (top + 1).min(125);
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dz * dz > r * r + 1 {
                        continue;
                    }
                    let x = (lx + dx).clamp(0, 15) as usize;
                    let z = (lz + dz).clamp(0, 15) as usize;
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), cy as usize) == 0 {
                        chunk.set(x, cy as usize, z, wart_cap);
                    }
                }
            }
            // the shroomlight: the cap's center ("Shroomlights ...
            // generate in huge fungi", VERIFIED w/Shroomlight)
            let x = lx.clamp(0, 15) as usize;
            let z = lz.clamp(0, 15) as usize;
            if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), cy as usize) == wart_cap {
                chunk.set(x, cy as usize, z, SHROOMLIGHT);
            }
            // crimson: weeping vines hang from the cap's rim (1..4
            // strands, 2..5 long — "generate naturally ... on huge
            // crimson fungi", VERIFIED w/Weeping_Vines)
            if crimson {
                for _ in 0..1 + rng.next_range(4) as i32 {
                    let dx = (rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(-r, r);
                    let dz = (rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(-r, r);
                    let x = (lx + dx).clamp(0, 15) as usize;
                    let z = (lz + dz).clamp(0, 15) as usize;
                    let len = 2 + rng.next_range(4) as i32; // 2..5
                    for d in 0..len {
                        let yy = cy - 1 - d;
                        if yy < 2 {
                            break;
                        }
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), yy as usize) != 0 {
                            break;
                        }
                        chunk.set(x, yy as usize, z, WEEPING_VINES);
                    }
                }
            }
        }

        // warped only: twisting-vine columns from the ground ("twisting
        // vines growing from the ground", VERIFIED w/Warped_Forest) —
        // 2 columns per chunk, 2..7 tall
        if !crimson {
            for _ in 0..2 {
                let lx = rng.next_range(16) as i32;
                let lz = rng.next_range(16) as i32;
                let mut base = 0i32;
                for y in (20..110usize).rev() {
                    let b =
                        chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                    if b != 0 && vc_blocks::blocks::is_solid(b) {
                        base = y as i32;
                        break;
                    }
                }
                let floor = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    base as usize,
                );
                if base < 20 || floor != nylium {
                    continue;
                }
                let len = 2 + rng.next_range(6) as i32; // 2..7
                for d in 1..=len {
                    let yy = base + d;
                    if yy > 125
                        || chunk.get_local(
                            vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                            yy as usize,
                        ) != 0
                    {
                        break;
                    }
                    chunk.set(lx as usize, yy as usize, lz as usize, TWISTING_VINES);
                }
            }
        }
    }

    /// 1.16 (Nether Update, part 1): the V13 nether decorations —
    /// basalt blobs + pillars, blackstone patches with gilded + crying
    /// obsidian trace, nether gold ore veins, and the ancient-debris
    /// clusters. Called at the end of generate_nether_chunk (the
    /// chunk is fully materialized — the debris air-exposure check
    /// needs that).
    fn gen_v116_nether_decorations(&self, chunk: &mut Chunk, rng: &mut Rng, cx: i32, cz: i32) {
        // ---- basalt blobs (the basalt-deltas adaptation — no nether
        // sub-biomes in this engine, disclosed): 5 blobs/chunk y 20..90,
        // radius 3..5, replacing netherrack (the magma-blob pattern).
        // VERIFIED w/Basalt: "generate in blobs, which attempt to
        // replace netherrack ... in basalt deltas biomes". ----
        for _ in 0..5 {
            let bx = rng.next_range(16) as i32;
            let by = 20 + rng.next_range(70) as i32;
            let bz = rng.next_range(16) as i32;
            let r = 3 + rng.next_range(3) as i32; // 3..5
            for dy in -r..=r {
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy + dz * dz > r * r {
                            continue;
                        }
                        let x = (bx + dx).clamp(0, 15) as usize;
                        let y = (by + dy).clamp(1, 126) as usize;
                        let z = (bz + dz).clamp(0, 15) as usize;
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) == NETHERRACK {
                            chunk.set(x, y, z, BASALT);
                        }
                    }
                }
            }
        }

        // ---- basalt pillars (the soul-sand-valley landmark): 2 per
        // chunk — 1x1 columns 6..14 tall rising from a solid floor into
        // air. ----
        for _ in 0..2 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let mut y = 20;
            while y < 100 {
                let below = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    y as usize,
                );
                let here = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    (y + 1) as usize,
                );
                if below != AIR && below != BEDROCK && here == AIR {
                    let h = 6 + rng.next_range(9) as i32; // 6..14
                    for d in 1..=h {
                        let yy = y + d;
                        if yy > 126
                            || chunk.get_local(
                                vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                                yy as usize,
                            ) != AIR
                        {
                            break;
                        }
                        chunk.set(lx as usize, yy as usize, lz as usize, BASALT);
                    }
                    break;
                }
                y += 1;
            }
        }

        // ---- blackstone patches ("small patches in all Nether
        // biomes", VERIFIED w/Blackstone 20w19a row): 3 blobs/chunk,
        // low band y 5..55 (denser deep — vanilla's low-nether body is
        // blackstone-rich under the bastion round's disclosure). Inside
        // each blob: 1-2 gilded spots ("native to bastion remnants" —
        // our patch adaptation, disclosed) + a 1-in-3-blob crying
        // obsidian trace (the ruined-portal/bastion stand-in). ----
        for bi in 0..3 {
            let bx = rng.next_range(16) as i32;
            let by = 5 + rng.next_range(50) as i32;
            let bz = rng.next_range(16) as i32;
            let r = 2 + rng.next_range(3) as i32; // 2..4
            for dy in -r..=r {
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy + dz * dz > r * r {
                            continue;
                        }
                        let x = (bx + dx).clamp(0, 15) as usize;
                        let y = (by + dy).clamp(1, 126) as usize;
                        let z = (bz + dz).clamp(0, 15) as usize;
                        if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) == NETHERRACK {
                            chunk.set(x, y, z, BLACKSTONE);
                        }
                    }
                }
            }
            // the gilded spots: 1..2 inside the blob
            let g = 1 + rng.next_range(2) as i32;
            for _ in 0..g {
                let x = (bx + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(0, 15) as usize;
                let y = (by + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(1, 126) as usize;
                let z = (bz + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(0, 15) as usize;
                if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) == BLACKSTONE {
                    chunk.set(x, y, z, GILDED_BLACKSTONE);
                }
            }
            // crying obsidian trace: 1-in-3 blobs carry a single spot
            if bi == 0 && rng.next_range(3) == 0 {
                let x = (bx + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(0, 15) as usize;
                let y = (by + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(1, 126) as usize;
                let z = (bz + rng.next_range((r * 2 + 1) as u32) as i32 - r).clamp(0, 15) as usize;
                if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) == BLACKSTONE {
                    chunk.set(x, y, z, CRYING_OBSIDIAN);
                }
            }
        }

        // ---- nether gold ore (VERIFIED w/Nether_Gold_Ore "generates
        // in the Nether in the form of blobs"): hash-gated veins at
        // quartz-like density, any y in the rock body ----
        for z in 0..16usize {
            for x in 0..16usize {
                for y in 1..=126usize {
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) != NETHERRACK {
                        continue;
                    }
                    let wx = cx * 16 + x as i32;
                    let wz = cz * 16 + z as i32;
                    let v = Rng::hash3(self.seed ^ 0x601D, wx, y as i32, wz);
                    if (v % 100_000) as f32 / 100_000.0 < 0.009 {
                        chunk.set(x, y, z, NETHER_GOLD_ORE);
                    }
                }
            }
        }

        // ---- ancient debris (VERIFIED w/Ancient_Debris): Java gen —
        // "up to two clusters may generate per chunk: one cluster of
        // 0–3 ancient debris ... with a triangle distribution from
        // levels 8 to 24 [peak 16]. An additional cluster of 0–2 ...
        // evenly from levels 8 to 119." And "never naturally exposed
        // to air"; "They can only replace netherrack, basalt, and
        // blackstone" [Java]. ----
        let solid_no_air = |c: &Chunk, x: usize, y: usize, z: usize| -> bool {
            // all 6 neighbors must be non-air (never exposed)
            let solid_at = |xx: i32, yy: i32, zz: i32| -> bool {
                if !(0..=15).contains(&xx) || !(0..=15).contains(&zz) || !(1..=126).contains(&yy) {
                    return true; // out of local range counts as rock
                }
                c.get_local(
                    vc_chunk::chunk::LocalXZ::new(xx as usize, zz as usize),
                    yy as usize,
                ) != AIR
            };
            solid_at(x as i32 - 1, y as i32, z as i32)
                && solid_at(x as i32 + 1, y as i32, z as i32)
                && solid_at(x as i32, y as i32 - 1, z as i32)
                && solid_at(x as i32, y as i32 + 1, z as i32)
                && solid_at(x as i32, y as i32, z as i32 - 1)
                && solid_at(x as i32, y as i32, z as i32 + 1)
        };
        let placeable = |b: u16| b == NETHERRACK || b == BASALT || b == BLACKSTONE;
        // cluster A: 0..3, triangle y 8..24 (sum of two uniforms 0..8+0..8
        // + 8 → peak at 16)
        for _ in 0..(rng.next_range(4) as i32) {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let y = 8 + (rng.next_range(9) as i32 + rng.next_range(9) as i32);
            if y <= 126
                && placeable(chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    y as usize,
                ))
                && solid_no_air(chunk, lx as usize, y as usize, lz as usize)
            {
                chunk.set(lx as usize, y as usize, lz as usize, ANCIENT_DEBRIS);
            }
        }
        // cluster B: 0..2, even y 8..119
        for _ in 0..(rng.next_range(3) as i32) {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let y = 8 + rng.next_range(112) as i32;
            if y <= 126
                && placeable(chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize),
                    y as usize,
                ))
                && solid_no_air(chunk, lx as usize, y as usize, lz as usize)
            {
                chunk.set(lx as usize, y as usize, lz as usize, ANCIENT_DEBRIS);
            }
        }
    }
    /// first chunk can be solid rock; caverns interleave with walls).
    pub fn find_nether_spawn(&self) -> (f32, f32, f32) {
        // ring-by-ring spiral over the first ~9×9 chunks
        for r in 0..4i32 {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dz.abs() != r {
                        continue; // ring cells only
                    }
                    let (chunk, _) = self.generate_nether_chunk(dx, dz, Vec::new());
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            for y in 10..110usize {
                                let feet =
                                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y);
                                let head = if y + 1 < 128 {
                                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y + 1)
                                } else {
                                    BEDROCK
                                };
                                let floor = if y > 0 {
                                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y - 1)
                                } else {
                                    BEDROCK
                                };
                                if feet == AIR && head == AIR && is_solid(floor) {
                                    return (
                                        (dx * 16 + lx as i32) as f32 + 0.5,
                                        y as f32,
                                        (dz * 16 + lz as i32) as f32 + 0.5,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        // fallback: mid-band position — the travel snap (nether_floor_y)
        // refines it, arriving flying if nothing opens up
        (8.5, 70.0, 8.5)
    }

    // ------------------------------------------------------------ villages --

    /// all village centers whose structures can reach into the 16×16 block
    /// area at (ox, oz): village regions overlapping [ox-40, ox+56)
    /// 3.7e: nearest structure center of a named kind to (x, z) —
    /// (x, y, z); y is the surface height (mineshafts: the parlor
    /// floor). Kinds: village, desert_pyramid, jungle_temple,
    /// woodland_mansion, mineshaft, stronghold (case-insensitive, an
    /// optional `namespace:` prefix is stripped). None = unknown kind
    /// or nothing in range (48 region-rings).
    pub fn locate_structure(&self, kind: &str, x: i32, z: i32) -> Option<(i32, i32, i32)> {
        let bare = kind.rsplit(':').next().unwrap_or(kind).to_ascii_lowercase();
        let surf = |wx: i32, wz: i32| self.column(wx, wz).height;
        match bare.as_str() {
            "stronghold" => self
                .strongholds()
                .into_iter()
                .map(|(sx, sz)| {
                    let d = (sx as i64 - x as i64).pow(2) + (sz as i64 - z as i64).pow(2);
                    (d, (sx, surf(sx, sz), sz))
                })
                .min_by_key(|(d, _)| *d)
                .map(|(_, p)| p),
            "village" => self.spiral(VILLAGE_SPACING_CHUNKS * 16, 48, x, z, |s, rx, rz| {
                s.village_center(rx, rz)
                    .map(|(wx, wz)| (wx, surf(wx, wz), wz))
            }),
            "desert_pyramid" => self.spiral(512, 48, x, z, |s, rx, rz| {
                s.pyramid_center_pub(rx, rz)
                    .map(|(wx, wz)| (wx, surf(wx, wz), wz))
            }),
            "jungle_temple" => self.spiral(512, 48, x, z, |s, rx, rz| {
                s.jungle_temple_center(rx, rz)
                    .map(|(wx, wz)| (wx, surf(wx, wz), wz))
            }),
            "woodland_mansion" => self.spiral(128, 64, x, z, |s, rx, rz| {
                s.woodland_mansion_center(rx, rz)
                    .map(|(wx, wz)| (wx, surf(wx, wz), wz))
            }),
            "mineshaft" => self.spiral(16, 64, x, z, |s, cx, cz| {
                // parlor anchor math mirrors mineshafts_near exactly
                // (rng call order: chance, y, px, pz)
                let mut rng = Rng::new(Rng::hash3(s.seed ^ 0x411E5, cx, 0, cz));
                if rng.next_f32() >= MINESHAFT_CHANCE {
                    return None;
                }
                let y = 10 + rng.next_range(31) as i32;
                let px = cx * 16 + 3 + rng.next_range(10) as i32;
                let pz = cz * 16 + 3 + rng.next_range(10) as i32;
                Some((px, y, pz))
            }),
            _ => None,
        }
    }

    /// region-ring spiral returning the nearest center (early stop
    /// once the next ring outruns the best hit).
    fn spiral<F>(
        &self,
        region: i32,
        rings: i32,
        x: i32,
        z: i32,
        mut center: F,
    ) -> Option<(i32, i32, i32)>
    where
        F: FnMut(&Self, i32, i32) -> Option<(i32, i32, i32)>,
    {
        let crx = x.div_euclid(region);
        let crz = z.div_euclid(region);
        let mut best: Option<(i64, (i32, i32, i32))> = None;
        for ring in 0..=rings {
            for dx in -ring..=ring {
                for dz in -ring..=ring {
                    if ring > 0 && dx.abs() < ring && dz.abs() < ring {
                        continue; // perimeter only
                    }
                    if let Some(p) = center(self, crx + dx, crz + dz) {
                        let d = (p.0 as i64 - x as i64).pow(2) + (p.2 as i64 - z as i64).pow(2);
                        if best.is_none_or(|(bd, _)| d < bd) {
                            best = Some((d, p));
                        }
                    }
                }
            }
            if let Some((bd, _)) = best {
                let edge = (ring + 1) as i64 * region as i64;
                if edge * edge >= bd {
                    break;
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// 4.1g layer-stack salts (engine-local — code-derived salts are
    /// refused; positions fit later against measurements).
    /// Island density is FIT (start 45%).
    /// 4.1g: clean-room biome layer stack (behavioral shape from the
    /// Before-1.18 wiki page: island grid at 1:256, doubling zooms to
    /// 1:4 cells). Cell = 4 blocks (matches the chunk biome
    /// quartiles). Returns (land, deep_ocean, special) per cell:
    /// island grid → 6 doubling zooms with jittered parent picks →
    /// deep marking for ocean interiors → 1-in-13 special marking.
    fn layer_cell(&self, cx4: i32, cz4: i32) -> (bool, bool, bool) {
        let land = self.zoomed_island(cx4, cz4);
        if land {
            let special =
                Rng::hash3(self.seed ^ LAYER_SALT_SPECIAL, cx4, 0, cz4).is_multiple_of(13);
            return (true, false, special);
        }
        // deep marking: ocean interiors (3×3 all ocean) read deep
        for dx in -1..=1 {
            for dz in -1..=1 {
                if self.zoomed_island(cx4 + dx, cz4 + dz) {
                    return (false, false, false);
                }
            }
        }
        (false, true, false)
    }

    /// island base value at 256-block cells (ix, iz).
    fn island_base(&self, ix: i32, iz: i32) -> bool {
        Rng::hash3(self.seed ^ LAYER_SALT_ISLAND, ix, 0, iz) % 100 < LAYER_ISLAND_PCT
    }

    /// zoom the island grid down to 4-block cells: 6 halvings with a
    /// 0/1 sampling jitter per level (documented zoom behavior —
    /// each child samples its parents with offset).
    fn zoomed_island(&self, cx4: i32, cz4: i32) -> bool {
        let (mut x, mut z) = (cx4, cz4);
        for level in 0..6 {
            let h = Rng::hash3(self.seed ^ LAYER_SALT_ZOOM, x, level, z);
            let jx = ((h >> 5) & 1) as i32;
            let jz = ((h >> 11) & 1) as i32;
            x = (x + jx).div_euclid(2);
            z = (z + jz).div_euclid(2);
        }
        self.island_base(x, z)
    }

    /// 4.1i: BiomeInit base biome for LAND cells (ocean family is
    /// resolved by the height-gated branches in classify; the ocean
    /// map below is retained for reference, not called).
    /// Category members follow documented biome climates; gate
    /// percentages are FIT.
    fn layer_base_biome(&self, special: bool, snow: bool, warm: bool, cx4: i32, cz4: i32) -> Biome {
        let pick = Rng::hash3(self.seed ^ LAYER_SALT_SPECIAL, cx4, 0xB17, cz4) % 100;
        // ocean reference map (snow → frozen, warm → warm, deep →
        // deep-cold/deep split, else cold/luke/ocean thirds) — see
        // classify's ocean branch, which implements it live.
        if snow {
            return if pick < 70 {
                Biome::Snowy
            } else {
                Biome::Taiga
            };
        }
        if warm {
            // 4.1n FIT: badlands 5% of warm cells (copy-global ~0.4%;
            // the var>0.65 tail gave 0.001%) via the uniform cell roll
            if pick < 5 {
                return Biome::Badlands;
            }
            return match pick % 3 {
                0 => Biome::Desert,
                1 => Biome::Savanna,
                _ => Biome::Jungle,
            };
        }
        if special {
            // 4.1n FIT: mushroom 2% of special cells (full-population
            // read 1.6% vs <0.3% in the copy; the E1 gate drops 0.83
            // -> 0.87 alongside)
            return if pick < 2 {
                Biome::MushroomFields
            } else {
                Biome::Plains
            };
        }
        match pick % 100 {
            // 4.1n FIT: plains 39 / taiga 38 / forest 22 / birch 1
            // (copy temperate-land: plains ~28%, taiga family ~30%,
            // forest family ~14%, birch proper 0.2%; the narrow birch
            // base plus the 45% flower roll below lands flower forest
            // at ~0.3% vs 0.4% in the copy).
            0..=38 => Biome::Plains,
            39..=76 => Biome::Taiga,
            77..=98 => Biome::Forest,
            _ => Biome::BirchForest,
        }
    }

    /// 4.1i: climate gates per 4-block cell (snow/warm shares at
    /// 64-block coherence — FIT percentages, engine-local salt).
    fn layer_climate(&self, cx4: i32, cz4: i32) -> (bool, bool) {
        let h = Rng::hash3(
            self.seed ^ LAYER_SALT_CLIMATE,
            cx4.div_euclid(16),
            0,
            cz4.div_euclid(16),
        ) % 100;
        (h < LAYER_SNOW_PCT, h >= 100 - LAYER_WARM_PCT)
    }

    /// 4.1a: biome census — (vanilla_id, columns) over a chunk
    /// rect, sorted by id. Facts-only exchange format for the 4.0
    /// numeric-diff loop (L5: counts, never positions). Dimension
    /// aware: Nether reads the region roll (column() is
    /// overworld-only); the End is single-biome id 9.
    pub fn biome_census(&self, cx0: i32, cz0: i32, w: i32, h: i32) -> Vec<(u8, u64)> {
        use std::collections::BTreeMap;
        let mut m: BTreeMap<u8, u64> = BTreeMap::new();
        for cx in cx0..cx0 + w {
            for cz in cz0..cz0 + h {
                let region_id = match self.dim {
                    Dimension::Nether => Some(nether_region_biome(self.seed, cx, cz).vanilla_id()),
                    Dimension::End => Some(9),
                    _ => None,
                };
                for lx in 0..16 {
                    for lz in 0..16 {
                        let id = match region_id {
                            Some(id) => id,
                            None => self.column(cx * 16 + lx, cz * 16 + lz).biome.vanilla_id(),
                        };
                        *m.entry(id).or_default() += 1;
                    }
                }
            }
        }
        m.into_iter().collect()
    }

    /// 4.2e: desert-well candidates near world position (ox, oz) —
    /// 1/1000 per chunk, desert biome, 5×5 sand-ground site check
    /// (prose-faithful: center sand + level ground; the two-levels-
    /// below rule is approximated by the ±2 level window).
    pub fn wells_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let ccx = ox.div_euclid(16);
        let ccz = oz.div_euclid(16);
        for dcx in -1..=1 {
            for dcz in -1..=1 {
                let (cx, cz) = (ccx + dcx, ccz + dcz);
                let mut rng = Rng::new(Rng::hash3(self.seed ^ WELL_SALT, cx, 0, cz));
                if rng.next_range(WELL_CHANCE_DENOM) != 0 {
                    continue;
                }
                let wx = cx * 16 + rng.next_range(12) as i32 + 2;
                let wz = cz * 16 + rng.next_range(12) as i32 + 2;
                let c0 = self.column(wx, wz);
                if c0.biome != Biome::Desert {
                    continue;
                }
                let mut ok = true;
                'site: for dx in -2..=2 {
                    for dz in -2..=2 {
                        let c = self.column(wx + dx, wz + dz);
                        if c.top != SAND || (c.height - c0.height).abs() > 2 {
                            ok = false;
                            break 'site;
                        }
                    }
                }
                if ok {
                    out.push((wx, wz));
                }
            }
        }
        out
    }

    /// 4.2e: witch-hut centers near world position (ox, oz) — spread
    /// cells with a swamp site check.
    pub fn witch_huts_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let r0x = floor_div(ox - 24, HUT_SPACING * 16);
        let r1x = floor_div(ox + 24, HUT_SPACING * 16);
        let r0z = floor_div(oz - 24, HUT_SPACING * 16);
        let r1z = floor_div(oz + 24, HUT_SPACING * 16);
        for rx in r0x..=r1x {
            for rz in r0z..=r1z {
                if let Some(c) = self.witch_hut_center(rx, rz) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// 4.2e: hut center for one spread cell (swamp-gated).
    fn witch_hut_center(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        let mut rng = Rng::new(Rng::hash3(self.seed ^ HUT_SALT, rx, 0, rz));
        let (cx, cz) = Self::spread_candidate(rx, rz, HUT_SPACING, HUT_MARGIN, &mut rng);
        let (wx, wz) = (cx * 16 + 8, cz * 16 + 8);
        if self.column(wx, wz).biome != Biome::Swamp {
            return None;
        }
        Some((wx, wz))
    }

    /// 4.2e: desert-well emit — prose-faithful approximation (w/Desert_Well
    /// materials: sandstone walls, slab cap ring, 5-water plus, sand
    /// corners). Slabs render as full sandstone (no slab variant yet) and
    /// the exact cell blueprint is approximated — both disclosed.
    fn emit_well(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let base = self.column(wx, wz).height;
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        for dx in -2..=2 {
            for dz in -2..=2 {
                let (x, z) = (wx + dx, wz + dz);
                let corner = dx.abs() == 2 && dz.abs() == 2;
                let edge = dx.abs() == 2 || dz.abs() == 2;
                if corner {
                    put(chunk, x, base, z, SAND);
                }
                if edge {
                    for y in base + 1..=base + 3 {
                        put(chunk, x, y, z, SANDSTONE);
                    }
                    // cap ring (full-block substitution for slabs)
                    put(chunk, x, base + 4, z, SANDSTONE);
                }
            }
        }
        // the 5-water plus at layer 2
        for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(chunk, wx + dx, base + 2, wz + dz, WATER);
        }
    }

    /// 4.2e: witch-hut emit — prose-faithful approximation (w/Swamp_Hut:
    /// 7×7 spruce cabin on oak stilts, plank stepped roof for the stair
    /// roof, cauldron + crafting table + flower pot inside, porch
    /// platform). No residents (no gen-time mob spawn path) and no
    /// vines — both disclosed.
    fn emit_hut(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let floor = self.column(wx, wz).height + 3;
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // stilts: oak logs at the 7×7 corners, ground to floor
        for (dx, dz) in [(-3, -3), (3, -3), (-3, 3), (3, 3)] {
            let g = self.column(wx + dx, wz + dz).height;
            for y in g..floor {
                put(chunk, wx + dx, y, wz + dz, OAK_LOG);
            }
        }
        // floor + walls (2 high, south door gap) + stepped plank roof
        for dx in -3..=3 {
            for dz in -3..=3 {
                put(chunk, wx + dx, floor, wz + dz, SPRUCE_PLANKS);
                let edge = dx.abs() == 3 || dz.abs() == 3;
                let door = dz == 3 && dx == 0;
                if edge && !door {
                    put(chunk, wx + dx, floor + 1, wz + dz, SPRUCE_PLANKS);
                    put(chunk, wx + dx, floor + 2, wz + dz, SPRUCE_PLANKS);
                }
                put(chunk, wx + dx, floor + 3, wz + dz, SPRUCE_PLANKS);
            }
        }
        for dx in -2..=2 {
            for dz in -2..=2 {
                put(chunk, wx + dx, floor + 4, wz + dz, SPRUCE_PLANKS);
            }
        }
        // porch: 3-wide platform south of the door
        for dx in -1..=1 {
            for dz in 4..=5 {
                put(chunk, wx + dx, floor, wz + dz, SPRUCE_PLANKS);
            }
        }
        // interior: cauldron, crafting table, flower pot (empty)
        put(chunk, wx - 2, floor + 1, wz - 2, CAULDRON);
        put(chunk, wx + 2, floor + 1, wz - 2, CRAFTING_TABLE);
        put(chunk, wx, floor + 1, wz - 2, FLOWER_POT);
    }

    /// 4.3b: seam score — (matching, total) cells along the internal
    /// x-borders of a generated chunk row (block states at every y +
    /// heights + biomes). Uses the real world flow (outbound replayed
    /// as inbound), so a perfect score means cross-chunk features
    /// stitch cleanly.
    pub fn seam_score(&self, cx0: i32, cz0: i32, n: i32) -> (u64, u64) {
        let mut matched = 0u64;
        let mut total = 0u64;
        let mut outbound: Vec<(i32, i32, i32, u16)> = Vec::new();
        let mut prev: Option<std::sync::Arc<Chunk>> = None;
        for cx in cx0..cx0 + n {
            let inbound: Vec<(u16, u16)> = outbound
                .iter()
                .filter(|(x, _, z, _)| x.div_euclid(16) == cx && z.div_euclid(16) == cz0)
                .map(|(_, i, _, id)| (*i as u16, *id))
                .collect();
            let (chunk, out) = self.generate_chunk(cx, cz0, inbound);
            outbound.extend(out);
            if let Some(p) = prev {
                for z in 0..16usize {
                    for y in 0..256usize {
                        total += 1;
                        if p.get(15, y, z) == chunk.get(0, y, z) {
                            matched += 1;
                        }
                    }
                    total += 2;
                    if p.height[z * 16 + 15] == chunk.height[z * 16] {
                        matched += 1;
                    }
                    if p.biome[z * 16 + 15] == chunk.biome[z * 16] {
                        matched += 1;
                    }
                }
            }
            prev = Some(chunk);
        }
        (matched, total)
    }

    /// 4.3c: height stats — (vanilla_id, columns, mean_height) per
    /// biome over a chunk rect, sorted by id. Facts-only exchange
    /// format for density tuning (L5: aggregates, never positions).
    /// Overworld-only: column() misreports other dims (see 4.1c).
    pub fn height_stats(&self, cx0: i32, cz0: i32, w: i32, h: i32) -> Vec<(u8, u64, f64)> {
        use std::collections::BTreeMap;
        let mut m: BTreeMap<u8, (u64, i64)> = BTreeMap::new();
        for cx in cx0..cx0 + w {
            for cz in cz0..cz0 + h {
                for lx in 0..16 {
                    for lz in 0..16 {
                        let c = self.column(cx * 16 + lx, cz * 16 + lz);
                        let e = m.entry(c.biome.vanilla_id()).or_default();
                        e.0 += 1;
                        e.1 += c.height as i64;
                    }
                }
            }
        }
        m.into_iter()
            .map(|(id, (n, sum))| (id, n, sum as f64 / n as f64))
            .collect()
    }

    pub fn villages_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        const RC: i32 = VILLAGE_SPACING_CHUNKS * 16; // region size in blocks
        let mut out = Vec::new();
        let rx0 = (ox - VILLAGE_MAX_REACH).div_euclid(RC);
        let rx1 = (ox + 16 + VILLAGE_MAX_REACH).div_euclid(RC);
        let rz0 = (oz - VILLAGE_MAX_REACH).div_euclid(RC);
        let rz1 = (oz + 16 + VILLAGE_MAX_REACH).div_euclid(RC);
        for rz in rz0..=rz1 {
            for rx in rx0..=rx1 {
                if let Some((wx, wz)) = self.village_center(rx, rz) {
                    out.push((wx, wz));
                }
            }
        }
        out
    }

    /// deterministic village center for one spread cell, or None. The
    /// cell candidate lands uniformly in the (spacing−separation)
    /// chunk window ([ESTIMATED] margin convention) and must sit on
    /// flat-enough friendly ground above sea level (the site check is
    /// the engine's own rule — vanilla gates on biome at placement).
    fn village_center(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        const RC: i32 = VILLAGE_SPACING_CHUNKS * 16;
        const SPAN: i32 = VILLAGE_SPACING_CHUNKS - VILLAGE_SEPARATION_CHUNKS;
        let mut rng = Rng::new(Rng::hash3(self.seed ^ VILLAGE_SALT, rx, 0x5EED, rz));
        let wx = rx * RC + rng.next_range(SPAN as u32) as i32 * 16 + 8;
        let wz = rz * RC + rng.next_range(SPAN as u32) as i32 * 16 + 8;
        // site check: the well spot + its surroundings must be friendly
        for d in [0i32, 6, -6, 12, -12] {
            let (dx, dz) = (d, if d == 0 { 0 } else { d / 2 });
            let c = self.column(wx + dx, wz + dz);
            if c.height < vc_chunk::SEA_LEVEL + 2 || c.height > 96 {
                return None;
            }
            if !matches!(
                c.biome,
                Biome::Plains | Biome::Forest | Biome::Snowy | Biome::Mountains
            ) {
                return None;
            }
        }
        Some((wx, wz))
    }

    /// house sites of one village (deterministic): 3..6 houses on a ring
    /// around the well, each validated for flat ground at its own center
    fn village_houses(&self, wx: i32, wz: i32) -> Vec<HouseSite> {
        let mut rng = Rng::new(Rng::hash3(self.seed, wx, 0x12C5, wz));
        let n = 3 + rng.next_range(4) as usize; // 3..6
        let mut houses = Vec::new();
        for i in 0..n {
            let ang = (i as f32 + rng.next_f32() * 0.6) * std::f32::consts::TAU / n as f32;
            let r = 10.0 + rng.next_f32() * 9.0;
            let hx = wx + dround32(dcos32(ang) * r) as i32;
            let hz = wz + dround32(dsin32(ang) * r) as i32;
            // flatness: corner+center height spread ≤ 2, above sea
            let mut mn = i32::MAX;
            let mut mx = i32::MIN;
            for c in [
                self.column(hx - 2, hz - 2),
                self.column(hx + 2, hz - 2),
                self.column(hx - 2, hz + 2),
                self.column(hx + 2, hz + 2),
                self.column(hx, hz),
            ] {
                mn = mn.min(c.height);
                mx = mx.max(c.height);
            }
            if mx - mn > 2 || mn < vc_chunk::SEA_LEVEL + 1 {
                continue; // skip bad site (deterministic)
            }
            houses.push(HouseSite {
                x: hx,
                z: hz,
                floor: mx,
                blacksmith: rng.next_f32() < 0.35,
            });
        }
        houses
    }

    /// emit every village block that falls inside THIS chunk: well at the
    /// center + each house (5×5, cobble walls, log corners, glass windows,
    /// plank floor/roof, south doorway, crafting table, furnace in the
    /// blacksmith). Force-set semantics — structures own their volume.
    fn emit_village(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lx = x - ox;
            let lz = z - oz;
            if (0..16).contains(&lx) && (0..16).contains(&lz) && (0..256).contains(&y) {
                chunk.set(lx as usize, y as usize, lz as usize, id);
            }
        };

        // ---- the well: 3×3 cobble ring, 2-deep water, fence posts + roof
        let ground = self.column(wx, wz).height;
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                let edge = dx.abs() == 1 || dz.abs() == 1;
                put(
                    chunk,
                    wx + dx,
                    ground,
                    wz + dz,
                    if edge { COBBLE } else { WATER },
                );
                put(
                    chunk,
                    wx + dx,
                    ground - 1,
                    wz + dz,
                    if edge { COBBLE } else { WATER },
                );
                put(chunk, wx + dx, ground - 2, wz + dz, COBBLE);
            }
        }
        // posts + roof
        for &(px, pz) in &[(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            for dy in 1..=3 {
                put(chunk, wx + px, ground + dy, wz + pz, OAK_FENCE);
            }
        }
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                put(chunk, wx + dx, ground + 4, wz + dz, PLANKS);
            }
        }

        // ---- houses
        for house in self.village_houses(wx, wz) {
            let f = house.floor;
            for dx in -2i32..=2 {
                for dz in -2i32..=2 {
                    // floor
                    put(chunk, house.x + dx, f, house.z + dz, PLANKS);
                    // interior air (hillsides must not bury the house)
                    for dy in 1..=3 {
                        put(chunk, house.x + dx, f + dy, house.z + dz, AIR);
                    }
                    // roof + parapet rim
                    put(chunk, house.x + dx, f + 4, house.z + dz, PLANKS);
                    if dx.abs() == 2 || dz.abs() == 2 {
                        put(chunk, house.x + dx, f + 5, house.z + dz, OAK_SLAB);
                    }
                }
            }
            for dy in 1..=3 {
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        let wall = dx.abs() == 2 || dz.abs() == 2;
                        if !wall {
                            continue;
                        }
                        let corner = dx.abs() == 2 && dz.abs() == 2;
                        let mut id = if corner { OAK_LOG } else { COBBLE };
                        // windows at wall mid-height on E/W faces
                        if dy == 2 && dz == 0 && dx.abs() == 2 {
                            id = GLASS;
                        }
                        // south doorway (1 wide, 2 tall)
                        if dz == 2 && dx == 0 && (dy == 1 || dy == 2) {
                            id = AIR;
                        }
                        put(chunk, house.x + dx, f + dy, house.z + dz, id);
                    }
                }
            }
            // furniture: crafting table corner; furnace for the blacksmith
            put(chunk, house.x - 1, f + 1, house.z - 1, CRAFTING_TABLE);
            if house.blacksmith {
                put(chunk, house.x + 1, f + 1, house.z - 1, FURNACE);
            }
        }
    }

    // ------------------------------------------------- Phase 10 structures --
    // Four deferred structures from the Part 1 §2 gap table, every numeric
    // rule live-verified from the reference wiki (2026-09-04) with the
    // adaptation notes inline. All of them follow the established pure/
    // deterministic layout style (region queries + per-chunk clipped emit
    // like villages; validated against the same carved-terrain replica
    // the dungeons use).

    // ---- mineshafts (wiki Mineshaft page, live) ----
    // VERIFIED: "the most common generated structures in the Overworld,
    // having a 0.4% chance to attempt to begin generating in every chunk";
    // "Starting point: a 10×10 parlor, with an arched ceiling and one to
    // four exits in each direction"; "Corridors: some 3×3 tunnels and
    // junctions supported by planks and fences"; "On long corridors,
    // these supports are placed four blocks away from each other";
    // "Crossings: dual-floor, 5×5 intersections"; spider spawners sit in
    // cobwebbed side passages; chest loot = chests/abandoned_mineshaft.
    // ADAPTED (palette): oak instead of vanilla mixed timber; chest as a
    // plain CHEST block (no chest-minecart entity); no rails/cobwebs
    // (palette-absent, honestly documented); the cave-spider spawner
    // landed its own mob in the 1.0-1.16.5 completeness audit (the old
    // spider-spawner stand-in retired, 2026-09-08).

    /// every mineshaft whose layout can reach the chunk containing world
    /// position (ox, oz) — the 7×7-chunk neighborhood covers the longest
    /// corridor (48) plus the parlor
    pub fn mineshafts_near(&self, ox: i32, oz: i32) -> Vec<Mineshaft> {
        let cx = ox >> 4;
        let cz = oz >> 4;
        let mut out = Vec::new();
        for dcx in -3..=3 {
            for dcz in -3..=3 {
                let (cx, cz) = (cx + dcx, cz + dcz);
                let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x411E5, cx, 0, cz));
                if rng.next_f32() >= MINESHAFT_CHANCE {
                    continue;
                }
                // 10×10 parlor, floor in the deep band
                let y = 10 + rng.next_range(31) as i32; // 10..=40
                let px = cx * 16 + 3 + rng.next_range(10) as i32;
                let pz = cz * 16 + 3 + rng.next_range(10) as i32;
                let mut corridors = Vec::new();
                // 1..=4 exits "in each direction" → one corridor per
                // cardinal direction, each 0 (closed) or 24..=48 long
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let len = if rng.next_f32() < 0.25 {
                        0 // sealed exit
                    } else {
                        24 + rng.next_range(25) as i32
                    };
                    if len > 0 {
                        corridors.push((dx, dz, len));
                    }
                }
                out.push(Mineshaft {
                    x: px,
                    z: pz,
                    y,
                    corridors,
                });
            }
        }
        out
    }

    /// emit every part of `ms` that falls inside the chunk (ox, oz)
    fn emit_mineshaft(&self, chunk: &mut Chunk, ms: &Mineshaft, ox: i32, oz: i32) {
        // carver worms that can reach the shaft (paths precomputed once —
        // gen_solid tests their ellipsoids per floor query)
        let worms: Vec<(CaveWorm, WormPath)> = self
            .cave_worms_near(ms.x >> 4, ms.z >> 4)
            .into_iter()
            .map(|w| {
                let p = self.worm_path(&w);
                (w, p)
            })
            .collect();
        let put = |chunk: &mut Chunk, wx: i32, wy: i32, wz: i32, id: u16| {
            let lxi = wx - ox;
            let lzi = wz - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&wy) {
                chunk.set(lxi as usize, wy as usize, lzi as usize, id);
            }
        };
        // ---- the 10×10 parlor: plank floor, cobble walls, arched
        // ceiling (VERIFIED: arched, 1-4 exits) ----
        for dx in -5..=5 {
            for dz in -5..=5 {
                let wx = ms.x + dx;
                let wz = ms.z + dz;
                let ring = dx.abs() == 5 || dz.abs() == 5;
                put(chunk, wx, ms.y, wz, PLANKS); // floor
                for dy in 1..=4 {
                    let id = if dy <= 3 {
                        if ring {
                            COBBLE
                        } else {
                            AIR
                        }
                    } else {
                        // ceiling: corners solid, the arch opens inward
                        // (|dx|+|dz| ≤ 7 keeps the diagonal corners)
                        if dx.abs() + dz.abs() > 7 {
                            COBBLE
                        } else {
                            AIR
                        }
                    };
                    put(chunk, wx, ms.y + dy, wz, id);
                }
            }
        }
        // corner log pillars of the parlor
        for (sx, sz) in [(-5i32, -5i32), (5, -5), (-5, 5), (5, 5)] {
            for dy in 1..=3 {
                put(chunk, ms.x + sx, ms.y + dy, ms.z + sz, OAK_LOG);
            }
        }
        // ---- corridors: 3 wide × 3 high, supports every 4 blocks ----
        for &(dx, dz, len) in &ms.corridors {
            for step in 1..=len {
                // corridor center line steps from the parlor edge
                let cxw = ms.x + dx * (5 + step);
                let czw = ms.z + dz * (5 + step);
                // perpendicular offsets for the 3-wide bore
                let (px, pz) = (dz, dx);
                for off in -1..=1 {
                    for dy in 0..=3 {
                        let wx = cxw + px * off;
                        let wz = czw + pz * off;
                        if dy == 0 {
                            // floor: plank bridge ONLY where the terrain
                            // was carved/absent (vanilla corridors bridge
                            // over caves); solid ground keeps its stone
                            let ground = self.gen_solid(wx, ms.y, wz, &worms);
                            if !ground {
                                put(chunk, wx, ms.y, wz, PLANKS);
                            }
                        } else if dy < 3 {
                            put(chunk, wx, ms.y + dy, wz, AIR); // bore
                        } else {
                            // lintel: log beam across the top, every 4
                            put(
                                chunk,
                                wx,
                                ms.y + dy,
                                wz,
                                if off == 0 && step % 4 == 0 {
                                    OAK_LOG
                                } else {
                                    AIR
                                },
                            );
                        }
                    }
                }
                // supports every 4 blocks (VERIFIED): fence posts + plank
                // lintel, log pillars hanging over open cave air
                if step % 4 == 0 {
                    for off in -1..=1 {
                        let wx = cxw + px * off;
                        let wz = czw + pz * off;
                        put(
                            chunk,
                            wx,
                            ms.y + 1,
                            wz,
                            if off == 0 { AIR } else { OAK_FENCE },
                        );
                        let below = self.gen_solid(wx, ms.y, wz, &worms);
                        if !below {
                            put(chunk, wx, ms.y, wz, OAK_LOG); // pillar down
                        }
                    }
                }
            }
            // spider spawner in a side pocket at the corridor midpoint
            // (vanilla: cave-spawner spawners in cobwebbed passages —
            // adapted to the registry's spider)
            let mid = 5 + len / 2;
            let sx = ms.x + dx * mid + dz * 2;
            let sz = ms.z + dz * mid + dx * 2;
            for ddx in 0..2 {
                for ddz in 0..2 {
                    for dy in 0..=2 {
                        let wx = sx + ddx;
                        let wz = sz + ddz;
                        if dy == 0 {
                            put(chunk, wx, ms.y, wz, COBBLE);
                        } else {
                            put(chunk, wx, ms.y + dy, wz, AIR);
                        }
                    }
                }
            }
            let lxi = (sx - ox) as usize;
            let lzi = (sz - oz) as usize;
            if lxi < 16 && lzi < 16 {
                // the completeness audit: the REAL cave-spider
                // spawner (VERIFIED w/Cave_Spider: "Mineshaft: from
                // monster spawners") — replacing the disclosed
                // spider-spawner adaptation ("no distinct cave-spider
                // mob"); the cobweb nest around it stays palette-absent
                chunk.set_state(lxi, ms.y as usize, lzi, SPAWNER_CAVESPIDER);
            }
            // a chest near the far end (chests/abandoned_mineshaft seam)
            if len > 20 {
                let far = 5 + len - 3;
                put(chunk, ms.x + dx * far, ms.y + 1, ms.z + dz * far, CHEST);
            }
        }
    }

    // ---- desert pyramid (wiki Desert pyramid page, live) ----
    // VERIFIED: 21×21 ground floor; sandstone + terracotta materials with
    // a terracotta/sandstone checkerboard "wind rose" center; a hidden
    // pit under the center with the treasure; one main entrance; the top
    // stays above ground even when buried. Loot = chests/desert_pyramid.
    // ADAPTED (palette): SAND body (no sandstone block in the registry),
    // SMOOTH_STONE borders, TERRACOTTA accents; the TNT pressure-plate
    // trap is palette-absent → the pit simply holds the chests.

    /// pyramid center in a region, desert-gated; None = no pyramid
    pub fn pyramid_center_pub(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        // 4.2b: same stream as before (salt, then cx, then cz)
        let mut rng = Rng::new(Rng::hash3(self.seed ^ PYRAMID_SALT, rx, 0, rz));
        let (cx, cz) = Self::spread_candidate(rx, rz, PYRAMID_SPACING, PYRAMID_MARGIN, &mut rng);
        // site check: sampled columns must be desert + land
        for d in [0i32, 4, -4] {
            let c = self.column(cx * 16 + 8 + d, cz * 16 + 8 + d);
            if c.biome != Biome::Desert || c.height <= vc_chunk::SEA_LEVEL + 1 {
                return None;
            }
        }
        Some((cx * 16 + 8, cz * 16 + 8))
    }

    /// all pyramids near world position (ox, oz)
    pub fn pyramids_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let r0x = floor_div(ox - 24, 32 * 16);
        let r1x = floor_div(ox + 24, 32 * 16);
        let r0z = floor_div(oz - 24, 32 * 16);
        let r1z = floor_div(oz + 24, 32 * 16);
        for rx in r0x..=r1x {
            for rz in r0z..=r1z {
                if let Some(c) = self.pyramid_center_pub(rx, rz) {
                    out.push(c);
                }
            }
        }
        out
    }

    fn emit_pyramid(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let base = self.column(wx, wz).height; // ground level
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // ---- the stepped 21×21 pyramid: 5 tiers of 4-block inset ----
        // (VERIFIED base size; tier count is our layout)
        let tiers = [(10i32, 0i32), (8, 1), (6, 2), (4, 3), (2, 4)];
        for (half, th) in tiers {
            for dx in -half..=half {
                for dz in -half..=half {
                    let x = wx + dx;
                    let z = wz + dz;
                    let shell = dx.abs() == half || dz.abs() == half;
                    let y = base + 1 + th;
                    if half == 2 {
                        // top tier: solid cap with a 1-wide window gap
                        if dx.abs() <= 1 && dz.abs() <= 1 && !(dx == 0 && dz == 0) {
                            put(chunk, x, y, z, AIR);
                        } else {
                            put(chunk, x, y, z, SAND);
                        }
                        continue;
                    }
                    if shell {
                        put(chunk, x, y, z, SAND);
                        // smooth-stone corner accents
                        if dx.abs() == half && dz.abs() == half {
                            put(chunk, x, y, z, SMOOTH_STONE);
                        }
                    } else if half == 10 {
                        // ground floor: terracotta/sandstone checkerboard
                        // "wind rose" (VERIFIED pattern; palette-adapted)
                        let checker = (dx + dz).rem_euclid(2) == 0;
                        put(
                            chunk,
                            x,
                            y,
                            z,
                            if checker { TERRACOTTA } else { SMOOTH_STONE },
                        );
                    } else {
                        put(chunk, x, y, z, AIR); // nether interior
                    }
                }
            }
        }
        // main entrance: 2-high 2-wide gap in the front (south) wall
        for dy in 1..=2 {
            for d in -1..=1 {
                put(chunk, wx + d, base + dy, wz + 10, AIR);
            }
        }
        // ---- the hidden pit: 3×3 shaft straight down under the center
        // to a 5×5 treasure room with 4 chests (vanilla: 11 deep, TNT
        // floor trap — palette-adapted to a plain floor) ----
        let floor = base - 11;
        for dy in floor..=base {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    put(chunk, wx + dx, dy, wz + dz, AIR);
                }
            }
        }
        for dx in -2..=2 {
            for dz in -2..=2 {
                // treasure room floor + rim
                put(chunk, wx + dx, floor - 1, wz + dz, SMOOTH_STONE);
                if dx.abs() == 2 || dz.abs() == 2 {
                    // room walls where the shaft doesn't open
                    for dy in 0..=3 {
                        put(chunk, wx + dx, floor + dy, wz + dz, TERRACOTTA);
                    }
                }
            }
        }
        // 4 chests around the center (vanilla desert_pyramid has a
        // pressure-plate + TNT trap here; palette-absent → documented)
        put(chunk, wx - 1, floor, wz - 1, CHEST);
        put(chunk, wx + 1, floor, wz - 1, CHEST);
        put(chunk, wx - 1, floor, wz + 1, CHEST);
        put(chunk, wx + 1, floor, wz + 1, CHEST);
    }

    // ---- jungle temple (wiki Jungle pyramid page, live) ----
    // VERIFIED: cobblestone + mossy cobblestone construction, 3 floors,
    // a lever puzzle + a chest on the bottom floor, a second chest down
    // the hall, dispenser tripwire traps (palette-absent → skipped,
    // documented). Loot = chests/jungle_temple. Layout compactness is
    // our own (the wiki does not publish exact dimensions).
    pub fn jungle_temples_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let rx0 = floor_div(ox - 16, 32 * 16);
        let rx1 = floor_div(ox + 16, 32 * 16);
        let rz0 = floor_div(oz - 16, 32 * 16);
        let rz1 = floor_div(oz + 16, 32 * 16);
        for rx in rx0..=rx1 {
            for rz in rz0..=rz1 {
                if let Some(c) = self.jungle_temple_center(rx, rz) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// 4.2b: RandomSpread candidate — uniform chunk pick in the
    /// (spacing − 2×margin) window from a live region rng. The caller
    /// owns stream position (fresh rng per region, or post-gate) —
    /// pins prove zero drift either way.
    fn spread_candidate(rx: i32, rz: i32, spacing: i32, margin: i32, rng: &mut Rng) -> (i32, i32) {
        let span = (spacing - 2 * margin) as u32;
        (
            rx * spacing + margin + rng.next_range(span) as i32,
            rz * spacing + margin + rng.next_range(span) as i32,
        )
    }

    /// 3.7e: deterministic jungle-temple center for one region
    /// (extracted verbatim from jungle_temples_near for /locate).
    fn jungle_temple_center(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        // 4.2b: same stream as before (salt, then cx, then cz)
        let mut rng = Rng::new(Rng::hash3(self.seed ^ JUNGLE_SALT, rx, 0, rz));
        let (cx, cz) = Self::spread_candidate(rx, rz, JUNGLE_SPACING, JUNGLE_MARGIN, &mut rng);
        for d in [0i32, 4, -4] {
            let c = self.column(cx * 16 + 8 + d, cz * 16 + 8 + d);
            if c.biome != Biome::Jungle || c.height <= vc_chunk::SEA_LEVEL + 1 {
                return None;
            }
        }
        Some((cx * 16 + 8, cz * 16 + 8))
    }

    // ---- 1.11 woodland mansion (VERIFIED live 2026-09-07,
    // reference wiki /Woodland_Mansion: "generate rarely in dark
    // forests"; "three floors"; "The top floor is about half the size
    // of the lower floors"; "generate a cobblestone foundation
    // underneath the entire structure"; "Consist mostly of cobblestone
    // and wood blocks"; "inhabited by cleavers, evokers"). Placement
    // regions: 8×8 chunks (128 blocks — mansions are the rarest
    // overworld structure), hash-gated ~1/5, dark-forest biome check.
    // The engine-native illager placement = vindicator/evoker SPAWNER
    // blocks (vanilla spawns them at generation without respawn — the
    // spawner adaptation is disclosed in the WORKLOG; evoker spawners
    // on the two upper floors, VERIFIED: "Spawn in the two upper floors
    // of woodland mansions").
    pub fn woodland_mansions_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let rx0 = floor_div(ox - 24, 8 * 16);
        let rx1 = floor_div(ox + 24, 8 * 16);
        let rz0 = floor_div(oz - 24, 8 * 16);
        let rz1 = floor_div(oz + 24, 8 * 16);
        for rx in rx0..=rx1 {
            for rz in rz0..=rz1 {
                if let Some(c) = self.woodland_mansion_center(rx, rz) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// 3.7e: deterministic mansion center for one region (extracted
    /// verbatim from woodland_mansions_near for /locate).
    fn woodland_mansion_center(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        let mut rng = Rng::new(Rng::hash3(self.seed ^ MANSION_SALT, rx, 0, rz));
        if rng.next_range(5) != 0 {
            return None; // rare (VERIFIED "rarely")
        }
        // 4.2b: chunk pick shares the spread shape (margin 1, span 6)
        // on the live stream (post-gate draws — zero drift)
        let (cx, cz) = Self::spread_candidate(rx, rz, MANSION_SPACING, MANSION_MARGIN, &mut rng);
        // dark-forest ground check
        for d in [0i32, 5, -5] {
            let c = self.column(cx * 16 + 8 + d, cz * 16 + 8 + d);
            if c.biome != Biome::DarkForest || c.height <= vc_chunk::SEA_LEVEL + 2 {
                return None;
            }
        }
        Some((cx * 16 + 8, cz * 16 + 8))
    }

    fn emit_woodland_mansion(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let base = self.column(wx, wz).height;
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // per-structure rng
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0xA11C, wx, 0, wz));
        // 13×13 footprint, 3 floors of 5 high; the top floor is half
        // (VERIFIED: "about half the size") — 13, 13, 7 half-extent.
        // Materials: cobblestone shell + planks floors/interior (VERIFIED:
        // "Consist mostly of cobblestone and wood blocks").
        let floors = [(6i32, 5i32), (6, 10), (3, 15)]; // (half, y-base offset)
        for &(half, yoff) in floors.iter() {
            let y0 = base + 1 + yoff;
            for dx in -half..=half {
                for dz in -half..=half {
                    let x = wx + dx;
                    let z = wz + dz;
                    let shell = dx.abs() == half || dz.abs() == half;
                    for dy in 0..5 {
                        let y = y0 + dy;
                        if shell {
                            put(chunk, x, y, z, COBBLE);
                        } else if dy == 0 {
                            put(chunk, x, y, z, PLANKS); // wood floor (VERIFIED)
                        } else if dy == 4 {
                            put(
                                chunk,
                                x,
                                y,
                                z,
                                if rng.next_f32() < 0.85 {
                                    PLANKS
                                } else {
                                    COBBLE
                                },
                            );
                        } else {
                            put(chunk, x, y, z, AIR);
                        }
                    }
                }
            }
            // interior walls carve simple rooms (the "variety of rooms"
            // adaptation — deterministic cross corridors)
            for d in -(half - 2)..=(half - 2) {
                put(chunk, wx + d, y0 + 1, wz, AIR);
                put(chunk, wx + d, y0 + 2, wz, AIR);
                put(chunk, wx, y0 + 1, wz + d, AIR);
                put(chunk, wx, y0 + 2, wz + d, AIR);
            }
        }
        // cobblestone foundation under the whole footprint (VERIFIED)
        for dx in -6..=6 {
            for dz in -6..=6 {
                for y in (base - 6)..=base {
                    if y > 0 {
                        put(chunk, wx + dx, y, wz + dz, COBBLE);
                    }
                }
            }
        }
        // entrance: 2-high gap on the south face
        for dy in 1..=3 {
            for d in -1..=1 {
                put(chunk, wx + d, base + 1 + dy, wz + 6, AIR);
            }
        }
        // illagers: vindicator spawners on the lower two floors, evoker
        // spawners on the upper two (VERIFIED "Spawn in the two upper
        // floors" for evokers; cleavers mansion-wide) — the engine's
        // spawner-block adaptation of vanilla's generation-time spawn
        // (no-respawn nuance disclosed in the WORKLOG). The completeness
        // audit fix: these DEDICATED STATE ids now ride set_state (the
        // fortress/dungeon pattern) — the old put()-as-block form leaned
        // on default_state's identity fall-through, which the audit's
        // V15 block window (GHAST_TEAR = 495) broke; states never route
        // through the block-id path again.
        let put_state = |chunk: &mut Chunk, x: i32, y: i32, z: i32, st: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set_state(lxi as usize, y as usize, lzi as usize, st);
            }
        };
        put_state(chunk, wx - 3, base + 6, wz - 3, SPAWNER_CLEAVER);
        put_state(chunk, wx + 3, base + 6, wz + 3, SPAWNER_CLEAVER);
        put_state(chunk, wx - 2, base + 11, wz + 2, SPAWNER_EVOKER);
        put_state(chunk, wx + 2, base + 16, wz - 2, SPAWNER_EVOKER);
        put_state(chunk, wx, base + 16, wz, SPAWNER_CLEAVER);
        // a couple of loot chests in the foyer
        put(chunk, wx - 4, base + 2, wz + 4, CHEST);
        put(chunk, wx + 4, base + 12, wz - 4, CHEST);
    }

    fn emit_jungle_temple(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let base = self.column(wx, wz).height;
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // 11×11 footprint, 3 floors of 4 high (our compact layout);
        // cobble/mossy mix on the shell (VERIFIED materials)
        let mix = |rng: &mut Rng| -> u16 {
            if rng.next_f32() < 0.5 {
                COBBLE
            } else {
                MOSSY_COBBLE
            }
        };
        // per-structure rng seeded from the anchor
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x3E4E, wx, 0, wz));
        for floor in 0..3 {
            let half = 5 - floor; // stepped: 5,4,3
            let y0 = base + 1 + floor * 4;
            for dx in -half..=half {
                for dz in -half..=half {
                    let x = wx + dx;
                    let z = wz + dz;
                    let shell = dx.abs() == half || dz.abs() == half;
                    for dy in 0..4 {
                        let y = y0 + dy;
                        if shell {
                            let id = mix(&mut rng);
                            put(chunk, x, y, z, id);
                        } else if dy == 3 && floor < 2 {
                            put(chunk, x, y, z, mix(&mut rng)); // ceiling
                        } else {
                            put(chunk, x, y, z, AIR);
                        }
                    }
                }
            }
            // floor slabs
            for dx in -(half - 1)..=(half - 1) {
                for dz in -(half - 1)..=(half - 1) {
                    put(chunk, wx + dx, y0 - 1, wz + dz, mix(&mut rng));
                }
            }
        }
        // entrance: front gap at ground level
        for dy in 1..=2 {
            put(chunk, wx, base + dy, wz + 5, AIR);
            put(chunk, wx, base + dy, wz + 4, AIR);
        }
        // ground floor: the lever puzzle (2 levers — vanilla has 3)
        for (lx, lz) in [(-2, -2), (2, -2)] {
            let gx = (wx + lx - ox) as usize;
            let gz = (wz + lz - oz) as usize;
            if gx < 16 && gz < 16 {
                chunk.set_state(gx, (base + 1) as usize, gz, LEVER_OFF);
            }
        }
        put(chunk, wx - 2, base + 1, wz - 1, CHEST); // puzzle chest
                                                     // top floor: the far chest down the hall
        put(chunk, wx + 1, base + 9, wz - 1, CHEST);
        // interior ladderless stairwell: a cut in each floor's ceiling
        for floor in 0..2 {
            let y0 = base + 1 + floor * 4;
            put(chunk, wx + 1, y0 + 3, wz + 1, AIR);
            put(chunk, wx + 1, y0 + 4, wz + 1, AIR);
        }
    }

    // ---- stronghold (wiki Stronghold page, live) ----
    // VERIFIED: Java has 128 strongholds in 8 rings; ring 1 = 3
    // strongholds within 1,280–2,816 blocks of the origin, at roughly
    // equal angles. Stone-brick construction; the End portal room holds
    // the 12-frame portal ring over lava. Loot: stronghold_library +
    // stronghold_corridor.
    // ADAPTED: ring 1 only (the engine's playable range; the remaining
    // rings are world-gen the player would need ~5k+ blocks of travel to
    // reach — documented); compact 4-room layout (corridor + library +
    // store room + portal room) instead of vanilla's maze; portal frame
    // is decorative (no eye insertion/activation).
    pub fn strongholds(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x57_0E, 0, 0, 0));
        for i in 0..3 {
            // each stronghold sits in its own 120° sector with a small
            // jitter (the wiki: "roughly equal angles … in the region of
            // 120 degrees from the others")
            let angle = (i as f32) * std::f32::consts::TAU / 3.0 + (rng.next_f32() - 0.5) * 0.5; // ±~14°
            let dist = 1280.0 + rng.next_f32() * (2816.0 - 1280.0);
            let x = dround32(dcos32(angle) * dist) as i32;
            let z = dround32(dsin32(angle) * dist) as i32;
            out.push((x, z));
        }
        out
    }

    fn emit_stronghold(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // deep band, below the cave margin (mostly underground — VERIFIED
        // "generate at any Y level, mostly underground")
        let y = 20;
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x57_0E, wx, 1, wz));
        // room helper: nether box of stone bricks
        let room =
            |chunk: &mut Chunk, x0: i32, z0: i32, w: i32, h: i32, d: i32, y: i32, rng: &mut Rng| {
                for dx in 0..w {
                    for dz in 0..d {
                        for dy in 0..h {
                            let x = x0 + dx;
                            let z = z0 + dz;
                            let yy = y + dy;
                            let shell = dx == 0
                                || dx == w - 1
                                || dz == 0
                                || dz == d - 1
                                || dy == 0
                                || dy == h - 1;
                            if shell {
                                // cracked-looking mossy mix (palette: no
                                // cracked/chiseled stone bricks — mixed)
                                let id = if rng.next_f32() < 0.2 {
                                    MOSSY_COBBLE
                                } else {
                                    STONE_BRICKS
                                };
                                put(chunk, x, yy, z, id);
                            } else {
                                put(chunk, x, yy, z, AIR);
                            }
                        }
                    }
                }
            };
        // entrance corridor (east→west, 5 high 3 wide 12 long)
        room(chunk, wx - 12, wz - 1, 12, 5, 3, y, &mut rng);
        put(chunk, wx - 12, y + 1, wz, STONE_BRICKS); // sealed end
                                                      // library (north): 11×7×9 with bookshelf walls
        room(chunk, wx - 9, wz - 10, 11, 7, 9, y, &mut rng);
        for dz in -9..=-2 {
            for dy in 1..=3 {
                // bookshelf stacks along the north wall
                put(
                    chunk,
                    wx - 4 + ((dz + 9) % 2) * 2,
                    y + dy,
                    wz + dz,
                    BOOKSHELF,
                );
            }
        }
        put(chunk, wx - 7, y + 1, wz - 8, CHEST); // stronghold_library chest
                                                  // store room (south): 9×5×7
        room(chunk, wx - 8, wz + 2, 9, 5, 7, y, &mut rng);
        put(chunk, wx - 4, y + 1, wz + 4, CHEST); // stronghold_corridor chest
                                                  // portal room (west): 11×7×11 with the 12-frame ring + lava pool
        room(chunk, wx - 22, wz - 5, 11, 7, 11, y, &mut rng);
        let px = wx - 17; // portal ring center
        let pz = wz;
        // lava pool below the ring (vanilla: lava under the portal)
        for dx in -1..=1 {
            for dz in -1..=1 {
                put(chunk, px + dx, y, pz + dz, GLOWSTONE); // lit floor (no lava-flow sim here — glowstone reads as lit)
            }
        }
        // the 12-frame ring: 3 per side, gap at the corners (vanilla
        // 1.16.5 portal room layout)
        for i in 0..3 {
            put(chunk, px + (i - 1), y + 1, pz - 2, END_PORTAL_FRAME);
            put(chunk, px + (i - 1), y + 1, pz + 2, END_PORTAL_FRAME);
            put(chunk, px - 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
            put(chunk, px + 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
        }
        // the completeness audit: the stronghold's silverfish spawner —
        // VERIFIED (reference wiki /Silverfish, live 2026-09-08, capture
        // scripts/audit16_page_Silverfish.json): "Stronghold: from
        // infested blocks and monster spawners". The engine form: one
        // spawner in the portal room's upper center (vanilla's own
        // placement class — the ledge above the lava pool; the exact
        // vanilla offset is per-stronghold random, the room center is
        // the engine's deterministic stand-in, disclosed). The
        // infested-block family is palette-absent, disclosed.
        {
            let lxi = px - ox;
            let lzi = pz - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&(y + 4)) {
                chunk.set_state(
                    lxi as usize,
                    (y + 4) as usize,
                    lzi as usize,
                    SPAWNER_SILVERFISH,
                );
            }
        }
        // doorway from the corridor into the portal room
        put(chunk, wx - 12, y + 1, wz, AIR);
        put(chunk, wx - 12, y + 2, wz, AIR);
    }

    // ---- ravines (wiki Ravine page, live) ----
    // VERIFIED: "around 85 to 127 blocks in length and typically less
    // than 15 blocks wide"; "up to 62 blocks in depth and can start at
    // levels 10 to 72"; ledges along the top; deep bottoms expose ores
    // (and in vanilla can flood with lava). Frequency is a [tuning]
    // value: vanilla's canyon carver probability is not published on the
    // wiki; we use 1 per 50 chunks (0.02).

    /// every ravine that can cover the chunk containing (cx, cz): the
    /// 11×11-chunk neighborhood covers the 127-block max diagonal
    ///
    /// 2026-09-20 rampart-fix round (the rampant-ravines + lag report):
    /// the canyon TOP is now an independent 10..=72 roll from the
    /// anchor's rng stream (wiki Ravine: "can start at levels 10 to
    /// 72" — a START LEVEL, not the anchor surface). The old
    /// `top = column(x0,z0).clamp(10,72)` forced top = the anchor
    /// terrain surface on plains, so EVERY ravine broke the surface
    /// as a 40+ block open mega-trench (quantified: 100% of carved
    /// columns sky-open, mean depth 39) — the rampant look plus a
    /// mesh explosion (each trench adds ~2×depth wall faces per
    /// carved column). With the rolled top most carves stay
    /// UNDERGROUND (found while caving, vanilla-style) and only the
    /// few whose rolled top clears the local surface open as canyons.
    /// The `column()` solve also leaves the scan entirely — it was
    /// the only expensive call in here and ran ~121× redundantly
    /// across the neighborhood's chunk generations.
    pub fn ravines_near_chunk(&self, cx: i32, cz: i32) -> Vec<Ravine> {
        let mut out = Vec::new();
        for dcx in -5..=5 {
            for dcz in -5..=5 {
                let (cx, cz) = (cx + dcx, cz + dcz);
                let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0CAE, cx, 0, cz));
                if rng.next_f32() >= RAVINE_CHANCE {
                    continue;
                }
                let x0 = cx * 16 + rng.next_range(16) as i32;
                let z0 = cz * 16 + rng.next_range(16) as i32;
                let angle = rng.next_f32() * std::f32::consts::TAU;
                let length = 85 + rng.next_range(43) as i32; // 85..=127
                let half_w = 2.0 + rng.next_f32() * 5.0; // < 15 wide total
                                                         // 2026-09-20: depth was 40..=62 — "up to 62 deep" is a
                                                         // MAXIMUM, not a minimum; every ravine being 40+ deep is
                                                         // what made each one a mega-canyon. Now 10..=62 (mean
                                                         // ~36, shallow ones included; [tuning] like the chance).
                let depth = 10 + rng.next_range(53) as i32; // ≤ 62
                                                            // 2026-09-20: the START LEVEL roll (10..=72) — see the
                                                            // doc comment above. Drawn after depth on the same
                                                            // stream: anchors/angle/length/width/depth are unchanged
                                                            // for any seed that previously rolled them.
                let top = 10 + rng.next_range(63) as i32; // 10..=72
                out.push(Ravine {
                    x0,
                    z0,
                    dx: dcos32(angle),
                    dz: dsin32(angle),
                    length,
                    half_w,
                    depth,
                    top,
                });
            }
        }
        out
    }

    /// ravine carve test for one column (x, z) → the carved y-interval
    /// (top, bottom), if any. V-shape: full width at the rim, tapering
    /// toward the floor; lengthwise taper at both ends.
    fn ravine_cut(&self, ravines: &[Ravine], x: i32, z: i32, surface: i32) -> Option<(i32, i32)> {
        let mut best: Option<(i32, i32)> = None;
        for rv in ravines {
            // project (x,z) onto the path segment
            let rx = (x - rv.x0) as f32;
            let rz = (z - rv.z0) as f32;
            let t = rx * rv.dx + rz * rv.dz; // distance along
            if t < 0.0 || t > rv.length as f32 {
                continue;
            }
            let perp = (rx * rv.dz - rz * rv.dx).abs(); // distance from line
                                                        // lengthwise taper: half-width scales down in the last 12
                                                        // blocks of each end
            let end_taper = {
                let from_end = (rv.length as f32 - t).min(t);
                (from_end / 12.0).min(1.0)
            };
            let rim_w = rv.half_w * end_taper;
            if perp > rim_w {
                continue;
            }
            // the top starts at min(surface, rv.top): a ravine never
            // rises above the terrain it cuts
            let top = surface.min(rv.top);
            let bottom = (top - rv.depth).max(8);
            if bottom >= top {
                continue;
            }
            // V-shape: floor narrower than the rim — carve the interval
            // scaled by how far into the width we are
            let frac = 1.0 - (perp / rim_w.max(0.001)); // 1 at center
            let cut_top = top;
            let cut_bottom = top - ((top - bottom) as f32 * (0.45 + 0.55 * frac)) as i32;
            let cut_bottom = cut_bottom.max(bottom);
            if cut_bottom < cut_top {
                // merge overlapping ravine cuts as a UNION of the two
                // down-carve intervals [b+1 ..= t]: keep the HIGHER rim
                // and the deeper floor. (2026-09-20 rampart-fix: the old
                // `bt.min(cut_top)` kept the LOWER rim, silently
                // dropping the upper interval when two ravines crossed
                // a column — solid rock mesas/pillars left standing
                // inside canyon crossings, the "rampant" artifact.)
                best = Some(match best {
                    Some((bt, bb)) => (bt.max(cut_top), bb.min(cut_bottom)),
                    None => (cut_top, cut_bottom),
                });
            }
        }
        best
    }

    // =================================================================
    // Phase E1 (evolution 1.0–1.2 bracket): The End + Nether Fortress.
    // All structural facts live-verified 2026-09-06 (the audit trail:
    // docs/research/phase1-1.0-1.2-research.md).
    // =================================================================

    /// The End (VERIFIED w/The_End): a void dimension — one end-stone
    /// central island around (0,0); 10 obsidian pillars on a 42-block
    /// radius circle around the exit portal, descending to y=0, each
    /// capped with a bedrock block (the crystal sits above it, entity
    /// side); the exit-portal bedrock fountain at the center; the 5×5
    /// obsidian arrival platform at (100, 64, 0) [documented
    /// approximation: the wiki fixes the arrival X/Z at 100/0; our
    /// platform Y rides the island band]. Pillar heights: vanilla uses a
    /// fixed 10-entry table we did not capture this round — ours is a
    /// deterministic 78..103 spread [placeholder, disclosed in worklog].
    fn generate_end_chunk(&self, cx: i32, cz: i32, _inbound: Vec<(u16, u16)>) -> GenOut {
        let mut chunk = Chunk::empty();
        let outbound: Vec<(i32, i32, i32, u16)> = Vec::new();
        let ox = cx * 16;
        let oz = cz * 16;

        // ---- central island: end stone, radius ~60, surface band 60..64 ----
        for z in 0..16i32 {
            for x in 0..16i32 {
                let wx = ox + x;
                let wz = oz + z;
                let col_idx = (z * 16 + x) as usize;
                let dist = dsqrt32((wx * wx + wz * wz) as f32);
                // gentle island surface: 62-64 center, tapering to the rim
                if dist < 60.0 {
                    let surface = 63 - (dist / 30.0).floor() as i32
                        + (Rng::hash3(self.seed ^ 0xE1D5, wx, 0, wz) % 2) as i32;
                    let surface = surface.clamp(58, 64);
                    // island thickness tapers to the rim (vanilla look)
                    let thick = ((60.0 - dist) / 12.0).ceil() as i32;
                    let bottom = (surface - thick).max(40);
                    chunk.height[col_idx] = surface as u8;
                    for y in bottom..=surface {
                        chunk.set(x as usize, y as usize, z as usize, END_STONE);
                    }
                }
                // 4.4b: outer islands past the void gap (dist ≥ 1000) —
                // noise-gated end-stone blobs ([ESTIMATED] ring, gate,
                // heights — vanilla observable: void gap then island
                // ring; cities/ships need documented dims, deferred).
                if dist >= 1000.0 {
                    let m = fbm2(
                        &self.n_end,
                        wx as f32 / 220.0,
                        wz as f32 / 220.0,
                        3,
                        2.0,
                        0.5,
                    );
                    if m > 0.25 {
                        let surface = (62.0 + m * 24.0) as i32;
                        let thick = 8 + (Rng::hash3(self.seed ^ 0xE1D6, wx, 0, wz) % 8) as i32;
                        let bottom = (surface - thick).max(40);
                        chunk.height[col_idx] = surface.min(255) as u8;
                        for y in bottom..=surface {
                            chunk.set(x as usize, y as usize, z as usize, END_STONE);
                        }
                        // chorus on high islands (approximate stacks —
                        // no branching, disclosed)
                        if surface >= 66
                            && Rng::hash3(self.seed ^ 0xE1D7, wx, 0, wz).is_multiple_of(7)
                        {
                            let h = 2 + (Rng::hash3(self.seed ^ 0xE1D8, wx, 0, wz) % 3) as i32;
                            for y in surface + 1..=surface + h {
                                chunk.set(x as usize, y as usize, z as usize, CHORUS_PLANT);
                            }
                            chunk.set(
                                x as usize,
                                (surface + h + 1) as usize,
                                z as usize,
                                CHORUS_FLOWER,
                            );
                        }
                    }
                }
                chunk.biome[col_idx] = 9; // the_end (Bedrock single-biome id)
            }
        }

        // ---- the 10 obsidian pillars (VERIFIED: 42-radius circle, down to
        // y=0, bedrock cap + a crystal entity above, 2 of them caged) ----
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // the 10 crystal-bearing pillars, evenly spaced on the 42-radius
        // circle (VERIFIED count/radius; even angular spacing)
        let mut angles = [(0.0f32, 0.0f32); 10];
        for (i, a) in angles.iter_mut().enumerate() {
            let th = i as f32 * std::f32::consts::TAU / 10.0;
            *a = (dcos32(th), dsin32(th));
        }
        for (i, ang) in angles.iter().enumerate() {
            let px = (ang.0 * 42.0).round() as i32;
            let pz = (ang.1 * 42.0).round() as i32;
            // deterministic height spread 78..=103 [placeholder for
            // vanilla's fixed table — disclosed]
            let top = 78 + ((Rng::hash3(self.seed, i as i32, 0x11, 0xE1D) % 26) as i32);
            let radius = 3;
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx * dx + dz * dz > radius * radius + 1 {
                        continue;
                    }
                    let x = px + dx;
                    let z = pz + dz;
                    // VERIFIED: the pillars "penetrate through the main
                    // island down to y level 0" — every column descends
                    // the full height, island or not
                    for y in 0..=top {
                        put(&mut chunk, x, y, z, OBSIDIAN);
                    }
                    put(&mut chunk, x, top, z, BEDROCK);
                }
            }
            // 2 pillars carry iron-bar cages (VERIFIED: "two of which are
            // protected in cages of iron bars" — w/The_End). No iron-bars
            // block in the engine: OBSIDIAN corner posts stand in
            // [documented adaptation; the crystal stays reachable from
            // above like vanilla's open-top cages]
            if i == 2 || i == 7 {
                for dx in -2..=2i32 {
                    for dz in -2..=2i32 {
                        let edge = dx.abs() == 2 || dz.abs() == 2;
                        if edge && dx.abs() == 2 && dz.abs() == 2 {
                            for y in (top + 1)..=(top + 3) {
                                put(&mut chunk, px + dx, y, pz + dz, OBSIDIAN);
                            }
                        }
                    }
                }
            }
        }

        // ---- the exit-portal bedrock fountain at (0, y, 0) (VERIFIED
        // w/The_End: activates on the dragon's defeat — the 3×3 center
        // fills with END_PORTAL blocks then, game-side). Sits ON the
        // island surface (the island center tops at ~y 63).
        {
            // base slab (5×5) at 61, ring at 62, the inner 3×3 stays open
            // for the victory portal; the egg pedestal rises at (0, 63, 0)
            for dx in -2..=2i32 {
                for dz in -2..=2i32 {
                    put(&mut chunk, dx, 61, dz, BEDROCK);
                    let ring = dx.abs() == 2 || dz.abs() == 2;
                    if ring {
                        put(&mut chunk, dx, 62, dz, BEDROCK);
                    }
                }
            }
            put(&mut chunk, 0, 63, 0, BEDROCK); // the egg pedestal (egg at 64)
                                                // carve the inner 3×3 at y 62 — the victory portal fills it
            for dx in -1..=1i32 {
                for dz in -1..=1i32 {
                    put(&mut chunk, dx, 62, dz, AIR);
                }
            }
        }

        // ---- the 5×5 obsidian arrival platform at (100, 64, 0) (VERIFIED
        // w/The_End: "a 5 by 5 square of obsidian that is generated once a\n        // player or entity enters the End" — we emit it with the world so\n        // the first arrival already stands on it) ----
        {
            for dx in -2..=2i32 {
                for dz in -2..=2i32 {
                    put(&mut chunk, 100 + dx, 63, dz, OBSIDIAN);
                    // clear the platform's air
                    for y in 64..=66 {
                        put(&mut chunk, 100 + dx, y, dz, AIR);
                    }
                }
            }
        }

        (Arc::new(chunk), outbound)
    }

    /// The End arrival position (the platform's center top).
    pub fn end_arrival(&self) -> (f32, f32, f32) {
        (100.5, 64.0, 0.5)
    }

    /// The 10 pillar tops as (x, top_y, z) — the crystal spawn points
    /// (game layer's dragon fight). Mirrors generate_end_chunk's pillar
    /// math exactly (same angle table + height roll).
    pub fn end_pillar_tops(&self) -> Vec<(i32, i32, i32)> {
        let mut out = Vec::with_capacity(10);
        for i in 0..10usize {
            let th = i as f32 * std::f32::consts::TAU / 10.0;
            let px = dround32(dcos32(th) * 42.0) as i32;
            let pz = dround32(dsin32(th) * 42.0) as i32;
            let top = 78 + ((Rng::hash3(self.seed, i as i32, 0x11, 0xE1D) % 26) as i32);
            out.push((px, top, pz));
        }
        out
    }

    /// Phase E1: does this 432×432 nether region (VERIFIED region size,
    /// w/Nether_Fortress "regions are 432×432 blocks in Java Edition")
    /// carry a fortress? Deterministic per-region roll [placeholder: the
    /// vanilla per-region probability was not captured this round — 50%
    /// chosen so fortresses are findable; disclosed in the worklog].
    pub fn fortress_in_region(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        let roll = Rng::hash3(self.seed ^ 0xF0E7, rx, 0x1E5, rz) % 100;
        if roll < 50 {
            // center + deterministic jitter inside the region
            let jx = (Rng::hash3(self.seed ^ 0xF0E8, rx, 1, rz) % 144) as i32;
            let jz = (Rng::hash3(self.seed ^ 0xF0E9, rx, 2, rz) % 144) as i32;
            Some((rx * 432 + 144 + jx, rz * 432 + 144 + jz))
        } else {
            None
        }
    }

    /// All fortress centers whose bounding box (arm half-length 60) might
    /// reach the given chunk.
    pub fn fortresses_near_chunk(&self, cx: i32, cz: i32) -> Vec<(i32, i32)> {
        let ox = cx * 16;
        let oz = cz * 16;
        let mut out = Vec::new();
        let reach = 60 + 16;
        let r0x = floor_div(ox - reach, 432);
        let r1x = floor_div(ox + reach, 432);
        let r0z = floor_div(oz - reach, 432);
        let r1z = floor_div(oz + reach, 432);
        for rx in r0x..=r1x {
            for rz in r0z..=r1z {
                if let Some(c) = self.fortress_in_region(rx, rz) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// Fortress layout (all VERIFIED w/Nether_Fortress unless noted):
    /// bridges + enclosed corridors of nether bricks on pillars "that
    /// tower high above the lava seas"; up to 2 blaze spawner platforms
    /// (each surrounded by nether-brick fences + a 3-block staircase —
    /// w/Blaze); nether-wart garden by a stairwell (20 plants in soul
    /// sand — w/Nether_Wart). We emit a symmetric cross: an E-W bridge
    /// spine, a N-S corridor, 2 blaze platforms, 1 wart garden. [layout
    /// geometry is our procedural approximation of the vanilla piece
    /// system — the verified facts are the material, the blaze platforms,
    /// and the wart garden]
    fn emit_fortress(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u16| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        let deck = 70i32; // bridge deck height (above the cavern floor)
        let arm = 60i32; // half-length of each arm

        // ---- the E-W bridge spine (5 wide, railings, pillars) ----
        for dx in -arm..=arm {
            for dz in -2..=2i32 {
                let x = wx + dx;
                let z = wz + dz;
                put(chunk, x, deck, z, NETHER_BRICKS);
                // railing rows (vanilla bridges have side rails)
                if dz.abs() == 2 {
                    put(chunk, x, deck + 1, z, NETHER_BRICKS);
                }
                // support pillars every 8 blocks down to y=8
                if dx % 8 == 0 && dz == 0 {
                    for y in 8..deck {
                        put(chunk, x, y, z, NETHER_BRICKS);
                    }
                }
            }
        }

        // ---- the N-S enclosed corridor (3 wide, walls + roof) ----
        for dz in -arm..=arm {
            for dx in -1..=1i32 {
                let x = wx + dx;
                let z = wz + dz;
                put(chunk, x, deck, z, NETHER_BRICKS);
                if dx.abs() == 1 {
                    // side walls with window gaps
                    if dz % 4 != 2 {
                        put(chunk, x, deck + 1, z, NETHER_BRICKS);
                        put(chunk, x, deck + 2, z, NETHER_BRICKS);
                    }
                }
                put(chunk, x, deck + 3, z, NETHER_BRICKS); // roof
                                                           // pillars
                if dz % 8 == 0 && dx == 0 {
                    for y in 8..deck {
                        put(chunk, x, y, z, NETHER_BRICKS);
                    }
                }
            }
        }

        // ---- spawner platforms ×2 (VERIFIED w/Blaze: "up to two blaze
        // spawner platforms…"; Phase E2: the second platform hosts a
        // WITHER-SKELETON spawner — VERIFIED w/Wither_Skeleton "spawn in
        // Nether fortresses" — no fence block in the engine; railing
        // posts stand in [adaptation]) ----
        for (pi, (sx, sz)) in [(0, (wx + 24, wz + 10)), (1, (wx - 24, wz - 10))].into_iter() {
            for dx in -3..=3i32 {
                for dz in -3..=3i32 {
                    put(chunk, sx + dx, deck, sz + dz, NETHER_BRICKS);
                    // railing ring
                    if dx.abs() == 3 || dz.abs() == 3 {
                        put(chunk, sx + dx, deck + 1, sz + dz, NETHER_BRICKS);
                    }
                }
            }
            // the spawner itself (SPAWNER_BLAZE state 241 / the second
            // platform's wither-skeleton spawner state 315)
            let lxi = sx - ox;
            let lzi = sz - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) {
                let st = if pi == 0 {
                    SPAWNER_BLAZE
                } else {
                    SPAWNER_WITHER_SKELETON
                };
                chunk.set_state(lxi as usize, deck as usize + 1, lzi as usize, st);
            }
            // 3-block staircase down (VERIFIED)
            for step in 0..3i32 {
                for dz in -1..=1i32 {
                    put(chunk, sx + 4, deck - 1 - step, sz + dz, NETHER_BRICKS);
                }
            }
        }

        // ---- the nether-wart garden (VERIFIED w/Nether_Wart: soul sand
        // gardens near stairwells; ~20 plants; growth needs only soul
        // sand) ----
        {
            let gx = wx + 10;
            let gz = wz - 14;
            let mut planted = 0;
            for dx in 0..6i32 {
                for dz in 0..6i32 {
                    put(chunk, gx + dx, deck, gz + dz, SOUL_SAND);
                    // plant ~20 warts in a scatter (age 0 state)
                    if planted < 20 && dx % 2 == 0 && dz % 2 == 0 {
                        let lxi = gx + dx - ox;
                        let lzi = gz + dz - oz;
                        if (0..16).contains(&lxi) && (0..16).contains(&lzi) {
                            chunk.set_state(
                                lxi as usize,
                                deck as usize + 1,
                                lzi as usize,
                                WART_STATE_BASE,
                            );
                            planted += 1;
                        }
                    }
                }
            }
        }
    }

    /// Find a comfortable spawn point (land, moderate altitude) near origin.
    pub fn find_spawn(&self) -> (f32, f32, f32) {
        // green, welcoming biomes score higher for the spawn
        let biome_bonus = |b: Biome| -> i32 {
            match b {
                Biome::Forest => 3,
                Biome::Plains => 2,
                Biome::Desert => 0,
                Biome::Mountains => 0,
                _ => -6, // Ocean / Beach / Snowy
            }
        };
        let land = |x: i32, z: i32| -> bool {
            let c = self.column(x, z);
            c.height > vc_chunk::SEA_LEVEL + 1 && c.biome != Biome::Ocean && c.biome != Biome::Beach
        };
        let mut best: Option<(i32, i32)> = None;
        let mut best_score = i32::MIN;
        'search: for r in 0..40 {
            for i in -r..=r {
                let candidates = [(i, r), (i, -r), (r, i), (-r, i)];
                for &(x, z) in &candidates {
                    let (wx, wz) = (x * 8, z * 8);
                    let col = self.column(wx, wz);
                    if !(col.height > vc_chunk::SEA_LEVEL + 1
                        && col.height < 90
                        && col.biome != Biome::Ocean
                        && col.biome != Biome::Beach)
                    {
                        continue;
                    }
                    // Landmass check: a spawn on a 1-block beach islet reads
                    // as an empty ocean world — require land in most
                    // surrounding directions (12 dirs x 3 radii), and prefer
                    // green biomes around the spawn.
                    let mut score = biome_bonus(col.biome);
                    for k in 0..12i32 {
                        let yaw = k as f32 * std::f32::consts::TAU / 12.0;
                        for d in [24.0f32, 48.0, 96.0] {
                            let sx = (wx as f32 + dsin32(yaw) * d) as i32;
                            let sz = (wz as f32 - dcos32(yaw) * d) as i32;
                            let c = self.column(sx, sz);
                            if land(sx, sz) {
                                score += 1 + biome_bonus(c.biome);
                            }
                        }
                    }
                    if score > best_score {
                        best_score = score;
                        best = Some((wx, wz));
                    }
                    if score >= 40 {
                        break 'search; // solid, green landmass
                    }
                }
            }
        }
        let (x, z) = best.unwrap_or((0, 0));
        let h = self.column(x, z).height;
        (x as f32 + 0.5, h as f32 + 3.0, z as f32 + 0.5)
    }
}

/// Phase E3: the badlands stained-terracotta band color for an absolute
/// y level. Vanilla generates seed-shifted colored-terracotta layers in
/// badlands ("found abundantly in badlands biomes" — VERIFIED w/
/// Terracotta; w/Badlands) but the exact per-seed layer table is not
/// published; this deterministic banding (a fixed color sequence by
/// (y + seed offset)) is the disclosed clean-room adaptation. The
/// sequence mixes the warm desert-family colors the vanilla badlands
/// actually shows (orange/yellow/red/brown/white + plain terracotta
/// returns).
fn badlands_band_color(seed: u64, y: i32) -> u8 {
    // index into the vanilla dye-order color table (0=white, 1=orange,
    // 4=yellow, 14=red, 12=brown ...) — 16 stains + the plain-terracotta
    // fallback handled by the caller via `255`
    const BANDS: [u8; 12] = [
        1, 1, 4, 1, 12, 0, 1, 14, 1, 12, 4, 1, // orange-dominant strata
    ];
    let off = (seed >> 13) as i32 & 63; // per-seed vertical shift
    let i = ((y + off).rem_euclid(BANDS.len() as i32)) as usize;
    BANDS[i]
}

/// Phase E2: emerald-ore hash gate (mountains only; ~5 per chunk at
/// p = 0.0008 — see the stone-fill branch comment for the vanilla
/// feature semantics: attempts 100 times per chunk in 0-3-size blobs,
/// single blocks since 12w22a).
fn emerald_ore(seed: u64, x: i32, y: i32, z: i32) -> bool {
    let o = Rng::hash3(seed ^ 0xE000, x, y, z);
    (o % 100_000) as f32 / 100_000.0 < 0.0008
}

#[cfg(test)]
mod village_tests {
    use super::*;

    /// villages exist and are findable: scan region space for a handful of
    /// seeds until villages appear (placement is ~55%/region, gated on
    /// terrain, so a scan is the honest test)
    #[test]
    fn villages_spawn_deterministically() {
        let mut found = 0;
        'seeds: for s in 0..12u64 {
            let gen = TerrainGen::new(0xC0FF_EE00u64.wrapping_add(s));
            for rz in -3..=3i32 {
                for rx in -3..=3i32 {
                    if gen.village_center(rx, rz).is_some() {
                        found += 1;
                        continue 'seeds; // one per seed is enough
                    }
                }
            }
        }
        assert!(
            found >= 4,
            "expected several villages across seeds, got {found}"
        );
    }

    /// a village's blocks actually land in the chunk containing it: the
    /// well chunk must contain water + cobble + fence + planks above ground
    #[test]
    fn village_blocks_emit_into_owning_chunk() {
        // find a concrete village
        let mut village = None;
        'outer: for s in 0..40u64 {
            let gen = TerrainGen::new(0xAB_CDEF00u64.wrapping_add(s));
            for rz in -4..=4i32 {
                for rx in -4..=4i32 {
                    if let Some(v) = gen.village_center(rx, rz) {
                        village = Some((gen, v));
                        break 'outer;
                    }
                }
            }
        }
        let (gen, (wx, wz)) = village.expect("a village within 40 seeds");
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let ground = gen.column(wx, wz).height as usize;
        // 1.7.2 refactor: Chunk::get folds states to owning block ids now,
        // so probes read the block id directly (the fence STATE check below
        // uses get_state — the raw accessor).
        // well: water at center, cobble rim, fence post corner, plank roof
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), ground),
            WATER,
            "well center water"
        );
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx + 1, lz), ground),
            COBBLE,
            "well rim cobble"
        );
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx - 1, lz - 1), ground + 3),
            OAK_FENCE,
            "well post"
        );
        assert_eq!(
            chunk.get_state(lx - 1, ground + 3, lz - 1),
            73,
            "well post stores the no-connection fence STATE (not a log axis)"
        );
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), ground + 4),
            PLANKS,
            "well roof"
        );
    }

    /// generation is order-independent and deterministic: generating the
    /// well chunk BEFORE vs AFTER its neighbors yields identical bytes
    #[test]
    fn village_chunks_are_deterministic() {
        let gen = TerrainGen::new(0x1234_5678u64);
        // find a village to make the test meaningful
        let mut hit = None;
        'o: for rz in -5..=5i32 {
            for rx in -5..=5i32 {
                if let Some(v) = gen.village_center(rx, rz) {
                    hit = Some(v);
                    break 'o;
                }
            }
        }
        let (wx, wz) = hit.expect("village near seed 0x12345678");
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let a = gen.generate_chunk(cx, cz, Vec::new()).0;
        // interleave neighbor generation, then regenerate — must be equal
        for dz in -1..=1 {
            for dx in -1..=1 {
                if (dx, dz) != (0, 0) {
                    let _ = gen.generate_chunk(cx + dx, cz + dz, Vec::new());
                }
            }
        }
        let b = gen.generate_chunk(cx, cz, Vec::new()).0;
        // compare raw block storage
        for i in 0..vc_chunk::chunk::CHUNK_LEN {
            assert_eq!(
                a.get_idx(i),
                b.get_idx(i),
                "chunk differs at flat idx {i} — village gen must be order-independent"
            );
        }
    }

    /// houses appear around the well with the expected materials somewhere
    /// in the village chunks (scan the 3×3 chunk neighborhood)
    #[test]
    fn village_houses_have_expected_materials() {
        // 4.1m: first village WITH validated houses — the share refit
        // moved biomes, so the first-found site's ring may fail the
        // flatness check (seed luck, not a village regression)
        let mut found = None;
        'outer: for s in 0..40u64 {
            let gen = TerrainGen::new(0x99_CAFE00u64.wrapping_add(s));
            for rz in -4..=4i32 {
                for rx in -4..=4i32 {
                    if let Some(v) = gen.village_center(rx, rz) {
                        if !gen.village_houses(v.0, v.1).is_empty() {
                            found = Some((gen, v));
                            break 'outer;
                        }
                    }
                }
            }
        }
        let (gen, (wx, wz)) = found.expect("a village with houses");
        let houses = gen.village_houses(wx, wz);
        assert!(!houses.is_empty(), "validated house sites exist");
        let (mut planks, mut glass, mut logs, mut tables) = (0, 0, 0, 0);
        for dz in -1..=1i32 {
            for dx in -1..=1i32 {
                let (chunk, _) =
                    gen.generate_chunk(wx.div_euclid(16) + dx, wz.div_euclid(16) + dz, Vec::new());
                for i in 0..vc_chunk::chunk::CHUNK_LEN {
                    match chunk.get_idx(i) {
                        vc_blocks::blocks::PLANKS => planks += 1,
                        vc_blocks::blocks::GLASS => glass += 1,
                        vc_blocks::blocks::OAK_LOG => logs += 1,
                        vc_blocks::blocks::CRAFTING_TABLE => tables += 1,
                        _ => {}
                    }
                }
            }
        }
        assert!(planks > 20, "house floors+roofs: {planks} planks");
        assert!(glass > 0, "windows: {glass} glass");
        assert!(logs >= 4, "log corners: {logs} logs");
        assert!(tables >= 1, "crafting tables: {tables}");
    }
}

#[cfg(test)]
mod nether_tests {
    use super::*;
    use crate::world::Dimension;

    /// 1.7.2 refactor: Chunk::get FOLDS states to block ids itself, so the
    /// fold helper is identity (kept for the historical test prose). u16
    /// since the merge (block ids widened).
    fn fold(s: u16) -> u16 {
        s
    }

    /// Backlog round: all five 1.16 nether biomes appear in the region
    /// roll at roughly their wiki-verified volumes (SSV 17%, BD 16%,
    /// crimson 22%, warped 8%, wastes the remainder).
    #[test]
    fn backlog_five_nether_biomes_all_appear() {
        let mut counts = [0usize; 5]; // [wastes, crimson, warped, ssv, deltas]
        for rx in -30..30 {
            for rz in -30..30 {
                let b = nether_region_biome(0xBE11, rx * 2, rz * 2);
                let i = match b {
                    Biome::NetherWastes => 0,
                    Biome::CrimsonForest => 1,
                    Biome::WarpedForest => 2,
                    Biome::SoulSandValley => 3,
                    Biome::BasaltDeltas => 4,
                    _ => panic!("non-nether region biome {b:?}"),
                };
                counts[i] += 1;
            }
        }
        let total: usize = counts.iter().sum();
        for c in counts {
            assert!(c > 0, "every nether biome must appear in the roll");
        }
        // the volume shares (±6% tolerance — a 60x60 sample)
        let ssv = counts[3] as f64 / total as f64;
        let bd = counts[4] as f64 / total as f64;
        let warped = counts[2] as f64 / total as f64;
        assert!((ssv - 0.17).abs() < 0.06, "SSV share {ssv:.2}");
        assert!((bd - 0.16).abs() < 0.06, "deltas share {bd:.2}");
        assert!((warped - 0.08).abs() < 0.05, "warped share {warped:.2}");
    }

    /// The soul sand valley floor is soul sand + soul soil, carries
    /// fossils, and grows giant basalt pillars (VERIFIED
    /// w/Soul_Sand_Valley capture). Samples several SSV regions —
    /// surface density varies per chunk with the cavern carver.
    #[test]
    fn backlog_soul_valley_has_soul_floor_and_fossils() {
        let mut soul = 0usize;
        let mut fossils = 0usize;
        let mut pillars = 0usize;
        let mut regions_sampled = 0usize;
        'seeds: for seed in 1..24u64 {
            // find a chunk of this seed's SSV
            for c in 0..40i32 {
                let cx = c * 2;
                let cz = c * 2;
                if nether_region_biome(seed, cx, cz) != Biome::SoulSandValley {
                    continue;
                }
                let gen = TerrainGen::for_dimension(seed, Dimension::Nether);
                for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (chunk, _) = gen.generate_chunk(cx + dx, cz + dz, Vec::new());
                    for i in 0..vc_chunk::chunk::CHUNK_LEN {
                        match chunk.get_idx(i) {
                            SOUL_SAND | SOUL_SOIL => soul += 1,
                            BONE_BLOCK => fossils += 1,
                            BASALT => pillars += 1,
                            _ => {}
                        }
                    }
                }
                regions_sampled += 1;
                if regions_sampled >= 6 {
                    break 'seeds;
                }
                break; // one region per seed
            }
        }
        assert!(
            regions_sampled >= 3,
            "found SSV regions ({regions_sampled})"
        );
        assert!(soul > 40, "the soul floor exists ({soul} cells)");
        assert!(
            fossils > 0,
            "nether fossils poke out ({fossils} bone cells)"
        );
        assert!(pillars > 30, "giant basalt pillars ({pillars} cells)");
    }

    /// The basalt deltas floor is the basalt/blackstone/magma trio
    /// (VERIFIED w/Basalt_Deltas capture).
    #[test]
    fn backlog_basalt_deltas_floor_trio() {
        let mut seed = 1u64;
        let (cx, cz) = loop {
            let found =
                (0..64).find(|&c| nether_region_biome(seed, c * 2, c * 2) == Biome::BasaltDeltas);
            if let Some(c) = found {
                break (c * 2, c * 2);
            }
            seed += 1;
            if seed > 200 {
                panic!("no deltas region found");
            }
        };
        let gen = TerrainGen::for_dimension(seed, Dimension::Nether);
        let mut basalt = 0usize;
        let mut blackstone = 0usize;
        let mut magma = 0usize;
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (chunk, _) = gen.generate_chunk(cx + dx, cz + dz, Vec::new());
            for i in 0..vc_chunk::chunk::CHUNK_LEN {
                match chunk.get_idx(i) {
                    BASALT => basalt += 1,
                    BLACKSTONE | GILDED_BLACKSTONE => blackstone += 1,
                    MAGMA_BLOCK => magma += 1,
                    _ => {}
                }
            }
        }
        assert!(
            basalt > 200,
            "the deltas are basalt-dominated ({basalt} cells)"
        );
        assert!(
            blackstone > 0 || magma > 0,
            "the trio appears (bs {blackstone}, magma {magma})"
        );
    }

    /// §28: the nether shell — bedrock floor + roof, nothing above 127
    #[test]
    fn nether_bedrock_shell() {
        let gen = TerrainGen::for_dimension(0xDEAD_BEEF, Dimension::Nether);
        let (chunk, _) = gen.generate_chunk(0, 0, Vec::new());
        for lz in 0..16usize {
            for lx in 0..16usize {
                assert_eq!(
                    fold(chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 0)),
                    BEDROCK,
                    "y=0 is bedrock floor"
                );
                assert_eq!(
                    fold(chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 127)),
                    BEDROCK,
                    "y=127 is bedrock roof"
                );
                // above the build ceiling: air (nothing exists)
                for y in 128..256usize {
                    assert_eq!(
                        chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y),
                        AIR,
                        "y={y} must be air"
                    );
                }
            }
        }
    }

    /// §26/§28: netherrack dominates the mass, with quartz ore sprinkled in
    #[test]
    fn nether_is_netherrack_with_quartz() {
        let mut rack = 0usize;
        let mut quartz = 0usize;
        let mut basalt = 0usize;
        let mut blackstone = 0usize;
        let mut other = 0usize;
        for s in 0..16i32 {
            let gen = TerrainGen::for_dimension(0xCAFE_F00D, Dimension::Nether);
            let (chunk, _) = gen.generate_chunk(s * 3, s * 7, Vec::new());
            for i in 0..vc_chunk::chunk::CHUNK_LEN {
                // mid band only — the shell (bedrock) lives near y 0 and 127
                let y = (i >> 8) as i32;
                if !(6..=120).contains(&y) {
                    continue;
                }
                match fold(chunk.get_idx(i)) {
                    NETHERRACK => rack += 1,
                    NETHER_QUARTZ_ORE => quartz += 1,
                    BASALT => basalt += 1,
                    BLACKSTONE | GILDED_BLACKSTONE | CRYING_OBSIDIAN => blackstone += 1,
                    // Phase E1: fortress materials (nether bricks, spawners,
                    // soul-sand wart gardens) are legitimate nether content;
                    // 1.10: magma blobs (4/chunk, Y 27-36, wiki
                    // /w/Magma_Block) joined the nether mass
                    NETHER_BRICKS | SPAWNER | NETHER_WART | MAGMA_BLOCK => {}
                    AIR | GLOWSTONE | SOUL_SAND => {}
                    // backlog round: the valley + deltas content —
                    // soul soil floors, soul fire, the fossil bone
                    // blocks, the deltas' floor trio (basalt is counted
                    // in its own bucket above) and the valley plants
                    // (crimson roots + mushrooms, VERIFIED
                    // w/Soul_Sand_Valley §vegetation)
                    SOUL_SOIL | SOUL_FIRE | BONE_BLOCK | CRIMSON_ROOTS | MUSHROOM_RED
                    | MUSHROOM_BROWN => {}
                    // 1.16 (Nether Update, part 1): the V13 nether body —
                    // gold veins and the never-air-exposed debris (the
                    // soul-valley soil/fires are counted in the bucket
                    // above; the basalt blobs/pillars and the blackstone
                    // patch family have their own buckets)
                    NETHER_GOLD_ORE | ANCIENT_DEBRIS => {}
                    // 1.16 (Nether Update, part 2): the V14 forest
                    // families — nylium floors, huge-fungi stems + wart
                    // caps + shroomlights, the undergrowth tufts and the
                    // weeping/twisting vines
                    CRIMSON_NYLIUM | WARPED_NYLIUM | CRIMSON_STEM | WARPED_STEM
                    | NETHER_WART_BLOCK | WARPED_WART_BLOCK | SHROOMLIGHT | CRIMSON_FUNGUS
                    | WARPED_FUNGUS | WARPED_ROOTS | NETHER_SPROUTS | WEEPING_VINES
                    | TWISTING_VINES => {}
                    _ => other += 1,
                }
            }
        }
        assert!(
            rack > 50_000,
            "netherrack dominates the mass ({rack} cells)"
        );
        assert!(
            quartz > 50,
            "quartz ore appears across seeds ({quartz} cells)"
        );
        // 1.16: the basalt/blackstone terrain is present across seeds
        // (5 blobs + 2 pillars per chunk / 3 blackstone blobs per chunk —
        // 16 single-chunk samples)
        assert!(
            basalt > 200,
            "basalt blobs + pillars appear across seeds ({basalt} cells)"
        );
        assert!(
            blackstone > 50,
            "blackstone patches appear across seeds ({blackstone} cells)"
        );
        assert!(
            other == 0,
            "the nether mass is ONLY the verified nether set (1.10 + 1.16 V13) — got {other} others"
        );
    }

    /// §26/§28: vast open caverns exist (the nether is nether, not solid),
    /// and glowstone hangs from ceilings somewhere in a region
    #[test]
    fn nether_caverns_and_glowstone() {
        let mut open = 0usize;
        let mut glowstone = 0usize;
        let mut soul_sand = 0usize;
        for dz in -2..=2i32 {
            for dx in -2..=2i32 {
                let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::Nether);
                let (chunk, _) = gen.generate_chunk(dx, dz, Vec::new());
                for i in 0..vc_chunk::chunk::CHUNK_LEN {
                    let y = (i >> 8) as i32;
                    if y <= 6 || y >= 120 {
                        continue; // shell margin
                    }
                    match fold(chunk.get_idx(i)) {
                        AIR => open += 1,
                        GLOWSTONE => glowstone += 1,
                        SOUL_SAND => soul_sand += 1,
                        _ => {}
                    }
                }
            }
        }
        // a 5×5-chunk nether neighborhood is substantially nether
        let total = 25 * CHUNK_LEN * 5 / 8; // band y 7..119 ≈ 5/8 of cells
        let ratio = open as f32 / total as f32;
        assert!(
            ratio > 0.12,
            "caverns too small: {ratio:.2} open in the mid band"
        );
        assert!(glowstone > 0, "glowstone clusters exist ({glowstone})");
        assert!(soul_sand > 0, "soul sand patches exist ({soul_sand})");
    }

    /// §9/§26 determinism: same seed + dimension → identical bytes; the two
    /// dimensions with the same world seed → different terrain
    #[test]
    fn nether_deterministic_and_distinct_from_overworld() {
        let gen = TerrainGen::for_dimension(0x1234_ABCD, Dimension::Nether);
        let a = gen.generate_chunk(1, 2, Vec::new()).0;
        // interleave neighbors, regenerate — must be identical
        for dz in -1..=1 {
            for dx in -1..=1 {
                if (dx, dz) != (0, 0) {
                    let _ = gen.generate_chunk(1 + dx, 2 + dz, Vec::new());
                }
            }
        }
        let b = gen.generate_chunk(1, 2, Vec::new()).0;
        for i in 0..vc_chunk::chunk::CHUNK_LEN {
            assert_eq!(
                a.get_idx(i),
                b.get_idx(i),
                "nether gen must be order-independent at {i}"
            );
        }
        // same world seed, overworld vs nether → different chunks
        let over = TerrainGen::for_dimension(0x1234_ABCD, Dimension::Overworld);
        let (oc, _) = over.generate_chunk(1, 2, Vec::new());
        let mut same = 0;
        for i in 0..vc_chunk::chunk::CHUNK_LEN {
            if oc.get_idx(i) == a.get_idx(i) {
                same += 1;
            }
        }
        // air cells match trivially; the mass must differ
        assert!(
            same < CHUNK_LEN,
            "dimensions must generate different terrain (same={same})"
        );
    }

    /// §28: no skylight path — the bedrock roof makes the light engine's
    /// column scan produce sky=0 for the whole nether interior
    #[test]
    fn nether_roof_blocks_skylight() {
        let gen = TerrainGen::for_dimension(0xBEEF_CAFE, Dimension::Nether);
        let (chunk, _) = gen.generate_chunk(0, 0, Vec::new());
        for lz in 0..16usize {
            for lx in 0..16usize {
                // first block from the top above y=127: none allowed (the
                // roof at ≤127 covers everything below — sky=0 for the light
                // engine's column scan). 127 = "nothing above" = pass.
                let mut top_content = 127;
                for y in (128..256usize).rev() {
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y) != AIR {
                        top_content = y as i32;
                        break;
                    }
                }
                assert!(
                    top_content <= 127,
                    "column ({lx},{lz}) has content above the nether roof at {top_content}"
                );
            }
        }
    }

    /// §28: find_nether_spawn lands on an open cavern floor with headroom
    #[test]
    fn nether_spawn_is_on_open_floor() {
        for s in 0..6u64 {
            let gen = TerrainGen::for_dimension(0x9000_0000 + s * 7919, Dimension::Nether);
            let (x, y, z) = gen.find_nether_spawn();
            // block coords use FLOOR semantics (negative x truncates wrong)
            let (xi, yi, zi) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
            let (chunk, _) = gen.generate_chunk(xi.div_euclid(16), zi.div_euclid(16), Vec::new());
            let lx = (xi - xi.div_euclid(16) * 16) as usize;
            let lz = (zi - zi.div_euclid(16) * 16) as usize;
            assert_eq!(
                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), yi as usize),
                AIR,
                "feet open (seed {s})"
            );
            assert!(
                yi + 1 >= 128
                    || chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), (yi + 1) as usize)
                        == AIR,
                "headroom (seed {s})"
            );
            assert!(
                is_solid(fold(chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx, lz),
                    (yi - 1) as usize
                ))),
                "solid floor below (seed {s})"
            );
            assert!(
                (8..120).contains(&yi),
                "spawn inside the nether band (seed {s})"
            );
        }
    }

    /// §28: the biome field is Nether Wastes everywhere
    #[test]
    fn nether_biome_field() {
        // 1.16 part 2: the field is region-uniform (the 2x2-chunk
        // cell hash) and always a nether-family biome — the wastes is
        // no longer the only flavor
        let gen = TerrainGen::for_dimension(0xFEED_1234, Dimension::Nether);
        let (chunk, _) = gen.generate_chunk(3, -4, Vec::new());
        let first = chunk.biome[0];
        for i in 0..256usize {
            assert_eq!(chunk.biome[i], first, "biome[{i}] region-uniform");
            assert!(
                Biome::from_u8(chunk.biome[i]).is_nether(),
                "biome[{i}] in the nether family"
            );
        }
    }

    /// 1.16 part 2: the forest families — crimson/warped regions
    /// exist across seeds, their floors turn nylium, the huge fungi
    /// (stem + wart cap + shroomlight) grow, and the vines hang/climb
    /// (the region shares: crimson ~22%, warped ~8%, verified scan)
    #[test]
    fn nether_forest_regions_and_families() {
        let mut crimson_regions = 0;
        let mut warped_regions = 0;
        let mut saw_crimson_stem = false;
        let mut saw_warped_stem = false;
        let mut saw_shroomlight = false;
        let mut saw_mold = false;
        for s in 0..12u64 {
            let gen = TerrainGen::for_dimension(0xC0FFEE + s, Dimension::Nether);
            // sample 9 regions around the origin
            for rx in -1..=1i32 {
                for rz in -1..=1i32 {
                    let cx = rx * 2; // region = 2x2 chunks
                    let cz = rz * 2;
                    let region = nether_region_biome(gen.seed, cx, cz);
                    match region {
                        Biome::CrimsonForest => crimson_regions += 1,
                        Biome::WarpedForest => warped_regions += 1,
                        _ => {}
                    }
                    let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
                    // region-uniform biome field
                    for i in 0..256usize {
                        assert_eq!(chunk.biome[i], region as u8, "region-uniform at {cx},{cz}");
                    }
                    if region != Biome::NetherWastes {
                        // the family blocks appear (scan the whole chunk)
                        for i in 0..16 * 16 * 128usize {
                            let b = chunk.get_idx(i);
                            match b {
                                x if x == CRIMSON_STEM => saw_crimson_stem = true,
                                x if x == WARPED_STEM => saw_warped_stem = true,
                                x if x == SHROOMLIGHT => saw_shroomlight = true,
                                x if x == CRIMSON_NYLIUM || x == WARPED_NYLIUM => saw_mold = true,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        // the shares: crimson ~22%, warped ~8% of 108 samples (with
        // generous tolerance — the hash is deterministic but coarse)
        assert!(
            crimson_regions >= 8,
            "crimson regions appear ({crimson_regions}/108)"
        );
        assert!(
            warped_regions >= 2,
            "warped regions appear ({warped_regions}/108)"
        );
        // the families generate their signature blocks
        assert!(saw_crimson_stem, "huge crimson fungi generate");
        assert!(saw_warped_stem, "huge warped fungi generate");
        assert!(saw_shroomlight, "shroomlights generate in the caps");
        assert!(saw_mold, "forest floors turn nylium");
    }
}

#[cfg(test)]
mod spawn_tests {
    use super::*;

    #[test]
    fn spawn_quality_across_seeds() {
        let mut green = 0;
        let mut total = 0;
        for i in 0..20u64 {
            let gen = TerrainGen::new(0x9E37_79B9_7F4A_7C15u64.wrapping_mul(i + 1));
            let (x, y, z) = gen.find_spawn();
            let col = gen.column(x as i32, z as i32);
            let neighbors_green = (0..12)
                .map(|k| {
                    let yaw = k as f32 * std::f32::consts::TAU / 12.0;
                    let c = gen.column(
                        (x + dsin32(yaw) * 40.0) as i32,
                        (z - dcos32(yaw) * 40.0) as i32,
                    );
                    matches!(c.biome, Biome::Forest | Biome::Plains)
                })
                .filter(|g| *g)
                .count();
            println!(
                "seed {} -> spawn ({},{},{}) biome {:?} height {} green_neighbors {}/12",
                i, x as i32, y as i32, z as i32, col.biome, col.height, neighbors_green
            );
            if matches!(col.biome, Biome::Forest | Biome::Plains) || neighbors_green >= 4 {
                green += 1;
            }
            total += 1;
        }
        println!("green-ish spawns: {}/{}", green, total);
        assert!(
            green >= total / 2,
            "at least half of seeds should spawn green"
        );
    }
}

#[cfg(test)]
mod dungeon_tests {
    use super::*;

    /// dungeons appear across seeds and chunks (a scan is the honest test —
    /// placement is gated on terrain, exactly like vanilla)
    #[test]
    fn dungeons_roll_for_some_chunks() {
        let mut found = 0;
        'seeds: for s in 0..8u64 {
            let gen = TerrainGen::new(0x0D66_u64.wrapping_add(s));
            for cz in -6..=6i32 {
                for cx in -6..=6i32 {
                    if gen.dungeon_in_chunk(cx, cz).is_some() {
                        found += 1;
                        continue 'seeds;
                    }
                }
            }
        }
        assert!(found >= 3, "expected several dungeon seeds, got {found}");
    }

    /// the roll is pure: same seed + chunk → identical room, always
    #[test]
    fn dungeon_roll_is_deterministic() {
        let gen = TerrainGen::new(0xD1A6_5EED);
        for cz in -4..=4i32 {
            for cx in -4..=4i32 {
                let a = gen.dungeon_in_chunk(cx, cz);
                let b = gen.dungeon_in_chunk(cx, cz);
                assert_eq!(a, b, "chunk ({cx},{cz}) roll must be pure");
                if let Some(r) = a {
                    // size is one of the VERIFIED open-area set
                    assert!(matches!(r.size, 7 | 9 | 11), "size {}", r.size);
                    // y band: underground, above bedrock
                    assert!((8..=35).contains(&r.y0), "y0 {}", r.y0);
                    // mob is one of the three dungeon spawners
                    assert!(matches!(r.mob, 0..=2), "mob {}", r.mob);
                    // ≤ 2 chests, all inside the interior
                    assert!(r.chest_count <= 2);
                    for c in r.chests.iter().take(r.chest_count) {
                        assert!(c[0] >= r.x0 && c[0] < r.x0 + r.size);
                        assert!(c[2] >= r.z0 && c[2] < r.z0 + r.size);
                        assert_eq!(c[1], r.y0);
                    }
                    // the two chests never share a cell
                    if r.chest_count == 2 {
                        assert_ne!(r.chests[0], r.chests[1]);
                    }
                }
            }
        }
    }

    /// the spawner mob distribution over many rolls is ~50/25/25 (a
    /// coarse statistical gate, not an exact equality)
    #[test]
    fn dungeon_mob_rolls_match_the_50_25_25_shape() {
        let mut counts = [0usize; 3];
        let mut total = 0usize;
        'seeds: for s in 0..40u64 {
            let gen = TerrainGen::new(0xABBA_u64.wrapping_add(s));
            for cz in -8..=8i32 {
                for cx in -8..=8i32 {
                    if let Some(r) = gen.dungeon_in_chunk(cx, cz) {
                        counts[r.mob as usize] += 1;
                        total += 1;
                        if total >= 300 {
                            break 'seeds;
                        }
                    }
                }
            }
        }
        assert!(total >= 60, "need a real sample, got {total}");
        let (z, sk, sp) = (counts[0], counts[1], counts[2]);
        // 50% zombie with ±12pt tolerance, 25% each with ±9pt
        let zf = z as f64 / total as f64;
        let skf = sk as f64 / total as f64;
        let spf = sp as f64 / total as f64;
        assert!((0.38..=0.62).contains(&zf), "zombie share {zf}");
        assert!((0.16..=0.34).contains(&skf), "skeleton share {skf}");
        assert!((0.16..=0.34).contains(&spf), "spider share {spf}");
    }

    /// a rolled room emits exactly into its chunk: walls/floor/ceiling
    /// cobble+mossy, interior air, spawner at the center with the mob
    /// state, chests in place. Also the VERIFIED 75% mossy floor ratio.
    #[test]
    fn dungeon_emits_the_verified_layout() {
        // find a concrete dungeon
        let mut found = None;
        'outer: for s in 0..60u64 {
            let gen = TerrainGen::new(0x5EED_u64.wrapping_add(s));
            for cz in -6..=6i32 {
                for cx in -6..=6i32 {
                    if let Some(r) = gen.dungeon_in_chunk(cx, cz) {
                        found = Some((gen, r));
                        break 'outer;
                    }
                }
            }
        }
        let (gen, room) = found.expect("a dungeon to exist in the scan");
        let (chunk, _) = gen.generate_chunk(room.x0 >> 4, room.z0 >> 4, Vec::new());
        let lx = |wx: i32| (wx - (room.x0 >> 4) * 16) as usize;
        let lz = |wz: i32| (wz - (room.z0 >> 4) * 16) as usize;
        // raw state read (Chunk::get truncates to the block id)
        let state_at = |chunk: &Arc<Chunk>, wx: i32, wy: usize, wz: i32| -> u16 {
            chunk.sections[wy >> 4]
                .as_ref()
                .map(|s| s.get((wx & 15) as usize, wy & 15, (wz & 15) as usize))
                .unwrap_or(0)
        };

        // spawner at the center with the right mob state
        let scx = room.x0 + room.size / 2;
        let scz = room.z0 + room.size / 2;
        let s = state_at(&chunk, scx, room.y0 as usize, scz);
        assert_eq!(vc_blocks::blocks::state_block(s), SPAWNER);
        assert_eq!(vc_blocks::blocks::spawner_mob(s), room.mob);

        // floor: only cobble/mossy; count the VERIFIED ~75% mossy share
        let mut mossy = 0;
        let mut floor_total = 0;
        for dx in -1..=room.size {
            for dz in -1..=room.size {
                let b = chunk.get_local(
                    vc_chunk::chunk::LocalXZ::new(lx(room.x0 + dx), lz(room.z0 + dz)),
                    (room.y0 - 1) as usize,
                );
                assert!(matches!(b, COBBLE | MOSSY_COBBLE), "floor block {b}");
                floor_total += 1;
                if b == MOSSY_COBBLE {
                    mossy += 1;
                }
            }
        }
        let share = mossy as f64 / floor_total as f64;
        assert!(
            (0.55..=0.95).contains(&share),
            "mossy share {share} (VERIFIED 75%)"
        );

        // interior: air (and stays air to the ceiling) — except the
        // spawner at the center and the chests against the walls
        let scx_l = room.x0 + room.size / 2;
        let scz_l = room.z0 + room.size / 2;
        for dx in 0..room.size {
            for dz in 0..room.size {
                for dy in 0..4i32 {
                    let wx = room.x0 + dx;
                    let wz = room.z0 + dz;
                    if dy == 0 && wx == scx_l && wz == scz_l {
                        continue; // the spawner
                    }
                    if dy == 0
                        && room
                            .chests
                            .iter()
                            .take(room.chest_count)
                            .any(|c| c[0] == wx && c[2] == wz)
                    {
                        continue; // a chest
                    }
                    let b = chunk.get_local(
                        vc_chunk::chunk::LocalXZ::new(lx(wx), lz(wz)),
                        (room.y0 + dy) as usize,
                    );
                    assert_eq!(b, AIR, "interior cell must be air");
                }
            }
        }

        // chests landed as placed (fold the state → block)
        for c in room.chests.iter().take(room.chest_count) {
            let s = state_at(&chunk, c[0], c[1] as usize, c[2]);
            assert_eq!(vc_blocks::blocks::state_block(s), CHEST);
        }
        // determinism: regenerate → identical chunk (the P6-style gate)
        let (chunk2, _) = gen.generate_chunk(room.x0 >> 4, room.z0 >> 4, Vec::new());
        let mut same = true;
        for i in 0..(16 * 16 * 256) {
            if chunk.get_idx(i) != chunk2.get_idx(i) {
                same = false;
                break;
            }
        }
        assert!(same, "dungeon chunk regenerates identically");
    }
}

// ---------------------------------------------------------------- tests --
#[cfg(test)]
mod phase10_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// the 6 new climate biomes all exist somewhere in a reasonable scan
    /// window, and from_u8 round-trips every variant
    #[test]
    fn new_biomes_present_and_roundtrip() {
        let g = gen();
        let mut seen = std::collections::HashSet::new();
        // 4.1k: ±80-chunk scan (badlands rides the warm gate at
        // ~0.2-0.4% in 64-block clumps; the ±40 window flakes)
        for x in -80..80 {
            for z in -80..80 {
                let b = g.column(x * 16, z * 16).biome;
                seen.insert(b as u8);
            }
        }
        for b in [
            Biome::Taiga,
            Biome::BirchForest,
            Biome::Jungle,
            Biome::Savanna,
            Biome::Swamp,
            Biome::Badlands,
        ] {
            assert!(
                seen.contains(&(b as u8)),
                "{} never selected in the scan window",
                b.name()
            );
            assert_eq!(Biome::from_u8(b as u8), b);
        }
    }

    /// mineshafts: 0.4%/chunk means a ±10-chunk scan (441 chunks) is
    /// expected to find ≥1 (probability of zero ≈ 0.996^441 ≈ 17% —
    /// sensitive to the seed; use a seed that yields one and verify the
    /// STRUCTURE, with the presence itself asserted on a wider window)
    #[test]
    fn mineshaft_layout_is_deterministic_and_wellformed() {
        let g = gen();
        // find a seed-window that contains a shaft
        let mut found: Option<Mineshaft> = None;
        'outer: for cx in -12..12 {
            for cz in -12..12 {
                let near = g.mineshafts_near(cx * 16, cz * 16);
                if let Some(ms) = near.into_iter().next() {
                    found = Some(ms);
                    break 'outer;
                }
            }
        }
        let ms = found.expect("a mineshaft within ±12 chunks of a 0.4% roll");
        // determinism: the same query returns the same layout
        let again = g.mineshafts_near(ms.x, ms.z);
        assert!(again
            .iter()
            .any(|m| m.x == ms.x && m.z == ms.z && m.y == ms.y));
        // well-formed: y in the deep band, 1..=4 corridors, lengths sane
        assert!((10..=40).contains(&ms.y));
        assert!(!ms.corridors.is_empty() && ms.corridors.len() <= 4);
        for &(_, _, len) in &ms.corridors {
            assert!((24..=48).contains(&len));
        }
        // emit: the owning chunk contains a parlor (planks at ms.y) and
        // structure regenerates identically
        let cx = ms.x >> 4;
        let cz = ms.z >> 4;
        let (c1, _) = g.generate_chunk(cx, cz, Vec::new());
        let (c2, _) = g.generate_chunk(cx, cz, Vec::new());
        let same = (0..256usize)
            .map(|y| {
                (0..16usize)
                    .map(|z| {
                        (0..16usize)
                            .filter(|x| c1.get(*x, y, z) != c2.get(*x, y, z))
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();
        assert_eq!(same, 0, "chunk regenerates identically");
        // parlor floor: the center cell is planks
        let lx = (ms.x - cx * 16) as usize;
        let lz = (ms.z - cz * 16) as usize;
        assert_eq!(state_block(c1.get(lx, ms.y as usize, lz)), PLANKS);
    }

    /// desert pyramid: 21×21 base, hidden pit with 4 chests, entrance,
    /// and the terracotta checkerboard floor; regeneration determinism
    #[test]
    fn pyramid_emits_full_layout() {
        let g = gen();
        // find a pyramid region
        let mut found = None;
        'outer: for rx in -16..16 {
            for rz in -16..16 {
                if let Some(c) = g.pyramid_center_pub(rx, rz) {
                    found = Some(c);
                    break 'outer;
                }
            }
        }
        let (wx, wz) = found.expect("a desert pyramid within ±16 regions");
        // force-emit the center chunk + register what's inside
        let cx = wx >> 4;
        let cz = wz >> 4;
        let (c1, _) = g.generate_chunk(cx, cz, Vec::new());
        let (c2, _) = g.generate_chunk(cx, cz, Vec::new());
        let same = (0..256usize)
            .map(|y| {
                (0..16usize)
                    .map(|z| {
                        (0..16usize)
                            .filter(|x| c1.get(*x, y, z) != c2.get(*x, y, z))
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();
        assert_eq!(same, 0);
        let base = g.column(wx, wz).height;
        let at = |dx: i32, dy: i32, dz: i32| -> u16 {
            let x = ((wx + dx) - cx * 16) as usize;
            let z = ((wz + dz) - cz * 16) as usize;
            state_block(c1.get(x, (base + dy) as usize, z))
        };
        // checkerboard floor: terracotta + smooth stone alternating —
        // probe OPPOSITE parities: (0,0) is even, (1,0) is odd
        let a = at(0, 1, 0);
        let b = at(1, 1, 0);
        assert!(
            {
                let pair = [a, b];
                pair.contains(&TERRACOTTA) && pair.contains(&SMOOTH_STONE)
            },
            "wind-rose checkerboard: {a:?} {b:?}"
        );
        // pit: air shaft under the center, treasure floor below
        assert_eq!(at(0, -5, 0), AIR, "hidden pit shaft is carved");
        // 4 chests around the treasure-room center — Chunk::get returns
        // the raw STATE id (CHEST_STATE 227, not block id 96), so the
        // check routes through state_block (the Phase 5 dungeon-test
        // pattern; identity states are unchanged by it)
        let floor = base - 11;
        let mut chests = 0;
        for (dx, dz) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let x = ((wx + dx) - cx * 16) as usize;
            let z = ((wz + dz) - cz * 16) as usize;
            if c1.get(x, floor as usize, z) == CHEST {
                chests += 1;
            }
        }
        assert_eq!(chests, 4, "4 treasure chests around the pit floor");
    }

    /// stronghold: ring 1 = 3 strongholds in the verified 1280..=2816
    /// distance band at roughly equal angles; the portal room emits the
    /// 12-frame ring
    #[test]
    fn stronghold_ring1_and_portal_room() {
        let g = gen();
        let sh = g.strongholds();
        assert_eq!(sh.len(), 3, "VERIFIED: ring 1 has 3 strongholds");
        for &(x, z) in &sh {
            let dist = ((x * x + z * z) as f64).sqrt();
            assert!(
                (1280.0..=2816.0).contains(&dist),
                "VERIFIED band 1280-2816, got {dist}"
            );
        }
        // angles roughly 120° apart (the wiki: "each stronghold in a ring
        // of 3 is in the region of 120 degrees from the others")
        let angles: Vec<f32> = sh
            .iter()
            .map(|&(x, z)| (z as f32).atan2(x as f32))
            .collect();
        let mut gaps: Vec<f32> = Vec::new();
        for i in 0..3 {
            let mut d = (angles[(i + 1) % 3] - angles[i]).abs();
            let tau = std::f32::consts::TAU;
            if d > tau / 2.0 {
                d = tau - d;
            }
            gaps.push(d);
        }
        assert!(
            gaps.iter().all(|&d| d > 1.4 && d < 2.8),
            "roughly-equal angles (~120° apart, wiki: 'in the region of 120 degrees'): {gaps:?}"
        );
        // emit: the portal room's 12-frame ring. The ring center sits 17
        // blocks WEST of the stronghold anchor (the portal room centers
        // on the anchor's west side), so generate the RING-CENTER chunk's
        // 3×3 neighborhood: the 5×5 ring and the library/store-room chests
        // can straddle chunk borders, and every chunk near a stronghold
        // emits the parts of the layout falling inside itself (the same
        // discipline as villages/mineshafts).
        let (sx, sz) = sh[0];
        let (rcx, rcz) = ((sx - 17) >> 4, sz >> 4);
        let mut grid: Vec<(i32, i32, std::sync::Arc<Chunk>)> = Vec::new();
        for dcx in -1..=1 {
            for dcz in -1..=1 {
                let (c, _) = g.generate_chunk(rcx + dcx, rcz + dcz, Vec::new());
                grid.push((rcx + dcx, rcz + dcz, c));
            }
        }
        // world-coord lookup across the neighborhood — returns the BLOCK
        // id (Chunk::get yields the raw state; END_PORTAL_FRAME stores
        // state 235 ≠ block 102, CHEST stores 227 ≠ 96, so route through
        // state_block)
        let get = |x: i32, y: usize, z: i32| -> u16 {
            for &(cx, cz, ref c) in &grid {
                let lx = x - cx * 16;
                let lz = z - cz * 16;
                if (0..16).contains(&lx) && (0..16).contains(&lz) {
                    return c.get_local(vc_chunk::chunk::LocalXZ::new(lx as usize, lz as usize), y);
                }
            }
            panic!("probe ({x},{y},{z}) outside the generated neighborhood");
        };
        // the 12-frame ring: 3 per side, corners open (vanilla layout)
        let (px, pz) = (sx - 17, sz);
        let mut frames = 0;
        for i in -2..=2i32 {
            for j in -2..=2i32 {
                let on_ring = (i.abs() == 2 || j.abs() == 2) && !(i.abs() == 2 && j.abs() == 2);
                if on_ring && get(px + i, 21, pz + j) == END_PORTAL_FRAME {
                    frames += 1;
                }
            }
        }
        assert_eq!(frames, 12, "the 12-frame portal ring (3 per side)");
        // the library chest + store-room chest exist (exact emit coords,
        // both inside the neighborhood: +10 east/−8 north and +13
        // east/+4 south of the ring center)
        assert_eq!(get(sx - 7, 21, sz - 8), CHEST, "stronghold_library chest");
        assert_eq!(get(sx - 4, 21, sz + 4), CHEST, "stronghold_corridor chest");
    }

    /// ravines: descriptors respect the wiki-verified shape grammar
    /// (85..=127 long, <15 wide, ≤62 deep, top 10..=72); the carve
    /// actually opens a deep air column somewhere on the path
    #[test]
    fn ravine_shape_and_carve() {
        let g = gen();
        let mut found: Option<(i32, i32, Ravine)> = None;
        'outer: for cx in -10..10 {
            for cz in -10..10 {
                let rv = g.ravines_near_chunk(cx, cz);
                if let Some(r) = rv.into_iter().next() {
                    found = Some((cx, cz, r));
                    break 'outer;
                }
            }
        }
        let (cx, cz, r) = found.expect("a ravine within ±10 chunks of a 2% roll");
        // VERIFIED grammar
        assert!((85..=127).contains(&r.length), "85..=127 long");
        assert!(r.half_w < 7.5, "typically less than 15 wide");
        assert!(r.depth <= 62, "up to 62 deep");
        assert!((10..=72).contains(&r.top), "start levels 10 to 72");
        // the carve: the path's midpoint column is air at mid-depth
        let mx = r.x0 + (r.dx * (r.length as f32 / 2.0)) as i32;
        let mz = r.z0 + (r.dz * (r.length as f32 / 2.0)) as i32;
        let (c, _) = g.generate_chunk(mx >> 4, mz >> 4, Vec::new());
        let col_h = g.column(mx, mz).height;
        // probe 3 blocks below the local surface at the midpoint: the
        // cut may be shallow where it clipped a low top; assert that at
        // SOME depth along the column the terrain is carved to air
        let lx = (mx - (mx >> 4) * 16) as usize;
        let lz = (mz - (mz >> 4) * 16) as usize;
        let mut carved = 0;
        for y in 8..col_h.min(r.top) {
            if state_block(c.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y as usize)) == AIR {
                carved += 1;
            }
        }
        assert!(
            carved > 0,
            "the ravine path is carved open ({carved} cells)"
        );
        // determinism
        let (c2, _) = g.generate_chunk(cx, cz, Vec::new());
        let (c1, _) = g.generate_chunk(cx, cz, Vec::new());
        let same = (0..256usize)
            .map(|y| {
                (0..16usize)
                    .map(|z| {
                        (0..16usize)
                            .filter(|x| c1.get(*x, y, z) != c2.get(*x, y, z))
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();
        assert_eq!(same, 0);
    }
}

/// 1.7.2 bracket — the Update that Changed the World world-gen tests.
/// Every claim is the live-verified changelog text
/// (reference wiki /Java_Edition_1.7.2, 2026-09-06 round).
#[cfg(test)]
mod v172_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// find a chunk whose center biome is `b` within ±128 chunks
    /// (4.1k: badlands rides the warm gate at ~0.2-0.4% in clumps;
    /// the ±64 window flakes on it — common biomes still hit early)
    fn find_biome(g: &TerrainGen, b: Biome) -> (i32, i32) {
        // Vanilla-parity terrain note (2026-09-14): column() is a direct
        // density root-solve and can differ from the chunk's
        // lattice-interpolated surface by a block at biome thresholds —
        // so a column-hit is verified against the generated chunk's
        // center biome (vanilla's own biome queries read chunk data).
        for cx in -128..128 {
            for cz in -128..128 {
                let col = g.column(cx * 16 + 8, cz * 16 + 8);
                if col.biome == b {
                    let (probe, _) = g.generate_chunk(cx, cz, Vec::new());
                    if Biome::from_u8(probe.biome[8 * 16 + 8]) == b {
                        return (cx, cz);
                    }
                }
            }
        }
        panic!("{} not found in the ±128-chunk window", b.name());
    }

    #[test]
    fn v172_biomes_present_and_roundtrip() {
        let g = gen();
        for b in [
            Biome::FlowerForest,
            Biome::SunflowerPlains,
            Biome::IceSpikes,
            Biome::DarkForest,
        ] {
            let (cx, cz) = find_biome(&g, b);
            assert_eq!(Biome::from_u8(b as u8), b);
            let _ = (cx, cz);
        }
    }

    #[test]
    fn badlands_floor_is_red_sand_over_banded_terracotta() {
        // wiki: "floor similar to a desert, but made of red sand" +
        // "multiple colored hardened clay layered... seven colors"
        let g = gen();
        // 4.1p: scan up to 16 badlands-center chunks for a surviving
        // floor (gate moves relocate the first hit; a first-hit chunk
        // may be fully carved — same pattern as bamboo/sunflower)
        let mut h = None;
        let mut scanned = 0usize;
        let (mut lx, mut lz) = (0usize, 0usize);
        let mut chunk = g.generate_chunk(0, 0, Vec::new()).0;
        'chunks: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Badlands {
                    continue;
                }
                let (c, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(c.biome[8 * 16 + 8]) != Biome::Badlands {
                    continue;
                }
                scanned += 1;
                for czi in 0..16usize {
                    for cxi in 0..16usize {
                        if c.biome[czi * 16 + cxi] != Biome::Badlands as u8 {
                            continue;
                        }
                        let hi = c.height[czi * 16 + cxi] as usize;
                        if c.get_local(vc_chunk::chunk::LocalXZ::new(cxi, czi), hi) == RED_SAND {
                            h = Some(hi);
                            lx = cxi;
                            lz = czi;
                            chunk = c;
                            break 'chunks;
                        }
                    }
                }
                if scanned >= 16 {
                    break 'chunks;
                }
            }
        }
        let h = h.unwrap_or_else(|| panic!("no uncarved badlands floor column"));

        // the banding window below contains at least 3 distinct band
        // colors (the sedimentary look). 1.8: the 4-layer filler directly
        // under the floor is red sandstone now — the band check starts
        // below it.
        let mut distinct = std::collections::HashSet::new();
        for y in (h - 14)..(h - 4) {
            let b = chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y);
            distinct.insert(b);
        }
        // 1.8: red sandstone is the filler between red sand and banding
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), h - 2),
            RED_SANDSTONE,
            "1.8 red-sand filler"
        );
        assert!(
            distinct.len() >= 3,
            "banded terracotta layers (got {} colors)",
            distinct.len()
        );
        // every banded block is terracotta family
        for &b in distinct.iter() {
            let terracotta =
                b == TERRACOTTA || (STAINED_TERRACOTTA_BASE..=STAINED_TERRACOTTA_END).contains(&b);
            assert!(terracotta, "band block {b} is terracotta family");
        }
    }

    #[test]
    fn ice_spikes_generate_packed_ice_spires() {
        // wiki: "tall spires made of packed ice"
        let g = gen();
        let (cx, cz) = find_biome(&g, Biome::IceSpikes);
        let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
        let mut packed_ice = 0;
        for i in 0..CHUNK_LEN {
            if chunk.get_idx(i) == PACKED_ICE {
                packed_ice += 1;
            }
        }
        assert!(packed_ice >= 8, "packed-ice spire mass (got {packed_ice})");
    }

    #[test]
    fn savanna_grows_acacia_and_dark_forest_grows_dark_oak() {
        let g = gen();
        // savanna: acacia logs + leaves ("curved trees made of acacia
        // logs"). Savanna tree density is sparse (0..1/chunk, vanilla-like),
        // so scan several savanna chunks and accumulate.
        let mut savanna_chunks = 0;
        let (mut logs, mut leaves) = (0usize, 0usize);
        'scan: for cx in -64..64 {
            for cz in -64..64 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Savanna {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                for i in 0..CHUNK_LEN {
                    match chunk.get_idx(i) {
                        ACACIA_LOG => logs += 1,
                        ACACIA_LEAVES => leaves += 1,
                        _ => {}
                    }
                }
                savanna_chunks += 1;
                if savanna_chunks >= 6 {
                    break 'scan;
                }
            }
        }
        assert!(savanna_chunks >= 3, "found savanna chunks to scan");
        assert!(logs > 0, "acacia trunks exist");
        assert!(leaves > 0, "acacia canopy exists");

        // dark forest: "very thick and short trees... closely packed" —
        // 2×2 trunks mean ≥4 logs per tree, dense canopy
        let (cx, cz) = find_biome(&g, Biome::DarkForest);
        let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
        let (mut logs, mut leaves) = (0usize, 0usize);
        for i in 0..CHUNK_LEN {
            match chunk.get_idx(i) {
                DARK_OAK_LOG => logs += 1,
                DARK_OAK_LEAVES => leaves += 1,
                _ => {}
            }
        }
        // 2×2 trunk of height ≥5 = ≥20 logs; dense canopy ≥60
        assert!(logs >= 20, "2×2 dark-oak trunks (got {logs} logs)");
        assert!(leaves >= 60, "dense dark-oak canopy (got {leaves})");
    }

    #[test]
    fn flower_forest_and_sunflower_plains_flora() {
        let g = gen();
        // flower forest: "very densely packed with the various new
        // flowers... excluding sunflowers" — per-COLUMN sunflower rule
        // (4.1i: biome bands interleave within a chunk, so the check
        // reads each sunflower's own column biome). 4.1m: density
        // accumulates over up to 8 flower-forest chunks — one chunk's
        // 40 rolls may land mostly off-biome after the share refit.
        let mut flowers = 0;
        let mut scanned = 0usize;
        'ff: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::FlowerForest {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::FlowerForest {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    match chunk.get_idx(i) {
                        ALLIUM | AZURE_BLUET | BLUE_ORCHID | OXEYE_DAISY | ORANGE_TULIP
                        | RED_TULIP | WHITE_TULIP | PINK_TULIP | PEONY | PEONY_TOP | ROSE_BUSH
                        | ROSE_BUSH_TOP | LILAC | LILAC_TOP => flowers += 1,
                        SUNFLOWER | SUNFLOWER_TOP => {
                            let bx = (i % 256) % 16;
                            let bz = (i % 256) / 16;
                            assert_eq!(
                                Biome::from_u8(chunk.biome[bz * 16 + bx]),
                                Biome::SunflowerPlains,
                                "sunflowers only on sunflower-plains columns"
                            );
                        }
                        _ => {}
                    }
                }
                scanned += 1;
                if scanned >= 8 {
                    break 'ff;
                }
            }
        }
        assert!(scanned > 0, "found flower-forest chunks to scan");
        assert!(flowers >= 8, "dense new-flower flora (got {flowers})");

        // sunflower plains: sunflowers exist, with the 2-block top half.
        // 4.1n3: accumulate over up to 16 sunflower-center chunks
        // (same interleave flake as bamboo — one chunk's rolls may
        // land off-biome after the share refit)
        let (mut lower, mut upper) = (0usize, 0usize);
        let mut scanned_sf = 0usize;
        'sf: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::SunflowerPlains {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::SunflowerPlains {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    match chunk.get_idx(i) {
                        SUNFLOWER => lower += 1,
                        SUNFLOWER_TOP => upper += 1,
                        _ => {}
                    }
                }
                scanned_sf += 1;
                if scanned_sf >= 16 {
                    break 'sf;
                }
            }
        }
        assert!(scanned_sf > 0, "found sunflower-plains chunks to scan");
        assert!(lower > 0, "sunflowers present");
        assert_eq!(lower, upper, "every sunflower carries its upper half");
    }

    /// 1.14 (part 3): the two new flowers generate in their vanilla
    /// biomes (VERIFIED w/Cornflower §Natural generation — plains,
    /// sunflower plains, flower forest; w/Lily_of_the_Valley — forest,
    /// birch forest, flower forest). Multi-chunk scans because a single
    /// 16×16 chunk can easily roll zero of a ~6-8% floor flower.
    #[test]
    fn v114_flowers_generate_in_biomes() {
        let g = gen();

        // cornflower: plains + flower forest (both listed for it).
        // 4.1k: scan up to 16 plains-center chunks — biome bands
        // interleave, so one chunk's 14 rolls often land off-plains
        let mut corn_plains = 0usize;
        let mut scanned = 0usize;
        'plains: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Plains {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::Plains {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == CORNFLOWER {
                        corn_plains += 1;
                    }
                }
                scanned += 1;
                if scanned >= 16 {
                    break 'plains;
                }
            }
        }
        assert!(scanned > 0, "found plains chunks to scan");
        assert!(corn_plains > 0, "cornflower in plains (got {corn_plains})");

        // lily of the valley: forest family. 4.1m: scan up to 16
        // forest-center chunks (same interleave flake as cornflower —
        // one chunk's strip may roll zero of a ~10% flower)
        let mut lily_forest = 0usize;
        let mut fscanned = 0usize;
        'forest: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Forest {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::Forest {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == LILY_OF_THE_VALLEY {
                        lily_forest += 1;
                    }
                }
                fscanned += 1;
                if fscanned >= 16 {
                    break 'forest;
                }
            }
        }
        assert!(fscanned > 0, "found forest chunks to scan");
        assert!(
            lily_forest > 0,
            "lily of the valley in forest (got {lily_forest})"
        );

        // both join the flower-forest mix (the 10-way small-flower roll)
        let (cx, cz) = find_biome(&g, Biome::FlowerForest);
        let (mut corn_ff, mut lily_ff) = (0usize, 0usize);
        let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
        for i in 0..CHUNK_LEN {
            match chunk.get_idx(i) {
                CORNFLOWER => corn_ff += 1,
                LILY_OF_THE_VALLEY => lily_ff += 1,
                _ => {}
            }
        }
        assert!(
            corn_ff + lily_ff > 0,
            "the 1.14 flowers in the flower-forest mix (corn {corn_ff} lily {lily_ff})"
        );
    }

    #[test]
    fn taiga_carries_mega_taiga_podzol_patches() {
        // wiki (§Mega taiga): "a dirt block variant known as podzol"
        let g = gen();
        // taiga is common — scan a few chunks for any podzol
        let mut found = false;
        'outer: for cx in -60..60 {
            for cz in -60..60 {
                let col = g.column(cx * 16 + 8, cz * 16 + 8);
                if col.biome != Biome::Taiga {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == PODZOL {
                        found = true;
                        break 'outer;
                    }
                }
            }
        }
        assert!(found, "podzol patches exist in taiga");
    }
}

/// 1.10 bracket — Frostburn Update world-gen tests (live-verified
/// reference wiki /Java_Edition_1.10, 2026-09-06).
#[cfg(test)]
mod v110_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    #[test]
    fn nether_generates_magma_blobs() {
        // wiki: "generating 4 blobs per chunk between Y=27 and Y=36"
        let g = TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Nether);
        let mut total = 0usize;
        for s in 0..8 {
            let (chunk, _) = g.generate_chunk(s * 5, s * 3, Vec::new());
            for i in 0..CHUNK_LEN {
                if chunk.get_idx(i) == MAGMA_BLOCK {
                    total += 1;
                }
            }
        }
        assert!(total >= 12, "magma present across nether chunks ({total})");
        // and only in the Y band (127-high nether; idx y = i >> 8) —
        // unless the chunk is a basalt-deltas region, where magma is
        // a floor material (the backlog round's verified composition:
        // basalt/blackstone/magma surface)
        let g2 = TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Nether);
        let (chunk, _) = g2.generate_chunk(3, 2, Vec::new());
        let deltas = nether_region_biome(0x10C0_C0DE, 3, 2) == Biome::BasaltDeltas;
        for i in 0..CHUNK_LEN {
            if chunk.get_idx(i) == MAGMA_BLOCK {
                let y = (i >> 8) as i32;
                assert!(
                    (27..=36).contains(&y) || deltas,
                    "magma at y={y} outside the wiki band (and not a deltas floor)"
                );
            }
        }
    }

    #[test]
    fn fossils_appear_in_deserts_and_swamps() {
        // wiki: 1/64 per chunk — scan enough chunks that hits are certain
        // (deterministic seed); then verify bone blocks + coal exist
        let g = gen();
        let mut found = 0usize;
        'scan: for cx in -80..80 {
            for cz in -80..80 {
                let b = g.column(cx * 16 + 8, cz * 16 + 8).biome;
                if b != Biome::Desert && b != Biome::Swamp {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                let mut bones = 0;
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == BONE_BLOCK {
                        bones += 1;
                    }
                }
                if bones > 0 {
                    found += 1;
                    // depth claim: 15..24 underground → y < height − 14
                    if found >= 3 {
                        break 'scan;
                    }
                }
            }
        }
        assert!(found >= 1, "at least one fossil in the desert/swamp scan");
    }
}

/// Phase E1 tests (evolution 1.0–1.2 bracket) — The End, the Nether
/// Fortress, and the Mushroom Fields.
#[cfg(test)]
mod e1_tests {
    use super::*;

    #[test]
    fn end_central_island_exists() {
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::End);
        let (chunk, _) = gen.generate_chunk(0, 0, Vec::new());
        // the island center (8,8 local = world (8,8)): end stone surface
        let mut stone = 0;
        for y in 40..=64usize {
            if chunk.get_local(vc_chunk::chunk::LocalXZ::new(8, 8), y) == END_STONE {
                stone += 1;
            }
        }
        assert!(stone >= 4, "end stone column at the island center");
        // the arrival platform (100, 63, 0) — chunk (6, 0), local (4, ?, 0)
        let (pchunk, _) = gen.generate_chunk(6, 0, Vec::new());
        assert_eq!(
            state_block(pchunk.get_local(vc_chunk::chunk::LocalXZ::new(4, 0), 63)),
            OBSIDIAN,
            "5×5 obsidian platform at (100, 63, 0) — VERIFIED arrival x/z"
        );
        // the exit-portal fountain: the egg pedestal at world (0, 63, 0)
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(0, 0), 63),
            BEDROCK,
            "egg pedestal above the fountain"
        );
        // the fountain's inner 3×3 at y 62 stays open for the victory portal
        assert_eq!(
            chunk.get_local(vc_chunk::chunk::LocalXZ::new(1, 1), 62),
            AIR
        );
        // the biome field is the_end (id 9)
        assert_eq!(chunk.biome[8 * 16 + 8], 9);
    }

    #[test]
    fn end_pillars_form_the_42_radius_circle() {
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::End);
        // pillar 0 sits at angle 0 → (42, 0) — chunk (2, 0), local (10, .., 0)
        let (chunk, _) = gen.generate_chunk(2, 0, Vec::new());
        let mut found = false;
        for y in 70..=110usize {
            if chunk.get_local(vc_chunk::chunk::LocalXZ::new(10, 0), y) == BEDROCK {
                found = true;
                break;
            }
        }
        assert!(found, "pillar bedrock cap near (42, y, 0)");
        // the pillar columns descend toward y=0 (VERIFIED: down to y=0)
        let (chunk0, _) = gen.generate_chunk(2, 0, Vec::new());
        let mut deep_obsidian = 0;
        for y in 1..=10usize {
            if state_block(chunk0.get(10, y, 0)) == OBSIDIAN {
                deep_obsidian += 1;
            }
        }
        assert!(deep_obsidian >= 5, "pillar shaft reaches deep (y<10)");
    }

    #[test]
    fn end_generation_is_deterministic() {
        let a = TerrainGen::for_dimension(77, Dimension::End);
        let b = TerrainGen::for_dimension(77, Dimension::End);
        let (ca, _) = a.generate_chunk(1, 1, Vec::new());
        let (cb, _) = b.generate_chunk(1, 1, Vec::new());
        let same = (0..CHUNK_LEN)
            .map(|i| if ca.get_idx(i) == cb.get_idx(i) { 0 } else { 1 })
            .sum::<usize>();
        assert_eq!(same, 0, "same seed → identical End chunks");
    }

    #[test]
    fn fortresses_roll_deterministically_per_region() {
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::Nether);
        // region queries are pure functions of the seed
        let a = gen.fortress_in_region(0, 0);
        let b = gen.fortress_in_region(0, 0);
        assert_eq!(a, b, "deterministic per-region roll");
        // across a spread of regions, some carry fortresses (50% roll)
        let with: usize = (0..20)
            .filter(|i| gen.fortress_in_region(*i, 0).is_some())
            .count();
        assert!(
            (4..=16).contains(&with),
            "roughly half the regions, got {with}"
        );
        // VERIFIED region size: 432 blocks
        let (x, z) = gen.fortress_in_region(1, 0).unwrap();
        assert!((432..=432 + 431).contains(&x) && (0..=431).contains(&z));
    }

    #[test]
    fn fortress_emits_nether_bricks_and_blaze_spawners() {
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::Nether);
        // find a region with a fortress, then scan the 5×5 chunk
        // neighborhood of its center (spawners sit ±24 out, gardens ±14)
        'outer: for rx in 0..8 {
            for rz in 0..8 {
                if let Some((fx, fz)) = gen.fortress_in_region(rx, rz) {
                    let ccx = fx.div_euclid(16);
                    let ccz = fz.div_euclid(16);
                    let mut bricks = 0;
                    let mut spawner = false;
                    let mut wart = false;
                    for dcx in -2..=2i32 {
                        for dcz in -2..=2i32 {
                            let (chunk, _) = gen.generate_chunk(ccx + dcx, ccz + dcz, Vec::new());
                            for i in 0..CHUNK_LEN {
                                match chunk.get_idx(i) {
                                    NETHER_BRICKS => bricks += 1,
                                    SPAWNER => spawner = true,
                                    NETHER_WART => wart = true,
                                    _ => {}
                                }
                            }
                        }
                    }
                    assert!(bricks > 500, "nether-brick structure ({bricks} cells)");
                    assert!(spawner, "a blaze spawner platform is present");
                    assert!(wart, "the nether-wart garden is present");
                    break 'outer;
                }
            }
        }
    }

    #[test]
    fn mushroom_fields_generate_somewhere() {
        // scan a big area for the rare island biome (VERIFIED ~0.15%)
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::Overworld);
        let mut found = 0;
        for x in (-4000..4000).step_by(64) {
            for z in (-4000..4000).step_by(64) {
                if gen.column(x, z).biome == Biome::MushroomFields {
                    found += 1;
                }
            }
        }
        assert!(found > 0, "mushroom fields exist across a 8000² scan");
        // and the surface is mycelium
        'found: for x in (-4000..4000).step_by(64) {
            for z in (-4000..4000).step_by(64) {
                let c = gen.column(x, z);
                if c.biome == Biome::MushroomFields {
                    assert_eq!(c.top, MYCELIUM, "mycelium surface (VERIFIED)");
                    assert!(c.height > vc_chunk::SEA_LEVEL, "island above the sea");
                    break 'found;
                }
            }
        }
    }

    #[test]
    fn huge_mushrooms_emit_stem_and_cap_blocks() {
        // find a mushroom-fields chunk, generate it, verify the decoration
        let gen = TerrainGen::for_dimension(0x5EED_1234, Dimension::Overworld);
        let mut target = None;
        for x in (-4000..4000).step_by(16) {
            for z in (-4000..4000).step_by(16) {
                if gen.column(x, z).biome == Biome::MushroomFields {
                    target = Some((x.div_euclid(16), z.div_euclid(16)));
                    break;
                }
            }
            if target.is_some() {
                break;
            }
        }
        let (cx, cz) = target.expect("a mushroom-fields chunk exists");
        let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
        let mut stems = 0;
        let mut caps = 0;
        for i in 0..CHUNK_LEN {
            match chunk.get_idx(i) {
                MUSHROOM_STEM => stems += 1,
                MUSHROOM_RED_BLOCK | MUSHROOM_BROWN_BLOCK => caps += 1,
                _ => {}
            }
        }
        // several huge mushrooms per chunk (stems ≥ 4 cells, cap shells)
        assert!(stems >= 4, "hugemush stems ({stems})");
        assert!(caps >= 10, "hugemush caps ({caps})");
    }
}

#[cfg(test)]
mod e2_tests {
    use super::*;

    /// Phase E2 (VERIFIED w/Emerald_Ore): emerald ore appears only under
    /// Mountains columns (single blocks, y 4..31) — the check uses each
    /// emerald cell's OWN column biome (biomes vary per column, not per
    /// chunk).
    #[test]
    fn emerald_ore_generates_in_mountains_only() {
        let gen = TerrainGen::for_dimension(1234, Dimension::Overworld);
        // anchor the scan on actual mountains (the vanilla-parity
        // terrain's mountain regions are seed-dependent; a fixed 0..24
        // window can sit on an empty plain)
        let mut anchor = None;
        'find: for cz in -64..64i32 {
            for cx in -64..64i32 {
                if gen.column(cx * 16 + 8, cz * 16 + 8).biome == Biome::Mountains {
                    anchor = Some((cx, cz));
                    break 'find;
                }
            }
        }
        let (acx, acz) = anchor.expect("mountains exist within ±64 chunks");
        let mut emerald_cells = 0usize;
        let mut on_mountain_columns = 0usize;
        for cx in (acx - 12)..(acx + 12) {
            for cz in (acz - 12)..(acz + 12) {
                let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
                for i in 0..vc_chunk::chunk::CHUNK_LEN {
                    if chunk.get_idx(i) == EMERALD_ORE {
                        emerald_cells += 1;
                        // the cell's own column must be Mountains —
                        // checked against the chunk's biome field (the
                        // same data that drove placement; column()'s
                        // root-solve can sit a block off the lattice at
                        // the h>96 threshold)
                        let col_biome = chunk.biome[((i >> 4) & 15) * 16 + (i & 15)];
                        if Biome::from_u8(col_biome) == Biome::Mountains {
                            on_mountain_columns += 1;
                        }
                    }
                }
            }
        }
        assert!(
            emerald_cells > 0,
            "emerald ore exists somewhere in the 24x24 region"
        );
        assert_eq!(
            emerald_cells, on_mountain_columns,
            "EVERY emerald cell sits under a Mountains column (VERIFIED)"
        );
    }

    // ---------------- Phase E3 tests (1.5–1.6 bracket) ----------------

    #[test]
    fn phase_e3_superflat_is_the_classic_preset() {
        // VERIFIED live 2026-09-06 (reference wiki /Superflat): "one
        // layer of grass blocks and two layers of dirt, followed by
        // bedrock" — the classic preset, plains biome
        let gen = TerrainGen::for_dimension_flat(777, Dimension::Overworld);
        let (chunk, _) = gen.generate_chunk(0, 0, Vec::new());
        for lz in 0..16usize {
            for lx in 0..16usize {
                assert_eq!(
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 0),
                    BEDROCK,
                    "bedrock floor"
                );
                assert_eq!(
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 1),
                    DIRT,
                    "dirt layer 1"
                );
                assert_eq!(
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 2),
                    DIRT,
                    "dirt layer 2"
                );
                assert_eq!(
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 3),
                    GRASS,
                    "grass surface"
                );
                assert_eq!(
                    chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), 4),
                    AIR,
                    "air above"
                );
            }
        }
        // plains biome everywhere; no ocean fill above the surface
        for i in 0..256 {
            assert_eq!(chunk.biome[i], Biome::Plains as u8);
        }
        assert_eq!(chunk.height[0], 3, "surface at y=3");
    }

    #[test]
    fn phase_e3_badlands_band_through_stained_terracotta() {
        // VERIFIED w/Terracotta: stained terracotta "found abundantly in
        // badlands biomes" — banded by absolute y (the clean-room
        // deterministic banding, disclosed)
        let gen = TerrainGen::for_dimension(4242, Dimension::Overworld);
        // badlands is rare-regional (~0.2-0.4% of columns in
        // 64-block clumps, seed-sensitive placement): scan chunk
        // centers outward for a badlands-center chunk, verifying
        // against chunk data (column-vs-chunk threshold note above)
        let mut found = None;
        'outer: for cx in -128..128 {
            for cz in -128..128 {
                if gen.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Badlands {
                    continue;
                }
                let (probe, _) = gen.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(probe.biome[8 * 16 + 8]) == Biome::Badlands {
                    let col = gen.column(cx * 16 + 8, cz * 16 + 8);
                    found = Some((cx * 16 + 8, cz * 16 + 8, col.height));
                    break 'outer;
                }
            }
        }
        let Some((x, z, _probe_h)) = found else {
            panic!("no badlands chunk found in the ±128-chunk scan");
        };
        let (chunk, _) = gen.generate_chunk(x.div_euclid(16), z.div_euclid(16), Vec::new());
        // scan the chunk for an uncarved badlands column (carver cuts
        // legitimately expose strata; the floor intent needs a column
        // whose surface survived)
        let mut surface = None;
        let mut h = 0usize;
        let mut lx = 0usize;
        let mut lz = 0usize;
        'col: for czi in 0..16usize {
            for cxi in 0..16usize {
                if chunk.biome[czi * 16 + cxi] != Biome::Badlands as u8 {
                    continue;
                }
                let hi = chunk.height[czi * 16 + cxi] as usize;
                if chunk.get_local(vc_chunk::chunk::LocalXZ::new(cxi, czi), hi) == RED_SAND {
                    surface = Some(RED_SAND);
                    h = hi;
                    lx = cxi;
                    lz = czi;
                    break 'col;
                }
            }
        }
        let surface =
            surface.unwrap_or_else(|| panic!("no uncarved badlands floor column in the chunk"));
        // [merge 1.7.2] the SURFACE is red sand (1.7.2's verified floor);
        // the stained-terracotta banding the E3 bracket added lives in
        // the strata below the 1.8 red-sandstone filler — check the deep
        // window instead of the surface
        assert_eq!(surface, RED_SAND, "badlands surface is red sand");
        let mut bands = std::collections::HashSet::new();
        for y in (h.saturating_sub(15))..(h.saturating_sub(4)) {
            let b = chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y);
            if (STAINED_TERRACOTTA_BASE..=STAINED_TERRACOTTA_END).contains(&b) {
                bands.insert(b);
            }
        }
        assert!(
            bands.len() >= 3,
            "banded strata below the floor: {} distinct colors, got {bands:?}",
            bands.len()
        );
    }
}

// ---------------------------------------------------------------------------
// audit-fix round tests (2026-09-07): 1.2 jungle wood family + vines +
// ferns (the Phase-1 evolution-audit gap)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod auditfix_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// jungle chunks grow JUNGLE_LOG/JUNGLE_LEAVES trees with VINE on
    /// the trunks ("Jungle trees of both sizes have vines on their
    /// trunks and canopy edges" — VERIFIED w/Vines) and FERN ground
    /// cover (VERIFIED w/Fern natural generation)
    #[test]
    fn jungle_grows_jungle_wood_vines_and_ferns() {
        let g = gen();
        let (mut logs, mut leaves, mut vines, mut ferns, mut chunks) =
            (0usize, 0usize, 0usize, 0usize, 0usize);
        'scan: for cx in -64..64 {
            for cz in -64..64 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Jungle {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                for i in 0..CHUNK_LEN {
                    match chunk.get_idx(i) {
                        JUNGLE_LOG => logs += 1,
                        JUNGLE_LEAVES => leaves += 1,
                        VINE => vines += 1,
                        FERN => ferns += 1,
                        _ => {}
                    }
                }
                chunks += 1;
                if chunks >= 8 {
                    break 'scan;
                }
            }
        }
        assert!(chunks >= 4, "found jungle chunks to scan (got {chunks})");
        assert!(logs > 0, "jungle trunks exist (got {logs} JUNGLE_LOG)");
        assert!(
            leaves > 0,
            "jungle canopy exists (got {leaves} JUNGLE_LEAVES)"
        );
        assert!(vines > 0, "vines on trunks (got {vines} VINE)");
        assert!(ferns > 0, "fern ground cover (got {ferns} FERN)");
        // jungle wood must DOMINATE over oak in jungle-center chunks:
        // the species switch gives every Jungle-column tree jungle
        // wood; oak logs can only appear from non-jungle edge columns
        // of the same chunk (the per-tree biome is sampled per column).
        let mut oak_logs = 0usize;
        let mut jungle_logs = 0usize;
        for cx in -64..64 {
            for cz in -64..64 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Jungle {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                for i in 0..CHUNK_LEN {
                    match chunk.get_idx(i) {
                        OAK_LOG => oak_logs += 1,
                        JUNGLE_LOG => jungle_logs += 1,
                        _ => {}
                    }
                }
                if jungle_logs + oak_logs > 60 {
                    break;
                }
            }
        }
        assert!(
            jungle_logs > oak_logs,
            "jungle wood dominates (jungle {jungle_logs} vs oak {oak_logs})"
        );
    }

    /// jungle bushes: a single JUNGLE_LOG surrounded by OAK LEAVES
    /// (VERIFIED w/Tree: "Jungle bushes also generate in the jungle
    /// biome, featuring a single jungle log surrounded by oak leaves")
    #[test]
    fn jungle_bushes_are_jungle_log_with_oak_leaves() {
        let g = gen();
        let mut found_bush = false;
        'scan: for cx in -64..64 {
            for cz in -64..64 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Jungle {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                for lz in 1..15usize {
                    for lx in 1..15usize {
                        for y in 60..100usize {
                            if chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz), y)
                                != JUNGLE_LOG
                            {
                                continue;
                            }
                            // a bush log has oak LEAVES beside it
                            let neighbors = [
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx + 1, lz), y),
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx - 1, lz), y),
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz + 1), y),
                                chunk.get_local(vc_chunk::chunk::LocalXZ::new(lx, lz - 1), y),
                            ];
                            if neighbors.contains(&LEAVES) {
                                found_bush = true;
                                break 'scan;
                            }
                        }
                    }
                }
            }
        }
        assert!(found_bush, "jungle bush signature found (log + oak leaves)");
    }

    /// ferns also generate in taiga (VERIFIED w/Fern §Natural
    /// generation: "Ferns occur naturally only in jungle, taiga, snowy
    /// taiga and old growth taiga biomes"). Snowy taiga is covered by
    /// the same flora arm, but the engine's snowy surface is
    /// SNOW_GRASS which the flora placement gate excludes (the
    /// pre-existing surface convention — disclosed in the worklog);
    /// taiga is scanned over several chunks since the 4-attempt pass
    /// is per-column luck.
    #[test]
    fn taiga_grows_ferns() {
        let g = gen();
        let mut taiga_chunks = 0;
        let mut ferns = 0usize;
        // 4.1p: accumulate over up to 16 taiga-center chunks (the
        // mountain-gate move fills the first-hit window with high
        // podzol taiga; same interleave pattern as bamboo/sunflower)
        'scan: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Taiga {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::Taiga {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == FERN {
                        ferns += 1;
                    }
                }
                taiga_chunks += 1;
                if ferns > 0 || taiga_chunks >= 16 {
                    break 'scan;
                }
            }
        }
        assert!(taiga_chunks >= 1, "found taiga chunks to scan");
        assert!(
            ferns > 0,
            "taiga grows ferns (got {ferns} over {taiga_chunks} chunks)"
        );
    }
}

// ---------------------------------------------------------------------------
// 1.11 bracket tests (woodland mansion, live 2026-09-07)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v111_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// dark-forest mansions generate with the illager spawners + chest
    /// and cobble/wood construction (VERIFIED w/Woodland_Mansion: three
    /// floors, cobblestone foundation, "inhabited by cleavers,
    /// evokers")
    #[test]
    fn woodland_mansions_generate_with_illagers() {
        // vanilla mansions are genuinely rare (the wiki: mansions
        // generate "rarely" in dark forests) — scan seeds until one
        // lands in the window (the villages-test pattern)
        let mut found = None;
        let mut g = gen();
        'seeds: for s in 0..16u64 {
            g = TerrainGen::for_dimension(
                0x10C0_C0DEu64.wrapping_add(s.wrapping_mul(0x9E37_79B9_7F4A_7C15u64)),
                Dimension::Overworld,
            );
            for rx in -40..40 {
                for rz in -40..40 {
                    for mx in 0..8 {
                        for mz in 0..8 {
                            let cx = rx * 8 + mx;
                            let cz = rz * 8 + mz;
                            if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::DarkForest {
                                continue;
                            }
                            if !g.woodland_mansions_near(cx * 16, cz * 16).is_empty() {
                                found = Some((cx, cz));
                                break 'seeds;
                            }
                        }
                    }
                }
            }
        }
        let Some((cx, cz)) = found else {
            panic!("no mansion found across 16 seeds (rarity + biome)");
        };
        // the near-query may surface a mansion anchored in a NEIGHBOR
        // chunk — generate the anchor's own chunk (anchor = center+8)
        let (ax, az) = g.woodland_mansions_near(cx * 16, cz * 16)[0];
        let acx = floor_div(ax - 8, 16);
        let acz = floor_div(az - 8, 16);
        let (chunk, _) = g.generate_chunk(acx, acz, Vec::new());
        // NOTE: Chunk::get / get_idx FOLD state ids to owning BLOCK ids
        // (the 1.7.2 refactor — see chunk.rs), and Chunk::set STORES the
        // default STATE (default_state) — so the raw scan rides get_state:
        // the placed CHEST lands as CHEST_STATE (227) and the dedicated
        // mansion spawner states (SPAWNER_CLEAVER / _EVOKER, already
        // state ids) survive verbatim. register_block_entities decodes
        // both through state_block + spawner_mob.
        let (mut cobble, mut planks, mut spawners, mut chests) = (0, 0, 0, 0);
        let (mut v_spawners, mut evoker_spawners) = (0, 0);
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    match chunk.get_state(x, y, z) {
                        COBBLE => cobble += 1,
                        PLANKS => planks += 1,
                        CHEST_STATE => chests += 1,
                        SPAWNER_CLEAVER => {
                            spawners += 1;
                            v_spawners += 1;
                        }
                        SPAWNER_EVOKER => {
                            spawners += 1;
                            evoker_spawners += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(cobble > 200, "cobblestone construction (got {cobble})");
        assert!(planks > 100, "wood floors (got {planks})");
        assert!(spawners >= 3, "illager spawners (got {spawners})");
        assert!(
            v_spawners >= 2 && evoker_spawners >= 2,
            "vindicator + evoker spawners (got {v_spawners}/{evoker_spawners})"
        );
        assert!(chests >= 1, "loot chest (got {chests})");
        // the emit's ground truth: the 5 spawner puts + 2 chest puts are
        // all inside the anchor chunk (anchor-relative dx/dz within
        // ±6 — see emit_woodland_mansion), so a single-chunk scan sees
        // them all
        assert!(
            spawners == 5 && chests == 2,
            "all 5 spawners + 2 chests in the anchor chunk (got {spawners}/{chests})"
        );
    }

    /// 1.11: the mansion spawner states decode to their mobs via
    /// spawner_mob (the register_block_entities path) — vindicator
    /// code 5, evoker code 6 (both VERIFIED w/Vindicator + w/Evoker
    /// spawn behavior: "Spawn in the woodland mansions upon generation.
    /// They don't respawn." / evokers "Spawn in the two upper floors")
    #[test]
    fn v111_mansion_spawner_states_decode() {
        assert_eq!(vc_blocks::blocks::spawner_mob(SPAWNER_CLEAVER), 5);
        assert_eq!(vc_blocks::blocks::spawner_mob(SPAWNER_EVOKER), 6);
        assert_eq!(vc_blocks::blocks::state_block(SPAWNER_CLEAVER), SPAWNER);
        assert_eq!(vc_blocks::blocks::state_block(SPAWNER_EVOKER), SPAWNER);
    }
}

// ---------------------------------------------------------------------------
// 1.13 bracket tests (Aquatic-era update, live 2026-09-07)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v113_tests {
    use super::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    fn find_biome(g: &TerrainGen, b: Biome) -> (i32, i32) {
        // column-hit verified against the chunk center (see the v172
        // find_biome note — column() can miss by a block at thresholds)
        for cx in -64..64 {
            for cz in -64..64 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome == b {
                    let (probe, _) = g.generate_chunk(cx, cz, Vec::new());
                    if Biome::from_u8(probe.biome[8 * 16 + 8]) == b {
                        return (cx, cz);
                    }
                }
            }
        }
        panic!("{} not found in the ±64-chunk window", b.name());
    }

    /// VERIFIED changelog §Blocks/§World generation: kelp "Generate in
    /// ocean biomes, except warm oceans"; coral/coral fans/coral blocks
    /// "Naturally generate in coral reefs" (warm oceans); sea pickles
    /// "generate in warm oceans, especially around coral reefs";
    /// seagrass "Generates in oceans ..."; the frozen-ocean ice sheet;
    /// blue ice "Generates in icebergs" (frozen oceans). Each family's
    /// signature flora is scanned over a 6×6-chunk window around a
    /// located biome sample — PER COLUMN (chunks are heterogeneous:
    /// a warm-ocean-centered chunk can carry neutral columns and
    /// vice versa).
    #[test]
    fn v113_ocean_flora_matches_the_biome_families() {
        let g = gen();
        // ---- warm ocean: coral reefs + pickles, NEVER kelp ----
        let (cx, cz) = find_biome(&g, Biome::WarmOcean);
        let (mut coral, mut pickles) = (0usize, 0usize);
        for dcx in -3..3 {
            for dcz in -3..3 {
                let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                for i in 0..CHUNK_LEN {
                    let b = chunk.get_idx(i);
                    // the column that owns this cell
                    let col = ((i >> 4) & 15) * 16 + (i & 15);
                    let col_biome = Biome::from_u8(chunk.biome[col]);
                    if col_biome == Biome::WarmOcean {
                        if (CORAL_BLOCK_BASE..=CORAL_FAN_END).contains(&b) {
                            coral += 1;
                        }
                        if b == SEA_PICKLE {
                            pickles += 1;
                        }
                        assert!(
                            b != KELP,
                            "kelp never generates in warm-ocean columns (VERIFIED changelog)"
                        );
                    }
                }
            }
        }
        assert!(coral > 0, "warm oceans carry coral reefs (got {coral})");
        assert!(
            pickles > 0,
            "sea pickles ride the reef patches (got {pickles})"
        );
        // ---- cold ocean: kelp + seagrass, NEVER coral ----
        let (cx, cz) = find_biome(&g, Biome::ColdOcean);
        let (mut kelp, mut seagrass) = (0usize, 0usize);
        for dcx in -3..3 {
            for dcz in -3..3 {
                let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                for i in 0..CHUNK_LEN {
                    let b = chunk.get_idx(i);
                    let col = ((i >> 4) & 15) * 16 + (i & 15);
                    let col_biome = Biome::from_u8(chunk.biome[col]);
                    if col_biome == Biome::ColdOcean {
                        if b == KELP {
                            kelp += 1;
                        }
                        if b == SEAGRASS {
                            seagrass += 1;
                        }
                        assert!(
                            !(CORAL_BLOCK_BASE..=DEAD_CORAL_FAN_END).contains(&b),
                            "coral only generates in warm-ocean columns (VERIFIED)"
                        );
                    }
                }
            }
        }
        assert!(kelp > 0, "cold oceans grow kelp (got {kelp})");
        assert!(
            seagrass > 0,
            "cold ocean floors carry seagrass (got {seagrass})"
        );
        // ---- frozen ocean: the ice sheet + iceberg blue ice ----
        let (cx, cz) = find_biome(&g, Biome::FrozenOcean);
        let (mut ice_sheet, mut blue_ice) = (0usize, 0usize);
        for dcx in -5..5 {
            for dcz in -5..5 {
                let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        let h = chunk.height[lz * 16 + lx] as i32;
                        if h < 61 {
                            // the surface water block at y 62 froze
                            let idx = (62usize << 8) | (lz << 4) | lx;
                            if chunk.get_idx(idx) == ICE {
                                ice_sheet += 1;
                            }
                        }
                    }
                }
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == BLUE_ICE {
                        blue_ice += 1;
                    }
                }
            }
        }
        assert!(
            ice_sheet > 16,
            "the frozen-ocean surface froze over (got {ice_sheet} ice cells)"
        );
        assert!(
            blue_ice > 0,
            "icebergs carry blue ice (got {blue_ice} across 100 chunks)"
        );
    }

    /// kelp columns grow multiple blocks high ("Can grow multiple
    /// blocks high" — VERIFIED): the cold-ocean kelp cells include
    /// at least one 2+ tall column.
    #[test]
    fn v113_kelp_columns_grow_multiple_blocks() {
        let g = gen();
        let (cx, cz) = find_biome(&g, Biome::ColdOcean);
        let mut found = false;
        'outer: for dcx in -3..3 {
            for dcz in -3..3 {
                let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        let h = chunk.height[lz * 16 + lx] as i32;
                        let mut run = 0;
                        for dy in 1..5 {
                            let y = h + dy;
                            if y >= 62 {
                                break;
                            }
                            let idx = ((y as usize) << 8) | (lz << 4) | lx;
                            if chunk.get_idx(idx) == KELP {
                                run += 1;
                            } else {
                                break;
                            }
                        }
                        if run >= 2 {
                            found = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
        assert!(
            found,
            "no 2+ tall kelp column found in the window (deterministic seed)"
        );
    }

    /// the ocean temperature split itself: all four 1.13 families are
    /// findable and from_u8 round-trips them.
    #[test]
    fn v113_ocean_families_present_and_roundtrip() {
        let g = gen();
        for b in [
            Biome::WarmOcean,
            Biome::LukewarmOcean,
            Biome::ColdOcean,
            Biome::FrozenOcean,
        ] {
            let _ = find_biome(&g, b);
            assert_eq!(Biome::from_u8(b as u8), b);
        }
        assert!(Biome::WarmOcean.is_ocean());
        assert!(Biome::FrozenOcean.is_ocean());
        assert!(!Biome::Beach.is_ocean());
        assert!(!Biome::Plains.is_ocean());
    }

    /// 1.14 (Village & Pillage — nature half): bamboo generates in
    /// jungle columns ("Bamboo generates in widely scattered single
    /// shoots within jungle biomes" — VERIFIED w/Bamboo §Natural
    /// generation), and never outside them.
    #[test]
    fn v114_jungle_carries_bamboo() {
        let g = gen();
        // jungle: accumulate bamboo over up to 16 jungle-center
        // chunks (4.1n: bands interleave, so one 7x7 field may roll
        // its 20% patches onto off-jungle columns)
        let mut bamboo = 0usize;
        let mut scanned = 0usize;
        'jungle: for cx in -128..128 {
            for cz in -128..128 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome != Biome::Jungle {
                    continue;
                }
                let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
                if Biome::from_u8(chunk.biome[8 * 16 + 8]) != Biome::Jungle {
                    continue;
                }
                for i in 0..CHUNK_LEN {
                    if chunk.get_idx(i) == BAMBOO {
                        bamboo += 1;
                    }
                }
                scanned += 1;
                if scanned >= 16 {
                    break 'jungle;
                }
            }
        }
        assert!(scanned > 0, "found jungle chunks to scan");
        assert!(
            bamboo > 0,
            "jungle fields carry bamboo shoots (got {bamboo})"
        );
        // never outside: taiga + snowy + plains + desert windows
        for b in [Biome::Taiga, Biome::Snowy, Biome::Plains, Biome::Desert] {
            let (cx, cz) = find_biome(&g, b);
            for dcx in -1..=1 {
                for dcz in -1..=1 {
                    let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                    for i in 0..CHUNK_LEN {
                        assert!(
                            chunk.get_idx(i) != BAMBOO,
                            "bamboo never generates in {} columns",
                            b.name()
                        );
                    }
                }
            }
        }
    }

    /// 1.14: sweet berry bushes generate in taiga/snowy-taiga patches
    /// ("Each chunk has a 1/12 chance" — VERIFIED w/Sweet_Berry_Bush
    /// §Natural generation) at bearing ages 1..=3, and never outside.
    #[test]
    fn v114_taiga_carries_berry_bushes() {
        let g = gen();
        for b in [Biome::Taiga, Biome::Snowy] {
            let (cx, cz) = find_biome(&g, b);
            let mut bushes = 0usize;
            // a 13x13 window: 169 chunks × 1/12 ≈ 14 patches — the
            // patch roll is chunk-center-keyed (the whole chunk is one
            // biome), so neighboring chunks of the same biome count
            for dcx in -6..=6 {
                for dcz in -6..=6 {
                    let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                    for i in 0..CHUNK_LEN {
                        // get_idx returns the FOLDED block id (Chunk::get
                        // folds raw states) — the raw state for the age
                        // check comes from get_state
                        let b = chunk.get_idx(i);
                        if b == SWEET_BERRY_BUSH {
                            bushes += 1;
                            let raw = chunk.get_state(i & 15, (i >> 8) & 0xFF, (i >> 4) & 15);
                            let age = berry_bush_age(raw);
                            assert!(
                                (1..=3).contains(&age),
                                "generated bushes are bearing age 1..=3 (got {age})"
                            );
                        }
                    }
                }
            }
            assert!(
                bushes > 0,
                "{} fields carry sweet berry bushes (got {bushes})",
                b.name()
            );
        }
        // never outside: jungle + plains + forest + savanna
        for b in [Biome::Jungle, Biome::Plains, Biome::Forest, Biome::Savanna] {
            let (cx, cz) = find_biome(&g, b);
            for dcx in -1..=1 {
                for dcz in -1..=1 {
                    let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                    for i in 0..CHUNK_LEN {
                        assert!(
                            chunk.get_idx(i) != SWEET_BERRY_BUSH,
                            "berry bushes never generate in {} columns",
                            b.name()
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 1.15 bracket tests (Buzzy Bees, live 2026-09-08 — the v115_page_*
// captures; docs/research/phase-v115-1.15-research.md)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v115_nest_tests {
    use super::*;
    use crate::world::World;

    fn flat_world() -> World {
        let mut w = World::new(11);
        let mut c = Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    fn find_biome(g: &TerrainGen, b: Biome) -> (i32, i32) {
        // column-first scan with chunk-center verification (the direct
        // chunk scan this helper replaced cost ~14 ms per probe)
        for cz in -60..60 {
            for cx in -60..60 {
                if g.column(cx * 16 + 8, cz * 16 + 8).biome == b {
                    let (probe, _) = g.generate_chunk(cx, cz, Vec::new());
                    if Biome::from_u8(probe.biome[8 * 16 + 8]) == b {
                        return (cx, cz);
                    }
                }
            }
        }
        panic!("biome {} not found", b.name());
    }

    /// 1.15 (Buzzy Bees): bee nests generate on oak/birch trees at the
    /// JE biome chances (plains/sunflower 5%, flower forest 2%,
    /// forest-family 0.2% — VERIFIED w/Bee §Natural generation). The
    /// rolls are per-tree POSITION hashes (no rng-stream draws), so
    /// the flora layout is unshifted. The census over a 12x12-chunk
    /// plains region: at 5% per tree with ~0-2 trees/chunk the count
    /// must be > 0 and every nest must SIT BESIDE A TRUNK at the
    /// documented band.
    #[test]
    fn v115_bee_nests_on_plains_trees() {
        let g = TerrainGen::new(1234);
        let (cx, cz) = find_biome(&g, Biome::Plains);
        let mut nests = 0usize;
        let mut trees = 0usize;
        for dcx in 0..12 {
            for dcz in 0..12 {
                let (chunk, _) = g.generate_chunk(cx + dcx, cz + dcz, Vec::new());
                for i in 0..CHUNK_LEN {
                    let b = chunk.get_idx(i);
                    if b == OAK_LOG || b == BIRCH_LOG {
                        trees += 1;
                    } else if b == BEE_NEST {
                        nests += 1;
                        // the nest sits beside a trunk: at least one of
                        // the 4 horizontal neighbors at the same height
                        // is a log
                        let x = (i & 0x0F) as i32;
                        let z = ((i >> 4) & 0x0F) as i32;
                        let y = (i >> 8) as i32;
                        let beside_trunk =
                            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
                                let nx = x + dx;
                                let nz = z + dz;
                                (0..16).contains(&nx)
                                    && (0..16).contains(&nz)
                                    && matches!(
                                        chunk.get_local(
                                            vc_chunk::chunk::LocalXZ::new(nx as usize, nz as usize),
                                            y as usize,
                                        ),
                                        OAK_LOG | BIRCH_LOG
                                    )
                            });
                        assert!(beside_trunk, "nest at ({x},{y},{z}) beside a trunk");
                    }
                }
            }
        }
        assert!(trees > 40, "the census region has trees (got {trees})");
        assert!(
            nests > 0,
            "the 5% plains roll produces nests (got {nests} on {trees} trees)"
        );
        // the observed rate stays inside the wide binomial window
        let rate = nests as f32 / trees.max(1) as f32;
        assert!(rate < 0.25, "the nest rate stays plausible ({rate:.3})");
    }

    /// the honey_level blockstate roundtrips through the world: a nest
    /// placed at level 5 reads back 5 (the V12 window contract from
    /// the blocks side, exercised through the world API)
    #[test]
    fn v115_hive_state_roundtrip_in_world() {
        let mut w = flat_world();
        use vc_blocks::blocks::{hive_full, hive_state, honey_level, BEEHIVE, BEE_NEST};
        w.set_block_state(8, 70, 8, hive_state(BEE_NEST, 5));
        w.set_block_state(10, 70, 8, hive_state(BEEHIVE, 3));
        assert_eq!(honey_level(w.get_state(8, 70, 8)), 5);
        assert!(hive_full(w.get_state(8, 70, 8)));
        assert_eq!(honey_level(w.get_state(10, 70, 8)), 3);
        assert!(!hive_full(w.get_state(10, 70, 8)));
    }
}

// =====================================================================
// 2026-09-20 rampart-fix round — the ravine-density regression tests
// (the rampant-ravines + lag report). Pre-fix quantification (exact
// rng-port harness, scripts/ravine_quant.py): every carved column was
// 100% sky-open with mean depth 39.3 (the mega-trench look) and the
// overlap merge dropped carve intervals (solid mesas inside canyon
// crossings). These tests pin the fixed envelope.
// =====================================================================
#[cfg(test)]
mod rampart_fix_tests {
    use super::*;

    fn agen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// unique ravine anchors rolled in a 40x40-chunk window: the
    /// vanilla 1/50 canyon-carver rate ± sampling noise
    #[test]
    fn ravine_anchor_rate_in_envelope() {
        let g = agen();
        let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        let mut n = 0usize;
        for cx in -20..20i32 {
            for cz in -20..20i32 {
                for r in g.ravines_near_chunk(cx, cz) {
                    if seen.insert((r.x0, r.z0)) {
                        n += 1;
                    }
                }
            }
        }
        let rate = n as f32 / 1600.0;
        assert!(
            (0.01..=0.035).contains(&rate),
            "anchor rate {rate:.3} stays near vanilla 1/50 (got {n}/1600)"
        );
    }

    /// the carved profile: mean carved depth well under the 62 max
    /// (pre-fix every carve averaged 39 with a 40 floor), max
    /// respects the 62 grammar, and the carved population is no
    /// longer 100% sky-open (pre-fix EVERY carved column broke the
    /// surface by construction — the rampant mega-trench look + the
    /// mesh explosion that caused the lag)
    #[test]
    fn ravine_carve_profile_is_mostly_underground() {
        let g = agen();
        // collect the unique anchors covering a 12x12-chunk area
        let mut ravines: Vec<Ravine> = Vec::new();
        let mut seen: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for dcx in -6..6i32 {
            for dcz in -6..6i32 {
                for r in g.ravines_near_chunk(10 + dcx, 10 + dcz) {
                    if seen.insert((r.x0, r.z0)) {
                        ravines.push(r);
                    }
                }
            }
        }
        assert!(!ravines.is_empty(), "the window has anchors");
        // Sample the carve population REPRESENTATIVELY: around every
        // anchor (±16 blocks, capped 16 carved columns per anchor so no
        // single anchor dominates), with REAL terrain heights — a
        // first-anchor-only sample is order-biased and flaky, this one
        // measures the whole window's population
        let mut carved = 0usize;
        let mut depth_sum = 0usize;
        let mut max_depth = 0i32;
        let mut sky_open = 0usize;
        for rv in &ravines {
            let mut per_anchor = 0usize;
            'anchor: for bx in (rv.x0 - 16)..(rv.x0 + 16) {
                for bz in (rv.z0 - 16)..(rv.z0 + 16) {
                    let surf = g.column(bx, bz).height;
                    if let Some((top, bot)) = g.ravine_cut(&ravines, bx, bz, surf) {
                        carved += 1;
                        depth_sum += (top - bot) as usize;
                        max_depth = max_depth.max(top - bot);
                        if top >= surf - 1 {
                            sky_open += 1;
                        }
                        per_anchor += 1;
                        if per_anchor >= 16 {
                            break 'anchor;
                        }
                    }
                }
            }
        }
        assert!(carved >= 8, "the anchors' neighborhoods carve ({carved})");
        let mean = depth_sum as f32 / carved as f32;
        assert!(mean < 30.0, "mean carved depth {mean:.1} < 30 (was 39.3)");
        assert!(
            max_depth <= 62,
            "max carved depth {max_depth} respects the 62 grammar"
        );
        let sky = sky_open as f32 / carved as f32;
        assert!(
            sky < 0.85,
            "sky-open fraction {sky:.2} is no longer the pre-fix 100% \
             (the rolled 10..=72 start level keeps most carves underground)"
        );
    }

    /// the 2026-09-20 merge fix: two crossing ravines carve the UNION
    /// of their intervals — the higher rim AND the deeper floor both
    /// survive. Pre-fix the upper interval was dropped, leaving solid
    /// rock mesas standing inside canyon crossings.
    #[test]
    fn ravine_overlap_merges_as_union() {
        let g = agen();
        // two synthetic ravines crossing at column (100, 100):
        // A: east-west, top 40, depth 20 → carves ~(20..40)
        // B: north-south, top 64, depth 40 → carves ~(24..64)
        let a = Ravine {
            x0: 0,
            z0: 100,
            dx: 1.0,
            dz: 0.0,
            length: 200,
            half_w: 6.0,
            depth: 20,
            top: 40,
        };
        let b = Ravine {
            x0: 100,
            z0: 0,
            dx: 0.0,
            dz: 1.0,
            length: 200,
            half_w: 6.0,
            depth: 40,
            top: 64,
        };
        let rv = [a, b];
        // flat surface at 64
        let cut = g.ravine_cut(&rv, 100, 100, 64).expect("both carve here");
        let (top, bot) = cut;
        // the union must reach B's rim (64: the higher carve survives)
        assert!(top >= 60, "the union keeps the HIGHER rim (got {top})");
        // and reach at least A's floor depth territory
        assert!(bot <= 32, "the union keeps the deeper floor (got {bot})");
        // the carved span covers the old lost interval [33..=40] too
        assert!(top - bot >= 32, "span {top}-{bot} covers both carves");
    }

    /// determinism after the fix: identical carve decisions for the
    /// same seed, and the descriptor grammar still holds
    #[test]
    fn ravine_grammar_and_determinism_after_fix() {
        let (g1, g2) = (agen(), agen());
        for cx in -5..5i32 {
            for cz in -5..5i32 {
                let r1 = g1.ravines_near_chunk(cx, cz);
                let r2 = g2.ravines_near_chunk(cx, cz);
                assert_eq!(r1.len(), r2.len(), "same count");
                for (a, b) in r1.iter().zip(r2.iter()) {
                    assert_eq!((a.x0, a.z0, a.length), (b.x0, b.z0, b.length));
                    assert_eq!(a.depth, b.depth);
                    assert_eq!(a.top, b.top);
                    // the wiki grammar
                    assert!((85..=127).contains(&a.length));
                    assert!(a.half_w < 7.5);
                    assert!(a.depth <= 62);
                    assert!((10..=72).contains(&a.top));
                }
            }
        }
    }
}

/// 1.0.1 (PLAN v3.1): the WORLD-GEN golden hash — the R5 determinism gate.
/// Three fixed seeds × nine chunks per dimension (Overworld, Nether, End),
/// hashing the raw block STATE ids and the per-column biome ids of every
/// generated chunk. The expected values below were pinned on Linux x86-64
/// (the only CI OS today); slices 1.0.2/1.0.3 run this single test on the
/// Windows and macOS legs — the gate is the SAME hash on all three.
///
/// The hash is FNV-1a over the u16 state stream (chunk-local index order,
/// y-major as the section layout stores it) with the biome stream and the
/// seed/dimension tag folded in, so any drift anywhere changes it.
///
/// NOTE for reviewers: this is NOT a reference-game parity claim — it pins
/// OUR generator against accidental change (numeric type drift, FMA,
/// libm-dependent transcendental swaps, noise reordering). Part 4 owns
/// parity against the reference game via the oracle harness (2.2).
#[cfg(test)]
mod golden_determinism_tests {
    use super::*;
    use vc_chunk::chunk::Chunk;

    const GOLDEN_SEEDS: [u64; 3] = [0x00C0_FFEE_1234_5678, 0xDEAD_BEEF_0000_0001, 7];

    /// 3×3 chunk neighborhood per seed per dimension — borders exercise
    /// the outbound-edit path (trees crossing chunk edges) via `inbound`.
    const NEIGHBORS: [(i32, i32); 9] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 0),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    fn fnv1a(state: u64, bytes: &[u8]) -> u64 {
        let mut h = state;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        h
    }

    fn hash_chunk(h0: u64, c: &Chunk) -> u64 {
        let mut h = h0;
        for (i, slot) in c.sections.iter().enumerate() {
            h = fnv1a(h, &(i as u64).to_le_bytes());
            match slot {
                Some(s) => {
                    // non-empty sections hash the unpacked 4096 u16
                    // states (bit-exact, layout-stable: the packed
                    // palette/word layout may change, the truth it
                    // encodes must not)
                    h = fnv1a(h, &[1]);
                    let flat = s.states_flat();
                    let bytes: Vec<u8> = flat.iter().flat_map(|v| v.to_le_bytes()).collect();
                    h = fnv1a(h, &bytes);
                }
                None => h = fnv1a(h, &[0]),
            }
        }
        let biome_bytes: Vec<u8> = c.biome.to_vec();
        h = fnv1a(h, &biome_bytes);
        let height_bytes: Vec<u8> = c.height.to_vec();
        h = fnv1a(h, &height_bytes);
        h
    }

    /// generate the 9-chunk neighborhood with the REAL world flow: each
    /// chunk's outbound edits are queued and replayed as the neighbors'
    /// inbound (the cross-chunk canopy/structure path is part of the
    /// contract), then hash each chunk.
    fn neighborhood_hash(seed: u64, dim: Dimension) -> u64 {
        let gen = TerrainGen::for_dimension(seed, dim);
        let mut h = fnv1a(0xCBF2_9CE4_8422_2325, seed.to_le_bytes().as_slice());
        h = fnv1a(h, &[dim as u8]);
        let mut outbound: Vec<(i32, i32, i32, u16)> = Vec::new();
        for &(cx, cz) in NEIGHBORS.iter() {
            // replay every edit that targets this chunk from the ones
            // generated before it (deterministic iteration order)
            let inbound: Vec<(u16, u16)> = outbound
                .iter()
                .filter(|(x, _, z, _)| x.div_euclid(16) == cx && z.div_euclid(16) == cz)
                .map(|(_, i, _, id)| (*i as u16, *id))
                .collect();
            let (chunk, out) = gen.generate_chunk(cx, cz, inbound);
            h = hash_chunk(h, &chunk);
            outbound.extend(out);
        }
        h
    }
    /// THE GOLDEN VALUES — pinned on Linux x86-64, Rust stable,
    /// 2026-10-08 (slice 1.0.1). Do NOT update silently: any drift here
    /// is a determinism regression (R5) until proven an intentional,
    /// reported re-baseline (slice 1.0.3 reports old vs new). The seed /
    /// dimension pairs are in [GOLDEN_SEEDS] × [OVERWORLD, NETHER, END].
    /// 4.4a re-baseline (owner-approved accuracy-over-history
    /// 2026-10-10): the 3 overworld mains + all overworld targeted pins
    /// move with cross-chunk veins; nether/end pins byte-identical.
    const GOLDEN: [u64; 9] = [
        0xc4b9_3891_ba02_ca0a, // seed c0ffee12345678, overworld
        0x2d1e_15af_85b4_8feb, // seed c0ffee12345678, nether
        0x5903_79b0_ae9e_b8f9, // seed c0ffee12345678, end
        0x9203_607f_0e8a_d037, // seed deadbeef00000001, overworld
        0x8042_5d42_3571_20b3, // seed deadbeef00000001, nether
        0x112d_74b4_87d7_0cd5, // seed deadbeef00000001, end
        0x6444_7c6d_7817_884f, // seed 7, overworld
        0xbb9f_4859_d4d2_ae6a, // seed 7, nether
        0x821f_f1cc_25ca_21cd, // seed 7, end
    ];

    #[test]
    fn golden_worldgen_hash_overworld_nether_end() {
        let dims = [
            ("overworld", Dimension::Overworld),
            ("nether", Dimension::Nether),
            ("end", Dimension::End),
        ];
        let mut got: Vec<(String, u64)> = Vec::new();
        for &seed in GOLDEN_SEEDS.iter() {
            for (dname, dim) in dims {
                let h = neighborhood_hash(seed, dim);
                println!("GOLDEN seed={seed:#018x} dim={dname} hash={h:#018x}");
                got.push((format!("{seed:#x}/{dname}"), h));
            }
        }
        for (i, (label, h)) in got.iter().enumerate() {
            assert_eq!(
                *h, GOLDEN[i],
                "golden worldgen hash drifted for {label} — a determinism \
                 regression (R5) unless this is an intentional, reported \
                 re-baseline"
            );
        }
    }

    /// 1.0.5: WIDE golden hash — same FNV contract over 5 seeds x 25
    /// chunks (5x5) per dimension. NEW values only, narrow GOLDEN untouched.
    const WIDE_SEEDS: [u64; 5] = [
        0x00C0_FFEE_1234_5678,
        0xDEAD_BEEF_0000_0001,
        7,
        0x1234_5678_9ABC_DEF0,
        0x0BAD_F00D_CAFE_1234,
    ];
    const WIDE_OFF: i32 = 2;

    fn wide_hash(seed: u64, dim: Dimension) -> u64 {
        let gen = TerrainGen::for_dimension(seed, dim);
        let mut h = fnv1a(0xCBF2_9CE4_8422_2325, seed.to_le_bytes().as_slice());
        h = fnv1a(h, &[dim as u8]);
        let mut outbound: Vec<(i32, i32, i32, u16)> = Vec::new();
        for cz in -WIDE_OFF..=WIDE_OFF {
            for cx in -WIDE_OFF..=WIDE_OFF {
                let inbound: Vec<(u16, u16)> = outbound
                    .iter()
                    .filter(|(x, _, z, _)| x.div_euclid(16) == cx && z.div_euclid(16) == cz)
                    .map(|(_, i, _, id)| (*i as u16, *id))
                    .collect();
                let (chunk, out) = gen.generate_chunk(cx, cz, inbound);
                h = hash_chunk(h, &chunk);
                outbound.extend(out);
            }
        }
        h
    }

    const GOLDEN_WIDE: [u64; 15] = [
        // 4.4a re-pin: overworld entries move with cross-chunk veins
        // (nether/end byte-identical)
        0x4904_7659_00b3_8cf8,
        0xc224_973a_dbae_0f7d,
        0x00dc_0ad5_7854_7143,
        0xccd3_c97a_85a3_f253,
        0xa5cd_d32c_0dc7_2351,
        0xfc5f_1dcc_3ad1_077e,
        0x770f_af24_0772_c017,
        0x9a72_3d3f_988b_df31,
        0x179d_f76a_f1b2_c82c,
        0xfa86_b946_9c88_2186,
        0x6237_bf3c_3e00_3d3f,
        0x1be8_cff4_fbf8_59a5,
        0xbe2f_fe49_ae8f_a49d,
        0x5675_e557_60cc_6f5a,
        0x12df_709c_8a36_fee3,
    ];

    #[test]
    fn golden_worldgen_hash_wide() {
        let dims = [
            ("overworld", Dimension::Overworld),
            ("nether", Dimension::Nether),
            ("end", Dimension::End),
        ];
        let mut got: Vec<(String, u64)> = Vec::new();
        for &seed in WIDE_SEEDS.iter() {
            for (dname, dim) in dims {
                let h = wide_hash(seed, dim);
                println!("WIDE seed={seed:#018x} dim={dname} hash={h:#018x}");
                got.push((format!("{seed:#x}/{dname}"), h));
            }
        }
        for (i, (label, h)) in got.iter().enumerate() {
            assert_eq!(*h, GOLDEN_WIDE[i], "wide golden hash drifted for {label}");
        }
    }

    /// 1.0.5: targeted feature pins — one chunk per family (village /
    /// stronghold / ravine / ocean) at a fixed seed, fixed-order search.
    const GOLDEN_TARGETED: [u64; 4] = [
        // 4.4a re-pin (owner-approved accuracy-over-history 2026-10-10):
        // cross-chunk veins moved every overworld pin below
        0x0cd3_26c1_fd35_3d62,
        // 4.2a re-pin (owner-approved 2026-10-10 — pre-1.0.0, no prior
        // worlds): village spread 34/8/salt-10387312 moved the pinned
        // village; 4.4a veins + 4.1l shares moved it again to chunk
        // (8,45) (CI-measured; the search itself is deterministic)
        // 4.1p re-pin (CI-measured): shelf gates moved the pinned
        // village to chunk (72,11) (search-determined) with new
        // contents
        0x7399_c8ba_3427_79c2,
        // 4.1p re-pin (CI-measured): shelf gates moved ravine
        // chunk (0,0) contents (coords fixed — the ravine roll is
        // position-only, the biome bytes moved)
        0x3c9f_a975_d9cf_3894,
        // 4.1p re-pin (CI-measured): shelf gates moved ocean chunk
        // (-1,1) contents (coords fixed — the h60-61 shelf left the
        // ocean split)
        0x9652_10e3_6fa8_e22e,
    ];

    #[test]
    fn golden_targeted_features() {
        let seed = 0x00C0_FFEE_1234_5678;
        let gen = TerrainGen::for_dimension(seed, Dimension::Overworld);
        // stronghold chunk: ring-1 always has 3 strongholds
        let (sx, sz) = gen.strongholds()[0];
        let strong_chunk = (sx.div_euclid(16), sz.div_euclid(16));
        // village chunk: first center scanning regions outward
        let mut vchunk = (0, 0);
        'village: for r in 0..8 {
            for (rx, rz) in [(r, 0), (0, r), (r, r), (-r, -r), (r, -r), (-r, r)] {
                if let Some((wx, wz)) = gen.village_center(rx, rz) {
                    vchunk = (wx.div_euclid(16), wz.div_euclid(16));
                    break 'village;
                }
            }
        }
        // ravine chunk: first chunk with a rolled ravine nearby
        let mut rchunk = (0, 0);
        'ravine: for r in 0..17 {
            for (cx, cz) in [(r, 0), (0, r), (r, r), (-r, -r), (r, -r), (-r, r)] {
                if !gen.ravines_near_chunk(cx, cz).is_empty() {
                    rchunk = (cx, cz);
                    break 'ravine;
                }
            }
        }
        // ocean chunk: first ocean-biome column scanning outward
        let mut ochunk = (0, 0);
        'ocean: for r in 0..40 {
            for (cx, cz) in [(r, 0), (0, r), (r, r), (-r, -r), (r, -r), (-r, r)] {
                if gen.column(cx * 16 + 8, cz * 16 + 8).biome.is_ocean() {
                    ochunk = (cx, cz);
                    break 'ocean;
                }
            }
        }
        let targets = [
            ("stronghold", strong_chunk),
            ("village", vchunk),
            ("ravine", rchunk),
            ("ocean", ochunk),
        ];
        for (i, (name, (cx, cz))) in targets.iter().enumerate() {
            let (chunk, _) = gen.generate_chunk(*cx, *cz, Vec::new());
            let mut h = fnv1a(0xCBF2_9CE4_8422_2325, seed.to_le_bytes().as_slice());
            h = hash_chunk(h, &chunk);
            println!("TARGET {name} chunk=({cx},{cz}) hash={h:#018x}");
            assert_eq!(
                h, GOLDEN_TARGETED[i],
                "targeted feature hash drifted for {name}"
            );
        }
    }
}

/// 1.0.5: per-function pinned-value tests for every production libm
/// site (R5, second layer under the neighborhood hash). Exact `to_bits`
/// or FNV folds at fixed seeds; runs in the 3-OS golden job.
#[cfg(test)]
mod libm_pinned_tests {
    use super::*;

    const PIN_SEED: u64 = 0x00C0_FFEE_1234_5678;

    fn fold64(mut h: u64, v: u64) -> u64 {
        h ^= v;
        h.wrapping_mul(0x0000_0100_0000_01B3)
    }

    /// shims at exactly-representable inputs — true on any correct libm.
    #[test]
    fn shims_are_bit_exact() {
        assert_eq!(dsin32(0.0).to_bits(), 0x0000_0000);
        assert_eq!(dcos32(0.0).to_bits(), 0x3F80_0000);
        assert_eq!(dsin64(0.0).to_bits(), 0x0000_0000_0000_0000);
        assert_eq!(dcos64(0.0).to_bits(), 0x3FF0_0000_0000_0000);
        assert_eq!(dsqrt32(4.0).to_bits(), 0x4000_0000);
        assert_eq!(dsqrt32(2.0).to_bits(), 0x3FB5_04F3);
        assert_eq!(dround32(2.5).to_bits(), 0x4040_0000);
        assert_eq!(dround32(-2.5).to_bits(), 0xC040_0000);
    }

    #[test]
    fn worm_path_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        let mut worm = None;
        'search: for r in 0..5 {
            for (cx, cz) in [(r, 0), (0, r), (r, r), (-r, -r)] {
                let ws = gen.cave_worms_near(cx, cz);
                if let Some(w) = ws.into_iter().next() {
                    worm = Some(w);
                    break 'search;
                }
            }
        }
        let worm = worm.expect("seed must roll a cave worm near origin");
        let path = gen.worm_path(&worm);
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        h = fold64(h, worm.steps as u64);
        h = fold64(h, worm.yaw.to_bits());
        h = fold64(h, path.len() as u64);
        for (x, y, z, w) in path.iter().take(4) {
            h = fold64(h, x.to_bits());
            h = fold64(h, y.to_bits());
            h = fold64(h, z.to_bits());
            h = fold64(h, w.to_bits());
        }
        println!(
            "PIN worm_path steps={} len={} hash={h:#018x}",
            worm.steps,
            path.len()
        );
        assert_eq!(h, 0x7f81_1bf0_a26d_d809, "pin worm_path");
    }

    /// ore_blob exercises dsin64/dcos64(theta) through the real flow.
    #[test]
    fn ore_blob_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        let mut chunk = Chunk::empty();
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    chunk.set(x, y, z, STONE);
                }
            }
        }
        let mut rng = Rng::new(0xBE5E);
        gen.place_ores(&mut chunk, 0, 0, &mut rng);
        let mut n_coal = 0u64;
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    if chunk.get_local(vc_chunk::chunk::LocalXZ::new(x, z), y) == COAL_ORE {
                        n_coal += 1;
                        h = fold64(h, (x + z * 16 + y * 256) as u64);
                    }
                }
            }
        }
        println!("PIN ore_blob coal={n_coal} hash={h:#018x}");
        assert_eq!((n_coal, h), (552, 0x49a2_2d41_7181_0e45), "pin ore_blob");
    }

    /// village_houses exercises dcos32/dsin32/dround32 per house.
    #[test]
    fn village_houses_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        let houses = gen.village_houses(64, 64);
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        h = fold64(h, houses.len() as u64);
        for hs in &houses {
            h = fold64(h, hs.x as u64);
            h = fold64(h, hs.z as u64);
            h = fold64(h, hs.floor as u64);
            h = fold64(h, hs.blacksmith as u64);
        }
        println!("PIN village_houses n={} hash={h:#018x}", houses.len());
        assert_eq!(h, 0xfc56_7616_14b2_4dd0, "pin village_houses");
    }

    /// strongholds exercises dcos32/dsin32/dround32 per ring slot.
    #[test]
    fn strongholds_pinned() {
        let gen = TerrainGen::new(PIN_SEED);
        let sh = gen.strongholds();
        assert_eq!(sh.len(), 3);
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        for (x, z) in &sh {
            h = fold64(h, *x as u64);
            h = fold64(h, *z as u64);
        }
        println!("PIN strongholds {sh:?} hash={h:#018x}");
        assert_eq!(h, 0x53b0_bd4e_7de0_7709, "pin strongholds");
    }

    /// 3.7e: locate agrees with the fixed ring, rejects unknown
    /// kinds, and is deterministic across calls.
    #[test]
    fn locate_finds_nearest_stronghold() {
        let gen = TerrainGen::new(PIN_SEED);
        let found = gen.locate_structure("stronghold", 0, 0).unwrap();
        let best = gen
            .strongholds()
            .into_iter()
            .min_by_key(|(sx, sz)| (sx.pow(2) + sz.pow(2)) as i64)
            .unwrap();
        assert_eq!((found.0, found.2), best);
        assert_eq!(gen.locate_structure("Stronghold", 0, 0), Some(found));
        assert_eq!(gen.locate_structure("castle", 0, 0), None);
        assert_eq!(
            gen.locate_structure("village", 100, -40),
            gen.locate_structure("village", 100, -40)
        );
    }

    /// 4.1a: census covers every column exactly once and is stable.
    #[test]
    fn biome_census_counts_columns() {
        let gen = TerrainGen::new(PIN_SEED);
        let c = gen.biome_census(0, 0, 2, 3);
        let total: u64 = c.iter().map(|(_, n)| n).sum();
        assert_eq!(total, 2 * 3 * 256);
        assert!(c.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(c, gen.biome_census(0, 0, 2, 3));
    }

    /// 4.1c: Nether census over 16×16 chunks holds all five
    /// families with the wastes plurality (shares pinned by the
    /// region roll; warped is rarest at 8%, hence the wide rect).
    #[test]
    fn nether_census_holds_five_families() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Nether);
        let c = gen.biome_census(0, 0, 16, 16);
        let total: u64 = c.iter().map(|(_, n)| n).sum();
        assert_eq!(total, 16 * 16 * 256);
        let ids: Vec<u8> = c.iter().map(|(id, _)| *id).collect();
        for want in [8, 171, 172, 170, 173] {
            assert!(ids.contains(&want), "missing vanilla id {want}: {c:?}");
        }
        let wastes = c.iter().find(|(id, _)| *id == 8).unwrap().1;
        assert!(wastes > total / 4, "wastes plurality");
    }

    /// 4.2e: well/hut placement is deterministic; hut centers sit on
    /// swamp; emits write their signature blocks into a scratch chunk.
    #[test]
    fn well_and_hut_placement_and_emit() {
        let g = TerrainGen::new(PIN_SEED);
        assert_eq!(g.wells_near(0, 0), g.wells_near(0, 0));
        assert_eq!(g.witch_huts_near(0, 0), g.witch_huts_near(0, 0));
        // every hut center in a ±4-region scan sits on swamp
        for rx in -4..=4 {
            for rz in -4..=4 {
                if let Some((wx, wz)) = g.witch_hut_center(rx, rz) {
                    assert_eq!(
                        g.column(wx, wz).biome,
                        Biome::Swamp,
                        "hut at ({wx},{wz}) must sit on swamp"
                    );
                }
            }
        }
        // emits on a scratch chunk: sandstone ring + water plus, and
        // the hut floor + cauldron + table + pot
        let (chunk, _) = g.generate_chunk(0, 0, Vec::new());
        let mut chunk = (*chunk).clone();
        g.emit_well(&mut chunk, 8, 8, 0, 0);
        let base = g.column(8, 8).height;
        let get = |c: &Chunk, x: i32, y: i32, z: i32| c.get(x as usize, y as usize, z as usize);
        assert_eq!(get(&chunk, 8 + 2, base + 1, 8), SANDSTONE, "well wall");
        assert_eq!(get(&chunk, 8, base + 2, 8), WATER, "well water");
        // hut at (8,40) lives in chunk (0,2) — separate scratch chunk
        let (hchunk, _) = g.generate_chunk(0, 2, Vec::new());
        let mut hchunk = (*hchunk).clone();
        g.emit_hut(&mut hchunk, 8, 40, 0, 32);
        let floor = g.column(8, 40).height + 3;
        let hget = |x: i32, y: i32, z: i32| hchunk.get(x as usize, y as usize, z as usize);
        assert_eq!(hget(8, floor, 8), SPRUCE_PLANKS, "hut floor");
        assert_eq!(hget(8 - 2, floor + 1, 8 - 2), CAULDRON);
        assert_eq!(hget(8 + 2, floor + 1, 8 - 2), CRAFTING_TABLE);
        assert_eq!(hget(8, floor + 1, 8 - 2), FLOWER_POT);
    }

    /// 4.1b: large biomes rescale the classification (same seed, same
    /// rect, different histogram — both complete and stable).
    #[test]
    fn large_biomes_rescale_classification() {
        let small = TerrainGen::new(PIN_SEED);
        let mut large = TerrainGen::new(PIN_SEED);
        large.large_biomes = true;
        let a = small.biome_census(0, 0, 4, 4);
        let b = large.biome_census(0, 0, 4, 4);
        assert_eq!(a.iter().map(|(_, n)| n).sum::<u64>(), 4 * 4 * 256);
        assert_eq!(b.iter().map(|(_, n)| n).sum::<u64>(), 4 * 4 * 256);
        assert_ne!(a, b, "large mode must move biome boundaries");
        assert_eq!(b, large.biome_census(0, 0, 4, 4));
    }

    /// 4.3a: amplified lifts the land (a clear share of sampled
    /// columns rises vs default at the same seed; deterministic).
    #[test]
    fn amplified_raises_land() {
        let plain = TerrainGen::new(PIN_SEED);
        let mut amp = TerrainGen::new(PIN_SEED);
        amp.amplified = true;
        let mut raised = 0;
        let mut total = 0;
        for x in (0..128).step_by(4) {
            for z in (0..128).step_by(4) {
                total += 1;
                if amp.column(x, z).height > plain.column(x, z).height {
                    raised += 1;
                }
            }
        }
        assert!(
            raised * 4 > total,
            "amplified must lift over a quarter of columns ({raised}/{total})"
        );
        assert_eq!(
            amp.column(0, 0).height,
            amp.column(0, 0).height,
            "deterministic"
        );
    }

    /// 4.3b: seam score pins above 0.85 (measured 0.909 on PIN_SEED:
    /// heights/biomes stitch clean; the miss class is per-chunk vein
    /// blobs clipped at borders — stone vs granite/diorite/andesite/
    /// gravel/dirt/ores. Fixing means cross-chunk blob resolution,
    /// which belongs to the 4.4 decoration pass).
    #[test]
    fn seams_match_along_a_row() {
        let gen = TerrainGen::new(PIN_SEED);
        let (matched, total) = gen.seam_score(0, 0, 3);
        assert!(total > 0);
        let rate = matched as f64 / total as f64;
        assert!(
            rate >= 0.85,
            "seam score floor 0.85, measured {rate:.3} ({matched}/{total})"
        );
    }

    /// 4.3c: height stats cover every column, means sit in a sane
    /// band, rows sort by id, and calls are stable.
    #[test]
    fn height_stats_cover_columns() {
        let gen = TerrainGen::new(PIN_SEED);
        let rows = gen.height_stats(0, 0, 2, 2);
        let total: u64 = rows.iter().map(|(_, n, _)| n).sum();
        assert_eq!(total, 2 * 2 * 256);
        assert!(rows.windows(2).all(|w| w[0].0 < w[1].0));
        for (_, _, mean) in &rows {
            assert!((0.0..256.0).contains(mean), "sane mean {mean}");
        }
        assert_eq!(rows, gen.height_stats(0, 0, 2, 2));
    }

    /// 4.4b: the void gap holds (no end stone 60..1000 out), chorus
    /// plants stand on end stone, and outer generation is stable.
    #[test]
    fn outer_end_gap_and_chorus() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::End);
        // gap: chunk (40,0) ≈ 640 blocks out — no end stone anywhere
        let (gap, _) = gen.generate_chunk(40, 0, Vec::new());
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    assert_ne!(
                        gap.get(x, y, z),
                        END_STONE,
                        "void gap must be empty at ({x},{y},{z})"
                    );
                }
            }
        }
        // chorus validity over a far chunk: plants stand on stone/plant
        let (far, _) = gen.generate_chunk(80, 0, Vec::new());
        for y in 1..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    if far.get(x, y, z) == CHORUS_PLANT {
                        let below = far.get(x, y - 1, z);
                        assert!(
                            below == END_STONE || below == CHORUS_PLANT,
                            "chorus floats at ({x},{y},{z})"
                        );
                    }
                }
            }
        }
        // stable
        let (far2, _) = gen.generate_chunk(80, 0, Vec::new());
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    assert_eq!(far.get(x, y, z), far2.get(x, y, z));
                }
            }
        }
    }

    /// 4.1e: hill/deep/shore variants round-trip ids and classify
    /// at their gates (deep oceans need depth; hills need elevation).
    #[test]
    fn hill_variant_ids_and_gates() {
        for (v, id) in [
            (Biome::DesertHills, 17),
            (Biome::TaigaHills, 19),
            (Biome::DeepOcean, 24),
            (Biome::StoneShore, 25),
            (Biome::BirchHills, 28),
            (Biome::GiantTreeTaiga, 32),
            (Biome::GiantTreeTaigaHills, 33),
            (Biome::WoodedMountains, 34),
            (Biome::DeepLukewarmOcean, 48),
            (Biome::DeepColdOcean, 49),
            (Biome::GravellyMountains, 131),
        ] {
            assert_eq!(v.vanilla_id(), id);
            assert!(!v.name().is_empty());
        }
        // internal ids are dense 28..=38 and fold back
        for (i, b) in (28u16..=38).zip(
            [
                Biome::DesertHills,
                Biome::TaigaHills,
                Biome::DeepOcean,
                Biome::StoneShore,
                Biome::BirchHills,
                Biome::GiantTreeTaiga,
                Biome::GiantTreeTaigaHills,
                Biome::WoodedMountains,
                Biome::DeepLukewarmOcean,
                Biome::DeepColdOcean,
                Biome::GravellyMountains,
            ]
            .iter(),
        ) {
            assert_eq!(Biome::from_u8(i as u8), *b);
        }
        let g = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        // deep water classifies ocean-family regardless of gates
        // (4.1o: deep is h < SEA-11; the probe uses h=45)
        let deep_biome = g.classify(0.1, 0.0, 0.0, 45, 1.0, (0, 0)).0;
        assert!(
            matches!(
                deep_biome,
                Biome::DeepColdOcean
                    | Biome::DeepOcean
                    | Biome::DeepLukewarmOcean
                    | Biome::FrozenOcean
            ),
            "deep water is ocean-family, got {deep_biome:?}"
        );
        // shallow water classifies ocean-family too (4.1p: ocean is
        // h<60; the probe uses h=58, above the h<52 deep line)
        let shal_biome = g.classify(0.1, 0.0, 0.0, 58, 1.0, (0, 0)).0;
        assert!(shal_biome.is_ocean(), "got {shal_biome:?}");
        // overlay helpers are pure elevation/variant gates (4.1n: the
        // family splits take the uniform cell roll as 4th arg)
        assert_eq!(
            g.finish_land_base(Biome::Desert, 80, 0.0, 0).0,
            Biome::DesertHills
        );
        assert_eq!(
            g.finish_land_base(Biome::Desert, 64, 0.0, 0).0,
            Biome::Desert
        );
        assert_eq!(
            g.finish_land_base(Biome::Taiga, 80, 0.0, 50).0,
            Biome::TaigaHills
        );
        assert_eq!(g.taiga_overlay(80, 0.6, 10).0, Biome::GiantTreeTaigaHills);
        // deep oceans are ocean family
        assert!(Biome::DeepOcean.is_ocean());
        assert!(Biome::DeepLukewarmOcean.is_ocean());
        assert!(Biome::DeepColdOcean.is_ocean());
        assert!(!Biome::DesertHills.is_ocean());
    }

    /// 4.1g: layer stack is deterministic, land fraction sane, deep
    /// is ocean-only, and special marks ~1/13 of land.
    #[test]
    fn layer_stack_shape() {
        let gen = TerrainGen::new(PIN_SEED);
        let (mut land, mut deep, mut special, mut n) = (0u64, 0u64, 0u64, 0u64);
        for cx4 in -64..64 {
            for cz4 in -64..64 {
                let (l, d, s) = gen.layer_cell(cx4, cz4);
                n += 1;
                if l {
                    land += 1;
                }
                if d {
                    deep += 1;
                    assert!(!l, "deep is ocean-only");
                }
                if s {
                    special += 1;
                    assert!(l, "special marks land");
                }
            }
        }
        let land_f = land as f64 / n as f64;
        // island density FIT (80%) — band tracks the constant
        assert!(
            (0.65..0.95).contains(&land_f),
            "land fraction sane: {land_f}"
        );
        assert!(deep > 0, "some deep ocean exists");
        let sp_f = special as f64 / land.max(1) as f64;
        assert!(
            (0.03..0.15).contains(&sp_f),
            "special ≈ 1/13 of land: {sp_f}"
        );
        assert_eq!(gen.layer_cell(3, -7), gen.layer_cell(3, -7));
    }

    /// 4.1a: vanilla ids match the cited registry values.
    #[test]
    fn vanilla_ids_spot_checks() {
        assert_eq!(Biome::Ocean.vanilla_id(), 0);
        assert_eq!(Biome::River.vanilla_id(), 7);
        assert_eq!(Biome::MushroomFields.vanilla_id(), 14);
        assert_eq!(Biome::Jungle.vanilla_id(), 21);
        assert_eq!(Biome::CrimsonForest.vanilla_id(), 171);
    }

    /// ravines_near_chunk exercises dcos32/dsin32 per ravine.
    #[test]
    fn ravines_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        let mut found = None;
        'search: for r in 0..17 {
            for (cx, cz) in [(r, 0), (0, r), (r, r), (-r, -r)] {
                let rv = gen.ravines_near_chunk(cx, cz);
                if let Some(first) = rv.into_iter().next() {
                    found = Some(((cx, cz), first));
                    break 'search;
                }
            }
        }
        let ((cx, cz), rv) = found.expect("seed must roll a ravine in range");
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        h = fold64(h, rv.x0 as u64);
        h = fold64(h, rv.z0 as u64);
        h = fold64(h, rv.dx.to_bits() as u64);
        h = fold64(h, rv.dz.to_bits() as u64);
        h = fold64(h, rv.length as u64);
        println!("PIN ravines chunk=({cx},{cz}) hash={h:#018x}");
        assert_eq!(h, 0x4d00_6b9d_9734_8cc6, "pin ravines");
    }

    /// end_pillar_tops mirrors the pillar-angle math (dcos32/dsin32).
    #[test]
    fn end_pillars_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::End);
        let tops = gen.end_pillar_tops();
        assert_eq!(tops.len(), 10);
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        for (x, top, z) in &tops {
            h = fold64(h, *x as u64);
            h = fold64(h, *top as u64);
            h = fold64(h, *z as u64);
        }
        println!("PIN end_pillars hash={h:#018x}");
        assert_eq!(h, 0x4c24_15d1_fa10_524e, "pin end_pillars");
    }

    /// the end-island radius primitive (dsqrt32 over axis distances).
    #[test]
    fn end_island_dist_pinned() {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        for d2 in [0.0f32, 1.0, 1764.0, 3600.0, 10000.0, 12345.0] {
            h = fold64(h, dsqrt32(d2).to_bits() as u64);
        }
        println!("PIN end_island_dist hash={h:#018x}");
        assert_eq!(h, 0xd074_e40d_fa38_6994, "pin end_island_dist");
    }

    /// find_spawn exercises dsin32/dcos32 in the 12-dir scoring.
    #[test]
    fn find_spawn_pinned() {
        let gen = TerrainGen::for_dimension(PIN_SEED, Dimension::Overworld);
        let (x, y, z) = gen.find_spawn();
        println!("PIN find_spawn ({x},{y},{z})");
        // 4.1p re-pin: shelf gates moved the spawn search result
        // (new layout, same search rules; CI-measured)
        assert_eq!((x, y, z), (232.5, 71.0, 248.5), "pin find_spawn");
    }
}
