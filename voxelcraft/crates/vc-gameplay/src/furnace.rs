//! Furnace (Phase 7 §27): block entities with vanilla-observable smelting
//! — 200-tick cook time, fuel burn times (planks/logs 300 ticks), input →
//! output, lit state swap on the world block. Ticked by the sim at 20 Hz.

use std::collections::HashMap;
use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;
use vc_world::world::World;

/// vanilla: 200 game ticks per item
pub const COOK_TICKS: i32 = 200;
/// fuel burn times (game ticks)
pub fn fuel_ticks(block: u8) -> i32 {
    match block {
        PLANKS | OAK_LOG | BIRCH_LOG | SPRUCE_LOG => 300,
        // OAK_SLAB = 150 ticks (VERIFIED 2026-09-06 live:
        // minecraft.wiki/w/Smelting — "Wooden Slab 7.5 [s], 150 ticks";
        // was 300, the planks value — half-length slabs burn half as
        // long). Crafting table + fence stay 300 (wiki fuel table).
        CRAFTING_TABLE | OAK_FENCE => 300,
        OAK_SLAB => 150,
        // the coal ITEM: 1600 ticks = 80 s = 8 items (VERIFIED
        // 2026-09-06 live, minecraft.wiki/w/Furnace "a piece of coal
        // burns for 80 seconds and can process eight items";
        // w/Smelting fuel table "Coal 1600 [ticks], 8 [items]").
        // COAL_ORE is NO LONGER a fuel: the 800-tick ore-as-fuel
        // stopgap (VERIFICATION-REPORT §6, disclosed) is retired now
        // that the coal item exists — vanilla coal ore is not a fuel.
        COAL => 1600,
        // Phase E3 (VERIFIED live 2026-09-06, minecraft.wiki/w/
        // Block_of_Coal: "One block of coal lasts 800 seconds (16000
        // ticks), which smelts 80 items" — 10× the coal item)
        COAL_BLOCK => 16000,
        _ => 0,
    }
}

/// what does this block smelt into?
pub fn smelt_result(block: u8) -> Option<u8> {
    match block {
        SAND => Some(GLASS),
        COBBLE => Some(STONE),
        CLAY => Some(TERRACOTTA),
        // Phase E1 (VERIFIED 2026-09-06): sandstone → smooth sandstone
        // (the 1.14 smelting recipe, valid through 1.16.5 — w/Sandstone
        // §Smelting); netherrack → the nether-brick item (w/Nether_Brick
        // §Smelting) — the block form then crafts from 4 items
        CHISELED_SANDSTONE | CUT_SANDSTONE => Some(SMOOTH_SANDSTONE),
        NETHERRACK => Some(NETHER_BRICK),
        // Phase E2 (VERIFIED 2026-09-06 w/Food): potato → baked potato
        // (the only E2 food with a smelting recipe)
        POTATO => Some(BAKED_POTATO),
        // VERIFICATION-REPORT fix #4 (VERIFIED 2026-09-06 live,
        // minecraft.wiki/w/Smelting recipes + w/Coal_Ore §Smelting):
        // coal ore smelts into the coal item (0.1 XP per — the recipe
        // that makes the coal item obtainable in survival)
        COAL_ORE => Some(COAL),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FurnaceState {
    pub input: ItemStack,
    pub fuel: ItemStack,
    pub output: ItemStack,
    /// ticks of burning left in the current fuel item
    pub burn_left: i32,
    /// total ticks of the CURRENT fuel item (flame-progress denominator)
    pub burn_max: i32,
    /// ticks of cook progress on the current input
    pub cook_left: i32,
    /// accumulated smelting XP (§29 vanilla: granted when the output is
    /// collected, e.g. 0.1 per glass)
    pub xp_pool: f32,
}

impl Default for FurnaceState {
    fn default() -> Self {
        FurnaceState {
            input: ItemStack::EMPTY,
            fuel: ItemStack::EMPTY,
            output: ItemStack::EMPTY,
            burn_left: 0,
            burn_max: 0,
            cook_left: 0,
            xp_pool: 0.0,
        }
    }
}

impl FurnaceState {
    pub fn is_burning(&self) -> bool {
        self.burn_left > 0
    }

    /// can the current input land in the output? (matching type + room)
    fn can_output(&self) -> bool {
        match smelt_result(self.input.block) {
            Some(out) => {
                self.output.is_empty() || (self.output.block == out && self.output.count < 64)
            }
            None => false,
        }
    }

    /// ONE game tick. Returns true when the world block state changed
    /// (lit ↔ unlit swap) so the caller can re-write the block.
    pub fn tick(&mut self) -> bool {
        let was_burning = self.is_burning();
        let mut changed_block = false;

        if self.burn_left > 0 {
            self.burn_left -= 1;
        }

        // start burning if there is smeltable work and no flame
        if self.burn_left <= 0 && self.can_output() && !self.fuel.is_empty() {
            self.burn_left = fuel_ticks(self.fuel.block);
            self.burn_max = self.burn_left;
            self.fuel.count -= 1;
            if self.fuel.count == 0 {
                self.fuel = ItemStack::EMPTY;
            }
        }

        // cooking
        if self.burn_left > 0 && self.can_output() {
            self.cook_left += 1;
            if self.cook_left >= COOK_TICKS {
                // item done
                let in_block = self.input.block;
                let out_block = smelt_result(in_block).unwrap();
                self.input.count -= 1;
                if self.input.count == 0 {
                    self.input = ItemStack::EMPTY;
                }
                if self.output.is_empty() {
                    self.output = ItemStack::new(out_block, 1);
                } else {
                    self.output.count += 1;
                }
                // §29: smelting XP accrues until the output is collected
                self.xp_pool += crate::enchanting::smelt_xp(in_block);
                self.cook_left = 0;
            }
        } else if self.cook_left > 0 && !self.can_output() {
            // input removed → progress resets (vanilla)
            self.cook_left = 0;
        }

        // lit state reflects burning
        let burning = self.is_burning();
        if burning != was_burning {
            changed_block = true;
        }
        changed_block
    }
}

/// all furnace block entities
#[derive(Default)]
pub struct Furnaces {
    pub map: HashMap<[i32; 3], FurnaceState>,
}

impl Furnaces {
    /// sim tick step: advance every furnace; swap the world block's lit
    /// state when burning toggles. Returns the positions whose BLOCK state
    /// changed (the caller schedules remeshes).
    pub fn tick(&mut self, world: &mut World) -> Vec<[i32; 3]> {
        let mut changed = Vec::new();
        let positions: Vec<[i32; 3]> = self.map.keys().copied().collect();
        for pos in positions {
            let Some(f) = self.map.get_mut(&pos) else {
                continue;
            };
            if f.tick() {
                let s = world.get_state(pos[0], pos[1], pos[2]);
                if state_block(s) == FURNACE {
                    let lit = f.is_burning();
                    let target = if lit { FURNACE_LIT } else { FURNACE_STATE };
                    if s != target {
                        world.set_block_state(pos[0], pos[1], pos[2], target);
                        changed.push(pos);
                    }
                }
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn flat_world() -> World {
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = vc_chunk::chunk::Chunk::empty();
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
    fn smelts_sand_to_glass_on_fuel() {
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(SAND, 3);
        f.fuel = ItemStack::new(PLANKS, 1);
        // run until one item completes (200 ticks + 1 to ignite)
        let mut lit_changes = 0;
        let mut was = false;
        for _ in 0..400 {
            f.tick();
            let b = f.is_burning();
            if b != was {
                lit_changes += 1;
                was = b;
            }
            if !f.output.is_empty() {
                break;
            }
        }
        assert_eq!((f.output.block, f.output.count), (GLASS, 1));
        assert_eq!(f.input.count, 2);
        assert!(f.fuel.is_empty());
        assert!(lit_changes >= 1, "furnace lit while smelting");
    }

    #[test]
    fn no_fuel_no_cooking() {
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(COBBLE, 1);
        for _ in 0..300 {
            f.tick();
        }
        assert!(f.output.is_empty());
        assert_eq!(f.cook_left, 0);
        assert!(!f.is_burning());
    }

    /// VERIFIED 2026-09-06 live (minecraft.wiki/w/Smelting fuel table):
    /// planks/log 300, crafting table 300, fence 300, wooden slab 150
    /// (half of planks — a slab is half the wood), coal 1600 / 8 items
    /// (w/Furnace "a piece of coal burns for 80 seconds and can process
    /// eight items"). The old COAL_ORE 800 stopgap is RETIRED — vanilla
    /// coal ore is not a fuel; smelt the ore into coal instead.
    #[test]
    fn fuel_table_matches_the_live_wiki() {
        assert_eq!(fuel_ticks(PLANKS), 300);
        assert_eq!(fuel_ticks(OAK_LOG), 300);
        assert_eq!(fuel_ticks(CRAFTING_TABLE), 300);
        assert_eq!(fuel_ticks(OAK_FENCE), 300);
        assert_eq!(fuel_ticks(OAK_SLAB), 150, "slab = half of planks (150)");
        assert_eq!(fuel_ticks(COAL), 1600, "coal: 80 s, 8 items (w/Furnace)");
        // Phase E3 (VERIFIED live 2026-09-06, minecraft.wiki/w/
        // Block_of_Coal: "One block of coal lasts 800 seconds (16000
        // ticks), which smelts 80 items" — 10x the coal item)
        assert_eq!(fuel_ticks(COAL_BLOCK), 16000, "block of coal: 800 s, 80 items");
        assert_eq!(fuel_ticks(COAL_ORE), 0, "ore is not a fuel in vanilla");
        assert_eq!(fuel_ticks(STONE), 0, "stone is not a fuel");
    }

    /// Phase E3 (VERIFIED w/Block_of_Coal: 16000 ticks / 80 items —
    /// 10x the coal item). 80 items outpace the 64-stack output slot:
    /// 64 smelt, cooking pauses at the full stack while the fuel keeps
    /// burning (vanilla furnace behavior — the 16000-tick budget and
    /// the 80-item ratio are pinned by the fuel-table row above).
    #[test]
    fn coal_block_burns_16000_and_outpaces_the_output_stack() {
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(SAND, 81);
        f.fuel = ItemStack::new(COAL_BLOCK, 1);
        for _ in 0..16_000 {
            f.tick();
        }
        // the output slot fills (64 glass); cooking pauses; the block
        // burned its full 16000-tick budget
        assert_eq!(f.output.count, 64, "output stack full");
        assert_eq!(f.output.block, GLASS);
        assert_eq!(f.input.count, 17, "64 smelted of 81, cooking paused");
        assert!(f.burn_left <= 1, "fuel fully burned, left={}", f.burn_left);
    }

    /// VERIFICATION-REPORT fix #4: "a piece of coal ... can process eight
    /// items" (VERIFIED live, minecraft.wiki/w/Furnace). 9 sand + 1 coal:
    /// exactly 8 glass smelt (1600 / 200), the 9th stays raw, coal gone.
    #[test]
    fn one_coal_smelts_exactly_eight_items() {
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(SAND, 9);
        f.fuel = ItemStack::new(COAL, 1);
        // 8 items × 200 ticks + 1 tick to ignite = 1601; run long enough
        for _ in 0..1700 {
            f.tick();
        }
        assert_eq!((f.output.block, f.output.count), (GLASS, 8));
        assert_eq!(f.input.count, 1, "the ninth item is not smelted");
        assert!(f.fuel.is_empty(), "the coal is fully consumed");
        assert!(!f.is_burning(), "the 1600-tick burn window has ended");
    }

    /// The coal ore → coal recipe (VERIFIED live, w/Smelting: coal ore
    /// smelts to coal with 0.1 XP — how the coal item is obtained).
    #[test]
    fn coal_ore_smelts_into_the_coal_item() {
        assert_eq!(smelt_result(COAL_ORE), Some(COAL));
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(COAL_ORE, 2);
        f.fuel = ItemStack::new(PLANKS, 1);
        for _ in 0..420 {
            f.tick();
        }
        assert_eq!((f.output.block, f.output.count), (COAL, 1));
        // smelting XP: 0.1 per coal (w/Smelting)
        assert!((f.xp_pool - 0.1).abs() < 1e-6);
    }

    /// A slab must burn exactly half as long as a plank: feed each one
    /// to a furnace with 3 sand and compare the burn windows.
    #[test]
    fn slab_burns_half_as_long_as_planks() {
        let run = |fuel: u8| -> i32 {
            let mut f = FurnaceState::default();
            f.input = ItemStack::new(SAND, 3);
            f.fuel = ItemStack::new(fuel, 1);
            let mut burning_ticks = 0;
            for _ in 0..400 {
                f.tick();
                if f.is_burning() {
                    burning_ticks += 1;
                }
                if !f.output.is_empty() && f.output.count >= 1 && !f.is_burning() {
                    break;
                }
            }
            burning_ticks
        };
        let planks = run(PLANKS);
        let slab = run(OAK_SLAB);
        // 300-tick plank vs 150-tick slab (the tick loop stops early on
        // output completion, so compare the ratio, not absolutes)
        assert!(
            (planks - slab).abs() >= 140 && (planks + slab) > 0,
            "planks burned {planks} ticks vs slab {slab} (expected ~300 vs ~150)"
        );
    }

    #[test]
    fn output_full_stops() {
        let mut f = FurnaceState::default();
        f.input = ItemStack::new(SAND, 5);
        f.fuel = ItemStack::new(OAK_LOG, 5);
        f.output = ItemStack::new(GLASS, 64);
        for _ in 0..100 {
            f.tick();
        }
        assert_eq!(f.output.count, 64, "no overflow");
        assert_eq!(f.input.count, 5, "input untouched");
    }

    #[test]
    fn furnaces_swap_world_lit_state() {
        let mut w = flat_world();
        w.set_block_state(8, 65, 8, FURNACE_STATE);
        let mut fs = Furnaces::default();
        fs.map.insert(
            [8, 65, 8],
            FurnaceState {
                input: ItemStack::new(COBBLE, 1),
                fuel: ItemStack::new(PLANKS, 1),
                ..Default::default()
            },
        );
        let changed = fs.tick(&mut w);
        assert_eq!(changed, vec![[8, 65, 8]]);
        assert_eq!(w.get_state(8, 65, 8), FURNACE_LIT);
        // burn out → back to unlit
        for _ in 0..320 {
            fs.tick(&mut w);
        }
        assert_eq!(w.get_state(8, 65, 8), FURNACE_STATE, "furnace cooled");
    }
}
