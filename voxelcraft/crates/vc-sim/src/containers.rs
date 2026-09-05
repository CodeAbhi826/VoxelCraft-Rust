//! Container block entities (Phase 3): chests (27 slots), dispensers and
//! droppers (9), hoppers (5). One map keyed by block position; created on
//! first use (right-click or hopper transfer), spilled as item drops when
//! the block breaks (the game layer drains `spilled`).
//!
//! VERIFIED (minecraft.wiki, 2026-09-04): hopper transfer cooldown is 8
//! game ticks (2.5 items/s); dispenser/dropper eject ONE item on the
//! rising edge of a redstone signal with a 4 game tick delay.

use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;

/// slot count per container kind
#[inline]
pub fn slot_count(block: u8) -> Option<usize> {
    Some(match block {
        CHEST => 27,
        DISPENSER | DROPPER => 9,
        HOPPER => 5,
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub struct ContainerInv {
    pub slots: Vec<ItemStack>,
}

impl ContainerInv {
    pub fn new(block: u8) -> Self {
        let n = slot_count(block).unwrap_or(9);
        ContainerInv {
            slots: vec![ItemStack::EMPTY; n],
        }
    }

    /// first non-empty slot index (hopper pulls from the front)
    pub fn first_item(&self) -> Option<usize> {
        self.slots.iter().position(|s| !s.is_empty())
    }

    /// add to the first matching/partial or empty slot; returns leftover
    pub fn add(&mut self, block: u8, count: u8) -> u8 {
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
    pub map: std::collections::HashMap<[i32; 3], ContainerInv>,
    /// positions whose container was destroyed — the game layer spills
    /// these as item drops
    pub spilled: Vec<([i32; 3], Vec<ItemStack>)>,
}

impl Containers {
    /// entry for a position, creating it on first use
    pub fn entry(&mut self, pos: [i32; 3], block: u8) -> &mut ContainerInv {
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
        assert_eq!(slot_count(STONE), None);
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
