//! Redstone core (§25 subset, Phase 6): lever sources, wire power levels
//! with 1-per-block decay, and the inverting redstone torch — driven by
//! the deterministic scheduled-tick backbone (2 game ticks = 1 redstone
//! tick, vanilla timing).
//!
//! Observable behaviors covered (the spec's "reproduce observable
//! update/tick ordering"):
//! * lever ON powers adjacent wire at 15; OFF lets it decay away
//! * wire power = max(neighbor wire power − 1, adjacent source = 15)
//! * torch INVERTS the power state of the block it stands on: wire
//!   feeding the support block turns the torch OFF; the torch powers
//!   adjacent wires at 15 while LIT — the classic NOT gate
//! * a wire loop through a torch oscillates (the classic torch clock)
//!
//! Documented deltas vs 1.16.5: no strong/weak block-power distinction
//! (wire powers only wires and torch supports), repeaters/comparators/
//! pistons and the remaining §25 component list are not in the registry
//! yet — this is the connectivity + ordering core they bolt onto.

use vc_blocks::blocks::*;
use crate::ticks::TickScheduler;
use vc_world::world::World;

/// 1 redstone tick = 2 game ticks (vanilla)
pub const REDSTONE_TICK_RATE: u64 = 2;

/// true if a position holds any redstone component (wire/torch/lever)
#[inline]
pub fn is_component(s: u16) -> bool {
    let b = state_block(s);
    b == REDSTONE_WIRE || b == REDSTONE_TORCH || b == LEVER
}

/// schedule redstone updates for a position and its neighbors after a
/// world edit (block-change notification, §25 ordering backbone)
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
        if is_component(world.get_state(nx, ny, nz)) {
            sched.schedule([nx, ny, nz], REDSTONE_TICK_RATE);
        }
        // a torch ALSO re-checks when its support block changes
        if world.get_block(nx, ny + 1, nz) == REDSTONE_TORCH {
            sched.schedule([nx, ny + 1, nz], REDSTONE_TICK_RATE);
        }
    }
    // if the changed position is a torch, its own support may matter
    if world.get_block(x, y, z) == REDSTONE_TORCH {
        sched.schedule([x, y, z], REDSTONE_TICK_RATE);
    }
}

/// power fed into a wire cell from DIRECT sources (levers, lit torches)
fn direct_feed(world: &World, x: i32, y: i32, z: i32) -> u8 {
    let mut best = 0u8;
    for (dx, dy, dz) in [
        (1i32, 0i32, 0i32),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        let s = world.get_state(x + dx, y + dy, z + dz);
        let b = state_block(s);
        if b == LEVER && lever_is_on(s) {
            best = best.max(15);
        }
        if b == REDSTONE_TORCH && torch_is_lit(s) {
            best = best.max(15);
        }
    }
    best
}

/// is the block at (x,y,z) "powered" for torch-support purposes? A wire
/// with power > 0 feeding it horizontally (the vanilla NOT-gate layout:
/// wire runs INTO the block the torch stands on)
fn support_powered(world: &World, x: i32, y: i32, z: i32) -> bool {
    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
        let p = wire_power(world.get_state(x + dx, y, z + dz));
        if p != 255 && p > 0 {
            return true;
        }
        // a lever attached to the support block also powers it
        let s = world.get_state(x + dx, y, z + dz);
        if state_block(s) == LEVER && lever_is_on(s) {
            return true;
        }
    }
    false
}

/// one wire update: re-derive power, propagate to neighbors
pub fn wire_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    if wire_power(s) == 255 {
        return; // stale entry
    }
    let mut target = direct_feed(world, x, y, z);
    // NOTE: vertical wire neighbors power each other (vanilla needs slope
    // geometry — air-block-air — for that; our free-floating wires accept
    // the direct vertical connection; documented permissiveness)
    for (dx, dy, dz) in [
        (1i32, 0i32, 0i32),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        let p = wire_power(world.get_state(x + dx, y + dy, z + dz));
        if p != 255 {
            target = target.max(p.saturating_sub(1));
        }
    }
    let old = wire_power(s);
    if target != old {
        world.set_block_state(x, y, z, wire_state(target));
        // notify: adjacent components re-check next redstone tick
        on_block_changed(sched, world, x, y, z);
        // torches AROUND a powered-changing wire re-check (their support
        // may be this wire)
        for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let (wx, wz) = (x + dx, z + dz);
            if world.get_block(wx, y + 1, wz) == REDSTONE_TORCH {
                sched.schedule([wx, y + 1, wz], REDSTONE_TICK_RATE);
            }
            if world.get_block(wx, y - 1, wz) == REDSTONE_TORCH {
                // wire under a torch's support column
                sched.schedule([wx, y - 1, wz], REDSTONE_TICK_RATE);
            }
        }
    }
}

/// one torch update: lit unless its support block is powered
pub fn torch_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    if state_block(s) != REDSTONE_TORCH {
        return; // stale entry
    }
    let should_be_lit = !support_powered(world, x, y - 1, z);
    if torch_is_lit(s) != should_be_lit {
        world.set_block_state(x, y, z, torch_state(should_be_lit));
        on_block_changed(sched, world, x, y, z);
    }
}

/// one lever update: levers only change by interaction (right-click);
/// this tick exists so a re-scheduled entry no-ops cleanly
pub fn lever_tick(world: &World, x: i32, y: i32, z: i32) {
    if state_block(world.get_state(x, y, z)) != LEVER {
        return; // stale
    }
}

/// toggle a lever (interactive) and kick the update cascade
pub fn toggle_lever(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) -> bool {
    let s = world.get_state(x, y, z);
    if state_block(s) != LEVER {
        return false;
    }
    let new = lever_state(!lever_is_on(s));
    world.set_block_state(x, y, z, new);
    on_block_changed(sched, world, x, y, z);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world() -> World {
        let mut w = World::new(7);
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

    fn drain(world: &mut World, sched: &mut TickScheduler, max_ticks: u64) {
        for _ in 0..max_ticks {
            let due = sched.tick();
            if due.is_empty() && sched.pending() == 0 {
                break;
            }
            for pos in due {
                let b = state_block(world.get_state(pos[0], pos[1], pos[2]));
                match b {
                    REDSTONE_WIRE => wire_tick(world, sched, pos[0], pos[1], pos[2]),
                    REDSTONE_TORCH => torch_tick(world, sched, pos[0], pos[1], pos[2]),
                    LEVER => lever_tick(world, pos[0], pos[1], pos[2]),
                    _ => {}
                }
            }
        }
    }

    /// lever ON → wire chain decays 15, 14, 13... and nothing beyond 15
    #[test]
    fn lever_powers_wire_chain_with_decay() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, lever_state(true));
        for x in 1..=17 {
            w.set_block_state(x, 65, 0, wire_state(0));
            on_block_changed(&mut sched, &w, x, 65, 0);
        }
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);

        for x in 1..=15i32 {
            let p = wire_power(w.get_state(x, 65, 0));
            assert_eq!(p, (16 - x) as u8, "wire at x={x} power {p}");
        }
        // past the 15-block reach: power 0
        assert_eq!(wire_power(w.get_state(16, 65, 0)), 0);
        assert_eq!(wire_power(w.get_state(17, 65, 0)), 0);
    }

    /// lever OFF → the chain drains back to 0
    #[test]
    fn lever_off_drains_chain() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, lever_state(true));
        for x in 1..=5 {
            w.set_block_state(x, 65, 0, wire_state(0));
        }
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 100);
        assert_eq!(wire_power(w.get_state(5, 65, 0)), 11, "chain live");

        toggle_lever(&mut w, &mut sched, 0, 65, 0);
        drain(&mut w, &mut sched, 200);
        for x in 1..=5 {
            assert_eq!(
                wire_power(w.get_state(x, 65, 0)),
                0,
                "wire at x={x} drained"
            );
        }
        // the lever itself is OFF
        assert!(!lever_is_on(w.get_state(0, 65, 0)));
    }

    /// the classic NOT gate: wire → torch support → torch inverts
    #[test]
    fn torch_inverts_its_support_signal() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // lever at (0,65,0) → wire at (1..3, 65) feeding the support block
        // at (4,65); torch stands ON the support at (4,66); output wire at
        // (5..6, 66) fed by the torch
        w.set_block_state(0, 65, 0, lever_state(true));
        for x in 1..=3 {
            w.set_block_state(x, 65, 0, wire_state(0));
        }
        // torch on top of the block the wire points into
        w.set_block_state(4, 66, 0, torch_state(true));
        w.set_block_state(5, 66, 0, wire_state(0));
        w.set_block_state(6, 66, 0, wire_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 300);

        // wire feeds the torch's support (block 4 at y=65 — the wire at
        // x=3 is horizontally adjacent to it) → torch OFF → output 0
        assert!(!torch_is_lit(w.get_state(4, 66, 0)), "torch inverted");
        assert_eq!(wire_power(w.get_state(5, 66, 0)), 0, "output dead");

        // kill the lever → torch relights → output powers up
        toggle_lever(&mut w, &mut sched, 0, 65, 0);
        drain(&mut w, &mut sched, 300);
        assert!(torch_is_lit(w.get_state(4, 66, 0)), "torch relit");
        assert_eq!(wire_power(w.get_state(5, 66, 0)), 15, "output powered");
        assert_eq!(wire_power(w.get_state(6, 66, 0)), 14, "output decays");
    }

    /// a torch + wire loop feeding its own support = the classic clock
    #[test]
    fn torch_clock_oscillates() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // torch at (0,66); wire loop: (1,66)→(2,66)→(2,65)... support of
        // the torch is (0,65): wire at (1,65) touches it
        w.set_block_state(0, 66, 0, torch_state(true));
        w.set_block_state(1, 66, 0, wire_state(0));
        w.set_block_state(2, 66, 0, wire_state(0));
        w.set_block_state(2, 65, 0, wire_state(0));
        w.set_block_state(1, 65, 0, wire_state(0));
        on_block_changed(&mut sched, &w, 0, 66, 0);
        // run 400 game ticks (20 s of redstone time) WITHOUT settling —
        // the clock keeps rescheduling
        let mut lit_changes = 0;
        let mut was_lit = true;
        for _ in 0..400 {
            let due = sched.tick();
            for pos in due {
                let b = state_block(w.get_state(pos[0], pos[1], pos[2]));
                match b {
                    REDSTONE_WIRE => wire_tick(&mut w, &mut sched, pos[0], pos[1], pos[2]),
                    REDSTONE_TORCH => torch_tick(&mut w, &mut sched, pos[0], pos[1], pos[2]),
                    LEVER => lever_tick(&w, pos[0], pos[1], pos[2]),
                    _ => {}
                }
            }
            let lit = torch_is_lit(w.get_state(0, 66, 0));
            if lit != was_lit {
                lit_changes += 1;
                was_lit = lit;
            }
        }
        assert!(
            lit_changes >= 4,
            "the torch clock oscillates ({} transitions)",
            lit_changes
        );
        assert!(sched.pending() > 0, "the clock keeps ticking");
    }

    /// §25 gate: identical circuit → identical state hash
    #[test]
    fn redstone_is_deterministic() {
        let run = || {
            let mut w = flat_world();
            let mut sched = TickScheduler::new();
            w.set_block_state(0, 65, 0, lever_state(true));
            for x in 1..=4 {
                w.set_block_state(x, 65, 0, wire_state(0));
            }
            w.set_block_state(5, 66, 0, torch_state(true));
            w.set_block_state(6, 66, 0, wire_state(0));
            on_block_changed(&mut sched, &w, 0, 65, 0);
            drain(&mut w, &mut sched, 200);
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for y in 64..68i32 {
                for x in 0..8i32 {
                    let s = w.get_state(x, y, 0);
                    for b in s.to_le_bytes() {
                        h ^= b as u64;
                        h = h.wrapping_mul(0x100_0000_01b3);
                    }
                    h ^= x as u64;
                    h = h.wrapping_mul(0x100_0000_01b3);
                    h ^= y as u64;
                    h = h.wrapping_mul(0x100_0000_01b3);
                }
            }
            h
        };
        assert_eq!(run(), run(), "identical circuit → identical hash");
    }
}
