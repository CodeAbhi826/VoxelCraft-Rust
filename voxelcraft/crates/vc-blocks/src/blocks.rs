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
// ---- 1.11 bracket tiles (Exploration Update, live 2026-09-07) ----
/// 1.11 mob sprites (w/Llama, /w/Vindicator, /w/Evoker, /w/Vex).
pub const TILE_LLAMA: u16 = 333;
pub const TILE_VINDICATOR: u16 = 334;
pub const TILE_EVOKER: u16 = 335;
pub const TILE_VEX: u16 = 336;
/// Shulker box (VERIFIED w/Shulker_Box: hardness 2, 27 slots, keeps
/// contents; craft = 2 shulker shells + chest column).
pub const TILE_SHULKER_BOX: u16 = 337;
/// Shulker shell (VERIFIED w/Shulker_Shell: 50% shulker drop — no
/// shulkers in engine, picker-only, documented).
pub const TILE_SHULKER_SHELL: u16 = 338;
/// Totem of undying (VERIFIED w/Totem_of_Undying live: restores 1 HP,
/// clears effects, Regeneration II 45 s + Absorption II 5 s — the Fire
/// Resistance row is a 1.16.2 addition per §History, version-scoped out
/// of this bracket).
pub const TILE_TOTEM: u16 = 339;

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

// ---- 1.11 bracket (Exploration Update): ids 282..=288, V7 window ----
/// Shulker Box — container (VERIFIED w/Shulker_Box live: hardness 2,
/// "All shulker boxes have 27 inventory slots, the same as a barrel, a
/// single chest, or an ender chest"; "keep their items when broken";
/// craft = "Shulker Shell + Chest" column; "cannot be placed inside
/// another" shulker box; "drops itself as an item if pushed by
/// pistons"). Engine adaptation: item entities carry no NBT, so a
/// broken box spills its contents (the vanilla keep-inside-the-item
/// needs item-NBT — disclosed); the no-nesting rule IS enforced.
pub const SHULKER_BOX: u16 = 282;
/// Shulker Shell — item (VERIFIED w/Shulker_Shell: 50% shulker drop; no
/// shulkers/End cities in engine → picker-only, documented).
pub const SHULKER_SHELL: u16 = 283;
/// Totem of Undying — item (VERIFIED live w/Totem_of_Undying: revives
/// the holder on lethal damage, "restores 1 HP, removes all existing
/// status effects and grants" Regeneration II 45 s + Absorption II 5 s.
/// Evoker drop ("Evokers always drop one of these upon death" — 1.11
/// changelog). Fire Resistance I (0:40) is a 1.16.2 addition
/// (w/Totem §History 20w28a) — version-scoped OUT of this bracket.
pub const TOTEM_OF_UNDYING: u16 = 284;
/// 1.11 spawn eggs (changelog §Items: "5 new spawn eggs" — Vindicator,
/// Llama, Evoker, Vex, Zombie Villager): kinds 23..=26. The zombie-
/// villager egg — the 5th of the new set — is ALREADY in the engine:
/// the E2-era egg item at id 129 (egg kind 5, added with the zombie-
/// villager cure round; an engine anachronism, disclosed in the WORKLOG
/// — it satisfies the 1.11 requirement without a duplicate item).
pub const SPAWN_EGG_LLAMA: u16 = 285;
pub const SPAWN_EGG_VINDICATOR: u16 = 286;
pub const SPAWN_EGG_EVOKER: u16 = 287;
pub const SPAWN_EGG_VEX: u16 = 288;
/// 1.11 re-added eggs (VERIFIED changelog §Items: "Eggs that were
/// removed in Java Edition 1.10-pre2 are re-added, except the cat
/// spawn egg, including: ... Husk spawn egg, Stray spawn egg"): husk
/// kind 27, stray 28. The wither-skeleton re-add is covered by the
/// existing kind 16; donkey/mule re-adds by kinds 21/22; skeleton
/// horse / zombie horse / elder guardian eggs are palette-absent
/// (those mobs don't exist — N/A, disclosed).
pub const SPAWN_EGG_HUSK: u16 = 289;
pub const SPAWN_EGG_STRAY: u16 = 290;

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

/// 1.11 mansion spawners: mob-kind code 5 (vindicator) — the
/// woodland-mansion illager placement (engine-native adaptation;
/// vanilla spawns them at generation without respawn — disclosed).
pub const SPAWNER_VINDICATOR: u16 = 495;
/// 1.11 mansion spawners: mob-kind code 6 (evoker, the two upper
/// floors — VERIFIED w/Evoker: "Spawn in the two upper floors of
/// woodland mansions upon generation").
pub const SPAWNER_EVOKER: u16 = 496;

pub const V7_STATE_BASE: u16 = 486;
// 9 ids: 282..=290 (the 1.11 bracket + the completion round's
// re-added husk/stray eggs). States 486..=494; the mansion spawner
// states sit directly above at 495..=496 (STATE_COUNT = 497).
pub const V7_COUNT: u16 = 9;
pub const V7_STATE_TO_BLOCK: [u16; V7_COUNT as usize] = [
    SHULKER_BOX, SHULKER_SHELL, TOTEM_OF_UNDYING, SPAWN_EGG_LLAMA,
    SPAWN_EGG_VINDICATOR, SPAWN_EGG_EVOKER, SPAWN_EGG_VEX,
    SPAWN_EGG_HUSK, SPAWN_EGG_STRAY,
];

#[inline]
pub fn v7_state(b: u16) -> Option<u16> {
    if (282..282 + V7_COUNT as u16).contains(&b) {
        Some(V7_STATE_BASE + (b - 282) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v7_state(s: u16) -> bool {
    (V7_STATE_BASE..V7_STATE_BASE + V7_COUNT).contains(&s)
}

// ---- 1.12 bracket (World of Color Update): ids 291..=360, V8 window ----
// All values VERIFIED live 2026-09-07 (minecraft.wiki/w/Java_Edition_1.12
// §Additions + the per-block pages; research record
// docs/research/phase-v112-1.12-research.md):
// * concrete — 16 colors; "Created when concrete powder comes into
//   contact with still or flowing water"; hardness 1.8 (w/Concrete)
// * concrete powder — 16 colors; "Gravity affected (like sand and
//   gravel). When it touches water, it turns into a concrete block";
//   hardness 0.5; recipe 4 sand + 4 gravel + 1 dye → 8, shapeless
// * glazed terracotta — 16 colors; "Smelt any stained terracotta";
//   4-directional facing; hardness 1.4
// * parrot spawn egg (kind 30); 16 dyes + 4 seeds + cookie — the
//   recipe/taming/cookie-death input items (palette-only; the dye
//   ACQUISITION economy stays deferred, the standing disclosure)
pub const CONCRETE_BASE: u16 = 291;
pub const CONCRETE_END: u16 = 306;
pub const CONCRETE_POWDER_BASE: u16 = 307;
pub const CONCRETE_POWDER_END: u16 = 322;
pub const GLAZED_TERRACOTTA_BASE: u16 = 323;
pub const GLAZED_TERRACOTTA_END: u16 = 338;
/// 1.12 parrot spawn egg (changelog §Items: "Parrot Spawn Egg — Spawns
/// parrots"). Mob kind 30 (egg_mob/egg_id 30 — the V8 egg window).
pub const SPAWN_EGG_PARROT: u16 = 339;
/// 16 dye items, engine color order (= the stained-terracotta order).
pub const DYE_BASE: u16 = 340;
pub const DYE_END: u16 = 355;
/// The 4 parrot-taming seeds (wheat/melon/pumpkin/beetroot).
pub const WHEAT_SEEDS: u16 = 356;
pub const MELON_SEEDS: u16 = 357;
pub const PUMPKIN_SEEDS: u16 = 358;
pub const BEETROOT_SEEDS: u16 = 359;
/// Cookie (the toxic parrot food — VERIFIED w/Parrot: "feeding a cookie
/// to a parrot kills it... the parrot receives 2128 (3.4028 x 10^38)").
pub const COOKIE: u16 = 360;

// ---- 1.13 bracket (Update Aquatic): ids 361..=416, V9 window ----
// All values VERIFIED live 2026-09-07 (minecraft.wiki/w/Java_Edition_1.13
// §Additions + the per-block pages; research record
// docs/research/phase-v113-1.13-research.md):
// * coral blocks ×5 — "Comes in the same 5 variants as coral: tube
//   (blue), brain (pink), bubble (purple), fire (red), horn (yellow)
//   ... Turns into a dead coral block if none of its six sides are
//   touching the water" — hardness 1.5 (w/Coral_Block)
// * dead coral blocks ×5 — irreversible; silk-touch-free drop of live
//   coral (w/Coral_Block: mined without Silk Touch → dead)
// * coral plants + fans ×5 each (+ dead) — underwater-only placement,
//   cross-plant rendering (the engine's non-cube form); the FAN form
//   renders as a cross instead of vanilla's side-mount — disclosed
//   adaptation
// * sea pickle — "Up to 4 of them can be placed on a block. Each one
//   adds 3 to the light level, but only when placed underwater"
//   (changelog; w/Sea_Pickle: 1 pickle = 6, +3 per extra → 6/9/12/15)
// * blue ice — hardness 2.8, "Slippier than ice and packed ice";
//   slipperiness 0.989 (w/Blue_Ice §History: 0.999 → 0.989 in 1.15;
//   the 1.13 value was 0.999 — version-scoped, we ship the modern
//   0.989 and document both)
// * dried kelp block — fuel "smelts 20 items" (4000 ticks; VERIFIED
//   w/Dried_Kelp_Block §Fuel)
// * kelp — "Can grow multiple blocks high ... Can be smelted into dry
//   kelp"; 14%/random-tick growth (w/Kelp §Farming)
// * seagrass — the tall-grass analogue underwater; turtles' breeding
//   food + "Drops from turtles when killed" (changelog)
// * conduit — "Crafted using 1 heart of the sea and 8 nautilus shells
//   ... Emits a strong glow, at light level 15" (changelog; the frame
//   needs 16-42 prismarine-family blocks — w/Conduit)
// * turtle egg — 3 hatch stages (uncracked/slightly/very cracked;
//   changelog: "After a while, they become slightly cracked and then
//   very cracked. Very cracked turtle eggs eventually hatch into baby
//   turtles")
// * items: heart of the sea, nautilus shell, scute, trident, phantom
//   membrane, dried kelp, turtle shell + the 4 potions (Slow Falling,
//   Slow Falling extended, Turtle Master, Turtle Master enhanced)
// * 8 spawn eggs (drowned/phantom/dolphin/cod/salmon/pufferfish/
//   tropical fish/turtle — the changelog's own §Items spawn-egg list)
pub const CORAL_BLOCK_BASE: u16 = 361;
pub const CORAL_BLOCK_END: u16 = 365;
pub const DEAD_CORAL_BLOCK_BASE: u16 = 366;
pub const DEAD_CORAL_BLOCK_END: u16 = 370;
pub const CORAL_PLANT_BASE: u16 = 371;
pub const CORAL_PLANT_END: u16 = 375;
pub const DEAD_CORAL_PLANT_BASE: u16 = 376;
pub const DEAD_CORAL_PLANT_END: u16 = 380;
pub const CORAL_FAN_BASE: u16 = 381;
pub const CORAL_FAN_END: u16 = 385;
pub const DEAD_CORAL_FAN_BASE: u16 = 386;
pub const DEAD_CORAL_FAN_END: u16 = 390;
/// 1.13 sea pickle — 1-4 pickles per block; light 6/9/12/15 when the
/// block is in water (VERIFIED w/Sea_Pickle: "A single sea pickle
/// produces a light level of 6, and each additional pickle increases
/// the level by 3").
pub const SEA_PICKLE: u16 = 391;
/// 1.13 blue ice — crafted from 9 packed ice; the slipperiest block.
pub const BLUE_ICE: u16 = 392;
/// 1.13 dried kelp block — fuel: 4000 ticks / 20 items (VERIFIED).
pub const DRIED_KELP_BLOCK: u16 = 393;
/// 1.13 kelp plant — grows 14% per random tick (VERIFIED w/Kelp).
pub const KELP: u16 = 394;
/// 1.13 seagrass — the underwater grass (turtle breeding food).
pub const SEAGRASS: u16 = 395;
/// 1.13 conduit — heart-of-the-sea + 8 nautilus shells; light 15.
pub const CONDUIT: u16 = 396;
/// 1.13 turtle egg — 3 hatch stages, tramples, hatches baby turtles.
pub const TURTLE_EGG: u16 = 397;
/// 1.13 heart of the sea — the conduit core (buried treasure loot).
pub const HEART_OF_THE_SEA: u16 = 398;
/// 1.13 nautilus shell — 8 per conduit; fished (treasure class) +
/// drowned offhand (3% JE — VERIFIED w/Drowned §Equipment).
pub const NAUTILUS_SHELL: u16 = 399;
/// 1.13 scute — "Dropped when baby turtles grow up" (changelog).
pub const SCUTE: u16 = 400;
/// 1.13 trident — "A new weapon ... dealing 9 HP damage" melee /
/// 8 HP thrown; drowned drop (changelog + w/Trident).
pub const TRIDENT: u16 = 401;
/// 1.13 phantom membrane — Slow Falling brewing + elytra repair.
pub const PHANTOM_MEMBRANE: u16 = 402;
/// 1.13 dried kelp — "Can be eaten, restoring 1 hunger point" (1 HP-
/// heal in the engine's hunger-bar-less convention, documented).
pub const DRIED_KELP: u16 = 403;
/// 1.13 turtle shell — 5 scutes; the Turtle Master brewing item (the
/// helmet's water-breathing-on-wear is deferred with the armor system
/// — disclosed in the worklog).
pub const TURTLE_SHELL: u16 = 404;
/// 1.13 potion of Slow Falling (1:30 — VERIFIED w/Slow_Falling).
pub const POTION_SLOW_FALLING: u16 = 405;
/// 1.13 potion of Slow Falling, extended (4:00 — redstone modifier).
pub const POTION_SLOW_FALLING_EXT: u16 = 406;
/// 1.13 potion of the Turtle Master (Slowness IV + Resistance III,
/// 1:00 — VERIFIED changelog §Items).
pub const POTION_TURTLE_MASTER: u16 = 407;
/// 1.13 potion of the Turtle Master, enhanced (Slowness VI +
/// Resistance IV — the glowstone modifier, VERIFIED changelog).
pub const POTION_TURTLE_MASTER_II: u16 = 408;
/// 1.13 spawn eggs (kinds 32..=39 — the V9 egg window; the changelog's
/// own list: Drowned/Dolphin/Cod/Salmon/Phantom/Pufferfish/
/// Tropical Fish/Turtle Spawn Eggs).
pub const SPAWN_EGG_DROWNED: u16 = 409;
pub const SPAWN_EGG_PHANTOM: u16 = 410;
pub const SPAWN_EGG_DOLPHIN: u16 = 411;
pub const SPAWN_EGG_COD: u16 = 412;
pub const SPAWN_EGG_SALMON: u16 = 413;
pub const SPAWN_EGG_PUFFERFISH: u16 = 414;
pub const SPAWN_EGG_TROPICAL_FISH: u16 = 415;
pub const SPAWN_EGG_TURTLE: u16 = 416;

pub const V9_STATE_BASE: u16 = 615;
// 61 states: coral families 30 (5 blocks + 5 dead + 5 plants + 5 dead
// plants + 5 fans + 5 dead fans), sea pickle 4 (count), blue ice 1,
// dried kelp block 1, kelp 1, seagrass 1, conduit 1, turtle egg 3
// (hatch stages), 11 items (heart/nautilus/scute/trident/membrane/
// dried kelp/turtle shell + 4 potions), 8 spawn eggs.
pub const V9_COUNT: u16 = 61;
/// V9 state → block fold: 1:1 for every window block (the pickle's
/// 4 count states and the egg's 3 hatch states all fold to their
/// parent — the per-state art rides state_tiles).
pub const V9_STATE_TO_BLOCK: [u16; V9_COUNT as usize] = [
    // coral blocks (5) + dead (5)
    CORAL_BLOCK_BASE, CORAL_BLOCK_BASE + 1, CORAL_BLOCK_BASE + 2,
    CORAL_BLOCK_BASE + 3, CORAL_BLOCK_BASE + 4,
    DEAD_CORAL_BLOCK_BASE, DEAD_CORAL_BLOCK_BASE + 1, DEAD_CORAL_BLOCK_BASE + 2,
    DEAD_CORAL_BLOCK_BASE + 3, DEAD_CORAL_BLOCK_BASE + 4,
    // coral plants (5) + dead (5)
    CORAL_PLANT_BASE, CORAL_PLANT_BASE + 1, CORAL_PLANT_BASE + 2,
    CORAL_PLANT_BASE + 3, CORAL_PLANT_BASE + 4,
    DEAD_CORAL_PLANT_BASE, DEAD_CORAL_PLANT_BASE + 1, DEAD_CORAL_PLANT_BASE + 2,
    DEAD_CORAL_PLANT_BASE + 3, DEAD_CORAL_PLANT_BASE + 4,
    // coral fans (5) + dead (5)
    CORAL_FAN_BASE, CORAL_FAN_BASE + 1, CORAL_FAN_BASE + 2,
    CORAL_FAN_BASE + 3, CORAL_FAN_BASE + 4,
    DEAD_CORAL_FAN_BASE, DEAD_CORAL_FAN_BASE + 1, DEAD_CORAL_FAN_BASE + 2,
    DEAD_CORAL_FAN_BASE + 3, DEAD_CORAL_FAN_BASE + 4,
    // sea pickle: 4 count states
    SEA_PICKLE, SEA_PICKLE, SEA_PICKLE, SEA_PICKLE,
    // blue ice / dried kelp block / kelp / seagrass / conduit
    BLUE_ICE, DRIED_KELP_BLOCK, KELP, SEAGRASS, CONDUIT,
    // turtle egg: 3 hatch stages
    TURTLE_EGG, TURTLE_EGG, TURTLE_EGG,
    // 11 items
    HEART_OF_THE_SEA, NAUTILUS_SHELL, SCUTE, TRIDENT, PHANTOM_MEMBRANE,
    DRIED_KELP, TURTLE_SHELL,
    POTION_SLOW_FALLING, POTION_SLOW_FALLING_EXT,
    POTION_TURTLE_MASTER, POTION_TURTLE_MASTER_II,
    // 8 spawn eggs
    SPAWN_EGG_DROWNED, SPAWN_EGG_PHANTOM, SPAWN_EGG_DOLPHIN, SPAWN_EGG_COD,
    SPAWN_EGG_SALMON, SPAWN_EGG_PUFFERFISH, SPAWN_EGG_TROPICAL_FISH,
    SPAWN_EGG_TURTLE,
];

#[inline]
pub fn v9_state(b: u16) -> Option<u16> {
    if (CORAL_BLOCK_BASE..=DEAD_CORAL_FAN_END).contains(&b) {
        Some(V9_STATE_BASE + (b - CORAL_BLOCK_BASE) as u16)
    } else if b == SEA_PICKLE {
        // default placement: one pickle
        Some(V9_STATE_BASE + 30)
    } else if (BLUE_ICE..=CONDUIT).contains(&b) {
        Some(V9_STATE_BASE + 34 + (b - BLUE_ICE) as u16)
    } else if b == TURTLE_EGG {
        Some(V9_STATE_BASE + 39)
    } else if (HEART_OF_THE_SEA..=SPAWN_EGG_TURTLE).contains(&b) {
        Some(V9_STATE_BASE + 42 + (b - HEART_OF_THE_SEA) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v9_state(s: u16) -> bool {
    (V9_STATE_BASE..V9_STATE_BASE + V9_COUNT).contains(&s)
}

/// sea pickle count (1..4) from its storage state.
#[inline]
pub fn sea_pickle_count(s: u16) -> u8 {
    if (V9_STATE_BASE + 30..=V9_STATE_BASE + 33).contains(&s) {
        1 + (s - (V9_STATE_BASE + 30)) as u8
    } else if state_block(s) == SEA_PICKLE {
        1
    } else {
        0
    }
}

/// sea pickle light level: 0 out of water; 6/9/12/15 by count in water
/// (VERIFIED w/Sea_Pickle — the waterlogged gate is the Java rule).
pub fn sea_pickle_light(s: u16, in_water: bool) -> u8 {
    if !in_water {
        return 0;
    }
    match sea_pickle_count(s) {
        1 => 6,
        2 => 9,
        3 => 12,
        4 => 15,
        _ => 0,
    }
}

/// turtle egg hatch stage (0..2) from its storage state.
#[inline]
pub fn turtle_egg_stage(s: u16) -> u8 {
    if (V9_STATE_BASE + 39..=V9_STATE_BASE + 41).contains(&s) {
        (s - (V9_STATE_BASE + 39)) as u8
    } else {
        0
    }
}

/// state for placing a turtle egg at a given hatch stage.
#[inline]
pub fn turtle_egg_state(stage: u8) -> u16 {
    V9_STATE_BASE + 39 + (stage.min(2)) as u16
}

/// state for placing sea pickles (count 1..4).
#[inline]
pub fn sea_pickle_state(count: u8) -> u16 {
    V9_STATE_BASE + 30 + (count.saturating_sub(1)).min(3) as u16
}

// ---- 1.14 bracket (Village & Pillage — NATURE HALF): ids 417..=425,
// the V10 window. All values VERIFIED live 2026-09-08 from the raw
// captures scripts/v114_page_*.json (bamboo, sweetberrybush, campfire,
// barrel, fox — minecraft.wiki) — the research record
// docs/research/phase-v114-1.14-research.md:
// * bamboo stalk — "a versatile, fast-growing plant found primarily in
//   jungles"; growth "Upon receiving a random tick, bamboo has a 1/3
//   chance of growing"; "The top of a bamboo plant requires a client
//   light level of 9 or above"; "can grow up to 12-16 blocks tall"
// * bamboo shoot — "the initial non-solid sapling form of planted
//   bamboo"
// * sweet berry bush — age 0..3 ("third growth stage" = age 2,
//   "mature" = age 3); "A sweet berry bush (at any stage) slows down
//   all entities (except items) passing through it. At stage 1 and
//   higher, it causes damage"; damage "1 HP every tick (although
//   damage immunity reduces this to once every half-second), only if
//   the entity is moving"; slow "to about 34.05% of their normal
//   speed"; growth "a 20% chance per random tick"; taiga/snowy-taiga
//   generation at "a 1/12 chance" per chunk
// * sweet berries (item) — food "restores 2 hunger and 0.4 [JE]
//   saturation"; plants a bush on grass-family blocks; fox breeding
//   food (VERIFIED w/Sweet_Berries §Breeding)
// * campfire — hardness 2, "Luminous Yes (15) when lit"; breaking
//   "drops 2 charcoal"; "deal 1 HP every tick (although damage
//   immunity reduces this to once every half-second)" to mobs
//   STANDING ON a lit one; cooking "30 seconds (600 ticks)" across
//   4 food slots, "campfires do not require any kind of fuel";
//   smoke "floats up around 10 blocks" (24 over a hay bale)
// * barrel — "a container inventory with 27 slots, which is the same
//   as a single chest"; "the action of opening a barrel is never
//   prevented"; hardness 2.5; crafted from "6 wood planks and 2 wood
//   slabs" (the 18w50a history row)
// * fox spawn egg (kind 40) + STICK and CHARCOAL — two LEGACY items
//   added late (see the row docs below): the first recipes that
//   consume a stick (the campfire) and produce charcoal (log
//   smelting + the campfire drop) arrive in THIS bracket, so the
//   1.14 window carries them. Disclosed.
//
// The village half of 1.14 (villages/pillager/raids/crossbow/bell/
// wandering trader/loom/stonecutter/...) is deferred until the engine
// has village+raid scaffolding — same deferral class as prior brackets.
pub const BAMBOO: u16 = 417;
pub const BAMBOO_SHOOT: u16 = 418;
pub const SWEET_BERRY_BUSH: u16 = 419;
pub const CAMPFIRE: u16 = 420;
pub const BARREL: u16 = 421;
/// 1.14: sweet berries — the food + planting item (inventory-only;
/// the plant branch in the game layer turns a use on soil into a bush).
pub const SWEET_BERRIES: u16 = 422;
/// 1.14: the fox spawn egg (kind 40 — the next egg window slot).
pub const SPAWN_EGG_FOX: u16 = 423;
/// LEGACY item added in the 1.14 window: the stick (an Alpha-era item
/// the engine never needed until the campfire recipe — 3 sticks per
/// campfire, VERIFIED w/Campfire §Crafting "Stick + Coal or Charcoal
/// + Any Log..."). Crafted 2 planks (vertical) -> 4 sticks.
pub const STICK: u16 = 424;
/// LEGACY item added in the 1.14 window: charcoal (smelted from any
/// log since Indev — the engine's furnace round predates cooked-meat
/// forms and never added it). Sources: log smelting + the campfire's
/// 2-charcoal drop. Fuel-identical to coal (1600 ticks).
pub const CHARCOAL: u16 = 425;

// ---- 1.14 (Village & Pillage — nature half, part 2): the smelting
// trio + lantern. VERIFIED 2026-09-08 from the raw captures
// scripts/v114b_page_*.json (w/Blast_Furnace, w/Smoker, w/Lantern,
// w/Smooth_Stone). ----
/// blast furnace: smelts the ORE/metal class 2x as fast (fuel burns
/// 2x as fast too — same items per fuel). Lit state emits light 13.
pub const BLAST_FURNACE: u16 = 426;
/// smoker: cooks the FOOD class 2x as fast (5 s per item vs 10).
/// Lit state emits light 13.
pub const SMOKER: u16 = 427;
/// lantern: light 15 (brighter than the torch's 14). Sits on top of
/// blocks or hangs from their underside (the hanging state).
pub const LANTERN: u16 = 428;
/// iron nugget — the 1.11-era item the lantern recipe needed
/// (8 nuggets + torch); crafts 9:1 with the engine's iron-ingot
/// stand-in (IRON_ORE items, the disclosed convention).
pub const IRON_NUGGET: u16 = 429;
// ---- 1.14 (part 3): the two new small flowers (18w43a). VERIFIED
// 2026-09-08 from the raw captures scripts/v114c_page_*.json
// (w/Cornflower, w/Lily_of_the_Valley). ----
/// cornflower: non-solid cross plant, instant-break, drops itself;
/// crafts into BLUE dye; generates in plains / sunflower plains /
/// flower forest on grass-dirt (our biome set: Plains, SunflowerPlains,
/// FlowerForest).
pub const CORNFLOWER: u16 = 430;
/// lily of the valley: non-solid cross plant, instant-break, drops
/// itself; crafts into WHITE dye; generates in forest-family biomes
/// (our set: Forest, BirchForest, FlowerForest).
pub const LILY_OF_THE_VALLEY: u16 = 431;

// ---- 1.15 bracket (Buzzy Bees). VERIFIED 2026-09-08 from the raw
// captures scripts/v115_page_*.json (w/Bee, w/Beehive, w/Bee_nest,
// w/Honey_Block, w/Honey_Bottle, w/Honeycomb, w/Honeycomb_Block).
// docs/research/phase-v115-1.15-research.md is the value contract. ----
/// 1.15: the bee nest — naturally generated on trees (plains/
/// sunflower plains 5%, flower forest 2%, forest-family 0.2% — the
/// 1.16.5 biome set; meadow/mangrove/cherry rows are post-1.16.5,
// out of scope), spawns holding 2-3 bees, hardness 0.3 / blast 0.3,
/// flammable 30, axe-quickest. NOT craftable. honey_level 0..=5 in
/// the V12 state window (level 5 = honey oozing). Shears on a full
/// nest drop 3 honeycomb (angering the bees inside unless a lit
/// campfire sits within 5 blocks below); a glass bottle fills with
/// honey. Broken without Silk Touch it drops NOTHING (bees emerge
/// angry) — the disclosed no-Silk-Touch adaptation.
pub const BEE_NEST: u16 = 432;
/// 1.15: the beehive — the craftable twin (6 planks + 3 honeycomb),
/// hardness 0.6 / blast 0.6, flammable 5, axe-quickest, note-block
/// Bass. Same honey_level 0..=5 machinery as the nest; always drops
/// itself when broken (bees inside released angry).
pub const BEEHIVE: u16 = 433;
/// 1.15: the honey block — "a storage block equivalent to the
/// contents of four honey bottles". Hardness 0 / blast 0, any tool,
/// non-flammable, translucent (diffuses sky light — the JE partial
/// row). Movement: entities walk at ~2.508 m/s (60% slow), jump
/// height cut to 3/16 blocks (85% reduction), fall damage reduced
/// by 80%, entities pressed against the sides slide down slowly
/// without fall damage. Crafted from 4 honey bottles and back into
/// 4 bottles (both directions).
pub const HONEY_BLOCK: u16 = 434;
/// 1.15: the honeycomb block — decorative, hardness 0.6 / blast 0.6,
/// any tool, non-flammable. Crafted from 4 honeycomb (2x2).
pub const HONEYCOMB_BLOCK: u16 = 435;
/// 1.15: honeycomb — the item shears pop out of a full nest/hive
/// (3 per harvest). Crafts beehives + honeycomb blocks. (Copper
/// waxing + candles are 1.17 — out of the 1.16.5 window.)
pub const HONEYCOMB: u16 = 436;
/// 1.15: the honey bottle — a drinkable food item: restores 6 hunger
/// + 1.2 saturation, REMOVES Poison (and only Poison), returns the
/// glass bottle when drunk. Obtained by using a glass bottle on a
/// honey_level-5 nest/hive. Craft ingredient for the honey block.
pub const HONEY_BOTTLE: u16 = 437;
/// LEGACY item added in the 1.15 window (the stick/charcoal
/// precedent): shears — vanilla is a Beta-era tool (2 iron ingots,
/// diagonal). This round needs them for the honeycomb harvest.
/// Durability 238 is NOT modeled (no tool-durability system,
/// disclosed). Sheep-wool shearing + fast leaf/cobweb breaking are
/// follow-ups (noted in the research record).
pub const SHEARS: u16 = 438;
/// 1.15: the bee spawn egg (changelog §Items: "Bee Spawn Egg") —
/// mob kind 41.
pub const SPAWN_EGG_BEE: u16 = 439;

// ---- 1.16 (Nether Update, PART 1 — the anchor family): the V13
// block window, ids 440..=453. All values VERIFIED against the
// captures v116_page_*.json (see the research record
// docs/research/phase-v116-1.16-research.md). ----
/// 1.16: soul soil — the nether's soul-fire host ("burns indefinitely
/// when manually ignited on the top side only, creating soul fire",
/// VERIFIED w/Soul_Soil). Hardness 0.5 / blast 0.5, shovel. The soul
/// sand valley terrain component (this engine: soul patches in the
/// Nether, the disclosed no-sub-biome adaptation).
pub const SOUL_SOIL: u16 = 440;
/// 1.16: basalt — hardness 1.25 / blast 4.2, pickaxe (VERIFIED
/// w/Basalt). Pillars + blobs on the Nether body (the soul-sand-valley
/// / basalt-deltas adaptation); polished/smooth variants ride the
/// part-2 wood round with their crafting families.
pub const BASALT: u16 = 441;
/// 1.16: blackstone — hardness 1.5 / blast 6, pickaxe (VERIFIED
/// w/Blackstone). "Small patches in all Nether biomes" (the 20w19a
/// row) — the bastion source is part 2 with the piglins.
pub const BLACKSTONE: u16 = 442;
/// 1.16: gilded blackstone — the gold-flecked blackstone: "a 10%
/// chance to drop 2–5 gold nuggets when mined with any pickaxe. If it
/// does not drop gold nuggets, it drops itself as a block" (VERIFIED
/// w/Gilded_Blackstone). Native to bastions; generates inside this
/// engine's blackstone patches (disclosed adaptation). Gold nugget =
/// the iron-nugget stand-in (the disclosed convention).
pub const GILDED_BLACKSTONE: u16 = 443;
/// 1.16: crying obsidian — the luminous (light 10) obsidian variant,
/// hardness 50 / blast 1,200 (VERIFIED w/Crying_Obsidian). Obtained
/// from piglin bartering / bastion loot (part 2) — here: world-gen
/// trace in blackstone + the crafting path. Crafts the respawn anchor.
pub const CRYING_OBSIDIAN: u16 = 444;
/// 1.16: the respawn anchor — "a block that allows the player to set
/// their spawn point in the Nether, provided it's fueled with
/// glowstone blocks" (VERIFIED w/Respawn_Anchor). Charges 0..=4
/// (blockstates): charge light 3/7/11/15; using it in the Nether sets
/// the respawn point (needs ≥ 1 charge, each respawn consumes one);
/// using it in the Overworld explodes it (power 5). Craft: 6 crying
/// obsidian + 3 glowstone. Hardness 50 / blast 1,200.
pub const RESPAWN_ANCHOR: u16 = 445;
/// 1.16: the target block — "produces a temporary redstone signal when
/// hit by a projectile" (VERIFIED w/Target): power 1..15 by proximity
/// to the block center, 8 game ticks (20 for arrows/tridents — the
/// stone-button row). Hardness 0.5 / blast 0.5. Craft: 4 redstone dust
/// (the engine's redstone-block stand-in, disclosed) + 1 hay bale.
pub const TARGET: u16 = 446;
/// 1.16: nether gold ore — "drops 2–6 gold nuggets when mined with any
/// pickaxe" (VERIFIED w/Nether_Gold_Ore; Fortune is absent —
/// disclosed). Smelts to a gold ingot (the iron stand-in).
pub const NETHER_GOLD_ORE: u16 = 447;
/// 1.16: ancient debris — "generates in the Nether in the form of
/// scatter ores which are never naturally exposed to air"; hardness
/// 30 / blast 1,200 (VERIFIED w/Ancient_Debris). Java gen: one
/// 0–3-cluster triangle y 8–24 (peak 16) + one 0–2-cluster even y
/// 8–119 per chunk. Smelts into 1 netherite scrap + 2 XP.
pub const ANCIENT_DEBRIS: u16 = 448;
/// 1.16: the block of netherite — 9 ingots both ways (the disc row:
/// blast 1,200, hardness 50 class). The gear-upgrade path needs the
/// tool/armor system (the standing 1.11+ deferral) — the block is the
/// material endpoint here.
pub const NETHERITE_BLOCK: u16 = 449;
/// 1.16: the chain — hardness 0.5 / blast 6, "a metallic decoration
/// block made from ingots and nuggets" (VERIFIED w/Chain — the 1.16
/// iron-only form; the copper variants are 1.21+). Craft: 2 iron
/// nuggets + 1 iron ingot. Sits on block tops or hangs from undersides
/// (the lantern's face-matched states, extended).
pub const CHAIN: u16 = 450;
/// 1.16: soul fire — "the fire inflicts damage at a rate of 2 HP per
/// tick, twice as many as with the normal fire (although damage
/// immunity reduces this to once every half-second)" (VERIFIED
/// w/Soul_Fire) → 2 HP per half-second through the engine's shared
/// damage-immunity window. Light 10; does not spread; generates on
/// soul sand/soil (flint-and-steel ignition is not in the engine —
/// disclosed).
pub const SOUL_FIRE: u16 = 451;
/// 1.16: netherite scrap — smelted from ancient debris ("which is
/// found in the Nether"), crafts netherite ingots. Item-row (never
/// placeable).
pub const NETHERITE_SCRAP: u16 = 452;
/// 1.16: netherite ingot — "crafting four netherite scraps and four
/// gold ingots together" (VERIFIED w/Netherite_Ingot; gold = the
/// engine's iron-ingot stand-in, the disclosed convention). Item-row.
pub const NETHERITE_INGOT: u16 = 453;

// ---- 1.16 (Nether Update, PART 2 — the crimson/warped families):
// the V14 block window, ids 454..=478. All values VERIFIED against
// the captures v116b_page_*.json (see the research record
// docs/research/phase-v116b-1.16-research.md). ----
/// 1.16: crimson stem — the huge-crimson-fungus trunk (VERIFIED
/// w/Crimson_Stem: hardness 2 / blast 2, the log class). "Used as
/// a building block, crafted into planks"; the flammability (5) is
/// the log class. Stripped forms need the axe-strip mechanic — the
/// engine's standing deferral, disclosed.
pub const CRIMSON_STEM: u16 = 454;
/// 1.16: crimson hyphae — the "bark" variant (all-six-sides stem
/// texture, the jungle-log-bark pattern).
pub const CRIMSON_HYPHAE: u16 = 455;
/// 1.16: crimson planks — hardness 2 / blast 3 (VERIFIED
/// w/Crimson_Planks), crafted 1:4 from stem or hyphae.
pub const CRIMSON_PLANKS: u16 = 456;
/// 1.16: crimson nylium — the crimson-forest floor ("The forest
/// floor is mostly covered with crimson nylium", VERIFIED
/// w/Crimson_Forest). Hardness 0.4, shovel; drops netherrack when
/// mined (the nylium row, engine's grass-block-to-dirt pattern).
pub const CRIMSON_NYLIUM: u16 = 457;
/// 1.16: crimson fungus — the crimson-forest mushroom (hardness 0,
/// any tool, transparent cross plant — VERIFIED w/Crimson_Fungus).
/// "Can also be fed to hoglins" (the hoglin breeding item).
pub const CRIMSON_FUNGUS: u16 = 458;
/// 1.16: crimson roots — the crimson-forest tuft (hardness 0,
/// transparent cross — VERIFIED w/Crimson_Roots; drops itself
/// without shears — the 1.16.2-pre2 revert row).
pub const CRIMSON_ROOTS: u16 = 459;
/// 1.16: weeping vines — the crimson-forest hanging vine (hardness
/// 0, transparent, "downwards-growing", VERIFIED w/Weeping_Vines);
/// "generate naturally ... on huge crimson fungi" ceilings; the
/// 1-in-3 self-drop chance rides the game layer (disclosed).
pub const WEEPING_VINES: u16 = 460;
/// 1.16: warped stem — the huge-warped-fungus trunk (the crimson
/// twin's verified values, w/Warped_Forest).
pub const WARPED_STEM: u16 = 461;
/// 1.16: warped hyphae — the warped "bark" variant.
pub const WARPED_HYPHAE: u16 = 462;
/// 1.16: warped planks — hardness 2 / blast 3; 1:4 from stem or
/// hyphae.
pub const WARPED_PLANKS: u16 = 463;
/// 1.16: warped nylium — the warped-forest floor ("The floor of
/// the biome is composed mostly of warped nylium", VERIFIED
/// w/Warped_Forest). Drops netherrack when mined.
pub const WARPED_NYLIUM: u16 = 464;
/// 1.16: warped fungus — hardness 0 cross plant (VERIFIED
/// w/Warped_Fungus). "Fed to striders or planted to repel
/// hoglins" — both mechanics ride the mob layer.
pub const WARPED_FUNGUS: u16 = 465;
/// 1.16: warped roots — the warped-forest tuft (hardness 0,
/// transparent cross — VERIFIED w/Warped_Roots).
pub const WARPED_ROOTS: u16 = 466;
/// 1.16: twisting vines — the warped-forest climbing vine
/// ("upward-growing", VERIFIED w/Twisting_Vines), grows from the
/// warped ground; 1-in-3 self-drop chance (disclosed).
pub const TWISTING_VINES: u16 = 467;
/// 1.16: warped wart block — the huge-warped-fungus cap (hardness
/// 1 / blast 1, VERIFIED w/Warped_Wart_Block).
pub const WARPED_WART_BLOCK: u16 = 468;
/// 1.16: shroomlight — the huge-fungus lamp (hardness 1 / blast 1,
/// light 15, VERIFIED w/Shroomlight: "Luminous Yes (15)").
/// "Generate in huge fungi".
pub const SHROOMLIGHT: u16 = 469;
/// 1.16: nether sprouts — the warped-forest tuft (hardness 0,
/// transparent cross — VERIFIED w/Nether_Sprouts). Drops nothing
/// when broken without shears (the engine has no tool-gated drops
/// — matches vanilla's empty-handed result, disclosed).
pub const NETHER_SPROUTS: u16 = 470;
/// 1.16: polished basalt — hardness 1.25 / blast 4.2 (VERIFIED
/// w/Polished_Basalt); crafted 4:4 from basalt (the 2x2 family
/// that part 1's BASALT doc deferred here).
pub const POLISHED_BASALT: u16 = 471;
/// 1.16: polished blackstone — hardness 2 / blast 6 (VERIFIED
/// w/Polished_Blackstone); 4:4 from blackstone.
pub const POLISHED_BLACKSTONE: u16 = 472;
/// 1.16: polished blackstone bricks — hardness 1.5 / blast 6
/// (VERIFIED w/Polished_Blackstone_Bricks); 4:4 from polished
/// blackstone. Chiseled/cracked forms are the stonecutter family
/// — no stonecutter in the engine, disclosed.
pub const POLISHED_BLACKSTONE_BRICKS: u16 = 473;
/// 1.16: soul torch — light 10, hardness 0, any tool (VERIFIED
/// w/Soul_Torch: "Luminous Yes (10)"); the torch family's soul
/// variant ("crafted with the addition of soul soil or soul
/// sand"). "Soul torches repel piglins" — the mob-layer rule.
pub const SOUL_TORCH: u16 = 474;
/// 1.16: soul lantern — light 10, hardness 3.5 / blast 3.5
/// (VERIFIED w/Soul_Lantern); sits on tops or hangs from
/// undersides (the lantern/chain face-matched pair). Craft: 8 iron
/// nuggets + 1 soul torch.
pub const SOUL_LANTERN: u16 = 475;
/// 1.16: strider spawn egg (kind 42).
pub const SPAWN_EGG_STRIDER: u16 = 476;
/// 1.16: piglin spawn egg (kind 43).
pub const SPAWN_EGG_PIGLIN: u16 = 477;
/// 1.16: hoglin spawn egg (kind 44).
pub const SPAWN_EGG_HOGLIN: u16 = 478;

// ---- the 1.0-1.16.5 completeness audit: the V15 item window (ids
// 479..=504). Every value VERIFIED against the audit-round captures
// (scripts/audit16_page_*.json — the Food page's hunger table, the
// Potion page's brewing chart, the Ghast/Cave_Spider/Silverfish
// infoboxes, the Egg/Bowl/Sugar/Mushroom_Stew/Beetroot_Soup/
// Pumpkin_Pie/Poisonous_Potato/Apple/Rabbit_Stew/Popped_Chorus_Fruit
// pages). The round closes the engine's own recorded gaps: the
// standing cooked-meat deferral (campfire.rs), the 1.8 rabbit-stew
// deferral (its blockers — the rabbit's foot, the bowl — now exist),
// and the three never-implemented classic mobs. ----

/// steak (cooked beef) — smelting raw beef. "Steak ... 8" hunger
/// (VERIFIED w/Food: the hunger table's row). Restores 8/2 = 4 HP.
pub const STEAK: u16 = 479;
/// cooked porkchop — smelting raw porkchop. Hunger 8 (VERIFIED w/Food).
pub const COOKED_PORKCHOP: u16 = 480;
/// cooked chicken — smelting raw chicken. Hunger 6 (VERIFIED w/Food).
pub const COOKED_CHICKEN: u16 = 481;
/// cooked mutton — smelting raw mutton. Hunger 6 (VERIFIED w/Food).
pub const COOKED_MUTTON: u16 = 482;
/// cooked cod — smelting raw fish. Hunger 5 (VERIFIED w/Food).
pub const COOKED_COD: u16 = 483;
/// cooked salmon — smelting raw salmon. Hunger 6 (VERIFIED w/Food).
pub const COOKED_SALMON: u16 = 484;
/// apple — "Oak and dark oak leaves have a 0.5% (1/200) chance of
/// dropping an apple when decayed or broken" (VERIFIED w/Apple
/// §Block_loot). Hunger 4 (VERIFIED w/Food).
pub const APPLE: u16 = 485;
/// bowl — "Any Planks" craft (4; VERIFIED w/Bowl §Crafting) + the
/// fishing junk class (VERIFIED w/Fishing — the bowl row listed).
pub const BOWL: u16 = 486;
/// mushroom stew — "Red Mushroom + Brown Mushroom + Bowl" (VERIFIED
/// w/Mushroom_Stew §Crafting). Hunger 6 (VERIFIED w/Food).
pub const MUSHROOM_STEW: u16 = 487;
/// rabbit stew — "Cooked Rabbit + Carrot + Baked Potato + Red Mushroom
/// or Brown Mushroom + Bowl" (VERIFIED w/Rabbit_Stew §Crafting);
/// "Eating one restores 10 hunger and 12 hunger saturation" (VERIFIED
/// same page) — the engine's biggest single-food heal at 5 HP.
pub const RABBIT_STEW: u16 = 488;
/// beetroot — the 1.9 root crop (hunger 1, VERIFIED w/Food; the
/// engine's seeds item has existed since the 1.12 taming round).
pub const BEETROOT: u16 = 489;
/// beetroot soup — "Beetroot + Bowl" craft (VERIFIED
/// w/Beetroot_Soup §Crafting), "restore 6 hunger points" (same page).
pub const BEETROOT_SOUP: u16 = 490;
/// sugar — the honey-bottle craft (1.15) + the pumpkin-pie ingredient
/// (VERIFIED w/Pumpkin_Pie §Crafting: "Pumpkin + Sugar + Any Egg").
pub const SUGAR: u16 = 491;
/// egg — "Every adult chicken lays an egg item every 5-10 minutes ...
/// the theoretical average would be expected at 1 egg every 7.5
/// minutes (9000 game ticks)" (VERIFIED w/Egg).
pub const EGG: u16 = 492;
/// poisonous potato — "Eating one restores 2 hunger and 1.2 hunger
/// saturation and has a 60% chance of applying 5 seconds of Poison I"
/// (VERIFIED w/Poisonous_Potato).
pub const POISONOUS_POTATO: u16 = 493;
/// popped chorus fruit — "obtained by smelting chorus fruit that is
/// used to craft End rods and purpur blocks" (VERIFIED
/// w/Popped_Chorus_Fruit).
pub const POPPED_CHORUS_FRUIT: u16 = 494;
/// ghast tear — "Ghasts ... are the only source of ghast tears"
/// (VERIFIED w/Ghast); the regeneration brewing base.
pub const GHAST_TEAR: u16 = 495;
/// potion of leaping (1:30 base is the glowstone-enhanced form's
/// pre-1.9... no — VERIFIED w/Potion: rabbit's-foot brewing, base
/// 3:00 Jump Boost I; the II form 1:30 via glowstone).
pub const POTION_LEAPING: u16 = 496;
/// potion of leaping II — glowstone-enhanced (1:30, Jump Boost II).
pub const POTION_LEAPING_II: u16 = 497;
/// potion of leaping (extended 8:00) — registry row; brewing is
/// redstone-gated (the engine has no redstone-dust item, the disclosed
/// SLOW_FALLING_EXT convention).
pub const POTION_LEAPING_LONG: u16 = 498;
/// potion of regeneration (0:45, Regeneration I) — ghast tear base
/// (VERIFIED w/Potion: the ingredient chart lists Ghast Tear).
pub const POTION_REGEN: u16 = 499;
/// potion of regeneration II (0:22) — glowstone-enhanced.
pub const POTION_REGEN_II: u16 = 500;
/// potion of regeneration (extended 1:30) — registry row, redstone-
/// gated (disclosed, the LEAPING_LONG/SLOW_FALLING_EXT convention).
pub const POTION_REGEN_LONG: u16 = 501;
/// the ghast spawn egg (kind 45 — the classic Nether mob joins).
pub const SPAWN_EGG_GHAST: u16 = 502;
/// the cave-spider spawn egg (kind 46 — mineshaft spawner mob).
pub const SPAWN_EGG_CAVE_SPIDER: u16 = 503;
/// the silverfish spawn egg (kind 47 — stronghold spawner mob).
pub const SPAWN_EGG_SILVERFISH: u16 = 504;
/// melon slice — "Restores 2 hunger" (VERIFIED w/Melon_Slice, live
/// 2026-09-09); the 1.0 staple food, dropped by breaking melons.
pub const MELON_SLICE: u16 = 505;

pub const V10_STATE_BASE: u16 = 676;
// 13 states: bamboo stalk 1, shoot 1, berry bush 4 (age 0..3),
// campfire 2 (unlit/lit), barrel 1, then the 4 item states (berries,
// fox egg, stick, charcoal).
pub const V10_COUNT: u16 = 13;
/// V10 state -> block fold: the bush's 4 age states and the campfire's
/// 2 lit states fold to their parent blocks (per-state art rides
/// state_tiles).
pub const V10_STATE_TO_BLOCK: [u16; V10_COUNT as usize] = [
    BAMBOO,
    BAMBOO_SHOOT,
    // sweet berry bush: 4 age states
    SWEET_BERRY_BUSH, SWEET_BERRY_BUSH, SWEET_BERRY_BUSH, SWEET_BERRY_BUSH,
    // campfire: unlit, lit
    CAMPFIRE, CAMPFIRE,
    BARREL,
    // item states
    SWEET_BERRIES, SPAWN_EGG_FOX, STICK, CHARCOAL,
];

#[inline]
pub fn v10_state(b: u16) -> Option<u16> {
    match b {
        BAMBOO => Some(V10_STATE_BASE),
        BAMBOO_SHOOT => Some(V10_STATE_BASE + 1),
        SWEET_BERRY_BUSH => Some(V10_STATE_BASE + 2), // age 0
        // vanilla places campfires LIT (extinguished later by water)
        CAMPFIRE => Some(V10_STATE_BASE + 7),
        BARREL => Some(V10_STATE_BASE + 8),
        SWEET_BERRIES => Some(V10_STATE_BASE + 9),
        SPAWN_EGG_FOX => Some(V10_STATE_BASE + 10),
        STICK => Some(V10_STATE_BASE + 11),
        CHARCOAL => Some(V10_STATE_BASE + 12),
        _ => None,
    }
}

#[inline]
pub fn is_v10_state(s: u16) -> bool {
    (V10_STATE_BASE..V10_STATE_BASE + V10_COUNT).contains(&s)
}

pub const V11_STATE_BASE: u16 = 689;
// 9 states: blast furnace unlit/lit, smoker unlit/lit, lantern
// sitting/hanging, the iron-nugget item state, and the two 1.14
// flowers (one state each — no properties).
pub const V11_COUNT: u16 = 9;
/// V11 state -> block fold: the lit states fold to their parent
/// blocks (per-state art rides state_tiles; the lit swap is the
/// furnace pattern).
pub const V11_STATE_TO_BLOCK: [u16; V11_COUNT as usize] = [
    BLAST_FURNACE, BLAST_FURNACE,
    SMOKER, SMOKER,
    LANTERN, LANTERN,
    IRON_NUGGET,
    CORNFLOWER, LILY_OF_THE_VALLEY,
];

#[inline]
pub fn v11_state(b: u16) -> Option<u16> {
    match b {
        BLAST_FURNACE => Some(V11_STATE_BASE), // unlit
        SMOKER => Some(V11_STATE_BASE + 2),    // unlit
        LANTERN => Some(V11_STATE_BASE + 4),   // sitting
        IRON_NUGGET => Some(V11_STATE_BASE + 6),
        CORNFLOWER => Some(V11_STATE_BASE + 7),
        LILY_OF_THE_VALLEY => Some(V11_STATE_BASE + 8),
        _ => None,
    }
}

#[inline]
pub fn is_v11_state(s: u16) -> bool {
    (V11_STATE_BASE..V11_STATE_BASE + V11_COUNT).contains(&s)
}

// ---- 1.15 (Buzzy Bees): the V12 state window ----
pub const V12_STATE_BASE: u16 = 698;
// 18 states: bee nest honey_level 0..=5, beehive honey_level 0..=5,
// the honey + honeycomb-block identity states (block ids >= 432 must
// not ride identity states — they collide with the old F-series state
// ids; the same invariant every block >= 57 respects), then the 4
// item states (honeycomb, honey bottle, shears, bee egg).
pub const V12_COUNT: u16 = 18;
/// V12 state -> block fold: the 12 hive states fold to their parent
/// blocks; level 1..=4 share the level-0 art and only level 5 swaps
/// to the honey-oozing front tiles (state_tiles arm).
pub const V12_STATE_TO_BLOCK: [u16; V12_COUNT as usize] = [
    BEE_NEST, BEE_NEST, BEE_NEST, BEE_NEST, BEE_NEST, BEE_NEST,
    BEEHIVE, BEEHIVE, BEEHIVE, BEEHIVE, BEEHIVE, BEEHIVE,
    HONEY_BLOCK, HONEYCOMB_BLOCK,
    HONEYCOMB, HONEY_BOTTLE, SHEARS, SPAWN_EGG_BEE,
];

#[inline]
pub fn v12_state(b: u16) -> Option<u16> {
    match b {
        BEE_NEST => Some(V12_STATE_BASE),          // honey_level 0
        BEEHIVE => Some(V12_STATE_BASE + 6),       // honey_level 0
        HONEY_BLOCK => Some(V12_STATE_BASE + 12),
        HONEYCOMB_BLOCK => Some(V12_STATE_BASE + 13),
        HONEYCOMB => Some(V12_STATE_BASE + 14),
        HONEY_BOTTLE => Some(V12_STATE_BASE + 15),
        SHEARS => Some(V12_STATE_BASE + 16),
        SPAWN_EGG_BEE => Some(V12_STATE_BASE + 17),
        _ => None,
    }
}

#[inline]
pub fn is_v12_state(s: u16) -> bool {
    (V12_STATE_BASE..V12_STATE_BASE + V12_COUNT).contains(&s)
}

/// 1.15: the honey_level 0..=5 stored in a nest/hive state (0 for
/// non-hive states). VERIFIED w/Beehive: "Every pollinated bee that
/// leaves the hive after working increases the honey level by one.
/// When at level 5, honey can be bottled or honeycombs can be
/// harvested."
#[inline]
pub fn honey_level(s: u16) -> u8 {
    if is_v12_state(s) && (s as u16) < V12_STATE_BASE + 12 {
        let off = s - V12_STATE_BASE;
        if off >= 6 {
            (off - 6) as u8
        } else {
            off as u8
        }
    } else {
        0
    }
}

/// state id for a nest/hive block at a given honey level (0..=5).
#[inline]
pub fn hive_state(b: u16, level: u8) -> u16 {
    let base = if b == BEEHIVE {
        V12_STATE_BASE + 6
    } else {
        V12_STATE_BASE
    };
    base + (level.min(5)) as u16
}

/// 1.15: is this state a honey_level-5 hive (the dripping/harvest
/// form)?
#[inline]
pub fn hive_full(s: u16) -> bool {
    honey_level(s) == 5 && is_v12_state(s) && (s as u16) < V12_STATE_BASE + 12
}

// ---- 1.16 (Nether Update, PART 1): the V13 state window ----
pub const V13_STATE_BASE: u16 = 716;
// 34 states: respawn anchor charges 0..=4 (5), target power 0..=15
// (16), then the identity states (soul soil, basalt, blackstone,
// gilded blackstone, crying obsidian, nether gold ore, ancient
// debris, netherite block, chain sitting/hanging, soul fire, and the
// 2 item rows). Every block >= 440 gets an explicit state slot (the
// standing invariant since id 57: block ids never double as state
// ids past the flat registry).
pub const V13_COUNT: u16 = 34;
/// V13 state -> block fold: the 5 anchor charge states and the 16
/// target power states fold to their parents; the chain's hanging
/// state folds like the lantern's.
pub const V13_STATE_TO_BLOCK: [u16; V13_COUNT as usize] = [
    RESPAWN_ANCHOR, RESPAWN_ANCHOR, RESPAWN_ANCHOR, RESPAWN_ANCHOR, RESPAWN_ANCHOR,
    TARGET, TARGET, TARGET, TARGET, TARGET, TARGET, TARGET, TARGET,
    TARGET, TARGET, TARGET, TARGET, TARGET, TARGET, TARGET, TARGET,
    SOUL_SOIL,
    BASALT,
    BLACKSTONE,
    GILDED_BLACKSTONE,
    CRYING_OBSIDIAN,
    NETHER_GOLD_ORE,
    ANCIENT_DEBRIS,
    NETHERITE_BLOCK,
    CHAIN,
    CHAIN,
    SOUL_FIRE,
    NETHERITE_SCRAP,
    NETHERITE_INGOT,
];

#[inline]
pub fn v13_state(b: u16) -> Option<u16> {
    match b {
        RESPAWN_ANCHOR => Some(V13_STATE_BASE),          // charge 0
        TARGET => Some(V13_STATE_BASE + 5),              // power 0
        SOUL_SOIL => Some(V13_STATE_BASE + 21),
        BASALT => Some(V13_STATE_BASE + 22),
        BLACKSTONE => Some(V13_STATE_BASE + 23),
        GILDED_BLACKSTONE => Some(V13_STATE_BASE + 24),
        CRYING_OBSIDIAN => Some(V13_STATE_BASE + 25),
        NETHER_GOLD_ORE => Some(V13_STATE_BASE + 26),
        ANCIENT_DEBRIS => Some(V13_STATE_BASE + 27),
        NETHERITE_BLOCK => Some(V13_STATE_BASE + 28),
        CHAIN => Some(V13_STATE_BASE + 29),              // sitting
        SOUL_FIRE => Some(V13_STATE_BASE + 31),
        NETHERITE_SCRAP => Some(V13_STATE_BASE + 32),
        NETHERITE_INGOT => Some(V13_STATE_BASE + 33),
        _ => None,
    }
}

#[inline]
pub fn is_v13_state(s: u16) -> bool {
    (V13_STATE_BASE..V13_STATE_BASE + V13_COUNT).contains(&s)
}

/// 1.16: the respawn anchor's charge 0..=4 stored in its state
/// ("fueled with glowstone blocks" — each glowstone adds one, max 4,
/// VERIFIED w/Respawn_Anchor).
#[inline]
pub fn anchor_charge(s: u16) -> u8 {
    if is_v13_state(s) && s < V13_STATE_BASE + 5 {
        (s - V13_STATE_BASE) as u8
    } else {
        0
    }
}

/// state id for a respawn anchor at a given charge (clamped 0..=4).
#[inline]
pub fn anchor_state(charge: u8) -> u16 {
    V13_STATE_BASE + (charge.min(4)) as u16
}

/// 1.16: the anchor's light by charge — "the respawn anchor glow with
/// a light level of 3. Each glowstone after the first increases the
/// light level by 4, up to a maximum of 15" (VERIFIED w/Respawn_Anchor
/// — charges 1/2/3/4 → light 3/7/11/15, the infobox row).
#[inline]
pub fn anchor_light(s: u16) -> u8 {
    match anchor_charge(s) {
        0 => 0,
        1 => 3,
        2 => 7,
        3 => 11,
        _ => 15,
    }
}

/// 1.16: the target's power 0..=15 stored in its state (the JE
/// blockstate — hit strength by center proximity, VERIFIED w/Target).
#[inline]
pub fn target_power(s: u16) -> u8 {
    if is_v13_state(s) && (V13_STATE_BASE + 5..V13_STATE_BASE + 21).contains(&s) {
        (s - (V13_STATE_BASE + 5)) as u8
    } else {
        0
    }
}

/// state id for a target at a given power (clamped 0..=15).
#[inline]
pub fn target_state(power: u8) -> u16 {
    V13_STATE_BASE + 5 + (power.min(15)) as u16
}

/// 1.16: is this V13 state the HANGING chain form (the lantern's
/// face-matched pattern)?
#[inline]
pub fn chain_hanging(s: u16) -> bool {
    s == V13_STATE_BASE + 30
}

/// 1.16: is this state the soul fire block? (the engine's first fire
/// block — non-solid, non-opaque, cross-sprite, light 10.)
#[inline]
pub fn is_soul_fire(s: u16) -> bool {
    s == V13_STATE_BASE + 31 || state_block(s) == SOUL_FIRE
}

// ---- 1.16 (Nether Update, PART 2): the V14 state window ----
pub const V14_STATE_BASE: u16 = 750;
// 26 states: the 22 crimson/warped family blocks as identity states
// (the soul lantern carries its sitting/hanging pair — the chain
// convention), plus the 3 spawn-egg item rows. Every block >= 454
// gets an explicit state slot (the standing invariant since id 57:
// block ids never double as state ids past the flat registry).
pub const V14_COUNT: u16 = 26;
/// V14 state -> block fold: all identity except the soul lantern's
/// hanging state, which folds like the lantern/chain's.
pub const V14_STATE_TO_BLOCK: [u16; V14_COUNT as usize] = [
    CRIMSON_STEM,
    CRIMSON_HYPHAE,
    CRIMSON_PLANKS,
    CRIMSON_NYLIUM,
    CRIMSON_FUNGUS,
    CRIMSON_ROOTS,
    WEEPING_VINES,
    WARPED_STEM,
    WARPED_HYPHAE,
    WARPED_PLANKS,
    WARPED_NYLIUM,
    WARPED_FUNGUS,
    WARPED_ROOTS,
    TWISTING_VINES,
    WARPED_WART_BLOCK,
    SHROOMLIGHT,
    NETHER_SPROUTS,
    POLISHED_BASALT,
    POLISHED_BLACKSTONE,
    POLISHED_BLACKSTONE_BRICKS,
    SOUL_TORCH,
    SOUL_LANTERN,
    SOUL_LANTERN, // hanging (the lantern/chain pair)
    SPAWN_EGG_STRIDER,
    SPAWN_EGG_PIGLIN,
    SPAWN_EGG_HOGLIN,
];

#[inline]
pub fn v14_state(b: u16) -> Option<u16> {
    match b {
        CRIMSON_STEM => Some(V14_STATE_BASE),
        CRIMSON_HYPHAE => Some(V14_STATE_BASE + 1),
        CRIMSON_PLANKS => Some(V14_STATE_BASE + 2),
        CRIMSON_NYLIUM => Some(V14_STATE_BASE + 3),
        CRIMSON_FUNGUS => Some(V14_STATE_BASE + 4),
        CRIMSON_ROOTS => Some(V14_STATE_BASE + 5),
        WEEPING_VINES => Some(V14_STATE_BASE + 6),
        WARPED_STEM => Some(V14_STATE_BASE + 7),
        WARPED_HYPHAE => Some(V14_STATE_BASE + 8),
        WARPED_PLANKS => Some(V14_STATE_BASE + 9),
        WARPED_NYLIUM => Some(V14_STATE_BASE + 10),
        WARPED_FUNGUS => Some(V14_STATE_BASE + 11),
        WARPED_ROOTS => Some(V14_STATE_BASE + 12),
        TWISTING_VINES => Some(V14_STATE_BASE + 13),
        WARPED_WART_BLOCK => Some(V14_STATE_BASE + 14),
        SHROOMLIGHT => Some(V14_STATE_BASE + 15),
        NETHER_SPROUTS => Some(V14_STATE_BASE + 16),
        POLISHED_BASALT => Some(V14_STATE_BASE + 17),
        POLISHED_BLACKSTONE => Some(V14_STATE_BASE + 18),
        POLISHED_BLACKSTONE_BRICKS => Some(V14_STATE_BASE + 19),
        SOUL_TORCH => Some(V14_STATE_BASE + 20),
        SOUL_LANTERN => Some(V14_STATE_BASE + 21), // sitting
        SPAWN_EGG_STRIDER => Some(V14_STATE_BASE + 23),
        SPAWN_EGG_PIGLIN => Some(V14_STATE_BASE + 24),
        SPAWN_EGG_HOGLIN => Some(V14_STATE_BASE + 25),
        _ => None,
    }
}

#[inline]
pub fn is_v14_state(s: u16) -> bool {
    (V14_STATE_BASE..V14_STATE_BASE + V14_COUNT).contains(&s)
}

// ---- the 1.0-1.16.5 completeness audit: the V15 state window ----
// 28 states: the 26 V15 item rows as identity states, plus the two
// new spawner states (the cave-spider / silverfish codes — the
// dedicated-state pattern of SPAWNER_BLAZE 241, riding the window).
pub const V15_STATE_BASE: u16 = 776;
pub const V15_COUNT: u16 = 29;
/// V15 state -> block fold: 26 identity item folds + the two spawner
/// states folding to the Monster Spawner block.
pub const V15_STATE_TO_BLOCK: [u16; V15_COUNT as usize] = [
    STEAK,
    COOKED_PORKCHOP,
    COOKED_CHICKEN,
    COOKED_MUTTON,
    COOKED_COD,
    COOKED_SALMON,
    APPLE,
    BOWL,
    MUSHROOM_STEW,
    RABBIT_STEW,
    BEETROOT,
    BEETROOT_SOUP,
    SUGAR,
    EGG,
    POISONOUS_POTATO,
    POPPED_CHORUS_FRUIT,
    GHAST_TEAR,
    POTION_LEAPING,
    POTION_LEAPING_II,
    POTION_LEAPING_LONG,
    POTION_REGEN,
    POTION_REGEN_II,
    POTION_REGEN_LONG,
    SPAWN_EGG_GHAST,
    SPAWN_EGG_CAVE_SPIDER,
    SPAWN_EGG_SILVERFISH,
    SPAWNER, // the cave-spider spawner state
    SPAWNER, // the silverfish spawner state
    // the sweep-2 row: the melon slice identity state (804)
    MELON_SLICE,
];

#[inline]
pub fn v15_state(b: u16) -> Option<u16> {
    match b {
        STEAK => Some(V15_STATE_BASE),
        COOKED_PORKCHOP => Some(V15_STATE_BASE + 1),
        COOKED_CHICKEN => Some(V15_STATE_BASE + 2),
        COOKED_MUTTON => Some(V15_STATE_BASE + 3),
        COOKED_COD => Some(V15_STATE_BASE + 4),
        COOKED_SALMON => Some(V15_STATE_BASE + 5),
        APPLE => Some(V15_STATE_BASE + 6),
        BOWL => Some(V15_STATE_BASE + 7),
        MUSHROOM_STEW => Some(V15_STATE_BASE + 8),
        RABBIT_STEW => Some(V15_STATE_BASE + 9),
        BEETROOT => Some(V15_STATE_BASE + 10),
        BEETROOT_SOUP => Some(V15_STATE_BASE + 11),
        SUGAR => Some(V15_STATE_BASE + 12),
        EGG => Some(V15_STATE_BASE + 13),
        POISONOUS_POTATO => Some(V15_STATE_BASE + 14),
        POPPED_CHORUS_FRUIT => Some(V15_STATE_BASE + 15),
        GHAST_TEAR => Some(V15_STATE_BASE + 16),
        POTION_LEAPING => Some(V15_STATE_BASE + 17),
        POTION_LEAPING_II => Some(V15_STATE_BASE + 18),
        POTION_LEAPING_LONG => Some(V15_STATE_BASE + 19),
        POTION_REGEN => Some(V15_STATE_BASE + 20),
        POTION_REGEN_II => Some(V15_STATE_BASE + 21),
        POTION_REGEN_LONG => Some(V15_STATE_BASE + 22),
        SPAWN_EGG_GHAST => Some(V15_STATE_BASE + 23),
        SPAWN_EGG_CAVE_SPIDER => Some(V15_STATE_BASE + 24),
        SPAWN_EGG_SILVERFISH => Some(V15_STATE_BASE + 25),
        MELON_SLICE => Some(V15_STATE_BASE + 28),
        // ---- backlog round (V16 window) ----
        FIRE => Some(V16_STATE_BASE),
        _ => None,
    }
}

#[inline]
pub fn is_v15_state(s: u16) -> bool {
    (V15_STATE_BASE..V15_STATE_BASE + V15_COUNT).contains(&s)
}

/// Backlog round (2026-09-09): the V16 state window — the weather/
/// farming/door/bed/TNT/jukebox brackets' state allocations.
pub const V16_STATE_BASE: u16 = 805;
pub const V16_COUNT: u16 = 1;
/// V16 state -> block fold: index = state − V16_STATE_BASE.
pub const V16_STATE_TO_BLOCK: [u16; V16_COUNT as usize] = [
    FIRE, // 805 — the fire block's identity state
];

#[inline]
pub fn is_v16_state(s: u16) -> bool {
    (V16_STATE_BASE..V16_STATE_BASE + V16_COUNT).contains(&s)
}

/// the 1.0-1.16.5 audit's cave-spider spawner state (kind code 7 —
/// the mineshaft spawner's mob; replaces the pre-audit spider-spawner
/// adaptation that gen.rs documented as "no distinct cave-spider mob").
pub const SPAWNER_CAVESPIDER: u16 = V15_STATE_BASE + 26;
/// the audit's silverfish spawner state (kind code 8 — the stronghold
/// portal-room spawner's mob).
pub const SPAWNER_SILVERFISH: u16 = V15_STATE_BASE + 27;

/// 1.16: is this V14 state the HANGING soul-lantern form (the
/// lantern/chain face-matched pattern)?
#[inline]
pub fn soul_lantern_hanging(s: u16) -> bool {
    s == V14_STATE_BASE + 22
}

/// 1.16: is this state (or block) one of the nether-forest CROSS
/// plants? (fungi / roots / sprouts — the tall-grass class: non-solid,
/// needs a floor for placement. Vines are NOT forest plants — they
/// hang/climb instead of rooting.)
#[inline]
pub fn is_forest_plant(s: u16) -> bool {
    // V14 states decode through the window; a raw V14 block id is
    // self-describing (these ids are new — no other window folds to
    // them, so both spellings are unambiguous)
    let b = if is_v14_state(s) {
        V14_STATE_TO_BLOCK[(s - V14_STATE_BASE) as usize]
    } else {
        s
    };
    matches!(b, CRIMSON_FUNGUS | CRIMSON_ROOTS | NETHER_SPROUTS | WARPED_FUNGUS | WARPED_ROOTS)
}

/// 1.14: is this V11 state a LIT smelter (blast furnace / smoker)?
/// (the lit arm of the block swap — mirrors campfire_lit)
#[inline]
pub fn v11_smelter_lit(s: u16) -> bool {
    is_v11_state(s)
        && (s == V11_STATE_BASE + 1 || s == V11_STATE_BASE + 3)
}

/// 1.14: is this V11 state the HANGING lantern form?
#[inline]
pub fn lantern_hanging(s: u16) -> bool {
    s == V11_STATE_BASE + 5
}

/// sweet berry bush growth age (0..3) from its storage state.
#[inline]
pub fn berry_bush_age(s: u16) -> u8 {
    if (V10_STATE_BASE + 2..=V10_STATE_BASE + 5).contains(&s) {
        (s - (V10_STATE_BASE + 2)) as u8
    } else {
        0
    }
}

/// state for a berry bush at a given age (clamped 0..3).
#[inline]
pub fn berry_bush_state(age: u8) -> u16 {
    V10_STATE_BASE + 2 + (age.min(3)) as u16
}

/// is the campfire storage state the LIT one?
#[inline]
pub fn campfire_lit(s: u16) -> bool {
    s == V10_STATE_BASE + 7
}

/// state for a campfire (lit or extinguished).
#[inline]
pub fn campfire_state(lit: bool) -> u16 {
    V10_STATE_BASE + 6 + if lit { 1 } else { 0 }
}

/// the 5 coral colors (tube/brain/bubble/fire/horn — the changelog's
/// own list; VERIFIED w/Coral_Block).
pub const CORAL_NAMES: [&str; 5] = ["Tube", "Brain", "Bubble", "Fire", "Horn"];

/// coral color index → clean-room RGB base (w/Coral variants: blue /
/// pink / purple / red / yellow).
pub const CORAL_RGB: [[i32; 3]; 5] = [
    [45, 118, 205],  // tube — blue
    [212, 118, 151], // brain — pink
    [160, 74, 183],  // bubble — purple
    [196, 76, 61],   // fire — red
    [209, 178, 60],  // horn — yellow
];

/// is this a live (water-dependent) coral block id — the death rule
/// applies (VERIFIED w/Coral_Block: "Turns into a dead coral block if
/// none of its six sides are touching the water, although not
/// instantly" — the random-tick delay).
#[inline]
pub fn is_live_coral_block(b: u16) -> bool {
    (CORAL_BLOCK_BASE..=CORAL_BLOCK_END).contains(&b)
}

/// coral block → its dead counterpart (same color).
#[inline]
pub fn dead_coral_of(b: u16) -> u16 {
    if is_live_coral_block(b) {
        DEAD_CORAL_BLOCK_BASE + (b - CORAL_BLOCK_BASE)
    } else if (CORAL_PLANT_BASE..=CORAL_PLANT_END).contains(&b) {
        DEAD_CORAL_PLANT_BASE + (b - CORAL_PLANT_BASE)
    } else if (CORAL_FAN_BASE..=CORAL_FAN_END).contains(&b) {
        DEAD_CORAL_FAN_BASE + (b - CORAL_FAN_BASE)
    } else {
        b
    }
}

pub const V8_STATE_BASE: u16 = 497;
// 70 ids: 291..=360 (16 concrete + 16 powder + 16 glazed + parrot egg
// + 16 dyes + 4 seeds + cookie). States: concrete 497..=512, powder
// 513..=528, glazed 529..=592 (4 facings per color), parrot egg 593,
// dyes 594..=609, seeds 610..=613, cookie 614 (STATE_COUNT = 615).
pub const V8_COUNT: u16 = 118;
/// V8 state → block fold table. Index = state − V8_STATE_BASE. Glazed
/// states occupy 64 consecutive entries (color*4 + facing) — the
/// reverse mapping is `glazed_terracotta_state`/`glazed_decode`.
pub const V8_STATE_TO_BLOCK: [u16; V8_COUNT as usize] = [
    // concrete (16)
    CONCRETE_BASE, CONCRETE_BASE + 1, CONCRETE_BASE + 2, CONCRETE_BASE + 3,
    CONCRETE_BASE + 4, CONCRETE_BASE + 5, CONCRETE_BASE + 6, CONCRETE_BASE + 7,
    CONCRETE_BASE + 8, CONCRETE_BASE + 9, CONCRETE_BASE + 10, CONCRETE_BASE + 11,
    CONCRETE_BASE + 12, CONCRETE_BASE + 13, CONCRETE_BASE + 14, CONCRETE_BASE + 15,
    // concrete powder (16)
    CONCRETE_POWDER_BASE, CONCRETE_POWDER_BASE + 1, CONCRETE_POWDER_BASE + 2,
    CONCRETE_POWDER_BASE + 3, CONCRETE_POWDER_BASE + 4, CONCRETE_POWDER_BASE + 5,
    CONCRETE_POWDER_BASE + 6, CONCRETE_POWDER_BASE + 7, CONCRETE_POWDER_BASE + 8,
    CONCRETE_POWDER_BASE + 9, CONCRETE_POWDER_BASE + 10, CONCRETE_POWDER_BASE + 11,
    CONCRETE_POWDER_BASE + 12, CONCRETE_POWDER_BASE + 13, CONCRETE_POWDER_BASE + 14,
    CONCRETE_POWDER_BASE + 15,
    // glazed terracotta (64: color 0..15 × facing 0..3)
    GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_BASE,
    GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_BASE + 1, GLAZED_TERRACOTTA_BASE + 1,
    GLAZED_TERRACOTTA_BASE + 1, GLAZED_TERRACOTTA_BASE + 1,
    GLAZED_TERRACOTTA_BASE + 2, GLAZED_TERRACOTTA_BASE + 2, GLAZED_TERRACOTTA_BASE + 2,
    GLAZED_TERRACOTTA_BASE + 2, GLAZED_TERRACOTTA_BASE + 3, GLAZED_TERRACOTTA_BASE + 3,
    GLAZED_TERRACOTTA_BASE + 3, GLAZED_TERRACOTTA_BASE + 3,
    GLAZED_TERRACOTTA_BASE + 4, GLAZED_TERRACOTTA_BASE + 4, GLAZED_TERRACOTTA_BASE + 4,
    GLAZED_TERRACOTTA_BASE + 4, GLAZED_TERRACOTTA_BASE + 5, GLAZED_TERRACOTTA_BASE + 5,
    GLAZED_TERRACOTTA_BASE + 5, GLAZED_TERRACOTTA_BASE + 5,
    GLAZED_TERRACOTTA_BASE + 6, GLAZED_TERRACOTTA_BASE + 6, GLAZED_TERRACOTTA_BASE + 6,
    GLAZED_TERRACOTTA_BASE + 6, GLAZED_TERRACOTTA_BASE + 7, GLAZED_TERRACOTTA_BASE + 7,
    GLAZED_TERRACOTTA_BASE + 7, GLAZED_TERRACOTTA_BASE + 7,
    GLAZED_TERRACOTTA_BASE + 8, GLAZED_TERRACOTTA_BASE + 8, GLAZED_TERRACOTTA_BASE + 8,
    GLAZED_TERRACOTTA_BASE + 8, GLAZED_TERRACOTTA_BASE + 9, GLAZED_TERRACOTTA_BASE + 9,
    GLAZED_TERRACOTTA_BASE + 9, GLAZED_TERRACOTTA_BASE + 9,
    GLAZED_TERRACOTTA_BASE + 10, GLAZED_TERRACOTTA_BASE + 10, GLAZED_TERRACOTTA_BASE + 10,
    GLAZED_TERRACOTTA_BASE + 10, GLAZED_TERRACOTTA_BASE + 11, GLAZED_TERRACOTTA_BASE + 11,
    GLAZED_TERRACOTTA_BASE + 11, GLAZED_TERRACOTTA_BASE + 11,
    GLAZED_TERRACOTTA_BASE + 12, GLAZED_TERRACOTTA_BASE + 12, GLAZED_TERRACOTTA_BASE + 12,
    GLAZED_TERRACOTTA_BASE + 12, GLAZED_TERRACOTTA_BASE + 13, GLAZED_TERRACOTTA_BASE + 13,
    GLAZED_TERRACOTTA_BASE + 13, GLAZED_TERRACOTTA_BASE + 13,
    GLAZED_TERRACOTTA_BASE + 14, GLAZED_TERRACOTTA_BASE + 14, GLAZED_TERRACOTTA_BASE + 14,
    GLAZED_TERRACOTTA_BASE + 14, GLAZED_TERRACOTTA_BASE + 15, GLAZED_TERRACOTTA_BASE + 15,
    GLAZED_TERRACOTTA_BASE + 15, GLAZED_TERRACOTTA_BASE + 15,
    // parrot egg (1)
    SPAWN_EGG_PARROT,
    // dyes (16)
    DYE_BASE, DYE_BASE + 1, DYE_BASE + 2, DYE_BASE + 3, DYE_BASE + 4,
    DYE_BASE + 5, DYE_BASE + 6, DYE_BASE + 7, DYE_BASE + 8, DYE_BASE + 9,
    DYE_BASE + 10, DYE_BASE + 11, DYE_BASE + 12, DYE_BASE + 13, DYE_BASE + 14,
    DYE_BASE + 15,
    // seeds (4)
    WHEAT_SEEDS, MELON_SEEDS, PUMPKIN_SEEDS, BEETROOT_SEEDS,
    // cookie (1)
    COOKIE,
];

/// default V8 state of a block id (None outside the 1.12 window):
/// concrete/powder are 1:1 (offsets 0..=31), glazed terracotta maps to
/// its facing-0 state (4 states per color), and the egg/dyes/seeds/
/// cookie items are 1:1 at offset 96+.
#[inline]
pub fn v8_state(b: u16) -> Option<u16> {
    if (GLAZED_TERRACOTTA_BASE..=GLAZED_TERRACOTTA_END).contains(&b) {
        Some(glazed_terracotta_state((b - GLAZED_TERRACOTTA_BASE) as u8, 0))
    } else if (CONCRETE_BASE..=CONCRETE_POWDER_END).contains(&b) {
        Some(V8_STATE_BASE + (b - CONCRETE_BASE) as u16)
    } else if (SPAWN_EGG_PARROT..=COOKIE).contains(&b) {
        Some(V8_STATE_BASE + 96 + (b - SPAWN_EGG_PARROT) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v8_state(s: u16) -> bool {
    (V8_STATE_BASE..V8_STATE_BASE + V8_COUNT).contains(&s)
}

// ---------------------------------------------------- 1.12 codecs --

/// concrete block id for a color index (0..15, engine color order).
#[inline]
pub fn concrete(color: u8) -> u16 {
    CONCRETE_BASE + color.min(15) as u16
}

/// concrete state id for a color index.
#[inline]
pub fn concrete_state(color: u8) -> u16 {
    V8_STATE_BASE + color.min(15) as u16
}

/// color index of a concrete block (255 = not one).
#[inline]
pub fn concrete_color(b: u16) -> u8 {
    if (CONCRETE_BASE..=CONCRETE_END).contains(&b) {
        (b - CONCRETE_BASE) as u8
    } else {
        255
    }
}

/// concrete-powder block id for a color index.
#[inline]
pub fn concrete_powder(color: u8) -> u16 {
    CONCRETE_POWDER_BASE + color.min(15) as u16
}

/// concrete-powder state id for a color index.
#[inline]
pub fn concrete_powder_state(color: u8) -> u16 {
    V8_STATE_BASE + 16 + color.min(15) as u16
}

/// color index of a concrete-powder block (255 = not one).
#[inline]
pub fn concrete_powder_color(b: u16) -> u8 {
    if (CONCRETE_POWDER_BASE..=CONCRETE_POWDER_END).contains(&b) {
        (b - CONCRETE_POWDER_BASE) as u8
    } else {
        255
    }
}

/// glazed-terracotta block id for a color index.
#[inline]
pub fn glazed_terracotta(color: u8) -> u16 {
    GLAZED_TERRACOTTA_BASE + color.min(15) as u16
}

/// glazed-terracotta state for (color, facing). Facing 0..3 =
/// north/east/south/west (the engine's horizontal-facing convention —
/// repeater/observer/dispenser). "Can be placed in 4 directions:
/// north, south, west, and east" (VERIFIED changelog §Blocks).
#[inline]
pub fn glazed_terracotta_state(color: u8, facing: u8) -> u16 {
    V8_STATE_BASE + 32 + (color.min(15) as u16) * 4 + (facing.min(3) as u16)
}

/// decode a glazed-terracotta state → (color, facing);
/// None if the state is not in the glazed window.
#[inline]
pub fn glazed_decode(s: u16) -> Option<(u8, u8)> {
    let lo = V8_STATE_BASE + 32;
    if (lo..lo + 64).contains(&s) {
        let off = s - lo;
        Some(((off / 4) as u8, (off % 4) as u8))
    } else {
        None
    }
}

/// dye item id for a color index (0..15, engine color order).
#[inline]
pub fn dye(color: u8) -> u16 {
    DYE_BASE + color.min(15) as u16
}

/// color index of a dye item (255 = not one).
#[inline]
pub fn dye_color(b: u16) -> u8 {
    if (DYE_BASE..=DYE_END).contains(&b) {
        (b - DYE_BASE) as u8
    } else {
        255
    }
}

/// true if the block is any concrete-powder color (the gravity set
/// membership test used by the sim layer).
#[inline]
pub fn is_concrete_powder(b: u16) -> bool {
    (CONCRETE_POWDER_BASE..=CONCRETE_POWDER_END).contains(&b)
}

/// true if the block is any concrete color.
#[inline]
pub fn is_concrete(b: u16) -> bool {
    (CONCRETE_BASE..=CONCRETE_END).contains(&b)
}

/// true if the block is any glazed-terracotta color.
#[inline]
pub fn is_glazed_terracotta(b: u16) -> bool {
    (GLAZED_TERRACOTTA_BASE..=GLAZED_TERRACOTTA_END).contains(&b)
}

/// true if the block is any of the 4 parrot-taming seed items.
#[inline]
pub fn is_seeds(b: u16) -> bool {
    (WHEAT_SEEDS..=BEETROOT_SEEDS).contains(&b)
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

pub const BLOCK_COUNT: usize = 507; // + the backlog round's Fire (506 — lightning ignition + flint-and-steel source)
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
pub const STATE_COUNT: usize = 806; // the backlog round's V16 window: 805 = fire (V15 states 776..=804)
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
    } else if s == SPAWNER_BLAZE {
        3 // fortress blaze (code 3 — fixes a LATENT Phase-E1 bug: the
          // dedicated state 241 fell through to the zombie code and
          // fortress spawners registered as zombie spawners)
    } else if s == SPAWNER_WITHER_SKELETON {
        4 // fortress platform 2 (same latent-bug fix)
    } else if s == SPAWNER_VINDICATOR {
        5 // 1.11 mansion
    } else if s == SPAWNER_EVOKER {
        6 // 1.11 mansion upper floors
    } else if s == SPAWNER_CAVESPIDER {
        7 // the completeness audit: the mineshaft spawner's own mob
          // (replaces the spider-spawner adaptation gen.rs disclosed)
    } else if s == SPAWNER_SILVERFISH {
        8 // the audit: the stronghold portal-room spawner's mob
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
        b if (282..282 + V7_COUNT as u16).contains(&b) => {
            V7_STATE_BASE + (b - 282) as u16
        }
        // 1.12 (World of Color Update): concrete/powder/egg/dyes/seeds/
        // cookie are 1:1; glazed terracotta defaults to facing 0
        // (north — the placement path writes the player-facing state)
        b if v8_state(b).is_some() => v8_state(b).unwrap(),
        // 1.13 (Update Aquatic): the V9 window — 1:1 defaults; the sea
        // pickle places with 1 pickle, the turtle egg at stage 0
        b if v9_state(b).is_some() => v9_state(b).unwrap(),
        // 1.14 (Village & Pillage): the V10 window — the berry bush
        // places at age 0; campfires place LIT (extinguished later);
        // placement of a BAMBOO item on soil routes to the SHOOT form
        // via the game layer's plant branch
        b if v10_state(b).is_some() => v10_state(b).unwrap(),
        // 1.14 (part 2): the V11 window — the smelters place UNLIT
        // (the furnace convention; the lit swap rides the burn state),
        // the lantern places SITTING (the placement path writes the
        // hanging state on underside clicks)
        b if v11_state(b).is_some() => v11_state(b).unwrap(),
        // 1.15: nests/hives place at honey_level 0; the V12 items ride
        // their item states
        b if v12_state(b).is_some() => v12_state(b).unwrap(),
        // 1.16 (Nether Update, part 1): the V13 window — the anchor
        // places at charge 0, the target at power 0, the chain SITTING
        // (the lantern convention; the placement path writes the
        // hanging state on underside clicks), the rest identity
        b if v13_state(b).is_some() => v13_state(b).unwrap(),
        // 1.16 (Nether Update, part 2): the V14 window — the soul
        // lantern places SITTING (the chain convention; the placement
        // path writes the hanging state on underside clicks), the
        // rest identity
        b if v14_state(b).is_some() => v14_state(b).unwrap(),
        // the completeness audit: V15 identity item states
        b if v15_state(b).is_some() => v15_state(b).unwrap(),
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
        // 1.11 mansion illager spawner states
        SPAWNER_VINDICATOR => return SPAWNER,
        SPAWNER_EVOKER => return SPAWNER,
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
        s if is_v7_state(s) => {
            return V7_STATE_TO_BLOCK[(s - V7_STATE_BASE) as usize];
        }
        // 1.12 (World of Color Update): the V8 window — 1:1 for
        // concrete/powder/egg/dyes/seeds/cookie; glazed terracotta's
        // 64 facing states fold to their color's block
        s if is_v8_state(s) => {
            return V8_STATE_TO_BLOCK[(s - V8_STATE_BASE) as usize];
        }
        // 1.13 (Update Aquatic): the V9 window — 1:1 folds; the pickle's
        // count states and the egg's hatch states fold to their parent
        s if is_v9_state(s) => {
            return V9_STATE_TO_BLOCK[(s - V9_STATE_BASE) as usize];
        }
        // 1.14 (Village & Pillage nature half): the V10 window — the
        // bush's 4 age states and the campfire's 2 lit states fold to
        // their parent
        s if is_v10_state(s) => {
            return V10_STATE_TO_BLOCK[(s - V10_STATE_BASE) as usize];
        }
        // 1.14 (nature half, part 2): the V11 window — the smelters'
        // lit states and the lantern's hanging state fold to their
        // parent blocks
        s if is_v11_state(s) => {
            return V11_STATE_TO_BLOCK[(s - V11_STATE_BASE) as usize];
        }
        // 1.15 (Buzzy Bees): the V12 window — the nest/hive
        // honey_level states fold to their parent blocks; the item
        // states fold to their item ids
        s if is_v12_state(s) => {
            return V12_STATE_TO_BLOCK[(s - V12_STATE_BASE) as usize];
        }
        // 1.16 (Nether Update, part 1): the V13 window — the anchor's
        // 5 charge states and the target's 16 power states fold to
        // their parents; the chain's hanging state folds like the
        // lantern's
        s if is_v13_state(s) => {
            return V13_STATE_TO_BLOCK[(s - V13_STATE_BASE) as usize];
        }
        // 1.16 (Nether Update, part 2): the V14 window — all identity
        // folds; the soul lantern's hanging state folds like the
        // lantern/chain's
        s if is_v14_state(s) => {
            return V14_STATE_TO_BLOCK[(s - V14_STATE_BASE) as usize];
        }
        // the 1.0-1.16.5 completeness audit: the V15 window — 26
        // identity item folds + the two spawner states folding to the
        // Monster Spawner block
        s if is_v16_state(s) => {
            return V16_STATE_TO_BLOCK[(s - V16_STATE_BASE) as usize];
        }
        s if is_v15_state(s) => {
            return V15_STATE_TO_BLOCK[(s - V15_STATE_BASE) as usize];
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
    // 1.13 V9 properties (F3 targeted-block lines): the pickle's count
    // and the egg's hatch stage are real vanilla blockstates
    if (V9_STATE_BASE + 30..=V9_STATE_BASE + 33).contains(&s) {
        return format!("Sea Pickle[pickles={}]", 1 + (s - (V9_STATE_BASE + 30)));
    }
    if (V9_STATE_BASE + 39..=V9_STATE_BASE + 41).contains(&s) {
        return format!("Turtle Egg[hatch={}]", s - (V9_STATE_BASE + 39));
    }
    // 1.14 V10 properties: berry bush age + campfire lit
    if (V10_STATE_BASE + 2..=V10_STATE_BASE + 5).contains(&s) {
        return format!("Sweet Berry Bush[age={}]", berry_bush_age(s));
    }
    if is_v10_state(s) && (V10_STATE_BASE + 6..=V10_STATE_BASE + 7).contains(&s) {
        return format!("Campfire[lit={}]", campfire_lit(s));
    }
    // 1.14 V11 properties: the smelters' lit flag + the lantern's
    // hanging flag (the F3 Targeted Block property lines)
    if is_v11_state(s) && (V11_STATE_BASE..=V11_STATE_BASE + 1).contains(&s) {
        return format!("Blast Furnace[lit={}]", v11_smelter_lit(s));
    }
    if is_v11_state(s) && (V11_STATE_BASE + 2..=V11_STATE_BASE + 3).contains(&s) {
        return format!("Smoker[lit={}]", v11_smelter_lit(s));
    }
    if is_v11_state(s) && (V11_STATE_BASE + 4..=V11_STATE_BASE + 5).contains(&s) {
        return format!("Lantern[hanging={}]", lantern_hanging(s));
    }
    // 1.15 V12 properties: the nest/hive honey level (the F3 Targeted
    // Block property line - a real vanilla blockstate)
    if is_v12_state(s) && (V12_STATE_BASE..=V12_STATE_BASE + 5).contains(&s) {
        return format!("Bee Nest[honey_level={}]", honey_level(s));
    }
    if is_v12_state(s) && (V12_STATE_BASE + 6..=V12_STATE_BASE + 11).contains(&s) {
        return format!("Beehive[honey_level={}]", honey_level(s));
    }
    // 1.16 V13 properties: the anchor's charge, the target's power,
    // the chain's hanging flag (the F3 Targeted Block property lines —
    // the vanilla blockstate spellings)
    if is_v13_state(s) && s < V13_STATE_BASE + 5 {
        return format!("Respawn Anchor[charge={}]", anchor_charge(s));
    }
    if is_v13_state(s) && (V13_STATE_BASE + 5..V13_STATE_BASE + 21).contains(&s) {
        return format!("Target[power={}]", target_power(s));
    }
    if is_v13_state(s) && (V13_STATE_BASE + 29..=V13_STATE_BASE + 30).contains(&s) {
        return format!("Chain[hanging={}]", chain_hanging(s));
    }
    // 1.16 V14 properties: the soul lantern's hanging flag (the F3
    // Targeted Block property line — the vanilla blockstate spelling)
    if is_v14_state(s) && (V14_STATE_BASE + 21..=V14_STATE_BASE + 22).contains(&s) {
        return format!("Soul Lantern[hanging={}]", soul_lantern_hanging(s));
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
        || is_v7_state(s)
        // 1.12 V8 window: concrete/powder/glazed are greedy cubes (their
        // BlockDef flags); the V8 items are cross/hud-only — never model
        || is_v8_state(s)
        // 1.13 V9 window: same shape — greedy cubes for the solids,
        // cross plants/items ride their BlockDef flags
        || is_v9_state(s)
        // 1.14 V10 window: same shape — bamboo/bush cross plants,
        // campfire/barrel greedy cubes, items ride their flags
        || is_v10_state(s)
        // 1.14 V11 window: same shape — the smelters are greedy cubes
        // (their BlockDef flags), the lantern + nugget item ride flags
        || is_v11_state(s)
        // 1.15 V12 window: same shape — nest/hive/honey/honeycomb-block
        // are greedy cubes (their BlockDef flags); the items ride flags
        || is_v12_state(s)
        // 1.16 V13 window: same shape — the stone-family blocks are
        // greedy cubes (their BlockDef flags); chain + soul fire + the
        // items ride their flags
        || is_v13_state(s)
        // 1.16 V14 window: same shape — the stems/nyliums/planks/
        // polished stones are greedy cubes (their BlockDef flags); the
        // fungi/roots/sprouts/vines + soul torch/lantern + the eggs
        // ride their flags
        || is_v14_state(s)
        // the completeness audit V15 window: the items ride their
        // flags; the two spawner states fold to the spawner block
        // (the SPAWNER_VINDICATOR/SPAWNER_EVOKER pattern)
        || is_v15_state(s)
        || is_v16_state(s)
        || s == SPAWNER_CAVESPIDER
        || s == SPAWNER_SILVERFISH
        || s == SPAWNER_VINDICATOR
        || s == SPAWNER_EVOKER
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
        // ---- 1.12: glazed terracotta — the facing selects the tile
        // ROTATION (top/bottom carry 4 pixel-rotated variants per color;
        // sides share one pattern — see TILE_GLAZED_*_BASE). "When
        // placed, glazed terracotta's texture rotates relative to the
        // direction the player is facing" (VERIFIED
        // w/Glazed_Terracotta §Placement).
        // ---- 1.13: sea pickle — the count state selects the tile
        // (1..4 pickles; VERIFIED w/Sea_Pickle: up to 4 per block) ----
        s if (V9_STATE_BASE + 30..=V9_STATE_BASE + 33).contains(&s) => {
            let off = s - (V9_STATE_BASE + 30);
            let t = TILE_SEA_PICKLE_BASE + off;
            [t, t, t, t]
        }
        // ---- 1.13: turtle egg — the hatch stage selects the tile
        // (0 = uncracked, 1 = slightly cracked, 2 = very cracked —
        // VERIFIED changelog §Blocks) ----
        s if (V9_STATE_BASE + 39..=V9_STATE_BASE + 41).contains(&s) => {
            let off = s - (V9_STATE_BASE + 39);
            let t = TILE_TURTLE_EGG_BASE + off;
            [t, t, t, t]
        }
        // ---- 1.14: sweet berry bush — the age state selects the tile
        // (0 = sapling shrub .. 3 = mature berry-laden bush) ----
        s if (V10_STATE_BASE + 2..=V10_STATE_BASE + 5).contains(&s) => {
            let off = s - (V10_STATE_BASE + 2);
            let t = TILE_BERRY_BUSH_BASE + off;
            [t, t, t, t]
        }
        // ---- 1.14: campfire — lit shows the glowing-coal tile, the
        // extinguished one the ash tile ----
        s if (V10_STATE_BASE + 6..=V10_STATE_BASE + 7).contains(&s) => {
            let t = if campfire_lit(s) { TILE_CAMPFIRE } else { TILE_CAMPFIRE_UNLIT };
            [t, t, t, t]
        }
        // ---- 1.14 (part 2): the smelters — lit swaps the side tiles
        // to the glowing variants (the FURNACE_LIT pattern), top stays
        // the shared stone furnace top ----
        s if is_v11_state(s) && (V11_STATE_BASE..=V11_STATE_BASE + 1).contains(&s) => {
            let t = if v11_smelter_lit(s) {
                TILE_BLAST_FURNACE_SIDE_LIT
            } else {
                TILE_BLAST_FURNACE_SIDE
            };
            [TILE_FURNACE_TOP, TILE_FURNACE_TOP, t, t]
        }
        s if is_v11_state(s) && (V11_STATE_BASE + 2..=V11_STATE_BASE + 3).contains(&s) => {
            let t = if v11_smelter_lit(s) {
                TILE_SMOKER_SIDE_LIT
            } else {
                TILE_SMOKER_SIDE
            };
            [TILE_FURNACE_TOP, TILE_FURNACE_TOP, t, t]
        }
        // ---- 1.14 (part 2): the lantern — one sprite for both forms
        // (sitting + hanging; the model difference is future work,
        // disclosed) ----
        s if is_v11_state(s) && (V11_STATE_BASE + 4..=V11_STATE_BASE + 6).contains(&s) => {
            [TILE_LANTERN, TILE_LANTERN, TILE_LANTERN, TILE_LANTERN]
        }
        // ---- 1.14 (part 3): the flowers — one sprite each, one state
        // each ----
        s if is_v11_state(s) && (V11_STATE_BASE + 7..=V11_STATE_BASE + 8).contains(&s) => {
            let t = if s == V11_STATE_BASE + 7 {
                TILE_CORNFLOWER
            } else {
                TILE_LILY_OF_THE_VALLEY
            };
            [t, t, t, t]
        }
        // ---- 1.15 (Buzzy Bees): the nest — honey_level 1..=4 share
        // the level-0 art; level 5 swaps the side tiles to the
        // honey-oozing front (VERIFIED w/Bee_nest: "Once it has the
        // maximum honey level of 5, it changes its appearance to show
        // honey oozing out") ----
        s if is_v12_state(s) && (V12_STATE_BASE..=V12_STATE_BASE + 5).contains(&s) => {
            let t = if honey_level(s) == 5 {
                TILE_BEE_NEST_FRONT_HONEY
            } else {
                TILE_BEE_NEST_FRONT
            };
            [TILE_BEE_NEST_TOP, TILE_BEE_NEST_TOP, t, t]
        }
        // ---- 1.15: the beehive — the same level-5 side swap ----
        s if is_v12_state(s)
            && (V12_STATE_BASE + 6..=V12_STATE_BASE + 11).contains(&s) =>
        {
            let t = if honey_level(s) == 5 {
                TILE_BEEHIVE_FRONT_HONEY
            } else {
                TILE_BEEHIVE_FRONT
            };
            [TILE_BEEHIVE_TOP, TILE_BEEHIVE_TOP, t, t]
        }
        // ---- 1.15: the honey + honeycomb blocks (identity states,
        // the BlockDef tile triple) ----
        s if is_v12_state(s) && s == V12_STATE_BASE + 12 => {
            [TILE_HONEY, TILE_HONEY, TILE_HONEY, TILE_HONEY]
        }
        s if is_v12_state(s) && s == V12_STATE_BASE + 13 => {
            [TILE_HONEYCOMB_BLOCK, TILE_HONEYCOMB_BLOCK, TILE_HONEYCOMB_BLOCK, TILE_HONEYCOMB_BLOCK]
        }
        // ---- 1.15: the item states — one sprite each ----
        s if is_v12_state(s) && (V12_STATE_BASE + 14..=V12_STATE_BASE + 17).contains(&s) => {
            let t = match s {
                x if x == V12_STATE_BASE + 14 => TILE_HONEYCOMB,
                x if x == V12_STATE_BASE + 15 => TILE_HONEY_BOTTLE,
                x if x == V12_STATE_BASE + 16 => TILE_SHEARS,
                _ => TILE_SPAWN_EGG_BEE,
            };
            [t, t, t, t]
        }
        // ---- 1.16 (Nether Update, part 1): the anchor's charge art —
        // the sides wake to the purple glow at charge >= 1 (the exact
        // charge rides the F3 line + the light); every other V13 state
        // is a plain identity fold through def().tiles ----
        s if is_v13_state(s) && s < V13_STATE_BASE + 5 => {
            let side = if anchor_charge(s) >= 1 {
                TILE_ANCHOR_SIDE_CHARGED
            } else {
                TILE_ANCHOR_SIDE
            };
            [TILE_ANCHOR_TOP, TILE_ANCHOR_TOP, side, side]
        }
        s if glazed_decode(s).is_some() => {
            let (color, facing) = glazed_decode(s).unwrap();
            let c = color as u16;
            [
                TILE_GLAZED_TOP_BASE + c * 4 + facing as u16,
                TILE_GLAZED_BOTTOM_BASE + c * 4 + facing as u16,
                TILE_GLAZED_SIDE_BASE + c,
                TILE_GLAZED_SIDE_BASE + c,
            ]
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
pub const TILE_MAX: u16 = 738; // the backlog round: 736/737 = rain streak + snowflake particle sprites, 738 = fire (weather bracket)
/// 1.11 egg tiles (egg-shaped, egg order 23..=28 = llama, vindicator,
/// evoker, vex, husk, stray) — the E1/E2/E3 egg-art convention
/// (e1_art::egg_art + palettes), replacing the interrupted round's
/// mob-sprite reuse for egg items. The zombie-villager egg (kind 5)
/// keeps its pre-existing tile in the base egg window.
pub const TILE_V7_EGG_BASE: u16 = 340;
pub const TILE_V7_EGG_END: u16 = 345;

// ---- 1.12 bracket tiles (World of Color Update, live 2026-09-07;
// minecraft.wiki/w/Java_Edition_1.12 + w/Concrete, w/Concrete_Powder,
// w/Glazed_Terracotta, w/Parrot, w/Illusioner — raw captures in
// scripts/v112_page_*.json) ----
/// 16 concrete tiles, engine color order (the vanilla dye-registry order
/// — same as the E3 stained-terracotta set, VERIFIED w/Terracotta):
/// white, orange, magenta, light blue, yellow, lime, pink, gray,
/// light gray, cyan, purple, blue, brown, green, red, black.
/// Concrete: hardness 1.8, blast 1.8 (VERIFIED w/Concrete infobox).
pub const TILE_CONCRETE_BASE: u16 = 346;
/// 16 concrete-powder tiles (hardness 0.5, blast 0.5, gravity-affected —
/// VERIFIED w/Concrete_Powder infobox + "Concrete powder falls when
/// there is a non-solid block beneath it").
pub const TILE_CONCRETE_POWDER_BASE: u16 = 362;
/// Glazed terracotta: 4 rotations per color for the TOP face (64 tiles,
/// color*4 + rotation). "When placed, glazed terracotta's texture
/// rotates relative to the direction the player is facing" (VERIFIED
/// w/Glazed_Terracotta §Placement). Rotation variants are pixel-rotated
/// copies of the base tile (v112_art::rotated_copy). Hardness 1.4.
pub const TILE_GLAZED_TOP_BASE: u16 = 378;
/// Glazed terracotta BOTTOM faces, 4 rotations per color (64 tiles).
pub const TILE_GLAZED_BOTTOM_BASE: u16 = 442;
/// Glazed terracotta side faces (16 tiles — all four sides share one
/// clean-room pattern; vanilla's model y-rotation moves side textures
/// between faces, which is invisible with a shared side tile — the
/// observable (top-pattern rotation) is carried by the top/bottom
/// rotation tiles).
pub const TILE_GLAZED_SIDE_BASE: u16 = 506;
/// 16 dye item icons. 1.12-era item names (the "White Dye" renames are
/// 1.14+ — version-scoped out): Bone Meal, Orange Dye, Magenta Dye,
/// Light Blue Dye, Dandelion Yellow, Lime Dye, Pink Dye, Gray Dye,
/// Light Gray Dye, Cyan Dye, Purple Dye, Lapis Lazuli, Cocoa Beans,
/// Cactus Green, Rose Red, Ink Sac. Acquisition economy (flowers, bone
/// meal crafting, squids etc.) stays under the standing dye-economy
/// deferral — palette-only items, consumed by the 1.12 concrete-powder
/// recipe (4 sand + 4 gravel + 1 dye → 8, shapeless — VERIFIED
/// changelog §Blocks + w/Concrete_Powder "The crafting recipe is
/// shapeless; the order of ingredients does not matter").
pub const TILE_DYE_BASE: u16 = 522;
/// 4 seed item icons (wheat/melon/pumpkin/beetroot — the 1.12 parrot
/// taming set, VERIFIED w/Parrot: "tamed by feeding wheat seeds, melon
/// seeds, pumpkin seeds, beetroot seeds"; torchflower seeds and pitcher
/// pods are 1.20+ additions, out of the bracket).
pub const TILE_SEEDS_BASE: u16 = 538;
/// Cookie item icon (the 1.12 parrot interaction: "Attempting to feed
/// cookies to a parrot now instantly kills the parrot, causing it to
/// emit poison particles" — VERIFIED w/Parrot §Cookies + 17w13a history).
pub const TILE_COOKIE: u16 = 542;
/// Parrot spawn-egg tile (egg-shaped, the E-series egg-art convention).
pub const TILE_PARROT_EGG: u16 = 543;
/// 5 parrot variant sprites (VERIFIED w/Parrot Variant NBT table:
/// 0=red "red_blue", 1=blue, 2=green, 3=cyan "yellow_blue", 4=gray).
pub const TILE_PARROT_BASE: u16 = 544;
/// Illusioner sprite (VERIFIED w/Illusioner: 32 HP hostile illager —
/// palette-only mob: vanilla has no spawn egg and it never spawns
/// naturally, "Unused and present only in Java Edition").
pub const TILE_ILLUSIONER: u16 = 549;

// ---- 1.13 (Update Aquatic) tiles: 550..=618 ----
/// coral block tiles ×5 (tube/brain/bubble/fire/horn).
pub const TILE_CORAL_BLOCK_BASE: u16 = 550;
/// dead coral block tiles ×5 (the irreversible gray form).
pub const TILE_DEAD_CORAL_BLOCK_BASE: u16 = 555;
/// coral plant (cross) tiles ×5.
pub const TILE_CORAL_PLANT_BASE: u16 = 560;
/// dead coral plant (cross) tiles ×5.
pub const TILE_DEAD_CORAL_PLANT_BASE: u16 = 565;
/// coral fan (cross-rendered) tiles ×5.
pub const TILE_CORAL_FAN_BASE: u16 = 570;
/// dead coral fan tiles ×5.
pub const TILE_DEAD_CORAL_FAN_BASE: u16 = 575;
/// sea pickle tiles ×4 (the 1..4 count states).
pub const TILE_SEA_PICKLE_BASE: u16 = 580;
/// blue ice (slipperiness 0.989).
pub const TILE_BLUE_ICE: u16 = 584;
/// dried kelp block.
pub const TILE_DRIED_KELP_BLOCK: u16 = 585;
/// kelp plant (cross).
pub const TILE_KELP: u16 = 586;
/// seagrass (cross).
pub const TILE_SEAGRASS: u16 = 587;
/// conduit (the heart-of-the-sea beacon block).
pub const TILE_CONDUIT: u16 = 588;
/// turtle egg tiles ×3 (hatch stages).
pub const TILE_TURTLE_EGG_BASE: u16 = 589;
/// heart of the sea item icon.
pub const TILE_HEART_OF_THE_SEA: u16 = 592;
/// nautilus shell item icon.
pub const TILE_NAUTILUS_SHELL: u16 = 593;
/// scute item icon.
pub const TILE_SCUTE: u16 = 594;
/// trident item icon.
pub const TILE_TRIDENT: u16 = 595;
/// phantom membrane item icon.
pub const TILE_PHANTOM_MEMBRANE: u16 = 596;
/// dried kelp item icon (the food).
pub const TILE_DRIED_KELP: u16 = 597;
/// turtle shell item icon.
pub const TILE_TURTLE_SHELL: u16 = 598;
/// potion of Slow Falling (cyan).
pub const TILE_POTION_SLOW_FALLING: u16 = 599;
/// potion of Slow Falling extended.
pub const TILE_POTION_SLOW_FALLING_EXT: u16 = 600;
/// potion of the Turtle Master.
pub const TILE_POTION_TURTLE_MASTER: u16 = 601;
/// potion of the Turtle Master (enhanced).
pub const TILE_POTION_TURTLE_MASTER_II: u16 = 602;
/// 1.13 spawn-egg tiles ×8 (the E-series egg-art convention).
pub const TILE_EGG_V113_BASE: u16 = 603;
/// 1.13 mob sprites: drowned/phantom/dolphin/cod/salmon/pufferfish/
/// tropical fish/turtle.
pub const TILE_MOB_DROWNED: u16 = 611;
pub const TILE_MOB_PHANTOM: u16 = 612;
pub const TILE_MOB_DOLPHIN: u16 = 613;
pub const TILE_MOB_COD: u16 = 614;
pub const TILE_MOB_SALMON: u16 = 615;
pub const TILE_MOB_PUFFERFISH: u16 = 616;
pub const TILE_MOB_TROPICAL_FISH: u16 = 617;
pub const TILE_MOB_TURTLE: u16 = 618;

// ---- 1.14 (Village & Pillage — nature half) tiles: 619..=633 ----
/// bamboo stalk (the segmented green culm, cross-rendered).
pub const TILE_BAMBOO: u16 = 619;
/// bamboo shoot (the planted sapling form).
pub const TILE_BAMBOO_SHOOT: u16 = 620;
/// sweet berry bush tiles ×4 (the age 0..3 growth stages).
pub const TILE_BERRY_BUSH_BASE: u16 = 621;
/// lit campfire (logs + glowing coals).
pub const TILE_CAMPFIRE: u16 = 625;
/// extinguished campfire (logs + ash).
pub const TILE_CAMPFIRE_UNLIT: u16 = 626;
/// barrel top/bottom faces (the banded lid).
pub const TILE_BARREL_TOP: u16 = 627;
/// barrel side faces (the staves).
pub const TILE_BARREL_SIDE: u16 = 628;
/// sweet berries item icon.
pub const TILE_SWEET_BERRIES: u16 = 629;
/// fox spawn egg (the E-series egg-art convention).
pub const TILE_EGG_FOX: u16 = 630;
/// the fox mob sprite.
pub const TILE_MOB_FOX: u16 = 631;
/// stick item icon (the legacy item added with this window).
pub const TILE_STICK: u16 = 632;
/// charcoal item icon (the legacy item added with this window).
pub const TILE_CHARCOAL: u16 = 633;

// ---- 1.14 (Village & Pillage — nature half, part 2) tiles: 634..=639 ----
/// blast furnace side/front (the dark iron furnace face).
pub const TILE_BLAST_FURNACE_SIDE: u16 = 634;
/// lit blast furnace side (the glowing opening).
pub const TILE_BLAST_FURNACE_SIDE_LIT: u16 = 635;
/// smoker side/front (the log-walled smoker face).
pub const TILE_SMOKER_SIDE: u16 = 636;
/// lit smoker side (the glowing vent).
pub const TILE_SMOKER_SIDE_LIT: u16 = 637;
/// the lantern sprite (cross-rendered, like the torch; item icon reuses).
pub const TILE_LANTERN: u16 = 638;
/// iron nugget item icon.
pub const TILE_IRON_NUGGET: u16 = 639;

// ---- 1.14 (part 3): the flower tiles 640..=641 ----
/// cornflower sprite (cross plant; the deep-blue petals + stem).
pub const TILE_CORNFLOWER: u16 = 640;
/// lily of the valley sprite (cross plant; white bell florets).
pub const TILE_LILY_OF_THE_VALLEY: u16 = 641;

// ---- 1.15 (Buzzy Bees): tiles 642..=654 (v115_art.rs) ----
/// bee-nest top — the woven straw crown.
pub const TILE_BEE_NEST_TOP: u16 = 642;
/// bee-nest front — the straw wall with the dark entrance hole (the
/// furnace-pattern side tile: all four sides show the entrance — no
/// facing states in the engine, disclosed).
pub const TILE_BEE_NEST_FRONT: u16 = 643;
/// beehive top — the planked crown.
pub const TILE_BEEHIVE_TOP: u16 = 644;
/// beehive front — the plank wall with the entrance slot.
pub const TILE_BEEHIVE_FRONT: u16 = 645;
/// the honey block — amber translucent gel (the "sticky" block).
pub const TILE_HONEY: u16 = 646;
/// the honeycomb block — the hexagon-cell wall.
pub const TILE_HONEYCOMB_BLOCK: u16 = 647;
/// the honeycomb item sprite.
pub const TILE_HONEYCOMB: u16 = 648;
/// the honey bottle sprite (the amber flask).
pub const TILE_HONEY_BOTTLE: u16 = 649;
/// the shears sprite (the legacy iron tool).
pub const TILE_SHEARS: u16 = 650;
/// the bee spawn egg sprite (kind 41 — EGG_PALETTES family).
pub const TILE_SPAWN_EGG_BEE: u16 = 651;
/// bee-nest front at honey_level 5 — honey oozing from the hole.
pub const TILE_BEE_NEST_FRONT_HONEY: u16 = 652;
/// beehive front at honey_level 5 — honey oozing from the slot.
pub const TILE_BEEHIVE_FRONT_HONEY: u16 = 653;
/// the bee mob billboard sprite (v115_art::bee_art).
pub const TILE_MOB_BEE: u16 = 654;

// ---- 1.16 (Nether Update, part 1): the V13 art window, tiles
// 655..=672 — painted from day one in v116_art.rs (the 1.13 blank-
// window lesson; the coverage guard auto-extends). ----
pub const TILE_SOUL_SOIL: u16 = 655;
pub const TILE_BASALT_SIDE: u16 = 656;
pub const TILE_BASALT_TOP: u16 = 657;
pub const TILE_BLACKSTONE: u16 = 658;
pub const TILE_GILDED_BLACKSTONE: u16 = 659;
pub const TILE_CRYING_OBSIDIAN: u16 = 660;
/// respawn-anchor top — the dark ring portal face.
pub const TILE_ANCHOR_TOP: u16 = 661;
/// respawn-anchor sides, uncharged — the crying-obsidian-dark shell.
pub const TILE_ANCHOR_SIDE: u16 = 662;
/// respawn-anchor sides at charge >= 1 — the purple glow wakes
/// (grows visually with charge via the emissive tint class; the F3
/// line carries the exact charge).
pub const TILE_ANCHOR_SIDE_CHARGED: u16 = 663;
/// the target — the concentric-ring bullseye face.
pub const TILE_TARGET: u16 = 664;
pub const TILE_NETHER_GOLD_ORE: u16 = 665;
pub const TILE_ANCIENT_DEBRIS_TOP: u16 = 666;
pub const TILE_ANCIENT_DEBRIS_SIDE: u16 = 667;
pub const TILE_NETHERITE_BLOCK: u16 = 668;
/// the chain (sitting + hanging share the sprite; the placement
/// state picks the offset — the lantern pattern).
pub const TILE_CHAIN: u16 = 669;
/// soul fire — the blue flame cross-sprite.
pub const TILE_SOUL_FIRE: u16 = 670;
pub const TILE_NETHERITE_SCRAP: u16 = 671;
pub const TILE_NETHERITE_INGOT: u16 = 672;

// ---- 1.16 (Nether Update, PART 2): the V14 art window, tiles
// 673..=705 — painted from day one in v116b_art.rs (the 1.13 blank-
// window lesson; the coverage guard auto-extends). ----
pub const TILE_CRIMSON_STEM_SIDE: u16 = 673;
pub const TILE_CRIMSON_STEM_TOP: u16 = 674;
/// crimson hyphae — the all-sides "bark" form (the jungle-bark class).
pub const TILE_CRIMSON_HYPHAE: u16 = 675;
pub const TILE_CRIMSON_PLANKS: u16 = 676;
pub const TILE_CRIMSON_NYLIUM_TOP: u16 = 677;
pub const TILE_CRIMSON_NYLIUM_SIDE: u16 = 678;
/// crimson fungus — the red-cap mushroom sprite.
pub const TILE_CRIMSON_FUNGUS: u16 = 679;
/// crimson roots — the red tuft sprite.
pub const TILE_CRIMSON_ROOTS: u16 = 680;
/// weeping vines — the hanging red-vine sprite.
pub const TILE_WEEPING_VINES: u16 = 681;
pub const TILE_WARPED_STEM_SIDE: u16 = 682;
pub const TILE_WARPED_STEM_TOP: u16 = 683;
/// warped hyphae — the all-sides "bark" form.
pub const TILE_WARPED_HYPHAE: u16 = 684;
pub const TILE_WARPED_PLANKS: u16 = 685;
pub const TILE_WARPED_NYLIUM_TOP: u16 = 686;
pub const TILE_WARPED_NYLIUM_SIDE: u16 = 687;
/// warped fungus — the teal-cap mushroom sprite.
pub const TILE_WARPED_FUNGUS: u16 = 688;
/// warped roots — the teal tuft sprite.
pub const TILE_WARPED_ROOTS: u16 = 689;
/// twisting vines — the climbing teal-vine sprite.
pub const TILE_TWISTING_VINES: u16 = 690;
pub const TILE_WARPED_WART_BLOCK: u16 = 691;
/// shroomlight — the glowing orange-pink fungus lamp.
pub const TILE_SHROOMLIGHT: u16 = 692;
/// nether sprouts — the teal curl sprout sprite.
pub const TILE_NETHER_SPROUTS: u16 = 693;
pub const TILE_POLISHED_BASALT_SIDE: u16 = 694;
pub const TILE_POLISHED_BASALT_TOP: u16 = 695;
pub const TILE_POLISHED_BLACKSTONE: u16 = 696;
pub const TILE_POLISHED_BLACKSTONE_BRICKS: u16 = 697;
/// soul torch — the blue-flame torch sprite (light 10).
pub const TILE_SOUL_TORCH: u16 = 698;
/// soul lantern — the blue-flame lantern sprite (sitting + hanging
/// share it; the placement state picks the offset — the lantern
/// pattern).
pub const TILE_SOUL_LANTERN: u16 = 699;
/// the strider spawn egg sprite (kind 42).
pub const TILE_SPAWN_EGG_STRIDER: u16 = 700;
/// the piglin spawn egg sprite (kind 43).
pub const TILE_SPAWN_EGG_PIGLIN: u16 = 701;
/// the hoglin spawn egg sprite (kind 44).
pub const TILE_SPAWN_EGG_HOGLIN: u16 = 702;
/// the strider mob billboard sprite (v116b_art::strider_art).
pub const TILE_MOB_STRIDER: u16 = 703;
/// the piglin mob billboard sprite (v116b_art::piglin_art).
pub const TILE_MOB_PIGLIN: u16 = 704;
/// the hoglin mob billboard sprite (v116b_art::hoglin_art).
pub const TILE_MOB_HOGLIN: u16 = 705;
// ---- the 1.0-1.16.5 completeness audit (V15 window, tiles 706..=734) ----
/// the steak / cooked-beef item sprite.
pub const TILE_STEAK: u16 = 706;
/// the cooked porkchop item sprite.
pub const TILE_COOKED_PORKCHOP: u16 = 707;
/// the cooked chicken item sprite.
pub const TILE_COOKED_CHICKEN: u16 = 708;
/// the cooked mutton item sprite.
pub const TILE_COOKED_MUTTON: u16 = 709;
/// the cooked cod item sprite.
pub const TILE_COOKED_COD: u16 = 710;
/// the cooked salmon item sprite.
pub const TILE_COOKED_SALMON: u16 = 711;
/// the apple item sprite.
pub const TILE_APPLE: u16 = 712;
/// the bowl item sprite.
pub const TILE_BOWL: u16 = 713;
/// the mushroom stew item sprite.
pub const TILE_MUSHROOM_STEW: u16 = 714;
/// the rabbit stew item sprite.
pub const TILE_RABBIT_STEW: u16 = 715;
/// the beetroot item sprite.
pub const TILE_BEETROOT: u16 = 716;
/// the beetroot soup item sprite.
pub const TILE_BEETROOT_SOUP: u16 = 717;
/// the sugar item sprite.
pub const TILE_SUGAR: u16 = 718;
/// the chicken egg item sprite.
pub const TILE_EGG: u16 = 719;
/// the poisonous potato item sprite.
pub const TILE_POISONOUS_POTATO: u16 = 720;
/// the popped chorus fruit item sprite.
pub const TILE_POPPED_CHORUS: u16 = 721;
/// the ghast tear item sprite.
pub const TILE_GHAST_TEAR: u16 = 722;
/// the potion of leaping item sprite.
pub const TILE_POTION_LEAPING: u16 = 723;
/// the potion of leaping II item sprite.
pub const TILE_POTION_LEAPING_II: u16 = 724;
/// the extended potion of leaping item sprite.
pub const TILE_POTION_LEAPING_LONG: u16 = 725;
/// the potion of regeneration item sprite.
pub const TILE_POTION_REGEN: u16 = 726;
/// the potion of regeneration II item sprite.
pub const TILE_POTION_REGEN_II: u16 = 727;
/// the extended potion of regeneration item sprite.
pub const TILE_POTION_REGEN_LONG: u16 = 728;
/// the ghast spawn-egg sprite (kind 45).
pub const TILE_SPAWN_EGG_GHAST: u16 = 729;
/// the cave-spider spawn-egg sprite (kind 46).
pub const TILE_SPAWN_EGG_CAVESPIDER: u16 = 730;
/// the silverfish spawn-egg sprite (kind 47).
pub const TILE_SPAWN_EGG_SILVERFISH: u16 = 731;
/// the ghast mob billboard sprite (audit16_art::ghast_art).
pub const TILE_MOB_GHAST: u16 = 732;
/// the cave-spider mob billboard sprite (audit16_art::cave_spider_art).
pub const TILE_MOB_CAVESPIDER: u16 = 733;
/// the silverfish mob billboard sprite (audit16_art::silverfish_art).
pub const TILE_MOB_SILVERFISH: u16 = 734;
/// the melon slice item sprite (audit16_art::melon_slice_art) — the
/// sweep-2 food row.
pub const TILE_MELON_SLICE: u16 = 735;
/// the rain streak particle sprite (backlog round: weather). A tall
/// 2×10-px droplet — NOT a vanilla asset (clean-room procedural art,
/// weather_art::rain_streak_art).
pub const TILE_RAIN_PARTICLE: u16 = 736;
/// the snowflake particle sprite (backlog round: weather) — a 5-px
/// soft flake (weather_art::snow_flake_art).
pub const TILE_SNOW_PARTICLE: u16 = 737;
/// the fire block sprite (backlog round: weather + flint-and-steel) —
/// clean-room flame art (weather_art::fire_art).
pub const TILE_FIRE: u16 = 738;
/// the fire block (id 506 — lightning ignition + flint-and-steel
/// source; VERIFIED w/Weather §Lightning).
pub const FIRE: u16 = 506;

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
            // 1.11 items (shulker shell, totem, the 4+1 new eggs and
            // the re-added husk/stray eggs)
            | SHULKER_SHELL
            | TOTEM_OF_UNDYING
            | SPAWN_EGG_LLAMA
            | SPAWN_EGG_VINDICATOR
            | SPAWN_EGG_EVOKER
            | SPAWN_EGG_VEX
            | SPAWN_EGG_HUSK
            | SPAWN_EGG_STRAY
            // VERIFICATION-REPORT fix #4: the coal fuel item
            | COAL
            // ---- F-series item-blocks (1.7.2-1.10, merge-renumbered) ----
            | RAW_FISH | RAW_SALMON | CLOWNFISH | PUFFERFISH
            | RAW_RABBIT | COOKED_RABBIT | RABBIT_HIDE | RABBIT_FOOT
            | PRISMARINE_SHARD | PRISMARINE_CRYSTALS
            | CHORUS_FRUIT | ELYTRA | SHIELD
            // ---- 1.12 item-blocks (World of Color): the 16 dye palette
            // items + the 4 taming seeds + the cookie — inventory-only
            // (concrete/powder/glazed are real placeable BLOCKS) ----
            | COOKIE
            // ---- 1.13 item-blocks (Update Aquatic): the conduit/turtle
            // craft items, the trident, the membrane, the food + the 4
            // new potions — inventory-only (coral/kelp/egg blocks are
            // real placeable BLOCKS) ----
            | HEART_OF_THE_SEA
            | NAUTILUS_SHELL
            | SCUTE
            | TRIDENT
            | PHANTOM_MEMBRANE
            | DRIED_KELP
            | TURTLE_SHELL
            | POTION_SLOW_FALLING
            | POTION_SLOW_FALLING_EXT
            | POTION_TURTLE_MASTER
            | POTION_TURTLE_MASTER_II
            // ---- 1.14 item-blocks (Village & Pillage): the berries
            // (food + planting — the plant branch handles soil uses),
            // the fox egg (caught by is_spawn_egg too), and the two
            // late-arriving legacy items stick + charcoal ----
            | SWEET_BERRIES
            | STICK
            | CHARCOAL
            // ---- 1.14 (part 2): the iron nugget (the lantern recipe's
            // material; 9:1 with the iron-ingot stand-in) ----
            | IRON_NUGGET
            // ---- 1.15 (Buzzy Bees): honeycomb + the honey bottle
            // (the drink branch + hive-harvest branch handle their
            // uses) and shears (the hive-harvest tool) ----
            | HONEYCOMB
            | HONEY_BOTTLE
            | SHEARS
            // ---- 1.16 (Nether Update, part 1): the netherite material
            // items (never placeable — the ancient-debris smelt output
            // and the ingot craft input) ----
            | NETHERITE_SCRAP
            | NETHERITE_INGOT
            // ---- the 1.0-1.16.5 completeness audit: the V15 item
            // rows — the cooked meats, the kitchen chain, the popped
            // chorus, the ghast tear, the two new potion families
            // (the 3 eggs ride is_spawn_egg below) ----
            | STEAK
            | COOKED_PORKCHOP
            | COOKED_CHICKEN
            | COOKED_MUTTON
            | COOKED_COD
            | COOKED_SALMON
            | APPLE
            | BOWL
            | MUSHROOM_STEW
            | RABBIT_STEW
            | BEETROOT
            | BEETROOT_SOUP
            | SUGAR
            | EGG
            | MELON_SLICE
            | POISONOUS_POTATO
            | POPPED_CHORUS_FRUIT
            | GHAST_TEAR
            | POTION_LEAPING
            | POTION_LEAPING_II
            | POTION_LEAPING_LONG
            | POTION_REGEN
            | POTION_REGEN_II
            | POTION_REGEN_LONG
    ) || is_spawn_egg(b)
        || (DYE_BASE..=DYE_END).contains(&b)
        || is_seeds(b)
}

/// true for the mob spawn-egg item ids (124..=143 + the E3 window
/// 196..=198).
#[inline]
pub fn is_spawn_egg(b: u16) -> bool {
    (SPAWN_EGG_BASE..=SPAWN_EGG_MAX).contains(&b)
        || (E3_SPAWN_EGG_BASE..=E3_SPAWN_EGG_END).contains(&b)
        // 1.11: the V7 egg window (llama/vindicator/evoker/vex + the
        // re-added husk/stray + the 5th new zombie-villager egg)
        || (SPAWN_EGG_LLAMA..=SPAWN_EGG_STRAY).contains(&b)
        // 1.12: the V8 parrot egg (kind 30)
        || b == SPAWN_EGG_PARROT
        // 1.13: the V9 egg window (kinds 32..=39 — drowned/phantom/
        // dolphin/cod/salmon/pufferfish/tropical fish/turtle)
        || (SPAWN_EGG_DROWNED..=SPAWN_EGG_TURTLE).contains(&b)
        // 1.14: the fox egg (kind 40)
        || b == SPAWN_EGG_FOX
        // 1.15: the bee egg (kind 41 — changelog §Items: "Bee Spawn
        // Egg")
        || b == SPAWN_EGG_BEE
        // 1.16: the V14 egg window (kinds 42..=44 — strider/piglin/
        // hoglin)
        || b == SPAWN_EGG_STRIDER
        || b == SPAWN_EGG_PIGLIN
        || b == SPAWN_EGG_HOGLIN
        // the completeness audit: the three classic-mob eggs (the
        // ghast/cave spider/silverfish rounds)
        || b == SPAWN_EGG_GHAST
        || b == SPAWN_EGG_CAVE_SPIDER
        || b == SPAWN_EGG_SILVERFISH
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
    // 1.11 bracket: kinds 23..=29 (the changelog's "5 new spawn eggs"
    // — llama/vindicator/evoker/vex/zombie-villager — plus the re-added
    // husk/stray eggs: "Eggs that were removed in Java Edition
    // 1.10-pre2 are re-added ... including: ... Husk spawn egg, Stray
    // spawn egg")
    if (SPAWN_EGG_LLAMA..=SPAWN_EGG_STRAY).contains(&b) {
        return Some(23 + (b - SPAWN_EGG_LLAMA) as u8);
    }
    // 1.12: the parrot egg — kind 30 (MobKind::Parrot::from_egg)
    if b == SPAWN_EGG_PARROT {
        return Some(30);
    }
    // 1.13: the V9 egg window — kinds 32..=39 (MobKind::from_egg)
    if (SPAWN_EGG_DROWNED..=SPAWN_EGG_TURTLE).contains(&b) {
        return Some(32 + (b - SPAWN_EGG_DROWNED) as u8);
    }
    // 1.14: the fox egg — kind 40 (MobKind::Fox::from_egg)
    if b == SPAWN_EGG_FOX {
        return Some(40);
    }
    // 1.15: the bee egg — kind 41 (MobKind::Bee::from_egg)
    if b == SPAWN_EGG_BEE {
        return Some(41);
    }
    // 1.16: the V14 egg window — kinds 42..=44 (the changelog's own
    // spawn-egg list: "Strider Spawn Egg", "Piglin Spawn Egg",
    // "Hoglin Spawn Egg" — the zoglin/brute eggs are trimmed with
    // their mobs, disclosed)
    if b == SPAWN_EGG_STRIDER {
        return Some(42);
    }
    if b == SPAWN_EGG_PIGLIN {
        return Some(43);
    }
    if b == SPAWN_EGG_HOGLIN {
        return Some(44);
    }
    // the completeness audit: kinds 45..=47 (the ghast/cave-spider/
    // silverfish eggs)
    if b == SPAWN_EGG_GHAST {
        return Some(45);
    }
    if b == SPAWN_EGG_CAVE_SPIDER {
        return Some(46);
    }
    if b == SPAWN_EGG_SILVERFISH {
        return Some(47);
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
    // 1.14: a LIT campfire emits 15 (VERIFIED w/Campfire infobox:
    // "Luminous Yes (15) when lit" — the unlit state stays 0)
    if is_v10_state(s) && campfire_lit(s) {
        return 15;
    }
    // 1.14 (part 2): a LIT blast furnace / smoker emits 13 (VERIFIED
    // from the captures v114b_page_Blast_Furnace.json +
    // v114b_page_Smoker.json infoboxes: "light: Yes (13) (when
    // active)" — the idle state stays 0). The lantern is emissive 15
    // through its BLOCK row in both sitting and hanging forms (the
    // fold routes them to the same d() emission).
    if v11_smelter_lit(s) {
        return 13;
    }
    // 1.16 (Nether Update, part 1): the respawn anchor's charge light —
    // "the respawn anchor glow with a light level of 3. Each glowstone
    // after the first increases the light level by 4, up to a maximum of
    // 15" (VERIFIED w/Respawn_Anchor: charges 1/2/3/4 → 3/7/11/15).
    // Crying obsidian (10) and soul fire (10) ride their BLOCK rows.
    if is_v13_state(s) && s < V13_STATE_BASE + 5 {
        return anchor_light(s);
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
    // ---- 1.11 bracket (Exploration Update) ----
    d("Shulker Box", [TILE_SHULKER_BOX, TILE_SHULKER_BOX, TILE_SHULKER_BOX], true, true, false, false, 0, SoundFamily::Stone),
    d("Shulker Shell", [TILE_SHULKER_SHELL, TILE_SHULKER_SHELL, TILE_SHULKER_SHELL], false, false, true, false, 0, SoundFamily::Stone),
    d("Totem of Undying", [TILE_TOTEM, TILE_TOTEM, TILE_TOTEM], false, false, true, false, 0, SoundFamily::Grass),
    // 1.11 egg items: egg-shaped tiles (TILE_V7_EGG_BASE order = the
    // egg_mob kind order 23..=29), NOT the mob billboard sprites — the
    // E1/E2/E3 egg convention
    d("Llama Spawn Egg", [TILE_V7_EGG_BASE, TILE_V7_EGG_BASE, TILE_V7_EGG_BASE], false, false, true, false, 0, SoundFamily::Stone),
    d("Vindicator Spawn Egg", [TILE_V7_EGG_BASE + 1, TILE_V7_EGG_BASE + 1, TILE_V7_EGG_BASE + 1], false, false, true, false, 0, SoundFamily::Stone),
    d("Evoker Spawn Egg", [TILE_V7_EGG_BASE + 2, TILE_V7_EGG_BASE + 2, TILE_V7_EGG_BASE + 2], false, false, true, false, 0, SoundFamily::Stone),
    d("Vex Spawn Egg", [TILE_V7_EGG_BASE + 3, TILE_V7_EGG_BASE + 3, TILE_V7_EGG_BASE + 3], false, false, true, false, 0, SoundFamily::Stone),
    d("Husk Spawn Egg", [TILE_V7_EGG_BASE + 4, TILE_V7_EGG_BASE + 4, TILE_V7_EGG_BASE + 4], false, false, true, false, 0, SoundFamily::Stone),
    d("Stray Spawn Egg", [TILE_V7_EGG_BASE + 5, TILE_V7_EGG_BASE + 5, TILE_V7_EGG_BASE + 5], false, false, true, false, 0, SoundFamily::Stone),
    // ---- 1.12 bracket (World of Color Update, live-verified 2026-09-07;
    // concrete hardness 1.8 w/Concrete, powder 0.5 sand-sound family
    // w/Concrete_Powder §Sounds "block.sand.*", glazed 1.4 stone family
    // w/Glazed_Terracotta §Sounds "block.stone.*") ----
    // concrete, 16 colors (solid, opaque, stone family)
    d("White Concrete", [TILE_CONCRETE_BASE, TILE_CONCRETE_BASE, TILE_CONCRETE_BASE], true, true, false, false, 0, SoundFamily::Stone),
    d("Orange Concrete", [TILE_CONCRETE_BASE + 1, TILE_CONCRETE_BASE + 1, TILE_CONCRETE_BASE + 1], true, true, false, false, 0, SoundFamily::Stone),
    d("Magenta Concrete", [TILE_CONCRETE_BASE + 2, TILE_CONCRETE_BASE + 2, TILE_CONCRETE_BASE + 2], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Blue Concrete", [TILE_CONCRETE_BASE + 3, TILE_CONCRETE_BASE + 3, TILE_CONCRETE_BASE + 3], true, true, false, false, 0, SoundFamily::Stone),
    d("Yellow Concrete", [TILE_CONCRETE_BASE + 4, TILE_CONCRETE_BASE + 4, TILE_CONCRETE_BASE + 4], true, true, false, false, 0, SoundFamily::Stone),
    d("Lime Concrete", [TILE_CONCRETE_BASE + 5, TILE_CONCRETE_BASE + 5, TILE_CONCRETE_BASE + 5], true, true, false, false, 0, SoundFamily::Stone),
    d("Pink Concrete", [TILE_CONCRETE_BASE + 6, TILE_CONCRETE_BASE + 6, TILE_CONCRETE_BASE + 6], true, true, false, false, 0, SoundFamily::Stone),
    d("Gray Concrete", [TILE_CONCRETE_BASE + 7, TILE_CONCRETE_BASE + 7, TILE_CONCRETE_BASE + 7], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Gray Concrete", [TILE_CONCRETE_BASE + 8, TILE_CONCRETE_BASE + 8, TILE_CONCRETE_BASE + 8], true, true, false, false, 0, SoundFamily::Stone),
    d("Cyan Concrete", [TILE_CONCRETE_BASE + 9, TILE_CONCRETE_BASE + 9, TILE_CONCRETE_BASE + 9], true, true, false, false, 0, SoundFamily::Stone),
    d("Purple Concrete", [TILE_CONCRETE_BASE + 10, TILE_CONCRETE_BASE + 10, TILE_CONCRETE_BASE + 10], true, true, false, false, 0, SoundFamily::Stone),
    d("Blue Concrete", [TILE_CONCRETE_BASE + 11, TILE_CONCRETE_BASE + 11, TILE_CONCRETE_BASE + 11], true, true, false, false, 0, SoundFamily::Stone),
    d("Brown Concrete", [TILE_CONCRETE_BASE + 12, TILE_CONCRETE_BASE + 12, TILE_CONCRETE_BASE + 12], true, true, false, false, 0, SoundFamily::Stone),
    d("Green Concrete", [TILE_CONCRETE_BASE + 13, TILE_CONCRETE_BASE + 13, TILE_CONCRETE_BASE + 13], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Concrete", [TILE_CONCRETE_BASE + 14, TILE_CONCRETE_BASE + 14, TILE_CONCRETE_BASE + 14], true, true, false, false, 0, SoundFamily::Stone),
    d("Black Concrete", [TILE_CONCRETE_BASE + 15, TILE_CONCRETE_BASE + 15, TILE_CONCRETE_BASE + 15], true, true, false, false, 0, SoundFamily::Stone),
    // concrete powder, 16 colors (solid, opaque, SAND family — VERIFIED
    // w/Concrete_Powder §Sounds: "block.sand.*")
    d("White Concrete Powder", [TILE_CONCRETE_POWDER_BASE, TILE_CONCRETE_POWDER_BASE, TILE_CONCRETE_POWDER_BASE], true, true, false, false, 0, SoundFamily::Sand),
    d("Orange Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 1, TILE_CONCRETE_POWDER_BASE + 1, TILE_CONCRETE_POWDER_BASE + 1], true, true, false, false, 0, SoundFamily::Sand),
    d("Magenta Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 2, TILE_CONCRETE_POWDER_BASE + 2, TILE_CONCRETE_POWDER_BASE + 2], true, true, false, false, 0, SoundFamily::Sand),
    d("Light Blue Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 3, TILE_CONCRETE_POWDER_BASE + 3, TILE_CONCRETE_POWDER_BASE + 3], true, true, false, false, 0, SoundFamily::Sand),
    d("Yellow Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 4, TILE_CONCRETE_POWDER_BASE + 4, TILE_CONCRETE_POWDER_BASE + 4], true, true, false, false, 0, SoundFamily::Sand),
    d("Lime Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 5, TILE_CONCRETE_POWDER_BASE + 5, TILE_CONCRETE_POWDER_BASE + 5], true, true, false, false, 0, SoundFamily::Sand),
    d("Pink Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 6, TILE_CONCRETE_POWDER_BASE + 6, TILE_CONCRETE_POWDER_BASE + 6], true, true, false, false, 0, SoundFamily::Sand),
    d("Gray Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 7, TILE_CONCRETE_POWDER_BASE + 7, TILE_CONCRETE_POWDER_BASE + 7], true, true, false, false, 0, SoundFamily::Sand),
    d("Light Gray Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 8, TILE_CONCRETE_POWDER_BASE + 8, TILE_CONCRETE_POWDER_BASE + 8], true, true, false, false, 0, SoundFamily::Sand),
    d("Cyan Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 9, TILE_CONCRETE_POWDER_BASE + 9, TILE_CONCRETE_POWDER_BASE + 9], true, true, false, false, 0, SoundFamily::Sand),
    d("Purple Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 10, TILE_CONCRETE_POWDER_BASE + 10, TILE_CONCRETE_POWDER_BASE + 10], true, true, false, false, 0, SoundFamily::Sand),
    d("Blue Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 11, TILE_CONCRETE_POWDER_BASE + 11, TILE_CONCRETE_POWDER_BASE + 11], true, true, false, false, 0, SoundFamily::Sand),
    d("Brown Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 12, TILE_CONCRETE_POWDER_BASE + 12, TILE_CONCRETE_POWDER_BASE + 12], true, true, false, false, 0, SoundFamily::Sand),
    d("Green Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 13, TILE_CONCRETE_POWDER_BASE + 13, TILE_CONCRETE_POWDER_BASE + 13], true, true, false, false, 0, SoundFamily::Sand),
    d("Red Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 14, TILE_CONCRETE_POWDER_BASE + 14, TILE_CONCRETE_POWDER_BASE + 14], true, true, false, false, 0, SoundFamily::Sand),
    d("Black Concrete Powder", [TILE_CONCRETE_POWDER_BASE + 15, TILE_CONCRETE_POWDER_BASE + 15, TILE_CONCRETE_POWDER_BASE + 15], true, true, false, false, 0, SoundFamily::Sand),
    // glazed terracotta, 16 colors (def tiles = facing-0 rotations; the
    // per-state tiles in state_tiles carry the full facing selection)
    d("White Glazed Terracotta", [TILE_GLAZED_TOP_BASE, TILE_GLAZED_BOTTOM_BASE, TILE_GLAZED_SIDE_BASE], true, true, false, false, 0, SoundFamily::Stone),
    d("Orange Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 4, TILE_GLAZED_BOTTOM_BASE + 4, TILE_GLAZED_SIDE_BASE + 1], true, true, false, false, 0, SoundFamily::Stone),
    d("Magenta Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 8, TILE_GLAZED_BOTTOM_BASE + 8, TILE_GLAZED_SIDE_BASE + 2], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Blue Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 12, TILE_GLAZED_BOTTOM_BASE + 12, TILE_GLAZED_SIDE_BASE + 3], true, true, false, false, 0, SoundFamily::Stone),
    d("Yellow Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 16, TILE_GLAZED_BOTTOM_BASE + 16, TILE_GLAZED_SIDE_BASE + 4], true, true, false, false, 0, SoundFamily::Stone),
    d("Lime Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 20, TILE_GLAZED_BOTTOM_BASE + 20, TILE_GLAZED_SIDE_BASE + 5], true, true, false, false, 0, SoundFamily::Stone),
    d("Pink Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 24, TILE_GLAZED_BOTTOM_BASE + 24, TILE_GLAZED_SIDE_BASE + 6], true, true, false, false, 0, SoundFamily::Stone),
    d("Gray Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 28, TILE_GLAZED_BOTTOM_BASE + 28, TILE_GLAZED_SIDE_BASE + 7], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Gray Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 32, TILE_GLAZED_BOTTOM_BASE + 32, TILE_GLAZED_SIDE_BASE + 8], true, true, false, false, 0, SoundFamily::Stone),
    d("Cyan Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 36, TILE_GLAZED_BOTTOM_BASE + 36, TILE_GLAZED_SIDE_BASE + 9], true, true, false, false, 0, SoundFamily::Stone),
    d("Purple Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 40, TILE_GLAZED_BOTTOM_BASE + 40, TILE_GLAZED_SIDE_BASE + 10], true, true, false, false, 0, SoundFamily::Stone),
    d("Blue Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 44, TILE_GLAZED_BOTTOM_BASE + 44, TILE_GLAZED_SIDE_BASE + 11], true, true, false, false, 0, SoundFamily::Stone),
    d("Brown Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 48, TILE_GLAZED_BOTTOM_BASE + 48, TILE_GLAZED_SIDE_BASE + 12], true, true, false, false, 0, SoundFamily::Stone),
    d("Green Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 52, TILE_GLAZED_BOTTOM_BASE + 52, TILE_GLAZED_SIDE_BASE + 13], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 56, TILE_GLAZED_BOTTOM_BASE + 56, TILE_GLAZED_SIDE_BASE + 14], true, true, false, false, 0, SoundFamily::Stone),
    d("Black Glazed Terracotta", [TILE_GLAZED_TOP_BASE + 60, TILE_GLAZED_BOTTOM_BASE + 60, TILE_GLAZED_SIDE_BASE + 15], true, true, false, false, 0, SoundFamily::Stone),
    // 1.12 parrot spawn egg (kind 30)
    d("Parrot Spawn Egg", [TILE_PARROT_EGG, TILE_PARROT_EGG, TILE_PARROT_EGG], false, false, true, false, 0, SoundFamily::Stone),
    // 1.12-era dye items — the names are the pre-1.14 forms (the
    // "White Dye"/"Black Dye" renames are 1.14 17w45a — version-scoped
    // OUT of this bracket; VERIFIED w/Dye §History)
    d("Bone Meal", [TILE_DYE_BASE, TILE_DYE_BASE, TILE_DYE_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Orange Dye", [TILE_DYE_BASE + 1, TILE_DYE_BASE + 1, TILE_DYE_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Magenta Dye", [TILE_DYE_BASE + 2, TILE_DYE_BASE + 2, TILE_DYE_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Light Blue Dye", [TILE_DYE_BASE + 3, TILE_DYE_BASE + 3, TILE_DYE_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Dandelion Yellow", [TILE_DYE_BASE + 4, TILE_DYE_BASE + 4, TILE_DYE_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Lime Dye", [TILE_DYE_BASE + 5, TILE_DYE_BASE + 5, TILE_DYE_BASE + 5], false, false, true, false, 0, SoundFamily::Grass),
    d("Pink Dye", [TILE_DYE_BASE + 6, TILE_DYE_BASE + 6, TILE_DYE_BASE + 6], false, false, true, false, 0, SoundFamily::Grass),
    d("Gray Dye", [TILE_DYE_BASE + 7, TILE_DYE_BASE + 7, TILE_DYE_BASE + 7], false, false, true, false, 0, SoundFamily::Grass),
    d("Light Gray Dye", [TILE_DYE_BASE + 8, TILE_DYE_BASE + 8, TILE_DYE_BASE + 8], false, false, true, false, 0, SoundFamily::Grass),
    d("Cyan Dye", [TILE_DYE_BASE + 9, TILE_DYE_BASE + 9, TILE_DYE_BASE + 9], false, false, true, false, 0, SoundFamily::Grass),
    d("Purple Dye", [TILE_DYE_BASE + 10, TILE_DYE_BASE + 10, TILE_DYE_BASE + 10], false, false, true, false, 0, SoundFamily::Grass),
    d("Lapis Lazuli", [TILE_DYE_BASE + 11, TILE_DYE_BASE + 11, TILE_DYE_BASE + 11], false, false, true, false, 0, SoundFamily::Stone),
    d("Cocoa Beans", [TILE_DYE_BASE + 12, TILE_DYE_BASE + 12, TILE_DYE_BASE + 12], false, false, true, false, 0, SoundFamily::Grass),
    d("Cactus Green", [TILE_DYE_BASE + 13, TILE_DYE_BASE + 13, TILE_DYE_BASE + 13], false, false, true, false, 0, SoundFamily::Grass),
    d("Rose Red", [TILE_DYE_BASE + 14, TILE_DYE_BASE + 14, TILE_DYE_BASE + 14], false, false, true, false, 0, SoundFamily::Grass),
    d("Ink Sac", [TILE_DYE_BASE + 15, TILE_DYE_BASE + 15, TILE_DYE_BASE + 15], false, false, true, false, 0, SoundFamily::Grass),
    // the 4 parrot-taming seeds + the cookie
    d("Wheat Seeds", [TILE_SEEDS_BASE, TILE_SEEDS_BASE, TILE_SEEDS_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Melon Seeds", [TILE_SEEDS_BASE + 1, TILE_SEEDS_BASE + 1, TILE_SEEDS_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Pumpkin Seeds", [TILE_SEEDS_BASE + 2, TILE_SEEDS_BASE + 2, TILE_SEEDS_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Beetroot Seeds", [TILE_SEEDS_BASE + 3, TILE_SEEDS_BASE + 3, TILE_SEEDS_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Cookie", [TILE_COOKIE, TILE_COOKIE, TILE_COOKIE], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.13 (Update Aquatic): the V9 window ----
    // coral blocks: full cubes, hardness 1.5, stone-family sound
    d("Tube Coral Block", [TILE_CORAL_BLOCK_BASE, TILE_CORAL_BLOCK_BASE, TILE_CORAL_BLOCK_BASE], true, true, false, false, 0, SoundFamily::Stone),
    d("Brain Coral Block", [TILE_CORAL_BLOCK_BASE + 1, TILE_CORAL_BLOCK_BASE + 1, TILE_CORAL_BLOCK_BASE + 1], true, true, false, false, 0, SoundFamily::Stone),
    d("Bubble Coral Block", [TILE_CORAL_BLOCK_BASE + 2, TILE_CORAL_BLOCK_BASE + 2, TILE_CORAL_BLOCK_BASE + 2], true, true, false, false, 0, SoundFamily::Stone),
    d("Fire Coral Block", [TILE_CORAL_BLOCK_BASE + 3, TILE_CORAL_BLOCK_BASE + 3, TILE_CORAL_BLOCK_BASE + 3], true, true, false, false, 0, SoundFamily::Stone),
    d("Horn Coral Block", [TILE_CORAL_BLOCK_BASE + 4, TILE_CORAL_BLOCK_BASE + 4, TILE_CORAL_BLOCK_BASE + 4], true, true, false, false, 0, SoundFamily::Stone),
    d("Dead Tube Coral Block", [TILE_DEAD_CORAL_BLOCK_BASE, TILE_DEAD_CORAL_BLOCK_BASE, TILE_DEAD_CORAL_BLOCK_BASE], true, true, false, false, 0, SoundFamily::Stone),
    d("Dead Brain Coral Block", [TILE_DEAD_CORAL_BLOCK_BASE + 1, TILE_DEAD_CORAL_BLOCK_BASE + 1, TILE_DEAD_CORAL_BLOCK_BASE + 1], true, true, false, false, 0, SoundFamily::Stone),
    d("Dead Bubble Coral Block", [TILE_DEAD_CORAL_BLOCK_BASE + 2, TILE_DEAD_CORAL_BLOCK_BASE + 2, TILE_DEAD_CORAL_BLOCK_BASE + 2], true, true, false, false, 0, SoundFamily::Stone),
    d("Dead Fire Coral Block", [TILE_DEAD_CORAL_BLOCK_BASE + 3, TILE_DEAD_CORAL_BLOCK_BASE + 3, TILE_DEAD_CORAL_BLOCK_BASE + 3], true, true, false, false, 0, SoundFamily::Stone),
    d("Dead Horn Coral Block", [TILE_DEAD_CORAL_BLOCK_BASE + 4, TILE_DEAD_CORAL_BLOCK_BASE + 4, TILE_DEAD_CORAL_BLOCK_BASE + 4], true, true, false, false, 0, SoundFamily::Stone),
    // coral plants + fans: cross-rendered underwater plants
    d("Tube Coral", [TILE_CORAL_PLANT_BASE, TILE_CORAL_PLANT_BASE, TILE_CORAL_PLANT_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Brain Coral", [TILE_CORAL_PLANT_BASE + 1, TILE_CORAL_PLANT_BASE + 1, TILE_CORAL_PLANT_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Bubble Coral", [TILE_CORAL_PLANT_BASE + 2, TILE_CORAL_PLANT_BASE + 2, TILE_CORAL_PLANT_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Fire Coral", [TILE_CORAL_PLANT_BASE + 3, TILE_CORAL_PLANT_BASE + 3, TILE_CORAL_PLANT_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Horn Coral", [TILE_CORAL_PLANT_BASE + 4, TILE_CORAL_PLANT_BASE + 4, TILE_CORAL_PLANT_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Tube Coral", [TILE_DEAD_CORAL_PLANT_BASE, TILE_DEAD_CORAL_PLANT_BASE, TILE_DEAD_CORAL_PLANT_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Brain Coral", [TILE_DEAD_CORAL_PLANT_BASE + 1, TILE_DEAD_CORAL_PLANT_BASE + 1, TILE_DEAD_CORAL_PLANT_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Bubble Coral", [TILE_DEAD_CORAL_PLANT_BASE + 2, TILE_DEAD_CORAL_PLANT_BASE + 2, TILE_DEAD_CORAL_PLANT_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Fire Coral", [TILE_DEAD_CORAL_PLANT_BASE + 3, TILE_DEAD_CORAL_PLANT_BASE + 3, TILE_DEAD_CORAL_PLANT_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Horn Coral", [TILE_DEAD_CORAL_PLANT_BASE + 4, TILE_DEAD_CORAL_PLANT_BASE + 4, TILE_DEAD_CORAL_PLANT_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Tube Coral Fan", [TILE_CORAL_FAN_BASE, TILE_CORAL_FAN_BASE, TILE_CORAL_FAN_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Brain Coral Fan", [TILE_CORAL_FAN_BASE + 1, TILE_CORAL_FAN_BASE + 1, TILE_CORAL_FAN_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Bubble Coral Fan", [TILE_CORAL_FAN_BASE + 2, TILE_CORAL_FAN_BASE + 2, TILE_CORAL_FAN_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Fire Coral Fan", [TILE_CORAL_FAN_BASE + 3, TILE_CORAL_FAN_BASE + 3, TILE_CORAL_FAN_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Horn Coral Fan", [TILE_CORAL_FAN_BASE + 4, TILE_CORAL_FAN_BASE + 4, TILE_CORAL_FAN_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Tube Coral Fan", [TILE_DEAD_CORAL_FAN_BASE, TILE_DEAD_CORAL_FAN_BASE, TILE_DEAD_CORAL_FAN_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Brain Coral Fan", [TILE_DEAD_CORAL_FAN_BASE + 1, TILE_DEAD_CORAL_FAN_BASE + 1, TILE_DEAD_CORAL_FAN_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Bubble Coral Fan", [TILE_DEAD_CORAL_FAN_BASE + 2, TILE_DEAD_CORAL_FAN_BASE + 2, TILE_DEAD_CORAL_FAN_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Fire Coral Fan", [TILE_DEAD_CORAL_FAN_BASE + 3, TILE_DEAD_CORAL_FAN_BASE + 3, TILE_DEAD_CORAL_FAN_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Dead Horn Coral Fan", [TILE_DEAD_CORAL_FAN_BASE + 4, TILE_DEAD_CORAL_FAN_BASE + 4, TILE_DEAD_CORAL_FAN_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    // sea pickle: cross-rendered (its 1-4 count states carry the art)
    d("Sea Pickle", [TILE_SEA_PICKLE_BASE, TILE_SEA_PICKLE_BASE, TILE_SEA_PICKLE_BASE], false, false, true, false, 0, SoundFamily::Grass),
    // blue ice: the slipperiest block (slipperiness 0.989)
    d("Blue Ice", [TILE_BLUE_ICE, TILE_BLUE_ICE, TILE_BLUE_ICE], true, true, false, false, 0, SoundFamily::Glass),
    // dried kelp block: fuel (4000 ticks / 20 items)
    d("Dried Kelp Block", [TILE_DRIED_KELP_BLOCK, TILE_DRIED_KELP_BLOCK, TILE_DRIED_KELP_BLOCK], true, true, false, false, 0, SoundFamily::Grass),
    // kelp + seagrass: underwater cross plants
    d("Kelp", [TILE_KELP, TILE_KELP, TILE_KELP], false, false, true, false, 0, SoundFamily::Grass),
    d("Seagrass", [TILE_SEAGRASS, TILE_SEAGRASS, TILE_SEAGRASS], false, false, true, false, 0, SoundFamily::Grass),
    // conduit: light-15 beacon block (the frame powers it)
    d("Conduit", [TILE_CONDUIT, TILE_CONDUIT, TILE_CONDUIT], true, true, false, false, 15, SoundFamily::Stone),
    // turtle egg: solid-ish (renders as a small-block-adapted cube)
    d("Turtle Egg", [TILE_TURTLE_EGG_BASE, TILE_TURTLE_EGG_BASE, TILE_TURTLE_EGG_BASE], true, true, false, false, 0, SoundFamily::Stone),
    // ---- the 1.13 items (cross-rendered item icons, never placeable) ----
    d("Heart of the Sea", [TILE_HEART_OF_THE_SEA, TILE_HEART_OF_THE_SEA, TILE_HEART_OF_THE_SEA], false, false, true, false, 0, SoundFamily::Stone),
    d("Nautilus Shell", [TILE_NAUTILUS_SHELL, TILE_NAUTILUS_SHELL, TILE_NAUTILUS_SHELL], false, false, true, false, 0, SoundFamily::Stone),
    d("Scute", [TILE_SCUTE, TILE_SCUTE, TILE_SCUTE], false, false, true, false, 0, SoundFamily::Grass),
    d("Trident", [TILE_TRIDENT, TILE_TRIDENT, TILE_TRIDENT], false, false, true, false, 0, SoundFamily::Stone),
    d("Phantom Membrane", [TILE_PHANTOM_MEMBRANE, TILE_PHANTOM_MEMBRANE, TILE_PHANTOM_MEMBRANE], false, false, true, false, 0, SoundFamily::Grass),
    d("Dried Kelp", [TILE_DRIED_KELP, TILE_DRIED_KELP, TILE_DRIED_KELP], false, false, true, false, 0, SoundFamily::Grass),
    d("Turtle Shell", [TILE_TURTLE_SHELL, TILE_TURTLE_SHELL, TILE_TURTLE_SHELL], false, false, true, false, 0, SoundFamily::Stone),
    d("Potion of Slow Falling", [TILE_POTION_SLOW_FALLING, TILE_POTION_SLOW_FALLING, TILE_POTION_SLOW_FALLING], false, false, true, false, 0, SoundFamily::None),
    d("Potion of Slow Falling (extended)", [TILE_POTION_SLOW_FALLING_EXT, TILE_POTION_SLOW_FALLING_EXT, TILE_POTION_SLOW_FALLING_EXT], false, false, true, false, 0, SoundFamily::None),
    d("Potion of the Turtle Master", [TILE_POTION_TURTLE_MASTER, TILE_POTION_TURTLE_MASTER, TILE_POTION_TURTLE_MASTER], false, false, true, false, 0, SoundFamily::None),
    d("Potion of the Turtle Master (enhanced)", [TILE_POTION_TURTLE_MASTER_II, TILE_POTION_TURTLE_MASTER_II, TILE_POTION_TURTLE_MASTER_II], false, false, true, false, 0, SoundFamily::None),
    // ---- the 1.13 spawn eggs (kinds 32..=39) ----
    d("Drowned Spawn Egg", [TILE_EGG_V113_BASE, TILE_EGG_V113_BASE, TILE_EGG_V113_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Phantom Spawn Egg", [TILE_EGG_V113_BASE + 1, TILE_EGG_V113_BASE + 1, TILE_EGG_V113_BASE + 1], false, false, true, false, 0, SoundFamily::Grass),
    d("Dolphin Spawn Egg", [TILE_EGG_V113_BASE + 2, TILE_EGG_V113_BASE + 2, TILE_EGG_V113_BASE + 2], false, false, true, false, 0, SoundFamily::Grass),
    d("Cod Spawn Egg", [TILE_EGG_V113_BASE + 3, TILE_EGG_V113_BASE + 3, TILE_EGG_V113_BASE + 3], false, false, true, false, 0, SoundFamily::Grass),
    d("Salmon Spawn Egg", [TILE_EGG_V113_BASE + 4, TILE_EGG_V113_BASE + 4, TILE_EGG_V113_BASE + 4], false, false, true, false, 0, SoundFamily::Grass),
    d("Pufferfish Spawn Egg", [TILE_EGG_V113_BASE + 5, TILE_EGG_V113_BASE + 5, TILE_EGG_V113_BASE + 5], false, false, true, false, 0, SoundFamily::Grass),
    d("Tropical Fish Spawn Egg", [TILE_EGG_V113_BASE + 6, TILE_EGG_V113_BASE + 6, TILE_EGG_V113_BASE + 6], false, false, true, false, 0, SoundFamily::Grass),
    d("Turtle Spawn Egg", [TILE_EGG_V113_BASE + 7, TILE_EGG_V113_BASE + 7, TILE_EGG_V113_BASE + 7], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.14 bracket (Village & Pillage — nature half): ids 417..=425,
    // the V10 window. Campfire: solid, NOT opaque (a ~7/16-high partial
    // block — full-cube collision is the engine's standing partial-
    // geometry approximation; entities stand ON it, which is the damage
    // gate). Barrel: solid AND opaque (w/Barrel infobox "Transparent
    // No"). Bamboo renders as a cross (the kelp-column adaptation:
    // vanilla's 2-px stalk collision can't be expressed yet — disclosed).
    // Berry bush: cross, non-solid (walk-through — the slow/damage hook
    // lives in the movement paths) ----
    d("Bamboo", [TILE_BAMBOO, TILE_BAMBOO, TILE_BAMBOO], false, false, true, false, 0, SoundFamily::Wood),
    d("Bamboo Shoot", [TILE_BAMBOO_SHOOT, TILE_BAMBOO_SHOOT, TILE_BAMBOO_SHOOT], false, false, true, false, 0, SoundFamily::Grass),
    d("Sweet Berry Bush", [TILE_BERRY_BUSH_BASE, TILE_BERRY_BUSH_BASE, TILE_BERRY_BUSH_BASE], false, false, true, false, 0, SoundFamily::Grass),
    d("Campfire", [TILE_CAMPFIRE, TILE_CAMPFIRE, TILE_CAMPFIRE], true, false, false, false, 0, SoundFamily::Wood),
    d("Barrel", [TILE_BARREL_TOP, TILE_BARREL_TOP, TILE_BARREL_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Sweet Berries", [TILE_SWEET_BERRIES, TILE_SWEET_BERRIES, TILE_SWEET_BERRIES], false, false, true, false, 0, SoundFamily::Grass),
    d("Fox Spawn Egg", [TILE_EGG_FOX, TILE_EGG_FOX, TILE_EGG_FOX], false, false, true, false, 0, SoundFamily::Grass),
    d("Stick", [TILE_STICK, TILE_STICK, TILE_STICK], false, false, true, false, 0, SoundFamily::Wood),
    d("Charcoal", [TILE_CHARCOAL, TILE_CHARCOAL, TILE_CHARCOAL], false, false, true, false, 0, SoundFamily::Stone),
    // ---- 1.14 (Village & Pillage — nature half, part 2): the V11
    // window. VERIFIED w/Blast_Furnace + w/Smoker + w/Lantern
    // infoboxes (all "tool: wooden pickaxe" — the engine has no tool
    // tiers, the standing disclosed deferral; lantern light 15 = the
    // infobox "light: Yes (15)", brighter than the torch's 14). ----
    d("Blast Furnace", [TILE_FURNACE_TOP, TILE_FURNACE_TOP, TILE_BLAST_FURNACE_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("Smoker", [TILE_FURNACE_TOP, TILE_FURNACE_TOP, TILE_SMOKER_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Lantern", [TILE_LANTERN, TILE_LANTERN, TILE_LANTERN], false, false, true, false, 15, SoundFamily::Wood),
    d("Iron Nugget", [TILE_IRON_NUGGET, TILE_IRON_NUGGET, TILE_IRON_NUGGET], false, false, true, false, 0, SoundFamily::Stone),
    // ---- 1.14 (part 3): the two new small flowers — the allium
    // pattern (cross plant, non-solid, instant-break, drops itself;
    // VERIFIED w/Cornflower + w/Lily_of_the_Valley) ----
    d("Cornflower", [TILE_CORNFLOWER, TILE_CORNFLOWER, TILE_CORNFLOWER], false, false, true, false, 0, SoundFamily::Grass),
    d("Lily of the Valley", [TILE_LILY_OF_THE_VALLEY, TILE_LILY_OF_THE_VALLEY, TILE_LILY_OF_THE_VALLEY], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.15 (Buzzy Bees) — VERIFIED w/Beehive + w/Bee_nest +
    // w/Honey_Block + w/Honeycomb_Block: the nest/hive are full solid
    // wood-sound blocks with the entrance on the side tiles (the
    // furnace pattern); honey is solid but translucent (opaque false —
    // the JE "partial (diffuses sky light)" row); honeycomb block is
    // an opaque decorative. ----
    d("Bee Nest", [TILE_BEE_NEST_TOP, TILE_BEE_NEST_TOP, TILE_BEE_NEST_FRONT], true, true, false, false, 0, SoundFamily::Wood),
    d("Beehive", [TILE_BEEHIVE_TOP, TILE_BEEHIVE_TOP, TILE_BEEHIVE_FRONT], true, true, false, false, 0, SoundFamily::Wood),
    d("Honey Block", [TILE_HONEY, TILE_HONEY, TILE_HONEY], true, false, false, false, 0, SoundFamily::Grass),
    d("Honeycomb Block", [TILE_HONEYCOMB_BLOCK, TILE_HONEYCOMB_BLOCK, TILE_HONEYCOMB_BLOCK], true, true, false, false, 0, SoundFamily::Grass),
    // 1.15 items — the item-row pattern (non-placeable, cross-sprited)
    d("Honeycomb", [TILE_HONEYCOMB, TILE_HONEYCOMB, TILE_HONEYCOMB], false, false, true, false, 0, SoundFamily::Grass),
    d("Honey Bottle", [TILE_HONEY_BOTTLE, TILE_HONEY_BOTTLE, TILE_HONEY_BOTTLE], false, false, true, false, 0, SoundFamily::Glass),
    d("Shears", [TILE_SHEARS, TILE_SHEARS, TILE_SHEARS], false, false, true, false, 0, SoundFamily::Wood),
    d("Bee Spawn Egg", [TILE_SPAWN_EGG_BEE, TILE_SPAWN_EGG_BEE, TILE_SPAWN_EGG_BEE], false, false, true, false, 0, SoundFamily::Grass),
    // ---- 1.16 (Nether Update, part 1 — the V13 window): all VERIFIED
    // against the v116 captures. soul soil is the soul-sand-textured
    // fire host; basalt carries the pillar top/side pair; the anchor is
    // an opaque cube (charge art through state_tiles); the target a
    // full cube; chain + soul fire are non-solid decorations (chain
    // offsets like the lantern, soul fire is the cross-sprite) ----
    d("Soul Soil", [TILE_SOUL_SOIL, TILE_SOUL_SOIL, TILE_SOUL_SOIL], true, true, false, false, 0, SoundFamily::Sand),
    d("Basalt", [TILE_BASALT_TOP, TILE_BASALT_TOP, TILE_BASALT_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("Blackstone", [TILE_BLACKSTONE, TILE_BLACKSTONE, TILE_BLACKSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Gilded Blackstone", [TILE_GILDED_BLACKSTONE, TILE_GILDED_BLACKSTONE, TILE_GILDED_BLACKSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Crying Obsidian", [TILE_CRYING_OBSIDIAN, TILE_CRYING_OBSIDIAN, TILE_CRYING_OBSIDIAN], true, true, false, false, 10, SoundFamily::Stone),
    d("Respawn Anchor", [TILE_ANCHOR_TOP, TILE_ANCHOR_TOP, TILE_ANCHOR_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("Target", [TILE_TARGET, TILE_TARGET, TILE_TARGET], true, true, false, false, 0, SoundFamily::Grass),
    d("Nether Gold Ore", [TILE_NETHER_GOLD_ORE, TILE_NETHER_GOLD_ORE, TILE_NETHER_GOLD_ORE], true, true, false, false, 0, SoundFamily::Stone),
    d("Ancient Debris", [TILE_ANCIENT_DEBRIS_TOP, TILE_ANCIENT_DEBRIS_TOP, TILE_ANCIENT_DEBRIS_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("Block of Netherite", [TILE_NETHERITE_BLOCK, TILE_NETHERITE_BLOCK, TILE_NETHERITE_BLOCK], true, true, false, false, 0, SoundFamily::Stone),
    d("Chain", [TILE_CHAIN, TILE_CHAIN, TILE_CHAIN], false, false, true, false, 0, SoundFamily::Stone),
    d("Soul Fire", [TILE_SOUL_FIRE, TILE_SOUL_FIRE, TILE_SOUL_FIRE], false, false, true, false, 10, SoundFamily::Grass),
    // 1.16 items — the item-row pattern (non-placeable, cross-sprited)
    d("Netherite Scrap", [TILE_NETHERITE_SCRAP, TILE_NETHERITE_SCRAP, TILE_NETHERITE_SCRAP], false, false, true, false, 0, SoundFamily::Stone),
    d("Netherite Ingot", [TILE_NETHERITE_INGOT, TILE_NETHERITE_INGOT, TILE_NETHERITE_INGOT], false, false, true, false, 0, SoundFamily::Stone),
    // ---- 1.16 (Nether Update, part 2 — the crimson/warped families):
    // the V14 window. All VERIFIED against the v116b captures: the
    // stems/hyphae are log-class wood-sound cubes (top/side pairs);
    // nylium carries the grass-block top/side pair; the fungi/roots/
    // sprouts/vines are non-solid cross plants; shroomlight is the
    // light-15 lamp; the polished stones are stone cubes; the soul
    // torch + soul lantern are the light-10 soul-lit pair ----
    d("Crimson Stem", [TILE_CRIMSON_STEM_TOP, TILE_CRIMSON_STEM_TOP, TILE_CRIMSON_STEM_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Crimson Hyphae", [TILE_CRIMSON_HYPHAE, TILE_CRIMSON_HYPHAE, TILE_CRIMSON_HYPHAE], true, true, false, false, 0, SoundFamily::Wood),
    d("Crimson Planks", [TILE_CRIMSON_PLANKS, TILE_CRIMSON_PLANKS, TILE_CRIMSON_PLANKS], true, true, false, false, 0, SoundFamily::Wood),
    d("Crimson Nylium", [TILE_CRIMSON_NYLIUM_TOP, TILE_NETHERRACK, TILE_CRIMSON_NYLIUM_SIDE], true, true, false, false, 0, SoundFamily::Dirt),
    d("Crimson Fungus", [TILE_CRIMSON_FUNGUS, TILE_CRIMSON_FUNGUS, TILE_CRIMSON_FUNGUS], false, false, true, false, 0, SoundFamily::Grass),
    d("Crimson Roots", [TILE_CRIMSON_ROOTS, TILE_CRIMSON_ROOTS, TILE_CRIMSON_ROOTS], false, false, true, false, 0, SoundFamily::Grass),
    d("Weeping Vines", [TILE_WEEPING_VINES, TILE_WEEPING_VINES, TILE_WEEPING_VINES], false, false, true, false, 0, SoundFamily::Grass),
    d("Warped Stem", [TILE_WARPED_STEM_TOP, TILE_WARPED_STEM_TOP, TILE_WARPED_STEM_SIDE], true, true, false, false, 0, SoundFamily::Wood),
    d("Warped Hyphae", [TILE_WARPED_HYPHAE, TILE_WARPED_HYPHAE, TILE_WARPED_HYPHAE], true, true, false, false, 0, SoundFamily::Wood),
    d("Warped Planks", [TILE_WARPED_PLANKS, TILE_WARPED_PLANKS, TILE_WARPED_PLANKS], true, true, false, false, 0, SoundFamily::Wood),
    d("Warped Nylium", [TILE_WARPED_NYLIUM_TOP, TILE_NETHERRACK, TILE_WARPED_NYLIUM_SIDE], true, true, false, false, 0, SoundFamily::Dirt),
    d("Warped Fungus", [TILE_WARPED_FUNGUS, TILE_WARPED_FUNGUS, TILE_WARPED_FUNGUS], false, false, true, false, 0, SoundFamily::Grass),
    d("Warped Roots", [TILE_WARPED_ROOTS, TILE_WARPED_ROOTS, TILE_WARPED_ROOTS], false, false, true, false, 0, SoundFamily::Grass),
    d("Twisting Vines", [TILE_TWISTING_VINES, TILE_TWISTING_VINES, TILE_TWISTING_VINES], false, false, true, false, 0, SoundFamily::Grass),
    d("Warped Wart Block", [TILE_WARPED_WART_BLOCK, TILE_WARPED_WART_BLOCK, TILE_WARPED_WART_BLOCK], true, true, false, false, 0, SoundFamily::Wool),
    d("Shroomlight", [TILE_SHROOMLIGHT, TILE_SHROOMLIGHT, TILE_SHROOMLIGHT], true, true, false, false, 15, SoundFamily::Wool),
    d("Nether Sprouts", [TILE_NETHER_SPROUTS, TILE_NETHER_SPROUTS, TILE_NETHER_SPROUTS], false, false, true, false, 0, SoundFamily::Grass),
    d("Polished Basalt", [TILE_POLISHED_BASALT_TOP, TILE_POLISHED_BASALT_TOP, TILE_POLISHED_BASALT_SIDE], true, true, false, false, 0, SoundFamily::Stone),
    d("Polished Blackstone", [TILE_POLISHED_BLACKSTONE, TILE_POLISHED_BLACKSTONE, TILE_POLISHED_BLACKSTONE], true, true, false, false, 0, SoundFamily::Stone),
    d("Polished Blackstone Bricks", [TILE_POLISHED_BLACKSTONE_BRICKS, TILE_POLISHED_BLACKSTONE_BRICKS, TILE_POLISHED_BLACKSTONE_BRICKS], true, true, false, false, 0, SoundFamily::Stone),
    d("Soul Torch", [TILE_SOUL_TORCH, TILE_SOUL_TORCH, TILE_SOUL_TORCH], false, false, true, false, 10, SoundFamily::Wood),
    d("Soul Lantern", [TILE_SOUL_LANTERN, TILE_SOUL_LANTERN, TILE_SOUL_LANTERN], false, false, true, false, 10, SoundFamily::Wood),
    // 1.16 part-2 eggs — the item-row pattern (non-placeable,
    // cross-sprited, creative picker + right-click spawn)
    d("Strider Spawn Egg", [TILE_SPAWN_EGG_STRIDER, TILE_SPAWN_EGG_STRIDER, TILE_SPAWN_EGG_STRIDER], false, false, true, false, 0, SoundFamily::Grass),
    d("Piglin Spawn Egg", [TILE_SPAWN_EGG_PIGLIN, TILE_SPAWN_EGG_PIGLIN, TILE_SPAWN_EGG_PIGLIN], false, false, true, false, 0, SoundFamily::Grass),
    d("Hoglin Spawn Egg", [TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN], false, false, true, false, 0, SoundFamily::Grass),
    // ---- the 1.0-1.16.5 completeness audit: the V15 item rows ----
    // the cooked-meat family (the standing campfire.rs deferral, closed)
    d("Steak", [TILE_STEAK, TILE_STEAK, TILE_STEAK], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Porkchop", [TILE_COOKED_PORKCHOP, TILE_COOKED_PORKCHOP, TILE_COOKED_PORKCHOP], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Chicken", [TILE_COOKED_CHICKEN, TILE_COOKED_CHICKEN, TILE_COOKED_CHICKEN], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Mutton", [TILE_COOKED_MUTTON, TILE_COOKED_MUTTON, TILE_COOKED_MUTTON], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Cod", [TILE_COOKED_COD, TILE_COOKED_COD, TILE_COOKED_COD], false, false, true, false, 0, SoundFamily::Grass),
    d("Cooked Salmon", [TILE_COOKED_SALMON, TILE_COOKED_SALMON, TILE_COOKED_SALMON], false, false, true, false, 0, SoundFamily::Grass),
    // the kitchen chain
    d("Apple", [TILE_APPLE, TILE_APPLE, TILE_APPLE], false, false, true, false, 0, SoundFamily::Grass),
    d("Bowl", [TILE_BOWL, TILE_BOWL, TILE_BOWL], false, false, true, false, 0, SoundFamily::Wood),
    d("Mushroom Stew", [TILE_MUSHROOM_STEW, TILE_MUSHROOM_STEW, TILE_MUSHROOM_STEW], false, false, true, false, 0, SoundFamily::Grass),
    d("Rabbit Stew", [TILE_RABBIT_STEW, TILE_RABBIT_STEW, TILE_RABBIT_STEW], false, false, true, false, 0, SoundFamily::Grass),
    d("Beetroot", [TILE_BEETROOT, TILE_BEETROOT, TILE_BEETROOT], false, false, true, false, 0, SoundFamily::Grass),
    d("Beetroot Soup", [TILE_BEETROOT_SOUP, TILE_BEETROOT_SOUP, TILE_BEETROOT_SOUP], false, false, true, false, 0, SoundFamily::Grass),
    d("Sugar", [TILE_SUGAR, TILE_SUGAR, TILE_SUGAR], false, false, true, false, 0, SoundFamily::Grass),
    d("Egg", [TILE_EGG, TILE_EGG, TILE_EGG], false, false, true, false, 0, SoundFamily::Grass),
    d("Poisonous Potato", [TILE_POISONOUS_POTATO, TILE_POISONOUS_POTATO, TILE_POISONOUS_POTATO], false, false, true, false, 0, SoundFamily::Grass),
    d("Popped Chorus Fruit", [TILE_POPPED_CHORUS, TILE_POPPED_CHORUS, TILE_POPPED_CHORUS], false, false, true, false, 0, SoundFamily::Grass),
    d("Ghast Tear", [TILE_GHAST_TEAR, TILE_GHAST_TEAR, TILE_GHAST_TEAR], false, false, true, false, 0, SoundFamily::Grass),
    // the leaping + regeneration potion rows
    d("Potion of Leaping", [TILE_POTION_LEAPING, TILE_POTION_LEAPING, TILE_POTION_LEAPING], false, false, true, false, 0, SoundFamily::Grass),
    d("Potion of Leaping II", [TILE_POTION_LEAPING_II, TILE_POTION_LEAPING_II, TILE_POTION_LEAPING_II], false, false, true, false, 0, SoundFamily::Grass),
    d("Potion of Leaping (extended)", [TILE_POTION_LEAPING_LONG, TILE_POTION_LEAPING_LONG, TILE_POTION_LEAPING_LONG], false, false, true, false, 0, SoundFamily::Grass),
    d("Potion of Regeneration", [TILE_POTION_REGEN, TILE_POTION_REGEN, TILE_POTION_REGEN], false, false, true, false, 0, SoundFamily::Grass),
    d("Potion of Regeneration II", [TILE_POTION_REGEN_II, TILE_POTION_REGEN_II, TILE_POTION_REGEN_II], false, false, true, false, 0, SoundFamily::Grass),
    d("Potion of Regeneration (extended)", [TILE_POTION_REGEN_LONG, TILE_POTION_REGEN_LONG, TILE_POTION_REGEN_LONG], false, false, true, false, 0, SoundFamily::Grass),
    // the three classic mobs' eggs
    d("Ghast Spawn Egg", [TILE_SPAWN_EGG_GHAST, TILE_SPAWN_EGG_GHAST, TILE_SPAWN_EGG_GHAST], false, false, true, false, 0, SoundFamily::Grass),
    d("Cave Spider Spawn Egg", [TILE_SPAWN_EGG_CAVESPIDER, TILE_SPAWN_EGG_CAVESPIDER, TILE_SPAWN_EGG_CAVESPIDER], false, false, true, false, 0, SoundFamily::Grass),
    d("Silverfish Spawn Egg", [TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH], false, false, true, false, 0, SoundFamily::Grass),
    d("Melon Slice", [TILE_MELON_SLICE, TILE_MELON_SLICE, TILE_MELON_SLICE], false, false, true, false, 0, SoundFamily::Grass),
    // ---- backlog round (weather, 2026-09-09): the fire block ----
    // VERIFIED w/Weather §Lightning: lightning "creating fires where it
    // strikes, igniting any nearby flammable materials, but the rain
    // usually puts the fire out before it can spread". Cross-rendered
    // like soul fire, emissive 15 (vanilla fire light level), burns out
    // on random ticks — the burnout timer lives in the game layer's
    // random-tick hook (rain-accelerated).
    d("Fire", [TILE_FIRE, TILE_FIRE, TILE_FIRE], false, false, true, false, 15, SoundFamily::Grass),
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
pub const PICKER_BLOCKS: [u16; 459] = [
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

    SHULKER_BOX, SHULKER_SHELL, TOTEM_OF_UNDYING,
    SPAWN_EGG_LLAMA, SPAWN_EGG_VINDICATOR, SPAWN_EGG_EVOKER, SPAWN_EGG_VEX,
    SPAWN_EGG_HUSK, SPAWN_EGG_STRAY,
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

    // ---- 1.12 (World of Color Update): concrete 16 + powder 16 +
    // glazed terracotta 16 + the parrot egg + the 16 dyes + 4 seeds +
    // cookie (the engine's single creative palette — vanilla's
    // "Materials merged with miscellaneous" tab-change is N/A) ----
    CONCRETE_BASE, CONCRETE_BASE + 1, CONCRETE_BASE + 2, CONCRETE_BASE + 3,
    CONCRETE_BASE + 4, CONCRETE_BASE + 5, CONCRETE_BASE + 6, CONCRETE_BASE + 7,
    CONCRETE_BASE + 8, CONCRETE_BASE + 9, CONCRETE_BASE + 10, CONCRETE_BASE + 11,
    CONCRETE_BASE + 12, CONCRETE_BASE + 13, CONCRETE_BASE + 14, CONCRETE_BASE + 15,
    CONCRETE_POWDER_BASE, CONCRETE_POWDER_BASE + 1, CONCRETE_POWDER_BASE + 2,
    CONCRETE_POWDER_BASE + 3, CONCRETE_POWDER_BASE + 4, CONCRETE_POWDER_BASE + 5,
    CONCRETE_POWDER_BASE + 6, CONCRETE_POWDER_BASE + 7, CONCRETE_POWDER_BASE + 8,
    CONCRETE_POWDER_BASE + 9, CONCRETE_POWDER_BASE + 10, CONCRETE_POWDER_BASE + 11,
    CONCRETE_POWDER_BASE + 12, CONCRETE_POWDER_BASE + 13, CONCRETE_POWDER_BASE + 14,
    CONCRETE_POWDER_BASE + 15,
    GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_BASE + 1, GLAZED_TERRACOTTA_BASE + 2,
    GLAZED_TERRACOTTA_BASE + 3, GLAZED_TERRACOTTA_BASE + 4, GLAZED_TERRACOTTA_BASE + 5,
    GLAZED_TERRACOTTA_BASE + 6, GLAZED_TERRACOTTA_BASE + 7, GLAZED_TERRACOTTA_BASE + 8,
    GLAZED_TERRACOTTA_BASE + 9, GLAZED_TERRACOTTA_BASE + 10, GLAZED_TERRACOTTA_BASE + 11,
    GLAZED_TERRACOTTA_BASE + 12, GLAZED_TERRACOTTA_BASE + 13, GLAZED_TERRACOTTA_BASE + 14,
    GLAZED_TERRACOTTA_BASE + 15,
    SPAWN_EGG_PARROT,
    DYE_BASE, DYE_BASE + 1, DYE_BASE + 2, DYE_BASE + 3, DYE_BASE + 4,
    DYE_BASE + 5, DYE_BASE + 6, DYE_BASE + 7, DYE_BASE + 8, DYE_BASE + 9,
    DYE_BASE + 10, DYE_BASE + 11, DYE_BASE + 12, DYE_BASE + 13, DYE_BASE + 14,
    DYE_BASE + 15,
    WHEAT_SEEDS, MELON_SEEDS, PUMPKIN_SEEDS, BEETROOT_SEEDS,
    COOKIE,
    // ---- 1.13 (Update Aquatic): the V9 window — coral families 30 +
    // the sea blocks + the craft items + potions + 8 spawn eggs (the
    // picker-gap fix that rode along with the 1.14 window: the 1.13
    // rounds shipped the blocks but never the picker rows) ----
    CORAL_BLOCK_BASE, CORAL_BLOCK_BASE + 1, CORAL_BLOCK_BASE + 2,
    CORAL_BLOCK_BASE + 3, CORAL_BLOCK_BASE + 4,
    DEAD_CORAL_BLOCK_BASE, DEAD_CORAL_BLOCK_BASE + 1, DEAD_CORAL_BLOCK_BASE + 2,
    DEAD_CORAL_BLOCK_BASE + 3, DEAD_CORAL_BLOCK_BASE + 4,
    CORAL_PLANT_BASE, CORAL_PLANT_BASE + 1, CORAL_PLANT_BASE + 2,
    CORAL_PLANT_BASE + 3, CORAL_PLANT_BASE + 4,
    DEAD_CORAL_PLANT_BASE, DEAD_CORAL_PLANT_BASE + 1, DEAD_CORAL_PLANT_BASE + 2,
    DEAD_CORAL_PLANT_BASE + 3, DEAD_CORAL_PLANT_BASE + 4,
    CORAL_FAN_BASE, CORAL_FAN_BASE + 1, CORAL_FAN_BASE + 2,
    CORAL_FAN_BASE + 3, CORAL_FAN_BASE + 4,
    DEAD_CORAL_FAN_BASE, DEAD_CORAL_FAN_BASE + 1, DEAD_CORAL_FAN_BASE + 2,
    DEAD_CORAL_FAN_BASE + 3, DEAD_CORAL_FAN_BASE + 4,
    SEA_PICKLE, BLUE_ICE, DRIED_KELP_BLOCK, KELP, SEAGRASS, CONDUIT, TURTLE_EGG,
    HEART_OF_THE_SEA, NAUTILUS_SHELL, SCUTE, TRIDENT, PHANTOM_MEMBRANE,
    DRIED_KELP, TURTLE_SHELL,
    POTION_SLOW_FALLING, POTION_SLOW_FALLING_EXT,
    POTION_TURTLE_MASTER, POTION_TURTLE_MASTER_II,
    SPAWN_EGG_DROWNED, SPAWN_EGG_PHANTOM, SPAWN_EGG_DOLPHIN, SPAWN_EGG_COD,
    SPAWN_EGG_SALMON, SPAWN_EGG_PUFFERFISH, SPAWN_EGG_TROPICAL_FISH,
    SPAWN_EGG_TURTLE,
    // ---- 1.14 (Village & Pillage — nature half): the V10 window ----
    BAMBOO, BAMBOO_SHOOT, SWEET_BERRY_BUSH, CAMPFIRE, BARREL,
    SWEET_BERRIES, SPAWN_EGG_FOX, STICK, CHARCOAL,
    // ---- 1.14 (part 2): the V11 window — the smelters + lantern ----
    BLAST_FURNACE, SMOKER, LANTERN,
    // ---- 1.14 (part 3): the two new small flowers ----
    CORNFLOWER, LILY_OF_THE_VALLEY,
    // 1.15 (Buzzy Bees): the nest/hive/honey/honeycomb blocks (the
    // comb/bottle/shears/egg items are item-blocks, never placeable)
    BEE_NEST, BEEHIVE, HONEY_BLOCK, HONEYCOMB_BLOCK,
    // ---- 1.16 (Nether Update, part 1 — the anchor family): the V13
    // window's 12 placeable blocks (the scrap/ingot material items
    // are item-blocks, never placeable — the standing convention;
    // soul fire is the engine's only fire block, so the picker is the
    // one manual placement path — the disclosed no-flint adaptation) ----
    SOUL_SOIL, BASALT, BLACKSTONE, GILDED_BLACKSTONE, CRYING_OBSIDIAN,
    RESPAWN_ANCHOR, TARGET, NETHER_GOLD_ORE, ANCIENT_DEBRIS,
    NETHERITE_BLOCK, CHAIN, SOUL_FIRE,
    // ---- 1.16 (Nether Update, part 2 — the crimson/warped families):
    // the V14 window's 22 placeable blocks + the 3 spawn eggs (the
    // standing egg-picker convention since the E1 window) ----
    CRIMSON_STEM, CRIMSON_HYPHAE, CRIMSON_PLANKS, CRIMSON_NYLIUM,
    CRIMSON_FUNGUS, CRIMSON_ROOTS, WEEPING_VINES,
    WARPED_STEM, WARPED_HYPHAE, WARPED_PLANKS, WARPED_NYLIUM,
    WARPED_FUNGUS, WARPED_ROOTS, TWISTING_VINES, WARPED_WART_BLOCK,
    SHROOMLIGHT, NETHER_SPROUTS,
    POLISHED_BASALT, POLISHED_BLACKSTONE, POLISHED_BLACKSTONE_BRICKS,
    SOUL_TORCH, SOUL_LANTERN,
    SPAWN_EGG_STRIDER, SPAWN_EGG_PIGLIN, SPAWN_EGG_HOGLIN,
    // ---- the 1.0-1.16.5 completeness audit: the V15 items ----
    STEAK, COOKED_PORKCHOP, COOKED_CHICKEN, COOKED_MUTTON, COOKED_COD, COOKED_SALMON,
    APPLE, BOWL, MUSHROOM_STEW, RABBIT_STEW, BEETROOT, BEETROOT_SOUP, SUGAR, EGG,
    MELON_SLICE,
    POISONOUS_POTATO, POPPED_CHORUS_FRUIT, GHAST_TEAR,
    POTION_LEAPING, POTION_LEAPING_II, POTION_LEAPING_LONG,
    POTION_REGEN, POTION_REGEN_II, POTION_REGEN_LONG,
    SPAWN_EGG_GHAST, SPAWN_EGG_CAVE_SPIDER, SPAWN_EGG_SILVERFISH,
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
                || s == SPAWNER_VINDICATOR
                || s == SPAWNER_EVOKER
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
                || is_v7_state(s)
                // 1.12 V8 (World of Color Update)
                || is_v8_state(s)
                || is_v9_state(s)
                // 1.14 V10 (Village & Pillage — nature half)
                || is_v10_state(s)
                // 1.14 V11 (nature half, part 2)
                || is_v11_state(s)
                // 1.15 V12 (Buzzy Bees)
                || is_v12_state(s)
                // 1.16 V13 (Nether Update, part 1)
                || is_v13_state(s)
                // 1.16 V14 (Nether Update, part 2)
                || is_v14_state(s)
                // the completeness audit V15
                || is_v15_state(s)
        || is_v16_state(s)
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
                // 1.11 V7 (Exploration Update): same roundtrip contract
                if is_v7_state(s) {
                    assert_eq!(default_state(b), s, "v7 state {s} roundtrip");
                }
                // 1.12 V8 (World of Color): 1:1 ids roundtrip; glazed
                // facing states (4 per color) decode consistently —
                // only facing 0 equals default_state (the placement
                // path writes the player-facing state)
                if is_v8_state(s) {
                    if let Some((color, facing)) = glazed_decode(s) {
                        assert_eq!(
                            state_block(s),
                            glazed_terracotta(color),
                            "glazed state {s} folds to color {color}"
                        );
                        if facing == 0 {
                            assert_eq!(
                                default_state(glazed_terracotta(color)),
                                s,
                                "glazed color {color} default = facing 0"
                            );
                        }
                    } else {
                        assert_eq!(default_state(b), s, "v8 state {s} roundtrip");
                    }
                }
                // 1.13 V9 (Update Aquatic): 1:1 ids roundtrip; the sea
                // pickle count states (4) and turtle egg hatch stages
                // (3) fold to their parent — only the default (count 1
                // / stage 0) equals default_state (the growth path
                // writes the advanced states)
                if is_v9_state(s) {
                    if (V9_STATE_BASE + 30..=V9_STATE_BASE + 33).contains(&s) {
                        assert_eq!(state_block(s), SEA_PICKLE);
                        if s == V9_STATE_BASE + 30 {
                            assert_eq!(default_state(SEA_PICKLE), s);
                        }
                    } else if (V9_STATE_BASE + 39..=V9_STATE_BASE + 41).contains(&s) {
                        assert_eq!(state_block(s), TURTLE_EGG);
                        if s == V9_STATE_BASE + 39 {
                            assert_eq!(default_state(TURTLE_EGG), s);
                        }
                    } else {
                        assert_eq!(default_state(b), s, "v9 state {s} roundtrip");
                    }
                }
                // 1.14 V10 (Village & Pillage nature half): 1:1 ids
                // roundtrip; the bush's 4 age states and the campfire's
                // unlit state fold to their parent (campfire places LIT —
                // the extinguished state is written by the water path)
                if is_v10_state(s) {
                    if (V10_STATE_BASE + 2..=V10_STATE_BASE + 5).contains(&s) {
                        assert_eq!(state_block(s), SWEET_BERRY_BUSH);
                        if s == V10_STATE_BASE + 2 {
                            assert_eq!(default_state(SWEET_BERRY_BUSH), s);
                        }
                    } else if s == V10_STATE_BASE + 6 {
                        assert_eq!(state_block(s), CAMPFIRE);
                    } else {
                        assert_eq!(default_state(b), s, "v10 state {s} roundtrip");
                    }
                }
                // 1.14 V11 (nature half, part 2): every state folds to
                // its parent; the defaults roundtrip (smelters UNLIT,
                // lantern SITTING, nugget item state)
                if is_v11_state(s) {
                    assert_eq!(state_block(s), V11_STATE_TO_BLOCK[(s - V11_STATE_BASE) as usize]);
                    if let Some(db) = v11_state(b) {
                        assert_eq!(default_state(b), db, "v11 default for {b}");
                    }
                }
                // 1.15 V12 (Buzzy Bees): every state folds to its
                // parent; honey_level decodes on the 12 hive states
                // (level 0 default roundtrip); the identity + item
                // states 1:1
                if is_v12_state(s) {
                    assert_eq!(state_block(s), V12_STATE_TO_BLOCK[(s - V12_STATE_BASE) as usize]);
                    if s < V12_STATE_BASE + 12 {
                        let want = hive_state(state_block(s), honey_level(s));
                        assert_eq!(s, want, "hive state {s} re-encodes");
                    } else if let Some(db) = v12_state(b) {
                        assert_eq!(default_state(b), db, "v12 default for {b}");
                    }
                }
                // 1.16 V13 (Nether Update, part 1): every state folds to
                // its parent; the anchor's charge + the target's power
                // re-encode; the defaults roundtrip (charge 0, power 0,
                // chain SITTING, identities + items 1:1)
                if is_v13_state(s) {
                    assert_eq!(state_block(s), V13_STATE_TO_BLOCK[(s - V13_STATE_BASE) as usize]);
                    if s < V13_STATE_BASE + 5 {
                        let want = anchor_state(anchor_charge(s));
                        assert_eq!(s, want, "anchor state {s} re-encodes");
                    } else if (V13_STATE_BASE + 5..V13_STATE_BASE + 21).contains(&s) {
                        let want = target_state(target_power(s));
                        assert_eq!(s, want, "target state {s} re-encodes");
                    } else if let Some(db) = v13_state(b) {
                        assert_eq!(default_state(b), db, "v13 default for {b}");
                    }
                }
                // 1.16 V14 (Nether Update, part 2): every state folds
                // 1:1 (identity + the soul lantern's hanging form + the
                // 3 egg rows); the defaults roundtrip (lantern SITTING,
                // the rest 1:1)
                if is_v14_state(s) {
                    assert_eq!(state_block(s), V14_STATE_TO_BLOCK[(s - V14_STATE_BASE) as usize]);
                    if let Some(db) = v14_state(b) {
                        assert_eq!(default_state(b), db, "v14 default for {b}");
                    }
                }
                // the completeness audit V15: 26 identity item folds +
                // the two spawner states folding to the Monster
                // Spawner block; the defaults roundtrip 1:1
                if is_v15_state(s) {
                    assert_eq!(state_block(s), V15_STATE_TO_BLOCK[(s - V15_STATE_BASE) as usize]);
                    if let Some(db) = v15_state(b) {
                        assert_eq!(default_state(b), db, "v15 default for {b}");
                    }
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
        assert_eq!(BLOCK_COUNT, 507, "merged registry + V6..V14 + the audit V15 window + the backlog fire");
        assert_eq!(STATE_COUNT, 806, "merged state space + the backlog V16 window (805 = fire)");
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
        assert_eq!(BLOCK_COUNT, 507); // + the backlog fire (block windows are cumulative)
        assert_eq!(STATE_COUNT, 806); // + the backlog V16 fire state (state windows are cumulative)
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
        assert_eq!(BLOCK_COUNT, 507); // + the backlog fire (block windows are cumulative)
        assert_eq!(STATE_COUNT, 806); // + the backlog V16 fire state (state windows are cumulative)
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

// ---------------------------------------------------------------------------
// 1.11 bracket tests (Exploration Update, live 2026-09-07)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v111_tests {
    use super::*;

    /// the V7 window (ids 282..=290, states 486..=494) + the mansion
    /// spawner states 495..=496 (the interrupted round had them at
    /// 493..=494; the completion round's re-added husk/stray eggs
    /// extended the V7 window past that point and pushed the spawner
    /// states up — nothing was ever pushed under the old numbering)
    #[test]
    fn v111_v7_registry_and_spawner_states() {
        for (b, s) in [
            (SHULKER_BOX, 486u16),
            (SHULKER_SHELL, 487),
            (TOTEM_OF_UNDYING, 488),
            (SPAWN_EGG_LLAMA, 489),
            (SPAWN_EGG_VINDICATOR, 490),
            (SPAWN_EGG_EVOKER, 491),
            (SPAWN_EGG_VEX, 492),
            (SPAWN_EGG_HUSK, 493),
            (SPAWN_EGG_STRAY, 494),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
        }
        assert_eq!(BLOCK_COUNT, 507); // + the backlog fire (block windows are cumulative)
        assert_eq!(STATE_COUNT, 806); // + the backlog V16 fire state (state windows are cumulative)
        // mansion spawner states fold to SPAWNER + decode their kinds
        assert_eq!(state_block(SPAWNER_VINDICATOR), SPAWNER);
        assert_eq!(state_block(SPAWNER_EVOKER), SPAWNER);
        assert_eq!(spawner_mob(SPAWNER_VINDICATOR), 5);
        assert_eq!(spawner_mob(SPAWNER_EVOKER), 6);
        // THE LATENT-BUG FIX: the fortress spawner states decode to their
        // kinds (they previously fell through to the zombie code!)
        assert_eq!(spawner_mob(SPAWNER_BLAZE), 3, "fortress blaze decodes");
        assert_eq!(spawner_mob(SPAWNER_WITHER_SKELETON), 4, "fortress wither-skeleton decodes");
        // eggs decode to the 1.11 kinds (new five + re-added two)
        assert_eq!(egg_mob(SPAWN_EGG_LLAMA), Some(23));
        assert_eq!(egg_mob(SPAWN_EGG_VINDICATOR), Some(24));
        assert_eq!(egg_mob(SPAWN_EGG_EVOKER), Some(25));
        assert_eq!(egg_mob(SPAWN_EGG_VEX), Some(26));
        assert_eq!(egg_mob(SPAWN_EGG_HUSK), Some(27), "re-added husk egg");
        assert_eq!(egg_mob(SPAWN_EGG_STRAY), Some(28), "re-added stray egg");
        // the zombie-villager egg (the changelog's 5th new egg) is the
        // pre-existing E2-era item at id 129 — anachronistic but present
        assert_eq!(egg_mob(SPAWN_EGG_BASE + 5), Some(5));
        // all six V7 eggs are usable eggs (the use-path gate)
        for b in [
            SPAWN_EGG_LLAMA, SPAWN_EGG_VINDICATOR, SPAWN_EGG_EVOKER,
            SPAWN_EGG_VEX, SPAWN_EGG_HUSK, SPAWN_EGG_STRAY,
        ] {
            assert!(is_spawn_egg(b), "egg {b} must pass the use gate");
        }
        // item-blocks + picker
        assert!(is_item_block(SHULKER_SHELL));
        assert!(is_item_block(TOTEM_OF_UNDYING));
        for b in [SHULKER_BOX, SHULKER_SHELL, TOTEM_OF_UNDYING,
                  SPAWN_EGG_LLAMA, SPAWN_EGG_VINDICATOR, SPAWN_EGG_EVOKER, SPAWN_EGG_VEX,
                  SPAWN_EGG_HUSK, SPAWN_EGG_STRAY] {
            assert!(PICKER_BLOCKS.contains(&b), "picker missing {b}");
        }
        // egg items render the EGG tiles (the E1/E2/E3 convention), not
        // the mob billboard sprites
        assert_eq!(BLOCK_TABLE[SPAWN_EGG_LLAMA as usize].tiles[0], TILE_V7_EGG_BASE);
        assert_eq!(BLOCK_TABLE[SPAWN_EGG_STRAY as usize].tiles[0], TILE_V7_EGG_BASE + 5);
        assert!(TILE_V7_EGG_END <= TILE_MAX, "egg tiles within the atlas guard");
        // shulker box is a solid placeable container
        assert!(is_solid(SHULKER_BOX) && is_opaque(SHULKER_BOX));
    }
}

// ---------------------------------------------------------------------------
// 1.12 bracket tests (World of Color Update, live round 2026-09-07)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v112_tests {
    use super::*;

    /// the V8 window: 70 ids (16 concrete + 16 powder + 16 glazed + egg
    /// + 16 dyes + 4 seeds + cookie) with their default states, plus the
    /// registry/state-space bounds (VERIFIED live: minecraft.wiki
    /// /w/Java_Edition_1.12 §Blocks/§Items)
    #[test]
    fn v112_v8_registry_window() {
        // concrete: 1:1, colors 0..15
        for c in 0u16..16 {
            assert_eq!(concrete(c as u8), CONCRETE_BASE + c);
            assert_eq!(default_state(CONCRETE_BASE + c), concrete_state(c as u8));
            assert_eq!(state_block(concrete_state(c as u8)), CONCRETE_BASE + c);
        }
        // powder: 1:1
        for c in 0u16..16 {
            assert_eq!(concrete_powder(c as u8), CONCRETE_POWDER_BASE + c);
            assert_eq!(
                default_state(CONCRETE_POWDER_BASE + c),
                concrete_powder_state(c as u8)
            );
            assert_eq!(
                state_block(concrete_powder_state(c as u8)),
                CONCRETE_POWDER_BASE + c
            );
        }
        // glazed: 4 facing states per color, folding to the color block
        for c in 0u16..16 {
            for f in 0u16..4 {
                let s = glazed_terracotta_state(c as u8, f as u8);
                assert_eq!(glazed_decode(s), Some((c as u8, f as u8)));
                assert_eq!(state_block(s), GLAZED_TERRACOTTA_BASE + c);
            }
            // default = facing 0 (the placement path writes player-facing)
            assert_eq!(
                default_state(GLAZED_TERRACOTTA_BASE + c),
                glazed_terracotta_state(c as u8, 0)
            );
        }
        // items: parrot egg + dyes + seeds + cookie, 1:1 at offset 96+
        assert_eq!(default_state(SPAWN_EGG_PARROT), V8_STATE_BASE + 96);
        for c in 0u16..16 {
            assert_eq!(dye(c as u8), DYE_BASE + c);
            assert_eq!(default_state(DYE_BASE + c), V8_STATE_BASE + 97 + c);
        }
        for (i, b) in [WHEAT_SEEDS, MELON_SEEDS, PUMPKIN_SEEDS, BEETROOT_SEEDS]
            .iter()
            .enumerate()
        {
            assert_eq!(default_state(*b), V8_STATE_BASE + 113 + i as u16);
            assert!(is_seeds(*b), "seeds gate covers {b}");
        }
        assert_eq!(default_state(COOKIE), V8_STATE_BASE + 117);
        // bounds
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
        assert_eq!(CONCRETE_BASE + 15, CONCRETE_END);
        assert_eq!(CONCRETE_POWDER_BASE + 15, CONCRETE_POWDER_END);
        assert_eq!(GLAZED_TERRACOTTA_BASE + 15, GLAZED_TERRACOTTA_END);
        assert_eq!(DYE_BASE + 15, DYE_END);
    }

    /// concrete/powder/glazed physical flags + the sound families
    /// (VERIFIED live: w/Concrete 1.8 stone; w/Concrete_Powder §Sounds
    /// "block.sand.*"; w/Glazed_Terracotta §Sounds "block.stone.*")
    #[test]
    fn v112_block_flags_and_sounds() {
        for b in [CONCRETE_BASE, CONCRETE_BASE + 7, CONCRETE_END] {
            let d = def(b);
            assert!(d.solid && d.opaque, "concrete is a solid opaque cube");
            assert_eq!(d.sound, SoundFamily::Stone);
            assert!(!is_item_block(b));
        }
        for b in [CONCRETE_POWDER_BASE, CONCRETE_POWDER_END] {
            let d = def(b);
            assert!(d.solid && d.opaque, "powder is a solid opaque cube");
            assert_eq!(d.sound, SoundFamily::Sand, "powder: block.sand.* family");
        }
        for b in [GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_END] {
            let d = def(b);
            assert!(d.solid && d.opaque, "glazed is a solid opaque cube");
            assert_eq!(d.sound, SoundFamily::Stone);
            // glazed facing states are NOT model states (greedy cubes —
            // the V8 arm in is_model_state)
            let s = glazed_terracotta_state(0, 2);
            assert!(!is_model_state(s));
        }
        // item-blocks: never placeable, picker-visible
        assert!(is_item_block(COOKIE));
        assert!(is_item_block(DYE_BASE + 11)); // Lapis Lazuli
        assert!(is_item_block(WHEAT_SEEDS));
        assert!(!is_item_block(CONCRETE_BASE));
        // the parrot egg passes the use gate + decodes kind 30
        assert!(is_spawn_egg(SPAWN_EGG_PARROT));
        assert_eq!(egg_mob(SPAWN_EGG_PARROT), Some(30));
    }

    /// the 1.12 names — the dye names are the pre-1.14 forms (VERIFIED
    /// w/Dye §History: the "White Dye"/"Black Dye" renames are 1.14
    /// 17w45a, version-scoped out of this bracket)
    #[test]
    fn v112_names_and_tiles() {
        assert_eq!(name(CONCRETE_BASE), "White Concrete");
        assert_eq!(name(CONCRETE_POWDER_BASE), "White Concrete Powder");
        assert_eq!(name(GLAZED_TERRACOTTA_BASE), "White Glazed Terracotta");
        assert_eq!(name(DYE_BASE), "Bone Meal");
        assert_eq!(name(DYE_BASE + 11), "Lapis Lazuli");
        assert_eq!(name(DYE_BASE + 15), "Ink Sac");
        assert_eq!(name(WHEAT_SEEDS), "Wheat Seeds");
        assert_eq!(name(COOKIE), "Cookie");
        // glazed facing picks the ROTATED tile variant (top/bottom), the
        // shared side tile (VERIFIED w/Glazed_Terracotta §Placement: "the
        // texture rotates relative to the direction the player is facing")
        for f in 0u16..4 {
            let t = state_tiles(glazed_terracotta_state(0, f as u8));
            assert_eq!(t[0], TILE_GLAZED_TOP_BASE + f, "top rotation {f}");
            assert_eq!(t[1], TILE_GLAZED_BOTTOM_BASE + f, "bottom rotation {f}");
            assert_eq!(t[2], TILE_GLAZED_SIDE_BASE, "shared side");
        }
        // every 1.12 id is in the creative picker
        for b in [
            CONCRETE_BASE, CONCRETE_END, CONCRETE_POWDER_BASE, CONCRETE_POWDER_END,
            GLAZED_TERRACOTTA_BASE, GLAZED_TERRACOTTA_END, SPAWN_EGG_PARROT,
            DYE_BASE, DYE_END, WHEAT_SEEDS, BEETROOT_SEEDS, COOKIE,
        ] {
            assert!(PICKER_BLOCKS.contains(&b), "picker missing {b}");
        }
        assert!(TILE_MAX >= TILE_ILLUSIONER, "1.12 tiles within the atlas guard");
        assert_eq!(PICKER_BLOCKS.len(), 459);
        // the V9 + V10 windows are all present (the picker-gap fix)
        for want in [SEA_PICKLE, CONDUIT, SPAWN_EGG_TURTLE, BAMBOO, CAMPFIRE, BARREL, SPAWN_EGG_FOX, STICK, CHARCOAL] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
    }
}

// ---------------------------------------------------------------------------
// 1.14 bracket tests (Village & Pillage — nature half, live round 2026-09-08)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v114_tests {
    use super::*;

    /// the V10 window: ids 417..=425, states 676..=688. Blocks fold 1:1
    /// except the berry bush's 4 age states and the campfire's 2 lit
    /// states (VERIFIED from the raw captures v114_page_*.json).
    #[test]
    fn v114_v10_registry_window() {
        // 1:1 blocks (shoot/stalk/barrel/items)
        for (b, s) in [
            (BAMBOO, V10_STATE_BASE),
            (BAMBOO_SHOOT, V10_STATE_BASE + 1),
            (BARREL, V10_STATE_BASE + 8),
            (SWEET_BERRIES, V10_STATE_BASE + 9),
            (SPAWN_EGG_FOX, V10_STATE_BASE + 10),
            (STICK, V10_STATE_BASE + 11),
            (CHARCOAL, V10_STATE_BASE + 12),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
            assert!(is_v10_state(s));
        }
        // berry bush: 4 age states, age decode/encode roundtrip
        for age in 0u8..4 {
            let s = berry_bush_state(age);
            assert_eq!(berry_bush_age(s), age, "age {age} roundtrip");
            assert_eq!(state_block(s), SWEET_BERRY_BUSH, "bush state {s} folds");
            assert_eq!(state_tiles(s)[0], TILE_BERRY_BUSH_BASE + age as u16, "per-age art");
        }
        assert_eq!(default_state(SWEET_BERRY_BUSH), berry_bush_state(0));
        // clamped encode
        assert_eq!(berry_bush_state(9), berry_bush_state(3));
        // campfire: unlit/lit pair, placed LIT (vanilla), light on lit only
        assert_eq!(campfire_state(true), V10_STATE_BASE + 7);
        assert_eq!(campfire_state(false), V10_STATE_BASE + 6);
        assert!(campfire_lit(campfire_state(true)));
        assert!(!campfire_lit(campfire_state(false)));
        for lit in [true, false] {
            assert_eq!(state_block(campfire_state(lit)), CAMPFIRE);
        }
        assert_eq!(default_state(CAMPFIRE), campfire_state(true), "placed lit");
        assert_eq!(state_emissive(campfire_state(true)), 15, "lit emits 15");
        assert_eq!(state_emissive(campfire_state(false)), 0, "unlit is dark");
        assert_eq!(
            state_tiles(campfire_state(true))[0],
            TILE_CAMPFIRE,
            "lit tile"
        );
        assert_eq!(
            state_tiles(campfire_state(false))[0],
            TILE_CAMPFIRE_UNLIT,
            "unlit tile"
        );
        // bounds + window shape
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
        assert_eq!(V10_COUNT, 13);
        assert_eq!(BAMBOO, 417);
        assert_eq!(CHARCOAL, 425);
    }

    /// the 1.14 physical flags (VERIFIED w/Barrel "Transparent No",
    /// w/Campfire "Transparent Yes ... Luminous Yes (15) when lit",
    /// w/Bamboo "non-solid sapling form", w/Sweet_Berry_Bush the
    /// walk-through slow/damage contract)
    #[test]
    fn v114_block_flags_and_eggs() {
        // plants: cross, non-solid, walk-through
        for b in [BAMBOO, BAMBOO_SHOOT, SWEET_BERRY_BUSH] {
            assert!(is_cross(b), "{b} renders as a cross");
            assert!(!is_solid(b), "{b} is non-solid");
            assert!(!is_opaque(b));
        }
        // campfire: solid (stand ON it — the damage gate), not opaque
        assert!(is_solid(CAMPFIRE));
        assert!(!is_opaque(CAMPFIRE));
        // barrel: solid opaque cube (w/Barrel infobox)
        assert!(is_solid(BARREL) && is_opaque(BARREL));
        // items: inventory-only
        for b in [SWEET_BERRIES, STICK, CHARCOAL, SPAWN_EGG_FOX] {
            assert!(is_item_block(b), "{b} is an item-block");
        }
        // the fox egg passes the use gate + decodes kind 40
        assert!(is_spawn_egg(SPAWN_EGG_FOX));
        assert_eq!(egg_mob(SPAWN_EGG_FOX), Some(40));
        // F3-style state descriptions (the targeted-block property lines)
        assert_eq!(
            state_description(berry_bush_state(2)),
            "Sweet Berry Bush[age=2]"
        );
        assert_eq!(state_description(campfire_state(false)), "Campfire[lit=false]");
        // tiles within the atlas guard (the Phase-4 blank-tile regression)
        assert!(TILE_MAX >= TILE_CHARCOAL, "1.14 tiles within the atlas guard");
        assert_eq!(TILE_BERRY_BUSH_BASE + 3, 624);
        assert_eq!(TILE_MOB_FOX, 631);
    }
    /// 1.14 (nature half, part 2): the V11 window — the smelting trio +
    /// lantern + iron nugget. VERIFIED from the v114b captures: the
    /// smelters' lit states emit 13, the lantern 15 in BOTH forms
    /// (sitting + hanging), the smelters place unlit, the lantern
    /// places sitting, and the F3 property lines decode.
    #[test]
    fn v114b_v11_registry_window() {
        // 1:1 defaults: smelters UNLIT, lantern SITTING, nugget item,
        // the flowers their single state
        for (b, s) in [
            (BLAST_FURNACE, V11_STATE_BASE),
            (SMOKER, V11_STATE_BASE + 2),
            (LANTERN, V11_STATE_BASE + 4),
            (IRON_NUGGET, V11_STATE_BASE + 6),
            (CORNFLOWER, V11_STATE_BASE + 7),
            (LILY_OF_THE_VALLEY, V11_STATE_BASE + 8),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
        }
        // the lit states fold back too (written by the burn swap)
        for (s, b) in [
            (V11_STATE_BASE + 1, BLAST_FURNACE),
            (V11_STATE_BASE + 3, SMOKER),
            (V11_STATE_BASE + 5, LANTERN),
        ] {
            assert_eq!(state_block(s), b, "lit state {s} folds to {b}");
        }
        // emission: lit smelters 13, the lantern 15 in both forms, the
        // idle smelters + nugget 0 (VERIFIED infobox rows)
        assert_eq!(state_emissive(V11_STATE_BASE + 1), 13, "lit blast furnace");
        assert_eq!(state_emissive(V11_STATE_BASE + 3), 13, "lit smoker");
        assert_eq!(state_emissive(V11_STATE_BASE), 0, "idle blast furnace");
        assert_eq!(state_emissive(V11_STATE_BASE + 2), 0, "idle smoker");
        assert_eq!(state_emissive(V11_STATE_BASE + 4), 15, "sitting lantern");
        assert_eq!(state_emissive(V11_STATE_BASE + 5), 15, "hanging lantern");
        assert_eq!(emissive(LANTERN), 15, "lantern block row");
        // the F3 targeted-block property lines
        assert_eq!(state_description(V11_STATE_BASE + 1), "Blast Furnace[lit=true]");
        assert_eq!(state_description(V11_STATE_BASE), "Blast Furnace[lit=false]");
        assert_eq!(state_description(V11_STATE_BASE + 3), "Smoker[lit=true]");
        assert_eq!(state_description(V11_STATE_BASE + 5), "Lantern[hanging=true]");
        // the flowers carry no properties — plain names in the F3 line
        assert_eq!(state_description(V11_STATE_BASE + 7), "Cornflower");
        assert_eq!(state_description(V11_STATE_BASE + 8), "Lily of the Valley");
        assert_eq!(state_description(V11_STATE_BASE + 4), "Lantern[hanging=false]");
        // the window is in the picker; the nugget is an item-block;
        // the flowers are placeable picker blocks (not items)
        for want in [BLAST_FURNACE, SMOKER, LANTERN, CORNFLOWER, LILY_OF_THE_VALLEY] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        assert!(!PICKER_BLOCKS.contains(&IRON_NUGGET), "the nugget is an item, not a picker block");
        assert!(is_item_block(IRON_NUGGET), "nugget is an item-block");
        assert!(!is_item_block(CORNFLOWER), "cornflower is placeable");
        assert!(!is_item_block(LILY_OF_THE_VALLEY), "lily of the valley is placeable");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_IRON_NUGGET);
        assert!(TILE_MAX >= TILE_LILY_OF_THE_VALLEY, "flower tiles within the atlas guard");
        // bounds + window shape
        assert_eq!(V11_COUNT, 9);
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
    }
}

// ---------------------------------------------------------------------------
// 1.15 bracket tests (Buzzy Bees, live 2026-09-08)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod v115_tests {
    use super::*;

    /// the V12 window (ids 432..=439, states 698..=715): hive levels
    /// fold + re-encode; the honey/honeycomb identity states; the item
    /// states 1:1 (all VERIFIED w/Bee, w/Beehive, w/Bee_nest,
    /// w/Honey_Block, w/Honey_Bottle, w/Honeycomb - the research
    /// record docs/research/phase-v115-1.15-research.md)
    #[test]
    fn v115_v12_registry_window() {
        // hive honey_level roundtrip: every level 0..=5 on both blocks
        for lvl in 0u8..=5 {
            let ns = hive_state(BEE_NEST, lvl);
            let hs = hive_state(BEEHIVE, lvl);
            assert_eq!(honey_level(ns), lvl, "nest level {lvl}");
            assert_eq!(honey_level(hs), lvl, "hive level {lvl}");
            assert_eq!(state_block(ns), BEE_NEST);
            assert_eq!(state_block(hs), BEEHIVE);
            assert!(is_v12_state(ns) && is_v12_state(hs));
        }
        // defaults: level 0 for both; the identity + item states 1:1
        for (b, s) in [
            (BEE_NEST, V12_STATE_BASE),
            (BEEHIVE, V12_STATE_BASE + 6),
            (HONEY_BLOCK, V12_STATE_BASE + 12),
            (HONEYCOMB_BLOCK, V12_STATE_BASE + 13),
            (HONEYCOMB, V12_STATE_BASE + 14),
            (HONEY_BOTTLE, V12_STATE_BASE + 15),
            (SHEARS, V12_STATE_BASE + 16),
            (SPAWN_EGG_BEE, V12_STATE_BASE + 17),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
        }
        // the harvest gate: only honey_level 5 is full
        assert!(!hive_full(hive_state(BEEHIVE, 4)));
        assert!(hive_full(hive_state(BEEHIVE, 5)));
        assert!(hive_full(hive_state(BEE_NEST, 5)));
        assert!(!hive_full(hive_state(BEE_NEST, 4)));
        assert!(!hive_full(V12_STATE_BASE + 12), "honey block is not a hive");
        // the F3 targeted-block property lines
        assert_eq!(state_description(hive_state(BEE_NEST, 3)), "Bee Nest[honey_level=3]");
        assert_eq!(state_description(hive_state(BEEHIVE, 5)), "Beehive[honey_level=5]");
        // solidity classes: nest/hive/honeycomb-block solid-opaque;
        // honey solid but translucent (the JE partial row); the four
        // items are cross-sprited item-blocks
        assert!(is_solid(BEE_NEST) && is_opaque(BEE_NEST));
        assert!(is_solid(BEEHIVE) && is_opaque(BEEHIVE));
        assert!(is_solid(HONEY_BLOCK) && !is_opaque(HONEY_BLOCK));
        assert!(is_solid(HONEYCOMB_BLOCK) && is_opaque(HONEYCOMB_BLOCK));
        for it in [HONEYCOMB, HONEY_BOTTLE, SHEARS, SPAWN_EGG_BEE] {
            assert!(is_item_block(it), "item {it}");
            assert!(is_cross(it), "item {it} cross sprite");
        }
        // the bee egg decodes to mob kind 41
        assert_eq!(egg_mob(SPAWN_EGG_BEE), Some(41));
        assert!(is_spawn_egg(SPAWN_EGG_BEE));
        // the placeables are picker blocks; the items are not
        for want in [BEE_NEST, BEEHIVE, HONEY_BLOCK, HONEYCOMB_BLOCK] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        for no in [HONEYCOMB, HONEY_BOTTLE, SHEARS, SPAWN_EGG_BEE] {
            assert!(!PICKER_BLOCKS.contains(&no), "item {no} is not a picker block");
        }
        // names (the F3 plain-name lines + the item hotbar)
        assert_eq!(name(BEE_NEST), "Bee Nest");
        assert_eq!(name(BEEHIVE), "Beehive");
        assert_eq!(name(HONEY_BLOCK), "Honey Block");
        assert_eq!(name(HONEYCOMB_BLOCK), "Honeycomb Block");
        assert_eq!(name(HONEYCOMB), "Honeycomb");
        assert_eq!(name(HONEY_BOTTLE), "Honey Bottle");
        assert_eq!(name(SHEARS), "Shears");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_MOB_BEE, "bee sprite within the atlas guard");
        assert!(TILE_MAX >= TILE_BEEHIVE_FRONT_HONEY, "honey front within the atlas guard");
        // bounds + window shape
        assert_eq!(V12_COUNT, 18);
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
        assert_eq!(PICKER_BLOCKS.len(), 459);
    }
}
#[cfg(test)]
mod v116_tests {
    use super::*;

    /// the V13 window (ids 440..=453, states 716..=749): the anchor's
    /// charge + the target's power fold + re-encode; the chain's two
    /// face-matched forms; the identity + item states 1:1 (all
    /// VERIFIED against the v116 captures — the research record
    /// docs/research/phase-v116-1.16-research.md)
    #[test]
    fn v116_v13_registry_window() {
        // anchor charge roundtrip: every charge 0..=4, light ladder
        // 0/3/7/11/15 (the infobox row — charge 0 unlit)
        for c in 0u8..=5 {
            let s = anchor_state(c);
            assert_eq!(anchor_charge(s), c.min(4), "charge {c} clamps + decodes");
            assert_eq!(state_block(s), RESPAWN_ANCHOR);
            assert!(is_v13_state(s));
        }
        assert_eq!(anchor_light(anchor_state(0)), 0);
        assert_eq!(anchor_light(anchor_state(1)), 3);
        assert_eq!(anchor_light(anchor_state(2)), 7);
        assert_eq!(anchor_light(anchor_state(3)), 11);
        assert_eq!(anchor_light(anchor_state(4)), 15);
        // target power roundtrip: every power 0..=15 (clamped)
        for p in 0u8..=16 {
            let s = target_state(p);
            assert_eq!(target_power(s), p.min(15), "power {p} clamps + decodes");
            assert_eq!(state_block(s), TARGET);
            assert!(is_v13_state(s));
        }
        // defaults: charge 0, power 0, chain SITTING; identity + item
        // states 1:1
        for (b, s) in [
            (RESPAWN_ANCHOR, V13_STATE_BASE),
            (TARGET, V13_STATE_BASE + 5),
            (SOUL_SOIL, V13_STATE_BASE + 21),
            (BASALT, V13_STATE_BASE + 22),
            (BLACKSTONE, V13_STATE_BASE + 23),
            (GILDED_BLACKSTONE, V13_STATE_BASE + 24),
            (CRYING_OBSIDIAN, V13_STATE_BASE + 25),
            (NETHER_GOLD_ORE, V13_STATE_BASE + 26),
            (ANCIENT_DEBRIS, V13_STATE_BASE + 27),
            (NETHERITE_BLOCK, V13_STATE_BASE + 28),
            (CHAIN, V13_STATE_BASE + 29),
            (SOUL_FIRE, V13_STATE_BASE + 31),
            (NETHERITE_SCRAP, V13_STATE_BASE + 32),
            (NETHERITE_INGOT, V13_STATE_BASE + 33),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
        }
        // the chain's hanging form folds to the same parent (the
        // lantern pattern); hanging is NOT the default
        assert!(chain_hanging(V13_STATE_BASE + 30));
        assert!(!chain_hanging(V13_STATE_BASE + 29));
        assert_eq!(state_block(V13_STATE_BASE + 30), CHAIN);
        // soul fire: cross-sprite, non-solid, light 10
        assert!(is_soul_fire(V13_STATE_BASE + 31));
        assert!(is_soul_fire(default_state(SOUL_FIRE)));
        assert!(is_cross(SOUL_FIRE));
        assert!(!is_solid(SOUL_FIRE));
        assert_eq!(emissive(SOUL_FIRE), 10);
        // crying obsidian light 10; the anchor's charge light rides
        // state_emissive (the state-level ladder)
        assert_eq!(emissive(CRYING_OBSIDIAN), 10);
        assert_eq!(state_emissive(anchor_state(2)), 7);
        assert_eq!(state_emissive(anchor_state(4)), 15);
        assert_eq!(state_emissive(anchor_state(0)), 0);
        // the F3 targeted-block property lines
        assert_eq!(state_description(anchor_state(3)), "Respawn Anchor[charge=3]");
        assert_eq!(state_description(target_state(12)), "Target[power=12]");
        assert_eq!(state_description(V13_STATE_BASE + 30), "Chain[hanging=true]");
        assert_eq!(state_description(default_state(SOUL_SOIL)), "Soul Soil");
        // solidity classes: the stone-family blocks solid-opaque; the
        // chain + soul fire are non-solid cross-sprite decorations
        for cube in [
            SOUL_SOIL,
            BASALT,
            BLACKSTONE,
            GILDED_BLACKSTONE,
            CRYING_OBSIDIAN,
            RESPAWN_ANCHOR,
            TARGET,
            NETHER_GOLD_ORE,
            ANCIENT_DEBRIS,
            NETHERITE_BLOCK,
        ] {
            assert!(is_solid(cube), "block {cube} solid");
            assert!(is_opaque(cube), "block {cube} opaque");
        }
        assert!(!is_solid(CHAIN) && !is_opaque(CHAIN));
        assert!(is_cross(CHAIN), "chain cross sprite (the lantern class)");
        // the material items are item-blocks, never picker blocks
        for it in [NETHERITE_SCRAP, NETHERITE_INGOT] {
            assert!(is_item_block(it), "item {it}");
            assert!(is_cross(it), "item {it} cross sprite");
            assert!(!PICKER_BLOCKS.contains(&it), "item {it} is not a picker block");
        }
        // the 12 placeables are picker blocks
        for want in [
            SOUL_SOIL,
            BASALT,
            BLACKSTONE,
            GILDED_BLACKSTONE,
            CRYING_OBSIDIAN,
            RESPAWN_ANCHOR,
            TARGET,
            NETHER_GOLD_ORE,
            ANCIENT_DEBRIS,
            NETHERITE_BLOCK,
            CHAIN,
            SOUL_FIRE,
        ] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        // the anchor's charge art switches at charge >= 1
        let uncharged = state_tiles(anchor_state(0));
        let charged = state_tiles(anchor_state(1));
        assert_eq!(uncharged[3], TILE_ANCHOR_SIDE);
        assert_eq!(charged[3], TILE_ANCHOR_SIDE_CHARGED);
        // names (the F3 plain-name lines + the item hotbar)
        assert_eq!(name(SOUL_SOIL), "Soul Soil");
        assert_eq!(name(BASALT), "Basalt");
        assert_eq!(name(BLACKSTONE), "Blackstone");
        assert_eq!(name(GILDED_BLACKSTONE), "Gilded Blackstone");
        assert_eq!(name(CRYING_OBSIDIAN), "Crying Obsidian");
        assert_eq!(name(RESPAWN_ANCHOR), "Respawn Anchor");
        assert_eq!(name(TARGET), "Target");
        assert_eq!(name(NETHER_GOLD_ORE), "Nether Gold Ore");
        assert_eq!(name(ANCIENT_DEBRIS), "Ancient Debris");
        assert_eq!(name(NETHERITE_BLOCK), "Block of Netherite");
        assert_eq!(name(CHAIN), "Chain");
        assert_eq!(name(SOUL_FIRE), "Soul Fire");
        assert_eq!(name(NETHERITE_SCRAP), "Netherite Scrap");
        assert_eq!(name(NETHERITE_INGOT), "Netherite Ingot");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_SOUL_FIRE, "soul fire within the atlas guard");
        assert!(TILE_MAX >= TILE_ANCHOR_SIDE_CHARGED, "anchor glow within the atlas guard");
        assert!(TILE_MAX >= TILE_NETHERITE_INGOT, "ingot within the atlas guard");
        // bounds + window shape
        assert_eq!(V13_COUNT, 34);
        assert_eq!(V13_STATE_BASE + V13_COUNT, 750);
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
        assert_eq!(PICKER_BLOCKS.len(), 459);
    }

    /// the V14 window (ids 454..=478, states 750..=775): the
    /// crimson/warped families — all identity folds except the soul
    /// lantern's sitting/hanging pair (all VERIFIED against the
    /// v116b captures — the research record
    /// docs/research/phase-v116b-1.16-research.md)
    #[test]
    fn v116b_v14_registry_window() {
        // defaults 1:1 for every family block; the soul lantern places
        // SITTING (the chain convention); the eggs are item states
        for (b, s) in [
            (CRIMSON_STEM, V14_STATE_BASE),
            (CRIMSON_HYPHAE, V14_STATE_BASE + 1),
            (CRIMSON_PLANKS, V14_STATE_BASE + 2),
            (CRIMSON_NYLIUM, V14_STATE_BASE + 3),
            (CRIMSON_FUNGUS, V14_STATE_BASE + 4),
            (CRIMSON_ROOTS, V14_STATE_BASE + 5),
            (WEEPING_VINES, V14_STATE_BASE + 6),
            (WARPED_STEM, V14_STATE_BASE + 7),
            (WARPED_HYPHAE, V14_STATE_BASE + 8),
            (WARPED_PLANKS, V14_STATE_BASE + 9),
            (WARPED_NYLIUM, V14_STATE_BASE + 10),
            (WARPED_FUNGUS, V14_STATE_BASE + 11),
            (WARPED_ROOTS, V14_STATE_BASE + 12),
            (TWISTING_VINES, V14_STATE_BASE + 13),
            (WARPED_WART_BLOCK, V14_STATE_BASE + 14),
            (SHROOMLIGHT, V14_STATE_BASE + 15),
            (NETHER_SPROUTS, V14_STATE_BASE + 16),
            (POLISHED_BASALT, V14_STATE_BASE + 17),
            (POLISHED_BLACKSTONE, V14_STATE_BASE + 18),
            (POLISHED_BLACKSTONE_BRICKS, V14_STATE_BASE + 19),
            (SOUL_TORCH, V14_STATE_BASE + 20),
            (SOUL_LANTERN, V14_STATE_BASE + 21),
            (SPAWN_EGG_STRIDER, V14_STATE_BASE + 23),
            (SPAWN_EGG_PIGLIN, V14_STATE_BASE + 24),
            (SPAWN_EGG_HOGLIN, V14_STATE_BASE + 25),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
            assert!(!is_model_state(s), "state {s} never routes to models");
        }
        // the soul lantern's hanging form folds to the same parent;
        // hanging is NOT the default (the lantern/chain pattern)
        assert!(soul_lantern_hanging(V14_STATE_BASE + 22));
        assert!(!soul_lantern_hanging(V14_STATE_BASE + 21));
        assert_eq!(state_block(V14_STATE_BASE + 22), SOUL_LANTERN);
        assert_eq!(
            state_description(V14_STATE_BASE + 22),
            "Soul Lantern[hanging=true]"
        );
        assert_eq!(
            state_description(V14_STATE_BASE + 21),
            "Soul Lantern[hanging=false]"
        );
        // the soul lights: torch + lantern at light 10, shroomlight 15
        assert_eq!(emissive(SOUL_TORCH), 10);
        assert_eq!(emissive(SOUL_LANTERN), 10);
        assert_eq!(state_emissive(default_state(SOUL_LANTERN)), 10);
        assert_eq!(state_emissive(default_state(V14_STATE_BASE + 22)), 10);
        assert_eq!(emissive(SHROOMLIGHT), 15);
        // solidity classes: stems/hyphae/planks/nyliums/wart/shroomlight/
        // polished stones are solid-opaque cubes; the plants + soul
        // torch/lantern are non-solid cross sprites
        for cube in [
            CRIMSON_STEM,
            CRIMSON_HYPHAE,
            CRIMSON_PLANKS,
            CRIMSON_NYLIUM,
            WARPED_STEM,
            WARPED_HYPHAE,
            WARPED_PLANKS,
            WARPED_NYLIUM,
            WARPED_WART_BLOCK,
            SHROOMLIGHT,
            POLISHED_BASALT,
            POLISHED_BLACKSTONE,
            POLISHED_BLACKSTONE_BRICKS,
        ] {
            assert!(is_solid(cube), "block {cube} solid");
            assert!(is_opaque(cube), "block {cube} opaque");
        }
        for plant in [
            CRIMSON_FUNGUS,
            CRIMSON_ROOTS,
            WEEPING_VINES,
            WARPED_FUNGUS,
            WARPED_ROOTS,
            TWISTING_VINES,
            NETHER_SPROUTS,
            SOUL_TORCH,
            SOUL_LANTERN,
        ] {
            assert!(!is_solid(plant), "plant {plant} non-solid");
            assert!(!is_opaque(plant), "plant {plant} transparent");
            assert!(is_cross(plant), "plant {plant} cross sprite");
        }
        // the forest-plant classifier (the placement + mob-repel helper)
        for p in [
            CRIMSON_FUNGUS,
            CRIMSON_ROOTS,
            NETHER_SPROUTS,
            WARPED_FUNGUS,
            WARPED_ROOTS,
        ] {
            assert!(is_forest_plant(p), "forest plant {p}");
            assert!(is_forest_plant(default_state(p)), "forest plant state");
        }
        // the vines are climbable vegetation, not floor plants (the
        // classifier drives floor placement + mob repel only)
        assert!(!is_forest_plant(WEEPING_VINES));
        assert!(!is_forest_plant(TWISTING_VINES));
        // the eggs: kinds 42..=44, item-blocks, picker blocks
        assert_eq!(egg_mob(SPAWN_EGG_STRIDER), Some(42));
        assert_eq!(egg_mob(SPAWN_EGG_PIGLIN), Some(43));
        assert_eq!(egg_mob(SPAWN_EGG_HOGLIN), Some(44));
        for e in [SPAWN_EGG_STRIDER, SPAWN_EGG_PIGLIN, SPAWN_EGG_HOGLIN] {
            assert!(is_item_block(e), "egg {e} item-block");
            assert!(is_spawn_egg(e), "egg {e}");
            assert!(PICKER_BLOCKS.contains(&e), "egg {e} in picker");
        }
        // the 22 placeables are picker blocks
        for want in [
            CRIMSON_STEM,
            CRIMSON_HYPHAE,
            CRIMSON_PLANKS,
            CRIMSON_NYLIUM,
            CRIMSON_FUNGUS,
            CRIMSON_ROOTS,
            WEEPING_VINES,
            WARPED_STEM,
            WARPED_HYPHAE,
            WARPED_PLANKS,
            WARPED_NYLIUM,
            WARPED_FUNGUS,
            WARPED_ROOTS,
            TWISTING_VINES,
            WARPED_WART_BLOCK,
            SHROOMLIGHT,
            NETHER_SPROUTS,
            POLISHED_BASALT,
            POLISHED_BLACKSTONE,
            POLISHED_BLACKSTONE_BRICKS,
            SOUL_TORCH,
            SOUL_LANTERN,
        ] {
            assert!(PICKER_BLOCKS.contains(&want), "picker missing {want}");
        }
        // names (the F3 plain-name lines + the item hotbar)
        assert_eq!(name(CRIMSON_STEM), "Crimson Stem");
        assert_eq!(name(CRIMSON_PLANKS), "Crimson Planks");
        assert_eq!(name(CRIMSON_NYLIUM), "Crimson Nylium");
        assert_eq!(name(CRIMSON_FUNGUS), "Crimson Fungus");
        assert_eq!(name(CRIMSON_ROOTS), "Crimson Roots");
        assert_eq!(name(WEEPING_VINES), "Weeping Vines");
        assert_eq!(name(WARPED_STEM), "Warped Stem");
        assert_eq!(name(WARPED_PLANKS), "Warped Planks");
        assert_eq!(name(WARPED_NYLIUM), "Warped Nylium");
        assert_eq!(name(WARPED_FUNGUS), "Warped Fungus");
        assert_eq!(name(WARPED_ROOTS), "Warped Roots");
        assert_eq!(name(TWISTING_VINES), "Twisting Vines");
        assert_eq!(name(WARPED_WART_BLOCK), "Warped Wart Block");
        assert_eq!(name(SHROOMLIGHT), "Shroomlight");
        assert_eq!(name(NETHER_SPROUTS), "Nether Sprouts");
        assert_eq!(name(POLISHED_BASALT), "Polished Basalt");
        assert_eq!(name(POLISHED_BLACKSTONE), "Polished Blackstone");
        assert_eq!(name(POLISHED_BLACKSTONE_BRICKS), "Polished Blackstone Bricks");
        assert_eq!(name(SOUL_TORCH), "Soul Torch");
        assert_eq!(name(SOUL_LANTERN), "Soul Lantern");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_SOUL_LANTERN, "soul lantern within the atlas guard");
        assert!(TILE_MAX >= TILE_SHROOMLIGHT, "shroomlight within the atlas guard");
        assert!(TILE_MAX >= TILE_MOB_HOGLIN, "hoglin sprite within the atlas guard");
        // bounds + window shape
        assert_eq!(V14_COUNT, 26);
        assert_eq!(V14_STATE_BASE + V14_COUNT, 776);
        // the completeness audit: the V15 window (26 items + the two
        // spawner states)
        assert_eq!(V15_COUNT, 29);
        assert_eq!(V15_STATE_BASE + V15_COUNT, 805);
        assert_eq!(BLOCK_COUNT, 507);
        assert_eq!(STATE_COUNT, 806);
        assert_eq!(PICKER_BLOCKS.len(), 459);
    }

    /// the V15 window (ids 479..=504, states 776..=803): the
    /// completeness-audit item rows — all identity folds; the two
    /// spawner states fold to the Monster Spawner block (all VERIFIED
    /// against the audit16 captures — the research record
    /// docs/research/audit15-1.0-1.16.5-research.md)
    #[test]
    fn audit16_v15_registry_window() {
        // defaults 1:1 for every item row; the eggs are item states
        for (b, s) in [
            (STEAK, V15_STATE_BASE),
            (COOKED_PORKCHOP, V15_STATE_BASE + 1),
            (COOKED_CHICKEN, V15_STATE_BASE + 2),
            (COOKED_MUTTON, V15_STATE_BASE + 3),
            (COOKED_COD, V15_STATE_BASE + 4),
            (COOKED_SALMON, V15_STATE_BASE + 5),
            (APPLE, V15_STATE_BASE + 6),
            (BOWL, V15_STATE_BASE + 7),
            (MUSHROOM_STEW, V15_STATE_BASE + 8),
            (RABBIT_STEW, V15_STATE_BASE + 9),
            (BEETROOT, V15_STATE_BASE + 10),
            (BEETROOT_SOUP, V15_STATE_BASE + 11),
            (SUGAR, V15_STATE_BASE + 12),
            (EGG, V15_STATE_BASE + 13),
            (POISONOUS_POTATO, V15_STATE_BASE + 14),
            (POPPED_CHORUS_FRUIT, V15_STATE_BASE + 15),
            (GHAST_TEAR, V15_STATE_BASE + 16),
            (POTION_LEAPING, V15_STATE_BASE + 17),
            (POTION_LEAPING_II, V15_STATE_BASE + 18),
            (POTION_LEAPING_LONG, V15_STATE_BASE + 19),
            (POTION_REGEN, V15_STATE_BASE + 20),
            (POTION_REGEN_II, V15_STATE_BASE + 21),
            (POTION_REGEN_LONG, V15_STATE_BASE + 22),
            (SPAWN_EGG_GHAST, V15_STATE_BASE + 23),
            (SPAWN_EGG_CAVE_SPIDER, V15_STATE_BASE + 24),
            (SPAWN_EGG_SILVERFISH, V15_STATE_BASE + 25),
            (MELON_SLICE, V15_STATE_BASE + 28),
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
            assert!(is_item_block(b), "block {b} is an item block");
        }
        // the two spawner states: fold to the spawner block + decode
        assert_eq!(state_block(SPAWNER_CAVESPIDER), SPAWNER);
        assert_eq!(state_block(SPAWNER_SILVERFISH), SPAWNER);
        assert_eq!(spawner_mob(SPAWNER_CAVESPIDER), 7);
        assert_eq!(spawner_mob(SPAWNER_SILVERFISH), 8);
        // the egg roundtrip: kinds 45..=47
        assert_eq!(egg_mob(SPAWN_EGG_GHAST), Some(45));
        assert_eq!(egg_mob(SPAWN_EGG_CAVE_SPIDER), Some(46));
        assert_eq!(egg_mob(SPAWN_EGG_SILVERFISH), Some(47));
        // names + tiles
        assert_eq!(name(STEAK), "Steak");
        assert_eq!(name(RABBIT_STEW), "Rabbit Stew");
        assert_eq!(name(POISONOUS_POTATO), "Poisonous Potato");
        assert_eq!(name(POPPED_CHORUS_FRUIT), "Popped Chorus Fruit");
        assert_eq!(name(GHAST_TEAR), "Ghast Tear");
        assert_eq!(name(POTION_LEAPING), "Potion of Leaping");
        // tiles within the atlas guard
        assert!(TILE_MAX >= TILE_MOB_SILVERFISH, "silverfish sprite within the atlas guard");
    }
}
