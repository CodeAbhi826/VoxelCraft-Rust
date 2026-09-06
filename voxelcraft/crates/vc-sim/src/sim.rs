//! The fixed-step simulation driver (Phase 6): one 20 Hz sim tick runs
//! scheduled block updates (fluids + gravity), random ticks (grass), and
//! item-entity physics + pickup. The game loop steps it with a dt
//! accumulator exactly like the particle system — same fixed-step
//! determinism the Phase-6 regression suite relies on.

use crate::entities::{ItemSystem, XpOrbSystem};
use crate::fluids;
use crate::ticks::{RandomTicker, TickScheduler};
use vc_world::world::World;

pub const SIM_HZ: f32 = 20.0;
/// random-tick blocks sampled per loaded chunk per tick (vanilla: 3)
pub const RANDOM_PER_CHUNK: usize = 3;

/// Phase 6 §26: the simulation-distance scope of one sim tick.
///
/// Dossier Part 1 §6: "simulation distance ≠ render distance (Mojang
/// 1.18+/26.x) — no separate tick radius currently exists". 1.16.5 — our
/// accuracy target — has NO simulation distance (everything loaded ticks),
/// so this is an opt-in optimization. Defaults follow modern vanilla
/// (VERIFIED, minecraft.wiki Options.txt + Simulation distance, 2026-09):
/// range 5–32, default 12 — at 12 ≥ the default render distances every
/// loaded chunk ticks and behavior is bit-identical to the 1.16.5 rule.
///
/// Gating semantics (wiki-anchored, documented):
/// * random ticks: only chunks inside the ring;
/// * scheduled ticks: WATER/SAND/GRAVEL and tick-machinery (pistons,
///   observers, dispensers, hoppers) outside the ring defer one tick —
///   "pure redstone circuits continue to function infinitely far away"
///   (wiki), so wire/torch/repeater/comparator/lever entries always run;
/// * entities (mobs/villagers/items) outside the ring freeze — 1.18+
///   semantics; spawning is clamped to the ring;
/// * block entities (furnaces/brewing) keep 1.16.5 always-tick semantics
///   (cheap, few, and freezing a furnace mid-smelt is an observable
///   gameplay regression — documented adaptation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickScope {
    pub center: (i32, i32),
    pub radius: i32,
}

impl TickScope {
    /// the un-gated scope (1.16.5 semantics — everything loaded ticks)
    pub fn everything() -> Self {
        TickScope {
            center: (0, 0),
            radius: i32::MAX,
        }
    }
    #[inline]
    pub fn chunk_in(&self, cx: i32, cz: i32) -> bool {
        // saturating_abs: the everything()-scope must never overflow on
        // extreme coordinates
        cx.wrapping_sub(self.center.0)
            .saturating_abs()
            .max(cz.wrapping_sub(self.center.1).saturating_abs())
            <= self.radius
    }
    #[inline]
    pub fn block_in(&self, x: i32, z: i32) -> bool {
        self.chunk_in(x.div_euclid(16), z.div_euclid(16))
    }
}

pub struct Sim {
    pub sched: TickScheduler,
    random: RandomTicker,
    pub items: ItemSystem,
    /// Phase E1: XP orbs — attraction physics + 10/s pickup, collected
    /// XP drained by the game layer
    pub xp_orbs: XpOrbSystem,
    /// furnace block entities (Phase 7 §27) — ticked at the sim rate so
    /// COOK_TICKS = 200 means the vanilla 10 seconds
    pub furnaces: vc_gameplay::furnace::Furnaces,
    /// brewing-stand block entities (Phase 7 §29) — BREW_TICKS = 400 means
    /// the vanilla 20 seconds
    pub brewing: vc_gameplay::brewing::Brewings,
    /// enchanting-table block entities (§29 — reactive, not ticked)
    pub enchants: vc_gameplay::enchanting::Enchants,
    /// villager NPCs (Phase 7 §27/§29, Phase 5 trading depth): wander
    /// AI + tiered trade state
    pub villagers: vc_gameplay::villagers::Villagers,
    /// monster spawners (Phase 5 §27): dungeon block entities
    pub spawners: vc_gameplay::spawners::Spawners,
    /// mobs (Phase 2): spawn/AI/physics + arrows; hits, deaths and
    /// explosions queue here for the game layer to drain
    pub mobs: vc_gameplay::mobs::MobSystem,
    /// Phase E1: the ender-dragon fight (End dimension only)
    pub dragon: vc_gameplay::dragon::DragonSystem,
    /// containers (Phase 3): chests/dispensers/droppers/hoppers
    pub containers: crate::containers::Containers,
    /// dispenser/dropper previous powered state (rising-edge detect)
    dispenser_prev: std::collections::HashMap<[i32; 3], bool>,
    /// dispenser/dropper eject countdowns (VERIFIED 4 game tick delay)
    pending_eject: std::collections::HashMap<[i32; 3], u64>,
    /// hopper transfer cooldowns (VERIFIED 8 game ticks)
    hopper_cd: std::collections::HashMap<[i32; 3], u64>,
    acc: f32,
    /// total sim ticks executed (stats/F3/E2E)
    pub ticks: u64,
    /// Phase E1: dragon-fight events queued for the game layer (world
    /// edits + XP + projectiles live there)
    pub dragon_events: Vec<vc_gameplay::dragon::DragonEvent>,
}

impl Sim {
    pub fn new(seed: u64) -> Self {
        Sim {
            sched: TickScheduler::new(),
            random: RandomTicker::new(seed),
            items: ItemSystem::new(seed ^ 0xD00_0042),
            xp_orbs: XpOrbSystem::new(seed ^ 0x0DB_5EED),
            furnaces: vc_gameplay::furnace::Furnaces::default(),
            brewing: vc_gameplay::brewing::Brewings::default(),
            enchants: vc_gameplay::enchanting::Enchants::default(),
            villagers: vc_gameplay::villagers::Villagers::new(seed ^ 0x315_7A9),
            spawners: vc_gameplay::spawners::Spawners::new(seed ^ 0x5C_0DE5),
            mobs: vc_gameplay::mobs::MobSystem::new(seed ^ 0x5C_0DE),
            dragon: vc_gameplay::dragon::DragonSystem::new(seed ^ 0xDA60_0005),
            containers: crate::containers::Containers::default(),
            dispenser_prev: std::collections::HashMap::new(),
            pending_eject: std::collections::HashMap::new(),
            hopper_cd: std::collections::HashMap::new(),
            acc: 0.0,
            ticks: 0,
            dragon_events: Vec::new(),
        }
    }

    /// advance wall-clock time; runs 0..n fixed sim steps
    pub fn update(
        &mut self,
        dt: f32,
        world: &mut World,
        light: &mut vc_world::light::LightEngine,
        scope: &TickScope,
    ) {
        self.acc += dt.min(0.25);
        let step = 1.0 / SIM_HZ;
        while self.acc >= step {
            self.acc -= step;
            self.step(world, light, scope);
        }
    }

    /// ONE deterministic sim tick (`scope` = the simulation-distance ring;
    /// `TickScope::everything()` = 1.16.5 behavior)
    pub fn step(
        &mut self,
        world: &mut World,
        light: &mut vc_world::light::LightEngine,
        scope: &TickScope,
    ) {
        self.ticks += 1;

        // 1. scheduled block updates in (due, insertion) order.
        // Phase 6 §26: entries OUTSIDE the simulation ring defer one tick —
        // except pure redstone (wiki: "pure redstone circuits continue to
        // function infinitely far away"). Deferred entries re-queue, so
        // walking back into range resumes them exactly where they left.
        let due = self.sched.tick();
        let mut deferred: Vec<[i32; 3]> = Vec::new();
        for pos in due {
            if !scope.block_in(pos[0], pos[2]) {
                let b = vc_blocks::blocks::state_block(world.get_state(pos[0], pos[1], pos[2]));
                let pure_redstone = matches!(
                    b,
                    vc_blocks::blocks::REDSTONE_WIRE
                        | vc_blocks::blocks::REDSTONE_TORCH
                        | vc_blocks::blocks::REPEATER
                        | vc_blocks::blocks::COMPARATOR
                        | vc_blocks::blocks::LEVER
                );
                if !pure_redstone {
                    deferred.push(pos);
                    continue;
                }
            }
            let s = world.get_state(pos[0], pos[1], pos[2]);
            let b = vc_blocks::blocks::state_block(s);
            match b {
                vc_blocks::blocks::WATER => {
                    fluids::water_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::SAND | vc_blocks::blocks::GRAVEL => {
                    fluids::gravity_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::REDSTONE_WIRE => {
                    crate::redstone::wire_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::REDSTONE_TORCH => {
                    crate::redstone::torch_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::LEVER => {
                    crate::redstone::lever_tick(world, pos[0], pos[1], pos[2]);
                }
                // ---- Phase 3 components ----
                vc_blocks::blocks::REPEATER => {
                    crate::redstone::repeater_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::COMPARATOR => {
                    let containers = &self.containers;
                    crate::redstone::comparator_tick(
                        world,
                        &mut self.sched,
                        containers,
                        pos[0],
                        pos[1],
                        pos[2],
                    );
                }
                vc_blocks::blocks::PISTON | vc_blocks::blocks::STICKY_PISTON => {
                    crate::redstone::piston_tick(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::OBSERVER => {
                    crate::redstone::observer_pulse(world, &mut self.sched, pos[0], pos[1], pos[2]);
                }
                vc_blocks::blocks::DISPENSER | vc_blocks::blocks::DROPPER => {
                    // rising-edge detection (VERIFIED: eject one item per
                    // activation, 4 game ticks later)
                    let powered = crate::redstone::dispenser_tick(world, pos[0], pos[1], pos[2]);
                    let prev = self.dispenser_prev.insert(pos, powered);
                    if powered && prev != Some(true) {
                        self.pending_eject
                            .insert(pos, crate::redstone::DISPENSER_DELAY);
                    }
                }
                vc_blocks::blocks::HOPPER => {
                    // hopper work happens in the hopper pass below; the
                    // scheduled entry just refreshes its enable state
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
        for pos in deferred {
            self.sched.schedule(pos, 1);
        }

        // random ticks (grass spread/die — progressive §26). The light
        // hook runs per edit inside random_plant_tick's callers; grass↔dirt
        // are both opaque so the light engine no-ops there anyway.
        // Phase 6 §26: only chunks inside the simulation ring roll.
        let chunks: Vec<(i32, i32)> = world
            .chunks
            .keys()
            .copied()
            .filter(|c| scope.chunk_in(c.0, c.1))
            .collect();
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

        // 4. item entities (Phase 6 §26: frozen outside the simulation ring)
        self.items.tick(world, scope.center, scope.radius);

        // 4b. Phase E1: XP orbs — attraction + pickup (the player anchor
        // rides the mob system's anchor; None when no player)
        let feet = self.mobs.player.map(|p| p);
        self.xp_orbs.tick(world, feet);

        // 4c. Phase E1: the ender-dragon fight (End only; the fight is
        // dormant elsewhere — begin_fight spawns it on arrival)
        if world.dimension == vc_world::world::Dimension::End {
            let evs = self.dragon.tick(world, feet);
            self.dragon_events.extend(evs);
        }

        // 5. villagers (§27): wander decisions + walking physics + the
        // twice-daily trade restock clock (sim.ticks is the day clock)
        // (Phase 6 §26: villagers outside the ring freeze — restock included)
        self.villagers
            .tick(world, self.ticks, scope.center, scope.radius);

        // 6. mobs (Phase 2): spawning + AI + arrows. Hits on the player,
        // deaths, and explosions queue inside for game.rs to drain (the
        // game layer owns damage gating, drops, and world edits).
        // (Phase 6 §26: AI frozen + spawning clamped inside the ring)
        self.mobs.tick(world, scope.center, scope.radius);

        // 6b. spawners (Phase 5 §27): dungeon block entities — activation
        // gate, delay, 4-attempt cycles, 6-mob cap
        self.spawners.tick(world, self.mobs.player, &mut self.mobs);

        // 7. hoppers (Phase 3): collect items above (VERIFIED: hoppers
        // collect every game tick, then 8gt cooldown), push one item per
        // 8gt into the container below when not redstone-locked
        // (Phase 6 §26: hopper positions outside the ring defer)
        self.hopper_pass(world, scope);

        // 8. dispenser/dropper ejects (4 game tick countdown — VERIFIED)
        let ejects: Vec<[i32; 3]> = {
            let mut fire = Vec::new();
            let mut done = Vec::new();
            for (pos, t) in self.pending_eject.iter_mut() {
                *t = t.saturating_sub(1);
                if *t == 0 {
                    fire.push(*pos);
                    done.push(*pos);
                }
            }
            for p in done {
                self.pending_eject.remove(&p);
            }
            fire
        };
        for pos in ejects {
            use vc_blocks::blocks::DISPENSER;
            let b = world.get_block(pos[0], pos[1], pos[2]);
            let facing = if b == DISPENSER {
                vc_blocks::blocks::dispenser_decode(world.get_state(pos[0], pos[1], pos[2]))
            } else {
                vc_blocks::blocks::dropper_decode(world.get_state(pos[0], pos[1], pos[2]))
            };
            let [fx, fy, fz] = vc_blocks::blocks::full_facing_vec(facing);
            // take the first item out of the container
            if let Some(inv) = self.containers.get_mut(&pos) {
                if let Some(i) = inv.first_item() {
                    let block = inv.slots[i].block;
                    inv.slots[i] = vc_inventory::inventory::ItemStack::EMPTY;
                    self.items
                        .drop_block(pos[0] + fx, pos[1] + fy, pos[2] + fz, block, 2, 15, 0);
                }
            }
        }
    }

    /// Phase 3 hopper pass: item collection + down-transfer.
    /// v1 scope (documented): transfers push DOWN only (chest, dispenser,
    /// dropper, hopper below); furnace input slots arrive with the
    /// container unification later.
    fn hopper_pass(&mut self, world: &World, scope: &TickScope) {
        use vc_blocks::blocks::*;
        // A. collect item entities resting on a hopper
        // (item entities outside the ring are frozen and never rest-hopped)
        let mut collected: Vec<usize> = Vec::new();
        for (i, it) in self.items.items.iter().enumerate() {
            let bx = it.pos[0].floor() as i32;
            let by = (it.pos[1] - 0.25).floor() as i32;
            let bz = it.pos[2].floor() as i32;
            if !scope.block_in(bx, bz) {
                continue;
            }
            let s = world.get_state(bx, by, bz);
            if state_block(s) == HOPPER && crate::redstone::hopper_enabled(world, bx, by, bz) {
                let slot = it.block;
                let inv = self.containers.entry([bx, by, bz], HOPPER);
                if inv.add(slot, 1) == 0 {
                    collected.push(i);
                }
            }
        }
        for i in collected.iter().rev() {
            self.items.items.remove(*i);
        }
        // B. push one item per 8gt from each hopper into the container below
        let hopper_positions: Vec<[i32; 3]> = self
            .containers
            .map
            .iter()
            .filter(|(pos, _)| state_block(world.get_state(pos[0], pos[1], pos[2])) == HOPPER)
            .map(|(pos, _)| *pos)
            .collect();
        for pos in hopper_positions {
            if !scope.block_in(pos[0], pos[2]) {
                continue; // outside the simulation ring: frozen this tick
            }
            if !crate::redstone::hopper_enabled(world, pos[0], pos[1], pos[2]) {
                continue;
            }
            let cd = self.hopper_cd.entry(pos).or_insert(0);
            if *cd > 0 {
                *cd -= 1;
                continue;
            }
            // is there an item to move?
            let take = self
                .containers
                .get(&pos)
                .and_then(|inv| inv.first_item())
                .map(|i| (i, self.containers.get(&pos).unwrap().slots[i].block));
            let Some((slot_i, block)) = take else {
                continue;
            };
            // the container below
            let below = [pos[0], pos[1] - 1, pos[2]];
            let below_block = state_block(world.get_state(below[0], below[1], below[2]));
            if matches!(below_block, CHEST | DISPENSER | DROPPER | HOPPER) {
                let inv = self.containers.entry(below, below_block);
                if inv.add(block, 1) == 0 {
                    // moved: clear the hopper slot, start the cooldown
                    if let Some(hinv) = self.containers.get_mut(&pos) {
                        hinv.slots[slot_i] = vc_inventory::inventory::ItemStack::EMPTY;
                    }
                    self.hopper_cd.insert(pos, crate::redstone::HOPPER_COOLDOWN);
                }
            }
        }
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
    use std::sync::Arc;
    use vc_blocks::blocks::*;

    fn flat_world() -> World {
        let mut w = World::new(11);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
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
            let mut light = vc_world::light::LightEngine::new();
            // scripted events at fixed ticks
            w.set_block_state(0, 65, 0, vc_blocks::blocks::water_state(0));
            fluids::on_block_changed(&mut sim.sched, &w, 0, 65, 0);
            w.set_block_state(8, 68, 8, SAND as u16);
            fluids::on_block_changed(&mut sim.sched, &w, 8, 68, 8);
            sim.items.drop_block(4, 66, 4, DIRT, 2, 15, 0);
            for _ in 0..400 {
                sim.step(&mut w, &mut light, &TickScope::everything());
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
        assert_eq!(
            a, b,
            "Phase 6 gate: scripted sim → identical world+entity hash"
        );
        // sanity: the scenario actually did something
        assert_eq!(a.1, 400, "400 sim ticks ran");
        // the scheduler SETTLES: water finished spreading, sand landed,
        // nothing keeps scheduling (no busy-loop ticks — that's the point
        // of the §12-style pruned scheduling)
        assert_eq!(a.3, 0, "scheduler settled, got {} pending", a.3);
    }

    // ------------------------------------------- Phase 6 §26: sim distance --

    #[test]
    fn tick_scope_ring_math() {
        let s = TickScope {
            center: (3, -2),
            radius: 2,
        };
        assert!(s.chunk_in(3, -2));
        assert!(s.chunk_in(5, 0));
        assert!(s.chunk_in(1, -4));
        assert!(!s.chunk_in(6, -2), "Chebyshev: 3 chunks east is out");
        assert!(!s.chunk_in(3, 1));
        // block → chunk mapping (negative coords floor correctly)
        assert!(s.block_in(48, -32)); // chunk (3,-2)
        assert!(!s.block_in(-1, 0)); // chunk (-1,0), 4 away
                                     // the everything() scope never excludes anything (incl. extremes)
        let all = TickScope::everything();
        assert!(all.chunk_in(i32::MIN, i32::MIN));
        assert!(all.chunk_in(i32::MAX, i32::MAX));
        assert!(all.block_in(-1, -1));
    }

    /// default radius 12 ≥ every loaded chunk at the default render
    /// distances → 1.16.5-identical behavior (the point of the default)
    #[test]
    fn default_scope_covers_default_render_distance() {
        let all = TickScope::everything();
        let default_ring = TickScope {
            center: (0, 0),
            radius: 12,
        };
        for dz in -11..=11 {
            for dx in -11..=11 {
                let r = default_ring.chunk_in(dx, dz);
                assert!(r, "12-ring must cover the loaded set (rd 10 + 1)");
                assert!(all.chunk_in(dx, dz));
            }
        }
    }

    /// water scheduled outside the ring defers; a pure-redstone entry runs
    /// regardless (wiki: "pure redstone circuits continue to function
    /// infinitely far away")
    #[test]
    fn scheduled_ticks_defer_outside_ring_but_redstone_runs() {
        let mut w = flat_world();
        let mut sim = Sim::new(7);
        let mut light = vc_world::light::LightEngine::new();
        // water at (0, 65, 0) — inside; water at (0, 65, 16) is in chunk
        // (0, 1), 1 away → outside a radius-0 ring
        w.set_block_state(0, 65, 0, water_state(0));
        w.set_block_state(0, 65, 16, water_state(0));
        fluids::on_block_changed(&mut sim.sched, &w, 0, 65, 0);
        fluids::on_block_changed(&mut sim.sched, &w, 0, 65, 16);
        // wire + torch far away: torch at (16, 65, 0) (chunk (1, 0), outside
        // the radius-0 ring) — pure redstone must still run
        w.set_block_state(16, 65, 0, REDSTONE_TORCH as u16);
        crate::redstone::on_block_changed(&mut sim.sched, &w, 16, 65, 0);
        let tiny = TickScope {
            center: (0, 0),
            radius: 0,
        };
        for _ in 0..20 {
            sim.step(&mut w, &mut light, &tiny);
        }
        // the in-ring water flowed (has a scheduled follow-up or spread);
        // the out-of-ring water never flowed: still a single source block
        let out_states: Vec<u16> = (0..16).map(|x| w.get_state(x, 65, 17)).collect();
        assert!(
            out_states.iter().all(|&s| s == AIR as u16),
            "out-of-ring water column untouched (got {out_states:?})"
        );
        // ...while the deferred out-of-ring entry keeps re-queueing
        assert!(
            sim.sched.pending() >= 1,
            "deferred out-of-ring entries re-queued"
        );
    }

    /// items outside the ring freeze (age + physics)
    #[test]
    fn items_freeze_outside_ring() {
        let mut w = flat_world();
        let mut sim = Sim::new(7);
        let mut light = vc_world::light::LightEngine::new();
        // two dirt drops: one above the floor at (8, 66, 8) (chunk (0,0)),
        // one at (8, 66, 24) (chunk (0,1)) — same world, different rings
        sim.items.drop_block(8, 66, 8, DIRT, 2, 15, 0);
        sim.items.drop_block(8, 66, 24, DIRT, 2, 15, 0);
        let tiny = TickScope {
            center: (0, 0),
            radius: 0,
        };
        for _ in 0..40 {
            sim.step(&mut w, &mut light, &tiny);
        }
        let a = sim.items.items.iter().find(|i| i.pos[2] < 16.0).unwrap();
        let b = sim.items.items.iter().find(|i| i.pos[2] >= 16.0).unwrap();
        assert!(a.age > 0, "in-ring item ages");
        assert_eq!(b.age, 0, "out-of-ring item frozen (age 0)");
        assert!(
            (a.pos[1] - b.pos[1]).abs() > 0.001,
            "in-ring item fell; frozen one did not"
        );
    }
}
