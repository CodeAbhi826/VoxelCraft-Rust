#!/usr/bin/env python3
"""Sweep-2 patch J — game.rs: the food effects + the throw branch + the
landing drain + the chorus teleport + the melon break + the e2e stage.

All values live-verified 2026-09-09 (captures: Rotten_Flesh, Spider_Eye,
Chorus_Fruit, Golden_Apple, Ender_Pearl, Egg, Snowball, Melon_Slice).
"""
import sys

fail = []


def patch(path, edits, tag):
    src = open(path).read()
    for what, old, new in edits:
        if new in src:
            continue
        if old not in src or src.count(old) != 1:
            fail.append(f"[{tag}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)


# ======================================================================
# 1) food_heal arms
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "food_heal arms",
        """        // the cookie bug fix: hunger 2 -> 1.0 (was falling to the 4.0
        // default — an inedible item's value never mattered before)
        COOKIE => 1.0,
        _ => 4.0, // the raw meats' established value""",
        """        // the cookie bug fix: hunger 2 -> 1.0 (was falling to the 4.0
        // default — an inedible item's value never mattered before)
        COOKIE => 1.0,
        // ---- the sweep-2 rows (VERIFIED live 2026-09-09 against the
        // fresh captures: Rotten_Flesh "Hunger 4", Spider_Eye "Hunger
        // 2", Chorus_Fruit "Hunger 4", Golden_Apple "Hunger 4",
        // Melon_Slice "Hunger 2") ----
        ROTTEN_FLESH => 2.0,
        SPIDER_EYE => 1.0,
        CHORUS_FRUIT => 2.0,
        GOLDEN_APPLE => 2.0,
        MELON_SLICE => 1.0,
        _ => 4.0, // the raw meats' established value""",
    ),
    (
        "is_food rows",
        """            // the audit bug fix: the cookie has existed since the 1.12
            // parrot round but was never edible (hunger 2 — the Food
            // table's "Cookie 2" row)
            | COOKIE
    )
}""",
        """            // the audit bug fix: the cookie has existed since the 1.12
            // parrot round but was never edible (hunger 2 — the Food
            // table's "Cookie 2" row)
            | COOKIE
            // ---- the sweep-2 rows (hunger values VERIFIED live
            // 2026-09-09: Rotten_Flesh 4, Spider_Eye 2, Chorus_Fruit
            // 4, Golden_Apple 4, Melon_Slice 2 — the pages above) ----
            | ROTTEN_FLESH
            | SPIDER_EYE
            | CHORUS_FRUIT
            | GOLDEN_APPLE
            | MELON_SLICE
    )
}""",
    ),
    (
        "eat-path effects",
        """                        // the audit: the poisonous potato — "a 60% chance
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
                        }""",
        """                        // the audit: the poisonous potato — "a 60% chance
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
                        // ---- the sweep-2 effect rows (all VERIFIED live
                        // 2026-09-09, same-capture pages) ----
                        // rotten flesh: "Hunger (0:30) (80% chance)"
                        if b == ROTTEN_FLESH
                            && !self.mode.invulnerable()
                            && self.audio_rng.next_f32() < 0.8
                        {
                            self.player
                                .effects
                                .apply(vc_gameplay::effects::EffectKind::Hunger, 0, 600);
                            self.ui.dirty = true;
                        }
                        // spider eye: "Poison (0:05)" — always
                        if b == SPIDER_EYE && !self.mode.invulnerable() {
                            self.player
                                .effects
                                .apply(vc_gameplay::effects::EffectKind::Poison, 0, 100);
                            self.ui.dirty = true;
                        }
                        // golden apple: "Absorption (2:00)" +
                        // "Regeneration II (0:05)"
                        if b == GOLDEN_APPLE && !self.mode.invulnerable() {
                            self.player
                                .effects
                                .apply(vc_gameplay::effects::EffectKind::Absorption, 0, 2400);
                            self.player
                                .effects
                                .apply(vc_gameplay::effects::EffectKind::Regeneration, 1, 100);
                            self.ui.dirty = true;
                        }
                        // the chorus teleport: "up to 16 attempts are
                        // made to choose a random destination within
                        // ±8 on all three axes in the same manner as
                        // enderman teleportation" (VERIFIED
                        // w/Chorus_Fruit §Teleportation) — runs after
                        // the heal, exactly vanilla's eat-then-warp
                        if b == CHORUS_FRUIT {
                            self.chorus_teleport();
                        }""",
    ),
], "game-food")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch j part 1 applied: food rows + effects")
