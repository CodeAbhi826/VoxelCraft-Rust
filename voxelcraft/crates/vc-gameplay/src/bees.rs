//! 1.15 (Buzzy Bees) — the hive lifecycle + per-instance bee state.
//!
//! VERIFIED 2026-09-08 from the raw captures scripts/v115_page_*.json
//! (w/Bee, w/Beehive, w/Bee_nest); the value contract is
//! docs/research/phase-v115-1.15-research.md. Everything below cites
//! the live quotes it implements.
//!
//! Architecture (the engine's established split):
//! - `BeeState` rides each bee MOB (the equine-state pattern) and the
//!   bee AI arm in mobs.rs drives the phase machine.
//! - `HiveSystem` lives on the Sim and owns the bees INSIDE hives
//!   (registered lazily from generated nests — 2-3 bees, VERIFIED
//!   w/Bee: "Naturally generated bee nests generate with 2-3 bees in
//!   them"), the work countdowns, and the day-release gate.
//! - World edits (honey-level state writes, crop pollination, bee
//!   releases) queue here and are drained by the game layer, exactly
//!   like the turtle-egg / campfire-done queues.

use std::collections::HashMap;
use vc_blocks::blocks::*;
use vc_rng::rng::Rng;
use vc_world::world::World;

// ------------------------------------------------------- constants --
/// one bee nest/beehive houses up to 3 bees (VERIFIED w/Beehive:
/// "Bee nests and beehives can house up to 3 bees at a time")
pub const HIVE_CAPACITY: usize = 3;
/// "After circling a flower for more than 400 game ticks (20 seconds),
/// a bee collects nectar" (VERIFIED w/Bee)
pub const FLOWER_CIRCLE_TICKS: i32 = 400;
/// "It takes about 2 minutes for the bee to do this [make honey]"
/// + "They stay in their nest or hive for at least 2400 game ticks
/// (2 minutes) before coming back out" (VERIFIED w/Bee)
pub const HIVE_WORK_TICKS: i32 = 2400;
/// "dies approximately one minute later" after losing its stinger
/// (VERIFIED w/Bee) — 1200 ticks
pub const STING_DEATH_TICKS: i32 = 1200;
/// "Anger duration is randomly selected between 20 and 39 seconds,
/// inclusive" (VERIFIED w/Bee) — 400..=780 ticks
pub const ANGER_TICKS_MIN: i32 = 400;
pub const ANGER_TICKS_MAX: i32 = 780;
/// "A bee can fertilize plants 10 times each time they have nectar.
/// There is an approximately 5% chance each tick to attempt
/// fertilization." (VERIFIED w/Bee)
pub const NECTAR_CHARGES: u8 = 10;
pub const POLLINATE_CHANCE: f32 = 0.05;
/// "unless a campfire is placed within five blocks below the hive"
/// (VERIFIED w/Bee — the pacify range)
pub const CAMPFIRE_PACIFY_BLOCKS: i32 = 5;
/// "To pollinate a plant, a bee must be 1 to 2 blocks directly above
/// the plant" (VERIFIED w/Bee)
pub const POLLINATE_MIN_ABOVE: i32 = 1;
pub const POLLINATE_MAX_ABOVE: i32 = 2;
/// flower-search radius for a hovering bee (the engine's standard
/// mob scan radius — vanilla's 8-block flower "reachable" scan is
/// pathfinding-gated; disclosed)
pub const FLOWER_SEARCH_R: i32 = 10;
/// the hive-scan band: generated trees (and their nests) live above
/// the terrain height — a wide band keeps the lazy registration cheap
pub const HIVE_SCAN_Y_MIN: i32 = 40;
pub const HIVE_SCAN_Y_MAX: i32 = 120;

// bee phases (BeeState.phase)
/// idle hover near the hive
pub const PH_HOVER: u8 = 0;
/// traveling to a flower
pub const PH_TO_FLOWER: u8 = 1;
/// circling the flower (the 400-tick visit)
pub const PH_CIRCLE: u8 = 2;
/// returning to the hive WITH nectar (the honey trip)
pub const PH_TO_HIVE: u8 = 3;
/// returning for the night (no nectar)
pub const PH_NIGHT: u8 = 4;
/// angry chase (the swarm state)
pub const PH_ANGRY: u8 = 5;

// --------------------------------------------------- per-mob state --
/// Per-instance bee lifecycle state (rides the Mob — the EquineState
/// pattern). The phase machine: HOVER → TO_FLOWER → CIRCLE (400 t) →
/// TO_HIVE (enter, work 2400 t, exit → honey +1) → HOVER. ANGRY
/// overrides everything until the anger counter runs out; NIGHT
/// overrides the trip at dusk. A stung bee never re-enters its hive
/// ("does not retreat to its nest (even at night)", VERIFIED w/Bee)
/// and dies on the 1200-tick timer.
#[derive(Clone, Debug)]
pub struct BeeState {
    /// home hive block position (None = homeless — searches; the
    /// "wandering generally in a northwest direction" class, VERIFIED)
    pub hive: Option<[i32; 3]>,
    /// current flower target (block position)
    pub flower: Option<[i32; 3]>,
    /// phase (PH_* above)
    pub phase: u8,
    /// phase countdown — the 400-tick circle timer (and the
    /// flower-pick retry stagger while hovering)
    pub timer: i32,
    /// carrying nectar (pollen) — the fertilize payload
    pub nectar: bool,
    /// fertilizations left this load (10, VERIFIED)
    pub pollinations: u8,
    /// lost the stinger — the death timer runs
    pub stung: bool,
    /// death countdown after the sting (1200, VERIFIED)
    pub death_t: i32,
    /// anger countdown (400..=780, VERIFIED); 0 = calm
    pub anger_t: i32,
    /// love-mode window (fed a flower — breeding; the fox pattern)
    pub love_t: i32,
    /// this bee is a baby (grows up after MATURITY_TICKS)
    pub baby: bool,
    /// maturity countdown while a baby
    pub maturity_t: i32,
    /// set by the AI arm when the bee reached its hive this tick —
    /// MobSystem::tick removes the mob and hands it to the game layer
    /// (the enter queue)
    pub arrived: bool,
}

impl BeeState {
    /// a fresh released/adult bee (homeless until the game layer
    /// assigns the hive it was released from)
    pub fn new() -> Self {
        BeeState {
            hive: None,
            flower: None,
            phase: PH_HOVER,
            timer: 0,
            nectar: false,
            pollinations: 0,
            stung: false,
            death_t: 0,
            anger_t: 0,
            love_t: 0,
            baby: false,
            maturity_t: 0,
            arrived: false,
        }
    }
}

/// a bee stored INSIDE a hive (not a live mob). "Bees in a nest/hive
/// retain their data (health, name, etc.)" (VERIFIED w/Bee) — the
/// engine keeps health + nectar + the work countdown + the anger flag.
#[derive(Clone, Debug)]
pub struct StoredBee {
    pub health: f32,
    /// entered carrying nectar (converts to honey on exit)
    pub nectar: bool,
    /// work countdown (2400 on enter; releases + increments honey at 0
    /// once day breaks — the "waits for daylight" rule, VERIFIED)
    pub work_t: i32,
    /// released angry (the harvest/break case)
    pub angry: bool,
}

impl StoredBee {
    pub fn new(health: f32, nectar: bool) -> Self {
        StoredBee {
            health,
            nectar,
            work_t: HIVE_WORK_TICKS,
            angry: false,
        }
    }
}

/// per-hive data (keyed by block position)
#[derive(Clone, Debug, Default)]
pub struct HiveData {
    /// bees currently inside
    pub bees: Vec<StoredBee>,
    /// true = a naturally generated nest (registered with 2-3 bees);
    /// false = a player-placed hive (starts empty)
    pub natural: bool,
}

// ------------------------------------------------------ the hives --
/// The hive registry + lifecycle clock. Ticked by the Sim at the sim
/// rate; world writes queue out for the game layer.
pub struct HiveSystem {
    pub hives: HashMap<[i32; 3], HiveData>,
    rng: Rng,
    scan_t: i32,
    /// queued honey-level writes: (position, new level) — the game
    /// layer applies the state write + remesh
    pub pending_hive_levels: Vec<([i32; 3], u8)>,
    /// queued bee releases: (hive position, the stored bee) — the game
    /// layer spawns the mob at the hive front with this data
    pub releases: Vec<([i32; 3], StoredBee)>,
    /// total hives registered (E2E/stat)
    pub registered_total: u64,
    /// total bees released (E2E/stat)
    pub released_total: u64,
    /// total honey-level increments applied (E2E/stat)
    pub honey_total: u64,
}

impl HiveSystem {
    pub fn new(seed: u64) -> Self {
        HiveSystem {
            hives: HashMap::new(),
            rng: Rng::new(seed),
            scan_t: 0,
            pending_hive_levels: Vec::new(),
            releases: Vec::new(),
            registered_total: 0,
            released_total: 0,
            honey_total: 0,
        }
    }

    /// ONE sim tick: lazy registration scans + in-hive work clocks +
    /// the day-release gate.
    pub fn tick(&mut self, world: &World, sim_center: (i32, i32), sim_radius: i32, is_day: bool) {
        self.scan_t += 1;

        // 1. lazy registration — one chunk in the ring every 20 ticks,
        //    ROUND-ROBIN (scan_t/20 rotates through the whole 17x17
        //    ring in 289 scans ≈ 4.8 min — a random pick could starve
        //    a chunk; the rotation guarantees every chunk registers
        //    once the player lingers). Generated nests (BEE_NEST)
        //    register with 2-3 bees; player-placed hives (BEEHIVE)
        //    register empty. The deterministic-count rule: 2 or 3
        //    (VERIFIED w/Bee).
        if self.scan_t % 20 == 0 {
            let k = (self.scan_t / 20) as i32;
            let cx = sim_center.0 + (k % 17) - 8;
            let cz = sim_center.1 + ((k / 17) % 17) - 8;
            let in_ring = cx.wrapping_sub(sim_center.0).saturating_abs().max(
                cz.wrapping_sub(sim_center.1).saturating_abs(),
            ) <= sim_radius;
            if in_ring && world.chunk((cx, cz)).is_some() {
                for y in HIVE_SCAN_Y_MIN..=HIVE_SCAN_Y_MAX {
                    for lx in 0..16usize {
                        for lz in 0..16usize {
                            let b = world.get_block(cx * 16 + lx as i32, y, cz * 16 + lz as i32);
                            if (b == BEE_NEST || b == BEEHIVE)
                                && !self.hives.contains_key(&[
                                    cx * 16 + lx as i32,
                                    y,
                                    cz * 16 + lz as i32,
                                ])
                            {
                                let pos = [
                                    cx * 16 + lx as i32,
                                    y,
                                    cz * 16 + lz as i32,
                                ];
                                let natural = b == BEE_NEST;
                                let mut data = HiveData { bees: Vec::new(), natural };
                                if natural {
                                    // "generate with 2-3 bees in them"
                                    let n = 2 + (self.rng.next_range(2)) as usize;
                                    for _ in 0..n {
                                        // staggered work timers so the
                                        // morning exit isn't simultaneous
                                        let mut sb = StoredBee::new(10.0, false);
                                        sb.work_t = (self.rng.next_range(2400) as i32).max(1);
                                        data.bees.push(sb);
                                    }
                                }
                                self.hives.insert(pos, data);
                                self.registered_total += 1;
                            }
                        }
                    }
                }
            }
        }

        // 2. in-hive work clocks + the day-release gate. A bee that
        //    entered with nectar exits at work_t == 0 (day required —
        //    "waits for daylight with no rain ... then exits to go
        //    collect more nectar", VERIFIED w/Beehive) and the honey
        //    level rises: "When a bee that has nectar enters and then
        //    leaves its nest or hive, the honey level ... is increased
        //    by one; there is a 1% chance it is increased by two."
        let mut levels: Vec<([i32; 3], u8)> = Vec::new();
        let mut releases: Vec<([i32; 3], StoredBee)> = Vec::new();
        for (pos, data) in self.hives.iter_mut() {
            let mut i = 0;
            while i < data.bees.len() {
                let sb = &mut data.bees[i];
                if sb.work_t > 0 {
                    sb.work_t -= 1;
                    i += 1;
                    continue;
                }
                // work done: exits only in daylight (angry bees exit
                // immediately — the harvest swarm, queued by anger())
                if is_day || sb.angry {
                    let done = data.bees.remove(i);
                    if done.nectar {
                        let bump = if self.rng.next_range(100) < 1 { 2u8 } else { 1u8 };
                        levels.push((*pos, bump));
                        self.honey_total += 1;
                    }
                    releases.push((*pos, done));
                    self.released_total += 1;
                } else {
                    i += 1; // waits for daylight
                }
            }
        }
        // the level bumps read + write the block state — game layer
        self.pending_hive_levels.extend(levels);
        self.releases.extend(releases);

        // 3. cleanup: hives whose block vanished (broken) drop out —
        //    their bees were already released angry by the game layer's
        //    break handler (bees_hive_broken)
        let dead: Vec<[i32; 3]> = self
            .hives
            .keys()
            .filter(|pos| {
                let b = world.get_block(pos[0], pos[1], pos[2]);
                b != BEE_NEST && b != BEEHIVE
            })
            .copied()
            .collect();
        for pos in dead {
            self.hives.remove(&pos);
        }
    }

    /// a bee MOB reached its hive — store it (the game layer calls
    /// this while draining `pending_bee_enters`; it also removes the
    /// mob from the list). Capacity is 3 (VERIFIED); a full hive
    /// refuses (the bee stays out — returns false).
    pub fn enter(&mut self, hive: [i32; 3], health: f32, nectar: bool) -> bool {
        let Some(data) = self.hives.get_mut(&hive) else {
            return false;
        };
        if data.bees.len() >= HIVE_CAPACITY {
            return false;
        }
        data.bees.push(StoredBee::new(health, nectar));
        true
    }

    /// anger the hive family: stored bees flag angry + queue release
    /// ("Collecting a honeycomb or honey bottle from a nest or hive
    /// causes the bees that are currently in that nest or hive to
    /// leave and swarm the player", VERIFIED w/Bee) and out bees get
    /// their anger timers set by the mobs layer (anger_bees_near).
    /// The campfire check happens BEFORE the game layer calls this
    /// (campfire_pacifies).
    pub fn anger(&mut self, hive: [i32; 3]) -> usize {
        let mut n = 0;
        if let Some(data) = self.hives.get_mut(&hive) {
            for sb in data.bees.iter_mut() {
                if !sb.angry {
                    sb.angry = true;
                    sb.work_t = 0; // exits on the next tick
                    n += 1;
                }
            }
        }
        n
    }

    /// the harvest/break pacify check: "unless a campfire is placed
    /// within five blocks below the hive" (VERIFIED w/Bee) — a LIT
    /// campfire (or plain fire) anywhere in the 5-block column under
    /// the hive, any of the 4 offsets around it.
    pub fn campfire_pacifies(world: &World, hive: [i32; 3]) -> bool {
        for dy in 1..=CAMPFIRE_PACIFY_BLOCKS {
            for (dx, dz) in [(0i32, 0i32), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                let s = world.get_state(hive[0] + dx, hive[1] - dy, hive[2] + dz);
                if campfire_lit(s) {
                    return true;
                }
            }
        }
        false
    }

    /// the honey level of the hive block AT a position (reads the
    /// live world state — 0 for non-hives)
    pub fn honey_level_at(world: &World, hive: [i32; 3]) -> u8 {
        honey_level(world.get_state(hive[0], hive[1], hive[2]))
    }

    /// the spawn point for a released bee: the cell under the hive
    /// (bees "exit only from the front" — the engine's no-facing
    /// simplification puts the front everywhere; the under-cell is
    /// the guaranteed open side for tree nests)
    pub fn release_pos(world: &World, hive: [i32; 3]) -> [f32; 3] {
        for dy in [0i32, -1, -2] {
            let y = hive[1] + dy;
            if world.get_block(hive[0], y, hive[2]) == AIR {
                return [
                    hive[0] as f32 + 0.5,
                    y as f32 + 0.3,
                    hive[2] as f32 + 0.5,
                ];
            }
        }
        [hive[0] as f32 + 0.5, hive[1] as f32 + 0.3, hive[2] as f32 + 0.5]
    }
}

// ------------------------------------------------- flower scanning --
/// is this block a bee-attracting flower? (the engine's flower set —
/// vanilla's 1/2-block-tall flowers; VERIFIED w/Bee §Pollinating:
/// "attracted to flowers ... which are the valid plants". The
/// flowering-azalea/cherry/mangrove/petals rows are post-1.16.5 or
/// absent from the engine — documented scope.)
#[inline]
pub fn is_flower(b: u16) -> bool {
    matches!(
        b,
        FLOWER_RED | FLOWER_YELLOW | OXEYE_DAISY | CORNFLOWER | LILY_OF_THE_VALLEY
    )
}

/// find a flower block within `r` of the bee (spiral-ish random probe
/// — cheap: samples a bounded box, picks the nearest). Returns the
/// block position.
pub fn find_flower(world: &World, from: [f32; 3], r: i32) -> Option<[i32; 3]> {
    let cx = from[0].floor() as i32;
    let cy = from[1].floor() as i32;
    let cz = from[2].floor() as i32;
    let mut best: Option<([i32; 3], f32)> = None;
    for dy in -3..=4 {
        for dx in -r..=r {
            for dz in -r..=r {
                let b = world.get_block(cx + dx, cy + dy, cz + dz);
                if is_flower(b) {
                    let d = (dx * dx + dz * dz) as f32 + (dy * dy) as f32;
                    if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                        best = Some(([cx + dx, cy + dy, cz + dz], d));
                    }
                }
            }
        }
    }
    best.map(|(p, _)| p)
}

/// the pollination probe: is there a pollinatable CROP 1-2 blocks
/// directly below the bee? The engine's growable-crop set is the
/// sweet berry bush (age 0..=3; the wheat/potato/carrot rows of
/// vanilla pollination need the farming subsystem — deferred,
/// documented). Returns the crop position + its current age.
pub fn pollination_target(world: &World, bee: [f32; 3]) -> Option<([i32; 3], u8)> {
    let bx = bee[0].floor() as i32;
    let by = bee[1].floor() as i32;
    let bz = bee[2].floor() as i32;
    for dy in POLLINATE_MIN_ABOVE..=POLLINATE_MAX_ABOVE {
        let s = world.get_state(bx, by - dy, bz);
        if state_block(s) == SWEET_BERRY_BUSH {
            let age = berry_bush_age(s);
            if age < 3 {
                return Some(([bx, by - dy, bz], age));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// tests (VERIFIED w/Bee + w/Beehive + w/Bee_nest — the research record)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn flat_world() -> World {
        let mut w = World::new(11);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    fn world_with_hive(natural: bool, level: u8) -> World {
        let mut w = flat_world();
        let b = if natural { BEE_NEST } else { BEEHIVE };
        w.set_block_state(8, 70, 8, hive_state(b, level));
        w
    }

    /// registration: a generated nest registers with 2-3 bees (VERIFIED
    /// w/Bee: "Naturally generated bee nests generate with 2-3 bees in
    /// them"); a player-placed hive registers empty
    #[test]
    fn hive_registration_counts() {
        let w = world_with_hive(true, 0);
        let mut sys = HiveSystem::new(7);
        // the round-robin slot for chunk (0,0) (the +8,+8 offset with
        // sim_center (0,0)) is k = 144 → arm scan_t one below
        sys.scan_t = 144 * 20 - 1;
        sys.tick(&w, (0, 0), 8, true);
        let hive = sys.hives.get(&[8, 70, 8]).expect("nest registered");
        assert!(hive.natural);
        assert!(
            (2..=3).contains(&hive.bees.len()),
            "2-3 bees in a generated nest (got {})",
            hive.bees.len()
        );
        // the placed hive
        let w2 = world_with_hive(false, 0);
        let mut sys2 = HiveSystem::new(8);
        sys2.scan_t = 144 * 20 - 1;
        sys2.tick(&w2, (0, 0), 8, true);
        let hive2 = sys2.hives.get(&[8, 70, 8]).expect("hive registered");
        assert!(!hive2.natural);
        assert!(hive2.bees.is_empty(), "placed hives start empty");
    }

    /// the work cycle: an entered nectar bee works 2400 ticks, exits in
    /// daylight, and the hive's honey level bumps by 1 (1%: by 2)
    /// (VERIFIED w/Beehive: "Every pollinated bee that leaves the hive
    /// after working increases the honey level by one")
    #[test]
    fn hive_work_cycle_bumps_honey() {
        let w = world_with_hive(false, 0);
        let mut sys = HiveSystem::new(9);
        let hive = [8, 70, 8];
        sys.hives.insert(hive, HiveData { bees: Vec::new(), natural: false });
        assert!(sys.enter(hive, 10.0, true), "enter at capacity");
        // three nectar bees inside (the honey bump only comes from
        // pollinated bees — "Every pollinated bee that leaves the hive
        // after working increases the honey level by one", VERIFIED)
        sys.enter(hive, 10.0, true);
        sys.enter(hive, 10.0, true);
        // 1 above capacity: 4th bee refused (VERIFIED: "up to 3 bees")
        sys.enter(hive, 10.0, false);
        assert_eq!(sys.hives[&hive].bees.len(), 3);
        // night: no release until day ("waits for daylight")
        for _ in 0..2500 {
            sys.tick(&w, (0, 0), 8, false);
        }
        assert!(sys.releases.is_empty(), "night holds the release");
        // day: the 2400-tick work ends, all three exit, honey bumps
        for _ in 0..10 {
            sys.tick(&w, (0, 0), 8, true);
        }
        assert_eq!(sys.releases.len(), 3);
        assert_eq!(sys.pending_hive_levels.len(), 3);
        let bump = sys.pending_hive_levels.iter().map(|(_, b)| *b).sum::<u8>();
        assert!(
            bump == 3 || bump == 4,
            "three +1 bumps (a 1% +2 makes four): {bump}"
        );
        assert_eq!(sys.released_total, 3);
    }

    /// anger: the stored bees flag angry + exit immediately (VERIFIED
    /// w/Bee: "the bees that are currently in that nest or hive to
    /// leave and swarm the player")
    #[test]
    fn anger_releases_stored_bees_now() {
        let w = world_with_hive(false, 0);
        let mut sys = HiveSystem::new(10);
        let hive = [8, 70, 8];
        sys.hives.insert(hive, HiveData { bees: Vec::new(), natural: false });
        sys.enter(hive, 10.0, false);
        sys.enter(hive, 10.0, false);
        let n = sys.anger(hive);
        assert_eq!(n, 2);
        // angry bees exit even at NIGHT (the harvest swarm)
        sys.tick(&w, (0, 0), 8, false);
        assert_eq!(sys.releases.len(), 2, "angry bees exit immediately");
        assert!(sys.releases.iter().all(|(_, sb)| sb.angry));
    }

    /// the campfire pacify: a LIT campfire within 5 blocks below the
    /// hive pacifies; an unlit one does not (VERIFIED w/Bee)
    #[test]
    fn campfire_pacify_rule() {
        let mut w = world_with_hive(false, 5);
        let hive = [8, 70, 8];
        // no campfire: no pacify
        assert!(!HiveSystem::campfire_pacifies(&w, hive));
        // lit campfire 2 below: pacified
        w.set_block_state(8, 68, 8, campfire_state(true));
        assert!(HiveSystem::campfire_pacifies(&w, hive));
        // 5 below (the max range): pacified
        let mut w2 = world_with_hive(false, 5);
        w2.set_block_state(8, 65, 8, campfire_state(true));
        assert!(HiveSystem::campfire_pacifies(&w2, hive));
        // 6 below (out of range): not pacified
        let mut w3 = world_with_hive(false, 5);
        w3.set_block_state(8, 64, 8, campfire_state(true));
        assert!(!HiveSystem::campfire_pacifies(&w3, hive));
        // UNLIT campfire 2 below: not pacified
        let mut w4 = world_with_hive(false, 5);
        w4.set_block_state(8, 68, 8, campfire_state(false));
        assert!(!HiveSystem::campfire_pacifies(&w4, hive));
    }

    /// honey-level read + the release position (the under-cell)
    #[test]
    fn honey_level_read_and_release_pos() {
        let mut w = world_with_hive(false, 4);
        let hive = [8, 70, 8];
        assert_eq!(HiveSystem::honey_level_at(&w, hive), 4);
        w.set_block_state(8, 70, 8, hive_state(BEEHIVE, 5));
        assert_eq!(HiveSystem::honey_level_at(&w, hive), 5);
        // the release cell: the air cell under the hive
        let p = HiveSystem::release_pos(&w, hive);
        assert!((p[0] - 8.5).abs() < 0.01 && (p[2] - 8.5).abs() < 0.01);
        assert!(p[1] < 70.0, "the release cell is under the hive");
    }

    /// flower finding + the pollination probe (VERIFIED w/Bee
    /// §Pollinating: the 1-2-blocks-above rule; the bush is the
    /// engine's growable crop)
    #[test]
    fn flower_and_pollination_probes() {
        let mut w = flat_world();
        w.set_block(6, 65, 6, FLOWER_RED);
        w.set_block(10, 65, 10, CORNFLOWER);
        let f = find_flower(&w, [8.0, 66.0, 8.0], 10).expect("flower found");
        assert!(matches!(
            w.get_block(f[0], f[1], f[2]),
            FLOWER_RED | FLOWER_YELLOW | OXEYE_DAISY | CORNFLOWER | LILY_OF_THE_VALLEY
        ));
        // a bee 1 block above a young bush pollinates it; a bee 3
        // blocks above does not (the 1-2 rule)
        w.set_block_state(6, 70, 6, berry_bush_state(1));
        let hit = pollination_target(&w, [6.5, 71.5, 6.5]);
        assert!(hit.is_some(), "1-2 blocks above the crop");
        let (pos, age) = hit.unwrap();
        assert_eq!(pos, [6, 70, 6]);
        assert_eq!(age, 1);
        let miss = pollination_target(&w, [6.5, 73.5, 6.5]);
        assert!(miss.is_none(), "3 blocks above: out of the rule");
        // a mature bush (age 3) is not a target
        w.set_block_state(6, 70, 6, berry_bush_state(3));
        assert!(pollination_target(&w, [6.5, 71.5, 6.5]).is_none());
    }

    /// a vanished hive block drops from the registry (the break path —
    /// the game layer also removes it on the break event)
    #[test]
    fn broken_hives_drop_out() {
        let mut w = world_with_hive(false, 0);
        let mut sys = HiveSystem::new(11);
        let hive = [8, 70, 8];
        sys.hives.insert(hive, HiveData { bees: Vec::new(), natural: false });
        // the block vanishes (broken)
        w.set_block_state(8, 70, 8, AIR as u16);
        sys.scan_t = 144 * 20 - 1;
        sys.tick(&w, (0, 0), 8, true);
        assert!(!sys.hives.contains_key(&hive), "registry drops dead hives");
    }
}
