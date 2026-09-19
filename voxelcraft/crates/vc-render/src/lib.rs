//! vc-render — wgpu renderer (§16/§33/§34): sky/terrain/water/selection/
//! UI pipelines, texture atlas, FSR 1.0 (EASU+RCAS), shader-pack API
//! (runtime WGSL validation), post-processing, HUD/UI canvas.

pub mod draw;
pub mod gpu_mesh;
pub mod gui;
pub mod gui_render;
pub mod item_icon_cache;
pub mod panorama;
pub mod iris;
/// LabPBR 1.3 material decode + the WGSL PBR/POM snippet (clean-room
/// against the published labPBR spec; wired into the resource-pack
/// material scan 2026-09-20)
pub mod pbr;
pub mod render;
pub mod shaders;
pub mod textures;
pub mod ui;
