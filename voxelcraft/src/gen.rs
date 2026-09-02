//! World generation: simplex noise, fBm, biomes, caves, trees.
//! Fully deterministic per seed; pure functions (safe on worker threads).

use crate::blocks::*;
use crate::chunk::{Chunk, CHUNK_LEN};
use crate::rng::Rng;
use crate::world::Dimension;
use std::sync::Arc;

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
            _ => Biome::Ocean,
        }
    }
}

// ---------------------------------------------------------------- simplex --

const GRAD3: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
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
                i1 = 1.0; j1 = 0.0; k1 = 0.0; i2 = 1.0; j2 = 1.0; k2 = 0.0;
            } else if x0 >= z0 {
                i1 = 1.0; j1 = 0.0; k1 = 0.0; i2 = 1.0; j2 = 0.0; k2 = 1.0;
            } else {
                i1 = 0.0; j1 = 0.0; k1 = 1.0; i2 = 1.0; j2 = 0.0; k2 = 1.0;
            }
        } else {
            if y0 < z0 {
                i1 = 0.0; j1 = 0.0; k1 = 1.0; i2 = 0.0; j2 = 1.0; k2 = 1.0;
            } else if x0 < z0 {
                i1 = 0.0; j1 = 1.0; k1 = 0.0; i2 = 0.0; j2 = 1.0; k2 = 1.0;
            } else {
                i1 = 0.0; j1 = 1.0; k1 = 0.0; i2 = 1.0; j2 = 1.0; k2 = 0.0;
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
            let g = &GRAD3[self.perm_mod12[ii + self.perm[jj + self.perm[kk] as usize] as usize] as usize];
            let t = t0 * t0;
            n += t * t * (g[0] * x0 + g[1] * y0 + g[2] * z0);
        }
        let t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
        if t1 > 0.0 {
            let g = &GRAD3[self.perm_mod12
                [ii + i1 as usize + self.perm[jj + j1 as usize + self.perm[kk + k1 as usize] as usize] as usize]
                as usize];
            let t = t1 * t1;
            n += t * t * (g[0] * x1 + g[1] * y1 + g[2] * z1);
        }
        let t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
        if t2 > 0.0 {
            let g = &GRAD3[self.perm_mod12
                [ii + i2 as usize + self.perm[jj + j2 as usize + self.perm[kk + k2 as usize] as usize] as usize]
                as usize];
            let t = t2 * t2;
            n += t * t * (g[0] * x2 + g[1] * y2 + g[2] * z2);
        }
        let t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
        if t3 > 0.0 {
            let g = &GRAD3[self.perm_mod12[ii + 1 + self.perm[jj + 1 + self.perm[kk + 1] as usize] as usize] as usize];
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
        let mfac = smoothstep(0.15, 0.55, fbm2(&self.n_mfac, (xf + 700.0) / 950.0, (zf - 300.0) / 950.0, 2, 2.0, 0.5));
        let detail = fbm2(&self.n_detail, xf / 60.0, zf / 60.0, 3, 2.0, 0.5) * 4.0;

        let h = (base + ridge * 52.0 * mfac + detail).clamp(8.0, 170.0).floor() as i32;

        let temp = fbm2(&self.n_temp, (xf + 3000.0) / 1700.0, (zf + 3000.0) / 1700.0, 2, 2.0, 0.5);
        let humid = fbm2(&self.n_humid, (xf - 5000.0) / 1400.0, (zf + 5000.0) / 1400.0, 2, 2.0, 0.5);

        let (biome, top, filler) = if h < crate::SEA_LEVEL - 1 {
            if h < crate::SEA_LEVEL - 6 {
                (Biome::Ocean, GRAVEL, GRAVEL)
            } else {
                (Biome::Ocean, SAND, SAND)
            }
        } else if h <= crate::SEA_LEVEL + 1 {
            (Biome::Beach, SAND, SAND)
        } else if h > 96 {
            if h > 112 {
                (Biome::Mountains, SNOW, STONE)
            } else {
                (Biome::Mountains, STONE, STONE)
            }
        } else if temp < -0.32 {
            (Biome::Snowy, SNOW_GRASS, DIRT)
        } else if temp > 0.3 && humid < 0.05 {
            (Biome::Desert, SAND, SAND)
        } else if humid > 0.12 {
            (Biome::Forest, GRASS, DIRT)
        } else {
            (Biome::Plains, GRASS, DIRT)
        };

        ColumnInfo { height: h, biome, top, filler }
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
        let n2 = self.n_cave2.noise3((xf + 800.0) / 110.0, yf / 55.0, (zf - 800.0) / 110.0);
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
        let sea = crate::SEA_LEVEL;
        let mut outbound: Vec<(i32, i32, i32, u8)> = Vec::new();

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
        let mut set_dec = |chunk: &mut Chunk, outbound: &mut Vec<(i32, i32, i32, u8)>, wx: i32, wy: i32, wz: i32, id: u8, replace_leaves: bool| {
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
                Biome::Plains => if rng.next_f32() < 0.5 { 1 } else { 0 },
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
            // species: forest mixes oak + birch; snowy taiga grows spruce
            let biome_here = Biome::from_u8(chunk.biome[col_idx]);
            let (log, leaf) = match biome_here {
                Biome::Snowy => (SPRUCE_LOG, SPRUCE_LEAVES),
                Biome::Forest => {
                    if rng.next_f32() < 0.35 {
                        (BIRCH_LOG, BIRCH_LEAVES)
                    } else {
                        (OAK_LOG, LEAVES)
                    }
                }
                _ => (OAK_LOG, LEAVES),
            };
            let th = if biome_here == Biome::Snowy {
                6 + rng.next_range(3) as i32 // spruce grows taller
            } else {
                4 + rng.next_range(3) as i32 // 4..6
            };
            let y0 = h + 1;

            // canopy: two 5x5 layers, two 3x3 layers (oak/birch);
            // spruce: stacked narrowing rings
            if biome_here == Biome::Snowy {
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
                            set_dec(&mut chunk, &mut outbound, ox + lx + dx, ly, oz + lz + dz, leaf, false);
                        }
                    }
                }
                // spire tip
                set_dec(&mut chunk, &mut outbound, ox + lx, y0 + th, oz + lz, leaf, false);
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
                            set_dec(&mut chunk, &mut outbound, ox + lx + dx, ly, oz + lz + dz, leaf, false);
                        }
                    }
                }
            }
            // trunk
            for ty in 0..th {
                set_dec(&mut chunk, &mut outbound, ox + lx, y0 + ty, oz + lz, log, true);
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
                Biome::Forest => 10,
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
            set_dec(&mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, id, false);
        }

        // mushrooms in forests (shaded floor)
        let mush_attempts = {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            match b {
                Biome::Forest => 6,
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
            let id = if rng.next_f32() < 0.5 { MUSHROOM_RED } else { MUSHROOM_BROWN };
            set_dec(&mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, id, false);
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
                        set_dec(&mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, DEAD_BUSH, false);
                    } else {
                        let ch = 1 + rng.next_range(3) as i32;
                        for dy in 0..ch {
                            set_dec(&mut chunk, &mut outbound, ox + lx, h + 1 + dy, oz + lz, CACTUS, false);
                        }
                    }
                }
                // shallow clay pockets (1.16.5 river/beach clay patches)
                for _ in 0..2 {
                    let lx = rng.next_range(16) as i32;
                    let lz = rng.next_range(16) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if h < crate::SEA_LEVEL - 2 || h > crate::SEA_LEVEL + 1 {
                        continue;
                    }
                    if chunk.get(lx as usize, h as usize, lz as usize) == SAND {
                        chunk.set(lx as usize, h as usize, lz as usize, CLAY);
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

        // ──────────────────────────────── P7 structures: villages ────
        // Deterministic per 24×24-chunk region: each chunk emits ONLY the
        // village blocks falling inside itself (positions are globally
        // derived, so every chunk independently agrees on the layout —
        // no cross-chunk handoff, no generation-order dependence).
        for &(village_wx, village_wz) in self.villages_near(ox, oz).iter() {
            self.emit_village(&mut chunk, village_wx, village_wz, ox, oz);
        }

        (Arc::new(chunk), outbound)
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
                        let n2 = self.n_neth2.noise3((xf + 800.0) / 150.0, yf / 70.0, (zf - 800.0) / 150.0);
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
                    let solid = is_bed || nether[(y.max(0) * 256 + z as i32 * 16 + x as i32) as usize];
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
                let below = if y > 0 { chunk.get(lx as usize, (y - 1) as usize, lz as usize) } else { BEDROCK };
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
                let above = if y < 127 { chunk.get(lx as usize, (y + 1) as usize, lz as usize) } else { BEDROCK };
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
                                let head = if y + 1 < 128 { chunk.get(lx, y + 1, lz) } else { BEDROCK };
                                let floor = if y > 0 { chunk.get(lx, y - 1, lz) } else { BEDROCK };
                                if feet == AIR
                                    && head == AIR
                                    && is_solid(state_block(floor as u16))
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
            if c.height < crate::SEA_LEVEL + 2 || c.height > 96 {
                return None;
            }
            if !matches!(c.biome, Biome::Plains | Biome::Forest | Biome::Snowy | Biome::Mountains) {
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
            if mx - mn > 2 || mn < crate::SEA_LEVEL + 1 {
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
                put(chunk, wx + dx, ground, wz + dz, if edge { COBBLE } else { WATER });
                put(chunk, wx + dx, ground - 1, wz + dz, if edge { COBBLE } else { WATER });
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
            c.height > crate::SEA_LEVEL + 1 && c.biome != Biome::Ocean && c.biome != Biome::Beach
        };
        let mut best: Option<(i32, i32)> = None;
        let mut best_score = i32::MIN;
        'search: for r in 0..40 {
            for i in -r..=r {
                let candidates = [(i, r), (i, -r), (r, i), (-r, i)];
                for &(x, z) in &candidates {
                    let (wx, wz) = (x * 8, z * 8);
                    let col = self.column(wx, wz);
                    if !(col.height > crate::SEA_LEVEL + 1
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
        assert!(found >= 4, "expected several villages across seeds, got {found}");
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
        let fold = |s: u16| crate::blocks::state_block(s);
        // well: water at center, cobble rim, fence post corner, plank roof
        assert_eq!(fold(chunk.get(lx, ground, lz) as u16), WATER, "well center water");
        assert_eq!(fold(chunk.get(lx + 1, ground, lz) as u16), COBBLE, "well rim cobble");
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
        assert_eq!(fold(chunk.get(lx, ground + 4, lz) as u16), PLANKS, "well roof");
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
        for i in 0..crate::chunk::CHUNK_LEN {
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
                let (chunk, _) = gen.generate_chunk(wx.div_euclid(16) + dx, wz.div_euclid(16) + dz, Vec::new());
                for i in 0..crate::chunk::CHUNK_LEN {
                    match chunk.get_idx(i) {
                        crate::blocks::PLANKS => planks += 1,
                        crate::blocks::GLASS => glass += 1,
                        crate::blocks::OAK_LOG => logs += 1,
                        crate::blocks::CRAFTING_TABLE => tables += 1,
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
    use crate::blocks::*;
    use crate::world::Dimension;
    use super::*;

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
                assert_eq!(fold(chunk.get(lx, 127, lz)), BEDROCK, "y=127 is bedrock roof");
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
            for i in 0..crate::chunk::CHUNK_LEN {
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
        assert!(rack > 50_000, "netherrack dominates the mass ({rack} cells)");
        assert!(quartz > 50, "quartz ore appears across seeds ({quartz} cells)");
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
                for i in 0..crate::chunk::CHUNK_LEN {
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
        assert!(ratio > 0.12, "caverns too small: {ratio:.2} open in the mid band");
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
        for i in 0..crate::chunk::CHUNK_LEN {
            assert_eq!(a.get_idx(i), b.get_idx(i), "nether gen must be order-independent at {i}");
        }
        // same world seed, overworld vs nether → different chunks
        let over = TerrainGen::for_dimension(0x1234_ABCD, Dimension::Overworld);
        let (oc, _) = over.generate_chunk(1, 2, Vec::new());
        let mut same = 0;
        for i in 0..crate::chunk::CHUNK_LEN {
            if oc.get_idx(i) == a.get_idx(i) {
                same += 1;
            }
        }
        // air cells match trivially; the mass must differ
        assert!(same < CHUNK_LEN, "dimensions must generate different terrain (same={same})");
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
            let (chunk, _) = gen.generate_chunk(
                xi.div_euclid(16),
                zi.div_euclid(16),
                Vec::new(),
            );
            let lx = (xi - xi.div_euclid(16) * 16) as usize;
            let lz = (zi - zi.div_euclid(16) * 16) as usize;
            assert_eq!(chunk.get(lx, yi as usize, lz), AIR, "feet open (seed {s})");
            assert!(yi + 1 >= 128 || chunk.get(lx, (yi + 1) as usize, lz) == AIR, "headroom (seed {s})");
            assert!(
                is_solid(fold(chunk.get(lx, (yi - 1) as usize, lz))),
                "solid floor below (seed {s})"
            );
            assert!((8..120).contains(&yi), "spawn inside the nether band (seed {s})");
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
            let neighbors_green = (0..12).map(|k| {
                let yaw = k as f32 * std::f32::consts::TAU / 12.0;
                let c = gen.column((x + yaw.sin() * 40.0) as i32, (z - yaw.cos() * 40.0) as i32);
                matches!(c.biome, Biome::Forest | Biome::Plains)
            }).filter(|g| *g).count();
            println!("seed {} -> spawn ({},{},{}) biome {:?} height {} green_neighbors {}/12",
                i, x as i32, y as i32, z as i32, col.biome, col.height, neighbors_green);
            if matches!(col.biome, Biome::Forest | Biome::Plains) || neighbors_green >= 4 {
                green += 1;
            }
            total += 1;
        }
        println!("green-ish spawns: {}/{}", green, total);
        assert!(green >= total / 2, "at least half of seeds should spawn green");
    }
}
