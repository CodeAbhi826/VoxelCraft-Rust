//! TNT round procedural art: the TNT block's three faces (TILE_TNT_SIDE
//! 794, TILE_TNT_TOP 795, TILE_TNT_BOTTOM 796). Clean-room pixel art —
//! no third-party asset was read, copied, or traced; the shapes follow
//! the wiki's public block description only ("a bundle of red sticks"
//! with the side band and the lettering concept). A child module of
//! textures.rs (shares the put/art helpers).

use super::{art, noise_fill, put, Rng};

/// the side face: a vertical red stick bundle with a horizontal
/// off-white band across the middle carrying the dark "TNT" lettering.
/// Rows are exactly 16 chars (the art() helper iterates chars); the
/// 4-column stick period (2 stick + 2 gap) tiles the full width.
pub(super) fn tnt_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    // the stick field: two red stick columns + a dark gap per period
    let sticks = "rRddrRddrRddrRdd";
    // the band: T (cols 0-3) · gap · N (cols 6-9) · gap · T (cols 12-15)
    let rows = [
        sticks,             // 0
        sticks,             // 1
        sticks,             // 2
        sticks,             // 3
        sticks,             // 4
        sticks,             // 5
        "tttt..n..n..tttt", // 6
        ".t....nn.n....t..", // 7
        ".t....n.nn....t..", // 8
        ".t....n..n....t..", // 9
        ".t....n..n....t..", // 10
        sticks,             // 11
        sticks,             // 12
        sticks,             // 13
        sticks,             // 14
        sticks,             // 15
    ];
    // every glyph maps its own color: the stick glyphs never appear in
    // the band and the letter glyphs never appear in the stick field
    art(a, t, rows, &|ch| {
        Some(match ch {
            'r' => (198, 46, 38, 255),  // stick body
            'R' => (224, 74, 48, 255),  // stick highlight
            'd' => (122, 28, 24, 255),  // the dark bundle gap
            't' | 'n' => (44, 38, 38, 255), // the dark lettering
            _ => (232, 228, 218, 255),  // the off-white band
        })
    });
}

/// the top face: the bundle seen from above — four red stick end-knots
/// in the corners and the dark fuse center, on the warm tan wrapper
/// field (procedural speckle).
pub(super) fn tnt_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [186, 164, 128], 10, rng);
    // the four corner end-knots (2×2 red dots with a dark rim)
    for (kx, ky) in [(2i32, 2i32), (12, 2), (2, 12), (12, 12)] {
        for dy in 0..2 {
            for dx in 0..2 {
                put(a, t, kx + dx, ky + dy, 198, 46, 38, 255);
            }
        }
        put(a, t, kx, ky, 140, 30, 26, 255); // the rim shade
    }
    // the fuse center: a 2×2 dark cord
    for dy in 0..2 {
        for dx in 0..2 {
            put(a, t, 7 + dx, 7 + dy, 66, 52, 40, 255);
        }
    }
}

/// the bottom face: the plain wrapper underside — the tan field with
/// the faint end-knot rings at the same corner positions (no fuse).
pub(super) fn tnt_bottom_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [172, 148, 112], 10, rng);
    for (kx, ky) in [(2i32, 2i32), (12, 2), (2, 12), (12, 12)] {
        for dy in 0..2 {
            for dx in 0..2 {
                put(a, t, kx + dx, ky + dy, 148, 120, 88, 255);
            }
        }
    }
}
