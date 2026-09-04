# VoxelCraft

A high-performance, **single-codebase Minecraft-1.16.5-style voxel engine** written in **Rust** on top of `wgpu` — the same graphics abstraction that powers Bevy and Veloren. One codebase, two targets:

- **Native** (Windows / Linux / macOS) → Vulkan · DirectX 12 · Metal
- **Browser** (WASM + WebGPU, with automatic WebGL2 fallback) → served as static files, **prebuilt and included** so it can be run instantly

## Libraries — download them separately (no all-in-one bundle)

The engine is split into **14 independent libraries** (`vc-nbt`, `vc-blocks`, `vc-world`, `vc-mesh`, `vc-render`, …), each one a normal Rust crate with its own README, usage example and test suite. On the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page **every library ships as its own archive** (`vc-nbt-0.3.0-source.tar.gz`, `vc-blocks-0.3.0-source.tar.gz`, …) next to the per-architecture game binaries — there is deliberately **no AIO zip**. Grab exactly what you need and drop it into your project as a path dependency.

→ Full index with per-library instructions: **[`voxelcraft/LIBRARIES.md`](voxelcraft/LIBRARIES.md)**

![VoxelCraft first person view](docs/screenshots/voxelcraft-release-2-game-first.png)

![VoxelCraft rotated view](docs/screenshots/voxelcraft-release-3-game-rotated.png)

## Quick start — run it in the browser (dev preview)

No toolchain needed. The repo ships the prebuilt WASM bundle — this is the fastest way to **run and verify** the engine (it is a development preview of the native build, not an end-user product):

```sh
cd voxelcraft
python3 -m http.server 8080
# open http://localhost:8080/play.html  (Chromium 113+ or any WebGPU browser)
```

Or just open `voxelcraft/play.html` through any static file server.

## Quick start — native build

```sh
# 1. Install Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable   # (see BUILD.md)
source "$HOME/.cargo/env"

# 2. Run
cd voxelcraft
cargo run --release
```

- Linux audio wants ALSA dev headers: `sudo apt install -y libasound2-dev` (otherwise `cargo run --release --no-default-features` builds without sound)
- macOS / Windows work out of the box (CoreAudio / WASAPI)
- wgpu auto-picks the best GPU backend per platform (Vulkan / DX12 / Metal)

## Why it is fast

The previous prototype of this game was JavaScript/Chromium and suffered heavy frame drops. This engine fixes all of it:

- **Greedy meshing** — up to hundreds of merged quads per chunk; an entire flat ocean is one quad
- **Per-vertex ambient occlusion + smooth skylight** flood-fill (BFS) — the exact "smooth lighting" look, while still allowing greedy merging (equal AO/sky corner tuples)
- **Multi-threaded** chunk generation and meshing on native (Rayon), time-budgeted inline streaming on WASM so the browser stays at 60 fps
- **Frustum culling** + per-chunk `draw_indexed` calls, one texture atlas = one bind group
- **Copy-on-write chunk edits** — in-flight mesh jobs with old snapshots stay consistent while you build

## Zero asset files

All 16×16 textures and every sound are **synthesized procedurally at startup** (in code). They are in the style of Minecraft 1.16.5 but were made from scratch — no Mojang assets were copied, bit-for-bit, anywhere.

## Controls

| Key                | Action                                      |
|--------------------|---------------------------------------------|
| WASD               | Move                                        |
| Mouse              | Look around (cursor is locked)              |
| Space              | Jump / swim up / fly up                     |
| **Double Space**   | Toggle flying (creative-style)              |
| Shift              | Fly down / sneak in water                   |
| Ctrl               | Sprint (FOV widens)                         |
| Left click (hold)  | Break blocks                                |
| Right click (hold) | Place blocks                                |
| Middle click       | Pick block into hotbar                      |
| 1–9 / wheel        | Select hotbar slot                          |
| F3                 | Debug overlay (fps, XYZ, biome, tris...)    |
| H                  | Help screen                                 |
| `[` `]`            | Render distance – / +                       |
| `-` `=`            | Volume – / +                                |
| V                  | Toggle V-Sync                               |
| Esc                | Pause / release mouse                       |

## World

- Procedural terrain: simplex 2D/3D noise, biomes (plains / forest / desert / snow / ocean), caves, trees
- Day/night cycle (10 min), dynamic sun/moon/stars, sunset band, fog
- Water with wave animation, glass, 18 block types
- 16×256×16 chunks streamed around the player

## Repository layout

```
voxelcraft/                     Cargo workspace (the engine + the game)
  Cargo.toml                    workspace manifest (shared versions, profile)
  LIBRARIES.md                  index of all 14 libraries + download instructions
  crates/
    vc-nbt/                     NBT codec (read/write, all 13 tag types)
    vc-blocks/                  block registry + BlockState + biome tint
    vc-rng/                     deterministic RNG
    vc-chunk/                   16×256×16 chunks, paletted sections
    vc-pack/                    resource-pack / blockstate+model JSON pipeline
    vc-inventory/               items, stacks, containers
    vc-world/                   world grid + terrain gen + light engine
    vc-mesh/                    greedy mesher (AO/skylight-aware merging)
    vc-particles/               vanilla-style break/hit particles
    vc-gameplay/                crafting, furnaces, brewing, enchanting, villagers
    vc-sim/                     20 Hz simulation: fluids, redstone, entities
    vc-anvil/                   vanilla 1.16.5 save/load (.mca + level.dat)
    vc-render/                  wgpu renderer, FSR 1.0, shader packs, UI
    vc-audio/                   synthesized sound bank + spatial audio
    voxelcraft/                 the APPLICATION (game, vc_bench, wasm entry)
  builtin-pack/                 1.16.5-format resource pack (blockstates/models/PNGs)
  shader-packs/                 demo shader packs (moonlit, warm-evening)
  wasm-out/                     prebuilt wasm-bindgen output (run instantly)
  play.html                     standalone browser loader
  BUILD.md                      full build instructions (native + wasm + all-arch)
docs/                           roadmap analysis, session logs
docs/screenshots/               in-game screenshots
public/                         same wasm build, wired into the Next.js preview wrapper
src/app/page.tsx                Next.js wrapper that serves the game at /
scripts/                        build-all.sh (all-arch one-shot builder)
```

The Next.js app in this repo root is only a thin preview wrapper (it iframes `public/voxelcraft.html`); the game itself is entirely in `voxelcraft/`.

## Rebuilding the browser bundle

```sh
cd voxelcraft                                   # workspace root
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --version 0.2.127 --target web \
  --out-dir ./wasm-out target/wasm32-unknown-unknown/release/voxelcraft.wasm
python3 patch-wasm-glue.py wasm-out/voxelcraft.js
```

See `voxelcraft/BUILD.md` for details.

## Architecture notes

- **Chunk = 16×256×16**, `u8` block ids, stored as `Arc<Chunk>` in a `HashMap`. Player edits use copy-on-write so in-flight mesh jobs with old snapshots stay consistent.
- **Mesh job** snapshots the 3×3 neighborhood into a padded 48×256×48 buffer, computes skylight (column scan + lateral BFS), then greedy-merges per `(block, AO corner tuple, corner sky level, face light)` key.
- **Water** = separate blended pipeline (no face culling, depth-write off, top surface at y+0.875 with a vertex wave).
- **Sky** = fullscreen triangle, inverse view-proj ray, gradient + sun disc + glow + moon + twinkling stars.
- **wgpu backend selection on the web**: WebGPU when `navigator.gpu` exists, otherwise WebGL2 with downlevel limits — verified working in headless Chromium (SwiftShader).

## Disclaimer

Not affiliated with Mojang or Microsoft. "Minecraft" is a trademark of Mojang Synergies AB. This is an independent Rust engine written in the visual style of Minecraft 1.16.5; all textures and sounds are generated procedurally from scratch — none are copied from Minecraft's asset files.

## License

Licensed under the **Apache License 2.0** — see [`LICENSE`](LICENSE) for the full text.

In short: you are free to use, copy, modify, and distribute this project (including commercially), as long as you retain the license notice and state significant changes. Game *mechanics and data* (formulas, timings, recipe/loot schemas, registry names) are not copyrightable and are replicated from published documentation; all *assets* (textures, sounds, UI art) are independently authored and contain no Mojang material.
