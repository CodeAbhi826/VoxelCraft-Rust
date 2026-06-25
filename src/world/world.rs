// World manager: owns all chunks, handles streaming around the player,
// async generation/meshing, and block get/set at world coordinates.
//
// THREADING MODEL:
// - Chunk generation runs on rayon's thread pool (uses all CPU cores)
// - Chunk meshing runs on rayon's thread pool (parallel)
// - Chunks are Arc<RwLock<Chunk>> — safe for concurrent read/write
// - GPU upload happens on the main thread (wgpu is not thread-safe)
// - The main thread polls drain_pending_meshes() each frame and uploads to GPU

use ahash::AHashMap;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::sync::Arc;

use crate::blocks::Block;
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::mesher::{ChunkMesh, MeshBuilder};
use crate::world::noise::WorldGenerator;

pub type ChunkArc = Arc<RwLock<Chunk>>;

pub struct World {
    pub chunks: RwLock<AHashMap<(i32, i32), ChunkArc>>,
    pub generator: WorldGenerator,
    pub pending_meshes: parking_lot::Mutex<Vec<PendingMesh>>,
    pub seed: u32,
}

pub struct PendingMesh {
    pub cx: i32,
    pub cz: i32,
    pub mesh: ChunkMesh,
    pub distance_sq: i64,  // for distance sorting (nearest first)
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: RwLock::new(AHashMap::new()),
            generator: WorldGenerator::new(seed),
            pending_meshes: parking_lot::Mutex::new(Vec::new()),
            seed,
        }
    }

    pub fn chunk_key(cx: i32, cz: i32) -> (i32, i32) { (cx, cz) }

    /// Get a chunk (or None) without loading it.
    pub fn get_chunk(&self, cx: i32, cz: i32) -> Option<ChunkArc> {
        self.chunks.read().get(&(cx, cz)).cloned()
    }

    /// Ensure a chunk exists and is generated. Returns the chunk.
    /// This is the synchronous path — used when a single chunk is needed immediately.
    pub fn ensure_chunk(&self, cx: i32, cz: i32) -> ChunkArc {
        // Fast path: already exists
        if let Some(c) = self.chunks.read().get(&(cx, cz)) {
            return c.clone();
        }
        // Slow path: create + generate
        let arc = {
            let mut chunks = self.chunks.write();
            // Double-check after acquiring write lock
            if let Some(c) = chunks.get(&(cx, cz)) {
                return c.clone();
            }
            let mut new_chunk = Chunk::new(cx, cz);
            self.generator.generate(&mut new_chunk);
            let arc = Arc::new(RwLock::new(new_chunk));
            chunks.insert((cx, cz), arc.clone());
            arc
        };
        arc
    }

    /// Ensure a chunk exists WITHOUT generating terrain (for manual block placement).
    pub fn ensure_chunk_empty(&self, cx: i32, cz: i32) -> ChunkArc {
        if let Some(c) = self.chunks.read().get(&(cx, cz)) {
            return c.clone();
        }
        let arc = {
            let mut chunks = self.chunks.write();
            if let Some(c) = chunks.get(&(cx, cz)) {
                return c.clone();
            }
            let new_chunk = Chunk::new(cx, cz);
            let arc = Arc::new(RwLock::new(new_chunk));
            chunks.insert((cx, cz), arc.clone());
            arc
        };
        arc
    }

    /// BATCH parallel chunk generation using rayon.
    /// Generates multiple chunks at once across all CPU cores.
    /// Call this from the main thread; it blocks until all chunks are generated.
    pub fn generate_chunks_parallel(&self, coords: &[(i32, i32)]) {
        // Filter out chunks that already exist
        let to_generate: Vec<(i32, i32)> = {
            let chunks = self.chunks.read();
            coords.iter()
                .filter(|(cx, cz)| !chunks.contains_key(&(*cx, *cz)))
                .copied()
                .collect()
        };

        if to_generate.is_empty() {
            return;
        }

        // Generate all chunks in parallel using rayon
        let generated: Vec<Chunk> = to_generate
            .par_iter()  // ← parallel iterator (uses all CPU cores)
            .map(|&(cx, cz)| {
                let mut chunk = Chunk::new(cx, cz);
                self.generator.generate(&mut chunk);
                chunk
            })
            .collect();

        // Insert into the map (single write lock)
        let mut chunks = self.chunks.write();
        for chunk in generated {
            let key = (chunk.cx, chunk.cz);
            if !chunks.contains_key(&key) {
                chunks.insert(key, Arc::new(RwLock::new(chunk)));
            }
        }
    }

    /// BATCH parallel chunk meshing using rayon.
    /// Meshes all dirty chunks in parallel and queues them for GPU upload.
    /// Call this from the main thread; it blocks until all meshes are built.
    pub fn build_meshes_parallel(&self, player_x: i32, player_z: i32, budget: usize) {
        // Collect dirty chunks with their distances
        let dirty_chunks: Vec<(i32, i32, i64)> = {
            let chunks = self.chunks.read();
            chunks.iter()
                .filter_map(|((cx, cz), arc)| {
                    let c = arc.read();
                    if c.dirty {
                        let dx = (cx * CHUNK_SIZE as i32 + 8) - player_x;
                        let dz = (cz * CHUNK_SIZE as i32 + 8) - player_z;
                        let dist_sq = (dx as i64) * (dx as i64) + (dz as i64) * (dz as i64);
                        Some((*cx, *cz, dist_sq))
                    } else {
                        None
                    }
                })
                .take(budget)
                .collect()
        };

        if dirty_chunks.is_empty() {
            return;
        }

        // Build meshes in parallel
        let meshes: Vec<Option<(i32, i32, i64, ChunkMesh)>> = dirty_chunks
            .par_iter()  // ← parallel meshing across CPU cores
            .map(|&(cx, cz, dist_sq)| {
                if let Some(chunk) = self.get_chunk(cx, cz) {
                    let chunk = chunk.read();
                    let mesh = MeshBuilder::build(&chunk, self);
                    Some((cx, cz, dist_sq, mesh))
                } else {
                    None
                }
            })
            .collect();

        // Queue for upload (sorted by distance — nearest first)
        let mut pending = self.pending_meshes.lock();
        for (cx, cz, dist_sq, mesh) in meshes.into_iter().flatten() {
            pending.push(PendingMesh { cx, cz, mesh, distance_sq: dist_sq });
        }
        // Sort nearest-first so close chunks upload before far ones
        pending.sort_by_key(|p| p.distance_sq);

        // Mark chunks as no longer dirty
        for (cx, cz, _) in &dirty_chunks {
            if let Some(arc) = self.get_chunk(*cx, *cz) {
                arc.write().dirty = false;
            }
        }
    }

    /// Get a block at world coordinates. Returns Air if chunk not loaded.
    pub fn get_block(&self, wx: i32, wy: i32, wz: i32) -> Block {
        if wy < 0 || wy >= crate::world::chunk::WORLD_HEIGHT as i32 {
            return Block::Air;
        }
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cz = wz.div_euclid(CHUNK_SIZE as i32);
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;
        if let Some(chunk) = self.get_chunk(cx, cz) {
            let chunk = chunk.read();
            chunk.get(lx, wy as usize, lz)
        } else {
            Block::Air
        }
    }

    /// Set a block at world coordinates. Loads the chunk if needed.
    /// Marks neighbor chunks dirty if on a border.
    pub fn set_block(&self, wx: i32, wy: i32, wz: i32, block: Block) -> bool {
        if wy < 0 || wy >= crate::world::chunk::WORLD_HEIGHT as i32 {
            return false;
        }
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cz = wz.div_euclid(CHUNK_SIZE as i32);
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        let chunk = self.ensure_chunk_empty(cx, cz);
        {
            let mut c = chunk.write();
            c.set(lx, wy as usize, lz, block);
        }

        // Mark neighbors dirty if on border
        let mut neighbors = Vec::new();
        if lx == 0 { neighbors.push((cx - 1, cz)); }
        if lx == CHUNK_SIZE - 1 { neighbors.push((cx + 1, cz)); }
        if lz == 0 { neighbors.push((cx, cz - 1)); }
        if lz == CHUNK_SIZE - 1 { neighbors.push((cx, cz + 1)); }
        for (nx, nz) in neighbors {
            if let Some(n) = self.get_chunk(nx, nz) {
                n.write().dirty = true;
            }
        }
        true
    }

    /// Update which chunks should be loaded based on player position.
    /// Loads chunks in a radius around the player, unloads far ones.
    /// Uses parallel generation for the new chunks.
    pub fn update_player_position(&self, px: i32, pz: i32, render_distance: i32) {
        let pcx = px.div_euclid(CHUNK_SIZE as i32);
        let pcz = pz.div_euclid(CHUNK_SIZE as i32);

        // Collect chunks to load (nearest first)
        let mut to_load = Vec::new();
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let cx = pcx + dx;
                let cz = pcz + dz;
                if !self.get_chunk(cx, cz).is_some() {
                    let dist_sq = (dx * dx + dz * dz) as i64;
                    to_load.push((cx, cz, dist_sq));
                }
            }
        }
        to_load.sort_by_key(|t| t.2);

        // Generate in parallel (limit per-frame to avoid spikes)
        let batch: Vec<(i32, i32)> = to_load.iter()
            .take(8)  // max 8 new chunks per frame
            .map(|&(cx, cz, _)| (cx, cz))
            .collect();
        self.generate_chunks_parallel(&batch);

        // Unload far chunks
        let mut to_remove = Vec::new();
        {
            let chunks = self.chunks.read();
            for (key, _) in chunks.iter() {
                let dx = key.0 - pcx;
                let dz = key.1 - pcz;
                if dx.abs() > render_distance + 1 || dz.abs() > render_distance + 1 {
                    to_remove.push(*key);
                }
            }
        }
        if !to_remove.is_empty() {
            let mut chunks = self.chunks.write();
            for key in to_remove {
                chunks.remove(&key);
            }
        }
    }

    /// Build mesh for a single chunk (synchronous path).
    pub fn build_mesh(&self, cx: i32, cz: i32) -> Option<ChunkMesh> {
        let chunk = self.get_chunk(cx, cz)?;
        let chunk = chunk.read();
        if !chunk.dirty {
            return None;
        }
        let mesh = MeshBuilder::build(&chunk, self);
        Some(mesh)
    }

    /// Queue a mesh for upload to GPU.
    pub fn queue_mesh(&self, cx: i32, cz: i32, mesh: ChunkMesh) {
        self.pending_meshes.lock().push(PendingMesh { cx, cz, mesh, distance_sq: 0 });
    }

    /// Drain pending meshes for GPU upload (sorted nearest-first).
    pub fn drain_pending_meshes(&self) -> Vec<PendingMesh> {
        let mut pending = std::mem::take(&mut *self.pending_meshes.lock());
        pending.sort_by_key(|p| p.distance_sq);
        pending
    }

    /// Count loaded chunks and how many are ready (not dirty).
    pub fn chunk_stats(&self) -> (usize, usize) {
        let chunks = self.chunks.read();
        let total = chunks.len();
        let ready = chunks.values().filter(|c| !c.read().dirty).count();
        (total, ready)
    }

    /// Get the number of CPU cores available for parallel work.
    pub fn cpu_cores() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_block_set_get_roundtrip() {
        let world = World::new(42);
        world.set_block(5, 10, 7, Block::Stone);
        assert_eq!(world.get_block(5, 10, 7), Block::Stone);
        assert_eq!(world.get_block(5, 10, 8), Block::Air);
    }

    #[test]
    fn world_handles_negative_coords() {
        let world = World::new(42);
        world.set_block(-3, 5, -7, Block::Dirt);
        assert_eq!(world.get_block(-3, 5, -7), Block::Dirt);
    }

    #[test]
    fn world_chunk_streaming() {
        let world = World::new(42);
        world.update_player_position(0, 0, 3);
        // Should have loaded some chunks
        let (total, _) = world.chunk_stats();
        assert!(total > 0, "no chunks loaded");
    }

    #[test]
    fn parallel_chunk_generation_is_correct() {
        let world = World::new(1337);
        let coords: Vec<(i32, i32)> = (0..4).flat_map(|x| (0..4).map(move |z| (x, z))).collect();
        world.generate_chunks_parallel(&coords);

        // All chunks should exist and have terrain
        for (cx, cz) in &coords {
            let chunk = world.get_chunk(*cx, *cz).expect("chunk should exist");
            let chunk = chunk.read();
            let solid = chunk.blocks.iter().filter(|b| !b.is_air()).count();
            assert!(solid > 0, "chunk ({},{}) is empty", cx, cz);
        }
    }

    #[test]
    fn parallel_meshing_produces_valid_meshes() {
        let world = World::new(1337);
        // Generate some chunks first
        let coords: Vec<(i32, i32)> = (0..3).flat_map(|x| (0..3).map(move |z| (x, z))).collect();
        world.generate_chunks_parallel(&coords);

        // Build meshes in parallel
        world.build_meshes_parallel(0, 0, 10);

        // Should have pending meshes
        let pending = world.drain_pending_meshes();
        assert!(!pending.is_empty(), "no meshes built");
        for p in &pending {
            assert!(!p.mesh.solid_vertices.is_empty() || !p.mesh.transparent_vertices.is_empty(),
                    "mesh at ({},{}) is empty", p.cx, p.cz);
        }
    }

    #[test]
    fn cpu_cores_detected() {
        let cores = World::cpu_cores();
        assert!(cores >= 1, "should detect at least 1 CPU core");
        println!("Detected {} CPU cores", cores);
    }
}
