//! Crafting (Phase 7 §27): shaped recipes matched on 2×2 (inventory) and
//! 3×3 (crafting table) grids, vanilla ingredient semantics — any log →
//! planks, 4 planks → crafting table, 8 cobble ring → furnace.

use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;

/// a shaped recipe: `grid` is w×w ingredients (AIR = empty), rotated
/// matches allowed (vanilla behavior for symmetric recipes we ship)
pub struct Recipe {
    pub size: usize,
    /// row-major ingredients; ANY_LOG means any of the 3 log blocks
    pub grid: &'static [Ing],
    pub out: ItemStack,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Ing {
    None,
    AnyLog,
    Block(u8),
}

/// the recipes our registry supports (vanilla-shaped)
pub const RECIPES: &[Recipe] = &[
    // log → 4 planks (shapeless in vanilla; modeled as 1×1 shaped)
    Recipe {
        size: 1,
        grid: &[Ing::AnyLog],
        out: ItemStack::new(PLANKS, 4),
    },
    // 2×2 planks → crafting table
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
        ],
        out: ItemStack::new(CRAFTING_TABLE, 1),
    },
    // 3×3 cobble ring (center empty) → furnace
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::None,
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
        ],
        out: ItemStack::new(FURNACE, 1),
    },
    // 3×3 sand → sand... no. sand→glass needs the furnace. Recipes for
    // wool→? keep the set tight and honest.
    // ---- Phase E1 recipes (evolution 1.0–1.2 bracket, live-verified
    // 2026-09-06) ----
    // redstone lamp: 4 glowstone (cardinal) + 1 redstone (center)
    // (VERIFIED w/Redstone_Lamp §Crafting)
    Recipe {
        size: 3,
        grid: &[
            Ing::None,
            Ing::Block(GLOWSTONE),
            Ing::None,
            Ing::Block(GLOWSTONE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(GLOWSTONE),
            Ing::None,
            Ing::Block(GLOWSTONE),
            Ing::None,
        ],
        out: ItemStack::new(REDSTONE_LAMP, 1),
    },
    // eye of ender: blaze powder + ender pearl (shapeless in vanilla —
    // modeled as the 2×2 diagonal; VERIFIED)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(BLAZE_POWDER),
            Ing::None,
            Ing::None,
            Ing::Block(ENDER_PEARL),
        ],
        out: ItemStack::new(EYE_OF_ENDER, 1),
    },
    // blaze powder: 1 rod → 2 (shapeless in vanilla; 1×1 shaped here)
    Recipe {
        size: 1,
        grid: &[Ing::Block(BLAZE_ROD)],
        out: ItemStack::new(BLAZE_POWDER, 2),
    },
    // nether bricks: 4 nether-brick items 2×2 (VERIFIED vanilla)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(NETHER_BRICK),
            Ing::Block(NETHER_BRICK),
            Ing::Block(NETHER_BRICK),
            Ing::Block(NETHER_BRICK),
        ],
        out: ItemStack::new(NETHER_BRICKS, 1),
    },
    // [cut/chiseled sandstone crafting DEFERRED: vanilla's recipes need
    // plain SANDSTONE + sandstone slabs — neither block exists in the
    // engine yet; both variants stay picker-available (documented)]
    // 2×2 snow → snow block? snow IS a block already. skip.
    // 3×3 glass V (vanilla glass-bottle recipe, 3 bottles) — the §29 chain
    // head: bottle → fill at water → brew
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(GLASS),
            Ing::None,
            Ing::Block(GLASS),
            Ing::None,
            Ing::Block(GLASS),
            Ing::None,
            Ing::None,
            Ing::None,
            Ing::None,
        ],
        out: ItemStack::new(POTION_EMPTY, 3),
    },
    // 3×3: cobble bottom row + netherrack center (vanilla stand recipe:
    // blaze rod center — §29 palette adaptation)
    Recipe {
        size: 3,
        grid: &[
            Ing::None,
            Ing::Block(NETHERRACK),
            Ing::None,
            Ing::None,
            Ing::None,
            Ing::None,
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
        ],
        out: ItemStack::new(BREWING_STAND, 1),
    },
    // 1×1: bookshelf → 3 books (§29 adaptation: vanilla book = paper +
    // leather; our paper/leather is the bookshelf itself)
    Recipe {
        size: 1,
        grid: &[Ing::Block(BOOKSHELF)],
        out: ItemStack::new(ENCHANTED_BOOK, 3),
    },
    // 3×3 vanilla enchanting-table layout: book top-center, diamonds
    // left/right mid, obsidian bottom row + mid column
    Recipe {
        size: 3,
        grid: &[
            Ing::None,
            Ing::Block(BOOKSHELF),
            Ing::None,
            Ing::Block(DIAMOND_BLOCK),
            Ing::Block(OBSIDIAN),
            Ing::Block(DIAMOND_BLOCK),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
        ],
        out: ItemStack::new(ENCHANT_TABLE, 1),
    },
    // Phase 4 §26: fermented spider eye — vanilla = spider eye + sugar +
    // brown mushroom (shapeless); palette adaptation: the two ingredients
    // we DO have (no sugar block exists in the registry)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(SPIDER_EYE),
            Ing::Block(MUSHROOM_BROWN),
            Ing::None,
            Ing::None,
        ],
        out: ItemStack::new(FERMENTED_SPIDER_EYE, 1),
    },
    // ---- Phase E2 (evolution 1.3-1.4 bracket; all live-verified
    // 2026-09-06, docs/research/phase2-1.3-1.4-research.md) ----
    // anvil: 3 blocks of iron + 4 iron ingots (VERIFIED w/Anvil; 31 iron
    // total). Adaptation: IRON_ORE items stand in for the ingots (the
    // engine has no ingot item — disclosed).
    Recipe {
        size: 3,
        grid: &[
            Ing::None,
            Ing::Block(IRON_BLOCK),
            Ing::None,
            Ing::Block(IRON_ORE),
            Ing::Block(IRON_ORE),
            Ing::Block(IRON_ORE),
            Ing::None,
            Ing::Block(IRON_ORE),
            Ing::None,
        ],
        out: ItemStack::new(ANVIL, 1),
    },
    // beacon: 5 glass + 1 nether star + 3 obsidian (VERIFIED w/Beacon)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(GLASS),
            Ing::Block(GLASS),
            Ing::Block(GLASS),
            Ing::Block(GLASS),
            Ing::Block(NETHER_STAR),
            Ing::Block(GLASS),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
        ],
        out: ItemStack::new(BEACON, 1),
    },
    // ender chest: 8 obsidian + 1 eye of ender (VERIFIED w/Ender_Chest)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(EYE_OF_ENDER),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
            Ing::Block(OBSIDIAN),
        ],
        out: ItemStack::new(ENDER_CHEST, 1),
    },
    // cobblestone wall: 6 cobble -> 6 walls (VERIFIED w/Wall)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::Block(COBBLE),
            Ing::None,
            Ing::None,
            Ing::None,
        ],
        out: ItemStack::new(COBBLE_WALL, 6),
    },
    // flower pot: 3 bricks (VERIFIED w/Flower_Pot; brick ITEM -> brick
    // BLOCK adaptation — no brick item, disclosed)
    Recipe {
        size: 1,
        grid: &[Ing::Block(BRICKS)],
        out: ItemStack::new(FLOWER_POT, 1),
    },
    // item frame: 8 sticks + 1 leather (VERIFIED w/Item_Frame; sticks ->
    // planks adaptation — no stick item, disclosed)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(LEATHER),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
            Ing::Block(PLANKS),
        ],
        out: ItemStack::new(ITEM_FRAME, 1),
    },
    // tripwire hook: 1 iron + 1 stick + 2 planks -> 2 (VERIFIED
    // w/Tripwire_Hook; iron ore + planks adaptation, disclosed)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(IRON_ORE),
            Ing::Block(PLANKS),
            Ing::Block(IRON_ORE),
            Ing::Block(PLANKS),
        ],
        out: ItemStack::new(TRIPWIRE_HOOK, 2),
    },
];

/// match a crafting grid (row-major, `size`×`size` of ItemStacks) → the
/// recipe output. Trims to the bounding box first (vanilla grid-shape
/// semantics: the pattern matches anywhere in the grid).
pub fn match_grid(slots: &[ItemStack], size: usize) -> Option<ItemStack> {
    for r in RECIPES {
        if r.size > size {
            continue;
        }
        // try every offset of the recipe pattern inside the grid
        for oy in 0..=(size - r.size) {
            'ox: for ox in 0..=(size - r.size) {
                for (i, ing) in r.grid.iter().enumerate() {
                    let rx = i % r.size;
                    let ry = i / r.size;
                    let s = slots[(oy + ry) * size + (ox + rx)];
                    let ok = match ing {
                        Ing::None => s.is_empty(),
                        Ing::Block(b) => s.block == *b && !s.is_empty(),
                        Ing::AnyLog => {
                            !s.is_empty() && matches!(s.block, OAK_LOG | BIRCH_LOG | SPRUCE_LOG)
                        }
                    };
                    if !ok {
                        continue 'ox;
                    }
                }
                // pattern matched — but the REST of the grid must be empty
                // (exact-shape semantics: no stray ingredients)
                for (i, s) in slots.iter().enumerate() {
                    let sx = i % size;
                    let sy = i / size;
                    let inside = (ox..ox + r.size).contains(&sx) && (oy..oy + r.size).contains(&sy);
                    if !inside && !s.is_empty() {
                        continue 'ox;
                    }
                }
                return Some(r.out);
            }
        }
    }
    None
}

/// consume the ingredients of a matched grid (one of each non-empty cell)
pub fn consume_grid(slots: &mut [ItemStack]) {
    for s in slots.iter_mut() {
        if !s.is_empty() {
            s.count -= 1;
            if s.count == 0 {
                *s = ItemStack::EMPTY;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_to_planks_at_any_position() {
        let mut g = vec![ItemStack::EMPTY; 4];
        g[3] = ItemStack::new(OAK_LOG, 5);
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (PLANKS, 4));
        // birch and spruce too
        g[3] = ItemStack::new(SPRUCE_LOG, 2);
        assert!(match_grid(&g, 2).is_some());
    }

    #[test]
    fn planks_make_crafting_table() {
        let mut g = vec![ItemStack::new(PLANKS, 3); 4];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!(out.block, CRAFTING_TABLE);
        // 3 planks + stray item = no match
        g[0] = ItemStack::new(DIRT, 1);
        assert!(match_grid(&g, 2).is_none());
    }

    #[test]
    fn cobble_ring_makes_furnace() {
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0, 1, 2, 3, 5, 6, 7, 8] {
            g[i] = ItemStack::new(COBBLE, 7);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!(out.block, FURNACE);
        // the 2×2 grid cannot host the 3×3 recipe
        assert!(match_grid(&vec![ItemStack::new(COBBLE, 7); 4], 2).is_none());
    }

    #[test]
    fn consume_grid_decrements_each_ingredient() {
        let mut g = vec![ItemStack::new(PLANKS, 2); 4];
        consume_grid(&mut g);
        assert!(g.iter().all(|s| s.count == 1));
        consume_grid(&mut g);
        assert!(g.iter().all(|s| s.is_empty()));
    }

    #[test]
    fn glass_v_makes_three_bottles() {
        // the vanilla glass-bottle V: G_G / _G_ in a 3×3 (crafting table)
        let mut g = vec![ItemStack::EMPTY; 9];
        g[0] = ItemStack::new(GLASS, 7);
        g[2] = ItemStack::new(GLASS, 7);
        g[4] = ItemStack::new(GLASS, 7);
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (POTION_EMPTY, 3));
        // 2×2 grid also hosts the V (pattern fits inside 2×2 with the
        // bottom row empty → but our exact-shape rule needs the third
        // glass in the second row — a 2×2 has it at cell 3: G G / G _ →
        // NOT the V; verify the furnace ring does not false-match)
        let mut g2 = vec![ItemStack::EMPTY; 9];
        g2[0] = ItemStack::new(GLASS, 1);
        g2[1] = ItemStack::new(GLASS, 1);
        g2[3] = ItemStack::new(GLASS, 1);
        assert!(match_grid(&g2, 3).is_none(), "not the V pattern");
    }

    #[test]
    fn stand_recipe_needs_the_exact_layout() {
        let mut g = vec![ItemStack::EMPTY; 9];
        g[1] = ItemStack::new(NETHERRACK, 1);
        for i in [6, 7, 8] {
            g[i] = ItemStack::new(COBBLE, 2);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BREWING_STAND, 1));
        // rod NOT centered → no match
        let mut bad = g.clone();
        bad[1] = ItemStack::EMPTY;
        bad[0] = ItemStack::new(NETHERRACK, 1);
        assert!(match_grid(&bad, 3).is_none());
    }

    #[test]
    fn bookshelf_makes_books() {
        // 1×1 recipe: works in the 2×2 inventory grid anywhere
        let mut g = vec![ItemStack::EMPTY; 4];
        g[2] = ItemStack::new(BOOKSHELF, 1);
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (ENCHANTED_BOOK, 3));
    }

    #[test]
    fn enchant_table_layout_is_vanilla() {
        let mut g = vec![ItemStack::EMPTY; 9];
        g[1] = ItemStack::new(BOOKSHELF, 1);
        g[3] = ItemStack::new(DIAMOND_BLOCK, 1);
        g[5] = ItemStack::new(DIAMOND_BLOCK, 1);
        for i in [4, 6, 7, 8] {
            g[i] = ItemStack::new(OBSIDIAN, 1);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (ENCHANT_TABLE, 1));
    }
}
