#!/usr/bin/env python3
"""game.rs patch — the completeness audit's food system + mob drops + apple roll."""
import sys

PATH = "crates/voxelcraft/src/game.rs"
src = open(PATH).read()
fail = []

def sub_once(old, new, what):
    global src
    if old not in src or src.count(old) != 1:
        fail.append(f"ANCHOR ({what}): {old[:70]!r}")
        return
    src = src.replace(old, new)

# ------------------------------------------------------------------
# 1) is_food — the audit round's additions (+ the cookie bug fix)
# ------------------------------------------------------------------
sub_once(
    """            // 1.14: sweet berries — "restores 2 hunger and 0.4 [JE]
            // saturation" (VERIFIED w/Sweet_Berries §Food)
            | SWEET_BERRIES
    )
}""",
    """            // 1.14: sweet berries — "restores 2 hunger and 0.4 [JE]
            // saturation" (VERIFIED w/Sweet_Berries §Food)
            | SWEET_BERRIES
            // ---- the 1.0-1.16.5 completeness audit (all hunger values
            // VERIFIED live 2026-09-08 against the Food page capture
            // scripts/audit16_page_Food.json — the hunger table) ----
            // the cooked-meat family (the standing deferral, closed)
            | STEAK
            | COOKED_PORKCHOP
            | COOKED_CHICKEN
            | COOKED_MUTTON
            | COOKED_COD
            | COOKED_SALMON
            // the kitchen chain: apple (hunger 4), the stews (6 / 10 /
            // 6), the beetroot (1), the poisonous potato (2)
            | APPLE
            | MUSHROOM_STEW
            | RABBIT_STEW
            | BEETROOT
            | BEETROOT_SOUP
            | POISONOUS_POTATO
            // the audit bug fix: the cookie has existed since the 1.12
            // parrot round but was never edible (hunger 2 — the Food
            // table's "Cookie 2" row)
            | COOKIE
    )
}""",
    "is_food",
)

# ------------------------------------------------------------------
# 2) food_heal — the hunger/2 values
# ------------------------------------------------------------------
sub_once(
    """        SWEET_BERRIES => 1.0,
        _ => 4.0, // the meats' established value
    }
}""",
    """        SWEET_BERRIES => 1.0,
        // ---- the completeness audit: the hunger/2 mapping (the Food
        // table capture's own rows — "Rabbit Stew 10 / Steak 8 /
        // Cooked Porkchop 8 / Beetroot Soup 6 / Cooked Chicken 6 /
        // Cooked Mutton 6 / Cooked Salmon 6 / ... Cooked Cod 5 /
        // Apple 4 / Cookie 2 / Beetroot 1") ----
        STEAK => 4.0,
        COOKED_PORKCHOP => 4.0,
        COOKED_CHICKEN => 3.0,
        COOKED_MUTTON => 3.0,
        COOKED_COD => 2.5,
        COOKED_SALMON => 3.0,
        APPLE => 2.0,
        MUSHROOM_STEW => 3.0,
        RABBIT_STEW => 5.0, // hunger 10 — the biggest single-food heal
        BEETROOT => 0.5,
        BEETROOT_SOUP => 3.0,
        POISONOUS_POTATO => 1.0,
        // the cookie bug fix: hunger 2 -> 1.0 (was falling to the 4.0
        // default — an inedible item's value never mattered before)
        COOKIE => 1.0,
        _ => 4.0, // the raw meats' established value
    }
}""",
    "food_heal",
)

# ------------------------------------------------------------------
# 3) the eat path — the stew bowl return + the poisonous-potato roll
# ------------------------------------------------------------------
sub_once(
    """                        if !self.mode.invulnerable() {
                            self.player.heal(heal_amt);
                        }
                        self.play_event("entity.generic.drink", None, 0.8);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: ate {} (+{heal_amt} hp -> {})",
                            name(b),
                            self.player.health
                        ));""",
    """                        if !self.mode.invulnerable() {
                            self.player.heal(heal_amt);
                        }
                        // the completeness audit: the stews return their
                        // bowl (the honey-bottle precedent returns the
                        // glass bottle — VERIFIED w/Mushroom_Stew /
                        // w/Rabbit_Stew / w/Beetroot_Soup: "the bowl is
                        // returned after eating"; inventory-full drops
                        // it at the player's feet)
                        if matches!(b, MUSHROOM_STEW | RABBIT_STEW | BEETROOT_SOUP) {
                            let left = self.player.inv.add(BOWL, 1);
                            if left > 0 {
                                self.sim.items.drop_block(
                                    self.player.pos[0] as i32,
                                    self.player.pos[1] as i32,
                                    self.player.pos[2] as i32,
                                    BOWL,
                                    2,
                                    15,
                                    0,
                                );
                            }
                        }
                        // the audit: the poisonous potato — "a 60% chance
                        // of applying 5 seconds of Poison I" (VERIFIED
                        // w/Poisonous_Potato; the pufferfish's exact-
                        // poison precedent, level I this time)
                        if b == POISONOUS_POTATO
                            && !self.mode.invulnerable()
                            && self.audio_rng.next_f32() < 0.6
                        {
                            self.player
                                .effects
                                .apply(vc_gameplay::effects::EffectKind::Poison, 0, 100);
                            self.ui.dirty = true;
                        }
                        self.play_event("entity.generic.drink", None, 0.8);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: ate {} (+{heal_amt} hp -> {})",
                            name(b),
                            self.player.health
                        ));""",
    "eat path extras",
)

# ------------------------------------------------------------------
# 4) the apple leaf roll — before the default self-drop arm
# ------------------------------------------------------------------
sub_once(
    """                            } else if broke == BEE_NEST || broke == BEEHIVE {""",
    """                            } else if broke == LEAVES || broke == DARK_OAK_LEAVES {
                                // the completeness audit: the apple roll
                                // — VERIFIED (minecraft.wiki/w/Apple, live
                                // 2026-09-08): "Oak and dark oak leaves
                                // have a 0.5% (1/200) chance of dropping
                                // an apple when decayed or broken, but
                                // not if burned". The engine's leaves
                                // self-drop convention is unchanged; the
                                // apple rides as the bonus roll (only
                                // the two apple-bearing species — the
                                // other four leaves never drop apples,
                                // VERIFIED).
                                self.sim.items.drop_block(
                                    pos[0], pos[1], pos[2], broke, biome, sky, blk,
                                );
                                if self.audio_rng.next_range(200) == 0 {
                                    self.sim.items.drop_block(
                                        pos[0], pos[1], pos[2], APPLE, biome, sky, blk,
                                    );
                                }
                            } else if broke == BEE_NEST || broke == BEEHIVE {""",
    "apple leaf roll",
)

# ------------------------------------------------------------------
# 5) the mob-drop arms — the audit trio
# ------------------------------------------------------------------
sub_once(
    """                // 1.15: bees drop no items (VERIFIED w/Bee §Drops —
                // only 1-3 XP, already granted via the XP orb path)
                mobs::MobKind::Bee => &[],""",
    """                // 1.15: bees drop no items (VERIFIED w/Bee §Drops —
                // only 1-3 XP, already granted via the XP orb path)
                mobs::MobKind::Bee => &[],
                // ---- the 1.0-1.16.5 completeness audit trio ----
                // the cave spider: string 0-2 (VERIFIED w/Cave_Spider
                // §Drops: the spider-family rows) + the spider-eye roll
                // below (the 1/3 row, same as the spider's)
                mobs::MobKind::CaveSpider => &[(STRING, 2)],
                // the ghast: gunpowder + the ghast tear — both are
                // percentage rows handled below (the blaze/phantom
                // pattern)
                mobs::MobKind::Ghast => &[],
                // the silverfish: "Silverfish have no drops other than
                // 5 XP experience points" (VERIFIED w/Silverfish §Drops
                // — the XP rides the orb path)
                mobs::MobKind::Silverfish => &[],""",
    "drop arms",
)

# 6) the spider-eye roll — the cave spider joins
sub_once(
    """            // Phase 4 §26: spiders additionally have a 1/3 chance to drop
            // one spider eye (VERIFIED, 1.16.5-era Spider page — only when
            // killed by a player, which drain_mob_events is)
            if kind == mobs::MobKind::Spider && self.audio_rng.next_f32() < 1.0 / 3.0 {""",
    """            // Phase 4 §26: spiders additionally have a 1/3 chance to drop
            // one spider eye (VERIFIED, 1.16.5-era Spider page — only when
            // killed by a player, which drain_mob_events is). The
            // completeness audit: the cave spider's own drops row is the
            // same "Spider Eye 0-1 33.33%" (VERIFIED w/Cave_Spider).
            if matches!(kind, mobs::MobKind::Spider | mobs::MobKind::CaveSpider)
                && self.audio_rng.next_f32() < 1.0 / 3.0
            {""",
    "spider eye roll",
)

# 7) the ghast's percentage drops — after the phantom membrane block
sub_once(
    """            // ---- 1.16 (Nether Update, part 2): the forest mobs' exact
            // drop rolls ----""",
    """            // ---- the completeness audit: the ghast's exact drop rows
            // (VERIFIED w/Ghast §Drops, live 2026-09-08): "Ghast Tear
            // 0-1 50.00%" + "Gunpowder 0-2 66.67%" (the uniform 0-2
            // whose P(>=1) is the printed 2/3; the Music Disc "Tears"
            // row is trimmed with the engine's no-music-disc class,
            // disclosed) ----
            if kind == mobs::MobKind::Ghast {
                if self.audio_rng.next_range(2) == 1 {
                    self.sim.items.drop_block(
                        pos[0].floor() as i32,
                        pos[1].floor() as i32,
                        pos[2].floor() as i32,
                        GHAST_TEAR,
                        2,
                        15,
                        0,
                    );
                }
                let n = self.audio_rng.next_range(3) as u8; // 0..=2
                for _ in 0..n {
                    self.sim.items.drop_block(
                        pos[0].floor() as i32,
                        pos[1].floor() as i32,
                        pos[2].floor() as i32,
                        GUNPOWDER,
                        2,
                        15,
                        0,
                    );
                }
            }
            // ---- 1.16 (Nether Update, part 2): the forest mobs' exact
            // drop rolls ----""",
    "ghast percentage drops",
)

if fail:
    print("FAILED ANCHORS:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
open(PATH, "w").write(src)
print(f"game.rs patched OK ({len(src)} chars)")
