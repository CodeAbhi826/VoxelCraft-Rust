# Minecraft Java 1.16.5 Chunk Storage Internals

**Research task R2 for VoxelCraft-Rust** — exact numbers, formats and sources for re-implementing
MC 1.16.5-style chunk storage in Rust. Every non-trivial claim has a source URL. Claims that
correct the uploaded roadmap / ROADMAP-ANALYSIS.md are marked **[CORRECTION]**.

---

## 0. The numbers that matter (TL;DR)

| Quantity | Value (1.16.5) | Source |
|---|---|---|
| Chunk column | 16 × 256 × 16 (blocks), 16 sections of 16³ | [Chunk format – Block format](https://minecraft.wiki/w/Chunk_format) |
| Section size | 16×16×16 = 4096 blocks; sections Y=0..15; empty sections not saved | same |
| Build range (Overworld) | block Y 0..=255 ("build limit 256"); heightmap values 0..=256 | [Anvil history](https://minecraft.wiki/w/Anvil_file_format), [protocol heightmap](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format) |
| Global block state count | **17,112 states, 763 blocks** (registry unchanged 1.16.2→1.16.5) | [minecraft-data 1.16.2](https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.16.2/blocks.json) + [yarn 1.16.5 Blocks javadoc](https://maven.fabricmc.net/docs/yarn-1.16.5+build.10/net/minecraft/block/Blocks.html) |
| Global palette bits (blocks) | **ceil(log2(17112)) = 15 bits** (direct palette) | computed; consistent with [protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format) ("15 bits" for modern vanilla, ≥9 ⇒ global) |
| Indirect palette bits | 4 (≤16 entries, linear/array), 5–8 (≤256 entries, hashmap); >256 ⇒ direct 15 | [quarry docs](https://quarry.readthedocs.io/en/latest/data_types/chunks.html), [wiki.vg-era rule](https://c4k3.github.io/wiki.vg/Chunk_Format.html) |
| Palette index packing | `entries_per_long = floor(64/bits)`; `longs = ceil(4096/entries_per_long)`; **entries never straddle a long boundary** | [protocol Data Array format](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format) |
| 4-bit section payload | 16 entries/long ⇒ **256 longs = 2048 bytes** | computed from rule above |
| 5-bit | 12/long ⇒ 342 longs (matches MC-239610 example) | [MC-239610](https://bugs.mojang.com/browse/MC-239610) |
| 6/7/8/15-bit | 410 / 456 / 512 / 1024 longs | computed |
| Index order in arrays | **YZX**: `i = (y<<8) | (z<<4) | x` ("letters=X, lines=Z, pages=Y") | [Chunk format – Block format](https://minecraft.wiki/w/Chunk_format) |
| Block/sky light | **2048 bytes per section per type** (4096 nibbles); **even index = LOW nibble, odd = HIGH nibble** | [Chunk format](https://minecraft.wiki/w/Chunk_format), [wiki.vg](https://c4k3.github.io/wiki.vg/Chunk_Format.html) |
| Biomes | **per column**: `Biomes` IntArray(256) at `Level` root; 3D 4×4×4 (64 entries/section) only since **1.18** | [Chunk format history](https://minecraft.wiki/w/Chunk_format) |
| Biome count 1.16.5 | 79 registered biomes | [minecraft-data 1.16.2 biomes.json](https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.16.2/biomes.json) |
| Heightmaps | 9-bit values 0..=256, **7 per long, 37 longs** per map; 6 types | [Chunk format](https://minecraft.wiki/w/Chunk_format), [Heightmap](https://minecraft.wiki/w/Heightmap) |
| Data version | 1.16.5 = **2586** | [Data version list](https://minecraft.wiki/w/Data_version), [misode versions](https://misode.github.io/versions) |
| Region file | 4 KiB sectors; 8 KiB header (1024×[3B offset+1B count] + 1024×4B timestamps); payload = 4B length + 1B compression + data; 1.16.5 saves **zlib (scheme 2)** | [Region file format](https://minecraft.wiki/w/Region_file_format) |
| 1.16.5 palette classes | `ArrayPalette`, `HashMapPalette`, `IdentityPalette` (global) — **no singleton palette in 1.16.5** | [Forge 1.16.5 javadoc package](https://nekoyue.github.io/ForgeJavaDocs-NG/javadoc/1.16.5/net/minecraft/util/palette/package-summary.html) |

---

## 1. Paletted containers in 1.16.5

### 1.1 Palette strategy (verified from 1.16.5 code structure + community docs)

1.16.5 has exactly three palette implementations (Mojang-mapped names, from the Forge 1.16.5
javadoc package listing: `ArrayPalette`, `HashMapPalette`, `IdentityPalette`, plus
`IResizeCallback` for growth and `PalettedContainer` itself):

- **ArrayPalette ("linear")** — dense array `index → entry`, used at **4 bits per entry**
  (holds up to 16 entries).
- **HashMapPalette** — bidirectional hash `entry ↔ index`, used at **5–8 bits per entry**
  (up to 256 entries).
- **IdentityPalette ("global"/direct)** — the registry itself (`ObjectIntIdentityMap`);
  used when a section would need **> 8 bits**, at **`ceil(log2(total_states))` bits = 15 in
  1.16.5** (17,112 states > 2^14 = 16,384, ≤ 2^15 = 32,768).

Rules on **serialization** (disk NBT and network in the 1.13–1.17 era):
- bits = **4** if palette size ≤ 16;
- bits = **`ceil(log2(palette_size))`** if 17..256 entries (i.e. 5..8);
- bits = **15 (global, indices are raw state IDs)** if the palette would exceed 256 entries.
  The disk-format wiki text phrases the indirect rule as "the minimum amount of bits required
  to represent the largest index in the palette, then set to a minimum size of 4 bits"
  ([Chunk format](https://minecraft.wiki/w/Chunk_format)); the 256-entry threshold before going
  global is documented e.g. by
  [quarry's chunk docs](https://quarry.readthedocs.io/en/latest/data_types/chunks.html)
  ("A palette is used when there are fewer than 256 unique values; the value width varies
  from 4 to 8 bits").
- On **read**, the notchian parser accepts any bits-per-entry ≥ 4; a bits value of 9+ implies
  the global palette (see the 1.16-era wiki.vg text: "For bits per block <= 4, 4 bits are used;
  for bits per block between 5 and 8, the given value is used" — indirect, otherwise global —
  [archived wiki.vg Chunk_Format](https://c4k3.github.io/wiki.vg/Chunk_Format.html)).
- **No single-value palette in 1.16.5**: the class doesn't exist in 1.16.5 (only added in 1.18's
  format overhaul, where a 1-entry palette may omit the data array entirely). A 1.16.5 section
  whose palette has 1 entry is an ArrayPalette @ 4 bits and **still writes the 256-long
  BlockStates array** (all zeros). Modern parsers treat a missing BlockStates with a 1-entry
  palette as valid; strict 1.16.5 parity writes it.
  Sources: package listing above; modern behavior note ("If only one block state is present in
  the palette, this field is not required") is from the *current*
  [Chunk format](https://minecraft.wiki/w/Chunk_format) page which describes 1.18+.

**Palette growth** in 1.16.5 (`PalettedContainer.onResize`, `setBits`): inserting an unseen
entry when the palette is full bumps bits by one (4→5→…→8), re-allocates the palette,
**repacks all 4096 entries** into the new bit width, and past 8 bits converts to the
IdentityPalette (15 bits) — one full repack of the section. This is the growth ladder to
re-implement. The 1.16.5 container guards this with a **`ReentrantLock`** plus public
`acquire()/release()` (fields visible in the
[Forge 1.16.5 PalettedContainer javadoc](https://nekoyue.github.io/ForgeJavaDocs-NG/javadoc/1.16.5/net/minecraft/util/palette/PalettedContainer.html);
yarn 1.16.5 mirrors it as `writeLock`
[yarn 1.16.5](https://maven.fabricmc.net/docs/yarn-1.16.5+build.10/net/minecraft/world/chunk/PalettedContainer.html)).
This is the "palette growth locking concern" from ROADMAP-ANALYSIS §2 — real in vanilla, but
avoidable in VoxelCraft via copy-on-write immutable sections (see §9).

### 1.2 Serialization shape (NBT, 1.16.5 disk)

Per section compound:

- `Y` (byte): section index **0..15**.
- `Palette` (list of compounds): each entry `{ Name: "minecraft:stone", Properties: { facing: "north", ... } }`
  — properties are **absent entirely for property-less blocks**.
- `BlockStates` (long array): 4096 palette indices, bit-packed (see 1.3).
- `BlockLight`, `SkyLight` (byte arrays, 2048 bytes each) — see §5.

### 1.3 Bit packing — the exact rule

> "The indices are not packed across multiple elements of the array, meaning that if there is
> no more space in a given 64-bit integer for the whole next index, it starts instead at the
> first (lowest) bit of the next 64-bit integer." —
> [Chunk format](https://minecraft.wiki/w/Chunk_format)

> "The number of entries per long may be calculated as floor(64 / bits_per_entry). The number
> of longs in the array may then be calculated as ceil(number_of_entries / entries_per_long)."
> — [Java Edition protocol/Chunk format](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)

So entries are **word-aligned, never straddling** longs (a Go re-implementation phrases it:
"Entries are packed whole into each long and never straddle one, so a long holds 64/bits of
them and the remainder costs a whole long" —
[go-theft-craft chunk pkg](https://pkg.go.dev/github.com/go-theft-craft/minecraft-protocol/wire/java/chunk)).

For a section (4096 entries):

| bits | entries/long | longs | bytes | when |
|---:|---:|---:|---:|---|
| 4 | 16 | 256 | 2,048 | palette ≤ 16 |
| 5 | 12 | 342 | 2,736 | ≤ 32 |
| 6 | 10 | 410 | 3,280 | ≤ 64 |
| 7 | 9 | 456 | 3,648 | ≤ 128 |
| 8 | 8 | 512 | 4,096 | ≤ 256 |
| 15 | 4 | 1,024 | 8,192 | direct/global |

(MC-239610 cross-checks the math: a 14-entry palette ⇒ 4 bits; a 5-bit array with 12
blocks/long ⇒ 342 longs. https://bugs.mojang.com/browse/MC-239610)

Element order inside the array is **YZX**: index `i = (y<<8) | (z<<4) | x`; within a long,
entry `k` of that long occupies bits `[k*bits, k*bits+bits)` (LSB-first).
Flat 2-D arrays (biomes, heightmap indexing) use `i = z*16 + x`.

---

## 2. Biome storage in 1.16.5 — per-column, NOT 3D **[CORRECTION]**

- In 1.16.5, biomes are stored once **per (x,z) column**: `Level.Biomes` is an **IntArray of
  256 values** (one per column, `z*16+x` order), values = numeric biome registry IDs.
  "In versions 1.18 and before, the biome for a column is stored at the top level of the
  chunk's data" and the history table: "1.18: *The Biomes array in the Level tag now contains
  1024 integers instead of 256, allowing biomes to differ based on altitude*" —
  [Chunk format](https://minecraft.wiki/w/Chunk_format).
- 3D biomes (quart resolution, 4×4×4 cells, 64 per section, paletted with 1–3 bit indirect /
  6-bit-class global) arrived **in 1.18** ("A chunk section can contain at maximum 4096 unique
  block state IDs, and 64 (4×4×4) unique biome IDs" —
  [protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format); Bedrock got per-block
  biomes in its own 1.18 — [unmined dev note](https://unmined.net/2021/12/10/dev-bedrock-1-18-3d-biome-format)).
- 1.16.5 has **79 registered biomes** (Overworld 61 + Nether 5 + End 5 + 8 technical/unused
  retained) — count from
  [minecraft-data 1.16.2 biomes.json](https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.16.2/biomes.json)
  (biome set unchanged through 1.16.5).
- **[CORRECTION]** The roadmap's comment "biome per-column (should be 3D in 1.16.5)" is wrong
  for 1.16.5 parity: 1.16.5 is per-column. 3D biomes are an optional future feature, not parity.

---

## 3. Chunk sections, Y ranges, and section bookkeeping

- Section = 16×16×16; column = 16 sections, `Y` byte 0 (bottom) .. 15 (top) in 1.16.5
  ("Each section is a 16×16×16-block area, with up to 16 sections in a chunk: from 0 at the
  bottom, to 15 on top. Empty sections are not saved." —
  [Chunk format – Block format](https://minecraft.wiki/w/Chunk_format)).
- Overworld block Y range: **0..=255** (the "256 build limit" — heightmaps can read 256 =
  "column fully empty above the world" / "highest position occupied is y+1"; converting a
  1.16 world to 1.18 adds 64 to every heightmap value, implying a 0-based bottom —
  [Chunk format](https://minecraft.wiki/w/Chunk_format)). Anvil raised the build height to
  256 in 1.2.1 ([Anvil](https://minecraft.wiki/w/Anvil_file_format)). Nether 0..=127
  (8 sections), End 0..=255. Negative-Y sections only exist from 1.17/1.18 on.
- **Per-section `BlockCount` / `nonEmptySectionCount` are runtime bookkeeping, not NBT**: the
  1.16.5 `ChunkSection` class carries `short nonEmptyBlockCount`, `short tickingBlockCount`,
  `short tickingFluidCount` (verified in the
  [Forge 1.16.5 ChunkSection javadoc](https://nekoyue.github.io/ForgeJavaDocs-NG/javadoc/1.16.5/net/minecraft/world/chunk/ChunkSection.html)),
  recomputed by `recalcBlockCounts()` on load. Purposes:
  - `nonEmptyBlockCount == 0` ⇒ `isEmpty()` ⇒ skip section in meshing/light/ticking/saving
    and treat as uniform air (also drives "don't send this section" on the network);
  - `tickingBlockCount` / `tickingFluidCount` ⇒ skip random-tick scheduling for sections with
    no tickable blocks;
  - a chunk-level "nonEmptySectionCount" (number of sections with any non-air block) is a
    derived quantity used by tooling/server forks for chunk-emptiness tests.
  Vanilla 1.16.5 Anvil NBT does **not** store these counts — empty sections are simply absent
  from the `Sections` list, and counts are recomputed on load.

---

## 4. BlockState global IDs (1.16.5)

### 4.1 Counts **[CORRECTION of "~26k states in 1.16.5"]**

- 1.16.5: **763 blocks / 17,112 block states** ⇒ **15 bits** global palette.
  Sources: block count cross-verified two ways — 763 `public static final Blocks` fields in the
  [yarn 1.16.5 Blocks javadoc](https://maven.fabricmc.net/docs/yarn-1.16.5+build.10/net/minecraft/block/Blocks.html)
  and 763 blocks with 17,112 min→max state IDs in
  [minecraft-data 1.16.2 blocks.json](https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.16.2/blocks.json)
  (the block registry is unchanged from 1.16.2 through 1.16.5; 1.16.3–1.16.5 were fix releases).
- The "~26,000 states" figure is a **modern** number, not 1.16.5. Measured from
  [minecraft-data](https://github.com/PrismarineJS/minecraft-data): 1.16.2=17,112;
  1.17/1.18=20,342; 1.19=21,448; 1.19.4=23,725; 1.20.2=24,276; 1.21.4=27,866. The current
  protocol page still quotes **15 bits** for vanilla block states
  ([source](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)), so 15 bits has been
  the answer from 1.16 through 1.21.x — convenient for us.
- Registry IDs are **signed ints in vanilla**, and the direct width can grow up to 31 bits with
  mods ([protocol note](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)).

### 4.2 How a state ID is built (verified empirically)

- One ID per unique combination; "if a block has multiple properties then the number of
  allocated states is the product of the number of values for each property. The block state
  IDs belonging to a given block are always consecutive." —
  [protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format).
- `state_id = block_base_id + combination_index` where the combination index is a **mixed-radix
  number over the block's properties**:
  1. **Properties are sorted alphabetically by property NAME** (the 1.16.5
     `StateContainer` keeps `ImmutableSortedMap<String, Property<?>> propertiesByName` —
     [Forge 1.16.5 StateContainer javadoc](https://nekoyue.github.io/ForgeJavaDocs-NG/javadoc/1.16.5/net/minecraft/state/StateContainer.html));
     the data-generator output iterates that sorted order, e.g. redstone_wire lists
     `east, north, power, south, west` even though the block registers
     `north, east, south, west, power`).
  2. **The alphabetically-LAST property varies fastest** (first property = most significant
     digit).
  3. Each property's **values are indexed in their declaration/enum order** — NOT
     alphabetically. Booleans are **`[true, false]`** (true = index 0). E.g. facing is
     `[north, south, west, east]` (Direction.values() order filtered to horizontal), stairs
     half `[top, bottom]`, doors half `[upper, lower]`.
  4. `combination_index = Σ_i value_index_i × (Π_{j>i} radix_j)` over the name-sorted
     property list.
  - Empirical validation (this research): using
    [minecraft-data 1.21.4 blocks.json](https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.21.4/blocks.json)
    `defaultState − minStateId` offsets, this exact algorithm predicts every tested block:
    oak_door 11, oak_leaves 27, hopper 0, chest 1, furnace 1, repeater 3, oak_stairs 11 —
    all exact matches. (The enumeration code is unchanged since the 1.13 flattening, so the
    rule holds for 1.16.5.)
  - **The "default state" of a block is NOT necessarily the base ID** (e.g. oak_stairs default
    is `facing=north, half=bottom, shape=straight, waterlogged=false` → base+11). Store a
    `default_state_id` per block; the base ID is just where the range starts.

### 4.3 What this means for a registry

- Keep `block_id → (base_state_id, props)` and a flat `Vec<BlockState>` of 17,112 entries for
  vanilla parity; each `BlockState` can lazily reference its block + property values.
- Property order for ID math: **name-sorted**; value order: **as declared**; booleans
  `true` before `false`.

---

## 5. Light storage

- Per section, per type: **2048 bytes** = 4096 nibbles, YZX order
  ("[Byte Array] BlockLight: 2048 Bytes stores the amount of block-emitted light in each
  block"; "[Byte Array] SkyLight: 2048 Bytes stores the maximum sky light that reaches each
  block" — [Chunk format](https://minecraft.wiki/w/Chunk_format)).
- **Nibble packing (canonical)**: "Light level is stored as 4 bits per block, 2 blocks sharing
  a byte: starting at 0, **even blocks take the first nibble, and odd blocks the second one**"
  ([Chunk format](https://minecraft.wiki/w/Chunk_format)); the wiki.vg-era protocol text
  agrees: "even-indexed items … are packed into the low bits, odd-indexed into the high bits"
  ([archived wiki.vg](https://c4k3.github.io/wiki.vg/Chunk_Format.html)). I.e.
  `get(i) = data[i>>1] >> ((i&1)<<2) & 0xF`; `set(i,v)`: clear then OR at `((i&1)<<2)`.
  Note: the roadmap's own `set_block_light` snippet (lines 366–375 of the uploaded roadmap)
  implements exactly this (mask `0xF0 >> nibble*4` preserves the *other* nibble); the
  ROADMAP-ANALYSIS "mask math backwards" flag on that specific snippet is a misread — the
  canonical rule above is what matters. **[CORRECTION]**
- Sky light above the heightmap is implicitly 15 (vanilla doesn't store all-15 skylight
  sections; sections with no skylight data are treated as full/empty accordingly). BlockLight
  is only present where non-trivial — in 1.16.5 saves, both are per-section tags and may be
  absent.
- The engine keeps light at **section granularity** so a light change re-meshes only affected
  sections; MC recomputes heightmap + light incrementally on edits.

---

## 6. Heightmaps

Six types exist ([Heightmap](https://minecraft.wiki/w/Heightmap)); the stored value is
"the amount of blocks above the bottom of the world", i.e. **`top_block_y + 1`**, range
0..=256 for a 1.16.5 column (0 = empty column, 256 = fully occupied) —
[Chunk format](https://minecraft.wiki/w/Chunk_format):

- `WORLD_SURFACE` (+`_WG` worldgen variant): highest **non-air** (all air types) block.
- `OCEAN_FLOOR` (+`_WG`): highest block whose **material blocks motion** (solid), i.e. the
  sea floor under water; server-side.
- `MOTION_BLOCKING`: highest block that **blocks motion or contains a fluid** (used for
  rain/snow rendering, entity "in rain" checks).
- `MOTION_BLOCKING_NO_LEAVES`: same, excluding leaves (used e.g. for pillager patrols).

Uses: weather particle placement, mob spawn height queries, sky-light seeding, beacon
obstruction, `/execute positioned over`, F3 "SH/CH" display
([Heightmap](https://minecraft.wiki/w/Heightmap); the protocol packet currently ships
WORLD_SURFACE (id 1), MOTION_BLOCKING (4), MOTION_BLOCKING_NO_LEAVES (5) —
[protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)).

Storage: long array of **256 entries × 9 bits**, 7 per long ⇒ **37 longs** ("In versions prior
to 1.16 the heightmaps were stored in 36 Long values, where the bits were arranged in an
uninterrupted stream … 1.16+: 37 longs, 7 values each, 7×9=63, last bit unused" —
[Chunk format](https://minecraft.wiki/w/Chunk_format)). Index = `z*16+x`, x fastest.
Bits-per-entry formula: `ceil(log2(world_height + 1))` = 9 for height 256
([protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)).
Heightmaps are updated incrementally on block add/remove (`Heightmap.update`), not recomputed
from scratch per edit.

---

## 7. What's actually on disk (Anvil, 1.16.5) — relevant to our future save system

Region file (`.mca`, `r.<regionX>.<regionZ>.mca`, 32×32 chunks each)
([Region file format](https://minecraft.wiki/w/Region_file_format)):

- Sectors of **4096 bytes**; the whole file is sector-aligned (size multiple of 4 KiB).
- Header = two tables, 1024 entries each (8 KiB total): location entries `3-byte big-endian
  sector offset + 1-byte sector count` (0 ⇒ chunk absent), then 1024 big-endian 4-byte
  **timestamps**.
- Chunk payload: **4-byte big-endian length** (of what follows), **1 byte compression type**,
  then `length−1` bytes of compressed data. Types: 1 = GZip, 2 = **zlib (what the vanilla
  1.16.5 client always writes)**, 3 = none; 4 = LZ4 and the 128-flag external `.mcc` variant
  only exist since 1.20.5.
- Decompressed payload = **NBT** (gzip-independent; region-level compression only). Root
  compound `""` with `DataVersion` (**2586** for 1.16.5) and `Level` compound:
  `Status`, `Biomes` IntArray(256), `Heightmaps` compound (subset of the 6 types — the four
  non-`_WG` ones for full chunks), `Sections` list (see §1.2), `TileTicks`/`LiquidTicks`,
  `ToBeTicked`/`LiquidsToBeTicked` (16 sublists of packed shorts, 1.16-era), `BlockEntities`,
  `Entities`, `InhabitedTime`, `LastUpdate`, `Lights` (proto-chunks), `Structures`.
  ([Chunk format NBT structure](https://minecraft.wiki/w/Chunk_format))
- The **roadmap's "AtomicPtr WorldGrid" / flat-grid plans can stay orthogonal** to this: the
  on-disk format only matters at save/load time. Design the in-memory structures for the
  renderer, and write a thin `Chunk ↔ NBT ⇄ zlib ⇄ region` codec later. DataFixerUpper-style
  `DataVersion` versioning (2586) is a good convention to copy even for our own saves.

---

## 8. Corrections to the roadmap's claims (summary)

| Roadmap claim | Reality (sourced above) |
|---|---|
| "~15,000–26,000 state variants in 1.16.5" | 17,112 states / 763 blocks; ~26k is a 1.21-era number. 15 bits either way. |
| "biome … should be 3D in 1.16.5" | 1.16.5 = per-column IntArray(256); 3D 4×4×4 came in 1.18. |
| Paletted container "4/8/16/32-bit" scheme | Real ladder: 4 (linear) → 5–8 (hashmap) → 15 (global). 16/32-bit widths aren't vanilla 1.16.5. |
| nibble `set_block_light` mask "backwards" (ROADMAP-ANALYSIS) | The roadmap snippet is actually correct; canonical rule = even index→low nibble, odd→high. |
| palette growth "requires repacking with locking" | True in vanilla (ReentrantLock + onResize repack 4→5→…→8→15); avoidable with CoW sections. |

---

## 9. Design recommendations for VoxelCraft

Goal: 1.16.5-faithful **paletted 16³ sections** that slot into the existing
`HashMap<ChunkPos, Arc<Chunk>>` + rayon-mesh + copy-on-write architecture
(`voxelcraft/src/chunk.rs`, `world.rs`) without a big-bang rewrite.

### 9.1 Types

```rust
/// Global block-state id. 1.16.5 vanilla needs 17,112 ids (15 bits),
/// mods/our-own growth can exceed u16 (vanilla ids are i32; direct width
/// can reach 31 bits) → use u32 everywhere OUTSIDE the packed arrays.
pub type StateId = u32;          // registry-wide ids
pub const AIR: StateId = 0;

pub const SECTION_BLOCKS: usize = 4096;   // 16*16*16
pub const SECTIONS_PER_CHUNK: usize = 16; // 1.16.5 overworld

#[inline]
pub fn local_index(x: u8, y: u8, z: u8) -> usize {  // YZX, MC order
    (y as usize) << 8 | (z as usize) << 4 | x as usize
}

/// One 16³ slice. Cheap to clone-on-write (Arc<Section>).
pub struct Section {
    pub blocks: PalettedContainer,
    pub block_light: Box<[u8; 2048]>, // nibbles, even=low, odd=high
    pub sky_light:   Box<[u8; 2048]>,
    pub non_air: u16,                  // 0 ⇒ empty section (skip mesh/tick/save)
}

pub enum Palette {
    /// 0 bits; whole section is one state. (Beyond 1.16.5 parity — 1.18
    /// behavior — but the single biggest win for caves/sky. Optional flag.)
    Single(StateId),
    /// 4..=8 bits, index → StateId. Linear search is fine ≤16 entries;
    /// use a small Vec + HashMap side-index for the 5–8 bit sizes.
    Indirect { bits: u8, entries: Vec<StateId> },
    /// 15+ bits, raw StateIds (global palette). bits = registry_direct_bits().
    Direct { bits: u8 },
}

pub struct PalettedContainer {
    palette: Palette,
    /// words.len() = ceil(4096 / (64/bits)); 0 words for Single.
    words: Vec<u64>,
}

pub struct Chunk {
    /// y-major sections; missing ⇒ uniform air (never store all-air sections).
    pub sections: [Option<Arc<Section>>; SECTIONS_PER_CHUNK],
    /// Per-column (1.16.5 parity; 3D quart biomes are 1.18+, add later if wanted).
    pub biomes: Box<[u8; 256]>,
    /// topmost-qualifying-block y + 1; 0..=256; keep the 4 MC types or start
    /// with WORLD_SURFACE + MOTION_BLOCKING. u16 per column.
    pub heightmaps: Heightmaps,          // e.g. Box<[u16; 256]> per type
    pub dirty_sections: u16,             // bitmask per section for remesh granularity
}
```

### 9.2 Packing (match MC exactly — same helpers serve future Anvil I/O)

```rust
impl PalettedContainer {
    fn entries_per_long(bits: u8) -> usize { 64 / bits as usize }     // floor
    fn long_count(bits: u8) -> usize {
        (SECTION_BLOCKS + Self::entries_per_long(bits) - 1) / Self::entries_per_long(bits)
    }
    pub fn get(&self, i: usize) -> StateId {
        match &self.palette {
            Palette::Single(s) => *s,
            Palette::Indirect { bits, entries } => {
                let epl = Self::entries_per_long(*bits);
                let w = self.words[i / epl];
                let v = ((w >> ((i % epl) * *bits as u64 as usize)) & ((1<<bits)-1)) as usize;
                entries.get(v).copied().unwrap_or(AIR)  // bounds-check: MC assumes valid
            }
            Palette::Direct { .. } => { /* same extraction, value IS the StateId */ }
        }
    }
}
```

`words.len()` table for a section: 4b→256, 5b→342, 6b→410, 7b→456, 8b→512, 15b→1024
(see §1.3). Light nibbles: `get: (b[idx>>1] >> ((idx&1)<<2)) & 0xF`.

### 9.3 Palette growth — the ladder, done on the private copy (no locks)

- `insert(state)`: if Indirect and `state` not in `entries`:
  - if `entries.len() + 1 <= 1 << bits` → push;
  - else if `bits < 8` → **`grow_to(bits + 1)`**: rebuild palette + repack all 4096 entries
    (read old, write new — 4096 ops, trivial);
  - else → **convert to Direct** at `direct_bits()` (≈15; recompute at registry build) and
    repack once. This is exactly MC 1.16.5's `onResize` ladder, minus the `ReentrantLock`:
    in VoxelCraft the section being mutated is a thread-private clone (see 9.4), so growth
    never contends with the meshing threads.
- Optional `Palette::Single` shrink-on-write is NOT vanilla 1.16.5 (no singleton palette
  existed); add it only as an explicit extension flag, and keep the NBT writer 1.16.5-correct
  (always emit `BlockStates`, even for 1-entry palettes, if byte-parity saves ever matter).

### 9.4 Copy-on-write at SECTION granularity + Arc immutability (the CoW/rayon question)

Current `world.rs::set_block` clones the whole 64 KiB chunk per edit. With sections:

1. `World.chunks: HashMap<ChunkPos, Arc<Chunk>>` stays exactly as-is (mesh jobs keep
   snapshotting `Arc<Chunk>` via `snapshot3x3` — zero changes to the rayon pipeline in
   `game.rs`/`mesh.rs`).
2. `set_block(wx, wy, wz, id)`:
   ```rust
   let sec = wy >> 4;
   // clone ONLY the 16³ section (~2–8 KiB + palette), then rebuild a cheap Chunk shell
   let mut new_section = Arc::unwrap_or_clone(chunk.sections[sec].clone());
   new_section.blocks.set(local_index(x, y & 15, z), id);
   new_section.non_air = new_section.non_air.saturating_add_signed(delta);
   let mut new_chunk = clone_chunk_shell_reusing_other_arcs(chunk);
   new_chunk.sections[sec] = Some(Arc::new(new_section));
   self.chunks.insert(pos, Arc::new(new_chunk));   // publish atomically
   ```
   Unchanged sections are **shared** `Arc<Section>`s — an edit now copies ~1/16th of the data,
   and in-flight mesh jobs that grabbed the old `Arc<Chunk>` keep reading a consistent old
   snapshot (same guarantee the current full-chunk CoW gives, at 16× finer granularity).
3. Chunk-shell cloning cost is 16 `Option<Arc<Section>>` + 2 `Box<[u8;256]>` + maps — to keep
   it truly cheap, wrap the arrays in `Arc` too (`Arc<[Option<Arc<Section>>; 16]>`,
   `Arc<[u8; 256]>`) so the shell clone is pure pointer copies. `Arc::make_mut` on the section
   array can even give free in-place mutation when the refcount is 1 (single-player fast path).
4. Meshing: `mesh_chunk` should read per-section (grab neighbor `Arc<Chunk>`, then the 3
   sections vertically adjacent to each face boundary). Keep the current per-chunk mesh output
   for now (phase 4 of the roadmap already plans per-section dirty tracking + MDI); use
   `dirty_sections: u16` to skip re-meshing untouched sections and `non_air == 0` to skip
   air sections entirely (mirrors `ChunkSection.isEmpty()`).
5. Never put interior mutability (locks/atomics) inside `Section`. All mutation happens on the
   exclusive clone before publication — this is the fix for the roadmap's "palette growth
   locking" concern and keeps `Arc<Chunk>` strictly immutable for rayon.

### 9.5 State IDs: u32 now, u16 never

- Use `StateId = u32` for the registry, palettes, mesh vertex channels, and APIs. Vanilla
  1.16.5 fits in u16 (17,112 < 65,536) but the *direct* palette already needs 15 bits, our own
  registry will grow past u16 once blockstate properties (facing/half/etc.) are enumerated,
  and vanilla itself is i32 (modded direct widths reach 31 bits —
  [protocol](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)). The only place
  bits actually matter is the packed `words` — u32 costs nothing there.
- Build the registry exactly like §4.2 (name-sorted properties, declared value order,
  `[true,false]` booleans, last-property-fastest) so future resource-pack/model work and any
  vanilla-data tooling (misode/mcmeta-style JSON) line up with real MC state IDs.

### 9.6 Biomes / heightmaps / light

- Biomes: keep `Box<[u8; 256]>` per column (1.16.5 parity — **[CORRECTION]** to roadmap); the
  current `chunk.biome` field already matches; just give it real registry ids later.
- Heightmaps: store `WORLD_SURFACE` and `MOTION_BLOCKING` as `Box<[u16; 256]>` (values
  0..=256). Update incrementally on `set_block` (top-down adjust like vanilla) instead of
  the current `top_solid_y` scan; use them to seed sky light and to cull mesh work. On save:
  repack to 9-bit × 7/long × 37 longs.
- Light: per-section nibble arrays per §5 (2048 bytes each, even→low nibble). Sky light above
  `WORLD_SURFACE` is implicitly 15 — don't allocate/store full-15 sections; represent as
  `Option<Box<[u8;2048]>> = None` meaning "default 15" for sky, "default 0" for block light.
  The existing BFS block-light pass moves to per-section data; a light edit marks only the
  touched sections dirty.

### 9.7 Memory budget (per chunk, typical)

| Layout | Empty chunk | Surface chunk (~5 sections @4b) | Worst case (all 16 @15b) |
|---|---|---|---|
| Current flat `u8[65536]` | 64 KiB | 64 KiB | 64 KiB (can't hold states) |
| Paletted sections | ~0.5 KiB (shell) | 5×(2 KiB data + palette + 2×2 KiB light) ≈ 25 KiB | 16×(8 KiB + 4 KiB light) ≈ 200 KiB |
| With `Single` palette extension | ~0.5 KiB | ~11 KiB (2 sky/cave sections free) | same |

(The 4-bit case: 256 longs = 2048 B data + ≤16 palette entries + 2×2048 B light.)

### 9.8 Migration order (fits the ROADMAP-ANALYSIS revised sequence)

1. Land `Section`/`PalettedContainer` (get/set/grow) **behind the existing `Chunk::get/set`
   API** — `chunk.rs` callers unchanged; keep the flat path compiling via a feature until the
   mesher is ported.
2. Move the mesher to section-relative reads + per-section dirty bits (still one mesh buffer
   per chunk); light arrays move into `Section`.
3. Switch `world.rs::set_block` to the section-granularity CoW of §9.4.
4. Heightmaps + per-column biomes as in §9.6.
5. Only then: `BlockState` registry (u32 ids, §4.2 math) + packed vertices; Anvil/NBT save
   codec last (§7) — the in-memory design above is deliberately wire/disk-compatible.

---

## 10. Source index

- Chunk format (NBT, sections, palette rule, light, heightmaps, biome history):
  https://minecraft.wiki/w/Chunk_format
- Protocol chunk format (packing formula, palette formats, heightmap bits formula, biome
  4×4×4, global palette 15 bits): https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format
- Region file format: https://minecraft.wiki/w/Region_file_format
- Anvil file format (build height 256, YZX ordering): https://minecraft.wiki/w/Anvil_file_format
- Heightmap types/semantics: https://minecraft.wiki/w/Heightmap
- Data version list (1.16.5 = 2586): https://minecraft.wiki/w/Data_version ,
  https://misode.github.io/versions , https://docs.rs/mcdata/latest/mcdata/data_version/index.html
- 1.16.5 code structure (PalettedContainer fields incl. ReentrantLock; palette classes
  ArrayPalette/HashMapPalette/IdentityPalette; StateContainer ImmutableSortedMap;
  ChunkSection nonEmpty/ticking counts):
  https://nekoyue.github.io/ForgeJavaDocs-NG/javadoc/1.16.5/net/minecraft/util/palette/PalettedContainer.html
  .../util/palette/package-summary.html , .../state/StateContainer.html ,
  .../world/chunk/ChunkSection.html ,
  https://maven.fabricmc.net/docs/yarn-1.16.5+build.10/net/minecraft/world/chunk/PalettedContainer.html ,
  .../net/minecraft/block/Blocks.html (763 fields)
- Counts (763 blocks / 17,112 states 1.16.2–1.16.5; 79 biomes; per-version state totals):
  https://github.com/PrismarineJS/minecraft-data/blob/master/data/pc/1.16.2/blocks.json ,
  .../1.16.2/biomes.json , .../1.21.4/blocks.json (defaultState offsets used to verify the
  ID-assignment algorithm)
- Packing cross-check (342 longs @5 bits / 14-entry palette): https://bugs.mojang.com/browse/MC-239610
- Indirect ≤ 256 / 4–8 bits rule: https://quarry.readthedocs.io/en/latest/data_types/chunks.html
- Non-straddling + nibble even/odd rule (1.16-era text): https://c4k3.github.io/wiki.vg/Chunk_Format.html
- "never straddle" (Go implementation phrasing):
  https://pkg.go.dev/github.com/go-theft-craft/minecraft-protocol/wire/java/chunk
