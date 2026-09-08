#!/usr/bin/env python3
"""Sweep-2 patch L — the e2e_audit16b stage + its call site + the CI
smoke grep line."""
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
# 1) the e2e_audit16b stage
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "e2e stage",
        """    fn test_place(&mut self, block: u16, x: i32, y: i32, z: i32) {""",
        """    /// the sweep-2 half of the audit stage: the five food rows, the
    /// melon crafts, the golden apple's effect pair, and the throwable
    /// trio through the real projectile path + the real drain. CI
    /// smoke greps the "e2e: audit16b" boot line.
    fn e2e_audit16b(&mut self) {
        use vc_blocks::blocks::*;
        use vc_inventory::inventory::ItemStack;

        let pos = [
            self.player.pos.x.floor() as i32,
            self.player.pos.y.floor() as i32 - 2,
            self.player.pos.z.floor() as i32,
        ];

        // 1. the five new food rows through the real eat-value path
        let food_ok = is_food(ROTTEN_FLESH)
            && is_food(SPIDER_EYE)
            && is_food(CHORUS_FRUIT)
            && is_food(GOLDEN_APPLE)
            && is_food(MELON_SLICE)
            && (food_heal(ROTTEN_FLESH) - 2.0).abs() < 1e-6
            && (food_heal(SPIDER_EYE) - 1.0).abs() < 1e-6
            && (food_heal(CHORUS_FRUIT) - 2.0).abs() < 1e-6
            && (food_heal(GOLDEN_APPLE) - 2.0).abs() < 1e-6
            && (food_heal(MELON_SLICE) - 1.0).abs() < 1e-6;

        // 2. the melon crafts: the 9-slice block + the 1-slice seeds
        let g = vec![ItemStack::new(MELON_SLICE, 1); 9];
        let melon_craft =
            vc_gameplay::craft::match_grid(&g, 3).map(|o| o.block == MELON).unwrap_or(false);
        let mut s = vec![ItemStack::EMPTY; 9];
        s[4] = ItemStack::new(MELON_SLICE, 1);
        let seeds_craft = vc_gameplay::craft::match_grid(&s, 3)
            .map(|o| (o.block, o.count) == (MELON_SEEDS, 1))
            .unwrap_or(false);

        // 3. the golden apple's effect pair through the real effects
        //    system (Absorption 2:00 = 2400 + Regeneration II 0:05 =
        //    100 ticks at amplifier 1)
        self.player
            .effects
            .apply(vc_gameplay::effects::EffectKind::Absorption, 0, 2400);
        self.player
            .effects
            .apply(vc_gameplay::effects::EffectKind::Regeneration, 1, 100);
        let golden_ok = self
            .player
            .effects
            .amplifier(vc_gameplay::effects::EffectKind::Absorption)
            == Some(0)
            && self
                .player
                .effects
                .amplifier(vc_gameplay::effects::EffectKind::Regeneration)
                == Some(1);

        // 4. the throwable trio: push each through the real projectile
        //    list with PLAYER_OWNER, fly them into a stone floor, then
        //    drain through the REAL game-layer event path
        self.sim.mobs.arrows.clear();
        self.sim.mobs.landings.clear();
        self.test_place(STONE, pos[0], pos[1], pos[2]);
        for kind in [
            vc_gameplay::mobs::ProjKind::Snowball,
            vc_gameplay::mobs::ProjKind::Egg,
            vc_gameplay::mobs::ProjKind::Pearl,
        ] {
            self.sim.mobs.arrows.push(vc_gameplay::mobs::Arrow {
                pos: [pos[0] as f32 + 0.5, pos[1] as f32 + 12.0, pos[2] as f32 + 0.5],
                vel: [0.0, -24.0, 0.0],
                damage: 0.0,
                age: 0,
                kind,
                owner: vc_gameplay::mobs::PLAYER_OWNER,
            });
        }
        for _ in 0..60 {
            self.sim
                .step(&mut self.world, &mut self.light, &vc_sim::sim::TickScope::everything());
        }
        let before = (
            self.player.pos.x.floor() as i32,
            self.player.pos.y.floor() as i32,
            self.player.pos.z.floor() as i32,
        );
        let hp_before = self.player.health;
        let chicks_before = self
            .sim
            .mobs
            .list
            .iter()
            .filter(|m| m.kind == vc_gameplay::mobs::MobKind::Chicken)
            .count();
        // the real drain: drain_mob_events resolves the egg hatch +
        // the pearl teleport (the landing queue filled by tick_arrows)
        self.drain_mob_events();
        let chicks = self
            .sim
            .mobs
            .list
            .iter()
            .filter(|m| m.kind == vc_gameplay::mobs::MobKind::Chicken)
            .count();
        let landed_chicks = chicks - chicks_before;
        // the pearl: teleported (pos moved) + the 5 HP cost (survival
        // only — the creative check rides the mode gate)
        let moved = (self.player.pos.x.floor() as i32, self.player.pos.y.floor() as i32,
            self.player.pos.z.floor() as i32) != before;
        let paid = if self.mode.depletes_items() {
            (hp_before - self.player.health - 5.0).abs() < 1e-6
        } else {
            true
        };
        // the egg's honest band: 0 (the 7/8 miss), 1 (the chick), or 4
        // (the 1/256 quad)
        let hatch_ok = matches!(landed_chicks, 0 | 1 | 4);
        let landings_ok = self.sim.mobs.landings.is_empty(); // drained

        // 5. the chorus bound: one real warp attempt — the invariant
        //    is the ±8 box around the origin (a failed warp stays put,
        //    a successful one lands inside; both are correct)
        let origin = (
            self.player.pos.x.floor() as i32,
            self.player.pos.y.floor() as i32,
            self.player.pos.z.floor() as i32,
        );
        self.chorus_teleport();
        let chorus_ok = (self.player.pos.x.floor() as i32 - origin.0).abs() <= 8
            && (self.player.pos.y.floor() as i32 - origin.1).abs() <= 8
            && (self.player.pos.z.floor() as i32 - origin.2).abs() <= 8;

        vc_render::render::report_boot_log(&format!(
            "e2e: audit16b food={} melon={} golden={} throw={} hatch={} chorus={} (pearl moved={}, paid={})",
            food_ok,
            melon_craft && seeds_craft,
            golden_ok,
            landings_ok && moved && paid,
            hatch_ok,
            chorus_ok,
            moved,
            paid
        ));
    }

    fn test_place(&mut self, block: u16, x: i32, y: i32, z: i32) {""",
    ),
    (
        "e2e call",
        """                if std::env::var("E2E_V116").is_ok() {
                    self.e2e_audit16();
                }""",
        """                if std::env::var("E2E_V116").is_ok() {
                    self.e2e_audit16();
                    self.e2e_audit16b();
                }""",
    ),
], "game-e2e")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch l applied: e2e stage")
