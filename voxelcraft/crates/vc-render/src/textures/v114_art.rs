//! 1.14 bracket (Village & Pillage — nature half) procedural tiles — the
//! V10 window (tiles 619..=633): the bamboo stalk + shoot, the sweet
//! berry bush's 4 growth stages, the lit + unlit campfire, the barrel's
//! lid and stave faces, the sweet berries / stick / charcoal item icons,
//! the fox spawn egg (via the E-series egg convention) and the fox mob
//! sprite.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares put/jit/noise_fill/art helpers). Guarded by the
//! `v114_tiles_all_painted` coverage test — the 1.13 art-gap regression
//! (a window with TILE_MAX raised but no painters renders BLANK) can
//! never silently repeat.

use super::{art, noise_fill, put, Rng};

/// clean-room bamboo green (the culm) + its node ring + leaf tone.
const BAMBOO_G: [i32; 3] = [116, 158, 58];
const BAMBOO_D: [i32; 3] = [86, 126, 42];
const BAMBOO_L: [i32; 3] = [150, 190, 80];

/// bamboo stalk (cross-rendered column tile): a 4-px segmented culm
/// with darker node rings and small leaf blades near the nodes.
pub(super) fn bamboo_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "..L...L...L..L..",
        "..GL..GL..G.GL..",
        "....G.G..G..G...",
        "....GGG.GGG.GG..",
        "....GGG.GGG.GG..",
        "....GGG.GGG.GG..",
        "....DDD.DDD.DD..",
        "....GGG.GGG.GG..",
        "L...GGG.GGG.GG..",
        "GL..GGG.GGG.GG..",
        ".G..GGG.GGG.GG..",
        "....DDD.DDD.DD..",
        "....GGG.GGG.GG..",
        "....GGG.GGG.GG..",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((BAMBOO_G[0], BAMBOO_G[1], BAMBOO_G[2], 255)),
        'D' => Some((BAMBOO_D[0], BAMBOO_D[1], BAMBOO_D[2], 255)),
        'L' => Some((BAMBOO_L[0], BAMBOO_L[1], BAMBOO_L[2], 255)),
        _ => None,
    });
}

/// bamboo shoot — the planted sapling form ("the initial non-solid
/// sapling form of planted bamboo", VERIFIED w/Bamboo): a tiny 2-px
/// sprout with one leaf pair.
pub(super) fn bamboo_shoot_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "......L.........",
        ".....LG.........",
        ".....GG...L.....",
        ".....GG..LG.....",
        ".....GG...G.....",
        "....DGG.........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((BAMBOO_G[0], BAMBOO_G[1], BAMBOO_G[2], 255)),
        'D' => Some((BAMBOO_D[0], BAMBOO_D[1], BAMBOO_D[2], 255)),
        'L' => Some((BAMBOO_L[0], BAMBOO_L[1], BAMBOO_L[2], 255)),
        _ => None,
    });
}

/// sweet berry bush growth stage art (0..3): stage 0 the small green
/// sapling, 1 the leafy shrub, 2 fewer berries ("third growth stage"),
/// 3 the mature berry-laden bush (VERIFIED w/Sweet_Berry_Bush §Growth:
/// 20% chance per random tick; harvest at stage 2/3).
pub(super) fn berry_bush_art(a: &mut [u8], t: u16, stage: u8, rng: &mut Rng) {
    const LEAF: [i32; 3] = [58, 96, 44];
    const LEAF_D: [i32; 3] = [40, 70, 34];
    const BERRY: [i32; 3] = [178, 32, 44];
    match stage.min(3) {
        0 => {
            let rows = [
                "................",
                "................",
                "................",
                "................",
                "................",
                "................",
                "................",
                ".......L........",
                "......LLL.......",
                ".....LLLLL......",
                ".....L.L.L......",
                "......LLL.......",
                ".......L........",
                "................",
                "................",
                "................",
            ];
            art(a, t, rows, &|c| match c {
                'L' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
                _ => None,
            });
        }
        1 => {
            let rows = [
                "................",
                "................",
                ".....L.L.L......",
                "....LLLLLLL.....",
                "...LLLLLLLLL....",
                "...LLdLLLdLL....",
                "..LLLLLLLLLLL...",
                "..LLdLLLLLLdL...",
                "..LLLLLdLLLLL...",
                "...LLLLLLLLL....",
                "...LLdLLLLLL....",
                "....LLLLLLL.....",
                ".....L.L.L......",
                "................",
                "................",
                "................",
            ];
            art(a, t, rows, &|c| match c {
                'L' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
                'd' => Some((LEAF_D[0], LEAF_D[1], LEAF_D[2], 255)),
                _ => None,
            });
        }
        2 => {
            let rows = [
                "................",
                "................",
                ".....L.B.L......",
                "....LLBLLLL.....",
                "...BLLLLLLLB....",
                "...LLdLLLdLL....",
                "..LLBLLLLLLLb...",
                "..LLdLLBLLdL....",
                "..LLLLLdLLBLL...",
                "...LLLLLLLLL....",
                "...LLBLLLLLL....",
                "....LLLLBLL.....",
                ".....L.B.L......",
                "................",
                "................",
                "................",
            ];
            art(a, t, rows, &|c| match c {
                'L' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
                'd' => Some((LEAF_D[0], LEAF_D[1], LEAF_D[2], 255)),
                'B' => Some((BERRY[0], BERRY[1], BERRY[2], 255)),
                'b' => Some((150, 26, 37, 255)),
                _ => None,
            });
        }
        _ => {
            let rows = [
                "................",
                "................",
                ".....B.B.B......",
                "....LBBBLB......",
                "...BBLLLLBB.....",
                "...LLdBLBdLL....",
                "..BLLBLLLBLLB...",
                "..LLdLLBLLdL....",
                "..LBLBLdLBLBL...",
                "...LLBLLLLL.....",
                "...LLBLLBLL.....",
                "....LBLLBL......",
                ".....B.B.B......",
                "................",
                "................",
                "................",
            ];
            art(a, t, rows, &|c| match c {
                'L' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
                'd' => Some((LEAF_D[0], LEAF_D[1], LEAF_D[2], 255)),
                'B' => Some((BERRY[0], BERRY[1], BERRY[2], 255)),
                'b' => Some((150, 26, 37, 255)),
                _ => None,
            });
            // a couple of jitter berries so mature bushes don't read
            // as a uniform dot grid
            for _ in 0..4 {
                let x = rng.next_range(16) as i32;
                let y = rng.next_range(16) as i32;
                put(a, t, x, y, BERRY[0], BERRY[1], BERRY[2], 255);
            }
        }
    }
}

/// campfire (lit/unlit): the vanilla look is crossed logs with a coal
/// core. Lit: glowing orange coals + pale flame pixels; unlit: gray
/// ash core (VERIFIED w/Campfire: luminous 15 when lit).
pub(super) fn campfire_art(a: &mut [u8], t: u16, lit: bool, rng: &mut Rng) {
    const LOG: [i32; 3] = [110, 84, 50];
    const LOG_D: [i32; 3] = [86, 64, 38];
    let rows = [
        "................",
        "................",
        "................",
        "..D..........D..",
        "...D..####..D...",
        "....D######D....",
        "..D..CCCCCC..D..",
        "...DCCcccCCD....",
        "....CCccCC......",
        "...D..CC..D.....",
        "..D..........D..",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    if lit {
        const COAL: [i32; 3] = [235, 120, 30];
        const EMBER: [i32; 3] = [255, 180, 60];
        art(a, t, rows, &|c| match c {
            'D' => Some((LOG_D[0], LOG_D[1], LOG_D[2], 255)),
            '#' => Some((LOG[0], LOG[1], LOG[2], 255)),
            'C' => Some((COAL[0], COAL[1], COAL[2], 255)),
            'c' => Some((EMBER[0], EMBER[1], EMBER[2], 255)),
            _ => None,
        });
        // flame flecks above the coals
        for _ in 0..7 {
            let x = 5 + rng.next_range(6) as i32;
            let y = 2 + rng.next_range(4) as i32;
            put(a, t, x, y, 255, 200, 80, 255);
        }
    } else {
        const ASH: [i32; 3] = [96, 92, 88];
        const ASH_D: [i32; 3] = [70, 68, 64];
        art(a, t, rows, &|c| match c {
            'D' => Some((LOG_D[0], LOG_D[1], LOG_D[2], 255)),
            '#' => Some((LOG[0], LOG[1], LOG[2], 255)),
            'C' => Some((ASH[0], ASH[1], ASH[2], 255)),
            'c' => Some((ASH_D[0], ASH_D[1], ASH_D[2], 255)),
            _ => None,
        });
    }
}

/// barrel lid (top/bottom face): a wood disc — plank cross + the dark
/// band ring (the barrel's metal hoop).
pub(super) fn barrel_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    const WOOD: [i32; 3] = [124, 94, 58];
    const WOOD_D: [i32; 3] = [96, 70, 42];
    const BAND: [i32; 3] = [62, 58, 52];
    noise_fill(a, t, WOOD, 6, rng);
    let rows = [
        "................",
        "..############..",
        ".#BBBBBBBBBBBB#.",
        ".#B##WWWWWW##B#.",
        ".#B#WWWWWWWW#B#.",
        ".#BWWDWWWWDWWB#.",
        ".#BWWDWWWWDWWB#.",
        ".#BWWWWWWWWWWB#.",
        ".#BWWWWWWWWWWB#.",
        ".#B#WWWWWWWW#B#.",
        ".#B##WWWWWW##B#.",
        ".#BBBBBBBBBBBB#.",
        "..############..",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        '#' => Some((WOOD[0], WOOD[1], WOOD[2], 255)),
        'W' => Some((132, 100, 62, 255)),
        'D' => Some((WOOD_D[0], WOOD_D[1], WOOD_D[2], 255)),
        'B' => Some((BAND[0], BAND[1], BAND[2], 255)),
        _ => None,
    });
}

/// barrel side face: vertical staves + two horizontal bands (hoops).
pub(super) fn barrel_side_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    const WOOD: [i32; 3] = [120, 90, 56];
    const STAVE_D: [i32; 3] = [98, 72, 44];
    const BAND: [i32; 3] = [58, 54, 48];
    noise_fill(a, t, WOOD, 5, rng);
    let rows = [
        "BBBBBBBBBBBBBBBB",
        "BBBBBBBBBBBBBBBB",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        "BBBBBBBBBBBBBBBB",
        "BBBBBBBBBBBBBBBB",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        ".S.S.S..S.S.S...",
        "BBBBBBBBBBBBBBBB",
        "BBBBBBBBBBBBBBBB",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((STAVE_D[0], STAVE_D[1], STAVE_D[2], 255)),
        'B' => Some((BAND[0], BAND[1], BAND[2], 255)),
        _ => None,
    });
}

/// sweet berries item icon: a cluster of red berries with a green leaf.
pub(super) fn sweet_berries_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    const BERRY: [i32; 3] = [182, 34, 46];
    const BERRY_D: [i32; 3] = [142, 22, 32];
    const LEAF: [i32; 3] = [62, 104, 46];
    let rows = [
        "................",
        "......LL........",
        ".....LL.........",
        "................",
        "....BBBB........",
        "...BBwBBBB......",
        "...BBBBBBB......",
        "....BBBBBd......",
        "...dBd.BB.......",
        "..BBB..BB.......",
        ".BBdBb..........",
        ".BBB............",
        "..d.............",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((BERRY[0], BERRY[1], BERRY[2], 255)),
        'b' => Some((BERRY_D[0], BERRY_D[1], BERRY_D[2], 255)),
        'd' => Some((BERRY_D[0], BERRY_D[1], BERRY_D[2], 255)),
        'L' => Some((LEAF[0], LEAF[1], LEAF[2], 255)),
        'w' => Some((240, 170, 160, 255)),
        _ => None,
    });
    // berry highlights (the shine dot on each)
    for _ in 0..3 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(a, t, x, y, 235, 120, 110, 255);
    }
}

/// the fox mob sprite (side profile, the 1.13 billboard convention):
/// orange body, white chest + tail tip, black ear tips and legs, the
/// pale face with a dark eye (VERIFIED w/Fox: red fox coloring).
pub(super) fn fox_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const FUR: [i32; 3] = [206, 108, 44];
    const FUR_D: [i32; 3] = [172, 84, 30];
    const WHITE: [i32; 3] = [238, 234, 228];
    const DARK: [i32; 3] = [46, 40, 34];
    let rows = [
        "................",
        "................",
        "................",
        "..K.....O....O..",
        "..O....OO....OK.",
        "..OO..OOOO..OO..",
        "..OOOOOOOOOOO...",
        "...OOWOOOOOO....",
        "..OOWWWOOOOO....",
        "..OWWWOOOOOOO...",
        "..OWWOOOOOOOO...",
        "..OOOOOOOOOOOO..",
        "...OOOOOOOOOOWW.",
        "...K..K..K..KWW.",
        "...K..K..K..K...",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'O' => Some((FUR[0], FUR[1], FUR[2], 255)),
        'o' => Some((FUR_D[0], FUR_D[1], FUR_D[2], 255)),
        'W' => Some((WHITE[0], WHITE[1], WHITE[2], 255)),
        'K' => Some((DARK[0], DARK[1], DARK[2], 255)),
        _ => None,
    });
}

/// stick item icon: a diagonal 2-px brown stick.
pub(super) fn stick_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const WOOD: [i32; 3] = [138, 104, 62];
    const WOOD_D: [i32; 3] = [104, 76, 44];
    let rows = [
        "................",
        "............##..",
        "...........##...",
        "..........##....",
        ".........##.....",
        "........##......",
        ".......##.......",
        "......##........",
        ".....##.........",
        "....##..........",
        "...##...........",
        "..##............",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        '#' => Some((WOOD[0], WOOD[1], WOOD[2], 255)),
        '=' => Some((WOOD_D[0], WOOD_D[1], WOOD_D[2], 255)),
        _ => None,
    });
}

/// charcoal item icon: a black faceted lump with gray sheen pixels.
pub(super) fn charcoal_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    const BLACK: [i32; 3] = [42, 40, 38];
    const DARK: [i32; 3] = [28, 26, 25];
    const SHEEN: [i32; 3] = [92, 90, 88];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        ".....#####......",
        "....##s####.....",
        "...#######s#....",
        "..#s########....",
        "..##########....",
        "...####s####....",
        "....##s###......",
        ".....#####......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        '#' => Some((BLACK[0], BLACK[1], BLACK[2], 255)),
        's' => Some((SHEEN[0], SHEEN[1], SHEEN[2], 255)),
        _ => None,
    });
    // speckle the facets
    for _ in 0..5 {
        let x = 4 + rng.next_range(8) as i32;
        let y = 4 + rng.next_range(8) as i32;
        put(a, t, x, y, DARK[0], DARK[1], DARK[2], 255);
    }
}
