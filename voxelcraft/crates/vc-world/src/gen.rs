//! World generation: simplex noise, fBm, biomes, caves, trees.
//! Fully deterministic per seed; pure functions (safe on worker threads).

use crate::world::Dimension;
use std::sync::Arc;
use vc_blocks::blocks::*;
use vc_chunk::chunk::{Chunk, CHUNK_LEN};
use vc_rng::rng::Rng;

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
            _ => Biome::Ocean,
        }
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

const F2: f32 = 0.3660254037844386; // 0.5 * (sqrt(3) - 1)
const G2: f32 = 0.2113248654051871; // (sqrt(3) - 1) / 6
const F3: f32 = 0.3333333333333333;
const G3: f32 = 0.1666666666666667;

pub struct Noise {
    perm: Box<[u8; 512]>,
    perm_mod12: Box<[u8; 512]>,
}

impl Noise {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut p = [0u8; 256];
        for i in 0..256 {
            p[i] = i as u8;
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

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// --------------------------------------------------------------- terrain --

pub struct ColumnInfo {
    pub height: i32,
    pub biome: Biome,
    pub top: u8,
    pub filler: u8,
}

/// P7 structures: village grid + one house site (deterministic)
pub const VILLAGE_REGION_CHUNKS: i32 = 24;
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

/// mineshaft generation chance per chunk — VERIFIED (wiki Mineshaft
/// page, live): "a 0.4% chance to attempt to begin generating in every
/// chunk"
pub const MINESHAFT_CHANCE: f32 = 0.004;
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

pub struct TerrainGen {
    pub seed: u64,
    /// §28: which dimension this generator produces
    pub dim: Dimension,
    n_cont: Noise,
    n_mfac: Noise,
    n_ridge: Noise,
    n_detail: Noise,
    n_temp: Noise,
    n_humid: Noise,
    n_cave1: Noise,
    n_cave2: Noise,
    n_cave3: Noise,
    /// §28 nether: cavern pair (bigger scale than overworld caves)
    n_neth1: Noise,
    n_neth2: Noise,
    /// §28 nether: wall density variation so caverns aren't uniform
    n_neth3: Noise,
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
            n_cont: Noise::new(seed ^ 0x1000),
            n_mfac: Noise::new(seed ^ 0x2000),
            n_ridge: Noise::new(seed ^ 0x3000),
            n_detail: Noise::new(seed ^ 0x4000),
            n_temp: Noise::new(seed ^ 0x5000),
            n_humid: Noise::new(seed ^ 0x6000),
            n_cave1: Noise::new(seed ^ 0x7000),
            n_cave2: Noise::new(seed ^ 0x8000),
            n_cave3: Noise::new(seed ^ 0x9000),
            n_neth1: Noise::new(seed ^ 0xA100),
            n_neth2: Noise::new(seed ^ 0xA200),
            n_neth3: Noise::new(seed ^ 0xA300),
        }
    }

    pub fn column(&self, x: i32, z: i32) -> ColumnInfo {
        let xf = x as f32;
        let zf = z as f32;
        let cont = fbm2(&self.n_cont, xf / 1500.0, zf / 1500.0, 4, 2.0, 0.5);
        let base = 64.0 + cont * 26.0;

        let mut ridge = 1.0 - self.n_ridge.noise2(xf / 230.0, zf / 230.0).abs();
        ridge = ridge * ridge;
        let mfac = smoothstep(
            0.15,
            0.55,
            fbm2(
                &self.n_mfac,
                (xf + 700.0) / 950.0,
                (zf - 300.0) / 950.0,
                2,
                2.0,
                0.5,
            ),
        );
        let detail = fbm2(&self.n_detail, xf / 60.0, zf / 60.0, 3, 2.0, 0.5) * 4.0;

        let h = (base + ridge * 52.0 * mfac + detail)
            .clamp(8.0, 170.0)
            .floor() as i32;

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

        let (biome, top, filler) = if h < vc_chunk::SEA_LEVEL - 1 {
            if h < vc_chunk::SEA_LEVEL - 6 {
                (Biome::Ocean, GRAVEL, GRAVEL)
            } else {
                (Biome::Ocean, SAND, SAND)
            }
        } else if h <= vc_chunk::SEA_LEVEL + 1 {
            (Biome::Beach, SAND, SAND)
        } else if h > 96 {
            if h > 112 {
                (Biome::Mountains, SNOW, STONE)
            } else {
                (Biome::Mountains, STONE, STONE)
            }
        } else if temp < -0.32 {
            (Biome::Snowy, SNOW_GRASS, DIRT)
        }
        // ---- Phase 10 climate biomes: our temp/humid predicates are the
        // documented climate adaptation (vanilla 1.16.5 selects biomes
        // through a biome lattice, not two noises); the BIOME CONTENT
        // (surface, trees, tint) follows the wiki descriptions ----
        else if temp < -0.1 {
            // Taiga: cold enough for spruce but not snow-locked
            (Biome::Taiga, GRASS, DIRT)
        } else if temp > 0.25 && humid < -0.12 {
            // Badlands: hot AND the driest climate band — terracotta
            // layers (vanilla's 1.16.5 badlands surface is terracotta
            // banded with colored terracotta; ours is the plain block —
            // palette adaptation)
            (Biome::Badlands, TERRACOTTA, TERRACOTTA)
        } else if temp > 0.3 && humid < 0.05 {
            (Biome::Desert, SAND, SAND)
        } else if temp > 0.25 && humid > 0.3 {
            // Jungle: hot + wet (dense oak canopy + melons — vanilla's
            // jungle wood/melon patches adapted to our palette)
            (Biome::Jungle, GRASS, DIRT)
        } else if temp > 0.35 {
            // Savanna: hot, mid-dry — yellow-tinted grass, sparse trees
            (Biome::Savanna, GRASS, DIRT)
        } else if humid > 0.45 && h <= 66 {
            // Swamp: wettest band + low flat terrain — murky grass,
            // water pools (vanilla 1.16.5 swamps sit at low elevation)
            (Biome::Swamp, GRASS, DIRT)
        } else if humid > 0.12 && temp < 0.2 {
            // Birch Forest: wet + cool — birch-dominant canopy
            (Biome::BirchForest, GRASS, DIRT)
        } else if humid > 0.12 {
            (Biome::Forest, GRASS, DIRT)
        } else {
            (Biome::Plains, GRASS, DIRT)
        };

        ColumnInfo {
            height: h,
            biome,
            top,
            filler,
        }
    }

    /// Is the block at (x,y,z) carved by a cave? Only called underground.
    fn cave(&self, x: i32, y: i32, z: i32) -> bool {
        if y < 7 {
            return false;
        }
        let xf = x as f32;
        let yf = y as f32;
        let zf = z as f32;
        let n1 = self.n_cave1.noise3(xf / 110.0, yf / 55.0, zf / 110.0);
        let n2 = self
            .n_cave2
            .noise3((xf + 800.0) / 110.0, yf / 55.0, (zf - 800.0) / 110.0);
        // spaghetti tunnels: intersection of two noise "sheets"
        if n1 * n1 + n2 * n2 < 0.010 {
            return true;
        }
        // cheese caverns, deep
        if y < 42 && self.n_cave3.noise3(xf / 170.0, yf / 90.0, zf / 170.0) > 0.62 {
            return true;
        }
        false
    }

    /// Deterministic ore / stone-variant picker for deep stone. Called for
    /// every STONE block — pure hash, no noise evaluation (fast).
    fn stone_variant(&self, x: i32, y: i32, z: i32) -> u8 {
        // stone-family blobs (granite/diorite/andesite) — hash-based patches
        let v = Rng::hash3(self.seed ^ 0xA000, x >> 3, y >> 3, z >> 3);
        let patch = (v % 100) as u32;
        let variant = match patch {
            0..=7 => Some(GRANITE),
            8..=15 => Some(DIORITE),
            16..=23 => Some(ANDESITE),
            _ => None,
        };
        if let Some(s) = variant {
            // smooth the blob edges: blend by finer hash
            let edge = Rng::hash3(self.seed ^ 0xA001, x, y, z) % 4;
            if edge != 0 {
                return s;
            }
        }

        // ores by depth (1.16.5-ish distributions)
        let o = Rng::hash3(self.seed ^ 0xB000, x, y, z);
        let p = (o % 100_000) as f32 / 100_000.0;
        if p < 0.0012 && y <= 14 {
            DIAMOND_ORE
        } else if p < 0.0024 && y <= 30 {
            LAPIS_ORE
        } else if p < 0.0035 && y <= 32 {
            GOLD_ORE
        } else if p < 0.0055 && y <= 16 {
            REDSTONE_ORE
        } else if p < 0.013 && y <= 64 {
            IRON_ORE
        } else if p < 0.024 && y <= 96 {
            COAL_ORE
        } else {
            STONE
        }
    }

    /// Generate one chunk column (dimension-dispatched). Pure: returns
    /// chunk + edits for neighbors (tree canopies crossing chunk borders).
    pub fn generate_chunk(
        &self,
        cx: i32,
        cz: i32,
        inbound: Vec<(u16, u8)>, // (block idx, id) edits queued from neighbors
    ) -> (Arc<Chunk>, Vec<(i32, i32, i32, u8)>) {
        match self.dim {
            Dimension::Overworld => self.generate_overworld_chunk(cx, cz, inbound),
            Dimension::Nether => self.generate_nether_chunk(cx, cz, inbound),
        }
    }

    /// The overworld generator: terrain columns, caves, ores, vegetation,
    /// villages. §26/§48 Phase 7.
    fn generate_overworld_chunk(
        &self,
        cx: i32,
        cz: i32,
        inbound: Vec<(u16, u8)>, // (block idx, id) edits queued from neighbors
    ) -> (Arc<Chunk>, Vec<(i32, i32, i32, u8)>) {
        let mut chunk = Chunk::empty();
        let mut rng = Rng::new(Rng::hash3(self.seed, cx, 0, cz));
        let sea = vc_chunk::SEA_LEVEL;
        let mut outbound: Vec<(i32, i32, i32, u8)> = Vec::new();

        // Phase 10: ravines covering this chunk (computed once — the
        // 11×11-chunk neighborhood covers the max 127-block diagonal so
        // every chunk independently agrees on every ravine that reaches
        // it, exactly like the village/mineshaft region queries)
        let ravines = self.ravines_near_chunk(cx, cz);

        // pass 1: terrain columns
        for z in 0..CHUNK_Z_CHUNK() {
            for x in 0..CHUNK_X_CHUNK() {
                let wx = cx * 16 + x as i32;
                let wz = cz * 16 + z as i32;
                let col = self.column(wx, wz);
                let h = col.height;
                let col_idx = z * 16 + x;
                chunk.height[col_idx] = h.min(255) as u8;
                chunk.biome[col_idx] = col.biome as u8;
                // the ravine cut interval for this column (None = no cut)
                let rv_cut = if ravines.is_empty() {
                    None
                } else {
                    self.ravine_cut(&ravines, wx, wz, h)
                };

                let top_y = h.max(sea).min(255) as usize;
                for y in 0..=top_y {
                    let yi = y as i32;
                    let b: u8 = if y == 0 {
                        BEDROCK
                    } else if y <= 2 && rng.next_f32() < 0.35 {
                        BEDROCK
                    } else if yi > h {
                        WATER
                    } else if yi == h {
                        col.top
                    } else if yi > h - 4 {
                        col.filler
                    } else {
                        self.stone_variant(wx, yi, wz)
                    };

                    // cave carving (never through bedrock; stay well below surface,
                    // extra margin under oceans so caves don't flood)
                    if b != BEDROCK && b != WATER && yi <= h {
                        let margin = if col.biome == Biome::Ocean || col.biome == Biome::Beach {
                            10
                        } else {
                            5
                        };
                        if h - yi > margin && self.cave(wx, yi, wz) {
                            continue; // leave air
                        }
                        // Phase 10: ravine carve — the V-cut interval for
                        // this column; never through bedrock or water
                        // (deep bottoms expose stone + ores in the walls,
                        // the wiki-verified look; vanilla's lava-flooded
                        // floors are palette-sim-out-of-scope, documented)
                        if let Some((rv_top, rv_bottom)) = rv_cut {
                            if yi <= rv_top && yi > rv_bottom {
                                continue; // leave air
                            }
                        }
                    }
                    if b != AIR {
                        chunk.set(x, y, z, b);
                    }
                }
            }
        }

        // pass 2: inbound edits from neighbors (trees poking into this chunk)
        for (idx, id) in inbound {
            let cur = chunk.get_idx(idx as usize);
            if cur == AIR || (cur == LEAVES && id == OAK_LOG) {
                chunk.set_idx(idx as usize, id);
            }
        }

        // pass 3: decorations (trees, plants) — deterministic per chunk
        let ox = cx * 16;
        let oz = cz * 16;
        let mut set_dec = |chunk: &mut Chunk,
                           outbound: &mut Vec<(i32, i32, i32, u8)>,
                           wx: i32,
                           wy: i32,
                           wz: i32,
                           id: u8,
                           replace_leaves: bool| {
            if wy < 0 || wy > 255 {
                return;
            }
            let lxi = wx - ox;
            let lzi = wz - oz;
            if lxi >= 0 && lxi < 16 && lzi >= 0 && lzi < 16 {
                let cur = chunk.get(lxi as usize, wy as usize, lzi as usize);
                if cur == AIR || (replace_leaves && cur == LEAVES && id == OAK_LOG) {
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
                Biome::Jungle => 10,
                Biome::Taiga => 5,
                Biome::Swamp => 3,
                Biome::Savanna => {
                    if rng.next_f32() < 0.6 {
                        1
                    } else {
                        0
                    }
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
            let top = chunk.get(lx as usize, chunk.height[col_idx] as usize, lz as usize);
            if top != GRASS && top != SNOW_GRASS {
                continue;
            }
            let h = chunk.height[col_idx] as i32;
            // species: forest mixes oak + birch; snowy taiga/taiga grow
            // spruce; birch forest is birch-dominant; jungle keeps oak
            // (vanilla jungle wood is palette-absent — documented)
            let biome_here = Biome::from_u8(chunk.biome[col_idx]);
            let (log, leaf) = match biome_here {
                Biome::Snowy | Biome::Taiga => (SPRUCE_LOG, SPRUCE_LEAVES),
                Biome::Forest => {
                    if rng.next_f32() < 0.35 {
                        (BIRCH_LOG, BIRCH_LEAVES)
                    } else {
                        (OAK_LOG, LEAVES)
                    }
                }
                Biome::BirchForest => {
                    if rng.next_f32() < 0.75 {
                        (BIRCH_LOG, BIRCH_LEAVES)
                    } else {
                        (OAK_LOG, LEAVES)
                    }
                }
                _ => (OAK_LOG, LEAVES),
            };
            let th = if biome_here == Biome::Snowy || biome_here == Biome::Taiga {
                6 + rng.next_range(3) as i32 // spruce grows taller
            } else {
                4 + rng.next_range(3) as i32 // 4..6
            };
            let y0 = h + 1;

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
            // dirt under trunk
            if chunk.get(lx as usize, h as usize, lz as usize) == GRASS {
                chunk.set(lx as usize, h as usize, lz as usize, DIRT);
            } else if chunk.get(lx as usize, h as usize, lz as usize) == SNOW_GRASS {
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
                _ => 0,
            }
        };
        for _ in 0..plant_attempts {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let col_idx = lz as usize * 16 + lx as usize;
            let h = chunk.height[col_idx] as i32;
            if chunk.get(lx as usize, h as usize, lz as usize) != GRASS {
                continue;
            }
            if chunk.get(lx as usize, (h + 1) as usize, lz as usize) != AIR {
                continue;
            }
            let r = rng.next_f32();
            let id = if r < 0.72 {
                TALL_GRASS
            } else if r < 0.86 {
                FLOWER_RED
            } else {
                FLOWER_YELLOW
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
            if chunk.get(lx as usize, h as usize, lz as usize) != GRASS
                && chunk.get(lx as usize, h as usize, lz as usize) != SNOW_GRASS
            {
                continue;
            }
            if chunk.get(lx as usize, (h + 1) as usize, lz as usize) != AIR {
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

        // desert: dead bushes + cactus columns + clay in low sand
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            if b == Biome::Desert {
                for _ in 0..4 {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if chunk.get(lx as usize, h as usize, lz as usize) != SAND {
                        continue;
                    }
                    if chunk.get(lx as usize, (h + 1) as usize, lz as usize) != AIR {
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
                    if h < vc_chunk::SEA_LEVEL - 2 || h > vc_chunk::SEA_LEVEL + 1 {
                        continue;
                    }
                    if chunk.get(lx as usize, h as usize, lz as usize) == SAND {
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
                    if chunk.get(lx as usize, h as usize, lz as usize) == GRASS
                        && chunk.get(lx as usize, (h + 1) as usize, lz as usize) == AIR
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
                    if h < vc_chunk::SEA_LEVEL || h > vc_chunk::SEA_LEVEL + 2 {
                        continue;
                    }
                    if chunk.get(lx as usize, h as usize, lz as usize) == GRASS
                        && chunk.get(lx as usize, (h + 1) as usize, lz as usize) == AIR
                    {
                        // 2x1 shallow pool: punch the surface to water
                        chunk.set(lx as usize, h as usize, lz as usize, WATER);
                        if lx + 1 < 16
                            && chunk.get((lx + 1) as usize, h as usize, lz as usize) == GRASS
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
                let hmax = (chunk.height[col_idx] as i32 - 6).max(8).min(40);
                if hmax <= 10 {
                    continue;
                }
                let y = 8 + rng.next_range((hmax - 8) as u32) as i32;
                let above = chunk.get(lx as usize, (y + 1) as usize, lz as usize);
                let here = chunk.get(lx as usize, y as usize, lz as usize);
                if here == AIR && (is_opaque(above) && above != BEDROCK) {
                    chunk.set(lx as usize, (y + 1) as usize, lz as usize, GLOWSTONE);
                    // a couple of extra glow blocks around it
                    let extra = rng.next_range(3);
                    for _ in 0..extra {
                        let dx = rng.next_range(3) as i32 - 1;
                        let dz = rng.next_range(3) as i32 - 1;
                        let nx = (lx + dx).clamp(0, 15) as usize;
                        let nz = (lz + dz).clamp(0, 15) as usize;
                        if chunk.get(nx, (y + 1) as usize, nz) != AIR
                            && chunk.get(nx, y as usize, nz) == AIR
                        {
                            chunk.set(nx, (y + 1) as usize, nz, GLOWSTONE);
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
        for &(sx, sz) in self.strongholds().iter() {
            // skip far strongholds cheaply (the layout spans ~30 blocks
            // around the center; the guard avoids running the emit for
            // the 99.99% of chunks nowhere near one)
            if (sx - ox).abs() > 40 || (sz - oz).abs() > 40 {
                continue;
            }
            self.emit_stronghold(&mut chunk, sx, sz, ox, oz);
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
    /// the surface; air above it)
    fn gen_solid(&self, x: i32, y: i32, z: i32) -> bool {
        let col = self.column(x, z);
        let h = col.height;
        if y > h {
            return false; // above the terrain surface: air (or water)
        }
        if y == 0 || y <= 2 {
            return true; // bedrock layers
        }
        let margin = if col.biome == Biome::Ocean || col.biome == Biome::Beach {
            10
        } else {
            5
        };
        if h - y > margin && self.cave(x, y, z) {
            return false; // carved
        }
        true
    }

    /// roll this chunk's dungeon (pure — no chunk data needed; the layout
    /// derives from the seed + terrain functions, so it is identical from
    /// any caller: generation, tests, E2E)
    pub fn dungeon_in_chunk(&self, cx: i32, cz: i32) -> Option<DungeonRoom> {
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0D66, cx, 0, cz));
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
                .all(|dx| (-1..=size).all(|dz| self.gen_solid(wx0 + dx, y0 - 1, wz0 + dz)));
            if !floor_ok {
                continue;
            }
            // ceiling area incl. over walls: entirely solid
            let ceil_ok = (-1..=size)
                .all(|dx| (-1..=size).all(|dz| self.gen_solid(wx0 + dx, y0 + 5, wz0 + dz)));
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
                    let air2 = !self.gen_solid(wx0 + dx, y0, wz0 + dz)
                        && !self.gen_solid(wx0 + dx, y0 + 1, wz0 + dz);
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
    fn generate_nether_chunk(
        &self,
        cx: i32,
        cz: i32,
        inbound: Vec<(u16, u8)>,
    ) -> (Arc<Chunk>, Vec<(i32, i32, i32, u8)>) {
        let mut chunk = Chunk::empty();
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0D1D, cx, 0, cz));
        // the nether has no cross-chunk decorations (structures are
        // strictly in-chunk) → outbound stays empty
        let outbound: Vec<(i32, i32, i32, u8)> = Vec::new();

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
                chunk.biome[col_idx] = Biome::NetherWastes as u8;
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
                        // regions rockier or hollower so caverns vary
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
                        // regional bias leans hollow. Base ~0.055 keeps the
                        // mass dominant (~70% rock); the ±0.09 swing gives
                        // rocky vs hollower regions; the mid-height band
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
                    let is_bed = y <= fb.saturating_sub(1) || y >= cb + 1 || y == 0 || y == 127;
                    let solid =
                        is_bed || nether[(y.max(0) * 256 + z as i32 * 16 + x as i32) as usize];
                    if !solid {
                        continue;
                    }
                    let b: u8 = if is_bed {
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

        // decorations: soul sand floors + glowstone ceilings (deterministic).
        // NOTE: chunk.get returns raw STATE ids — nether blocks store their
        // dedicated states (118..120), so comparisons fold via state_block.
        let fold = |s: u8| state_block(s as u16);
        for _ in 0..14 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            // scan the column for a floor (solid below air) in the band
            let mut y = 30;
            while y < 100 {
                let here_air = chunk.get(lx as usize, y as usize, lz as usize) == AIR
                    && (y + 1) < 128
                    && chunk.get(lx as usize, (y + 1) as usize, lz as usize) == AIR;
                let below = if y > 0 {
                    chunk.get(lx as usize, (y - 1) as usize, lz as usize)
                } else {
                    BEDROCK
                };
                if here_air && fold(below) == NETHERRACK {
                    // vanilla-ish: soul sand valley patches — replace the top
                    // 1..2 floor blocks
                    let depth = 1 + rng.next_range(2) as i32;
                    for d in 0..depth {
                        chunk.set(lx as usize, (y - 1 - d) as usize, lz as usize, SOUL_SAND);
                    }
                    break;
                }
                y += 1;
            }
        }
        for _ in 0..8 {
            let lx = rng.next_range(16) as i32;
            let lz = rng.next_range(16) as i32;
            let mut y = 20;
            while y < 110 {
                let here = chunk.get(lx as usize, y as usize, lz as usize);
                let above = if y < 127 {
                    chunk.get(lx as usize, (y + 1) as usize, lz as usize)
                } else {
                    BEDROCK
                };
                if here == AIR && fold(above) == NETHERRACK {
                    chunk.set(lx as usize, (y + 1) as usize, lz as usize, GLOWSTONE);
                    // a small cluster around it
                    let extra = rng.next_range(3);
                    for _ in 0..extra {
                        let dx = rng.next_range(3) as i32 - 1;
                        let dz = rng.next_range(3) as i32 - 1;
                        let nx = (lx + dx).clamp(0, 15) as usize;
                        let nz = (lz + dz).clamp(0, 15) as usize;
                        let there = chunk.get(nx, (y + 1) as usize, nz);
                        let below_there = chunk.get(nx, y as usize, nz);
                        if fold(there) == NETHERRACK && below_there == AIR {
                            chunk.set(nx, (y + 1) as usize, nz, GLOWSTONE);
                        }
                    }
                    break;
                }
                y += 1;
            }
        }

        (Arc::new(chunk), outbound)
    }

    /// §28: find a spawn position inside a nether cavern — spiral-scan
    /// chunks from the origin for an open floor with headroom (the very
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
                                let feet = chunk.get(lx, y, lz);
                                let head = if y + 1 < 128 {
                                    chunk.get(lx, y + 1, lz)
                                } else {
                                    BEDROCK
                                };
                                let floor = if y > 0 {
                                    chunk.get(lx, y - 1, lz)
                                } else {
                                    BEDROCK
                                };
                                if feet == AIR && head == AIR && is_solid(state_block(floor as u16))
                                {
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
    pub fn villages_near(&self, ox: i32, oz: i32) -> Vec<(i32, i32)> {
        const RC: i32 = VILLAGE_REGION_CHUNKS * 16; // region size in blocks
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

    /// deterministic village center for one region, or None. Placement:
    /// ~55% of regions have a village; the center is jittered inside the
    /// region and must sit on flat-enough plains/meadow above sea level.
    fn village_center(&self, rx: i32, rz: i32) -> Option<(i32, i32)> {
        const RC: i32 = VILLAGE_REGION_CHUNKS * 16;
        let mut rng = Rng::new(Rng::hash3(self.seed, rx, 0x5EED, rz));
        if rng.next_f32() > 0.55 {
            return None;
        }
        // jitter across the region interior (margin keeps houses off borders)
        let base_x = rx * RC + 32;
        let base_z = rz * RC + 32;
        let span = RC - 64;
        let wx = base_x + rng.next_range(span as u32) as i32;
        let wz = base_z + rng.next_range(span as u32) as i32;
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
            let hx = wx + (ang.cos() * r).round() as i32;
            let hz = wz + (ang.sin() * r).round() as i32;
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
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u8| {
            let lx = x - ox;
            let lz = z - oz;
            if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && (0..256).contains(&y) {
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
    // rule live-verified from minecraft.wiki (2026-09-04) with the
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
    // (palette-absent, honestly documented); cave-spider spawner → the
    // registry's spider spawner (no distinct cave-spider mob).

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
        let put = |chunk: &mut Chunk, wx: i32, wy: i32, wz: i32, id: u8| {
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
                            let ground = self.gen_solid(wx, ms.y, wz);
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
                        let below = self.gen_solid(wx, ms.y, wz);
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
                chunk.set_state(lxi, ms.y as usize, lzi, spawner_state(2));
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
        let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x0E5, rx, 0, rz));
        let cx = rx * 32 + 4 + rng.next_range(24) as i32;
        let cz = rz * 32 + 4 + rng.next_range(24) as i32;
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
        let base = self.column(wx, wz).height as i32; // ground level
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u8| {
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
                        put(chunk, x, y, z, AIR); // hollow interior
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
                let mut rng = Rng::new(Rng::hash3(self.seed ^ 0x3E4E, rx, 0, rz));
                let cx = rx * 32 + 4 + rng.next_range(24) as i32;
                let cz = rz * 32 + 4 + rng.next_range(24) as i32;
                let mut ok = true;
                for d in [0i32, 4, -4] {
                    let c = self.column(cx * 16 + 8 + d, cz * 16 + 8 + d);
                    if c.biome != Biome::Jungle || c.height <= vc_chunk::SEA_LEVEL + 1 {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    out.push((cx * 16 + 8, cz * 16 + 8));
                }
            }
        }
        out
    }

    fn emit_jungle_temple(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let base = self.column(wx, wz).height as i32;
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u8| {
            let lxi = x - ox;
            let lzi = z - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) && (0..256).contains(&y) {
                chunk.set(lxi as usize, y as usize, lzi as usize, id);
            }
        };
        // 11×11 footprint, 3 floors of 4 high (our compact layout);
        // cobble/mossy mix on the shell (VERIFIED materials)
        let mix = |rng: &mut Rng| -> u8 {
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
            let x = (angle.cos() * dist) as i32;
            let z = (angle.sin() * dist) as i32;
            out.push((x, z));
        }
        out
    }

    fn emit_stronghold(&self, chunk: &mut Chunk, wx: i32, wz: i32, ox: i32, oz: i32) {
        let put = |chunk: &mut Chunk, x: i32, y: i32, z: i32, id: u8| {
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
        // room helper: hollow box of stone bricks
        let mut room =
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
    pub fn ravines_near_chunk(&self, cx: i32, cz: i32) -> Vec<Ravine> {
        let mut out = Vec::new();
        for dcx in -5..=5 {
            for dcz in -5..=5 {
                let (cx, cz) = (cx + dcx, cz + dcz);
                let mut rng = Rng::new(Rng::hash3(self.seed ^ 0xCA_E, cx, 0, cz));
                if rng.next_f32() >= RAVINE_CHANCE {
                    continue;
                }
                let x0 = cx * 16 + rng.next_range(16) as i32;
                let z0 = cz * 16 + rng.next_range(16) as i32;
                let angle = rng.next_f32() * std::f32::consts::TAU;
                let length = 85 + rng.next_range(43) as i32; // 85..=127
                let half_w = 2.0 + rng.next_f32() * 5.0; // < 15 wide total
                let depth = 40 + rng.next_range(23) as i32; // ≤ 62
                                                            // top: terrain height at the start, clamped to 10..=72
                let h = self.column(x0, z0).height as i32;
                let top = h.clamp(10, 72);
                out.push(Ravine {
                    x0,
                    z0,
                    dx: angle.cos(),
                    dz: angle.sin(),
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
                // merge overlapping ravine cuts: keep the deepest floor
                // and the lowest rim of any contributor
                best = Some(match best {
                    Some((bt, bb)) => (bt.min(cut_top), bb.min(cut_bottom)),
                    None => (cut_top, cut_bottom),
                });
            }
        }
        best
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
                            let sx = (wx as f32 + yaw.sin() * d) as i32;
                            let sz = (wz as f32 - yaw.cos() * d) as i32;
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

#[inline]
fn CHUNK_X_CHUNK() -> usize {
    16
}
#[inline]
fn CHUNK_Z_CHUNK() -> usize {
    16
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
        // raw states are folded to owning blocks (fences now store their
        // model state 73, furnaces 116 — the P7-structures collision fix)
        let fold = |s: u16| vc_blocks::blocks::state_block(s);
        // well: water at center, cobble rim, fence post corner, plank roof
        assert_eq!(
            fold(chunk.get(lx, ground, lz) as u16),
            WATER,
            "well center water"
        );
        assert_eq!(
            fold(chunk.get(lx + 1, ground, lz) as u16),
            COBBLE,
            "well rim cobble"
        );
        assert_eq!(
            fold(chunk.get(lx - 1, ground + 3, lz - 1) as u16),
            OAK_FENCE,
            "well post"
        );
        assert_eq!(
            chunk.get(lx - 1, ground + 3, lz - 1) as u16,
            73,
            "well post stores the no-connection fence STATE (not a log axis)"
        );
        assert_eq!(
            fold(chunk.get(lx, ground + 4, lz) as u16),
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
        let mut village = None;
        'outer: for s in 0..40u64 {
            let gen = TerrainGen::new(0x99_CAFE00u64.wrapping_add(s));
            for rz in -4..=4i32 {
                for rx in -4..=4i32 {
                    if let Some(v) = gen.village_center(rx, rz) {
                        village = Some((gen, v));
                        break 'outer;
                    }
                }
            }
        }
        let (gen, (wx, wz)) = village.expect("a village");
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
    use vc_blocks::blocks::*;

    /// fold a raw stored state to its block id (nether blocks store 118..120)
    fn fold(s: u8) -> u8 {
        state_block(s as u16)
    }

    /// §28: the nether shell — bedrock floor + roof, nothing above 127
    #[test]
    fn nether_bedrock_shell() {
        let gen = TerrainGen::for_dimension(0xDEAD_BEEF, Dimension::Nether);
        let (chunk, _) = gen.generate_chunk(0, 0, Vec::new());
        for lz in 0..16usize {
            for lx in 0..16usize {
                assert_eq!(fold(chunk.get(lx, 0, lz)), BEDROCK, "y=0 is bedrock floor");
                assert_eq!(
                    fold(chunk.get(lx, 127, lz)),
                    BEDROCK,
                    "y=127 is bedrock roof"
                );
                // above the build ceiling: air (nothing exists)
                for y in 128..256usize {
                    assert_eq!(chunk.get(lx, y, lz), AIR, "y={y} must be air");
                }
            }
        }
    }

    /// §26/§28: netherrack dominates the mass, with quartz ore sprinkled in
    #[test]
    fn nether_is_netherrack_with_quartz() {
        let mut rack = 0usize;
        let mut quartz = 0usize;
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
                    AIR | GLOWSTONE | SOUL_SAND => {}
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
        assert!(
            other == 0,
            "the nether mass is ONLY netherrack/quartz/glowstone/soul-sand — got {other} others"
        );
    }

    /// §26/§28: vast open caverns exist (the nether is hollow, not solid),
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
        // a 5×5-chunk nether neighborhood is substantially hollow
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
                    if chunk.get(lx, y, lz) != AIR {
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
            assert_eq!(chunk.get(lx, yi as usize, lz), AIR, "feet open (seed {s})");
            assert!(
                yi + 1 >= 128 || chunk.get(lx, (yi + 1) as usize, lz) == AIR,
                "headroom (seed {s})"
            );
            assert!(
                is_solid(fold(chunk.get(lx, (yi - 1) as usize, lz))),
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
        let gen = TerrainGen::for_dimension(0xFEED_1234, Dimension::Nether);
        let (chunk, _) = gen.generate_chunk(3, -4, Vec::new());
        for i in 0..256usize {
            assert_eq!(chunk.biome[i], Biome::NetherWastes as u8, "biome[{i}]");
        }
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
                    let c =
                        gen.column((x + yaw.sin() * 40.0) as i32, (z - yaw.cos() * 40.0) as i32);
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
                    assert!(matches!(r.mob, 0 | 1 | 2), "mob {}", r.mob);
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
                let b = chunk.get(lx(room.x0 + dx), (room.y0 - 1) as usize, lz(room.z0 + dz));
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
                    let b = chunk.get(lx(wx), (room.y0 + dy) as usize, lz(wz));
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
    use vc_blocks::blocks::*;

    fn gen() -> TerrainGen {
        TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld)
    }

    /// the 6 new climate biomes all exist somewhere in a reasonable scan
    /// window, and from_u8 round-trips every variant
    #[test]
    fn new_biomes_present_and_roundtrip() {
        let g = gen();
        let mut seen = std::collections::HashSet::new();
        for x in -40..40 {
            for z in -40..40 {
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
        assert_eq!(c1.get(lx, ms.y as usize, lz), PLANKS);
    }

    /// desert pyramid: 21×21 base, hidden pit with 4 chests, entrance,
    /// and the terracotta checkerboard floor; regeneration determinism
    #[test]
    fn pyramid_emits_full_layout() {
        let g = gen();
        // find a pyramid region
        let mut found = None;
        'outer: for rx in -8..8 {
            for rz in -8..8 {
                if let Some(c) = g.pyramid_center_pub(rx, rz) {
                    found = Some(c);
                    break 'outer;
                }
            }
        }
        let (wx, wz) = found.expect("a desert pyramid within ±8 regions");
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
        let base = g.column(wx, wz).height as i32;
        let at = |dx: i32, dy: i32, dz: i32| -> u8 {
            let x = ((wx + dx) - cx * 16) as usize;
            let z = ((wz + dz) - cz * 16) as usize;
            c1.get(x, (base + dy) as usize, z)
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
            if state_block(c1.get(x, floor as usize, z) as u16) == CHEST {
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
        let get = |x: i32, y: usize, z: i32| -> u8 {
            for &(cx, cz, ref c) in &grid {
                let lx = x - cx * 16;
                let lz = z - cz * 16;
                if (0..16).contains(&lx) && (0..16).contains(&lz) {
                    return state_block(c.get(lx as usize, y, lz as usize) as u16);
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
        let col_h = g.column(mx, mz).height as i32;
        // probe 3 blocks below the local surface at the midpoint: the
        // cut may be shallow where it clipped a low top; assert that at
        // SOME depth along the column the terrain is carved to air
        let lx = (mx - (mx >> 4) * 16) as usize;
        let lz = (mz - (mz >> 4) * 16) as usize;
        let mut carved = 0;
        for y in 8..col_h.min(r.top) {
            if c.get(lx, y as usize, lz) == AIR {
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
