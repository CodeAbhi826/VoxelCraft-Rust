#!/usr/bin/env python3
"""Test patch 3 — fishing/mobs/gen/game tests."""
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

# ---------------- fishing.rs ----------------
patch("crates/vc-gameplay/src/fishing.rs", [
    (
        "fishing bowl test",
        """        // enchanted book is the one treasure we CAN award today
        assert!(TREASURE_TABLE.iter().any(|r| r.item == ENCHANTED_BOOK));
    }
}""",
        """        // enchanted book is the one treasure we CAN award today
        assert!(TREASURE_TABLE.iter().any(|r| r.item == ENCHANTED_BOOK));
    }

    /// the completeness audit: the bowl junk row is wired to the real
    /// BOWL item (the palette-absent marker retired — "Bowls are
    /// obtainable through fishing", VERIFIED w/Bowl §Fishing)
    #[test]
    fn audit16_bowl_junk_row() {
        assert!(JUNK_TABLE.iter().any(|r| r.item == BOWL));
    }
}""",
    ),
])

# ---------------- mobs.rs ----------------
patch("crates/vc-gameplay/src/mobs.rs", [
    (
        "mobs tests",
        """        assert_eq!(MobKind::Hoglin.sprite_tile(), TILE_MOB_HOGLIN);""",
        """        assert_eq!(MobKind::Hoglin.sprite_tile(), TILE_MOB_HOGLIN);
        // the completeness audit trio: sprites + names + hostility
        assert_eq!(MobKind::Ghast.sprite_tile(), TILE_MOB_GHAST);
        assert_eq!(MobKind::CaveSpider.sprite_tile(), TILE_MOB_CAVESPIDER);
        assert_eq!(MobKind::Silverfish.sprite_tile(), TILE_MOB_SILVERFISH);
        assert_eq!(MobKind::Ghast.name(), "minecraft:ghast");
        assert_eq!(MobKind::CaveSpider.name(), "minecraft:cave_spider");
        assert_eq!(MobKind::Silverfish.name(), "minecraft:silverfish");
        assert!(MobKind::Ghast.hostile());
        assert!(MobKind::CaveSpider.hostile());
        assert!(MobKind::Silverfish.hostile());
        assert!(MobKind::Ghast.flies());
        assert!(!MobKind::Silverfish.flies());
        assert_eq!(MobKind::from_name("ghast"), Some(MobKind::Ghast));
        assert_eq!(MobKind::from_name("cave_spider"), Some(MobKind::CaveSpider));
        assert_eq!(MobKind::from_name("silverfish"), Some(MobKind::Silverfish));
        // the egg window: kinds 45..=47 roundtrip
        assert_eq!(MobKind::from_egg(45), MobKind::Ghast);
        assert_eq!(MobKind::from_egg(46), MobKind::CaveSpider);
        assert_eq!(MobKind::from_egg(47), MobKind::Silverfish);
        assert_eq!(MobKind::Ghast.egg_id(), 45);
        assert_eq!(MobKind::CaveSpider.egg_id(), 46);
        assert_eq!(MobKind::Silverfish.egg_id(), 47);
        // the verified infobox rows (audit16_page_{Ghast,Cave_Spider,
        // Silverfish}.json)
        let gh = def(MobKind::Ghast);
        assert_eq!(gh.health, 10.0);
        assert_eq!(gh.damage, 6.0, "fireball impact Normal (VERIFIED)");
        assert_eq!((gh.height, gh.width), (4.0, 4.0), "the 4x4x4 hitbox");
        let cs = def(MobKind::CaveSpider);
        assert_eq!(cs.health, 12.0);
        assert_eq!(cs.damage, 2.0, "Normal melee (VERIFIED)");
        assert_eq!((cs.height, cs.width), (0.5, 0.7));
        let sf = def(MobKind::Silverfish);
        assert_eq!(sf.health, 8.0);
        assert_eq!(sf.damage, 1.0, "Easy/Normal attack (VERIFIED)");
        assert_eq!((sf.height, sf.width), (0.3, 0.4));
        assert_eq!(sf.xp, 5, "\\"no drops other than 5 XP\\" (VERIFIED)");""",
    ),
])

# ---------------- game.rs ----------------
patch("crates/voxelcraft/src/game.rs", [
    (
        "game food tests",
        """    /// golden carrot heals hunger 6 / 2 = 3.0 HP (VERIFIED live
    /// 2026-09-07 w/Golden_Carrot: "Hunger 6", "Saturation 14.4")
    #[test]
    fn golden_carrot_food_values() {
        assert_eq!(food_heal(GOLDEN_CARROT), 3.0);""",
        """    /// the completeness audit: the hunger/2 table for the whole V15
    /// kitchen (all VERIFIED live 2026-09-08 against the Food page
    /// capture scripts/audit16_page_Food.json)
    #[test]
    fn audit16_food_values() {
        // the cooked-meat family
        assert_eq!(food_heal(STEAK), 4.0, "hunger 8");
        assert_eq!(food_heal(COOKED_PORKCHOP), 4.0, "hunger 8");
        assert_eq!(food_heal(COOKED_CHICKEN), 3.0, "hunger 6");
        assert_eq!(food_heal(COOKED_MUTTON), 3.0, "hunger 6");
        assert_eq!(food_heal(COOKED_COD), 2.5, "hunger 5");
        assert_eq!(food_heal(COOKED_SALMON), 3.0, "hunger 6");
        // the kitchen chain
        assert_eq!(food_heal(APPLE), 2.0, "hunger 4");
        assert_eq!(food_heal(MUSHROOM_STEW), 3.0, "hunger 6");
        assert_eq!(food_heal(RABBIT_STEW), 5.0, "hunger 10 — the top food");
        assert_eq!(food_heal(BEETROOT), 0.5, "hunger 1");
        assert_eq!(food_heal(BEETROOT_SOUP), 3.0, "hunger 6");
        assert_eq!(food_heal(POISONOUS_POTATO), 1.0, "hunger 2");
        // the cookie bug fix (hunger 2; was falling to the 4.0 default)
        assert_eq!(food_heal(COOKIE), 1.0, "hunger 2 — the 1.12 cookie");
        // and everything is actually food now
        for b in [
            STEAK, COOKED_PORKCHOP, COOKED_CHICKEN, COOKED_MUTTON, COOKED_COD,
            COOKED_SALMON, APPLE, MUSHROOM_STEW, RABBIT_STEW, BEETROOT,
            BEETROOT_SOUP, POISONOUS_POTATO, COOKIE,
        ] {
            assert!(is_food(b), "block {b} must be food");
        }
    }

    /// golden carrot heals hunger 6 / 2 = 3.0 HP (VERIFIED live
    /// 2026-09-07 w/Golden_Carrot: "Hunger 6", "Saturation 14.4")
    #[test]
    fn golden_carrot_food_values() {
        assert_eq!(food_heal(GOLDEN_CARROT), 3.0);""",
    ),
])

print("FAILED:" if fail else "test patch 3 applied (fishing/mobs/game)")
for f in fail:
    print("  -", f)
sys.exit(1 if fail else 0)
