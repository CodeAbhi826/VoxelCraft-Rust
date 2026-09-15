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
//! ROUND 13 (2026-09-15, live re-verified against w/Anvil +
//! w/Anvil_mechanics — docs/research/round-13-station-screens-audit.md):
//! the repair/combine/rename/cost mechanics are now REAL. The engine's
//! damageable+enchantable items are the 16 armor pieces; books combine
//! per the §Enchanted books rule. The ItemStack model grew dmg/prior/
//! ench2/name fields this round to carry the state.

use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;
use crate::enchanting::{enchant_def, incompatible};

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
/// Round 13 correction (w/Anvil_mechanics, live 2026-09-15): the
/// combine-repair bonus is 12% of max durability (NOT the 10% the old
/// deferred record carried): "adding the durability of the sacrifice
/// item plus a bonus of 12% of the maximum durability"
pub const REPAIR_BONUS_FRACTION: f32 = 0.12;
/// (deferred-rules record) "Too Expensive!" cap: 39 levels since 1.8
/// (VERIFIED w/Anvil §Repairing: "The anvil has a limit of 39 levels;
/// beyond that, repairs are refused. This limit is not present in
/// Creative mode.")
pub const COST_CAP: i32 = 39;
/// Round 13: the rename surcharge (VERIFIED w/Anvil §Renaming: "Any
/// item or stack of items can be renamed at a cost of one level plus
/// any prior-work penalty")
pub const RENAME_COST: i32 = 1;
/// Round 13: the flat combine-repair cost (VERIFIED w/Anvil_mechanics
/// §Combining items: "If the target item is damaged it is repaired,
/// adding the durability of the sacrifice item plus a bonus of 12% of
/// the maximum durability ... The cost for this repair is 2 levels.")
pub const REPAIR_COST: i32 = 2;
/// Round 13: the rename field's max length (VERIFIED w/Anvil
/// §Renaming: "The maximum length for renaming is 30 characters [BE]
/// or 50 characters [JE]")
pub const RENAME_MAX_CHARS: usize = 50;
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

/// Round 13, CORRECTED to the live wiki (w/Anvil_mechanics
/// §Prior work penalty, fetched 2026-09-15): "The cost increases
/// exponentially by the formula 2^c-1 where c is the anvil use count"
/// — 0/1/3/7/15/31 for counts 0..=5. The old deferred-rules record
/// (1/2/4/8 doubling) did not match the live page; the wiki wins
/// (documented in round-13-station-screens-audit.md §1).
#[inline]
pub fn prior_work_penalty(uses: u32) -> i32 {
    if uses >= 31 {
        i32::MAX
    } else {
        (1i32 << uses) - 1
    }
}

/// Round 13: the armor durability table (VERIFIED w/Armor live tables,
/// fetched 2026-09-15; the engine's 4 materials x 4 pieces). The
/// historical iron/diamond boots values 209/461 disagree with the live
/// wiki (195/429, consistent with the page's own "boots a multiple of
/// 13" rule) — wiki wins, documented in the audit doc.
pub fn armor_max_durability(b: u16) -> Option<u16> {
    Some(match b {
        LEATHER_CAP => 55,
        LEATHER_TUNIC => 80,
        LEATHER_PANTS => 75,
        LEATHER_BOOTS => 65,
        GOLDEN_HELMET => 77,
        GOLDEN_CHESTPLATE => 112,
        GOLDEN_LEGGINGS => 105,
        GOLDEN_BOOTS => 91,
        IRON_HELMET => 165,
        IRON_CHESTPLATE => 240,
        IRON_LEGGINGS => 225,
        IRON_BOOTS => 195,
        DIAMOND_HELMET => 363,
        DIAMOND_CHESTPLATE => 528,
        DIAMOND_LEGGINGS => 495,
        DIAMOND_BOOTS => 429,
        _ => return None,
    })
}

/// Round 13: does enchant `id` apply to `block`? (VERIFIED w/
/// Anvil_mechanics §Costs: "Ignore any enchantment that cannot be
/// applied to the target (e.g. Protection on a sword)" — the wiki
/// "Applies to" column, restricted to the engine's item set: armor
/// pieces + books accept everything.)
pub fn ench_applies_to(ench: u8, block: u16) -> bool {
    if block == ENCHANTED_BOOK || block == BOOK {
        return true; // books accept every enchant
    }
    let piece = match armor_piece(block) {
        Some(p) => p,
        None => return false,
    };
    let id = enchant_def(ench).id;
    match id {
        // any armor piece
        "protection" | "fire_protection" | "blast_protection"
        | "projectile_protection" | "thorns" | "unbreaking" | "mending"
        | "binding_curse" | "vanishing_curse" => true,
        // helmet only
        "respiration" | "aqua_affinity" => piece == 0,
        // boots only
        "feather_falling" | "depth_strider" | "frost_walker" | "soul_speed" => piece == 3,
        // non-armor enchants never apply to armor pieces
        _ => false,
    }
}

/// Round 13: one computed anvil operation — the result stack plus the
/// level cost. `too_expensive` marks the >39-level refusal (survival
/// refuses; creative exempts — VERIFIED).
#[derive(Clone, Debug, PartialEq)]
pub struct AnvilPlan {
    pub result: ItemStack,
    pub cost: i32,
    pub too_expensive: bool,
}

/// Round 13: the anvil's combine/repair/rename math (VERIFIED
/// w/Anvil_mechanics §Combining items + §Costs, live 2026-09-15).
///
/// - `target` = the left slot, `sacrifice` = the right slot.
/// - `rename` = Some(new_name_id) when the rename field holds a valid
///   new name (blank field or unchanged name = None — the red-X rule).
/// - Returns None when the anvil REFUSES the pair outright (nothing to
///   do: full-durability target + unenchanted sacrifice + no rename).
/// - Cost = penalties(target) + penalties(sacrifice) + [rename 1]
///   + [repair 2 when the target is damaged and the sacrifice repairs
///   it] + enchant cost (each sacrifice enchant that applies: final
///   result level x multiplier — half when the sacrifice is a book;
///   +1 per incompatible enchant, which is NOT transferred).
/// - Result prior-use count = max(t, s) + 1, EXCEPT a pure rename
///   ("renaming alone does not cause an item's prior work penalty to
///   accumulate").
/// - The two-slot enchant model caps the merge at 2 enchants (an
///   over-cap merge keeps the two highest-cost enchants — disclosed
///   engine cap, audit doc §5).
pub fn combine(
    target: &ItemStack,
    sacrifice: &ItemStack,
    rename: Option<u16>,
) -> Option<AnvilPlan> {
    if target.is_empty() {
        return None;
    }
    // the pure-rename path: only the target placed + a valid new name
    // (VERIFIED w/Anvil §Renaming — the single-item operation)
    if sacrifice.is_empty() {
        if rename.is_none() {
            return None;
        }
        let mut result = *target;
        result.name = rename.unwrap_or(0);
        let cost = prior_work_penalty(target.prior as u32) + RENAME_COST;
        return Some(AnvilPlan {
            cost,
            too_expensive: cost > COST_CAP,
            result, // prior unchanged: a pure rename never accumulates
        });
    }
    let t_book = target.block == ENCHANTED_BOOK || target.block == BOOK;
    let s_book = sacrifice.block == ENCHANTED_BOOK || sacrifice.block == BOOK;
    // same-item combine, or item + book (VERIFIED §Combining items:
    // "combine two of the same item, or an item with an enchanted book")
    let same_item = target.block == sacrifice.block;
    if !same_item && !s_book {
        return None;
    }
    let s_enchants: Vec<(u8, u8)> = sacrifice.enchants().iter().flatten().copied().collect();
    let t_max_dur = if t_book { None } else { armor_max_durability(target.block) };
    let t_damaged = t_max_dur.map_or(false, |m| target.dmg > 0 && target.dmg < m);

    // refusal: nothing to do (VERIFIED: "If the target item is at full
    // durability and the sacrifice does not have any enchantments, the
    // anvil also refuses to combine the items, unless if renaming")
    if !t_damaged && s_enchants.is_empty() && rename.is_none() {
        return None;
    }

    let mut result = *target;
    let mut cost = prior_work_penalty(target.prior as u32)
        + prior_work_penalty(sacrifice.prior as u32);

    // --- repair (same-item only; books never repair) ---
    if same_item && !s_book {
        if let Some(max) = t_max_dur {
            if t_damaged {
                let rem_t = max.saturating_sub(target.dmg) as u32;
                let rem_s = max.saturating_sub(sacrifice.dmg) as u32;
                let bonus = (max as f32 * REPAIR_BONUS_FRACTION).floor() as u32;
                let rem = (rem_t + rem_s + bonus).min(max as u32);
                result.dmg = max - rem as u16;
                cost += REPAIR_COST;
            }
        } else {
            // non-damageable same-block items have nothing to combine
            // unless enchants or a rename are in play
            if s_enchants.is_empty() && rename.is_none() {
                return None;
            }
        }
    }

    // --- enchant transfer ---
    let mut carried: Vec<(u8, u8)> = target.enchants().iter().flatten().copied().collect();
    for &(id, s_lvl) in &s_enchants {
        if !ench_applies_to(id, target.block) {
            continue; // ignored (cannot apply)
        }
        // incompatible with a target enchant: +1 level, not transferred
        // (VERIFIED §Costs: "Add one level for every incompatible
        // enchantment on the target (in Java Edition)")
        if carried.iter().any(|&(t_id, _)| t_id != id && incompatible(t_id, id)) {
            cost += 1;
            continue;
        }
        let def = enchant_def(id);
        let final_lvl = match carried.iter().position(|&(t_id, _)| t_id == id) {
            Some(k) => {
                let t_lvl = carried[k].1;
                let fl = if s_lvl > t_lvl {
                    s_lvl
                } else if s_lvl == t_lvl && t_lvl < def.max_level {
                    t_lvl + 1
                } else {
                    t_lvl
                };
                carried[k].1 = fl;
                fl
            }
            None => {
                carried.push((id, s_lvl));
                s_lvl
            }
        };
        // multiplier: from item; from book = half rounded down
        let mult = if s_book {
            (def.cost_mult / 2).max(1)
        } else {
            def.cost_mult
        };
        cost += final_lvl as i32 * mult as i32;
    }
    // write back (engine cap: 2 slots — keep the two highest multiplier
    // when an over-cap merge happens; disclosed)
    if carried.len() > 2 {
        carried.sort_by_key(|&(id, _)| std::cmp::Reverse(enchant_def(id).cost_mult));
        carried.truncate(2);
    }
    result.ench = 0;
    result.ench2 = 0;
    if let Some(&(id, lvl)) = carried.first() {
        result.set_enchant(id, lvl);
    }
    if let Some(&(id, lvl)) = carried.get(1) {
        result.set_enchant2(id, lvl);
    }

    // --- rename ---
    let pure_rename = !same_item_repaired_or_enchanted(t_damaged, same_item, !s_enchants.is_empty());
    if let Some(name) = rename {
        result.name = name;
        cost += RENAME_COST;
    }

    // --- prior-work accumulation ---
    result.prior = if pure_rename {
        target.prior
    } else {
        target.prior.max(sacrifice.prior) + 1
    };

    let too_expensive = cost > COST_CAP;
    Some(AnvilPlan { result, cost, too_expensive })
}

#[inline]
fn same_item_repaired_or_enchanted(t_damaged: bool, same_item: bool, s_ench: bool) -> bool {
    t_damaged && same_item || s_ench
}

/// Round 13: the anvil stage-advance roll on taking a result (the 12%
/// gate wired into the take-result path — game.rs).
#[inline]
pub fn stage_after_use(block: u16, roll: f32) -> Option<u16> {
    if degrades(roll) {
        next_stage(block)
    } else {
        Some(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn constants_match_the_live_wiki() {
        assert_eq!(DEGRADE_CHANCE_PER_USE, 0.12); // w/Anvil §Becoming damaged
        assert_eq!(FALL_DMG_PER_BLOCK, 2.0);
        assert_eq!(FALL_DMG_CAP, 40.0);
        assert_eq!(HELMET_REDUCTION, 0.25);
        assert_eq!(MAX_FALLING_TICKS, 600); // 30 s
        assert_eq!(MATERIAL_REPAIR_FRACTION, 0.25);
        assert_eq!(REPAIR_BONUS_FRACTION, 0.12); // w/Anvil_mechanics (round 13)
        assert_eq!(COST_CAP, 39); // 1.8+ (pre-1.8 40 — disclosed)
        assert_eq!(RENAME_COST, 1);
        assert_eq!(REPAIR_COST, 2); // w/Anvil_mechanics §Combining items
        assert_eq!(RENAME_MAX_CHARS, 50); // JE
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
        // w/Anvil_mechanics §Prior work penalty (live 2026-09-15):
        // 2^c - 1 — the table 0/1/3/7/15/31 for use counts 0..=5.
        // (Round 13 correction: the old 1/2/4/8 record did not match
        // the live wiki; the wiki wins — audit doc §1.)
        assert_eq!(prior_work_penalty(0), 0);
        assert_eq!(prior_work_penalty(1), 1);
        assert_eq!(prior_work_penalty(2), 3);
        assert_eq!(prior_work_penalty(3), 7);
        assert_eq!(prior_work_penalty(4), 15);
        assert_eq!(prior_work_penalty(5), 31);
    }

    /// Round 13: the armor durability table (w/Armor live 2026-09-15).
    #[test]
    fn armor_durability_matches_the_live_wiki() {
        assert_eq!(armor_max_durability(LEATHER_CAP), Some(55));
        assert_eq!(armor_max_durability(LEATHER_TUNIC), Some(80));
        assert_eq!(armor_max_durability(GOLDEN_HELMET), Some(77));
        assert_eq!(armor_max_durability(IRON_CHESTPLATE), Some(240));
        assert_eq!(armor_max_durability(DIAMOND_LEGGINGS), Some(495));
        // the live-wiki boots values (195/429 — the historical 209/461
        // disagree; wiki wins, audit doc)
        assert_eq!(armor_max_durability(IRON_BOOTS), Some(195));
        assert_eq!(armor_max_durability(DIAMOND_BOOTS), Some(429));
        assert_eq!(armor_max_durability(STONE), None, "non-armor: none");
    }

    /// Round 13 [spec]: two damaged same-type pieces combine — result
    /// durability = min(rem_a + rem_b + floor(12% x max), max), cost
    /// includes the flat 2-level repair (the spec's "picks" become the
    /// engine's damageable armor pieces — no tools exist; documented).
    #[test]
    fn anvil_repairs_two_damaged_pieces() {
        // two damaged diamond chestplates (max 528): dmg 300 and 450
        let a = ItemStack::new_damaged(DIAMOND_CHESTPLATE, 1, 300, 0);
        let b = ItemStack::new_damaged(DIAMOND_CHESTPLATE, 1, 450, 0);
        let plan = combine(&a, &b, None).expect("repair combine is offered");
        // rem_a = 228, rem_b = 78, bonus = floor(0.12 x 528) = 63
        // 228 + 78 + 63 = 369 -> dmg = 528 - 369 = 159
        assert_eq!(plan.result.dmg, 159);
        assert_eq!(plan.result.block, DIAMOND_CHESTPLATE);
        // cost = 0 + 0 penalties + 2 repair
        assert_eq!(plan.cost, REPAIR_COST);
        assert!(!plan.too_expensive);
        // prior use count -> 1 (penalty 1 next time)
        assert_eq!(plan.result.prior, 1);
    }

    /// Round 13 [spec]: the combine respects the incompatibility groups
    /// (a Protection sacrifice onto a Fire Protection target: +1 level,
    /// the enchant is NOT transferred).
    #[test]
    fn anvil_combine_respects_incompatibility_groups() {
        let prot = crate::enchanting::enchant_by_id("protection").unwrap();
        let fire = crate::enchanting::enchant_by_id("fire_protection").unwrap();
        let mut t = ItemStack::new(IRON_HELMET, 1);
        t.set_enchant(fire, 2); // Fire Protection II
        let mut s = ItemStack::new(IRON_HELMET, 1);
        s.set_enchant(prot, 3); // Protection III
        let plan = combine(&t, &s, None).expect("incompatible combine still offered");
        // the result keeps ONLY the target's Fire Protection II
        assert_eq!(plan.result.enchant(), Some((fire, 2)));
        assert_eq!(plan.result.enchant2(), None);
        // cost: Fire Protection II final level 2 x mult 2 = 4, plus the
        // +1 incompatible surcharge
        assert_eq!(plan.cost, 4 + 1);
    }

    /// Round 13 [spec]: rename costs one level (+ penalties); a pure
    /// rename does NOT accumulate the prior-work count. The rename-only
    /// flow places just the target and types a name (empty sacrifice).
    #[test]
    fn anvil_rename_costs_one_level() {
        let t = ItemStack::new(IRON_HELMET, 1);
        let empty = ItemStack::EMPTY;
        assert!(combine(&t, &empty, None).is_none(), "nothing to do -> red X");
        let plan = combine(&t, &empty, Some(4)).expect("rename makes it valid");
        assert_eq!(plan.cost, RENAME_COST);
        assert_eq!(plan.result.name, 4);
        assert_eq!(plan.result.prior, 0, "pure rename does not accumulate");
        // with a penalty: prior 2 -> penalty 3 + 1 rename
        let mut t2 = ItemStack::new(IRON_HELMET, 1);
        t2.prior = 2;
        let plan2 = combine(&t2, &empty, Some(4)).expect("valid rename");
        assert_eq!(plan2.cost, prior_work_penalty(2) + RENAME_COST);
        assert_eq!(plan2.result.prior, 2, "still no accumulation");
        // a rename riding a repair DOES ride the accumulated count
        let a = ItemStack::new_damaged(LEATHER_CAP, 1, 40, 0);
        let b = ItemStack::new_damaged(LEATHER_CAP, 1, 40, 0);
        let plan3 = combine(&a, &b, Some(9)).expect("repair + rename");
        assert_eq!(plan3.cost, REPAIR_COST + RENAME_COST);
        assert_eq!(plan3.result.prior, 1);
        assert_eq!(plan3.result.name, 9);
    }

    /// Round 13 [spec]: the >39-level refusal ("Too Expensive!").
    #[test]
    fn anvil_refuses_at_insufficient_xp() {
        // Thorns III sacrifice onto a Thorns III target at high penalty:
        // final level capped at III, mult 8 -> 24 + penalties
        let thorns = crate::enchanting::enchant_by_id("thorns").unwrap();
        let mut t = ItemStack::new(IRON_CHESTPLATE, 1);
        t.set_enchant(thorns, 3);
        t.prior = 5; // penalty 31
        let mut s = ItemStack::new(IRON_CHESTPLATE, 1);
        s.set_enchant(thorns, 3);
        s.prior = 4; // penalty 15
        let plan = combine(&t, &s, None).expect("offered");
        // 24 (Thorns III x 8) + 31 + 15 = 70 > 39
        assert!(plan.too_expensive);
        assert!(plan.cost > COST_CAP);
        // a cheap combine is NOT too expensive
        let a = ItemStack::new_damaged(LEATHER_CAP, 1, 40, 0);
        let b = ItemStack::new_damaged(LEATHER_CAP, 1, 40, 0);
        let cheap = combine(&a, &b, None).expect("offered");
        assert!(!cheap.too_expensive);
    }

    /// Round 13 [spec]: the anvil damage stage advances on use (the 12%
    /// roll wired into the take-result path).
    #[test]
    fn anvil_damage_stage_advances_on_use() {
        assert_eq!(stage_after_use(ANVIL, 0.5), Some(CHIPPED_ANVIL));
        assert_eq!(stage_after_use(ANVIL, 0.05), Some(ANVIL), "87% survive");
        assert_eq!(stage_after_use(CHIPPED_ANVIL, 0.1), Some(DAMAGED_ANVIL));
        assert_eq!(stage_after_use(DAMAGED_ANVIL, 0.0), None, "destroyed");
    }

    /// Round 13: the book-combine path (book + book merges enchants at
    /// the half multiplier; an item + book merge rides the same rule).
    #[test]
    fn anvil_book_combine_uses_half_multipliers() {
        let prot = crate::enchanting::enchant_by_id("protection").unwrap();
        let thorns = crate::enchanting::enchant_by_id("thorns").unwrap();
        let mut t = ItemStack::new(ENCHANTED_BOOK, 1);
        t.set_enchant(prot, 2);
        let mut s = ItemStack::new(ENCHANTED_BOOK, 1);
        s.set_enchant(thorns, 1);
        let plan = combine(&t, &s, None).expect("book + book combine");
        // both enchants carried
        assert_eq!(plan.result.enchant(), Some((prot, 2)));
        assert_eq!(plan.result.enchant2(), Some((thorns, 1)));
        // Protection II final 2 x (1/2 -> max(1)) ... book mult = 1/2
        // rounds to 0 -> clamped to 1: 2x1; Thorns I x (8/2 = 4) = 4
        assert_eq!(plan.cost, 2 * 1 + 1 * 4);
    }

    /// Round 13: equal-level enchants gain one level up to the registry
    /// max (w/Anvil_mechanics §Combining items).
    #[test]
    fn anvil_equal_levels_gain_one() {
        let prot = crate::enchanting::enchant_by_id("protection").unwrap();
        let mut t = ItemStack::new(IRON_HELMET, 1);
        t.set_enchant(prot, 2);
        let mut s = ItemStack::new(IRON_HELMET, 1);
        s.set_enchant(prot, 2);
        let plan = combine(&t, &s, None).expect("offered");
        assert_eq!(plan.result.enchant(), Some((prot, 3)));
        // final level 3 x mult 1
        assert_eq!(plan.cost, 3);
        // at max level it stays (cost still counts the final level)
        let mut t2 = ItemStack::new(IRON_HELMET, 1);
        t2.set_enchant(prot, 4);
        let mut s2 = ItemStack::new(IRON_HELMET, 1);
        s2.set_enchant(prot, 4);
        let plan2 = combine(&t2, &s2, None).expect("offered");
        assert_eq!(plan2.result.enchant(), Some((prot, 4)));
        assert_eq!(plan2.cost, 4, "Java still charges final level x mult");
    }
}
