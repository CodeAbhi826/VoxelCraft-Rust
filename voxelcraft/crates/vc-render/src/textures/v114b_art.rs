//! 1.14 bracket (Village & Pillage — nature half, part 2) procedural
//! tiles — the V11 window (tiles 634..=639): the blast furnace's dark
//! iron face (unlit + glowing lit variants), the smoker's log-walled
//! face (unlit + lit), the lantern sprite (cross-rendered like the
//! torch — the sitting and hanging forms share it; the model
//! difference is future work, disclosed), and the iron nugget item
//! icon.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Guarded by the `v114b_tiles_all_painted`
//! coverage test — the 1.13 art-gap regression guard.

use super::{art, Rng};

/// clean-room iron tones (the blast furnace body + frame).
const IRON_D: [i32; 3] = [58, 58, 64];
const IRON_M: [i32; 3] = [92, 92, 100];
const VENT: [i32; 3] = [38, 38, 42];
/// the lit opening's glow (core + rim).
const GLOW_O: [i32; 3] = [255, 148, 40];
const GLOW_Y: [i32; 3] = [255, 204, 66];
/// smoker log tones (the wood-walled variant).
const WOOD_D: [i32; 3] = [86, 62, 36];
const WOOD_L: [i32; 3] = [136, 100, 58];
/// lantern: iron frame + warm glass + flame core.
const LANT_GLASS: [i32; 3] = [255, 216, 110];
const LANT_FLAME: [i32; 3] = [255, 170, 70];
/// iron nugget gray + sheen.
const NUG: [i32; 3] = [198, 198, 204];
const NUG_SHEEN: [i32; 3] = [236, 236, 242];

/// blast furnace face — dark iron frame, metal body, a wide dark
/// opening (the lit variant's opening glows: VERIFIED w/Blast_Furnace
/// infobox "light: Yes (13) (when active)" — the visible fire that
/// produces it).
pub(super) fn blast_furnace_art(a: &mut [u8], t: u16, lit: bool, _rng: &mut Rng) {
    let rows = if lit {
        [
            "................",
            ".IIIIIIIIIIIIII.",
            ".IMMMMMMMMMMMMI.",
            ".IMMYYYYYYYYMMI.",
            ".IMMYOOOOOOYMMI.",
            ".IMMOOOOOOOOMMI.",
            ".IMMOOOOOOOOMMI.",
            ".IMMYOOOOOOYMMI.",
            ".IMMYYYYYYYYMMI.",
            ".IMMMMMMMMMMMMI.",
            ".IMMMMMMMMMMMMI.",
            ".IMMYYYYYYYYMMI.",
            ".IMMYOOOOOOYMMI.",
            ".IMMYYYYYYYYMMI.",
            ".IIIIIIIIIIIIII.",
            "................",
        ]
    } else {
        [
            "................",
            ".IIIIIIIIIIIIII.",
            ".IMMMMMMMMMMMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMMMMMMMMMMMI.",
            ".IMMMMMMMMMMMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IMMDDDDDDDDMMI.",
            ".IIIIIIIIIIIIII.",
            "................",
        ]
    };
    art(a, t, rows, &|c| match c {
        'I' => Some((IRON_D[0], IRON_D[1], IRON_D[2], 255)),
        'M' => Some((IRON_M[0], IRON_M[1], IRON_M[2], 255)),
        'D' => Some((VENT[0], VENT[1], VENT[2], 255)),
        'Y' => Some((GLOW_Y[0], GLOW_Y[1], GLOW_Y[2], 255)),
        'O' => Some((GLOW_O[0], GLOW_O[1], GLOW_O[2], 255)),
        _ => None,
    });
}

/// smoker face — log walls around a dark cooking vent (the lit variant
/// glows through the vent: VERIFIED w/Smoker infobox "light: Yes (13)
/// (when active)").
pub(super) fn smoker_art(a: &mut [u8], t: u16, lit: bool, _rng: &mut Rng) {
    let rows = if lit {
        [
            "WWWWWWWWWWWWWWWW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLYYYYLLLLLW",
            "WLLLLYOOOYLLLLLW",
            "WLLLLYOOOYLLLLLW",
            "WLLLYYOOOYYLLLLW",
            "WLLLYYOOOYYLLLLW",
            "WLLLLYOOOYLLLLLW",
            "WLLLLLYYYYLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLYYYYLLLLLW",
            "WLLLLLYYYYLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WWWWWWWWWWWWWWWW",
            "................",
        ]
    } else {
        [
            "WWWWWWWWWWWWWWWW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLDDDDDDLLLLW",
            "WLLLLDDDDDDLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLDDDDLLLLLW",
            "WLLLLLLLLLLLLLLW",
            "WWWWWWWWWWWWWWWW",
            "................",
        ]
    };
    art(a, t, rows, &|c| match c {
        'W' => Some((WOOD_D[0], WOOD_D[1], WOOD_D[2], 255)),
        'L' => Some((WOOD_L[0], WOOD_L[1], WOOD_L[2], 255)),
        'D' => Some((VENT[0], VENT[1], VENT[2], 255)),
        'Y' => Some((GLOW_Y[0], GLOW_Y[1], GLOW_Y[2], 255)),
        'O' => Some((GLOW_O[0], GLOW_O[1], GLOW_O[2], 255)),
        _ => None,
    });
}

/// lantern sprite (cross-rendered, like the torch): a framed box
/// lantern with a chain handle, warm glass and a flame core. Light 15
/// (VERIFIED w/Lantern infobox: "light: Yes (15)" — brighter than the
/// torch's 14; both sitting and hanging forms share this sprite, the
/// model difference is disclosed as future work).
pub(super) fn lantern_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......II.......",
        ".......II.......",
        "......IIII......",
        ".....I....I.....",
        "....IIIIIIII....",
        "....I.YYYY.I....",
        "....I.YOOY.I....",
        "....I.YOOY.I....",
        "....I.YYYY.I....",
        "....I.YYYY.I....",
        "....IIIIIIII....",
        ".....I....I.....",
        ".....I....I.....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'I' => Some((IRON_D[0], IRON_D[1], IRON_D[2], 255)),
        'Y' => Some((LANT_GLASS[0], LANT_GLASS[1], LANT_GLASS[2], 255)),
        'O' => Some((LANT_FLAME[0], LANT_FLAME[1], LANT_FLAME[2], 255)),
        _ => None,
    });
}

/// iron nugget item icon: a small faceted gray lump with sheen pixels
/// (the 1.11-era item the lantern recipe needed — 8 nuggets + torch,
/// VERIFIED w/Lantern §Crafting).
pub(super) fn iron_nugget_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "......NNNN......",
        ".....NNsNNN.....",
        "....NNNNNNN.....",
        "....NNsNNNN.....",
        ".....NNNNN......",
        ".....NNsN.......",
        "......NN........",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'N' => Some((NUG[0], NUG[1], NUG[2], 255)),
        's' => Some((NUG_SHEEN[0], NUG_SHEEN[1], NUG_SHEEN[2], 255)),
        _ => None,
    });
}
