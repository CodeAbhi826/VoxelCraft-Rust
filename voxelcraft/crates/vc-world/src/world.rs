//! World: chunk map, block access, copy-on-write edits (thread-safe meshing),
//! pending cross-chunk decoration edits.

use crate::gen::TerrainGen;
use crate::light::LightData;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use vc_blocks::blocks::*;
use vc_chunk::chunk::Chunk;
use vc_rng::rng::Rng;

pub type ChunkPos = (i32, i32);

/// §28 world family: which dimension a World generates. Each dimension
/// derives its own generator instance from the shared world seed (the
/// vanilla pattern: same seed, per-dimension generators), so chunk content
/// is independent per dimension and deterministic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dimension {
    Overworld = 0,
    /// the Nether: 8:1 coordinate scale, caverns, no skylight
    Nether = 1,
}

impl Dimension {
    /// registry-style identifier (F3 parity: "minecraft:overworld")
    pub fn id(self) -> &'static str {
        match self {
            Dimension::Overworld => "overworld",
            Dimension::Nether => "the_nether",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Dimension::Overworld => "Overworld",
            Dimension::Nether => "Nether",
        }
    }

    pub fn from_u8(v: u8) -> Dimension {
        if v == 1 {
            Dimension::Nether
        } else {
            Dimension::Overworld
        }
    }

    /// horizontal block-per-block scale of this dimension relative to the
    /// overworld (vanilla nether travel: 8 overworld blocks = 1 nether block)
    pub fn coord_scale(self) -> i32 {
        match self {
            Dimension::Overworld => 1,
            Dimension::Nether => 8,
        }
    }

    /// seed salt mixed into the per-dimension generator seed
    pub fn seed_salt(self) -> u64 {
        match self {
            Dimension::Overworld => 0,
            Dimension::Nether => 0x1DE1_1E77_0D1D_1234,
        }
    }

    /// map a horizontal position from `self` into `other` (travel scaling)
    pub fn map_coords(self, other: Dimension, x: i32, z: i32) -> (i32, i32) {
        let from = self.coord_scale();
        let to = other.coord_scale();
        // x is in `self` units; convert to shared units then to `other`
        let shared = x as i64 * from as i64;
        let out = shared / to as i64;
        (out as i32, (z as i64 * from as i64 / to as i64) as i32)
    }
}

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

/// material class for §12 cause accounting: cutout/transparent vs solid
#[inline]
fn is_transparent_class(b: u8) -> bool {
    !is_opaque(b) || b == WATER
}

pub struct World {
    pub seed: u64,
    /// §28: which dimension this world generates/stores — travel swaps the
    /// whole World (fresh generator + chunk maps), vanilla-style
    pub dimension: Dimension,
    pub gen: TerrainGen,
    pub chunks: HashMap<ChunkPos, Arc<Chunk>>,
    /// persistent per-chunk light (Phase 4 §18): sky + block channels,
    /// updated incrementally by light::LightEngine, snapshotted for mesh
    /// jobs exactly like the block data (Arc COW)
    pub light: HashMap<ChunkPos, Arc<LightData>>,
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
    /// overworld world (back-compatible)
    pub fn new(seed: u64) -> Self {
        World::new_in_dimension(seed, Dimension::Overworld)
    }

    /// §28: world in a specific dimension — the generator seed derives from
    /// the shared world seed + the dimension salt
    pub fn new_in_dimension(seed: u64, dim: Dimension) -> Self {
        World {
            seed,
            dimension: dim,
            gen: TerrainGen::for_dimension(seed, dim),
            chunks: HashMap::new(),
            light: HashMap::new(),
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
                c.get(lx, wy as usize, lz) // already folded by Chunk::get
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
                c.get_state(lx, wy as usize, lz) // raw, never truncated
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
    /// Returns (old_state, new_state) for the light engine when the edit
    /// landed (Phase 4), None otherwise.
    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, id: u8) -> Option<(u16, u16)> {
        self.set_block_state(wx, wy, wz, id as u16)
    }

    /// Player-driven BLOCK-STATE edit (e.g. a log placed with axis=x).
    /// Copy-on-write at section granularity; marks the affected SECTIONS
    /// dirty (§12 geometry region — the light engine marks light regions).
    /// Returns (old_state, new_state).
    pub fn set_block_state(&mut self, wx: i32, wy: i32, wz: i32, state: u16) -> Option<(u16, u16)> {
        if wy < 0 || wy > 255 {
            return None;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let pos = (cx, cz);
        let Some(old) = self.chunks.get(&pos) else {
            return None;
        };
        let old = Arc::clone(old);
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        let old_state = old.get_state(lx, wy as usize, lz);
        if old_state == state {
            return None; // no-op edit (skip dirty churn)
        }
        let mut new_chunk = (*old).clone();
        new_chunk.set_state(lx, wy as usize, lz, state);
        self.chunks.insert(pos, Arc::new(new_chunk));

        self.save_dirty.insert(pos); // persist player edits (§28)
        self.mark_edit(wx, wy, wz, old_state, state);
        Some((old_state, state))
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

        // ---- light region (Phase 4): the incremental LightEngine marks the
        // EXACT changed sections via its `changed` map — no heuristics here.
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
                let idx = vc_chunk::chunk::idx(lx, wy as usize, lz) as u16;
                self.pending.entry(pos).or_default().push((idx, id));
            }
        } else {
            let lx = (wx - cx * 16) as usize;
            let lz = (wz - cz * 16) as usize;
            let idx = vc_chunk::chunk::idx(lx, wy as usize, lz) as u16;
            self.pending.entry(pos).or_default().push((idx, id));
        }
    }

    /// Insert a freshly generated chunk + apply outbound decorations.
    /// Newly generated content has never been saved → save-dirty.
    pub fn insert_generated(
        &mut self,
        pos: ChunkPos,
        chunk: Arc<Chunk>,
        outbound: Vec<(i32, i32, i32, u8)>,
    ) {
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

    /// Snapshot the 3×3 LIGHT neighborhood for a mesh job (Phase 4).
    pub fn snapshot3x3_light(&self, cx: i32, cz: i32) -> Option<[Option<Arc<LightData>>; 9]> {
        let mut snap: [Option<Arc<LightData>>; 9] = Default::default();
        let mut i = 0;
        for dz in -1..=1 {
            for dx in -1..=1 {
                let pos = (cx + dx, cz + dz);
                let c = self.chunks.get(&pos)?; // blocks must exist
                snap[i] = self.light.get(&pos).cloned();
                let _ = c;
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
        match self.dimension {
            Dimension::Overworld => self.gen.find_spawn(),
            Dimension::Nether => self.gen.find_nether_spawn(),
        }
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
        assert_eq!(
            *w.dirty_causes.get(&(0, 0)).unwrap(),
            CAUSE_GEOMETRY | CAUSE_MATERIAL
        );
    }

    /// §12 + §18: light regions now come from the incremental LightEngine's
    /// exact `changed` map — see light.rs::tests (differential gate +
    /// region coverage). These two cases (mid-air shadow column, glowstone
    /// ±15 box) are asserted there against the engine, not heuristics.

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
        assert_eq!(
            m & (1 << 7),
            1 << 7,
            "neighbor section 7 (y 112..127) must be dirty"
        );
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
        assert_eq!(
            mask,
            1 << 4,
            "geometry section 4 (y 64..79) only, got {mask:#b}"
        );
    }

    /// §12: a NO-OP edit (same block re-written) dirties NOTHING — the
    /// write is skipped entirely (no mesh churn on repeated edits).
    #[test]
    fn noop_edit_dirties_nothing() {
        let mut w = flat_world(70);
        assert!(w.set_block(8, 65, 8, STONE).is_none()); // STONE → STONE
        assert!(w.dirty.is_empty());
        assert!(w.dirty_causes.is_empty());
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
        assert_eq!(
            *w.dirty.get(&(0, 0)).unwrap(),
            1 << 9,
            "in-flight edit bit survives"
        );
        assert!(w.dirty_causes.contains_key(&(0, 0)));
        // final job clears the rest → both maps drop the chunk
        w.clear_dirty_mask((0, 0), 1 << 9);
        assert!(!w.dirty.contains_key(&(0, 0)));
        assert!(!w.dirty_causes.contains_key(&(0, 0)));
    }
}

#[cfg(test)]
mod dimension_tests {
    use super::*;

    /// §28: dimension construction — same world seed, dimension-salted
    /// generators, distinct chunk content
    #[test]
    fn dimension_construction() {
        let seed = 0xAB12_CD34;
        let a = World::new(seed);
        let b = World::new_in_dimension(seed, Dimension::Nether);
        assert_eq!(a.dimension, Dimension::Overworld);
        assert_eq!(b.dimension, Dimension::Nether);
        assert_eq!(a.seed, b.seed, "the world seed is shared across dimensions");
        // the generators derive different seeds
        assert_ne!(a.gen.seed, b.gen.seed);
        // and generate different terrain at the same chunk: thousands of
        // cells differ AND the nether's bedrock roof (y=127) contrasts with
        // the overworld's open sky — unambiguous even before counting
        let (ca, _) = a.gen.generate_chunk(0, 0, Vec::new());
        let (cb, _) = b.gen.generate_chunk(0, 0, Vec::new());
        let mut diff = 0usize;
        for i in 0..vc_chunk::chunk::CHUNK_LEN {
            if ca.get_idx(i) != cb.get_idx(i) {
                diff += 1;
            }
        }
        assert!(
            diff > 8_000,
            "dimensions must generate different terrain (diff={diff})"
        );
        for z in 0..16usize {
            for x in 0..16usize {
                let i = (127 << 8) | (z << 4) | x;
                assert_ne!(
                    ca.get_idx(i),
                    cb.get_idx(i),
                    "y=127: nether roof (bedrock) vs overworld sky"
                );
            }
        }
    }

    /// §28: the vanilla 8:1 coordinate rule both ways
    #[test]
    fn coordinate_mapping_8_to_1() {
        use Dimension::{Nether, Overworld};
        // overworld → nether divides by 8
        assert_eq!(Overworld.map_coords(Nether, 800, -1600), (100, -200));
        // nether → overworld multiplies by 8
        assert_eq!(Nether.map_coords(Overworld, 100, -200), (800, -1600));
        // rounding: not-multiples floor toward zero on the shared axis
        let (x, _) = Overworld.map_coords(Nether, 805, 0);
        assert_eq!(x, 100);
        // self-mapping is identity
        assert_eq!(Overworld.map_coords(Overworld, 123, -45), (123, -45));
        assert_eq!(Nether.map_coords(Nether, 123, -45), (123, -45));
    }

    /// §28: overworld worlds keep the legacy constructor behavior (same
    /// terrain as TerrainGen::new)
    #[test]
    fn overworld_new_matches_legacy_gen() {
        let w = World::new(777);
        let legacy = TerrainGen::new(777);
        let (a, _) = w.gen.generate_chunk(2, -3, Vec::new());
        let (b, _) = legacy.generate_chunk(2, -3, Vec::new());
        for i in 0..vc_chunk::chunk::CHUNK_LEN {
            assert_eq!(a.get_idx(i), b.get_idx(i));
        }
    }
}
