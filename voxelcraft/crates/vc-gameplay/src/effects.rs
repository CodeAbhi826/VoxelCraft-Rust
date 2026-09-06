//! Phase E2 (evolution 1.3–1.4 bracket): a minimal timed status-effect
//! system. This bracket needs EXACTLY these effects:
//! - Wither (1.4's signature effect — wither skeleton hits + wither
//!   skulls): damage 1 HP per 20 ticks at level II (VERIFIED
//!   w/Wither_Skeleton: "1 HP every two seconds"; w/Wither: "1 HP per
//!   sec" — Wither II ticks every 20 game ticks, 0.5 hearts)
//! - Poison (witch splash potions): 1 HP per 25 ticks at level I,
//!   cannot kill (floors at 1 HP — VERIFIED w/Effect §Poison)
//! - Regeneration (beacon secondary power): 1 HP per 50 ticks at level
//!   I (VERIFIED w/Effect §Regeneration: level I every 2.5 s)
//! - Speed / Haste / Resistance / Jump Boost / Strength (beacon primary
//!   powers — stat modifiers, VERIFIED w/Beacon §Powers)
//!
//! Design: one flat table keyed by effect kind with (level, ticks_left).
//! The player tick applies periodic damage/heal and stat modifiers are
//! read by the movement/combat code through `amplifier()` lookups.
//! Vanilla's full effect stack (particles, HUD icons, /effect command,
//! ~30 effects) is out of scope — this is the minimal set the 1.3–1.4
//! content requires, disclosed in the worklog.

/// Effect kinds the engine simulates (vanilla registry names as
/// mechanical data).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Wither,
    Poison,
    Regeneration,
    Speed,
    Haste,
    Resistance,
    JumpBoost,
    Strength,
    /// 1.9/1.10 bracket (stray tipped arrows): movement penalty
    /// (VERIFIED w/Effect §Slowness: −15% per level)
    Slowness,
    /// 1.10 bracket (husk hits apply Hunger 7 s × regional difficulty;
    /// VERIFIED w/Husk) — food-poisoning drain flag
    Hunger,
}

impl EffectKind {
    pub fn name(self) -> &'static str {
        match self {
            EffectKind::Wither => "minecraft:wither",
            EffectKind::Poison => "minecraft:poison",
            EffectKind::Regeneration => "minecraft:regeneration",
            EffectKind::Speed => "minecraft:speed",
            EffectKind::Haste => "minecraft:haste",
            EffectKind::Resistance => "minecraft:resistance",
            EffectKind::JumpBoost => "minecraft:jump_boost",
            EffectKind::Strength => "minecraft:strength",
            EffectKind::Slowness => "minecraft:slowness",
            EffectKind::Hunger => "minecraft:hunger",
        }
    }
}

/// One active effect: amplifier 0 = level I, 1 = level II (vanilla
/// convention); ticks_left counts DOWN at 20 Hz.
#[derive(Clone, Copy, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub amplifier: u8,
    pub ticks_left: i32,
}

/// Damage/heal period per kind (VERIFIED w/Effect rows):
/// - Wither II: every 20 ticks (1 HP)
/// - Poison I: every 25 ticks (1 HP, cannot kill)
/// - Regeneration I: every 50 ticks (1 HP)
pub fn period_ticks(kind: EffectKind, amplifier: u8) -> i32 {
    match kind {
        // Wither I: 1 HP per 40 ticks (2 s — w/Wither_Skeleton phrasing);
        // Wither II: per 20 ticks (1 s — w/Wither row)
        EffectKind::Wither => 40 >> (amplifier as i32).min(1),
        // Poison I: per 25 ticks (1.25 s — w/Effect). Raw cadence halves
        // per level (25 >> amplifier), but the 10-tick hurt-immunity
        // window floors the EFFECTIVE cadence at 10 ticks (VERIFIED live
        // 2026-09-06, w/Poison: level IV lists 3 ticks/HP raw, ~1 HP/s
        // effective) — the floor models the immunity window.
        EffectKind::Poison => (25 >> (amplifier as i32)).max(10),
        // Regeneration I: per 50 ticks (2.5 s — w/Effect)
        EffectKind::Regeneration => 50 >> (amplifier as i32).min(1),
        _ => i32::MAX, // stat effects are continuous, no period
    }
}

/// The effect-holder state (the player; witches drink potions
/// engine-side through instant amounts instead of this table).
#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub active: Vec<Effect>,
    /// tick accumulators per kind (parallel to `active`)
    acc: Vec<i32>,
}

impl Effects {
    pub fn new() -> Self {
        Effects::default()
    }

    /// Apply (or refresh) an effect: a stronger/longer application wins
    /// (vanilla: the higher amplifier wins; ties → longer duration).
    pub fn apply(&mut self, kind: EffectKind, amplifier: u8, ticks: i32) {
        if let Some(i) = self.active.iter().position(|e| e.kind == kind) {
            let e = &mut self.active[i];
            if amplifier > e.amplifier || (amplifier == e.amplifier && ticks > e.ticks_left) {
                e.amplifier = amplifier;
                e.ticks_left = ticks;
                self.acc[i] = 0;
            }
        } else {
            self.active.push(Effect {
                kind,
                amplifier,
                ticks_left: ticks,
            });
            self.acc.push(0);
        }
    }

    /// current amplifier for a kind (None = not active)
    pub fn amplifier(&self, kind: EffectKind) -> Option<u8> {
        self.active
            .iter()
            .find(|e| e.kind == kind && e.ticks_left > 0)
            .map(|e| e.amplifier)
    }

    /// Advance one game tick. Returns (damage, heal) to apply this tick:
    /// damage > 0 from wither/poison; heal > 0 from regeneration.
    /// `health` is the holder's current HP (poison floors at 1).
    pub fn tick(&mut self, health: f32) -> (f32, f32) {
        let mut dmg = 0.0;
        let mut heal = 0.0;
        // iterate by index: expiry removes by swap
        let mut i = 0;
        while i < self.active.len() {
            let e = self.active[i];
            if e.ticks_left <= 0 {
                self.active.swap_remove(i);
                self.acc.swap_remove(i);
                continue;
            }
            self.active[i].ticks_left -= 1;
            self.acc[i] += 1;
            let period = period_ticks(e.kind, e.amplifier);
            if self.acc[i] >= period {
                self.acc[i] = 0;
                match e.kind {
                    EffectKind::Wither => {
                        // can kill (unlike poison — VERIFIED)
                        dmg += 1.0;
                    }
                    EffectKind::Poison => {
                        // cannot kill: floors at 1 HP (VERIFIED w/Effect)
                        if health > 1.0 {
                            dmg += 1.0;
                        }
                    }
                    EffectKind::Regeneration => {
                        heal += 1.0;
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        (dmg, heal)
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.acc.clear();
    }
}

/// Speed multiplier for movement (VERIFIED w/Effect §Speed: +20% per
/// level). The player's base walk/sprint scales by this.
pub fn speed_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Speed)
        .map(|a| 1.0 + 0.20 * (a as f32 + 1.0))
        .unwrap_or(1.0)
}

/// Slowness multiplier for movement (VERIFIED w/Effect §Slowness: −15%
/// per level; level 6+ floors at zero — w/Effect §Slowness notes
/// "slowness 7" makes movement impossible). Applied to the player's
/// walk/sprint target alongside speed_multiplier.
pub fn slowness_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Slowness)
        .map(|a| (1.0 - 0.15 * (a as f32 + 1.0)).max(0.0))
        .unwrap_or(1.0)
}

/// Melee damage bonus (VERIFIED w/Effect §Strength: +3 HP per level in
/// 1.9+; pre-1.9 ×1.3/×1.6 multiplier — 1.16.5 uses the flat +3/level
/// rule; 1.3–1.4 content in a 1.16.5-target engine follows the 1.16
/// formula, disclosed).
pub fn strength_bonus(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Strength)
        .map(|a| 3.0 * (a as f32 + 1.0))
        .unwrap_or(0.0)
}

/// Incoming damage multiplier (VERIFIED w/Effect §Resistance: −20% per
/// level, floors at 20% damage taken at level 4).
pub fn resistance_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Resistance)
        .map(|a| (1.0 - 0.20 * (a as f32 + 1.0)).max(0.2))
        .unwrap_or(1.0)
}

/// Jump velocity boost (VERIFIED w/Effect §Jump Boost: +0.1 b/t per
/// level on top of the vanilla 0.42 launch, caps at level II for our
/// beacon use).
pub fn jump_boost_velocity(effects: &Effects, base: f32) -> f32 {
    effects
        .amplifier(EffectKind::JumpBoost)
        .map(|a| base + 0.1 * (a as f32 + 1.0))
        .unwrap_or(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wither_ticks_damage_every_second_and_can_kill() {
        let mut e = Effects::new();
        // wither skeleton hit: Wither 10 s (VERIFIED w/Wither_Skeleton —
        // level I in 1.16.5; the page's damage row describes the II row
        // for the wither boss; skeleton applies level I? — wiki text:
        // "inflicted with the Wither effect for 10 seconds ... decreases
        // it by 1 HP every two seconds" → level I, 40-tick period is the
        // II row; 1 HP per 2 s at I = period 40.
        e.apply(EffectKind::Wither, 0, 200);
        let mut dmg = 0.0;
        for _ in 0..200 {
            let (d, _) = e.tick(20.0);
            dmg += d;
        }
        // 10 s at 1 HP per 2 s (level I) = 5 HP — VERIFIED phrasing
        assert!((dmg - 5.0).abs() < 0.01, "wither I total = 5 HP, got {dmg}");
        // expired
        assert!(e.amplifier(EffectKind::Wither).is_none());
    }

    #[test]
    fn wither_ii_ticks_at_the_boss_rate() {
        let mut e = Effects::new();
        // wither skull: Wither II 10 s (VERIFIED w/Wither: 1 HP per sec)
        e.apply(EffectKind::Wither, 1, 200);
        let mut dmg = 0.0;
        for _ in 0..200 {
            let (d, _) = e.tick(20.0);
            dmg += d;
        }
        assert!((dmg - 10.0).abs() < 0.01, "wither II total = 10 HP, got {dmg}");
    }

    #[test]
    fn poison_cannot_kill() {
        let mut e = Effects::new();
        e.apply(EffectKind::Poison, 0, 1000);
        let mut dmg = 0.0;
        for _ in 0..1000 {
            let (d, _) = e.tick(1.0); // at 1 HP already
            dmg += d;
        }
        assert_eq!(dmg, 0.0, "poison floors at 1 HP (VERIFIED)");
    }

    #[test]
    fn regeneration_heals_every_2_point_5s() {
        let mut e = Effects::new();
        e.apply(EffectKind::Regeneration, 0, 100);
        let mut heal = 0.0;
        for _ in 0..100 {
            let (_, h) = e.tick(20.0);
            heal += h;
        }
        assert!((heal - 2.0).abs() < 0.01, "regen I: 2 HP over 5 s, got {heal}");
    }

    #[test]
    fn stronger_application_wins_ties_prefer_longer() {
        let mut e = Effects::new();
        e.apply(EffectKind::Speed, 0, 100);
        e.apply(EffectKind::Speed, 0, 50); // weaker duration: ignored
        assert_eq!(e.amplifier(EffectKind::Speed), Some(0));
        assert_eq!(e.active[0].ticks_left, 100);
        e.apply(EffectKind::Speed, 1, 10); // stronger amplifier: wins
        assert_eq!(e.amplifier(EffectKind::Speed), Some(1));
    }

    #[test]
    fn stat_modifiers_match_the_wiki() {
        let mut e = Effects::new();
        assert_eq!(speed_multiplier(&e), 1.0);
        assert_eq!(strength_bonus(&e), 0.0);
        assert_eq!(resistance_multiplier(&e), 1.0);
        e.apply(EffectKind::Speed, 0, 10);
        e.apply(EffectKind::Strength, 0, 10);
        e.apply(EffectKind::Resistance, 0, 10);
        assert!((speed_multiplier(&e) - 1.2).abs() < 1e-6); // +20% level I
        assert!((strength_bonus(&e) - 3.0).abs() < 1e-6); // +3 HP level I
        assert!((resistance_multiplier(&e) - 0.8).abs() < 1e-6); // -20%
        // level II variants (beacon secondary)
        e.apply(EffectKind::Speed, 1, 10);
        assert!((speed_multiplier(&e) - 1.4).abs() < 1e-6);
        // resistance level 4 floors at 20%
        e.apply(EffectKind::Resistance, 3, 10);
        assert!((resistance_multiplier(&e) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn jump_boost_adds_a_tenth_per_level() {
        let mut e = Effects::new();
        assert_eq!(jump_boost_velocity(&e, 0.42), 0.42);
        e.apply(EffectKind::JumpBoost, 0, 10);
        assert!((jump_boost_velocity(&e, 0.42) - 0.52).abs() < 1e-6);
    }
}
