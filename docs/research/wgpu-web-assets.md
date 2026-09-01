# R3 Research: wgpu-on-web limits & cross-platform asset/resource-pack loading

**Task ID:** R3 · **Agent:** research · **Date:** 2026-09-01 (approx, session date)
**Scope:** wgpu 22 (exact lockfile versions: `wgpu 22.1.0`, `wgpu-hal 22.0.0`, `wgpu-core 22.1.0`, `wgpu-types 22.0.0`), target browsers = WebGPU (Chrome/Edge/Safari 18+/Firefox nightly) **and** WebGL2 fallback (wgpu `webgl` feature).
**Method:** primary evidence = wgpu 22.1.0 source (GitHub v22.1.0 tag + local cargo-registry copies of the exact crates in our `Cargo.lock`) + docs.rs; corroborated with ~17 web searches. No project code was modified.

---

## 0. Key context fact: our WASM binary is dual-backend at runtime

wgpu 22 on wasm can include **both** the browser-WebGPU backend (`webgpu` feature, on by default) **and** the WebGL2 backend (`webgl` feature) in one binary — the old "either/or" limitation (pre-22, see [SO 76640552](https://stackoverflow.com/questions/76640552/compute-shaders-in-google-chrome-and-apple-m1)) is gone.

- Evidence: `wgpu` 22.1.0 `Cargo.toml`: `default = ["wgsl","dx12","metal","webgpu"]`, and `webgl = ["dep:hal","wgc/gles"]` — features are additive, our `Cargo.toml` (wasm dep `wgpu = { version = "22", features = ["webgl"] }`) keeps defaults → **both** compiled in. ([Cargo.toml.orig](https://github.com/gfx-rs/wgpu/blob/v22.1.0/Cargo.toml.orig))
- Selection logic: `Instance::new` → if browser has `navigator.gpu` → `ContextWebGpu` (WebGPU **only**); else → wgpu-core + GL = WebGL2. Doc comment says exactly this ("If it is set and WebGPU support is detected, this instance will *only* be able to create WebGPU adapters… if not detected… the webgl feature… is able to create a WebGL adapter"). ([wgpu/src/lib.rs, v22.1.0, `Instance::new`](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu/src/lib.rs) ~lines 2300–2360)
- Our `render.rs` already implements this chain explicitly (`BROWSER_WEBGPU` w/ high-performance → `Backends::GL`), with a `Limits::downlevel_webgl2_defaults()` retry — verified at current HEAD.

**Consequence:** every "optional" wgpu feature we adopt must either (a) work on **all three** backends (native / browser-WebGPU / WebGL2), or (b) be runtime-gated on `adapter.features()` with a fallback path, because the same wasm binary will land on WebGPU on Chrome and on WebGL2 on Safari-old/Firefox.

---

## 1. TOPIC A — Feature support matrix

Legend: ✅ supported (requestable) · ⚠️ partial/conditional · ❌ not requestable (validation error, not silent breakage) · **n/a** = feature not applicable.

### 1.1 The five roadmap-relevant features

| wgpu `Features` flag | Native (Vulkan/DX12/Metal) | Browser WebGPU (wgpu 22 `webgpu` backend) | WebGL2 (wgpu 22 `gles` backend) | Notes / failure mode |
|---|---|---|---|---|
| `MULTI_DRAW_INDIRECT` (multi-draw, no count) | ✅ DX12, Vulkan, Metal (emulated on Metal) | ❌ not in WebGPU JS API | ❌ GL backend never exposes it | Native-only feature per docs.rs |
| `MULTI_DRAW_INDIRECT_COUNT` | ✅ (Vulkan/DX12) | ❌ | ❌ — gles backend `draw_indirect_count` = `unreachable!()` → **panic** if somehow reached | Never request on GL |
| *base* indirect draw (`draw_indexed_indirect`, no feature flag) | ✅ | ✅ core WebGPU (`drawIndexedIndirect`) | ❌ — `DownlevelFlags::INDIRECT_EXECUTION` false (ES 3.1/GL 4.3 needed; WebGL2 = ES 3.0) → **validation error at pass-record time** | This is the killer for "region MDI" on WebGL2 |
| `PUSH_CONSTANTS` | ✅ (256 B; GL desktop = emulated with uniforms) | ❌ — not a WebGPU feature; `max_push_constant_size` hardcoded to 0; `compute_pass_set_push_constants` panics | ✅ **surprisingly yes** — GL backend exposes it unconditionally, naga's GLSL backend emulates push constants as **uniforms**, uploaded via `gl.uniform_*` per draw; `max_push_constant_size = 256` | ✅GL-advertised at [gles/adapter.rs:436–440](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/adapter.rs); emulation at [gles/command.rs:783](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/command.rs) (`set_push_constants` → uniform writes) and [gles/queue.rs:1515](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/queue.rs) (`C::SetPushConstants` → `gl.uniform_*`); docs.rs: "OpenGL (emulated with uniforms)". Caveat: `Limits::downlevel_webgl2_defaults()` ships `max_push_constant_size: 0` — that's wgpu's conservative cross-platform default, **not** a backend cap |
| `TEXTURE_BINDING_ARRAY` (`binding_array<texture_2d<f32>>`) | ✅ DX12, Metal (MSL 2.0+), Vulkan | ❌ not in WebGPU JS API | ❌ GL backend doesn't expose it (no bindless/ext support) | Native-only feature; keep single-atlas design (we already do) |
| `SHADER_F16` | ⚠️ Vulkan/Metal (docs note: "not supported in naga yet, only through spirv-passthrough") | ⚠️ mapped to WebGPU `shader-f16` adapter feature — only if browser adapter exposes it | ❌ not exposed by GL backend | Practically: don't rely on it anywhere |
| `TIMESTAMP_QUERY` | ✅ Vulkan/DX12/Metal | ⚠️ mapped to browser `timestamp-query` adapter feature — Chrome desktop ships it; Chrome Android / Safari / Firefox WebGPU: no → must check `adapter.features()` | ❌ requires `GL_ARB_timer_query` (desktop-GL extension string); WebGL2 has no ARB extensions (its `EXT_disjoint_timer_query_webgl2` is not wired into wgpu) → not exposed | Fine for F3 GPU-timestamp stats as native/Chrome-WebGPU-only nicety |

Sources: [docs.rs wgpu 22.1.0 `Features`](https://docs.rs/wgpu/22.1.0/wgpu/struct.Features.html) (per-feature "Supported platforms" annotations); GL backend feature list: [gles/adapter.rs](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/adapter.rs) lines 436–478 (only `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`, `CLEAR_TEXTURE`, `PUSH_CONSTANTS`, `DEPTH32FLOAT_STENCIL8` unconditional; everything else gated on desktop-GL extensions that WebGL2 never reports); WebGPU backend mapping: [wgpu/src/backend/webgpu.rs](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu/src/backend/webgpu.rs) `FEATURES_MAPPING` (exactly 11 mappable features: DEPTH_CLIP_CONTROL, DEPTH32FLOAT_STENCIL8, BC/ETC2/ASTC texture compression, TIMESTAMP_QUERY, INDIRECT_FIRST_INSTANCE, SHADER_F16, RG11B10UFLOAT_RENDERABLE, BGRA8UNORM_STORAGE, FLOAT32_FILTERABLE).

### 1.2 Compute shaders (asked: does wgpu error or emulate on GL/WebGL2?)

| Capability | Native | Browser WebGPU | WebGL2 |
|---|---|---|---|
| Compute pipelines / dispatch | ✅ | ✅ core | ❌ **error, no emulation** |

- wgpu does **not** emulate compute on WebGL2. The GL backend computes `supports_compute = ES ≥ 3.1 || GL ≥ 4.3 || GL_ARB_compute_shader` ([gles/adapter.rs:307–308](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/adapter.rs)); WebGL2 = ES 3.0 → false → `DownlevelFlags::COMPUTE_SHADERS` unset and all `max_compute_*` limits report **0**.
- Failure mode is a **clean validation error**, not a crash: `create_compute_pipeline` calls `require_downlevel_flags(DownlevelFlags::COMPUTE_SHADERS)` → `MissingDownlevelFlags` error ([wgpu-core/src/device/resource.rs:2620](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-core/src/device/resource.rs)). If one *did* reach `dispatch`, the GL call doesn't exist in ES 3.0 (glow would fail).
- Our code already handles this: `request_device` retries with `Limits::downlevel_webgl2_defaults()` (compute limits all 0), and the roadmap's fragment-shader FSR fallback on WebGL2 was the right call.
- Also note: on **native GL** < 4.3 the same error occurs — native GL is *not* a guaranteed-compute backend either. For "runs everywhere" code paths, keep a non-compute variant of any GPU-culling/GPU-meshing pass.

### 1.3 Texture sizes, 2D array textures, mipmaps

| Item | Native | Browser WebGPU | WebGL2 |
|---|---|---|---|
| `max_texture_dimension_2d` | 8192–16384+ (Vulkan/DX12/Metal report device caps) | ≥ 4096 guaranteed by spec default; desktop browsers typically report 8192 ([WebGPU spec §limits](https://www.w3.org/TR/webgpu/), [MDN GPUSupportedLimits](https://developer.mozilla.org/en-US/docs/Web/API/GPUSupportedLimits)) | = `gl.MAX_TEXTURE_SIZE`: **spec minimum 2048**; 4096 on ~99% of devices (2020 data), 8192–16384 on desktop ([webgl2fundamentals cross-platform](https://webgl2fundamentals.org/webgl/lessons/webgl-cross-platform-issues.html), [wgpu discussion #2952](https://github.com/gfx-rs/wgpu/discussions/2952)) |
| `TextureViewDimension::D2Array` (`TEXTURE_2D_ARRAY`) | ✅ | ✅ core | ✅ **core ES 3.0** — mapped directly in wgpu GL backend ([gles/conv.rs:310](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/conv.rs) `Tvd::D2Array => TEXTURE_2D_ARRAY`; GLSL ES 3.00 has `sampler2DArray`) |
| `max_texture_array_layers` | 256–2048+ | ≥256 default (2048 common) | = `gl.MAX_ARRAY_TEXTURE_LAYERS`, spec min **256** (wgpu's `downlevel_webgl2_defaults()` uses 256) |
| Mipmaps (`generate_mipmap`) | ✅ | ✅ | ✅ (ES 3.0 core `generateMipmap`) |
| Cube **array** textures | ✅ | ✅ | ⚠️ sharp edge: GL backend sets `CUBE_ARRAY_TEXTURES` downlevel flag unconditionally, but ES 3.0/WebGL2 core has no cube-map arrays — avoid `CubeArray` views on WebGL2 |

**Atlas verdict:** a **2048² atlas is guaranteed on every backend** (incl. WebGL2 spec minimum); 4096² is safe in practice (99% of WebGL2 devices, ≥ default WebGPU limit); 8192² only after checking `device.limits().max_texture_dimension_2d()` at runtime. The roadmap's plan to keep tiles 16px in a big single 2D atlas (possibly a D2Array for block/particle layers) is **fully portable**; the UltraPackedVertex UV bit-budget fix (from ROADMAP-ANALYSIS §2.1) remains the actual blocker, not texture limits.

### 1.4 Storage buffers (SSBOs)

| Item | Native | Browser WebGPU | WebGL2 |
|---|---|---|---|
| Storage buffers, any stage | ✅ | ✅ core (read-only in vertex OK; writable only in fragment/compute per WebGPU spec — [gpuweb #4132](https://github.com/gpuweb/gpuweb/issues/4132)) | ❌ **none, any stage** |
| `BufferBindingType::Storage` visible to VERTEX stage | ✅ | ✅ read-only | ❌ → `create_bind_group_layout` validation error (`DownlevelFlags::VERTEX_STORAGE` missing) |
| Writable storage in FRAGMENT | ✅ | ✅ | ❌ (`FRAGMENT_WRITABLE_STORAGE` missing) |

- GL backend: `supports_storage = ES ≥ 3.1 || GL ≥ 4.3 || GL_ARB_shader_storage_buffer_object` ([gles/adapter.rs:305–306](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/adapter.rs)) → false on WebGL2, so `max_storage_buffers_per_shader_stage = 0`, `max_storage_buffer_binding_size = 0`, `max_storage_block_size = 0` (adapter.rs lines 312–352).
- wgpu-core enforcement happens at bind-group-layout creation (`VERTEX_STORAGE` / `FRAGMENT_STORAGE` downlevel flags — [wgpu-core device/resource.rs ~1800–1830](https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-core/src/device/resource.rs)).
- **Roadmap impact:** the planned "global vertex/instance buffer + SSBO-indexed per-draw data" designs must degrade to uniform/vertex buffers on WebGL2. Base indirect draw is *also* unavailable on WebGL2 (§1.1), so region-MDI on WebGL2 can't even fall back to "SSBO of draw args + one indirect draw".

### 1.5 Verdicts that change the roadmap

1. **Region MDI (phase 8) is WebGL2-impossible** (no indirect draws at all — validation error; no MDI feature even on browser-WebGPU). Native/WebGPU can use *plain* `draw_indexed_indirect` loops (per-region args buffers). WebGL2 keeps the current per-chunk draw loop; consider draw-call reduction via instancing/merging instead of MDI.
2. **Per-draw data:** prefer **dynamic-offset uniform buffers** (portable to all 3 backends) over push constants. Push constants work native + WebGL2 (emulated as uniforms) but **hard-fail on browser-WebGPU** — since Chrome will take the WebGPU path of our binary, unconditionally requesting `PUSH_CONSTANTS` breaks Chrome. If we want them, gate on `adapter.is_feature_supported(PUSH_CONSTANTS)` and keep the UBO path as default.
3. **Texture arrays:** keep the single atlas; `binding_array` is native-only. D2Array texture layers are fine everywhere.
4. **Compute:** any GPU-side meshing/culling/GPU-light-BFS must have a CPU or fragment-shader fallback for WebGL2 (and old native GL) — error is clean but the feature is simply absent.
5. **F3 GPU timestamps:** nice-to-have behind `TIMESTAMP_QUERY` feature check (native + Chrome-WebGPU only).

---

## 2. TOPIC B — Asset / resource-pack loading cross-platform

### 2.1 `zip` crate on wasm32-unknown-unknown — works, with feature gating ✅

- The `zip` crate (2.x, roadmap's "zip 2" pin is current — [docs.rs/zip](https://docs.rs/crate/zip/latest), v2.2.x line) **compiles and runs on wasm32-unknown-unknown** if you disable its default features and keep `deflate` only: `zip = { version = "2", default-features = false, features = ["deflate"] }`.
- Why: default features include `bzip2`, `zstd`, `lzma` (C-backed crates that don't build for wasm32-unknown-unknown) and `time`/`aes-crypto` you don't need ([zip2 README features table](https://github.com/zip-rs/zip2)); `deflate` → `flate2` → **`miniz_oxide`, a pure-Rust DEFLATE** port ([flate2-rs README](https://github.com/rust-lang/flate2-rs): "This crate by default uses the miniz_oxide crate, a port of miniz.c to pure Rust"; [nickb.dev "Deflate yourself"](https://nickb.dev/blog/deflate-yourself-for-faster-rust-zips)). No zlib-ng/fuchsia/native code involved.
- Vanilla MC 1.16.5 resource-pack zips use **deflate** (and some entries stored/uncompressed) — covered. `deflate64` is decompress-only and available as an optional feature if we ever need it.
- Prior art, exact same pattern: **Stevenarella** (the reference Rust Minecraft client) uses `zip = { version = "0.6.3", features = ["deflate"], default-features = false }` + `image` for resource packs ([stevenarella Cargo.toml](https://github.com/iceiix/stevenarella/blob/master/Cargo.toml)).

### 2.2 `image` crate on wasm — PNG decode works ✅

- `image = { version = "0.25", default-features = false, features = ["png"] }` → pure-Rust PNG decoding (`png` crate → `fdeflate`/`miniz_oxide`, all no-C). The crates.io page itself recommends default-features=false + explicit formats for library use ([crates.io/crates/image](https://crates.io/crates/image)).
- Avoid default features: they pull **`rayon`** (useless-to-broken on single-threaded wasm32-unknown-unknown — no threads without SharedArrayBuffer/COOP-COEP; our native rayon streaming is already cfg-gated) and many decoders (avif native asm etc.) we don't need. ([image v0.25.5 Cargo.toml features](https://github.com/image-rs/image/blob/v0.25.5/Cargo.toml): `default = ["rayon", "default-formats"]`.)
- Known wasm pitfalls: none for PNG *decode*; `getrandom`-using crates need `features=["js"]` on wasm (stevenarella carries `getrandom = { features = ["js"] }` — keep in mind if `rand` sneaks into the pack loader).
- RGBA8 output feeds `Queue::write_texture` directly — works on all three backends (no mapping/readback involved).

### 2.3 Getting a user-selected pack zip into Rust in the browser

Two good patterns; both end with **`Vec<u8>` in Rust**, then one shared Rust pipeline:

1. **`rfd` (Rusty File Dialogs) — recommended.** `rfd::AsyncFileDialog::new().add_filter("resource pack", &["zip"]).pick_file()` → `FileHandle` → `handle.read().await` → `Vec<u8>`. Works on **Windows/Linux(GTK)/macOS and WASM32** (on web it creates a hidden `<input type=file>`) — one code path for native + web. ([rfd GitHub](https://github.com/PolyMeilex/rfd), [docs.rs AsyncFileDialog](https://docs.rs/rfd/latest/rfd/struct.AsyncFileDialog.html) — "Supported platforms: Windows; Linux; Mac; WASM32"; [users.rust-lang.org thread](https://users.rust-lang.org/t/how-can-i-read-a-file-from-disk-by-filedialog-on-wasm/97868)). This is also what the Bevy ecosystem uses for web file import (the `bevy_file_dialog` plugin is rfd-based; Bevy's own asset server only `fetch()`es HTTP paths on web — [bevy wasm asset loading](https://bevy.org/news/bevy-0-3)).
2. **Hand-rolled JS glue** (drag-and-drop or styled file input): JS listens for `drop`/`change`, calls `file.arrayBuffer()`, then hands `js_sys::Uint8Array` to an exported Rust `#[wasm_bindgen] fn load_pack(bytes: js_sys::Uint8Array)` which does `.to_vec()`. This is the pattern used in Bevy web apps that need drag-drop ([rustunit: "Rust Web Drag & Drop" w/ Bevy+WASM](https://rustunit.com/blog/2024/12-10-rust-web-drag-drop-image)). Needed because **winit's `DroppedFile` is not delivered on the web backend** (winit drag-drop is desktop-only; see [winit #1499](https://github.com/rust-windowing/winit/issues/1499) — web drag-drop must be JS-side glue). Our page already has a JS glue layer (`public/voxelcraft.js`) where a 20-line drop handler fits naturally.
   - Alternative micro-crate: `wasm-bindgen-file-reader` (implements `Read` over a JS `File`, avoids full buffering) — [crates.io](https://crates.io/crates/wasm-bindgen-file-reader/1.0.0). Not needed at pack sizes (10–100 MB is fine in linear memory; browsers allow ≥1 GB wasm heap growth, and a vanilla-style pack is ~20 MB zipped).
   - Also viable for bundled packs: `fetch("packs/vanilla.zip")` → `.arrayBuffer()` → same `Uint8Array → Vec<u8>` bridge (no CORS issue since same-origin under `/public`).

**JSZip on the JS side instead?** Unnecessary — `zip`+`image` work in wasm, so decoding in Rust keeps **one** asset pipeline, testable natively (cargo test), with no duplicated JS zip+PNG logic and no JS/Rust RGBA hand-off. Only if wasm decode ever becomes a jank problem would a Web-Worker+JSZip path (handing decoded RGBA arrays to Rust) be worth it; at 16×16-tile scale (a few hundred small PNGs, ≤ ~1 s decode on desktop wasm) it won't be. Community practice agrees: Rust voxel/MC-style projects (Stevenarella, mc tooling like `mc-repack`) do zip+PNG handling **in Rust**; browser games that need *user* files use rfd or a thin JS drop/input listener that only ferries bytes.

### 2.4 Recommended pipeline: "user drops a vanilla-style resource-pack zip → engine builds atlas"

**Rust crates (shared native+wasm):**
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
# add: (native side mirrors without the wasm notes)
zip = { version = "2", default-features = false, features = ["deflate"] }
image = { version = "0.25", default-features = false, features = ["png"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen-futures = "0.4"   # already present
```
(serde/serde_json are pure Rust, wasm-fine; likely already in tree for settings persistence patterns.)

**Pipeline steps (one Rust code path, cfg'd only at the acquisition + threading edges):**

1. **Acquire bytes → `Vec<u8>`**
   - Native: `rfd::AsyncFileDialog` pick → `handle.read().await` (or existing winit `DroppedFile` → `std::fs::read`).
   - Web: (a) `rfd::AsyncFileDialog` for a "Load Resource Pack" button, and/or (b) JS `drop`/`<input type=file>` listener in `public/voxelcraft.js` → `arrayBuffer()` → `voxelcraft_load_pack(new Uint8Array(buf))` exported shim (fits the existing shim/screen-state pattern from the E-picker work).
2. **Open zip in memory** — `zip::ZipArchive::new(Cursor::new(bytes))`; read `pack.mcmeta` (serde: `pack_format ≥ 6` for 1.16.5) → pack metadata; enumerate `assets/minecraft/textures/**.png` (+ later `blockstates/`, `models/`, `sounds.json` for phases 2/12).
3. **Decode PNGs** — per entry: `image::load_from_memory` (png feature only) → `RgbaImage`; reject/resize non-16×16 (vanilla allows 16/32 px; animations strip first frame initially).
4. **Stitch atlas (CPU, Rust)** — greedy/shelf packing into `2048²` (guaranteed everywhere) or `4096²` if `device.limits().max_texture_dimension_2d() ≥ 4096`; keep the ROADMAP-ANALYSIS-corrected UV encoding (tile_index + tile-local fract, ≥2048-addressable). Fill absent tiles from the existing procedural 44-tile fallback (zero-asset default stays).
5. **Upload** — `device.create_texture` + `queue.write_texture` (portable); `generate_mipmap` optional (all 3 backends support it).
6. **Threading** — native: `rayon` par-iter over entries (existing pattern); wasm: single-threaded, but chunk the loop (~64 entries) and `await` a yield (`wasm_bindgen_futures::JsFuture` on `setTimeout(0)` / `Promise::resolve()` microtask, or just run inside the loading screen with progress bar) so RAF/pointer-lock don't stall. Vanilla-scale packs decode < 1 s desktop / a few s mobile — acceptable behind a "Building atlas…" spinner.
7. **Persist choice** — native: settings path; web: `localStorage` remembers pack *name* but bytes must be re-imported per session (or stash the zip in IndexedDB for reload without re-pick — optional future nicety).

**Why this shape:** rfd gives the "pick a file" UX on 4 platforms with zero JS; the JS drop-glue covers the "drops a zip on the canvas" UX which winit can't deliver on web; everything after the byte-handoff is shared Rust, testable on native with `cargo test`, using only pure-Rust decoders that compile for `wasm32-unknown-unknown` (miniz_oxide/deflate + png), and uploads via queue writes that exist on native/WebGPU/WebGL2 alike.

---

## 3. Source index (per claim)

**wgpu backend internals (v22.1.0 = our lockfile):**
- GL feature set + PUSH_CONSTANTS exposure + TIMESTAMP via GL_ARB_timer_query: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/adapter.rs (lines 436–478, 307–308, 305–306)
- GL storage/compute/indirect downlevel flags: same file (lines 375–400) + dispatch/indirect command paths: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/command.rs ; execution: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/queue.rs (DrawIndirect → `draw_elements_indirect_offset`, Dispatch → `dispatch_compute`)
- Push-constant emulation as uniforms (GLSL backend + `C::SetPushConstants` → `gl.uniform_*`): gles/command.rs:783–830, gles/queue.rs:1515+, device.rs naga glsl `PushConstantItem`
- `draw_indirect_count` = `unreachable!()` on GL: gles/command.rs:1118–1140
- D2Array mapping: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-hal/src/gles/conv.rs:310
- WebGPU (browser) feature mapping (11 features; no push constants; limits passthrough; `max_push_constant_size = Limits::default() = 0`): https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu/src/backend/webgpu.rs (FEATURES_MAPPING ~line 729, map_wgt_limits ~777, panics at 3031/3229)
- Compute-pipeline downlevel gate: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu-core/src/device/resource.rs:2620 ; VERTEX_STORAGE BGL gate: same file ~1800–1830 ; UNRESTRICTED_INDEX_BUFFER buffer gate: ~505–520
- Indirect-execution gate: wgpu-core/src/command/render.rs:2478,2554
- `Limits::downlevel_webgl2_defaults()`: https://docs.rs/wgpu-types/22.0.0/wgpu_types/struct.Limits.html (0 storage / 0 compute / 2048² / 256 layers)
- Per-feature platform docs: https://docs.rs/wgpu/22.1.0/wgpu/struct.Features.html
- Dual webgl+webgpu in one binary: https://github.com/gfx-rs/wgpu/blob/v22.1.0/wgpu/src/lib.rs (Instance::new + `enabled_backend_features`) and https://github.com/gfx-rs/wgpu/blob/v22.1.0/Cargo.toml.orig
- wgpu README platform table (wasm: WebGL2 🆗 / WebGPU ✅): https://github.com/gfx-rs/wgpu/blob/v22.1.0/README.md
- Historical one-build limitation (pre-22): https://stackoverflow.com/questions/76640552/compute-shaders-in-google-chrome-and-apple-m1

**Texture limits:**
- WebGL2 min 2048 / 99% ≥4096: https://webgl2fundamentals.org/webgl/lessons/webgl-cross-platform-issues.html ; https://github.com/gfx-rs/wgpu/discussions/2952
- WebGPU limits: https://www.w3.org/TR/webgpu/ ; https://developer.mozilla.org/en-US/docs/Web/API/GPUSupportedLimits
- WebGPU timestamp-query in Chrome: https://chromestatus.com/feature/5136606877188096 (+ adapter-feature model: https://github.com/gpuweb/gpuweb/discussions/3354)
- WebGPU vertex-stage writable-storage prohibition: https://github.com/gpuweb/gpuweb/issues/4132

**Assets:**
- zip features/default-features: https://github.com/zip-rs/zip2 (README) ; https://docs.rs/crate/zip/latest
- flate2 → miniz_oxide pure-Rust default: https://github.com/rust-lang/flate2-rs ; https://nickb.dev/blog/deflate-yourself-for-faster-rust-zips
- image default-features=false guidance + rayon default: https://crates.io/crates/image ; https://github.com/image-rs/image/blob/v0.25.5/Cargo.toml
- Stevenarella pack stack (`zip` deflate-only + `image` + `getrandom/js`): https://github.com/iceiix/stevenarella/blob/master/Cargo.toml
- rfd wasm/native async file dialog: https://github.com/PolyMeilex/rfd ; https://docs.rs/rfd/latest/rfd/struct.AsyncFileDialog.html ; https://users.rust-lang.org/t/how-can-i-read-a-file-from-disk-by-filedialog-on-wasm/97868
- JS-side drag&drop → bytes into Rust (Bevy pattern): https://rustunit.com/blog/2024/12-10-rust-web-drag-drop-image ; winit web drag-drop gap: https://github.com/rust-windowing/winit/issues/1499
- wasm-bindgen-file-reader (streaming Read over JS File): https://crates.io/crates/wasm-bindgen-file-reader/1.0.0
- Bevy web assets = fetch only: https://bevy.org/news/bevy-0-3 ; wasm-bindgen Vec<u8> bridging: https://docs.rs/js-sys/latest/js_sys/struct.Uint8Array.html

---

## 4. One-paragraph bottom line

WebGL2 (wgpu `gles`) gives us **no compute, no indirect draws, no storage buffers, no TEXTURE_BINDING_ARRAY, no f16, no GPU timestamps — but does give us push constants (emulated as uniforms), 2048²+ 2D atlases and D2Array layers**; browser-WebGPU adds back compute/storage/indirect-base but **forbids push constants and everything else in our wishlist that isn't one of the 11 spec features**; native has it all. Therefore: keep the single-atlas + dynamic-offset-UBO design, gate MDI/compute/timestamps behind `adapter.features()` with per-chunk-draw / CPU / fragment-shader fallbacks (which mostly exist), and load resource packs as **bytes → Rust (`rfd` or 20 lines of JS drop glue) → `zip`/`image` with `default-features=false` (deflate/png, pure-Rust) → CPU-stitch 2048/4096² atlas → `queue.write_texture`** — one pipeline for native, WebGPU and WebGL2, with the procedural atlas remaining the zero-asset fallback.
