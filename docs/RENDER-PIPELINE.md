# Render pipeline — the wgpu frame, pass by pass

Overview of what the renderer does every frame
(`vc-render/src/render.rs`, `Renderer::render`). This is our own engine's
design (a clean-room implementation, not a port); where a choice exists in
the wider voxel-game design space, the note names the parity or the
deviation. Companion to the 2026-10-06 reference hardware bench:
[BENCH-1A3-reference hardware-2026-10-06.md](BENCH-1A3-reference hardware-2026-10-06.md).

## Frame flow (CPU → GPU)

```
sim (20 Hz fixed) ─▶ streaming (gen/light) ─▶ meshing (CPU rayon | GPU mesher)
      ─▶ uploads (region arenas, origin VB, MDI args, billboard stream)
      ─▶ encoders: [pass 0 shadow] (own submit) + [passes 1..7] (one submit)
      ─▶ present (readback first when F2/E2E capture is armed)
```

The `stream`/`results`/`ui` phase timers and the draw phase are what
`--benchmark` reports (`FramePhases`, µs resolution, rolling 240-frame
window); on the reference hardware the draw phase is ~100% of the frame.

## The passes, in order

| # | label | target | what it draws | when it runs |
|---|---|---|---|---|
| 0 | `shadow` | 2048² packed depth (+ depth view) | terrain only, from the light's ortho camera; depth-only | `shadowq > 0` AND not the Nether (§28: no sun → no shadow pass). Own command encoder + submit — the shadow texture is a color target here and a sampled resource only in the next encoder (wgpu usage scopes) |
| 1 | `scene` | offscreen LINEAR scene tex (MSAA color+depth → auto-resolve when msaa>0) | in order: sky (skipped in skyless dims), terrain (region-grouped near→far for early-z), selection wireframe (24 verts), F3+G chunk-border grid (one 600-vert draw, one uniform write), water (far→near blended), the shared billboard stream (particles + modeled entity box rigs + 3D item cuboids, depth-tested, alpha-blended), clouds (Fast = opaque plane / Fancy = blend) | every frame. Menu/panorama mode replaces all of this with ONE fullscreen cubemap draw (no depth attachment — the panorama pipeline must not run in a depth-carrying pass) |
| 2/3 | `bright`, `blur-h`, `blur-v` | ¼ and ⅛ pyramids | the graphics=Fabulous bloom chain | only when `post.mode > 0`; the mode is permanently 0 (the post pass is vanilla-only) so these three are dormant |
| 3.5 | `fsr-easu` | `up` (full surface res) | FSR 1.0 EASU edge-adaptive upsample — mathematically identity at 1:1 scale, so it runs at every upscale setting | every frame |
| 4 | `post` | surface (or the LINEAR pack handoff when a shader pack is active) | composite: RCAS sharpening (lobe 0.6, only when upscale > 0), menu blur, grade/vignette; the surface write does the single sRGB encode of the frame | every frame |
| 4.5 | `v2-pack-pass` ×N or `shader-pack` | scratch/handoff ping-pong → surface | the external pack's composite chain (colortex0 = chained scene, colortex1 = engine bloom), last pass writes the surface | only with an active shader pack |
| 5 | `gui-quads` | surface (Load) | GUI chrome quads | when enabled + quads non-empty; a failure drops chrome for one frame (logged, never fatal) |
| 6 | `ui` | surface (Load) | the UI canvas blit: flat icon tiles, splash bitmap, F3 strips, crosshair, CPU-fallback chrome — crisp, unblurred, over the chrome | every frame (one draw_indexed) |
| 7 | `gui-text` | surface (Load) | glyph text quads, composited last so no label can hide under an opaque fill (the z-order bug class is closed structurally) | when enabled + text non-empty |

Glyph-atlas upkeep (`sync_glyph_atlas`) runs before pass 5 regardless of the
chrome list — a text-only screen still needs its glyphs uploaded before
pass 7. Screenshot readback (`capture_frame_if_requested`) runs after the
submit, before `present()` invalidates the swapchain texture.

## Draw submission (the Phase-9 work)

* **Region arenas**: chunk meshes are packed into per-region vertex/index
  buffers; chunk origins live in one whole-frame origin vertex buffer bound
  once per terrain/water/shadow list.
* **MDI path** (default on native Vulkan): per-region
  `multi_draw_indexed_indirect` runs against a frame args buffer laid out as
  `[terrain | shadow | water]` segments, written once per frame. One bind +
  one indirect call per region run — `stats.draws` counts region runs, not
  chunks.
* **Zero-rebind loop** (fallback / no-MDI devices): per-chunk indexed draws
  re-binding the arena only when the region changes.
* **Billboard stream** (particles + entities + items): one shared vertex
  buffer, clamped per-frame write. Budget: 4096 particles × 6 verts + 128
  mobs × 360 verts (the 10-box rig worst case) + 16384 margin for item
  cuboids — an over-full scene drops its tail verts instead of panicking the
  device.

## Upstream of the frame

Meshing is CPU (rayon) by default (`gpu_meshing: false` on native) with a GPU
mesher available (`gmesh=1` when the device supports it). Meshes upload into
the region arenas; worldgen/lighting stream ahead of the camera. All of it is
decoupled from the menus — the title panorama is pre-rendered once at init
and re-drawn by pass 1's panorama branch, so a stalled mesher cannot stall
the menus.

## Where the time goes on weak hardware (measured 2026-10-06)

The reference hardware bench (link above) shows the draw phase ≈ the whole frame
(120.9 ms of 120.9 ms avg at rd=12), dominated by CPU-side command
building/upload submission on 2 slow cores; FSR scaling helps only ~12%
because fill rate is not the bottleneck, and rd=4 saves ~35%. Per-pass GPU
timestamp queries are the natural next diagnostic (all 12 passes currently
pass `timestamp_writes: None`), but wgpu timestamps need the
TIMESTAMP_QUERY feature and a working query-period resolve — not guaranteed
on the the other ICD target — so they are recorded as follow-up work, not added in
Phase 1A.
