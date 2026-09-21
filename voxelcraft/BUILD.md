# VoxelCraft

A high-performance, **single-codebase 1.16.5-era reference-style voxel engine** written in Rust on top of `wgpu` (the same backend powering Bevy and Veloren). It compiles to:

- **Native** (Windows / Linux / macOS) → Vulkan · DirectX 12 · Metal  
- **WASM** (browsers with WebGPU) → served as a single static bundle

Everything that an old JS/Chromium version had problems with is fixed here:

- **Greedy meshing** + per-vertex ambient occlusion + smooth skylight flood-fill
- **Multi-threaded chunk generation & meshing** on native (Rayon), time-budgeted inline on WASM so the browser stays at 60 fps
- **Frustum culling** + per-chunk `draw_indexed` calls; one texture atlas = one bind-group
- **No asset files.** All 16×16 textures and every sound are synthesized at startup (in code). They look/sound in the style of the 1.16.5 reference game but were not copied from any third-party asset — they are made from scratch.
- **Native build is the real high-performance target** — the WASM build is what you can preview in a browser here; both come from the exact same source.

## Build & Run

### 1. Install Rust

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"
```

### 2. Run native (Windows / Linux / macOS)

```sh
cargo run --release          # from the workspace root (this directory)
```

The engine is a Cargo **workspace**: `crates/vc-*` are the 14 libraries,
`crates/voxelcraft` is the application. Run everything from this root —
`builtin-pack/` is resolved from the working directory (and when the
folder is absent, the pack baked into the binary at compile time by
`crates/voxelcraft/build.rs` takes over — that is what makes the
single-file release work: a lone `voxelcraft` executable with zero
companion files still boots the full pack; a real folder next to the
binary always overrides the embedded copy).

- Linux audio needs ALSA dev headers: `sudo apt install -y libasound2-dev` (or the equivalent on your distro).
- macOS / Windows work out of the box (CoreAudio / WASAPI).
- Vulkan drivers / MoltenVK handle the GPU backend; wgpu auto-picks the best available backend per platform.

If you can't install ALSA headers locally, you can disable audio and still get the full game engine:

```sh
cargo run --release --no-default-features
```

#### Linux input (mouse / pointer capture) — troubleshooting

Look-around uses a pointer-capture ladder `Locked → Confined → delta-look`.
**Linux never asks for `Locked`** (X11 cannot honor it — an X pointer grab IS
a confinement; Wayland's `Confined` delivers the identical raw relative
motion AND keeps `CursorMoved` flowing as the watchdog's recovery channel):
the Linux ladder is `Confined → Delta`. `Locked` is only attempted on
Windows/macOS. `Delta` (no pointer-lock protocol available) keeps the cursor
visible and feeds look input from cursor position deltas — the
guaranteed-delivery channel. The engine never hides the cursor without a
working grab and never feeds two motion streams at once; if the first click
after a menu finds the grab lost, it re-attempts the ladder. Every rung logs
a `pointer:` line through the boot-log sinks (visible with `--debug`,
mirrored to `logs/latest.log`), and the boot carries a
`pointer env: session=… wayland_display=… winit_unix_backend=… VC_POINTER=…`
line so every input bug report self-describes its environment.

The 2026-09-21 build wires the full click-side routing hardening (ported
from the owner's parallel track):

- **Pointer-starvation watchdog**: every capture arms an evidence window —
  `CursorMoved` and raw `DeviceEvent::MouseMotion` counters, judged every
  frame. A real grab where the user demonstrably moves the mouse (>= 3
  `CursorMoved`) but not ONE raw motion arrives > 1 s into the capture is
  demoted to delta-look automatically: grab released, cursor visible, the
  broken rung marked sticky-unreliable (never retried this process). You
  see the cursor come back and keep playing instead of a frozen camera.
- **`VC_POINTER=auto|delta|confined|locked`** env override — the documented
  manual escape hatch AND the first diagnostic step for a pointer report.
- **Raw-button click routing**: quick physical clicks that arrive ONLY as
  raw `DeviceEvent::Button` (never as `WindowEvent::MouseInput` — the
  classic X11-under-grab case behind "click sound but nothing happens") are
  routed through the same authoritative click path, with a dedup guard for
  platforms delivering both forms of one physical click. Short clicks
  (press+release inside one input batch, before the next tick) are latched
  as taps so they still act exactly once.
- A fresh screen re-derives hover from the current cursor position
  immediately (the initial cursor can sit on a button with no `CursorMoved`
  ever arriving — the painted highlight and the hit-test table now agree
  from the first frame).

- Run with `--debug` and read the `pointer:` line to see which rung your
  compositor actually granted (`locked` / `confined to the window (raw
  motion + watchdog)` / `lock unavailable — delta-look fallback`).
- Want a hidden-cursor lock on a Wayland session? Force the XWayland
  path, which uses XInput2 raw motion: `WINIT_UNIX_BACKEND=x11 ./voxelcraft-*`.
- Edge note: in delta-look mode the camera stops turning when the (visible)
  cursor reaches the screen edge — inherent to position-delta input; pull the
  mouse back toward the center to keep turning.

### 3. Build & serve the browser (WebGPU) version

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --version 0.2.127 --target web \
  --out-dir ./wasm-out target/wasm32-unknown-unknown/release/voxelcraft.wasm
python3 patch-wasm-glue.py wasm-out/voxelcraft.js     # re-applies the CDP input hardening

# any static file server works; example:
python3 -m http.server 8080
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
crates/
  vc-nbt/         NBT codec (all 13 tag types, round-trip safe)
  vc-blocks/      block registry, BlockState ids, sound families, biome tint
  vc-chunk/       16×256×16 storage, paletted sections, heightmaps
  vc-rng/         deterministic seeded RNG
  vc-pack/        resource-pack / blockstate+model JSON pipeline
  vc-inventory/   items, ItemStacks, containers
  vc-world/       chunk map + COW edits, terrain gen, light engine
  vc-mesh/        greedy mesher + skylight BFS + per-vertex AO
  vc-particles/   break/hit particle pool (vanilla physics)
  vc-gameplay/    crafting, furnaces, brewing, enchanting, villagers
  vc-sim/         20 Hz tick loop, fluids, redstone, item entities
  vc-anvil/       vanilla 1.16.5 save/load (NBT + .mca regions)
  vc-render/      wgpu renderer, atlas, FSR 1.0, shader packs, UI canvas
  vc-audio/       synthesized sound bank + rodio/WebAudio backends
  voxelcraft/     the app: game.rs, player.rs, main.rs, wasm_entry.rs, vc_bench
```

## Engine answer (custom vs existing?)

**Neither extreme.** This is a thin custom voxel layer (~5k LoC) on top of `wgpu` + `winit` — the approach used by Veloren at scale. Unity / Unreal / Godot were rejected (scripting GC pauses, poor voxel fit, heavyweight). Bevy was considered (gives ECS + audio + input for free) but adds version churn and less direct GPU control. The voxel-specific subsystems (chunk meshing, greedy merging, voxel lighting) are inherently custom — no engine ships them.

## Architecture notes

- **Chunk = 16×256×16**, `u8` block ids, stored as `Arc<Chunk>` in a `HashMap`. Player edits use copy-on-write so in-flight mesh jobs with old snapshots stay consistent.
- **Mesh job** snapshots the 3×3 neighborhood into a padded 48×256×48 buffer, computes skylight (column scan + lateral BFS), then greedy-merges per `(block, AO corner tuple, corner sky level, face light)` key.
- **Per-vertex light**: face shade × AO × skylight — exactly the *the reference game "smooth lighting"* look, while still allowing greedy merging (equal corner tuples).
- **Water** = separate blended pipeline (no culling, depth-write off, top at y+0.875 with a vertex wave). Greedy-merged too, so the open ocean is one quad per chunk.
- **Sky** = fullscreen triangle, inv-`view_proj` ray, gradient + sun disc + glow + moon + stars (hash-based, twinkling), sunset band near horizon.
- **Day cycle**: 10-minute day, sun rotates, `day_light` uniform scales skylight (0.16 floor for night), fog color blends day↔night with sunset boost.

## Disclaimer

Not affiliated with, endorsed by, or connected to any game publisher or studio. All product names and trademarks mentioned or implied by the "reference game" phrasing belong to their respective owners; none are used here as this project's own branding. This project is an independent Rust engine written in the visual style of the 1.16.5-era reference game; all textures and sounds are generated procedurally from scratch at startup — none are copied from any third-party asset files.

### 4. Post-build step (required): patch the generated JS glue

`wasm-bindgen` regenerates `wasm-out/voxelcraft.js` from scratch on every
build and overwrites manual fixes. Re-apply the pointerType hardening patch
after every `wasm-bindgen` run, then copy the bundle into `../public/`:

```sh
python3 patch-wasm-glue.py wasm-out/voxelcraft.js
cp wasm-out/voxelcraft.js ../public/voxelcraft.js
cp wasm-out/voxelcraft_bg.wasm ../public/voxelcraft_bg.wasm
```

The patch makes winit's pointer-event handlers tolerate synthetic /
automation events that lack `pointerType` (CDP `Input.dispatchMouseEvent`,
manually dispatched `MouseEvent`s) instead of crashing with
`TypeError: Cannot read properties of undefined (reading 'length')`.
Real browser pointer events always carry `pointerType` and are unaffected.

## Resource pack (Phase 1, Master Spec §5.2/§19)

The engine ships a clean-room **builtin pack** at `voxelcraft/builtin-pack/`
(vanilla 1.16.5 layout: `pack.mcmeta` + `assets/voxelcraft/{blockstates,
models,textures}`). Native reads the folder — falling back to the **copy
embedded in the binary at compile time** (`crates/voxelcraft/build.rs`
packages the pack into `.rodata` via `include_bytes!`; the
`linux-game.yml` workflow ships that as the single-file game) — and WASM
fetches the same files from `/voxelcraft-pack/` (CI deploys
`public/voxelcraft-pack`). Any failure falls back to the procedural atlas
+ missing-texture tile (§46). Precedence: real folder > embedded copy >
procedural.

Regenerate the PNG textures (procedural art → PNG strips) on demand:

```sh
cd voxelcraft
cargo test write_builtin_pack_pngs -- --ignored --nocapture
```

Deps added (pure-Rust, wasm-safe per docs/research/wgpu-web-assets.md):
`serde` + `serde_json` (JSON), `image` png-only (texture decode).

## All-architecture builds (one command)

```sh
./scripts/build-all.sh              # host binary + WASM browser bundle
./scripts/build-all.sh --cross      # + linux-arm64, windows, macOS x64/arm64
./scripts/build-all.sh --wasm-only  # just the browser bundle
```

Artifacts land in `../dist/voxelcraft-<commit>-<target>/` — binary +
`builtin-pack/` + README (web bundles also include `play.html` and the
wasm-bindgen JS glue). Cross-targets need `rustup target add <target>`
(and `cross` via Docker when plain cargo lacks the system libs).

## CI automation (GitHub Actions)

| workflow | trigger | what it does |
|----------|---------|--------------|
| `ci.yml` | every push (any branch) + PRs to main | full test suite (489+ tests), wasm32 lib check, native check with audio, headless `vc_bench` + JSON artifact |
| `linux-game.yml` | every push to main + manual | builds the **single-file Linux game** (pack embedded in the binary via `build.rs`) and attaches it to the run — one executable, zero companion files |
| `wasm-build.yml` | pushes to main touching `voxelcraft/**` | rebuilds the WASM bundle and **commits it back to `public/`** — the deploy branch always carries a playable build |
| `release.yml` | manual (`workflow_dispatch`) or tag `v*` | builds **every architecture** — Linux x64 + arm64, Windows x64, macOS x64 + arm64, WASM web — packages each with the builtin pack, uploads artifacts, and on tags publishes a GitHub Release |

Publish a release:

```sh
git tag v0.2.0 && git push origin v0.2.0   # → release.yml builds + publishes
```
