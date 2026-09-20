# Packs — how to use shader packs & resource packs

This engine ships with **zero** third-party assets and **zero** built-in
shaders. Everything visual that is not code-synthesized comes from packs
**you** drop into the game folder. This document is the complete operator
manual for the compiled binary (Linux/Windows/macOS) — where files go,
what formats are accepted, and what the engine does with them.

---

## 1. The game folder (where everything lives)

Run the binary from its own folder (e.g. `cd` into the release dir and
`./voxelcraft`). On first boot the engine materializes its working set
**relative to the working directory**:

```text
voxelcraft            ← the compiled binary
├── builtin-pack/     ← the embedded resource pack, EXTRACTED on first
│                        run. The engine prefers this folder over the
│                        embedded copy — edit/add files here to re-skin
│                        the game without rebuilding.
├── resourcepacks/    ← YOUR resource packs (folders or .zip)
├── shader-packs/     ← YOUR shader packs (folders or .zip)
├── saves/            ← worlds
├── logs/
│   └── latest.log    ← every boot line — pack reports land here
└── options.txt       ← settings (persisted on every change)
```

> If you launch the binary from a different working directory (e.g.
> `~/release/voxelcraft` from `~`), the folders are created in **that**
> directory. Always launch from the folder you want the game folder in.

---

## 2. Resource packs (`resourcepacks/`)

**Accepted formats** — a pack is any folder or `.zip` in
`resourcepacks/` that carries a manifest:

| Manifest | Who uses it |
|---|---|
| `pack.json` (`{"pack":{"format":1,...}}`) | **Our format** (what the builtin packs carry) |
| `pack.mcmeta` (`{"pack":{"pack_format":N,...}}`) | The wider ecosystem's format — also accepted |

**Texture layout** — both layouts resolve identically (the loader strips
the namespace layer):

```text
pack.json
textures/block/stone.png            ← our flat layout
OR
assets/<any-namespace>/textures/block/stone.png   ← namespaced layout
```

Animation sidecars: `stone.png.json` (ours) or `stone.png.mcmeta`
(ecosystem) — either is read.

**labPBR materials (NAPP-style)** — put `stone_n.png` (normal map) and
`stone_s.png` (specular map) next to `stone.png`. Enable
`Options → Video Settings → Shaders… → LABPBR MATERIALS`. The Shaders
screen's row reports how many atlas tiles gained material maps.

**Enabling packs** — `Options → Resource Packs…`, click a pack in
*Available* to move it to *Selected* (top = highest priority), press
*Done*. The atlas recompiles and the world re-meshes immediately.

**Editing the builtin look without a pack** — edit files in
`builtin-pack/` directly; restart. That folder always wins over the
embedded copy.

---

## 3. Shader packs (`shader-packs/`)

**Accepted format** — Iris/OptiFine-style structure, as a folder or a
`.zip`:

```text
MyShaderPack/            (or MyShaderPack.zip)
├── shaders.properties   (optional — options surface)
├── pack.properties      (optional — display name)
└── shaders/
    ├── composite.fsh    (post pass, reads colortex0)
    ├── composite1.fsh
    ├── …
    └── final.fsh       (last pass → screen)
```

**What the engine does with it** (the 2026-09-21 v2 pipeline):

1. `shaders.properties` + the program list + the option surface are
   parsed (structure report).
2. Every `composite*.fsh` / `final.fsh` is translated **GLSL → WGSL**
   in-engine (naga): `#include` inlined, option `#define`s applied,
   legacy syntax rewritten (`varying`/`gl_FragColor`/`texture2D`),
   bindings remapped (`colortex0..15`, noise, one std140 uniform block).
3. The translated chain **actually runs** on the post pipeline:
   `colortex0` = the chained scene color (ping-pong), `colortex1` = the
   engine's bloom buffer, the noise texture at the OptiFine binding.
   The last pass writes the screen (sRGB-encoded).
4. Per-frame uniforms are filled from engine state: `viewWidth`,
   `viewHeight`, `aspect`, `frameTimeCounter`, `frameCounter`. Unknown
   uniforms stay zero-filled — the pack report lists every one.

**The honest subset** (never silently faked):

* Passes that write multiple render targets (`DRAWBUFFERS:1+`, MRT) are
  skipped — the chain writes `colortex0` only.
* Passes that sample `colortex2+` are skipped (the engine provides
  `colortex0` + `colortex1` only).
* gbuffers/shadow-pass programs are not re-rendered — no shadow map,
  no per-material geometry shaders.
* Every skip carries a reason. Read them in `logs/latest.log`:

```text
shader pack BSL-v10: tier IRIS-COMPOSITE-SUBSET — 4 passes translated, chain installed
  composite: installed (composite)
  composite1: installed (composite)
  composite2: skipped — samples colortex3 (outside the runnable subset)
  final: installed (final→surface)
```

**Selecting a pack** — `Options → Video Settings → Shaders…` (the row
re-scans the folder, so newly dropped packs appear without a restart),
click a pack row. The build+install happens immediately — the report
lands in the boot log, the visuals change the same frame. `(none)`
restores the vanilla post pipeline.

**Selection persists** in `options.txt` (`shaderPack`); the pack is
rebuilt on the next boot.

**Licensing note** — the engine never ships a shader pack. BSL, SEUS
and friends are third-party downloads with their own licenses; you
drop them in, the engine parses + translates them at runtime. That is
the same legal posture as any mod loader (LEGAL-COMPLIANCE.md §4.4).

---

## 4. Troubleshooting

| Symptom | Where to look |
|---|---|
| Pack does not appear in the list | No manifest (`pack.json` / `pack.mcmeta` / `shaders.properties`) at the pack root — check the exact filenames |
| Pack selected but looks vanilla | `logs/latest.log` — the per-pass report; zero runnable passes leaves the vanilla post pipeline |
| Textures unchanged after pack edit | `builtin-pack/` wins over `resourcepacks/` for the same key — remove the override or edit the builtin |
| Wrong folders appear | You launched from a different working directory — see §1 |
