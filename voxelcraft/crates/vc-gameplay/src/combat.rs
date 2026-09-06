//! Combat math (master prompt Phase 2). Every formula here was verified
//! against minecraft.wiki on 2026-09-04 (see the verification notes per
//! function) — not copied from dossier prose or memory.
//!
//! VERIFIED (minecraft.wiki/w/Attack_cooldown, /Critical_hit, /Armor):
//! - cooldown damage scaling: `0.2 + 0.8·p²` for base melee damage
//! - critical hits: ×1.5, requires falling + cooldown ≥ 84.8% + NOT sprinting
//! - armor: `min(20, max(armor/5, armor − 4·damage/(toughness+8))) / 25`
//!   (percent), i.e. 4%/point base, 80% cap, toughness dampens the fall-off
//! - attack cooldown ticks: `20 / attack_speed` (sword 1.6 → 12.5 ticks =
//!   0.625 s — matches the wiki's cooldown table)
//! - difficulty scaling (wiki mob pages, e.g. zombie Easy 2.5 / Normal 3 /
//!   Hard 4.5): Hard = 1.5×, Easy = min(d, 0.5·d + 1)
//!
//! Documented adaptations:
//! - knockback is a horizontal impulse + small lift (vanilla's 0.4 base
//!   velocity knockback is applied per-attribute; ours is the observable
//!   equivalent — flagged, not exact)
//! - sweep damage: vanilla sweeps only with a SWORD on the sweep edge
//!   (1 HP + weapon damage/2 to nearby targets). Kept sword-gated: fists
//!   never sweep, exactly like vanilla.

/// attack-speed attribute → full-cooldown duration in game ticks.
/// VERIFIED: T = 20 / attack_speed (fists 4.0 → 5 ticks = 0.25 s,
/// sword 1.6 → 12.5 ticks = 0.625 s).
#[inline]
pub fn attack_cooldown_ticks(attack_speed: f32) -> f32 {
    20.0 / attack_speed.max(0.05)
}

/// Cooldown-completion damage multiplier for BASE melee damage.
/// VERIFIED: 0.2 + 0.8·p² where p ∈ [0,1] is the charge fraction.
#[inline]
pub fn cooldown_damage_scale(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    0.2 + 0.8 * p * p
}

/// Critical-hit gate.
/// VERIFIED: ×1.5 damage, requires (a) attacker falling, (b) cooldown
/// completion ≥ 84.8%, (c) attacker NOT sprinting (sprint+knockback attack
/// replaces it in Java).
#[inline]
pub fn is_critical(falling: bool, sprinting: bool, cooldown_p: f32) -> bool {
    falling && !sprinting && cooldown_p >= 0.848
}

/// Armor + toughness damage reduction, the exact vanilla formula.
/// VERIFIED (minecraft.wiki/w/Armor, "Damage formulas"):
/// `points = min(20, max(armor/5, armor − 4·damage/(toughness+8)))`,
/// reduction% = points × 4 (base 4%/point, floor armor/5 points,
/// cap 20 points = 80%; the equivalent percent form on the wiki is
/// `min(80, max(4/5·armor, 4·armor − 16·damage/(toughness+8)))`).
/// Returns the damage that gets THROUGH the armor.
#[inline]
pub fn armor_reduce(damage: f32, armor: f32, toughness: f32) -> f32 {
    if damage <= 0.0 {
        return damage;
    }
    let armor = armor.clamp(0.0, 30.0);
    let toughness = toughness.clamp(0.0, 20.0);
    let min_reduction = armor / 5.0; // floor: 4% × armor/5
    let scaled = armor - 4.0 * damage / (toughness + 8.0);
    let points = min_reduction.max(scaled).min(20.0); // cap: 80%
    damage * (1.0 - points / 25.0)
}

/// Difficulty damage scaling (mob melee/arrow hits).
/// VERIFIED against the wiki mob stat rows (zombie 2.5 / 3 / 4.5):
/// Hard = 1.5×, Easy = min(d, 0.5·d + 1), Normal unchanged.
#[inline]
pub fn difficulty_scale(damage: f32, difficulty: Difficulty) -> f32 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => damage.min(0.5 * damage + 1.0),
        Difficulty::Normal => damage,
        Difficulty::Hard => 1.5 * damage,
    }
}

/// Difficulty of the current session. Peaceful exists in the enum for
/// completeness (mob AI checks it); the world-creation flow only offers
/// Survival/Hardcore mapping to Normal/Hard.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

/// Player melee damage for a held item. Fists (or any non-weapon block —
/// we have no sword items yet) are VERIFIED vanilla "Other Items": 1 HP
/// base at attack speed 4.0.
#[inline]
pub fn held_attack(held_block: u16) -> (f32, f32) {
    let _ = held_block; // swords arrive with tool items; fists until then
    (1.0, 4.0) // (damage HP, attack_speed attribute)
}

/// One melee hit resolution (player → mob), all modifiers applied.
/// Order per vanilla: difficulty scaling happens on the MOB's attack; the
/// player's own hit is NOT difficulty-scaled. Crits multiply total base;
/// armor applies after.
pub struct MeleeOutcome {
    pub damage: f32,
    pub critical: bool,
}

pub fn player_melee(
    held_block: u16,
    cooldown_p: f32,
    falling: bool,
    sprinting: bool,
    target_armor: f32,
    target_toughness: f32,
) -> MeleeOutcome {
    let (base, _) = held_attack(held_block);
    let scale = cooldown_damage_scale(cooldown_p);
    let crit = is_critical(falling, sprinting, cooldown_p);
    let mut dmg = base * scale;
    if crit {
        dmg *= 1.5; // VERIFIED: +50%
    }
    let damage = armor_reduce(dmg, target_armor, target_toughness);
    MeleeOutcome {
        damage,
        critical: crit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_ticks_match_vanilla() {
        // fists 4.0 → 5 ticks (0.25 s); sword 1.6 → 12.5 ticks (0.625 s)
        assert!((attack_cooldown_ticks(4.0) - 5.0).abs() < 1e-6);
        assert!((attack_cooldown_ticks(1.6) - 12.5).abs() < 1e-6);
    }

    #[test]
    fn cooldown_scale_endpoints() {
        // uncharged hit keeps 20%; fully charged 100%
        assert!((cooldown_damage_scale(0.0) - 0.2).abs() < 1e-6);
        assert!((cooldown_damage_scale(1.0) - 1.0).abs() < 1e-6);
        // p=0.5 → 0.2 + 0.8·0.25 = 0.4
        assert!((cooldown_damage_scale(0.5) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn critical_conditions() {
        // all three must hold: falling, ≥84.8%, not sprinting
        assert!(is_critical(true, false, 0.9));
        assert!(!is_critical(true, false, 0.7)); // not charged enough
        assert!(!is_critical(true, true, 0.9)); // sprint-knockback instead
        assert!(!is_critical(false, false, 0.9)); // must be falling
                                                  // exact boundary: 0.848 counts as charged
        assert!(is_critical(true, false, 0.848));
        assert!(!is_critical(true, false, 0.847));
    }

    #[test]
    fn armor_formula_matches_vanilla_examples() {
        // no armor: full damage through
        assert!((armor_reduce(6.0, 0.0, 0.0) - 6.0).abs() < 1e-6);
        // 20 armor, 0 toughness vs a 6 hit: points = 20 − 4·6/8 = 17
        // (the wiki's "−2% per damage point": 80% − 12% = 68% reduced)
        assert!((armor_reduce(6.0, 20.0, 0.0) - 1.92).abs() < 1e-4);
        // huge hit vs 20 armor/0 toughness: floor = armor/5 = 4 points
        // → 16% reduced → 84 through
        assert!((armor_reduce(100.0, 20.0, 0.0) - 84.0).abs() < 1e-4);
        // zombie's natural 2 armor vs a 3 hit: max(0.4, 2 − 1.5) = 0.5 pts
        // → 3 × (1 − 0.02) = 2.94
        assert!((armor_reduce(3.0, 2.0, 0.0) - 2.94).abs() < 1e-4);
        // toughness 8 dampens: 20 armor vs 10 dmg → 20 − 40/16 = 17.5 pts
        // → 70% reduced → 3 through
        assert!((armor_reduce(10.0, 20.0, 8.0) - 3.0).abs() < 1e-4);
        // tiny hit vs 20 armor: scaled = 20 − 0.05 = 19.95 pts
        // → 0.1 × (1 − 19.95/25) = 0.0202 (approaching the 80% cap)
        assert!((armor_reduce(0.1, 20.0, 0.0) - 0.0202).abs() < 1e-6);
    }

    #[test]
    fn difficulty_scaling_matches_wiki_rows() {
        // zombie melee: Easy 2.5 / Normal 3 / Hard 4.5 (the wiki row)
        assert!((difficulty_scale(3.0, Difficulty::Easy) - 2.5).abs() < 1e-6);
        assert!((difficulty_scale(3.0, Difficulty::Normal) - 3.0).abs() < 1e-6);
        assert!((difficulty_scale(3.0, Difficulty::Hard) - 4.5).abs() < 1e-6);
        // enderman: 4.5 / 7 / 10.5 — hard = 1.5× again
        assert!((difficulty_scale(7.0, Difficulty::Hard) - 10.5).abs() < 1e-6);
        assert!((difficulty_scale(7.0, Difficulty::Easy) - 4.5).abs() < 1e-6);
        // easy clamps LOW damage: 1 → min(1, 1.5) = 1 (unchanged)
        assert!((difficulty_scale(1.0, Difficulty::Easy) - 1.0).abs() < 1e-6);
        assert_eq!(difficulty_scale(9.0, Difficulty::Peaceful), 0.0);
    }

    #[test]
    fn player_fist_full_pipeline() {
        // fully charged fist on an unarmored target = 1 HP
        let o = player_melee(0, 1.0, false, false, 0.0, 0.0);
        assert!((o.damage - 1.0).abs() < 1e-6);
        assert!(!o.critical);
        // falling + charged + not sprinting → crit: 1.5 HP
        let o = player_melee(0, 1.0, true, false, 0.0, 0.0);
        assert!(o.critical);
        assert!((o.damage - 1.5).abs() < 1e-6);
        // uncharged: 0.2 HP
        let o = player_melee(0, 0.0, true, false, 0.0, 0.0);
        assert!((o.damage - 0.2).abs() < 1e-6);
        // zombie natural armor 2: 1 HP → max(0.4, 2−0.5) = 1.5 pts
        // → 1 × (1 − 0.06) = 0.94
        let o = player_melee(0, 1.0, false, false, 2.0, 0.0);
        assert!((o.damage - 0.94).abs() < 1e-4);
    }
}
