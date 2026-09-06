# VoxelCraft-Rust → Minecraft Replication: Research Dossier
*Compiled from repo analysis (v0.3.0 / main, uploaded zip) + web research. Status: research-only, no code changes made yet. Not the master prompt — reference material for building it later.*

---

## 1. Legal framework (governs every phase below)

- **Tetris Holding v. Xio Interactive (2012)** — game mechanics/rules are not copyrightable; specific expressive assets (exact textures, models, sounds, UI art) are. Xio lost because "look and feel" as a whole was too similar, not because mechanics were copied.
- **Spry Fox v. Lolapps (2012)** — affirmed Tetris; UI elements specifically were named as evidence of infringement in that case. Implication: functional UI *layout* (hotbar slot count, grid dimensions, debug-screen categories) is safe; pixel-identical UI *art* (exact bevel style, exact icon art) is the risky part.
- **Mojang Usage Guidelines** — only relevant if actual Mojang assets/branding are touched (they aren't here). Never call the project "Minecraft," never imply affiliation.
- **Interoperability reverse-engineering doctrine** (Sega v. Accolade, Sony v. Connectix, EU Software Directive Art. 6) — implementing a published *interface/protocol* (e.g. Iris's documented shader pack spec) for compatibility is legitimate; copying the reference implementation's actual code is not.
- **Rule of thumb applied throughout:** mechanical/data files (JSON schemas, recipe data, block registries, uniform names, protocol specs) = safe to replicate exactly. Expressive assets (textures, 3D models, audio, exact UI art, lang strings wholesale) = must be independently authored, never extracted from Mojang's files.

### License notes on reference projects
| Project | License | Use as |
|---|---|---|
| Veloren (Rust/wgpu voxel RPG) | GPL-3.0 | Architectural ideas only, never copy code |
| Cuberite (C++ clean-room MC server) | Apache-2.0 | Safe to port logic/values from, attribution only |
| Sodium | LGPL-3.0 | Technique reference only (multi-draw, chunk-graph culling) |
| Phosphor | GPL/LGPL-3.0 | Technique reference only (deferred/batched lighting) |
| Lithium | LGPL-3.0 | Principle reference only (correctness-preserving speedups) |
| Iris | LGPL-3.0 | Study the *published spec*, not the Java code |
| Minecraft-Font / Monocraft | OFL-1.1 (font) / GPL-3 (generator) | Font file itself is safe to bundle |
| AMD FidelityFX FSR 1.0 | **MIT** | Fully safe, already faithfully ported in repo |
| LabPBR Material Standard | Open community spec (shaderLABS) | Fully safe, not Mojang IP |
| Datapack format (JSON/mcfunction) | Mojang-designed, implementation-agnostic by design | Safe — used identically across Vanilla/Spigot/Paper/Fabric/Forge |

---

## 2. Repo architecture ground truth (verified by reading the actual source, not the README)

- **Workspace:** `voxelcraft/` Cargo workspace, 15 crates, **30,320 total lines of Rust**.
- Per-crate line counts: vc-render 8,600 · voxelcraft(bin) 6,280 · vc-world 3,430 · vc-anvil 1,643 · vc-gameplay 1,744 · vc-mesh 1,510 · vc-sim 1,464 · vc-pack 1,407 · vc-blocks 1,360 · vc-audio 1,021 · vc-chunk 586 · vc-nbt 558 · vc-particles 382 · vc-inventory 245 · vc-rng 52.
- **External deps (only ~15, all low-level):** wgpu, winit, glam, rayon, serde/serde_json, bytemuck, image, naga, flate2, rodio, pollster, wasm-bindgen stack. **No egui despite the repo's own tagline claiming it** — UI is a fully hand-built immediate-mode system (5×7 bitmap font baked to texture).
- **Code hygiene:** zero `todo!()`/`unimplemented!()`/stub markers anywhere. 182 `#[test]` functions. Zero `unsafe` blocks. 228 `.unwrap()` + 25 `.expect()` (~1 per 118 lines — not alarming but worth hardening).
- **Concurrency model:** Rayon thread pool + `Arc`-based COW chunk snapshots. **Zero `Mutex`/`RwLock` anywhere** — deliberate lock-free design.

### What's already implemented (confirmed by reading code, better than the README suggests)
- **Nether dimension** — `Dimension::Nether`, 8:1 coordinate scaling, no-skylight rendering, portal search matching vanilla's algorithm. Not mentioned in README at all.
- **Villagers** — real `Villager` struct, position/velocity/yaw, wander with jump-steps at ~0.5 blocks/s, 6 professions with trade tables. Code self-documents: *"pathfinding is straight-line steering + step jumps (no full A*)"*.
- **Redstone/fluids/gravity** — real tick functions in `vc-sim` (`wire_tick`, `torch_tick`, `water_tick`, `gravity_tick`).
- **Save system** — real 1.16.5 Anvil format: `./saves/VoxelCraft/region/r.X.Z.mca` + `level.dat`. Matches vanilla's exact save layout.
- **Title screen** — real animated panorama background (13-tap blur disc), splash text, proper widget layout.
- **Options menu** — FOV, brightness, sensitivity, volume, render distance, shaders on/off, graphics fancy/fast, shadows, smooth lighting, upscaling, clouds, max FPS/VSync, music slider, pause menu. Structurally close to vanilla already.
- **FSR 1.0** — genuinely faithful WGSL port of AMD's real algorithm (EASU + RCAS), comments reference actual AMD source file names. MIT-licensed, cross-vendor by design. **This is NOT a weak part of the engine.**
- **Shader-pack system** — custom WGSL format, runtime-validated via `naga`, scans a `shader-packs/` folder at runtime (native only).
- **Resource-pack pipeline** — `vc-pack` parses real 1.16.5-format blockstate/model JSON (`builtin-pack/`), but **no runtime `resourcepacks/` folder scanning** exists — it's compiled-in only.

### Confirmed gaps (from reading code, not guessing)
| System | Status |
|---|---|
| Mobs (hostile or passive) | **Zero** — 0 hits for zombie/skeleton/creeper/cow/pig/sheep/wolf/animal |
| Combat | Token only — `combat`: 1 hit, `attack`: 2 hits, no real system |
| Multiplayer/networking | **Zero** — 0 hits for socket/tcp/udp |
| Game modes (Survival/Creative/Hardcore) | **Not implemented as a system** — flying always available, items still deplete, save hardcodes `GameType: 1` |
| Redstone components | Only wire/torch/lever. Code *itself* documents: repeaters/comparators/pistons/dispensers/observers/hoppers "not in the registry" |
| Structures | Villages (house + well) only — no dungeons/strongholds/mineshafts/temples/ravines |
| Item entity persistence | Dropped items don't survive save/reload — no serialization in `vc-anvil/save.rs` |
| Biomes | 8 implemented (Plains/Forest/Desert/Snowy/Ocean/Beach/Mountains/NetherWastes) vs vanilla's 60+ |
| Enchantments | 12 defined vs vanilla's ~40 |
| World border | Not implemented |
| Achievements/statistics | Not implemented |
| Occlusion culling | **Not implemented** — every render pass explicitly sets `occlusion_query_set: None` |
| LOD (distant chunks) | Not implemented |
| Simulation distance (separate from render distance) | Not implemented — only one distance setting exists |
| Mipmapping | **Zero mentions anywhere in code** |
| Anisotropic filtering | **Not implemented** (only unrelated math hits inside FSR algorithm) |
| Antialiasing/MSAA | Off everywhere — `MultisampleState::default()` used throughout |
| Settings persistence (native) | `save_settings()`/`load_settings()` exist **web-only**; native likely resets every launch |
| Screenshots folder / F2 | Not implemented |
| Crash-reports / logs folder | Not implemented |
| Rebindable keybinds | Not implemented — controls appear fixed |
| GPU compute (meshing/world-gen) | **Zero compute shaders anywhere** — all CPU/Rayon |

---

## 3. Settings/config gap — exact, measured against real captured files

VoxelCraft's `Settings` struct: **14 fields total** (`render_distance, sensitivity, volume, fov, brightness, smooth_lighting, clouds, graphics, shader, shadow_quality, upscale, music_volume, maxfps`).

Real Minecraft + OptiFine + Iris across the three attached files: **~180 distinct settings.**

### Real vanilla `options.txt` reveals (partial list, full file preserved in chat upload)
`simulationDistance:32` (separate from `renderDistance:9` — confirms the 26.x simulation/render split), `guiScale:2`, `mipmapLevels:2`, `ao:false`, `biomeBlendRadius:1`, `entityShadows`, `narrator`, `highContrast`, `damageTiltStrength`, `darknessEffectScale`, `screenEffectScale`, `fovEffectScale`, `glintSpeed/Strength`, `~30 rebindable key_* keybinds`, `10 separate soundCategory_* sliders` (master/music/record/weather/block/hostile/neutral/player/ambient/voice), `7 modelPart_* skin-layer toggles`, full chat subsystem settings (colors/links/opacity/scale/width/height/delay), `prioritizeChunkUpdates`, `pauseOnLostFocus`, `rawMouseInput`.

### Real `optionsof.txt` (OptiFine) reveals
`ofOcclusionFancy` (direct proof of the occlusion-culling gap), `ofMipmapType`, `ofAaLevel`, `ofAfLevel:4`, individually toggleable animations (`ofAnimatedFire/Portal/Redstone/Explosion/Flame/Smoke/Water/Lava/Terrain/Textures`), `ofBetterGrass`, `ofConnectedTextures`, `ofBetterSnow`, `ofNaturalTextures`, `ofRandomEntities`, `ofDynamicLights`, `ofDynamicFov`, `ofCustomSky/Colors/Items/Fonts/Guis/EntityModels`, `ofLazyChunkLoading`, `ofRenderRegions`, `ofSmartAnimations`, `ofChunkUpdates(Dynamic)`, `ofFastRender`, `ofShowCapes`.

### Real `optionsshaders.txt` (Iris) reveals
`shaderPack` (active pack selector), **per-texture-type filtering** — `TexMinFilB/N/S`, `TexMagFilB/N/S` (separate Block/Normal/Specular map filter settings — proves LabPBR material support), `specularMapEnabled`, `normalMapEnabled`, `renderResMul` (continuous scale, vs. VoxelCraft's fixed 75%/50% presets), `shadowResMul`, `shadowClipFrustrum`, `antialiasingLevel`, `cloudShadow`, `oldLighting`/`oldHandLight` (back-compat toggles), `handDepthMul`, `tweakBlockDamage`.

---

## 4. Rendering/GPU-architecture findings

- **wgpu already fully solves the "Vulkan/OpenGL/Windows/macOS don't overlap" concern** — exactly one backend selected at runtime per platform (Vulkan/DX12 on Windows, Vulkan on Linux, Metal on macOS). No architectural risk here, nothing to build.
- **Mojang itself is now moving to Vulkan** — Minecraft changed to year-based versioning in 2026 (26.1, 26.2...). Current release: **26.2 "Chaos Cubed," shipped June 16, 2026**, which introduces an **experimental Vulkan backend**, moving away from OpenGL. Validates VoxelCraft's original architecture choice.
- **GPU compute for meshing/world-gen**: no mainstream open-source engine does this fully yet — genuinely bleeding-edge. `cgerikj/binary-greedy-meshing` is a legally-clean technique reference (algorithm writeup, not code to copy) for a future WGSL compute-shader port.
- **Iris/GLSL shader-pack compatibility layer**: real, buildable, but large. `naga`'s GLSL frontend only supports "GLSL 440+, Vulkan semantics" — real shader packs use GLSL 330 compatibility profile with implicit OpenGL-style uniforms. Requires a custom GLSL-330-compat parser feeding into your own IR → WGSL, plus restructuring the render loop to match Iris's **gbuffers (opaque) → deferred → gbuffers (translucent) → composite → final** pass chain, including special comment directives (`/* RENDERTARGETS: n */`). Iris's own public uniform reference (shaders.properties) is the legitimate spec to build against. Even official Iris doesn't hit 100% pack compatibility — target "high compatibility with popular packs," not literally every pack.
- **LabPBR** (see section on PBR above) is the target for actual material-based shading (normal/specular maps), separate from the GLSL-parsing problem.
- **User's proposal, agreed as sound**: build Iris compatibility as a **separate sister project**, referenced/imported by VoxelCraft — to be written into the master prompt as its own linked sub-project when that's drafted.

---

## 5. Mods — real answer is datapacks, not Forge/Fabric

- Real code mods (Forge/Fabric) are Java bytecode hooking into Mojang's compiled game — categorically impossible to run in a Rust/wgpu engine.
- **Mojang's own datapack system** is the legitimate answer: pure JSON + `.mcfunction` text, zero compiled code, explicitly implementation-agnostic (identical format across Vanilla/Spigot/Paper/Fabric/Forge). Covers recipes, loot tables, advancements, tags, structures (NBT), world-gen configs. Version-pinned via `pack_format` numbers.
- This gets real "bring your own content" compatibility with a large existing ecosystem, legally cleanly, without ever touching the Forge/Fabric impossibility.

---

## 6. Optimization techniques mapped to specific VoxelCraft gaps

| Technique (source) | Addresses | License / reference status |
|---|---|---|
| Simulation distance ≠ render distance (Mojang 1.18+/26.x) | No separate tick radius currently exists | N/A — official Mojang feature, mechanic |
| Chunk-graph occlusion culling (Sodium's technique / classic portal-culling) | `occlusion_query_set: None` everywhere | LGPL-3.0 — technique only |
| Multi-draw/indirect chunk rendering (Sodium) | Per-chunk draw call overhead; direct lever for "more GPU-bound" goal | LGPL-3.0 — technique only; `wgpu` supports `multi_draw_indirect` natively |
| Deferred/batched lighting updates (Phosphor) | Eager BFS skylight recompute on every edit | GPL/LGPL-3.0 — technique only |
| Correctness-preserving tick-loop speedups (Lithium) | General `vc-sim` performance philosophy | LGPL-3.0 — principle only |
| Async path-traced entity/block-entity culling (EntityCulling) | Future mob rendering, once mobs exist | Custom/"other" license — technique reference only |
| Batched immediate-mode draw calls (ImmediatelyFast) | **Directly relevant** — VoxelCraft's UI is hand-built immediate-mode | Reference technique |
| Background FPS cap when unfocused (Dynamic FPS) | Not implemented, cheap win | Reference technique |
| Parallel chunk gen/IO (C2ME) | Already partially covered by Rayon; check if region-file I/O itself is still synchronous | Reference technique |
| **Not applicable to this engine** | Indium, Fabric API, ModMenu (Java/Fabric-ecosystem glue — no equivalent mod ecosystem exists here); FerriteCore, LazyDFU, ModernFix (JVM-specific memory/schema-migration problems Rust doesn't have) | — |

---

## 7. Files still worth gathering (from user's own legally-owned MC install)

**Safe to use as exact reference data (mechanical/schema):**
- Vanilla server `--reports` generated data (`generated/reports/blocks.json`, `registries.json`, `commands.json`) — official Mojang-sanctioned tool-developer export.
- Vanilla data pack JSON (`data/minecraft/...` inside the jar) — recipes, loot tables, tags, advancements.
- `sounds.json` — sound *event name*/category schema only, not audio files.
- More real `options.txt`/`optionsof.txt`/`optionsshaders.txt` variants if settings differ from defaults (e.g. custom keybinds).

**Look-but-don't-lift (reference only, never extract wholesale):**
- `assets/minecraft/textures/`, `models/`, `.ogg` sound files — copyrighted expression, redraw/re-record independently only.
- `lang/en_us.json` — spot-check only, never bulk-import.

---

## 8. Open decisions still needed before the master prompt is written

1. **VoxelCraft's own license** — README still has "Add license here." Blocks knowing what's safely portable from Apache-2.0 sources (Cuberite) vs. what stays inspiration-only.
2. **Phase priority order** — not yet decided between: game modes, mobs+combat, GPU-architecture migration, rendering/options-menu overhaul, Iris-compat sister project, datapack support.
3. **Iris compatibility layer** — confirmed as a separate linked sister project (both owned by user), to be referenced/instructed for in the master prompt, not built inside the main repo.
4. **Numeric ground truth** — for exact values (block hardness, mob stats, drop rates), pull from official generated reports / Minecraft Wiki at prompt-writing time, not from AI memory.

---

## 9. Honest status assessment (as of this dossier)

Rough weighted completeness toward "a real Minecraft replica," per earlier analysis, still holds at approximately **25–35%**. Strongest areas: rendering foundation, save-format compatibility, title/options UI structure. Weakest areas: mobs (0%), combat (0%), game modes (0%), redstone completeness (~3 of ~9 components), biome/structure variety. This is a systems-and-mechanics map, not exact numeric parity — the latter still requires the official generated-reports/wiki pass noted above.
