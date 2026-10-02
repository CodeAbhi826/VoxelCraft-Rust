//! vc-gameplay — the Nether portal (Round K): the frame validation, the
//! portal search, and the far-side build-spot scan. Every rule is the
//! live-verified vanilla 1.16.5 behavior (reference wiki /Nether_portal +
//! /Nether_Portal_(block), live 2026-09-26); the functions are pure over
//! the World — the game layer owns the world edits (the interior fill,
//! the frame/platform build) and the light-engine hooks, exactly the
//! crate layering of every other gameplay system.

use vc_blocks::blocks::{
    state_block, AIR, FIRE, FLOWER_RED, FLOWER_YELLOW, NETHER_PORTAL, OBSIDIAN, TALL_GRASS,
};
use vc_world::world::World;

/// VERIFIED Nether_portal§Portal_search (live 2026-09-26): the JE point
/// of interest "can be within a 256×256 square block area (128-block
/// radius) in the Overworld and a 32×32 square block area (16-block
/// radius) in the Nether" (MC-197538, WAI). The 1.16.2 20w28a row
/// reduced the Nether-side radius from 128 to 16 "to correctly account
/// for the 1:8 position scale" — the 1.16.5 target ships 128/16.
pub const SEARCH_RADIUS_OVERWORLD: i32 = 128;
pub const SEARCH_RADIUS_NETHER: i32 = 16;

/// VERIFIED Nether_portal§Behavior (live 2026-09-26): "a player in the
/// Overworld or the Nether stands in a Nether portal block for 80 game
/// ticks (4 seconds) in survival mode or 1 game tick (1⁄20 second) in
/// creative mode, the player is taken to the other dimension".
pub const TRAVEL_TICKS_SURVIVAL: u32 = 80;
pub const TRAVEL_TICKS_CREATIVE: u32 = 1;

/// VERIFIED Nether_portal§Creation (live 2026-09-26): "a vertical,
/// rectangular frame of obsidian (4×5 minimum, 23×23 maximum)" — the
/// interior is 2 wide × 3 tall minimum, 21×21 maximum; "the four corners
/// of the frame are not required".
pub const MIN_INTERIOR_W: u32 = 2;
pub const MIN_INTERIOR_H: u32 = 3;
pub const MAX_INTERIOR_W: u32 = 21;
pub const MAX_INTERIOR_H: u32 = 21;

/// VERIFIED Nether_portal§Portal_creation (live 2026-09-26): "the game
/// creates a new one by looking for the closest suitable location to
/// place a portal, within 16 blocks horizontally (but any distance
/// vertically) of the incoming entity's destination coordinates".
pub const BUILD_HORIZON: i32 = 16;

/// VERIFIED Nether_portal§Portal_creation (live 2026-09-26): "a portal
/// is forced at the target coordinates, but with Y constrained to be
/// between 70 and 10 less than the world height (i.e. 118 for the Nether
/// or 310 for the Overworld)". The engine's world height is 256
/// (vc-chunk CHUNK_LEN 16·256·16) — the disclosed adaptation clamps the
/// Overworld ceiling to 246 (256 − 10); the Nether ceiling stays the
/// vanilla 118 (the engine's Nether is a cavern under the bedrock roof,
/// the same 128-tall band the vanilla value assumes).
pub const FORCED_Y_MIN: i32 = 70;
pub const FORCED_Y_NETHER_MAX: i32 = 118;
pub const FORCED_Y_OVERWORLD_MAX: i32 = 246;

/// the wiki-exact 8:1 coordinate conversion for the portal travel
/// (VERIFIED Nether_portal§Coordinate_conversion, live 2026-09-26):
/// "for a given location (X, Y, Z) in the Overworld, the corresponding
/// coordinates in the Nether are (floor(X ÷ 8), Y, floor(Z ÷ 8)), and
/// conversely... the matching Overworld coordinates are (X × 8, Y,
/// Z × 8)". The Java floor() "rounds down to the largest integer less
/// than or equal to the argument (toward smaller positive values and
/// toward larger negative values)" — i.e. floor division, NOT
/// truncation: div_euclid matches it exactly (floor(−29 ÷ 8) = −4).
/// The Y-coordinate is not changed (the caller keeps it).
#[inline]
pub fn portal_coords(entry_is_nether: bool, x: i32, z: i32) -> (i32, i32) {
    if entry_is_nether {
        (x * 8, z * 8)
    } else {
        (x.div_euclid(8), z.div_euclid(8))
    }
}

/// the walk-in wait in seconds (VERIFIED Nether_portal§Behavior, live
/// 2026-09-26: "stands in a Nether portal block for 80 game ticks
/// (4 seconds) in survival mode or 1 game tick (1⁄20 second) in creative
/// mode" — 80 ticks at 20 tps = 4.0 s, 1 tick = 0.05 s).
#[inline]
pub fn travel_wait_secs(creative: bool) -> f32 {
    if creative {
        TRAVEL_TICKS_CREATIVE as f32 / 20.0
    } else {
        TRAVEL_TICKS_SURVIVAL as f32 / 20.0
    }
}

/// The portal plane's long axis (the interior's width axis): X = the
/// interior extends along X (the frame spans X×Y and faces ±Z); Z = the
/// interior extends along Z (facing ±X). VERIFIED
/// Nether_portal§Portal_creation (live 2026-09-26): the new portal is
/// built "with the long axis matching the long axis of the source
/// portal".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortalAxis {
    X,
    Z,
}

impl PortalAxis {
    /// the horizontal unit step along the interior width
    #[inline]
    pub fn step(self) -> (i32, i32) {
        match self {
            PortalAxis::X => (1, 0),
            PortalAxis::Z => (0, 1),
        }
    }
}

/// one validated portal: the interior rectangle (inclusive min/max) +
/// the long axis. The interior is `2×3..=21×21` cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PortalFrame {
    pub axis: PortalAxis,
    /// the interior's minimum cell (inclusive)
    pub min: [i32; 3],
    /// the interior's maximum cell (inclusive)
    pub max: [i32; 3],
}

impl PortalFrame {
    /// the interior's width along its long axis
    #[inline]
    pub fn width(&self) -> u32 {
        let (sx, sz) = self.axis.step();
        let a = if sx != 0 {
            self.max[0] - self.min[0]
        } else {
            self.max[2] - self.min[2]
        };
        (a + 1) as u32
    }

    /// the interior's height along Y
    #[inline]
    pub fn height(&self) -> u32 {
        (self.max[1] - self.min[1] + 1) as u32
    }

    /// every interior cell, in a stable order (width-major bottom-up —
    /// the fill order is deterministic for the light-engine hooks)
    pub fn interior_cells(&self) -> Vec<[i32; 3]> {
        let (sx, sz) = self.axis.step();
        let w = self.width() as i32;
        let h = self.height() as i32;
        let mut out = Vec::with_capacity((w * h) as usize);
        for k in 0..h {
            for i in 0..w {
                out.push([self.min[0] + sx * i, self.min[1] + k, self.min[2] + sz * i]);
            }
        }
        out
    }
}

/// the interior-candidate blocks of the frame detection: air, the fire
/// the flint-and-steel just placed, or an existing portal block (the
/// re-validation path — vanilla's detection runs before the fill AND
/// when the portal block re-checks its structure; water/plants do NOT
/// count — the detection would run away through them). VERIFIED
/// Nether_portal§Creation (live 2026-09-26): "it is activated by fire
/// placed inside the frame... This creates portal blocks inside the
/// frame".
#[inline]
fn interior_cell(b: u16) -> bool {
    matches!(b, AIR | FIRE | NETHER_PORTAL)
}

/// is this block a nether-portal frame cell? The frame material is
/// OBSIDIAN strictly — VERIFIED Nether_portal§Creation (live 2026-09-26):
/// "a vertical, rectangular frame of obsidian" (crying obsidian is the
/// 1.16 respawn-anchor ingredient, not a frame material).
#[inline]
fn frame_cell(b: u16) -> bool {
    b == OBSIDIAN
}

/// the vanilla frame detection: any rectangle 2 wide × 3 tall interior
/// (up to 21×21) inside an obsidian frame outline, found from one
/// interior cell. Both axes are tried (X first — the order is
/// deterministic). Returns None when the cell sits in no valid frame.
pub fn find_frame(world: &World, x: i32, y: i32, z: i32) -> Option<PortalFrame> {
    if !interior_cell(world.get_block(x, y, z)) {
        return None;
    }
    find_frame_axis(world, x, y, z, PortalAxis::X)
        .or_else(|| find_frame_axis(world, x, y, z, PortalAxis::Z))
}

/// the one-axis frame detection (the vanilla shape scan): descend to the
/// interior bottom, slide to the left edge, measure the width, measure
/// the height per column, and verify the obsidian outline. Corners are
/// NOT required (VERIFIED — see MIN_INTERIOR_W's citation).
fn find_frame_axis(world: &World, x: i32, y: i32, z: i32, axis: PortalAxis) -> Option<PortalFrame> {
    let (sx, sz) = axis.step();
    // 1. descend to the interior's bottom row (the cell below it is the
    //    frame's base) — the walk caps at MAX_INTERIOR_H + 1: a deeper
    //    run means the interior exceeds the vanilla 21-tall ceiling and
    //    the frame is invalid by construction
    let mut by = y;
    let mut steps = 0u32;
    while by > 0 && interior_cell(world.get_block(x, by - 1, z)) {
        by -= 1;
        steps += 1;
        if steps > MAX_INTERIOR_H + 1 {
            return None;
        }
    }
    if by == 0 || !frame_cell(world.get_block(x, by - 1, z)) {
        return None;
    }
    // 2. slide left along the bottom row to the frame's left edge
    let mut lx = x;
    let mut lz = z;
    let mut wsteps = 0u32;
    while interior_cell(world.get_block(lx - sx, by, lz - sz)) {
        lx -= sx;
        lz -= sz;
        wsteps += 1;
        if wsteps > MAX_INTERIOR_W + 1 {
            return None;
        }
    }
    if !frame_cell(world.get_block(lx - sx, by, lz - sz)) {
        return None;
    }
    // 3. measure the interior width along the bottom row; the anchor
    //    column counts as 1 (the interior cells are lx + sx·i for
    //    i in 0..w); the stopping cell is the frame's right edge
    let mut w = 1u32;
    loop {
        let c = world.get_block(lx + sx * w as i32, by, lz + sz * w as i32);
        if interior_cell(c) {
            w += 1;
            if w > MAX_INTERIOR_W {
                return None;
            }
        } else if frame_cell(c) {
            break;
        } else {
            return None; // the frame edge is neither obsidian nor interior
        }
    }
    if w < MIN_INTERIOR_W {
        return None;
    }
    // 4. measure the height of every column; the interior is a
    //    rectangle (one shared height), the cap cell is the frame's top.
    //    The bottom row counts as 1 (the width scan verified it interior)
    let mut h: Option<u32> = None;
    for i in 0..w as i32 {
        let cx = lx + sx * i;
        let cz = lz + sz * i;
        let mut col_h = 1u32;
        loop {
            let c = world.get_block(cx, by + col_h as i32, cz);
            if interior_cell(c) {
                col_h += 1;
                if col_h > MAX_INTERIOR_H {
                    return None;
                }
            } else if frame_cell(c) {
                break;
            } else {
                return None;
            }
        }
        if col_h < MIN_INTERIOR_H {
            return None;
        }
        match h {
            None => h = Some(col_h),
            Some(prev) if prev == col_h => {}
            Some(_) => return None, // ragged columns — not a rectangle
        }
    }
    let h = h?;
    // 5. verify the obsidian outline: the base and top rows across the
    //    full width, and the side columns across the full height (the
    //    four corners are not required)
    for i in 0..w as i32 {
        let cx = lx + sx * i;
        let cz = lz + sz * i;
        if !frame_cell(world.get_block(cx, by - 1, cz))
            || !frame_cell(world.get_block(cx, by + h as i32, cz))
        {
            return None;
        }
    }
    for k in 0..h as i32 {
        if !frame_cell(world.get_block(lx - sx, by + k, lz - sz))
            || !frame_cell(world.get_block(lx + sx * w as i32, by + k, lz + sz * w as i32))
        {
            return None;
        }
    }
    Some(PortalFrame {
        axis,
        min: [lx, by, lz],
        max: [
            lx + sx * (w as i32 - 1),
            by + h as i32 - 1,
            lz + sz * (w as i32 - 1),
        ],
    })
}

/// the portal search on arrival (VERIFIED Nether_portal§Portal_search,
/// live 2026-09-26): "Starting at these destination coordinates, the
/// game looks for all nearby portal points of interest... the game
/// selects the closest one as determined by its distance in the new
/// coordinate system (including the Y coordinate...)" — the distance is
/// Euclidean, "the shortest path is chosen, counting the Y difference".
/// The engine scans the GENERATED chunks within the radius (the
/// disclosed adaptation: vanilla load-ticks a 17×17-chunk area first —
/// ours searches the loaded set; the caller force-generates the
/// destination chunk). Returns the closest portal block's cell + its
/// squared distance.
///
/// Determinism: the chunk scan walks a SORTED chunk list (the FxHashMap
/// iteration order is nondeterministic) in a fixed y/z/x order with the
/// strict-`<` closest tracking — the same world state always selects
/// the same portal.
pub fn search_existing_portal(
    world: &World,
    cx: i32,
    cz: i32,
    y: i32,
    radius: i32,
) -> Option<([i32; 3], f32)> {
    let r_chunks = radius.div_euclid(16) + 1;
    let mut chunks: Vec<(i32, i32)> = world
        .chunks
        .keys()
        .copied()
        .filter(|&(kx, kz)| {
            (kx * 16 - cx).abs() <= radius + 16 && (kz * 16 - cz).abs() <= radius + 16
        })
        .collect();
    chunks.sort_unstable();
    let mut best: Option<([i32; 3], f32)> = None;
    for (kx, kz) in chunks {
        if (kx - cx.div_euclid(16)).abs() > r_chunks || (kz - cz.div_euclid(16)).abs() > r_chunks {
            continue;
        }
        let Some(chunk) = world.chunks.get(&(kx, kz)) else {
            continue;
        };
        for ly in 0..256usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    if state_block(chunk.get(lx, ly, lz)) != NETHER_PORTAL {
                        continue;
                    }
                    let wx = kx * 16 + lx as i32;
                    let wz = kz * 16 + lz as i32;
                    let dx = (wx - cx) as f32;
                    let dy = (ly as i32 - y) as f32;
                    let dz = (wz - cz) as f32;
                    let d = dx * dx + dy * dy + dz * dz;
                    if best.map_or(true, |(_, bd)| d < bd) {
                        best = Some(([wx, ly as i32, wz], d));
                    }
                }
            }
        }
    }
    best
}

/// the blocks a new portal may overwrite (VERIFIED
/// Nether_portal§Portal_creation, live 2026-09-26: "A valid location
/// consists of 3×4 buildable blocks... with 4 blocks of air above all
/// 12 blocks" — the engine's buildable set is air + the replaceable
/// plants, the vc-world `replaceable` predicate).
#[inline]
fn buildable(b: u16) -> bool {
    matches!(b, AIR | TALL_GRASS | FLOWER_RED | FLOWER_YELLOW)
}

/// the new-portal placement scan (VERIFIED Nether_portal§Portal_creation,
/// live 2026-09-26): pass 1 finds "3×4 buildable blocks with 4 blocks of
/// air above all 12 blocks, with the long axis matching the long axis of
/// the source portal"; the fallback (pass 2) "a 1×4 expanse of buildable
/// blocks with air 4 high above each". "The closest valid position in
/// the 3D distance is always picked". The scan walks the ±BUILD_HORIZON
/// square in deterministic ring order (nearest first, ties broken by the
/// fixed scan order) and only over generated chunks — unloaded columns
/// are skipped (the disclosed adaptation of vanilla's loaded-area scan).
///
/// Returns the interior-bottom anchor (the 2-wide × 3-tall interior's
/// bottom cell): pass 1 reserves the buildable bottom row for the
/// frame's base, so the interior starts one above; pass 2's expanse IS
/// the interior column, so the interior starts at its first buildable
/// cell (the frame's base overwrites whatever sits below).
pub fn find_build_spot(
    world: &World,
    dx: i32,
    dy: i32,
    dz: i32,
    axis: PortalAxis,
) -> Option<([i32; 3], bool)> {
    // pass 1 over the whole horizon; the fallback (pass 2) is redone
    // only when the first check "fails entirely" (VERIFIED — see the
    // doc comment above)
    scan_build_pass(world, dx, dy, dz, axis, false)
        .map(|a| (a, false))
        .or_else(|| scan_build_pass(world, dx, dy, dz, axis, true).map(|a| (a, true)))
}

/// one build-spot scan pass (the wiki's first check = 3×4 buildable +
/// 4 air above; the fallback pass = 1×4 + air). "The closest valid
/// position in the 3D distance is always picked" — the scan walks the
/// ±BUILD_HORIZON square in deterministic ring order (nearest first,
/// ties broken by the fixed scan order) and only over generated chunks.
fn scan_build_pass(
    world: &World,
    dx: i32,
    dy: i32,
    dz: i32,
    axis: PortalAxis,
    pass2: bool,
) -> Option<[i32; 3]> {
    let (sx, sz) = axis.step();
    let mut best: Option<([i32; 3], i64)> = None;
    // ring order from the destination column: r = 0..=BUILD_HORIZON,
    // each ring walked in a fixed (dz, dx) order
    for r in 0..=BUILD_HORIZON {
        for oz in -r..=r {
            for ox in -r..=r {
                if r > 0 && ox.abs() != r && oz.abs() != r {
                    continue; // walk the ring, not the disc
                }
                let wx = dx + ox;
                let wz = dz + oz;
                if world
                    .chunks
                    .get(&(wx.div_euclid(16), wz.div_euclid(16)))
                    .is_none()
                {
                    continue; // not generated — skip (the loaded-area scan)
                }
                for y in 1..=248i32 {
                    let fits = if pass2 {
                        // pass 2: a 1-wide × 4-tall buildable expanse
                        // with air 4 high above each
                        buildable(world.get_block(wx, y, wz))
                            && buildable(world.get_block(wx, y + 1, wz))
                            && buildable(world.get_block(wx, y + 2, wz))
                            && buildable(world.get_block(wx, y + 3, wz))
                            && buildable(world.get_block(wx, y + 4, wz))
                            && buildable(world.get_block(wx, y + 5, wz))
                            && buildable(world.get_block(wx, y + 6, wz))
                            && buildable(world.get_block(wx, y + 7, wz))
                    } else {
                        // pass 1: a 3-wide × 4-tall buildable rectangle
                        // with 4 blocks of air above all 12
                        (0..4i32).all(|k| {
                            (0..3i32).all(|i| {
                                buildable(world.get_block(wx + sx * i, y + k, wz + sz * i))
                            })
                        }) && (0..4i32).all(|k| {
                            (0..3i32).all(|i| {
                                buildable(world.get_block(wx + sx * i, y + 4 + k, wz + sz * i))
                            })
                        })
                    };
                    if !fits {
                        continue;
                    }
                    // the interior-bottom anchor: pass 1 reserves the
                    // buildable bottom row for the frame's base, so the
                    // interior starts one above; pass 2's expanse IS the
                    // interior column
                    let anchor = [wx, if pass2 { y } else { y + 1 }, wz];
                    let ddx = (anchor[0] - dx) as i64;
                    let ddy = (anchor[1] - dy) as i64;
                    let ddz = (anchor[2] - dz) as i64;
                    let d = ddx * ddx + ddy * ddy + ddz * ddz;
                    if best.map_or(true, |(_, bd)| d < bd) {
                        best = Some((anchor, d));
                    }
                }
            }
        }
    }
    best.map(|(anchor, _)| anchor)
}

/// the forced-portal Y clamp (VERIFIED Nether_portal§Portal_creation,
/// live 2026-09-26: "with Y constrained to be between 70 and 10 less
/// than the world height" — 118 for the Nether; the Overworld ceiling
/// is the engine's 256 − 10, the disclosed adaptation).
#[inline]
pub fn forced_y(y: i32, nether: bool) -> i32 {
    let max = if nether {
        FORCED_Y_NETHER_MAX
    } else {
        FORCED_Y_OVERWORLD_MAX
    };
    y.clamp(FORCED_Y_MIN, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_blocks::blocks::STONE;

    /// a stone-filled chunk at (0, 0) — the loaded-chunk pattern (World
    /// edits no-op on missing chunks; the campfire-test pattern)
    fn stone_world(seed: u64) -> World {
        let mut w = World::new(seed);
        let mut c = vc_chunk::chunk::Chunk::empty();
        for y in 0..=80usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    c.set(lx, y, lz, STONE);
                }
            }
        }
        w.insert_generated((0, 0), Arc::new(c), Vec::new());
        w.dirty.clear();
        w
    }

    /// an obsidian frame around a `w_int` × `h_int` interior (the frame
    /// spans the interior plus both side columns and both caps — 4×5
    /// including the corners for the 2×3 minimum)
    fn build_frame(w: &mut World, anchor: [i32; 3], axis: PortalAxis, w_int: u32, h_int: u32) {
        let (sx, sz) = axis.step();
        let (wi, hi) = (w_int as i32, h_int as i32);
        for i in -1..=wi {
            for k in -1..=hi {
                let interior = i >= 0 && i < wi && k >= 0 && k < hi;
                let b = if interior { AIR } else { OBSIDIAN };
                let _ = w.set_block_state(anchor[0] + sx * i, anchor[1] + k, anchor[2] + sz * i, b);
            }
        }
    }

    /// VERIFIED Nether_portal§Creation (live 2026-09-26): "a vertical,
    /// rectangular frame of obsidian (4×5 minimum, 23×23 maximum)" — the
    /// interior is 2 wide × 3 tall minimum; "The four corners of the
    /// frame are not required".
    #[test]
    fn valid_frame_minimum_2x3_interior() {
        let mut w = stone_world(1);
        let anchor = [4, 65, 4];
        build_frame(&mut w, anchor, PortalAxis::X, 2, 3);
        let f = find_frame(&w, 4, 66, 4).expect("the 2×3 interior must validate");
        assert_eq!(f.axis, PortalAxis::X);
        assert_eq!(f.width(), 2);
        assert_eq!(f.height(), 3);
        assert_eq!(f.min, [4, 65, 4]);
        assert_eq!(f.max, [5, 67, 4]);
        // detection from any interior cell (the bottom-right corner here)
        assert!(
            find_frame(&w, 5, 67, 4).is_some(),
            "any interior cell validates"
        );
        // the Z axis mirrors (the frame spans Z×Y)
        let mut w2 = stone_world(2);
        build_frame(&mut w2, [4, 65, 4], PortalAxis::Z, 2, 3);
        let f2 = find_frame(&w2, 4, 66, 5).expect("the Z-axis frame must validate");
        assert_eq!(f2.axis, PortalAxis::Z);
        assert_eq!(f2.width(), 2);
        assert_eq!(f2.height(), 3);
    }

    /// the invalid shapes: an outline gap, a non-obsidian frame, a
    /// 1-wide interior, ragged columns, and an interior beyond the
    /// 21-tall ceiling — all rejected (VERIFIED Nether_portal§Creation:
    /// the frame material is obsidian, the 4×5 minimum / 23×23 maximum).
    #[test]
    fn invalid_frames_rejected() {
        // (a) a gap in the frame outline — the detection requires the
        // full obsidian outline (the corners excepted)
        let mut w = stone_world(3);
        build_frame(&mut w, [4, 65, 4], PortalAxis::X, 2, 3);
        let _ = w.set_block_state(6, 66, 4, AIR); // a side-column gap
        assert!(
            find_frame(&w, 4, 66, 4).is_none(),
            "an outline gap must reject"
        );
        // (b) a stone frame — the frame material is OBSIDIAN strictly
        // ("a vertical, rectangular frame of obsidian", VERIFIED)
        let mut w2 = stone_world(4);
        for k in 0..3i32 {
            for i in 0..2i32 {
                let _ = w2.set_block_state(4 + i, 65 + k, 4, AIR); // carve the interior only
            }
        }
        assert!(
            find_frame(&w2, 4, 66, 4).is_none(),
            "a stone frame must reject"
        );
        // (c) a 1-wide interior — below the 2-wide minimum
        let mut w3 = stone_world(5);
        build_frame(&mut w3, [4, 65, 4], PortalAxis::X, 1, 3);
        assert!(
            find_frame(&w3, 4, 66, 4).is_none(),
            "a 1-wide interior must reject"
        );
        // (d) ragged columns — the interior is a rectangle
        let mut w4 = stone_world(6);
        build_frame(&mut w4, [4, 65, 4], PortalAxis::X, 2, 3);
        let _ = w4.set_block_state(5, 67, 4, OBSIDIAN); // one column shorter
        assert!(
            find_frame(&w4, 4, 66, 4).is_none(),
            "ragged columns must reject"
        );
        // (e) a 22-tall interior — beyond the 21-tall ceiling (the
        // 23×23 maximum frame, VERIFIED §Creation)
        let mut w5 = stone_world(7);
        build_frame(&mut w5, [4, 65, 4], PortalAxis::X, 2, 22);
        assert!(
            find_frame(&w5, 4, 66, 4).is_none(),
            "a 22-tall interior must reject"
        );
    }

    /// VERIFIED Nether_portal§Behavior (live 2026-09-26): "stands in a
    /// Nether portal block for 80 game ticks (4 seconds) in survival
    /// mode or 1 game tick (1⁄20 second) in creative mode" — 80 ticks at
    /// the 20 Hz simulation = 4.0 s.
    #[test]
    fn walk_in_waits_80_game_ticks_survival() {
        assert_eq!(TRAVEL_TICKS_SURVIVAL, 80);
        assert_eq!(TRAVEL_TICKS_CREATIVE, 1);
        let wait = travel_wait_secs(false);
        assert!((wait - 4.0).abs() < f32::EPSILON, "80 ticks = 4.0 s");
        assert!(
            (travel_wait_secs(true) - 1.0 / 20.0).abs() < f32::EPSILON,
            "creative = 1 tick"
        );
    }

    /// VERIFIED Nether_portal§Coordinate_conversion (live 2026-09-26):
    /// "for a given location (X, Y, Z) in the Overworld, the
    /// corresponding coordinates in the Nether are (floor(X ÷ 8), Y,
    /// floor(Z ÷ 8)), and conversely... the matching Overworld
    /// coordinates are (X × 8, Y, Z × 8)"; "The Java floor() method...
    /// rounds down to the largest integer less than or equal to the
    /// argument (toward smaller positive values and toward larger
    /// negative values)" — floor division, NOT truncation.
    #[test]
    fn arrival_coordinates_scale_8_to_1() {
        assert_eq!(portal_coords(false, 1600, 16), (200, 2));
        assert_eq!(portal_coords(false, -29, 29), (-4, 3)); // floor(−29 ÷ 8) = −4, NOT −3
        assert_eq!(portal_coords(true, 200, 2), (1600, 16));
        assert_eq!(portal_coords(true, -1, -1), (-8, -8));
    }

    /// VERIFIED Nether_portal§Portal_search (live 2026-09-26): "the game
    /// selects the closest one as determined by its distance in the new
    /// coordinate system (including the Y coordinate...)" — Euclidean;
    /// no portal in range → none (the build path follows).
    #[test]
    fn portal_search_finds_closest_within_radius() {
        let mut w = stone_world(8);
        build_frame(&mut w, [4, 65, 4], PortalAxis::X, 2, 3);
        build_frame(&mut w, [12, 65, 12], PortalAxis::X, 2, 3);
        let (cell, d) =
            search_existing_portal(&w, 5, 5, 65, 128).expect("a portal within range must be found");
        assert_eq!(cell, [5, 65, 4], "the closest portal receives the player");
        assert!(
            d < 98.0,
            "the far portal at distance² 98 loses to the close one"
        );
        // no portal in range → None (a new portal is built instead)
        assert!(search_existing_portal(&w, 200, 5, 65, 16).is_none());
    }

    /// VERIFIED Nether_portal§Portal_creation (live 2026-09-26): "within
    /// 16 blocks horizontally (but any distance vertically) of the
    /// incoming entity's destination coordinates. A valid location
    /// consists of 3×4 buildable blocks with 4 blocks of air above all
    /// 12 blocks"; a solid world finds nothing → None (the forced build
    /// at the target coordinates follows).
    #[test]
    fn build_spot_found_within_horizon() {
        let mut w = stone_world(9);
        // an air pocket in the pass-1 shape: 3 wide × 4 tall + 4 air above
        for k in 0..8i32 {
            for i in 0..3i32 {
                let _ = w.set_block_state(6 + i, 65 + k, 4, AIR);
            }
        }
        let (anchor, pass2) =
            find_build_spot(&w, 5, 65, 5, PortalAxis::X).expect("the pocket must validate");
        assert!(!pass2, "the 3×4 pocket is a pass-1 spot");
        assert_eq!(anchor, [6, 66, 4]);
        // a solid world: no buildable spot → the forced build
        let w2 = stone_world(10);
        assert!(find_build_spot(&w2, 5, 65, 5, PortalAxis::X).is_none());
    }

    /// VERIFIED Nether_portal§Portal_creation (live 2026-09-26): "with Y
    /// constrained to be between 70 and 10 less than the world height
    /// (i.e. 118 for the Nether or 310 for the Overworld)" — the
    /// Overworld ceiling is the engine's 256 − 10 (the disclosed
    /// adaptation; the engine's world height is 256).
    #[test]
    fn forced_y_clamped_to_world_height() {
        assert_eq!(forced_y(65, true), 70, "the floor at 70");
        assert_eq!(forced_y(90, true), 90);
        assert_eq!(forced_y(200, true), 118, "the Nether ceiling");
        assert_eq!(forced_y(300, false), 246, "the engine's 256 − 10 ceiling");
    }
}
