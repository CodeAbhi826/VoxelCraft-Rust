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

/// One cuboid face: (corners CCW seen from outside, normal, shade, tile).
type CuboidFace = ([[f32; 3]; 4], [f32; 3], f32, u16);

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
                // VERIFIED entity physics table (reference wiki /
                // Falling_Block, research-verdicts.md live round):
                // Drag-Y 0.98 applies in air too — items share the
                // falling-block profile (gravity 0.04, drag 0.98,
                // terminal 1.96 b/t)
                it.vel[1] *= 0.98;
            }
        }
        self.items.retain(|it| it.age < DESPAWN_TICKS);
    }

    /// vanilla pickup: the item must intersect the player's AABB
    /// (0.6×1.8×0.6, feet-anchored) INFLATED by (1.0, 0.5, 1.0) — and be
    /// past the 10-tick pickup delay. Returns the picked-up block ids
    /// (the caller routes them into the inventory).
    ///
    /// 2026-09-21 root cause of "nothing gets into the inventory": the
    /// old test measured 3D distance from the EYE with radius 1.0 — an
    /// item resting at the player's FEET is ≥1.57 below the eye
    /// (d² ≥ 2.46 > 1.0, ALWAYS out of reach), so items only ever
    /// entered the inventory by never landing. The feet-anchored box
    /// is the vanilla rule (ItemEntity.playerTouch: AABB grow +
    /// intersect). The old unit test passed only because it handed in
    /// a fabricated "eye" at item height.
    pub fn collect(&mut self, feet: [f32; 3]) -> Vec<u16> {
        const PLAYER_HALF: f32 = 0.3;
        const PLAYER_HEIGHT: f32 = 1.8;
        const INFLATE_H: f32 = 1.0;
        const INFLATE_Y: f32 = 0.5;
        let hh = PLAYER_HALF + INFLATE_H;
        let mut picked = Vec::new();
        let mut i = 0;
        while i < self.items.len() {
            let it = &self.items[i];
            let near = (it.pos[0] - feet[0]).abs() <= hh
                && (it.pos[2] - feet[2]).abs() <= hh
                && it.pos[1] >= feet[1] - INFLATE_Y
                && it.pos[1] <= feet[1] + PLAYER_HEIGHT + INFLATE_Y;
            if near && it.age > PICKUP_DELAY {
                picked.push(it.block);
                self.items.remove(i);
            } else {
                i += 1;
            }
        }
        self.picked_total += picked.len() as u64;
        picked
    }

    /// 3D mini-block rendering (2026-09-21 "the broken stuff on the
    /// ground is just black stuff" round): a 0.25-block cuboid with
    /// per-face tiles (state_tiles), vanilla face shading
    /// (top 1.0 / bottom 0.5 / X 0.6 / Z 0.8), spinning around Y with
    /// the vanilla bob — replacing the old flat billboard quad
    /// ("not the item in 3D moving like the real game"). Faces are
    /// painter-ordered far→near by the face normal vs the camera
    /// direction (the shared billboard pipeline has no depth-write,
    /// exactly like push_held_item's cube and the entity models).
    pub fn build_vertices(
        &self,
        time: f32,
        right: [f32; 3],
        up: [f32; 3],
        dir: [f32; 3],
        out: &mut Vec<vc_particles::particles::ParticleVertex>,
    ) {
        let _ = (right, up); // basis unused: the cube is world-space
        let half = 0.125f32; // 0.25-block item cuboid (vanilla)
        for it in self.items.iter() {
            let tiles = state_tiles(it.block);
            let bob = (time * 2.2 + it.pos[0] + it.pos[2]).sin() * 0.04;
            let cy = it.pos[1] + half + 0.1 + bob; // hover just off the floor
            let ang = time * 1.6;
            let (s, c) = (ang.sin(), ang.cos());
            // world-space Y rotation of a local (x, y, z) offset
            let rot = |x: f32, z: f32| [c * x + s * z, -s * x + c * z];
            let col = |k: f32| {
                [
                    it.light * it.tint[0] * k,
                    it.light * it.tint[1] * k,
                    it.light * it.tint[2] * k,
                ]
            };
            // (corners CCW seen from outside, normal, shade, tile, uv per corner)
            let faces: [CuboidFace; 6] = [
                // +Y top
                (
                    [
                        [-half, half, -half],
                        [half, half, -half],
                        [half, half, half],
                        [-half, half, half],
                    ],
                    [0.0, 1.0, 0.0],
                    1.0,
                    tiles[0],
                ),
                // −Y bottom
                (
                    [
                        [-half, -half, half],
                        [half, -half, half],
                        [half, -half, -half],
                        [-half, -half, -half],
                    ],
                    [0.0, -1.0, 0.0],
                    0.5,
                    tiles[1],
                ),
                // +X
                (
                    [
                        [half, -half, -half],
                        [half, half, -half],
                        [half, half, half],
                        [half, -half, half],
                    ],
                    [1.0, 0.0, 0.0],
                    0.6,
                    tiles[2],
                ),
                // −X
                (
                    [
                        [-half, -half, half],
                        [-half, half, half],
                        [-half, half, -half],
                        [-half, -half, -half],
                    ],
                    [-1.0, 0.0, 0.0],
                    0.6,
                    tiles[2],
                ),
                // +Z
                (
                    [
                        [half, -half, half],
                        [half, half, half],
                        [-half, half, half],
                        [-half, -half, half],
                    ],
                    [0.0, 0.0, 1.0],
                    0.8,
                    tiles[3],
                ),
                // −Z
                (
                    [
                        [-half, -half, -half],
                        [-half, half, -half],
                        [half, half, -half],
                        [half, -half, -half],
                    ],
                    [0.0, 0.0, -1.0],
                    0.8,
                    tiles[3],
                ),
            ];
            // painter order: farthest-from-camera face first (its normal
            // points most WITH the view direction); nearest last
            let mut order: [usize; 6] = [0, 1, 2, 3, 4, 5];
            order.sort_by(|a, b| {
                let na = rot(faces[*a].1[0], faces[*a].1[2]);
                let nb = rot(faces[*b].1[0], faces[*b].1[2]);
                let da = na[0] * dir[0] + na[1] * dir[2];
                let db = nb[0] * dir[0] + nb[1] * dir[2];
                db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
            });
            for fi in order {
                let (corners, _nrm, shade, tile) = &faces[fi];
                // [1.12 fix] 32-tile atlas rows (was %16//16)
                let tx = (tile % 32) as f32;
                let ty = (tile / 32) as f32;
                let col = col(*shade);
                // UV: v flipped so texture top = block top (side faces);
                // top/bottom map the tile straight on
                let uvs = if fi < 2 {
                    [
                        [tx / 32.0, (ty + 1.0) / 32.0],
                        [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
                        [(tx + 1.0) / 32.0, ty / 32.0],
                        [tx / 32.0, ty / 32.0],
                    ]
                } else {
                    [
                        [tx / 32.0, (ty + 1.0) / 32.0],
                        [tx / 32.0, ty / 32.0],
                        [(tx + 1.0) / 32.0, ty / 32.0],
                        [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
                    ]
                };
                for ci in [0usize, 1, 2, 0, 2, 3] {
                    let cn = corners[ci];
                    let rz = rot(cn[0], cn[2]);
                    out.push(vc_particles::particles::ParticleVertex {
                        pos: [it.pos[0] + rz[0], cy + cn[1], it.pos[2] + rz[1]],
                        uv: [uvs[ci][0], uvs[ci][1]],
                        col,
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Phase E1 — XP orbs (evolution 1.0–1.2 bracket, all values live-verified
// 2026-09-06 against reference wiki /Experience; see
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
            let tile = if o.value >= 17 {
                TILE_XP_ORB_BIG
            } else {
                TILE_XP_ORB
            };
            // [1.12 fix] 32-tile atlas rows (was %16//16)
            let tx = (tile % 32) as f32;
            let ty = (tile / 32) as f32;
            // green↔yellow flash (VERIFIED: "fade between green and yellow")
            let flash = 0.5 + 0.5 * (time * 3.0 + o.pos[0]).sin();
            let col = [0.55 + flash * 0.45, 0.85 + flash * 0.15, 0.25];
            let bob = (time * 2.0 + o.pos[0] + o.pos[2]).sin() * 0.05;
            let half = 0.12f32;
            let corners = [
                (
                    [
                        -right[0] * half - up[0] * half,
                        -right[1] * half - up[1] * half,
                        -right[2] * half - up[2] * half,
                    ],
                    [tx / 32.0, ty / 32.0],
                ),
                (
                    [
                        right[0] * half - up[0] * half,
                        right[1] * half - up[1] * half,
                        right[2] * half - up[2] * half,
                    ],
                    [(tx + 1.0) / 32.0, ty / 32.0],
                ),
                (
                    [
                        right[0] * half + up[0] * half,
                        right[1] * half + up[1] * half,
                        right[2] * half + up[2] * half,
                    ],
                    [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
                ),
                (
                    [
                        -right[0] * half + up[0] * half,
                        -right[1] * half + up[1] * half,
                        -right[2] * half + up[2] * half,
                    ],
                    [tx / 32.0, (ty + 1.0) / 32.0],
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

// ---------------------------------------------------------------------------
// TNT round — primed TNT (the gravity-affected fuse entity, all values
// live-verified 2026-09-22 against reference wiki /TNT §Behavior; see
// also w/Explosion §Damage/§Dropping blocks)
// ---------------------------------------------------------------------------

pub const MAX_PRIMED_TNT: usize = 256;
/// the fuse: fire/redstone-activated TNT explodes after 80 game ticks
/// (4 seconds) — VERIFIED w/TNT §Behavior: "If activated by fire or a
/// redstone signal, or summoned by commands, primed TNT explodes after
/// 80 game ticks (4 seconds)". The timer decreases by 1 every game tick
/// and the primed TNT explodes when it reaches 0 (§Behavior "Countdown
/// timer").
pub const TNT_FUSE_TICKS: i32 = 80;
/// the chain-prime fuse: explosion-activated TNT explodes after a random
/// number of game ticks between 10 and 30 (0.5 to 1.5 s) — VERIFIED
/// w/TNT §Behavior.
pub const TNT_CHAIN_FUSE_MIN: i32 = 10;
pub const TNT_CHAIN_FUSE_MAX: i32 = 30;
/// the primed entity's initial velocity: 0.2 blocks per tick upward and
/// 0.02 blocks per tick in a random direction — VERIFIED w/TNT §Behavior
/// ("given an initial velocity of 0.2 blocks per tick upward, and 0.02
/// blocks per tick in a random direction").
pub const TNT_PRIME_VEL_UP: f32 = 0.2;
pub const TNT_PRIME_VEL_RANDOM: f32 = 0.02;
/// the primed entity's hitbox — VERIFIED w/TNT §Behavior (infobox):
/// "Height: 0.98 blocks, Width: 0.98 blocks". The explosion sits 0.06125
/// blocks above the entity's position (the same verified row).
pub const TNT_HITBOX: f32 = 0.98;
pub const TNT_EXPLOSION_HEIGHT_OFFSET: f32 = 0.06125;
/// the flash: the primed TNT's texture blinks, alternating every 0.5
/// seconds between the TNT block's texture and a near-white brightened
/// copy — VERIFIED w/TNT §Behavior (§Appearance). Engine form: the
/// 10-tick phase at the 20 Hz sim (0.5 s).
pub const TNT_FLASH_PERIOD_TICKS: i32 = 10;
/// the TNT explosion's power — VERIFIED w/TNT §Behavior: "Primed TNT
/// creates explosions with a power of 4, which can break most blocks"
/// (also w/Explosion §Causes: the TNT row is 4, the creeper's row is 3).
pub const TNT_EXPLOSION_POWER: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct PrimedTnt {
    /// the entity's center-bottom position (the block position +
    /// [0.5, +0.0, +0.5] — VERIFIED w/TNT §Behavior)
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// game ticks until the explosion (VERIFIED: decrements by 1 every
    /// tick, explodes at 0)
    pub fuse: i32,
    /// sim ticks alive (drives the 10-tick flash phase)
    pub age: i32,
    /// baked billboard brightness + tint at prime (the item-drop pattern)
    pub light: f32,
    pub tint: [f32; 3],
}

pub struct PrimedTntSystem {
    pub tnts: Vec<PrimedTnt>,
    rng: Rng,
    /// explosions queued for the game layer to drain (world edits + light
    /// + entity damage live there — the creeper-explosion split)
    pub explosions: Vec<[f32; 3]>,
    /// registered TNT block placements (the ignition sweep's scan set —
    /// chain-primed and exploded blocks leave the set)
    blocks: rustc_hash::FxHashSet<[i32; 3]>,
    /// stats
    pub primed_total: u64,
    pub exploded_total: u64,
}

impl PrimedTntSystem {
    pub fn new(seed: u64) -> Self {
        PrimedTntSystem {
            tnts: Vec::with_capacity(16),
            rng: Rng::new(seed ^ 0x7D_0001),
            explosions: Vec::new(),
            blocks: rustc_hash::FxHashSet::default(),
            primed_total: 0,
            exploded_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tnts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tnts.is_empty()
    }

    /// register a placed TNT block for the ignition sweep (the placement
    /// path is the only registrar — the conduits pattern)
    pub fn register_block(&mut self, pos: [i32; 3]) {
        self.blocks.insert(pos);
    }

    /// ONE ignition sweep over the registered TNT placements. Returns the
    /// positions to prime (the caller removes the block, hooks the light,
    /// and spawns the entity with the 80-tick fuse); stale entries (the
    /// block broke or chain-primed since registration) drop from the set.
    ///
    /// VERIFIED w/TNT §Activation (live 2026-09-22): the block can be
    /// activated by "a redstone signal", "Fire spreading onto the TNT
    /// block", and "Other explosions" — plus flint-and-steel/fire charge,
    /// flaming projectiles, and dispensers (all trimmed — see below).
    /// Engine forms:
    /// * redstone — the engine's verified power-source neighborhood scan
    ///   (the same set the redstone lamp uses: a lit torch, an ON lever,
    ///   powered wire); TNT is a redstone mechanism component (VERIFIED
    ///   w/TNT §Redstone component)
    /// * fire/lava — a FIRE/SOUL_FIRE/LAVA block at or adjacent to the
    ///   TNT cell ignites it (VERIFIED w/TNT §infobox: "Catches fire from
    ///   lava Yes"; §Activation: "Fire spreading onto the TNT block").
    ///   DOCUMENTED TRIM: the engine has no fire-spread mechanic (fire
    ///   never spreads onto the block), so the vanilla "the block burns
    ///   for several seconds before activating" window collapses to
    ///   contact = activation
    /// * explosions — game.rs's explode() chain-primes the TNT in its
    ///   radius (never this sweep). TNT is NOT redstone-conductive:
    ///   "When a TNT block receives a redstone signal, it does not
    ///   activate any other adjacent TNT blocks via redstone, but any
    ///   adjacent TNT blocks are activated by the explosion" (VERIFIED
    ///   w/TNT §Redstone component)
    /// * DOCUMENTED TRIMS (the engine has no such systems — w/TNT
    ///   §Activation rows): flint-and-steel/fire charge (no such item),
    ///   dispenser placement + dispenser flint-and-steel use (the engine's
    ///   dispenser ejects items only), flaming projectiles (no Flame
    ///   enchant; burning projectiles never hit TNT), commands, and the
    ///   `unstable` blockstate (the engine's punch path breaks and drops
    ///   like the vanilla default state, §Block states).
    pub fn ignition_sweep(
        &mut self,
        world: &vc_world::world::World,
        scope_center: (i32, i32),
        scope_radius: i32,
    ) -> Vec<[i32; 3]> {
        use crate::redstone::powered_by_neighbors;
        use vc_blocks::blocks::{state_block, FIRE, LAVA, SOUL_FIRE, TNT};
        let mut to_prime = Vec::new();
        let registered: Vec<[i32; 3]> = self.blocks.iter().copied().collect();
        for pos in registered {
            // stale: the block broke (or chain-primed) since registration
            if world.get_block(pos[0], pos[1], pos[2]) != TNT {
                self.blocks.remove(&pos);
                continue;
            }
            // Phase 6 §26: the sweep freezes outside the simulation ring
            let in_ring = pos[0]
                .div_euclid(16)
                .wrapping_sub(scope_center.0)
                .saturating_abs()
                .max(
                    pos[2]
                        .div_euclid(16)
                        .wrapping_sub(scope_center.1)
                        .saturating_abs(),
                )
                <= scope_radius;
            if !in_ring {
                continue;
            }
            // redstone: the mechanism component's power check (the lamp's
            // verified source set)
            if powered_by_neighbors(world, pos[0], pos[1], pos[2]) {
                to_prime.push(pos);
                continue;
            }
            // fire/lava contact: at the cell or any of the 6 neighbors
            let mut burning = false;
            for (dx, dy, dz) in [
                (0i32, 0i32, 0i32),
                (1i32, 0i32, 0i32),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ] {
                let b = state_block(world.get_state(pos[0] + dx, pos[1] + dy, pos[2] + dz));
                if b == FIRE || b == SOUL_FIRE || b == LAVA {
                    burning = true;
                    break;
                }
            }
            if burning {
                to_prime.push(pos);
            }
        }
        for pos in &to_prime {
            self.blocks.remove(pos);
        }
        to_prime
    }

    /// ONE sim tick for all primed TNT entities: the fuse countdown (1
    /// per tick, explodes at 0 — VERIFIED), gravity 0.04 + drag 0.98 (the
    /// verified shared entity profile the items/falling blocks carry),
    /// per-axis collision probing the 0.98 hitbox's extremes, and the
    /// explosion at fuse 0 queued at pos + 0.06125 above (VERIFIED) for
    /// game.rs to drain via explode().
    ///
    /// DOCUMENTED ADAPTATION (water): vanilla primed TNT is "pushed by
    /// flowing water" and does not float (no buoyancy); the engine's
    /// shared water drag 0.9 applies — the exact vanilla water drag for
    /// the primed entity was not separately verified.
    pub fn tick(&mut self, world: &vc_world::world::World) {
        let half = TNT_HITBOX * 0.5;
        let mut i = 0;
        while i < self.tnts.len() {
            // the fuse countdown (VERIFIED: 1 per tick, explode at 0)
            self.tnts[i].age += 1;
            self.tnts[i].fuse -= 1;
            if self.tnts[i].fuse <= 0 {
                // the explosion location: 0.06125 blocks above the
                // entity's position (VERIFIED w/TNT §Behavior)
                let p = self.tnts[i].pos;
                self.explosions
                    .push([p[0], p[1] + TNT_EXPLOSION_HEIGHT_OFFSET, p[2]]);
                self.exploded_total += 1;
                self.tnts.remove(i);
                continue;
            }
            // physics: gravity 0.04 (the shared entity profile; no
            // buoyancy — the documented water adaptation above)
            let t = &mut self.tnts[i];
            t.vel[1] -= 0.04;
            // per-axis move; the collision probes the 0.98 hitbox's
            // extremes along the moved axis. pos is BOTTOM-anchored on Y
            // (the render centers the cuboid at pos + half) and CENTERED
            // on X/Z — the Y probe spans [target, target + 0.98], the
            // horizontal probes [target − 0.49, target + 0.49] (the
            // 2026-10-02 CI catch: the old ±half Y probe floated the
            // entity ~0.5 above the floor)
            for axis in 0..3 {
                let target = t.pos[axis] + t.vel[axis];
                let (alo, ahi) = if axis == 1 {
                    (target, target + TNT_HITBOX)
                } else {
                    (target - half, target + half)
                };
                let mut lo = t.pos;
                let mut hi = t.pos;
                lo[axis] = alo;
                hi[axis] = ahi;
                let hit = is_solid(world.get_block(lo[0] as i32, lo[1] as i32, lo[2] as i32))
                    || is_solid(world.get_block(hi[0] as i32, hi[1] as i32, hi[2] as i32));
                if hit {
                    if axis == 1 {
                        t.vel[1] = 0.0;
                        // ground friction (the item pattern's slip)
                        t.vel[0] *= 0.6;
                        t.vel[2] *= 0.6;
                    } else {
                        t.vel[axis] = 0.0;
                    }
                } else {
                    t.pos[axis] = target;
                }
            }
            // drag: 0.98 in air (the shared profile), 0.9 in water (the
            // documented adaptation)
            let in_water =
                world.get_block(t.pos[0] as i32, t.pos[1] as i32, t.pos[2] as i32) == WATER;
            let drag = if in_water { 0.9 } else { 0.98 };
            t.vel[0] *= drag;
            t.vel[2] *= drag;
            if in_water {
                t.vel[1] *= 0.9;
            } else {
                t.vel[1] *= 0.98;
            }
            i += 1;
        }
    }

    /// prime a TNT block: the block is replaced with the primed entity
    /// placed offset from the block's bottom center by [+0.5, +0.0, +0.5]
    /// (VERIFIED w/TNT §Behavior), given the initial velocity (0.2 up +
    /// 0.02 random — VERIFIED), and the fuse (80 for fire/redstone —
    /// VERIFIED; the caller passes the random 10–30 chain-prime fuse).
    /// `biome`/`sky`/`blk` bake the entity's brightness + tint at prime
    /// (the item-drop pattern; TNT has no biome tint).
    #[allow(clippy::too_many_arguments)]
    pub fn prime(&mut self, wx: i32, wy: i32, wz: i32, biome: u8, sky: u8, blk: u8, fuse: i32) {
        if self.tnts.len() >= MAX_PRIMED_TNT {
            return; // cap: the oldest entities still live, refuse the prime
        }
        let tint = vc_blocks::tint::block_tint_color(TNT, biome);
        let s = sky.min(15) as f32 / 15.0;
        let b = blk.min(15) as f32 / 15.0;
        let light = (s.max(b) * 0.96 + 0.04) * s.max(b).powf(1.2);
        let ang = self.rng.next_f32() * std::f32::consts::TAU;
        self.tnts.push(PrimedTnt {
            pos: [wx as f32 + 0.5, wy as f32, wz as f32 + 0.5],
            vel: [
                ang.sin() * TNT_PRIME_VEL_RANDOM,
                TNT_PRIME_VEL_UP,
                ang.cos() * TNT_PRIME_VEL_RANDOM,
            ],
            fuse,
            age: 0,
            light,
            tint,
        });
        self.primed_total += 1;
        self.blocks.remove(&[wx, wy, wz]);
    }

    /// the chain-prime fuse roll: a random number of game ticks between
    /// 10 and 30 inclusive (VERIFIED w/TNT §Behavior)
    pub fn chain_fuse(&mut self) -> i32 {
        TNT_CHAIN_FUSE_MIN
            + self
                .rng
                .next_range((TNT_CHAIN_FUSE_MAX - TNT_CHAIN_FUSE_MIN + 1) as u32)
                as i32
    }

    /// the 10-tick flash phase (VERIFIED: "blinks, alternating every 0.5
    /// seconds between the TNT block's texture, and a copy of it that has
    /// been brightened to near-white")
    pub fn flash_bright(&self, t: &PrimedTnt) -> f32 {
        // age is incremented BEFORE the phase read (tick's first sample is
        // age 1), so the phase folds the COMPLETED ticks: dark ages 1..10
        // (the first 0.5 s), bright 11..20, dark 21..30, bright 31..40 —
        // the verified "alternating every 0.5 seconds" from priming
        if ((t.age - 1) / TNT_FLASH_PERIOD_TICKS) % 2 == 1 {
            1.0 // the near-white brightened copy
        } else {
            0.0 // the TNT block's texture
        }
    }

    /// block-model rendering: a 0.98-block cuboid (VERIFIED hitbox) with
    /// the TNT faces (state_tiles), vanilla face shading (top 1.0 /
    /// bottom 0.5 / X 0.6 / Z 0.8) — no spin (the primed entity renders
    /// as the block model, not a spinning item). The flash phase
    /// brightens every face toward near-white on the bright phase.
    /// Emitted into the shared particle stream like the item cubes.
    pub fn build_vertices(&self, out: &mut Vec<vc_particles::particles::ParticleVertex>) {
        let half = TNT_HITBOX * 0.5;
        for t in self.tnts.iter() {
            let tiles = state_tiles(TNT);
            let flash = self.flash_bright(t);
            let cy = t.pos[1] + half; // the hitbox is bottom-anchored
            let base = [
                t.light * t.tint[0],
                t.light * t.tint[1],
                t.light * t.tint[2],
            ];
            // the near-white brightened copy (the flash's bright phase)
            let col = [
                base[0] + (1.0 - base[0]) * flash,
                base[1] + (1.0 - base[1]) * flash,
                base[2] + (1.0 - base[2]) * flash,
            ];
            // (corners CCW seen from outside, normal, shade, tile)
            let faces: [CuboidFace; 6] = [
                // +Y top
                (
                    [
                        [-half, half, -half],
                        [half, half, -half],
                        [half, half, half],
                        [-half, half, half],
                    ],
                    [0.0, 1.0, 0.0],
                    1.0,
                    tiles[0],
                ),
                // −Y bottom
                (
                    [
                        [-half, -half, half],
                        [half, -half, half],
                        [half, -half, -half],
                        [-half, -half, -half],
                    ],
                    [0.0, -1.0, 0.0],
                    0.5,
                    tiles[1],
                ),
                // +X
                (
                    [
                        [half, -half, -half],
                        [half, half, -half],
                        [half, half, half],
                        [half, -half, half],
                    ],
                    [1.0, 0.0, 0.0],
                    0.6,
                    tiles[2],
                ),
                // −X
                (
                    [
                        [-half, -half, half],
                        [-half, half, half],
                        [-half, half, -half],
                        [-half, -half, -half],
                    ],
                    [-1.0, 0.0, 0.0],
                    0.6,
                    tiles[2],
                ),
                // +Z
                (
                    [
                        [half, -half, half],
                        [half, half, half],
                        [-half, half, half],
                        [-half, -half, half],
                    ],
                    [0.0, 0.0, 1.0],
                    0.8,
                    tiles[3],
                ),
                // −Z
                (
                    [
                        [-half, -half, -half],
                        [-half, half, -half],
                        [half, half, -half],
                        [-half, -half, -half],
                    ],
                    [0.0, 0.0, -1.0],
                    0.8,
                    tiles[3],
                ),
            ];
            for (fi, (corners, _nrm, shade, tile)) in faces.iter().enumerate() {
                // [1.12 fix] 32-tile atlas rows (was %16//16)
                let tx = (*tile % 32) as f32;
                let ty = (*tile / 32) as f32;
                // per-face shade on the flash color (the top keeps 1.0)
                let fc = [col[0] * shade, col[1] * shade, col[2] * shade];
                // UV: v flipped so texture top = block top (side faces);
                // top/bottom map the tile straight on
                let uvs = if fi < 2 {
                    [
                        [tx / 32.0, (ty + 1.0) / 32.0],
                        [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
                        [(tx + 1.0) / 32.0, ty / 32.0],
                        [tx / 32.0, ty / 32.0],
                    ]
                } else {
                    [
                        [tx / 32.0, (ty + 1.0) / 32.0],
                        [tx / 32.0, ty / 32.0],
                        [(tx + 1.0) / 32.0, ty / 32.0],
                        [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
                    ]
                };
                for ci in [0usize, 1, 2, 0, 2, 3] {
                    let cn = corners[ci];
                    out.push(vc_particles::particles::ParticleVertex {
                        pos: [t.pos[0] + cn[0], cy + cn[1], t.pos[2] + cn[2]],
                        uv: [uvs[ci][0], uvs[ci][1]],
                        col: fc,
                    });
                }
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
        // before the delay: no pickup — the player stands right on the
        // item's cell (feet-anchored box semantics, 2026-09-21)
        for _ in 0..5 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        assert!(
            is.collect([0.5, 65.001, 0.5]).is_empty(),
            "pickup delay guards"
        );
        // after the delay: collected — the item RESTS AT THE FEET, the
        // exact case the old eye-radius test could never cover honestly
        // (it fabricated an "eye" at item height to pass)
        for _ in 0..10 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        let got = is.collect([0.5, 65.001, 0.5]);
        assert_eq!(got, vec![DIRT]);
        assert_eq!(is.len(), 0);
        assert_eq!(is.picked_total, 1);
        // far away: no pickup — 4 blocks off horizontally is outside
        // the 1.3 half-width inflated AABB
        is.drop_block(4, 66, 4, STONE, 2, 15, 0);
        for _ in 0..20 {
            is.tick(&w, (0, 0), i32::MAX);
        }
        assert!(is.collect([0.5, 65.001, 0.5]).is_empty(), "distance guards");
        assert_eq!(is.len(), 1);
    }

    #[test]
    fn item_vertices_are_a_3d_cuboid() {
        let mut is = ItemSystem::new(5);
        is.drop_block(0, 70, 0, GRASS, 3, 15, 0);
        let mut out = Vec::new();
        is.build_vertices(
            1.0,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, -1.0],
            &mut out,
        );
        // 2026-09-21: a full painter-ordered cuboid — 6 faces × 6
        // verts — replaces the old single billboard quad ("not the
        // item in 3D moving like the real game")
        assert_eq!(out.len(), 36, "six faces per item cuboid");
        // tint baked: Forest grass
        let c = out[0].col;
        assert!(c[1] > c[0], "green-dominant: {c:?}");
        // UVs inside the atlas (any valid atlas UV)
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
        assert_eq!(
            split_xp(12000).iter().sum::<i32>(),
            12000,
            "dragon XP preserved"
        );
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

    // ---------------- TNT round: primed TNT ----------------

    #[test]
    fn tnt_fuse_is_80_ticks_and_explodes_at_zero() {
        // VERIFIED w/TNT §Behavior: "primed TNT explodes after 80 game
        // ticks (4 seconds)"; "The timer decreases by 1 every game tick,
        // and the Primed TNT explodes when it reaches 0"
        let w = flat_world();
        let mut sys = PrimedTntSystem::new(3);
        sys.prime(0, 66, 0, 2, 15, 0, TNT_FUSE_TICKS);
        assert_eq!(sys.len(), 1);
        assert_eq!(sys.tnts[0].pos, [0.5, 66.0, 0.5], "block pos +[0.5,0,0.5]");
        assert_eq!(
            sys.tnts[0].vel[1], TNT_PRIME_VEL_UP,
            "0.2 blocks/tick upward (VERIFIED)"
        );
        assert!(
            sys.tnts[0].vel[0].hypot(sys.tnts[0].vel[2]) <= TNT_PRIME_VEL_RANDOM + 1e-6,
            "0.02 blocks/tick random horizontal (VERIFIED)"
        );
        // no explosion before the fuse
        for _ in 0..TNT_FUSE_TICKS - 1 {
            sys.tick(&w);
            assert!(sys.explosions.is_empty(), "the fuse guards");
            assert_eq!(sys.len(), 1);
        }
        // the 80th tick drains the fuse to 0 → the explosion queues
        sys.tick(&w);
        assert_eq!(sys.explosions.len(), 1, "explodes at fuse 0");
        assert_eq!(sys.exploded_total, 1);
        assert!(sys.tnts.is_empty(), "the entity is consumed");
        let c = sys.explosions[0];
        // the entity fell and rests on the y=64 stone's top surface
        // (~65.0 + the one-tick residual) — the explosion fires
        // 0.06125 above the RESTED position (the rest-height
        // tolerance covers the residual)
        assert!(
            (c[1] - (65.0 + TNT_EXPLOSION_HEIGHT_OFFSET)).abs() < 0.06,
            "0.06125 above the rested entity position, got {}",
            c[1]
        );
    }

    #[test]
    fn tnt_falls_with_the_shared_entity_profile() {
        // gravity 0.04, drag 0.98 (the verified shared profile the
        // items/falling blocks carry) — the entity rests on the floor
        let w = flat_world();
        let mut sys = PrimedTntSystem::new(5);
        sys.prime(0, 66, 0, 2, 15, 0, 400);
        for _ in 0..60 {
            sys.tick(&w);
        }
        assert!(!sys.tnts.is_empty(), "a long fuse still lives");
        let t = &sys.tnts[0];
        // the 0.98 hitbox rests on the y=64 floor's top surface (y=65)
        assert!((t.pos[1] - 65.0).abs() < 0.06, "rest height: {}", t.pos[1]);
    }

    #[test]
    fn tnt_chain_fuse_rolls_between_10_and_30() {
        // VERIFIED w/TNT §Behavior: "If activated by an explosion, primed
        // TNT explodes after a random number of game ticks between 10
        // and 30 (0.5 to 1.5 seconds)"
        let mut sys = PrimedTntSystem::new(7);
        for _ in 0..400 {
            let f = sys.chain_fuse();
            assert!(
                (TNT_CHAIN_FUSE_MIN..=TNT_CHAIN_FUSE_MAX).contains(&f),
                "fuse {f} outside the verified 10–30 window"
            );
        }
        // the roll is not pinned to one value: many seeds cover the
        // window (the old hand-picked f·7919 seeds assumed the seed
        // maps to its own fuse — brittle)
        let mut seen: Vec<i32> = Vec::new();
        for s in 0..200u64 {
            let mut sys = PrimedTntSystem::new(s);
            let f = sys.chain_fuse();
            if !seen.contains(&f) {
                seen.push(f);
            }
        }
        assert!(
            seen.len() >= 2,
            "the roll covers the window ({} distinct of 200 seeds)",
            seen.len()
        );
        assert_eq!(TNT_FLASH_PERIOD_TICKS, 10, "0.5 s at 20 Hz");
        assert_eq!(TNT_HITBOX, 0.98);
        assert_eq!(TNT_EXPLOSION_POWER, 4.0);
    }

    #[test]
    fn tnt_flash_phase_alternates_every_10_ticks() {
        // VERIFIED w/TNT §Behavior (§Appearance): "blinks, alternating
        // every 0.5 seconds between the TNT block's texture, and a copy
        // of it that has been brightened to near-white"
        let w = flat_world();
        let mut sys = PrimedTntSystem::new(9);
        sys.prime(0, 66, 0, 2, 15, 0, TNT_FUSE_TICKS);
        let mut phases = Vec::new();
        for _ in 0..40 {
            sys.tick(&w);
            if let Some(t) = sys.tnts.first() {
                phases.push(sys.flash_bright(t));
            }
        }
        assert_eq!(phases.len(), 40);
        // 0.5 s dark / 0.5 s bright: phase flips at ticks 10 and 30
        assert_eq!(phases[0], 0.0, "the first 0.5 s is the block texture");
        assert_eq!(phases[9], 0.0);
        assert_eq!(phases[10], 1.0, "the near-white copy");
        assert_eq!(phases[19], 1.0);
        assert_eq!(phases[20], 0.0);
        assert_eq!(phases[30], 1.0);
        // the flash renders as near-white vertices (VERIFIED w/TNT
        // §Appearance: "brightened to near-white")
        let mut out = Vec::new();
        sys.build_vertices(&mut out);
        assert_eq!(out.len(), 36, "six faces per entity cuboid");
        for v in &out {
            assert!(v.col[0] >= 0.9, "near-white brightened: {:?}", v.col);
        }
    }

    #[test]
    fn tnt_power_4_versus_the_creeper_3() {
        // VERIFIED w/TNT §Behavior: "Primed TNT creates explosions with
        // a power of 4"; w/Explosion §Causes: the TNT row is 4, the
        // creeper's row is 3 (the engine's mobs.rs constant)
        assert_eq!(TNT_EXPLOSION_POWER, 4.0);
        assert_eq!(vc_gameplay::mobs::CREEPER_POWER, 3.0);
        assert_eq!(
            TNT_EXPLOSION_POWER as i32,
            vc_gameplay::mobs::CREEPER_POWER as i32 + 1,
            "TNT outranges the creeper by one power step"
        );
        // the charged creeper's 6 outranges TNT (the End-crystal class)
        assert_eq!(vc_gameplay::mobs::CHARGED_CREEPER_POWER, 6.0);
        assert!(vc_gameplay::mobs::CHARGED_CREEPER_POWER > TNT_EXPLOSION_POWER);
    }

    #[test]
    fn tnt_ignites_by_redstone_power() {
        // VERIFIED w/TNT §Activation: "A redstone signal" — the block is
        // a redstone mechanism component (§Redstone component); the
        // engine's verified power-source set (the lamp's scan)
        let mut w = flat_world();
        let mut sys = PrimedTntSystem::new(11);
        // the TNT block sits on the floor (the dedicated state, never the
        // colliding identity 533)
        w.set_block_state(0, 65, 0, default_state(TNT));
        sys.register_block([0, 65, 0]);
        // no power, no fire: no ignition
        let p = sys.ignition_sweep(&w, (0, 0), i32::MAX);
        assert!(p.is_empty(), "an unpowered, unlit TNT stays");
        assert_eq!(sys.len(), 0, "the sweep reports; the caller primes");
        // a powered wire adjacent → ignition (the mechanism component)
        w.set_block_state(1, 65, 0, wire_state(15));
        let p = sys.ignition_sweep(&w, (0, 0), i32::MAX);
        assert_eq!(p, vec![[0, 65, 0]], "redstone ignition");
        assert!(sys.blocks.is_empty(), "primed blocks leave the set");
        // an unpowered wire (power 0) does NOT ignite
        sys.register_block([0, 65, 0]);
        w.set_block_state(1, 65, 0, wire_state(0));
        assert!(
            sys.ignition_sweep(&w, (0, 0), i32::MAX).is_empty(),
            "power 0 is not a signal"
        );
        // an ON lever adjacent → ignition
        w.set_block_state(1, 65, 0, lever_state(true));
        assert_eq!(
            sys.ignition_sweep(&w, (0, 0), i32::MAX),
            vec![[0, 65, 0]],
            "lever ignition"
        );
    }

    #[test]
    fn tnt_ignites_by_fire_and_lava_contact() {
        // VERIFIED w/TNT §infobox: "Catches fire from lava | Yes";
        // §Activation: "Fire spreading onto the TNT block" (the engine's
        // contact = activation form — the no-fire-spread trim)
        let mut w = flat_world();
        let mut sys = PrimedTntSystem::new(13);
        w.set_block_state(0, 65, 0, default_state(TNT));
        sys.register_block([0, 65, 0]);
        // fire above the TNT cell
        w.set_block_state(0, 66, 0, default_state(FIRE));
        assert_eq!(
            sys.ignition_sweep(&w, (0, 0), i32::MAX),
            vec![[0, 65, 0]],
            "fire contact ignition"
        );
        // lava adjacent (the side neighbor)
        sys.register_block([0, 65, 0]);
        w.set_block_state(0, 66, 0, default_state(LAVA));
        assert_eq!(
            sys.ignition_sweep(&w, (0, 0), i32::MAX),
            vec![[0, 65, 0]],
            "lava contact ignition"
        );
    }

    #[test]
    fn tnt_sweep_drops_stale_registrations() {
        // the block broke since registration → the entry drops and never
        // reports a phantom prime
        let mut w = flat_world();
        let mut sys = PrimedTntSystem::new(17);
        w.set_block_state(0, 65, 0, default_state(TNT));
        sys.register_block([0, 65, 0]);
        // the block is removed (broken by the player)
        w.set_block(0, 65, 0, AIR);
        let p = sys.ignition_sweep(&w, (0, 0), i32::MAX);
        assert!(p.is_empty(), "no phantom prime for a stale entry");
        assert!(sys.blocks.is_empty(), "the stale entry drops");
    }
}
