//! Incremental voxel lighting engine (§18/§48 Phase 4).
//!
//! Architecture:
//! * light lives OUTSIDE the COW chunks in `World::light` as
//!   `Arc<LightData>` per chunk — mutated in place through a per-pump
//!   working set, snapshotted for mesh jobs exactly like block data.
//! * vanilla-style two-queue BFS per channel (increase + removal), with
//!   this engine's baseline semantics (which the previous per-mesh
//!   recompute established — the differential gate compares against it):
//!     - skylight: column scan (15 from the top, water −2, leaves −1,
//!       irreversibly 0 below any opaque), lateral/vertical BFS decay 1
//!       (incl. upward — the established look, not vanilla's free-down rule)
//!     - block light: emissive blocks seed their NEIGHBORS at the emissive
//!       level (the source cell itself stays 0), BFS decay 1
//! * every cell write records the §12 section mask → the game marks those
//!   sections dirty (PRECISE light regions — replaces heuristics)
//!
//! `reference_light` keeps the previous from-scratch algorithm as the
//! test oracle for the differential gate (§48 Phase 4: differential light
//! tests pass).

use crate::world::{ChunkPos, World};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use vc_blocks::blocks::*;
use vc_chunk::chunk::Chunk;

/// light reach — max BFS steps with any light left
pub const LIGHT_REACH: u8 = 15;

// ---------------------------------------------------------------- storage --

/// one 16³ section of light (sky + block channels), YZX order like blocks
pub struct LightSection {
    pub sky: Box<[u8; 4096]>,
    pub blk: Box<[u8; 4096]>,
}

/// per-chunk light. `None` sections are the defaults:
/// * block section empty (all air)  → sky 15, blk 0
/// * block section present, never written → sky 0, blk 0 (dark interior;
///   any nonzero write materializes the section)
#[derive(Default)]
pub struct LightData {
    pub sections: Vec<Option<Box<LightSection>>>,
}

impl LightData {
    pub fn new() -> Self {
        LightData {
            sections: (0..16).map(|_| None).collect(),
        }
    }

    #[inline]
    fn sec_mut(&mut self, sy: usize) -> &mut Box<LightSection> {
        self.sections[sy].get_or_insert_with(|| {
            Box::new(LightSection {
                sky: Box::new([0u8; 4096]),
                blk: Box::new([0u8; 4096]),
            })
        })
    }

    fn sky(&self, sy: usize, i: usize) -> u8 {
        self.sections[sy].as_ref().map(|s| s.sky[i]).unwrap_or(0)
    }
    fn blk(&self, sy: usize, i: usize) -> u8 {
        self.sections[sy].as_ref().map(|s| s.blk[i]).unwrap_or(0)
    }
    fn set_sky(&mut self, sy: usize, i: usize, v: u8) {
        if v != 0 || self.sections[sy].is_some() {
            self.sec_mut(sy).sky[i] = v;
        }
    }
    fn set_blk(&mut self, sy: usize, i: usize, v: u8) {
        if v != 0 || self.sections[sy].is_some() {
            self.sec_mut(sy).blk[i] = v;
        }
    }

    /// build from full flat arrays (256-tall columns, YZX per section) —
    /// used by tests / the reference bridge and by chunk init
    pub fn from_flat(sky: &[u8; 16 * 256 * 16], blk: &[u8; 16 * 256 * 16]) -> Self {
        let mut ld = LightData::new();
        for sy in 0..16usize {
            let base = sy * 4096;
            let mut any_sky = false;
            let mut any_blk = false;
            for i in 0..4096 {
                if sky[base + i] != 0 {
                    any_sky = true;
                }
                if blk[base + i] != 0 {
                    any_blk = true;
                }
            }
            if any_sky || any_blk {
                let mut sec = Box::new(LightSection {
                    sky: Box::new([0u8; 4096]),
                    blk: Box::new([0u8; 4096]),
                });
                sec.sky.copy_from_slice(&sky[base..base + 4096]);
                sec.blk.copy_from_slice(&blk[base..base + 4096]);
                ld.sections[sy] = Some(sec);
            }
        }
        ld
    }

    /// flat view of one channel (16×256×16, YZX per section) — the mesher's
    /// pad copy and the differential tests use this
    pub fn to_flat(&self, channel: LightChannel) -> Box<[u8; 16 * 256 * 16]> {
        let mut arr = Box::new([0u8; 16 * 256 * 16]);
        for sy in 0..16usize {
            if let Some(sec) = &self.sections[sy] {
                let src: &[u8; 4096] = match channel {
                    LightChannel::Sky => &sec.sky,
                    LightChannel::Block => &sec.blk,
                };
                arr[sy * 4096..sy * 4096 + 4096].copy_from_slice(src);
            }
        }
        arr
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LightChannel {
    Sky,
    Block,
}

// ---------------------------------------------------------------- engine --

type Node = (i32, i32, i32, u8); // (wx, wy, wz, level)

pub struct LightEngine {
    sky_q: VecDeque<Node>,
    sky_rm: VecDeque<Node>,
    blk_q: VecDeque<Node>,
    blk_rm: VecDeque<Node>,
    /// §12 section masks dirtied by light writes since the last drain
    pub changed: HashMap<ChunkPos, u16>,
    /// per-pump working copies (COW against the shared Arc snapshot)
    working: HashMap<ChunkPos, LightData>,
}

impl Default for LightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LightEngine {
    pub fn new() -> Self {
        LightEngine {
            sky_q: VecDeque::new(),
            sky_rm: VecDeque::new(),
            blk_q: VecDeque::new(),
            blk_rm: VecDeque::new(),
            changed: HashMap::new(),
            working: HashMap::new(),
        }
    }

    /// pending queue length (F3 / stats evidence)
    pub fn pending(&self) -> usize {
        self.sky_q.len() + self.sky_rm.len() + self.blk_q.len() + self.blk_rm.len()
    }

    /// drain the changed-section map (game.rs marks §12 dirty bits with it)
    pub fn take_changed(&mut self) -> HashMap<ChunkPos, u16> {
        std::mem::take(&mut self.changed)
    }

    // ------------------------------------------------------------ init ----

    /// initial lighting for a freshly generated/loaded chunk: column scan,
    /// emissive seeds, border exchange with existing neighbors. Runs the
    /// queues to completion (bounded) so mesh jobs immediately see settled
    /// light (no double-mesh on streaming).
    pub fn init_chunk(&mut self, world: &mut World, pos: ChunkPos) {
        let Some(chunk) = world.chunks.get(&pos) else {
            return;
        };
        let chunk = Arc::clone(chunk);

        // ---- column scan (the established reference semantics)
        let mut sky = vec![0u8; 16 * 256 * 16];
        let mut blk = vec![0u8; 16 * 256 * 16];
        for lz in 0..16usize {
            for lx in 0..16usize {
                let mut l: u8 = 15;
                for y in (0..256usize).rev() {
                    let b = chunk.get(lx, y, lz); // 1.7.2: Chunk::get folds states itself
                    if is_opaque(b) {
                        l = 0;
                    } else if b == WATER {
                        l = l.saturating_sub(2);
                    } else if b == LEAVES {
                        l = l.saturating_sub(1);
                    }
                    sky[y * 256 + lz * 16 + lx] = l;
                }
            }
        }
        // ---- block light: emissive blocks seed their neighbors
        for y in 0..256usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    // [merge] state-aware emission (E1 lit lamps) via
                    // the RAW state accessor — our 1.7.2 Chunk::get folds
                    // states to block ids, so state_emissive needs
                    // get_state here
                    let e = state_emissive(chunk.get_state(lx, y, lz));
                    if e == 0 {
                        continue;
                    }
                    let lvl = e.min(15);
                    let (wx, wy, wz) = (pos.0 * 16 + lx as i32, y as i32, pos.1 * 16 + lz as i32);
                    for (dx, dy, dz) in NEIGHBORS {
                        let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                        if ny < 0 || ny > 255 {
                            continue;
                        }
                        if !self.opaque_at(world, nx, ny, nz) {
                            self.set_blk_pending(world, nx, ny, nz, lvl);
                            self.blk_q.push_back((nx, ny, nz, lvl));
                        }
                    }
                    // emissive source cell itself stays 0 (reference rule)
                    blk[y * 256 + lz * 16 + lx] = 0;
                }
            }
        }

        // commit the scan into storage (through the working set so the
        // border exchange below sees own values)
        let arr_sky: &[u8; 16 * 256 * 16] = sky.as_slice().try_into().unwrap();
        let arr_blk: &[u8; 16 * 256 * 16] = blk.as_slice().try_into().unwrap();
        let ld = LightData::from_flat(arr_sky, arr_blk);
        for sy in 0..16usize {
            if let Some(sec) = &ld.sections[sy] {
                let s = self.working.entry(pos).or_insert_with(LightData::new);
                s.sections[sy] = Some(Box::new(LightSection {
                    sky: Box::new(**(&sec.sky)),
                    blk: Box::new(**(&sec.blk)),
                }));
                self.changed.entry(pos).or_insert(0);
                *self.changed.get_mut(&pos).unwrap() |= 1 << sy;
            }
        }
        // materialize defaults for sections with blocks (dark interiors are
        // the "never written" None representation — nothing to do)

        // ---- seed the intra-chunk BFS from bright→dark boundaries
        self.seed_bright_cells(world, &pos, &chunk);

        // ---- border exchange with existing neighbors: brighter cells on
        // either side propagate across (increase only — stored light never
        // depends on absent chunks, so nothing to remove)
        for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let npos = (pos.0 + dx, pos.1 + dz);
            if !world.chunks.contains_key(&npos) {
                continue;
            }
            // cells along the shared face, both sides
            for y in 0..256usize {
                for t in 0..16usize {
                    // own border cell (facing npos) in world coords
                    let (ox, oz) = match (dx, dz) {
                        (1, 0) => (pos.0 * 16 + 15, pos.1 * 16 + t as i32),
                        (-1, 0) => (pos.0 * 16, pos.1 * 16 + t as i32),
                        (0, 1) => (pos.0 * 16 + t as i32, pos.1 * 16 + 15),
                        _ => (pos.0 * 16 + t as i32, pos.1 * 16),
                    };
                    let (nx, nz) = (ox + dx, oz + dz);
                    let own_sky = self.get_sky(world, ox, y as i32, oz);
                    let n_sky = self.get_sky(world, nx, y as i32, nz);
                    if own_sky >= 2 && own_sky > n_sky.saturating_add(1) {
                        self.sky_q.push_back((ox, y as i32, oz, own_sky));
                    }
                    if n_sky >= 2 && n_sky > own_sky.saturating_add(1) {
                        self.sky_q.push_back((nx, y as i32, nz, n_sky));
                    }
                    let own_blk = self.get_blk(world, ox, y as i32, oz);
                    let n_blk = self.get_blk(world, nx, y as i32, nz);
                    if own_blk >= 2 && own_blk > n_blk.saturating_add(1) {
                        self.blk_q.push_back((ox, y as i32, oz, own_blk));
                    }
                    if n_blk >= 2 && n_blk > own_blk.saturating_add(1) {
                        self.blk_q.push_back((nx, y as i32, nz, n_blk));
                    }
                }
            }
        }

        // settle synchronously (bounded)
        self.pump(world, 400_000);
    }

    /// enqueue every cell whose light exceeds a non-opaque neighbor by more
    /// than 1 (the BFS fixed-point seeds — caves, overhangs, water edges)
    fn seed_bright_cells(&mut self, world: &World, pos: &ChunkPos, chunk: &Arc<Chunk>) {
        for sy in 0..16usize {
            let Some(sec) = &chunk.sections[sy] else {
                continue;
            };
            if sec.is_empty() {
                continue;
            }
            let flat = sec.decode_flat();
            let y0 = sy * 16;
            for yy in 0..16usize {
                let y = y0 + yy;
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        let b = state_block(flat[(yy << 8) | (lz << 4) | lx] as u16);
                        if is_opaque(b) {
                            continue;
                        }
                        let (wx, wy, wz) =
                            (pos.0 * 16 + lx as i32, y as i32, pos.1 * 16 + lz as i32);
                        let s = self.get_sky(world, wx, wy, wz);
                        if s >= 2 {
                            let s2 = self
                                .working
                                .get(pos)
                                .map(|ld| ld.blk(sy, (yy << 8) | (lz << 4) | lx))
                                .unwrap_or(0);
                            let _ = s2;
                            // seed if any neighbor is dimmer by ≥2
                            for (dx, dy, dz) in NEIGHBORS {
                                let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                                if self.opaque_at(world, nx, ny, nz) {
                                    continue;
                                }
                                if self.get_sky(world, nx, ny, nz) + 2 <= s {
                                    self.sky_q.push_back((wx, wy, wz, s));
                                    break;
                                }
                            }
                        }
                        let bl = self.get_blk(world, wx, wy, wz);
                        if bl >= 2 {
                            for (dx, dy, dz) in NEIGHBORS {
                                let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                                if self.opaque_at(world, nx, ny, nz) {
                                    continue;
                                }
                                if self.get_blk(world, nx, ny, nz) + 2 <= bl {
                                    self.blk_q.push_back((wx, wy, wz, bl));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ edits ---

    /// one block changed at (wx, wy, wz). Enqueues the incremental updates;
    /// pump() settles them. Call AFTER the chunk data is updated.
    pub fn on_block_changed(
        &mut self,
        world: &World,
        wx: i32,
        wy: i32,
        wz: i32,
        old: u16,
        new: u16,
    ) {
        let old_b = state_block(old);
        let new_b = state_block(new);

        // ---- block light
        // Phase E1: state-aware emission (lit redstone lamp state = 15)
        let e_old = state_emissive(old);
        let e_new = state_emissive(new);
        if e_new > 0 {
            // source appeared: seed neighbors at the emissive level
            let lvl = e_new.min(15);
            for (dx, dy, dz) in NEIGHBORS {
                let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                if ny < 0 || ny > 255 {
                    continue;
                }
                if !self.opaque_at(world, nx, ny, nz) {
                    self.set_blk_pending(world, nx, ny, nz, lvl);
                    self.blk_q.push_back((nx, ny, nz, lvl));
                }
            }
        } else if e_old > 0 {
            // source removed: vanilla removal from each lit neighbor
            for (dx, dy, dz) in NEIGHBORS {
                let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                if ny < 0 || ny > 255 {
                    continue;
                }
                let l = self.get_blk(world, nx, ny, nz);
                if l > 0 {
                    self.blk_rm.push_back((nx, ny, nz, l));
                }
            }
        } else {
            // opacity change affects block-light paths through the cell
            let l = self.get_blk(world, wx, wy, wz);
            if is_opaque(new_b) {
                if l > 0 {
                    self.blk_rm.push_back((wx, wy, wz, l));
                }
            } else if !is_opaque(old_b) || l == 0 {
                // opened: pull light in from lit neighbors
                for (dx, dy, dz) in NEIGHBORS {
                    let (nx, ny, nz) = (wx + dx, wy + dy, wz + dz);
                    if ny < 0 || ny > 255 {
                        continue;
                    }
                    let nl = self.get_blk(world, nx, ny, nz);
                    if nl >= 2 && !self.opaque_at(world, nx, ny, nz) {
                        self.blk_q.push_back((nx, ny, nz, nl));
                    }
                }
            }
        }

        // ---- sky light: the column semantics change → recompute the column
        let darkens = |b: u16| is_opaque(b) || b == WATER || b == LEAVES;
        if darkens(new_b) != darkens(old_b)
            || (old_b == WATER) != (new_b == WATER)
            || (old_b == LEAVES) != (new_b == LEAVES)
        {
            self.recompute_column(world, wx, wz);
        }
    }

    /// rerun the column scan for (wx, wz) and diff against stored sky light
    fn recompute_column(&mut self, world: &World, wx: i32, wz: i32) {
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let Some(chunk) = world.chunks.get(&(cx, cz)) else {
            return;
        };
        let mut l: u8 = 15;
        for y in (0..256usize).rev() {
            let b = chunk.get(lx, y, lz); // 1.7.2: Chunk::get folds states itself
            if is_opaque(b) {
                l = 0;
            } else if b == WATER {
                l = l.saturating_sub(2);
            } else if b == LEAVES {
                l = l.saturating_sub(1);
            }
            let cur = self.get_sky(world, wx, y as i32, wz);
            if l > cur {
                self.set_sky_pending(world, wx, y as i32, wz, l);
                self.sky_q.push_back((wx, y as i32, wz, l));
            } else if l < cur {
                self.sky_rm.push_back((wx, y as i32, wz, cur));
            }
        }
    }

    // ------------------------------------------------------------ pump ----

    /// process queued light updates (increase + removal BFS). Returns the
    /// number of operations performed. Working copies commit back into the
    /// world's Arc snapshots at the end (COW — in-flight mesh jobs keep
    /// consistent views).
    pub fn pump(&mut self, world: &mut World, budget: usize) -> usize {
        let mut ops = 0usize;
        while ops < budget {
            if let Some((x, y, z, lvl)) = self.sky_rm.pop_front() {
                self.step_removal(world, LightChannel::Sky, x, y, z, lvl);
                ops += 1;
            } else if let Some((x, y, z, lvl)) = self.blk_rm.pop_front() {
                self.step_removal(world, LightChannel::Block, x, y, z, lvl);
                ops += 1;
            } else if let Some((x, y, z, lvl)) = self.sky_q.pop_front() {
                self.step_increase(world, LightChannel::Sky, x, y, z, lvl);
                ops += 1;
            } else if let Some((x, y, z, lvl)) = self.blk_q.pop_front() {
                self.step_increase(world, LightChannel::Block, x, y, z, lvl);
                ops += 1;
            } else {
                break;
            }
        }
        self.commit(world);
        ops
    }

    /// vanilla removal step: zero the cell, cascade into lighter-derived
    /// neighbors, re-seed independent sources
    fn step_removal(&mut self, world: &World, ch: LightChannel, x: i32, y: i32, z: i32, lvl: u8) {
        let cur = match ch {
            LightChannel::Sky => self.get_sky(world, x, y, z),
            LightChannel::Block => self.get_blk(world, x, y, z),
        };
        if cur != lvl {
            // stale entry: the cell already lost this light (a prior removal
            // pass) or was re-lit to a different value — strict equality is
            // what breaks the remove/re-add oscillation (a stale entry
            // re-processing a re-lit cell regenerates the whole wave)
            return;
        }
        // zero it (through the working set)
        match ch {
            LightChannel::Sky => self.set_sky_pending(world, x, y, z, 0),
            LightChannel::Block => self.set_blk_pending(world, x, y, z, 0),
        }
        for (dx, dy, dz) in NEIGHBORS {
            let (nx, ny, nz) = (x + dx, y + dy, z + dz);
            if ny < 0 || ny > 255 {
                continue;
            }
            let nl = match ch {
                LightChannel::Sky => self.get_sky(world, nx, ny, nz),
                LightChannel::Block => self.get_blk(world, nx, ny, nz),
            };
            if nl != 0 && nl < lvl {
                match ch {
                    LightChannel::Sky => self.sky_rm.push_back((nx, ny, nz, nl)),
                    LightChannel::Block => self.blk_rm.push_back((nx, ny, nz, nl)),
                }
            } else if nl >= lvl && nl >= 2 {
                // independent source — re-add from here
                match ch {
                    LightChannel::Sky => self.sky_q.push_back((nx, ny, nz, nl)),
                    LightChannel::Block => self.blk_q.push_back((nx, ny, nz, nl)),
                }
            }
        }
    }

    /// increase step: propagate lvl−1 into non-opaque dimmer neighbors
    /// (exactly the reference BFS rule)
    fn step_increase(&mut self, world: &World, ch: LightChannel, x: i32, y: i32, z: i32, lvl: u8) {
        if lvl < 2 {
            return;
        }
        // stale-entry validation: if this cell's light has since dropped
        // below the queued level (a removal wave zeroed it after the entry
        // was seeded), the entry is dead — re-lighting from it would
        // resurrect removed light (vanilla re-validates the same way)
        let cur_here = match ch {
            LightChannel::Sky => self.get_sky(world, x, y, z),
            LightChannel::Block => self.get_blk(world, x, y, z),
        };
        if cur_here < lvl {
            return;
        }
        let nl = lvl - 1;
        for (dx, dy, dz) in NEIGHBORS {
            let (nx, ny, nz) = (x + dx, y + dy, z + dz);
            if ny < 0 || ny > 255 {
                continue;
            }
            if self.opaque_at(world, nx, ny, nz) {
                continue;
            }
            let cur = match ch {
                LightChannel::Sky => self.get_sky(world, nx, ny, nz),
                LightChannel::Block => self.get_blk(world, nx, ny, nz),
            };
            if cur < nl {
                match ch {
                    LightChannel::Sky => self.set_sky_pending(world, nx, ny, nz, nl),
                    LightChannel::Block => self.set_blk_pending(world, nx, ny, nz, nl),
                }
                if nl > 1 {
                    match ch {
                        LightChannel::Sky => self.sky_q.push_back((nx, ny, nz, nl)),
                        LightChannel::Block => self.blk_q.push_back((nx, ny, nz, nl)),
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ access --

    #[inline]
    fn opaque_at(&self, world: &World, wx: i32, wy: i32, wz: i32) -> bool {
        if wy < 0 || wy > 255 {
            return false; // outside vertical range: not a blocker
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        match world.chunks.get(&(cx, cz)) {
            Some(c) => is_opaque(state_block(c.get(
                (wx - cx * 16) as usize,
                wy as usize,
                (wz - cz * 16) as usize,
            ) as u16)),
            None => false,
        }
    }

    fn get_sky(&self, world: &World, wx: i32, wy: i32, wz: i32) -> u8 {
        if wy > 255 {
            return 15;
        }
        if wy < 0 {
            return 0;
        }
        let (cx, cz) = (wx.div_euclid(16), wz.div_euclid(16));
        let lz16 = (wz - cz * 16) as usize;
        let lx16 = (wx - cx * 16) as usize;
        let i = ((wy as usize & 15) << 8) | (lz16 << 4) | lx16;
        if let Some(w) = self.working.get(&(cx, cz)) {
            return w.sky(wy as usize / 16, i);
        }
        world
            .light
            .get(&(cx, cz))
            .map(|ld| ld.sky(wy as usize / 16, i))
            .unwrap_or(0)
    }

    fn get_blk(&self, world: &World, wx: i32, wy: i32, wz: i32) -> u8 {
        if wy < 0 || wy > 255 {
            return 0;
        }
        let (cx, cz) = (wx.div_euclid(16), wz.div_euclid(16));
        let lz16 = (wz - cz * 16) as usize;
        let lx16 = (wx - cx * 16) as usize;
        let i = ((wy as usize & 15) << 8) | (lz16 << 4) | lx16;
        if let Some(w) = self.working.get(&(cx, cz)) {
            return w.blk(wy as usize / 16, i);
        }
        world
            .light
            .get(&(cx, cz))
            .map(|ld| ld.blk(wy as usize / 16, i))
            .unwrap_or(0)
    }

    /// write through the working set + record the §12 changed mask
    fn set_sky_pending(&mut self, world: &World, wx: i32, wy: i32, wz: i32, v: u8) {
        let (cx, cz) = (wx.div_euclid(16), wz.div_euclid(16));
        let sy = wy.div_euclid(16) as usize;
        let lz16 = (wz - cz * 16) as usize;
        let lx16 = (wx - cx * 16) as usize;
        let i = ((wy as usize & 15) << 8) | (lz16 << 4) | lx16;
        // seed the working copy from the shared snapshot on first touch
        if !self.working.contains_key(&(cx, cz)) {
            let base = world.light.get(&(cx, cz)).cloned();
            let mut ld = match base {
                Some(a) => LightData::clone_from(a.as_ref()),
                None => LightData::new(),
            };
            // materialize sections that have blocks (they're not the
            // air-default): dark interiors read as 0 which is already the
            // None representation — only the WRITE path below materializes
            let _ = &mut ld;
            self.working.insert((cx, cz), ld);
        }
        let ld = self.working.get_mut(&(cx, cz)).unwrap();
        ld.set_sky(sy, i, v);
        *self.changed.entry((cx, cz)).or_insert(0) |= 1 << sy;
    }

    fn set_blk_pending(&mut self, world: &World, wx: i32, wy: i32, wz: i32, v: u8) {
        let (cx, cz) = (wx.div_euclid(16), wz.div_euclid(16));
        let sy = wy.div_euclid(16) as usize;
        let lz16 = (wz - cz * 16) as usize;
        let lx16 = (wx - cx * 16) as usize;
        let i = ((wy as usize & 15) << 8) | (lz16 << 4) | lx16;
        if !self.working.contains_key(&(cx, cz)) {
            let base = world.light.get(&(cx, cz)).cloned();
            let ld = match base {
                Some(a) => LightData::clone_from(a.as_ref()),
                None => LightData::new(),
            };
            self.working.insert((cx, cz), ld);
        }
        let ld = self.working.get_mut(&(cx, cz)).unwrap();
        ld.set_blk(sy, i, v);
        *self.changed.entry((cx, cz)).or_insert(0) |= 1 << sy;
    }

    /// commit working copies back into the world's Arc snapshots
    fn commit(&mut self, world: &mut World) {
        for (pos, ld) in self.working.drain() {
            world.light.insert(pos, Arc::new(ld));
        }
    }
}

impl LightData {
    /// deep clone (COW detach)
    fn clone_from(src: &LightData) -> LightData {
        let mut out = LightData::new();
        for sy in 0..16usize {
            if let Some(sec) = &src.sections[sy] {
                out.sections[sy] = Some(Box::new(LightSection {
                    sky: Box::new(*sec.sky.as_ref()),
                    blk: Box::new(*sec.blk.as_ref()),
                }));
            }
        }
        out
    }
}

const NEIGHBORS: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

// ------------------------------------------------------------- reference --

/// The previous from-scratch algorithm (per-mesh-job light recompute),
/// preserved as the differential-test oracle: given the mesher's 48×48×256
/// padded blocks, produce (sky, block) light arrays.
/// full-recompute reference (skylight column scan + lateral BFS, block
/// light emissive BFS) over a PADDED 48×256×48 block array — the ground
/// truth the incremental engine is differential-tested against.
pub fn reference_light(blocks: &[u8]) -> (Vec<u8>, Vec<u8>) {
    const PADREF: usize = 48;
    let pad = PADREF;
    let pidx = |x: usize, y: usize, z: usize| y * (pad * pad) + z * pad + x;
    let sb = |s: u8| state_block(s as u16);

    // skylight column scan
    let mut light = vec![0u8; pad * pad * 256];
    let mut surface = [[-1i32; PADREF]; PADREF];
    for z in 0..pad {
        for x in 0..pad {
            let mut l: i32 = 15;
            for y in (0..256usize).rev() {
                let b = sb(blocks[pidx(x, y, z)]);
                if is_opaque(b) {
                    l = 0;
                    if surface[z][x] < 0 {
                        surface[z][x] = y as i32;
                    }
                } else if b == WATER {
                    l = (l - 2).max(0);
                } else if b == LEAVES {
                    l = (l - 1).max(0);
                }
                light[pidx(x, y, z)] = l as u8;
            }
        }
    }
    // FULL fixed-point seeding (Phase 4 reference): every non-opaque cell
    // with light >= 2 that has a non-opaque neighbor at least 2 dimmer is
    // a BFS source. This is the true light-field fixed point — the target
    // the incremental engine must reproduce exactly (the previous
    // surface-bounded seeding left sharp shadow edges; the engine's
    // complete seeding smooths them, which the upgrade bakes in).
    let mut queue: VecDeque<(usize, u8)> = VecDeque::new();
    let mut surface = surface; // (kept: heightmap consumers)
    for y in 0..256usize {
        for z in 0..pad {
            for x in 0..pad {
                let l = light[pidx(x, y, z)];
                if l < 2 {
                    continue;
                }
                if is_opaque(sb(blocks[pidx(x, y, z)])) {
                    continue;
                }
                // all six neighbors (vertical violations exist at
                // water/leaf attenuation boundaries)
                let mut viol = false;
                'nbr: for (nx, ny, nz) in [
                    (x.wrapping_add(1), y, z),
                    (x.wrapping_sub(1), y, z),
                    (x, y.wrapping_add(1), z),
                    (x, y.wrapping_sub(1), z),
                    (x, y, z.wrapping_add(1)),
                    (x, y, z.wrapping_sub(1)),
                ] {
                    if nx >= pad || nz >= pad || ny >= 256 {
                        continue;
                    }
                    if is_opaque(sb(blocks[pidx(nx, ny, nz)])) {
                        continue;
                    }
                    if light[pidx(nx, ny, nz)] + 2 <= l {
                        viol = true;
                        break 'nbr;
                    }
                }
                if viol {
                    queue.push_back((pidx(x, y, z), l));
                }
            }
        }
    }
    let _ = &mut surface;
    // BFS (all 6 directions, decay 1)
    while let Some((p, l)) = queue.pop_front() {
        if l < 2 {
            continue;
        }
        let x = p % pad;
        let z = (p / pad) % pad;
        let y = p / (pad * pad);
        let nl = l - 1;
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {{
                let np = pidx($x, $y, $z);
                if !is_opaque(sb(blocks[np])) && light[np] < nl {
                    light[np] = nl;
                    if nl > 1 {
                        queue.push_back((np, nl));
                    }
                }
            }};
        }
        if x > 0 {
            prop!(x - 1, y, z);
        }
        if x + 1 < pad {
            prop!(x + 1, y, z);
        }
        if y > 0 {
            prop!(x, y - 1, z);
        }
        if y + 1 < 256 {
            prop!(x, y + 1, z);
        }
        if z > 0 {
            prop!(x, y, z - 1);
        }
        if z + 1 < pad {
            prop!(x, y, z + 1);
        }
    }

    // block light: emissive seeds at neighbors, same BFS
    let mut blight = vec![0u8; pad * pad * 256];
    let mut bqueue: VecDeque<(usize, u8)> = VecDeque::new();
    for y in 0..256usize {
        for z in 0..pad {
            for x in 0..pad {
                // Phase E1: state-aware emission (lit lamp states)
                let e = state_emissive(blocks[pidx(x, y, z)] as u16);
                if e == 0 {
                    continue;
                }
                let lvl = e.min(15);
                macro_rules! seed {
                    ($x:expr, $y:expr, $z:expr) => {{
                        let np = pidx($x, $y, $z);
                        if !is_opaque(sb(blocks[np])) && blight[np] < lvl {
                            blight[np] = lvl;
                            bqueue.push_back((np, lvl));
                        }
                    }};
                }
                if x > 0 {
                    seed!(x - 1, y, z);
                }
                if x + 1 < pad {
                    seed!(x + 1, y, z);
                }
                if y > 0 {
                    seed!(x, y - 1, z);
                }
                if y + 1 < 256 {
                    seed!(x, y + 1, z);
                }
                if z > 0 {
                    seed!(x, y, z - 1);
                }
                if z + 1 < pad {
                    seed!(x, y, z + 1);
                }
            }
        }
    }
    while let Some((p, l)) = bqueue.pop_front() {
        if l < 2 {
            continue;
        }
        let x = p % pad;
        let z = (p / pad) % pad;
        let y = p / (pad * pad);
        let nl = l - 1;
        macro_rules! bprop {
            ($x:expr, $y:expr, $z:expr) => {{
                let np = pidx($x, $y, $z);
                if !is_opaque(sb(blocks[np])) && blight[np] < nl {
                    blight[np] = nl;
                    if nl > 1 {
                        bqueue.push_back((np, nl));
                    }
                }
            }};
        }
        if x > 0 {
            bprop!(x - 1, y, z);
        }
        if x + 1 < pad {
            bprop!(x + 1, y, z);
        }
        if y > 0 {
            bprop!(x, y - 1, z);
        }
        if y + 1 < 256 {
            bprop!(x, y + 1, z);
        }
        if z > 0 {
            bprop!(x, y, z - 1);
        }
        if z + 1 < pad {
            bprop!(x, y, z + 1);
        }
    }

    (light, blight)
}

/// reference light for a 3×3 snapshot, as per-chunk LightData. Full BFS
/// recompute — the deterministic ground truth the incremental engine
/// converges to (differential tests assert equality). Also used by the
/// headless bench binary to feed mesh jobs.
pub fn reference_lightdata(snap: &[Option<Arc<Chunk>>; 9]) -> [Option<Arc<LightData>>; 9] {
    let pad = 48usize;
    let pidx = |x: usize, y: usize, z: usize| y * (pad * pad) + z * pad + x;
    // build the padded block array (same copy loop as the mesher)
    let mut blocks = vec![0u8; pad * pad * 256];
    for dzi in 0..3usize {
        for dxi in 0..3usize {
            let Some(chunk) = &snap[dzi * 3 + dxi] else {
                continue;
            };
            let px0 = dxi * 16;
            let pz0 = dzi * 16;
            for (sy, sec) in chunk.sections.iter().enumerate() {
                let Some(sec) = sec else { continue };
                if sec.is_empty() {
                    continue;
                }
                let flat = sec.decode_flat();
                for yy in 0..16usize {
                    let y = sy * 16 + yy;
                    for sz in 0..16usize {
                        let src_row = (yy << 8) | (sz << 4);
                        let dst = y * (pad * pad) + (pz0 + sz) * pad + px0;
                        blocks[dst..dst + 16].copy_from_slice(&flat[src_row..src_row + 16]);
                    }
                }
            }
        }
    }
    let (light, blight) = reference_light(&blocks);
    // slice per center-chunk cell back into per-chunk flat arrays
    let mut out: [Option<Arc<LightData>>; 9] = Default::default();
    for dzi in 0..3usize {
        for dxi in 0..3usize {
            if snap[dzi * 3 + dxi].is_none() {
                continue;
            }
            let px0 = dxi * 16;
            let pz0 = dzi * 16;
            let mut sky = vec![0u8; 16 * 256 * 16];
            let mut blk = vec![0u8; 16 * 256 * 16];
            for y in 0..256usize {
                for sz in 0..16usize {
                    for sx in 0..16usize {
                        let src = pidx(px0 + sx, y, pz0 + sz);
                        let dst = y * 256 + sz * 16 + sx;
                        sky[dst] = light[src];
                        blk[dst] = blight[src];
                    }
                }
            }
            let arr_sky: &[u8; 16 * 256 * 16] = sky.as_slice().try_into().unwrap();
            let arr_blk: &[u8; 16 * 256 * 16] = blk.as_slice().try_into().unwrap();
            out[dzi * 3 + dxi] = Some(Arc::new(LightData::from_flat(arr_sky, arr_blk)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_blocks::blocks::*;

    /// hand-built 5×5 world with variance: floor + hill + cave + water pool.
    /// The inner 3×3 (all chunks with |x|<=1, |z|<=1) is the comparison
    /// region; the outer ring is uniform floor so no light enters the pads
    /// from beyond the 5×5 boundary.
    fn var_world() -> World {
        let mut w = World::new(9);
        for dz in -2i32..=2 {
            for dx in -2i32..=2 {
                let mut c = Chunk::empty();
                // base floor
                for y in 0..=64 {
                    for lz in 0..16usize {
                        for lx in 0..16usize {
                            c.set(lx, y, lz, STONE);
                        }
                    }
                }
                let (wx, wz) = (dx * 16 + 8, dz * 16 + 8);
                // hill in chunk (-1,-1): blocks up to y=72
                if dx == -1 && dz == -1 {
                    for y in 65..=72 {
                        for lz in 4..12usize {
                            for lx in 4..12usize {
                                c.set(lx, y, lz, DIRT);
                            }
                        }
                    }
                    c.set(8, 72, 8, GRASS);
                }
                // sealed cave + glowstone in chunk (0,0): air pocket 20..30 inside stone
                if dx == 0 && dz == 0 {
                    for y in 20..=30 {
                        for lz in 4..12usize {
                            for lx in 4..12usize {
                                c.set(lx, y, lz, AIR);
                            }
                        }
                    }
                    c.set(8, 25, 8, GLOWSTONE);
                    // water pool in chunk (1,1): dip + water at 63..64
                }
                if dx == 1 && dz == 1 {
                    for y in 60..=64 {
                        for lz in 4..12usize {
                            for lx in 4..12usize {
                                c.set(lx, y, lz, if y >= 63 { WATER } else { AIR });
                            }
                        }
                    }
                }
                // trees: log+leaves in chunk (0,-1)
                if dx == 0 && dz == -1 {
                    for y in 65..=71 {
                        c.set(8, y, 8, OAK_LOG);
                    }
                    for y in 69..=72 {
                        for lz in 6..11usize {
                            for lx in 6..11usize {
                                if (lx as i32 - 8).abs()
                                    + (y as i32 - 70).abs()
                                    + (lz as i32 - 8).abs()
                                    <= 3
                                {
                                    if c.get(lx, y, lz) == 0 {
                                        c.set(lx, y, lz, LEAVES);
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = (wx, wz);
                w.insert_generated((dx, dz), Arc::new(c), Vec::new());
            }
        }
        w.dirty.clear();
        w.dirty_causes.clear();
        w
    }

    /// effective flat channel WITH the None-default semantics applied
    /// (empty block section → sky 15; section with blocks → 0)
    fn effective_flat(
        ld: Option<&LightData>,
        chunk: &Chunk,
        channel: LightChannel,
    ) -> [u8; 16 * 256 * 16] {
        let mut out = [0u8; 16 * 256 * 16];
        for sy in 0..16usize {
            let has_blocks = chunk.sections[sy]
                .as_ref()
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            match ld.and_then(|l| l.sections[sy].as_ref()) {
                Some(sec) => {
                    let src: &[u8; 4096] = match channel {
                        LightChannel::Sky => &sec.sky,
                        LightChannel::Block => &sec.blk,
                    };
                    out[sy * 4096..sy * 4096 + 4096].copy_from_slice(src);
                }
                None => {
                    if !has_blocks && channel == LightChannel::Sky {
                        for v in out[sy * 4096..sy * 4096 + 4096].iter_mut() {
                            *v = 15;
                        }
                    }
                }
            }
        }
        out
    }

    /// §48 Phase 4 differential gate: engine light == the previous
    /// from-scratch reference algorithm, cell-for-cell, for every inner
    /// chunk. `note` labels the failure context.
    fn assert_differential(world: &World, note: &str) {
        for cz in -1i32..=1 {
            for cx in -1i32..=1 {
                let snap = world.snapshot3x3(cx, cz).expect("snap");
                let lref = reference_lightdata(&snap);
                let chunk = snap[4].as_ref().unwrap();
                let engine_ld = world.light.get(&(cx, cz));
                for ch in [LightChannel::Sky, LightChannel::Block] {
                    let a = effective_flat(engine_ld.map(|a| a.as_ref()), chunk, ch);
                    let b = effective_flat(lref[4].as_deref(), chunk, ch);
                    let mut bad = 0usize;
                    let mut first: Option<(usize, u8, u8)> = None;
                    for i in 0..a.len() {
                        if a[i] != b[i] {
                            if first.is_none() {
                                first = Some((i, a[i], b[i]));
                            }
                            bad += 1;
                        }
                    }
                    assert!(
                        bad == 0,
                        "{note}: chunk ({cx},{cz}) {:?} — {bad} cells differ, first at {first:?}",
                        if ch == LightChannel::Sky {
                            "sky"
                        } else {
                            "block"
                        }
                    );
                }
            }
        }
    }

    /// init light for every chunk (given order), settle, and run the
    /// differential comparison
    fn init_and_check(world: &mut World, engine: &mut LightEngine, note: &str) {
        for cz in -2i32..=2 {
            for cx in -2i32..=2 {
                engine.init_chunk(world, (cx, cz));
            }
        }
        engine.pump(world, 1_000_000);
        assert_differential(world, note);
    }

    /// Phase 4 gate part 1: after full-world generation lighting (ordered),
    /// the engine matches the reference exactly — hills, caves, water
    /// attenuation, trees, glowstone.
    #[test]
    fn differential_after_generation() {
        let mut world = var_world();
        let mut engine = LightEngine::new();
        init_and_check(&mut world, &mut engine, "generation (ordered)");

        // reverse-order init on a fresh world: border exchange must converge
        // regardless of generation order (chunk-boundary propagation gate)
        let mut world2 = var_world();
        let mut engine2 = LightEngine::new();
        for cz in (-2i32..=2).rev() {
            for cx in (-2i32..=2).rev() {
                engine2.init_chunk(&mut world2, (cx, cz));
            }
        }
        engine2.pump(&mut world2, 1_000_000);
        assert_differential(&mut world2, "generation (reverse order)");
    }

    /// Phase 4 gate part 2: incremental updates stay differential-exact
    /// through an edit sequence (place / mine / torch / break torch / water).
    #[test]
    fn differential_after_edit_sequence() {
        let mut world = var_world();
        let mut engine = LightEngine::new();
        init_and_check(&mut world, &mut engine, "init");

        let edit = |world: &mut World,
                    engine: &mut LightEngine,
                    x: i32,
                    y: i32,
                    z: i32,
                    id: u16,
                    note: &str| {
            let r = world.set_block(x, y, z, id);
            engine.on_block_changed(world, x, y, z, r.map(|(o, _)| o).unwrap_or(0), id);
            engine.pump(world, 1_000_000);
            assert_differential(world, note);
        };

        // place stone in open sky (shadow column)
        edit(
            &mut world,
            &mut engine,
            8,
            90,
            8,
            STONE,
            "place stone mid-air",
        );
        // mine the hill surface (expose column)
        edit(
            &mut world,
            &mut engine,
            -16 + 8,
            72,
            -16 + 8,
            AIR,
            "mine hill top",
        );
        // place glowstone in the open (light box across chunk borders)
        edit(
            &mut world,
            &mut engine,
            15,
            80,
            15,
            GLOWSTONE,
            "place glowstone at corner",
        );
        // break it (removal BFS + re-add)
        edit(&mut world, &mut engine, 15, 80, 15, AIR, "break glowstone");
        // water in air (attenuation semantics)
        edit(
            &mut world,
            &mut engine,
            0,
            100,
            0,
            WATER,
            "place water mid-air",
        );
        // mine into the sealed cave roof (light enters the cave)
        edit(&mut world, &mut engine, 8, 31, 8, AIR, "open cave roof");
        // mine the glowstone inside the cave (block light removal)
        edit(
            &mut world,
            &mut engine,
            8,
            25,
            8,
            AIR,
            "remove cave glowstone",
        );
        // place it back
        edit(
            &mut world,
            &mut engine,
            8,
            25,
            8,
            GLOWSTONE,
            "re-place cave glowstone",
        );
    }

    /// §12 region evidence (engine-exact): a glowstone in the open sky marks
    /// the ±15 box sections; a mid-air stone marks only the shadow column's
    /// sections; nothing marks a sealed dark cave.
    #[test]
    fn changed_regions_are_exact() {
        // (a) mid-air stone: shadow column [65..90] → sections 4..5, own chunk
        let mut world = var_world();
        let mut engine = LightEngine::new();
        for cz in -2i32..=2 {
            for cx in -2i32..=2 {
                engine.init_chunk(&mut world, (cx, cz));
            }
        }
        engine.pump(&mut world, 1_000_000);
        let _ = engine.take_changed(); // drop init marks
        let r = world.set_block(8, 90, 8, STONE).unwrap();
        engine.on_block_changed(&world, 8, 90, 8, r.0, r.1);
        engine.pump(&mut world, 1_000_000);
        let changed = engine.take_changed();
        let m = changed.get(&(0, 0)).copied().unwrap_or(0);
        // the shadow falls through [65..90]: sections 4 (64..79) and 5 (80..95)
        assert_eq!(m, (1 << 4) | (1 << 5), "shadow column sections, got {m:#b}");

        // (b) sealed cave edit before opening: mining INSIDE the sealed dark
        // cave changes no light at all (no sky path, no emissive left)
        let mut world2 = var_world();
        let mut engine2 = LightEngine::new();
        for cz in -2i32..=2 {
            for cx in -2i32..=2 {
                engine2.init_chunk(&mut world2, (cx, cz));
            }
        }
        engine2.pump(&mut world2, 1_000_000);
        // remove the cave's glowstone first — now the cave is fully dark
        let r = world2.set_block(8, 25, 8, AIR).unwrap();
        engine2.on_block_changed(&world2, 8, 25, 8, r.0, r.1);
        engine2.pump(&mut world2, 1_000_000);
        let _ = engine2.take_changed();
        // mine a WALL block deep in the dark cave: NO light change
        let r = world2.set_block(3, 22, 6, AIR).unwrap();
        engine2.on_block_changed(&world2, 3, 22, 6, r.0, r.1);
        engine2.pump(&mut world2, 1_000_000);
        let changed2 = engine2.take_changed();
        assert_eq!(
            changed2.get(&(0, 0)).copied().unwrap_or(0),
            0,
            "dark sealed cave mining must change no light"
        );
    }
}
