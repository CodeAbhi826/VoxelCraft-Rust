//! 1.15 bracket (Buzzy Bees) procedural tiles — the bee window
//! (tiles 642..=654): the bee-nest and beehive faces (with the
//! honey_level-5 oozing variants), the honey + honeycomb blocks, the
//! honeycomb / honey-bottle / shears / bee-egg item sprites, and the
//! bee mob billboard.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Covered by the `v114b_tiles_all_painted`
//! atlas guard (its loop runs 634..=TILE_MAX, so the new window rides
//! the same art-gap regression guard).

use super::{art, Rng};

// ---- shared palettes ----
/// straw tones (the nest's woven hay body).
const STRAW_D: [i32; 3] = [146, 104, 44];
const STRAW_M: [i32; 3] = [186, 140, 62];
const STRAW_L: [i32; 3] = [214, 168, 84];
/// plank tones (the beehive body).
const PLANK_D: [i32; 3] = [120, 88, 50];
const PLANK_M: [i32; 3] = [160, 122, 74];
const PLANK_L: [i32; 3] = [196, 158, 104];
/// the entrance hole (dark).
const HOLE: [i32; 3] = [34, 24, 12];
/// amber honey tones (the ooze + the gel + the bottle fill).
const HONEY_D: [i32; 3] = [190, 122, 16];
const HONEY_M: [i32; 3] = [232, 164, 32];
const HONEY_L: [i32; 3] = [252, 204, 78];
/// iron tones (the shears blades).
const IRON_M: [i32; 3] = [190, 196, 202];
const IRON_L: [i32; 3] = [230, 234, 238];
const IRON_D: [i32; 3] = [130, 136, 142];
/// bee body tones.
const BEE_AMBER: [i32; 3] = [238, 178, 42];
const BEE_DARK: [i32; 3] = [42, 32, 20];
const BEE_WING: [i32; 3] = [226, 236, 244];

/// bee-nest TOP — the woven-straw crown: concentric ragged rings.
pub(super) fn bee_nest_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "DMMMLLMMMMMLLMMMD",
        "DMMLDMMMLLMMDMMMD",
        "DMLDMMMMMMMMLDMMD",
        "DMMDMMLLLLMMDMMMD",
        "DMMDMLDDDMLMDMMMD",
        "DMMDMDMMMMDMDMMMD",
        "DMMDMDMMMMDMDMMMD",
        "DMMDMDMMMMDMDMMMD",
        "DMMDMDMMMMDMDMMMD",
        "DMMDMLDDDMLMDMMMD",
        "DMMDMMLLLLMMDMMMD",
        "DMLDMMMMMMMMLDMMD",
        "DMMLDMMMLLMMDMMMD",
        "DMMMLLMMMMMLLMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((STRAW_D[0], STRAW_D[1], STRAW_D[2], 255)),
        'M' => Some((STRAW_M[0], STRAW_M[1], STRAW_M[2], 255)),
        'L' => Some((STRAW_L[0], STRAW_L[1], STRAW_L[2], 255)),
        _ => None,
    });
    // straw-fiber flecks
    for _ in 0..14 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, STRAW_L[0], STRAW_L[1], STRAW_L[2], 255);
    }
}

/// bee-nest FRONT — the straw wall with the dark entrance hole (all
/// four sides show it — the furnace pattern, no facing states).
pub(super) fn bee_nest_front_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "MMLLMMDMMDMMLLMM",
        "MLDMMMMMMMMLDMMM",
        "MMDMMLLLLMMDMMMD",
        "MMMMMMMMMMMMMMMM",
        "MMMMMDDDDDDMMMMM",
        "MMMMDDDDDDDDMMMM",
        "MMMDHHHHHHHHDMMM",
        "MMMDHHHHHHHHDMMM",
        "MMMDHHHHHHHHDMMM",
        "MMMMDDDDDDDDMMMM",
        "MMMMMDDDDDDMMMMM",
        "MMLLMMDMMDMMLLMM",
        "MLDMMMMMMMMLDMMM",
        "MMDMMLLLLMMDMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((STRAW_D[0], STRAW_D[1], STRAW_D[2], 255)),
        'M' => Some((STRAW_M[0], STRAW_M[1], STRAW_M[2], 255)),
        'L' => Some((STRAW_L[0], STRAW_L[1], STRAW_L[2], 255)),
        'H' => Some((HOLE[0], HOLE[1], HOLE[2], 255)),
        _ => None,
    });
    for _ in 0..10 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, STRAW_L[0], STRAW_L[1], STRAW_L[2], 255);
    }
}

/// bee-nest FRONT at honey_level 5 — honey oozing out of the hole
/// (VERIFIED w/Bee_nest: "it changes its appearance to show honey
/// oozing out").
pub(super) fn bee_nest_front_honey_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "MMLLMMDMMDMMLLMM",
        "MLDMMMMMMMMLDMMM",
        "MMDMMLLLLMMDMMMD",
        "MMMMMMDDDDMMMMMM",
        "MMMMDDLHHLLDDMMM",
        "MMMDLHHHHHHHLDMM",
        "MMDLHHHHHHHHHHDMM",
        "MMDLHHHHHHHHHDMM",
        "MMMDLHHHHHHHHDMM",
        "MMMMDLHHHHHHDMMM",
        "MMMMMDLHHHHDMMMM",
        "MMMMMMDLHHDMMMMM",
        "MMMMMMMDLDMMMMMM",
        "MMDMMLLLLMMDMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((STRAW_D[0], STRAW_D[1], STRAW_D[2], 255)),
        'M' => Some((STRAW_M[0], STRAW_M[1], STRAW_M[2], 255)),
        'L' => Some((HONEY_L[0], HONEY_L[1], HONEY_L[2], 255)),
        'H' => Some((HONEY_M[0], HONEY_M[1], HONEY_M[2], 255)),
        _ => None,
    });
}

/// beehive TOP — the planked crown.
pub(super) fn beehive_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows: [&str; 16] = [
        "DDDDDDDDDDDDDDDD",
        "DMMMMMMMMMMMMMMD",
        "DMDDDDDDDDDDDDMD",
        "DMDMMMMMMMMMMDMD",
        "DMDMDDDDDDDDMDMD",
        "DMDMDMMMMMMDMDMD",
        "DMDMDMDDDDMDMDMD",
        "DMDMDMDMMDMDMDMD",
        "DMDMDMDDDDMDMDMD",
        "DMDMDMMMMMMDMDMD",
        "DMDMDDDDDDDDMDMD",
        "DMDMMMMMMMMMMDMD",
        "DMDDDDDDDDDDDDMD",
        "DMMMMMMMMMMMMMMD",
        "DDDDDDDDDDDDDDDD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((PLANK_D[0], PLANK_D[1], PLANK_D[2], 255)),
        'M' => Some((PLANK_M[0], PLANK_M[1], PLANK_M[2], 255)),
        _ => None,
    });
}

/// beehive FRONT — the plank wall with the entrance slot.
pub(super) fn beehive_front_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
        "MMMMMMDDDDMMMMMM",
        "MMMMMDHHHHDMMMMM",
        "MMMMMDHHHHDMMMMM",
        "MMMMMMDDDDMMMMMM",
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((PLANK_D[0], PLANK_D[1], PLANK_D[2], 255)),
        'M' => Some((PLANK_M[0], PLANK_M[1], PLANK_M[2], 255)),
        'L' => Some((PLANK_L[0], PLANK_L[1], PLANK_L[2], 255)),
        'H' => Some((HOLE[0], HOLE[1], HOLE[2], 255)),
        _ => None,
    });
}

/// beehive FRONT at honey_level 5 — honey oozing from the slot.
pub(super) fn beehive_front_honey_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "MMLLLMMMMMMLLLMM",
        "DDDDLLHLLHLLDDDD",
        "MMMMLHHHHHHLMMMM",
        "MMMDLHHHHHHHDMMM",
        "MMMDLHHHHHHHDMMM",
        "MMMMLHHHHHHLMMMM",
        "DDDDLLHHHLLLDDDD",
        "MMLLLMLHHLMLLLMM",
        "MMLLLMMHHMMLLLMM",
        "DDDDDDLLDDDDDDDD",
        "MMLLLMMMMMMLLLMM",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((PLANK_D[0], PLANK_D[1], PLANK_D[2], 255)),
        'M' => Some((PLANK_M[0], PLANK_M[1], PLANK_M[2], 255)),
        'L' => Some((HONEY_L[0], HONEY_L[1], HONEY_L[2], 255)),
        'H' => Some((HONEY_M[0], HONEY_M[1], HONEY_M[2], 255)),
        _ => None,
    });
}

/// the honey BLOCK — amber translucent gel (alpha ~170): a soft
/// gradient body with lighter highlight streaks and a denser rim.
pub(super) fn honey_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            let edge = x == 0 || x == 15 || y == 0 || y == 15;
            let corner = (x < 3 && y < 3) || (x > 12 && y < 3) || (x < 3 && y > 12) || (x > 12 && y > 12);
            let (r, g, b, al) = if edge {
                (HONEY_D[0], HONEY_D[1], HONEY_D[2], 210)
            } else if corner {
                (HONEY_M[0], HONEY_M[1], HONEY_M[2], 200)
            } else {
                // inner gel: radial lightening
                let dx = (x as f32 - 7.5) / 7.5;
                let dy = (y as f32 - 7.5) / 7.5;
                let d = (dx * dx + dy * dy).sqrt();
                let mix = (1.0 - d).max(0.0);
                (
                    HONEY_M[0] + ((HONEY_L[0] - HONEY_M[0]) as f32 * mix) as i32,
                    HONEY_M[1] + ((HONEY_L[1] - HONEY_M[1]) as f32 * mix) as i32,
                    HONEY_M[2] + ((HONEY_L[2] - HONEY_M[2]) as f32 * mix) as i32,
                    170,
                )
            };
            super::put(a, t, x, y, r, g, b, al);
        }
    }
    // the glossy streak (a diagonal highlight band)
    for i in 2..9 {
        super::put(a, t, i, 13 - i, HONEY_L[0], HONEY_L[1], HONEY_L[2], 220);
        super::put(a, t, i + 1, 13 - i, HONEY_L[0], HONEY_L[1], HONEY_L[2], 190);
    }
    // a couple of trapped-air glints
    for _ in 0..3 {
        let x = 2 + rng.next_range(12) as i32;
        let y = 2 + rng.next_range(12) as i32;
        super::put(a, t, x, y, 252, 236, 170, 230);
    }
}

/// the honeycomb BLOCK — the hexagonal-cell wall (amber cells with
/// darker rims — the signature honeycomb pattern).
pub(super) fn honeycomb_block_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MHHLLHHLLHHLLHHM",
        "MLHDDHHLHDDHHLHM",
        "MLHDDHHLHDDHHLHM",
        "MHHLLHHLLHHLLHHM",
        "MHHLLHHLLHHLLHHM",
        "MLHDDHHLHDDHHLHM",
        "MLHDDHHLHDDHHLHM",
        "MHHLLHHLLHHLLHHM",
        "MHHLLHHLLHHLLHHM",
        "MLHDDHHLHDDHHLHM",
        "MLHDDHHLHDDHHLHM",
        "MHHLLHHLLHHLLHHM",
        "MHHLLHHLLHHLLHHM",
        "MLHDDHHLHDDHHLHM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some((HONEY_D[0], HONEY_D[1], HONEY_D[2], 255)),
        'L' => Some((HONEY_L[0], HONEY_L[1], HONEY_L[2], 255)),
        'H' => Some((HONEY_M[0], HONEY_M[1], HONEY_M[2], 255)),
        'D' => Some((HOLE[0], HOLE[1], HOLE[2], 255)),
        _ => None,
    });
}

/// the honeycomb ITEM — a broken comb chunk: hexagon cells with a
/// honey-filled face on a transparent background.
pub(super) fn honeycomb_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "....DDDDDDD.....",
        "..DDHHHHHHHDD...",
        ".DHHLLHHHLLHHD..",
        ".DHLLHHDHHLLHD..",
        "DHHLLHHDHHLLHHD.",
        "DHLHDDHHHDDHLHD.",
        "DHHDDHHHHHDDHHD.",
        ".DHHHHHHHHHHHD..",
        ".DHLHHLLHHLHHD..",
        "..DHHLHHHLHHD...",
        "...DDHHHHHDD....",
        ".....DDDDD......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((HONEY_D[0], HONEY_D[1], HONEY_D[2], 255)),
        'L' => Some((HONEY_L[0], HONEY_L[1], HONEY_L[2], 255)),
        'H' => Some((HONEY_M[0], HONEY_M[1], HONEY_M[2], 255)),
        _ => None,
    });
}

/// the honey BOTTLE sprite — the glass flask with the amber fill.
pub(super) fn honey_bottle_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "......KKKK......",
        "......KDDK......",
        "......KDDK......",
        ".....GGGGGG.....",
        "....GGGGGGGG....",
        "...GGGGGGGGGG...",
        "...GGHHHHHHGG...",
        "...GGHHHHHHGG...",
        "...GGHHHHHHGG...",
        "...GGHLHHHHGG...",
        "...GGHLHHHHGG...",
        "....GGHHHHGG....",
        ".....GGGGGG.....",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((96, 70, 40, 255)),       // cork
        'D' => Some((74, 54, 34, 255)),       // neck shadow
        'G' => Some((196, 220, 228, 190)),    // glass body (translucent)
        'H' => Some((HONEY_M[0], HONEY_M[1], HONEY_M[2], 235)),
        'L' => Some((HONEY_L[0], HONEY_L[1], HONEY_L[2], 235)),
        _ => None,
    });
}

/// the SHEARS sprite — two crossed steel blades with dark handles
/// (the legacy tool, clean-room).
pub(super) fn shears_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        ".LL..........LL",
        ".LML........LML",
        "..LML......LML.",
        "...LML....LML..",
        "....LML..LML...",
        ".....LMLLML....",
        "......LMMML....",
        ".....LLDDLL....",
        "....HDL..LDH...",
        "...HDH....HDH..",
        "..HDH......HDH.",
        ".HDH........HDH",
        ".HH..........HH",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'L' => Some((IRON_L[0], IRON_L[1], IRON_L[2], 255)),
        'M' => Some((IRON_M[0], IRON_M[1], IRON_M[2], 255)),
        'D' => Some((IRON_D[0], IRON_D[1], IRON_D[2], 255)),
        'H' => Some((84, 60, 36, 255)), // handle
        _ => None,
    });
}

/// the bee SPAWN EGG sprite — the egg-shaped item with the bee's
/// amber + dark spot palette (the EGG_PALETTES convention).
pub(super) fn bee_egg_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
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
        'A' => Some((236, 188, 108, 255)), // amber shell
        'S' => Some((58, 44, 28, 255)),    // dark spots
        _ => None,
    });
}

/// the bee MOB billboard — the hovering bee seen from the side:
/// striped amber abdomen, dark head with eyes, two pale wings, the
/// stinger at the tail. Nectar-carrying bees show the pollen spots
/// (rendered via the tint channel at draw time — this is the base
/// sprite; the angry red-eyes variant tints at draw time too).
pub(super) fn bee_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "....WW....WW....",
        "...WWWW..WWWW...",
        "....WW.SS.WW....",
        ".....WSSSSW.....",
        "......SSSS......",
        "....DDSSSSDD....",
        "...DHHDHHHDHH...",
        "..DHHHDHHHDHHD..",
        "..DHHDHHHHHDHH..",
        "..DDHHDHHHDHHD..",
        "...DDHDDDDHDD...",
        ".....D.ww.D.....",
        "..........w.....",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((BEE_WING[0], BEE_WING[1], BEE_WING[2], 160)), // translucent wings
        'S' => Some((BEE_DARK[0], BEE_DARK[1], BEE_DARK[2], 255)), // head/thorax
        'H' => Some((BEE_AMBER[0], BEE_AMBER[1], BEE_AMBER[2], 255)), // amber stripes
        'D' => Some((BEE_DARK[0], BEE_DARK[1], BEE_DARK[2], 255)),   // dark stripes
        'w' => Some((60, 46, 30, 255)),  // the stinger
        _ => None,
    });
}
