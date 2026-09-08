//! The 1.0-1.16.5 completeness-audit tiles, the V15 window (tiles
//! 706..=734): the cooked-meat family (steak, cooked porkchop/chicken/
//! mutton/cod/salmon), the kitchen chain (apple, bowl, mushroom + rabbit
//! stew, beetroot, beetroot soup, sugar, egg, poisonous potato), popped
//! chorus fruit, the ghast tear, the ghast/cave-spider/silverfish spawn
//! eggs + the three mob billboard sprites. (The six leaping/regeneration
//! potion rows ride the shared `potion_art` helper in the dispatcher.)
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Covered by the `v114b_tiles_all_painted`
//! atlas guard (its loop runs 634..=TILE_MAX, so the new window rides
//! the same art-gap regression guard).

use super::{art, Rng};

// ---- shared palettes ----
/// cooked-meat sear browns (the outer crust).
const SEAR_D: [i32; 3] = [104, 54, 30];
const SEAR_M: [i32; 3] = [142, 80, 46];
const SEAR_L: [i32; 3] = [186, 120, 72];
/// cooked porkchop: the rosy interior.
const PORK_IN: [i32; 3] = [206, 130, 108];
const PORK_IN_D: [i32; 3] = [176, 104, 86];
/// cooked chicken: golden-brown skin.
const CHK_SKIN: [i32; 3] = [198, 138, 62];
const CHK_SKIN_D: [i32; 3] = [160, 106, 44];
/// cooked mutton: the darker mutton chop.
const MUT_M: [i32; 3] = [160, 86, 62];
const MUT_D: [i32; 3] = [128, 66, 48];
/// cooked cod: the white flake fillet.
const COD_F: [i32; 3] = [236, 228, 212];
const COD_F_D: [i32; 3] = [204, 196, 182];
/// cooked salmon: the orange flake fillet.
const SAL_F: [i32; 3] = [238, 158, 96];
const SAL_F_D: [i32; 3] = [204, 126, 70];
/// bone stubs (the chop/drumstick tell).
const BONE: [i32; 3] = [238, 234, 222];
/// apple: the red skin + leaf-green + stem.
const APL_R: [i32; 3] = [196, 32, 32];
const APL_RD: [i32; 3] = [156, 22, 26];
const APL_HI: [i32; 3] = [236, 92, 82];
const APL_STEM: [i32; 3] = [92, 62, 38];
/// wooden bowl tones.
const BOWL_M: [i32; 3] = [156, 112, 62];
const BOWL_D: [i32; 3] = [118, 82, 44];
const BOWL_IN: [i32; 3] = [186, 140, 86];
/// stew surfaces.
const STEW_BROWN: [i32; 3] = [148, 92, 52];
const STEW_CHUNK: [i32; 3] = [196, 148, 92];
const STEW_RED: [i32; 3] = [170, 60, 44];
const STEW_ORANGE: [i32; 3] = [206, 120, 58];
/// beetroot + soup purples.
const BET_M: [i32; 3] = [128, 28, 78];
const BET_D: [i32; 3] = [96, 18, 60];
const BET_HI: [i32; 3] = [176, 52, 110];
const BET_STEM: [i32; 3] = [110, 140, 62];
/// sugar: the white pile.
const SUGAR: [i32; 3] = [240, 240, 246];
/// egg: the speckled shell.
const EGG_S: [i32; 3] = [232, 226, 210];
const EGG_SD: [i32; 3] = [200, 194, 178];
/// poisonous potato: the green-spotted tuber.
const POT_M: [i32; 3] = [196, 152, 88];
const POT_ROT: [i32; 3] = [116, 158, 62];
/// popped chorus: the pale-gold popped kernel.
const POP_M: [i32; 3] = [226, 206, 186];
const POP_D: [i32; 3] = [192, 168, 148];
/// ghast tear: the liquid-white tear.
const TEAR: [i32; 3] = [226, 238, 240];
const TEAR_D: [i32; 3] = [182, 200, 208];

// ---- the cooked-meat family ----

/// steak — the dark seared slab (hunger 8; w/Food).
pub(super) fn steak_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "...MMMMMMMMM....",
        "..MDDLLLLLLDM..",
        "..MDLLDDLLLLDM.",
        "..MLLLDDLLLDMM.",
        "..MLDLLDDLLLLM.",
        "..MDDLLLLLLDDM.",
        "..MDDDDDDDDDDM.",
        "...MMMMMMMMMM..",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((SEAR_D[0], SEAR_D[1], SEAR_D[2], 255)),
        'M' => Some((SEAR_M[0], SEAR_M[1], SEAR_M[2], 255)),
        'L' => Some((SEAR_L[0], SEAR_L[1], SEAR_L[2], 255)),
        _ => None,
    });
}

/// cooked porkchop — the rosy chop with a bone stub (hunger 8).
pub(super) fn cooked_porkchop_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "...PPPPPP.......",
        "..PPDDDDPP......",
        "..PDDPPDDPP.....",
        "..PDPPPPDDP.....",
        "..PDDPPPPDP.....",
        "..PPDDPPDDP.....",
        "...PPPPDDPPB....",
        "...PPPPPPPB.....",
        "....PPPPPPB.....",
        "....PPPPPB......",
        "......PPB.......",
        ".......B........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((PORK_IN[0], PORK_IN[1], PORK_IN[2], 255)),
        'D' => Some((PORK_IN_D[0], PORK_IN_D[1], PORK_IN_D[2], 255)),
        'B' => Some((BONE[0], BONE[1], BONE[2], 255)),
        _ => None,
    });
}

/// cooked chicken — the golden drumstick (hunger 6).
pub(super) fn cooked_chicken_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "....SSSS........",
        "...SSDDSS.......",
        "..SDDSSDSS......",
        "..SDSSSSDS......",
        "..SSDDDDSS......",
        "...SSSSSS.......",
        "...SSSSSSB......",
        "...SSSSSSB......",
        "....SSSSB.......",
        "....SSSSB.......",
        "......SS........",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((CHK_SKIN[0], CHK_SKIN[1], CHK_SKIN[2], 255)),
        'D' => Some((CHK_SKIN_D[0], CHK_SKIN_D[1], CHK_SKIN_D[2], 255)),
        'B' => Some((BONE[0], BONE[1], BONE[2], 255)),
        _ => None,
    });
}

/// cooked mutton — the darker chop (hunger 6).
pub(super) fn cooked_mutton_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "....MMMMM.......",
        "...MMDDDMM......",
        "..MMDDDDMM......",
        "..MDDMMDDMB.....",
        "..MDMMMDDMB.....",
        "..MDDMMDDMB.....",
        "..MMDDDDMB......",
        "...MMMMMMB......",
        "....MMMMB.......",
        "......MMB.......",
        ".......MB.......",
        "........B.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((MUT_M[0], MUT_M[1], MUT_M[2], 255)),
        'D' => Some((MUT_D[0], MUT_D[1], MUT_D[2], 255)),
        'B' => Some((BONE[0], BONE[1], BONE[2], 255)),
        _ => None,
    });
}

/// cooked cod — the white flake fillet (hunger 5).
pub(super) fn cooked_cod_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "...FFFFFFFFFF...",
        "..FFDDFFFFDDF...",
        "..FDFFFFFDFFF...",
        "..FFFFFFFFDDF...",
        "..FFDDFFFFFFF...",
        "..FFFFDDFFFFF...",
        "..FFFFFFFFFDF...",
        "...FFFFFFFFFF...",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((COD_F[0], COD_F[1], COD_F[2], 255)),
        'D' => Some((COD_F_D[0], COD_F_D[1], COD_F_D[2], 255)),
        _ => None,
    });
}

/// cooked salmon — the orange flake fillet (hunger 6).
pub(super) fn cooked_salmon_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "...FFFFFFFFFF...",
        "..FFDDFFFFDDF...",
        "..FDFFFFFDFFF...",
        "..FFFFFFFFDDF...",
        "..FFDDFFFFFFF...",
        "..FFFFDDFFFFF...",
        "..FFFFFFFFFDF...",
        "...FFFFFFFFFF...",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((SAL_F[0], SAL_F[1], SAL_F[2], 255)),
        'D' => Some((SAL_F_D[0], SAL_F_D[1], SAL_F_D[2], 255)),
        _ => None,
    });
}

// ---- the kitchen chain ----

/// apple — the red fruit with a stem (hunger 4; leaf-drop 0.5%).
pub(super) fn apple_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......S........",
        ".......S........",
        "....RRRSRR......",
        "...RRRRRRRR.....",
        "..RRHRRRRRRR....",
        "..RHRRRRRRDR....",
        ".RRRRRRRRRDDR...",
        ".RRRRRRRRRDDR...",
        ".RRRRRRRRRDR....",
        "..RRRRRRRDR.....",
        "..RRRRRRDDR.....",
        "...RRRRDDR......",
        ".....RRR........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((APL_R[0], APL_R[1], APL_R[2], 255)),
        'D' => Some((APL_RD[0], APL_RD[1], APL_RD[2], 255)),
        'H' => Some((APL_HI[0], APL_HI[1], APL_HI[2], 255)),
        'S' => Some((APL_STEM[0], APL_STEM[1], APL_STEM[2], 255)),
        _ => None,
    });
}

/// bowl — the empty wooden bowl (3 planks -> 4; the fishing junk row).
pub(super) fn bowl_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "...W.........W..",
        "..WWDDDDDDDWW...",
        "..WDIIIIIIDW....",
        "..WDIIIIIIIW....",
        "...WDIIIIIW.....",
        "...WDDDDDDW.....",
        "....WWWWW.......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((BOWL_M[0], BOWL_M[1], BOWL_M[2], 255)),
        'D' => Some((BOWL_D[0], BOWL_D[1], BOWL_D[2], 255)),
        'I' => Some((BOWL_IN[0], BOWL_IN[1], BOWL_IN[2], 255)),
        _ => None,
    });
}

/// mushroom stew — the red/brown caps on the stew (hunger 6).
pub(super) fn mushroom_stew_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "....RRR.........",
        "...RDRRRR..BBB..",
        "...RRRDR..BDBB..",
        "....RRR...BBB...",
        "...W.........W..",
        "..WWDDDDDDDWW...",
        "..WDSBSDSDSDW...",
        "..WDBSDSBSDBW...",
        "...WDDDDDDDW....",
        "...WDDDDDDWW....",
        "....WWWWWW......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((BOWL_M[0], BOWL_M[1], BOWL_M[2], 255)),
        'D' => Some((BOWL_D[0], BOWL_D[1], BOWL_D[2], 255)),
        'S' => Some((STEW_BROWN[0], STEW_BROWN[1], STEW_BROWN[2], 255)),
        'B' => Some((STEW_CHUNK[0], STEW_CHUNK[1], STEW_CHUNK[2], 255)),
        'R' => Some((STEW_RED[0], STEW_RED[1], STEW_RED[2], 255)),
        _ => None,
    });
}

/// rabbit stew — the chunky orange stew (hunger 10 — the top food).
pub(super) fn rabbit_stew_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "...CC..CC..C....",
        "..CCCCCCCCC.....",
        "..COCOOOCCC.....",
        "...OOOOOOO......",
        "...W.........W..",
        "..WWDDDDDDDWW...",
        "..WDOOCCOOODW...",
        "..WDOCOOCCODW...",
        "...WDDDDDDDW....",
        "...WDDDDDDWW....",
        "....WWWWWW......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((BOWL_M[0], BOWL_M[1], BOWL_M[2], 255)),
        'D' => Some((BOWL_D[0], BOWL_D[1], BOWL_D[2], 255)),
        'O' => Some((STEW_ORANGE[0], STEW_ORANGE[1], STEW_ORANGE[2], 255)),
        'C' => Some((STEW_CHUNK[0], STEW_CHUNK[1], STEW_CHUNK[2], 255)),
        _ => None,
    });
}

/// beetroot — the purple root (hunger 1; the soup ingredient).
pub(super) fn beetroot_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".....SS.........",
        "....SSS.........",
        "....S...........",
        "...BBBBB........",
        "..BBHBBDB.......",
        ".BBBBBBBDB......",
        ".BBBBBBBDB......",
        ".BBBBBBDDDB.....",
        ".BDBBBBDBB......",
        "..BDBBDBB.......",
        "...BBBBB........",
        "....BBB.........",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((BET_M[0], BET_M[1], BET_M[2], 255)),
        'D' => Some((BET_D[0], BET_D[1], BET_D[2], 255)),
        'H' => Some((BET_HI[0], BET_HI[1], BET_HI[2], 255)),
        'S' => Some((BET_STEM[0], BET_STEM[1], BET_STEM[2], 255)),
        _ => None,
    });
}

/// beetroot soup — the purple soup (6 beetroots + bowl; hunger 6).
pub(super) fn beetroot_soup_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "...BB.BBB..B....",
        "..BBBBBBBBB.....",
        "..BDBBBBDBB.....",
        "...BBBBBBB......",
        "...W.........W..",
        "..WWDDDDDDDWW...",
        "..WDBBBBBBBDW...",
        "..WDBBDBBDBDW...",
        "...WDDDDDDDW....",
        "...WDDDDDDWW....",
        "....WWWWWW......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((BOWL_M[0], BOWL_M[1], BOWL_M[2], 255)),
        'D' => Some((BOWL_D[0], BOWL_D[1], BOWL_D[2], 255)),
        'B' => Some((BET_M[0], BET_M[1], BET_M[2], 255)),
        _ => None,
    });
}

/// sugar — the white pile (the honey-bottle craft).
pub(super) fn sugar_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "......WW........",
        "....WWWWWW......",
        "...WWWWWWWW.....",
        "..WWWWWWWWWW....",
        "..WWWWWWWWWW....",
        "...WWWWWWWW.....",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((SUGAR[0], SUGAR[1], SUGAR[2], 255)),
        _ => None,
    });
}

/// egg — the speckled shell (the 9000-tick chicken lay).
pub(super) fn egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "......EE........",
        ".....EEEE.......",
        ".....EESE.......",
        "....EEESSE......",
        "....ESESSE......",
        "....EESSESE.....",
        "....EESESSE.....",
        "....EEESSEE.....",
        ".....EESEE......",
        ".....ESEE.......",
        "......EE........",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'E' => Some((EGG_S[0], EGG_S[1], EGG_S[2], 255)),
        'S' => Some((EGG_SD[0], EGG_SD[1], EGG_SD[2], 255)),
        _ => None,
    });
}

/// poisonous potato — the green-rotted tuber (60% Poison 5 s).
pub(super) fn poisonous_potato_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "....PPPPPP......",
        "...PPGGPPPP.....",
        "..PPPPPPPGP.....",
        "..PGPPPPPPPP....",
        "..PPPPPGPPPP....",
        "..PPPPPPPPGP....",
        "..PGPPPPPPPP....",
        "...PPPPGPPP.....",
        "....PPPPPP......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((POT_M[0], POT_M[1], POT_M[2], 255)),
        'G' => Some((POT_ROT[0], POT_ROT[1], POT_ROT[2], 255)),
        _ => None,
    });
}

/// popped chorus fruit — the pale popped kernel (the purpur input).
pub(super) fn popped_chorus_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        ".....MMMMM......",
        "....MMDDMMM.....",
        "...MMDDMMDMM....",
        "...MDMMMMMMM....",
        "...MMDMMMMDM....",
        "...MMMMMDMMM....",
        "...MMDMMMMMM....",
        "....MMMDDMM.....",
        ".....MMMMM......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((POP_M[0], POP_M[1], POP_M[2], 255)),
        'D' => Some((POP_D[0], POP_D[1], POP_D[2], 255)),
        _ => None,
    });
}

/// ghast tear — the liquid-white tear ("the only source" — the ghast).
pub(super) fn ghast_tear_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "........T.......",
        ".......TT.......",
        "......TTT.......",
        "......TTT.......",
        ".....TTTTT......",
        ".....TTTTT......",
        ".....TTTTTT.....",
        ".....TTTTTT.....",
        ".....TTTTTD.....",
        "......TTTTD.....",
        "......TTDD......",
        ".......TD.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((TEAR[0], TEAR[1], TEAR[2], 255)),
        'D' => Some((TEAR_D[0], TEAR_D[1], TEAR_D[2], 255)),
        _ => None,
    });
}

// ---- the spawn eggs (the EGG_PALETTES convention) ----

/// ghast egg — the white shell with dark-gray mottling (kind 45).
pub(super) fn ghast_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......AA.......",
        "......AAAA......",
        ".....AASAAA.....",
        ".....ASAAAS.....",
        "....AAASAAAA....",
        "....ASAAASAA....",
        "....AAASAAAS....",
        "....ASAAASAA....",
        "....AASAAAAS....",
        ".....ASAAAS.....",
        ".....AASAAA.....",
        "......AAAS......",
        ".......AA.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'A' => Some((232, 232, 236, 255)),
        'S' => Some((64, 64, 72, 255)),
        _ => None,
    });
}

/// cave-spider egg — the blue-gray shell with red eyespots (kind 46).
pub(super) fn cave_spider_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......AA.......",
        "......AAAA......",
        ".....AASAAA.....",
        ".....ASAAAS.....",
        "....AAASAAAA....",
        "....ASAAASAA....",
        "....AAASAAAS....",
        "....ASAAASAA....",
        "....AASAAAAS....",
        ".....ASAAAS.....",
        ".....AASAAA.....",
        "......AAAS......",
        ".......AA.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'A' => Some((26, 44, 76, 255)),
        'S' => Some((170, 40, 40, 255)),
        _ => None,
    });
}

/// silverfish egg — the stone-gray shell with dark scales (kind 47).
pub(super) fn silverfish_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......AA.......",
        "......AAAA......",
        ".....AASAAA.....",
        ".....ASAAAS.....",
        "....AAASAAAA....",
        "....ASAAASAA....",
        "....AAASAAAS....",
        "....ASAAASAA....",
        "....AASAAAAS....",
        ".....ASAAAS.....",
        ".....AASAAA.....",
        "......AAAS......",
        ".......AA.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'A' => Some((148, 148, 158, 255)),
        'S' => Some((70, 70, 84, 255)),
        _ => None,
    });
}

// ---- the mob billboards ----

/// ghast — the huge white floating cube with closed eyes and the
/// nine-stand-in tentacles (the 4x4 hitbox reads as the big sprite).
pub(super) fn ghast_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "...GGGGGGGGGG...",
        "..GGGGGGGGGGGG..",
        "..GGDDGGGGDDGG..",
        "..GGDDGGGGDDGG..",
        "..GGGGGGGGGGGG..",
        "..GGGGDDDDGGGG..",
        "..GGGGDDDDGGGG..",
        "..GGGGGGGGGGGG..",
        "..GGGGGGGGGGGG..",
        "...GGGGGGGGGG...",
        "...TT.TT.TT.T...",
        "...TT.TT.TT.T...",
        "..TTT.TT.TT.TT..",
        "..TT..TT..TT....",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((232, 232, 236, 255)),
        'D' => Some((48, 48, 56, 255)),
        'T' => Some((214, 214, 220, 255)),
        _ => None,
    });
}

/// cave spider — the small blue-gray spider with the red eyes (the
/// 0.5x0.7 hitbox reads as the compact sprite).
pub(super) fn cave_spider_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "..L....RR....L..",
        "...L..RBBR..L...",
        "....L.RBBR.L....",
        "...LLLBBBBLLL...",
        "..LL.BBBBBB.LL..",
        ".LL..BDDBD..LL..",
        ".LL..BBBBBB..LL.",
        "..LL.BBBBBB.LL..",
        "...LLLBBBBLLL...",
        "....L.BBBB.L....",
        "...L..L..L..L...",
        "..L...L..L...L..",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((32, 50, 84, 255)),
        'D' => Some((20, 32, 58, 255)),
        'R' => Some((178, 42, 42, 255)),
        'L' => Some((24, 38, 66, 255)),
        _ => None,
    });
}

/// silverfish — the small stone-gray bristletail (the 0.3x0.4 hitbox
/// reads as the tiny sprite).
pub(super) fn silverfish_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "..A..BBBB..A....",
        "...A.BDBB.A.....",
        "...ABBBBBA......",
        "....BDBBBB......",
        "....BBBBBB..C...",
        "....BDBB..CC....",
        ".....BBB.C......",
        ".....BB.C.......",
        "......CC........",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((158, 158, 168, 255)),
        'D' => Some((108, 108, 120, 255)),
        'A' => Some((120, 120, 132, 255)),
        'C' => Some((92, 92, 104, 255)),
        _ => None,
    });

}

/// melon slice — the sweep-2 food row (w/Melon_Slice: "Restores 2
/// hunger"; the 1.0 staple). A wedge viewed flesh-on: green rind arc
/// along the hypotenuse, red flesh, pale seeds.
pub(super) fn melon_slice_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".....GDDGG......",
        "....GRRRFFG.....",
        "....RRRFFFG.....",
        "...RRFSRRFFG....",
        "...RRSRRFFFG....",
        "...RRRFFSRFG....",
        "..GRRSFFFRFG....",
        "..GRRRFFSFG.....",
        "..GGRFFFRFG.....",
        "...GGRRRFG......",
        ".....GGGG.......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((46, 120, 40, 255)),   // rind green
        'D' => Some((30, 90, 24, 255)),    // rind dark
        'R' => Some((206, 46, 46, 255)),   // flesh red
        'F' => Some((178, 34, 34, 255)),   // flesh shade
        'S' => Some((240, 235, 200, 255)), // pale seed
        _ => None,
    });
}
