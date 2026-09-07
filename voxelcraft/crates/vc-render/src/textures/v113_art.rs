//! 1.13 bracket (Update Aquatic) procedural tiles — the V9 window
//! (tiles 550..=618): coral blocks/plants/fans ×5 colors + their dead
//! gray forms, sea pickle ×4 counts, blue ice, dried kelp block, kelp,
//! seagrass, the conduit, turtle eggs ×3 hatch stages, the 11 craft/
//! food/potion item icons, the 8 spawn eggs and the 8 aquatic mob
//! billboards.
//!
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares put/jit/noise_fill/art helpers). This closes the recovery
//! round's art gap: the interrupted-session commit landed the V9
//! registry window with TILE_MAX=618 but NO painters — every 1.13
//! block, item, egg and mob billboard rendered BLANK until now
//! (caught by the atlas-coverage audit; guarded by the new
//! `v113_tiles_all_painted` test).

use super::{art, noise_fill, put, Rng};

/// clean-room coral palette — the five vanilla color families
/// (tube=blue, brain=pink, bubble=purple, fire=red, horn=yellow);
/// per-pixel RGB are clean-room approximations (the wiki names the
/// colors, not hex values).
pub(super) const CORAL: [[i32; 3]; 5] = [
    [48, 105, 145],  // tube — blue
    [215, 130, 140], // brain — pink
    [150, 100, 170], // bubble — purple
    [190, 65, 60],   // fire — red
    [215, 175, 70],  // horn — yellow
];

/// dead coral: the irreversible desaturated gray form (VERIFIED
/// w/Coral: "when at least one of its sides is exposed to air" it dies).
const DEAD: [i32; 3] = [115, 115, 115];

/// coral block: a dense bumpy polyp texture — base color with darker
/// cellular flecks (the "many small tube/cavity" look).
pub(super) fn coral_block_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CORAL[(color as usize) % 5];
    noise_fill(a, t, base, 7, rng);
    for _ in 0..26 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(
            a,
            t,
            x,
            y,
            base[0] * 62 / 100,
            base[1] * 62 / 100,
            base[2] * 62 / 100,
            255,
        );
    }
}

/// dead coral block: the gray form, same structure.
pub(super) fn dead_coral_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, DEAD, 6, rng);
    for _ in 0..26 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(a, t, x, y, DEAD[0] * 62 / 100, DEAD[1] * 62 / 100, DEAD[2] * 62 / 100, 255);
    }
}

/// coral plant (cross-rendered): a thin stem with a polyp head.
pub(super) fn coral_plant_art(a: &mut [u8], t: u16, color: u8, _rng: &mut Rng) {
    let base = CORAL[(color as usize) % 5];
    let rows = [
        "................",
        "................",
        "................",
        "......PP........",
        ".....PPPP.......",
        ".....PPPP.......",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some(((base[0]), (base[1]), (base[2]), 255)),
        'S' => Some((
            (base[0] * 80 / 100),
            (base[1] * 80 / 100),
            (base[2] * 80 / 100),
            255,
        )),
        _ => None,
    });
}

/// dead coral plant.
pub(super) fn dead_coral_plant_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "......PP........",
        ".....PPPP.......",
        ".....PPPP.......",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "......SS........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some(((DEAD[0]), (DEAD[1]), (DEAD[2]), 255)),
        'S' => Some((
            (DEAD[0] * 80 / 100),
            (DEAD[1] * 80 / 100),
            (DEAD[2] * 80 / 100),
            255,
        )),
        _ => None,
    });
}

/// coral fan (cross-rendered): a branching fan wedge.
pub(super) fn coral_fan_art(a: &mut [u8], t: u16, color: u8, _rng: &mut Rng) {
    let base = CORAL[(color as usize) % 5];
    let rows = [
        "................",
        "....B..B..B.....",
        "....B..B..B.....",
        "....BB.B.BB.....",
        ".....B.B.B......",
        "....BBBBBBB.....",
        ".....BBBBB......",
        ".....BBBBB......",
        "......BBB.......",
        "......BBB.......",
        ".......B........",
        ".......B........",
        ".......B........",
        ".......B........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((
            (base[0] * 85 / 100),
            (base[1] * 85 / 100),
            (base[2] * 85 / 100),
            255,
        )),
        _ => None,
    });
}

/// dead coral fan.
pub(super) fn dead_coral_fan_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "....B..B..B.....",
        "....B..B..B.....",
        "....BB.B.BB.....",
        ".....B.B.B......",
        "....BBBBBBB.....",
        ".....BBBBB......",
        ".....BBBBB......",
        "......BBB.......",
        "......BBB.......",
        ".......B........",
        ".......B........",
        ".......B........",
        ".......B........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((DEAD[0] * 85 / 100), (DEAD[1] * 85 / 100), (DEAD[2] * 85 / 100), 255)),
        _ => None,
    });
}

/// sea pickle (cross-rendered): `count` 1..4 pickles side by side, each
/// a pale-green knobby rod (VERIFIED w/Sea_Pickle: 1..4 per block,
/// light 6/9/12/15 when submerged).
pub(super) fn sea_pickle_art(a: &mut [u8], t: u16, count: u8, _rng: &mut Rng) {
    const GREEN: [i32; 3] = [95, 140, 70];
    const LIGHT: [i32; 3] = [150, 200, 110];
    let n = (count as usize).clamp(1, 4);
    // 1: center; 2: pair; 3/4: row — thin vertical rods with nubs
    let xs: [i32; 4] = match n {
        1 => [7, -1, -1, -1],
        2 => [5, 9, -1, -1],
        3 => [4, 7, 10, -1],
        _ => [3, 6, 9, 12],
    };
    for x in xs.iter().take(n) {
        for y in 4..15 {
            for dy in 0..2 {
                put(
                    a,
                    t,
                    x + dy - 1,
                    y,
                    (GREEN[0]),
                    (GREEN[1]),
                    (GREEN[2]),
                    255,
                );
            }
        }
        // knobby head
        for (hx, hy) in [(0, 3), (1, 3), (0, 4), (1, 4)] {
            put(
                a,
                t,
                x + hx,
                hy,
                (LIGHT[0]),
                (LIGHT[1]),
                (LIGHT[2]),
                255,
            );
        }
    }
}

/// blue ice: a deep-blue near-flat sheet, slightly smoother than packed
/// ice with faint bright streaks (VERIFIED w/Blue_Ice: crafted from 9
/// packed ice; the slipperiest block).
pub(super) fn blue_ice_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [116, 167, 222], 4, rng);
    for _ in 0..8 {
        let x = rng.next_range(14) as i32;
        let y = rng.next_range(16) as i32;
        for i in 0..3 {
            put(a, t, x + i, y, 170, 215, 245, 255);
        }
    }
}

/// dried kelp block: dark pressed-kelp strands (fuel 4000 ticks).
pub(super) fn dried_kelp_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [45, 70, 45], 5, rng);
    for y in 0..16 {
        if y % 3 == 0 {
            for x in 0..16 {
                if rng.next_range(3) == 0 {
                    put(a, t, x, y, 30, 48, 30, 255);
                }
            }
        }
    }
}

/// kelp plant (cross): a green frond with leafy side blades (grows
/// 14% per random tick — VERIFIED w/Kelp).
pub(super) fn kelp_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const K: [i32; 3] = [60, 110, 55];
    const K2: [i32; 3] = [45, 85, 42];
    let rows = [
        "..L..L..L..L....",
        "...L..L..L..L...",
        "..L..S..L..L....",
        "...L.S.L..L.....",
        "..L..S..L..L....",
        "...L.S.L...L....",
        "..L..S..L..L....",
        "...L.S.L..L.....",
        "..L..S..L..L....",
        "...L.S.L...L....",
        "..L..S..L..L....",
        "...L.S.L..L.....",
        "..L..S..L..L....",
        "...L.S.L...L....",
        "......S.........",
        "......S.........",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some(((K[0]), (K[1]), (K[2]), 255)),
        'L' => Some(((K2[0]), (K2[1]), (K2[2]), 255)),
        _ => None,
    });
}

/// seagrass (cross): a tuft of curved green blades (turtle breeding
/// food — VERIFIED w/Turtle §Breeding: seagrass).
pub(super) fn seagrass_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const G: [i32; 3] = [70, 130, 60];
    let rows = [
        "................",
        "..G...G...G.....",
        "..G..G....G.....",
        "...G.G..G.G.....",
        "...G.G..G.G.....",
        "....GG.GG.......",
        "....GG.GG.......",
        "....G..G.G......",
        "....G..G.G......",
        "....G.G..G......",
        "....G.G..G......",
        "....GG.GG.......",
        ".....G..G.......",
        ".....G..G.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some(((G[0]), (G[1]), (G[2]), 255)),
        _ => None,
    });
}

/// the conduit: a shell frame with the glowing heart core (light 15).
pub(super) fn conduit_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const SHELL: [i32; 3] = [130, 200, 205];
    const HEART: [i32; 3] = [40, 90, 160];
    let rows = [
        "................",
        "................",
        "....SSSSSSSS....",
        "...S........S...",
        "..S...HHHH...S..",
        "..S..HHhhHH..S..",
        "..S.HHhhhhHH.S..",
        "..S.HhhhhhhH.S..",
        "..S.HhhhhhhH.S..",
        "..S.HHhhhhHH.S..",
        "..S..HHHHHH..S..",
        "...S........S...",
        "....SSSSSSSS....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some(((SHELL[0]), (SHELL[1]), (SHELL[2]), 255)),
        'H' => Some(((HEART[0]), (HEART[1]), (HEART[2]), 255)),
        'h' => Some((90, 160, 220, 255)),
        _ => None,
    });
}

/// turtle egg: `stage` 0=uncracked, 1=slightly cracked, 2=very cracked
/// (VERIFIED changelog §Blocks: "Turtle eggs ... go through stages").
pub(super) fn turtle_egg_art(a: &mut [u8], t: u16, stage: u8, _rng: &mut Rng) {
    const SHELL: [i32; 3] = [235, 225, 190];
    const CRACK: [i32; 3] = [150, 138, 105];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......EEEE......",
        "....EEEEEEEE....",
        "...EEEEEEEEEE...",
        "...EEEEcEEEEE...",
        "..EEcEEEEEEEEc..",
        "..EEEEEEcEEEEE..",
        "..cEEEEEEEEEEc..",
        "..EEEEEEEEEEEE..",
        "...EEEEEEEEEE...",
        "....EEEEEEEE....",
        "................",
        "................",
    ];
    // stage overlays: more/larger cracks as hatching approaches
    let extra = match stage {
        0 => vec![],
        1 => vec![(6, 8), (9, 6), (10, 10)],
        _ => vec![(5, 7), (6, 8), (7, 9), (9, 5), (10, 6), (11, 10), (8, 11), (7, 6)],
    };
    art(a, t, rows, &|c| match c {
        'E' => Some(((SHELL[0]), (SHELL[1]), (SHELL[2]), 255)),
        'c' => Some((CRACK[0], CRACK[1], CRACK[2], 255)),
        _ => None,
    });
    for (x, y) in extra {
        put(a, t, x, y, CRACK[0], CRACK[1], CRACK[2], 255);
    }
}

// ------------------------------------------------- item icons ----

/// heart of the sea: a deep-blue heart-shaped core (the conduit's
/// ingredient — buried-treasure loot).
pub(super) fn heart_of_the_sea_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "...HH......HH...",
        "..HHHH....HHHH..",
        "..HHHHH..HHHHH..",
        "..HHHHHHHHHHHH..",
        "..HHHHHHHHHHHH..",
        "...HHHHHHHHHH...",
        "....HHHHHHHH....",
        ".....HHHHHH.....",
        "......HHHH......",
        ".......HH.......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'H' => Some((42, 95, 175, 255)),
        _ => None,
    });
}

/// nautilus shell: a spiral shell (amber with banding).
pub(super) fn nautilus_shell_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const AMBER: [i32; 3] = [180, 160, 130];
    const DARK: [i32; 3] = [140, 120, 95];
    let rows = [
        "................",
        "................",
        "......NNNN......",
        ".....NddddN.....",
        "....Ndd..ddN....",
        "...Ndd....ddN...",
        "..Ndd......dN...",
        "..Nd.......dN...",
        "..Nd....SS..N...",
        "..Ndd...SSd.N...",
        "...Ndd.SSddN....",
        "....NddSSdN.....",
        ".....NdddN......",
        "......NNN.......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'N' => Some(((AMBER[0]), (AMBER[1]), (AMBER[2]), 255)),
        'd' => Some((DARK[0], DARK[1], DARK[2], 255)),
        'S' => Some((210, 195, 165, 255)),
        _ => None,
    });
}

/// scute: the baby-turtle scale (turtle-shell crafting).
pub(super) fn scute_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const SC: [i32; 3] = [95, 145, 90];
    let rows = [
        "................",
        "................",
        "....SSSSSSSS....",
        "...S........S...",
        "...S..SSSS..S...",
        "...S.S....S.S...",
        "...S.S..SS.S.S..",
        "....S.S..S.S....",
        "....S.S..S.S....",
        ".....S....S.....",
        ".....SSSSSS.....",
        "......SSSS......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some(((SC[0]), (SC[1]), (SC[2]), 255)),
        _ => None,
    });
}

/// trident: a three-pronged spear (the drowned's weapon).
pub(super) fn trident_art(a: &mut [u8], t: u16) {
    const MET: [i32; 3] = [140, 145, 150];
    const DARK: [i32; 3] = [95, 100, 105];
    let rows = [
        "..P..P..P.......",
        "..P.PPP.P.......",
        "..PPPMPPP.......",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        "....MMM.........",
        ".....M..........",
        ".....M..........",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((MET[0], MET[1], MET[2], 255)),
        'M' => Some((DARK[0], DARK[1], DARK[2], 255)),
        _ => None,
    });
}

/// phantom membrane: a tattered scrap of membrane.
pub(super) fn phantom_membrane_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const MEM: [i32; 3] = [130, 130, 160];
    let rows = [
        "................",
        "................",
        "...MMMMMM.......",
        "..MM....MM......",
        "..MM.....MM.....",
        "..MM......MM....",
        "..MM...M..MM....",
        "...MMMMMMMMM....",
        "..MM......MM....",
        "..MM.......MM...",
        "...MM...M..MM...",
        "...MMMMMMMMM....",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'M' => Some(((MEM[0]), (MEM[1]), (MEM[2]), 255)),
        _ => None,
    });
}

/// dried kelp: the 1-hunger food strip.
pub(super) fn dried_kelp_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const K: [i32; 3] = [40, 65, 40];
    let rows = [
        "................",
        "................",
        "................",
        "......KKK.......",
        ".....KKKKK......",
        ".....KKKKK......",
        ".....KKKKKK.....",
        "......KKKKK.....",
        "......KKKKK.....",
        ".....KKKKK......",
        ".....KKKKK......",
        "......KKK.......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some(((K[0]), (K[1]), (K[2]), 255)),
        _ => None,
    });
}

/// turtle shell: the 5-scute helmet.
pub(super) fn turtle_shell_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const TS: [i32; 3] = [95, 150, 85];
    let rows = [
        "................",
        "................",
        "................",
        ".....TTTTTT.....",
        "...TTTTTTTTTT...",
        "...TTsTTTTsTT...",
        "..TTTTTTTTTTTT..",
        "..TTTTTTTTTTTT..",
        "...TTTTTTTTTT...",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some(((TS[0]), (TS[1]), (TS[2]), 255)),
        's' => Some((TS[0] * 70 / 100, TS[1] * 70 / 100, TS[2] * 70 / 100, 255)),
        _ => None,
    });
}

// -------------------------------------------------- mob sprites ----

/// drowned: the waterlogged zombie — teal-tinged skin.
pub(super) fn drowned_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const SKIN: [i32; 3] = [70, 115, 110];
    const DARK: [i32; 3] = [50, 85, 82];
    const EYE: [i32; 3] = [200, 220, 210];
    let rows = [
        "....HHHH........",
        "...HHHHHH.......",
        "...HeHHdH.......",
        "...HHHHHH.......",
        "....HHHH........",
        "..BBBBBBBB......",
        ".BBBBBBBBBB.....",
        ".BBBBBBBBBB.....",
        ".BBdbBBbdBB.....",
        ".BBBBBBBBBB.....",
        "..BBBBBBBB......",
        "..BBBBBBBB......",
        "..BB....BB......",
        "..BB....BB......",
        "..dd....dd......",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'H' => Some(((SKIN[0]), (SKIN[1]), (SKIN[2]), 255)),
        'B' => Some(((DARK[0] + 15), (DARK[1] + 15), (DARK[2] + 15), 255)),
        'd' => Some((DARK[0], DARK[1], DARK[2], 255)),
        'e' => Some((EYE[0], EYE[1], EYE[2], 255)),
        _ => None,
    });
}

/// phantom: the insomnia manta — blue-gray wings, glowing eyes.
pub(super) fn phantom_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [90, 105, 150];
    const WING: [i32; 3] = [70, 82, 120];
    const EYE: [i32; 3] = [230, 240, 90];
    let rows = [
        "................",
        "..W..........W..",
        ".WWW........WWW.",
        ".WWWW..HH..WWWW.",
        ".WWWWWHHHHWWWWW.",
        "..WWWHHeHHWWWW..",
        "..WWWHHHHHWWWW..",
        "...WWHHHHHWWW...",
        "....HHHHHHH.....",
        ".....HHHHH......",
        ".....H...H......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some(((WING[0]), (WING[1]), (WING[2]), 255)),
        'H' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'e' => Some((EYE[0], EYE[1], EYE[2], 255)),
        _ => None,
    });
}

/// dolphin: gray-blue with a pale belly (Dolphin's Grace host).
pub(super) fn dolphin_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [100, 130, 160];
    const BELLY: [i32; 3] = [190, 215, 225];
    let rows = [
        "................",
        "................",
        "................",
        "......BBBB......",
        "....BBBBBBBB....",
        "..BBBBBBBBBBBB..",
        ".BBBBbbbbbbBBBB.",
        ".BBbbbbbbbbbbBB.",
        ".BBbbbbbbbbbbBB.",
        "..BBBBBBBBBBBBF.",
        "....FFFFFFFF..F.",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'b' => Some(((BELLY[0]), (BELLY[1]), (BELLY[2]), 255)),
        'F' => Some((BODY[0] * 75 / 100, BODY[1] * 75 / 100, BODY[2] * 75 / 100, 255)),
        _ => None,
    });
}

/// cod: the olive-brown staple fish.
pub(super) fn cod_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [140, 120, 90];
    const DARK: [i32; 3] = [100, 85, 62];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......BBBBB.....",
        "....BBBBBBBB.F..",
        "...BBBBBBBBBFF..",
        "..BDBBBBBBBBF...",
        "..BDBBBBBBBBB...",
        "...BBBBBBBB.....",
        "....BBBBB.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'D' => Some((DARK[0], DARK[1], DARK[2], 255)),
        'F' => Some((DARK[0] * 80 / 100, DARK[1] * 80 / 100, DARK[2] * 80 / 100, 255)),
        _ => None,
    });
}

/// salmon: the red-silver river fish.
pub(super) fn salmon_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [190, 105, 95];
    const BELLY: [i32; 3] = [215, 180, 170];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......BBBBB.....",
        "....BBBBBBBB.F..",
        "...BBBBBBBBBFF..",
        "..BBBBBBBBBBF...",
        "..BBBBBBBBBBB...",
        "...bBBBBBBB.....",
        "....bBBBB.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'b' => Some(((BELLY[0]), (BELLY[1]), (BELLY[2]), 255)),
        'F' => Some((150, 80, 75, 255)),
        _ => None,
    });
}

/// pufferfish: the puffed yellow ball with spikes (poisonous).
pub(super) fn pufferfish_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [220, 180, 60];
    const DARK: [i32; 3] = [110, 80, 40];
    let rows = [
        "................",
        "..S..........S..",
        "...S..SSSS..S...",
        "....SBBBBB......",
        "...SBBBBBBBS....",
        "..SBBBBBBBBBS...",
        "..BBBeBBBeBBB...",
        "..SBBBBBBBBS....",
        "...SBBBBBBS.....",
        "....SBBBBS......",
        "...S..SSS..S....",
        "..S..........S..",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'S' => Some((DARK[0], DARK[1], DARK[2], 255)),
        'e' => Some((30, 30, 30, 255)),
        _ => None,
    });
}

/// tropical fish: a small striped reef fish.
pub(super) fn tropical_fish_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const BODY: [i32; 3] = [220, 120, 60];
    const STRIPE: [i32; 3] = [240, 200, 80];
    const FIN: [i32; 3] = [180, 90, 45];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......BBBBF.....",
        "....SBBBBBFF....",
        "...BSBBBBBFF....",
        "..BBBBBBBBB.....",
        "..BSBBBBBBB.....",
        "...BBBBBBB......",
        "....BBBB........",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some(((BODY[0]), (BODY[1]), (BODY[2]), 255)),
        'S' => Some((STRIPE[0], STRIPE[1], STRIPE[2], 255)),
        'F' => Some((FIN[0], FIN[1], FIN[2], 255)),
        _ => None,
    });
}

/// turtle: the shelled beach nester (scute dropper).
pub(super) fn turtle_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    const SHELL: [i32; 3] = [80, 140, 75];
    const SKIN: [i32; 3] = [120, 180, 110];
    const DARK: [i32; 3] = [60, 110, 58];
    let rows = [
        "................",
        "................",
        "................",
        ".....KK..SSS....",
        "....KKKKSSSSS...",
        "...KKKKKKKKKK...",
        "..KKsKKKKKKsKK..",
        "..KKKKKKKKKKKK..",
        "..KsKKsKKsKKsK..",
        "...KKKKKKKKKK...",
        "....KKKKKKKK....",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some(((SKIN[0]), (SKIN[1]), (SKIN[2]), 255)),
        'K' => Some(((SHELL[0]), (SHELL[1]), (SHELL[2]), 255)),
        's' => Some((DARK[0], DARK[1], DARK[2], 255)),
        _ => None,
    });
}

// -------------------------------------------- spawn-egg palettes ----

/// 8 egg palettes (base, spots) for the V9 eggs — the E-series
/// egg-art convention (e1_art::egg_art + per-mob palettes).
pub(super) const V9_EGG_PALETTES: [[i32; 6]; 8] = [
    // drowned: teal + pale
    [70, 110, 130, 100, 150, 160],
    // phantom: slate blue + dark
    [90, 70, 130, 60, 50, 90],
    // dolphin: blue-gray + pale belly
    [90, 130, 160, 180, 220, 230],
    // cod: olive + brown
    [140, 110, 80, 90, 70, 50],
    // salmon: red + pink
    [180, 100, 80, 230, 170, 160],
    // pufferfish: yellow + dark
    [220, 180, 60, 110, 80, 40],
    // tropical fish: orange + yellow
    [220, 120, 60, 240, 200, 80],
    // turtle: green + cream
    [110, 160, 90, 230, 230, 180],
];
