//! Tiny deterministic PRNG (xorshift64*) — no external deps, reproducible worlds.

pub struct Rng {
    pub state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform float in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// Uniform int in [0, n).
    pub fn next_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            // modulo sampling: DISCLOSED bias, negligible at n ≤ 32 (u64
            // modulo: ~1e-17 skew). Rejection sampling would be uniform but
            // changes the value STREAM — worldgen's Fisher-Yates shuffle
            // (gen.rs) is seeded from this, so the swap would shift every
            // golden-seed world. The bias stays; do not "fix" it without
            // re-pinning the determinism tests.
            (self.next_u64() % n as u64) as u32
        }
    }

    /// Deterministic hash of coordinates + seed (for chunk-seeded streams).
    pub fn hash3(seed: u64, x: i32, y: i32, z: i32) -> u64 {
        let mut h = seed.wrapping_add(0x27D4EB2F165667C5);
        h ^= (x as u64).wrapping_mul(0x9E3779B185EBA8A9);
        h ^= h >> 29;
        h = h.wrapping_mul(0xBF58476D1CE4E5B9);
        h ^= (y as u64).wrapping_mul(0x94D049BB133111EB);
        h ^= h >> 32;
        h = h.wrapping_mul(0x2545F4914F6CDD1D);
        h ^= (z as u64).wrapping_mul(0x9E3779B97F4A7C15);
        h ^= h >> 29;
        h
    }
}

// ---------------------------------------------------------------------
// Phase 0: this crate is 60 lines with ZERO tests, yet ~20 crates depend
// on it and every world seed, mob spawn, particle roll and procedurally
// generated texture is a pure function of this exact stream. The tests
// below pin the three properties the rest of the engine relies on:
// determinism (reproducible worlds), stream separation (two seeds must
// not share a trajectory), and the documented edge-case behaviour.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// The stream is LOAD-BEARING: worldgen's Fisher-Yates shuffle and every
    /// golden-seed world are seeded from it (see next_range's note). These
    /// literals must never change silently -- if they do, every golden seed
    /// in the repo moves and the divergence must be deliberate, not a side
    /// effect of a refactor.
    #[test]
    fn the_u64_stream_is_pinned() {
        let mut r = Rng::new(12345);
        assert_eq!(r.next_u64(), 0x9857FB32C9EFB5E4);
        assert_eq!(r.next_u64(), 0xC0CEBA4B4A71BCE4);
        assert_eq!(r.next_u64(), 0x1399CE5B8ADB52C4);
        assert_eq!(r.next_u64(), 0xBC6F73FBE045B705);
    }

    #[test]
    fn the_same_seed_replays_the_same_world() {
        let draw = |seed: u64| {
            let mut r = Rng::new(seed);
            (0..64).map(|_| r.next_u64()).collect::<Vec<_>>()
        };
        assert_eq!(draw(7), draw(7), "a seed must be reproducible");
        assert_ne!(draw(7), draw(8), "adjacent seeds must not collide");
    }

    #[test]
    fn seed_zero_falls_back_instead_of_sticking() {
        // xorshift is stuck at 0 forever; Rng::new(0) substitutes the
        // golden-ratio constant so a default seed still produces a stream.
        let mut zero = Rng::new(0);
        let mut fallback = Rng::new(0x9E3779B97F4A7C15);
        assert_eq!(zero.next_u64(), fallback.next_u64());
        assert_ne!(Rng::new(0).next_u64(), 0);
    }

    #[test]
    fn next_f32_stays_in_the_unit_interval_and_is_centred() {
        let mut r = Rng::new(99);
        let mut sum = 0.0f64;
        let n = 20_000;
        for _ in 0..n {
            let v = r.next_f32();
            assert!((0.0..1.0).contains(&v), "next_f32 left [0,1): {v}");
            sum += v as f64;
        }
        let mean = sum / n as f64;
        assert!((0.49..0.51).contains(&mean), "mean {mean} is not ~0.5");
    }

    #[test]
    fn next_range_is_total_and_in_bounds() {
        let mut r = Rng::new(4);
        // n == 0 is the degenerate argument every caller's `n` can reach
        // (an empty list); it must return 0, not divide by zero.
        assert_eq!(r.next_range(0), 0);
        assert_eq!(r.next_range(1), 0);
        for n in 1..=64u32 {
            for _ in 0..200 {
                let v = r.next_range(n);
                assert!(v < n, "next_range({n}) returned {v}");
            }
        }
    }

    #[test]
    fn hash3_separates_neighbouring_chunks() {
        // Adjacent chunks share two of three coordinates; if the hash
        // collapsed on either, neighbouring terrain would correlate.
        let base = Rng::hash3(1, 0, 0, 0);
        let mut all = std::collections::HashSet::new();
        for x in -2i32..=2 {
            for z in -2i32..=2 {
                let h = Rng::hash3(1, x, 64, z);
                assert!(all.insert(h), "hash3 collided at ({x},{z})");
                if x != 0 || z != 0 {
                    assert_ne!(h, base, "neighbour hash matched the origin");
                }
            }
        }
        // deterministic and seed-sensitive
        assert_eq!(Rng::hash3(1, 3, 64, -3), Rng::hash3(1, 3, 64, -3));
        assert_ne!(Rng::hash3(1, 3, 64, -3), Rng::hash3(2, 3, 64, -3));
    }

    #[test]
    fn hash3_pins_its_value() {
        assert_eq!(Rng::hash3(0, 0, 0, 0), 0x1739910ABA15DAA5);
    }
}
