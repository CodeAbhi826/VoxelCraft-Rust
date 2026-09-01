// Voxel chunk: 16 x 64 x 16 column of blocks.
// Stored as a flat Vec<Block> for cache-friendly iteration.

use crate::blocks::Block;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 64;
pub const SEA_LEVEL: usize = 22;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * WORLD_HEIGHT;

#[derive(Debug)]
pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    pub blocks: Vec<Block>,
    pub min_y: usize,
    pub max_y: usize,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(cx: i32, cz: i32) -> Self {
        Self {
            cx,
            cz,
            blocks: vec![Block::Air; CHUNK_VOLUME],
            min_y: WORLD_HEIGHT,
            max_y: 0,
            dirty: true,
        }
    }

    /// Convert (x, y, z) within the chunk to a flat index.
    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        if y >= WORLD_HEIGHT {
            return Block::Air;
        }
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if y >= WORLD_HEIGHT {
            return;
        }
        let idx = Self::index(x, y, z);
        self.blocks[idx] = block;
        if block != Block::Air {
            if y < self.min_y { self.min_y = y; }
            if y > self.max_y { self.max_y = y; }
        }
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_index_within_bounds() {
        assert!(Chunk::index(0, 0, 0) < CHUNK_VOLUME);
        assert!(Chunk::index(15, 63, 15) < CHUNK_VOLUME);
    }

    #[test]
    fn chunk_set_get_roundtrip() {
        let mut c = Chunk::new(0, 0);
        c.set(5, 10, 7, Block::Stone);
        assert_eq!(c.get(5, 10, 7), Block::Stone);
        assert_eq!(c.get(5, 10, 8), Block::Air);
    }

    #[test]
    fn chunk_y_out_of_bounds_returns_air() {
        let c = Chunk::new(0, 0);
        assert_eq!(c.get(0, 100, 0), Block::Air);
    }

    #[test]
    fn chunk_dirty_flag_on_set() {
        let mut c = Chunk::new(0, 0);
        assert!(c.dirty);
        c.dirty = false;
        c.set(0, 0, 0, Block::Dirt);
        assert!(c.dirty);
    }
}
