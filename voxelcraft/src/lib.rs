//! VoxelCraft — high-performance Minecraft-1.16.5-style voxel engine.
//! Rust + wgpu. One codebase → native (Vulkan/DX12/Metal) + WASM (WebGPU).

pub mod bench;
pub mod blocks;
pub mod chunk;
pub mod game;
pub mod gen;
#[cfg(not(target_arch = "wasm32"))]
pub mod anvil;
pub mod mesh;
pub mod model;
pub mod nbt;
pub mod pack;
pub mod player;
pub mod render;
pub mod rng;
#[cfg(not(target_arch = "wasm32"))]
pub mod save;
pub mod sounds;
pub mod textures;
pub mod ui;
pub mod world;

pub const CHUNK_X: usize = 16;
pub const CHUNK_Y: usize = 256;
pub const CHUNK_Z: usize = 16;
pub const SEA_LEVEL: i32 = 62;

#[cfg(target_arch = "wasm32")]
pub mod wasm_entry;
#[cfg(target_arch = "wasm32")]
pub mod web_input;
