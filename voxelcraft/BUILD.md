# VoxelCraft

A high-performance, **single-codebase Minecraft-1.16.5-style voxel engine** written in Rust on top of `wgpu` (the same backend powering Bevy and Veloren). It compiles to:

- **Native** (Windows / Linux / macOS) → Vulkan · DirectX 12 · Metal  
- **WASM** (browsers with WebGPU) → served as a single static bundle

Everything that an old JS/Chromium version had problems with is fixed here:

- **Greedy meshing** + per-vertex ambient occlusion + smooth skylight flood-fill
- **Multi-threaded chunk generation & meshing** on native (Rayon), time-budgeted inline on WASM so the browser stays at 60 fps
- **Frustum culling** + per-chunk `draw_indexed` calls; one texture atlas = one bind-group
- **No asset files.** All 16×16 textures and every sound are synthesized at startup (in code). They look/sound in the style of Minecraft 1.16.5 but were not copied from any Mojang asset — they are made from scratch.
- **Native build is the real high-performance target** — the WASM build is what you can preview in a browser here; both come from the exact same source.

## Build & Run

### 1. Install Rust

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"
```

### 2. Run native (Windows / Linux / macOS)

```sh
cargo run --release
```

- Linux audio needs ALSA dev headers: `sudo apt install -y libasound2-dev` (or the equivalent on your distro).
- macOS / Windows work out of the box (CoreAudio / WASAPI).
- Vulkan drivers / MoltenVK handle the GPU backend; wgpu auto-picks the best available backend per platform.

If you can't install ALSA headers locally, you can disable audio and still get the full game engine:

```sh
cargo run --release --no-default-features
```

### 3. Build & serve the browser (WebGPU) version

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --version 0.2.127 --target web \
  --out-dir ./wasm-out target/wasm32-unknown-unknown/release/voxelcraft.wasm

# any static file server works; example:
cd wasm-out && python3 -m http.server 8080
# open http://localhost:8080/play.html
```

A matching `play.html` loader is included in this repo (`/play.html`) — it expects the `wasm-bindgen` output next to it: `voxelcraft.js`, `voxelcraft_bg.wasm`. Use Chromium 113+ (or any WebGPU-enabled browser).

## Controls

| Key                | Action                                     |
|--------------------|--------------------------------------------|
| WASD               | Move                                        |
| Mouse             | Look around (cursor is locked)              |
| Space             | Jump / swim up / fly up                     |
| **Double Space**  | Toggle flying (creative-style)             |
| Shift             | Fly down / sneak in water                   |
| Ctrl              | Sprint (FOV widens)                         |
| Left click (hold) | Break blocks                                |
| Right click (hold)| Place blocks                                |
| Middle click      | Pick block into hotbar                      |
| 1–9 / wheel       | Select hotbar slot                          |
| F3                | Debug overlay (fps, XYZ, biome, tris...)   |
| H                 | Help screen                                 |
| `[` `]`           | Render distance – / +                        |
| `-` `=`           | Volume – / +                                |
| V                 | Toggle V-Sync                               |
| Esc               | Pause / release mouse                       |

## Project layout

```
src/
  lib.rs          module roots
  blocks.rs       block registry: 18 types, tile mapping, sound families
  chunk.rs        16×256×16 storage + heightmap
  gen.rs          simplex 2D/3D noise, biomes, caves, trees (pure, thread-safe)
  world.rs        chunk map + COW edits + cross-chunk decoration edits
  mesh.rs         greedy mesher + skylight BFS + per-vertex AO
  textures.rs     256×256 procedural atlas (grass/stone/wood/water/glass/...)
  sounds.rs       synthesised 1.16.5-style sound bank + rodio/WebAudio backends
  render.rs       wgpu device + 5 pipelines + WGSL (terrain/water/sky/ui/lines)
  ui.rs           5×7 bitmap font, crosshair, hotbar, F3 overlay, pause/help
  player.rs       physics, AABB voxel collision, DDA raycast, footstep logic
  game.rs         GameApp: event handling + streaming work queue + day/night
  main.rs         native entry (Windows / Linux / macOS)
  wasm_entry.rs   WASM entry (WebGPU)
```

## Engine answer (custom vs existing?)

**Neither extreme.** This is a thin custom voxel layer (~5k LoC) on top of `wgpu` + `winit` — the approach used by Veloren at scale. Unity / Unreal / Godot were rejected (scripting GC pauses, poor voxel fit, heavyweight). Bevy was considered (gives ECS + audio + input for free) but adds version churn and less direct GPU control. The voxel-specific subsystems (chunk meshing, greedy merging, voxel lighting) are inherently custom — no engine ships them.

## Architecture notes

- **Chunk = 16×256×16**, `u8` block ids, stored as `Arc<Chunk>` in a `HashMap`. Player edits use copy-on-write so in-flight mesh jobs with old snapshots stay consistent.
- **Mesh job** snapshots the 3×3 neighborhood into a padded 48×256×48 buffer, computes skylight (column scan + lateral BFS), then greedy-merges per `(block, AO corner tuple, corner sky level, face light)` key.
- **Per-vertex light**: face shade × AO × skylight — exactly the *Minecraft "smooth lighting"* look, while still allowing greedy merging (equal corner tuples).
- **Water** = separate blended pipeline (no culling, depth-write off, top at y+0.875 with a vertex wave). Greedy-merged too, so the open ocean is one quad per chunk.
- **Sky** = fullscreen triangle, inv-`view_proj` ray, gradient + sun disc + glow + moon + stars (hash-based, twinkling), sunset band near horizon.
- **Day cycle**: 10-minute day, sun rotates, `day_light` uniform scales skylight (0.16 floor for night), fog color blends day↔night with sunset boost.

## Disclaimer

Not affiliated with Mojang or Microsoft. "Minecraft" is a trademark of Mojang Synergies AB. This project is an independent Rust engine written in the visual style of Minecraft 1.16.5; all textures and sounds are generated procedurally from scratch at startup — none are copied from Minecraft's asset files.
