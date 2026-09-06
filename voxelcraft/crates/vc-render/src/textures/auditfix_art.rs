//! Audit-fix round (2026-09-07) procedural tiles — the Phase-1/2 audit's
//! missing 1.2/1.4 content: jungle wood family, vines, ferns, golden
//! carrot. Clean-room art (no Mojang assets), a child module of
//! textures.rs (shares put/jit/noise_fill/art helpers).

use super::{art, jit, noise_fill, put, Rng};

/// Golden carrot item sprite: a diagonal golden-orange root with green
/// fronds at the top (item-block, cross-rendered like the other foods).
pub(super) fn golden_carrot(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "................",
        "..........gG....",
        ".........gGg....",
        "........gG.g....",
        ".......oO..g....",
        "......oOo.......",
        ".....oOo........",
        "....oOo.........",
        "...oOo..........",
        "..oOo...........",
        ".oOo............",
        ".Oo.............",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'o' => Some((228, 170, 42, 255)),
        'O' => Some((244, 197, 66, 255)),
        'g' => Some((72, 138, 52, 255)),
        'G' => Some((104, 178, 76, 255)),
        _ => None,
    });
}

/// Jungle log bark: deeper, richer brown than oak (107/83/49) with
/// strongly vertical grain — reads as a distinct species at a glance.
pub(super) fn jungle_log_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for x in 0..16 {
        // wide dark grain columns + narrow pale streaks between
        let col = match x % 6 {
            0 => -22,
            1 => -10,
            3 => 12,
            4 => 6,
            _ => 0,
        };
        for y in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(88 + col, 6, rng),
                jit(66 + col, 5, rng),
                jit(38 + col, 4, rng),
                255,
            );
        }
    }
    // knots
    for _ in 0..2 {
        let x = 2 + rng.next_range(12) as i32;
        let y = 2 + rng.next_range(12) as i32;
        put(a, t, x, y, 62, 46, 26, 255);
        put(a, t, x + 1, y, 70, 52, 30, 255);
    }
}

/// Jungle log top: growth rings in a warm mid brown, bark rim darker.
pub(super) fn jungle_log_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let dx = (x - 8) as f32;
            let dy = (y - 8) as f32;
            let r = (dx * dx + dy * dy).sqrt();
            let ring = ((r / 2.2).floor() as i32) % 2;
            let (br, bg, bb) = if r > 7.0 {
                (74, 56, 32) // bark rim
            } else if ring == 0 {
                (154 + jit(0, 8, rng), 118 + jit(0, 8, rng), 74 + jit(0, 6, rng))
            } else {
                (132, 100, 60)
            };
            put(a, t, x, y, br, bg, bb, 255);
        }
    }
}

/// Jungle leaves: saturated deep jungle green, denser mottle than oak.
pub(super) fn jungle_leaves(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [38, 92, 28];
    noise_fill(a, t, base, 16, rng);
    for _ in 0..22 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 24, 66, 18, 255);
    }
    // sun-lit leaf flecks
    for _ in 0..8 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 58, 126, 42, 255);
    }
}

/// Jungle planks: warm tan, horizontal board rows with seams.
pub(super) fn jungle_planks(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        let board = (y / 4) % 2;
        let col = if board == 0 { 8 } else { -6 };
        for x in 0..16 {
            let seam = y % 4 == 3;
            let (r, g, b) = if seam {
                (118, 86, 50)
            } else {
                (
                    jit(158 + col, 8, rng),
                    jit(122 + col, 7, rng),
                    jit(74 + col, 6, rng),
                )
            };
            put(a, t, x, y, r, g, b, 255);
        }
    }
    // staggered vertical joints
    for b in 0..4 {
        let jx = if b % 2 == 0 { 3 } else { 11 };
        for y in 0..4 {
            put(a, t, jx, b * 4 + y, 118, 86, 50, 255);
        }
    }
}

/// Vine: hanging green strands on transparency (cross-rendered
/// adaptation of the wall-attached vanilla plant).
pub(super) fn vine(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "................",
        "..a....b........",
        "..b...a.c..a....",
        "..c..a.b..b.c...",
        "...b.b.c.a.b.b..",
        "...c.c.b.b.c.a..",
        "....a.b.a.c.b...",
        "....b.c.b.b.c...",
        ".....b.a.c.a....",
        ".....c.b.b.b....",
        "......a.c.c.b...",
        "......b.b.a.c...",
        ".......c.a.b....",
        ".......b.b.c.a..",
        "........a.c.b...",
        ".........b.a....",
    ];
    art(a, t, rows, &|c| match c {
        'a' => Some((44, 92, 34, 255)),
        'b' => Some((58, 118, 44, 255)),
        'c' => Some((76, 142, 58, 255)),
        _ => None,
    });
}

/// Fern: radiating fronds from a base — a bushier silhouette than tall
/// grass, with a distinct center stem.
pub(super) fn fern(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "...b......b.....",
        "....c....c.c....",
        "..c.c.b..c.b..b.",
        "...b.c.c.b.c.c..",
        ".a..b.b.c.b.b...",
        "...c.c.a.c.c..c.",
        "....b.b.b.b.c...",
        "..c.c.c.a.c.b...",
        "...b.a.b.b.b.c..",
        "....c.c.c.c.a...",
        ".....b.b.a.b....",
        "......a.c.c.....",
        ".......a.a......",
        ".......a........",
    ];
    art(a, t, rows, &|c| match c {
        'a' => Some((46, 96, 36, 255)),
        'b' => Some((62, 122, 46, 255)),
        'c' => Some((84, 148, 62, 255)),
        _ => None,
    });
}
