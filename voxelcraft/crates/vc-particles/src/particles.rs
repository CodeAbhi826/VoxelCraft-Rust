//! Block particles — vanilla 1.16.5-style break/hit particles (Phase 5,
//! §16.2 pass 4). Fixed-capacity pool, vanilla tick physics (20 Hz fixed
//! steps: gravity 0.04/t, air friction 0.98, ground slip 0.7), billboard
//! quads built CPU-side against the camera basis.
//!
//! Rendering (render.rs) draws the billboard vertex buffer after the
//! translucent water pass — the §16.2 order (particles between water and
//! clouds). Light and tint are baked per particle at SPAWN time
//! (particles live ~1 s; day/night drift over a particle lifetime is
//! imperceptible — vanilla bakes per frame, we document the difference).

use crate::blocks::*;
use crate::rng::Rng;
use crate::tint;

/// pool cap — 64/break * 64 simultaneous breaks worst case; E2E asserts
/// the cap is respected
pub const MAX_PARTICLES: usize = 4096;

/// vanilla: 4×4×4 = 64 particles per block break
pub const BREAK_GRID: i32 = 4;

/// fixed sim rate (vanilla 20 Hz)
pub const TICK_HZ: f32 = 20.0;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// remaining sim ticks
    pub life: i32,
    /// half-extent of the billboard quad (world units)
    pub half: f32,
    /// absolute atlas UV base (top-left of the sub-rect)
    pub u0: f32,
    pub v0: f32,
    /// sub-rect size in atlas UV (a 4×4-px quarter of a 16-px tile)
    pub du: f32,
    pub dv: f32,
    /// baked brightness (0..1-ish, terrain light curve) — multiplies the
    /// texture in the shader
    pub light: f32,
    /// baked tint color (biome grass/foliage, else white)
    pub tint: [f32; 3],
}

/// billboard vertex — 8 floats: pos.xyz, atlas uv, rgb (light * tint)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub col: [f32; 3],
}

pub struct ParticleSystem {
    pub parts: Vec<Particle>,
    /// fixed-step accumulator (seconds)
    acc: f32,
    rng: Rng,
    /// total spawned (E2E/stat evidence)
    pub spawned_total: u64,
}

impl ParticleSystem {
    pub fn new(seed: u64) -> Self {
        ParticleSystem {
            parts: Vec::with_capacity(256),
            acc: 0.0,
            rng: Rng::new(seed),
            spawned_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// vanilla break burst: a 4×4×4 grid of sub-tile particles across the
    /// block volume, launched with randomized velocities. `sky`/`blk` are
    /// the light levels at the block (baked brightness), `biome` its column.
    pub fn spawn_block_break(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        block: u8,
        biome: u8,
        sky: u8,
        blk: u8,
    ) {
        let state = block as u16;
        let tile = state_tiles(state)[3]; // side tile like vanilla (top for grass tops)
        let tint_col = tint::block_tint_color(block, biome);
        let light = particle_light(sky, blk);
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        for gy in 0..BREAK_GRID {
            for gz in 0..BREAK_GRID {
                for gx in 0..BREAK_GRID {
                    // random 4×4 px quarter of the tile
                    let qx = self.rng.next_range(4) as f32;
                    let qy = self.rng.next_range(4) as f32;
                    let p = Particle {
                        pos: [
                            wx as f32 + (gx as f32 + 0.125) / BREAK_GRID as f32 * 0.75 + 0.125,
                            wy as f32 + (gy as f32 + 0.125) / BREAK_GRID as f32 * 0.75 + 0.125,
                            wz as f32 + (gz as f32 + 0.125) / BREAK_GRID as f32 * 0.75 + 0.125,
                        ],
                        // vanilla-ish launch: horizontal ±0.2, up to +0.24
                        vel: [
                            (self.rng.next_f32() - 0.5) * 0.4,
                            self.rng.next_f32() * 0.28 + 0.06,
                            (self.rng.next_f32() - 0.5) * 0.4,
                        ],
                        life: 8 + self.rng.next_range(18) as i32,
                        half: 0.06,
                        // atlas: 16×16-tile grid; quarter of one tile
                        u0: (tx + qx * 0.25) / 16.0,
                        v0: (ty + qy * 0.25) / 16.0,
                        du: 0.25 / 16.0,
                        dv: 0.25 / 16.0,
                        light,
                        tint: tint_col,
                    };
                    self.push(p);
                }
            }
        }
    }

    /// single hit particle while mining (vanilla spawns per hit on the face)
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_hit(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        block: u8,
        biome: u8,
        sky: u8,
        blk: u8,
    ) {
        let tile = state_tiles(block as u16)[3];
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        let qx = self.rng.next_range(4) as f32;
        let qy = self.rng.next_range(4) as f32;
        let p = Particle {
            pos: [
                wx as f32 + 0.5 + (self.rng.next_f32() - 0.5) * 0.5,
                wy as f32 + 0.5 + (self.rng.next_f32() - 0.5) * 0.5,
                wz as f32 + 0.5 + (self.rng.next_f32() - 0.5) * 0.5,
            ],
            vel: [
                (self.rng.next_f32() - 0.5) * 0.16,
                0.05 + self.rng.next_f32() * 0.1,
                (self.rng.next_f32() - 0.5) * 0.16,
            ],
            life: 8 + self.rng.next_range(10) as i32,
            half: 0.05,
            u0: (tx + qx * 0.25) / 16.0,
            v0: (ty + qy * 0.25) / 16.0,
            du: 0.25 / 16.0,
            dv: 0.25 / 16.0,
            light: particle_light(sky, blk),
            tint: tint::block_tint_color(block, biome),
        };
        self.push(p);
    }

    fn push(&mut self, p: Particle) {
        if self.parts.len() < MAX_PARTICLES {
            self.parts.push(p);
            self.spawned_total += 1;
        }
    }

    /// advance the fixed 20 Hz sim by `dt` seconds of wall time. Collisions
    /// against the world's solid blocks: per-axis sweep, vanilla's "land →
    /// vy = 0, horizontal slip 0.7", buoyancy in water.
    pub fn update(&mut self, dt: f32, world: &crate::world::World) {
        self.acc += dt.min(0.25); // clamp so a hitch can't teleport particles
        let step = 1.0 / TICK_HZ;
        while self.acc >= step {
            self.acc -= step;
            self.tick(world);
        }
    }

    fn tick(&mut self, world: &crate::world::World) {
        for p in self.parts.iter_mut() {
            p.life -= 1;
            if p.life <= 0 {
                continue;
            }
            let in_water = world.get_block(
                p.pos[0] as i32,
                p.pos[1] as i32,
                p.pos[2] as i32,
            ) == WATER;

            // gravity / buoyancy
            p.vel[1] += if in_water { 0.02 } else { -0.04 };

            // per-axis move with world collision
            for axis in 0..3 {
                let target = p.pos[axis] + p.vel[axis];
                let mut probe = p.pos;
                probe[axis] = target;
                let hit = is_solid(
                    world.get_block(probe[0] as i32, probe[1] as i32, probe[2] as i32),
                );
                if hit {
                    if axis == 1 {
                        p.vel[1] = 0.0;
                        // landed: horizontal slip (vanilla 0.7)
                        p.vel[0] *= 0.7;
                        p.vel[2] *= 0.7;
                    } else {
                        p.vel[axis] = 0.0;
                    }
                } else {
                    p.pos[axis] = target;
                }
            }

            // drag
            let drag = if in_water { 0.85 } else { 0.98 };
            p.vel[0] *= drag;
            p.vel[2] *= drag;
            if in_water {
                p.vel[1] *= 0.85;
            }
        }
        self.parts.retain(|p| p.life > 0);
    }

    /// billboard quads for the current camera basis (right/up in world
    /// space); 6 vertices per particle, triangle-list (no index buffer —
    /// the buffer is rebuilt every frame anyway).
    pub fn build_vertices(&self, right: [f32; 3], up: [f32; 3], out: &mut Vec<ParticleVertex>) {
        out.clear();
        out.reserve(self.parts.len() * 6);
        for p in self.parts.iter() {
            if p.life <= 0 {
                continue;
            }
            let r = [right[0] * p.half, right[1] * p.half, right[2] * p.half];
            let u = [up[0] * p.half, up[1] * p.half, up[2] * p.half];
            let col = [p.light * p.tint[0], p.light * p.tint[1], p.light * p.tint[2]];
            // corners: (-r-u) (r-u) (r+u) (-r+u) with matching sub-tile UVs
            let corners = [
                ([-r[0] - u[0], -r[1] - u[1], -r[2] - u[2]], [p.u0, p.v0 + p.dv]),
                ([r[0] - u[0], r[1] - u[1], r[2] - u[2]], [p.u0 + p.du, p.v0 + p.dv]),
                ([r[0] + u[0], r[1] + u[1], r[2] + u[2]], [p.u0 + p.du, p.v0]),
                ([-r[0] + u[0], -r[1] + u[1], -r[2] + u[2]], [p.u0, p.v0]),
            ];
            for ci in [0usize, 1, 2, 0, 2, 3] {
                let (c, uv) = corners[ci];
                out.push(ParticleVertex {
                    pos: [p.pos[0] + c[0], p.pos[1] + c[1], p.pos[2] + c[2]],
                    uv: [uv[0], uv[1]],
                    col,
                });
            }
        }
    }
}

/// terrain-light-curve brightness for a particle (sky vs block light, no
/// per-frame day factor — baked at spawn)
fn particle_light(sky: u8, blk: u8) -> f32 {
    let s = sky.min(15) as f32 / 15.0;
    let b = blk.min(15) as f32 / 15.0;
    // approximate vanilla brightness curve: light^1.6-ish falloff with a
    // small floor so cave particles stay visible
    let l = s.max(b);
    (l * 0.96 + 0.04) * l.powf(1.2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    fn flat_world() -> World {
        // hand-built: solid floor at y<=64, open sky above (deterministic,
        // no terrain randomness — same pattern as world::tests::flat_world)
        let mut w = World::new(7);
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
                w.insert_generated((dx, dz), std::sync::Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w
    }

    #[test]
    fn break_burst_spawns_64_within_cap() {
        let mut ps = ParticleSystem::new(7);
        ps.spawn_block_break(0, 65, 0, GRASS, 3, 15, 0);
        assert_eq!(ps.len(), 64);
        assert_eq!(ps.spawned_total, 64);
        // tint baked (Forest grass = vanilla 0x79C05A)
        let t = ps.parts[0].tint;
        assert!((t[1] - 0xC0 as f32 / 255.0).abs() < 0.02, "green-dominant {t:?}");
        // further bursts capped at MAX_PARTICLES
        for _ in 0..200 {
            ps.spawn_block_break(0, 65, 0, DIRT, 2, 15, 0);
        }
        assert!(ps.len() <= MAX_PARTICLES);
    }

    #[test]
    fn particles_fall_and_die() {
        let mut ps = ParticleSystem::new(9);
        let w = flat_world();
        ps.spawn_block_break(0, 66, 0, SAND, 4, 15, 0);
        let start_y = ps.parts[0].pos[1];
        // step like the game loop (update() clamps dt to 0.25 s)
        let mut t = 0.0f32;
        while t < 1.0 {
            ps.update(1.0 / 60.0, &w);
            t += 1.0 / 60.0;
        }
        assert!(ps.len() <= 64);
        // launch is upward, gravity pulls down: after 1 s nothing is more
        // than a block above the spawn (no runaway)
        for p in &ps.parts {
            assert!(p.pos[1] <= start_y + 1.0, "particle rose: {:?}", p.pos);
        }
        // sand bursts land ON the floor (floor top = y 64, particles stop at
        // the surface, never sink through)
        for p in &ps.parts {
            assert!(p.pos[1] >= 63.9, "particle sank below the floor: {:?}", p.pos);
        }
        // long enough → all dead (max life 26 ticks = 1.3 s)
        let mut t = 0.0f32;
        while t < 2.0 {
            ps.update(1.0 / 60.0, &w);
            t += 1.0 / 60.0;
        }
        assert_eq!(ps.len(), 0, "all particles expire");
    }

    #[test]
    fn vertices_are_quads_with_subtile_uvs() {
        let mut ps = ParticleSystem::new(11);
        ps.spawn_block_break(0, 80, 0, STONE, 2, 15, 0);
        let mut out = Vec::new();
        ps.build_vertices([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], &mut out);
        assert_eq!(out.len(), ps.len() * 6);
        // each quad: same color across its 6 verts, uv within one atlas tile
        for (i, v) in out.iter().enumerate() {
            assert!(v.uv[0] >= 0.0 && v.uv[0] <= 1.0);
            assert!(v.uv[1] >= 0.0 && v.uv[1] <= 1.0);
            if i % 6 == 0 {
                assert_eq!(v.col, out[i + 5].col, "quad color constant");
            }
        }
    }
}
