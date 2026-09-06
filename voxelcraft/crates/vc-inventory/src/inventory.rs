//! Inventory system (Phase 7 §27 gameplay): item stacks with vanilla 64
//! cap, 36-slot player inventory (9 hotbar + 27 storage), add/merge,
//! click-to-move cursor semantics for the container screens.

use vc_blocks::blocks::*;

pub const STACK_MAX: u8 = 64;
pub const INV_SLOTS: usize = 36; // 0..9 hotbar, 9..36 storage

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemStack {
    pub block: u16,
    pub count: u8,
    /// carried enchantment (§29): 0 = none; encoding (enchant_id << 8) |
    /// level — books keep their enchant through every slot/cursor move
    /// because the WHOLE struct is copied, never rebuilt from (block,count)
    pub ench: u16,
}

impl ItemStack {
    pub const EMPTY: ItemStack = ItemStack { block: AIR, count: 0, ench: 0 };

    pub const fn new(block: u16, count: u8) -> Self {
        ItemStack { block, count, ench: 0 }
    }

    pub const fn new_enchanted(block: u16, count: u8, ench: u16) -> Self {
        ItemStack { block, count, ench }
    }

    /// decode the carried enchant → (registry id, level) or None
    pub fn enchant(&self) -> Option<(u8, u8)> {
        if self.ench == 0 {
            None
        } else {
            Some(((self.ench >> 8) as u8, (self.ench & 0xFF) as u8))
        }
    }

    /// encode an enchant onto this stack (id < 256, level 1..=255)
    pub fn set_enchant(&mut self, id: u8, level: u8) {
        self.ench = ((id as u16) << 8) | level.min(255) as u16;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.block == AIR
    }

    pub fn split(&mut self) -> ItemStack {
        // vanilla right-click half-take (round up: 3 → 2 + 1)
        let half = self.count.div_ceil(2);
        let out = ItemStack::new(self.block, half);
        self.count -= half;
        if self.count == 0 {
            *self = Self::EMPTY;
        }
        out
    }
}

#[derive(Clone)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Inventory { slots: vec![ItemStack::EMPTY; capacity] }
    }

    /// vanilla pickup: merge into existing stacks first (hotbar-first
    /// order), then empty slots. Returns the leftover count.
    pub fn add(&mut self, block: u16, mut count: u8) -> u8 {
        if block == AIR || count == 0 {
            return 0;
        }
        // merge pass
        for s in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if s.block == block && s.count > 0 && s.count < STACK_MAX {
                let room = STACK_MAX - s.count;
                let take = room.min(count);
                s.count += take;
                count -= take;
            }
        }
        // empty-slot pass
        for s in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if s.is_empty() {
                let take = STACK_MAX.min(count);
                *s = ItemStack::new(block, take);
                count -= take;
            }
        }
        count
    }

    /// count of a block across all slots
    pub fn count_of(&self, block: u16) -> u32 {
        self.slots
            .iter()
            .filter(|s| s.block == block)
            .map(|s| s.count as u32)
            .sum()
    }

    /// consume `n` of a block (crafting/placing costs); true when fully paid
    pub fn consume(&mut self, block: u16, mut n: u8) -> bool {
        if self.count_of(block) < n as u32 {
            return false;
        }
        for s in self.slots.iter_mut() {
            if n == 0 {
                break;
            }
            if s.block == block && s.count > 0 {
                let take = s.count.min(n);
                s.count -= take;
                n -= take;
                if s.count == 0 {
                    *s = ItemStack::EMPTY;
                }
            }
        }
        true
    }

    /// click semantics for container UIs: swap/merge the cursor with a
    /// slot; LEFT = whole stack, RIGHT = single item (place) / half (take)
    pub fn slot_click(
        slot: &mut ItemStack,
        cursor: &mut ItemStack,
        right_click: bool,
    ) {
        if !right_click {
            // whole-stack swap or merge
            if cursor.is_empty() {
                *cursor = *slot;
                *slot = ItemStack::EMPTY;
            } else if slot.is_empty() {
                *slot = *cursor;
                *cursor = ItemStack::EMPTY;
            } else if slot.block == cursor.block {
                let room = STACK_MAX - slot.count;
                let take = room.min(cursor.count);
                slot.count += take;
                cursor.count -= take;
                if cursor.count == 0 {
                    *cursor = ItemStack::EMPTY;
                }
            } else {
                std::mem::swap(slot, cursor);
            }
        } else {
            // right-click: take half from the slot, or place ONE
            if cursor.is_empty() {
                if !slot.is_empty() {
                    *cursor = slot.split();
                }
            } else if slot.is_empty() || (slot.block == cursor.block && slot.count < STACK_MAX) {
                if slot.is_empty() {
                    *slot = ItemStack::new(cursor.block, 1);
                } else {
                    slot.count += 1;
                }
                cursor.count -= 1;
                if cursor.count == 0 {
                    *cursor = ItemStack::EMPTY;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_merges_then_fills() {
        let mut inv = Inventory::new(4);
        assert_eq!(inv.add(DIRT, 70), 0);
        assert_eq!(inv.slots[0], ItemStack::new(DIRT, 64));
        assert_eq!(inv.slots[1], ItemStack::new(DIRT, 6));
        // more merges into the partial stack first
        assert_eq!(inv.add(DIRT, 10), 0);
        assert_eq!(inv.slots[1].count, 16);
        assert_eq!(inv.count_of(DIRT), 80);
    }

    #[test]
    fn add_reports_overflow() {
        let mut inv = Inventory::new(1);
        assert_eq!(inv.add(STONE, 100), 36); // 64 in, 36 left over
        assert_eq!(inv.slots[0].count, 64);
    }

    #[test]
    fn consume_across_stacks() {
        let mut inv = Inventory::new(3);
        inv.add(PLANKS, 30);
        inv.add(SAND, 5);
        inv.add(PLANKS, 20);
        assert!(inv.consume(PLANKS, 45));
        assert_eq!(inv.count_of(PLANKS), 5);
        assert!(!inv.consume(PLANKS, 6), "not enough");
    }

    #[test]
    fn click_swap_merge_split() {
        // left swap
        let mut slot = ItemStack::new(STONE, 10);
        let mut cursor = ItemStack::new(DIRT, 5);
        Inventory::slot_click(&mut slot, &mut cursor, false);
        assert_eq!((slot.block, slot.count), (DIRT, 5));
        assert_eq!((cursor.block, cursor.count), (STONE, 10));
        // left merge
        let mut slot2 = ItemStack::new(STONE, 60);
        let mut cursor2 = ItemStack::new(STONE, 10);
        Inventory::slot_click(&mut slot2, &mut cursor2, false);
        assert_eq!(slot2.count, 64);
        assert_eq!(cursor2.count, 6);
        // right half-take
        let mut slot3 = ItemStack::new(DIRT, 9);
        let mut cursor3 = ItemStack::EMPTY;
        Inventory::slot_click(&mut slot3, &mut cursor3, true);
        assert_eq!(cursor3.count, 5);
        assert_eq!(slot3.count, 4);
        // right place-one
        let mut slot4 = ItemStack::EMPTY;
        let mut cursor4 = ItemStack::new(SAND, 3);
        Inventory::slot_click(&mut slot4, &mut cursor4, true);
        assert_eq!(slot4.count, 1);
        assert_eq!(cursor4.count, 2);
    }
}
