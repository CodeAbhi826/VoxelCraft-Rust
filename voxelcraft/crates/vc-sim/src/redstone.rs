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

use crate::ticks::TickScheduler;
use vc_blocks::blocks::*;
use vc_world::world::World;

/// 1 redstone tick = 2 game ticks (vanilla)
pub const REDSTONE_TICK_RATE: u64 = 2;

/// true if a position holds any redstone component (wire/torch/lever +
/// the Phase 3 set)
#[inline]
pub fn is_component(s: u16) -> bool {
    let b = state_block(s);
    b == REDSTONE_WIRE
        || b == REDSTONE_TORCH
        || b == LEVER
        || b == REPEATER
        || b == COMPARATOR
        || b == PISTON
        || b == STICKY_PISTON
        || b == DISPENSER
        || b == DROPPER
        || b == OBSERVER
        || b == HOPPER
        // Phase E1: the redstone lamp re-checks its inputs on neighbor
        // changes (light on/off rides the block state)
        || b == REDSTONE_LAMP
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
        let ns = world.get_state(nx, ny, nz);
        if state_block(ns) == REPEATER {
            // VERIFIED: repeater delay is 1..4 redstone ticks (2..8 game
            // ticks) — schedule at the repeater's OWN delay so the commit
            // lands after exactly that long
            let (_, delay, _) = repeater_decode(ns);
            sched.schedule([nx, ny, nz], delay as u64 * REDSTONE_TICK_RATE);
        } else if is_component(ns) && state_block(ns) != OBSERVER {
            // OBSERVER is deliberately EXCLUDED here: for every other
            // component a scheduled entry means "re-check your inputs"
            // (idempotent — a stale entry settles to a no-op); for the
            // observer it means TOGGLE, so scheduling it on its own
            // state change would oscillate forever (a free-running
            // 2gt clock with an ever-growing queue — caught live in the
            // Phase 3 E2E: a probed observer sat powered=true minutes
            // after its single pulse)
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
    // Phase 3: observers watching THIS cell pulse — their facing points
    // at it. This is the ONLY observer scheduling path (see the note in
    // the loop above for why the generic component arm skips them).
    // VERIFIED vanilla semantics: an observer pulses when the block at
    // its watched face changes — including wire power changes (wire
    // states encode power) and piston-pushed blocks.
    schedule_observers(sched, world, x, y, z);

    // Phase 3: quasi-connectivity shadow — pistons/dispensers/droppers
    // watch the neighbors of the cell ABOVE them, so a change here must
    // notify QC consumers at the shadow positions (unit − (0,1,0)):
    // (x±1, y−1, z), (x, y−1, z±1), (x, y, z), (x, y−2, z)
    for (sx, sy, sz) in [
        (x + 1, y - 1, z),
        (x - 1, y - 1, z),
        (x, y - 1, z + 1),
        (x, y - 1, z - 1),
        (x, y, z),
        (x, y - 2, z),
    ] {
        let sb_ = state_block(world.get_state(sx, sy, sz));
        if matches!(sb_, PISTON | STICKY_PISTON | DISPENSER | DROPPER) {
            sched.schedule([sx, sy, sz], REDSTONE_TICK_RATE);
        }
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
        // Phase 3: repeater/comparator/observer outputs power adjacent
        // wire at full strength (comparator strength nuance: v1 binary —
        // documented; the exact level formula still drives its own tick)
        if b == REPEATER {
            let (f, _, powered) = repeater_decode(s);
            if powered {
                let [fx, _, fz] = horiz_facing_vec(f);
                if (dx, dz) == (fx, fz) {
                    best = best.max(15);
                }
            }
        }
        if b == COMPARATOR {
            let (f, _, powered) = comparator_decode(s);
            if powered {
                let [fx, _, fz] = horiz_facing_vec(f);
                if (dx, dz) == (fx, fz) {
                    best = best.max(15);
                }
            }
        }
        if b == OBSERVER {
            let (f, powered) = observer_decode(s);
            if powered {
                let [fx, fy, fz] = full_facing_vec(f);
                if (dx, dy, dz) == (fx, fy, fz) {
                    best = best.max(15);
                }
            }
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

/// Phase E1: one redstone-lamp update (VERIFIED w/Redstone_Lamp):
/// - an adjacent active power source lights it (on torch / on lever /
///   powered wire)
/// - it turns off when power is gone. Vanilla's exact OFF delay is 4
///   GAME ticks ("takes 4 ticks (0.2 seconds) to turn off in Java");
///   our redstone backbone ticks at 2-game-tick granularity, so the
///   delayed OFF lands at the next redstone tick — a documented
///   approximation of the verified 4gt (cancel-on-repower falls out
///   naturally: a returned power source keeps it lit).
pub fn lamp_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    if state_block(s) != REDSTONE_LAMP {
        return; // stale entry
    }
    let powered = lamp_powered(world, x, y, z);
    let lit = s == REDSTONE_LAMP_LIT;
    if powered && !lit {
        // ON (VERIFIED: "A redstone lamp activates instantly")
        world.set_block_state(x, y, z, REDSTONE_LAMP_LIT);
        on_block_changed(sched, world, x, y, z);
    } else if !powered && lit {
        // OFF at the next redstone tick (the ~4gt approximation above)
        world.set_block_state(x, y, z, REDSTONE_LAMP_STATE);
        on_block_changed(sched, world, x, y, z);
    }
}

/// is any adjacent cell feeding the lamp? (torch lit / lever on /
/// wire powered — the engine's verified power sources)
fn lamp_powered(world: &World, x: i32, y: i32, z: i32) -> bool {
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
        if b == REDSTONE_TORCH && torch_is_lit(s) {
            return true;
        }
        if b == LEVER && lever_is_on(s) {
            return true;
        }
        let p = wire_power(s);
        if p != 255 && p > 0 {
            return true;
        }
    }
    false
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
                    REDSTONE_LAMP => lamp_tick(world, sched, pos[0], pos[1], pos[2]),
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
                    REDSTONE_LAMP => lamp_tick(&mut w, &mut sched, pos[0], pos[1], pos[2]),
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

// ============================================================================
// Phase 3 components — repeater, comparator, observer, pistons (+QC),
// dispenser/dropper rising edge, hopper transfer. VERIFIED values (wiki,
// 2026-09-04) noted per constant; everything else documented as adapted.
// ============================================================================

/// piston push limit (VERIFIED: "it pushes at most 12 blocks")
pub const PISTON_PUSH_LIMIT: usize = 12;
/// comparator output delay (VERIFIED: "with a 2 game tick delay")
pub const COMPARATOR_DELAY: u64 = 2;
/// observer pulse length (VERIFIED: "emits ... for 2 game ticks")
pub const OBSERVER_PULSE: u64 = 2;
/// hopper transfer cooldown (VERIFIED: 8 game ticks = 2.5 items/s)
pub const HOPPER_COOLDOWN: u64 = 8;
/// dispenser/dropper activation delay (VERIFIED: 4 game ticks)
pub const DISPENSER_DELAY: u64 = 4;

/// is a position powered by any wire/lever/torch/repeater-output/observer?
/// (our power model: wire power > 0 at the position, or an adjacent lit
/// torch / on-lever / powered repeater/comparator/observer output facing
/// into it)
fn power_at(world: &World, x: i32, y: i32, z: i32) -> u8 {
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
        // wire above/beside feeding this cell
        let p = wire_power(s);
        if p != 255 && p > 0 && !is_wire_power(s) {
            // wire_power only decodes actual wire states; for the cell
            // itself this arm is handled by direct sources below
        }
        match b {
            REDSTONE_WIRE => {
                let p = wire_power(s);
                if p != 255 {
                    best = best.max(p);
                }
            }
            LEVER => {
                if lever_is_on(s) {
                    best = best.max(15);
                }
            }
            REDSTONE_TORCH => {
                if torch_is_lit(s) {
                    best = best.max(15);
                }
            }
            REPEATER => {
                let (f, _, powered) = repeater_decode(s);
                if powered {
                    // facing = INPUT direction → output cell = source −
                    // facing; the target IS that cell iff the offset from
                    // the target to the source equals +facing
                    let [fx, _, fz] = horiz_facing_vec(f);
                    if (dx, dz) == (fx, fz) {
                        best = best.max(15);
                    }
                }
            }
            COMPARATOR => {
                let (f, _, powered) = comparator_decode(s);
                if powered {
                    let [fx, _, fz] = horiz_facing_vec(f);
                    if (dx, dz) == (fx, fz) {
                        best = best.max(15);
                    }
                }
            }
            OBSERVER => {
                let (f, powered) = observer_decode(s);
                if powered {
                    // facing = WATCH direction; the 2gt pulse emits from
                    // the BACK cell = source − facing (VERIFIED: the
                    // observer's back face is the output). Target ==
                    // source − facing ⟺ (dx,dy,dz) == (fx,fy,fz) — the
                    // same algebra as the repeater/comparator arms above.
                    // (A transiently-inverted copy of this condition was
                    // caught by the browser E2E observer→piston chain —
                    // the unit suite masked it via direct_feed.)
                    let [fx, fy, fz] = full_facing_vec(f);
                    if (dx, dy, dz) == (fx, fy, fz) {
                        best = best.max(15);
                    }
                }
            }
            _ => {}
        }
    }
    best
}

/// the SIGNAL present AT a cell (for component inputs): the wire power
/// stored there, or a source block sitting in the cell itself, or the
/// power its neighbors feed it — the max of all three.
fn signal_at(world: &World, x: i32, y: i32, z: i32) -> u8 {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    let mut v = match b {
        REDSTONE_WIRE => wire_power(s),
        LEVER => {
            if lever_is_on(s) {
                15
            } else {
                0
            }
        }
        REDSTONE_TORCH => {
            if torch_is_lit(s) {
                15
            } else {
                0
            }
        }
        // a powered repeater sitting in the input cell feeds us 15
        // (direct back-to-back chaining; the wire medium covers the
        // general case)
        REPEATER => {
            let (_, _, powered) = repeater_decode(s);
            if powered {
                15
            } else {
                0
            }
        }
        _ => 0,
    };
    if v == 0 {
        v = power_at(world, x, y, z);
    }
    v
}

/// quasi-connectivity (VERIFIED as Java-only intentional behavior): a
/// piston/dispenser/dropper is also powered when the block ABOVE its
/// position is powered (any powered neighbor of the cell one up).
pub fn qc_powered(world: &World, x: i32, y: i32, z: i32) -> bool {
    if power_at(world, x, y, z) > 0 {
        return true;
    }
    // QC: check the neighbors of the cell above
    power_at(world, x, y + 1, z) > 0
        || power_at(world, x + 1, y + 1, z) > 0
        || power_at(world, x - 1, y + 1, z) > 0
        || power_at(world, x, y + 1, z + 1) > 0
        || power_at(world, x, y + 1, z - 1) > 0
}

/// blocks a piston cannot push (VERIFIED wiki table, our-registry subset:
/// bedrock, obsidian, enchanting table, ender chest analog, extended
/// pistons, container blocks holding entities — chests/dispensers/
/// droppers/hoppers/furnaces/brewing stands)
pub fn piston_unpushable(b: u8) -> bool {
    matches!(
        b,
        BEDROCK
            | OBSIDIAN
            | ENCHANT_TABLE
            | PISTON
            | STICKY_PISTON
            | CHEST
            | DISPENSER
            | DROPPER
            | HOPPER
            | FURNACE
            | BREWING_STAND
            | WATER
    ) || (b >= REPEATER && b <= COMPARATOR) // directional plates ride the
                                            // floor in vanilla; ours stay put (documented simplification)
}

/// blocks that break (pop as drops) when pushed — our cross family
/// (wire/torch/lever/plants) and the new plates
pub fn piston_breaks(b: u8) -> bool {
    is_cross(b)
}

/// one repeater update. VERIFIED: delay 1..4 redstone ticks (scheduled
/// at the repeater's OWN delay by on_block_changed); a powered repeater
/// facing into this one's SIDE locks it — the output freezes (vanilla
/// persists the lock bit; we DERIVE it each tick — observably identical).
pub fn repeater_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    if state_block(s) != REPEATER {
        return; // stale
    }
    let (f, delay, powered) = repeater_decode(s);
    // lock check: a powered repeater facing INTO our side
    let mut is_locked = false;
    let [fx, _, fz] = horiz_facing_vec(f);
    for side in [[-fz, 0, fx], [fz, 0, -fx]] {
        let ss = world.get_state(x + side[0], y, z + side[2]);
        if state_block(ss) == REPEATER {
            let (sf, _, sp) = repeater_decode(ss);
            let [sfx, _, sfz] = horiz_facing_vec(sf);
            // its output points at us when its facing equals our offset
            // from it (output cell = source − facing = us)
            if sp && (sfx, sfz) == (side[0] as i32, side[2] as i32) {
                is_locked = true;
            }
        }
    }
    if is_locked {
        return; // frozen: output held despite input changes
    }
    // input: the signal AT the rear cell (behind = +facing) — its own
    // wire power / source, or what its neighbors feed it
    let (rx, rz) = (x + fx, z + fz);
    let input = signal_at(world, rx, y, rz).max(signal_at(world, rx, y + 1, rz));
    let want = input > 0;
    if want != powered {
        // commit — the delay already elapsed via the scheduler (the
        // repeater is scheduled at its OWN delay: 1..4 redstone ticks =
        // 2..8 game ticks, VERIFIED). Pulses shorter than the delay are
        // filtered because the input is re-read here at fire time
        // (documented approximation of vanilla's schedule-time latching).
        let ns = repeater_state(f, delay, want);
        world.set_block_state(x, y, z, ns);
        on_block_changed(sched, world, x, y, z);
    }
}

/// one comparator update (VERIFIED formulas):
/// compare mode:  out = rear × [left ≤ rear AND right ≤ rear]
/// subtract mode: out = max(rear − max(left, right), 0)
/// delay: 2 game ticks; container reading: rear input from a container's
/// fill fraction (vanilla signal = 1 + 14·fill — VERIFIED shape)
pub fn comparator_tick(
    world: &mut World,
    sched: &mut TickScheduler,
    containers: &crate::containers::Containers,
    x: i32,
    y: i32,
    z: i32,
) {
    let s = world.get_state(x, y, z);
    if state_block(s) != COMPARATOR {
        return;
    }
    let (f, subtract, powered) = comparator_decode(s);
    let [fx, _, fz] = horiz_facing_vec(f);
    // rear input (the cell behind the comparator)
    let (rx, rz) = (x + fx, z + fz);
    let mut rear = signal_at(world, rx, y, rz).max(signal_at(world, rx, y + 1, rz));
    // container reading (vanilla: comparator reads the block BEHIND it)
    if let Some(inv) = containers.get(&[rx, y, rz]) {
        rear = rear.max((1.0 + 14.0 * inv.fill_fraction()) as u8);
    }
    // side inputs: the signal AT the side cells
    let left = signal_at(world, x - fz, y, z + fx);
    let right = signal_at(world, x + fz, y, z - fx);
    let out = if subtract {
        (rear as i16 - left.max(right) as i16).max(0) as u8
    } else {
        if left <= rear && right <= rear {
            rear
        } else {
            0
        }
    };
    let want = out > 0;
    if want != powered {
        // 2 game tick delay (VERIFIED): commit on the next fire
        world.set_block_state(x, y, z, comparator_state(f, subtract, want));
        on_block_changed(sched, world, x, y, z);
    }
}

/// observer pulse bookkeeping: observers fire 2 game ticks after the
/// block in front changes (VERIFIED: pulse of strength 15 for 2 game
/// ticks). The pulse-off is scheduled when firing.
pub fn observer_pulse(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    if state_block(s) != OBSERVER {
        return;
    }
    let (f, powered) = observer_decode(s);
    if !powered {
        // fire: power on for 2 game ticks, then off
        world.set_block_state(x, y, z, observer_state(f, true));
        on_block_changed(sched, world, x, y, z);
        sched.schedule([x, y, z], OBSERVER_PULSE);
    }
    // (powered observers arriving here are turning OFF)
    else {
        world.set_block_state(x, y, z, observer_state(f, false));
        on_block_changed(sched, world, x, y, z);
    }
}

/// observer detection hook — call from on_block_changed: any observer
/// whose FACE points at the changed cell schedules a pulse
fn schedule_observers(sched: &mut TickScheduler, world: &World, x: i32, y: i32, z: i32) {
    for f in 0..6usize {
        // observer at (x - dx, ...) with facing f watches (x, y, z)
        let [dx, dy, dz] = full_facing_vec(f);
        let (ox, oy, oz) = (x - dx, y - dy, z - dz);
        let s = world.get_state(ox, oy, oz);
        if state_block(s) == OBSERVER {
            let (of, _) = observer_decode(s);
            if of == f {
                sched.schedule([ox, oy, oz], OBSERVER_PULSE);
            }
        }
    }
}

/// piston update: extend when powered (QC included — VERIFIED Java
/// behavior), retract when unpowered. Extension moves up to 12 blocks
/// (VERIFIED); unpushable blocks stop it; break-on-push blocks pop.
/// Sticky pistons pull one block back on retraction.
/// Instant block motion (documented adaptation: vanilla animates over
/// 2 game ticks with a moving-block placeholder; we commit the move when
/// the scheduled tick fires, which IS 2gt after the edge).
pub fn piston_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    if b != PISTON && b != STICKY_PISTON {
        return;
    }
    let (f, extended) = if b == PISTON {
        piston_decode(s)
    } else {
        sticky_piston_decode(s)
    };
    let [fx, fy, fz] = full_facing_vec(f);
    let powered = qc_powered(world, x, y, z);
    let set_state = |world: &mut World, ext: bool| {
        let ns = if b == PISTON {
            piston_state(f, ext)
        } else {
            sticky_piston_state(f, ext)
        };
        world.set_block_state(x, y, z, ns);
    };

    if powered && !extended {
        // try to extend: walk the push line
        let mut line: Vec<[i32; 3]> = Vec::new();
        let (mut px, mut py, mut pz) = (x + fx, y + fy, z + fz);
        let mut breakables: Vec<[i32; 3]> = Vec::new();
        let mut blocked = false;
        // walk up to 13 cells: 12 pushed + the 13th checked for fit
        for i in 0..=PISTON_PUSH_LIMIT {
            let tb = world.get_block(px, py, pz);
            if tb == AIR || tb == WATER {
                break; // open space — the line fits
            }
            if i == PISTON_PUSH_LIMIT {
                // the 13th cell is occupied by a movable solid: the
                // 12-block limit is exceeded (VERIFIED) → refuse
                blocked = !piston_breaks(tb);
                break;
            }
            if piston_unpushable(tb) {
                blocked = true;
                break;
            }
            if piston_breaks(tb) {
                breakables.push([px, py, pz]);
                break; // pop + extension proceeds one cell short
            }
            line.push([px, py, pz]);
            px += fx;
            py += fy;
            pz += fz;
        }
        if blocked {
            return; // cannot extend
        }
        // move the line forward (reverse order so nothing overwrites)
        for p in line.iter().rev() {
            let moving = world.get_block(p[0], p[1], p[2]);
            let _ = world.set_block(p[0] + fx, p[1] + fy, p[2] + fz, moving);
        }
        // vacate the head space only when a real shift happened (the
        // nearest line block moved away) — never clobber water/air fronts
        if !line.is_empty() {
            let _ = world.set_block(x + fx, y + fy, z + fz, AIR);
        }
        // broken blocks vanish (their drops are handled by the game
        // layer's block-break path in a real game; here they pop via the
        // observer/notification cascade — documented simplification)
        for p in breakables {
            let _ = world.set_block(p[0], p[1], p[2], AIR);
        }
        // notify the world around the moved region
        for p in line.iter() {
            on_block_changed(sched, world, p[0] + fx, p[1] + fy, p[2] + fz);
        }
        on_block_changed(sched, world, x + fx, y + fy, z + fz);
        set_state(world, true);
        on_block_changed(sched, world, x, y, z);
    } else if !powered && extended {
        // retract: sticky pulls the adjacent movable block
        set_state(world, false);
        let (hx, hy, hz) = (x + fx, y + fy, z + fz);
        if b == STICKY_PISTON {
            // the block beyond the head space (which we cleared on extend
            // — reconstruct: the block now sitting in the head's old spot)
            let pulled = world.get_block(hx + fx, hy + fy, hz + fz);
            if pulled != AIR && !piston_unpushable(pulled) && !piston_breaks(pulled) {
                let _ = world.set_block(hx, hy, hz, pulled);
                let _ = world.set_block(hx + fx, hy + fy, hz + fz, AIR);
                on_block_changed(sched, world, hx, hy, hz);
                on_block_changed(sched, world, hx + fx, hy + fy, hz + fz);
            } else {
                let _ = world.set_block(hx, hy, hz, AIR);
            }
        } else {
            let _ = world.set_block(hx, hy, hz, AIR);
        }
        on_block_changed(sched, world, x, y, z);
    }
}

/// dispenser/dropper update: fire ONE item on the rising edge (VERIFIED:
/// 4 game tick delay, single eject per activation). The actual eject is
/// done by the game layer (it owns the item system); this tick detects
/// the edge and marks the fire via the returned flag on the Sim.
pub fn dispenser_tick(world: &mut World, x: i32, y: i32, z: i32) -> bool {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    if b != DISPENSER && b != DROPPER {
        return false;
    }
    // rising edge: powered now (QC included — VERIFIED for dispensers)
    qc_powered(world, x, y, z)
}

/// hopper update gate: enabled unless powered (VERIFIED: redstone locks
/// hoppers). Transfer cadence (8gt) lives in the Sim tick loop.
pub fn hopper_enabled(world: &World, x: i32, y: i32, z: i32) -> bool {
    let s = world.get_state(x, y, z);
    if state_block(s) != HOPPER {
        return false;
    }
    // VERIFIED: redstone power locks hoppers (enabled=false). Derived
    // each tick rather than persisted — observably identical.
    power_at(world, x, y, z) == 0 && power_at(world, x, y + 1, z) == 0
}

#[cfg(test)]
mod phase3_tests {
    use super::*;
    use crate::containers::Containers;
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

    fn drain3(world: &mut World, sched: &mut TickScheduler, containers: &Containers, ticks: u64) {
        for _ in 0..ticks {
            let due = sched.tick();
            if due.is_empty() && sched.pending() == 0 {
                break;
            }
            for pos in due {
                let b = state_block(world.get_state(pos[0], pos[1], pos[2]));
                match b {
                    REDSTONE_WIRE => wire_tick(world, sched, pos[0], pos[1], pos[2]),
                    REDSTONE_TORCH => torch_tick(world, sched, pos[0], pos[1], pos[2]),
                    REPEATER => repeater_tick(world, sched, pos[0], pos[1], pos[2]),
                    COMPARATOR => comparator_tick(world, sched, containers, pos[0], pos[1], pos[2]),
                    PISTON | STICKY_PISTON => piston_tick(world, sched, pos[0], pos[1], pos[2]),
                    OBSERVER => observer_pulse(world, sched, pos[0], pos[1], pos[2]),
                    LEVER => lever_tick(world, pos[0], pos[1], pos[2]),
                    REDSTONE_LAMP => lamp_tick(world, sched, pos[0], pos[1], pos[2]),
                    _ => {}
                }
            }
        }
    }

    /// VERIFIED: repeater passes the signal (output powers wire at 15)
    /// after its 1..4 redstone-tick delay
    #[test]
    fn repeater_passes_and_delays_signal() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // lever → wire(in) → repeater → wire(out)
        // repeater at (3,65,0) facing WEST (idx 3): input cell = (4,65,0),
        // output cell = (2,65,0)
        w.set_block_state(0, 65, 0, lever_state(true));
        w.set_block_state(1, 65, 0, wire_state(0));
        w.set_block_state(2, 65, 0, wire_state(0));
        w.set_block_state(3, 65, 0, repeater_state(3, 1, false));
        w.set_block_state(4, 65, 0, wire_state(0));
        w.set_block_state(5, 65, 0, wire_state(0));
        // chain: 5 (15) → 4 (14) feeds the repeater; 2 (out) → 1 (14)
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain3(&mut w, &mut sched, &Containers::default(), 400);
        let (_, _, powered) = repeater_decode(w.get_state(3, 65, 0));
        assert!(powered, "repeater turned on");
        // the OUTPUT cell (facing west → output at +x) sees 15
        assert_eq!(wire_power(w.get_state(4, 65, 0)), 15, "output full power");
    }

    /// VERIFIED: a powered repeater facing into another repeater's side
    /// LOCKS it — output freezes even when its input drops
    #[test]
    fn repeater_lock_holds_output() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // main repeater M at (2,65,0) facing west (idx 3): input cell
        // (1,65,0), output cell (3,65,0). Locking repeater L at (2,65,1)
        // (a side cell of M) facing SOUTH (idx 2, input (2,65,2)) — its
        // output cell = (2,65,0) = M itself → locks M.
        w.set_block_state(1, 65, 0, lever_state(true)); // M's input
        w.set_block_state(2, 65, 0, repeater_state(3, 1, false));
        w.set_block_state(3, 65, 0, wire_state(0)); // M's output probe
        w.set_block_state(2, 65, 2, lever_state(true)); // L's input
        w.set_block_state(2, 65, 1, repeater_state(2, 1, false));
        on_block_changed(&mut sched, &w, 1, 65, 0);
        on_block_changed(&mut sched, &w, 2, 65, 2);
        drain3(&mut w, &mut sched, &Containers::default(), 600);
        assert_eq!(wire_power(w.get_state(3, 65, 0)), 15, "M output live");

        // cut M's INPUT — L still locks M → output must HOLD
        toggle_lever(&mut w, &mut sched, 1, 65, 0);
        drain3(&mut w, &mut sched, &Containers::default(), 600);
        assert_eq!(
            wire_power(w.get_state(3, 65, 0)),
            15,
            "locked repeater holds its output (VERIFIED lock behavior)"
        );
        // now unlock (kill L) → M's output dies with its dead input
        toggle_lever(&mut w, &mut sched, 2, 65, 2);
        drain3(&mut w, &mut sched, &Containers::default(), 600);
        assert_eq!(
            wire_power(w.get_state(3, 65, 0)),
            0,
            "unlocked repeater follows its (dead) input"
        );
    }

    /// VERIFIED formulas: compare out = rear × [l ≤ rear ∧ r ≤ rear];
    /// subtract out = max(rear − max(l, r), 0)
    #[test]
    fn comparator_modes_match_wiki_formulas() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        let containers = Containers::default();
        // comparator at (2,65,0) facing EAST (idx 1): input cell (3,65,0),
        // output cell (1,65,0); sides at (2,65,±1)
        w.set_block_state(3, 65, 0, lever_state(true)); // rear = 15
        w.set_block_state(2, 65, 1, wire_state(7)); // left = 7
        w.set_block_state(2, 65, -1, wire_state(7)); // right = 7
        w.set_block_state(2, 65, 0, comparator_state(1, false, false));
        comparator_tick(&mut w, &mut sched, &containers, 2, 65, 0);
        let (_, _, p) = comparator_decode(w.get_state(2, 65, 0));
        assert!(p, "compare: 15 ≥ 7 → passes rear");

        // sides higher than rear: wire at 15 on a side → output OFF
        w.set_block_state(2, 65, 1, wire_state(15));
        w.set_block_state(2, 65, 0, comparator_state(1, false, true));
        comparator_tick(&mut w, &mut sched, &containers, 2, 65, 0);
        let (_, _, p2) = comparator_decode(w.get_state(2, 65, 0));
        assert!(p2, "compare: side 15 == rear 15 → still passes (≤)");
        // make the side strictly greater: hand-set a fresh 15 beside a
        // rear of 14 (wire decay) — use two wires: rear reads 14
        w.set_block_state(3, 65, 0, wire_state(14));
        comparator_tick(&mut w, &mut sched, &containers, 2, 65, 0);
        let (_, _, p3) = comparator_decode(w.get_state(2, 65, 0));
        assert!(!p3, "compare: side 15 > rear 14 → output off");

        // subtract: rear 14 − max(15, 7) = 0 → off
        w.set_block_state(2, 65, 0, comparator_state(1, true, true));
        comparator_tick(&mut w, &mut sched, &containers, 2, 65, 0);
        let (_, _, p4) = comparator_decode(w.get_state(2, 65, 0));
        assert!(!p4, "subtract: 14 − 15 = 0 → off");

        // subtract with lower sides: 14 − 7 = 7 → on
        w.set_block_state(2, 65, 1, wire_state(7));
        w.set_block_state(2, 65, 0, comparator_state(1, true, false));
        comparator_tick(&mut w, &mut sched, &containers, 2, 65, 0);
        let (_, _, p5) = comparator_decode(w.get_state(2, 65, 0));
        assert!(p5, "subtract: 14 − 7 = 7 > 0 → on");
    }

    /// VERIFIED: pistons push at most 12 blocks; the 13th refuses
    #[test]
    fn piston_push_limit_is_twelve() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // piston at (0,65,0) facing EAST (idx 1)
        w.set_block_state(0, 65, 0, piston_state(1, false));
        for x in 1..=12i32 {
            w.set_block_state(x, 65, 0, DIRT as u16);
        }
        w.set_block_state(0, 66, 0, lever_state(true));
        on_block_changed(&mut sched, &w, 0, 66, 0);
        drain3(&mut w, &mut sched, &Containers::default(), 100);
        let (_, ext) = piston_decode(w.get_state(0, 65, 0));
        assert!(ext, "12 blocks push (the limit — VERIFIED)");
        assert_eq!(w.get_block(13, 65, 0), DIRT, "line shifted by one");
        assert_eq!(w.get_block(1, 65, 0), AIR, "head space vacated");

        // 13 blocks: must NOT extend
        let mut w2 = flat_world();
        let mut sched2 = TickScheduler::new();
        w2.set_block_state(0, 65, 0, piston_state(1, false));
        for x in 1..=13i32 {
            w2.set_block_state(x, 65, 0, DIRT as u16);
        }
        w2.set_block_state(0, 66, 0, lever_state(true));
        on_block_changed(&mut sched2, &w2, 0, 66, 0);
        drain3(&mut w2, &mut sched2, &Containers::default(), 100);
        let (_, ext2) = piston_decode(w2.get_state(0, 65, 0));
        assert!(!ext2, "13 blocks refuse (12-block limit — VERIFIED)");
        assert_eq!(w2.get_block(13, 65, 0), DIRT, "nothing moved");
    }

    /// VERIFIED (Java intentional): quasi-connectivity — a lever adjacent
    /// to the cell ABOVE the piston powers it
    #[test]
    fn quasi_connectivity_powers_the_piston() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // piston at (0,65,0); lever at (2,66,0): adjacent to the cell
        // above the piston (1,66,0)? — use (1,66,0) directly (dist 1)
        w.set_block_state(0, 65, 0, piston_state(1, false));
        w.set_block_state(1, 66, 0, lever_state(true));
        on_block_changed(&mut sched, &w, 1, 66, 0);
        drain3(&mut w, &mut sched, &Containers::default(), 100);
        let (_, ext) = piston_decode(w.get_state(0, 65, 0));
        assert!(ext, "QC: power above the piston extended it");
    }

    /// VERIFIED: observers pulse ON for 2 game ticks when the watched
    /// block changes, then turn off
    #[test]
    fn observer_pulses_on_neighbor_change() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // observer at (0,65,0) facing EAST (idx 1) watches (1,65,0)
        w.set_block_state(0, 65, 0, observer_state(1, false));
        w.set_block_state(1, 65, 0, DIRT as u16);
        on_block_changed(&mut sched, &w, 1, 65, 0);
        // run exactly 3 ticks: the pulse fires at tick 2 and stays ON
        // until the 2-game-tick expiry (tick 4)
        for _ in 0..3 {
            let due = sched.tick();
            for pos in due {
                if state_block(w.get_state(pos[0], pos[1], pos[2])) == OBSERVER {
                    observer_pulse(&mut w, &mut sched, pos[0], pos[1], pos[2]);
                }
            }
        }
        let (_, powered) = observer_decode(w.get_state(0, 65, 0));
        assert!(powered, "observer pulsed ON");
        // the pulse self-terminates after 2 game ticks
        for _ in 0..6 {
            let due = sched.tick();
            for pos in due {
                if state_block(w.get_state(pos[0], pos[1], pos[2])) == OBSERVER {
                    observer_pulse(&mut w, &mut sched, pos[0], pos[1], pos[2]);
                }
            }
        }
        let (_, powered2) = observer_decode(w.get_state(0, 65, 0));
        assert!(!powered2, "pulse ended after 2 game ticks (VERIFIED)");
        // REGRESSION (caught live in the Phase 3 browser E2E): the
        // observer must SETTLE — no self-rescheduling entries may remain.
        // Before the on_block_changed exclusion, a powered observer's own
        // state change re-scheduled it → toggle → re-schedule → a
        // free-running 2gt clock with a growing queue (the probe sat at
        // powered=true minutes after one pulse). Run LONG enough that any
        // oscillation would flip the state repeatedly, then require: off
        // AND an empty scheduler.
        for _ in 0..64 {
            let due = sched.tick();
            for pos in due {
                if state_block(w.get_state(pos[0], pos[1], pos[2])) == OBSERVER {
                    observer_pulse(&mut w, &mut sched, pos[0], pos[1], pos[2]);
                }
            }
        }
        let (_, powered3) = observer_decode(w.get_state(0, 65, 0));
        assert!(!powered3, "observer settled off (no self-oscillation)");
        assert_eq!(
            sched.pending(),
            0,
            "scheduler drained — no perpetuating observer entries"
        );
    }

    /// VERIFIED: the observer's pulse emits from its BACK face (opposite
    /// the watched direction) — a wire behind the observer reads 15,
    /// a wire beside it reads nothing
    #[test]
    fn observer_output_comes_from_the_back() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // observer at (0,65,0) facing EAST watches (1,65,0); its back is
        // (−1,65,0). Back wire + side wire (control).
        w.set_block_state(0, 65, 0, observer_state(1, false));
        w.set_block_state(-1, 65, 0, wire_state(0));
        w.set_block_state(0, 65, 1, wire_state(0));
        // dirt appears AT the watched cell → change → pulse
        w.set_block_state(1, 65, 0, DIRT as u16);
        on_block_changed(&mut sched, &w, 1, 65, 0);
        // t2: observer fires ON; t4: the wires tick (insertion order puts
        // the wire entry before the pulse-off entry, so the wire reads
        // the observer still mid-pulse)
        for _ in 0..4 {
            let due = sched.tick();
            for pos in due {
                match state_block(w.get_state(pos[0], pos[1], pos[2])) {
                    OBSERVER => observer_pulse(&mut w, &mut sched, pos[0], pos[1], pos[2]),
                    REDSTONE_WIRE => wire_tick(&mut w, &mut sched, pos[0], pos[1], pos[2]),
                    _ => {}
                }
            }
        }
        // mid-pulse read: the BACK wire is powered, the side wire is not
        assert_eq!(
            wire_power(w.get_state(-1, 65, 0)),
            15,
            "back wire sees the pulse"
        );
        assert_eq!(
            wire_power(w.get_state(0, 65, 1)),
            0,
            "side wire sees nothing (output is the back face only)"
        );
    }

    /// verified constants
    #[test]
    fn phase3_constants_match_the_wiki() {
        assert_eq!(PISTON_PUSH_LIMIT, 12);
        assert_eq!(COMPARATOR_DELAY, 2);
        assert_eq!(OBSERVER_PULSE, 2);
        assert_eq!(HOPPER_COOLDOWN, 8);
        assert_eq!(DISPENSER_DELAY, 4);
    }
}

#[cfg(test)]
mod e1_lamp_tests {
    use super::*;

    fn flat_world() -> World {
        let mut w = World::new(3);
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

    fn drain(w: &mut World, sched: &mut TickScheduler, max: u64) {
        for _ in 0..max {
            let due = sched.tick();
            if due.is_empty() && sched.pending() == 0 {
                break;
            }
            for pos in due {
                let b = state_block(w.get_state(pos[0], pos[1], pos[2]));
                match b {
                    REDSTONE_WIRE => wire_tick(w, sched, pos[0], pos[1], pos[2]),
                    REDSTONE_TORCH => torch_tick(w, sched, pos[0], pos[1], pos[2]),
                    LEVER => lever_tick(w, pos[0], pos[1], pos[2]),
                    REDSTONE_LAMP => lamp_tick(w, sched, pos[0], pos[1], pos[2]),
                    _ => {}
                }
            }
        }
    }

    /// Phase E1 (VERIFIED w/Redstone_Lamp): the lamp lights next to an
    /// on lever, and goes dark after the lever turns off (our redstone
    /// backbone's 2gt approximation of the vanilla 4gt delay).
    #[test]
    fn lamp_lights_and_turns_off_with_the_lever() {
        let mut w = flat_world();
        let mut sched = TickScheduler::new();
        // lever at (0,65,0), lamp at (1,65,0)
        w.set_block_state(0, 65, 0, lever_state(true));
        w.set_block_state(1, 65, 0, REDSTONE_LAMP_STATE);
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 50);
        assert_eq!(
            w.get_state(1, 65, 0),
            REDSTONE_LAMP_LIT,
            "lit next to the ON lever (VERIFIED)"
        );

        // lever OFF → the lamp goes dark on the next redstone tick
        let _ = toggle_lever(&mut w, &mut sched, 0, 65, 0);
        drain(&mut w, &mut sched, 50);
        assert_eq!(
            w.get_state(1, 65, 0),
            REDSTONE_LAMP_STATE,
            "off after the lever (2gt-backbone approximation of 4gt)"
        );
        // emission rides the state (VERIFIED: lit = 15, off = 0)
        assert_eq!(state_emissive(REDSTONE_LAMP_LIT), 15);
        assert_eq!(state_emissive(REDSTONE_LAMP_STATE), 0);
    }
}
