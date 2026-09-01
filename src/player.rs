// Player controller: physics, AABB collision, raycast for block targeting.

use glam::{Vec3, IVec3, FloatExt};
use crate::blocks::Block;
use crate::world::world::World;
use crate::world::chunk::WORLD_HEIGHT;

pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE: f32 = 1.62;
pub const GRAVITY: f32 = 28.0;
pub const JUMP_VELOCITY: f32 = 9.0;
pub const WALK_SPEED: f32 = 4.5;
pub const SPRINT_SPEED: f32 = 7.5;
pub const FLY_SPEED: f32 = 9.0;
pub const FLY_SPRINT_SPEED: f32 = 22.0;
pub const SWIM_SPEED: f32 = 3.5;
pub const TERMINAL_VELOCITY: f32 = -30.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub crouch: bool,
}

#[derive(Debug)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,    // around Y, radians
    pub pitch: f32,  // around X, radians
    pub on_ground: bool,
    pub in_water: bool,
    pub flying: bool,
    pub sprinting: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.5, 40.0, 0.5),
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            in_water: false,
            flying: false,
            sprinting: false,
        }
    }
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn eye_position(&self) -> Vec3 {
        Vec3::new(self.position.x, self.position.y + PLAYER_EYE, self.position.z)
    }

    pub fn forward_vector(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn horizontal_forward(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
    }

    pub fn horizontal_right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin())
    }

    /// Find a safe spawn Y above the terrain at (x, z).
    pub fn find_spawn(world: &World, x: i32, z: i32) -> f32 {
        for y in (0..WORLD_HEIGHT as i32).rev() {
            if world.get_block(x, y, z).is_solid() {
                return (y + 2) as f32;
            }
        }
        40.0
    }

    /// Update physics + collision. `dt` is clamped internally to 0.1s.
    pub fn update(&mut self, dt: f32, input: InputState, world: &World) {
        let dt = dt.min(0.1);
        self.in_water = self.check_in_water(world);

        // Build desired horizontal velocity from input
        let fwd = self.horizontal_forward();
        let right = self.horizontal_right();
        let mut move_dir = Vec3::ZERO;
        if input.forward { move_dir += fwd; }
        if input.back { move_dir -= fwd; }
        if input.right { move_dir += right; }
        if input.left { move_dir -= right; }

        self.sprinting = input.sprint && input.forward && !self.flying;
        let speed = if self.flying {
            if input.sprint { FLY_SPRINT_SPEED } else { FLY_SPEED }
        } else if self.in_water {
            SWIM_SPEED
        } else if self.sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize() * speed;
        }

        // Smooth acceleration toward target velocity
        let accel = if self.on_ground { 15.0 } else { 5.0 };
        let t = (accel * dt).min(1.0);
        self.velocity.x = self.velocity.x.lerp(move_dir.x, t);
        self.velocity.z = self.velocity.z.lerp(move_dir.z, t);

        // Vertical
        if self.flying {
            let mut vy = 0.0;
            if input.jump { vy += speed; }
            if input.crouch { vy -= speed; }
            self.velocity.y = vy;
        } else if self.in_water {
            self.velocity.y -= GRAVITY * 0.25 * dt;
            self.velocity.y = self.velocity.y.max(-4.0);
            if input.jump { self.velocity.y = 4.0; }
        } else {
            self.velocity.y -= GRAVITY * dt;
            self.velocity.y = self.velocity.y.max(TERMINAL_VELOCITY);
            if input.jump && self.on_ground {
                self.velocity.y = JUMP_VELOCITY;
                self.on_ground = false;
            }
        }

        // Move with per-axis collision
        let delta = self.velocity * dt;
        self.move_axis_x(delta.x, world);
        self.move_axis_y(delta.y, world);
        self.move_axis_z(delta.z, world);

        // Void respawn
        if self.position.y < -10.0 {
            let spawn_y = Self::find_spawn(world, self.position.x as i32, self.position.z as i32);
            self.position = Vec3::new(self.position.x, spawn_y, self.position.z);
            self.velocity = Vec3::ZERO;
        }
    }

    fn check_in_water(&self, world: &World) -> bool {
        let feet = world.get_block(
            self.position.x as i32,
            (self.position.y + 0.5) as i32,
            self.position.z as i32,
        );
        feet == Block::Water
    }

    fn collides(&self, pos: Vec3, world: &World) -> bool {
        let min_x = (pos.x - PLAYER_WIDTH / 2.0).floor() as i32;
        let max_x = (pos.x + PLAYER_WIDTH / 2.0).floor() as i32;
        let min_y = pos.y.floor() as i32;
        let max_y = (pos.y + PLAYER_HEIGHT - 0.001).floor() as i32;
        let min_z = (pos.z - PLAYER_WIDTH / 2.0).floor() as i32;
        let max_z = (pos.z + PLAYER_WIDTH / 2.0).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.get_block(x, y, z).is_solid() {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn move_axis_x(&mut self, dx: f32, world: &World) {
        if dx == 0.0 { return; }
        let mut new_pos = self.position;
        new_pos.x += dx;
        if !self.collides(new_pos, world) {
            self.position.x = new_pos.x;
        } else {
            self.velocity.x = 0.0;
        }
    }

    fn move_axis_y(&mut self, dy: f32, world: &World) {
        if dy == 0.0 { return; }
        let mut new_pos = self.position;
        new_pos.y += dy;
        if !self.collides(new_pos, world) {
            self.position.y = new_pos.y;
            if dy < 0.0 { self.on_ground = false; }
        } else {
            if dy < 0.0 {
                self.on_ground = true;
            }
            self.velocity.y = 0.0;
        }
    }

    fn move_axis_z(&mut self, dz: f32, world: &World) {
        if dz == 0.0 { return; }
        let mut new_pos = self.position;
        new_pos.z += dz;
        if !self.collides(new_pos, world) {
            self.position.z = new_pos.z;
        } else {
            self.velocity.z = 0.0;
        }
    }

    /// DDA voxel raycast from eye position. Returns (block_pos, face_normal) of first hit.
    pub fn raycast(&self, world: &World, max_dist: f32) -> Option<(IVec3, IVec3)> {
        let origin = self.eye_position();
        let dir = self.forward_vector().normalize();

        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else if dir.x < 0.0 { -1 } else { 0 };
        let step_y = if dir.y > 0.0 { 1 } else if dir.y < 0.0 { -1 } else { 0 };
        let step_z = if dir.z > 0.0 { 1 } else if dir.z < 0.0 { -1 } else { 0 };

        let t_delta_x = if dir.x != 0.0 { 1.0 / dir.x.abs() } else { f32::INFINITY };
        let t_delta_y = if dir.y != 0.0 { 1.0 / dir.y.abs() } else { f32::INFINITY };
        let t_delta_z = if dir.z != 0.0 { 1.0 / dir.z.abs() } else { f32::INFINITY };

        let dist_to_edge = |o: f32, s: i32| -> f32 {
            if s > 0 { (o.floor() + 1.0 - o) }
            else if s < 0 { o - o.floor() }
            else { 0.0 }
        };
        let mut t_max_x = if step_x != 0 { dist_to_edge(origin.x, step_x) * t_delta_x } else { f32::INFINITY };
        let mut t_max_y = if step_y != 0 { dist_to_edge(origin.y, step_y) * t_delta_y } else { f32::INFINITY };
        let mut t_max_z = if step_z != 0 { dist_to_edge(origin.z, step_z) * t_delta_z } else { f32::INFINITY };

        let mut normal = IVec3::ZERO;
        let mut t = 0.0;

        while t <= max_dist {
            let block = world.get_block(x, y, z);
            if !block.is_air() && block != Block::Water {
                return Some((IVec3::new(x, y, z), normal));
            }

            if t_max_x < t_max_y && t_max_x < t_max_z {
                x += step_x;
                t = t_max_x;
                t_max_x += t_delta_x;
                normal = IVec3::new(-step_x, 0, 0);
            } else if t_max_y < t_max_z {
                y += step_y;
                t = t_max_y;
                t_max_y += t_delta_y;
                normal = IVec3::new(0, -step_y, 0);
            } else {
                z += step_z;
                t = t_max_z;
                t_max_z += t_delta_z;
                normal = IVec3::new(0, 0, -step_z);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_falls_with_gravity() {
        let world = World::new(42);
        let mut player = Player::new(Vec3::new(0.5, 50.0, 0.5));
        let input = InputState::default();
        let initial_y = player.position.y;
        player.update(0.1, input, &world);
        assert!(player.position.y < initial_y, "player should fall");
        assert!(player.velocity.y < 0.0, "velocity should be negative");
    }

    #[test]
    fn player_lands_on_ground() {
        let world = World::new(42);
        // Place a solid floor at y=10
        for x in -2..=2 {
            for z in -2..=2 {
                world.set_block(x, 10, z, Block::Stone);
            }
        }
        let mut player = Player::new(Vec3::new(0.5, 12.0, 0.5));
        let input = InputState::default();

        // Simulate enough frames to land
        for _ in 0..200 {
            player.update(0.05, input, &world);
        }
        assert!(player.on_ground, "player should be on ground");
        assert!(player.position.y >= 11.0, "player y {} should be >= 11", player.position.y);
        assert!(player.position.y < 12.0, "player y {} should be < 12", player.position.y);
    }

    #[test]
    fn player_collides_with_walls() {
        let world = World::new(42);
        // Build a wall at x=2
        for y in 0..20 {
            for z in -1..=1 {
                world.set_block(2, y, z, Block::Stone);
            }
        }
        let mut player = Player::new(Vec3::new(0.5, 5.0, 0.5));
        // Directly move the player toward +X in small steps, bypassing update()
        for _ in 0..100 {
            player.move_axis_x(0.1, &world);
        }
        // Player should be blocked by the wall at x=2 (stops around x=1.4-1.7)
        assert!(player.position.x < 1.8, "player x {} passed through wall", player.position.x);
    }

    #[test]
    fn raycast_hits_block() {
        let world = World::new(42);
        // Place a block at (0, 5, 5)
        world.set_block(0, 5, 5, Block::Stone);
        assert_eq!(world.get_block(0, 5, 5), Block::Stone, "block should be placed");
        // Player at y=5 - 1.62 + 1 = 4.38... no. Eye = position.y + 1.62.
        // For eye at y=5 (block center), position.y = 5 - 1.62 = 3.38
        let mut player = Player::new(Vec3::new(0.5, 3.38, 0.5));
        // Face +Z: forward = (-sin(yaw), 0, -cos(yaw)) = (0, 0, 1) requires yaw = PI
        player.yaw = std::f32::consts::PI;
        player.pitch = 0.0;

        let hit = player.raycast(&world, 10.0);
        assert!(hit.is_some(), "raycast should hit the block");
        let (pos, _normal) = hit.unwrap();
        assert_eq!(pos.x, 0, "raycast x should be 0, got {}", pos.x);
        assert_eq!(pos.z, 5, "raycast z should be 5, got {}", pos.z);
    }

    #[test]
    fn raycast_misses_empty_space() {
        let world = World::new(42);
        let player = Player::new(Vec3::new(0.5, 50.0, 0.5));
        let hit = player.raycast(&world, 5.0);
        assert!(hit.is_none(), "raycast should not hit anything in empty space");
    }

    #[test]
    fn void_respawn_works() {
        let world = World::new(42);
        // Place a floor at y=5
        for x in -2..=2 {
            for z in -2..=2 {
                world.set_block(x, 5, z, Block::Stone);
            }
        }
        let mut player = Player::new(Vec3::new(0.5, -20.0, 0.5));
        let input = InputState::default();
        player.update(0.1, input, &world);
        // Should have been respawned above y=5
        assert!(player.position.y > 0.0, "player should respawn, got y={}", player.position.y);
    }
}
