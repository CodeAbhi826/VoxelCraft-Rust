//! Enchanting (Phase 7 §29): the vanilla-observable table mechanics —
//! bookshelf power (the 5×5 ring, two layers, cap 15), three option rows
//! scaled by power, level costs 1..3 (levels + lapis), and enchanted books
//! that carry their enchant through the inventory (ItemStack.ench).
//!
//! Vanilla-exact pieces (VERIFIED):
//! - XP level-up curve: 2L+7 / 5L−38 / 9L−158
//! - level cost = slot index + 1, paid in LEVELS and lapis 1:1
//! - bookshelf power = bookshelves in the 5×5 ring around the table at the
//!   table's own height and one above, capped at 15
//! - applying an enchant re-rolls the option list (new xpSeed)
//!
//! Documented adaptations (palette-bounded):
//! - lapis item = LAPIS_ORE block (no standalone lapis item in this engine)
//! - the enchantable target = ENCHANTED_BOOK (the book item-block); tools
//!   arrive with the future tool/combat system
//! - per-slot option-level curve and the enchant-weighted pick are a
//!   close approximation of vanilla's (documented, deterministic)

use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;
use vc_rng::rng::Rng;
use std::collections::HashMap;

/// one enchantment row offered by the table
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnchOption {
    /// the row's enchantment LEVEL (1..=30, the big number in vanilla)
    pub level: u8,
    /// enchant registry id
    pub ench: u8,
    /// enchant level (I..V)
    pub ench_level: u8,
    /// cost in player LEVELS (and lapis) = slot index + 1
    pub cost: u8,
}

/// the enchant registry (§29 data-driven): id = index
pub struct EnchantDef {
    pub name: &'static str,
    pub max_level: u8,
    /// vanilla rarity weight (higher = more common)
    pub weight: u32,
}

pub const ENCHANTS: &[EnchantDef] = &[
    EnchantDef { name: "Protection", max_level: 4, weight: 10 },
    EnchantDef { name: "Feather Falling", max_level: 4, weight: 5 },
    EnchantDef { name: "Sharpness", max_level: 5, weight: 10 },
    EnchantDef { name: "Efficiency", max_level: 5, weight: 10 },
    EnchantDef { name: "Unbreaking", max_level: 3, weight: 5 },
    EnchantDef { name: "Fortune", max_level: 3, weight: 2 },
    EnchantDef { name: "Silk Touch", max_level: 1, weight: 2 },
    EnchantDef { name: "Mending", max_level: 1, weight: 2 },
    EnchantDef { name: "Power", max_level: 5, weight: 10 },
    EnchantDef { name: "Looting", max_level: 3, weight: 2 },
];

/// registry lookup by id (§46: out-of-range ids fold to index 0)
pub fn enchant_def(id: u8) -> &'static EnchantDef {
    &ENCHANTS[(id as usize).min(ENCHANTS.len() - 1)]
}

/// Roman numeral for the enchant level display
pub fn roman(level: u8) -> &'static str {
    match level {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        _ => "?",
    }
}

/// XP needed to go from `level` to `level + 1` (vanilla 1.16.5, VERIFIED):
/// 2L+7 below 16, 5L−38 for 16..=30, 9L−158 above
pub fn xp_to_next(level: i32) -> i32 {
    if level < 16 {
        2 * level + 7
    } else if level < 31 {
        5 * level - 38
    } else {
        9 * level - 158
    }
}

/// vanilla xp granted by mining an ore (VERIFIED ranges; we pay the fixed
/// midpoint — the engine's ore drops are deterministic)
pub fn ore_xp(block: u8) -> i32 {
    match block {
        COAL_ORE => 1,       // 0..2
        LAPIS_ORE => 3,      // 2..5
        REDSTONE_ORE => 3,   // 1..5
        DIAMOND_ORE => 5,    // 3..7
        EMERALD_ORE => 5,    // 3..7
        _ => 0,
    }
}

/// vanilla xp granted when a smelted item is collected (VERIFIED):
/// glass 0.1, stone 0.1, terracotta 0.35 — stored as a float pool
pub fn smelt_xp(block: u8) -> f32 {
    match block {
        SAND => 0.1,
        COBBLE => 0.1,
        CLAY => 0.35,
        _ => 0.0,
    }
}

/// bookshelf power: bookshelves in the 5×5 outer ring (|dx| or |dz| == 2)
/// at the table's height and one above, capped at 15 (vanilla scan)
pub fn bookshelf_power(world: &vc_world::world::World, pos: [i32; 3]) -> u8 {
    let mut n = 0u8;
    for dy in 0..=1 {
        for dz in -2i32..=2 {
            for dx in -2i32..=2 {
                // the ring: skip the 3×3 center (vanilla never counts the
                // blocks directly around the table's column)
                if dx.abs() < 2 && dz.abs() < 2 {
                    continue;
                }
                let b = world.get_block(pos[0] + dx, pos[1] + dy, pos[2] + dz);
                if b == BOOKSHELF {
                    n += 1;
                    if n >= 15 {
                        return 15;
                    }
                }
            }
        }
    }
    n
}

/// one option row's enchantment level from power + row (adapted curve:
/// row share 1/3, 2/3, 3/3 of the 2×power spread + noise, capped 30 —
/// vanilla's exact per-row noise is a documented approximation here)
fn row_level(power: u8, row: usize, rng: &mut Rng) -> u8 {
    let base = (power as u32 * (row as u32 + 1) * 2 / 3) as u8;
    let noise = if power > 1 {
        rng.next_range(power as u32 / 2 + 1) as u8
    } else {
        0
    };
    (base + 1 + noise).max(1).min(30)
}

/// weighted-random enchant pick from the registry (books accept all)
fn pick_enchant(rng: &mut Rng) -> u8 {
    let total: u32 = ENCHANTS.iter().map(|e| e.weight).sum();
    let mut pick = rng.next_range(total);
    for (i, e) in ENCHANTS.iter().enumerate() {
        if pick < e.weight {
            return i as u8;
        }
        pick -= e.weight;
    }
    0
}

/// enchant level for an option: scales with the row level toward the max
/// (option level 30 → the enchant's max level; ~half → ~half the levels)
fn ench_level_for(def: &EnchantDef, level: u8) -> u8 {
    let l = (level as u32 * def.max_level as u32 + 29) / 30;
    l.max(1).min(def.max_level as u32) as u8
}

/// generate the three option rows from a power + seed
pub fn gen_options(power: u8, seed: u64) -> [EnchOption; 3] {
    let mut rng = Rng::new(seed);
    let mut out = [EnchOption { level: 0, ench: 0, ench_level: 0, cost: 0 }; 3];
    for (row, o) in out.iter_mut().enumerate() {
        let level = row_level(power, row, &mut rng);
        let id = pick_enchant(&mut rng);
        let def = enchant_def(id);
        *o = EnchOption {
            level,
            ench: id,
            ench_level: ench_level_for(def, level),
            cost: (row + 1) as u8,
        };
    }
    out
}

/// enchanting-table block entity: item + lapis slots and the current offer
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantState {
    /// the item being enchanted (books; ItemStack.ench carries the result)
    pub item: ItemStack,
    /// lapis slot (LAPIS_ORE stacks)
    pub lapis: ItemStack,
    /// current offer rows (re-rolled on item change / successful enchant)
    pub options: [EnchOption; 3],
    /// bookshelf power snapshot at the last re-roll
    pub power: u8,
    /// the xpSeed analogue — re-rolls derive from world seed + position
    /// + a monotonically advancing counter
    pub seed: u64,
}

impl Default for EnchantState {
    fn default() -> Self {
        EnchantState {
            item: ItemStack::EMPTY,
            lapis: ItemStack::EMPTY,
            options: [EnchOption { level: 0, ench: 0, ench_level: 0, cost: 0 }; 3],
            power: 0,
            seed: 0,
        }
    }
}

impl EnchantState {
    /// re-roll the options (fresh power scan + advancing seed)
    pub fn reroll(&mut self, world: &vc_world::world::World, pos: [i32; 3], base_seed: u64) {
        self.power = bookshelf_power(world, pos);
        self.seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.options = gen_options(self.power, base_seed ^ self.seed ^ (pos[0] as u64) << 32 ^ pos[2] as u64);
    }

    /// can this offer be paid for? (vanilla: levels + lapis + an item)
    pub fn can_apply(&self, row: usize, player_level: i32) -> bool {
        let Some(o) = self.options.get(row) else { return false };
        o.level > 0
            && !self.item.is_empty()
            && self.item.block == ENCHANTED_BOOK
            && self.lapis.count >= o.cost
            && player_level >= o.cost as i32
    }

    /// apply an offer: mutates the item (ench field), returns the consumed
    /// cost on success (the caller pays the levels/lapis)
    pub fn apply(&mut self, row: usize) -> Option<u8> {
        let o = self.options.get(row).copied()?;
        if o.level == 0 || self.item.is_empty() || self.item.block != ENCHANTED_BOOK {
            return None;
        }
        self.item.set_enchant(o.ench, o.ench_level);
        Some(o.cost)
    }
}

/// all enchanting-table block entities (not ticked — reactive, like the
/// vanilla container)
#[derive(Default)]
pub struct Enchants {
    pub map: HashMap<[i32; 3], EnchantState>,
    /// total enchants applied since boot (stats/F3/E2E)
    pub total_enchanted: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_chunk::chunk::Chunk;
    use std::sync::Arc;

    fn flat_world() -> vc_world::world::World {
        let mut w = vc_world::world::World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
                for y in 0..=64i32 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y as usize, lz, STONE);
                        }
                    }
                }
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w
    }

    #[test]
    fn xp_curve_matches_vanilla() {
        // VERIFIED data points from the vanilla wiki table
        assert_eq!(xp_to_next(0), 7);
        assert_eq!(xp_to_next(1), 9);
        assert_eq!(xp_to_next(15), 37); // 5*15-38
        assert_eq!(xp_to_next(16), 42); // 5*16-38
        assert_eq!(xp_to_next(30), 112); // 9*30-158
        assert_eq!(xp_to_next(31), 121);
    }

    #[test]
    fn bookshelf_ring_counts_two_layers() {
        let mut w = flat_world();
        let pos = [0, 65, 0];
        assert_eq!(bookshelf_power(&w, pos), 0);
        // one bookshelf in the ring at dy=0
        let _ = w.set_block(2, 65, 0, BOOKSHELF);
        assert_eq!(bookshelf_power(&w, pos), 1);
        // one at dy=1 also counts
        let _ = w.set_block(-2, 66, 0, BOOKSHELF);
        assert_eq!(bookshelf_power(&w, pos), 2);
        // the 3×3 center never counts (vanilla scan)
        let _ = w.set_block(1, 65, 1, BOOKSHELF);
        let _ = w.set_block(1, 66, 1, BOOKSHELF);
        assert_eq!(bookshelf_power(&w, pos), 2);
        // cap at 15
        for dy in 0..=1 {
            for dz in [-2, 2] {
                for dx in -2i32..=2 {
                    let _ = w.set_block(dx, 65 + dy, dz, BOOKSHELF);
                }
            }
            for dx in [-2, 2] {
                for dz in -1i32..=1 {
                    let _ = w.set_block(dx, 65 + dy, dz, BOOKSHELF);
                }
            }
        }
        assert_eq!(bookshelf_power(&w, pos), 15);
    }

    #[test]
    fn options_scale_with_power_and_rows() {
        let low = gen_options(0, 42);
        let high = gen_options(15, 42);
        // zero power: every row is a low level
        assert!(low.iter().all(|o| o.level <= 2), "{low:?}");
        // full power: the top row reaches near 30
        assert!(high[2].level >= 25, "{high:?}");
        assert!(high[2].level <= 30);
        // rows are strictly non-decreasing in cost 1..3
        for (i, o) in high.iter().enumerate() {
            assert_eq!(o.cost as usize, i + 1);
        }
        // same seed → same options (deterministic for E2E)
        assert_eq!(gen_options(15, 42), high);
        // every option references a valid enchant with a valid level
        for o in high.iter() {
            let def = enchant_def(o.ench);
            assert!(o.ench_level >= 1 && o.ench_level <= def.max_level);
            assert!(o.level >= 1 && o.level <= 30);
        }
    }

    #[test]
    fn apply_enchants_the_book_and_rerolls() {
        let mut w = flat_world();
        let pos = [4, 65, 4];
        // bookshelves for real power
        let _ = w.set_block(6, 65, 4, BOOKSHELF);
        let _ = w.set_block(6, 66, 4, BOOKSHELF);
        let mut st = EnchantState::default();
        st.reroll(&w, pos, 123);
        let before = st.options;
        st.item = ItemStack::new(ENCHANTED_BOOK, 1);
        st.lapis = ItemStack::new(LAPIS_ORE, 3);
        assert!(st.can_apply(2, 30));
        assert!(!st.can_apply(2, 2), "not enough levels");
        assert!(st.can_apply(0, 30));
        let cost = st.apply(1).expect("apply row 1");
        assert_eq!(cost, 2);
        assert_eq!(st.item.enchant().map(|(i, _)| i), Some(before[1].ench));
        assert_eq!(st.item.enchant().map(|(_, l)| l), Some(before[1].ench_level));
        // only books are enchantable
        st.item = ItemStack::new(STONE, 1);
        assert!(st.apply(1).is_none());
    }

    #[test]
    fn itemstack_ench_roundtrip() {
        let mut s = ItemStack::new(ENCHANTED_BOOK, 2);
        assert!(s.enchant().is_none());
        s.set_enchant(3, 4); // Unbreaking IV (level clamps per registry later)
        assert_eq!(s.enchant(), Some((3, 4)));
        // the enchant survives a full copy (slot move)
        let moved = s;
        assert_eq!(moved.enchant(), Some((3, 4)));
        // equality includes the enchant
        assert_ne!(moved, ItemStack::new(ENCHANTED_BOOK, 2));
    }

    #[test]
    fn ore_and_smelt_xp_tables() {
        assert_eq!(ore_xp(COAL_ORE), 1);
        assert_eq!(ore_xp(DIAMOND_ORE), 5);
        assert_eq!(ore_xp(STONE), 0);
        assert!((smelt_xp(SAND) - 0.1).abs() < 1e-6);
        assert!((smelt_xp(CLAY) - 0.35).abs() < 1e-6);
    }
}
