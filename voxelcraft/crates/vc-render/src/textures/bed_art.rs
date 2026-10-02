//! beds round (2026-09-22) procedural art: the red bed's four atlas
//! faces — the foot-top blanket (TILE_BED_FOOT_TOP 797), the head-top
//! pillow (TILE_BED_HEAD_TOP 798), the shared side face (TILE_BED_SIDE
//! 799), and the plain plank underside (TILE_BED_BOTTOM 800). Clean-room
//! pixel art — no third-party asset was read, copied, or traced; the
//! shapes follow the wiki's public block description only (a red bed:
//! the colored blanket + the white pillow on a wooden base). A child
//! module of textures.rs (shares the put/jit/art helpers).

use super::{art, jit, put, Rng};

/// the foot half's top face: the red blanket weave with darker fold
/// lines running across the bed axis and a lighter seam column at the
/// head-side edge.
pub(super) fn foot_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let r: [i32; 3] = [176, 46, 38]; // blanket red
    let d: [i32; 3] = [142, 34, 28]; // fold shade
    let h: [i32; 3] = [198, 70, 56]; // weave highlight
                                     // the blanket weave: flat red with a
                                     // per-pixel jitter (the atlas vocabulary)
    for y in 0..16 {
        for x in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(r[0], 6, rng),
                jit(r[1], 5, rng),
                jit(r[2], 5, rng),
                255,
            );
        }
    }
    // the tucked fold lines (two darker bands across the weave) + the
    // head-side seam column
    for k in 0..16 {
        put(a, t, k, 4, d[0], d[1], d[2], 255);
        put(a, t, k, 5, d[0], d[1], d[2], 255);
        put(a, t, k, 11, d[0], d[1], d[2], 255);
        put(a, t, k, 12, d[0], d[1], d[2], 255);
        put(a, t, 0, k, h[0], h[1], h[2], 255);
    }
    // deterministic weave speckle
    for _ in 0..12 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        put(a, t, x, y, h[0], h[1], h[2], 255);
    }
}

/// the head half's top face: the white pillow across the head end
/// (rows 0..6, the seam-shaded border ring) over the red blanket field
/// with its fold line — the classic red bed's top view.
pub(super) fn head_top_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "qqqqqqqqqqqqqqqq",
        "qppppppppppppppq",
        "qppppppppppppppq",
        "qppppppppppppppq",
        "qppppppppppppppq",
        "qppppppppppppppq",
        "qqqqqqqqqqqqqqqq",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "dddddddddddddddd",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
    ];
    let p: [i32; 3] = [228, 228, 226]; // pillow white
    let q: [i32; 3] = [196, 196, 192]; // pillow shade
    let r: [i32; 3] = [176, 46, 38]; // blanket red
    let d: [i32; 3] = [142, 34, 28]; // fold shade
    art(a, t, rows, &|ch| {
        Some(match ch {
            'p' => (p[0], p[1], p[2], 255),
            'q' => (q[0], q[1], q[2], 255),
            'r' => (r[0], r[1], r[2], 255),
            'd' => (d[0], d[1], d[2], 255),
            _ => return None,
        })
    });
}

/// the shared side face: the red blanket hangs over the upper half
/// (dark overhang line on top, shadow band under the hem) above the
/// wooden base with its plank seams — the bed's half-block side.
pub(super) fn side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "dddddddddddddddd",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "rrrrrrrrrrrrrrrr",
        "dddddddddddddddd",
        "wwwwwwwwwwwwwwww",
        "wvvwwwwvvwwwwvvw",
        "wwwwwwwwwwwwwwww",
        "wvvwwwwvvwwwwvvw",
        "wwwwwwwwwwwwwwww",
        "wwwwwwwwwwwwwwww",
    ];
    let d: [i32; 3] = [142, 34, 28]; // blanket shade (overhang + hem)
    let r: [i32; 3] = [176, 46, 38]; // blanket red
    let w: [i32; 3] = [138, 106, 62]; // wood base
    let v: [i32; 3] = [108, 82, 46]; // plank seam
    art(a, t, rows, &|ch| {
        Some(match ch {
            'd' => (d[0], d[1], d[2], 255),
            'r' => (r[0], r[1], r[2], 255),
            'w' => (w[0], w[1], w[2], 255),
            'v' => (v[0], v[1], v[2], 255),
            _ => return None,
        })
    });
}

/// the underside: the plain plank base — noise-filled plank tone with
/// two horizontal seams (the shared wood grain).
pub(super) fn bottom_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let w: [i32; 3] = [138, 106, 62]; // plank wood
    let v: [i32; 3] = [108, 82, 46]; // plank seam
    for y in 0..16 {
        for x in 0..16 {
            put(
                a,
                t,
                x,
                y,
                jit(w[0], 8, rng),
                jit(w[1], 6, rng),
                jit(w[2], 6, rng),
                255,
            );
        }
    }
    for k in 0..16 {
        put(a, t, k, 5, v[0], v[1], v[2], 255);
        put(a, t, k, 10, v[0], v[1], v[2], 255);
    }
}
