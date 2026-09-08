//! 1.14 bracket (Village & Pillage — nature half, part 3) procedural
//! tiles — the flower window (tiles 640..=641): the cornflower sprite
//! (deep-blue petal cluster on a leafy stem) and the lily of the
//! valley sprite (white bell florets along a stem with basal leaves).
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Covered by the `v114b_tiles_all_painted`
//! atlas guard (its loop runs 634..=TILE_MAX, so the new window rides
//! the same art-gap regression guard).

use super::{art, Rng};

/// cornflower petal tones (the deep-blue cluster + outline + sheen).
const CORN_D: [i32; 3] = [40, 58, 138];
const CORN_B: [i32; 3] = [74, 104, 200];
const CORN_W: [i32; 3] = [148, 168, 236];
/// shared plant greens (stem + leaf).
const STEM: [i32; 3] = [62, 124, 52];
const LEAF: [i32; 3] = [78, 144, 66];
/// lily of the valley whites (bell body + shaded rim).
const LILY_W: [i32; 3] = [246, 248, 250];
const LILY_w: [i32; 3] = [212, 216, 224];

/// cornflower — the 1.14 deep-blue small flower (cross plant, the
/// allium layout: petal blob at the crown, stem with paired leaves).
/// VERIFIED w/Cornflower (18w43a, non-solid, plains/flower-forest).
pub(super) fn cornflower_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        ".....DDDDD......",
        "....DBBBBBD.....",
        "...DBBBWBBBD....",
        "...DBBWWWBBD....",
        "...DBBWWWBBD....",
        "....DBBBBBD.....",
        ".....DDBBD......",
        "......SS........",
        "..gg...SS...gg..",
        "...gg..SS..gg...",
        "......SS........",
        "......SS........",
        ".....gSSg.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((CORN_D[0], CORN_D[1], CORN_D[2], 255)),
        'B' => Some((CORN_B[0], CORN_B[1], CORN_B[2], 255)),
        'W' => Some((CORN_W[0], CORN_W[1], CORN_W[2], 255)),
        'S' => Some((STEM[0], STEM[1], STEM[2], 255)),
        'g' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
        _ => None,
    });
}

/// lily of the valley — the 1.14 white-bell small flower (cross
/// plant): bell florets hanging off a central stem, broad basal
/// leaves. VERIFIED w/Lily_of_the_Valley (18w43a, forest floors).
pub(super) fn lily_of_the_valley_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "......WW........",
        ".....WwwW.......",
        "......WW........",
        "...WW..S..WW....",
        "...WwwW.S.WwwW..",
        "....WW..S..WW...",
        ".........S......",
        "......WW.S......",
        "......wWS.......",
        ".......SS.......",
        "..gg...SS...gg..",
        "...gg..SS..gg...",
        ".....ggSSgg.....",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((LILY_W[0], LILY_W[1], LILY_W[2], 255)),
        'w' => Some((LILY_w[0], LILY_w[1], LILY_w[2], 255)),
        'S' => Some((STEM[0], STEM[1], STEM[2], 255)),
        'g' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
        _ => None,
    });
}
