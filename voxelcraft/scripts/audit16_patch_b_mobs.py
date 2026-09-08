#!/usr/bin/env python3
"""mobs.rs patch — the completeness-audit mobs (ghast/cave spider/silverfish)
+ chicken egg-laying + the exact 1.16 nether-wastes spawn weights."""
import sys

PATH = "crates/vc-gameplay/src/mobs.rs"
src = open(PATH).read()
fail = []

def sub_once(old, new, what):
    global src
    if old not in src or src.count(old) != 1:
        fail.append(f"ANCHOR ({what}) not found/unique: {old[:80]!r}")
        return
    src = src.replace(old, new)

# 1) MobKind enum — 3 new kinds after Hoglin
sub_once(
    """    Hoglin,
    /// The squid — a PRE-1.13 legacy marker""",
    """    Hoglin,
    // ---- the 1.0-1.16.5 completeness audit (2026-09-08): the three
    // classic mobs no earlier bracket ever accounted for ----
    /// The ghast — the floating Nether artillery. VERIFIED
    /// (minecraft.wiki/w/Ghast, live 2026-09-08, capture
    /// scripts/audit16_page_Ghast.json): 10 HP hostile, hitbox
    /// 4.0×4.0 ("They have a hitbox of 4×4×4 blocks"), speed 0.7,
    /// fireball impact "Normal: 6 HP", "target players within 64
    /// blocks horizontally and 4 blocks vertically", "shoots a
    /// fireball every 3 seconds", "Ghasts do not attempt to approach
    /// the player once aggravated, but instead fire at the player
    /// from their position". Spawns in Nether Wastes / Soul Sand
    /// Valley / Basalt Deltas. Drops: ghast tear 0–1 at 50%, gunpowder
    /// 0–2 at 66.67% ("are the only source of ghast tears").
    Ghast,
    /// The cave spider — the mineshaft spawner's own mob. VERIFIED
    /// (w/Cave_Spider, live, capture audit16_page_Cave_Spider.json):
    /// 12 HP, "Melee: Easy: 2 HP Normal: 2 HP Hard: 3 HP", venom
    /// "Normal: Poison for 7 seconds" (Easy gets none, Hard 15 s —
    /// the engine's single-difficulty row is Normal's 7 s = 140
    /// ticks), "Hitbox size Height: 0.5 blocks Width: 0.7 blocks",
    /// "Speed 0.3", "Mineshaft: from monster spawners". Drops: string
    /// 0–2 at 66.67% + spider eye 0–1 at 33.33% (the spider rows).
    CaveSpider,
    /// The silverfish — the stronghold's infestant. VERIFIED
    /// (w/Silverfish, live, capture audit16_page_Silverfish.json):
    /// 8 HP hostile, "Attack strength Easy and Normal: 1 HP Hard:
    /// 1.5 HP", "Hitbox size Height: 0.3 Blocks Width: 0.4 Blocks",
    /// "Speed 0.25", "Stronghold: from infested blocks and monster
    /// spawners" (the engine's spawner form; the infested-block
    /// spread is the trimmed half, disclosed), "Silverfish have no
    /// drops other than 5 XP".
    Silverfish,
    /// The squid — a PRE-1.13 legacy marker""",
    "MobKind variants",
)

# 2) from_name arms
sub_once(
    """            "hoglin" => MobKind::Hoglin,
            _ => return None,""",
    """            "hoglin" => MobKind::Hoglin,
            // the completeness audit's classic trio
            "ghast" => MobKind::Ghast,
            "cave_spider" => MobKind::CaveSpider,
            "silverfish" => MobKind::Silverfish,
            _ => return None,""",
    "from_name",
)

# 3) name() arms
sub_once(
    '            MobKind::Hoglin => "minecraft:hoglin",',
    '''            MobKind::Hoglin => "minecraft:hoglin",
            MobKind::Ghast => "minecraft:ghast",
            MobKind::CaveSpider => "minecraft:cave_spider",
            MobKind::Silverfish => "minecraft:silverfish",''',
    "name()",
)

# 4) sprite_tile arms
sub_once(
    """            MobKind::Hoglin => TILE_MOB_HOGLIN,
            // classification-only marker""",
    """            MobKind::Hoglin => TILE_MOB_HOGLIN,
            // the completeness audit's classic trio (audit16_art)
            MobKind::Ghast => TILE_MOB_GHAST,
            MobKind::CaveSpider => TILE_MOB_CAVESPIDER,
            MobKind::Silverfish => TILE_MOB_SILVERFISH,
            // classification-only marker""",
    "sprite_tile",
)

# 5) hostile() arms
sub_once(
    """                // 1.16 (Nether Update, part 2): the hoglin — VERIFIED
                // w/Hoglin infobox "Behavior Hostile" (the piglin is
                // the neutral one: "Neutral (adult)")
                | MobKind::Hoglin
        )""",
    """                // 1.16 (Nether Update, part 2): the hoglin — VERIFIED
                // w/Hoglin infobox "Behavior Hostile" (the piglin is
                // the neutral one: "Neutral (adult)")
                | MobKind::Hoglin
                // the completeness audit's classic trio — all three
                // infoboxes read "Behavior Hostile" (the cave spider's
                // own "Neutral" row is the spider family's
                // light-conditional hostility; the engine's standing
                // spider adaptation treats the family as hostile,
                // disclosed)
                | MobKind::Ghast
                | MobKind::CaveSpider
                | MobKind::Silverfish
        )""",
    "hostile()",
)

# 6) flies() — the ghast joins the flying class
sub_once(
    """        matches!(
            self,
            MobKind::Phantom | MobKind::Vex | MobKind::Bat | MobKind::Parrot | MobKind::Bee
        )""",
    """        matches!(
            self,
            MobKind::Phantom
                | MobKind::Vex
                | MobKind::Bat
                | MobKind::Parrot
                | MobKind::Bee
                // the completeness audit: the ghast — "large, floating,
                // ghost-like" (VERIFIED w/Ghast; the bat/phantom class)
                | MobKind::Ghast
        )""",
    "flies()",
)

# 7) from_egg arms
sub_once(
    """            42 => MobKind::Strider,
            43 => MobKind::Piglin,
            44 => MobKind::Hoglin,
            _ => MobKind::Chicken,""",
    """            42 => MobKind::Strider,
            43 => MobKind::Piglin,
            44 => MobKind::Hoglin,
            // the completeness audit's classic trio — kinds 45..=47
            45 => MobKind::Ghast,
            46 => MobKind::CaveSpider,
            47 => MobKind::Silverfish,
            _ => MobKind::Chicken,""",
    "from_egg",
)

# 8) egg_id arms
sub_once(
    """            // 1.16: the V14 egg window — kinds 42..=44
            MobKind::Strider => 42,
            MobKind::Piglin => 43,
            MobKind::Hoglin => 44,""",
    """            // 1.16: the V14 egg window — kinds 42..=44
            MobKind::Strider => 42,
            MobKind::Piglin => 43,
            MobKind::Hoglin => 44,
            // the completeness audit's classic trio — kinds 45..=47
            MobKind::Ghast => 45,
            MobKind::CaveSpider => 46,
            MobKind::Silverfish => 47,""",
    "egg_id",
)

# 9) MOB_DATA — 3 new rows (after the Hoglin row, before `];`)
sub_once(
    """        kind: MobKind::Hoglin,
        health: 40.0,
        damage: 5.5,
        speed_attr: 0.3,
        armor: 0.0,
        height: 1.4,
        width: 1.3965,
        xp: 5,
    },
];""",
    """        kind: MobKind::Hoglin,
        health: 40.0,
        damage: 5.5,
        speed_attr: 0.3,
        armor: 0.0,
        height: 1.4,
        width: 1.3965,
        xp: 5,
    },
    // ---- the 1.0-1.16.5 completeness audit (all VERIFIED live
    // 2026-09-08 against the audit16 captures) ----
    MobDef {
        kind: MobKind::Ghast,
        // VERIFIED w/Ghast infobox: 10 HP; damage = the fireball's
        // Normal impact row (6); the hitbox is the 4×4×4 cube
        health: 10.0,
        damage: 6.0,
        speed_attr: 0.7,
        armor: 0.0,
        height: 4.0,
        width: 4.0,
        xp: 5,
    },
    MobDef {
        kind: MobKind::CaveSpider,
        // VERIFIED w/Cave_Spider infobox: 12 HP; Normal melee 2 (the
        // 7-second Poison rides the hit payload); 0.5×0.7 hitbox
        health: 12.0,
        damage: 2.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 0.5,
        width: 0.7,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Silverfish,
        // VERIFIED w/Silverfish infobox: 8 HP; Easy/Normal attack 1;
        // 0.3×0.4 hitbox; "no drops other than 5 XP"
        health: 8.0,
        damage: 1.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 0.3,
        width: 0.4,
        xp: 5,
    },
];""",
    "MOB_DATA rows",
)
sub_once(
    "pub const MOB_DATA: [MobDef; 45] = [",
    "pub const MOB_DATA: [MobDef; 48] = [",
    "MOB_DATA size",
)

# 10) the nether-wastes roll — the exact published weights
sub_once(
    """                    vc_world::gen::Biome::WarpedForest => MobKind::Enderman,
                    _ => match self.rng.next_range(21) {
                        0 | 1 => MobKind::MagmaCube,
                        2 => MobKind::Piglin,
                        _ => MobKind::Zombie,
                    },""",
    """                    vc_world::gen::Biome::WarpedForest => MobKind::Enderman,
                    // the completeness audit: the exact 1.16.5 wastes
                    // weights — zombified piglin 100 / ghast 50 /
                    // magma cube 40 / piglin 25 out of 215 (the
                    // zombie rides the zombified-piglin slot, the
                    // standing disclosed filler; the audit also adds
                    // the wastes' own ghast). The part-1-era 21-slot
                    // approximation (2/21 magma, 1/21 piglin) is
                    // retired.
                    _ => match self.rng.next_range(215) {
                        0..=49 => MobKind::Ghast,
                        50..=89 => MobKind::MagmaCube,
                        90..=114 => MobKind::Piglin,
                        _ => MobKind::Zombie,
                    },""",
    "nether wastes roll",
)

# 11) ai_tick: the ghast block — after the blaze block
sub_once(
    """    // ---- Phase E1: magma cube — hop movement: idle jump every 40–120
    // ticks, 13–40 with a target ≤ 16 blocks; jump height = size blocks,""",
    """    // ---- the completeness audit: GHAST — the floating Nether
    // artillery. VERIFIED (w/Ghast §Behavior, live 2026-09-08):
    // "Ghasts do not attempt to approach the player once aggravated,
    // but instead fire at the player from their position" (no chase —
    // a drift hold); "When within range, a ghast faces the player and
    // shoots a fireball every 3 seconds" (the 60-tick cadence);
    // "target players within 64 blocks horizontally". Flying (the
    // bat/phantom class — no gravity); the fireball rides the blaze's
    // ProjKind::Fireball at the 6-HP Normal impact row (the explosion
    // radius is the dragon-fireball deferral class, disclosed; the
    // redirected-fireball self-kill is trimmed with it). ----
    if m.kind == MobKind::Ghast {
        if aggro && dist < 64.0 && !invuln {
            face_player(m);
            // hold position (VERIFIED: no approach) — slow drift only
            m.vel[0] *= 0.98;
            m.vel[2] *= 0.98;
            // gentle hover band (the wiki's own "wander aimlessly"
            // vertical half — a 2 b/s ceiling-ish drift)
            m.vel[1] += (1.2 - m.vel[1]) * 0.05;
            if m.attack_cd == 0 {
                m.attack_cd = 60; // every 3 s (VERIFIED)
                spawn_projectile(m, p, rng, arrows, ProjKind::Fireball, 14.0, 6.0);
            }
        } else {
            wander_3d(rng, m, speed * 0.4);
        }
        return;
    }

    // ---- Phase E1: magma cube — hop movement: idle jump every 40–120
    // ticks, 13–40 with a target ≤ 16 blocks; jump height = size blocks,""",
    "ghast AI block",
)

# 12) the generic hostile melee arm — cave spider + silverfish join;
# the cave spider's hit carries the 7-second poison payload
sub_once(
    """        MobKind::Zombie
        | MobKind::ZombieVillager
        | MobKind::Husk
        | MobKind::Spider
        | MobKind::Enderman => {""",
    """        MobKind::Zombie
        | MobKind::ZombieVillager
        | MobKind::Husk
        | MobKind::Spider
        // the completeness audit: the two ground-classic hostiles
        // (the cave spider rides the spider's chase; the silverfish
        // the zombie's)
        | MobKind::CaveSpider
        | MobKind::Silverfish
        | MobKind::Enderman => {""",
    "generic melee arm kinds",
)

# 12b) the poison payload inside that arm's hit push — locate the
# arm's melee hit (the one right after the Zombie-family engage code)
sub_once(
    """                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage,
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                poison_effect: None,
            });""",
    """                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage,
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                // the completeness audit: the cave spider's venom —
                // "Normal: Poison for 7 seconds" (VERIFIED
                // w/Cave_Spider; 140 ticks; Easy gets none and Hard
                // 15 s — the engine's Normal row, disclosed)
                poison_effect: if m.kind == MobKind::CaveSpider {
                    Some(140)
                } else {
                    None
                },
            });""",
    "generic melee poison payload",
)

# 13) chicken egg-laying — environmental (the turtle/fox precedent)
sub_once(
    """    // ---- 1.14 (Village & Pillage): FOX life-cycle clocks —
    // environmental (player-independent, the turtle precedent):""",
    """    // ---- the completeness audit: CHICKEN egg laying —
    // environmental (the turtle/fox precedent, player-independent).
    // VERIFIED (minecraft.wiki/w/Egg, live 2026-09-08, capture
    // scripts/audit16_page_Egg.json): "Every adult chicken lays an
    // egg item every 5-10 minutes ... The theoretical average would
    // be expected at 1 egg every 7.5 minutes (9000 game ticks)".
    // Stateless engine form: a 1/9000 per-tick roll per adult —
    // the page's own steady-state figure (the 5-10-minute window is
    // the per-chicken timer's uniform bounds; the per-tick roll
    // reproduces the same mean, disclosed). ----
    if m.kind == MobKind::Chicken && m.variant == 0 && rng.next_range(9000) == 0 {
        pending_drops.push((m.pos, EGG));
    }

    // ---- 1.14 (Village & Pillage): FOX life-cycle clocks —
    // environmental (player-independent, the turtle precedent):""",
    "chicken egg laying",
)

# 14) test guards: the MOB_DATA count assertions (4 sites)
sub_once(
    '        assert_eq!(MOB_DATA.len(), 45); // + 1.11 four + 1.12 two + 1.13 eight + 1.14 fox + 1.16 three',
    '        assert_eq!(MOB_DATA.len(), 48); // + 1.11 four + 1.12 two + 1.13 eight + 1.14 fox + 1.16 three + the audit trio',
    "count guard 1",
)
sub_once(
    '        assert_eq!(MOB_DATA.len(), 45, "+ the 1.13 aquatic eight + the 1.14 fox + the 1.16 forest three");',
    '        assert_eq!(MOB_DATA.len(), 48, "+ the 1.13 aquatic eight + the 1.14 fox + the 1.16 forest three + the audit trio");',
    "count guard 2",
)
sub_once(
    '        assert_eq!(MOB_DATA.len(), 45, "32 prior + 8 aquatic + the 1.14 fox + the 1.16 forest three");',
    '        assert_eq!(MOB_DATA.len(), 48, "32 prior + 8 aquatic + the 1.14 fox + the 1.16 forest three + the audit trio");',
    "count guard 3",
)
sub_once(
    '        assert_eq!(MOB_DATA.len(), 45, "32 + 8 aquatic + the fox + the 1.16 forest three");',
    '        assert_eq!(MOB_DATA.len(), 48, "32 + 8 aquatic + the fox + the 1.16 forest three + the audit trio");',
    "count guard 4",
)

if fail:
    print("FAILED ANCHORS:")
    for f in fail:
        print("  -", f)
    sys.exit(1)

open(PATH, "w").write(src)
print(f"mobs.rs patched OK ({len(src)} chars)")
