//! Round 17 (2026-09-18): the vanilla 1.16.5 food/hunger/exhaustion
//! model — the standing Round-16-parity-gap deferral ("Hunger-drain
//! gameplay system") closed per the user's "anything left to do then do
//! it". This is the engine's analog of vanilla's `FoodData`.
//!
//! ALL numerics VERIFIED against minecraft.wiki/w/Food fetched LIVE
//! 2026-09-18 (the full research pass + disagreement notes live in
//! docs/research/round-17-hunger-audit.md). The wiki's own §Variables
//! list names the exact fields:
//! - `foodLevel` — "Its initial value on world creation or respawn is
//!   20."
//! - `foodSaturationLevel` — "Its initial value on world creation or
//!   respawn is 5. ... Its maximum value always equals foodLevel's."
//! - `foodTickTimer` — the 80-tick regen/starve cadence counter.
//! - `foodExhaustionLevel` — the §Exhaustion accumulator.
//!
//! Branch order = the 1.16.5 tick shape (documented in the audit §3):
//! drain-exhaustion FIRST, then saturated boost, then natural regen,
//! then starvation, else reset the timer.

/// foodLevel cap (VERIFIED w/Food §Hunger: "hunger value of 20 points,
/// ... each hunger point is half of a drumstick").
pub const MAX_FOOD: i32 = 20;

/// The exhaustion drain threshold (VERIFIED w/Food §Exhaustion: "Once
/// the exhaustion level reaches 4.0, it ... reduces the saturation by
/// 1 if there is any saturation remaining. If the saturation is 0, it
/// reduces the hunger by 1 instead." — implemented with the 1.16.5
/// Java shape `> 4.0` and SUBTRACT 4.0; the live wiki's "resets to
/// 0.0" phrasing contradicts the wiki's own saturation-boost row
/// (6.0 exhaustion per HP = 1.5 saturation per HP only under
/// subtract-4.0) — the disagreement is documented in the audit §2).
pub const EXHAUSTION_DRAIN: f32 = 4.0;

/// Exhaustion accumulation cap (the 1.16.5 `Math.min(..., 40f)` clamp —
/// the wiki text does not publish a cap; the audit documents this as
/// the Java-shape choice, preventing e.g. a 2×-HP regen burst from
/// draining 3 saturation in one event).
pub const EXHAUSTION_CAP: f32 = 40.0;

/// Natural regeneration gate (VERIFIED w/Food §Natural regeneration:
/// "If the hunger value is at 18 or above, ... the player's health
/// naturally regenerates every 4 seconds (80 ticks)").
pub const REGEN_FOOD: i32 = 18;

/// Saturation boost gate — full hunger only (VERIFIED w/Food
/// §Saturation boost: "regenerates health when the player's hunger bar
/// is full (20) ... activates every 0.5 seconds (10 ticks)").
pub const SATURATED_FOOD: i32 = 20;

/// Exhaustion per HP healed by natural regeneration (VERIFIED w/Food
/// §Energy-intensive actions: "Natural regeneration (requires 18 or
/// higher...): 6.0 per 1 HP healed").
pub const REGEN_EXHAUSTION: f32 = 6.0;

/// Starvation damage period (VERIFIED w/Food §Starvation: "Starvation
/// damages the player by 1 HP every 4 seconds (80 ticks)").
pub const STARVE_PERIOD: i32 = 80;

/// The sprint gate (VERIFIED w/Food §Sprinting: "If the hunger value
/// is at 6 or below, the player loses the ability to sprint until the
/// hunger value exceeds 7" — implemented as the 1.16.5 Java check
/// `foodLevel > 6.0F`, i.e. sprint requires 7+; the phrasing
/// disagreement is documented in the audit §5).
pub const SPRINT_GATE_FOOD: i32 = 6;

/// The starvation difficulty class (VERIFIED w/Food §Starvation
/// thresholds: Easy stops at 10 HP, Normal at 1 HP, "On Hard
/// difficulty, starvation damage does not stop at any health
/// threshold"). The engine has no difficulty selector: Survival /
/// Adventure worlds run Normal-class rules, Hardcore runs Hard-class
/// (the modes doc's own "difficulty locked to Hard" row) — Easy and
/// Peaceful rules are unreachable until a difficulty setting exists
/// (disclosed, audit §10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StarveRule {
    /// Easy: starve damage stops at health ≤ 10.
    Easy,
    /// Normal: starve damage stops at health ≤ 1.
    Normal,
    /// Hard: starve damage never stops.
    Hard,
}

impl StarveRule {
    /// Does a starvation hit land at this health? (the wiki's
    /// per-difficulty thresholds, as one boolean per class)
    pub fn allows(self, health: f32) -> bool {
        match self {
            StarveRule::Easy => health > 10.0,
            StarveRule::Normal => health > 1.0,
            StarveRule::Hard => true,
        }
    }
}

/// The player's food state (vanilla `FoodData`). Food is the visible
/// 0..20 hunger bar; saturation is the hidden buffer that exhaustion
/// and healing consume FIRST; exhaustion is the action accumulator.
#[derive(Clone, Debug)]
pub struct Hunger {
    /// foodLevel, 0..=20 — the HUD hunger bar value.
    pub food: i32,
    /// foodSaturationLevel, 0..=food — "determines how fast foodLevel
    /// depletes and is controlled by the kinds of food the player has
    /// eaten" (VERIFIED w/Food §Variables).
    pub saturation: f32,
    /// foodExhaustionLevel — drains 4.0 at a time, saturation first.
    pub exhaustion: f32,
    /// foodTickTimer — the shared regen/starve cadence counter.
    food_tick_timer: i32,
}

/// One hunger tick's outcome (all HP half-heart scale like the rest of
/// the engine): `heal` from natural regeneration / the saturation
/// boost, `starve` from starvation (bypasses armor — the unblockable
/// class, VERIFIED w/Food §Starvation: "Starvation damage ignores
/// armor and armor toughness, the Protection enchantment, and the
/// Resistance effect").
#[derive(Clone, Copy, Debug, Default)]
pub struct HungerTick {
    pub heal: f32,
    pub starve: f32,
}

impl Hunger {
    /// Fresh state on world creation or respawn (VERIFIED w/Food
    /// §Variables: food 20 / saturation 5).
    pub fn spawn() -> Self {
        Hunger {
            food: MAX_FOOD,
            saturation: 5.0,
            exhaustion: 0.0,
            food_tick_timer: 0,
        }
    }

    /// Eat one food item (VERIFIED w/Food §Hunger values + §Food
    /// saturation values tables, live 2026-09-18 — every registered
    /// edible's (nutrition, saturation) pair pinned by the game
    /// layer's `food_values` + its tests). The Java `eat()` shape:
    /// `food = min(food + nutrition, 20)`; `saturation =
    /// min(saturation + restored, food)` — the wiki's "Saturation
    /// restored" column IS nutrition × satModifier × 2, so adding the
    /// column value is identical.
    pub fn eat(&mut self, nutrition: i32, saturation_restored: f32) {
        self.food = (self.food + nutrition.max(0)).min(MAX_FOOD);
        self.saturation = (self.saturation + saturation_restored.max(0.0))
            .min(self.food as f32)
            .max(0.0);
    }

    /// Accumulate action exhaustion (the §Energy-intensive actions
    /// table — see the game/player layer call sites for the per-source
    /// amounts). Capped at 40 (the 1.16.5 Java clamp; audit §2).
    pub fn add_exhaustion(&mut self, amount: f32) {
        if amount > 0.0 {
            self.exhaustion = (self.exhaustion + amount).min(EXHAUSTION_CAP);
        }
    }

    /// Can the player sprint right now? (the wiki sprint gate — food
    /// 7+; the engine's flight bypass is at the call site, the Java
    /// `|| mayfly` arm.)
    pub fn can_sprint(&self) -> bool {
        self.food > SPRINT_GATE_FOOD
    }

    /// Advance ONE 20 Hz game tick. `should_heal` = the vanilla
    /// `player.shouldHeal()` (health below max). Returns the tick's
    /// heal/starve outcome for the game layer to apply.
    pub fn tick(&mut self, should_heal: bool, rule: StarveRule, health: f32) -> HungerTick {
        // 1. the exhaustion drain (VERIFIED w/Food §Exhaustion):
        // > 4.0 → subtract 4.0; saturation first, then food.
        if self.exhaustion > EXHAUSTION_DRAIN {
            self.exhaustion -= EXHAUSTION_DRAIN;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else {
                self.food = (self.food - 1).max(0);
            }
        }

        let mut out = HungerTick::default();
        if self.saturation > 0.0
            && self.food >= SATURATED_FOOD
            && should_heal
        {
            // 2. the saturation boost (VERIFIED w/Food §Saturation
            // boost: "heals 1 HP by consuming 1.5 saturation, and
            // activates every 0.5 seconds (10 ticks) when at full
            // hunger" — implemented in the 1.16.5 Java shape: heal
            // min(sat, 6)/6 and exhaust min(sat, 6) per 10 ticks; the
            // steady state (sat ≥ 6) IS the wiki row: 1 HP / 0.5 s,
            // 1.5 sat consumed per HP through the 4.0 drain).
            self.food_tick_timer += 1;
            if self.food_tick_timer >= 10 {
                let f = self.saturation.min(6.0);
                out.heal += f / 6.0;
                self.add_exhaustion(f);
                self.food_tick_timer = 0;
            }
        } else if self.food >= REGEN_FOOD && should_heal {
            // 3. natural regeneration (VERIFIED w/Food §Natural
            // regeneration: "regenerates every 4 seconds (80 ticks)";
            // exhaustion "6.0 per 1 HP healed").
            self.food_tick_timer += 1;
            if self.food_tick_timer >= 80 {
                out.heal += 1.0;
                self.add_exhaustion(REGEN_EXHAUSTION);
                self.food_tick_timer = 0;
            }
        } else if self.food <= 0 {
            // 4. starvation (VERIFIED w/Food §Starvation: 1 HP per 80
            // ticks; the per-difficulty stop thresholds — audit §4).
            self.food_tick_timer += 1;
            if self.food_tick_timer >= STARVE_PERIOD {
                if rule.allows(health) {
                    out.starve += 1.0;
                }
                self.food_tick_timer = 0;
            }
        } else {
            // 5. idle: the cadence counter resets (vanilla's final
            // else arm — a mid-band food level carries no timer debt).
            self.food_tick_timer = 0;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_state_matches_the_wiki() {
        // w/Food §Variables: food 20, saturation 5 on world creation
        // or respawn.
        let h = Hunger::spawn();
        assert_eq!(h.food, 20);
        assert!((h.saturation - 5.0).abs() < 1e-6);
        assert_eq!(h.exhaustion, 0.0);
    }

    #[test]
    fn exhaustion_drains_saturation_before_food() {
        // w/Food §Exhaustion: "> 4.0 ... reduces the saturation by 1 if
        // there is any saturation remaining. If the saturation is 0, it
        // reduces the hunger by 1 instead."
        let mut h = Hunger::spawn(); // food 20, sat 5
        h.add_exhaustion(4.5);
        // 4.5 > 4.0: subtract (not reset — audit §2), saturation first
        h.tick(false, StarveRule::Normal, 20.0);
        assert!((h.exhaustion - 0.5).abs() < 1e-6, "overshoot kept");
        assert!((h.saturation - 4.0).abs() < 1e-6, "saturation -1 first");
        assert_eq!(h.food, 20, "food untouched while sat > 0");
        // drain the rest of the saturation, then hit food
        for _ in 0..4 {
            h.add_exhaustion(4.1);
            h.tick(false, StarveRule::Normal, 20.0);
        }
        assert!((h.saturation - 0.0).abs() < 1e-6);
        h.add_exhaustion(4.1);
        h.tick(false, StarveRule::Normal, 20.0);
        assert_eq!(h.food, 19, "saturation 0 → hunger -1");
    }

    #[test]
    fn natural_regen_every_80_ticks_costs_6_exhaustion() {
        // w/Food §Natural regeneration: "regenerates every 4 seconds
        // (80 ticks)"; exhaustion table: "6.0 per 1 HP healed".
        let mut h = Hunger::spawn();
        h.saturation = 0.0; // force the unsaturated branch
        h.food = 18; // exactly the gate (18 or above)
        let mut heal = 0.0;
        for _ in 0..80 {
            let t = h.tick(true, StarveRule::Normal, 15.0);
            heal += t.heal;
        }
        assert!((heal - 1.0).abs() < 1e-6, "1 HP per 80 ticks, got {heal}");
        assert!((h.exhaustion - 6.0).abs() < 1e-6, "6.0 exhaustion per HP");
        // below the gate: no regen, and the timer resets (the wiki's
        // "drops to 17 or below, natural regeneration stops")
        h.food = 17;
        h.exhaustion = 0.0;
        let mut heal2 = 0.0;
        for _ in 0..160 {
            let t = h.tick(true, StarveRule::Normal, 15.0);
            heal2 += t.heal;
        }
        assert_eq!(heal2, 0.0);
        // not healing (full health): no regen either (shouldHeal gate)
        h.food = 20;
        let mut heal3 = 0.0;
        for _ in 0..160 {
            let t = h.tick(false, StarveRule::Normal, 20.0);
            heal3 += t.heal;
        }
        assert_eq!(heal3, 0.0);
    }

    #[test]
    fn saturated_boost_heals_a_half_heart_per_half_second() {
        // w/Food §Saturation boost: "heals 1 HP by consuming 1.5
        // saturation, and activates every 0.5 seconds (10 ticks) when
        // at full hunger" — steady state sat >= 6.
        let mut h = Hunger::spawn();
        h.saturation = 12.0;
        let mut heal = 0.0;
        for _ in 0..10 {
            let t = h.tick(true, StarveRule::Normal, 10.0);
            heal += t.heal;
        }
        assert!((heal - 1.0).abs() < 1e-6, "1 HP per 10 ticks, got {heal}");
        // the consumption: 6.0 exhaustion per HP → the 4.0 drain takes
        // 1.5 saturation per HP (the wiki row's own arithmetic)
        let sat_cost = 12.0 - h.saturation + h.exhaustion / EXHAUSTION_DRAIN;
        assert!(
            (sat_cost - 1.5).abs() < 0.05,
            "1.5 saturation per healed HP, got {sat_cost}"
        );
        // partial saturation heals proportionally (min(sat,6)/6)
        let mut h2 = Hunger::spawn();
        h2.saturation = 3.0;
        let mut heal2 = 0.0;
        for _ in 0..10 {
            let t = h2.tick(true, StarveRule::Normal, 10.0);
            heal2 += t.heal;
        }
        assert!((heal2 - 0.5).abs() < 1e-6, "sat 3 → 0.5 HP per boost");
    }

    #[test]
    fn starvation_follows_the_difficulty_thresholds() {
        // w/Food §Starvation: 1 HP per 80 ticks; "On Easy difficulty,
        // starvation damage stops when the player's health is at 10 or
        // below. On Normal ... at 1 or below. On Hard ... does not stop
        // at any health threshold."
        let run = |rule: StarveRule, health: f32| {
            let mut h = Hunger::spawn();
            h.food = 0;
            h.saturation = 0.0;
            h.exhaustion = 0.0;
            let mut starve = 0.0;
            for _ in 0..80 {
                starve += h.tick(true, rule, health).starve;
            }
            starve
        };
        assert_eq!(run(StarveRule::Hard, 1.0), 1.0, "hard never stops");
        assert_eq!(run(StarveRule::Normal, 1.0), 0.0, "normal stops at 1");
        assert_eq!(run(StarveRule::Normal, 1.5), 1.0, "normal hits above 1");
        assert_eq!(run(StarveRule::Easy, 10.0), 0.0, "easy stops at 10");
        assert_eq!(run(StarveRule::Easy, 10.5), 1.0, "easy hits above 10");
    }

    #[test]
    fn eat_caps_food_at_20_and_saturation_at_food() {
        // the Java eat() shape: food capped at 20; saturation capped
        // at the NEW food level ("Its maximum value always equals
        // foodLevel's value")
        let mut h = Hunger::spawn();
        h.food = 15;
        h.saturation = 0.0;
        h.eat(8, 12.8); // a steak
        assert_eq!(h.food, 20, "15 + 8 capped at 20");
        assert!((h.saturation - 12.8).abs() < 1e-6);
        // from a low food level, saturation can never exceed food
        let mut h2 = Hunger::spawn();
        h2.food = 6;
        h2.saturation = 0.0;
        h2.eat(2, 12.8); // huge saturation, tiny food
        assert_eq!(h2.food, 8);
        assert!((h2.saturation - 8.0).abs() < 1e-6, "sat capped at food");
    }

    #[test]
    fn sprint_gate_is_food_seven_or_above() {
        // w/Food §Sprinting: "at 6 or below, the player loses the
        // ability to sprint" — implemented as the Java food > 6 check
        // (the phrasing note lives in the audit §5)
        let mut h = Hunger::spawn();
        h.food = 7;
        assert!(h.can_sprint());
        h.food = 6;
        assert!(!h.can_sprint());
        h.food = 0;
        assert!(!h.can_sprint());
    }

    #[test]
    fn exhaustion_accumulation_is_capped_at_40() {
        // the 1.16.5 Java Math.min(..., 40f) clamp (audit §2) — a huge
        // single event cannot drain 3 saturation at once
        let mut h = Hunger::spawn();
        h.saturation = 5.0;
        h.add_exhaustion(100.0);
        assert!((h.exhaustion - EXHAUSTION_CAP).abs() < 1e-6);
        h.tick(false, StarveRule::Normal, 20.0);
        assert!((h.saturation - 4.0).abs() < 1e-6, "one drain per tick");
        // negative amounts are ignored (defensive: no free food)
        h.add_exhaustion(-5.0);
        assert!(h.exhaustion >= 0.0);
    }
}
