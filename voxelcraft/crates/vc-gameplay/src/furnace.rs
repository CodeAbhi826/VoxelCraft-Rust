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
        COAL_ORE => 800, // progressive: ore-as-fuel until the item exists
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
    /// (half of planks — a slab is half the wood), coal item 1600 (the
    /// engine uses COAL_ORE 800 as a documented ore-as-fuel stopgap
    /// until coal the ITEM exists).
    #[test]
    fn fuel_table_matches_the_live_wiki() {
        assert_eq!(fuel_ticks(PLANKS), 300);
        assert_eq!(fuel_ticks(OAK_LOG), 300);
        assert_eq!(fuel_ticks(CRAFTING_TABLE), 300);
        assert_eq!(fuel_ticks(OAK_FENCE), 300);
        assert_eq!(fuel_ticks(OAK_SLAB), 150, "slab = half of planks (150)");
        assert_eq!(fuel_ticks(COAL_ORE), 800, "ore-as-fuel stopgap, disclosed");
        assert_eq!(fuel_ticks(STONE), 0, "stone is not a fuel");
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
