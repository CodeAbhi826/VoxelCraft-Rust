#!/usr/bin/env python3
"""Test patch 4 — mobs behavioral tests + the audit16 e2e stage + CI greps."""
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

# ---------------- mobs.rs: behavioral tests ----------------
patch("crates/vc-gameplay/src/mobs.rs", [
    (
        "egg laying test",
        """    /// the sting contract: an angry bee stings ONCE (2 HP + Poison I
    /// 10 s payload), loses the stinger, "dies approximately one
    /// minute later" (1200 ticks), and never attacks again (VERIFIED
    /// w/Bee §Attacking)
    #[test]
    fn v115_bee_sting_rules() {""",
        """    /// the completeness audit: the chicken's egg laying — "Every adult
    /// chicken lays an egg item every 5-10 minutes ... The theoretical
    /// average would be expected at 1 egg every 7.5 minutes (9000 game
    /// ticks)" (VERIFIED w/Egg). Statistical form: 300k ticks of one
    /// adult chicken -> ~33 eggs expected; assert a generous Poisson
    /// band (the per-tick 1/9000 roll reproduces the steady state).
    #[test]
    fn audit16_chicken_lays_eggs_at_the_9000_tick_average() {
        let world = v115_world();
        let mut sys = MobSystem::new(9);
        sys.spawn_at(MobKind::Chicken, 8, 66, 8).unwrap();
        let mut eggs = 0;
        for _ in 0..300_000 {
            sys.tick(&world, (0, 0), i32::MAX);
            eggs += sys.pending_drops.iter().filter(|(_, b)| *b == EGG).count();
            sys.pending_drops.clear();
        }
        // expected 33.3; band 10..=70 covers ~±4 sigma (Poisson(33))
        assert!(eggs >= 10 && eggs <= 70, "expected ~33 eggs, got {eggs}");
    }

    /// the completeness audit: the cave spider's venom — the melee hit
    /// carries "Poison for 7 seconds" on Normal (140 ticks, VERIFIED
    /// w/Cave_Spider's venom row)
    #[test]
    fn audit16_cave_spider_venom_payload() {
        let world = v115_world();
        let mut sys = MobSystem::new(9);
        sys.player = Some([8.6, 66.5, 8.5]);
        sys.spawn_at(MobKind::CaveSpider, 8, 66, 8).unwrap();
        let mut bitten = false;
        for _ in 0..80 {
            sys.tick(&world, (0, 0), i32::MAX);
            let hits = std::mem::take(&mut sys.hits);
            if let Some(h) = hits.first() {
                assert_eq!(h.source, MobKind::CaveSpider);
                assert!((h.damage - 2.0).abs() < 1e-4, "Normal melee 2 (VERIFIED)");
                assert_eq!(h.poison_effect, Some(140), "Poison 7 s = 140 ticks");
                assert_eq!(h.wither_effect, None);
                bitten = true;
                break;
            }
        }
        assert!(bitten, "the cave spider reached + bit the player");
    }

    /// the completeness audit: the ghast fires its fireball — "a ghast
    /// faces the player and shoots a fireball every 3 seconds" within
    /// the 64-block range (VERIFIED w/Ghast §Behavior); the impact
    /// damage is the Normal 6 row.
    #[test]
    fn audit16_ghast_fires_the_3_second_fireball() {
        let world = v115_world();
        let mut sys = MobSystem::new(9);
        sys.player = Some([8.5, 70.5, 8.5]);
        // 20 blocks out — inside the 64-block target range
        sys.spawn_at(MobKind::Ghast, 28, 70, 8).unwrap();
        let mut fired = false;
        for _ in 0..200 {
            sys.tick(&world, (0, 0), i32::MAX);
            if let Some(a) = sys.arrows.first() {
                assert_eq!(a.kind, ProjKind::Fireball, "the ghast's projectile");
                assert!((a.damage - 6.0).abs() < 1e-4, "impact Normal 6 (VERIFIED)");
                fired = true;
                break;
            }
        }
        assert!(fired, "the ghast shot within 200 ticks (60-tick cadence)");
        // and the flying class: no gravity-driven fall distance
        let g = sys.by_id(sys.arrows.first().map(|_| 0).unwrap_or(0));
        let _ = g; // (the arrows themselves fly; the mob-class check is below)
        let gh = sys
            .snapshot()
            .iter()
            .find(|(_, k, _, _)| *k == MobKind::Ghast);
        let _ = gh;
        assert!(MobKind::Ghast.flies(), "the ghast is a FlyingMob-class");
    }

    /// the sting contract: an angry bee stings ONCE (2 HP + Poison I
    /// 10 s payload), loses the stinger, "dies approximately one
    /// minute later" (1200 ticks), and never attacks again (VERIFIED
    /// w/Bee §Attacking)
    #[test]
    fn v115_bee_sting_rules() {""",
    ),
])

print("FAILED:" if fail else "mobs behavioral tests applied")
for f in fail:
    print("  -", f)
sys.exit(1 if fail else 0)
