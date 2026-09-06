//! Villagers (Phase 7 §27/§29, expanded Phase 5): village NPCs —
//! deterministic auto-spawn at village wells, wander-around-home AI with
//! vanilla-observable movement (0.5 blocks/s walk, 1-block jump-ups), and
//! the per-profession tiered trade tables the trade screen serves.
//!
//! Phase 5 trading depth (every value VERIFIED against minecraft.wiki,
//! live pull Sep 2026 — Villager + Trading pages, including the 1.16.5-era
//! Trading revision 1952066):
//! - the full **15 villager types** (13 professions + Unemployed + Nitwit),
//!   registry ids in the `minecraft:villager_profession` spelling
//! - **5 career levels** (Novice..Master) with XP thresholds
//!   0 / 10 / 70 / 150 / 250 (Villager "Experience levels" table)
//! - per-trade stock: **16 uses at Novice, 12 at every higher tier** (the
//!   1.16.5-era Trading tables' "Trades until disabled" columns)
//! - villager XP per trade by tier: 2 / 5 / 10 / 15 / 30 (the tables'
//!   "XP to villager" columns, top-of-range pattern)
//! - restock: offers re-activate **up to twice per day** while the villager
//!   works (Trading: "When villagers work at their job site blocks, they
//!   activate their offers again, up to twice per day")
//! - Nitwit/Unemployed: no trade offers
//!
//! Documented adaptations (palette-bounded):
//! - emerald item → EMERALD_ORE block (no standalone emerald item)
//! - vanilla job-site blocks (composter, lectern, blast furnace...) are
//!   mostly outside the palette: professions are assigned at village
//!   populate time instead of claimed from job blocks, and restock ties
//!   to the village home (the well) rather than a workstation. The 16/12
//!   uses, 2x/day cadence and XP economics are the verified real values.
//! - villager entity state (XP/level/stock) lives in memory: it resets on
//!   world reload (vanilla persists per-entity NBT — logged open tail)

use std::collections::HashSet;
use vc_blocks::blocks::*;
use vc_rng::rng::Rng;

pub const MAX_VILLAGERS: usize = 96;
/// vanilla villager walk speed (blocks/s)
pub const WALK_SPEED: f32 = 0.5;
/// vanilla jump height ≈ 1.25 blocks → our 1-block step-up velocity
pub const JUMP_VEL: f32 = 0.42;
/// max trades one profession's table holds (rows × tiers bound)
pub const MAX_TRADES: usize = 16;

/// career levels 1..=5 (VERIFIED names + XP thresholds on the wiki)
pub const LEVEL_NAMES: [&str; 5] = ["Novice", "Apprentice", "Journeyman", "Expert", "Master"];
/// cumulative villager XP required per career level (VERIFIED:
/// Novice 0, Apprentice 10, Journeyman 70, Expert 150, Master 250)
pub const LEVEL_XP: [u32; 5] = [0, 10, 70, 150, 250];
/// per-tier trade stock (VERIFIED: 16 at Novice, 12 above)
pub const TIER_USES: [u16; 5] = [16, 12, 12, 12, 12];
/// villager XP granted by a trade of each tier (VERIFIED pattern:
/// 1-2 / ~5 / ~10 / ~15 / 30 — we use the canonical column values)
pub const TIER_XP: [u16; 5] = [2, 5, 10, 15, 30];
/// sim ticks between restock opportunities (12000 ticks/day ÷ 2 — the
/// VERIFIED twice-per-day cadence)
pub const RESTOCK_TICKS: u64 = 6000;

// ------------------------------------------------ gossip (§Gossiping) --
// VERIFIED against minecraft.wiki/w/Villager §Gossiping (live round,
// research-verdicts.md): per-type gain / decay / sharing cost / maximum /
// reputation multiplier. Decay runs every 20 minutes (24000 ticks);
// shared gossip arrives reduced by the sharing cost; major_positive
// can never be shared (cost 100 > max 20); reputation = Σ value ×
// multiplier; trade prices shift by −floor(reputation × 0.05).
/// gossip decay / share cadence: 20 real minutes = 24000 game ticks
pub const GOSSIP_DECAY_TICKS: u64 = 24_000;
/// vanilla standard price multiplier for common trades (the reputation
/// discount scales with it — minecraft.wiki/w/Trading §Sale prices)
pub const PRICE_MULTIPLIER: f32 = 0.05;

/// The five gossip types (VERIFIED table). `major_positive`/`minor_positive`
/// gain on curing zombie villagers — this engine has no zombie-villager
/// curing, so those gain paths are unreachable today; the rows are kept
/// (decay + share + reputation stay type-complete) for the future system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GossipKind {
    Trading,
    MajorPositive,
    MinorPositive,
    MinorNegative,
    MajorNegative,
}

/// Per-type constants: (gain, decay, sharing cost, maximum, multiplier)
const GOSSIP_TABLE: [(u16, u16, u16, u16, i32); 5] = [
    (4, 2, 20, 25, 1),     // Trading
    (20, 0, 100, 20, 5),   // MajorPositive
    (25, 1, 5, 25, 1),     // MinorPositive
    (25, 20, 20, 200, -1), // MinorNegative
    (25, 10, 10, 100, -5), // MajorNegative
];

impl GossipKind {
    pub fn gain(self) -> u16 {
        GOSSIP_TABLE[self as usize].0
    }
    pub fn decay(self) -> u16 {
        GOSSIP_TABLE[self as usize].1
    }
    pub fn share_cost(self) -> u16 {
        GOSSIP_TABLE[self as usize].2
    }
    pub fn max(self) -> u16 {
        GOSSIP_TABLE[self as usize].3
    }
    pub fn multiplier(self) -> i32 {
        GOSSIP_TABLE[self as usize].4
    }
    /// shareable: the wiki note — a type whose sharing cost exceeds its
    /// maximum can never be shared (major_positive)
    pub fn shareable(self) -> bool {
        self.share_cost() < self.max()
    }
}

/// One villager's gossip state with the (single) player. Values are
/// clamped to each type's maximum; the wiki's "line of sight" kill-share
/// box is approximated by the 16-block cube (documented simplification).
#[derive(Clone, Copy, Default, Debug)]
pub struct Gossip {
    pub trading: u16,
    pub major_positive: u16,
    pub minor_positive: u16,
    pub minor_negative: u16,
    pub major_negative: u16,
}

impl Gossip {
    pub fn get(&self, k: GossipKind) -> u16 {
        match k {
            GossipKind::Trading => self.trading,
            GossipKind::MajorPositive => self.major_positive,
            GossipKind::MinorPositive => self.minor_positive,
            GossipKind::MinorNegative => self.minor_negative,
            GossipKind::MajorNegative => self.major_negative,
        }
    }
    pub fn set(&mut self, k: GossipKind, v: u16) {
        let v = v.min(k.max());
        match k {
            GossipKind::Trading => self.trading = v,
            GossipKind::MajorPositive => self.major_positive = v,
            GossipKind::MinorPositive => self.minor_positive = v,
            GossipKind::MinorNegative => self.minor_negative = v,
            GossipKind::MajorNegative => self.major_negative = v,
        }
    }
    /// value += gain, clamped at the type maximum
    pub fn gain_event(&mut self, k: GossipKind) {
        let v = self.get(k).saturating_add(k.gain()).min(k.max());
        self.set(k, v);
    }
    /// receive shared gossip: value − sharing cost, clamped at the max
    pub fn receive_share(&mut self, k: GossipKind, shared: u16) {
        let v = self
            .get(k)
            .saturating_add(shared.saturating_sub(k.share_cost()))
            .min(k.max());
        self.set(k, v);
    }
    /// periodic decay (every 20 min): value -= decay, floor at 0
    pub fn decay_pass(&mut self) {
        for k in [
            GossipKind::Trading,
            GossipKind::MajorPositive,
            GossipKind::MinorPositive,
            GossipKind::MinorNegative,
            GossipKind::MajorNegative,
        ] {
            let v = self.get(k).saturating_sub(k.decay());
            self.set(k, v);
        }
    }
    /// reputation = Σ value × multiplier (VERIFIED)
    pub fn reputation(&self) -> i32 {
        [
            GossipKind::Trading,
            GossipKind::MajorPositive,
            GossipKind::MinorPositive,
            GossipKind::MinorNegative,
            GossipKind::MajorNegative,
        ]
        .iter()
        .map(|&k| self.get(k) as i32 * k.multiplier())
        .sum()
    }
}

/// the 15 villager types: 13 professions + Unemployed + Nitwit
/// (display names; registry spellings in PROFESSION_IDS)
pub const PROFESSIONS: [&str; 15] = [
    "Armorer",
    "Butcher",
    "Cartographer",
    "Cleric",
    "Farmer",
    "Fisherman",
    "Fletcher",
    "Leatherworker",
    "Librarian",
    "Mason",
    "Nitwit",
    "Shepherd",
    "Toolsmith",
    "Unemployed",
    "Weaponsmith",
];

/// registry ids exactly as `minecraft:villager_profession` spells them
/// (the mechanical-name discipline: exact key strings, no renaming)
pub const PROFESSION_IDS: [&str; 15] = [
    "minecraft:armorer",
    "minecraft:butcher",
    "minecraft:cartographer",
    "minecraft:cleric",
    "minecraft:farmer",
    "minecraft:fisherman",
    "minecraft:fletcher",
    "minecraft:leatherworker",
    "minecraft:librarian",
    "minecraft:mason",
    "minecraft:nitwit",
    "minecraft:shepherd",
    "minecraft:toolsmith",
    "minecraft:unemployed",
    "minecraft:weaponsmith",
];

/// the vanilla job-site block of each profession (documentation/UI only —
/// palette-bounded adaptation: professions assign at village populate
/// instead of job-block claiming, see the module header)
pub const JOB_SITES: [&str; 15] = [
    "Blast Furnace",
    "Smoker",
    "Cartography Table",
    "Brewing Stand",
    "Composter",
    "Barrel",
    "Fletching Table",
    "Cauldron",
    "Lectern",
    "Stonecutter",
    "(none)",
    "Loom",
    "Smithing Table",
    "(none)",
    "Grindstone",
];

/// profession index of the two trade-less types
pub const NITWIT: u8 = 10;
pub const UNEMPLOYED: u8 = 13;

/// career level of a villager from its XP (1..=5)
pub fn level_for_xp(xp: u32) -> u8 {
    let mut lvl = 1u8;
    for (i, t) in LEVEL_XP.iter().enumerate() {
        if xp >= *t {
            lvl = (i + 1) as u8;
        }
    }
    lvl.min(5)
}

/// display name of a career level
pub fn level_name(level: u8) -> &'static str {
    LEVEL_NAMES[(level.clamp(1, 5) - 1) as usize]
}

/// one trade row: pay (block, count) → receive (block, count), gated at
/// career `tier` (1..=5), with per-restock stock and villager-XP value
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    pub give: (u16, u8),
    pub get: (u16, u8),
    /// career tier that unlocks this row (1 = Novice .. 5 = Master)
    pub tier: u8,
    /// stock per restock cycle (16 tier-1 / 12 above, VERIFIED)
    pub max_uses: u16,
    /// villager XP this trade grants (VERIFIED column values)
    pub xp: u16,
}

const fn tr(give: (u16, u8), get: (u16, u8), tier: u8) -> Trade {
    Trade {
        give,
        get,
        tier,
        max_uses: TIER_USES[(tier - 1) as usize],
        xp: TIER_XP[(tier - 1) as usize],
    }
}

/// per-profession trade tables — palette-bounded adaptations of the
/// vanilla per-tier shape (same tier gating, stock, and XP economics;
/// items remapped to this engine's palette, emerald = EMERALD_ORE).
/// Each profession: 2 rows × 5 tiers.
pub fn trades(profession: u8) -> &'static [Trade] {
    // (Armorer 0) metal trader: buys ores, sells metal blocks
    const ARMORER: &[Trade] = &[
        tr((IRON_ORE, 5), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 4), (IRON_BLOCK, 1), 1),
        tr((COAL, 10), (EMERALD_ORE, 1), 2), // fix #4: the real coal item
        tr((EMERALD_ORE, 5), (GOLD_BLOCK, 1), 2),
        tr((LAPIS_ORE, 8), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 9), (DIAMOND_BLOCK, 1), 3),
        tr((DIAMOND_ORE, 3), (EMERALD_ORE, 2), 4),
        tr((EMERALD_ORE, 12), (OBSIDIAN, 4), 4),
        tr((REDSTONE_ORE, 10), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 3), (GLOWSTONE, 2), 5),
    ];
    // (Butcher 1) meat trader
    const BUTCHER: &[Trade] = &[
        tr((CHICKEN_RAW, 7), (EMERALD_ORE, 1), 1),
        tr((MUTTON, 7), (EMERALD_ORE, 1), 1),
        tr((BEEF, 5), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (MUTTON, 5), 2),
        tr((ROTTEN_FLESH, 16), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 2), (BEEF, 5), 3),
        tr((PORKCHOP, 5), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 3), (PORKCHOP, 5), 4),
        tr((MUSHROOM_BROWN, 10), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 2), (CHICKEN_RAW, 8), 5),
    ];
    // (Cartographer 2) earth-and-glass trader; maps → ENCHANTED_BOOK
    const CARTOGRAPHER: &[Trade] = &[
        tr((CLAY, 16), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (GLASS, 4), 1),
        tr((SAND, 20), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 2), (TERRACOTTA, 4), 2),
        tr((GRAVEL, 16), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 3), (GLASS, 10), 3),
        tr((DIORITE, 12), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 5), (TERRACOTTA, 10), 4),
        tr((GRANITE, 12), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 8), (ENCHANTED_BOOK, 1), 5),
    ];
    // (Cleric 3) VERIFIED vanilla shape: buys Rotten Flesh, sells
    // potions / ender pearls / glowstone
    const CLERIC: &[Trade] = &[
        tr((ROTTEN_FLESH, 12), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (POTION_HEALING, 1), 1),
        tr((SPIDER_EYE, 8), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 2), (POTION_HEALING_II, 1), 2),
        tr((BONE, 12), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 3), (GLOWSTONE, 2), 3),
        tr((GUNPOWDER, 8), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 4), (POTION_HARMING, 1), 4),
        tr((EMERALD_ORE, 5), (ENDER_PEARL, 1), 5),
        tr((FERMENTED_SPIDER_EYE, 6), (EMERALD_ORE, 1), 5),
    ];
    // (Farmer 4) crop trader
    const FARMER: &[Trade] = &[
        tr((MELON, 8), (EMERALD_ORE, 1), 1),
        tr((PUMPKIN, 6), (EMERALD_ORE, 1), 1),
        tr((MUSHROOM_BROWN, 12), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (PUMPKIN, 4), 2),
        tr((MUSHROOM_RED, 12), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 3), (MELON, 6), 3),
        tr((CACTUS, 6), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 2), (MUSHROOM_BROWN, 6), 4),
        tr((MELON, 20), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 3), (MUSHROOM_RED, 8), 5),
    ];
    // (Fisherman 5) line-and-catch trader (fish absent → STRING/BONE/
    // LEATHER, the fishing-rod and bycatch materials)
    const FISHERMAN: &[Trade] = &[
        tr((STRING, 10), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (BONE, 6), 1),
        tr((FEATHER, 12), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (LEATHER, 2), 2),
        tr((GRAVEL, 20), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 2), (BONE, 12), 3),
        tr((ROTTEN_FLESH, 12), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 3), (LEATHER, 6), 4),
        tr((ENDER_PEARL, 1), (EMERALD_ORE, 4), 5),
        tr((EMERALD_ORE, 4), (SPIDER_EYE, 6), 5),
    ];
    // (Fletcher 6) VERIFIED vanilla shape: buys String + feathers,
    // sells arrows
    const FLETCHER: &[Trade] = &[
        tr((STRING, 15), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (ARROW_ITEM, 12), 1),
        tr((FEATHER, 12), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 2), (ARROW_ITEM, 24), 2),
        tr((BIRCH_LOG, 8), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 3), (ARROW_ITEM, 48), 3),
        tr((GRAVEL, 16), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 2), (BONE, 8), 4),
        tr((SPIDER_EYE, 8), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 6), (ARROW_ITEM, 96), 5),
    ];
    // (Leatherworker 7) hide trader
    const LEATHERWORKER: &[Trade] = &[
        tr((LEATHER, 6), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 2), (LEATHER, 6), 1),
        tr((ROTTEN_FLESH, 16), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (WOOL_WHITE, 4), 2),
        tr((BEEF, 6), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 3), (WOOL_RED, 8), 3),
        tr((LEATHER, 20), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 2), (WOOL_BLACK, 8), 4),
        tr((MUTTON, 16), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 4), (WOOL_BLUE, 12), 5),
    ];
    // (Librarian 8) paper-goods trader (paper → BOOKSHELF stand-in),
    // sells ENCHANTED_BOOK (vanilla's signature librarian offer)
    const LIBRARIAN: &[Trade] = &[
        tr((BOOKSHELF, 2), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 2), (ENCHANTED_BOOK, 1), 1),
        tr((SPRUCE_LOG, 10), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (BOOKSHELF, 2), 2),
        tr((TALL_GRASS, 20), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 4), (ENCHANTED_BOOK, 2), 3),
        tr((BIRCH_LOG, 16), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 6), (ENCHANTED_BOOK, 3), 4),
        tr((OAK_LOG, 24), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 10), (ENCHANTED_BOOK, 5), 5),
    ];
    // (Mason 9) stone trader (VERIFIED vanilla shape: buys stone/clay,
    // sells chiseled/smooth stone — here STONE_BRICKS/SMOOTH_STONE)
    const MASON: &[Trade] = &[
        tr((CLAY, 12), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (STONE_BRICKS, 4), 1),
        tr((STONE, 16), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 1), (SMOOTH_STONE, 4), 2),
        tr((COBBLE, 20), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 2), (STONE_BRICKS, 12), 3),
        tr((GRANITE, 12), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 3), (SMOOTH_STONE, 12), 4),
        tr((DIORITE, 16), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 4), (TERRACOTTA, 12), 5),
    ];
    // (Nitwit 10) — no trades (VERIFIED)
    const NITWIT_TRADES: &[Trade] = &[];
    // (Shepherd 11) wool trader (VERIFIED vanilla shape: 18 wool → em)
    const SHEPHERD: &[Trade] = &[
        tr((WOOL_WHITE, 12), (EMERALD_ORE, 1), 1),
        tr((EMERALD_ORE, 1), (LEATHER, 3), 1),
        tr((WOOL_BLACK, 12), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 2), (WOOL_RED, 6), 2),
        tr((WOOL_YELLOW, 12), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 2), (WOOL_BLUE, 6), 3),
        tr((WOOL_RED, 16), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 3), (WOOL_WHITE, 12), 4),
        tr((TALL_GRASS, 24), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 4), (WOOL_YELLOW, 12), 5),
    ];
    // (Toolsmith 12) buys fuel/ore, sells metal blocks
    const TOOLSMITH: &[Trade] = &[
        tr((COAL, 8), (EMERALD_ORE, 1), 1), // fix #4: the real coal item
        tr((EMERALD_ORE, 3), (IRON_BLOCK, 1), 1),
        tr((IRON_ORE, 8), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 4), (CRAFTING_TABLE, 2), 2),
        tr((GRAVEL, 24), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 6), (IRON_BLOCK, 2), 3),
        tr((GOLD_ORE, 6), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 8), (GOLD_BLOCK, 1), 4),
        tr((DIAMOND_ORE, 4), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 12), (DIAMOND_BLOCK, 1), 5),
    ];
    // (Unemployed 13) — no trades until a profession (VERIFIED)
    const UNEMPLOYED_TRADES: &[Trade] = &[];
    // (Weaponsmith 14) buys coal/iron/gravel (vanilla) → diamond at master
    const WEAPONSMITH: &[Trade] = &[
        tr((COAL, 10), (EMERALD_ORE, 1), 1), // fix #4: the real coal item
        tr((EMERALD_ORE, 4), (IRON_BLOCK, 1), 1),
        tr((IRON_ORE, 10), (EMERALD_ORE, 1), 2),
        tr((EMERALD_ORE, 7), (GOLD_BLOCK, 1), 2),
        tr((GRAVEL, 20), (EMERALD_ORE, 1), 3),
        tr((EMERALD_ORE, 9), (IRON_BLOCK, 2), 3),
        tr((GRAVEL, 32), (EMERALD_ORE, 1), 4),
        tr((EMERALD_ORE, 10), (OBSIDIAN, 4), 4),
        tr((DIAMOND_ORE, 6), (EMERALD_ORE, 1), 5),
        tr((EMERALD_ORE, 16), (DIAMOND_BLOCK, 1), 5),
    ];
    match profession as usize {
        0 => ARMORER,
        1 => BUTCHER,
        2 => CARTOGRAPHER,
        3 => CLERIC,
        4 => FARMER,
        5 => FISHERMAN,
        6 => FLETCHER,
        7 => LEATHERWORKER,
        8 => LIBRARIAN,
        9 => MASON,
        10 => NITWIT_TRADES,
        11 => SHEPHERD,
        12 => TOOLSMITH,
        13 => UNEMPLOYED_TRADES,
        _ => WEAPONSMITH,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Villager {
    pub id: u32,
    /// feet center
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub yaw: f32,
    /// the home anchor (the village well) — wander stays within its radius
    pub home: [i32; 3],
    pub profession: u8,
    /// career XP (drives level; VERIFIED thresholds 0/10/70/150/250)
    pub xp: u32,
    /// per-trade uses this restock cycle (parallel to the trade table)
    pub used: [u16; MAX_TRADES],
    /// sim tick of the last restock (2x/day cadence at the home "work site")
    pub restocked_at: u64,
    /// wander target (None = idle stand)
    target: Option<[f32; 3]>,
    /// ticks until the next wander decision
    wander_t: i32,
    /// cooldown after a jump
    jump_cd: i32,
    /// gossip with the player (VERIFIED table — see GossipKind)
    pub gossip: Gossip,
    /// health (VERIFIED: villagers have 20 HP, no natural armor)
    pub health: f32,
}

impl Villager {
    /// career level 1..=5 from current XP
    pub fn level(&self) -> u8 {
        level_for_xp(self.xp)
    }

    /// the trades the trade screen serves: rows of the current tier and
    /// below (vanilla: higher-tier offers appear on level-up)
    pub fn offers(&self) -> Vec<usize> {
        let lvl = self.level();
        trades(self.profession)
            .iter()
            .enumerate()
            .filter(|(_, t)| t.tier <= lvl)
            .map(|(i, _)| i)
            .collect()
    }

    /// stock left in trade row `i` (None = no such row)
    pub fn stock_left(&self, i: usize) -> Option<u16> {
        let t = trades(self.profession).get(i).copied()?;
        let used = self.used[i.min(MAX_TRADES - 1)];
        Some(t.max_uses.saturating_sub(used))
    }
}

pub struct Villagers {
    pub list: Vec<Villager>,
    rng: Rng,
    next_id: u32,
    /// village wells already populated (never double-spawn)
    populated: HashSet<[i32; 2]>,
    /// total trades executed since boot (stats/F3/E2E)
    pub trades_done: u64,
    /// total ever spawned (E2E)
    pub spawned_total: u64,
    /// sim tick of the last tick() call (restock clock)
    last_tick: u64,
    /// sim tick of the last gossip decay boundary (20-min clock)
    gossip_decay_at: u64,
    /// cadence counter for the pairwise gossip-sharing pass
    share_t: u32,
}

impl Villagers {
    pub fn new(seed: u64) -> Self {
        Villagers {
            list: Vec::with_capacity(32),
            rng: Rng::new(seed ^ 0x1A11A_9E),
            next_id: 1,
            populated: HashSet::new(),
            trades_done: 0,
            spawned_total: 0,
            last_tick: 0,
            gossip_decay_at: 0,
            share_t: 0,
        }
    }

    /// spawn one villager at a position with a chosen (or random) profession
    pub fn spawn_at(&mut self, wx: i32, wy: i32, wz: i32, profession: Option<u8>) -> Option<u32> {
        if self.list.len() >= MAX_VILLAGERS {
            return None;
        }
        let prof = profession.unwrap_or_else(|| {
            // village-roll flavor: mostly employed, some unemployed, a
            // few nitwits (the vanilla unemployment structure)
            match self.rng.next_range(20) {
                0..=1 => NITWIT,
                2..=4 => UNEMPLOYED,
                _ => self.rng.next_range(13) as u8,
            }
        });
        let id = self.next_id;
        self.next_id += 1;
        self.list.push(Villager {
            id,
            pos: [wx as f32 + 0.5, wy as f32 + 0.1, wz as f32 + 0.5],
            vel: [0.0; 3],
            yaw: 0.0,
            home: [wx, wy, wz],
            profession: prof.min(14),
            xp: 0,
            used: [0; MAX_TRADES],
            restocked_at: 0,
            target: None,
            wander_t: (self.rng.next_range(40) as i32).max(10),
            jump_cd: 0,
            gossip: Gossip::default(),
            health: 20.0,
        });
        self.spawned_total += 1;
        Some(id)
    }

    /// populate the villages whose reach covers the given chunk — called
    /// once per generated chunk; each village well seeds 3..5 villagers
    /// (deterministic from the world seed + well position)
    pub fn populate_villages(&mut self, world: &vc_world::world::World, cx: i32, cz: i32) {
        let ox = cx * 16;
        let oz = cz * 16;
        for (wx, wz) in world.gen.villages_near(ox, oz) {
            if self.populated.contains(&[wx, wz]) {
                continue;
            }
            self.populated.insert([wx, wz]);
            // ground at the well: the generator's column height (the well
            // sits at height+1; villagers stand on the well rim area)
            let h = world.gen.column(wx, wz).height;
            let mut rng = Rng::new(Rng::hash3(world.seed, wx, 0x0E5, wz));
            let n = 3 + rng.next_range(3) as usize; // 3..5
            for _ in 0..n {
                let dx = rng.next_range(7) as i32 - 3;
                let dz = rng.next_range(7) as i32 - 3;
                let gy = world.gen.column(wx + dx, wz + dz).height;
                let prof = match rng.next_range(20) {
                    0..=1 => NITWIT,
                    2..=4 => UNEMPLOYED,
                    _ => rng.next_range(13) as u8,
                };
                self.spawn_at(wx + dx, gy.max(h) + 1, wz + dz, Some(prof));
            }
        }
    }

    /// the villager under the crosshair: vertical-capsule test (0.6 wide,
    /// 1.9 tall — the vanilla villager hitbox) against the ray, within
    /// `max_dist` (interaction reach)
    pub fn ray_hit(&self, eye: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<u32> {
        let mut best: Option<(f32, u32)> = None;
        for v in &self.list {
            // project the capsule axis onto the ray
            let ox = v.pos[0] - eye[0];
            let oy = (v.pos[1] + 0.95) - eye[1]; // axis center
            let oz = v.pos[2] - eye[2];
            let t = ox * dir[0] + oy * dir[1] + oz * dir[2];
            if t < 0.0 || t > max_dist {
                continue;
            }
            let px = eye[0] + dir[0] * t;
            let py = eye[1] + dir[1] * t;
            let pz = eye[2] + dir[2] * t;
            // horizontal distance to the axis + vertical containment
            let hd2 = (px - v.pos[0]).powi(2) + (pz - v.pos[2]).powi(2);
            let v_ok = py >= v.pos[1] - 0.1 && py <= v.pos[1] + 1.9;
            if hd2 < 0.45 * 0.45 && v_ok {
                if best.map(|(bt, _)| t < bt).unwrap_or(true) {
                    best = Some((t, v.id));
                }
            }
        }
        best.map(|(_, id)| id)
    }

    pub fn by_id(&self, id: u32) -> Option<&Villager> {
        self.list.iter().find(|v| v.id == id)
    }

    /// execute trade row `i` of villager `id`: validates tier + stock,
    /// consumes stock, grants villager XP. Returns the Trade (for the
    /// caller to move items) and whether the trade LEVELED the villager.
    pub fn execute_trade(&mut self, id: u32, i: usize) -> Option<(Trade, bool)> {
        let v = self.list.iter_mut().find(|v| v.id == id)?;
        let t = trades(v.profession).get(i).copied()?;
        if t.tier > v.level() {
            return None; // locked tier
        }
        let used = v.used[i.min(MAX_TRADES - 1)];
        if used >= t.max_uses {
            return None; // out of stock until restock
        }
        v.used[i.min(MAX_TRADES - 1)] = used + 1;
        let before = v.level();
        v.xp = v.xp.saturating_add(t.xp as u32);
        // VERIFIED (§Gossiping): each trade gives the TARGET villager
        // +4 trading gossip (cap 25) — the reputation that discounts
        // future prices
        v.gossip.gain_event(GossipKind::Trading);
        let leveled = v.level() > before;
        self.trades_done += 1;
        Some((t, leveled))
    }

    /// restock pass: every RESTOCK_TICKS (twice a day) while near the
    /// home (= the job-site adaptation), used counts reset (VERIFIED
    /// "activate their offers again, up to twice per day")
    fn restock_pass(&mut self, sim_ticks: u64) {
        if sim_ticks < self.last_tick {
            self.last_tick = sim_ticks; // clock reset (new world / load)
        }
        if sim_ticks.saturating_sub(self.last_tick) < RESTOCK_TICKS {
            return;
        }
        // one boundary crossing = one restock opportunity; catch-up walks
        // the clock forward by whole restock periods
        let mut now = self.last_tick + RESTOCK_TICKS;
        while now <= sim_ticks {
            for v in self.list.iter_mut() {
                v.used = [0; MAX_TRADES];
                v.restocked_at = now;
            }
            now += RESTOCK_TICKS;
        }
        self.last_tick = sim_ticks;
    }

    /// Reputation with the (single) player for villager `id`
    /// (VERIFIED: Σ value × multiplier — positive lowers prices).
    pub fn reputation_of(&self, id: u32) -> Option<i32> {
        self.by_id(id).map(|v| v.gossip.reputation())
    }

    /// Player attacked villager `id` (survived): the targeted villager
    /// gains minor_negative +25 (cap 200) — VERIFIED.
    pub fn on_player_attack(&mut self, id: u32) {
        if let Some(v) = self.list.iter_mut().find(|v| v.id == id) {
            v.gossip.gain_event(GossipKind::MinorNegative);
        }
    }

    /// Player melee on villager `id` (VERIFIED: 20 HP, no armor).
    /// Returns (applied damage, Some(position) when the hit killed the
    /// villager — the entity is removed; the caller broadcasts the
    /// major_negative kill gossip to the survivors).
    pub fn damage(&mut self, id: u32, dmg: f32) -> (f32, Option<[f32; 3]>) {
        let Some(v) = self.list.iter_mut().find(|v| v.id == id) else {
            return (0.0, None);
        };
        let applied = dmg.min(v.health).max(0.0);
        v.health -= applied;
        if v.health <= 0.0 {
            let pos = v.pos;
            self.list.retain(|x| x.id != id);
            (applied, Some(pos))
        } else {
            (applied, None)
        }
    }

    /// Player killed a villager at `pos`: every villager within the
    /// VERIFIED 16-block box gets major_negative +25 (cap 100). (The
    /// wiki's line-of-sight condition is approximated by the box.)
    /// Returns how many villagers received the gossip.
    pub fn on_player_kill(&mut self, pos: [f32; 3]) -> usize {
        let mut n = 0;
        for v in self.list.iter_mut() {
            let inside = (v.pos[0] - pos[0]).abs() <= 16.0
                && (v.pos[1] - pos[1]).abs() <= 16.0
                && (v.pos[2] - pos[2]).abs() <= 16.0;
            if inside {
                v.gossip.gain_event(GossipKind::MajorNegative);
                n += 1;
            }
        }
        n
    }

    /// Gossip decay (VERIFIED: every 20 real minutes = 24000 ticks, each
    /// gossip decays by its per-type Decay value; major_positive decay 0
    /// is permanent). Same boundary-crossing shape as restock_pass.
    fn gossip_decay_pass(&mut self, sim_ticks: u64) {
        if sim_ticks < self.gossip_decay_at {
            self.gossip_decay_at = sim_ticks; // clock reset (new world / load)
        }
        if sim_ticks.saturating_sub(self.gossip_decay_at) < GOSSIP_DECAY_TICKS {
            return;
        }
        let mut now = self.gossip_decay_at + GOSSIP_DECAY_TICKS;
        while now <= sim_ticks {
            for v in self.list.iter_mut() {
                v.gossip.decay_pass();
            }
            now += GOSSIP_DECAY_TICKS;
        }
        self.gossip_decay_at = sim_ticks;
    }

    /// Gossip sharing (VERIFIED one-liner: villagers share gossip by
    /// talking; the shared value is reduced by the sharing cost;
    /// major_positive is unshareable since cost 100 > max 20).
    /// Adaptation: every 20 ticks each pair within 3 blocks has a 10%
    /// conversation chance; the conversation transfers the pair's
    /// strongest shareable type ("higher-strength gossip is more likely
    /// to be shared" — approximated by strongest-first).
    fn gossip_share_pass(&mut self) {
        if self.list.len() < 2 {
            return;
        }
        for i in 0..self.list.len() {
            for j in (i + 1)..self.list.len() {
                let dx = self.list[i].pos[0] - self.list[j].pos[0];
                let dy = self.list[i].pos[1] - self.list[j].pos[1];
                let dz = self.list[i].pos[2] - self.list[j].pos[2];
                if dx * dx + dy * dy + dz * dz > 9.0 {
                    continue;
                }
                if self.rng.next_range(10) != 0 {
                    continue;
                }
                // strongest shareable type across the pair
                let kinds = [
                    GossipKind::Trading,
                    GossipKind::MajorPositive,
                    GossipKind::MinorPositive,
                    GossipKind::MinorNegative,
                    GossipKind::MajorNegative,
                ];
                let mut best: Option<(GossipKind, u16, usize)> = None;
                for &k in &kinds {
                    if !k.shareable() {
                        continue;
                    }
                    for (src, val) in [
                        (i, self.list[i].gossip.get(k)),
                        (j, self.list[j].gossip.get(k)),
                    ] {
                        if val > 0 && best.map(|b| val > b.1).unwrap_or(true) {
                            best = Some((k, val, src));
                        }
                    }
                }
                let Some((k, val, src)) = best else { continue };
                let dst = if src == i { j } else { i };
                self.list[dst].gossip.receive_share(k, val);
            }
        }
    }

    /// ONE sim tick (20 Hz): wander decisions + walking physics + the
    /// twice-daily restock clock. `sim_ticks` is the global sim tick count.
    /// Phase 6 §26: villagers outside the simulation ring freeze — wander
    /// and walking physics (1.18+ semantics; radius i32::MAX = 1.16.5
    /// behavior). The restock day-clock stays global (documented
    /// simplification: a village's trades refresh on the day cycle even
    /// while frozen — freezing it would punish legitimate play patterns
    /// for a few CPU cycles).
    pub fn tick(
        &mut self,
        world: &vc_world::world::World,
        sim_ticks: u64,
        sim_center: (i32, i32),
        sim_radius: i32,
    ) {
        self.restock_pass(sim_ticks);
        self.gossip_decay_pass(sim_ticks);
        self.share_t = self.share_t.wrapping_add(1);
        if self.share_t % 20 == 0 {
            self.gossip_share_pass();
        }
        let ring = |p: [f32; 3]| {
            let (cx, cz) = ((p[0] / 16.0).floor() as i32, (p[2] / 16.0).floor() as i32);
            cx.wrapping_sub(sim_center.0)
                .saturating_abs()
                .max(cz.wrapping_sub(sim_center.1).saturating_abs())
                <= sim_radius
        };
        for v in self.list.iter_mut() {
            if !ring(v.pos) {
                continue; // out of the simulation ring: frozen this tick
            }
            // wander state machine: idle countdown → pick a target (with a
            // generous walk deadline) → walk until arrival or deadline →
            // idle again. wander_t is the countdown in BOTH states.
            if v.target.is_none() {
                if v.wander_t > 0 {
                    v.wander_t -= 1;
                } else {
                    let ang = self.rng.next_f32() * std::f32::consts::TAU;
                    let r = 2.0 + self.rng.next_f32() * 6.0;
                    v.target = Some([
                        v.home[0] as f32 + 0.5 + ang.cos() * r,
                        0.0, // y resolved during walking
                        v.home[2] as f32 + 0.5 + ang.sin() * r,
                    ]);
                    // walk deadline: 15..25 s covers any wander leg
                    v.wander_t = 300 + self.rng.next_range(200) as i32;
                }
            } else if v.wander_t > 0 {
                v.wander_t -= 1;
            } else {
                // gave up → idle
                v.target = None;
                v.wander_t = 40 + self.rng.next_range(80) as i32; // 2..6 s
            }

            // steering toward the target
            if let Some(t) = v.target {
                let dx = t[0] - v.pos[0];
                let dz = t[2] - v.pos[2];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < 0.35 {
                    v.target = None;
                    v.wander_t = 40 + self.rng.next_range(80) as i32;
                } else {
                    let speed = WALK_SPEED / 20.0; // per tick
                    v.vel[0] = dx / dist * speed;
                    v.vel[2] = dz / dist * speed;
                    v.yaw = dz.atan2(dx);
                }
            } else {
                v.vel[0] *= 0.6;
                v.vel[2] *= 0.6;
            }

            if v.jump_cd > 0 {
                v.jump_cd -= 1;
            }

            // physics: gravity + axis-separated collision (the item-entity
            // pattern, villager-scale); jump when horizontally blocked.
            // Vanilla entity gravity, EXACT per-tick form (VERIFIED,
            // research-verdicts.md: v1 = (v0 − 0.08) × 0.98 — villager
            // velocities are b/tick). Terminal −3.92 b/t is the inherent
            // fixed point; the old non-vanilla −0.5 clamp is gone, and
            // the vertical move is substepped so a terminal fall cannot
            // tunnel through 1–3-block floors.
            v.vel[1] = (v.vel[1] - 0.08) * 0.98;

            // horizontal X
            let nx = v.pos[0] + v.vel[0];
            if !solid_at(world, nx, v.pos[1] + 0.1, v.pos[2])
                && !solid_at(world, nx, v.pos[1] + 1.5, v.pos[2])
            {
                v.pos[0] = nx;
            } else if v.on_ground(world) && v.jump_cd == 0 {
                v.vel[1] = JUMP_VEL;
                v.jump_cd = 10;
            } else {
                v.vel[0] = 0.0;
            }
            // horizontal Z
            let nz = v.pos[2] + v.vel[2];
            if !solid_at(world, v.pos[0], v.pos[1] + 0.1, nz)
                && !solid_at(world, v.pos[0], v.pos[1] + 1.5, nz)
            {
                v.pos[2] = nz;
            } else if v.on_ground(world) && v.jump_cd == 0 {
                v.vel[1] = JUMP_VEL;
                v.jump_cd = 10;
            } else {
                v.vel[2] = 0.0;
            }
            // vertical — substepped (≤0.9 blocks per probe)
            let steps = (v.vel[1].abs() / 0.9).ceil().max(1.0) as i32;
            let step = v.vel[1] / steps as f32;
            'vertical: for _ in 0..steps {
                let ny = v.pos[1] + step;
                if step < 0.0 && solid_at(world, v.pos[0], ny, v.pos[2]) {
                    v.pos[1] = ny.floor() + 1.0; // rest on the surface
                    v.vel[1] = 0.0;
                    v.vel[0] *= 0.7;
                    v.vel[2] *= 0.7;
                    break 'vertical;
                } else if step > 0.0 && solid_at(world, v.pos[0], ny + 1.8, v.pos[2]) {
                    v.vel[1] = 0.0;
                    break 'vertical;
                }
                v.pos[1] = ny;
            }

            // never sink below bedrock world floor
            if v.pos[1] < 1.0 {
                v.pos[1] = 1.0;
                v.vel[1] = 0.0;
            }
        }
    }
}

impl Villager {
    fn on_ground(&self, world: &vc_world::world::World) -> bool {
        solid_at(world, self.pos[0], self.pos[1] - 0.05, self.pos[2])
    }
}

fn solid_at(world: &vc_world::world::World, x: f32, y: f32, z: f32) -> bool {
    is_solid(world.get_block(x.floor() as i32, y.floor() as i32, z.floor() as i32))
}

/// Reputation-adjusted give-count (price) for trade row `i` of villager
/// `v` — VERIFIED price rule (minecraft.wiki/w/Trading §Sale prices,
/// research-verdicts.md live round):
///     cost = clamp(base − floor(reputation × 0.05), 1, 64)
/// Positive reputation discounts, negative raises (Java behavior).
pub fn give_count_adjusted(v: &Villager, i: usize) -> u8 {
    let Some(t) = trades(v.profession).get(i) else {
        return 0;
    };
    let base = t.give.1 as i32;
    let discount = (v.gossip.reputation() as f32 * PRICE_MULTIPLIER).floor() as i32;
    (base - discount).clamp(1, 64) as u8
}

/// billboard quads for the render pass: one crossed pair per villager,
/// villager-scale (0.6 × 1.9) — rides the same particle pipeline the item
/// entities use
#[allow(clippy::too_many_arguments)]
pub fn build_vertices(
    list: &[Villager],
    time: f32,
    right: [f32; 3],
    up: [f32; 3],
    out: &mut Vec<vc_particles::particles::ParticleVertex>,
) {
    let tile = TILE_VILLAGER as u16;
    let tx = (tile % 16) as f32;
    let ty = (tile / 16) as f32;
    for v in list {
        // face the movement direction (billboard around world Y)
        let yaw = v.yaw + time * 0.0; // no spin — grounded NPCs
        let (s, c) = (yaw.sin(), yaw.cos());
        let rr = [
            c * right[0] + s * right[2],
            0.0,
            -s * right[0] + c * right[2],
        ];
        let half = 0.30f32;
        let h = 1.9f32;
        let col = [0.92, 0.86, 0.78]; // baked neutral light (villager robe tones come from the tile)
        let bob = 0.0;
        // the quad spans feet..head
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
                pos: [v.pos[0] + c[0], v.pos[1] + c[1] + bob, v.pos[2] + c[2]],
                uv: [uv[0], uv[1]],
                col,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_chunk::chunk::Chunk;

    fn flat_world() -> vc_world::world::World {
        let mut w = vc_world::world::World::new(9);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
                for y in 0..=64i32 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y as usize, lz, STONE);
                        }
                    }
                }
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w
    }

    #[test]
    fn spawn_walks_and_stays_near_home() {
        let mut vs = Villagers::new(3);
        let id = vs.spawn_at(0, 65, 0, Some(0)).unwrap();
        let w = flat_world();
        // 10 simulated seconds of wandering
        for t in 0..200 {
            vs.tick(&w, t as u64, (0, 0), i32::MAX);
        }
        let v = vs.by_id(id).unwrap();
        let d2 = (v.pos[0] - 0.5).powi(2) + (v.pos[2] - 0.5).powi(2);
        assert!(
            d2 < 9.0 * 9.0,
            "villager stays within the wander radius, d={}",
            d2.sqrt()
        );
        assert!(
            v.pos[1] >= 64.0 && v.pos[1] < 67.0,
            "on the ground: {}",
            v.pos[1]
        );
        assert_eq!(vs.list.len(), 1);
    }

    #[test]
    fn jumps_single_block_steps() {
        let mut w = flat_world();
        // a 1-block wall ahead
        let _ = w.set_block(2, 65, 0, COBBLE);
        let mut vs = Villagers::new(4);
        let id = vs.spawn_at(0, 65, 0, Some(1)).unwrap();
        // steer straight at the wall: force a target behind it + a fresh
        // walk deadline (the state machine would otherwise idle first)
        let i = vs.list.iter().position(|v| v.id == id).unwrap();
        vs.list[i].target = Some([6.5, 0.0, 0.5]);
        vs.list[i].wander_t = 400;
        let mut jumped = false;
        for t in 0..120 {
            vs.tick(&w, t as u64, (0, 0), i32::MAX);
            let v = vs.by_id(id).unwrap();
            if v.vel[1] > 0.1 {
                jumped = true;
            }
        }
        assert!(jumped, "villager jumps the 1-block step");
        let v = vs.by_id(id).unwrap();
        assert!(v.pos[0] > 2.0, "villager crossed the wall: {}", v.pos[0]);
    }

    #[test]
    fn ray_hit_finds_the_crosshair_villager() {
        let mut vs = Villagers::new(5);
        let id = vs.spawn_at(0, 65, 0, Some(2)).unwrap();
        let eye = [0.5, 66.6, -3.0];
        let dir = [0.0, 0.0, 1.0];
        assert_eq!(vs.ray_hit(eye, dir, 8.0), Some(id));
        // looking away: nothing
        assert_eq!(vs.ray_hit(eye, [1.0, 0.0, 0.0], 8.0), None);
        // too far: nothing
        assert_eq!(vs.ray_hit([0.5, 66.6, -30.0], dir, 8.0), None);
    }

    #[test]
    fn all_15_professions_have_ids_and_names() {
        assert_eq!(PROFESSIONS.len(), 15);
        assert_eq!(PROFESSION_IDS.len(), 15);
        for (n, id) in PROFESSIONS.iter().zip(PROFESSION_IDS.iter()) {
            assert!(!n.is_empty());
            assert!(id.starts_with("minecraft:"), "registry id {id}");
        }
        // the two trade-less types sit at their documented indices
        assert_eq!(PROFESSIONS[NITWIT as usize], "Nitwit");
        assert_eq!(PROFESSION_IDS[NITWIT as usize], "minecraft:nitwit");
        assert_eq!(PROFESSIONS[UNEMPLOYED as usize], "Unemployed");
        assert_eq!(PROFESSION_IDS[UNEMPLOYED as usize], "minecraft:unemployed");
    }

    #[test]
    fn trade_tables_cover_all_professions() {
        for p in 0..15u8 {
            let t = trades(p);
            if p == NITWIT || p == UNEMPLOYED {
                assert!(t.is_empty(), "{} has no trades", PROFESSIONS[p as usize]);
                continue;
            }
            // 5 tiers, 2 rows each
            assert_eq!(t.len(), 10, "{} rows", PROFESSIONS[p as usize]);
            for tier in 1..=5u8 {
                let rows = t.iter().filter(|x| x.tier == tier).count();
                assert_eq!(rows, 2, "{} tier {} rows", PROFESSIONS[p as usize], tier);
            }
            for tr in t {
                // every side is a real, obtainable item/block
                assert!(tr.give.0 != AIR && tr.get.0 != AIR);
                assert!(tr.give.1 > 0 && tr.get.1 > 0);
                assert_ne!(tr.give.0, tr.get.0, "no self-trades");
                // VERIFIED economics
                assert_eq!(tr.max_uses, TIER_USES[(tr.tier - 1) as usize]);
                assert_eq!(tr.xp, TIER_XP[(tr.tier - 1) as usize]);
            }
        }
    }

    #[test]
    fn career_levels_match_the_verified_thresholds() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(9), 1);
        assert_eq!(level_for_xp(10), 2); // Apprentice
        assert_eq!(level_for_xp(69), 2);
        assert_eq!(level_for_xp(70), 3); // Journeyman
        assert_eq!(level_for_xp(149), 3);
        assert_eq!(level_for_xp(150), 4); // Expert
        assert_eq!(level_for_xp(249), 4);
        assert_eq!(level_for_xp(250), 5); // Master
        assert_eq!(level_for_xp(9_999_999), 5);
        assert_eq!(level_name(3), "Journeyman");
    }

    #[test]
    fn trades_grant_xp_level_up_and_gate_tiers() {
        let mut vs = Villagers::new(30);
        let id = vs.spawn_at(0, 65, 0, Some(3)).unwrap(); // Cleric
                                                          // tier-1 row 0 (Rotten Flesh → emerald), xp 2/trade, 16 stock
                                                          // 5 trades = 10 xp → Apprentice (threshold 10)
        let mut leveled_at = None;
        for k in 0..5 {
            let (t, lv) = vs.execute_trade(id, 0).expect("trade executes");
            assert_eq!(t.tier, 1);
            if lv {
                leveled_at = Some(k + 1);
            }
        }
        assert_eq!(
            leveled_at,
            Some(5),
            "levels up exactly at 10 XP (5 trades × 2)"
        );
        assert_eq!(vs.by_id(id).unwrap().level(), 2);
        // tier-2 rows are now visible in offers()
        let offers = vs.by_id(id).unwrap().offers();
        assert!(
            offers.contains(&2) && offers.contains(&0),
            "tier-2 unlocked: {offers:?}"
        );
        // a tier-5 row is still locked (returns None)
        assert!(
            vs.execute_trade(id, 8).is_none(),
            "tier-5 row locked at level 2"
        );
    }

    #[test]
    fn stock_exhausts_then_restocks_twice_a_day() {
        let mut vs = Villagers::new(31);
        let id = vs.spawn_at(0, 65, 0, Some(1)).unwrap(); // Butcher
                                                          // row 0: tier 1 → 16 uses
        for _ in 0..16 {
            assert!(vs.execute_trade(id, 0).is_some(), "in stock");
        }
        assert!(vs.execute_trade(id, 0).is_none(), "out of stock at 16 uses");
        assert_eq!(vs.by_id(id).unwrap().stock_left(0), Some(0));
        // tick forward one full day — TWO restock windows pass
        let w = flat_world();
        for t in 0..12_000u64 {
            vs.tick(&w, t, (0, 0), i32::MAX);
        }
        assert!(vs.execute_trade(id, 0).is_some(), "restocked after the day");
        assert_eq!(vs.by_id(id).unwrap().stock_left(0), Some(15));
    }

    #[test]
    fn nitwit_and_unemployed_have_no_trade_flow() {
        let mut vs = Villagers::new(32);
        let n = vs.spawn_at(0, 65, 0, Some(NITWIT)).unwrap();
        let u = vs.spawn_at(2, 65, 2, Some(UNEMPLOYED)).unwrap();
        assert!(vs.execute_trade(n, 0).is_none());
        assert!(vs.execute_trade(u, 0).is_none());
        assert!(vs.by_id(n).unwrap().offers().is_empty());
        assert!(vs.by_id(u).unwrap().offers().is_empty());
    }

    #[test]
    fn populate_uses_the_village_well_once() {
        // whatever villages this world's generator reports near chunk
        // (0,0) — the invariant is: the SECOND populate call for the same
        // chunk is a complete no-op (the well set guards double-spawn)
        let mut vs = Villagers::new(6);
        let w = flat_world();
        vs.populate_villages(&w, 0, 0);
        let n = vs.list.len();
        assert!(
            n == 0 || (3..=5).contains(&n),
            "villages seed 3..5 villagers, got {n}"
        );
        vs.populate_villages(&w, 0, 0);
        assert_eq!(vs.list.len(), n, "second populate: zero new spawns");
    }

    // ---------------------------------------------------- gossip (VERIFIED) --

    #[test]
    fn gossip_table_matches_wiki() {
        // VERIFIED minecraft.wiki/w/Villager §Gossiping (research-verdicts
        // live round): (gain, decay, share cost, maximum, multiplier)
        assert_eq!(GossipKind::Trading.gain(), 4);
        assert_eq!(GossipKind::Trading.decay(), 2);
        assert_eq!(GossipKind::Trading.share_cost(), 20);
        assert_eq!(GossipKind::Trading.max(), 25);
        assert_eq!(GossipKind::Trading.multiplier(), 1);
        assert_eq!(GossipKind::MajorPositive.gain(), 20);
        assert_eq!(GossipKind::MajorPositive.decay(), 0);
        assert_eq!(GossipKind::MajorPositive.share_cost(), 100);
        assert_eq!(GossipKind::MajorPositive.max(), 20);
        assert_eq!(GossipKind::MajorPositive.multiplier(), 5);
        assert_eq!(GossipKind::MinorPositive.gain(), 25);
        assert_eq!(GossipKind::MinorPositive.decay(), 1);
        assert_eq!(GossipKind::MinorPositive.share_cost(), 5);
        assert_eq!(GossipKind::MinorPositive.max(), 25);
        assert_eq!(GossipKind::MinorNegative.gain(), 25);
        assert_eq!(GossipKind::MinorNegative.decay(), 20);
        assert_eq!(GossipKind::MinorNegative.share_cost(), 20);
        assert_eq!(GossipKind::MinorNegative.max(), 200);
        assert_eq!(GossipKind::MinorNegative.multiplier(), -1);
        assert_eq!(GossipKind::MajorNegative.gain(), 25);
        assert_eq!(GossipKind::MajorNegative.decay(), 10);
        assert_eq!(GossipKind::MajorNegative.share_cost(), 10);
        assert_eq!(GossipKind::MajorNegative.max(), 100);
        assert_eq!(GossipKind::MajorNegative.multiplier(), -5);
        // the wiki note: major_positive can never be shared
        assert!(!GossipKind::MajorPositive.shareable());
        assert!(GossipKind::Trading.shareable());
    }

    #[test]
    fn reputation_and_price_discount() {
        let mut vs = Villagers::new(4);
        let id = vs.spawn_at(0, 65, 0, Some(4)).unwrap();
        // baseline price = table value
        let base = give_count_adjusted(vs.by_id(id).unwrap(), 0);
        assert_eq!(base, trades(4)[0].give.1);
        // 7 trades → trading gossip 28 → clamped at max 25 → rep 25
        for _ in 0..7 {
            vs.execute_trade(id, 0);
        }
        let v = vs.by_id(id).unwrap();
        assert_eq!(v.gossip.trading, 25, "trading caps at 25");
        assert_eq!(v.gossip.reputation(), 25);
        // discount = floor(25 × 0.05) = 1 → price drops by 1 (min 1)
        let disc = give_count_adjusted(v, 0);
        assert_eq!(disc as i32, (base as i32 - 1).max(1));
        // attack once: minor_negative 25 → rep = 25 − 25 = 0 → base price
        vs.on_player_attack(id);
        let v = vs.by_id(id).unwrap();
        assert_eq!(v.gossip.minor_negative, 25);
        assert_eq!(v.gossip.reputation(), 0);
        assert_eq!(give_count_adjusted(v, 0), base);
        // second attack: minor_negative 50 → rep −25 → surcharge: Java
        // Math.floor semantics — floor(−25 × 0.05) = floor(−1.25) = −2,
        // so the price rises by 2 (vanilla rounds toward −infinity)
        vs.on_player_attack(id);
        let v = vs.by_id(id).unwrap();
        assert_eq!(v.gossip.reputation(), -25);
        assert_eq!(give_count_adjusted(v, 0) as i32, (base as i32 + 2).min(64));
    }

    #[test]
    fn kill_broadcasts_major_negative_within_16_blocks() {
        let mut vs = Villagers::new(4);
        let a = vs.spawn_at(0, 65, 0, Some(4)).unwrap();
        let b = vs.spawn_at(8, 65, 0, Some(4)).unwrap(); // within 16
        let c = vs.spawn_at(200, 65, 0, Some(4)).unwrap(); // far
        let n = vs.on_player_kill([4.5, 65.0, 0.5]);
        assert_eq!(n, 2, "the two villagers within the 16-block box");
        assert_eq!(vs.by_id(a).unwrap().gossip.major_negative, 25);
        assert_eq!(vs.by_id(b).unwrap().gossip.major_negative, 25);
        assert_eq!(vs.by_id(c).unwrap().gossip.major_negative, 0);
        // each kill adds +25 up to the cap of 100; reputation −5 per point
        for _ in 0..5 {
            vs.on_player_kill([4.5, 65.0, 0.5]);
        }
        assert_eq!(vs.by_id(a).unwrap().gossip.major_negative, 100, "cap 100");
        assert_eq!(vs.by_id(a).unwrap().gossip.reputation(), -500);
    }

    #[test]
    fn gossip_decays_every_20_minutes() {
        let mut vs = Villagers::new(4);
        let id = vs.spawn_at(0, 65, 0, Some(4)).unwrap();
        vs.execute_trade(id, 0);
        vs.on_player_attack(id);
        let v = vs.by_id(id).unwrap();
        assert_eq!((v.gossip.trading, v.gossip.minor_negative), (4, 25));
        // one 20-minute period (24000 ticks) → trading −2, minor_neg −20
        let w = flat_world();
        vs.tick(&w, GOSSIP_DECAY_TICKS, (0, 0), i32::MAX);
        let v = vs.by_id(id).unwrap();
        assert_eq!(v.gossip.trading, 2);
        assert_eq!(v.gossip.minor_negative, 5);
        // a second period floors minor_negative at 0, trading follows
        vs.tick(&w, GOSSIP_DECAY_TICKS * 2, (0, 0), i32::MAX);
        let v = vs.by_id(id).unwrap();
        assert_eq!(v.gossip.trading, 0);
        assert_eq!(v.gossip.minor_negative, 0);
        // major_positive is permanent (decay 0) — direct structural check
        let mut g = Gossip::default();
        g.set(GossipKind::MajorPositive, 20);
        g.decay_pass();
        assert_eq!(g.major_positive, 20);
    }

    #[test]
    fn villager_health_and_player_kill_path() {
        // VERIFIED: 20 HP; two 10-damage hits kill; the kill removes the
        // entity and the caller-side broadcast sees only the survivors
        let mut vs = Villagers::new(4);
        let a = vs.spawn_at(0, 65, 0, Some(4)).unwrap();
        let b = vs.spawn_at(4, 65, 0, Some(4)).unwrap();
        assert_eq!(vs.by_id(a).unwrap().health, 20.0);
        let (d1, kill) = vs.damage(a, 10.0);
        assert_eq!((d1, kill.is_none()), (10.0, true));
        assert_eq!(vs.by_id(a).unwrap().health, 10.0);
        // surviving a hit grows minor_negative (the game layer calls
        // on_player_attack per non-lethal hit)
        vs.on_player_attack(a);
        assert_eq!(vs.by_id(a).unwrap().gossip.minor_negative, 25);
        let (d2, kill) = vs.damage(a, 15.0);
        assert_eq!((d2, kill.is_some()), (10.0, true));
        assert!(vs.by_id(a).is_none(), "dead villager is removed");
        // the caller broadcasts the kill gossip
        let n = vs.on_player_kill(kill.unwrap());
        assert_eq!(n, 1, "only the survivor is in the 16-block box");
        assert_eq!(vs.by_id(b).unwrap().gossip.major_negative, 25);
    }

    #[test]
    fn gossip_shares_reduced_by_cost() {
        let mut vs = Villagers::new(4);
        let a = vs.spawn_at(0, 65, 0, Some(4)).unwrap();
        let b = vs.spawn_at(1, 65, 0, Some(4)).unwrap();
        // a holds trading 24; sharing cost 20 → b receives 4
        let v = vs.list.iter_mut().find(|v| v.id == a).unwrap();
        v.gossip.set(GossipKind::Trading, 24);
        // force a conversation (10% roll → call the pass directly)
        for _ in 0..200 {
            vs.gossip_share_pass();
        }
        let g = vs.by_id(b).unwrap().gossip;
        assert!(
            g.trading >= 4,
            "received 24 − 20 = 4 (more if a re-shared), got {}",
            g.trading
        );
        // the sender is never reduced by sharing; a mutual pair converges
        // toward the type max through re-sharing (25 here) — never below 24
        let a_val = vs.by_id(a).unwrap().gossip.trading;
        assert!(
            (24..=25).contains(&a_val),
            "sender never reduced, got {a_val}"
        );
    }
}
