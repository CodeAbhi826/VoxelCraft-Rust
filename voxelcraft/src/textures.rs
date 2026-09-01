//! Procedural 16x16 texture atlas (256x256, 16x16 tiles) in the visual style
//! of Minecraft 1.16.5. Every pixel is synthesized at startup — zero asset files.

use crate::blocks::*;
use crate::rng::Rng;

pub const ATLAS_SIZE: usize = 256;
pub const TILE_PX: usize = 16;

#[inline]
fn put(a: &mut [u8], t: u16, x: i32, y: i32, r: i32, g: i32, b: i32, al: i32) {
    let tx = (t % 16) as i32;
    let ty = (t / 16) as i32;
    if x < 0 || x > 15 || y < 0 || y > 15 {
        return;
    }
    let idx = ((ty * 16 + y) as usize * ATLAS_SIZE + (tx * 16 + x) as usize) * 4;
    a[idx] = r.clamp(0, 255) as u8;
    a[idx + 1] = g.clamp(0, 255) as u8;
    a[idx + 2] = b.clamp(0, 255) as u8;
    a[idx + 3] = al.clamp(0, 255) as u8;
}

/// jitter each channel independently by ±j
#[inline]
fn jit(v: i32, j: i32, rng: &mut Rng) -> i32 {
    (v as f32 + (rng.next_f32() * 2.0 - 1.0) * j as f32) as i32
}

fn noise_fill(a: &mut [u8], t: u16, base: [i32; 3], j: i32, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            put(a, t, x, y, jit(base[0], j, rng), jit(base[1], j, rng), jit(base[2], j, rng), 255);
        }
    }
}

// ------------------------------------------------------------------ tiles --

fn grass_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let mut s = 0;
            if rng.next_f32() < 0.14 {
                s = -22;
            }
            put(a, t, x, y, jit(141 + s, 9, rng), jit(189 + s, 11, rng), jit(89 + s, 9, rng), 255);
        }
    }
}

const DIRT_SHADES: [[i32; 3]; 4] = [
    [134, 96, 67],
    [121, 85, 58],
    [148, 109, 77],
    [110, 78, 52],
];

fn dirt_px(a: &mut [u8], t: u16, x: i32, y: i32, rng: &mut Rng) {
    let s = DIRT_SHADES[(rng.next_range(4)) as usize];
    put(a, t, x, y, jit(s[0], 6, rng), jit(s[1], 5, rng), jit(s[2], 5, rng), 255);
}

fn dirt(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            dirt_px(a, t, x, y, rng);
        }
    }
}

fn grass_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    dirt(a, t, rng);
    for y in 0..4 {
        for x in 0..16 {
            let keep = match y {
                0..=2 => true,
                3 => rng.next_f32() < 0.55,
                _ => rng.next_f32() < 0.15,
            };
            if keep {
                put(a, t, x, y, jit(141, 9, rng), jit(189, 11, rng), jit(89, 9, rng), 255);
            }
        }
    }
}

fn stone(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [127, 127, 127], 5, rng);
    // blobby darker patches
    for _ in 0..7 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let r = 2 + rng.next_range(2) as i32;
        let shade = -(12 + rng.next_range(10) as i32);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r && rng.next_f32() < 0.8 {
                    put(a, t, cx + dx, cy + dy, jit(127 + shade, 4, rng), jit(127 + shade, 4, rng), jit(127 + shade, 4, rng), 255);
                }
            }
        }
    }
}

fn cobble(a: &mut [u8], t: u16, rng: &mut Rng) {
    // jittered-cell "stones" with dark mortar
    let mut seeds = [(0f32, 0f32, 0i32); 7];
    for s in seeds.iter_mut() {
        s.0 = rng.next_f32() * 16.0;
        s.1 = rng.next_f32() * 16.0;
        s.2 = 104 + rng.next_range(42) as i32;
    }
    for y in 0..16 {
        for x in 0..16 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut d1 = 999.0f32;
            let mut d2 = 999.0f32;
            let mut shade = 127i32;
            for s in seeds.iter() {
                let d = (fx - s.0) * (fx - s.0) + (fy - s.1) * (fy - s.1);
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                    shade = s.2;
                } else if d < d2 {
                    d2 = d;
                }
            }
            let edge = (d2.sqrt() - d1.sqrt()) < 1.4;
            let s = if edge { 82 } else { shade };
            put(a, t, x, y, jit(s, 6, rng), jit(s, 6, rng), jit(s, 6, rng), 255);
        }
    }
}

fn sand(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [219, 207, 163], 6, rng);
    for _ in 0..10 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(a, t, x, y, jit(198, 5, rng), jit(186, 5, rng), jit(143, 5, rng), 255);
    }
}

fn gravel(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [127, 122, 118], 8, rng);
    for _ in 0..14 {
        let x = rng.next_range(15) as i32;
        let y = rng.next_range(15) as i32;
        let w = 1 + rng.next_range(2) as i32;
        let h = if w == 2 { 1 } else { 1 + rng.next_range(2) as i32 };
        let s = 92 + rng.next_range(70) as i32;
        for dy in 0..h {
            for dx in 0..w {
                put(a, t, x + dx, y + dy, jit(s, 6, rng), jit(s - 4, 6, rng), jit(s - 7, 6, rng), 255);
            }
        }
    }
}

fn log_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for x in 0..16 {
        let col = match x % 5 {
            0 => -16,
            2 => 8,
            _ => 0,
        };
        for y in 0..16 {
            put(a, t, x, y, jit(107 + col, 5, rng), jit(83 + col, 5, rng), jit(49 + col, 4, rng), 255);
        }
    }
    // knots
    for _ in 0..2 {
        let x = 2 + rng.next_range(12) as i32;
        let y = 2 + rng.next_range(12) as i32;
        for dy in 0..2 {
            for dx in 0..2 {
                put(a, t, x + dx, y + dy, jit(70, 5, rng), jit(54, 4, rng), jit(32, 3, rng), 255);
            }
        }
    }
}

fn log_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let d = ((x as f32 - 7.5).abs()).max((y as f32 - 7.5).abs());
            let (r, g, b) = if d > 6.5 {
                (94, 72, 42)
            } else if ((d as i32) % 2) == 0 {
                (177, 144, 88)
            } else {
                (146, 116, 70)
            };
            put(a, t, x, y, jit(r, 5, rng), jit(g, 5, rng), jit(b, 4, rng), 255);
        }
    }
}

fn planks(a: &mut [u8], t: u16, rng: &mut Rng) {
    let shades: [i32; 4] = [0, -8, 6, -3];
    for y in 0..16usize {
        let plank = shades[y / 4];
        let joint_x = ((y / 4) as i32 * 7 + 3) % 16;
        for x in 0..16usize {
            let mut s = plank;
            if y % 4 == 3 {
                s -= 42; // seam
            }
            if x as i32 == joint_x && y % 4 != 3 {
                s -= 34; // vertical joint
            }
            if rng.next_f32() < 0.06 {
                s -= 14; // grain
            }
            put(a, t, x as i32, y as i32, jit(162 + s, 6, rng), jit(131 + s, 6, rng), jit(79 + s, 5, rng), 255);
        }
    }
}

const LEAF_SHADES: [[i32; 3]; 4] = [
    [39, 64, 26],
    [54, 88, 35],
    [66, 108, 44],
    [77, 126, 51],
];

fn leaves(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            if rng.next_f32() < 0.12 {
                put(a, t, x, y, 0, 0, 0, 0);
            } else {
                let s = LEAF_SHADES[rng.next_range(4) as usize];
                put(a, t, x, y, jit(s[0], 5, rng), jit(s[1], 6, rng), jit(s[2], 5, rng), 255);
            }
        }
    }
}

fn water(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let s = if rng.next_f32() < 0.12 { 14 } else { 0 };
            put(a, t, x, y, jit(63 + s, 7, rng), jit(118 + s, 9, rng), jit(228, 6, rng), 255);
        }
    }
}

fn glass(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            put(a, t, x, y, 0, 0, 0, 0);
        }
    }
    // frame
    for i in 0..16 {
        let s = 255;
        put(a, t, i, 0, 213, 231, 238, s);
        put(a, t, i, 15, 213, 231, 238, s);
        put(a, t, 0, i, 213, 231, 238, s);
        put(a, t, 15, i, 213, 231, 238, s);
    }
    // corner accents
    put(a, t, 1, 1, 240, 248, 250, 255);
    put(a, t, 14, 1, 240, 248, 250, 255);
    put(a, t, 1, 14, 240, 248, 250, 255);
    put(a, t, 14, 14, 240, 248, 250, 255);
    // diagonal streaks
    for i in 0..4 {
        put(a, t, 11 - i, 2 + i, 235, 245, 248, 255);
    }
    for i in 0..3 {
        put(a, t, 13 - i, 8 + i, 235, 245, 248, 255);
    }
    let _ = rng;
}

fn bedrock(a: &mut [u8], t: u16, rng: &mut Rng) {
    // 2x2 clustered high-contrast blobs
    for cy in 0..8 {
        for cx in 0..8 {
            let s = match rng.next_range(4) {
                0 => 38,
                1 => 64,
                2 => 92,
                _ => 122,
            };
            for dy in 0..2 {
                for dx in 0..2 {
                    put(a, t, cx * 2 + dx, cy * 2 + dy, jit(s, 5, rng), jit(s, 5, rng), jit(s, 5, rng), 255);
                }
            }
        }
    }
}

fn snow(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [250, 252, 252], 3, rng);
    for _ in 0..6 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(a, t, x, y, 238, 243, 248, 255);
    }
}

fn snow_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    dirt(a, t, rng);
    for y in 0..4 {
        for x in 0..16 {
            let keep = y < 3 || rng.next_f32() < 0.6;
            if keep {
                put(a, t, x, y, jit(250, 3, rng), jit(252, 3, rng), jit(252, 3, rng), 255);
            }
        }
    }
}

// ---------------------------------------------------------- string art ----

fn art(a: &mut [u8], t: u16, rows: [&str; 16], map: &dyn Fn(char) -> Option<(i32, i32, i32, i32)>) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some((r, g, b, al)) = map(ch) {
                put(a, t, x as i32, y as i32, r, g, b, al);
            }
        }
    }
}

fn tall_grass_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        ".....b....d.....",
        "....cb..d.c.....",
        "....c.b.c.b.....",
        "...d..c.b..d....",
        "...b..d.c..c....",
        "...c.b..b..b....",
        "..d..c..c..c....",
        "..b..b..b..d....",
        "..c..c.d.c.b....",
        "..b.d..b.b.c....",
        "..c.b..c.c.b....",
        ".d..c..b..b.c...",
        ".b..b..c..c.b...",
        ".c..c.d..d..c...",
        "....b....b..b...",
    ];
    art(a, t, rows, &|c| match c {
        'a' => Some((61, 99, 38, 255)),
        'b' => Some((73, 114, 45, 255)),
        'c' => Some((91, 141, 57, 255)),
        'd' => Some((110, 166, 70, 255)),
        _ => None,
    });
}

fn poppy_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "......rr........",
        ".....rRrr.......",
        ".....rRrrr......",
        ".....rrRr.......",
        "......rrr.......",
        "......K.........",
        "......g.........",
        ".....g..........",
        "..G..g..........",
        "...g.g..G.......",
        "....gg.g........",
        ".....g..........",
        "....g.g.........",
        ".....g..........",
    ];
    art(a, t, rows, &|c| match c {
        'r' => Some((178, 48, 48, 255)),
        'R' => Some((206, 66, 54, 255)),
        'K' => Some((40, 20, 20, 255)),
        'g' => Some((62, 96, 32, 255)),
        'G' => Some((74, 112, 38, 255)),
        _ => None,
    });
}

fn dandelion_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "......yy........",
        ".....yYYy.......",
        ".....yYYy.......",
        "......yy........",
        "......g.........",
        "......g.........",
        "......g.........",
        "..G...g.........",
        "...g..g..G......",
        "....g.g.g.......",
        ".....gg.........",
        "......g.........",
        "......g.........",
    ];
    art(a, t, rows, &|c| match c {
        'y' => Some((232, 215, 58, 255)),
        'Y' => Some((250, 240, 88, 255)),
        'g' => Some((62, 96, 32, 255)),
        'G' => Some((74, 112, 38, 255)),
        _ => None,
    });
}

// ------------------------------------------------------------------ entry --

pub fn generate_atlas() -> Vec<u8> {
    let mut a = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
    for t in 0..=TILE_FLOWER_YELLOW {
        let mut rng = Rng::new(0xBE7A5 + t as u64 * 7919);
        match t {
            TILE_GRASS_TOP => grass_top(&mut a, t, &mut rng),
            TILE_GRASS_SIDE => grass_side(&mut a, t, &mut rng),
            TILE_DIRT => dirt(&mut a, t, &mut rng),
            TILE_STONE => stone(&mut a, t, &mut rng),
            TILE_COBBLE => cobble(&mut a, t, &mut rng),
            TILE_SAND => sand(&mut a, t, &mut rng),
            TILE_LOG_SIDE => log_side(&mut a, t, &mut rng),
            TILE_LOG_TOP => log_top(&mut a, t, &mut rng),
            TILE_PLANKS => planks(&mut a, t, &mut rng),
            TILE_LEAVES => leaves(&mut a, t, &mut rng),
            TILE_WATER => water(&mut a, t, &mut rng),
            TILE_GLASS => glass(&mut a, t, &mut rng),
            TILE_BEDROCK => bedrock(&mut a, t, &mut rng),
            TILE_GRAVEL => gravel(&mut a, t, &mut rng),
            TILE_SNOW => snow(&mut a, t, &mut rng),
            TILE_SNOW_SIDE => snow_side(&mut a, t, &mut rng),
            TILE_TALL_GRASS => tall_grass_art(&mut a, t),
            TILE_FLOWER_RED => poppy_art(&mut a, t),
            TILE_FLOWER_YELLOW => dandelion_art(&mut a, t),
            _ => {}
        }
    }
    a
}

/// Blit a tile (scaled) into a UI pixel buffer — used for hotbar icons.
pub fn blit_tile(atlas: &[u8], tile: u16, scale: usize, ox: usize, oy: usize, out: &mut [u8], out_w: usize) {
    let tx = (tile % 16) as usize;
    let ty = (tile / 16) as usize;
    for y in 0..TILE_PX {
        for x in 0..TILE_PX {
            let src = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX + x) * 4;
            for dy in 0..scale {
                for dx in 0..scale {
                    let dxp = ox + x * scale + dx;
                    let dyp = oy + y * scale + dy;
                    let dst = (dyp * out_w + dxp) * 4;
                    if dst + 3 < out.len() {
                        out[dst] = atlas[src];
                        out[dst + 1] = atlas[src + 1];
                        out[dst + 2] = atlas[src + 2];
                        out[dst + 3] = atlas[src + 3];
                    }
                }
            }
        }
    }
}
