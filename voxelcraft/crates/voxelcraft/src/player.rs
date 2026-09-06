//! Player: movement (walk/sprint/swim/fly), AABB voxel collision,
//! DDA block raycast, hotbar state, footstep logic.

use glam::Vec3;
use vc_blocks::blocks::*;
use vc_world::world::World;

pub const WALK_SPEED: f32 = 4.317;
pub const SPRINT_SPEED: f32 = 5.612;
pub const FLY_SPEED: f32 = 10.9;
pub const FLY_SPRINT: f32 = 21.8;
pub const GRAVITY: f32 = 32.0; // 0.08 b/tick² × 20² — reference constant
/// vanilla jump velocity 0.42 blocks/tick × 20 (§23; with the exact
/// per-tick drag integration this yields the vanilla 1.25-block apex)
pub const JUMP_VEL: f32 = 8.4;
pub const TERMINAL: f32 = 78.4; // 3.92 b/tick × 20 — inherent fixed point of the drag formula
/// research-verdicts.md live round (minecraft.wiki/w/Transportation):
/// still-water surface swim 2.20 b/s, underwater 1.97 b/s,
/// sprint-swim 3.918 b/s. (The mechanics document's "downstream 1.81 /
/// upstream 0.39" labels were mislabeled — see verdicts.)
pub const SWIM_SPEED_SURFACE: f32 = 2.20;
pub const SWIM_SPEED_UNDERWATER: f32 = 1.97;
pub const SPRINT_SWIM_SPEED: f32 = 3.918;
/// air supply / drowning (research-verdicts.md live round,
/// minecraft.wiki/w/Damage §Drowning): max 300 air (depletes 1/tick
/// submerged), 10 bubbles × 30 air; damage 2 HP when air reaches −20,
/// then air resets to 0; regen 30 air per 4 ticks when the head is out.
pub const AIR_MAX: f32 = 300.0;
pub const AIR_DROWN_AT: f32 = -20.0;
pub const DROWN_DMG: f32 = 2.0;
pub const AIR_REGEN_PER_TICK: f32 = 7.5; // 30 air / 4 ticks
/// fixed simulation rate — vanilla per-tick formulas integrate verbatim
pub const TPS: f32 = 20.0;
pub const TICK_DT: f32 = 1.0 / 20.0;
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

/// 1.7.2 bracket: status-effect state carried by the player. Durations are
/// remaining game ticks (20 Hz); the poison timer is the inter-damage
/// countdown. Numbers live-verified (minecraft.wiki/w/Poison, /w/Pufferfish,
/// 2026-09-06) — see `Player::tick_effects` for the cadence derivation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StatusEffects {
    /// remaining Poison ticks (pufferfish: Poison IV, 1200 = 1:00)
    pub poison_ticks: i32,
    /// countdown to the next poison damage application
    poison_timer: i32,
    /// remaining Slowness ticks (1.10 stray arrows: 600 = 0:30)
    pub slowness_ticks: i32,
    /// remaining Hunger ticks (1.10 husk hits: 7×floor(regional difficulty) s)
    pub hunger_ticks: i32,
}

/// observable poison cadence: the 10-tick hurt-immunity floor (the raw
/// level-IV cadence is 3 ticks per HP, but damage below the immunity
/// window is skipped — the wiki's own effective-rate row)
pub const POISON_INTERVAL_TICKS: i32 = 10;
/// poison cannot kill — health floors at 1 HP (wiki /w/Poison)
pub const POISON_FLOOR_HP: f32 = 1.0;

/// 1.8 slime-block restitution: rebound reaches "up to 60% of initial
/// height" (wiki §Slime Block) → v ratio = sqrt(0.6)
pub const SLIME_RESTITUTION: f32 = 0.7746;

pub struct Player {
    pub pos: Vec3, // feet center
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub flying: bool,
    pub in_water: bool,
    pub head_in_water: bool,
    /// health points (0..20, vanilla half-heart scale ×2; §29 potions act
    /// on this, the HUD renders it as the real health bar)
    pub health: f32,
    /// XP points progress within the current level (§29)
    pub xp_points: i32,
    /// XP level (§29; enchanting pays 1..3 of these per option)
    pub xp_level: i32,
    /// 36-slot inventory: 0..9 hotbar, 9..36 storage (Phase 7)
    pub inv: vc_inventory::inventory::Inventory,
    pub selected: usize,
    pub fov: f32,
    pub fov_cur: f32,
    last_space_tap: f32,
    step_accum: f32,
    was_in_water: bool,
    /// Phase 1: blocks fallen since last reset (vanilla `fallDistance`).
    /// Reset by water, flight, climbing — NOT by mere landing (landing is
    /// what CONVERTS it into damage).
    pub fall_dist: f32,
    /// Phase 1: fall damage queued for the game layer (HP, half-heart
    /// scale). Vanilla formula, verified against Mojang's own tracker
    /// (MC-12357, Dossier Part 5 §25): damage = fall_distance − 3, so a
    /// 4-block fall costs 1 HP and a 23-block fall is lethal.
    pending_fall_dmg: f32,
    /// drowning damage queued for the game layer (2 HP per second once
    /// the air supply is depleted — research-verdicts.md live round).
    pending_drown_dmg: f32,
    /// air supply 0..300 (over-depletes to −20, then resets on damage).
    /// 300 = full (10 bubbles × 30).
    pub air: f32,
    /// 1.7.2 bracket: player status effects. Poison is the pufferfish
    /// effect (Poison IV 1:00 — cadence + cannot-kill floor live-verified
    /// on minecraft.wiki/w/Poison). Slowness arrives with the 1.10 stray
    /// (0:30 arrows) and Hunger with the 1.10 husk — all fields ride the
    /// same struct so later brackets extend without re-plumbing.
    pub effects: StatusEffects,
    /// 1.8: spectator no-clip — set by the game layer each frame from
    /// the mode; move_axis skips collision while set
    pub noclip: bool,
    /// fixed 20 Hz substep accumulator (vanilla per-tick physics)
    tick_accum: f32,
    air_accum: f32,
    was_on_ground: bool,
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
            health: 20.0,
            xp_points: 0,
            xp_level: 0,
            inv: {
                let mut inv =
                    vc_inventory::inventory::Inventory::new(vc_inventory::inventory::INV_SLOTS);
                for (i, &b) in PALETTE.iter().enumerate() {
                    inv.slots[i] = vc_inventory::inventory::ItemStack::new(b, 64);
                }
                inv
            },
            selected: 0,
            fov: 1.2217, // 70 degrees
            fov_cur: 1.2217,
            last_space_tap: -1.0,
            step_accum: 0.0,
            was_in_water: false,
            fall_dist: 0.0,
            pending_fall_dmg: 0.0,
            effects: StatusEffects::default(),
            noclip: false,
            pending_drown_dmg: 0.0,
            air: AIR_MAX,
            tick_accum: 0.0,
            air_accum: 0.0,
            was_on_ground: false,
        }
    }

    /// Phase 1: drain queued fall damage (HP). One-shot: landing converts
    /// the whole accumulated fall into one hit, exactly like vanilla.
    pub fn take_pending_fall_damage(&mut self) -> f32 {
        let d = self.pending_fall_dmg;
        self.pending_fall_dmg = 0.0;
        d
    }

    /// Drain queued drowning damage (2 HP per second once depleted —
    /// research-verdicts.md live round). One-shot per damage tick.
    pub fn take_pending_drown_damage(&mut self) -> f32 {
        let d = self.pending_drown_dmg;
        self.pending_drown_dmg = 0.0;
        d
    }

    /// Reset fall accumulation (spawn snap / respawn / mode change).
    pub fn reset_fall(&mut self) {
        self.fall_dist = 0.0;
        self.pending_fall_dmg = 0.0;
        self.was_on_ground = false;
    }

    /// Reset air supply (respawn / mode change).
    pub fn reset_air(&mut self) {
        self.air = AIR_MAX;
        self.pending_drown_dmg = 0.0;
    }

    /// Reset status effects (respawn / mode change / milk-when-it-exists).
    pub fn reset_effects(&mut self) {
        self.effects = StatusEffects::default();
    }

    /// 1.7.2 bracket: one status-effect GAME tick (called from the fixed
    /// 20 Hz sim step, alongside the air/drown logic). Returns the poison
    /// damage dealt this tick (0 if none) so the game layer can play the
    /// hurt sound exactly once per application.
    ///
    /// Poison cadence VERIFIED (minecraft.wiki/w/Poison, live 2026-09-06):
    /// level IV damages 1 HP per 3 ticks raw, but the 10-tick hurt
    /// immunity makes the OBSERVABLE rate 1 HP per 10 ticks (1 HP/s) —
    /// we tick at the observable rate. Poison can never kill: it floors
    /// at 1 HP ("it cannot kill. It can, however, reduce the player's
    /// health all the way to 1").
    pub fn tick_effects(&mut self) -> f32 {
        let mut dealt = 0.0;
        if self.effects.poison_ticks > 0 {
            self.effects.poison_ticks -= 1;
            self.effects.poison_timer -= 1;
            if self.effects.poison_timer <= 0 {
                self.effects.poison_timer = POISON_INTERVAL_TICKS;
                // cannot-kill floor at 1 HP
                if self.health > POISON_FLOOR_HP {
                    let cap = self.health - POISON_FLOOR_HP;
                    dealt = self.damage(1.0_f32.min(cap));
                }
            }
        } else {
            self.effects.poison_timer = 0;
        }
        if self.effects.slowness_ticks > 0 {
            self.effects.slowness_ticks -= 1;
        }
        if self.effects.hunger_ticks > 0 {
            self.effects.hunger_ticks -= 1;
        }
        dealt
    }

    /// pufferfish poisoning: Poison IV for 1:00 (wiki /w/Pufferfish)
    pub fn apply_pufferfish_poison(&mut self) {
        self.effects.poison_ticks = 20 * 60;
        self.effects.poison_timer = POISON_INTERVAL_TICKS;
    }

    /// the selected hotbar stack
    pub fn held(&self) -> vc_inventory::inventory::ItemStack {
        self.inv.slots[self.selected]
    }

    /// clamp-to-max healing (§29 potions)
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(20.0);
    }

    /// damage clamped to 0; returns the ACTUAL damage applied
    pub fn damage(&mut self, amount: f32) -> f32 {
        let before = self.health;
        self.health = (self.health - amount).max(0.0);
        before - self.health
    }

    /// add XP points → levels advance on the vanilla curve (§29); returns
    /// how many levels were gained (for the level-up sound hook)
    pub fn add_xp(&mut self, points: i32) -> i32 {
        self.xp_points += points.max(0);
        let mut gained = 0;
        loop {
            let need = vc_gameplay::enchanting::xp_to_next(self.xp_level);
            if self.xp_points >= need {
                self.xp_points -= need;
                self.xp_level += 1;
                gained += 1;
            } else {
                break;
            }
        }
        gained
    }

    /// pay `levels` levels for an enchant (§29): requires the levels,
    /// resets in-level progress (vanilla sets the bar to 0)
    pub fn spend_levels(&mut self, levels: i32) -> bool {
        if self.xp_level < levels {
            return false;
        }
        self.xp_level -= levels;
        self.xp_points = 0;
        true
    }

    /// XP bar fraction for the HUD (progress within the current level)
    pub fn xp_fraction(&self) -> f32 {
        let need = vc_gameplay::enchanting::xp_to_next(self.xp_level);
        (self.xp_points as f32 / need as f32).clamp(0.0, 1.0)
    }

    /// mutable access to the selected hotbar stack
    pub fn held_mut(&mut self) -> &mut vc_inventory::inventory::ItemStack {
        &mut self.inv.slots[self.selected]
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
        self.pitch = (self.pitch - mdy * 0.0022 * sensitivity).clamp(-1.5685, 1.5685);

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
            // swim targets from the verified wiki table (research-verdicts
            // live round): sprint-swim 3.918, underwater 1.97, surface 2.20
            let speed = if self.in_water {
                if sprinting {
                    SPRINT_SWIM_SPEED
                } else if self.head_in_water {
                    SWIM_SPEED_UNDERWATER
                } else {
                    SWIM_SPEED_SURFACE
                }
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
                // no per-tick gravity while buoyant — also freeze the
                // substep accumulator so leaving water starts fresh
                self.tick_accum = 0.0;
            } else {
                if input.jump && self.on_ground {
                    self.vel.y = JUMP_VEL;
                    self.on_ground = false;
                    // a velocity discontinuity re-aligns the 20 Hz
                    // substep phase: in vanilla the jump lands ON a tick
                    // boundary, so the fresh velocity is owed a FULL
                    // first tick before gravity touches it (without this
                    // the jump apex loses up to 2/3 of tick 0 and
                    // undershoots the vanilla 1.25-block height)
                    self.tick_accum = 0.0;
                }
                // the per-tick gravity/drag itself runs AFTER the
                // position integration below — vanilla tick order is
                // [move by v] THEN [v ← (v − 0.08) × 0.98], so each
                // velocity value drives exactly one 50 ms slice of
                // motion (semi-implicit ordering steals up to a frame
                // from the fresh velocity and shortens jumps)
            }
        }

        // Air supply / drowning (VERIFIED — research-verdicts.md live
        // round, minecraft.wiki/w/Damage §Drowning): fixed 20 Hz substep;
        // 1 air per tick submerged; at −20 → 2 HP queued and air resets
        // to 0 (so damage repeats once per second); +7.5 air per tick
        // (30 per 4 ticks) with the head above water.
        self.air_accum += dt;
        let mut air_ticks = 0u8;
        while self.air_accum >= TICK_DT && air_ticks < 40 {
            self.air_accum -= TICK_DT;
            air_ticks += 1;
            if self.head_in_water {
                self.air -= 1.0;
                if self.air <= AIR_DROWN_AT {
                    self.pending_drown_dmg += DROWN_DMG;
                    self.air = 0.0;
                }
            } else {
                self.air = (self.air + AIR_REGEN_PER_TICK).min(AIR_MAX);
            }
        }

        // FOV: sprint zoom like MC
        let target_fov = if sprinting && !self.flying {
            self.fov * 1.12
        } else {
            self.fov
        };
        self.fov_cur += (target_fov - self.fov_cur) * (10.0 * dt).min(1.0);

        // Phase 1: vanilla fall-distance bookkeeping — accumulated per
        // tick inside the 20 Hz gravity substep above (vanilla semantics);
        // water and flight zero it (a dive or a hover is free); landing
        // converts it to queued damage via the MC-12357 formula
        // `fall_distance − 3` HP.
        if self.flying || self.in_water {
            self.fall_dist = 0.0;
        }

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

        // Vanilla entity gravity, EXACT per-tick form (VERIFIED,
        // research-verdicts.md + decompiled snippet):
        //     v1 = (v0 − 0.08) × 0.98      [blocks/tick]
        // Runs on a fixed 20 Hz substep AFTER the position integration
        // (vanilla tick order: move by v, then update v — each velocity
        // value drives exactly one 50 ms slice of motion). Horizontal
        // motion stays continuous. The 0.98 drag makes terminal velocity
        // an inherent fixed point (−3.92 b/t = −78.4 b/s) approached from
        // BOTH sides — no clamp is needed (NaN guard only).
        if !self.flying && !self.in_water {
            self.tick_accum += dt;
            let mut ticks = 0u8;
            while self.tick_accum >= TICK_DT && ticks < 40 {
                self.tick_accum -= TICK_DT;
                ticks += 1;
                // vanilla fallDistance: the distance THIS tick's motion
                // covered (the pre-update velocity is what moved us)
                let v_bpt = self.vel.y / TPS;
                if v_bpt < 0.0 {
                    self.fall_dist += -v_bpt;
                }
                let v1 = (v_bpt - 0.08) * 0.98;
                self.vel.y = v1 * TPS;
                if self.vel.y.is_nan() {
                    self.vel.y = 0.0;
                }
            }
        } else {
            self.tick_accum = 0.0;
        }

        // Phase 1: the landing itself — one hit for the whole fall
        // (vanilla applies it on the ground-contact tick; water/flight
        // already zeroed fall_dist above, so a dive lands free)
        //
        // 1.8 (VERIFIED — minecraft.wiki/w/Java_Edition_1.8 §Slime Block,
        // live 2026-09-06): landing on a slime block negates all fall
        // damage and bounces the player ("This negates all fall damage...
        // Height can reach up to 60% of initial height"). Sneaking
        // negates the rebound AND the fall-damage negation ("Holding
        // ⇧ Shift negates the rebound, and does not negate the fall
        // damage"). Rebound velocity = sqrt(0.6) ≈ 0.775 × impact (the
        // 60% height ratio; height ∝ v²).
        let landed_this_frame = self.on_ground && !self.was_on_ground;
        let under = world.get_block(
            self.pos.x.floor() as i32,
            (self.pos.y - 0.4).floor() as i32,
            self.pos.z.floor() as i32,
        );
        let landed_on_slime = under == SLIME_BLOCK;
        if landed_this_frame && landed_on_slime && !input.sneak && self.fall_dist > 0.6 {
            // bounce: convert the fall into an upward launch, skip damage
            let impact = (2.0 * GRAVITY * self.fall_dist).sqrt(); // b/s
            self.vel.y = impact * SLIME_RESTITUTION;
            self.fall_dist = 0.0;
            // the bounce itself is airborne: leave on_ground false-y by
            // pushing off immediately (next tick gravity takes over)
            self.pending_fall_dmg = 0.0;
        } else if landed_this_frame && self.fall_dist > 3.0 {
            self.pending_fall_dmg += self.fall_dist - 3.0;
        }
        if self.on_ground {
            self.fall_dist = 0.0;
        }
        self.was_on_ground = self.on_ground;

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
        // 1.8 spectator: no-clip — straight-line motion, no collision
        if self.noclip {
            match axis {
                0 => self.pos.x += d,
                1 => self.pos.y += d,
                _ => self.pos.z += d,
            }
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
        bmin.x < pmax.x
            && bmax.x > pmin.x
            && bmin.y < pmax.y
            && bmax.y > pmin.y
            && bmin.z < pmax.z
            && bmax.z > pmin.z
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
pub fn raycast(
    world: &World,
    eye: Vec3,
    dir: Vec3,
    max_dist: f32,
) -> Option<([i32; 3], u8, [i32; 3])> {
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
        let gen = vc_world::gen::TerrainGen::new(42);
        let (chunk, outbound) = gen.generate_chunk(0, 0, vec![]);
        world.insert_generated((0, 0), chunk, outbound);
        let eye = Vec3::new(0.5, 90.0, 0.5);
        let hit = raycast(&world, eye, Vec3::new(0.0, -1.0, 0.0), 50.0);
        assert!(hit.is_some(), "straight-down ray must hit terrain");
        let (p, b, _) = hit.unwrap();
        println!("hit block {} at {:?}", b, p);
        assert!(b != vc_blocks::blocks::AIR);
    }

    #[test]
    fn raycast_hits_forward_down_45() {
        let mut world = World::new(42);
        let gen = vc_world::gen::TerrainGen::new(42);
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
            if b != vc_blocks::blocks::AIR && b != vc_blocks::blocks::WATER {
                return y + 1;
            }
        }
        64
    }

    // ------------------------------------------------ §23 physics constants --

    /// hand-built flat world: stone floor top surface at y=65
    fn flat_floor() -> World {
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
                for y in 0..=64i32 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y as usize, lz, vc_blocks::blocks::STONE);
                        }
                    }
                }
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w
    }

    /// vanilla 1.16.5 walking speed converges to 4.317 m/s (§23)
    #[test]
    fn walk_speed_converges_to_vanilla() {
        let w = flat_floor();
        let mut p = Player::new(Vec3::new(0.5, 65.0, 0.5));
        p.flying = false;
        let mut input = Input::default();
        input.fwd = true;
        // 3 seconds at 60 Hz — acceleration converges (rate 12 on ground)
        for _ in 0..180 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
        }
        let horiz = (p.vel.x * p.vel.x + p.vel.z * p.vel.z).sqrt();
        assert!(
            (horiz - WALK_SPEED).abs() < 0.05,
            "walk speed {horiz} vs vanilla {WALK_SPEED}"
        );
        // actually moved (yaw 0 faces −Z)
        assert!(p.pos.z < -10.0, "walked forward, z={}", p.pos.z);
        // stayed on the floor
        assert!((p.pos.y - 65.0).abs() < 0.02, "grounded, y={}", p.pos.y);
    }

    /// sprint = 5.612 m/s (§23)
    #[test]
    fn sprint_speed_converges_to_vanilla() {
        let w = flat_floor();
        let mut p = Player::new(Vec3::new(0.5, 65.0, 0.5));
        p.flying = false;
        let mut input = Input::default();
        input.fwd = true;
        input.sprint = true;
        for _ in 0..180 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
        }
        let horiz = (p.vel.x * p.vel.x + p.vel.z * p.vel.z).sqrt();
        assert!(
            (horiz - SPRINT_SPEED).abs() < 0.05,
            "sprint speed {horiz} vs vanilla {SPRINT_SPEED}"
        );
    }

    /// jump apex ≈ 1.25 blocks (§23 vanilla 1.16.5 jump height) — under
    /// the EXACT per-tick drag formula (v1 = (v0 − 0.08) × 0.98) the
    /// vanilla 0.42 b/t launch rises 1.2492 blocks; the engine launches
    /// at 8.4 b/s = 0.42 b/t and must match
    #[test]
    fn jump_apex_is_vanilla_height() {
        let w = flat_floor();
        let mut p = Player::new(Vec3::new(0.5, 65.0, 0.5));
        p.flying = false;
        let mut input = Input::default();
        input.jump = true;
        let start = p.pos.y;
        let mut apex = start;
        for _ in 0..120 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
            apex = apex.max(p.pos.y);
            if p.on_ground && p.pos.y > start + 0.5 {
                break; // landed
            }
        }
        let height = apex - start;
        // reference: per-tick positions under the drag formula
        // (integrate v0 = 0.42 b/t: v1 = (v0 − 0.08) × 0.98 each tick)
        let mut v = 0.42f32;
        let mut ref_apex = 0.0f32;
        let mut h = 0.0f32;
        while v > 0.0 {
            h += v;
            ref_apex = ref_apex.max(h);
            v = (v - 0.08) * 0.98;
        }
        assert!(
            (height - ref_apex).abs() < 0.03,
            "jump height {height} vs drag-integrated apex {ref_apex}"
        );
        assert!((ref_apex - 1.25).abs() < 0.01, "vanilla jump parity");
    }

    /// terminal velocity under the drag formula: the fixed point is
    /// −3.92 b/t (−78.4 b/s), approached from BOTH sides — a launch
    /// FASTER than terminal decays toward it, a free fall converges up
    #[test]
    fn fall_accelerates_and_terminates() {
        let mut w = flat_floor();
        // remove the floor under the column so the fall is unbounded
        for y in 0..=64i32 {
            w.set_block(0, y, 0, vc_blocks::blocks::AIR);
        }
        let mut input = Input::default();
        // past terminal (−5 b/t = −100 b/s): drag must pull it back UP
        // toward −3.92 b/t (i.e. next tick is SLOWER than the launch)
        let mut p = Player::new(Vec3::new(0.5, 65.0, 0.5));
        p.flying = false;
        p.vel.y = -100.0;
        let _ = p.update(1.0 / 20.0, 0.0, &w, &mut input, 1.0, true);
        // one update at exactly one tick: v ← (v0 − 1.6) × 0.98 in b/s,
        // i.e. (v0/20 − 0.08) × 0.98 in b/t
        let expect = (-5.0f32 - 0.08) * 0.98; // b/t: (−5 − 0.08) × 0.98
        let got = p.vel.y / TPS;
        assert!(
            (got - expect).abs() < 1e-3,
            "launch past terminal decays toward it: {got} vs {expect}"
        );
        assert!(got > -5.0, "|v| shrinks from above terminal");
        // long convergence: 400 ticks from rest stays near the −3.92 fixed
        // point and never blows past the old hard clamp territory
        let mut p2 = Player::new(Vec3::new(0.5, 65.0, 0.5));
        p2.flying = false;
        for _ in 0..400 {
            let _ = p2.update(1.0 / 20.0, 0.0, &w, &mut input, 1.0, true);
        }
        assert!(
            p2.vel.y > -TERMINAL - 0.01 && p2.vel.y < -TERMINAL + 4.0,
            "converges around terminal, got {}",
            p2.vel.y
        );
    }

    /// the exact per-tick formula, verbatim: one update at dt = 1/20 from
    /// a known velocity must equal (v0 − 0.08) × 0.98 in b/t units
    #[test]
    fn gravity_drag_matches_vanilla_formula() {
        let mut w = flat_floor();
        for y in 0..=64i32 {
            w.set_block(0, y, 0, vc_blocks::blocks::AIR);
        }
        let mut input = Input::default();
        for v0_bps in [0.0f32, -20.0, 8.4, -78.4, -100.0] {
            let mut p = Player::new(Vec3::new(0.5, 65.0, 0.5));
            p.flying = false;
            p.vel.y = v0_bps;
            let _ = p.update(TICK_DT, 0.0, &w, &mut input, 1.0, true);
            let v0 = v0_bps / TPS;
            let expect = (v0 - 0.08) * 0.98 * TPS;
            assert!(
                (p.vel.y - expect).abs() < 1e-3,
                "v0 {v0_bps} b/s: got {} want {expect}",
                p.vel.y
            );
        }
    }

    /// drowning (research-verdicts live round): 300 ticks submerged → 0
    /// air; 20 more → 2 HP queued + reset to 0; out of water refills in
    /// 40 ticks (30 air / 4 ticks)
    #[test]
    fn air_depletes_drowns_and_regenerates() {
        // head submerged: eye 1.62 over a pool floor
        let mut w = flat_floor();
        for y in 62..=70i32 {
            for z in -2..=2i32 {
                for x in -2..=2i32 {
                    w.set_block(x, y, z, vc_blocks::blocks::WATER);
                }
            }
        }
        let mut p = Player::new(Vec3::new(0.5, 64.0, 0.5));
        p.flying = false;
        let mut input = Input::default();
        // 300 ticks = 15 s at the fixed step
        for _ in 0..300 {
            let _ = p.update(TICK_DT, 0.0, &w, &mut input, 1.0, true);
        }
        assert!(p.head_in_water, "submerged");
        assert!((p.air).abs() < 1.5, "air drained to ~0, got {}", p.air);
        assert_eq!(p.take_pending_drown_damage(), 0.0, "no damage at 0");
        // 20 more ticks → −20 threshold → 2 HP queued, air reset to 0
        for _ in 0..20 {
            let _ = p.update(TICK_DT, 0.0, &w, &mut input, 1.0, true);
        }
        assert_eq!(p.take_pending_drown_damage(), DROWN_DMG);
        assert!((p.air).abs() < 1.0, "air reset to 0, got {}", p.air);
        // surfacing: head above water (hover above the pool — flying so
        // gravity does not drop the player back in)
        p.pos.y = 72.0;
        p.vel.y = 0.0;
        p.flying = true;
        for _ in 0..2 {
            let _ = p.update(TICK_DT, 0.0, &w, &mut input, 1.0, true);
        }
        for _ in 0..40 {
            let _ = p.update(TICK_DT, 0.0, &w, &mut input, 1.0, true);
        }
        assert!((p.air - AIR_MAX).abs() < 1.0, "refilled, got {}", p.air);
    }

    /// swimming: buoyancy + water speed scaling (§23 water movement)
    #[test]
    fn water_slows_and_buoys() {
        let mut w = flat_floor();
        // flood a wide pool (the player walks forward for 2 s — must
        // stay inside the water the whole time)
        for y in 65..70i32 {
            for z in -12..=12i32 {
                for x in -12..=12i32 {
                    w.set_block(x, y, z, vc_blocks::blocks::WATER);
                }
            }
        }
        let mut p = Player::new(Vec3::new(0.5, 66.0, 0.5));
        p.flying = false;
        let mut input = Input::default();
        input.fwd = true;
        for _ in 0..120 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
        }
        assert!(p.in_water, "in water");
        let horiz = (p.vel.x * p.vel.x + p.vel.z * p.vel.z).sqrt();
        // verified swim speed (research-verdicts live round): the pool is
        // 5 deep so the head is under → 1.97 b/s underwater target
        assert!(
            (horiz - SWIM_SPEED_UNDERWATER).abs() < 0.05,
            "water speed {} vs {}",
            horiz,
            SWIM_SPEED_UNDERWATER
        );
        // sprint-swim converges to the verified 3.918 b/s
        let mut p2 = Player::new(Vec3::new(0.5, 66.0, 0.5));
        p2.flying = false;
        let mut input2 = Input::default();
        input2.fwd = true;
        input2.sprint = true;
        for _ in 0..120 {
            let _ = p2.update(1.0 / 60.0, 0.0, &w, &mut input2, 1.0, true);
        }
        let sprint_horiz = (p2.vel.x * p2.vel.x + p2.vel.z * p2.vel.z).sqrt();
        assert!(
            (sprint_horiz - SPRINT_SWIM_SPEED).abs() < 0.06,
            "sprint-swim {} vs {}",
            sprint_horiz,
            SPRINT_SWIM_SPEED
        );
        // sank to the pool floor (vy 0 on the ground) or still sinking,
        // never faster than the -4 water terminal
        assert!(
            p.vel.y > -4.5 && p.vel.y <= 0.0,
            "buoyant sink, vy={}",
            p.vel.y
        );
        assert!(
            (p.pos.y - 65.0).abs() < 0.1,
            "resting on the pool floor, y={}",
            p.pos.y
        );
    }

    /// Phase 1: fall damage follows the vanilla MC-12357 formula —
    /// damage = fall_distance − 3 HP (a 4-block drop costs 1 HP, a
    /// 23-block drop is lethal at 20 HP). Landing in water is free.
    #[test]
    fn fall_damage_is_distance_minus_three() {
        let mut input = Input::default();
        // drop from 7 blocks onto the flat floor: expect 7 − 3 = 4 HP
        let mut w = flat_floor();
        let top = chunk_top(&w) as f32;
        let mut p = Player::new(Vec3::new(0.5, top + 7.0 + 1.0, 0.5));
        p.flying = false;
        let mut frames = 0;
        while !p.on_ground && frames < 600 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
            frames += 1;
        }
        assert!(p.on_ground, "must land within 10 s");
        let dmg = p.take_pending_fall_damage();
        // fall_dist counts the ~7-block descent (exact distance differs by
        // the integration step and the +1 spawn epsilon — accept a window)
        assert!(
            (3.0..=6.5).contains(&dmg),
            "7-block fall deals ~4 HP, got {dmg}"
        );
        // queued damage drains once, then nothing
        assert_eq!(p.take_pending_fall_damage(), 0.0);
    }

    #[test]
    fn short_falls_and_water_entries_cost_nothing() {
        let mut input = Input::default();
        // sub-3-block hop: below the 3-block grace, zero damage (spawned a
        // hair under the threshold so integration overshoot can't trip it)
        let w = flat_floor();
        let top = chunk_top(&w) as f32;
        let mut p = Player::new(Vec3::new(0.5, top + 2.5, 0.5));
        p.flying = false;
        let mut frames = 0;
        while !p.on_ground && frames < 600 {
            let _ = p.update(1.0 / 60.0, 0.0, &w, &mut input, 1.0, true);
            frames += 1;
        }
        assert_eq!(p.take_pending_fall_damage(), 0.0, "2.5-block fall is free");
        // deep dive into water: fall distance resets, no queued damage
        let mut w2 = flat_floor();
        for y in 50..=64i32 {
            for z in -4..=4i32 {
                for x in -4..=4i32 {
                    w2.set_block(x, y, z, vc_blocks::blocks::WATER);
                }
            }
        }
        let mut p2 = Player::new(Vec3::new(0.5, 64.0, 0.5));
        p2.flying = false;
        for _ in 0..120 {
            let _ = p2.update(1.0 / 60.0, 0.0, &w2, &mut input, 1.0, true);
        }
        assert!(p2.in_water, "should be submerged");
        assert_eq!(p2.take_pending_fall_damage(), 0.0, "water entry is free");
        // flying never accumulates
        let mut p3 = Player::new(Vec3::new(0.5, 120.0, 0.5));
        p3.flying = true;
        p3.vel.y = -30.0;
        for _ in 0..60 {
            let _ = p3.update(1.0 / 60.0, 0.0, &w2, &mut input, 1.0, true);
        }
        assert_eq!(p3.fall_dist, 0.0, "flight resets fall distance");
    }
}

#[cfg(test)]
mod v172_tests {
    use super::*;

    #[test]
    fn pufferfish_poison_cadence_and_floor() {
        // VERIFIED (minecraft.wiki/w/Poison, live 2026-09-06): the
        // observable rate for Poison IV is the 10-tick hurt-immunity
        // floor (1 HP/s), and poison can never kill (floors at 1 HP).
        let mut p = Player::new(glam::Vec3::new(0.0, 80.0, 0.0));
        p.health = 20.0;
        p.apply_pufferfish_poison();
        assert_eq!(p.effects.poison_ticks, 1200); // 1:00

        // 1200 ticks at one damage per 10 ticks = 120 HP of raw poison —
        // far more than 19 available above the floor
        let mut dealt_total = 0.0;
        for _ in 0..1200 {
            dealt_total += p.tick_effects();
        }
        assert!((dealt_total - 19.0).abs() < 1e-6, "total {dealt_total}");
        assert!((p.health - 1.0).abs() < 1e-6, "floors at 1 HP, never kills");
        assert_eq!(p.effects.poison_ticks, 0, "expired");
    }

    #[test]
    fn poison_stops_mid_tick_at_floor() {
        // a short exposure still respects the cadence: 25 ticks of poison
        // = 2 applications (at ticks 10 and 20)
        let mut p = Player::new(glam::Vec3::new(0.0, 80.0, 0.0));
        p.health = 20.0;
        p.apply_pufferfish_poison();
        p.effects.poison_ticks = 25;
        let mut dealt = 0.0;
        for _ in 0..25 {
            dealt += p.tick_effects();
        }
        assert_eq!(dealt, 2.0);
        assert!((p.health - 18.0).abs() < 1e-6);
    }

    #[test]
    fn creative_immune_path_is_game_layered() {
        // the effect TICKS damage even in creative at this layer; the game
        // layer gates on invulnerability (same pattern as drowning) — the
        // effect state itself is what we verify here
        let mut p = Player::new(glam::Vec3::new(0.0, 80.0, 0.0));
        p.apply_pufferfish_poison();
        p.effects.hunger_ticks = 300; // the 0:15 hunger half
        assert_eq!(p.effects.hunger_ticks, 300);
        assert!(p.tick_effects() == 0.0, "first tick is pre-interval");
    }
}

#[cfg(test)]
mod v18_tests {
    use super::*;

    /// 1.8: landing on slime bounces (60% height ratio) and negates fall
    /// damage; sneaking on slime keeps the fall damage (VERIFIED against
    /// the 1.8 changelog §Slime Block).
    #[test]
    fn slime_bounce_and_sneak_damage() {
        // restitution from the 60% height ratio
        assert!((SLIME_RESTITUTION - 0.6_f32.sqrt()).abs() < 1e-4);
        // fall velocity check: a 10-block fall reaches ~13.9 b/s → rebound
        // ≈ 10.78 → apex ≈ 6 blocks = 60% of 10
        let impact = (2.0 * GRAVITY * 10.0).sqrt();
        let rebound = impact * SLIME_RESTITUTION;
        let apex = rebound * rebound / (2.0 * GRAVITY);
        assert!((apex - 6.0).abs() < 0.05, "apex {apex} ≈ 60% of 10");
    }
}
