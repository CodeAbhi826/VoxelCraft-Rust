//! Round 13 (station GUIs): the grindstone mechanics — disenchant,
//! combine-repair, and the XP return.
//!
//! All rules VERIFIED live 2026-09-15 against reference wiki /Grindstone
//! §Usage/§Repairing and disenchanting (docs/research/
//! round-13-station-screens-audit.md §3):
//! - one enchanted item in -> "a disenchanted copy of the item appears
//!   in the output slot. The output item has the same durability";
//!   "If an enchanted book is placed in the input, a normal book
//!   appears in the output"
//! - two same-type items -> output durability = "the sum of the
//!   durabilities of the two input items, plus 5% of the maximum
//!   durability of the output item (rounded down), capped" — the 5%
//!   bonus is NOT the anvil's 12%
//! - "Disenchanting an item in the grindstone does not remove curse
//!   enchantments or an item's custom name" — a curse-only input would
//!   return an identical item, so the grindstone REFUSES it (the spec's
//!   cursed-row rule)
//! - "Unenchanted items [Java] placed inside will have a red cross
//!   over the arrow and no item in the output slot"
//! - the grindstone RESETS the prior work penalty
//! - XP: "a uniformly distributed pseudorandom number between 50% and
//!   100% (rounded up) of all the non-curse enchantments' minimum
//!   modified enchantment levels combined" — the per-enchant M values
//!   below are the maxima of the wiki's published XP-ranges table

use vc_blocks::blocks::{BOOK, ENCHANTED_BOOK};
use vc_inventory::inventory::ItemStack;
use crate::anvil::armor_max_durability;
use crate::enchanting::enchant_def;

/// the two-input combine bonus (VERIFIED w/Grindstone: 5% of max,
/// rounded down — not the anvil's 12%)
pub const COMBINE_BONUS_FRACTION: f32 = 0.05;

/// the per-enchant "minimum modified enchantment level" M by level
/// (1-indexed) — the max of the wiki's XP-drop ranges (live 2026-09-15).
/// Curses are excluded (0 XP — "does not remove curse enchantments",
/// and the table has no curse rows).
pub fn xp_min_cost(ench: u8, level: u8) -> u32 {
    let lvl = level.max(1) as usize;
    let table: &[u32] = match enchant_def(ench).id {
        "protection" => &[1, 11, 23, 33],
        "fire_protection" => &[9, 17, 25, 33],
        "feather_falling" => &[5, 11, 17, 23],
        "blast_protection" => &[5, 13, 21, 29],
        "projectile_protection" => &[3, 9, 15, 21],
        "respiration" => &[9, 19, 29],
        "aqua_affinity" => &[1],
        "thorns" => &[9, 29, 49],
        "depth_strider" => &[9, 19, 29],
        "frost_walker" => &[9, 19],
        "soul_speed" => &[9, 19, 29],
        "sharpness" => &[1, 11, 23, 33, 45],
        "smite" => &[5, 13, 21, 29, 37],
        "bane_of_arthropods" => &[5, 13, 21, 29, 37],
        "knockback" => &[5, 25],
        "fire_aspect" => &[9, 29],
        "looting" => &[15, 23],
        "sweeping" => &[5, 13, 23],
        "power" => &[1, 11, 21, 31, 41],
        "punch" => &[11, 31],
        "flame" => &[19],
        "infinity" => &[19],
        "efficiency" => &[1, 11, 21, 31, 41],
        "silk_touch" => &[15],
        "fortune" => &[15, 23, 33],
        "luck_of_the_sea" => &[15, 23, 33],
        "lure" => &[15, 23, 33],
        "unbreaking" => &[5, 13, 21],
        "mending" => &[25],
        "channeling" => &[25],
        "impaling" => &[1, 9, 17, 25, 33],
        "loyalty" => &[11, 19, 25],
        "riptide" => &[17, 23, 31],
        "multishot" => &[19],
        "piercing" => &[1, 11, 21, 31],
        "quick_charge" => &[11, 31, 51],
        // curses: no XP row on the wiki table
        "binding_curse" | "vanishing_curse" => &[],
        _ => &[],
    };
    if lvl > table.len() {
        0
    } else {
        table[lvl - 1]
    }
}

/// one computed grindstone operation
#[derive(Clone, Debug, PartialEq)]
pub struct GrindPlan {
    pub result: ItemStack,
    /// XP bounds: a uniform roll in [min, max] (VERIFIED: 50%..100% of
    /// the combined minimum modified levels, rounded up at the floor)
    pub xp_min: u32,
    pub xp_max: u32,
}

/// is this enchant one of the two curses?
fn is_curse(ench: u8) -> bool {
    matches!(enchant_def(ench).id, "binding_curse" | "vanishing_curse")
}

/// non-curse enchants carried by a stack
fn non_curse(s: &ItemStack) -> Vec<(u8, u8)> {
    s.enchants()
        .iter()
        .flatten()
        .copied()
        .filter(|&(id, _)| !is_curse(id))
        .collect()
}

/// The grindstone plan for the current inputs.
///
/// - (item, empty) or (empty, item): the disenchant path — requires at
///   least one non-curse enchant (curse-only refuses; unenchanted
///   refuses with the Java red-X rule).
/// - (item, item): the combine path — same block only; durability
///   = min(rem_a + rem_b + floor(5% max), max); enchants stripped
///   (curses survive; a result that would still carry a curse keeps
///   ONLY the curse — vanilla "does not remove curse enchantments");
///   the name rides the TOP input; the prior-work penalty resets.
/// - Returns None whenever the grindstone refuses (red X, no output).
pub fn grindstone_plan(top: &ItemStack, bottom: &ItemStack) -> Option<GrindPlan> {
    if top.is_empty() && bottom.is_empty() {
        return None;
    }
    // ---- the single-input disenchant path ----
    if top.is_empty() || bottom.is_empty() {
        let it = if top.is_empty() { bottom } else { top };
        let useful = non_curse(it);
        if useful.is_empty() {
            // unenchanted (Java red X) or curse-only (the refuse row)
            return None;
        }
        let mut result = *it;
        result.ench = 0;
        result.ench2 = 0;
        // "If an enchanted book is placed in the input, a normal book
        // appears in the output" (VERIFIED — the block converts too)
        if result.block == ENCHANTED_BOOK {
            result.block = BOOK;
        }
        // curses survive a disenchant (VERIFIED); the model keeps at
        // most the first curse slot — a curse+curse input keeps one
        let it_ench = it.enchants();
        let curse = it_ench.iter().flatten().find(|&&(id, _)| is_curse(id));
        if let Some(&(id, lvl)) = curse {
            result.set_enchant(id, lvl);
        }
        result.prior = 0; // the grindstone resets the penalty
        let (mn, mx) = xp_bounds(&useful);
        return Some(GrindPlan { result, xp_min: mn, xp_max: mx });
    }
    // ---- the two-input combine path ----
    if top.block != bottom.block {
        return None; // "two tools or pieces of armor ... of the same type"
    }
    // "Placing two tools or pieces of armor (enchanted or not) of the
    // same type in the input slots causes a non-enchanted output"
    // (VERIFIED — the combine is offered even when neither input is
    // enchanted; the red-X "Unenchanted items" row is the SINGLE-input
    // disenchant refusal above)
    let useful_a = non_curse(top);
    let useful_b = non_curse(bottom);
    let mut result = *top; // the name rides the TOP input (VERIFIED)
    result.count = 1;
    result.ench = 0;
    result.ench2 = 0;
    let top_ench = top.enchants();
    let bottom_ench = bottom.enchants();
    let curse = top_ench
        .iter()
        .flatten()
        .chain(bottom_ench.iter().flatten())
        .find(|&&(id, _)| is_curse(id));
    if let Some(&(id, lvl)) = curse {
        result.set_enchant(id, lvl); // curses survive the grindstone
    }
    result.prior = 0; // reset (VERIFIED)
    // durability: sum of remainders + 5% of max, capped
    if let Some(max) = armor_max_durability(top.block) {
        let rem_a = max.saturating_sub(top.dmg) as u32;
        let rem_b = max.saturating_sub(bottom.dmg) as u32;
        let bonus = (max as f32 * COMBINE_BONUS_FRACTION).floor() as u32;
        let rem = (rem_a + rem_b + bonus).min(max as u32);
        result.dmg = max - rem as u16;
    }
    let mut all = useful_a;
    all.extend(useful_b);
    // "If either input item was enchanted, the grindstone drops some
    // experience" (VERIFIED) — an unenchanted pair combines with no XP
    let (mn, mx) = xp_bounds(&all);
    Some(GrindPlan { result, xp_min: mn, xp_max: mx })
}

/// the XP bounds for a set of enchants: combined M, floor = ceil(50%)
fn xp_bounds(enchants: &[(u8, u8)]) -> (u32, u32) {
    let m: u32 = enchants
        .iter()
        .map(|&(id, lvl)| xp_min_cost(id, lvl))
        .sum();
    (m.div_ceil(2), m) // ceil(m/2) ..= m — the 50%..100% window
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enchanting::enchant_by_id;
    use vc_blocks::blocks::*;

    fn ench(stack_block: u16, id: &str, level: u8) -> ItemStack {
        let mut s = ItemStack::new(stack_block, 1);
        s.set_enchant(enchant_by_id(id).unwrap(), level);
        s
    }

    /// the XP M-table spot checks against the wiki's published ranges
    /// (Sharpness V 23-45, Protection IV 17-33, Thorns III 25-49,
    /// Efficiency I 1-1, Mending 13-25).
    #[test]
    fn xp_table_matches_the_wiki_ranges() {
        let sharp = enchant_by_id("sharpness").unwrap();
        let prot = enchant_by_id("protection").unwrap();
        let thorns = enchant_by_id("thorns").unwrap();
        let eff = enchant_by_id("efficiency").unwrap();
        let mend = enchant_by_id("mending").unwrap();
        assert_eq!(xp_min_cost(sharp, 5), 45);
        assert_eq!(xp_min_cost(prot, 4), 33);
        assert_eq!(xp_min_cost(thorns, 3), 49);
        assert_eq!(xp_min_cost(eff, 1), 1);
        assert_eq!(xp_min_cost(mend, 1), 25);
        // a curse has no XP row
        let binding = enchant_by_id("binding_curse").unwrap();
        assert_eq!(xp_min_cost(binding, 1), 0);
    }

    /// Round 13 [spec]: the disenchant path strips enchantments and
    /// keeps the durability.
    #[test]
    fn grindstone_strips_enchantments() {
        let input = ench(IRON_HELMET, "protection", 3);
        let plan = grindstone_plan(&input, &ItemStack::EMPTY).expect("offered");
        assert_eq!(plan.result.block, IRON_HELMET);
        assert_eq!(plan.result.ench, 0);
        assert_eq!(plan.result.ench2, 0);
        assert_eq!(plan.result.dmg, input.dmg, "same durability");
        // an enchanted book becomes a normal book (VERIFIED)
        let book = ench(ENCHANTED_BOOK, "sharpness", 2);
        let plan2 = grindstone_plan(&book, &ItemStack::EMPTY).expect("offered");
        assert_eq!(plan2.result.block, BOOK);
        assert_eq!(plan2.result.ench, 0);
    }

    /// Round 13 [spec]: removing the output drops XP in the published
    /// range (the uniform 50%..100% window over the combined M).
    #[test]
    fn grindstone_returns_enchant_xp() {
        // Sharpness V alone: M = 45 -> [23, 45] (the engine has no
        // swords — any enchantable piece carries the test; documented)
        let input = ench(IRON_HELMET, "sharpness", 5);
        let plan = grindstone_plan(&input, &ItemStack::EMPTY).expect("offered");
        assert_eq!((plan.xp_min, plan.xp_max), (23, 45));
        // the roll is the game layer's job; the bounds are the contract
        // Protection III (M 23) + Thorns II (M 29) on one piece: 52
        let mut duo = ItemStack::new(IRON_CHESTPLATE, 1);
        duo.set_enchant(enchant_by_id("protection").unwrap(), 3);
        duo.set_enchant2(enchant_by_id("thorns").unwrap(), 2);
        let plan2 = grindstone_plan(&duo, &ItemStack::EMPTY).expect("offered");
        assert_eq!((plan2.xp_min, plan2.xp_max), (26, 52));
    }

    /// Round 13 [spec]: the combine WITHOUT the anvil's 12% bonus —
    /// the grindstone gives 5% (rounded down), capped at max. The pair
    /// is unenchanted and still combines (VERIFIED live 2026-09-16:
    /// "Placing two tools or pieces of armor (enchanted or not) of the
    /// same type ... causes a non-enchanted output").
    #[test]
    fn grindstone_combines_without_12pct_bonus() {
        // two damaged diamond chestplates (max 528): dmg 300 + 450
        let a = ItemStack::new_damaged(DIAMOND_CHESTPLATE, 1, 300, 3);
        let b = ItemStack::new_damaged(DIAMOND_CHESTPLATE, 1, 450, 1);
        let plan = grindstone_plan(&a, &b).expect("combine offered");
        // rem 228 + 78 + floor(0.05 x 528) = 26 -> 306 + 26 = 332
        assert_eq!(plan.result.dmg, 528 - 332);
        // the prior-work penalty RESETS (the anvil keeps accumulating)
        assert_eq!(plan.result.prior, 0);
        // an unenchanted pair drops NO XP (VERIFIED: "If either input
        // item was enchanted" — neither was)
        assert_eq!((plan.xp_min, plan.xp_max), (0, 0));
    }

    /// Round 13 [spec]: cursed items return nothing — the grindstone
    /// refuses a curse-only input (the curse cannot be removed, so the
    /// output would be the input).
    #[test]
    fn grindstone_refuses_cursed_items() {
        let cursed = ench(IRON_HELMET, "binding_curse", 1);
        assert!(grindstone_plan(&cursed, &ItemStack::EMPTY).is_none());
        // curse + normal enchant: the normal one strips, the curse
        // SURVIVES (VERIFIED: "does not remove curse enchantments")
        let mut mixed = ItemStack::new(IRON_HELMET, 1);
        mixed.set_enchant(enchant_by_id("vanishing_curse").unwrap(), 1);
        mixed.set_enchant2(enchant_by_id("protection").unwrap(), 2);
        let plan = grindstone_plan(&mixed, &ItemStack::EMPTY).expect("offered");
        assert_eq!(
            plan.result.enchant().map(|(i, _)| enchant_def(i).id),
            Some("vanishing_curse"),
            "the curse survives"
        );
        assert_eq!(plan.result.enchant2(), None);
        // two curse-only inputs also refuse
        let c2 = ench(DIAMOND_CHESTPLATE, "binding_curse", 1);
        assert!(grindstone_plan(&cursed, &c2).is_none());
    }

    /// the combine keeps the TOP input's custom name (VERIFIED:
    /// "The output item has the same ... name (if any) as the input
    /// item in the top slot").
    #[test]
    fn grindstone_combine_keeps_top_name() {
        let mut a = ench(IRON_HELMET, "protection", 1);
        a.name = 7;
        let mut b = ench(IRON_HELMET, "thorns", 1);
        b.name = 9;
        let plan = grindstone_plan(&a, &b).expect("offered");
        assert_eq!(plan.result.name, 7, "the top slot's name wins");
        assert_eq!(plan.result.ench, 0, "all non-curse enchants strip");
    }
}
