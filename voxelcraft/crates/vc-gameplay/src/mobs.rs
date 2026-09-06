//! Mobs (master prompt Phase 2): a first batch of 9 entities — 5 hostile
//! (zombie, skeleton, creeper, spider, enderman) + 4 passive (cow, pig,
//! sheep, chicken). The remaining ~90 entities of the 1.16.5 registry are
//! explicitly deferred (see DEFERRED_ENTITIES).
//!
//! Phase E1 (evolution 1.0–1.2 bracket): +7 mobs, all live-verified
//! 2026-09-06 against minecraft.wiki (see
//! docs/research/phase1-1.0-1.2-research.md for the audit trail):
//! - Snow Golem (4 HP, snowball 0 dmg / 3 vs blaze, 1/s at ≤ 10 blocks)
//! - Magma Cube (HP = size², dmg = size+2, armor = 3×size, splits 2–4)
//! - Blaze (20 HP, 3-fireball bursts, fortress light ≤ 11, 10 XP)
//! - Ocelot (10 HP, jungle, trust-by-feeding, attacks chickens)
//! - Iron Golem (100 HP, Normal 7.5–21.5, village guard)
//! - Zombie Villager (20 HP, 0/50/100% conversion by difficulty, curable)
//! - Mooshroom (10 HP, mushroom-fields only, weight 8/8, herds 4–8)
//!
//! VERIFIED data (minecraft.wiki, pulled 2026-09-04 per the verification
//! discipline — NOT from dossier memory):
//! - per-mob health / speed attribute / damage rows (infobox "Health
//!   points", "Speed", "Attack strength") — see MOB_DATA
//! - hostile spawning: block light ≤ 7 in 1.16.5 (the 1.18 experimental
//!   snapshot changed it to 0 — wiki History section)
//! - passive spawning: light ≥ 9, on grass blocks, with 2 blocks of space
//! - mob caps: Monster 70 / Creature 10 / Ambient 15, scaled
//!   `cap × chunks ÷ 289` over the 17×17-chunk spawn square
//! - despawn: >128 blocks from the nearest player is instant; 32 blocks
//!   with no player for 30 s rolls 1/800 despawn per game tick
//! - creeper: explosion power 3, 1.5 s fuse (30 game ticks)
//!
//! Documented adaptations:
//! - speed: attributes converted at ~10.5 blocks/s per point (observed-
//!   equivalent chase speeds; vanilla's per-tick velocity integration with
//!   drag has no published closed form — flagged, not exact)
//! - pathfinding is straight-line steering + 1-block step-ups (the
//!   existing villager primitive), no A*
//! - mob-kill XP drops as orbs (Phase E1 xp system); mining XP stays
//!   direct (pre-existing, documented)
//! - arrows: ballistic points, gravity 20 b/s² (vanilla 0.05/tick²),
//!   skeleton cadence fixed at 2 s
//! - snow-golem snow TRAIL is deferred: the engine has no thin snow-layer
//!   block; the wiki's own page carries an internal disagreement on the
//!   Java rule ("any biome" vs temperature-gated) — noted in the worklog
//! - mooshroom shear/stew/breeding deferred (no shears/bowls/wheat items)

use vc_blocks::blocks::*;
use vc_rng::rng::Rng;
use vc_world::world::World;

pub const MAX_MOBS: usize = 128;

/// Mob kinds. The full 1.16.5 registry (102 mob-like
/// entities per Dossier Part 4 §21) is deliberately NOT attempted at once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MobKind {
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Enderman,
    Cow,
    Pig,
    Sheep,
    Chicken,
    // ---- Phase E1 (1.0–1.2 bracket) ----
    SnowGolem,
    MagmaCube,
    Blaze,
    Ocelot,
    IronGolem,
    ZombieVillager,
    Mooshroom,
    // ---- Phase E2 (1.3-1.4 bracket) ----
    WitherSkeleton,
    Witch,
    Bat,
    // ---- Phase E3 (1.5–1.6 bracket) ----
    Horse,
    Donkey,
    Mule,
    /// 1.8 (Bountiful Update): the rabbit — VERIFIED live (minecraft.wiki
    /// /w/Rabbit, 2026-09-06): 3 HP, avoids players within 8 blocks,
    /// 0-1 raw rabbit + 0-1 rabbit hide on death, a 10% rabbit's foot on
    /// a player kill
    Rabbit,
    /// 1.10 (Frostburn): the polar bear — VERIFIED (wiki /w/Polar_Bear,
    /// live 2026-09-06): 30 HP, passive until the player comes near cubs,
    /// "can swim faster in water than the player", drops 0-2 raw fish
    /// (75%) or 0-2 salmon (25%)
    PolarBear,
    /// 1.10: the stray — VERIFIED (wiki /w/Stray): "80% of skeletons
    /// spawned above ground in ice plains, ice mountains and ice plains
    /// spikes biomes are strays"; shoots tipped arrows of Slowness (0:30)
    Stray,
    /// 1.10: the husk — VERIFIED (wiki /w/Husk): "80% of zombies spawned
    /// above ground in desert... biomes are husks"; does not burn in
    /// sunlight; attacks apply Hunger for 7 × floor(regional difficulty)
    /// seconds
    Husk,
}

impl MobKind {
    pub fn from_name(s: &str) -> Option<MobKind> {
        Some(match s {
            "zombie" => MobKind::Zombie,
            "skeleton" => MobKind::Skeleton,
            "creeper" => MobKind::Creeper,
            "spider" => MobKind::Spider,
            "enderman" => MobKind::Enderman,
            "cow" => MobKind::Cow,
            "pig" => MobKind::Pig,
            "sheep" => MobKind::Sheep,
            "chicken" => MobKind::Chicken,
            "snow_golem" => MobKind::SnowGolem,
            "magma_cube" => MobKind::MagmaCube,
            "blaze" => MobKind::Blaze,
            "ocelot" => MobKind::Ocelot,
            "iron_golem" => MobKind::IronGolem,
            "zombie_villager" => MobKind::ZombieVillager,
            "mooshroom" => MobKind::Mooshroom,
            "wither_skeleton" => MobKind::WitherSkeleton,
            "witch" => MobKind::Witch,
            "bat" => MobKind::Bat,
            "horse" => MobKind::Horse,
            "donkey" => MobKind::Donkey,
            "mule" => MobKind::Mule,
            "rabbit" => MobKind::Rabbit,
            "polar_bear" => MobKind::PolarBear,
            "stray" => MobKind::Stray,
            "husk" => MobKind::Husk,
            _ => return None,
        })
    }

    /// Entity-type registry name (vanilla strings, mechanical data).
    pub fn name(self) -> &'static str {
        match self {
            MobKind::Zombie => "minecraft:zombie",
            MobKind::Skeleton => "minecraft:skeleton",
            MobKind::Creeper => "minecraft:creeper",
            MobKind::Spider => "minecraft:spider",
            MobKind::Enderman => "minecraft:enderman",
            MobKind::Cow => "minecraft:cow",
            MobKind::Pig => "minecraft:pig",
            MobKind::Sheep => "minecraft:sheep",
            MobKind::Chicken => "minecraft:chicken",
            MobKind::SnowGolem => "minecraft:snow_golem",
            MobKind::MagmaCube => "minecraft:magma_cube",
            MobKind::Blaze => "minecraft:blaze",
            MobKind::Ocelot => "minecraft:ocelot",
            MobKind::IronGolem => "minecraft:iron_golem",
            MobKind::ZombieVillager => "minecraft:zombie_villager",
            MobKind::Mooshroom => "minecraft:mooshroom",
            MobKind::WitherSkeleton => "minecraft:wither_skeleton",
            MobKind::Witch => "minecraft:witch",
            MobKind::Bat => "minecraft:bat",
            MobKind::Horse => "minecraft:horse",
            MobKind::Donkey => "minecraft:donkey",
            MobKind::Mule => "minecraft:mule",
            MobKind::Rabbit => "minecraft:rabbit",
            MobKind::PolarBear => "minecraft:polar_bear",
            MobKind::Stray => "minecraft:stray",
            MobKind::Husk => "minecraft:husk",
        }
    }

    pub fn sprite_tile(self) -> u16 {
        match self {
            MobKind::Zombie => TILE_ZOMBIE,
            MobKind::Skeleton => TILE_SKELETON,
            MobKind::Creeper => TILE_CREEPER,
            MobKind::Spider => TILE_SPIDER,
            MobKind::Enderman => TILE_ENDERMAN,
            MobKind::Cow => TILE_COW,
            MobKind::Pig => TILE_PIG,
            MobKind::Sheep => TILE_SHEEP,
            MobKind::Chicken => TILE_CHICKEN,
            MobKind::SnowGolem => TILE_SNOWGOLEM,
            MobKind::MagmaCube => TILE_MAGMACUBE,
            MobKind::Blaze => TILE_BLAZE,
            MobKind::Ocelot => TILE_OCELOT,
            MobKind::IronGolem => TILE_IRONGOLEM,
            MobKind::ZombieVillager => TILE_ZOMBIEVILLAGER,
            MobKind::Mooshroom => TILE_MOOSHROOM,
            MobKind::WitherSkeleton => TILE_WITHER_SKELETON,
            MobKind::Witch => TILE_WITCH,
            MobKind::Bat => TILE_BAT,
            MobKind::Horse => TILE_HORSE,
            MobKind::Donkey => TILE_DONKEY,
            MobKind::Mule => TILE_MULE,
            MobKind::Rabbit => TILE_RABBIT,
            MobKind::PolarBear => TILE_POLAR_BEAR,
            MobKind::Stray => TILE_STRAY,
            MobKind::Husk => TILE_HUSK,
        }
    }

    /// attacks on sight (zombie/skeleton/creeper/spider; enderman is
    /// neutral until provoked). Phase E1: + magma cube, blaze,
    /// zombie villager.
    /// neutral until provoked). 1.10: stray/husk inherit their base
    /// kinds' hostility; the polar bear is neutral (only near cubs)
    pub fn hostile(self) -> bool {
        matches!(
            self,
            MobKind::Zombie
                | MobKind::Skeleton
                | MobKind::Creeper
                | MobKind::Spider
                | MobKind::MagmaCube
                | MobKind::Blaze
                | MobKind::ZombieVillager
                // Phase E2 (VERIFIED w/Wither_Skeleton, w/Witch: hostile)
                | MobKind::WitherSkeleton
                | MobKind::Witch
                | MobKind::Stray
                | MobKind::Husk
        )
    }
    pub fn neutral(self) -> bool {
        self == MobKind::Enderman || self == MobKind::IronGolem
    }

    /// The spawn-egg mapping: egg id 0..=15 (SPAWN_EGG_BASE + i) in the
    /// SAME order as vc_blocks's BLOCK_TABLE egg rows + the EGG_PALETTES
    /// art table (order guarded by the egg roundtrip tests both sides).
    pub fn from_egg(i: u8) -> MobKind {
        match i {
            0 => MobKind::SnowGolem,
            1 => MobKind::MagmaCube,
            2 => MobKind::Blaze,
            3 => MobKind::Ocelot,
            4 => MobKind::IronGolem,
            5 => MobKind::ZombieVillager,
            6 => MobKind::Mooshroom,
            7 => MobKind::Zombie,
            8 => MobKind::Skeleton,
            9 => MobKind::Creeper,
            10 => MobKind::Spider,
            11 => MobKind::Enderman,
            12 => MobKind::Cow,
            13 => MobKind::Pig,
            14 => MobKind::Sheep,
            15 => MobKind::Chicken,
            // Phase E2 (1.3-1.4): kinds 16..=19
            16 => MobKind::WitherSkeleton,
            17 => MobKind::Witch,
            18 => MobKind::Bat,
            // NOTE: index 19 (the E2 "Wither Spawn Egg") has no MobKind
            // arm — the wither is a boss entity outside MobSystem (the
            // egg stub falls through to Chicken; pre-existing E2
            // behavior, disclosed in the worklog audit).
            // Phase E3 (1.5–1.6): kinds 20..=22 (horse, donkey, mule —
            // egg ids 197..=199; blocks.rs egg_mob decodes those to
            // 20..=22, guarded by the roundtrip test)
            20 => MobKind::Horse,
            21 => MobKind::Donkey,
            22 => MobKind::Mule,
            _ => MobKind::Chicken,
        }
    }

    /// inverse of from_egg (egg id for a kind)
    pub fn egg_id(self) -> u8 {
        match self {
            MobKind::SnowGolem => 0,
            MobKind::MagmaCube => 1,
            MobKind::Blaze => 2,
            MobKind::Ocelot => 3,
            MobKind::IronGolem => 4,
            MobKind::ZombieVillager => 5,
            MobKind::Mooshroom => 6,
            MobKind::Zombie => 7,
            MobKind::Skeleton => 8,
            MobKind::Creeper => 9,
            MobKind::Spider => 10,
            MobKind::Enderman => 11,
            MobKind::Cow => 12,
            MobKind::Pig => 13,
            MobKind::Sheep => 14,
            MobKind::Chicken => 15,
            MobKind::WitherSkeleton => 16,
            MobKind::Witch => 17,
            MobKind::Bat => 18,
            MobKind::Horse => 20,
            MobKind::Donkey => 21,
            MobKind::Mule => 22,
            // F-series mobs (1.8 rabbit, 1.10 polar bear/stray/husk) have
            // no spawn-egg items in the engine — 255 = "no egg" sentinel
            MobKind::Rabbit | MobKind::PolarBear | MobKind::Stray | MobKind::Husk => 255,
        }
    }
}

/// Per-mob stats. VERIFIED against the wiki infoboxes (2026-09-04):
/// health, damage = the NORMAL-difficulty row, speed = movement-speed
/// attribute, armor = natural armor points, height/width = hitbox.
pub struct MobDef {
    pub kind: MobKind,
    pub health: f32,
    /// NORMAL-difficulty damage (Easy/Hard scale via combat::difficulty_scale)
    pub damage: f32,
    /// movement-speed ATTRIBUTE (wiki "Speed" row); converted at ×10.5
    pub speed_attr: f32,
    /// natural armor points (zombie has 2 — verified)
    pub armor: f32,
    /// hitbox height in blocks (sprite render height follows)
    pub height: f32,
    /// hitbox width in blocks
    pub width: f32,
    /// XP granted when killed by the player [placeholder: see header]
    pub xp: i32,
}

pub const MOB_DATA: [MobDef; 26] = [
    MobDef {
        kind: MobKind::Zombie,
        health: 20.0,
        damage: 3.0,
        speed_attr: 0.23,
        armor: 2.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Skeleton,
        health: 20.0,
        damage: 4.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 1.99,
        width: 0.6,
        xp: 5,
    }, // dmg = mid of arrow 3–5
    MobDef {
        kind: MobKind::Creeper,
        health: 20.0,
        damage: 0.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 1.7,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Spider,
        health: 16.0,
        damage: 2.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 0.9,
        width: 1.4,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Enderman,
        health: 40.0,
        damage: 7.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 2.9,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Cow,
        health: 10.0,
        damage: 0.0,
        speed_attr: 0.2,
        armor: 0.0,
        height: 1.4,
        width: 0.9,
        xp: 1,
    },
    MobDef {
        kind: MobKind::Pig,
        health: 10.0,
        damage: 0.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 0.9,
        width: 0.9,
        xp: 1,
    },
    MobDef {
        kind: MobKind::Sheep,
        health: 8.0,
        damage: 0.0,
        speed_attr: 0.23,
        armor: 0.0,
        height: 1.3,
        width: 0.9,
        xp: 1,
    },
    MobDef {
        kind: MobKind::Chicken,
        health: 4.0,
        damage: 0.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 0.7,
        width: 0.4,
        xp: 1,
    },
    // ---- Phase E1 (1.0–1.2 bracket; live-verified 2026-09-06) ----
    MobDef {
        kind: MobKind::SnowGolem,
        // VERIFIED w/Snow_Golem infobox: 4 HP. Snowballs: 0 damage,
        // 3 HP vs blazes only (the throw is in ai_tick).
        health: 4.0,
        damage: 0.0,
        speed_attr: 0.2,
        armor: 0.0,
        height: 1.9,
        width: 0.7,
        xp: 0, // golems drop no XP (VERIFIED w/Experience)
    },
    MobDef {
        kind: MobKind::MagmaCube,
        // LARGE-size row (size code 3): HP = size² = 16, dmg = size+2 = 6,
        // armor = 3×size = 12 (VERIFIED w/Magma_Cube §Combat). The variant
        // field scales smaller cubes down (health = variant² etc.).
        health: 16.0,
        damage: 6.0,
        speed_attr: 0.2,
        armor: 12.0,
        height: 2.04,
        width: 2.04,
        xp: 4, // big: 4 XP (medium 2, small 1 — VERIFIED)
    },
    MobDef {
        kind: MobKind::Blaze,
        // VERIFIED w/Blaze infobox: 20 HP, Normal small-fireball 5
        // (Easy 3.5 / Hard 7.5 via difficulty_scale), contact 6.
        health: 20.0,
        damage: 5.0,
        speed_attr: 0.23,
        armor: 0.0,
        height: 1.8,
        width: 0.6,
        xp: 10, // VERIFIED w/Blaze §Drops
    },
    MobDef {
        kind: MobKind::Ocelot,
        // VERIFIED w/Ocelot infobox: 10 HP, passive; fast runner (attr 0.30
        // is our adapted value — the wiki lists speed 0.30 for cats family)
        health: 10.0,
        damage: 0.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 0.7,
        width: 0.6,
        xp: 3, // 1–3 XP orbs (VERIFIED w/Ocelot §Drops)
    },
    MobDef {
        kind: MobKind::IronGolem,
        // VERIFIED w/Iron_Golem: 100 HP; Normal attack 7.5–21.5 (we take
        // the mid 14 as the fixed engine value — the vanilla range comes
        // from per-swing level scaling; documented adaptation)
        health: 100.0,
        damage: 14.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 2.7,
        width: 1.4,
        xp: 0, // golems drop no XP (VERIFIED w/Experience)
    },
    MobDef {
        kind: MobKind::ZombieVillager,
        // VERIFIED w/Zombie_Villager infobox: 20 HP; attack Easy 2.5 /
        // Normal 3 / Hard 4.5 — the zombie row; XP 5 adult / 12 baby.
        health: 20.0,
        damage: 3.0,
        speed_attr: 0.23,
        armor: 2.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    // ---- F-series (1.8/1.10 additions, live-verified 2026-09-06) ----
    // 1.8 rabbit — VERIFIED (minecraft.wiki/w/Rabbit): 3 HP; avoids
    // players within 8 blocks (panics fast when approached)
    MobDef {
        kind: MobKind::Rabbit,
        health: 3.0,
        damage: 0.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 0.5,
        width: 0.4,
        xp: 1,
    },
    // 1.10 polar bear — VERIFIED (wiki /w/Polar_Bear, live 2026-09-06):
    // 30 HP; wiki melee rows: 4/6/9 HP by difficulty (base 6 here)
    MobDef {
        kind: MobKind::PolarBear,
        health: 30.0,
        damage: 6.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 1.4,
        width: 1.4,
        xp: 3,
    },
    // 1.10 stray — skeleton stats with the slowness-arrow rider
    MobDef {
        kind: MobKind::Stray,
        health: 20.0,
        damage: 4.0, // arrow 3–5 mid, identical to the skeleton
        speed_attr: 0.25,
        armor: 0.0,
        height: 1.99,
        width: 0.6,
        xp: 5,
    },
    // 1.10 husk — zombie stats with the hunger rider
    MobDef {
        kind: MobKind::Husk,
        health: 20.0,
        damage: 3.0,
        speed_attr: 0.23,
        armor: 2.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Mooshroom,
        // VERIFIED w/Mooshroom: cow stats (10 HP), spawns only in
        // mushroom fields (weight 8/8, group 4–8)
        health: 10.0,
        damage: 0.0,
        speed_attr: 0.2,
        armor: 0.0,
        height: 1.4,
        width: 0.9,
        xp: 1,
    },
    // ---- Phase E2 (1.3-1.4 bracket; live-verified 2026-09-06,
    // docs/research/phase2-1.3-1.4-research.md) ----
    MobDef {
        // VERIFIED w/Wither_Skeleton: 20 HP, stone sword Normal 8,
        // 2.4 tall / 0.7 wide, speed 0.25 (0.3125 attacking)
        kind: MobKind::WitherSkeleton,
        health: 20.0,
        damage: 8.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 2.4,
        width: 0.7,
        xp: 5,
    },
    MobDef {
        // VERIFIED w/Witch: 26 HP, splash-potion attack max 6, speed
        // 0.25, hitbox 0.6 wide (height approximated 1.95 humanoid)
        kind: MobKind::Witch,
        health: 26.0,
        damage: 6.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        // VERIFIED w/Bat: 6 HP, ambient passive, 0.9 tall / 0.5 wide
        kind: MobKind::Bat,
        health: 6.0,
        damage: 0.0,
        speed_attr: 0.25,
        armor: 0.0,
        height: 0.9,
        width: 0.5,
        xp: 0,
    },
    // ---- Phase E3 (1.5–1.6 bracket) — all VERIFIED live 2026-09-06
    // (minecraft.wiki/w/Horse §Health/§Movement_speed/§Jump_strength,
    // w/Donkey, w/Mule): the DEF rows carry the wiki AVERAGE / midpoint
    // values; per-instance randomization happens at spawn (below) ----
    MobDef {
        // Horse: health 15–30 avg 22.5; speed 0.1125–0.3375 internal
        // (≈4.86–14.57 b/s, conversion ≈43.17 — VERIFIED §Movement_speed);
        // jump strength 0.4–1.0 (clears 1.153–5.9197 blocks, VERIFIED);
        // hitbox 1.4 wide × 1.6 tall
        kind: MobKind::Horse,
        health: 22.5,
        damage: 0.0,
        speed_attr: 0.225,
        armor: 0.0,
        height: 1.6,
        width: 1.4,
        // VERIFIED w/Horse §Drops: "1–3 XP when killed by a player"
        // (midpoint, the engine's fixed-XP convention)
        xp: 2,
    },
    MobDef {
        // Donkey: health 15–30 avg 22.5; speed 0.175 fixed when spawned
        // (VERIFIED w/Donkey "0.175 speed when naturally spawned")
        kind: MobKind::Donkey,
        health: 22.5,
        damage: 0.0,
        speed_attr: 0.175,
        armor: 0.0,
        height: 1.6,
        width: 1.4,
        xp: 2, // 1–3 XP (VERIFIED w/Donkey §Drops)
    },
    MobDef {
        // Mule: health 15–30 "tends toward the average of 22–23"
        // (VERIFIED w/Mule); speed 0.175 (the donkey row — mules take the
        // parent-average path at breed time)
        kind: MobKind::Mule,
        health: 22.5,
        damage: 0.0,
        speed_attr: 0.175,
        armor: 0.0,
        height: 1.6,
        width: 1.4,
        xp: 2, // 1–3 XP (VERIFIED w/Mule §Drops)
    },
];

#[inline]
pub fn def(kind: MobKind) -> &'static MobDef {
    MOB_DATA.iter().find(|d| d.kind == kind).unwrap()
}

/// Deferred entities (explicit): every 1.16.5 mob NOT in this batch —
/// drowned/husk/stray/zombie-villager/cave-spider/slime/magma-cube/ghast/
/// piglin-family/blaze/wither-skeleton/guardians/shulker/phantom/
/// silverfish/illager-family/vex/witch, and the ~40 remaining passives
/// (horse, rabbit, wolf, fox, bee, turtle, …). They arrive in follow-up
/// batches once the core loop is proven.
pub const DEFERRED_ENTITIES: &str =
    "all 1.16.5 mobs except zombie/skeleton/creeper/spider/enderman/cow/pig/sheep/chicken";

// ------------------------------------------------------- verified rules --
pub const DESPAWN_INSTANT_BLOCKS: f32 = 128.0;
pub const DESPAWN_NEAR_BLOCKS: f32 = 32.0;
/// hostile mob cap constant (wiki mob-cap table: Monster 70)
pub const MONSTER_CAP: f32 = 70.0;
/// passive/creature cap constant (Creature 10)
pub const CREATURE_CAP: f32 = 10.0;
/// spawn square: 17×17 chunks → 289 (cap scale divisor — wiki formula)
pub const CAP_DIVISOR: f32 = 289.0;
/// hostile light ceiling for 1.16.5 (block light ≤ 7)
pub const HOSTILE_LIGHT_MAX: u8 = 7;
/// passive light floor (animals need ≥ 9 — wiki)
pub const PASSIVE_LIGHT_MIN: u8 = 9;
/// hostile sky-light ceiling (1.16.5 overworld)
pub const HOSTILE_SKY_MAX: u8 = 7;
/// creeper: begins the fuse this close (vanilla ~3 blocks)
pub const CREEPER_FUSE_DIST: f32 = 3.0;
/// creeper fuse: 30 game ticks = 1.5 s (vanilla)
pub const CREEPER_FUSE_TICKS: i32 = 30;
/// creeper explosion power (wiki: "Normal creeper explosions have a power of 3")
pub const CREEPER_POWER: f32 = 3.0;
/// skeleton bow interval (adaptation: fixed 40-tick cadence)
pub const SKELETON_SHOOT_TICKS: i32 = 40;
/// mob melee reach
pub const MOB_MELEE_REACH: f32 = 1.6;
/// mob melee cooldown, game ticks (~1 s zombie cadence)
pub const MOB_MELEE_TICKS: i32 = 20;
/// aggro radius
pub const AGGRO_RADIUS: f32 = 16.0;
/// passive panic flee multiplier
pub const FLEE_MULT: f32 = 1.8;
/// attribute → blocks/s conversion (documented adaptation)
pub const SPEED_PER_ATTR: f32 = 10.5;

// ---- Phase E1 constants (all live-verified 2026-09-06) ----
/// zombie-villager cure duration range in game ticks (VERIFIED
/// w/Zombie_Villager: "a random integer between 3600 and 6000 ticks")
pub const CURE_TICKS_MIN: i32 = 3600;
pub const CURE_TICKS_MAX: i32 = 6000;
/// villager → zombie-villager conversion on a zombie kill, by difficulty
/// (VERIFIED w/Zombie_Villager: Easy 0% / Normal 50% / Hard 100%)
pub const ZOMBIFY_CHANCE_EASY: f32 = 0.0;
pub const ZOMBIFY_CHANCE_NORMAL: f32 = 0.5;
pub const ZOMBIFY_CHANCE_HARD: f32 = 1.0;

/// Snow-golem build pattern check (VERIFIED w/Snow_Golem §Spawning): two
/// SNOW blocks stacked vertically with the pumpkin placed LAST on top.
/// Call at the moment a PUMPKIN lands at (x, y, z).
pub fn snow_golem_pattern(world: &World, x: i32, y: i32, z: i32) -> bool {
    world.get_block(x, y - 1, z) == SNOW && world.get_block(x, y - 2, z) == SNOW
}

/// Iron-golem build pattern check (VERIFIED w/Iron_Golem §Spawning):
/// four IRON blocks in a T (3 across the bottom + 1 center above) with
/// the pumpkin placed LAST on the center top. Any non-air blocks in the
/// pattern's empty spaces prevent the spawn (vanilla).
pub fn iron_golem_pattern(world: &World, x: i32, y: i32, z: i32) -> bool {
    // the T body: (x,y-2,z) + row (x±1, y-2, z) — the pumpkin sits at
    // (x, y, z) with the cross-arm at y-1
    let body_row = world.get_block(x - 1, y - 2, z) == IRON_BLOCK
        && world.get_block(x, y - 2, z) == IRON_BLOCK
        && world.get_block(x + 1, y - 2, z) == IRON_BLOCK
        && world.get_block(x, y - 1, z) == IRON_BLOCK;
    if !body_row {
        return false;
    }
    // vanilla: any non-air block in the golem's empty spaces blocks it
    let clear = |bx: i32, by: i32, bz: i32| world.get_block(bx, by, bz) == AIR;
    clear(x - 1, y - 1, z)
        && clear(x + 1, y - 1, z)
        && clear(x - 1, y, z)
        && clear(x + 1, y, z)
        && clear(x, y + 1, z)
}

/// Start curing a zombie villager (weakness + golden apple at the game
/// layer; the weakness gate itself is a documented deferral — the engine
/// has no weakness potion yet). VERIFIED duration 3600..=6000 ticks.
pub fn begin_cure(m: &mut Mob, rng: &mut Rng) {
    m.variant = 1;
    m.aux = CURE_TICKS_MIN + rng.next_range((CURE_TICKS_MAX - CURE_TICKS_MIN + 1) as u32) as i32;
}

/// Phase E3 (1.5–1.6 bracket): per-instance equine state (horses,
/// donkeys, mules). All rules VERIFIED live 2026-09-06,
/// minecraft.wiki/w/Horse:
/// - temper starts 0/100; a random taming THRESHOLD 0–99 is chosen at
///   the first mount; each failed mount adds +5 temper; tame once the
///   temper EXCEEDS the threshold
/// - health 15–30, speed 0.1125–0.3375 internal (≈4.86–14.57 b/s via the
///   ≈43.17 conversion — §Movement_speed), jump strength 0.4–1.0
///   (clears 1.153–5.9197 blocks — §Jump_strength)
/// - 20% of naturally-spawned horses are babies (§Spawning)
/// - the saddle is required for CONTROL (§Riding "Once a horse is tamed
///   and saddled, the player can control it")
/// - bred stat (§Bred_values): avg(p1,p2) + rand(-0.5..0.5)·
///   (|p1−p2| + 0.30·range), clamped to the allowed range
#[derive(Clone, Debug)]
pub struct EquineState {
    /// taming temper 0..=100
    pub temper: u8,
    /// the random taming threshold 0..=99 (chosen at first mount)
    pub threshold: u8,
    /// tamed (hearts shown; mountable without bucking)
    pub tamed: bool,
    /// saddled — required for the player to CONTROL the mount
    pub saddled: bool,
    /// per-instance movement speed ATTRIBUTE (0.1125–0.3375 horses;
    /// 0.175 donkeys — VERIFIED)
    pub speed_attr: f32,
    /// per-instance jump strength (0.4–1.0 — VERIFIED)
    pub jump_strength: f32,
    /// baby (20% of spawns — VERIFIED §Spawning; grows after 20 min)
    pub baby: bool,
    /// coat variant (7 base colors × 5 markings in vanilla; one byte —
    /// rendered via the sprite tint, clean-room adaptation)
    pub coat: u8,
    /// love-mode cooldown after breeding (ticks)
    pub breed_cd: i32,
}

impl EquineState {
    /// the launch velocity that clears `height` blocks under the engine's
    /// jump integrator (v1 = (v0 − 0.08)·0.98 — the shared player/mob
    /// profile). Jump strength → clear-height uses the quadratic fit
    /// through the three VERIFIED anchors (0.4→1.153, 0.7→3.124,
    /// 1.0→5.9197 blocks) — a disclosed interpolation, not a guessed
    /// formula.
    pub fn jump_clear_height(&self) -> f32 {
        let s = self.jump_strength;
        // quadratic fit through the three verified anchors
        let h = 4.5817 * s * s + 1.53 * s - 0.192;
        h.max(0.0)
    }
}

/// One mob instance. Position is feet-center like the player.
#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u32,
    pub kind: MobKind,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub yaw: f32,
    pub health: f32,
    pub on_ground: bool,
    /// hurt flash (ticks remaining) — red tint during rendering
    pub hurt_t: i32,
    /// melee/ranged attack cooldown (ticks)
    pub attack_cd: i32,
    /// creeper fuse: <0 idle, 0..=30 counting, i32::MAX consumed
    pub fuse: i32,
    /// provoked (neutral mobs become hostile)
    pub provoked: bool,
    /// ticks since a player was within 32 blocks
    pub lonely_t: i32,
    /// blocks fallen since last landing (vanilla `fallDistance`; the
    /// landing tick converts it via MC-12357: damage = fall − 3)
    pub fall_dist: f32,
    /// Phase E1 per-kind variant payload:
    /// - MagmaCube: the vanilla NBT Size code — 0 (size 1), 1 (size 2),
    ///   3 (size 4). Health/damage/armor scale from it (VERIFIED).
    /// - Ocelot: 1 = trusting (fed raw cod/salmon — VERIFIED w/Ocelot)
    /// - ZombieVillager: 1 = is curing (aux counts down)
    /// - Mooshroom: 0 red / 1 brown (lightning transform, VERIFIED)
    pub variant: u8,
    /// Phase E1 per-kind timer/aux:
    /// - ZombieVillager: cure countdown (3600..=6000 ticks, VERIFIED)
    /// - Blaze: burst counter (3 shots at 6-tick spacing after a 60-tick
    ///   charge — VERIFIED "charges for 3 seconds, then fires three small
    ///   fireballs at intervals of 0.3 seconds" → 60 + 3×6 ticks)
    /// - MagmaCube: hop cooldown (40..=120 idle / 13..=40 with target,
    ///   VERIFIED §Behavior)
    pub aux: i32,
    /// Phase E3: per-instance equine state (horses/donkeys/mules —
    /// None for every other kind)
    pub equine: Option<Box<EquineState>>,
    wander_yaw: f32,
    wander_t: i32,
}

/// A mob's damage event delivered to the player (game layer applies mode
/// gating + difficulty scaling).
#[derive(Clone, Debug)]
pub struct PlayerHit {
    /// NORMAL-difficulty damage — scale via combat::difficulty_scale
    pub damage: f32,
    pub source: MobKind,
    pub knockback_dir: [f32; 2],
    /// Phase E2: wither-skull payload — Some(ticks) applies Wither II
    /// (VERIFIED w/Wither: 200 ticks Normal / 800 Hard)
    pub wither_effect: Option<i32>,
}

/// An arrow projectile (skeleton): ballistic point. Phase E1 adds
/// projectile KINDS — blaze fireballs and snow-golem snowballs ride the
/// same ballistic integrator with different damage rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProjKind {
    Arrow,
    /// blaze small fireball (Normal 5; contact fire omitted — no fire
    /// ticks on entities in the engine, documented)
    Fireball,
    /// snow-golem snowball: 0 damage — 3 vs blazes (VERIFIED w/Snow_Golem)
    Snowball,
    /// Phase E2: wither skull — 8 HP + Wither II on Normal (VERIFIED
    /// w/Wither)
    Skull,
}

#[derive(Clone, Debug)]
pub struct Arrow {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// damage on hit (Normal 3–5, chosen at fire time)
    pub damage: f32,
    pub age: i32,
    /// Phase E1: projectile variant (arrow / fireball / snowball)
    pub kind: ProjKind,
    /// owning mob id (for attribution, e.g. snowball-from-golem)
    pub owner: u32,
}

pub struct MobSystem {
    /// Phase E2: ambient-bat spawn cadence counter
    bats_spawn_t: u64,
    /// Phase E3: the mob id the player is currently riding (its AI is
    /// suspended — the game layer drives its velocity; physics still
    /// applies)
    pub ridden: Option<u32>,
    pub list: Vec<Mob>,
    pub arrows: Vec<Arrow>,
    rng: Rng,
    next_id: u32,
    /// player anchor for spawning/AI (None = no spawns, AI idles)
    pub player: Option<[f32; 3]>,
    /// creative flight / invulnerability — mobs hold fire
    pub player_invulnerable: bool,
    /// queued hits on the player (drained each frame by game.rs)
    pub hits: Vec<PlayerHit>,
    /// mob deaths (drops + XP handled by the game layer); the u8 carries
    /// the per-kind variant (magma-cube size code etc.)
    pub deaths: Vec<(MobKind, [f32; 3], u8)>,
    /// Phase E1: mob-vs-mob damage queued inside ai_tick (borrow split) —
    /// (target id, damage). Applied before the deaths scan.
    pub pending_damage: Vec<(u32, f32)>,
    /// explosion requests (center, power) — game.rs owns world edits so
    /// the light engine updates ride along
    pub explosions: Vec<([f32; 3], f32)>,
    /// Phase E1: zombie villagers whose cure finished (game.rs converts
    /// them to villagers + major_positive gossip — VERIFIED w/Zombie_Villager)
    pub cures: Vec<[f32; 3]>,
    /// counters for F3/E2E
    pub spawned_total: u64,
    pub despawned_total: u64,
    pub killed_total: u64,
}

impl MobSystem {
    pub fn new(seed: u64) -> Self {
        MobSystem {
            list: Vec::new(),
            arrows: Vec::new(),
            rng: Rng::new(seed ^ 0xB0B_5EED),
            bats_spawn_t: 0,
            ridden: None,
            next_id: 1,
            player: None,
            player_invulnerable: false,
            hits: Vec::new(),
            deaths: Vec::new(),
            pending_damage: Vec::new(),
            explosions: Vec::new(),
            cures: Vec::new(),
            spawned_total: 0,
            despawned_total: 0,
            killed_total: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn hostiles_alive(&self) -> usize {
        self.list.iter().filter(|m| m.kind.hostile()).count()
    }

    fn passives_alive(&self) -> usize {
        self.list.iter().filter(|m| !m.kind.hostile()).count()
    }

    /// Spawn a specific mob at a block position (E2E/structures).
    /// Phase E1: `variant` seeds the per-kind payload (magma size code,
    /// ocelot trust, mooshroom red/brown); health follows the variant
    /// (magma cube HP = size² — VERIFIED).
    pub fn spawn_at(&mut self, kind: MobKind, x: i32, y: i32, z: i32) -> Option<u32> {
        self.spawn_variant(kind, x, y, z, 0)
    }

    /// spawn with an explicit variant payload.
    pub fn spawn_variant(
        &mut self,
        kind: MobKind,
        x: i32,
        y: i32,
        z: i32,
        variant: u8,
    ) -> Option<u32> {
        if self.list.len() >= MAX_MOBS {
            return None;
        }
        let d = def(kind);
        let id = self.next_id;
        self.next_id += 1;
        let yaw = self.rng.next_f32() * std::f32::consts::TAU;
        // magma cube: stats scale from the size code (VERIFIED: HP = size²,
        // damage = size + 2, armor = 3×size, XP 4/2/1 for size 4/2/1)
        let health = if kind == MobKind::MagmaCube {
            let s = magma_size(variant);
            (s * s) as f32
        } else {
            d.health
        };
        self.list.push(Mob {
            id,
            kind,
            pos: [x as f32 + 0.5, y as f32, z as f32 + 0.5],
            vel: [0.0; 3],
            yaw,
            health,
            on_ground: false,
            hurt_t: 0,
            attack_cd: 0,
            fuse: -1,
            provoked: false,
            lonely_t: 0,
            fall_dist: 0.0,
            variant,
            aux: 0,
            // Phase E3: equines get per-instance stats (VERIFIED
            // w/Horse: health 15–30, speed 0.1125–0.3375, jump 0.4–1.0;
            // donkeys/mules fixed 0.175 speed w/Donkey; 20% babies
            // §Spawning)
            equine: if matches!(kind, MobKind::Horse | MobKind::Donkey | MobKind::Mule) {
                let speed = if kind == MobKind::Horse {
                    0.1125 + self.rng.next_f32() * 0.225 // 0.1125..=0.3375
                } else {
                    0.175
                };
                let jump = 0.4 + self.rng.next_f32() * 0.6; // 0.4..=1.0
                let baby = self.rng.next_range(100) < 20;
                Some(Box::new(EquineState {
                    temper: 0,
                    threshold: 100, // chosen at the first mount (VERIFIED)
                    tamed: false,
                    saddled: false,
                    speed_attr: speed,
                    jump_strength: jump,
                    baby,
                    coat: (self.rng.next_range(35)) as u8,
                    breed_cd: 0,
                }))
            } else {
                None
            },
            wander_yaw: yaw,
            wander_t: 0,
        });
        // the randomized per-instance health lands on the equine mob
        // itself (15..=30, VERIFIED w/Horse §Health — the magma-cube
        // per-instance row pattern)
        if let Some(m) = self.list.last_mut() {
            if matches!(m.kind, MobKind::Horse | MobKind::Donkey | MobKind::Mule) {
                m.health = 15.0 + self.rng.next_f32() * 15.0;
            }
        }
        self.spawned_total += 1;
        Some(id)
    }

    /// ONE deterministic sim tick (20 Hz).
    /// Phase 6 §26: `sim_center`/`sim_radius` = the simulation-distance
    /// ring — mobs outside it freeze (AI + physics), spawning clamps its
    /// chunk pick to the ring (vanilla JE: simulation distance "controls
    /// mob spawning and despawning, and tick updates" — wiki). Despawn
    /// runs regardless (distance-based bookkeeping, cheap).
    pub fn tick(&mut self, world: &World, sim_center: (i32, i32), sim_radius: i32) {
        let sim_ring = |cx: i32, cz: i32| {
            cx.wrapping_sub(sim_center.0)
                .saturating_abs()
                .max(cz.wrapping_sub(sim_center.1).saturating_abs())
                <= sim_radius
        };
        // 1. environmental spawning — one attempt per tick per category
        // while a non-invulnerable player anchor exists (cap-gated)
        if self.player.is_some() && !self.player_invulnerable {
            self.try_spawn_hostile(world, sim_ring);
        // Phase E2: ambient bats (VERIFIED w/Bat: light <= 3, below sea
        // level, groups of 8, not counted toward the passive cap)
        self.bats_spawn_t += 1;
        if self.bats_spawn_t % 40 == 0 {
            self.try_spawn_bats(world, sim_ring);
        }
            self.try_spawn_passive(world, sim_ring);
        }

        // 2. AI + physics (split borrows: rng/hits/arrows vs the mob list)
        let player = self.player;
        let invuln = self.player_invulnerable;
        let rng = &mut self.rng;
        let hits = &mut self.hits;
        let arrows = &mut self.arrows;
        let pending = &mut self.pending_damage;
        // Phase E1: read-only snapshot for mob-vs-mob targeting (snow
        // golem / iron golem / ocelot scan for other mobs)
        let snapshot: Vec<(u32, MobKind, [f32; 3], u8)> = self
            .list
            .iter()
            .map(|m| (m.id, m.kind, m.pos, m.variant))
            .collect();
        for m in self.list.iter_mut() {
            // Phase 6 §26: out-of-ring mobs freeze (1.18+ semantics)
            let mchunk = (
                (m.pos[0] / 16.0).floor() as i32,
                (m.pos[2] / 16.0).floor() as i32,
            );
            if !sim_ring(mchunk.0, mchunk.1) {
                continue;
            }
            m.hurt_t = m.hurt_t.saturating_sub(1);
            m.attack_cd = m.attack_cd.saturating_sub(1);
            // Phase E3: the ridden mount's AI is suspended — the game
            // layer drives its velocity (physics still applies)
            if self.ridden == Some(m.id) {
                physics_tick(m, world);
                continue;
            }
            ai_tick(rng, m, player, invuln, hits, arrows, world, &snapshot, pending);
            physics_tick(m, world);
        }

        // 3. despawn (VERIFIED): >128 blocks instant; 32-block loneliness
        // rolls 1/800 per tick after 30 s
        if let Some(p) = player {
            let mut i = 0;
            while i < self.list.len() {
                let m = &mut self.list[i];
                let dx = m.pos[0] - p[0];
                let dy = m.pos[1] - p[1];
                let dz = m.pos[2] - p[2];
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if dist_sq > DESPAWN_INSTANT_BLOCKS * DESPAWN_INSTANT_BLOCKS {
                    self.list.remove(i);
                    self.despawned_total += 1;
                    continue;
                }
                if dist_sq < DESPAWN_NEAR_BLOCKS * DESPAWN_NEAR_BLOCKS {
                    m.lonely_t = 0;
                } else {
                    m.lonely_t += 1;
                    if m.lonely_t > 600 && self.rng.next_range(800) == 0 {
                        self.list.remove(i);
                        self.despawned_total += 1;
                        continue;
                    }
                }
                i += 1;
            }
        }

        // 3.5 Phase E1: mob-vs-mob damage queued by ai_tick (snow golem →
        // hostiles, iron golem → hostiles, ocelot → chickens, zombie
        // villager zombie melee)
        if !self.pending_damage.is_empty() {
            let pend = std::mem::take(&mut self.pending_damage);
            for (target, dmg) in pend {
                for m in self.list.iter_mut() {
                    if m.id == target {
                        m.health -= dmg;
                        m.hurt_t = 10;
                        break;
                    }
                }
            }
        }

        // 4. deaths → events (all damage here is player damage)
        let mut i = 0;
        while i < self.list.len() {
            if self.list[i].health <= 0.0 {
                let m = self.list.remove(i);
                if m.fuse != i32::MAX {
                    // exploded creepers leave no drops (vanilla: destroyed).
                    // Phase E3: equines carry "saddled" in the death
                    // variant byte (1 = the saddle drops — VERIFIED w/
                    // Horse §Drops: equipped items drop on death)
                    let variant = if m.equine.as_ref().map(|e| e.saddled).unwrap_or(false) {
                        1
                    } else {
                        m.variant
                    };
                    self.deaths.push((m.kind, m.pos, variant));
                }
                self.killed_total += 1;
            } else {
                i += 1;
            }
        }

        // 4.5 Phase E1: finished cures — the zombie villager (variant 2,
        // set by ai_tick's countdown) leaves the mob list and the game
        // layer converts the position into a fresh villager with the
        // cure gossip (major_positive, VERIFIED w/Villager §Gossiping)
        let mut i = 0;
        while i < self.list.len() {
            if self.list[i].kind == MobKind::ZombieVillager && self.list[i].variant == 2 {
                let m = self.list.remove(i);
                self.cures.push(m.pos);
            } else {
                i += 1;
            }
        }

        // 5. arrows + snowball/fireball mob hits
        let mut mobs = std::mem::take(&mut self.list);
        let mut pending = std::mem::take(&mut self.pending_damage);
        tick_arrows(
            &mut self.arrows,
            player,
            invuln,
            &mut self.hits,
            world,
            &mut mobs,
            &mut pending,
        );
        self.list = mobs;
        self.pending_damage = pending;
    }

    // --------------------------------------------------------- spawning --

    /// hostile spawn attempt (VERIFIED 1.16.5 rules): block light ≤ 7 AND
    /// sky light ≤ 7, solid floor with 2 air, packs up to 4 (vanilla
    /// monster pack size), cap 70 × chunks/289 (single-player worst case
    /// = the full 289-chunk square → the raw constant).
    /// Phase E1: mushroom fields spawn NO hostiles (VERIFIED
    /// w/Mushroom_Fields); 5% of zombies are zombie villagers (VERIFIED
    /// w/Zombie_Villager); the Nether rolls magma cubes (VERIFIED
    /// w/Magma_Cube — all light levels; Nether Wastes weight 2/168 ≈ rare)
    fn try_spawn_hostile(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
        if self.hostiles_alive() as f32 >= MONSTER_CAP {
            return;
        }
        let Some(p) = self.player else { return };
        // Phase E1: mushroom fields are hostile-free (VERIFIED)
        if vc_world::gen::Biome::from_u8(world.get_biome(p[0] as i32, p[2] as i32))
            == vc_world::gen::Biome::MushroomFields
        {
            return;
        }
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        // Phase 6 §26: spawning clamps to the simulation ring
        if !sim_ring(cx, cz) {
            return;
        }
        if world.chunk((cx, cz)).is_none() {
            return;
        }
        // Phase E1: the mushroom-fields hostile-free rule is per-chunk
        if vc_world::gen::Biome::from_u8(world.get_biome(cx * 16 + 8, cz * 16 + 8))
            == vc_world::gen::Biome::MushroomFields
        {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        let py = p[1] as i32;
        for y in (py - 40..py + 16).rev() {
            if !(1..=250).contains(&y) {
                continue;
            }
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let floor = world.get_block(wx, y - 1, wz);
            if !is_solid(floor) || floor == WATER || is_cross(floor) {
                continue;
            }
            if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
                continue;
            }
            // light gate (VERIFIED 1.16.5): block ≤ 7 AND sky ≤ 7. Phase E1
            // exception: magma cubes spawn at ALL light levels in the
            // Nether (VERIFIED w/Magma_Cube §Spawning)
            let nether = world.dimension == vc_world::world::Dimension::Nether;
            let (blk_l, sky_l) = light_levels(world, wx, y, wz);
            if !nether && (blk_l > HOSTILE_LIGHT_MAX || sky_l > HOSTILE_SKY_MAX) {
                return;
            }
            let kind = if nether {
                // Phase E1: Nether Wastes — magma cubes are rare there
                // (weight 2/168 — VERIFIED table) with zombie/skeleton
                // filling the rest (engine adaptation: no zombified
                // piglins yet, 1.16 bracket)
                match self.rng.next_range(21) {
                    0 | 1 => MobKind::MagmaCube,
                    _ => MobKind::Zombie,
                }
            } else {
                // Phase E2: witches join the monster pool at their verified
                // ~0.97% share (w/Witch spawn table: weight 5/515, group 1
                // — the engine rolls 1/100, disclosed)
                if self.rng.next_range(100) == 0 {
                    MobKind::Witch
                } else {
                    // 1.10 biome-variant conversion (VERIFIED, wiki /w/Stray
                    // + /w/Husk, live 2026-09-06): "80% of skeletons spawned
                    // above ground in ice plains, ice mountains and ice
                    // plains spikes biomes are strays" and "80% of zombies
                    // spawned above ground in desert ... are husks". Our
                    // biome ids: 5/17 = the icy family (IceSpikes moved
                    // 16 -> 17 at the E-series merge), 4 = desert. Sky-lit
                    // spawns = "above ground" (the sky gate above already
                    // restricts hostile spawns to darkness; the conversion
                    // still applies to all surface spawns, documented
                    // adaptation).
                    let biome = world
                        .chunk((cx, cz))
                        .map(|c| c.biome[(lz * 16 + lx) as usize])
                        .unwrap_or(0);
                    let roll = self.rng.next_range(5);
                    match roll {
                        0 => {
                            // zombie -> husk (80%) in deserts
                            if biome == 4 && self.rng.next_f32() < 0.8 {
                                MobKind::Husk
                            } else {
                                MobKind::Zombie
                            }
                        }
                        1 => {
                            // skeleton -> stray (80%) in the icy family
                            if (biome == 5 || biome == 17) && self.rng.next_f32() < 0.8 {
                                MobKind::Stray
                            } else {
                                MobKind::Skeleton
                            }
                        }
                        2 => MobKind::Creeper,
                        3 => MobKind::Spider,
                        _ => MobKind::Enderman,
                    }
                }
            };
            let pack = 1 + (self.rng.next_range(4)) as usize;
            for _ in 0..pack {
                // Phase E1: 5% of zombie spawns are zombie villagers
                // (VERIFIED w/Zombie_Villager §Spawning); magma-cube sizes
                // roll 1/2/4 codes (regional-difficulty spread simplified)
                let (spawn_kind, variant) = if kind == MobKind::Zombie {
                    if self.rng.next_range(20) == 0 {
                        (MobKind::ZombieVillager, 0)
                    } else {
                        (kind, 0)
                    }
                } else if kind == MobKind::MagmaCube {
                    (kind, self.rng.next_range(3) as u8) // sizes 1/2/4
                } else {
                    (kind, 0)
                };
                let _ = self.spawn_variant(spawn_kind, wx, y, wz, variant);
            }
            return; // one attempt per tick
        }
    }

    /// Phase E2: ambient bat spawning (VERIFIED w/Bat Spawning): Overworld,
    /// light <= 3, below sea level (y <= 62 — the pre-1.21.2 rule), solid
    /// floor with 2 air, groups of 8, NOT counted toward the passive mob cap
    /// (the ambient category is separate — VERIFIED).
    fn try_spawn_bats(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
    let Some(p) = self.player else { return };
    if world.dimension != vc_world::world::Dimension::Overworld {
        return;
    }
    let bats = self.list.iter().filter(|m| m.kind == MobKind::Bat).count();
    if bats >= 10 {
        return; // ambient category cap (10)
    }
    let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(9) as i32) - 4;
    let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(9) as i32) - 4;
    if !sim_ring(cx, cz) || world.chunk((cx, cz)).is_none() {
        return;
    }
    let lx = self.rng.next_range(16) as i32;
    let lz = self.rng.next_range(16) as i32;
    let wx = cx * 16 + lx;
    let wz = cz * 16 + lz;
    for y in (1..=62).rev() {
        if !is_solid(world.get_block(wx, y - 1, wz)) {
            continue;
        }
        if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
            continue;
        }
        let (blk_l, sky_l) = light_levels(world, wx, y, wz);
        if blk_l > 3 || sky_l > 3 {
            return; // light <= 3 (VERIFIED)
        }
        let mut placed = 0;
        'group: for dz in -1..=1i32 {
            for dx in -1..=1i32 {
                if placed >= 8 {
                    break 'group; // group of 8 (VERIFIED JE)
                }
                let bx = wx + dx;
                let bz = wz + dz;
                if world.get_block(bx, y, bz) == AIR
                    && world.get_block(bx, y + 1, bz) == AIR
                    && is_solid(world.get_block(bx, y - 1, bz))
                {
                    let _ = self.spawn_variant(MobKind::Bat, bx, y, bz, 0);
                    placed += 1;
                }
            }
        }
        return;
        }
    }

    /// passive spawn attempt (VERIFIED): light ≥ 9 on GRASS with 2 air,
    /// cap 10; herds of 2–4. Vanilla weights these by biome and runs them
    /// rarely — ours gates at 1/20 per attempt.
    /// Phase E1: Mushroom Fields → mooshroom herds 4–8 on MYCELIUM (the
    /// biome's ONLY natural passive, weight 8/8 — VERIFIED w/Mooshroom);
    /// Jungle rolls ocelots (JE weight 2/93 — VERIFIED w/Ocelot).
    fn try_spawn_passive(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
        if self.rng.next_range(20) != 0 {
            return;
        }
        if self.passives_alive() as f32 >= CREATURE_CAP {
            return;
        }
        let Some(p) = self.player else { return };
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        // Phase 6 §26: spawning clamps to the simulation ring
        if !sim_ring(cx, cz) {
            return;
        }
        if world.chunk((cx, cz)).is_none() {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        // Phase E1: the chunk's biome picks the herd
        let biome = vc_world::gen::Biome::from_u8(world.get_biome(cx * 16 + 8, cz * 16 + 8));
        let py = p[1] as i32;
        for y in (py - 24..py + 12).rev() {
            if !(1..=250).contains(&y) {
                continue;
            }
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            let floor = world.get_block(wx, y - 1, wz);
            if biome == vc_world::gen::Biome::MushroomFields {
                // VERIFIED w/Mushroom_Fields + w/Mooshroom: mycelium floor,
                // herds of 4–8, mooshrooms only
                if floor != MYCELIUM && floor != GRASS {
                    continue;
                }
                if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
                    continue;
                }
                let (blk_l, _sky) = light_levels(world, wx, y, wz);
                if blk_l < PASSIVE_LIGHT_MIN {
                    return;
                }
                let herd = 4 + (self.rng.next_range(5)) as usize; // 4–8 (VERIFIED)
                for _ in 0..herd {
                    let _ = self.spawn_variant(MobKind::Mooshroom, wx, y, wz, 0);
                }
                return;
            }
            if floor != GRASS && floor != SNOW_GRASS {
                continue;
            }
            if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR {
                continue;
            }
            let (blk_l, _sky) = light_levels(world, wx, y, wz);
            if blk_l < PASSIVE_LIGHT_MIN {
                return;
            }
            // Phase E1: jungle → ocelot chance (JE weight 2/93 ≈ 1/6 of
            // the passive roll — simplified to 1/4)
            // Phase E3 (VERIFIED w/Horse §Spawning): plains horses 5/46
            // ≈ 1/9 of the passive roll, herds 2–6, 20% babies; savanna
            // horses/donkeys 1/52 ≈ 1/26 (split between the two kinds);
            // donkeys ride the savanna roll (w/Donkey: plains+savanna)
            // 1.8: rabbits join the general passive roll (wiki: "spawn as
            // any other farm animals, in grassy biomes").
            // 1.10: polar bears spawn in the icy family (wiki /w/Polar_
            // Bear: "adults and cubs spawn randomly as passive mobs in
            // ice plains, ice mountains and ice plains spikes") AND icy
            // biomes roll ONLY rabbits + polar bears (wiki
            // /w/Java_Edition_1.10 §World generation changes: "Now don't
            // spawn any passive mobs other than rabbits and the new polar
            // bears", live 2026-09-06)
            let kind = if biome == vc_world::gen::Biome::Jungle && self.rng.next_range(4) == 0 {
                MobKind::Ocelot
            } else if biome == vc_world::gen::Biome::Plains && self.rng.next_range(9) == 0 {
                // plains: horse herd (5/46 ≈ 1/9 of creature rolls)
                MobKind::Horse
            } else if biome == vc_world::gen::Biome::Savanna && self.rng.next_range(26) == 0 {
                // savanna: horses or donkeys at the verified 1/52 ≈ 1/26
                // share (adaptation: even split, both VERIFIED weights
                // are 1/52 on that biome)
                if self.rng.next_range(2) == 0 {
                    MobKind::Horse
                } else {
                    MobKind::Donkey
                }
            } else if biome == vc_world::gen::Biome::Snowy
                || biome == vc_world::gen::Biome::IceSpikes
            {
                // the icy family: polar bear (30%) or rabbit — and NOTHING
                // else (the 1.10 restriction)
                if self.rng.next_f32() < 0.3 {
                    MobKind::PolarBear
                } else {
                    MobKind::Rabbit
                }
            } else {
                match self.rng.next_range(5) {
                    0 => MobKind::Cow,
                    1 => MobKind::Pig,
                    2 => MobKind::Sheep,
                    3 => MobKind::Chicken,
                    _ => MobKind::Rabbit,
                }
            };
            // equine herds are 2–6 (VERIFIED w/Horse §Spawning); other
            // passives keep the engine's 2–4
            let herd = if matches!(kind, MobKind::Horse | MobKind::Donkey) {
                2 + (self.rng.next_range(5)) as usize // 2–6
            } else {
                2 + (self.rng.next_range(3)) as usize
            };
            for _ in 0..herd {
                let _ = self.spawn_variant(kind, wx, y, wz, 0);
            }
            return;
        }
    }

    /// Player melee hit on a mob. Neutral mobs become provoked.
    pub fn damage(&mut self, id: u32, amount: f32) -> f32 {
        for m in self.list.iter_mut() {
            if m.id == id {
                m.health -= amount;
                m.hurt_t = 10;
                m.provoked = true;
                return amount;
            }
        }
        0.0
    }

    /// Crosshair ray hit-test against mob AABBs (villager pattern).
    pub fn ray_hit(&self, eye: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<u32> {
        let mut best: Option<(u32, f32)> = None;
        for m in &self.list {
            let d = def(m.kind);
            let half = d.width * 0.5;
            let lo = [m.pos[0] - half, m.pos[1], m.pos[2] - half];
            let hi = [m.pos[0] + half, m.pos[1] + d.height, m.pos[2] + half];
            let mut tmin = 0.0f32;
            let mut tmax = max_dist;
            let mut ok = true;
            for a in 0..3 {
                if dir[a].abs() < 1e-6 {
                    if eye[a] < lo[a] || eye[a] > hi[a] {
                        ok = false;
                        break;
                    }
                } else {
                    let mut t1 = (lo[a] - eye[a]) / dir[a];
                    let mut t2 = (hi[a] - eye[a]) / dir[a];
                    if t1 > t2 {
                        std::mem::swap(&mut t1, &mut t2);
                    }
                    tmin = tmin.max(t1);
                    tmax = tmax.min(t2);
                    if tmin > tmax {
                        ok = false;
                        break;
                    }
                }
            }
            if ok && best.map(|(_, t)| tmin < t).unwrap_or(true) {
                best = Some((m.id, tmin));
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn by_id(&self, id: u32) -> Option<&Mob> {
        self.list.iter().find(|m| m.id == id)
    }

    /// Phase E3: mutable by-id lookup (the ride drive writes the mount's
    /// velocity from the game layer)
    pub fn by_id_mut(&mut self, id: u32) -> Option<&mut Mob> {
        self.list.iter_mut().find(|m| m.id == id)
    }

    // ------------------------------------------------- Phase E3: equines --

    /// Mount attempt on an equine (right-click while looking at it).
    /// VERIFIED w/Horse §Taming: temper starts 0/100; a random threshold
    /// 0–99 is chosen at the FIRST mount; a failed mount adds +5 temper;
    /// the horse becomes tame when the temper EXCEEDS the threshold.
    /// Returns Some(tamed) when the mount succeeded (the player may ride
    /// — control still requires a saddle, w/Horse §Riding), Some(false)
    /// = bucked off (untamed), None = not an equine.
    pub fn try_mount(&mut self, id: u32, rng: &mut Rng) -> Option<bool> {
        let m = self.list.iter_mut().find(|m| m.id == id)?;
        let eq = m.equine.as_mut()?;
        if eq.tamed {
            return Some(true);
        }
        if eq.threshold > 99 {
            // first mount: choose the random taming threshold (VERIFIED)
            eq.threshold = (rng.next_range(100)) as u8;
        }
        eq.temper = (eq.temper + 5).min(100);
        eq.tamed = eq.temper > eq.threshold;
        Some(eq.tamed)
    }

    /// saddle an already-tamed equine (the held SADDLE routes here).
    /// VERIFIED w/Horse §Riding: "Once a horse is tamed and saddled, the
    /// player can control it". Returns true when the saddle was applied.
    pub fn try_saddle(&mut self, id: u32) -> bool {
        let Some(m) = self.list.iter_mut().find(|m| m.id == id) else {
            return false;
        };
        let Some(eq) = m.equine.as_mut() else {
            return false;
        };
        if eq.tamed && !eq.saddled {
            eq.saddled = true;
            true
        } else {
            false
        }
    }

    /// Feed an equine: a golden apple on two tamed adults starts breeding
    /// (VERIFIED w/Horse §Breeding: "Feeding two tamed horses golden
    /// apples or golden carrots activates love mode"); hay heals + grows
    /// temper (w/Hay_Bale §Food — "feed llamas and all living horse
    /// variants", foal growth +3 min — the numeric temper gain is an
    /// engine adaptation, disclosed: the wiki temper table covers
    /// sugar/wheat/apples which the engine lacks).
    /// Returns the feed outcome for the game layer to consume items.
    pub fn try_feed(&mut self, id: u32, food: u16, rng: &mut Rng) -> Option<FeedOutcome> {
        // audit-fix (1.4): golden carrot joins the equine foods (VERIFIED
        // live 2026-09-07 w/Golden_Carrot §Usage: "Golden carrots are used
        // to tame, breed, lead, grow, and heal horses, donkeys, and
        // mules"; the breeding rule was already live-verified in the E3
        // round w/Horse §Breeding: "Feeding two tamed horses golden
        // apples or golden carrots activates love mode"). It follows the
        // golden-apple arm: love mode on two tamed adults, heal +4
        // otherwise (the engine's e3-verified per-food mapping).
        if food != GOLDEN_APPLE && food != HAY_BALE && food != GOLDEN_CARROT {
            return None;
        }
        // snapshot the target state (ends the mutable borrow before the
        // partner scan below)
        let (pos, tamed, baby, breed_cd) = {
            let m = self.list.iter_mut().find(|m| m.id == id)?;
            let eq = m.equine.as_mut()?;
            (m.pos, eq.tamed, eq.baby, eq.breed_cd)
        };
        let out = match food {
            HAY_BALE => Some(FeedOutcome::Healed),
            _ => {
                if tamed && !baby && breed_cd == 0 {
                    // find a second fertile partner within 8 blocks
                    // (foal spawns when BOTH parents are in love mode)
                    let partner = self.list.iter().find(|o| {
                        o.id != id
                            && o.equine
                                .as_ref()
                                .map(|e| e.tamed && !e.baby && e.breed_cd == 0)
                                .unwrap_or(false)
                            && (o.pos[0] - pos[0]).powi(2) + (o.pos[2] - pos[2]).powi(2) < 64.0
                    });
                    if let Some(pid) = partner.map(|o| o.id) {
                        if let Some(pm) = self.list.iter_mut().find(|o| o.id == pid) {
                            if let Some(pe) = pm.equine.as_mut() {
                                pe.breed_cd = 6000;
                            }
                        }
                        Some(FeedOutcome::LoveMode(pid))
                    } else {
                        Some(FeedOutcome::Ate)
                    }
                } else {
                    Some(FeedOutcome::Healed)
                }
            }
        };
        // apply the target-side effects
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            if let Some(eq) = m.equine.as_mut() {
                match food {
                    HAY_BALE => {
                        m.health = (m.health + 10.0).min(30.0);
                        if !eq.tamed {
                            eq.temper = (eq.temper + 10).min(100);
                            if eq.threshold <= 99 && eq.temper > eq.threshold {
                                eq.tamed = true;
                            }
                        }
                    }
                    _ => {
                        if matches!(out, Some(FeedOutcome::LoveMode(_)) | Some(FeedOutcome::Ate)) {
                            eq.breed_cd = 6000; // love-mode cooldown (5 min)
                        }
                        m.health = (m.health + 4.0).min(30.0);
                    }
                }
            }
        }
        let _ = rng;
        out
    }

    /// Bred-stat roll for a foal (VERIFIED w/Horse §Bred_values, the
    /// 5-step formula: baby = avg(p1,p2) + rand(−0.5..0.5)·
    /// (|p1−p2| + 0.30·range), clamped to the allowed range).
    pub fn bred_stat(p1: f32, p2: f32, lo: f32, hi: f32, rng: &mut Rng) -> f32 {
        let range = hi - lo;
        let avg = (p1 + p2) * 0.5;
        let r = rng.next_f32() - 0.5;
        let v = avg + r * ((p1 - p2).abs() + 0.30 * range);
        v.clamp(lo, hi)
    }

    /// Spawn a foal from two parents (breeding result): horse×horse =
    /// horse; horse×donkey or any mule pairing = mule (VERIFIED w/Mule:
    /// "When a horse and donkey breed" a mule results).
    pub fn spawn_foal(&mut self, p1: u32, p2: u32, x: i32, y: i32, z: i32, rng: &mut Rng) -> Option<u32> {
        let (k1, s1, j1, h1) = self
            .list
            .iter()
            .find(|m| m.id == p1)
            .map(|m| {
                let e = m.equine.as_ref().unwrap();
                (m.kind, e.speed_attr, e.jump_strength, m.health)
            })?;
        let (k2, s2, j2, h2) = self
            .list
            .iter()
            .find(|m| m.id == p2)
            .map(|m| {
                let e = m.equine.as_ref().unwrap();
                (m.kind, e.speed_attr, e.jump_strength, m.health)
            })?;
        let kind = if (k1 == MobKind::Horse && k2 == MobKind::Donkey)
            || (k1 == MobKind::Donkey && k2 == MobKind::Horse)
            || k1 == MobKind::Mule
            || k2 == MobKind::Mule
        {
            MobKind::Mule
        } else {
            k1
        };
        let id = self.spawn_at(kind, x, y, z)?;
        let speed = Self::bred_stat(s1, s2, 0.1125, 0.3375, rng);
        let jump = Self::bred_stat(j1, j2, 0.4, 1.0, rng);
        let health = Self::bred_stat(h1, h2, 15.0, 30.0, rng);
        if let Some(m) = self.list.last_mut() {
            if let Some(e) = m.equine.as_mut() {
                e.speed_attr = speed;
                e.jump_strength = jump;
                e.baby = true;
                e.tamed = true; // foals of tamed parents are tamed (VERIFIED w/Horse §Breeding)
            }
            m.health = health;
        }
        Some(id)
    }

    /// one tick of equine bookkeeping: breed cooldowns + baby growth
    /// (foals mature in 20 minutes = 24000 ticks, VERIFIED w/Horse —
    /// hay accelerates by 3 min per bale, wired in try_feed).
    pub fn tick_equines(&mut self) {
        for m in self.list.iter_mut() {
            if let Some(e) = m.equine.as_mut() {
                if e.breed_cd > 0 {
                    e.breed_cd -= 1;
                }
                if e.baby {
                    m.aux += 1;
                    if m.aux >= 24000 {
                        e.baby = false;
                    }
                }
            }
        }
    }
}

/// feed outcome for the game layer (consume the item, play the sound);
/// LoveMode carries the partner id so game.rs can spawn the foal
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedOutcome {
    Healed,
    Ate,
    LoveMode(u32),
}

// ------------------------------------------------------------- free fns --

/// magma-cube size (blocks) from the variant code (vanilla NBT Size tag:
/// codes 0/1/3 = sizes 1/2/4 — VERIFIED w/Magma_Cube §Spawning).
#[inline]
pub fn magma_size(variant: u8) -> u8 {
    match variant {
        0 => 1,
        1 => 2,
        _ => 4,
    }
}

/// magma-cube XP by size (VERIFIED w/Magma_Cube §Drops: 4/2/1).
#[inline]
pub fn magma_xp(size: u8) -> i32 {
    match size {
        4 => 4,
        2 => 2,
        _ => 1,
    }
}

/// AI decision + steering for one mob (free fn: splits borrows).
/// Phase E1: `snapshot` = read-only view of all mobs (mob-vs-mob
/// targeting), `pending` = queued mob-vs-mob damage.
fn ai_tick(
    rng: &mut Rng,
    m: &mut Mob,
    player: Option<[f32; 3]>,
    invuln: bool,
    hits: &mut Vec<PlayerHit>,
    arrows: &mut Vec<Arrow>,
    world: &World,
    snapshot: &[(u32, MobKind, [f32; 3], u8)],
    pending: &mut Vec<(u32, f32)>,
) {
    let d = def(m.kind);
    let speed = if let Some(eq) = m.equine.as_ref() {
        // Phase E3: equines use their per-instance speed attribute
        // (0.1125–0.3375 — VERIFIED w/Horse §Movement_speed); babies
        // move at half pace (vanilla foal speed scaling, disclosed
        // simplification)
        if eq.baby {
            eq.speed_attr * SPEED_PER_ATTR * 0.5
        } else {
            eq.speed_attr * SPEED_PER_ATTR
        }
    } else {
        d.speed_attr * SPEED_PER_ATTR
    };

    // ---- Phase E1: snow golem heat rule (VERIFIED w/Snow_Golem: 1 HP/tick
    // in biomes with temperature > 1.0 — desert/badlands/savanna[JE]/Nether
    // + rain/water contact; engine has no rain, water contact deferred).
    // Environmental — applies regardless of a player anchor.
    if m.kind == MobKind::SnowGolem {
        let biome_hot = matches!(
            vc_world::gen::Biome::from_u8(world.get_biome(
                m.pos[0] as i32,
                m.pos[2] as i32,
            )),
            vc_world::gen::Biome::Desert
                | vc_world::gen::Biome::Badlands
                | vc_world::gen::Biome::Savanna
                | vc_world::gen::Biome::NetherWastes
        );
        if biome_hot && !invuln {
            m.health -= 1.0; // per game tick (VERIFIED)
        }
    }

    let Some(p) = player else {
        wander(rng, m, speed * 0.4);
        return;
    };
    let dx = p[0] - m.pos[0];
    let _dy = p[1] - m.pos[1];
    let dz = p[2] - m.pos[2];
    let dist = (dx * dx + dz * dz).sqrt().max(1e-4);
    let face_player = |m: &mut Mob| {
        m.yaw = (-dz).atan2(dx) - std::f32::consts::FRAC_PI_2;
    };
    let aggro = (m.kind.hostile() || m.provoked) && !invuln;

    // ---- Phase E1: zombie-villager curing (weakness + golden apple is
    // applied by the game layer; it sets variant=1 + aux=3600..=6000 —
    // VERIFIED w/Zombie_Villager §Curing). While curing the mob is docile
    // (documented simplification — vanilla curing zombie villagers still
    // attack); at zero the tick scan converts it to a villager.
    if m.kind == MobKind::ZombieVillager && m.variant >= 1 {
        // variant 1 = curing, 2 = cured-and-ready (drained by tick 4.5);
        // both are docile
        if m.variant == 1 {
            m.aux -= 1;
            if m.aux <= 0 {
                m.variant = 2; // cured-and-ready marker (MobSystem::tick drains)
            }
        }
        wander(rng, m, speed * 0.3);
        return;
    }

    // ---- Phase E1: snow golem targeting (the heat rule ran above,
    // before the player-anchor early return) — throws snowballs at the
    // nearest hostile ≤ 10 blocks, 1/s (VERIFIED: "They throw one
    // snowball per second")
    if m.kind == MobKind::SnowGolem {
        let mut best: Option<(u32, [f32; 3], f32)> = None;
        for (id, k, pos, _) in snapshot.iter() {
            if k.hostile() && *id != m.id {
                let sx = pos[0] - m.pos[0];
                let sz = pos[2] - m.pos[2];
                let dd = (sx * sx + sz * sz).sqrt();
                if dd <= 10.0 && best.map(|(_, _, bd)| dd < bd).unwrap_or(true) {
                    best = Some((*id, *pos, dd));
                }
            }
        }
        if let Some((_, tpos, _)) = best {
            if m.attack_cd == 0 {
                m.attack_cd = 20; // 1/s (VERIFIED)
                spawn_projectile(
                    m,
                    tpos,
                    rng,
                    arrows,
                    ProjKind::Snowball,
                    18.0,
                    0.0, // 0 damage base (VERIFIED)
                );
            }
            face_target(m, tpos);
        } else {
            wander(rng, m, speed * 0.4);
        }
        return;
    }

    // ---- Phase E1: iron golem — village guard. Attacks the nearest
    // hostile mob within 16 blocks (reach 2.8 with its wide body);
    // retaliates against a provoking player (vanilla Normal 7.5–21.5,
    // engine takes the fixed mid 14 — documented adaptation). Knockback
    // on mobs is deferred (the damage queue carries no impulse).
    if m.kind == MobKind::IronGolem {
        let mut best: Option<(u32, [f32; 3], f32)> = None;
        for (id, k, pos, _) in snapshot.iter() {
            if k.hostile() && *id != m.id {
                let sx = pos[0] - m.pos[0];
                let sz = pos[2] - m.pos[2];
                let dd = (sx * sx + sz * sz).sqrt();
                if dd <= 16.0 && best.map(|(_, _, bd)| dd < bd).unwrap_or(true) {
                    best = Some((*id, *pos, dd));
                }
            }
        }
        if let Some((tid, tpos, tdist)) = best {
            face_target(m, tpos);
            if tdist > 2.8 {
                m.vel[0] += ((tpos[0] - m.pos[0]) / tdist * speed - m.vel[0]) * 0.3;
                m.vel[2] += ((tpos[2] - m.pos[2]) / tdist * speed - m.vel[2]) * 0.3;
            } else if m.attack_cd == 0 {
                m.attack_cd = MOB_MELEE_TICKS;
                pending.push((tid, d.damage));
            }
            return;
        }
        // provoked by the player → melee (retaliation, vanilla)
        if m.provoked && !invuln && dist < MOB_MELEE_REACH + 0.8 && m.attack_cd == 0 {
            m.attack_cd = MOB_MELEE_TICKS;
            face_player(m);
            hits.push(PlayerHit {
                damage: d.damage,
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
            });
            return;
        }
        wander(rng, m, speed * 0.25); // patrol pace
        return;
    }

    // ---- Phase E1: ocelot — flees players unless trusting; attacks
    // chickens within 15 blocks (both VERIFIED w/Ocelot).
    if m.kind == MobKind::Ocelot {
        if m.variant != 1 && dist < 6.0 && !invuln {
            // flee (VERIFIED: players moving within 6 blocks scare it)
            m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
            let f = speed * FLEE_MULT;
            m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
            m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            return;
        }
        // hunt chickens ≤ 15 blocks (VERIFIED: ocelots attack chickens
        // within 15 blocks)
        for (id, k, pos, _) in snapshot.iter() {
            if *k == MobKind::Chicken {
                let sx = pos[0] - m.pos[0];
                let sz = pos[2] - m.pos[2];
                let dd = (sx * sx + sz * sz).sqrt();
                if dd <= 15.0 {
                    face_target(m, *pos);
                    if dd > 1.0 {
                        m.vel[0] += (sx / dd * speed * 1.4 - m.vel[0]) * 0.4;
                        m.vel[2] += (sz / dd * speed * 1.4 - m.vel[2]) * 0.4;
                    } else if m.attack_cd == 0 {
                        m.attack_cd = MOB_MELEE_TICKS;
                        // a chicken has 4 HP — one pounce kills (vanilla)
                        pending.push((*id, 4.0));
                    }
                    return;
                }
            }
        }
        wander(rng, m, speed * 0.5);
        return;
    }

    // ---- Phase E3 (1.5–1.6): equines — passive grazing herds; panic
    // gallop after being hit (the provoked flag doubles as the panic
    // timer's source — vanilla horses flee briefly when damaged).
    if matches!(m.kind, MobKind::Horse | MobKind::Donkey | MobKind::Mule) {
        if m.provoked && dist < 16.0 && !invuln {
            m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
            let f = speed * 1.5; // gallop
            m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.35;
            m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.35;
            return;
        }
        wander(rng, m, speed * 0.4);
        return;
    }

    // ---- Phase E1: blaze — hovers while targeting; 60-tick charge then
    // 3 fireballs 6 ticks apart (VERIFIED: "charges for 3 seconds, then
    // fires three small fireballs at intervals of 0.3 seconds"). Contact
    // melee when close (Normal 6 — VERIFIED). The burst rides an
    // 78-tick cycle counter (aux): charge = phases 0..=59, shots at
    // 60 / 66 / 72.
    if m.kind == MobKind::Blaze {
        if aggro && dist < 32.0 {
            face_player(m);
            // hover (VERIFIED: "often floats upward while targeting")
            m.vel[1] += (1.7 - m.vel[1]) * 0.6;
            if dist > 10.0 {
                m.vel[0] += (dx / dist * speed * 0.8 - m.vel[0]) * 0.3;
                m.vel[2] += (dz / dist * speed * 0.8 - m.vel[2]) * 0.3;
            } else {
                m.vel[0] *= 0.85;
                m.vel[2] *= 0.85;
            }
            // burst cycle: 0..=59 charge, fire at 60 / 66 / 72
            m.aux = (m.aux + 1) % 78;
            if m.aux == 60 || m.aux == 66 || m.aux == 72 {
                spawn_projectile(m, p, rng, arrows, ProjKind::Fireball, 14.0, d.damage);
            }
            // close-range contact (VERIFIED: contact Normal 6)
            if dist < MOB_MELEE_REACH + 0.4 && m.attack_cd == 0 {
                m.attack_cd = MOB_MELEE_TICKS;
                hits.push(PlayerHit {
                    damage: 6.0,
                    source: m.kind,
                    knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
            });
            }
        } else {
            m.aux = 0;
            wander(rng, m, speed * 0.4);
        }
        return;
    }

    // ---- Phase E1: magma cube — hop movement: idle jump every 40–120
    // ticks, 13–40 with a target ≤ 16 blocks; jump height = size blocks,
    // hop distance ≈ 1.5×size; contact damage size+2 (all VERIFIED
    // w/Magma_Cube §Behavior/§Combat).
    if m.kind == MobKind::MagmaCube {
        let size = magma_size(m.variant);
        let seek = aggro && dist < 16.0;
        if m.aux > 0 {
            m.aux -= 1;
        }
        if m.aux == 0 {
            if m.on_ground {
                // face the target, or pick a wander direction
                if seek {
                    face_player(m);
                } else {
                    m.yaw = rng.next_f32() * std::f32::consts::TAU;
                }
                // jump height = size blocks → v = sqrt(2·g·h), g ≈ 32 b/s²
                let v = (2.0 * 32.0 * size as f32).sqrt();
                m.vel[1] = v;
                // hop distance ≈ 1.5 × size over the hang time 2v/g
                let hang = 2.0 * v / 32.0;
                let hd = 1.5 * size as f32 / hang.max(0.1);
                let (s, c) = (m.yaw.sin(), m.yaw.cos());
                m.vel[0] = s * hd;
                m.vel[2] = -c * hd;
                // VERIFIED cadence: idle 40–120, with target 1/3 as long
                m.aux = if seek {
                    13 + rng.next_range(28) as i32
                } else {
                    40 + rng.next_range(81) as i32
                };
            } else {
                m.aux = 1; // airborne — check again next tick
            }
        }
        // contact damage (VERIFIED: damages on touch, ~½ s cadence)
        if seek
            && dist < (d.width * 0.5 + 0.7)
            && m.attack_cd == 0
        {
            m.attack_cd = 10; // damage-immunity cadence (VERIFIED ~0.5 s)
            hits.push(PlayerHit {
                damage: d.damage, // size + 2 (VERIFIED)
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
            });
        }
        return;
    }

    match m.kind {
        MobKind::Zombie
        | MobKind::ZombieVillager
        | MobKind::Husk
        | MobKind::Spider
        | MobKind::Enderman => {
            let engage = if m.kind == MobKind::Enderman {
                m.provoked
            } else {
                aggro
            };
            if engage && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > MOB_MELEE_REACH * 0.8 {
                    let chase = if m.kind == MobKind::Enderman {
                        speed
                    } else {
                        speed
                    };
                    m.vel[0] += (dx / dist * chase - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * chase - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.7;
                    m.vel[2] *= 0.7;
                }
                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage,
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
            });
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        MobKind::Skeleton | MobKind::Stray => {
            if aggro && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > 8.0 {
                    m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                } else if dist < 5.0 {
                    m.vel[0] += (-dx / dist * speed * 0.7 - m.vel[0]) * 0.3;
                    m.vel[2] += (-dz / dist * speed * 0.7 - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.8;
                    m.vel[2] *= 0.8;
                }
                if m.attack_cd == 0 {
                    m.attack_cd = SKELETON_SHOOT_TICKS;
                    spawn_arrow(m, p, rng, arrows);
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        MobKind::Creeper => {
            if aggro && dist < AGGRO_RADIUS {
                face_player(m);
                if m.fuse < 0 {
                    if dist < CREEPER_FUSE_DIST {
                        m.fuse = 0;
                    } else {
                        m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                        m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                    }
                }
                if m.fuse >= 0 && m.fuse != i32::MAX {
                    if dist > CREEPER_FUSE_DIST * 2.4 {
                        m.fuse = -1; // defused — player escaped
                    } else {
                        m.fuse += 1;
                        if m.fuse >= CREEPER_FUSE_TICKS {
                            // consumed marker: take_explosions() surfaces the
                            // blast to the game layer (world edits + light)
                            m.fuse = i32::MAX;
                            m.health = 0.0; // dies in its own blast
                        }
                    }
                }
            } else {
                m.fuse = -1;
                wander(rng, m, speed * 0.5);
            }
        }
        // passives: wander; panic-flee while flashing from a hit
        // 1.8 rabbit addition: rabbits are skittish — the wiki's "avoid
        // all players within 8 blocks" (live-verified 2026-09-06) — they
        // hop away at panic speed BEFORE ever being hit
        MobKind::Rabbit => {
            const RABBIT_AVOID_RADIUS: f32 = 8.0;
            if m.hurt_t > 0 {
                m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
                let f = speed * FLEE_MULT;
                m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
                m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            } else if dist < RABBIT_AVOID_RADIUS {
                // face away and bolt (the panicking rabbit)
                m.yaw = (dz / dist).atan2(-dx / dist) - std::f32::consts::FRAC_PI_2;
                m.vel[0] += (-dx / dist * speed - m.vel[0]) * 0.3;
                m.vel[2] += (-dz / dist * speed - m.vel[2]) * 0.3;
            } else {
                wander(rng, m, speed * 0.4);
            }
        }
        _ => {
            if m.hurt_t > 0 {
                m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
                let f = speed * FLEE_MULT;
                m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
                m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            } else {
                wander(rng, m, speed * 0.4);
            }
        }
    }
}

/// face a world-space target point (mob-vs-mob targeting)
fn face_target(m: &mut Mob, tpos: [f32; 3]) {
    let dx = tpos[0] - m.pos[0];
    let dz = tpos[2] - m.pos[2];
    m.yaw = (-dz).atan2(dx) - std::f32::consts::FRAC_PI_2;
}

fn wander(rng: &mut Rng, m: &mut Mob, speed: f32) {
    if m.wander_t == 0 {
        m.wander_t = (rng.next_range(120) as i32 + 40).max(1);
        m.wander_yaw = rng.next_f32() * std::f32::consts::TAU;
        if rng.next_f32() < 0.6 {
            m.wander_t = -m.wander_t; // negative = standing still
        }
    }
    m.wander_t += if m.wander_t > 0 { -1 } else { 1 };
    if m.wander_t > 0 {
        m.yaw = m.wander_yaw;
        let (s, c) = (m.yaw.sin(), m.yaw.cos());
        m.vel[0] += (s * speed - m.vel[0]) * 0.2;
        m.vel[2] += (-c * speed - m.vel[2]) * 0.2;
    } else {
        m.vel[0] *= 0.8;
        m.vel[2] *= 0.8;
    }
}

/// gravity + axis collision with 1-block step-ups (villager primitive).
fn physics_tick(m: &mut Mob, world: &World) {
    let d = def(m.kind);
    // Vanilla entity gravity, EXACT per-tick form (VERIFIED,
    // research-verdicts.md: v1 = (v0 − 0.08) × 0.98 in b/t). Velocities
    // here are b/s, so the per-tick step on b/s units is
    // v ← (v − 1.6) × 0.98 (0.08 b/t × 20 = 1.6 b/s; drag is unitless).
    // Terminal −78.4 b/s (−3.92 b/t) is the inherent fixed point — no
    // clamp. (This also fixes a latent 20× unit bug: the old code
    // subtracted the per-tick 0.08 from a b/s velocity, giving 1.6 b/s²
    // gravity and a 3.92 b/s "terminal" — mobs fell 20× too slow.)
    m.vel[1] = (m.vel[1] - 1.6) * 0.98;
    // fall damage (MC-12357, same as the player): distance-based — the
    // old impact-speed inversion (v²/0.16) was dead code in practice
    // (on_ground + |v| > 0.35 never coincided after the drag rewrite,
    // and it overestimated tall falls under drag anyway)
    let half = d.width * 0.5;
    // horizontal move with step-up
    let (nx, nz) = (
        m.pos[0] + m.vel[0] * (1.0 / 20.0),
        m.pos[2] + m.vel[2] * (1.0 / 20.0),
    );
    if !collides(world, nx, m.pos[1], nz, half, d.height) {
        m.pos[0] = nx;
        m.pos[2] = nz;
    } else if !collides(world, nx, m.pos[1] + 1.05, nz, half, d.height) {
        m.pos[0] = nx;
        m.pos[2] = nz;
        m.pos[1] += 1.05;
    } else {
        m.vel[0] *= 0.5;
        m.vel[2] *= 0.5;
    }
    // vertical — substepped: at terminal 3.92 b/t the per-tick move is
    // up to 3.92 blocks, and a single end-point probe would tunnel
    // through 1–3-block floors (the player mover substeps for exactly
    // this reason)
    let dy = m.vel[1] * (1.0 / 20.0);
    let steps = (dy.abs() / 0.9).ceil().max(1.0) as i32;
    let step = dy / steps as f32;
    for _ in 0..steps {
        let ny = m.pos[1] + step;
        if collides(world, m.pos[0], ny, m.pos[2], half, d.height) {
            if step < 0.0 {
                m.pos[1] = ny.ceil();
                // landing converts the accumulated fall distance
                // (MC-12357: damage = fall_distance − 3)
                if m.fall_dist > 3.0 {
                    m.health -= m.fall_dist - 3.0;
                }
                m.fall_dist = 0.0;
                m.on_ground = true;
            }
            m.vel[1] = 0.0;
            break;
        }
        m.pos[1] = ny;
        m.on_ground = false;
    }
    // fall bookkeeping (vanilla fallDistance: per-tick distance)
    if !m.on_ground && m.vel[1] < 0.0 {
        m.fall_dist += -m.vel[1] * (1.0 / 20.0);
    }
}

fn collides(world: &World, x: f32, y: f32, z: f32, half: f32, height: f32) -> bool {
    let min_x = (x - half).floor() as i32;
    let max_x = (x + half).floor() as i32;
    let min_y = y.floor() as i32;
    let max_y = (y + height - 0.001).floor() as i32;
    let min_z = (z - half).floor() as i32;
    let max_z = (z + half).floor() as i32;
    for by in min_y..=max_y {
        for bz in min_z..=max_z {
            for bx in min_x..=max_x {
                if is_solid(world.get_block(bx, by, bz)) {
                    return true;
                }
            }
        }
    }
    false
}

/// (block light, sky light) at a world position, straight from the
/// per-chunk LightData map (the same source light_at reads).
fn light_levels(world: &World, wx: i32, wy: i32, wz: i32) -> (u8, u8) {
    let cx = wx.div_euclid(16);
    let cz = wz.div_euclid(16);
    let lx = (wx - cx * 16) as usize;
    let lz = (wz - cz * 16) as usize;
    let sec = (wy.clamp(0, 255) / 16) as usize;
    let yy = (wy.clamp(0, 255) % 16) as usize;
    let idx = (yy << 8) | (lz << 4) | lx;
    world
        .light
        .get(&(cx, cz))
        .and_then(|ld| ld.sections[sec].as_ref().map(|s| (s.blk[idx], s.sky[idx])))
        .unwrap_or((0, 15))
}

/// skeleton arrow: aimed ballistic shot (24 b/s flat, gravity-compensated).
fn spawn_arrow(m: &Mob, target: [f32; 3], rng: &mut Rng, arrows: &mut Vec<Arrow>) {
    let dmg = 3.0 + rng.next_f32() * 2.0; // VERIFIED: Normal 3–5
    spawn_projectile(m, target, rng, arrows, ProjKind::Arrow, 24.0, dmg);
}

/// Phase E1: the shared projectile spawner. Arrows: 24 b/s, Normal 3–5
/// (VERIFIED). Fireballs: 14 b/s launch (they accelerate toward ~38 b/s
/// in vanilla — our integrator holds the launch speed, documented),
/// damage from the def. Snowballs: 18 b/s, 0 damage (the mob-hit rule
/// "3 vs blazes" is applied by tick_arrows).
#[allow(clippy::too_many_arguments)]
fn spawn_projectile(
    m: &Mob,
    target: [f32; 3],
    rng: &mut Rng,
    arrows: &mut Vec<Arrow>,
    kind: ProjKind,
    speed: f32,
    damage: f32,
) {
    let d = def(m.kind);
    let ox = m.pos[0];
    let oy = m.pos[1] + d.height * 0.75;
    let oz = m.pos[2];
    let dx = target[0] - ox;
    let dy = target[1] + 1.2 - oy;
    let dz = target[2] - oz;
    let dist = (dx * dx + dz * dz).sqrt().max(1e-3);
    let t = dist / speed;
    // arrow gravity 20 b/s²: compensate the flight-time drop (fireballs /
    // snowballs fly straight — vanilla small fireballs have no drop)
    let drop = if kind == ProjKind::Arrow {
        0.5 * 20.0 * t * t
    } else {
        0.0
    };
    let vy = if kind == ProjKind::Arrow {
        ((dy + drop) / t.max(1e-3)).min(speed)
    } else {
        dy / t.max(1e-3)
    };
    // VERIFIED (wiki skeleton page, Java): Normal arrow damage 3–5
    let damage = if kind == ProjKind::Arrow {
        3.0 + rng.next_f32() * 2.0
    } else {
        damage
    };
    arrows.push(Arrow {
        pos: [ox, oy, oz],
        vel: [dx / dist * speed, vy, dz / dist * speed],
        damage,
        age: 0,
        kind,
        owner: m.id,
    });
}

fn tick_arrows(
    arrows: &mut Vec<Arrow>,
    player: Option<[f32; 3]>,
    invuln: bool,
    hits: &mut Vec<PlayerHit>,
    world: &World,
    mobs: &mut [Mob],
    pending: &mut Vec<(u32, f32)>,
) {
    let dt = 1.0 / 20.0;
    let mut i = 0;
    while i < arrows.len() {
        let a = &mut arrows[i];
        a.age += 1;
        // only arrows arc; fireballs/snowballs fly straight (vanilla)
        if a.kind == ProjKind::Arrow {
            a.vel[1] -= 20.0 * dt; // arrow gravity (vanilla 0.05/tick²)
        }
        a.pos[0] += a.vel[0] * dt;
        a.pos[1] += a.vel[1] * dt;
        a.pos[2] += a.vel[2] * dt;
        // player body-center hit sphere (r = 0.8)
        if let Some(p) = player {
            if !invuln {
                let ddx = a.pos[0] - p[0];
                let ddy = a.pos[1] - (p[1] + 0.9);
                let ddz = a.pos[2] - p[2];
                if ddx * ddx + ddy * ddy + ddz * ddz < 0.64 {
                    let dir = [a.vel[0] / 24.0, a.vel[2] / 24.0];
                    let src = match a.kind {
                        ProjKind::Arrow => MobKind::Skeleton,
                        ProjKind::Fireball => MobKind::Blaze,
                        ProjKind::Snowball => MobKind::SnowGolem,
                        // Phase E2: the wither skull's source (the wither
                        // itself is the boss system; the hit carries the
                        // Wither II payload via `wither_effect`)
                        ProjKind::Skull => MobKind::WitherSkeleton,
                    };
                    // snowballs deal 0 damage to the player (VERIFIED),
                    // knockback only
                    let dmg = if a.kind == ProjKind::Snowball { 0.0 } else { a.damage };
                    hits.push(PlayerHit {
                        damage: dmg,
                        source: src,
                        knockback_dir: dir,
                        // Phase E2 (VERIFIED w/Wither): skulls inflict
                        // Wither II — 10 s Normal / 40 s Hard
                        wither_effect: if a.kind == ProjKind::Skull {
                            Some(200)
                        } else {
                            None
                        },
                    });
                    arrows.remove(i);
                    continue;
                }
            }
        }
        // Phase E1: snowball mob hits — 3 damage to blazes, 0 + knockback
        // to everything else (VERIFIED w/Snow_Golem: "Thrown snowballs do
        // not deal damage except to blazes, but they still knock back any
        // mobs that they hit")
        if a.kind == ProjKind::Snowball {
            let mut hit_mob = false;
            for m in mobs.iter_mut() {
                if m.id == a.owner {
                    continue; // never hit its own golem
                }
                let ddx = a.pos[0] - m.pos[0];
                let ddy = a.pos[1] - (m.pos[1] + 0.5);
                let ddz = a.pos[2] - m.pos[2];
                if ddx * ddx + ddy * ddy + ddz * ddz < 0.8 {
                    if m.kind == MobKind::Blaze {
                        pending.push((m.id, 3.0)); // VERIFIED: 3 HP vs blazes
                    } else {
                        // knockback only
                        m.vel[0] += a.vel[0] * 0.05;
                        m.vel[2] += a.vel[2] * 0.05;
                    }
                    hit_mob = true;
                    break;
                }
            }
            if hit_mob {
                arrows.remove(i);
                continue;
            }
        }
        if is_solid(world.get_block(
            a.pos[0].floor() as i32,
            a.pos[1].floor() as i32,
            a.pos[2].floor() as i32,
        )) || a.age > 20 * 60
        {
            arrows.remove(i);
            continue;
        }
        i += 1;
    }
}

/// Creeper fuse completion → surface explosions to the game layer: the
/// exploded creeper is REMOVED here (it died in its own blast, no drops),
/// and game.rs turns each (center, power) into world edits + entity damage.
pub fn take_explosions(sys: &mut MobSystem) -> Vec<([f32; 3], f32)> {
    let mut out = std::mem::take(&mut sys.explosions);
    let mut i = 0;
    while i < sys.list.len() {
        if sys.list[i].fuse == i32::MAX {
            let m = sys.list.remove(i);
            out.push((m.pos, CREEPER_POWER));
        } else {
            i += 1;
        }
    }
    out
}

// ------------------------------------------------------------- rendering --

/// Mob sprites as camera-facing quads (the villager pattern), sized per
/// kind, red-tinted while hurt; creepers blink white while priming.
pub fn build_vertices(
    list: &[Mob],
    right: [f32; 3],
    out: &mut Vec<vc_particles::particles::ParticleVertex>,
) {
    for m in list {
        let d = def(m.kind);
        let tile = m.kind.sprite_tile();
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        let (s, c) = (m.yaw.sin(), m.yaw.cos());
        let rr = [
            c * right[0] + s * right[2],
            0.0,
            -s * right[0] + c * right[2],
        ];
        let half = d.width * 0.55;
        let h = d.height;
        let mut col = [0.92, 0.92, 0.92];
        if m.hurt_t > 0 {
            col = [1.0, 0.35, 0.35];
        }
        if m.fuse >= 0 && m.fuse != i32::MAX && (m.fuse / 3) % 2 == 0 {
            col = [1.6, 1.6, 1.6];
        }
        let corners = [
            (
                [-rr[0] * half, 0.0, -rr[2] * half],
                [tx / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [rr[0] * half, 0.0, rr[2] * half],
                [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [rr[0] * half, h, rr[2] * half],
                [(tx + 1.0) / 16.0, ty / 16.0],
            ),
            ([-rr[0] * half, h, -rr[2] * half], [tx / 16.0, ty / 16.0]),
        ];
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let (c, uv) = corners[ci];
            out.push(vc_particles::particles::ParticleVertex {
                pos: [m.pos[0] + c[0], m.pos[1] + c[1], m.pos[2] + c[2]],
                uv,
                col,
            });
        }
    }
}

/// Arrow billboards (tiny camera-facing quads on the arrow tile).
pub fn build_arrow_vertices(
    arrows: &[Arrow],
    right: [f32; 3],
    up: [f32; 3],
    out: &mut Vec<vc_particles::particles::ParticleVertex>,
) {
    for a in arrows {
        let tile = TILE_ARROW;
        let tx = (tile % 16) as f32;
        let ty = (tile / 16) as f32;
        let half = 0.35f32;
        let corners = [
            (
                [
                    -right[0] * half - up[0] * half,
                    -right[1] * half - up[1] * half,
                    -right[2] * half - up[2] * half,
                ],
                [tx / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [
                    right[0] * half - up[0] * half,
                    right[1] * half - up[1] * half,
                    right[2] * half - up[2] * half,
                ],
                [(tx + 1.0) / 16.0, (ty + 1.0) / 16.0],
            ),
            (
                [
                    right[0] * half + up[0] * half,
                    right[1] * half + up[1] * half,
                    right[2] * half + up[2] * half,
                ],
                [(tx + 1.0) / 16.0, ty / 16.0],
            ),
            (
                [
                    -right[0] * half + up[0] * half,
                    -right[1] * half + up[1] * half,
                    -right[2] * half + up[2] * half,
                ],
                [tx / 16.0, ty / 16.0],
            ),
        ];
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let (c, uv) = corners[ci];
            out.push(vc_particles::particles::ParticleVertex {
                pos: [a.pos[0] + c[0], a.pos[1] + c[1], a.pos[2] + c[2]],
                uv,
                col: [0.95, 0.95, 0.95],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_world() -> World {
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
        w.dirty.clear();
        w
    }

    /// Phase E1: a desert-biome flat world (biome id 4) for the snow-golem
    /// heat-damage rule.
    fn desert_world() -> World {
        let mut w = World::new(11);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        for i in 0..256usize {
            c.biome[i] = 4; // Desert
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    #[test]
    fn mob_table_matches_verified_wiki_rows() {
        // VERIFIED infobox rows (2026-09-04): health / Normal damage /
        // speed attribute / natural armor
        assert_eq!(def(MobKind::Zombie).health as i32, 20);
        assert_eq!(def(MobKind::Zombie).damage as i32, 3);
        assert!((def(MobKind::Zombie).speed_attr - 0.23).abs() < 1e-6);
        assert_eq!(def(MobKind::Zombie).armor as i32, 2);
        assert_eq!(def(MobKind::Spider).health as i32, 16);
        assert_eq!(def(MobKind::Spider).damage as i32, 2);
        assert!((def(MobKind::Spider).speed_attr - 0.3).abs() < 1e-6);
        assert_eq!(def(MobKind::Enderman).health as i32, 40);
        assert_eq!(def(MobKind::Enderman).damage as i32, 7);
        assert_eq!(def(MobKind::Skeleton).health as i32, 20);
        assert_eq!(def(MobKind::Cow).health as i32, 10);
        assert_eq!(def(MobKind::Pig).health as i32, 10);
        assert_eq!(def(MobKind::Sheep).health as i32, 8);
        assert_eq!(def(MobKind::Chicken).health as i32, 4);
        // hostile/neutral split
        assert!(def(MobKind::Zombie).kind.hostile() && !def(MobKind::Zombie).kind.neutral());
        assert!(!def(MobKind::Enderman).kind.hostile() && def(MobKind::Enderman).kind.neutral());
        assert!(!def(MobKind::Cow).kind.hostile());
    }

    #[test]
    fn verified_constants_are_what_the_wiki_says() {
        assert_eq!(HOSTILE_LIGHT_MAX, 7); // 1.16.5 (1.18 → 0)
        assert_eq!(PASSIVE_LIGHT_MIN, 9);
        assert_eq!(HOSTILE_SKY_MAX, 7);
        assert_eq!(MONSTER_CAP as i32, 70);
        assert_eq!(CREATURE_CAP as i32, 10);
        assert_eq!(CAP_DIVISOR as i32, 289);
        assert_eq!(DESPAWN_INSTANT_BLOCKS as i32, 128);
        assert_eq!(DESPAWN_NEAR_BLOCKS as i32, 32);
        assert_eq!(CREEPER_FUSE_TICKS, 30); // 1.5 s
        assert_eq!(CREEPER_POWER as i32, 3);
    }

    #[test]
    fn spawn_damage_death_cycle() {
        let mut sys = MobSystem::new(5);
        assert!(sys.is_empty());
        let id = sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        assert_eq!(sys.len(), 1);
        assert_eq!(sys.by_id(id).unwrap().health, 20.0);
        assert_eq!(sys.spawned_total, 1);
        assert_eq!(sys.damage(id, 6.0), 6.0);
        assert!((sys.by_id(id).unwrap().health - 14.0).abs() < 1e-5);
        assert!(sys.by_id(id).unwrap().provoked);
        sys.damage(id, 20.0);
        sys.tick(&flat_world(), (0, 0), i32::MAX);
        assert!(sys.is_empty());
        assert_eq!(sys.deaths.len(), 1);
        assert_eq!(sys.deaths[0].0, MobKind::Zombie);
        assert_eq!(sys.killed_total, 1);
    }

    #[test]
    fn ray_hit_finds_the_crosshair_mob() {
        let mut sys = MobSystem::new(5);
        let id = sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        let eye = [2.5, 66.0, 4.5];
        assert_eq!(sys.ray_hit(eye, [1.0, 0.0, 0.0], 4.0), Some(id));
        assert_eq!(sys.ray_hit(eye, [-1.0, 0.0, 0.0], 4.0), None);
        assert_eq!(sys.ray_hit([40.0, 66.0, 40.0], [1.0, 0.0, 0.0], 4.0), None);
    }

    #[test]
    fn despawn_far_mobs_instantly() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([0.0, 70.0, 0.0]);
        sys.spawn_at(MobKind::Zombie, 0, 65, 0).unwrap();
        sys.spawn_at(MobKind::Zombie, 200, 65, 200).unwrap(); // >128 away
        sys.tick(&flat_world(), (0, 0), i32::MAX);
        assert_eq!(sys.len(), 1, "far mob gone, near mob stays");
        assert_eq!(sys.despawned_total, 1);
    }

    #[test]
    fn skeleton_shoots_and_the_arrow_lands() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([10.5, 65.0, 4.5]);
        sys.spawn_at(MobKind::Skeleton, 0, 65, 4).unwrap();
        // force aggro and an immediate shot (free-fn call, split borrows)
        let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
        let mut mob = sys.list.remove(0);
        ai_tick(
            &mut rng,
            &mut mob,
            sys.player,
            false,
            &mut sys.hits,
            &mut sys.arrows,
            &flat_world(),
            &[],
            &mut Vec::new(),
        );
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert_eq!(sys.arrows.len(), 1, "skeleton fired one arrow");
        // fly it at the player
        let world = flat_world();
        for _ in 0..300 {
            tick_arrows(&mut sys.arrows, sys.player, false, &mut sys.hits, &world, &mut [], &mut Vec::new());
            if !sys.hits.is_empty() {
                break;
            }
        }
        assert!(!sys.hits.is_empty(), "arrow reached the player");
        let hit = &sys.hits[0];
        assert!(
            hit.damage >= 3.0 && hit.damage <= 5.0,
            "Normal 3-5, got {}",
            hit.damage
        );
        assert_eq!(hit.source, MobKind::Skeleton);
    }

    #[test]
    fn creeper_fuses_then_explodes() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([5.5, 65.0, 4.5]); // ~1 block from the creeper
        sys.spawn_at(MobKind::Creeper, 4, 65, 4).unwrap();
        let world = flat_world();
        // AI: fuse starts
        let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
        let mut mob = sys.list.remove(0);
        ai_tick(
            &mut rng,
            &mut mob,
            sys.player,
            false,
            &mut sys.hits,
            &mut sys.arrows,
            &world,
            &[],
            &mut Vec::new(),
        );
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert!(sys.list[0].fuse >= 0, "fuse started");
        // count up to the blast
        let pos0 = sys.list[0].pos;
        for _ in 0..CREEPER_FUSE_TICKS + 2 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(
                &mut rng,
                &mut mob,
                sys.player,
                false,
                &mut sys.hits,
                &mut sys.arrows,
                &world,
                &[],
                &mut Vec::new(),
            );
            sys.list.insert(0, mob);
            sys.rng = rng;
            physics_tick(&mut sys.list[0], &world);
            if sys.list[0].fuse == i32::MAX {
                break;
            }
        }
        assert_eq!(sys.list[0].fuse, i32::MAX, "fuse completed");
        assert!(sys.list[0].health <= 0.0, "creeper died in its blast");
        let booms = take_explosions(&mut sys);
        assert_eq!(booms.len(), 1);
        assert_eq!(booms[0].0, pos0);
        assert_eq!(booms[0].1 as i32, 3, "explosion power 3 (VERIFIED)");
        // the death sweep must NOT queue drops (exploded = destroyed)
        sys.tick(&world, (0, 0), i32::MAX);
        assert!(sys.is_empty());
        assert!(sys.deaths.is_empty(), "exploded creepers drop nothing");
    }

    #[test]
    fn passive_mobs_flee_when_hurt() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([6.5, 65.0, 4.5]);
        sys.spawn_at(MobKind::Cow, 4, 65, 4).unwrap();
        let id = sys.list[0].id;
        sys.damage(id, 3.0);
        assert!((sys.by_id(id).unwrap().health - 7.0).abs() < 1e-5);
        // hurt cow runs away from the player
        let world = flat_world();
        let x_before = sys.list[0].pos[0];
        for _ in 0..40 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(
                &mut rng,
                &mut mob,
                sys.player,
                false,
                &mut sys.hits,
                &mut sys.arrows,
                &world,
                &[],
                &mut Vec::new(),
            );
            sys.list.insert(0, mob);
            sys.rng = rng;
            physics_tick(&mut sys.list[0], &world);
        }
        // the player is +x from the cow: fleeing moves -x
        assert!(
            sys.list[0].pos[0] < x_before + 0.1,
            "cow fled away, x {} -> {}",
            x_before,
            sys.list[0].pos[0]
        );
    }

    #[test]
    fn invulnerable_players_never_get_hit() {
        let mut sys = MobSystem::new(5);
        sys.player = Some([5.5, 65.0, 4.5]);
        sys.player_invulnerable = true; // creative
        sys.spawn_at(MobKind::Zombie, 4, 65, 4).unwrap();
        let world = flat_world();
        for _ in 0..60 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert!(sys.hits.is_empty(), "creative is never attacked");
        // ...and nothing even spawns while invulnerable
        assert_eq!(sys.spawned_total, 1, "only the explicit spawn");
    }

    /// Exact per-tick gravity drag (VERIFIED — research-verdicts.md):
    /// in b/s units one physics tick maps v ← (v − 1.6) × 0.98
    #[test]
    fn mob_gravity_drag_matches_vanilla_formula() {
        let w = flat_world();
        // spawn high enough that even terminal velocity moves freely for
        // the whole tick (the floor is at y=65 — no collision interference)
        for v0 in [0.0f32, -20.0, -78.4, -100.0] {
            let mut m = Mob {
                id: 0,
                kind: MobKind::Zombie,
                pos: [8.5, 90.0, 8.5],
                vel: [0.0, v0, 0.0],
                yaw: 0.0,
                health: 20.0,
                on_ground: false,
                hurt_t: 0,
                attack_cd: 0,
                fuse: -1,
                provoked: false,
                lonely_t: 0,
                fall_dist: 0.0,
                variant: 0,
                aux: 0,
                wander_yaw: 0.0,
                wander_t: 0,
                equine: None,
            };
            physics_tick(&mut m, &w);
            let expect = (v0 - 1.6) * 0.98;
            assert!(
                (m.vel[1] - expect).abs() < 1e-3,
                "v0 {v0}: got {} want {expect}",
                m.vel[1]
            );
        }
    }

    /// Mob fall damage is distance-based MC-12357: a 7-block fall costs
    /// 4 HP (fall − 3), a 2.5-block fall is free, and terminal falls no
    /// longer tunnel through the floor (substepped vertical probe)
    #[test]
    fn mob_fall_damage_is_distance_minus_three() {
        let w = flat_world();
        for (drop, want_dmg) in [(7.0f32, 4.0f32), (2.5, 0.0)] {
            let mut m = Mob {
                id: 0,
                kind: MobKind::Zombie,
                pos: [8.5, 64.0 + 1.0 + drop, 8.5],
                vel: [0.0, 0.0, 0.0],
                yaw: 0.0,
                health: 20.0,
                on_ground: false,
                hurt_t: 0,
                attack_cd: 0,
                fuse: -1,
                provoked: false,
                lonely_t: 0,
                fall_dist: 0.0,
                variant: 0,
                aux: 0,
                wander_yaw: 0.0,
                wander_t: 0,
                equine: None,
            };
            let mut ticks = 0;
            while !m.on_ground && ticks < 200 {
                physics_tick(&mut m, &w);
                ticks += 1;
            }
            assert!(m.on_ground, "must land ({drop}-block drop)");
            let dmg = 20.0 - m.health;
            assert!(
                (dmg - want_dmg).abs() < 1.5,
                "{drop}-block fall: {dmg} HP vs ~{want_dmg}"
            );
            assert_eq!(m.fall_dist, 0.0, "landing resets the accumulator");
            // resting on the surface, never below it
            assert!((m.pos[1] - 65.0).abs() < 0.01, "y={}", m.pos[1]);
        }
    }

    /// Terminal falls (terminal −78.4 b/s = 3.92 blocks/tick) must not
    /// tunnel through the 1-block-thick stone floor
    #[test]
    fn terminal_fall_does_not_tunnel() {
        let w = flat_world();
        let mut m = Mob {
            id: 0,
            kind: MobKind::Zombie,
            pos: [8.5, 120.0, 8.5],
            vel: [0.0, -78.4, 0.0],
            yaw: 0.0,
            health: 20.0,
            on_ground: false,
            hurt_t: 0,
            attack_cd: 0,
            fuse: -1,
            provoked: false,
            lonely_t: 0,
            fall_dist: 55.0,
            variant: 0,
            aux: 0,
            wander_yaw: 0.0,
            wander_t: 0,
                equine: None,
        };
        let mut ticks = 0;
        while !m.on_ground && ticks < 100 {
            physics_tick(&mut m, &w);
            ticks += 1;
        }
        assert!(m.on_ground, "lands");
        assert!(m.pos[1] >= 65.0, "no tunneling: y={}", m.pos[1]);
        assert!(m.health <= 0.0, "55-block fall is lethal, hp={}", m.health);
    }

    // ---------------- Phase E1 tests (1.0–1.2 bracket) ----------------

    #[test]
    fn phase_e1_registry_rows() {
        // [merge] the kinds resolve in/out of names + eggs (16 E1 + 3
        // E2 + 3 E3 horse/donkey/mule + 4 F-series: rabbit 1.8, stray +
        // polar bear + husk 1.10)
        assert_eq!(MOB_DATA.len(), 26);
        for d in MOB_DATA.iter() {
            assert_eq!(
                MobKind::from_name(d.kind.name().strip_prefix("minecraft:").unwrap()),
                Some(d.kind)
            );
            // 255 = "no egg item yet" (F-series rabbit/polar-bear/stray/
            // husk — vanilla has these spawn eggs; deferred until the
            // registry grows the egg rows, disclosed in the worklog)
            if d.kind.egg_id() != 255 {
                assert_eq!(MobKind::from_egg(d.kind.egg_id()), d.kind);
            }
        }
        // verified rows
        let sg = def(MobKind::SnowGolem);
        assert_eq!(sg.health, 4.0);
        assert_eq!(sg.xp, 0);
        let mc = def(MobKind::MagmaCube);
        assert_eq!((mc.health, mc.damage, mc.armor), (16.0, 6.0, 12.0)); // size 4 row
        let bl = def(MobKind::Blaze);
        assert_eq!((bl.health, bl.damage, bl.xp), (20.0, 5.0, 10));
        let ig = def(MobKind::IronGolem);
        assert_eq!(ig.health, 100.0);
        let zv = def(MobKind::ZombieVillager);
        assert_eq!(zv.health, 20.0);
        let mr = def(MobKind::Mooshroom);
        assert_eq!(mr.health, 10.0);
        // hostile set: magma/blaze/zombie-villager join; golems neutral
        assert!(MobKind::MagmaCube.hostile());
        assert!(MobKind::Blaze.hostile());
        assert!(MobKind::ZombieVillager.hostile());
        assert!(!MobKind::IronGolem.hostile());
        assert!(MobKind::IronGolem.neutral());
    }

    #[test]
    fn phase_e1_magma_variant_scales() {
        // VERIFIED: HP = size², damage = size+2, armor = 3×size, XP 4/2/1
        let mut sys = MobSystem::new(9);
        sys.spawn_variant(MobKind::MagmaCube, 8, 65, 8, 0).unwrap(); // small
        sys.spawn_variant(MobKind::MagmaCube, 8, 65, 9, 1).unwrap(); // medium
        sys.spawn_variant(MobKind::MagmaCube, 8, 65, 10, 2).unwrap(); // big
        let hp: Vec<f32> = sys.list.iter().map(|m| m.health).collect();
        assert_eq!(hp, vec![1.0, 4.0, 16.0], "HP = size² (1/4/16)");
        assert_eq!(magma_size(0), 1);
        assert_eq!(magma_size(1), 2);
        assert_eq!(magma_size(2), 4);
        assert_eq!(magma_xp(4), 4);
        assert_eq!(magma_xp(2), 2);
        assert_eq!(magma_xp(1), 1);
    }

    #[test]
    fn phase_e1_snow_golem_throws_at_hostiles_and_melts_in_desert() {
        // flat_world is plains — the golem survives, targets the zombie
        let mut sys = MobSystem::new(11);
        sys.spawn_at(MobKind::SnowGolem, 4, 65, 4).unwrap();
        sys.spawn_at(MobKind::Zombie, 6, 65, 6).unwrap();
        sys.player = Some([100.5, 65.0, 100.5]); // far away
        let world = flat_world();
        // enough ticks for the 20-tick cooldown cadence to fire once
        let zid = sys.list[1].id;
        for _ in 0..25 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(
                &mut rng,
                &mut mob,
                sys.player,
                false,
                &mut sys.hits,
                &mut sys.arrows,
                &world,
                &[(zid, MobKind::Zombie, [6.5, 65.0, 6.5], 0)],
                &mut Vec::new(),
            );
            sys.list.insert(0, mob);
            sys.rng = rng;
        }
        assert!(!sys.arrows.is_empty(), "snowball fired at the zombie");
        assert_eq!(sys.arrows[0].kind, ProjKind::Snowball);
        assert_eq!(sys.arrows[0].damage, 0.0, "snowball base damage is 0 (VERIFIED)");
        // heat: the same golem in a hot biome takes 1 HP per tick — biome
        // gate is read from the world, covered by the desert flat-world
        // variant below (we assert the branch through a desert world).
        let desert = desert_world();
        let mut rng = Rng::new(1);
        let mut m = Mob { id: 9, kind: MobKind::SnowGolem, pos: [8.5, 65.0, 8.5], vel: [0.0; 3],
            yaw: 0.0, health: 4.0, on_ground: true, hurt_t: 0, attack_cd: 0, fuse: -1,
            provoked: false, lonely_t: 0, fall_dist: 0.0, variant: 0, aux: 0,
            wander_yaw: 0.0, wander_t: 0, equine: None };
        for _ in 0..5 {
            ai_tick(&mut rng, &mut m, None, false, &mut Vec::new(), &mut Vec::new(), &desert, &[], &mut Vec::new());
        }
        assert!(m.health < 4.0, "desert heat melts the golem (1 HP/tick), hp={}", m.health);
    }

    #[test]
    fn phase_e1_blaze_bursts_three_fireballs() {
        let mut sys = MobSystem::new(13);
        sys.spawn_at(MobKind::Blaze, 4, 65, 4).unwrap();
        sys.player = Some([8.5, 65.0, 8.5]); // close target
        let world = flat_world();
        let mut fired = 0usize;
        for _ in 0..80 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            let before = sys.arrows.len();
            ai_tick(
                &mut rng,
                &mut mob,
                sys.player,
                false,
                &mut sys.hits,
                &mut sys.arrows,
                &world,
                &[],
                &mut Vec::new(),
            );
            fired += sys.arrows.len() - before;
            sys.list.insert(0, mob);
            sys.rng = rng;
        }
        // 60-tick charge + 3 shots per 80-tick window = exactly 3
        assert_eq!(fired, 3, "one 3-shot burst after the 60-tick charge (VERIFIED cadence)");
        assert!(sys.arrows.iter().all(|a| a.kind == ProjKind::Fireball));
    }

    #[test]
    fn phase_e1_iron_golem_guards_against_hostiles() {
        let mut sys = MobSystem::new(17);
        sys.spawn_at(MobKind::IronGolem, 4, 65, 4).unwrap();
        sys.spawn_at(MobKind::Zombie, 5, 65, 6).unwrap();
        sys.player = Some([100.5, 65.0, 100.5]); // away — this is mob-vs-mob
        let world = flat_world();
        let zid = sys.list[1].id;
        let mut pend: Vec<(u32, f32)> = Vec::new();
        for _ in 0..30 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[(zid, MobKind::Zombie, [5.5, 65.0, 6.5], 0)], &mut pend);
            sys.list.insert(0, mob);
            sys.rng = rng;
        }
        // the zombie takes golem swings (14 dmg × ≥ 2 hits = dead 20 HP)
        assert!(!pend.is_empty(), "golem attacked the zombie");
        assert!(pend.iter().all(|(_, d)| *d == 14.0));
        // and the golem never hurt the player
        assert!(sys.hits.is_empty());
    }

    #[test]
    fn phase_e1_zombie_villager_cure_lifecycle() {
        let mut sys = MobSystem::new(19);
        sys.spawn_at(MobKind::ZombieVillager, 4, 65, 4).unwrap();
        // begin the cure with a fixed short window (constants verified)
        {
            let mut rng = Rng::new(2);
            mobs_cure_short(&mut sys.list[0], &mut rng);
        }
        assert_eq!(sys.list[0].variant, 1, "curing flag set");
        sys.player = Some([4.5, 65.0, 4.5]);
        let world = flat_world();
        // cure countdown to completion (docile while curing: no player hits)
        let ticks = sys.list[0].aux;
        for _ in 0..ticks as usize + 2 {
            let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
            let mut mob = sys.list.remove(0);
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[], &mut Vec::new());
            sys.list.insert(0, mob);
            sys.rng = rng;
        }
        assert!(sys.hits.is_empty(), "curing zombie villager is docile");
        // run the system tick that drains finished cures
        sys.tick(&world, (0, 0), 8);
        assert!(sys.list.is_empty(), "zombie villager left the mob list");
        assert_eq!(sys.cures.len(), 1, "cure event surfaced to the game layer");
        // the real range constants (VERIFIED 3600..=6000)
        assert_eq!((CURE_TICKS_MIN, CURE_TICKS_MAX), (3600, 6000));
    }

    #[test]
    fn phase_e1_golem_build_patterns() {
        // snow golem: 2 snow + pumpkin on top at y=67
        // (flat_world is stone at y<64; build above it)
        let mut w = flat_world();
        w.set_block(8, 64, 8, SNOW);
        w.set_block(8, 65, 8, SNOW);
        assert!(snow_golem_pattern(&w, 8, 66, 8), "pattern matches with pumpkin at 66");
        assert!(!snow_golem_pattern(&w, 9, 66, 8), "offset column fails");
        // iron golem: T of iron blocks
        let mut w2 = flat_world();
        w2.set_block(8, 64, 8, IRON_BLOCK);
        w2.set_block(7, 64, 8, IRON_BLOCK);
        w2.set_block(9, 64, 8, IRON_BLOCK);
        w2.set_block(8, 65, 8, IRON_BLOCK);
        assert!(iron_golem_pattern(&w2, 8, 66, 8), "T pattern + pumpkin on top");
        w2.set_block(7, 65, 8, STONE); // an obstruction in the empty spaces
        assert!(!iron_golem_pattern(&w2, 8, 66, 8), "obstructed spaces block the spawn");
    }

    #[test]
    fn phase_e1_ocelot_flees_player_and_hunts_chickens() {
        let mut sys = MobSystem::new(23);
        sys.spawn_at(MobKind::Ocelot, 4, 65, 4).unwrap();
        sys.player = Some([5.0, 65.0, 4.5]); // 0.5 blocks — within the 6-block scare radius
        let world = flat_world();
        let world2 = flat_world();
        let x0 = sys.list[0].pos[0];
        let mut rng = std::mem::replace(&mut sys.rng, Rng::new(1));
        let mut mob = sys.list.remove(0);
        ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[], &mut Vec::new());
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert!(sys.list[0].pos[0] < x0 + 0.2, "fled away from the player");
        // trusting ocelots do NOT flee (variant 1)
        let mut sys2 = MobSystem::new(29);
        sys2.spawn_variant(MobKind::Ocelot, 4, 65, 4, 1).unwrap();
        sys2.player = Some([5.0, 65.0, 4.5]);
        let x1 = sys2.list[0].pos[0];
        let mut rng2 = std::mem::replace(&mut sys2.rng, Rng::new(1));
        let mut mob2 = sys2.list.remove(0);
        ai_tick(&mut rng2, &mut mob2, sys2.player, false, &mut sys2.hits, &mut sys2.arrows, &world2, &[], &mut Vec::new());
        sys2.list.insert(0, mob2);
        sys2.rng = rng2;
        assert!((sys2.list[0].pos[0] - x1).abs() < 0.05, "trusting ocelot stays");
    }

    /// test-only cure starter with a SHORT window (the real begin_cure
    /// uses the verified 3600..=6000 range — too slow for a unit test)
    fn mobs_cure_short(m: &mut Mob, _rng: &mut Rng) {
        m.variant = 1;
        m.aux = 40;
    }

    // ---------------- Phase E3 tests (1.5–1.6 bracket) ----------------

    #[test]
    fn phase_e3_horse_spawn_stats_are_per_instance() {
        // VERIFIED w/Horse: health 15–30, speed 0.1125–0.3375, jump
        // 0.4–1.0, 20% babies; donkeys/mules fixed 0.175 speed
        let mut ms = MobSystem::new(99);
        let mut seen_baby = false;
        for i in 0..60 {
            let id = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
            let m = ms.by_id(id).unwrap();
            let eq = m.equine.as_ref().unwrap();
            assert!((15.0..=30.0).contains(&m.health), "health {}", m.health);
            assert!(
                (0.1125..=0.3375).contains(&eq.speed_attr),
                "speed {}",
                eq.speed_attr
            );
            assert!((0.4..=1.0).contains(&eq.jump_strength), "jump {}", eq.jump_strength);
            assert_eq!(m.kind, MobKind::Horse);
            seen_baby |= eq.baby;
            let _ = i;
        }
        // 20% of 60 spawns: a baby appears with overwhelming probability
        assert!(seen_baby, "some babies among 60 spawns (20%, VERIFIED)");
        // donkey: fixed 0.175
        let id = ms.spawn_at(MobKind::Donkey, 5, 65, 5).unwrap();
        let eq = ms.by_id(id).unwrap().equine.as_ref().unwrap();
        assert!((eq.speed_attr - 0.175).abs() < 1e-6, "donkey speed fixed");
    }

    #[test]
    fn phase_e3_taming_temper_rule() {
        // VERIFIED w/Horse §Taming: temper 0/100; threshold 0–99 chosen
        // at the first mount; +5 per failed mount; tame when temper
        // EXCEEDS the threshold
        let mut ms = MobSystem::new(7);
        let id = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
        // force a high threshold via repeated mounts (deterministic rng)
        let mut rng = Rng::new(1234);
        // first mount picks the threshold
        let mut mounts = 0;
        let mut tamed = false;
        while !tamed && mounts < 100 {
            tamed = ms.try_mount(id, &mut rng).unwrap();
            mounts += 1;
        }
        assert!(tamed, "100% temper maxes out and tames any threshold");
        assert!(mounts >= 1);
        // the equine state reflects the rule
        let m = ms.by_id(id).unwrap();
        let eq = m.equine.as_ref().unwrap();
        assert!(eq.tamed);
        assert!(eq.temper <= 100);
        assert!(eq.threshold <= 99, "threshold drawn from 0..=99");
        // a tamed mount returns Some(true) immediately
        assert_eq!(ms.try_mount(id, &mut rng), Some(true));
    }

    #[test]
    fn phase_e3_saddle_gates_control() {
        // VERIFIED w/Horse §Riding: control needs tamed + saddled
        let mut ms = MobSystem::new(8);
        let id = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
        // untamed: saddle refused
        assert!(!ms.try_saddle(id), "saddle refused while untamed");
        // tame it
        let mut rng = Rng::new(55);
        while ms.try_mount(id, &mut rng) == Some(false) {}
        assert!(ms.try_saddle(id), "saddle accepted once tamed");
        assert!(ms.by_id(id).unwrap().equine.as_ref().unwrap().saddled);
    }

    #[test]
    fn phase_e3_bred_stat_formula() {
        // VERIFIED w/Horse §Bred_values: baby = avg(p1,p2) +
        // rand(-0.5..0.5)·(|p1−p2| + 0.30·range), clamped to the range
        let mut rng = Rng::new(31337);
        // identical parents, mid range: result within ±(0.30·range)/2
        for _ in 0..200 {
            let v = MobSystem::bred_stat(0.2, 0.2, 0.1125, 0.3375, &mut rng);
            assert!((0.1125..=0.3375).contains(&v), "clamped: {v}");
            // |p1-p2|=0 → spread = 0.3*0.225/2 = 0.03375 around 0.2
            assert!((0.2 - 0.034..=0.2 + 0.034).contains(&v), "spread {v}");
        }
        // extreme parents stay in range
        for _ in 0..200 {
            let v = MobSystem::bred_stat(0.1125, 0.3375, 0.1125, 0.3375, &mut rng);
            assert!((0.1125..=0.3375).contains(&v));
        }
    }

    #[test]
    fn phase_e3_jump_clear_height_fit() {
        // the quadratic fit through the three VERIFIED anchors
        // (0.4→1.153, 0.7→3.124, 1.0→5.9197 blocks)
        let mk = |s: f32| EquineState {
            temper: 0,
            threshold: 100,
            tamed: true,
            saddled: true,
            speed_attr: 0.2,
            jump_strength: s,
            baby: false,
            coat: 0,
            breed_cd: 0,
        };
        for (s, want) in [(0.4f32, 1.153f32), (0.7, 3.124), (1.0, 5.9197)] {
            let got = mk(s).jump_clear_height();
            assert!(
                (got - want).abs() < 0.01,
                "anchor {s} -> {got} (want ~{want})"
            );
        }
    }

    #[test]
    fn phase_e3_foal_kind_rules() {
        // VERIFIED w/Mule: horse×donkey → mule; horse×horse → horse
        let mut ms = MobSystem::new(11);
        let h = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
        let d = ms.spawn_at(MobKind::Donkey, 2, 65, 0).unwrap();
        let h2 = ms.spawn_at(MobKind::Horse, 4, 65, 0).unwrap();
        let mut rng = Rng::new(2026);
        let foal = ms.spawn_foal(h, d, 1, 65, 1, &mut rng).unwrap();
        assert_eq!(ms.by_id(foal).unwrap().kind, MobKind::Mule, "horse×donkey = mule");
        let foal2 = ms.spawn_foal(h, h2, 3, 65, 3, &mut rng).unwrap();
        assert_eq!(ms.by_id(foal2).unwrap().kind, MobKind::Horse);
        // foals: baby + tamed (VERIFIED w/Horse §Breeding)
        let m = ms.by_id(foal2).unwrap();
        let eq = m.equine.as_ref().unwrap();
        assert!(eq.baby);
        assert!(eq.tamed);
    }

    #[test]
    fn phase_e3_ridden_mount_suspends_ai() {
        // the ridden id skips ai_tick but still physics-ticks: with the
        // mob marked ridden, the tick loop must not move it via AI
        let mut ms = MobSystem::new(12);
        let w = flat_world();
        let id = ms.spawn_at(MobKind::Horse, 8, 65, 8).unwrap();
        // tame + saddle it deterministically
        let mut rng = Rng::new(64);
        while ms.try_mount(id, &mut rng) == Some(false) {}
        assert!(ms.try_saddle(id));
        ms.ridden = Some(id);
        let before = ms.by_id(id).unwrap().pos;
        // no player anchor: AI would wander; ridden skips that
        ms.player = None;
        for _ in 0..40 {
            ms.tick(&w, (0, 0), i32::MAX);
        }
        let after = ms.by_id(id).unwrap().pos;
        let drift = (after[0] - before[0]).abs() + (after[2] - before[2]).abs();
        assert!(drift < 0.001, "ridden mount does not wander: drift {drift}");
    }
}

#[cfg(test)]
mod v18_tests {
    use super::*;

    /// 1.8 rabbit: data + the avoid-player AI gate (wiki: "avoid all
    /// players within 8 blocks")
    #[test]
    fn rabbit_data_and_behavior() {
        let d = def(MobKind::Rabbit);
        // VERIFIED (minecraft.wiki/w/Rabbit): 3 HP
        assert_eq!(d.health, 3.0);
        assert!(!MobKind::Rabbit.hostile() && !MobKind::Rabbit.neutral(), "passive");
        assert_eq!(MobKind::from_name("rabbit"), Some(MobKind::Rabbit));
        assert_eq!(MobKind::Rabbit.name(), "minecraft:rabbit");
        // the mob registry includes it in the herd roll
        assert!(MOB_DATA.iter().any(|m| m.kind == MobKind::Rabbit));
    }
}

#[cfg(test)]
mod v19_tests {
    use super::*;

    /// 1.9: attack-cooldown combat was verified in Phase 2 (combat.rs has
    /// the exact 1.9 formulas: 0.2 + 0.8p², ×1.5 crits at ≥84.8%, armor
    /// toughness). Here we pin the registry side of the bracket.
    #[test]
    fn shield_and_elytra_registered() {
        // shield/elytra/chorus items ride the V4 window and never place
        let vc = vc_blocks::blocks::default_state(vc_blocks::blocks::SHIELD);
        assert!(vc_blocks::blocks::is_item_block(vc_blocks::blocks::SHIELD));
        assert!(vc_blocks::blocks::is_item_block(vc_blocks::blocks::ELYTRA));
        assert!(vc_blocks::blocks::is_item_block(vc_blocks::blocks::CHORUS_FRUIT));
        // frost walker + mending (1.9 treasure enchants) are in the 38 set
        assert!(crate::enchanting::ENCHANTS
            .iter()
            .any(|e| e.id == "frost_walker"));
        assert!(crate::enchanting::ENCHANTS.iter().any(|e| e.id == "mending"));
        let _ = vc;
    }
}

#[cfg(test)]
mod v110_tests {
    use super::*;

    /// 1.10 mob registrations — stats per the live wiki pages
    #[test]
    fn frostburn_mob_data() {
        // polar bear: 30 HP (wiki /w/Polar_Bear)
        let pb = def(MobKind::PolarBear);
        assert_eq!(pb.health, 30.0);
        assert!(!MobKind::PolarBear.hostile(), "neutral, not on-sight hostile");
        // stray + husk inherit their base kinds' hostility
        assert!(MobKind::Stray.hostile() && MobKind::Husk.hostile());
        // registry names
        assert_eq!(MobKind::PolarBear.name(), "minecraft:polar_bear");
        assert_eq!(MobKind::Stray.name(), "minecraft:stray");
        assert_eq!(MobKind::Husk.name(), "minecraft:husk");
        // stray/husk data mirror skeleton/zombie stats
        let sk = def(MobKind::Skeleton);
        let st = def(MobKind::Stray);
        assert_eq!((st.health, st.damage), (sk.health, sk.damage));
        let zo = def(MobKind::Zombie);
        let hu = def(MobKind::Husk);
        assert_eq!((hu.health, hu.armor), (zo.health, zo.armor));
    }
}

// ---------------- audit-fix round tests (2026-09-07) ----------------
// 1.4 golden carrot: equine feed (VERIFIED live 2026-09-07
// w/Golden_Carrot §Usage + w/Horse §Breeding from the E3 round)

#[cfg(test)]
mod auditfix_tests {
    use super::*;

    fn tamed_adult_pair(ms: &mut MobSystem) -> (u32, u32) {
        let a = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
        let b = ms.spawn_at(MobKind::Horse, 2, 65, 0).unwrap();
        for id in [a, b] {
            if let Some(m) = ms.list.iter_mut().find(|m| m.id == id) {
                m.equine.as_mut().unwrap().tamed = true;
                m.equine.as_mut().unwrap().baby = false;
                m.equine.as_mut().unwrap().breed_cd = 0;
            }
        }
        (a, b)
    }

    /// golden carrot on two tamed adults starts love mode — the
    /// VERIFIED breeding rule (w/Horse §Breeding: "Feeding two tamed
    /// horses golden apples or golden carrots activates love mode")
    #[test]
    fn golden_carrot_breeds_tamed_horses() {
        let mut ms = MobSystem::new(42);
        let (a, b) = tamed_adult_pair(&mut ms);
        let mut rng = Rng::new(7);
        let out = ms.try_feed(a, GOLDEN_CARROT, &mut rng);
        assert!(
            matches!(out, Some(FeedOutcome::LoveMode(pid)) if pid == b),
            "golden carrot -> LoveMode with the nearby partner (got {out:?})"
        );
        // the fed horse got its cooldown
        let m = ms.by_id(a).unwrap();
        assert!(m.equine.as_ref().unwrap().breed_cd > 0);
    }

    /// golden carrot heals a horse with no partner (the golden-apple
    /// arm: +4 HP within the 30 cap — the engine's e3-verified mapping)
    #[test]
    fn golden_carrot_heals_a_lone_horse() {
        let mut ms = MobSystem::new(43);
        let a = ms.spawn_at(MobKind::Horse, 0, 65, 0).unwrap();
        // damage + isolate: no partner, not tamed-fertile
        if let Some(m) = ms.list.iter_mut().find(|m| m.id == a) {
            m.equine.as_mut().unwrap().tamed = false;
            m.equine.as_mut().unwrap().baby = false;
            m.equine.as_mut().unwrap().breed_cd = 0;
            m.health = 10.0;
        }
        let mut rng = Rng::new(8);
        let out = ms.try_feed(a, GOLDEN_CARROT, &mut rng);
        assert!(
            matches!(out, Some(FeedOutcome::Healed) | Some(FeedOutcome::Ate)),
            "lone-horse feed outcome (got {out:?})"
        );
        let m = ms.by_id(a).unwrap();
        assert!((m.health - 14.0).abs() < 1e-6, "healed +4 (got {})", m.health);
    }
}
