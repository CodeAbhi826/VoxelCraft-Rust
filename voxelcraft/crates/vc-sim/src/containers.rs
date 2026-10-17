//! Container block entities (Phase 3): chests (27 slots), dispensers and
//! droppers (9), hoppers (5), item frames (1). One map keyed by block
//! position; created on first use (right-click or hopper transfer),
//! spilled as item drops when the block breaks (the game layer drains
//! `spilled`).
//!
//! Part 6 §6.1: the item frame is the one entry here that is NOT a UI
//! container — no screen ever opens for it. Its single slot exists so the
//! held item rides the persisted container store (save round-trip) and
//! spills on break like every other container.
//!
//! VERIFIED (the reference wiki, 2026-09-04): hopper transfer cooldown is 8
//! game ticks (2.5 items/s); dispenser/dropper eject ONE item on the
//! rising edge of a redstone signal with a 4 game tick delay.

use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;

/// slot count per container kind
#[inline]
pub fn slot_count(block: u16) -> Option<usize> {
    Some(match block {
        CHEST => 27,
        // 1.11 (VERIFIED w/Shulker_Box: "All shulker boxes have 27
        // inventory slots, the same as a barrel, a single chest, or an
        // ender chest")
        SHULKER_BOX => 27,
        // 1.14 (VERIFIED w/Barrel: "Barrels have a container inventory
        // with 27 slots, which is the same as a single chest")
        BARREL => 27,
        DISPENSER | DROPPER => 9,
        HOPPER => 5,
        // Part 6 §6.1: the item frame holds exactly ONE item (VERIFIED
        // w/Item_Frame: "Item frames can be used to hold ... a single
        // item"; the item is displayed in the frame). The item a frame
        // holds is drawn on its plate by the game layer — this slot is
        // not a GUI container.
        ITEM_FRAME => 1,
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub struct ContainerInv {
    pub slots: Vec<ItemStack>,
    /// Part 6 §6.1: the item frame's display rotation, in 45° steps
    /// (0..=7; every other container ignores it). VERIFIED w/Item_Frame:
    /// "The displayed item can be rotated by right-clicking the frame."
    /// Persisted with the container record so a rotated item survives
    /// save/reload (vanilla keeps it in the frame's block entity).
    pub rot: u8,
}

impl ContainerInv {
    pub fn new(block: u16) -> Self {
        let n = slot_count(block).unwrap_or(9);
        ContainerInv {
            slots: vec![ItemStack::EMPTY; n],
            rot: 0,
        }
    }

    /// first non-empty slot index (hopper pulls from the front)
    pub fn first_item(&self) -> Option<usize> {
        self.slots.iter().position(|s| !s.is_empty())
    }

    /// add to the first matching/partial or empty slot; returns leftover
    pub fn add(&mut self, block: u16, count: u8) -> u8 {
        let mut left = count;
        // merge into partial stacks first
        for s in self.slots.iter_mut() {
            if !s.is_empty() && s.block == block && s.count < 64 {
                let take = (64 - s.count).min(left);
                s.count += take;
                left -= take;
                if left == 0 {
                    return 0;
                }
            }
        }
        for s in self.slots.iter_mut() {
            if s.is_empty() {
                let take = 64.min(left);
                *s = ItemStack::new(block, take);
                left -= take;
                if left == 0 {
                    return 0;
                }
            }
        }
        left
    }

    /// fill fraction 0..1 (comparator reading — vanilla signal =
    /// 1 + 14·fill; the fill fraction is over slots, simplified)
    pub fn fill_fraction(&self) -> f32 {
        if self.slots.is_empty() {
            return 0.0;
        }
        let filled = self.slots.iter().filter(|s| !s.is_empty()).count();
        filled as f32 / self.slots.len() as f32
    }
}

#[derive(Default)]
pub struct Containers {
    pub map: rustc_hash::FxHashMap<[i32; 3], ContainerInv>,
    /// positions whose container was destroyed — the game layer spills
    /// these as item drops
    pub spilled: Vec<([i32; 3], Vec<ItemStack>)>,
}

impl Containers {
    /// entry for a position, creating it on first use
    pub fn entry(&mut self, pos: [i32; 3], block: u16) -> &mut ContainerInv {
        self.map
            .entry(pos)
            .or_insert_with(|| ContainerInv::new(block))
    }

    pub fn get(&self, pos: &[i32; 3]) -> Option<&ContainerInv> {
        self.map.get(pos)
    }

    pub fn get_mut(&mut self, pos: &[i32; 3]) -> Option<&mut ContainerInv> {
        self.map.get_mut(pos)
    }

    /// remove a container (block broken) → queue its contents for spill
    pub fn remove(&mut self, pos: &[i32; 3]) {
        if let Some(inv) = self.map.remove(pos) {
            let items: Vec<ItemStack> = inv.slots.into_iter().filter(|s| !s.is_empty()).collect();
            if !items.is_empty() {
                self.spilled.push((*pos, items));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_counts_match_vanilla() {
        // VERIFIED: chest 27, dispenser/dropper 9, hopper 5
        assert_eq!(slot_count(CHEST), Some(27));
        assert_eq!(slot_count(DISPENSER), Some(9));
        assert_eq!(slot_count(DROPPER), Some(9));
        assert_eq!(slot_count(HOPPER), Some(5));
        // Part 6 §6.1: the item frame holds exactly one item (not a GUI)
        assert_eq!(slot_count(ITEM_FRAME), Some(1));
        assert_eq!(ContainerInv::new(ITEM_FRAME).slots.len(), 1);
        assert_eq!(slot_count(STONE), None);
    }

    #[test]
    fn frame_item_rides_the_persisted_store_and_spills() {
        // the frame slot reuses the container store: one item in, one
        // item spilled when the frame breaks (slice-3 drop behaviour)
        let mut cs = Containers::default();
        let pos = [4, 70, -3];
        {
            let f = cs.entry(pos, ITEM_FRAME);
            assert_eq!(f.slots.len(), 1);
            f.slots[0] = ItemStack::new(DIAMOND_BLOCK, 1);
        }
        assert_eq!(cs.get(&pos).unwrap().slots[0].block, DIAMOND_BLOCK);
        cs.remove(&pos);
        assert_eq!(cs.spilled.len(), 1);
        assert_eq!(cs.spilled[0].1[0].block, DIAMOND_BLOCK);
        assert_eq!(cs.spilled[0].1[0].count, 1);
    }

    #[test]
    fn add_merge_and_fill() {
        let mut c = ContainerInv::new(CHEST);
        assert_eq!(c.add(STONE, 40), 0);
        assert_eq!(c.slots[0].count, 40);
        assert_eq!(c.add(STONE, 40), 0); // merges to 64, spills 16 into slot 2
        assert_eq!(c.slots[0].count, 64);
        assert_eq!(c.slots[1].count, 16);
        // 27-slot chest: 40+40+200 = 280 ≤ 27×64 → no leftover
        assert_eq!(c.add(STONE, 200), 0, "27-slot chest swallows it all");
        let fill = c.fill_fraction();
        assert!(fill > 0.0 && fill <= 1.0);
    }

    #[test]
    fn spill_on_remove() {
        let mut cs = Containers::default();
        let pos = [1, 65, 2];
        {
            let e = cs.entry(pos, DISPENSER);
            e.add(DIRT, 5);
        }
        cs.remove(&pos);
        assert_eq!(cs.spilled.len(), 1);
        assert_eq!(cs.spilled[0].1.len(), 1);
        assert_eq!(cs.spilled[0].1[0].count, 5);
        assert!(cs.get(&pos).is_none());
    }
}
