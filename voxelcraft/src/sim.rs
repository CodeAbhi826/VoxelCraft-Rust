//! The fixed-step simulation driver (Phase 6): one 20 Hz sim tick runs
//! scheduled block updates (fluids + gravity), random ticks (grass), and
//! item-entity physics + pickup. The game loop steps it with a dt
//! accumulator exactly like the particle system — same fixed-step
//! determinism the Phase-6 regression suite relies on.

use crate::entities::ItemSystem;
use crate::fluids;
use crate::ticks::{RandomTicker, TickScheduler};
use crate::world::World;

pub const SIM_HZ: f32 = 20.0;
/// random-tick blocks sampled per loaded chunk per tick (vanilla: 3)
pub const RANDOM_PER_CHUNK: usize = 3;

pub struct Sim {
    pub sched: TickScheduler,
    random: RandomTicker,
    pub items: ItemSystem,
    /// furnace block entities (Phase 7 §27) — ticked at the sim rate so
    /// COOK_TICKS = 200 means the vanilla 10 seconds
    pub furnaces: crate::furnace::Furnaces,
    /// brewing-stand block entities (Phase 7 §29) — BREW_TICKS = 400 means
    /// the vanilla 20 seconds
    pub brewing: crate::brewing::Brewings,
    /// enchanting-table block entities (§29 — reactive, not ticked)
    pub enchants: crate::enchanting::Enchants,
    acc: f32,
    /// total sim ticks executed (stats/F3/E2E)
    pub ticks: u64,
}

impl Sim {
    pub fn new(seed: u64) -> Self {
        Sim {
            sched: TickScheduler::new(),
            random: RandomTicker::new(seed),
            items: ItemSystem::new(seed ^ 0xD00_0042),
            furnaces: crate::furnace::Furnaces::default(),
            brewing: crate::brewing::Brewings::default(),
            enchants: crate::enchanting::Enchants::default(),
            acc: 0.0,
            ticks: 0,
        }
    }

    /// advance wall-clock time; runs 0..n fixed sim steps
    pub fn update(&mut self, dt: f32, world: &mut World, light: &mut crate::light::LightEngine) {
        self.acc += dt.min(0.25);
        let step = 1.0 / SIM_HZ;
        while self.acc >= step {
            self.acc -= step;
            self.step(world, light);
        }
    }

    /// ONE deterministic sim tick
    pub fn step(&mut self, world: &mut World, light: &mut crate::light::LightEngine) {
        self.ticks += 1;

        // 1. scheduled block updates in (due, insertion) order
        let due = self.sched.tick();
        for pos in due {
            let s = world.get_state(pos[0], pos[1], pos[2]);
            let b = crate::blocks::state_block(s);
            match b {
                crate::blocks::WATER => {
                    fluids::water_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                crate::blocks::SAND | crate::blocks::GRAVEL => {
                    fluids::gravity_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                crate::blocks::REDSTONE_WIRE => {
                    crate::redstone::wire_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                crate::blocks::REDSTONE_TORCH => {
                    crate::redstone::torch_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                crate::blocks::LEVER => {
                    crate::redstone::lever_tick(world, pos[0], pos[1], pos[2]);
                }
                _ => {} // stale entry: block changed since scheduling
            }
            // light follows every sim-side block edit (water/sand move
            // between non-opaque and air mostly — cheap no-ops; sand is
            // opaque and lights/darkens properly)
            let new = world.get_state(pos[0], pos[1], pos[2]);
            if new != s {
                light.on_block_changed(world, pos[0], pos[1], pos[2], s, new);
            }
        }

        // random ticks (grass spread/die — progressive §26). The light
        // hook runs per edit inside random_plant_tick's callers; grass↔dirt
        // are both opaque so the light engine no-ops there anyway.
        let chunks: Vec<(i32, i32)> = world.chunks.keys().copied().collect();
        let rt = &self.random;
        rt.tick(&chunks, self.ticks, RANDOM_PER_CHUNK, |pos| {
            fluids::random_plant_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
        });

        // 3. furnace block entities (§27): smelt progress, fuel burn, and
        // the lit/unlit world-state swap (remeshes via set_block_state's
        // dirty marking; visual-only light — FURNACE_STATE/LIT both map to
        // the FURNACE block id so the light engine sees no delta)
        let _changed = self.furnaces.tick(world);

        // 3b. brewing stands (§29): brew cycles; `completed` positions are
        // drained by game.rs for the bubble sound + stats
        self.brewing.tick();

        // 4. item entities
        self.items.tick(world);
    }

    /// player pickup collection (called by the game each frame with the
    /// player eye) — routed into the hotbar by the caller
    pub fn collect_items(&mut self, eye: [f32; 3]) -> Vec<u8> {
        self.items.collect(eye)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::*;
    use std::sync::Arc;

    fn flat_world() -> World {
        let mut w = World::new(11);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = crate::chunk::Chunk::empty();
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

    /// the Phase-6 GATE: a scripted scenario (water source + floating sand)
    /// simulated for N ticks must produce a bit-identical world every run
    #[test]
    fn sim_regression_deterministic() {
        let run = || {
            let mut w = flat_world();
            let mut sim = Sim::new(99);
            let mut light = crate::light::LightEngine::new();
            // scripted events at fixed ticks
            w.set_block_state(0, 65, 0, crate::blocks::water_state(0));
            fluids::on_block_changed(&mut sim.sched, &w, 0, 65, 0);
            w.set_block_state(8, 68, 8, SAND as u16);
            fluids::on_block_changed(&mut sim.sched, &w, 8, 68, 8);
            sim.items.drop_block(4, 66, 4, DIRT, 2, 15, 0);
            for _ in 0..400 {
                sim.step(&mut w, &mut light);
            }
            // hash the world + item positions
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for y in 60..70i32 {
                for z in -8..16i32 {
                    for x in -8..16i32 {
                        let s = w.get_state(x, y, z);
                        for b in s.to_le_bytes() {
                            h ^= b as u64;
                            h = h.wrapping_mul(0x100_0000_01b3);
                        }
                        h ^= (x as i64 as u64).wrapping_mul(31);
                        h = h.wrapping_mul(0x100_0000_01b3);
                        h ^= (z as i64 as u64).wrapping_mul(31);
                        h = h.wrapping_mul(0x100_0000_01b3);
                        h ^= y as u64;
                        h = h.wrapping_mul(0x100_0000_01b3);
                    }
                }
            }
            for it in &sim.items.items {
                for v in it.pos {
                    h ^= v.to_bits() as u64;
                    h = h.wrapping_mul(0x100_0000_01b3);
                }
            }
            (h, sim.ticks, sim.items.len(), sim.sched.pending())
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "Phase 6 gate: scripted sim → identical world+entity hash");
        // sanity: the scenario actually did something
        assert_eq!(a.1, 400, "400 sim ticks ran");
        // the scheduler SETTLES: water finished spreading, sand landed,
        // nothing keeps scheduling (no busy-loop ticks — that's the point
        // of the §12-style pruned scheduling)
        assert_eq!(a.3, 0, "scheduler settled, got {} pending", a.3);
    }
}
