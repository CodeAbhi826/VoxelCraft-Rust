//! Deterministic scheduled-tick scheduler (§25 "scheduled block updates",
//! Phase 6). Vanilla semantics: a block requests a future update at a tick
//! offset; each game tick (20 Hz) the due updates run in a stable order.
//!
//! Determinism: entries live in a BTreeMap keyed by (due_tick, seq) — the
//! insertion sequence breaks ties, so the same schedule+tick order always
//! produces the same update order. This is the backbone the Phase-6
//! regression suite hashes.

use crate::world::World;
use std::collections::BTreeMap;

pub struct TickScheduler {
    /// (due_tick, insertion_seq) → position
    queue: BTreeMap<(u64, u64), [i32; 3]>,
    seq: u64,
    /// current sim tick (20 Hz since world start)
    now: u64,
    /// stats: entries scheduled in total (E2E / F3 evidence)
    pub scheduled_total: u64,
    /// stats: ticks actually executed
    pub executed_total: u64,
}

impl Default for TickScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TickScheduler {
    pub fn new() -> Self {
        TickScheduler { queue: BTreeMap::new(), seq: 0, now: 0, scheduled_total: 0, executed_total: 0 }
    }

    /// current sim tick
    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// schedule an update at `pos` in `delay` sim ticks. Re-scheduling the
    /// same position is allowed (vanilla dedupes per-position; multiple
    /// entries are harmless because the handlers are idempotent — a stale
    /// entry for a changed block no-ops).
    pub fn schedule(&mut self, pos: [i32; 3], delay: u64) {
        let seq = self.seq;
        self.seq += 1;
        self.queue.insert((self.now + delay.max(1), seq), pos);
        self.scheduled_total += 1;
    }

    /// advance one sim tick; returns the due positions in deterministic
    /// (due, insertion) order.
    pub fn tick(&mut self) -> Vec<[i32; 3]> {
        self.now += 1;
        let mut due = Vec::new();
        while let Some((&key, _)) = self.queue.iter().next() {
            if key.0 > self.now {
                break;
            }
            let (_, pos) = self.queue.remove_entry(&key).unwrap();
            due.push(pos);
        }
        self.executed_total += due.len() as u64;
        due
    }
}

/// random ticks (§26 "random ticks", progressive): vanilla samples 3 random
/// blocks per loaded section per game tick with the chunk's own seeded
/// RNG. We reproduce the observable contract: deterministic per (seed,
/// chunk, tick) sampling — grass spread/die in our behavior set.
pub struct RandomTicker {
    pub seed: u64,
}

impl RandomTicker {
    pub fn new(seed: u64) -> Self {
        RandomTicker { seed }
    }

    /// one game tick: sample `per_chunk` random positions in each of the
    /// given chunks (deterministic per seed/chunk/tick), run `f(pos)` on
    /// each. Takes a pre-collected chunk list so the caller can mutate the
    /// world inside `f` without borrow conflicts.
    pub fn tick<F: FnMut([i32; 3])>(
        &self,
        chunks: &[(i32, i32)],
        tick: u64,
        per_chunk: usize,
        mut f: F,
    ) {
        for &(cx, cz) in chunks.iter() {
            // deterministic sample stream: seed ← world seed ⊕ chunk ⊕ tick
            let mut s = self.seed ^ (cx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (cz as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ tick.wrapping_mul(0x1656_67B1_9E37_79B9);
            for _ in 0..per_chunk {
                // xorshift — fast, stable across platforms
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let lx = (s >> 32) as i32 & 15;
                let ly = (s >> 16) as i32 & 255;
                let lz = s as i32 & 15;
                f([cx * 16 + lx, ly, cz * 16 + lz]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_order_is_fifo_within_a_tick() {
        let mut t = TickScheduler::new();
        t.schedule([1, 1, 1], 1);
        t.schedule([2, 2, 2], 1);
        t.schedule([3, 3, 3], 3);
        assert_eq!(t.tick(), vec![[1, 1, 1], [2, 2, 2]]);
        assert_eq!(t.tick(), Vec::<[i32; 3]>::new());
        assert_eq!(t.tick(), vec![[3, 3, 3]]);
        assert_eq!(t.tick(), Vec::<[i32; 3]>::new());
        assert_eq!(t.now(), 4);
    }

    #[test]
    fn deterministic_random_sampling() {
        let rt = RandomTicker::new(7);
        let chunks: Vec<(i32, i32)> = vec![(0, 0), (1, -2)];
        let collect = |tick: u64| -> Vec<[i32; 3]> {
            let mut out = Vec::new();
            rt.tick(&chunks, tick, 3, |p| out.push(p));
            out
        };
        // same (seed, chunk, tick) → same sample stream
        let a = collect(9);
        let b = collect(9);
        assert_eq!(a, b);
        assert_eq!(a.len(), 6);
        // samples land inside their chunks
        for p in &a {
            assert!((0..16).contains(&p[0]) || (16..32).contains(&p[0]));
            assert!(p[1] >= 0 && p[1] <= 255);
        }
    }
}
