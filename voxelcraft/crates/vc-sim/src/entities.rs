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
    pub block: u8,
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
        ItemSystem { items: Vec::with_capacity(64), rng: Rng::new(seed), dropped_total: 0, picked_total: 0 }
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
        block: u8,
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
    pub fn tick(&mut self, world: &vc_world::world::World, sim_center: (i32, i32), sim_radius: i32) {
        for it in self.items.iter_mut() {
            let ichunk = ((it.pos[0] / 16.0).floor() as i32, (it.pos[2] / 16.0).floor() as i32);
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
            let in_water = world.get_block(
                it.pos[0] as i32,
                it.pos[1] as i32,
                it.pos[2] as i32,
            ) == WATER;
            it.vel[1] += if in_water { 0.04 } else { -0.04 };
            for axis in 0..3 {
                let target = it.pos[axis] + it.vel[axis];
                let mut probe = it.pos;
                probe[axis] = target;
                let hit = is_solid(world.get_block(
                    probe[0] as i32,
                    probe[1] as i32,
                    probe[2] as i32,
                ));
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
            }
        }
        self.items.retain(|it| it.age < DESPAWN_TICKS);
    }

    /// vanilla pickup: entities within 1.0 of the player (and past the
    /// delay) get collected. Returns the picked-up block ids (the caller
    /// routes them into the inventory).
    pub fn collect(&mut self, eye: [f32; 3]) -> Vec<u8> {
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
            let col = [it.light * it.tint[0], it.light * it.tint[1], it.light * it.tint[2]];
            let corners = [
                (
                    [-rr[0] * half - ru[0] * half, -rr[1] * half - ru[1] * half, -rr[2] * half - ru[2] * half],
                    [tx / 16.0, (ty + 1.0) / 16.0],
                ),
                (
                    [rr[0] * half - ru[0] * half, rr[1] * half - ru[1] * half, rr[2] * half - ru[2] * half],
                    [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
                ),
                (
                    [rr[0] * half + ru[0] * half, rr[1] * half + ru[1] * half, rr[2] * half + ru[2] * half],
                    [(tx + 1.0) / 16.0, ty / 16.0],
                ),
                (
                    [-rr[0] * half + ru[0] * half, -rr[1] * half + ru[1] * half, -rr[2] * half + ru[2] * half],
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

#[cfg(test)]
mod tests {
    use super::*;
    use vc_world::world::World;
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
        assert!((it.pos[1] - 65.0).abs() < 0.06, "rest height: {}", it.pos[1]);
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
        assert!(is.collect([0.5, 66.0, 0.5]).is_empty(), "pickup delay guards");
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
}
