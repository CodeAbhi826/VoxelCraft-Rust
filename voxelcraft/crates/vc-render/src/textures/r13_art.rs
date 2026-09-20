//! Round 13 (station GUIs) procedural art: the plain-book item sprite
//! (TILE_BOOK 791) and the grindstone block sprite (TILE_GRINDSTONE 792).
//! Clean-room pixel art — no third-party asset was read, copied, or traced;
//! the shapes follow the wiki's public item/block descriptions only.
//! A child module of textures.rs (shares the put/art helpers).

use super::{art, put, Rng};

/// the plain book item: a closed tan book, spine on the left, page
/// block on the right, small clasp at the spine lip. Rows are exactly
/// 16 chars (the art() helper iterates chars).
pub(super) fn book_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let body = ".dcccccceepepeddd";
    let clasp = ".dcccccceepepedkk";
    let border = ".ddddddddddddddd";
    let rows = [
        "................",
        "................",
        border,
        body,
        body,
        body,
        body,
        clasp,
        clasp,
        body,
        body,
        body,
        body,
        border,
        "................",
        "................",
    ];
    let c: [i32; 3] = [160, 120, 70]; // worn leather cover
    let d: [i32; 3] = [128, 94, 54]; // spine shade
    let e: [i32; 3] = [222, 214, 190]; // page edges
    let p: [i32; 3] = [196, 186, 158]; // page shade
    let k: [i32; 3] = [90, 74, 50]; // clasp
    art(a, t, rows, &|ch| {
        Some(match ch {
            'c' => (c[0], c[1], c[2], 255),
            'd' => (d[0], d[1], d[2], 255),
            'e' => (e[0], e[1], e[2], 255),
            'p' => (p[0], p[1], p[2], 255),
            'k' => (k[0], k[1], k[2], 255),
            _ => return None,
        })
    });
}

/// the grindstone block face: a wooden frame (border + side legs)
/// around the stone wheel — grey disc, darker rim, lighter hub band.
pub(super) fn grindstone_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    let w: [i32; 3] = [138, 106, 62]; // wood
    let d: [i32; 3] = [108, 82, 46]; // wood shade
    let o: [i32; 3] = [138, 138, 138]; // stone
    let s: [i32; 3] = [104, 104, 104]; // stone rim
    let i: [i32; 3] = [168, 168, 168]; // stone highlight hub
    // base fill: the wooden frame color everywhere
    for y in 0..16 {
        for x in 0..16 {
            put(a, t, x, y, w[0], w[1], w[2], 255);
        }
    }
    // frame shading: darker border ring + inner leg columns
    for k in 0..16 {
        put(a, t, k, 0, d[0], d[1], d[2], 255);
        put(a, t, k, 15, d[0], d[1], d[2], 255);
        put(a, t, 0, k, d[0], d[1], d[2], 255);
        put(a, t, 15, k, d[0], d[1], d[2], 255);
        put(a, t, 2, k, d[0], d[1], d[2], 255);
        put(a, t, 13, k, d[0], d[1], d[2], 255);
    }
    // the wheel: a disc centered between the legs
    let (cx, cy) = (7.5f32, 8.0f32);
    for y in 0..16 {
        for x in 0..16 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            if r <= 4.7 {
                let c = if r > 3.8 {
                    s // rim
                } else if (dy.abs() < 1.1 && dx.abs() < 2.4) || r < 1.5 {
                    i // hub band
                } else {
                    o
                };
                put(a, t, x, y, c[0], c[1], c[2], 255);
            }
        }
    }
    // deterministic stone speckle inside the wheel
    for _ in 0..16 {
        let x = rng.next_range(16) as i32;
        let y = rng.next_range(16) as i32;
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        if dx * dx + dy * dy <= 14.0 {
            put(a, t, x, y, s[0], s[1], s[2], 255);
        }
    }
}
