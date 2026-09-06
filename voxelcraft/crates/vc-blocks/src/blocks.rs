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
/// coal ITEM tile (VERIFICATION-REPORT mechanical fix #4 — base-game
/// coal as an item-block; fuel 1600 ticks, smelt product of coal ore)
pub const TILE_COAL: u16 = 206;
// ---- Phase E3 (evolution 1.5–1.6 bracket, live-verified 2026-09-06) ----
// world-block tiles (207..=224)
pub const TILE_COAL_BLOCK: u16 = 207;
pub const TILE_QUARTZ_BLOCK: u16 = 208;
pub const TILE_CHISELED_QUARTZ: u16 = 209;
pub const TILE_QUARTZ_PILLAR_TOP: u16 = 210;
pub const TILE_QUARTZ_PILLAR_SIDE: u16 = 211;
/// 16 stained-terracotta tiles (base 212..=227) — engine color order:
/// white, orange, magenta, light blue, yellow, lime, pink, gray,
/// light gray, cyan, purple, blue, brown, green, red, black (the
/// vanilla dye-color registry order, VERIFIED w/Terracotta)
pub const TILE_TERRACOTTA_STAINED_BASE: u16 = 212;
pub const TILE_HAY_TOP: u16 = 228;
pub const TILE_HAY_SIDE: u16 = 229;
pub const TILE_DAYLIGHT_TOP: u16 = 230;
pub const TILE_DAYLIGHT_SIDE: u16 = 231;
pub const TILE_PLATE_LIGHT: u16 = 232;
pub const TILE_PLATE_HEAVY: u16 = 233;
pub const TILE_REDSTONE_BLOCK: u16 = 234;
// item tiles
pub const TILE_NETHER_QUARTZ: u16 = 235;
pub const TILE_LEAD: u16 = 236;
pub const TILE_SADDLE: u16 = 237;
// E3 mob sprites (billboards)
pub const TILE_HORSE: u16 = 238;
pub const TILE_DONKEY: u16 = 239;
pub const TILE_MULE: u16 = 240;
// E3 spawn eggs (3 tiles, one per new mob kind — the E2 pattern)
pub const TILE_E3_EGG_BASE: u16 = 241;
// E3 spawn eggs (kinds 20..=22: horse, donkey, mule — ids 197..=199;
// the legacy 124..=143 egg window is full)
pub const TILE_E3_EGG_HORSE: u16 = 241;
pub const TILE_E3_EGG_DONKEY: u16 = 242;
pub const TILE_E3_EGG_MULE: u16 = 243;
// ---------------------------------------------------------------------------
// 1.7.2 bracket ("The Update that Changed the World", 2013-10-25 —
// [merge 2026-09-06] tile ids shifted past the E-series backfill
// (TILE_MAX 243); the 16 stained-terracotta tiles DROPPED as duplicates
// of the E3 terracotta set (TILE_TERRACOTTA_STAINED_BASE 212).
// minecraft.wiki/w/Java_Edition_1.7.2, live round 2026-09-06): the 16
// stained-glass + 16 stained-clay tiles, red sand, packed ice, podzol,
// acacia/dark-oak logs, the 8 new small flowers, the 4 two-block-tall
// flowers (lower + upper halves), and 4 fish item icons.
// Clean-room palettes approximate the vanilla 16 dye hues (our art, not
// Mojang's); acacia/dark-oak LEAVES reuse TILE_LEAVES exactly — the 1.7.2
// changelog itself notes both are "visually identical to regular oak
// leaves".
// ---------------------------------------------------------------------------
pub const TILE_STAINED_GLASS_WHITE: u16 = 244;
pub const TILE_STAINED_GLASS_ORANGE: u16 = 245;
pub const TILE_STAINED_GLASS_MAGENTA: u16 = 246;
pub const TILE_STAINED_GLASS_LIGHT_BLUE: u16 = 247;
pub const TILE_STAINED_GLASS_YELLOW: u16 = 248;
pub const TILE_STAINED_GLASS_LIME: u16 = 249;
pub const TILE_STAINED_GLASS_PINK: u16 = 250;
pub const TILE_STAINED_GLASS_GRAY: u16 = 251;
pub const TILE_STAINED_GLASS_LIGHT_GRAY: u16 = 252;
pub const TILE_STAINED_GLASS_CYAN: u16 = 253;
pub const TILE_STAINED_GLASS_PURPLE: u16 = 254;
pub const TILE_STAINED_GLASS_BLUE: u16 = 255;
pub const TILE_STAINED_GLASS_BROWN: u16 = 256;
pub const TILE_STAINED_GLASS_GREEN: u16 = 257;
pub const TILE_STAINED_GLASS_RED: u16 = 258;
pub const TILE_STAINED_GLASS_BLACK: u16 = 259;
pub const TILE_RED_SAND: u16 = 260;
pub const TILE_PACKED_ICE: u16 = 261;
pub const TILE_PODZOL_TOP: u16 = 262;
pub const TILE_PODZOL_SIDE: u16 = 263;
pub const TILE_ACACIA_LOG_SIDE: u16 = 264;
pub const TILE_ACACIA_LOG_TOP: u16 = 265;
pub const TILE_DARK_OAK_LOG_SIDE: u16 = 266;
pub const TILE_DARK_OAK_LOG_TOP: u16 = 267;
pub const TILE_ALLIUM: u16 = 268;
pub const TILE_AZURE_BLUET: u16 = 269;
pub const TILE_BLUE_ORCHID: u16 = 270;
pub const TILE_OXEYE_DAISY: u16 = 271;
pub const TILE_ORANGE_TULIP: u16 = 272;
pub const TILE_RED_TULIP: u16 = 273;
pub const TILE_WHITE_TULIP: u16 = 274;
pub const TILE_PINK_TULIP: u16 = 275;
pub const TILE_SUNFLOWER_LOWER: u16 = 276;
pub const TILE_SUNFLOWER_TOP: u16 = 277;
pub const TILE_LILAC_LOWER: u16 = 278;
pub const TILE_LILAC_TOP: u16 = 279;
pub const TILE_PEONY_LOWER: u16 = 280;
pub const TILE_PEONY_TOP: u16 = 281;
pub const TILE_ROSE_BUSH_LOWER: u16 = 282;
pub const TILE_ROSE_BUSH_TOP: u16 = 283;
pub const TILE_RAW_FISH: u16 = 284;
pub const TILE_RAW_SALMON: u16 = 285;
pub const TILE_CLOWNFISH: u16 = 286;
pub const TILE_PUFFERFISH: u16 = 287;
// ---------------------------------------------------------------------------
// 1.8 bracket tiles (Bountiful Update — live round 2026-09-06): slime,
// coarse dirt, the three polished stones, red sandstone family,
// prismarine family, sea lantern, iron trapdoor, barrier, and the
// rabbit/prismarine item icons.
// ---------------------------------------------------------------------------
pub const TILE_SLIME: u16 = 288;
pub const TILE_COARSE_DIRT: u16 = 289;
pub const TILE_POLISHED_GRANITE: u16 = 290;
pub const TILE_POLISHED_DIORITE: u16 = 291;
pub const TILE_POLISHED_ANDESITE: u16 = 292;
pub const TILE_RED_SANDSTONE: u16 = 293;
pub const TILE_SMOOTH_RED_SANDSTONE: u16 = 294;
pub const TILE_PRISMARINE: u16 = 295;
pub const TILE_PRISMARINE_BRICKS: u16 = 296;
pub const TILE_DARK_PRISMARINE: u16 = 297;
pub const TILE_SEA_LANTERN: u16 = 298;
pub const TILE_IRON_TRAPDOOR: u16 = 299;
pub const TILE_BARRIER: u16 = 300;
pub const TILE_RAW_RABBIT: u16 = 301;
pub const TILE_COOKED_RABBIT: u16 = 302;
pub const TILE_RABBIT_HIDE: u16 = 303;
pub const TILE_RABBIT_FOOT: u16 = 304;
pub const TILE_PRISMARINE_SHARD: u16 = 305;
pub const TILE_PRISMARINE_CRYSTALS: u16 = 306;
/// 1.8 rabbit entity sprite (clean-room, like the other mob tiles)
pub const TILE_RABBIT: u16 = 307;
// ---------------------------------------------------------------------------
// 1.9 bracket tiles (Combat Update — live round 2026-09-06): grass path,
// purpur family, end stone bricks, end rod, chorus plant/flower, and the
// chorus fruit / elytra / shield item icons.
// ---------------------------------------------------------------------------
pub const TILE_GRASS_PATH: u16 = 308;
pub const TILE_GRASS_PATH_SIDE: u16 = 309;
pub const TILE_PURPUR: u16 = 310;
pub const TILE_PURPUR_PILLAR_SIDE: u16 = 311;
pub const TILE_END_STONE_BRICKS: u16 = 312;
pub const TILE_END_ROD: u16 = 313;
pub const TILE_CHORUS_PLANT: u16 = 314;
pub const TILE_CHORUS_FLOWER: u16 = 315;
pub const TILE_CHORUS_FRUIT: u16 = 316;
pub const TILE_ELYTRA: u16 = 317;
pub const TILE_SHIELD: u16 = 318;
// 1.10 Frostburn tiles (live round 2026-09-06)
pub const TILE_MAGMA: u16 = 319;
pub const TILE_NETHER_WART_BLOCK: u16 = 320;
pub const TILE_RED_NETHER_BRICKS: u16 = 321;
pub const TILE_BONE_BLOCK: u16 = 322;
// 1.10 mob sprites
pub const TILE_POLAR_BEAR: u16 = 323;
pub const TILE_STRAY: u16 = 324;
pub const TILE_HUSK: u16 = 325;

// ---- audit-fix round (2026-09-07): the Phase-1/2 audit's missed 1.2/1.4
// content — jungle wood family + vines + ferns (1.2) + golden carrot
// (1.4). All live-verified this round (minecraft.wiki/w/Jungle_Log via
// the Log page, /w/Leaves, /w/Vines, /w/Fern, /w/Golden_Carrot, /w/Tree,
// /w/Ladder; research record in scripts/auditfix_page_*.json). ----
/// Golden Carrot item sprite (VERIFIED w/Golden_Carrot: hunger 6,
/// saturation 14.4).
pub const TILE_GOLDEN_CARROT: u16 = 326;
/// Jungle log bark (VERIFIED w/Log — Jungle Log redirect: hardness 2,
/// blast 2, flammable 5, axe-quickest, smelts to charcoal).
pub const TILE_JUNGLE_LOG_SIDE: u16 = 327;
pub const TILE_JUNGLE_LOG_TOP: u16 = 328;
/// Jungle leaves (VERIFIED w/Leaves: hardness 0.2, flammable 30,
/// jungle-sapling drop rate 2.5% — no saplings in engine, drops nothing).
pub const TILE_JUNGLE_LEAVES: u16 = 329;
pub const TILE_JUNGLE_PLANKS: u16 = 330;
/// Vine (VERIFIED w/Vines: climbable non-solid, hardness 0.2 —
/// cross-rendered in our engine, side-attachment states deferred).
pub const TILE_VINE: u16 = 331;
/// Fern (VERIFIED w/Fern: non-solid, hardness 0, 12.5% wheat-seed drop
/// — no seeds item in engine, drops nothing).
pub const TILE_FERN: u16 = 332;

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
pub const AIR: u16 = 0;
pub const GRASS: u16 = 1;
pub const DIRT: u16 = 2;
pub const STONE: u16 = 3;
pub const COBBLE: u16 = 4;
pub const SAND: u16 = 5;
pub const OAK_LOG: u16 = 6;
pub const PLANKS: u16 = 7;
pub const LEAVES: u16 = 8;
pub const WATER: u16 = 9;
pub const GLASS: u16 = 10;
pub const BEDROCK: u16 = 11;
pub const GRAVEL: u16 = 12;
pub const SNOW: u16 = 13;
pub const SNOW_GRASS: u16 = 14;
pub const TALL_GRASS: u16 = 15;
pub const FLOWER_RED: u16 = 16;
pub const FLOWER_YELLOW: u16 = 17;
// stone family
pub const GRANITE: u16 = 18;
pub const DIORITE: u16 = 19;
pub const ANDESITE: u16 = 20;
pub const STONE_BRICKS: u16 = 21;
pub const BRICKS: u16 = 22;
pub const MOSSY_COBBLE: u16 = 23;
pub const SMOOTH_STONE: u16 = 24;
pub const OBSIDIAN: u16 = 25;
// ores
pub const COAL_ORE: u16 = 26;
pub const IRON_ORE: u16 = 27;
pub const GOLD_ORE: u16 = 28;
pub const DIAMOND_ORE: u16 = 29;
pub const REDSTONE_ORE: u16 = 30;
pub const LAPIS_ORE: u16 = 31;
pub const EMERALD_ORE: u16 = 32;
// mineral blocks
pub const IRON_BLOCK: u16 = 33;
pub const GOLD_BLOCK: u16 = 34;
pub const DIAMOND_BLOCK: u16 = 35;
// misc
pub const GLOWSTONE: u16 = 36;
pub const BOOKSHELF: u16 = 37;
pub const CRAFTING_TABLE: u16 = 38;
pub const CLAY: u16 = 39;
pub const TERRACOTTA: u16 = 40;
pub const PUMPKIN: u16 = 41;
pub const MELON: u16 = 42;
pub const ICE: u16 = 43;
pub const CACTUS: u16 = 44;
// wool
pub const WOOL_WHITE: u16 = 45;
pub const WOOL_RED: u16 = 46;
pub const WOOL_BLUE: u16 = 47;
pub const WOOL_YELLOW: u16 = 48;
pub const WOOL_BLACK: u16 = 49;
// wood variants
pub const BIRCH_LOG: u16 = 50;
pub const BIRCH_LEAVES: u16 = 51;
pub const SPRUCE_LOG: u16 = 52;
pub const SPRUCE_LEAVES: u16 = 53;
// plants
pub const MUSHROOM_RED: u16 = 54;
pub const MUSHROOM_BROWN: u16 = 55;
pub const DEAD_BUSH: u16 = 56;

// (BLOCK_COUNT moved below — after all item ids are declared)

// redstone core (Phase 6 §25 subset)
pub const REDSTONE_WIRE: u16 = 60;
pub const REDSTONE_TORCH: u16 = 61;
pub const LEVER: u16 = 62;
// gameplay (Phase 7)
pub const FURNACE: u16 = 63;
// nether blocks (Phase 7 §28 dimensions): identities collide with the
// log-axis/model state slots exactly like FURNACE — they too always store
// their dedicated STATE ids below
pub const NETHERRACK: u16 = 64;
pub const NETHER_QUARTZ_ORE: u16 = 65;
pub const SOUL_SAND: u16 = 66;
pub const NETHERRACK_STATE: u16 = 118;
pub const QUARTZ_ORE_STATE: u16 = 119;
pub const SOUL_SAND_STATE: u16 = 120;
// brewing (Phase 7 §29): the stand block + potion ITEM ids. Potions live in
// inventories/hotbar only — never stored in the world. Their identity ids
// (67..73) collide with the COBBLE_STAIRS/OAK_FENCE model-state range, so
// like the nether blocks they get dedicated registry states and fold
// through state_block like everything else (§46 defensive folding).
pub const BREWING_STAND: u16 = 67;
pub const POTION_EMPTY: u16 = 68; // "Glass Bottle"
pub const POTION_WATER: u16 = 69; // "Water Bottle"
pub const POTION_AWKWARD: u16 = 70;
pub const POTION_MUNDANE: u16 = 71;
pub const POTION_HEALING: u16 = 72;
pub const POTION_HEALING_II: u16 = 73;
pub const BREWING_STAND_STATE: u16 = 121;
pub const POTION_EMPTY_STATE: u16 = 122;
pub const POTION_WATER_STATE: u16 = 123;
pub const POTION_AWKWARD_STATE: u16 = 124;
pub const POTION_MUNDANE_STATE: u16 = 125;
pub const POTION_HEALING_STATE: u16 = 126;
pub const POTION_HEALING_II_STATE: u16 = 127;
// enchanting (Phase 7 §29): table block + the book item-block (same
// dedicated-state pattern; the book carries the enchant in ItemStack.ench)
pub const ENCHANT_TABLE: u16 = 74;
pub const ENCHANTED_BOOK: u16 = 75;
pub const ENCHANT_TABLE_STATE: u16 = 128;
pub const ENCHANTED_BOOK_STATE: u16 = 129;
// mob drops (Phase 2): item-only ids in the potion pattern — they live in
// inventories/hotbar, never stored in the world. Registered names are
// vanilla registry strings (mechanical data, safe to match); the art is ours.
pub const BEEF: u16 = 76;
pub const PORKCHOP: u16 = 77;
pub const MUTTON: u16 = 78;
pub const CHICKEN_RAW: u16 = 79;
pub const FEATHER: u16 = 80;
pub const LEATHER: u16 = 81;
pub const BONE: u16 = 82;
pub const STRING: u16 = 83;
pub const GUNPOWDER: u16 = 84;
pub const ENDER_PEARL: u16 = 85;
pub const ROTTEN_FLESH: u16 = 86;
pub const ARROW_ITEM: u16 = 87;
// redstone components (Phase 3): ids 88..=96, dedicated sim states above
pub const REPEATER: u16 = 88;
pub const COMPARATOR: u16 = 89;
pub const PISTON: u16 = 90;
pub const STICKY_PISTON: u16 = 91;
pub const DISPENSER: u16 = 92;
pub const DROPPER: u16 = 93;
pub const OBSERVER: u16 = 94;
pub const HOPPER: u16 = 95;
pub const CHEST: u16 = 96;
// brewing expansion (Phase 4 §26/§30): the corruption chain + its items
pub const POTION_HARMING: u16 = 97;
pub const POTION_HARMING_II: u16 = 98;
pub const SPIDER_EYE: u16 = 99;
pub const FERMENTED_SPIDER_EYE: u16 = 100;
/// Phase 5 §27: monster spawner (dungeon block entity). Mob type is
/// encoded in the block state (232 zombie / 233 skeleton / 234 spider).
pub const SPAWNER: u16 = 101;
/// Phase 10: end-portal frame block (stronghold portal room ring).
/// Decorative-only: eye-of-ender insertion + portal activation are out
/// of scope (documented); the frame marks the vanilla portal room's
/// 12-frame ring, ours renders as a full cube with a frame inset.
pub const END_PORTAL_FRAME: u16 = 102;

// ---- Phase E1 block ids (evolution 1.0–1.2 bracket) — all values
// live-verified against minecraft.wiki on 2026-09-06 (see
// docs/research/phase1-1.0-1.2-research.md for the per-claim audit) ----
/// Mycelium — mushroom-fields surface block. Spreads to dirt (1 up /
/// 1 sideways / 3 down, light gates 9/4 — VERIFIED w/Mycelium §Spread).
/// Drops DIRT without Silk Touch (adaptation: no Silk Touch in engine).
pub const MYCELIUM: u16 = 103;
/// End stone — hardness 3, blast resistance 9 (VERIFIED w/End_Stone).
pub const END_STONE: u16 = 104;
/// Nether bricks — the fortress structural block.
pub const NETHER_BRICKS: u16 = 105;
/// Redstone lamp — light 0 when off; the LIT state emits 15. Turns on
/// instantly, off after 4 game ticks (VERIFIED w/Redstone_Lamp: "takes
/// 4 ticks (0.2 seconds) to turn off in Java Edition"; the 1.2.4
/// history note "2-tick delay" = 2 redstone ticks = the same 4 game
/// ticks). Crafted 4 glowstone + 1 redstone dust.
pub const REDSTONE_LAMP: u16 = 106;
/// Chiseled stone bricks — decorative variant (recipe needs stone-brick
/// slabs, out of engine scope; picker-only, documented).
pub const CHISELED_STONE_BRICKS: u16 = 107;
/// Chiseled sandstone (2 sandstone slabs — slabless engine: picker-only).
pub const CHISELED_SANDSTONE: u16 = 108;
/// Cut sandstone — 2×2 sandstone → 4 (craftable).
pub const CUT_SANDSTONE: u16 = 109;
/// Smooth sandstone — smelt sandstone (1.14+ recipe, valid for 1.16.5).
pub const SMOOTH_SANDSTONE: u16 = 110;
/// Huge red mushroom cap block.
pub const MUSHROOM_RED_BLOCK: u16 = 111;
/// Huge brown mushroom cap block.
pub const MUSHROOM_BROWN_BLOCK: u16 = 112;
/// Huge mushroom stem.
pub const MUSHROOM_STEM: u16 = 113;
/// Nether wart crop — 4 stages (age 0..3), 10%/random-tick growth, only
/// on soul sand (VERIFIED w/Nether_Wart). Storage states 237..=240.
pub const NETHER_WART: u16 = 114;
/// Dragon egg — spawns above the End exit portal after the first dragon
/// kill (light level 1).
pub const DRAGON_EGG: u16 = 115;
/// End portal block — the 3×3 active portal in the stronghold room /
/// the End exit portal. Emissive 15. Entering it dimension-travels.
pub const END_PORTAL: u16 = 116;
// ---- Phase E1 item-blocks (inventory-only, the potion pattern) ----
pub const END_CRYSTAL: u16 = 117;
pub const EYE_OF_ENDER: u16 = 118;
pub const BLAZE_ROD: u16 = 119;
pub const BLAZE_POWDER: u16 = 120;
pub const GOLDEN_APPLE: u16 = 121;
pub const SNOWBALL: u16 = 122;
pub const NETHER_BRICK: u16 = 123;
// spawn eggs: ids 124..=143, one per implemented mob kind (20).
/// Vanilla mechanic (VERIFIED w/Spawn_Egg §Usage): use on a surface →
/// the mob spawns with feet adjacent to the surface; the egg is
/// consumed. Creative-picker item.
pub const SPAWN_EGG_BASE: u16 = 124;
pub const SPAWN_EGG_MAX: u16 = 143;
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
pub const ANVIL: u16 = 144;
/// Chipped Anvil — damage stage 1.
pub const CHIPPED_ANVIL: u16 = 145;
/// Damaged Anvil — damage stage 2 (next degrade = destroyed).
pub const DAMAGED_ANVIL: u16 = 146;
/// Beacon — light 15, pyramid 1–4 levels, effects (VERIFIED w/Beacon).
/// Craft: 5 glass + 1 nether star + 3 obsidian.
pub const BEACON: u16 = 147;
/// Cobblestone Wall — 6 cobble → 6 walls (VERIFIED w/Wall); 1.5-block
/// collision like fences; connects to neighbors at mesh time.
pub const COBBLE_WALL: u16 = 148;
/// Ender Chest — 27 slots, per-player, shared across all ender chests
/// (VERIFIED w/Ender_Chest). Craft: 8 obsidian + eye of ender. Light 7.
/// Break drops 8 obsidian (no Silk Touch in engine — documented).
pub const ENDER_CHEST: u16 = 149;
/// Flower Pot — craft 3 bricks (VERIFIED w/Flower_Pot; brick ITEM →
/// brick BLOCK adaptation, documented); hardness 0, instant break.
pub const FLOWER_POT: u16 = 150;
/// Item Frame — craft 8 sticks + 1 leather (VERIFIED w/Item_Frame;
/// stick item absent → planks stand-in, documented). Displays the item
/// placed in it; interact rotates 45°.
pub const ITEM_FRAME: u16 = 151;
/// Tripwire Hook — craft 1 iron + 1 stick + 2 planks → 2 (VERIFIED
/// w/Tripwire_Hook; iron ore + planks adaptation). Pairs + a 1–40
/// string line emit redstone while tripped.
pub const TRIPWIRE_HOOK: u16 = 152;
/// Wither Skeleton Skull — the wither-summon block (2.5% drop, VERIFIED
/// w/Wither_Skeleton); hardness 1.
pub const WITHER_SKELETON_SKULL: u16 = 153;
/// Command Block — creative/`give` only (VERIFIED w/Command_Block);
/// executes the engine command bridge on redstone pulse. Impulse variant
/// (chain/repeating are 1.9 — deferred).
pub const COMMAND_BLOCK: u16 = 154;
// ---- Phase E2 item-blocks (inventory-only, the potion pattern) ----
/// Emerald — ore drop + beacon feed + trade currency (VERIFIED
/// w/Emerald, w/Emerald_Ore: drops 1, XP 3–7).
pub const EMERALD: u16 = 155;
/// Nether Star — the wither's guaranteed drop (VERIFIED w/Wither:
/// 100%, 50 XP, 10-min despawn); beacon ingredient.
pub const NETHER_STAR: u16 = 156;
/// Potato — food 1 / 0.6 (VERIFIED w/Food).
pub const POTATO: u16 = 157;
/// Baked Potato — food 5 / 6.0 (VERIFIED w/Food); smelted from potato.
pub const BAKED_POTATO: u16 = 158;
/// Carrot — food 3 / 3.6 (VERIFIED w/Food).
pub const CARROT: u16 = 159;
/// Pumpkin Pie — food 8 / 4.8 (VERIFIED w/Food). Recipe needs sugar +
/// egg (absent in engine) → picker-only, recipe deferred (documented).
pub const PUMPKIN_PIE: u16 = 160;
/// Lava — light-emitting fluid (VERIFIED w/Lava infobox: luminance 15,
/// transparent, flow distance 4 blocks Overworld/End & 8 Nether —
/// counted including the source, i.e. 3/7 spread; flow speed 30/10
/// ticks per block; contact damage 4 HP per 10 ticks via the damage
/// immunity window). States: source 307 + flow levels 1..7 at 308..=314.
pub const LAVA: u16 = 161;
/// Coal — the fuel item (VERIFICATION-REPORT mechanical fix #4).
/// VERIFIED live 2026-09-06 (minecraft.wiki/w/Furnace "a piece of coal
/// burns for 80 seconds and can process eight items"; w/Smelting fuel
/// table "Coal 1600 ticks / 8 items"): fuel_ticks = 1600 = 80 s × 20 tps.
/// Obtained by smelting coal ore (vanilla recipe, w/Smelting "coal ore
/// → coal, 0.1 XP"). Inventory-only item-block, the E2 pattern.
pub const COAL: u16 = 162;
// ---- Phase E3 (evolution 1.5–1.6 bracket, live-verified 2026-09-06) ----
/// Block of Coal — fuel 16000 ticks / 800 s / 80 items (VERIFIED live
/// 2026-09-06: minecraft.wiki/w/Block_of_Coal "One block of coal lasts
/// 800 seconds (16000 ticks), which smelts 80 items" — 10× the coal
/// item's 1600). Craft: 9 coal ↔ 1 block (w/Block_of_Coal recipe).
pub const COAL_BLOCK: u16 = 163;
/// Block of Quartz — craft 4 nether quartz (VERIFIED live 2026-09-06,
/// minecraft.wiki/w/Block_of_Quartz).
pub const QUARTZ_BLOCK: u16 = 164;
/// Chiseled Quartz Block — vanilla crafts from 2 quartz slabs
/// (w/Chiseled_Quartz_Block); the engine has no quartz-slab model →
/// picker-only this bracket, recipe deferred (disclosed).
pub const CHISELED_QUARTZ: u16 = 165;
/// Quartz Pillar — craft 2 blocks of quartz → 2 pillars (VERIFIED live
/// 2026-09-06: minecraft.wiki/w/Quartz_Pillar "Block of Quartz 2";
/// output count 2 confirmed by a second live source).
pub const QUARTZ_PILLAR: u16 = 166;
/// Stained Terracotta — 16 colors (ids 166..=181). VERIFIED live
/// 2026-09-06: minecraft.wiki/w/Terracotta "comes in the sixteen dye
/// colors ... found abundantly in badlands biomes"; crafting needs
/// dye (no dye system in the engine — recipe deferred, disclosed);
/// Badlands banding generation is the natural acquisition path.
/// Order = the vanilla dye-color registry order.
pub const STAINED_TERRACOTTA_BASE: u16 = 167;
pub const STAINED_TERRACOTTA_END: u16 = 182;
/// Carpets — the 5 engine wool colors (vanilla has 16; the engine wool
/// palette is 5 — carpets match it 1:1, adaptation disclosed). Craft
/// 2 wool → 3 carpets (VERIFIED live: minecraft.wiki/w/Carpet "13w17a
/// The crafting recipe of carpets now returns 3 carpets from two
/// wool"). Hitbox height 1/16 block (VERIFIED: "14w29a Carpets now
/// have a hitbox height of 1⁄16 of a block") — rendered as a thin
/// non-solid floor overlay (engine blocks are full-height; disclosed).
pub const CARPET_WHITE: u16 = 183;
pub const CARPET_RED: u16 = 184;
pub const CARPET_YELLOW: u16 = 185;
pub const CARPET_BLUE: u16 = 186;
pub const CARPET_BLACK: u16 = 187;
pub const CARPET_BASE: u16 = 183;
/// Hay Bale — fall damage reduced by 80% (take 20%: VERIFIED live
/// 2026-09-06, minecraft.wiki/w/Hay_Bale "Falling onto a hay bale
/// reduces the fall damage by 80%, meaning whatever falls on a hay
/// bale takes 20% of the normal fall damage"). Craft 9 wheat — no
/// wheat/farming in the engine → recipe deferred, picker/loot only
/// (disclosed). Feeds horses (mobs.rs).
pub const HAY_BALE: u16 = 188;
/// Daylight Sensor — redstone signal from sky light; recipe glass +
/// nether quartz + any wooden slab (VERIFIED live: minecraft.wiki/w/
/// Daylight_Detector "Glass + Nether Quartz + Any Wooden Slab").
/// Java 1.16.5 signal: driven by time-of-day + weather + sky exposure
/// (the engine maps its real sky-light engine through the same 0–15
/// ladder — adaptation disclosed in redstone.rs).
pub const DAYLIGHT_SENSOR: u16 = 189;
/// Trapped Chest — container + redstone signal = number of players
/// viewing, max 15 (VERIFIED live: minecraft.wiki/w/Trapped_Chest "to
/// a power level equal to the number of players ... accessing the
/// trapped chest at once (maximum 15)"). Recipe: tripwire hook +
/// chest (VERIFIED, same page).
pub const TRAPPED_CHEST: u16 = 190;
/// Light Weighted Pressure Plate (gold) — signal = entity count on
/// the plate, 1..15 (VERIFIED live: minecraft.wiki/w/
/// Light_Weighted_Pressure_Plate "signal strength ... range from 1 to
/// 15", "signal strength from a light weighted pressure plate does
/// not vary with the type of entity"). Craft: 2 gold — engine has no
/// ingots, 2 GOLD_ORE instead (the E2 ore-block convention, disclosed).
pub const LIGHT_WEIGHTED_PLATE: u16 = 191;
/// Heavy Weighted Pressure Plate (iron) — signal = ceil(entities/10),
/// 1..15 (VERIFIED live: minecraft.wiki/w/Heavy_Weighted_Pressure_Plate
/// "equal to 1⁄10 of the amount of entities on top of them (rounded
/// up to the nearest integer), up to a maximum power level of 15").
/// Craft: 2 iron — 2 IRON_ORE (convention, disclosed).
pub const HEAVY_WEIGHTED_PLATE: u16 = 192;
/// Block of Redstone — always-on power source, weak power 15 to direct
/// neighbors (VERIFIED live: minecraft.wiki/w/Block_of_Redstone "acts
/// as a permanently powered redstone power source", "provide weak
/// power to their direct neighbors at signal strength 15"). Craft
/// 9 redstone ↔ 1 block — engine redstone dust is the WIRE block →
/// 9 REDSTONE_WIRE (adaptation, disclosed).
pub const REDSTONE_BLOCK: u16 = 193;
/// Nether Quartz — item dropped by nether quartz ore (VERIFIED live:
/// minecraft.wiki/w/Nether_Quartz_Ore "it drops 1 Nether quartz";
/// ore XP 2–5 from the same page). Quartz-block ingredient.
pub const NETHER_QUARTZ: u16 = 194;
/// Lead — leash item (VERIFIED live: minecraft.wiki/w/Lead "A lead can
/// stretch a maximum of 12 blocks" on the CURRENT wiki — but that 12
/// value is the 2025 "Chase the Skies" buff ("Leash snapping distance
/// has been increased to 12 blocks", minecraft.wiki/w/Lead §History);
/// for the 1.16.5 target the value is 10 blocks — version-scoped, both
/// cited). Craft 4 string + 1 slimeball — no slimeballs in the engine
/// → recipe deferred, picker-only (disclosed).
pub const LEAD: u16 = 195;
/// Saddle — required to CONTROL a tamed horse (VERIFIED live:
/// minecraft.wiki/w/Horse "Once a horse is tamed and saddled, the
/// player can control it"; w/Riding). Not craftable in vanilla —
/// dungeon-chest loot + picker (the engine loot path).
pub const SADDLE: u16 = 196;
/// E3 spawn eggs (horse/donkey/mule — kinds 20..=22). The legacy egg
/// window 124..=143 is full, so these live at 197..=199 with their own
/// state arithmetic (the E2 item-block pattern).
pub const E3_SPAWN_EGG_BASE: u16 = 197;
pub const E3_SPAWN_EGG_END: u16 = 199;
pub const E3_EGG_HORSE: u16 = 197;
pub const E3_EGG_DONKEY: u16 = 198;
pub const E3_EGG_MULE: u16 = 199;
// ---------------------------------------------------------------------------
// 1.7.2 bracket — V2 block window (ids 200..=242 [merged renumber past the E-series], states 236..=294).
// The pre-1.7 registry is fully allocated (identity states 0..=56, model
// states 57..=88, sim states 89..=235). New blocks get ids past 102 that
// [merge 2026-09-06] the F-series (1.7.2-1.10) block ids were shifted
// past the E-series backfill (ids 0..=199); our 16 stained-terracotta
// blocks were DROPPED as exact duplicates of the E3 set
// (STAINED_TERRACOTTA_BASE 167..=182) — V2_COUNT 59 -> 43.
// Block ids widened u8 -> u16 at this merge: 276 ids > the u8 ceiling.
// never live in the world as identity states — each stores its dedicated
// V2 state, folded through V2_STATE_TO_BLOCK (the table IS the state→block
// mapping; `Chunk::get` now folds every state through state_block so u16
// states above 255 are safe).
// All content verified against minecraft.wiki/w/Java_Edition_1.7.2
// (live round, 2026-09-06).
// ---------------------------------------------------------------------------

// stained glass, 16 colors — translucent like glass (solid, non-opaque)
pub const STAINED_GLASS_WHITE: u16 = 200;
pub const STAINED_GLASS_ORANGE: u16 = 201;
pub const STAINED_GLASS_MAGENTA: u16 = 202;
pub const STAINED_GLASS_LIGHT_BLUE: u16 = 203;
pub const STAINED_GLASS_YELLOW: u16 = 204;
pub const STAINED_GLASS_LIME: u16 = 205;
pub const STAINED_GLASS_PINK: u16 = 206;
pub const STAINED_GLASS_GRAY: u16 = 207;
pub const STAINED_GLASS_LIGHT_GRAY: u16 = 208;
pub const STAINED_GLASS_CYAN: u16 = 209;
pub const STAINED_GLASS_PURPLE: u16 = 210;
pub const STAINED_GLASS_BLUE: u16 = 211;
pub const STAINED_GLASS_BROWN: u16 = 212;
pub const STAINED_GLASS_GREEN: u16 = 213;
pub const STAINED_GLASS_RED: u16 = 214;
pub const STAINED_GLASS_BLACK: u16 = 215;
// stained terracotta, 16 colors — "stained clay" in 1.7 parlance
pub const RED_SAND: u16 = 216;
pub const PACKED_ICE: u16 = 217;
pub const PODZOL: u16 = 218;
pub const ACACIA_LOG: u16 = 219;
pub const ACACIA_LEAVES: u16 = 220;
pub const DARK_OAK_LOG: u16 = 221;
pub const DARK_OAK_LEAVES: u16 = 222;
// the 8 new small flowers (1.7.2: allium, azure bluet, blue orchid,
// oxeye daisy + 4 tulips; poppy already exists as FLOWER_RED)
pub const ALLIUM: u16 = 223;
pub const AZURE_BLUET: u16 = 224;
pub const BLUE_ORCHID: u16 = 225;
pub const OXEYE_DAISY: u16 = 226;
pub const ORANGE_TULIP: u16 = 227;
pub const RED_TULIP: u16 = 228;
pub const WHITE_TULIP: u16 = 229;
pub const PINK_TULIP: u16 = 230;
// the 4 two-block-tall flowers, lower + upper halves (vanilla models one
// block with half=lower/upper; ours is two ids — same observable shape)
pub const SUNFLOWER: u16 = 231;
pub const SUNFLOWER_TOP: u16 = 232;
pub const LILAC: u16 = 233;
pub const LILAC_TOP: u16 = 234;
pub const PEONY: u16 = 235;
pub const PEONY_TOP: u16 = 236;
pub const ROSE_BUSH: u16 = 237;
pub const ROSE_BUSH_TOP: u16 = 238;
// 1.7.2 fish items — inventory-only (never placed):
// VERIFIED (wiki §Items): clownfish restores 1, raw salmon 2 (cooked 6),
// pufferfish 1 + Poison IV 1:00 + Hunger III 0:15 + Nausea 0:15 and brews
// Water Breathing; raw fish (cod) restores 2.
pub const RAW_FISH: u16 = 239;
pub const RAW_SALMON: u16 = 240;
pub const CLOWNFISH: u16 = 241;
pub const PUFFERFISH: u16 = 242;

/// first state of the V2 window (1.7.2 bracket)
pub const V2_STATE_BASE: u16 = 400;
/// V2 state count: one dedicated state per new block, order = id order
pub const V2_COUNT: u16 = 43; // ids 200..=242 [merged renumber past the E-series]
/// state → block fold table for the V2 window. Index = state − BASE.
pub const V2_STATE_TO_BLOCK: [u16; V2_COUNT as usize] = [
    STAINED_GLASS_WHITE, STAINED_GLASS_ORANGE, STAINED_GLASS_MAGENTA,
    STAINED_GLASS_LIGHT_BLUE, STAINED_GLASS_YELLOW, STAINED_GLASS_LIME,
    STAINED_GLASS_PINK, STAINED_GLASS_GRAY, STAINED_GLASS_LIGHT_GRAY,
    STAINED_GLASS_CYAN, STAINED_GLASS_PURPLE, STAINED_GLASS_BLUE,
    STAINED_GLASS_BROWN, STAINED_GLASS_GREEN, STAINED_GLASS_RED,
    STAINED_GLASS_BLACK,
    RED_SAND, PACKED_ICE, PODZOL, ACACIA_LOG, ACACIA_LEAVES, DARK_OAK_LOG,
    DARK_OAK_LEAVES,
    ALLIUM, AZURE_BLUET, BLUE_ORCHID, OXEYE_DAISY, ORANGE_TULIP, RED_TULIP,
    WHITE_TULIP, PINK_TULIP,
    SUNFLOWER, SUNFLOWER_TOP, LILAC, LILAC_TOP, PEONY, PEONY_TOP,
    ROSE_BUSH, ROSE_BUSH_TOP,
    RAW_FISH, RAW_SALMON, CLOWNFISH, PUFFERFISH,
];

/// default (and only) V2 state of a block id — `b − 200 + V2_STATE_BASE`.
/// Returns None for pre-V2 blocks.
#[inline]
pub fn v2_state(b: u16) -> Option<u16> {
    if (200..200 + V2_COUNT as u16).contains(&b) {
        Some(V2_STATE_BASE + (b - 200) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v2_state(s: u16) -> bool {
    (V2_STATE_BASE..V2_STATE_BASE + V2_COUNT).contains(&s)
}

// ---------------------------------------------------------------------------
// 1.8 bracket — the Bountiful Update (2014-09-02,
// minecraft.wiki/w/Java_Edition_1.8, live round 2026-09-06). V3 window:
// ids 243..=261, states 447..=465 [merged renumber] (after the V2 log-axis states).
// ---------------------------------------------------------------------------
pub const SLIME_BLOCK: u16 = 243;
pub const COARSE_DIRT: u16 = 244;
pub const POLISHED_GRANITE: u16 = 245;
pub const POLISHED_DIORITE: u16 = 246;
pub const POLISHED_ANDESITE: u16 = 247;
pub const RED_SANDSTONE: u16 = 248;
pub const SMOOTH_RED_SANDSTONE: u16 = 249;
pub const PRISMARINE: u16 = 250;
pub const PRISMARINE_BRICKS: u16 = 251;
pub const DARK_PRISMARINE: u16 = 252;
/// sea lantern: light level 15 (wiki: "Emit light at a light level of 15")
pub const SEA_LANTERN: u16 = 253;
/// iron trapdoor: redstone-only opening (vanilla) — our redstone gates are
/// documented as deferred; the block places/plays like a trapdoor
pub const IRON_TRAPDOOR: u16 = 254;
/// barrier: bedrock-like, fully transparent, creative-only (wiki)
pub const BARRIER: u16 = 255;
// 1.8 rabbit items (inventory-only): VERIFIED §Items — raw rabbit 3,
// cooked 5, hide crafts to leather, foot brews Leaping
pub const RAW_RABBIT: u16 = 256;
pub const COOKED_RABBIT: u16 = 257;
pub const RABBIT_HIDE: u16 = 258;
pub const RABBIT_FOOT: u16 = 259;
// 1.8 prismarine materials (inventory-only; guardian drops — guardians
// are a documented deferral, the items register now so recipes exist)
pub const PRISMARINE_SHARD: u16 = 260;
pub const PRISMARINE_CRYSTALS: u16 = 261;

/// first state of the V3 window (1.8 bracket)
pub const V3_STATE_BASE: u16 = 447;
pub const V3_COUNT: u16 = 19; // ids 162..=180
pub const V3_STATE_TO_BLOCK: [u16; V3_COUNT as usize] = [
    SLIME_BLOCK, COARSE_DIRT, POLISHED_GRANITE, POLISHED_DIORITE, POLISHED_ANDESITE,
    RED_SANDSTONE, SMOOTH_RED_SANDSTONE, PRISMARINE, PRISMARINE_BRICKS, DARK_PRISMARINE,
    SEA_LANTERN, IRON_TRAPDOOR, BARRIER,
    RAW_RABBIT, COOKED_RABBIT, RABBIT_HIDE, RABBIT_FOOT, PRISMARINE_SHARD,
    PRISMARINE_CRYSTALS,
];

#[inline]
pub fn v3_state(b: u16) -> Option<u16> {
    if (243..243 + V3_COUNT as u16).contains(&b) {
        Some(V3_STATE_BASE + (b - 243) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v3_state(s: u16) -> bool {
    (V3_STATE_BASE..V3_STATE_BASE + V3_COUNT).contains(&s)
}

// ---------------------------------------------------------------------------
// 1.9 bracket — the Combat Update (2016-02-29,
// minecraft.wiki/w/Java_Edition_1.9, live round 2026-09-06). V4 window:
// ids 262..=271, states 466..=475 [merged renumber].
// ---------------------------------------------------------------------------
/// grass path: "15/16 of a block (15 pixels) tall. Obtainable by using a
/// shovel on a grass block" (wiki §Blocks). Full-cube render + collision
/// (documented simplification); vanilla always drops dirt even with Silk
/// Touch — our drops match via the break path.
pub const GRASS_PATH: u16 = 262;
pub const PURPUR_BLOCK: u16 = 263;
pub const PURPUR_PILLAR: u16 = 264;
pub const END_STONE_BRICKS: u16 = 265;
/// end rod: "lighting source with the same brightness as torches" (14)
pub const END_ROD: u16 = 266;
pub const CHORUS_PLANT: u16 = 267;
pub const CHORUS_FLOWER: u16 = 268;
/// chorus fruit: eat + random teleport (VERIFIED §Items: heals 4, "can be
/// eaten even if the player is not hungry... teleports the player to a
/// random nearby location")
pub const CHORUS_FRUIT: u16 = 269;
/// elytra: gliding wings (§Items: "they function according to hang glider
/// aerodynamics" — chest slot in vanilla; our adaptation: active when the
/// SELECTED item while falling, documented)
pub const ELYTRA: u16 = 270;
/// shield: "new tool used for blocking incoming attacks" (§Items; crafted
/// 6 planks + 1 iron — our craft hook is a documented deferral, the item
/// blocks while held + right-click)
pub const SHIELD: u16 = 271;

pub const V4_STATE_BASE: u16 = 466;
pub const V4_COUNT: u16 = 10; // ids 181..=190
pub const V4_STATE_TO_BLOCK: [u16; V4_COUNT as usize] = [
    GRASS_PATH, PURPUR_BLOCK, PURPUR_PILLAR, END_STONE_BRICKS, END_ROD,
    CHORUS_PLANT, CHORUS_FLOWER, CHORUS_FRUIT, ELYTRA, SHIELD,
];

#[inline]
pub fn v4_state(b: u16) -> Option<u16> {
    if (262..262 + V4_COUNT as u16).contains(&b) {
        Some(V4_STATE_BASE + (b - 262) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v4_state(s: u16) -> bool {
    (V4_STATE_BASE..V4_STATE_BASE + V4_COUNT).contains(&s)
}

// ---------------------------------------------------------------------------
// 1.10 bracket — the Frostburn Update (2016-06-08,
// minecraft.wiki/w/Java_Edition_1.10, live round 2026-09-06). V5 window:
// ids 272..=275, states 476..=479 [merged renumber].
// ---------------------------------------------------------------------------
/// magma block — VERIFIED (wiki /w/Magma_Block, live 2026-09-06): emits
/// light level 3; "mobs and players take 1 HP damage every second while
/// touching it, similar to a cactus"; sneaking / Frost Walker / Fire
/// Resistance grant immunity; Nether: 4 blobs per chunk between Y=27-36
pub const MAGMA_BLOCK: u16 = 272;
pub const NETHER_WART_BLOCK: u16 = 273;
pub const RED_NETHER_BRICKS: u16 = 274;
pub const BONE_BLOCK: u16 = 275;

// ---- audit-fix round (2026-09-07): ids 276..=281, V6 window 480..=485.
// Phase-1/2 evolution audit found these silently absent from the 1.2 and
// 1.4 brackets (never implemented, never deferred — the audit report and
// the WORKLOG entry document the finding + the fixes). ----
/// Golden Carrot — food 6 / 14.4 (VERIFIED live 2026-09-07,
/// minecraft.wiki/w/Golden_Carrot infobox: "Hunger 6", "Saturation
/// 14.4"; consumption 32 game ticks). Added Java 1.4.2 12w34a (VERIFIED
/// w/Golden_Carrot §History). Craft = gold nugget + carrot (no gold
/// nuggets in engine → picker-only, recipe deferred, documented).
/// Feeds/breeds/heals horses per "Golden carrots are used to tame,
/// breed, lead, grow, and heal horses, donkeys, and mules" (VERIFIED
/// w/Golden_Carrot §Usage; the breeding rule also live-verified in the
/// E3 round w/Horse §Breeding: "Feeding two tamed horses golden apples
/// or golden carrots activates love mode").
pub const GOLDEN_CARROT: u16 = 276;
/// Jungle Log — hardness 2, blast 2, flammable (5), axe-quickest,
/// smelts to charcoal, fuel (VERIFIED w/Log — "Jungle Log" redirect;
/// the E1-era comment "vanilla jungle wood is palette-absent" is now
/// obsolete). Trees added Java 1.2.1 12w03a (VERIFIED w/Tree §History);
/// 1×1 trunk "can extend up to 10 blocks tall" (VERIFIED w/Jungle_Tree
/// search round: "Regular jungle trees... 1×1 trunk, which can extend
/// up to 10 blocks tall"). Axis X/Z placement states omitted (vertical
/// placement unaffected — disclosed simplification).
pub const JUNGLE_LOG: u16 = 277;
/// Jungle Leaves — hardness 0.2, blast 0.2, transparent, flammable (30)
/// (VERIFIED w/Leaves). Drops nothing without shears (jungle-sapling
/// 2.5%/sticks 2% rows VERIFIED; neither item exists in engine —
/// documented).
pub const JUNGLE_LEAVES: u16 = 278;
/// Jungle Planks — same family stats as other planks (1 jungle log →
/// 4 planks, the universal recipe).
pub const JUNGLE_PLANKS: u16 = 279;
/// Vine — climbable non-solid block (VERIFIED w/Vines: "Vines are
/// climbable non-solid vegetation blocks that grow on walls"); climb
/// = ladder physics (VERIFIED w/Vines §History 12w04a: "Players are
/// now slowed when going through vines due to their nature of being a
/// collisionless ladder" + w/Ladder §Climbing: up ~2.35 b/s, max
/// descent ~3 b/s). Cross-rendered adaptation (side-attachment block
/// states deferred with the sapling class).
pub const VINE: u16 = 280;
/// Fern — non-solid plant, hardness 0 (VERIFIED w/Fern: "non-solid
/// plant blocks... same characteristics as grass"); 12.5% wheat-seed
/// drop (no seeds item — drops nothing, documented); placed on
/// grass/dirt family (VERIFIED w/Fern §Placement).
pub const FERN: u16 = 281;

pub const V5_STATE_BASE: u16 = 476;
pub const V5_COUNT: u16 = 4; // ids 191..=194
pub const V5_STATE_TO_BLOCK: [u16; V5_COUNT as usize] = [
    MAGMA_BLOCK, NETHER_WART_BLOCK, RED_NETHER_BRICKS, BONE_BLOCK,
];

#[inline]
pub fn v5_state(b: u16) -> Option<u16> {
    if (272..272 + V5_COUNT as u16).contains(&b) {
        Some(V5_STATE_BASE + (b - 272) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v5_state(s: u16) -> bool {
    (V5_STATE_BASE..V5_STATE_BASE + V5_COUNT).contains(&s)
}

pub const V6_STATE_BASE: u16 = 480;
pub const V6_COUNT: u16 = 6; // ids 276..=281 (audit-fix round)
pub const V6_STATE_TO_BLOCK: [u16; V6_COUNT as usize] = [
    GOLDEN_CARROT, JUNGLE_LOG, JUNGLE_LEAVES, JUNGLE_PLANKS, VINE, FERN,
];

#[inline]
pub fn v6_state(b: u16) -> Option<u16> {
    if (276..276 + V6_COUNT as u16).contains(&b) {
        Some(V6_STATE_BASE + (b - 276) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v6_state(s: u16) -> bool {
    (V6_STATE_BASE..V6_STATE_BASE + V6_COUNT).contains(&s)
}

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
/// coal ITEM state (VERIFICATION-REPORT fix #4 — dedicated slot; the
/// identity id 162 collides with repeater states like every E2 block)
pub const COAL_STATE: u16 = 316;

// ---- Phase E3 states (317..=353; ids 163..=209 all collide with the
// legacy sim-state ranges 142..=227 — the E2 dedicated-state pattern) ----
/// block of coal
pub const COAL_BLOCK_STATE: u16 = 317;
/// block of quartz
pub const QUARTZ_BLOCK_STATE: u16 = 318;
/// chiseled quartz block
pub const CHISELED_QUARTZ_STATE: u16 = 319;
/// quartz pillar
pub const QUARTZ_PILLAR_STATE: u16 = 320;
/// stained terracotta, 16 colors (321..=336) — `state = 321 + color`
pub const TERRACOTTA_STAINED_STATE_BASE: u16 = 321;
pub const TERRACOTTA_STAINED_STATE_END: u16 = 336;
/// carpets — the 5 engine wool colors (337..=341):
/// `state = 337 + (block - 182)`
pub const CARPET_STATE_BASE: u16 = 337;
pub const CARPET_STATE_END: u16 = 341;
/// hay bale
pub const HAY_BALE_STATE: u16 = 342;
/// daylight sensor
pub const DAYLIGHT_SENSOR_STATE: u16 = 343;
/// trapped chest
pub const TRAPPED_CHEST_STATE: u16 = 344;
/// trapped chest OPEN (a viewer present — signal 1, the single-player
/// form of the VERIFIED "power level equal to the number of players");
/// folds to TRAPPED_CHEST
pub const TRAPPED_CHEST_OPEN_STATE: u16 = 354;
/// light weighted pressure plate
pub const LIGHT_PLATE_STATE: u16 = 345;
/// heavy weighted pressure plate
pub const HEAVY_PLATE_STATE: u16 = 346;
/// block of redstone
pub const REDSTONE_BLOCK_STATE: u16 = 347;
/// nether-quartz ITEM state (never world-stored)
pub const NETHER_QUARTZ_STATE: u16 = 348;
/// lead ITEM state (never world-stored)
pub const LEAD_STATE: u16 = 349;
/// saddle ITEM state (never world-stored)
pub const SADDLE_STATE: u16 = 350;
/// E3 spawn-egg ITEM states 351..=353 (never world-stored):
/// `state = 351 + (block - 207)`
pub const E3_EGG_STATE_BASE: u16 = 351;
pub const E3_EGG_STATE_END: u16 = 353;

// ---- Phase E3 POWER states (the vanilla `power` blockstate pattern:
// the sensor/plates/TRAPPED-CHEST signal lives in the state so the
// stateless wire re-derivation reads it as a real source) ----
/// daylight sensor power 1..=15 (355..=369); power 0 = the idle
/// DAYLIGHT_SENSOR_STATE 343. `state = 355 + (power - 1)`
pub const DAYLIGHT_POWER_BASE: u16 = 355;
pub const DAYLIGHT_POWER_END: u16 = 369;
/// light weighted plate power 1..=15 (370..=384); power 0 = 345
pub const LIGHT_PLATE_POWER_BASE: u16 = 370;
pub const LIGHT_PLATE_POWER_END: u16 = 384;
/// heavy weighted plate power 1..=15 (385..=399); power 0 = 346
pub const HEAVY_PLATE_POWER_BASE: u16 = 385;
pub const HEAVY_PLATE_POWER_END: u16 = 399;

/// daylight-sensor state for a signal power (0..=15)
#[inline]
pub fn daylight_sensor_state(p: u8) -> u16 {
    if p == 0 {
        DAYLIGHT_SENSOR_STATE
    } else {
        DAYLIGHT_POWER_BASE + (p.clamp(1, 15) - 1) as u16
    }
}

/// daylight-sensor signal power of a state (0 = idle/none)
#[inline]
pub fn daylight_sensor_power(s: u16) -> u16 {
    if (DAYLIGHT_POWER_BASE..=DAYLIGHT_POWER_END).contains(&s) {
        (s - DAYLIGHT_POWER_BASE + 1) as u16
    } else {
        0
    }
}

/// weighted-plate state for a signal power (0..=15)
#[inline]
pub fn plate_state(block: u16, p: u8) -> u16 {
    let base = if block == LIGHT_WEIGHTED_PLATE {
        LIGHT_PLATE_POWER_BASE
    } else {
        HEAVY_PLATE_POWER_BASE
    };
    if p == 0 {
        if block == LIGHT_WEIGHTED_PLATE {
            LIGHT_PLATE_STATE
        } else {
            HEAVY_PLATE_STATE
        }
    } else {
        base + (p.clamp(1, 15) - 1) as u16
    }
}

/// weighted-plate signal power of a state (0 = idle/none)
#[inline]
pub fn plate_power(s: u16) -> u16 {
    if (LIGHT_PLATE_POWER_BASE..=LIGHT_PLATE_POWER_END).contains(&s) {
        (s - LIGHT_PLATE_POWER_BASE + 1) as u16
    } else if (HEAVY_PLATE_POWER_BASE..=HEAVY_PLATE_POWER_END).contains(&s) {
        (s - HEAVY_PLATE_POWER_BASE + 1) as u16
    } else {
        0
    }
}
/// E3 item-block arithmetic (quartz/lead/saddle items: 204..=206 ↔
/// 348..=350)
#[inline]
pub fn e3_item_block_state(b: u16) -> Option<u16> {
    if (NETHER_QUARTZ..=SADDLE).contains(&b) {
        Some(NETHER_QUARTZ_STATE + (b - NETHER_QUARTZ) as u16)
    } else {
        None
    }
}

#[inline]
pub fn e3_item_state_block(s: u16) -> Option<u16> {
    if (NETHER_QUARTZ_STATE..=SADDLE_STATE).contains(&s) {
        Some(NETHER_QUARTZ + (s - NETHER_QUARTZ_STATE) as u16)
    } else {
        None
    }
}

/// E3 spawn-egg state arithmetic (ids 207..=209 ↔ 351..=353)
#[inline]
pub fn e3_egg_block_state(b: u16) -> Option<u16> {
    if (E3_SPAWN_EGG_BASE..=E3_SPAWN_EGG_END).contains(&b) {
        Some(E3_EGG_STATE_BASE + (b - E3_SPAWN_EGG_BASE) as u16)
    } else {
        None
    }
}

#[inline]
pub fn e3_egg_state_block(s: u16) -> Option<u16> {
    if (E3_EGG_STATE_BASE..=E3_EGG_STATE_END).contains(&s) {
        Some(E3_SPAWN_EGG_BASE + (s - E3_EGG_STATE_BASE) as u16)
    } else {
        None
    }
}

/// stained terracotta block id for a color index (0..15, the vanilla
/// dye-color registry order)
#[inline]
pub fn stained_terracotta(color: u8) -> u16 {
    STAINED_TERRACOTTA_BASE + color.min(15) as u16
}

/// state id for a stained-terracotta color
#[inline]
pub fn stained_terracotta_state(color: u8) -> u16 {
    TERRACOTTA_STAINED_STATE_BASE + color.min(15) as u16
}

/// color index of a stained-terracotta state (255 = not one)
#[inline]
pub fn stained_terracotta_color(s: u16) -> u16 {
    if (TERRACOTTA_STAINED_STATE_BASE..=TERRACOTTA_STAINED_STATE_END).contains(&s) {
        (s - TERRACOTTA_STAINED_STATE_BASE) as u16
    } else {
        255
    }
}

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
pub fn lava_level(s: u16) -> u16 {
    if s == LAVA_STATE {
        0
    } else if (LAVA_FLOW_BASE..=LAVA_FLOW_END).contains(&s) {
        (s - LAVA_FLOW_BASE + 1) as u16
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
pub fn e2_item_block_state(b: u16) -> Option<u16> {
    if (EMERALD..=PUMPKIN_PIE).contains(&b) {
        Some(E2_ITEM_STATE_BASE + (b - EMERALD) as u16)
    } else {
        None
    }
}

#[inline]
pub fn e2_item_state_block(s: u16) -> Option<u16> {
    if (E2_ITEM_STATE_BASE..=E2_ITEM_STATE_END).contains(&s) {
        Some(EMERALD + (s - E2_ITEM_STATE_BASE) as u16)
    } else {
        None
    }
}

/// state ↔ item-block arithmetic helpers (item ids 117..=139 ↔ 256..=278)
#[inline]
pub fn item_block_state(b: u16) -> Option<u16> {
    if (END_CRYSTAL..=SPAWN_EGG_MAX).contains(&b) {
        Some(ITEM_STATE_BASE + (b - END_CRYSTAL) as u16)
    } else {
        None
    }
}

#[inline]
pub fn item_state_block(s: u16) -> Option<u16> {
    if (ITEM_STATE_BASE..=ITEM_STATE_END).contains(&s) {
        Some(END_CRYSTAL + (s - ITEM_STATE_BASE) as u16)
    } else {
        None
    }
}

pub const BLOCK_COUNT: usize = 282;
/// [merge renumber] acacia/dark-oak log axis states moved to 443..=446
/// (past the E-series states, which end at 354; V2 base is now 400)
/// acacia/dark-oak log axis states (the V2 log window — same pattern as
/// the pre-1.7 oak/birch/spruce axis states 57..=62)
pub const ACACIA_LOG_X: u16 = 443;
pub const ACACIA_LOG_Z: u16 = 444;
pub const DARK_OAK_LOG_X: u16 = 445;
pub const DARK_OAK_LOG_Z: u16 = 446;




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
/// Phase E3: quartz family + stained terracotta (16) + carpets (5) +
/// hay/sensor/trapped-chest/plates/redstone-block + quartz/lead/saddle
/// items + eggs 20..=22 + the POWER-state ladders (317..=399)
/// [merge renumber] F-series states: V2 400..=442 + log-axis 443..=446,
/// V3 447..=465, V4 466..=475, V5 476..=479, V6 480..=485 (audit-fix)
pub const STATE_COUNT: usize = 486;
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
pub fn wire_power(s: u16) -> u16 {
    if is_wire_power(s) {
        (s - WIRE_POWER_BASE) as u16
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
pub fn default_state(b: u16) -> u16 {
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
        COAL => COAL_STATE,
        // ---- Phase E3 defaults (dedicated states ≥ 317) ----
        COAL_BLOCK => COAL_BLOCK_STATE,
        QUARTZ_BLOCK => QUARTZ_BLOCK_STATE,
        CHISELED_QUARTZ => CHISELED_QUARTZ_STATE,
        QUARTZ_PILLAR => QUARTZ_PILLAR_STATE,
        b if (STAINED_TERRACOTTA_BASE..=STAINED_TERRACOTTA_END).contains(&b) => {
            stained_terracotta_state((b - STAINED_TERRACOTTA_BASE) as u8)
        }
        b if (CARPET_WHITE..=CARPET_BLACK).contains(&b) => CARPET_STATE_BASE + (b - CARPET_WHITE) as u16,
        HAY_BALE => HAY_BALE_STATE,
        DAYLIGHT_SENSOR => DAYLIGHT_SENSOR_STATE,
        TRAPPED_CHEST => TRAPPED_CHEST_STATE,
        LIGHT_WEIGHTED_PLATE => LIGHT_PLATE_STATE,
        HEAVY_WEIGHTED_PLATE => HEAVY_PLATE_STATE,
        REDSTONE_BLOCK => REDSTONE_BLOCK_STATE,
        // ---- F-series defaults (V2..V5 windows, merge-renumbered 2026-09-06) ----
        b if (200..200 + V2_COUNT as u16).contains(&b) => {
            V2_STATE_BASE + (b - 200) as u16
        }
        b if (243..243 + V3_COUNT as u16).contains(&b) => {
            V3_STATE_BASE + (b - 243) as u16
        }
        b if (276..276 + V6_COUNT as u16).contains(&b) => {
            V6_STATE_BASE + (b - 276) as u16
        }
        b if (262..262 + V4_COUNT as u16).contains(&b) => {
            V4_STATE_BASE + (b - 262) as u16
        }
        b if (272..272 + V5_COUNT as u16).contains(&b) => {
            V5_STATE_BASE + (b - 272) as u16
        }
        OAK_SLAB => 63,     // PROP_BLOCKS[0].base_state (half=bottom)
        COBBLE_STAIRS => 65, // base_state (facing=north, half=bottom)
        OAK_FENCE => 73,    // base_state (no connections)
        // Phase E1 item-blocks: dedicated states ≥ 256 (never world-stored)
        // Phase E2 item-blocks: dedicated states 301..=306 (never world-stored)
        // Phase E3 item-blocks (quartz/lead/saddle) + eggs 20..=22
        _ => item_block_state(b)
            .or_else(|| e2_item_block_state(b))
            .or_else(|| e3_item_block_state(b))
            .or_else(|| e3_egg_block_state(b))
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
pub fn water_level(s: u16) -> u16 {
    if s == WATER as u16 {
        0
    } else if is_water_flow(s) {
        (s - WATER_FLOW_BASE + 1) as u16
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
    pub block: u16,
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
pub const OAK_SLAB: u16 = 57;
pub const COBBLE_STAIRS: u16 = 58;
pub const OAK_FENCE: u16 = 59;

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
pub fn prop_state_decode(s: u16) -> Option<(u16, Vec<(&'static str, &'static str)>)> {
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
pub fn prop_state_encode(block: u16, set: &[(&str, &str)]) -> Option<u16> {
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
pub fn state_block(s: u16) -> u16 {
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
        COAL_STATE => return COAL,
        // ---- Phase E3 state folding ----
        COAL_BLOCK_STATE => return COAL_BLOCK,
        QUARTZ_BLOCK_STATE => return QUARTZ_BLOCK,
        CHISELED_QUARTZ_STATE => return CHISELED_QUARTZ,
        QUARTZ_PILLAR_STATE => return QUARTZ_PILLAR,
        s if (TERRACOTTA_STAINED_STATE_BASE..=TERRACOTTA_STAINED_STATE_END).contains(&s) => {
            return stained_terracotta((s - TERRACOTTA_STAINED_STATE_BASE) as u8)
        }
        s if (CARPET_STATE_BASE..=CARPET_STATE_END).contains(&s) => {
            return CARPET_BASE + (s - CARPET_STATE_BASE) as u16
        }
        HAY_BALE_STATE => return HAY_BALE,
        DAYLIGHT_SENSOR_STATE => return DAYLIGHT_SENSOR,
        TRAPPED_CHEST_STATE => return TRAPPED_CHEST,
        LIGHT_PLATE_STATE => return LIGHT_WEIGHTED_PLATE,
        HEAVY_PLATE_STATE => return HEAVY_WEIGHTED_PLATE,
        REDSTONE_BLOCK_STATE => return REDSTONE_BLOCK,
        s if (NETHER_QUARTZ_STATE..=SADDLE_STATE).contains(&s) => {
            return e3_item_state_block(s).unwrap_or(AIR)
        }
        s if (E3_EGG_STATE_BASE..=E3_EGG_STATE_END).contains(&s) => {
            return e3_egg_state_block(s).unwrap_or(AIR)
        }
        // Phase E3 POWER-state folding (sensor/plates/trapped-chest
        // signal ladders — all fold to their parent block)
        TRAPPED_CHEST_OPEN_STATE => return TRAPPED_CHEST,
        s if (DAYLIGHT_POWER_BASE..=DAYLIGHT_POWER_END).contains(&s) => return DAYLIGHT_SENSOR,
        s if (LIGHT_PLATE_POWER_BASE..=LIGHT_PLATE_POWER_END).contains(&s) => {
            return LIGHT_WEIGHTED_PLATE
        }
        s if (HEAVY_PLATE_POWER_BASE..=HEAVY_PLATE_POWER_END).contains(&s) => {
            return HEAVY_WEIGHTED_PLATE
        }
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
        s if is_v2_state(s) => {
            return V2_STATE_TO_BLOCK[(s - V2_STATE_BASE) as usize];
        }
        s if is_v3_state(s) => {
            return V3_STATE_TO_BLOCK[(s - V3_STATE_BASE) as usize];
        }
        s if is_v4_state(s) => {
            return V4_STATE_TO_BLOCK[(s - V4_STATE_BASE) as usize];
        }
        s if is_v5_state(s) => {
            return V5_STATE_TO_BLOCK[(s - V5_STATE_BASE) as usize];
        }
        s if is_v6_state(s) => {
            return V6_STATE_TO_BLOCK[(s - V6_STATE_BASE) as usize];
        }
        ACACIA_LOG_X | ACACIA_LOG_Z => return ACACIA_LOG,
        DARK_OAK_LOG_X | DARK_OAK_LOG_Z => return DARK_OAK_LOG,
        _ => {}
    }
    if let Some((b, _)) = prop_state_decode(s) {
        return b;
    }
    match s {
        OAK_LOG_X | OAK_LOG_Z => OAK_LOG,
        BIRCH_LOG_X | BIRCH_LOG_Z => BIRCH_LOG,
        SPRUCE_LOG_X | SPRUCE_LOG_Z => SPRUCE_LOG,
        _ => s as u16, // identity for 0..=56
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
            OAK_LOG_X | BIRCH_LOG_X | SPRUCE_LOG_X | ACACIA_LOG_X | DARK_OAK_LOG_X => "[axis=x]",
            OAK_LOG_Z | BIRCH_LOG_Z | SPRUCE_LOG_Z | ACACIA_LOG_Z | DARK_OAK_LOG_Z => "[axis=z]",
            _ => "",
        };
        // fold log-variant states to their owning block for the name
        format!("{}{}", name(state_block(s)), axis)
    }
}

/// true if this state renders through the JSON-model path (mesher dispatch)
#[inline]
pub fn is_model_state(s: u16) -> bool {
    // 1.7.2 V2 window: never model states — greedy cubes / cross plants
    // per their BlockDef flags, exactly like the sim-state windows
    if is_v2_state(s)
        || is_v3_state(s)
        || is_v4_state(s)
        || is_v5_state(s)
        || is_v6_state(s)
        || s == ACACIA_LOG_X
        || s == ACACIA_LOG_Z
        || s == DARK_OAK_LOG_X
        || s == DARK_OAK_LOG_Z
    {
        return false;
    }
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
        // VERIFICATION-REPORT fix #4: the coal item state (inventory-only)
        && s != COAL_STATE
        // ---- Phase E3: every E3 state is a full-cube/cross/sim state —
        // none routes to the JSON-model dispatcher ----
        && !((COAL_BLOCK_STATE..=E3_EGG_STATE_END).contains(&s)
            || s == TRAPPED_CHEST_OPEN_STATE
            || (DAYLIGHT_POWER_BASE..=DAYLIGHT_POWER_END).contains(&s)
            || (LIGHT_PLATE_POWER_BASE..=LIGHT_PLATE_POWER_END).contains(&s)
            || (HEAVY_PLATE_POWER_BASE..=HEAVY_PLATE_POWER_END).contains(&s))
}

/// true if this block id has property-driven model states
#[inline]
pub fn is_model_block(b: u16) -> bool {
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
        ACACIA_LOG_X => [TILE_ACACIA_LOG_SIDE, TILE_ACACIA_LOG_SIDE, TILE_ACACIA_LOG_TOP, TILE_ACACIA_LOG_SIDE],
        ACACIA_LOG_Z => [TILE_ACACIA_LOG_SIDE, TILE_ACACIA_LOG_SIDE, TILE_ACACIA_LOG_SIDE, TILE_ACACIA_LOG_TOP],
        DARK_OAK_LOG_X => [TILE_DARK_OAK_LOG_SIDE, TILE_DARK_OAK_LOG_SIDE, TILE_DARK_OAK_LOG_TOP, TILE_DARK_OAK_LOG_SIDE],
        DARK_OAK_LOG_Z => [TILE_DARK_OAK_LOG_SIDE, TILE_DARK_OAK_LOG_SIDE, TILE_DARK_OAK_LOG_SIDE, TILE_DARK_OAK_LOG_TOP],
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
pub fn is_log(b: u16) -> bool {
    b == OAK_LOG || b == BIRCH_LOG || b == SPRUCE_LOG || b == ACACIA_LOG || b == DARK_OAK_LOG
}

/// state for placing a log with the given axis (0=X, 1=Y, 2=Z).
/// Vanilla placement rule: the log's axis follows the clicked face.
#[inline]
pub fn log_axis_state(block: u16, axis: u8) -> u16 {
    match (block, axis) {
        (OAK_LOG, 0) => OAK_LOG_X,
        (OAK_LOG, 2) => OAK_LOG_Z,
        (BIRCH_LOG, 0) => BIRCH_LOG_X,
        (BIRCH_LOG, 2) => BIRCH_LOG_Z,
        (SPRUCE_LOG, 0) => SPRUCE_LOG_X,
        (SPRUCE_LOG, 2) => SPRUCE_LOG_Z,
        (ACACIA_LOG, 0) => ACACIA_LOG_X,
        (ACACIA_LOG, 2) => ACACIA_LOG_Z,
        (DARK_OAK_LOG, 0) => DARK_OAK_LOG_X,
        (DARK_OAK_LOG, 2) => DARK_OAK_LOG_Z,
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
// [merge] E-series tiles end at 243; the F-series (1.7.2-1.10) tiles
// continue at 244..=325; the audit-fix round adds 326..=332
pub const TILE_MAX: u16 = 332;

/// inventory-only ITEM blocks (potions/bottles/books): never placeable in
/// the world — right-click drinks (potions) / fills (glass bottle at water).
#[inline]
pub fn is_item_block(b: u16) -> bool {
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
            | GOLDEN_CARROT
            // VERIFICATION-REPORT fix #4: the coal fuel item
            | COAL
            // ---- F-series item-blocks (1.7.2-1.10, merge-renumbered) ----
            | RAW_FISH | RAW_SALMON | CLOWNFISH | PUFFERFISH
            | RAW_RABBIT | COOKED_RABBIT | RABBIT_HIDE | RABBIT_FOOT
            | PRISMARINE_SHARD | PRISMARINE_CRYSTALS
            | CHORUS_FRUIT | ELYTRA | SHIELD
    ) || is_spawn_egg(b)
}

/// true for the mob spawn-egg item ids (124..=143 + the E3 window
/// 196..=198).
#[inline]
pub fn is_spawn_egg(b: u16) -> bool {
    (SPAWN_EGG_BASE..=SPAWN_EGG_MAX).contains(&b)
        || (E3_SPAWN_EGG_BASE..=E3_SPAWN_EGG_END).contains(&b)
}

/// The mob this spawn-egg id spawns. Tile order in the BLOCK_TABLE egg
/// rows MUST match this mapping (guarded by the egg roundtrip test).
/// The egg ids follow the Phase-2/Phase-E1 MobKind discriminant order
/// (see vc_gameplay::mobs::MobKind and the egg_art palette table).
#[inline]
pub fn egg_mob(b: u16) -> Option<u8> {
    if (E3_SPAWN_EGG_BASE..=E3_SPAWN_EGG_END).contains(&b) {
        // kinds 20..=22 (horse/donkey/mule — MobKind::from_egg)
        return Some(20 + (b - E3_SPAWN_EGG_BASE) as u8);
    }
    if !is_spawn_egg(b) {
        return None;
    }
    Some((b - SPAWN_EGG_BASE) as u8) // 0..=19, decoded by the gameplay layer
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
    emissive(state_block(s))
}

/// Phase E1: nether-wart crop age (0..3) from its storage state.
#[inline]
pub fn wart_age(s: u16) -> u16 {
    if (WART_STATE_BASE..=WART_STATE_END).contains(&s) {
        (s - WART_STATE_BASE) as u16
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
    // coal item (VERIFICATION-REPORT fix #4): inventory-only fuel item,
    // 1600 ticks / 8 items per piece (VERIFIED live w/Furnace + w/Smelting)
    d("Coal", [TILE_COAL, TILE_COAL, TILE_COAL], false, false, true, false, 0, SoundFamily::Stone),
    // ---- Phase E3 (1.5–1.6 bracket) ----
    // block of coal: 16000 ticks / 80 items (VERIFIED w/Block_of_Coal)
    d("Block of Coal", [TILE_COAL_BLOCK, TILE_COAL_BLOCK, TILE_COAL_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Block of Quartz", [TILE_QUARTZ_BLOCK, TILE_QUARTZ_BLOCK, TILE_QUARTZ_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Chiseled Quartz Block", [TILE_CHISELED_QUARTZ, TILE_CHISELED_QUARTZ, TILE_CHISELED_QUARTZ], true, true, false, false, 0, SoundFamily::Stone),
    d("Quartz Pillar", [TILE_QUARTZ_PILLAR_TOP, TILE_QUARTZ_PILLAR_TOP, TILE_QUARTZ_PILLAR_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    // 16 stained terracotta (vanilla dye-color order; Badlands banding)
    d("White Terracotta", [TILE_TERRACOTTA_STAINED_BASE, TILE_TERRACOTTA_STAINED_BASE, TILE_TERRACOTTA_STAINED_BASE], true, true, false, false, 0, SoundFamily::Stone),
    d("Orange Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 1, TILE_TERRACOTTA_STAINED_BASE + 1, TILE_TERRACOTTA_STAINED_BASE + 1], true, true, false, false, 0, SoundFamily::Stone),
    d("Magenta Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 2, TILE_TERRACOTTA_STAINED_BASE + 2, TILE_TERRACOTTA_STAINED_BASE + 2], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Blue Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 3, TILE_TERRACOTTA_STAINED_BASE + 3, TILE_TERRACOTTA_STAINED_BASE + 3], true, true, false, false, 0, SoundFamily::Stone),
    d("Yellow Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 4, TILE_TERRACOTTA_STAINED_BASE + 4, TILE_TERRACOTTA_STAINED_BASE + 4], true, true, false, false, 0, SoundFamily::Stone),
    d("Lime Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 5, TILE_TERRACOTTA_STAINED_BASE + 5, TILE_TERRACOTTA_STAINED_BASE + 5], true, true, false, false, 0, SoundFamily::Stone),
    d("Pink Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 6, TILE_TERRACOTTA_STAINED_BASE + 6, TILE_TERRACOTTA_STAINED_BASE + 6], true, true, false, false, 0, SoundFamily::Stone),
    d("Gray Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 7, TILE_TERRACOTTA_STAINED_BASE + 7, TILE_TERRACOTTA_STAINED_BASE + 7], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Gray Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 8, TILE_TERRACOTTA_STAINED_BASE + 8, TILE_TERRACOTTA_STAINED_BASE + 8], true, true, false, false, 0, SoundFamily::Stone),
    d("Cyan Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 9, TILE_TERRACOTTA_STAINED_BASE + 9, TILE_TERRACOTTA_STAINED_BASE + 9], true, true, false, false, 0, SoundFamily::Stone),
    d("Purple Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 10, TILE_TERRACOTTA_STAINED_BASE + 10, TILE_TERRACOTTA_STAINED_BASE + 10], true, true, false, false, 0, SoundFamily::Stone),
    d("Blue Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 11, TILE_TERRACOTTA_STAINED_BASE + 11, TILE_TERRACOTTA_STAINED_BASE + 11], true, true, false, false, 0, SoundFamily::Stone),
    d("Brown Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 12, TILE_TERRACOTTA_STAINED_BASE + 12, TILE_TERRACOTTA_STAINED_BASE + 12], true, true, false, false, 0, SoundFamily::Stone),
    d("Green Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 13, TILE_TERRACOTTA_STAINED_BASE + 13, TILE_TERRACOTTA_STAINED_BASE + 13], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 14, TILE_TERRACOTTA_STAINED_BASE + 14, TILE_TERRACOTTA_STAINED_BASE + 14], true, true, false, false, 0, SoundFamily::Stone),
    d("Black Terracotta", [TILE_TERRACOTTA_STAINED_BASE + 15, TILE_TERRACOTTA_STAINED_BASE + 15, TILE_TERRACOTTA_STAINED_BASE + 15], true, true, false, false, 0, SoundFamily::Stone),
    // carpets: 1/16-block visual (VERIFIED w/Carpet 14w29a) — non-solid
    // floor overlay, the engine's full-cube adaptation (disclosed)
    d("White Carpet", [TILE_WOOL_WHITE, TILE_WOOL_WHITE, TILE_WOOL_WHITE], false, false, false, false, 0, SoundFamily::Wool),
    d("Red Carpet", [TILE_WOOL_RED, TILE_WOOL_RED, TILE_WOOL_RED], false, false, false, false, 0, SoundFamily::Wool),
    d("Yellow Carpet", [TILE_WOOL_YELLOW, TILE_WOOL_YELLOW, TILE_WOOL_YELLOW], false, false, false, false, 0, SoundFamily::Wool),
    d("Blue Carpet", [TILE_WOOL_BLUE, TILE_WOOL_BLUE, TILE_WOOL_BLUE], false, false, false, false, 0, SoundFamily::Wool),
    d("Black Carpet", [TILE_WOOL_BLACK, TILE_WOOL_BLACK, TILE_WOOL_BLACK], false, false, false, false, 0, SoundFamily::Wool),
    d("Hay Bale", [TILE_HAY_TOP, TILE_HAY_TOP, TILE_HAY_SIDE], true, true, false, false, 0, SoundFamily::Grass),
    d("Daylight Sensor", [TILE_DAYLIGHT_TOP, TILE_DAYLIGHT_TOP, TILE_DAYLIGHT_SIDE], false, false, false, false, 0, SoundFamily::Wood),
    d("Trapped Chest", [TILE_CHEST, TILE_CHEST, TILE_CHEST], true, true, false, false, 0, SoundFamily::Wood),
    d("Light Weighted Pressure Plate", [TILE_PLATE_LIGHT, TILE_PLATE_LIGHT, TILE_PLATE_LIGHT], false, false, false, false, 0, SoundFamily::Stone),
    d("Heavy Weighted Pressure Plate", [TILE_PLATE_HEAVY, TILE_PLATE_HEAVY, TILE_PLATE_HEAVY], false, false, false, false, 0, SoundFamily::Stone),
    d("Block of Redstone", [TILE_REDSTONE_BLOCK, TILE_REDSTONE_BLOCK, TILE_REDSTONE_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    // items: nether quartz / lead / saddle (inventory-only)
    d("Nether Quartz", [TILE_NETHER_QUARTZ, TILE_NETHER_QUARTZ, TILE_NETHER_QUARTZ], false, false, true, false, 0, SoundFamily::Stone),
    d("Lead", [TILE_LEAD, TILE_LEAD, TILE_LEAD], false, false, true, false, 0, SoundFamily::Grass),
    d("Saddle", [TILE_SADDLE, TILE_SADDLE, TILE_SADDLE], false, false, true, false, 0, SoundFamily::Grass),
    // E3 spawn eggs (kinds 20..=22)
    d("Horse Spawn Egg", [TILE_E3_EGG_HORSE, TILE_E3_EGG_HORSE, TILE_E3_EGG_HORSE], false, false, true, false, 0, SoundFamily::Grass),
    d("Donkey Spawn Egg", [TILE_E3_EGG_DONKEY, TILE_E3_EGG_DONKEY, TILE_E3_EGG_DONKEY], false, false, true, false, 0, SoundFamily::Grass),
    d("Mule Spawn Egg", [TILE_E3_EGG_MULE, TILE_E3_EGG_MULE, TILE_E3_EGG_MULE], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.7.2 bracket (V2 window) — minecraft.wiki/w/Java_Edition_1.7.2,
    // live round 2026-09-06. [merge] rows moved to ids 200.. (past the
    // E-series); the 16 stained-clay rows DROPPED (E3 covers them). ----
    // stained glass: solid, NOT opaque (translucent), glass sounds
    d("White Stained Glass", [TILE_STAINED_GLASS_WHITE, TILE_STAINED_GLASS_WHITE, TILE_STAINED_GLASS_WHITE], true, false, false, false, 0, SoundFamily::Glass),
    d("Orange Stained Glass", [TILE_STAINED_GLASS_ORANGE, TILE_STAINED_GLASS_ORANGE, TILE_STAINED_GLASS_ORANGE], true, false, false, false, 0, SoundFamily::Glass),
    d("Magenta Stained Glass", [TILE_STAINED_GLASS_MAGENTA, TILE_STAINED_GLASS_MAGENTA, TILE_STAINED_GLASS_MAGENTA], true, false, false, false, 0, SoundFamily::Glass),
    d("Light Blue Stained Glass", [TILE_STAINED_GLASS_LIGHT_BLUE, TILE_STAINED_GLASS_LIGHT_BLUE, TILE_STAINED_GLASS_LIGHT_BLUE], true, false, false, false, 0, SoundFamily::Glass),
    d("Yellow Stained Glass", [TILE_STAINED_GLASS_YELLOW, TILE_STAINED_GLASS_YELLOW, TILE_STAINED_GLASS_YELLOW], true, false, false, false, 0, SoundFamily::Glass),
    d("Lime Stained Glass", [TILE_STAINED_GLASS_LIME, TILE_STAINED_GLASS_LIME, TILE_STAINED_GLASS_LIME], true, false, false, false, 0, SoundFamily::Glass),
    d("Pink Stained Glass", [TILE_STAINED_GLASS_PINK, TILE_STAINED_GLASS_PINK, TILE_STAINED_GLASS_PINK], true, false, false, false, 0, SoundFamily::Glass),
    d("Gray Stained Glass", [TILE_STAINED_GLASS_GRAY, TILE_STAINED_GLASS_GRAY, TILE_STAINED_GLASS_GRAY], true, false, false, false, 0, SoundFamily::Glass),
    d("Light Gray Stained Glass", [TILE_STAINED_GLASS_LIGHT_GRAY, TILE_STAINED_GLASS_LIGHT_GRAY, TILE_STAINED_GLASS_LIGHT_GRAY], true, false, false, false, 0, SoundFamily::Glass),
    d("Cyan Stained Glass", [TILE_STAINED_GLASS_CYAN, TILE_STAINED_GLASS_CYAN, TILE_STAINED_GLASS_CYAN], true, false, false, false, 0, SoundFamily::Glass),
    d("Purple Stained Glass", [TILE_STAINED_GLASS_PURPLE, TILE_STAINED_GLASS_PURPLE, TILE_STAINED_GLASS_PURPLE], true, false, false, false, 0, SoundFamily::Glass),
    d("Blue Stained Glass", [TILE_STAINED_GLASS_BLUE, TILE_STAINED_GLASS_BLUE, TILE_STAINED_GLASS_BLUE], true, false, false, false, 0, SoundFamily::Glass),
    d("Brown Stained Glass", [TILE_STAINED_GLASS_BROWN, TILE_STAINED_GLASS_BROWN, TILE_STAINED_GLASS_BROWN], true, false, false, false, 0, SoundFamily::Glass),
    d("Green Stained Glass", [TILE_STAINED_GLASS_GREEN, TILE_STAINED_GLASS_GREEN, TILE_STAINED_GLASS_GREEN], true, false, false, false, 0, SoundFamily::Glass),
    d("Red Stained Glass", [TILE_STAINED_GLASS_RED, TILE_STAINED_GLASS_RED, TILE_STAINED_GLASS_RED], true, false, false, false, 0, SoundFamily::Glass),
    d("Black Stained Glass", [TILE_STAINED_GLASS_BLACK, TILE_STAINED_GLASS_BLACK, TILE_STAINED_GLASS_BLACK], true, false, false, false, 0, SoundFamily::Glass),
    // stained terracotta ("stained clay"): full opaque cubes, stone sounds
    // red sand: mesa floor — falls like sand, smelts to glass
    d("Red Sand", [TILE_RED_SAND, TILE_RED_SAND, TILE_RED_SAND], true, true, false, false, 0, SoundFamily::Sand),
    // packed ice: OPAQUE (1.7.2 changelog), does not melt
    d("Packed Ice", [TILE_PACKED_ICE, TILE_PACKED_ICE, TILE_PACKED_ICE], true, true, false, false, 0, SoundFamily::Glass),
    // podzol: mega-taiga dirt variant (top + side + dirt bottom)
    d("Podzol", [TILE_PODZOL_TOP, TILE_DIRT, TILE_PODZOL_SIDE], true, true, false, false, 0, SoundFamily::Dirt),
    // acacia / dark oak
    d("Acacia Log", [TILE_ACACIA_LOG_TOP, TILE_ACACIA_LOG_TOP, TILE_ACACIA_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Acacia Leaves", [TILE_LEAVES, TILE_LEAVES, TILE_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    d("Dark Oak Log", [TILE_DARK_OAK_LOG_TOP, TILE_DARK_OAK_LOG_TOP, TILE_DARK_OAK_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Dark Oak Leaves", [TILE_LEAVES, TILE_LEAVES, TILE_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    // 8 new small flowers (cross plants)
    d("Allium", [TILE_ALLIUM, TILE_ALLIUM, TILE_ALLIUM], false, false, true, false, 0, SoundFamily::Grass),
    d("Azure Bluet", [TILE_AZURE_BLUET, TILE_AZURE_BLUET, TILE_AZURE_BLUET], false, false, true, false, 0, SoundFamily::Grass),
    d("Blue Orchid", [TILE_BLUE_ORCHID, TILE_BLUE_ORCHID, TILE_BLUE_ORCHID], false, false, true, false, 0, SoundFamily::Grass),
    d("Oxeye Daisy", [TILE_OXEYE_DAISY, TILE_OXEYE_DAISY, TILE_OXEYE_DAISY], false, false, true, false, 0, SoundFamily::Grass),
    d("Orange Tulip", [TILE_ORANGE_TULIP, TILE_ORANGE_TULIP, TILE_ORANGE_TULIP], false, false, true, false, 0, SoundFamily::Grass),
    d("Red Tulip", [TILE_RED_TULIP, TILE_RED_TULIP, TILE_RED_TULIP], false, false, true, false, 0, SoundFamily::Grass),
    d("White Tulip", [TILE_WHITE_TULIP, TILE_WHITE_TULIP, TILE_WHITE_TULIP], false, false, true, false, 0, SoundFamily::Grass),
    d("Pink Tulip", [TILE_PINK_TULIP, TILE_PINK_TULIP, TILE_PINK_TULIP], false, false, true, false, 0, SoundFamily::Grass),
    // 2-block-tall flowers, lower + upper halves
    d("Sunflower", [TILE_SUNFLOWER_LOWER, TILE_SUNFLOWER_LOWER, TILE_SUNFLOWER_LOWER], false, false, true, false, 0, SoundFamily::Grass),
    d("Sunflower", [TILE_SUNFLOWER_TOP, TILE_SUNFLOWER_TOP, TILE_SUNFLOWER_TOP], false, false, true, false, 0, SoundFamily::Grass),
    d("Lilac", [TILE_LILAC_LOWER, TILE_LILAC_LOWER, TILE_LILAC_LOWER], false, false, true, false, 0, SoundFamily::Grass),
    d("Lilac", [TILE_LILAC_TOP, TILE_LILAC_TOP, TILE_LILAC_TOP], false, false, true, false, 0, SoundFamily::Grass),
    d("Peony", [TILE_PEONY_LOWER, TILE_PEONY_LOWER, TILE_PEONY_LOWER], false, false, true, false, 0, SoundFamily::Grass),
    d("Peony", [TILE_PEONY_TOP, TILE_PEONY_TOP, TILE_PEONY_TOP], false, false, true, false, 0, SoundFamily::Grass),
    d("Rose Bush", [TILE_ROSE_BUSH_LOWER, TILE_ROSE_BUSH_LOWER, TILE_ROSE_BUSH_LOWER], false, false, true, false, 0, SoundFamily::Grass),
    d("Rose Bush", [TILE_ROSE_BUSH_TOP, TILE_ROSE_BUSH_TOP, TILE_ROSE_BUSH_TOP], false, false, true, false, 0, SoundFamily::Grass),
    // 1.7.2 fish items — inventory-only, cross-rendered icons
    d("Raw Fish", [TILE_RAW_FISH, TILE_RAW_FISH, TILE_RAW_FISH], false, false, true, false, 0, SoundFamily::Grass),
    d("Raw Salmon", [TILE_RAW_SALMON, TILE_RAW_SALMON, TILE_RAW_SALMON], false, false, true, false, 0, SoundFamily::Grass),
    d("Clownfish", [TILE_CLOWNFISH, TILE_CLOWNFISH, TILE_CLOWNFISH], false, false, true, false, 0, SoundFamily::Grass),
    d("Pufferfish", [TILE_PUFFERFISH, TILE_PUFFERFISH, TILE_PUFFERFISH], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.8 bracket (V3 window) — minecraft.wiki/w/Java_Edition_1.8,
    // live round 2026-09-06 ----
    // slime block: solid, translucent, bounces (the trampoline block)
    d("Slime Block", [TILE_SLIME, TILE_SLIME, TILE_SLIME], true, false, false, false, 0, SoundFamily::Grass),
    d("Coarse Dirt", [TILE_COARSE_DIRT, TILE_COARSE_DIRT, TILE_COARSE_DIRT], true, true, false, false, 0, SoundFamily::Dirt),
    d("Polished Granite", [TILE_POLISHED_GRANITE, TILE_POLISHED_GRANITE, TILE_POLISHED_GRANITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Polished Diorite", [TILE_POLISHED_DIORITE, TILE_POLISHED_DIORITE, TILE_POLISHED_DIORITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Polished Andesite", [TILE_POLISHED_ANDESITE, TILE_POLISHED_ANDESITE, TILE_POLISHED_ANDESITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Sandstone", [TILE_RED_SANDSTONE, TILE_RED_SANDSTONE, TILE_RED_SANDSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Smooth Red Sandstone", [TILE_SMOOTH_RED_SANDSTONE, TILE_SMOOTH_RED_SANDSTONE, TILE_SMOOTH_RED_SANDSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Prismarine", [TILE_PRISMARINE, TILE_PRISMARINE, TILE_PRISMARINE], true, true, false, false, 0, SoundFamily::Stone),
    d("Prismarine Bricks", [TILE_PRISMARINE_BRICKS, TILE_PRISMARINE_BRICKS, TILE_PRISMARINE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Dark Prismarine", [TILE_DARK_PRISMARINE, TILE_DARK_PRISMARINE, TILE_DARK_PRISMARINE], true, true, false, false, 0, SoundFamily::Stone),
    // wiki: "Emit light at a light level of 15"
    d("Sea Lantern", [TILE_SEA_LANTERN, TILE_SEA_LANTERN, TILE_SEA_LANTERN], true, true, false, false, 15, SoundFamily::Glass),
    d("Iron Trapdoor", [TILE_IRON_TRAPDOOR, TILE_IRON_TRAPDOOR, TILE_IRON_TRAPDOOR], true, false, false, false, 0, SoundFamily::Wood),
    // barrier: wiki "acts like bedrock, but is completely transparent" —
    // near-invisible tile + solid collision
    d("Barrier", [TILE_BARRIER, TILE_BARRIER, TILE_BARRIER], true, false, false, false, 0, SoundFamily::Glass),
    // 1.8 rabbit + prismarine items
    d("Raw Rabbit", [TILE_RAW_RABBIT, TILE_RAW_RABBIT, TILE_RAW_RABBIT], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Rabbit", [TILE_COOKED_RABBIT, TILE_COOKED_RABBIT, TILE_COOKED_RABBIT], false, false, true, false, 0, SoundFamily::Grass),
    d("Rabbit Hide", [TILE_RABBIT_HIDE, TILE_RABBIT_HIDE, TILE_RABBIT_HIDE], false, false, true, false, 0, SoundFamily::Grass),
    d("Rabbit's Foot", [TILE_RABBIT_FOOT, TILE_RABBIT_FOOT, TILE_RABBIT_FOOT], false, false, true, false, 0, SoundFamily::Grass),
    d("Prismarine Shard", [TILE_PRISMARINE_SHARD, TILE_PRISMARINE_SHARD, TILE_PRISMARINE_SHARD], false, false, true, false, 0, SoundFamily::Stone),
    d("Prismarine Crystals", [TILE_PRISMARINE_CRYSTALS, TILE_PRISMARINE_CRYSTALS, TILE_PRISMARINE_CRYSTALS], false, false, true, false, 0, SoundFamily::Stone),
    // ---- 1.9 bracket (V4 window) — minecraft.wiki/w/Java_Edition_1.9,
    // live round 2026-09-06 ----
    d("Grass Path", [TILE_GRASS_PATH, TILE_DIRT, TILE_GRASS_PATH_SIDE], true, true, false, false, 0, SoundFamily::Grass),
    d("Purpur Block", [TILE_PURPUR, TILE_PURPUR, TILE_PURPUR], true, true, false, false, 0, SoundFamily::Stone),
    d("Purpur Pillar", [TILE_PURPUR, TILE_PURPUR, TILE_PURPUR_PILLAR_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("End Stone Bricks", [TILE_END_STONE_BRICKS, TILE_END_STONE_BRICKS, TILE_END_STONE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    // wiki: "same brightness as torches" (14)
    d("End Rod", [TILE_END_ROD, TILE_END_ROD, TILE_END_ROD], false, false, true, false, 14, SoundFamily::Wood),
    d("Chorus Plant", [TILE_CHORUS_PLANT, TILE_CHORUS_PLANT, TILE_CHORUS_PLANT], true, false, true, false, 0, SoundFamily::Wood),
    d("Chorus Flower", [TILE_CHORUS_FLOWER, TILE_CHORUS_FLOWER, TILE_CHORUS_FLOWER], true, false, true, false, 0, SoundFamily::Wood),
    // 1.9 items
    d("Chorus Fruit", [TILE_CHORUS_FRUIT, TILE_CHORUS_FRUIT, TILE_CHORUS_FRUIT], false, false, true, false, 0, SoundFamily::Grass),
    d("Elytra", [TILE_ELYTRA, TILE_ELYTRA, TILE_ELYTRA], false, false, true, false, 0, SoundFamily::Grass),
    d("Shield", [TILE_SHIELD, TILE_SHIELD, TILE_SHIELD], false, false, true, false, 0, SoundFamily::Wood),
    // ---- 1.10 bracket (V5 window) — minecraft.wiki/w/Java_Edition_1.10,
    // live round 2026-09-06 ----
    // magma: light level 3 (wiki /w/Magma_Block, live round)
    d("Magma Block", [TILE_MAGMA, TILE_MAGMA, TILE_MAGMA], true, true, false, false, 3, SoundFamily::Stone),
    d("Nether Wart Block", [TILE_NETHER_WART_BLOCK, TILE_NETHER_WART_BLOCK, TILE_NETHER_WART_BLOCK], true, true, false, false, 0, SoundFamily::Wool),
    d("Red Nether Bricks", [TILE_RED_NETHER_BRICKS, TILE_RED_NETHER_BRICKS, TILE_RED_NETHER_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Bone Block", [TILE_BONE_BLOCK, TILE_BONE_BLOCK, TILE_BONE_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    // ---- audit-fix round (1.2 jungle family + 1.4 golden carrot) ----
    d("Golden Carrot", [TILE_GOLDEN_CARROT, TILE_GOLDEN_CARROT, TILE_GOLDEN_CARROT], false, false, true, false, 0, SoundFamily::Grass),
    d("Jungle Log", [TILE_JUNGLE_LOG_TOP, TILE_JUNGLE_LOG_TOP, TILE_JUNGLE_LOG_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Jungle Leaves", [TILE_JUNGLE_LEAVES, TILE_JUNGLE_LEAVES, TILE_JUNGLE_LEAVES], true, false, false, false, 0, SoundFamily::Leaves),
    d("Jungle Planks", [TILE_JUNGLE_PLANKS, TILE_JUNGLE_PLANKS, TILE_JUNGLE_PLANKS], true, true, false, false, 0, SoundFamily::Wood),
    d("Vine", [TILE_VINE, TILE_VINE, TILE_VINE], false, false, true, false, 0, SoundFamily::Grass),
    d("Fern", [TILE_FERN, TILE_FERN, TILE_FERN], false, false, true, false, 0, SoundFamily::Grass),
];

#[inline]
pub fn def(id: u16) -> &'static BlockDef {
    // §46 resilience: a raw STATE id that slipped past a fold indexes the
    // table out of bounds — clamp to the last def instead of panicking
    // (the honest fix is folding via state_block; this is the crash guard)
    &BLOCK_TABLE[(id as usize).min(BLOCK_COUNT - 1)]
}

#[inline]
pub fn is_solid(id: u16) -> bool {
    def(id).solid
}

#[inline]
pub fn is_opaque(id: u16) -> bool {
    def(id).opaque
}

#[inline]
pub fn is_cross(id: u16) -> bool {
    def(id).cross
}

#[inline]
pub fn is_fluid(id: u16) -> bool {
    def(id).fluid
}

#[inline]
pub fn emissive(id: u16) -> u8 {
    def(id).emissive
}

#[inline]
pub fn name(id: u16) -> &'static str {
    def(id).name
}

/// Should a face of `b` facing neighbor `n` be rendered?
#[inline]
pub fn face_visible(b: u16, n: u16) -> bool {
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
pub const PICKER_BLOCKS: [u16; 242] = [
    GRASS, DIRT, STONE, COBBLE, SMOOTH_STONE, STONE_BRICKS, BRICKS, MOSSY_COBBLE,
    GRANITE, DIORITE, ANDESITE, OBSIDIAN,
    SAND, GRAVEL, CLAY, TERRACOTTA,
    RED_SAND, PACKED_ICE, PODZOL,
    OAK_LOG, LEAVES, PLANKS, BIRCH_LOG, BIRCH_LEAVES, SPRUCE_LOG, SPRUCE_LEAVES,
    ACACIA_LOG, ACACIA_LEAVES, DARK_OAK_LOG, DARK_OAK_LEAVES,
    COAL_ORE, IRON_ORE, GOLD_ORE, REDSTONE_ORE, LAPIS_ORE, EMERALD_ORE, DIAMOND_ORE,
    IRON_BLOCK, GOLD_BLOCK, DIAMOND_BLOCK, GLOWSTONE,
    BOOKSHELF, CRAFTING_TABLE, FURNACE, GLASS, ICE, SNOW,
    PUMPKIN, MELON, CACTUS,
    WOOL_WHITE, WOOL_RED, WOOL_YELLOW, WOOL_BLUE, WOOL_BLACK,
    
    STAINED_GLASS_WHITE, STAINED_GLASS_ORANGE, STAINED_GLASS_MAGENTA,
    STAINED_GLASS_LIGHT_BLUE, STAINED_GLASS_YELLOW, STAINED_GLASS_LIME,
    STAINED_GLASS_PINK, STAINED_GLASS_GRAY, STAINED_GLASS_LIGHT_GRAY,
    STAINED_GLASS_CYAN, STAINED_GLASS_PURPLE, STAINED_GLASS_BLUE,
    STAINED_GLASS_BROWN, STAINED_GLASS_GREEN, STAINED_GLASS_RED,
    STAINED_GLASS_BLACK,

    TALL_GRASS, FLOWER_RED, FLOWER_YELLOW, MUSHROOM_RED, MUSHROOM_BROWN,
    
    ALLIUM, AZURE_BLUET, BLUE_ORCHID, OXEYE_DAISY,
    ORANGE_TULIP, RED_TULIP, WHITE_TULIP, PINK_TULIP,
    SUNFLOWER, LILAC, PEONY, ROSE_BUSH,
    
    RAW_FISH, RAW_SALMON, CLOWNFISH, PUFFERFISH,
    
    MAGMA_BLOCK, NETHER_WART_BLOCK, RED_NETHER_BRICKS, BONE_BLOCK,
    
    GRASS_PATH, PURPUR_BLOCK, PURPUR_PILLAR, END_STONE_BRICKS, END_ROD,
    CHORUS_PLANT, CHORUS_FLOWER, CHORUS_FRUIT, ELYTRA, SHIELD,
    
    SLIME_BLOCK, COARSE_DIRT,
    POLISHED_GRANITE, POLISHED_DIORITE, POLISHED_ANDESITE,
    RED_SANDSTONE, SMOOTH_RED_SANDSTONE,
    PRISMARINE, PRISMARINE_BRICKS, DARK_PRISMARINE, SEA_LANTERN,
    IRON_TRAPDOOR, BARRIER,
    RAW_RABBIT, COOKED_RABBIT, RABBIT_HIDE, RABBIT_FOOT,
    PRISMARINE_SHARD, PRISMARINE_CRYSTALS,
    OAK_SLAB, COBBLE_STAIRS, OAK_FENCE,
    NETHERRACK, NETHER_QUARTZ_ORE, SOUL_SAND,
    BREWING_STAND,
    POTION_EMPTY, POTION_WATER, POTION_AWKWARD, POTION_MUNDANE, POTION_HEALING, POTION_HEALING_II,
    ENCHANT_TABLE, ENCHANTED_BOOK,
    
    MYCELIUM, END_STONE, NETHER_BRICKS, NETHER_BRICK,
    REDSTONE_LAMP,
    CHISELED_STONE_BRICKS, CHISELED_SANDSTONE, CUT_SANDSTONE, SMOOTH_SANDSTONE,
    MUSHROOM_RED_BLOCK, MUSHROOM_BROWN_BLOCK, MUSHROOM_STEM,
    NETHER_WART, DRAGON_EGG, END_CRYSTAL,
    EYE_OF_ENDER, BLAZE_ROD, BLAZE_POWDER, GOLDEN_APPLE, SNOWBALL,
    
    SPAWN_EGG_BASE, SPAWN_EGG_BASE + 1, SPAWN_EGG_BASE + 2, SPAWN_EGG_BASE + 3,
    SPAWN_EGG_BASE + 4, SPAWN_EGG_BASE + 5, SPAWN_EGG_BASE + 6, SPAWN_EGG_BASE + 7,
    SPAWN_EGG_BASE + 8, SPAWN_EGG_BASE + 9, SPAWN_EGG_BASE + 10, SPAWN_EGG_BASE + 11,
    SPAWN_EGG_BASE + 12, SPAWN_EGG_BASE + 13, SPAWN_EGG_BASE + 14, SPAWN_EGG_BASE + 15,
    
    ANVIL, CHIPPED_ANVIL, DAMAGED_ANVIL, BEACON, COBBLE_WALL,
    ENDER_CHEST, FLOWER_POT, ITEM_FRAME, TRIPWIRE_HOOK,
    WITHER_SKELETON_SKULL, COMMAND_BLOCK,
    EMERALD, NETHER_STAR, POTATO, BAKED_POTATO, CARROT, PUMPKIN_PIE,
    GOLDEN_CARROT, JUNGLE_LOG, JUNGLE_LEAVES, JUNGLE_PLANKS, VINE, FERN,
    LAVA,
    
    
    COAL,
    
    SPAWN_EGG_BASE + 16, SPAWN_EGG_BASE + 17, SPAWN_EGG_BASE + 18, SPAWN_EGG_BASE + 19,
    
    COAL_BLOCK,
    QUARTZ_BLOCK, CHISELED_QUARTZ, QUARTZ_PILLAR, NETHER_QUARTZ,
    STAINED_TERRACOTTA_BASE, STAINED_TERRACOTTA_BASE + 1, STAINED_TERRACOTTA_BASE + 2,
    STAINED_TERRACOTTA_BASE + 3, STAINED_TERRACOTTA_BASE + 4, STAINED_TERRACOTTA_BASE + 5,
    STAINED_TERRACOTTA_BASE + 6, STAINED_TERRACOTTA_BASE + 7, STAINED_TERRACOTTA_BASE + 8,
    STAINED_TERRACOTTA_BASE + 9, STAINED_TERRACOTTA_BASE + 10, STAINED_TERRACOTTA_BASE + 11,
    STAINED_TERRACOTTA_BASE + 12, STAINED_TERRACOTTA_BASE + 13, STAINED_TERRACOTTA_BASE + 14,
    STAINED_TERRACOTTA_BASE + 15,
    CARPET_WHITE, CARPET_RED, CARPET_YELLOW, CARPET_BLUE, CARPET_BLACK,
    HAY_BALE, DAYLIGHT_SENSOR, TRAPPED_CHEST,
    LIGHT_WEIGHTED_PLATE, HEAVY_WEIGHTED_PLATE, REDSTONE_BLOCK,
    LEAD, SADDLE,
    
    E3_SPAWN_EGG_BASE, E3_SPAWN_EGG_BASE + 1, E3_SPAWN_EGG_BASE + 2,
];

/// default hotbar palette
pub const PALETTE: [u16; 9] = [GRASS, DIRT, STONE, COBBLE, PLANKS, OAK_LOG, LEAVES, GLOWSTONE, GLASS];

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn identity_states_fold_to_their_blocks() {
        // flat-registry states 0..=56 are identity-mapped; 57..=62 are the
        // log axis variants (state ids ≠ block ids there); 63+ are property
        // states (covered by prop_states_roundtrip)
        for b in 0..57u16 {
            assert_eq!(state_block(b), b as u16, "state {b}");
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
            assert!(b < BLOCK_COUNT as u16, "state {s} maps to bad block {b}");
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
        for b in 0..BLOCK_COUNT as u16 {
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
            assert_eq!(wart_age(WART_STATE_BASE + a), a as u16);
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
            let b = SPAWN_EGG_BASE + i as u16;
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
        // [merge scroll] the grid is a fixed 11-row window (514px) that
        // wheel-scrolls (vanilla creative-grid behavior) since the
        // F-series (1.7.2–1.10) grew PICKER_BLOCKS past one page. The
        // invariant: the VISIBLE window always fits the 960×540 canvas
        // and the scroll range reaches every entry.
        let cols = 15;
        let vis_rows = 11;
        assert!(vis_rows * 44 + 30 <= 540, "visible picker window too tall");
        assert!(cols * 44 + 8 <= 960, "picker grid too wide");
        let total_rows = (PICKER_BLOCKS.len() + cols - 1) / cols;
        let max_scroll = total_rows.saturating_sub(vis_rows);
        // every entry is reachable: max first-row × cols < len, and the
        // window bottom covers the tail
        assert!(max_scroll * cols < PICKER_BLOCKS.len());
        assert!((max_scroll + vis_rows) * cols >= PICKER_BLOCKS.len());
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
                assert_eq!(state_block(s), (s - BREWING_STAND_STATE + BREWING_STAND as u16) as u16);
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
                // VERIFICATION-REPORT fix #4: the coal item state
                || s == COAL_STATE
                // Phase E3 dedicated world-block + item + egg + POWER states
                || (COAL_BLOCK_STATE..=E3_EGG_STATE_END).contains(&s)
                || s == TRAPPED_CHEST_OPEN_STATE
                || (DAYLIGHT_POWER_BASE..=DAYLIGHT_POWER_END).contains(&s)
                || (LIGHT_PLATE_POWER_BASE..=LIGHT_PLATE_POWER_END).contains(&s)
                || (HEAVY_PLATE_POWER_BASE..=HEAVY_PLATE_POWER_END).contains(&s)
                || is_v2_state(s)
                || is_v3_state(s)
                || is_v4_state(s)
                || is_v5_state(s)
                || is_v6_state(s)
                || matches!(s, ACACIA_LOG_X | ACACIA_LOG_Z | DARK_OAK_LOG_X | DARK_OAK_LOG_Z)
            {
                assert!(!is_model_state(s), "component/item state {s} never routes to models");
                // identity: the state folds to the block whose def table
                // lists it (verified per-block in the dedicated ranges'
                // own tests)
                let b = state_block(s);
                assert!(b < BLOCK_COUNT as u16, "state {s} folds to valid block");
                // 1.7.2 V2: default_state must invert the fold exactly
                if is_v2_state(s) {
                    assert_eq!(default_state(b), s, "v2 state {s} roundtrip");
                }
                // 1.8 V3: same roundtrip contract
                if is_v3_state(s) {
                    assert_eq!(default_state(b), s, "v3 state {s} roundtrip");
                }
                // 1.9 V4: same roundtrip contract
                if is_v4_state(s) {
                    assert_eq!(default_state(b), s, "v4 state {s} roundtrip");
                }
                // 1.10 V5: same roundtrip contract
                if is_v5_state(s) {
                    assert_eq!(default_state(b), s, "v5 state {s} roundtrip");
                }
                // audit-fix V6 (1.2 jungle family + 1.4 golden carrot):
                // same roundtrip contract
                if is_v6_state(s) {
                    assert_eq!(default_state(b), s, "v6 state {s} roundtrip");
                }
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
            assert_eq!(water_level(s), l as u16);
            assert_eq!(state_block(s), WATER);
            assert!(!is_model_state(s));
        }
        assert_eq!(water_level(STONE as u16), 255);
        // redstone state roundtrips
        for p in 0u8..=15 {
            let s = wire_state(p);
            assert_eq!(wire_power(s), p as u16, "wire power roundtrip {p}");
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
        for b in 0..BLOCK_COUNT as u16 {
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
        for raw in [73u16, 88, 96, 118, 121, 255] {
            let d = def(raw);
            let _ = d.solid;
            let _ = is_solid(raw);
            let _ = is_opaque(raw);
            let _ = name(raw);
        }
    }

    // ---------------- Phase E3 tests (1.5–1.6 bracket) ----------------

    #[test]
    fn phase_e3_registry_roundtrips() {
        // every E3 block folds through its dedicated state and back
        let cases = [
            (COAL_BLOCK, COAL_BLOCK_STATE),
            (QUARTZ_BLOCK, QUARTZ_BLOCK_STATE),
            (CHISELED_QUARTZ, CHISELED_QUARTZ_STATE),
            (QUARTZ_PILLAR, QUARTZ_PILLAR_STATE),
            (HAY_BALE, HAY_BALE_STATE),
            (DAYLIGHT_SENSOR, DAYLIGHT_SENSOR_STATE),
            (TRAPPED_CHEST, TRAPPED_CHEST_STATE),
            (LIGHT_WEIGHTED_PLATE, LIGHT_PLATE_STATE),
            (HEAVY_WEIGHTED_PLATE, HEAVY_PLATE_STATE),
            (REDSTONE_BLOCK, REDSTONE_BLOCK_STATE),
            (NETHER_QUARTZ, NETHER_QUARTZ_STATE),
            (LEAD, LEAD_STATE),
            (SADDLE, SADDLE_STATE),
        ];
        for (b, s) in cases {
            assert_eq!(default_state(b), s, "block {b}");
            assert_eq!(state_block(s), b, "state {s}");
            assert!(!is_model_state(s), "state {s} never routes to models");
        }
        // 16 stained terracotta colors
        for c in 0u8..16 {
            let b = stained_terracotta(c);
            let s = stained_terracotta_state(c);
            assert_eq!(default_state(b), s, "terracotta color {c}");
            assert_eq!(state_block(s), b, "terracotta state {c}");
            assert_eq!(stained_terracotta_color(s), c as u16);
        }
        // 5 carpets (the engine wool palette)
        for (i, b) in [CARPET_WHITE, CARPET_RED, CARPET_YELLOW, CARPET_BLUE, CARPET_BLACK]
            .iter()
            .enumerate()
        {
            let s = CARPET_STATE_BASE + i as u16;
            assert_eq!(default_state(*b), s, "carpet {b}");
            assert_eq!(state_block(s), *b, "carpet state {s}");
            // carpets are non-solid, non-opaque (the 1/16-floor overlay)
            assert!(!def(*b).solid);
            assert!(!def(*b).opaque);
        }
        // E3 eggs decode to kinds 20..=22 (the horse/donkey/mule rows)
        for (i, want) in [(0u8, 20u8), (1, 21), (2, 22)] {
            let b = E3_SPAWN_EGG_BASE + i as u16;
            assert_eq!(egg_mob(b), Some(want), "E3 egg {b}");
            assert!(is_spawn_egg(b));
            assert_eq!(default_state(b), E3_EGG_STATE_BASE + i as u16);
        }
    }

    #[test]
    fn phase_e3_counts_and_picker() {
        // [merge renumber] E3-era totals (200 blocks / 400 states) grew
        // with the 1.7.2–1.10 F-series: 276 blocks / 480 states
        // (E-series states end at 354; V2 400..=442, V3 447..=465,
        // V4 466..=475, V5 476..=479)
        assert_eq!(BLOCK_COUNT, 282, "E1+E2+E3+1.7–1.10 merged registry + audit-fix V6");
        assert_eq!(STATE_COUNT, 486, "merged state space, V6 ends at 485");
        assert_eq!(BLOCK_TABLE.len(), BLOCK_COUNT);
        for want in [
            COAL_BLOCK,
            QUARTZ_BLOCK,
            QUARTZ_PILLAR,
            NETHER_QUARTZ,
            STAINED_TERRACOTTA_BASE,
            STAINED_TERRACOTTA_BASE + 15,
            CARPET_WHITE,
            CARPET_BLACK,
            HAY_BALE,
            DAYLIGHT_SENSOR,
            TRAPPED_CHEST,
            LIGHT_WEIGHTED_PLATE,
            HEAVY_WEIGHTED_PLATE,
            REDSTONE_BLOCK,
            LEAD,
            SADDLE,
            E3_SPAWN_EGG_BASE,
        ] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        // TILE_MAX covers every E3 tile (the Phase-4 blank-tile guard)
        assert!(TILE_MAX >= 243);
    }
}

#[cfg(test)]
mod v110_tests {
    use super::*;

    /// 1.10 Frostburn V5 window: ids 191..=194, states 328..=331 — the
    /// four new blocks round-trip through the registry (live-verified
    /// block list, minecraft.wiki/w/Java_Edition_1.10 §Blocks, fetched
    /// 2026-09-06)
    #[test]
    fn v5_window_roundtrips() {
        // [merge renumber] the V5 window moved from the pre-merge local
        // 328..=331 to 476..=479 (after the E1–E3 state series)
        for (b, s) in [
            (MAGMA_BLOCK, 476u16),
            (NETHER_WART_BLOCK, 477),
            (RED_NETHER_BRICKS, 478),
            (BONE_BLOCK, 479),
        ] {
            assert_eq!(v5_state(b), Some(s), "block {b} → state {s}");
            assert_eq!(state_block(s), b, "state {s} → block {b}");
            assert_eq!(default_state(b), s);
            assert!(is_v5_state(s));
        }
        assert_eq!(BLOCK_COUNT, 282);
        assert_eq!(STATE_COUNT, 486);
    }

    /// magma emits light level 3 (VERIFIED — minecraft.wiki/w/Magma_Block,
    /// live round 2026-09-06: "Magma blocks emit a light level of 3")
    #[test]
    fn magma_emits_light_level_3() {
        assert_eq!(def(MAGMA_BLOCK).emissive, 3);
    }
}

// ---------------------------------------------------------------------------
// audit-fix round tests (2026-09-07): the 1.2 jungle family registry
// ---------------------------------------------------------------------------
#[cfg(test)]
mod auditfix_tests {
    use super::*;

    /// the V6 window: jungle wood family + vine + fern register with
    /// correct solidity/cross flags and roundtrip through their states
    #[test]
    fn auditfix_v6_registry() {
        // ids 276..=281, states 480..=485 — non-overlapping with every
        // earlier window (the merge-round invariant)
        for (b, s) in [
            (GOLDEN_CARROT, 480u16),
            (JUNGLE_LOG, 481),
            (JUNGLE_LEAVES, 482),
            (JUNGLE_PLANKS, 483),
            (VINE, 484),
            (FERN, 485),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
            assert!(!is_model_state(s), "V6 states are cube/cross defs, not model states");
        }
        assert_eq!(V6_COUNT, 6);
        assert_eq!(BLOCK_COUNT, 282);
        assert_eq!(STATE_COUNT, 486);
        // solidity classes: log/planks solid-opaque (hardness family 2
        // per w/Log + w/Planks), leaves see-through, vine/fern non-solid
        // cross plants (w/Vines: "climbable non-solid"; w/Fern:
        // "non-solid plant blocks"), golden carrot an item-block
        assert!(is_solid(JUNGLE_LOG) && is_opaque(JUNGLE_LOG));
        assert!(is_solid(JUNGLE_PLANKS) && is_opaque(JUNGLE_PLANKS));
        assert!(is_solid(JUNGLE_LEAVES) && !is_opaque(JUNGLE_LEAVES));
        assert!(!is_solid(VINE) && is_cross(VINE));
        assert!(!is_solid(FERN) && is_cross(FERN));
        assert!(is_item_block(GOLDEN_CARROT) && is_cross(GOLDEN_CARROT));
        // every new tile is within the atlas guard (the Phase-4
        // blank-tile regression)
        assert!(TILE_GOLDEN_CARROT <= TILE_MAX && TILE_VINE <= TILE_MAX && TILE_FERN <= TILE_MAX);
        // the picker carries the family
        for b in [GOLDEN_CARROT, JUNGLE_LOG, JUNGLE_LEAVES, JUNGLE_PLANKS, VINE, FERN] {
            assert!(PICKER_BLOCKS.contains(&b), "picker missing {b}");
        }
    }
}
