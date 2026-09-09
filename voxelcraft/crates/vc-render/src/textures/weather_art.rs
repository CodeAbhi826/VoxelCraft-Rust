//! Backlog-round procedural tiles — the weather bracket's two particle
//! sprites (rain streak + snowflake). Clean-room art (no Mojang assets),
//! a child module of textures.rs (shares put/jit/art helpers).

use super::{art, put, Rng};

/// Rain streak particle: a narrow vertical droplet band with a brighter
/// core and soft alpha edges — drawn as a 16×16 tile but the particle
/// system samples a thin sub-rect so each quad is a 2-px-wide streak.
pub(super) fn rain_streak(a: &mut [u8], t: u16, _rng: &mut Rng) {
    // vertical gradient band centered x 6..9, fading at the ends
    for y in 0..16 {
        // taper: solid core rows 3..13, faint tips
        let core = (3..=13).contains(&y);
        let alpha = if core { 190 } else if y == 2 || y == 14 { 120 } else { 0 };
        if alpha == 0 {
            continue;
        }
        for x in 6..=9 {
            let edge = x == 6 || x == 9;
            let al = if edge { alpha / 2 } else { alpha };
            // pale slate-blue droplet; slightly brighter core column
            let (r, g, b) = if x == 7 || x == 8 {
                (148, 178, 224)
            } else {
                (118, 148, 200)
            };
            put(a, t, x, y, r, g, b, al);
        }
    }
}

/// Snowflake particle: a soft 5-armed flake with a bright center —
/// sampled sub-rect by the weather particle system.
pub(super) fn snow_flake(a: &mut [u8], t: u16, _rng: &mut Rng) {
    // center at (8,8), arms out to radius 5 — drawn as pixel rows
    let rows = [
        "................",
        "................",
        "................",
        "......w.........",
        "......w.........",
        ".....w.w........",
        "....w...w.......",
        ".....w.w........",
        "..w...WWW...w...",
        ".....w.w........",
        "....w...w.......",
        ".....w.w........",
        "......w.........",
        "......w.........",
        "................",
        "................",
    ];
    art(a, t, rows, &|c| match c {
        'W' => Some((236, 246, 255, 255)),
        'w' => Some((206, 224, 244, 210)),
        _ => None,
    });
}

/// The fire block sprite: a clean-room flame — layered orange/yellow
/// tongues rising from a blue-violet base, cross-rendered like soul
/// fire. No vanilla pixels; the palette is public-domain fire colors.
pub(super) fn fire_art(a: &mut [u8], t: u16, rng: &mut Rng) {
    for y in 0..16 {
        // flame occupies the lower 13 rows; tips flicker with jitter
        let base = if y < 3 {
            // the blue-violet root band
            (86, 60, 190)
        } else if y < 7 {
            (196, 92, 16)
        } else if y < 11 {
            (232, 140, 28)
        } else {
            (250, 202, 66)
        };
        let width = match y {
            0..=2 => 6,
            3..=5 => 10,
            6..=8 => 12,
            _ => 8,
        };
        let cx = 7.5f32;
        for x in 0..16 {
            let dx = (x as f32 - cx).abs();
            if dx > width as f32 / 2.0 {
                continue;
            }
            // jitter the edges (per-pixel hash keeps it stable per tile)
            let edge = dx > width as f32 / 2.0 - 1.5;
            let j = ((x * 31 + y * 17) % 5) as f32 / 5.0;
            if edge && j < 0.4 {
                continue; // ragged flame edge
            }
            let bright = if x % 3 == 0 { 1.08 } else { 1.0 };
            let (br, bg, bb) = base;
            let r = ((br as f32 * bright) + (j - 0.5) * 18.0).clamp(0.0, 255.0) as i32;
            let g = ((bg as f32 * bright) + (j - 0.5) * 14.0).clamp(0.0, 255.0) as i32;
            let b = ((bb as f32 * bright) + (j - 0.5) * 10.0).clamp(0.0, 255.0) as i32;
            put(a, t, x, y, r, g, b, if edge { 200 } else { 255 });
        }
    }
    let _ = rng;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_streak_has_pixels() {
        let mut a = vec![0u8; super::super::ATLAS_SIZE * super::super::ATLAS_SIZE * 4];
        let mut rng = Rng::new(1);
        rain_streak(&mut a, 0, &mut rng);
        // the core column must have nonzero pixels
        assert!(a.iter().any(|&v| v != 0));
    }

    #[test]
    fn fire_has_pixels() {
        let mut a = vec![0u8; super::super::ATLAS_SIZE * super::super::ATLAS_SIZE * 4];
        let mut rng = Rng::new(1);
        fire_art(&mut a, 0, &mut rng);
        assert!(a.iter().any(|&v| v != 0));
    }

    #[test]
    fn snow_flake_has_pixels() {
        let mut a = vec![0u8; super::super::ATLAS_SIZE * super::super::ATLAS_SIZE * 4];
        let mut rng = Rng::new(1);
        snow_flake(&mut a, 0, &mut rng);
        assert!(a.iter().any(|&v| v != 0));
    }
}
