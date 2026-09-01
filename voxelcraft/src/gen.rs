//! World generation: simplex noise, fBm, biomes, caves, trees.
//! Fully deterministic per seed; pure functions (safe on worker threads).

use crate::blocks::*;
use crate::chunk::{Chunk, CHUNK_LEN};
use crate::rng::Rng;
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

pub struct TerrainGen {
    pub seed: u64,
    n_cont: Noise,
    n_mfac: Noise,
    n_ridge: Noise,
    n_detail: Noise,
    n_temp: Noise,
    n_humid: Noise,
    n_cave1: Noise,
    n_cave2: Noise,
    n_cave3: Noise,
}

impl TerrainGen {
    pub fn new(seed: u64) -> Self {
        TerrainGen {
            seed,
            n_cont: Noise::new(seed ^ 0x1000),
            n_mfac: Noise::new(seed ^ 0x2000),
            n_ridge: Noise::new(seed ^ 0x3000),
            n_detail: Noise::new(seed ^ 0x4000),
            n_temp: Noise::new(seed ^ 0x5000),
            n_humid: Noise::new(seed ^ 0x6000),
            n_cave1: Noise::new(seed ^ 0x7000),
            n_cave2: Noise::new(seed ^ 0x8000),
            n_cave3: Noise::new(seed ^ 0x9000),
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

    /// Generate one chunk column. Pure: returns chunk + edits for neighbors
    /// (tree canopies crossing chunk borders).
    pub fn generate_chunk(
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
                        STONE
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
            let cur = chunk.blocks[idx as usize];
            if cur == AIR || (cur == LEAVES && id == OAK_LOG) {
                chunk.blocks[idx as usize] = id;
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
            let th = 4 + rng.next_range(3) as i32; // 4..6
            let y0 = h + 1;

            // canopy: two 5x5 layers, two 3x3 layers
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
                        set_dec(&mut chunk, &mut outbound, ox + lx + dx, ly, oz + lz + dz, LEAVES, false);
                    }
                }
            }
            // trunk
            for ty in 0..th {
                set_dec(&mut chunk, &mut outbound, ox + lx, y0 + ty, oz + lz, OAK_LOG, true);
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

        (Arc::new(chunk), outbound)
    }

    /// Find a comfortable spawn point (land, moderate altitude) near origin.
    pub fn find_spawn(&self) -> (f32, f32, f32) {
        let mut best: Option<(i32, i32)> = None;
        'search: for r in 0..40 {
            for i in -r..=r {
                let candidates = [(i, r), (i, -r), (r, i), (-r, i)];
                for &(x, z) in &candidates {
                    let col = self.column(x * 8, z * 8);
                    if col.height > crate::SEA_LEVEL + 1
                        && col.height < 90
                        && col.biome != Biome::Ocean
                        && col.biome != Biome::Beach
                    {
                        best = Some((x * 8, z * 8));
                        break 'search;
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
