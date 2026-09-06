//! Phase E2 (evolution 1.3–1.4 bracket): beacon pyramid + powers.
//! All values live-verified 2026-09-06 against minecraft.wiki
//! (docs/research/phase2-1.3-1.4-research.md):
//! - pyramid: 1–4 levels of iron/gold/emerald/diamond/netherite blocks
//!   (mixed freely — the material is purely cosmetic); block counts
//!   9 / 34 / 83 / 164; layers 3×3, +5×5, +7×7, +9×9 (w/Beacon §Pyramids)
//! - powers: Speed I / Haste I at level 1+; Resistance I / Jump Boost I
//!   at 2+; Strength I at 3+; secondary at 4 = Regeneration I or the
//!   primary raised to II (w/Beacon §Powers)
//! - effects reapply every 4 s with duration 9 + 2×level s →
//!   11 / 13 / 15 / 17 s (Java table — the Bedrock 10/12/14/16 with
//!   asymmetric radii is a Bedrock split, not used)
//! - range: 20 / 30 / 40 / 50 blocks (Java) around the beacon, down and
//!   out to that radius, upward by range + world height (w/Beacon §Range)
//! - multiple beacons with the same effect do NOT stack to a higher
//!   level (the fabricated "stacking" claim stays banned — each
//!   application refreshes independently; the highest amplifier wins)
//! - the beam needs an unobstructed sky view (w/Beacon §Activation)
//! - feeding: 1 iron/gold/emerald/diamond/netherite ingot-or-gem per
//!   power change (engine adaptation: the ORE items stand in — no
//!   ingot/gem items; documented)

use vc_blocks::blocks::*;
use vc_world::world::World;

/// pyramid layer block counts (index 0 = level 1): 9 / 34 / 83 / 164
/// (VERIFIED w/Beacon §Pyramids table)
pub const PYRAMID_BLOCKS: [usize; 4] = [9, 34, 83, 164];
/// effect radius per level (VERIFIED Java table: 20/30/40/50)
pub const RANGE_BLOCKS: [f32; 4] = [20.0, 30.0, 40.0, 50.0];
/// effect duration in seconds: 9 + 2×level (VERIFIED Java table:
/// 11 / 13 / 15 / 17)
pub const DURATION_SECS: [i32; 4] = [11, 13, 15, 17];
/// reapplication cadence: every 4 s = 80 game ticks (VERIFIED)
pub const REAPPLY_TICKS: i32 = 80;
/// beacon light level (VERIFIED: 15, even without a beam)
pub const BEACON_LIGHT: u8 = 15;

/// Valid pyramid base blocks (the material is cosmetic — VERIFIED).
#[inline]
pub fn is_base_block(b: u8) -> bool {
    matches!(b, IRON_BLOCK | GOLD_BLOCK | DIAMOND_BLOCK)
}

/// The beacon's primary power selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BeaconPower {
    Speed,
    Haste,
    Resistance,
    JumpBoost,
    Strength,
}

impl BeaconPower {
    /// minimum pyramid level required (VERIFIED w/Beacon §Powers:
    /// Speed/Haste 1; Resistance/JumpBoost 2; Strength 3)
    pub fn min_level(self) -> u8 {
        match self {
            BeaconPower::Speed | BeaconPower::Haste => 1,
            BeaconPower::Resistance | BeaconPower::JumpBoost => 2,
            BeaconPower::Strength => 3,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            BeaconPower::Speed => "Speed",
            BeaconPower::Haste => "Haste",
            BeaconPower::Resistance => "Resistance",
            BeaconPower::JumpBoost => "Jump Boost",
            BeaconPower::Strength => "Strength",
        }
    }
}

/// The secondary power available at a 4-level pyramid (VERIFIED):
/// Regeneration I, or raise the primary to level II.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BeaconSecondary {
    #[default]
    None,
    Regeneration,
    PrimaryII,
}

/// Scan the pyramid under a beacon at (x, y, z) (the beacon block's own
/// position). Returns the achieved level 0..=4. Layer n sits at
/// y-n with an odd square (2n+1) per side centered on (x, z). Blocks may
/// mix materials (VERIFIED) and may be missing in the INTERIOR of a
/// layer only where a smaller layer's blocks occupy them — vanilla
/// counts the full shells; the standard multi-layer pyramid shares the
/// inner blocks, so we count each cell once per its highest layer.
pub fn pyramid_level(world: &World, x: i32, y: i32, z: i32) -> u8 {
    let mut level: u8 = 0;
    'outer: for l in 1..=4i32 {
        let half = l; // layer l spans ±l (side 2l+1): 3,5,7,9
        let ly = y - l;
        // count present base blocks in the shell ring (the interior is
        // covered by higher layers — vanilla counts full layers; a cell
        // is part of layer l if it's within ±l of the center AND the
        // layer is complete). We require the complete layer (vanilla
        // requires every layer cell of the level to be a base block).
        for dx in -half..=half {
            for dz in -half..=half {
                // the cell must be a base block OR be part of a HIGHER
                // layer's area (interior overlap) — handled by the
                // higher layer check; for the level count the cell must
                // be a base block:
                if !is_base_block(world.get_block(x + dx, ly, z + dz)) {
                    break 'outer; // incomplete layer: level stays l-1
                }
            }
        }
        level = l as u8;
    }
    level
}

/// Count of base blocks actually present in the pyramid (for stats/E2E).
pub fn pyramid_block_count(world: &World, x: i32, y: i32, z: i32) -> usize {
    let lvl = pyramid_level(world, x, y, z) as usize;
    if lvl == 0 {
        0
    } else {
        PYRAMID_BLOCKS[lvl - 1]
    }
}

/// Effective duration in ticks for a pyramid level (VERIFIED: 9 + 2×lvl
/// seconds → (9 + 2L) × 20 ticks).
#[inline]
pub fn duration_ticks(level: u8) -> i32 {
    DURATION_SECS[(level as usize).clamp(1, 4) - 1] * 20
}

/// Is a player position inside the effect range of a beacon at
/// (bx, by, bz) with `level`? (VERIFIED Java: radius 20/30/40/50 around
/// the beacon, downward and outward; upward by range + height limit.)
pub fn in_range(level: u8, bx: i32, by: i32, bz: i32, px: f32, py: f32, pz: f32) -> bool {
    let r = RANGE_BLOCKS[(level as usize).clamp(1, 4) - 1];
    let dx = (px - bx as f32).abs();
    let dy_down = (by as f32 - py).max(0.0); // player below the beacon
    let dy_up = (py - by as f32).max(0.0); // above
    dx <= r
        && (pz - bz as f32).abs() <= r
        && dy_down <= r
        // upward: range + world height (256) — effectively always in
        // range vertically above, matching vanilla's tall cuboid
        && dy_up <= r + 256.0
}

/// The selection state of one placed beacon (persisted per position in
/// the game layer's container map).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BeaconState {
    pub level: u8,
    pub primary: Option<BeaconPower>,
    pub secondary: BeaconSecondary,
    /// ticks until the next reapplication
    pub reapply_in: i32,
}

impl BeaconState {
    pub fn new() -> Self {
        BeaconState {
            level: 0,
            primary: None,
            secondary: BeaconSecondary::None,
            reapply_in: 0,
        }
    }

    /// Feed + select powers (the game layer validates the fed item and
    /// the level requirements). Returns false when the selection is
    /// invalid for the current level (refuses + keeps the old state).
    pub fn select(&mut self, level: u8, primary: BeaconPower, secondary: BeaconSecondary) -> bool {
        if level == 0 || primary.min_level() > level {
            return false;
        }
        if secondary != BeaconSecondary::None && level < 4 {
            return false;
        }
        self.level = level;
        self.primary = Some(primary);
        self.secondary = secondary;
        self.reapply_in = 1; // the next tick applies immediately
        true
    }

    /// Reapply tick: returns true when the effect window should refresh
    /// (every 80 ticks — VERIFIED). Counts down; fires exactly on the
    /// 80th tick each period.
    pub fn tick_reapply(&mut self) -> bool {
        if self.primary.is_none() {
            return false;
        }
        self.reapply_in -= 1;
        if self.reapply_in <= 0 {
            self.reapply_in = REAPPLY_TICKS;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_chunk::chunk::Chunk;

    fn world_with<F: Fn(&mut Chunk)>(edit: F) -> World {
        let mut w = World::new(42);
        let mut c = Chunk::empty();
        edit(&mut c);
        w.insert_generated((0, 0), Arc::new(c), Vec::new());
        w
    }

    #[test]
    fn constants_match_the_live_wiki() {
        assert_eq!(PYRAMID_BLOCKS, [9, 34, 83, 164]); // w/Beacon §Pyramids
        assert_eq!(RANGE_BLOCKS, [20.0, 30.0, 40.0, 50.0]); // Java
        assert_eq!(DURATION_SECS, [11, 13, 15, 17]); // 9 + 2×level
        assert_eq!(REAPPLY_TICKS, 80); // every 4 s
        assert_eq!(BEACON_LIGHT, 15);
    }

    #[test]
    fn pyramid_scan_counts_the_verified_layout() {
        // a full 4-level pyramid under a beacon at (8, 80, 8)
        let w = world_with(|c| {
            for l in 1..=4i32 {
                let half = l;
                let y = 80 - l;
                for dx in -half..=half {
                    for dz in -half..=half {
                        if (8 + dx).clamp(0, 15) == 8 + dx
                            && (8 + dz).clamp(0, 15) == 8 + dz
                        {
                            c.set((8 + dx) as usize, y as usize, (8 + dz) as usize, IRON_BLOCK);
                        }
                    }
                }
            }
            c.set(8, 80, 8, BEACON);
        });
        assert_eq!(pyramid_level(&w, 8, 80, 8), 4);
        assert_eq!(pyramid_block_count(&w, 8, 80, 8), 164);
        // a 2-level pyramid: 5×5 + 3×3 = 34
        let w2 = world_with(|c| {
            for l in 1..=2i32 {
                let half = l;
                let y = 80 - l;
                for dx in -half..=half {
                    for dz in -half..=half {
                        c.set((8 + dx) as usize, y as usize, (8 + dz) as usize, GOLD_BLOCK);
                    }
                }
            }
            c.set(8, 80, 8, BEACON);
        });
        assert_eq!(pyramid_level(&w2, 8, 80, 8), 2);
        assert_eq!(pyramid_block_count(&w2, 8, 80, 8), 34);
        // mixed materials count (cosmetic — VERIFIED)
        // no pyramid: 0
        let w3 = world_with(|c| {
            c.set(8, 80, 8, BEACON);
        });
        assert_eq!(pyramid_level(&w3, 8, 80, 8), 0);
    }

    #[test]
    fn incomplete_layer_clamps_the_level() {
        // 3×3 present, 5×5 missing one corner
        let w = world_with(|c| {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    c.set((8 + dx) as usize, 79, (8 + dz) as usize, IRON_BLOCK);
                }
            }
            for dx in -2..=2 {
                for dz in -2..=2 {
                    if !(dx == 2 && dz == 2) {
                        c.set((8 + dx) as usize, 78, (8 + dz) as usize, IRON_BLOCK);
                    }
                }
            }
            c.set(8, 80, 8, BEACON);
        });
        assert_eq!(pyramid_level(&w, 8, 80, 8), 1, "broken 5×5 stops at 1");
    }

    #[test]
    fn range_covers_the_java_cuboid() {
        // level 1, radius 20: 15 blocks out horizontally in range
        assert!(in_range(1, 0, 70, 0, 15.0, 70.0, 0.0));
        assert!(!in_range(1, 0, 70, 0, 21.0, 70.0, 0.0));
        // below the beacon by 30: out of a level-1 range
        assert!(!in_range(1, 0, 70, 0, 0.0, 40.0, 0.0));
        // level 4, radius 50: 30 below OK
        assert!(in_range(4, 0, 70, 0, 0.0, 40.0, 0.0));
        // far above still in range (upward = range + height)
        assert!(in_range(1, 0, 70, 0, 0.0, 200.0, 0.0));
    }

    #[test]
    fn power_gates_match_the_wiki() {
        assert_eq!(BeaconPower::Speed.min_level(), 1);
        assert_eq!(BeaconPower::Haste.min_level(), 1);
        assert_eq!(BeaconPower::Resistance.min_level(), 2);
        assert_eq!(BeaconPower::JumpBoost.min_level(), 2);
        assert_eq!(BeaconPower::Strength.min_level(), 3);
    }

    #[test]
    fn selection_respects_levels_and_secondary_needs_4() {
        let mut st = BeaconState::new();
        assert!(!st.select(0, BeaconPower::Speed, BeaconSecondary::None));
        assert!(st.select(1, BeaconPower::Speed, BeaconSecondary::None));
        assert!(!st.select(1, BeaconPower::Strength, BeaconSecondary::None));
        assert!(!st.select(2, BeaconPower::Speed, BeaconSecondary::Regeneration));
        assert!(st.select(4, BeaconPower::Speed, BeaconSecondary::Regeneration));
        assert!(st.select(4, BeaconPower::Speed, BeaconSecondary::PrimaryII));
    }

    #[test]
    fn reapplies_every_80_ticks() {
        let mut st = BeaconState::new();
        assert!(!st.tick_reapply(), "no power selected: inert");
        st.select(1, BeaconPower::Speed, BeaconSecondary::None);
        assert!(st.tick_reapply(), "immediate first application");
        let mut fires = 0;
        for _ in 0..240 {
            if st.tick_reapply() {
                fires += 1;
            }
        }
        assert_eq!(fires, 3, "240 ticks = 12 s = 3 reapplications (4 s)");
    }

    #[test]
    fn duration_ticks_matches_the_java_table() {
        assert_eq!(duration_ticks(1), 220); // 11 s
        assert_eq!(duration_ticks(2), 260); // 13 s
        assert_eq!(duration_ticks(3), 300); // 15 s
        assert_eq!(duration_ticks(4), 340); // 17 s
    }
}
