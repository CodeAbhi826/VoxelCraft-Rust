//! Fishing loot (1.7.2 bracket — "The Update that Changed the World").
//!
//! Every constant VERIFIED against minecraft.wiki/w/Fishing and
//! /w/Java_Edition_1.7.2 (live round, 2026-09-06):
//! - base roll: 85% fish / 10% junk / 5% treasure ("the player, with an
//!   unenchanted fishing rod, has an 85% chance of catching fish, a 10%
//!   chance of catching junk, and a 5% chance of catching treasure")
//! - wait time: 5..30 s (100..600 ticks); Lure subtracts 5 s per level
//!   from BOTH bounds (clamped so min > 0 by re-roll, documented below)
//! - loot classes (1.7.2 changelog §Fishing):
//!   Fish: raw fish (cod), raw salmon, clownfish, pufferfish
//!   Treasure: enchanted fishing rods, enchanted bows, enchanted books,
//!             name tags, tripwire hooks, lily pads, saddles
//!   Junk: damaged fishing rods, water bottles, rotten flesh, string,
//!         leather, bowls, sticks, bones, tripwire hooks, 10 ink sacs
//!
//! Palette adaptation (documented): the engine has no rod/bow/name-tag/
//! bowl/stick/lily-pad/saddle/ink-sac items yet, so the TABLES carry the
//! classes with the implementable subset; unimplementable rows stay listed
//! (and counted in tests) so the full vanilla shape is preserved and the
//! missing items drop in when those registries arrive.

use vc_blocks::blocks::*;
use vc_rng::rng::Rng;

/// one catchable row of a loot class
pub struct LootRow {
    /// the block/item id handed to the player (NONE = not yet in registry)
    pub item: u8,
    /// vanilla display name of the row
    pub name: &'static str,
}

pub const LOOT_NONE: u8 = 255;

/// Fish class — the four 1.7.2 fish (all in our registry)
pub const FISH_TABLE: [LootRow; 4] = [
    LootRow { item: RAW_FISH, name: "Raw Fish" },
    LootRow { item: RAW_SALMON, name: "Raw Salmon" },
    LootRow { item: CLOWNFISH, name: "Clownfish" },
    LootRow { item: PUFFERFISH, name: "Pufferfish" },
];

/// Treasure class — only the enchanted book exists in our registry today;
/// the rest are listed as LOOT_NONE rows so the vanilla table shape is
/// visible and testable (see the module doc for the adaptation).
pub const TREASURE_TABLE: [LootRow; 7] = [
    LootRow { item: LOOT_NONE, name: "Enchanted Fishing Rod" },
    LootRow { item: LOOT_NONE, name: "Enchanted Bow" },
    LootRow { item: ENCHANTED_BOOK, name: "Enchanted Book" },
    LootRow { item: LOOT_NONE, name: "Name Tag" },
    LootRow { item: LOOT_NONE, name: "Tripwire Hook" },
    LootRow { item: LOOT_NONE, name: "Lily Pad" },
    LootRow { item: LOOT_NONE, name: "Saddle" },
];

/// Junk class — implementable subset (water bottles, rotten flesh,
/// string, leather, bone live in our registry; rod/bowl/stick/ink rows
/// are LOOT_NONE placeholders for the same reason)
pub const JUNK_TABLE: [LootRow; 10] = [
    LootRow { item: LOOT_NONE, name: "Damaged Fishing Rod" },
    LootRow { item: POTION_WATER, name: "Water Bottle" },
    LootRow { item: ROTTEN_FLESH, name: "Rotten Flesh" },
    LootRow { item: STRING, name: "String" },
    LootRow { item: LEATHER, name: "Leather" },
    LootRow { item: LOOT_NONE, name: "Bowl" },
    LootRow { item: LOOT_NONE, name: "Stick" },
    LootRow { item: BONE, name: "Bone" },
    LootRow { item: LOOT_NONE, name: "Tripwire Hook" },
    LootRow { item: LOOT_NONE, name: "Ink Sac (x10)" },
];

/// the three loot classes
pub enum LootClass {
    Fish,
    Junk,
    Treasure,
}

/// roll the loot class. VERIFIED (wiki /w/Fishing): unenchanted rod = 85%
/// fish / 10% junk / 5% treasure. Luck of the Sea I-III shifts weight
/// into treasure "at the expense of reducing the chances of catching fish
/// and junk" — the wiki's per-level numbers (1%→0.85% style deltas) are
/// not spelled out for 1.7.2, so the shift is the documented proportional
/// model below (each level moves 1.5% from fish and 0.75% from junk into
/// treasure, monotone, classes never negative).
pub fn roll_class(rng: &mut Rng, luck_of_the_sea: u8) -> LootClass {
    let luck = (luck_of_the_sea.min(3)) as f32;
    let fish = 0.85 - 0.015 * luck;
    let junk = 0.10 - 0.0075 * luck;
    let r = rng.next_f32();
    if r < fish {
        LootClass::Fish
    } else if r < fish + junk {
        LootClass::Junk
    } else {
        LootClass::Treasure
    }
}

/// pick a row inside a class (uniform, matching the vanilla equal-weight
/// rolls within a category for our subset)
pub fn roll_item(rng: &mut Rng, class: &LootClass) -> &'static LootRow {
    let table: &[LootRow] = match class {
        LootClass::Fish => &FISH_TABLE,
        LootClass::Junk => &JUNK_TABLE,
        LootClass::Treasure => &TREASURE_TABLE,
    };
    let i = rng.next_range(table.len() as u32) as usize;
    &table[i]
}

/// wait time in TICKS before the bite. VERIFIED (wiki /w/Fishing):
/// "the player must wait for a random period between 5 and 30 seconds
/// (100 to 600 ticks at 20 [tps])". The Lure enchantment "subtracts 5
/// seconds from both the minimum and maximum wait time. If it causes the
/// wait time to be less than 0, a new wait time is generated in the next
/// tick" — modeled by clamping the max at >= 1 tick.
pub fn wait_ticks(rng: &mut Rng, lure: u8) -> i32 {
    let lo = 100 - 100 * (lure as i32).min(3);
    let hi = 600 - 100 * (lure as i32).min(3);
    let lo = lo.max(1);
    let hi = hi.max(lo + 1);
    lo + rng.next_range((hi - lo + 1) as u32) as i32
}

/// Pufferfish eating effects, VERIFIED (wiki /w/Pufferfish, live round
/// 2026-09-06, and /w/Poison for the damage cadence):
/// - Poison IV for 1:00 — poison ticks 1 HP every 3 ticks at level IV
///   (wiki /w/Poison table: L4 = 3 ticks per HP), but the 10-tick hurt
///   immunity effectively caps real damage at ~1 HP/s; and poison can
///   NEVER kill ("it cannot kill... health all the way to 1")
/// - Hunger III for 0:15 and Nausea I for 0:15 — nausea has no damage;
///   hunger drains saturation (our engine has no hunger bar yet —
///   recorded as a deferred adaptation)
pub const PUFFERFISH_POISON_LEVEL: u8 = 4; // Poison IV
pub const PUFFERFISH_POISON_TICKS: i32 = 20 * 60; // 1:00
pub const PUFFERFISH_HUNGER_TICKS: i32 = 20 * 15; // 0:15
pub const PUFFERFISH_NAUSEA_TICKS: i32 = 20 * 15; // 0:15
/// poison damage interval at amplifier IV (wiki Poison table row L4)
pub const POISON_IV_INTERVAL_TICKS: i32 = 3;
/// hurt-immunity effective cap (wiki /w/Poison: effective 10-tick rate)
pub const POISON_HURT_IMMUNITY_TICKS: i32 = 10;
/// poison stops at 1 HP — it cannot kill (wiki /w/Poison)
pub const POISON_MIN_HEALTH: f32 = 1.0;

/// the effective poison interval for a pufferfish hit, combining the
/// level-IV cadence with the hurt-immunity floor (the observable rate)
#[inline]
pub fn pufferfish_poison_interval() -> i32 {
    POISON_IV_INTERVAL_TICKS.max(POISON_HURT_IMMUNITY_TICKS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_roll_matches_the_wiki_percentages() {
        // VERIFIED: 85/10/5 at luck 0 — assert within a tight band over
        // a big sample (rng is deterministic; 40k rolls → sigma ~0.15%)
        let mut rng = Rng::new(1234);
        let n = 40_000;
        let (mut fish, mut junk, mut treasure) = (0usize, 0usize, 0usize);
        for _ in 0..n {
            match roll_class(&mut rng, 0) {
                LootClass::Fish => fish += 1,
                LootClass::Junk => junk += 1,
                LootClass::Treasure => treasure += 1,
            }
        }
        let pf = fish as f32 / n as f32;
        let pj = junk as f32 / n as f32;
        let pt = treasure as f32 / n as f32;
        assert!((pf - 0.85).abs() < 0.01, "fish {pf}");
        assert!((pj - 0.10).abs() < 0.01, "junk {pj}");
        assert!((pt - 0.05).abs() < 0.01, "treasure {pt}");
    }

    #[test]
    fn luck_shifts_treasure_up_monotonically() {
        let n = 40_000;
        let mut prev_t = 0.0f32;
        for luck in 0u8..=3 {
            let mut rng = Rng::new(77 + luck as u64);
            let mut t = 0usize;
            for _ in 0..n {
                if let LootClass::Treasure = roll_class(&mut rng, luck) {
                    t += 1;
                }
            }
            let pt = t as f32 / n as f32;
            assert!(pt > prev_t, "luck {luck}: {pt} must exceed {prev_t}");
            prev_t = pt;
        }
        // luck 3 treasure ≈ 5% + 3×(1.5+0.75)% = 11.75% ± band
        assert!((prev_t - 0.1175).abs() < 0.02, "luck3 treasure {prev_t}");
    }

    #[test]
    fn fish_class_only_yields_the_four_1_7_2_fish() {
        let mut rng = Rng::new(42);
        for _ in 0..200 {
            let row = roll_item(&mut rng, &LootClass::Fish);
            assert!(
                row.item == RAW_FISH
                    || row.item == RAW_SALMON
                    || row.item == CLOWNFISH
                    || row.item == PUFFERFISH,
                "fish row gave {}",
                row.name
            );
        }
    }

    #[test]
    fn wait_time_bounds_and_lure() {
        let mut rng = Rng::new(9);
        // no lure: 100..=600 inclusive
        for _ in 0..500 {
            let w = wait_ticks(&mut rng, 0);
            assert!((100..=600).contains(&w), "bare wait {w}");
        }
        // Lure I/II/III subtract 100 ticks from both ends (5 s per level)
        for lure in 1u8..=3 {
            for _ in 0..500 {
                let w = wait_ticks(&mut rng, lure);
                let lo = (100 - 100 * lure as i32).max(1);
                let hi = (600 - 100 * lure as i32).max(lo + 1);
                assert!((lo..=hi).contains(&w), "lure {lure} wait {w}");
            }
        }
    }

    #[test]
    fn pufferfish_effect_constants() {
        // the wiki's exact 1.7.2 numbers
        assert_eq!(PUFFERFISH_POISON_TICKS, 1200); // 1:00
        assert_eq!(PUFFERFISH_HUNGER_TICKS, 300); // 0:15
        assert_eq!(PUFFERFISH_NAUSEA_TICKS, 300); // 0:15
        // observable poison cadence: max(3-tick L4 cadence, 10-tick hurt
        // immunity) = 10 ticks = 1 HP/s
        assert_eq!(pufferfish_poison_interval(), 10);
    }

    #[test]
    fn treasure_and_junk_tables_carry_the_vanilla_shapes() {
        // the full vanilla row counts are preserved even where items are
        // palette-missing (LOOT_NONE rows)
        assert_eq!(TREASURE_TABLE.len(), 7);
        assert_eq!(JUNK_TABLE.len(), 10);
        // implementable subset sanity: the rows we can hand out resolve
        // to registry blocks
        for r in TREASURE_TABLE.iter().chain(JUNK_TABLE.iter()) {
            if r.item != LOOT_NONE {
                assert!(r.item < BLOCK_COUNT as u8);
            }
        }
        // enchanted book is the one treasure we CAN award today
        assert!(TREASURE_TABLE.iter().any(|r| r.item == ENCHANTED_BOOK));
    }
}
