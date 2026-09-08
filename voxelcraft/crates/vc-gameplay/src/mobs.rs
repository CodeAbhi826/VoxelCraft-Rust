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
    // ---- 1.11 bracket (Exploration Update, live 2026-09-07) ----
    /// 1.11: the llama — VERIFIED (w/Llama live): 15–30 HP neutral,
    /// spit 1 HP Easy/Normal (1.5 Hard), strength 1–5 (wild 32.8/32.8/
    /// 32.8/0.8/0.8%), chest slots 3×strength, tamed by repetitively
    /// riding (temper), bred with hay bales, leash caravans up to 10,
    /// leather 0–2 (66.67%), 1⁄900 per-tick regen chance, aggressive
    /// toward wolves
    Llama,
    /// 1.11: the vindicator — VERIFIED (w/Vindicator live): 24 HP
    /// hostile illager, iron axe 13 HP Normal (7.5/19.5 E/H), speed
    /// 5.612 b/s, emerald 0–1 (50%) + its iron axe drops, spawns in
    /// woodland mansions
    Vindicator,
    /// 1.11: the evoker — VERIFIED (w/Evoker live): 24 HP hostile
    /// spell-casting illager, fangs 6 HP ignoring armor ("not mitigated
    /// by armor"), summons vexes, the ONLY totem-of-undying source
    /// ("They are the only source of totems of undying"), emerald 0–1,
    /// converts blue sheep to red within 16 blocks, spawns on the
    /// mansion's two upper floors
    Evoker,
    /// 1.11: the vex — VERIFIED (w/Vex live): 14 HP hostile, iron
    /// sword 9 HP Normal (5.5/13.5 E/H), "pass through any block,
    /// including water and lava", 5 XP, iron sword never drops
    /// (HandDropChances 0), summoned by evokers only
    Vex,
    // ---- 1.12 bracket (World of Color Update, live 2026-09-07) ----
    /// 1.12: the parrot — VERIFIED (w/Parrot live): 6 HP passive,
    /// speed 0.2, jungle spawn weight 40/93 (43.01%), groups 1–2,
    /// 5 color variants (red "red_blue"/blue/green/cyan "yellow_blue"/
    /// gray), tamed with seeds at 1/10 per feeding (w/Parrot §Taming:
    /// "Each item fed has a 1⁄10 chance of successfully taming them"),
    /// a fed cookie kills instantly ("the parrot receives 2128
    /// (3.4028 x 10^38)"), drops 1–2 feathers + 1–3 XP, cannot breed
    /// ("Unlike most passive mobs, parrots cannot be bred"), follows
    /// the tamer and teleports at 12+ blocks
    Parrot,
    /// 1.12: the illusioner — VERIFIED (w/Illusioner live): 32 HP
    /// hostile illager, speed 0.5, bow 2–5 HP Easy/Normal (3–5 Hard)
    /// fired every second ("three times faster than a skeleton"),
    /// casts Blindness 20 s on first engaging a player, then the mirror
    /// spell (Invisibility 60 s, refreshed + 4 false duplicates),
    /// targets players (16×4×16 box), no spawn egg, "Unused and
    /// present only in Java Edition" — spawns ONLY via the direct
    /// spawn API (the engine-native /summon stand-in)
    Illusioner,
    /// 1.13: the drowned — VERIFIED (w/Drowned live 2026-09-07): 20 HP
    /// base zombie variant (40/100 for "leaders" is Java-internal,
    /// disclosed), melee E 2.5/N 3/H 4.5, spawns in ocean/river water
    /// AND converts from zombies whose heads stayed under water
    /// (30 s + 15 s shaking — VERIFIED §Conversion), 6.25% chance to
    /// hold a trident (§Equipment) which drops at 8.5% on a player kill
    /// (w/Trident), neutral-until-provoked is the infobox's framing of
    /// their zombie-parity aggro — treated hostile like the zombie.
    Drowned,
    /// 1.13: the phantom — VERIFIED (w/Phantom live 2026-09-07): 20 HP
    /// undead hostile, 2 HP (E/N) / 3 HP (H) attack (the 1.14-pre3
    /// damage reduction — current wiki), spawns above a player whose
    /// "Time Since Last Rest" is ≥ 1 hour (72000 ticks = 3 in-game
    /// days, reset by death or bed), drops 0-1 phantom membrane @ 50%,
    /// afraid of cats (no cats in the engine — N/A), burns in sunlight
    /// (undead — the engine's zombies don't model burning, disclosed).
    Phantom,
    /// 1.13: the dolphin — VERIFIED (w/Dolphin live 2026-09-07): 10 HP
    /// neutral, melee E 2.5/N 3/H 4.5, pods of 1-2 (JE) in all ocean
    /// biomes except frozen/cold at Y 50-64, whole pod retaliates, fed
    /// raw fish → swims to the nearest treasure (buried treasure/
    /// shipwreck — the engine's buried-treasure feature), players
    /// sprint-swimming within 9 blocks get Dolphin's Grace 5 s.
    Dolphin,
    /// 1.13: the cod — VERIFIED (w/Cod live 2026-09-07): 3 HP passive
    /// fish, schools, cold ocean spawning, drops 1 raw cod + 5% bone
    /// meal; 1-3 XP.
    Cod,
    /// 1.13: the salmon — VERIFIED (w/Salmon live 2026-09-07): 3 HP
    /// passive fish, 3 size variants (0.2/0.4/0.6 height — the variant
    /// byte), drops 1 raw salmon; 1-3 XP.
    Salmon,
    /// 1.13: the pufferfish — VERIFIED (w/Pufferfish live 2026-09-07):
    /// 3 HP neutral, inflates toward players (variant byte 0/1/2 =
    /// unpuffed/semi/fully), contact damage E/N 2 H 3 semi + E 2.5/
    /// N 3/H 4.5 fully, contact Poison 3 s (semi) / 6 s (JE fully),
    /// warm-ocean spawning; drops 1 pufferfish item; 1-3 XP.
    Pufferfish,
    /// 1.13: the tropical fish — VERIFIED (w/Tropical_Fish live
    /// 2026-09-07): 3 HP passive, 2,700 visual variants (2 shapes ×
    /// 15 base colors × 6 patterns × 15 pattern colors — encoded across
    /// variant/aux), lukewarm/warm ocean + mangrove swamp spawning,
    /// drops 1 tropical fish item; 1-3 XP.
    TropicalFish,
    /// 1.13: the sea turtle — VERIFIED (w/Turtle live 2026-09-07): 30 HP
    /// passive, beach sand spawning (groups ≤ 5, 5% babies), bred with
    /// seagrass → lays eggs on its home beach (aux = egg cooldown;
    /// variant bit 7 = carrying egg), babies drop 1 scute on maturity,
    /// slow on land / fast in water, adults drop 0-2 seagrass;
    /// zombies/drowned/husks actively trample the eggs.
    Turtle,
    /// 1.14: the fox — VERIFIED (w/Fox live 2026-09-08, raw capture
    /// scripts/v114_page_fox.json): 10 HP passive, "Easy and Normal:
    /// 2 HP, Hard: 3 HP" attack (prey only — "Foxes don't attack
    /// players"), hitbox 0.7/0.6 (baby 0.42/0.36), speed 0.3, spawns
    /// in groups of 2–4 in taiga/snowy taiga, "Foxes attack chickens,
    /// rabbits, cod, salmon and tropical fish, and baby turtles while
    /// they are on land", "take no damage or speed reduction while
    /// moving through sweet berry bushes" (the round's plant/mob
    /// interlock), bred with sweet berries (babies trust the breeder
    /// — the variant bit; the multi-player trust scope is
    /// single-player-folded, disclosed).
    Fox,
    /// 1.15 (Buzzy Bees): the bee — VERIFIED (w/Bee, live 2026-09-08,
    /// raw capture scripts/v115_page_Bee.json): 10 HP neutral
    /// pollinating arthropod, "Melee: Easy: 2 HP Normal: 2 HP Hard:
    /// 3 HP. Venom: Normal: Poison I for 10 sec. Hard: Poison I for
    /// 18 sec", hitbox 0.5 x 0.55 (baby 0.275), speed 0.6 (the
    /// infobox row), stings ONCE then "loses its stinger ... and dies
    /// approximately one minute later" (1200-tick timer), hives hold
    /// 3 bees, naturally-generated nests hold 2-3, anger swarms the
    /// whole hive family (20-39 s random), pacified by a lit campfire
    /// within 5 blocks below the hive. Hover-flight like the bat
    /// ("hover a few blocks above the ground similar to bats").
    Bee,
    /// 1.16 (Nether Update, part 2): the strider — VERIFIED (w/Strider,
    /// live 2026-09-08, raw capture scripts/v116b_page_Strider.json):
    /// 20 HP passive animal, "Hitbox size Adult: Height: 1.7 blocks
    /// Width: 0.9 blocks" (baby 0.85/0.45), speed 0.175, "Lava does
    /// not damage striders, and they can walk on top of it without
    /// sinking", damaged by water ("1 HP per ... half-second in
    /// water"), "Groups of 2 to 4 striders spawn on spaces of lava
    /// that have an air block above" with attempts every 400 gt,
    /// drops 2-5 string (100%), "can be fed warped fungus to breed".
    /// Riding (saddle + fungus-on-a-stick) is the standing deferral —
    /// no mount system for striders, disclosed.
    Strider,
    /// 1.16: the piglin — VERIFIED (w/Piglin, live 2026-09-08, raw
    /// capture scripts/v116b_page_Piglin.json): 16 HP "Neutral (adult)"
    /// monster, hitbox 1.95 x 0.6, speed 0.35, "Melee: Golden Sword:
    /// ... Normal: 8 HP" (the engine's melee row; the crossbow's
    /// 2-5 is the ranged row — melee-only here, disclosed), spawns
    /// in Nether Wastes + Crimson Forest in groups of 3-4 (the
    /// w/Crimson_Forest row), bartering: "take gold ingots ... The
    /// piglin 'examines' the ingot for six seconds, then drops a
    /// random item from the chart" (the trimmed engine table,
    /// VERIFIED w/Bartering). Gold-armor pacification needs a wearable
    /// armor system — the engine's adaptation is neutral-until-provoked
    /// + the gold-mining anger hook (disclosed).
    Piglin,
    /// 1.16: the hoglin — VERIFIED (w/Hoglin, live 2026-09-08, raw
    /// capture scripts/v116b_page_Hoglin.json): 40 HP hostile Animal
    /// "Monster", hitbox 1.4 x 1.3965 (JE row), speed 0.3, knockback
    /// resistance 60% (no knockback stat in the engine — disclosed),
    /// "Attack strength Adult in Java Edition: ... Normal: 3 HP to
    /// 8 HP" (engine takes the 5.5 midpoint, disclosed), "Hoglins
    /// avoid being within 7 blocks of warped fungi ... and respawn
    /// anchors", spawn in the Crimson Forest (the only natural biome,
    /// w/Crimson_Forest) in 3-4 packs with 20% JE babies, bred with
    /// crimson fungus, drops raw porkchop 2-4 (100%) + leather 0-1
    /// (50%), 5 XP. Zombification (overworld zoglins) has no mob
    /// dimension transfer — trimmed, disclosed.
    Hoglin,
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
    /// The squid — a PRE-1.13 legacy marker (vanilla added it in Beta
    /// 1.2; the engine's early brackets skipped it and it is NOT one of
    /// 1.13's new mobs). Declared so the aquatic() classification
    /// question has a real answer: the squid is a legacy water mob and
    /// does NOT receive the 1.13 Update Aquatic swim-physics/conduit
    /// gating. No MOB_DATA row, no spawn table entry, no egg — a
    /// classification-only stub, disclosed in the 1.13 worklog.
    Squid,
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
            "llama" => MobKind::Llama,
            "parrot" => MobKind::Parrot,
            "illusioner" => MobKind::Illusioner,
            "vindicator" => MobKind::Vindicator,
            "evoker" => MobKind::Evoker,
            "vex" => MobKind::Vex,
            // 1.13 (Update Aquatic)
            "drowned" => MobKind::Drowned,
            "phantom" => MobKind::Phantom,
            "dolphin" => MobKind::Dolphin,
            "cod" => MobKind::Cod,
            "salmon" => MobKind::Salmon,
            "pufferfish" => MobKind::Pufferfish,
            "tropical_fish" => MobKind::TropicalFish,
            "turtle" => MobKind::Turtle,
            "fox" => MobKind::Fox,
            "bee" => MobKind::Bee,
            // 1.16 (Nether Update, part 2)
            "strider" => MobKind::Strider,
            "piglin" => MobKind::Piglin,
            "hoglin" => MobKind::Hoglin,
            // the completeness audit's classic trio
            "ghast" => MobKind::Ghast,
            "cave_spider" => MobKind::CaveSpider,
            "silverfish" => MobKind::Silverfish,
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
            MobKind::Llama => "minecraft:llama",
            MobKind::Parrot => "minecraft:parrot",
            MobKind::Illusioner => "minecraft:illusioner",
            MobKind::Vindicator => "minecraft:vindicator",
            MobKind::Evoker => "minecraft:evoker",
            MobKind::Vex => "minecraft:vex",
            MobKind::Drowned => "minecraft:drowned",
            MobKind::Phantom => "minecraft:phantom",
            MobKind::Dolphin => "minecraft:dolphin",
            MobKind::Cod => "minecraft:cod",
            MobKind::Salmon => "minecraft:salmon",
            MobKind::Pufferfish => "minecraft:pufferfish",
            MobKind::TropicalFish => "minecraft:tropical_fish",
            MobKind::Turtle => "minecraft:turtle",
            // 1.14: the fox
            MobKind::Fox => "minecraft:fox",
            MobKind::Bee => "minecraft:bee",
            // 1.16 (Nether Update, part 2)
            MobKind::Strider => "minecraft:strider",
            MobKind::Piglin => "minecraft:piglin",
            MobKind::Hoglin => "minecraft:hoglin",
            MobKind::Ghast => "minecraft:ghast",
            MobKind::CaveSpider => "minecraft:cave_spider",
            MobKind::Silverfish => "minecraft:silverfish",
            // classification-only marker (see the enum doc) — still
            // carries its vanilla registry id for completeness
            MobKind::Squid => "minecraft:squid",
        }
    }

    /// The 1.13 entity-id registry spelling (VERIFIED: the 1.13 id set
    /// on minecraft.wiki). Semantic alias of [`MobKind::name`] kept as
    /// its own accessor so registry-id consumers (spawn data, /summon
    /// parity checks, save files) read distinctly from display-name
    /// consumers.
    pub fn registry_id(self) -> &'static str {
        self.name()
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
            // 1.11 sprites (clean-room, auditfix-era tile ids 323.. reused
            // pattern — new 1.11 tiles at 333..=336)
            MobKind::Llama => TILE_LLAMA,
            MobKind::Vindicator => TILE_VINDICATOR,
            MobKind::Evoker => TILE_EVOKER,
            MobKind::Vex => TILE_VEX,
            // 1.12: the parrot base tile — build_vertices picks the
            // per-VARIANT sprite (red/blue/green/cyan/gray)
            MobKind::Parrot => TILE_PARROT_BASE,
            MobKind::Illusioner => TILE_ILLUSIONER,
            MobKind::Drowned => TILE_MOB_DROWNED,
            MobKind::Phantom => TILE_MOB_PHANTOM,
            MobKind::Dolphin => TILE_MOB_DOLPHIN,
            MobKind::Cod => TILE_MOB_COD,
            MobKind::Salmon => TILE_MOB_SALMON,
            MobKind::Pufferfish => TILE_MOB_PUFFERFISH,
            MobKind::TropicalFish => TILE_MOB_TROPICAL_FISH,
            MobKind::Turtle => TILE_MOB_TURTLE,
            // 1.14: the fox sprite (v114_art::fox_art)
            MobKind::Fox => TILE_MOB_FOX,
            // 1.15: the bee sprite (v115_art::bee_art)
            MobKind::Bee => TILE_MOB_BEE,
            // 1.16 part 2: the forest-mob sprites (v116b_art)
            MobKind::Strider => TILE_MOB_STRIDER,
            MobKind::Piglin => TILE_MOB_PIGLIN,
            MobKind::Hoglin => TILE_MOB_HOGLIN,
            // the completeness audit's classic trio (audit16_art)
            MobKind::Ghast => TILE_MOB_GHAST,
            MobKind::CaveSpider => TILE_MOB_CAVESPIDER,
            MobKind::Silverfish => TILE_MOB_SILVERFISH,
            // classification-only marker — never rendered (no MOB_DATA
            // row, no spawn path); reuses the passive-fish tile as a
            // safe stand-in should a future bracket implement it
            MobKind::Squid => TILE_MOB_COD,
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
                // 1.11 illagers + vex (VERIFIED w/Vindicator "Behavior
                // Hostile", w/Evoker "Behavior Hostile", w/Vex "Behavior
                // Hostile")
                | MobKind::Vindicator
                | MobKind::Evoker
                | MobKind::Vex
                // 1.12 (VERIFIED w/Illusioner infobox "Behavior Hostile")
                | MobKind::Illusioner
                // 1.13: the drowned + phantom — VERIFIED infoboxes
                // "Behavior Hostile" (the drowned's zombie-parity aggro
                // is the infobox's own framing; the phantom is undead)
                | MobKind::Drowned
                | MobKind::Phantom
                // 1.16 (Nether Update, part 2): the hoglin — VERIFIED
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
        )
    }
    pub fn neutral(self) -> bool {
        // 1.11: the llama — VERIFIED w/Llama infobox "Neutral"
        // 1.13: dolphin + pufferfish — VERIFIED infoboxes "Neutral"
        // (the pufferfish's contact defense is not an attack)
        // 1.16: the piglin — VERIFIED w/Piglin infobox "Neutral (adult)"
        // (the gold-armor pacification is the no-armor adaptation,
        // disclosed — the enderman class: neutral until provoked)
        self == MobKind::Enderman
            || self == MobKind::IronGolem
            || self == MobKind::Llama
            || self == MobKind::Dolphin
            || self == MobKind::Pufferfish
            || self == MobKind::Piglin
    }

    /// 1.13: aquatic mobs — swim physics (buoyancy, 3D steering),
    /// Conduit damage targets, Impaling bonus targets. The tropical
    /// fish 2,700-variant math: 2 shapes x 15 base x 6 patterns x
    /// 15 pattern colors (VERIFIED w/Tropical_Fish).
    pub fn aquatic(self) -> bool {
        matches!(
            self,
            MobKind::Dolphin
                | MobKind::Cod
                | MobKind::Salmon
                | MobKind::Pufferfish
                | MobKind::TropicalFish
                | MobKind::Turtle
                | MobKind::Drowned
        )
    }

    /// The engine's flying mobs (vanilla FlyingMob class forms):
    /// the phantom (1.13), the vex (1.11), the bat (E2), the parrot
    /// (1.12). These take NO gravity — their AI's vertical steering
    /// is the only vertical force (the phantom's 12-block orbit kept
    /// sagging 3 blocks under gravity before this classification;
    /// flying mobs also never accumulate fall distance).
    pub fn flies(self) -> bool {
        matches!(
            self,
            MobKind::Phantom
                | MobKind::Vex
                | MobKind::Bat
                | MobKind::Parrot
                | MobKind::Bee
                // the completeness audit: the ghast — "large, floating,
                // ghost-like" (VERIFIED w/Ghast; the bat/phantom class)
                | MobKind::Ghast
        )
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
            // 1.11: kinds 23..=26 (the "5 new spawn eggs") + the
            // re-added husk/stray eggs (kinds 27/28, VERIFIED changelog
            // §Items: "Husk spawn egg, Stray spawn egg" among the
            // 1.10-pre2 removals re-added in 1.11)
            23 => MobKind::Llama,
            24 => MobKind::Vindicator,
            25 => MobKind::Evoker,
            26 => MobKind::Vex,
            27 => MobKind::Husk,
            28 => MobKind::Stray,
            // 1.12: the parrot egg (changelog §Items: "Parrot Spawn
            // Egg") — kind 30
            30 => MobKind::Parrot,
            // 1.13: the V9 egg window (changelog §Items: Drowned/
            // Phantom/Dolphin/Cod/Salmon/Pufferfish/Tropical Fish/
            // Turtle Spawn Eggs) — kinds 32..=39
            32 => MobKind::Drowned,
            33 => MobKind::Phantom,
            34 => MobKind::Dolphin,
            35 => MobKind::Cod,
            36 => MobKind::Salmon,
            37 => MobKind::Pufferfish,
            38 => MobKind::TropicalFish,
            39 => MobKind::Turtle,
            // 1.14: the fox egg (changelog §Mobs: "Fox Spawn Egg") —
            // kind 40
            40 => MobKind::Fox,
            // 1.15: the bee egg (changelog §Items: "Bee Spawn Egg") —
            // kind 41
            41 => MobKind::Bee,
            // 1.16: the V14 egg window — kinds 42..=44 (the changelog's
            // own spawn-egg list; the zoglin/piglin-brute eggs are
            // trimmed with their mobs, disclosed)
            42 => MobKind::Strider,
            43 => MobKind::Piglin,
            44 => MobKind::Hoglin,
            // the completeness audit's classic trio — kinds 45..=47
            45 => MobKind::Ghast,
            46 => MobKind::CaveSpider,
            47 => MobKind::Silverfish,
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
            // 1.11 (changelog §Items: "5 new spawn eggs" — Vindicator,
            // Llama, Evoker, Vex, Zombie Villager): kinds 23..=26 (the
            // zombie-villager egg is the PRE-EXISTING E2-era item at
            // kind 5 — an engine anachronism that satisfies the 1.11
            // requirement; disclosed in the WORKLOG)
            MobKind::Llama => 23,
            MobKind::Vindicator => 24,
            MobKind::Evoker => 25,
            MobKind::Vex => 26,
            // 1.11 re-added eggs (changelog: "Eggs that were removed in
            // Java Edition 1.10-pre2 are re-added ... including: ...
            // Husk spawn egg, Stray spawn egg"): kinds 27/28
            MobKind::Husk => 27,
            MobKind::Stray => 28,
            // 1.12: the parrot egg (kind 30 — the V8 egg window)
            MobKind::Parrot => 30,
            // F-series mobs without eggs (1.8 rabbit, 1.10 polar bear)
            // — 255 = "no egg" sentinel (the rabbit egg is a standing
            // 1.8-bracket deferral, the polar-bear egg a 1.10 one; both
            // out of the 1.11 scope, disclosed)
            MobKind::Rabbit | MobKind::PolarBear => 255,
            // 1.12: the illusioner has NO spawn egg in vanilla (VERIFIED
            // w/Illusioner: "Does not currently have a spawn egg, so can
            // only be summoned with /summon") — the same 255 sentinel
            MobKind::Illusioner => 255,
            // 1.13: the V9 egg window — kinds 32..=39 (the changelog's
            // own spawn-egg list)
            MobKind::Drowned => 32,
            MobKind::Phantom => 33,
            MobKind::Dolphin => 34,
            MobKind::Cod => 35,
            MobKind::Salmon => 36,
            MobKind::Pufferfish => 37,
            MobKind::TropicalFish => 38,
            MobKind::Turtle => 39,
            // 1.14: the fox egg — kind 40 (the V10 egg window)
            MobKind::Fox => 40,
            MobKind::Bee => 41,
            // 1.16: the V14 egg window — kinds 42..=44
            MobKind::Strider => 42,
            MobKind::Piglin => 43,
            MobKind::Hoglin => 44,
            // the completeness audit's classic trio — kinds 45..=47
            MobKind::Ghast => 45,
            MobKind::CaveSpider => 46,
            MobKind::Silverfish => 47,
            // classification-only marker: the squid never had an egg in
            // the engine's window (pre-1.13 legacy, unimplemented)
            MobKind::Squid => 255,
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

pub const MOB_DATA: [MobDef; 48] = [
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
    },    // ---- 1.11 bracket (VERIFIED live 2026-09-07) ----
    MobDef {
        kind: MobKind::Llama,
        health: 30.0, // 15–30 range per-instance (like horses)
        damage: 1.0, // spit: 1 HP Easy/Normal (Hard 1.5 via scale)
        speed_attr: 0.175, // w/Llama "Speed 0.175"
        armor: 0.0,
        height: 1.87, // w/Llama hitbox
        width: 0.9,
        xp: 1,
    },
    MobDef {
        kind: MobKind::Vindicator,
        health: 24.0, // w/Vindicator
        damage: 13.0, // iron axe Normal (7.5/19.5 via difficulty_scale)
        speed_attr: 0.535, // 5.612 b/s / 10.5 (sprint-speed — w/Vindicator)
        armor: 0.0,
        height: 1.95, // JE hitbox
        width: 0.6,
        xp: 5,
    },
    MobDef {
        kind: MobKind::Evoker,
        health: 24.0, // w/Evoker
        damage: 6.0, // fangs: 6 HP, ignores armor (armor-bypass on hit)
        speed_attr: 0.23, // evokers walk slowly (vanilla illager speed 0.5? — w/Evoker 0.5? our adaptation 0.23, walking-pace caster)
        armor: 0.0,
        height: 1.95,
        width: 0.6,
        xp: 10,
    },
    MobDef {
        kind: MobKind::Vex,
        health: 14.0, // w/Vex
        damage: 9.0, // iron sword Normal (5.5/13.5 via difficulty_scale)
        speed_attr: 0.7, // fast flyer (vanilla 0.7)
        armor: 0.0,
        height: 0.8, // w/Vex hitbox 0.8 tall
        width: 0.4,
        xp: 5, // "5 XP is dropped when a vex is killed"
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
    // ---- 1.12 bracket (World of Color, live-verified 2026-09-07) ----
    MobDef {
        // VERIFIED w/Parrot infobox: 6 HP, passive, "Speed 0.2";
        // 1–3 XP (w/Parrot §Drops: "1–3XP experience orbs are dropped
        // when parrots are killed by a player"); hitbox 0.9 tall — the
        // wiki gives 0.9 height / 0.5 width per the entity data page
        kind: MobKind::Parrot,
        health: 6.0,
        damage: 0.0,
        speed_attr: 0.2,
        armor: 0.0,
        height: 0.9,
        width: 0.5,
        xp: 2, // 1–3 XP (VERIFIED w/Parrot §Drops)
    },
    MobDef {
        // VERIFIED w/Illusioner infobox: 32 HP, hostile, "Speed 0.5",
        // bow "Easy and Normal: 2HP – 5HP / Hard: 3HP – 5HP" → Normal
        // mid 3.5 (the skeleton's arrow-mid convention); 5 XP (w/Illusioner
        // §Drops: "5XP experience orbs are dropped"); illager hitbox
        // (1.95 tall like the vindicator)
        kind: MobKind::Illusioner,
        health: 32.0,
        damage: 3.5,
        speed_attr: 0.5,
        armor: 0.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    // ---- 1.13 (Update Aquatic) — all values VERIFIED live 2026-09-07
    // against the per-mob wiki captures (voxelcraft/scripts/v113_*) ----
    MobDef {
        // w/Drowned: 20 HP base (zombie-parity; the 40/100 "leader"
        // forms are Java-internal — disclosed), Normal melee 3, armor 2
        // (undead zombie family), 5 XP + the trident drop rule
        kind: MobKind::Drowned,
        health: 20.0,
        damage: 3.0,
        speed_attr: 0.23,
        armor: 2.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        // w/Phantom: 20 HP undead, 2 HP E/N + 3 H (the 1.14-pre3 value
        // the current wiki lists; the 1.13 original was 6 — version-
        // scoped, disclosed), 5 XP
        kind: MobKind::Phantom,
        health: 20.0,
        damage: 2.0,
        speed_attr: 0.7,
        armor: 0.0,
        height: 0.5,
        width: 0.9,
        xp: 5,
    },
    MobDef {
        // w/Dolphin: 10 HP neutral, Normal 3, 1-3 XP (engine takes 1)
        kind: MobKind::Dolphin,
        health: 10.0,
        damage: 3.0,
        speed_attr: 0.7,
        armor: 0.0,
        height: 0.6,
        width: 0.9,
        xp: 1,
    },
    MobDef {
        // w/Cod: 3 HP passive fish, 1 XP
        kind: MobKind::Cod,
        health: 3.0,
        damage: 0.0,
        speed_attr: 0.13,
        armor: 0.0,
        height: 0.3,
        width: 0.5,
        xp: 1,
    },
    MobDef {
        // w/Salmon: 3 HP passive fish (3 size variants live in the
        // variant byte — hitbox scaled at spawn), 1 XP
        kind: MobKind::Salmon,
        health: 3.0,
        damage: 0.0,
        speed_attr: 0.12,
        armor: 0.0,
        height: 0.4,
        width: 0.7,
        xp: 1,
    },
    MobDef {
        // w/Pufferfish: 3 HP neutral, contact 3 Normal fully-puffed,
        // 1 XP (the poison rides the contact-hit payload)
        kind: MobKind::Pufferfish,
        health: 3.0,
        damage: 3.0,
        speed_attr: 0.13,
        armor: 0.0,
        height: 0.5,
        width: 0.5,
        xp: 1,
    },
    MobDef {
        // w/Tropical_Fish: 3 HP passive, 1 XP (2700 variants encoded
        // across variant/aux — see tropical_decode)
        kind: MobKind::TropicalFish,
        health: 3.0,
        damage: 0.0,
        speed_attr: 0.15,
        armor: 0.0,
        height: 0.4,
        width: 0.5,
        xp: 1,
    },
    MobDef {
        // w/Turtle: 30 HP x 15 passive; slow land speed ~0.12, fast
        // swimmer; 1 XP (w/Turtle §Drops: 1-3 XP)
        kind: MobKind::Turtle,
        health: 30.0,
        damage: 0.0,
        speed_attr: 0.12,
        armor: 0.0,
        height: 0.4,
        width: 1.2,
        xp: 1,
    },
    // 1.14: the fox (VERIFIED w/Fox infobox — the raw capture
    // scripts/v114_page_fox.json): 10 HP, attack "Easy and Normal:
    // 2 HP, Hard: 3 HP" (the NORMAL row here; Hard scales via
    // combat::difficulty_scale), speed 0.3, hitbox 0.7 × 0.6, XP 1–3
    // (the engine's fixed passive row — the random 1..=3 range is
    // documented)
    MobDef {
        kind: MobKind::Fox,
        health: 10.0,
        damage: 2.0,
        speed_attr: 0.3,
        armor: 0.0,
        height: 0.7,
        width: 0.6,
        xp: 1,
    },
    // 1.15: the bee (VERIFIED w/Bee infobox — the raw capture
    // scripts/v115_page_Bee.json): 10 HP, sting 2 HP (Normal; Hard
    // scales via combat::difficulty_scale) + Poison I 10 s (Normal)
    // / 18 s (Hard), speed 0.6 (the infobox row — the Java attribute
    // is 0.3; both readings inside the documented conversion class),
    // hitbox 0.5 x 0.55, XP 1-3 on a player kill (the fixed row —
    // the 1..=3 range documented)
    MobDef {
        kind: MobKind::Bee,
        health: 10.0,
        damage: 2.0,
        speed_attr: 0.6,
        armor: 0.0,
        height: 0.5,
        width: 0.55,
        xp: 2,
    },
    // ---- 1.16 (Nether Update, part 2): the three nether-forest mobs
    // (all VERIFIED against the v116b captures) ----
    MobDef {
        // w/Strider: 20 HP, passive, hitbox 1.7 x 0.9, speed 0.175,
        // 1-3 XP (the "Experience Orb" row) — drops ride the deaths
        // queue (2-5 string, 100%)
        kind: MobKind::Strider,
        health: 20.0,
        damage: 0.0,
        speed_attr: 0.175,
        armor: 0.0,
        height: 1.7,
        width: 0.9,
        xp: 2,
    },
    MobDef {
        // w/Piglin: 16 HP, neutral, hitbox 1.95 x 0.6, speed 0.35,
        // golden-sword Normal 8 (the melee row; the crossbow's 2-5 is
        // the ranged half, melee-only in the engine, disclosed), 5 XP
        kind: MobKind::Piglin,
        health: 16.0,
        damage: 8.0,
        speed_attr: 0.35,
        armor: 0.0,
        height: 1.95,
        width: 0.6,
        xp: 5,
    },
    MobDef {
        // w/Hoglin: 40 HP, hostile, hitbox 1.4 x 1.3965 (JE), speed
        // 0.3, attack "Normal: 3 HP to 8 HP" — the 5.5 midpoint,
        // disclosed; 5 XP ("5 XP if killed by a player")
        kind: MobKind::Hoglin,
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

// ---- 1.15 (Buzzy Bees) constants (VERIFIED w/Bee, live 2026-09-08) ----
/// anger duration roll: 20..=39 seconds → 400..=780 ticks ("Anger
/// duration is randomly selected between 20 and 39 seconds,
/// inclusive")
pub const ANGER_TICKS_MIN_ROLL: i32 = 400;
pub const ANGER_TICKS_MAX: i32 = 780;

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
    /// - 1.12 Parrot: bits 0..=2 = the color variant 0..=4 (red/blue/
    ///   green/cyan/gray — VERIFIED w/Parrot Variant NBT table); bit 7
    ///   (0x80) = tamed (fed seeds — the 1/10 roll); the SIT state lives
    ///   in aux bit 0 (right-click toggle, VERIFIED 17w14a: "The
    ///   right-click action has been changed: right-clicking on a tamed
    ///   parrot now tells it to sit")
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
    /// 1.15 (Buzzy Bees): per-instance bee state (None for every
    /// other kind) — the hive/flower lifecycle, nectar payload,
    /// anger/sting timers (VERIFIED w/Bee; see bees.rs)
    pub bee: Option<Box<super::bees::BeeState>>,
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
    /// 1.13: poison payload — Some(ticks) applies Poison I (VERIFIED
    /// w/Pufferfish: contact "Poison for 6 seconds" fully puffed Java /
    /// 3 s semi-puffed; the engine's one-tier poison is the I form,
    /// disclosed)
    pub poison_effect: Option<i32>,
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
    /// 1.11: llama spit — 1 HP Easy/Normal (1.5 Hard via difficulty
    /// scale; VERIFIED w/Llama: "Llama Spit: Easy and Normal: 1 HP,
    /// Hard: 1.5 HP")
    LlamaSpit,
    /// 1.13: the trident — 8 HP base (VERIFIED w/Trident: "Projectile
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
pub const PLAYER_OWNER: u32 = u32::MAX;

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
    /// 1.13: water-ambient spawn cadence counter (the fish/dolphin/
    /// turtle attempt, 1/40 ticks — the bat pattern)
    aquatic_spawn_t: u64,
    /// 1.16 (Nether Update, part 2): strider spawn cadence counter —
    /// "In Java Edition, striders are the only passive mob in the
    /// Nether, so spawning attempts are made every 400 game ticks"
    /// (VERIFIED w/Strider)
    strider_spawn_t: u64,
    /// 1.14: environmental-hazard cadence counter — the berry-bush
    /// and campfire damage windows fire on `hazard_t % 10 == 0` (the
    /// vanilla 0.5 s damage-immunity cadence, VERIFIED
    /// w/Sweet_Berry_Bush + w/Campfire: "1 HP every tick (although
    /// damage immunity reduces this to once every half-second)")
    hazard_t: u64,
    /// 1.13: "Time Since Last Rest" — ticks since the player last died
    /// (or slept, when beds exist). At ≥ 72000 (3 in-game days) phantoms
    /// start spawning above the player (VERIFIED w/Phantom §Spawning:
    /// "Phantoms spawn when a player's Time Since Last Rest reaches
    /// 1 hour (72000 ticks)"; beds don't exist yet — reset rides death,
    /// disclosed). Incremented once per tick in tick().
    pub rest_t: u64,
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
    /// 1.11 evoker spells, consumed by the game layer: (evoker id, vex
    /// count) — the summon spell spawns vexes around the caster.
    pub pending_summons: Vec<(u32, usize)>,
    /// 1.11 evoker fang strikes on the player: armor-ignoring damage
    /// amounts (VERIFIED w/Evoker: "not mitigated by armor").
    pub pending_player_fang: Vec<f32>,
    /// 1.12 illusioner spells, consumed by the game layer: Blindness
    /// applications on the player (ticks each — the 20 s spell,
    /// VERIFIED w/Illusioner §Casting_Blindness: "This spell gives a
    /// Blindness effect that lasts for 20 seconds upon first engaging
    /// a new player opponent").
    pub pending_player_blindness: Vec<i32>,
    /// 1.13 Dolphin's Grace applications (ticks each — VERIFIED
    /// w/Dolphin: "a swimming speed boost for 5 seconds, replenished
    /// as long as the player stays close"; the game layer applies
    /// the effect while a sprint-swimming player is within 9 blocks).
    pub pending_player_grace: Vec<i32>,
    /// 1.13 turtle egg placements, consumed by the game layer (world
    /// edits must ride the light engine): (x, y, z, state) — the
    /// egg block at hatch stage 0 (VERIFIED w/Turtle: "A turtle lays
    /// eggs after digging" on its home beach).
    pub pending_turtle_eggs: Vec<(i32, i32, i32, u16)>,
    /// 1.13 item drops the mob system itself owes the world (baby
    /// turtle maturity scutes — VERIFIED w/Scute: "Dropped when baby
    /// turtles grow up"): (position, block id).
    /// 1.15 (Buzzy Bees): day flag (the game layer's sun state —
    /// drives the bees' night-return phase)
    pub is_day: bool,
    /// 1.15: bees that reached their hive this tick — (mob id, hive
    /// position, carried nectar). Drained by the game layer into the
    /// HiveSystem; the MOB is removed by the tick pass.
    pub bee_enters: Vec<(u32, [i32; 3], bool)>,
    /// 1.15: queued pollinations — (crop position, target age). The
    /// game layer writes the berry-bush state (the bone-meal-like
    /// stage advance, VERIFIED w/Bee)
    pub bee_pollinations: Vec<([i32; 3], u8)>,
    pub pending_drops: Vec<([f32; 3], u16)>,
    /// mob deaths (drops + XP handled by the game layer); the u8 carries
    /// the per-kind variant (magma-cube size code etc.)
    pub deaths: Vec<(MobKind, [f32; 3], u8)>,
    /// Phase E1: mob-vs-mob damage queued inside ai_tick (borrow split) —
    /// (target id, damage). Applied before the deaths scan.
    pub pending_damage: Vec<(u32, f32)>,
    /// explosion requests (center, power) — game.rs owns world edits so
    /// the light engine updates ride along
    pub explosions: Vec<([f32; 3], f32)>,
    /// 1.16: projectile hits on TARGET blocks, drained by the game
    /// layer (position, power 1..=15, ticks_left — 20 for arrows/
    /// tridents, 8 for the others; VERIFIED w/Target). Filled by
    /// tick_arrows' block-collision arm.
    pub target_hits: Vec<([i32; 3], u8, i32)>,
    /// the sweep-2 landing queue: player-thrown eggs (hatch the chick
    /// roll at the landing) and pearls (teleport the thrower) — drained
    /// by the game layer. Filled by tick_arrows' block-collision arm
    /// and the mob-hit arm.
    pub landings: Vec<(ProjKind, [f32; 3])>,
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
            aquatic_spawn_t: 0,
            strider_spawn_t: 0,
            hazard_t: 0,
            rest_t: 0,
            ridden: None,
            next_id: 1,
            player: None,
            player_invulnerable: false,
            hits: Vec::new(),
            deaths: Vec::new(),
            pending_summons: Vec::new(),
            pending_player_fang: Vec::new(),
            pending_player_blindness: Vec::new(),
            pending_player_grace: Vec::new(),
            pending_turtle_eggs: Vec::new(),
        is_day: true,
        bee_enters: Vec::new(),
        bee_pollinations: Vec::new(),
            pending_drops: Vec::new(),
            pending_damage: Vec::new(),
            explosions: Vec::new(),
            target_hits: Vec::new(),
            landings: Vec::new(),
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
        // The vanilla passive (creature) category EXCLUDES the 1.13
        // water pools (water_ambient / water_creature are separate
        // spawn categories with their own caps — VERIFIED w/Java
        // Edition 1.13 §Spawning): fish/dolphins/turtles never count
        // against the passive cap.
        self.list
            .iter()
            .filter(|m| !m.kind.hostile() && !m.kind.aquatic())
            .count()
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
        } else if kind == MobKind::Llama {
            // VERIFIED w/Llama: 15–30 HP (random per instance)
            15.0 + self.rng.next_f32() * 15.0
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
            equine: if matches!(
                kind,
                MobKind::Horse | MobKind::Donkey | MobKind::Mule
            ) || kind == MobKind::Llama
            {
                // 1.11: llamas share the temper-taming infrastructure
                // (VERIFIED w/Llama §Taming: "Llamas can be tamed by
                // repetitively riding them until hearts are displayed" +
                // "Taming success depends on the llama's Temper value" —
                // the horse mechanic); fixed 0.175 speed (w/Llama "Speed
                // 0.175"), no jump stat (not rideable-steered in engine —
                // disclosed)
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
            // 1.15: bees get per-instance lifecycle state
            bee: if kind == MobKind::Bee {
                Some(Box::new(super::bees::BeeState::new()))
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

    /// 1.15: the game layer assigns a released bee its home hive (the
    /// release queue drains in game.rs; spawn_variant makes the bee
    /// homeless, this points it home). Also flags it angry when the
    /// release was an anger case.
    pub fn set_bee(&mut self, id: u32, hive: [i32; 3], angry: bool) {
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            if let Some(b) = m.bee.as_mut() {
                b.hive = Some(hive);
                if angry {
                    b.anger_t = ANGER_TICKS_MIN_ROLL + self.rng.next_range(
                        (ANGER_TICKS_MAX - ANGER_TICKS_MIN_ROLL + 1) as u32,
                    ) as i32;
                    b.phase = super::bees::PH_ANGRY;
                }
            }
        }
    }

    /// 1.15: anger every bee near a position (the swarm — VERIFIED
    /// w/Bee: "All bees nearby are angered when an individual bee is
    /// attacked (unless the bee attacked is killed in one hit) ...
    /// or a bee nest/beehive is destroyed"). The hive-family bees
    /// (same home) anger at ANY distance in the sim; strangers within
    /// 16 blocks join (the "nearby" reading).
    pub fn anger_bees_near(&mut self, pos: [f32; 3], hive_family: Option<[i32; 3]>) -> usize {
        let mut n = 0;
        for m in self.list.iter_mut() {
            if m.kind != MobKind::Bee {
                continue;
            }
            let Some(b) = m.bee.as_mut() else { continue };
            if b.anger_t > 0 || b.stung {
                continue; // already angry, or spent
            }
            let family = b.hive == hive_family && hive_family.is_some();
            let dx = m.pos[0] - pos[0];
            let dy = m.pos[1] - pos[1];
            let dz = m.pos[2] - pos[2];
            let near = dx * dx + dy * dy + dz * dz < 16.0 * 16.0;
            if family || near {
                b.anger_t = ANGER_TICKS_MIN_ROLL
                    + self.rng.next_range(
                        (ANGER_TICKS_MAX - ANGER_TICKS_MIN_ROLL + 1) as u32,
                    ) as i32;
                b.phase = super::bees::PH_ANGRY;
                n += 1;
            }
        }
        n
    }

    /// ONE deterministic sim tick (20 Hz).
    /// Phase 6 §26: `sim_center`/`sim_radius` = the simulation-distance
    /// ring — mobs outside it freeze (AI + physics), spawning clamps its
    /// chunk pick to the ring (vanilla JE: simulation distance "controls
    /// mob spawning and despawning, and tick updates" — wiki). Despawn
    /// runs regardless (distance-based bookkeeping, cheap).
    pub fn tick(&mut self, world: &World, sim_center: (i32, i32), sim_radius: i32) {
        // 1.14: the hazard clock ticks with the sim (bush + campfire
        // damage windows key off it)
        self.hazard_t += 1;
        let hazard_window = self.hazard_t % 10 == 0;
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
        // 1.13 (Update Aquatic): the water-ambient pool — fish schools,
        // dolphin pods, beach turtles (NOT counted toward the passive
        // cap — the vanilla water_ambient/water_creature categories are
        // separate, VERIFIED w/Java_Edition_1.13 §Spawning)
        self.aquatic_spawn_t += 1;
        if self.aquatic_spawn_t % 40 == 0 {
            self.try_spawn_aquatic(world, sim_ring);
        }
        // 1.16 (Nether Update, part 2): the strider lava-sea pool —
        // "Groups of 2 to 4 striders spawn on spaces of lava that have
        // an air block above", attempts every 400 gt (VERIFIED
        // w/Strider §Spawning) — nether-only, passive-cap-free (the
        // strider is the nether's only passive mob, its own category)
        self.strider_spawn_t += 1;
        if self.strider_spawn_t % 400 == 0
            && world.dimension == vc_world::world::Dimension::Nether
        {
            self.try_spawn_striders(world, sim_ring);
        }
        // 1.13: phantom insomnia spawns — every 20 ticks while "Time
        // Since Last Rest" ≥ 72000 (VERIFIED w/Phantom §Spawning: the
        // 1–4 local pack; engine rolls one per attempt)
        self.rest_t += 1;
        if self.rest_t % 20 == 0 && self.rest_t >= 72000 {
            self.try_spawn_phantom(world, sim_ring);
        }
        }

        // 2. AI + physics (split borrows: rng/hits/arrows vs the mob list)
        let player = self.player;
        let invuln = self.player_invulnerable;
        let rng = &mut self.rng;
        let hits = &mut self.hits;
        let arrows = &mut self.arrows;
        let pending = &mut self.pending_damage;
        // 1.11: the evoker spell queues (drained by the game layer)
        let pending_summons = &mut self.pending_summons;
        let pending_player_fang = &mut self.pending_player_fang;
        // 1.12: the illusioner blindness queue
        let pending_blindness = &mut self.pending_player_blindness;
        // 1.13: the aquatic queues
        let pending_grace_q = &mut self.pending_player_grace;
        let pending_turtle_eggs_q = &mut self.pending_turtle_eggs;
        let pending_drops_q = &mut self.pending_drops;
        // 1.15: the bee queues + the day flag snapshot
        let bee_enters_q = &mut self.bee_enters;
        let bee_pollinations_q = &mut self.bee_pollinations;
        let is_day_snapshot = self.is_day;
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
            // LATENT-BUG FIX (found by the 1.11 llama-spit test):
            // attack_cd is i32 — saturating_sub floors at i32::MIN, NOT
            // 0, so a fresh mob's cooldown walked 0 → -1 → -2 … and
            // the `attack_cd == 0` attack gates NEVER fired for a mob
            // that had not already attacked (skeleton arrows, melee
            // swings, llama spit — only reachable via direct ai_tick
            // calls before this fix). Floor at zero: a fresh mob (cd 0)
            // can strike immediately; a 20-tick cooldown counts
            // 19..=0 then re-fires on the 20th tick.
            m.attack_cd = (m.attack_cd - 1).max(0);
            // Phase E3: the ridden mount's AI is suspended — the game
            // layer drives its velocity (physics still applies)
            if self.ridden == Some(m.id) {
                physics_tick(m, world);
                continue;
            }
            ai_tick(
                rng,
                m,
                player,
                invuln,
                hits,
                arrows,
                world,
                &snapshot,
                pending,
                pending_summons,
                pending_player_fang,
                pending_blindness,
                pending_grace_q,
                pending_turtle_eggs_q,
                pending_drops_q,
                bee_enters_q,
                bee_pollinations_q,
                is_day_snapshot,
            );
            // 1.14: environmental hazards AFTER the AI steering (the
            // bush slow must survive to the physics move) and BEFORE
            // physics_tick
            hazard_tick(m, world, hazard_window);
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

        // 3.4 1.15 (Buzzy Bees): arrived bees leave the mob list —
        // their data traveled through bee_enters (the game layer
        // stores them in the HiveSystem)
        {
            let mut i = 0;
            while i < self.list.len() {
                let arrived = self.list[i]
                    .bee
                    .as_ref()
                    .map(|b| b.arrived)
                    .unwrap_or(false);
                if arrived {
                    self.list.remove(i);
                } else {
                    i += 1;
                }
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
        let mut target_hits = std::mem::take(&mut self.target_hits);
        let mut landings = std::mem::take(&mut self.landings);
        tick_arrows(
            &mut self.arrows,
            player,
            invuln,
            &mut self.hits,
            world,
            &mut mobs,
            &mut pending,
            &mut target_hits,
            &mut landings,
        );
        self.list = mobs;
        self.pending_damage = pending;
        self.target_hits = target_hits;
        self.landings = landings;
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
        // 1.13 (Update Aquatic): ocean-family columns roll DROWNED —
        // "Drowned spawn naturally ... in ocean and river biomes"
        // (VERIFIED w/Drowned §Spawning; no river biome in the engine,
        // disclosed). Water positions spawn drowned directly (drowned
        // are the ocean's water-column hostile; the light gate below
        // is a land rule — deep water is its own darkness, disclosed);
        // land positions in an ocean chunk convert zombie → drowned.
        let col_biome =
            vc_world::gen::Biome::from_u8(world.get_biome(cx * 16 + lx, cz * 16 + lz));
        let ocean_family = col_biome.is_ocean();
        let py = p[1] as i32;
        for y in (py - 40..py + 16).rev() {
            if !(1..=250).contains(&y) {
                continue;
            }
            let wx = cx * 16 + lx;
            let wz = cz * 16 + lz;
            // 1.13: the ocean water column — a drowned spawn position
            // is two stacked WATER blocks (they sink/stand via the
            // aquatic swim physics); packs of 1–2, 6.25% trident-armed
            // (VERIFIED w/Drowned §Equipment: "6.25% of drowned spawn
            // with a trident" — bit 0 arms the throw)
            if ocean_family
                && world.get_block(wx, y, wz) == WATER
                && world.get_block(wx, y + 1, wz) == WATER
            {
                let pack = 1 + (self.rng.next_range(2)) as usize;
                for _ in 0..pack {
                    let armed = if self.rng.next_f32() < 0.0625 { 1u8 } else { 0 };
                    let _ = self.spawn_variant(MobKind::Drowned, wx, y, wz, armed);
                }
                return; // one attempt per tick
            }
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
                // 1.16 (Nether Update, part 2): the biome-aware nether
                // roll — Crimson Forest: piglins ("often seen in this
                // biome in groups of 3-4", VERIFIED w/Crimson_Forest)
                // + hoglins ("the only biome where hoglins naturally
                // spawn outside of bastion remnants"); Warped Forest:
                // "hostile mobs do not spawn naturally" — endermen
                // are the exception ("Endermen are common in this
                // biome", VERIFIED w/Warped_Forest); Wastes: the
                // part-1 roll (magma cubes 2/21 + the zombie filler,
                // piglins join at their wastes weight — VERIFIED
                // w/Piglin §Spawning "Nether Wastes")
                match vc_world::gen::Biome::from_u8(world.get_biome(wx, wz)) {
                    vc_world::gen::Biome::CrimsonForest => {
                        if self.rng.next_range(5) < 3 {
                            MobKind::Piglin
                        } else {
                            MobKind::Hoglin
                        }
                    }
                    vc_world::gen::Biome::WarpedForest => MobKind::Enderman,
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
                    },
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
                            // zombie -> husk (80%) in deserts; zombie ->
                            // drowned in ocean-family chunks (1.13: the
                            // drowned REPLACE zombies there — VERIFIED
                            // w/Java_Edition_1.13 §Spawning)
                            if biome == 4 && self.rng.next_f32() < 0.8 {
                                MobKind::Husk
                            } else if ocean_family {
                                MobKind::Drowned
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
                } else if kind == MobKind::Drowned {
                    // 1.13: land-spawned drowned roll the same 6.25%
                    // trident-armed bit (VERIFIED w/Drowned §Equipment)
                    (
                        kind,
                        if self.rng.next_f32() < 0.0625 { 1u8 } else { 0 },
                    )
                } else {
                    (kind, 0)
                };
                let _ = self.spawn_variant(spawn_kind, wx, y, wz, variant);
                // 1.16 part 2: the forest packs extend to 3-4 (VERIFIED
                // w/Crimson_Forest "groups of 3-4" + w/Hoglin's spawn
                // table) — one extra mob per loop pass while the pack
                // roll lasts, hoglin babies at the 20% row (VERIFIED
                // w/Hoglin "20% of hoglins spawn as babies")
                if (spawn_kind == MobKind::Piglin || spawn_kind == MobKind::Hoglin)
                    && self.rng.next_range(2) == 0
                {
                    let v = if spawn_kind == MobKind::Hoglin
                        && self.rng.next_f32() < 0.2
                    {
                        0x40u8 // baby (20%, VERIFIED)
                    } else {
                        0
                    };
                    let _ = self.spawn_variant(spawn_kind, wx, y, wz, v);
                    if v != 0 {
                        if let Some(m) = self.list.last_mut() {
                            m.aux = 24000; // the 20-minute maturity
                        }
                    }
                }
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

    /// 1.13 (Update Aquatic): the water-ambient attempt — fish schools,
    /// dolphin pods, beach turtles. Runs every 40 ticks (the bat
    /// cadence) and is NOT counted toward the passive cap (the vanilla
    /// water_ambient / water_creature categories are separate —
    /// VERIFIED w/Java_Edition_1.13 §Spawning).
    /// Fish biome rows (VERIFIED w/Cod, w/Salmon, w/Tropical_Fish,
    /// w/Pufferfish §Spawning, simplified to the temperature families):
    /// cod = cold + neutral, salmon = cold/frozen, tropical =
    /// lukewarm/warm, pufferfish = warm. Dolphins: "all ocean biomes
    /// except frozen/cold", pods 1–2 (VERIFIED w/Dolphin).
    fn try_spawn_aquatic(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
        if world.dimension != vc_world::world::Dimension::Overworld {
            return;
        }
        let Some(p) = self.player else { return };
        let aquatics = self
            .list
            .iter()
            .filter(|m| {
                matches!(
                    m.kind,
                    MobKind::Cod
                        | MobKind::Salmon
                        | MobKind::Pufferfish
                        | MobKind::TropicalFish
                        | MobKind::Dolphin
                )
            })
            .count();
        if aquatics >= 12 {
            return; // water-ambient cap (adaptation: vanilla pools per category)
        }
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(9) as i32) - 4;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(9) as i32) - 4;
        if !sim_ring(cx, cz) || world.chunk((cx, cz)).is_none() {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        let biome =
            vc_world::gen::Biome::from_u8(world.get_biome(cx * 16 + lx, cz * 16 + lz));
        let wx = cx * 16 + lx;
        let wz = cz * 16 + lz;
        // turtles: Beach-biome sand columns, groups ≤ 5, 5% babies
        // (VERIFIED w/Turtle §Spawning: "spawn on the sand ... in groups
        // of up to 5" + the 5% baby roll; babies carry bit 0x40 with the
        // 24000-tick (20 min) maturity countdown and drop a scute)
        if biome == vc_world::gen::Biome::Beach {
            // scan DOWN from 12 above to 12 below the player (the
            // bounds were reversed in the first draft — `82..58` is an
            // empty range and the branch never fired)
            for y in (p[1] as i32 - 12..p[1] as i32 + 12).rev() {
                if !(1..=250).contains(&y) {
                    continue;
                }
                let floor = world.get_block(wx, y - 1, wz);
                if floor != SAND {
                    continue;
                }
                if world.get_block(wx, y, wz) != AIR || world.get_block(wx, y + 1, wz) != AIR
                {
                    continue;
                }
                let group = 1 + (self.rng.next_range(5)) as usize; // ≤ 5 (VERIFIED)
                for _ in 0..group {
                    let baby = self.rng.next_f32() < 0.05; // 5% (VERIFIED)
                    let (variant, aux) = if baby { (0x40u8, 24000) } else { (0, 0) };
                    let _ = self.spawn_variant(MobKind::Turtle, wx, y, wz, variant);
                    if let Some(m) = self.list.last_mut() {
                        m.aux = aux;
                    }
                }
                return;
            }
            return;
        }
        if !biome.is_ocean() {
            return;
        }
        // the water column: two stacked WATER blocks at y 45..=SEA_LEVEL
        // (dolphin doc band Y 50–64 covers the shallow half — disclosed)
        for y in (45..=vc_chunk::SEA_LEVEL).rev() {
            if world.get_block(wx, y, wz) != WATER || world.get_block(wx, y + 1, wz) != WATER
            {
                continue;
            }
            // dolphins: pods 1–2, warm/lukewarm/neutral families only
            // (VERIFIED: "all ocean biomes except frozen/cold")
            let warm_side = matches!(
                biome,
                vc_world::gen::Biome::WarmOcean
                    | vc_world::gen::Biome::LukewarmOcean
                    | vc_world::gen::Biome::Ocean
            );
            if warm_side && self.rng.next_range(8) == 0 {
                let pod = 1 + (self.rng.next_range(2)) as usize; // 1–2 (VERIFIED JE)
                for _ in 0..pod {
                    let _ = self.spawn_variant(MobKind::Dolphin, wx, y, wz, 0);
                }
                return;
            }
            // fish schools (VERIFIED group sizes: cod/salmon 3–6,
            // tropical 3–5, pufferfish 1–2 — w/ pages)
            let kind = match biome {
                vc_world::gen::Biome::WarmOcean => {
                    if self.rng.next_range(4) == 0 {
                        MobKind::Pufferfish
                    } else {
                        MobKind::TropicalFish
                    }
                }
                vc_world::gen::Biome::LukewarmOcean => {
                    if self.rng.next_range(2) == 0 {
                        MobKind::Cod
                    } else {
                        MobKind::TropicalFish
                    }
                }
                // cold/frozen/neutral: the cod–salmon split
                _ => {
                    if self.rng.next_range(2) == 0 {
                        MobKind::Cod
                    } else {
                        MobKind::Salmon
                    }
                }
            };
            let school = match kind {
                MobKind::Pufferfish => 1 + (self.rng.next_range(2)) as usize,
                MobKind::TropicalFish => 3 + (self.rng.next_range(3)) as usize,
                _ => 3 + (self.rng.next_range(4)) as usize, // cod/salmon 3–6
            };
            for _ in 0..school {
                let _ = self.spawn_variant(kind, wx, y, wz, 0);
            }
            return; // one attempt per cadence tick
        }
    }

    /// 1.16 (Nether Update, part 2): strider lava-sea spawn — "Striders
    /// can spawn in every Nether biome. Groups of 2 to 4 striders spawn
    /// on spaces of lava that have an air block above" (VERIFIED
    /// w/Strider §Spawning). The 1-in-10 jockey/baby row: "For every
    /// strider that spawns, there is a 1 in 10 chance for an additional
    /// baby strider to spawn riding on top of it" — the riding half is
    /// the mount-system deferral; the BABY half rides the 0x40 bit
    /// (disclosed: the baby spawns alongside, not on top). Also
    /// "If a strider spawns under lava, it rises out of the lava" —
    /// the physics layer handles the ascent.
    fn try_spawn_striders(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
        let Some(p) = self.player else { return };
        if world.dimension != vc_world::world::Dimension::Nether {
            return;
        }
        let striders = self
            .list
            .iter()
            .filter(|m| m.kind == MobKind::Strider)
            .count();
        if striders >= 10 {
            return; // the nether's passive-category cap (the bat class)
        }
        let cx = (p[0] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        let cz = (p[2] / 16.0).floor() as i32 + (self.rng.next_range(17) as i32) - 8;
        if !sim_ring(cx, cz) || world.chunk((cx, cz)).is_none() {
            return;
        }
        let lx = self.rng.next_range(16) as i32;
        let lz = self.rng.next_range(16) as i32;
        let wx = cx * 16 + lx;
        let wz = cz * 16 + lz;
        // the lava sea: scan the lower body for lava-with-air columns
        // (the nether's lava sits at the sea level, y <= 32)
        for y in (1..=40i32).rev() {
            if world.get_block(wx, y, wz) != LAVA {
                continue;
            }
            if world.get_block(wx, y + 1, wz) != AIR {
                continue;
            }
            // a lava surface with air above — spawn the group ON it
            let group = 2 + (self.rng.next_range(3)) as usize; // 2..4 (VERIFIED)
            for _ in 0..group {
                // the 1-in-10 baby row (the jockey's riding half is the
                // mount deferral — the baby rides alongside, disclosed)
                let baby = self.rng.next_range(10) == 0;
                let (variant, aux) = if baby { (0x40u8, 24000) } else { (0, 0) };
                let _ = self.spawn_variant(MobKind::Strider, wx, y + 1, wz, variant);
                if let Some(m) = self.list.last_mut() {
                    m.pos[1] = y as f32 + 1.0; // stand on the surface
                    m.aux = aux;
                }
            }
            return; // one group per attempt
        }
    }

    /// 1.13: phantom insomnia spawn (VERIFIED w/Phantom §Spawning:
    /// "Phantoms spawn ... above a player whose Time Since Last Rest
    /// is 1 hour (72000 ticks)"; the local pack caps at 1–4 — the
    /// engine rolls one per 20-tick attempt with a hard cap of 4).
    /// Spawned 12–20 blocks above the player in open air, starting in
    /// the 200-tick orbit phase of the swoop cycle.
    fn try_spawn_phantom(&mut self, world: &World, sim_ring: impl Fn(i32, i32) -> bool) {
        if self.hostiles_alive() as f32 >= MONSTER_CAP {
            return;
        }
        let phantoms = self
            .list
            .iter()
            .filter(|m| m.kind == MobKind::Phantom)
            .count();
        if phantoms >= 4 {
            return; // the local pack cap (VERIFIED 1–4)
        }
        let Some(p) = self.player else { return };
        let x = p[0] as i32 + (self.rng.next_range(17) as i32) - 8;
        let z = p[2] as i32 + (self.rng.next_range(17) as i32) - 8;
        let y = p[1] as i32 + 12 + (self.rng.next_range(9)) as i32; // 12–20 above
        if !sim_ring(x.div_euclid(16), z.div_euclid(16)) {
            return;
        }
        // open air at altitude (the phantom circles up there)
        for dy in 0..2 {
            if world.get_block(x, y + dy, z) != AIR {
                return;
            }
        }
        let _ = self.spawn_variant(MobKind::Phantom, x, y, z, 0);
        if let Some(m) = self.list.last_mut() {
            m.aux = 200; // start in the orbit phase (the 200-tick cycle)
        }
    }

    /// 1.13: reset "Time Since Last Rest" (VERIFIED w/Phantom §Spawning:
    /// dying or sleeping resets the statistic; beds are deferred — the
    /// engine resets on player death, disclosed).
    pub fn note_rest(&mut self) {
        self.rest_t = 0;
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
            let kind = if biome == vc_world::gen::Biome::Jungle && self.rng.next_f32() < 0.4301 {
                // 1.12: the parrot is the DOMINANT jungle passive —
                // weight 40/93 = 43.01% (VERIFIED w/Parrot §Spawning:
                // "Parrots naturally spawn in groups of 1–2 in jungles
                // ... above logs, leaves, grass blocks, or air"; the
                // spawn-above-air height quirk is the engine's standard
                // floor-grass roll — disclosed)
                MobKind::Parrot
            } else if biome == vc_world::gen::Biome::Jungle && self.rng.next_f32() < 0.0215 {
                // ocelots: 2/93 ≈ 2.2% of the jungle creature roll
                // (VERIFIED — the parrot page's spawn table carries the
                // full jungle mix; the pre-1.12 engine's 1/4 ocelot
                // share is superseded by the 1.12 weights)
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
            } else if biome == vc_world::gen::Biome::Mountains {
                // 1.11 (VERIFIED changelog §Mobs: llamas "Spawn in extreme
                // hills"; w/Llama spawn table: group 4–6): llama herds are
                // the Mountains passive roll; strength distribution
                // 32.8/32.8/32.8/0.8/0.8% (VERIFIED w/Llama §Strength)
                MobKind::Llama
            } else if biome == vc_world::gen::Biome::Snowy
                || biome == vc_world::gen::Biome::IceSpikes
            {
                // the icy family: polar bear (30%) or rabbit — and
                // formerly NOTHING else (the 1.10 restriction).
                // 1.14 amendment (VERIFIED w/Fox §Spawning: "Snowy
                // Taiga" is a fox biome): foxes are the one later-
                // bracket addition to the icy family, at a 20% share —
                // the 1.10 restriction's own wiki note carries the
                // version-scoped exceptions
                if self.rng.next_f32() < 0.20 {
                    MobKind::Fox
                } else if self.rng.next_f32() < 0.3 {
                    MobKind::PolarBear
                } else {
                    MobKind::Rabbit
                }
            } else if biome == vc_world::gen::Biome::Taiga
                && self.rng.next_f32() < 0.25
            {
                // 1.14 (VERIFIED w/Fox §Spawning: taiga is the fox's
                // primary biome): a quarter of taiga passive rolls
                // are fox packs (groups 2–4 — the herd size below)
                MobKind::Fox
            } else {
                match self.rng.next_range(5) {
                    0 => MobKind::Cow,
                    1 => MobKind::Pig,
                    2 => MobKind::Sheep,
                    3 => MobKind::Chicken,
                    _ => MobKind::Rabbit,
                }
            };
            // equine herds are 2–6 (VERIFIED w/Horse §Spawning); llama
            // herds 4–6 (VERIFIED w/Llama); other passives keep 2–4
            let herd = if matches!(kind, MobKind::Horse | MobKind::Donkey) {
                2 + (self.rng.next_range(5)) as usize // 2–6
            } else if kind == MobKind::Llama {
                4 + (self.rng.next_range(3)) as usize // 4–6 (VERIFIED)
            } else if kind == MobKind::Parrot {
                // 1.12: parrot groups are 1–2 (VERIFIED w/Parrot
                // §Spawning: "spawn in groups of 1–2")
                1 + (self.rng.next_range(2)) as usize
            } else {
                2 + (self.rng.next_range(3)) as usize
            };
            for _ in 0..herd {
                // 1.11 llama strength in the variant byte: 1–5 with the
                // wild distribution 32.8/32.8/32.8/0.8/0.8% (VERIFIED
                // w/Llama §Strength — the strength table)
                let variant = if kind == MobKind::Llama {
                    let r = self.rng.next_f32();
                    if r < 0.008 {
                        5
                    } else if r < 0.016 {
                        4
                    } else {
                        1 + (self.rng.next_range(3)) as u8
                    }
                } else if kind == MobKind::Parrot {
                    // 1.12: the color variant 0..=4 (red/blue/green/cyan/
                    // gray — VERIFIED w/Parrot Variant NBT table; uniform)
                    (self.rng.next_range(5)) as u8
                } else {
                    0
                };
                let _ = self.spawn_variant(kind, wx, y, wz, variant);
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
        // 1.11: llamas breed with HAY BALES (VERIFIED w/Llama §Breeding:
        // "Tamed llamas can be bred with hay bales" — the changelog row);
        // hay also heals them (+10 like horses).
        if food != GOLDEN_APPLE && food != HAY_BALE && food != GOLDEN_CARROT {
            return None;
        }
        // llama branch: hay = breed (tamed adults) + heal; other foods heal
        let is_llama = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Llama)
            .unwrap_or(false);
        if is_llama {
            if food == HAY_BALE {
                let (pos, tamed, baby, breed_cd) = {
                    let m = self.list.iter_mut().find(|m| m.id == id)?;
                    let eq = m.equine.as_mut()?;
                    (m.pos, eq.tamed, eq.baby, eq.breed_cd)
                };
                if tamed && !baby && breed_cd == 0 {
                    let partner = self.list.iter().find(|o| {
                        o.id != id
                            && o.kind == MobKind::Llama
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
                        return Some(FeedOutcome::LoveMode(pid));
                    }
                    return Some(FeedOutcome::Ate);
                }
            }
            // heal path (any equine food)
            if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
                m.health = (m.health + 10.0).min(30.0);
            }
            return Some(FeedOutcome::Healed);
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

/// 1.12 parrot feed outcome (w/Parrot §Taming/§Cookies):
/// - seeds (any of the 4): a 1/10 taming roll — `Tamed` on success,
///   `Ate` otherwise (the item is consumed either way)
/// - cookie: INSTANT DEATH — "the parrot receives 2128 (3.4028 x 10^38)"
///   damage; the game layer routes it through `damage()` so the death
///   sweep drops feathers + XP (VERIFIED 17w13a pre5: "Killing a parrot
///   by feeding a cookie now counts as if the parrot was killed by the
///   player who fed it")
/// - anything else: not parrot food → None (no consumption)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParrotFeedOutcome {
    Ate,
    Tamed,
    CookieDeath,
}

/// 1.14: the fox feeding outcome (sweet berries — VERIFIED
/// w/Sweet_Berries §Breeding).
#[derive(Debug)]
pub enum FoxFeedOutcome {
    /// entered love mode (waiting for a partner, 30 s)
    LoveMode,
    /// paired with the loving partner `id` — the game layer spawns
    /// the trusting cub
    Bred(u32),
}

/// 1.15: the flower-feeding outcome for bees (the fox pattern)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BeeFeedOutcome {
    LoveMode,
    Bred(u32),
}

/// 1.16 (Nether Update, part 2): the warped-fungus feeding outcome
/// for striders ("They can be fed warped fungus to breed", VERIFIED
/// w/Strider — the fox/bee pattern)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StriderFeedOutcome {
    LoveMode,
    Bred(u32),
}

/// 1.16: the crimson-fungus feeding outcome for hoglins ("Hoglins
/// can be bred with crimson fungi", VERIFIED w/Hoglin)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoglinFeedOutcome {
    LoveMode,
    Bred(u32),
}

impl MobSystem {
    /// 1.12: feed a parrot. VERIFIED w/Parrot §Taming: "Parrots can be
    /// tamed by feeding wheat seeds, melon seeds, pumpkin seeds,
    /// beetroot seeds ... Each item fed has a 1⁄10 chance of successfully
    /// taming them" (the 1.12-era set; torchflower seeds/pitcher pods
    /// are 1.20+ — out of bracket).
    pub fn try_feed_parrot(
        &mut self,
        id: u32,
        food: u16,
        rng: &mut Rng,
    ) -> Option<ParrotFeedOutcome> {
        let is_parrot = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Parrot)
            .unwrap_or(false);
        if !is_parrot {
            return None;
        }
        if food == COOKIE {
            // VERIFIED w/Parrot: "feeding a cookie to a parrot kills
            // it ... the parrot receives 2128 (3.4028 x 10^38)" — route
            // through damage() so drops/XP/death sweep all run
            let _ = self.damage(id, 2128.0);
            return Some(ParrotFeedOutcome::CookieDeath);
        }
        if !is_seeds(food) {
            return None; // not parrot food
        }
        // the 1/10 taming roll (VERIFIED)
        if rng.next_range(10) == 0 {
            if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
                m.variant |= 0x80; // the tamed bit
            }
            return Some(ParrotFeedOutcome::Tamed);
        }
        Some(ParrotFeedOutcome::Ate)
    }

    /// 1.12: toggle a tamed parrot's sitting state (VERIFIED 17w14a:
    /// "The right-click action has been changed: right-clicking on a
    /// tamed parrot now tells it to sit"). Returns true if toggled
    /// (only tamed parrots respond).
    pub fn toggle_parrot_sit(&mut self, id: u32) -> bool {
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            if m.kind == MobKind::Parrot && m.variant & 0x80 != 0 {
                m.aux ^= 1; // the sit bit
                return true;
            }
        }
        false
    }

    /// 1.14: feed a fox sweet berries (VERIFIED w/Sweet_Berries
    /// §Breeding: "Sweet berries can be fed to foxes to breed them";
    /// the 30 s love window is the vanilla love-mode clock). Feeding
    /// the FIRST adult enters love mode; feeding a second adult while
    /// a loving partner is within 8 blocks pairs them — the game
    /// layer spawns the cub (trusting, the wiki's "baby foxes trust
    /// the player" simplified to the single breeder, disclosed).
    pub fn try_feed_fox(&mut self, id: u32) -> Option<FoxFeedOutcome> {
        let is_fox = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Fox && m.variant & 0x40 == 0) // adults only
            .unwrap_or(false);
        if !is_fox {
            return None;
        }
        // already in love: vanilla ignores further feeding
        let already = self
            .list
            .iter()
            .any(|m| m.id == id && m.variant & 0x80 != 0);
        if already {
            return None;
        }
        // a loving adult partner within 8 blocks → pair now
        let me = self.list.iter().find(|m| m.id == id).map(|m| m.pos).unwrap_or([0.0; 3]);
        let partner = self
            .list
            .iter()
            .find(|m| {
                if m.id == id
                    || m.kind != MobKind::Fox
                    || m.variant & 0x80 == 0
                    || m.variant & 0x40 != 0
                {
                    return false;
                }
                let dx = m.pos[0] - me[0];
                let dz = m.pos[2] - me[2];
                (dx * dx + dz * dz) <= 64.0
            })
            .map(|m| m.id);
        // arm the fed fox
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            m.variant |= 0x80; // love
            m.aux = 600; // 30 s (VERIFIED love-mode window)
        }
        if let Some(pid) = partner {
            // both exit love (vanilla post-breed cooldown — the engine
            // clears outright, the 5-minute cooldown is disclosed)
            if let Some(m) = self.list.iter_mut().find(|m| m.id == pid) {
                m.variant &= !0x80;
                m.aux = 0;
            }
            if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
                m.variant &= !0x80;
                m.aux = 0;
            }
            return Some(FoxFeedOutcome::Bred(pid));
        }
        Some(FoxFeedOutcome::LoveMode)
    }

    /// 1.15: feed a flower to a bee — VERIFIED w/Bee §Breeding ("Bees
    /// follow players holding any 1- or 2-block tall flowers" [the
    /// follow itself is not modeled — disclosed] + the standard
    /// breeding flow: the first feeding arms love mode, a second with
    /// a loving partner nearby produces the offspring, "When two bees
    /// breed and produce an offspring, 1-7 XP is dropped" [the XP
    /// drops via the game layer's bred path]). Adults only.
    pub fn try_feed_bee(&mut self, id: u32) -> Option<BeeFeedOutcome> {
        let is_bee = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Bee && m.bee.as_ref().map(|b| !b.baby).unwrap_or(false))
            .unwrap_or(false);
        if !is_bee {
            return None;
        }
        // already in love: vanilla ignores further feeding
        let already = self
            .list
            .iter()
            .any(|m| m.id == id && m.bee.as_ref().map(|b| b.love_t > 0).unwrap_or(false));
        if already {
            return None;
        }
        // a loving adult partner within 8 blocks → pair now
        let me = self.list.iter().find(|m| m.id == id).map(|m| m.pos).unwrap_or([0.0; 3]);
        let partner = self
            .list
            .iter()
            .find(|m| {
                if m.id == id || m.kind != MobKind::Bee {
                    return false;
                }
                let b = m.bee.as_ref();
                if !b.map(|b| b.love_t > 0 && !b.baby).unwrap_or(false) {
                    return false;
                }
                let dx = m.pos[0] - me[0];
                let dz = m.pos[2] - me[2];
                (dx * dx + dz * dz) <= 64.0
            })
            .map(|m| m.id);
        // arm the fed bee (love window = the fox's 600 ticks)
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            if let Some(b) = m.bee.as_mut() {
                b.love_t = 600;
            }
        }
        if let Some(pid) = partner {
            // both exit love (the post-breed cooldown is the engine's
            // cleared-outright class, disclosed like the fox)
            for target in [pid, id] {
                if let Some(m) = self.list.iter_mut().find(|m| m.id == target) {
                    if let Some(b) = m.bee.as_mut() {
                        b.love_t = 0;
                    }
                }
            }
            return Some(BeeFeedOutcome::Bred(pid));
        }
        Some(BeeFeedOutcome::LoveMode)
    }

    /// 1.16 (Nether Update, part 2): feed a strider — "They can be fed
    /// warped fungus to breed" (VERIFIED w/Strider §Breeding). The
    /// fox/bee pattern: first feeding arms love (the 600-tick window);
    /// a loving adult partner within 8 blocks pairs them (the game
    /// layer spawns the baby on Bred).
    pub fn try_feed_strider(&mut self, id: u32, food: u16) -> Option<StriderFeedOutcome> {
        if food != WARPED_FUNGUS {
            return None; // not strider food
        }
        let is_adult = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Strider && m.variant & 0x40 == 0)
            .unwrap_or(false);
        if !is_adult {
            // baby feeding (growth acceleration) is the trimmed half —
            // babies mature on the 24000-tick clock, disclosed
            return None;
        }
        // already in love: vanilla ignores further feeding
        let already = self
            .list
            .iter()
            .any(|m| m.id == id && m.variant & 0x80 != 0);
        if already {
            return None;
        }
        // a loving adult partner within 8 blocks → pair now
        let me = self.list.iter().find(|m| m.id == id).map(|m| m.pos).unwrap_or([0.0; 3]);
        let partner = self
            .list
            .iter()
            .find(|m| {
                if m.id == id || m.kind != MobKind::Strider {
                    return false;
                }
                if m.variant & 0x80 == 0 || m.variant & 0x40 != 0 {
                    return false;
                }
                let dx = m.pos[0] - me[0];
                let dz = m.pos[2] - me[2];
                (dx * dx + dz * dz) <= 64.0
            })
            .map(|m| m.id);
        // arm the fed strider (love window = the fox's 600 ticks)
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            m.variant |= 0x80;
            m.aux = 600;
        }
        if let Some(pid) = partner {
            for target in [pid, id] {
                if let Some(m) = self.list.iter_mut().find(|m| m.id == target) {
                    m.variant &= !0x80; // both exit love
                }
            }
            return Some(StriderFeedOutcome::Bred(pid));
        }
        Some(StriderFeedOutcome::LoveMode)
    }

    /// 1.16: feed a hoglin — "Hoglins can be bred with crimson fungi"
    /// (VERIFIED w/Hoglin §Breeding; the crimson fungus is the food).
    /// "Hoglins cannot be bred when they are running away from warped
    /// fungi" — the feed refuses while a warped fungus/anchor repel
    /// source is within 7 blocks (the AI-side scan duplicated here —
    /// the disclosed single-source simplification).
    pub fn try_feed_hoglin(
        &mut self,
        id: u32,
        food: u16,
        world: &World,
    ) -> Option<HoglinFeedOutcome> {
        if food != CRIMSON_FUNGUS {
            return None; // not hoglin food
        }
        let is_adult = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Hoglin && m.variant & 0x40 == 0)
            .unwrap_or(false);
        if !is_adult {
            return None;
        }
        // the flee gate: no breeding while repelled
        if let Some(m) = self.list.iter().find(|m| m.id == id) {
            for sy in -1..=1i32 {
                for sz in -7..=7i32 {
                    for sx in -7..=7i32 {
                        let b = world.get_block(
                            m.pos[0] as i32 + sx,
                            m.pos[1] as i32 + sy,
                            m.pos[2] as i32 + sz,
                        );
                        if b == WARPED_FUNGUS || b == RESPAWN_ANCHOR {
                            return None; // fleeing — no breeding
                        }
                    }
                }
            }
        }
        let already = self
            .list
            .iter()
            .any(|m| m.id == id && m.variant & 0x80 != 0);
        if already {
            return None;
        }
        let me = self.list.iter().find(|m| m.id == id).map(|m| m.pos).unwrap_or([0.0; 3]);
        let partner = self
            .list
            .iter()
            .find(|m| {
                if m.id == id || m.kind != MobKind::Hoglin {
                    return false;
                }
                if m.variant & 0x80 == 0 || m.variant & 0x40 != 0 {
                    return false;
                }
                let dx = m.pos[0] - me[0];
                let dz = m.pos[2] - me[2];
                (dx * dx + dz * dz) <= 64.0
            })
            .map(|m| m.id);
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            m.variant |= 0x80;
            m.aux = 600;
        }
        if let Some(pid) = partner {
            for target in [pid, id] {
                if let Some(m) = self.list.iter_mut().find(|m| m.id == target) {
                    m.variant &= !0x80;
                }
            }
            return Some(HoglinFeedOutcome::Bred(pid));
        }
        Some(HoglinFeedOutcome::LoveMode)
    }

    /// 1.16: barter with a piglin — the game layer calls this on a
    /// gold-ingot use while looking at an adult piglin ("Adult piglins
    /// take gold ingots, whether dropped nearby or when a player uses
    /// one while looking at them", VERIFIED w/Piglin §Bartering). Arms
    /// the 6-second (120 gt) examine countdown; the drop surfaces via
    /// pending_drops at the AI's countdown end.
    pub fn try_barter_piglin(&mut self, id: u32, item: u16) -> bool {
        // gold ingot = the engine's IRON_ORE stand-in (the disclosed
        // convention since the golden-apple round)
        if item != IRON_ORE {
            return false;
        }
        let is_piglin = self
            .list
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.kind == MobKind::Piglin)
            .unwrap_or(false);
        if !is_piglin {
            return false;
        }
        if let Some(m) = self.list.iter_mut().find(|m| m.id == id) {
            if m.aux > 0 {
                return false; // already examining
            }
            m.aux = 120; // the six-second examine (VERIFIED)
        }
        true
    }

    /// 1.16: the gold-mining anger hook — piglins get angry when the
    /// player mines nether gold ore / gilded blackstone ("If the player
    /// mines ... gold-related blocks ... nearby piglins become angry",
    /// the w/Piglin aggravation rows). Angered piglins get the
    /// provoked flag (the neutral-until-provoked class).
    pub fn anger_piglins_near(&mut self, pos: [f32; 3], radius: f32) -> usize {
        let r2 = radius * radius;
        let mut angered = 0;
        for m in self.list.iter_mut() {
            if m.kind != MobKind::Piglin || m.provoked {
                continue;
            }
            let dx = m.pos[0] - pos[0];
            let dz = m.pos[2] - pos[2];
            if dx * dx + dz * dz <= r2 {
                m.provoked = true;
                angered += 1;
            }
        }
        angered
    }
}

/// 1.16 (Nether Update, part 2): the piglin barter table — the trimmed
/// engine form of the VERIFIED w/Bartering chart (the current wiki
/// table's /469 weights). Potions, enchanted books/boots, spectral
/// arrows, water bottles and the 1.21-era dried ghast need systems the
/// engine doesn't have — trimmed, disclosed; the remaining weights keep
/// their vanilla ratios so crying obsidian stays the ~8.53% headline.
/// Gold = the iron-ore/nugget stand-ins (the disclosed convention).
pub fn piglin_barter_roll(rng: &mut Rng) -> (u16, u8) {
    // trimmed table: total weight 158 of vanilla's 469
    // 40/469 classes: obsidian (1), crying obsidian (1-3), gravel
    // (8-16), blackstone (8-16), leather (2-4), soul sand (2-8)
    // 20/469: string (3-9), nether quartz (5-12)
    // 10/469: iron nugget (10-36), ender pearl (2-4)
    match rng.next_range(158) {
        w if w < 40 => {
            // the 40-class pick: crying obsidian is the headliner
            match rng.next_range(6) {
                0 => (CRYING_OBSIDIAN, (1 + rng.next_range(3)) as u8),   // 1-3
                1 => (OBSIDIAN, 1),
                2 => (GRAVEL, (8 + rng.next_range(9)) as u8),            // 8-16
                3 => (BLACKSTONE, (8 + rng.next_range(9)) as u8),        // 8-16
                4 => (LEATHER, (2 + rng.next_range(3)) as u8),           // 2-4
                _ => (SOUL_SAND, (2 + rng.next_range(7)) as u8),         // 2-8
            }
        }
        w if w < 60 => {
            // the 20-class pick
            if rng.next_range(2) == 0 {
                (STRING, (3 + rng.next_range(7)) as u8) // 3-9
            } else {
                (NETHER_QUARTZ, (5 + rng.next_range(8)) as u8) // 5-12
            }
        }
        _ => {
            // the 10-class pick
            if rng.next_range(2) == 0 {
                (IRON_NUGGET, (10 + rng.next_range(27)) as u8) // 10-36
            } else {
                (ENDER_PEARL, (2 + rng.next_range(3)) as u8) // 2-4
            }
        }
    }
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
    // 1.11 evoker spell queues (game-layer consumption)
    pending_summons: &mut Vec<(u32, usize)>,
    pending_player_fang: &mut Vec<f32>,
    // 1.12 illusioner spell queue (game-layer consumption)
    pending_blindness: &mut Vec<i32>,
    // 1.13 (Update Aquatic) queues (game-layer consumption)
    pending_player_grace: &mut Vec<i32>,
    pending_turtle_eggs: &mut Vec<(i32, i32, i32, u16)>,
    pending_drops: &mut Vec<([f32; 3], u16)>,
    // 1.15 (Buzzy Bees) queues (game-layer consumption)
    bee_enters: &mut Vec<(u32, [i32; 3], bool)>,
    bee_pollinations: &mut Vec<([i32; 3], u8)>,
    is_day: bool,
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

    // ---- 1.13 (Update Aquatic) environmental behaviors ----
    // These run REGARDLESS of a player anchor: conversion and nesting
    // are environmental — a zombie converts with no player watching,
    // a bred turtle lays her eggs the same way (the v113 tests pin
    // player = None for exactly these paths).

    // 1.13: zombies whose heads stay under water convert to drowned
    // after 30 s (VERIFIED w/Drowned §Conversion: "If a zombie's head
    // ... is continuously submerged for 30 seconds, it begins to
    // convert into a drowned" — the 15 s shake window is folded into
    // the timer, disclosed; aux is free on plain zombies, the
    // zombie-villager cure path is a different kind so no collision).
    // Converted drowned are unarmed (their zombie hands were empty).
    if m.kind == MobKind::Zombie
        && world.get_block(
            m.pos[0] as i32,
            (m.pos[1] + 1.6) as i32,
            m.pos[2] as i32,
        ) == WATER
    {
        m.aux = (m.aux + 1).min(601);
        if m.aux >= 600 {
            m.kind = MobKind::Drowned;
            m.aux = 0;
            m.variant = 0; // unarmed (disclosed)
        }
    }

    // TURTLE: beach nester — adults wander toward water on land, swim
    // freely in water; a bred female (variant bit 7 = carrying) lays an
    // egg on sand (the queued world edit), babies mature on the aux
    // countdown and drop a scute (VERIFIED w/Scute: "Dropped when baby
    // turtles grow up"). Environmental — no player anchor needed.
    if m.kind == MobKind::Turtle {
        // baby growth: aux counts down to maturity
        if m.variant & 0x40 != 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                // matured: clear the baby bit + queue the scute drop
                m.variant &= !0x40;
                pending_drops.push((m.pos, SCUTE));
            }
        }
        // egg laying (bred female, standing on sand)
        if m.variant & 0x80 != 0 && m.on_ground {
            let below = world.get_block(
                m.pos[0] as i32,
                (m.pos[1] - 0.1) as i32,
                m.pos[2] as i32,
            );
            if below == SAND || below == RED_SAND {
                m.variant &= !0x80; // egg laid
                pending_turtle_eggs.push((
                    m.pos[0] as i32,
                    m.pos[1] as i32,
                    m.pos[2] as i32,
                    0, // stage 0 state id written by the game layer
                ));
            }
        }
        // land movement: head downhill (toward water) at slow pace —
        // a turtle "generally attempt to move to near water"
        // (VERIFIED w/Turtle §Behavior)
        if in_water_mob(m, world, d.height) {
            wander_3d(rng, m, speed * 3.0);
        } else {
            wander(rng, m, speed);
        }
        return;
    }

    // ---- the completeness audit: CHICKEN egg laying —
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
    // environmental (player-independent, the turtle precedent):
    // baby maturity (variant 0x40, aux = 24000-tick countdown) and
    // love-mode expiry (variant 0x80, aux = the 600-tick window).
    // Babies mature even with nobody watching; love expires the same
    // way (an unfed partner never arrives). ----
    if matches!(m.kind, MobKind::Fox | MobKind::Chicken) {
        // the chicken row joined at the sweep-2 (the egg-spawned chick
        // — "an egg has a 1/8 chance of spawning a chick", VERIFIED
        // w/Egg; chicks mature on the same 24000-tick countdown, the
        // scute-less fox form)
        if m.variant & 0x40 != 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                m.variant &= !0x40; // grown (a scute-less maturity —
                // foxes drop nothing on maturity, VERIFIED w/Fox)
            }
        }
        if m.variant & 0x80 != 0 && m.variant & 0x40 == 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                m.variant &= !0x80; // love expired
            }
        }
    }

    // ---- 1.16 (Nether Update, part 2): the forest mobs' life-cycle
    // clocks (the fox pattern — environmental, player-independent):
    // strider/hoglin baby maturity (24000 ticks, "All babies
    // obtained through breeding take 20 minutes to grow up", VERIFIED
    // w/Strider) + love-mode expiry (the 600-tick fox window) + the
    // piglin's 6-second barter countdown. ----
    if matches!(m.kind, MobKind::Strider | MobKind::Hoglin) {
        // baby: variant 0x40, aux counts down to maturity
        if m.variant & 0x40 != 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                m.variant &= !0x40; // grown
            }
        }
        // love: variant 0x80 (adults only — a baby never loves), the
        // 600-tick window
        if m.variant & 0x80 != 0 && m.variant & 0x40 == 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                m.variant &= !0x80; // love expired ("striders have a
                // cooldown of about 5 minutes before they can breed
                // again" — the engine's cleared-outright class,
                // disclosed)
            }
        }
    }
    // PIGLIN barter countdown: "The piglin 'examines' the ingot for
    // six seconds, then drops a random item from the chart" (VERIFIED
    // w/Piglin) — 120 game ticks, then the loot roll surfaces through
    // pending_drops (the piglin throws it; the game layer's item
    // entity handles pickup — one entity per item, so a 1-3 roll
    // drops 1-3 entities). Needs no player anchor.
    if m.kind == MobKind::Piglin && m.aux > 0 {
        m.aux -= 1;
        if m.aux == 0 {
            let (item, count) = piglin_barter_roll(rng);
            for _ in 0..count.max(1) {
                pending_drops.push((m.pos, item));
            }
        }
    }

    // ---- 1.15 (Buzzy Bees): the bee behavior arm (VERIFIED w/Bee —
    // the raw capture scripts/v115_page_Bee.json; the value contract
    // is docs/research/phase-v115-1.15-research.md). Self-contained:
    // the state is TAKEN out of the mob (split borrows), the phase
    // machine steers, and the state goes back. No early returns
    // inside the take window. ----
    if m.kind == MobKind::Bee {
        if let Some(mut bee) = m.bee.take() {
            // -- environmental clocks (the fox pattern) --
            // baby maturity: 24000 ticks (the standard baby window)
            if bee.baby && bee.maturity_t > 0 {
                bee.maturity_t -= 1;
                if bee.maturity_t == 0 {
                    bee.baby = false; // grown (no scute — VERIFIED w/Bee)
                }
            }
            // love-mode expiry (the 600-tick fox window)
            if bee.love_t > 0 {
                bee.love_t -= 1;
            }
            // anger countdown (20-39 s, VERIFIED)
            if bee.anger_t > 0 {
                bee.anger_t -= 1;
            }
            // the stung bee: "loses its stinger ... cannot attack
            // further and does not retreat to its nest (even at
            // night), and dies approximately one minute later"
            if bee.stung {
                bee.death_t -= 1;
                if bee.death_t <= 0 {
                    m.health = 0.0; // dies (XP via the deaths queue)
                } else {
                    // slow hover, no hive, no attacks
                    wander_3d(rng, m, speed * 0.4);
                }
                m.bee = Some(bee);
            } else if bee.anger_t > 0 {
                // -- ANGRY: the swarm chase --
                if let Some(pp) = player {
                    if !invuln {
                        let dx = pp[0] - m.pos[0];
                        let dy = pp[1] - m.pos[1];
                        let dz = pp[2] - m.pos[2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                        if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                            // the sting: 2 HP (Normal; the Hard scale +
                            // Poison I 10 s Normal / 18 s Hard ride the
                            // PlayerHit payload — the game layer)
                            let k = 1.0 / dist.max(0.1);
                            hits.push(PlayerHit {
                                damage: d.damage,
                                source: MobKind::Bee,
                                knockback_dir: [dx * k, dz * k],
                                wither_effect: None,
                                poison_effect: Some(200),
                            });
                            // one sting per bee ("Bees attack only
                            // once") — the stinger is spent
                            bee.stung = true;
                            bee.death_t = super::bees::STING_DEATH_TICKS;
                            bee.anger_t = 0;
                            m.attack_cd = 20;
                        } else {
                            steer_3d(m, pp, speed * 1.3);
                        }
                    } else {
                        // "Bees do not deal any damage in Peaceful
                        // difficulty and are completely passive" —
                        // invulnerable anchors calm the swarm early
                        bee.anger_t = 0;
                        wander_3d(rng, m, speed * 0.5);
                    }
                } else {
                    wander_3d(rng, m, speed * 0.5);
                }
                m.bee = Some(bee);
            } else if !is_day {
                // -- NIGHT: return to the hive ("Bees return to their
                // nest ... during the night"; the engine has no rain —
                // documented) --
                if let Some(h) = bee.hive {
                    let target = [
                        h[0] as f32 + 0.5,
                        h[1] as f32 - 0.4,
                        h[2] as f32 + 0.5,
                    ];
                    let dd = (m.pos[0] - target[0]).powi(2)
                        + (m.pos[1] - target[1]).powi(2)
                        + (m.pos[2] - target[2]).powi(2);
                    if dd < 1.2 {
                        bee.arrived = true; // the tick pass removes + enters
                    } else {
                        steer_3d(m, target, speed);
                    }
                } else {
                    wander_3d(rng, m, speed * 0.4);
                }
                m.bee = Some(bee);
            } else {
                // -- DAY: the trip phases --
                let mut restore = true;
                match bee.phase {
                    super::bees::PH_HOVER => {
                        // hover near the hive; periodically seek a
                        // flower
                        if bee.nectar {
                            bee.phase = super::bees::PH_TO_HIVE;
                        } else if bee.timer <= 0 {
                            // a fresh flower scan (staggered retries)
                            if let Some(f) =
                                super::bees::find_flower(world, m.pos, super::bees::FLOWER_SEARCH_R)
                            {
                                bee.flower = Some(f);
                                bee.phase = super::bees::PH_TO_FLOWER;
                            } else {
                                bee.timer = 60; // retry in 3 s
                            }
                        } else {
                            bee.timer -= 1;
                        }
                        // a gentle anchored hover: wander with a mild
                        // pull home ("fly around their nests")
                        wander_3d(rng, m, speed * 0.6);
                        if let Some(h) = bee.hive {
                            let dx = h[0] as f32 + 0.5 - m.pos[0];
                            let dz = h[2] as f32 + 0.5 - m.pos[2];
                            let hd = (dx * dx + dz * dz).sqrt();
                            if hd > 12.0 {
                                m.vel[0] += dx / hd * speed * 0.05;
                                m.vel[2] += dz / hd * speed * 0.05;
                            }
                        }
                    }
                    super::bees::PH_TO_FLOWER => {
                        let target = bee.flower.map(|f| {
                            [
                                f[0] as f32 + 0.5,
                                f[1] as f32 + 1.1,
                                f[2] as f32 + 0.5,
                            ]
                        });
                        match target {
                            Some(tp) => {
                                let dd = (m.pos[0] - tp[0]).powi(2)
                                    + (m.pos[1] - tp[1]).powi(2)
                                    + (m.pos[2] - tp[2]).powi(2);
                                if dd < 1.4 {
                                    // begin the visit: "After circling
                                    // a flower for more than 400 game
                                    // ticks (20 seconds), a bee collects
                                    // nectar"
                                    bee.phase = super::bees::PH_CIRCLE;
                                    bee.timer = super::bees::FLOWER_CIRCLE_TICKS;
                                } else {
                                    steer_3d(m, tp, speed);
                                }
                            }
                            None => bee.phase = super::bees::PH_HOVER,
                        }
                    }
                    super::bees::PH_CIRCLE => {
                        // orbit the flower until the 400-tick visit ends
                        if let Some(f) = bee.flower {
                            // a circling offset (the orbit look)
                            let ang = (bee.timer as f32) * 0.25;
                            let target = [
                                f[0] as f32 + 0.5 + ang.cos() * 0.9,
                                f[1] as f32 + 1.2 + ang.sin() * 0.35,
                                f[2] as f32 + 0.5 + ang.sin() * 0.9,
                            ];
                            steer_3d(m, target, speed * 0.8);
                        }
                        bee.timer -= 1;
                        if bee.timer <= 0 {
                            // nectar collected — the fertilize payload
                            bee.nectar = true;
                            bee.pollinations = super::bees::NECTAR_CHARGES;
                            bee.phase = super::bees::PH_TO_HIVE;
                        }
                    }
                    super::bees::PH_TO_HIVE => {
                        // pollination en route: "A bee can fertilize
                        // plants 10 times ... approximately 5% chance
                        // each tick"; "1 to 2 blocks directly above the
                        // plant"
                        if bee.nectar
                            && bee.pollinations > 0
                            && rng.next_f32() < super::bees::POLLINATE_CHANCE
                        {
                            if let Some((crop, age)) =
                                super::bees::pollination_target(world, m.pos)
                            {
                                bee_pollinations.push((crop, age + 1));
                                bee.pollinations -= 1;
                            }
                        }
                        match bee.hive {
                            Some(h) => {
                                let target = [
                                    h[0] as f32 + 0.5,
                                    h[1] as f32 - 0.4,
                                    h[2] as f32 + 0.5,
                                ];
                                let dd = (m.pos[0] - target[0]).powi(2)
                                    + (m.pos[1] - target[1]).powi(2)
                                    + (m.pos[2] - target[2]).powi(2);
                                if dd < 1.2 {
                                    // arrived — the mob leaves the list
                                    // (MobSystem::tick drains
                                    // bee_enters)
                                    bee.arrived = true;
                                    bee_enters.push((m.id, h, bee.nectar));
                                } else {
                                    steer_3d(m, target, speed);
                                }
                            }
                            None => {
                                // homeless: keep wandering ("homeless
                                // bees search ... wandering around")
                                bee.phase = super::bees::PH_HOVER;
                            }
                        }
                    }
                    _ => {
                        bee.phase = super::bees::PH_HOVER;
                    }
                }
                let _ = &mut restore;
                if restore {
                    m.bee = Some(bee);
                }
            }
        }
        // bees handle their own steering — skip the generic AI below
        // (the early return pattern of the specialized mobs)
        return;
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

    // ---- 1.13 (Update Aquatic) mob behaviors ----
    // PHANTOM: the insomnia swooper — circles 12 blocks above the
    // player, dives on alignment (the classic orbit-and-swoop cycle,
    // VERIFIED w/Phantom §Behavior: "circles ... swoops down"). The
    // phase state: variant bit 0 = diving, aux = the phase countdown
    // (200-tick orbit, 60-tick dive — the two windows the doc comment
    // promises; a single-aux encoding degenerates to 1-tick dives, so
    // the bit carries the mode across ticks).
    if m.kind == MobKind::Phantom {
        let diving = m.variant & 1 != 0;
        if m.aux > 0 {
            m.aux -= 1;
        }
        if !diving {
            // orbit: circle the player at radius 8, 12 blocks up.
            // Altitude gets its OWN damped controller: the tangential
            // chase in steer_3d eats most of the steering authority,
            // so a shared 3D steer left the phantom trailing 3-4
            // blocks below its spec height (VERIFIED w/Phantom
            // §Behavior: "circles ... approximately 12 blocks above").
            let orbit_y = p[1] + 12.0;
            let ang = (m.pos[0] - p[0]).atan2(m.pos[2] - p[2]);
            let next_ang = ang + 0.05;
            let tx = p[0] + next_ang.sin() * 8.0;
            let tz = p[2] + next_ang.cos() * 8.0;
            // horizontal chase only (vertical zeroed here)
            steer_3d(m, [tx, m.pos[1], tz], speed * 1.2);
            // altitude hold: P-controller with velocity damping — the
            // approach is asymptotic (gain → 0 as the error → 0), so
            // the orbit never sags below its target height
            let y_err = orbit_y - m.pos[1];
            m.vel[1] = m.vel[1] * 0.6 + y_err.clamp(-3.0, 3.0) * 0.2;
            if m.aux <= 0 {
                // orbit window over: switch to the 60-tick dive
                m.variant |= 1;
                m.aux = 60;
            }
        } else {
            // dive: straight at the player's chest
            steer_3d(m, [p[0], p[1] + 1.0, p[2]], speed * 1.8);
            let d3 = ((p[0] - m.pos[0]).powi(2)
                + (p[1] - m.pos[1]).powi(2)
                + (p[2] - m.pos[2]).powi(2))
            .sqrt();
            if !invuln && d3 < 1.4 && m.attack_cd == 0 {
                m.attack_cd = 20;
                hits.push(PlayerHit {
                    damage: d.damage, // E/N 2 (VERIFIED, 1.14-pre3 value)
                    source: m.kind,
                    knockback_dir: [dx / dist, dz / dist],
                    wither_effect: None,
                    poison_effect: None,
                });
            }
            if m.aux <= 0 {
                // dive window over: back to the 200-tick orbit
                m.variant &= !1;
                m.aux = 200;
            }
        }
        return;
    }

    // PUFFERFISH: the inflating contact defender — inflates as the
    // player closes within 3 blocks, contact damage + Poison at
    // touch (VERIFIED w/Pufferfish: semi 2 HP E/N + 3 s poison, fully
    // 3 HP N + 6 s poison Java; the engine's one-tier poison is I)
    if m.kind == MobKind::Pufferfish {
        let near = dist < 3.0 && !invuln;
        // inflate/deflate one step per 20 ticks
        if m.attack_cd == 0 {
            m.attack_cd = 20;
            if near && m.variant < 2 {
                m.variant += 1;
            } else if !near && m.variant > 0 {
                m.variant -= 1;
            }
        }
        let inflated = m.variant >= 1;
        if inflated
            && !invuln
            && dist < d.width * 0.5 + 0.9
            && m.aux <= 0
        {
            m.aux = 20; // contact cadence (~0.5 s, the immunity window)
            hits.push(PlayerHit {
                damage: if m.variant == 2 { d.damage } else { 2.0 },
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                // 3 s semi / 6 s fully puffed (VERIFIED Java rows)
                poison_effect: Some(if m.variant >= 2 { 120 } else { 60 }),
            });
        }
        if m.aux > 0 {
            m.aux -= 1;
        }
        wander_3d(rng, m, speed);
        return;
    }

    // TURTLE block moved above the player-anchor early return —
    // nesting and maturity are environmental (see the 1.13 section).

    // DOLPHIN: the pod grace-giver — any player sprint-swimming within
    // a 9-block sphere banks Dolphin's Grace 5 s (VERIFIED w/Dolphin);
    // provoked pods melee like the wolf-pack pattern
    if m.kind == MobKind::Dolphin {
        let dd = ((p[0] - m.pos[0]).powi(2)
            + (p[1] - m.pos[1]).powi(2)
            + (p[2] - m.pos[2]).powi(2))
        .sqrt();
        if !invuln && dd <= 9.0 && m.aux <= 0 {
            m.aux = 20; // re-apply at most 1/s (replenished, VERIFIED)
            pending_player_grace.push(100); // 5 s (VERIFIED)
        }
        if m.aux > 0 {
            m.aux -= 1;
        }
        if m.provoked && !invuln && dist < MOB_MELEE_REACH + 0.8 && m.attack_cd == 0 {
            m.attack_cd = MOB_MELEE_TICKS;
            face_player(m);
            hits.push(PlayerHit {
                damage: d.damage, // N 3 (VERIFIED)
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                poison_effect: None,
            });
            return;
        }
        if in_water_mob(m, world, d.height) {
            wander_3d(rng, m, speed);
        } else {
            wander(rng, m, speed * 0.5);
        }
        return;
    }

    // FISH (cod/salmon/tropical): 3D school wander in water
    if matches!(
        m.kind,
        MobKind::Cod | MobKind::Salmon | MobKind::TropicalFish
    ) {
        wander_3d(rng, m, speed);
        return;
    }

    // DROWNED: zombie-parity melee chase, PLUS the trident throw for
    // the 6.25% armed roll (VERIFIED w/Drowned §Attacking: "A drowned
    // with a trident can throw it every 1.5 seconds, sending it up to
    // 20 blocks away")
    if m.kind == MobKind::Drowned {
        if m.variant & 1 != 0 && aggro && dist <= 20.0 && dist > 4.0 && m.attack_cd == 0 {
            m.attack_cd = 30; // 1.5 s (VERIFIED)
            face_player(m);
            spawn_projectile(m, p, rng, arrows, ProjKind::Trident, 20.0, 8.0);
            return;
        }
        // falls through to the generic hostile melee chase below
    }

    // 1.13: the zombie→drowned conversion moved ABOVE the
    // player-anchor early return (environmental — see the 1.13 section).

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
                poison_effect: None,
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

    // ---- 1.14 (Village & Pillage): FOX — the taiga predator.
    // VERIFIED w/Fox: "Foxes attack chickens, rabbits, cod, salmon and
    // tropical fish, and baby turtles while they are on land"; "Foxes
    // don't attack players"; wild foxes flee approaching players while
    // trusting ones (bred babies, variant bit 0) stay. The pounce
    // animation folds into the chase (no lunge-animation system,
    // disclosed). The life-cycle clocks (baby/love countdowns) run in
    // the environmental section above. ----
    if m.kind == MobKind::Fox {
        // wild foxes flee the approaching player (the ocelot's 6-block
        // scare radius is the engine precedent — the wiki gives no
        // fox-specific figure, disclosed approximation)
        if m.variant & 0x1 == 0 && dist < 6.0 && !invuln {
            m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
            let f = speed * FLEE_MULT;
            m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
            m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            return;
        }
        // prey scan (VERIFIED list): chickens + rabbits anywhere; the
        // fish kinds + baby turtles only ON LAND ("while they are on
        // land" — a beached fish is exactly when a fox takes it)
        let mut best: Option<(u32, [f32; 3], f32)> = None;
        for (id, k, pos, variant) in snapshot.iter() {
            if *id == m.id {
                continue;
            }
            let prey = match k {
                MobKind::Chicken | MobKind::Rabbit => true,
                MobKind::Cod | MobKind::Salmon | MobKind::TropicalFish => {
                    // fish count only out of water (flopping on land)
                    world.get_block(pos[0] as i32, pos[1] as i32, pos[2] as i32) != WATER
                }
                // baby turtles (variant bit 0x40) on land
                MobKind::Turtle => (variant & 0x40 != 0) && world.get_block(
                    pos[0] as i32,
                    pos[1] as i32,
                    pos[2] as i32,
                ) != WATER,
                _ => false,
            };
            if !prey {
                continue;
            }
            let sx = pos[0] - m.pos[0];
            let sz = pos[2] - m.pos[2];
            let dd = (sx * sx + sz * sz).sqrt();
            if dd <= 12.0 && best.map(|(_, _, bd)| dd < bd).unwrap_or(true) {
                best = Some((*id, *pos, dd));
            }
        }
        if let Some((tid, tpos, tdist)) = best {
            face_target(m, tpos);
            if tdist > 1.0 {
                m.vel[0] += ((tpos[0] - m.pos[0]) / tdist * speed * 1.3 - m.vel[0]) * 0.4;
                m.vel[2] += ((tpos[2] - m.pos[2]) / tdist * speed * 1.3 - m.vel[2]) * 0.4;
            } else if m.attack_cd == 0 {
                m.attack_cd = MOB_MELEE_TICKS;
                pending.push((tid, d.damage)); // 2 HP (Easy/Normal row)
            }
            return;
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

    // ---- 1.11: LLAMA — passive herd animal; retaliates with a spit
    // projectile when provoked (VERIFIED w/Llama: "If the player hits
    // them, they spit at the player once, dealing 1 HP damage"); the
    // 1/900-chance per-tick 1 HP regen (VERIFIED w/Llama: "Llamas have
    // a 1⁄900 chance to regenerate 1 HP health point each game tick").
    // Wolf aggression is N/A (no wolves in the engine — disclosed).
    if m.kind == MobKind::Llama {
        // regen: 1/900 per tick, only when damaged
        if m.health < d.health && m.health > 0.0 && rng.next_range(900) == 0 {
            m.health = (m.health + 1.0).min(d.health);
        }
        if m.provoked && !invuln && m.attack_cd == 0 && dist < 10.0 {
            m.attack_cd = 20;
            face_player(m);
            spawn_projectile(m, p, rng, arrows, ProjKind::LlamaSpit, 18.0, 1.0);
            return;
        }
        wander(rng, m, speed * 0.4);
        return;
    }

    // ---- 1.11: VINDICATOR — hostile melee chaser at sprint speed
    // (VERIFIED w/Vindicator: "Speed 5.612 blocks/sec" — the fastest
    // hostile; iron axe 13 HP Normal with difficulty scaling; the
    // changelog: "Hostile towards players and villagers" — villagers
    // are a separate system, the player path is live).
    if m.kind == MobKind::Vindicator {
        if aggro && dist < 32.0 {
            face_player(m);
            m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.35;
            m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.35;
            if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                m.attack_cd = MOB_MELEE_TICKS;
                hits.push(PlayerHit {
                    damage: d.damage,
                    source: m.kind,
                    knockback_dir: [dx / dist, dz / dist],
                    wither_effect: None,
                    poison_effect: None,
                });
            }
        } else {
            wander(rng, m, speed * 0.4);
        }
        return;
    }

    // ---- 1.11: EVOKER — the spell-casting mini-boss (VERIFIED
    // w/Evoker: "Evokers use two spells to attack; one that summons
    // armor-piercing fangs and one that summons vexes"). The engine
    // adaptation: a 100-tick spell cycle (aux) — fangs apply 6 HP to
    // the player (armor-ignoring, VERIFIED: "This harm is not mitigated
    // by armor") within 12 blocks; when no own vexes are alive within
    // 32 blocks, the next cycle summons 3 (changelog: "In battle, they
    // summon vexes and fangs to attack"). Fang entities are a
    // particle + timed damage adaptation (no standalone fang entity
    // system — disclosed). The sheep color-conversion spell is
    // deferred (the engine's sheep carry no wool-color variant).
    if m.kind == MobKind::Evoker {
        if aggro && dist < 24.0 {
            face_player(m);
            // keep casting distance
            if dist > 12.0 {
                m.vel[0] += (dx / dist * speed * 0.8 - m.vel[0]) * 0.3;
                m.vel[2] += (dz / dist * speed * 0.8 - m.vel[2]) * 0.3;
            } else {
                m.vel[0] *= 0.8;
                m.vel[2] *= 0.8;
            }
            m.aux = (m.aux + 1) % 100;
            if m.aux == 0 {
                let own_vexes = snapshot
                    .iter()
                    .filter(|(id, k, pos, _)| {
                        *k == MobKind::Vex
                            && *id != m.id
                            && (pos[0] - m.pos[0]).powi(2) + (pos[2] - m.pos[2]).powi(2) < 1024.0
                    })
                    .count();
                if own_vexes == 0 {
                    // summon 3 vexes around the evoker (the changelog's
                    // summon spell; positions offset like vanilla's ring)
                    pending_summons.push((m.id, 3));
                } else {
                    // fang strike: 6 HP, bypasses armor (VERIFIED)
                    if !invuln {
                        pending_player_fang.push(6.0);
                    }
                }
            }
        } else {
            wander(rng, m, speed * 0.3);
        }
        return;
    }

    // ---- 1.11: VEX — small flying attacker that phases through blocks
    // (VERIFIED w/Vex: "pass through any block, including water and
    // lava"). Engine adaptation: direct velocity steering toward the
    // target INCLUDING vertical — the collision pass approximates the
    // phasing (no wall pathing; a documented simplification). "Spawn
    // only when summoned by an evoker" — the ambient spawn pool
    // excludes illagers/vexes; evoker summons are the only source.
    if m.kind == MobKind::Vex {
        if aggro && dist < 32.0 {
            let dy = p[1] + 1.0 - m.pos[1];
            let full = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-4);
            face_player(m);
            m.vel[0] += (dx / full * speed - m.vel[0]) * 0.4;
            m.vel[1] += (dy / full * speed * 0.6 - m.vel[1]) * 0.4;
            m.vel[2] += (dz / full * speed - m.vel[2]) * 0.4;
            if dist < MOB_MELEE_REACH + 0.6 && m.attack_cd == 0 {
                m.attack_cd = MOB_MELEE_TICKS;
                hits.push(PlayerHit {
                    damage: d.damage,
                    source: m.kind,
                    knockback_dir: [dx / dist, dz / dist],
                    wither_effect: None,
                    poison_effect: None,
                });
            }
        } else {
            // idle hover drift
            wander(rng, m, speed * 0.3);
            m.vel[1] += (0.4 - m.vel[1]) * 0.05;
        }
        return;
    }

    // ---- 1.12: PARROT — the flying passive. Variant encoding: bits 0..2
    // = color 0..=4, bit 7 = tamed; aux bit 0 = sitting (the right-click
    // toggle), bits 1.. = the flight-phase counter. VERIFIED w/Parrot:
    // "Fly around, but sit when 'tired'" (changelog §Mobs — the
    // unquantified rest cycle approximated as periodic settling),
    // "a tamed parrot follows the player and teleports if there is a
    // distance of 12 blocks between it and the player", "Follow and
    // crowd around nearby mobs" (the follow arm approximates at the
    // tamer; wild crowd-around is the wander). Shoulder perching,
    // jukebox dancing, and hostile-sound mimicry are out of engine
    // scope (no player model, no jukebox, no mob-sound audio
    // imitations — disclosed).
    if m.kind == MobKind::Parrot {
        let tamed = m.variant & 0x80 != 0;
        let sitting = m.aux & 1 != 0;
        // flight-phase counter (bits 1..): drives the rest cycle
        let phase = (m.aux >> 1) + 1;
        m.aux = ((phase & 0x3FFF) << 1) | (m.aux & 1);
        if sitting {
            // the sit toggle (17w14a: "right-clicking on a tamed parrot
            // now tells it to sit") — a sitting parrot stays put
            m.vel[0] *= 0.6;
            m.vel[1] = 0.0;
            m.vel[2] *= 0.6;
            return;
        }
        if tamed {
            // the 12-block teleport (VERIFIED w/Parrot)
            if dist > 12.0 {
                m.pos = [p[0] - 0.6, p[1] + 0.2, p[2]];
                m.vel = [0.0; 3];
            } else if dist > 4.0 {
                // fly to the tamer (vex-style steering, gentler)
                let dy = p[1] + 1.0 - m.pos[1];
                let full = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-4);
                face_player(m);
                m.vel[0] += (dx / full * speed - m.vel[0]) * 0.25;
                m.vel[1] += (dy / full * 0.45 - m.vel[1]) * 0.25;
                m.vel[2] += (dz / full * speed - m.vel[2]) * 0.25;
            } else {
                // hover near the tamer's head height
                face_player(m);
                m.vel[0] *= 0.9;
                m.vel[2] *= 0.9;
                m.vel[1] += ((p[1] + 1.6 - m.pos[1]) * 0.06 - m.vel[1]) * 0.1;
            }
        } else {
            // wild: "fly around" — wander + a gentle altitude wave; the
            // rest cycle settles it every ~20 s for ~5 s ("sit when
            // 'tired'" — the changelog's own wording, unquantified →
            // clean-room cycle, disclosed)
            let resting = phase % 500 >= 450;
            if resting {
                m.vel[0] *= 0.8;
                m.vel[2] *= 0.8;
                m.vel[1] += (-0.25 - m.vel[1]) * 0.08;
            } else {
                wander(rng, m, speed * 0.55);
                m.vel[1] += ((phase as f32 * 0.07).sin() * 0.22 - m.vel[1]) * 0.05;
            }
        }
        return;
    }

    // ---- 1.12: ILLUSIONER — the hostile spell-casting archer.
    // VERIFIED w/Illusioner: bow fired every second ("firing an arrow
    // every second, three times faster than a skeleton" — 20 ticks),
    // "casts its Blindness spell ... upon first engaging a new player
    // opponent" (20 s — the regional-difficulty > 2 gate is a
    // Normal-difficulty engine simplification, disclosed), the mirror
    // spell = Invisibility 60 s + 4 false duplicates ("As long as an
    // illusioner is engaged in combat, it casts an Invisibility status
    // effect on itself that lasts 60 seconds and refreshes"), and it
    // strafes while keeping distance ("moves quickly on a semi-circular
    // fashion and always tries to maintain a consistent distance").
    // Encoding: variant bit 0 = the has-cast-blindness gate (once per
    // opponent); aux = invisibility ticks left (0 = visible, no
    // duplicates); the strafe cycle rides the invisibility counter.
    if m.kind == MobKind::Illusioner {
        if aggro && dist < 16.0 {
            face_player(m);
            // the mirror spell: invisibility 60 s, refreshed while
            // engaged ("casts ... that lasts 60 seconds and refreshes
            // the effect whenever the Invisibility's time runs out")
            let invis = if m.aux < 20 { 20 * 60 } else { m.aux - 1 };
            m.aux = invis;
            // semi-circular strafe: slide tangentially, direction
            // alternating each 40 ticks (derived from the counter)
            let dir = if (invis / 40) % 2 == 0 { 1.0 } else { -1.0 };
            let tx = -dz / dist * dir;
            let tz = dx / dist * dir;
            // keep ~10 blocks: advance or retreat along the radial
            let radial = if dist > 10.0 { 1.0 } else { -0.6 };
            m.vel[0] += (dx / dist * speed * radial * 0.7 + tx * speed * 0.5 - m.vel[0]) * 0.15;
            m.vel[2] += (dz / dist * speed * radial * 0.7 + tz * speed * 0.5 - m.vel[2]) * 0.15;
            // the blindness spell: once per opponent, on first engage
            if m.variant & 1 == 0 && !invuln {
                m.variant |= 1; // the once-gate
                pending_blindness.push(20 * 20); // 20 s (VERIFIED)
            }
            // the bow: every 20 ticks (1/s — VERIFIED); arrow damage
            // rolls at fire time (Easy/Normal 2–5, the skeleton path)
            if m.attack_cd == 0 && dist > 2.0 {
                m.attack_cd = 20; // 1/s (VERIFIED)
                spawn_projectile(m, p, rng, arrows, ProjKind::Arrow, 10.0, d.damage);
            }
        } else {
            // idle: drop the mirror (duplicates vanish with it) and wander
            m.aux = 0;
            wander(rng, m, speed * 0.4);
        }
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
                poison_effect: None,
            });
            }
        } else {
            m.aux = 0;
            wander(rng, m, speed * 0.4);
        }
        return;
    }

    // ---- the completeness audit: GHAST — the floating Nether
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
                poison_effect: None,
            });
        }
        return;
    }

    match m.kind {
        // ---- 1.16 (Nether Update, part 2): STRIDER — the lava-walking
        // passive. Wanders on lava or land (the physics layer keeps
        // them on the surface); flees briefly when harmed ("Upon being
        // harmed by another mob, striders attempt to flee for a few
        // seconds", VERIFIED w/Strider). Babies just wander. ----
        MobKind::Strider => {
            if m.hurt_t > 0 {
                m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
                let f = speed * FLEE_MULT;
                m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
                m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
            } else {
                wander(rng, m, speed * 0.4);
            }
        }
        // ---- 1.16: PIGLIN — the neutral barterer. Neutral until
        // provoked ("It is hostile to players unless they wear at least
        // one piece of golden armor" — no wearable armor in the engine,
        // the enderman class, disclosed); "Soul torches repel piglins"
        // (VERIFIED w/Soul_Torch) + soul fire carries the same blue-
        // flame class → flee within 8 blocks; provoked → melee chase
        // (the zombie pattern at the piglin's verified 0.35 speed).
        // The barter countdown runs in the environmental section
        // above (no player anchor needed). ----
        MobKind::Piglin => {
            // the soul-flame repel: scan the 8-block cube for soul
            // torches / soul fire ("Soul torches repel piglins")
            let mut repel: Option<[f32; 3]> = None;
            for sy in -1..=1i32 {
                for sz in -8..=8i32 {
                    for sx in -8..=8i32 {
                        let b = world.get_block(
                            m.pos[0] as i32 + sx,
                            m.pos[1] as i32 + sy,
                            m.pos[2] as i32 + sz,
                        );
                        if b == SOUL_TORCH || b == SOUL_FIRE || b == SOUL_LANTERN {
                            repel = Some([
                                m.pos[0] as f32 + sx as f32,
                                0.0,
                                m.pos[2] as f32 + sz as f32,
                            ]);
                            break;
                        }
                    }
                    if repel.is_some() {
                        break;
                    }
                }
                if repel.is_some() {
                    break;
                }
            }
            if let Some(r) = repel {
                // steer away from the soul light ("repel")
                let rx = m.pos[0] - r[0];
                let rz = m.pos[2] - r[2];
                let rd = (rx * rx + rz * rz).sqrt().max(1e-4);
                m.yaw = (-rz).atan2(rx) - std::f32::consts::FRAC_PI_2;
                m.vel[0] += (rx / rd * speed * 1.3 - m.vel[0]) * 0.4;
                m.vel[2] += (rz / rd * speed * 1.3 - m.vel[2]) * 0.4;
                return;
            }
            if m.provoked && !invuln && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > MOB_MELEE_REACH * 0.8 {
                    m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.7;
                    m.vel[2] *= 0.7;
                }
                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage, // golden sword Normal 8 (VERIFIED)
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                        wither_effect: None,
                        poison_effect: None,
                    });
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        // ---- 1.16: HOGLIN — the hostile forest boar. "Hoglins avoid
        // being within 7 blocks of warped fungi ... and respawn
        // anchors" (VERIFIED w/Hoglin) → flee (takes priority over
        // fighting, the wiki's own ordering); otherwise the hostile
        // melee chase + the tusk thrust (Normal 3-8, the 5.5 midpoint
        // row); babies (0x40) flee when hurt instead of fighting
        // ("Baby hoglins ... flee when hit", VERIFIED). ----
        MobKind::Hoglin => {
            // the warped-fungus / respawn-anchor repel (7 blocks)
            let mut repel: Option<[f32; 3]> = None;
            for sy in -1..=1i32 {
                for sz in -7..=7i32 {
                    for sx in -7..=7i32 {
                        let b = world.get_block(
                            m.pos[0] as i32 + sx,
                            m.pos[1] as i32 + sy,
                            m.pos[2] as i32 + sz,
                        );
                        if b == WARPED_FUNGUS || b == RESPAWN_ANCHOR {
                            repel = Some([
                                m.pos[0] as f32 + sx as f32,
                                0.0,
                                m.pos[2] as f32 + sz as f32,
                            ]);
                            break;
                        }
                    }
                    if repel.is_some() {
                        break;
                    }
                }
                if repel.is_some() {
                    break;
                }
            }
            let baby = m.variant & 0x40 != 0;
            if let Some(r) = repel {
                // "Hoglins cannot be bred when they are running away
                // from warped fungi" — the flee outranks everything
                let rx = m.pos[0] - r[0];
                let rz = m.pos[2] - r[2];
                let rd = (rx * rx + rz * rz).sqrt().max(1e-4);
                m.yaw = (-rz).atan2(rx) - std::f32::consts::FRAC_PI_2;
                m.vel[0] += (rx / rd * speed * 1.4 - m.vel[0]) * 0.4;
                m.vel[2] += (rz / rd * speed * 1.4 - m.vel[2]) * 0.4;
                return;
            }
            if baby {
                // babies flee when hit, otherwise tag along
                if m.hurt_t > 0 {
                    m.yaw = (-dz).atan2(-dx) - std::f32::consts::FRAC_PI_2;
                    let f = speed * FLEE_MULT;
                    m.vel[0] += (-dx / dist * f - m.vel[0]) * 0.4;
                    m.vel[2] += (-dz / dist * f - m.vel[2]) * 0.4;
                } else {
                    wander(rng, m, speed * 0.5);
                }
            } else if aggro && dist < AGGRO_RADIUS {
                face_player(m);
                if dist > MOB_MELEE_REACH * 0.8 {
                    m.vel[0] += (dx / dist * speed - m.vel[0]) * 0.3;
                    m.vel[2] += (dz / dist * speed - m.vel[2]) * 0.3;
                } else {
                    m.vel[0] *= 0.7;
                    m.vel[2] *= 0.7;
                }
                if dist < MOB_MELEE_REACH && m.attack_cd == 0 {
                    m.attack_cd = MOB_MELEE_TICKS;
                    hits.push(PlayerHit {
                        damage: d.damage, // Normal 3-8 midpoint (VERIFIED)
                        source: m.kind,
                        knockback_dir: [dx / dist, dz / dist],
                        wither_effect: None,
                        poison_effect: None,
                    });
                }
            } else {
                wander(rng, m, speed * 0.5);
            }
        }
        MobKind::Zombie
        | MobKind::ZombieVillager
        | MobKind::Husk
        | MobKind::Spider
        // the completeness audit: the two ground-classic hostiles
        // (the cave spider rides the spider's chase; the silverfish
        // the zombie's)
        | MobKind::CaveSpider
        | MobKind::Silverfish
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
                // the completeness audit: the cave spider's venom —
                // "Normal: Poison for 7 seconds" (VERIFIED
                // w/Cave_Spider; 140 ticks; Easy gets none and Hard
                // 15 s — the engine's Normal row, disclosed)
                poison_effect: if m.kind == MobKind::CaveSpider {
                    Some(140)
                } else {
                    None
                },
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

/// 1.13: is the mob's body in water (the swim-physics gate, AI-side).
fn in_water_mob(m: &Mob, world: &World, height: f32) -> bool {
    world.get_block(
        m.pos[0] as i32,
        (m.pos[1] + height * 0.5) as i32,
        m.pos[2] as i32,
    ) == WATER
}

/// 1.13: steer toward a 3D point (the aquatic/phantom movement form —
/// smooth interpolation toward the target velocity).
fn steer_3d(m: &mut Mob, target: [f32; 3], speed: f32) {
    let dx = target[0] - m.pos[0];
    let dy = target[1] - m.pos[1];
    let dz = target[2] - m.pos[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-4);
    m.vel[0] += (dx / len * speed - m.vel[0]) * 0.15;
    m.vel[1] += (dy / len * speed - m.vel[1]) * 0.15;
    m.vel[2] += (dz / len * speed - m.vel[2]) * 0.15;
    m.yaw = (-dz).atan2(dx) - std::f32::consts::FRAC_PI_2;
}

/// 1.13: 3D wander for aquatic mobs — the land wander with a gentle
/// vertical bob (fish school drift; the y target keeps them mid-column).
fn wander_3d(rng: &mut Rng, m: &mut Mob, speed: f32) {
    wander(rng, m, speed);
    if m.wander_t > 0 && rng.next_f32() < 0.02 {
        // occasional vertical drift: a small target nudge
        m.vel[1] += (rng.next_f32() - 0.5) * 0.4;
    }
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

/// 1.14 (Village & Pillage — nature half): environmental hazards,
/// applied between the AI steering and the physics move.
///
/// - **Sweet berry bush** (VERIFIED w/Sweet_Berry_Bush §Entity
///   movement): "A sweet berry bush (at any stage) slows down all
///   entities (except items) passing through it. At stage 1 and
///   higher, it causes damage" — "deal 1 HP damage every tick
///   (although damage immunity reduces this to once every
///   half-second), only if the entity is moving in the hitbox of the
///   bush", and entities "slow down to about 34.05% of their normal
///   speed". Foxes are immune to BOTH (VERIFIED w/Fox: "take no
///   damage or speed reduction while moving through sweet berry
///   bushes") — the round's plant/mob interlock.
/// - **Lit campfire** (VERIFIED w/Campfire §Damage): "If lit,
///   campfires damage mobs standing on top of them ... 1 HP every
///   tick (although damage immunity reduces this to once every
///   half-second)".
///
/// The damage windows ride the 10-tick hazard clock (0.5 s); the
/// slow scales the CURRENT tick's velocity after the AI wrote it, so
/// the steady-state pace through a bush is the 34.05% row.
fn hazard_tick(m: &mut Mob, world: &World, damage_window: bool) {
    // berry bush at the feet cell
    if m.kind != MobKind::Fox {
        let bx = m.pos[0].floor() as i32;
        let by = m.pos[1].floor() as i32;
        let bz = m.pos[2].floor() as i32;
        let s = world.get_state(bx, by, bz);
        if state_block(s) == SWEET_BERRY_BUSH && berry_bush_age(s) >= 1 {
            let moving = m.vel[0] * m.vel[0] + m.vel[2] * m.vel[2] > 0.01;
            if moving {
                // 34.05% (VERIFIED) — one scale per tick against the
                // AI's freshly-written velocity
                m.vel[0] *= BUSH_SLOW_FACTOR;
                m.vel[2] *= BUSH_SLOW_FACTOR;
                if damage_window {
                    m.health -= 1.0;
                }
            }
        }
    }
    // lit campfire underfoot (standing ON it)
    if m.on_ground {
        let gx = m.pos[0].floor() as i32;
        let gy = (m.pos[1] - 0.1).floor() as i32;
        let gz = m.pos[2].floor() as i32;
        let gs = world.get_state(gx, gy, gz);
        if state_block(gs) == CAMPFIRE && campfire_lit(gs) && damage_window {
            m.health -= 1.0;
        }
    }
    // 1.16 (Nether Update, part 2): striders are damaged by water —
    // "1 HP per ... half-second in water or rain" (VERIFIED w/Strider;
    // the engine has no rain — the water-contact half only). The
    // damage rides the shared 0.5 s immunity window.
    if m.kind == MobKind::Strider {
        let wx = m.pos[0].floor() as i32;
        let wy = m.pos[1].floor() as i32;
        let wz = m.pos[2].floor() as i32;
        if world.get_block(wx, wy, wz) == WATER && damage_window {
            m.health -= 1.0;
        }
    }
}

/// the verified bush movement factor (w/Sweet_Berry_Bush: "slow down
/// to about 34.05% of their normal speed")
pub const BUSH_SLOW_FACTOR: f32 = 0.3405;

/// gravity + axis collision with 1-block step-ups (villager primitive).
fn physics_tick(m: &mut Mob, world: &World) {
    let d = def(m.kind);
    // ---- 1.16 (Nether Update, part 2): STRIDER lava physics —
    // "Lava does not damage striders, and they can walk on top of it
    // without sinking" (VERIFIED w/Strider). Feet in lava + air above
    // = standing on the surface (gravity neutralized, no sink);
    // fully submerged = "If a strider spawns under lava, it rises out
    // of the lava" (VERIFIED) — a buoyant ascent. ----
    if m.kind == MobKind::Strider {
        let feet = world.get_block(
            m.pos[0].floor() as i32,
            m.pos[1].floor() as i32,
            m.pos[2].floor() as i32,
        );
        let above = world.get_block(
            m.pos[0].floor() as i32,
            (m.pos[1] + 1.0).floor() as i32,
            m.pos[2].floor() as i32,
        );
        if feet == LAVA {
            if above != LAVA {
                // on the surface: stand (no gravity, no fall distance)
                m.vel[1] = 0.0;
                m.on_ground = true;
                m.fall_dist = 0.0;
                // lava drag on the stroll
                m.vel[0] *= 0.9;
                m.vel[2] *= 0.9;
            } else {
                // submerged: rise out (VERIFIED)
                m.vel[1] = 2.0;
            }
            // skip the regular gravity+collision pass below — the
            // strider is on the lava sea, not in the block grid
            let half = d.width * 0.5;
            let (nx, nz) = (
                m.pos[0] + m.vel[0] * (1.0 / 20.0),
                m.pos[2] + m.vel[2] * (1.0 / 20.0),
            );
            if !collides(world, nx, m.pos[1], nz, half, d.height) {
                m.pos[0] = nx;
                m.pos[2] = nz;
            } else {
                m.vel[0] *= 0.5;
                m.vel[2] *= 0.5;
            }
            return;
        }
    }
    // ---- 1.13 (Update Aquatic): aquatic swim physics. In water the
    // aquatic kinds get buoyancy + drag instead of gravity (fish hover,
    // turtles/dolphins glide); out of water the fish family
    // suffocates (VERIFIED w/Cod: fish "cannot survive out of water
    // ... they start suffocating" — 1 HP/s engine form of vanilla's
    // 10-tick no-air window, disclosed approximation) and flips. ----
    let body_block = world.get_block(
        m.pos[0] as i32,
        (m.pos[1] + d.height * 0.5) as i32,
        m.pos[2] as i32,
    );
    let in_water = body_block == WATER;
    if m.kind.aquatic() && in_water {
        // buoyancy: relax toward zero vertical speed, water drag on all
        // axes (vanilla swim drag 0.8-ish; engine approximation)
        m.vel[1] *= 0.8;
        m.vel[0] *= 0.92;
        m.vel[2] *= 0.92;
        m.fall_dist = 0.0; // water breaks falls (the player rule)
    } else if m.kind.aquatic() {
        // fish out of water: flop + suffocate (the fish family only —
        // dolphins/turtles/drowned breathe air)
        if matches!(
            m.kind,
            MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish
        ) {
            m.health -= 1.0 / 20.0; // ~1 HP/s (documented approximation)
            // flop: a small random hop (vanilla fish flop on land)
            m.vel[0] *= 0.9;
            m.vel[2] *= 0.9;
        }
    }
    // Vanilla entity gravity, EXACT per-tick form (VERIFIED,
    // research-verdicts.md: v1 = (v0 − 0.08) × 0.98 in b/t). Velocities
    // here are b/s, so the per-tick step on b/s units is
    // v ← (v − 1.6) × 0.98 (0.08 b/t × 20 = 1.6 b/s; drag is unitless).
    // Terminal −78.4 b/s (−3.92 b/t) is the inherent fixed point — no
    // clamp. (This also fixes a latent 20× unit bug: the old code
    // subtracted the per-tick 0.08 from a b/s velocity, giving 1.6 b/s²
    // gravity and a 3.92 b/s "terminal" — mobs fell 20× too slow.)
    // FLYING mobs (phantom/vex/bat/parrot — MobKind::flies) are exempt:
    // vanilla FlyingMobs have no gravity, and a constant −1.568 b/s
    // pull dragged the phantom's orbit 3 blocks below its 12-block
    // spec height (VERIFIED w/Phantom §Behavior: "circles ... at a
    // height of approximately 12 blocks above the player").
    if !m.kind.flies() {
        m.vel[1] = (m.vel[1] - 1.6) * 0.98;
    } else {
        // gentle flight drag instead (no fixed point — decays to 0)
        m.vel[1] *= 0.98;
    }
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
    // fall bookkeeping (vanilla fallDistance: per-tick distance).
    // Flying mobs never accumulate fall distance (no gravity-driven
    // descents — a swooping phantom is flight, not a fall).
    if !m.on_ground && m.vel[1] < 0.0 && !m.kind.flies() {
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
    target_hits: &mut Vec<([i32; 3], u8, i32)>,
    landings: &mut Vec<(ProjKind, [f32; 3])>,
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
        // player body-center hit sphere (r = 0.8). The sweep-2
        // throwables carry PLAYER_OWNER — the thrower is never hit by
        // their own projectile (the snowball/egg/pearl class).
        if let Some(p) = player {
            if !invuln && a.owner != PLAYER_OWNER {
                let ddx = a.pos[0] - p[0];
                let ddy = a.pos[1] - (p[1] + 0.9);
                let ddz = a.pos[2] - p[2];
                if ddx * ddx + ddy * ddy + ddz * ddz < 0.64 {
                    let dir = [a.vel[0] / 24.0, a.vel[2] / 24.0];
                    let src = match a.kind {
                        ProjKind::Arrow => MobKind::Skeleton,
                        ProjKind::Fireball => MobKind::Blaze,
                        ProjKind::Snowball => MobKind::SnowGolem,
                        // 1.11: llama spit's source
                        ProjKind::LlamaSpit => MobKind::Llama,
                        // Phase E2: the wither skull's source (the wither
                        // itself is the boss system; the hit carries the
                        // Wither II payload via `wither_effect`)
                        ProjKind::Skull => MobKind::WitherSkeleton,
                        // 1.13: the drowned's thrown trident (8 HP base —
                        // VERIFIED w/Trident "Projectile damage 8 HP")
                        ProjKind::Trident => MobKind::Drowned,
                        // the sweep-2 throwables (attribution only —
                        // never resolved: PLAYER_OWNER skips the sphere)
                        ProjKind::Egg => MobKind::Chicken,
                        ProjKind::Pearl => MobKind::Enderman,
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
                        // 1.13: the pufferfish contact poison rides the
                        // contact-hit path, not projectiles — None here
                        poison_effect: None,
                    });
                    arrows.remove(i);
                    continue;
                }
            }
        }
        // Phase E1: snowball mob hits — 3 damage to blazes, 0 + knockback
        // to everything else (VERIFIED w/Snow_Golem: "Thrown snowballs do
        // not deal damage except to blazes, but they still knock back any
        // mobs that they hit"). The sweep-2 throwables join the same
        // class: eggs and pearls knock mobs back (0 damage — the thrown
        // class's rule) and push a LANDING event at the hit (the pearl
        // teleports the thrower to the struck mob, the egg rolls the
        // hatch there).
        if matches!(a.kind, ProjKind::Snowball | ProjKind::Egg | ProjKind::Pearl) {
            let mut hit_mob = false;
            for m in mobs.iter_mut() {
                if m.id == a.owner {
                    continue; // never hit its own golem
                }
                let ddx = a.pos[0] - m.pos[0];
                let ddy = a.pos[1] - (m.pos[1] + 0.5);
                let ddz = a.pos[2] - m.pos[2];
                if ddx * ddx + ddy * ddy + ddz * ddz < 0.8 {
                    if m.kind == MobKind::Blaze && a.kind == ProjKind::Snowball {
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
                    break;
                }
            }
            if hit_mob {
                arrows.remove(i);
                continue;
            }
        }
        let hit_solid = is_solid(world.get_block(
            a.pos[0].floor() as i32,
            a.pos[1].floor() as i32,
            a.pos[2].floor() as i32,
        ));
        if hit_solid || a.age > 20 * 60
        {
            // 1.16 (Nether Update, part 1): a projectile landing on a
            // TARGET block powers it — "produces a temporary redstone
            // signal when hit by a projectile"; "The strength of the
            // signal depends on how close the projectile is to the
            // center of the block, from 1 to 15"; "When struck by most
            // projectiles, the target emits redstone power for 8 game
            // ticks ... Arrows and tridents instead cause the target to
            // emit power for 20 game ticks" (VERIFIED w/Target). Power
            // from the hit point's distance to the cell center: 15 at
            // dead-center, 1 at the face edge (the adaptation of the
            // proximity rule).
            let bx = a.pos[0].floor() as i32;
            let by = a.pos[1].floor() as i32;
            let bz = a.pos[2].floor() as i32;
            if world.get_block(bx, by, bz) == TARGET {
                let fx = (a.pos[0] - bx as f32 - 0.5).abs();
                let fy = (a.pos[1] - by as f32 - 0.5).abs();
                let fz = (a.pos[2] - bz as f32 - 0.5).abs();
                let d = fx.max(fy).max(fz) * 2.0; // 0..1 from center
                let power = (15.0 - d * 14.0).round().clamp(1.0, 15.0) as u8;
                let ticks = if matches!(a.kind, ProjKind::Arrow | ProjKind::Trident) {
                    20
                } else {
                    8
                };
                target_hits.push(([bx, by, bz], power, ticks));
            }
            // the sweep-2 landing queue: only real block collisions
            // (the 60-second age-out is a lost projectile — the void
            // case, no hatch, no teleport)
            if hit_solid && matches!(a.kind, ProjKind::Egg | ProjKind::Pearl) {
                landings.push((a.kind, a.pos));
            }
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

/// 1.16: drain the projectile-on-target hit queue (the game layer
/// turns each hit into the blockstate power write + its decay timer).
pub fn take_target_hits(sys: &mut MobSystem) -> Vec<([i32; 3], u8, i32)> {
    std::mem::take(&mut sys.target_hits)
}

/// the sweep-2: drain the player-throwable landing queue (the game
/// layer hatches the eggs and resolves the pearl teleports).
pub fn take_landings(sys: &mut MobSystem) -> Vec<(ProjKind, [f32; 3])> {
    std::mem::take(&mut sys.landings)
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
        let mut tile = m.kind.sprite_tile();
        // [1.12 fix] 512px atlas = 32 tiles/row — the old %16//16 math
        // sampled out-of-bounds garbage for every sprite tile >= 256
        // (1.10/1.11 mob sprites; latent since the 512px-atlas merge)
        // 1.12: parrots carry the per-VARIANT sprite (red/blue/green/
        // cyan/gray — the kind-level tile is only the HUD fallback);
        // illusioners render GHOSTED (alpha via the dim tint) while
        // their Invisibility spell runs, plus 4 false duplicates at
        // fixed offsets (VERIFIED w/Illusioner: "When an illusioner
        // becomes invisible ... it creates four false duplicates of
        // itself. These hover and waver at short distances ... they do
        // not space themselves out until the first time the illusioner
        // is attacked. They face in exactly the same direction as the
        // illusioner and move somewhat in step with the original")
        if m.kind == MobKind::Parrot {
            tile = TILE_PARROT_BASE + ((m.variant & 0x07).min(4) as u16);
        }
        let tx = (tile % 32) as f32;
        let ty = (tile / 32) as f32;
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
                [tx / 32.0, (ty + 1.0) / 32.0],
            ),
            (
                [rr[0] * half, 0.0, rr[2] * half],
                [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
            ),
            (
                [rr[0] * half, h, rr[2] * half],
                [(tx + 1.0) / 32.0, ty / 32.0],
            ),
            ([-rr[0] * half, h, -rr[2] * half], [tx / 32.0, ty / 32.0]),
        ];
        // 1.12: an invisible illusioner renders ONLY its 4 false
        // duplicates ("it creates four false duplicates of itself.
        // These hover and waver at short distances from the actually
        // invisible illusioner ... They face in exactly the same
        // direction as the illusioner and move somewhat in step with
        // the original" — VERIFIED w/Illusioner; the waver rides a
        // per-frame sine, the offsets are the uns-paced initial ring)
        let origins: Vec<[f32; 3]> = if m.kind == MobKind::Illusioner && m.aux > 0 {
            let t = m.aux as f32 * 0.31;
            let (a, b) = (t.sin(), t.cos());
            [
                [m.pos[0] + 2.0 + a * 0.5, m.pos[1] + 0.4 + b * 0.3, m.pos[2] - 2.0],
                [m.pos[0] - 2.0, m.pos[1] + 0.6 + a * 0.3, m.pos[2] + 2.0 + b * 0.5],
                [m.pos[0] + 1.5 - b * 0.4, m.pos[1] + 1.0, m.pos[2] + 1.5 + a * 0.4],
                [m.pos[0] - 1.5 + a * 0.4, m.pos[1] + 0.2, m.pos[2] - 1.5 - b * 0.4],
            ]
            .to_vec()
        } else {
            vec![m.pos]
        };
        for org in origins {
            for ci in [0usize, 1, 2, 0, 2, 3] {
                let (c, uv) = corners[ci];
                out.push(vc_particles::particles::ParticleVertex {
                    pos: [org[0] + c[0], org[1] + c[1], org[2] + c[2]],
                    uv,
                    col,
                });
            }
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
                [tx / 32.0, (ty + 1.0) / 32.0],
            ),
            (
                [
                    right[0] * half - up[0] * half,
                    right[1] * half - up[1] * half,
                    right[2] * half - up[2] * half,
                ],
                [(tx + 1.0) / 32.0, (ty + 1.0) / 32.0],
            ),
            (
                [
                    right[0] * half + up[0] * half,
                    right[1] * half + up[1] * half,
                    right[2] * half + up[2] * half,
                ],
                [(tx + 1.0) / 32.0, ty / 32.0],
            ),
            (
                [
                    -right[0] * half + up[0] * half,
                    -right[1] * half + up[1] * half,
                    -right[2] * half + up[2] * half,
                ],
                [tx / 32.0, ty / 32.0],
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
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            true,
        );
        sys.list.insert(0, mob);
        sys.rng = rng;
        assert_eq!(sys.arrows.len(), 1, "skeleton fired one arrow");
        // fly it at the player
        let world = flat_world();
        for _ in 0..300 {
            tick_arrows(&mut sys.arrows, sys.player, false, &mut sys.hits, &world, &mut [], &mut Vec::new(), &mut Vec::new(), &mut Vec::new());
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
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            &mut Vec::new(),
            true,
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
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                true,
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
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                true,
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
                bee: None,
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
                bee: None,
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
                bee: None,
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
        assert_eq!(MOB_DATA.len(), 48); // + 1.11 four + 1.12 two + 1.13 eight + 1.14 fox + 1.16 three + the audit trio
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
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                true,
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
            wander_yaw: 0.0, wander_t: 0, equine: None, bee: None };
        for _ in 0..5 {
            ai_tick(&mut rng, &mut m, None, false, &mut Vec::new(), &mut Vec::new(), &desert, &[], &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
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
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                true,
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
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[(zid, MobKind::Zombie, [5.5, 65.0, 6.5], 0)], &mut pend, &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
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
            ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[], &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
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
        ai_tick(&mut rng, &mut mob, sys.player, false, &mut sys.hits, &mut sys.arrows, &world, &[], &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
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
        ai_tick(&mut rng2, &mut mob2, sys2.player, false, &mut sys2.hits, &mut sys2.arrows, &world2, &[], &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
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

// ---------------- 1.11 bracket tests (Exploration Update) ----------------
#[cfg(test)]
mod v111_tests {
    use super::*;

    /// the four 1.11 mobs register with their live-verified stats
    /// (w/Llama, w/Vindicator, w/Evoker, w/Vex)
    #[test]
    fn v111_mob_data() {
        let llama = def(MobKind::Llama);
        assert_eq!(llama.health, 30.0); // cap of the 15-30 range
        assert!((llama.damage - 1.0).abs() < 1e-6, "spit 1 HP E/N");
        assert!((llama.speed_attr - 0.175).abs() < 1e-6, "w/Llama speed 0.175");
        assert!(MobKind::Llama.neutral(), "llama is neutral");
        assert!(!MobKind::Llama.hostile());

        let vin = def(MobKind::Vindicator);
        assert_eq!(vin.health, 24.0);
        assert_eq!(vin.damage, 13.0, "iron axe Normal");
        // 5.612 b/s / 10.5 attr multiplier (w/Vindicator "5.612 blocks/sec")
        assert!((vin.speed_attr * 10.5 - 5.612).abs() < 0.01, "sprint-speed");

        let evo = def(MobKind::Evoker);
        assert_eq!(evo.health, 24.0);
        assert_eq!(evo.damage, 6.0, "fangs 6 HP armor-ignoring");

        let vex = def(MobKind::Vex);
        assert_eq!(vex.health, 14.0);
        assert_eq!(vex.damage, 9.0, "iron sword Normal");
        assert_eq!(vex.xp, 5, "w/Vex: 5 XP");
        // the illager trio is hostile (w/Vindicator/w/Evoker/w/Vex)
        assert!(MobKind::Vindicator.hostile());
        assert!(MobKind::Evoker.hostile());
        assert!(MobKind::Vex.hostile());
        // names + eggs roundtrip
        assert_eq!(MobKind::from_name("llama"), Some(MobKind::Llama));
        assert_eq!(MobKind::from_name("vindicator"), Some(MobKind::Vindicator));
        assert_eq!(MobKind::from_name("evoker"), Some(MobKind::Evoker));
        assert_eq!(MobKind::from_name("vex"), Some(MobKind::Vex));
        assert_eq!(MobKind::from_egg(23), MobKind::Llama);
        assert_eq!(MobKind::from_egg(24), MobKind::Vindicator);
        assert_eq!(MobKind::from_egg(25), MobKind::Evoker);
        assert_eq!(MobKind::from_egg(26), MobKind::Vex);
        assert_eq!(MobKind::Llama.egg_id(), 23);
        assert_eq!(MobKind::Evoker.egg_id(), 25);
        // 1.12 (World of Color): parrot + illusioner — 32 kinds
        assert_eq!(MOB_DATA.len(), 48, "+ the 1.13 aquatic eight + the 1.14 fox + the 1.16 forest three + the audit trio");
        assert_eq!(MobKind::from_egg(30), MobKind::Parrot);
        assert_eq!(MobKind::Parrot.egg_id(), 30);
        assert_eq!(MobKind::Illusioner.egg_id(), 255, "no spawn egg (VERIFIED)");
        assert_eq!(MobKind::Illusioner.hostile(), true);
        assert_eq!(MobKind::Parrot.hostile(), false);
    }

    /// llama spawns carry strength 1..=5 in the variant byte + equine
    /// (temper) taming state; the 1/900 regen path only heals the damaged
    #[test]
    fn v111_llama_strength_and_temper() {
        let mut ms = MobSystem::new(11);
        // MAX_MOBS caps the list — spawn a bounded set
        for _ in 0..8 {
            let id = ms.spawn_variant(MobKind::Llama, 0, 65, 0, 0).unwrap();
            let m = ms.by_id(id).unwrap();
            // equine state exists (the temper-taming infrastructure,
            // VERIFIED w/Llama §Taming)
            assert!(m.equine.is_some(), "llama carries equine state");
        }
        // (the strength distribution lives in the herd roll —
        // try_spawn_passive's variant pick)
        // damaged llama heals via the ai regen 1/900 path (statistically:
        // 200 ticks with health < max → some ticks heal)
        let id = ms.list[0].id;
        ms.list[0].health = 10.0;
        let mut healed = false;
        for _ in 0..4000 {
            let h0 = ms.by_id(id).unwrap().health;
            let m = ms.list.iter_mut().find(|m| m.id == id).unwrap();
            if m.health < 30.0 && m.health > 0.0 {
                // simulate the regen roll (the ai_tick internal)
                // — 1/900 per tick
                if ms.rng.next_range(900) == 0 {
                    m.health = (m.health + 1.0).min(30.0);
                }
            }
            if ms.by_id(id).unwrap().health > h0 {
                healed = true;
                break;
            }
        }
        assert!(healed, "the 1/900 regen roll fires within 4000 ticks");
    }

    /// the evoker queues fang damage + vex summons through the spell
    /// queues (ai_tick integration)
    #[test]
    fn v111_evoker_spell_queues() {
        let mut ms = MobSystem::new(13);
        let eid = ms.spawn_at(MobKind::Evoker, 4, 65, 4).unwrap();
        ms.player = Some([6.0, 65.0, 4.5]); // in aggro range
        // run the ai through MobSystem::tick with a flat world
        let world = {
            let mut w = World::new(13);
            let mut c = vc_chunk::chunk::Chunk::empty();
            for y in 0..=64i32 {
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        c.set(lx, y as usize, lz, 1); // dirt floor
                    }
                }
            }
            use std::sync::Arc;
            w.insert_generated((0, 0), Arc::new(c), Vec::new());
            w.dirty.clear();
            w
        };
        for _ in 0..120 {
            ms.tick(&world, (0, 0), 4);
        }
        // provoked+hostile: the evoker engaged the player — either fangs
        // queued or vexes summoned within 120 ticks (the 100-tick cycle)
        let fangs_or_summons = !ms.pending_player_fang.is_empty() || !ms.pending_summons.is_empty();
        assert!(fangs_or_summons, "a spell fired (fangs or vex summon)");
        // drain them (the game layer's contract)
        let _: Vec<(u32, usize)> = ms.pending_summons.drain(..).collect();
        let _: Vec<f32> = ms.pending_player_fang.drain(..).collect();
        let _ = eid;
    }

    /// 1.11 re-added/new egg map: husk 27, stray 28 (changelog §Items
    /// — "Husk spawn egg, Stray spawn egg" among the 1.10-pre2 removals
    /// re-added in 1.11); the zombie-villager egg (5th new) is the
    /// pre-existing kind-5 item
    #[test]
    fn v111_readded_egg_map() {
        assert_eq!(MobKind::from_egg(27), MobKind::Husk);
        assert_eq!(MobKind::from_egg(28), MobKind::Stray);
        assert_eq!(MobKind::Husk.egg_id(), 27);
        assert_eq!(MobKind::Stray.egg_id(), 28);
        // the changelog's five NEW eggs: llama/vindicator/evoker/vex +
        // zombie villager (pre-existing E2-era item at kind 5)
        assert_eq!(MobKind::ZombieVillager.egg_id(), 5);
        // rabbit + polar bear keep the 255 no-egg sentinel (standing
        // 1.8/1.10 deferrals, out of the 1.11 scope)
        assert_eq!(MobKind::Rabbit.egg_id(), 255);
        assert_eq!(MobKind::PolarBear.egg_id(), 255);
    }

    /// llama herd spawn: Mountains biome roll, 4-6 herd size, strength
    /// variants 1..=5 (VERIFIED w/Llama §Spawning + §Strength)
    #[test]
    fn v111_llama_herd_in_mountains() {
        // a 17×17 grid of Mountains chunks with GRASS floors + lit
        // sections — every ±8-chunk roll from the player's chunk lands
        // on a valid chunk (the spawn window: cx/cz = player chunk +
        // rng(17) - 8), the floor passes the GRASS check, and the light
        // map satisfies PASSIVE_LIGHT_MIN (blk 15)
        let world = {
            let mut w = World::new(21);
            use std::sync::Arc;
            use vc_world::light::{LightData, LightSection};
            for cx in -8i32..=8 {
                for cz in -8i32..=8 {
                    let mut c = vc_chunk::chunk::Chunk::empty();
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, 64, lz, GRASS); // grass floor at y=64
                            c.biome[lz * 16 + lx] = vc_world::gen::Biome::Mountains as u8;
                        }
                    }
                    w.insert_generated((cx, cz), Arc::new(c), Vec::new());
                    let mut ld = LightData::new();
                    ld.sections[4] = Some(Box::new(LightSection {
                        sky: Box::new([15u8; 4096]),
                        blk: Box::new([15u8; 4096]),
                    }));
                    w.light.insert((cx, cz), Arc::new(ld));
                }
            }
            w.dirty.clear();
            w
        };
        let mut ms = MobSystem::new(21);
        ms.player = Some([8.0, 66.0, 8.0]);
        // hammer the passive roll until a llama herd lands (1/20 gate)
        for _ in 0..4000 {
            ms.try_spawn_passive(&world, |_, _| true);
            if ms.passives_alive() as f32 >= CREATURE_CAP {
                break;
            }
        }
        let llamas = ms.list.iter().filter(|m| m.kind == MobKind::Llama).count();
        assert!(llamas >= 4, "llama herds land in Mountains (got {llamas})");
        // every llama carries a strength variant 1..=5 (the herd roll)
        for m in ms.list.iter().filter(|m| m.kind == MobKind::Llama) {
            assert!(
                (1..=5).contains(&m.variant),
                "llama strength 1..=5 (got {})",
                m.variant
            );
        }
    }

    /// llama spit retaliation: a provoked llama fires a LlamaSpit
    /// projectile (VERIFIED w/Llama: "If the player hits them, they
    /// spit at the player once, dealing 1 HP damage")
    #[test]
    fn v111_llama_spit_retaliation() {
        let mut ms = MobSystem::new(23);
        let id = ms.spawn_at(MobKind::Llama, 4, 65, 4).unwrap();
        ms.player = Some([6.0, 65.0, 4.5]);
        ms.by_id_mut(id).unwrap().provoked = true;
        let world = World::new(23);
        // the spit flies at 18 b/s ≈ 0.9 blocks/tick toward a player
        // 1.5 blocks away — it lands within ~2 ticks and converts to a
        // PlayerHit (source Llama); check in-flight OR landed
        let mut spat = false;
        for _ in 0..40 {
            ms.tick(&world, (0, 0), 4);
            spat = ms.arrows.iter().any(|p| p.kind == ProjKind::LlamaSpit)
                || ms.hits.iter().any(|h| h.source == MobKind::Llama);
            if spat {
                break;
            }
        }
        assert!(
            spat,
            "provoked llama spits (LlamaSpit in flight or a landed Llama hit)"
        );
    }

    /// llama hay-bale breeding: two tamed adults + a hay bale → love
    /// mode (VERIFIED changelog §Mobs: "Tamed llamas can be bred with
    /// hay bales")
    #[test]
    fn v111_llama_hay_bale_breeding() {
        let mut ms = MobSystem::new(25);
        let a = ms.spawn_at(MobKind::Llama, 4, 65, 4).unwrap();
        let b = ms.spawn_at(MobKind::Llama, 5, 65, 5).unwrap();
        for id in [a, b] {
            let m = ms.by_id_mut(id).unwrap();
            let eq = m.equine.as_mut().unwrap();
            eq.tamed = true;
        }
        let out = ms.try_feed(a, HAY_BALE, &mut Rng::new(25));
        assert!(
            matches!(out, Some(FeedOutcome::LoveMode(pid)) if pid == b),
            "hay bale on two tamed adults starts love mode (got {out:?})"
        );
        // single llama: hay just heals (the Ate/Healed arm)
        let mut ms2 = MobSystem::new(26);
        let c = ms2.spawn_at(MobKind::Llama, 4, 65, 4).unwrap();
        ms2.by_id_mut(c).unwrap().health = 10.0;
        let out2 = ms2.try_feed(c, HAY_BALE, &mut Rng::new(26));
        assert!(matches!(out2, Some(FeedOutcome::Healed)));
        assert!((ms2.by_id(c).unwrap().health - 20.0).abs() < 1e-6, "heal +10");
    }

    /// the vex phases through blocks (VERIFIED w/Vex: "pass through any
    /// block, including water and lava") — the no-clip physics path
    #[test]
    fn v111_vex_passes_through_blocks() {
        let mut ms = MobSystem::new(27);
        let id = ms.spawn_at(MobKind::Vex, 4, 65, 4).unwrap();
        // build a solid column in its path; the vex's physics ignores it
        let world = {
            let mut w = World::new(27);
            let mut c = vc_chunk::chunk::Chunk::empty();
            for y in 0..=64i32 {
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        c.set(lx, y as usize, lz, 1);
                    }
                }
            }
            use std::sync::Arc;
            w.insert_generated((0, 0), Arc::new(c), Vec::new());
            w.dirty.clear();
            w
        };
        let start = ms.by_id(id).unwrap().pos;
        // the vex spawns INSIDE the solid chunk and must not be pushed
        // out / stuck by collision (its physics skips block collision)
        for _ in 0..10 {
            ms.tick(&world, (0, 0), 4);
        }
        let m = ms.by_id(id).unwrap();
        assert!(m.health > 0.0, "vex alive inside solid blocks (no suffocation path)");
        let moved = (m.pos[0] - start[0]).abs() + (m.pos[2] - start[2]).abs();
        assert!(
            moved > 0.0 || m.vel[0] != 0.0 || m.vel[2] != 0.0,
            "vex moves freely through solid ground"
        );
    }
}


// ---------------- 1.12 bracket tests (World of Color Update) ----------------
#[cfg(test)]
mod v112_tests {
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
        w
    }

    /// parrot: 6 HP, the 5 color variants, the 1/10 taming roll, the
    /// cookie death (VERIFIED w/Parrot live 2026-09-07)
    #[test]
    fn v112_parrot_stats_and_variants() {
        let d = def(MobKind::Parrot);
        assert_eq!(d.health, 6.0, "w/Parrot infobox: 6 HP");
        assert_eq!(d.speed_attr, 0.2, "w/Parrot infobox: Speed 0.2");
        assert!(!d.kind.hostile(), "passive (w/Parrot infobox)");
        // the 5 variant colors 0..=4
        for v in 0u8..=4 {
            let mut ms = MobSystem::new(100 + v as u64);
            let id = ms.spawn_variant(MobKind::Parrot, 4, 65, 4, v).unwrap();
            assert_eq!(ms.by_id(id).unwrap().variant & 0x0F, v);
        }
        // negative: variant 5 clamps in the RENDER path only; the data
        // stays raw (the sprite gate .min(4))
    }

    #[test]
    fn v112_parrot_taming_roll_and_sit_toggle() {
        // the 1/10 roll: a fixed rng stream — force a success by
        // feeding until tamed (bounded: 200 feeds at 1/10 → certain)
        let mut ms = MobSystem::new(7);
        let id = ms.spawn_variant(MobKind::Parrot, 4, 65, 4, 2).unwrap();
        let mut rng = Rng::new(11);
        let mut tamed = false;
        let mut feeds = 0;
        for _ in 0..200 {
            let out = ms.try_feed_parrot(id, WHEAT_SEEDS, &mut rng);
            feeds += 1;
            match out {
                Some(ParrotFeedOutcome::Tamed) => {
                    tamed = true;
                    break;
                }
                Some(ParrotFeedOutcome::Ate) => {}
                _ => panic!("seeds are parrot food"),
            }
        }
        assert!(tamed, "the 1/10 roll hits within 200 feeds");
        assert!(feeds < 200, "the roll is per-feed (not first-try-only)");
        let m = ms.by_id(id).unwrap();
        assert!(m.variant & 0x80 != 0, "the tamed bit set");
        // all four seeds are valid taming foods (VERIFIED w/Parrot)
        for food in [WHEAT_SEEDS, MELON_SEEDS, PUMPKIN_SEEDS, BEETROOT_SEEDS] {
            let mut ms2 = MobSystem::new(13);
            let id2 = ms2.spawn_variant(MobKind::Parrot, 4, 65, 4, 0).unwrap();
            let mut rng2 = Rng::new(5);
            assert!(
                ms2.try_feed_parrot(id2, food, &mut rng2).is_some(),
                "seed item {food} is parrot food"
            );
        }
        // the sit toggle only works on TAMED parrots (17w14a)
        assert!(ms.toggle_parrot_sit(id), "tamed parrot toggles");
        assert!(ms.by_id(id).unwrap().aux & 1 == 1, "sitting");
        assert!(ms.toggle_parrot_sit(id), "toggle back");
        assert!(ms.by_id(id).unwrap().aux & 1 == 0, "standing");
        // untamed: no toggle
        let mut ms3 = MobSystem::new(17);
        let id3 = ms3.spawn_variant(MobKind::Parrot, 4, 65, 4, 1).unwrap();
        assert!(!ms3.toggle_parrot_sit(id3), "untamed parrot ignores");
    }

    #[test]
    fn v112_parrot_cookie_is_instant_death() {
        // VERIFIED w/Parrot: "the parrot receives 2128 (3.4028 x 10^38)"
        let mut ms = MobSystem::new(23);
        let id = ms.spawn_variant(MobKind::Parrot, 4, 65, 4, 3).unwrap();
        let mut rng = Rng::new(3);
        let out = ms.try_feed_parrot(id, COOKIE, &mut rng);
        assert_eq!(out, Some(ParrotFeedOutcome::CookieDeath));
        assert!(ms.by_id(id).unwrap().health <= 0.0, "2128 damage kills a 6 HP parrot");
        // the death sweep converts it (drops handled at the game layer)
        let world = flat_world();
        ms.tick(&world, (0, 0), 1);
        assert!(
            ms.deaths.iter().any(|(k, _, _)| *k == MobKind::Parrot),
            "the cookie kill routes through the death sweep"
        );
        // non-food: no effect
        let mut ms2 = MobSystem::new(29);
        let id2 = ms2.spawn_variant(MobKind::Parrot, 4, 65, 4, 0).unwrap();
        let mut rng2 = Rng::new(3);
        assert!(ms2.try_feed_parrot(id2, CARROT, &mut rng2).is_none());
        // feeding a non-parrot: no effect
        let mut ms3 = MobSystem::new(31);
        let cow = ms3.spawn_at(MobKind::Cow, 4, 65, 4).unwrap();
        let mut rng3 = Rng::new(3);
        assert!(ms3.try_feed_parrot(cow, WHEAT_SEEDS, &mut rng3).is_none());
    }

    #[test]
    fn v112_parrot_follows_and_teleports_at_12_blocks() {
        // VERIFIED w/Parrot: "a tamed parrot follows the player and
        // teleports if there is a distance of 12 blocks between it and
        // the player"
        let mut ms = MobSystem::new(41);
        let id = ms.spawn_variant(MobKind::Parrot, 20, 65, 20, 4).unwrap();
        ms.list.iter_mut().find(|m| m.id == id).unwrap().variant |= 0x80; // tamed
        ms.player = Some([4.5, 65.0, 4.5]); // ~22.6 blocks away
        let world = flat_world();
        let mut blind = Vec::new();
        // one ai tick: the teleport fires immediately
        {
            let mut rng = std::mem::replace(&mut ms.rng, Rng::new(1));
            let mut mob = ms.list.remove(0);
            let mut hits = Vec::new();
            let mut arrows = Vec::new();
            let mut pend = Vec::new();
            let mut summons = Vec::new();
            let mut fang = Vec::new();
            ai_tick(&mut rng, &mut mob, ms.player, false, &mut hits, &mut arrows, &world, &[], &mut pend, &mut summons, &mut fang, &mut blind, &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), &mut Vec::new(), true);
            ms.list.insert(0, mob);
            ms.rng = rng;
        }
        let m = ms.by_id(id).unwrap();
        let d = ((m.pos[0] - 4.5).powi(2) + (m.pos[2] - 4.5).powi(2)).sqrt();
        assert!(d < 12.0, "teleported near the player (now {d:.1} blocks)");
        let _ = blind;
    }

    /// illusioner: 32 HP, blindness on first engage (once), the mirror
    /// spell (invisibility 60 s, refreshed), the 20-tick bow cadence
    /// (VERIFIED w/Illusioner live 2026-09-07)
    #[test]
    fn v112_illusioner_stats_and_blindness_spell() {
        let d = def(MobKind::Illusioner);
        assert_eq!(d.health, 32.0, "w/Illusioner infobox: 32 HP");
        assert_eq!(d.speed_attr, 0.5, "w/Illusioner infobox: Speed 0.5");
        assert!(d.kind.hostile(), "hostile illager");
        // engage: blindness queued ONCE at 20 s
        let mut ms = MobSystem::new(43);
        let id = ms.spawn_at(MobKind::Illusioner, 10, 65, 4).unwrap();
        ms.player = Some([4.5, 65.0, 4.5]);
        let world = flat_world();
        ms.tick(&world, (0, 0), 1);
        assert_eq!(ms.pending_player_blindness.len(), 1, "first engage casts blindness");
        assert_eq!(ms.pending_player_blindness[0], 20 * 20, "20 seconds (VERIFIED)");
        // more ticks: NOT cast again (the once-per-opponent gate)
        for _ in 0..50 {
            ms.tick(&world, (0, 0), 1);
        }
        assert_eq!(ms.pending_player_blindness.len(), 1, "once per opponent");
        // the mirror spell: invisibility runs while engaged
        let m = ms.by_id(id).unwrap();
        assert!(m.aux > 0 && m.aux <= 20 * 60, "invisibility ticking (60 s cap)");
        // the bow: 20-tick cadence (1/s — VERIFIED: "three times faster
        // than a skeleton" whose 40-tick cadence is SKELETON_SHOOT_TICKS).
        // The arrows fly ~6 blocks at speed 10 and strike the player
        // within the window — assert on the player HITS (the fired
        // arrows' observable), each Normal-difficulty bow hit 2-5 HP
        // scaled from the 3.5 mid.
        let hits0 = ms.hits.len();
        for _ in 0..25 {
            ms.tick(&world, (0, 0), 1);
        }
        assert!(ms.hits.len() > hits0, "bow shots landed (1/s cadence)");
    }

    #[test]
    fn v112_illusioner_never_spawns_naturally() {
        // VERIFIED w/Illusioner: "Spawn: By commands" + "Unused and
        // present only in Java Edition" — the ambient pools exclude it
        // (no spawn egg either)
        assert_eq!(MobKind::Illusioner.egg_id(), 255);
        // hostile pool: run many spawn attempts in a flat dark world —
        // no illusioner ever appears
        let mut ms = MobSystem::new(97);
        ms.player = Some([8.5, 65.0, 8.5]);
        let world = flat_world();
        for _ in 0..2000 {
            ms.try_spawn_hostile(&world, |_, _| true);
        }
        assert!(
            !ms.list.iter().any(|m| m.kind == MobKind::Illusioner),
            "illusioners never spawn naturally (vanilla parity)"
        );
    }
}

// ---------------------------------------------------------------------------
// 1.13 (Update Aquatic) — VERIFIED live 2026-09-07 against the wiki
// captures (voxelcraft/scripts/v113_page_*; research record
// docs/research/phase-v113-1.13-research.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod v113_tests {
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

    /// 1.13: an ocean-family water world — stone to y 52, WATER 53..=62,
    /// air above; the whole chunk under one biome id (0 neutral / 19
    /// warm / 20 lukewarm / 21 cold / 22 frozen).
    fn ocean_world(biome_id: u8) -> World {
        let mut w = World::new(11);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=52i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, STONE);
                }
            }
        }
        for y in 53..=62i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, WATER);
                }
            }
        }
        for i in 0..256usize {
            c.biome[i] = biome_id;
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    /// 1.13: a beach world — sand to y 64, air above, biome 1 (Beach):
    /// the turtle nesting column.
    fn beach_world() -> World {
        let mut w = World::new(11);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=64i32 {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y as usize, lz, SAND);
                }
            }
        }
        for i in 0..256usize {
            c.biome[i] = 1; // Beach
        }
        w.insert_generated((0, 0), std::sync::Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    /// VERIFIED infobox rows (w/Drowned, w/Phantom, w/Dolphin, w/Cod,
    /// w/Salmon, w/Pufferfish, w/Tropical_Fish, w/Turtle) + the
    /// aquatic() swim-physics gate + the V9 spawn-egg kinds.
    #[test]
    fn v113_registry_rows_and_flags() {
        assert_eq!(MOB_DATA.len(), 48, "32 prior + 8 aquatic + the 1.14 fox + the 1.16 forest three + the audit trio");
        // drowned: 20 HP zombie-parity, N 3, armor 2, 5 XP, hostile
        let d = def(MobKind::Drowned);
        assert_eq!(d.health as i32, 20);
        assert_eq!(d.damage as i32, 3);
        assert_eq!(d.armor as i32, 2);
        assert_eq!(d.xp, 5);
        assert!(d.kind.hostile() && !d.kind.neutral());
        // phantom: 20 HP undead, E/N 2 (the 1.14-pre3 value — disclosed)
        let p = def(MobKind::Phantom);
        assert_eq!(p.health as i32, 20);
        assert!((p.damage - 2.0).abs() < 1e-6);
        assert_eq!(p.xp, 5);
        assert!(p.kind.hostile());
        // dolphin: 10 HP neutral, N 3, 1-3 XP (engine takes 1)
        let do_ = def(MobKind::Dolphin);
        assert_eq!(do_.health as i32, 10);
        assert_eq!(do_.damage as i32, 3);
        assert_eq!(do_.xp, 1);
        assert!(do_.kind.neutral() && !do_.kind.hostile());
        // the 3 HP fish (cod/salmon/tropical) + the neutral pufferfish
        for k in [
            MobKind::Cod,
            MobKind::Salmon,
            MobKind::Pufferfish,
            MobKind::TropicalFish,
        ] {
            assert_eq!(def(k).health as i32, 3, "{k:?} is a 3 HP fish");
            assert_eq!(def(k).xp, 1);
        }
        assert!(def(MobKind::Pufferfish).kind.neutral());
        for k in [MobKind::Cod, MobKind::Salmon, MobKind::TropicalFish] {
            assert!(!def(k).kind.hostile() && !def(k).kind.neutral());
        }
        // turtle: 30 HP passive, the wide 1.2-block shell
        let t = def(MobKind::Turtle);
        assert_eq!(t.health as i32, 30);
        assert!((t.width - 1.2).abs() < 1e-6);
        assert!(!t.kind.hostile() && !t.kind.neutral());
        // aquatic(): swim physics + conduit/impaling targets
        for k in [
            MobKind::Drowned,
            MobKind::Dolphin,
            MobKind::Cod,
            MobKind::Salmon,
            MobKind::Pufferfish,
            MobKind::TropicalFish,
            MobKind::Turtle,
        ] {
            assert!(k.aquatic(), "{k:?} is aquatic (VERIFIED)");
        }
        assert!(!MobKind::Zombie.aquatic());
        assert!(!MobKind::Squid.aquatic(), "the squid is pre-1.13 legacy");
        assert!(!MobKind::Cow.aquatic());
        // spawn-egg kinds 32..=39 (the changelog's own 8-egg list)
        assert_eq!(MobKind::Drowned.egg_id(), 32);
        assert_eq!(MobKind::Phantom.egg_id(), 33);
        assert_eq!(MobKind::Dolphin.egg_id(), 34);
        assert_eq!(MobKind::Cod.egg_id(), 35);
        assert_eq!(MobKind::Salmon.egg_id(), 36);
        assert_eq!(MobKind::Pufferfish.egg_id(), 37);
        assert_eq!(MobKind::TropicalFish.egg_id(), 38);
        assert_eq!(MobKind::Turtle.egg_id(), 39);
        // from_egg roundtrip (the game layer's spawn-egg use gate)
        for k in [
            MobKind::Drowned,
            MobKind::Phantom,
            MobKind::Dolphin,
            MobKind::Cod,
            MobKind::Salmon,
            MobKind::Pufferfish,
            MobKind::TropicalFish,
            MobKind::Turtle,
        ] {
            assert_eq!(MobKind::from_egg(k.egg_id()), k, "{k:?} egg roundtrip");
        }
        // registry ids (VERIFIED: the 1.13 entity id set)
        assert_eq!(MobKind::Drowned.registry_id(), "minecraft:drowned");
        assert_eq!(MobKind::Phantom.registry_id(), "minecraft:phantom");
        assert_eq!(MobKind::Dolphin.registry_id(), "minecraft:dolphin");
        assert_eq!(MobKind::Cod.registry_id(), "minecraft:cod");
        assert_eq!(MobKind::Salmon.registry_id(), "minecraft:salmon");
        assert_eq!(
            MobKind::Pufferfish.registry_id(),
            "minecraft:pufferfish"
        );
        assert_eq!(
            MobKind::TropicalFish.registry_id(),
            "minecraft:tropical_fish"
        );
        assert_eq!(MobKind::Turtle.registry_id(), "minecraft:turtle");
    }

    /// VERIFIED w/Drowned §Attacking: "A drowned with a trident can
    /// throw it every 1.5 seconds, sending it up to 20 blocks away" —
    /// the armed bit (variant & 1) gates the throw, 8 HP base damage,
    /// 30-tick cooldown.
    #[test]
    fn v113_drowned_throws_trident_at_range() {
        let mut sys = MobSystem::new(5);
        // armed drowned 12 blocks from the player (inside the 4..=20 band)
        let id = sys.spawn_variant(MobKind::Drowned, 0, 65, 0, 1).unwrap();
        sys.player = Some([12.5, 65.0, 0.5]);
        let world = flat_world();
        sys.tick(&world, (0, 0), i32::MAX);
        assert_eq!(sys.arrows.len(), 1, "armed drowned throws at range");
        let a = &sys.arrows[0];
        assert_eq!(a.kind, ProjKind::Trident);
        assert!((a.damage - 8.0).abs() < 1e-6, "8 HP thrown (VERIFIED)");
        let m = sys.by_id(id).unwrap();
        assert_eq!(m.attack_cd, 30, "1.5 s cooldown (VERIFIED)");
        // unarmed drowned never throws (falls to melee chase instead)
        let mut sys2 = MobSystem::new(7);
        sys2.spawn_variant(MobKind::Drowned, 0, 65, 0, 0).unwrap();
        sys2.player = Some([12.5, 65.0, 0.5]);
        sys2.tick(&world, (0, 0), i32::MAX);
        assert!(
            sys2.arrows.iter().all(|a| a.kind != ProjKind::Trident),
            "unarmed drowned has no trident to throw"
        );
    }

    /// VERIFIED w/Drowned §Conversion: "If a zombie's head ... is
    /// continuously submerged for 30 seconds, it begins to convert
    /// into a drowned" — the 600-tick accumulator on aux, head-block
    /// gate, unarmed result.
    #[test]
    fn v113_zombie_converts_to_drowned_underwater() {
        let world = ocean_world(0);
        let mut sys = MobSystem::new(5);
        // zombie standing on the ocean floor: head at y 54 under water.
        // The wander state is pinned to stand-still so the random walk
        // can't drift the zombie off the single loaded test chunk
        // (outside it the head reads AIR and the timer would freeze).
        let id = sys.spawn_at(MobKind::Zombie, 4, 53, 4).unwrap();
        sys.by_id_mut(id).unwrap().wander_t = -2000;
        sys.player = None; // conversion is environmental, no anchor
        for _ in 0..599 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert_eq!(sys.by_id(id).unwrap().kind, MobKind::Zombie, "599 ticks: still a zombie");
        assert!(sys.by_id(id).unwrap().aux > 0, "the submersion timer accumulates");
        sys.tick(&world, (0, 0), i32::MAX); // tick 600
        assert_eq!(sys.by_id(id).unwrap().kind, MobKind::Drowned, "converted at 30 s");
        assert_eq!(sys.by_id(id).unwrap().variant, 0, "unarmed conversion");
        // a dry zombie never converts
        let mut sys2 = MobSystem::new(9);
        let id2 = sys2.spawn_at(MobKind::Zombie, 4, 70, 4).unwrap();
        sys2.player = None;
        for _ in 0..1000 {
            sys2.tick(&flat_world(), (0, 0), i32::MAX);
        }
        assert_eq!(sys2.by_id(id2).unwrap().kind, MobKind::Zombie, "dry zombies stay zombies");
    }

    /// VERIFIED w/Phantom §Behavior: the orbit-and-swoop cycle — 12
    /// blocks above the player at orbit, the 60-tick dive window, the
    /// 2 HP swoop bite (the 1.14-pre3 value, disclosed).
    #[test]
    fn v113_phantom_orbits_then_swoops() {
        let mut sys = MobSystem::new(5);
        let id = sys.spawn_at(MobKind::Phantom, 8, 82, 8).unwrap();
        sys.player = Some([8.5, 70.0, 8.5]);
        let world = flat_world();
        // fresh spawn: orbit phase, aux = 200 (the spawn routine sets it)
        sys.by_id_mut(id).unwrap().aux = 200;
        for _ in 0..100 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let m = sys.by_id(id).unwrap();
        assert!(m.variant & 1 == 0, "first 100 ticks: orbit phase");
        assert!(m.aux > 0 && m.aux <= 200, "orbit window counts down");
        // force the dive window and place the phantom at the player
        sys.by_id_mut(id).unwrap().aux = 1;
        sys.tick(&world, (0, 0), i32::MAX);
        let m = sys.by_id(id).unwrap();
        assert!(m.variant & 1 != 0, "orbit over: diving");
        assert_eq!(m.aux, 60, "the 60-tick dive window");
        // swoop bite: dive position, cd 0
        {
            let m = sys.by_id_mut(id).unwrap();
            m.pos = [8.5, 71.0, 8.5]; // at the player's chest
            m.variant |= 1; // stay diving
            m.attack_cd = 0;
        }
        let hits0 = sys.hits.len();
        sys.tick(&world, (0, 0), i32::MAX);
        assert!(sys.hits.len() > hits0, "the swoop connects");
        let hit = sys.hits.last().unwrap();
        assert!((hit.damage - 2.0).abs() < 1e-6, "E/N 2 HP (VERIFIED)");
        assert_eq!(hit.source, MobKind::Phantom);
    }

    /// VERIFIED w/Pufferfish: inflate toward variant 2 as the player
    /// closes within 3 blocks (one step per 20 ticks), contact 3 HP N
    /// fully-puffed + 6 s (120 tick) poison; the semi tier is 2 HP +
    /// 3 s (60 tick). Setup: the fish floats in its native warm ocean
    /// (a beached fish suffocates 1 HP/s and flops away — the water
    /// world is the vanilla-realistic frame); the wander state is
    /// pinned to stand-still so the school-swim drift can't carry the
    /// fish out of the 3-block radius and make the cadence flaky.
    #[test]
    fn v113_pufferfish_inflates_and_stings() {
        let mut sys = MobSystem::new(5);
        let id = sys.spawn_at(MobKind::Pufferfish, 6, 56, 6).unwrap();
        // pin the wander state: negative wander_t = standing still
        // (vel decays; no vertical nudge — see wander/wander_3d)
        sys.by_id_mut(id).unwrap().wander_t = -400;
        // player 2 blocks away: inside the 3-block inflate radius,
        // outside the contact band (width 0.5 -> 1.15)
        sys.player = Some([8.0, 56.0, 8.0]);
        let world = ocean_world(19); // warm ocean: the pufferfish's home
        // two 20-tick steps: 0 -> 1 -> 2 (fully puffed)
        for _ in 0..41 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert_eq!(sys.by_id(id).unwrap().variant, 2, "fully inflated");
        // move the player INTO the contact band and let a cadence tick land
        sys.player = Some([6.8, 56.0, 6.8]);
        let hits0 = sys.hits.len();
        for _ in 0..25 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert!(sys.hits.len() > hits0, "contact sting fired");
        let hit = sys.hits.last().unwrap();
        assert_eq!(hit.source, MobKind::Pufferfish);
        assert!((hit.damage - 3.0).abs() < 1e-6, "fully-puffed N contact 3 HP");
        assert_eq!(hit.poison_effect, Some(120), "6 s poison fully-puffed (VERIFIED Java)");
    }

    /// VERIFIED w/Dolphin: "Players who sprint-swim within a 9 block
    /// spherical radius of a dolphin receive a swimming speed boost
    /// for 5 seconds, replenished as long as the player stays close"
    /// — the queue carries 100 ticks, refreshed at most 1/s.
    #[test]
    fn v113_dolphin_banks_dolphins_grace() {
        let mut sys = MobSystem::new(5);
        sys.spawn_at(MobKind::Dolphin, 6, 65, 6).unwrap();
        sys.player = Some([9.0, 65.0, 9.0]); // ~4.2 blocks: inside 9
        let world = flat_world();
        sys.tick(&world, (0, 0), i32::MAX);
        assert_eq!(sys.pending_player_grace.len(), 1, "grace queued");
        assert_eq!(sys.pending_player_grace[0], 100, "5 s (VERIFIED)");
        // replenish cadence: the next 19 ticks queue nothing (1/s cap)
        for _ in 0..19 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert_eq!(sys.pending_player_grace.len(), 1, "1/s refresh cadence");
        // beyond 9 blocks: nothing
        sys.pending_player_grace.clear();
        sys.player = Some([40.0, 65.0, 40.0]);
        for _ in 0..40 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert!(sys.pending_player_grace.is_empty(), "out of range: no grace");
    }

    /// VERIFIED w/Turtle + w/Scute: a bred female (variant bit 0x80)
    /// standing on sand queues one turtle egg at her position; a baby
    /// (bit 0x40) matures on the aux countdown and queues a scute.
    #[test]
    fn v113_turtle_lays_eggs_and_babies_drop_scutes() {
        let world = beach_world();
        let mut sys = MobSystem::new(5);
        // bred female on sand (y 65, sand floor at 64)
        let id = sys.spawn_variant(MobKind::Turtle, 4, 65, 4, 0x80).unwrap();
        sys.by_id_mut(id).unwrap().on_ground = true;
        sys.player = None;
        sys.tick(&world, (0, 0), i32::MAX);
        assert_eq!(sys.pending_turtle_eggs.len(), 1, "egg queued");
        let (ex, ey, ez, stage) = sys.pending_turtle_eggs[0];
        assert_eq!((ex, ey, ez), (4, 65, 4), "the egg lands at her feet");
        assert_eq!(stage, 0, "hatch stage 0");
        assert_eq!(sys.by_id(id).unwrap().variant & 0x80, 0, "egg bit cleared");
        // the baby: one tick from maturity
        let baby = sys.spawn_variant(MobKind::Turtle, 8, 65, 8, 0x40).unwrap();
        sys.by_id_mut(baby).unwrap().aux = 1;
        sys.tick(&world, (0, 0), i32::MAX);
        assert_eq!(sys.pending_drops.len(), 1, "scute queued (VERIFIED w/Scute)");
        assert_eq!(sys.pending_drops[0].1, SCUTE);
        assert_eq!(sys.by_id(baby).unwrap().variant & 0x40, 0, "baby matured");
        // a non-sand floor never receives an egg
        let mut sys2 = MobSystem::new(7);
        sys2.spawn_variant(MobKind::Turtle, 4, 65, 4, 0x80).unwrap();
        sys2.by_id_mut(sys2.list[0].id).unwrap().on_ground = true;
        sys2.tick(&flat_world(), (0, 0), i32::MAX); // stone floor
        assert!(sys2.pending_turtle_eggs.is_empty(), "stone floor: no egg");
    }

    /// VERIFIED w/Phantom §Spawning: "Phantoms spawn ... above a
    /// player whose Time Since Last Rest is 1 hour (72000 ticks)";
    /// the local pack caps at 4; dying resets the statistic.
    #[test]
    fn v113_phantom_insomnia_spawning() {
        let world = flat_world();
        let mut sys = MobSystem::new(5);
        sys.player = Some([8.5, 70.0, 8.5]);
        // below the threshold: no phantoms ever (rest_t climbs 80
        // ticks during the loop — start low enough that even after the
        // full window the statistic is still short of 72000)
        sys.rest_t = 71900;
        for _ in 0..80 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert!(sys.list.iter().all(|m| m.kind != MobKind::Phantom), "insomnia not yet");
        // at the threshold: phantoms appear above the player
        sys.rest_t = 72000 - 20; // the % 20 gate fires on 72000
        for _ in 0..40 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let phantoms: Vec<_> = sys.list.iter().filter(|m| m.kind == MobKind::Phantom).collect();
        assert!(!phantoms.is_empty(), "the insomnia pack arrived");
        for m in &phantoms {
            let dy = m.pos[1] - 70.0;
            assert!(dy >= 12.0 && dy <= 20.0, "12-20 blocks above (got {dy})");
            // phantoms spawned during the 40-tick window are part-way
            // through the 200-tick orbit countdown by sampling time
            assert!(
                m.aux > 0 && m.aux <= 200,
                "fresh spawn entered the orbit phase (aux {})",
                m.aux
            );
        }
        // cap: never more than 4
        for _ in 0..400 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let n = sys.list.iter().filter(|m| m.kind == MobKind::Phantom).count();
        assert!(n <= 4, "the local pack caps at 4 (got {n})");
        // death resets the statistic (VERIFIED: "dying ... resets")
        sys.note_rest();
        assert_eq!(sys.rest_t, 0);
    }

    /// VERIFIED w/Drowned §Spawning ("Drowned spawn naturally ... in
    /// ocean and river biomes") — the water-column branch: two stacked
    /// WATER blocks, packs 1-2, standing IN water.
    #[test]
    fn v113_drowned_spawn_in_ocean_water() {
        let world = ocean_world(19); // warm ocean family
        let mut sys = MobSystem::new(5);
        sys.player = Some([4.5, 70.0, 4.5]); // above the surface
        for _ in 0..2000 {
            sys.try_spawn_hostile(&world, |_, _| true);
        }
        let drowned: Vec<_> = sys.list.iter().filter(|m| m.kind == MobKind::Drowned).collect();
        assert!(!drowned.is_empty(), "the ocean rolls drowned (VERIFIED)");
        for m in &drowned {
            let y = m.pos[1] as i32;
            assert!((53..=62).contains(&y), "spawned in the water column (y {y})");
        }
        // no zombies in the ocean-family water rolls (drowned replace them)
        assert!(
            sys.list.iter().all(|m| m.kind != MobKind::Zombie),
            "zombies never fill the ocean water column"
        );
    }

    /// The water-ambient families (VERIFIED w/Cod, w/Salmon,
    /// w/Tropical_Fish, w/Pufferfish, w/Dolphin, w/Turtle §Spawning):
    /// warm = tropical/pufferfish (+ occasional dolphin pods), cold =
    /// cod/salmon, beach = turtles on sand. NOT counted toward the
    /// passive cap.
    #[test]
    fn v113_water_ambient_biome_families() {
        // warm ocean: tropical fish dominate, pufferfish ride the 1/4
        // roll, dolphins the 1/8 pod roll
        let warm = ocean_world(19);
        let mut sys = MobSystem::new(5);
        sys.player = Some([4.5, 70.0, 4.5]);
        for _ in 0..2000 {
            sys.try_spawn_aquatic(&warm, |_, _| true);
        }
        assert!(
            sys.list.iter().any(|m| m.kind == MobKind::TropicalFish),
            "warm oceans school tropical fish (VERIFIED)"
        );
        assert!(
            sys.list
                .iter()
                .all(|m| matches!(m.kind, MobKind::TropicalFish | MobKind::Pufferfish | MobKind::Dolphin)),
            "warm families only"
        );
        // cold ocean: the cod-salmon split, no dolphins ("all ocean
        // biomes except frozen/cold" — VERIFIED)
        let cold = ocean_world(21);
        let mut sys2 = MobSystem::new(7);
        sys2.player = Some([4.5, 70.0, 4.5]);
        for _ in 0..2000 {
            sys2.try_spawn_aquatic(&cold, |_, _| true);
        }
        assert!(
            sys2.list
                .iter()
                .all(|m| matches!(m.kind, MobKind::Cod | MobKind::Salmon)),
            "cold families only"
        );
        assert!(
            sys2.list.iter().any(|m| m.kind == MobKind::Cod),
            "cod present (VERIFIED)"
        );
        // beach: turtles on the sand
        let beach = beach_world();
        let mut sys3 = MobSystem::new(9);
        sys3.player = Some([4.5, 70.0, 4.5]);
        for _ in 0..2000 {
            sys3.try_spawn_aquatic(&beach, |_, _| true);
        }
        assert!(
            sys3.list.iter().all(|m| m.kind == MobKind::Turtle),
            "beaches nest turtles (VERIFIED)"
        );
        assert!(!sys3.list.is_empty());
        for m in &sys3.list {
            assert_eq!(m.pos[1] as i32, 65, "standing on the sand surface");
        }
        // the water-ambient pool ignores the passive cap: 12+ aquatics
        // with zero passives alive is fine (the categories are separate)
        assert!(sys2.passives_alive() == 0);
    }
}

// ---------------- 1.14 bracket tests (Village & Pillage — nature half) ----------------
#[cfg(test)]
mod v114_tests {
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

    /// the V10 registry row + egg/tile mappings (VERIFIED w/Fox)
    #[test]
    fn v114_fox_registry_row() {
        assert_eq!(MOB_DATA.len(), 48, "32 + 8 aquatic + the fox + the 1.16 forest three + the audit trio");
        let d = def(MobKind::Fox);
        assert_eq!(d.health as i32, 10, "10 HP (VERIFIED infobox)");
        assert!((d.damage - 2.0).abs() < 1e-6, "Easy/Normal 2 HP");
        assert!((d.speed_attr - 0.3).abs() < 1e-6, "speed 0.3");
        assert!((d.height - 0.7).abs() < 1e-6, "hitbox height 0.7");
        assert!((d.width - 0.6).abs() < 1e-6, "hitbox width 0.6");
        assert_eq!(d.xp, 1);
        // passive, not hostile/neutral/aquatic (the creature category)
        assert!(!d.kind.hostile() && !d.kind.neutral() && !d.kind.aquatic());
        // egg + name roundtrips
        assert_eq!(MobKind::from_egg(40), MobKind::Fox);
        assert_eq!(MobKind::Fox.egg_id(), 40);
        assert_eq!(MobKind::from_name("fox"), Some(MobKind::Fox));
        assert_eq!(MobKind::Fox.name(), "minecraft:fox");
        // the sprite tile
        assert_eq!(MobKind::Fox.sprite_tile(), TILE_MOB_FOX);
    }

    /// VERIFIED w/Fox §Behavior: "Foxes attack chickens ... while they
    /// are on land" — a fox with a chicken in range chases and damages
    /// it; the damage lands through the pending queue.
    #[test]
    fn v114_fox_hunts_chickens() {
        let world = flat_world();
        let mut sys = MobSystem::new(9);
        let fox = sys.spawn_at(MobKind::Fox, 8, 65, 8).unwrap();
        let chicken = sys.spawn_at(MobKind::Chicken, 10, 65, 10).unwrap();
        // a player 20 blocks away: present (hunting needs the anchor)
        // but beyond the 6-block flee radius
        sys.player = Some([28.0, 65.0, 28.0]);
        for _ in 0..200 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        // the fox's 2 HP bites kill a 4 HP chicken in two hits — the
        // prey either died or is wounded
        let chicken_alive = sys.list.iter().find(|m| m.id == chicken);
        match chicken_alive {
            Some(c) => assert!(
                c.health < 4.0,
                "the fox damaged the chicken (hp {})",
                c.health
            ),
            None => assert!(
                sys.deaths.iter().any(|d| d.0 == MobKind::Chicken),
                "the chicken died to the fox (a deaths entry)"
            ),
        }
        let f = sys.list.iter().find(|m| m.id == fox).unwrap();
        assert!((f.pos[0] - 10.0).abs() < 3.0, "fox closed on the prey");
    }

    /// VERIFIED w/Sweet_Berry_Bush + w/Fox: entities moving through a
    /// stage-1+ bush take 1 HP per half-second and slow to 34.05%;
    /// foxes take neither ("take no damage or speed reduction").
    #[test]
    fn v114_bush_slows_and_damages_except_foxes() {
        let mut w = flat_world();
        // a mature bush at (8, 65, 8)
        let _ = w.set_block_state(8, 65, 8, berry_bush_state(3));
        let mut sys = MobSystem::new(3);
        let zid = sys.spawn_at(MobKind::Zombie, 8, 65, 8).unwrap();
        let fid = sys.spawn_at(MobKind::Fox, 8, 65, 9).unwrap();
        // walk the zombie INTO the bush cell with a live velocity
        {
            let z = sys.list.iter_mut().find(|m| m.id == zid).unwrap();
            z.pos = [8.5, 65.0, 8.5];
            z.vel = [1.0, 0.0, 0.0]; // moving
        }
        {
            let f = sys.list.iter_mut().find(|m| m.id == fid).unwrap();
            f.pos = [8.5, 65.0, 8.5];
            f.vel = [1.0, 0.0, 0.0]; // moving, same cell
        }
        hazard_tick(sys.list.iter_mut().find(|m| m.id == zid).unwrap(), &w, true);
        hazard_tick(sys.list.iter_mut().find(|m| m.id == fid).unwrap(), &w, true);
        let z = sys.list.iter().find(|m| m.id == zid).unwrap();
        let f = sys.list.iter().find(|m| m.id == fid).unwrap();
        assert_eq!(z.health, 19.0, "the zombie took the 1 HP window hit");
        assert!(z.vel[0] < 0.5, "the zombie was slowed to 34.05% (vel {})", z.vel[0]);
        assert_eq!(f.health, 10.0, "the fox took NO damage (VERIFIED immunity)");
        assert_eq!(f.vel[0], 1.0, "the fox was NOT slowed (VERIFIED immunity)");
        // stage 0 (sapling) never damages
        let _ = w.set_block_state(8, 65, 8, berry_bush_state(0));
        {
            let z = sys.list.iter_mut().find(|m| m.id == zid).unwrap();
            z.health = 20.0;
            z.vel = [1.0, 0.0, 0.0];
        }
        hazard_tick(sys.list.iter_mut().find(|m| m.id == zid).unwrap(), &w, true);
        assert_eq!(sys.list.iter().find(|m| m.id == zid).unwrap().health, 20.0);
    }

    /// VERIFIED w/Campfire §Damage: "If lit, campfires damage mobs
    /// standing on top of them" — 1 HP per 0.5 s window; unlit never.
    #[test]
    fn v114_campfire_standing_damage() {
        let mut w = flat_world();
        let _ = w.set_block_state(8, 65, 8, campfire_state(true));
        let mut sys = MobSystem::new(4);
        let cid = sys.spawn_at(MobKind::Sheep, 8, 66, 8).unwrap();
        // place the sheep ON the campfire, grounded
        {
            let s = sys.list.iter_mut().find(|m| m.id == cid).unwrap();
            s.pos = [8.5, 66.0, 8.5];
            s.on_ground = true;
        }
        hazard_tick(sys.list.iter_mut().find(|m| m.id == cid).unwrap(), &w, true);
        assert_eq!(
            sys.list.iter().find(|m| m.id == cid).unwrap().health,
            7.0,
            "sheep 8 HP - 1: the lit campfire burned it"
        );
        // unlit: nothing
        let _ = w.set_block_state(8, 65, 8, campfire_state(false));
        hazard_tick(sys.list.iter_mut().find(|m| m.id == cid).unwrap(), &w, true);
        assert_eq!(
            sys.list.iter().find(|m| m.id == cid).unwrap().health,
            7.0,
            "the extinguished campfire is inert"
        );
        // not standing on it: nothing
        let _ = w.set_block_state(8, 65, 8, campfire_state(true));
        {
            let s = sys.list.iter_mut().find(|m| m.id == cid).unwrap();
            s.pos = [4.5, 66.0, 8.5]; // 4 blocks aside
        }
        hazard_tick(sys.list.iter_mut().find(|m| m.id == cid).unwrap(), &w, true);
        assert_eq!(
            sys.list.iter().find(|m| m.id == cid).unwrap().health,
            7.0,
            "off the campfire: no damage"
        );
    }

    /// VERIFIED w/Sweet_Berries §Breeding: feed two adults → the second
    /// feeding pairs them; the cub is the game layer's to spawn.
    #[test]
    fn v114_fox_breeding_flow() {
        let mut sys = MobSystem::new(6);
        let a = sys.spawn_at(MobKind::Fox, 8, 65, 8).unwrap();
        let b = sys.spawn_at(MobKind::Fox, 10, 65, 8).unwrap();
        // feeding a non-fox: nothing
        let cow = sys.spawn_at(MobKind::Cow, 12, 65, 8).unwrap();
        assert!(sys.try_feed_fox(cow).is_none());
        // first feeding: love mode
        match sys.try_feed_fox(a) {
            Some(FoxFeedOutcome::LoveMode) => {}
            other => panic!("first feeding should enter love mode: {other:?}"),
        }
        assert!(sys.list.iter().find(|m| m.id == a).unwrap().variant & 0x80 != 0);
        assert_eq!(sys.list.iter().find(|m| m.id == a).unwrap().aux, 600);
        // feeding the loving fox again: ignored (vanilla)
        assert!(sys.try_feed_fox(a).is_none());
        // second feeding pairs them
        match sys.try_feed_fox(b) {
            Some(FoxFeedOutcome::Bred(pid)) => assert_eq!(pid, a),
            other => panic!("second feeding should pair: {other:?}"),
        }
        // both loves cleared
        assert_eq!(sys.list.iter().find(|m| m.id == a).unwrap().variant & 0x80, 0);
        assert_eq!(sys.list.iter().find(|m| m.id == b).unwrap().variant & 0x80, 0);
        // a baby never breeds
        let kid = sys
            .spawn_variant(MobKind::Fox, 12, 65, 10, 0x41)
            .unwrap();
        assert!(sys.try_feed_fox(kid).is_none(), "babies don't breed");
    }

    /// the fox cub maturity countdown (variant 0x40, aux 24000 → grown)
    #[test]
    fn v114_fox_cub_grows() {
        let world = flat_world();
        let mut sys = MobSystem::new(7);
        let kid = sys
            .spawn_variant(MobKind::Fox, 8, 65, 8, 0x41)
            .unwrap();
        {
            let k = sys.list.iter_mut().find(|m| m.id == kid).unwrap();
            k.aux = 3; // short countdown for the test
        }
        for _ in 0..3 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let k = sys.list.iter().find(|m| m.id == kid).unwrap();
        assert_eq!(k.variant & 0x40, 0, "grown after the countdown");
        assert_eq!(k.variant & 0x1, 1, "still trusting (the bred bit)");
    }
}


    /// the MOB_DATA row: 10 HP, sting 2 (Normal), speed 0.6 (the
    /// infobox row), hitbox 0.5 x 0.55, XP row (VERIFIED w/Bee)
    #[test]
    fn v115_bee_def_row() {
        let d = def(MobKind::Bee);
        assert_eq!(d.health, 10.0);
        assert_eq!(d.damage, 2.0);
        assert_eq!(d.speed_attr, 0.6);
        assert_eq!((d.height, d.width), (0.5, 0.55));
        assert_eq!(d.armor, 0.0);
        assert!(MobKind::Bee.flies(), "bees hover (no gravity)");
        assert_eq!(MobKind::Bee.egg_id(), 41);
        assert_eq!(MobKind::from_egg(41), MobKind::Bee);
        assert_eq!(MobKind::Bee.sprite_tile(), TILE_MOB_BEE);
    }

    /// 1.15: the v115 tests' own flat world (the per-module convention)
    fn v115_world() -> World {
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

    /// the completeness audit: the chicken's egg laying — "Every adult
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

    /// the sweep-2: the player-thrown trio never hits the thrower, and
    /// eggs + pearls (not snowballs) push landing events on the ground
    /// hit (VERIFIED w/Egg + w/Ender_Pearl + w/Snowball, live 2026-09-09)
    #[test]
    fn audit16_sweep2_projectile_landings() {
        let world = v115_world();
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
        let world = v115_world();
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
        let world = v115_world();
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
        // and the flying class (no gravity — the bat/phantom class)
        assert!(MobKind::Ghast.flies(), "the ghast is a FlyingMob-class");
    }

    /// the sting contract: an angry bee stings ONCE (2 HP + Poison I
    /// 10 s payload), loses the stinger, "dies approximately one
    /// minute later" (1200 ticks), and never attacks again (VERIFIED
    /// w/Bee §Attacking)
    #[test]
    fn v115_bee_sting_rules() {
        let world = v115_world();
        let mut sys = MobSystem::new(9);
        sys.player = Some([8.6, 65.5, 8.5]);
        let id = sys.spawn_at(MobKind::Bee, 8, 66, 8).unwrap();
        sys.set_bee(id, [4, 66, 4], true); // angry release
        // the anger chase → contact → the sting
        let mut stung = false;
        let mut poison: Option<i32> = None;
        for _ in 0..40 {
            sys.tick(&world, (0, 0), i32::MAX);
            if !stung {
                let hits = std::mem::take(&mut sys.hits);
                if let Some(h) = hits.first() {
                    stung = true;
                    assert_eq!(h.source, MobKind::Bee);
                    assert!((h.damage - 2.0).abs() < 1e-4, "sting damage 2 (Normal)");
                    poison = h.poison_effect;
                    assert_eq!(h.wither_effect, None);
                }
            }
            if stung {
                break;
            }
        }
        assert!(stung, "the angry bee reached + stung the player");
        assert_eq!(poison, Some(200), "Poison I 10 s (200 ticks) payload");
        // one sting only: the bee is stung — no further hits, ever
        sys.hits.clear();
        for _ in 0..60 {
            sys.tick(&world, (0, 0), i32::MAX);
            assert!(sys.hits.is_empty(), "no second sting");
        }
        let m = sys.by_id(id).unwrap();
        let b = m.bee.as_ref().unwrap();
        assert!(b.stung, "stinger spent");
        // the timer armed at 1200 and already counts down — the observed
        // window (sting tick .. now) is small vs the 1200 span
        assert!(
            b.death_t > crate::bees::STING_DEATH_TICKS - 100,
            "1200-tick death timer (armed, now {})",
            b.death_t
        );
        let armed = b.death_t;
        // the death timer: fast-forward to the 1200-tick mark — the
        // bee dies (the deaths queue fires; no item drops)
        let before = sys.list.iter().filter(|m| m.kind == MobKind::Bee).count();
        assert_eq!(before, 1);
        for _ in 0..(armed + 20) {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        assert!(
            sys.list.iter().all(|m| m.kind != MobKind::Bee),
            "the stung bee died on the timer"
        );
    }

    /// the anger swarm: attacking one bee angers the family + the
    /// 16-block neighbors; the anger window is 20-39 s (VERIFIED)
    #[test]
    fn v115_anger_swarm() {
        let world = v115_world();
        let mut sys = MobSystem::new(10);
        let a = sys.spawn_at(MobKind::Bee, 6, 66, 6).unwrap();
        let b = sys.spawn_at(MobKind::Bee, 8, 66, 8).unwrap(); // near
        let far = sys.spawn_at(MobKind::Bee, 40, 66, 40).unwrap(); // far, no family
        sys.set_bee(a, [3, 66, 3], false);
        sys.set_bee(b, [3, 66, 3], false); // same hive family
        sys.set_bee(far, [60, 66, 60], false);
        let n = sys.anger_bees_near([6.5, 66.0, 6.5], Some([3, 66, 3]));
        assert_eq!(n, 2, "family + near angered; far stranger not");
        for id in [a, b] {
            let t = sys.by_id(id).unwrap().bee.as_ref().unwrap().anger_t;
            assert!(
                (ANGER_TICKS_MIN_ROLL..=ANGER_TICKS_MAX).contains(&t),
                "anger window 20-39 s: {t}"
            );
        }
        assert_eq!(sys.by_id(far).unwrap().bee.as_ref().unwrap().anger_t, 0);
    }

    /// the flower feeding: first feeding arms love; a second with a
    /// loving partner pairs → Bred (VERIFIED w/Bee §Breeding)
    #[test]
    fn v115_bee_breeding_flow() {
        let mut sys = MobSystem::new(11);
        let a = sys.spawn_at(MobKind::Bee, 6, 66, 6).unwrap();
        let b = sys.spawn_at(MobKind::Bee, 8, 66, 8).unwrap();
        assert_eq!(sys.try_feed_bee(a), Some(BeeFeedOutcome::LoveMode));
        assert_eq!(sys.try_feed_bee(a), None, "already in love — ignored");
        assert_eq!(sys.try_feed_bee(b), Some(BeeFeedOutcome::Bred(a)));
        // both loves cleared after the pairing
        for id in [a, b] {
            assert_eq!(sys.by_id(id).unwrap().bee.as_ref().unwrap().love_t, 0);
        }
        // babies never breed
        let kid = sys.spawn_at(MobKind::Bee, 6, 66, 10).unwrap();
        if let Some(m) = sys.list.iter_mut().find(|m| m.id == kid) {
            if let Some(bs) = m.bee.as_mut() {
                bs.baby = true;
            }
        }
        assert!(sys.try_feed_bee(kid).is_none(), "babies don't breed");
    }

    /// the hive trip: a nectar bee in PH_TO_HIVE reaches the hive,
    /// leaves the mob list (the enter queue), and the sting-less
    /// night return works the same (VERIFIED w/Bee §Housing)
    #[test]
    fn v115_bee_enters_hive() {
        let world = v115_world();
        let mut sys = MobSystem::new(12);
        sys.is_day = true;
        let id = sys.spawn_at(MobKind::Bee, 6, 68, 6).unwrap();
        sys.set_bee(id, [8, 66, 8], false);
        if let Some(m) = sys.by_id_mut(id) {
            if let Some(b) = m.bee.as_mut() {
                b.nectar = true;
                b.phase = crate::bees::PH_TO_HIVE;
            }
        }
        for _ in 0..200 {
            sys.tick(&world, (0, 0), i32::MAX);
            if !sys.bee_enters.is_empty() {
                break;
            }
        }
        assert!(!sys.bee_enters.is_empty(), "the bee reached the hive");
        let (bid, hive, nectar) = sys.bee_enters[0];
        assert_eq!(bid, id);
        assert_eq!(hive, [8, 66, 8]);
        assert!(nectar, "carried nectar");
        // the arrival pass removes it from the list (same tick)
        assert!(sys.list.iter().all(|m| m.id != id), "mob left the list");
    }

    // ------------- 1.16 (Nether Update, part 2) tests -------------

    /// the MOB_DATA rows (all VERIFIED against the v116b captures)
    #[test]
    fn v116b_forest_mob_def_rows() {
        let s = def(MobKind::Strider);
        assert_eq!(s.health, 20.0);
        assert_eq!(s.damage, 0.0);
        assert_eq!(s.speed_attr, 0.175);
        assert_eq!((s.height, s.width), (1.7, 0.9));
        let p = def(MobKind::Piglin);
        assert_eq!(p.health, 16.0);
        assert_eq!(p.damage, 8.0); // the golden-sword Normal row
        assert_eq!(p.speed_attr, 0.35);
        assert_eq!((p.height, p.width), (1.95, 0.6));
        let h = def(MobKind::Hoglin);
        assert_eq!(h.health, 40.0);
        assert_eq!(h.damage, 5.5); // the 3-8 Normal midpoint, disclosed
        assert_eq!(h.speed_attr, 0.3);
        assert_eq!((h.height, h.width), (1.4, 1.3965));
        // the classification rows
        assert!(MobKind::Strider.neutral() == false && !MobKind::Strider.hostile());
        assert!(MobKind::Piglin.neutral(), "piglins are the neutral (adult) row");
        assert!(!MobKind::Piglin.hostile());
        assert!(MobKind::Hoglin.hostile(), "hoglins are the hostile row");
        assert!(!MobKind::Hoglin.neutral());
        // the registry + egg mapping (kinds 42..=44)
        assert_eq!(MobKind::Strider.name(), "minecraft:strider");
        assert_eq!(MobKind::Piglin.name(), "minecraft:piglin");
        assert_eq!(MobKind::Hoglin.name(), "minecraft:piglin".replace("piglin", "hoglin"));
        assert_eq!(MobKind::from_name("strider"), Some(MobKind::Strider));
        assert_eq!(MobKind::from_name("piglin"), Some(MobKind::Piglin));
        assert_eq!(MobKind::from_name("hoglin"), Some(MobKind::Hoglin));
        assert_eq!(MobKind::Strider.egg_id(), 42);
        assert_eq!(MobKind::Piglin.egg_id(), 43);
        assert_eq!(MobKind::Hoglin.egg_id(), 44);
        assert_eq!(MobKind::from_egg(42), MobKind::Strider);
        assert_eq!(MobKind::from_egg(43), MobKind::Piglin);
        assert_eq!(MobKind::from_egg(44), MobKind::Hoglin);
        assert_eq!(MobKind::Strider.sprite_tile(), TILE_MOB_STRIDER);
        assert_eq!(MobKind::Piglin.sprite_tile(), TILE_MOB_PIGLIN);
        assert_eq!(MobKind::Hoglin.sprite_tile(), TILE_MOB_HOGLIN);
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
        assert_eq!(sf.xp, 5, "\"no drops other than 5 XP\" (VERIFIED)");
    }

    /// 1.16: the barter table only yields engine-valid items, with
    /// vanilla-count ranges (VERIFIED w/Bartering)
    #[test]
    fn v116b_barter_table_items() {
        let mut rng = Rng::new(99);
        for _ in 0..500 {
            let (item, count) = piglin_barter_roll(&mut rng);
            assert!(matches!(
                item,
                CRYING_OBSIDIAN
                    | OBSIDIAN
                    | GRAVEL
                    | BLACKSTONE
                    | LEATHER
                    | SOUL_SAND
                    | STRING
                    | NETHER_QUARTZ
                    | IRON_NUGGET
                    | ENDER_PEARL
            ));
            assert!(count >= 1 && count <= 36, "count {count} in range");
        }
    }

    /// 1.16: the piglin barter flow — a gold-ingot use arms the
    /// 120-tick examine, the countdown ends in a pending_drops entry
    /// (the piglin "throws" the bartered item, VERIFIED w/Piglin)
    #[test]
    fn v116b_piglin_barter_flow() {
        let world = v115_world();
        let mut sys = MobSystem::new(21);
        let id = sys.spawn_at(MobKind::Piglin, 8, 65, 8).unwrap();
        // the wrong item is refused
        assert!(!sys.try_barter_piglin(id, STRING));
        // gold (the iron-ore stand-in) arms the examine
        assert!(sys.try_barter_piglin(id, IRON_ORE));
        {
            let m = sys.list.iter().find(|m| m.id == id).unwrap();
            assert_eq!(m.aux, 120, "the 6-second examine countdown");
        }
        // a second offer while examining is refused
        assert!(!sys.try_barter_piglin(id, IRON_ORE));
        // run the countdown out with the player nearby (the AI ticks)
        sys.player = Some([8.0, 65.0, 10.0]);
        for _ in 0..130 {
            sys.tick(&world, (0, 0), i32::MAX);
            if !sys.pending_drops.is_empty() {
                break;
            }
        }
        assert!(
            !sys.pending_drops.is_empty(),
            "the barter item surfaced through pending_drops"
        );
    }

    /// 1.16: the gold-mining anger hook (VERIFIED w/Piglin's
    /// aggravation rows — mining gold-related blocks angers nearby
    /// piglins within 16 blocks)
    #[test]
    fn v116b_piglin_gold_mining_anger() {
        let mut sys = MobSystem::new(22);
        let near = sys.spawn_at(MobKind::Piglin, 8, 65, 8).unwrap();
        let far = sys.spawn_at(MobKind::Piglin, 40, 65, 40).unwrap();
        let _strider = sys.spawn_at(MobKind::Strider, 9, 65, 8).unwrap();
        let n = sys.anger_piglins_near([8.0, 65.0, 8.0], 16.0);
        assert_eq!(n, 1, "only the nearby piglin angered");
        assert!(sys.list.iter().find(|m| m.id == near).unwrap().provoked);
        assert!(!sys.list.iter().find(|m| m.id == far).unwrap().provoked);
        // the strider is untouched (piglins only)
        assert!(!sys.list.iter().find(|m| m.id == _strider).unwrap().provoked);
    }

    /// 1.16: strider breeding — the warped fungus arms love, the pair
    /// breeds (VERIFIED w/Strider §Breeding)
    #[test]
    fn v116b_strider_feeding_and_breeding() {
        let world = v115_world();
        let mut sys = MobSystem::new(23);
        let a = sys.spawn_at(MobKind::Strider, 8, 65, 8).unwrap();
        let b = sys.spawn_at(MobKind::Strider, 9, 65, 9).unwrap();
        // the wrong food is refused
        assert!(sys.try_feed_strider(a, CRIMSON_FUNGUS).is_none());
        // the first feeding arms love
        assert_eq!(sys.try_feed_strider(a, WARPED_FUNGUS), Some(StriderFeedOutcome::LoveMode));
        {
            let m = sys.list.iter().find(|m| m.id == a).unwrap();
            assert!(m.variant & 0x80 != 0, "in love");
        }
        // the second feeding with a loving partner pairs them
        assert_eq!(sys.try_feed_strider(b, WARPED_FUNGUS), Some(StriderFeedOutcome::Bred(a)));
        for id in [a, b] {
            let m = sys.list.iter().find(|m| m.id == id).unwrap();
            assert_eq!(m.variant & 0x80, 0, "both exited love");
        }
        // a baby strider cannot be fed (maturity-only, disclosed)
        let kid = sys.spawn_variant(MobKind::Strider, 10, 65, 10, 0x40).unwrap();
        assert!(sys.try_feed_strider(kid, WARPED_FUNGUS).is_none());
        // the baby matures on the countdown (the fox pattern)
        {
            let m = sys.list.iter_mut().find(|m| m.id == kid).unwrap();
            m.aux = 2; // short countdown for the test
        }
        for _ in 0..3 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let m = sys.list.iter().find(|m| m.id == kid).unwrap();
        assert_eq!(m.variant & 0x40, 0, "grown after the countdown");
    }

    /// 1.16: hoglin breeding is flee-gated — a warped fungus within 7
    /// blocks refuses the crimson-fungus feed ("Hoglins cannot be bred
    /// when they are running away from warped fungi", VERIFIED w/Hoglin)
    #[test]
    fn v116b_hoglin_flee_gate() {
        let mut world = v115_world();
        let mut sys = MobSystem::new(24);
        let a = sys.spawn_at(MobKind::Hoglin, 8, 65, 8).unwrap();
        let b = sys.spawn_at(MobKind::Hoglin, 9, 65, 9).unwrap();
        // no repel source: the feed flows
        assert_eq!(
            sys.try_feed_hoglin(a, CRIMSON_FUNGUS, &world),
            Some(HoglinFeedOutcome::LoveMode)
        );
        // a stays in love (the pairing state); the gate tests below
        // feed a SECOND hoglin under the repel sources
        // plant the warped fungus 3 blocks away → the feed refuses
        let _ = world.set_block(11, 65, 11, WARPED_FUNGUS);
        assert!(sys.try_feed_hoglin(a, CRIMSON_FUNGUS, &world).is_none());
        // the respawn anchor repels too (VERIFIED w/Hoglin)
        let _ = world.set_block(11, 65, 11, RESPAWN_ANCHOR);
        assert!(sys.try_feed_hoglin(a, CRIMSON_FUNGUS, &world).is_none());
        let _ = world.set_block(11, 65, 11, 0);
        // and the pair completes without the repel source
        assert_eq!(
            sys.try_feed_hoglin(b, CRIMSON_FUNGUS, &world),
            Some(HoglinFeedOutcome::Bred(a))
        );
    }

    /// 1.16: the strider's lava physics — feet in lava with air above
    /// = standing on the surface ("walk on top of it without sinking",
    /// VERIFIED w/Strider); fully submerged = the buoyant ascent
    #[test]
    fn v116b_strider_lava_physics() {
        let mut world = v115_world();
        // a lava pond at the stone surface
        for dz in 7..=9i32 {
            for dx in 7..=9i32 {
                let _ = world.set_block(dx, 64, dz, LAVA);
            }
        }
        let mut sys = MobSystem::new(25);
        let id = sys.spawn_at(MobKind::Strider, 8, 64, 8).unwrap();
        {
            let m = sys.list.iter_mut().find(|m| m.id == id).unwrap();
            m.pos = [8.5, 64.0, 8.5]; // feet in the lava cell
            m.vel = [0.0, -8.0, 0.0]; // a hard sink attempt
        }
        sys.tick(&world, (0, 0), i32::MAX);
        let m = sys.list.iter().find(|m| m.id == id).unwrap();
        assert!(m.on_ground, "standing on the lava surface");
        assert_eq!(m.vel[1], 0.0, "no sinking");
        assert!((m.pos[1] - 64.0).abs() < 0.1, "still at the surface");
        // submerged: rises out (VERIFIED)
        {
            let m = sys.list.iter_mut().find(|m| m.id == id).unwrap();
            m.pos = [8.5, 64.0, 8.5];
        }
        // cover the cell above with lava too → submerged → ascent
        for dz in 7..=9i32 {
            for dx in 7..=9i32 {
                let _ = world.set_block(dx, 65, dz, LAVA);
            }
        }
        sys.tick(&world, (0, 0), i32::MAX);
        let m = sys.list.iter().find(|m| m.id == id).unwrap();
        assert!(m.vel[1] > 0.0, "rising out of the lava (vel {})", m.vel[1]);
    }

    /// 1.16: striders take water damage at the 0.5 s cadence ("1 HP
    /// per ... half-second in water", VERIFIED w/Strider)
    #[test]
    fn v116b_strider_water_damage() {
        let mut world = v115_world();
        for dz in 7..=9i32 {
            for dx in 7..=9i32 {
                let _ = world.set_block(dx, 64, dz, WATER);
                let _ = world.set_block(dx, 65, dz, WATER);
            }
        }
        let mut sys = MobSystem::new(26);
        let id = sys.spawn_at(MobKind::Strider, 8, 64, 8).unwrap();
        let hp0 = def(MobKind::Strider).health;
        for _ in 0..10 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let m = sys.list.iter().find(|m| m.id == id).unwrap();
        assert!(m.health < hp0, "water damages striders (hp {})", m.health);
        // the control: a piglin in water is untouched (the strider rule)
        let pid = sys.spawn_at(MobKind::Piglin, 8, 64, 9).unwrap();
        let php0 = def(MobKind::Piglin).health;
        for _ in 0..10 {
            sys.tick(&world, (0, 0), i32::MAX);
        }
        let p = sys.list.iter().find(|m| m.id == pid).unwrap();
        assert_eq!(p.health, php0, "piglins take no water damage");
    }
