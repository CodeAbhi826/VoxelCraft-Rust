//! Phase E2 procedural tiles (evolution 1.3–1.4 bracket) — clean-room
//! art, a child module of textures.rs (shares put/jit/noise_fill/art).
//! All pixel art is ours (distinct silhouettes/palettes); nothing is
//! extracted or recreated from Mojang assets.

use super::{art, jit, noise_fill, put, Rng};

// ---- world blocks ----

/// Anvil — grey iron body. The three damage stages share the silhouette;
/// chipped adds a crack line, damaged removes a corner (VERIFIED w/Anvil:
/// "gradually becomes a chipped anvil, then a damaged anvil").
pub(super) fn anvil_art(a: &mut [u8], t: u16, damage: u8) {
    let rows = [
        "................",
        "..IIIIIIIIIIII..",
        "..IIIIIIIIIIII..",
        "................",
        ".......II.......",
        ".......II.......",
        ".......II.......",
        ".......II.......",
        "......IIII......",
        "....IIIIIIII....",
        "..IIIIIIIIIIII..",
        "..IIIIIIIIIIII..",
        "..IIIIIIIIIIII..",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'I' => Some((96, 96, 100, 255)),
        _ => None,
    });
    match damage {
        1 => {
            // chipped: a diagonal crack through the top
            for i in 0..6 {
                put(a, t, 11 - i, 1 + i, 52, 52, 56, 255);
                put(a, t, 12 - i, 1 + i, 52, 52, 56, 255);
            }
        }
        2 => {
            // damaged: crack + missing corner blocks of the face
            for i in 0..6 {
                put(a, t, 11 - i, 1 + i, 52, 52, 56, 255);
            }
            for y in 0..3 {
                for x in 0..4 {
                    put(a, t, 2 + x, 10 + y, 0, 0, 0, 0);
                }
            }
        }
        _ => {}
    }
}

/// Beacon — glassy core with an iron frame and a warm center (light 15,
/// VERIFIED w/Beacon; emits light even unpowered).
pub(super) fn beacon_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let frame = [120, 122, 130];
    noise_fill(a, t, frame, 8, rng);
    // glassy inner square
    for y in 3..13 {
        for x in 3..13 {
            put(a, t, x, y, 200, 225, 240, 190);
        }
    }
    // warm core glow
    for y in 6..10 {
        for x in 6..10 {
            put(a, t, x, y, 255, 236, 170, 255);
        }
    }
    // corner rivets
    for (x, y) in [(3, 3), (12, 3), (3, 12), (12, 12)] {
        put(a, t, x, y, 90, 92, 100, 255);
    }
}

/// Beacon beam — the vertical light column rendered as a bright
/// translucent streak (billboard quads stack this tile).
pub(super) fn beacon_beam_art(a: &mut [u8], t: u16) {
    for y in 0..16 {
        for x in 0..16 {
            let d = ((x - 8) as i32).abs();
            let a_ = if d < 2 { 210 } else if d < 4 { 130 } else { 60 };
            put(a, t, x, y, 240, 250, 255, a_);
        }
    }
    // inner hot line
    for y in 0..16 {
        put(a, t, 7, y, 255, 255, 255, 235);
        put(a, t, 8, y, 255, 255, 255, 235);
    }
}

/// Cobblestone wall — the shared cobble field; mesh connections come
/// from the fence-style path (single visual tile, VERIFIED w/Wall: 6
/// cobble → 6 walls, 1.5-block-tall boundary).
pub(super) fn cobble_wall_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [124, 124, 124], 16, rng);
    // mortar cracks: darker lattice
    for _ in 0..26 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 84, 84, 86, 255);
    }
    // a few pale highlights
    for _ in 0..10 {
        let x = (rng.next_range(16)) as i32;
        let y = (rng.next_range(16)) as i32;
        put(a, t, x, y, 158, 158, 160, 255);
    }
}

/// Ender chest — dark obsidian body with a glowing teal center eye
/// (light 7, VERIFIED w/Ender_Chest).
pub(super) fn ender_chest_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let body = [28, 24, 40];
    noise_fill(a, t, body, 10, rng);
    // lid seam
    for x in 1..15 {
        put(a, t, x, 5, 16, 14, 24, 255);
    }
    // border
    for i in 0..16 {
        put(a, t, i, 0, 42, 38, 58, 255);
        put(a, t, i, 15, 42, 38, 58, 255);
        put(a, t, 0, i, 42, 38, 58, 255);
        put(a, t, 15, i, 42, 38, 58, 255);
    }
    // teal center eye
    for y in 6..9 {
        for x in 6..9 {
            put(a, t, x, y, 40, 190, 170, 255);
        }
    }
    put(a, t, 7, 7, 130, 255, 240, 255);
}

/// Flower pot — terracotta pot sprite (cross-rendered; hardness 0,
/// VERIFIED w/Flower_Pot).
pub(super) fn flower_pot_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "...TTTTTTTTT....",
        "...T.......T....",
        "....T.....T.....",
        "....TTTTTTT.....",
        "....T.....T.....",
        "....T.....T.....",
        "...TTTTTTTTT....",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'T' => Some((150, 92, 68, 255)),
        _ => None,
    });
    // rim shading
    for x in 3..12 {
        put(a, t, x, 5, 120, 70, 52, 255);
    }
}

/// Item frame — dark-wood frame with a pale face (cross-rendered sprite;
/// the displayed item blits over the face at draw time).
pub(super) fn item_frame_art(a: &mut [u8], t: u16) {
    let rows = [
        "FFFFFFFFFFFFFFFF",
        "FWWWWWWWWWWWWWWF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FW............WF",
        "FWWWWWWWWWWWWWF",
        "FFFFFFFFFFFFFFFF",
    ];
    art(a, t, rows, &|c| match c {
        'F' => Some((96, 70, 44, 255)),
        'W' => Some((160, 128, 92, 255)),
        _ => None,
    });
}

/// Tripwire hook — a small wall hook with a wheel (off: grey wood; on:
/// the wheel glows red — powered state, w/Tripwire_Hook).
pub(super) fn tripwire_hook_art(a: &mut [u8], t: u16, powered: bool) {
    let rows = [
        "................",
        "................",
        "......HHH.......",
        "......HKH.......",
        "......HHH.......",
        ".....W...W......",
        "....W.....W.....",
        "....W.....W.....",
        "....W.....W.....",
        ".....W...W......",
        "......WWW.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    let hot = if powered { (240, 60, 40, 255) } else { (120, 90, 60, 255) };
    art(a, t, rows, &|c| match c {
        'H' => Some((140, 106, 70, 255)),
        'K' => Some(hot),
        'W' => Some((160, 128, 92, 255)),
        _ => None,
    });
}

/// Wither skeleton skull — the blackened skull (summon component + 2.5%
/// drop, VERIFIED w/Wither_Skeleton).
pub(super) fn wither_skull_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....KKKKKKKK....",
        "...KKKKKKKKKK...",
        "...KKWKKKKWKK...",
        "...KKKKKKKKKK...",
        "...KKKKKKKKKK...",
        "....KKKKKKKK....",
        "....K.K.K.K.K...",
        "....K.K.K.K.K...",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((38, 34, 36, 255)),
        'W' => Some((200, 60, 50, 255)),
        _ => None,
    });
}

/// Command block — grey-violet block with a chevron motif; the ON
/// variant brightens the motif (VERIFIED w/Command_Block: not craftable,
/// executes commands when powered).
pub(super) fn command_block_art(a: &mut [u8], t: u16, on: bool) {
    let base = [138, 132, 148];
    let body = if on { [168, 150, 170] } else { base };
    for y in 0..16 {
        for x in 0..16 {
            let v = jit(-8, 8, &mut Rng::new(t as u64 * 31 + y as u64 * 7 + x as u64));
            put(a, t, x, y, (body[0] + v).clamp(0, 255), (body[1] + v).clamp(0, 255), (body[2] + v).clamp(0, 255), 255);
        }
    }
    let hot = if on { (255, 220, 90, 255) } else { (214, 210, 80, 255) };
    // chevron motif
    for i in 0..5 {
        put(a, t, 4 + i, 10 - i, hot.0, hot.1, hot.2, 255);
        put(a, t, 11 - i, 6 + i, hot.0, hot.1, hot.2, 255);
    }
    // border pins
    for i in 0..16 {
        put(a, t, i, 0, 110, 104, 116, 255);
        put(a, t, i, 15, 110, 104, 116, 255);
        put(a, t, 0, i, 110, 104, 116, 255);
        put(a, t, 15, i, 110, 104, 116, 255);
    }
}

// ---- item icons ----

/// Emerald — green gem with facet highlights (beacon feed + currency,
/// VERIFIED w/Emerald).
pub(super) fn emerald_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".....GGGGG......",
        "....GGGGGGG.....",
        "...GGLGGGGGG....",
        "...GGGGGGGGG....",
        "...GGGGGGGGG....",
        "...GGGGGDDGG....",
        "....GGDDGGG.....",
        ".....GGGGG......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((30, 170, 90, 255)),
        'L' => Some((160, 240, 190, 255)),
        'D' => Some((16, 110, 60, 255)),
        _ => None,
    });
}

/// Nether star — pale four-point star with a warm core (the wither's
/// drop, VERIFIED w/Wither).
pub(super) fn nether_star_art(a: &mut [u8], t: u16) {
    for y in 0..16 {
        for x in 0..16 {
            let dx = (x as i32 - 8).abs();
            let dy = (y as i32 - 8).abs();
            // four-point star: |dx|+|dy| small on the diagonals
            let m = dx + dy;
            if m < 3 {
                put(a, t, x, y, 255, 252, 240, 255);
            } else if m < 5 && dx.max(dy) < 5 {
                put(a, t, x, y, 232, 224, 200, 255);
            }
        }
    }
    // warm core
    put(a, t, 8, 8, 255, 220, 120, 255);
    put(a, t, 7, 8, 255, 220, 120, 255);
    put(a, t, 8, 7, 255, 220, 120, 255);
    put(a, t, 7, 7, 255, 220, 120, 255);
}

/// Potato — tan oval with eyes (food 1/0.6, VERIFIED w/Food).
pub(super) fn potato_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "....PPPPPP......",
        "...PPPPPPPP.....",
        "..PPP.PPPPPP....",
        "..PPPPPPPPPP....",
        "..PPPPPP.PPP....",
        "..PPPPPPPPPP....",
        "...PPPPPPPP.....",
        "....PPPPPP......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((198, 168, 106, 255)),
        _ => None,
    });
    // eyes (darker spots)
    put(a, t, 5, 5, 150, 118, 70, 255);
    put(a, t, 10, 8, 150, 118, 70, 255);
    put(a, t, 7, 9, 150, 118, 70, 255);
}

/// Baked potato — roasted: darker skin with a butter pat (food 5/6.0,
/// VERIFIED w/Food).
pub(super) fn baked_potato_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "....BBBBBB......",
        "...BBBBBBBB.....",
        "..BB.BBBBBBB....",
        "..BBBBBBBBBB....",
        "..BBBBB.BBBB....",
        "..BBBBBBBBBB....",
        "...BBBBBBBB.....",
        "....BBBBBB......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((160, 108, 58, 255)),
        _ => None,
    });
    // butter pat
    for y in 6..8 {
        for x in 7..10 {
            put(a, t, x, y, 240, 220, 90, 255);
        }
    }
}

/// Carrot — orange tap root with a green top (food 3/3.6, VERIFIED
/// w/Food).
pub(super) fn carrot_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "......GG.G......",
        ".....G.GG.G.....",
        ".....GG.GG......",
        "......GOO.......",
        "......OOO.......",
        ".....OOOO.......",
        ".....OOO........",
        ".....OOOO.......",
        "......OOO.......",
        "......OOO.......",
        ".......OO.......",
    ];
    let mut rows2: [&str; 16] = ["................"; 16];
    for (i, r) in rows.iter().enumerate() {
        rows2[i] = r;
    }
    art(a, t, rows2, &|c| match c {
        'G' => Some((70, 160, 60, 255)),
        'O' => Some((230, 120, 30, 255)),
        _ => None,
    });
}

/// Pumpkin pie — tan pie with a crust rim and a dollop (food 8/4.8,
/// VERIFIED w/Food).
pub(super) fn pumpkin_pie_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        ".....CCCCCC.....",
        "...CCCCCCCCCC...",
        "..CCCWWWWWCCCC..",
        ".CCCWWWWWWWCCC..",
        ".CCCWWWWWWWCCC..",
        ".CCCCCCCCCCCCC..",
        ".CCCCCCCCCCCCC..",
        "..CCCCCCCCCCC...",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'C' => Some((200, 150, 90, 255)),
        'W' => Some((226, 190, 140, 255)),
        _ => None,
    });
}

// ---- mob sprites (billboards; clean-room, ours) ----

/// The Wither — three-headed dark boss (VERIFIED w/Wither: 300 HP, 3.5
/// blocks tall). Engine adaptation: single sprite carrying the
/// three-head silhouette.
pub(super) fn wither_art(a: &mut [u8], t: u16) {
    let rows = [
        "K...........K..",
        "KKK.......KKK..",
        ".KKK.....KKK...",
        "KKKKK...KKKKK..",
        "..KKKKKKKKK....",
        "..KKKKKKKKK....",
        "KKKDDDDDDDKKK..",
        "KKKDWDWDWDKKK..",
        "KKKDDDDDDDKKK..",
        "..KKKKKKKKK....",
        "..KK.KKK.KK....",
        ".....KKK.......",
        ".....KKK.......",
        "....KKKKK......",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((30, 26, 32, 255)),
        'D' => Some((48, 42, 50, 255)),
        'W' => Some((220, 60, 60, 255)),
        _ => None,
    });
}

/// Wither Skeleton — tall blackened skeleton with a stone sword
/// (VERIFIED w/Wither_Skeleton: 20 HP, 2.4 tall, stone sword).
pub(super) fn wither_skeleton_art(a: &mut [u8], t: u16) {
    let rows = [
        "....KKKK....",
        "....KRRK....",
        "....KKKK....",
        ".....KK.....",
        "..S..KK..S..",
        "..S..KK..S..",
        "..S.KKKK.S..",
        "....KKKK....",
        "....K..K....",
        "....K..K....",
        "....K..K....",
        "...KK..KK...",
    ];
    let mut rows2: [&str; 16] = ["................"; 16];
    let mut owned: Vec<String> = Vec::with_capacity(rows.len());
    for r in rows.iter() {
        let mut line = "................".to_string();
        for (x, ch) in r.chars().enumerate().take(12) {
            line.replace_range(2 + x..3 + x, &ch.to_string());
        }
        owned.push(line);
    }
    for (i, line) in owned.iter().enumerate() {
        rows2[i + 2] = line.as_str();
    }
    art(a, t, rows2, &|c| match c {
        'K' => Some((44, 40, 42, 255)),
        'R' => Some((190, 60, 50, 255)),
        'S' => Some((150, 150, 155, 255)),
        _ => None,
    });
}

/// Witch — purple hat, green skin, big nose (VERIFIED w/Witch: 26 HP,
/// potion thrower).
pub(super) fn witch_art(a: &mut [u8], t: u16) {
    let rows = [
        ".......PP.......",
        "......PPPP......",
        ".....PPPPPP.....",
        "....PPPPPPPP....",
        "...PPPPPPPPPP...",
        "..PPPPPPPPPPPP..",
        "....GGGGGGGG....",
        "....GWGGGGWG....",
        "....GGGNNGG.....",
        "....GGGGGGG.....",
        ".....GGGGG......",
        "....RRRRRRR.....",
        "....R.....R.....",
        "....GG...GG.....",
        "....GG...GG.....",
        "...GGG...GGG....",
    ];
    art(a, t, rows, &|c| match c {
        'P' => Some((60, 40, 90, 255)),
        'G' => Some((70, 120, 70, 255)),
        'W' => Some((220, 220, 210, 255)),
        'N' => Some((110, 160, 100, 255)),
        'R' => Some((90, 60, 120, 255)),
        _ => None,
    });
}

/// Bat — small brown bat in flight (VERIFIED w/Bat: 6 HP, ambient,
/// 0.9×0.5 hitbox).
pub(super) fn bat_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "..K.........K...",
        ".KKK.......KKK..",
        "KKKKK.....KKKKK.",
        "KKKKKKKKKKKKKKK.",
        ".KKKKKBBBKKKKK..",
        "..KKKBBBBBKKK...",
        "....KBWBKBK.....",
        ".....BBBBB......",
        "......BBB.......",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((74, 56, 44, 255)),
        'B' => Some((96, 72, 58, 255)),
        'W' => Some((220, 200, 180, 255)),
        _ => None,
    });
}

/// Wither skull projectile — the black skull bomb the wither fires
/// (8 HP + Wither II on Normal, VERIFIED w/Wither).
pub(super) fn wither_skull_proj_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "....KKKKKKKK....",
        "...KKKKKKKKKK...",
        "...KKWKKKKWKK...",
        "...KKKKKKKKKK...",
        "....KKKKKKKK....",
        "....K.K.K.K.K...",
        "....K.K.K.K.K...",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'K' => Some((28, 24, 30, 255)),
        'W' => Some((120, 200, 255, 255)),
        _ => None,
    });
}

/// Phase E2 egg palettes (kinds 17..=20: wither skeleton, witch, bat,
/// wither) — appended to the E1 16-entry table at the call site.
pub const E2_EGG_PALETTES: [(i32, i32, i32, i32, i32, i32); 4] = [
    (44, 40, 42, 190, 60, 50),   // wither skeleton: black + red eyes
    (60, 40, 90, 70, 120, 70),   // witch: purple hat + green skin
    (96, 72, 58, 74, 56, 44),    // bat: brown + dark wings
    (30, 26, 32, 220, 60, 60),   // wither: black + red
];
