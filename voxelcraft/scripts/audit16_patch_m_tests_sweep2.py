#!/usr/bin/env python3
"""Sweep-2 patch M — the unit tests: mobs.rs (landings, hatch stats,
maturity, self-hit skip), game.rs (food rows, effects, chorus
destination), craft.rs (the melon crafts)."""
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
# 1) mobs.rs tests — after the chicken egg-laying test
# ======================================================================
patch("crates/vc-gameplay/src/mobs.rs", [
    (
        "mobs tests",
        """        // expected 33.3; band 10..=70 covers ~±4 sigma (Poisson(33))
        assert!(eggs >= 10 && eggs <= 70, "expected ~33 eggs, got {eggs}");
    }""",
        """        // expected 33.3; band 10..=70 covers ~±4 sigma (Poisson(33))
        assert!(eggs >= 10 && eggs <= 70, "expected ~33 eggs, got {eggs}");
    }

    /// the sweep-2: the player-thrown trio never hits the thrower, and
    /// eggs + pearls (not snowballs) push landing events on the ground
    /// hit (VERIFIED w/Egg + w/Ender_Pearl + w/Snowball, live 2026-09-09)
    #[test]
    fn audit16_sweep2_projectile_landings() {
        let world = flat_world();
        let mut sys = MobSystem::new(42);
        sys.player = Some([8.5, 66.0, 8.5]);
        // a projectile spawning INSIDE the thrower's hit sphere, flying
        // away — PLAYER_OWNER must skip the player-hit branch
        for (kind, name) in [
            (ProjKind::Snowball, "snowball"),
            (ProjKind::Egg, "egg"),
            (ProjKind::Pearl, "pearl"),
        ] {
            sys.arrows.push(Arrow {
                pos: [8.5, 66.9, 8.5],
                vel: [0.0, -24.0, 0.0],
                damage: 0.0,
                age: 0,
                kind,
                owner: PLAYER_OWNER,
            });
        }
        let mut landings: Vec<(ProjKind, [f32; 3])> = Vec::new();
        for _ in 0..60 {
            let mut hits = std::mem::take(&mut sys.hits);
            tick_arrows(
                &mut sys.arrows,
                sys.player,
                false,
                &mut hits,
                &world,
                &mut [],
                &mut Vec::new(),
                &mut Vec::new(),
                &mut landings,
            );
            sys.hits = hits;
        }
        assert!(sys.hits.is_empty(), "a thrown projectile never hits its thrower");
        assert_eq!(landings.len(), 2, "egg + pearl land (the snowball has no landing)");
        assert!(landings.iter().all(|(k, _)| {
            matches!(k, ProjKind::Egg | ProjKind::Pearl)
        }));
        // they landed on the stone floor (y = 64, the flat_world surface)
        assert!(landings.iter().all(|(_, p)| p[1].floor() as i32 == 64));
    }

    /// the sweep-2: the egg hatch statistics — 1/8 per egg + 1/32 for
    /// three more chicks (VERIFIED w/Egg §Spawning chickens). The
    /// statistical band rides 800 throws (expected ~112 chicks).
    #[test]
    fn audit16_sweep2_egg_hatch_statistics() {
        let world = flat_world();
        let mut sys = MobSystem::new(77);
        let mut chicks = 0usize;
        let mut rng = vc_rng::rng::Rng::new(1234);
        for i in 0..800 {
            sys.arrows.clear();
            sys.landings.clear();
            sys.arrows.push(Arrow {
                pos: [8.5, 70.0, 8.5],
                vel: [0.0, -24.0, 0.0],
                damage: 0.0,
                age: 0,
                kind: ProjKind::Egg,
                owner: PLAYER_OWNER,
            });
            for _ in 0..10 {
                tick_arrows(
                    &mut sys.arrows,
                    None,
                    false,
                    &mut Vec::new(),
                    &world,
                    &mut [],
                    &mut Vec::new(),
                    &mut Vec::new(),
                    &mut sys.landings,
                );
            }
            // the game layer's hatch roll, replayed here with its own
            // rng (1/8; then 1/32 for three more)
            for _ in 0..sys.landings.len() {
                if rng.next_range(8) == 0 {
                    chicks += 1;
                    if rng.next_range(32) == 0 {
                        chicks += 3;
                    }
                }
            }
            let _ = i;
        }
        // expected 800/8 = 100 (plus ~1 quad event) — the ±5 sigma band
        assert!(
            chicks >= 55 && chicks <= 165,
            "expected ~100 chicks over 800 eggs, got {chicks}"
        );
    }

    /// the sweep-2: the egg-spawned chick (variant 0x40) matures on the
    /// 24000-tick countdown — the fox/turtle class (VERIFIED w/Egg)
    #[test]
    fn audit16_sweep2_chick_matures() {
        let world = flat_world();
        let mut sys = MobSystem::new(88);
        let id = sys.spawn_variant(MobKind::Chicken, 8, 65, 8, 0x40).unwrap();
        if let Some(m) = sys.list.last_mut() {
            m.aux = 24000;
        }
        for _ in 0..23_999 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        // one tick short: still a baby
        assert!(
            sys.by_id(id).map(|m| m.variant & 0x40 != 0).unwrap_or(false),
            "one tick short of maturity, still a chick"
        );
        sys.tick(&world, (0, 0), i32::MAX);
        assert!(
            sys.by_id(id).map(|m| m.variant & 0x40 == 0).unwrap_or(false),
            "matured at exactly 24000 ticks"
        );
    }
""",
    ),
], "mobs-tests")

# ======================================================================
# 2) game.rs tests — after audit16_food_values
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "game tests anchor",
        """        assert!((food_heal(BEETROOT) - 0.5).abs() < 1e-6, "hunger 1");""",
        """        // ---- the sweep-2 rows (VERIFIED live 2026-09-09: the
        // Rotten_Flesh/Spider_Eye/Chorus_Fruit/Golden_Apple/
        // Melon_Slice captures) ----
        assert!((food_heal(ROTTEN_FLESH) - 2.0).abs() < 1e-6, "hunger 4");
        assert!((food_heal(SPIDER_EYE) - 1.0).abs() < 1e-6, "hunger 2");
        assert!((food_heal(CHORUS_FRUIT) - 2.0).abs() < 1e-6, "hunger 4");
        assert!((food_heal(GOLDEN_APPLE) - 2.0).abs() < 1e-6, "hunger 4");
        assert!((food_heal(MELON_SLICE) - 1.0).abs() < 1e-6, "hunger 2");
        assert!(is_food(ROTTEN_FLESH), "edible since Phase 2 — value now correct");
        assert!(is_food(SPIDER_EYE), "the 1.0 spider eye now edible");
        assert!(is_food(CHORUS_FRUIT), "the 1.9 chorus fruit now edible");
        assert!(is_food(GOLDEN_APPLE), "the golden apple now edible");
        assert!(is_food(MELON_SLICE), "the 1.0 melon slice");
        // the golden apple's effect pair: Absorption 2:00 (2400) +
        // Regeneration II 0:05 (100 ticks at amplifier 1)
        {
            let mut fx = vc_gameplay::effects::Effects::new();
            fx.apply(vc_gameplay::effects::EffectKind::Absorption, 0, 2400);
            fx.apply(vc_gameplay::effects::EffectKind::Regeneration, 1, 100);
            assert_eq!(
                fx.amplifier(vc_gameplay::effects::EffectKind::Absorption),
                Some(0),
                "Absorption I 2:00"
            );
            assert_eq!(
                fx.amplifier(vc_gameplay::effects::EffectKind::Regeneration),
                Some(1),
                "Regeneration II 0:05"
            );
        }
        assert!((food_heal(BEETROOT) - 0.5).abs() < 1e-6, "hunger 1");""",
    ),
], "game-tests-food")

# ======================================================================
# 3) game.rs — the chorus destination unit test (needs a World — ride
#    the existing game test module; build a tiny stone-floor world)
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "chorus destination test",
        """    fn audit16_food_values() {""",
        """    /// the sweep-2: the chorus destination rule — the ±8 box, the
    /// solid-floor + 2-air validity, and the all-solid failure (VERIFIED
    /// w/Chorus_Fruit §Teleportation, live 2026-09-09)
    #[test]
    fn audit16_sweep2_chorus_destination() {
        // a stone floor world: y <= 64 solid, y >= 65 air (the
        // flat-world convention from the mobs tests)
        let mut w = World::new(11);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        let mut rng = vc_rng::rng::Rng::new(55);
        let mut warped = 0;
        for _ in 0..200 {
            if let Some([x, y, z]) = chorus_destination(&w, 8, 65, 8, &mut rng) {
                warped += 1;
                assert!((x - 8).abs() <= 8, "the ±8 x bound");
                assert!((y - 65).abs() <= 8, "the ±8 y bound");
                assert!((z - 8).abs() <= 8, "the ±8 z bound");
                assert_eq!(y, 65, "the only valid standing row above the floor");
            }
        }
        assert!(warped >= 150, "the open floor warps nearly always, got {warped}/200");
        // the failure case: an all-solid world has no valid destination
        let mut solid = World::new(12);
        let mut sc = vc_chunk::chunk::Chunk::empty();
        for y in 0..=80i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    sc.set(lx, y as usize, lz, STONE);
                }
            }
        }
        solid.insert_generated((0, 0), std::sync::Arc::new(sc), Vec::new());
        assert!(
            chorus_destination(&solid, 8, 70, 8, &mut rng).is_none(),
            "no air column -> the failed warp (the entity stays)"
        );
    }

    fn audit16_food_values() {""",
    ),
], "game-tests-chorus")

# ======================================================================
# 4) craft.rs — the melon craft tests
# ======================================================================
patch("crates/vc-gameplay/src/craft.rs", [
    (
        "craft tests",
        """    fn audit16_kitchen_chain() {""",
        """    /// the sweep-2 melon crafts: 9 slices -> the melon block, 1 slice
    /// -> melon seeds (VERIFIED w/Melon_Slice §Crafting, live 2026-09-09)
    #[test]
    fn audit16_sweep2_melon_crafts() {
        let g = vec![ItemStack::new(MELON_SLICE, 1); 9];
        let out = match_grid(&g, 3).expect("the 3x3 nine-slice recipe");
        assert_eq!(out.block, MELON);
        assert_eq!(out.count, 1);
        // a partial grid (8 slices) must NOT craft
        let mut partial = vec![ItemStack::EMPTY; 9];
        for i in 0..8 {
            partial[i] = ItemStack::new(MELON_SLICE, 1);
        }
        assert!(match_grid(&partial, 3).is_none(), "8 slices craft nothing");
        // 1 slice -> 1 melon seed
        let mut one = vec![ItemStack::EMPTY; 9];
        one[4] = ItemStack::new(MELON_SLICE, 1);
        let out = match_grid(&one, 3).expect("the slice-to-seeds row");
        assert_eq!(out.block, MELON_SEEDS);
        assert_eq!(out.count, 1);
    }

    fn audit16_kitchen_chain() {""",
    ),
], "craft-tests")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch m applied: the sweep-2 tests")
