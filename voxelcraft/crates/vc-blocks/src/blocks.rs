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
// ---------------------------------------------------------------------------
// 1.7.2 bracket ("The Update that Changed the World", 2013-10-25 —
// minecraft.wiki/w/Java_Edition_1.7.2, live round 2026-09-06): the 16
// stained-glass + 16 stained-clay tiles, red sand, packed ice, podzol,
// acacia/dark-oak logs, the 8 new small flowers, the 4 two-block-tall
// flowers (lower + upper halves), and 4 fish item icons.
// Clean-room palettes approximate the vanilla 16 dye hues (our art, not
// Mojang's); acacia/dark-oak LEAVES reuse TILE_LEAVES exactly — the 1.7.2
// changelog itself notes both are "visually identical to regular oak
// leaves".
// ---------------------------------------------------------------------------
pub const TILE_STAINED_GLASS_WHITE: u16 = 124;
pub const TILE_STAINED_GLASS_ORANGE: u16 = 125;
pub const TILE_STAINED_GLASS_MAGENTA: u16 = 126;
pub const TILE_STAINED_GLASS_LIGHT_BLUE: u16 = 127;
pub const TILE_STAINED_GLASS_YELLOW: u16 = 128;
pub const TILE_STAINED_GLASS_LIME: u16 = 129;
pub const TILE_STAINED_GLASS_PINK: u16 = 130;
pub const TILE_STAINED_GLASS_GRAY: u16 = 131;
pub const TILE_STAINED_GLASS_LIGHT_GRAY: u16 = 132;
pub const TILE_STAINED_GLASS_CYAN: u16 = 133;
pub const TILE_STAINED_GLASS_PURPLE: u16 = 134;
pub const TILE_STAINED_GLASS_BLUE: u16 = 135;
pub const TILE_STAINED_GLASS_BROWN: u16 = 136;
pub const TILE_STAINED_GLASS_GREEN: u16 = 137;
pub const TILE_STAINED_GLASS_RED: u16 = 138;
pub const TILE_STAINED_GLASS_BLACK: u16 = 139;
pub const TILE_STAINED_TERRACOTTA_WHITE: u16 = 140;
pub const TILE_STAINED_TERRACOTTA_ORANGE: u16 = 141;
pub const TILE_STAINED_TERRACOTTA_MAGENTA: u16 = 142;
pub const TILE_STAINED_TERRACOTTA_LIGHT_BLUE: u16 = 143;
pub const TILE_STAINED_TERRACOTTA_YELLOW: u16 = 144;
pub const TILE_STAINED_TERRACOTTA_LIME: u16 = 145;
pub const TILE_STAINED_TERRACOTTA_PINK: u16 = 146;
pub const TILE_STAINED_TERRACOTTA_GRAY: u16 = 147;
pub const TILE_STAINED_TERRACOTTA_LIGHT_GRAY: u16 = 148;
pub const TILE_STAINED_TERRACOTTA_CYAN: u16 = 149;
pub const TILE_STAINED_TERRACOTTA_PURPLE: u16 = 150;
pub const TILE_STAINED_TERRACOTTA_BLUE: u16 = 151;
pub const TILE_STAINED_TERRACOTTA_BROWN: u16 = 152;
pub const TILE_STAINED_TERRACOTTA_GREEN: u16 = 153;
pub const TILE_STAINED_TERRACOTTA_RED: u16 = 154;
pub const TILE_STAINED_TERRACOTTA_BLACK: u16 = 155;
pub const TILE_RED_SAND: u16 = 156;
pub const TILE_PACKED_ICE: u16 = 157;
pub const TILE_PODZOL_TOP: u16 = 158;
pub const TILE_PODZOL_SIDE: u16 = 159;
pub const TILE_ACACIA_LOG_SIDE: u16 = 160;
pub const TILE_ACACIA_LOG_TOP: u16 = 161;
pub const TILE_DARK_OAK_LOG_SIDE: u16 = 162;
pub const TILE_DARK_OAK_LOG_TOP: u16 = 163;
pub const TILE_ALLIUM: u16 = 164;
pub const TILE_AZURE_BLUET: u16 = 165;
pub const TILE_BLUE_ORCHID: u16 = 166;
pub const TILE_OXEYE_DAISY: u16 = 167;
pub const TILE_ORANGE_TULIP: u16 = 168;
pub const TILE_RED_TULIP: u16 = 169;
pub const TILE_WHITE_TULIP: u16 = 170;
pub const TILE_PINK_TULIP: u16 = 171;
pub const TILE_SUNFLOWER_LOWER: u16 = 172;
pub const TILE_SUNFLOWER_TOP: u16 = 173;
pub const TILE_LILAC_LOWER: u16 = 174;
pub const TILE_LILAC_TOP: u16 = 175;
pub const TILE_PEONY_LOWER: u16 = 176;
pub const TILE_PEONY_TOP: u16 = 177;
pub const TILE_ROSE_BUSH_LOWER: u16 = 178;
pub const TILE_ROSE_BUSH_TOP: u16 = 179;
pub const TILE_RAW_FISH: u16 = 180;
pub const TILE_RAW_SALMON: u16 = 181;
pub const TILE_CLOWNFISH: u16 = 182;
pub const TILE_PUFFERFISH: u16 = 183;
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
/// spawner mob-kind code 0: zombie (dungeon roll 50%)
pub const SPAWNER_ZOMBIE: u8 = 0;
/// spawner mob-kind code 1: skeleton (dungeon roll 25%)
pub const SPAWNER_SKELETON: u8 = 1;
/// spawner mob-kind code 2: spider (dungeon roll 25%)
pub const SPAWNER_SPIDER: u8 = 2;

// ---------------------------------------------------------------------------
// 1.7.2 bracket — V2 block window (ids 103..=161, states 236..=294).
// The pre-1.7 registry is fully allocated (identity states 0..=56, model
// states 57..=88, sim states 89..=235). New blocks get ids past 102 that
// never live in the world as identity states — each stores its dedicated
// V2 state, folded through V2_STATE_TO_BLOCK (the table IS the state→block
// mapping; `Chunk::get` now folds every state through state_block so u16
// states above 255 are safe).
// All content verified against minecraft.wiki/w/Java_Edition_1.7.2
// (live round, 2026-09-06).
// ---------------------------------------------------------------------------

// stained glass, 16 colors — translucent like glass (solid, non-opaque)
pub const STAINED_GLASS_WHITE: u8 = 103;
pub const STAINED_GLASS_ORANGE: u8 = 104;
pub const STAINED_GLASS_MAGENTA: u8 = 105;
pub const STAINED_GLASS_LIGHT_BLUE: u8 = 106;
pub const STAINED_GLASS_YELLOW: u8 = 107;
pub const STAINED_GLASS_LIME: u8 = 108;
pub const STAINED_GLASS_PINK: u8 = 109;
pub const STAINED_GLASS_GRAY: u8 = 110;
pub const STAINED_GLASS_LIGHT_GRAY: u8 = 111;
pub const STAINED_GLASS_CYAN: u8 = 112;
pub const STAINED_GLASS_PURPLE: u8 = 113;
pub const STAINED_GLASS_BLUE: u8 = 114;
pub const STAINED_GLASS_BROWN: u8 = 115;
pub const STAINED_GLASS_GREEN: u8 = 116;
pub const STAINED_GLASS_RED: u8 = 117;
pub const STAINED_GLASS_BLACK: u8 = 118;
// stained terracotta, 16 colors — "stained clay" in 1.7 parlance
pub const STAINED_TERRACOTTA_WHITE: u8 = 119;
pub const STAINED_TERRACOTTA_ORANGE: u8 = 120;
pub const STAINED_TERRACOTTA_MAGENTA: u8 = 121;
pub const STAINED_TERRACOTTA_LIGHT_BLUE: u8 = 122;
pub const STAINED_TERRACOTTA_YELLOW: u8 = 123;
pub const STAINED_TERRACOTTA_LIME: u8 = 124;
pub const STAINED_TERRACOTTA_PINK: u8 = 125;
pub const STAINED_TERRACOTTA_GRAY: u8 = 126;
pub const STAINED_TERRACOTTA_LIGHT_GRAY: u8 = 127;
pub const STAINED_TERRACOTTA_CYAN: u8 = 128;
pub const STAINED_TERRACOTTA_PURPLE: u8 = 129;
pub const STAINED_TERRACOTTA_BLUE: u8 = 130;
pub const STAINED_TERRACOTTA_BROWN: u8 = 131;
pub const STAINED_TERRACOTTA_GREEN: u8 = 132;
pub const STAINED_TERRACOTTA_RED: u8 = 133;
pub const STAINED_TERRACOTTA_BLACK: u8 = 134;
pub const RED_SAND: u8 = 135;
pub const PACKED_ICE: u8 = 136;
pub const PODZOL: u8 = 137;
pub const ACACIA_LOG: u8 = 138;
pub const ACACIA_LEAVES: u8 = 139;
pub const DARK_OAK_LOG: u8 = 140;
pub const DARK_OAK_LEAVES: u8 = 141;
// the 8 new small flowers (1.7.2: allium, azure bluet, blue orchid,
// oxeye daisy + 4 tulips; poppy already exists as FLOWER_RED)
pub const ALLIUM: u8 = 142;
pub const AZURE_BLUET: u8 = 143;
pub const BLUE_ORCHID: u8 = 144;
pub const OXEYE_DAISY: u8 = 145;
pub const ORANGE_TULIP: u8 = 146;
pub const RED_TULIP: u8 = 147;
pub const WHITE_TULIP: u8 = 148;
pub const PINK_TULIP: u8 = 149;
// the 4 two-block-tall flowers, lower + upper halves (vanilla models one
// block with half=lower/upper; ours is two ids — same observable shape)
pub const SUNFLOWER: u8 = 150;
pub const SUNFLOWER_TOP: u8 = 151;
pub const LILAC: u8 = 152;
pub const LILAC_TOP: u8 = 153;
pub const PEONY: u8 = 154;
pub const PEONY_TOP: u8 = 155;
pub const ROSE_BUSH: u8 = 156;
pub const ROSE_BUSH_TOP: u8 = 157;
// 1.7.2 fish items — inventory-only (never placed):
// VERIFIED (wiki §Items): clownfish restores 1, raw salmon 2 (cooked 6),
// pufferfish 1 + Poison IV 1:00 + Hunger III 0:15 + Nausea 0:15 and brews
// Water Breathing; raw fish (cod) restores 2.
pub const RAW_FISH: u8 = 158;
pub const RAW_SALMON: u8 = 159;
pub const CLOWNFISH: u8 = 160;
pub const PUFFERFISH: u8 = 161;

/// first state of the V2 window (1.7.2 bracket)
pub const V2_STATE_BASE: u16 = 236;
/// V2 state count: one dedicated state per new block, order = id order
pub const V2_COUNT: u16 = 59; // ids 103..=161
/// state → block fold table for the V2 window. Index = state − BASE.
pub const V2_STATE_TO_BLOCK: [u8; V2_COUNT as usize] = [
    STAINED_GLASS_WHITE, STAINED_GLASS_ORANGE, STAINED_GLASS_MAGENTA,
    STAINED_GLASS_LIGHT_BLUE, STAINED_GLASS_YELLOW, STAINED_GLASS_LIME,
    STAINED_GLASS_PINK, STAINED_GLASS_GRAY, STAINED_GLASS_LIGHT_GRAY,
    STAINED_GLASS_CYAN, STAINED_GLASS_PURPLE, STAINED_GLASS_BLUE,
    STAINED_GLASS_BROWN, STAINED_GLASS_GREEN, STAINED_GLASS_RED,
    STAINED_GLASS_BLACK,
    STAINED_TERRACOTTA_WHITE, STAINED_TERRACOTTA_ORANGE,
    STAINED_TERRACOTTA_MAGENTA, STAINED_TERRACOTTA_LIGHT_BLUE,
    STAINED_TERRACOTTA_YELLOW, STAINED_TERRACOTTA_LIME,
    STAINED_TERRACOTTA_PINK, STAINED_TERRACOTTA_GRAY,
    STAINED_TERRACOTTA_LIGHT_GRAY, STAINED_TERRACOTTA_CYAN,
    STAINED_TERRACOTTA_PURPLE, STAINED_TERRACOTTA_BLUE,
    STAINED_TERRACOTTA_BROWN, STAINED_TERRACOTTA_GREEN,
    STAINED_TERRACOTTA_RED, STAINED_TERRACOTTA_BLACK,
    RED_SAND, PACKED_ICE, PODZOL, ACACIA_LOG, ACACIA_LEAVES, DARK_OAK_LOG,
    DARK_OAK_LEAVES,
    ALLIUM, AZURE_BLUET, BLUE_ORCHID, OXEYE_DAISY, ORANGE_TULIP, RED_TULIP,
    WHITE_TULIP, PINK_TULIP,
    SUNFLOWER, SUNFLOWER_TOP, LILAC, LILAC_TOP, PEONY, PEONY_TOP,
    ROSE_BUSH, ROSE_BUSH_TOP,
    RAW_FISH, RAW_SALMON, CLOWNFISH, PUFFERFISH,
];

/// default (and only) V2 state of a block id — `b − 103 + V2_STATE_BASE`.
/// Returns None for pre-V2 blocks.
#[inline]
pub fn v2_state(b: u8) -> Option<u16> {
    if (103..103 + V2_COUNT as u8).contains(&b) {
        Some(V2_STATE_BASE + (b - 103) as u16)
    } else {
        None
    }
}

#[inline]
pub fn is_v2_state(s: u16) -> bool {
    (V2_STATE_BASE..V2_STATE_BASE + V2_COUNT).contains(&s)
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

/// acacia/dark-oak log axis states (the V2 log window — same pattern as
/// the pre-1.7 oak/birch/spruce axis states 57..=62)
pub const ACACIA_LOG_X: u16 = 295;
pub const ACACIA_LOG_Z: u16 = 296;
pub const DARK_OAK_LOG_X: u16 = 297;
pub const DARK_OAK_LOG_Z: u16 = 298;

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
pub const STATE_COUNT: usize = 299;
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
        b if (103..103 + V2_COUNT as u8).contains(&b) => {
            V2_STATE_BASE + (b - 103) as u16
        }
        OAK_SLAB => 63,     // PROP_BLOCKS[0].base_state (half=bottom)
        COBBLE_STAIRS => 65, // base_state (facing=north, half=bottom)
        OAK_FENCE => 73,    // base_state (no connections)
        _ => b as u16,
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
        _ => s as u8, // identity for 0..=56
    }
}

/// vanilla-style state description for F3: "Oak Slab[half=top]"
#[inline]
pub fn state_description(s: u16) -> String {
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
    b == OAK_LOG || b == BIRCH_LOG || b == SPRUCE_LOG || b == ACACIA_LOG || b == DARK_OAK_LOG
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
pub const TILE_MAX: u16 = 183;

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
            | RAW_FISH | RAW_SALMON | CLOWNFISH | PUFFERFISH
    )
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
    // ---- 1.7.2 bracket (V2 window) — minecraft.wiki/w/Java_Edition_1.7.2,
    // live round 2026-09-06 ----
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
    d("White Terracotta", [TILE_STAINED_TERRACOTTA_WHITE, TILE_STAINED_TERRACOTTA_WHITE, TILE_STAINED_TERRACOTTA_WHITE], true, true, false, false, 0, SoundFamily::Stone),
    d("Orange Terracotta", [TILE_STAINED_TERRACOTTA_ORANGE, TILE_STAINED_TERRACOTTA_ORANGE, TILE_STAINED_TERRACOTTA_ORANGE], true, true, false, false, 0, SoundFamily::Stone),
    d("Magenta Terracotta", [TILE_STAINED_TERRACOTTA_MAGENTA, TILE_STAINED_TERRACOTTA_MAGENTA, TILE_STAINED_TERRACOTTA_MAGENTA], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Blue Terracotta", [TILE_STAINED_TERRACOTTA_LIGHT_BLUE, TILE_STAINED_TERRACOTTA_LIGHT_BLUE, TILE_STAINED_TERRACOTTA_LIGHT_BLUE], true, true, false, false, 0, SoundFamily::Stone),
    d("Yellow Terracotta", [TILE_STAINED_TERRACOTTA_YELLOW, TILE_STAINED_TERRACOTTA_YELLOW, TILE_STAINED_TERRACOTTA_YELLOW], true, true, false, false, 0, SoundFamily::Stone),
    d("Lime Terracotta", [TILE_STAINED_TERRACOTTA_LIME, TILE_STAINED_TERRACOTTA_LIME, TILE_STAINED_TERRACOTTA_LIME], true, true, false, false, 0, SoundFamily::Stone),
    d("Pink Terracotta", [TILE_STAINED_TERRACOTTA_PINK, TILE_STAINED_TERRACOTTA_PINK, TILE_STAINED_TERRACOTTA_PINK], true, true, false, false, 0, SoundFamily::Stone),
    d("Gray Terracotta", [TILE_STAINED_TERRACOTTA_GRAY, TILE_STAINED_TERRACOTTA_GRAY, TILE_STAINED_TERRACOTTA_GRAY], true, true, false, false, 0, SoundFamily::Stone),
    d("Light Gray Terracotta", [TILE_STAINED_TERRACOTTA_LIGHT_GRAY, TILE_STAINED_TERRACOTTA_LIGHT_GRAY, TILE_STAINED_TERRACOTTA_LIGHT_GRAY], true, true, false, false, 0, SoundFamily::Stone),
    d("Cyan Terracotta", [TILE_STAINED_TERRACOTTA_CYAN, TILE_STAINED_TERRACOTTA_CYAN, TILE_STAINED_TERRACOTTA_CYAN], true, true, false, false, 0, SoundFamily::Stone),
    d("Purple Terracotta", [TILE_STAINED_TERRACOTTA_PURPLE, TILE_STAINED_TERRACOTTA_PURPLE, TILE_STAINED_TERRACOTTA_PURPLE], true, true, false, false, 0, SoundFamily::Stone),
    d("Blue Terracotta", [TILE_STAINED_TERRACOTTA_BLUE, TILE_STAINED_TERRACOTTA_BLUE, TILE_STAINED_TERRACOTTA_BLUE], true, true, false, false, 0, SoundFamily::Stone),
    d("Brown Terracotta", [TILE_STAINED_TERRACOTTA_BROWN, TILE_STAINED_TERRACOTTA_BROWN, TILE_STAINED_TERRACOTTA_BROWN], true, true, false, false, 0, SoundFamily::Stone),
    d("Green Terracotta", [TILE_STAINED_TERRACOTTA_GREEN, TILE_STAINED_TERRACOTTA_GREEN, TILE_STAINED_TERRACOTTA_GREEN], true, true, false, false, 0, SoundFamily::Stone),
    d("Red Terracotta", [TILE_STAINED_TERRACOTTA_RED, TILE_STAINED_TERRACOTTA_RED, TILE_STAINED_TERRACOTTA_RED], true, true, false, false, 0, SoundFamily::Stone),
    d("Black Terracotta", [TILE_STAINED_TERRACOTTA_BLACK, TILE_STAINED_TERRACOTTA_BLACK, TILE_STAINED_TERRACOTTA_BLACK], true, true, false, false, 0, SoundFamily::Stone),
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
/// hotbar (drink), never placeable.
/// 1.7.2 bracket: +55 — the 16 stained glass, 16 stained terracotta, red
/// sand, packed ice, podzol, acacia/dark-oak wood, the 8 new flowers,
/// the 4 two-block flowers (lower halves place both via worldgen-style
/// pairs; the picker offers the lower id) and the 4 fish items.
pub const PICKER_BLOCKS: [u8; 123] = [
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
    // 1.7.2 stained glass, all 16
    STAINED_GLASS_WHITE, STAINED_GLASS_ORANGE, STAINED_GLASS_MAGENTA,
    STAINED_GLASS_LIGHT_BLUE, STAINED_GLASS_YELLOW, STAINED_GLASS_LIME,
    STAINED_GLASS_PINK, STAINED_GLASS_GRAY, STAINED_GLASS_LIGHT_GRAY,
    STAINED_GLASS_CYAN, STAINED_GLASS_PURPLE, STAINED_GLASS_BLUE,
    STAINED_GLASS_BROWN, STAINED_GLASS_GREEN, STAINED_GLASS_RED,
    STAINED_GLASS_BLACK,
    // 1.7.2 stained terracotta, all 16
    STAINED_TERRACOTTA_WHITE, STAINED_TERRACOTTA_ORANGE, STAINED_TERRACOTTA_MAGENTA,
    STAINED_TERRACOTTA_LIGHT_BLUE, STAINED_TERRACOTTA_YELLOW, STAINED_TERRACOTTA_LIME,
    STAINED_TERRACOTTA_PINK, STAINED_TERRACOTTA_GRAY, STAINED_TERRACOTTA_LIGHT_GRAY,
    STAINED_TERRACOTTA_CYAN, STAINED_TERRACOTTA_PURPLE, STAINED_TERRACOTTA_BLUE,
    STAINED_TERRACOTTA_BROWN, STAINED_TERRACOTTA_GREEN, STAINED_TERRACOTTA_RED,
    STAINED_TERRACOTTA_BLACK,
    TALL_GRASS, FLOWER_RED, FLOWER_YELLOW, MUSHROOM_RED, MUSHROOM_BROWN,
    // 1.7.2 flowers + tall flowers
    ALLIUM, AZURE_BLUET, BLUE_ORCHID, OXEYE_DAISY,
    ORANGE_TULIP, RED_TULIP, WHITE_TULIP, PINK_TULIP,
    SUNFLOWER, LILAC, PEONY, ROSE_BUSH,
    // 1.7.2 fish (item-blocks: eatable, never placeable)
    RAW_FISH, RAW_SALMON, CLOWNFISH, PUFFERFISH,
    OAK_SLAB, COBBLE_STAIRS, OAK_FENCE,
    NETHERRACK, NETHER_QUARTZ_ORE, SOUL_SAND,
    BREWING_STAND,
    POTION_EMPTY, POTION_WATER, POTION_AWKWARD, POTION_MUNDANE, POTION_HEALING, POTION_HEALING_II,
    ENCHANT_TABLE, ENCHANTED_BOOK,
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
                || is_v2_state(s)
                || matches!(s, ACACIA_LOG_X | ACACIA_LOG_Z | DARK_OAK_LOG_X | DARK_OAK_LOG_Z)
            {
                assert!(!is_model_state(s), "component/item state {s} never routes to models");
                // identity: the state folds to the block whose def table
                // lists it (verified per-block in the dedicated ranges'
                // own tests)
                let b = state_block(s);
                assert!(b < BLOCK_COUNT as u8, "state {s} folds to valid block");
                // 1.7.2 V2: default_state must invert the fold exactly
                if is_v2_state(s) {
                    assert_eq!(default_state(b), s, "v2 state {s} roundtrip");
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
