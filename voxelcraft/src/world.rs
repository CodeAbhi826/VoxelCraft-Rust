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

pub struct World {
    pub seed: u64,
    pub gen: TerrainGen,
    pub chunks: HashMap<ChunkPos, Arc<Chunk>>,
    /// chunks fully generated + decorated (meshable)
    pub decorated: HashSet<ChunkPos>,
    /// edits queued for not-yet-generated chunks: (block_idx, id)
    pub pending: HashMap<ChunkPos, Vec<(u16, u8)>>,
    /// chunks whose mesh is stale (block edit / light change)
    pub dirty: HashSet<ChunkPos>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        World {
            seed,
            gen: TerrainGen::new(seed),
            chunks: HashMap::new(),
            decorated: HashSet::new(),
            pending: HashMap::new(),
            dirty: HashSet::new(),
        }
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
    /// Copy-on-write at section granularity; marks affected chunks dirty.
    pub fn set_block_state(&mut self, wx: i32, wy: i32, wz: i32, state: u16) {
        if wy < 0 || wy > 255 {
            return;
        }
        let cx = wx.div_euclid(16);
        let cz = wz.div_euclid(16);
        let pos = (cx, cz);
        let Some(old) = self.chunks.get(&pos) else { return };
        let old = Arc::clone(old);
        let mut new_chunk = (*old).clone();
        let lx = (wx - cx * 16) as usize;
        let lz = (wz - cz * 16) as usize;
        new_chunk.set_state(lx, wy as usize, lz, state);
        self.chunks.insert(pos, Arc::new(new_chunk));

        self.dirty.insert(pos);
        // border blocks change neighbor face culling + light
        let mut touch = |dx: i32, dz: i32| {
            self.dirty.insert((cx + dx, cz + dz));
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
                self.dirty.insert(pos);
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
    pub fn insert_generated(&mut self, pos: ChunkPos, chunk: Arc<Chunk>, outbound: Vec<(i32, i32, i32, u8)>) {
        self.chunks.insert(pos, chunk);
        self.decorated.insert(pos);
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
