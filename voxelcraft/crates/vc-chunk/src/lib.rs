//! vc-chunk — chunk & section storage (§6): 16x256x16 chunks,
//! paletted section containers, heightmaps, world-grid constants.

pub mod chunk;
pub use chunk::*;

pub const CHUNK_X: usize = 16;
pub const CHUNK_Y: usize = 256;
pub const CHUNK_Z: usize = 16;
pub const SEA_LEVEL: i32 = 62;
