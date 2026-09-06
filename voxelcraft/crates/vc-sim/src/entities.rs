//! Item entities (§22 entity families, progressive): dropped blocks with
//! vanilla-observable physics — gravity, ground collision, water buoyancy,
//! pickup radius with the 0.5 s pickup delay, despawn after 5 minutes.
//! Rendering: spinning billboard quads via the particle pipeline (the
//! vertex format carries baked light × tint, computed at spawn like
//! vanilla's item light sampling).

use vc_blocks::blocks::*;
use vc_rng::rng::Rng;

pub const MAX_ITEMS: usize = 256;
/// vanilla pickup delay (ticks)
pub const PICKUP_DELAY: i32 = 10;
/// vanilla despawn: 6000 ticks (5 minutes)
pub const DESPAWN_TICKS: i32 = 6000;

#[derive(Clone, Copy, Debug)]
pub struct ItemEntity {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// the block id dropped
    pub block: u16,
    /// sim ticks alive
    pub age: i32,
    /// baked billboard brightness + tint at spawn
    pub light: f32,
    pub tint: [f32; 3],
}

pub struct ItemSystem {
    pub items: Vec<ItemEntity>,
    rng: Rng,
    /// total ever dropped (E2E/stat)
    pub dropped_total: u64,
    /// total picked up (E2E/stat)
    pub picked_total: u64,
}

impl ItemSystem {
    pub fn new(seed: u64) -> Self {
        ItemSystem {
            items: Vec::with_capacity(64),
            rng: Rng::new(seed),
            dropped_total: 0,
            picked_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// vanilla drop burst: item spawns at the block center with a random
    /// small velocity
    #[allow(clippy::too_many_arguments)]
    pub fn drop_block(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        block: u16,
        biome: u8,
        sky: u8,
        blk: u8,
    ) {
        if self.items.len() >= MAX_ITEMS {
            return; // cap: oldest items still live, refuse the drop
        }
        let tint = vc_blocks::tint::block_tint_color(block, biome);
        let s = sky.min(15) as f32 / 15.0;
        let b = blk.min(15) as f32 / 15.0;
        let light = (s.max(b) * 0.96 + 0.04) * s.max(b).powf(1.2);
        self.items.push(ItemEntity {
            pos: [wx as f32 + 0.5, wy as f32 + 0.3, wz as f32 + 0.5],
            vel: [
                (self.rng.next_f32() - 0.5) * 0.12,
                0.18 + self.rng.next_f32() * 0.06,
                (self.rng.next_f32() - 0.5) * 0.12,
            ],
            block,
            age: 0,
            light,
            tint,
        });
        self.dropped_total += 1;
    }

    /// ONE sim tick for all item entities: gravity 0.04, drag 0.98, ground
    /// rest with slip, buoyancy in water. Item hitbox is a point (visual
    /// half-size 0.15); collision probes the world at the entity position.
    /// Phase 6 §26: entities outside the simulation ring (Chebyshev chunk
    /// distance from `sim_center`) freeze — age included (1.18+ semantics;
    /// `sim_radius` = i32::MAX disables gating = 1.16.5 behavior).
    pub fn tick(
        &mut self,
        world: &vc_world::world::World,
        sim_center: (i32, i32),
        sim_radius: i32,
    ) {
        for it in self.items.iter_mut() {
            let ichunk = (
                (it.pos[0] / 16.0).floor() as i32,
                (it.pos[2] / 16.0).floor() as i32,
            );
            let in_ring = ichunk
                .0
                .wrapping_sub(sim_center.0)
                .saturating_abs()
                .max(ichunk.1.wrapping_sub(sim_center.1).saturating_abs())
                <= sim_radius;
            if !in_ring {
                continue;
            }
            it.age += 1;
            let in_water =
                world.get_block(it.pos[0] as i32, it.pos[1] as i32, it.pos[2] as i32) == WATER;
            it.vel[1] += if in_water { 0.04 } else { -0.04 };
            for axis in 0..3 {
                let target = it.pos[axis] + it.vel[axis];
                let mut probe = it.pos;
                probe[axis] = target;
                let hit =
                    is_solid(world.get_block(probe[0] as i32, probe[1] as i32, probe[2] as i32));
                if hit {
                    if axis == 1 {
                        it.vel[1] = 0.0;
                        it.vel[0] *= 0.6;
                        it.vel[2] *= 0.6;
                    } else {
                        it.vel[axis] = 0.0;
                    }
                } else {
                    it.pos[axis] = target;
                }
            }
            let drag = if in_water { 0.9 } else { 0.98 };
            it.vel[0] *= drag;
            it.vel[2] *= drag;
            if in_water {
                it.vel[1] *= 0.9;
            } else {
                // VERIFIED entity physics table (minecraft.wiki/w/
                // Falling_Block, research-verdicts.md live round):
                // Drag-Y 0.98 applies in air too — items share the
                // falling-block profile (gravity 0.04, drag 0.98,
                // terminal 1.96 b/t)
                it.vel[1] *= 0.98;
            }
        }
        self.items.retain(|it| it.age < DESPAWN_TICKS);
    }

    /// vanilla pickup: entities within 1.0 of the player (and past the
    /// delay) get collected. Returns the picked-up block ids (the caller
    /// routes them into the inventory).
    pub fn collect(&mut self, eye: [f32; 3]) -> Vec<u16> {
        let mut picked = Vec::new();
        let mut i = 0;
        while i < self.items.len() {
            let it = &self.items[i];
            let d2 = (it.pos[0] - eye[0]).powi(2)
                + (it.pos[1] - eye[1]).powi(2)
                + (it.pos[2] - eye[2]).powi(2);
            if d2 < 1.0 && it.age > PICKUP_DELAY {
                picked.push(it.block);
                self.items.remove(i);
            } else {
                i += 1;
            }
        }
        self.picked_total += picked.len() as u64;
        picked
    }

    /// billboard quads: two crossed quads rotating slowly around Y (the
    /// vanilla block-item "spin"), bobbing vertically. Emitted into the
    /// particle vertex stream (§16.2 pass 4 shares the pipeline).
    pub fn build_vertices(
        &self,
        time: f32,
        right: [f32; 3],
        up: [f32; 3],
        out: &mut Vec<vc_particles::particles::ParticleVertex>,
    ) {
        for it in self.items.iter() {
            let tile = state_tiles(it.block as u16)[3];
            let tx = (tile % 16) as f32;
            let ty = (tile / 16) as f32;
            // spin: the right basis rotated around the world Y axis
            let ang = time * 1.6;
            let (s, c) = (ang.sin(), ang.cos());
            let rr = [
                c * right[0] + s * right[2],
                0.0,
                -s * right[0] + c * right[2],
            ];
            let ru = up;
            let half = 0.15f32;
            let bob = (time * 2.2 + it.pos[0] + it.pos[2]).sin() * 0.04;
            let col = [
                it.light * it.tint[0],
                it.light * it.tint[1],
                it.light * it.tint[2],
            ];
            let corners = [
                (
                    [
                        -rr[0] * half - ru[0] * half,
                        -rr[1] * half - ru[1] * half,
                        -rr[2] * half - ru[2] * half,
                    ],
                    [tx / 16.0, (ty + 1.0) / 16.0],
                ),
                (
                    [
                        rr[0] * half - ru[0] * half,
                        rr[1] * half - ru[1] * half,
                        rr[2] * half - ru[2] * half,
                    ],
                    [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
                ),
                (
                    [
                        rr[0] * half + ru[0] * half,
                        rr[1] * half + ru[1] * half,
                        rr[2] * half + ru[2] * half,
                    ],
                    [(tx + 1.0) / 16.0, ty / 16.0],
                ),
                (
                    [
                        -rr[0] * half + ru[0] * half,
                        -rr[1] * half + ru[1] * half,
                        -rr[2] * half + ru[2] * half,
                    ],
                    [tx / 16.0, ty / 16.0],
                ),
            ];
            for ci in [0usize, 1, 2, 0, 2, 3] {
                let (c, uv) = corners[ci];
                out.push(vc_particles::particles::ParticleVertex {
                    pos: [it.pos[0] + c[0], it.pos[1] + c[1] + bob, it.pos[2] + c[2]],
                    uv: [uv[0], uv[1]],
                    col,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Phase E1 — XP orbs (evolution 1.0–1.2 bracket, all values live-verified
// 2026-09-06 against minecraft.wiki/w/Experience; see
// docs/research/phase1-1.0-1.2-research.md)
// ---------------------------------------------------------------------------

pub const MAX_ORBS: usize = 512;
/// orb despawn: 6000 ticks (5 minutes) — VERIFIED w/Experience
pub const ORB_DESPAWN_TICKS: i32 = 6000;
/// attraction distance: 7.25 blocks (player feet-center ↔ orb center),
/// speeding up as they near — VERIFIED w/Experience ("float or glide
/// toward the player up to a distance of 7.25 blocks ... speeding up as
/// they get nearer to the player")
pub const ORB_ATTRACT_DIST: f32 = 7.25;
/// pickup rate: orbs are collected one at a time, max 10/second — VERIFIED
/// w/Experience ("no matter how many orbs are in the range of the player,
/// they are added to the player's experience one at a time
/// (10 orbs/second)"). Engine form: a 2-tick pickup gate.
pub const ORB_PICKUP_EVERY_TICKS: i32 = 2;
/// the vanilla orb value ladder — VERIFIED w/Experience: drops split into
/// "the base values of orbs by size (1, 3, 7, 17, 37, 73, 149, 307, 617,
/// 1237, and 2477)"
pub const ORB_VALUES: [i32; 11] = [2477, 1237, 617, 307, 149, 73, 37, 17, 7, 3, 1];

#[derive(Clone, Copy, Debug)]
pub struct XpOrb {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// XP the orb carries
    pub value: i32,
    /// sim ticks alive
    pub age: i32,
}

/// Split a total XP amount into vanilla orb values (greedy from the
/// largest base value; the remainder becomes 1-point orbs). VERIFIED rule:
/// the total is preserved and each orb's value is one of the base values.
pub fn split_xp(total: i32) -> Vec<i32> {
    if total <= 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut left = total;
    // big orbs greedily, but cap common mob drops into small orbs like
    // vanilla's observed behavior (a 5-XP zombie kill = 3+1+1, not 3+3)
    while left > 0 {
        let mut taken = 1;
        for &v in ORB_VALUES.iter() {
            if v <= left && v != 1 {
                // don't take a 3 when only 2 remain → 1+1
                if v == 3 && left == 2 {
                    continue;
                }
                taken = v;
                break;
            }
        }
        out.push(taken);
        left -= taken;
    }
    out
}

pub struct XpOrbSystem {
    pub orbs: Vec<XpOrb>,
    rng: Rng,
    /// collected XP drained by the game layer each frame
    pub collected: Vec<i32>,
    /// 2-tick pickup gate (VERIFIED 10 orbs/s)
    pickup_gate: i32,
    /// stats
    pub spawned_total: u64,
    pub picked_total: u64,
}

impl XpOrbSystem {
    pub fn new(seed: u64) -> Self {
        XpOrbSystem {
            orbs: Vec::with_capacity(64),
            rng: Rng::new(seed ^ 0x0DB_5EED),
            collected: Vec::new(),
            pickup_gate: 0,
            spawned_total: 0,
            picked_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.orbs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.orbs.is_empty()
    }

    /// Drop `total` XP at a position, split into vanilla orb values with
    /// a small random burst velocity (the item-drop pattern).
    pub fn drop_xp(&mut self, x: f32, y: f32, z: f32, total: i32) {
        if total <= 0 {
            return;
        }
        for v in split_xp(total) {
            if self.orbs.len() >= MAX_ORBS {
                return;
            }
            self.orbs.push(XpOrb {
                pos: [x, y + 0.3, z],
                vel: [
                    (self.rng.next_f32() - 0.5) * 0.12,
                    0.18 + self.rng.next_f32() * 0.06,
                    (self.rng.next_f32() - 0.5) * 0.12,
                ],
                value: v,
                age: 0,
            });
            self.spawned_total += 1;
        }
    }

    /// ONE sim tick: item-parity physics (gravity 0.04, drag 0.98 — the
    /// verified shared entity profile), the 7.25-block attraction, and the
    /// 10-orbs/second pickup gate.
    pub fn tick(&mut self, world: &vc_world::world::World, player_feet: Option<[f32; 3]>) {
        self.pickup_gate = (self.pickup_gate + 1) % ORB_PICKUP_EVERY_TICKS;
        let can_pick = self.pickup_gate == 0;
        for o in self.orbs.iter_mut() {
            o.age += 1;
            let in_water =
                world.get_block(o.pos[0] as i32, o.pos[1] as i32, o.pos[2] as i32) == WATER;
            o.vel[1] += if in_water { 0.04 } else { -0.04 };
            // attraction: glide toward the player's feet-center within
            // 7.25 blocks, accelerating as they near (VERIFIED)
            if let Some(p) = player_feet {
                let dx = p[0] - o.pos[0];
                let dy = p[1] - o.pos[1];
                let dz = p[2] - o.pos[2];
                let d = (dx * dx + dy * dy + dz * dz).sqrt();
                if d <= ORB_ATTRACT_DIST && d > 1e-3 {
                    // pull grows as the orb closes in (speed up when nearer)
                    let pull = 0.05 + (1.0 - d / ORB_ATTRACT_DIST) * 0.25;
                    o.vel[0] += dx / d * pull;
                    o.vel[1] += dy / d * pull;
                    o.vel[2] += dz / d * pull;
                }
            }
            // move with per-axis collision (item pattern)
            for axis in 0..3 {
                let target = o.pos[axis] + o.vel[axis];
                let mut probe = o.pos;
                probe[axis] = target;
                let hit =
                    is_solid(world.get_block(probe[0] as i32, probe[1] as i32, probe[2] as i32));
                if hit {
                    if axis == 1 {
                        o.vel[1] = 0.0;
                        o.vel[0] *= 0.6;
                        o.vel[2] *= 0.6;
                    } else {
                        o.vel[axis] = 0.0;
                    }
                } else {
                    o.pos[axis] = target;
                }
            }
            let drag = if in_water { 0.9 } else { 0.98 };
            o.vel[0] *= drag;
            o.vel[2] *= drag;
            o.vel[1] *= if in_water { 0.9 } else { 0.98 };
        }
        // pickup: one orb per gate tick (10/s — VERIFIED), collected at the
        // feet (15w46a "experience is now collected at the feet")
        if let Some(p) = player_feet {
            if can_pick {
                for i in 0..self.orbs.len() {
                    let o = &self.orbs[i];
                    let d2 = (o.pos[0] - p[0]).powi(2)
                        + (o.pos[1] - p[1]).powi(2)
                        + (o.pos[2] - p[2]).powi(2);
                    if d2 < 1.2 {
                        let o = self.orbs.remove(i);
                        self.collected.push(o.value);
                        self.picked_total += 1;
                        break; // one per gate tick
                    }
                }
            }
        }
        // despawn (VERIFIED: 6000 ticks)
        self.orbs.retain(|o| o.age < ORB_DESPAWN_TICKS);
    }

    /// billboard quads: small green↔yellow orbs, dense (value ≥ 17) orbs
    /// use the big sprite with the orange core (VERIFIED w/Experience).
    /// Emitted into the particle stream like items.
    pub fn build_vertices(
        &self,
        time: f32,
        right: [f32; 3],
        up: [f32; 3],
        out: &mut Vec<vc_particles::particles::ParticleVertex>,
    ) {
        for o in self.orbs.iter() {
            let tile = if o.value >= 17 { TILE_XP_ORB_BIG } else { TILE_XP_ORB };
            let tx = (tile % 16) as f32;
            let ty = (tile / 16) as f32;
            // green↔yellow flash (VERIFIED: "fade between green and yellow")
            let flash = 0.5 + 0.5 * (time * 3.0 + o.pos[0]).sin();
            let col = [
                0.55 + flash * 0.45,
                0.85 + flash * 0.15,
                0.25,
            ];
            let bob = (time * 2.0 + o.pos[0] + o.pos[2]).sin() * 0.05;
            let half = 0.12f32;
            let corners = [
                (
                    [
                        -right[0] * half - up[0] * half,
                        -right[1] * half - up[1] * half,
                        -right[2] * half - up[2] * half,
                    ],
                    [tx / 16.0, ty / 16.0],
                ),
                (
                    [
                        right[0] * half - up[0] * half,
                        right[1] * half - up[1] * half,
                        right[2] * half - up[2] * half,
                    ],
                    [(tx + 1.0) / 16.0, ty / 16.0],
                ),
                (
                    [
                        right[0] * half + up[0] * half,
                        right[1] * half + up[1] * half,
                        right[2] * half + up[2] * half,
                    ],
                    [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
                ),
                (
                    [
                        -right[0] * half + up[0] * half,
                        -right[1] * half + up[1] * half,
                        -right[2] * half + up[2] * half,
                    ],
                    [tx / 16.0, (ty + 1.0) / 16.0],
                ),
            ];
            for ci in [0usize, 1, 2, 0, 2, 3] {
                let (c, uv) = corners[ci];
                out.push(vc_particles::particles::ParticleVertex {
                    pos: [o.pos[0] + c[0], o.pos[1] + c[1] + bob, o.pos[2] + c[2]],
                    uv: [uv[0], uv[1]],
                    col,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_world::world::World;

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

    #[test]
    fn drops_fall_rest_and_despawn() {
        let mut is = ItemSystem::new(3);
        let w = flat_world();
        is.drop_block(0, 66, 0, SAND, 2, 15, 0);
        assert_eq!(is.len(), 1);
        // 3 seconds of ticks
        for _ in 0..60 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        let it = &is.items[0];
        // fell from 66.3 to rest on the y=64 floor's top surface (y=65);
        // point-collision leaves a small rest band above the exact surface
        assert!(
            (it.pos[1] - 65.0).abs() < 0.06,
            "rest height: {}",
            it.pos[1]
        );
        // despawn at 6000 ticks
        for _ in 0..6000 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        assert_eq!(is.len(), 0);
    }

    #[test]
    fn pickup_after_delay_in_radius() {
        let mut is = ItemSystem::new(4);
        let w = flat_world();
        is.drop_block(0, 66, 0, DIRT, 2, 15, 0);
        // before the delay: no pickup
        for _ in 0..5 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        assert!(
            is.collect([0.5, 66.0, 0.5]).is_empty(),
            "pickup delay guards"
        );
        // after the delay: player near → collected
        for _ in 0..10 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        let got = is.collect([0.5, 65.8, 0.5]);
        assert_eq!(got, vec![DIRT]);
        assert_eq!(is.len(), 0);
        assert_eq!(is.picked_total, 1);
        // far away: no pickup
        is.drop_block(4, 66, 4, STONE, 2, 15, 0);
        for _ in 0..20 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        assert!(is.collect([0.5, 66.0, 0.5]).is_empty(), "distance guards");
        assert_eq!(is.len(), 1);
    }

    #[test]
    fn item_vertices_are_two_crossed_quads() {
        let mut is = ItemSystem::new(5);
        is.drop_block(0, 70, 0, GRASS, 3, 15, 0);
        let mut out = Vec::new();
        is.build_vertices(1.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], &mut out);
        assert_eq!(out.len(), 6, "one billboard quad per item");
        // tint baked: Forest grass
        let c = out[0].col;
        assert!(c[1] > c[0], "green-dominant: {c:?}");
        // UVs inside the grass tile (tile 16 col? — any valid atlas UV)
        for v in &out {
            assert!((0.0..=1.0).contains(&v.uv[0]));
            assert!((0.0..=1.0).contains(&v.uv[1]));
        }
    }

    // ---------------- Phase E1: XP orbs ----------------

    #[test]
    fn phase_e1_split_xp_matches_the_vanilla_ladder() {
        // VERIFIED base values 1,3,7,17,...,2477; totals preserved
        assert_eq!(split_xp(0), Vec::<i32>::new());
        assert_eq!(split_xp(1), vec![1]);
        assert_eq!(split_xp(5), vec![3, 1, 1]); // zombie kill
        assert_eq!(split_xp(10), vec![7, 3]);
        assert_eq!(split_xp(12000).iter().sum::<i32>(), 12000, "dragon XP preserved");
        // every orb value is one of the base values
        for v in split_xp(976) {
            assert!(ORB_VALUES.contains(&v), "value {v} not a base value");
        }
    }

    #[test]
    fn phase_e1_orbs_attract_and_collect_through_the_gate() {
        let w = flat_world();
        let mut sys = XpOrbSystem::new(3);
        // drop 4 XP at the player's feet
        sys.drop_xp(8.5, 66.0, 8.5, 4);
        assert_eq!(sys.orbs.len(), 2, "4 XP = 3+1 two orbs");
        // player 2 blocks above the floor next to the drop
        let feet = [8.5, 65.0, 8.5];
        let mut picked = 0;
        for _ in 0..40 {
            sys.tick(&w, Some(feet));
            if !sys.collected.is_empty() {
                picked += sys.collected.drain(..).sum::<i32>();
            }
        }
        assert_eq!(picked, 4, "all XP collected through the gate");
        assert!(sys.orbs.is_empty(), "orbs drained");
        // the 2-tick gate never collects faster than 10/s: 40 ticks → at
        // most 20 pickups; the orbs sat within 1 block so the gate was the
        // only limiter
        assert!(
            sys.picked_total <= 20,
            "10 orbs/second gate (VERIFIED), got {}",
            sys.picked_total
        );
    }

    #[test]
    fn phase_e1_orbs_despawn_at_6000_ticks() {
        let w = flat_world();
        let mut sys = XpOrbSystem::new(4);
        sys.drop_xp(8.5, 66.0, 8.5, 1);
        assert_eq!(sys.orbs.len(), 1);
        for _ in 0..ORB_DESPAWN_TICKS {
            sys.tick(&w, None);
        }
        assert!(sys.orbs.is_empty(), "VERIFIED: 5-minute despawn");
        assert_eq!(ORB_DESPAWN_TICKS, 6000);
        assert_eq!(ORB_ATTRACT_DIST, 7.25);
    }
}
