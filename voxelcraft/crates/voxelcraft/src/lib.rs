//! VoxelCraft — high-performance Minecraft-1.16.5-style voxel game.
//! Rust + wgpu. One codebase -> native (Vulkan/DX12/Metal) + WASM (WebGPU).
//!
//! This crate is the APPLICATION/game shell; the reusable engine lives in
//! the `vc-*` library crates of this workspace (see LIBRARIES.md).

pub mod bench;
pub mod game;
pub mod player;
#[cfg(target_arch = "wasm32")]
pub mod wasm_entry;
#[cfg(target_arch = "wasm32")]
pub mod web_input;

pub use vc_chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, SEA_LEVEL};
