//! Phase E1 procedural tiles (evolution 1.0–1.2 bracket) — clean-room
//! art, a child module of textures.rs (shares put/jit/noise_fill/art).

use super::{art, jit, noise_fill, put, Rng};

/// Mycelium top: pale grey-purple mottled fungal mat (mushroom-fields
/// surface). Distinct from grass (green) and snow (white).
pub(super) fn mycelium_top(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [124, 105, 124];
    noise_fill(a, t, base, 14, rng);
    // sparse darker speckle (fungal patches)
    for _ in 0..14 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 96, 78, 96, 255);
    }
}

/// Mycelium side: dirt body with a mycelium cap band on top.
pub(super) fn mycelium_side(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            if y < 3 {
                put(a, t, x, y, 118 + jit(0, 10, rng), 100 + jit(0, 10, rng), 118 + jit(0, 10, rng), 255);
            } else {
                put(a, t, x, y, 134 + jit(0, 12, rng), 96 + jit(0, 12, rng), 67 + jit(0, 12, rng), 255);
            }
        }
    }
}

/// End stone: pale cream-yellow, wart-like mottle (the End island surface).
pub(super) fn end_stone(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [221, 223, 165];
    noise_fill(a, t, base, 9, rng);
    // darker blotches give the "warted" vanilla look
    for _ in 0..10 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 196, 199, 141, 255);
    }
}

/// Nether bricks: dark crimson mortar rows (the fortress material).
pub(super) fn nether_bricks(a: &mut [u8], t: u16, rng: &mut Rng) {
    let brick = [68, 40, 44];
    let mortar = [40, 22, 24];
    for y in 0..16 {
        for x in 0..16 {
            let row = y / 4;
            let offset = if row % 2 == 0 { 0 } else { 4 };
            let in_mortar_h = y % 4 == 3;
            let in_mortar_v = (x + offset) % 8 == 7;
            let s = if in_mortar_h || in_mortar_v { mortar } else { brick };
            put(a, t, x, y, jit(s[0], 6, rng), jit(s[1], 6, rng), jit(s[2], 6, rng), 255);
        }
    }
}

/// Redstone lamp (off): amber glass grid on a dark frame — glowstone-like
/// but dim; the lit variant swaps to bright yellow cores.
pub(super) fn redstone_lamp(a: &mut [u8], t: u16, rng: &mut Rng, lit: bool) {
    let frame = if lit { [120, 92, 50] } else { [66, 50, 30] };
    let cell = if lit { [255, 216, 80] } else { [148, 108, 58] };
    let hot = if lit { Some([255, 244, 160]) } else { None };
    for y in 0..16 {
        for x in 0..16 {
            // 3x3 glowing cells with 1px frame (grid at 0,6,12..)
            let fx = x % 5 == 4;
            let fy = y % 5 == 4;
            let border = x == 0 || y == 0 || x == 15 || y == 15;
            let c = if fx || fy || border {
                frame
            } else {
                match hot {
                    // lit: hot cores at the center of each cell
                    Some(h) if (x % 5 == 2) && (y % 5 == 2) => h,
                    _ => cell,
                }
            };
            put(a, t, x, y, jit(c[0], 8, rng), jit(c[1], 8, rng), jit(c[2], 8, rng), 255);
        }
    }
}

/// Chiseled stone bricks: stone-brick base with a darker carved square.
pub(super) fn chiseled_stone_bricks(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [122, 121, 121];
    let carved = [92, 91, 91];
    for y in 0..16 {
        for x in 0..16 {
            let in_ring = (4..12).contains(&x) && (4..12).contains(&y);
            let on_ring_edge = in_ring && (x == 4 || x == 11 || y == 4 || y == 11);
            let c = if on_ring_edge { carved } else { base };
            put(a, t, x, y, jit(c[0], 6, rng), jit(c[1], 6, rng), jit(c[2], 6, rng), 255);
        }
    }
}

/// Chiseled sandstone: sand base with a carved sunken face.
pub(super) fn chiseled_sandstone(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [216, 203, 155];
    let carved = [188, 174, 126];
    for y in 0..16 {
        for x in 0..16 {
            let in_ring = (5..11).contains(&x) && (3..13).contains(&y);
            let on_edge = in_ring && (x == 5 || x == 10 || y == 3 || y == 12);
            let c = if on_edge { carved } else { base };
            put(a, t, x, y, jit(c[0], 6, rng), jit(c[1], 6, rng), jit(c[2], 6, rng), 255);
        }
    }
}

/// Cut sandstone: stacked chiseled horizontal bands.
pub(super) fn cut_sandstone(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [216, 203, 155];
    let cut = [194, 180, 132];
    for y in 0..16 {
        for x in 0..16 {
            let band = (y / 4) % 2 == 1;
            let groove = y % 4 == 0;
            let c = if groove { cut } else if band { cut } else { base };
            put(a, t, x, y, jit(c[0], 6, rng), jit(c[1], 6, rng), jit(c[2], 6, rng), 255);
        }
    }
}

/// Smooth sandstone: near-uniform pale sand.
pub(super) fn smooth_sandstone(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [220, 207, 160], 5, rng);
}

/// Huge red mushroom cap block: red skin with pale pores.
pub(super) fn mushroom_block_red(a: &mut [u8], t: u16, rng: &mut Rng) {
    let skin = [183, 36, 33];
    let pore = [225, 218, 210];
    for y in 0..16 {
        for x in 0..16 {
            // pores on a 4-cell grid, skins between
            let pore_cell = (x % 4 == 1 || x % 4 == 2) && (y % 4 == 1 || y % 4 == 2);
            let c = if pore_cell { pore } else { skin };
            put(a, t, x, y, jit(c[0], 10, rng), jit(c[1], 10, rng), jit(c[2], 10, rng), 255);
        }
    }
}

/// Huge brown mushroom cap block: tan skin with pale pores.
pub(super) fn mushroom_block_brown(a: &mut [u8], t: u16, rng: &mut Rng) {
    let skin = [151, 118, 85];
    let pore = [196, 180, 155];
    for y in 0..16 {
        for x in 0..16 {
            let pore_cell = (x % 4 == 1 || x % 4 == 2) && (y % 4 == 1 || y % 4 == 2);
            let c = if pore_cell { pore } else { skin };
            put(a, t, x, y, jit(c[0], 8, rng), jit(c[1], 8, rng), jit(c[2], 8, rng), 255);
        }
    }
}

/// Huge mushroom stem: pale ridged stalk.
pub(super) fn mushroom_stem(a: &mut [u8], t: u16, rng: &mut Rng) {
    let base = [205, 196, 178];
    let ridge = [180, 170, 150];
    for y in 0..16 {
        for x in 0..16 {
            let ridge_col = (x / 2) % 2 == 1;
            let c = if ridge_col { ridge } else { base };
            put(a, t, x, y, jit(c[0], 7, rng), jit(c[1], 7, rng), jit(c[2], 7, rng), 255);
        }
    }
}

/// Nether wart crop stage 0..3 (VERIFIED: 4 stages; stage art grows with
/// the age — tiny sprout to a full bushy cluster).
pub(super) fn nether_wart_art(a: &mut [u8], t: u16, stage: u8) {
    // rows used per stage: sprout (2) → small (4) → tall (6) → bushy (8)
    let rows: [&str; 16] = match stage {
        0 => [
            "................", "................", "................", "................",
            "................", "................", "................", "................",
            "................", "................", "................", "......W.........",
            "......W.........", "......W.........", "......W.........", "................",
        ],
        1 => [
            "................", "................", "................", "................",
            "................", "................", "......W.........", ".....WWW........",
            ".....WWW........", "..W..WWW..W.....", "..W..WWW..W.....", "..WW.WWW.WW.....",
            "...WWWWW........", "....WWWW........", ".....WWW........", "......W.........",
        ],
        2 => [
            "................", "................", "......W.........", ".....WWW........",
            ".....WWW........", "..W..WWW..W.....", "..WW.WWW.WW.....", "..WWWWWWWWW.....",
            "...WWWWWWW......", "...WWWWWWW......", "..WWWWWWWWW.....", "..WWWWWWWWW.....",
            "...WWWWWWW......", "....WWWWW.......", ".....WWW........", "......W.........",
        ],
        _ => [
            ".....W..W.......", "....WWWWW..W....", "...WWWWWWW......", "..WWWWWWWWW.....",
            "..WWWWWWWWW.....", ".WWWWWWWWWWW....", ".WWWWWWWWWWW....", ".WWWWWWWWWWW....",
            "..WWWWWWWWW.....", "..WWWWWWWWW.....", "...WWWWWWW......", "...WWWWWWW......",
            "....WWWWW.......", ".....WWW........", "......W.........", "......W.........",
        ],
    };
    art(a, t, rows, &|c| match c {
        'W' => Some((165, 22, 22, 255)),
        _ => None,
    });
}

/// Dragon egg: near-black ovoid with purple sheen dots.
pub(super) fn dragon_egg_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        ".....KKKKK......",
        "....KKKKKKK.....",
        "...KKKKKKKKK....",
        "...KKPKKKKKK....",
        "..KKKPKKKKKKK...",
        "..KKKKKKKKKKK...",
        "..KKKPKKKKKKK...",
        "..KKKKKKKKKKK...",
        "...KKKKKKKKK....",
        "...KKKKKKKKK....",
        "....KKKKKKK.....",
        ".....KKKKK......",
        "......KKK.......",
        ".......K........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((18, 10, 24, 255)),
        'P' => Some((96, 42, 128, 255)),
        _ => None,
    });
}

/// End portal face: deep void purple with a star field.
pub(super) fn end_portal_art(a: &mut [u8], t: u16) {
    let rows = [
        "VVVVVVVVVVVVVVVV",
        "VVVVVVSVVVVVVVVV",
        "VVVVVVVVVVSVVVVV",
        "VVSVVVVVVVVVVVVV",
        "VVVVVVVVVVVVVSVV",
        "VVVVVVSVVVVVVVVV",
        "VVVVVVVVVVVVVVVV",
        "VSVVVVVVVVSVVVVV",
        "VVVVVVVVVVVVVVVV",
        "VVVVVVVVVVVVVVVV",
        "VVVSVVVVVVVVVSVV",
        "VVVVVVVVVVVVVVVV",
        "VVVVVVVVVSVVVVVV",
        "VVVVVVSVVVVVVVVV",
        "VVVVVVVVVVVVVVVV",
        "VVVVVVVVVVVVVVVV",
    ];
    art(a, t, rows, &|c| match c {
        'V' => Some((12, 8, 20, 255)),
        'S' => Some((126, 200, 190, 255)),
        _ => None,
    });
}

/// End crystal item/entity icon: pale crystal prism on a bedrock base.
pub(super) fn end_crystal_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".......C........",
        "......CCC.......",
        "......CWC.......",
        ".....CCWCC......",
        ".....CCWCC......",
        "....CCCWCCC.....",
        ".....CCWCC......",
        "......CCC.......",
        "......BBB.......",
        ".....BBBBB......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'C' => Some((222, 190, 240, 255)),
        'W' => Some((255, 255, 255, 255)),
        'B' => Some((70, 70, 74, 255)),
        _ => None,
    });
}

/// XP orb: glowing green-yellow ball (big variant for value ≥ 17 with a
/// dense orange core — VERIFIED w/Experience).
pub(super) fn xp_orb_art(a: &mut [u8], t: u16, big: bool) {
    let rows: [&str; 16] = if big {
        [
            "................", "................", "................", "................",
            "......GGG.......", "....GGYYGG......", "...GGYOOYGG.....", "...GYOOOOGY.....",
            "...GYOOOOGY.....", "...GGYOOYGG.....", "....GGYYGG......", "......GGG.......",
            "................", "................", "................", "................",
        ]
    } else {
        [
            "................", "................", "................", "................",
            "................", "................", "......GG........", ".....GYYG.......",
            ".....GYYG.......", "......GG........", "................", "................",
            "................", "................", "................", "................",
        ]
    };
    art(a, t, rows, &|c| match c {
        'G' => Some((96, 220, 60, 255)),
        'Y' => Some((180, 240, 70, 255)),
        'O' => Some((230, 150, 40, 255)),
        _ => None,
    });
}

/// Eye of ender item: green pupil in a pearl shell.
pub(super) fn eye_of_ender_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".....PPPP.......",
        "....PPGGPP......",
        "...PPGGGGPP.....",
        "...PGGBBGGP.....",
        "...PGGBBGGP.....",
        "...PPGGGGPP.....",
        "....PPGGPP......",
        ".....PPPP.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((26, 84, 32, 255)),
        'G' => Some((120, 190, 90, 255)),
        'B' => Some((40, 50, 40, 255)),
        _ => None,
    });
}

/// Blaze rod: golden ember rod.
pub(super) fn blaze_rod_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "...........GG...",
        "..........GGG...",
        ".........GGG....",
        "........GGG.....",
        ".......GGG......",
        "......GGG.......",
        ".....GGG........",
        "....GGG.........",
        "...GGG..........",
        "...GG...........",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((240, 190, 70, 255)),
        _ => None,
    });
}

/// Blaze powder: a small pile of golden dust.
pub(super) fn blaze_powder_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "......GG........",
        ".....GGGG.......",
        "....GGGGGG......",
        "...GGGGGGGG.....",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((238, 170, 60, 255)),
        _ => None,
    });
}

/// Golden apple: shining gold apple with a stem.
pub(super) fn golden_apple_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "........S.......",
        ".......S........",
        ".....GGGGG......",
        "....GGGGGGG.....",
        "...GGWGGGGGG....",
        "...GWGGGGGGG....",
        "...GWGGGGGGG....",
        "...GGGGGGGGG....",
        "...GGGGGGGGG....",
        "....GGGGGGG.....",
        ".....GGGGG......",
        "......GGG.......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((246, 200, 60, 255)),
        'W' => Some((255, 245, 180, 255)),
        'S' => Some((108, 74, 42, 255)),
        _ => None,
    });
}

/// Snowball: packed white ball with a shading edge.
pub(super) fn snowball_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......WWWW......",
        "....WWWWWWWW....",
        "...WWWWWWWWWW...",
        "...WWWWWWWWWW...",
        "...WWWwWWWWWW...",
        "...WWWWWWWWWW...",
        "....WWWWWWWW....",
        "......WWWW......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((242, 248, 250, 255)),
        'w' => Some((206, 216, 222, 255)),
        _ => None,
    });
}

/// Nether brick item: a single small dark brick.
pub(super) fn nether_brick_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "....BBBBBBBB....",
        "....BBBBBBBB....",
        "....BBBBBBBB....",
        "....BBBBBBBB....",
        "....BBBBBBBB....",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((60, 34, 38, 255)),
        _ => None,
    });
}

/// Spawn egg: clean-room egg silhouette with a base color + darker spots.
/// Two-tone parameterization — the 16 eggs get distinct palette pairs.
pub(super) fn egg_art(a: &mut [u8], t: u16, base: (i32, i32, i32), spots: (i32, i32, i32)) {
    let rows = [
        "................",
        "................",
        "................",
        "......XX........",
        ".....XXXX.......",
        ".....XXXX.......",
        "....XXXXXX......",
        "....XsXXXX......",
        "....XXXsXX......",
        "....XXXXXX......",
        "...XXXXsXXX.....",
        "...XXsXXXXX.....",
        "...XXXXXXXXX....",
        "....XXXXXXX.....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'X' => Some((base.0, base.1, base.2, 255)),
        's' => Some((spots.0, spots.1, spots.2, 255)),
        _ => None,
    });
}

/// The 16 spawn-egg palettes, in egg-id order (0..=15 — must match
/// vc_gameplay::mobs::egg_mob_kind). Distinct two-tone pairs, ours.
pub const EGG_PALETTES: [(i32, i32, i32, i32, i32, i32); 16] = [
    (235, 240, 245, 180, 130, 40),   // snow golem: white + pumpkin
    (58, 22, 22, 240, 130, 40),      // magma cube: dark red + ember
    (250, 180, 60, 120, 40, 20),     // blaze: gold + ember red
    (235, 200, 90, 60, 40, 30),      // ocelot: yellow + spots
    (190, 190, 200, 90, 60, 40),     // iron golem: iron + rust
    (70, 110, 70, 60, 90, 60),       // zombie villager: sickly green + robe
    (190, 60, 60, 235, 235, 235),    // mooshroom: red + white
    (60, 110, 60, 70, 40, 40),       // zombie
    (200, 200, 200, 80, 80, 80),     // skeleton
    (90, 200, 90, 40, 90, 40),       // creeper
    (70, 60, 60, 140, 40, 40),       // spider
    (20, 15, 20, 130, 70, 160),      // enderman
    (80, 60, 40, 230, 230, 230),     // cow
    (235, 170, 170, 200, 120, 120),  // pig
    (235, 235, 235, 240, 200, 200),  // sheep
    (240, 240, 240, 200, 160, 40),  // chicken
];

// ---- Phase E1 mob sprites (clean-room, ours) ----

pub(super) fn snow_golem_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        ".....PPPPP......",
        "....PPPPPPP.....",
        "....PpPPPpP.....",
        "....PPPPPPP.....",
        ".....PPPPP......",
        "....WWWWWWW.....",
        "...WWWWWWWWW....",
        "...WWwWWwWWW....",
        "...WWWWWWWWW....",
        "....WWWWWWW.....",
        "...WWWWWWWWW....",
        "..WWWWWWWWWWW...",
        "..WWWWWWWWWWW...",
        "...WWWWWWWWW....",
        "....WWWWWWW.....",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((240, 246, 250, 255)),
        'w' => Some((210, 218, 224, 255)),
        'P' => Some((214, 124, 32, 255)),
        'p' => Some((60, 40, 20, 255)),
        _ => None,
    });
}

pub(super) fn magma_cube_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "..MMMMMMMMMM....",
        "..MMMMMMMMMM....",
        "..MMYMMMMYMM....",
        "..MMYMMMMYMM....",
        "..MMMMMMMMMM....",
        "..MMMMooMMMM....",
        "..MMMooooMMM....",
        "..MMMMooMMMM....",
        "..MMMMMMMMMM....",
        "..MMMMMMMMMM....",
        "..MMMMMMMMMM....",
        "..MMMMMMMMMM....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((62, 30, 30, 255)),
        'Y' => Some((250, 190, 60, 255)),
        'o' => Some((240, 120, 30, 255)),
        _ => None,
    });
}

pub(super) fn blaze_art(a: &mut [u8], t: u16) {
    let rows = [
        "......ff........",
        ".....ffff.......",
        "....YYoYYY......",
        "...YYYoYYYo.....",
        "...YYoYYoYY.....",
        "...YYYYYYYY.....",
        "....foYYof......",
        "..f...YY...f....",
        "..f..fYYf..f....",
        "....f.YY.f......",
        "..f...YY...f....",
        ".....fYYf.......",
        "....f.YY.f......",
        "..f...ff...f....",
        "......ff........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'Y' => Some((244, 190, 70, 255)),
        'o' => Some((60, 40, 20, 255)),
        'f' => Some((240, 150, 40, 200)),
        _ => None,
    });
}

pub(super) fn ocelot_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".....YYo........",
        "....YYYY........",
        "....YyYYo.......",
        "....YYYY........",
        "...sYYYYs.......",
        "...sYYYYYYs.....",
        "...sYYYYYYs.....",
        "...sYYYYYYs.....",
        "...sYYYYYYo.....",
        "...sYYYYs.......",
        "...sYY.sYs......",
        "...sY...s.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'Y' => Some((226, 194, 96, 255)),
        'y' => Some((170, 130, 60, 255)),
        'o' => Some((60, 45, 30, 255)),
        's' => Some((40, 45, 55, 255)),
        _ => None,
    });
}

pub(super) fn iron_golem_art(a: &mut [u8], t: u16) {
    let rows = [
        "....IIIIII......",
        "....IivvI.......",
        "....IvvIi.......",
        "....IIIIII......",
        ".....IIII.......",
        "..IIIIIIIIII....",
        "..IIIIIIIIII....",
        "..IIIIIIIIII....",
        "..IiIIIIIIiI....",
        "...IIIIIIII.....",
        "...IIIvvIII.....",
        "...IIIIIIII.....",
        "...III..III.....",
        "...III..III.....",
        "...III..III.....",
        "..IIII..IIII....",
    ];
    art(a, t, rows, &|c| match c {
        'I' => Some((188, 188, 196, 255)),
        'i' => Some((150, 150, 160, 255)),
        'v' => Some((70, 110, 60, 255)),
        _ => None,
    });
}

pub(super) fn zombie_villager_art(a: &mut [u8], t: u16) {
    let rows = [
        "....GGGGGG......",
        "....GGGGGG......",
        "....GKGKGG......",
        "....GGGGGG......",
        "....GGNNNG......",
        "....GGNNGG......",
        ".....GGGG.......",
        "...RRRGGRRR.....",
        "..RRRRGGRRRR....",
        "..RRRRRRRRRR....",
        "..RRRRRRRRRR....",
        "..RRRRRRRRRR....",
        "..RRRRRRRRRR....",
        "...RRRRRRRR.....",
        "...RRRRRRRR.....",
        "...RR..RR.RR....",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((90, 130, 90, 255)),
        'K' => Some((30, 40, 30, 255)),
        'N' => Some((110, 150, 110, 255)),
        'R' => Some((110, 92, 66, 255)),
        _ => None,
    });
}

pub(super) fn mooshroom_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "..RR..RR..R.....",
        "..RRRRRRRRRR....",
        "..RRRRRRRRRR....",
        "...RRRRRRRRR....",
        "...RRwwRRRRR....",
        "...RRRRRRRRR....",
        "...RRRRRRRR.....",
        "...RRRRRRRRR....",
        "....RRRRRRR.....",
        "....RRRRRRR.....",
        "....RRRRRRR.....",
        "....RR...RR.....",
        "....RR...RR.....",
        "....RR...RR.....",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((190, 60, 60, 255)),
        'w' => Some((240, 240, 240, 255)),
        _ => None,
    });
}

pub(super) fn ender_dragon_art(a: &mut [u8], t: u16) {
    let rows = [
        "W..............W",
        "WW....DDDD....WW",
        ".WWW.DDDDDD.WWW.",
        "..WWDDKDDKDDWW..",
        "...WDDDDDDDDW...",
        "....DDDKDDD.....",
        "...PDDDDDDDP...",
        "..PPDDDDDDDPP...",
        "..PP.DDDDD.PP...",
        ".....DDDDD......",
        ".....DDDDD......",
        "....DD...DD.....",
        "....D.....D.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((32, 28, 40, 255)),
        'K' => Some((214, 90, 200, 255)),
        'W' => Some((90, 78, 110, 255)),
        'P' => Some((120, 100, 150, 255)),
        _ => None,
    });
}

