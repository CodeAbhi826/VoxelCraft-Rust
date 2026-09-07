//! 1.12 bracket (World of Color Update) procedural tiles — concrete +
//! concrete powder + glazed terracotta (with pixel-rotated facing
//! variants), the 16 dye palette items, 4 seed items, the cookie, the
//! parrot egg + 5 parrot variant sprites + the illusioner sprite.
//! Clean-room art (no Mojang assets), a child module of textures.rs
//! (shares put/jit/noise_fill/art helpers). All row/palette values are
//! live-verified where the wiki publishes them (see
//! docs/research/phase-v112-1.12-research.md); the per-color RGB tables
//! are clean-room approximations — the wiki publishes no per-color hex
//! table for these families (the 1.6-era terracotta precedent).

use super::{art, jit, noise_fill, put, Rng};

/// clean-room concrete palette — saturated, evenly-lit solid colors
/// (the observable: "This update introduced a more vibrant color
/// palette ... like glazed terracotta, concrete" — changelog §1)
const CONCRETE: [[i32; 3]; 16] = [
    [207, 213, 214], // white
    [240, 118, 19],  // orange
    [189, 68, 179],  // magenta
    [90, 145, 201],  // light blue
    [248, 198, 39],  // yellow
    [112, 185, 25],  // lime
    [237, 141, 172], // pink
    [63, 68, 81],    // gray
    [125, 125, 115], // light gray
    [21, 137, 145],  // cyan
    [121, 42, 172],  // purple
    [53, 57, 157],   // blue
    [97, 76, 69],    // brown
    [89, 109, 23],   // green
    [142, 33, 33],   // red
    [25, 26, 26],    // black
];

/// concrete: a smooth near-flat colored cube (hardness 1.8, VERIFIED
/// w/Concrete) — subtle per-pixel jitter, no pattern (vanilla's concrete
/// is patternless).
pub(super) fn concrete_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    noise_fill(a, t, CONCRETE[(color as usize) & 15], 4, rng);
}

/// concrete powder: the same colors, grainy — a coarse two-tone speckle
/// (the sand+gravel aggregate look; "Craftable using 4 sand, 4 gravel
/// and one of any dye" — VERIFIED changelog §Blocks).
pub(super) fn concrete_powder_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CONCRETE[(color as usize) & 15];
    for x in 0..16 {
        for y in 0..16 {
            // half the pixels darken 18% (the gravel flecks), the rest
            // stay near base — coarser jitter than concrete
            let dark = rng.next_range(2) == 0;
            let k = if dark { 82 } else { 100 };
            put(
                a,
                t,
                x,
                y,
                jit(base[0] * k / 100, 6, rng),
                jit(base[1] * k / 100, 6, rng),
                jit(base[2] * k / 100, 6, rng),
                255,
            );
        }
    }
}

/// copy a painted 16×16 tile into `dst`, rotated by `quarter` × 90°
/// clockwise. Used for the glazed-terracotta facing variants (the base
/// tile must already be painted — the atlas loop paints ascending ids,
/// and the base (rotation 0) sits at the lowest id of each 4-tile group).
pub(super) fn rotated_copy(a: &mut [u8], src: u16, dst: u16, quarter: u8) {
    let stx = (src % 32) as i32;
    let sty = (src / 32) as i32;
    let dtx = (dst % 32) as i32;
    let dty = (dst / 32) as i32;
    for y in 0..16i32 {
        for x in 0..16i32 {
            // rotate the SOURCE coordinate so the dst grid shows the
            // image turned by quarter × 90° clockwise
            let (sx, sy) = match quarter & 3 {
                1 => (y, 15 - x),
                2 => (15 - x, 15 - y),
                3 => (15 - y, x),
                _ => (x, y),
            };
            let si = (((sty * 16 + sy) as usize) * 512 + (stx * 16 + sx) as usize) * 4;
            let di = (((dty * 16 + y) as usize) * 512 + (dtx * 16 + x) as usize) * 4;
            a[di] = a[si];
            a[di + 1] = a[si + 1];
            a[di + 2] = a[si + 2];
            a[di + 3] = a[si + 3];
        }
    }
}

/// glazed terracotta TOP face: a rotationally-ASYMMETRIC swirl (the
/// rotation must be visible — "the texture rotates relative to the
/// direction the player is facing", VERIFIED w/Glazed_Terracotta
/// §Placement; 2×2 same-color placements in mixed facings tile into a
/// larger mosaic, the changelog's "repeating pattern" observable).
/// The three rotation copies are made by rotated_copy.
pub(super) fn glazed_top_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CONCRETE[(color as usize) & 15];
    let (hi, lo): ([i32; 3], [i32; 3]) = (
        [
            (base[0] * 135 / 100).min(255),
            (base[1] * 135 / 100).min(255),
            (base[2] * 135 / 100).min(255),
        ],
        [base[0] * 62 / 100, base[1] * 62 / 100, base[2] * 62 / 100],
    );
    // glaze field: near-glossy with fine jitter
    noise_fill(a, t, base, 5, rng);
    // an off-center spiral: quarter arcs + the offset nucleus
    for y in 0..16i32 {
        for x in 0..16i32 {
            let r = ((x - 7) * (x - 7) + (y - 7) * (y - 7)) as f32;
            let r = r.sqrt();
            let ang = ((y - 7) as f32).atan2((x - 7) as f32);
            // three arc bands at growing radii, each offset by a phase
            // (spiral — NOT 4-fold symmetric, so a 90° turn is visible)
            let on_arc = (r - 3.0).abs() < 0.9
                || (r - 6.0).abs() < 0.9 && (ang.cos() + ang.sin()) < 0.4
                || (r - 9.5).abs() < 0.9 && ang.sin() < 0.0;
            if on_arc {
                let c = lo;
                put(a, t, x, y, jit(c[0], 5, rng), jit(c[1], 5, rng), jit(c[2], 5, rng), 255);
            }
            // nucleus: bright dot offset toward +x/+y (asymmetry pin)
            if r < 1.4 {
                let c = hi;
                put(a, t, x, y, jit(c[0], 4, rng), jit(c[1], 4, rng), jit(c[2], 4, rng), 255);
            }
        }
    }
    // two highlight ticks on the +x arm (reads as the swirl tail)
    for y in [8, 10] {
        for x in 10..13 {
            let c = if (x + y) % 2 == 0 { hi } else { base };
            put(a, t, x, y, c[0], c[1], c[2], 255);
        }
    }
}

/// glazed terracotta BOTTOM face: the inverse swirl (mirrored motif —
/// distinct from the top, like vanilla's top/bottom pair).
pub(super) fn glazed_bottom_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CONCRETE[(color as usize) & 15];
    let hi = [
        (base[0] * 130 / 100).min(255),
        (base[1] * 130 / 100).min(255),
        (base[2] * 130 / 100).min(255),
    ];
    let lo = [base[0] * 66 / 100, base[1] * 66 / 100, base[2] * 66 / 100];
    noise_fill(a, t, base, 5, rng);
    // corner brackets (open toward -x/-y — asymmetric)
    for i in 2..7i32 {
        for &(x, y) in &[(i, 2), (i, 13), (2, i), (13, i)] {
            let c = lo;
            put(a, t, x, y, c[0], c[1], c[2], 255);
        }
    }
    // the nucleus offset toward -x/-y
    for y in 5..8 {
        for x in 5..8 {
            let c = hi;
            put(a, t, x, y, c[0], c[1], c[2], 255);
        }
    }
}

/// glazed terracotta SIDE faces (one shared tile per color — see the
/// TILE_GLAZED_SIDE_BASE disclosure): horizontal glaze bands with an
/// offset seam.
pub(super) fn glazed_side_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CONCRETE[(color as usize) & 15];
    let lo = [base[0] * 70 / 100, base[1] * 70 / 100, base[2] * 70 / 100];
    noise_fill(a, t, base, 4, rng);
    for y in [2, 3, 8, 9, 13] {
        for x in 0..16 {
            let c = if (x + y) % 4 == 0 { base } else { lo };
            put(a, t, x, y, jit(c[0], 3, rng), jit(c[1], 3, rng), jit(c[2], 3, rng), 255);
        }
    }
}

/// dye item icon (16 colors): a small dye pile — a mound of powder with
/// a highlight. The item NAMES are the 1.12 forms (pre-1.14 renames).
pub(super) fn dye_art(a: &mut [u8], t: u16, color: u8, rng: &mut Rng) {
    let base = CONCRETE[(color as usize) & 15];
    let hi = [
        (base[0] * 140 / 100).min(255),
        (base[1] * 140 / 100).min(255),
        (base[2] * 140 / 100).min(255),
    ];
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "......hhh.......",
        ".....hbbbh......",
        "....hbbbbbh.....",
        "...hbbbbbbbh....",
        "..hbbbbbbbbbh...",
        ".hbbbbbbbbbbbh..",
        ".hbbbbbbbbbbbh..",
        "..bbbbbbbbbb....",
        "................",
        "................",
        "................",
        "................",
    ];
    let b = base;
    art(a, t, rows, &|c| match c {
        'b' => Some((b[0], b[1], b[2], 255)),
        'h' => Some((hi[0], hi[1], hi[2], 255)),
        _ => None,
    });
    let _ = rng;
}

/// seeds item icons: wheat (pale tan), melon (dark-rind teardrop),
/// pumpkin (cream), beetroot (mahogany) — the 1.12 taming set.
pub(super) fn seeds_art(a: &mut [u8], t: u16, kind: u8) {
    let (body, tip) = match kind & 3 {
        0 => ((219, 187, 78), (188, 152, 48)),    // wheat seeds
        1 => ((94, 130, 45), (66, 96, 30)),       // melon seeds
        2 => ((232, 214, 148), (204, 180, 104)),  // pumpkin seeds
        _ => ((150, 70, 58), (118, 50, 40)),      // beetroot seeds
    };
    let rows = [
        "................",
        "................",
        "................",
        "................",
        ".....b..b.......",
        "....b.bb.b......",
        ".....bbbb.......",
        "......bb........",
        "......bb........",
        "......bb........",
        ".....tbbt.......",
        "....tt..tt......",
        "................",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'b' => Some((body.0, body.1, body.2, 255)),
        't' => Some((tip.0, tip.1, tip.2, 255)),
        _ => None,
    });
}

/// cookie item icon: a round biscuit with darker chip freckles (the
/// toxic parrot food — VERIFIED w/Parrot: a fed cookie deals
/// 3.4 × 10^38 damage).
pub(super) fn cookie_art(a: &mut [u8], t: u16) {
    let rows = [
        "................",
        "................",
        "................",
        ".....cccc.......",
        "...cccccccc.....",
        "..cccCccCccc....",
        "..cccccccccc....",
        ".cccCccccCccc...",
        ".ccccccCccccc...",
        ".ccccCccccccc...",
        "..cccccccccc....",
        "...cccccccc.....",
        ".....cccc.......",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'c' => Some((198, 158, 92, 255)),
        'C' => Some((122, 84, 42, 255)), // chocolate freckles
        _ => None,
    });
}

/// parrot sprite (5 variants — VERIFIED w/Parrot Variant NBT table:
/// 0 red "red_blue", 1 blue, 2 green, 3 cyan "yellow_blue", 4 gray).
/// One clean-room silhouette (scarlet-macaw posture: crested head,
/// long tail feathers) with per-variant palettes.
pub(super) fn parrot_art(a: &mut [u8], t: u16, variant: u8, rng: &mut Rng) {
    // (head+crest, body, wing edge, tail)
    const PALETTES: [[i32; 3]; 4] = [
        [206, 48, 36],  // red — scarlet macaw
        [38, 88, 172],  // blue — hyacinth macaw
        [74, 158, 62],  // green — black-billed amazon
        [168, 84, 40],  // gray — cockatiel
    ];
    let p = match variant & 3 {
        3 => PALETTES[2], // cyan: green body, blue-and-yellow accents
        v => PALETTES[v as usize],
    };
    let belly = if (variant & 3) == 3 {
        [52, 116, 196]
    } else {
        [p[0] * 70 / 100, p[1] * 70 / 100, p[2] * 70 / 100]
    };
    let rows = [
        "....hh..........",
        "...hhhh.........",
        "..hHhhb.........",
        "..hhybb.........",
        "...bBBB.........",
        "...bBBWW........",
        "....BBWW..tt....",
        "....BBW..tTTt...",
        "....BBW..tTt....",
        "....bBb...tt....",
        "....bBb...tt....",
        ".....b....t.....",
        "..........t.....",
        "................",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'h' => Some((p[0], p[1], p[2], 255)),
        'H' => Some((p[0].min(255), p[1].min(255), 30, 255)), // crest highlight
        'y' => Some((238, 208, 68, 255)),  // beak
        'b' => Some((belly[0], belly[1], belly[2], 255)),
        'B' => Some((p[0], p[1], p[2], 255)),
        'W' => Some((168, 96, 52, 255)),   // wing edge
        't' => Some((88, 92, 116, 255)),   // tail feathers
        'T' => Some((120, 126, 150, 255)),
        _ => None,
    });
    // per-variant feather flecks (the "red_blue" two-tone of variants
    // 0/3: blue flight feathers on the red/cyan body)
    if variant == 0 || variant == 3 {
        for &(x, y) in &[(9, 6), (10, 8), (8, 7)] {
            put(a, t, x, y, 40, 76, 150, 255);
        }
    }
    let _ = rng;
}

/// illusioner sprite: a gray-robed illager holding a bow (VERIFIED
/// w/Illusioner: "Natural equipment: Bow (right hand: 95%...)"). Rendered
/// like the evoker/vindicator family — tall robe, hood, hidden face.
pub(super) fn illusioner_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let rows = [
        "....GGGG........",
        "...GGGGGG.......",
        "...GddddG.......",
        "...Gd..dG.......",
        "...GddddG...bb..",
        "....GGGG....bBb.",
        "...RRRRRR...bBb.",
        "..RRRRRRRR..bBb.",
        "..RRrrrrRR..bBb.",
        "..RRrrrrRR...bb.",
        "..RRrrrrRR...bb.",
        "..RRrrrrRR......",
        "..RRRRRRRR......",
        "...RRRRRR.......",
        "....RRRR........",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'G' => Some((58, 62, 74, 255)),    // hood
        'd' => Some((24, 26, 32, 255)),    // shadowed face
        'R' => Some((96, 104, 122, 255)),  // robe
        'r' => Some((120, 128, 148, 255)), // robe highlight
        'b' => Some((124, 92, 52, 255)),   // bow limbs
        'B' => Some((96, 70, 36, 255)),
        _ => None,
    });
    let _ = rng;
}
