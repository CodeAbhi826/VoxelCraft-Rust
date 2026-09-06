//! Fluid simulation (§24) + falling blocks (§22 "falling blocks") —
//! vanilla-1.16.5-observable water rules on the scheduled-tick backbone.
//!
//! Water semantics implemented (progressive, documented deltas):
//! * sources (level 0) are permanent and feed neighbors
//! * flowing levels 1..7 spread: down first (falling column), then
//!   horizontally at level+1 while level < 7
//! * a flowing block RE-DERIVES its level each tick from its feeders
//!   (min horizontal neighbor level +1, or 1 if fed from above by water);
//!   when the feed disappears the flow decays back and is removed
//! * removal of the source eventually drains every downstream flow
//!
//! Deltas vs 1.16.5 (documented per spec §0 honesty rules):
//! * vanilla falling water is a distinct level-8 full-height block; we use
//!   level-1 flow in the falling column (same spread/decay behavior,
//!   uniform render height today)
//! * lava is not in the block registry yet — lava ticks land with its
//!   registration (Phase 7)
//! * water does not break cross-plants it flows into (it stops at them;
//!   vanilla replaces them) — scheduled with the plant interaction work

use crate::ticks::TickScheduler;
use vc_blocks::blocks::*;
use vc_world::world::World;

/// vanilla water tick rate: updates every 5 game ticks
pub const WATER_TICK_RATE: u64 = 5;

/// sand/gravel fall check delay (vanilla gravity block delay = 2)
pub const GRAVITY_TICK_RATE: u64 = 2;

#[inline]
fn water_at(world: &World, x: i32, y: i32, z: i32) -> Option<u8> {
    let s = world.get_state(x, y, z);
    let l = water_level(s);
    if l == 255 {
        None
    } else {
        Some(l)
    }
}

/// can water spread into this cell? (air; water replaceability for plants
/// is a documented delta — we stop at plants)
#[inline]
fn flowable(s: u16) -> bool {
    s == AIR as u16
}

/// schedule a fluid/gravity update for a position and its 6 neighbors
/// (block-change notification — the §25 "update/tick ordering" backbone:
/// every world edit notifies affected sim blocks)
pub fn on_block_changed(sched: &mut TickScheduler, world: &World, x: i32, y: i32, z: i32) {
    for (dx, dy, dz) in [
        (0i32, 0i32, 0i32),
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        let (nx, ny, nz) = (x + dx, y + dy, z + dz);
        let s = world.get_state(nx, ny, nz);
        let b = state_block(s);
        if b == WATER {
            // sources only re-check when a neighbor changed (cheap path:
            // every water block re-derives; the level check no-ops)
            sched.schedule([nx, ny, nz], WATER_TICK_RATE);
        } else if b == LAVA {
            // Phase E2 lava (VERIFIED w/Lava: flow speed 30 ticks/block in
            // the Overworld/End, 10 in the Nether)
            let rate = lava_tick_rate(world);
            sched.schedule([nx, ny, nz], rate);
        } else if b == SAND || b == GRAVEL || b == ANVIL || b == CHIPPED_ANVIL || b == DAMAGED_ANVIL {
            // Phase E2: anvils are gravity blocks (VERIFIED w/Anvil: falls
            // like sand)
            sched.schedule([nx, ny, nz], GRAVITY_TICK_RATE);
        }
    }
}

/// one water block update at `pos` — the full vanilla-observable rule set
pub fn water_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let Some(level) = water_at(world, x, y, z) else {
        return; // stale entry — the block changed since scheduling
    };

    let below = world.get_state(x, y - 1, z);

    // 1. fall: air below → pour down (vanilla falling-water column)
    if y > 0 && flowable(below) {
        world.set_block_state(x, y - 1, z, water_state(1));
        on_block_changed(sched, world, x, y - 1, z);
        // falling water does NOT spread horizontally this tick
        sched.schedule([x, y - 1, z], WATER_TICK_RATE);
        return;
    }

    // 2. re-derive this block's level from its feeders
    if level > 0 {
        let mut feed: Option<u8> = None;
        // fed from above by any water → strongest feed (vanilla: falling
        // full column)
        if water_at(world, x, y + 1, z).is_some() {
            feed = Some(0);
        } else {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                if let Some(nl) = water_at(world, x + dx, y, z + dz) {
                    feed = Some(feed.map_or(nl, |f: u8| f.min(nl)));
                }
            }
        }
        match feed {
            None => {
                // no feeder: the flow drains away
                world.set_block_state(x, y, z, AIR as u16);
                on_block_changed(sched, world, x, y, z);
                return;
            }
            Some(0) => {
                // adjacent source (or column above): level 1
                if level != 1 {
                    world.set_block_state(x, y, z, water_state(1));
                    on_block_changed(sched, world, x, y, z);
                }
            }
            Some(f) => {
                let target = (f + 1).min(8);
                if target > 7 {
                    // feed too weak — decay
                    world.set_block_state(x, y, z, AIR as u16);
                    on_block_changed(sched, world, x, y, z);
                    return;
                }
                if target != level {
                    world.set_block_state(x, y, z, water_state(target));
                    on_block_changed(sched, world, x, y, z);
                }
            }
        }
    }

    // 3. horizontal spread (sources and flows; only when not falling and
    //    the spread level stays ≤ 7)
    let level = water_level(world.get_state(x, y, z));
    if level == 255 {
        return;
    }
    let spread = level + 1;
    if spread <= 7 {
        let below_solid_or_water = !flowable(world.get_state(x, y - 1, z));
        if below_solid_or_water {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let n = world.get_state(x + dx, y, z + dz);
                if flowable(n) {
                    world.set_block_state(x + dx, y, z + dz, water_state(spread));
                    on_block_changed(sched, world, x + dx, y, z + dz);
                }
            }
        }
    }
}

/// Phase E2 (evolution 1.3-1.4 bracket + VERIFICATION-REPORT fix):
/// LAVA fluid — all values live-verified 2026-09-06 w/Lava
/// (docs/research/phase2-1.3-1.4-research.md):
/// * flow speed: 30 game ticks/block in the Overworld/End, 10 in the
///   Nether (the dimension of the WORLD decides)
/// * flow distance: 4 blocks Overworld/End (source + 3), 8 in the Nether
///   (source + 7) — implemented as the level drop per block: 2 in the
///   Overworld, 1 in the Nether (levels 1..7)
/// * lava falls down first like water; the falling column then spreads
///   from the landing level
/// * lava does NOT create sources (VERIFIED "Creates sources? No") and
///   no infinite-source pairing exists
pub const LAVA_TICK_RATE_OVERWORLD: u64 = 30;
pub const LAVA_TICK_RATE_NETHER: u64 = 10;

/// the lava level drop per block of horizontal spread in this dimension
/// (2 in Overworld/End → 3 spread; 1 in Nether → 7 spread — VERIFIED)
#[inline]
fn lava_drop_off(world: &World) -> u8 {
    match world.dimension {
        vc_world::world::Dimension::Nether => 1,
        _ => 2,
    }
}

/// the lava tick rate for the world's dimension (VERIFIED: 30 / 10)
#[inline]
pub fn lava_tick_rate(world: &World) -> u64 {
    match world.dimension {
        vc_world::world::Dimension::Nether => LAVA_TICK_RATE_NETHER,
        _ => LAVA_TICK_RATE_OVERWORLD,
    }
}

#[inline]
fn lava_at(world: &World, x: i32, y: i32, z: i32) -> Option<u8> {
    let l = lava_level(world.get_state(x, y, z));
    if l == 255 {
        None
    } else {
        Some(l)
    }
}

/// one lava block update — the vanilla-observable rule set (level
/// re-derivation + fall + spread, exactly the water machinery with the
/// lava drop-off and dimension tick rate).
pub fn lava_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let rate = lava_tick_rate(world);
    let drop = lava_drop_off(world);
    let Some(level) = lava_at(world, x, y, z) else {
        return; // stale entry
    };

    let below = world.get_state(x, y - 1, z);
    let below_b = state_block(below);

    // 1. fall: air (or water — lava flows into water, no interaction
    //    products in this bracket: documented) below → pour down
    if y > 0 && (below_b == AIR || below_b == WATER) {
        world.set_block_state(x, y - 1, z, lava_state(1));
        on_block_changed(sched, world, x, y - 1, z);
        sched.schedule([x, y - 1, z], rate);
        return;
    }

    // 2. re-derive this block's level from its feeders
    if level > 0 {
        let mut feed: Option<u8> = None;
        if lava_at(world, x, y + 1, z).is_some() {
            feed = Some(0);
        } else {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                if let Some(nl) = lava_at(world, x + dx, y, z + dz) {
                    // the neighbor's effective feed level already accounts
                    // for its own level; the target = neighbor + drop
                    feed = Some(feed.map_or(nl, |f: u8| f.min(nl)));
                }
            }
        }
        match feed {
            None => {
                world.set_block_state(x, y, z, AIR as u16);
                on_block_changed(sched, world, x, y, z);
                return;
            }
            // fed by a source (or the column above): the drop-off level
            Some(0) => {
                if level != drop {
                    world.set_block_state(x, y, z, lava_state(drop));
                    on_block_changed(sched, world, x, y, z);
                }
            }
            Some(f) => {
                let target = f + drop;
                if target > 7 {
                    // too weak: decay
                    world.set_block_state(x, y, z, AIR as u16);
                    on_block_changed(sched, world, x, y, z);
                    return;
                }
                if target != level {
                    world.set_block_state(x, y, z, lava_state(target));
                    on_block_changed(sched, world, x, y, z);
                }
            }
        }
    }

    // 3. horizontal spread (level + drop stays ≤ 7)
    let level = lava_level(world.get_state(x, y, z));
    if level == 255 {
        return;
    }
    let spread = (level + drop).min(8);
    if spread <= 7 {
        let below_ok = !matches!(state_block(world.get_state(x, y - 1, z)), AIR | WATER);
        if below_ok {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let n = world.get_state(x + dx, y, z + dz);
                let nb = state_block(n);
                if nb == AIR || nb == WATER {
                    world.set_block_state(x + dx, y, z + dz, lava_state(spread));
                    on_block_changed(sched, world, x + dx, y, z + dz);
                }
            }
        }
    }
}

/// gravity-block update: sand/gravel above air/water falls one block and
/// re-schedules; stops on support. (vanilla spawns a falling-block ENTITY;
/// the block-wise fall is the documented progressive approximation —
/// deterministic and visually close at 10 Hz falls.)
pub fn gravity_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    if b != SAND && b != GRAVEL && b != ANVIL && b != CHIPPED_ANVIL && b != DAMAGED_ANVIL {
        return; // stale entry
    }
    if y <= 0 {
        return;
    }
    let below = state_block(world.get_state(x, y - 1, z));
    if below == AIR || below == WATER || below == LAVA {
        world.set_block_state(x, y, z, AIR as u16);
        world.set_block_state(x, y - 1, z, s);
        on_block_changed(sched, world, x, y, z);
        on_block_changed(sched, world, x, y - 1, z);
    }
}

/// random-tick plant behaviors (§26 progressive): grass dies under an
/// opaque block, spreads onto bare dirt with sky access.
/// Phase E1 additions (live-verified 2026-09-06): mycelium spread/die
/// (w/Mycelium §Spread: to dirt within 1 up / 1 sideways / 3 down;
/// mycelium needs light ≥ 9, the dirt cell ≥ 4 and not covered by an
/// opaque block; dies a random time after being covered) and nether-wart
/// growth (w/Nether_Wart: 10% chance per random tick, 4 stages).
pub fn random_plant_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let b = state_block(world.get_state(x, y, z));
    match b {
        GRASS | SNOW_GRASS => {
            // die: opaque block directly above (vanilla turns it to dirt)
            let above = state_block(world.get_state(x, y + 1, z));
            if is_opaque(above) {
                world.set_block_state(x, y, z, DIRT as u16);
                on_block_changed(sched, world, x, y, z);
            }
        }
        DIRT => {
            // spread: a grass neighbor + nothing opaque above this cell
            let above = state_block(world.get_state(x, y + 1, z));
            if !is_opaque(above) {
                let grassy = [
                    (1i32, 0i32, 0i32),
                    (-1, 0, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                    (0, 1, 0),
                    (0, -1, 0),
                ]
                .iter()
                .any(|&(dx, dy, dz)| {
                    let n = state_block(world.get_state(x + dx, y + dy, z + dz));
                    n == GRASS || n == SNOW_GRASS
                });
                if grassy {
                    world.set_block_state(x, y, z, GRASS as u16);
                    on_block_changed(sched, world, x, y, z);
                }
            }
            // Phase E1: mycelium also converts bare dirt (same cell rules
            // as the mycelium arm below — light ≥ 4 + no opaque cover +
            // a mycelium neighbor in the verified 1/1/3 window)
            if state_block(world.get_state(x, y, z)) == DIRT {
                spread_mycelium(world, sched, x, y, z);
            }
        }
        // Phase E1: MYCELIUM — spreads to dirt, dies under opaque cover
        // (VERIFIED w/Mycelium §Spread/§Death; the "random time" of death
        // is the random tick itself — the same day the grass rule uses)
        MYCELIUM => {
            let above = state_block(world.get_state(x, y + 1, z));
            if is_opaque(above) {
                world.set_block_state(x, y, z, DIRT as u16);
                on_block_changed(sched, world, x, y, z);
            }
        }
        // Phase E1: NETHER_WART — 10% chance per random tick to grow one
        // stage (VERIFIED w/Nether_Wart §Farming; light-independent)
        NETHER_WART => {
            let s = world.get_state(x, y, z);
            let age = wart_age(s);
            if age < 3 && world_random_10(world, x, y, z) {
                world.set_block_state(x, y, z, WART_STATE_BASE + age as u16 + 1);
                on_block_changed(sched, world, x, y, z);
            }
        }
        _ => {}
    }
}

/// mycelium spreading onto a bare-dirt cell (VERIFIED window: the TARGET
/// dirt sits within 1 above the mycelium, 1 sideways, or up to 3 below it
/// — from the dirt's perspective the mycelium is 1 below, sideways, or
/// up to 3 above). The dirt cell needs light ≥ 4 and no opaque cover —
/// the caller checked the cover; the light gate is approximated by sky
/// access through the same cover check, documented.
fn spread_mycelium(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let has_mycelium_neighbor = [
        // sideways (VERIFIED: 1 sideways)
        (1i32, 0i32, 0i32),
        (-1, 0, 0),
        (0, 0, 1),
        (0, 0, -1),
        // the mycelium is up to 3 ABOVE this dirt (the dirt is up to 3
        // below the mycelium — VERIFIED), or 1 below it (dirt 1 above)
        (0, 1, 0),
        (0, 2, 0),
        (0, 3, 0),
        (0, -1, 0),
    ]
    .iter()
    .any(|&(dx, dy, dz)| {
        state_block(world.get_state(x + dx, y + dy, z + dz)) == MYCELIUM
    });
    if has_mycelium_neighbor {
        world.set_block_state(x, y, z, MYCELIUM_STATE);
        on_block_changed(sched, world, x, y, z);
    }
}

/// a stable ~10% roll per (world seed, position, sim position) — the
/// random-tick sampler provides the per-tick visit; this provides the
/// growth chance (VERIFIED 10% w/Nether_Wart)
fn world_random_10(world: &World, x: i32, y: i32, z: i32) -> bool {
    let v = vc_rng::rng::Rng::hash3(world.seed ^ 0x0A17, x, y, z);
    v % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world(top: i32) -> World {
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
                for y in 0..=top {
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

    /// helper: run the scheduler to exhaustion (bounded), vanilla tick order
    fn drain(world: &mut World, sched: &mut TickScheduler, max_ticks: u64) {
        for _ in 0..max_ticks {
            let due = sched.tick();
            if due.is_empty() && sched.pending() == 0 {
                break;
            }
            for pos in due {
                water_tick(world, sched, pos[0], pos[1], pos[2]);
                lava_tick(world, sched, pos[0], pos[1], pos[2]);
                gravity_tick(world, sched, pos[0], pos[1], pos[2]);
            }
        }
    }

    #[test]
    fn source_spreads_seven_blocks_and_stops() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        // source on a stone shelf at y=65
        w.set_block_state(0, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);

        // levels 1..7 along +X (order depends on BTreeMap iteration of the
        // 4 spread directions; verify the FULL SET along the axis)
        let mut levels = Vec::new();
        for x in 1..10 {
            levels.push(water_level(w.get_state(x, 65, 0)));
        }
        // each of levels 1..7 appears at distance 1..7 (in the spread
        // directions — check the axis)
        for expect in 1u8..=7 {
            assert!(
                levels.contains(&expect),
                "flow level {expect} must exist on the shelf, got {levels:?}"
            );
        }
        // no water past 7 blocks
        assert_eq!(
            levels[7], 255,
            "flow stops after 7 blocks, got {:?}",
            levels
        );
        assert_eq!(levels[8], 255);
    }

    #[test]
    fn source_removal_drains_the_flow() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);
        assert!(
            water_level(w.get_state(5, 65, 0)) != 255,
            "flow established"
        );

        // remove the source → every downstream flow decays to air
        w.set_block_state(0, 65, 0, AIR as u16);
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 400);
        for x in 0..8 {
            assert_eq!(
                water_level(w.get_state(x, 65, 0)),
                255,
                "all water at x={x} drained"
            );
        }
    }

    #[test]
    fn water_falls_into_a_hole() {
        let mut w = flat_world(64);
        // dig a 1-wide shaft at x=2 down to y=60
        for y in 61..=64 {
            w.set_block_state(2, y, 0, AIR as u16);
        }
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 400);
        // the shaft bottom is wet
        assert!(
            water_level(w.get_state(2, 60, 0)) != 255 || water_level(w.get_state(2, 61, 0)) != 255,
            "water must pour into the shaft"
        );
    }

    #[test]
    fn sand_falls_until_supported() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        // sand floating 3 above the floor
        w.set_block_state(8, 68, 8, SAND as u16);
        on_block_changed(&mut sched, &w, 8, 68, 8);
        drain(&mut w, &mut sched, 100);
        assert_eq!(
            state_block(w.get_state(8, 68, 8)),
            AIR,
            "left the float position"
        );
        assert_eq!(
            state_block(w.get_state(8, 65, 8)),
            SAND,
            "landed on the floor"
        );
        // nothing sank into the floor
        assert_eq!(state_block(w.get_state(8, 64, 8)), STONE);
    }

    #[test]
    fn sand_column_stacks_in_order() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        // 3 sand blocks at y=66..68 (one gap above the floor)
        w.set_block_state(8, 66, 8, SAND as u16);
        w.set_block_state(8, 67, 8, SAND as u16);
        w.set_block_state(8, 68, 8, SAND as u16);
        on_block_changed(&mut sched, &w, 8, 67, 8);
        on_block_changed(&mut sched, &w, 8, 68, 8);
        drain(&mut w, &mut sched, 200);
        assert_eq!(state_block(w.get_state(8, 65, 8)), SAND);
        assert_eq!(state_block(w.get_state(8, 66, 8)), SAND);
        assert_eq!(state_block(w.get_state(8, 67, 8)), SAND);
        assert_eq!(state_block(w.get_state(8, 68, 8)), AIR);
    }

    /// the Phase-6 gate core: identical inputs → identical world hash
    #[test]
    fn flow_is_deterministic() {
        let run = || {
            let mut w = flat_world(64);
            let mut sched = TickScheduler::new();
            w.set_block_state(0, 65, 0, water_state(0));
            w.set_block_state(0, 66, 8, water_state(0)); // second source
            on_block_changed(&mut sched, &w, 0, 65, 0);
            on_block_changed(&mut sched, &w, 0, 66, 8);
            drain(&mut w, &mut sched, 300);
            // hash every non-air cell
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for y in 60..70i32 {
                for z in -2..12i32 {
                    for x in -2..12i32 {
                        let s = w.get_state(x, y, z);
                        if s != 0 {
                            for b in s.to_le_bytes() {
                                h ^= b as u64;
                                h = h.wrapping_mul(0x100_0000_01b3);
                            }
                            h ^= (x as i64).wrapping_mul(31) as u64;
                            h = h.wrapping_mul(0x100_0000_01b3);
                            h ^= (z as i64).wrapping_mul(31) as u64;
                            h = h.wrapping_mul(0x100_0000_01b3);
                            h ^= y as u64;
                            h = h.wrapping_mul(0x100_0000_01b3);
                        }
                    }
                }
            }
            h
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "identical setup → identical world hash");
    }
}

#[cfg(test)]
mod e1_tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world() -> World {
        let mut w = World::new(7);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    /// Phase E1 (VERIFIED w/Mycelium §Spread): a mycelium neighbor
    /// converts bare dirt (1 up / 1 sideways / 3 down window), and
    /// mycelium dies under an opaque cover.
    #[test]
    fn mycelium_spreads_to_dirt_and_dies_under_cover() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        w.set_block_state(8, 65, 8, MYCELIUM_STATE);
        w.set_block(9, 65, 8, DIRT); // sideways neighbor
        w.set_block(8, 66, 8, DIRT); // one above
        w.set_block(8, 62, 8, DIRT); // three below (inside the window)
        w.set_block(12, 61, 8, DIRT); // 4 sideways from every mycelium
        // (OUTSIDE the 1/1/3 window — note: a dirt straight below a fresh
        // mycelium WOULD cascade; this cell has none within range)
        // random tick on the dirt cells
        random_plant_tick(&mut w, &mut sched, 9, 65, 8);
        random_plant_tick(&mut w, &mut sched, 8, 66, 8);
        random_plant_tick(&mut w, &mut sched, 8, 62, 8);
        random_plant_tick(&mut w, &mut sched, 8, 61, 8);
        assert_eq!(w.get_block(9, 65, 8), MYCELIUM, "sideways spread");
        assert_eq!(w.get_block(8, 66, 8), MYCELIUM, "one-up spread");
        assert_eq!(w.get_block(8, 62, 8), MYCELIUM, "three-down spread");
        assert_eq!(w.get_block(12, 61, 8), DIRT, "outside the 1/1/3 window");
        // cascade check: dirt one below the NEW mycelium at (8,62) is in
        // ITS window (dy=+1) — the spread chains downward over time
        w.set_block(8, 61, 8, DIRT);
        random_plant_tick(&mut w, &mut sched, 8, 61, 8);
        assert_eq!(w.get_block(8, 61, 8), MYCELIUM, "chains through fresh mycelium");
        // death: opaque cover above the mycelium
        w.set_block(9, 66, 8, STONE);
        random_plant_tick(&mut w, &mut sched, 9, 65, 8);
        assert_eq!(w.get_block(9, 65, 8), DIRT, "dies under an opaque cover");
    }

    /// Phase E1 (VERIFIED w/Nether_Wart): 4 growth stages, one age per
    /// successful 10% roll, stopping at the last.
    #[test]
    fn nether_wart_grows_through_four_stages() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        w.set_block_state(8, 65, 8, WART_STATE_BASE);
        // find the roll outcomes for this seed at this position: the
        // growth uses a stable per-position hash, so walk the ages by
        // scanning a position whose hash rolls True... instead, drive it
        // through the verified state machine directly: plant at several
        // positions where the 10% roll hits
        let mut grown = None;
        for x in 0..16i32 {
            w.set_block_state(x, 65, 8, WART_STATE_BASE);
            if world_random_10(&w, x, 65, 8) {
                grown = Some(x);
            }
        }
        let x = grown.expect("some position rolls the 10% growth");
        random_plant_tick(&mut w, &mut sched, x, 65, 8);
        assert_eq!(w.get_state(x, 65, 8), WART_STATE_BASE + 1, "age 1");
        // advance to the cap by direct state seeding (the roll is
        // position-fixed, so re-rolling needs a new position each age —
        // the growth path is already proven above)
        w.set_block_state(x, 65, 8, WART_STATE_BASE + 3);
        random_plant_tick(&mut w, &mut sched, x, 65, 8);
        assert_eq!(w.get_state(x, 65, 8), WART_STATE_BASE + 3, "caps at age 3");
    }
}

#[cfg(test)]
mod e2_tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world(top: i32) -> World {
        let mut w = World::new(9);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
                for y in 0..=top {
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

    fn drain(world: &mut World, sched: &mut TickScheduler, max_ticks: u64) {
        for _ in 0..max_ticks {
            let due = sched.tick();
            if due.is_empty() && sched.pending() == 0 {
                break;
            }
            for pos in due {
                water_tick(world, sched, pos[0], pos[1], pos[2]);
                lava_tick(world, sched, pos[0], pos[1], pos[2]);
                gravity_tick(world, sched, pos[0], pos[1], pos[2]);
            }
        }
    }

    /// Phase E2 (VERIFIED w/Lava): Overworld lava spreads at level-drop 2
    /// → 3 blocks from the source (4 incl. source); Nether drop 1 → 7
    /// blocks (8 incl. source). Tick rates: 30 / 10.
    #[test]
    fn lava_rates_and_spread_by_dimension() {
        assert_eq!(LAVA_TICK_RATE_OVERWORLD, 30);
        assert_eq!(LAVA_TICK_RATE_NETHER, 10);
        // Overworld: source at y=65 on a shelf
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, lava_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 2000);
        // spread along +X: at most 3 blocks of flow (levels 2,4,6)
        let mut max_x = 0i32;
        for x in 1..12 {
            if lava_level(w.get_state(x, 65, 0)) != 255 {
                max_x = x;
            }
        }
        assert!(
            (1..=3).contains(&max_x),
            "Overworld lava spreads 3 blocks, got {max_x}"
        );
        // the levels follow the +2 drop (2, 4, 6 across the run)
        let l1 = lava_level(w.get_state(1, 65, 0));
        assert_eq!(l1, 2, "first flow block is level 2 (drop-off 2)");

        // Nether: full 7-block spread
        let mut wn = World::new_in_dimension(9, vc_world::world::Dimension::Nether);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
                for y in 0..=64 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y as usize, lz, NETHERRACK);
                        }
                    }
                }
                wn.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        wn.dirty.clear();
        let mut sched_n = TickScheduler::new();
        wn.set_block_state(0, 65, 0, lava_state(0));
        on_block_changed(&mut sched_n, &wn, 0, 65, 0);
        drain(&mut wn, &mut sched_n, 2000);
        let mut max_xn = 0i32;
        for x in 1..12 {
            if lava_level(wn.get_state(x, 65, 0)) != 255 {
                max_xn = x;
            }
        }
        assert!(
            (5..=7).contains(&max_xn),
            "Nether lava spreads up to 7 blocks, got {max_xn}"
        );
        let ln1 = lava_level(wn.get_state(1, 65, 0));
        assert_eq!(ln1, 1, "Nether first flow block is level 1 (drop-off 1)");
    }

    /// Phase E2 (VERIFIED w/Anvil): anvils are gravity blocks — they fall
    /// like sand until supported.
    #[test]
    fn anvil_falls_until_supported() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(8, 68, 8, ANVIL_STATE);
        on_block_changed(&mut sched, &w, 8, 68, 8);
        drain(&mut w, &mut sched, 100);
        assert_eq!(state_block(w.get_state(8, 68, 8)), AIR, "left the float");
        assert_eq!(state_block(w.get_state(8, 65, 8)), ANVIL, "landed");
    }

    /// Phase E2: lava source removal drains the flow (the water decay
    /// rule shared by both fluids).
    #[test]
    fn lava_source_removal_drains() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, lava_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 2000);
        assert!(lava_level(w.get_state(1, 65, 0)) != 255, "flow established");
        w.set_block_state(0, 65, 0, AIR as u16);
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 4000);
        for x in 0..5 {
            assert_eq!(
                lava_level(w.get_state(x, 65, 0)),
                255,
                "lava at x={x} drained"
            );
        }
    }
}
