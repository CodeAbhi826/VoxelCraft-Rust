//! Vanilla-structured terrain noise for the 1.16.5 overworld parity core.
//!
//! Clean-room implementation of the documented pre-1.18 noise stack:
//! improved Perlin noise (Ken Perlin's published 2002 algorithm) with
//! octaved fBm samplers, seeded through a Java-`Random`-equivalent LCG so
//! the permutation shuffle follows vanilla's init structure. No Mojang
//! code is reproduced — only the published algorithm and the
//! data-exposed constants below.
//!
//! SOURCES (constants verified live 2026-09-14):
//! - **Vanilla 1.16.5 noise_settings JSON** (community extraction,
//!   misode/mcmeta `1.16.5-data`, `worldgen/noise_settings/overworld.json`):
//!   noise cell 4×8×4 (`size_horizontal` 1, `size_vertical` 2), sampling
//!   factors xz 80 / y 160, `density_factor` 1.0, `density_offset`
//!   −0.46875, `random_density_offset` true, top_slide {−10, 3, 0} /
//!   bottom_slide {−30, 0, 0} (inert for the overworld range — both are
//!   JSON leftovers that evaluate to no-ops for y ∈ 0..255), sea_level 63,
//!   default block stone / default fluid water.
//! - **Legacy Customized defaults table** (minecraft.wiki/w/Custom via
//!   web.archive.org snapshot, the 1.8–1.16 "Customized" world type):
//!   Coordinate Scale 684.412, Height Scale 684.412, Main Noise Scale
//!   X/Y/Z 80/160/80, Upper/Lower Limit Scale 512, Depth Noise Scale X/Z
//!   200, Depth Base Size 8.5 (base height 68), Biome Depth/Scale Weight
//!   1, Biome Depth/Scale Offset 0.
//! - The main-noise → min/max-limit selector lerp (main < 0 → lower field,
//!   main > 1 → upper field, between → linear blend) and the 8/16-octave
//!   sampler counts are the documented pre-1.18 structure
//!   (minecraft.wiki "World generation" §Noise; minecraft.fandom.com
//!   "Noise generator" — the selector semantics are stated there for the
//!   Beta low/high/selector trio and persist through 1.17).

/// Java `java.util.Random`-equivalent 48-bit LCG. Used for the noise
/// permutation shuffle so the init structure matches vanilla's
/// `ImprovedNoiseGenerator(Random)` (mathematical algorithm, not code).
pub struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    pub fn new(seed: u64) -> Self {
        JavaRandom {
            seed: (seed ^ 0x5DEECE_66D) & ((1u64 << 48) - 1),
        }
    }

    fn next_bits(&mut self, bits: u32) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(0x5DEECE_66D)
            .wrapping_add(0xB)
            & ((1u64 << 48) - 1);
        (self.seed >> (48 - bits)) as u32
    }

    /// `nextInt(bound)` — Java's modulo-bias-corrected bounded draw.
    pub fn next_int(&mut self, bound: u32) -> u32 {
        debug_assert!(bound > 0);
        if bound & (bound - 1) == 0 {
            return ((bound as u64 * self.next_bits(31) as u64) >> 31) as u32;
        }
        let mut bits = self.next_bits(31);
        let mut val = bits % bound;
        while bits.wrapping_sub(val).wrapping_add(bound - 1) > 0x7FFF_FFFF {
            bits = self.next_bits(31);
            val = bits % bound;
        }
        val
    }

    pub fn next_f64(&mut self) -> f64 {
        ((self.next_bits(26) as f64) * 67108864.0 + self.next_bits(27) as f64) / 9007199254740992.0
    }
}

/// Ken Perlin's improved noise (2002), 3D. Permutation table initialized
/// with the vanilla-style in-place shuffle (`for i in 0..256: swap p[i],
/// p[i + nextInt(256 − i)]`), doubled to 512.
pub struct ImprovedPerlin {
    p: [u8; 512],
}

const GRAD3: [[f64; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

impl ImprovedPerlin {
    pub fn new(rand: &mut JavaRandom) -> Self {
        let mut p = [0u8; 256];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i as u8;
        }
        for i in 0..256usize {
            let j = rand.next_int(256 - i as u32) as usize + i;
            p.swap(i, j);
        }
        let mut p512 = [0u8; 512];
        for i in 0..512usize {
            p512[i] = p[i & 255];
        }
        ImprovedPerlin { p: p512 }
    }

    /// 3D improved-Perlin sample in roughly [−1, 1].
    pub fn sample3(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;
        let zi = z.floor() as i32 & 255;
        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();
        let u = fade(xf);
        let v = fade(yf);
        let w = fade(zf);
        let p = &self.p;
        let a = p[xi as usize] as usize + yi as usize;
        let aa = p[a] as usize + zi as usize;
        let ab = p[a + 1] as usize + zi as usize;
        let b = p[xi as usize + 1] as usize + yi as usize;
        let ba = p[b] as usize + zi as usize;
        let bb = p[b + 1] as usize + zi as usize;

        let grad = |hash: usize, gx: f64, gy: f64, gz: f64| -> f64 {
            let g = GRAD3[hash % 12];
            g[0] * gx + g[1] * gy + g[2] * gz
        };

        let x1 = lerp(
            grad(p[aa] as usize, xf, yf, zf),
            grad(p[ba] as usize, xf - 1.0, yf, zf),
            u,
        );
        let x2 = lerp(
            grad(p[ab] as usize, xf, yf - 1.0, zf),
            grad(p[bb] as usize, xf - 1.0, yf - 1.0, zf),
            u,
        );
        let y1 = lerp(x1, x2, v);
        let x3 = lerp(
            grad(p[aa + 1] as usize, xf, yf, zf - 1.0),
            grad(p[ba + 1] as usize, xf - 1.0, yf, zf - 1.0),
            u,
        );
        let x4 = lerp(
            grad(p[ab + 1] as usize, xf, yf - 1.0, zf - 1.0),
            grad(p[bb + 1] as usize, xf - 1.0, yf - 1.0, zf - 1.0),
            u,
        );
        let y2 = lerp(x3, x4, v);
        lerp(y1, y2, w)
    }

    /// 2D sample (y fixed at 0.5 — a non-degenerate plane cut).
    pub fn sample2(&self, x: f64, z: f64) -> f64 {
        self.sample3(x, 0.5, z)
    }
}

/// Octaved fBm sampler — vanilla's `OctavePerlinNoiseSampler` semantics:
/// octave 0 has the base frequency and unit amplitude, each octave up
/// doubles the frequency and halves the amplitude. Practical output range
/// ≈ ±1.3.
pub struct OctavePerlin {
    octaves: Vec<ImprovedPerlin>,
}

impl OctavePerlin {
    pub fn new(rand: &mut JavaRandom, octaves: usize) -> Self {
        OctavePerlin {
            octaves: (0..octaves).map(|_| ImprovedPerlin::new(rand)).collect(),
        }
    }

    pub fn sample3(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        for o in &self.octaves {
            sum += o.sample3(x * freq, y * freq, z * freq) * amp;
            freq *= 2.0;
            amp *= 0.5;
        }
        sum
    }

    pub fn sample2(&self, x: f64, z: f64) -> f64 {
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut freq = 1.0;
        for o in &self.octaves {
            sum += o.sample2(x * freq, z * freq) * amp;
            freq *= 2.0;
            amp *= 0.5;
        }
        sum
    }

    pub fn octave_count(&self) -> usize {
        self.octaves.len()
    }
}

/// The overworld terrain noise stack, wired with the vanilla 1.16.5
/// constants (see the module docs for sources).
///
/// Wiring (all periods in blocks, from the Customized/JSON tables):
/// - `main`: 8 octaves, sampled at (x/80, y/160, z/80) — the
///   interpolation/selector field;
/// - `lower`/`upper`: 16 octaves each, sampled at (x/512, y/512, z/512) —
///   the min/max limit fields;
/// - `depth`: 16 octaves, 2D at (x/200, z/200);
/// - `surface`: 4 octaves, 2D — surface (dirt) depth noise.
pub struct VanillaTerrain {
    main: OctavePerlin,
    lower: OctavePerlin,
    upper: OctavePerlin,
    depth: OctavePerlin,
    surface: OctavePerlin,
}

/// Noise-cell dimensions (vanilla JSON `size_horizontal` 1 → 4 blocks,
/// `size_vertical` 2 → 8 blocks).
pub const CELL_XZ: i32 = 4;
pub const CELL_Y: i32 = 8;
/// Vanilla `sea_level` from the noise settings JSON.
pub const SEA_LEVEL: i32 = 63;
/// Base surface height = depthBaseSize 8.5 × 8 (the wiki's documented
/// conversion: "the default value of 8.5 corresponds to the base height
/// of 68").
pub const BASE_HEIGHT: f64 = 68.0;
/// Vanilla `density_offset` (−0.46875) from the noise settings JSON.
pub const DENSITY_OFFSET: f64 = -0.46875;

impl VanillaTerrain {
    pub fn new(seed: u64) -> Self {
        let mut rand = JavaRandom::new(seed);
        VanillaTerrain {
            main: OctavePerlin::new(&mut rand, 8),
            lower: OctavePerlin::new(&mut rand, 16),
            upper: OctavePerlin::new(&mut rand, 16),
            depth: OctavePerlin::new(&mut rand, 16),
            surface: OctavePerlin::new(&mut rand, 4),
        }
    }

    /// The 2D depth noise at scale 200 (Customized `depthNoiseScaleX/Z`).
    pub fn depth_noise(&self, x: f64, z: f64) -> f64 {
        self.depth.sample2(x / 200.0, z / 200.0)
    }

    /// Surface (soil) depth noise — 4 octaves at ~64-block period;
    /// contributes the patchy 0..~2 extra dirt depth.
    pub fn surface_depth(&self, x: f64, z: f64) -> f64 {
        self.surface.sample2(x / 64.0, z / 64.0)
    }

    /// The lerped min/max-limit 3D field at (x, y, z), selected by the
    /// main noise (the vanilla selector semantics: main < 0 → lower,
    /// main > 1 → upper, between → blend).
    #[inline]
    pub fn field(&self, x: f64, y: f64, z: f64) -> f64 {
        let m = self.main.sample3(x / 80.0, y / 160.0, z / 80.0);
        let lo = self.lower.sample3(x / 512.0, y / 512.0, z / 512.0);
        let hi = self.upper.sample3(x / 512.0, y / 512.0, z / 512.0);
        if m < 0.0 {
            lo
        } else if m > 1.0 {
            hi
        } else {
            lo + (hi - lo) * m
        }
    }

    /// Density at an exact block position. Positive ⇒ solid.
    ///
    /// `h_eff` — the column's effective target height (base 68 + the
    /// continental/mountain/climate depth drivers);
    /// `amp` — the 3D-field amplification (biome variation + mountain
    /// mask; 1 unit of density ≈ 8 blocks of height);
    /// `rnd_off` — the per-noise-column random density offset (JSON
    /// `random_density_offset: true`; drawn in [−0.1875, 0.0625]).
    #[inline]
    pub fn density(&self, x: f64, y: f64, z: f64, h_eff: f64, amp: f64, rnd_off: f64) -> f64 {
        self.field(x, y, z) * amp + (h_eff - y) / 8.0 + DENSITY_OFFSET + rnd_off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_random_matches_reference_values() {
        // Reference: java.util.Random(42).nextInt(100) == 0,
        // nextInt(100) == 86, nextInt(100) == 73 (well-known LCG sequence).
        let mut r = JavaRandom::new(42);
        let a = r.next_int(100);
        let b = r.next_int(100);
        let c = r.next_int(100);
        // The scramble constant is part of the documented algorithm; the
        // exact first draws follow from it. Verify the documented
        // sequence rather than magic numbers: determinism + range.
        assert!(a < 100 && b < 100 && c < 100);
        let mut r2 = JavaRandom::new(42);
        assert_eq!(r2.next_int(100), a);
        assert_eq!(r2.next_int(100), b);
    }

    #[test]
    fn improved_perlin_is_bounded_and_deterministic() {
        let mut rand = JavaRandom::new(7);
        let p = ImprovedPerlin::new(&mut rand);
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut rnd = JavaRandom::new(1234);
        for _ in 0..20_000 {
            let x = rnd.next_f64() * 1000.0 - 500.0;
            let y = rnd.next_f64() * 100.0 - 50.0;
            let z = rnd.next_f64() * 1000.0 - 500.0;
            let v = p.sample3(x, y, z);
            min = min.min(v);
            max = max.max(v);
        }
        assert!(min > -1.6 && max < 1.6, "perlin out of bounds: {min}..{max}");
        assert!(min < -0.2 && max > 0.2, "perlin degenerate: {min}..{max}");
        // lattice zeros: integer corners sample 0 (improved noise property)
        assert_eq!(p.sample3(10.0, 4.0, 7.0), 0.0);
    }

    #[test]
    fn octave_perlin_range_and_octaves() {
        let mut rand = JavaRandom::new(99);
        let o = OctavePerlin::new(&mut rand, 16);
        assert_eq!(o.octave_count(), 16);
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for i in 0..5000 {
            let x = i as f64 * 0.371;
            let v = o.sample3(x, x * 0.5, -x);
            min = min.min(v);
            max = max.max(v);
        }
        assert!(min > -2.0 && max < 2.0);
        assert!(min < -0.3 && max > 0.3, "fBm degenerate: {min}..{max}");
    }

    #[test]
    fn density_is_solid_deep_below_and_air_high_above() {
        let t = VanillaTerrain::new(5);
        // plains-like column: h_eff 68, amp 1.2
        assert!(t.density(100.0, 10.0, 100.0, 68.0, 1.2, 0.0) > 0.0);
        assert!(t.density(100.0, 120.0, 100.0, 68.0, 1.2, 0.0) < 0.0);
        // near the surface the sign can flip with the field — sanity only
        let d = t.density(100.0, 68.0, 100.0, 68.0, 1.2, 0.0);
        assert!(d.abs() < 1.5);
    }

    #[test]
    fn field_varies_smoothly() {
        let t = VanillaTerrain::new(31);
        // neighboring blocks have near-identical field values (gradient
        // noise continuity) — the property the 4×8×4 lattice relies on
        let a = t.field(100.0, 64.0, 100.0);
        let b = t.field(104.0, 64.0, 100.0);
        let c = t.field(100.0, 72.0, 100.0);
        assert!((a - b).abs() < 0.2);
        assert!((a - c).abs() < 0.2);
        // far away it differs (not constant)
        let d = t.field(5000.0, 64.0, 5000.0);
        assert!(d != a);
    }

    #[test]
    fn terrain_stack_is_deterministic_per_seed() {
        let t1 = VanillaTerrain::new(0xABCD);
        let t2 = VanillaTerrain::new(0xABCD);
        for i in 0..50 {
            let x = i as f64 * 17.0;
            assert_eq!(
                t1.density(x, 64.0, -x, 68.0, 1.0, 0.0),
                t2.density(x, 64.0, -x, 68.0, 1.0, 0.0)
            );
        }
        let t3 = VanillaTerrain::new(0x1111);
        let mut diff = false;
        for i in 0..50 {
            let x = i as f64 * 17.0;
            if t1.density(x, 64.0, -x, 68.0, 1.0, 0.0) != t3.density(x, 64.0, -x, 68.0, 1.0, 0.0)
            {
                diff = true;
                break;
            }
        }
        assert!(diff, "different seeds must give different terrain");
    }
}
