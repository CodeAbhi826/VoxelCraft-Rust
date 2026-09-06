//! Procedural 16x16 texture atlas (256x256, 16x16 tiles) in the visual style
//! of Minecraft 1.16.5. Every pixel is synthesized at startup — zero asset files.

use vc_blocks::blocks::*;
use vc_rng::rng::Rng;

/// Phase E1 procedural tiles (evolution 1.0–1.2 bracket) — child module so
/// the art additions stay reviewable while sharing the put/jit/art helpers.
mod e1_art;

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
            put(
                a,
                t,
                x,
                y,
                jit(base[0], j, rng),
                jit(base[1], j, rng),
                jit(base[2], j, rng),
                255,
            );
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
            put(
                a,
                t,
                x,
                y,
                jit(141 + s, 9, rng),
                jit(189 + s, 11, rng),
                jit(89 + s, 9, rng),
                255,
            );
        }
    }
}

const DIRT_SHADES: [[i32; 3]; 4] = [[134, 96, 67], [121, 85, 58], [148, 109, 77], [110, 78, 52]];

fn dirt_px(a: &mut [u8], t: u16, x: i32, y: i32, rng: &mut Rng) {
    let s = DIRT_SHADES[(rng.next_range(4)) as usize];
    put(
        a,
        t,
        x,
        y,
        jit(s[0], 6, rng),
        jit(s[1], 5, rng),
        jit(s[2], 5, rng),
        255,
    );
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
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(141, 9, rng),
                    jit(189, 11, rng),
                    jit(89, 9, rng),
                    255,
                );
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
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(127 + shade, 4, rng),
                        jit(127 + shade, 4, rng),
                        jit(127 + shade, 4, rng),
                        255,
                    );
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
            put(
                a,
                t,
                x,
                y,
                jit(s, 6, rng),
                jit(s, 6, rng),
                jit(s, 6, rng),
                255,
            );
        }
    }
}

fn sand(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [219, 207, 163], 6, rng);
    for _ in 0..10 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(198, 5, rng),
            jit(186, 5, rng),
            jit(143, 5, rng),
            255,
        );
    }
}

fn gravel(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [127, 122, 118], 8, rng);
    for _ in 0..14 {
        let x = rng.next_range(15) as i32;
        let y = rng.next_range(15) as i32;
        let w = 1 + rng.next_range(2) as i32;
        let h = if w == 2 {
            1
        } else {
            1 + rng.next_range(2) as i32
        };
        let s = 92 + rng.next_range(70) as i32;
        for dy in 0..h {
            for dx in 0..w {
                put(
                    a,
                    t,
                    x + dx,
                    y + dy,
                    jit(s, 6, rng),
                    jit(s - 4, 6, rng),
                    jit(s - 7, 6, rng),
                    255,
                );
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
            put(
                a,
                t,
                x,
                y,
                jit(107 + col, 5, rng),
                jit(83 + col, 5, rng),
                jit(49 + col, 4, rng),
                255,
            );
        }
    }
    // knots
    for _ in 0..2 {
        let x = 2 + rng.next_range(12) as i32;
        let y = 2 + rng.next_range(12) as i32;
        for dy in 0..2 {
            for dx in 0..2 {
                put(
                    a,
                    t,
                    x + dx,
                    y + dy,
                    jit(70, 5, rng),
                    jit(54, 4, rng),
                    jit(32, 3, rng),
                    255,
                );
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
            put(
                a,
                t,
                x,
                y,
                jit(r, 5, rng),
                jit(g, 5, rng),
                jit(b, 4, rng),
                255,
            );
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
            put(
                a,
                t,
                x as i32,
                y as i32,
                jit(162 + s, 6, rng),
                jit(131 + s, 6, rng),
                jit(79 + s, 5, rng),
                255,
            );
        }
    }
}

const LEAF_SHADES: [[i32; 3]; 4] = [[39, 64, 26], [54, 88, 35], [66, 108, 44], [77, 126, 51]];

fn leaves(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            if rng.next_f32() < 0.12 {
                put(a, t, x, y, 0, 0, 0, 0);
            } else {
                let s = LEAF_SHADES[rng.next_range(4) as usize];
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(s[0], 5, rng),
                    jit(s[1], 6, rng),
                    jit(s[2], 5, rng),
                    255,
                );
            }
        }
    }
}

fn water(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let s = if rng.next_f32() < 0.12 { 14 } else { 0 };
            put(
                a,
                t,
                x,
                y,
                jit(63 + s, 7, rng),
                jit(118 + s, 9, rng),
                jit(228, 6, rng),
                255,
            );
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
                    put(
                        a,
                        t,
                        cx * 2 + dx,
                        cy * 2 + dy,
                        jit(s, 5, rng),
                        jit(s, 5, rng),
                        jit(s, 5, rng),
                        255,
                    );
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
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(250, 3, rng),
                    jit(252, 3, rng),
                    jit(252, 3, rng),
                    255,
                );
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

/// §25 redstone wire: a flat red cross laid on the block floor
/// furnace side: stone body with a dark mouth
fn furnace_side_art(a: &mut [u8], t: u16, lit: bool) {
    let rows = [
        "ssssssssssssssss",
        "ssssssssssssssss",
        "ssssssssssssssss",
        "ssssdddddddsssss",
        "sssdddddddddssss",
        "ssdddddddddddsss",
        "ssdddddddddddsss",
        "ssdddddddddddsss",
        "ssdddmmmmmdddsss",
        "ssdddmmmmmdddsss",
        "ssdddddddddddsss",
        "ssdddddddddddsss",
        "sssdddddddddssss",
        "ssssdddddddsssss",
        "ssssssssssssssss",
        "ssssssssssssssss",
    ];
    art(a, t, rows, &|c| match c {
        's' => Some((120, 120, 120, 255)),
        'd' => Some((70, 70, 70, 255)),
        'm' => {
            if lit {
                Some((255, 160, 30, 255))
            } else {
                Some((30, 30, 30, 255))
            }
        }
        _ => None,
    });
}

fn furnace_top_art(a: &mut [u8], t: u16) {
    let rows = [
        "ssssssssssssssss",
        "sdddddddddddddss",
        "sdddddddddddddss",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sddssssssssssdds",
        "sdddddddddddddss",
        "sdddddddddddddss",
        "ssssssssssssssss",
        "ssssssssssssssss",
    ];
    art(a, t, rows, &|c| match c {
        's' => Some((100, 100, 100, 255)),
        'd' => Some((125, 125, 125, 255)),
        _ => None,
    });
}

// nether blocks (Phase 7 §28): our own procedural art, clean-room

/// netherrack: maroon rock — mottled base, darker pits, occasional bright
/// flecks (the "living rock" look)
fn netherrack(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [111, 54, 54], 10, rng);
    for _ in 0..9 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let shade = 60 + rng.next_range(24) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if rng.next_f32() < 0.75 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(shade + 18, 6, rng),
                        jit(shade, 5, rng),
                        jit(shade, 5, rng),
                        255,
                    );
                }
            }
        }
    }
    // sparse warm flecks
    for _ in 0..5 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(178, 14, rng),
            jit(96, 10, rng),
            jit(88, 10, rng),
            255,
        );
    }
}

/// nether quartz ore: netherrack base + cream crystal clusters
fn quartz_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    netherrack(a, t, rng);
    for _ in 0..4 {
        let cx = 2 + rng.next_range(12) as i32;
        let cy = 2 + rng.next_range(12) as i32;
        let shade = 214 + rng.next_range(24) as i32;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let corner = dx.abs() == 1 && dy.abs() == 1;
                if !corner || rng.next_f32() < 0.5 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(shade, 8, rng),
                        jit(shade - 12, 8, rng),
                        jit(shade - 34, 8, rng),
                        255,
                    );
                }
            }
        }
    }
}

/// soul sand: swampy brown with dark pores and sagging swirls
fn soul_sand(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [84, 64, 51], 8, rng);
    // sagging darker striations
    for y in 0..16i32 {
        let band = ((y as f32 / 16.0) * 6.0).sin() * 8.0;
        for x in 0..16i32 {
            if rng.next_f32() < 0.55 {
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(72 + band as i32, 6, rng),
                    jit(55 + band as i32, 6, rng),
                    jit(44 + band as i32, 6, rng),
                    255,
                );
            }
        }
    }
    // the faces-in-the-sand pores
    for _ in 0..6 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
            put(
                a,
                t,
                cx + dx,
                cy + dy,
                jit(44, 6, rng),
                jit(33, 5, rng),
                jit(26, 5, rng),
                255,
            );
        }
    }
    // light speckle so it reads as grain, not mud
    for _ in 0..7 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(118, 10, rng),
            jit(92, 9, rng),
            jit(74, 8, rng),
            255,
        );
    }
}

fn redstone_wire_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "......r........",
        "....rrrrrrrr....",
        "......r........",
        "......r........",
        "......r........",
        "......r....r...",
        "......r..rrrr..",
        "......r..r.....",
        "......r..r.....",
    ];
    art(a, t, rows, &|c| match c {
        'r' => Some((120, 2, 2, 255)),
        _ => None,
    });
}

/// redstone torch: a lit stick with a glowing tip
fn redstone_torch_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".......gg.......",
        ".......gg.......",
        "......gyyg......",
        "......gyyg......",
        ".......yy.......",
        ".......yy.......",
        ".......ww.......",
        ".......ww.......",
        ".......ww.......",
        ".......ww.......",
        ".......ww.......",
        ".......ww.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'g' => Some((200, 40, 20, 255)),
        'y' => Some((255, 150, 40, 255)),
        'w' => Some((120, 84, 48, 255)),
        _ => None,
    });
}

/// brewing stand (Phase 7 §29): a rod with a cross-arm and hint bottles —
/// cross-rendered like redstone components (not a full cube)
fn brewing_stand_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        ".....r...r......",
        ".....r...r......",
        "......rrr.......",
        ".......w........",
        ".......w........",
        ".......w........",
        "....wwwwwww.....",
        ".......w........",
        ".......w........",
        ".......w........",
        "......www.......",
        ".....wwwww......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'r' => Some((150, 40, 50, 255)), // hint bottles (red-ish)
        'w' => Some((120, 84, 48, 255)), // wood rod
        _ => None,
    });
}

/// enchanting table (§29): a dark slab with glowing runes
fn enchant_table_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "...dddddddddd...",
        "...dDDDDDDDDd...",
        "...dDrDDrDDrd...",
        "...dDDrDDrDDd...",
        "...dDrDDDDrd...",
        "...dDDDDDDDDd...",
        "...dddddddddd...",
        "...dd.dddd.dd...",
        "....d..dd..d....",
        "....d..dd..d....",
        "...dd..dd..dd...",
        "...ddddddddd....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'd' => Some((30, 24, 40, 255)),    // obsidian dark
        'D' => Some((58, 44, 74, 255)),    // slab face
        'r' => Some((255, 130, 255, 255)), // glowing runes
        _ => None,
    });
}

/// villager (§27): the classic robe NPC — brown robe, tan face, big nose
/// (drawn from scratch; the tile maps onto a 0.6×1.9 crossed-quad sprite)
fn villager_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "....hhhhhhhh....",
        "....hFFFFFFh....",
        "....hFEFFEFh....",
        "....hFFFFFFh....",
        "....hFFNNFFh....",
        "....hFFNNFFh....",
        "....hFFFFFFh....",
        "....rFFFFFFr....",
        "...rrFFFFFFrr...",
        "...rrrRRRRrrr...",
        "...rrrRRRRrrr...",
        "...rrrRRRRrrr...",
        "...rrRRRRRRrr...",
        "...rrRRRRRRrr...",
        "...rrr....rrr...",
    ];
    art(a, t, rows, &|c| match c {
        'h' => Some((70, 45, 24, 255)),    // hair
        'F' => Some((198, 157, 110, 255)), // face
        'E' => Some((40, 30, 20, 255)),    // eyes
        'N' => Some((180, 125, 85, 255)),  // the big nose
        'r' => Some((102, 60, 32, 255)),   // robe
        'R' => Some((120, 74, 40, 255)),   // robe front
        _ => None,
    });
}

/// enchanted book (§29): a book with a sparkle
fn enchanted_book_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....ss..........",
        "...bBBs.........",
        "...bBBBs........",
        "...bPPPPs.......",
        "...bPPPPPs......",
        "...bPPPPPPs.....",
        "...bPPPPPPPs....",
        "...bPPPPPPPPs...",
        "...bPPPPPPPPs...",
        "...bPPPPPPPPs...",
        "...bbbbbbbbbb...",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'b' => Some((110, 70, 30, 255)),   // leather cover
        'B' => Some((150, 100, 45, 255)),  // cover highlight
        'P' => Some((235, 225, 200, 255)), // pages
        's' => Some((120, 255, 200, 255)), // sparkle
        _ => None,
    });
}

// ---------------------------------------------------- Phase 2 mob art --
//
// All clean-room pixel art: distinct silhouettes and palettes of our own
// design — deliberately NOT recreations of any Mojang sprite. The palette
// choices are generic (undead green, bone white, spotted hide...) which
// are unprotectable style conventions for the creature archetypes.

/// zombie: shambler silhouette, sickly green skin, ragged tunic
fn zombie_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "....GGGGGG......",
        "....GGGGGG......",
        "....GDGGDG......",
        "....GGGGGG......",
        "....GMMMMMG.....",
        "...TTMMMMMTT....",
        "..TTTMMMMMTTT...",
        "..TTTMMMMMTTT...",
        "..TTTMMMMMTTT...",
        "...TTMMMMMTT....",
        "...TTMMMMMTT....",
        "...LL......LL...",
        "...LL......LL...",
        "...LL......LL...",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((96, 140, 86, 255)), // skin
        'D' => Some((30, 42, 26, 255)),  // dark eyes
        'M' => Some((70, 100, 62, 255)), // mouth strip
        'T' => Some((60, 84, 110, 255)), // tunic
        'L' => Some((52, 62, 90, 255)),  // legs
        _ => None,
    });
}

/// skeleton: bone frame, hollow eyes, bow arm
fn skeleton_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "....WWWWWW......",
        "....WWWWWW......",
        "....WDWWDW......",
        "....WWWWWW......",
        "....WWNNWW......",
        "...BWWWWWWB.....",
        "..BBWWWWWWBB....",
        "...BWWWWWWB.....",
        "....WWWWWW......",
        "....W....W......",
        "....W....W......",
        "....W....W......",
        "....W....W......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((212, 210, 200, 255)), // bone
        'D' => Some((40, 40, 38, 255)),    // hollow eyes
        'N' => Some((120, 118, 110, 255)), // nasal
        'B' => Some((150, 110, 60, 255)),  // bow limb
        _ => None,
    });
}

/// creeper: tall body, four stubby feet, sad face
fn creeper_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....CCCCCCC.....",
        "....CCDCCDC.....",
        "....CCDCCDC.....",
        "....CCCCCCCC....",
        "....CCNNNNCC....",
        "....CCNNNNCC....",
        "....CCCCCCCC....",
        "....CC.CC.CC....",
        "....CC.CC.CC....",
        "....CC.CC.CC....",
        "...CCC.CC.CCC...",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'C' => Some((88, 190, 96, 255)), // plant-flesh
        'D' => Some((20, 40, 22, 255)),  // droopy eyes
        'N' => Some((28, 58, 30, 255)),  // frowning mouth
        _ => None,
    });
}

/// spider: wide low body + eight leg tips at the corners
fn spider_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "..L..........L..",
        "..LL.BBBBBB.LL..",
        "...LLBBBBBBLL...",
        "...LBBBBBBBBL...",
        "..LBBRDBBDRBBL..",
        "..LBBBBBBBBBBL..",
        "...LBBBBBBBBL...",
        "...LLB BBBBLL...",
        "....LL....LL....",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((58, 44, 40, 255)),  // cephalothorax/abdomen
        'R' => Some((150, 40, 40, 255)), // red eye specks
        'D' => Some((226, 60, 60, 255)), // brighter eyes
        'L' => Some((44, 34, 32, 255)),  // legs
        _ => None,
    });
}

/// enderman: extremely tall thin dark frame, glowing eyes
fn enderman_art(a: &mut [u8], t: u16) {
    let rows = [
        "....KKKKKK......",
        "....KKKKKK......",
        "....VEVVEV......",
        "....KKKKKK......",
        "....KKKKKK......",
        "...K.KKKK.K.....",
        "...K.KKKK.K.....",
        "...K.KKKK.K.....",
        "...K.KKKK.K.....",
        "...K.KKKK.K.....",
        "...K.KKKK.K.....",
        "......KK........",
        "......KK........",
        "......KK........",
        "......KK........",
        "......KK........",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((18, 16, 24, 255)),    // void-black body
        'V' => Some((206, 220, 255, 255)), // pale violet-white eyes
        'E' => Some((150, 90, 220, 255)),  // violet eye core
        _ => None,
    });
}

/// cow: blocky body with hide patches, pale horns
fn cow_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "..H...........H.",
        "...BBBBBBBBBB...",
        "...BWBBBPPPBB...",
        "...BBWBPPPPBB...",
        "...BBBBPPWBBB...",
        "...BBBBBBBBBB...",
        "..BBBBBBBBBBBB..",
        "..BBBBBBBBBBBB..",
        "..BBBBBBBBBBBB..",
        "..BBBBBBBBBBBB..",
        "...L.L....L.L...",
        "...L.L....L.L...",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((72, 54, 40, 255)),    // brown hide
        'W' => Some((232, 228, 220, 255)), // white patches
        'P' => Some((90, 70, 58, 255)),    // darker patches
        'L' => Some((58, 44, 34, 255)),    // legs
        'H' => Some((220, 214, 200, 255)), // horns
        _ => None,
    });
}

/// pig: round pink body, snout
fn pig_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....PPPPPPP.....",
        "....PPPPPPP.....",
        "....PNPPPNP.....",
        "....PPPPPddS....",
        "...PPPPPPPPP....",
        "..PPPPPPPPPPP...",
        "..PPPPPPPPPPP...",
        "..PPPPPPPPPPP...",
        "..PPPPPPPPPPP...",
        "...L.L..L.L.....",
        "...L.L..L.L.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((238, 160, 158, 255)), // pink hide
        'N' => Some((120, 74, 70, 255)),   // nostril-line eyes
        'd' => Some((190, 110, 108, 255)), // darker snout
        'S' => Some((198, 120, 118, 255)), // snout side
        'L' => Some((210, 130, 128, 255)), // legs
        _ => None,
    });
}

/// sheep: wool cloud body, wool cap, bare face
fn sheep_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....WWWWWW......",
        "....WFFFFW......",
        "....WFDFFW......",
        "...WWFFFFWWW....",
        "..WWWWWWWWWW....",
        "..WWWWWWWWWW....",
        "..WWWWWWWWWW....",
        "..WWWWWWWWWW....",
        "..WWWWWWWWWW....",
        "...L.L..L.L.....",
        "...L.L..L.L.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((234, 230, 224, 255)), // wool
        'F' => Some((222, 210, 196, 255)), // face
        'D' => Some((40, 36, 32, 255)),    // eyes
        'L' => Some((200, 192, 182, 255)), // legs
        _ => None,
    });
}

/// chicken: small body, red comb, yellow beak
fn chicken_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".....RRR........",
        "....WWWWWR......",
        "....WDWWRYY.....",
        "....WWWW........",
        "...WWWWWWW......",
        "...WWWWWWWW.....",
        "...WWWWWWWW.....",
        "....WWWWWW......",
        ".....Y..Y.......",
        ".....Y..Y.......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((240, 240, 236, 255)), // feathers
        'R' => Some((206, 60, 54, 255)),   // comb/wattle
        'D' => Some((30, 30, 28, 255)),    // eye
        'Y' => Some((230, 180, 60, 255)),  // beak/feet
        _ => None,
    });
}

/// arrow projectile: slim diagonal shaft with head + fletching
fn arrow_art(a: &mut [u8], t: u16) {
    let rows = [
        "..............F.",
        ".............FF.",
        "............FF..",
        "...........HS...",
        "..........HS....",
        ".........HS.....",
        "........HS......",
        ".......HS.......",
        "......HS........",
        ".....HS.........",
        "....HS..........",
        "...FS...........",
        "..FF............",
        "..F.............",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((220, 220, 216, 255)), // fletching
        'H' => Some((180, 184, 190, 255)), // head
        'S' => Some((140, 100, 50, 255)),  // shaft
        _ => None,
    });
}

/// generic raw-meat icon: shape shared, per-animal palette
fn meat_art(a: &mut [u8], t: u16, main: (i32, i32, i32), dark: (i32, i32, i32)) {
    let rows = [
        "................",
        "................",
        "................",
        "...MMMMMMM......",
        "..MMMMMMMMM.....",
        "..MMDDMMDDMM....",
        "..MMDDMMDDMM....",
        "..MMMMMMMMMM....",
        "..MMMMMMMMMM....",
        "...MMMMMMMMM....",
        "....MMMMMMM.....",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((main.0, main.1, main.2, 255)),
        'D' => Some((dark.0, dark.1, dark.2, 255)),
        _ => None,
    });
}

fn feather_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "............W...",
        "...........WW...",
        "..........WWW...",
        ".........WWW....",
        "........WWW.....",
        ".......WWW......",
        "......WWW.......",
        ".....WWW........",
        "....WWW.........",
        "....W...........",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((240, 240, 244, 255)),
        _ => None,
    });
}

fn leather_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "...LLLLLLLL.....",
        "..LLBLLLLBLL....",
        "..LLLLLLLLLL....",
        "..LLLLBLLLLL....",
        "..LLLLLLLLLL....",
        "...LLLLLLLL.....",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'L' => Some((140, 90, 42, 255)),
        'B' => Some((112, 70, 32, 255)),
        _ => None,
    });
}

fn bone_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "..WW........WW..",
        "...WW......WW...",
        "....WWWWWWWW....",
        ".....WSSSSW.....",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....WSSSSW.....",
        "....WWWWWWWW....",
        "...WW......WW...",
        "..WW........WW..",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((230, 226, 214, 255)),
        'S' => Some((208, 204, 192, 255)),
        _ => None,
    });
}

fn string_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "..SS..........S.",
        "...SS........S..",
        "....SS......SS..",
        ".....SS....SS...",
        "......SS..SS....",
        ".......SSSS.....",
        "......SS..SS....",
        ".....SS....SS...",
        "....SS......SS..",
        "...S........SS..",
        "..S..........S..",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((234, 234, 230, 255)),
        _ => None,
    });
}

fn gunpowder_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "....GGGGGG......",
        "..GGDGGGGDGG....",
        "..GGGGDDGGGG....",
        "..GDGGGGGGDG....",
        "..GGGGGGGGGG....",
        "...GGDDDDGG.....",
        "....GGGGGG......",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((86, 84, 82, 255)),
        'D' => Some((54, 52, 50, 255)),
        _ => None,
    });
}

fn ender_pearl_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "......KKKK......",
        "....KKVVVVK K...",
        "....KVVKKVVK....",
        "...KKVKKKKVKK...",
        "...KKVKKKKVKK...",
        "...KKKVVVVKKK...",
        "....KKVVVKKK....",
        "......KKKK......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((22, 34, 44, 255)),
        'V' => Some((110, 200, 190, 255)),
        _ => None,
    });
}

fn rotten_flesh_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "...MMMMMMM......",
        "..MMGMMMMGM.....",
        "..MMMGMMGMMM....",
        "..MGGMMMMMMM....",
        "..MMMMMGGMMM....",
        "...MMMMMMMMM....",
        "....MMMMMM......",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((150, 96, 76, 255)),
        'G' => Some((110, 130, 70, 255)),
        _ => None,
    });
}

/// spider eye (Phase 4): a clean-room eye item icon — red sclera,
/// dark pupil, off-center glint (ours, not Mojang's)
fn spider_eye_art(a: &mut [u8], t: u16, fermented: bool) {
    let rows = [
        "................",
        "................",
        "................",
        ".....RRRR.......",
        "....RRrrRR......",
        "...RrrPPrrR.....",
        "...RrPWWPrR.....",
        "...RrPWWPrR.....",
        "...RrrPPrrR.....",
        "....RRrrRR......",
        ".....RRRR.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    // fresh eye: saturated red with a dark pupil; fermented: dulled
    // brown-red (the fermentation tints it) + mold speckles on the rim
    let (sclera, rim, mold) = if fermented {
        ((122, 62, 44), (86, 44, 32), (96, 116, 58))
    } else {
        ((168, 34, 30), (128, 22, 20), (128, 22, 20))
    };
    art(a, t, rows, &|c| match c {
        'R' => Some((sclera.0, sclera.1, sclera.2, 255)),
        'r' => Some((rim.0, rim.1, rim.2, 255)),
        'P' => Some((20, 12, 10, 255)),  // pupil
        'W' => Some((60, 150, 90, 255)), // iris glint (greenish)
        _ => None,
    });
    if fermented {
        // mold speckles: a few green dots scattered on the rim rows
        let tx = (t % 16) as usize;
        let ty = (t / 16) as usize;
        for (dy, dx) in [(3usize, 5usize), (6, 3), (8, 11), (4, 9)] {
            let i = ((ty * TILE_PX + dy) * ATLAS_SIZE + tx * TILE_PX + dx) * 4;
            if i + 3 < a.len() {
                a[i] = mold.0;
                a[i + 1] = mold.1;
                a[i + 2] = mold.2;
                a[i + 3] = 255;
            }
        }
    }
}

/// spawner (Phase 5 §27): clean-room monster cage face — dark steel
/// lattice bars with an open mesh (ours, not Mojang's): horizontal rails
/// + vertical bars, rivet dots on the frame
fn spawner_art(a: &mut [u8], t: u16) {
    let rows = [
        "FFFFFFFFFFFFFFFF",
        "F..............F",
        "FB.B.B..B.B.B..F",
        "F..............F",
        "FFB.B.B..B.B.BFF",
        "F..............F",
        "FB.B.B..B.B.B..F",
        "F..............F",
        "FFB.B.B..B.B.BFF",
        "F..............F",
        "FB.B.B..B.B.B..F",
        "F..............F",
        "FBB.B..B.B.B.BBF",
        "F..............F",
        "FFFFFFFFFFFFFFFF",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((28, 30, 36, 255)), // frame rail
        'B' => Some((54, 58, 66, 255)), // bar
        _ => Some((12, 13, 16, 255)),   // mesh hole (dark, not cut out — full cube)
    });
}

/// end-portal frame (Phase 10): clean-room portal-room frame block —
/// end-stone pale base with a recessed dark ring inset (the eye socket)
/// + green glint corners (ours, not Mojang's)
fn end_portal_frame_art(a: &mut [u8], t: u16) {
    let rows = [
        "S.S..........S.S",
        "................",
        ".RRRRRRRRRRRRRR.",
        ".R............R.",
        ".R.D........D.R.",
        ".R..D......D..R.",
        ".R............R.",
        ".R..G......G..R.",
        ".R..G......G..R.",
        ".R............R.",
        ".R..D......D..R.",
        ".R.D........D.R.",
        ".R............R.",
        ".RRRRRRRRRRRRRR.",
        "................",
        "S.S..........S.S",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((96, 116, 112, 255)), // end-stone pale speck
        'R' => Some((74, 92, 88, 255)),   // ring rail
        'D' => Some((30, 38, 40, 255)),   // socket recess
        'G' => Some((70, 160, 130, 255)), // green glint
        _ => Some((120, 140, 134, 255)),  // base
    });
}

/// potion bottle (Phase 7 §29): one bottle shape, per-potion liquid color.
/// `glow` brightens the liquid (the level-II variant sparkles).
fn potion_art(a: &mut [u8], t: u16, liquid: (i32, i32, i32), glow: bool) {
    let rows = [
        "................",
        "................",
        "................",
        ".......c........",
        ".......c........",
        "......ggg.......",
        "......gGg.......",
        "......lLl.......",
        "......lLl.......",
        ".....glLlg......",
        ".....glLlg......",
        ".....glLlg......",
        ".....glllg......",
        "......ggg.......",
        "................",
        "................",
    ];
    let (r, g, b) = liquid;
    let (r, g, b) = if glow {
        (r + 60, g + 60, b + 60)
    } else {
        (r, g, b)
    };
    art(a, t, rows, &|ch| match ch {
        'c' => Some((150, 116, 68, 255)),  // cork
        'g' => Some((200, 220, 235, 190)), // glass (translucent)
        'G' => Some((235, 245, 250, 220)), // glass highlight
        'l' => Some((r.clamp(0, 255), g.clamp(0, 255), b.clamp(0, 255), 235)),
        'L' => Some((
            (r + 40).clamp(0, 255),
            (g + 40).clamp(0, 255),
            (b + 40).clamp(0, 255),
            245,
        )),
        _ => None,
    });
}

/// lever: a wooden base with a diagonal handle
fn lever_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "..........hh....",
        ".........hh.....",
        "........hh......",
        ".......hh.......",
        "......hh........",
        ".....bb.........",
        "....bbbbb.......",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'h' => Some((140, 100, 60, 255)),
        'b' => Some((90, 64, 40, 255)),
        _ => None,
    });
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

// --------------------------------------------------- extended tile set --
// Stone family, ores, mineral blocks, misc, wool, wood variants, plants.

fn granite(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [149, 103, 85], 7, rng);
    for _ in 0..6 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let shade = 125 + rng.next_range(30) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if rng.next_f32() < 0.7 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(shade, 5, rng),
                        jit(shade - 42, 4, rng),
                        jit(shade - 60, 4, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn diorite(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [189, 188, 189], 6, rng);
    for _ in 0..8 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let dark = rng.next_f32() < 0.5;
        let shade: i32 = if dark { 78 } else { 235 };
        for dy in -1..=1 {
            for dx in -1..=1 {
                if rng.next_f32() < 0.6 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(shade, 5, rng),
                        jit(shade, 5, rng),
                        jit(shade, 5, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn andesite(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [136, 136, 137], 6, rng);
    for _ in 0..5 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let shade = 112 + rng.next_range(18) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if rng.next_f32() < 0.5 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(shade, 4, rng),
                        jit(shade, 4, rng),
                        jit(shade + 1, 4, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn stone_bricks(a: &mut [u8], t: u16, rng: &mut Rng) {
    let brick = [122, 121, 121];
    let mortar = [88, 87, 87];
    for y in 0..16 {
        for x in 0..16 {
            // rows of 8x4 bricks offset every other row (row h=4)
            let row = y / 4;
            let offset = if row % 2 == 0 { 0 } else { 4 };
            let in_mortar_h = y % 4 == 3;
            let in_mortar_v = (x + offset) % 8 == 7;
            let s = if in_mortar_h || in_mortar_v {
                mortar
            } else {
                brick
            };
            put(
                a,
                t,
                x,
                y,
                jit(s[0], 5, rng),
                jit(s[1], 5, rng),
                jit(s[2], 5, rng),
                255,
            );
        }
    }
}

fn bricks(a: &mut [u8], t: u16, rng: &mut Rng) {
    let brick = [151, 97, 83];
    let mortar = [139, 133, 126];
    for y in 0..16 {
        for x in 0..16 {
            let row = y / 4;
            let offset = if row % 2 == 0 { 0 } else { 4 };
            let in_mortar_h = y % 4 == 3;
            let in_mortar_v = (x + offset) % 8 == 7;
            let s = if in_mortar_h || in_mortar_v {
                mortar
            } else {
                brick
            };
            put(
                a,
                t,
                x,
                y,
                jit(s[0], 6, rng),
                jit(s[1], 5, rng),
                jit(s[2], 5, rng),
                255,
            );
        }
    }
}

fn mossy_cobble(a: &mut [u8], t: u16, rng: &mut Rng) {
    cobble(a, t, rng);
    // moss blotches
    for _ in 0..7 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let r = 1 + rng.next_range(2) as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r && rng.next_f32() < 0.55 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(88, 8, rng),
                        jit(120, 10, rng),
                        jit(62, 7, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn smooth_stone(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [160, 160, 160], 4, rng);
    // faint horizontal strata
    for y in 0..16 {
        if y % 5 == 2 {
            for x in 0..16 {
                let s = 148 + rng.next_range(8) as i32;
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(s, 3, rng),
                    jit(s, 3, rng),
                    jit(s, 3, rng),
                    255,
                );
            }
        }
    }
}

fn obsidian(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [21, 18, 32], 5, rng);
    // purple sparkles
    for _ in 0..9 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        let bright = rng.next_f32() < 0.4;
        if bright {
            put(
                a,
                t,
                x,
                y,
                jit(136, 20, rng),
                jit(96, 15, rng),
                jit(182, 20, rng),
                255,
            );
        } else {
            put(
                a,
                t,
                x,
                y,
                jit(58, 10, rng),
                jit(42, 8, rng),
                jit(84, 12, rng),
                255,
            );
        }
    }
}

/// ore helper: stone base + clustered colored blobs
fn ore_blob(a: &mut [u8], t: u16, rng: &mut Rng, colors: &[[i32; 3]; 3], attempts: usize) {
    stone(a, t, rng);
    for _ in 0..attempts {
        let cx = 1 + rng.next_range(13) as i32;
        let cy = 1 + rng.next_range(13) as i32;
        let r = 1 + rng.next_range(2) as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() + dy.abs() <= r && rng.next_f32() < 0.85 {
                    let c = colors[rng.next_range(3) as usize];
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(c[0], 8, rng),
                        jit(c[1], 8, rng),
                        jit(c[2], 8, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn coal_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(a, t, rng, &[[28, 28, 28], [45, 45, 45], [16, 16, 16]], 5);
}
fn iron_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(
        a,
        t,
        rng,
        &[[216, 175, 147], [175, 142, 120], [229, 190, 159]],
        5,
    );
}
fn gold_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(
        a,
        t,
        rng,
        &[[250, 238, 77], [222, 206, 66], [253, 246, 130]],
        4,
    );
}
fn diamond_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(
        a,
        t,
        rng,
        &[[93, 236, 245], [61, 214, 224], [140, 247, 252]],
        4,
    );
}
fn redstone_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(a, t, rng, &[[255, 40, 40], [190, 24, 24], [232, 70, 70]], 5);
}
fn lapis_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(
        a,
        t,
        rng,
        &[[38, 97, 214], [28, 71, 160], [61, 121, 232]],
        5,
    );
}
fn emerald_ore(a: &mut [u8], t: u16, rng: &mut Rng) {
    ore_blob(
        a,
        t,
        rng,
        &[[23, 217, 101], [17, 168, 77], [58, 232, 128]],
        3,
    );
}

fn metal_block(a: &mut [u8], t: u16, rng: &mut Rng, base: [i32; 3]) {
    for y in 0..16 {
        for x in 0..16 {
            let border = x == 0 || y == 0 || x == 15 || y == 15;
            let inner_frame = x == 1 || y == 1 || x == 14 || y == 14;
            let mut s = base;
            if border {
                s = [base[0] - 28, base[1] - 28, base[2] - 28];
            } else if inner_frame {
                s = [base[0] + 14, base[1] + 14, base[2] + 14];
            }
            put(
                a,
                t,
                x,
                y,
                jit(s[0], 4, rng),
                jit(s[1], 4, rng),
                jit(s[2], 4, rng),
                255,
            );
        }
    }
}

fn iron_block(a: &mut [u8], t: u16, rng: &mut Rng) {
    metal_block(a, t, rng, [220, 220, 220]);
}
fn gold_block(a: &mut [u8], t: u16, rng: &mut Rng) {
    metal_block(a, t, rng, [247, 233, 76]);
}
fn diamond_block(a: &mut [u8], t: u16, rng: &mut Rng) {
    metal_block(a, t, rng, [98, 231, 231]);
}

fn glowstone(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [144, 106, 51], 8, rng);
    // bright glowing pocks
    for _ in 0..12 {
        let cx = rng.next_range(16) as i32;
        let cy = rng.next_range(16) as i32;
        let r = 1 + rng.next_range(2) as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r && rng.next_f32() < 0.8 {
                    put(
                        a,
                        t,
                        cx + dx,
                        cy + dy,
                        jit(255, 4, rng),
                        jit(222, 12, rng),
                        jit(140, 16, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn bookshelf_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    planks(a, t, rng);
    // two shelf rows of colored book spines
    let spine_colors: [[i32; 3]; 6] = [
        [161, 64, 50],
        [62, 98, 168],
        [96, 138, 66],
        [170, 130, 60],
        [120, 70, 140],
        [200, 190, 170],
    ];
    for (row, y0) in [2i32, 9i32].iter().enumerate() {
        let y0 = *y0;
        let mut x = 1;
        while x < 15 {
            let w = 1 + rng.next_range(3) as i32;
            let c = spine_colors[rng.next_range(6) as usize];
            for dy in 0..5 {
                for dx in 0..w {
                    if x + dx < 15 {
                        put(
                            a,
                            t,
                            x + dx,
                            y0 + dy,
                            jit(c[0], 10, rng),
                            jit(c[1], 10, rng),
                            jit(c[2], 10, rng),
                            255,
                        );
                    }
                }
            }
            x += w;
        }
        let _ = row;
    }
}

fn bookshelf_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    planks(a, t, rng);
}

fn craft_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    planks(a, t, rng);
    // 3x3 grid of darker squares
    for gy in 0..3 {
        for gx in 0..3 {
            let x0 = 2 + gx * 4;
            let y0 = 2 + gy * 4;
            for dy in 0..3 {
                for dx in 0..3 {
                    let s = 138 + rng.next_range(14) as i32;
                    put(
                        a,
                        t,
                        x0 + dx,
                        y0 + dy,
                        jit(s, 4, rng),
                        jit(s - 14, 4, rng),
                        jit(s - 30, 4, rng),
                        255,
                    );
                }
            }
        }
    }
}

fn craft_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    planks(a, t, rng);
    // tool silhouettes hanging
    for y in 6..10 {
        put(
            a,
            t,
            4,
            y,
            jit(90, 6, rng),
            jit(70, 5, rng),
            jit(44, 4, rng),
            255,
        );
        put(
            a,
            t,
            10,
            y,
            jit(120, 6, rng),
            jit(100, 5, rng),
            jit(60, 4, rng),
            255,
        );
    }
}

fn clay(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [160, 166, 179], 5, rng);
    for _ in 0..5 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(141, 5, rng),
            jit(147, 5, rng),
            jit(160, 5, rng),
            255,
        );
    }
}

fn terracotta(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [152, 94, 67], 6, rng);
    for _ in 0..6 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(141, 6, rng),
            jit(85, 5, rng),
            jit(60, 5, rng),
            255,
        );
    }
}

fn pumpkin_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for x in 0..16 {
        let lobe = if x % 5 == 0 { -26 } else { 0 };
        for y in 0..16 {
            let top = y < 2;
            let s = if top { lobe - 30 } else { lobe };
            put(
                a,
                t,
                x,
                y,
                jit(208 + s, 8, rng),
                jit(126 + s, 7, rng),
                jit(26 + s / 2, 5, rng),
                255,
            );
        }
    }
}

fn pumpkin_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [196, 118, 28], 6, rng);
    // stem
    for dy in 0..2 {
        for dx in 0..2 {
            put(
                a,
                t,
                7 + dx,
                7 + dy,
                jit(124, 8, rng),
                jit(96, 6, rng),
                jit(30, 5, rng),
                255,
            );
        }
    }
    // ridge rings
    for i in 0..16 {
        put(
            a,
            t,
            i,
            3,
            jit(186, 6, rng),
            jit(110, 5, rng),
            jit(24, 4, rng),
            255,
        );
        put(
            a,
            t,
            i,
            12,
            jit(186, 6, rng),
            jit(110, 5, rng),
            jit(24, 4, rng),
            255,
        );
    }
}

fn melon_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    // vertical light/dark green stripes
    for x in 0..16 {
        let s = if (x / 2) % 2 == 0 { 0 } else { -34 };
        for y in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(112 + s, 7, rng),
                jit(172 + s, 8, rng),
                jit(72 + s / 2, 6, rng),
                255,
            );
        }
    }
}

fn melon_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [112, 172, 72], 6, rng);
    for _ in 0..8 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(88, 6, rng),
            jit(146, 7, rng),
            jit(58, 5, rng),
            255,
        );
    }
}

fn ice(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let shard = ((x + y * 2) / 5) % 2 == 0;
            let s = if shard { 16 } else { 0 };
            put(
                a,
                t,
                x,
                y,
                jit(160 + s, 5, rng),
                jit(198 + s, 5, rng),
                jit(246, 3, rng),
                190,
            );
        }
    }
}

fn cactus_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for x in 0..16 {
        let edge = x < 2 || x > 13;
        let s = if edge { -26 } else { 0 };
        for y in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(14 + s, 4, rng),
                jit(124 + s, 7, rng),
                jit(36 + s / 2, 4, rng),
                255,
            );
        }
    }
    // spines
    for _ in 0..10 {
        let x = 2 + rng.next_range(12) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(230, 6, rng),
            jit(230, 6, rng),
            jit(180, 8, rng),
            255,
        );
    }
}

fn cactus_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [24, 134, 46], 5, rng);
    for _ in 0..8 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            jit(220, 8, rng),
            jit(220, 8, rng),
            jit(170, 10, rng),
            255,
        );
    }
}

fn wool(a: &mut [u8], t: u16, rng: &mut Rng, base: [i32; 3]) {
    // woven cross-hatch
    for y in 0..16 {
        for x in 0..16 {
            let weave = if (x / 3 + y / 3) % 2 == 0 { 10 } else { -10 };
            let bump = (x % 3 == 0 || y % 3 == 0) as i32 * 6;
            put(
                a,
                t,
                x,
                y,
                jit(base[0] + weave + bump, 5, rng),
                jit(base[1] + weave + bump, 5, rng),
                jit(base[2] + weave + bump, 5, rng),
                255,
            );
        }
    }
}

fn wool_white(a: &mut [u8], t: u16, rng: &mut Rng) {
    wool(a, t, rng, [233, 236, 236]);
}
fn wool_red(a: &mut [u8], t: u16, rng: &mut Rng) {
    wool(a, t, rng, [160, 39, 34]);
}
fn wool_blue(a: &mut [u8], t: u16, rng: &mut Rng) {
    wool(a, t, rng, [53, 57, 157]);
}
fn wool_yellow(a: &mut [u8], t: u16, rng: &mut Rng) {
    wool(a, t, rng, [254, 216, 59]);
}
fn wool_black(a: &mut [u8], t: u16, rng: &mut Rng) {
    wool(a, t, rng, [33, 35, 38]);
}

fn birch_log_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [216, 215, 206], 5, rng);
    // dark dashes
    for _ in 0..7 {
        let x = rng.next_range(14) as i32;
        let y = rng.next_range(16) as i32;
        let w = 2 + rng.next_range(3) as i32;
        for dx in 0..w {
            put(
                a,
                t,
                x + dx,
                y,
                jit(64, 8, rng),
                jit(62, 8, rng),
                jit(58, 8, rng),
                255,
            );
        }
    }
}

const BIRCH_LEAF_SHADES: [[i32; 3]; 4] =
    [[106, 156, 66], [88, 138, 55], [120, 168, 75], [78, 124, 48]];

fn birch_leaves(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            if rng.next_f32() < 0.12 {
                put(a, t, x, y, 0, 0, 0, 0);
            } else {
                let s = BIRCH_LEAF_SHADES[rng.next_range(4) as usize];
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(s[0], 5, rng),
                    jit(s[1], 6, rng),
                    jit(s[2], 5, rng),
                    255,
                );
            }
        }
    }
}

fn spruce_log_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for x in 0..16 {
        let col = match x % 5 {
            0 => -18,
            2 => 6,
            _ => 0,
        };
        for y in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(64 + col, 5, rng),
                jit(46 + col, 4, rng),
                jit(26 + col, 3, rng),
                255,
            );
        }
    }
}

const SPRUCE_LEAF_SHADES: [[i32; 3]; 4] = [[50, 90, 58], [38, 74, 47], [60, 104, 66], [32, 62, 40]];

fn spruce_leaves(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            if rng.next_f32() < 0.10 {
                put(a, t, x, y, 0, 0, 0, 0);
            } else {
                let s = SPRUCE_LEAF_SHADES[rng.next_range(4) as usize];
                put(
                    a,
                    t,
                    x,
                    y,
                    jit(s[0], 5, rng),
                    jit(s[1], 6, rng),
                    jit(s[2], 5, rng),
                    255,
                );
            }
        }
    }
}

fn mushroom_red_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....RRRRRRRR....",
        "..RRWWRRRRRRRR..",
        ".RRWWWWRRWWRRRR.",
        ".RRWWWWRRWWWWRR.",
        ".RRRWWRRRWWWWRR.",
        ".RRRRRRRRWWWRRR.",
        "..RRRRRRRRRRRR..",
        "......ssss......",
        "......ssss......",
        "......ssss......",
        ".....ssssss.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((188, 38, 34, 255)),
        'W' => Some((245, 240, 235, 255)),
        's' => Some((206, 194, 176, 255)),
        _ => None,
    });
}

fn mushroom_brown_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....MMMMMMMM....",
        "..MMMMMMMMMMMM..",
        ".MMMLLLLMMMMMMM.",
        ".MMMLLLLMMMMMMM.",
        ".MMMMMMMMMMMMMM.",
        ".MMMMMMMMMMMMMM.",
        "..MMMMMMMMMMMM..",
        "......ssss......",
        "......ssss......",
        "......ssss......",
        ".....ssssss.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((150, 112, 78, 255)),
        'L' => Some((182, 148, 110, 255)),
        's' => Some((206, 194, 176, 255)),
        _ => None,
    });
}

fn dead_bush_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "......b.........",
        ".....b.b...b....",
        "....b..b..b.....",
        "...b...b.b......",
        "....b..b.b......",
        ".....b.b..b.....",
        "....b..b...b....",
        ".....b.b..b.....",
        "......bb.b......",
        ".......bb.......",
        ".......b........",
        "......bbb.......",
        ".....bbbbb......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'b' => Some((123, 79, 25, 255)),
        _ => None,
    });
}

// ------------------------------------------------------------------ entry --

pub fn generate_atlas() -> Vec<u8> {
    let mut a = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
    for t in 0..=TILE_MAX {
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
            // stone family
            TILE_GRANITE => granite(&mut a, t, &mut rng),
            TILE_DIORITE => diorite(&mut a, t, &mut rng),
            TILE_ANDESITE => andesite(&mut a, t, &mut rng),
            TILE_STONE_BRICKS => stone_bricks(&mut a, t, &mut rng),
            TILE_BRICKS => bricks(&mut a, t, &mut rng),
            TILE_MOSSY_COBBLE => mossy_cobble(&mut a, t, &mut rng),
            TILE_SMOOTH_STONE => smooth_stone(&mut a, t, &mut rng),
            TILE_OBSIDIAN => obsidian(&mut a, t, &mut rng),
            // ores
            TILE_COAL_ORE => coal_ore(&mut a, t, &mut rng),
            TILE_IRON_ORE => iron_ore(&mut a, t, &mut rng),
            TILE_GOLD_ORE => gold_ore(&mut a, t, &mut rng),
            TILE_DIAMOND_ORE => diamond_ore(&mut a, t, &mut rng),
            TILE_REDSTONE_ORE => redstone_ore(&mut a, t, &mut rng),
            TILE_LAPIS_ORE => lapis_ore(&mut a, t, &mut rng),
            TILE_EMERALD_ORE => emerald_ore(&mut a, t, &mut rng),
            // mineral blocks
            TILE_IRON_BLOCK => iron_block(&mut a, t, &mut rng),
            TILE_GOLD_BLOCK => gold_block(&mut a, t, &mut rng),
            TILE_DIAMOND_BLOCK => diamond_block(&mut a, t, &mut rng),
            // misc
            TILE_GLOWSTONE => glowstone(&mut a, t, &mut rng),
            TILE_BOOKSHELF_SIDE => bookshelf_side(&mut a, t, &mut rng),
            TILE_BOOKSHELF_TOP => bookshelf_top(&mut a, t, &mut rng),
            TILE_CRAFT_TOP => craft_top(&mut a, t, &mut rng),
            TILE_CRAFT_SIDE => craft_side(&mut a, t, &mut rng),
            TILE_CLAY => clay(&mut a, t, &mut rng),
            TILE_TERRACOTTA => terracotta(&mut a, t, &mut rng),
            TILE_PUMPKIN_SIDE => pumpkin_side(&mut a, t, &mut rng),
            TILE_PUMPKIN_TOP => pumpkin_top(&mut a, t, &mut rng),
            TILE_MELON_SIDE => melon_side(&mut a, t, &mut rng),
            TILE_MELON_TOP => melon_top(&mut a, t, &mut rng),
            TILE_ICE => ice(&mut a, t, &mut rng),
            TILE_CACTUS_SIDE => cactus_side(&mut a, t, &mut rng),
            TILE_CACTUS_TOP => cactus_top(&mut a, t, &mut rng),
            // wool
            TILE_WOOL_WHITE => wool_white(&mut a, t, &mut rng),
            TILE_WOOL_RED => wool_red(&mut a, t, &mut rng),
            TILE_WOOL_BLUE => wool_blue(&mut a, t, &mut rng),
            TILE_WOOL_YELLOW => wool_yellow(&mut a, t, &mut rng),
            TILE_WOOL_BLACK => wool_black(&mut a, t, &mut rng),
            // wood variants
            TILE_BIRCH_LOG_SIDE => birch_log_side(&mut a, t, &mut rng),
            TILE_BIRCH_LEAVES => birch_leaves(&mut a, t, &mut rng),
            TILE_SPRUCE_LOG_SIDE => spruce_log_side(&mut a, t, &mut rng),
            TILE_SPRUCE_LEAVES => spruce_leaves(&mut a, t, &mut rng),
            // plants
            TILE_MUSHROOM_RED => mushroom_red_art(&mut a, t),
            TILE_MUSHROOM_BROWN => mushroom_brown_art(&mut a, t),
            TILE_DEAD_BUSH => dead_bush_art(&mut a, t),
            // redstone core (Phase 6 §25)
            TILE_REDSTONE_WIRE => redstone_wire_art(&mut a, t),
            TILE_REDSTONE_TORCH => redstone_torch_art(&mut a, t),
            TILE_LEVER => lever_art(&mut a, t),
            // gameplay (Phase 7)
            TILE_FURNACE_SIDE => furnace_side_art(&mut a, t, false),
            TILE_FURNACE_TOP => furnace_top_art(&mut a, t),
            TILE_FURNACE_LIT_SIDE => furnace_side_art(&mut a, t, true),
            // nether blocks (Phase 7 §28)
            TILE_NETHERRACK => netherrack(&mut a, t, &mut rng),
            TILE_QUARTZ_ORE => quartz_ore(&mut a, t, &mut rng),
            TILE_SOUL_SAND => soul_sand(&mut a, t, &mut rng),
            // brewing (Phase 7 §29)
            TILE_BREWING_STAND => brewing_stand_art(&mut a, t),
            TILE_BOTTLE_EMPTY => potion_art(&mut a, t, (120, 130, 150), false),
            TILE_POTION_WATER => potion_art(&mut a, t, (60, 100, 220), false),
            TILE_POTION_AWKWARD => potion_art(&mut a, t, (200, 120, 200), false),
            TILE_POTION_MUNDANE => potion_art(&mut a, t, (130, 130, 130), false),
            TILE_POTION_HEALING => potion_art(&mut a, t, (220, 60, 110), false),
            TILE_POTION_HEALING_II => potion_art(&mut a, t, (240, 80, 60), true),
            // enchanting (Phase 7 §29)
            TILE_ENCHANT_TABLE => enchant_table_art(&mut a, t),
            TILE_ENCHANTED_BOOK => enchanted_book_art(&mut a, t),
            // villagers (Phase 7 §27/§29)
            TILE_VILLAGER => villager_art(&mut a, t),
            // mobs (Phase 2) — entity sprites, clean-room art (ours)
            TILE_ZOMBIE => zombie_art(&mut a, t),
            TILE_SKELETON => skeleton_art(&mut a, t),
            TILE_CREEPER => creeper_art(&mut a, t),
            TILE_SPIDER => spider_art(&mut a, t),
            TILE_ENDERMAN => enderman_art(&mut a, t),
            TILE_COW => cow_art(&mut a, t),
            TILE_PIG => pig_art(&mut a, t),
            TILE_SHEEP => sheep_art(&mut a, t),
            TILE_CHICKEN => chicken_art(&mut a, t),
            TILE_ARROW => arrow_art(&mut a, t),
            // mob drops (Phase 2) — item icons
            TILE_BEEF => meat_art(&mut a, t, (196, 92, 76), (150, 62, 52)),
            TILE_PORKCHOP => meat_art(&mut a, t, (240, 150, 140), (222, 110, 96)),
            TILE_MUTTON => meat_art(&mut a, t, (200, 80, 70), (150, 50, 46)),
            TILE_CHICKEN_RAW => meat_art(&mut a, t, (232, 176, 168), (206, 130, 120)),
            TILE_FEATHER => feather_art(&mut a, t),
            TILE_LEATHER => leather_art(&mut a, t),
            TILE_BONE => bone_art(&mut a, t),
            TILE_STRING => string_art(&mut a, t),
            TILE_GUNPOWDER => gunpowder_art(&mut a, t),
            TILE_ENDER_PEARL => ender_pearl_art(&mut a, t),
            TILE_ROTTEN_FLESH => rotten_flesh_art(&mut a, t),
            TILE_ARROW_ITEM => arrow_art(&mut a, t),
            // Phase 4 §26/§30: corruption-chain potions + ingredients
            TILE_POTION_HARMING => potion_art(&mut a, t, (110, 30, 30), false),
            TILE_POTION_HARMING_II => potion_art(&mut a, t, (130, 40, 26), true),
            TILE_SPIDER_EYE => spider_eye_art(&mut a, t, false),
            TILE_FERMENTED_EYE => spider_eye_art(&mut a, t, true),
            // Phase 5 §27: dungeon spawner cage face
            TILE_SPAWNER => spawner_art(&mut a, t),
            // Phase 10: stronghold portal-room frame
            TILE_END_PORTAL_FRAME => end_portal_frame_art(&mut a, t),
            // ---- Phase E1 tiles (evolution 1.0–1.2 bracket) ----
            TILE_MYCELIUM_TOP => e1_art::mycelium_top(&mut a, t, &mut rng),
            TILE_MYCELIUM_SIDE => e1_art::mycelium_side(&mut a, t, &mut rng),
            TILE_END_STONE => e1_art::end_stone(&mut a, t, &mut rng),
            TILE_NETHER_BRICKS => e1_art::nether_bricks(&mut a, t, &mut rng),
            TILE_REDSTONE_LAMP => e1_art::redstone_lamp(&mut a, t, &mut rng, false),
            TILE_REDSTONE_LAMP_ON => e1_art::redstone_lamp(&mut a, t, &mut rng, true),
            TILE_CHISELED_STONE_BRICKS => e1_art::chiseled_stone_bricks(&mut a, t, &mut rng),
            TILE_CHISELED_SANDSTONE => e1_art::chiseled_sandstone(&mut a, t, &mut rng),
            TILE_CUT_SANDSTONE => e1_art::cut_sandstone(&mut a, t, &mut rng),
            TILE_SMOOTH_SANDSTONE => e1_art::smooth_sandstone(&mut a, t, &mut rng),
            TILE_MUSHROOM_RED_BLOCK => e1_art::mushroom_block_red(&mut a, t, &mut rng),
            TILE_MUSHROOM_BROWN_BLOCK => e1_art::mushroom_block_brown(&mut a, t, &mut rng),
            TILE_MUSHROOM_STEM => e1_art::mushroom_stem(&mut a, t, &mut rng),
            TILE_NETHER_WART_0 => e1_art::nether_wart_art(&mut a, t, 0),
            TILE_NETHER_WART_1 => e1_art::nether_wart_art(&mut a, t, 1),
            TILE_NETHER_WART_2 => e1_art::nether_wart_art(&mut a, t, 2),
            TILE_NETHER_WART_3 => e1_art::nether_wart_art(&mut a, t, 3),
            TILE_DRAGON_EGG => e1_art::dragon_egg_art(&mut a, t),
            TILE_END_PORTAL => e1_art::end_portal_art(&mut a, t),
            TILE_END_CRYSTAL => e1_art::end_crystal_art(&mut a, t),
            TILE_XP_ORB => e1_art::xp_orb_art(&mut a, t, false),
            TILE_XP_ORB_BIG => e1_art::xp_orb_art(&mut a, t, true),
            TILE_EYE_OF_ENDER => e1_art::eye_of_ender_art(&mut a, t),
            TILE_BLAZE_ROD => e1_art::blaze_rod_art(&mut a, t),
            TILE_BLAZE_POWDER => e1_art::blaze_powder_art(&mut a, t),
            TILE_GOLDEN_APPLE => e1_art::golden_apple_art(&mut a, t),
            TILE_SNOWBALL => e1_art::snowball_art(&mut a, t),
            TILE_NETHER_BRICK => e1_art::nether_brick_art(&mut a, t),
            // mob sprites (Phase E1)
            TILE_SNOWGOLEM => e1_art::snow_golem_art(&mut a, t),
            TILE_MAGMACUBE => e1_art::magma_cube_art(&mut a, t),
            TILE_BLAZE => e1_art::blaze_art(&mut a, t),
            TILE_OCELOT => e1_art::ocelot_art(&mut a, t),
            TILE_IRONGOLEM => e1_art::iron_golem_art(&mut a, t),
            TILE_ZOMBIEVILLAGER => e1_art::zombie_villager_art(&mut a, t),
            TILE_MOOSHROOM => e1_art::mooshroom_art(&mut a, t),
            TILE_ENDERDRAGON => e1_art::ender_dragon_art(&mut a, t),
            // spawn eggs: palette pairs indexed by egg id (order = egg_mob)
            t if (TILE_EGG_BASE..=TILE_EGG_MAX).contains(&t) => {
                let p = e1_art::EGG_PALETTES[(t - TILE_EGG_BASE) as usize];
                e1_art::egg_art(&mut a, t, (p.0, p.1, p.2), (p.3, p.4, p.5));
            }
            _ => {}
        }
    }
    a
}

/// Blit a tile (scaled) into a UI pixel buffer — used for hotbar icons.
pub fn blit_tile(
    atlas: &[u8],
    tile: u16,
    scale: usize,
    ox: usize,
    oy: usize,
    out: &mut [u8],
    out_w: usize,
) {
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

// -------------------------------------------------- Phase 6: mipmaps (§26) --

/// maximum mipmap levels BEYOND level 0 (vanilla `mipmapLevels` slider caps
/// at 4 — VERIFIED against minecraft.wiki Options.txt: range 0–4, default 4,
/// option exists since 1.7.2). A 16px tile bottoms out at 1px on level 4;
/// level 5+ would collapse whole tiles into single texels (cross-tile
/// blending) which is exactly why vanilla stops at 4.
pub const MAX_MIP_LEVELS: u8 = 4;

/// Generate the mipmap chain for the atlas: returns images for levels
/// 1..=levels (`out[L-1]` is level L, sized ATLAS_SIZE>>L). Pure sequential
/// 2×2 box filter over the raw RGBA bytes.
///
/// **Tile safety (the vanilla-parity property):** vanilla generates mips
/// PER SPRITE (each sprite's mip data comes from the sprite alone, so
/// neighboring atlas sprites never bleed into each other). We get the same
/// guarantee for free from geometry: tiles are 16px-aligned power-of-2
/// squares, so a 2×2 average block at any level ≤ 4 always lies inside one
/// tile (the block's origin is even-aligned in source space, and tile
/// boundaries are multiples of the level-1 tile stride). Verified by test.
///
/// **Documented adaptation:** vanilla's MipmapGenerator blends in linear
/// space with alpha-weighting to avoid dark halos on cutout textures; we
/// average the raw sRGB bytes (what GL's generateMipmap does on sRGB
/// formats — a classic quirk). Deterministic, and the halo difference is
/// invisible on our 16px procedural tiles.
pub fn generate_mips(atlas: &[u8], levels: u8) -> Vec<Vec<u8>> {
    let levels = levels.min(MAX_MIP_LEVELS);
    let mut out = Vec::with_capacity(levels as usize);
    let mut cur = atlas.to_vec();
    for _ in 0..levels {
        let next = box_downsample(&cur);
        out.push(next.clone());
        cur = next;
    }
    out
}

/// one 2×2 box-filter step: `src` is a square RGBA image, `dst` is half-size.
/// Square side is derived from the data (256, 128, 64, …).
fn box_downsample(src: &[u8]) -> Vec<u8> {
    let px = src.len() / 4;
    let side = (px as f64).sqrt() as usize; // exact for power-of-2 squares
    let half = side / 2;
    let mut dst = vec![0u8; half * half * 4];
    for y in 0..half {
        for x in 0..half {
            // 2×2 source block — never crosses a tile boundary (see above)
            let i00 = ((y * 2) * side + x * 2) * 4;
            let i01 = ((y * 2) * side + x * 2 + 1) * 4;
            let i10 = ((y * 2 + 1) * side + x * 2) * 4;
            let i11 = ((y * 2 + 1) * side + x * 2 + 1) * 4;
            let d = (y * half + x) * 4;
            for c in 0..4 {
                dst[d + c] = ((src[i00 + c] as u32
                    + src[i01 + c] as u32
                    + src[i10 + c] as u32
                    + src[i11 + c] as u32)
                    / 4) as u8;
            }
        }
    }
    dst
}

// ------------------------------------------------------- pack texture merge --

/// first free atlas tile after the procedural set + missing-texture tile.
/// DERIVED from TILE_MAX so Phase-6/7+ procedural tiles can never collide
/// with pack textures again (they did: base 64 overwrote the wire/torch).
pub const PACK_TILE_BASE: u16 = vc_blocks::blocks::TILE_MAX + 1;
/// hard cap: 16×16 tile grid = 256 tiles in the 256² atlas
pub const PACK_TILE_MAX: u16 = 255;

/// draw the missing-texture tile (magenta/black 8×8 checker, §46 fallback —
/// never crash, always something visible)
pub fn draw_missing_tile(atlas: &mut [u8]) {
    let t = vc_mesh::mesh::TILE_MISSING;
    let tx = (t % 16) as usize;
    let ty = (t / 16) as usize;
    for y in 0..TILE_PX {
        for x in 0..TILE_PX {
            let magenta = ((x / 4) + (y / 4)) % 2 == 0;
            let (r, g, b) = if magenta { (248, 0, 248) } else { (0, 0, 0) };
            let i = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX + x) * 4;
            atlas[i] = r;
            atlas[i + 1] = g;
            atlas[i + 2] = b;
            atlas[i + 3] = 255;
        }
    }
}

/// one pack texture animation (VERIFIED semantics: vertical strip, frame =
/// width×width, `frametime` in ticks, optional explicit `frames` list)
#[derive(Clone, Debug)]
pub struct AnimatedTile {
    pub tile: u16,
    /// precomputed 16×16 RGBA frames
    pub frames: Vec<Vec<u8>>,
    /// ticks per frame (1 tick = 1/20 s; we advance by game time)
    pub frametime: f32,
    pub current: usize,
    pub timer: f32,
}

/// Merge pack textures into the procedural atlas + fill the ModelSet's
/// tile registry. Returns the animations to drive per frame.
///
/// * texture paths come from the compiled model faces (`models.tiles`)
/// * missing files fall back to the missing-texture tile (§46)
/// * non-16×16 sources are nearest-resampled to 16×16
/// * vertical-strip PNGs with `.png.mcmeta {animation:{...}}` become
///   AnimatedTiles (geometry is NOT rebuilt — only the atlas region updates)
pub fn merge_pack_textures(
    atlas: &mut [u8],
    models: &mut vc_pack::model::ModelSet,
    source: &dyn vc_pack::pack::PackSource,
) -> Vec<AnimatedTile> {
    let mut animations = Vec::new();
    // stable order: collect locations from by_state (dispatch order)
    let mut locs: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = Default::default();
    for choices in models.by_state.values() {
        for choice in choices {
            for ap in &choice.alts {
                for el in ap.model.elements.iter() {
                    for f in el.faces.iter() {
                        if seen.insert(f.texture.clone()) {
                            locs.push(f.texture.clone());
                        }
                    }
                }
            }
        }
    }

    let mut next_tile = PACK_TILE_BASE;
    for loc in locs {
        if next_tile > PACK_TILE_MAX {
            // atlas full: everything remaining falls back to the missing tile
            models.tiles.insert(loc, vc_mesh::mesh::TILE_MISSING);
            continue;
        }
        let path = vc_pack::model::texture_path(&loc);
        let Some(bytes) = source.read(&path) else {
            models.tiles.insert(loc, vc_mesh::mesh::TILE_MISSING);
            continue;
        };
        let Ok(img) = image::load_from_memory(&bytes) else {
            models.tiles.insert(loc, vc_mesh::mesh::TILE_MISSING);
            continue;
        };
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let tile = next_tile;
        next_tile += 1;

        // animation metadata?
        let mcmeta: Option<serde_json::Value> = source
            .read(&format!("{path}.mcmeta"))
            .and_then(|b| serde_json::from_slice(&b).ok());
        let anim = mcmeta.as_ref().and_then(|m| m.get("animation")).cloned();
        let strip = anim.is_some() && w > 0 && h > w && (h % w == 0);

        if strip && w <= 64 {
            // vertical animation strip: frames stacked top→bottom
            let frames_n = (h / w) as usize;
            let frametime = anim
                .as_ref()
                .and_then(|a| a.get("frametime"))
                .and_then(|f| f.as_u64())
                .unwrap_or(1) as f32;
            // explicit frame order (defaults to 0..n sequential)
            let order: Vec<usize> = anim
                .as_ref()
                .and_then(|a| a.get("frames"))
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            v.as_u64().map(|i| i as usize).or_else(|| {
                                v.get("index").and_then(|i| i.as_u64()).map(|i| i as usize)
                            })
                        })
                        .collect()
                })
                .unwrap_or_else(|| (0..frames_n).collect());
            let mut frames: Vec<Vec<u8>> = Vec::with_capacity(order.len());
            for fi in order.iter() {
                if *fi < frames_n {
                    frames.push(resample_strip_frame(&rgba, *fi, w as usize));
                }
            }
            if !frames.is_empty() {
                // frame 0 goes into the atlas immediately
                blit_16(atlas, tile, &frames[0]);
                models.tiles.insert(loc, tile);
                animations.push(AnimatedTile {
                    tile,
                    frames,
                    frametime: (frametime / 20.0).max(0.05),
                    current: 0,
                    timer: 0.0,
                });
                continue;
            }
        }

        // static texture: nearest-resample to 16×16
        let frame = resample_full(&rgba, w as usize, h as usize);
        blit_16(atlas, tile, &frame);
        models.tiles.insert(loc, tile);
    }
    animations
}

/// blit a 16×16 RGBA frame into an atlas tile slot
fn blit_16(atlas: &mut [u8], tile: u16, frame: &[u8]) {
    let tx = (tile % 16) as usize;
    let ty = (tile / 16) as usize;
    for y in 0..TILE_PX {
        for x in 0..TILE_PX {
            let src = (y * TILE_PX + x) * 4;
            let dst = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX + x) * 4;
            atlas[dst..dst + 4].copy_from_slice(&frame[src..src + 4]);
        }
    }
}

/// nearest-neighbor resample of a full image to 16×16
fn resample_full(rgba: &image::RgbaImage, w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; TILE_PX * TILE_PX * 4];
    for y in 0..TILE_PX {
        for x in 0..TILE_PX {
            let sx = x * w / TILE_PX;
            let sy = y * h / TILE_PX;
            let dst = (y * TILE_PX + x) * 4;
            let p = rgba.get_pixel(sx as u32, sy as u32).0;
            out[dst] = p[0];
            out[dst + 1] = p[1];
            out[dst + 2] = p[2];
            out[dst + 3] = p[3];
        }
    }
    out
}

/// extract + resample one frame from a vertical strip (frame fi)
fn resample_strip_frame(rgba: &image::RgbaImage, fi: usize, w: usize) -> Vec<u8> {
    let mut out = vec![0u8; TILE_PX * TILE_PX * 4];
    let fh = w; // frame height == width (square frames)
    let off_y = fi * fh;
    for y in 0..TILE_PX {
        for x in 0..TILE_PX {
            let sx = x * w / TILE_PX;
            let sy = off_y + y * fh / TILE_PX;
            let dst = (y * TILE_PX + x) * 4;
            let p = rgba.get_pixel(sx as u32, sy as u32).0;
            out[dst] = p[0];
            out[dst + 1] = p[1];
            out[dst + 2] = p[2];
            out[dst + 3] = p[3];
        }
    }
    out
}

/// advance every animation by dt; returns (tile, frame) pairs to re-upload
pub fn tick_animations(anims: &mut [AnimatedTile], dt: f32) -> Vec<(u16, u16)> {
    let mut updates = Vec::new();
    for a in anims.iter_mut() {
        a.timer += dt;
        while a.timer >= a.frametime {
            a.timer -= a.frametime;
            a.current = (a.current + 1) % a.frames.len();
            updates.push((a.tile, a.current as u16));
        }
    }
    updates
}

#[cfg(test)]
mod pack_tex_tests {
    use super::*;

    #[test]
    fn missing_tile_draws_checker() {
        let mut atlas = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
        draw_missing_tile(&mut atlas);
        let t = vc_mesh::mesh::TILE_MISSING;
        let tx = (t % 16) as usize;
        let ty = (t / 16) as usize;
        let i = ((ty * TILE_PX + 0) * ATLAS_SIZE + tx * TILE_PX + 0) * 4;
        assert_eq!(&atlas[i..i + 4], &[248, 0, 248, 255]); // magenta
        let i2 = ((ty * TILE_PX + 0) * ATLAS_SIZE + tx * TILE_PX + 4) * 4;
        assert_eq!(&atlas[i2..i2 + 4], &[0, 0, 0, 255]); // black
    }

    #[test]
    fn animation_tick_advances_frames() {
        let mut anims = vec![AnimatedTile {
            tile: 64,
            frames: vec![vec![1u8; 1024], vec![2u8; 1024], vec![3u8; 1024]],
            frametime: 0.05,
            current: 0,
            timer: 0.0,
        }];
        let upd = tick_animations(&mut anims, 0.05);
        assert_eq!(upd, vec![(64, 1)]);
        assert_eq!(anims[0].current, 1);
        // 3 frames wrap
        let _ = tick_animations(&mut anims, 0.11); // 2+ steps
        assert_eq!(anims[0].current, 0);
    }

    /// Regenerate the builtin pack's PNG textures from the procedural atlas
    /// (our own clean-room art). Not part of CI: run on demand with
    ///   cargo test write_builtin_pack_pngs -- --ignored --nocapture
    #[test]
    #[ignore]
    fn write_builtin_pack_pngs() {
        let out_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../builtin-pack/assets/minecraft/textures/block"
        );
        std::fs::create_dir_all(out_dir).unwrap();
        let atlas = generate_atlas();

        let save = |_tile: u16, name: &str, w: u32, h: u32, buf: &[u8]| {
            let img = image::RgbaImage::from_raw(w, h, buf.to_vec()).unwrap();
            let path = format!("{out_dir}/{name}");
            img.save_with_format(&path, image::ImageFormat::Png)
                .unwrap();
            println!("wrote {path} ({w}x{h})");
        };

        let extract = |tile: u16| -> Vec<u8> {
            let tx = (tile % 16) as usize;
            let ty = (tile / 16) as usize;
            let mut out = vec![0u8; TILE_PX * TILE_PX * 4];
            for y in 0..TILE_PX {
                for x in 0..TILE_PX {
                    let src = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX + x) * 4;
                    let dst = (y * TILE_PX + x) * 4;
                    out[dst..dst + 4].copy_from_slice(&atlas[src..src + 4]);
                }
            }
            out
        };

        // static: oak_planks (slab + fence textures)
        save(
            TILE_PLANKS,
            "oak_planks.png",
            TILE_PX as u32,
            TILE_PX as u32,
            &extract(TILE_PLANKS),
        );

        // animated: cobblestone 4-frame shimmer strip (16x64) + .mcmeta
        let strip: Vec<u8> = (0..4)
            .flat_map(|f| {
                // each frame: the procedural cobble re-jittered by frame seed
                let mut frame = extract(TILE_COBBLE);
                let mut rng = Rng::new(0x4C0B + f as u64 * 131);
                for i in 0..TILE_PX * TILE_PX {
                    let j = rng.next_u64() as i32 % 7 - 3;
                    let d = (i * 4) + (i % 3);
                    frame[d] = (frame[d] as i32 + j).clamp(0, 255) as u8;
                }
                frame
            })
            .collect();
        save(
            TILE_COBBLE,
            "cobblestone.png",
            TILE_PX as u32,
            (TILE_PX * 4) as u32,
            &strip,
        );
        let mcmeta = r#"{ "animation": { "frametime": 8 } }"#;
        std::fs::write(format!("{out_dir}/cobblestone.png.mcmeta"), mcmeta).unwrap();
        println!("wrote {out_dir}/cobblestone.png.mcmeta (frametime 8)");
    }
}

// ---------------------------------------------------------------- clouds --

/// Procedural cloud texture size (128x128, blocky 2x2 cells → 64x64 clouds).
pub const CLOUD_TEX: usize = 128;
/// Vanilla-style blocky cloud layer: periodic value noise thresholded on a
/// 64x64 cell grid (periodic by construction → seamless tiling), each cell
/// rendered as a 2x2 block for the crisp Minecraft cloud look.
pub fn generate_cloud_atlas() -> Vec<u8> {
    const CELLS: usize = 64;
    let hash = |x: i32, y: i32| -> f32 {
        let n = ((x.wrapping_mul(73856093)) ^ (y.wrapping_mul(19349663))) as f64;
        let s = (n * 0.0001).fract().abs();
        let _ = s;
        let v = (n.sin() * 43758.5453).fract().abs() as f32;
        v
    };
    let cell = |cx: usize, cy: usize| -> bool {
        // wrap → seamless tiling over CELLS
        let x = (cx & (CELLS - 1)) as i32;
        let y = (cy & (CELLS - 1)) as i32;
        let n = hash(x, y) * 0.62 + hash((x >> 1) & 31, (y >> 1) & 31) * 0.38;
        n > 0.56
    };
    let mut px = vec![0u8; CLOUD_TEX * CLOUD_TEX * 4];
    for ty in 0..CLOUD_TEX {
        for tx in 0..CLOUD_TEX {
            let c = cell(tx / 2, ty / 2);
            let i = (ty * CLOUD_TEX + tx) * 4;
            if c {
                px[i] = 255;
                px[i + 1] = 255;
                px[i + 2] = 255;
                px[i + 3] = 255;
            }
        }
    }
    px
}

#[cfg(test)]
mod pack_merge_tests {
    use super::*;

    /// helper: raw atlas bytes of one tile (RGBA, 16×16)
    fn tile_bytes(atlas: &[u8], tile: u16) -> Vec<u8> {
        let tx = (tile % 16) as usize;
        let ty = (tile / 16) as usize;
        let mut out = Vec::with_capacity(TILE_PX * TILE_PX * 4);
        for y in 0..TILE_PX {
            let row = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX) * 4;
            out.extend_from_slice(&atlas[row..row + TILE_PX * 4]);
        }
        out
    }

    /// PACK_TILE_BASE must sit strictly ABOVE every procedural tile
    /// (incl. the missing-texture tile). Phase-6/7 added tiles 64..69
    /// (wire/torch/lever/furnace) while PACK_TILE_BASE stayed 64 — the
    /// builtin pack then overwrote the wire/torch art in the atlas.
    #[test]
    fn pack_tile_base_is_above_all_procedural_tiles() {
        assert!(
            PACK_TILE_BASE > vc_blocks::blocks::TILE_MAX,
            "PACK_TILE_BASE {PACK_TILE_BASE} must clear procedural TILE_MAX {}",
            vc_blocks::blocks::TILE_MAX
        );
        assert!(PACK_TILE_BASE > vc_mesh::mesh::TILE_MISSING);
    }

    // --------------------------------------------- Phase 6 §26: mipmaps --

    /// the mip chain has the right sizes: level L is (ATLAS_SIZE>>L)²
    #[test]
    fn mip_chain_sizes() {
        let atlas = generate_atlas();
        let mips = generate_mips(&atlas, 4);
        assert_eq!(mips.len(), 4);
        assert_eq!(mips[0].len(), 128 * 128 * 4);
        assert_eq!(mips[1].len(), 64 * 64 * 4);
        assert_eq!(mips[2].len(), 32 * 32 * 4);
        assert_eq!(mips[3].len(), 16 * 16 * 4);
    }

    /// levels is clamped to MAX_MIP_LEVELS (vanilla's 0-4 range)
    #[test]
    fn mip_levels_clamped() {
        let atlas = generate_atlas();
        assert!(generate_mips(&atlas, 0).is_empty());
        assert_eq!(generate_mips(&atlas, 9).len(), MAX_MIP_LEVELS as usize);
    }

    /// THE vanilla-parity property: adjacent tiles must never bleed into
    /// each other at any mip level. Craft an atlas with tile (0,0) pure
    /// white and tile (1,0) pure black; every mip level must keep tile
    /// (0,0)'s area pure white and tile (1,0)'s area pure black.
    #[test]
    fn mips_never_blend_across_tiles() {
        let mut atlas = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
        // tile (0,0): white; tile (1,0): black; rest mid-gray
        for ty in 0..16 {
            for tx in 0..16 {
                let c = match (tx, ty) {
                    (0, 0) => [255, 255, 255, 255],
                    (1, 0) => [0, 0, 0, 255],
                    _ => [128, 128, 128, 255],
                };
                for y in 0..TILE_PX {
                    for x in 0..TILE_PX {
                        let i = ((ty * TILE_PX + y) * ATLAS_SIZE + tx * TILE_PX + x) * 4;
                        atlas[i..i + 4].copy_from_slice(&c);
                    }
                }
            }
        }
        for (li, mip) in generate_mips(&atlas, MAX_MIP_LEVELS).iter().enumerate() {
            let level = li + 1; // mip[0] is level 1
            let side = ATLAS_SIZE >> level;
            let tile_side = TILE_PX >> level;
            for t in [0usize, 1usize] {
                let px = mip[(0 * side + t * tile_side) * 4]; // first px of tile (t, 0)
                let want = if t == 0 { 255 } else { 0 };
                // every pixel of the tile must stay pure (uniform source tiles)
                for y in 0..tile_side {
                    for x in 0..tile_side {
                        let i = (y * side + t * tile_side + x) * 4;
                        assert_eq!(
                            mip[i], want,
                            "level {level} tile {t} px ({x},{y}) = {} — cross-tile bleed!",
                            mip[i]
                        );
                        assert_eq!(mip[i + 3], 255, "alpha must stay opaque");
                    }
                }
                assert_eq!(px, want);
            }
        }
    }

    /// a flat 2×2 block averages exactly (determinism + correctness of the
    /// box filter itself, on a known constant)
    #[test]
    fn mip_averages_a_flat_block() {
        let mut atlas = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
        for px in atlas.chunks_exact_mut(4) {
            px.copy_from_slice(&[100, 150, 200, 255]);
        }
        let mips = generate_mips(&atlas, 1);
        assert!(mips[0].chunks_exact(4).all(|px| px == [100, 150, 200, 255]));
    }

    /// the real merge must not touch ANY procedural tile: generate the
    /// atlas, snapshot the wire/torch tiles, compile the real builtin
    /// dispatch (so the pack's textures actually merge), compare
    /// (native-only — reads the folder).
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn builtin_pack_merge_preserves_procedural_tiles() {
        let source = std::sync::Arc::new(vc_pack::pack::FolderSource::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../builtin-pack"),
            "test",
        ));
        use vc_pack::pack::PackSource as _;
        assert!(
            source.exists(),
            "builtin pack missing — run from voxelcraft/"
        );
        // compile the real dispatch so the pack textures genuinely merge
        let mut by_state: std::collections::HashMap<u16, Vec<vc_pack::model::ModelChoice>> =
            Default::default();
        for pb in vc_blocks::blocks::PROP_BLOCKS.iter() {
            let spec = vc_pack::model::BlockDispatchSpec {
                name: pb.name,
                props: pb.props,
                base_state: pb.base_state,
                state_count: pb.state_count,
            };
            let map = vc_pack::model::compile_block_dispatch(&spec, &|p| source.read(p))
                .unwrap_or_else(|e| panic!("dispatch {name:?}: {e}", name = pb.name));
            by_state.extend(map);
        }
        let mut set = vc_pack::model::ModelSet {
            by_state,
            tiles: Default::default(),
        };
        let mut atlas = generate_atlas();
        let watched = [
            vc_blocks::blocks::TILE_REDSTONE_WIRE,
            vc_blocks::blocks::TILE_REDSTONE_TORCH,
            vc_blocks::blocks::TILE_LEVER,
            vc_blocks::blocks::TILE_FURNACE_SIDE,
            vc_blocks::blocks::TILE_FURNACE_TOP,
            vc_blocks::blocks::TILE_FURNACE_LIT_SIDE,
        ];
        let before: Vec<(u16, Vec<u8>)> = watched
            .iter()
            .map(|&t| (t, tile_bytes(&atlas, t)))
            .collect();
        let _ = merge_pack_textures(&mut atlas, &mut set, source.as_ref());
        assert!(
            !set.tiles.is_empty(),
            "test setup broken — no pack textures merged, comparison would be vacuous"
        );
        for (t, b) in before {
            assert_eq!(
                tile_bytes(&atlas, t),
                b,
                "atlas tile {t} changed under the pack merge — pack tiles overlap procedural tiles"
            );
        }
    }
}

#[cfg(test)]
mod cloud_tests {
    use super::*;

    #[test]
    fn cloud_atlas_has_sparse_coverage() {
        let px = generate_cloud_atlas();
        let mut opaque = 0;
        for i in (3..px.len()).step_by(4) {
            if px[i] > 127 {
                opaque += 1;
            }
        }
        let total = CLOUD_TEX * CLOUD_TEX;
        println!(
            "cloud coverage: {}/{} = {:.1}%",
            opaque,
            total,
            100.0 * opaque as f32 / total as f32
        );
        // expect ~30-60% puffy coverage, NOT 0% and NOT ~100%
        assert!(opaque > total / 10, "clouds vanished (no opaque pixels)");
        assert!(opaque < total * 7 / 10, "clouds blanket the whole sky");
    }
}
