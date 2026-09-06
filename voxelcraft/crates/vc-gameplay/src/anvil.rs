//! Phase E2 (evolution 1.3–1.4 bracket): anvil mechanics.
//! All values live-verified 2026-09-06 against minecraft.wiki
//! (docs/research/phase2-1.3-1.4-research.md):
//! - 12% chance per use to degrade one stage; average 25 uses (w/Anvil
//!   §Becoming damaged)
//! - falling damage: 2 HP per block fallen after the first, cap 40 HP;
//!   helmets absorb 25% (w/Anvil §Falling anvils)
//! - anvil falls like sand; >600 ticks falling → drops as an item;
//!   landing on a torch/slab-style non-replaceable → breaks as an item
//!
//! DEFERRED (disclosed in the worklog): the repair/combine/rename/cost
//! mechanics. The engine has NO damageable items (no tools, no armor
//! durability, ItemStack carries only block/count/ench), no custom
//! item names, and no anvil-GUI — implementing a fake repair math
//! against nonexistent systems would be dishonest. Constants for the
//! repair rules (material 25%, combine +10%, prior-work doubling,
//! 39-level "Too Expensive" cap since 1.8, rename 1 level) are recorded
//! here for the tools/armor bracket that will use them.

use vc_blocks::blocks::{DAMAGED_ANVIL, ANVIL, CHIPPED_ANVIL};

/// anvil degrade chance per use (VERIFIED w/Anvil: 12%)
pub const DEGRADE_CHANCE_PER_USE: f32 = 0.12;
/// falling damage per block after the first (VERIFIED: 2 HP)
pub const FALL_DMG_PER_BLOCK: f32 = 2.0;
/// falling damage cap (VERIFIED: 40 HP)
pub const FALL_DMG_CAP: f32 = 40.0;
/// helmet damage reduction vs falling anvils (VERIFIED: 25%)
pub const HELMET_REDUCTION: f32 = 0.25;
/// (deferred-rules record) 1 material repairs 25% of max durability
pub const MATERIAL_REPAIR_FRACTION: f32 = 0.25;
/// (deferred-rules record) combine bonus: 10% of max on top of the sum
pub const COMBINE_BONUS_FRACTION: f32 = 0.10;
/// (deferred-rules record) prior-work penalty doubles per repair
pub const PRIOR_WORK_BASE: i32 = 2;
/// (deferred-rules record) "Too Expensive!" cap: 39 levels since 1.8
pub const COST_CAP: i32 = 39;
/// (deferred-rules record) rename surcharge: 1 level + prior-work
pub const RENAME_COST: i32 = 1;
/// a falling anvil drops as an item after this many falling ticks
/// (VERIFIED: 600 ticks / 30 s)
pub const MAX_FALLING_TICKS: i32 = 600;

/// falling anvil damage for a fall of `blocks` blocks (VERIFIED: 2 HP
/// per block after the first, capped at 40; helmets absorb 25%).
#[inline]
pub fn falling_damage(blocks: i32, wearing_helmet: bool) -> f32 {
    let raw = ((blocks - 1).max(0) as f32) * FALL_DMG_PER_BLOCK;
    let dmg = raw.min(FALL_DMG_CAP);
    if wearing_helmet {
        dmg * (1.0 - HELMET_REDUCTION)
    } else {
        dmg
    }
}

/// Should the anvil degrade one stage on a use? (12% — VERIFIED.)
#[inline]
pub fn degrades(roll: f32) -> bool {
    roll < DEGRADE_CHANCE_PER_USE
}

/// Next damage-stage block id (anvil → chipped → damaged → None =
/// destroyed). The three-stage ladder is a VERIFIED w/Anvil behavior.
#[inline]
pub fn next_stage(b: u16) -> Option<u16> {
    match b {
        ANVIL => Some(CHIPPED_ANVIL),
        CHIPPED_ANVIL => Some(DAMAGED_ANVIL),
        DAMAGED_ANVIL => None, // destroyed — breaks and disappears
        _ => None,
    }
}

/// (deferred-rules record) prior-work penalty for `repairs` prior
/// anvil operations (doubles; clamped at the refusal threshold).
#[inline]
pub fn prior_work_penalty(repairs: u32) -> i32 {
    (1i32 << repairs.min(6)).min(COST_CAP + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_blocks::blocks::*;

    #[test]
    fn constants_match_the_live_wiki() {
        assert_eq!(DEGRADE_CHANCE_PER_USE, 0.12); // w/Anvil §Becoming damaged
        assert_eq!(FALL_DMG_PER_BLOCK, 2.0);
        assert_eq!(FALL_DMG_CAP, 40.0);
        assert_eq!(HELMET_REDUCTION, 0.25);
        assert_eq!(MAX_FALLING_TICKS, 600); // 30 s
        assert_eq!(MATERIAL_REPAIR_FRACTION, 0.25);
        assert_eq!(COMBINE_BONUS_FRACTION, 0.10);
        assert_eq!(COST_CAP, 39); // 1.8+ (pre-1.8 40 — disclosed)
        assert_eq!(RENAME_COST, 1);
    }

    #[test]
    fn falling_damage_formula_cap_and_helmet() {
        // 4-block fall: (4-1)*2 = 6 — the wiki's own worked example
        assert_eq!(falling_damage(4, false), 6.0);
        assert_eq!(falling_damage(1, false), 0.0);
        assert_eq!(falling_damage(25, false), 40.0, "capped at 40");
        assert_eq!(falling_damage(4, true), 4.5, "helmet 25% off");
    }

    #[test]
    fn degrade_gate_is_12_percent() {
        assert!(degrades(0.0));
        assert!(degrades(0.11));
        assert!(!degrades(0.12));
        assert!(!degrades(0.99));
    }

    #[test]
    fn damage_ladder_walks_all_three_stages() {
        assert_eq!(next_stage(ANVIL), Some(CHIPPED_ANVIL));
        assert_eq!(next_stage(CHIPPED_ANVIL), Some(DAMAGED_ANVIL));
        assert_eq!(next_stage(DAMAGED_ANVIL), None, "destroyed");
    }

    #[test]
    fn prior_work_penalty_doubles() {
        assert_eq!(prior_work_penalty(0), 1);
        assert_eq!(prior_work_penalty(1), 2);
        assert_eq!(prior_work_penalty(2), 4);
        assert_eq!(prior_work_penalty(3), 8);
    }
}
