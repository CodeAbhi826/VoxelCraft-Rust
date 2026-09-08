#!/usr/bin/env python3
"""The audit round's test additions — one test block per system."""
import sys

fail = []

def patch(path, edits):
    src = open(path).read()
    for what, old, new in edits:
        if old not in src or src.count(old) != 1:
            fail.append(f"[{path}] ANCHOR ({what}): {old[:60]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)

# ---------------- furnace.rs ----------------
patch("crates/vc-gameplay/src/furnace.rs", [
    (
        "smelt tests",
        """        // the material items never smelt (already refined)
        assert_eq!(smelt_result(NETHERITE_SCRAP), None);
        assert_eq!(smelt_result(NETHERITE_INGOT), None);
    }
}""",
        """        // the material items never smelt (already refined)
        assert_eq!(smelt_result(NETHERITE_SCRAP), None);
        assert_eq!(smelt_result(NETHERITE_INGOT), None);
    }

    /// the 1.0-1.16.5 completeness audit: the cooked-meat family + the
    /// chorus + cactus rows (all VERIFIED against the audit16 captures)
    #[test]
    fn audit16_cooked_meat_and_chorus_smelts() {
        // the six cooked-meat rows (the standing deferral, closed)
        assert_eq!(smelt_result(BEEF), Some(STEAK));
        assert_eq!(smelt_result(PORKCHOP), Some(COOKED_PORKCHOP));
        assert_eq!(smelt_result(CHICKEN_RAW), Some(COOKED_CHICKEN));
        assert_eq!(smelt_result(MUTTON), Some(COOKED_MUTTON));
        assert_eq!(smelt_result(RAW_FISH), Some(COOKED_COD));
        assert_eq!(smelt_result(RAW_SALMON), Some(COOKED_SALMON));
        // 1.9: chorus fruit -> popped ("obtained by smelting chorus
        // fruit", VERIFIED w/Popped_Chorus_Fruit)
        assert_eq!(smelt_result(CHORUS_FRUIT), Some(POPPED_CHORUS_FRUIT));
        // the classic cactus -> green dye row (Cactus Green is the
        // DYE_BASE + 13 palette entry)
        assert_eq!(smelt_result(CACTUS), Some(DYE_BASE + 13));
        // the smoker class now carries the raw meats
        assert!(is_food_smelting(BEEF));
        assert!(is_food_smelting(RAW_SALMON));
        assert!(!is_food_smelting(CHORUS_FRUIT), "chorus is not smoker food");
        // the furnace accepts everything with a recipe; the blast
        // furnace still rejects food (VERIFIED w/Blast_Furnace)
        use crate::furnace::FurnaceKind;
        assert!(FurnaceKind::Furnace.accepts(BEEF));
        assert!(!FurnaceKind::Blast.accepts(BEEF));
        assert!(FurnaceKind::Smoker.accepts(BEEF));
    }
}""",
    ),
])

# ---------------- campfire.rs ----------------
patch("crates/vc-gameplay/src/campfire.rs", [
    (
        "campfire tests",
        """        assert!(outs.contains(&BAKED_POTATO));
        assert!(outs.contains(&COOKED_RABBIT));
        assert!(outs.contains(&DRIED_KELP));
    }
}""",
        """        assert!(outs.contains(&BAKED_POTATO));
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
}""",
    ),
])

# ---------------- brewing.rs ----------------
patch("crates/vc-gameplay/src/brewing.rs", [
    (
        "brewing tests",
        """        // the instant family carries no duration rows
        assert!(potion_effects(POTION_HEALING).is_empty());
        assert!(potion_effects(POTION_WATER).is_empty());
    }
}""",
        """        // the instant family carries no duration rows
        assert!(potion_effects(POTION_HEALING).is_empty());
        assert!(potion_effects(POTION_WATER).is_empty());
    }

    /// the completeness audit: the 1.8 rabbit's-foot deferral, unblocked
    /// — the leaping + regeneration families (VERIFIED live 2026-09-08
    /// against the audit16_page_Potion.json capture)
    #[test]
    fn audit16_leaping_and_regen_brews() {
        // leaping: awkward + rabbit's foot -> 3:00 Jump Boost I
        assert_eq!(brew_result(POTION_AWKWARD, RABBIT_FOOT), Some(POTION_LEAPING));
        // glowstone enhances: 1:30 Jump Boost II
        assert_eq!(brew_result(POTION_LEAPING, GLOWSTONE), Some(POTION_LEAPING_II));
        // regeneration: awkward + ghast tear -> 0:45 Regeneration I
        assert_eq!(brew_result(POTION_AWKWARD, GHAST_TEAR), Some(POTION_REGEN));
        // glowstone enhances: 0:22 Regeneration II
        assert_eq!(brew_result(POTION_REGEN, GLOWSTONE), Some(POTION_REGEN_II));
        // the effect rows: durations exact (3:00 = 3600, 1:30 = 1800,
        // 0:45 = 900, 0:22 = 440; the long rows are registry-gated)
        use crate::effects::EffectKind;
        assert_eq!(potion_effects(POTION_LEAPING), &[(EffectKind::JumpBoost, 0, 3600)]);
        assert_eq!(potion_effects(POTION_LEAPING_II), &[(EffectKind::JumpBoost, 1, 1800)]);
        assert_eq!(potion_effects(POTION_LEAPING_LONG), &[(EffectKind::JumpBoost, 0, 9600)]);
        assert_eq!(potion_effects(POTION_REGEN), &[(EffectKind::Regeneration, 0, 900)]);
        assert_eq!(potion_effects(POTION_REGEN_II), &[(EffectKind::Regeneration, 1, 440)]);
        assert_eq!(potion_effects(POTION_REGEN_LONG), &[(EffectKind::Regeneration, 0, 1800)]);
        // no redstone item: the long rows are registry-only (the
        // SLOW_FALLING_EXT convention — nothing brews them)
        for ing in [GLOWSTONE, RABBIT_FOOT, GHAST_TEAR, MUSHROOM_RED] {
            assert_eq!(brew_result(POTION_LEAPING, ing), if ing == GLOWSTONE {
                Some(POTION_LEAPING_II)
            } else {
                None
            });
        }
    }
}""",
    ),
])

# ---------------- craft.rs ----------------
patch("crates/vc-gameplay/src/craft.rs", [
    (
        "craft tests",
        """    #[test]
    fn log_to_planks_at_any_position() {""",
        """    /// the completeness audit: the kitchen chain + the purpur family
    /// (all VERIFIED live 2026-09-08 against the audit16 captures —
    /// Bowl/Sugar/Mushroom_Stew/Rabbit_Stew/Beetroot_Soup/Pumpkin_Pie/
    /// Popped_Chorus_Fruit)
    #[test]
    fn audit16_kitchen_chain() {
        // bowl: 3 planks -> 4 (shapeless — the V shape's 3 items)
        let mut g = vec![ItemStack::new(PLANKS, 3); 9];
        g[3] = ItemStack::new(DIRT, 1); // must be EXACTLY 3 planks
        assert!(match_grid(&g, 3).is_none(), "stray dirt blocks the bowl");
        let g = vec![
            ItemStack::new(PLANKS, 1), ItemStack::new(PLANKS, 1), ItemStack::EMPTY,
            ItemStack::new(PLANKS, 1), ItemStack::EMPTY, ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BOWL, 4));
        // crimson planks make bowls too (the "Any Planks" row)
        let g = vec![
            ItemStack::new(CRIMSON_PLANKS, 1), ItemStack::new(CRIMSON_PLANKS, 1), ItemStack::new(CRIMSON_PLANKS, 1),
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BOWL, 4));
        // sugar: 1 honey bottle -> 3
        let mut g = vec![ItemStack::EMPTY; 4];
        g[2] = ItemStack::new(HONEY_BOTTLE, 1);
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (SUGAR, 3));
        // mushroom stew: red + brown + bowl (shapeless)
        let mut g = vec![ItemStack::EMPTY; 9];
        g[0] = ItemStack::new(MUSHROOM_RED, 1);
        g[4] = ItemStack::new(MUSHROOM_BROWN, 1);
        g[8] = ItemStack::new(BOWL, 1);
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (MUSHROOM_STEW, 1));
        // rabbit stew: cooked rabbit + carrot + baked potato + mushroom
        // + bowl (VERIFIED w/Rabbit_Stew: the 5-ingredient row)
        let mut g = vec![ItemStack::EMPTY; 9];
        g[0] = ItemStack::new(COOKED_RABBIT, 1);
        g[2] = ItemStack::new(CARROT, 1);
        g[4] = ItemStack::new(BAKED_POTATO, 1);
        g[6] = ItemStack::new(MUSHROOM_BROWN, 1);
        g[8] = ItemStack::new(BOWL, 1);
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (RABBIT_STEW, 1));
        // beetroot soup: 6 beetroot + bowl
        let mut g = vec![ItemStack::new(BEETROOT, 6); 9];
        g[0] = ItemStack::new(BEETROOT, 6);
        g[1] = ItemStack::new(BOWL, 1);
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BEETROOT_SOUP, 1));
        // pumpkin pie: pumpkin + sugar + egg (shapeless)
        let mut g = vec![ItemStack::EMPTY; 4];
        g[0] = ItemStack::new(PUMPKIN, 1);
        g[1] = ItemStack::new(SUGAR, 1);
        g[3] = ItemStack::new(EGG, 1);
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (PUMPKIN_PIE, 1));
    }

    /// the audit: the purpur + end-rod crafts (the 1.9 purpur family
    /// finally crafts from popped chorus)
    #[test]
    fn audit16_purpur_and_end_rod() {
        // 4 popped chorus -> 4 purpur (2x2)
        let g = vec![ItemStack::new(POPPED_CHORUS_FRUIT, 1); 4];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (PURPUR_BLOCK, 4));
        // blaze rod + popped chorus -> 4 end rods (1x2 column)
        let g = vec![
            ItemStack::new(BLAZE_ROD, 1),
            ItemStack::new(POPPED_CHORUS_FRUIT, 1),
        ];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (END_ROD, 4));
    }

    #[test]
    fn log_to_planks_at_any_position() {""",
    ),
])

print("FAILED:" if fail else "test patch 1 applied (furnace/campfire/brewing/craft)")
for f in fail:
    print("  -", f)
sys.exit(1 if fail else 0)
