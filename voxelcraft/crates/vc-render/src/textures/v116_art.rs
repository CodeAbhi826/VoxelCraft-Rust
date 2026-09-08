//! 1.16 bracket (Nether Update, part 1 — the anchor family) procedural
//! tiles, the V13 window (tiles 655..=672): soul soil, the basalt
//! top/side pair, blackstone + gilded, crying obsidian, the respawn
//! anchor trio (top/side/charged side), the target, nether gold ore,
//! the ancient-debris top/side pair, the block of netherite, the
//! chain, soul fire, and the netherite scrap/ingot item sprites.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares the art helper). Covered by the `v114b_tiles_all_painted`
//! atlas guard (its loop runs 634..=TILE_MAX, so the new window rides
//! the same art-gap regression guard).

use super::{art, Rng};

// ---- shared palettes ----
/// soul soil: ash-brown dirt with charcoal grain (no faces — that is
/// soul sand's trademark; VERIFIED distinct from the sand family).
const SOIL_D: [i32; 3] = [64, 48, 38];
const SOIL_M: [i32; 3] = [86, 66, 50];
const SOIL_L: [i32; 3] = [108, 84, 62];
/// basalt: blue-gray volcanic strata.
const BASALT_D: [i32; 3] = [72, 74, 78];
const BASALT_M: [i32; 3] = [96, 98, 104];
const BASALT_L: [i32; 3] = [122, 124, 130];
/// blackstone: near-black rough charcoal.
const BLACK_D: [i32; 3] = [24, 22, 26];
const BLACK_M: [i32; 3] = [42, 39, 44];
const BLACK_L: [i32; 3] = [60, 56, 62];
/// the gold flecks (gilded blackstone + nether gold ore).
const GOLD_L: [i32; 3] = [250, 214, 64];
const GOLD_M: [i32; 3] = [222, 178, 45];
/// crying obsidian: obsidian-dark with glowing purple tears.
const OBS_D: [i32; 3] = [16, 10, 26];
const OBS_M: [i32; 3] = [28, 18, 44];
const TEAR: [i32; 3] = [130, 84, 200];
const TEAR_L: [i32; 3] = [178, 128, 240];
/// the anchor's purple glow (the charged sides).
const GLOW: [i32; 3] = [124, 62, 190];
const GLOW_L: [i32; 3] = [172, 110, 240];
/// the target: bone-white rings on a red-ocher center.
const RING_W: [i32; 3] = [222, 214, 196];
const RING_R: [i32; 3] = [196, 84, 52];
const RING_D: [i32; 3] = [156, 60, 36];
/// netherrack base tones (the gold-ore host).
const RACK_D: [i32; 3] = [86, 30, 24];
const RACK_M: [i32; 3] = [112, 42, 32];
/// ancient debris: bronze-brown with the distinctive spiral.
const DEB_D: [i32; 3] = [96, 70, 42];
const DEB_M: [i32; 3] = [128, 96, 58];
const DEB_L: [i32; 3] = [158, 122, 74];
/// netherite: dark slate metal.
const NETH_D: [i32; 3] = [44, 42, 48];
const NETH_M: [i32; 3] = [62, 58, 66];
const NETH_L: [i32; 3] = [84, 80, 90];
/// iron chain links.
const IRON_M: [i32; 3] = [146, 150, 156];
const IRON_D: [i32; 3] = [96, 100, 106];
/// soul fire: the blue flame.
const FLAME_IN: [i32; 3] = [200, 236, 252];
const FLAME_M: [i32; 3] = [92, 158, 226];
const FLAME_OUT: [i32; 3] = [36, 88, 190];

/// soul soil — ash-dirt with dark grain and hollow specks.
pub(super) fn soul_soil_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMDDMMMMMDDMM",
        "MDDMMLLMMMDDMMMm",
        "DMLLMMMDMMLLMMMD",
        "MMDDMMMDDMMMDDMM",
        "MMLLDMMLLMMMLLMM",
        "MDDMMMMDDMMMDDMM",
        "MMMMDMMMLLDMMMMD",
        "DMLLMMMDDMMMMDDM",
        "MMDMMMMLLMMMMDMM",
        "MMDDMMMDDMMLLMMM",
        "MLLMMMMMDMMMDDMM",
        "MDDMMMMLLMMMMDMM",
        "MMMMDMMMDDMMMLLm",
        "DMLLMMMMDMMLLMMM",
        "MDDMMMMDDMMMDDMM",
        "MMMMMDDMMMMMDDMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((SOIL_D[0], SOIL_D[1], SOIL_D[2], 255)),
        'M' => Some((SOIL_M[0], SOIL_M[1], SOIL_M[2], 255)),
        'L' => Some((SOIL_L[0], SOIL_L[1], SOIL_L[2], 255)),
        'm' => Some((BLACK_D[0], BLACK_D[1], BLACK_D[2], 255)),
        _ => None,
    });
    for _ in 0..10 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, BLACK_D[0], BLACK_D[1], BLACK_D[2], 255);
    }
}

/// basalt side — vertical volcanic strata.
pub(super) fn basalt_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMDMMMMDMMMMDMM",
        "LLMMDMMLLMMDMMMD",
        "MDDMMMDMDDMMMDMM",
        "MDMMMLLMDMMMLLMD",
        "MMMDMMMDMMMMDMMM",
        "MMDMMDDMMDMMDDMM",
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
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((BASALT_D[0], BASALT_D[1], BASALT_D[2], 255)),
        'M' => Some((BASALT_M[0], BASALT_M[1], BASALT_M[2], 255)),
        'L' => Some((BASALT_L[0], BASALT_L[1], BASALT_L[2], 255)),
        _ => None,
    });
}

/// basalt top — the pillar's ringed end grain.
pub(super) fn basalt_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDDDDDDDDDDDDDM",
        "MDMMMMMMMMMMMMDM",
        "MDMDDDDDDDDDMDMM",
        "MDMDMMMMMMMMDMDM",
        "MDMMDMDDDMMDMMDM",
        "MDMDMDMMMDMDMDMD",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMLLMDMDMDM",
        "MDMDMDMMMDMDMDMD",
        "MDMMDMDDDMMDMMDM",
        "MDMDMMMMMMMMDMDM",
        "MDMDDDDDDDDDMDMM",
        "MDMMMMMMMMMMMMDM",
        "MDDDDDDDDDDDDDDM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((BASALT_D[0], BASALT_D[1], BASALT_D[2], 255)),
        'M' => Some((BASALT_M[0], BASALT_M[1], BASALT_M[2], 255)),
        'L' => Some((BASALT_L[0], BASALT_L[1], BASALT_L[2], 255)),
        _ => None,
    });
}

/// blackstone — rough near-black charcoal.
pub(super) fn blackstone_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMDMMMMMMDMMM",
        "MMDDMMMMDDMMMMDM",
        "MDMMMMDMMMMMMDMM",
        "MMMMDMMMDDMMMMMD",
        "MDMMMMDMMMMMDDMM",
        "MMMDDMMMMMDMMMMD",
        "MMDMMMMDDMMMMDMM",
        "MDMMMMDMMMMMMDMM",
        "MMMMDMMMDDMMMMDM",
        "MDMMMMDMMMMMDMMM",
        "MMMDDMMMMMMDMMMD",
        "MMDMMMDDMMMMMDMM",
        "MDMMMMMMMMDDMMMD",
        "MMMDMMMMDMMMMMDM",
        "MDDMMMMMMMMMMDMM",
        "MMMMMDMMMMMDDMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((BLACK_D[0], BLACK_D[1], BLACK_D[2], 255)),
        'M' => Some((BLACK_M[0], BLACK_M[1], BLACK_M[2], 255)),
        _ => None,
    });
    for _ in 0..8 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, BLACK_L[0], BLACK_L[1], BLACK_L[2], 255);
    }
}

/// gilded blackstone — blackstone with gold veins.
pub(super) fn gilded_blackstone_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    blackstone_art(a, t, rng);
    // the gold runs: diagonal glints in fixed + jittered spots
    let veins = [(3, 4), (4, 4), (5, 5), (11, 8), (12, 8), (12, 9), (7, 12), (8, 12), (8, 13)];
    for (x, y) in veins {
        super::put(a, t, x, y, GOLD_M[0], GOLD_M[1], GOLD_M[2], 255);
    }
    for _ in 0..4 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, GOLD_L[0], GOLD_L[1], GOLD_L[2], 255);
    }
}

/// crying obsidian — dark glass with luminous purple tears.
pub(super) fn crying_obsidian_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MMTTMMMMMMMMMMTM",
        "MMTLMMTTMMMMMMDM",
        "MMTMMMTLTMMMMMMM",
        "MMMMMMTMMMMTTMMM",
        "MMMMMMMMMMTLMMMM",
        "MMTTMMMMMMTMMMMM",
        "MMTLMMMMMMMMMMMM",
        "MMTMMMMMMTTMMMMM",
        "MMMMMMMMMTLTMMMM",
        "MMMMDMMMMTMMMMMM",
        "MMMMMMMMMMMMMTTM",
        "MTTMMMMMMMMMMTLM",
        "MTLTMMMMMMMMMTMM",
        "MTMMMMMMMMMMMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((OBS_D[0], OBS_D[1], OBS_D[2], 255)),
        'M' => Some((OBS_M[0], OBS_M[1], OBS_M[2], 255)),
        'T' => Some((TEAR[0], TEAR[1], TEAR[2], 255)),
        'L' => Some((TEAR_L[0], TEAR_L[1], TEAR_L[2], 255)),
        _ => None,
    });
    for _ in 0..5 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, TEAR_L[0], TEAR_L[1], TEAR_L[2], 255);
    }
}

/// respawn-anchor top — the dark shell ring around the swirling
/// portal face (the glow wakes with charge; the tile is shared, the
/// emissive tint does the waking).
pub(super) fn anchor_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDDDDDDDDDDDDDM",
        "MDMMGGGGGGGGMMDM",
        "MDMGGLLGGGGLLGMD",
        "MDMGLLGGGGGGLGMD",
        "MDMGGGGGDDGGGGMD",
        "MDMGGGGDDDDGGGMD",
        "MDMGGGDDLLDDGGMD",
        "MDMGGGDDLLDDGGMD",
        "MDMGGGGDDDDGGGMD",
        "MDMGGGGGDDGGGGMD",
        "MDMGLLGGGGGGLGMD",
        "MDMGGLLGGGGLLGMD",
        "MDMMGGGGGGGGMMDM",
        "MDDDDDDDDDDDDDDM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((OBS_D[0], OBS_D[1], OBS_D[2], 255)),
        'M' => Some((OBS_M[0], OBS_M[1], OBS_M[2], 255)),
        'G' => Some((GLOW[0], GLOW[1], GLOW[2], 255)),
        'L' => Some((GLOW_L[0], GLOW_L[1], GLOW_L[2], 255)),
        _ => None,
    });
    for _ in 0..6 {
        let x = 1 + rng.next_range(14) as i32;
        let y = 1 + rng.next_range(14) as i32;
        super::put(a, t, x, y, GLOW_L[0], GLOW_L[1], GLOW_L[2], 255);
    }
}

/// respawn-anchor sides, uncharged — the dark obsidian shell.
pub(super) fn anchor_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMDMMMMMMMMDMMD",
        "DMMDMMMMMMMMDMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMDMMMMMMMMDMMD",
        "DMMDMMMMMMMMDMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMMMMMMMMMMMMMD",
        "DMMDMMMMMMMMDMMD",
        "DMMMMMMMMMMMMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((OBS_D[0], OBS_D[1], OBS_D[2], 255)),
        'M' => Some((OBS_M[0], OBS_M[1], OBS_M[2], 255)),
        _ => None,
    });
}

/// respawn-anchor sides at charge >= 1 — the purple glow veins wake.
pub(super) fn anchor_side_charged_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "DDDDDDDDDDDDDDDD",
        "DGMMMMMMMMMMGMD",
        "DMGMMDMMMMGMMMMD",
        "DMMMGMMMDMMGMGMD",
        "DMMDMMMGMMMMDMMD",
        "DMMMMMDMMMGMMMMD",
        "DMMDMMMMMMGMMMMD",
        "DMMGMMMMMDMMMMMD",
        "DMMDMMGMMMMMGmmd",
        "DMMMMMMMDMGMMMMD",
        "DMMDGMMMMMMMMMMD",
        "DMMMDMMMGMMMMMMD",
        "DMGMMMMMDMMMMGMD",
        "DMMMMGMMMMMMMDMD",
        "DMDMMMMMMMMMMMMD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((OBS_D[0], OBS_D[1], OBS_D[2], 255)),
        'M' => Some((OBS_M[0], OBS_M[1], OBS_M[2], 255)),
        'G' => Some((GLOW[0], GLOW[1], GLOW[2], 255)),
        'm' => Some((GLOW_L[0], GLOW_L[1], GLOW_L[2], 255)),
        _ => None,
    });
    for _ in 0..8 {
        let x = rng.next_range(16) as i32;
        let y = 1 + rng.next_range(14) as i32;
        super::put(a, t, x, y, GLOW[0], GLOW[1], GLOW[2], 255);
    }
}

/// the target — concentric rings, the bullseye face.
pub(super) fn target_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "WWWWWWWWWWWWWWWW",
        "WRRRRRRRRRRRRRRW",
        "WRWWWWWWWWWWWWRW",
        "WRWRRRRRRRRRWWRW",
        "WRWRWWWWWWWWRWRW",
        "WRWRWRRRRRWWRWRW",
        "WRWRWRWDDWRWRWRW",
        "WRWRWRWDDWRWRWRW",
        "WRWRWRWDDWRWRWRW",
        "WRWRWRRRRRWWRWRW",
        "WRWRWWWWWWWWRWRW",
        "WRWRRRRRRRRRWWRW",
        "WRWWWWWWWWWWWWRW",
        "WRRRRRRRRRRRRRRW",
        "WWWWWWWWWWWWWWWW",
        "WWWWWWWWWWWWWWWW",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((RING_W[0], RING_W[1], RING_W[2], 255)),
        'R' => Some((RING_R[0], RING_R[1], RING_R[2], 255)),
        'D' => Some((RING_D[0], RING_D[1], RING_D[2], 255)),
        _ => None,
    });
}

/// nether gold ore — netherrack host with gold nuggets.
pub(super) fn nether_gold_ore_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDMMGLGMMDDMMMM",
        "MDMMMGLMMMDMMMMD",
        "MMMMMMMMMMDMGLMM",
        "MGLMMDDMMMMDMGMM",
        "MGLMMMMDMMMMMMMM",
        "MMMMMMMMMMDDMGLM",
        "MDDMMDDMMMMDMGLM",
        "MDMMMMMMMMDMMMMM",
        "MMMMMGLGMMMMMDDM",
        "MDDMMGLMMDDMMMMD",
        "MDMMMMMMMMMMMMMM",
        "MMMMDDMMMGLMMMMD",
        "MGLMMMMDMGLMMMMM",
        "MGLMMMMMMMMMMDMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((RACK_D[0], RACK_D[1], RACK_D[2], 255)),
        'M' => Some((RACK_M[0], RACK_M[1], RACK_M[2], 255)),
        'G' => Some((GOLD_M[0], GOLD_M[1], GOLD_M[2], 255)),
        'L' => Some((GOLD_L[0], GOLD_L[1], GOLD_L[2], 255)),
        _ => None,
    });
    for _ in 0..4 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, GOLD_L[0], GOLD_L[1], GOLD_L[2], 255);
    }
}

/// ancient-debris top — the bronze end grain.
pub(super) fn ancient_debris_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "MMMMMMMMMMMMMMMM",
        "MDDDDDDDDDDDDDDM",
        "MDMMLLLLMMLLLDMD",
        "MDMLLDDDLLMMDLMD",
        "MDMLDDLLDDLLDLMD",
        "MDMLDLLDDLLLDLMD",
        "MDMLDLDDLLDDLLMD",
        "MDMLDLLLLDLLDLMD",
        "MDMLDLDDLLDLLLMD",
        "MDMLDLLDDLLDDLMD",
        "MDMLDDLLDLLDDLLM",
        "MDMLLDDDLLDDLLLM",
        "MDMMLLLLLMMLLLMD",
        "MDDDDDDDDDDDDDDM",
        "MMMMMMMMMMMMMMMM",
        "MMMMMMMMMMMMMMMM",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((DEB_D[0], DEB_D[1], DEB_D[2], 255)),
        'M' => Some((DEB_M[0], DEB_M[1], DEB_M[2], 255)),
        'L' => Some((DEB_L[0], DEB_L[1], DEB_L[2], 255)),
        _ => None,
    });
}

/// ancient-debris side — the distinctive spiral banding.
pub(super) fn ancient_debris_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "LLLLLLLLLLLLLLLL",
        "LMMMMMMMMMMMMMML",
        "LMDDLLDDLLDDLLML",
        "LMDLLDDLLDDLLDML",
        "LMDLDDLLDDLLLDML",
        "LMMLLDDLLDDLLLML",
        "LMDDLLDDLLDDLLML",
        "LMDLLDDLLDDLLDML",
        "LMDLDDLLDDLLLDML",
        "LMMLLDDLLDDLLLML",
        "LMDDLLDDLLDDLLML",
        "LMDLLDDLLDDLLDML",
        "LMDDDDDDDDDDDDML",
        "LMMMMMMMMMMMMMML",
        "LLLLLLLLLLLLLLLL",
        "LLLLLLLLLLLLLLLL",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((DEB_D[0], DEB_D[1], DEB_D[2], 255)),
        'M' => Some((DEB_M[0], DEB_M[1], DEB_M[2], 255)),
        'L' => Some((DEB_L[0], DEB_L[1], DEB_L[2], 255)),
        _ => None,
    });
}

/// the block of netherite — dark slate with a subtle bevel.
pub(super) fn netherite_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "LLLLLLLLLLLLLLLD",
        "LMMMMMMMMMMMMMDD",
        "LMDDDDDDDDDDDMDd",
        "LMDMMMMMMMMMMDMD",
        "LMDMLLLLLLLLMdMD",
        "LMDMLDMMMDDLDMDD",
        "LMDMLDMDDLDLDMDD",
        "LMDMLDMLLDLDMDDD",
        "LMDMLDLDDLMDMDDD",
        "LMDMLDMDDLDMDDDD",
        "LMDMLLLLLLLDMDDD",
        "LMDMMMMMMMMMDDDD",
        "LMDDDDDDDDDDDMDD",
        "LMMMMMMMMMMMMMDD",
        "LDDDDDDDDDDDDDDD",
        "DDDDDDDDDDDDDDDD",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((NETH_D[0], NETH_D[1], NETH_D[2], 255)),
        'M' => Some((NETH_M[0], NETH_M[1], NETH_M[2], 255)),
        'L' => Some((NETH_L[0], NETH_L[1], NETH_L[2], 255)),
        'd' => Some((NETH_L[0], NETH_L[1], NETH_L[2], 255)),
        _ => None,
    });
    for _ in 0..6 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        super::put(a, t, x, y, NETH_D[0], NETH_D[1], NETH_D[2], 255);
    }
}

/// the chain — vertical iron links on transparency (the cross-sprite
/// render path; the placement state offsets it like the lantern).
pub(super) fn chain_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "....XXXXXXXX....",
        "...X........X...",
        "...X.XXXXX.X....",
        "...X.X....XX....",
        "....XXXXXX......",
        "................",
        "....XXXXXX......",
        "...X.X....XX....",
        "...X.XXXXX.X....",
        "...X........X...",
        "....XXXXXXXX....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'X' => Some((IRON_M[0], IRON_M[1], IRON_M[2], 255)),
        '.' => Some((0, 0, 0, 0)),
        _ => None,
    });
    // link shading: darker left edge of each link column
    for y in [2, 8] {
        for dy in 0..5 {
            super::put(a, t, 3, y + dy, IRON_D[0], IRON_D[1], IRON_D[2], 255);
            super::put(a, t, 12, y + dy, IRON_D[0], IRON_D[1], IRON_D[2], 255);
        }
    }
}

/// soul fire — the blue flame cross-sprite (transparent ground).
pub(super) fn soul_fire_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "................",
        "......I.........",
        "......I....M....",
        "...M..I..M.M....",
        "...M.MI.M..I.M..",
        "..M..MIM..I..M..",
        "..M.M.MI.M..M...",
        "...M..MIM.M.....",
        "..M.M..MIM......",
        "..M.M.M.MIM.M...",
        "...M.M...MIM....",
        "..M..M.M..MI....",
        "...M...M..M.M...",
        "........M..M....",
        ".............M..",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'I' => Some((FLAME_IN[0], FLAME_IN[1], FLAME_IN[2], 255)),
        'M' => Some((FLAME_M[0], FLAME_M[1], FLAME_M[2], 255)),
        '.' => None,
        _ => None,
    });
    for _ in 0..10 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(14) as i32;
        super::put(a, t, x, y, FLAME_OUT[0], FLAME_OUT[1], FLAME_OUT[2], 200);
    }
    // the bright core column
    for y in 1..14 {
        super::put(a, t, 7, y, FLAME_IN[0], FLAME_IN[1], FLAME_IN[2], 255);
    }
}

/// netherite scrap item — a rough bronze shard.
pub(super) fn netherite_scrap_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "......LLLL......",
        ".....LMMMDL.....",
        "....LMDLLMDL....",
        "...LMLLDDLLML...",
        "...LMLDLLDDLL...",
        "..LMLLDDDLDLML..",
        "..LMLDLLLLDLLM..",
        "..LMDLLLLLLDM...",
        "...LMLLLLLML....",
        "...LDLLLLLMD....",
        "....LMLLLML.....",
        ".....LLMML......",
        "......LLM.......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((DEB_D[0], DEB_D[1], DEB_D[2], 255)),
        'M' => Some((DEB_M[0], DEB_M[1], DEB_M[2], 255)),
        'L' => Some((DEB_L[0], DEB_L[1], DEB_L[2], 255)),
        '.' => Some((0, 0, 0, 0)),
        _ => None,
    });
}

/// netherite ingot item — the dark slate bar.
pub(super) fn netherite_ingot_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "....LLLLLLLL....",
        "...LMMMMMMMMD...",
        "..LMLDDDDDMMMD..",
        "..LMMDLLLLMMMD..",
        ".LMMDLLLLLMMMD..",
        ".LMMDLLLLMMMDD..",
        ".LMMDLLLMMMDDD..",
        ".LMDDDDDDDDDDD..",
        ".LMMMMMMMMMMMD..",
        ".LDDDDDDDDDDDD..",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'D' => Some((NETH_D[0], NETH_D[1], NETH_D[2], 255)),
        'M' => Some((NETH_M[0], NETH_M[1], NETH_M[2], 255)),
        'L' => Some((NETH_L[0], NETH_L[1], NETH_L[2], 255)),
        '.' => Some((0, 0, 0, 0)),
        _ => None,
    });
}
