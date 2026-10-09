//! 3.1b: tool/weapon stats — the data behind the 540..=569 registry window.
//!
//! Material-major id order (sword/pickaxe/axe/shovel/hoe per material,
//! matching the blocks.rs layout comment): tier = (id − 540) / 5, class
//! = (id − 540) % 5. All values are the vanilla 1.16.5 set (see the
//! reference wiki Swords / Axes / Pickaxes / Shovels / Hoes infoboxes
//! for damage, attack speed, dig speed, and durability columns).
//! Art stays placeholder until Part 6; behavior (melee wiring in
//! combat.rs, mining speeds in 3.1c) reads these tables, never literals.

use vc_blocks::blocks as blk;

/// Material tier, in id order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    Wood,
    Stone,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

/// Tool class, in id order within each material.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolClass {
    Sword,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
}

/// Split a tool/weapon id into (tier, class); None outside 540..=569.
pub fn tool_kind(item: u16) -> Option<(Tier, ToolClass)> {
    if !(blk::WOODEN_SWORD..=blk::NETHERITE_HOE).contains(&item) {
        return None;
    }
    let tier = match (item - blk::WOODEN_SWORD) / 5 {
        0 => Tier::Wood,
        1 => Tier::Stone,
        2 => Tier::Iron,
        3 => Tier::Gold,
        4 => Tier::Diamond,
        5 => Tier::Netherite,
        _ => return None,
    };
    let class = match (item - blk::WOODEN_SWORD) % 5 {
        0 => ToolClass::Sword,
        1 => ToolClass::Pickaxe,
        2 => ToolClass::Axe,
        3 => ToolClass::Shovel,
        4 => ToolClass::Hoe,
        _ => return None,
    };
    Some((tier, class))
}

/// Dig-speed multiplier (the tier base; 1.0 = bare hand / non-tool).
pub fn dig_speed(item: u16) -> f32 {
    let Some((tier, _)) = tool_kind(item) else {
        return 1.0;
    };
    match tier {
        Tier::Wood => 2.0,
        Tier::Stone => 4.0,
        Tier::Iron => 6.0,
        Tier::Gold => 12.0,
        Tier::Diamond => 8.0,
        Tier::Netherite => 9.0,
    }
}

/// Max durability (hits/blocks before breaking); 0 = not a tool.
pub fn durability_max(item: u16) -> u16 {
    // 3.3c: the bow is not tiered (vanilla 384)
    if item == blk::BOW {
        return 384;
    }
    let Some((tier, _)) = tool_kind(item) else {
        return 0;
    };
    match tier {
        Tier::Wood => 59,
        Tier::Stone => 131,
        Tier::Iron => 250,
        Tier::Gold => 32,
        Tier::Diamond => 1561,
        Tier::Netherite => 2031,
    }
}

/// Melee (damage, attack speed); None for non-tools (caller falls back
/// to the fist profile).
pub fn melee_profile(item: u16) -> Option<(f32, f32)> {
    let (tier, class) = tool_kind(item)?;
    let damage = match (tier, class) {
        (_, ToolClass::Sword) => [4.0, 5.0, 6.0, 4.0, 7.0, 8.0][tier as usize],
        (_, ToolClass::Pickaxe) => [2.0, 3.0, 4.0, 2.0, 5.0, 6.0][tier as usize],
        (_, ToolClass::Axe) => [7.0, 9.0, 9.0, 7.0, 9.0, 10.0][tier as usize],
        (_, ToolClass::Shovel) => [2.5, 3.5, 4.5, 2.5, 5.5, 6.5][tier as usize],
        (_, ToolClass::Hoe) => 1.0,
    };
    let speed = match class {
        ToolClass::Sword => 1.6,
        ToolClass::Pickaxe => 1.2,
        ToolClass::Shovel => 1.0,
        ToolClass::Axe => [0.8, 0.8, 0.9, 1.0, 1.0, 1.0][tier as usize],
        ToolClass::Hoe => [1.0, 2.0, 3.0, 1.0, 4.0, 4.0][tier as usize],
    };
    Some((damage, speed))
}

/// Apply `amount` wear to a stack against its `max` durability; returns
/// true when the item BROKE (stack zeroed, vanilla: the item shatters).
/// max 0 (non-tools, and callers pass armor maxima from the anvil table
/// for armor pieces) never breaks.
pub fn damage_item(stack: &mut vc_inventory::inventory::ItemStack, amount: u16, max: u16) -> bool {
    if max == 0 {
        return false;
    }
    let next = stack.dmg.saturating_add(amount);
    if next >= max {
        *stack = vc_inventory::inventory::ItemStack::EMPTY;
        return true;
    }
    stack.dmg = next;
    false
}

/// Enchant level on a stack's two slots for registry `id`
/// (ItemStack encoding: (id << 8) | level); 0 when absent.
pub fn ench_level(ench: u16, ench2: u16, id: u8) -> u32 {
    for e in [ench, ench2] {
        if (e >> 8) as u8 == id {
            return (e & 0xFF) as u32;
        }
    }
    0
}

/// The correct tool class for a block (None = bare hand mines at full
/// hand time). Covers the clear vanilla-preferred cases; everything
/// else falls back to hand speed (drops-gating for wrong tools is a
/// separate follow-up, disclosed).
pub fn preferred_tool(block: u16) -> Option<ToolClass> {
    use vc_blocks::blocks as blk;
    match block {
        blk::STONE
        | blk::COBBLE
        | blk::GRANITE
        | blk::DIORITE
        | blk::ANDESITE
        | blk::STONE_BRICKS
        | blk::BRICKS
        | blk::MOSSY_COBBLE
        | blk::SMOOTH_STONE
        | blk::OBSIDIAN
        | blk::NETHERRACK
        | blk::END_STONE
        | blk::NETHER_BRICKS
        | blk::COAL_ORE
        | blk::IRON_ORE
        | blk::GOLD_ORE
        | blk::DIAMOND_ORE
        | blk::REDSTONE_ORE
        | blk::LAPIS_ORE
        | blk::EMERALD_ORE
        | blk::NETHER_QUARTZ_ORE => Some(ToolClass::Pickaxe),
        blk::OAK_LOG
        | blk::BIRCH_LOG
        | blk::SPRUCE_LOG
        | blk::ACACIA_LOG
        | blk::DARK_OAK_LOG
        | blk::PLANKS
        | blk::OAK_FENCE
        | blk::CRAFTING_TABLE
        | blk::CHEST
        | blk::BOOKSHELF
        | blk::PUMPKIN
        | blk::MELON => Some(ToolClass::Axe),
        blk::DIRT
        | blk::GRASS
        | blk::SAND
        | blk::GRAVEL
        | blk::SOUL_SAND
        | blk::SNOW
        | blk::CLAY => Some(ToolClass::Shovel),
        blk::LEAVES
        | blk::BIRCH_LEAVES
        | blk::SPRUCE_LEAVES
        | blk::ACACIA_LEAVES
        | blk::DARK_OAK_LEAVES
        | blk::HAY_BALE => Some(ToolClass::Hoe),
        _ => None,
    }
}

/// Full break time for `block` with `held` in hand: hand time divided
/// by the tier multiplier when the held class is preferred (+30% per
/// Efficiency level). Instant (0) and unbreakable (inf) hand times pass
/// through untouched.
pub fn mine_time_secs(block: u16, held: u16, efficiency_level: u32) -> f32 {
    let hand = vc_blocks::blocks::break_time_secs(block);
    let Some((_, class)) = tool_kind(held) else {
        return hand;
    };
    if preferred_tool(block) != Some(class) {
        return hand;
    }
    let speed = dig_speed(held) * (1.0 + 0.3 * efficiency_level as f32);
    if speed <= 0.0 {
        hand
    } else {
        hand / speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_mapping_covers_all_30() {
        assert_eq!(
            tool_kind(blk::WOODEN_SWORD),
            Some((Tier::Wood, ToolClass::Sword))
        );
        assert_eq!(
            tool_kind(blk::NETHERITE_HOE),
            Some((Tier::Netherite, ToolClass::Hoe))
        );
        assert_eq!(
            tool_kind(blk::IRON_PICKAXE),
            Some((Tier::Iron, ToolClass::Pickaxe))
        );
        assert_eq!(
            tool_kind(blk::GOLDEN_AXE),
            Some((Tier::Gold, ToolClass::Axe))
        );
        assert_eq!(
            tool_kind(blk::DIAMOND_SHOVEL),
            Some((Tier::Diamond, ToolClass::Shovel))
        );
        assert_eq!(tool_kind(539), None);
        assert_eq!(tool_kind(570), None);
        assert_eq!(tool_kind(blk::STONE), None);
    }

    #[test]
    fn dig_speed_matches_tier_base() {
        assert_eq!(dig_speed(blk::WOODEN_PICKAXE), 2.0);
        assert_eq!(dig_speed(blk::STONE_PICKAXE), 4.0);
        assert_eq!(dig_speed(blk::IRON_PICKAXE), 6.0);
        assert_eq!(dig_speed(blk::GOLDEN_PICKAXE), 12.0);
        assert_eq!(dig_speed(blk::DIAMOND_PICKAXE), 8.0);
        assert_eq!(dig_speed(blk::NETHERITE_PICKAXE), 9.0);
        assert_eq!(dig_speed(blk::STONE), 1.0);
    }

    #[test]
    fn durability_matches_vanilla_max() {
        assert_eq!(durability_max(blk::WOODEN_SWORD), 59);
        assert_eq!(durability_max(blk::GOLDEN_SWORD), 32);
        assert_eq!(durability_max(blk::DIAMOND_SWORD), 1561);
        assert_eq!(durability_max(blk::NETHERITE_SWORD), 2031);
        assert_eq!(durability_max(blk::STONE), 0);
    }

    #[test]
    fn melee_profiles_match_vanilla() {
        assert_eq!(melee_profile(blk::WOODEN_SWORD), Some((4.0, 1.6)));
        assert_eq!(melee_profile(blk::DIAMOND_SWORD), Some((7.0, 1.6)));
        assert_eq!(melee_profile(blk::IRON_AXE), Some((9.0, 0.9)));
        assert_eq!(melee_profile(blk::STONE_SHOVEL), Some((3.5, 1.0)));
        assert_eq!(melee_profile(blk::DIAMOND_PICKAXE), Some((5.0, 1.2)));
        assert_eq!(melee_profile(blk::STONE_HOE), Some((1.0, 2.0)));
        assert_eq!(melee_profile(blk::STONE), None);
    }

    #[test]
    fn damage_item_accumulates_and_breaks() {
        use vc_inventory::inventory::ItemStack;
        let mut s = ItemStack::new(blk::IRON_PICKAXE, 1);
        // 250 max: 249 hits survive, the 250th breaks (stack zeroed)
        let max = durability_max(s.block);
        for _ in 0..249 {
            assert!(!damage_item(&mut s, 1, max));
        }
        assert_eq!(s.dmg, 249);
        assert!(damage_item(&mut s, 1, durability_max(blk::IRON_PICKAXE)));
        assert_eq!(s, ItemStack::EMPTY);
        // non-tools never break (max 0)
        let mut stone = ItemStack::new(blk::STONE, 64);
        assert!(!damage_item(&mut stone, 1, durability_max(blk::STONE)));
        assert_eq!(stone.dmg, 0);
        // overkill in one hit still just breaks
        let mut wood = ItemStack::new(blk::WOODEN_SWORD, 1);
        assert!(damage_item(
            &mut wood,
            1000,
            durability_max(blk::WOODEN_SWORD)
        ));
        assert_eq!(wood, ItemStack::EMPTY);
    }

    #[test]
    fn ench_level_reads_both_slots() {
        let id = crate::enchanting::enchant_by_id("sweeping").unwrap();
        let enc = ((id as u16) << 8) | 3;
        assert_eq!(ench_level(enc, 0, id), 3);
        assert_eq!(ench_level(0, enc, id), 3);
        assert_eq!(ench_level(0, 0, id), 0);
        // wrong enchant id in the slot reads 0
        assert_eq!(ench_level(((id as u16 + 1) << 8) | 3, 0, id), 0);
    }

    #[test]
    fn preferred_tool_covers_the_clear_classes() {
        use vc_blocks::blocks as blk;
        assert_eq!(preferred_tool(blk::STONE), Some(ToolClass::Pickaxe));
        assert_eq!(preferred_tool(blk::DIAMOND_ORE), Some(ToolClass::Pickaxe));
        assert_eq!(preferred_tool(blk::OAK_LOG), Some(ToolClass::Axe));
        assert_eq!(preferred_tool(blk::CHEST), Some(ToolClass::Axe));
        assert_eq!(preferred_tool(blk::DIRT), Some(ToolClass::Shovel));
        assert_eq!(preferred_tool(blk::SAND), Some(ToolClass::Shovel));
        assert_eq!(preferred_tool(blk::LEAVES), Some(ToolClass::Hoe));
        assert_eq!(preferred_tool(blk::HAY_BALE), Some(ToolClass::Hoe));
        assert_eq!(preferred_tool(blk::BEDROCK), None);
        assert_eq!(preferred_tool(blk::GLASS), None);
    }

    #[test]
    fn mine_time_divides_by_tier_speed() {
        use vc_blocks::blocks as blk;
        // stone hand time 2.25 (hardness 1.5 x 1.5); diamond pick
        // speed 8 -> ~0.28; wrong tool (axe) stays hand time
        let hand = vc_blocks::blocks::break_time_secs(blk::STONE);
        assert!((hand - 2.25).abs() < 1e-6);
        assert!((mine_time_secs(blk::STONE, blk::DIAMOND_PICKAXE, 0) - hand / 8.0).abs() < 1e-6);
        assert!((mine_time_secs(blk::STONE, blk::DIAMOND_AXE, 0) - hand).abs() < 1e-6);
        assert!((mine_time_secs(blk::STONE, blk::STONE, 0) - hand).abs() < 1e-6);
        // efficiency III: x1.9
        assert!(
            (mine_time_secs(blk::STONE, blk::DIAMOND_PICKAXE, 3) - hand / 8.0 / 1.9).abs() < 1e-5
        );
        // instant (plants) and unbreakable (bedrock) pass through
        assert_eq!(
            mine_time_secs(blk::TALL_GRASS, blk::DIAMOND_PICKAXE, 5),
            0.0
        );
        assert!(mine_time_secs(blk::BEDROCK, blk::DIAMOND_PICKAXE, 5).is_infinite());
    }
}
