//! Monster spawners (Phase 5 §27): the dungeon block entity. All values
//! VERIFIED against minecraft.wiki "Monster Spawner" (live pull, Sep 2026):
//!
//! - activates when a player is within a spherical radius of **16 blocks**
//!   of the spawner block's center (feet-level test)
//! - each cycle attempts to spawn **4 mobs** at random points in the
//!   (at most) **9×3×9** volume around the block
//! - between cycles it waits **200..=799 game ticks** (10..39.95 s); the
//!   wait restarts only after at least one mob actually spawned — if all
//!   4 points fail, it retries the very next tick
//! - Java rule: if **≥ 6 mobs of the spawner's type** intersect a **9×9×9**
//!   volume centered on the spawner, it "poofs" (spawns nothing) and
//!   resets its delay
//! - a spawn point needs a solid floor block with 2 non-solid cells above
//!
//! Documented adaptation: vanilla stores the mob type in the spawner NBT
//! (`SpawnData`); we encode it in the block state (3 states). The tiny
//! spinning model inside the cage is a client visual we omit — the
//! mechanics (range/delay/cycle/cap) are the gameplay.

use crate::mobs::{MobKind, MobSystem};
use std::collections::HashMap;
use vc_blocks::blocks::*;
use vc_rng::rng::Rng;
use vc_world::world::World;

/// activation range (blocks, sphere radius from the block center)
pub const PLAYER_RANGE: f32 = 16.0;
/// mobs attempted per cycle (VERIFIED)
pub const SPAWN_COUNT: u32 = 4;
/// minimum wait between cycles in game ticks (VERIFIED 200)
pub const MIN_DELAY: u32 = 200;
/// exclusive upper bound of the next-delay roll (VERIFIED 799 max)
pub const MAX_DELAY: u32 = 800;
/// spawning stops when this many same-type mobs are in the 9×9×9 (Java)
pub const MAX_NEARBY: usize = 6;
/// half-extents of the nearby-mob box (9×9×9 → 4.5 either side)
const NEAR_HALF: i32 = 4;
/// half-extents of the spawn-attempt volume (9×3×9)
const SPAWN_HALF_XZ: i32 = 4;
const SPAWN_HALF_Y: i32 = 1;
/// vanilla NBT `Delay` default — the first activation fires quickly
const INITIAL_DELAY: u32 = 20;

/// block-entity state of one spawner
#[derive(Clone, Copy, Debug)]
pub struct Spawner {
    /// mob kind code (SPAWNER_ZOMBIE / SKELETON / SPIDER / 3 = blaze,
    /// the Phase E1 fortress platform spawner)
    pub mob: u8,
    /// ticks until the next spawn cycle
    pub delay: u32,
    /// cycles that ended in a successful spawn (stats/E2E)
    pub cycles: u64,
}

/// mob kind code → MobKind
pub fn mob_kind(code: u8) -> MobKind {
    match code {
        SPAWNER_SKELETON => MobKind::Skeleton,
        SPAWNER_SPIDER => MobKind::Spider,
        3 => MobKind::Blaze, // Phase E1 fortress spawner (SPAWNER_BLAZE)
        4 => MobKind::WitherSkeleton, // Phase E2 fortress platform
        _ => MobKind::Zombie,
    }
}

pub struct Spawners {
    /// keyed by the spawner block position
    pub map: HashMap<[i32; 3], Spawner>,
    rng: Rng,
    /// total mobs ever spawned from spawners (stats/F3/E2E)
    pub spawned_total: u64,
}

impl Default for Spawners {
    fn default() -> Self {
        Spawners::new(0x5D_EA1)
    }
}

impl Spawners {
    pub fn new(seed: u64) -> Self {
        Spawners {
            map: HashMap::new(),
            rng: Rng::new(seed),
            spawned_total: 0,
        }
    }

    /// register (or refresh) a spawner block. Called by the game layer
    /// whenever a chunk containing spawners arrives, and when a spawner
    /// block is placed. Re-registering an existing position keeps its
    /// delay/cycle state.
    pub fn register(&mut self, pos: [i32; 3], mob: u8) {
        self.map.entry(pos).or_insert(Spawner {
            mob,
            delay: INITIAL_DELAY,
            cycles: 0,
        });
    }

    /// the spawner block was removed — drop its entity state
    pub fn remove(&mut self, pos: [i32; 3]) {
        self.map.remove(&pos);
    }

    /// ONE deterministic sim tick: activation gate → delay → cycle.
    /// `player` is the feet position the gate tests (None = no player →
    /// everything sleeps, vanilla behavior when nobody is near).
    pub fn tick(&mut self, world: &World, player: Option<[f32; 3]>, mobs: &mut MobSystem) {
        let positions: Vec<[i32; 3]> = self.map.keys().copied().collect();
        for pos in positions {
            let sp = *self.map.get(&pos).unwrap();
            let kind = mob_kind(sp.mob);

            // activation: player within the 16-block sphere (feet test)
            let active = player.map(|p| {
                let dx = p[0] - (pos[0] as f32 + 0.5);
                let dy = p[1] - (pos[1] as f32 + 0.5);
                let dz = p[2] - (pos[2] as f32 + 0.5);
                dx * dx + dy * dy + dz * dz <= PLAYER_RANGE * PLAYER_RANGE
            });
            if !active.unwrap_or(false) {
                continue;
            }

            // wait out the delay
            if sp.delay > 0 {
                self.map.get_mut(&pos).unwrap().delay = sp.delay - 1;
                continue;
            }

            // Java cap: ≥6 same-type mobs in the 9×9×9 box → poof
            let nearby = mobs
                .list
                .iter()
                .filter(|m| {
                    m.kind == kind
                        && (m.pos[0] as i32 - pos[0]).abs() <= NEAR_HALF
                        && (m.pos[1] as i32 - pos[1]).abs() <= NEAR_HALF
                        && (m.pos[2] as i32 - pos[2]).abs() <= NEAR_HALF
                })
                .count();
            if nearby >= MAX_NEARBY {
                let d = self.next_delay();
                self.map.get_mut(&pos).unwrap().delay = d;
                continue;
            }

            // the cycle: 4 attempts at random points in 9×3×9
            let mut spawned = 0;
            for _ in 0..SPAWN_COUNT {
                let dx = self.rng.next_range((SPAWN_HALF_XZ * 2 + 1) as u32) as i32 - SPAWN_HALF_XZ;
                let dy = self.rng.next_range((SPAWN_HALF_Y * 2 + 1) as u32) as i32 - SPAWN_HALF_Y;
                let dz = self.rng.next_range((SPAWN_HALF_XZ * 2 + 1) as u32) as i32 - SPAWN_HALF_XZ;
                let sx = pos[0] + dx;
                let sy = (pos[1] + dy).max(1);
                let sz = pos[2] + dz;
                // solid floor + two non-solid cells (spawner-ish spawn rule)
                let floor_ok = is_solid(world.get_block(sx, sy - 1, sz));
                let clear_ok = !is_solid(world.get_block(sx, sy, sz))
                    && !is_solid(world.get_block(sx, sy + 1, sz));
                if floor_ok && clear_ok {
                    if mobs.spawn_at(kind, sx, sy, sz).is_some() {
                        spawned += 1;
                        self.spawned_total += 1;
                    }
                }
            }

            if spawned > 0 {
                // only a successful cycle starts the next wait
                let d = self.next_delay();
                let e = self.map.get_mut(&pos).unwrap();
                e.delay = d;
                e.cycles += 1;
            }
            // else: delay stays 0 — retry next tick (vanilla)
        }
    }

    /// next wait: uniform 200..=799 ticks (VERIFIED)
    fn next_delay(&mut self) -> u32 {
        MIN_DELAY + self.rng.next_range(MAX_DELAY - MIN_DELAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_chunk::chunk::Chunk;

    fn flat_world() -> World {
        let mut w = World::new(21);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
                for y in 0..=64i32 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y as usize, lz, STONE);
                        }
                    }
                }
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w
    }

    #[test]
    fn delays_roll_in_the_verified_range() {
        let mut s = Spawners::new(7);
        for _ in 0..2000 {
            let d = s.next_delay();
            assert!(
                (MIN_DELAY..MAX_DELAY).contains(&d),
                "delay {d} out of 200..=799"
            );
        }
    }

    #[test]
    fn spawns_only_when_player_is_close() {
        let w = flat_world();
        let mut mobs = MobSystem::new(7);
        let mut s = Spawners::new(8);
        s.register([8, 65, 8], SPAWNER_ZOMBIE);
        s.map.get_mut(&[8, 65, 8]).unwrap().delay = 0;
        // player 40 blocks away: nothing happens, delay stays armed
        s.tick(&w, Some([48.0, 65.0, 8.0]), &mut mobs);
        assert_eq!(mobs.list.len(), 0, "out of range: no spawn");
        assert_eq!(
            s.map.get(&[8, 65, 8]).unwrap().delay,
            0,
            "delay not consumed while idle"
        );
        // player 10 blocks away: cycle fires immediately (delay 0)
        s.tick(&w, Some([18.0, 65.0, 8.0]), &mut mobs);
        assert!(!mobs.list.is_empty(), "in range: mobs spawned");
        assert!(s.spawned_total > 0);
    }

    #[test]
    fn cap_stops_spawning_at_6_nearby() {
        let w = flat_world();
        let mut mobs = MobSystem::new(9);
        // pre-place 6 zombies inside the 9×9×9 box
        for i in 0..6 {
            mobs.spawn_at(MobKind::Zombie, 6 + i, 65, 8).unwrap();
        }
        let mut s = Spawners::new(10);
        s.register([8, 65, 8], SPAWNER_ZOMBIE);
        s.map.get_mut(&[8, 65, 8]).unwrap().delay = 0;
        s.tick(&w, Some([9.0, 65.0, 9.0]), &mut mobs);
        assert_eq!(mobs.list.len(), 6, "cap: no 7th mob from the spawner");
        // and the poof still resets the delay (a wait cycle began)
        assert!(s.map.get(&[8, 65, 8]).unwrap().delay >= MIN_DELAY);
    }

    #[test]
    fn successful_cycle_resets_delay_unsuccessful_does_not() {
        let w = flat_world();
        let mut mobs = MobSystem::new(11);
        let mut s = Spawners::new(12);
        s.register([8, 65, 8], SPAWNER_ZOMBIE);
        s.map.get_mut(&[8, 65, 8]).unwrap().delay = 0;
        // success path (open stone plain): delay resets to 200..=799
        s.tick(&w, Some([9.0, 65.0, 9.0]), &mut mobs);
        let d = s.map.get(&[8, 65, 8]).unwrap().delay;
        assert!((MIN_DELAY..MAX_DELAY).contains(&d));
        // failure path: wall the whole 9×3×9 volume with solid blocks so
        // every point fails the 2-air rule → delay stays 0 (retry next tick)
        let mut w2 = flat_world();
        for dy in 0..=2i32 {
            for dz in -4..=4i32 {
                for dx in -4..=4i32 {
                    let _ = w2.set_block(8 + dx, 66 + dy, 8 + dz, STONE);
                }
            }
        }
        s.map.get_mut(&[8, 65, 8]).unwrap().delay = 0;
        let before = s.spawned_total;
        s.tick(&w2, Some([9.0, 65.0, 9.0]), &mut mobs);
        assert_eq!(
            s.spawned_total, before,
            "no spawn possible through the wall"
        );
        assert_eq!(
            s.map.get(&[8, 65, 8]).unwrap().delay,
            0,
            "failed cycle retries next tick"
        );
    }

    #[test]
    fn mob_kinds_map_to_the_dungeon_roll() {
        assert_eq!(mob_kind(SPAWNER_ZOMBIE), MobKind::Zombie);
        assert_eq!(mob_kind(SPAWNER_SKELETON), MobKind::Skeleton);
        assert_eq!(mob_kind(SPAWNER_SPIDER), MobKind::Spider);
        // out-of-range code folds to zombie (the 50% roll)
        assert_eq!(mob_kind(200), MobKind::Zombie);
    }

    #[test]
    fn remove_drops_the_entity_state() {
        let mut s = Spawners::new(13);
        s.register([1, 2, 3], SPAWNER_SPIDER);
        assert!(s.map.contains_key(&[1, 2, 3]));
        s.remove([1, 2, 3]);
        assert!(s.map.is_empty());
    }
}
