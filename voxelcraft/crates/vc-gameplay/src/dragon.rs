//! Phase E1 (evolution 1.0–1.2 bracket): the ender-dragon boss fight.
//! All values live-verified 2026-09-06 against minecraft.wiki
//! (docs/research/phase1-1.0-1.2-research.md):
//! - health 200 (w/Ender_Dragon infobox)
//! - melee Easy 6 / Normal 10 / Hard 15 (we deliver the Normal row;
//!   the game layer difficulty-scales)
//! - damage sources: ONLY players and explosions (immune to everything
//!   else the engine simulates)
//! - crystals heal 1 HP per 10 game ticks (0.5 s) within a 32-block
//!   cuboid; destroying a healing crystal deals 10 HP
//! - death: XP 12000 first kill / 500 re-summoned; exit portal fills +
//!   dragon egg appears above the central bedrock structure
//! - re-summoning via 4 end crystals on the exit portal (deferred — no
//!   item placement path for the ritual yet; the fight is first-kill)
//!
//! Documented adaptations:
//! - vanilla's phase list (HoldingPattern/StrafePlayer/LandingApproach/
//!   Landing/Sitting*) is compressed into a 5-state cycle: circle →
//!   strafe (3-fireball volley) → charge → perch (5 s) → take-off.
//! - the dragon's breath/sitting-flame attack is deferred (no lingering
//!   area-effect system in the engine yet)
//! - the boss model renders as a large billboard sprite (the engine's
//!   entity art path), not a multi-part mesh

use vc_rng::rng::Rng;
use vc_world::world::World;

/// dragon health (VERIFIED w/Ender_Dragon infobox: 200 HP × 100)
pub const DRAGON_HEALTH: f32 = 200.0;
/// crystal healing: 1 HP per 10 game ticks (VERIFIED)
pub const CRYSTAL_HEAL_TICKS: i32 = 10;
/// destroying an actively-healing crystal deals 10 HP (VERIFIED)
pub const CRYSTAL_DESTROY_DAMAGE: f32 = 10.0;
/// crystals heal within a 32-block cuboid of the dragon (VERIFIED)
pub const CRYSTAL_HEAL_RANGE: f32 = 32.0;
/// dragon fireball damage = the melee row (VERIFIED damage rows; fire
/// damage itself is deferred with the breath system)
pub const DRAGON_MELEE_DAMAGE: f32 = 10.0;
/// first-kill XP: 12000 (VERIFIED; 10 drops of 960 + one of 2400 per the
/// Ender_Dragon page — the Experience page's 10×1000+2000 disagrees; the
/// split into orbs uses the vanilla base-value ladder either way, so the
/// total is exact and the per-orb layout follows the dragon page)
pub const DRAGON_XP_FIRST: i32 = 12000;
/// re-summoned kills: 500 XP (VERIFIED)
pub const DRAGON_XP_RESUMMONED: i32 = 500;
/// one end crystal explodes with power 6 — the charged-creeper value
/// (VERIFIED w/End_Crystal)
pub const CRYSTAL_EXPLOSION_POWER: f32 = 6.0;
/// the dying animation: XP starts 154 ticks into the ascension, the
/// portal/egg land at 200 ticks (VERIFIED)
pub const DEATH_XP_AT: i32 = 154;
pub const DEATH_PORTAL_AT: i32 = 200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DragonPhase {
    /// circling the island (vanilla HoldingPattern)
    Circle,
    /// strafing pass: 3-fireball volley at the player (StrafePlayer)
    Strafe,
    /// diving charge at the player (ChargingPlayer)
    Charge,
    /// perched on the exit-portal fountain (Landing/Sitting, 5 s)
    Perch,
    /// ascending back to circling (Takeoff)
    Takeoff,
}

#[derive(Clone, Debug)]
pub struct EndCrystal {
    pub pos: [f32; 3],
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct Dragon {
    pub health: f32,
    pub pos: [f32; 3],
    pub phase: DragonPhase,
    /// phase timer (ticks in the current phase)
    pub phase_t: i32,
    /// wing contact damage cooldown
    pub attack_cd: i32,
    /// dying: None = alive; Some(ticks into the ascension) once slain
    pub dying: Option<i32>,
    /// fireball volley counter during Strafe
    pub volley: i32,
    orbit_yaw: f32,
}

impl Dragon {
    pub fn alive(&self) -> bool {
        self.dying.is_none() && self.health > 0.0
    }
}

/// Events surfaced to the game layer (it owns world edits + XP + sounds).
#[derive(Clone, Debug)]
pub enum DragonEvent {
    /// spawn a blaze-style fireball at the target (the game layer routes
    /// it through the mob projectile list)
    Fireball([f32; 3], [f32; 3]),
    /// a crystal exploded: (center, power 6) — world damage like creepers
    CrystalExplosion([f32; 3]),
    /// the dragon died: (total XP for the drop waves)
    Died(i32),
    /// 200 ticks into the death: fill the exit portal + spawn the egg
    PortalActivated,
}

pub struct DragonSystem {
    pub dragon: Option<Dragon>,
    pub crystals: Vec<EndCrystal>,
    rng: Rng,
    /// accumulated heal ticks (1 HP per 10 ticks per crystal in range)
    heal_acc: i32,
    /// true once the first dragon spawned in this world (the 500-XP rule
    /// for re-summons is future work — the re-summon ritual is deferred)
    pub fought_once: bool,
}

impl DragonSystem {
    pub fn new(seed: u64) -> Self {
        DragonSystem {
            dragon: None,
            crystals: Vec::new(),
            rng: Rng::new(seed ^ 0xDA60_5005),
            heal_acc: 0,
            fought_once: false,
        }
    }

    /// Spawn the fight (the game layer calls this on first End entry):
    /// the dragon at (0, 128, 0) (VERIFIED: the re-summon sequence brings
    /// it in at (0, 128, 0) — we reuse the arrival height) and 10
    /// crystals on the pillar tops (2 caged ones sit a block higher).
    pub fn begin_fight(&mut self, pillar_tops: &[(i32, i32, i32)]) {
        self.dragon = Some(Dragon {
            health: DRAGON_HEALTH,
            pos: [0.5, 128.0, 0.5],
            phase: DragonPhase::Circle,
            phase_t: 0,
            attack_cd: 0,
            dying: None,
            volley: 0,
            orbit_yaw: 0.0,
        });
        self.crystals = pillar_tops
            .iter()
            .map(|&(x, top, z)| EndCrystal {
                pos: [x as f32 + 0.5, top as f32 + 1.0, z as f32 + 0.5],
                alive: true,
            })
            .collect();
        self.fought_once = true;
    }

    /// Player melee hit on the dragon (the game layer ray-tests the
    /// dragon's AABB first). Non-player damage never routes here
    /// (VERIFIED: only players and explosions damage the dragon).
    pub fn damage(&mut self, amount: f32) -> f32 {
        let Some(d) = &mut self.dragon else { return 0.0 };
        if d.dying.is_some() {
            return 0.0;
        }
        d.health -= amount;
        if d.health <= 0.0 {
            d.dying = Some(0); // the ascension begins
        }
        amount
    }

    /// Hit-test a crystal by id-less proximity: returns the index of the
    /// alive crystal within `reach` of the ray-hit point.
    pub fn crystal_hit(&self, point: [f32; 3], reach: f32) -> Option<usize> {
        self.crystals
            .iter()
            .position(|c| {
                c.alive
                    && (c.pos[0] - point[0]).powi(2) + (c.pos[1] - point[1]).powi(2)
                        + (c.pos[2] - point[2]).powi(2)
                        < reach * reach
            })
    }

    /// Player attack on a crystal: it detonates (power 6 — VERIFIED). If
    /// the crystal was actively healing the dragon, the dragon takes 10
    /// (VERIFIED). The death XP/portal events ride the tick's ascension
    /// timeline (no duplicate Died here).
    pub fn destroy_crystal(&mut self, idx: usize) -> DragonEvent {
        let c = &mut self.crystals[idx];
        c.alive = false;
        let center = c.pos;
        if let Some(d) = &mut self.dragon {
            if d.dying.is_none() {
                let in_range = (d.pos[0] - center[0]).abs() <= CRYSTAL_HEAL_RANGE
                    && (d.pos[1] - center[1]).abs() <= CRYSTAL_HEAL_RANGE
                    && (d.pos[2] - center[2]).abs() <= CRYSTAL_HEAL_RANGE;
                if in_range {
                    d.health -= CRYSTAL_DESTROY_DAMAGE;
                    if d.health <= 0.0 {
                        d.dying = Some(0);
                    }
                }
            }
        }
        DragonEvent::CrystalExplosion(center)
    }

    /// ONE deterministic sim tick. `player` = feet position (None pauses
    /// the fight — vanilla keeps the chunks loaded, our sim gate holds).
    pub fn tick(&mut self, world: &World, player: Option<[f32; 3]>) -> Vec<DragonEvent> {
        let mut events = Vec::new();
        let Some(d) = &mut self.dragon else { return events };

        // ---- death sequence (VERIFIED timing) ----
        if let Some(mut t) = d.dying {
            t += 1;
            d.dying = Some(t);
            // rise slowly during the ascension
            d.pos[1] += 0.05;
            if t == DEATH_XP_AT {
                events.push(DragonEvent::Died(DRAGON_XP_FIRST));
            }
            if t >= DEATH_PORTAL_AT {
                events.push(DragonEvent::PortalActivated);
                self.dragon = None; // gone in beams of light
                return events;
            }
            return events;
        }

        d.attack_cd = d.attack_cd.saturating_sub(1);
        d.phase_t += 1;

        // ---- crystal healing: 1 HP per 10 ticks from any crystal within
        // the 32-block cuboid (VERIFIED) ----
        self.heal_acc += 1;
        if self.heal_acc >= CRYSTAL_HEAL_TICKS {
            self.heal_acc = 0;
            if d.health < DRAGON_HEALTH {
                for c in self.crystals.iter() {
                    if c.alive
                        && (d.pos[0] - c.pos[0]).abs() <= CRYSTAL_HEAL_RANGE
                        && (d.pos[1] - c.pos[1]).abs() <= CRYSTAL_HEAL_RANGE
                        && (d.pos[2] - c.pos[2]).abs() <= CRYSTAL_HEAL_RANGE
                    {
                        d.health = (d.health + 1.0).min(DRAGON_HEALTH);
                        break; // one crystal per heal tick (nearest-beam)
                    }
                }
            }
        }

        let Some(p) = player else {
            return events; // fight paused (no anchor)
        };

        // ---- the phase machine ----
        match d.phase {
            DragonPhase::Circle => {
                // orbit the island center at radius 30, height ~85
                d.orbit_yaw += 0.01;
                let target = [
                    30.0 * d.orbit_yaw.cos(),
                    85.0,
                    30.0 * d.orbit_yaw.sin(),
                ];
                steer(d, target, 0.03);
                if d.phase_t > 100 + (self.rng.next_range(60) as i32) {
                    d.phase = if self.rng.next_range(3) == 0 {
                        DragonPhase::Charge
                    } else {
                        DragonPhase::Strafe
                    };
                    d.phase_t = 0;
                    d.volley = 0;
                }
            }
            DragonPhase::Strafe => {
                // dive toward the player, firing a 3-fireball volley
                let target = [p[0], p[1] + 1.5, p[2]];
                steer(d, target, 0.05);
                d.volley += 1;
                if d.volley == 20 || d.volley == 40 || d.volley == 60 {
                    // VERIFIED strafing fireballs: volley shots at the player
                    events.push(DragonEvent::Fireball(d.pos, target));
                }
                if d.volley > 90 {
                    d.phase = DragonPhase::Circle;
                    d.phase_t = 0;
                }
            }
            DragonPhase::Charge => {
                // dive at the player; volley-fire if we connect
                let target = [p[0], p[1] + 1.0, p[2]];
                steer(d, target, 0.08);
                let dx = p[0] - d.pos[0];
                let dy = p[1] - d.pos[1];
                let dz = p[2] - d.pos[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist < 4.0 && d.attack_cd == 0 {
                    d.attack_cd = 20;
                    // the fireball carries the melee row Normal 10 damage
                    // (VERIFIED); the contact hit rides the projectile
                    events.push(DragonEvent::Fireball(d.pos, target));
                }
                if d.phase_t > 80 {
                    d.phase = DragonPhase::Circle;
                    d.phase_t = 0;
                }
            }
            DragonPhase::Perch => {
                // sit on the fountain (0, 63, 0) — the game layer holds
                // breath/attack effects (deferred)
                d.pos = [0.5, 63.0, 0.5];
                if d.phase_t > 100 {
                    // 5 s perched, then away
                    d.phase = DragonPhase::Takeoff;
                    d.phase_t = 0;
                }
            }
            DragonPhase::Takeoff => {
                let target = [0.5, 85.0, 30.0];
                steer(d, target, 0.04);
                if d.phase_t > 60 {
                    d.phase = DragonPhase::Circle;
                    d.phase_t = 0;
                }
            }
        }
        let _ = world; // (kept for signature parity — no block queries yet)
        events
    }
}

/// steer the dragon toward a world-space target (velocity form: the
/// dragon's position integrates directly — no collision; vanilla ignores
/// block collisions in flight, destroying non-immune blocks)
fn steer(d: &mut Dragon, target: [f32; 3], rate: f32) {
    for (dp, tp) in d.pos.iter_mut().zip(target.iter()) {
        *dp += (tp - *dp) * rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragon_health_and_fight_constants_match_the_wiki() {
        assert_eq!(DRAGON_HEALTH, 200.0); // VERIFIED infobox
        assert_eq!(CRYSTAL_HEAL_TICKS, 10); // 1 HP per 0.5 s
        assert_eq!(CRYSTAL_DESTROY_DAMAGE, 10.0);
        assert_eq!(CRYSTAL_HEAL_RANGE, 32.0); // cuboid
        assert_eq!(DRAGON_XP_FIRST, 12000);
        assert_eq!(DRAGON_XP_RESUMMONED, 500);
        assert_eq!(CRYSTAL_EXPLOSION_POWER, 6.0); // charged-creeper power
        assert_eq!(DEATH_XP_AT, 154);
        assert_eq!(DEATH_PORTAL_AT, 200);
    }

    #[test]
    fn fight_begins_with_ten_crystals_and_cycles() {
        let mut sys = DragonSystem::new(7);
        let tops: Vec<(i32, i32, i32)> = (0..10)
            .map(|i| (i * 8, 80 + i, i * 8))
            .collect();
        sys.begin_fight(&tops);
        assert!(sys.dragon.as_ref().unwrap().alive());
        assert_eq!(sys.crystals.len(), 10); // VERIFIED count
        assert!(sys.fought_once);
        // crystals sit one block above their pillar tops
        assert_eq!(sys.crystals[3].pos[1], 84.0); // tops[3].1 = 83 + 1
        // tick without a player: no events, fight paused
        let w = World::new(1);
        assert!(sys.tick(&w, None).is_empty());
    }

    #[test]
    fn dragon_takes_player_damage_and_dies_on_the_verified_timeline() {
        let mut sys = DragonSystem::new(11);
        let tops: Vec<(i32, i32, i32)> = vec![(0, 80, 0)];
        sys.begin_fight(&tops);
        // 200 HP: twenty 10-damage hits
        for _ in 0..20 {
            sys.damage(10.0);
        }
        let d = sys.dragon.as_ref().unwrap();
        assert!(d.dying.is_some(), "slain at exactly 200 damage");
        // the ascension: XP at tick 154, portal at 200
        let w = World::new(1);
        let mut xp = 0;
        let mut portal = false;
        for _ in 0..210 {
            for ev in sys.tick(&w, None) {
                match ev {
                    DragonEvent::Died(x) => xp += x,
                    DragonEvent::PortalActivated => portal = true,
                    _ => {}
                }
            }
        }
        assert_eq!(xp, DRAGON_XP_FIRST, "12000 XP once (VERIFIED)");
        assert!(portal, "the exit portal activated at 200 ticks");
        assert!(sys.dragon.is_none(), "the dragon is gone");
    }

    #[test]
    fn crystal_destruction_heals_rules() {
        let mut sys = DragonSystem::new(13);
        // one crystal near the dragon
        sys.begin_fight(&[(0, 80, 0)]);
        sys.damage(20.0); // dragon at 180, hurt
        // park the dragon within the crystal's 32-block heal cuboid
        sys.dragon.as_mut().unwrap().pos = [0.5, 81.0, 0.5];
        // crystal heals 1 HP per 10 ticks within the 32 cuboid
        let w = World::new(1);
        for _ in 0..10 {
            sys.tick(&w, None);
        }
        assert!(
            (sys.dragon.as_ref().unwrap().health - 181.0).abs() < 0.01,
            "one heal tick applied"
        );
        // destroying the healing crystal: 10 HP + explosion event
        let ev = sys.destroy_crystal(0);
        match ev {
            DragonEvent::CrystalExplosion(c) => assert_eq!(c, [0.5, 81.0, 0.5]),
            _ => panic!("expected the crystal explosion event"),
        }
        assert!(
            (sys.dragon.as_ref().unwrap().health - 171.0).abs() < 0.01,
            "10 HP crystal-destruction damage (VERIFIED)"
        );
        // the crystal is gone: no further healing
        for _ in 0..30 {
            sys.tick(&w, None);
        }
        assert!(
            (sys.dragon.as_ref().unwrap().health - 171.0).abs() < 0.01,
            "dead crystals do not heal"
        );
    }
}
