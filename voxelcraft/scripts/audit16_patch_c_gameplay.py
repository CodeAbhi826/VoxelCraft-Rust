#!/usr/bin/env python3
"""Gameplay-systems patch — furnace/campfire/brewing/craft/effects/spawners/fishing."""
import sys

fail = []

def patch(path, edits):
    src = open(path).read()
    for what, old, new in edits:
        if old not in src or src.count(old) != 1:
            fail.append(f"[{path}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)

# ---------------- furnace.rs ----------------
patch("crates/vc-gameplay/src/furnace.rs", [
    (
        "is_food_smelting",
        """pub fn is_food_smelting(b: u16) -> bool {
    matches!(b, POTATO | RAW_RABBIT | KELP)
}""",
        """pub fn is_food_smelting(b: u16) -> bool {
    matches!(
        b,
        POTATO | RAW_RABBIT | KELP
            // the completeness audit: the raw-meat rows (the standing
            // cooked-meat deferral, closed — the smoker class VERIFIED
            // w/Smoker: "cooks food items twice as fast"; vanilla's
            // food-smelting rows)
            | BEEF
            | PORKCHOP
            | CHICKEN_RAW
            | MUTTON
            | RAW_FISH
            | RAW_SALMON
    )
}""",
    ),
    (
        "smelt_result",
        """        ANCIENT_DEBRIS => Some(NETHERITE_SCRAP),
        NETHER_GOLD_ORE => Some(IRON_ORE),
        _ => None,
    }
}""",
        """        ANCIENT_DEBRIS => Some(NETHERITE_SCRAP),
        NETHER_GOLD_ORE => Some(IRON_ORE),
        // ---- the 1.0-1.16.5 completeness audit (all VERIFIED live
        // 2026-09-08 against the audit16 captures / the vanilla food
        // smelting rows) ----
        // the cooked-meat family: the standing deferral, closed
        // ("Steak ... 8" hunger, w/Food; smelting raw beef -> steak
        // etc. — the vanilla furnace rows)
        BEEF => Some(STEAK),
        PORKCHOP => Some(COOKED_PORKCHOP),
        CHICKEN_RAW => Some(COOKED_CHICKEN),
        MUTTON => Some(COOKED_MUTTON),
        RAW_FISH => Some(COOKED_COD),
        RAW_SALMON => Some(COOKED_SALMON),
        // 1.9: popped chorus fruit — "obtained by smelting chorus
        // fruit" (VERIFIED w/Popped_Chorus_Fruit)
        CHORUS_FRUIT => Some(POPPED_CHORUS_FRUIT),
        // the classic cactus -> green dye row (Cactus Green = the
        // engine's DYE_BASE + 13, the 1.12 palette's green)
        CACTUS => Some(DYE_BASE + 13),
        _ => None,
    }
}""",
    ),
])

# ---------------- campfire.rs ----------------
patch("crates/vc-gameplay/src/campfire.rs", [
    (
        "accepts",
        """    pub fn accepts(block: u16) -> bool {
        matches!(block, POTATO | RAW_RABBIT | KELP)
    }""",
        """    pub fn accepts(block: u16) -> bool {
        matches!(
            block,
            POTATO | RAW_RABBIT | KELP
                // the completeness audit: the meat forms have landed —
                // the set extends exactly as this deferral promised
                // ("The set extends automatically when the meat forms
                // land"); vanilla's campfire set is the smeltable
                // FOODS (raw meats + potato + kelp)
                | BEEF
                | PORKCHOP
                | CHICKEN_RAW
                | MUTTON
                | RAW_FISH
                | RAW_SALMON
        )
    }""",
    ),
])

# ---------------- brewing.rs ----------------
patch("crates/vc-gameplay/src/brewing.rs", [
    (
        "brew recipes",
        """    // NOTE: the redstone-dust EXTENDED forms (slow falling 4:00,
    // turtle master 3:00) are palette-absent — the engine has no
    // redstone-dust ITEM (disclosed; POTION_SLOW_FALLING_EXT exists as
    // a registry row for future rounds)
];""",
        """    // NOTE: the redstone-dust EXTENDED forms (slow falling 4:00,
    // turtle master 3:00) are palette-absent — the engine has no
    // redstone-dust ITEM (disclosed; POTION_SLOW_FALLING_EXT exists as
    // a registry row for future rounds)
    // ---- the 1.0-1.16.5 completeness audit: the 1.8 rabbit's-foot
    // deferral, unblocked (the foot item landed in 1.8; the 1.8 round
    // deferred "Potion of Leaping (brewing needs the rabbit's-foot
    // recipe hook)" — the hook is this table). VERIFIED (live
    // 2026-09-08, capture scripts/audit16_page_Potion.json — the
    // ingredient chart lists Rabbit's Foot and Ghast Tear; the
    // glowstone-enhanced forms follow the healing family's pattern) ----
    // leaping: awkward + rabbit's foot -> Potion of Leaping (3:00)
    BrewRecipe {
        input: POTION_AWKWARD,
        ingredient: RABBIT_FOOT,
        output: POTION_LEAPING,
    },
    // leaping II: glowstone enhancement (1:30)
    BrewRecipe {
        input: POTION_LEAPING,
        ingredient: GLOWSTONE,
        output: POTION_LEAPING_II,
    },
    // regeneration: awkward + ghast tear -> Potion of Regeneration
    // (0:45) — "Ghasts ... are the only source of ghast tears"
    // (VERIFIED w/Ghast; the ghast itself lands this round)
    BrewRecipe {
        input: POTION_AWKWARD,
        ingredient: GHAST_TEAR,
        output: POTION_REGEN,
    },
    // regeneration II: glowstone enhancement (0:22)
    BrewRecipe {
        input: POTION_REGEN,
        ingredient: GLOWSTONE,
        output: POTION_REGEN_II,
    },
];""",
    ),
    (
        "potion effects",
        """pub fn potion_effects(b: u16) -> &'static [(EffectKind, u8, i32)] {
    use EffectKind::{Resistance, Slowness, SlowFalling};
    match b {""",
        """pub fn potion_effects(b: u16) -> &'static [(EffectKind, u8, i32)] {
    use EffectKind::{JumpBoost, Regeneration, Resistance, Slowness, SlowFalling};
    match b {
        // ---- the completeness audit: the leaping + regeneration
        // families (the 1.8 deferral, unblocked) ----
        // "Gives the player Jump Boost I for 3:00" (the vanilla row)
        POTION_LEAPING => &[(JumpBoost, 0, 3600)],
        // Jump Boost II, 1:30 (the glowstone-enhanced form)
        POTION_LEAPING_II => &[(JumpBoost, 1, 1800)],
        // redstone-extended 8:00 (the registry-row convention, gated)
        POTION_LEAPING_LONG => &[(JumpBoost, 0, 9600)],
        // Regeneration I, 0:45 ("applied every 25 ticks" per the
        // engine's effect tick — the level-I row)
        POTION_REGEN => &[(Regeneration, 0, 900)],
        // Regeneration II, 0:22 (the glowstone-enhanced form)
        POTION_REGEN_II => &[(Regeneration, 1, 440)],
        // redstone-extended 1:30 (registry row, redstone-gated)
        POTION_REGEN_LONG => &[(Regeneration, 0, 1800)],""",
    ),
])

# ---------------- effects.rs ----------------
patch("crates/vc-gameplay/src/effects.rs", [
    (
        "jump boost helper",
        """pub fn speed_multiplier(effects: &Effects) -> f32 {""",
        """/// Jump Boost launch bonus in b/s (VERIFIED w/Effect §Jump_Boost:
/// "+0.1 blocks per tick per level" added to the 0.42 b/t launch —
/// 0.1 b/t x 20 = +2.0 b/s per level; the engine's JUMP_VEL is 8.4
/// b/s = 0.42 b/t, so level I launches at 10.4 b/s). Applied at the
/// jump sites (player.rs).
pub fn jump_boost_bonus(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::JumpBoost)
        .map(|a| 2.0 * (a as f32 + 1.0))
        .unwrap_or(0.0)
}

pub fn speed_multiplier(effects: &Effects) -> f32 {""",
    ),
])

# ---------------- spawners.rs ----------------
patch("crates/vc-gameplay/src/spawners.rs", [
    (
        "mob_kind arms",
        """        6 => MobKind::Evoker, // 1.11 mansion upper floors (w/Evoker)
        _ => MobKind::Zombie,""",
        """        6 => MobKind::Evoker, // 1.11 mansion upper floors (w/Evoker)
        // the completeness audit: the mineshaft cave-spider spawner
        // (replaces gen.rs's disclosed spider-spawner adaptation) and
        // the stronghold portal-room silverfish spawner
        7 => MobKind::CaveSpider,
        8 => MobKind::Silverfish,
        _ => MobKind::Zombie,""",
    ),
])

# ---------------- fishing.rs ----------------
patch("crates/vc-gameplay/src/fishing.rs", [
    (
        "bowl row",
        '    LootRow { item: LOOT_NONE, name: "Bowl" },',
        '''    // the completeness audit: the bowl item now exists (the V15
    // window) — the junk row's palette-absent marker is retired
    LootRow { item: BOWL, name: "Bowl" },''',
    ),
])

print("FAILED:" if fail else "all gameplay patches applied")
for f in fail:
    print("  -", f)
sys.exit(1 if fail else 0)
