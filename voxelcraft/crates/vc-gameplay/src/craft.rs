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
    Block(u16),
}

/// the recipes our registry supports (vanilla-shaped)
pub const RECIPES: &[Recipe] = &[
    // log → 4 planks (shapeless in vanilla; modeled as 1×1 shaped)
    Recipe {
        size: 1,
        grid: &[Ing::AnyLog],
        out: ItemStack::new(PLANKS, 4),
    },
    // ---- audit-fix (1.2): jungle log → 4 JUNGLE planks (the universal
    // log→planks rule; the jungle family has its own planks block) ----
    Recipe {
        size: 1,
        grid: &[Ing::Block(JUNGLE_LOG)],
        out: ItemStack::new(JUNGLE_PLANKS, 4),
    },
    // ---- 1.11 (VERIFIED changelog §Blocks: shulker boxes "Crafted in a
    // crafting table as a single column, with a chest in the middle of
    // the row and a shulker shell both above and below the chest") ----
    // the engine's square-pattern matcher places 3×3 patterns at the
    // top-left, so the column lives in the MIDDLE column (vanilla also
    // accepts the side columns — a disclosed placement constraint)
    Recipe {
        size: 3,
        grid: &[
            Ing::None, Ing::Block(SHULKER_SHELL), Ing::None,
            Ing::None, Ing::Block(CHEST), Ing::None,
            Ing::None, Ing::Block(SHULKER_SHELL), Ing::None,
        ],
        out: ItemStack::new(SHULKER_BOX, 1),
    },
    // ---- 1.8 bracket (VERIFIED minecraft.wiki/w/Java_Edition_1.8
    // §Blocks, live 2026-09-06) ----
    // "Polished variants of Diorite, Andesite & Granite — crafting recipe:
    // 4 pieces of one of the materials, in a 2×2 configuration"
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(GRANITE),
            Ing::Block(GRANITE),
            Ing::Block(GRANITE),
            Ing::Block(GRANITE),
        ],
        out: ItemStack::new(POLISHED_GRANITE, 4),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(DIORITE),
            Ing::Block(DIORITE),
            Ing::Block(DIORITE),
            Ing::Block(DIORITE),
        ],
        out: ItemStack::new(POLISHED_DIORITE, 4),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(ANDESITE),
            Ing::Block(ANDESITE),
            Ing::Block(ANDESITE),
            Ing::Block(ANDESITE),
        ],
        out: ItemStack::new(POLISHED_ANDESITE, 4),
    },
    // "Coarse Dirt — crafting recipe: dirt and gravel in a 2×2 checkered
    // pattern yields four coarse dirt"
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(DIRT),
            Ing::Block(GRAVEL),
            Ing::Block(GRAVEL),
            Ing::Block(DIRT),
        ],
        out: ItemStack::new(COARSE_DIRT, 4),
    },
    // red sandstone: 4 red sand 2×2 (vanilla red-sandstone recipe)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(RED_SAND),
            Ing::Block(RED_SAND),
            Ing::Block(RED_SAND),
            Ing::Block(RED_SAND),
        ],
        out: ItemStack::new(RED_SANDSTONE, 1),
    },
    // prismarine family (wiki §Blocks: prismarine = shards, bricks =
    // shards, dark = shards + ink; ink sacs are palette-absent — the dark
    // variant rides the same shard recipe, documented simplification)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(PRISMARINE_SHARD),
            Ing::Block(PRISMARINE_SHARD),
            Ing::Block(PRISMARINE_SHARD),
            Ing::Block(PRISMARINE_SHARD),
        ],
        out: ItemStack::new(PRISMARINE, 1),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(PRISMARINE_CRYSTALS),
            Ing::Block(PRISMARINE_CRYSTALS),
            Ing::Block(PRISMARINE_CRYSTALS),
            Ing::Block(PRISMARINE_CRYSTALS),
        ],
        out: ItemStack::new(SEA_LANTERN, 1),
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
    // ---- Phase E3 (evolution 1.5-1.6 bracket; all live-verified
    // 2026-09-06) ----
    // block of coal: 9 coal -> 1 (VERIFIED w/Block_of_Coal)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
            Ing::Block(COAL),
        ],
        out: ItemStack::new(COAL_BLOCK, 1),
    },
    // block of coal -> 9 coal (the vanilla reverse craft, w/Block_of_Coal)
    Recipe {
        size: 1,
        grid: &[Ing::Block(COAL_BLOCK)],
        out: ItemStack::new(COAL, 9),
    },
    // block of quartz: 4 nether quartz (VERIFIED w/Block_of_Quartz)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(NETHER_QUARTZ),
        ],
        out: ItemStack::new(QUARTZ_BLOCK, 1),
    },
    // quartz pillar: 2 blocks of quartz (vertical) -> 2 pillars
    // (VERIFIED w/Quartz_Pillar "Block of Quartz 2"; output count 2
    // confirmed by a second live source; modeled as a 2×2 left column —
    // the engine's w×w grid convention)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(QUARTZ_BLOCK),
            Ing::None,
            Ing::Block(QUARTZ_BLOCK),
            Ing::None,
        ],
        out: ItemStack::new(QUARTZ_PILLAR, 2),
    },
    // carpets: 2 wool (vertical) -> 3 (VERIFIED w/Carpet 13w17a "now
    // returns 3 carpets from two wool") — one recipe per engine wool
    // color; modeled as a 2×2 left column (the w×w grid convention)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(WOOL_WHITE),
            Ing::None,
            Ing::Block(WOOL_WHITE),
            Ing::None,
        ],
        out: ItemStack::new(CARPET_WHITE, 3),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(WOOL_RED),
            Ing::None,
            Ing::Block(WOOL_RED),
            Ing::None,
        ],
        out: ItemStack::new(CARPET_RED, 3),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(WOOL_YELLOW),
            Ing::None,
            Ing::Block(WOOL_YELLOW),
            Ing::None,
        ],
        out: ItemStack::new(CARPET_YELLOW, 3),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(WOOL_BLUE),
            Ing::None,
            Ing::Block(WOOL_BLUE),
            Ing::None,
        ],
        out: ItemStack::new(CARPET_BLUE, 3),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(WOOL_BLACK),
            Ing::None,
            Ing::Block(WOOL_BLACK),
            Ing::None,
        ],
        out: ItemStack::new(CARPET_BLACK, 3),
    },
    // trapped chest: 1 tripwire hook + 1 chest (VERIFIED w/Trapped_Chest;
    // the 2-ingredient shapeless craft modeled as a 2×2 column pair)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(TRIPWIRE_HOOK),
            Ing::None,
            Ing::Block(CHEST),
            Ing::None,
        ],
        out: ItemStack::new(TRAPPED_CHEST, 1),
    },
    // daylight sensor: 3 glass + 3 quartz + 3 wooden slabs (VERIFIED
    // w/Daylight_Detector "Glass + Nether Quartz + Any Wooden Slab")
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(GLASS),
            Ing::Block(GLASS),
            Ing::Block(GLASS),
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(NETHER_QUARTZ),
            Ing::Block(OAK_SLAB),
            Ing::Block(OAK_SLAB),
            Ing::Block(OAK_SLAB),
        ],
        out: ItemStack::new(DAYLIGHT_SENSOR, 1),
    },
    // light weighted pressure plate: 2 gold (VERIFIED w/
    // Light_Weighted_Pressure_Plate; gold ore — no ingots, disclosed;
    // 2×2 column model)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(GOLD_ORE),
            Ing::None,
            Ing::Block(GOLD_ORE),
            Ing::None,
        ],
        out: ItemStack::new(LIGHT_WEIGHTED_PLATE, 1),
    },
    // heavy weighted pressure plate: 2 iron (VERIFIED w/
    // Heavy_Weighted_Pressure_Plate; iron ore — no ingots, disclosed;
    // 2×2 column model)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(IRON_ORE),
            Ing::None,
            Ing::Block(IRON_ORE),
            Ing::None,
        ],
        out: ItemStack::new(HEAVY_WEIGHTED_PLATE, 1),
    },
    // block of redstone: 9 redstone (VERIFIED w/Block_of_Redstone "nine
    // redstone dust"; redstone WIRE block = the engine's dust — disclosed)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
            Ing::Block(REDSTONE_WIRE),
        ],
        out: ItemStack::new(REDSTONE_BLOCK, 1),
    },
    // block of redstone -> 9 redstone (the vanilla reverse craft)
    Recipe {
        size: 1,
        grid: &[Ing::Block(REDSTONE_BLOCK)],
        out: ItemStack::new(REDSTONE_WIRE, 9),
    },
    // ---- 1.13 (Update Aquatic, VERIFIED changelog §Items live
    // 2026-09-07) ----
    // turtle shell: 5 scutes in the helmet shape (w/Turtle_Shell
    // §Crafting: "Scutes x5" arranged like a helmet)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(SCUTE), Ing::Block(SCUTE), Ing::Block(SCUTE),
            Ing::Block(SCUTE), Ing::None,   Ing::Block(SCUTE),
            Ing::None,       Ing::None,     Ing::None,
        ],
        out: ItemStack::new(TURTLE_SHELL, 1),
    },
    // dried kelp block: 9 dried kelp (w/Dried_Kelp_Block §Crafting)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP),
            Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP),
            Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP), Ing::Block(DRIED_KELP),
        ],
        out: ItemStack::new(DRIED_KELP_BLOCK, 1),
    },
    // dried kelp block -> 9 dried kelp (the vanilla reverse craft —
    // changelog: "can also be crafted back into dried kelp")
    Recipe {
        size: 1,
        grid: &[Ing::Block(DRIED_KELP_BLOCK)],
        out: ItemStack::new(DRIED_KELP, 9),
    },
    // conduit: heart of the sea + 8 nautilus shells (the ring —
    // changelog: "Crafted using 1 heart of the sea and 8 nautilus
    // shells")
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(NAUTILUS_SHELL), Ing::Block(NAUTILUS_SHELL), Ing::Block(NAUTILUS_SHELL),
            Ing::Block(NAUTILUS_SHELL), Ing::Block(HEART_OF_THE_SEA), Ing::Block(NAUTILUS_SHELL),
            Ing::Block(NAUTILUS_SHELL), Ing::Block(NAUTILUS_SHELL), Ing::Block(NAUTILUS_SHELL),
        ],
        out: ItemStack::new(CONDUIT, 1),
    },
    // blue ice: 9 packed ice (changelog: "Crafted using 9 packed ice")
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE),
            Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE),
            Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE), Ing::Block(PACKED_ICE),
        ],
        out: ItemStack::new(BLUE_ICE, 1),
    },
];

/// 1.12 (World of Color): the concrete-powder recipe — the engine's
/// first truly SHAPELESS 9-slot craft. VERIFIED changelog §Blocks:
/// "Craftable using 4 sand, 4 gravel and one of any dye to get 8
/// concrete powder blocks. The recipe is shapeless." + w/Concrete_
/// Powder §Crafting: "The crafting recipe is shapeless; the order of
/// ingredients does not matter." The output color follows the dye's
/// color index (w/Concrete_Powder: any of the 16 dye colors).
fn match_concrete_powder(slots: &[ItemStack], size: usize) -> Option<ItemStack> {
    if size < 3 {
        return None; // the 9-ingredient recipe needs the 3×3 table
    }
    let mut sand = 0;
    let mut gravel = 0;
    let mut dye_color: Option<u8> = None;
    for s in slots {
        if s.is_empty() {
            continue;
        }
        match s.block {
            SAND => sand += 1,
            GRAVEL => gravel += 1,
            b if (DYE_BASE..=DYE_END).contains(&b) => {
                if dye_color.is_some() {
                    return None; // more than one dye — not the recipe
                }
                dye_color = Some(vc_blocks::blocks::dye_color(b));
            }
            _ => return None, // any other ingredient breaks the multiset
        }
    }
    if sand == 4 && gravel == 4 {
        let c = dye_color?; // exactly one dye, 4 sand, 4 gravel
        Some(ItemStack::new(concrete_powder(c), 8))
    } else {
        None
    }
}

/// match a crafting grid (row-major, `size`×`size` of ItemStacks) → the
/// recipe output. Trims to the bounding box first (vanilla grid-shape
/// semantics: the pattern matches anywhere in the grid).
pub fn match_grid(slots: &[ItemStack], size: usize) -> Option<ItemStack> {
    // 1.12: the shapeless concrete-powder recipe (any arrangement)
    if let Some(out) = match_concrete_powder(slots, size) {
        return Some(out);
    }
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

    // ---------------- Phase E3 tests (1.5–1.6 bracket) ----------------

    fn grid2(items: [u16; 4]) -> [ItemStack; 4] {
        let mut g = [ItemStack::EMPTY; 4];
        for (i, &b) in items.iter().enumerate() {
            if b != 0 {
                g[i] = ItemStack::new(b, 1);
            }
        }
        g
    }

    #[test]
    fn phase_e3_coal_block_recipes() {
        // 9 coal -> 1 block (VERIFIED w/Block_of_Coal)
        let mut g = [ItemStack::new(COAL, 1); 9];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (COAL_BLOCK, 1));
        // 1 block -> 9 coal (the vanilla reverse craft)
        let g2 = [ItemStack::new(COAL_BLOCK, 1), ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY];
        let out2 = match_grid(&g2, 2).unwrap();
        assert_eq!((out2.block, out2.count), (COAL, 9));
    }

    #[test]
    fn phase_e3_quartz_and_pillar_recipes() {
        // 4 nether quartz -> 1 block of quartz (VERIFIED w/Block_of_Quartz)
        let out = match_grid(&grid2([NETHER_QUARTZ, NETHER_QUARTZ, NETHER_QUARTZ, NETHER_QUARTZ]), 2).unwrap();
        assert_eq!((out.block, out.count), (QUARTZ_BLOCK, 1));
        // 2 blocks of quartz (vertical) -> 2 pillars (VERIFIED
        // w/Quartz_Pillar; output count 2 confirmed by a 2nd source)
        let out2 = match_grid(&grid2([QUARTZ_BLOCK, 0, QUARTZ_BLOCK, 0]), 2).unwrap();
        assert_eq!((out2.block, out2.count), (QUARTZ_PILLAR, 2));
    }

    #[test]
    fn phase_e3_carpet_recipes() {
        // 2 wool (vertical) -> 3 carpets (VERIFIED w/Carpet 13w17a)
        for (wool, carpet) in [
            (WOOL_WHITE, CARPET_WHITE),
            (WOOL_RED, CARPET_RED),
            (WOOL_YELLOW, CARPET_YELLOW),
            (WOOL_BLUE, CARPET_BLUE),
            (WOOL_BLACK, CARPET_BLACK),
        ] {
            let out = match_grid(&grid2([wool, 0, wool, 0]), 2).unwrap();
            assert_eq!((out.block, out.count), (carpet, 3), "wool {wool}");
        }
    }

    #[test]
    fn phase_e3_redstone_component_recipes() {
        // trapped chest: tripwire hook + chest (VERIFIED w/Trapped_Chest)
        let out = match_grid(&grid2([TRIPWIRE_HOOK, 0, CHEST, 0]), 2).unwrap();
        assert_eq!((out.block, out.count), (TRAPPED_CHEST, 1));
        // daylight sensor: glass + quartz + slabs (VERIFIED
        // w/Daylight_Detector)
        let mut g = [ItemStack::EMPTY; 9];
        for i in 0..3 {
            g[i] = ItemStack::new(GLASS, 1);
            g[3 + i] = ItemStack::new(NETHER_QUARTZ, 1);
            g[6 + i] = ItemStack::new(OAK_SLAB, 1);
        }
        let out2 = match_grid(&g, 3).unwrap();
        assert_eq!((out2.block, out2.count), (DAYLIGHT_SENSOR, 1));
        // plates: 2 ore blocks (the no-ingot convention, disclosed)
        let out3 = match_grid(&grid2([GOLD_ORE, 0, GOLD_ORE, 0]), 2).unwrap();
        assert_eq!((out3.block, out3.count), (LIGHT_WEIGHTED_PLATE, 1));
        let out4 = match_grid(&grid2([IRON_ORE, 0, IRON_ORE, 0]), 2).unwrap();
        assert_eq!((out4.block, out4.count), (HEAVY_WEIGHTED_PLATE, 1));
        // block of redstone: 9 wire (the engine's dust-as-block row) + back
        let mut g9 = [ItemStack::new(REDSTONE_WIRE, 1); 9];
        let out5 = match_grid(&g9, 3).unwrap();
        assert_eq!((out5.block, out5.count), (REDSTONE_BLOCK, 1));
        g9[0] = ItemStack::new(REDSTONE_BLOCK, 1);
        g9[1..].fill(ItemStack::EMPTY);
        let out6 = match_grid(&g9, 3).unwrap();
        assert_eq!((out6.block, out6.count), (REDSTONE_WIRE, 9));
    }
}
// ---------------- audit-fix round (2026-09-07): 1.2 jungle planks ----------------
#[cfg(test)]
mod auditfix_tests {
    use super::*;

    /// jungle log crafts into 4 JUNGLE planks (the universal
    /// log→planks rule; VERIFIED family behavior per w/Log)
    #[test]
    fn jungle_log_crafts_jungle_planks() {
        let grid = [ItemStack::new(JUNGLE_LOG, 1); 1];
        let out = match_grid(&grid, 1).expect("jungle log matches a recipe");
        assert_eq!(out.block, JUNGLE_PLANKS);
        assert_eq!(out.count, 4);
        // the generic AnyLog recipe still serves oak/birch/spruce
        let oak = [ItemStack::new(OAK_LOG, 1); 1];
        let out_oak = match_grid(&oak, 1).expect("oak still matches");
        assert_eq!(out_oak.block, PLANKS);
        assert_eq!(out_oak.count, 4);
    }
}

#[cfg(test)]
mod v112_tests {
    use super::*;

    /// 1.12 (World of Color): the shapeless concrete-powder recipe —
    /// 4 sand + 4 gravel + 1 dye → 8 powder of the dye's color, ANY
    /// arrangement (VERIFIED changelog §Blocks + w/Concrete_Powder
    /// §Crafting: "The crafting recipe is shapeless; the order of
    /// ingredients does not matter")
    #[test]
    fn v112_concrete_powder_shapeless_recipe() {
        // the canonical arrangement
        let g = [
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(DYE_BASE + 11, 1), // Lapis Lazuli → blue
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!(out.block, concrete_powder(11));
        assert_eq!(out.count, 8, "8 powder per craft (VERIFIED)");

        // a scrambled arrangement (the shapeless property)
        let g2 = [
            ItemStack::new(GRAVEL, 1), ItemStack::new(DYE_BASE, 1),
            ItemStack::new(SAND, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(SAND, 1),
            ItemStack::new(GRAVEL, 1),
        ];
        let out2 = match_grid(&g2, 3).unwrap();
        assert_eq!(out2.block, concrete_powder(0), "Bone Meal → white powder");
        assert_eq!(out2.count, 8);

        // every dye color routes to its powder color
        for c in 0u16..16 {
            let mut g3 = [ItemStack::EMPTY; 9];
            for i in 0..4 {
                g3[i] = ItemStack::new(SAND, 1);
                g3[4 + i] = ItemStack::new(GRAVEL, 1);
            }
            g3[8] = ItemStack::new(DYE_BASE + c, 1);
            let out3 = match_grid(&g3, 3).unwrap();
            assert_eq!(out3.block, concrete_powder(c as u8), "dye {c}");
        }
    }

    #[test]
    fn v112_powder_recipe_rejects_wrong_counts() {
        // 3 sand + 5 gravel: not the recipe
        let g = [
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(SAND, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(DYE_BASE, 1),
        ];
        assert!(match_grid(&g, 3).is_none(), "wrong sand count");
        // two dyes: not the recipe
        let g2 = [
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(SAND, 1), ItemStack::new(SAND, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(GRAVEL, 1), ItemStack::new(GRAVEL, 1),
            ItemStack::new(DYE_BASE, 1),
        ];
        let mut g3 = g2;
        g3[0] = ItemStack::new(DYE_BASE + 1, 1);
        g3[1] = ItemStack::new(DYE_BASE + 1, 1);
        g3[2] = ItemStack::new(SAND, 1);
        g3[3] = ItemStack::new(SAND, 1);
        // 3 sand + 1 dye swapped in — now sand is 3? recompute: slots
        // [dye, dye, sand, sand, gravel×4, dye] — three dyes total
        assert!(match_grid(&g3, 3).is_none(), "multiple dyes rejected");
        // a foreign ingredient (cobble) breaks it
        let mut g4 = g2;
        g4[0] = ItemStack::new(COBBLE, 1);
        g4[1] = ItemStack::new(SAND, 1);
        assert!(match_grid(&g4, 3).is_none(), "foreign ingredient rejected");
        // the 2×2 inventory grid can't fit 9 ingredients
        let small = [ItemStack::new(SAND, 1); 4];
        assert!(match_grid(&small, 2).is_none());
    }

    /// 1.13 (Update Aquatic) recipes — VERIFIED changelog §Items (live
    /// 2026-09-07): turtle shell (5 scutes, helmet shape), dried kelp
    /// block (9 dried kelp) + the reverse, the conduit ring (8 shells
    /// + heart of the sea), blue ice (9 packed ice).
    #[test]
    fn v113_aquatic_recipes() {
        // turtle shell: the helmet-shaped 5-scute pattern
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 1, 2, 3, 5] {
            g[i] = ItemStack::new(SCUTE, 2);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!(out.block, TURTLE_SHELL);
        assert_eq!(out.count, 1);
        // 4 scutes (missing a side) is not the helmet
        let mut g2 = g.clone();
        g2[5] = ItemStack::EMPTY;
        assert!(match_grid(&g2, 3).is_none(), "4 scutes do not craft the shell");
        // dried kelp block: 9 dried kelp
        let g3 = vec![ItemStack::new(DRIED_KELP, 1); 9];
        let out3 = match_grid(&g3, 3).unwrap();
        assert_eq!(out3.block, DRIED_KELP_BLOCK);
        // the reverse: one block -> 9 dried kelp
        let g4 = vec![ItemStack::new(DRIED_KELP_BLOCK, 1)];
        let out4 = match_grid(&g4, 1).unwrap();
        assert_eq!(out4.block, DRIED_KELP);
        assert_eq!(out4.count, 9);
        // the conduit: 8 nautilus shells + heart of the sea (ring)
        let mut g5 = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 1, 2, 3, 5, 6, 7, 8] {
            g5[i] = ItemStack::new(NAUTILUS_SHELL, 1);
        }
        g5[4] = ItemStack::new(HEART_OF_THE_SEA, 1);
        let out5 = match_grid(&g5, 3).unwrap();
        assert_eq!(out5.block, CONDUIT);
        // 8 shells + cobble center: not the conduit
        let mut g6 = g5.clone();
        g6[4] = ItemStack::new(COBBLE, 1);
        assert!(match_grid(&g6, 3).is_none(), "the heart of the sea is required");
        // blue ice: 9 packed ice
        let g7 = vec![ItemStack::new(PACKED_ICE, 1); 9];
        let out7 = match_grid(&g7, 3).unwrap();
        assert_eq!(out7.block, BLUE_ICE);
    }
}
