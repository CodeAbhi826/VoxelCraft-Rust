//! `armor_art.rs` — Sub-round 3 (2026-09-15): the 16 clean-room armor
//! item sprites.
//!
//! clean-room: shape language = the four wearable-armor silhouettes
//! (helmet dome with brim + face-guard hint, chestplate torso with
//! shoulder flanges, leggings waist-to-ankle, ankle boot) drawn as
//! ASCII-grid sprites with per-material palettes (leather tans, iron
//! silvers, gold yellows, diamond cyans). reference fact(s):
//! reference wiki /Armor (live 2026-09-15): the four armor pieces per
//! material and the vanilla defense values (leather 1/3/2/1, golden
//! 2/5/3/1, iron 2/6/5/2, diamond 3/8/6/3). no third-party asset was read,
//! copied, or traced — these are original pixel grids in the same
//! *shape family*, not recreations of vanilla pixels.

use super::art;

// ---- material palettes: (light, base, dark, edge) ----

const LEATHER: [(i32, i32, i32); 4] = [
    (166, 113, 54), // light tan
    (143, 90, 40),  // base tan
    (107, 66, 24),  // dark brown
    (74, 46, 17),   // edge
];
const IRON: [(i32, i32, i32); 4] = [
    (216, 216, 220),
    (176, 176, 184),
    (122, 122, 130),
    (84, 84, 92),
];
const GOLD: [(i32, i32, i32); 4] = [
    (253, 245, 95),
    (233, 197, 49),
    (176, 139, 22),
    (120, 92, 12),
];
const DIAMOND: [(i32, i32, i32); 4] = [
    (99, 236, 224),
    (47, 197, 192),
    (27, 156, 154),
    (16, 108, 110),
];

fn material_palette(mat: u8) -> &'static [(i32, i32, i32); 4] {
    match mat {
        0 => &LEATHER,
        1 => &IRON,
        2 => &GOLD,
        _ => &DIAMOND,
    }
}

/// the helmet sprite — a domed cap with a brim and a nose-guard hint.
fn helmet_rows() -> [&'static str; 16] {
    [
        "................",
        "................",
        "................",
        "......LLLL......",
        ".....LBBBBL.....",
        "....LBBBBBBL....",
        "...LBBBBBBBBL...",
        "..LLBBBBBBBBLL..",
        "..LLBBBBBBBBLL..",
        "..LELBBBBBBLEL..",
        "..LEELBBBBLEEL..",
        "...LEEELBLEEE...",
        "....EEEEEEEE....",
        "................",
        "................",
        "................",
    ]
}

/// the chestplate sprite — a torso with shoulder flanges.
fn chestplate_rows() -> [&'static str; 16] {
    [
        "................",
        "..LLL......LLL..",
        "..LBL......LBL..",
        "..LBBL....LBBL..",
        "..LBBBL..LBBBL..",
        "...LBBBBBBBBL...",
        "...LBBBBBBBBL...",
        "...LBBiBBiBBL...",
        "...LBBiBBiBBL...",
        "...LBBBBBBBBL...",
        "...LBBiBBiBBL...",
        "....LBBBBBBL....",
        "....LEEEEEEL....",
        "................",
        "................",
        "................",
    ]
}

/// the leggings sprite — a waistband over two legs.
fn leggings_rows() -> [&'static str; 16] {
    [
        "................",
        "................",
        "................",
        "...LLLLLLLLLL...",
        "...LBBBBBBBBL...",
        "...LBBBBBBBBL...",
        "...LBBL..LBBB...",
        "...LBB....BBB...",
        "...LBB....BBB...",
        "...LBB....BBB...",
        "...LBB....BBB...",
        "...LEE....EEE...",
        "................",
        "................",
        "................",
        "................",
    ]
}

/// the boots sprite — an ankle boot with sole.
fn boots_rows() -> [&'static str; 16] {
    [
        "................",
        "................",
        "................",
        "................",
        ".....LLLLLL.....",
        ".....LBBBBL.....",
        ".....LBBBBL.....",
        ".....LBBBBL.....",
        ".....LBBBBL.....",
        "....LBBBBBL.....",
        "...LBBBBBBL.....",
        "...LBBBBBBLL....",
        "..LLEEEEELLL....",
        "..EEEEEEEEEE....",
        "................",
        "................",
    ]
}

/// draw one armor sprite: `t` = the atlas tile, `mat` 0..=3
/// (leather/iron/gold/diamond), `piece` 0..=3 (helmet/chest/legs/boots).
pub(super) fn armor_art(a: &mut [u8], t: u16, mat: u8, piece: u8) {
    let pal = material_palette(mat);
    let rows: [&'static str; 16] = match piece {
        0 => helmet_rows(),
        1 => chestplate_rows(),
        2 => leggings_rows(),
        _ => boots_rows(),
    };
    art(a, t, rows, &|c| match c {
        'L' => Some((pal[0].0, pal[0].1, pal[0].2, 255)),
        'B' => Some((pal[1].0, pal[1].1, pal[1].2, 255)),
        'E' => Some((pal[2].0, pal[2].1, pal[2].2, 255)),
        'i' => Some((pal[3].0, pal[3].1, pal[3].2, 255)),
        _ => None,
    });
}
