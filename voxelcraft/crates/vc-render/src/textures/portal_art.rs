//! Round K (the Nether portal) — procedural tiles: the purple animated
//! portal-block swirl + the flint-and-steel item sprite. Clean-room art
//! (no third-party assets — no vanilla pixels were sampled), a child
//! module of textures.rs (shares put/jit/art helpers).
//!
//! The portal palette follows the wiki's color description only: the
//! block is "the translucent part of the Nether portal" that emits "a
//! light level of 11" and "emit the same purple particles produced by
//! endermen" (VERIFIED live 2026-09-26, reference wiki
//! /Nether_Portal_(block) infobox + §Usage) — a translucent violet
//! vortex, OUR swirl, not the original texture.

use super::{art, put, Rng};

/// The nether-portal block face: a translucent violet vortex — a deep
/// violet field with a swirling diagonal ribbon and sparse bright motes.
/// Alpha stays translucent (the block renders see-through like glass);
/// the per-pixel variation is a fixed pixel hash (stable per tile), so
/// the animated pulse in merge_pack_textures only moves the bright
/// pixels and the dark field stays stable.
pub(super) fn nether_portal_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    for y in 0..16 {
        for x in 0..16 {
            // distance + angle around the tile center — the vortex arms
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            let dist = (dx * dx + dy * dy).sqrt();
            let ang = dy.atan2(dx); // -pi..pi
                                    // the swirl: arcs of increasing radius, tighter toward the
                                    // center (dist + 2.2·angle phase mod 7)
            let phase = (dist + 2.2 * ang).rem_euclid(7.0);
            let (r, g, b, al) = if phase < 1.4 {
                // the bright swirl ribbon core
                (168, 96, 236, 245)
            } else if phase < 2.8 {
                // the ribbon's violet falloff
                (124, 52, 190, 232)
            } else {
                // the deep violet field
                (72, 28, 128, 210)
            };
            // sparse bright motes (the enderman-particle sparkle — every
            // 7th pixel of a fixed lattice, inside the field only)
            let mote = x % 5 == 2 && y % 7 == 3 && phase >= 2.8;
            let (r, g, b) = if mote { (206, 160, 250) } else { (r, g, b) };
            // fixed pixel hash — a stable per-tile shimmer
            let j = ((x * 31 + y * 17) % 5) as f32 / 5.0 - 0.5;
            let r = (r as f32 + j * 12.0).clamp(0.0, 255.0) as i32;
            let g = (g as f32 + j * 8.0).clamp(0.0, 255.0) as i32;
            let b = (b as f32 + j * 10.0).clamp(0.0, 255.0) as i32;
            put(a, t, x, y, r, g, b, al);
        }
    }
}

/// The flint-and-steel item sprite: a steel "C" striker striking a dark
/// flint shard, clean-room drawn (the wiki's item — VERIFIED
/// Nether_portal§Creation: "use of flint and steel" lights the frame).
pub(super) fn flint_and_steel_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    let rows = [
        "................",
        "................",
        "................",
        "....hhhh........",
        "...hssssh.......",
        "..hs...ssh......",
        "..hs....ssh.....",
        "..hs.....ss.....",
        "..hs.....ss.....",
        "..hs....ssh.....",
        "..hs...ssh......",
        "...hsssshd.ff...",
        "....hhhh...ff...",
        "............f.f.",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        // steel striker body — cold iron gray
        's' => Some((156, 160, 166, 255)),
        // the striker's dark outline
        'h' => Some((86, 90, 94, 255)),
        // the flint shard — near-black with a lighter facet
        'f' => Some((52, 50, 56, 255)),
        _ => None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_tile_is_painted_and_translucent() {
        let mut a = vec![0u8; super::super::ATLAS_SIZE * super::super::ATLAS_SIZE * 4];
        let mut rng = Rng::new(1);
        nether_portal_art(&mut a, 801, &mut rng);
        // the tile slot: every pixel painted, alpha translucent (< 255)
        let tx = (801 % 32) as usize;
        let ty = (801 / 32) as usize;
        let mut translucent = 0usize;
        for y in 0..super::super::TILE_PX {
            for x in 0..super::super::TILE_PX {
                let i = ((ty * 16 + y) * super::super::ATLAS_SIZE + tx * 16 + x) * 4;
                assert!(a[i + 3] > 0, "portal pixel ({x},{y}) unpainted");
                if a[i + 3] < 255 {
                    translucent += 1;
                }
            }
        }
        assert!(
            translucent > 128,
            "portal tile must read translucent ({translucent}/256 pixels < 255 alpha)"
        );
    }

    #[test]
    fn flint_and_steel_has_striker_and_flint() {
        let mut a = vec![0u8; super::super::ATLAS_SIZE * super::super::ATLAS_SIZE * 4];
        let mut rng = Rng::new(2);
        flint_and_steel_art(&mut a, 802, &mut rng);
        let tx = (802 % 32) as usize;
        let ty = (802 / 32) as usize;
        let (mut steel, mut flint) = (0usize, 0usize);
        for y in 0..super::super::TILE_PX {
            for x in 0..super::super::TILE_PX {
                let i = ((ty * 16 + y) * super::super::ATLAS_SIZE + tx * 16 + x) * 4;
                let (r, g, b) = (a[i] as i32, a[i + 1] as i32, a[i + 2] as i32);
                if (r, g, b) == (156, 160, 166) {
                    steel += 1;
                }
                if (r, g, b) == (52, 50, 56) {
                    flint += 1;
                }
            }
        }
        assert!(
            steel >= 20,
            "the steel striker body is missing ({steel} px)"
        );
        assert!(flint >= 4, "the flint shard is missing ({flint} px)");
    }
}
