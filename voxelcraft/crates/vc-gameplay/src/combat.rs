//! Combat math (master prompt Phase 2). Every formula here was verified
//! against the reference wiki on 2026-09-04 (see the verification notes per
//! function) — not copied from dossier prose or memory.
//!
//! VERIFIED (reference wiki /Attack_cooldown, /Critical_hit, /Armor):
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
/// VERIFIED (reference wiki /Armor, "Damage formulas"):
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

// ---------------------------------------------------------------------------
// TNT round — the verified explosion math (all values live-verified
// 2026-09-22 against reference wiki /Explosion §Damage/§Velocity/
// §Dropping blocks; see also w/TNT §Behavior)
// ---------------------------------------------------------------------------

/// The explosion's per-entity blast impact — VERIFIED w/Explosion §Damage
/// (the Java code): `impact = (1 - distance/(2*power)) * exposure`.
/// `distance` is measured from the explosion center to the entity's
/// position; `exposure` (Explosion §Exposure) is the fraction of
/// unobstructed rays from the explosion center to the sample points on
/// the entity's bounding-box grid. Clamped at 0 (a fully blocked blast
/// has impact 0 — the +1 floor below still damages in-range entities).
#[inline]
pub fn explosion_impact(distance: f32, power: f32, exposure: f32) -> f32 {
    (1.0 - distance / (2.0 * power)).max(0.0) * exposure.max(0.0)
}

/// The explosion's per-entity damage — VERIFIED w/Explosion §Damage (the
/// Java code): `damage = (impact*impact + impact)/2 * 7*(2*power) + 1`,
/// i.e. `7 · power · (impact² + impact) + 1`. The +1 at the end means all
/// entities in range (within 2·power) receive at least 1 damage even when
/// the explosion is fully blocked (the invulnerable/Peaceful exceptions
/// are the callers' gates).
#[inline]
pub fn explosion_damage(impact: f32, power: f32) -> f32 {
    7.0 * power * (impact * impact + impact) + 1.0
}

/// The explosion's knockback magnitude — VERIFIED w/Explosion §Velocity
/// (the Java code): `magnitude = (1 - distance/(2*power)) * exposure *
/// knockbackMultiplier * (1 - EXPLOSION_KNOCKBACK_RESISTANCE)`. The
/// knockback multiplier is 1.0 for every engine explosion cause (the
/// wind-charge rows are post-1.16.5); the engine's mobs carry no
/// `explosion_knockback_resistance` attribute — both factors reduce out.
#[inline]
pub fn explosion_knockback(distance: f32, power: f32, exposure: f32) -> f32 {
    (1.0 - distance / (2.0 * power)).max(0.0) * exposure.max(0.0)
}

/// The per-block item-drop chance for an explosion — VERIFIED w/Explosion
/// §Dropping blocks: "Blocks destroyed by TNT have a 100% chance of
/// dropping. Other explosions have a 1/power chance of dropping items."
#[inline]
pub fn explosion_drop_chance(power: f32, tnt: bool) -> f32 {
    if tnt {
        1.0
    } else {
        1.0 / power.max(1.0)
    }
}

/// The melee profile of whatever the player is holding, as
/// `(damage HP, attack_speed attribute)`.
///
/// VANILLA: the held *item* decides both numbers — a wooden sword is 4 HP at
/// attack speed 1.6, a diamond axe 7 HP at 1.0, a netherite sword 8 HP at
/// 1.6, and anything else is "Other Items" at 1 HP / 4.0.
///
/// Phase 0: the engine has no tool ITEMS yet — `held_block` is a *block* id
/// from `player.held()`, and no block id is a weapon — so every block
/// resolves to FIST. The argument is matched explicitly rather than
/// discarded, so the tool arms are the one place to extend when Phase 6 adds
/// items; before this, the parameter was dropped with `let _ =` and any
/// caller passing anything at all silently got the same 1.0 HP.
#[inline]
pub fn held_attack(held_block: u16) -> (f32, f32) {
    melee_profile(held_block)
}

#[inline]
fn melee_profile(held_block: u16) -> (f32, f32) {
    // VERIFIED w/Item §Attack damage: "Other Items" = 1 damage, and the
    // attack-speed attribute for a bare hand is 4.0 (2.5 ticks of cooldown
    // is applied separately by the caller).
    match tool_profile(held_block) {
        Some(profile) => profile,
        None => FIST,
    }
}

/// The "Other Items" melee profile: 1 damage, attack speed 4.0.
const FIST: (f32, f32) = (1.0, 4.0);

/// Phase 6: the one table that turns a held item into vanilla's two melee
/// numbers (a wooden sword is 4 HP at 1.6, a diamond sword 7 HP at 1.6, and
/// so on). 3.1b arms it from the tools registry; non-tool inputs still
/// fall through to FIST (a *block* id is never a weapon).
#[inline]
fn tool_profile(held_block: u16) -> Option<(f32, f32)> {
    crate::tools::melee_profile(held_block)
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

/// Sweep gating: swords only, never while sprinting (sprint attacks
/// knock back instead) — fists never sweep, exactly like vanilla.
pub fn can_sweep(held_block: u16, sprinting: bool) -> bool {
    !sprinting
        && matches!(
            crate::tools::tool_kind(held_block),
            Some((_, crate::tools::ToolClass::Sword))
        )
}

/// Sweep damage for the arc: 1 HP + weapon base scaled by Sweeping
/// Edge (level/(level+1) — level 0 deals the flat 1, level III deals
/// 1 + 75% of base). Generalizes the module-doc formula.
pub fn sweep_damage(base_weapon_damage: f32, sweeping_level: u32) -> f32 {
    1.0 + base_weapon_damage * (sweeping_level as f32 / (sweeping_level as f32 + 1.0))
}

/// Looting common-drop cap: the table max plus one per level
/// (saturating — the vanilla "+1 per level" common-drop rule).
pub fn looted_max(max_n: u8, looting: u8) -> u8 {
    max_n.saturating_add(looting)
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
    fn sweep_gate_and_damage() {
        use vc_blocks::blocks as blk;
        assert!(can_sweep(blk::IRON_SWORD, false));
        assert!(!can_sweep(blk::IRON_SWORD, true));
        assert!(!can_sweep(blk::IRON_PICKAXE, false));
        assert!(!can_sweep(blk::STONE, false));
        // flat 1 unenchanted; 1 + 75% of base at III
        assert!((sweep_damage(6.0, 0) - 1.0).abs() < 1e-6);
        assert!((sweep_damage(6.0, 1) - 4.0).abs() < 1e-6);
        assert!((sweep_damage(6.0, 3) - 5.5).abs() < 1e-6);
    }

    #[test]
    fn looted_max_adds_one_per_level() {
        assert_eq!(looted_max(2, 0), 2);
        assert_eq!(looted_max(2, 3), 5);
        assert_eq!(looted_max(255, 3), 255);
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

    // Phase 0: held_attack must CONSUME its argument, not drop it. These pin
    // the contract so a future tool table cannot be added silently-wrong.
    #[test]
    fn held_attack_is_fists_for_every_block_and_never_panics() {
        // a real block id, the max block id, and an id past the end of the
        // registry: all must resolve, none may panic or index.
        for id in [0u16, 1, 539, u16::MAX] {
            let (dmg, spd) = held_attack(id);
            assert_eq!(dmg, 1.0, "id {id} should be a 1.0 HP fist hit");
            assert_eq!(spd, 4.0, "id {id} should have attack speed 4.0");
        }
    }

    #[test]
    fn player_melee_reads_its_damage_from_held_attack() {
        // the two must never disagree about what is being held
        for id in [0u16, 7, 539] {
            let (dmg, _) = held_attack(id);
            let o = player_melee(id, 1.0, false, false, 0.0, 0.0);
            assert_eq!(o.damage, dmg, "id {id}: player_melee must use held_attack");
        }
    }

    #[test]
    fn tnt_impact_matches_the_verified_formula() {
        // VERIFIED w/Explosion §Damage (the Java code): impact =
        // (1 − distance/(2·power)) · exposure. Power 4 (TNT), fully
        // exposed, at the center: impact = 1
        assert!((explosion_impact(0.0, 4.0, 1.0) - 1.0).abs() < 1e-6);
        // 4 blocks out (the blast radius): 1 − 4/8 = 0.5
        assert!((explosion_impact(4.0, 4.0, 1.0) - 0.5).abs() < 1e-6);
        // half-exposed at the center: 0.5
        assert!((explosion_impact(0.0, 4.0, 0.5) - 0.5).abs() < 1e-6);
        // 8 blocks out (2·power): 0 — out of range
        assert!((explosion_impact(8.0, 4.0, 1.0).abs()) < 1e-6);
        // beyond 2·power clamps at 0 (never negative)
        assert_eq!(explosion_impact(12.0, 4.0, 1.0), 0.0);
        // a fully blocked blast: exposure 0
        assert_eq!(explosion_impact(0.0, 4.0, 0.0), 0.0);
        // the creeper's power 3: 3 blocks out → 1 − 3/6 = 0.5
        assert!((explosion_impact(3.0, 3.0, 1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tnt_damage_matches_the_verified_formula() {
        // VERIFIED w/Explosion §Damage (the Java code): damage =
        // (impact² + impact)/2 · 7·(2·power) + 1 = 7·power·(impact² +
        // impact) + 1. Power 4, impact 1 (a point-blank fully exposed
        // hit): 7·4·2 + 1 = 57
        assert!((explosion_damage(1.0, 4.0) - 57.0).abs() < 1e-4);
        // the +1 floor: impact 0 (fully blocked, still in range) = 1
        assert!((explosion_damage(0.0, 4.0) - 1.0).abs() < 1e-6);
        // impact 0.5 at the blast radius: 7·4·(0.25 + 0.5) + 1 = 22
        assert!((explosion_damage(0.5, 4.0) - 22.0).abs() < 1e-4);
        // the creeper's power 3, impact 1: 7·3·2 + 1 = 43
        assert!((explosion_damage(1.0, 3.0) - 43.0).abs() < 1e-4);
        // difficulty scaling rides difficulty_scale: 57 → Hard 85.5 /
        // Easy 29.5 (min(57/2+1, 57)) — the wiki's per-difficulty rows
        assert!((difficulty_scale(57.0, Difficulty::Hard) - 85.5).abs() < 1e-4);
        assert!((difficulty_scale(57.0, Difficulty::Easy) - 29.5).abs() < 1e-4);
        assert_eq!(difficulty_scale(57.0, Difficulty::Peaceful), 0.0);
    }

    #[test]
    fn tnt_knockback_and_drop_chances_match() {
        // VERIFIED w/Explosion §Velocity (the Java code): magnitude =
        // (1 − distance/(2·power)) · exposure (the default 1.0 knockback
        // multiplier; no explosion_knockback_resistance in the engine)
        assert!((explosion_knockback(0.0, 4.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((explosion_knockback(4.0, 4.0, 1.0) - 0.5).abs() < 1e-6);
        assert_eq!(explosion_knockback(0.0, 4.0, 0.0), 0.0);
        // VERIFIED w/Explosion §Dropping blocks: TNT explosions drop
        // 100%; other explosions 1/power
        assert_eq!(explosion_drop_chance(4.0, true), 1.0);
        assert!((explosion_drop_chance(3.0, false) - 1.0 / 3.0).abs() < 1e-6);
        assert!((explosion_drop_chance(4.0, false) - 0.25).abs() < 1e-6);
        // a degenerate power never divides by zero
        assert!((explosion_drop_chance(0.0, false) - 1.0).abs() < 1e-6);
    }
}
