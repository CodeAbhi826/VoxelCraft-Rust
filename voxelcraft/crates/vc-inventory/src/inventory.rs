//! Inventory system (Phase 7 §27 gameplay): item stacks with vanilla 64
//! cap, 36-slot player inventory (9 hotbar + 27 storage), add/merge,
//! click-to-move cursor semantics for the container screens.

use vc_blocks::blocks::*;

pub const STACK_MAX: u8 = 64;
pub const INV_SLOTS: usize = 36; // 0..9 hotbar, 9..36 storage

/// Round 13 (station GUIs): up to TWO enchantments per stack — the
/// anvil's combine mode merges the sacrifice's enchant onto the target,
/// and a one-slot model cannot represent item(ench A) + book(ench B).
/// Vanilla allows more; the two-slot cap is disclosed in
/// docs/research/round-13-station-screens-audit.md §5.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemStack {
    pub block: u16,
    pub count: u8,
    /// carried enchantment (§29): 0 = none; encoding (enchant_id << 8) |
    /// level — books keep their enchant through every slot/cursor move
    /// because the WHOLE struct is copied, never rebuilt from (block,count)
    pub ench: u16,
    /// Round 13: second enchant slot (anvil combine merge target);
    /// 0 = none; same encoding as `ench`
    pub ench2: u16,
    /// Round 13: damage taken (armor pieces; 0 = undamaged). Durability
    /// max comes from the anvil module's armor table (w/Armor live)
    pub dmg: u16,
    /// Round 13: prior anvil USE count (the penalty itself is
    /// 2^prior - 1 — VERIFIED w/Anvil_mechanics, fetched 2026-09-15)
    pub prior: u8,
    /// Round 13: custom-name id into the game layer's name pool
    /// (0 = the registry name). Renamed items stack only with
    /// same-name stacks (VERIFIED w/Anvil §Renaming)
    pub name: u16,
}

impl ItemStack {
    pub const EMPTY: ItemStack = ItemStack {
        block: AIR,
        count: 0,
        ench: 0,
        ench2: 0,
        dmg: 0,
        prior: 0,
        name: 0,
    };

    pub const fn new(block: u16, count: u8) -> Self {
        ItemStack { block, count, ench: 0, ench2: 0, dmg: 0, prior: 0, name: 0 }
    }

    pub const fn new_enchanted(block: u16, count: u8, ench: u16) -> Self {
        ItemStack { block, count, ench, ench2: 0, dmg: 0, prior: 0, name: 0 }
    }

    /// Round 13: a damaged armor piece (the anvil/grindstone repair input)
    pub const fn new_damaged(block: u16, count: u8, dmg: u16, prior: u8) -> Self {
        ItemStack { block, count, ench: 0, ench2: 0, dmg, prior, name: 0 }
    }

    /// decode the carried enchant → (registry id, level) or None
    pub fn enchant(&self) -> Option<(u8, u8)> {
        if self.ench == 0 {
            None
        } else {
            Some(((self.ench >> 8) as u8, (self.ench & 0xFF) as u8))
        }
    }

    /// Round 13: decode the second enchant slot
    pub fn enchant2(&self) -> Option<(u8, u8)> {
        if self.ench2 == 0 {
            None
        } else {
            Some(((self.ench2 >> 8) as u8, (self.ench2 & 0xFF) as u8))
        }
    }

    /// both carried enchants as a pair of Option<(id, level)>
    pub fn enchants(&self) -> [Option<(u8, u8)>; 2] {
        [self.enchant(), self.enchant2()]
    }

    /// encode an enchant onto this stack (id < 256, level 1..=255)
    pub fn set_enchant(&mut self, id: u8, level: u8) {
        self.ench = ((id as u16) << 8) | level as u16;
    }

    /// Round 13: encode the second enchant slot
    pub fn set_enchant2(&mut self, id: u8, level: u8) {
        self.ench2 = ((id as u16) << 8) | level as u16;
    }

    /// Round 13: does this stack carry the given enchant id? Returns
    /// its level when present.
    pub fn enchant_level(&self, id: u8) -> Option<u8> {
        self.enchants().iter().find_map(|e| {
            e.filter(|&(i, _)| i == id).map(|(_, l)| l)
        })
    }

    /// Round 13: vanilla stacking identity — "Named items do not stack
    /// with unnamed or differently-named items of the same type" and
    /// damaged/enchanted pieces only stack at identical state
    /// (w/Anvil §Renaming + §Anvil_mechanics, live 2026-09-15).
    pub fn stackable_with(&self, other: &ItemStack) -> bool {
        self.block == other.block
            && self.ench == other.ench
            && self.ench2 == other.ench2
            && self.dmg == other.dmg
            && self.name == other.name
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.block == AIR
    }

    pub fn split(&mut self) -> ItemStack {
        // vanilla right-click half-take (round up: 3 → 2 + 1)
        let half = self.count.div_ceil(2);
        let mut out = *self;
        out.count = half;
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
    /// order), then empty slots. Returns the leftover count. Round 13:
    /// the merge respects the vanilla stacking identity (renamed /
    /// damaged / enchanted stacks only merge with identical state).
    pub fn add(&mut self, block: u16, mut count: u8) -> u8 {
        if block == AIR || count == 0 {
            return 0;
        }
        // merge pass (plain unnamed undamaged stacks only — the
        // pickup path never carries enchant/name state)
        let probe = ItemStack::new(block, 1);
        for s in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if s.block == block && s.count > 0 && s.count < STACK_MAX && s.stackable_with(&probe) {
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

    /// consume `n` of a block (crafting/placing costs); true when fully paid.
    /// Round 13: consumes only plain stacks (renamed/damaged/enchanted
    /// variants of the same block are never spent — vanilla parity for
    /// named items)
    pub fn consume(&mut self, block: u16, mut n: u8) -> bool {
        let plain = self
            .slots
            .iter()
            .filter(|s| s.block == block && s.count > 0 && s.stackable_with(&ItemStack::new(block, 1)))
            .map(|s| s.count as u32)
            .sum::<u32>();
        if plain < n as u32 {
            return false;
        }
        let probe = ItemStack::new(block, 1);
        for s in self.slots.iter_mut() {
            if n == 0 {
                break;
            }
            if s.block == block && s.count > 0 && s.stackable_with(&probe) {
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
            // whole-stack swap or merge (Round 13: merges require the
            // vanilla stacking identity — renamed/damaged/enchanted
            // stacks of the same block do NOT merge)
            if cursor.is_empty() {
                *cursor = *slot;
                *slot = ItemStack::EMPTY;
            } else if slot.is_empty() {
                *slot = *cursor;
                *cursor = ItemStack::EMPTY;
            } else if slot.stackable_with(cursor) {
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
            } else if slot.is_empty() || (slot.stackable_with(cursor) && slot.count < STACK_MAX) {
                if slot.is_empty() {
                    let mut one = *cursor;
                    one.count = 1;
                    *slot = one;
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

    /// Round 13: renamed / enchanted / damaged stacks of the same block
    /// do NOT merge (VERIFIED w/Anvil §Renaming: "Named items do not
    /// stack with unnamed or differently-named items of the same type").
    #[test]
    fn named_and_stateful_stacks_do_not_merge() {
        let mut named = ItemStack::new(STONE, 32);
        named.name = 3;
        let mut plain = ItemStack::new(STONE, 32);
        assert!(!named.stackable_with(&plain));
        Inventory::slot_click(&mut named, &mut plain, false);
        // refused merge -> swap instead
        assert_eq!((named.block, named.count, named.name), (STONE, 32, 0));
        assert_eq!((plain.block, plain.count, plain.name), (STONE, 32, 3));

        let mut a = ItemStack::new(IRON_HELMET, 1);
        a.set_enchant(1, 2); // Fire Protection II
        let mut b = ItemStack::new(IRON_HELMET, 1);
        b.set_enchant2(1, 2);
        assert!(!a.stackable_with(&b), "slot-1 vs slot-2 differ");

        let mut d1 = ItemStack::new(IRON_HELMET, 1);
        d1.dmg = 40;
        let mut d2 = ItemStack::new(IRON_HELMET, 1);
        d2.dmg = 41;
        assert!(!d1.stackable_with(&d2), "unequal damage never merges");
        d2.dmg = 40;
        assert!(d1.stackable_with(&d2), "identical state merges");
    }

    /// Round 13: split() carries the full state (an enchanted, renamed,
    /// damaged stack splits into two identical-state halves).
    #[test]
    fn split_carries_item_state() {
        let mut s = ItemStack::new(DIAMOND_CHESTPLATE, 2);
        s.set_enchant(0, 4); // Protection IV
        s.dmg = 100;
        s.prior = 2;
        s.name = 7;
        let half = s.split();
        assert_eq!(half.count, 1);
        assert_eq!(s.count, 1);
        assert_eq!(half.ench, s.ench);
        assert_eq!((half.dmg, half.prior, half.name), (100, 2, 7));
    }
}
