//! World: chunk map, block access, copy-on-write edits (thread-safe meshing),
//! pending cross-chunk decoration edits.

use crate::blocks::*;
use crate::chunk::Chunk;
use crate::gen::TerrainGen;
use crate::rng::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub type ChunkPos = (i32, i32);

/// Block ids that may be overwritten by decorations.
#[inline]
fn replaceable(cur: u8) -> bool {
    cur == AIR || cur == TALL_GRASS || cur == FLOWER_RED || cur == FLOWER_YELLOW
}

/// §12 dirty-cause bits (tracked separately per dirty chunk)
pub const CAUSE_GEOMETRY: u8 = 1;
pub const CAUSE_LIGHT: u8 = 2;
pub const CAUSE_MATERIAL: u8 = 4;
pub const CAUSE_TRANSPARENCY: u8 = 8;
pub const CAUSE_VISIBILITY: u8 = 16;

/// light propagation budget — the radius a block-light/sky-light change can
/// reach through air (vanilla 15 levels, one per BFS step)
const LIGHT_RADIUS: i32 = 15;

/// section bitset covering the y range [ylo, yhi] (clamped to 0..255)
#[inline]
fn sections_spanning(ylo: i32, yhi: i32) -> u16 {
    let lo = (ylo.max(0) / 16) as usize;
    let hi = (yhi.min(255).max(0) / 16) as usize;
    let mut m = 0u16;
    for s in lo..=hi.min(15) {
        m |= 1 << s;
    }
    m
}

/// material class for §12 cause accounting: cutout/transparent vs solid
#[inline]
fn is_transparent_class(b: u8) -> bool {
    !is_opaque(b) || b == WATER
}

pub struct World {
    pub seed: u64,
    pub gen: TerrainGen,
    pub chunks: HashMap<ChunkPos, Arc<Chunk>>,
    /// chunks fully generated + decorated (meshable)
    pub decorated: HashSet<ChunkPos>,
    /// edits queued for not-yet-generated chunks: (block_idx, id)
    pub pending: HashMap<ChunkPos, Vec<(u16, u8)>>,
    /// sections (bit s = 16-block section s) whose mesh is stale (§12:
    /// fine-grained — a block edit rebuilds sections, not the whole chunk)
    pub dirty: HashMap<ChunkPos, u16>,
    /// accumulated §12 causes for the dirty sections of each chunk
    pub dirty_causes: HashMap<ChunkPos, u8>,
    /// chunks with unsaved content (player edits + newly generated;
    /// drained by the native autosave — §28)
    pub save_dirty: HashSet<ChunkPos>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        World {
            seed,
            gen: TerrainGen::new(seed),
            chunks: HashMap::new(),
            decorated: HashSet::new(),
            pending: HashMap::new(),
            dirty: HashMap::new(),
            dirty_causes: HashMap::new(),
            save_dirty: HashSet::new(),
        }
    }

    /// Mark every section of a chunk dirty (streaming / full invalidation).
    #[inline]
    pub fn mark_all_dirty(&mut self, pos: ChunkPos, cause: u8) {
        self.dirty.insert(pos, u16::MAX);
        *self.dirty_causes.entry(pos).or_insert(0) |= cause;
    }

    /// Mark specific sections (bitset, bit s = section s) dirty.
    #[inline]
    pub fn mark_sections_dirty(&mut self, pos: ChunkPos, mask: u16, cause: u8) {
        if mask == 0 {
            return;
        }
        *self.dirty.entry(pos).or_insert(0) |= mask;
        *self.dirty_causes.entry(pos).or_insert(0) |= cause;
    }

    /// Clear the dirty bits a remesh job covered. Bits added while the job
    /// was in flight (edits that landed after its snapshot) survive — the
    /// chunk re-queues for another pass instead of staying stale.
    #[inline]
    pub fn clear_dirty_mask(&mut self, pos: ChunkPos, mask: u16) {
        if let Some(m) = self.dirty.get_mut(&pos) {
            *m &= !mask;
            if *m == 0 {
                self.dirty.remove(&pos);
                self.dirty_causes.remove(&pos);
            }
        }
    }

    /// Dirty-section count (F3 / __vcStats evidence for §12).
    pub fn dirty_section_count(&self) -> usize {
        self.dirty.values().map(|m| m.count_ones() as usize).sum()
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Arc<Chunk>> {
        self.chunks.get(&pos)
    }

    #[inline]
    pub fn get_block(&self, wx: i32, wy: i32, wz: i32) -> u8 {
        if wy < 0 || wy > 255 {
            return AIR;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        match self.chunks.get(&(cx, cz)) {
            Some(c) => {
                let lx = (wx - cx * 16) as usize;
                let lz = (wz - cz * 16) as usize;
                state_block(c.get(lx, wy as usize, lz) as u16)
            }
            None => AIR,
        }
    }

    /// raw state id at a position (property variants included)
    #[inline]
    pub fn get_state(&self, wx: i32, wy: i32, wz: i32) -> u16 {
        if wy < 0 || wy > 255 {
            return 0;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        match self.chunks.get(&(cx, cz)) {
            Some(c) => {
                let lx = (wx - cx * 16) as usize;
                let lz = (wz - cz * 16) as usize;
                c.get(lx, wy as usize, lz) as u16
            }
            None => 0,
        }
    }

    /// True if the block position sits inside a loaded chunk.
    pub fn is_loaded(&self, wx: i32, wz: i32) -> bool {
        self.chunks
            .contains_key(&(wx.div_euclid(16), wz.div_euclid(16)))
    }

    /// Player-driven block edit (copy-on-write so in-flight mesh jobs with old
    /// snapshots stay consistent). Marks affected chunks dirty for re-mesh.
    /// `id` is a BLOCK id — the default state of that block is stored.
    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, id: u8) {
        self.set_block_state(wx, wy, wz, id as u16);
    }

    /// Player-driven BLOCK-STATE edit (e.g. a log placed with axis=x).
    /// Copy-on-write at section granularity; marks the affected SECTIONS
    /// dirty (§12): geometry near the edit, light within the propagation
    /// radius (block light ±15, sky light down the column).
    pub fn set_block_state(&mut self, wx: i32, wy: i32, wz: i32, state: u16) {
        if wy < 0 || wy > 255 {
            return;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let pos = (cx, cz);
        let Some(old) = self.chunks.get(&pos) else { return };
        let old = Arc::clone(old);
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let old_state = old.get(lx, wy as usize, lz) as u16;
        let mut new_chunk = (*old).clone();
        new_chunk.set_state(lx, wy as usize, lz, state);
        self.chunks.insert(pos, Arc::new(new_chunk));

        self.save_dirty.insert(pos); // persist player edits (§28)
        self.mark_edit(wx, wy, wz, old_state, state);
    }

    /// §12 invalidation region for one block edit.
    ///
    /// * geometry — the edit's section plus boundary-adjacent sections in
    ///   this and neighboring chunks (face culling + AO read ±1 cells)
    /// * light — block light within ±15 of the edit; sky light additionally
    ///   down the whole column below it (attenuation without decrement)
    ///
    /// The region is conservative (light is recomputed per remesh anyway);
    /// correctness never depends on it being tight, only on covering the
    /// true change.
    fn mark_edit(&mut self, wx: i32, wy: i32, wz: i32, old_state: u16, new_state: u16) {
        let old_b = state_block(old_state);
        let new_b = state_block(new_state);
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let lx = (wx - cx * 16) as i32;
        let lz = (wz - cz * 16) as i32;
        let sy = (wy / 16) as usize;
        let own_sec = 1u16 << sy;

        // ---- cause bits (§12: tracked separately per dirty chunk)
        let mut cause = CAUSE_GEOMETRY;
        if is_transparent_class(old_b) != is_transparent_class(new_b) {
            cause |= CAUSE_MATERIAL;
        }
        if old_b == WATER || new_b == WATER {
            cause |= CAUSE_TRANSPARENCY;
        }

        // ---- geometry: own + boundary-adjacent sections (±1 in x/y/z,
        //      crossing both section and chunk borders for culling/AO)
        let mut gmask_self = own_sec;
        if wy % 16 == 0 && sy > 0 {
            gmask_self |= 1 << (sy - 1);
        }
        if wy % 16 == 15 && sy < 15 {
            gmask_self |= 1 << (sy + 1);
        }
        self.mark_sections_dirty((cx, cz), gmask_self, cause);
        // neighbor chunks: only when the edit touches a chunk border; the
        // affected band is this section ± one (AO corners reach diagonally)
        let mut neighbor_mask = own_sec;
        if wy % 16 == 0 && sy > 0 {
            neighbor_mask |= 1 << (sy - 1);
        }
        if wy % 16 == 15 && sy < 15 {
            neighbor_mask |= 1 << (sy + 1);
        }
        let mut touch = |dx: i32, dz: i32| {
            self.mark_sections_dirty((cx + dx, cz + dz), neighbor_mask, cause);
        };
        if lx == 0 {
            touch(-1, 0);
        }
        if lx == 15 {
            touch(1, 0);
        }
        if lz == 0 {
            touch(0, -1);
        }
        if lz == 15 {
            touch(0, 1);
        }
        // corners: diagonal neighbors read the edit cell for AO
        if lx == 0 && lz == 0 {
            touch(-1, -1);
        }
        if lx == 0 && lz == 15 {
            touch(-1, 1);
        }
        if lx == 15 && lz == 0 {
            touch(1, -1);
        }
        if lx == 15 && lz == 15 {
            touch(1, 1);
        }

        // ---- light region (§12: mark only RELEVANT lighting regions)
        //
        // Block light changes require a source: the edit's emissivity, or
        // an emissive block inside the ±15 box (light reach is 15).
        // Sky light changes require the sky: an exposed column at the edit
        // (shadow / lit segment + lateral spread into cover), or — when
        // mining under cover — a nearby exposed column. Dark sealed caves
        // with no emissive sources get NO light region at all.
        let darkens = |b: u8| is_opaque(b) || b == WATER || b == LEAVES;
        let new_d = darkens(new_b);
        let old_d = darkens(old_b);
        let emissive_changed = emissive(old_b) != emissive(new_b);
        let light_changed = emissive_changed || new_d != old_d;
        if !light_changed {
            return;
        }
        let cause_l = cause | CAUSE_LIGHT;

        // (1) block light — ±15 box around the edit
        let bl_mask = if emissive_changed || self.any_emissive_in_box(wx, wy, wz) {
            sections_spanning(wy - LIGHT_RADIUS, wy + LIGHT_RADIUS)
        } else {
            0
        };

        // (2) sky light — the edit's column (shadow / lit segment) plus
        //     lateral spread when nearby cover can hold sub-15 light
        let mut sky_own = 0u16;
        let mut sky_box = 0u16;
        if new_d != old_d {
            let exposed = self.column_exposed(wx, wy, wz);
            let floor = self.first_opaque_below(wx, wy, wz);
            // the lit segment [floor+1, wy] matters only when it spans MORE
            // than the edit cell itself (floor == wy-1 → nothing below to
            // shadow; geometry already covers the edit section)
            if exposed && floor < wy - 1 {
                // the segment darkens (place) or lights (mine)
                sky_own = sections_spanning((floor + 1).max(0), wy);
            }
            let scan_lo = ((floor + 1).max(wy - LIGHT_RADIUS)).max(0);
            let scan_hi = (wy + LIGHT_RADIUS).min(255);
            if exposed {
                // entering/leaving light spreads into nearby cover
                if self.any_cover_in_box(wx, wz, scan_lo, scan_hi) {
                    sky_box = sections_spanning(scan_lo, scan_hi);
                }
            } else if old_d {
                // mining under cover: light can only appear via a nearby
                // exposed column (opaque-only walk — water/leaves pass dimmed sky)
                if self.any_exposed_in_box(wx, wz, scan_lo, scan_hi) {
                    sky_box = sections_spanning(scan_lo, scan_hi);
                }
            } else {
                // placing under cover: existing lateral light routes through
                // the cell — cover present means sub-15 light could be nearby
                if self.any_cover_in_box(wx, wz, scan_lo, scan_hi) {
                    sky_box = sections_spanning(scan_lo, scan_hi);
                }
            }
        }

        if sky_own != 0 {
            self.mark_sections_dirty((cx, cz), sky_own, cause_l);
        }
        let box_mask = sky_box | bl_mask;
        if box_mask != 0 {
            let x0 = wx - LIGHT_RADIUS;
            let x1 = wx + LIGHT_RADIUS;
            let z0 = wz - LIGHT_RADIUS;
            let z1 = wz + LIGHT_RADIUS;
            for cxx in x0.div_euclid(16)..=x1.div_euclid(16) {
                for czz in z0.div_euclid(16)..=z1.div_euclid(16) {
                    self.mark_sections_dirty((cxx, czz), box_mask, cause_l);
                }
            }
        }
    }

    // ---------------- §12 light-region probes (all read-only, main thread) --

    /// is there any emissive block in the ±15 box around the edit?
    /// Palette-level probe — sections without emissive entries cost O(1).
    fn any_emissive_in_box(&self, wx: i32, wy: i32, wz: i32) -> bool {
        let y0 = (wy - LIGHT_RADIUS).max(0);
        let y1 = (wy + LIGHT_RADIUS).min(255);
        for cxx in (wx - LIGHT_RADIUS).div_euclid(16)..=(wx + LIGHT_RADIUS).div_euclid(16) {
            for czz in (wz - LIGHT_RADIUS).div_euclid(16)..=(wz + LIGHT_RADIUS).div_euclid(16) {
                let Some(c) = self.chunks.get(&(cxx, czz)) else { continue };
                for sy in (y0 / 16) as usize..=((y1 / 16) as usize).min(15) {
                    if let Some(sec) = &c.sections[sy] {
                        if sec.has_emissive() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// is the edit column exposed to the sky at the edit height? (no opaque
    /// block above). Walks sections top-down: empty sections pass for free,
    /// all-opaque sections terminate immediately.
    fn column_exposed(&self, wx: i32, wy: i32, wz: i32) -> bool {
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let Some(c) = self.chunks.get(&(cx, cz)) else { return false };
        let top = (wy + 1).min(255);
        for sy in ((wy / 16) as usize..16).rev() {
            if let Some(sec) = &c.sections[sy] {
                if sec.is_empty() {
                    continue;
                }
                if sec.all_opaque() {
                    return false;
                }
                // cells to check in this section: down to `top`, not lower
                let y_from = if sy == (wy / 16) as usize { top as usize } else { sy * 16 };
                for y in (y_from..=sy * 16 + 15).rev() {
                    if is_opaque(state_block(c.get(lx, y, lz) as u16)) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// y of the first opaque block strictly below wy in the edit column
    /// (-1 if none). Walks sections downward with all-opaque early exit.
    fn first_opaque_below(&self, wx: i32, wy: i32, wz: i32) -> i32 {
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let Some(c) = self.chunks.get(&(cx, cz)) else { return -1 };
        let bot = (wy - 1).max(0);
        for sy in (0..=(bot / 16) as usize).rev() {
            if let Some(sec) = &c.sections[sy] {
                if sec.is_empty() {
                    continue;
                }
                if sec.all_opaque() {
                    return (sy * 16 + 15) as i32;
                }
                // cells to check in this section: from `bot` down
                let y_top = if sy == (bot / 16) as usize { bot as usize } else { sy * 16 + 15 };
                for y in (sy * 16..=y_top).rev() {
                    if is_opaque(state_block(c.get(lx, y, lz) as u16)) {
                        return y as i32;
                    }
                }
            }
        }
        -1
    }

    /// any NON-AIR block in the ±15 xz box within the y band (the edit's own
    /// column excluded) — "cover": cells there may hold light below full sky
    /// that derives from the lit segment at the edit.
    fn any_cover_in_box(&self, wx: i32, wz: i32, y_lo: i32, y_hi: i32) -> bool {
        for cxx in (wx - LIGHT_RADIUS).div_euclid(16)..=(wx + LIGHT_RADIUS).div_euclid(16) {
            for czz in (wz - LIGHT_RADIUS).div_euclid(16)..=(wz + LIGHT_RADIUS).div_euclid(16) {
                let Some(c) = self.chunks.get(&(cxx, czz)) else { continue };
                for sy in (y_lo / 16) as usize..=((y_hi / 16) as usize).min(15) {
                    let Some(sec) = &c.sections[sy] else { continue };
                    if sec.is_empty() {
                        continue;
                    }
                    let yl = (sy * 16).max(y_lo as usize);
                    let yh = (sy * 16 + 15).min(y_hi as usize);
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            // world coords of this column (skip the edit column)
                            let wx_l = cxx * 16 + lx as i32;
                            let wz_l = czz * 16 + lz as i32;
                            if wx_l == wx && wz_l == wz {
                                continue;
                            }
                            for y in yl..=yh {
                                if c.get(lx, y, lz) != AIR {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// any column in the ±15 xz box exposed to the sky at band level?
    /// (highest opaque block strictly below the band top). Empty sections
    /// pass free, all-opaque sections terminate the column walk instantly.
    fn any_exposed_in_box(&self, wx: i32, wz: i32, y_lo: i32, y_hi: i32) -> bool {
        for cxx in (wx - LIGHT_RADIUS).div_euclid(16)..=(wx + LIGHT_RADIUS).div_euclid(16) {
            for czz in (wz - LIGHT_RADIUS).div_euclid(16)..=(wz + LIGHT_RADIUS).div_euclid(16) {
                let Some(c) = self.chunks.get(&(cxx, czz)) else { continue };
                for lz in 0..16usize {
                    'col: for lx in 0..16usize {
                        // walk this column from the sky down to the band top
                        for sy in (((y_hi / 16) as usize)..16).rev() {
                            if let Some(sec) = &c.sections[sy] {
                                if sec.is_empty() {
                                    continue;
                                }
                                if sec.all_opaque() {
                                    continue 'col; // blocked above the band
                                }
                                // cells to check in this section: down to y_hi
                                let y_from = if sy == (y_hi / 16) as usize {
                                    y_hi as usize
                                } else {
                                    sy * 16
                                };
                                for y in (y_from..=sy * 16 + 15).rev() {
                                    if is_opaque(state_block(c.get(lx, y, lz) as u16)) {
                                        continue 'col;
                                    }
                                }
                            }
                        }
                        // no opaque above the band top → the band is sky-lit
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Apply a generation-time outbound edit (tree canopy crossing borders).
    pub fn apply_gen_edit(&mut self, wx: i32, wy: i32, wz: i32, id: u8) {
        if wy < 0 || wy > 255 {
            return;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let pos = (cx, cz);
        if let Some(old) = self.chunks.get(&pos) {
            let old = Arc::clone(old);
            let lx = (wx - cx * 16) as usize;
            let lz = (wz - cz * 16) as usize;
            let cur = old.get(lx, wy as usize, lz);
            let target_ok = replaceable(cur) || (cur == LEAVES && id == OAK_LOG);
            if !target_ok {
                return;
            }
            // only if this chunk is already decorated; otherwise fold into pending
            if self.decorated.contains(&pos) {
                let mut new_chunk = (*old).clone();
                new_chunk.set(lx, wy as usize, lz, id);
                self.chunks.insert(pos, Arc::new(new_chunk));
                let sy = (wy / 16) as usize;
                self.mark_sections_dirty(pos, 1 << sy, CAUSE_GEOMETRY);
            } else {
                let idx = crate::chunk::idx(lx, wy as usize, lz) as u16;
                self.pending.entry(pos).or_default().push((idx, id));
            }
        } else {
            let lx = (wx - cx * 16) as usize;
            let lz = (wz - cz * 16) as usize;
            let idx = crate::chunk::idx(lx, wy as usize, lz) as u16;
            self.pending.entry(pos).or_default().push((idx, id));
        }
    }

    /// Insert a freshly generated chunk + apply outbound decorations.
    /// Newly generated content has never been saved → save-dirty.
    pub fn insert_generated(&mut self, pos: ChunkPos, chunk: Arc<Chunk>, outbound: Vec<(i32, i32, i32, u8)>) {
        self.chunks.insert(pos, chunk);
        self.decorated.insert(pos);
        self.save_dirty.insert(pos);
        for (wx, wy, wz, id) in outbound {
            self.apply_gen_edit(wx, wy, wz, id);
        }
    }

    /// Take (drain) the pending inbound edits for a chunk being generated.
    pub fn take_pending(&mut self, pos: ChunkPos) -> Vec<(u16, u8)> {
        self.pending.remove(&pos).unwrap_or_default()
    }

    /// Snapshot the 3x3 neighborhood for a mesh job (all must be generated).
    pub fn snapshot3x3(&self, cx: i32, cz: i32) -> Option<[Option<Arc<Chunk>>; 9]> {
        let mut snap: [Option<Arc<Chunk>>; 9] = Default::default();
        let mut i = 0;
        for dz in -1..=1 {
            for dx in -1..=1 {
                let pos = (cx + dx, cz + dz);
                let c = self.chunks.get(&pos)?;
                snap[i] = Some(Arc::clone(c));
                i += 1;
            }
        }
        Some(snap)
    }

    /// All 3x3 neighbors decorated (meshable)?
    pub fn meshable(&self, cx: i32, cz: i32) -> bool {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if !self.decorated.contains(&(cx + dx, cz + dz)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_spawn(&self) -> (f32, f32, f32) {
        self.gen.find_spawn()
    }

    /// Random seed for this world.
    pub fn random_seed() -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        #[cfg(target_arch = "wasm32")]
        let nanos = (js_sys::Date::now() * 1_000_000.0) as u64;
        Rng::new(nanos).next_u64()
    }
}

#[cfg(test)]
mod phase3_tests {
    use super::*;

    /// one generated, inserted chunk at the origin
    fn world_with_chunk() -> World {
        let mut w = World::new(42);
        let (chunk, _) = w.gen.generate_chunk(0, 0, Vec::new());
        w.insert_generated((0, 0), chunk, Vec::new());
        w.dirty.clear();
        w.dirty_causes.clear();
        w
    }

    /// hand-built flat world: solid floor at y<70, open sky above —
    /// exact §12 region expectations without terrain randomness
    fn flat_world(floor_top: i32) -> World {
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
                for y in 0..=floor_top {
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
        w.dirty_causes.clear();
        w
    }

    /// §12: an edit ON the floor surface dirties only the geometry section —
    /// the column below is already opaque (dark), no cover, no emissive →
    /// zero light region. THE common-case single-section edit.
    #[test]
    fn floor_surface_edit_is_geometry_only() {
        let mut w = flat_world(70);
        w.set_block(8, 71, 8, STONE); // directly on the floor
        assert_eq!(w.dirty.len(), 1, "only the own chunk");
        let mask = *w.dirty.get(&(0, 0)).unwrap();
        assert_eq!(mask, 1 << 4, "section 4 (y 64..79) only, got {mask:#b}");
        assert_eq!(*w.dirty_causes.get(&(0, 0)).unwrap(), CAUSE_GEOMETRY | CAUSE_MATERIAL);
    }

    /// §12: placing a block ABOVE the floor (mid-air) shadows the open
    /// column below it — sections from the floor to the edit go dirty; still
    /// no lateral box (no cover above the floor within ±15).
    #[test]
    fn floor_edit_shadows_column() {
        let mut w = flat_world(70);
        w.set_block(8, 80, 8, STONE); // 10 above the floor, open sky
        let mask = *w.dirty.get(&(0, 0)).unwrap();
        // lit segment [71..80] → sections 4 (64..79) and 5 (80..95) ∪ geometry section 5
        assert_eq!(mask, (1 << 4) | (1 << 5), "shadow column sections, got {mask:#b}");
        assert_eq!(*w.dirty_causes.get(&(0, 0)).unwrap(), CAUSE_GEOMETRY | CAUSE_LIGHT | CAUSE_MATERIAL);
        assert_eq!(w.dirty.len(), 1, "no neighbor chunks — no cover, no emissive");
    }

    /// §12: glowstone in the open sky dirties the ±15 block-light box in
    /// every chunk the box touches (own + neighbors), band y±15.
    #[test]
    fn glowstone_box_reaches_neighbors() {
        let mut w = flat_world(70);
        w.set_block(8, 120, 8, GLOWSTONE);
        // emissive changed → box [105..135] = sections 6..8; sky shadow
        // [71..120] = sections 4..7 (own only)
        let own = *w.dirty.get(&(0, 0)).unwrap();
        assert_eq!(own, 0x1F0, "sections 4..8, got {own:#b}");
        for pos in [(1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
            let m = *w.dirty.get(&pos).unwrap_or(&0);
            assert_eq!(m, 0x1C0, "neighbor {pos:?} gets the light box sections 6..8, got {m:#b}");
        }
        assert_eq!(w.dirty.len(), 9, "±15 box spans the 3×3 chunks");
    }

    /// §12: a border edit reaches the adjacent chunk's matching sections
    /// (face culling + AO), including the diagonal corner chunk.
    #[test]
    fn border_edit_reaches_neighbors() {
        let mut w = flat_world(70);
        // corner position: lx=15, lz=15 → neighbors (+1,0), (0,+1), (+1,+1)
        w.set_block(15, 120, 15, STONE);
        for pos in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            assert!(w.dirty.contains_key(&pos), "chunk {pos:?} must be dirty");
        }
        assert!(!w.dirty.contains_key(&(-1, -1)), "far diagonal not touched");
        let m = *w.dirty.get(&(1, 0)).unwrap();
        assert_eq!(m & (1 << 7), 1 << 7, "neighbor section 7 (y 112..127) must be dirty");
    }

    /// §12: mining deep under a sealed stone ceiling with no emissive
    /// sources dirties ONLY the geometry sections — no light can change in
    /// a dark sealed volume (the big cave-mining win).
    #[test]
    fn sealed_cave_mining_is_geometry_only() {
        // floor 0..70, air 71..90, sealed ceiling 91..110, more stone above
        let mut w = World::new(7);
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut c = Chunk::empty();
                for y in 0..=70 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y, lz, STONE);
                        }
                    }
                }
                for y in 91..=110 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y, lz, STONE);
                        }
                    }
                }
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w.dirty_causes.clear();
        // mine the cave floor wall: (8,70,8) is stone, becomes air
        w.set_block(8, 70, 8, AIR);
        assert_eq!(w.dirty.len(), 1);
        let mask = *w.dirty.get(&(0, 0)).unwrap();
        assert_eq!(mask, 1 << 4, "geometry section 4 (y 64..79) only, got {mask:#b}");
    }

    /// §12: a NO-OP edit (same block re-written) changes nothing that light
    /// depends on — only the geometry section re-meshes.
    #[test]
    fn noop_edit_skips_light_region() {
        let mut w = flat_world(70);
        w.set_block(8, 65, 8, STONE); // STONE → STONE
        let mask = *w.dirty.get(&(0, 0)).unwrap();
        assert_eq!(mask.count_ones(), 1, "only the geometry section, got {mask:#b}");
        assert_eq!(w.dirty_section_count(), 1);
        assert_eq!(*w.dirty_causes.get(&(0, 0)).unwrap(), CAUSE_GEOMETRY);
    }

    /// §12: clear_dirty_mask keeps bits added while a job was in flight —
    /// an edit landing mid-job re-queues the chunk instead of going stale.
    #[test]
    fn clear_mask_keeps_inflight_edits() {
        let mut w = world_with_chunk();
        w.mark_sections_dirty((0, 0), 1 << 4, CAUSE_GEOMETRY);
        // job for section 4 is in flight; an edit lands on section 9
        w.mark_sections_dirty((0, 0), 1 << 9, CAUSE_LIGHT);
        // job completes: clear ONLY section 4
        w.clear_dirty_mask((0, 0), 1 << 4);
        assert_eq!(*w.dirty.get(&(0, 0)).unwrap(), 1 << 9, "in-flight edit bit survives");
        assert!(w.dirty_causes.contains_key(&(0, 0)));
        // final job clears the rest → both maps drop the chunk
        w.clear_dirty_mask((0, 0), 1 << 9);
        assert!(!w.dirty.contains_key(&(0, 0)));
        assert!(!w.dirty_causes.contains_key(&(0, 0)));
    }


}
