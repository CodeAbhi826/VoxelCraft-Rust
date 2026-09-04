//! Mobs (master prompt Phase 2): a first batch of 9 entities — 5 hostile
//! (zombie, skeleton, creeper, spider, enderman) + 4 passive (cow, pig,
//! sheep, chicken). The remaining ~90 entities of the 1.16.5 registry are
//! explicitly deferred (see DEFERRED_ENTITIES).
//!
//! VERIFIED data (minecraft.wiki, pulled 2026-09-04 per the verification
//! discipline — NOT from dossier memory):
//! - per-mob health / speed attribute / damage rows (infobox "Health
//!   points", "Speed", "Attack strength") — see MOB_DATA
//! - hostile spawning: block light ≤ 7 in 1.16.5 (the 1.18 experimental
//!   snapshot changed it to 0 — wiki History section)
//! - passive spawning: light ≥ 9, on grass blocks, with 2 blocks of space
//! - mob caps: Monster 70 / Creature 10 / Ambient 15, scaled
//!   `cap × chunks ÷ 289` over the 17×17-chunk spawn square
//! - despawn: >128 blocks from the nearest player is instant; 32 blocks
//!   with no player for 30 s rolls 1/800 despawn per game tick
//! - creeper: explosion power 3, 1.5 s fuse (30 game ticks)
//!
//! Documented adaptations:
//! - speed: attributes converted at ~10.5 blocks/s per point (observed-
//!   equivalent chase speeds; vanilla's per-tick velocity integration with
//!   drag has no published closed form — flagged, not exact)
//! - pathfinding is straight-line steering + 1-block step-ups (the
//!   existing villager primitive), no A*
//! - XP granted directly on kill (no XP orbs, same as our ore mining);
//!   5 XP for standard hostiles is a [placeholder: common long-standing
//!   value, not re-verified this pass]
//! - arrows: ballistic points, gravity 20 b/s² (vanilla 0.05/tick²),
//!   skeleton cadence fixed at 2 s

use vc_blocks::blocks::*;
use vc_rng::rng::Rng;
use vc_world::world::World;

pub const MAX_MOBS: usize = 128;

/// Mob kinds in this first batch. The full 1.16.5 registry (102 mob-like
/// entities per Dossier Part 4 §21) is deliberately NOT attempted at once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MobKind {
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Enderman,
    Cow,
    Pig,
    Sheep,
    Chicken,
}

impl MobKind {
    pub fn from_name(s: &str) -> Option<MobKind> {
        Some(match s {
            "zombie" => MobKind::Zombie,
            "skeleton" => MobKind::Skeleton,
            "creeper" => MobKind::Creeper,
            "spider" => MobKind::Spider,
            "enderman" => MobKind::Enderman,
            "cow" => MobKind::Cow,
            "pig" => MobKind::Pig,
            "sheep" => MobKind::Sheep,
            "chicken" => MobKind::Chicken,
            _ => return None,
        })
    }

    /// Entity-type registry name (vanilla strings, mechanical data).
    pub fn name(self) -> &'static str {
        match self {
            MobKind::Zombie => "minecraft:zombie",
            MobKind::Skeleton => "minecraft:skeleton",
            MobKind::Creeper => "minecraft:creeper",
            MobKind::Spider => "minecraft:spider",
            MobKind::Enderman => "minecraft:enderman",
            MobKind::Cow => "minecraft:cow",
            MobKind::Pig => "minecraft:pig",
            MobKind::Sheep => "minecraft:sheep",
            MobKind::Chicken => "minecraft:chicken",
        }
    }

    pub fn sprite_tile(self) -> u16 {
        match self {
            MobKind::Zombie => TILE_ZOMBIE,
            MobKind::Skeleton => TILE_SKELETON,
            MobKind::Creeper => TILE_CREEPER,
            MobKind::Spider => TILE_SPIDER,
            MobKind::Enderman => TILE_ENDERMAN,
            MobKind::Cow => TILE_COW,
            MobKind::Pig => TILE_PIG,
            MobKind::Sheep => TILE_SHEEP,
            MobKind::Chicken => TILE_CHICKEN,
        }
    }

    /// attacks on sight (zombie/skeleton/creeper/spider; enderman is
    /// neutral until provoked)
    pub fn hostile(self) -> bool {
        matches!(
            self,
            MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper | MobKind::Spider
        )
    }

    pub fn neutral(self) -> bool {
        self == MobKind::Enderman
    }
}

/// Per-mob stats. VERIFIED against the wiki infoboxes (2026-09-04):
/// health, damage = the NORMAL-difficulty row, speed = movement-speed
/// attribute, armor = natural armor points, height/width = hitbox.
pub struct MobDef {
    pub kind: MobKind,
    pub health: f32,
    /// NORMAL-difficulty damage (Easy/Hard scale via combat::difficulty_scale)
    pub damage: f32,
    /// movement-speed ATTRIBUTE (wiki "Speed" row); converted at ×10.5
    pub speed_attr: f32,
    /// natural armor points (zombie has 2 — verified)
    pub armor: f32,
    /// hitbox height in blocks (sprite render height follows)
    pub height: f32,
    /// hitbox width in blocks
    pub width: f32,
    /// XP granted when killed by the player [placeholder: see header]
    pub xp: i32,
}

pub const MOB_DATA: [MobDef; 9] = [
    MobDef { kind: MobKind::Zombie, health: 20.0, damage: 3.0, speed_attr: 0.23, armor: 2.0, height: 1.95, width: 0.6, xp: 5 },
    MobDef { kind: MobKind::Skeleton, health: 20.0, damage: 4.0, speed_attr: 0.25, armor: 0.0, height: 1.99, width: 0.6, xp: 5 }, // dmg = mid of arrow 3–5
    MobDef { kind: MobKind::Creeper, health: 20.0, damage: 0.0, speed_attr: 0.25, armor: 0.0, height: 1.7, width: 0.6, xp: 5 },
    MobDef { kind: MobKind::Spider, health: 16.0, damage: 2.0, speed_attr: 0.3, armor: 0.0, height: 0.9, width: 1.4, xp: 5 },
    MobDef { kind: MobKind::Enderman, health: 40.0, damage: 7.0, speed_attr: 0.3, armor: 0.0, height: 2.9, width: 0.6, xp: 5 },
    MobDef { kind: MobKind::Cow, health: 10.0, damage: 0.0, speed_attr: 0.2, armor: 0.0, height: 1.4, width: 0.9, xp: 1 },
    MobDef { kind: MobKind::Pig, health: 10.0, damage: 0.0, speed_attr: 0.25, armor: 0.0, height: 0.9, width: 0.9, xp: 1 },
    MobDef { kind: MobKind::Sheep, health: 8.0, damage: 0.0, speed_attr: 0.23, armor: 0.0, height: 1.3, width: 0.9, xp: 1 },
    MobDef { kind: MobKind::Chicken, health: 4.0, damage: 0.0, speed_attr: 0.25, armor: 0.0, height: 0.7, width: 0.4, xp: 1 },
];

#[inline]
pub fn def(kind: MobKind) -> &'static MobDef {
    MOB_DATA.iter().find(|d| d.kind == kind).unwrap()
}

/// Deferred entities (explicit): every 1.16.5 mob NOT in this batch —
/// drowned/husk/stray/zombie-villager/cave-spider/slime/magma-cube/ghast/
/// piglin-family/blaze/wither-skeleton/guardians/shulker/phantom/
/// silverfish/illager-family/vex/witch, and the ~40 remaining passives
/// (horse, rabbit, wolf, fox, bee, turtle, …). They arrive in follow-up
/// batches once the core loop is proven.
pub const DEFERRED_ENTITIES: &str =
    "all 1.16.5 mobs except zombie/skeleton/creeper/spider/enderman/cow/pig/sheep/chicken";

// ------------------------------------------------------- verified rules --
pub const DESPAWN_INSTANT_BLOCKS: f32 = 128.0;
pub const DESPAWN_NEAR_BLOCKS: f32 = 32.0;
/// hostile mob cap constant (wiki mob-cap table: Monster 70)
pub const MONSTER_CAP: f32 = 70.0;
/// passive/creature cap constant (Creature 10)
pub const CREATURE_CAP: f32 = 10.0;
/// spawn square: 17×17 chunks → 289 (cap scale divisor — wiki formula)
pub const CAP_DIVISOR: f32 = 289.0;
/// hostile light ceiling for 1.16.5 (block light ≤ 7)
pub const HOSTILE_LIGHT_MAX: u8 = 7;
/// passive light floor (animals need ≥ 9 — wiki)
pub const PASSIVE_LIGHT_MIN: u8 = 9;
/// hostile sky-light ceiling (1.16.5 overworld)
pub const HOSTILE_SKY_MAX: u8 = 7;
/// creeper: begins the fuse this close (vanilla ~3 blocks)
pub const CREEPER_FUSE_DIST: f32 = 3.0;
/// creeper fuse: 30 game ticks = 1.5 s (vanilla)
pub const CREEPER_FUSE_TICKS: i32 = 30;
/// creeper explosion power (wiki: "Normal creeper explosions have a power of 3")
pub const CREEPER_POWER: f32 = 3.0;
/// skeleton bow interval (adaptation: fixed 40-tick cadence)
pub const SKELETON_SHOOT_TICKS: i32 = 40;
/// mob melee reach
pub const MOB_MELEE_REACH: f32 = 1.6;
/// mob melee cooldown, game ticks (~1 s zombie cadence)
pub const MOB_MELEE_TICKS: i32 = 20;
/// aggro radius
pub const AGGRO_RADIUS: f32 = 16.0;
/// passive panic flee multiplier
pub const FLEE_MULT: f32 = 1.8;
/// attribute → blocks/s conversion (documented adaptation)
pub const SPEED_PER_ATTR: f32 = 10.5;

/// One mob instance. Position is feet-center like the player.
#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u32,
    pub kind: MobKind,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub yaw: f32,
    pub health: f32,
    pub on_ground: bool,
    /// hurt flash (ticks remaining) — red tint during rendering
    pub hurt_t: i32,
    /// melee/ranged attack cooldown (ticks)
    pub attack_cd: i32,
    /// creeper fuse: <0 idle, 0..=30 counting, i32::MAX consumed
    pub fuse: i32,
    /// provoked (neutral mobs become hostile)
    pub provoked: bool,
    /// ticks since a player was within 32 blocks
    pub lonely_t: i32,
    wander_yaw: f32,
    wander_t: i32,
}

/// A mob's damage event delivered to the player (game layer applies mode
/// gating + difficulty scaling).
#[derive(Clone, Debug)]
pub struct PlayerHit {
    /// NORMAL-difficulty damage — scale via combat::difficulty_scale
    pub damage: f32,
    pub source: MobKind,
    pub knockback_dir: [f32; 2],
}

/// An arrow projectile (skeleton): ballistic point.
#[derive(Clone, Debug)]
pub struct Arrow {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// damage on hit (Normal 3–5, chosen at fire time)
    pub damage: f32,
    pub age: i32,
}

pub struct MobSystem {
    pub list: Vec<Mob>,
    pub arrows: Vec<Arrow>,
    rng: Rng,
    next_id: u32,
    /// player anchor for spawning/AI (None = no spawns, AI idles)
    pub player: Option<[f32; 3]>,
    /// creative flight / invulnerability — mobs hold fire
    pub player_invulnerable: bool,
    /// queued hits on the player (drained each frame by game.rs)
    pub hits: Vec<PlayerHit>,
    /// mob deaths (drops + XP handled by the game layer)
    pub deaths: Vec<(MobKind, [f32; 3])>,
    /// explosion requests (center, power) — game.rs owns world edits so
    /// the light engine updates ride along
    pub explosions: Vec<([f32; 3], f32)>,
    /// counters for F3/E2E
    pub spawned_total: u64,
    pub despawned_total: u64,
    pub killed_total: u64,
}

impl MobSystem {
    pub fn new(seed: u64) -> Self {
        MobSystem {
            list: Vec::new(),
            arrows: Vec::new(),
            rng: Rng::new(seed ^ 0xB0B_5EED),
            next_id: 1,
            player: None,
            player_invulnerable: false,
            hits: Vec::new(),
            deaths: Vec::new(),
            explosions: Vec::new(),
            spawned_total: 0,
            despawned_total: 0,
            killed_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn hostiles_alive(&self) -> usize {
        self.list.iter().filter(|m| m.kind.hostile()).count()
    }

    fn passives_alive(&self) -> usize {
        self.list.iter().filter(|m| !m.kind.hostile()).count()
    }

    /// Spawn a specific mob at a block position (E2E/structures).
    pub fn spawn_at(&mut self, kind: MobKind, x: i32, y: i32, z: i32) -> Option<u32> {
        if self.list.len() >= MAX_MOBS {
            return None;
        }
        let d = def(kind);
        let id = self.next_id;
        self.next_id += 1;
        let yaw = self.rng.next_f32() * std::f32::consts::TAU;
        self.list.push(Mob {
            id,
            kind,
            pos: [x as f32 + 0.5, y as f32, z as f32 + 0.5],
            vel: [0.0; 3],
            yaw,
            health: d.health,
            on_ground: false,
            hurt_t: 0,
            attack_cd: 0,
            fuse: -1,
            provoked: false,
            lonely_t: 0,
            wander_yaw: yaw,
            wander_t: 0,
        });
        self.spawned_total += 1;
        Some(id)
    }

    /// ONE deterministic sim tick (20 Hz).
    pub fn tick(&mut self, world: &World) {
        // 1. environmental spawning — one attempt per tick per category
        // while a non-invulnerable player anchor exists (cap-gated)
        if self.player.is_some() && !self.player_invulnerable {
            self.try_spawn_hostile(world);
            self.try_spawn_passive(world);
        }

        // 2. AI + physics (split borrows: rng/hits/arrows vs the mob list)
        let player = self.player;
        let invuln = self.player_invulnerable;
        let rng = &mut self.rng;
        let hits = &mut self.hits;
        let arrows = &mut self.arrows;
        for m in self.list.iter_mut() {
            m.hurt_t = m.hurt_t.saturating_sub(1);
            m.attack_cd = m.attack_cd.saturating_sub(1);
            ai_tick(rng, m, player, invuln, hits, arrows, world);
            physics_tick(m, world);
        }

        // 3. despawn (VERIFIED): >128 blocks instant; 32-block loneliness
        // rolls 1/800 per tick after 30 s
        if let Some(p) = player {
            let mut i = 0;
            while i < self.list.len() {
                let m = &mut self.list[i];
                let dx = m.pos[0] - p[0];
                let dy = m.pos[1] - p[1];
                let dz = m.pos[2] - p[2];
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if dist_sq > DESPAWN_INSTANT_BLOCKS * DESPAWN_INSTANT_BLOCKS {
                    self.list.remove(i);
                    self.despawned_total += 1;
                    continue;
                }
                if dist_sq < DESPAWN_NEAR_BLOCKS * DESPAWN_NEAR_BLOCKS {
                    m.lonely_t = 0;
                } else {
                    m.lonely_t += 1;
                    if m.lonely_t > 600 && self.rng.next_range(800) == 0 {
                        self.list.remove(i);
                        self.despawned_total += 1;
                        continue;
                    }
                }
                i += 1;
            }
        }

        // 4. deaths → events (all damage here is player damage)
        let mut i = 0;
        while i < self.list.len() {
            if self.list[i].health <= 0.0 {
                let m = self.list.remove(i);
                if m.fuse != i32::MAX {
                    // exploded creepers leave no drops (vanilla: destroyed)
                    self.deaths.push((m.kind, m.pos));
                }
                self.killed_total += 1;
            } else {
                i += 1;
            }
        }

        // 5. arrows
        tick_arrows(&mut self.arrows, player, invuln, &mut self.hits, world);
    }

    // --------------------------------------------------------- spawning --

    /// hostile spawn attempt (VERIFIED 1.16.5 rules): block light ≤ 7 AND
    /// sky light ≤ 7, solid floor with 2 air, packs up to 4 (vanilla
    /// monster pack size), cap 70 × chunks/289 (single-player worst case
    /// = the full 289-chunk square → the raw constant).
    fn try_spawn_hostile(&mut self, world: &World) {
        if self.hostiles_alive() as f32 >= MONSTER_CAP {
            return;
        }
        let Some(p) = self.player else { return };
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        if world.chunk((cx, cz)).is_none() {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        let py = p[1] as i32;
        for y in (py - 40..py + 16).rev() {
            if !(1..=250).contains(&y) {
                continue;
            }
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let floor = world.get_block(wx, y - 1, wz);
            if !is_solid(floor) || floor == WATER || is_cross(floor) {
                continue;
            }
            if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
                continue;
            }
            // light gate (VERIFIED 1.16.5): block ≤ 7 AND sky ≤ 7
            let (blk_l, sky_l) = light_levels(world, wx, y, wz);
            if blk_l > HOSTILE_LIGHT_MAX || sky_l > HOSTILE_SKY_MAX {
                return;
            }
            let kind = match self.rng.next_range(5) {
                0 => MobKind::Zombie,
                1 => MobKind::Skeleton,
                2 => MobKind::Creeper,
                3 => MobKind::Spider,
                _ => MobKind::Enderman,
            };
            let pack = 1 + (self.rng.next_range(4)) as usize;
            for _ in 0..pack {
                let _ = self.spawn_at(kind, wx, y, wz);
            }
            return; // one attempt per tick
        }
    }

    /// passive spawn attempt (VERIFIED): light ≥ 9 on GRASS with 2 air,
    /// cap 10; herds of 2–4. Vanilla weights these by biome and runs them
    /// rarely — ours gates at 1/20 per attempt.
    fn try_spawn_passive(&mut self, world: &World) {
        if self.rng.next_range(20) != 0 {
            return;
        }
        if self.passives_alive() as f32 >= CREATURE_CAP {
            return;
        }
        let Some(p) = self.player else { return };
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        if world.chunk((cx, cz)).is_none() {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        let py = p[1] as i32;
        for y in (py - 24..py + 12).rev() {
            if !(1..=250).contains(&y) {
                continue;
            }
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let floor = world.get_block(wx, y - 1, wz);
            if floor != GRASS && floor != SNOW_GRASS {
                continue;
            }
            if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
                continue;
            }
            let (blk_l, _sky) = light_levels(world, wx, y, wz);
            if blk_l < PASSIVE_LIGHT_MIN {
                return;
            }
            let kind = match self.rng.next_range(4) {
                0 => MobKind::Cow,
                1 => MobKind::Pig,
                2 => MobKind::Sheep,
                _ => MobKind::Chicken,
            };
            let herd = 2 + (self.rng.next_range(3)) as usize;
            for _ in 0..herd {
                let _ = self.spawn_at(kind, wx, y, wz);
            }
            return;
        }
    }

    /// Player melee hit on a mob. Neutral mobs become provoked.
    pub fn damage(&mut self, id: u32, amount: f32) -> f32 {
        for m in self.list.iter_mut() {
            if m.id == id {
                m.health -= amount;
                m.hurt_t = 10;
                m.provoked = true;
                return amount;
            }
        }
        0.0
    }

    /// Crosshair ray hit-test against mob AABBs (villager pattern).
    pub fn ray_hit(&self, eye: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<u32> {
        let mut best: Option<(u32, f32)> = None;
        for m in &self.list {
            let d = def(m.kind);
            let half = d.width * 0.5;
            let lo = [m.pos[0] - half, m.pos[1], m.pos[2] - half];
            let hi = [m.pos[0] + half, m.pos[1] + d.height, m.pos[2] + half];
            let mut tmin = 0.0f32;
            let mut tmax = max_dist;
            let mut ok = true;
            for a in 0..3 {
                if dir[a].abs() < 1e-6 {
                    if eye[a] < lo[a] || eye[a] > hi[a] {
                        ok = false;
                        break;
                    }
                } else {
                    let mut t1 = (lo[a] - eye[a]) / dir[a];
                    let mut t2 = (hi[a] - eye[a]) / dir[a];
                    if t1 > t2 {
                        std::mem::swap(&mut t1, &mut t2);
                    }
                    tmin = tmin.max(t1);
                    tmax = tmax.min(t2);
                    if tmin > tmax {
                        ok = false;
                        break;
                    }
                }
            }
            if ok && best.map(|(_, t)| tmin < t).unwrap_or(true) {
                best = Some((m.id, tmin));
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn by_id(&self, id: u32) -> Option<&Mob> {
        self.list.iter().find(|m| m.id == id)
    }
}

// ------------------------------------------------------------- free fns --

/// AI decision + steering for one mob (free fn: splits borrows).
fn ai_tick(
    rng: &mut Rng,
    m: &mut Mob,
    player: Option<[f32; 3]>,
    invuln: bool,
    hits: &mut Vec<PlayerHit>,
    arrows: &mut Vec<Arrow>,
    world: &World,
) {
    let d = def(m.kind);
    let speed = d.speed_attr * SPEED_PER_ATTR;
    let Some(p) = player else {
        wander(rng, m, speed * 0.4);
        return;
    };
    let dx = p[0] - m.pos[0];
    let _dy = p[1] - m.pos[1];
    let dz = p[2] - m.pos[2];
    let dist = (dx * dx + dz * dz).sqrt().max(1e-4);
    let face_player = |m: &mut Mob| {
        m.yaw = (-dz).atan2(dx) - std::f32::consts::FRAC_PI_2;
    };
    let aggro = (m.kind.hostile() || m.provoked) && !invuln;

    match m.kind {
        MobKind::Zombie | MobKind::Spider | MobKind::Enderman => {
            let engage = if m.kind == MobKind::Enderman { m.provoked } else { aggro };
            if engage && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > MOB_MELEE_REACH * 0.8 {
                    let chase = if m.kind == MobKind::Enderman { speed } else { speed };
                    m.vel[0] += (dx / dist * chase - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * chase - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.7;
                    m.vel[2] *= 0.7;
                }
                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage,
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                    });
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        MobKind::Skeleton => {
            if aggro && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > 8.0 {
                    m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                } else if dist < 5.0 {
                    m.vel[0] += (-dx / dist * speed * 0.7 - m.vel[0]) * 0.3;
                    m.vel[2] += (-dz / dist * speed * 0.7 - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.8;
                    m.vel[2] *= 0.8;
                }
                if m.attack_cd == 0 {
                    m.attack_cd = SKELETON_SHOOT_TICKS;
                    spawn_arrow(m, p, rng, arrows);
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        MobKind::Creeper => {
            if aggro && dist < AGGRO_RADIUS {
                face_player(m);
                if m.fuse < 0 {
                    if dist < CREEPER_FUSE_DIST {
                        m.fuse = 0;
                    } else {
                        m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                        m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                    }
                }
                if m.fuse >= 0 && m.fuse != i32::MAX {
                    if dist > CREEPER_FUSE_DIST * 2.4 {
                        m.fuse = -1; // defused — player escaped
                    } else {
                        m.fuse += 1;
                        if m.fuse >= CREEPER_FUSE_TICKS {
                            // consumed marker: take_explosions() surfaces the
                            // blast to the game layer (world edits + light)
                            m.fuse = i32::MAX;
                            m.health = 0.0; // dies in its own blast
                        }
                    }
                }
            } else {
                m.fuse = -1;
                wander(rng, m, speed * 0.5);
            }
        }
        // passives: wander; panic-flee while flashing from a hit
        _ => {
            if m.hurt_t > 0 {
                m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
                let f = speed * FLEE_MULT;
                m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
                m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            } else {
                wander(rng, m, speed * 0.4);
            }
        }
    }
}

fn wander(rng: &mut Rng, m: &mut Mob, speed: f32) {
    if m.wander_t == 0 {
        m.wander_t = (rng.next_range(120) as i32 + 40).max(1);
        m.wander_yaw = rng.next_f32() * std::f32::consts::TAU;
        if rng.next_f32() < 0.6 {
            m.wander_t = -m.wander_t; // negative = standing still
        }
    }
    m.wander_t += if m.wander_t > 0 { -1 } else { 1 };
    if m.wander_t > 0 {
        m.yaw = m.wander_yaw;
        let (s, c) = (m.yaw.sin(), m.yaw.cos());
        m.vel[0] += (s * speed - m.vel[0]) * 0.2;
        m.vel[2] += (-c * speed - m.vel[2]) * 0.2;
    } else {
        m.vel[0] *= 0.8;
        m.vel[2] *= 0.8;
    }
}

/// gravity + axis collision with 1-block step-ups (villager primitive).
fn physics_tick(m: &mut Mob, world: &World) {
    let d = def(m.kind);
    // entity gravity per tick (vanilla 0.08) + terminal 3.92/tick
    m.vel[1] -= 0.08;
    if m.vel[1] < -3.92 {
        m.vel[1] = -3.92;
    }
    // fall damage for mobs (same MC-12357 formula, from impact speed)
    if m.vel[1] < -0.35 && m.on_ground {
        let dist = (m.vel[1] * m.vel[1]) / 0.16;
        if dist > 3.0 {
            m.health -= dist - 3.0;
        }
    }
    let half = d.width * 0.5;
    // horizontal move with step-up
    let (nx, nz) = (m.pos[0] + m.vel[0] * (1.0 / 20.0), m.pos[2] + m.vel[2] * (1.0 / 20.0));
    if !collides(world, nx, m.pos[1], nz, half, d.height) {
        m.pos[0] = nx;
        m.pos[2] = nz;
    } else if !collides(world, nx, m.pos[1] + 1.05, nz, half, d.height) {
        m.pos[0] = nx;
        m.pos[2] = nz;
        m.pos[1] += 1.05;
    } else {
        m.vel[0] *= 0.5;
        m.vel[2] *= 0.5;
    }
    // vertical
    let ny = m.pos[1] + m.vel[1] * (1.0 / 20.0);
    if collides(world, m.pos[0], ny, m.pos[2], half, d.height) {
        if m.vel[1] < 0.0 {
            m.pos[1] = ny.ceil();
            m.on_ground = true;
        }
        m.vel[1] = 0.0;
    } else {
        m.pos[1] = ny;
        m.on_ground = false;
    }
}

fn collides(world: &World, x: f32, y: f32, z: f32, half: f32, height: f32) -> bool {
    let min_x = (x - half).floor() as i32;
    let max_x = (x + half).floor() as i32;
    let min_y = y.floor() as i32;
    let max_y = (y + height - 0.001).floor() as i32;
    let min_z = (z - half).floor() as i32;
    let max_z = (z + half).floor() as i32;
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

/// (block light, sky light) at a world position, straight from the
/// per-chunk LightData map (the same source light_at reads).
fn light_levels(world: &World, wx: i32, wy: i32, wz: i32) -> (u8, u8) {
    let cx = wx.div_euclid(16);
    let cz = wz.div_euclid(16);
    let lx = (wx - cx * 16) as usize;
    let lz = (wz - cz * 16) as usize;
    let sec = (wy.clamp(0, 255) / 16) as usize;
    let yy = (wy.clamp(0, 255) % 16) as usize;
    let idx = (yy << 8) | (lz << 4) | lx;
    world
        .light
        .get(&(cx, cz))
        .and_then(|ld| ld.sections[sec].as_ref().map(|s| (s.blk[idx], s.sky[idx])))
        .unwrap_or((0, 15))
}

/// skeleton arrow: aimed ballistic shot (24 b/s flat, gravity-compensated).
fn spawn_arrow(m: &Mob, target: [f32; 3], rng: &mut Rng, arrows: &mut Vec<Arrow>) {
    let d = def(m.kind);
    let ox = m.pos[0];
    let oy = m.pos[1] + d.height * 0.75;
    let oz = m.pos[2];
    let dx = target[0] - ox;
    let dy = target[1] + 1.2 - oy;
    let dz = target[2] - oz;
    let dist = (dx * dx + dz * dz).sqrt().max(1e-3);
    let speed = 24.0;
    let t = dist / speed;
    // arrow gravity 20 b/s²: compensate the flight-time drop
    let drop = 0.5 * 20.0 * t * t;
    let vy = ((dy + drop) / t.max(1e-3)).min(speed);
    // VERIFIED (wiki skeleton page, Java): Normal arrow damage 3–5
    let damage = 3.0 + rng.next_f32() * 2.0;
    arrows.push(Arrow {
        pos: [ox, oy, oz],
        vel: [dx / dist * speed, vy, dz / dist * speed],
        damage,
        age: 0,
    });
}

fn tick_arrows(
    arrows: &mut Vec<Arrow>,
    player: Option<[f32; 3]>,
    invuln: bool,
    hits: &mut Vec<PlayerHit>,
    world: &World,
) {
    let dt = 1.0 / 20.0;
    let mut i = 0;
    while i < arrows.len() {
        let a = &mut arrows[i];
        a.age += 1;
        a.vel[1] -= 20.0 * dt; // arrow gravity (vanilla 0.05/tick²)
        a.pos[0] += a.vel[0] * dt;
        a.pos[1] += a.vel[1] * dt;
        a.pos[2] += a.vel[2] * dt;
        // player body-center hit sphere (r = 0.8)
        if let Some(p) = player {
            if !invuln {
                let ddx = a.pos[0] - p[0];
                let ddy = a.pos[1] - (p[1] + 0.9);
                let ddz = a.pos[2] - p[2];
                if ddx * ddx + ddy * ddy + ddz * ddz < 0.64 {
                    let dir = [a.vel[0] / 24.0, a.vel[2] / 24.0];
                    hits.push(PlayerHit {
                        damage: a.damage,
                        source: MobKind::Skeleton,
                        knockback_dir: dir,
                    });
                    arrows.remove(i);
                    continue;
                }
            }
        }
        if is_solid(world.get_block(
            a.pos[0].floor() as i32,
            a.pos[1].floor() as i32,
            a.pos[2].floor() as i32,
        )) || a.age > 20 * 60
        {
            arrows.remove(i);
            continue;
        }
        i += 1;
    }
}

/// Creeper fuse completion → surface explosions to the game layer: the
/// exploded creeper is REMOVED here (it died in its own blast, no drops),
/// and game.rs turns each (center, power) into world edits + entity damage.
pub fn take_explosions(sys: &mut MobSystem) -> Vec<([f32; 3], f32)> {
    let mut out = std::mem::take(&mut sys.explosions);
    let mut i = 0;
    while i < sys.list.len() {
        if sys.list[i].fuse == i32::MAX {
            let m = sys.list.remove(i);
            out.push((m.pos, CREEPER_POWER));
        } else {
            i += 1;
        }
    }
    out
}

// ------------------------------------------------------------- rendering --

/// Mob sprites as camera-facing quads (the villager pattern), sized per
/// kind, red-tinted while hurt; creepers blink white while priming.
pub fn build_vertices(
    list: &[Mob],
    right: [f32; 3],
    out: &mut Vec<vc_particles::particles::ParticleVertex>,
) {
    for m in list {
        let d = def(m.kind);
        let tile = m.kind.sprite_tile();
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        let (s, c) = (m.yaw.sin(), m.yaw.cos());
        let rr = [c * right[0] + s * right[2], 0.0, -s * right[0] + c * right[2]];
        let half = d.width * 0.55;
        let h = d.height;
        let mut col = [0.92, 0.92, 0.92];
        if m.hurt_t > 0 {
            col = [1.0, 0.35, 0.35];
        }
        if m.fuse >= 0 && m.fuse != i32::MAX && (m.fuse / 3) % 2 == 0 {
            col = [1.6, 1.6, 1.6];
        }
        let corners = [
            ([-rr[0] * half, 0.0, -rr[2] * half], [tx / 16.0, (ty + 1.0) / 16.0]),
            ([rr[0] * half, 0.0, rr[2] * half], [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0]),
            ([rr[0] * half, h, rr[2] * half], [(tx + 1.0) / 16.0, ty / 16.0]),
            ([-rr[0] * half, h, -rr[2] * half], [tx / 16.0, ty / 16.0]),
        ];
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let (c, uv) = corners[ci];
            out.push(vc_particles::particles::ParticleVertex {
                pos: [m.pos[0] + c[0], m.pos[1] + c[1], m.pos[2] + c[2]],
                uv,
                col,
            });
        }
    }
}

/// Arrow billboards (tiny camera-facing quads on the arrow tile).
pub fn build_arrow_vertices(
    arrows: &[Arrow],
    right: [f32; 3],
    up: [f32; 3],
    out: &mut Vec<vc_particles::particles::ParticleVertex>,
) {
    for a in arrows {
        let tile = TILE_ARROW;
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        let half = 0.35f32;
        let corners = [
            (
                [-right[0] * half - up[0] * half, -right[1] * half - up[1] * half, -right[2] * half - up[2] * half],
                [tx / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [right[0] * half - up[0] * half, right[1] * half - up[1] * half, right[2] * half - up[2] * half],
                [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [right[0] * half + up[0] * half, right[1] * half + up[1] * half, right[2] * half + up[2] * half],
                [(tx + 1.0) / 16.0, ty / 16.0],
            ),
            (
                [-right[0] * half + up[0] * half, -right[1] * half + up[1] * half, -right[2] * half + up[2] * half],
                [tx / 16.0, ty / 16.0],
            ),
        ];
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let (c, uv) = corners[ci];
            out.push(vc_particles::particles::ParticleVertex {
                pos: [a.pos[0] + c[0], a.pos[1] + c[1], a.pos[2] + c[2]],
                uv,
                col: [0.95, 0.95, 0.95],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_world() -> World {
        let mut w = World::new(11);
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

    #[test]
    fn mob_table_matches_verified_wiki_rows() {
        // VERIFIED infobox rows (2026-09-04): health / Normal damage /
        // speed attribute / natural armor
        assert_eq!(def(MobKind::Zombie).health as i32, 20);
        assert_eq!(def(MobKind::Zombie).damage as i32, 3);
        assert!((def(MobKind::Zombie).speed_attr - 0.23).abs() < 1e-6);
        assert_eq!(def(MobKind::Zombie).armor as i32, 2);
        assert_eq!(def(MobKind::Spider).health as i32, 16);
        assert_eq!(def(MobKind::Spider).damage as i32, 2);
        assert!((def(MobKind::Spider).speed_attr - 0.3).abs() < 1e-6);
        assert_eq!(def(MobKind::Enderman).health as i32, 40);
        assert_eq!(def(MobKind::Enderman).damage as i32, 7);
        assert_eq!(def(MobKind::Skeleton).health as i32, 20);
        assert_eq!(def(MobKind::Cow).health as i32, 10);
        assert_eq!(def(MobKind::Pig).health as i32, 10);
        assert_eq!(def(MobKind::Sheep).health as i32, 8);
        assert_eq!(def(MobKind::Chicken).health as i32, 4);
        // hostile/neutral split
        assert!(def(MobKind::Zombie).kind.hostile() && !def(MobKind::Zombie).kind.neutral());
        assert!(!def(MobKind::Enderman).kind.hostile() && def(MobKind::Enderman).kind.neutral());
        assert!(!def(MobKind::Cow).kind.hostile());
    }

    #[test]
    fn verified_constants_are_what_the_wiki_says() {
        assert_eq!(HOSTILE_LIGHT_MAX, 7); // 1.16.5 (1.18 → 0)
        assert_eq!(PASSIVE_LIGHT_MIN, 9);
        assert_eq!(HOSTILE_SKY_MAX, 7);
        assert_eq!(MONSTER_CAP as i32, 70);
        assert_eq!(CREATURE_CAP as i32, 10);
        assert_eq!(CAP_DIVISOR as i32, 289);
        assert_eq!(DESPAWN_INSTANT_BLOCKS as i32, 128);
        assert_eq!(DESPAWN_NEAR_BLOCKS as i32, 32);
        assert_eq!(CREEPER_FUSE_TICKS, 30); // 1.5 s
        assert_eq!(CREEPER_POWER as i32, 3);
    }

    #[test]
    fn spawn_damage_death_cycle() {
        let mut sys = MobSystem::new(5);
        assert!(sys.is_empty());
        let id = sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        assert_eq!(sys.len(), 1);
        assert_eq!(sys.by_id(id).unwrap().health, 20.0);
        assert_eq!(sys.spawned_total, 1);
        assert_eq!(sys.damage(id, 6.0), 6.0);
        assert!((sys.by_id(id).unwrap().health - 14.0).abs() < 1e-5);
        assert!(sys.by_id(id).unwrap().provoked);
        sys.damage(id, 20.0);
        sys.tick(&flat_world());
        assert!(sys.is_empty());
        assert_eq!(sys.deaths.len(), 1);
        assert_eq!(sys.deaths[0].0, MobKind::Zombie);
        assert_eq!(sys.killed_total, 1);
    }

    #[test]
    fn ray_hit_finds_the_crosshair_mob() {
        let mut sys = MobSystem::new(5);
        let id = sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        let eye = [2.5, 66.0, 4.5];
        assert_eq!(sys.ray_hit(eye, [1.0, 0.0, 0.0], 4.0), Some(id));
        assert_eq!(sys.ray_hit(eye, [-1.0, 0.0, 0.0], 4.0), None);
        assert_eq!(sys.ray_hit([40.0, 66.0, 40.0], [1.0, 0.0, 0.0], 4.0), None);
    }

    #[test]
    fn despawn_far_mobs_instantly() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([0.0, 70.0, 0.0]);
        sys.spawn_at(MobKind::Zombie, 0, 65, 0).unwrap();
        sys.spawn_at(MobKind::Zombie, 200, 65, 200).unwrap(); // >128 away
        sys.tick(&flat_world());
        assert_eq!(sys.len(), 1, "far mob gone, near mob stays");
        assert_eq!(sys.despawned_total, 1);
    }

    #[test]
    fn skeleton_shoots_and_the_arrow_lands() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([10.5, 65.0, 4.5]);
        sys.spawn_at(MobKind::Skeleton, 0, 65, 4).unwrap();
        // force aggro and an immediate shot (free-fn call, split borrows)
        let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
        let mut mob = sys.list.remove(0);
        ai_tick(
            &mut rng,
            &mut mob,
            sys.player,
            false,
            &mut sys.hits,
            &mut sys.arrows,
            &flat_world(),
        );
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert_eq!(sys.arrows.len(), 1, "skeleton fired one arrow");
        // fly it at the player
        let world = flat_world();
        for _ in 0..300 {
            tick_arrows(&mut sys.arrows, sys.player, false, &mut sys.hits, &world);
            if !sys.hits.is_empty() {
                break;
            }
        }
        assert!(!sys.hits.is_empty(), "arrow reached the player");
        let hit = &sys.hits[0];
        assert!(hit.damage >= 3.0 && hit.damage <= 5.0, "Normal 3-5, got {}", hit.damage);
        assert_eq!(hit.source, MobKind::Skeleton);
    }

    #[test]
    fn creeper_fuses_then_explodes() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([5.5, 65.0, 4.5]); // ~1 block from the creeper
        sys.spawn_at(MobKind::Creeper, 4, 65, 4).unwrap();
        let world = flat_world();
        // AI: fuse starts
        let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
        let mut mob = sys.list.remove(0);
        ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world);
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert!(sys.list[0].fuse >= 0, "fuse started");
        // count up to the blast
        let pos0 = sys.list[0].pos;
        for _ in 0..CREEPER_FUSE_TICKS + 2 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world);
            sys.list.insert(0, mob);
            sys.rng = rng;
            physics_tick(&mut sys.list[0], &world);
            if sys.list[0].fuse == i32::MAX {
                break;
            }
        }
        assert_eq!(sys.list[0].fuse, i32::MAX, "fuse completed");
        assert!(sys.list[0].health <= 0.0, "creeper died in its blast");
        let booms = take_explosions(&mut sys);
        assert_eq!(booms.len(), 1);
        assert_eq!(booms[0].0, pos0);
        assert_eq!(booms[0].1 as i32, 3, "explosion power 3 (VERIFIED)");
        // the death sweep must NOT queue drops (exploded = destroyed)
        sys.tick(&world);
        assert!(sys.is_empty());
        assert!(sys.deaths.is_empty(), "exploded creepers drop nothing");
    }

    #[test]
    fn passive_mobs_flee_when_hurt() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([6.5, 65.0, 4.5]);
        sys.spawn_at(MobKind::Cow, 4, 65, 4).unwrap();
        let id = sys.list[0].id;
        sys.damage(id, 3.0);
        assert!((sys.by_id(id).unwrap().health - 7.0).abs() < 1e-5);
        // hurt cow runs away from the player
        let world = flat_world();
        let x_before = sys.list[0].pos[0];
        for _ in 0..40 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world);
            sys.list.insert(0, mob);
            sys.rng = rng;
            physics_tick(&mut sys.list[0], &world);
        }
        // the player is +x from the cow: fleeing moves -x
        assert!(
            sys.list[0].pos[0] < x_before + 0.1,
            "cow fled away, x {} -> {}",
            x_before,
            sys.list[0].pos[0]
        );
    }

    #[test]
    fn invulnerable_players_never_get_hit() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([5.5, 65.0, 4.5]);
        sys.player_invulnerable = true; // creative
        sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        let world = flat_world();
        for _ in 0..60 {
            sys.tick(&world);
        }
        assert!(sys.hits.is_empty(), "creative is never attacked");
        // ...and nothing even spawns while invulnerable
        assert_eq!(sys.spawned_total, 1, "only the explicit spawn");
    }
}
