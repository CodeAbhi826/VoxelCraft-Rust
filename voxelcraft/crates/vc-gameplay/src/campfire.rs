//! Campfire (1.14 Village & Pillage — nature half): the 4-slot,
//! fuel-less 600-tick cooker. All values VERIFIED live 2026-09-08 from
//! the raw capture scripts/v114_page_campfire.json (minecraft.wiki
//! /w/Campfire):
//!
//! - "Food items take 30 seconds (600 ticks) to cook, compared to 10
//!   seconds for furnaces or 5 seconds for smokers" — "Assuming that
//!   one uses all four slots to cook at once"
//! - "campfires do not require any kind of fuel"
//! - the lit state carries the light (15 when lit); cooking only runs
//!   while LIT — the game layer swaps the world state on
//!   extinguish/relight and ticks the map only for lit campfires
//!   (honest simplification: vanilla also pauses cooking when
//!   extinguished, dropping the food instead on break)
//! - breaking: "Campfires now drop the food being cooked" (20w22a) +
//!   "When mined regularly, a campfire drops 2 charcoal" — both ride
//!   the game layer's break path, which drains `slots`
//!
//! Documented adaptation: vanilla extracts cooked food via hopper or
//! breaking; the engine has no campfire UI, so completed items are
//! queued in `done` for the game layer to spawn as item entities on
//! top of the campfire (disclosed in the worklog).

use std::collections::HashMap;
use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;
use vc_world::world::World;

/// vanilla: 600 game ticks per food item (VERIFIED — 30 s).
pub const COOK_TICKS: i32 = 600;
/// vanilla: 4 food slots (VERIFIED).
pub const SLOTS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct CampfireState {
    /// the 4 food slots (raw food in, cooked food out)
    pub slots: [ItemStack; SLOTS],
    /// cook countdown per slot (ticks remaining; 0 = empty/free)
    pub cook: [i32; SLOTS],
}

impl Default for CampfireState {
    fn default() -> Self {
        CampfireState {
            slots: [ItemStack::EMPTY; SLOTS],
            cook: [0; SLOTS],
        }
    }
}

impl CampfireState {
    /// can this item cook here? — vanilla's campfire set is the
    /// smeltable FOODS (raw meats + potato + kelp). The engine's
    /// standing cooked-meat deferral (beef/porkchop/chicken/mutton/
    /// cod/salmon have no cooked item forms yet — the furnace round
    /// never added them) narrows the list to the three raw→cooked
    /// pairs the furnace actually ships: potato, raw rabbit, kelp.
    /// The set extends automatically when the meat forms land.
    pub fn accepts(block: u16) -> bool {
        matches!(
            block,
            POTATO | RAW_RABBIT | KELP
                // the completeness audit: the meat forms have landed —
                // the set extends exactly as this deferral promised
                // ("The set extends automatically when the meat forms
                // land"); vanilla's campfire set is the smeltable
                // FOODS (raw meats + potato + kelp)
                | BEEF
                | PORKCHOP
                | CHICKEN_RAW
                | MUTTON
                | RAW_FISH
                | RAW_SALMON
        )
    }

    /// add a food item to the first free slot; false when full or the
    /// item can't cook.
    pub fn add(&mut self, block: u16) -> bool {
        if !Self::accepts(block) {
            return false;
        }
        for i in 0..SLOTS {
            if self.slots[i].is_empty() {
                self.slots[i] = ItemStack::new(block, 1);
                self.cook[i] = COOK_TICKS;
                return true;
            }
        }
        false
    }
}

pub struct Campfires {
    /// keyed by the campfire block position
    pub map: HashMap<[i32; 3], CampfireState>,
    /// completed cooking: (pos, cooked-item block id) — drained by the
    /// game layer (spawns the item entity above the campfire)
    pub done: Vec<([i32; 3], u16)>,
}

impl Default for Campfires {
    fn default() -> Self {
        Campfires {
            map: HashMap::new(),
            done: Vec::new(),
        }
    }
}

impl Campfires {
    /// entry for a position, creating it on first use
    pub fn entry(&mut self, pos: [i32; 3]) -> &mut CampfireState {
        self.map.entry(pos).or_default()
    }

    /// ONE sim tick: cook countdowns run ONLY for LIT campfires
    /// (vanilla pauses cooking on an extinguished campfire — the
    /// fire is the heat source). Completed items move to `done`.
    pub fn tick(&mut self, world: &World) {
        let positions: Vec<[i32; 3]> = self.map.keys().copied().collect();
        for pos in positions {
            let lit = campfire_lit(world.get_state(pos[0], pos[1], pos[2]));
            if !lit {
                continue;
            }
            let cf = self.map.get_mut(&pos).unwrap();
            for i in 0..SLOTS {
                if cf.slots[i].is_empty() || cf.cook[i] <= 0 {
                    continue;
                }
                cf.cook[i] -= 1;
                if cf.cook[i] == 0 {
                    let raw = cf.slots[i].block;
                    cf.slots[i] = ItemStack::EMPTY;
                    if let Some(cooked) = crate::furnace::smelt_result(raw) {
                        self.done.push((pos, cooked));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campfire_cook_timing_is_600_ticks() {
        assert_eq!(COOK_TICKS, 600, "30 s (VERIFIED w/Campfire)");
        assert_eq!(SLOTS, 4, "4 slots (VERIFIED)");
    }

    #[test]
    fn four_slots_fill_then_reject() {
        let mut cf = CampfireState::default();
        for _ in 0..4 {
            assert!(cf.add(POTATO), "potato cooks (smelt result exists)");
        }
        assert!(!cf.add(POTATO), "the 5th item finds no free slot");
        // non-food never accepted
        let mut cf2 = CampfireState::default();
        assert!(!cf2.add(STONE), "stone has no smelt result");
        assert!(!cf2.add(COBBLE), "cobble is smeltable but not FOOD");
    }

    fn lit_world_with_campfire() -> World {
        // a loaded chunk under the campfire (World edits no-op on
        // missing chunks — the spawners-test pattern)
        let mut w = World::new(7);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        // the LIT campfire state on the floor
        let _ = w.set_block_state(8, 65, 8, campfire_state(true));
        assert_eq!(w.get_state(8, 65, 8), campfire_state(true));
        w
    }

    #[test]
    fn cooking_completes_at_exactly_600_ticks() {
        let w = lit_world_with_campfire();
        let mut cfs = Campfires::default();
        cfs.entry([8, 65, 8]).add(POTATO);
        // 599 ticks: not done
        for _ in 0..599 {
            cfs.tick(&w);
        }
        assert!(cfs.done.is_empty(), "599 ticks: still cooking");
        cfs.tick(&w);
        assert_eq!(cfs.done.len(), 1, "the 600th tick completes");
        assert_eq!(cfs.done[0], ([8, 65, 8], BAKED_POTATO));
        // the slot is free again
        assert!(cfs.map.get(&[8, 65, 8]).unwrap().slots[0].is_empty());
    }

    #[test]
    fn extinguished_campfire_does_not_cook() {
        let mut w = lit_world_with_campfire();
        // extinguish it (the shovel/water path's world state)
        let _ = w.set_block_state(8, 65, 8, campfire_state(false));
        let mut cfs = Campfires::default();
        cfs.entry([8, 65, 8]).add(POTATO);
        for _ in 0..2000 {
            cfs.tick(&w);
        }
        assert!(cfs.done.is_empty(), "no heat, no cooking (vanilla)");
        // the food stays in its slot (dropped only when broken)
        assert!(!cfs.map.get(&[8, 65, 8]).unwrap().slots[0].is_empty());
    }

    #[test]
    fn all_four_slots_cook_in_parallel() {
        let w = lit_world_with_campfire();
        let mut cfs = Campfires::default();
        let cf = cfs.entry([8, 65, 8]);
        cf.add(POTATO);
        cf.add(RAW_RABBIT);
        cf.add(KELP);
        assert!(cf.add(POTATO), "4th slot free");
        for _ in 0..600 {
            cfs.tick(&w);
        }
        assert_eq!(cfs.done.len(), 4, "all four finished together");
        let outs: Vec<u16> = cfs.done.iter().map(|(_, b)| *b).collect();
        assert!(outs.contains(&BAKED_POTATO));
        assert!(outs.contains(&COOKED_RABBIT));
        assert!(outs.contains(&DRIED_KELP));
    }

    /// the completeness audit: the meat forms landed — the campfire's
    /// set extends exactly as the deferral promised ("The set extends
    /// automatically when the meat forms land")
    #[test]
    fn audit16_campfire_cooks_the_meats() {
        for b in [BEEF, PORKCHOP, CHICKEN_RAW, MUTTON, RAW_FISH, RAW_SALMON] {
            assert!(CampfireState::accepts(b), "campfire accepts {b}");
        }
        assert!(!CampfireState::accepts(STEAK), "cooked food does not recook");
        assert!(!CampfireState::accepts(COBBLE));
    }
}
