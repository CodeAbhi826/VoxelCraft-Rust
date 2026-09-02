//! Villagers (Phase 7 §27/§29): village NPCs — deterministic auto-spawn at
//! village wells when their chunks generate, wander-around-home AI with
//! vanilla-observable movement (0.5 blocks/s walk, 1-block jump-ups), and
//! the per-profession trade tables the trade screen serves.
//!
//! Vanilla-exact pieces (VERIFIED):
//! - villagers spawn with the village, tied to its well (their home POI)
//! - wander radius around home, jumping single-block steps
//! - trade rows: pay N of one item → receive M of another; per profession
//!
//! Documented adaptations (palette-bounded):
//! - emerald item → EMERALD_ORE block (no standalone emerald item)
//! - 6 professions (vanilla has ~15; ours bound to our tradeable blocks)
//! - pathfinding is straight-line steering + step jumps (no full A*)

use crate::blocks::*;
use crate::rng::Rng;
use std::collections::HashSet;

pub const MAX_VILLAGERS: usize = 96;
/// vanilla villager walk speed (blocks/s)
pub const WALK_SPEED: f32 = 0.5;
/// vanilla jump height ≈ 1.25 blocks → our 1-block step-up velocity
pub const JUMP_VEL: f32 = 0.42;

/// professions (§27 registry, palette-bounded subset)
pub const PROFESSIONS: [&str; 6] = [
    "Farmer", "Librarian", "Cleric", "Armorer", "Butcher", "Fletcher",
];

/// one trade row: pay (block, count) → receive (block, count)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    pub give: (u8, u8),
    pub get: (u8, u8),
}

/// per-profession trade tables (§29 data-driven). Each row mirrors a
/// vanilla-style deal: farmers buy crops, clerics sell potions, librarians
/// sell books — tied to this engine's palette.
pub fn trades(profession: u8) -> &'static [Trade] {
    const FARMER: &[Trade] = &[
        Trade { give: (MELON, 16), get: (EMERALD_ORE, 1) },      // buy crop
        Trade { give: (EMERALD_ORE, 1), get: (PUMPKIN, 6) },     // sell crop
    ];
    const LIBRARIAN: &[Trade] = &[
        Trade { give: (EMERALD_ORE, 2), get: (ENCHANTED_BOOK, 1) }, // sell books
        Trade { give: (BOOKSHELF, 3), get: (EMERALD_ORE, 1) },      // buy paper-goods
    ];
    const CLERIC: &[Trade] = &[
        Trade { give: (EMERALD_ORE, 3), get: (POTION_HEALING, 1) }, // sell potions
        Trade { give: (POTION_HEALING_II, 1), get: (EMERALD_ORE, 4) }, // buy fine potions
    ];
    const ARMORER: &[Trade] = &[
        Trade { give: (EMERALD_ORE, 4), get: (IRON_BLOCK, 1) },  // sell metal
        Trade { give: (GOLD_BLOCK, 1), get: (EMERALD_ORE, 2) },  // buy metal
    ];
    const BUTCHER: &[Trade] = &[
        Trade { give: (MUSHROOM_BROWN, 16), get: (EMERALD_ORE, 1) }, // buy food
        Trade { give: (EMERALD_ORE, 1), get: (MUSHROOM_RED, 8) },    // sell food
    ];
    const FLETCHER: &[Trade] = &[
        Trade { give: (TALL_GRASS, 16), get: (EMERALD_ORE, 1) },   // buy stalks
        Trade { give: (EMERALD_ORE, 1), get: (BIRCH_LOG, 4) },     // sell wood
    ];
    match profession {
        0 => FARMER,
        1 => LIBRARIAN,
        2 => CLERIC,
        3 => ARMORER,
        4 => BUTCHER,
        _ => FLETCHER,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Villager {
    pub id: u32,
    /// feet center
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub yaw: f32,
    /// the home anchor (the village well) — wander stays within its radius
    pub home: [i32; 3],
    pub profession: u8,
    /// wander target (None = idle stand)
    target: Option<[f32; 3]>,
    /// ticks until the next wander decision
    wander_t: i32,
    /// cooldown after a jump
    jump_cd: i32,
}

pub struct Villagers {
    pub list: Vec<Villager>,
    rng: Rng,
    next_id: u32,
    /// village wells already populated (never double-spawn)
    populated: HashSet<[i32; 2]>,
    /// total trades executed since boot (stats/F3/E2E)
    pub trades_done: u64,
    /// total ever spawned (E2E)
    pub spawned_total: u64,
}

impl Villagers {
    pub fn new(seed: u64) -> Self {
        Villagers {
            list: Vec::with_capacity(32),
            rng: Rng::new(seed ^ 0x1A11A_9E),
            next_id: 1,
            populated: HashSet::new(),
            trades_done: 0,
            spawned_total: 0,
        }
    }

    /// spawn one villager at a position with a chosen (or random) profession
    pub fn spawn_at(&mut self, wx: i32, wy: i32, wz: i32, profession: Option<u8>) -> Option<u32> {
        if self.list.len() >= MAX_VILLAGERS {
            return None;
        }
        let prof = profession.unwrap_or_else(|| self.rng.next_range(PROFESSIONS.len() as u32) as u8);
        let id = self.next_id;
        self.next_id += 1;
        self.list.push(Villager {
            id,
            pos: [wx as f32 + 0.5, wy as f32 + 0.1, wz as f32 + 0.5],
            vel: [0.0; 3],
            yaw: 0.0,
            home: [wx, wy, wz],
            profession: prof,
            target: None,
            wander_t: (self.rng.next_range(40) as i32).max(10),
            jump_cd: 0,
        });
        self.spawned_total += 1;
        Some(id)
    }

    /// populate the villages whose reach covers the given chunk — called
    /// once per generated chunk; each village well seeds 3..5 villagers
    /// (deterministic from the world seed + well position)
    pub fn populate_villages(&mut self, world: &crate::world::World, cx: i32, cz: i32) {
        let ox = cx * 16;
        let oz = cz * 16;
        for (wx, wz) in world.gen.villages_near(ox, oz) {
            if self.populated.contains(&[wx, wz]) {
                continue;
            }
            self.populated.insert([wx, wz]);
            // ground at the well: the generator's column height (the well
            // sits at height+1; villagers stand on the well rim area)
            let h = world.gen.column(wx, wz).height;
            let mut rng = Rng::new(Rng::hash3(world.seed, wx, 0x0E5, wz));
            let n = 3 + rng.next_range(3) as usize; // 3..5
            for _ in 0..n {
                let dx = rng.next_range(7) as i32 - 3;
                let dz = rng.next_range(7) as i32 - 3;
                let gy = world.gen.column(wx + dx, wz + dz).height;
                let prof = rng.next_range(PROFESSIONS.len() as u32) as u8;
                self.spawn_at(wx + dx, gy.max(h) + 1, wz + dz, Some(prof));
            }
        }
    }

    /// the villager under the crosshair: vertical-capsule test (0.6 wide,
    /// 1.9 tall — the vanilla villager hitbox) against the ray, within
    /// `max_dist` (interaction reach)
    pub fn ray_hit(&self, eye: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<u32> {
        let mut best: Option<(f32, u32)> = None;
        for v in &self.list {
            // project the capsule axis onto the ray
            let ox = v.pos[0] - eye[0];
            let oy = (v.pos[1] + 0.95) - eye[1]; // axis center
            let oz = v.pos[2] - eye[2];
            let t = ox * dir[0] + oy * dir[1] + oz * dir[2];
            if t < 0.0 || t > max_dist {
                continue;
            }
            let px = eye[0] + dir[0] * t;
            let py = eye[1] + dir[1] * t;
            let pz = eye[2] + dir[2] * t;
            // horizontal distance to the axis + vertical containment
            let hd2 = (px - v.pos[0]).powi(2) + (pz - v.pos[2]).powi(2);
            let v_ok = py >= v.pos[1] - 0.1 && py <= v.pos[1] + 1.9;
            if hd2 < 0.45 * 0.45 && v_ok {
                if best.map(|(bt, _)| t < bt).unwrap_or(true) {
                    best = Some((t, v.id));
                }
            }
        }
        best.map(|(_, id)| id)
    }

    pub fn by_id(&self, id: u32) -> Option<&Villager> {
        self.list.iter().find(|v| v.id == id)
    }

    /// ONE sim tick (20 Hz): wander decisions + walking physics
    pub fn tick(&mut self, world: &crate::world::World) {
        for v in self.list.iter_mut() {
            // wander state machine: idle countdown → pick a target (with a
            // generous walk deadline) → walk until arrival or deadline →
            // idle again. wander_t is the countdown in BOTH states.
            if v.target.is_none() {
                if v.wander_t > 0 {
                    v.wander_t -= 1;
                } else {
                    let ang = self.rng.next_f32() * std::f32::consts::TAU;
                    let r = 2.0 + self.rng.next_f32() * 6.0;
                    v.target = Some([
                        v.home[0] as f32 + 0.5 + ang.cos() * r,
                        0.0, // y resolved during walking
                        v.home[2] as f32 + 0.5 + ang.sin() * r,
                    ]);
                    // walk deadline: 15..25 s covers any wander leg
                    v.wander_t = 300 + self.rng.next_range(200) as i32;
                }
            } else if v.wander_t > 0 {
                v.wander_t -= 1;
            } else {
                // gave up → idle
                v.target = None;
                v.wander_t = 40 + self.rng.next_range(80) as i32; // 2..6 s
            }

            // steering toward the target
            if let Some(t) = v.target {
                let dx = t[0] - v.pos[0];
                let dz = t[2] - v.pos[2];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < 0.35 {
                    v.target = None;
                    v.wander_t = 40 + self.rng.next_range(80) as i32;
                } else {
                    let speed = WALK_SPEED / 20.0; // per tick
                    v.vel[0] = dx / dist * speed;
                    v.vel[2] = dz / dist * speed;
                    v.yaw = dz.atan2(dx);
                }
            } else {
                v.vel[0] *= 0.6;
                v.vel[2] *= 0.6;
            }

            if v.jump_cd > 0 {
                v.jump_cd -= 1;
            }

            // physics: gravity + axis-separated collision (the item-entity
            // pattern, villager-scale); jump when horizontally blocked
            v.vel[1] -= 0.08;
            v.vel[1] = v.vel[1].max(-0.5);

            // horizontal X
            let nx = v.pos[0] + v.vel[0];
            if !solid_at(world, nx, v.pos[1] + 0.1, v.pos[2])
                && !solid_at(world, nx, v.pos[1] + 1.5, v.pos[2])
            {
                v.pos[0] = nx;
            } else if v.on_ground(world) && v.jump_cd == 0 {
                v.vel[1] = JUMP_VEL;
                v.jump_cd = 10;
            } else {
                v.vel[0] = 0.0;
            }
            // horizontal Z
            let nz = v.pos[2] + v.vel[2];
            if !solid_at(world, v.pos[0], v.pos[1] + 0.1, nz)
                && !solid_at(world, v.pos[0], v.pos[1] + 1.5, nz)
            {
                v.pos[2] = nz;
            } else if v.on_ground(world) && v.jump_cd == 0 {
                v.vel[1] = JUMP_VEL;
                v.jump_cd = 10;
            } else {
                v.vel[2] = 0.0;
            }
            // vertical
            let ny = v.pos[1] + v.vel[1];
            if v.vel[1] < 0.0 && solid_at(world, v.pos[0], ny, v.pos[2]) {
                v.pos[1] = ny.floor() + 1.0; // rest on the surface
                v.vel[1] = 0.0;
                v.vel[0] *= 0.7;
                v.vel[2] *= 0.7;
            } else if v.vel[1] > 0.0 && solid_at(world, v.pos[0], ny + 1.8, v.pos[2]) {
                v.vel[1] = 0.0;
            } else {
                v.pos[1] = ny;
            }

            // never sink below bedrock world floor
            if v.pos[1] < 1.0 {
                v.pos[1] = 1.0;
                v.vel[1] = 0.0;
            }
        }
    }
}

impl Villager {
    fn on_ground(&self, world: &crate::world::World) -> bool {
        solid_at(world, self.pos[0], self.pos[1] - 0.05, self.pos[2])
    }
}

fn solid_at(world: &crate::world::World, x: f32, y: f32, z: f32) -> bool {
    is_solid(world.get_block(x.floor() as i32, y.floor() as i32, z.floor() as i32))
}

/// billboard quads for the render pass: one crossed pair per villager,
/// villager-scale (0.6 × 1.9) — rides the same particle pipeline the item
/// entities use
#[allow(clippy::too_many_arguments)]
pub fn build_vertices(
    list: &[Villager],
    time: f32,
    right: [f32; 3],
    up: [f32; 3],
    out: &mut Vec<crate::particles::ParticleVertex>,
) {
    let tile = TILE_VILLAGER as u16;
    let tx = (tile % 16) as f32;
    let ty = (tile / 16) as f32;
    for v in list {
        // face the movement direction (billboard around world Y)
        let yaw = v.yaw + time * 0.0; // no spin — grounded NPCs
        let (s, c) = (yaw.sin(), yaw.cos());
        let rr = [c * right[0] + s * right[2], 0.0, -s * right[0] + c * right[2]];
        let half = 0.30f32;
        let h = 1.9f32;
        let col = [0.92, 0.86, 0.78]; // baked neutral light (villager robe tones come from the tile)
        let bob = 0.0;
        // the quad spans feet..head
        let corners = [
            (
                [-rr[0] * half, 0.0, -rr[2] * half],
                [tx / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [rr[0] * half, 0.0, rr[2] * half],
                [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [rr[0] * half, h, rr[2] * half],
                [(tx + 1.0) / 16.0, ty / 16.0],
            ),
            (
                [-rr[0] * half, h, -rr[2] * half],
                [tx / 16.0, ty / 16.0],
            ),
        ];
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let (c, uv) = corners[ci];
            out.push(crate::particles::ParticleVertex {
                pos: [v.pos[0] + c[0], v.pos[1] + c[1] + bob, v.pos[2] + c[2]],
                uv: [uv[0], uv[1]],
                col,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    use std::sync::Arc;

    fn flat_world() -> crate::world::World {
        let mut w = crate::world::World::new(9);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
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
    fn spawn_walks_and_stays_near_home() {
        let mut vs = Villagers::new(3);
        let id = vs.spawn_at(0, 65, 0, Some(0)).unwrap();
        let w = flat_world();
        // 10 simulated seconds of wandering
        for _ in 0..200 {
            vs.tick(&w);
        }
        let v = vs.by_id(id).unwrap();
        let d2 = (v.pos[0] - 0.5).powi(2) + (v.pos[2] - 0.5).powi(2);
        assert!(d2 < 9.0 * 9.0, "villager stays within the wander radius, d={}", d2.sqrt());
        assert!(v.pos[1] >= 64.0 && v.pos[1] < 67.0, "on the ground: {}", v.pos[1]);
        assert_eq!(vs.list.len(), 1);
    }

    #[test]
    fn jumps_single_block_steps() {
        let mut w = flat_world();
        // a 1-block wall ahead
        let _ = w.set_block(2, 65, 0, COBBLE);
        let mut vs = Villagers::new(4);
        let id = vs.spawn_at(0, 65, 0, Some(1)).unwrap();
        // steer straight at the wall: force a target behind it + a fresh
        // walk deadline (the state machine would otherwise idle first)
        let i = vs.list.iter().position(|v| v.id == id).unwrap();
        vs.list[i].target = Some([6.5, 0.0, 0.5]);
        vs.list[i].wander_t = 400;
        let mut jumped = false;
        for _ in 0..120 {
            vs.tick(&w);
            let v = vs.by_id(id).unwrap();
            if v.vel[1] > 0.1 {
                jumped = true;
            }
        }
        assert!(jumped, "villager jumps the 1-block step");
        let v = vs.by_id(id).unwrap();
        assert!(v.pos[0] > 2.0, "villager crossed the wall: {}", v.pos[0]);
    }

    #[test]
    fn ray_hit_finds_the_crosshair_villager() {
        let mut vs = Villagers::new(5);
        let id = vs.spawn_at(0, 65, 0, Some(2)).unwrap();
        let eye = [0.5, 66.6, -3.0];
        let dir = [0.0, 0.0, 1.0];
        assert_eq!(vs.ray_hit(eye, dir, 8.0), Some(id));
        // looking away: nothing
        assert_eq!(vs.ray_hit(eye, [1.0, 0.0, 0.0], 8.0), None);
        // too far: nothing
        assert_eq!(vs.ray_hit([0.5, 66.6, -30.0], dir, 8.0), None);
    }

    #[test]
    fn trade_tables_cover_all_professions() {
        for p in 0..PROFESSIONS.len() as u8 {
            let t = trades(p);
            assert!(t.len() >= 2, "{} needs trades", PROFESSIONS[p as usize]);
            for tr in t {
                // every side is a real, obtainable item/block
                assert!(tr.give.0 != AIR && tr.get.0 != AIR);
                assert!(tr.give.1 > 0 && tr.get.1 > 0);
                assert_ne!(tr.give.0, tr.get.0, "no self-trades");
            }
        }
    }

    #[test]
    fn populate_uses_the_village_well_once() {
        // whatever villages this world's generator reports near chunk
        // (0,0) — the invariant is: the SECOND populate call for the same
        // chunk is a complete no-op (the well set guards double-spawn)
        let mut vs = Villagers::new(6);
        let w = flat_world();
        vs.populate_villages(&w, 0, 0);
        let n = vs.list.len();
        assert!(n == 0 || (3..=5).contains(&n), "villages seed 3..5 villagers, got {n}");
        vs.populate_villages(&w, 0, 0);
        assert_eq!(vs.list.len(), n, "second populate: zero new spawns");
    }
}
