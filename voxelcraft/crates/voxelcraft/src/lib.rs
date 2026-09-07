//! VoxelCraft — high-performance Minecraft-1.16.5-style voxel game.
//! Rust + wgpu. One codebase -> native (Vulkan/DX12/Metal) + WASM (WebGPU).
//!
//! This crate is the APPLICATION/game shell; the reusable engine lives in
//! the `vc-*` library crates of this workspace (see LIBRARIES.md).

pub mod alloc_stats;
pub mod bench;
/// the builtin resource pack, baked into the binary by build.rs (native
/// only — the single-file release). Used when no `builtin-pack/` folder
/// sits next to the binary; see build.rs for the precedence rules.
#[cfg(not(target_arch = "wasm32"))]
pub mod embedded_pack {
    include!(concat!(env!("OUT_DIR"), "/embedded_pack.rs"));
}
pub mod game;
pub mod player;
#[cfg(target_arch = "wasm32")]
pub mod wasm_entry;
#[cfg(target_arch = "wasm32")]
pub mod web_input;

// F3 "Allocated" telemetry: install the counting allocator process-wide.
// (native: declared in main.rs instead — the binary owns the allocator;
// wasm has no bin crate, so it lives here)
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: alloc_stats::Counting = alloc_stats::Counting;

pub use vc_chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, SEA_LEVEL};
