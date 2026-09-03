# voxelcraft — the game application

The application shell that wires the 14 `vc-*` engine libraries into a
playable game: the winit event loop, player physics (AABB voxel
collision + DDA raycast), the chunk streaming/mesh job scheduler,
day–night cycle, HUD/UI, settings, the benchmark harness, and the WASM
entry (WebGPU with automatic WebGL2 fallback).

This is the **only crate that produces binaries**:

| Binary | What it does |
|---|---|
| `voxelcraft` | the game — native Vulkan / DX12 / Metal via wgpu |
| `vc_bench` | headless CPU benchmark (no GPU needed) — `--features bench-bin` |
| `cdylib` (wasm32) | boots on the `#game` canvas — **dev preview** of the engine, not an end-user product |

## Run

```sh
# from the workspace root (voxelcraft/) — builtin-pack/ is looked up in CWD:
cargo run --release

# no ALSA headers? build without audio:
cargo run --release --no-default-features

# headless benchmark (generation + meshing + remesh + memory + draw-prep):
cargo run --release --features bench-bin --bin vc_bench

# in-game scripted benchmark on a real desktop:
cargo run --release -- --benchmark 600 120
```

## Web build (dev preview)

> The browser build exists so the engine can be **run and verified
> instantly without a toolchain** — it is a development/preview tool,
> not the product. The native build is the performance target.

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --version 0.2.127 --target web \
  --out-dir ../../wasm-out ../../target/wasm32-unknown-unknown/release/voxelcraft.wasm
python3 ../../patch-wasm-glue.py ../../wasm-out/voxelcraft.js
# serve the workspace root and open play.html
```

## Where the engine lives

Every subsystem is a separately downloadable library — see
[`../../LIBRARIES.md`](../../LIBRARIES.md).

## Spec reference

Master Spec §1 (goal), §8 (streaming), §23 (player physics), §31 (F3),
§36 (WASM), §44 (frame pacing).
