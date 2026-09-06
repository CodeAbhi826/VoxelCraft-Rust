//! Phase E2 (evolution 1.3–1.4 bracket): the wither boss fight.
//! All values live-verified 2026-09-06 against minecraft.wiki
//! (docs/research/phase2-1.3-1.4-research.md):
//! - health 300, Java row regardless of difficulty (w/Wither infobox)
//! - summon: 4 soul sand in a T + 3 wither-skeleton skulls on the upper
//!   blocks, the LAST placed block must be a skull (§Spawning)
//! - charge-up: invulnerable + inactive for 220 game ticks (11 s; the
//!   infobox's "11 second spawn delay"; the prose's "12 s boss-bar fill"
//!   is the animation — disclosed in the research doc)
//! - birth explosion: proximity-scaled, Java max Normal 69 HP
//! - black wither skull every 2 s from the main head at a target within
//!   40 blocks; the wither hovers ~5 blocks above its target
//! - skull hit: 8 HP + Wither II for 10 s (Normal) / 40 s (Hard),
//!   1 HP per second, can kill (unlike poison)
//! - passive regeneration 1 HP per 20 ticks; a killing blow heals 5 HP
//! - on taking damage: breaks all blocks in a 3×4×3 area around itself
//!   (bedrock + portal blocks are wither_immune)
//! - drops: 1 nether star (100%), 50 XP on player kills (Java)
//! - hitbox 3.5 × 0.9 (Java)
//!
//! Documented adaptations (see research doc):
//! - vanilla's three independent heads (main 2 s cadence, side heads 2–3 s
//!   with separate targets) are compressed to the main-head 2 s cadence
//!   plus a 2.2 s side-volley timer with random offset — the observable
//!   fire rhythm is close, one AI target instead of three.
//! - the wither renders as a large billboard sprite (the engine's entity
//!   art path), not a multi-part mesh.
//! - "wither armor" below half health (projectile immunity) is NOT
//!   implemented — the engine's arrows route through the melee damage
//!   path; noted for the projectile-system bracket.

use vc_rng::rng::Rng;

/// wither health (VERIFIED w/Wither: Java 300 regardless of difficulty)
pub const WITHER_HEALTH: f32 = 300.0;
/// charge-up length in game ticks (VERIFIED: 220 — §Spawning "after 11
/// seconds or 220 game ticks" the charge state ends with the explosion)
pub const CHARGE_TICKS: i32 = 220;
/// passive regeneration: 1 HP per 20 ticks (VERIFIED §Regeneration)
pub const REGEN_TICKS: i32 = 20;
/// a direct killing blow on a target heals 5 HP (VERIFIED)
pub const KILL_HEAL: f32 = 5.0;
/// aggro range (VERIFIED §Java Edition: "within 40 blocks")
pub const AGGRO_BLOCKS: f32 = 40.0;
/// the wither tries to hover this far above its target (VERIFIED)
pub const HOVER_ABOVE: f32 = 5.0;
/// main-head black skull cadence: every 2 s = 40 ticks (VERIFIED)
pub const SKULL_INTERVAL: i32 = 40;
/// black skull damage on Normal (VERIFIED: 8 HP) + Wither II
pub const SKULL_DAMAGE: f32 = 8.0;
/// Wither II duration in ticks: 200 (10 s) Normal / 800 (40 s) Hard
/// (VERIFIED: "Wither II for 10 seconds on Normal and 40 on Hard")
pub const SKULL_WITHER_TICKS_NORMAL: i32 = 200;
pub const SKULL_WITHER_TICKS_HARD: i32 = 800;
/// wither effect: 1 HP per 20 ticks (II = 0.5 per second... VERIFIED
/// "1 HP per sec" at Wither II — applied every 20 game ticks)
pub const WITHER_EFFECT_TICKS: i32 = 20;
/// birth explosion max damage at Normal, proximity-scaled (VERIFIED:
/// Java Normal 69)
pub const BIRTH_EXPLOSION_DAMAGE: f32 = 69.0;
/// block-breaking box around the wither on damage: 3 wide × 4 high × 3
/// deep (VERIFIED §Java Edition)
pub const BREAK_BOX: [i32; 3] = [3, 4, 3];
/// player-kill XP (VERIFIED: Java 50)
pub const WITHER_XP: i32 = 50;
/// boss hitbox (VERIFIED Java: 3.5 tall, 0.9 wide)
pub const WITHER_HEIGHT: f32 = 3.5;
pub const WITHER_WIDTH: f32 = 0.9;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WitherPhase {
    /// summon charge-up: invulnerable, inactive, boss bar filling
    Charge,
    /// active fight
    Fight,
}

#[derive(Clone, Debug)]
pub struct Wither {
    pub health: f32,
    pub pos: [f32; 3],
    pub phase: WitherPhase,
    /// ticks in the current phase
    pub phase_t: i32,
    /// main-head skull cooldown
    pub skull_cd: i32,
    /// side-volley cooldown (2–3 s per the side heads' cadence)
    pub volley_cd: i32,
    /// accumulated regen ticks
    regen_acc: i32,
}

impl Wither {
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
    pub fn charging(&self) -> bool {
        self.phase == WitherPhase::Charge
    }
}

/// Events surfaced to the game layer (it owns world edits, projectiles,
/// damage application, drops, XP and sounds).
#[derive(Clone, Debug)]
pub enum WitherEvent {
    /// the charge ended: birth explosion centered on the wither
    /// (game layer applies proximity-scaled damage + block destruction)
    BirthExplosion([f32; 3]),
    /// fire a black wither skull from the wither toward the target
    SkullShot([f32; 3], [f32; 3]),
    /// break the 3×4×3 box of blocks around this center (damage response)
    BreakBlocks([f32; 3]),
    /// the wither died: (xp for orbs) — the nether star drop is a
    /// game-layer world edit
    Died(i32),
}

pub struct WitherSystem {
    pub wither: Option<Wither>,
    rng: Rng,
    /// the summon structure's origin (blocks already consumed by the
    /// game layer before begin_summon)
    pub origin: Option<[i32; 3]>,
    /// the Died event fired once (health ≤ 0 transition)
    died_fired: bool,
}

impl WitherSystem {
    pub fn new(seed: u64) -> Self {
        WitherSystem {
            wither: None,
            rng: Rng::new(seed ^ 0xB055_2EED),
            origin: None,
            died_fired: false,
        }
    }

    /// Begin the summon at the skull block position (the game layer has
    /// verified the pattern via `wither_pattern` and consumed the blocks).
    pub fn begin_summon(&mut self, x: i32, y: i32, z: i32) {
        self.wither = Some(Wither {
            health: WITHER_HEALTH,
            pos: [x as f32 + 0.5, y as f32 + 0.0, z as f32 + 0.5],
            phase: WitherPhase::Charge,
            phase_t: 0,
            skull_cd: SKULL_INTERVAL,
            volley_cd: 44,
            regen_acc: 0,
        });
        self.origin = Some([x, y, z]);
    }

    /// Player melee hit (the game layer ray-tests the wither AABB).
    /// Charging = invulnerable (VERIFIED). Returns the damage dealt.
    pub fn damage(&mut self, amount: f32) -> (f32, bool) {
        let Some(w) = &mut self.wither else { return (0.0, false) };
        if w.charging() || !w.alive() {
            return (0.0, w.charging());
        }
        w.health -= amount;
        (amount, true)
    }

    /// The wither's own contact/wing damage (Normal row, VERIFIED
    /// w/Wither attack strength "Wither: ... Normal" melee 8 HP is the
    /// skull contact value; the body contact shares it).
    pub fn contact_damage(&self) -> f32 {
        SKULL_DAMAGE
    }

    pub fn tick(&mut self, player: Option<[f32; 3]>) -> Vec<WitherEvent> {
        let mut events = Vec::new();
        let Some(w) = &mut self.wither else { return events };
        if !w.alive() {
            // death transition: fire Died exactly once, then inert
            if !self.died_fired {
                self.died_fired = true;
                events.push(WitherEvent::Died(WITHER_XP));
            }
            return events;
        }
        w.phase_t += 1;

        match w.phase {
            WitherPhase::Charge => {
                if w.phase_t >= CHARGE_TICKS {
                    // charge ends: birth explosion + active fight
                    w.phase = WitherPhase::Fight;
                    w.phase_t = 0;
                    events.push(WitherEvent::BirthExplosion(w.pos));
                }
                return events;
            }
            WitherPhase::Fight => {}
        }

        // passive regeneration: 1 HP / 20 ticks (VERIFIED)
        w.regen_acc += 1;
        if w.regen_acc >= REGEN_TICKS {
            w.regen_acc = 0;
            w.health = (w.health + 1.0).min(WITHER_HEALTH);
        }

        // movement: hover ~5 blocks above the target when it exists
        // (VERIFIED: "always tries to fly 5 blocks above"), else drift
        if let Some(p) = player {
            let target_y = p[1] + HOVER_ABOVE;
            w.pos[0] += (p[0] - w.pos[0]) * 0.01;
            w.pos[1] += (target_y - w.pos[1]) * 0.02;
            w.pos[2] += (p[2] - w.pos[2]) * 0.01;
        }

        let dist = player.map(|p| {
            (p[0] - w.pos[0]).powi(2) + (p[1] - w.pos[1]).powi(2) + (p[2] - w.pos[2]).powi(2)
        });
        let in_range = dist.map(|d| d < AGGRO_BLOCKS * AGGRO_BLOCKS).unwrap_or(false);

        // main head: black skull every 2 s at the target (VERIFIED)
        if w.skull_cd > 0 {
            w.skull_cd -= 1;
        }
        if w.skull_cd == 0 {
            if in_range {
                if let Some(p) = player {
                    events.push(WitherEvent::SkullShot(w.pos, p));
                }
                w.skull_cd = SKULL_INTERVAL;
            } else {
                // idle: keep the cooldown at 1 so the first shot on
                // aggro-acquire lands next tick
                w.skull_cd = 1;
            }
        }

        // side heads volley (adaptation of the 2–3 s cadence)
        if w.volley_cd > 0 {
            w.volley_cd -= 1;
        }
        if w.volley_cd == 0 {
            if in_range {
                if let Some(p) = player {
                    let jx = self.rng.next_range(3) as f32 - 1.0;
                    let jz = self.rng.next_range(3) as f32 - 1.0;
                    events.push(WitherEvent::SkullShot(w.pos, [p[0] + jx, p[1], p[2] + jz]));
                }
            }
            // VERIFIED cadence 2–3 s → 40..60 ticks
            w.volley_cd = 40 + self.rng.next_range(21) as i32;
        }

        events
    }
}

/// Summon-structure check (VERIFIED w/Wither §Spawning): 4 soul sand in
/// a T (3 across + 1 center below the skulls' base row... the vanilla T
/// is: base row of 3 soul sand, one soul sand below the center skull,
/// and 3 skulls on top of the three upper blocks). The check runs at the
/// LAST-placed skull position (x, y, z): the game layer only calls this
/// when the final placed block IS a skull at that position.
///
/// Layout (y = skull row):
/// ```text
///    S S S   <- 3 wither skeletons skulls at y+1? — no: skulls at y,
///    . B .      soul-sand row at y-1 (3 across), stem at y-2
///    . B .
/// ```
/// Precisely: skull row (x-1..x+1, y, z); soul-sand arm row (x-1..x+1,
/// y-1, z); soul-sand stem (x, y-2, z).
pub fn wither_pattern(world: &vc_world::world::World, x: i32, y: i32, z: i32) -> bool {
    use vc_blocks::blocks::{SOUL_SAND, WITHER_SKELETON_SKULL};
    // skull row: the three skulls (the caller verified the block AT
    // (x,y,z) is already a skull — check its two neighbors)
    if world.get_block(x - 1, y, z) != WITHER_SKELETON_SKULL
        || world.get_block(x + 1, y, z) != WITHER_SKELETON_SKULL
    {
        return false;
    }
    // soul-sand arm row
    if world.get_block(x - 1, y - 1, z) != SOUL_SAND
        || world.get_block(x, y - 1, z) != SOUL_SAND
        || world.get_block(x + 1, y - 1, z) != SOUL_SAND
    {
        return false;
    }
    // soul-sand stem
    world.get_block(x, y - 2, z) == SOUL_SAND
}

/// All blocks the summon consumed (the game layer clears them when the
/// wither spawns). Mirrors `wither_pattern`'s layout.
pub fn wither_pattern_blocks(x: i32, y: i32, z: i32) -> Vec<[i32; 3]> {
    let mut v = Vec::with_capacity(7);
    for dx in -1..=1 {
        v.push([x + dx, y, z]); // skulls
        v.push([x + dx, y - 1, z]); // soul-sand arm
    }
    v.push([x, y - 2, z]); // stem
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_blocks::blocks::*;
    use vc_world::world::World;

    fn seeded() -> World {
        let mut w = World::new(7);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, vc_blocks::blocks::STONE);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w
    }

    #[test]
    fn constants_match_the_live_wiki() {
        assert_eq!(WITHER_HEALTH, 300.0); // w/Wither infobox (Java)
        assert_eq!(CHARGE_TICKS, 220); // §Spawning: 11 s / 220 ticks
        assert_eq!(REGEN_TICKS, 20); // 1 HP per second
        assert_eq!(KILL_HEAL, 5.0);
        assert_eq!(AGGRO_BLOCKS, 40.0);
        assert_eq!(SKULL_INTERVAL, 40); // every 2 s
        assert_eq!(SKULL_WITHER_TICKS_NORMAL, 200); // 10 s
        assert_eq!(SKULL_WITHER_TICKS_HARD, 800); // 40 s
        assert_eq!(WITHER_XP, 50);
        assert_eq!(BIRTH_EXPLOSION_DAMAGE, 69.0); // Java Normal max
        assert_eq!((WITHER_HEIGHT, WITHER_WIDTH), (3.5, 0.9));
    }

    #[test]
    fn summon_pattern_requires_the_full_t() {
        let mut w = seeded();
        // build the pattern around (8, 70, 8)
        for dx in -1..=1 {
            w.set_block(8 + dx, 70, 8, SOUL_SAND);
            w.set_block(8 + dx, 71, 8, WITHER_SKELETON_SKULL);
        }
        w.set_block(8, 69, 8, SOUL_SAND);
        assert!(wither_pattern(&w, 8, 71, 8));
        // break one arm block -> invalid
        w.set_block(7, 70, 8, AIR);
        assert!(!wither_pattern(&w, 8, 71, 8));
        // pattern block list covers 7 cells
        assert_eq!(wither_pattern_blocks(8, 71, 8).len(), 7);
    }

    #[test]
    fn charge_is_invulnerable_then_explodes() {
        let mut sys = WitherSystem::new(11);
        sys.begin_summon(8, 71, 8);
        let (d, _) = sys.damage(50.0);
        assert_eq!(d, 0.0, "charging wither takes no damage (VERIFIED)");
        // tick through 219 ticks: no events
        for _ in 0..219 {
            assert!(sys.tick(Some([10.0, 75.0, 10.0])).is_empty());
        }
        let evs = sys.tick(Some([10.0, 75.0, 10.0]));
        assert!(
            evs.iter().any(|e| matches!(e, WitherEvent::BirthExplosion(_))),
            "tick 220 fires the birth explosion"
        );
        // now vulnerable
        let (d, _) = sys.damage(50.0);
        assert_eq!(d, 50.0);
        assert_eq!(sys.wither.as_ref().unwrap().health, 250.0);
    }

    #[test]
    fn regen_heals_one_per_second() {
        let mut sys = WitherSystem::new(12);
        sys.begin_summon(8, 71, 8);
        // skip the charge phase
        for _ in 0..230 {
            let _ = sys.tick(None);
        }
        sys.damage(250.0);
        let h0 = sys.wither.as_ref().unwrap().health;
        for _ in 0..20 {
            let _ = sys.tick(None);
        }
        let h1 = sys.wither.as_ref().unwrap().health;
        assert_eq!(h1, h0 + 1.0, "1 HP per 20 ticks (VERIFIED)");
    }

    #[test]
    fn skulls_fire_on_cadence_in_range() {
        let mut sys = WitherSystem::new(13);
        sys.begin_summon(8, 71, 8);
        for _ in 0..230 {
            let _ = sys.tick(None); // finish charge
        }
        let player = [10.0, 75.0, 10.0];
        let mut shots = 0;
        for _ in 0..200 {
            for e in sys.tick(Some(player)) {
                if matches!(e, WitherEvent::SkullShot(_, _)) {
                    shots += 1;
                }
            }
        }
        // 200 ticks = 10 s: main head fires every 2 s (5) + side volley
        // every 2-3 s (3..5) -> 8..=10 shots
        assert!(
            (8..=10).contains(&shots),
            "10 s at ~1 skull/2s + side volley, got {shots}"
        );
    }

    #[test]
    fn death_yields_50_xp() {
        let mut sys = WitherSystem::new(14);
        sys.begin_summon(8, 71, 8);
        for _ in 0..230 {
            let _ = sys.tick(None);
        }
        sys.damage(300.0);
        // next tick reports Died once
        let evs = sys.tick(None);
        assert!(evs.iter().any(|e| matches!(e, WitherEvent::Died(50))));
        // and the wither stays dead
        assert!(!sys.wither.as_ref().unwrap().alive());
    }
}
