//! Block registry — ids, tiles, physical + optical properties, sound families.
//! 57 blocks in the style of MC 1.16.5's overworld palette (all textures
//! procedurally synthesized — none copied from Mojang assets).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundFamily {
    Grass,
    Dirt,
    Stone,
    Wood,
    Sand,
    Leaves,
    Glass,
    Wool,
    Water,
    None,
}

// Tile indices inside the 16x16-tile procedural atlas (256x256 px).
pub const TILE_GRASS_TOP: u16 = 0;
pub const TILE_GRASS_SIDE: u16 = 1;
pub const TILE_DIRT: u16 = 2;
pub const TILE_STONE: u16 = 3;
pub const TILE_COBBLE: u16 = 4;
pub const TILE_SAND: u16 = 5;
pub const TILE_LOG_SIDE: u16 = 6;
pub const TILE_LOG_TOP: u16 = 7;
pub const TILE_PLANKS: u16 = 8;
pub const TILE_LEAVES: u16 = 9;
pub const TILE_WATER: u16 = 10;
pub const TILE_GLASS: u16 = 11;
pub const TILE_BEDROCK: u16 = 12;
pub const TILE_GRAVEL: u16 = 13;
pub const TILE_SNOW: u16 = 14;
pub const TILE_SNOW_SIDE: u16 = 15;
pub const TILE_TALL_GRASS: u16 = 16;
pub const TILE_FLOWER_RED: u16 = 17;
pub const TILE_FLOWER_YELLOW: u16 = 18;
// stone family
pub const TILE_GRANITE: u16 = 19;
pub const TILE_DIORITE: u16 = 20;
pub const TILE_ANDESITE: u16 = 21;
pub const TILE_STONE_BRICKS: u16 = 22;
pub const TILE_BRICKS: u16 = 23;
pub const TILE_MOSSY_COBBLE: u16 = 24;
pub const TILE_SMOOTH_STONE: u16 = 25;
pub const TILE_OBSIDIAN: u16 = 26;
// ores
pub const TILE_COAL_ORE: u16 = 27;
pub const TILE_IRON_ORE: u16 = 28;
pub const TILE_GOLD_ORE: u16 = 29;
pub const TILE_DIAMOND_ORE: u16 = 30;
pub const TILE_REDSTONE_ORE: u16 = 31;
pub const TILE_LAPIS_ORE: u16 = 32;
pub const TILE_EMERALD_ORE: u16 = 33;
// mineral blocks
pub const TILE_IRON_BLOCK: u16 = 34;
pub const TILE_GOLD_BLOCK: u16 = 35;
pub const TILE_DIAMOND_BLOCK: u16 = 36;
// misc
pub const TILE_GLOWSTONE: u16 = 37;
pub const TILE_BOOKSHELF_SIDE: u16 = 38;
pub const TILE_BOOKSHELF_TOP: u16 = 39;
pub const TILE_CRAFT_TOP: u16 = 40;
pub const TILE_CRAFT_SIDE: u16 = 41;
pub const TILE_CLAY: u16 = 42;
pub const TILE_TERRACOTTA: u16 = 43;
pub const TILE_PUMPKIN_SIDE: u16 = 44;
pub const TILE_PUMPKIN_TOP: u16 = 45;
pub const TILE_MELON_SIDE: u16 = 46;
pub const TILE_MELON_TOP: u16 = 47;
pub const TILE_ICE: u16 = 48;
pub const TILE_CACTUS_SIDE: u16 = 49;
pub const TILE_CACTUS_TOP: u16 = 50;
// wool
pub const TILE_WOOL_WHITE: u16 = 51;
pub const TILE_WOOL_RED: u16 = 52;
pub const TILE_WOOL_BLUE: u16 = 53;
pub const TILE_WOOL_YELLOW: u16 = 54;
pub const TILE_WOOL_BLACK: u16 = 55;
// wood variants
pub const TILE_BIRCH_LOG_SIDE: u16 = 56;
pub const TILE_BIRCH_LEAVES: u16 = 57;
pub const TILE_SPRUCE_LOG_SIDE: u16 = 58;
pub const TILE_SPRUCE_LEAVES: u16 = 59;
// plants
pub const TILE_MUSHROOM_RED: u16 = 60;
pub const TILE_MUSHROOM_BROWN: u16 = 61;
pub const TILE_DEAD_BUSH: u16 = 62;
// redstone core (Phase 6 §25)
pub const TILE_REDSTONE_WIRE: u16 = 64;
pub const TILE_REDSTONE_TORCH: u16 = 65;
pub const TILE_LEVER: u16 = 66;
// gameplay (Phase 7)
pub const TILE_FURNACE_SIDE: u16 = 67;
pub const TILE_FURNACE_TOP: u16 = 68;
pub const TILE_FURNACE_LIT_SIDE: u16 = 69;
// nether blocks (Phase 7 §28 dimensions)
pub const TILE_NETHERRACK: u16 = 70;
pub const TILE_QUARTZ_ORE: u16 = 71;
pub const TILE_SOUL_SAND: u16 = 72;
// brewing (Phase 7 §29): stand + potion bottles (item-only tiles)
pub const TILE_BREWING_STAND: u16 = 73;
pub const TILE_BOTTLE_EMPTY: u16 = 74;
pub const TILE_POTION_WATER: u16 = 75;
pub const TILE_POTION_AWKWARD: u16 = 76;
pub const TILE_POTION_MUNDANE: u16 = 77;
pub const TILE_POTION_HEALING: u16 = 78;
pub const TILE_POTION_HEALING_II: u16 = 79;
// enchanting (Phase 7 §29): table + enchanted book
pub const TILE_ENCHANT_TABLE: u16 = 80;
pub const TILE_ENCHANTED_BOOK: u16 = 81;
// villagers (Phase 7 §27/§29): the NPC sprite
pub const TILE_VILLAGER: u16 = 82;
// Phase 4 §26/§30: corruption-chain potion + item tiles
pub const TILE_POTION_HARMING: u16 = 118;
pub const TILE_POTION_HARMING_II: u16 = 119;
pub const TILE_SPIDER_EYE: u16 = 120;
pub const TILE_FERMENTED_EYE: u16 = 121;
// Phase 5 §27: monster-spawner cage face (dark lattice, vanilla-like
// cage silhouette, clean-room drawn — see textures.rs spawner_art)
pub const TILE_SPAWNER: u16 = 122;
/// Phase 10: end-portal frame tile (clean-room inset-ring face)
pub const TILE_END_PORTAL_FRAME: u16 = 123;
// ---- Phase E1 (evolution 1.0–1.2 bracket, live-verified 2026-09-06) ----
// world blocks
pub const TILE_MYCELIUM_TOP: u16 = 124;
pub const TILE_MYCELIUM_SIDE: u16 = 125;
pub const TILE_END_STONE: u16 = 126;
pub const TILE_NETHER_BRICKS: u16 = 127;
pub const TILE_REDSTONE_LAMP: u16 = 128;
pub const TILE_REDSTONE_LAMP_ON: u16 = 129;
pub const TILE_CHISELED_STONE_BRICKS: u16 = 130;
pub const TILE_CHISELED_SANDSTONE: u16 = 131;
pub const TILE_CUT_SANDSTONE: u16 = 132;
pub const TILE_SMOOTH_SANDSTONE: u16 = 133;
pub const TILE_MUSHROOM_RED_BLOCK: u16 = 134;
pub const TILE_MUSHROOM_BROWN_BLOCK: u16 = 135;
pub const TILE_MUSHROOM_STEM: u16 = 136;
pub const TILE_NETHER_WART_0: u16 = 137;
pub const TILE_NETHER_WART_1: u16 = 138;
pub const TILE_NETHER_WART_2: u16 = 139;
pub const TILE_NETHER_WART_3: u16 = 140;
pub const TILE_DRAGON_EGG: u16 = 141;
pub const TILE_END_PORTAL: u16 = 142;
// items + entity sprites
pub const TILE_END_CRYSTAL: u16 = 143;
pub const TILE_XP_ORB: u16 = 144;
pub const TILE_XP_ORB_BIG: u16 = 145;
pub const TILE_EYE_OF_ENDER: u16 = 146;
pub const TILE_BLAZE_ROD: u16 = 147;
pub const TILE_BLAZE_POWDER: u16 = 148;
pub const TILE_GOLDEN_APPLE: u16 = 149;
pub const TILE_SNOWBALL: u16 = 150;
pub const TILE_NETHER_BRICK: u16 = 151;
pub const TILE_SNOWGOLEM: u16 = 152;
pub const TILE_MAGMACUBE: u16 = 153;
pub const TILE_BLAZE: u16 = 154;
pub const TILE_OCELOT: u16 = 155;
pub const TILE_IRONGOLEM: u16 = 156;
pub const TILE_ZOMBIEVILLAGER: u16 = 157;
pub const TILE_MOOSHROOM: u16 = 158;
pub const TILE_ENDERDRAGON: u16 = 159;
// spawn eggs: 20 tiles, one per implemented mob kind (base 160..=179)
pub const TILE_EGG_BASE: u16 = 160;
pub const TILE_EGG_MAX: u16 = 179;
// ---- Phase E2 (evolution 1.3–1.4 bracket, live-verified 2026-09-06) ----
// world-block + item tiles (180..=196)
pub const TILE_ANVIL: u16 = 180;
pub const TILE_ANVIL_CHIPPED: u16 = 181;
pub const TILE_ANVIL_DAMAGED: u16 = 182;
pub const TILE_BEACON: u16 = 183;
pub const TILE_BEACON_BEAM: u16 = 184;
pub const TILE_COBBLE_WALL: u16 = 185;
pub const TILE_ENDER_CHEST: u16 = 186;
pub const TILE_FLOWER_POT: u16 = 187;
pub const TILE_ITEM_FRAME: u16 = 188;
pub const TILE_TRIPWIRE_HOOK: u16 = 189;
pub const TILE_TRIPWIRE_HOOK_ON: u16 = 190;
pub const TILE_WITHER_SKULL: u16 = 191;
pub const TILE_COMMAND_BLOCK: u16 = 192;
pub const TILE_COMMAND_BLOCK_ON: u16 = 193;
pub const TILE_EMERALD: u16 = 194;
pub const TILE_NETHER_STAR: u16 = 195;
pub const TILE_POTATO: u16 = 196;
pub const TILE_BAKED_POTATO: u16 = 197;
pub const TILE_CARROT: u16 = 198;
pub const TILE_PUMPKIN_PIE: u16 = 199;
// E2 mob sprites (billboards)
pub const TILE_WITHER: u16 = 200;
pub const TILE_WITHER_SKELETON: u16 = 201;
pub const TILE_WITCH: u16 = 202;
pub const TILE_BAT: u16 = 203;
pub const TILE_WITHER_SKULL_PROJ: u16 = 204;
/// lava fluid tile (the flowing face art — one shared tile like water)
pub const TILE_LAVA: u16 = 205;
// mobs (Phase 2): entity sprites + drops' item tiles. Mob sprites are
// clean-room pixel art (ours, not Mojang's) — distinct silhouettes/palettes
pub const TILE_ZOMBIE: u16 = 83;
pub const TILE_SKELETON: u16 = 84;
pub const TILE_CREEPER: u16 = 85;
pub const TILE_SPIDER: u16 = 86;
pub const TILE_ENDERMAN: u16 = 87;
pub const TILE_COW: u16 = 88;
pub const TILE_PIG: u16 = 89;
pub const TILE_SHEEP: u16 = 90;
pub const TILE_CHICKEN: u16 = 91;
pub const TILE_ARROW: u16 = 92;
pub const TILE_BEEF: u16 = 93;
pub const TILE_PORKCHOP: u16 = 94;
pub const TILE_MUTTON: u16 = 95;
pub const TILE_CHICKEN_RAW: u16 = 96;
pub const TILE_FEATHER: u16 = 97;
pub const TILE_LEATHER: u16 = 98;
pub const TILE_BONE: u16 = 99;
pub const TILE_STRING: u16 = 100;
pub const TILE_GUNPOWDER: u16 = 101;
pub const TILE_ENDER_PEARL: u16 = 102;
pub const TILE_ROTTEN_FLESH: u16 = 103;
pub const TILE_ARROW_ITEM: u16 = 104;
// redstone components (Phase 3): repeater, comparator, pistons,
// dispenser, dropper, observer, hopper + chest (hopper target)
pub const TILE_REPEATER: u16 = 105;
pub const TILE_COMPARATOR: u16 = 106;
pub const TILE_PISTON: u16 = 107;
pub const TILE_STICKY_PISTON: u16 = 108;
pub const TILE_DISPENSER: u16 = 109;
pub const TILE_DROPPER: u16 = 110;
pub const TILE_OBSERVER: u16 = 111;
pub const TILE_HOPPER: u16 = 112;
pub const TILE_CHEST: u16 = 113;
// their lit/active overlays where the state needs a second tile
pub const TILE_REPEATER_ON: u16 = 114;
pub const TILE_COMPARATOR_ON: u16 = 115;
pub const TILE_OBSERVER_ON: u16 = 116;
pub const TILE_PISTON_HEAD: u16 = 117;

// Block ids (u8 in chunk storage).
pub const AIR: u8 = 0;
pub const GRASS: u8 = 1;
pub const DIRT: u8 = 2;
pub const STONE: u8 = 3;
pub const COBBLE: u8 = 4;
pub const SAND: u8 = 5;
pub const OAK_LOG: u8 = 6;
pub const PLANKS: u8 = 7;
pub const LEAVES: u8 = 8;
pub const WATER: u8 = 9;
pub const GLASS: u8 = 10;
pub const BEDROCK: u8 = 11;
pub const GRAVEL: u8 = 12;
pub const SNOW: u8 = 13;
pub const SNOW_GRASS: u8 = 14;
pub const TALL_GRASS: u8 = 15;
pub const FLOWER_RED: u8 = 16;
pub const FLOWER_YELLOW: u8 = 17;
// stone family
pub const GRANITE: u8 = 18;
pub const DIORITE: u8 = 19;
pub const ANDESITE: u8 = 20;
pub const STONE_BRICKS: u8 = 21;
pub const BRICKS: u8 = 22;
pub const MOSSY_COBBLE: u8 = 23;
pub const SMOOTH_STONE: u8 = 24;
pub const OBSIDIAN: u8 = 25;
// ores
pub const COAL_ORE: u8 = 26;
pub const IRON_ORE: u8 = 27;
pub const GOLD_ORE: u8 = 28;
pub const DIAMOND_ORE: u8 = 29;
pub const REDSTONE_ORE: u8 = 30;
pub const LAPIS_ORE: u8 = 31;
pub const EMERALD_ORE: u8 = 32;
// mineral blocks
pub const IRON_BLOCK: u8 = 33;
pub const GOLD_BLOCK: u8 = 34;
pub const DIAMOND_BLOCK: u8 = 35;
// misc
pub const GLOWSTONE: u8 = 36;
pub const BOOKSHELF: u8 = 37;
pub const CRAFTING_TABLE: u8 = 38;
pub const CLAY: u8 = 39;
pub const TERRACOTTA: u8 = 40;
pub const PUMPKIN: u8 = 41;
pub const MELON: u8 = 42;
pub const ICE: u8 = 43;
pub const CACTUS: u8 = 44;
// wool
pub const WOOL_WHITE: u8 = 45;
pub const WOOL_RED: u8 = 46;
pub const WOOL_BLUE: u8 = 47;
pub const WOOL_YELLOW: u8 = 48;
pub const WOOL_BLACK: u8 = 49;
// wood variants
pub const BIRCH_LOG: u8 = 50;
pub const BIRCH_LEAVES: u8 = 51;
pub const SPRUCE_LOG: u8 = 52;
pub const SPRUCE_LEAVES: u8 = 53;
// plants
pub const MUSHROOM_RED: u8 = 54;
pub const MUSHROOM_BROWN: u8 = 55;
pub const DEAD_BUSH: u8 = 56;

// (BLOCK_COUNT moved below — after all item ids are declared)

// redstone core (Phase 6 §25 subset)
pub const REDSTONE_WIRE: u8 = 60;
pub const REDSTONE_TORCH: u8 = 61;
pub const LEVER: u8 = 62;
// gameplay (Phase 7)
pub const FURNACE: u8 = 63;
// nether blocks (Phase 7 §28 dimensions): identities collide with the
// log-axis/model state slots exactly like FURNACE — they too always store
// their dedicated STATE ids below
pub const NETHERRACK: u8 = 64;
pub const NETHER_QUARTZ_ORE: u8 = 65;
pub const SOUL_SAND: u8 = 66;
pub const NETHERRACK_STATE: u16 = 118;
pub const QUARTZ_ORE_STATE: u16 = 119;
pub const SOUL_SAND_STATE: u16 = 120;
// brewing (Phase 7 §29): the stand block + potion ITEM ids. Potions live in
// inventories/hotbar only — never stored in the world. Their identity ids
// (67..73) collide with the COBBLE_STAIRS/OAK_FENCE model-state range, so
// like the nether blocks they get dedicated registry states and fold
// through state_block like everything else (§46 defensive folding).
pub const BREWING_STAND: u8 = 67;
pub const POTION_EMPTY: u8 = 68; // "Glass Bottle"
pub const POTION_WATER: u8 = 69; // "Water Bottle"
pub const POTION_AWKWARD: u8 = 70;
pub const POTION_MUNDANE: u8 = 71;
pub const POTION_HEALING: u8 = 72;
pub const POTION_HEALING_II: u8 = 73;
pub const BREWING_STAND_STATE: u16 = 121;
pub const POTION_EMPTY_STATE: u16 = 122;
pub const POTION_WATER_STATE: u16 = 123;
pub const POTION_AWKWARD_STATE: u16 = 124;
pub const POTION_MUNDANE_STATE: u16 = 125;
pub const POTION_HEALING_STATE: u16 = 126;
pub const POTION_HEALING_II_STATE: u16 = 127;
// enchanting (Phase 7 §29): table block + the book item-block (same
// dedicated-state pattern; the book carries the enchant in ItemStack.ench)
pub const ENCHANT_TABLE: u8 = 74;
pub const ENCHANTED_BOOK: u8 = 75;
pub const ENCHANT_TABLE_STATE: u16 = 128;
pub const ENCHANTED_BOOK_STATE: u16 = 129;
// mob drops (Phase 2): item-only ids in the potion pattern — they live in
// inventories/hotbar, never stored in the world. Registered names are
// vanilla registry strings (mechanical data, safe to match); the art is ours.
pub const BEEF: u8 = 76;
pub const PORKCHOP: u8 = 77;
pub const MUTTON: u8 = 78;
pub const CHICKEN_RAW: u8 = 79;
pub const FEATHER: u8 = 80;
pub const LEATHER: u8 = 81;
pub const BONE: u8 = 82;
pub const STRING: u8 = 83;
pub const GUNPOWDER: u8 = 84;
pub const ENDER_PEARL: u8 = 85;
pub const ROTTEN_FLESH: u8 = 86;
pub const ARROW_ITEM: u8 = 87;
// redstone components (Phase 3): ids 88..=96, dedicated sim states above
pub const REPEATER: u8 = 88;
pub const COMPARATOR: u8 = 89;
pub const PISTON: u8 = 90;
pub const STICKY_PISTON: u8 = 91;
pub const DISPENSER: u8 = 92;
pub const DROPPER: u8 = 93;
pub const OBSERVER: u8 = 94;
pub const HOPPER: u8 = 95;
pub const CHEST: u8 = 96;
// brewing expansion (Phase 4 §26/§30): the corruption chain + its items
pub const POTION_HARMING: u8 = 97;
pub const POTION_HARMING_II: u8 = 98;
pub const SPIDER_EYE: u8 = 99;
pub const FERMENTED_SPIDER_EYE: u8 = 100;
/// Phase 5 §27: monster spawner (dungeon block entity). Mob type is
/// encoded in the block state (232 zombie / 233 skeleton / 234 spider).
pub const SPAWNER: u8 = 101;
/// Phase 10: end-portal frame block (stronghold portal room ring).
/// Decorative-only: eye-of-ender insertion + portal activation are out
/// of scope (documented); the frame marks the vanilla portal room's
/// 12-frame ring, ours renders as a full cube with a frame inset.
pub const END_PORTAL_FRAME: u8 = 102;

// ---- Phase E1 block ids (evolution 1.0–1.2 bracket) — all values
// live-verified against minecraft.wiki on 2026-09-06 (see
// docs/research/phase1-1.0-1.2-research.md for the per-claim audit) ----
/// Mycelium — mushroom-fields surface block. Spreads to dirt (1 up /
/// 1 sideways / 3 down, light gates 9/4 — VERIFIED w/Mycelium §Spread).
/// Drops DIRT without Silk Touch (adaptation: no Silk Touch in engine).
pub const MYCELIUM: u8 = 103;
/// End stone — hardness 3, blast resistance 9 (VERIFIED w/End_Stone).
pub const END_STONE: u8 = 104;
/// Nether bricks — the fortress structural block.
pub const NETHER_BRICKS: u8 = 105;
/// Redstone lamp — light 0 when off; the LIT state emits 15. Turns on
/// instantly, off after 4 game ticks (VERIFIED w/Redstone_Lamp: "takes
/// 4 ticks (0.2 seconds) to turn off in Java Edition"; the 1.2.4
/// history note "2-tick delay" = 2 redstone ticks = the same 4 game
/// ticks). Crafted 4 glowstone + 1 redstone dust.
pub const REDSTONE_LAMP: u8 = 106;
/// Chiseled stone bricks — decorative variant (recipe needs stone-brick
/// slabs, out of engine scope; picker-only, documented).
pub const CHISELED_STONE_BRICKS: u8 = 107;
/// Chiseled sandstone (2 sandstone slabs — slabless engine: picker-only).
pub const CHISELED_SANDSTONE: u8 = 108;
/// Cut sandstone — 2×2 sandstone → 4 (craftable).
pub const CUT_SANDSTONE: u8 = 109;
/// Smooth sandstone — smelt sandstone (1.14+ recipe, valid for 1.16.5).
pub const SMOOTH_SANDSTONE: u8 = 110;
/// Huge red mushroom cap block.
pub const MUSHROOM_RED_BLOCK: u8 = 111;
/// Huge brown mushroom cap block.
pub const MUSHROOM_BROWN_BLOCK: u8 = 112;
/// Huge mushroom stem.
pub const MUSHROOM_STEM: u8 = 113;
/// Nether wart crop — 4 stages (age 0..3), 10%/random-tick growth, only
/// on soul sand (VERIFIED w/Nether_Wart). Storage states 237..=240.
pub const NETHER_WART: u8 = 114;
/// Dragon egg — spawns above the End exit portal after the first dragon
/// kill (light level 1).
pub const DRAGON_EGG: u8 = 115;
/// End portal block — the 3×3 active portal in the stronghold room /
/// the End exit portal. Emissive 15. Entering it dimension-travels.
pub const END_PORTAL: u8 = 116;
// ---- Phase E1 item-blocks (inventory-only, the potion pattern) ----
pub const END_CRYSTAL: u8 = 117;
pub const EYE_OF_ENDER: u8 = 118;
pub const BLAZE_ROD: u8 = 119;
pub const BLAZE_POWDER: u8 = 120;
pub const GOLDEN_APPLE: u8 = 121;
pub const SNOWBALL: u8 = 122;
pub const NETHER_BRICK: u8 = 123;
// spawn eggs: ids 124..=143, one per implemented mob kind (20).
/// Vanilla mechanic (VERIFIED w/Spawn_Egg §Usage): use on a surface →
/// the mob spawns with feet adjacent to the surface; the egg is
/// consumed. Creative-picker item.
pub const SPAWN_EGG_BASE: u8 = 124;
pub const SPAWN_EGG_MAX: u8 = 143;
/// spawner mob-kind code 0: zombie (dungeon roll 50%)
pub const SPAWNER_ZOMBIE: u8 = 0;
/// spawner mob-kind code 1: skeleton (dungeon roll 25%)
pub const SPAWNER_SKELETON: u8 = 1;
/// spawner mob-kind code 2: spider (dungeon roll 25%)
pub const SPAWNER_SPIDER: u8 = 2;

// ---- Phase E2 block ids (evolution 1.3–1.4 bracket) — all values
// live-verified 2026-09-06 against minecraft.wiki (see
// docs/research/phase2-1.3-1.4-research.md for the per-claim audit) ----
/// Anvil — gravity block, 3 damage stages (VERIFIED w/Anvil: 12% per use
/// to degrade, falls like sand, 2 HP/block falling damage after the
/// first, cap 40). Craft: 3 iron blocks + 4 iron ingots (engine
/// adaptation: iron ORE items stand in for ingots — no ingot item yet).
pub const ANVIL: u8 = 144;
/// Chipped Anvil — damage stage 1.
pub const CHIPPED_ANVIL: u8 = 145;
/// Damaged Anvil — damage stage 2 (next degrade = destroyed).
pub const DAMAGED_ANVIL: u8 = 146;
/// Beacon — light 15, pyramid 1–4 levels, effects (VERIFIED w/Beacon).
/// Craft: 5 glass + 1 nether star + 3 obsidian.
pub const BEACON: u8 = 147;
/// Cobblestone Wall — 6 cobble → 6 walls (VERIFIED w/Wall); 1.5-block
/// collision like fences; connects to neighbors at mesh time.
pub const COBBLE_WALL: u8 = 148;
/// Ender Chest — 27 slots, per-player, shared across all ender chests
/// (VERIFIED w/Ender_Chest). Craft: 8 obsidian + eye of ender. Light 7.
/// Break drops 8 obsidian (no Silk Touch in engine — documented).
pub const ENDER_CHEST: u8 = 149;
/// Flower Pot — craft 3 bricks (VERIFIED w/Flower_Pot; brick ITEM →
/// brick BLOCK adaptation, documented); hardness 0, instant break.
pub const FLOWER_POT: u8 = 150;
/// Item Frame — craft 8 sticks + 1 leather (VERIFIED w/Item_Frame;
/// stick item absent → planks stand-in, documented). Displays the item
/// placed in it; interact rotates 45°.
pub const ITEM_FRAME: u8 = 151;
/// Tripwire Hook — craft 1 iron + 1 stick + 2 planks → 2 (VERIFIED
/// w/Tripwire_Hook; iron ore + planks adaptation). Pairs + a 1–40
/// string line emit redstone while tripped.
pub const TRIPWIRE_HOOK: u8 = 152;
/// Wither Skeleton Skull — the wither-summon block (2.5% drop, VERIFIED
/// w/Wither_Skeleton); hardness 1.
pub const WITHER_SKELETON_SKULL: u8 = 153;
/// Command Block — creative/`give` only (VERIFIED w/Command_Block);
/// executes the engine command bridge on redstone pulse. Impulse variant
/// (chain/repeating are 1.9 — deferred).
pub const COMMAND_BLOCK: u8 = 154;
// ---- Phase E2 item-blocks (inventory-only, the potion pattern) ----
/// Emerald — ore drop + beacon feed + trade currency (VERIFIED
/// w/Emerald, w/Emerald_Ore: drops 1, XP 3–7).
pub const EMERALD: u8 = 155;
/// Nether Star — the wither's guaranteed drop (VERIFIED w/Wither:
/// 100%, 50 XP, 10-min despawn); beacon ingredient.
pub const NETHER_STAR: u8 = 156;
/// Potato — food 1 / 0.6 (VERIFIED w/Food).
pub const POTATO: u8 = 157;
/// Baked Potato — food 5 / 6.0 (VERIFIED w/Food); smelted from potato.
pub const BAKED_POTATO: u8 = 158;
/// Carrot — food 3 / 3.6 (VERIFIED w/Food).
pub const CARROT: u8 = 159;
/// Pumpkin Pie — food 8 / 4.8 (VERIFIED w/Food). Recipe needs sugar +
/// egg (absent in engine) → picker-only, recipe deferred (documented).
pub const PUMPKIN_PIE: u8 = 160;
/// Lava — light-emitting fluid (VERIFIED w/Lava infobox: luminance 15,
/// transparent, flow distance 4 blocks Overworld/End & 8 Nether —
/// counted including the source, i.e. 3/7 spread; flow speed 30/10
/// ticks per block; contact damage 4 HP per 10 ticks via the damage
/// immunity window). States: source 307 + flow levels 1..7 at 308..=314.
pub const LAVA: u8 = 161;
pub const BEEF_STATE: u16 = 130;
pub const PORKCHOP_STATE: u16 = 131;
pub const MUTTON_STATE: u16 = 132;
pub const CHICKEN_RAW_STATE: u16 = 133;
pub const FEATHER_STATE: u16 = 134;
pub const LEATHER_STATE: u16 = 135;
pub const BONE_STATE: u16 = 136;
pub const STRING_STATE: u16 = 137;
pub const GUNPOWDER_STATE: u16 = 138;
pub const ENDER_PEARL_STATE: u16 = 139;
pub const ROTTEN_FLESH_STATE: u16 = 140;
pub const ARROW_ITEM_STATE: u16 = 141;

// redstone component states (Phase 3, §25 pattern — dedicated states,
// never identity slots). CHUNK `get` truncates to u8, so every state must
// stay ≤ 255 — this layout fits the free window 142..=227:
// * repeater: facing(4) × delay(4: 1..4 redstone ticks) × powered(2) = 32,
//   142..=173 (LOCK is DERIVED from side repeaters at tick time — vanilla
//   persists it, we recompute; same observable behavior)
// * comparator: facing(4) × mode(2) × powered(2) = 16, 174..=189
// * piston/sticky: facing(4 horizontal — vertical pistons deferred) ×
//   extended(2) = 8 each, 190..=197 / 198..=205
// * dispenser/dropper: facing(4) = 4 each, 206..=209 / 210..=213
// * observer: facing(4) × powered(2) = 8, 214..=221
// * hopper: facing(5: down+n/e/s/w) = 5, 222..=226 (ENABLED is derived
// from redstone power at tick time)
// * chest: single state 227
pub const REPEATER_STATE_BASE: u16 = 142;
pub const REPEATER_STATE_END: u16 = 173;
pub const COMPARATOR_STATE_BASE: u16 = 174;
pub const COMPARATOR_STATE_END: u16 = 189;
pub const PISTON_STATE_BASE: u16 = 190;
pub const PISTON_STATE_END: u16 = 197;
pub const STICKY_PISTON_STATE_BASE: u16 = 198;
pub const STICKY_PISTON_STATE_END: u16 = 205;
pub const DISPENSER_STATE_BASE: u16 = 206;
pub const DISPENSER_STATE_END: u16 = 209;
pub const DROPPER_STATE_BASE: u16 = 210;
pub const DROPPER_STATE_END: u16 = 213;
pub const OBSERVER_STATE_BASE: u16 = 214;
pub const OBSERVER_STATE_END: u16 = 221;
pub const HOPPER_STATE_BASE: u16 = 222;
pub const HOPPER_STATE_END: u16 = 226;
pub const CHEST_STATE: u16 = 227;
// Phase 4 §26/§30: potion-of-harming + corruption-chain item states
pub const POTION_HARMING_STATE: u16 = 228;
pub const POTION_HARMING_II_STATE: u16 = 229;
pub const SPIDER_EYE_STATE: u16 = 230;
pub const FERMENTED_EYE_STATE: u16 = 231;
// Phase 5 §27: spawner mob type (0 zombie / 1 skeleton / 2 spider) — the
// vanilla MobSpawner NBT `Entity` rides the block state here instead
// (documented adaptation; 3 states, 232..=234)
pub const SPAWNER_STATE_BASE: u16 = 232;
pub const SPAWNER_STATE_END: u16 = 234;
/// Phase 10: end-portal frame state (single)
pub const END_PORTAL_FRAME_STATE: u16 = 235;
// ---- Phase E1 states ----
// The identity slots of block ids 103..=139 COLLIDE with the legacy sim
// state ranges (wire 96..=111, lever/torch 112..=115, furnace 116..=117,
// nether 118..=120, brewing 121..=127, enchant 128..=129, item states
// 130..=141) — exactly the FURNACE/NETHERRACK pattern. Every new block
// therefore stores one of these DEDICATED states; all world-stored states
// stay ≤ 255 (CHUNK `get` truncates to u8). The 22 item-blocks' states
// live at ≥ 256 — they are NEVER stored in the world (items are
// inventory-only), so the u8 truncation never meets them.

/// redstone lamp, lit (light 15 — read via state_emissive)
pub const REDSTONE_LAMP_LIT: u16 = 236;
/// nether wart crop ages 0..3 (VERIFIED: 4 stages, 10%/random-tick)
pub const WART_STATE_BASE: u16 = 237;
pub const WART_STATE_END: u16 = 240;
/// spawner mob-kind code 3: blaze (fortress platforms — extends the
/// Phase 5 232..=234 zombie/skeleton/spider set)
pub const SPAWNER_BLAZE: u16 = 241;
/// end-portal frame with an eye of ender inserted (activation step)
pub const END_PORTAL_FRAME_EYE: u16 = 242;
// dedicated world-block states (the last free window 243..=255)
pub const NETHER_BRICKS_STATE: u16 = 243;
pub const REDSTONE_LAMP_STATE: u16 = 244;
pub const CHISELED_STONE_BRICKS_STATE: u16 = 245;
pub const CHISELED_SANDSTONE_STATE: u16 = 246;
pub const CUT_SANDSTONE_STATE: u16 = 247;
pub const SMOOTH_SANDSTONE_STATE: u16 = 248;
pub const MUSHROOM_RED_BLOCK_STATE: u16 = 249;
pub const MUSHROOM_BROWN_BLOCK_STATE: u16 = 250;
pub const MUSHROOM_STEM_STATE: u16 = 251;
pub const DRAGON_EGG_STATE: u16 = 252;
pub const END_PORTAL_STATE: u16 = 253;
pub const MYCELIUM_STATE: u16 = 254;
pub const END_STONE_STATE: u16 = 255;
/// item-block states (END_CRYSTAL..=SPAWN_EGG_MAX, ids 117..=143):
/// `state = 256 + (block - 117)` — never stored in chunks.
pub const ITEM_STATE_BASE: u16 = 256;
pub const ITEM_STATE_END: u16 = 282;

// ---- Phase E2 states (the u16 space ≥ 283 — the ≤ 255 window is FULL;
// world storage is u16 sections, so these store fine since the Phase-E2
// Chunk::get widening) ----
/// anvil damage states (chipped/damaged; the pristine anvil has its own)
pub const ANVIL_STATE: u16 = 283;
pub const CHIPPED_ANVIL_STATE: u16 = 284;
pub const DAMAGED_ANVIL_STATE: u16 = 285;
/// beacon (single state)
pub const BEACON_STATE: u16 = 286;
/// cobblestone wall (single state — connections derive at mesh time)
pub const COBBLE_WALL_STATE: u16 = 287;
/// ender chest
pub const ENDER_CHEST_STATE: u16 = 288;
/// flower pot
pub const FLOWER_POT_STATE: u16 = 289;
/// item frame
pub const ITEM_FRAME_STATE: u16 = 290;
/// tripwire hook: facing(4, N/E/S/W) × powered(2) = 8 states 291..=298
pub const TRIPWIRE_HOOK_STATE_BASE: u16 = 291;
pub const TRIPWIRE_HOOK_STATE_END: u16 = 298;
/// wither skeleton skull
pub const WITHER_SKELETON_SKULL_STATE: u16 = 299;
/// command block
pub const COMMAND_BLOCK_STATE: u16 = 300;
/// E2 item-block states (EMERALD..=PUMPKIN_PIE, ids 155..=160):
/// `state = 301 + (block - 155)` — never stored in chunks.
pub const E2_ITEM_STATE_BASE: u16 = 301;
pub const E2_ITEM_STATE_END: u16 = 306;
/// lava source state (level 0)
pub const LAVA_STATE: u16 = 307;
/// lava flow levels 1..7 (the same level ladder as water; the LEVEL DROP
/// per block is dimension-dependent — 2 in the Overworld/End (3 spread),
/// 1 in the Nether (7 spread) — VERIFIED w/Lava flow-distance rows)
pub const LAVA_FLOW_BASE: u16 = 308;
pub const LAVA_FLOW_END: u16 = 314;
/// Phase E2: spawner mob-kind code 4 — wither skeleton (the fortress's
/// second platform; VERIFIED w/Wither_Skeleton: Nether fortresses only)
pub const SPAWNER_WITHER_SKELETON: u16 = 315;

/// lava state for a level (0 = source)
#[inline]
pub fn lava_state(level: u8) -> u16 {
    if level == 0 {
        LAVA_STATE
    } else {
        LAVA_FLOW_BASE + (level.clamp(1, 7)) as u16 - 1
    }
}

/// lava level of a state: 0 = source, 1..7 = flowing, 255 = not lava
#[inline]
pub fn lava_level(s: u16) -> u8 {
    if s == LAVA_STATE {
        0
    } else if (LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s) {
        (s - LAVA_FLOW_BASE + 1) as u8
    } else {
        255
    }
}

/// true if this state is flowing lava (not the source)
#[inline]
pub fn is_lava_flow(s: u16) -> bool {
    (LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s)
}

// E2 tripwire-hook codec: facing 0..3 = N/E/S/W, powered flag
#[inline]
pub fn tripwire_hook_state(facing: usize, powered: bool) -> u16 {
    TRIPWIRE_HOOK_STATE_BASE + (facing.min(3)) as u16 * 2 + (powered as u16)
}

#[inline]
pub fn tripwire_hook_decode(s: u16) -> (usize, bool) {
    let v = s - TRIPWIRE_HOOK_STATE_BASE;
    ((v / 2) as usize, v & 1 != 0)
}

/// E2 item-block arithmetic (ids 155..=160 ↔ states 301..=306)
#[inline]
pub fn e2_item_block_state(b: u8) -> Option<u16> {
    if (EMERALD..=PUMPKIN_PIE).contains(&b) {
        Some(E2_ITEM_STATE_BASE + (b - EMERALD) as u16)
    } else {
        None
    }
}

#[inline]
pub fn e2_item_state_block(s: u16) -> Option<u8> {
    if (E2_ITEM_STATE_BASE..=E2_ITEM_STATE_END).contains(&s) {
        Some(EMERALD + (s - E2_ITEM_STATE_BASE) as u8)
    } else {
        None
    }
}

/// state ↔ item-block arithmetic helpers (item ids 117..=139 ↔ 256..=278)
#[inline]
pub fn item_block_state(b: u8) -> Option<u16> {
    if (END_CRYSTAL..=SPAWN_EGG_MAX).contains(&b) {
        Some(ITEM_STATE_BASE + (b - END_CRYSTAL) as u16)
    } else {
        None
    }
}

#[inline]
pub fn item_state_block(s: u16) -> Option<u8> {
    if (ITEM_STATE_BASE..=ITEM_STATE_END).contains(&s) {
        Some(END_CRYSTAL + (s - ITEM_STATE_BASE) as u8)
    } else {
        None
    }
}

pub const BLOCK_COUNT: usize = 162;

// ---------------------------------------------------------------------------
// BlockState registry (1.16.5 pattern, miniature)
// ---------------------------------------------------------------------------
// State ids are u16 in the paletted sections + VC-16 vertices. States
// 0..=56 are IDENTITY-mapped (each block's default state = its own id), so
// every existing code path that stores/compares u8 block ids keeps working
// unchanged. States 57+ are property variants — today: logs with
// `axis=x|y|z`, exactly how vanilla models oak_log[axis=...]. The 1.16.5
// global palette is 15 bits (~17k states); this registry grows into it
// without touching storage (u16) or the mesher key.
/// total state-id space the registries cover (test-facing bound: every
/// state ≤ STATE_COUNT must fold to a valid block + renderable tiles).
/// Phase 4: extended to cover the Phase 2/3/4 state ids (130..=231) —
/// before, the range tests stopped at 130 and never saw them.
/// Phase 5: spawner states 232..=234
/// Phase E1: lamp/wart/spawner-blaze/frame-eye + dedicated world-block and
/// item states (140..=141, 236..=253, 256..=278)
/// Phase E2: anvil/beacon/wall/ender-chest/frame/tripwire/skull/command
/// + E2 items + eggs 17..=20 + lava (283..=314)
pub const STATE_COUNT: usize = 316;
pub const OAK_LOG_X: u16 = 57;
pub const OAK_LOG_Z: u16 = 58;
pub const BIRCH_LOG_X: u16 = 59;
pub const BIRCH_LOG_Z: u16 = 60;
pub const SPRUCE_LOG_X: u16 = 61;
pub const SPRUCE_LOG_Z: u16 = 62;

// ---------------------------------------------------------------------------
// flowing water states (§24 Fluids, Phase 6)
// ---------------------------------------------------------------------------
// Level 0 (source) is the identity state `WATER`; flowing levels 1..7 are
// SIMULATION states — they mesh through the greedy WATER path (never the
// JSON model dispatch) and fold to block WATER everywhere block ids are
// expected. Save/load round-trips the u16 palette unchanged.
pub const WATER_FLOW_BASE: u16 = 89;
pub const WATER_FLOW_END: u16 = 95; // 7 flowing levels

// ---------------------------------------------------------------------------
// redstone sim states (§25, Phase 6)
// ---------------------------------------------------------------------------
// Wire power 0..15 → states 96..111; lever off/on → 112/113; torch
// lit/unlit → 114/115. NOTE: the wire/torch/lever BLOCK ids (60..62) can
// never be stored as identity states — those state slots belong to the log
// axis variants (57..62). Every placement goes through wire_state() /
// torch_state() / lever_state(); state_block folds everything back. All
// fold to their block ids and never route to the JSON model dispatch
// (is_model_state exempts them like water).
pub const WIRE_POWER_BASE: u16 = 96;
pub const WIRE_POWER_END: u16 = 111; // 16 power levels
pub const LEVER_OFF: u16 = 112;
pub const LEVER_ON: u16 = 113;
pub const TORCH_LIT: u16 = 114;
pub const TORCH_OFF: u16 = 115;
// furnace (Phase 7): unlit/lit sim states — block id 63's identity slot
// collides with the oak-slab model state, so like redstone the furnace
// always stores one of these
pub const FURNACE_STATE: u16 = 116;
pub const FURNACE_LIT: u16 = 117;
// nether blocks (Phase 7 §28): identity slots 64..66 collide with the
// oak-fence model-state range — dedicated states above the sim range
// (see the constants next to the block ids)

#[inline]
pub fn is_wire_power(s: u16) -> bool {
    (WIRE_POWER_BASE..=WIRE_POWER_END).contains(&s)
}

// ------------------------------------------------- Phase 3 state codecs --

/// horizontal facing index 0..3 = north/east/south/west
#[inline]
fn horiz_facing(f: usize) -> [i32; 3] {
    match f {
        0 => [0, 0, -1],
        1 => [1, 0, 0],
        2 => [0, 0, 1],
        _ => [-1, 0, 0],
    }
}

/// full (6-way) facing vector — used by observer/dispenser code that may
/// later grow vertical support; pistons/observers/dispensers v1 are
/// horizontal-only (documented simplification)
#[inline]
fn full_facing(f: usize) -> [i32; 3] {
    match f {
        0 => [0, 0, -1],
        1 => [1, 0, 0],
        2 => [0, 0, 1],
        3 => [-1, 0, 0],
        4 => [0, 1, 0],
        _ => [0, -1, 0],
    }
}

/// repeater state: facing(4) × delay(1..4 redstone ticks) × powered.
/// VERIFIED (wiki blockstate table): "delay 1 2 3 4 ... in redstone
/// ticks (double game ticks)" — 1..4 rt = 2..8 game ticks.
#[inline]
pub fn repeater_state(facing: usize, delay_rt: u8, powered: bool) -> u16 {
    let f = (facing.min(3)) as u16;
    let d = (delay_rt.clamp(1, 4) - 1) as u16;
    REPEATER_STATE_BASE + f * 8 + d * 2 + (powered as u16)
}

/// decode a repeater state → (facing, delay_rt, powered)
#[inline]
pub fn repeater_decode(s: u16) -> (usize, u8, bool) {
    let v = s - REPEATER_STATE_BASE;
    ((v / 8) as usize, (v % 8 / 2 + 1) as u8, v & 1 != 0)
}

/// comparator state: facing(4) × subtract_mode × powered
#[inline]
pub fn comparator_state(facing: usize, subtract: bool, powered: bool) -> u16 {
    COMPARATOR_STATE_BASE + (facing.min(3)) as u16 * 4
        + (subtract as u16) * 2 + (powered as u16)
}

#[inline]
pub fn comparator_decode(s: u16) -> (usize, bool, bool) {
    let v = s - COMPARATOR_STATE_BASE;
    ((v / 4) as usize, v & 2 != 0, v & 1 != 0)
}

/// piston state: facing(4 horizontal) × extended
#[inline]
pub fn piston_state(facing: usize, extended: bool) -> u16 {
    PISTON_STATE_BASE + (facing.min(3)) as u16 * 2 + (extended as u16)
}

#[inline]
pub fn piston_decode(s: u16) -> (usize, bool) {
    let v = s - PISTON_STATE_BASE;
    ((v / 2) as usize, v & 1 != 0)
}

#[inline]
pub fn sticky_piston_state(facing: usize, extended: bool) -> u16 {
    STICKY_PISTON_STATE_BASE + (facing.min(3)) as u16 * 2 + (extended as u16)
}

#[inline]
pub fn sticky_piston_decode(s: u16) -> (usize, bool) {
    let v = s - STICKY_PISTON_STATE_BASE;
    ((v / 2) as usize, v & 1 != 0)
}

#[inline]
pub fn dispenser_state(facing: usize) -> u16 {
    DISPENSER_STATE_BASE + (facing.min(3)) as u16
}

#[inline]
pub fn dispenser_decode(s: u16) -> usize {
    (s - DISPENSER_STATE_BASE) as usize
}

#[inline]
pub fn dropper_state(facing: usize) -> u16 {
    DROPPER_STATE_BASE + (facing.min(3)) as u16
}

#[inline]
pub fn dropper_decode(s: u16) -> usize {
    (s - DROPPER_STATE_BASE) as usize
}

/// observer state: facing(4) × powered (the 2-game-tick pulse)
#[inline]
pub fn observer_state(facing: usize, powered: bool) -> u16 {
    OBSERVER_STATE_BASE + (facing.min(3)) as u16 * 2 + (powered as u16)
}

#[inline]
pub fn observer_decode(s: u16) -> (usize, bool) {
    let v = s - OBSERVER_STATE_BASE;
    ((v / 2) as usize, v & 1 != 0)
}

/// hopper state: facing(5: 0=down, 1..4 = n/e/s/w). ENABLED is derived
/// from redstone power at tick time (vanilla persists it; deriving is
/// observably identical) — v1 transfers push down only.
#[inline]
pub fn hopper_state(facing: usize) -> u16 {
    HOPPER_STATE_BASE + (facing.min(4)) as u16
}

#[inline]
pub fn hopper_decode(s: u16) -> usize {
    (s - HOPPER_STATE_BASE) as usize
}

// ---- Phase 5 §27: spawner mob-type codec ----
// The vanilla MobSpawner stores its entity in NBT (`SpawnData.Entity.id`);
// we encode the mob kind in the 3-state window instead — same observable
// behavior, no extra persistence channel.

/// spawner state for a mob kind (SPAWNER_ZOMBIE / SKELETON / SPIDER)
#[inline]
pub fn spawner_state(mob: u8) -> u16 {
    SPAWNER_STATE_BASE + (mob.min(SPAWNER_SPIDER)) as u16
}

/// mob kind of a spawner state (0 zombie / 1 skeleton / 2 spider)
#[inline]
pub fn spawner_mob(s: u16) -> u8 {
    if (SPAWNER_STATE_BASE..=SPAWNER_STATE_END).contains(&s) {
        (s - SPAWNER_STATE_BASE) as u8
    } else {
        SPAWNER_ZOMBIE
    }
}

/// horizontal facing vector for a decoded facing index
#[inline]
pub fn horiz_facing_vec(f: usize) -> [i32; 3] {
    horiz_facing(f)
}

/// full (6-way) facing vector for a decoded facing index
#[inline]
pub fn full_facing_vec(f: usize) -> [i32; 3] {
    full_facing(f)
}

/// wire power of a state: 0..15, 255 = not wire
#[inline]
pub fn wire_power(s: u16) -> u8 {
    if is_wire_power(s) {
        (s - WIRE_POWER_BASE) as u8
    } else {
        255
    }
}

#[inline]
pub fn wire_state(power: u8) -> u16 {
    WIRE_POWER_BASE + power.min(15) as u16
}

#[inline]
pub fn lever_state(on: bool) -> u16 {
    if on { LEVER_ON } else { LEVER_OFF }
}

#[inline]
pub fn lever_is_on(s: u16) -> bool {
    s == LEVER_ON
}

#[inline]
pub fn torch_state(lit: bool) -> u16 {
    if lit { TORCH_LIT } else { TORCH_OFF }
}

#[inline]
pub fn torch_is_lit(s: u16) -> bool {
    s == TORCH_LIT
}

/// default STATE id for a freshly placed block — sim blocks (wire/torch/
/// lever/furnace) and nether blocks never store their identity state:
/// those slots belong to model variants / log axes (FURNACE=63 is exactly
/// MODEL_STATE_BASE, so a raw identity placement would render as the
/// oak-slab model!). Prop blocks map to their FIRST state (all properties
/// at their first value — the vanilla default-state rule) because their
/// identity ids (57..59) collide with the log-axis states.
#[inline]
pub fn default_state(b: u8) -> u16 {
    match b {
        REDSTONE_WIRE => wire_state(0),
        REDSTONE_TORCH => torch_state(true),
        LEVER => lever_state(false),
        FURNACE => FURNACE_STATE,
        NETHERRACK => NETHERRACK_STATE,
        NETHER_QUARTZ_ORE => QUARTZ_ORE_STATE,
        SOUL_SAND => SOUL_SAND_STATE,
        BREWING_STAND => BREWING_STAND_STATE,
        POTION_EMPTY => POTION_EMPTY_STATE,
        POTION_WATER => POTION_WATER_STATE,
        POTION_AWKWARD => POTION_AWKWARD_STATE,
        POTION_MUNDANE => POTION_MUNDANE_STATE,
        POTION_HEALING => POTION_HEALING_STATE,
        POTION_HEALING_II => POTION_HEALING_II_STATE,
        POTION_HARMING => POTION_HARMING_STATE,
        POTION_HARMING_II => POTION_HARMING_II_STATE,
        SPIDER_EYE => SPIDER_EYE_STATE,
        FERMENTED_SPIDER_EYE => FERMENTED_EYE_STATE,
        // default spawner = zombie (the 50% dungeon roll)
        SPAWNER => SPAWNER_STATE_BASE,
        END_PORTAL_FRAME => END_PORTAL_FRAME_STATE,
        // ---- Phase E1 defaults (dedicated states — identity slots of
        // 103..=139 collide with the legacy sim-state ranges) ----
        MYCELIUM => MYCELIUM_STATE,
        END_STONE => END_STONE_STATE,
        NETHER_BRICKS => NETHER_BRICKS_STATE,
        REDSTONE_LAMP => REDSTONE_LAMP_STATE,
        CHISELED_STONE_BRICKS => CHISELED_STONE_BRICKS_STATE,
        CHISELED_SANDSTONE => CHISELED_SANDSTONE_STATE,
        CUT_SANDSTONE => CUT_SANDSTONE_STATE,
        SMOOTH_SANDSTONE => SMOOTH_SANDSTONE_STATE,
        MUSHROOM_RED_BLOCK => MUSHROOM_RED_BLOCK_STATE,
        MUSHROOM_BROWN_BLOCK => MUSHROOM_BROWN_BLOCK_STATE,
        MUSHROOM_STEM => MUSHROOM_STEM_STATE,
        NETHER_WART => WART_STATE_BASE, // age 0 (VERIFIED 4 stages)
        DRAGON_EGG => DRAGON_EGG_STATE,
        END_PORTAL => END_PORTAL_STATE,
        ENCHANT_TABLE => ENCHANT_TABLE_STATE,
        ENCHANTED_BOOK => ENCHANTED_BOOK_STATE,
        BEEF => BEEF_STATE,
        PORKCHOP => PORKCHOP_STATE,
        MUTTON => MUTTON_STATE,
        CHICKEN_RAW => CHICKEN_RAW_STATE,
        FEATHER => FEATHER_STATE,
        LEATHER => LEATHER_STATE,
        BONE => BONE_STATE,
        STRING => STRING_STATE,
        GUNPOWDER => GUNPOWDER_STATE,
        ENDER_PEARL => ENDER_PEARL_STATE,
        ROTTEN_FLESH => ROTTEN_FLESH_STATE,
        ARROW_ITEM => ARROW_ITEM_STATE,
        REPEATER => repeater_state(0, 1, false),
        COMPARATOR => comparator_state(0, false, false),
        PISTON => piston_state(0, false),
        STICKY_PISTON => sticky_piston_state(0, false),
        DISPENSER => dispenser_state(0),
        DROPPER => dropper_state(0),
        OBSERVER => observer_state(0, false),
        HOPPER => hopper_state(0),
        CHEST => CHEST_STATE,
        // ---- Phase E2 defaults (dedicated states ≥ 283; identity slots of
        // 144..=160 collide with the E1 item-state range) ----
        ANVIL => ANVIL_STATE,
        CHIPPED_ANVIL => CHIPPED_ANVIL_STATE,
        DAMAGED_ANVIL => DAMAGED_ANVIL_STATE,
        BEACON => BEACON_STATE,
        COBBLE_WALL => COBBLE_WALL_STATE,
        ENDER_CHEST => ENDER_CHEST_STATE,
        FLOWER_POT => FLOWER_POT_STATE,
        ITEM_FRAME => ITEM_FRAME_STATE,
        TRIPWIRE_HOOK => tripwire_hook_state(0, false),
        WITHER_SKELETON_SKULL => WITHER_SKELETON_SKULL_STATE,
        COMMAND_BLOCK => COMMAND_BLOCK_STATE,
        LAVA => LAVA_STATE,
        OAK_SLAB => 63,     // PROP_BLOCKS[0].base_state (half=bottom)
        COBBLE_STAIRS => 65, // base_state (facing=north, half=bottom)
        OAK_FENCE => 73,    // base_state (no connections)
        // Phase E1 item-blocks: dedicated states ≥ 256 (never world-stored)
        // Phase E2 item-blocks: dedicated states 301..=306 (never world-stored)
        _ => item_block_state(b)
            .or_else(|| e2_item_block_state(b))
            .unwrap_or(b as u16),
    }
}

/// true if a state id is a flowing-water level (not the source)
#[inline]
pub fn is_water_flow(s: u16) -> bool {
    (WATER_FLOW_BASE..=WATER_FLOW_END).contains(&s)
}

/// water level of a state: 0 = source, 1..7 = flowing, 255 = not water
#[inline]
pub fn water_level(s: u16) -> u8 {
    if s == WATER as u16 {
        0
    } else if is_water_flow(s) {
        (s - WATER_FLOW_BASE + 1) as u8
    } else {
        255
    }
}

/// state id for a water level (0 = source)
#[inline]
pub fn water_state(level: u8) -> u16 {
    if level == 0 {
        WATER as u16
    } else {
        (WATER_FLOW_BASE + level.min(7) as u16 - 1) as u16
    }
}

// ---------------------------------------------------------------------------
// property-driven states (Phase 1, Master Spec §5.1)
// ---------------------------------------------------------------------------
// Blocks whose variants come from compact PROPERTY DEFINITIONS instead of
// hand-listed ids. State ids follow the vanilla assignment algorithm
// (research R2, VERIFIED): properties sorted alphabetically by name,
// mixed-radix index, last-sorted property varies fastest. These states map
// to JSON models via the blockstate dispatch in model.rs.

/// one blockstate property (compact definition, not per-block boilerplate)
#[derive(Clone, Copy, Debug)]
pub struct PropDef {
    pub name: &'static str,
    /// value strings in declaration order (index = radix digit)
    pub values: &'static [&'static str],
}

/// a block with property-driven states
pub struct PropBlock {
    pub block: u8,
    /// registry name → blockstates/<name>.json
    pub name: &'static str,
    /// sorted alphabetically (vanilla state order)
    pub props: &'static [PropDef],
    /// first state id of this block's range
    pub base_state: u16,
    /// number of states (product of radixes)
    pub state_count: u16,
}

// new blocks (Phase 1): ids continue after the flat registry
pub const OAK_SLAB: u8 = 57;
pub const COBBLE_STAIRS: u8 = 58;
pub const OAK_FENCE: u8 = 59;

pub const HALF: PropDef = PropDef { name: "half", values: &["bottom", "top"] };
pub const FACING: PropDef =
    PropDef { name: "facing", values: &["north", "east", "south", "west"] };
pub const EAST_B: PropDef = PropDef { name: "east", values: &["false", "true"] };
pub const NORTH_B: PropDef = PropDef { name: "north", values: &["false", "true"] };
pub const SOUTH_B: PropDef = PropDef { name: "south", values: &["false", "true"] };
pub const WEST_B: PropDef = PropDef { name: "west", values: &["false", "true"] };

/// property-driven blocks, states starting at MODEL_STATE_BASE
pub const MODEL_STATE_BASE: u16 = 63;
pub const PROP_BLOCKS: [PropBlock; 3] = [
    PropBlock {
        block: OAK_SLAB,
        name: "oak_slab",
        props: &[HALF],
        base_state: 63,
        state_count: 2,
    },
    PropBlock {
        block: COBBLE_STAIRS,
        name: "cobblestone_stairs",
        // sorted: facing (radix 4, slower) × half (radix 2, fastest)
        props: &[FACING, HALF],
        base_state: 65,
        state_count: 8,
    },
    PropBlock {
        block: OAK_FENCE,
        name: "oak_fence",
        // sorted: east (slowest) × north × south × west (fastest)
        props: &[EAST_B, NORTH_B, SOUTH_B, WEST_B],
        base_state: 73,
        state_count: 16,
    },
];

/// decode a property state id → (block, [(prop, value)])
#[inline]
pub fn prop_state_decode(s: u16) -> Option<(u8, Vec<(&'static str, &'static str)>)> {
    if s < MODEL_STATE_BASE {
        return None;
    }
    for pb in PROP_BLOCKS.iter() {
        let off = s - pb.base_state;
        if off < pb.state_count {
            let mut idx = off as usize;
            let mut out: Vec<(&str, &str)> = Vec::with_capacity(pb.props.len());
            for p in pb.props.iter().rev() {
                let radix = p.values.len().max(1);
                out.push((p.name, p.values[idx % radix]));
                idx /= radix;
            }
            out.reverse();
            return Some((pb.block, out));
        }
    }
    None
}

/// find a prop-block's state id from a property assignment (missing props →
/// their first value, vanilla default-state pattern)
#[inline]
pub fn prop_state_encode(block: u8, set: &[(&str, &str)]) -> Option<u16> {
    let pb = PROP_BLOCKS.iter().find(|pb| pb.block == block)?;
    let mut idx = 0usize;
    for (i, p) in pb.props.iter().enumerate() {
        let v = set
            .iter()
            .find(|(k, _)| *k == p.name)
            .map(|(_, v)| *v)
            .unwrap_or(p.values[0]);
        let digit = p.values.iter().position(|&c| c == v).unwrap_or(0);
        // radix products: earlier props are slower
        let radix_after: usize = pb.props[i + 1..]
            .iter()
            .map(|q| q.values.len().max(1))
            .product();
        idx += digit * radix_after;
    }
    Some(pb.base_state + idx as u16)
}

/// state id → owning block id (property variants fold to their parent)
#[inline]
pub fn state_block(s: u16) -> u8 {
    if is_water_flow(s) {
        return WATER;
    }
    if is_wire_power(s) {
        return REDSTONE_WIRE;
    }
    match s {
        LEVER_OFF | LEVER_ON => return LEVER,
        TORCH_LIT | TORCH_OFF => return REDSTONE_TORCH,
        FURNACE_STATE | FURNACE_LIT => return FURNACE,
        NETHERRACK_STATE => return NETHERRACK,
        QUARTZ_ORE_STATE => return NETHER_QUARTZ_ORE,
        SOUL_SAND_STATE => return SOUL_SAND,
        BREWING_STAND_STATE => return BREWING_STAND,
        POTION_EMPTY_STATE => return POTION_EMPTY,
        POTION_WATER_STATE => return POTION_WATER,
        POTION_AWKWARD_STATE => return POTION_AWKWARD,
        POTION_MUNDANE_STATE => return POTION_MUNDANE,
        POTION_HEALING_STATE => return POTION_HEALING,
        POTION_HEALING_II_STATE => return POTION_HEALING_II,
        POTION_HARMING_STATE => return POTION_HARMING,
        POTION_HARMING_II_STATE => return POTION_HARMING_II,
        SPIDER_EYE_STATE => return SPIDER_EYE,
        FERMENTED_EYE_STATE => return FERMENTED_SPIDER_EYE,
        s if (SPAWNER_STATE_BASE..=SPAWNER_STATE_END).contains(&s) => return SPAWNER,
        END_PORTAL_FRAME_STATE => return END_PORTAL_FRAME,
        // ---- Phase E1 state folding ----
        REDSTONE_LAMP_LIT | REDSTONE_LAMP_STATE => return REDSTONE_LAMP,
        s if (WART_STATE_BASE..=WART_STATE_END).contains(&s) => return NETHER_WART,
        SPAWNER_BLAZE => return SPAWNER,
        END_PORTAL_FRAME_EYE => return END_PORTAL_FRAME,
        MYCELIUM_STATE => return MYCELIUM,
        END_STONE_STATE => return END_STONE,
        NETHER_BRICKS_STATE => return NETHER_BRICKS,
        CHISELED_STONE_BRICKS_STATE => return CHISELED_STONE_BRICKS,
        CHISELED_SANDSTONE_STATE => return CHISELED_SANDSTONE,
        CUT_SANDSTONE_STATE => return CUT_SANDSTONE,
        SMOOTH_SANDSTONE_STATE => return SMOOTH_SANDSTONE,
        MUSHROOM_RED_BLOCK_STATE => return MUSHROOM_RED_BLOCK,
        MUSHROOM_BROWN_BLOCK_STATE => return MUSHROOM_BROWN_BLOCK,
        MUSHROOM_STEM_STATE => return MUSHROOM_STEM,
        DRAGON_EGG_STATE => return DRAGON_EGG,
        END_PORTAL_STATE => return END_PORTAL,
        s if (ITEM_STATE_BASE..=ITEM_STATE_END).contains(&s) => {
            return item_state_block(s).unwrap_or(AIR)
        }
        // ---- Phase E2 state folding ----
        ANVIL_STATE => return ANVIL,
        CHIPPED_ANVIL_STATE => return CHIPPED_ANVIL,
        DAMAGED_ANVIL_STATE => return DAMAGED_ANVIL,
        BEACON_STATE => return BEACON,
        COBBLE_WALL_STATE => return COBBLE_WALL,
        ENDER_CHEST_STATE => return ENDER_CHEST,
        FLOWER_POT_STATE => return FLOWER_POT,
        ITEM_FRAME_STATE => return ITEM_FRAME,
        s if (TRIPWIRE_HOOK_STATE_BASE..=TRIPWIRE_HOOK_STATE_END).contains(&s) => {
            return TRIPWIRE_HOOK
        }
        WITHER_SKELETON_SKULL_STATE => return WITHER_SKELETON_SKULL,
        COMMAND_BLOCK_STATE => return COMMAND_BLOCK,
        s if (E2_ITEM_STATE_BASE..=E2_ITEM_STATE_END).contains(&s) => {
            return e2_item_state_block(s).unwrap_or(AIR)
        }
        // lava source + flows fold to LAVA (the water-flow pattern)
        LAVA_STATE => return LAVA,
        s if (LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s) => return LAVA,
        SPAWNER_WITHER_SKELETON => return SPAWNER,
        ENCHANT_TABLE_STATE => return ENCHANT_TABLE,
        ENCHANTED_BOOK_STATE => return ENCHANTED_BOOK,
        BEEF_STATE => return BEEF,
        PORKCHOP_STATE => return PORKCHOP,
        MUTTON_STATE => return MUTTON,
        CHICKEN_RAW_STATE => return CHICKEN_RAW,
        FEATHER_STATE => return FEATHER,
        LEATHER_STATE => return LEATHER,
        BONE_STATE => return BONE,
        STRING_STATE => return STRING,
        GUNPOWDER_STATE => return GUNPOWDER,
        ENDER_PEARL_STATE => return ENDER_PEARL,
        ROTTEN_FLESH_STATE => return ROTTEN_FLESH,
        ARROW_ITEM_STATE => return ARROW_ITEM,
        s if (REPEATER_STATE_BASE..=REPEATER_STATE_END).contains(&s) => return REPEATER,
        s if (COMPARATOR_STATE_BASE..=COMPARATOR_STATE_END).contains(&s) => return COMPARATOR,
        s if (PISTON_STATE_BASE..=PISTON_STATE_END).contains(&s) => return PISTON,
        s if (STICKY_PISTON_STATE_BASE..=STICKY_PISTON_STATE_END).contains(&s) => return STICKY_PISTON,
        s if (DISPENSER_STATE_BASE..=DISPENSER_STATE_END).contains(&s) => return DISPENSER,
        s if (DROPPER_STATE_BASE..=DROPPER_STATE_END).contains(&s) => return DROPPER,
        s if (OBSERVER_STATE_BASE..=OBSERVER_STATE_END).contains(&s) => return OBSERVER,
        s if (HOPPER_STATE_BASE..=HOPPER_STATE_END).contains(&s) => return HOPPER,
        CHEST_STATE => return CHEST,
        _ => {}
    }
    if let Some((b, _)) = prop_state_decode(s) {
        return b;
    }
    match s {
        OAK_LOG_X | OAK_LOG_Z => OAK_LOG,
        BIRCH_LOG_X | BIRCH_LOG_Z => BIRCH_LOG,
        SPRUCE_LOG_X | SPRUCE_LOG_Z => SPRUCE_LOG,
        _ => s as u8, // identity for 0..=56
    }
}

/// vanilla-style state description for F3: "Oak Slab[half=top]"
#[inline]
pub fn state_description(s: u16) -> String {
    // Phase E1 properties (lit lamp / wart age / frame eye) first
    if s == REDSTONE_LAMP_LIT {
        return "Redstone Lamp[lit=true]".into();
    }
    if (WART_STATE_BASE..=WART_STATE_END).contains(&s) {
        return format!("Nether Wart[age={}]", s - WART_STATE_BASE);
    }
    if s == END_PORTAL_FRAME_EYE {
        return "End Portal Frame[eye=true]".into();
    }
    if let Some((b, props)) = prop_state_decode(s) {
        if props.is_empty() {
            return name(b).to_string();
        }
        let inner: Vec<String> = props.iter().map(|(k, v)| format!("{k}={v}")).collect();
        format!("{}[{}]", name(b), inner.join(","))
    } else {
        let axis = match s {
            OAK_LOG_X | BIRCH_LOG_X | SPRUCE_LOG_X => "[axis=x]",
            OAK_LOG_Z | BIRCH_LOG_Z | SPRUCE_LOG_Z => "[axis=z]",
            _ => "",
        };
        // fold log-variant states to their owning block for the name
        format!("{}{}", name(state_block(s)), axis)
    }
}

/// true if this state renders through the JSON-model path (mesher dispatch)
#[inline]
pub fn is_model_state(s: u16) -> bool {
    s >= MODEL_STATE_BASE
        && !is_water_flow(s)
        && !is_wire_power(s)
        && !matches!(
            s,
            LEVER_OFF | LEVER_ON | TORCH_LIT | TORCH_OFF | FURNACE_STATE | FURNACE_LIT
                | NETHERRACK_STATE | QUARTZ_ORE_STATE | SOUL_SAND_STATE
                | BREWING_STAND_STATE
                | POTION_EMPTY_STATE | POTION_WATER_STATE | POTION_AWKWARD_STATE
                | POTION_MUNDANE_STATE | POTION_HEALING_STATE | POTION_HEALING_II_STATE
                | ENCHANT_TABLE_STATE | ENCHANTED_BOOK_STATE
                | BEEF_STATE | PORKCHOP_STATE | MUTTON_STATE | CHICKEN_RAW_STATE
                | FEATHER_STATE | LEATHER_STATE | BONE_STATE | STRING_STATE
                | GUNPOWDER_STATE | ENDER_PEARL_STATE | ROTTEN_FLESH_STATE
                | ARROW_ITEM_STATE
                | POTION_HARMING_STATE | POTION_HARMING_II_STATE
                | SPIDER_EYE_STATE | FERMENTED_EYE_STATE
        ) && !((REPEATER_STATE_BASE..=REPEATER_STATE_END).contains(&s)
            || (COMPARATOR_STATE_BASE..=COMPARATOR_STATE_END).contains(&s)
            || (PISTON_STATE_BASE..=PISTON_STATE_END).contains(&s)
            || (STICKY_PISTON_STATE_BASE..=STICKY_PISTON_STATE_END).contains(&s)
            || (DISPENSER_STATE_BASE..=DISPENSER_STATE_END).contains(&s)
            || (DROPPER_STATE_BASE..=DROPPER_STATE_END).contains(&s)
            || (OBSERVER_STATE_BASE..=OBSERVER_STATE_END).contains(&s)
            || (HOPPER_STATE_BASE..=HOPPER_STATE_END).contains(&s)
            || (SPAWNER_STATE_BASE..=SPAWNER_STATE_END).contains(&s)
            || s == CHEST_STATE
            || s == END_PORTAL_FRAME_STATE)
        && !matches!(
            s,
            REDSTONE_LAMP_LIT | REDSTONE_LAMP_STATE | SPAWNER_BLAZE | END_PORTAL_FRAME_EYE
                | MYCELIUM_STATE | END_STONE_STATE | NETHER_BRICKS_STATE
                | CHISELED_STONE_BRICKS_STATE | CHISELED_SANDSTONE_STATE | CUT_SANDSTONE_STATE
                | SMOOTH_SANDSTONE_STATE | MUSHROOM_RED_BLOCK_STATE | MUSHROOM_BROWN_BLOCK_STATE
                | MUSHROOM_STEM_STATE | DRAGON_EGG_STATE | END_PORTAL_STATE
        )
        && !((WART_STATE_BASE..=WART_STATE_END).contains(&s))
        && !((ITEM_STATE_BASE..=ITEM_STATE_END).contains(&s))
        // ---- Phase E2: every E2 state is a cross/full-cube/sim state —
        // none routes to the JSON-model dispatcher ----
        && !matches!(
            s,
            ANVIL_STATE | CHIPPED_ANVIL_STATE | DAMAGED_ANVIL_STATE | BEACON_STATE
                | COBBLE_WALL_STATE | ENDER_CHEST_STATE | FLOWER_POT_STATE | ITEM_FRAME_STATE
                | WITHER_SKELETON_SKULL_STATE | COMMAND_BLOCK_STATE
        )
        && !((TRIPWIRE_HOOK_STATE_BASE..=TRIPWIRE_HOOK_STATE_END).contains(&s))
        && !((E2_ITEM_STATE_BASE..=E2_ITEM_STATE_END).contains(&s))
        // lava source/flows ride the fluid-quad path (never models)
        && s != LAVA_STATE
        && !((LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s))
        && s != SPAWNER_WITHER_SKELETON
}

/// true if this block id has property-driven model states
#[inline]
pub fn is_model_block(b: u8) -> bool {
    b >= OAK_SLAB && b <= OAK_FENCE
}

/// per-state tiles: [top(+Y), bottom(−Y), side_x(±X), side_z(±Z)].
/// Vanilla logs show the ring texture on the ±axis faces and bark on the
/// rest — the axis property drives the tile rotation.
#[inline]
pub fn state_tiles(s: u16) -> [u16; 4] {
    match s {
        OAK_LOG_X => [TILE_LOG_SIDE, TILE_LOG_SIDE, TILE_LOG_TOP, TILE_LOG_SIDE],
        OAK_LOG_Z => [TILE_LOG_SIDE, TILE_LOG_SIDE, TILE_LOG_SIDE, TILE_LOG_TOP],
        BIRCH_LOG_X => [TILE_BIRCH_LOG_SIDE, TILE_BIRCH_LOG_SIDE, TILE_LOG_TOP, TILE_BIRCH_LOG_SIDE],
        BIRCH_LOG_Z => [TILE_BIRCH_LOG_SIDE, TILE_BIRCH_LOG_SIDE, TILE_BIRCH_LOG_SIDE, TILE_LOG_TOP],
        SPRUCE_LOG_X => [TILE_SPRUCE_LOG_SIDE, TILE_SPRUCE_LOG_SIDE, TILE_LOG_TOP, TILE_SPRUCE_LOG_SIDE],
        SPRUCE_LOG_Z => [TILE_SPRUCE_LOG_SIDE, TILE_SPRUCE_LOG_SIDE, TILE_SPRUCE_LOG_SIDE, TILE_LOG_TOP],
        // §27: lit furnace swaps the SIDE tiles to the glowing variant
        FURNACE_LIT => [
            TILE_FURNACE_TOP,
            TILE_FURNACE_TOP,
            TILE_FURNACE_LIT_SIDE,
            TILE_FURNACE_LIT_SIDE,
        ],
        // Phase E1: the lit lamp swaps every face to the glowing tile
        REDSTONE_LAMP_LIT => [
            TILE_REDSTONE_LAMP_ON,
            TILE_REDSTONE_LAMP_ON,
            TILE_REDSTONE_LAMP_ON,
            TILE_REDSTONE_LAMP_ON,
        ],
        // Phase E1: nether-wart crop ages (4 stages, VERIFIED)
        s if (WART_STATE_BASE..=WART_STATE_END).contains(&s) => {
            let off = s - WART_STATE_BASE;
            let t = match off {
                0 => TILE_NETHER_WART_0,
                1 => TILE_NETHER_WART_1,
                2 => TILE_NETHER_WART_2,
                _ => TILE_NETHER_WART_3,
            };
            [t, t, t, t]
        }
        // Phase E1: frame with an eye shows the filled inset
        END_PORTAL_FRAME_EYE => {
            // same tile for now — the eye state is functional (activation),
            // the art carries the inset; hotbar shows the frame face
            [TILE_END_PORTAL_FRAME, TILE_END_PORTAL_FRAME, TILE_END_PORTAL_FRAME, TILE_END_PORTAL_FRAME]
        }
        // Phase E2: anvil damage stages swap the face tile (chipped shows a
        // cracked face; damaged shows a broken face — VERIFIED w/Anvil
        // "gradually becomes chipped, then damaged, then breaks")
        CHIPPED_ANVIL_STATE => {
            [TILE_ANVIL_CHIPPED, TILE_ANVIL_CHIPPED, TILE_ANVIL_CHIPPED, TILE_ANVIL_CHIPPED]
        }
        DAMAGED_ANVIL_STATE => {
            [TILE_ANVIL_DAMAGED, TILE_ANVIL_DAMAGED, TILE_ANVIL_DAMAGED, TILE_ANVIL_DAMAGED]
        }
        // Phase E2: powered tripwire hook glows red (hook + trip state)
        s if (TRIPWIRE_HOOK_STATE_BASE..=TRIPWIRE_HOOK_STATE_END).contains(&s) => {
            let (_, powered) = tripwire_hook_decode(s);
            let t = if powered { TILE_TRIPWIRE_HOOK_ON } else { TILE_TRIPWIRE_HOOK };
            [t, t, t, t]
        }
        _ => {
            // fold property states to their block (model geometry supplies
            // the real tiles; these are for the HUD/hotbar blit path)
            let b = state_block(s);
            let d = def(b);
            [d.tiles[0], d.tiles[1], d.tiles[2], d.tiles[2]]
        }
    }
}

#[inline]
pub fn is_log(b: u8) -> bool {
    b == OAK_LOG || b == BIRCH_LOG || b == SPRUCE_LOG
}

/// state for placing a log with the given axis (0=X, 1=Y, 2=Z).
/// Vanilla placement rule: the log's axis follows the clicked face.
#[inline]
pub fn log_axis_state(block: u8, axis: u8) -> u16 {
    match (block, axis) {
        (OAK_LOG, 0) => OAK_LOG_X,
        (OAK_LOG, 2) => OAK_LOG_Z,
        (BIRCH_LOG, 0) => BIRCH_LOG_X,
        (BIRCH_LOG, 2) => BIRCH_LOG_Z,
        (SPRUCE_LOG, 0) => SPRUCE_LOG_X,
        (SPRUCE_LOG, 2) => SPRUCE_LOG_Z,
        _ => block as u16,
    }
}

/// highest tile index the generator must draw. Phase 4 BUG FIX: this sat
/// at 82 while Phase 2 (tiles 83–104) and Phase 3 (tiles 105–117) kept
/// adding art arms ABOVE it — the atlas loop `for t in 0..=TILE_MAX`
/// never reached them, so every mob sprite, mob-drop icon, and redstone
/// component tile rendered BLANK since Phase 2. Now derived from the
/// highest tile constant (118–121 here) and guarded by the
/// `all_def_tiles_within_tile_max` test so it can never drift again.
pub const TILE_MAX: u16 = 205;

/// inventory-only ITEM blocks (potions/bottles/books): never placeable in
/// the world — right-click drinks (potions) / fills (glass bottle at water).
#[inline]
pub fn is_item_block(b: u8) -> bool {
    matches!(
        b,
        POTION_EMPTY | POTION_WATER | POTION_AWKWARD | POTION_MUNDANE | POTION_HEALING
            | POTION_HEALING_II | POTION_HARMING | POTION_HARMING_II | ENCHANTED_BOOK
            | SPIDER_EYE | FERMENTED_SPIDER_EYE
            | BEEF | PORKCHOP | MUTTON | CHICKEN_RAW | FEATHER | LEATHER | BONE | STRING
            | GUNPOWDER | ENDER_PEARL | ROTTEN_FLESH | ARROW_ITEM
            | END_CRYSTAL | EYE_OF_ENDER | BLAZE_ROD | BLAZE_POWDER | GOLDEN_APPLE
            | SNOWBALL | NETHER_BRICK
            // Phase E2 items (evolution 1.3-1.4)
            | EMERALD | NETHER_STAR | POTATO | BAKED_POTATO | CARROT | PUMPKIN_PIE
    ) || is_spawn_egg(b)
}

/// true for the 20 mob spawn-egg item ids (124..=143).
#[inline]
pub fn is_spawn_egg(b: u8) -> bool {
    (SPAWN_EGG_BASE..=SPAWN_EGG_MAX).contains(&b)
}

/// The mob this spawn-egg id spawns. Tile order in the BLOCK_TABLE egg
/// rows MUST match this mapping (guarded by the egg roundtrip test).
/// The egg ids follow the Phase-2/Phase-E1 MobKind discriminant order
/// (see vc_gameplay::mobs::MobKind and the egg_art palette table).
#[inline]
pub fn egg_mob(b: u8) -> Option<u8> {
    if !is_spawn_egg(b) {
        return None;
    }
    Some(b - SPAWN_EGG_BASE) // 0..=15, decoded by the gameplay layer
}

/// Phase E1: state-aware emission — the lit redstone lamp state emits 15
/// even though the lamp BLOCK id (off) emits 0. Call sites pass the raw
/// stored state; everything else folds to the block table value.
/// VERIFIED w/Redstone_Lamp: "An active redstone lamp produces block light
/// level 15. An inactive redstone lamp produces no light."
#[inline]
pub fn state_emissive(s: u16) -> u8 {
    if s == REDSTONE_LAMP_LIT {
        return 15;
    }
    emissive(state_block(s) as u8)
}

/// Phase E1: nether-wart crop age (0..3) from its storage state.
#[inline]
pub fn wart_age(s: u16) -> u8 {
    if (WART_STATE_BASE..=WART_STATE_END).contains(&s) {
        (s - WART_STATE_BASE) as u8
    } else if state_block(s) == NETHER_WART {
        0
    } else {
        0
    }
}

pub struct BlockDef {
    pub name: &'static str,
    /// [top, bottom, side] tile indices
    pub tiles: [u16; 3],
    /// collides with entities
    pub solid: bool,
    /// blocks skylight & culls neighbor faces
    pub opaque: bool,
    /// rendered as X-shaped plant
    pub cross: bool,
    pub fluid: bool,
    /// 0..15 self-illuminated light level (glowstone = 15)
    pub emissive: u8,
    pub sound: SoundFamily,
}

const fn d(
    name: &'static str,
    tiles: [u16; 3],
    solid: bool,
    opaque: bool,
    cross: bool,
    fluid: bool,
    emissive: u8,
    sound: SoundFamily,
) -> BlockDef {
    BlockDef { name, tiles, solid, opaque, cross, fluid, emissive, sound }
}

pub const BLOCK_TABLE: [BlockDef; BLOCK_COUNT] = [
    d("Air", [0, 0, 0], false, false, false, false, 0, SoundFamily::None),
    d("Grass Block", [TILE_GRASS_TOP, TILE_DIRT, TILE_GRASS_SIDE], true, true, false, false, 0, SoundFamily::Grass),
    d("Dirt", [TILE_DIRT, TILE_DIRT, TILE_DIRT], true, true, false, false, 0, SoundFamily::Dirt),
    d("Stone", [TILE_STONE, TILE_STONE, TILE_STONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Cobblestone", [TILE_COBBLE, TILE_COBBLE, TILE_COBBLE], true, true, false, false, 0, SoundFamily::Stone),
    d("Sand", [TILE_SAND, TILE_SAND, TILE_SAND], true, true, false, false, 0, SoundFamily::Sand),
    d("Oak Log", [TILE_LOG_TOP, TILE_LOG_TOP, TILE_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Oak Planks", [TILE_PLANKS, TILE_PLANKS, TILE_PLANKS], true, true, false, false, 0, SoundFamily::Wood),
    d("Oak Leaves", [TILE_LEAVES, TILE_LEAVES, TILE_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    d("Water", [TILE_WATER, TILE_WATER, TILE_WATER], false, false, false, true, 0, SoundFamily::Water),
    d("Glass", [TILE_GLASS, TILE_GLASS, TILE_GLASS], true, false, false, false, 0, SoundFamily::Glass),
    d("Bedrock", [TILE_BEDROCK, TILE_BEDROCK, TILE_BEDROCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Gravel", [TILE_GRAVEL, TILE_GRAVEL, TILE_GRAVEL], true, true, false, false, 0, SoundFamily::Sand),
    d("Snow Block", [TILE_SNOW, TILE_SNOW, TILE_SNOW], true, true, false, false, 0, SoundFamily::Sand),
    d("Snowy Grass", [TILE_SNOW, TILE_DIRT, TILE_SNOW_SIDE], true, true, false, false, 0, SoundFamily::Grass),
    d("Grass", [TILE_TALL_GRASS, TILE_TALL_GRASS, TILE_TALL_GRASS], false, false, true, false, 0, SoundFamily::Grass),
    d("Poppy", [TILE_FLOWER_RED, TILE_FLOWER_RED, TILE_FLOWER_RED], false, false, true, false, 0, SoundFamily::Grass),
    d("Dandelion", [TILE_FLOWER_YELLOW, TILE_FLOWER_YELLOW, TILE_FLOWER_YELLOW], false, false, true, false, 0, SoundFamily::Grass),
    // stone family
    d("Granite", [TILE_GRANITE, TILE_GRANITE, TILE_GRANITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Diorite", [TILE_DIORITE, TILE_DIORITE, TILE_DIORITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Andesite", [TILE_ANDESITE, TILE_ANDESITE, TILE_ANDESITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Stone Bricks", [TILE_STONE_BRICKS, TILE_STONE_BRICKS, TILE_STONE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Bricks", [TILE_BRICKS, TILE_BRICKS, TILE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Mossy Cobblestone", [TILE_MOSSY_COBBLE, TILE_MOSSY_COBBLE, TILE_MOSSY_COBBLE], true, true, false, false, 0, SoundFamily::Stone),
    d("Smooth Stone", [TILE_SMOOTH_STONE, TILE_SMOOTH_STONE, TILE_SMOOTH_STONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Obsidian", [TILE_OBSIDIAN, TILE_OBSIDIAN, TILE_OBSIDIAN], true, true, false, false, 0, SoundFamily::Stone),
    // ores
    d("Coal Ore", [TILE_COAL_ORE, TILE_COAL_ORE, TILE_COAL_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Iron Ore", [TILE_IRON_ORE, TILE_IRON_ORE, TILE_IRON_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Gold Ore", [TILE_GOLD_ORE, TILE_GOLD_ORE, TILE_GOLD_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Diamond Ore", [TILE_DIAMOND_ORE, TILE_DIAMOND_ORE, TILE_DIAMOND_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Redstone Ore", [TILE_REDSTONE_ORE, TILE_REDSTONE_ORE, TILE_REDSTONE_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Lapis Ore", [TILE_LAPIS_ORE, TILE_LAPIS_ORE, TILE_LAPIS_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Emerald Ore", [TILE_EMERALD_ORE, TILE_EMERALD_ORE, TILE_EMERALD_ORE], true, true, false, false, 0, SoundFamily::Stone),
    // mineral blocks
    d("Block of Iron", [TILE_IRON_BLOCK, TILE_IRON_BLOCK, TILE_IRON_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Block of Gold", [TILE_GOLD_BLOCK, TILE_GOLD_BLOCK, TILE_GOLD_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Block of Diamond", [TILE_DIAMOND_BLOCK, TILE_DIAMOND_BLOCK, TILE_DIAMOND_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    // misc
    d("Glowstone", [TILE_GLOWSTONE, TILE_GLOWSTONE, TILE_GLOWSTONE], true, true, false, false, 15, SoundFamily::Glass),
    d("Bookshelf", [TILE_BOOKSHELF_TOP, TILE_BOOKSHELF_TOP, TILE_BOOKSHELF_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Crafting Table", [TILE_CRAFT_TOP, TILE_PLANKS, TILE_CRAFT_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Clay", [TILE_CLAY, TILE_CLAY, TILE_CLAY], true, true, false, false, 0, SoundFamily::Dirt),
    d("Terracotta", [TILE_TERRACOTTA, TILE_TERRACOTTA, TILE_TERRACOTTA], true, true, false, false, 0, SoundFamily::Stone),
    d("Pumpkin", [TILE_PUMPKIN_TOP, TILE_PUMPKIN_TOP, TILE_PUMPKIN_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Melon", [TILE_MELON_TOP, TILE_MELON_TOP, TILE_MELON_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Ice", [TILE_ICE, TILE_ICE, TILE_ICE], true, false, false, false, 0, SoundFamily::Glass),
    d("Cactus", [TILE_CACTUS_TOP, TILE_CACTUS_TOP, TILE_CACTUS_SIDE], true, true, false, false, 0, SoundFamily::Wool),
    // wool
    d("White Wool", [TILE_WOOL_WHITE, TILE_WOOL_WHITE, TILE_WOOL_WHITE], true, true, false, false, 0, SoundFamily::Wool),
    d("Red Wool", [TILE_WOOL_RED, TILE_WOOL_RED, TILE_WOOL_RED], true, true, false, false, 0, SoundFamily::Wool),
    d("Blue Wool", [TILE_WOOL_BLUE, TILE_WOOL_BLUE, TILE_WOOL_BLUE], true, true, false, false, 0, SoundFamily::Wool),
    d("Yellow Wool", [TILE_WOOL_YELLOW, TILE_WOOL_YELLOW, TILE_WOOL_YELLOW], true, true, false, false, 0, SoundFamily::Wool),
    d("Black Wool", [TILE_WOOL_BLACK, TILE_WOOL_BLACK, TILE_WOOL_BLACK], true, true, false, false, 0, SoundFamily::Wool),
    // wood variants
    d("Birch Log", [TILE_LOG_TOP, TILE_LOG_TOP, TILE_BIRCH_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Birch Leaves", [TILE_BIRCH_LEAVES, TILE_BIRCH_LEAVES, TILE_BIRCH_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    d("Spruce Log", [TILE_LOG_TOP, TILE_LOG_TOP, TILE_SPRUCE_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Spruce Leaves", [TILE_SPRUCE_LEAVES, TILE_SPRUCE_LEAVES, TILE_SPRUCE_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    // plants
    d("Red Mushroom", [TILE_MUSHROOM_RED, TILE_MUSHROOM_RED, TILE_MUSHROOM_RED], false, false, true, false, 0, SoundFamily::Grass),
    d("Brown Mushroom", [TILE_MUSHROOM_BROWN, TILE_MUSHROOM_BROWN, TILE_MUSHROOM_BROWN], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Bush", [TILE_DEAD_BUSH, TILE_DEAD_BUSH, TILE_DEAD_BUSH], false, false, true, false, 0, SoundFamily::Grass),
    // Phase-1 model blocks: rendered through the blockstate/model JSON path
    // (partial geometry — not opaque, not greedy-meshed). Collision uses the
    // full-cube approximation until per-shape collision lands (Phase 6 TODO).
    d("Oak Slab", [TILE_PLANKS, TILE_PLANKS, TILE_PLANKS], true, false, false, false, 0, SoundFamily::Wood),
    d("Cobblestone Stairs", [TILE_COBBLE, TILE_COBBLE, TILE_COBBLE], true, false, false, false, 0, SoundFamily::Stone),
    d("Oak Fence", [TILE_PLANKS, TILE_PLANKS, TILE_PLANKS], true, false, false, false, 0, SoundFamily::Wood),
    // redstone core (Phase 6 §25): cross-rendered components; power lives
    // in the sim states (wire 113..128, lever 129/130, torch 131/132)
    d("Redstone Wire", [TILE_REDSTONE_WIRE, TILE_REDSTONE_WIRE, TILE_REDSTONE_WIRE], false, false, true, false, 0, SoundFamily::Grass),
    d("Redstone Torch", [TILE_REDSTONE_TORCH, TILE_REDSTONE_TORCH, TILE_REDSTONE_TORCH], false, false, true, false, 7, SoundFamily::Wood),
    d("Lever", [TILE_LEVER, TILE_LEVER, TILE_LEVER], false, false, true, false, 0, SoundFamily::Wood),
    // gameplay (Phase 7): full-cube greedy-meshed, lit variant glows
    d("Furnace", [TILE_FURNACE_TOP, TILE_FURNACE_TOP, TILE_FURNACE_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    // nether blocks (Phase 7 §28): full-cube greedy-meshed; quartz ore in
    // netherrack; soul sand slows (§23 hook later) and sinks slightly
    d("Netherrack", [TILE_NETHERRACK, TILE_NETHERRACK, TILE_NETHERRACK], true, true, false, false, 0, SoundFamily::Stone),
    d("Nether Quartz Ore", [TILE_QUARTZ_ORE, TILE_QUARTZ_ORE, TILE_QUARTZ_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Soul Sand", [TILE_SOUL_SAND, TILE_SOUL_SAND, TILE_SOUL_SAND], true, true, false, false, 0, SoundFamily::Sand),
    // brewing (Phase 7 §29): cross-rendered stand (vanilla stand is a small
    // rod — not a full cube, not solid); potion bottles are ITEM-only ids
    // that exist in inventories — is_item_block() guards placement
    d("Brewing Stand", [TILE_BREWING_STAND, TILE_BREWING_STAND, TILE_BREWING_STAND], false, false, true, false, 1, SoundFamily::Wood),
    d("Glass Bottle", [TILE_BOTTLE_EMPTY, TILE_BOTTLE_EMPTY, TILE_BOTTLE_EMPTY], false, false, true, false, 0, SoundFamily::Glass),
    d("Water Bottle", [TILE_POTION_WATER, TILE_POTION_WATER, TILE_POTION_WATER], false, false, true, false, 0, SoundFamily::Water),
    d("Awkward Potion", [TILE_POTION_AWKWARD, TILE_POTION_AWKWARD, TILE_POTION_AWKWARD], false, false, true, false, 0, SoundFamily::Water),
    d("Mundane Potion", [TILE_POTION_MUNDANE, TILE_POTION_MUNDANE, TILE_POTION_MUNDANE], false, false, true, false, 0, SoundFamily::Water),
    d("Potion of Healing", [TILE_POTION_HEALING, TILE_POTION_HEALING, TILE_POTION_HEALING], false, false, true, false, 0, SoundFamily::Water),
    d("Potion of Healing II", [TILE_POTION_HEALING_II, TILE_POTION_HEALING_II, TILE_POTION_HEALING_II], false, false, true, false, 0, SoundFamily::Water),
    // enchanting (§29): cross-rendered table slab (vanilla table is a small
    // block with runes); the book is an ITEM block that carries enchants
    d("Enchanting Table", [TILE_ENCHANT_TABLE, TILE_ENCHANT_TABLE, TILE_ENCHANT_TABLE], false, false, true, false, 4, SoundFamily::Stone),
    d("Enchanted Book", [TILE_ENCHANTED_BOOK, TILE_ENCHANTED_BOOK, TILE_ENCHANTED_BOOK], false, false, true, false, 0, SoundFamily::Wood),
    // mob drops (Phase 2) — item-only, cross-rendered icons
    d("Raw Beef", [TILE_BEEF, TILE_BEEF, TILE_BEEF], false, false, true, false, 0, SoundFamily::Grass),
    d("Raw Porkchop", [TILE_PORKCHOP, TILE_PORKCHOP, TILE_PORKCHOP], false, false, true, false, 0, SoundFamily::Grass),
    d("Raw Mutton", [TILE_MUTTON, TILE_MUTTON, TILE_MUTTON], false, false, true, false, 0, SoundFamily::Grass),
    d("Raw Chicken", [TILE_CHICKEN_RAW, TILE_CHICKEN_RAW, TILE_CHICKEN_RAW], false, false, true, false, 0, SoundFamily::Grass),
    d("Feather", [TILE_FEATHER, TILE_FEATHER, TILE_FEATHER], false, false, true, false, 0, SoundFamily::Grass),
    d("Leather", [TILE_LEATHER, TILE_LEATHER, TILE_LEATHER], false, false, true, false, 0, SoundFamily::Grass),
    d("Bone", [TILE_BONE, TILE_BONE, TILE_BONE], false, false, true, false, 0, SoundFamily::Stone),
    d("String", [TILE_STRING, TILE_STRING, TILE_STRING], false, false, true, false, 0, SoundFamily::Grass),
    d("Gunpowder", [TILE_GUNPOWDER, TILE_GUNPOWDER, TILE_GUNPOWDER], false, false, true, false, 0, SoundFamily::Sand),
    d("Ender Pearl", [TILE_ENDER_PEARL, TILE_ENDER_PEARL, TILE_ENDER_PEARL], false, false, true, false, 0, SoundFamily::Glass),
    d("Rotten Flesh", [TILE_ROTTEN_FLESH, TILE_ROTTEN_FLESH, TILE_ROTTEN_FLESH], false, false, true, false, 0, SoundFamily::Grass),
    d("Arrow", [TILE_ARROW_ITEM, TILE_ARROW_ITEM, TILE_ARROW_ITEM], false, false, true, false, 0, SoundFamily::Stone),
    // redstone components (Phase 3) — cross-rendered sprites (visual
    // simplification: vanilla repeaters/comparators are flat plates;
    // mechanics are fully directional via the state props)
    d("Redstone Repeater", [TILE_REPEATER, TILE_REPEATER, TILE_REPEATER], false, false, true, false, 0, SoundFamily::Wood),
    d("Redstone Comparator", [TILE_COMPARATOR, TILE_COMPARATOR, TILE_COMPARATOR], false, false, true, false, 0, SoundFamily::Wood),
    d("Piston", [TILE_PISTON, TILE_PISTON, TILE_PISTON], true, false, true, false, 0, SoundFamily::Wood),
    d("Sticky Piston", [TILE_STICKY_PISTON, TILE_STICKY_PISTON, TILE_STICKY_PISTON], true, false, true, false, 0, SoundFamily::Wood),
    d("Dispenser", [TILE_DISPENSER, TILE_DISPENSER, TILE_DISPENSER], true, false, true, false, 0, SoundFamily::Wood),
    d("Dropper", [TILE_DROPPER, TILE_DROPPER, TILE_DROPPER], true, false, true, false, 0, SoundFamily::Wood),
    d("Observer", [TILE_OBSERVER, TILE_OBSERVER, TILE_OBSERVER], true, false, true, false, 0, SoundFamily::Wood),
    d("Hopper", [TILE_HOPPER, TILE_HOPPER, TILE_HOPPER], true, false, true, false, 0, SoundFamily::Wood),
    d("Chest", [TILE_CHEST, TILE_CHEST, TILE_CHEST], true, false, true, false, 0, SoundFamily::Wood),
    // Phase 4 §26/§30: corruption-chain potions + items (item-only,
    // cross-rendered icons; potions drink, eyes are ingredients)
    d("Potion of Harming", [TILE_POTION_HARMING, TILE_POTION_HARMING, TILE_POTION_HARMING], false, false, true, false, 0, SoundFamily::Water),
    d("Potion of Harming II", [TILE_POTION_HARMING_II, TILE_POTION_HARMING_II, TILE_POTION_HARMING_II], false, false, true, false, 0, SoundFamily::Water),
    d("Spider Eye", [TILE_SPIDER_EYE, TILE_SPIDER_EYE, TILE_SPIDER_EYE], false, false, true, false, 0, SoundFamily::Grass),
    d("Fermented Spider Eye", [TILE_FERMENTED_EYE, TILE_FERMENTED_EYE, TILE_FERMENTED_EYE], false, false, true, false, 0, SoundFamily::Grass),
    // Phase 5 §27: solid cage block, NOT opaque (vanilla's spawner shows
    // the mob inside; ours is a full lattice cube — see-through faces on
    // all sides like glass)
    d("Monster Spawner", [TILE_SPAWNER, TILE_SPAWNER, TILE_SPAWNER], true, false, false, false, 0, SoundFamily::Stone),
    // Phase 10: end-portal frame (stronghold portal room) — solid cube
    // with the frame inset face; eye insertion + activation out of scope
    d("End Portal Frame", [TILE_END_PORTAL_FRAME, TILE_END_PORTAL_FRAME, TILE_END_PORTAL_FRAME], true, false, false, false, 0, SoundFamily::Stone),
    // ---- Phase E1 rows (ids 103..=139; evolution 1.0–1.2 bracket) ----
    d("Mycelium", [TILE_MYCELIUM_TOP, TILE_DIRT, TILE_MYCELIUM_SIDE], true, true, false, false, 0, SoundFamily::Grass),
    d("End Stone", [TILE_END_STONE, TILE_END_STONE, TILE_END_STONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Nether Bricks", [TILE_NETHER_BRICKS, TILE_NETHER_BRICKS, TILE_NETHER_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    // VERIFIED w/Redstone_Lamp: off = no light; lit state emits 15; opaque
    d("Redstone Lamp", [TILE_REDSTONE_LAMP, TILE_REDSTONE_LAMP, TILE_REDSTONE_LAMP], true, true, false, false, 0, SoundFamily::Glass),
    d("Chiseled Stone Bricks", [TILE_CHISELED_STONE_BRICKS, TILE_CHISELED_STONE_BRICKS, TILE_CHISELED_STONE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Chiseled Sandstone", [TILE_CHISELED_SANDSTONE, TILE_CHISELED_SANDSTONE, TILE_CHISELED_SANDSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Cut Sandstone", [TILE_CUT_SANDSTONE, TILE_CUT_SANDSTONE, TILE_CUT_SANDSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Smooth Sandstone", [TILE_SMOOTH_SANDSTONE, TILE_SMOOTH_SANDSTONE, TILE_SMOOTH_SANDSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Mushroom Block", [TILE_MUSHROOM_RED_BLOCK, TILE_MUSHROOM_RED_BLOCK, TILE_MUSHROOM_RED_BLOCK], true, true, false, false, 0, SoundFamily::Wood),
    d("Brown Mushroom Block", [TILE_MUSHROOM_BROWN_BLOCK, TILE_MUSHROOM_BROWN_BLOCK, TILE_MUSHROOM_BROWN_BLOCK], true, true, false, false, 0, SoundFamily::Wood),
    d("Mushroom Stem", [TILE_MUSHROOM_STEM, TILE_MUSHROOM_STEM, TILE_MUSHROOM_STEM], true, true, false, false, 0, SoundFamily::Wood),
    d("Nether Wart", [TILE_NETHER_WART_0, TILE_NETHER_WART_0, TILE_NETHER_WART_0], false, false, true, false, 0, SoundFamily::Grass),
    d("Dragon Egg", [TILE_DRAGON_EGG, TILE_DRAGON_EGG, TILE_DRAGON_EGG], true, true, false, false, 1, SoundFamily::Stone),
    // the portal itself: walk-in block, no skylight-blocking opacity,
    // full block light (vanilla star-field face)
    d("End Portal", [TILE_END_PORTAL, TILE_END_PORTAL, TILE_END_PORTAL], true, false, false, false, 15, SoundFamily::Stone),
    // ---- item-blocks (117..=139) — inventory-only, the potion pattern ----
    d("End Crystal", [TILE_END_CRYSTAL, TILE_END_CRYSTAL, TILE_END_CRYSTAL], false, false, true, false, 0, SoundFamily::Glass),
    d("Eye of Ender", [TILE_EYE_OF_ENDER, TILE_EYE_OF_ENDER, TILE_EYE_OF_ENDER], false, false, true, false, 0, SoundFamily::Glass),
    d("Blaze Rod", [TILE_BLAZE_ROD, TILE_BLAZE_ROD, TILE_BLAZE_ROD], false, false, true, false, 0, SoundFamily::Wood),
    d("Blaze Powder", [TILE_BLAZE_POWDER, TILE_BLAZE_POWDER, TILE_BLAZE_POWDER], false, false, true, false, 0, SoundFamily::Sand),
    d("Golden Apple", [TILE_GOLDEN_APPLE, TILE_GOLDEN_APPLE, TILE_GOLDEN_APPLE], false, false, true, false, 0, SoundFamily::Grass),
    d("Snowball", [TILE_SNOWBALL, TILE_SNOWBALL, TILE_SNOWBALL], false, false, true, false, 0, SoundFamily::Sand),
    d("Nether Brick", [TILE_NETHER_BRICK, TILE_NETHER_BRICK, TILE_NETHER_BRICK], false, false, true, false, 0, SoundFamily::Stone),
    d("Snow Golem Spawn Egg", [TILE_EGG_BASE, TILE_EGG_BASE, TILE_EGG_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Magma Cube Spawn Egg", [TILE_EGG_BASE + 1, TILE_EGG_BASE + 1, TILE_EGG_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Blaze Spawn Egg", [TILE_EGG_BASE + 2, TILE_EGG_BASE + 2, TILE_EGG_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Ocelot Spawn Egg", [TILE_EGG_BASE + 3, TILE_EGG_BASE + 3, TILE_EGG_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Iron Golem Spawn Egg", [TILE_EGG_BASE + 4, TILE_EGG_BASE + 4, TILE_EGG_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Zombie Villager Spawn Egg", [TILE_EGG_BASE + 5, TILE_EGG_BASE + 5, TILE_EGG_BASE + 5], false, false, true, false, 0, SoundFamily::Grass),
    d("Mooshroom Spawn Egg", [TILE_EGG_BASE + 6, TILE_EGG_BASE + 6, TILE_EGG_BASE + 6], false, false, true, false, 0, SoundFamily::Grass),
    d("Zombie Spawn Egg", [TILE_EGG_BASE + 7, TILE_EGG_BASE + 7, TILE_EGG_BASE + 7], false, false, true, false, 0, SoundFamily::Grass),
    d("Skeleton Spawn Egg", [TILE_EGG_BASE + 8, TILE_EGG_BASE + 8, TILE_EGG_BASE + 8], false, false, true, false, 0, SoundFamily::Grass),
    d("Creeper Spawn Egg", [TILE_EGG_BASE + 9, TILE_EGG_BASE + 9, TILE_EGG_BASE + 9], false, false, true, false, 0, SoundFamily::Grass),
    d("Spider Spawn Egg", [TILE_EGG_BASE + 10, TILE_EGG_BASE + 10, TILE_EGG_BASE + 10], false, false, true, false, 0, SoundFamily::Grass),
    d("Enderman Spawn Egg", [TILE_EGG_BASE + 11, TILE_EGG_BASE + 11, TILE_EGG_BASE + 11], false, false, true, false, 0, SoundFamily::Grass),
    d("Cow Spawn Egg", [TILE_EGG_BASE + 12, TILE_EGG_BASE + 12, TILE_EGG_BASE + 12], false, false, true, false, 0, SoundFamily::Grass),
    d("Pig Spawn Egg", [TILE_EGG_BASE + 13, TILE_EGG_BASE + 13, TILE_EGG_BASE + 13], false, false, true, false, 0, SoundFamily::Grass),
    d("Sheep Spawn Egg", [TILE_EGG_BASE + 14, TILE_EGG_BASE + 14, TILE_EGG_BASE + 14], false, false, true, false, 0, SoundFamily::Grass),
    d("Chicken Spawn Egg", [TILE_EGG_BASE + 15, TILE_EGG_BASE + 15, TILE_EGG_BASE + 15], false, false, true, false, 0, SoundFamily::Grass),
    // ---- Phase E2 eggs (mob kinds 17..=20: wither skeleton, witch, bat,
    // wither — VERIFIED spawn-egg usage rule) ----
    d("Wither Skeleton Spawn Egg", [TILE_EGG_BASE + 16, TILE_EGG_BASE + 16, TILE_EGG_BASE + 16], false, false, true, false, 0, SoundFamily::Grass),
    d("Witch Spawn Egg", [TILE_EGG_BASE + 17, TILE_EGG_BASE + 17, TILE_EGG_BASE + 17], false, false, true, false, 0, SoundFamily::Grass),
    d("Bat Spawn Egg", [TILE_EGG_BASE + 18, TILE_EGG_BASE + 18, TILE_EGG_BASE + 18], false, false, true, false, 0, SoundFamily::Grass),
    d("Wither Spawn Egg", [TILE_EGG_BASE + 19, TILE_EGG_BASE + 19, TILE_EGG_BASE + 19], false, false, true, false, 0, SoundFamily::Grass),
    // ---- Phase E2 world blocks (evolution 1.3–1.4 bracket) ----
    // anvil family: solid, opaque; damage tiles come from state_tiles
    d("Anvil", [TILE_ANVIL, TILE_ANVIL, TILE_ANVIL], true, true, false, false, 0, SoundFamily::Stone),
    d("Chipped Anvil", [TILE_ANVIL_CHIPPED, TILE_ANVIL_CHIPPED, TILE_ANVIL_CHIPPED], true, true, false, false, 0, SoundFamily::Stone),
    d("Damaged Anvil", [TILE_ANVIL_DAMAGED, TILE_ANVIL_DAMAGED, TILE_ANVIL_DAMAGED], true, true, false, false, 0, SoundFamily::Stone),
    // beacon: solid, NOT opaque (vanilla glass-like core; the beam rides
    // the billboard pipeline) — self-lit 15 (VERIFIED w/Beacon)
    d("Beacon", [TILE_BEACON, TILE_BEACON, TILE_BEACON], true, false, false, false, 15, SoundFamily::Glass),
    // wall: solid, not opaque (fence-class boundary block — 1.5-tall
    // collision; connections at mesh time)
    d("Cobblestone Wall", [TILE_COBBLE_WALL, TILE_COBBLE_WALL, TILE_COBBLE_WALL], true, false, false, false, 0, SoundFamily::Stone),
    // ender chest: solid, opaque, light 7 (VERIFIED w/Ender_Chest)
    d("Ender Chest", [TILE_ENDER_CHEST, TILE_ENDER_CHEST, TILE_ENDER_CHEST], true, true, false, false, 7, SoundFamily::Stone),
    // flower pot: cross-rendered, instant break (hardness 0)
    d("Flower Pot", [TILE_FLOWER_POT, TILE_FLOWER_POT, TILE_FLOWER_POT], false, false, true, false, 0, SoundFamily::Grass),
    // item frame: cross-rendered sprite of the frame; contents ride the
    // container system (1-slot, position-keyed)
    d("Item Frame", [TILE_ITEM_FRAME, TILE_ITEM_FRAME, TILE_ITEM_FRAME], false, false, true, false, 0, SoundFamily::Wood),
    // tripwire hook: cross-rendered; facing × powered in the state
    d("Tripwire Hook", [TILE_TRIPWIRE_HOOK, TILE_TRIPWIRE_HOOK, TILE_TRIPWIRE_HOOK], false, false, true, false, 0, SoundFamily::Wood),
    // wither skeleton skull: cross-rendered head on the ground
    d("Wither Skeleton Skull", [TILE_WITHER_SKULL, TILE_WITHER_SKULL, TILE_WITHER_SKULL], false, false, true, false, 0, SoundFamily::Stone),
    // command block: solid opaque cube; the ON face tile glows (lit state
    // = last-fired pulse, functional only)
    d("Command Block", [TILE_COMMAND_BLOCK, TILE_COMMAND_BLOCK, TILE_COMMAND_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    // ---- Phase E2 item-blocks (155..=160) — inventory-only ----
    d("Emerald", [TILE_EMERALD, TILE_EMERALD, TILE_EMERALD], false, false, true, false, 0, SoundFamily::Grass),
    d("Nether Star", [TILE_NETHER_STAR, TILE_NETHER_STAR, TILE_NETHER_STAR], false, false, true, false, 0, SoundFamily::Glass),
    d("Potato", [TILE_POTATO, TILE_POTATO, TILE_POTATO], false, false, true, false, 0, SoundFamily::Grass),
    d("Baked Potato", [TILE_BAKED_POTATO, TILE_BAKED_POTATO, TILE_BAKED_POTATO], false, false, true, false, 0, SoundFamily::Grass),
    d("Carrot", [TILE_CARROT, TILE_CARROT, TILE_CARROT], false, false, true, false, 0, SoundFamily::Grass),
    d("Pumpkin Pie", [TILE_PUMPKIN_PIE, TILE_PUMPKIN_PIE, TILE_PUMPKIN_PIE], false, false, true, false, 0, SoundFamily::Grass),
    // lava: fluid, emissive 15 (VERIFIED w/Lava luminance), not solid —
    // meshes through the fluid-quad path with the fixed lava tint
    d("Lava", [TILE_LAVA, TILE_LAVA, TILE_LAVA], false, false, false, true, 15, SoundFamily::Water),
];

#[inline]
pub fn def(id: u8) -> &'static BlockDef {
    // §46 resilience: a raw STATE id that slipped past a fold indexes the
    // table out of bounds — clamp to the last def instead of panicking
    // (the honest fix is folding via state_block; this is the crash guard)
    &BLOCK_TABLE[(id as usize).min(BLOCK_COUNT - 1)]
}

#[inline]
pub fn is_solid(id: u8) -> bool {
    def(id).solid
}

#[inline]
pub fn is_opaque(id: u8) -> bool {
    def(id).opaque
}

#[inline]
pub fn is_cross(id: u8) -> bool {
    def(id).cross
}

#[inline]
pub fn is_fluid(id: u8) -> bool {
    def(id).fluid
}

#[inline]
pub fn emissive(id: u8) -> u8 {
    def(id).emissive
}

#[inline]
pub fn name(id: u8) -> &'static str {
    def(id).name
}

/// Should a face of `b` facing neighbor `n` be rendered?
#[inline]
pub fn face_visible(b: u8, n: u8) -> bool {
    if b == AIR {
        return false;
    }
    if b == WATER {
        // water visible through non-water, non-opaque neighbors (air, glass, plants)
        return !is_opaque(n) && n != WATER;
    }
    if b == LEAVES || b == BIRCH_LEAVES || b == SPRUCE_LEAVES {
        // "fancy" leaves: render even against other leaves
        return !is_opaque(n);
    }
    if b == GLASS {
        return !is_opaque(n) && n != GLASS;
    }
    if b == ICE {
        return !is_opaque(n) && n != ICE;
    }
    // fully opaque blocks
    !is_opaque(n)
}

/// blocks offered in the E-key picker (creative-style), in display order.
/// Everything placeable except air, bedrock (unbreakable) and water
/// (needs fluid sim to be fun). Potions are item-blocks — usable from the
/// hotbar (drink), never placeable. Phase E1 adds the 1.0–1.2 bracket
/// blocks/items + the 16 spawn eggs (creative-only items, w/Spawn_Egg).
pub const PICKER_BLOCKS: [u8; 126] = [
    GRASS, DIRT, STONE, COBBLE, SMOOTH_STONE, STONE_BRICKS, BRICKS, MOSSY_COBBLE,
    GRANITE, DIORITE, ANDESITE, OBSIDIAN,
    SAND, GRAVEL, CLAY, TERRACOTTA,
    OAK_LOG, LEAVES, PLANKS, BIRCH_LOG, BIRCH_LEAVES, SPRUCE_LOG, SPRUCE_LEAVES,
    COAL_ORE, IRON_ORE, GOLD_ORE, REDSTONE_ORE, LAPIS_ORE, EMERALD_ORE, DIAMOND_ORE,
    IRON_BLOCK, GOLD_BLOCK, DIAMOND_BLOCK, GLOWSTONE,
    BOOKSHELF, CRAFTING_TABLE, FURNACE, GLASS, ICE, SNOW,
    PUMPKIN, MELON, CACTUS,
    WOOL_WHITE, WOOL_RED, WOOL_YELLOW, WOOL_BLUE, WOOL_BLACK,
    TALL_GRASS, FLOWER_RED, FLOWER_YELLOW, MUSHROOM_RED, MUSHROOM_BROWN,
    OAK_SLAB, COBBLE_STAIRS, OAK_FENCE,
    NETHERRACK, NETHER_QUARTZ_ORE, SOUL_SAND,
    BREWING_STAND,
    POTION_EMPTY, POTION_WATER, POTION_AWKWARD, POTION_MUNDANE, POTION_HEALING, POTION_HEALING_II,
    ENCHANT_TABLE, ENCHANTED_BOOK,
    // ---- Phase E1 (1.0–1.2 bracket) ----
    MYCELIUM, END_STONE, NETHER_BRICKS, NETHER_BRICK,
    REDSTONE_LAMP,
    CHISELED_STONE_BRICKS, CHISELED_SANDSTONE, CUT_SANDSTONE, SMOOTH_SANDSTONE,
    MUSHROOM_RED_BLOCK, MUSHROOM_BROWN_BLOCK, MUSHROOM_STEM,
    NETHER_WART, DRAGON_EGG, END_CRYSTAL,
    EYE_OF_ENDER, BLAZE_ROD, BLAZE_POWDER, GOLDEN_APPLE, SNOWBALL,
    // spawn eggs (16, order = MobKind egg table)
    SPAWN_EGG_BASE, SPAWN_EGG_BASE + 1, SPAWN_EGG_BASE + 2, SPAWN_EGG_BASE + 3,
    SPAWN_EGG_BASE + 4, SPAWN_EGG_BASE + 5, SPAWN_EGG_BASE + 6, SPAWN_EGG_BASE + 7,
    SPAWN_EGG_BASE + 8, SPAWN_EGG_BASE + 9, SPAWN_EGG_BASE + 10, SPAWN_EGG_BASE + 11,
    SPAWN_EGG_BASE + 12, SPAWN_EGG_BASE + 13, SPAWN_EGG_BASE + 14, SPAWN_EGG_BASE + 15,
    // ---- Phase E2 (1.3–1.4 bracket) ----
    ANVIL, CHIPPED_ANVIL, DAMAGED_ANVIL, BEACON, COBBLE_WALL,
    ENDER_CHEST, FLOWER_POT, ITEM_FRAME, TRIPWIRE_HOOK,
    WITHER_SKELETON_SKULL, COMMAND_BLOCK,
    EMERALD, NETHER_STAR, POTATO, BAKED_POTATO, CARROT, PUMPKIN_PIE,
    LAVA,
    // E2 spawn eggs (kinds 17..=20: wither skeleton, witch, bat, wither)
    SPAWN_EGG_BASE + 16, SPAWN_EGG_BASE + 17, SPAWN_EGG_BASE + 18, SPAWN_EGG_BASE + 19,
];

/// default hotbar palette
pub const PALETTE: [u8; 9] = [GRASS, DIRT, STONE, COBBLE, PLANKS, OAK_LOG, LEAVES, GLOWSTONE, GLASS];

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn identity_states_fold_to_their_blocks() {
        // flat-registry states 0..=56 are identity-mapped; 57..=62 are the
        // log axis variants (state ids ≠ block ids there); 63+ are property
        // states (covered by prop_states_roundtrip)
        for b in 0..57u16 {
            assert_eq!(state_block(b), b as u8, "state {b}");
        }
        for s in 57..=62u16 {
            assert!(is_log(state_block(s)), "state {s} must fold to a log");
        }
    }

    #[test]
    fn log_axis_variants() {
        assert_eq!(state_block(OAK_LOG_X), OAK_LOG);
        assert_eq!(state_block(OAK_LOG_Z), OAK_LOG);
        assert_eq!(state_block(BIRCH_LOG_X), BIRCH_LOG);
        assert_eq!(state_block(SPRUCE_LOG_Z), SPRUCE_LOG);
        // default (axis Y) is the identity state
        assert_eq!(log_axis_state(OAK_LOG, 1), OAK_LOG as u16);
        assert_eq!(log_axis_state(OAK_LOG, 0), OAK_LOG_X);
        assert_eq!(log_axis_state(OAK_LOG, 2), OAK_LOG_Z);
        // non-logs pass through untouched
        assert_eq!(log_axis_state(STONE, 0), STONE as u16);
    }

    #[test]
    fn log_tiles_rotate_with_axis() {
        // axis Y (default): rings on top/bottom, bark on the sides
        let y = state_tiles(OAK_LOG as u16);
        assert_eq!(y[0], TILE_LOG_TOP);
        assert_eq!(y[1], TILE_LOG_TOP);
        assert_eq!(y[2], TILE_LOG_SIDE);
        assert_eq!(y[3], TILE_LOG_SIDE);
        // axis X: rings on the ±X faces, bark elsewhere
        let x = state_tiles(OAK_LOG_X);
        assert_eq!(x[0], TILE_LOG_SIDE);
        assert_eq!(x[1], TILE_LOG_SIDE);
        assert_eq!(x[2], TILE_LOG_TOP);
        assert_eq!(x[3], TILE_LOG_SIDE);
        // axis Z: rings on the ±Z faces
        let z = state_tiles(OAK_LOG_Z);
        assert_eq!(z[2], TILE_LOG_SIDE);
        assert_eq!(z[3], TILE_LOG_TOP);
        // every non-variant state mirrors its block def
        let g = state_tiles(GRASS as u16);
        assert_eq!(g, [TILE_GRASS_TOP, TILE_DIRT, TILE_GRASS_SIDE, TILE_GRASS_SIDE]);
    }

    #[test]
    fn all_states_in_range() {
        for s in 0..STATE_COUNT as u16 {
            let b = state_block(s);
            assert!(b < BLOCK_COUNT as u8, "state {s} maps to bad block {b}");
            let t = state_tiles(s);
            assert!(t.iter().all(|&t| t <= TILE_MAX));
        }
    }

    /// REGRESSION (Phase 4 bug fix): TILE_MAX sat at 82 while Phases 2/3
    /// defined tiles 83–117 — the atlas loop (0..=TILE_MAX) never drew
    /// them, so mob sprites / drop icons / redstone tiles rendered blank.
    /// Every def-referenced tile must now be within TILE_MAX so the atlas
    /// generator provably reaches it.
    #[test]
    fn all_def_tiles_within_tile_max() {
        for b in 0..BLOCK_COUNT as u8 {
            for &t in def(b).tiles.iter() {
                assert!(
                    t <= TILE_MAX,
                    "block {b} ({}) references tile {t} > TILE_MAX {TILE_MAX} — it would render blank",
                    def(b).name
                );
            }
        }
    }

    // ---- Phase E1 tests (1.0–1.2 bracket) ----

    /// Every state ≤ STATE_COUNT folds to a real block — extended to the
    /// lamp/wart/spawner-blaze/frame-eye + dedicated world/item states.
    #[test]
    fn phase_e1_states_fold_and_emissive() {
        // lit lamp folds to the lamp block but emits 15; the OFF state
        // (the lamp's stored default) emits 0
        assert_eq!(state_block(REDSTONE_LAMP_LIT), REDSTONE_LAMP);
        assert_eq!(state_emissive(REDSTONE_LAMP_LIT), 15);
        assert_eq!(state_block(REDSTONE_LAMP_STATE), REDSTONE_LAMP);
        assert_eq!(state_emissive(REDSTONE_LAMP_STATE), 0);
        assert_eq!(default_state(REDSTONE_LAMP), REDSTONE_LAMP_STATE);
        // warts fold + per-age tiles
        for a in 0..4u16 {
            assert_eq!(state_block(WART_STATE_BASE + a), NETHER_WART);
            assert_eq!(wart_age(WART_STATE_BASE + a), a as u8);
            let t = state_tiles(WART_STATE_BASE + a);
            assert!(t.iter().all(|&x| x >= TILE_NETHER_WART_0 && x <= TILE_NETHER_WART_3));
        }
        assert_eq!(default_state(NETHER_WART), WART_STATE_BASE);
        // frame-with-eye folds to the frame
        assert_eq!(state_block(END_PORTAL_FRAME_EYE), END_PORTAL_FRAME);
        // spawner blaze state folds to the spawner
        assert_eq!(state_block(SPAWNER_BLAZE), SPAWNER);
        // every new world block's default state folds back to it
        for b in [
            MYCELIUM, END_STONE, NETHER_BRICKS, REDSTONE_LAMP, CHISELED_STONE_BRICKS,
            CHISELED_SANDSTONE, CUT_SANDSTONE, SMOOTH_SANDSTONE, MUSHROOM_RED_BLOCK,
            MUSHROOM_BROWN_BLOCK, MUSHROOM_STEM, NETHER_WART, DRAGON_EGG, END_PORTAL,
        ] {
            assert_eq!(
                state_block(default_state(b)),
                b,
                "default_state({}) must fold back",
                name(b)
            );
        }
        // item states (≥ 256, never world-stored) fold back to their items
        for b in [END_CRYSTAL, EYE_OF_ENDER, BLAZE_ROD, BLAZE_POWDER, GOLDEN_APPLE, SNOWBALL, NETHER_BRICK, SPAWN_EGG_BASE, SPAWN_EGG_MAX] {
            let s = item_block_state(b).unwrap();
            assert!(s >= 256, "item states must live above the u8 window");
            assert_eq!(state_block(s), b);
            assert_eq!(default_state(b), s);
            assert!(!is_model_state(s), "item state {s} must not hit the model path");
        }
        // the end portal emits full block light through its stored state
        assert_eq!(state_emissive(END_PORTAL_STATE), 15);
        // dragon egg glows level 1
        assert_eq!(state_emissive(DRAGON_EGG_STATE), 1);
        // the full 236..=255 window is allocated — no spares, no overlaps
        // with the legacy ranges (this catches the MYCELIUM_STATE=140 / 140 =
        // ROTTEN_FLESH_STATE class of collision for good)
        let legacy_top = FERMENTED_EYE_STATE.max(SPAWNER_STATE_END).max(END_PORTAL_FRAME_STATE);
        assert!(legacy_top < 236, "legacy ranges must stay below 236");
        for b in [MYCELIUM, END_STONE, NETHER_WART, DRAGON_EGG, END_PORTAL] {
            assert!(default_state(b) >= 236);
        }
    }

    /// The 16 spawn eggs map in and out, are item-blocks (never placeable),
    /// and their def tiles land in the egg tile window.
    #[test]
    fn phase_e1_spawn_eggs() {
        for i in 0..16u8 {
            let b = SPAWN_EGG_BASE + i;
            assert!(is_spawn_egg(b), "egg {b}");
            assert_eq!(egg_mob(b), Some(i));
            assert!(is_item_block(b), "egg {b} must be an item block");
            let t = def(b).tiles[0];
            assert!((TILE_EGG_BASE..=TILE_EGG_MAX).contains(&t), "egg {b} tile {t}");
        }
        assert_eq!(egg_mob(SPAWN_EGG_BASE - 1), None);
        assert!(!is_spawn_egg(BLAZE_ROD));
        // eggs are distinct ids — the gameplay layer decodes 0..=19
        assert_eq!(SPAWN_EGG_MAX - SPAWN_EGG_BASE, 19);
    }

    /// New item-blocks are recognized; placeables are not.
    #[test]
    fn phase_e1_item_blocks() {
        for b in [END_CRYSTAL, EYE_OF_ENDER, BLAZE_ROD, BLAZE_POWDER, GOLDEN_APPLE, SNOWBALL, NETHER_BRICK] {
            assert!(is_item_block(b), "{b} must be an item block");
        }
        for b in [MYCELIUM, END_STONE, NETHER_BRICKS, REDSTONE_LAMP, DRAGON_EGG] {
            assert!(!is_item_block(b), "{b} must be placeable");
        }
    }

    /// The picker carries valid ids only, and the new E1 entries are present.
    #[test]
    fn phase_e1_picker_entries() {
        for &b in PICKER_BLOCKS.iter() {
            assert!((b as usize) < BLOCK_COUNT, "picker id {b} out of range");
        }
        for want in [MYCELIUM, END_STONE, REDSTONE_LAMP, SPAWN_EGG_BASE, SPAWN_EGG_BASE + 15] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        // 12-col grid must stay inside the 960×540 UI canvas
        let rows = (PICKER_BLOCKS.len() + 11) / 12;
        assert!(rows * 44 + 30 <= 540, "picker grid too tall: {rows} rows");
        assert!(12 * 44 + 8 <= 960);
    }

    /// Phase 4: the dedicated item tiles exist in the 16×16-tile atlas grid
    #[test]
    fn phase4_tiles_fit_the_atlas() {
        assert!(TILE_POTION_HARMING < 256);
        assert!(TILE_FERMENTED_EYE < 256);
        // the tile ids the Phase 4 blocks reference are exactly the new ones
        assert_eq!(def(POTION_HARMING).tiles[0], TILE_POTION_HARMING);
        assert_eq!(def(FERMENTED_SPIDER_EYE).tiles[0], TILE_FERMENTED_EYE);
    }

    #[test]
    fn prop_states_roundtrip() {
        // slab: half=bottom at base, half=top next
        assert_eq!(prop_state_decode(63), Some((OAK_SLAB, vec![("half", "bottom")])));
        assert_eq!(prop_state_decode(64), Some((OAK_SLAB, vec![("half", "top")])));
        assert_eq!(prop_state_encode(OAK_SLAB, &[("half", "top")]), Some(64));
        assert_eq!(
            prop_state_encode(OAK_SLAB, &[]),
            Some(63),
            "missing props default to first value"
        );
        // stairs: facing (radix 4, slow) × half (fast)
        assert_eq!(
            prop_state_decode(65),
            Some((COBBLE_STAIRS, vec![("facing", "north"), ("half", "bottom")]))
        );
        assert_eq!(
            prop_state_decode(66),
            Some((COBBLE_STAIRS, vec![("facing", "north"), ("half", "top")]))
        );
        assert_eq!(
            prop_state_decode(67),
            Some((COBBLE_STAIRS, vec![("facing", "east"), ("half", "bottom")]))
        );
        assert_eq!(
            prop_state_encode(COBBLE_STAIRS, &[("facing", "west"), ("half", "top")]),
            Some(72)
        );
        // fence: east×north×south×west, west fastest
        assert_eq!(
            prop_state_decode(73),
            Some((OAK_FENCE, vec![("east", "false"), ("north", "false"), ("south", "false"), ("west", "false")]))
        );
        assert_eq!(
            prop_state_decode(88),
            Some((OAK_FENCE, vec![("east", "true"), ("north", "true"), ("south", "true"), ("west", "true")]))
        );
        assert_eq!(
            prop_state_encode(OAK_FENCE, &[("north", "true")]),
            Some(77),
            "single connection north = base + 1*4 (east slot slow radix)"
        );
        // exhaustive roundtrip over all model states (water flow states
        // 89..=95 are SIM states — decoded by water_level, not the prop
        // machinery)
        for s in MODEL_STATE_BASE..STATE_COUNT as u16 {
            if is_water_flow(s) {
                assert!(!is_model_state(s), "flow state {s} never routes to models");
                assert_eq!(state_block(s), WATER);
                continue;
            }
            if is_wire_power(s) {
                assert!(!is_model_state(s), "wire state {s} never routes to models");
                assert_eq!(state_block(s), REDSTONE_WIRE);
                continue;
            }
            if matches!(s, LEVER_OFF | LEVER_ON) {
                assert_eq!(state_block(s), LEVER);
                assert!(!is_model_state(s));
                continue;
            }
            if matches!(s, TORCH_LIT | TORCH_OFF) {
                assert_eq!(state_block(s), REDSTONE_TORCH);
                assert!(!is_model_state(s));
                continue;
            }
            if matches!(s, FURNACE_STATE | FURNACE_LIT) {
                assert_eq!(state_block(s), FURNACE);
                assert!(!is_model_state(s));
                continue;
            }
            // nether blocks (§28): full-cube greedy-meshed, dedicated states
            if matches!(s, NETHERRACK_STATE | QUARTZ_ORE_STATE | SOUL_SAND_STATE) {
                assert!(!is_model_state(s), "nether state {s} never routes to models");
                assert!(s > FURNACE_LIT, "nether states live above the sim range");
                continue;
            }
            // brewing (§29): stand + potion item-blocks, dedicated states
            if matches!(
                s,
                BREWING_STAND_STATE
                    | POTION_EMPTY_STATE
                    | POTION_WATER_STATE
                    | POTION_AWKWARD_STATE
                    | POTION_MUNDANE_STATE
                    | POTION_HEALING_STATE
                    | POTION_HEALING_II_STATE
                    | ENCHANT_TABLE_STATE
                    | ENCHANTED_BOOK_STATE
            ) {
                assert!(!is_model_state(s), "brewing/enchant state {s} never routes to models");
                assert!(s > SOUL_SAND_STATE, "brewing states live above the sim range");
                // every dedicated state folds back to its own block id
                // (BREWING_STAND_STATE..ENCHANTED_BOOK_STATE == blocks 67..75)
                assert_eq!(state_block(s), (s - BREWING_STAND_STATE + BREWING_STAND as u16) as u8);
                continue;
            }
            // Phase 2 mob-drop item states + Phase 3 redstone-component
            // states + Phase 4 corruption-chain states + Phase 5 spawner
            // states: dedicated identity states (fold 1:1 to their block,
            // never model states — the components' visuals are
            // cross-sprites or plain cube defs, not block models)
            if (BEEF_STATE..=ARROW_ITEM_STATE).contains(&s)
                || (REPEATER_STATE_BASE..=CHEST_STATE).contains(&s)
                || (POTION_HARMING_STATE..=FERMENTED_EYE_STATE).contains(&s)
                || (SPAWNER_STATE_BASE..=SPAWNER_STATE_END).contains(&s)
                || s == END_PORTAL_FRAME_STATE
                // Phase E1 dedicated world-block states + item states
                || (REDSTONE_LAMP_LIT..=END_STONE_STATE).contains(&s)
                || (ITEM_STATE_BASE..=ITEM_STATE_END).contains(&s)
                // Phase E2 dedicated world-block + item states + lava
                || (ANVIL_STATE..=E2_ITEM_STATE_END).contains(&s)
                || s == LAVA_STATE
                || (LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s)
                || s == SPAWNER_WITHER_SKELETON
            {
                assert!(!is_model_state(s), "component/item state {s} never routes to models");
                // identity: the state folds to the block whose def table
                // lists it (verified per-block in the dedicated ranges'
                // own tests)
                let b = state_block(s);
                assert!(b < BLOCK_COUNT as u8, "state {s} folds to valid block");
                continue;
            }
            let Some((b, props)) = prop_state_decode(s) else {
                panic!("state {s} failed to decode");
            };
            let set: Vec<(&str, &str)> = props.iter().map(|(k, v)| (*k, *v)).collect();
            assert_eq!(prop_state_encode(b, &set), Some(s), "state {s}");
            assert!(is_model_block(b) && is_model_state(s));
        }
        // water level roundtrip
        for l in 0u8..=7 {
            let s = water_state(l);
            assert_eq!(water_level(s), l);
            assert_eq!(state_block(s), WATER);
            assert!(!is_model_state(s));
        }
        assert_eq!(water_level(STONE as u16), 255);
        // redstone state roundtrips
        for p in 0u8..=15 {
            let s = wire_state(p);
            assert_eq!(wire_power(s), p, "wire power roundtrip {p}");
            assert_eq!(state_block(s), REDSTONE_WIRE);
            assert!(!is_model_state(s));
        }
        assert_eq!(wire_power(STONE as u16), 255);
        assert!(lever_is_on(lever_state(true)));
        assert!(!lever_is_on(lever_state(false)));
        assert_eq!(state_block(lever_state(true)), LEVER);
        assert!(torch_is_lit(torch_state(true)));
        assert!(!torch_is_lit(torch_state(false)));
        assert_eq!(state_block(torch_state(false)), REDSTONE_TORCH);
        // block ids 60..62 never appear as identity states (57..62 = logs)
        for b in [REDSTONE_WIRE, REDSTONE_TORCH, LEVER] {
            let d = def(b);
            assert!(d.cross, "{} renders as a cross plant", d.name);
        }
        // states below the base are legacy, never model states
        assert!(!is_model_state(62));
        assert!(!is_model_block(STONE));
    }

    #[test]
    fn state_descriptions_vanilla_style() {
        assert_eq!(state_description(64), "Oak Slab[half=top]");
        assert_eq!(state_description(65), "Cobblestone Stairs[facing=north,half=bottom]");
        assert_eq!(state_description(OAK_LOG_X), "Oak Log[axis=x]");
        assert_eq!(state_description(STONE as u16), "Stone");
    }

    /// §28 + the P7-structures followup: every BLOCK id must place a state
    /// that (a) folds back to itself and (b) never masquerades as another
    /// block's model/log state. This is exactly the collision class that
    /// made village furnaces render as oak slabs and well posts as birch
    /// logs (raw identity ids 57..63 land on log-axis/model slots).
    #[test]
    fn default_states_never_collide_and_fold_back() {
        for b in 0..BLOCK_COUNT as u8 {
            if b == AIR {
                continue; // air's identity state is legal
            }
            let s = default_state(b);
            assert_eq!(
                state_block(s),
                b,
                "default_state({b} = {}) folds back to {}",
                name(b),
                name(state_block(s))
            );
        }
        // the previously-colliding placements, pinned:
        assert_eq!(default_state(OAK_SLAB), 63, "slab → half=bottom model state");
        assert_eq!(default_state(COBBLE_STAIRS), 65, "stairs → facing=north, half=bottom");
        assert_eq!(default_state(OAK_FENCE), 73, "fence → no connections");
        assert_eq!(default_state(FURNACE), FURNACE_STATE);
        assert_eq!(default_state(NETHERRACK), NETHERRACK_STATE);
        assert_eq!(default_state(NETHER_QUARTZ_ORE), QUARTZ_ORE_STATE);
        assert_eq!(default_state(SOUL_SAND), SOUL_SAND_STATE);
        // sim blocks keep their states
        assert_eq!(default_state(REDSTONE_WIRE), wire_state(0));
        assert_eq!(default_state(REDSTONE_TORCH), torch_state(true));
        assert_eq!(default_state(LEVER), lever_state(false));
    }

    /// §46: a raw STATE id fed to the u8 block helpers must not panic —
    /// def() clamps; the fold helpers are total functions.
    #[test]
    fn def_clamps_out_of_range_state_ids() {
        // simulate the top_solid_y bug class: state 73 (fence) truncated
        // to u8 = 73 lands past the table end
        for raw in [73u8, 88, 96, 118, 121, 255] {
            let d = def(raw);
            let _ = d.solid;
            let _ = is_solid(raw);
            let _ = is_opaque(raw);
            let _ = name(raw);
        }
    }
}
