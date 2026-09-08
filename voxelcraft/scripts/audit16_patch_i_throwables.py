#!/usr/bin/env python3
"""Sweep-2 patch I — the throwable family + the food effects + the melon
break, all live-verified 2026-09-09 (captures: Ender_Pearl, Snowball,
Egg, Chorus_Fruit, Golden_Apple, Rotten_Flesh, Spider_Eye, Melon_Slice).

- ROTTEN_FLESH 2.0 HP + Hunger (0:30) at the 80% roll
- SPIDER_EYE edible 1.0 HP + Poison (0:05)
- GOLDEN_APPLE edible 2.0 HP + Absorption (2:00) + Regeneration II (0:05)
- CHORUS_FRUIT edible 2.0 HP + the 16-attempt +-8-axes teleport
- the throwable trio: snowball / egg / ender pearl, PLAYER_OWNER-marked,
  with the landing queue (egg: 1/8 chick + 1/32 three more; pearl:
  teleport + 5 HP)
- melon blocks drop 3-7 slices
"""
import sys

fail = []


def patch(path, edits, tag):
    src = open(path).read()
    for what, old, new in edits:
        if new in src:
            continue  # already applied
        if old not in src or src.count(old) != 1:
            fail.append(f"[{tag}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)


# ======================================================================
# 1) mobs.rs — ProjKind::Egg/Pearl + PLAYER_OWNER + the landing queue
# ======================================================================
patch("crates/vc-gameplay/src/mobs.rs", [
    (
        "ProjKind variants",
        """    /// 1.13: the trident — 8 HP base (VERIFIED w/Trident: "Projectile
    /// damage 8 HP"; the drowned throw "sends it up to 20 blocks away"
    /// at a 1.5 s cadence — VERIFIED w/Drowned §Attacking). The
    /// player-thrown form applies Impaling bonuses at the game layer.
    Trident,
}""",
        """    /// 1.13: the trident — 8 HP base (VERIFIED w/Trident: "Projectile
    /// damage 8 HP"; the drowned throw "sends it up to 20 blocks away"
    /// at a 1.5 s cadence — VERIFIED w/Drowned §Attacking). The
    /// player-thrown form applies Impaling bonuses at the game layer.
    Trident,
    /// the sweep-2 player throwables (the 1.0-era class): the egg —
    /// "an egg has a 1/8 (12.5%) chance of spawning a chick. If this
    /// occurs, there is a 1/32 (3.125%) chance of spawning three
    /// additional chicks" (VERIFIED w/Egg, live 2026-09-09)
    Egg,
    /// the ender pearl — "consumes the item and teleports the player to
    /// where the pearl lands, dealing 5 HP damage" + "a cooldown of one
    /// second (20 ticks)" (VERIFIED w/Ender_Pearl, live 2026-09-09)
    Pearl,
}

/// the sweep-2 throwables: the owner marker for player-thrown
/// projectiles — never matches a mob id, and tick_arrows skips the
/// player-hit sphere for it (a thrower can't be hit by their own
/// projectile).
pub const PLAYER_OWNER: u32 = u32::MAX;""",
    ),
    (
        "landings field",
        """    pub target_hits: Vec<([i32; 3], u8, i32)>,""",
        """    pub target_hits: Vec<([i32; 3], u8, i32)>,
    /// the sweep-2 landing queue: player-thrown eggs (hatch the chick
    /// roll at the landing) and pearls (teleport the thrower) — drained
    /// by the game layer. Filled by tick_arrows' block-collision arm
    /// and the mob-hit arm.
    pub landings: Vec<(ProjKind, [f32; 3])>,""",
    ),
    (
        "landings init",
        """            target_hits: Vec::new(),""",
        """            target_hits: Vec::new(),
            landings: Vec::new(),""",
    ),
    (
        "tick_arrows signature",
        """            &mut mobs,
            &mut pending,
            &mut target_hits,
        );
        self.list = mobs;""",
        """            &mut mobs,
            &mut pending,
            &mut target_hits,
            &mut landings,
        );
        self.list = mobs;""",
    ),
    (
        "tick_arrows take landings",
        """        let mut target_hits = std::mem::take(&mut self.target_hits);
        tick_arrows(""",
        """        let mut target_hits = std::mem::take(&mut self.target_hits);
        let mut landings = std::mem::take(&mut self.landings);
        tick_arrows(""",
    ),
    (
        "tick_arrows restore landings",
        """        self.target_hits = target_hits;
    }""",
        """        self.target_hits = target_hits;
        self.landings = landings;
    }""",
    ),
    (
        "test call site",
        """            tick_arrows(&mut sys.arrows, sys.player, false, &mut sys.hits, &world, &mut [], &mut Vec::new(), &mut Vec::new());""",
        """            tick_arrows(&mut sys.arrows, sys.player, false, &mut sys.hits, &world, &mut [], &mut Vec::new(), &mut Vec::new(), &mut Vec::new());""",
    ),
    (
        "tick_arrows params",
        """    mobs: &mut [Mob],
    pending: &mut Vec<(u32, f32)>,
    target_hits: &mut Vec<([i32; 3], u8, i32)>,
) {""",
        """    mobs: &mut [Mob],
    pending: &mut Vec<(u32, f32)>,
    target_hits: &mut Vec<([i32; 3], u8, i32)>,
    landings: &mut Vec<(ProjKind, [f32; 3])>,
) {""",
    ),
    (
        "player-hit skip for player-owned",
        """        // player body-center hit sphere (r = 0.8)
        if let Some(p) = player {
            if !invuln {""",
        """        // player body-center hit sphere (r = 0.8). The sweep-2
        // throwables carry PLAYER_OWNER — the thrower is never hit by
        // their own projectile (the snowball/egg/pearl class).
        if let Some(p) = player {
            if !invuln && a.owner != PLAYER_OWNER {""",
    ),
    (
        "src match arms",
        """                        // 1.13: the drowned's thrown trident (8 HP base —
                        // VERIFIED w/Trident "Projectile damage 8 HP")
                        ProjKind::Trident => MobKind::Drowned,
                    };""",
        """                        // 1.13: the drowned's thrown trident (8 HP base —
                        // VERIFIED w/Trident "Projectile damage 8 HP")
                        ProjKind::Trident => MobKind::Drowned,
                        // the sweep-2 throwables (attribution only —
                        // never resolved: PLAYER_OWNER skips the sphere)
                        ProjKind::Egg => MobKind::Chicken,
                        ProjKind::Pearl => MobKind::Enderman,
                    };""",
    ),
    (
        "snowball branch head",
        """        // Phase E1: snowball mob hits — 3 damage to blazes, 0 + knockback
        // to everything else (VERIFIED w/Snow_Golem: "Thrown snowballs do
        // not deal damage except to blazes, but they still knock back any
        // mobs that they hit")
        if a.kind == ProjKind::Snowball {""",
        """        // Phase E1: snowball mob hits — 3 damage to blazes, 0 + knockback
        // to everything else (VERIFIED w/Snow_Golem: "Thrown snowballs do
        // not deal damage except to blazes, but they still knock back any
        // mobs that they hit"). The sweep-2 throwables join the same
        // class: eggs and pearls knock mobs back (0 damage — the thrown
        // class's rule) and push a LANDING event at the hit (the pearl
        // teleports the thrower to the struck mob, the egg rolls the
        // hatch there).
        if matches!(a.kind, ProjKind::Snowball | ProjKind::Egg | ProjKind::Pearl) {""",
    ),
    (
        "mob-hit landing push",
        """                    if m.kind == MobKind::Blaze {
                        pending.push((m.id, 3.0)); // VERIFIED: 3 HP vs blazes
                    } else {
                        // knockback only
                        m.vel[0] += a.vel[0] * 0.05;
                        m.vel[2] += a.vel[2] * 0.05;
                    }
                    hit_mob = true;
                    break;""",
        """                    if m.kind == MobKind::Blaze && a.kind == ProjKind::Snowball {
                        pending.push((m.id, 3.0)); // VERIFIED: 3 HP vs blazes
                    } else {
                        // knockback only
                        m.vel[0] += a.vel[0] * 0.05;
                        m.vel[2] += a.vel[2] * 0.05;
                    }
                    if matches!(a.kind, ProjKind::Egg | ProjKind::Pearl) {
                        landings.push((a.kind, m.pos));
                    }
                    hit_mob = true;
                    break;""",
    ),
    (
        "ground-hit landing push",
        """        if is_solid(world.get_block(
            a.pos[0].floor() as i32,
            a.pos[1].floor() as i32,
            a.pos[2].floor() as i32,
        )) || a.age > 20 * 60
        {""",
        """        let hit_solid = is_solid(world.get_block(
            a.pos[0].floor() as i32,
            a.pos[1].floor() as i32,
            a.pos[2].floor() as i32,
        ));
        if hit_solid || a.age > 20 * 60
        {""",
    ),
    (
        "landing push before remove",
        """                target_hits.push(([bx, by, bz], power, ticks));
            }
            arrows.remove(i);
            continue;
        }""",
        """                target_hits.push(([bx, by, bz], power, ticks));
            }
            // the sweep-2 landing queue: only real block collisions
            // (the 60-second age-out is a lost projectile — the void
            // case, no hatch, no teleport)
            if hit_solid && matches!(a.kind, ProjKind::Egg | ProjKind::Pearl) {
                landings.push((a.kind, a.pos));
            }
            arrows.remove(i);
            continue;
        }""",
    ),
    (
        "take_landings drain",
        """/// 1.16: drain the projectile-on-target hit queue (the game layer
/// turns each hit into the blockstate power write + its decay timer).
pub fn take_target_hits(sys: &mut MobSystem) -> Vec<([i32; 3], u8, i32)> {
    std::mem::take(&mut sys.target_hits)
}""",
        """/// 1.16: drain the projectile-on-target hit queue (the game layer
/// turns each hit into the blockstate power write + its decay timer).
pub fn take_target_hits(sys: &mut MobSystem) -> Vec<([i32; 3], u8, i32)> {
    std::mem::take(&mut sys.target_hits)
}

/// the sweep-2: drain the player-throwable landing queue (the game
/// layer hatches the eggs and resolves the pearl teleports).
pub fn take_landings(sys: &mut MobSystem) -> Vec<(ProjKind, [f32; 3])> {
    std::mem::take(&mut sys.landings)
}""",
    ),
    (
        "chick maturity (fox branch)",
        """    if m.kind == MobKind::Fox {
        if m.variant & 0x40 != 0 && m.aux > 0 {""",
        """    if matches!(m.kind, MobKind::Fox | MobKind::Chicken) {
        // the chicken row joined at the sweep-2 (the egg-spawned chick
        // — "an egg has a 1/8 chance of spawning a chick", VERIFIED
        // w/Egg; chicks mature on the same 24000-tick countdown, the
        // scute-less fox form)
        if m.variant & 0x40 != 0 && m.aux > 0 {""",
    ),
], "mobs")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch i part 1 applied: mobs.rs throwables")
