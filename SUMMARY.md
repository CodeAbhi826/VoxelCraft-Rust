# VoxelCraft-Rust — Rewrite Summary

## What This Is

A complete Rust rewrite of the VoxelCraft Minecraft-style voxel sandbox. The old project was C++ with OpenGL 4.6, CMake build hell, hand-drawn GL-quad UI, bitmap font, and palette bit-packing bugs. This is a **brand new Rust project** — zero C++ code was copied or ported.

---

## What Was Done

### The Engine (lib.rs — pure logic, no GPU deps)

All written from scratch in Rust:

- **18 block types** with properties (solid/opaque/liquid/hardness)
- **16×64×16 chunks** stored in flat Vec per layer
- **Multi-octave Perlin terrain** (continents + hills + detail + caves + ores + trees + water)
- **Greedy face culling** + per-vertex ambient occlusion
- **Player physics**: AABB collision, gravity, jump, sprint, swim, fly mode, DDA voxel raycast
- **Chunk streaming**: generate/mesh around player, unload far chunks
- **Structured logger** with `tracing` backend + ring buffer for in-game UI

### The Renderer (main.rs — wgpu + egui shell)

Written as the graphical frontend:

- **wgpu 0.20** renderer (Vulkan/Metal/DX12/WebGPU — no OpenGL dependency)
- **Procedural texture atlas**: 19 tiles, 16×16 px, CPU-generated RGBA
- **World shader** (WGSL): texture array sampling + per-vertex AO tint + distance fog + alpha test
- **Frustum + distance culling** for chunk rendering
- **egui 0.28** UI: start screen, settings panel, debug HUD, hotbar, crosshair, pause menu, logger panel
- **Loading screen** with progress bar during world generation
- **Full game state machine**: Loading → StartScreen ↔ Playing ↔ Paused/Settings/Logger

---

## API Mismatches Fixed (~20)

The `main.rs.full` file was written expecting wgpu 22 APIs, but `egui-wgpu 0.28` depends on **wgpu 0.20**. This incompatibility caused ~20 compile errors. Here's every fix:

### Cargo.toml
| Before | After | Reason |
|--------|-------|--------|
| `wgpu = "22"` | `wgpu = "0.20"` | egui-wgpu 0.28 depends on wgpu 0.20, not 22. Using "22" caused dual-version conflict |

### src/main.rs
| Line | Issue | Fix |
|------|-------|-----|
| 17 | `KeyboardInput` removed in winit 0.29 | Renamed to `KeyEvent` |
| 50 | `Instance::new(&desc)` | Removed `&` (takes by value in wgpu 0.20) |
| 99 | Missing `..` for `device_id`/`is_synthetic` fields | Added `..` in match pattern |
| 102-125 | `physical_key` is `PhysicalKey` not `KeyCode` | Wrapped all patterns in `PhysicalKey::Code(...)` |
| 125 | `Digit1..=Digit9` range pattern on enum | Changed to nested `\|` pattern `digit @ (Digit1 \| Digit2 \| ...)` |

### src/renderer/mod.rs
| Line | Issue | Fix |
|------|-------|-----|
| 20 | `Surface` needs lifetime | Added `<'static>` |
| 87 | `BindGroupLayout` doesn't implement Clone | Changed `WorldPipeline::new` to take `&BindGroupLayout` |
| 145 | `Renderer::Output` doesn't exist | Changed to `egui::ClippedPrimitive` |
| 184 | `occlusion_query_writes` doesn't exist | Changed to `occlusion_query_set` |
| 188 | `to_cols_array()` returns `[f32; 16]` | Changed to `to_cols_array_2d()` |
| 205 | `powi` on ambiguous float | Added `_f32` suffix |
| 248-254 | egui-wgpu 0.28 API different | Restructured: `update_buffers` returns `Vec<CommandBuffer>`, `render` takes `(&RenderPass, &[ClippedPrimitive], &ScreenDescriptor)` |
| 158-246 | Duplicate world pass block | Removed original, kept the refactored one (my edit accidentally duplicated it) |

### src/renderer/pipeline.rs
| Line | Issue | Fix |
|------|-------|-----|
| 25 | `BingGroupLayout` by value | Changed parameter to `&BindGroupLayout` |
| 29 | `to_cols_array()` returns `[f32; 16]` | Changed to `to_cols_array_2d()` |
| 58 | Double reference `&&BindGroupLayout` | Changed `&texture_layout` to `texture_layout` |

### src/renderer/world.wgsl
| Line | Issue | Fix |
|------|-------|-----|
| 13-14 | Sampler/texture bindings swapped vs pipeline layout | Swapped `@binding(0)` and `@binding(1)` |
| 44 | `texture()` is not a WGSL builtin | Changed to `textureSample()` with appropriate args |
| 52 | Can't assign to swizzle `col.rgb = ...` | Changed to `col = vec4<f32>(col.rgb * in.color, col.a)` |
| 57 | Same swizzle assignment issue | Same fix with `mix()` |

### src/ui/mod.rs
| Line | Issue | Fix |
|------|-------|-----|
| 109 | `ProgressBar::show(ui)` doesn't exist | Changed to `ui.add(ProgressBar::new(...))` |

---

## Test Results

### Unit Tests (22/22 pass)
```
test world::chunk::tests::chunk_dirty_flag_on_set ... ok
test world::chunk::tests::chunk_index_within_bounds ... ok
test world::chunk::tests::chunk_set_get_roundtrip ... ok
test world::chunk::tests::chunk_y_out_of_bounds_returns_air ... ok
test world::mesher::tests::empty_chunk_produces_empty_mesh ... ok
test world::mesher::tests::single_block_produces_12_triangles ... ok
test world::mesher::tests::two_adjacent_blocks_cull_shared_face ... ok
test world::noise::tests::chunk_generation_produces_terrain ... ok
test world::noise::tests::terrain_height_is_reasonable ... ok
test world::noise::tests::water_at_sea_level_for_low_terrain ... ok
test player::tests::player_collides_with_walls ... ok
test player::tests::player_falls_with_gravity ... ok
test player::tests::player_lands_on_ground ... ok
test player::tests::raycast_hits_block ... ok
test player::tests::raycast_misses_empty_space ... ok
test player::tests::void_respawn_works ... ok
test world::world::tests::cpu_cores_detected ... ok
test world::world::tests::parallel_chunk_generation_is_correct ... ok
test world::world::tests::world_block_set_get_roundtrip ... ok
test world::world::tests::world_chunk_streaming ... ok
test world::world::tests::world_handles_negative_coords ... ok
test world::world::tests::parallel_meshing_produces_valid_meshes ... ok
```

### Integration Tests (8/8 pass)
```
test all_blocks_have_textures ... ok
test block_break_place_roundtrip ... ok
test chunk_mesh_face_culling_works ... ok
test player_physics_simulates_correctly ... ok
test raycast_accuracy ... ok
test terrain_has_variety ... ok
test full_engine_smoke_test ... ok
test world_chunk_streaming_loads_and_unloads ... ok
```

**Total: 30 tests, 0 failures**

### Headless Render Test
The `render_png` example generates a 800×600 CPU raycasted image + 128×128 top-down map:
```
Render done in 3.31s
  330575 terrain pixels, 149425 sky pixels
```

### Full Game Test
`cargo run --release` opens a 1280×720 window, initializes wgpu, compiles shaders, generates chunks, and displays the loading screen transitioning to the start screen.

---

## How to Build & Run

```bash
cargo test                 # 30 tests must pass
cargo run --example render_png --release   # headless terrain render
cargo build --release      # build the game
./target/release/voxelcraft # run the game
```

---

## Key Decisions

| Decision | Why |
|----------|-----|
| wgpu 0.20 instead of 22 | egui-wgpu 0.28 depends on wgpu 0.20 — version must match |
| Engine as `lib.rs` | Pure logic, no GPU deps, fully testable with `cargo test` |
| egui for UI | Professional widgets (sliders, panels, scrollbars) for free |
| Procedural textures only | Zero Mojang or external assets |
| `Arc<Window>` for surface | Allows `'static` lifetime on wgpu `Surface` |
