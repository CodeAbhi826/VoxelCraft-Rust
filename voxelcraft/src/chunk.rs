//! Chunk storage: 16 x 256 x 16 block column, u8 block ids.

pub const CHUNK_LEN: usize = 16 * 256 * 16; // 65536

#[inline]
pub fn idx(x: usize, y: usize, z: usize) -> usize {
    (y << 8) | (z << 4) | x
}

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Box<[u8; CHUNK_LEN]>,
    /// surface height (topmost terrain block y) per column
    pub height: Box<[u8; 256]>,
    /// Biome id per column
    pub biome: Box<[u8; 256]>,
}

impl Chunk {
    pub fn empty() -> Self {
        Chunk {
            blocks: Box::new([0u8; CHUNK_LEN]),
            height: Box::new([0u8; 256]),
            biome: Box::new([0u8; 256]),
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.blocks[idx(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, id: u8) {
        self.blocks[idx(x, y, z)] = id;
    }

    /// Topmost solid block y (for spawn placement). -1 if none.
    pub fn top_solid_y(&self, x: usize, z: usize) -> i32 {
        for y in (0..256usize).rev() {
            if crate::blocks::is_solid(self.get(x, y, z)) {
                return y as i32;
            }
        }
        -1
    }
}
