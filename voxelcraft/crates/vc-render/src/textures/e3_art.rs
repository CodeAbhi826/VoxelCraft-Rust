//! Phase E3 procedural tiles (evolution 1.5–1.6 bracket) — clean-room
//! art, a child module of textures.rs (shares put/jit/noise_fill/art).
//! All pixel art is ours (distinct silhouettes/palettes); nothing is
//! extracted or recreated from Mojang assets.

use super::{art, noise_fill, put, Rng};

// ---- world blocks ----

/// Block of Coal — compressed dark fuel block: matte black with
/// anthracite glints (16000 ticks / 80 items, VERIFIED w/Block_of_Coal).
pub(super) fn coal_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [28, 28, 30], 6, rng);
    // glint seams: a few bright anthracite lines
    for y in 2..14 {
        for x in 2..14 {
            if (x + y * 3) % 11 == 0 {
                put(a, t, x, y, 70, 72, 78, 255);
            }
            if (x * 5 + y) % 17 == 0 {
                put(a, t, x, y, 18, 18, 20, 255);
            }
        }
    }
    // block border
    for i in 0..16 {
        put(a, t, i, 0, 40, 40, 44, 255);
        put(a, t, i, 15, 22, 22, 24, 255);
        put(a, t, 0, i, 40, 40, 44, 255);
        put(a, t, 15, i, 22, 22, 24, 255);
    }
}

/// Block of Quartz — smooth pale cream mineral (craft 4 nether quartz,
/// VERIFIED w/Block_of_Quartz).
pub(super) fn quartz_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [236, 233, 226], 5, rng);
    for (x, y) in [(2, 2), (13, 3), (4, 12), (12, 12), (7, 7)] {
        put(a, t, x, y, 248, 246, 240, 255);
        put(a, t, x + 1, y, 248, 246, 240, 255);
    }
    for i in 0..16 {
        put(a, t, i, 0, 244, 242, 236, 255);
        put(a, t, i, 15, 220, 217, 208, 255);
        put(a, t, 0, i, 244, 242, 236, 255);
        put(a, t, 15, i, 220, 217, 208, 255);
    }
}

/// Chiseled Quartz — carved face: a sunken diamond lattice (vanilla
/// crafts from quartz slabs, w/Chiseled_Quartz_Block — picker-only in
/// the engine, no quartz-slab model; disclosed).
pub(super) fn chiseled_quartz_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    quartz_block_art(a, t, rng);
    for i in 0..7 {
        // sunken diamond outline (top-left triangle set)
        put(a, t, 4 + i, 4 + i, 214, 210, 200, 255);
        put(a, t, 11 - i, 4 + i, 214, 210, 200, 255);
        put(a, t, 4 + i, 11 - i, 214, 210, 200, 255);
        put(a, t, 11 - i, 11 - i, 214, 210, 200, 255);
    }
    put(a, t, 7, 7, 226, 222, 214, 255);
    put(a, t, 8, 8, 226, 222, 214, 255);
}

/// Quartz Pillar side — vertical fluting; top — a ringed face (craft
/// 2 blocks → 2 pillars, VERIFIED w/Quartz_Pillar).
pub(super) fn quartz_pillar_side_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [238, 235, 228], 4, rng);
    for y in 0..16 {
        put(a, t, 3, y, 222, 219, 210, 255);
        put(a, t, 4, y, 246, 244, 238, 255);
        put(a, t, 7, y, 222, 219, 210, 255);
        put(a, t, 8, y, 246, 244, 238, 255);
        put(a, t, 11, y, 222, 219, 210, 255);
        put(a, t, 12, y, 246, 244, 238, 255);
    }
}

pub(super) fn quartz_pillar_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    quartz_block_art(a, t, rng);
    for i in 0..16 {
        put(a, t, i, 3, 214, 210, 200, 255);
        put(a, t, i, 12, 214, 210, 200, 255);
        put(a, t, 3, i, 214, 210, 200, 255);
        put(a, t, 12, i, 214, 210, 200, 255);
    }
}

/// Stained terracotta — 16 colors in the vanilla dye-color registry
/// order (VERIFIED w/Terracotta "comes in the sixteen dye colors ...");
/// clean-room palette approximations (not verbatim vanilla RGB), each
/// with the matte-clay noise + a subtle diagonal band.
pub(super) fn stained_terracotta_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    // clean-room approximation of the vanilla stained-terracotta family
    // (muted, desaturated clay tones)
    const PALETTES: [[i32; 3]; 16] = [
        [210, 178, 160], // white
        [164, 84, 38],   // orange
        [150, 106, 156], // magenta
        [108, 137, 166], // light blue
        [186, 133, 34],  // yellow
        [94, 123, 35],   // lime
        [203, 141, 150], // pink
        [84, 80, 90],    // gray
        [138, 134, 140], // light gray
        [72, 114, 122],  // cyan
        [122, 78, 140],  // purple
        [75, 86, 134],   // blue
        [110, 74, 50],   // brown
        [88, 108, 62],   // green
        [142, 60, 52],   // red
        [52, 44, 50],    // black
    ];
    let base = PALETTES[(color as usize) & 15];
    noise_fill(a, t, base, 7, rng);
    // faint diagonal clay band (the pressed-clay look)
    for y in 0..16 {
        for x in 0..16 {
            if (x + y) % 16 == 7 {
                let (r, g, b) = (base[0] * 92 / 100, base[1] * 92 / 100, base[2] * 92 / 100);
                put(a, t, x, y, r, g, b, 255);
            }
        }
    }
    for i in 0..16 {
        let (r, g, b) = (base[0] * 110 / 100, base[1] * 110 / 100, base[2] * 110 / 100);
        put(a, t, i, 0, r.min(255), g.min(255), b.min(255), 255);
    }
}

/// Hay Bale top — golden circular bale rings; side — horizontal straw
/// bands with stitching (fall damage −80%, VERIFIED w/Hay_Bale).
pub(super) fn hay_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [172, 140, 64], 10, rng);
    for r in 2..8 {
        let rr = r * r;
        for y in 0..16 {
            for x in 0..16 {
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                let d = dx * dx + dy * dy;
                if (d - rr).abs() < 2 {
                    put(a, t, x, y, 148, 116, 48, 255);
                }
            }
        }
    }
    for (x, y) in [(8, 8), (7, 8), (8, 7), (9, 8), (8, 9)] {
        put(a, t, x, y, 196, 164, 88, 255);
    }
    rng.next_f32(); // keep the rng stream stable across calls
}

pub(super) fn hay_side_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [176, 144, 66], 9, rng);
    // two horizontal straw bands
    for x in 0..16 {
        for y in 2..6 {
            put(a, t, x, y, 188, 156, 78, 255);
        }
        for y in 9..13 {
            put(a, t, x, y, 160, 128, 52, 255);
        }
    }
    // baling-twine stitches crossing the bands
    for y in 0..16 {
        put(a, t, 4, y, 120, 96, 40, 255);
        put(a, t, 11, y, 120, 96, 40, 255);
    }
}

/// Daylight Sensor — top: a grid of dark-blue sensor cells on a stone
/// bezel; side: a low wooden base (recipe glass + quartz + wooden slab,
/// VERIFIED w/Daylight_Detector).
pub(super) fn daylight_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [126, 128, 130], 6, rng);
    for cy in 0..4 {
        for cx in 0..4 {
            for y in 0..2 {
                for x in 0..2 {
                    let px = 2 + cx * 3 + x;
                    let py = 2 + cy * 3 + y;
                    put(a, t, px, py, 42, 58, 102, 255);
                }
            }
        }
    }
    // one highlighted cell (the sensor reading the sky)
    put(a, t, 8, 5, 96, 130, 200, 255);
    put(a, t, 9, 5, 96, 130, 200, 255);
    rng.next_f32();
}

pub(super) fn daylight_side_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "SSSSSSSSSSSSSSSS",
        "SSSSSSSSSSSSSSSS",
        "WWWWWWWWWWWWWWWW",
        "WWWWWWWWWWWWWWWW",
        "WWWWWWWWWWWWWWWW",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'S' => Some((112, 114, 116, 255)),
        'W' => Some((156, 122, 74, 255)),
        _ => None,
    });
}

/// Light (gold) / Heavy (iron) Weighted Pressure Plates — a flat plate
/// with a metallic sheen; the side view shows the 1/16-block-thick lip.
/// Signals: light = entities, heavy = ceil(entities/10) (both VERIFIED).
pub(super) fn plate_art(a: &mut [u8], t: u16, gold: bool) {
    let (hi, mid_r, mid_g, mid_b): (i32, i32, i32, i32) = if gold {
        (238, 206, 158, 62)
    } else {
        (200, 160, 160, 166)
    };
    let (lo_r, lo_g, lo_b): (i32, i32, i32) = if gold { (170, 126, 44) } else { (128, 128, 134) };
    // flat plate filling most of the tile (pressure-plate proportions)
    for y in 3..13 {
        for x in 2..14 {
            put(a, t, x, y, mid_r, mid_g, mid_b, 255);
        }
    }
    // top highlight + shadow rows
    for x in 2..14 {
        put(a, t, x, 3, hi, hi, hi, 255);
        put(a, t, x, 12, lo_r, lo_g, lo_b, 255);
    }
    // center rivet
    put(a, t, 7, 7, hi, hi, hi, 255);
    put(a, t, 8, 8, lo_r, lo_g, lo_b, 255);
}

/// Block of Redstone — deep red mineral block with a wire-lattice motif
/// (always-on weak power 15, VERIFIED w/Block_of_Redstone).
pub(super) fn redstone_block_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [146, 22, 22], 14, rng);
    // lattice of darker seams
    for i in 0..16 {
        put(a, t, i, 5, 104, 12, 12, 255);
        put(a, t, i, 10, 104, 12, 12, 255);
        put(a, t, 5, i, 104, 12, 12, 255);
        put(a, t, 10, i, 104, 12, 12, 255);
    }
    // bright redstone specks
    for (x, y) in [(3, 3), (12, 4), (7, 8), (4, 12), (13, 12)] {
        put(a, t, x, y, 220, 40, 40, 255);
    }
}

// ---- items ----

/// Nether Quartz — pale crystal shard (ore drop, VERIFIED
/// w/Nether_Quartz_Ore "it drops 1 Nether quartz").
pub(super) fn nether_quartz_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "......Q.........",
        ".....QQQ........",
        ".....QQQQ.......",
        "....QQQQQ.......",
        "....QQQQQQ......",
        "....QQQQQ.......",
        ".....QQQQ.......",
        ".....QQQ........",
        "......QQ........",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'Q' => Some((238, 234, 226, 255)),
        _ => None,
    });
}

/// Lead — a rope loop with a knot (leash item; 1.16.5 stretch max 10
/// blocks — VERIFIED w/Lead, version-scoped vs the 2025 12-block buff).
pub(super) fn lead_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "....RRRRRR......",
        "...R......R.....",
        "..R........R....",
        "..R........R....",
        "..R........R....",
        "..R........R....",
        "...R......R.....",
        "....RRRRRR......",
        "......RR........",
        "......RR........",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'R' => Some((160, 124, 66, 255)),
        _ => None,
    });
}

/// Saddle — brown seat with a horn and strap (not craftable in vanilla;
/// dungeon loot + picker — VERIFIED w/Horse/w/Riding).
pub(super) fn saddle_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "......LL........",
        ".....LLLL.......",
        "..LLLLLLLLLL....",
        ".LBBBBBBBBBBL...",
        ".LBBBBBBBBBBL...",
        ".LLBBBBBBBLLL...",
        "...LLLBBLLLL....",
        "....LLLLLL......",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'L' => Some((120, 82, 38, 255)),
        'B' => Some((162, 112, 54, 255)),
        _ => None,
    });
}

// ---- mob sprites (billboards) ----

/// Horse — chestnut quadruped with a dark mane (7 base colors exist in
/// vanilla; one representative clean-room coat, variant via tint at
/// spawn). Stats VERIFIED w/Horse: health 15–30, speed 0.1125–0.3375
/// internal (≈4.86–14.57 b/s), jump strength 0.4–1.0.
pub(super) fn horse_art(a: &mut [u8], t: u16, body: [i32; 3], mane: [i32; 3]) {
    let rows = [
        "................",
        "................",
        "......MM........",
        ".....MMMM.......",
        "..BBBBBBMMM.....",
        ".BBBBBBBBBB.....",
        ".BBBBBBBBBB.....",
        "BBBBBBBBBBBB....",
        ".BBBBBBBBBB.....",
        ".BB..BB..BB.....",
        ".BB..BB..BB.....",
        "HH...HH...HH....",
        "HH...HH...HH....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'B' => Some((body[0], body[1], body[2], 255)),
        'M' => Some((mane[0], mane[1], mane[2], 255)),
        'H' => Some((64, 46, 30, 255)),
        _ => None,
    });
}

/// Donkey — grey-brown coat, long ears, smaller frame (15–30 HP, speed
/// 0.175 fixed — VERIFIED w/Donkey).
pub(super) fn donkey_art(a: &mut [u8], t: u16) {
    horse_art(a, t, [124, 102, 78], [70, 58, 44]);
    // long ears
    put(a, t, 7, 1, 124, 102, 78, 255);
    put(a, t, 6, 0, 124, 102, 78, 255);
    put(a, t, 9, 1, 124, 102, 78, 255);
    put(a, t, 10, 0, 124, 102, 78, 255);
}

/// Mule — horse×donkey hybrid: dark bay coat + long ears (no natural
/// spawns — breeding only, VERIFIED w/Mule).
pub(super) fn mule_art(a: &mut [u8], t: u16) {
    horse_art(a, t, [84, 60, 42], [36, 28, 22]);
    put(a, t, 7, 1, 84, 60, 42, 255);
    put(a, t, 6, 0, 84, 60, 42, 255);
    put(a, t, 9, 1, 84, 60, 42, 255);
    put(a, t, 10, 0, 84, 60, 42, 255);
}

/// E3 spawn-egg palettes (horse / donkey / mule — kinds 20..=22; the
/// E1/E2 egg_art renders the shell + spots from these pairs).
pub const E3_EGG_PALETTES: [(i32, i32, i32, i32, i32, i32); 3] = [
    (140, 96, 58, 240, 234, 226),  // horse: chestnut + cream spots
    (110, 92, 72, 200, 190, 170),  // donkey: grey-brown + light muzzle
    (70, 52, 40, 160, 130, 100),   // mule: dark bay + tan spots
];
