//! Backlog round (farming, 2026-09-09) procedural tiles — the farming
//! window (tiles 739..=763): dry + hydrated farmland, wheat's 8 growth
//! stages, the carrot/potato 4-stage root-crop ladders (mapped onto the
//! 8-age blockstate ladder like vanilla), beetroot's 4 stages, and the
//! wheat / bread / hoe item sprites.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the put/art helpers). Guarded by the farming coverage test
//! so the 1.13 art-gap regression (a window with TILE_MAX raised but no
//! painters renders blank) can never repeat.
//!
//! Vanilla behavior anchors (captured 2026-09-09, see
//! scripts/backlog_page_*.json): Farmland (dry/wet visuals), Wheat_
//! Crops §Block states (age 0-7, golden at maturity), Carrot/Potato
//! (4-texture/8-stage mapping), Beetroot_Seeds (age 0-3).

use super::{art, put, Rng};

// ---- palette (clean-room tones) ----
const SOIL_L: [i32; 3] = [134, 96, 67]; // dry tilled soil
const SOIL_D: [i32; 3] = [96, 66, 46]; // dry furrow shadow
const SOIL_WET_L: [i32; 3] = [92, 62, 44]; // wet soil
const SOIL_WET_D: [i32; 3] = [66, 44, 32]; // wet furrow
const SPROUT: [i32; 3] = [106, 170, 64]; // young green
const STALK_G: [i32; 3] = [126, 178, 66]; // green stalk
const STALK_Y: [i32; 3] = [196, 168, 62]; // ripening straw
const STALK_GOLD: [i32; 3] = [219, 192, 86]; // golden straw
const HEAD: [i32; 3] = [228, 201, 96]; // wheat head
const HEAD_D: [i32; 3] = [184, 152, 62]; // wheat head awn shadow
const CARROT_TOP: [i32; 3] = [74, 136, 58]; // carrot leaves
const CARROT_TOP_D: [i32; 3] = [52, 100, 42];
const CARROT_ROOT: [i32; 3] = [214, 108, 34]; // the orange shoulder
const POTATO_TOP: [i32; 3] = [88, 142, 66];
const POTATO_TOP_D: [i32; 3] = [60, 104, 48];
const POTATO_ROOT: [i32; 3] = [198, 172, 110]; // the tan shoulder
const BEET_LEAF: [i32; 3] = [96, 150, 70];
const BEET_LEAF_D: [i32; 3] = [70, 112, 52];
const BEET_ROOT: [i32; 3] = [142, 48, 84]; // the beet shoulder
const BREAD_C: [i32; 3] = [199, 148, 86]; // crust
const BREAD_C_D: [i32; 3] = [168, 118, 64]; // crust shade
const BREAD_IN: [i32; 3] = [236, 199, 142]; // inner crumb
const WOOD: [i32; 3] = [158, 116, 68]; // hoe handle
const IRON: [i32; 3] = [176, 176, 182]; // hoe blade
const IRON_D: [i32; 3] = [130, 130, 138];

/// farmland tile — seen from above: tilled rows (furrows run along X,
/// the hoe's drag lines). `wet` swaps to the dark waterlogged palette
/// (moisture 1..7, VERIFIED w/Farmland §Hydration).
pub(super) fn farmland_art(a: &mut [u8], t: u16, wet: bool) {
    let (l, d) = if wet { (SOIL_WET_L, SOIL_WET_D) } else { (SOIL_L, SOIL_D) };
    // the top strip (y 0..4) shows a touch of un-tilled crust — the
    // block reads as dirt from the side; the furrow body below
    for y in 0..16usize {
        for x in 0..16usize {
            // furrow bands: 3-px ridges with 2-px grooves, offset per row
            let ridge = ((x + y * 5) % 5) < 3;
            let (r, g, b) = if y < 3 {
                // the crust crown
                (l[0] + 10, l[1] + 6, l[2] + 4)
            } else if ridge {
                (l[0], l[1], l[2])
            } else {
                (d[0], d[1], d[2])
            };
            put(a, t, x as i32, y as i32, r, g, b, 255);
        }
    }
}

/// wheat growth stage art (0..=7): a short green sprout pair → tall
/// stalks → the golden heads of maturity (age 7 "Fully grown",
/// VERIFIED w/Wheat_Crops §Block states).
pub(super) fn wheat_art(a: &mut [u8], t: u16, stage: u8, _rng: &mut Rng) {
    let s = stage.min(7);
    // height profile: stage 0 → 3px, each stage +~1.6px, 7 → 14px
    let h = 3 + (s as usize * 11) / 7;
    let base = 15usize;
    let top = base - h;
    let golden = s >= 5;
    let stalk = if golden { STALK_Y } else { STALK_G };
    match s {
        0 => {
            // a single tiny sprout
            let rows = [
                "................", "................", "................",
                "................", "................", "................",
                "................", "................", "................",
                "................", "................", "................",
                "......g.........", ".....ggg........", "......g.........",
                "......g.........",
            ];
            art(a, t, rows, &|c| match c {
                'g' => Some((SPROUT[0], SPROUT[1], SPROUT[2], 255)),
                _ => None,
            });
        }
        1 | 2 => {
            // a few thin blades
            let rows = [
                "................", "................", "................",
                "................", "................", "................",
                "................", "......g...g.....", ".....g...g......",
                "..g...g..g...g..", "..g..g....g..g..", "...g.g...g.g....",
                "...g.g...g.g....", "....g.....g.....", "....g.....g.....",
                "....g.....g.....",
            ];
            art(a, t, rows, &|c| match c {
                'g' => Some((stalk[0], stalk[1], stalk[2], 255)),
                _ => None,
            });
        }
        _ => {
            // stalk columns + heads at the top when ripening
            for (i, x) in [3usize, 6, 9, 12].iter().enumerate() {
                let sway = if (s + i as u8) % 2 == 0 { 1i32 } else { -1 };
                for y in top..base {
                    // slight sway toward the top third
                    let dx = if y < top + h / 3 { sway } else { 0 };
                    let (r, g, b) = if golden && y < top + 4 {
                        (HEAD[0], HEAD[1], HEAD[2])
                    } else if golden {
                        (STALK_GOLD[0], STALK_GOLD[1], STALK_GOLD[2])
                    } else {
                        (stalk[0], stalk[1], stalk[2])
                    };
                    put(a, t, *x as i32 + dx, y as i32, r, g, b, 255);
                    if golden && y < top + 4 {
                        // the awn fringe beside each head
                        put(a, t, *x as i32 + dx + 1, y as i32, HEAD_D[0], HEAD_D[1], HEAD_D[2], 255);
                    }
                }
            }
        }
    }
}

/// root-crop stage art (0..=3) — the carrot/potato leafy tops; the
/// orange/tan root shoulders surface at stage 3 (maturity, when the
/// plant "pops" with produce, VERIFIED w/Carrot §Breaking 2-5).
pub(super) fn root_crop_art(a: &mut [u8], t: u16, carrot: bool, stage: u8, _rng: &mut Rng) {
    let (top, top_d, root) = if carrot {
        (CARROT_TOP, CARROT_TOP_D, CARROT_ROOT)
    } else {
        (POTATO_TOP, POTATO_TOP_D, POTATO_ROOT)
    };
    let s = stage.min(3);
    let (leaf_h, shoulder) = match s {
        0 => (2, false),
        1 => (5, false),
        2 => (8, false),
        _ => (11, true),
    };
    let base = 15usize;
    let top_y = base - leaf_h;
    for x in 2..14usize {
        if (x + 1) % 3 == 0 && x != 7 {
            continue; // gaps between the leaf clumps
        }
        for y in top_y..base {
            let clump_top = (y - top_y) < (leaf_h / 3);
            let (r, g, b) = if clump_top {
                (top[0], top[1], top[2])
            } else {
                (top_d[0], top_d[1], top_d[2])
            };
            // slight x sway for organic feel
            let sway = (((x * 7 + y * 3) % 5) as i32) - 2;
            if sway.abs() < 2 {
                put(a, t, x as i32 + if y < top_y + 2 { sway } else { 0 }, y as i32, r, g, b, 255);
            }
        }
    }
    // the surfaced root shoulders at maturity (two roots peeking out)
    if shoulder {
        for (x, y) in [(4usize, 13usize), (10, 13), (5, 14), (9, 14)] {
            put(a, t, x as i32, y as i32, root[0], root[1], root[2], 255);
        }
    }
}

/// beetroot stage art (0..=3): the purple-veined leaves rise with age;
/// at 3 the dark beet shoulders surface (mature, VERIFIED w/Beetroot_
/// Seeds §Farming: harvest yields 1 beetroot + 1-4 seeds).
pub(super) fn beetroot_art(a: &mut [u8], t: u16, stage: u8, _rng: &mut Rng) {
    let s = stage.min(3);
    let (leaf_h, beet) = match s {
        0 => (2, false),
        1 => (4, false),
        2 => (7, false),
        _ => (10, true),
    };
    let base = 15usize;
    let top_y = base - leaf_h;
    for x in 3..13usize {
        if (x + 2) % 4 == 0 && x != 7 {
            continue;
        }
        for y in top_y..base {
            // the leaves carry a faint purple blush (vanilla's
            // beetroot-leaf look — deep green with red veins)
            let blush = ((x * 5 + y * 3) % 7) < 3;
            let (r, g, b) = if blush {
                (BEET_LEAF_D[0] + 18, BEET_LEAF_D[1], BEET_LEAF_D[2] + 10)
            } else {
                (BEET_LEAF[0], BEET_LEAF[1], BEET_LEAF[2])
            };
            put(a, t, x as i32, y as i32, r, g, b, 255);
        }
    }
    if beet {
        for (x, y) in [(6usize, 13usize), (7, 13), (8, 13), (6, 14), (7, 14), (8, 14)] {
            put(a, t, x as i32, y as i32, BEET_ROOT[0], BEET_ROOT[1], BEET_ROOT[2], 255);
        }
    }
}

/// the wheat item sprite — three golden stalks side by side.
pub(super) fn wheat_item_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        ".....h..h..h....",
        "....hH.hH.hH....",
        "....hH.hH.hH....",
        "....sH.sH.sH....",
        "....s..s..s.....",
        "....s..s..s.....",
        "....s..s..s.....",
        ".....s.s.s......",
        ".....s.s.s......",
        ".....s.s.s......",
        ".....s.s.s......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'H' => Some((HEAD[0], HEAD[1], HEAD[2], 255)),
        'h' => Some((HEAD_D[0], HEAD_D[1], HEAD_D[2], 255)),
        's' => Some((STALK_GOLD[0], STALK_GOLD[1], STALK_GOLD[2], 255)),
        _ => None,
    });
}

/// the bread item sprite — the classic top-lit loaf with the scored
/// diagonal slashes.
pub(super) fn bread_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "....bbbbbbbb....",
        "...bBBBBBBBBb...",
        "..bBBiBBiBBiBb..",
        "..bBiBBiBBiBBb..",
        "..bBBBBBBBBBBb..",
        "..bBBBBBBBBBBb..",
        "...bBBBBBBBBb...",
        "....bbbbbbbb....",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'b' => Some((BREAD_C_D[0], BREAD_C_D[1], BREAD_C_D[2], 255)),
        'B' => Some((BREAD_C[0], BREAD_C[1], BREAD_C[2], 255)),
        'i' => Some((BREAD_IN[0], BREAD_IN[1], BREAD_IN[2], 255)),
        _ => None,
    });
}

/// the hoe item sprite — a diagonal wooden handle with the iron
/// tilling blade at the head (the classic silhouette).
pub(super) fn hoe_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".....ii.........",
        "....iiii........",
        "....iiiii.......",
        ".....iiid.......",
        "......d.........",
        "......w.........",
        "......w.........",
        "......w.........",
        "......w.........",
        ".....w..........",
        ".....w..........",
        ".....w..........",
        ".....w..........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'i' => Some((IRON[0], IRON[1], IRON[2], 255)),
        'd' => Some((IRON_D[0], IRON_D[1], IRON_D[2], 255)),
        'w' => Some((WOOD[0], WOOD[1], WOOD[2], 255)),
        _ => None,
    });
}
