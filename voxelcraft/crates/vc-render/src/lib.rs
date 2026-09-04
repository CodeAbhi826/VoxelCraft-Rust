//! vc-render — wgpu renderer (§16/§33/§34): sky/terrain/water/selection/
//! UI pipelines, texture atlas, FSR 1.0 (EASU+RCAS), shader-pack API
//! (runtime WGSL validation), post-processing, HUD/UI canvas.

pub mod draw;
pub mod gpu_mesh;
pub mod iris;
pub mod render;
pub mod shaders;
pub mod textures;
pub mod ui;
