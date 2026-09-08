#!/usr/bin/env python3
"""V15 window patch for blocks.rs — the 1.0-1.16.5 completeness audit round.

Adds:
- 26 new item ids (479..=504): the cooked-meat family, the kitchen chain
  (apple/bowl/stews/beetroot/sugar/egg/poisonous potato), popped chorus
  fruit, ghast tear, the leaping/regeneration potion rows, and the
  ghast/cave-spider/silverfish spawn eggs
- 28 new states (776..=803): 26 identity item states + the cave-spider
  and silverfish spawner states
- 30 new tiles (706..=734)
- egg decode arms (kinds 45..=47), spawner state constants
"""
import re, sys

PATH = "crates/vc-blocks/src/blocks.rs"
src = open(PATH).read()
orig = src
fail = []

def sub_once(old, new, what):
    global src
    if old not in src:
        fail.append(f"ANCHOR NOT FOUND ({what}): {old[:90]!r}")
        return
    if src.count(old) != 1:
        fail.append(f"ANCHOR NOT UNIQUE ({what})")
        return
    src = src.replace(old, new)

# ------------------------------------------------------------------
# 1) TILE constants — after TILE_MOB_HOGLIN
# ------------------------------------------------------------------
sub_once(
    "/// the hoglin mob billboard sprite (v116b_art::hoglin_art).\npub const TILE_MOB_HOGLIN: u16 = 705;\n",
    """/// the hoglin mob billboard sprite (v116b_art::hoglin_art).
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
""",
    "tile constants",
)

# ------------------------------------------------------------------
# 2) BLOCK id constants — after SPAWN_EGG_HOGLIN
# ------------------------------------------------------------------
sub_once(
    "pub const SPAWN_EGG_HOGLIN: u16 = 478;",
    """pub const SPAWN_EGG_HOGLIN: u16 = 478;

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
pub const SPAWN_EGG_SILVERFISH: u16 = 504;""",
    "block id constants",
)

# ------------------------------------------------------------------
# 3) V15 state window — after is_v14_state
# ------------------------------------------------------------------
sub_once(
    """#[inline]
pub fn is_v14_state(s: u16) -> bool {
    (V14_STATE_BASE..V14_STATE_BASE + V14_COUNT).contains(&s)
}""",
    """#[inline]
pub fn is_v14_state(s: u16) -> bool {
    (V14_STATE_BASE..V14_STATE_BASE + V14_COUNT).contains(&s)
}

// ---- the 1.0-1.16.5 completeness audit: the V15 state window ----
// 28 states: the 26 V15 item rows as identity states, plus the two
// new spawner states (the cave-spider / silverfish codes — the
// dedicated-state pattern of SPAWNER_BLAZE 241, riding the window).
pub const V15_STATE_BASE: u16 = 776;
pub const V15_COUNT: u16 = 28;
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
    MONSTER_SPAWNER, // the cave-spider spawner state
    MONSTER_SPAWNER, // the silverfish spawner state
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
        _ => None,
    }
}

#[inline]
pub fn is_v15_state(s: u16) -> bool {
    (V15_STATE_BASE..V15_STATE_BASE + V15_COUNT).contains(&s)
}

/// the 1.0-1.16.5 audit's cave-spider spawner state (kind code 7 —
/// the mineshaft spawner's mob; replaces the pre-audit spider-spawner
/// adaptation that gen.rs documented as "no distinct cave-spider mob").
pub const SPAWNER_CAVESPIDER: u16 = V15_STATE_BASE + 26;
/// the audit's silverfish spawner state (kind code 8 — the stronghold
/// portal-room spawner's mob).
pub const SPAWNER_SILVERFISH: u16 = V15_STATE_BASE + 27;""",
    "V15 state window",
)

# ------------------------------------------------------------------
# 4) state_block — V15 fold arm after the V14 arm
# ------------------------------------------------------------------
sub_once(
    """        s if is_v14_state(s) => {
            return V14_STATE_TO_BLOCK[(s - V14_STATE_BASE) as usize];
        }""",
    """        s if is_v14_state(s) => {
            return V14_STATE_TO_BLOCK[(s - V14_STATE_BASE) as usize];
        }
        // the 1.0-1.16.5 completeness audit: the V15 window — 26
        // identity item folds + the two spawner states folding to the
        // Monster Spawner block
        s if is_v15_state(s) => {
            return V15_STATE_TO_BLOCK[(s - V15_STATE_BASE) as usize];
        }""",
    "state_block V15 arm",
)

# ------------------------------------------------------------------
# 5) default_state — v15 arm after the v14 arm
# ------------------------------------------------------------------
sub_once(
    "        b if v14_state(b).is_some() => v14_state(b).unwrap(),",
    """        b if v14_state(b).is_some() => v14_state(b).unwrap(),
        // the completeness audit: V15 identity item states
        b if v15_state(b).is_some() => v15_state(b).unwrap(),""",
    "default_state V15 arm",
)

# ------------------------------------------------------------------
# 6) BLOCK_COUNT / STATE_COUNT / TILE_MAX
# ------------------------------------------------------------------
sub_once(
    "pub const BLOCK_COUNT: usize = 479; // 1.16 (Nether Update, part 2): V14 window ids 454..=478 (crimson/warped stems, hyphae, planks, nylium, fungi, roots, vines, warped wart block, shroomlight, nether sprouts, polished basalt/blackstone/bricks, soul torch/lantern + the strider/piglin/hoglin eggs)",
    "pub const BLOCK_COUNT: usize = 505; // the 1.0-1.16.5 completeness audit: V15 window ids 479..=504 (the cooked-meat family, the kitchen chain — apple/bowl/mushroom-rabbit-beetroot stews/sugar/egg/poisonous potato — popped chorus, ghast tear, the leaping/regeneration potions, and the ghast/cave-spider/silverfish spawn eggs)",
    "BLOCK_COUNT",
)
sub_once(
    "pub const STATE_COUNT: usize = 776; // 1.16 (Nether Update, part 2): V14 states 750..=775 (the 22 crimson/warped family identity states, the soul lantern's sitting/hanging pair, and the 3 spawn-egg item states)",
    "pub const STATE_COUNT: usize = 804; // the completeness audit: V15 states 776..=803 (26 identity item states + the cave-spider/silverfish spawner states)",
    "STATE_COUNT",
)
sub_once(
    "pub const TILE_MAX: u16 = 705; // 1.16 (Nether Update, part 2): tiles 673..=705 (the V14 window — the crimson/warped families, shroomlight, polished stones, soul torch/lantern, the strider/piglin/hoglin eggs + mob sprites)",
    "pub const TILE_MAX: u16 = 734; // the completeness audit: tiles 706..=734 (the V15 window — the cooked meats, the kitchen items, the leaping/regen potions, the ghast/cave-spider/silverfish eggs + mob sprites)",
    "TILE_MAX",
)

# ------------------------------------------------------------------
# 7) BLOCK_TABLE rows — after the Hoglin Spawn Egg row
# ------------------------------------------------------------------
sub_once(
    '    d("Hoglin Spawn Egg", [TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN], false, false, true, false, 0, SoundFamily::Grass),',
    '''    d("Hoglin Spawn Egg", [TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN, TILE_SPAWN_EGG_HOGLIN], false, false, true, false, 0, SoundFamily::Grass),
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
    d("Silverfish Spawn Egg", [TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH], false, false, true, false, 0, SoundFamily::Grass),''',
    "BLOCK_TABLE rows",
)

# ------------------------------------------------------------------
# 8) is_item_block — V15 arm before the closing
# ------------------------------------------------------------------
sub_once(
    """            | NETHERITE_SCRAP
            | NETHERITE_INGOT
    ) || is_spawn_egg(b)""",
    """            | NETHERITE_SCRAP
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
            | POISONOUS_POTATO
            | POPPED_CHORUS_FRUIT
            | GHAST_TEAR
            | POTION_LEAPING
            | POTION_LEAPING_II
            | POTION_LEAPING_LONG
            | POTION_REGEN
            | POTION_REGEN_II
            | POTION_REGEN_LONG
    ) || is_spawn_egg(b)""",
    "is_item_block V15",
)

# ------------------------------------------------------------------
# 9) is_spawn_egg + egg_mob — the three new eggs
# ------------------------------------------------------------------
sub_once(
    """        || b == SPAWN_EGG_STRIDER
        || b == SPAWN_EGG_PIGLIN
        || b == SPAWN_EGG_HOGLIN
}""",
    """        || b == SPAWN_EGG_STRIDER
        || b == SPAWN_EGG_PIGLIN
        || b == SPAWN_EGG_HOGLIN
        // the completeness audit: the three classic-mob eggs (the
        // ghast/cave spider/silverfish rounds)
        || b == SPAWN_EGG_GHAST
        || b == SPAWN_EGG_CAVE_SPIDER
        || b == SPAWN_EGG_SILVERFISH
}""",
    "is_spawn_egg V15",
)
sub_once(
    """    if b == SPAWN_EGG_HOGLIN {
        return Some(44);
    }
    if !is_spawn_egg(b) {
        return None;
    }""",
    """    if b == SPAWN_EGG_HOGLIN {
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
    }""",
    "egg_mob V15",
)

# ------------------------------------------------------------------
# 10) spawner_mob — the two new spawner state arms
# ------------------------------------------------------------------
sub_once(
    """    } else if s == SPAWNER_EVOKER {
        6 // 1.11 mansion upper floors
    } else {""",
    """    } else if s == SPAWNER_EVOKER {
        6 // 1.11 mansion upper floors
    } else if s == SPAWNER_CAVESPIDER {
        7 // the completeness audit: the mineshaft spawner's own mob
          // (replaces the spider-spawner adaptation gen.rs disclosed)
    } else if s == SPAWNER_SILVERFISH {
        8 // the audit: the stronghold portal-room spawner's mob
    } else {""",
    "spawner_mob V15",
)

# ------------------------------------------------------------------
# 11) PICKER_BLOCKS — append the V15 items (432 -> 458) + fix len
# ------------------------------------------------------------------
sub_once(
    "    SPAWN_EGG_STRIDER, SPAWN_EGG_PIGLIN, SPAWN_EGG_HOGLIN,\n];",
    """    SPAWN_EGG_STRIDER, SPAWN_EGG_PIGLIN, SPAWN_EGG_HOGLIN,
    // ---- the 1.0-1.16.5 completeness audit: the V15 items ----
    STEAK, COOKED_PORKCHOP, COOKED_CHICKEN, COOKED_MUTTON, COOKED_COD, COOKED_SALMON,
    APPLE, BOWL, MUSHROOM_STEW, RABBIT_STEW, BEETROOT, BEETROOT_SOUP, SUGAR, EGG,
    POISONOUS_POTATO, POPPED_CHORUS_FRUIT, GHAST_TEAR,
    POTION_LEAPING, POTION_LEAPING_II, POTION_LEAPING_LONG,
    POTION_REGEN, POTION_REGEN_II, POTION_REGEN_LONG,
    SPAWN_EGG_GHAST, SPAWN_EGG_CAVE_SPIDER, SPAWN_EGG_SILVERFISH,
];""",
    "PICKER_BLOCKS append",
)
sub_once(
    "pub const PICKER_BLOCKS: [u16; 432] = [",
    "pub const PICKER_BLOCKS: [u16; 458] = [",
    "PICKER_BLOCKS len",
)

# ------------------------------------------------------------------
# 12) window-shape test updates
# ------------------------------------------------------------------
sub_once(
    """        // bounds + window shape
        assert_eq!(V14_COUNT, 26);
        assert_eq!(V14_STATE_BASE + V14_COUNT, 776);
        assert_eq!(BLOCK_COUNT, 479);
        assert_eq!(STATE_COUNT, 776);
        assert_eq!(PICKER_BLOCKS.len(), 432);
    }
}""",
    """        // bounds + window shape
        assert_eq!(V14_COUNT, 26);
        assert_eq!(V14_STATE_BASE + V14_COUNT, 776);
        // the completeness audit: the V15 window (26 items + the two
        // spawner states)
        assert_eq!(V15_COUNT, 28);
        assert_eq!(V15_STATE_BASE + V15_COUNT, 804);
        assert_eq!(BLOCK_COUNT, 505);
        assert_eq!(STATE_COUNT, 804);
        assert_eq!(PICKER_BLOCKS.len(), 458);
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
        ] {
            assert_eq!(default_state(b), s, "block {b} default state");
            assert_eq!(state_block(s), b, "state {s} folds back");
            assert!(is_item_block(b), "block {b} is an item block");
        }
        // the two spawner states: fold to the spawner block + decode
        assert_eq!(state_block(SPAWNER_CAVESPIDER), MONSTER_SPAWNER);
        assert_eq!(state_block(SPAWNER_SILVERFISH), MONSTER_SPAWNER);
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
}""",
    "window-shape test + V15 test",
)

if fail:
    print("FAILED ANCHORS:")
    for f in fail:
        print("  -", f)
    sys.exit(1)

open(PATH, "w").write(src)
print(f"blocks.rs patched OK ({orig and len(orig)} -> {len(src)} chars)")
