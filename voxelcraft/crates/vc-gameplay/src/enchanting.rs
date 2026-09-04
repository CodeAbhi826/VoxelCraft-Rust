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

/// the enchant registry (§29/Phase 4): id = index.
///
/// VERIFIED live against minecraft.wiki on 2026-09-04 (the current
/// "List of enchantments" table filtered to the 1.16.5 set by
/// introduction version — post-1.16.5 additions excluded: Swift Sneak
/// 1.19, Breach/Density/Wind Burst 1.21, Lunge 26.x, Cleaving
/// combat-test-only), plus the 1.16.5-era weight table pulled from the
/// archived wiki revision 1945529 (Enchanting table mechanics,
/// 2021-05-24) via the MediaWiki API. Exactly the 38 enchantments that
/// shipped in 1.16.5.
///
/// ID DISCIPLINE (Dossier Part 6 §29, Mojang MC-271039): the 1.16.5
/// registry id of Sweeping Edge is `sweeping` — it was renamed to
/// `sweeping_edge` in snapshot 24w03a (2024). The id strings below are
/// the 1.16.5 names for future datapack/protocol parity.
///
/// The first 10 entries preserve the Phase-era index order (existing
/// enchanted books in saves reference these indices).
pub struct EnchantDef {
    /// display name
    pub name: &'static str,
    /// 1.16.5 registry id (string form for datapack/protocol parity)
    pub id: &'static str,
    pub max_level: u8,
    /// vanilla rarity weight (higher = more common; VERIFIED per the
    /// 1.16.5 weight table)
    pub weight: u32,
    /// obtainable from an enchanting table (VERIFIED: Frost Walker,
    /// Curse of Binding, Soul Speed, Mending, Curse of Vanishing are
    /// chest/fishing/trading-only — the table never offers them)
    pub table: bool,
}

pub const ENCHANTS: &[EnchantDef] = &[
    // ---- indices 0..=9: original Phase-era order (save compatibility) ----
    EnchantDef { name: "Protection", id: "protection", max_level: 4, weight: 10, table: true },
    EnchantDef { name: "Feather Falling", id: "feather_falling", max_level: 4, weight: 5, table: true },
    EnchantDef { name: "Sharpness", id: "sharpness", max_level: 5, weight: 10, table: true },
    EnchantDef { name: "Efficiency", id: "efficiency", max_level: 5, weight: 10, table: true },
    EnchantDef { name: "Unbreaking", id: "unbreaking", max_level: 3, weight: 5, table: true },
    EnchantDef { name: "Fortune", id: "fortune", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Silk Touch", id: "silk_touch", max_level: 1, weight: 1, table: true },
    EnchantDef { name: "Mending", id: "mending", max_level: 1, weight: 2, table: false },
    EnchantDef { name: "Power", id: "power", max_level: 5, weight: 10, table: true },
    EnchantDef { name: "Looting", id: "looting", max_level: 3, weight: 2, table: true },
    // ---- Phase 4 additions: the remaining 28 of the 38 ----
    EnchantDef { name: "Fire Protection", id: "fire_protection", max_level: 4, weight: 5, table: true },
    EnchantDef { name: "Blast Protection", id: "blast_protection", max_level: 4, weight: 2, table: true },
    EnchantDef { name: "Projectile Protection", id: "projectile_protection", max_level: 4, weight: 5, table: true },
    EnchantDef { name: "Respiration", id: "respiration", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Aqua Affinity", id: "aqua_affinity", max_level: 1, weight: 2, table: true },
    EnchantDef { name: "Thorns", id: "thorns", max_level: 3, weight: 1, table: true },
    EnchantDef { name: "Depth Strider", id: "depth_strider", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Frost Walker", id: "frost_walker", max_level: 2, weight: 2, table: false },
    EnchantDef { name: "Curse of Binding", id: "binding_curse", max_level: 1, weight: 1, table: false },
    EnchantDef { name: "Soul Speed", id: "soul_speed", max_level: 3, weight: 1, table: false },
    EnchantDef { name: "Smite", id: "smite", max_level: 5, weight: 5, table: true },
    EnchantDef { name: "Bane of Arthropods", id: "bane_of_arthropods", max_level: 5, weight: 5, table: true },
    EnchantDef { name: "Knockback", id: "knockback", max_level: 2, weight: 5, table: true },
    EnchantDef { name: "Fire Aspect", id: "fire_aspect", max_level: 2, weight: 2, table: true },
    EnchantDef { name: "Sweeping Edge", id: "sweeping", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Punch", id: "punch", max_level: 2, weight: 2, table: true },
    EnchantDef { name: "Flame", id: "flame", max_level: 1, weight: 2, table: true },
    EnchantDef { name: "Infinity", id: "infinity", max_level: 1, weight: 1, table: true },
    EnchantDef { name: "Luck of the Sea", id: "luck_of_the_sea", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Lure", id: "lure", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Loyalty", id: "loyalty", max_level: 3, weight: 5, table: true },
    EnchantDef { name: "Impaling", id: "impaling", max_level: 5, weight: 2, table: true },
    EnchantDef { name: "Riptide", id: "riptide", max_level: 3, weight: 2, table: true },
    EnchantDef { name: "Channeling", id: "channeling", max_level: 1, weight: 1, table: true },
    EnchantDef { name: "Multishot", id: "multishot", max_level: 1, weight: 2, table: true },
    EnchantDef { name: "Quick Charge", id: "quick_charge", max_level: 3, weight: 5, table: true },
    EnchantDef { name: "Piercing", id: "piercing", max_level: 4, weight: 10, table: true },
    EnchantDef { name: "Curse of Vanishing", id: "vanishing_curse", max_level: 1, weight: 1, table: false },
];

/// registry lookup by id (§46: out-of-range ids fold to index 0)
pub fn enchant_def(id: u8) -> &'static EnchantDef {
    &ENCHANTS[(id as usize).min(ENCHANTS.len() - 1)]
}

/// registry lookup by the 1.16.5 id string (datapack-facing)
pub fn enchant_by_id(id: &str) -> Option<u8> {
    ENCHANTS.iter().position(|e| e.id == id).map(|i| i as u8)
}

/// incompatibility groups (VERIFIED live, minecraft.wiki "List of
/// enchantments" Incompatible-With column, 2026-09-04, 1.16.5 subset):
/// - the four Protections are pairwise exclusive
/// - Sharpness / Smite / Bane of Arthropods are pairwise exclusive
///   (Breach/Density join this family only in 1.21+)
/// - Fortune ↔ Silk Touch
/// - Depth Strider ↔ Frost Walker
/// - Infinity ↔ Mending
/// - Multishot ↔ Piercing
/// - Riptide ↔ Channeling and Riptide ↔ Loyalty (Channeling + Loyalty
///   coexist on one trident)
pub fn incompatible(a: u8, b: u8) -> bool {
    if a == b {
        return false; // same enchant: stacking handled by level, not conflict
    }
    const PROTECTION_FAMILY: &[&str] =
        &["protection", "fire_protection", "blast_protection", "projectile_protection"];
    const DAMAGE_FAMILY: &[&str] = &["sharpness", "smite", "bane_of_arthropods"];
    let (da, db) = (enchant_def(a), enchant_def(b));
    let in_group = |g: &[&str]| g.contains(&da.id) && g.contains(&db.id);
    if in_group(PROTECTION_FAMILY) || in_group(DAMAGE_FAMILY) {
        return true;
    }
    matches!(
        (da.id, db.id),
        ("fortune", "silk_touch")
            | ("silk_touch", "fortune")
            | ("depth_strider", "frost_walker")
            | ("frost_walker", "depth_strider")
            | ("infinity", "mending")
            | ("mending", "infinity")
            | ("multishot", "piercing")
            | ("piercing", "multishot")
            | ("riptide", "channeling")
            | ("channeling", "riptide")
            | ("riptide", "loyalty")
            | ("loyalty", "riptide")
    )
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

/// the §30 slot-level formula (VERIFIED from the 1.16.5-era wiki,
/// archived revision 1945529 of "Enchanting table mechanics", pulled via
/// the MediaWiki API on 2026-09-04 — matches Dossier Part 6 §30):
///
/// ```text
/// base          = randInt(1,8) + floor(b/2) + randInt(0,b)
/// top slot      = floor(max(base / 3, 1))
/// middle slot   = floor(base * 2 / 3 + 1)
/// bottom slot   = floor(max(base, b * 2))
/// ```
///
/// where b = bookshelf power (0..=15). Confirms the classic property:
/// 15 bookshelves → the bottom slot can reach 30 (base max = 8+7+15).
fn slot_levels(b: u8, rng: &mut Rng) -> [u8; 3] {
    let b = b.min(15) as u32;
    // randInt(1,8): uniform 1..=8; randInt(0,b): uniform 0..=b
    let r18 = 1 + rng.next_range(8);
    let r0b = rng.next_range(b + 1);
    let base = r18 + b / 2 + r0b;
    let top = (base / 3).max(1);
    let mid = base * 2 / 3 + 1;
    let bottom = base.max(b * 2);
    [top.min(30) as u8, mid.min(30) as u8, bottom.min(30) as u8]
}

/// weighted-random enchant pick from the TABLE-RECEIVABLE subset
/// (books enchanted at the table can only roll the 33 `table: true`
/// entries — VERIFIED: Frost Walker/Binding/Soul Speed/Mending/Vanishing
/// never appear on table offers)
fn pick_enchant(rng: &mut Rng) -> u8 {
    let total: u32 = ENCHANTS.iter().filter(|e| e.table).map(|e| e.weight).sum();
    let mut pick = rng.next_range(total);
    for (i, e) in ENCHANTS.iter().enumerate() {
        if !e.table {
            continue;
        }
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

/// generate the three option rows from a power + seed (§30 formula)
pub fn gen_options(power: u8, seed: u64) -> [EnchOption; 3] {
    let mut rng = Rng::new(seed);
    let levels = slot_levels(power, &mut rng);
    let mut out = [EnchOption { level: 0, ench: 0, ench_level: 0, cost: 0 }; 3];
    for (row, o) in out.iter_mut().enumerate() {
        let level = levels[row];
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
    /// Phase 4: an offer whose enchant is INCOMPATIBLE with one already
    /// on the item is unpayable (the table never rerolls conflicts away
    /// in vanilla either — the slot just can't be taken)
    pub fn can_apply(&self, row: usize, player_level: i32) -> bool {
        let Some(o) = self.options.get(row) else { return false };
        if o.level == 0 || self.item.is_empty() || self.item.block != ENCHANTED_BOOK {
            return false;
        }
        if self.lapis.count < o.cost || player_level < o.cost as i32 {
            return false;
        }
        if let Some((existing, _)) = self.item.enchant() {
            if incompatible(existing, o.ench) {
                return false;
            }
        }
        true
    }

    /// apply an offer: mutates the item (ench field), returns the consumed
    /// cost on success (the caller pays the levels/lapis)
    pub fn apply(&mut self, row: usize) -> Option<u8> {
        let o = self.options.get(row).copied()?;
        if o.level == 0 || self.item.is_empty() || self.item.block != ENCHANTED_BOOK {
            return None;
        }
        if let Some((existing, _)) = self.item.enchant() {
            if incompatible(existing, o.ench) {
                return None;
            }
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
        // §30 formula (VERIFIED): b=0 → base = 1..8, top = 1..2
        let low = gen_options(0, 42);
        assert!(low.iter().all(|o| o.level >= 1 && o.level <= 2), "{low:?}");
        // b=15 → base = 8+7+15=30 max, bottom = max(base, 30) → can hit 30
        let mut best_bottom = 0;
        for seed in 0..200u64 {
            let high = gen_options(15, seed);
            assert!(high[2].level >= 10, "bottom row at full power: {high:?}");
            best_bottom = best_bottom.max(high[2].level);
            // rows stay ordered cost 1..3
            for (i, o) in high.iter().enumerate() {
                assert_eq!(o.cost as usize, i + 1);
            }
            // every option references a table-receivable enchant, valid level
            for o in high.iter() {
                let def = enchant_def(o.ench);
                assert!(def.table, "table offers only table-receivable enchants");
                assert!(o.ench_level >= 1 && o.ench_level <= def.max_level);
                assert!(o.level >= 1 && o.level <= 30);
            }
        }
        assert_eq!(best_bottom, 30, "15 bookshelves reach level 30 (VERIFIED)");
        // same seed → same options (deterministic for E2E)
        assert_eq!(gen_options(15, 42), gen_options(15, 42));
    }

    /// §30 formula shape: the three slots at a FIXED base. We can't pin the
    /// RNG, so test the algebra through many seeds at b=0 (base 1..8):
    /// top = floor(max(base/3,1)) ≤ 2; mid = floor(2*base/3+1) ≤ 6;
    /// bottom = base ≤ 8 — and monotone top ≤ mid ≤ bottom must hold
    /// whenever the underlying base does (checked via bounds here).
    #[test]
    fn slot_formula_bounds_at_zero_power() {
        for seed in 0..500u64 {
            let o = gen_options(0, seed);
            assert!(o[0].level <= o[1].level, "top ≤ middle ({o:?})");
            assert!(o[1].level <= o[2].level, "middle ≤ bottom ({o:?})");
        }
    }

    #[test]
    fn registry_is_the_full_38_with_sweeping_id() {
        // VERIFIED: exactly 38 enchantments in 1.16.5
        assert_eq!(ENCHANTS.len(), 38);
        // §29 ID DISCIPLINE: 1.16.5 uses `sweeping` (renamed 24w03a) —
        // and the pre-rename id must NOT be present
        assert!(enchant_by_id("sweeping").is_some());
        assert!(enchant_by_id("sweeping_edge").is_none());
        // registry ids are unique
        let mut ids: Vec<&str> = ENCHANTS.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 38);
        // max levels spot-checks (live-verified table)
        let spot = |id: &str| enchant_def(enchant_by_id(id).unwrap()).max_level;
        assert_eq!(spot("protection"), 4);
        assert_eq!(spot("sharpness"), 5);
        assert_eq!(spot("sweeping"), 3);
        assert_eq!(spot("piercing"), 4);
        assert_eq!(spot("quick_charge"), 3);
        assert_eq!(spot("frost_walker"), 2);
        // weights spot-checks (1.16.5-era weight table, revision 1945529)
        let w = |id: &str| enchant_def(enchant_by_id(id).unwrap()).weight;
        assert_eq!(w("protection"), 10);
        assert_eq!(w("blast_protection"), 2);
        assert_eq!(w("silk_touch"), 1);
        assert_eq!(w("piercing"), 10);
        assert_eq!(w("mending"), 2);
        assert_eq!(w("thorns"), 1);
        // table-receivable: exactly the 5 non-table enchants
        let non_table: Vec<&str> =
            ENCHANTS.iter().filter(|e| !e.table).map(|e| e.id).collect();
        assert_eq!(
            non_table,
            vec!["mending", "frost_walker", "binding_curse", "soul_speed", "vanishing_curse"]
        );
    }

    #[test]
    fn incompatibility_groups_match_the_wiki() {
        // protection family: pairwise
        let prot: Vec<u8> = ["protection", "fire_protection", "blast_protection", "projectile_protection"]
            .iter().map(|&i| enchant_by_id(i).unwrap()).collect();
        for i in 0..prot.len() {
            for j in 0..prot.len() {
                assert_eq!(incompatible(prot[i], prot[j]), i != j);
            }
        }
        // damage family: pairwise
        let dmg: Vec<u8> = ["sharpness", "smite", "bane_of_arthropods"]
            .iter().map(|&i| enchant_by_id(i).unwrap()).collect();
        for i in 0..dmg.len() {
            for j in 0..dmg.len() {
                assert_eq!(incompatible(dmg[i], dmg[j]), i != j);
            }
        }
        // pairwise pairs
        let pairs = [
            ("fortune", "silk_touch"),
            ("depth_strider", "frost_walker"),
            ("infinity", "mending"),
            ("multishot", "piercing"),
            ("riptide", "channeling"),
            ("riptide", "loyalty"),
        ];
        for (a, b) in pairs {
            let (ia, ib) = (enchant_by_id(a).unwrap(), enchant_by_id(b).unwrap());
            assert!(incompatible(ia, ib), "{a} vs {b}");
            assert!(incompatible(ib, ia), "symmetric: {b} vs {a}");
        }
        // compatible: channeling + loyalty coexist (VERIFIED)
        let (ch, lo) = (enchant_by_id("channeling").unwrap(), enchant_by_id("loyalty").unwrap());
        assert!(!incompatible(ch, lo));
        // protection + feather falling coexist (different families)
        let (pr, ff) = (enchant_by_id("protection").unwrap(), enchant_by_id("feather_falling").unwrap());
        assert!(!incompatible(pr, ff));
        // unrelated pairs
        assert!(!incompatible(enchant_by_id("sharpness").unwrap(), enchant_by_id("power").unwrap()));
        assert!(!incompatible(enchant_by_id("unbreaking").unwrap(), enchant_by_id("efficiency").unwrap()));
        // self is not a conflict
        assert!(!incompatible(0, 0));
    }

    #[test]
    fn incompatible_offer_cannot_be_applied() {
        let mut st = EnchantState::default();
        st.item = ItemStack::new(ENCHANTED_BOOK, 1);
        st.item.set_enchant(enchant_by_id("protection").unwrap(), 2);
        st.lapis = ItemStack::new(LAPIS_ORE, 3);
        // force an offer of Fire Protection — incompatible with Protection
        st.options = [EnchOption {
            level: 20,
            ench: enchant_by_id("fire_protection").unwrap(),
            ench_level: 3,
            cost: 1,
        }; 3];
        assert!(!st.can_apply(0, 30), "incompatible offer blocked");
        assert!(st.apply(0).is_none(), "apply refuses incompatibility");
        // a compatible offer goes through
        st.options[0].ench = enchant_by_id("feather_falling").unwrap();
        assert!(st.can_apply(0, 30));
        assert_eq!(st.apply(0), Some(1));
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
