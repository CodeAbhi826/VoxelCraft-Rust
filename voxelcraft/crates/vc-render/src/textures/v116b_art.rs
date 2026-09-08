//! 1.16 bracket (Nether Update, part 2 — the crimson/warped families)
//! procedural tiles, the V14 window (tiles 673..=705): the crimson
//! stem/hyphae/planks/nylium quartet, the warped quartet, the fungi/
//! roots/sprouts/vines sprites, the warped wart block, shroomlight,
//! the polished basalt/blackstone/bricks stones, the soul torch +
//! soul lantern, the strider/piglin/hoglin spawn eggs, and the three
//! mob billboard sprites.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Covered by the `v114b_tiles_all_painted`
//! atlas guard (its loop runs 634..=TILE_MAX, so the new window rides
//! the same art-gap regression guard).

use super::{art, Rng};

// ---- shared palettes ----
/// crimson stem: deep maroon wood with blood-red streaks.
const CR_D: [i32; 3] = [74, 22, 26];
const CR_M: [i32; 3] = [104, 34, 38];
const CR_L: [i32; 3] = [142, 48, 50];
/// crimson nylium top: the scarlet fungal turf.
const CRNY_D: [i32; 3] = [96, 28, 30];
const CRNY_M: [i32; 3] = [140, 44, 42];
const CRNY_L: [i32; 3] = [188, 66, 58];
/// warped stem: teal wood with cyan streaks.
const WP_D: [i32; 3] = [18, 62, 66];
const WP_M: [i32; 3] = [28, 84, 90];
const WP_L: [i32; 3] = [40, 112, 118];
/// warped nylium top: the cyan-lichen turf.
const WPNY_D: [i32; 3] = [26, 74, 78];
const WPNY_M: [i32; 3] = [38, 104, 108];
const WPNY_L: [i32; 3] = [54, 138, 142];
/// crimson fungus: the red cap over a pale stalk.
const CAP_R: [i32; 3] = [158, 38, 40];
const CAP_RD: [i32; 3] = [120, 26, 28];
const STALK: [i32; 3] = [196, 186, 178];
/// warped fungus: the teal cap over a pale stalk.
const CAP_T: [i32; 3] = [42, 128, 132];
const CAP_TD: [i32; 3] = [28, 96, 100];
/// the netherrack base tones (nylium sides, dirt-specked hosts).
const RACK_D: [i32; 3] = [86, 30, 24];
const RACK_M: [i32; 3] = [112, 42, 32];
/// shroomlight: the glowing orange-pink lamp.
const SH_D: [i32; 3] = [158, 66, 40];
const SH_M: [i32; 3] = [198, 96, 52];
const SH_L: [i32; 3] = [240, 168, 96];
/// warped wart block: teal warty knobs.
const WW_D: [i32; 3] = [22, 84, 88];
const WW_M: [i32; 3] = [34, 116, 120];
const WW_L: [i32; 3] = [52, 150, 152];
/// polished basalt: the smoothed blue-gray stone.
const PB_D: [i32; 3] = [78, 80, 86];
const PB_M: [i32; 3] = [106, 108, 114];
const PB_L: [i32; 3] = [134, 136, 142];
/// polished blackstone: the near-black sheen.
const PBL_D: [i32; 3] = [30, 27, 32];
const PBL_M: [i32; 3] = [52, 48, 55];
const PBL_L: [i32; 3] = [76, 71, 80];
/// the soul flame: the blue fire (torch + lantern share it).
const SF_IN: [i32; 3] = [214, 240, 252];
const SF_M: [i32; 3] = [108, 168, 232];
const SF_OUT: [i32; 3] = [44, 96, 200];
/// iron gray (the soul lantern frame).
const IRON_M: [i32; 3] = [146, 150, 156];
const IRON_D: [i32; 3] = [96, 100, 106];
/// the strider: crimson quadruped on stilt legs.
const ST_BODY: [i32; 3] = [138, 44, 42];
const ST_BODY_D: [i32; 3] = [102, 30, 30];
const ST_HAIR: [i32; 3] = [186, 78, 62];
const ST_LEG: [i32; 3] = [88, 24, 24];
/// the piglin: pink brute with golden trappings.
const PG_SKIN: [i32; 3] = [232, 158, 138];
const PG_SKIN_D: [i32; 3] = [196, 118, 100];
const PG_EAR: [i32; 3] = [178, 92, 82];
const PG_GOLD: [i32; 3] = [238, 192, 66];
/// the hoglin: the brown tusked boar.
const HG_HIDE: [i32; 3] = [148, 96, 58];
const HG_HIDE_D: [i32; 3] = [112, 68, 40];
const HG_TUSK: [i32; 3] = [228, 220, 200];
const HG_MANE: [i32; 3] = [84, 52, 32];

// ---- crimson family ----

/// crimson stem side — the maroon trunk with vertical streaks.
pub(super) fn crimson_stem_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMDMMMMMDMMMMMD",
        "MDDMMMDMMMMMDMMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
        "MDDMMMDMDDMMMDMM",
        "MDMMMLLMDMMMLLMD",
        "MMMDMMMDMMMMDMMM",
        "MMDMMDDMMDMMDDMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((CR_D[0], CR_D[1], CR_D[2], 255)),
        'M' => Some((CR_M[0], CR_M[1], CR_M[2], 255)),
        'L' => Some((CR_L[0], CR_L[1], CR_L[2], 255)),
        _ => None,
    });
}

/// crimson stem top — the ringed end grain.
pub(super) fn crimson_stem_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDDDDDDDDDDDDDM",
        "MDMMMMMMMMMMMMDM",
        "MDMDDDDDDDDDDMDM",
        "MDMDMMMMMMMMDMDM",
        "MDMDMDDDDDDMDMDM",
        "MDMDMDMMMMDMDMDM",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMMMMDMDMDM",
        "MDMDMDDDDDDMDMDM",
        "MDMDMMMMMMMMDMDM",
        "MDMDDDDDDDDDDMDM",
        "MDMMMMMMMMMMMMDM",
        "MDDDDDDDDDDDDDDM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((CR_D[0], CR_D[1], CR_D[2], 255)),
        'M' => Some((CR_M[0], CR_M[1], CR_M[2], 255)),
        'L' => Some((CR_L[0], CR_L[1], CR_L[2], 255)),
        _ => None,
    });
}

/// crimson hyphae — the all-sides "bark" (the jungle-bark class).
pub(super) fn crimson_hyphae_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMDMMMMMDMMMMMD",
        "MDDMMMDMMMMMDMMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
        "MDDMMMDMDDMMMDMM",
        "MDMMMLLMDMMMLLMD",
        "MMMDMMMDMMMMDMMM",
        "MMDMMDDMMDMMDDMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((CR_D[0], CR_D[1], CR_D[2], 255)),
        'M' => Some((CR_M[0], CR_M[1], CR_M[2], 255)),
        'L' => Some((CR_L[0], CR_L[1], CR_L[2], 255)),
        _ => None,
    });
    // the hyphae tell: a faint pale speckle over the bark (the
    // all-sides bark's fungus residue)
    for (i, row) in rows.iter().enumerate() {
        for (j, c) in row.char_indices() {
            if c == 'L' && (i + j) % 5 == 0 {
                super::put(a, t, j as i32, i as i32, 176, 128, 126, 255);
            }
        }
    }
}

/// crimson planks — horizontal boards with nail dots.
pub(super) fn crimson_planks_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MMLMMMMMMMLMMMMM",
        "MMMLLMMMMMMLLMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMMLLMMMMMMML",
        "MLLMMMMMMMLLMMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMMMLMMMMMMML",
        "MMLLMMMMMMMLLMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMLLMMMMMMLMM",
        "MLMMMMMMLLMMMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((CR_D[0], CR_D[1], CR_D[2], 255)),
        'M' => Some((CR_M[0], CR_M[1], CR_M[2], 255)),
        'L' => Some((CR_L[0], CR_L[1], CR_L[2], 255)),
        _ => None,
    });
}

/// crimson nylium top — scarlet fungal turf on netherrack.
pub(super) fn crimson_nylium_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMLMMMMMLMMMMMM",
        "MLLLMMLLLMMMLLLM",
        "MLLLLLMMLLLLLMMM",
        "MMLLLMMMMLLLMMMM",
        "MMMLMMMMMMMLMMMM",
        "MLLLLLMMMMLLLLMM",
        "MLLLMMMMMLLLLLMM",
        "MMMMMLLMMMMMLLMM",
        "MMMMLLLMMMMLMMMM",
        "MLLMMMMMLLLLLMMM",
        "MLLLMMLLMLLLMMMM",
        "MMMMMLMMMMMLMMMM",
        "MLLMMMMLLMMMMMLM",
        "MLLLLLMMMMMLLLMM",
        "MMLLLMMMMLLLMMMM",
        "MMMMLMMMMMMLMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((CRNY_M[0], CRNY_M[1], CRNY_M[2], 255)),
        'L' => Some((CRNY_L[0], CRNY_L[1], CRNY_L[2], 255)),
        _ => None,
    });
    for _ in 0..14 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, CRNY_D[0], CRNY_D[1], CRNY_D[2], 255);
    }
}

/// crimson nylium side — netherrack under a red lip.
pub(super) fn crimson_nylium_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "LLLLLLLLLLLLLLLL",
        "MMMMMLMMMMMMMMMM",
        "MDDMMMMMDDMMMMDM",
        "MMDMMMMDMMMMMMDM",
        "DMMDMMMMMDMMMMDM",
        "MDMMMMDDMMMMDMMM",
        "MMMDMMMMMMMMMDDM",
        "MDMMMMDMMMMMDMMD",
        "MMDMMMMMDMMMMMMM",
        "DMMMMDMMMMMDMMMD",
        "MMDMMMMMMMDMMMDM",
        "MDMMMDMMMMDMMMMD",
        "MMMDMMMMMDMMMMDM",
        "MDMMMMDMMMMMDMMM",
        "MMDMMMMMMMDMMMMM",
        "DMMMMMDMMMMDMMMD",
    ];
    art(a, t, rows, &|c| match c {
        'L' => Some((CRNY_L[0], CRNY_L[1], CRNY_L[2], 255)),
        'M' => Some((RACK_M[0], RACK_M[1], RACK_M[2], 255)),
        'D' => Some((RACK_D[0], RACK_D[1], RACK_D[2], 255)),
        _ => None,
    });
}

/// crimson fungus — the red cap over a pale stalk (cross sprite).
pub(super) fn crimson_fungus_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        ".....RRRRRR.....",
        "....RRRRRRRR....",
        "...RRRWWRRRRR...",
        "..RRRWWWWRRRRR..",
        "..RRRRWWRRRRRR..",
        "..DRRRRRRRRRRD..",
        "...DDDDDDDDDD...",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....S.SS.S.....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((CAP_R[0], CAP_R[1], CAP_R[2], 255)),
        'D' => Some((CAP_RD[0], CAP_RD[1], CAP_RD[2], 255)),
        'W' => Some((STALK[0], STALK[1], STALK[2], 255)),
        'S' => Some((STALK[0], STALK[1], STALK[2], 255)),
        _ => None,
    });
}

/// crimson roots — the red tuft (cross sprite).
pub(super) fn crimson_roots_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".....R..R.......",
        "....R.RR.R..R...",
        "....R.R..R.R....",
        "..R.RR.R.RRR.R..",
        "...RR.R.RR.R.R..",
        "....R.RR.R.R....",
        "...RR.R.R.RR....",
        "..R.RR.RR.R.R...",
        "...R.R.R.RR.R...",
        "..R.RR.R.R.R....",
        "....R.RR.RR.....",
        ".....R.R.R......",
        ".....RR.RR......",
        "......RRR.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((CRNY_L[0], CRNY_L[1], CRNY_L[2], 255)),
        _ => None,
    });
}

/// weeping vines — hanging crimson strands (cross sprite).
pub(super) fn weeping_vines_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "RRRRRRRRRRRRRRRR",
        ".R.RRR.RR.R.RRR.",
        ".RR.R.RRR.RR.R..",
        "..RR.R.R.RR.RR..",
        "..R.RR.RR.R.R...",
        "...RR.R.R.RR.R..",
        "...R.RR.RR.R.R..",
        "..RR.R.R.RR.R...",
        "..R.RR.RR.R.R...",
        "...R.R.R.RR.R...",
        "...RR.RR.R.RR...",
        "....R.R.RR.R....",
        "....RR.R.R.R....",
        ".....R.RR.R.....",
        ".....RR.RR......",
        "......R.R.......",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((CAP_R[0], CAP_R[1], CAP_R[2], 255)),
        _ => None,
    });
}

// ---- warped family ----

/// warped stem side — the teal trunk with vertical streaks.
pub(super) fn warped_stem_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMDMMMMMDMMMMMD",
        "MDDMMMDMMMMMDMMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
        "MDDMMMDMDDMMMDMM",
        "MDMMMLLMDMMMLLMD",
        "MMMDMMMDMMMMDMMM",
        "MMDMMDDMMDMMDDMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((WP_D[0], WP_D[1], WP_D[2], 255)),
        'M' => Some((WP_M[0], WP_M[1], WP_M[2], 255)),
        'L' => Some((WP_L[0], WP_L[1], WP_L[2], 255)),
        _ => None,
    });
}

/// warped stem top — the ringed end grain.
pub(super) fn warped_stem_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDDDDDDDDDDDDDM",
        "MDMMMMMMMMMMMMDM",
        "MDMDDDDDDDDDDMDM",
        "MDMDMMMMMMMMDMDM",
        "MDMDMDDDDDDMDMDM",
        "MDMDMDMMMMDMDMDM",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMMMMDMDMDM",
        "MDMDMDDDDDDMDMDM",
        "MDMDMMMMMMMMDMDM",
        "MDMDDDDDDDDDDMDM",
        "MDMMMMMMMMMMMMDM",
        "MDDDDDDDDDDDDDDM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((WP_D[0], WP_D[1], WP_D[2], 255)),
        'M' => Some((WP_M[0], WP_M[1], WP_M[2], 255)),
        'L' => Some((WP_L[0], WP_L[1], WP_L[2], 255)),
        _ => None,
    });
}

/// warped hyphae — the all-sides "bark".
pub(super) fn warped_hyphae_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMDMMMMMDMMMMMD",
        "MDDMMMDMMMMMDMMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
        "MDDMMMDMDDMMMDMM",
        "MDMMMLLMDMMMLLMD",
        "MMMDMMMDMMMMDMMM",
        "MMDMMDDMMDMMDDMM",
        "MDMMLLMDMMLLMDMM",
        "MMMDMMMDMMMMDMMM",
        "DMMMDDMMMDDMMMDD",
        "MMDMMMDMMDMMMDMM",
        "LLMDMMLLLMDMMLLM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((WP_D[0], WP_D[1], WP_D[2], 255)),
        'M' => Some((WP_M[0], WP_M[1], WP_M[2], 255)),
        'L' => Some((WP_L[0], WP_L[1], WP_L[2], 255)),
        _ => None,
    });
}

/// warped planks — horizontal boards.
pub(super) fn warped_planks_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MMLMMMMMMMLMMMMM",
        "MMMLLMMMMMMLLMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMMLLMMMMMMML",
        "MLLMMMMMMMLLMMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMMMLMMMMMMML",
        "MMLLMMMMMMMLLMMM",
        "MMMMMMMMMMMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMLLMMMMMMLMM",
        "MLMMMMMMLLMMMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((WP_D[0], WP_D[1], WP_D[2], 255)),
        'M' => Some((WP_M[0], WP_M[1], WP_M[2], 255)),
        'L' => Some((WP_L[0], WP_L[1], WP_L[2], 255)),
        _ => None,
    });
}

/// warped nylium top — cyan-lichen turf on netherrack.
pub(super) fn warped_nylium_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMLMMMMMLMMMMMM",
        "MLLLMMLLLMMMLLLM",
        "MLLLLLMMLLLLLMMM",
        "MMLLLMMMMLLLMMMM",
        "MMMLMMMMMMMLMMMM",
        "MLLLLLMMMMLLLLMM",
        "MLLLMMMMMLLLLLMM",
        "MMMMMLLMMMMMLLMM",
        "MMMMLLLMMMMLMMMM",
        "MLLMMMMMLLLLLMMM",
        "MLLLMMLLMLLLMMMM",
        "MMMMMLMMMMMLMMMM",
        "MLLMMMMLLMMMMMLM",
        "MLLLLLMMMMMLLLMM",
        "MMLLLMMMMLLLMMMM",
        "MMMMLMMMMMMLMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((WPNY_M[0], WPNY_M[1], WPNY_M[2], 255)),
        'L' => Some((WPNY_L[0], WPNY_L[1], WPNY_L[2], 255)),
        _ => None,
    });
    for _ in 0..14 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, WPNY_D[0], WPNY_D[1], WPNY_D[2], 255);
    }
}

/// warped nylium side — netherrack under a teal lip.
pub(super) fn warped_nylium_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "LLLLLLLLLLLLLLLL",
        "MMMMMLMMMMMMMMMM",
        "MDDMMMMMDDMMMMDM",
        "MMDMMMMDMMMMMMDM",
        "DMMDMMMMMDMMMMDM",
        "MDMMMMDDMMMMDMMM",
        "MMMDMMMMMMMMMDDM",
        "MDMMMMDMMMMMDMMD",
        "MMDMMMMMDMMMMMMM",
        "DMMMMDMMMMMDMMMD",
        "MMDMMMMMMMDMMMDM",
        "MDMMMDMMMMDMMMMD",
        "MMMDMMMMMDMMMMDM",
        "MDMMMMDMMMMMDMMM",
        "MMDMMMMMMMDMMMMM",
        "DMMMMMDMMMMDMMMD",
    ];
    art(a, t, rows, &|c| match c {
        'L' => Some((WPNY_L[0], WPNY_L[1], WPNY_L[2], 255)),
        'M' => Some((RACK_M[0], RACK_M[1], RACK_M[2], 255)),
        'D' => Some((RACK_D[0], RACK_D[1], RACK_D[2], 255)),
        _ => None,
    });
}

/// warped fungus — the teal cap over a pale stalk (cross sprite).
pub(super) fn warped_fungus_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        ".....TTTTTT.....",
        "....TTTTTTTT....",
        "...TTTWWTTTTT...",
        "..TTTWWWWTTTTT..",
        "..TTTTWWTTTTTT..",
        "..DTTTTTTTTTTD..",
        "...DDDDDDDDDD...",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....SSSSSS.....",
        ".....S.SS.S.....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((CAP_T[0], CAP_T[1], CAP_T[2], 255)),
        'D' => Some((CAP_TD[0], CAP_TD[1], CAP_TD[2], 255)),
        'W' => Some((STALK[0], STALK[1], STALK[2], 255)),
        'S' => Some((STALK[0], STALK[1], STALK[2], 255)),
        _ => None,
    });
}

/// warped roots — the teal tuft (cross sprite).
pub(super) fn warped_roots_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".....T..T.......",
        "....T.TT.T..T...",
        "....T.T..T.T....",
        "..T.TT.T.TTT.T..",
        "...TT.T.TT.T.T..",
        "....T.TT.T.T....",
        "...TT.T.T.TT....",
        "..T.TT.TT.T.T...",
        "...T.T.T.TT.T...",
        "..T.TT.T.T.T....",
        "....T.TT.TT.....",
        ".....T.T.T......",
        ".....TT.TT......",
        "......TTT.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((WPNY_L[0], WPNY_L[1], WPNY_L[2], 255)),
        _ => None,
    });
}

/// twisting vines — climbing teal strands (cross sprite).
pub(super) fn twisting_vines_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "......T.T.......",
        ".....TT.TT......",
        "....TT.T.T......",
        "....T.TT.TT.....",
        ".....T.T.TT.....",
        "....TT.TT.T.....",
        "....T.T.T.T.....",
        ".....TT.TT.T....",
        "....T.T.T.TT....",
        "....TT.TT.T.....",
        ".....T.T.TT.....",
        "....TT.TT.T.....",
        "....T.T.T.T.....",
        ".....TT.TT......",
        "....T.T.TT......",
        "TTTTTTTTTTTTTTTT",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((CAP_T[0], CAP_T[1], CAP_T[2], 255)),
        _ => None,
    });
}

/// warped wart block — teal warty knobs.
pub(super) fn warped_wart_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMLLMMMMMMLLMM",
        "MLLLMMMMMLLLMMMM",
        "MLLLLLMMMMLLLMMM",
        "MMMMLLMMMMLLMMMM",
        "MMMMMMMMMMMMLMMM",
        "MLLLMMMMLLMMMMMM",
        "MLLLMMMMLLLMMMML",
        "MMMMLLMMMMMLLLMM",
        "MMMMLLMMMMMMLMMM",
        "MLLLMMMMLLLMMMMM",
        "MLLLMMMMLLLMMMML",
        "MMMMLLMMMMMMLLMM",
        "MMMMMMMMMMMMLMMM",
        "MLLLMMMMLLLMMMMM",
        "MLLLMMMMLLLMMMML",
        "MMMMMLLMMMMMMLMM",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((WW_M[0], WW_M[1], WW_M[2], 255)),
        'L' => Some((WW_L[0], WW_L[1], WW_L[2], 255)),
        _ => None,
    });
    for _ in 0..12 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, WW_D[0], WW_D[1], WW_D[2], 255);
    }
}

/// nether sprouts — the teal curl sprouts (cross sprite).
pub(super) fn nether_sprouts_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "..T....T....T...",
        "..TT..TT...TT...",
        "...T..T....T.T..",
        "..T.T.T..T.T.T..",
        "..TTT.TT.TT.TT..",
        "....T.T..T.T....",
        "..T..T.T.T...T..",
        "..TT.TT.TT..TT..",
        "...T.T..T....T..",
        "..T.T.T.T.T.T...",
        "..TT.TT.TT.TT...",
        "....T..T.T......",
        "...TT.TT.TT.....",
        "....T..T.T......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((WPNY_L[0], WPNY_L[1], WPNY_L[2], 255)),
        _ => None,
    });
}

// ---- shroomlight + polished stones ----

/// shroomlight — the glowing orange-pink fungus lamp (light 15).
pub(super) fn shroomlight_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MMLLMMMMMMLLMMMM",
        "MLLLLMMMMLLLMMMM",
        "MLLLLLMMMMLLLMMM",
        "MLLLLMMMMMLLMMMM",
        "MMLLMMMMMMMLMMMM",
        "MMMMMLLMMMMMMMMM",
        "MLLMMMLLMMMMLLMM",
        "MLLLMMLLLMMLLLMM",
        "MMLLMMMLLMMMLLMM",
        "MMMMMMMMLLMMMMMM",
        "MMLLMMMMMMMLLMMM",
        "MLLLMMMMLLLLLMMM",
        "MLLLMMMMLLLMMMMM",
        "MMLLMMMMMMLLMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((SH_D[0], SH_D[1], SH_D[2], 255)),
        'M' => Some((SH_M[0], SH_M[1], SH_M[2], 255)),
        'L' => Some((SH_L[0], SH_L[1], SH_L[2], 255)),
        _ => None,
    });
}

/// polished basalt side — the smooth chamfered column face.
pub(super) fn polished_basalt_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "DMMMMMMMMMMMMMMD",
        "DMLLMMMMMMMMLLMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMLLMMMMMMLLMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMLLMMMMMMMMLLMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMLLMMMMMMLLMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMLLMMMMMMMMLLMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((PB_D[0], PB_D[1], PB_D[2], 255)),
        'M' => Some((PB_M[0], PB_M[1], PB_M[2], 255)),
        'L' => Some((PB_L[0], PB_L[1], PB_L[2], 255)),
        _ => None,
    });
}

/// polished basalt top — the smooth ringed cap.
pub(super) fn polished_basalt_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "DMMMMMMMMMMMMMMD",
        "DMLLMMMMMMMMLLMD",
        "DMLMMMMMMMMMMLMD",
        "DMLMDDDDDDDDMLMD",
        "DMLMDDDDDDDDMLMD",
        "DMLMDDMMDDMMMLMD",
        "DMLMDDMLLDDMMLMD",
        "DMLMDDMLLDDMMLMD",
        "DMLMDDMMDDMMMLMD",
        "DMLMDDDDDDDDMLMD",
        "DMLMDDDDDDDDMLMD",
        "DMLMMMMMMMMMMLMD",
        "DMLLMMMMMMMMLLMD",
        "DMMMMMMMMMMMMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((PB_D[0], PB_D[1], PB_D[2], 255)),
        'M' => Some((PB_M[0], PB_M[1], PB_M[2], 255)),
        'L' => Some((PB_L[0], PB_L[1], PB_L[2], 255)),
        _ => None,
    });
}

/// polished blackstone — the near-black sheen.
pub(super) fn polished_blackstone_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MMLMMMMMMMMMLMMM",
        "MMMMMMMMLMMMMMMM",
        "MMMMLMMMMMMMMLMM",
        "MMMMMMMMMLMMMMMM",
        "MMLMMMMMMMMMLMMM",
        "MMMMMLMMMMMMMMMM",
        "MMMMMMMMMMMLMMMM",
        "MMLMMMMMLMMMMMMM",
        "MMMMMMMMMMMMLMMM",
        "MMMMMLMMMMMMMMMM",
        "MMMMLMMMMMMMLMMM",
        "MMMMMMMMMLMMMMMM",
        "MMLMMMMMMMMMLMMM",
        "MMMMMLMMMMMMMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((PBL_M[0], PBL_M[1], PBL_M[2], 255)),
        'L' => Some((PBL_L[0], PBL_L[1], PBL_L[2], 255)),
        _ => None,
    });
    for _ in 0..8 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, PBL_D[0], PBL_D[1], PBL_D[2], 255);
    }
}

/// polished blackstone bricks — the dark brick bond.
pub(super) fn polished_blackstone_bricks_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "DMMMMMMMDMMMMMMM",
        "DMMMMMMMDMMMMMMM",
        "DMMMMMMMDMMMMMMM",
        "MMMMMMMMDMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "MMMMMMMDMMMMMMMD",
        "DMMMMMMMDMMMMMMM",
        "DMMMMMMMDMMMMMMM",
        "DMMMMMMMDMMMMMMM",
        "MMMMMMMMDMMMMMMD",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((PBL_M[0], PBL_M[1], PBL_M[2], 255)),
        'D' => Some((PBL_D[0], PBL_D[1], PBL_D[2], 255)),
        _ => None,
    });
}

// ---- the soul lights ----

/// soul torch — the stick with a blue flame (cross sprite, light 10).
pub(super) fn soul_torch_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......FF.......",
        "......FFFF......",
        ".....FFIIFF.....",
        "......FFFF......",
        ".....FFFFFF.....",
        "......FFFF......",
        ".......SS.......",
        ".......SS.......",
        ".......SS.......",
        ".......SS.......",
        ".......SS.......",
        ".......SS.......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((SF_M[0], SF_M[1], SF_M[2], 255)),
        'I' => Some((SF_IN[0], SF_IN[1], SF_IN[2], 255)),
        'S' => Some((96, 72, 48, 255)),
        _ => None,
    });
}

/// soul lantern — the iron frame with a blue flame (cross sprite,
/// light 10; sitting + hanging share the sprite).
pub(super) fn soul_lantern_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......II.......",
        ".......II.......",
        "......I..I......",
        ".....I....I.....",
        "......I..I......",
        "......FFFF......",
        ".....FFIIFF.....",
        "......FFFF......",
        ".....FFFFFF.....",
        "......FFFF......",
        "......I..I......",
        ".....I....I.....",
        "......IIII......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((SF_M[0], SF_M[1], SF_M[2], 255)),
        'I' => Some((SF_IN[0], SF_IN[1], SF_IN[2], 255)),
        _ => None,
    });
}

// ---- the spawn eggs (the EGG_PALETTES convention) ----

/// strider egg — crimson/maroon shell (kind 42).
pub(super) fn strider_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
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
        'A' => Some((ST_BODY[0], ST_BODY[1], ST_BODY[2], 255)),
        'S' => Some((ST_HAIR[0], ST_HAIR[1], ST_HAIR[2], 255)),
        _ => None,
    });
}

/// piglin egg — pink/gold shell (kind 43).
pub(super) fn piglin_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......AA.......",
        "......AAAA......",
        ".....AAGAAA.....",
        ".....AGAAAG.....",
        "....AAAGAAAA....",
        "....AGAAAGAA....",
        "....AAAGAAAG....",
        "....AGAAAGAA....",
        "....AAGAAAAG....",
        ".....AGAAAG.....",
        ".....AAGAAA.....",
        "......AAAG......",
        ".......AA.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'A' => Some((PG_SKIN[0], PG_SKIN[1], PG_SKIN[2], 255)),
        'G' => Some((PG_GOLD[0], PG_GOLD[1], PG_GOLD[2], 255)),
        _ => None,
    });
}

/// hoglin egg — brown/tan shell (kind 44).
pub(super) fn hoglin_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".......AA.......",
        "......AAAA......",
        ".....AATAAA.....",
        ".....ATAAAT.....",
        "....AAATAAAA....",
        "....ATAAATAA....",
        "....AAATAAAT....",
        "....ATAAATAA....",
        "....AATAAAAT....",
        ".....ATAAAT.....",
        ".....AATAAA.....",
        "......AAAT......",
        ".......AA.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'A' => Some((HG_HIDE[0], HG_HIDE[1], HG_HIDE[2], 255)),
        'T' => Some((HG_HIDE_D[0], HG_HIDE_D[1], HG_HIDE_D[2], 255)),
        _ => None,
    });
}

// ---- the mob billboards ----

/// strider — the crimson quadruped on stilt legs (the "striding"
/// silhouette; the wiki sprite's two tall front legs + hair tuft).
pub(super) fn strider_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "....H...........",
        "...HHH..........",
        "...HHHH..H......",
        "..BBBBB.HHH.....",
        "..BDBDBBBHH.....",
        "..BBBBBBBBB.....",
        "..BDBDBDBDB.....",
        "..BBBBBBBBB.....",
        "...B.B...B.B....",
        "...B.B...B.B....",
        "...B.B...B.B....",
        "...B.B...B.B....",
        "...B.B...B.B....",
        "...B.B...B.B....",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((ST_BODY[0], ST_BODY[1], ST_BODY[2], 255)),
        'D' => Some((ST_BODY_D[0], ST_BODY_D[1], ST_BODY_D[2], 255)),
        'H' => Some((ST_HAIR[0], ST_HAIR[1], ST_HAIR[2], 255)),
        _ => None,
    });
    // the stilt legs read darker than the body (the silhouette's tell)
    for y in 9..15usize {
        for x in [3usize, 5, 9, 11] {
            super::put(a, t, x as i32, y as i32, ST_LEG[0], ST_LEG[1], ST_LEG[2], 255);
        }
    }
}

/// piglin — the pink brute with golden trappings (the tall-eared
/// humanoid silhouette).
pub(super) fn piglin_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "..E.E...........",
        "..EEE...........",
        "..SSSS..........",
        ".SSSSSS.........",
        ".SDSSDS.........",
        ".SSSSSS.........",
        "..SSSS..........",
        ".GGGGGG.........",
        "GSGGGGSG........",
        "GSGGGGSG........",
        "..SSSS..........",
        "..SS.SS.........",
        "..SS.SS.........",
        "..SS.SS.........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((PG_SKIN[0], PG_SKIN[1], PG_SKIN[2], 255)),
        'D' => Some((PG_SKIN_D[0], PG_SKIN_D[1], PG_SKIN_D[2], 255)),
        'E' => Some((PG_EAR[0], PG_EAR[1], PG_EAR[2], 255)),
        'G' => Some((PG_GOLD[0], PG_GOLD[1], PG_GOLD[2], 255)),
        _ => None,
    });
}

/// hoglin — the brown tusked boar (the wide, low silhouette with
/// the mane ridge + forward tusks).
pub(super) fn hoglin_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "..MMMM..........",
        ".MMTTMMMM.......",
        "MMMTTTMMMMM.....",
        "MMBBTBBBBBBB....",
        "MBBBBBBBBBBBD...",
        "BBBBTBBBBBBBT...",
        "MBBBBBBBBBBBD...",
        "MMBBBBBBBBBD....",
        ".MMBBBBBBBD.....",
        "..BBBBBBBD......",
        "..B.B..B.B......",
        "..B.B..B.B......",
        "..L.L..L.L......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((HG_HIDE[0], HG_HIDE[1], HG_HIDE[2], 255)),
        'D' => Some((HG_HIDE_D[0], HG_HIDE_D[1], HG_HIDE_D[2], 255)),
        'M' => Some((HG_MANE[0], HG_MANE[1], HG_MANE[2], 255)),
        'T' => Some((HG_TUSK[0], HG_TUSK[1], HG_TUSK[2], 255)),
        'L' => Some((HG_MANE[0], HG_MANE[1], HG_MANE[2], 255)),
        _ => None,
    });
}
