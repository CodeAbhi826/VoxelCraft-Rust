//! TNT block art — Part 6 §3 redesign (owner verdict: palette + pattern).
//! CLEAN-ROOM ORIGINAL: no third-party pixels were read, copied or
//! traced. The vocabulary (corrugated charcoal casing, a bolted hazard-
//! chevron plate, a coiled-fuse emblem, a riveted lid) is generic
//! industrial iconography; the block keeps its ids, footprint and
//! explosive role. This replaces the earlier red-stick-bundle + band +
//! lettering motif, which followed the reference's expression too
//! closely. Changed vs that motif: palette, pattern, band proportions,
//! emblem. A child module of textures.rs (shares put/noise_fill).

use super::{noise_fill, put, Rng};

/// the side face: corrugated charcoal body, a dark-seamed hazard plate
/// across the middle (amber/black diagonal chevrons + corner bolts) and
/// a copper fuse-coil emblem on the upper body.
pub(super) fn tnt_side_art(a: &mut [u8], t: u16, _rng: &mut Rng) {
    // corrugated casing: vertical ribs on a 4-column period
    for y in 0..16 {
        for x in 0..16 {
            let (r, g, b) = match x % 4 {
                0 => (34, 32, 36),
                1 => (52, 50, 55),
                2 => (46, 44, 49),
                _ => (28, 27, 30),
            };
            put(a, t, x, y, r, g, b, 255);
        }
    }
    // fuse emblem (upper body, centred): a copper diamond with a spark
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        put(a, t, 8 + dx, 3 + dy, 176, 110, 72, 255);
    }
    put(a, t, 8, 3, 255, 236, 170, 255);
    // the hazard plate: dark bolted seams above/below, chevrons between
    for y in [6, 10] {
        for x in 0..16 {
            put(a, t, x, y, 18, 17, 20, 255);
        }
    }
    for y in 7i32..10i32 {
        for x in 0i32..16i32 {
            // period 4 shifted by the row → thin descending hazard tape
            let s = (x + y).rem_euclid(4i32);
            let (r, g, b) = if s < 2 { (224, 162, 56) } else { (26, 24, 26) };
            put(a, t, x, y, r, g, b, 255);
        }
    }
    // corner bolts holding the plate
    for (bx, by) in [(1, 7), (14, 7), (1, 9), (14, 9)] {
        put(a, t, bx, by, 212, 150, 104, 255);
    }
}

/// the top face: a speckled charcoal lid with four copper corner rivets
/// and a coiled fuse (dark cord ring, copper wind, bright spark) plus a
/// short cord stub pointing at one corner.
pub(super) fn tnt_top_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [58, 56, 62], 8, rng);
    for (rx, ry) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        for dy in 0..2 {
            for dx in 0..2 {
                put(a, t, rx + dx, ry + dy, 184, 120, 80, 255);
            }
        }
        put(a, t, rx, ry, 96, 62, 42, 255); // rivet rim shade
    }
    // the coil: outer dark cord, inner copper wind, bright spark core
    for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
        put(a, t, 7 + dx, 7 + dy, 30, 28, 30, 255);
    }
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        put(a, t, 7 + dx, 7 + dy, 176, 110, 72, 255);
    }
    put(a, t, 7, 7, 255, 236, 170, 255);
    // the cord stub toward the top-left corner
    put(a, t, 4, 4, 30, 28, 30, 255);
    put(a, t, 5, 4, 30, 28, 30, 255);
    put(a, t, 4, 5, 30, 28, 30, 255);
}

/// the bottom face: the plain casing underside — darker speckle with the
/// faint bolt rings that hold the lid.
pub(super) fn tnt_bottom_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    noise_fill(a, t, [44, 42, 48], 8, rng);
    for (rx, ry) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        put(a, t, rx, ry, 70, 68, 74, 255);
        put(a, t, rx + 1, ry + 1, 62, 48, 40, 255);
        put(a, t, rx, ry + 1, 56, 54, 60, 255);
        put(a, t, rx + 1, ry, 56, 54, 60, 255);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::textures::{ATLAS_SIZE, TILE_PX};

    /// render a painter into tile 0 of a scratch atlas and read a pixel
    fn scratch() -> Vec<u8> {
        vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4]
    }
    fn px(a: &[u8], x: usize, y: usize) -> [u8; 4] {
        let i = (y * ATLAS_SIZE + x) * 4;
        [a[i], a[i + 1], a[i + 2], a[i + 3]]
    }

    #[test]
    fn side_face_has_a_hazard_plate_and_a_charcoal_body() {
        let mut a = scratch();
        tnt_side_art(&mut a, 0, &mut Rng::new(1));
        // the body rows are low-chroma charcoal
        for x in 0..TILE_PX {
            let p = px(&a, x, 1);
            let (mx, mn) = (
                p[0].max(p[1]).max(p[2]),
                p[0].min(p[1]).min(p[2]),
            );
            assert!(mx - mn < 30, "body px {x} not charcoal: {p:?}");
            assert!(p[0] < 90, "body px {x} too bright: {p:?}");
        }
        // the plate rows carry both amber and near-black chevron pixels
        let mut amber = 0;
        let mut black = 0;
        for y in 7..10 {
            for x in 0..TILE_PX {
                let p = px(&a, x, y);
                if p == [224, 162, 56, 255] {
                    amber += 1;
                }
                if p == [26, 24, 26, 255] {
                    black += 1;
                }
            }
        }
        assert!(amber >= 20, "amber chevrons missing ({amber})");
        assert!(black >= 20, "black chevrons missing ({black})");
    }

    #[test]
    fn side_face_drops_the_old_red_bundle_palette() {
        // redesign guard: the replaced motif's stick reds must be gone
        let mut a = scratch();
        tnt_side_art(&mut a, 0, &mut Rng::new(7));
        for y in 0..TILE_PX {
            for x in 0..TILE_PX {
                let p = px(&a, x, y);
                assert!(
                    !(p[0] > 150 && p[1] < 90 && p[2] < 80),
                    "red-bundle pixel survived at ({x},{y}): {p:?}"
                );
            }
        }
    }

    #[test]
    fn top_face_carries_rivets_and_the_fuse_coil() {
        let mut a = scratch();
        tnt_top_art(&mut a, 0, &mut Rng::new(3));
        // four copper 2x2 rivets at the corners
        for (rx, ry) in [(2usize, 2usize), (12, 2), (2, 12), (12, 12)] {
            let p = px(&a, rx + 1, ry + 1);
            assert!(
                p[0] > 150 && p[1] > 90 && p[1] < 150 && p[2] < 110,
                "rivet at ({rx},{ry}) not copper: {p:?}"
            );
        }
        // the spark core sits at the centre
        assert_eq!(px(&a, 7, 7), [255, 236, 170, 255]);
    }

    #[test]
    fn painters_are_deterministic() {
        let mut a = scratch();
        let mut b = scratch();
        let (mut ra, mut rb) = (Rng::new(11), Rng::new(11));
        tnt_side_art(&mut a, 0, &mut ra);
        tnt_top_art(&mut a, 0, &mut ra);
        tnt_bottom_art(&mut a, 0, &mut ra);
        tnt_side_art(&mut b, 0, &mut rb);
        tnt_top_art(&mut b, 0, &mut rb);
        tnt_bottom_art(&mut b, 0, &mut rb);
        assert_eq!(a, b, "tnt art must be deterministic");
    }
}
