# VoxelCraft-Rust — Minecraft Java Edition 1.16.5 Parity & High-Performance Master Engineering Specification

> **Purpose:** This is the master specification for an autonomous coding/development AI working on `CodeAbhi826/VoxelCraft-Rust`.
>
> **Primary goal:** Transform VoxelCraft-Rust into an independent, clean-room-style implementation that approaches the **observable gameplay, data, rendering, audio, UI, and world behavior of Minecraft Java Edition 1.16.5**, while using a modern Rust/wgpu architecture designed for strong CPU/GPU scalability.
>
> **Important:** This document is an engineering specification, not a claim that Mojang's internal implementation must be reproduced. Reproduce externally observable behavior and compatible formats where feasible, while implementing the engine independently.

---

## 0. Non-Negotiable Operating Rules

### 0.1 Inspect before modifying

Before changing any subsystem:

1. Inspect the current repository and current branch/commit.
2. Inspect the actual source files involved, not just the README.
3. Identify existing behavior, tests, benchmarks, assets, APIs, and dependencies.
4. Preserve working functionality unless the change has a demonstrated reason to replace it.
5. After every substantial change, compile/test and record regressions.
6. Never assume the repository matches an older analysis; re-audit the current state.

### 0.2 Use evidence levels

Every major architectural decision must be categorized as one of:

- **VERIFIED:** directly supported by an authoritative source or reproducible experiment.
- **STRONGLY SUPPORTED:** supported by multiple high-quality sources or clear engine evidence.
- **ENGINEERING RECOMMENDATION:** a design choice made for VoxelCraft-Rust, not a Minecraft requirement.
- **EXPERIMENTAL:** useful to test, but must not be treated as mandatory until benchmarks validate it.
- **UNKNOWN:** behavior not sufficiently established; create a measurement/differential test instead of inventing an answer.

Do not convert an engineering recommendation into a supposed Minecraft fact.

### 0.3 Performance rules

Do not optimize by ideology.

Never assume that any of the following is automatically faster:

- deferred rendering,
- Forward+,
- MDI/indirect rendering,
- lock-free atomics,
- `AtomicPtr`,
- 8-byte vertices,
- texture atlases,
- texture arrays,
- ECS,
- more threads,
- more batching,
- more GPU work.

When practical, benchmark competing implementations on representative scenes.

### 0.4 Compatibility rules

The target version is **Minecraft Java Edition 1.16.5** unless a feature is explicitly labelled as an optional modern enhancement.

Do not silently copy behavior from 1.17+ or newer Minecraft versions.

Version-specific data formats, biome storage, renderer assumptions, and resource semantics must be verified for 1.16.5.

---

# 1. Project Goal and Scope

## 1.1 Vanilla parity target

The core target is:

> **Minecraft Java Edition 1.16.5-equivalent observable behavior and presentation.**

The project should aim for increasingly strong parity in these layers:

1. World and block data
2. Block states and models
3. Terrain generation
4. Lighting
5. Fluids
6. Redstone and scheduled updates
7. Entities and AI
8. Player physics
9. Items, inventories, crafting, smelting, brewing, enchanting and combat
10. Dimensions and structures
11. Save/load and compatible data formats where feasible
12. Resource packs and textures
13. Animation and particles
14. Sounds and audio categories
15. HUD, menus, F3/debug information and controls
16. Rendering and visual effects
17. Networking/multiplayer compatibility where feasible

A subsystem can be marked:

- `PARITY-CORE`
- `PARITY-HIGH`
- `PARITY-MEDIUM`
- `OPTIONAL-ENHANCEMENT`

---

# 2. Clean-Room / Legal Boundary

The implementation must remain independent.

### Allowed engineering sources

- Publicly documented file/data formats
- Public protocol documentation
- Public resource-pack specifications
- Open-source software under compatible licenses
- Open-source optimization techniques and architectural patterns
- Black-box observation of a legally obtained local reference installation
- Differential testing based on observed inputs/outputs

### Do not

- copy Mojang source code,
- distribute Mojang binaries,
- redistribute Mojang textures, sounds, fonts, music, or other proprietary assets,
- copy decompiled implementation code into the repository,
- paste proprietary code into generated source.

The engine may support **user-supplied resource packs** and other user-owned/legally obtained assets.

Where legal boundaries are uncertain, mark the issue for professional legal review rather than asserting legal certainty.

---

# 3. Reference Architecture

Use this as the desired direction, not as a requirement to imitate one particular engine.

```text
                    ┌───────────────────────────┐
                    │        Input / UI         │
                    └────────────┬──────────────┘
                                 │
                                 ▼
                    ┌───────────────────────────┐
                    │  Deterministic Simulation │
                    │  ticks / block updates    │
                    │  entities / redstone      │
                    └───────┬─────────────┬─────┘
                            │             │
                   immutable/safe      jobs/events
                     world views          │
                            │             ▼
                            │    ┌─────────────────────┐
                            │    │ Worker Pool         │
                            │    │ chunk gen            │
                            │    │ lighting             │
                            │    │ meshing              │
                            │    │ pathfinding          │
                            │    │ asset preparation    │
                            │    └─────────┬───────────┘
                            │              │
                            └──────┬───────┘
                                   ▼
                         ┌─────────────────────┐
                         │ Rendering Frontend  │
                         │ culling / sorting   │
                         │ command preparation │
                         └─────────┬───────────┘
                                   ▼
                         ┌─────────────────────┐
                         │ wgpu Backend        │
                         │ Vulkan / DX12 /     │
                         │ Metal / WebGPU      │
                         └─────────┬───────────┘
                                   ▼
                                  GPU
```

Simulation and rendering should be decoupled enough that rendering can run at 60/120/240+ Hz even when world simulation uses a different update cadence.

---

# 4. Current Repository Audit Requirement

The current repository has historically included:

- Rust + `wgpu`
- native + WASM/WebGPU support
- multithreaded chunk generation/meshing
- greedy meshing
- copy-on-write chunk editing
- procedural assets
- frustum culling
- basic lighting and water
- an expanding block registry and modern rendering features

However, **the live repository is authoritative**.

Before each phase, re-read the actual current code and current commits.

Do not rely on an old “18 blocks” description if the repository has moved beyond it.

---

# 5. Block Registry and BlockState System

## 5.1 Required design

Replace simplistic block IDs with a compact data-driven representation containing:

- block type ID,
- runtime state ID,
- property values,
- collision shape,
- render/model reference,
- transparency/render class,
- light emission,
- light opacity,
- hardness and interaction metadata,
- sound group,
- tint class,
- fluid behavior when applicable.

The system must support state properties such as:

- facing,
- axis,
- half,
- shape,
- powered,
- lit,
- open,
- waterlogged,
- age,
- level,
- attachment,
- rotation,
- etc.

Do not hard-code a finite list of properties into every block object. Use compact property definitions plus interned/packed state representations.

## 5.2 Blockstate/model data

Implement loaders for the relevant 1.16.5 resource-pack JSON structures:

- `assets/minecraft/blockstates/*.json`
- `assets/minecraft/models/block/*.json`
- model parents
- texture references
- elements
- face UVs
- rotations
- tint indexes
- cullface rules
- multipart models

Resolve model inheritance during asset loading so that the renderer/mesher uses a compact precompiled representation.

### Requirement

Do not parse JSON during every block render/mesh operation.

Parse once, validate, canonicalize and cache.

---

# 6. Chunk and Section Storage

## 6.1 Section structure

Use 16×16×16 sections as the internal organization unless a benchmark proves a different structure is substantially better.

Use a palette-based representation for block states where it materially reduces memory.

Conceptually:

```rust
pub struct ChunkSection {
    pub blocks: PalettedContainer,
    pub block_light: LightContainer,
    pub sky_light: LightContainer,
    pub non_air_count: u16,
    // version-specific auxiliary data as required
}
```

## 6.2 Paletted container requirements

Support a compact representation for:

- single-value sections,
- small local palettes,
- larger palettes,
- a global-ID fallback.

Do not assume only `4 -> 8 -> 16` bits are sufficient for every representation.

Use the version-appropriate palette semantics for the target format, while allowing a separate optimized runtime representation internally.

## 6.3 Correctness requirements

Test:

- bit reads/writes,
- values spanning 64-bit boundaries,
- palette expansion,
- palette compaction if implemented,
- section cloning/snapshotting,
- serialization/deserialization,
- random stress access.

Use property-based tests for bit-packing where possible.

---

# 7. World Grid and Concurrency

## 7.1 Do not blindly use AtomicPtr

A lock-free pointer grid is **experimental**, not a requirement.

Prefer the simplest high-performance safe structure that profiles well.

Candidates may include:

- dense spatial arrays around the player,
- sharded hash maps,
- chunk-coordinate slabs,
- read-optimized maps,
- generation-indexed grids,
- region structures,
- RCU/copy-on-write style snapshots,
- lock-free structures only where justified.

### Mandatory rule

No `unsafe` pointer-based world structure should be introduced merely because it sounds like Sodium.

If unsafe memory reclamation is required, document:

- ownership,
- lifetime,
- reclamation,
- ABA handling,
- thread safety,
- failure modes,
- benchmark justification.

---

# 8. Chunk Streaming and Job System

Build a real job/dependency model.

Jobs should be able to represent:

```text
request
  ↓
terrain generation
  ↓
decoration / structures
  ↓
lighting
  ↓
neighbor availability
  ↓
mesh compilation
  ↓
GPU upload
  ↓
visibility
```

Use priorities based on:

1. camera proximity,
2. visibility,
3. player movement direction,
4. simulation necessity,
5. background prefetch.

Cancel obsolete work where safe.

Avoid saturating all CPU cores with background jobs when doing so harms the main simulation or frame pacing.

---

# 9. Deterministic Simulation

This is a major priority.

The engine should distinguish:

### Parallelizable work

- terrain generation
- chunk preprocessing
- lighting propagation
- mesh generation
- asset processing
- pathfinding where results do not depend on ordering
- broad-phase calculations
- some AI computations

### Deterministic/ordered work

- block update ordering where observable
- redstone update semantics
- scheduled ticks
- entity interaction ordering
- inventory transactions
- block placement/breaking
- physics interactions whose ordering changes results

Use job barriers/dependency graphs when parallel work feeds deterministic simulation.

The target is:

> **Parallel where independent, deterministic where order is observable.**

Never trade correctness for thread count.

---

# 10. Lighting System

Implement both:

- skylight,
- block light.

Requirements:

- attenuation,
- occlusion,
- cross-section propagation,
- chunk boundary propagation,
- light removal,
- re-lighting after block changes,
- incremental updates,
- save/load compatibility where applicable.

Starlight, Phosphor and similar projects may be used as **architectural references**, not as a requirement to reproduce their exact code or behavior.

The target is Minecraft 1.16.5-visible lighting behavior first.

Benchmark:

- initial world lighting,
- light updates,
- large light-source edits,
- mass block removal,
- chunk loading,
- cross-chunk propagation.

---

# 11. Meshing

## 11.1 Core requirements

Support:

- opaque cubes,
- cutout geometry,
- translucent geometry,
- non-cube JSON models,
- model rotations,
- blockstate-driven variants,
- face culling,
- AO,
- biome tints,
- block light,
- sky light,
- animated textures/material flags.

## 11.2 Greedy meshing

Retain and improve greedy meshing where it performs well.

However, do not assume all models can be represented by a cube-only greedy mesher.

Use a hybrid path:

```text
cube-friendly block
    → fast specialized mesher

complex JSON model
    → generalized model compiler/mesher
```

Precompile complex models when possible.

---

# 12. Mesh Invalidation

Prefer fine-grained invalidation.

A block edit should normally dirty:

- its own section,
- neighboring sections/chunks when a boundary face/model/light interaction requires it,
- relevant translucent geometry,
- relevant lighting regions.

Do not re-mesh an entire 16×256×16 chunk for a localized edit unless required.

Track dirty causes separately:

- geometry,
- lighting,
- material,
- transparency,
- visibility.

---

# 13. Vertex and Index Representation

An 8-byte vertex is **not mandatory**.

The required optimization goal is:

> Minimize vertex bandwidth and CPU/GPU memory traffic while preserving all required 1.16.5 rendering information.

Benchmark candidates such as:

- 8 bytes,
- 12 bytes,
- 16 bytes,
- packed 32-bit attributes,
- separate per-draw/per-section data,
- indexed vs non-indexed paths,
- instanced metadata.

Choose the smallest format that is actually advantageous on representative GPUs.

Never remove required information just to hit “8 bytes.”

A packed format should support, as needed:

- position/local geometry,
- model/face orientation,
- UV or texture reference,
- AO,
- sky light,
- block light,
- tint/material flags,
- section/draw metadata.

---

# 14. GPU Mesh Storage and Draw Submission

Benchmark these designs:

1. per-section buffers,
2. per-chunk merged buffers,
3. regional mega-buffers,
4. indirect draw commands,
5. multi-draw indirect where supported,
6. GPU-driven visibility where justified.

Do not assume MDI is automatically best on every backend.

Use capability detection and maintain a fallback path.

For WebGPU/WebGL2 compatibility, verify the exact supported feature set before relying on an advanced indirect feature.

---

# 15. Culling

Implement:

- frustum culling,
- distance culling,
- section/chunk empty checks,
- optional occlusion culling,
- optional horizon/portal-like optimization only if applicable.

Benchmark occlusion culling against its CPU/GPU cost.

A culling system is only an optimization if it saves more work than it costs.

---

# 16. Rendering Architecture

## 16.1 Do NOT mandate deferred rendering

The current preferred direction is a **forward/Forward+ hybrid renderer**, because Minecraft-style geometry is dominated by simple block materials and has substantial transparency/cutout requirements.

However, maintain an architecture that could support other pipelines.

Benchmark at least:

- forward,
- Forward+,
- deferred,
- hybrid variants.

Use the best architecture per platform and quality mode.

## 16.2 Desired pass structure

A possible native path:

```text
1. Shadow pass
2. Opaque/cutout world pass
3. Translucent world pass
4. Particles/entities
5. Post-processing / upscaling
6. HUD/UI
```

The exact ordering must be validated against the target visual behavior.

---

# 17. Shadows

Implement directional sun/moon shadows.

Support:

- depth map rendering,
- PCF,
- configurable quality,
- stable shadow projection,
- bias controls,
- optional cascaded shadow maps.

Do not call a single 2048² map “cascaded shadows.”

If CSM is used, implement actual cascades with split distances and per-cascade matrices.

Benchmark:

- 1024,
- 2048,
- 4096,
- different cascade counts,
- filtering costs.

---

# 18. Lighting and Vanilla Visuals

A Minecraft parity renderer must distinguish:

- raw light level,
- directional sunlight,
- ambient contribution,
- AO,
- biome tint,
- emissive behavior,
- fog,
- sky color,
- dimension-specific visual behavior.

Do not replace Minecraft's lighting model with a generic PBR system and call it parity.

PBR-like rendering can exist as an optional enhancement/shader mode.

Vanilla compatibility comes first.

---

# 19. Resource Pack and Texture Pipeline

Implement a real resource pipeline supporting user-supplied packs.

Required concepts:

- archive/folder loading,
- `pack.mcmeta` validation,
- asset indexing,
- texture loading,
- missing-texture handling,
- texture animation metadata,
- mipmap generation,
- texture filtering,
- biome tint assets,
- model textures,
- item textures,
- GUI textures,
- font resources,
- sounds.

## Atlas vs texture array

Do not assume one is universally superior.

Benchmark:

- packed atlas,
- texture array,
- bindless/resource-indexed approaches where available.

Atlas must account for:

- mip bleed,
- padding,
- animation,
- varying texture dimensions,
- UV precision.

---

# 20. Animated Textures

Support version-appropriate animation metadata and behavior for assets such as:

- water,
- lava,
- fire,
- portals,
- animated UI textures,
- other pack-defined animations.

Precompute animation state where possible.

Avoid rebuilding geometry solely because a texture frame changed unless required.

---

# 21. Audio

Implement a data-driven sound-event system.

Support:

- sound categories,
- multiple sound variants,
- weight,
- pitch variation,
- volume,
- streaming flags,
- attenuation,
- spatial positioning,
- music,
- ambient/cave sounds where applicable.

Keep native and WASM audio backends behind a common interface.

Do not assume a native audio crate has full browser compatibility.

---

# 22. Entity and AI Framework

Design entities as data + behavior components or another efficient structure, but do not require ECS unless benchmarking justifies it.

Support the 1.16.5 entity families progressively:

- passive mobs,
- hostile mobs,
- projectiles,
- item entities,
- XP orbs,
- paintings,
- minecarts,
- boats,
- TNT,
- falling blocks,
- bosses,
- villager systems.

Entity update scheduling must be deterministic enough for reproducible tests.

---

# 23. Player Physics

Reproduce observable 1.16.5 movement behavior.

Test:

- walking,
- sprinting,
- sneaking,
- jumping,
- step height,
- swimming,
- climbing,
- falling,
- water movement,
- lava movement,
- knockback,
- gravity,
- friction,
- slipperiness,
- collisions,
- eye height,
- hitbox changes,
- block interaction distance.

Do not trust remembered constants blindly. Differential-test against the 1.16.5 reference.

---

# 24. Fluids

Implement water and lava as real simulation/state systems.

Support:

- source blocks,
- flowing levels,
- spreading,
- decay,
- interaction with blocks,
- visual geometry,
- lighting interaction,
- sound interaction.

Treat fluid behavior as a version-specific parity problem, not merely a rendering effect.

---

# 25. Redstone

This is a major parity milestone.

Implement and test:

- dust connectivity,
- signal strength,
- repeaters,
- comparators,
- torches,
- observers,
- pistons,
- sticky pistons,
- buttons,
- levers,
- pressure plates,
- tripwires,
- doors/trapdoors,
- dispensers/droppers,
- hoppers,
- rails,
- powered rails,
- detector rails,
- scheduled block updates.

Most importantly:

> Reproduce observable update/tick ordering.

A visually similar redstone system is not sufficient for parity.

---

# 26. World Generation

Implement deterministic 1.16.5-style generation.

Required categories:

- terrain height,
- biomes,
- caves,
- ores,
- structures,
- villages,
- dungeons,
- strongholds,
- mineshafts,
- ravines,
- Nether generation,
- End generation,
- vegetation,
- lakes,
- decorations,
- mob spawning inputs.

Do not invent a “Minecraft-like” generator and call it 1.16.5 parity.

For seed parity, use deterministic differential testing.

---

# 27. Biomes

Use the **actual 1.16.5 representation and semantics** for compatibility.

Do not automatically adopt later Minecraft biome storage formats merely because they are newer.

Internally, the engine may use an optimized representation if:

- external behavior remains correct,
- serialization compatibility is preserved where targeted,
- memory/performance improves.

---

# 28. Save/Load and Data Compatibility

Implement, where feasible:

- world save/load,
- chunk serialization,
- NBT,
- relevant Anvil region handling,
- player data,
- block entities,
- entities,
- level metadata,
- compatible world seeds/settings.

Separate:

`internal runtime format`

from

`external compatibility format`.

Do not force the runtime memory layout to exactly match the disk layout.

---

# 29. Items, Containers and Crafting

Progressively implement:

- inventory,
- hotbar,
- armor,
- item stacks,
- durability,
- crafting,
- smelting,
- blasting,
- smoking,
- campfires,
- brewing,
- enchanting,
- anvils,
- loot,
- containers,
- villagers,
- trading.

Use data-driven recipes and registries.

---

# 30. UI / HUD

Rebuild the visible 1.16.5 experience, including:

- crosshair,
- hotbar,
- health,
- hunger,
- armor,
- air,
- XP bar,
- XP level,
- status effects,
- item names,
- boss bars,
- subtitles,
- chat,
- inventory/container GUIs,
- pause menu,
- options menu,
- controls,
- video settings,
- accessibility where applicable.

Use actual resource-pack assets when the user provides them.

---

# 31. F3 and Performance Telemetry

Provide an enhanced debug overlay containing:

- FPS,
- frame time,
- 1% low,
- 0.1% low,
- average frame time,
- CPU frame time,
- render submission time,
- GPU frame time when supported,
- loaded chunks,
- rendered chunks,
- vertices/indices,
- chunk generation queue,
- mesh queue,
- world coordinates,
- target block,
- biome,
- dimension,
- light levels,
- memory statistics where reliably available.

Do not display fabricated VRAM/GPU utilization.

If a metric cannot be queried reliably on a backend, label it unavailable.

---

# 32. Settings

Use a persistent settings system.

Include at least:

### Performance

- render distance,
- simulation distance,
- max FPS,
- VSync,
- worker count,
- chunk-build budget,
- mesh-upload budget.

### Graphics

- graphics preset,
- mipmap levels,
- biome blending,
- AO,
- clouds,
- particles,
- entity distance,
- shadow quality,
- transparency quality.

### Modern enhancements

- upscaler enable/disable,
- upscaler scale,
- sharpening,
- shader selection,
- shadow enhancements.

Settings must be validated, versioned and migrated when the schema changes.

---

# 33. AMD FSR 1.0

If FSR 1 is implemented, it must be a **real implementation of the published/reference FSR 1 algorithm**, not a simple blur/edge filter renamed “FSR.”

FSR 1 consists of:

1. EASU — Edge-Adaptive Spatial Upsampling
2. RCAS — Robust Contrast Adaptive Sharpening

Use AMD GPUOpen/reference guidance for:

- algorithm constants,
- sampling pattern,
- color-space/integration rules,
- sharpening behavior,
- render target placement,
- resource transitions.

Do not substitute a simplified five-tap or twelve-tap filter and call it production FSR 1.

UI/high-frequency overlays must not be unnecessarily resampled and sharpened.

---

# 34. Shader System

## 34.1 Native VoxelCraft shader framework

Implement a modular shader architecture supporting:

- configurable passes,
- WGSL shaders,
- material definitions,
- post-processing,
- shadow passes,
- custom effects,
- shader recompilation/caching.

## 34.2 Iris/OptiFine compatibility

Treat this as a **separate major compatibility project**.

A generic WGSL shader loader is NOT equivalent to Iris compatibility.

True compatibility requires research into:

- expected shader stages,
- shader naming,
- uniforms,
- macros,
- include handling,
- transformations,
- framebuffer semantics,
- samplers,
- shadow buffers,
- composite passes,
- compatibility quirks,
- version expectations.

Create a compatibility layer only after the native shader architecture is stable.

Label compatibility in tiers:

- `NATIVE-SHADER`
- `SHADER-PACK-API`
- `OPTIFINE-COMPATIBLE-SUBSET`
- `HIGH-COMPATIBILITY`
- `FULL-COMPATIBILITY` only when actually demonstrated.

Never claim “Iris compatible” simply because a shader file can be loaded.

---

# 35. Post-Processing

Optional effects may include:

- tone mapping,
- bloom,
- color grading,
- vignette,
- fog enhancements,
- god rays,
- depth effects.

Do not add expensive effects to the default vanilla path unless parity requires them.

Keep optional enhancements switchable.

---

# 36. WASM / WebGPU / WebGL2

The project should preserve a strong browser target where practical.

Architecture rule:

```text
                    Shared engine logic
                           │
              ┌────────────┴────────────┐
              │                         │
         Native backend             Web backend
      Vulkan / DX12 / Metal     WebGPU / fallback
```

Platform-specific functionality must be abstracted.

Watch for:

- filesystem limitations,
- asynchronous asset loading,
- browser threading rules,
- Web Workers,
- `SharedArrayBuffer`/cross-origin isolation requirements when relevant,
- WebGPU feature availability,
- WebGL2 restrictions,
- browser audio policies,
- compute shader availability.

Do not weaken the native renderer purely to accommodate the lowest common browser capability.

Use capability tiers.

---

# 37. Benchmarking and Profiling

Create a reproducible benchmark suite.

## Scene categories

1. Empty superflat scene
2. Normal terrain
3. Forest
4. Village
5. Cave
6. Nether
7. Dense redstone
8. Many transparent blocks
9. Large render distance
10. Fast player movement causing aggressive chunk streaming
11. High entity count
12. Many dynamic lights
13. Shader-enabled scene
14. Upscaled low-resolution scene

## Metrics

Measure:

- average FPS,
- median frame time,
- 1% low,
- 0.1% low,
- worst-frame time,
- CPU frame time,
- GPU frame time,
- render submission time,
- chunk generation latency,
- meshing latency,
- light-update latency,
- asset-load time,
- RAM,
- GPU memory where queryable,
- number of draw calls,
- indirect draws,
- vertices,
- indices,
- queue lengths,
- worker utilization.

---

# 38. CPU Scaling

Test at least:

- 2 physical cores,
- 4 physical cores,
- 6 physical cores,
- 8 physical cores,
- 12+ physical cores.

Do not expect linear scaling.

Report:

```text
2 cores → baseline
4 cores → scaling
6 cores → scaling
8 cores → scaling
12+    → scaling / saturation
```

Find the point of diminishing returns.

Thread count should be adaptive by default and manually overrideable for testing.

---

# 39. Differential Testing Against Minecraft 1.16.5

This is one of the highest-value parts of the project.

Use a legally obtained local reference installation as a black-box oracle.

For a given test:

```text
input seed/state/configuration
        │
        ├──────────────► Minecraft 1.16.5
        │                    │
        │                    ▼
        │                 observed output
        │
        └──────────────► VoxelCraft-Rust
                             │
                             ▼
                          observed output
```

Compare:

- block states,
- positions,
- collision outcomes,
- player movement,
- light levels,
- fluid states,
- tick outcomes,
- world generation,
- entity states,
- inventory behavior,
- crafting results,
- redstone outcomes,
- serialized data where targeted.

When a difference appears:

1. preserve a minimal reproducer,
2. determine whether Minecraft, VoxelCraft, or the test oracle is wrong,
3. document the exact behavior,
4. add a regression test.

---

# 40. Golden Tests

Create deterministic fixtures for:

- block states,
- model transforms,
- mesh output,
- texture coordinates,
- light propagation,
- player movement,
- fluids,
- redstone,
- world generation,
- save/load,
- inventory logic.

Store compact test vectors rather than proprietary assets.

---

# 41. Rendering Validation

Use image-based comparisons only with legally available/reference-controlled assets and test scenes.

Measure:

- pixel differences,
- structural differences,
- geometry differences,
- lighting differences,
- shadow differences,
- UV/texture differences,
- transparency differences.

Do not optimize for screenshot similarity at the expense of actual game behavior.

---

# 42. Memory and Cache Locality

Profile:

- bytes per block,
- bytes per loaded chunk,
- bytes per mesh vertex,
- allocator activity,
- cache misses where tools permit,
- copy volume,
- upload volume,
- queue contention.

Optimize for locality only after profiling.

Potential techniques:

- SoA for hot iteration data,
- packed arrays,
- compact runtime IDs,
- region-local storage,
- immutable snapshots,
- arena allocation,
- pooled buffers.

---

# 43. GPU Upload Strategy

Investigate:

- staging buffers,
- ring buffers,
- persistent/reused allocations,
- batched uploads,
- section-level updates,
- upload budgeting per frame.

Avoid a design in which one changed block causes a huge synchronized GPU upload.

---

# 44. Frame Pacing

Average FPS is not enough.

The target is:

> **high FPS + low frame-time variance + minimal chunk-streaming stutter**

Implement diagnostics that expose frame spikes and their causes.

When possible, record:

```text
simulation
chunk generation
meshing
GPU upload
draw preparation
render
present
```

for the same frame.

---

# 45. “Do Not Cargo-Cult” Rules

Do not implement the following solely because another engine/mod uses them:

- AtomicPtr world grids
- ECS
- 8-byte vertices
- deferred rendering
- Forward+
- MDI
- GPU-driven rendering
- clustered lighting
- cascaded shadows
- bindless resources
- giant 64 MB buffers
- 64-chunk render distance
- 16 worker threads
- PBR
- shader packs
- FSR

Every one must have a reason tied to measured constraints or compatibility.

---

# 46. Error Handling and Resilience

The engine should gracefully handle:

- missing texture,
- missing model,
- malformed JSON,
- unsupported resource-pack feature,
- corrupt save,
- unsupported graphics feature,
- shader compilation failure,
- out-of-memory pressure,
- browser capability restrictions.

Never crash because a user supplied an imperfect resource pack.

Fall back to safe assets/materials.

---

# 47. Versioning

All persistent data should carry version information.

When a schema changes:

- migrate old settings,
- migrate internal caches if needed,
- invalidate stale compiled assets,
- invalidate incompatible shader caches.

Do not silently interpret an old binary cache as a new format.

---

# 48. Priority Roadmap

## Phase 0 — Baseline

- repository audit,
- build verification,
- benchmark harness,
- frame-time instrumentation,
- regression framework.

**Gate:** establish a measurable baseline.

## Phase 1 — Block/Asset Foundation

- BlockState registry,
- model parser,
- blockstate parser,
- resource-pack loader,
- texture pipeline,
- animated textures.

**Gate:** representative 1.16.5 blocks render correctly.

## Phase 2 — World Data

- section storage,
- palettes,
- light containers,
- serialization,
- world/chunk loading.

**Gate:** deterministic read/write tests pass.

## Phase 3 — Mesh System

- hybrid mesher,
- complex models,
- fine-grained invalidation,
- packed vertex benchmarks,
- GPU upload batching.

**Gate:** no regression against baseline frame time.

## Phase 4 — Lighting

- block light,
- skylight,
- incremental updates,
- chunk-boundary propagation.

**Gate:** differential light tests pass.

## Phase 5 — Vanilla Rendering

- shadows,
- correct vanilla-style lighting,
- transparency,
- fog,
- biome tint,
- particles,
- animated materials.

**Gate:** representative rendering comparison passes.

## Phase 6 — Simulation

- player physics,
- fluids,
- scheduled ticks,
- redstone,
- entities,
- AI.

**Gate:** deterministic simulation regression suite.

## Phase 7 — Gameplay

- inventories,
- crafting,
- furnaces,
- brewing,
- enchanting,
- villagers,
- structures,
- dimensions,
- world generation.

**Gate:** gameplay feature matrix.

## Phase 8 — UI/Audio

- HUD,
- menus,
- settings,
- F3,
- sound events,
- music/ambient audio.

**Gate:** usability and resource-pack tests.

## Phase 9 — Advanced Performance

Only now evaluate:

- indirect draws,
- MDI,
- GPU-driven visibility,
- occlusion,
- LOD,
- more aggressive batching,
- lock-free structures.

**Gate:** benchmark improvement must be measurable.

## Phase 10 — FSR

- real FSR 1 EASU,
- real RCAS,
- proper integration,
- scaling presets.

**Gate:** quality/performance comparison.

## Phase 11 — Shader Compatibility

- native shader API,
- shader-pack metadata,
- compatibility subset,
- transformation layer,
- progressively broader compatibility.

**Gate:** demonstrated compatibility with explicitly tested packs.

---

# 49. Definition of Done

A feature is not “done” because code compiles.

A subsystem is done only when:

1. implemented,
2. tested,
3. documented,
4. benchmarked where performance-sensitive,
5. integrated with the actual engine,
6. regression-tested,
7. failure cases handled,
8. version behavior verified,
9. no known severe correctness regression remains.

---

# 50. Autonomous Agent Execution Protocol

For every implementation task:

### Step A — Inspect

Read the relevant source and current architecture.

### Step B — Research

Verify version-specific facts and API constraints.

### Step C — Design

Propose the smallest change that can achieve the goal.

### Step D — Implement

Modify only necessary systems.

### Step E — Test

Compile, run unit/integration/regression tests.

### Step F — Benchmark

For performance-sensitive changes, compare before/after.

### Step G — Validate parity

Run differential tests where applicable.

### Step H — Document

Record what changed, why, measurable impact, and remaining limitations.

### Step I — Commit safely

Make small, logically grouped commits.

Never perform a giant unrelated rewrite simply because a newer architecture sounds cleaner.

---

# 51. Final Strategic Direction

The end state should look conceptually like:

```text
Minecraft Java Edition 1.16.5
          │
          │ observable behavior / compatible formats
          ▼
┌────────────────────────────────────────────┐
│            VoxelCraft-Rust                 │
│                                            │
│ Rust                                      │
│ wgpu                                      │
│ deterministic simulation                  │
│ multicore worker system                   │
│ data-driven assets                        │
│ efficient world storage                   │
│ hybrid high-performance renderer          │
│ native + WebGPU architecture              │
│ optional modern enhancements               │
└────────────────────────────────────────────┘
```

The project's competitive advantage should not be:

> “It copies another engine's tricks.”

It should be:

> **“It reproduces the target behavior while being engineered from the beginning for modern CPUs, GPUs, memory hierarchies, frame pacing and scalable workloads.”**

---

# 52. Final Source Hierarchy

When resolving conflicting information, prefer:

1. Official Minecraft/technical documentation where available
2. Official AMD GPUOpen documentation for FSR
3. Official wgpu documentation
4. Official Vulkan/WebGPU/Metal/DX documentation
5. Primary GitHub repositories of Iris/Sodium and other relevant projects
6. Reputable technical research
7. Community documentation
8. Forum/Reddit discussion only as supporting evidence

Never use a low-quality community claim as the sole basis for a critical architectural decision.

---

# 53. Master Instruction to the Coding AI

**Treat this document as a living engineering specification.**

Before every major implementation:

- inspect the current VoxelCraft-Rust repository,
- verify the target Minecraft 1.16.5 behavior,
- check current dependencies and platform constraints,
- preserve working functionality,
- prefer safe designs,
- benchmark performance assumptions,
- create regression tests,
- document uncertainties,
- and never present an approximation as exact parity.

The final objective is not merely a Minecraft-like game.

The objective is:

> **A highly capable, independently implemented, Minecraft Java 1.16.5-oriented voxel engine with strong observable parity and a modern, scalable Rust/wgpu architecture — while preserving a clean separation between vanilla parity and optional performance/visual enhancements.**
