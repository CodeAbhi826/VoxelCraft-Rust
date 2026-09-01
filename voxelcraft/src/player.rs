//! Player: movement (walk/sprint/swim/fly), AABB voxel collision,
//! DDA block raycast, hotbar state, footstep logic.

use crate::blocks::*;
use crate::world::World;
use glam::Vec3;

pub const WALK_SPEED: f32 = 4.317;
pub const SPRINT_SPEED: f32 = 5.612;
pub const FLY_SPEED: f32 = 10.9;
pub const FLY_SPRINT: f32 = 21.8;
pub const GRAVITY: f32 = 32.0;
pub const JUMP_VEL: f32 = 8.95;
pub const TERMINAL: f32 = 78.4;
pub const REACH: f32 = 4.5;

pub const PLAYER_HALF: f32 = 0.3;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const EYE_HEIGHT: f32 = 1.62;

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub fwd: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
    pub break_hold: bool,
    pub place_hold: bool,
    mouse_dx: f32,
    mouse_dy: f32,
}

impl Input {
    pub fn add_mouse(&mut self, dx: f32, dy: f32) {
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }
    fn take_mouse(&mut self) -> (f32, f32) {
        let r = (self.mouse_dx, self.mouse_dy);
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        r
    }
}

pub struct SoundEvent {
    pub family: SoundFamily,
    pub volume: f32,
    pub pitch: f32,
}

pub struct Player {
    pub pos: Vec3, // feet center
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub flying: bool,
    pub in_water: bool,
    pub head_in_water: bool,
    pub hotbar: [u8; 9],
    pub selected: usize,
    pub fov: f32,
    pub fov_cur: f32,
    last_space_tap: f32,
    step_accum: f32,
    was_in_water: bool,
}

impl Player {
    pub fn new(spawn: Vec3) -> Self {
        Player {
            pos: spawn,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            flying: true, // spawn in air, land when chunks ready
            in_water: false,
            head_in_water: false,
            hotbar: PALETTE,
            selected: 0,
            fov: 1.2217, // 70 degrees
            fov_cur: 1.2217,
            last_space_tap: -1.0,
            step_accum: 0.0,
            was_in_water: false,
        }
    }

    pub fn eye(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + EYE_HEIGHT, self.pos.z)
    }

    pub fn look_dir(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn update(
        &mut self,
        dt: f32,
        time: f32,
        world: &World,
        input: &mut Input,
        sensitivity: f32,
        loaded: bool,
    ) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();

        // look
        let (mdx, mdy) = input.take_mouse();
        self.yaw += mdx * 0.0022 * sensitivity;
        self.pitch = (self.pitch - mdy * 0.0022 * sensitivity)
            .clamp(-1.5685, 1.5685);

        if !loaded {
            return sounds;
        }

        // water state
        let feet_block = world.get_block(
            self.pos.x.floor() as i32,
            (self.pos.y + 0.4).floor() as i32,
            self.pos.z.floor() as i32,
        );
        let head_block = world.get_block(
            self.pos.x.floor() as i32,
            (self.pos.y + EYE_HEIGHT).floor() as i32,
            self.pos.z.floor() as i32,
        );
        self.in_water = feet_block == WATER;
        self.head_in_water = head_block == WATER;
        if self.in_water && !self.was_in_water && self.vel.y < -4.0 {
            sounds.push(SoundEvent {
                family: SoundFamily::Water,
                volume: 0.8,
                pitch: 1.0,
            });
        }
        self.was_in_water = self.in_water;

        // wish direction (horizontal)
        let fwd = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        let mut wish = Vec3::ZERO;
        if input.fwd {
            wish += fwd;
        }
        if input.back {
            wish -= fwd;
        }
        if input.right {
            wish += right;
        }
        if input.left {
            wish -= right;
        }
        let has_input = wish.length_squared() > 1e-4;
        if has_input {
            wish = wish.normalize();
        }

        let sprinting = input.sprint && (input.fwd || input.back || input.left || input.right);

        if self.flying {
            let speed = if sprinting { FLY_SPRINT } else { FLY_SPEED };
            let mut target = wish * speed;
            target.y = if input.jump {
                speed * 0.8
            } else if input.sneak {
                -speed * 0.8
            } else {
                0.0
            };
            // smooth
            self.vel += (target - self.vel) * (12.0 * dt).min(1.0);
        } else {
            let speed = if self.in_water {
                if sprinting { SPRINT_SPEED * 0.55 } else { WALK_SPEED * 0.55 }
            } else if sprinting {
                SPRINT_SPEED
            } else {
                WALK_SPEED
            };
            let target = wish * speed;
            let rate = if self.on_ground { 12.0 } else { 2.5 };
            let f = (rate * dt).min(1.0);
            self.vel.x += (target.x - self.vel.x) * f;
            self.vel.z += (target.z - self.vel.z) * f;

            if self.in_water {
                // buoyant swimming
                let target_y = if input.jump { 3.5 } else { -2.2 };
                self.vel.y += (target_y - self.vel.y) * (6.0 * dt).min(1.0);
                self.vel.y -= 4.0 * dt;
                self.vel.y = self.vel.y.clamp(-4.0, 5.0);
            } else {
                if input.jump && self.on_ground {
                    self.vel.y = JUMP_VEL;
                    self.on_ground = false;
                }
                self.vel.y -= GRAVITY * dt;
                if self.vel.y < -TERMINAL {
                    self.vel.y = -TERMINAL;
                }
            }
        }

        // FOV: sprint zoom like MC
        let target_fov = if sprinting && !self.flying { self.fov * 1.12 } else { self.fov };
        self.fov_cur += (target_fov - self.fov_cur) * (10.0 * dt).min(1.0);

        // integrate with axis-separated collision substeps
        let delta = self.vel * dt;
        let max_comp = delta.x.abs().max(delta.y.abs()).max(delta.z.abs());
        let steps = (max_comp / 0.4).ceil().max(1.0) as i32;
        let step = delta / steps as f32;
        self.on_ground = false;
        for _ in 0..steps {
            self.move_axis(world, 0, step.x);
            self.move_axis(world, 1, step.y);
            self.move_axis(world, 2, step.z);
        }
        // grounded check: small downward probe
        if !self.flying && self.vel.y <= 0.0 {
            let probe = self.pos.y - 0.06;
            if Self::collides(world, Vec3::new(self.pos.x, probe, self.pos.z)) {
                self.on_ground = true;
            }
        }

        // footsteps
        if self.on_ground && !self.flying {
            let horiz = (self.vel.x * self.vel.x + self.vel.z * self.vel.z).sqrt() * dt;
            self.step_accum += horiz;
            let need = if sprinting { 1.7 } else { 2.1 };
            if self.step_accum > need {
                self.step_accum = 0.0;
                let under = world.get_block(
                    self.pos.x.floor() as i32,
                    (self.pos.y - 0.4).floor() as i32,
                    self.pos.z.floor() as i32,
                );
                if under != AIR {
                    sounds.push(SoundEvent {
                        family: def(under).sound,
                        volume: 0.32,
                        pitch: 0.92 + rand_small(time) * 0.16,
                    });
                }
            }
        } else {
            self.step_accum = 0.0;
        }

        sounds
    }

    fn move_axis(&mut self, world: &World, axis: usize, d: f32) {
        if d == 0.0 {
            return;
        }
        let mut p = self.pos;
        match axis {
            0 => p.x += d,
            1 => p.y += d,
            _ => p.z += d,
        }
        if !Self::collides(world, p) {
            self.pos = p;
            return;
        }
        // clamp against the block boundary we penetrated
        let eps = 0.001;
        let clamped = match axis {
            1 => {
                if d > 0.0 {
                    (p.y + PLAYER_HEIGHT).floor() - PLAYER_HEIGHT - eps
                } else {
                    self.on_ground = true;
                    p.y.floor() + 1.0 + eps
                }
            }
            0 => {
                if d > 0.0 {
                    (p.x + PLAYER_HALF).floor() - PLAYER_HALF - eps
                } else {
                    (p.x - PLAYER_HALF).floor() + 1.0 + PLAYER_HALF + eps
                }
            }
            _ => {
                if d > 0.0 {
                    (p.z + PLAYER_HALF).floor() - PLAYER_HALF - eps
                } else {
                    (p.z - PLAYER_HALF).floor() + 1.0 + PLAYER_HALF + eps
                }
            }
        };
        let mut q = self.pos;
        match axis {
            0 => q.x = clamped,
            1 => q.y = clamped,
            _ => q.z = clamped,
        }
        if !Self::collides(world, q) {
            self.pos = q;
        }
        match axis {
            0 => self.vel.x = 0.0,
            1 => self.vel.y = 0.0,
            _ => self.vel.z = 0.0,
        }
    }

    /// Does the player AABB at `p` overlap any solid block?
    fn collides(world: &World, p: Vec3) -> bool {
        let min_x = (p.x - PLAYER_HALF).floor() as i32;
        let max_x = (p.x + PLAYER_HALF).floor() as i32;
        let min_y = p.y.floor() as i32;
        let max_y = (p.y + PLAYER_HEIGHT - 0.001).floor() as i32;
        let min_z = (p.z - PLAYER_HALF).floor() as i32;
        let max_z = (p.z + PLAYER_HALF).floor() as i32;
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                for bx in min_x..=max_x {
                    if is_solid(world.get_block(bx, by, bz)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Would placing a solid block at `p` intersect the player?
    pub fn block_intersects_player(&self, p: [i32; 3]) -> bool {
        let bmin = Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32);
        let bmax = bmin + Vec3::ONE;
        let pmin = self.pos - Vec3::new(PLAYER_HALF, 0.0, PLAYER_HALF);
        let pmax = pmin + Vec3::new(PLAYER_HALF * 2.0, PLAYER_HEIGHT, PLAYER_HALF * 2.0);
        bmin.x < pmax.x && bmax.x > pmin.x
            && bmin.y < pmax.y && bmax.y > pmin.y
            && bmin.z < pmax.z && bmax.z > pmin.z
    }

    /// Double-tap space toggles fly.
    pub fn try_fly_toggle(&mut self, time: f32) {
        if time - self.last_space_tap < 0.28 {
            self.flying = !self.flying;
            if self.flying {
                self.vel.y = 0.0;
            }
            self.last_space_tap = -1.0;
        } else {
            self.last_space_tap = time;
        }
    }
}

/// Small deterministic wobble from time (no RNG dependency).
fn rand_small(t: f32) -> f32 {
    (t * 12.9898).sin().fract().abs()
}

/// DDA voxel raycast (Amanatides & Woo). Returns hit block + previous cell.
pub fn raycast(world: &World, eye: Vec3, dir: Vec3, max_dist: f32) -> Option<([i32; 3], u8, [i32; 3])> {
    let mut x = eye.x.floor() as i32;
    let mut y = eye.y.floor() as i32;
    let mut z = eye.z.floor() as i32;
    let step_x = if dir.x > 0.0 { 1 } else { -1 };
    let step_y = if dir.y > 0.0 { 1 } else { -1 };
    let step_z = if dir.z > 0.0 { 1 } else { -1 };
    // distance-per-block along each axis; clamp only the degenerate (0)
    // components so the DDA never steps on an unused axis. NOTE: .min, NOT
    // .max — .max would floor every step at 1e30 and the ray would stall
    // after one block (this exact bug broke all block targeting beyond ~1
    // block of the eye).
    let tdx = (1.0 / dir.x.abs()).min(1e30);
    let tdy = (1.0 / dir.y.abs()).min(1e30);
    let tdz = (1.0 / dir.z.abs()).min(1e30);
    let dist_to_boundary = |o: f32, d: f32| -> f32 {
        if d > 0.0 {
            (o.floor() + 1.0 - o) / d
        } else if d < 0.0 {
            (o - o.floor()) / (-d)
        } else {
            // axis unused by the ray: never cross a boundary on it.
            // (0/-0.0 = -inf clamped to 0 would make the DDA take a spurious
            // sideways step and track along a shifted column.)
            f32::INFINITY
        }
    };
    let mut tmx = dist_to_boundary(eye.x, dir.x).max(0.0);
    let mut tmy = dist_to_boundary(eye.y, dir.y).max(0.0);
    let mut tmz = dist_to_boundary(eye.z, dir.z).max(0.0);

    let mut prev = [x, y, z];
    let mut t = 0.0f32;
    for _ in 0..256 {
        let b = world.get_block(x, y, z);
        if b != AIR && b != WATER {
            return Some(([x, y, z], b, prev));
        }
        prev = [x, y, z];
        if tmx < tmy && tmx < tmz {
            t = tmx;
            x += step_x;
            tmx += tdx;
        } else if tmy < tmz {
            t = tmy;
            y += step_y;
            tmy += tdy;
        } else {
            t = tmz;
            z += step_z;
            tmz += tdz;
        }
        if t > max_dist {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn raycast_hits_ground_straight_down() {
        let mut world = World::new(42);
        let gen = crate::gen::TerrainGen::new(42);
        let (chunk, outbound) = gen.generate_chunk(0, 0, vec![]);
        world.insert_generated((0, 0), chunk, outbound);
        let eye = Vec3::new(0.5, 90.0, 0.5);
        let hit = raycast(&world, eye, Vec3::new(0.0, -1.0, 0.0), 50.0);
        assert!(hit.is_some(), "straight-down ray must hit terrain");
        let (p, b, _) = hit.unwrap();
        println!("hit block {} at {:?}", b, p);
        assert!(b != crate::blocks::AIR);
    }

    #[test]
    fn raycast_hits_forward_down_45() {
        let mut world = World::new(42);
        let gen = crate::gen::TerrainGen::new(42);
        // the ray travels in -z: load the neighbor chunk too
        for pos in [(0, 0), (0, -1), (-1, 0), (-1, -1)] {
            let (chunk, outbound) = gen.generate_chunk(pos.0, pos.1, vec![]);
            world.insert_generated(pos, chunk, outbound);
        }
        let top = chunk_top(&world);
        let eye = Vec3::new(0.5, top as f32 + 2.7, 0.5);
        let dir = Vec3::new(0.0, -0.9, -0.447).normalize();
        let hit = raycast(&world, eye, dir, 10.0);
        assert!(hit.is_some(), "45-degree down ray must hit terrain");
        let (p, _, _) = hit.unwrap();
        assert!(p[1] <= top, "hit should be at or below the surface");
    }

    fn chunk_top(world: &World) -> i32 {
        // scan down from 255 for the first solid block at (0, ?, 0)
        for y in (0..255).rev() {
            let b = world.get_block(0, y, 0);
            if b != crate::blocks::AIR && b != crate::blocks::WATER {
                return y + 1;
            }
        }
        64
    }
}



