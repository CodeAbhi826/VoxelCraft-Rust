# Research: Sodium-style chunk meshing, vertex packing & Multi-Draw Indirect for a wgpu native+web engine

**Task ID:** R1 · **Date:** 2026-09 (repo state: commit 69ab97b, wgpu 22.1.0) · **Agent:** research
**Method:** direct source study of Sodium (sodium-fabric tags `mc1.16.5-0.2.0`, `mc1.17.1-0.3.4`, `mc1.20.1-0.5.0/0.5.1`, and current `CaffeineMC/sodium` main ≈ 0.9.x) + wgpu 22.1.0 / wgpu-hal 22.0.0 / naga 22.1.0 sources from the local cargo registry + web search (Chrome/MDN/docs.rs/GitHub).

All bit layouts below were transcribed **from the actual Java/GLSL source**, not from blog posts. Byte offsets are little-endian memory order (`memPutInt(ptr+0, x | (y<<16))` ⇒ low 16 bits at lower address).

---

## 1. Sodium's actual chunk vertex formats (per version)

### 1.1 The myth of the "u64 compact vertex"

No released Sodium version ever shipped an 8-byte (single `u64`) chunk vertex. The smallest Sodium formats are **16 bytes** (0.5.x) and **20 bytes** (0.3.x, 0.6+). The roadmap's `UltraPackedVertex` (~8 bytes) is not a Sodium design — see §8 for why it cannot work anyway.

### 1.2 Sodium 0.2.0 — MC **1.16.5** (the version VoxelCraft targets)

Source: `sodium-fabric` tag `mc1.16.5-0.2.0`, `src/main/java/me/jellysquid/mods/sodium/client/render/chunk/format/…`
This version has **two** selectable formats (the video-settings "Use Compact Vertex Format" toggle, issues #460/#491):

**SFP (single-precision floats) — 32 bytes, vanilla-like:**

| Offset | Size | Field | Type |
|---|---|---|---|
| 0 | 12 | POSITION | f32 × 3 |
| 12 | 4 | COLOR | u8 × 4 (normalized; alpha = AO shade) |
| 16 | 8 | TEXTURE | f32 × 2 (whole-atlas absolute UV) |
| 24 | 4 | LIGHT | u16 × 2 (normalized light-map texcoords) |

**HFP ("compact") — 20 bytes:**

| Offset | Size | Field | Encoding |
|---|---|---|---|
| 0 | 6 | POSITION | u16 × 3, `(v) * 2048.0` → value×2048/65536 ⇒ **0..32 blocks @ 1/2048 block** (`MODEL_SCALE = 32/65536`) |
| 8 | 4 | COLOR | u8 × 4 normalized (AO folded into alpha) |
| 12 | 4 | TEXTURE | u16 × 2, `v * 32768` → **whole-atlas normalized 0..1 @ 15 bits** (`TEXTURE_SCALE = 1/32768`) |
| 16 | 4 | LIGHT | u16 × 2 normalized; `encodeLightMapTexCoord`: `(sl<<8)+2048`, `(bl<<8)+2048` — vanilla lightmap coords 0..255 with the ¼-texel centering offset **baked into the vertex** |

Key point: even in 2020, Sodium **never** stored a tile index — UVs are *absolute, whole-atlas-normalized* coordinates. Position is *chunk-model-relative* (0..32 blocks around the section origin), never world-absolute.

Source: `format/hfp/HFPModelVertexType.java`, `format/sfp/SFPModelVertexType.java`, `format/ModelVertexUtil.java` (all at tag `mc1.16.5-0.2.0`; raw URLs under `https://raw.githubusercontent.com/CaffeineMC/sodium-fabric/mc1.16.5-0.2.0/src/main/java/me/jellysquid/mods/sodium/client/render/chunk/format/`).

### 1.3 Sodium 0.3.x — MC 1.17 (adds a chunk-id field)

Source: tag `mc1.17.1-0.3.4`, `format/sfp/ModelVertexType.java` + `ModelVertexBufferWriterUnsafe.java`. **20 bytes:**

| Offset | Field | Encoding |
|---|---|---|
| 0..5 | POSITION | u16 × 3, `(MODEL_ORIGIN=8 + v) * 65536/32` ⇒ **-8..+24 blocks @ 1/2048 block** (16³ section + 8-block overhang for walls/fences/slabs) |
| 6..7 | chunkId | u16 raw (render-chunk/section id) |
| 8..11 | COLOR | u8 × 4 (AO in alpha) |
| 12..15 | BLOCK_TEXTURE | u16 × 2 whole-atlas absolute |
| 16..19 | LIGHT_TEXTURE | u16 × 2 light-map texcoords |

### 1.4 Sodium 0.5.x — the famous 16-byte "compact" format

Source: tag `mc1.20.1-0.5.1` commit `0a096a73`, `render/chunk/vertex/format/impl/CompactChunkVertex.java` (linked from issue #2004). **16 bytes = 4 × u32, exposed as ONE `UNSIGNED_INT × 4` attribute:**

```
ptr+0:  (pos_y:u16 << 16) | pos_x:u16
ptr+4:  (draw_params:u16 << 16) | pos_z:u16
        draw_params = (section_index & 0xFF) << 8 | (material_bits & 0xFF)
ptr+8:  (light:u8 << 24) | color:u24
        light = block:4 | sky:4        (two 0-15 nibbles!)
        color = RGB with AO *multiplied into the channels* (alpha consumed as AO shade)
ptr+12: (tex_v:u16 << 16) | tex_u:u16  (absolute whole-atlas texel coords, 65536² addressable)
```

- `encodePosition(v) = (8.0 + v) * (65536/32)` → **-8..+24 block range, 1/2048-block precision**, exactly like 0.3.x.
- `encodeTexture(v) = min(0.99999997, v) * 65536` → 16-bit absolute atlas UV ("…16-bit, which gives them the ability to address *every texel* on a 65536×65536 atlas… modern GPUs are actually limited to 16384×16384" — jellysquid, issue #2004).
- `encodeLight(l)`: block light in low nibble, sky light in high nibble — **each 0-15, 4 bits** (this is exactly our "2 light layers 0-15" requirement; vanilla keeps 0-255 lightmap coords, sodium 0.5 threw that away).
- AO is **not a separate field** — it is pre-multiplied into the 24-bit RGB color (old alpha slot).
- Section index is **8 bits** (0..255; 1.18+ worlds have ≤ 24 sections/column), material 8 bits (`MaterialParameters`, of which only 3 bits were used: mipped + alpha-cutoff 2 bits).

### 1.5 Sodium 0.6 → 0.9 (current `CaffeineMC/sodium` main) — 20 bytes, the "more precision" format

Source: `common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java` and shader `common/src/main/resources/assets/sodium/shaders/include/chunk_vertex.glsl`. **20 bytes:**

| Offset | Attribute (GL type) | Layout |
|---|---|---|
| 0 | `a_Position` RG32_UINT | 60 bits of position: 3 coordinates × **20 bits**, split `hi=(v>>10)&0x3FF` in word0 bits [0,10,20], `lo=v&0x3FF` in word1 bits [0,10,20] |
| 8 | `a_Color` RGBA8_UNORM | `ColorARGB.mulRGB(color, ao)` — AO multiplied into RGB (A unused) |
| 12 | `a_TexCoord` RG16_UINT | per axis: **15 bits** quantized whole-atlas UV + **1 bit = sign of inset bias** (see §2) |
| 16 | `a_LightAndData` RGBA8_UINT | R=block light, G=sky light (each clamped `((l & 0xFF) + 8)` to **8..248** — vanilla 0-255 lightmap coords + centering offset), B=material bits:u8, A=**section index:u8** |

Shader decode (verbatim constants from `chunk_vertex.glsl`):
```
POSITION_BITS = 20; TEXTURE_BITS = 15;
VERTEX_SCALE = 32.0 / (1<<20);  VERTEX_OFFSET = -8.0;      // -8..+24 blocks @ 1/32768 block
_vert_position = deinterleave_u20x3(a_Position) * VERTEX_SCALE + VERTEX_OFFSET;
_vert_tex_diffuse_coord = (a_TexCoord & 0x7FFF) / 32768.0;
_vert_tex_diffuse_coord_bias = mix(-1.0, 1.0, a_TexCoord >> 15);   // bleed-inset direction
_vert_tex_light_coord = vec2(a_LightAndData.xy) / 256.0;
_material_params = a_LightAndData[2];  _draw_id = a_LightAndData[3];
```
`_draw_id` (the section byte) is decoded to a section-relative world translation in the vertex shader:
`uvec3(_draw_id) >> uvec3(5,0,2) & uvec3(7,3,7)` — X:3 bits, Y:2 bits, Z:3 bits ⇒ **256 sections per render region (8×4×8)** × 16 = 128×64×128-block region (`RenderRegion.REGION_WIDTH/HEIGHT/LENGTH = 8/4/8`).

### 1.6 Comparison table

| Version (MC) | Bytes | Position | UV | Light | AO | Section idx | Material |
|---|---|---|---|---|---|---|---|
| 0.2.0 SFP (1.16.5) | 32 | 3×f32 | 2×f32 | 2×u16 lightmap | in alpha | — (uniform offset) | — |
| 0.2.0 HFP (1.16.5) | 20 | 3×u16 @1/2048 blk | 2×u16 @15-bit atlas | 2×u16 lightmap | in alpha | — | — |
| 0.3.4 (1.17) | 20 | 3×u16 @1/2048, −8..24 | 2×u16 atlas | 2×u16 lightmap | in alpha | u16 | — |
| 0.5.1 (1.20) | 16 | 3×u16 @1/2048, −8..24 | 2×u16 @16-bit atlas | 2×4-bit (0-15) | ×into RGB | u8 | u8 |
| 0.6–0.9 (1.21+) | 20 | 3×20-bit @1/32768 | 2×(15-bit+bias sign) | 2×u8 (8..248) | ×into RGB | u8 | u8 |
| **VoxelCraft today** | **40** | 3×f32 | 2×f32 block-units + 2×f32 tile | 3×f32 | f32 (in `light`) | — | — |

---

## 2. Texture coordinates & atlas sampling (Q2)

**Sodium does *not* use "tile index + local fract".** In every version the vertex carries **absolute, whole-atlas-normalized UVs** (15–16 bits per axis), pointing directly into vanilla's single 2D block-atlas texture (`uniform sampler2D u_BlockTex`, terrain fragment shader `blocks/block_layer_opaque.fsh`). Vanilla MC 1.16.5 itself uses the different "midTexCoord" trick (per-quad tile-center attribute used to fract-wrap and clamp mips); Sodium 0.5+ replaced it with a cheaper scheme:

1. **Quantized absolute UV** (15 bits per axis over the whole atlas).
2. **Per-quad CPU-computed texel centroid** — the encoder averages the 4 vertices' UVs (`texCentroidU/V`) *per quad on the CPU*, but does **not** store it. Instead each vertex stores only **1 bit per axis: the sign of (v − centroid)** (`encodeTexture`: `bias = (x < center) ? 1 : -1`, stored in bit 15).
3. The vertex shader reconstructs the inset: `v_TexCoord = (_vert_tex_diffuse_coord_bias * u_TexCoordShrink) + _vert_tex_diffuse_coord` — the shrink epsilon is a *uniform* (`u_TexCoordShrink`, ~1 texel), so the inset is not quantized into the vertex data. This prevents atlas bleeding between adjacent tiles at mip 0.
4. Higher mips are handled in the fragment shader, not by the sampler: `sampleNearest()` snaps the interpolated UV back onto texel centers (using `dFdx/dFdy` screen-space derivatives) and samples with `textureGrad`; `sampleRGSS()` does 4-tap rotated-grid supersampling with `textureLod` at a hand-computed mip, blended in by screen-space texel size — this is Sodium's shimmer/aliasing fix for atlas mipmap filtering.
5. **No texture arrays are used for terrain** (`sampler2D` in all versions, incl. current). Atlas mipmaps beyond tile size are simply not usable, hence the manual mip logic. (This is the price of a single-atlas design and the main argument for `texture_2d_array` in a new engine — see §8.)

Implication for VoxelCraft's **greedy mesher** (Sodium itself never greedy-meshes; it emits per-face quads): with a single 2D atlas you cannot use `AddressMode::Repeat` (it would wrap across tile boundaries), so the shader must do `atlas_uv = tile_origin + fract(local_uv) * tile_size` **plus** manual mip selection (`log2(screen-space derivative)` clamped to ≤ `log2(tile_size)`) — exactly the vanilla/sodium approach. Alternatively a `texture_2d_array` gives per-layer Repeat + clean per-layer mips for free (§5).

---

## 3. Multi-Draw Indirect in wgpu 22 — native vs web (Q3)

All claims verified against wgpu 22.1.0 / wgpu-hal 22.0.0 sources (local cargo registry) and docs.rs/wgpu/22.1.0:

| Capability | Vulkan/DX12 native | Metal native | **WebGPU backend (wasm)** | **WebGL2 backend (wasm)** |
|---|---|---|---|---|
| `Features::MULTI_DRAW_INDIRECT` (`multi_draw_*_indirect(buffer, offset, count)`) | ✅ DX12, Vulkan | ✅ **emulated** on top of `draw_*_indirect` loop | ❌ **never exposed** | ❌ never exposed |
| `Features::MULTI_DRAW_INDIRECT_COUNT` | ✅ Vulkan 1.2+ / `VK_KHR_draw_indirect_count` **only** | ❌ | ❌ | ❌ |
| Single `draw_indexed_indirect(buffer, offset)` | ✅ | ✅ | ✅ (→ WebGPU core `drawIndexedIndirect`) | ❌ **validation error** (see §4) |
| `INDIRECT_FIRST_INSTANCE` (non-zero firstInstance) | ✅ (mostly) | ✅ | ◐ optional WebGPU feature `indirect-first-instance` | ❌ |

Evidence, wgpu 22.1.0:

- `wgpu::Features` docs (docs.rs/wgpu/22.1.0): *"MULTI_DRAW_INDIRECT … Supported platforms: DX12, Vulkan, Metal on Apple3+ or Mac1+ (Emulated on top of draw_indirect and draw_indexed_indirect). **This is a native only feature.**"* Same for `MULTI_DRAW_INDIRECT_COUNT` (Vulkan 1.2+ only) and `INDIRECT_FIRST_INSTANCE` (*"Not Supported: OpenGL ES / WebGL"*).
- wgpu's WebGPU backend (`wgpu-22.1.0/src/backend/webgpu.rs`): `FEATURES_MAPPING` is a **fixed 11-entry table** of standard WebGPU feature names (depth-clip-control, texture-compression-*, timestamp-query, indirect-first-instance, shader-f16, etc.). `MULTI_DRAW_INDIRECT` is not in it and cannot be: it is not a standard WebGPU feature. `render_pass_multi_draw_indexed_indirect` literally `panic!`s unless the feature is enabled — i.e. unreachable on web.
- Chrome 131 (Nov 2024) shipped **experimental** MDI as the non-standard GPU feature `"chromium-experimental-multi-draw-indirect"` behind `chrome://flags/#enable-unsafe-webgpu` (multiDrawIndirect/multiDrawIndexedIndirect methods). wgpu 22 does **not** request browser-experimental features, so this is unusable from wgpu today (and it is Chrome-only, unsafe-flag-gated, unstandardized — gpuweb issue #1354 tracks standardization).
- In the browser world, single **`drawIndexedIndirect` is core WebGPU** (MDN: `GPURenderPassEncoder.drawIndexedIndirect`; if `indirect-first-instance` is disabled and `firstInstance != 0` the call is a no-op). So on WebGPU the standard fallback is: **a CPU loop of `draw_indexed_indirect` calls, one per section, all reading different offsets of one prebuilt args buffer** — no per-draw state changes, no CPU readback, and GPU-culled sections can simply have `index_count = 0` written by a compute pass. This is the pattern used by WebGPU engines pending MDI standardization.

### wgpu 22 status summary (Q3, direct answer)

- `MULTI_DRAW_INDIRECT` / `MULTI_DRAW_INDIRECT_COUNT`: **native-only in wgpu 22; not on WebGPU; not on WebGL2.** Even desktop-GL doesn't get them in wgpu 22 (the gles adapter never sets these features, though plain indirect works there at GL 4.3/GLES 3.1+).
- Standard web fallback: **WebGPU → loop of `draw_indexed_indirect`** (args from a buffer; fixed upper bound, zero-count padding for culled sections). **WebGL2 → plain `draw_indexed` loop** (indirect is entirely unavailable — next section).

---

## 4. Indirect draws on the WebGL2/GL backend (Q4)

Verified in wgpu-hal 22.0.0 `src/gles/`:

- The GL backend implements `draw_indirect`/`draw_indexed_indirect` by pushing one command per draw and calling `gl.draw_arrays_indirect_offset` / `gl.draw_elements_indirect_offset` (`command.rs` L1075-1117, `queue.rs` L265-287). "Multi" = a plain CPU loop. `draw_*_indirect_count` are `unreachable!()` (never advertised).
- Indirect support is gated by `DownlevelFlags::INDIRECT_EXECUTION`, which the gles adapter sets only when `supported((3,1),(4,3)) || GL_ARB_multi_draw_indirect` (`adapter.rs` L387-390) — i.e. **GLES 3.1 / GL 4.3 / the ARB extension**.
- wgpu-core validates every indirect call with `require_downlevel_flags(INDIRECT_EXECUTION)` (`command/render.rs` L2478, L2554) → a draw on a device without the flag is a **validation error/panic**, not a silent fallback.
- **WebGL2 is GLES 3.0**: `gles/mod.rs` documents *"`WebGL 2` version returned as `OpenGL ES 3.0`"* (context creation always requests `webgl2`). ES 3.0 has **no** `glDrawElementsIndirect` in the JS API at all (that arrived in ES 3.1, which the web never exposed).
- Conclusion: **`draw_indexed_indirect` does NOT work on wgpu's WebGL2 backend** — the only path is regular `draw_indexed` per section with CPU-decided visibility. Also note `INDIRECT_FIRST_INSTANCE` is explicitly "Not Supported: OpenGL ES / WebGL" even on desktop GL (it needs GL 4.2 + `ARB_shader_draw_parameters`; the backend "pretends `first_instance` is 0" on GLES — `gles/mod.rs` header comment).
- wgpu master (2026) has the same gating (`indirect_execution = supported((3,1),(4,3)) || ARB_draw_indirect&&compute`) — no WebGL2 emulation has been added.

Practical consequence: a dual-target renderer needs **three draw paths**:
1. native (Vulkan/DX12): `multi_draw_indexed_indirect` after GPU culling (one call per region/pass);
2. WebGPU: fixed-size CPU loop of `draw_indexed_indirect` over the same args buffer (zero-count args for culled sections — still one bind group, no readback);
3. WebGL2: CPU loop of `draw_indexed` with per-section ranges; keep *all* per-draw variation out of the draw loop (bake section origin/section id into the vertex stream like Sodium does, so one pipeline + one bind group serves every draw).

---

## 5. `texture_2d_array` in WGSL on WebGL2 (Q5)

**Yes — supported.** Verified in wgpu-hal 22.0.0 `src/gles/`:

- `TextureViewDimension::D2Array` maps to `glow::TEXTURE_2D_ARRAY` (`gles/mod.rs` L400, `conv.rs` L310); creation, view binding, uploads (`queue.rs` L834) and copy origins (`queue.rs` L34) all handle the array case.
- `TEXTURE_2D_ARRAY` is **core GLES 3.0 = core WebGL2** (`sampler2DArray` is a WebGL2/GLSL-ES-3.00 builtin; see webgl2fundamentals / Khronos). Naga's GLSL backend emits `sampler2DArray` + `vec3(u, v, layer)` sampling for WGSL `texture_2d_array` — no extensions needed.
- docs.rs `wgpu::TextureViewDimension`: *"A two dimensional array texture. `texture_2d_array` in WGSL and `texture2DArray` in GLSL."*
- **Limit**: `wgpu::Limits::max_texture_array_layers` defaults to **256** (WebGPU default limit, matching the WebGL2 spec minimum for `MAX_ARRAY_TEXTURE_LAYERS`). A 1-tile-per-layer array for "~400 max tiles" would need >256 layers — you may request a higher limit where the adapter allows it, but the *portable* designs are: (a) single 2D atlas (Sodium-style, §2), or (b) array with multiple tiles per layer (but then per-layer `Repeat` wrapping bleeds across the tiles in the layer, defeating the purpose) or (c) small atlas + ≤256 hot tiles.
- One historical caveat: wgpu issue #2161 (Nov 2021) — GLES backend *guessed* texture dimension and mis-typed 1-layer `texture_2d_array` as plain 2D (silent failure). The modern backend requires/infers the view dimension explicitly (`mod.rs`: "`D2` textures with `depth_or_array_layers > 1` are assumed to have view dimension `D2Array`") — **always create the view with an explicit `view_dimension: D2Array`** and this is a non-issue on wgpu 22.

So: texture arrays are available on **both** web backends and natively; the only real constraint is the 256-layer guaranteed-minimum, which is why the single-atlas + shader-side tile-wrap remains the safest default for 44→400 tiles.

---

## 6. Per-chunk buffers vs one global buffer (Q6)

### Sodium 0.5.x — one arena buffer per region (2×2×2 sections? no: per *render region*)

`GlBufferArena` (tag `mc1.20.1-0.5.1`, `client/gl/arena/GlBufferArena.java`):

- One big GPU buffer per region & data type (vertex 20 B stride, index 4 B stride), sub-allocated with a **linked-list segment manager with a single free head segment** (`GlBufferSegment`), `STATIC_DRAW` storage.
- Growth: `resizeIncrement = initialCapacity / 16`; when an allocation doesn't fit, the arena **compacts**: it takes all used segments, builds a `buildTransferList` of GPU copy commands (glCopyBufferSubData-style) moving live data to the front, and rewrites every section's stored offset (sections keep `BufferSegment` handles; `PendingBufferCopyCommand`s are recorded and the draw parameters re-pointed). Fragmentation is handled by *resize-with-compaction*, not by free-list merging.
- Uploads go through a **`StagingBuffer`** (persistent-mapped) with fence-based synchronization, so meshes upload asynchronously without stalling the frame.

### Sodium 0.6+ (current) — multi-arena `ArenaAggregator` with explicit defrag budgets

`common/.../gpu/arena/ArenaAggregator.java` + `RenderRegion.java` + `SectionRenderDataStorage.java`:

- Regions are **8×4×8 = 256 sections** (128×64×128 blocks). `SECTION_VERTEX_COUNT_ESTIMATE = 756` verts/section; per-section buffer estimate ≈ 756×20 B + 1134×4 B ≈ 20 KiB.
- **Separate arena pools per data type**: `index` (4 B elements, arena size 16/32/64 MiB by arena count, capacity = required×3) and `geometry` (20 B elements, 32/128/256 MiB, capacity = required×2…×7 over-allocation). `MAX_DYNAMIC_BUFFER_SIZE = 1.5 GiB`.
- Free space tracked with a **`SizedTreeMap` (best-fit size-indexed free tree)** per arena; regions own `RegionAllocatorHandle`s that can be **moved between arenas** (live data copied, offsets rebased via `onSegmentChanged`).
- **Defragmentation is budgeted per frame**: `DEFRAG_COPIES_PER_FRAME = 32 per GiB`, `DEFRAG_BYTES_PER_FRAME = 32 MiB per GiB`; "resize to compact" triggers when one arena's free space ≥ 5% of total free (`RESIZE_TO_COMPACT_TOTAL_FREE_FRACTION`), with a 10% compaction margin; deallocation pauses if allocation rate exceeds 2%/s of total memory. Freed arenas (up to 8 buffers) are cached and reused if ≤ 1.4× the requested size (`MAX_BUFFER_REUSE_SIZE_FACTOR`).
- Per-section mesh data is stored in **native heap memory arrays** (`SectionRenderDataUnsafe`, 256 entries) holding base_vertex, per-facing vertex counts, `sliceMask` & `facingList` — the CPU mirror of the GPU segments, so indirect draw args are assembled on the CPU in O(sections) with no GPU reads.
- Sections store vertices **grouped by quad facing** (ModelQuadFacing buckets) — enables cheap translucent sorting (`translucent_sorting/data/*`, including `DynamicData` GFNI-style quads re-sorted on camera move) and "shared index buffer" reuse (`SharedQuadIndexBuffer`: most sections share the same 0..N quad index pattern, indices are not stored per section unless sorting requires it).
- Rendering: **one `MultiDrawBatch` (GLDrawBatch → glMultiDrawElementsIndirect; VKMultiDrawBatch / VKIndirectDrawBatch on the new Vulkan backend) per region per terrain pass** — i.e. draw calls scale with *regions*, not sections.
- Issue #3809 etc. show the operational cost: "Overflowed the mesh time buffer" — arenas + staging have hard capacity budgets and back-pressure (build tasks get queued/dropped), worth replicating (bounded upload bytes/frame).

### Other engines / general practice

- The universal baseline (Unreal/Unity/forum consensus, vkguide): **one shared vertex/index buffer per material, many draw ranges** — "You will want a singular vertex buffer, and multiple index buffers. Then dispatch a series of batched draw calls using the same vertex buffer" (Unreal forums). nickmcd.me's voxel series splits chunk meshes into **6 vertex pools bucketed by face direction** so each pool is a single contiguous buffer + one draw per direction per chunk (helps deferred-light batching).
- **"ModernUO"** is an Ultima Online server emulator (C#) — it has no meshing/GPU layer; the task's mention is presumably "modern MC-like engines". The relevant modern references are Sodium (above), VulkanMod (uses per-chunk buffers + explicit section invalidation — cf. "Iodium" mod optimizing exactly that), and Distant Horizons.
- WebGL2-practical pattern (what wgpu/WebGL2 engines converge on): per-region `wgpu::Buffer` arenas with `queue.write_buffer` staging, per-section draw ranges; global single buffers are used on WebGPU where `write_buffer` + big single allocation avoids many buffer objects; on WebGL2 many small buffers are actually fine (buffer re-spec is the expensive part, avoid `Buffer::destroy` churn).

### What VoxelCraft should take from this

- Keep per-region GPU buffers (not global): a 32×256×32 region (2×16×2 sections = 64 sections… or sodium-style 8×4×8=256-section regions later) → one vertex arena + one index arena per region, sized `sections × ~20 KiB × overalloc(×1.5-2)`, grown by 1/16th increments, compacted by the sodium 0.5 scheme (copy-live-to-front via `queue.copy_buffer_to_buffer` + rebased offsets) when free-in-middle exceeds ~30% (simple threshold is fine at our scale), **bounded upload budget per frame** (e.g. 4 MiB/frame) to prevent mesh-time overflow stalls.
- Track per-section `(buffer_offset, vertex_count, index_offset, index_count)` CPU-side (like `SectionRenderDataUnsafe`) so indirect-args assembly is a memcpy, and rebuild = free old segment + alloc new (copy-on-write upload path through a small staging queue).
- Store vertices bucketed by facing is optional; the win for us is per-section draw args + no re-upload of untouched sections.

---

## 7. Sectional dirty tracking (Q7)

**Vanilla 1.16.5 semantics, as preserved by Sodium 0.2.0** (`mixin/features/chunk_rendering/MixinWorldRenderer.java`, tag `mc1.16.5-0.2.0` — vanilla method bodies overwritten with equivalent calls):

```java
// single block change:  (vanilla LevelRenderer.scheduleBlockRenders(x,y,z))
scheduleRebuildForChunks(x - 1, y - 1, z - 1, x + 1, y + 1, z + 1, important=false);
// i.e. sections(x-1..x+1, y-1..y+1, z-1..z+1) — dedup via section coords (>>4 of the ±1 box)

// single section change: (vanilla scheduleSectionRender(pos, important))
scheduleRebuildForBlockArea(pos - 1, pos + 1, important);

// area change (explosions, sponge, worldgen): scheduleBlockRenders(min..max)
scheduleRebuildForChunks(min>>4 … max>>4)  // all sections the AABB touches
```

So the canonical rule: **a block edit marks its own 16³ section dirty, plus any section whose boundary is within 1 block of the edit** (edge blocks: `x&15==0 → +X neighbor section`, `x&15==15 → −X`, same for Y/Z; corner edits mark up to 2³=8… in practice the ±1 box dedups to 1, 2, or 4 sections). The ±1 box also covers AO/light seams one block deep across the border.

Modern engines add on top (Sodium 0.6+ `ChunkUpdateTypes`):

- **Update types as bitflags with priorities**: `SORT | REBUILD | IMPORTANT | INITIAL_BUILD`; `INITIAL_BUILD(0) > REBUILD(1) > SORT(2)`; "important" (player-caused, e.g. block place/break) tasks jump the queue and can block ≤1 frame ("defer mode 0/1 frame") while background chunk-load rebuilds defer indefinitely. This kills the classic "placing a block stutters the frame" problem.
- Light updates: sodium's light data caches read a **2-block-radius apron** around each section (`ArrayLightDataCache`: `BLOCK_LENGTH = 16 + 2×2`) because AO/face light of border blocks depends on light 2 blocks away — i.e. a light change of N blocks reach must dirty every section whose *apron* intersects the changed cells. Vanilla block-light BFS spreads the "dirty section" notification alongside the BFS the same way.
- Chunk *load/unload* of a neighbor → mark border sections of adjacent loaded chunks dirty (reddit r/VoxelGameDev consensus + gamedev.se "Updating chunk borders when generating new chunks": only mesh a section when all 6 neighbors exist *or* emit border faces as unculled).
- Common voxel-engine practice summary (bugnet.io, lets-make-a-voxel-engine, unity threads): 16³ section granularity, own section + border-adjacent sections on edit, dirty queue with priority, never remesh a whole column.

**Current VoxelCraft** (verified in `src/world.rs`): dirty set is **chunk-column granularity** (16×256×16), `set_block` marks own chunk + X/Z border neighbors (no Y sections exist, no light-BFS-driven dirty marking, no priorities). Moving to 16³ sections with the ±1 rule + priorities is a straightforward port of the sodium 0.2.0 mixin semantics above.

---

## 8. Design recommendations for VoxelCraft — corrected packed vertex & rendering plan

### 8.1 Why `UltraPackedVertex` (roadmap §B) cannot work — the math

- 6-bit U + 6-bit V = 4096 steps over a **2048-px** atlas ⇒ 0.5-px steps at best, and with 16-px tiles only 64 positions *inside a tile* (4/block) — sub-block faces (slabs/water) quantize to ¼-tile error, and there is **no way to express greedy runs > 1 tile** without fract logic that itself needs the tile origin (another 14 bits for a 128×128-tile atlas: 7+7). Sodium uses 15-16 bits per axis for a reason; even vanilla's absolute-float UV is 32 bits/axis.
- 5-bit X/Z only addresses 0-31 — usable *only* as section-relative, but then a section id (≥6 bits) + a per-draw region origin must exist somewhere, which the roadmap's layout has no room for.
- 8 bytes (64 bits) total is below Sodium's *minimum* ever shipped (16 B) once you count: position 3×16=48 bits + UV ≥ 24 bits + light 8 bits + section 6-8 bits + normal 3 + AO 2 ⇒ **≥ 91 bits ≈ 12 B floor without a state id, 16 B with one**. See the layout below — it lands exactly on Sodium's own 16 B while carrying *more* than Sodium (state id + explicit AO/normal).

### 8.2 Recommended vertex format — **16 bytes (4 × u32), one `Uint32x4` attribute**

Section-relative positions + section id in-vertex (Sodium-proven; required for GPU-driven indirect draws later). All fields unsigned integers, decoded in the vertex shader with shifts/masks (WGSL `u32` math — works on Vulkan/DX12/Metal/WebGPU **and** WebGL2/GLSL-ES-3.00, which has native uint attributes & bit ops; naga handles the translation).

```
VC-16 vertex (4 × u32, little-endian bit packing, LSB = bit 0):

w0 =  z:u16 << 16 | x:u16
w1 =  flags:u16 << 16 | y:u16
w2 =  tile:u14 << 18 | u:u8 << 10 | v:u8 << 2 | bias:u2
w3 =  state:u16 << 16 | section:u8 << 8 | sky:u4 << 4 | block:u4

flags (w1[31:16]) = normal:3 | ao:2 | material:4 | spare:7
```

| Field | Bits | Value range | Why exactly this many bits |
|---|---|---|---|
| `x, y, z` | 16 each | −8…+24 blocks @ **1/2048 block**; encode `(v + 8) × 2048` | Section-relative like Sodium 0.2→0.5: 32-block span covers a 16³ section + an 8-block overhang on each side for geometry that pokes out (slabs, walls, fences, tall grass, water surface). 2048 steps/block = Sodium's exact precision (their 0.6 bump to 20 bits/1-32768 was for *mods* complaining, not MC-native geometry). 16 bits keeps each axis word-aligned for cheap pack/unpack. |
| `section` | 8 | 0..255 | A 32×256×32 region = 2×16×2 = **64 sections → 6 bits would do**, but 8 bits is Sodium parity, costs nothing here (w3 has exactly 8 free), and allows a future 8×4×8-section (128×64×128) region with the same shader decode (`section >> (5,2) & (7,7,3)`-style). **Required** in-vertex once draws become indirect (no per-draw uniform), which is precisely why Sodium packs it. |
| `tile` | 14 | 0..16383 | 2048² atlas ÷ 16² tiles = 128×128 = **16384 slots = 14 bits exactly**. Requirement is only ~400 max (10 bits), but the other 4 bits are free in w2 and full-slot addressing removes any future atlas-repack constraint. (If the 2 spare bits in w2 are ever needed for animation flags, `tile:u10` still covers 1024 tiles ≈ 2.5× the 400-tile max.) |
| `u, v` (tile-local) | 8 each | 0..16 tile-units, **16 steps/block = 1 texel** for 16-px tiles | Greedy quads span up to 16 tiles; a local range of [0,16) with 256 steps is texel-exact at 16-px tiles (256 = 16 blocks × 16 steps). Shader wraps with `fract(f32(u) / 16.0)` — this *replaces* the current `uv: [f32;2] + fract()` scheme at ¼ the bits and keeps sub-block faces (water at 14/16 height etc.) representable to 1/16 block. |
| `bias` | 2 | ±1 per axis | Sodium-0.6's texture-bleed inset: each vertex stores only the *sign* of (uv − quad-centroid); the fragment shader applies `uv += bias × u_texel_shrink` (uniform ε). Kills atlas bleeding without spending an epsilon in the vertex data. |
| `sky`, `block` | 4 + 4 | 0..15 each | Requirement: two light layers 0-15. Sodium 0.5 used exactly two nibbles; our engine has no lightmap *texture* (we compute the curve in-shader), so 4 bits per layer is lossless. |
| `state` | 16 | 0..65535 | Requirement: state ids up to u16. Note: **neither vanilla nor Sodium stores block ids in vertices** — if no shader logic needs it (tint/variation can come from the material bits), drop this word-pair and use the §8.3 variant. If kept, it enables per-vertex biome tint/emissive lookup without extra buffers, and survives the 57→~400-block→15k-state growth without a repack. |
| `normal` | 3 | 6 faces + 1 spare | Requirement "6+ normals": 3 bits = 8 values (±X, ±Y, ±Z, + "none"). Face shading in shader (`shade[normal]`), replaces the current per-vertex `light: f32` face-shade float. |
| `ao` | 2 | 0..3 | Requirement: 4 AO levels. (Sodium instead multiplies AO into the 24-bit RGB color; we have no color channel, so 2 explicit bits is the cheap equivalent — `ao_factor = [0.45, 0.65, 0.85, 1.0]`.) |
| `material` | 4 | 16 flags | Sodium's field is 8 bits with 3 used (mipped, alpha-cutoff ×2). 4 bits covers: cutout, mips-off, translucent/water, emissive-glow, tinted, animated — everything our two passes (solid/water) + graphics presets need. |
| spare | 7 (flags) + 2 (w2) | — | Headroom for animated-tile frame, water-flow phase, custom tint index. |

**Size: 40 B → 16 B per vertex (−60%), equal to Sodium 0.5.1, below Sodium 0.9's 20 B.** At Sodium's own estimate of ~756 verts/section ≈ 12 KiB/section; a 64-section region ≈ 0.8 MiB vertex arena (vs 2 MiB today).

WGSL decode sketch:
```wgsl
@location(0) v_data: vec4<u32>;   // one Uint32x4 attribute, 16-byte stride
let x = f32(v_data.x & 0xFFFFu) * (32.0 / 65536.0) - 8.0;          // world = section_origin + this
let y = f32(v_data.y & 0xFFFFu) * (32.0 / 65536.0) - 8.0;
let z = f32((v_data.x >> 16u) & 0xFFFFu) * (32.0 / 65536.0) - 8.0;
let tile = (v_data.z >> 18u) & 0x3FFFu;                             // 0..16383
let tile_origin = vec2<f32>(f32(tile & 127u), f32(tile >> 7u)) * 16.0 / 2048.0; // in atlas UV units
let local = vec2<f32>(vec2<f32>((v_data.z >> 10u) & 0xFFu, (v_data.z >> 2u) & 0xFFu)) / 256.0; // 0..1 per 16 tiles
let uv = tile_origin + fract(local) * (16.0 / 2048.0) + bias_sign * u_texel_shrink;
let block_light = f32(v_data.w & 0xFu); let sky_light = f32((v_data.w >> 4u) & 0xFu);
let section = (v_data.w >> 8u) & 0xFFu; let state = (v_data.w >> 16u) & 0xFFFFu;
// section → world offset: x=(section&1)*16 (region 2×16×2), y=(section>>1)*16, z=(section>>9)*16
```
(For WebGL2/GLSL the same math compiles from naga; `Uint32x4` vertex attributes are core ES 3.0.)

**Optional 12-byte variant** (drop `state:u16`, keep everything else): 48 (pos) + 11 (flags: normal3+ao2+mat4+bias2) + 28 (tile:10+u8+v8) + 8 (light) + 8 (section) = **103 bits → pads to 13 B; only a 12-B layout by also trimming `tile`→10 and moving `section` to a per-draw dynamic-uniform offset** — worthwhile only if profiling shows bandwidth-bound draws; otherwise keep the uniform 16-B format and its GPU-driven future. **Do not chase 8 B.**

### 8.3 Atlas strategy (44 tiles → ~400 max, 2048², 16-px tiles)

- **Primary (portable): single 2048² 2D atlas + shader tile-wrap** — `tile:u14` + fract + manual mip clamp `lod = clamp(log2(screen_deriv), 0, log2(16))`, plus the 2-bit bias inset for bleed (this is the vanilla+sodium recipe and needs no `Repeat` sampler). Use `textureSampleLevel`/`textureGrad` (WGSL) — both fine on WebGL2 via naga (ES 3.00 has `textureGrad`, `textureLod`).
- **Optional (WebGPU + desktop-GL where `max_texture_array_layers ≥ 512`): `texture_2d_array`, layer = tile** — gives true per-tile `Repeat` (greedy runs wrap natively!), correct per-layer mips, zero bleed, and deletes the fract/bias shader logic (uv becomes plain `vec2<f32>` from u8 fields; layer = tile index). Guaranteed-minimum is only 256 layers, so gate: request `max_texture_array_layers: 512`, fall back to the atlas path if the adapter refuses. 400 max tiles ⇒ 512 layers fits desktop/mobile-class WebGPU today.
- Keep procedural-tile generation unchanged; both paths consume the same tile registry (44 today).

### 8.4 Rendering strategy (answering Q3/Q4 concretely for VoxelCraft)

```
if adapter.features.contains(MULTI_DRAW_INDIRECT)        // Vulkan/DX12 native
    → GPU cull (compute) writes DrawIndexedIndirectArgs[64] + counts=0 for culled;
      one multi_draw_indexed_indirect call per region-pass.   (args: 20 B each: count,instance,firstIndex,baseVertex,firstInstance=0)
else if backend is WebGPU                                  // browser WebGPU
    → same args buffer, but a CPU loop: for i in 0..visible_cap { draw_indexed_indirect(&args, i*20) }
      (zero-count args make culled sections no-ops; one pipeline, one bind group; still no CPU readback)
else                                                        // WebGL2 fallback
    → CPU-side visibility (existing frustum code), loop of draw_indexed per section;
      all per-section data lives in the vertex stream (section id) + one dynamic-uniform bind group
      (wgpu GL backend supports dynamic offsets) → no set_bind_group in the loop.
```
- Keep `first_instance = 0` everywhere (INDIRECT_FIRST_INSTANCE is unavailable on web/GL — MDN: non-zero firstInstance is a no-op without the feature).
- Draw-count scale: 32×256×32 region = 64 sections; render distance 8 ⇒ ~289 columns ⇒ ~4.6k sections total, ~1-2k visible ⇒ WebGPU loop ≈ 1-2k cheap indirect calls (fine), WebGL2 ⇒ prefer *merging sections per region into contiguous ranges* (sort by region, draw `draw_indexed` ranges of the region arena; degrade to per-section only when translucent sorting requires).
- Index buffer: shared quad index pattern per facing-bucket (sodium `SharedQuadIndexBuffer` idea) — most solid-mesh sections can share one big pre-generated 0..4N index buffer; per-section indices only for water/sorting.

### 8.5 Sectional dirty tracking plan

- Port the sodium-0.2.0/vanilla rule: `set_block` → dirty sections of `pos±1` box (dedup via `>>4`); area edits → span; **priorities**: IMPORTANT (player edits, ≤1-frame deferral) vs deferred (worldgen/lighting background); light BFS marks every section whose 2-block apron intersects a changed cell (light/AO of border faces depends on light 2 blocks out).
- Keep the current 3×3-chunk *snapshot* mesher — it already reads a 48×256×48 apron; switching to 16³ sections shrinks remesh cost ~16× on edits (only the affected section + border neighbors rebuild instead of a 16×256×16 column × 3).
- Wire dirty flags into `ChunkUpdateTypes`-style bitflags (REBUILD/SORT/IMPORTANT/INITIAL) from day one so the translucent-sort work later slots in.

### 8.6 Migration order (lowest risk → highest)

1. **VC-16 vertex + section-relative mesher output** (pure data change; shader decode swap; keeps current per-chunk buffers) — immediate 60% vertex-memory/bandwidth cut.
2. **16³ sections + dirty set + priorities** (world.rs/chunk.rs; mesher unchanged logic, new granularity).
3. **Region arenas** (one wgpu buffer per 32×256×32 region, segment free-list, sodium-0.5 compaction-on-overflow, bounded per-frame upload budget).
4. **Draw-path split** (native MDI / WebGPU indirect-loop / WebGL2 draw-loop) behind one `TerrainDrawList` abstraction.
5. Only then: GPU culling compute pass + optional texture-array path.

---

## 9. Sources

Sodium (Java sources, fetched from raw.githubusercontent.com):
- sodium-fabric tag `mc1.16.5-0.2.0`: `client/render/chunk/format/sfp/SFPModelVertexType.java`, `format/hfp/HFPModelVertexType.java`, `format/hfp/HFPModelVertexBufferWriterUnsafe.java`, `format/ModelVertexUtil.java`, `format/DefaultModelVertexFormats.java`; `mixin/features/chunk_rendering/MixinWorldRenderer.java`; `resources/assets/sodium/shaders/chunk_gl20.f.glsl`
- sodium-fabric tag `mc1.17.1-0.3.4`: `client/render/chunk/format/sfp/ModelVertexType.java` + `ModelVertexBufferWriterUnsafe.java`, `format/ChunkMeshAttribute.java`
- sodium-fabric tag `mc1.20.1-0.5.1` (commit `0a096a73`): `client/render/chunk/vertex/format/impl/CompactChunkVertex.java`; `client/gl/arena/GlBufferArena.java`
- CaffeineMC/sodium main (≈0.9.x, cloned): `common/.../render/chunk/vertex/format/impl/CompactChunkVertex.java`; `shaders/include/chunk_vertex.glsl`, `include/chunk_material.glsl`, `blocks/block_layer_opaque.{vsh,fsh}`; `render/chunk/region/RenderRegion.java`; `render/chunk/data/SectionRenderDataStorage.java`; `gpu/arena/ArenaAggregator.java`, `BufferSegment.java`, `RegionAllocatorHandle.java`; `render/chunk/ChunkUpdateTypes.java`; `model/light/data/ArrayLightDataCache.java`
- Issue #2004 "The precision of the modified data is too low…": https://github.com/CaffeineMC/sodium/issues/2004 (20-bit positions / 16-bit UV / 8-bit light in 0.6; jellysquid's UV-precision quote)
- Issues #460/#491 (1.16.5 "compact vertex format" option): https://github.com/CaffeineMC/sodium/issues/491

wgpu 22.1.0 / wgpu-hal 22.0.0 / naga 22.1.0 (local cargo registry `~/.cargo/registry/src/index.crates.io-…/`):
- `wgpu-22.1.0/src/backend/webgpu.rs` — `FEATURES_MAPPING` (11 web features), `render_pass_draw_indexed_indirect` → `draw_indexed_indirect_with_f64`, multi-draw panics
- `wgpu-core-22.1.0/src/command/render.rs` L2451-2557 — `require_features(MULTI_DRAW_INDIRECT)` + `require_downlevel_flags(INDIRECT_EXECUTION)` validation
- `wgpu-hal-22.0.0/src/gles/adapter.rs` L387-390 (INDIRECT_EXECUTION gating), L627 (INDIRECT_FIRST_INSTANCE), `command.rs` L1075-1137 (indirect = per-draw loop; count = unreachable), `queue.rs` L265-297, `mod.rs` L40-85 (WebGL2 = ES 3.0; first_instance emulation), `conv.rs` L310 / `mod.rs` L400 (D2Array → TEXTURE_2D_ARRAY)
- `wgpu-types-22.0.0/src/lib.rs` — `MULTI_DRAW_INDIRECT`/`COUNT` docs ("native only"); `max_texture_array_layers` default 256
- docs.rs: https://docs.rs/wgpu/22.1.0/wgpu/struct.Features.html ; https://docs.rs/wgpu/latest/wgpu/struct.RenderPass.html ("requires DownlevelFlags::INDIRECT_EXECUTION") ; https://docs.rs/wgpu/latest/wgpu/enum.TextureViewDimension.html
- wgpu issue #2161 (1-layer D2Array mis-typed on old GLES): https://github.com/gfx-rs/wgpu/issues/2161
- wgpu master `gles/adapter.rs` (2026): INDIRECT_EXECUTION gating unchanged

Web / WebGPU:
- Chrome 131 "Experimental support for multi-draw indirect" (`chromium-experimental-multi-draw-indirect`, unsafe flag): https://developer.chrome.com/blog/new-in-webgpu-131
- MDN `GPURenderPassEncoder.drawIndexedIndirect` (core WebGPU; firstInstance no-op without indirect-first-instance): https://developer.mozilla.org/en-US/docs/Web/API/GPURenderPassEncoder/drawIndexedIndirect
- gpuweb standardization threads: https://github.com/gpuweb/gpuweb/issues/1354 ; CTS #3961
- WebGL2 texture arrays: https://webgl2fundamentals.org/webgl/lessons/webgl-texture-units.html ; https://betterprogramming.pub/how-to-use-texture-arrays-in-webgl-921dff1c22d8

Voxel-engine practice (dirty tracking / buffers):
- https://bugnet.io/blog/how-to-fix-voxel-block-removal-leaving-floating-mesh-faces ("Dirty neighbor chunks at borders")
- https://www.reddit.com/r/VoxelGameDev/comments/1m8uwgb/neighbor_chunk_problem ; https://gamedev.stackexchange.com/questions/200205/updating-chunk-borders-when-generating-new-chunks
- https://nickmcd.me (voxel series — 6 face-bucketed vertex pools); https://forums.unrealengine.com (single VB + ranges + batched draws); https://www.vkguide.dev/docs/ascendant/ascendant_geometry
