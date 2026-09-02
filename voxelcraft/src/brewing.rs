//! Brewing (Phase 7 §29): brewing-stand block entities with
//! vanilla-observable mechanics — 400-tick brew cycle, one fuel item funds
//! 20 operations (blaze-powder analogue), the ALL-bottles-must-brew rule,
//! progress reset when the ingredient is pulled, and the vanilla bottle
//! chain: glass bottle → water bottle → awkward → effect potion →
//! glowstone-upgraded II.
//!
//! Engine adaptations (documented, palette-bounded):
//! - blaze powder → NETHERRACK as the fuel item (nether-native, plentiful)
//! - nether wart → MUSHROOM_RED as the base ingredient (red like wart)
//! - glistering melon → MUSHROOM_BROWN as the effect ingredient
//! - glowstone upgrade is exactly vanilla

use crate::blocks::*;
use crate::inventory::ItemStack;
use std::collections::HashMap;

/// vanilla: 400 game ticks per brew (20 s)
pub const BREW_TICKS: i32 = 400;
/// vanilla: one blaze powder fuels 20 operations (§29 adaptation: netherrack)
pub const FUEL_OPERATIONS: i32 = 20;

/// is this block a valid brewing fuel?
pub fn is_fuel(b: u8) -> bool {
    b == NETHERRACK
}

/// one brewing recipe: `ingredient` + `input` bottle → `output` bottle
pub struct BrewRecipe {
    pub input: u8,
    pub ingredient: u8,
    pub output: u8,
}

/// the recipe registry (data-driven-shaped: a const table the engine and
/// the tests both read — §29 "use data-driven recipes and registries")
pub const BREW_RECIPES: &[BrewRecipe] = &[
    // base: water bottle + wart-analogue → awkward (vanilla chain head)
    BrewRecipe {
        input: POTION_WATER,
        ingredient: MUSHROOM_RED,
        output: POTION_AWKWARD,
    },
    // water bottle + mundane ingredient → mundane (vanilla no-effect)
    BrewRecipe {
        input: POTION_WATER,
        ingredient: MUSHROOM_BROWN,
        output: POTION_MUNDANE,
    },
    // effect: awkward + effect ingredient → healing
    BrewRecipe {
        input: POTION_AWKWARD,
        ingredient: MUSHROOM_BROWN,
        output: POTION_HEALING,
    },
    // modifier: glowstone upgrades to level II (exactly vanilla)
    BrewRecipe {
        input: POTION_HEALING,
        ingredient: GLOWSTONE,
        output: POTION_HEALING_II,
    },
];

/// look up the brew result for an (input, ingredient) pair
pub fn brew_result(input: u8, ingredient: u8) -> Option<u8> {
    BREW_RECIPES
        .iter()
        .find(|r| r.input == input && r.ingredient == ingredient)
        .map(|r| r.output)
}

/// vanilla instant-health healing amounts (half-hearts ×2 = HP)
pub fn potion_heal(b: u8) -> Option<f32> {
    match b {
        POTION_HEALING => Some(4.0),
        POTION_HEALING_II => Some(8.0),
        _ => None, // water/awkward/mundane have no effect (vanilla)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrewingState {
    /// the top slot: consumed at the END of a brew cycle
    pub ingredient: ItemStack,
    /// the fuel slot: consumed only when the charge pool is empty
    pub fuel: ItemStack,
    /// charge pool: how many operations the current fuel item still funds
    pub fuel_charges: i32,
    /// the three bottle slots (bottom row)
    pub bottles: [ItemStack; 3],
    /// ticks of progress on the current cycle (0..=BREW_TICKS)
    pub brew_left: i32,
}

impl Default for BrewingState {
    fn default() -> Self {
        BrewingState {
            ingredient: ItemStack::EMPTY,
            fuel: ItemStack::EMPTY,
            fuel_charges: 0,
            bottles: [ItemStack::EMPTY; 3],
            brew_left: 0,
        }
    }
}

impl BrewingState {
    /// vanilla rule: brewing only proceeds when EVERY non-empty bottle slot
    /// has a recipe for the current ingredient (empty slots are fine)
    fn can_brew(&self) -> bool {
        let filled: Vec<&ItemStack> =
            self.bottles.iter().filter(|s| !s.is_empty()).collect();
        if filled.is_empty() || self.ingredient.is_empty() {
            return false;
        }
        let ing = self.ingredient.block;
        filled
            .iter()
            .all(|s| brew_result(s.block, ing).is_some())
    }

    pub fn is_brewing(&self) -> bool {
        self.brew_left > 0
    }

    /// progress fraction for the UI bubble column
    pub fn progress(&self) -> f32 {
        if self.brew_left <= 0 {
            0.0
        } else {
            self.brew_left as f32 / BREW_TICKS as f32
        }
    }

    /// ONE sim tick. Returns true when a brew COMPLETED this tick (the
    /// caller plays the bubble event + reports stats).
    pub fn tick(&mut self) -> bool {
        // fuel refill: only when there is work and the pool is dry (the
        // vanilla "blaze powder consumed only when needed" behavior)
        if self.fuel_charges <= 0 && self.can_brew() && !self.fuel.is_empty() {
            if is_fuel(self.fuel.block) {
                self.fuel_charges = FUEL_OPERATIONS;
                self.fuel.count -= 1;
                if self.fuel.count == 0 {
                    self.fuel = ItemStack::EMPTY;
                }
            }
        }

        // vanilla quirk: pulling the ingredient mid-cycle resets progress
        // (same reset rule as the furnace input)
        if self.brew_left > 0 && !self.can_brew() {
            self.brew_left = 0;
            return false;
        }

        if self.fuel_charges > 0 && self.can_brew() {
            self.brew_left += 1;
            if self.brew_left >= BREW_TICKS {
                // cycle done: consume the ingredient, transform every
                // filled bottle, spend one charge
                let ing = self.ingredient.block;
                self.ingredient.count -= 1;
                if self.ingredient.count == 0 {
                    self.ingredient = ItemStack::EMPTY;
                }
                for s in self.bottles.iter_mut() {
                    if !s.is_empty() {
                        if let Some(out) = brew_result(s.block, ing) {
                            *s = ItemStack::new(out, 1);
                        }
                    }
                }
                self.fuel_charges -= 1;
                self.brew_left = 0;
                return true;
            }
        }
        false
    }
}

/// all brewing-stand block entities, ticked by the sim at 20 Hz
#[derive(Default)]
pub struct Brewings {
    pub map: HashMap<[i32; 3], BrewingState>,
    /// positions where a brew completed on the LAST tick (game.rs drains
    /// this each frame to play the bubble event + bump stats)
    pub completed: Vec<[i32; 3]>,
    /// total brews completed since boot (stats/F3/E2E)
    pub total_brewed: u64,
}

impl Brewings {
    /// sim tick step; records completed positions in `completed`
    pub fn tick(&mut self) {
        self.completed.clear();
        let positions: Vec<[i32; 3]> = self.map.keys().copied().collect();
        for pos in positions {
            let Some(b) = self.map.get_mut(&pos) else { continue };
            if b.tick() {
                self.completed.push(pos);
                self.total_brewed += 1;
            }
        }
    }

    /// drop everything inside a stand (block-broken path) as item entities
    /// is handled by game.rs (it owns the item system); here we only
    /// expose the removal.
    pub fn remove(&mut self, pos: &[i32; 3]) -> Option<BrewingState> {
        self.map.remove(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_plus_wart_makes_awkward() {
        let mut b = BrewingState::default();
        b.bottles = [ItemStack::new(POTION_WATER, 1), ItemStack::EMPTY, ItemStack::new(POTION_WATER, 1)];
        b.ingredient = ItemStack::new(MUSHROOM_RED, 1);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        let mut completions = 0;
        for _ in 0..BREW_TICKS {
            if b.tick() {
                completions += 1;
            }
        }
        assert_eq!(completions, 1, "exactly one brew cycle");
        assert_eq!(b.bottles[0].block, POTION_AWKWARD);
        assert_eq!(b.bottles[2].block, POTION_AWKWARD);
        assert!(b.bottles[1].is_empty(), "empty slot untouched");
        assert!(b.ingredient.is_empty(), "ingredient consumed");
        assert_eq!(b.fuel_charges, FUEL_OPERATIONS - 1, "one charge spent");
        assert!(b.fuel.is_empty(), "the single fuel item was consumed into 20 charges");
    }

    #[test]
    fn no_fuel_no_brewing() {
        let mut b = BrewingState::default();
        b.bottles[0] = ItemStack::new(POTION_WATER, 1);
        b.ingredient = ItemStack::new(MUSHROOM_RED, 1);
        for _ in 0..1000 {
            assert!(!b.tick());
        }
        assert_eq!(b.bottles[0].block, POTION_WATER);
        assert_eq!(b.brew_left, 0);
    }

    #[test]
    fn all_filled_bottles_must_have_a_recipe() {
        // vanilla: a water bottle + a healing bottle with a wart ingredient
        // → brewing does NOT start (healing has no wart recipe)
        let mut b = BrewingState::default();
        b.bottles[0] = ItemStack::new(POTION_WATER, 1);
        b.bottles[1] = ItemStack::new(POTION_HEALING, 1);
        b.ingredient = ItemStack::new(MUSHROOM_RED, 1);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        for _ in 0..600 {
            assert!(!b.tick());
        }
        assert_eq!(b.bottles[0].block, POTION_WATER, "nothing brewed");
    }

    #[test]
    fn glowstone_upgrades_to_level_ii() {
        let mut b = BrewingState::default();
        b.bottles[0] = ItemStack::new(POTION_HEALING, 1);
        b.ingredient = ItemStack::new(GLOWSTONE, 1);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        for _ in 0..BREW_TICKS + 1 {
            b.tick();
        }
        assert_eq!(b.bottles[0].block, POTION_HEALING_II);
    }

    #[test]
    fn awkward_plus_brown_mushroom_heals() {
        let mut b = BrewingState::default();
        b.bottles[0] = ItemStack::new(POTION_AWKWARD, 1);
        b.ingredient = ItemStack::new(MUSHROOM_BROWN, 1);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        for _ in 0..BREW_TICKS + 1 {
            b.tick();
        }
        assert_eq!(b.bottles[0].block, POTION_HEALING);
    }

    #[test]
    fn one_fuel_item_funds_twenty_brews() {
        // vanilla: each stand slot holds ONE bottle (potions don't stack),
        // so we re-fill the slot between cycles — the fuel charge pool must
        // fund exactly 20 operations from one netherrack item
        let mut b = BrewingState::default();
        b.ingredient = ItemStack::new(MUSHROOM_RED, 64);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        let mut completions = 0;
        let mut ticks = 0i64;
        while ticks < (BREW_TICKS as i64) * 25 {
            if b.bottles[0].is_empty() {
                b.bottles[0] = ItemStack::new(POTION_WATER, 1);
            }
            if b.tick() {
                completions += 1;
                // the bottle came out — take it, the loop refills
                b.bottles[0] = ItemStack::EMPTY;
            }
            ticks += 1;
        }
        assert_eq!(completions, 20, "one netherrack = exactly 20 operations");
        assert!(b.fuel.is_empty());
        assert_eq!(b.fuel_charges, 0);
    }

    #[test]
    fn pulling_the_ingredient_resets_progress() {
        let mut b = BrewingState::default();
        b.bottles[0] = ItemStack::new(POTION_WATER, 1);
        b.ingredient = ItemStack::new(MUSHROOM_RED, 1);
        b.fuel = ItemStack::new(NETHERRACK, 1);
        for _ in 0..200 {
            b.tick();
        }
        assert!(b.brew_left > 0, "mid-cycle");
        b.ingredient = ItemStack::EMPTY;
        b.tick();
        assert_eq!(b.brew_left, 0, "progress reset (vanilla)");
        // bottles untouched
        assert_eq!(b.bottles[0].block, POTION_WATER);
    }

    #[test]
    fn brews_tick_through_the_map_and_report() {
        let mut bs = Brewings::default();
        let mut st = BrewingState::default();
        st.bottles[0] = ItemStack::new(POTION_WATER, 1);
        st.ingredient = ItemStack::new(MUSHROOM_RED, 1);
        st.fuel = ItemStack::new(NETHERRACK, 1);
        bs.map.insert([1, 65, 1], st);
        for _ in 0..BREW_TICKS {
            bs.tick();
        }
        assert_eq!(bs.completed, vec![[1, 65, 1]]);
        assert_eq!(bs.total_brewed, 1);
        assert_eq!(bs.map[&[1, 65, 1]].bottles[0].block, POTION_AWKWARD);
        // removal (block broken)
        assert!(bs.remove(&[1, 65, 1]).is_some());
        assert!(bs.map.is_empty());
    }

    #[test]
    fn potion_heal_amounts() {
        assert_eq!(potion_heal(POTION_HEALING), Some(4.0));
        assert_eq!(potion_heal(POTION_HEALING_II), Some(8.0));
        assert_eq!(potion_heal(POTION_WATER), None);
        assert_eq!(potion_heal(POTION_AWKWARD), None);
        assert_eq!(potion_heal(POTION_MUNDANE), None);
    }
}
