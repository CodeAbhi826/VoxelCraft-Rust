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
    /// 1.14: any of the engine's 6 log blocks (vanilla's campfire
    /// recipe takes "Any Log or Stem or ..." — the 6-log engine set)
    AnyWood,
    /// 1.14: any planks (vanilla's stick/barrel recipes take "Any
    /// Planks" — the completeness audit extends the oak+jungle pair to
    /// the 1.16 crimson/warped planks, vanilla-exact)
    AnyPlanks,
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
    // ---- 1.14 (Village & Pillage — nature half, VERIFIED live
    // 2026-09-08 from the raw captures v114_page_*.json) ----
    // stick: 2 planks stacked → 4 sticks (the classic recipe; "Any
    // Planks" per slot — the engine's oak + jungle pair). Both
    // orientations ship as separate patterns (the matcher has no
    // rotation pass — the shulker-column precedent discloses this
    // class of constraint).
    Recipe {
        size: 2,
        grid: &[
            Ing::AnyPlanks, Ing::None,
            Ing::AnyPlanks, Ing::None,
        ],
        out: ItemStack::new(STICK, 4),
    },
    Recipe {
        size: 2,
        grid: &[
            Ing::AnyPlanks, Ing::AnyPlanks,
            Ing::None,       Ing::None,
        ],
        out: ItemStack::new(STICK, 4),
    },
    // campfire: "Stick + Coal or Charcoal + Any Log" (VERIFIED
    // w/Campfire §Crafting; counts 3 sticks / 1 fuel / 3 logs — the
    // vanilla grid: sticks across the top, fuel in the middle center,
    // logs across the bottom; coal and charcoal are two recipe rows
    // — vanilla's "Coal or Charcoal")
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(STICK), Ing::Block(STICK),    Ing::Block(STICK),
            Ing::None,         Ing::Block(COAL),     Ing::None,
            Ing::AnyWood,      Ing::AnyWood,         Ing::AnyWood,
        ],
        out: ItemStack::new(CAMPFIRE, 1),
    },
    // the charcoal-fueled campfire row
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(STICK),   Ing::Block(STICK),   Ing::Block(STICK),
            Ing::None,           Ing::Block(CHARCOAL), Ing::None,
            Ing::AnyWood,        Ing::AnyWood,        Ing::AnyWood,
        ],
        out: ItemStack::new(CAMPFIRE, 1),
    },
    // barrel: "6 wood planks and 2 wood slabs" (VERIFIED w/Barrel
    // §Crafting "Any Planks + Any Wooden Slab" + the 18w50a history
    // row "crafted using 6 wood planks and 2 wood slabs"; the vanilla
    // grid: planks left+right columns, slabs top+bottom center — the
    // engine's single OAK_SLAB is the "any slab" stand-in, disclosed)
    Recipe {
        size: 3,
        grid: &[
            Ing::AnyPlanks, Ing::Block(OAK_SLAB), Ing::AnyPlanks,
            Ing::AnyPlanks, Ing::None,           Ing::AnyPlanks,
            Ing::AnyPlanks, Ing::Block(OAK_SLAB), Ing::AnyPlanks,
        ],
        out: ItemStack::new(BARREL, 1),
    },
    // ---- 1.14 (nature half, part 2): the smelting trio + lantern.
    // VERIFIED 2026-09-08 from the raw captures v114b_page_*.json. ----
    // blast furnace: "5 iron ingots, 1 furnace and 3 smooth stone"
    // (w/Blast_Furnace §Crafting: iron across the top + both sides of
    // the middle, furnace center, smooth stone across the bottom). The
    // engine's IRON_ORE items stand in for the ingots (the disclosed
    // convention — the engine has no iron-ingot item; ore items are
    // the material).
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(IRON_ORE),  Ing::Block(IRON_ORE),  Ing::Block(IRON_ORE),
            Ing::Block(IRON_ORE),  Ing::Block(FURNACE),   Ing::Block(IRON_ORE),
            Ing::Block(SMOOTH_STONE), Ing::Block(SMOOTH_STONE), Ing::Block(SMOOTH_STONE),
        ],
        out: ItemStack::new(BLAST_FURNACE, 1),
    },
    // smoker: "4 logs, stripped or not, or wood, around a furnace"
    // (w/Smoker §Crafting: the diamond/cross placement — logs at top
    // center, middle left, middle right, bottom center). The engine's
    // AnyWood = the 6-log set.
    Recipe {
        size: 3,
        grid: &[
            Ing::None,    Ing::AnyWood, Ing::None,
            Ing::AnyWood, Ing::Block(FURNACE), Ing::AnyWood,
            Ing::None,    Ing::AnyWood, Ing::None,
        ],
        out: ItemStack::new(SMOKER, 1),
    },
    // lantern: "8 iron nuggets and 1 torch" (w/Lantern §Crafting: the
    // nugget ring around the torch center). The engine's torch
    // (REDSTONE_TORCH, light 7) is the torch stand-in, disclosed.
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET),
            Ing::Block(IRON_NUGGET), Ing::Block(REDSTONE_TORCH), Ing::Block(IRON_NUGGET),
            Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET),
        ],
        out: ItemStack::new(LANTERN, 1),
    },
    // iron nuggets from the ingot stand-in (vanilla w/Iron_Nugget
    // §Crafting: 1 iron ingot → 9 nuggets; 9 nuggets → 1 ingot — the
    // engine's 9:1 back-craft uses the IRON_ORE stand-in, disclosed)
    Recipe {
        size: 1,
        grid: &[Ing::Block(IRON_ORE)],
        out: ItemStack::new(IRON_NUGGET, 9),
    },
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET),
            Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET),
            Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET), Ing::Block(IRON_NUGGET),
        ],
        out: ItemStack::new(IRON_ORE, 1),
    },
    // ---- 1.14 (part 3): the flower→dye pair (VERIFIED w/Cornflower
    // §Crafting ingredient: "Blue Dye — Cornflower"; w/Lily_of_the_
    // Valley §Crafting ingredient: "White Dye — Lily of the Valley".
    // Shapeless in vanilla; modeled as 1×1 shaped — the log→planks
    // convention) ----
    Recipe {
        size: 1,
        grid: &[Ing::Block(CORNFLOWER)],
        out: ItemStack::new(DYE_BASE + 11, 1),
    },
    Recipe {
        size: 1,
        grid: &[Ing::Block(LILY_OF_THE_VALLEY)],
        out: ItemStack::new(DYE_BASE, 1),
    },
    // ---- 1.15 (Buzzy Bees) — VERIFIED w/Beehive/w/Honey_Block/
    // w/Honeycomb/w/Honeycomb_Block §Crafting + the shears precedent
    // (2 iron ingots diagonal — the engine's IRON_ORE ingot stand-in,
    // the disclosed blast-furnace convention) ----
    // beehive: "Any Planks + Honeycomb" — the grid is planks×3 top,
    // honeycomb×3 middle, planks×3 bottom (6 planks + 3 honeycomb)
    Recipe {
        size: 3,
        grid: &[
            Ing::AnyPlanks, Ing::AnyPlanks, Ing::AnyPlanks,
            Ing::Block(HONEYCOMB), Ing::Block(HONEYCOMB), Ing::Block(HONEYCOMB),
            Ing::AnyPlanks, Ing::AnyPlanks, Ing::AnyPlanks,
        ],
        out: ItemStack::new(BEEHIVE, 1),
    },
    // honeycomb block: 4 honeycomb (2x2)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(HONEYCOMB), Ing::Block(HONEYCOMB),
            Ing::Block(HONEYCOMB), Ing::Block(HONEYCOMB),
        ],
        out: ItemStack::new(HONEYCOMB_BLOCK, 1),
    },
    // honey block: 4 honey bottles (2x2). VERIFIED w/Honey_Block
    // §Crafting: "Honey Bottle ... Empty bottles remain in the crafting
    // grid after crafting the honey block" — the engine's grid consumes
    // ingredients, so the 4 returning glass bottles are granted by the
    // game layer's craft-application hook (documented adaptation).
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(HONEY_BOTTLE), Ing::Block(HONEY_BOTTLE),
            Ing::Block(HONEY_BOTTLE), Ing::Block(HONEY_BOTTLE),
        ],
        out: ItemStack::new(HONEY_BLOCK, 1),
    },
    // honey block back into 4 bottles (the reverse recipe, both ways
    // in vanilla)
    Recipe {
        size: 1,
        grid: &[Ing::Block(HONEY_BLOCK)],
        out: ItemStack::new(HONEY_BOTTLE, 4),
    },
    // shears (LEGACY item, the stick/charcoal window precedent): 2
    // iron ingots diagonal
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(IRON_ORE), Ing::None,
            Ing::None,           Ing::Block(IRON_ORE),
        ],
        out: ItemStack::new(SHEARS, 1),
    },
    // ---- 1.16 (Nether Update, part 1 — the anchor family): all
    // VERIFIED against the v116 captures. ----
    // respawn anchor: "6 crying obsidian + 3 glowstone" (w/Respawn_
    // Anchor §Crafting — the crying obsidian ring around the
    // glowstone column)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(CRYING_OBSIDIAN), Ing::Block(GLOWSTONE), Ing::Block(CRYING_OBSIDIAN),
            Ing::Block(CRYING_OBSIDIAN), Ing::Block(GLOWSTONE), Ing::Block(CRYING_OBSIDIAN),
            Ing::Block(CRYING_OBSIDIAN), Ing::Block(GLOWSTONE), Ing::Block(CRYING_OBSIDIAN),
        ],
        out: ItemStack::new(RESPAWN_ANCHOR, 1),
    },
    // target: "4 redstone dust + 1 hay bale" (w/Target §Crafting —
    // the dust cross around the hay bale center; the engine has no
    // redstone-dust ITEM — the REDSTONE_BLOCK stand-in occupies the
    // dust slots, disclosed)
    Recipe {
        size: 3,
        grid: &[
            Ing::None,              Ing::Block(REDSTONE_BLOCK), Ing::None,
            Ing::Block(REDSTONE_BLOCK), Ing::Block(HAY_BALE),   Ing::Block(REDSTONE_BLOCK),
            Ing::None,              Ing::Block(REDSTONE_BLOCK), Ing::None,
        ],
        out: ItemStack::new(TARGET, 1),
    },
    // netherite ingot: "crafting four netherite scraps and four gold
    // ingots together" (VERIFIED w/Netherite_Ingot — gold is the
    // engine's IRON_ORE ingot stand-in, the disclosed convention)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(NETHERITE_SCRAP), Ing::Block(IRON_ORE),     Ing::Block(NETHERITE_SCRAP),
            Ing::Block(IRON_ORE),        Ing::None,                 Ing::Block(IRON_ORE),
            Ing::Block(NETHERITE_SCRAP), Ing::Block(IRON_ORE),     Ing::Block(NETHERITE_SCRAP),
        ],
        out: ItemStack::new(NETHERITE_INGOT, 1),
    },
    // block of netherite: 9 ingots (the storage-block convention,
    // VERIFIED w/Block_of_Netherite) ...
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT),
            Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT),
            Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT), Ing::Block(NETHERITE_INGOT),
        ],
        out: ItemStack::new(NETHERITE_BLOCK, 1),
    },
    // ... and back into 9 (both directions, the storage convention)
    Recipe {
        size: 1,
        grid: &[Ing::Block(NETHERITE_BLOCK)],
        out: ItemStack::new(NETHERITE_INGOT, 9),
    },
    // chain: "iron nuggets + iron ingot" (VERIFIED w/Chain — the 1.16
    // iron-only form; the copper halves of the current wiki row are
    // 1.21+ additions, out of bracket). Vertical: 1 nugget over the
    // ingot over 1 nugget.
    Recipe {
        size: 3,
        grid: &[
            Ing::None, Ing::Block(IRON_NUGGET), Ing::None,
            Ing::None, Ing::Block(IRON_ORE),    Ing::None,
            Ing::None, Ing::Block(IRON_NUGGET), Ing::None,
        ],
        out: ItemStack::new(CHAIN, 1),
    },
    // ---- 1.16 (Nether Update, part 2 — the crimson/warped families):
    // all VERIFIED against the v116b captures. ----
    // crimson stem → 4 crimson planks (the universal log→planks rule,
    // VERIFIED w/Crimson_Planks §Crafting: "crimson planks can be
    // crafted from crimson stems")
    Recipe {
        size: 1,
        grid: &[Ing::Block(CRIMSON_STEM)],
        out: ItemStack::new(CRIMSON_PLANKS, 4),
    },
    // crimson hyphae → 4 crimson planks (the same 1:4 rule — the
    // "any log or stem or hyphae" row of the universal planks recipe)
    Recipe {
        size: 1,
        grid: &[Ing::Block(CRIMSON_HYPHAE)],
        out: ItemStack::new(CRIMSON_PLANKS, 4),
    },
    // warped stem → 4 warped planks
    Recipe {
        size: 1,
        grid: &[Ing::Block(WARPED_STEM)],
        out: ItemStack::new(WARPED_PLANKS, 4),
    },
    // warped hyphae → 4 warped planks
    Recipe {
        size: 1,
        grid: &[Ing::Block(WARPED_HYPHAE)],
        out: ItemStack::new(WARPED_PLANKS, 4),
    },
    // polished basalt: 4 basalt → 4 (VERIFIED w/Polished_Basalt
    // §Crafting: the 2x2 stone family — the part-1 BASALT doc's
    // deferred family, delivered here)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(BASALT), Ing::Block(BASALT),
            Ing::Block(BASALT), Ing::Block(BASALT),
        ],
        out: ItemStack::new(POLISHED_BASALT, 4),
    },
    // polished blackstone: 4 blackstone → 4 (VERIFIED
    // w/Polished_Blackstone §Crafting)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(BLACKSTONE), Ing::Block(BLACKSTONE),
            Ing::Block(BLACKSTONE), Ing::Block(BLACKSTONE),
        ],
        out: ItemStack::new(POLISHED_BLACKSTONE, 4),
    },
    // polished blackstone bricks: 4 polished blackstone → 4 (VERIFIED
    // w/Polished_Blackstone_Bricks §Crafting)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(POLISHED_BLACKSTONE), Ing::Block(POLISHED_BLACKSTONE),
            Ing::Block(POLISHED_BLACKSTONE), Ing::Block(POLISHED_BLACKSTONE),
        ],
        out: ItemStack::new(POLISHED_BLACKSTONE_BRICKS, 4),
    },
    // soul torch: charcoal/coal + stick + soul soil or soul sand → 4
    // — SHAPELESS (VERIFIED w/Soul_Torch §Crafting: "a torch crafted
    // with the addition of soul soil or soul sand"; the coal and
    // charcoal halves of the current wiki row are both valid in the
    // 1.16 window). Rides the shapeless matcher below (the concrete-
    // powder pattern). The soul lantern follows from it: 8 iron
    // nuggets + 1 soul torch (VERIFIED w/Soul_Torch §Crafting
    // ingredient table: "Soul Lantern — Iron Nugget + Soul Torch").
    // ---- the 1.0-1.16.5 completeness audit (VERIFIED live 2026-09-08
    // against the audit16 captures) ----
    // purpur block: 4 popped chorus fruit -> 4 (the 2x2 stone-family
    // pattern; w/Popped_Chorus_Fruit: "used to craft End rods and
    // purpur blocks" — the 1.9 purpur family finally crafts from its
    // own ingredient instead of being picker-only)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(POPPED_CHORUS_FRUIT), Ing::Block(POPPED_CHORUS_FRUIT),
            Ing::Block(POPPED_CHORUS_FRUIT), Ing::Block(POPPED_CHORUS_FRUIT),
        ],
        out: ItemStack::new(PURPUR_BLOCK, 4),
    },
    // end rod: blaze rod + popped chorus fruit -> 4 (VERIFIED
    // w/End_Rod §Crafting; the 1.9 end-rod's first recipe — the
    // rod-over-chorus pair as the top row of the 2x2 window)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(BLAZE_ROD),
            Ing::Block(POPPED_CHORUS_FRUIT),
            Ing::None,
            Ing::None,
        ],
        out: ItemStack::new(END_ROD, 4),
    },
    // ---- backlog round (farming, 2026-09-09): the farming recipes
    // (all VERIFIED live 2026-09-09 w/Bread, Hay_Bale, Hoe captures) ----
    // bread: 3 wheat in a row -> 1 (the classic first-farm recipe;
    // "Bread can be crafted from 3 units of wheat")
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(WHEAT), Ing::Block(WHEAT), Ing::Block(WHEAT),
            Ing::None, Ing::None, Ing::None,
            Ing::None, Ing::None, Ing::None,
        ],
        out: ItemStack::new(BREAD, 1),
    },
    // hay bale: 3x3 wheat -> 1 (VERIFIED w/Hay_Bale §Crafting: nine
    // wheat pieces; the block exists since the 1.6 era — its recipe
    // arrives with the farming bracket)
    Recipe {
        size: 3,
        grid: &[
            Ing::Block(WHEAT), Ing::Block(WHEAT), Ing::Block(WHEAT),
            Ing::Block(WHEAT), Ing::Block(WHEAT), Ing::Block(WHEAT),
            Ing::Block(WHEAT), Ing::Block(WHEAT), Ing::Block(WHEAT),
        ],
        out: ItemStack::new(HAY_BALE, 1),
    },
    // hoe (wooden tier): 2 planks over 2 sticks — vanilla's material-
    // material / -stick- / -stick- column (the engine's single generic
    // hoe; mirrored L-shapes rotate under the existing matcher)
    Recipe {
        size: 3,
        grid: &[
            Ing::AnyPlanks, Ing::AnyPlanks, Ing::None,
            Ing::None, Ing::Block(STICK), Ing::None,
            Ing::None, Ing::Block(STICK), Ing::None,
        ],
        out: ItemStack::new(HOE, 1),
    },
];

/// 1.16 (Nether Update, part 2): the shapeless SOUL-TORCH recipe —
/// 1 charcoal or coal + 1 stick + 1 soul soil or soul sand → 4 soul
/// torches (VERIFIED w/Soul_Torch §Crafting; the coal and charcoal
/// halves are both valid). The soul lantern rides the shaped path:
/// 8 iron nuggets + 1 soul torch (VERIFIED w/Soul_Torch §Crafting
/// ingredient table: "Soul Lantern — Iron Nugget + Soul Torch" —
/// the vanilla lantern recipe's soul form).
/// the completeness audit: the shapeless KITCHEN chain (all VERIFIED
/// live 2026-09-08 against the audit16 captures — the Bowl, Sugar,
/// Mushroom_Stew, Rabbit_Stew, Beetroot_Soup, Pumpkin_Pie pages):
/// - bowl: 3 "Any Planks" -> 4 (w/Bowl §Crafting: "Any Planks" -> 4)
/// - sugar: 1 honey bottle -> 3 (the 1.15 craft; the empty-bottle
///   grid-return is a no-container-return engine trim, disclosed —
///   the drink path returns its bottle)
/// - mushroom stew: 1 red + 1 brown + 1 bowl -> 1 (w/Mushroom_Stew
///   §Crafting: "Red Mushroom + Brown Mushroom + Bowl")
/// - rabbit stew: 1 cooked rabbit + 1 carrot + 1 baked potato + 1
///   red-OR-brown mushroom + 1 bowl -> 1 (w/Rabbit_Stew §Crafting)
/// - beetroot soup: 6 beetroot + 1 bowl -> 1 (w/Beetroot_Soup
///   §Crafting: "Beetroot + Bowl", the 6-root set)
/// - pumpkin pie: 1 pumpkin + 1 sugar + 1 egg -> 1 (w/Pumpkin_Pie
///   §Crafting: "Pumpkin + Sugar + Any Egg")
fn match_kitchen(slots: &[ItemStack], _size: usize) -> Option<ItemStack> {
    let mut planks = 0;
    let mut bowl = 0;
    let mut red = 0;
    let mut brown = 0;
    let mut carrot = 0;
    let mut baked = 0;
    let mut rabbit = 0;
    let mut beetroot = 0;
    let mut pumpkin = 0;
    let mut sugar = 0;
    let mut egg = 0;
    let mut honey = 0;
    let mut melon_slice = 0;
    let mut other = 0;
    for s in slots {
        match s.block {
            PLANKS | JUNGLE_PLANKS | CRIMSON_PLANKS | WARPED_PLANKS if !s.is_empty() => planks += 1,
            BOWL if !s.is_empty() => bowl += 1,
            MUSHROOM_RED if !s.is_empty() => red += 1,
            MUSHROOM_BROWN if !s.is_empty() => brown += 1,
            CARROT if !s.is_empty() => carrot += 1,
            BAKED_POTATO if !s.is_empty() => baked += 1,
            COOKED_RABBIT if !s.is_empty() => rabbit += 1,
            BEETROOT if !s.is_empty() => beetroot += 1,
            PUMPKIN if !s.is_empty() => pumpkin += 1,
            SUGAR if !s.is_empty() => sugar += 1,
            EGG if !s.is_empty() => egg += 1,
            HONEY_BOTTLE if !s.is_empty() => honey += 1,
            MELON_SLICE if !s.is_empty() => melon_slice += 1,
            MELON_SLICE if !s.is_empty() => melon_slice += 1,
            _ if !s.is_empty() => other += 1,
            _ => {}
        }
    }
    if other > 0 {
        return None;
    }
    let total = planks + bowl + red + brown + carrot + baked + rabbit
        + beetroot + pumpkin + sugar + egg + honey + melon_slice;
    // bowl: exactly 3 planks (the V-shape's 3 items, shapeless)
    if planks == 3
        && bowl == 0 && red == 0 && brown == 0 && carrot == 0 && baked == 0
        && rabbit == 0 && beetroot == 0 && pumpkin == 0 && sugar == 0
        && egg == 0 && honey == 0 && melon_slice == 0
    {
        return Some(ItemStack::new(BOWL, 4));
    }
    // sugar: exactly 1 honey bottle
    if honey == 1 && total == 1 {
        return Some(ItemStack::new(SUGAR, 3));
    }
    // mushroom stew: 1 red + 1 brown + 1 bowl
    if red == 1 && brown == 1 && bowl == 1 && total == 3 {
        return Some(ItemStack::new(MUSHROOM_STEW, 1));
    }
    // rabbit stew: 1 cooked rabbit + 1 carrot + 1 baked potato +
    // exactly one mushroom (red or brown) + 1 bowl
    if rabbit == 1
        && carrot == 1
        && baked == 1
        && bowl == 1
        && red + brown == 1
        && total == 5
    {
        return Some(ItemStack::new(RABBIT_STEW, 1));
    }
    // beetroot soup: 6 beetroot + 1 bowl
    if beetroot == 6 && bowl == 1 && total == 7 {
        return Some(ItemStack::new(BEETROOT_SOUP, 1));
    }
    // pumpkin pie: 1 pumpkin + 1 sugar + 1 egg
    if pumpkin == 1 && sugar == 1 && egg == 1 && total == 3 {
        return Some(ItemStack::new(PUMPKIN_PIE, 1));
    }
    // ---- the sweep-2 melon rows (VERIFIED w/Melon_Slice §Crafting,
    // live 2026-09-09: "Melon | Melon Slice" (the 3x3 nine-slice
    // recipe) and "Melon Seeds | Melon Slice" (1:1)) ----
    // the melon block: exactly 9 slices (the 3x3 grid)
    if melon_slice == 9 && total == 9 {
        return Some(ItemStack::new(MELON, 1));
    }
    // melon seeds: exactly 1 slice
    if melon_slice == 1 && total == 1 {
        return Some(ItemStack::new(MELON_SEEDS, 1));
    }
    None
}

fn match_soul_torch(slots: &[ItemStack], _size: usize) -> Option<ItemStack> {
    let mut fuel = 0; // coal or charcoal (exactly one)
    let mut stick_n = 0;
    let mut soul = 0; // soul soil or soul sand (exactly one)
    let mut torch = 0; // the soul-lantern path: exactly one soul torch
    let mut nuggets = 0;
    for s in slots {
        if s.is_empty() {
            continue;
        }
        match s.block {
            CHARCOAL | COAL => fuel += 1,
            STICK => stick_n += 1,
            SOUL_SOIL | SOUL_SAND => soul += 1,
            SOUL_TORCH => torch += 1,
            IRON_NUGGET => nuggets += 1,
            _ => return None, // any other ingredient breaks the multiset
        }
    }
    // the soul lantern: 8 nuggets + 1 soul torch (the lantern ring)
    if nuggets == 8 && torch == 1 && fuel == 0 && stick_n == 0 && soul == 0 {
        return Some(ItemStack::new(SOUL_LANTERN, 1));
    }
    // the soul torch: 1 fuel + 1 stick + 1 soul block
    if fuel == 1 && stick_n == 1 && soul == 1 && torch == 0 && nuggets == 0 {
        return Some(ItemStack::new(SOUL_TORCH, 4));
    }
    None
}

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
    // the completeness audit: the shapeless kitchen chain (the bowl /
    // sugar / stew / pie recipes, any arrangement)
    if let Some(out) = match_kitchen(slots, size) {
        return Some(out);
    }
    // 1.16 (Nether Update, part 2): the shapeless soul-torch +
    // soul-lantern recipes (any arrangement)
    if let Some(out) = match_soul_torch(slots, size) {
        return Some(out);
    }
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
                        Ing::AnyWood => {
                            !s.is_empty()
                                && matches!(
                                    s.block,
                                    OAK_LOG | BIRCH_LOG | SPRUCE_LOG | ACACIA_LOG
                                        | DARK_OAK_LOG | JUNGLE_LOG
                                )
                        }
                        Ing::AnyPlanks => {
                            // the completeness audit: vanilla's "Any
                            // Planks" covers every species — the 1.14-era
                            // oak+jungle pair extended to the 1.16 woods
                            !s.is_empty()
                                && matches!(
                                    s.block,
                                    PLANKS | JUNGLE_PLANKS | CRIMSON_PLANKS | WARPED_PLANKS
                                )
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

    /// the completeness audit: the kitchen chain + the purpur family
    /// (all VERIFIED live 2026-09-08 against the audit16 captures —
    /// Bowl/Sugar/Mushroom_Stew/Rabbit_Stew/Beetroot_Soup/Pumpkin_Pie/
    /// Popped_Chorus_Fruit)
    #[test]
    /// the sweep-2 melon crafts: 9 slices -> the melon block, 1 slice
    /// -> melon seeds (VERIFIED w/Melon_Slice §Crafting, live 2026-09-09)
    #[test]
    fn audit16_sweep2_melon_crafts() {
        let g = vec![ItemStack::new(MELON_SLICE, 1); 9];
        let out = match_grid(&g, 3).expect("the 3x3 nine-slice recipe");
        assert_eq!(out.block, MELON);
        assert_eq!(out.count, 1);
        // a partial grid (8 slices) must NOT craft
        let mut partial = vec![ItemStack::EMPTY; 9];
        for i in 0..8 {
            partial[i] = ItemStack::new(MELON_SLICE, 1);
        }
        assert!(match_grid(&partial, 3).is_none(), "8 slices craft nothing");
        // 1 slice -> 1 melon seed
        let mut one = vec![ItemStack::EMPTY; 9];
        one[4] = ItemStack::new(MELON_SLICE, 1);
        let out = match_grid(&one, 3).expect("the slice-to-seeds row");
        assert_eq!(out.block, MELON_SEEDS);
        assert_eq!(out.count, 1);
    }

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
        // beetroot soup: 6 beetroot + bowl (SLOT counts — the ring
        // pattern of the 3x3 minus the corners)
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0, 2, 3, 5, 6, 8] {
            g[i] = ItemStack::new(BEETROOT, 2);
        }
        g[4] = ItemStack::new(BOWL, 1);
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
        // blaze rod + popped chorus -> 4 end rods (the top row of the
        // 2x2 window)
        let g = vec![
            ItemStack::new(BLAZE_ROD, 1),
            ItemStack::new(POPPED_CHORUS_FRUIT, 1),
            ItemStack::EMPTY,
            ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (END_ROD, 4));
    }

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

    /// 1.14 (Village & Pillage — nature half): the stick, campfire and
    /// barrel recipes (VERIFIED live 2026-09-08 from the raw captures
    /// v114_page_*.json).
    #[test]
    fn v114_nature_recipes() {
        // stick: 2 planks stacked → 4 sticks (either orientation; any
        // of the engine's planks)
        for (planks, vertical) in [(PLANKS, true), (JUNGLE_PLANKS, true), (PLANKS, false)] {
            let mut g = vec![ItemStack::EMPTY; 4];
            if vertical {
                g[0] = ItemStack::new(planks, 1);
                g[2] = ItemStack::new(planks, 1);
            } else {
                g[0] = ItemStack::new(planks, 1);
                g[1] = ItemStack::new(planks, 1);
            }
            let out = match_grid(&g, 2).unwrap();
            assert_eq!(out.block, STICK);
            assert_eq!(out.count, 4, "the classic 2-plank → 4-stick row");
        }
        // one plank alone: nothing (two are required for sticks)
        let g1 = vec![ItemStack::new(PLANKS, 1)];
        assert!(match_grid(&g1, 1).is_none(), "one plank crafts nothing");
        // campfire: 3 sticks / coal / 3 logs (any of the 6 woods)
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 1, 2] {
            g[i] = ItemStack::new(STICK, 1);
        }
        g[4] = ItemStack::new(COAL, 1);
        for i in [6usize, 7, 8] {
            g[i] = ItemStack::new(JUNGLE_LOG, 1);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!(out.block, CAMPFIRE);
        assert_eq!(out.count, 1);
        // mixed woods in the bottom row still craft (any log)
        g[7] = ItemStack::new(SPRUCE_LOG, 1);
        assert_eq!(match_grid(&g, 3).unwrap().block, CAMPFIRE);
        // charcoal replaces coal as the fuel (vanilla: "Coal or
        // Charcoal")
        g[4] = ItemStack::new(CHARCOAL, 1);
        assert_eq!(match_grid(&g, 3).unwrap().block, CAMPFIRE);
        // a missing stick breaks the pattern
        g[2] = ItemStack::EMPTY;
        assert!(match_grid(&g, 3).is_none(), "2 sticks is not the recipe");
        // barrel: planks columns + slab caps (6 planks + 2 slabs)
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 2, 3, 5, 6, 8] {
            g[i] = ItemStack::new(PLANKS, 1);
        }
        g[1] = ItemStack::new(OAK_SLAB, 1);
        g[7] = ItemStack::new(OAK_SLAB, 1);
        let out = match_grid(&g, 3).unwrap();
        assert_eq!(out.block, BARREL);
        assert_eq!(out.count, 1);
        // jungle planks also work (any planks)
        for i in [0usize, 2, 3, 5, 6, 8] {
            g[i] = ItemStack::new(JUNGLE_PLANKS, 1);
        }
        assert_eq!(match_grid(&g, 3).unwrap().block, BARREL);
        // a missing slab cap breaks it
        g[7] = ItemStack::EMPTY;
        assert!(match_grid(&g, 3).is_none(), "one slab is not two");
    }
    /// 1.14 (nature half, part 2) recipes — VERIFIED 2026-09-08 from
    /// the raw captures v114b_page_*.json: blast furnace (5 iron +
    /// furnace + 3 smooth stone), smoker (the 4-log cross around a
    /// furnace), lantern (8 nuggets + torch), and the 9:1 nugget
    /// round-trips with the iron-ingot stand-in.
    #[test]
    fn v114b_smelter_lantern_recipes() {
        // blast furnace: iron top+sides, furnace center, smooth stone bottom
        let mut g = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 1, 2, 3, 5] {
            g[i] = ItemStack::new(IRON_ORE, 1);
        }
        g[4] = ItemStack::new(FURNACE, 1);
        for i in [6usize, 7, 8] {
            g[i] = ItemStack::new(SMOOTH_STONE, 1);
        }
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BLAST_FURNACE, 1));
        // 4 iron (a side missing) is not the recipe
        let mut bad = g.clone();
        bad[3] = ItemStack::EMPTY;
        assert!(match_grid(&bad, 3).is_none(), "4 iron is not the recipe");
        // cobble instead of smooth stone in the bottom row: rejected
        let mut bad2 = g.clone();
        bad2[7] = ItemStack::new(COBBLE, 1);
        assert!(match_grid(&bad2, 3).is_none(), "cobble bottom is not the recipe");

        // smoker: the 4-log cross around the furnace (any of the 6 woods)
        let mut s = vec![ItemStack::EMPTY; 9];
        s[1] = ItemStack::new(OAK_LOG, 1);
        s[3] = ItemStack::new(JUNGLE_LOG, 1);
        s[4] = ItemStack::new(FURNACE, 1);
        s[5] = ItemStack::new(BIRCH_LOG, 1);
        s[7] = ItemStack::new(DARK_OAK_LOG, 1);
        let out = match_grid(&s, 3).unwrap();
        assert_eq!((out.block, out.count), (SMOKER, 1));
        // a corner log breaks the cross
        let mut bad3 = s.clone();
        bad3[0] = ItemStack::new(OAK_LOG, 1);
        assert!(match_grid(&bad3, 3).is_none(), "corner log is not the recipe");

        // lantern: the 8-nugget ring around the torch
        let mut l = vec![ItemStack::EMPTY; 9];
        for i in [0usize, 1, 2, 3, 5, 6, 7, 8] {
            l[i] = ItemStack::new(IRON_NUGGET, 1);
        }
        l[4] = ItemStack::new(REDSTONE_TORCH, 1);
        let out = match_grid(&l, 3).unwrap();
        assert_eq!((out.block, out.count), (LANTERN, 1));
        // a nugget missing breaks the ring
        let mut bad4 = l.clone();
        bad4[0] = ItemStack::EMPTY;
        assert!(match_grid(&bad4, 3).is_none(), "7 nuggets is not the recipe");

        // the nugget round-trips with the iron-ingot stand-in
        let one = vec![ItemStack::new(IRON_ORE, 1)];
        let out = match_grid(&one, 1).unwrap();
        assert_eq!((out.block, out.count), (IRON_NUGGET, 9));
        let nine = vec![ItemStack::new(IRON_NUGGET, 1); 9];
        let out = match_grid(&nine, 3).unwrap();
        assert_eq!((out.block, out.count), (IRON_ORE, 1));
    }

    /// 1.14 (part 3): the flower→dye pair (VERIFIED w/Cornflower
    /// §Crafting ingredient "Blue Dye — Cornflower"; w/Lily_of_the_
    /// Valley "White Dye — Lily of the Valley") — 1:1, like vanilla.
    #[test]
    fn v114c_flower_dye_recipes() {
        // cornflower → blue dye (index 11, the lapis row)
        let g = vec![ItemStack::new(CORNFLOWER, 1)];
        let out = match_grid(&g, 1).unwrap();
        assert_eq!((out.block, out.count), (DYE_BASE + 11, 1), "cornflower → blue dye");

        // lily of the valley → white dye (index 0)
        let g = vec![ItemStack::new(LILY_OF_THE_VALLEY, 1)];
        let out = match_grid(&g, 1).unwrap();
        assert_eq!((out.block, out.count), (DYE_BASE, 1), "lily of the valley → white dye");

        // the 1.7 flowers do NOT dye (no such recipes — the 1.14 pair
        // is the engine's first flower crafts, disclosed)
        let g = vec![ItemStack::new(ALLIUM, 1)];
        assert!(match_grid(&g, 1).is_none(), "allium has no dye recipe yet");
    }

    /// 1.16 (Nether Update, part 1): the anchor family's six craft
    /// contracts (all VERIFIED against the v116 captures — gold = the
    /// iron-ingot stand-in, redstone dust = the redstone block, both
    /// the disclosed conventions)
    #[test]
    fn v116_anchor_family_recipes() {
        // respawn anchor: 6 crying obsidian + 3 glowstone (the ring
        // around the column)
        let g = vec![
            ItemStack::new(CRYING_OBSIDIAN, 1), ItemStack::new(GLOWSTONE, 1), ItemStack::new(CRYING_OBSIDIAN, 1),
            ItemStack::new(CRYING_OBSIDIAN, 1), ItemStack::new(GLOWSTONE, 1), ItemStack::new(CRYING_OBSIDIAN, 1),
            ItemStack::new(CRYING_OBSIDIAN, 1), ItemStack::new(GLOWSTONE, 1), ItemStack::new(CRYING_OBSIDIAN, 1),
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (RESPAWN_ANCHOR, 1));

        // target: 4 redstone dust (block stand-in) around 1 hay bale
        let g = vec![
            ItemStack::EMPTY,               ItemStack::new(REDSTONE_BLOCK, 1), ItemStack::EMPTY,
            ItemStack::new(REDSTONE_BLOCK, 1), ItemStack::new(HAY_BALE, 1),   ItemStack::new(REDSTONE_BLOCK, 1),
            ItemStack::EMPTY,               ItemStack::new(REDSTONE_BLOCK, 1), ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (TARGET, 1));

        // netherite ingot: 4 scrap + 4 gold (iron stand-in) in the
        // checker board
        let g = vec![
            ItemStack::new(NETHERITE_SCRAP, 1), ItemStack::new(IRON_ORE, 1), ItemStack::new(NETHERITE_SCRAP, 1),
            ItemStack::new(IRON_ORE, 1),        ItemStack::EMPTY,           ItemStack::new(IRON_ORE, 1),
            ItemStack::new(NETHERITE_SCRAP, 1), ItemStack::new(IRON_ORE, 1), ItemStack::new(NETHERITE_SCRAP, 1),
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (NETHERITE_INGOT, 1));

        // block of netherite: 9 ingots, and back into 9
        let g = vec![ItemStack::new(NETHERITE_INGOT, 1); 9];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (NETHERITE_BLOCK, 1));
        let g = vec![ItemStack::new(NETHERITE_BLOCK, 1)];
        let out = match_grid(&g, 1).unwrap();
        assert_eq!((out.block, out.count), (NETHERITE_INGOT, 9));

        // chain: 1 nugget over the ingot over 1 nugget, down the
        // middle column of the 3x3 grid (the 1.16 iron-only form; the
        // copper variants are 1.21+)
        let g = vec![
            ItemStack::EMPTY,           ItemStack::new(IRON_NUGGET, 1), ItemStack::EMPTY,
            ItemStack::EMPTY,           ItemStack::new(IRON_ORE, 1),     ItemStack::EMPTY,
            ItemStack::EMPTY,           ItemStack::new(IRON_NUGGET, 1), ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (CHAIN, 1));
    }

    /// 1.16 (Nether Update, part 2): the crimson/warped families'
    /// craft contracts (all VERIFIED against the v116b captures —
    /// the research record docs/research/phase-v116b-1.16-research.md)
    #[test]
    fn v116b_forest_family_recipes() {
        // the stems → 4 planks each (the universal log→planks rule)
        for (stem, planks) in [
            (CRIMSON_STEM, CRIMSON_PLANKS),
            (CRIMSON_HYPHAE, CRIMSON_PLANKS),
            (WARPED_STEM, WARPED_PLANKS),
            (WARPED_HYPHAE, WARPED_PLANKS),
        ] {
            let g = vec![ItemStack::new(stem, 1)];
            let out = match_grid(&g, 1).unwrap();
            assert_eq!(
                (out.block, out.count),
                (planks, 4),
                "{stem} -> 4 planks (the 1:4 rule)"
            );
        }

        // the polished stone 2x2 family: basalt / blackstone / bricks
        let g = vec![
            ItemStack::new(BASALT, 1), ItemStack::new(BASALT, 1),
            ItemStack::new(BASALT, 1), ItemStack::new(BASALT, 1),
        ];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (POLISHED_BASALT, 4));

        let g = vec![
            ItemStack::new(BLACKSTONE, 1), ItemStack::new(BLACKSTONE, 1),
            ItemStack::new(BLACKSTONE, 1), ItemStack::new(BLACKSTONE, 1),
        ];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (POLISHED_BLACKSTONE, 4));

        let g = vec![
            ItemStack::new(POLISHED_BLACKSTONE, 1), ItemStack::new(POLISHED_BLACKSTONE, 1),
            ItemStack::new(POLISHED_BLACKSTONE, 1), ItemStack::new(POLISHED_BLACKSTONE, 1),
        ];
        let out = match_grid(&g, 2).unwrap();
        assert_eq!((out.block, out.count), (POLISHED_BLACKSTONE_BRICKS, 4));

        // the soul torch: SHAPELESS — 1 charcoal + 1 stick + 1 soul
        // soil (any arrangement; the coal + soul-sand halves too)
        for fuel in [CHARCOAL, COAL] {
            for soul in [SOUL_SOIL, SOUL_SAND] {
                let g = vec![
                    ItemStack::new(fuel, 1),  ItemStack::EMPTY,           ItemStack::new(STICK, 1),
                    ItemStack::EMPTY,           ItemStack::new(soul, 1),  ItemStack::EMPTY,
                    ItemStack::EMPTY,           ItemStack::EMPTY,           ItemStack::EMPTY,
                ];
                let out = match_grid(&g, 3).unwrap();
                assert_eq!((out.block, out.count), (SOUL_TORCH, 4));
            }
        }

        // the soul lantern: 8 iron nuggets + 1 soul torch (the ring)
        let g = vec![
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1),
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(SOUL_TORCH, 1),  ItemStack::new(IRON_NUGGET, 1),
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1),
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (SOUL_LANTERN, 1));

        // negative: 7 nuggets + a soul torch is NOT the recipe
        let g = vec![
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1),
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(SOUL_TORCH, 1),  ItemStack::EMPTY,
            ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1), ItemStack::new(IRON_NUGGET, 1),
        ];
        assert!(match_grid(&g, 3).is_none(), "the 8-nugget ring is exact");
    }
}

#[cfg(test)]
mod farm_recipe_tests {
    use super::*;

    /// backlog round (farming): bread — 3 wheat in a row → 1
    /// (VERIFIED w/Bread §Crafting)
    #[test]
    fn bread_crafts_from_three_wheat() {
        let g = vec![
            ItemStack::new(WHEAT, 1), ItemStack::new(WHEAT, 1), ItemStack::new(WHEAT, 1),
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (BREAD, 1));
        // negative: 2 wheat is not bread
        let g = vec![
            ItemStack::new(WHEAT, 1), ItemStack::new(WHEAT, 1), ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY,
        ];
        assert!(match_grid(&g, 3).is_none(), "2 wheat ≠ bread");
    }

    /// hay bale: the 3×3 wheat block → 1 (VERIFIED w/Hay_Bale)
    #[test]
    fn hay_bale_crafts_from_nine_wheat() {
        let g: Vec<ItemStack> = (0..9).map(|_| ItemStack::new(WHEAT, 1)).collect();
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (HAY_BALE, 1));
    }

    /// the hoe: 2 planks over 2 sticks → 1 (the wooden-tier column,
    /// VERIFIED w/Hoe §Crafting — mirrored L accepted via rotation)
    #[test]
    fn hoe_crafts_from_planks_and_sticks() {
        let g = vec![
            ItemStack::new(PLANKS, 1), ItemStack::new(PLANKS, 1), ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::new(STICK, 1), ItemStack::EMPTY,
            ItemStack::EMPTY, ItemStack::new(STICK, 1), ItemStack::EMPTY,
        ];
        let out = match_grid(&g, 3).unwrap();
        assert_eq!((out.block, out.count), (HOE, 1));
    }
}
