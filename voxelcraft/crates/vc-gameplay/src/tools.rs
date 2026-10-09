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
}
