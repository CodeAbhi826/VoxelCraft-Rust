//! Tiny deterministic PRNG (xorshift64*) — no external deps, reproducible worlds.

pub struct Rng {
    pub state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed } }
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
