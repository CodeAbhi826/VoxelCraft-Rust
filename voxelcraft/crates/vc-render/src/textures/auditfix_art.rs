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

// ---- 1.11 bracket (Exploration Update) tiles ----

/// Llama sprite: cream body, pale face, ears.
pub(super) fn llama(a: &mut [u8], t: u16, rng: &mut Rng) {
    let body = [196 + jit(0, 12, rng), 182 + jit(0, 12, rng), 156];
    let face = [226, 216, 196];
    for y in 2..14 {
        for x in 3..13 {
            if y < 6 && x > 8 {
                put(a, t, x, y, face[0], face[1], face[2], 255);
            } else {
                put(a, t, x, y, body[0] as i32, body[1] as i32, body[2] as i32, 255);
            }
        }
    }
    // ears
    put(a, t, 9, 1, face[0], face[1], face[2], 255);
    put(a, t, 11, 1, face[0], face[1], face[2], 255);
    // eye
    put(a, t, 10, 4, 40, 34, 30, 255);
}

/// Vindicator: grey-skinned illager, dark clothes, axe.
pub(super) fn vindicator(a: &mut [u8], t: u16, rng: &mut Rng) {
    let skin = [158 + jit(0, 10, rng), 158, 152];
    let robe = [58 + jit(0, 10, rng), 54, 62];
    for y in 2..14 {
        for x in 4..12 {
            if y < 7 {
                put(a, t, x, y, skin[0] as i32, skin[1], skin[2], 255);
            } else {
                put(a, t, x, y, robe[0] as i32, robe[1], robe[2], 255);
            }
        }
    }
    put(a, t, 6, 4, 40, 30, 30, 255); // eye
    put(a, t, 9, 4, 40, 30, 30, 255);
    // iron axe in hand
    put(a, t, 12, 8, 200, 200, 205, 255);
    put(a, t, 12, 9, 140, 100, 62, 255);
}

/// Evoker: pale-grey illager caster, dark robe, raised arms hint.
pub(super) fn evoker(a: &mut [u8], t: u16, rng: &mut Rng) {
    let skin = [178 + jit(0, 10, rng), 172, 166];
    let robe = [44 + jit(0, 10, rng), 40, 50];
    for y in 2..14 {
        for x in 4..12 {
            if y < 7 {
                put(a, t, x, y, skin[0] as i32, skin[1], skin[2], 255);
            } else {
                put(a, t, x, y, robe[0] as i32, robe[1], robe[2], 255);
            }
        }
    }
    put(a, t, 6, 4, 40, 30, 30, 255);
    put(a, t, 9, 4, 40, 30, 30, 255);
    // spell swirl
    put(a, t, 3, 8, 160, 110, 190, 255);
    put(a, t, 13, 8, 160, 110, 190, 255);
}

/// Vex: small pale ghost with sword.
pub(super) fn vex(a: &mut [u8], t: u16, rng: &mut Rng) {
    let body = [196 + jit(0, 14, rng), 206, 216];
    for y in 4..12 {
        for x in 5..11 {
            put(a, t, x, y, body[0] as i32, body[1], body[2], 255);
        }
    }
    put(a, t, 6, 6, 40, 30, 30, 255);
    put(a, t, 9, 6, 40, 30, 30, 255);
    // iron sword
    put(a, t, 12, 9, 210, 210, 215, 255);
    put(a, t, 12, 10, 150, 150, 158, 255);
}

/// Shulker box: purple box shell with a lighter lid band.
pub(super) fn shulker_box(a: &mut [u8], t: u16, rng: &mut Rng) {
    let shell = [138 + jit(0, 12, rng), 108, 168];
    let lid = [168, 140, 196];
    for y in 1..15 {
        for x in 1..15 {
            if y < 6 {
                put(a, t, x, y, lid[0], lid[1], lid[2], 255);
            } else {
                put(a, t, x, y, shell[0] as i32, shell[1], shell[2], 255);
            }
        }
    }
    // darker rim + face
    for x in 1..15 {
        put(a, t, x, 6, 96, 70, 120, 255);
        put(a, t, x, 15, 96, 70, 120, 255);
    }
    put(a, t, 6, 9, 230, 220, 240, 255);
    put(a, t, 9, 9, 230, 220, 240, 255);
}

/// Shulker shell item: a purple shell half.
pub(super) fn shulker_shell(a: &mut [u8], t: u16, rng: &mut Rng) {
    let shell = [150 + jit(0, 12, rng), 118, 180];
    for y in 4..14 {
        for x in 4..12 {
            let edge = y == 4 || y == 13 || x == 4 || x == 11;
            if edge {
                put(a, t, x, y, 108, 80, 132, 255);
            } else {
                put(a, t, x, y, shell[0] as i32, shell[1], shell[2], 255);
            }
        }
    }
}

/// Totem of undying: golden emerald-eyed figure.
pub(super) fn totem(a: &mut [u8], t: u16, rng: &mut Rng) {
    let gold = [232 + jit(0, 12, rng), 190, 62];
    for y in 3..14 {
        for x in 5..11 {
            put(a, t, x, y, gold[0] as i32, gold[1], gold[2], 255);
        }
    }
    // emerald eyes
    put(a, t, 6, 6, 60, 200, 120, 255);
    put(a, t, 9, 6, 60, 200, 120, 255);
    // arms out
    for y in 8..10 {
        put(a, t, 4, y, gold[0] as i32, gold[1], gold[2], 255);
        put(a, t, 11, y, gold[0] as i32, gold[1], gold[2], 255);
    }
}

/// 1.11 spawn-egg palettes (egg order 23..=28 = llama, vindicator,
/// evoker, vex, husk, stray — the re-added zombie-villager egg keeps
/// its pre-existing tile in the base window). The E1/E2 egg_art renders
/// the shell + spots from these pairs; colors are clean-room approximations
/// of each mob's vanilla egg (disclosed — no Mojang assets).
pub const V7_EGG_PALETTES: [(i32, i32, i32, i32, i32, i32); 6] = [
    (226, 216, 196, 139, 90, 43),   // llama: cream body + brown spots
    (158, 158, 152, 58, 54, 62),    // vindicator: grey skin + dark robe
    (178, 172, 166, 216, 191, 120), // evoker: pale skin + gold accents
    (170, 190, 200, 40, 60, 70),    // vex: pale blue + dark slate
    (135, 110, 75, 80, 60, 40),     // husk: sandy brown + dark husk
    (208, 224, 230, 100, 120, 140), // stray: icy white + grey-blue rags
];
