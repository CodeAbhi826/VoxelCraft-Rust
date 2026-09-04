//! Phase 9 (Master Spec §48 / §14 / §43) — draw submission data layer.
//!
//! Pure-CPU, headless-testable core of the region-arena draw submission
//! system. Everything wgpu-touching lives in `render.rs`; this module owns
//! the data structures and the math so they can be unit-tested and
//! benchmarked without a GPU (§37: CI-runnable; §49: tested + documented).
//!
//! Design (spec §14 ladder, items 3+4+5 with capability detection):
//!
//! * **Regional mega-buffers** — one vertex + index buffer pair per 8×8
//!   chunk region ("mesh region"). Chunks sub-allocate element ranges
//!   (`MeshSlot`) via a first-fit free-list (`SlotAlloc`). Remeshes that
//!   fit the old slot write in place — §14 "no churn on repeated edits";
//!   growth is a doubling realloc + GPU→GPU copy, submitted strictly
//!   before the new data write (§43: no synchronized host→device stalls).
//! * **Draw paths** — `render.rs` picks per device:
//!   - native w/ `MULTI_DRAW_INDIRECT` + `INDIRECT_FIRST_INSTANCE`:
//!     one `multi_draw_indexed_indirect` per region run (`IndirectArgs`),
//!   - everything else (WebGPU, WebGL2, GL): bind the region arena once
//!     per region run + per-chunk `draw_indexed(range, base_vertex,
//!     origin..origin+1)` — zero per-chunk buffer binds. Verified in
//!     wgpu-hal 22's gles/mod.rs: GL/GLES/WebGL2 emulate non-zero
//!     `first_instance` on *direct* draws by offsetting the instance
//!     attribute, so the same vertex shader + origin buffer work on every
//!   backend unchanged.
//! * **Ordering** — region-major near→far (`order_by_region`): chunks are
//!   grouped into contiguous per-region runs (one bind per run) while
//!   keeping an approximately front-to-back order for early-z. Water
//!   reverses the whole order (far→near) for correct blending.
//! * **Origins stay instance-rate** — the per-frame origin buffer is bound
//!   WHOLE once per pass; each draw selects its row with
//!   `first_instance = origin` (indirect records carry it too, hence the
//!   `INDIRECT_FIRST_INSTANCE` requirement for the MDI path).
//!
//! Old per-chunk path (for scale): 3 buffer binds per chunk per pass ×
//! 3 passes (shadow/terrain/water). New: binds = 3 + 2·regions·passes;
//! draws = chunks (loop path) or regions (MDI path).

use vc_world::world::ChunkPos;
use std::collections::HashMap;

/// chunks per mesh-region side (8 → 128×128 blocks, ≈1–16 MB arena)
pub const REGION_CHUNKS: i32 = 8;

/// mesh-region grid key of a chunk column
#[inline]
pub fn region_of(pos: ChunkPos) -> (i32, i32) {
    (
        pos.0.div_euclid(REGION_CHUNKS),
        pos.1.div_euclid(REGION_CHUNKS),
    )
}

/// region center in world blocks (x, z)
#[inline]
pub fn region_center(region: (i32, i32)) -> (f32, f32) {
    let c = REGION_CHUNKS as f32 * 16.0;
    (
        region.0 as f32 * c + c * 0.5,
        region.1 as f32 * c + c * 0.5,
    )
}

// ---------------------------------------------------------------- slots --

/// element range in an arena buffer (vertices or u32 indices)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElemRange {
    pub off: u32,
    pub len: u32,
}

/// first-fit free-range allocator over one arena buffer (element
/// granularity; vertices are 16 B and indices 4 B so every offset is
/// copy/write-aligned). Free ranges are NOT split — the taker owns the
/// whole range as its capacity, which preserves the §14 in-place-reuse
/// semantics (a smaller remesh keeps the slot; a bigger one re-allocs).
#[derive(Clone, Debug, Default)]
pub struct SlotAlloc {
    bump: u32,
    free: Vec<ElemRange>,
    /// number of live (allocated, not released) slots — the "region is
    /// dead and its buffers can be destroyed" signal
    live: u32,
}

impl SlotAlloc {
    /// allocate a slot holding at least `min_len` elements.
    /// `min_len == 0` returns the null slot (no arena space used).
    pub fn alloc(&mut self, min_len: u32) -> (u32, u32) {
        if min_len == 0 {
            return (0, 0);
        }
        self.live += 1;
        if let Some(i) = self.free.iter().position(|r| r.len >= min_len) {
            let r = self.free.swap_remove(i);
            return (r.off, r.len);
        }
        let off = self.bump;
        self.bump += min_len;
        (off, min_len)
    }

    /// return a slot's capacity to the free pool
    pub fn release(&mut self, off: u32, cap: u32) {
        if cap > 0 {
            self.live -= 1;
            self.free.push(ElemRange { off, len: cap });
        }
    }

    /// high-water mark (elements ever bump-allocated)
    pub fn used(&self) -> u32 {
        self.bump
    }

    /// true when NO live slots remain (free pool + bump are just
    /// bookkeeping — the backing buffer can be destroyed)
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// number of pooled free ranges (diagnostics / tests)
    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    /// live slot count (diagnostics / tests)
    pub fn live_count(&self) -> u32 {
        self.live
    }
}

/// new arena capacity (elements) that covers `needed`, doubling from
/// `cur` with a floor so tiny meshes don't strangle-grow the buffer
pub fn grow_plan(needed: u32, cur: u32) -> u32 {
    let mut cap = cur.max(4096);
    while cap < needed {
        cap = cap.saturating_mul(2);
    }
    cap
}

/// a mesh's placement inside one region arena.
/// `n` is the live index count (≤ i_cap); v/i offsets are element indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshSlot {
    pub region: (i32, i32),
    pub v_off: u32,
    pub v_cap: u32,
    pub i_off: u32,
    pub i_cap: u32,
    pub n: u32,
}

impl MeshSlot {
    /// null slot for empty meshes (never drawn, no arena space)
    pub const EMPTY: MeshSlot = MeshSlot { region: (0, 0), v_off: 0, v_cap: 0, i_off: 0, i_cap: 0, n: 0 };

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

/// GPU-side bookkeeping for one chunk column (pure data — the actual
/// buffers live in the owning `RegionArena` in render.rs)
#[derive(Clone, Debug)]
pub struct ChunkGpu {
    pub solid: MeshSlot,
    pub water: Option<MeshSlot>,
    /// Phase 6 §26: occlusion-graph data for this column (see `ChunkOccl`)
    pub occl: ChunkOccl,
}

// ------------------------------------------- Phase 6 §26: occlusion graph --

/// face indices into `ChunkOccl::sides` (band*4 + face)
pub const FACE_PX: u8 = 0;
pub const FACE_NX: u8 = 1;
pub const FACE_PZ: u8 = 2;
pub const FACE_NZ: u8 = 3;

/// Occlusion-graph data for one chunk column, computed at mesh time from
/// the block snapshot + the fresh section meshes.
///
/// **Technique** (dossier Part 1 §6: "chunk-graph occlusion culling
/// (Sodium's technique / classic portal-culling), LGPL-3.0 — technique
/// only"): a flood through the section grid gates propagation on the
/// actual boundary geometry:
///
/// * `sides` bit `(band*4 + face)`: that 16×16×1 wall of the section has
///   ≥1 non-opaque cell — straight rays leaving the column horizontally
///   through band `b` must pass through one of those cells.
/// * `planes` bit `s`: the horizontal plane between bands `s`/`s+1`
///   (y = s·16+15) has ≥1 non-opaque cell — rays moving vertically inside
///   the column cross exactly that plane.
/// * `geo` bit `b`: band `b`'s mesh has triangles (interior faces of
///   fully-buried solid are culled by the mesher, so a sealed band has no
///   geometry to hide anyway — geo marks bands worth drawing).
///
/// The graph flood is **sound** (never culls a chunk that could be
/// visible): every propagation step corresponds to a family of straight
/// rays — horizontal crossings are gated by the exact wall band the ray
/// crosses (entry cell shares the crossing y ⇒ same section band), and
/// vertical movement is gated by the exact plane. It is *conservative*
/// in chunk granularity: a column is drawn when ANY of its geometry bands
/// is reachable (the merged per-chunk mesh can't be partially drawn).
///
/// **Honest scope notes** (§49 documented behavior):
/// * overworld surfaces barely cull — surface sections interconnect
///   through open air, and steep sky rays can descend behind hills
///   (correctly conservative);
/// * the wins are sealed subterranean columns — cave chunks with no
///   surface exposure, and Nether rock sealed from the tunnels — where
///   every wall/plane of the geometry bands is opaque;
/// * fully-buried solid columns were already skipped as empty meshes;
///   this system culls columns that HAVE geometry but no reachable band.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChunkOccl {
    pub sides: u64,
    pub planes: u16,
    pub geo: u16,
}

impl ChunkOccl {
    #[inline]
    pub fn wall_open(&self, band: u8, face: u8) -> bool {
        self.sides & (1u64 << (band as u32 * 4 + face as u32)) != 0
    }
    /// plane between `band` and `band+1` open
    #[inline]
    pub fn plane_open(&self, band: u8) -> bool {
        band < 15 && self.planes & (1u16 << band) != 0
    }
}

/// Chunk-graph visibility flood from the camera's section.
///
/// Returns `None` when the camera's chunk has no GPU mesh yet (boot,
/// travel, extreme un-mesh) — the caller then skips occlusion culling
/// entirely (draws everything frustum-visible; conservative fallback).
/// Otherwise returns the set of drawable chunks: the camera's own column
/// plus every column with a reachable geometry band.
pub fn occlusion_visible(
    chunks: &HashMap<ChunkPos, ChunkGpu>,
    cam_chunk: ChunkPos,
    cam_band: u8,
) -> Option<std::collections::HashSet<ChunkPos>> {
    chunks.get(&cam_chunk)?;
    let mut seen: std::collections::HashSet<(ChunkPos, u8)> = Default::default();
    let mut queue: std::collections::VecDeque<(ChunkPos, u8)> = Default::default();
    seen.insert((cam_chunk, cam_band));
    queue.push_back((cam_chunk, cam_band));
    while let Some((p, b)) = queue.pop_front() {
        let Some(g) = chunks.get(&p) else { continue };
        // horizontal: same band, gated by this column's wall
        for (face, d) in [
            (FACE_PX, (1i32, 0i32)),
            (FACE_NX, (-1, 0)),
            (FACE_PZ, (0, 1)),
            (FACE_NZ, (0, -1)),
        ] {
            if g.occl.wall_open(b, face) {
                let n = (p.0 + d.0, p.1 + d.1);
                if chunks.contains_key(&n) && seen.insert((n, b)) {
                    queue.push_back((n, b));
                }
            }
        }
        // vertical: gated by the plane between the bands
        if g.occl.plane_open(b) && seen.insert((p, b + 1)) {
            queue.push_back((p, b + 1));
        }
        if b > 0 && g.occl.plane_open(b - 1) && seen.insert((p, b - 1)) {
            queue.push_back((p, b - 1));
        }
    }
    let mut vis: std::collections::HashSet<ChunkPos> = Default::default();
    vis.insert(cam_chunk); // the column you stand in is always drawn
    for (p, b) in seen {
        if let Some(g) = chunks.get(&p) {
            if g.occl.geo & (1u16 << b) != 0 {
                vis.insert(p);
            }
        }
    }
    Some(vis)
}

/// Frame-to-frame cache for [`occlusion_visible`] (§26 rendering-cost fix).
///
/// The flood's result depends ONLY on (camera chunk, camera band, the set
/// of meshed chunk columns) — none of which change while the camera stands
/// still in one section and no chunk uploads land. Yet the old path re-ran
/// the full BFS (two `HashSet`s + a `VecDeque`, `O(rd²·bands)` node visits)
/// EVERY frame — the single largest fixed CPU cost in `Renderer::render`
/// outside the draw submission itself.
///
/// The renderer owns a monotonically increasing `mesh_rev` counter bumped
/// by every mesh upload/removal/clear; passing it here lets a stale cache
/// be detected in one integer compare. `None` (camera chunk unmeshed) is
/// NOT cached — the conservative draw-everything fallback stays cheap.
#[derive(Default)]
pub struct OcclCache {
    key: (ChunkPos, u8, u64),
    vis: std::collections::HashSet<ChunkPos>,
}

/// cached flood: recomputes only when (cam chunk, band, mesh revision)
/// changes; returns the drawable set by reference (no per-frame clone).
/// The returned reference borrows the CACHE (the `chunks` borrow ends at
/// the call — the renderer's disjoint-field borrows of `self` stay valid).
pub fn occlusion_visible_cached<'a>(
    chunks: &HashMap<ChunkPos, ChunkGpu>,
    cam_chunk: ChunkPos,
    cam_band: u8,
    mesh_rev: u64,
    cache: &'a mut OcclCache,
) -> Option<&'a std::collections::HashSet<ChunkPos>> {
    if cache.key == (cam_chunk, cam_band, mesh_rev) && !cache.vis.is_empty() {
        return Some(&cache.vis);
    }
    let vis = occlusion_visible(chunks, cam_chunk, cam_band)?;
    cache.key = (cam_chunk, cam_band, mesh_rev);
    cache.vis = vis;
    Some(&cache.vis)
}

// ------------------------------------------------------------ draw lists --

/// one draw: where in the region arena the indices live and which
/// origin-instance row selects this chunk's world offset. Indices in the
/// arena are ABSOLUTE (baked with +v_off at upload) so base_vertex is
/// ALWAYS 0 — required by the WebGL2/GL backend, which cannot issue
/// instanced draws with a non-zero base vertex (glow panics); verified
/// empirically in browser E2E and against wgpu-hal 22 gles/queue.rs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawCmd {
    pub region: (i32, i32),
    pub i_first: u32,
    pub i_count: u32,
    pub origin: u32,
}

/// visible-chunk record for one frame: chunk position, squared
/// chunk-center distance, and its stable row in the origin instance buffer
pub type VisEntry = (ChunkPos, f32, u32);

/// Region-major ordering: sort by (region-center distance, region key,
/// chunk distance), near→far. The (dist, key) prefix is CONSTANT per region
/// — so every region's chunks form one contiguous block even when several
/// regions sit at the exact same distance (ring/symmetric layouts) → one
/// arena bind per region run — while the overall order stays roughly
/// front-to-back for early-z. `reverse` (water) flips to far→near (block
/// contiguity is preserved by reversal).
pub fn order_by_region(vis: &[VisEntry], cam: (f32, f32), reverse: bool) -> Vec<VisEntry> {
    let mut out = vis.to_vec();
    out.sort_by(|a, b| {
        let ra = region_of(a.0);
        let rb = region_of(b.0);
        let (ax, az) = region_center(ra);
        let (bx, bz) = region_center(rb);
        let da = (ax - cam.0) * (ax - cam.0) + (az - cam.1) * (az - cam.1);
        let db = (bx - cam.0) * (bx - cam.0) + (bz - cam.1) * (bz - cam.1);
        da.partial_cmp(&db)
            .unwrap_or(std::cmp::Ordering::Equal)
            // equal-distance regions must not interleave (contiguous runs!)
            .then(ra.cmp(&rb))
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    if reverse {
        out.reverse();
    }
    out
}

/// build the draw list for one pass from an (already ordered) visible set.
/// `water` selects the water slot; `max_dist2` filters (shadow radius);
/// missing/empty chunks are skipped exactly like the old inline loops.
pub fn build_draw_list(
    chunks: &HashMap<ChunkPos, ChunkGpu>,
    order: &[VisEntry],
    water: bool,
    max_dist2: Option<f32>,
) -> Vec<DrawCmd> {
    order
        .iter()
        .filter_map(|&(pos, dist2, origin)| {
            if let Some(md) = max_dist2 {
                if dist2 > md {
                    return None;
                }
            }
            let g = chunks.get(&pos)?;
            let s = if water { g.water.as_ref()? } else { &g.solid };
            if s.n == 0 {
                return None;
            }
            Some(DrawCmd {
                region: s.region,
                i_first: s.i_off,
                i_count: s.n,
                origin,
            })
        })
        .collect()
}

/// contiguous same-region runs: `(region, first_cmd_index, cmd_count)`
/// — one arena bind (loop path) or one multi-draw (MDI path) per run
pub fn region_runs(cmds: &[DrawCmd]) -> Vec<((i32, i32), usize, usize)> {
    let mut runs = Vec::new();
    let mut i = 0;
    while i < cmds.len() {
        let region = cmds[i].region;
        let start = i;
        while i < cmds.len() && cmds[i].region == region {
            i += 1;
        }
        runs.push((region, start, i - start));
    }
    runs
}

// -------------------------------------------------------- indirect args --

/// `wgpu::DrawIndexedIndirectArgs` mirror — repr(C), 20 bytes, no padding
/// (field-for-field identical to wgpu-types 22's struct; a local mirror
/// keeps this module GPU-free and gives us bytemuck Pod).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

impl IndirectArgs {
    pub const SIZE: u64 = std::mem::size_of::<IndirectArgs>() as u64;
}

/// pack draw commands 1:1 into indirect records (MDI path). Each record
/// draws one chunk: its absolute-index range in the region arena,
/// instance_count 1, first_instance = origin row, base_vertex 0
/// (indices are baked absolute — see DrawCmd).
pub fn pack_args(cmds: &[DrawCmd]) -> Vec<IndirectArgs> {
    cmds.iter()
        .map(|c| IndirectArgs {
            index_count: c.i_count,
            instance_count: 1,
            first_index: c.i_first,
            base_vertex: 0,
            first_instance: c.origin,
        })
        .collect()
}

/// validation shared by tests + the render integration: expanding the
/// packed records must reproduce the loop path's draws exactly
/// (same order, same ranges, same origins)
pub fn assert_args_match_loop(cmds: &[DrawCmd], args: &[IndirectArgs]) -> bool {
    if cmds.len() != args.len() {
        return false;
    }
    cmds.iter().zip(args.iter()).all(|(c, a)| {
        a.index_count == c.i_count
            && a.instance_count == 1
            && a.first_index == c.i_first
            && a.base_vertex == 0
            && a.first_instance == c.origin
    })
}

/// per-frame draw-prep accounting used by F3, the bench JSON and tests:
/// given the three pass lists and their region runs, how many buffer
/// binds and draw calls does each submission path issue?
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawCallAccounting {
    /// draw_indexed-family API calls per frame (all passes)
    pub draws: u32,
    /// set_vertex_buffer + set_index_buffer calls per frame
    pub binds: u32,
}

impl DrawCallAccounting {
    /// loop path: 1 whole-buffer origin bind + 2 arena binds per region run
    pub fn loop_path(terrain: &[DrawCmd], water: &[DrawCmd], shadow: &[DrawCmd]) -> Self {
        let runs = region_runs(terrain).len() + region_runs(water).len() + region_runs(shadow).len();
        DrawCallAccounting {
            draws: (terrain.len() + water.len() + shadow.len()) as u32,
            binds: 3 + (runs as u32) * 2,
        }
    }

    /// MDI path: one multi_draw per region run; still per-pass origin binds
    pub fn mdi_path(terrain: &[DrawCmd], water: &[DrawCmd], shadow: &[DrawCmd]) -> Self {
        let runs = region_runs(terrain).len() + region_runs(water).len() + region_runs(shadow).len();
        DrawCallAccounting {
            draws: runs as u32,
            binds: 3 + (runs as u32) * 2,
        }
    }

    /// the pre-P9 path for comparison: 3 slice binds per drawn chunk per
    /// pass + the draw itself (measured from the same lists)
    pub fn legacy(terrain: &[DrawCmd], water: &[DrawCmd], shadow: &[DrawCmd]) -> Self {
        let n = (terrain.len() + water.len() + shadow.len()) as u32;
        DrawCallAccounting {
            draws: n,
            binds: n * 3,
        }
    }
}

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(region: (i32, i32), v: u32, vc: u32, i: u32, ic: u32, n: u32) -> MeshSlot {
        MeshSlot { region, v_off: v, v_cap: vc, i_off: i, i_cap: ic, n }
    }

    #[test]
    fn region_math() {
        assert_eq!(region_of((0, 0)), (0, 0));
        assert_eq!(region_of((7, 7)), (0, 0));
        assert_eq!(region_of((8, 0)), (1, 0));
        assert_eq!(region_of((-1, -1)), (-1, -1));
        assert_eq!(region_of((-8, 15)), (-1, 1));
        let (cx, cz) = region_center((0, 0));
        assert!((cx - 64.0).abs() < 0.01 && (cz - 64.0).abs() < 0.01);
    }

    #[test]
    fn slot_alloc_basic_and_reuse() {
        let mut a = SlotAlloc::default();
        assert!(a.is_empty());
        let (o1, c1) = a.alloc(100);
        assert_eq!((o1, c1), (0, 100));
        let (o2, _) = a.alloc(50);
        assert_eq!(o2, 100); // bump, no overlap
        assert_eq!(a.live_count(), 2);
        a.release(o1, c1);
        assert_eq!(a.live_count(), 1);
        assert!(!a.is_empty());
        // a released range satisfies new requests without bumping
        let (o3, c3) = a.alloc(80);
        assert_eq!((o3, c3), (0, 100)); // whole-range handout, cap preserved
        assert_eq!(a.used(), 150);
        assert_eq!(a.live_count(), 2);
        // zero-length request is the null slot
        assert_eq!(a.alloc(0), (0, 0));
        assert!(!a.is_empty());
        // releasing everything → dead (buffer destroyable) even though the
        // free pool still holds ranges and bump > 0
        a.release(o2, 50);
        a.release(o3, c3);
        assert!(a.is_empty());
    }

    #[test]
    fn slot_alloc_no_overlap_invariant() {
        // randomized-ish: alloc/free pattern, then re-alloc everything and
        // verify live ranges never overlap
        let mut a = SlotAlloc::default();
        let mut live: Vec<(u32, u32)> = Vec::new();
        for i in 0..64u32 {
            let len = 1 + (i % 13);
            let (o, c) = a.alloc(len);
            live.push((o, c));
            if i % 3 == 0 && live.len() > 2 {
                let (ro, rc) = live.swap_remove(0);
                a.release(ro, rc);
            }
        }
        let mut sorted = live.clone();
        sorted.sort();
        for w in sorted.windows(2) {
            let (o1, c1) = w[0];
            let (o2, _) = w[1];
            assert!(o1 + c1 <= o2, "overlap: {w:?}");
        }
    }

    #[test]
    fn grow_plan_doubles() {
        assert_eq!(grow_plan(100, 0), 4096);
        assert_eq!(grow_plan(5000, 4096), 8192);
        assert_eq!(grow_plan(4096, 4096), 4096);
        assert!(grow_plan(u32::MAX - 1, 4096) >= u32::MAX - 1);
    }

    #[test]
    fn order_groups_regions_contiguously() {
        // camera at origin; three regions; interleaved chunk distances
        let vis: Vec<VisEntry> = [
            ((0, 0), 1.0, 0),
            ((9, 9), 2.0, 1),
            ((1, 0), 3.0, 2),
            ((8, 8), 4.0, 3),
            ((-8, 0), 5.0, 4),
            ((10, 10), 6.0, 5),
        ]
        .to_vec();
        let ordered = order_by_region(&vis, (8.0, 8.0), false);
        // region (0,0) holds (0,0),(1,0); region (1,1) holds (8,8),(9,9),(10,10);
        // every region's chunks must form ONE contiguous run
        let idx = |p: ChunkPos| ordered.iter().position(|v| v.0 == p).unwrap() as i32;
        assert!((idx((0, 0)) - idx((1, 0))).abs() == 1); // both region-(0,0), adjacent
        let r11 = [idx((8, 8)), idx((9, 9)), idx((10, 10))];
        assert_eq!(*r11.iter().max().unwrap() - *r11.iter().min().unwrap(), 2); // contiguous triple
        // no other region's chunk interleaves into that span
        for v in ordered.iter() {
            let i = idx(v.0);
            if r11.contains(&i) {
                assert_eq!(region_of(v.0), (1, 1));
            }
        }
        // near region (0,0) before far region (-1,0)
        assert!(idx((0, 0)) < idx((-8, 0)));
        // reversal flips everything
        let rev = order_by_region(&vis, (8.0, 8.0), true);
        assert_eq!(rev.last().unwrap().0, ordered.first().unwrap().0);
    }

    #[test]
    fn draw_list_and_runs() {
        let mut chunks = HashMap::new();
        chunks.insert(
            (0, 0),
            ChunkGpu { solid: slot((0, 0), 0, 10, 0, 12, 12), water: None, occl: Default::default() },
        );
        chunks.insert(
            (1, 0),
            ChunkGpu {
                solid: slot((0, 0), 10, 10, 12, 6, 6),
                water: Some(slot((0, 0), 20, 10, 18, 9, 9)),
                occl: Default::default(),
            },
        );
        chunks.insert(
            (8, 8),
            ChunkGpu { solid: slot((1, 1), 0, 10, 0, 4, 4), water: None, occl: Default::default() },
        );
        chunks.insert(
            (64, 64),
            ChunkGpu { solid: MeshSlot::EMPTY, water: None, occl: Default::default() }, // empty → skipped
        );
        let vis: Vec<VisEntry> =
            vec![((0, 0), 1.0, 0), ((1, 0), 2.0, 1), ((8, 8), 3.0, 2), ((64, 64), 4.0, 3)];

        let t = build_draw_list(&chunks, &vis, false, None);
        assert_eq!(t.len(), 3);
        assert_eq!(t[0], DrawCmd { region: (0, 0), i_first: 0, i_count: 12, origin: 0 });
        assert_eq!(t[1], DrawCmd { region: (0, 0), i_first: 12, i_count: 6, origin: 1 });
        assert_eq!(t[2], DrawCmd { region: (1, 1), i_first: 0, i_count: 4, origin: 2 });

        // water only where a water slot exists
        let w = build_draw_list(&chunks, &vis, true, None);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].i_first, 18);

        // distance filter (shadow radius): dist² ≤ 5.0 keeps the three
        // nearest (1.0, 2.0, 3.0) — (64,64) at 4.0 is EMPTY-skipped anyway
        let s = build_draw_list(&chunks, &vis, false, Some(5.0));
        assert_eq!(s.len(), 3);

        // runs: (0,0) run of 2 then (1,1) run of 1
        let runs = region_runs(&t);
        assert_eq!(runs, vec![((0, 0), 0, 2), ((1, 1), 2, 1)]);

        // accounting: loop = 6 draws + 3 origin binds + 2 binds per region run
        // runs: terrain 2, water 1, shadow 2 → 3 + 2*5 = 13 binds
        let acc = DrawCallAccounting::loop_path(&t, &w, &s);
        assert_eq!(acc.draws, 3 + 1 + 3);
        assert_eq!(acc.binds as usize, 3 + 2 * (2 + 1 + 2));
        // legacy would have been 3 binds per drawn chunk
        let leg = DrawCallAccounting::legacy(&t, &w, &s);
        assert_eq!(leg.binds, 7 * 3);
    }

    #[test]
    fn args_packing_matches_loop() {
        let cmds = vec![
            DrawCmd { region: (0, 0), i_first: 12, i_count: 6, origin: 4 },
            DrawCmd { region: (0, 0), i_first: 40, i_count: 90, origin: 17 },
            DrawCmd { region: (2, -3), i_first: 0, i_count: 3, origin: 2047 },
        ];
        let args = pack_args(&cmds);
        assert_eq!(std::mem::size_of::<IndirectArgs>(), 20);
        assert!(assert_args_match_loop(&cmds, &args));
        // a corrupted record must fail the equivalence check
        let mut bad = args.clone();
        bad[1].first_index += 1;
        assert!(!assert_args_match_loop(&cmds, &bad));
    }

    // ------------------------------------------- Phase 6 §26: occlusion --

    /// helper: one column with all walls/planes open in a set of bands and
    /// geometry in a set of bands
    fn col(bands: u16, geo_bands: u16) -> ChunkGpu {
        let mut sides = 0u64;
        for b in 0..16u32 {
            if bands & (1 << b) != 0 {
                for f in 0..4u32 {
                    sides |= 1 << (b * 4 + f);
                }
            }
        }
        let mut planes = 0u16;
        for s in 0..15u16 {
            // a band's airspace connects vertically through its ceiling
            // plane when the band (or its neighbor band) is open
            if (bands & (1 << s) != 0) && (bands & (1 << (s + 1)) != 0) {
                planes |= 1 << s;
            }
        }
        ChunkGpu {
            solid: slot((0, 0), 0, 8, 0, 6, 6),
            water: None,
            occl: ChunkOccl { sides, planes, geo: geo_bands },
        }
    }

    /// surface world: camera on the surface (band 5), air bands 4..8 all
    /// interconnected across columns, geometry at band 4 — every column
    /// with a surface band is visible
    #[test]
    fn occlusion_surface_world_sees_all_surfaces() {
        let mut chunks = HashMap::new();
        for dz in -2..=2 {
            for dx in -2..=2 {
                chunks.insert((dx, dz), col(0b1111111_0, 1 << 4)); // bands 4..10 air, geo band 4
            }
        }
        let vis = occlusion_visible(&chunks, (0, 0), 5).unwrap();
        for dz in -2..=2 {
            for dx in -2..=2 {
                assert!(vis.contains(&(dx, dz)), "surface chunk ({dx},{dz}) culled");
            }
        }
    }

    /// sealed cave: geometry bands 1..2 under a stone ceiling (plane
    /// closed), walls closed — the column is culled even though its upper
    /// air bands would be open (no geo up there → nothing worth drawing
    /// below the sealed ceiling). The camera's column is always drawn.
    #[test]
    fn occlusion_culls_sealed_cave_column() {
        let mut chunks = HashMap::new();
        // camera column: air bands 3..7 (walls + planes open), geo at 4
        chunks.insert((0, 0), col(0b1111100_0, 1 << 4));
        // neighbor columns: air bands 3..7 open (connected), but their geo
        // band 1 has CLOSED walls and the plane above it (band 1→2) closed:
        // no path reaches it → culled
        for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut g = col(0b1111100_0, 1 << 1); // geo deep down
            g.occl.sides &= !0xF; // close band-0..: only band 1 walls closed
            // close ALL walls of band 1 and the plane between band 1/2
            g.occl.sides &= !(0xF << 4);
            g.occl.planes &= !(1 << 1);
            chunks.insert(d, g);
        }
        let vis = occlusion_visible(&chunks, (0, 0), 5).unwrap();
        assert!(vis.contains(&(0, 0)));
        for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            assert!(!vis.contains(&d), "sealed cave column {d:?} not culled");
        }
    }

    /// tunnel: the neighbor's deep geo band has ONE open wall cell (the
    /// tunnel mouth in the shared wall) — the flood walks through the wall
    /// via that band → the column is drawn
    #[test]
    fn occlusion_keeps_tunnel_connected_cave() {
        let mut chunks = HashMap::new();
        chunks.insert((0, 0), col(0b1111100_0, 1 << 4));
        let mut g = col(0b1111100_0, 1 << 1);
        g.occl.sides &= !(0xF << 4); // band-1 walls all closed…
        g.occl.sides |= 1 << (1 * 4 + FACE_NX); // …except the wall facing the camera
        g.occl.planes &= !(1 << 1);
        chunks.insert((1, 0), g);
        // (1,0)'s open band set has no band 1 → the flood must reach band 1
        // through the camera column: camera band 5 → down through planes →
        // band 1 → wall open → (1,0) band 1. The camera column's planes must
        // therefore reach band 1: give it open bands 1..7.
        chunks.get_mut(&(0, 0)).unwrap().occl = {
            let mut c = col(0b1111111_0, 1 << 4).occl;
            c.planes |= 0b11_1111_11; // open planes down to band 0
            c
        };
        let vis = occlusion_visible(&chunks, (0, 0), 5).unwrap();
        assert!(vis.contains(&(1, 0)), "tunnel-visible cave column culled");
    }

    /// missing camera chunk → None (caller must skip occlusion culling)
    #[test]
    fn occlusion_camera_chunk_missing_is_none() {
        let mut chunks = HashMap::new();
        chunks.insert((1, 0), col(0b1111111_0, 1 << 4));
        assert!(occlusion_visible(&chunks, (0, 0), 5).is_none());
    }

    /// bit helpers agree with the raw fields
    #[test]
    fn occl_bit_helpers() {
        let o = ChunkOccl { sides: 1 << (3 * 4 + FACE_PZ), planes: 1 << 7, geo: 1 << 9 };
        assert!(o.wall_open(3, FACE_PZ));
        assert!(!o.wall_open(3, FACE_PX));
        assert!(!o.wall_open(4, FACE_PZ));
        assert!(o.plane_open(7));
        assert!(!o.plane_open(6));
        assert!(!o.plane_open(15), "no plane above band 15");
    }

    /// §26 rendering-cost fix: the cached flood returns EXACTLY what a
    /// direct flood returns for the same (camera, mesh set), a same-key
    /// repeat call hits the cache (identical set, by reference), and a
    /// mesh-revision bump after the world changes recomputes.
    #[test]
    fn occlusion_cache_hits_and_invalidates() {
        let mut chunks = HashMap::new();
        for dz in -2..=2 {
            for dx in -2..=2 {
                chunks.insert((dx, dz), col(0b1111111_0, 1 << 4));
            }
        }
        let mut cache = OcclCache::default();
        // rev 1: first call computes and stores
        let direct = occlusion_visible(&chunks, (0, 0), 5).unwrap();
        assert!(direct.contains(&(1, 0)));
        let cached = occlusion_visible_cached(&chunks, (0, 0), 5, 1, &mut cache).unwrap();
        assert_eq!(direct, *cached, "cached flood must equal direct flood");
        // same key: cache hit — identical set
        let again = occlusion_visible_cached(&chunks, (0, 0), 5, 1, &mut cache).unwrap();
        assert_eq!(*again, direct);
        // default-constructed state (key ((0,0),0,0), empty set): the
        // is_empty guard must fall through to a real compute, never serve
        // the empty default set as a hit
        let mut cold = OcclCache::default();
        let r = occlusion_visible_cached(&chunks, (0, 0), 0, 0, &mut cold);
        assert!(r.is_some(), "default-state key must fall through to compute");
        assert!(r.unwrap().contains(&(0, 0)));
        // mesh revision bump (a chunk remeshed with no drawable faces):
        // recompute and match the fresh direct flood
        chunks.get_mut(&(1, 0)).unwrap().occl.geo = 0;
        let after = occlusion_visible_cached(&chunks, (0, 0), 5, 2, &mut cache).unwrap();
        assert_eq!(*after, occlusion_visible(&chunks, (0, 0), 5).unwrap());
        assert!(
            !after.contains(&(1, 0)),
            "geometry-less column must be culled after invalidation"
        );
    }

    /// missing camera chunk → None is never cached as a result (the caller
    /// retries next frame — the conservative fallback stays cheap)
    #[test]
    fn occlusion_cache_none_is_transient() {
        let mut chunks = HashMap::new();
        let mut cache = OcclCache::default();
        assert!(occlusion_visible_cached(&chunks, (0, 0), 5, 1, &mut cache).is_none());
        // camera chunk arrives (mesh upload → rev 2)
        chunks.insert((0, 0), col(0b1111111_0, 1 << 4));
        let r = occlusion_visible_cached(&chunks, (0, 0), 5, 2, &mut cache);
        assert!(r.is_some(), "None state must not stick");
    }
}
