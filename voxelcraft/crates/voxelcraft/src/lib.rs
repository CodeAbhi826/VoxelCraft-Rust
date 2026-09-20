//! VoxelCraft — high-performance 1.16.5-era reference-style voxel game.
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
/// The Programmer Art builtin pack, baked into the native binary (the
/// vanilla "pre-1.14 textures" analog — clean-room look-alikes generated
/// by scripts/gen_programmer_art.py). Preferred at runtime as the
/// extracted `builtin-packs/programmer-art/` folder; this embedded table
/// is the single-file fallback. Wasm fetches it from
/// `/voxelcraft-pack-programmer-art/` instead (pack.rs).
#[cfg(not(target_arch = "wasm32"))]
pub mod embedded_programmer_art {
    include!(concat!(env!("OUT_DIR"), "/embedded_programmer_art.rs"));
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
