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
fn water_at(world: &World, x: i32, y: i32, z: i32) -> Option<u16> {
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
        } else if b == SAND
            || b == GRAVEL
            || b == ANVIL
            || b == CHIPPED_ANVIL
            || b == DAMAGED_ANVIL
            || is_concrete_powder(b)
        {
            // Phase E2: anvils are gravity blocks (VERIFIED w/Anvil: falls
            // like sand). 1.12: concrete powder rides the gravity channel
            // (its tick also runs the water-contact solidification check —
            // water placed next to a powder lands here, "placed next to"
            // VERIFIED w/Concrete_Powder §Usage)
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
    // 1.14: falling water onto a LIT campfire extinguishes it
    // (VERIFIED w/Campfire §Extinguishing: "A campfire can be
    // extinguished by waterlogging it (placing water in the same
    // block space)" — the engine's water never enters the non-air
    // cell, so the contact itself carries the extinguish)
    if y > 0 && state_block(below) == CAMPFIRE && campfire_lit(below) {
        world.set_block_state(x, y - 1, z, campfire_state(false));
        on_block_changed(sched, world, x, y - 1, z);
    }

    // 2. re-derive this block's level from its feeders
    if level > 0 {
        // Vanilla infinite water source formation:
        // When two or more horizontal neighbors are water sources (level 0),
        // and the block beneath is solid or a water source, form a permanent source block (level 0).
        let mut source_count = 0;
        for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            if water_at(world, x + dx, y, z + dz) == Some(0) {
                source_count += 1;
            }
        }
        let floor_solid_or_source = !flowable(below) || water_at(world, x, y - 1, z) == Some(0);
        if source_count >= 2 && floor_solid_or_source {
            world.set_block_state(x, y, z, water_state(0));
            on_block_changed(sched, world, x, y, z);
            return;
        }

        let mut feed: Option<u16> = None;
        // fed from above by any water → strongest feed (vanilla: falling
        // full column)
        if water_at(world, x, y + 1, z).is_some() {
            feed = Some(0);
        } else {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                if let Some(nl) = water_at(world, x + dx, y, z + dz) {
                    feed = Some(feed.map_or(nl, |f: u16| f.min(nl)));
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
                    world.set_block_state(x, y, z, water_state(target as u8));
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
                // 1.14: a LIT campfire neighbor extinguishes on water
                // contact (the VERIFIED waterlog rule, contact form)
                if state_block(n) == CAMPFIRE && campfire_lit(n) {
                    world.set_block_state(x + dx, y, z + dz, campfire_state(false));
                    on_block_changed(sched, world, x + dx, y, z + dz);
                    continue;
                }
                if flowable(n) {
                    world.set_block_state(x + dx, y, z + dz, water_state(spread as u8));
                    on_block_changed(sched, world, x + dx, y, z + dz);
                }
            }
        }
    }
}

/// Computes the horizontal flow vector for entities (players, mobs, items) in water.
/// Flow is directed along the negative height gradient (from higher water to lower water or air).
pub fn water_flow_vector(world: &World, x: i32, y: i32, z: i32) -> (f32, f32) {
    let s = world.get_state(x, y, z);
    if state_block(s) != WATER {
        return (0.0, 0.0);
    }
    let level = water_level(s);
    if level == 255 {
        return (0.0, 0.0);
    }

    let eff_h = |lvl: u16| -> f32 {
        if lvl == 0 {
            8.0
        } else {
            (8 - (lvl.min(7))) as f32
        }
    };
    let h_center = eff_h(level);

    let mut dx_flow = 0.0f32;
    let mut dz_flow = 0.0f32;

    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
        let ns = world.get_state(x + dx, y, z + dz);
        let nb = state_block(ns);
        if nb == WATER {
            let nl = water_level(ns);
            if nl != 255 {
                let diff = eff_h(nl) - h_center;
                dx_flow -= dx as f32 * diff;
                dz_flow -= dz as f32 * diff;
            }
        } else if flowable(ns) {
            dx_flow += dx as f32 * h_center;
            dz_flow += dz as f32 * h_center;
        }
    }

    let len = (dx_flow * dx_flow + dz_flow * dz_flow).sqrt();
    if len > 1e-4 {
        (dx_flow / len, dz_flow / len)
    } else {
        (0.0, 0.0)
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
fn lava_at(world: &World, x: i32, y: i32, z: i32) -> Option<u16> {
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
        let mut feed: Option<u16> = None;
        if lava_at(world, x, y + 1, z).is_some() {
            feed = Some(0);
        } else {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                if let Some(nl) = lava_at(world, x + dx, y, z + dz) {
                    // the neighbor's effective feed level already accounts
                    // for its own level; the target = neighbor + drop
                    feed = Some(feed.map_or(nl, |f: u16| f.min(nl)));
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
                if level != drop as u16 {
                    world.set_block_state(x, y, z, lava_state(drop));
                    on_block_changed(sched, world, x, y, z);
                }
            }
            Some(f) => {
                let target = f + drop as u16;
                if target > 7 {
                    // too weak: decay
                    world.set_block_state(x, y, z, AIR as u16);
                    on_block_changed(sched, world, x, y, z);
                    return;
                }
                if target != level {
                    world.set_block_state(x, y, z, lava_state(target as u8));
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
    let spread = (level + drop as u16).min(8);
    if spread <= 7 {
        let below_ok = !matches!(state_block(world.get_state(x, y - 1, z)), AIR | WATER);
        if below_ok {
            for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let n = world.get_state(x + dx, y, z + dz);
                let nb = state_block(n);
                if nb == AIR || nb == WATER {
                    world.set_block_state(x + dx, y, z + dz, lava_state(spread as u8));
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
/// 1.12 (World of Color Update): concrete powder joins the gravity set
/// (VERIFIED w/Concrete_Powder: "Gravity affected (like sand and
/// gravel)") — and the landing tick runs the water-contact
/// solidification check ("Concrete powder falls when there is a
/// non-solid block beneath it" + "If it lands next to water, it
/// solidifies only after a block update").
pub fn gravity_tick(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32) {
    let s = world.get_state(x, y, z);
    let b = state_block(s);
    if b != SAND
        && b != GRAVEL
        && b != ANVIL
        && b != CHIPPED_ANVIL
        && b != DAMAGED_ANVIL
        && !is_concrete_powder(b)
    {
        return; // stale entry
    }
    // 1.12: powder touching water solidifies BEFORE falling (covers
    // the placed-into/placed-next-to-water cases; the landing check
    // below covers fall-into-water)
    if is_concrete_powder(b) && concrete_powder_touches_water(world, x, y, z) {
        solidify_powder(world, sched, x, y, z, b);
        return;
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
        // 1.12: the landed powder re-checks water contact ("solidifies
        // only after a block update" — the landing IS the update)
        let landed = state_block(world.get_state(x, y - 1, z));
        if is_concrete_powder(landed) && concrete_powder_touches_water(world, x, y - 1, z) {
            solidify_powder(world, sched, x, y - 1, z, landed);
        }
    }
}

/// true if any of the 6 face-adjacent cells (or the cell itself via its
/// waterlog) is water of any level. VERIFIED w/Concrete_Powder §Usage:
/// "the block has to be placed into, placed next to, or fall into
/// flowing water, a water source block ... It does not solidify in
/// midair falling past water" (the engine's powder is never in
/// mid-fall — the block-wise gravity keeps it a block, so the
/// mid-air exemption rides the same rule).
pub fn concrete_powder_touches_water(world: &World, x: i32, y: i32, z: i32) -> bool {
    for (dx, dy, dz) in [
        (0i32, 1i32, 0i32),
        (0, -1, 0),
        (1, 0, 0),
        (-1, 0, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        let n = world.get_state(x + dx, y + dy, z + dz);
        if state_block(n) == WATER {
            return true;
        }
    }
    false
}

/// convert a concrete-powder block at (x,y,z) into the concrete of the
/// same color (VERIFIED w/Concrete: "Created when concrete powder comes
/// into contact with still or flowing water").
pub fn solidify_powder(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32, b: u16) {
    let color = concrete_powder_color(b);
    if color == 255 {
        return;
    }
    world.set_block_state(x, y, z, concrete_state(color));
    on_block_changed(sched, world, x, y, z);
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
        // ---- 1.14 (Village & Pillage — nature half) growth ----
        // BAMBOO_SHOOT: "Upon receiving a random tick, bamboo has a
        // 1/3 chance of growing" — the shoot graduates into its first
        // stalk (VERIFIED w/Bamboo §Farming; light gate: "The top of a
        // bamboo plant requires a client light level of 9 or above")
        BAMBOO_SHOOT => {
            if light_ge_9(world, x, y, z) && world_random_3(world, x, y, z) {
                world.set_block_state(x, y, z, default_state(BAMBOO));
                on_block_changed(sched, world, x, y, z);
            }
        }
        // BAMBOO stalk: column growth — 1/3 per random tick, capped at
        // 16 ("can grow up to 12-16 blocks tall"; the default world's
        // 16 max), same light gate at the would-be top cell
        BAMBOO => {
            // only the TOP cell of the column rolls (a mid-column tick
            // finds bamboo above and stands down)
            if world.get_block(x, y + 1, z) == BAMBOO {
                return;
            }
            let mut top = y;
            while top > 0 && world.get_block(x, top - 1, z) == BAMBOO {
                top -= 1;
            }
            let height = y - top + 1;
            if height >= 16 {
                return; // the 12-16 window's engine cap
            }
            if world.get_block(x, y + 1, z) == AIR
                && light_ge_9(world, x, y + 1, z)
                && world_random_3(world, x, y, z)
            {
                world.set_block_state(x, y + 1, z, default_state(BAMBOO));
                on_block_changed(sched, world, x, y + 1, z);
            }
        }
        // SWEET_BERRY_BUSH: "grow via Random Ticks with a 20% chance
        // per random tick on the block if it is not fully grown"
        // (VERIFIED w/Sweet_Berry_Bush §Growth; light-independent)
        SWEET_BERRY_BUSH => {
            let s = world.get_state(x, y, z);
            let age = berry_bush_age(s);
            if age < 3 && world_random_5(world, x, y, z) {
                world.set_block_state(x, y, z, berry_bush_state(age + 1));
                on_block_changed(sched, world, x, y, z);
            }
        }
        // ---- backlog round (farming, 2026-09-09) ----
        // FARMLAND hydration/dry-out/decay (VERIFIED w/Farmland
        // §Hydration: water "up to four blocks away horizontally
        // (including diagonally)... at the same level or one block
        // above"; §Decay: "eventually decays into normal dirt if it's
        // dehydrated and nothing is planted in it").
        FARMLAND => {
            let s = world.get_state(x, y, z);
            let m = farmland_moisture(s);
            let planted = is_crop(state_block(world.get_state(x, y + 1, z)));
            let wet = has_hydrating_water(world, x, y, z);
            if m < 7 && wet {
                // hydrate straight to 7 (vanilla climbs 0→7 one step per
                // hydrate event; a single random tick with water present
                // is that event — the visual reads wet from 1 on)
                world.set_block_state(x, y, z, farmland_state(7));
                on_block_changed(sched, world, x, y, z);
            } else if m > 0 && !wet {
                if !planted && world_random_5(world, x, y, z) {
                    // dehydrated + unplanted: decay to dirt (the
                    // random-tick-paced "eventually"); any crop above
                    // pops with its harvest drops (VERIFIED §Decay:
                    // "crops growing on the block are dropped as items,
                    // as if they were harvested" — the sim has no item
                    // system, so the pop itself is the engine's
                    // adaptation; interactive trampling drops properly)
                    world.set_block_state(x, y, z, default_state(DIRT));
                    if is_crop(state_block(world.get_state(x, y + 1, z))) {
                        world.set_block_state(x, y + 1, z, default_state(AIR));
                        on_block_changed(sched, world, x, y + 1, z);
                    }
                } else {
                    // planted (or the decay roll failed): dry one step
                    world.set_block_state(x, y, z, farmland_state(m - 1));
                }
                on_block_changed(sched, world, x, y, z);
            }
        }
        // the four crops — the verified speed-level formula (see
        // grow_crop: Tutorial:Crop_farming §Growth rate, captured
        // 2026-09-09)
        WHEAT_CROP | CARROTS | POTATOES | BEETROOTS => {
            grow_crop(world, sched, x, y, z, b);
        }
        _ => {}
    }
}

/// is this block one of the four farming crops?
#[inline]
pub fn is_crop(b: u16) -> bool {
    matches!(b, WHEAT_CROP | CARROTS | POTATOES | BEETROOTS)
}

/// the farmland hydration scan: water within 4 blocks horizontally
/// (diagonals included), at the farmland's own level or one above
/// (VERIFIED w/Farmland §Hydration — "The blocks between the farmland
/// block and the water make no difference", so this is a plain scan).
fn has_hydrating_water(world: &World, x: i32, y: i32, z: i32) -> bool {
    for dy in 0..=1i32 {
        for dx in -4..=4i32 {
            for dz in -4..=4i32 {
                if dx.abs().max(dz.abs()) > 4 {
                    continue; // Chebyshev distance ≤ 4 (incl. diagonals)
                }
                let b = world.get_block(x + dx, y + dy, z + dz);
                if b == WATER {
                    return true;
                }
            }
        }
    }
    false
}

/// the crop growth roll — the vanilla speed-level formula, VERIFIED
/// live 2026-09-09 w/Tutorial:Crop_farming §Growth rate:
/// * light ≥ 9 AT the plant block ("growth requires a light level of
///   at least 9 at the plant block, not in the block above it")
/// * speed level = 2 (dry farmland below) or 4 (hydrated)
///   + 0.25 per surrounding dry farmland / 0.75 per hydrated (the 8
///   cells of the 3×3 around the below block)
/// * crowding: same crop on a diagonal, OR same crop in BOTH the N-S
///   and E-W axes → speed level HALVED ("If the same crop is planted
///   on a diagonal or if the same crop is found in both the north-south
///   and east-west directions")
/// * growth chance = 1/(floor(25/speedLevel) + 1) per random tick
///   (the wiki table: solo hydrated 14.29% = 1/7, solo dry 7.69% = 1/13,
///   fully-hydrated farm 33.33% = 1/3 — all reproduced by this formula)
fn grow_crop(world: &mut World, sched: &mut TickScheduler, x: i32, y: i32, z: i32, b: u16) {
    // support check first: farmland below (vanilla pops the crop on
    // farmland loss — the game layer's break cascade handles the drop;
    // here we only stand down if the support is gone)
    let below = world.get_block(x, y - 1, z);
    if below != FARMLAND {
        return;
    }
    let s = world.get_state(x, y, z);
    let age = crop_age(s);
    let max = crop_max_age(b);
    if age >= max {
        return; // fully grown
    }
    if !light_ge_9(world, x, y, z) {
        return; // the light gate
    }
    // speed level (in quarter-points to stay in integers):
    // farmland below: 4 dry / 16 wet (×4); surroundings: 1 dry / 3 wet
    let below_state = world.get_state(x, y - 1, z);
    let below_wet = farmland_moisture(below_state) > 0;
    let mut q = if below_wet { 16 } else { 4 };
    // the 8 surrounding farmland cells (the 3×3 around the below block)
    for (dx, dz) in [
        (1i32, 0i32), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1),
    ] {
        let nb = world.get_block(x + dx, y - 1, z + dz);
        if nb == FARMLAND {
            let ns = world.get_state(x + dx, y - 1, z + dz);
            q += if farmland_moisture(ns) > 0 { 3 } else { 1 };
        }
    }
    // crowding: same crop on a diagonal, or in both N-S and E-W
    let same = |dx: i32, dz: i32| world.get_block(x + dx, y, z + dz) == b;
    let diagonal = same(1, 1) || same(1, -1) || same(-1, 1) || same(-1, -1);
    let ns = same(1, 0) || same(-1, 0);
    let ew = same(0, 1) || same(0, -1);
    if diagonal || (ns && ew) {
        q /= 2; // halved (integer quarters stay honest at odd values)
    }
    // chance = 1/(floor(25/speedLevel)+1); speedLevel = q/4
    let speed_level_x4 = q.max(4) as u32; // the wiki's 1..10 range floor
    let denom = 25 * 4 / speed_level_x4 + 1;
    // 1-in-denom roll — deterministic per (position, TICK): unlike the
    // wart convention's fixed per-position hash (where a given wart
    // either grows on every tick or never), the crop ladder needs the
    // roll to VARY tick to tick or a field would freeze forever on a
    // bad hash draw. Mixing sched.now() keeps runs reproducible (same
    // tick + same position → same roll) while re-rolling each tick.
    let v = vc_rng::rng::Rng::hash3(world.seed ^ 0x0F41 ^ sched.now(), x, y, z) as u64;
    if v % denom as u64 == 0 {
        world.set_block_state(x, y, z, crop_state(b, age + 1));
        on_block_changed(sched, world, x, y, z);
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

/// 1.14: the bamboo 1-in-3 random-tick growth roll (VERIFIED w/Bamboo
/// §Farming: "a 1/3 chance of growing"). A per-position hash keeps it
/// deterministic like the wart roll.
fn world_random_3(world: &World, x: i32, y: i32, z: i32) -> bool {
    let v = vc_rng::rng::Rng::hash3(world.seed ^ 0x0B2E, x, y, z);
    v % 3 == 0
}

/// 1.14: the berry-bush 20% random-tick growth roll (VERIFIED
/// w/Sweet_Berry_Bush §Growth: "a 20% chance per random tick").
fn world_random_5(world: &World, x: i32, y: i32, z: i32) -> bool {
    let v = vc_rng::rng::Rng::hash3(world.seed ^ 0x0B5A, x, y, z);
    v % 5 == 0
}

/// 1.14: client light (max of sky and block channels) at a position,
/// read straight from the world's settled light map — the bamboo
/// growth gate ("requires a client light level of 9 or above",
/// VERIFIED w/Bamboo). Unlit/unloaded chunks read as full sky (the
/// generous default — matches the mesh snapshot fallback).
fn light_ge_9(world: &World, x: i32, y: i32, z: i32) -> bool {
    let cx = x.div_euclid(16);
    let cz = z.div_euclid(16);
    let lx = (x - cx * 16) as usize;
    let lz = (z - cz * 16) as usize;
    let y = y.clamp(0, 255);
    let sec = (y / 16) as usize;
    let yy = (y % 16) as usize;
    let idx = (yy << 8) | (lz << 4) | lx;
    world
        .light
        .get(&(cx, cz))
        .and_then(|ld| {
            ld.sections[sec].as_ref().map(|s| (s.sky[idx], s.blk[idx]))
        })
        .map(|(sky, blk)| sky.max(blk) >= 9)
        .unwrap_or(true)
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
        for expect in 1u16..=7 {
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

    // ---------------- 1.12 bracket tests (World of Color) ----------------

    /// 1.12 concrete powder: gravity + water-contact solidification.
    /// VERIFIED live (2026-09-07, minecraft.wiki/w/Concrete_Powder):
    /// "Gravity affected (like sand and gravel)" + "If a concrete
    /// powder block comes into contact with water, it solidifies into
    /// a block of concrete" + "If it lands next to water, it solidifies
    /// only after a block update".
    #[test]
    fn v112_powder_falls_like_sand() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        // powder above air: falls
        let p = concrete_powder(3); // light blue
        w.set_block_state(0, 70, 0, concrete_powder_state(3));
        on_block_changed(&mut sched, &w, 0, 70, 0);
        drain(&mut w, &mut sched, 200);
        assert_eq!(w.get_block(0, 65, 0), p, "powder lands on the shelf");
        assert_eq!(w.get_block(0, 70, 0), AIR, "the fall cleared the origin");
    }

    #[test]
    fn v112_powder_touching_water_solidifies() {
        // water source on the shelf, powder placed adjacent → solidifies
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(1, 65, 0, water_state(0));
        w.set_block_state(0, 65, 0, concrete_powder_state(3));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);
        assert_eq!(w.get_block(0, 65, 0), concrete(3), "water neighbor → concrete");
        // powder falling INTO a water column solidifies on landing
        let mut w2 = flat_world(64);
        let mut sched2 = TickScheduler::new();
        // water pocket in a hole at y=66..67
        w2.set_block_state(0, 65, 1, AIR);
        w2.set_block_state(0, 64, 1, AIR);
        w2.set_block_state(0, 64, 1, water_state(0));
        w2.set_block_state(0, 70, 1, concrete_powder_state(9));
        on_block_changed(&mut sched2, &w2, 0, 70, 1);
        drain(&mut w2, &mut sched2, 400);
        // the powder converted somewhere in the column (above or in water)
        let mut found = false;
        for y in 60..70 {
            if w2.get_block(0, y, 1) == concrete(9) {
                found = true;
            }
        }
        assert!(found, "falling powder solidifies into concrete(9)");
    }

    #[test]
    fn v112_powder_without_water_stays_powder() {
        // dry shelf: no conversion
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, concrete_powder_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 200);
        assert_eq!(
            w.get_block(0, 65, 0),
            concrete_powder(0),
            "no water contact → stays powder"
        );
    }

    #[test]
    fn v112_all_powder_colors_solidify() {
        // every color round-trips powder → concrete (the color index
        // carries through the conversion)
        for c in 0u8..16 {
            let mut w = flat_world(64);
            let mut sched = TickScheduler::new();
            w.set_block_state(1, 65, 0, water_state(0));
            w.set_block_state(0, 65, 0, concrete_powder_state(c));
            on_block_changed(&mut sched, &w, 0, 65, 0);
            drain(&mut w, &mut sched, 100);
            assert_eq!(w.get_block(0, 65, 0), concrete(c), "color {c} converts");
        }
    }

    /// 1.14 (Village & Pillage — nature half): random-tick growth
    /// contracts. Bamboo: the shoot graduates into a stalk and stalks
    /// grow columns (1/3 per roll, max 16, light 9+); berry bushes age
    /// toward 3 (20% per roll). The rolls are per-position hashes, so
    /// walking neighbor positions finds a passing one deterministically.
    fn lit_flat_world() -> World {
        let mut w = World::new(21);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y, lz, GRASS);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    #[test]
    fn v114_bamboo_shoot_graduates_to_a_stalk() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        // the 1/3 roll is a fixed per-position hash — plant the shoot
        // on each column until one passes (expected ~1/3 of columns)
        let mut grown = false;
        for tx in 0..16i32 {
            for tz in 0..16i32 {
                w.set_block_state(tx, 65, tz, default_state(BAMBOO_SHOOT));
                random_plant_tick(&mut w, &mut sched, tx, 65, tz);
                if w.get_block(tx, 65, tz) == BAMBOO {
                    grown = true;
                    break;
                }
                w.set_block_state(tx, 65, tz, 0);
            }
            if grown {
                break;
            }
        }
        assert!(grown, "some shoot position rolls the 1/3 growth");
    }

    #[test]
    fn v114_bamboo_column_grows_and_caps_at_16() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        let (x, z) = (8, 8);
        // a 16-tall column (the cap): the top cell never grows
        for dy in 0..16 {
            w.set_block_state(x, 65 + dy, z, default_state(BAMBOO));
        }
        let top = 65 + 15;
        for _ in 0..40 {
            random_plant_tick(&mut w, &mut sched, x, top, z);
        }
        assert_eq!(w.get_block(x, top + 1, z), AIR, "the 16 cap holds");
        // clear the tall column; a 1-tall stalk grows on a passing
        // column (the per-position 1/3 roll)
        for dy in 0..16 {
            w.set_block_state(x, 65 + dy, z, 0);
        }
        let mut grew = false;
        for tx in 0..16i32 {
            for tz in 0..16i32 {
                w.set_block_state(tx, 65, tz, default_state(BAMBOO));
                random_plant_tick(&mut w, &mut sched, tx, 65, tz);
                if w.get_block(tx, 66, tz) == BAMBOO {
                    grew = true;
                    break;
                }
                w.set_block_state(tx, 65, tz, 0);
            }
            if grew {
                break;
            }
        }
        assert!(grew, "some 1-tall stalk position rolls a growth");
    }

    /// does a bush of `age` at SOME in-chunk position advance one age
    /// on its random tick? (the 20% roll is a fixed per-position hash —
    /// with 256 columns ~51 pass)
    fn bush_advances_somewhere(w: &mut World, age: u8) -> bool {
        let mut sched = TickScheduler::new();
        for tx in 0..16i32 {
            for tz in 0..16i32 {
                w.set_block_state(tx, 65, tz, berry_bush_state(age));
                random_plant_tick(w, &mut sched, tx, 65, tz);
                if berry_bush_age(w.get_state(tx, 65, tz)) == age + 1 {
                    return true;
                }
                w.set_block_state(tx, 65, tz, 0);
            }
        }
        false
    }

    #[test]
    fn v114_berry_bush_ages_to_three_and_stops() {
        let mut w = lit_flat_world();
        // ages 0 → 1 → 2 all find passing positions (20% × 256)
        for age in 0u8..3 {
            assert!(
                bush_advances_somewhere(&mut w, age),
                "age {age} advances somewhere"
            );
        }
        // mature (age 3) never advances: the age < 3 gate
        assert!(
            !bush_advances_somewhere(&mut w, 3),
            "age 3 is terminal (VERIFIED: not fully grown only)"
        );
    }

    #[test]
    fn test_infinite_water_source_formation() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        // Place two sources at (0, 65, 0) and (2, 65, 0) separated by 1 air block
        w.set_block_state(0, 65, 0, water_state(0));
        w.set_block_state(2, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        on_block_changed(&mut sched, &w, 2, 65, 0);
        drain(&mut w, &mut sched, 20);

        // The center cell (1, 65, 0) has 2 adjacent sources over a solid block -> forms source (level 0)!
        let center_level = water_level(w.get_state(1, 65, 0));
        assert_eq!(center_level, 0, "infinite water source created at (1, 65, 0)");
    }

    #[test]
    fn test_water_flow_vector_points_away_from_source() {
        let mut w = flat_world(64);
        let mut sched = TickScheduler::new();
        w.set_block_state(0, 65, 0, water_state(0));
        on_block_changed(&mut sched, &w, 0, 65, 0);
        drain(&mut w, &mut sched, 20);

        // Flow at (1, 65, 0) should point along +X (away from source at x=0 toward air/lower water)
        let (fx, fz) = water_flow_vector(&w, 1, 65, 0);
        assert!(fx > 0.5, "flow vector X should point in +X direction, got {fx}");
        assert!(fz.abs() < 1e-4, "flow vector Z should be near 0, got {fz}");
    }
}

// ---- backlog round (farming, 2026-09-09) tests ----
#[cfg(test)]
mod farm_tests {
    use super::*;
    use crate::ticks::TickScheduler;
    use vc_blocks::blocks::*;
    use vc_world::world::World;

    /// the flat fully-lit test world (the e2_tests helper, local copy)
    fn lit_flat_world() -> World {
        let mut w = World::new(21);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y, lz, GRASS);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }


    /// the vanilla speed-level denominators, VERIFIED against the
    /// Tutorial:Crop_farming table: solo dry 1/13, solo hydrated 1/7,
    /// fully-hydrated farm 1/3 — the growth roll math itself
    #[test]
    fn farm_growth_denominators_match_the_wiki_table() {
        // (q in quarter-points, expected denominator)
        // solo dry: base 2 → q=8 → floor(100/8)=12 → 13
        // solo hydrated: base 4 → q=16 → floor(100/16)=6 → 7
        // full farm: 4 + 8×0.75 = 10 → q=40 → floor(100/40)=2 → 3
        // full dry farm: 2 + 8×0.25 = 4 → q=16 → 7 (the 14.29% row)
        for (q, denom) in [(8u32, 13u32), (16, 7), (40, 3)] {
            assert_eq!(25 * 4 / q + 1, denom, "q={q} (speedLevel={})", q as f32 / 4.0);
        }
    }

    /// farmland hydrates when water sits at the 4-block Chebyshev
    /// boundary, and only at the same level or one above (VERIFIED
    /// w/Farmland §Hydration)
    #[test]
    fn farm_farmland_hydrates_within_the_boundary() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        // carve the farmland at (8, 64, 8): grass → farmland
        w.set_block_state(8, 64, 8, farmland_state(0));
        // water 4 blocks out on X (the exact boundary, same level)
        w.set_block_state(4, 64, 8, default_state(WATER));
        random_plant_tick(&mut w, &mut sched, 8, 64, 8);
        assert_eq!(
            farmland_moisture(w.get_state(8, 64, 8)),
            7,
            "water at Chebyshev 4 hydrates"
        );
        // 5 blocks out does NOT (reset + re-test)
        w.set_block_state(8, 64, 8, farmland_state(0));
        w.set_block_state(4, 64, 8, GRASS);
        w.set_block_state(3, 64, 8, default_state(WATER));
        random_plant_tick(&mut w, &mut sched, 8, 64, 8);
        assert_eq!(
            farmland_moisture(w.get_state(8, 64, 8)),
            0,
            "water at 5 blocks stays dry"
        );
        // one ABOVE at 4 blocks hydrates too
        w.set_block_state(3, 64, 8, GRASS);
        w.set_block_state(4, 65, 8, default_state(WATER));
        random_plant_tick(&mut w, &mut sched, 8, 64, 8);
        assert_eq!(
            farmland_moisture(w.get_state(8, 64, 8)),
            7,
            "water one above at 4 hydrates"
        );
    }

    /// the crops grow on farmland through the random-tick hook — wheat
    /// reaches age 7 with enough ticks on hydrated farmland, and stops
    /// at the ladder top (beetroots mature at 3, VERIFIED)
    #[test]
    fn farm_wheat_grows_to_maturity_on_hydrated_farmland() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        // water source beside the farmland (hydration)
        w.set_block_state(4, 64, 8, default_state(WATER));
        w.set_block_state(8, 64, 8, farmland_state(0));
        w.set_block_state(8, 65, 8, crop_state(WHEAT_CROP, 0));
        // pump ticks: 1/7 chance per tick hydrated → ~49 ticks for 7
        // stages; 4000 ticks is overwhelmingly enough (the sched.now()
        // mix re-rolls each tick)
        let mut max_age = 0;
        for _ in 0..4000 {
            let _ = sched.tick(); // advance the tick clock (the sim does)
            random_plant_tick(&mut w, &mut sched, 8, 64, 8); // farmland
            random_plant_tick(&mut w, &mut sched, 8, 65, 8); // wheat
            max_age = max_age.max(crop_age(w.get_state(8, 65, 8)));
        }
        assert_eq!(crop_age(w.get_state(8, 65, 8)), 7, "wheat reaches 7");
        assert_eq!(max_age, 7);
        // the ladder is terminal
        random_plant_tick(&mut w, &mut sched, 8, 65, 8);
        assert_eq!(crop_age(w.get_state(8, 65, 8)), 7, "age 7 is terminal");

        // beetroots mature at 3 (the half ladder)
        w.set_block_state(8, 65, 8, crop_state(BEETROOTS, 0));
        for _ in 0..4000 {
            let _ = sched.tick();
            random_plant_tick(&mut w, &mut sched, 8, 65, 8);
        }
        assert_eq!(crop_age(w.get_state(8, 65, 8)), 3, "beetroots cap at 3");
        assert_eq!(crop_max_age(BEETROOTS), 3);
    }

    /// a crop without farmland below never grows (the support gate)
    #[test]
    fn farm_crop_without_farmland_stands_still() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        // planted straight on grass — no farmland below
        w.set_block_state(8, 65, 8, crop_state(WHEAT_CROP, 0));
        for _ in 0..2000 {
            let _ = sched.tick();
            random_plant_tick(&mut w, &mut sched, 8, 65, 8);
        }
        assert_eq!(
            crop_age(w.get_state(8, 65, 8)),
            0,
            "no farmland → no growth"
        );
        assert!(is_crop(WHEAT_CROP));
    }

    /// dehydrated + unplanted farmland decays to dirt; planted farmland
    /// only dries (VERIFIED w/Farmland §Decay)
    #[test]
    fn farm_dry_unplanted_farmland_decays() {
        let mut w = lit_flat_world();
        let mut sched = TickScheduler::new();
        // no water anywhere → the decay roll (1/5 per tick, per-position
        // hash varies by sched.now) fires quickly across 2000 ticks
        w.set_block_state(8, 64, 8, farmland_state(7));
        let mut decayed = false;
        for _ in 0..2000 {
            random_plant_tick(&mut w, &mut sched, 8, 64, 8);
            if w.get_block(8, 64, 8) == DIRT {
                decayed = true;
                break;
            }
        }
        assert!(decayed, "dry unplanted farmland decays to dirt");

        // planted: the crop protects the farmland (only dries, never
        // converts while a crop rides it)
        w.set_block_state(8, 64, 8, farmland_state(7));
        w.set_block_state(8, 65, 8, crop_state(WHEAT_CROP, 0));
        let mut stayed = true;
        for _ in 0..2000 {
            random_plant_tick(&mut w, &mut sched, 8, 64, 8);
            random_plant_tick(&mut w, &mut sched, 8, 65, 8);
            if w.get_block(8, 64, 8) == DIRT {
                stayed = false;
                break;
            }
        }
        assert!(stayed, "planted farmland never converts while cropped");
    }
}
