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

use crate::blocks::*;
use crate::ticks::TickScheduler;
use crate::world::World;

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
        } else if b == SAND || b == GRAVEL {
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

/// gravity-block update: sand/gravel above air/water falls one block and
/// re-schedules; stops on support. (vanilla spawns a falling-block ENTITY;
/// the block-wise fall is the documented progressive approximation —
/// deterministic and visually close at 10 Hz falls.)
pub fn gravity_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    if b != SAND && b != GRAVEL {
        return; // stale entry
    }
    if y <= 0 {
        return;
    }
    let below = state_block(world.get_state(x, y - 1, z));
    if below == AIR || below == WATER {
        world.set_block_state(x, y, z, AIR as u16);
        world.set_block_state(x, y - 1, z, s);
        on_block_changed(sched, world, x, y, z);
        on_block_changed(sched, world, x, y - 1, z);
    }
}

/// random-tick plant behaviors (§26 progressive): grass dies under an
/// opaque block, spreads onto bare dirt with sky access.
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
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world(top: i32) -> World {
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = crate::chunk::Chunk::empty();
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
        assert_eq!(levels[7], 255, "flow stops after 7 blocks, got {:?}", levels);
        assert_eq!(levels[8], 255);
    }

    #[test]
    fn source_removal_drains_the_flow() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);
        assert!(water_level(w.get_state(5, 65, 0)) != 255, "flow established");

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
        assert_eq!(state_block(w.get_state(8, 68, 8)), AIR, "left the float position");
        assert_eq!(state_block(w.get_state(8, 65, 8)), SAND, "landed on the floor");
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
