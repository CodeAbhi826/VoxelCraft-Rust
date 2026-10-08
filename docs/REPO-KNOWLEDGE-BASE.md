# REPO-KNOWLEDGE-BASE.md — the complete repository knowledge file

> The live chronology is `docs/WORKLOG.md` (append entries per slice);
> root `worklog.md` is the prior era's closed log (read-only). THIS file
> is the complete, always-CURRENT picture
> of the whole repository, at source-code-level depth: if you hand this file
> to an engineer who has never seen the repo, they get a full, accurate
> understanding of how everything works — the architecture, every crate's
> internals, every system's mechanics with the actual formulas and flows,
> the build/run/test recipes, the decisions and their reasons, the known
> gaps, and the history. Updated IN PLACE as things change; a section whose
> fact changes gets edited, never left stale. Everything below is
> code-read/verified by the maintaining agent THIS era — never copied from
> stale research dumps (the old audit files in docs/research/ describe OLD
> repo states and contain errors in both directions — treat them as
> history, not truth).

---

## PART 1 — THE OVERVIEW, THE BUILD, THE CRATE MAP

## 1.1 What this repository is

VoxelCraft — a clean-room Rust voxel engine + survival game in the mold of
the 1.16.5-era block game. The parity target: the 1.16.5 mechanics set (the
simulation constants, the block behaviors, the mob AI, the recipes — every
constant carries a live-verified citation from the reference wiki, read as
raw wikitext through the MediaWiki API). Every asset (textures, sounds,
font) is procedurally synthesized IN-PROJECT from measured values — none of the
reference game's material, the binding legal policy in docs/LEGAL-COMPLIANCE.md.
(Corrected 2026-10-06: the ACTIVE font is Monocraft, IdreesInc, SIL OFL 1.1,
shipped with its licence; Voxelfont is the in-repo original spare. The old
“zero third-party material” wording was inaccurate.)

The numbers (current, October 2026 — counted from the tree and CI logs):
- 14 `vc-*` crates + the `voxelcraft` app — ~170,000 lines of Rust.
- wgpu 22 (the graphics), glam 0.29 (the math), naga (the runtime WGSL
  validation), winit (the windowing), mimalloc (the native allocator,
  platform convention), rustc-hash/FxHash (the integer-keyed maps — the
  platform convention: every integer-keyed map in the engine uses FxHash
  over std SipHash), web-time (NOT std::time — std Instant PANICs on
  wasm32), libm (pure-Rust sin/cos/sqrt — worldgen never touches the
  platform libm, R5).
- 20 Hz deterministic simulation (the sim is fixed-step and reproducible
  from the seed — the same seed always produces the same world and the
  same tick stream).
- 1.16.5 parity: BLOCK_COUNT 539, STATE_COUNT 892, 32 effect kinds (the
  COMPLETE 1.16.5 set), 66 declared MobKinds (65 MOB_DATA rows — Squid is
  a classification-only stub), 28/66 biomes, ~1120 recipe entries.
- 930 `#[test]` attributes; CI sums the `test result:` lines per run
  (latest full-green: 933 passed / 0 failed / 2 ignored — counts move
  with the suite; never quote a stale total, re-count from the log).
- Repo is PRIVATE (owner order 2026-10-08); owner hardware specs never
  appear anywhere (AGENTS.md hardware-privacy rule).

## 1.2 How to build, run, and verify (the exact recipes)

The workspace root (the directory holding the root Cargo.toml) is
`voxelcraft/`. The repo root is `CodeAbhi826/VoxelCraft-Rust` on GitHub,
branch `test/full-sweep-2026-09-25`.

```sh
# HEAVY WORK RUNS ON GITHUB ACTIONS ONLY — never locally (owner rule;
# rustfmt is the only local command besides quick filtered test runs on
# an idle machine). The 9 CI jobs (ci.yml, every push):
#   1. legal audit  — scripts/legal_audit.py (the trademark scanner)
#   2. cargo fmt --check
#   3. cargo test --release --no-default-features --workspace
#   4-6. worldgen golden-hash gate on ubuntu + windows + macos (R5:
#        bit-identical terrain on all three OSes)
#   7. cargo check --release --no-default-features --workspace --lib \
#        --target wasm32-unknown-unknown
#   8. cargo clippy -- -D warnings
#   9. the headless bench (uploads the JSON + the vc_bench binary)
# NOTE (2026-10-08): artifact uploads fail while the Actions storage
# quota is exhausted (recalculated every 6-12h) — code jobs still
# validate; close-outs wait for green including uploads.

# the game binary (the single file, the builtin pack embedded by build.rs):
gh workflow run linux-game.yml --ref test/full-sweep-2026-09-25
gh run watch <run-id> --exit-status
gh run download <run-id> -n voxelcraft-linux-single-file -D /tmp/vc-e2e
chmod +x /tmp/vc-e2e/voxelcraft-*linux-x64   # the glob: HYPHEN not dot

# the in-game E2E legs under Xvfb (the owner's rule: the in-game test is
# the most important thing — the code/test-only pass is NOT enough).
# Every gameplay/rendering slice gets its own linux-game run at its
# commit (manual dispatch — linux-game.yml runs on dispatch + main only):
E2E_FKEYS=1  xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
E2E_BEDS=1   xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
E2E_FLUIDS=1 xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
E2E_CONTAINERS=1 xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke
E2E_PHASES=1 xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
E2E_TURNTABLE=1 TURNTABLE_MOB=creeper xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
# the legs exit 1 on a FAILED contract; the local pipe can mask the code —
# grep the verdict lines, never trust the pipeline exit code alone.
# Foreground runs (owner-approved, only when CI can't answer): download
# the CI-built binary and run it here under xvfb-run + MangoHud.

# the headless benchmark (the perf baselines):
cargo run --release --no-default-features --features bench-bin --bin vc_bench -- chunks=96 json=bench-headless.json
```

The CLI flags (main.rs): (no args) the normal launch (intro → title →
world); `--verbose` THE raw-log flag (the full raw stream — see §9.3);
`--smoke` the CI smoke run; `--benchmark [frames=600] [warmup=120]
[seed=…] [json=bench.json] [streaming]` (the streaming walk: 8 b/s at eye
height through fresh terrain); `--gpu-timing` (scene-pass GPU timestamps
when supported); `--help`/`-h` the usage card. (The old `--debug` name is
retired — the same stream, everything raw.)

## 1.3 The crate map — what each crate owns, in dependency order

The dependency direction is STRICT: the lower crates never depend on the
higher ones. vc-nbt/vc-rng/vc-blocks sit at the bottom (the pure data);
vc-chunk/vc-inventory/vc-pack next; vc-world (the world + the light +
the generation + the save); vc-mesh/vc-particles/vc-gameplay/vc-sim/vc-anvil
(the simulation + the gameplay); vc-render/vc-audio (the presentation);
the voxelcraft app on top (the game loop).

| Crate | Owns (the actual file map) |
|---|---|
| **vc-nbt** | The NBT (Named Binary Tag) read/write — the save format's data layer (the region files' compound tags). Zero-unsafe, the tag tree in/out. |
| **vc-rng** | The deterministic RNG: the xorshift stream (`RandomTicker`-grade), `Rng::next_f32`/`next_range` (the modulo-bias note: negligible at n ≤ 32). Every system's randomness seeds from the world seed — the same seed reproduces everything. |
| **vc-blocks** | `src/blocks.rs` (~16k lines): the block/state registry. BLOCK_COUNT 539, STATE_COUNT 892. `BlockDef` (the name, the 3 face tiles, opaque/solid/cross/fluid flags, the light emission, the sound family), the BLOCK_TABLE (the def array — a new BlockDef MUST join it), PICKER_BLOCKS (471, the creative-inventory order), CREATIVE_TABS (9) + creative_tab/creative_tab_items (the census pinned), state_block (the state → block fold — a LONG match chain over the state windows), default_state (the block → its placement state), the state windows: is_v2..is_v16_state (the version brackets 1.7.2..1.16.5), is_v17_state (the armor), is_r13_state (the stations), is_bed_state, is_fire_block/fire_age/fire_age_state/FIRE_AGE_BASE 877/FIRE_AGE_MAX 15, is_waterlogged_state/waterlogged_state (the chest pair 874/875), BUBBLE_COLUMN 538/876, NETHER_PORTAL 536/872, FLINT_AND_STEEL 537/873, TNT 533/863, water_level/water_state/lava_level/lava_state, face_visible (the cull rules — the water/lava/leaves/glass/ice/portal/bubble same-cull classes), is_opaque, break_time_secs (the vanilla hardness table), flammability (the wiki's ignite/burn odds), state_tiles, block_tint_color + tint.rs (the biome tint packs — the KIND_WATER/KIND_GRASS/KIND_LEAVES rows, the per-biome color LUTs). |
| **vc-chunk** | `src/chunk.rs`: the 16×256×16 chunk (CHUNK_LEN 65536). `Chunk::sections: [Option<Arc<Section>>; 16]` — the EMPTY section = all-air (None = no allocation, ~2.5 KiB per populated section). `Chunk::get(x,y,z)` FOLDS to the owning BLOCK id (through state_block — never truncate; the historical `as u8` would alias the high states onto low ids); `Chunk::get_state` is the RAW state accessor (the honest one for the property variants); `Chunk::set` routes the block id through default_state (the generator-side guarantee — villages once placed furnaces as slabs); `set_state` is the raw write; the copy-on-write at SECTION granularity (Arc::make_mut clones only the affected section when the chunk is shared with in-flight mesh jobs). `idx(x,y,z)` = the linear index. |
| **vc-pack** | The read-side interop: FolderSource/NsAliasSource (whatever folder prefix a user pack carries resolves onto the flat key space), the model dispatch (BlockDispatchSpec → compile_block_dispatch → the ModelSet's by_state map), legacy_aliases.rs (the retired coined names → the real terms, read-side only, never rendered). |
| **vc-inventory** | The inventory: the 36-slot grid (hotbar 9 + main 27), ItemStack (block, count), the stack rules, INV_SLOTS. |
| **vc-world** | `src/world.rs`: the World (see §1.4). `src/gen.rs`: TerrainGen — the terrain generation (the biomes 28/66, the Biome enum + from_u8 + precipitation, the structures: village/dungeon/mineshaft/pyramid/jungle temple/stronghold/fortress/mansion/icebergs), vanilla_noise.rs (the Perlin formulation — the doubled 512-entry permutation table). `src/light.rs`: the LightEngine — the sky+block channels, init_chunk (THE #1 perf cost — see §6.4), the incremental on_block_changed (the ±15 block-light / the sky down-column regions), the pending sets. `src/anvil.rs`: the region-file save/load (the 1.16.5 format — the Biomes IntArrays ground-truthed against the owner's real saves). |
| **vc-mesh** | `src/mesh.rs`: the greedy mesher (see §6.1). mesh_sections (the reference CPU path), build_mesh_inputs (the 3×3 padded snapshot + has_cross/has_models flags), MeshData (the solid/water buffers). |
| **vc-particles** | The shared particle stream: ParticleVertex (pos/uv/col), spawn_kind (the named kinds: explosion_emitter, dust, dripping_water, the bubble trail, the splash burst...), 20/90 kinds. |
| **vc-gameplay** | The gameplay systems, one module each: mobs.rs (66 declared MobKinds — see §5.1), combat.rs (the melee/explosion math), effects.rs (32 effect kinds — see §5.3), hunger.rs (the food/hunger/exhaustion), sleep.rs (the bed decision layer), portal.rs (the nether portal), weather.rs (the rain/thunder machine), furnace.rs/campfire.rs/brewing.rs/enchanting.rs (the block entities), bees.rs (the hive system), villagers.rs (the NPC AI + the trades), spawners.rs, dragon.rs (the End fight), anvil.rs (the repair), beacon.rs, fishing.rs, craft.rs (the recipe matching), datapack.rs (the loot/set_count), entity_model.rs (the 12 jointed rigs + billboard fallback). |
| **vc-sim** | `src/sim.rs`: the 20 Hz Sim (see §1.5). `src/ticks.rs`: TickScheduler (the delayed ticks) + RandomTicker (the deterministic per-chunk sampling). `src/fluids.rs`: the water/lava/bubble/waterlogging ticks (see §5.5). `src/entities.rs`: the ItemSystem/ItemSystem, XpOrbSystem, PrimedTntSystem (see §5.6). `src/redstone.rs`: the redstone components (the wire/torch/lever/repeater/comparator/piston/QC/observer/dispenser/hopper/plates/target/daylight sensor). |
| **vc-anvil** | The region-file format (the native save target — §8). |
| **vc-render** | `src/render.rs` (~7k lines): the Renderer (the surface, the pipelines, the atlas, the debug stream), render.rs's report_boot_log/report_debug_log (the raw stream sinks), the chunk-border overlay, the particle draw, the post chain (bloom → EASU → composite+RCAS → pack stages → UI), the FXAA pass (pre-upscale), scene-pass GPU timestamps (1.5). `src/gpu_mesh.rs`: the GPU compute mesher (see §6.2) — the WGSL greedy key + the LUT mirrors. `src/textures/` (TILE_MAX 818): generate_atlas (the 512×512 tile atlas — the clean-room painters per module: portal_art/tnt_art/weather_art/farming_art/v112..v116_art/e2_art/e3_art/r13_art/gui_art + the 16 mob billboard tiles + the player skin), merge_pack_textures (the user-pack override + the animated strips). `src/gui/`: the HUD + the container screens (the GuiTextureSet, the SpriteSheet, the 9-slice panels, the 32 effect icons). `src/ui.rs`: the UI canvas (the widget tree, the click routing, the screens — engine page carries the UPSCALING/SHARPNESS/CUSTOM/AA/GEN-THREADS buttons). |
| **vc-audio** | The sound events (the SoundEvent family/volume/pitch), the .ogg decode, the SoundRegistry (the data-driven §21 registry), SoundBank. |
| **voxelcraft (app)** | `src/game.rs` (~29k lines): GameApp — the update loop (see §1.6), the E2E legs (FKEYS/BEDS/FLUIDS/CONTAINERS/MENU/PHASES/TURNTABLE — §9.2), the explosion, the interactions, the streaming (dedicated cores-1 pool, 6 ms apply budget, stale-mesh drop, mesh-first order), the UI. `src/player.rs`: Player — the movement/combat/effects/detection (see §5.2). `src/main.rs`: the CLI. `src/bench.rs`: the benchmark harness (orbit/streaming cameras, the 5-phase meter with the 0.9-coverage contract). `src/wasm_entry.rs`/`web_input.rs`: the browser build. `src/alloc_stats.rs`: the counting allocator (the F3 "Allocated" telemetry). |

## 1.4 The World (vc-world/src/world.rs) — the core data model

```rust
pub struct World {
    pub seed: u64,
    pub dimension: Dimension,          // Overworld | Nether | End
    pub gen: TerrainGen,
    pub chunks: FxHashMap<ChunkPos, Arc<Chunk>>,     // ChunkPos = (i32, i32)
    pub light: FxHashMap<ChunkPos, Arc<LightData>>,  // the persistent per-chunk light
    pub decorated: FxHashSet<ChunkPos>,  // fully generated + decorated (meshable)
    pub pending: FxHashMap<ChunkPos, Vec<(u16, u16)>>,  // the queued edits for ungenerated chunks
    pub dirty: FxHashMap<ChunkPos, u16>,  // §12: the stale SECTION bitmask per chunk
    pub dirty_causes: FxHashMap<ChunkPos, u8>,  // the accumulated causes (geometry/light)
    pub save_dirty: FxHashSet<ChunkPos>,  // the unsaved chunks (the autosave drains)
    pub journal: FxHashMap<[i32; 3], u16>,  // the web build's localStorage journal
    journaling: bool,
}
```

- **The chunk lifecycle**: the generation (gen.rs emits the block edits +
  the decorations) → `insert_generated` (the chunk + the light data enter
  the maps) → the mesh jobs (the §12 dirty bits drive the remesh) → the
  autosave (the save_dirty drain → the anvil regions).
- **The block edit path** (`set_block_state(wx, wy, wz, state)`):
  the y-range guard (0..=255) → the chunk lookup (`?` — MISSING CHUNK =
  NO-OP, the E2E legs wait for the chunk-ready guard because of this) →
  the Arc clone + the section COW write → the journal (the web build's
  persistence) → save_dirty → `mark_edit` (the §12 invalidation region:
  the geometry = the edit's section + the boundary-adjacent sections in
  this and the neighboring chunks (the face culling + the AO read ±1);
  the light = the block light within ±15 of the edit + the sky light down
  the whole column below it — the region is CONSERATIVE: correctness never
  depends on it being tight, only on covering the true change).
- **The dimension travel**: the whole World swaps (a fresh generator + the
  chunk maps) — vanilla-style; the Nether: 8:1 the coordinate scale, no
  skylight; the End: the void islands + the obsidian pillars.
- **The light engine** (light.rs): the sky+block channels per chunk
  (LightData, the Arc COW snapshot for the mesh jobs); `init_chunk` builds
  the initial light (the column scan + the emissive seeds + the BFS —
  THE #1 perf cost, 86.6 ms/chunk on the reference hardware tier — the Phase-4
  optimization design in docs/PHASE4-LIGHT-DESIGN.md); the incremental
  `on_block_changed` marks the EXACT changed sections (the changed map —
  no heuristics). The skylight propagation's deviation from vanilla's
  free-down rule is documented with a differential test oracle —
  disclosed, not a violation.
- **The biomes**: the Biome enum (28 of the 1.16.5's 66), from_u8 (the
  1.16.5 registry ids — GROUND-TRUTHED against the owner's real saves'
  59 region files: the ids [0,1,3,4,5,6,7,16,18,19,21,22,24,25,27,29,32,
  33,34,45,46,48,131,132]; Sunflower Plains = Java id 129 — the audit's
  WRONG row (130 = Desert M) resolved by the real data), precipitation
  (Rain/Snow/None).

## 1.5 The 20 Hz simulation (vc-sim/src/sim.rs)

```rust
pub struct Sim {
    pub sched: TickScheduler,          // the delayed ticks
    random: RandomTicker,              // the deterministic per-chunk sampler (pub seed)
    pub items: ItemSystem,             // the item entities
    pub xp_orbs: XpOrbSystem,          // Phase E1
    pub furnaces: Furnaces, pub campfires: Campfires, pub brewing: Brewings,
    pub enchants: Enchants,            // the block entities
    pub villagers: Villagers, pub spawners: Spawners,
    pub mobs: MobSystem,               // 50 MobKinds (§5.1)
    pub tnt: PrimedTntSystem,          // the primed TNT
    pub hives: HiveSystem,             // 1.15 bees
    pub is_day: bool,                  // set by the game layer from the sun state
    pub rain: bool,                    // set by the game layer (the fire tick's dousing)
    pub fire_difficulty: u8,           // 0-3 (the spread degree's d; Survival → 2, Hardcore → 3)
    pub sky_factor: f32,               // 1.0 clear, 12/15 rain, 10/15 thunder
    ...
}
```

- **The fixed-step loop**: `update(dt, world, light, scope)` accumulates
  `dt.min(0.25)`; while `acc >= 1/20`: `step(world, light, scope)`.
- **One tick** (`step`): (1) the SCHEDULED TICKS drain (the TickScheduler's
  due entries: the fluids/gravity/bubble/redstone/fire/plants — each
  position's block routes to its tick function; the light hook follows
  every sim-side block edit: `new != s → light.on_block_changed`); (2) the
  RANDOM TICKS (the RandomTicker samples 3 positions per chunk in the sim
  ring — the grass spread/die, the plant growth); (3) the block entities
  (the furnaces' 200-tick cook, the campfires' 600-tick, the brewing's
  400-tick); (4) the entity systems (the items/orbs/TNT/mobs/arrows).
- **TickScope**: the simulation-distance ring (`chunk_in(cx, cz)`) —
  Phase 6 §26: the ticks freeze outside the ring;
  `TickScope::everything()` = the 1.16.5 behavior (the E2E/tests use it).
- **The determinism**: every roll seeds from the world seed (⊕ the
  position ⊕ the tick) — the same seed reproduces the same tick stream
  exactly; the scheduler's dedupe: a position pending is never
  double-scheduled.

## 1.6 The game loop (voxelcraft/src/game.rs)

`GameApp` holds: window (&'static — the Box::leak gives the wgpu surface a
'static lifetime without cloning), renderer, world, player, ui (UiCanvas),
atlas (the 512×512 RGBA tile atlas), gui_set (the procedural GUI texture
set), icon_cache (the 3D item icons — the CPU-baked isometric models in a
2048×2048 GPU atlas, LRU 512, 4 bakes/frame), bank/sounds/audio_rng (the
audio), audio (the AudioBackend), settings (upscale ladder 0..5 + Sharpness
+ custom scale + AA mode + gen-threads knob), the streaming pool, and the
E2E/stat fields.

`GameApp::new(window)` (the boot): the wgpu adapter/device/queue → the
pipelines (the chunk opaque/water/translucent, the particles, the GUI
quads, the sky, the clouds, the selection box) → the atlas generation (the
clean-room painters) → the pack merge (the user-pack override) → the font
→ the sound bank → the intro screen.

`update(dt)` — the ORDER (each stage reads the world the previous stage
wrote):
1. `time += dt`; the day clock (`day_time = (day_time + dt/1200) % 1.0`
   — frozen while the E2E capture ladder is mid-pair, the T5 contract);
   `sim.is_day` (day_time < 0.5 || the Nether).
2. `weather_update(dt)`: the rain/thunder machine (the vanilla cycles),
   the lightning strikes (the 30 s cadence between flashes, the
   rain-exposed column pick), the rain wiring into the sim (`sim.rain`,
   `sim.sky_factor`, `sim.mobs.weather`).
3. The player update (the movement/combat/detection — §5.2).
4. The interaction timers: the break/place (the vanilla MINING MODEL —
   the progress over the hardness-derived break time × the Mining Fatigue
   factor, the 10-stage destroy overlay, the creative instant), the use
   (the beds/containers/portals), the attack (the melee — §5.4).
5. The scheduled E2E clicks (the smoke script) + the E2E legs (the
   world-entry + chunk-ready guards, the gated VERDICT lines — §9.2).
6. The particles (the ambient rolls: the portal shimmer, the redstone
   dust, the leaf drips in rain, the bubble trail, the splash burst).
7. The 1 Hz heartbeats: [perf] (the fps envelope, the frame phases, the
   chunk depths, the mobs), [sim] (the internals — §9.3), [gfx] (every
   screen — the UI pipeline), the autosave (20 s cadence, the save_dirty
   drain → the anvil regions / the web journal), the F3 stats publish.
8. The phases (the frame's five phases measured): PHASE_SIM (the mob
   anchor + `sim.update`) → PHASE_STREAM (`stream()`: the job results'
   collection + the GPU-mesh advance + the chunk gen/mesh dispatch +
   `PHASE_RESULTS` inside) → PHASE_UI (`rebuild_ui()`: the widget tree)
   → PHASE_DRAW (`draw()`: the render passes + the PHASE_DRAW timing).
   The meter spans update+draw (`begin_frame` in AboutToWait, `end_frame`
   at the end of `draw()`); the contract `sum(phases) >= 0.9 × frame`
   holds as a unit test AND as the E2E_PHASES leg's in-game verdict.
   Streaming runs on the dedicated cores-1 pool with worker-scaled
   dispatch caps, a 6 ms time-boxed apply drain (mesh results first,
   leftovers in `pending_apply`), and stale-mesh drops (dirty bits
   preserved). The capability probe logs the hardware tier at boot.

## 1.7 The screens and the UI (vc-render/src/ui.rs + gui/)

The screen enum: Intro, Title, Create (the world creation: the seed
buffer, the mode Survival/Hardcore), Game, Pause, Inventory, Crafting,
Chest/Furnace/Brewing (the container screens), Death, and the debug
overlays. The widget tree: the buttons (the 9-slice chrome, the hover
variant with the verified #FFFFFF @ alpha 51 overlay), the slots, the
panels, the labels (the Voxelfont bitmap font — the clean-room 5×8
glyphs), the item icons (the 3D isometric bakes + the flat blit fallback).
The click routing: the widget id/kind/label + the hover/hit cross-check
(the [input] debug trace — the "click sound but nothing opens" report
class).

---

## PART 2 — THE GAMEPLAY SYSTEMS, IN DEPTH

## 5.1 The mobs (vc-gameplay/src/mobs.rs)

`MobKind` — 66 declared variants of the 1.16.5 set (65 `MOB_DATA` rows;
Squid is a classification-only stub, never spawned). 12 jointed entity
rigs exist (player/cow/pig/sheep/horse/donkey/mule + families); the rest
render as billboard sprites. Each kind carries a `MobDef`:
the health, the damage, the speed_attr (× SPEED_PER_ATTR), the hitbox,
the tile (the flat billboard sprite; the entity-model loading path is a
L-tier gap — the mobs render as billboards today), the sound family,
`hostile()`, `prevents_sleep()` (the bed gate's monster kinds).

- **MobSystem::tick(world, sim_center, sim_radius)**: per mob —
  `ai_tick` (the AI), `hazard_tick` (the environmental hazards), then
  `physics_tick` (the shared entity integrator: gravity 0.04, drag 0.98,
  the per-axis collision probing).
- **The anchored player**: `mobs.player: Option<[f32; 3]>`,
  `player_invulnerable: bool`, `player_health: f32` — set by the game
  layer each update BEFORE the tick (the spawns/AI need the player).
- **ai_tick** (19 params — the queues pass in, the game layer drains):
  the per-kind match: the hostile melee chase (the
  MOB_MELEE_REACH/ MOB_MELEE_TICKS cadence, the PlayerHit payload),
  the ranged attacks (the skeleton's arrows via the ProjKind, the ghast's
  fireball, the witch's splash potions — the verified ladder), the
  environmental behaviors (the zombie→drowned conversion after 30 s
  submerged, the turtle nesting, the bee pollination, the strider's
  lava walk), the flee/pacify rules (the hoglin's warped-fungi flee, the
  piglin pacification, the bee's anger swarm), the evoker/illusioner
  spell queues.
- **PlayerHit** (the game layer's consumption): damage (the
  NORMAL-difficulty value — scale via combat::difficulty_scale), source
  (the MobKind), knockback_dir, wither_effect (Some(ticks) → Wither II),
  poison_effect (Some(ticks) → Poison I), potion_effect (the witch's
  splash rider: Some((kind_java_id, amplifier, ticks))).
- **The drops** (game.rs's mob-death path): the per-kind drop tables (the
  verified 0-2 rolls), the wither-skeleton's skull 2.5% (the special
  roll), the 8.5% potion drop (the witch, killed while drinking).
- **The spawns**: the hostile light rule (the light ≤ 7? the engine's
  spawn light gates), the passive grass rule (light ≥ 9 on GRASS, cap 10,
  herds of 2-4, gated at 1/20 per attempt), the Nether biomes' sets.

## 5.2 The player (voxelcraft/src/player.rs)

- **The fields**: pos/vel (the feet-anchored 0.6×1.8×0.6 hitbox),
  yaw/pitch, on_ground, health (20.0 + the Health Boost), food/saturation/
  exhaustion/food_tick_timer (the Hunger struct), air (300 = full — 10
  bubbles × 30; the drowning 2 HP/s after the 15 s breath), effects (the
  32-kind Effects), inv (the 36-slot inventory), xp_level/xp_points, the
  detection flags: in_water/head_in_water, in_lava, in_fire/in_soul_fire,
  fire_ticks (the Fire tag), bubble_kind/head_in_bubble, in_portal/
  portal_accum, on_vine, collided_h (the surface hop).
- **update(dt, ?, world, input, ?, ?)** — per frame: the detection block
  (the feet/head cell reads: the water level + the fluid height →
  in_water/head_in_water; the folded block id → in_lava/in_fire/
  in_soul_fire/on_vine/in_portal + the bubble kind scan) → the movement
  (the walk-speed selection: the swim speeds × 3, SPRINT_SPEED, the sneak
  caps 1.3/1.8, WALK_SPEED 4.317 — × the stat multipliers (the Speed/
  Slowness/Dolphin's Grace effects); the water control rate 4.0, the
  ground 12.0, the air 2.5; the sprint-jump extra drag) → the 20 Hz
  substep (the gravity: the water/lava travel form v←v×0.8−0.02 (the lava
  0.5 drag), the levitation float, the fall distance, the flowing-water
  current push) → the air meter (the drowning) → the FOV/sounds.
- **The combat**: `swing_t` (the 1.9 attack-cooldown), the melee outcome
  via combat::player_melee(held_block, cooldown_p, falling, sprinting,
  armor, toughness) − the Weakness bonus (−4 HP × level, the ≤ 0 attack
  fails to connect); the critical (falling && !sprinting && cooldown full)
  ×1.5; the held weapon's base (the sword axes' table).
- **The damage paths**: `damage(amount)` (the armor recompute, the hurt
  flash 0.25 s), `starve(amount)` (the unblockable class), the timed
  damage in game.rs's sim-tick loop (the lava 4 HP per 10 ticks, the fire
  1 HP per 10 ticks / the soul fire 2, the outside burn 1 HP per 20, the
  Fire Resistance gate), the fall damage (the MC-12357 formula:
  fall_distance − 3 HP, the water/flight zero it).

## 5.3 The status effects (vc-gameplay/src/effects.rs)

32 kinds — the COMPLETE 1.16.5 set (the Java ids 1..32; Darkness 33 and
the 1.21 omens are post-1.16.5, excluded). `EffectKind::java_id()` is the
registry-order table. `Effects { active: Vec<Effect>, acc: Vec<i32> }` —
`apply(kind, amplifier, ticks)`, `amplifier(kind) -> Option<u8>`,
`tick(health) -> (dmg, heal)` (the period_ticks cadence per kind: the
wither 40>>level, the poison 25>>level floored at 10 (the immunity
window), the regeneration 50>>level, the saturation 1 (the per-tick
restore), the stat effects i32::MAX). The accessors the paths read:
speed_multiplier (the +20%/level), slowness_multiplier (the −15%/level),
strength_bonus (+3 HP/level... the wiki's +3), resistance_multiplier
(−20%/level), jump_boost_bonus (+0.1/level velocity), mining_fatigue_factor
(0.3^min(level,4)) + mining_fatigue_attack_speed (−10%/level),
fire_resistance_active, invisibility_active, night_vision_active,
nausea_active, glowing_active, weakness_bonus (−4 HP × level),
saturation_restore ((1, 2) × level per tick), health_boost_bonus
(+4 HP/level), levitation_velocity (0.9 × level — DOCUMENTED
APPROXIMATION, the wiki publishes no scalar), luck_amplifier (+/−),
water_breathing_active, slow_falling_active, conduit_power_active,
dolphins_grace_multiplier (×2 — the documented approximation).
DISCLOSED future hooks: the Luck/Bad Luck loot modifier (the drop rolls),
the Bad Omen raid trigger, the Hero of the Village trade discount.

## 5.4 The combat math (vc-gameplay/src/combat.rs)

- `player_melee(held_block, cooldown_p, falling, sprinting, armor,
  toughness)`: the base = held_attack(block) × cooldown_damage_scale(p)
  (the 1.9 cooldown curve); the crit ×1.5 (falling && !sprinting && the
  cooldown > 0.9); the armor reduction (the 1.9 armor formula: the
  damage × (1 − min(20, max(armor/5, armor − damage/(2 + toughness/4)))/25)).
- `explosion_damage/impact/knockback` (the VERIFIED explosion math): the
  impact = (1 − distance/(2·power)) × exposure; the damage = the
  (2·impact... the scaling); the knockback = the impact along the
  eye-vector; `explosion_drop_chance(power, tnt)` (TNT 100%, others
  1/power); `difficulty_scale(dmg, difficulty)` (the player's rows:
  Easy min(dmg/2+1, dmg), Normal dmg, Hard ×3/2 — the mobs take the
  unscaled value).
- `armor_reduce`, `held_attack`, `attack_cooldown_ticks`.

## 5.5 The fluids (vc-sim/src/fluids.rs) — the full mechanics

- **Water**: WATER_TICK_RATE 5 (1 block/5gt = 4 blocks/s); the levels
  0 (source)..7 (flowing) + the falling-water block (the level-8 state —
  the mesher renders full height); the tick: the fall first (air below →
  pour down level 1, the falling water does NOT spread horizontally this
  tick), the mixing products, the source conversion, the horizontal
  spread (the level + drop ≤ 7; the down-first weight rule — the flow
  weights: the direction weights start 1000, the shortest path to a way
  down wins).
- **The infinite water source** (VERIFIED w/Water §Source blocks, live
  2026-10-03): "a water source block is created from a flowing block that
  is horizontally adjacent to two or more other source blocks, and sitting
  on top of a solid block or another water source block"; the vertical
  variant: "a flowing block adjacent to one source block horizontally and
  one vertically above the flowing block". The water_tick converts the
  flowing cell → WATER (a source).
- **The mixing products** (VERIFIED w/Fluid §Mixing + w/Water §Water and
  lava — the two sources AGREE, the cross-check): "if water touches a
  lava source, the lava source turns to obsidian. If both touch each
  other while flowing, cobblestone is made and no sources are removed,
  and if lava flows downward onto water, the water turns to stone." The
  water_tick: the below cell or a horizontal neighbor being a lava SOURCE
  → obsidian. The lava_tick: the flowing lava (level > 0) touching water
  (above or the sides) → cobblestone; the lava falling onto water → the
  WATER turns to stone.
- **Lava**: the 30/10-tick rates (3 blocks/30 ticks Overworld+End, 7
  blocks/10 ticks the Hollow — the dimension-gated rate); the drop-off
  2/1; no sources created (the lava never spreads sources).
- **The waterlogging** (VERIFIED w/Waterlogging): the waterlogged cell
  carries a FULL water source in the same block space ("Both the
  non-cube block and the water source block occupy the same space"). The
  chest pair's dedicated states 874/875; flowable() = AIR + a dry
  waterloggable container; the spread target state: the container takes
  its waterlogged variant; the mesher renders BOTH the container AND the
  water overlay (level 0, the WATER tint/culling — CPU + GPU parity).
  DISCLOSED TRIM: slab/stairs/fence waterlogging (the JSON-model
  dispatch).
- **Bubble columns** (VERIFIED w/Bubble_column): created 20gt after
  placing a magma block (the whirlpool/downward) or soul sand (the
  upward); destroyed 5gt after destroying the base (all the column's
  water blocks simultaneously); propagate only through SOURCE water
  (stopping at flowing water/waterlogged); the upward column drags
  entities up (11 b/s), the whirlpool down (4.9 b/s); air-providing (the
  drowning meter refills); the boat sink. The player wiring: the
  bubble_kind scan (the column's base block) + the smooth drag toward
  the verified terminal speed.
- **The gravity blocks**: sand/gravel (the block-wise fall — the
  documented progressive approximation of vanilla's falling-block
  ENTITY), the concrete powder's solidification (all 16 colors, VERIFIED
  w/Concrete_Powder).

## 5.6 The entities (vc-sim/src/entities.rs)

- **ItemSystem**: the dropped items — the gravity 0.04/drag 0.98, the
  point-collision (the rest band above the surface), the collect through
  the game layer (the 7.25-block attract is the XP orbs'), the 6000-tick
  (5-minute) despawn, the merge.
- **XpOrbSystem**: the drop splitting (the 3+1 two-orb rule), the 7.25
  attract distance, the 2-tick pickup gate (10/s cap), the 6000-tick
  despawn; the collected XP drains to the player's xp_points.
- **PrimedTntSystem**: `prime(wx, wy, wz, biome, sky, blk, fuse)` — the
  entity placed at the block + [0.5, 0, 0.5], the velocity 0.2 up +
  0.02 random horizontal; the fuse 80 (the fire/redstone) or the random
  10-30 (the chain-prime); the flash: alternating every 0.5 s (the
  completed-ticks fold — dark ages 1-10, bright 11-20); the hitbox 0.98
  BOTTOM-anchored on Y (the probe spans [target, target+0.98] — the
  centered X/Z ±0.49); the explosion at fuse 0: queued at pos +
  0.06125 above for game.rs's explode() to drain; the power 4.0.
- **The ignition_sweep** (per tick over the registered TNT placements,
  the sim ring): the redstone power check (the lamp's verified source
  set) OR the fire/lava contact (the cell or the 6 neighbors) → prime
  (the chain fuse 10-30).

## 5.7 The block entities + the world systems (vc-gameplay)

- **Furnaces** (the 200-tick cook = the vanilla 10 s, the fuel ladder),
  **Campfires** (the 4-slot fuel-less 600-tick), **Brewings** (the
  400-tick = 20 s), **Enchants** (the reactive): the map-keyed registries,
  the sim-rate ticks, the done/completed queues the game layer drains
  (the remesh + the sound).
- **Sleep/beds** (sleep.rs): the PURE decision layer —
  `attempt(day_time, weather, monster_near) -> SleepAttempt`
  (Sleep/RefuseNight/RefuseMonsters — the monster box FIRST, the daytime
  use still sets the spawn); `window_contains` (the clear 12523..23477,
  the rain 12002..23998, the thunder any time — the f32 day fraction
  SNAPPED to the tick so the inclusive bounds stay exact); the monster
  gate (the 8/5 box around the head); the game layer's use_bed() → the
  gates in order (the distance 3.0 → the monsters → the window), the
  skip (the sunrise + the rest reset + the weather reset), the spawn
  point = the head cell ("Respawn point set" only when the saved head
  CHANGES), the respawn arm (the bed absent → the world spawn + cleared;
  obstructed → the same; the candidate ladder: the side cells → above
  the head → above the foot).
- **The nether portal** (portal.rs): `find_frame` (the 2×3..21×21
  interior, the obsidian frame outline, the corners not required — the
  descend/slide/measure scan), `search_existing_portal` (the 128/16
  radii — the 1.16.2 20w28a Nether-side reduction; the SORTED chunk walk,
  the strict-< closest tracking), `find_build_spot` (pass 1: the 3×4
  buildable + 4 air above, the long axis matching; pass 2: the 1×4),
  `forced_y` (70..118 Nether / 70..246 Overworld — the engine's 256
  height adaptation), `portal_coords` (the 8:1 div_euclid — floor
  division, NOT truncation), `travel_wait_secs` (80 ticks survival /
  1 tick creative = 4.0/0.05 s).
- **Weather** (weather.rs): the rain/thunder machine (the vanilla
  cycles: the timers, the 20 s autosave interplay), the force_rain/
  force_clear/force_thunder (/weather stand-ins), `sleep_reset()` (the
  bed skip's caller — the flags reset, the timers keep), `sky_factor()`
  (1.0/12/15... the daylight-sensor factor), `can_strike()` (the
  thunderstorm + the 30 s cadence).
- **Villagers** (villagers.rs): the wander AI, the well-populated
  registry (never double-spawn), the tiered trade state (the trades
  executed since boot — stats/E2E), the zombie-villager cure path.
- **The dragon** (dragon.rs): the End fight — the phases + the timeline,
  the perch/dash cycles, the crystal healing, the acid breath.
- **Bees** (bees.rs): the hive registry + the work/release clocks, the
  stinger (one sting → the death timer), the pollination queues.

## 5.8 The redstone (vc-sim/src/redstone.rs)

The components: the wire (the power propagation), the torch (the
inversion), the lever/button, the repeater/comparator (the delay/signal
strength), the piston/sticky piston (+ the QUASI-CONNECTIVITY: the
qc_powered + the QC shadow scheduling — VERIFIED), the observer (the
rising-edge pulse), the dispenser/dropper (the rising-edge detection, the
eject one item per activation, 4gt later), the hopper (one item per 8gt
= 2.5/s), the pressure plates (the LIGHT/HEAVY weighted only — the
wooden/stone are missing), the target (the projectile-hit pulse decay,
8gt/20gt), the daylight sensor (the sky light × the day brightness × the
weather factor, the 20gt self-reschedule). TNT: the redstone mechanism
component (NOT conductive — "any adjacent TNT blocks are activated by
the explosion", VERIFIED w/TNT §Redstone component).

---

## PART 3 — THE RENDER PIPELINE, THE ASSETS, THE SAVE, THE E2E/CI, THE HISTORY

## 6.1 The greedy mesher (vc-mesh/src/mesh.rs)

`mesh_sections(pos, snapshot: &[Option<Arc<Chunk>>; 9], light, smooth,
mask, ..., ) -> MeshData` — the REFERENCE CPU path (the hybrid scope: the
chunks with cross plants / model states use ONLY this).

- **The 3×3 snapshot**: the center + the 8 neighbor chunks (the
  chunk-boundary faces read the neighbors — a face exists only when
  exactly one neighbouring bit is set).
- **The greedy key** (T1/T2 FIXED — the bit layout is era-locked):
  bits 0-4 the block light, 4-20 the sky_pack (the 4-corner sky), 20-28
  the ao_pack (the 4-corner AO), 28-44 the FULL u16 state (the 2026-10-02
  fix: the old 8-bit pack truncated every state ≥ 256 — basalt 738 → 222),
  44-52 the tint byte. The runs merge only on the EXACT key match — the
  tint/level/state all ride the key so runs never merge across them.
- **The fluid paths** (water/lava/bubble/waterlogged): the water-quad
  path — the level rides the key bits 19..21 (the runs never merge across
  levels); the fluid height = (8−l)/9 (the source slab 14/16); the step
  faces between different-height water cells: the TALLER side renders the
  shared vertical face (the vanilla behavior — the 2026-10-01 fix: the
  all-water side faces were culled, leaving see-through gaps); the lava:
  level-0 uniform, the same-cull against lava (the T4 fix — the double
  faces); the bubble column: the water path (the biome tint, uniform);
  the waterlogged container: BOTH the container's greedy faces AND the
  water overlay (level 0).
- **The cross plants** (flowers/grass/fire): the special CPU emit path
  (the two crossed quads, the fixed shade) — has_cross routes the chunk
  to the CPU fallback.
- **The face culling**: face_visible(block, neighbor) — the per-class
  rules (§1.3 vc-blocks): the water visible through non-water non-opaque
  (the bubble column IS water — culls); the lava culled against lava;
  the "fancy" leaves render against other leaves; the glass/ice/portal
  same-cull; the opaque hides everything.
- **The AO + the corner sky**: the smooth lighting (the 3-neighbor +
  diagonal solid checks per corner; the vanilla "Minimum" option lifts
  the corners half way — the (a+3)/2 remap, mirrored exactly in the
  WGSL).

## 6.2 The GPU compute mesher (vc-render/src/gpu_mesh.rs)

The greedy-eligible chunks go to the compute shader (the hybrid scope's
GPU half); the lost readbacks release the inflight markers → the §12
dirty bits trigger a CPU remesh (correctness never depends on the GPU
path).

- **The WGSL** (the compute shader, validated by naga in shader_tests +
  the runtime): `build_mask_cell` — one thread per cell, porting
  mesh_sections' mask build bit-for-bit (the sb/fl/tint_packed/
  face_visible/water_level_s mirrors); `job_getb/get_sky/get_blk` (the
  padded inputs' reads); `fluid_height_w(level, water_above)`.
- **The LUT**: build_lut() writes the shared buffer: L_SB (the state →
  block, 892), L_FL (the block flags — F_OPAQUE 1 / F_WATER 2 / F_LEAVES
  4 / F_GLASS 8 / F_ICE 16 / F_CROSS 32 / F_LAVA 64 / F_PORTAL 128),
  L_TC (the tint class), L_ST (the state tiles, 4/state). The WGSL's
  consts L_FL=892/L_TC=1431/L_ST=1970 + the clamps (fl/tint min(b,538),
  sb min(s,891)) are GUARDED by wgsl_lut_offsets_match_rust + the census
  tests (the drift fails the build, not a silent mislabel).
- **The CPU↔GPU parity tests**: gpu_mesh_parity_states_above_255 (the
  byte-identical greedy keys for the states ≥ 256), lut_mirrors_vc_blocks
  (every block's flags/tint/tiles + the face_visible via flags against
  vc-blocks' truth), mesh_compute_wgsl_validates (the naga parse +
  type-check).

## 6.3a The colorimetry cross-check (the visuals round, 2026-10-04 — PASSED)

The engine's tiles vs the vanilla 1.16.5 jar's textures MEASURED (the
STUDY-ONLY grant — the measured colorimetry, never copied; the pixel-
identity check stays):

| Tile | The engine's base | The vanilla measured avg | Drift |
|---|---|---|---|
| stone | (127,127,127) | (126,126,126) | 1 |
| sand | (219,207,163) | (219,207,163) | 0 — EXACT |
| planks | (162,131,79) | (162,131,79) | 0 — EXACT |
| log side | (107,83,51) | (109,85,51) | 2 |
| dirt | the shades' avg ~(128,92,64) | (134,96,67) | ~6 |
| grass top / water | GREYSCALE × the biome tint | GREYSCALE × the biome tint | the EXACT vanilla mechanism |

The tint values are vanilla-EXACT (the Plains grass 0x91BD59, the water
0x44AFF5 — the code cites the vanilla hex). The rendered-capture drift
(the grass (97,148,45) vs the tinted want (84,109,51)) is the LIGHT
contribution (the shade × the sky × the AO) — correct behavior, not a
tint error. The vanilla's grass_block_top.png/water_still.png being
GREYSCALE confirms the engine's tint-pack approach matches vanilla
exactly.

## 6.3 The assets (vc-render/src/textures/ + the audio)

- **The atlas**: 512×512, TILE_PX 16 — the 1024-tile grid (tile = ty*32+tx).
  Every tile is painted by a CLEAN-ROOM painter (the procedural
  synthesizers from measured values — the colorimetry, the dimensions,
  the animation timing): the terrain painters, the portal_art (the nether
  portal's vortex: the clean-room 4-frame shimmer, the pulse only on the
  bright swirl pixels, frame 0 == the base tile — the seamless loop), the
  tnt_art (the 3 faces 794..796), the weather_art, the farming_art, the
  v112..v116_art (the version brackets' blocks), the e2_art/e3_art, the
  r13_art, the gui_art (the 32 effect icons + the widget chrome + the HUD
  sprites).
- **The pack merge**: `merge_pack_textures(atlas, set, source)` — the
  USER-pack textures replace the built-ins on the LUT (the pack override);
  the animated strips: the cobblestone 4-frame shimmer (the pack's
  .mcmeta), the built-in magma (1.10), the built-in nether-portal vortex
  (the mesh census pins 3).
- **The audio**: the sound events (the family/volume/pitch) — the
  data-driven SoundRegistry (§21), the music pads (the 2.5-4 min cadence,
  the day/night progression), the underwater/rain loops, the ambient cave
  rolls, the per-mob sounds. 122 entries vs the ~700 events (the L-tier
  gap: ~580 more + the .ogg files + the discs/jukebox).

## 6.4 The performance (Part 1 re-measured — verdict: CPU-bound)

Phase-1 baselines (docs/BASELINE-PERF-2026-10-02.md): light-engine init
86.6 ms/chunk (reference tier) / 15.6 (GH runner); meshing 49.1/10.6;
generation 53.1/11.0; sim tick 0.041 ms; drawprep 13.9 µs; 10.3 KiB/chunk.
Part 1 streaming series (lavapipe walk, CI artifacts — runner numbers,
never reference numbers): avg 268.4 ms pre-budgets → 52–53 (1.3 work
budgets) → 83.2 (1.5) → 69.2 (1.6) → 78.0 (1.10 gate run); scene GPU
8.9/11.7/30.5 ms across runs (software-rasterizer variance is ~3x).
Headless bench: gen 11.44 → ~8.4–11.0 ms (inside the ±20% runner
variance — directionally faster, honestly not proven). Verdict: the
frame is CPU-bound by worldgen + lighting + mesh-apply on every
measured tier (exact/gameplay classes — no lawful GPU offload left;
see docs/OFFLOAD-AUDIT.md). Reference-hardware numbers come from the
owner's runs (1.10); the streaming artifact carries avg/p99/worst +
chunks + gpu_ms every run, gated structurally in CI.

## 7. The save format (vc-anvil + the web journal)

- **The native**: the 1.16.5 anvil region files (32×32 chunks per region,
  the NBT compound tags — the block states, the biomes IntArrays, the
  block entities), the level.dat (the seed, the spawn, the player). The
  autosave: 20 s cadence while in-world (the save_dirty drain).
- **The web**: the localStorage journal — the position → final STATE id
  of every landed block mutation, replayed after the chunk gen (the web
  build's substitute for region files).
- **The save_dirty tracking**: set_block_state marks the chunk; the
  newly generated chunks enter dirty too.

## 8. The cross-crate integration rules (the 2026-10 catches)

The rules that WILL bite a new engineer (each caught by a CI failure):
1. `Chunk::get` FOLDS to the owning block id — never
   `state_block(chunk.get(...))` (the double fold aliases the high
   states onto low ids — the portal search was DEAD because of this);
   `Section::get` returns the raw state — a fold is required there.
2. `World::set_block_state` stores the STATE raw — pass
   `default_state(BLOCK)` for any block whose state id ≥ 256 (the raw
   block id 536 folds to the glazed-terracotta class).
3. Every new BlockDef joins BLOCK_TABLE (the array size) — every new
   special state joins: state_block's fold, prop_states_roundtrip's
   audit chain, is_model_state's never-model window, the WGSL LUT
   (the consts + the clamps), the census pins (the STATE_COUNT/
   BLOCK_COUNT asserts — 11 pins updated for the fire ages alone).
4. The effect-icon/build guards: a new EffectKind without a matching
   icon/name/arm BREAKS the build (the declaration-order guards) — never
   a silent mislabel.
5. The PlayerHit-style struct changes touch EVERY construction site (the
   struct literals are exhaustive).
6. Test-only imports go INSIDE mod tests (the lib-only clippy build).
7. Doc comments: the list items after a bullet indent 3 spaces; a bare
   comparison line (`> 0 the player...`) reads as a quote marker — the
   lazy-continuation/empty-line lints fail the build.
8. clippy stops at the FIRST failing crate — after fixing one, expect
   possibly more.
9. f32 precision: the boundary roundtrips (12523/24000·24000 =
   12522.999...) — snap to the tick; epoch seconds overflow f32 (the
   24-bit mantissa) — use process uptime (the now_secs catch: the
   "stuck on loading >1 minute" was exactly this).
10. The mid-pass `queue.write_buffer` collapse: the writes apply before
    the whole command buffer executes — the multi-instance uniform
    rewrites collapse to the LAST value (the T5 root cause — the
    chunk-border overlay is one static grid + one uniform for this).
11. The F2-dumper yield: `take_screenshot()` in draw() consumes a waiting
    PNG only while every capture leg is idle (FKEYS stage 0/done,
    iconic disarmed, PHASES stage 0, turntable stage 0) — a leg that
    armed a readback but finds its PNG eaten deadlocks to its step
    timeout (caught twice: 1.1v, and the same class in 1A.6).
12. The E2E leg pattern: world-entry + chunk-ready + settle guards, a
    gated VERDICT line CI greps, pull-based PNG consume (arm one frame,
    take next update), exit ownership in BOTH smoke-exit guards, and no
    push atop a running slice CI (the concurrency group cancels it).
13. The phase contract `sum(phases) >= 0.9 × frame` holds three ways:
    the `phases_span_the_frame` unit test, `min_coverage()` for the
    runtime check, and the E2E_PHASES in-game verdict.
14. The streaming pool type is per-target (`StreamPool` = rayon pool
    natively, unit on wasm — rayon is a native-only dep, so no
    `rayon::` path may be named in shared code); the pool rebuilds
    live under the gen-threads knob.
15. Upscale input selection lives in `rebuild_post_targets` (EASU +
    composite inputs follow fsr_off/aa_on) and the constructor must
    agree with it — a Native-booted renderer whose composite binds the
    never-written upscale target renders a black 3D view (the 1.6 catch;
    UI-only legs still pass on it, only pixel contracts catch it).

## 9. The verification infrastructure (current)

## 9.1 The CI gates (ci.yml)

The 9 jobs (code jobs must be green — nothing is done until they are;
artifact uploads additionally need Actions storage quota, which
recalculates every 6-12h and fails uploads — never code — while
exhausted):
1. The legal audit (the trademark scanner) — every push.
2. cargo fmt --check (rustfmt; run `cargo fmt -p <crate>` locally).
3. cargo test --release --no-default-features --workspace (14 libraries +
   the app; counts come ONLY from summing that run's `test result:` lines).
4-6. The worldgen golden-hash gate on ubuntu + windows + macos (R5:
   bit-identical terrain on all three; the full suite stays ubuntu-only).
7. The wasm32 check (the headless-safe subset — --no-run, honestly named
   compile-check).
8. cargo clippy -- -D warnings (ALSA headers on the full-audio leg;
   lib-only — `clippy --workspace --all-targets` fails on pre-existing
   test-code lints, so match CI exactly).
9. The headless bench (the sim-tick section 200 ticks + the light-engine
   section — the glowstone-edit pump; uploads the JSON + the vc_bench
   binary artifact).

## 9.2 The in-game E2E legs (linux-game.yml — the owner's rule)

The single-file binary builds on CI → the Xvfb legs (1280×720×24, the
lavapipe/Vulkan backend) → the gated VERDICT lines:
- **E2E_FKEYS**: the 4-stage ladder — stage 0 waits for world entry + 10
  stable frames (the meshing burst settles); the keys (F1 HUD toggle, F5
  the perspective cycle 0→1→2→0 + the third-person rig from 4.0, F3+G the
  chunk borders chord, F11 fullscreen, F2 the capture arm); stage 1/2: the
  borders-ON/OFF capture pair — the T5 contract: the DIFFERENTIAL
  COLORIMETRY (the yellow-ish predicate r>60 && r>b+40 && g>b+40 &&
  |r-g|<40; ok = on_n > off_n + 300); stage 3: the FRONT view; the day
  clock freezes from the ladder arm through the verdict (the margin
  flake's fix). The capture PNGs persist to screenshots/.
- **E2E_BEDS**: the sleep gates in-game (all 7 flags: the monster/window/
  distance refusals, the night skip + the weather reset + the spawn set,
  the respawn arm both branches) — the leg waits for world entry + the
  chunks it touches (the unguarded first-update run placed no bed and
  failed 5 of 7 flags silently).
- **E2E_FLUIDS**: the 4 scenarios through the REAL sim ticks (the bench
  pump pattern: sim.step per tick) — the infinite water source, the
  lava/water mixing product (obsidian), the bubble column, the
  waterlogged chest.
- **E2E_CONTAINERS**: the container screens (the chest/furnace opens, the
  9-slice panel on the GPU quad layer, the canvas dumps).
- **E2E_MENU**: the settings-tree click script through every page and back.
- **E2E_PHASES** (1.1v): the runtime phase-meter proof — min over the live
  ring of sum(phases)/frame ≥ 0.9 with a VERDICT line, then the F3 overlay
  (with the phase row) captured to screenshots/.
- **E2E_TURNTABLE** (1.11): one subject mob alone on flat grass (TURNTABLE_MOB,
  default creeper), HUD hidden, neutral daylight, natural spawning off —
  front/side/back/three-quarter/close-up PNGs to screenshots/ (CI artifacts;
  the PNGs are the owner's art verdicts under V1).
- **Streaming bench** (1.2v, non-gating): `--benchmark streaming` walks fresh
  terrain; the JSON artifact carries avg/p99/worst + chunks + gpu_ms, and
  the 1.10 structural gate greps it (fields/frames/camera present, never
  lavapipe timing thresholds).
- The smoke legs: the boot (intro → panorama title → world entry through
  the real pipeline) + the exit contract (the exit holds cover every
  stage).

## 9.3 The raw stream (--verbose)

THE flag: the full raw diagnostic stream ([t+SSS.s][cat] lines to the
terminal + logs/latest.log). The categories: **input** (every event: the
hover changes, the menu clicks — the widget id/kind/label + the hover/hit
cross-check, the unhandled-id WARNs, the screen routing, the menu-
activation traces, the no-sound cases); **gfx** (the boot/surface/
gui-scale/canvas geometry, the per-screen widget-table dumps, the
duplicate-id + overlap WARNs, the 1 Hz heartbeat — runs in EVERY screen);
**sim** (the 1 Hz internals heartbeat: the sim ticks, the scheduler
depth, the entity counts items/orbs/tnt/furnaces/campfires/brewing/
villagers, the player's environment flags in-water/bubble/portal/lava,
the day time, the weather); **screen** (every transition); **perf/world/
save/f3** (the steady-state stream). The wasm: ?verbose (+ ?debug as the
legacy alias).

## 9.4 The known E2E flakiness + the fixes (do not re-learn these)

- The smoke-exit race: the exit holds cover EVERY stage (`< 4`), both the
  immediate and the delayed exit paths.
- The byte-diff between live frames is meaningless (the day-light, the
  clouds, the meshing burst change every frame) → the differential
  colorimetry is the border contract.
- The colorimetry's margin flake (834 then 51 on the same binary) → the
  day clock freezes through the capture pair (4e56285). If it flakes
  again: freeze the weather too.
- The unguarded E2E legs: World edits NO-OP on missing chunks → the
  world-entry + chunk-ready guards (62a6d73) + the CI-grepped VERDICT
  lines (8a6af38).
- The xvfb-run in a pipeline: grep the verdict lines, never the pipeline
  exit code (tail/grep mask it).
- Never push atop a running slice CI (the concurrency group cancels the
  older run); dispatch a workflow once, then wait.
- The stream teardown ("Stream ended without finish_reason"): the route
  tears a long stream down — small commits survive (one giant commit
  loses everything if the turn is cut).
- `--benchmark` auto-pauses on headless focus loss: the bench holds Game
  like the capture ladders, or `bs.seen` never advances (the 1.2v stall).
- Lavapipe streaming runs at ~0.5–2 s/frame: size CI bench windows to it
  (150 frames fit; 300 do not).
- Actions artifact quota is finite: prune to the newest 2 per name when
  uploads fail with quota errors; close-outs wait for the recalculation.

## 10. The legal policy (docs/LEGAL-COMPLIANCE.md — the binding rules)

- The owner's directive (the top of the hierarchy): never use copyrighted
  reference material directly — never copy, never trace, never
  modify-and-ship; study the idea, author the expression.
- The enforcement: scripts/legal_audit.py (the trademark/copyright
  scanner) on every push; the pixel-identity check before any asset
  release (the reference sets stay OUTSIDE the repo).
- The study-only grants (§6a): the versions/ jars (the vanilla + OptiFine
  1.16.5 — the jar-derived counts/blockstates/loot tables/biome ids), the
  mods/1.16.5/ set (Sodium etc. — the optimization TECHNIQUES studied,
  never the code), the real saves (the biome-id ground truth).
- The trademark terms never appear in any shipped file (the paths are
  described, not quoted — the audit flags them).
- The ecosystem interop (read-side only): the user-supplied packs/saves/
  shader packs load; never bundled; the compatibility statements are
  nominative fair use.
- Game mechanics/data are facts — free to replicate from published
  documentation. Original art/code are the project's (per the licensing
  decision below); the ACTIVE engine font is third-party Monocraft
  (IdreesInc, SIL OFL 1.1, shipped with its licence), the Voxelfont
  spare is ours (MIT).
- Licensing (owner-approved, lands after slice 1.12): code + art-generator
  scripts GPL-3.0-or-later; original art + aggregate spec CC BY-SA 4.0;
  Monocraft stays OFL 1.1; Voxelfont stays MIT; no custom GPLv3 §7 terms,
  no dual licensing. Slices L1–L8 (license texts, REUSE, CI lint, NOTICE/
  TRADEMARKS/CONTRIBUTING, About screen, AUTHORSHIP, provider-terms
  finding, doc updates) — tracked in AGENT-STATE.md.
- The repo is PRIVATE (owner order). Owner hardware specs never appear
  anywhere (AGENTS.md hardware-privacy rule); the study-corpus
  provenance statements stay visible in LEGAL.md/docs forever (R4).

## 11. The known gaps + the disclosed trims (current, October 2026)

- **Landed since**: the 16 missing mobs (66 declared), gamerules live,
  smoker/blast/dispenser/dropper GUIs, subtitles, the F3 phase + CPU/GPU
  rows, the upscale ladder + Sharpness + FXAA + Native skip, GPU
  timestamps, the capability probe + gen-threads knob, the streaming
  bench + perf gate, the turntable tool.
- **The L-tier gaps**: raids/advancements/slash commands/scoreboard;
  the 11 missing structures; entity-model loading for the billboard
  kinds; the particles (20/90) + the sounds (122/~700) breadth; A*
  pathfinding; the stonecutter/smithing/special recipes + the recipe book;
  the tool/weapon registry; vanilla world import (Part 2).
- **Open engine findings**: EMERALD_BLOCK absent (T12); vanilla
  shade:false unimplemented (T13); the Nether lava sea; Nether/End death
  travel; InhabitedTime stub; bits_for 8-bit cap; SMAA-vs-FXAA 3 ms call
  (soft blocker — needs reference-hardware numbers).
- **The engine findings (open)**: EMERALD_BLOCK absent from the registry
  (T12); the vanilla shade:false unimplemented (T13 — CompiledFace.shade
  is dead data); the Nether lava sea; death in the Nether/End does not
  travel back to the Overworld; InhabitedTime stub; bits_for 8-bit cap;
  the world_source_conversion gamerule; the water_source_conversion's
  opposite (the lava never spreads sources — verified).
- **The documented trims** (each disclosed in-code): the random-tick
  density (3/chunk vs 3/section); the slab/stairs/fence waterlogging;
  the splash AoE (the witch's throw hits the player directly); the
  witch's drinkable-potion defense; the missing flammable blocks (the
  slabs/fences/lectern/scaffolding/bamboo/cave-vines/dripleaf sets); the
  vanilla 101-tick sleep animation; the drop-down respawn scan; the
  resurrect-order facing; the horse armor items; the bubble columns' air
  particles; the mobs' knockback (the damage-only blast); the Looting
  enchant unwired; the portal vortex frametime (the clean-room 4-frame);
  the Overworld forced-Y ceiling (246 = 256−10); the Levitation float
  (0.9 × level — the documented approximation); the Dolphin's Grace ×2.

## 12. The history (the decision record, updated in place)

- **The pre-overhaul sweep** (2026-10-01): 4 review agents, ~139k lines
  read, 2 critical (the greedy key's 8-bit state truncation + the tint
  overlap) + 4 major + 45 minor findings — ALL fixed since (the T-round
  burn-down: the key widened to the full u16 state at 28..44, the tint at
  44..52, the WGSL mirror bit-identical, the lava T4, the T5 collapse,
  the T6 camera, ...); docs/PRE-OVERHAUL-SWEEP-2026-10-01.md §8 is the
  dated addendum.
- **Phase 1** (the measure): the baselines committed (the #1 cost = the
  light-engine init); the bench gained the sim-tick + the light-engine
  sections; the regression check live via the bench job.
- **Phase 2** (the audit): the 11-domain matrix + the gap list + the
  round order; the jar-derived reference counts; the ground-truth biome
  ids from the real saves (the audit's WRONG row resolved).
- **Phase 3** (in progress): Round T → the portal (Round K) → TNT →
  beds → fluids → fire → sneak/effects → witch (the attack landed, the
  skull drop pending) → **THE VISUALS UPDATE (re-sequenced EARLIER by the
  owner's 2026-10-04 directive — the GUI batch + the 3-G UI/feel round
  come NEXT, before the mobs/systems/structures batches; nothing breaks —
  the visuals work on the render layer and unblocks the mobs'
  entity-model rendering)** → the mobs batch → the systems → the
  structures → the particles/sounds → the breadth → the polish.
- **Phase 4** (designed, not started): the light-engine optimization
  (docs/PHASE4-LIGHT-DESIGN.md).
- **Phase 5 (the hardening, build-stage)**: NO RELEASES — the owner's
  2026-10-05 directive: the project is in the BUILDING stage, not
  production; all 3 releases (v0.2.0/v0.3.0/v0.4.0) and their tags were
  DELETED (the releases' binaries/assets removed from GitHub; the tags
  pushed down). The tag/release step is DEFERRED to production.
- **PLAN v3.1 Part 1** (2026-10-08, this era — full record in
  docs/WORKLOG.md): 1.0 determinism baseline (libm everywhere, 3-OS
  golden gate, f32 census) → 1.0.5 per-function libm pins + wide/
  targeted hashes → 1.1v phase-meter proof (unit + in-game VERDICT +
  F3 phase row, V1-viewed) → 1.2v streaming bench + CI step (bench
  auto-pause stall fixed) → 1.3 work budgets (dedicated pool, 6 ms
  apply cap, stale-mesh drop; FIFO mesh starvation caught by the new
  metric) → 1.4 cell-major fill (identical output, −4% inside noise) →
  1.5 GPU timestamps (scene 8.9 ms lavapipe) → 1.6 upscale ladder +
  Sharpness (Native black-screen catch) → 1.7 FXAA (SMAA decision
  parked) → 1.8 probe + F3 split + gen knob → 1.9 offload audit
  (CPU-bound verdict) → 1.10 perf gate → 1.11 turntable tool.
- ** Privacy + repo ops** (2026-10-08): repo PUBLIC → PRIVATE; owner
  hardware specs scrubbed from the tree AND (on explicit order) from
  all pushed history via filter-repo (verified zero hits; bundle-kept
  safety copy); stale local tags dropped; workspace junk cleared
  (10 GB target/, session artifacts); no-push-atop-CI and
  parallel-write-race scars logged as rules.

## 13. The worklog relationship

Root `worklog.md` is the PRIOR era's closed log (Tasks 1–25, through
2026-10-05) — read-only, never append. `docs/WORKLOG.md` is the ACTIVE
session log (this era's dated entries) — append every slice. THIS file
(REPO-KNOWLEDGE-BASE.md) is the always-current knowledge: when a fact
changes, edit the section here; the worklog records THAT the change
happened. Together they are the full record: the worklog = the
chronology, this file = the current truth. Session state (next slices,
blockers) lives in `docs/AGENT-STATE.md`; Part reviews in
`docs/CHECKPOINTS.md`; soft/hard blocks in `docs/BLOCKERS.md`.

---

## PART 4 — THE FULL TABLES (the verified content, current)

## 14.1 The 32 status effects (the COMPLETE 1.16.5 set, java_id order)

| id | Kind | The verified behavior (w/Effect + each page, live 2026-10-03) |
|---|---|---|
| 1 | Speed | the movement +20%/level; the wired multiplier |
| 2 | Slowness | the movement −15%/level; the wired multiplier; the witch's Slowness splash I = 1:30 (1800t) at 8-10 blocks |
| 3 | Haste | the mining speed +45%/level (the wired hook) |
| 4 | Mining Fatigue | the mining ×0.3^min(level,4) (III = the 97.3% decrease); the attack speed −10%/level; the elder guardian's default III |
| 5 | Strength | the melee +3 HP/level (the wired bonus) |
| 6 | Instant Health | instantly heals 2 HP × 2^(amplifier+1) (I = 4 HP, II = 8); the undead take it as Instant Damage |
| 7 | Instant Damage | instantly 3 HP × 2^(amplifier+1) (I = 6 HP, II = 12) MAGIC (only Resistance/Protection decrease); the undead heal |
| 8 | Jump Boost | +0.1/level the jump velocity (the wired bonus) |
| 9 | Nausea | the wobbly view (the render layer) |
| 10 | Regeneration | heals 1 HP per 50>>level ticks |
| 11 | Resistance | −20%/level ALL damage (the wired multiplier) |
| 12 | Fire Resistance | the fire/lava damage NEGATED (the wired gate) |
| 13 | Water Breathing | the air meter frozen (the wired hook) |
| 14 | Invisibility | the model hidden (the render layer) |
| 15 | Blindness | the fog in + the sprint/crit disabled |
| 16 | Night Vision | the brightness (the render layer) |
| 17 | Hunger | the exhaustion +0.005/tick/level |
| 18 | Weakness | the melee −4 HP × level; the ≤ 0 attack fails to connect (no knockback); the witch's Weakness splash 1:30 at < 3 blocks + the health/poison gates |
| 19 | Poison | 1 HP per 25>>level ticks (the 10-tick immunity floor), CANNOT kill; the undead immune |
| 20 | Wither | 1 HP per 40>>level ticks, CAN kill; the wither-skeleton melee inflicts I 10 s; the skull hits inflict II 200/800 |
| 21 | Health Boost | +4 HP (2 hearts)/level the max health (the wired hook) |
| 22 | Absorption | the damage buffer 4 points/level (the player struct, cleared on expiry) |
| 23 | Saturation | 1 hunger + 2 saturation per tick per level (the wired restore); the particle #F82421 red |
| 24 | Glowing (Java-only) | the outline (the render layer) |
| 25 | Levitation | the float up 0.9 × level b/s (DOCUMENTED APPROXIMATION — the wiki publishes no scalar); the shulker bullet |
| 26 | Luck (Java-only) | the loot modifier (the disclosed hook) |
| 27 | Bad Luck (Java-only) | the negative loot modifier (the disclosed hook) |
| 28 | Slow Falling | the gravity clamps at −9.8 b/s (the terminal velocity), the fall damage negated |
| 29 | Conduit Power | the Water Breathing + Night Vision + Haste benefits (the air gate + the mining hook) |
| 30 | Dolphin's Grace | the swim ×2 (the documented approximation) |
| 31 | Bad Omen | the raid trigger (VERIFIED w/Raid; the raids are MISSING — the disclosed hook) |
| 32 | Hero of the Village | the trade discounts (the disclosed hook) |

The immune sets (VERIFIED w/Effect §Immunity): the undead immune to
Regeneration/Poison; the wither-skeletons immune to Wither too; the
spiders/cave-spiders immune to Poison; the ender dragon/the wither immune
to ALL; the witches take 85% less EFFECT damage (JE).

## 14.2 The 66 MobKinds (current — the 1.16.5 set COMPLETE)

66 declared variants (65 `MOB_DATA` rows — Squid is a classification-only
stub that never spawns). 12 jointed rigs (player/cow/pig/sheep/horse/
donkey/mule + families, walk gait + hurt recoil); the rest are billboard
sprites. The turntable leg (E2E_TURNTABLE) captures any mob solo for the
owner's art verdicts.
PARTIAL behaviors (AI halves/drop-table edges vary per kind — see the
per-kind tests). MISSING as full vanilla AI: raids, A* navigation.
(The exact per-kind rows live in mobs.rs's def() — the health/damage/
speed/hitbox/sound table, every row live-verified.)

## 14.3 The 28 biomes (of the 1.16.5's 66)

Plains (1), Sunflower Plains (129 — the ground truth), Forest, Birch
Forest, Dark Forest, Taiga, Snowy Taiga, Savanna, Desert, Badlands,
Jungle (10), Swamp (12), Swamp Hills, Mountains, Snowy Mountains
(Ice Spikes 140 — verified), Snowy Plains (Snowy), Frozen Ocean, Frozen
River, Beach, River, Ocean (the variants), Mushroom Fields, Deep Ocean,
Warm/Lukewarm Ocean, Stone Shore, Wooded Mountains, Gravelly Mountains
+ the Mountain variants. The nether: Nether Wastes, Soul Sand Valley,
Crimson Forest, Warped Forest, Basalt Deltas (the ids 170-173 match the
1.16.5 registry). MISSING (38/66): the wooded-badlands/eroded/jungle-
edge/bamboo/mountains-edge variants + the End's islands + the rare
oceans. (The Biome enum: gen.rs — from_u8 is the 1.16.5 registry order;
the precipitation(): the dry set (Desert/Savanna/Badlands), the snow set
(Snowy/IceSpikes/FrozenOcean), the nether/end never see weather.)

## 14.4 The crafting/recipes (craft.rs + datapack.rs)

1120 recipe entries vs the 859 jar JSONs (the jar-derived counts: 491
shaped + 143 shapeless + 13 special + 53 smelting + 11 blasting +
9 smoking + 9 campfire + 121 stonecutter + 9 smithing). The engine's
matchers: match_kitchen (the food), the smelting/blasting/smoking (the
200-tick/100-tick/100-tick + the fuel ladder), the shaped/shapeless (the
grid matching), the stonecutter (the 1:1 trims), the datapack's
set_count/loot. The E2E: the audit16 leg (the smelt/smoker/kitchen/
purpur/trio/food rows). The recipe book: MISSING.

## 14.5 The F3 debug screen (render.rs's f3 category)

The two-column layout (the F3 toggle + F3+G the chunk borders): the left
column (the fps envelope, the INTEGRATED phase row `sim/stream/results/
ui/draw [cpu]` (1.1v), the position, the chunk, the facing, the
biome, the day time, the light); the right column (the memory, the
allocated — the counting allocator, the CPU model, the CPU/GPU ms split
(1.8 — GPU from the 1.5 timestamps, "n/a" when off), the display, the GPU). The
Targeted Block/Fluid line: the bottom-left (the crosshair's block +
the break progress). The reduced_debug_info hides the detail rows. The
1.16.5 rows: the wiki's Debug screen table (the verified anchors).

## 14.6 The atlas painters (the clean-room art vocabulary)

The `put(a, t, x, y, r, g, b, al)` scalar vocabulary (8 params — 300+
call sites, silenced deliberately): tile = ty*32+tx, the 16×16 cell, the
clamped RGBA write. The painters per module: the terrain (the grass/
stone/dirt/sand...), the ores (the spots), the woods (the bark/rings),
the leaves (the alpha holes), the portal_art (the vortex: the deep
violet field + the bright swirl pixels r>120 pulsed 0→peak→0 over the 4
frames), the tnt_art (the red stick-bundle field, the end-knots, the
plain underside), the weather_art (the rain/thunder), the farming_art
(the crops' growth stages), the gui_art (the effect icons: the 9×9
O/F/A masks + the palettes; the widget chrome: the 9-slice buttons with
the 1-px black frame + the light/shade rows + the ±4 noise band; the
HUD sprites: the hearts/armor/bubbles). The deterministic generators
(the python scripts in scripts/) for the repetitive output.

## 14.7 The slash commands (the stand-ins, current)

- `/weather <clear|rain|thunder> [seconds]` → weather.rs's
  force_clear/force_rain/force_thunder (the locked-duration timers).
- The other commands (the /effect, /give, /tp, /gamerule, /setblock,
  /fill, /summon, /scoreboard): the engine has the FUNCTION (the effect
  apply, the gamerule flags, the setblock/fill via the world edits, the
  summon via the mob spawns) but no SLASH-COMMAND PARSER — the systems
  round's gap (the commands are the L-tier gap list's row).
- The gamerules: the live Gamerules struct (doFireTick gates the fire
  tick, doDaylightCycle freezes the day clock, doWeatherCycle,
  naturalRegeneration gates hunger regen, keepInventory keeps death drops,
  mobGriefing gates mob-explosion terrain, doMobSpawning gates natural
  spawns) + the set_gamerule stand-in (slash parser = Part 3).

## 14.8 The block-entity containers (the GUI screens, current)

The chest + furnace + smoker + blast + dispenser/dropper screens
(E2E_CONTAINERS verified): the 9-slice
panel (the vanilla-grey), the slots (the 18×18 in the 20×20 cells with
the 1-px buffer), the drag/click routing, the shift-click. The brewing
stand + enchanting table: the reactive screens (the enchant list, the
potion slots). MISSING: stonecutter/loom/smithing/
cartography/lectern GUIs (the blocks-breadth round's row).

## 15. The chronology (superseded — see docs/WORKLOG.md)

The Tasks 1..14 table below is the pre-rewrite era (kept for the old
hashes; messages were de-specced in the 2026-10-08 rewrite). The live
chronology is `docs/WORKLOG.md` (this era's dated entries) and the todo
state is `docs/AGENT-STATE.md`.

| Task | What | The key commits |
|---|---|---|
| 1-3 | The pre-overhaul sweep + the report; the CI setup; the clippy burn-down | the 6-gate ci.yml |
| 4-5 | The internal game check + the baselines | the bench's sim/light sections; docs/BASELINE-PERF |
| 6 | The parity matrix (11 domains) | docs/PARITY-MATRIX-1.16.5 |
| 7-8 | Phase 0/1/2 records | the sweep's §8 addendum |
| 9 | The Round K + TNT + beds boundary (10 CI-failure fix loop) | 2dd2cd8, cdfee8c, 640b1e8, e616cca, b384f50, 933e861, d0350aa, 54b1d24, 22bb437, d276074, 3c7b786, 52769b4, 7ffe322, 8b4face, d605f0e |
| 10 | The in-game verification ledger + the E2E flake fixes | docs/INGAME-E2E-LEDGER; 62a6d73, 8a6af38, fddcd26, 4e56285 |
| 11 | The fluids round (the takeover) + the double-fold class fixed | 564ab2c, fb1f6f0, 9af843f, bbd1e08 + fc3f646, 780f7a6, 8b3c120, 14cff45, e4ef744, 530a752, 7a59a18 |
| 12 | The --verbose flag + the E2E_FLUIDS leg | 3a0eaf1, fdd68ab, c7f0dda, e739a6a, 1fdbd4d |
| 13 | The fire round | 3128f36, bde9c25 + 42d4a6e, b905d2d, 0654d26, 6316b8d, 559f2d0, d9de67b, 0c66f7f, bdfd246 |
| 14 | The 16 effects + the icons + the sneak caps + the witch attack | 99ce7ca, 1b44253, cc34715, bd99cfd, 44118d6, 075c582, ea50aa5, 7fbb915, a69f0b1, 7425296 |

## 16. The recovery procedures

- **The lost workspace**: the repo's state is always recoverable from the
  GitHub remote. History rewrites and force-pushes happen ONLY on the
  owner's explicit confirmation of that exact operation (R1) — with a
  bundle backup kept until post-push CI is green, and a verification
  pass (messages/contents/filenames) before pushing.
- **The corrupted commit**: the pre-corruption state is in the git
  history — `git show <good>:<path> > <path>` restores the file; the
  re-apply of the intended fixes PROPERLY (the a69f0b1 corruption: the
  script deleted the arg lines — restored from 7fbb915).
- **The stale info**: everything logged must be code-read/verified THIS
  era — the old docs (docs/research/) describe old repo states and
  contain errors in both directions; never take a number from them
  without re-verifying live (the reference wiki via the MediaWiki API).
- **The quota/harness failures**: the /tmp disk quota kills the artifact
  downloads + the harness's own writes — clean the /tmp downloads (the
  owner's say-so), then retry.

## 17. The glossary (the engine's vocabulary)

- **Section**: a 16×16×16 sub-cube of a chunk (16 per chunk vertically);
  the mesh/dirty granularity; the None section = all-air.
- **The greedy key**: the packed bitfield that merges same-face runs —
  the runs merge only on the EXACT key match (the state/level/tint all
  ride it).
- **The fluid height**: (8−l)/9 — the rendered water surface height per
  level (the source slab 14/16); the step faces between different
  heights.
- **The §12 region**: the block-edit invalidation (the conservative
  section set + the light region).
- **The COW**: copy-on-write (the Arc make_mut) — the chunk/section
  clones only when shared with in-flight jobs.
- **The anchor**: the player position/health/invulnerable the game layer
  feeds the sim before the tick.
- **The VERDICT line**: the E2E leg's gated boot line ("e2e: X VERDICT
  OK") — CI greps it; a FAILED verdict fails the leg.
- **The census pin**: the test-asserted count that regenerates when
  content is added (the STATE_COUNT/BLOCK_COUNT/tab/icon pins).
- **The hybrid scope**: the greedy-eligible chunks → the GPU compute;
  the cross/model chunks → the CPU mesh.
- **The sim ring**: the TickScope's simulation-distance circle — the
  ticks freeze outside it.
- **The phase meter**: `begin_frame` (AboutToWait) → `end_frame` (end of
  draw); `min_coverage()` = min over the ring of sum(phases)/frame.
- **The streaming pool**: the dedicated cores-1 rayon pool (native;
  unit on wasm), rebuilt live under the gen-threads knob; `pending_apply`
  holds results past the 6 ms apply budget; mesh applies go first.
- **fsr_off / aa_on**: Native mode (EASU skipped, composite reads scene)
  / effective FXAA (mode or forced below 1.0 render scale).
- **HwTier**: Low/Medium/High from cores + software-GL detection; the
  boot log carries the recommended defaults (1.8b applies them).
- **The turntable**: the solo-mob 5-view capture leg (E2E_TURNTABLE).
- **The perf gate**: the structural CI check on streaming-bench.json
  (fields/frames/camera; never lavapipe timing thresholds).

## 18. How to add a feature (the proven recipe, in order)

1. **Verify live**: read the reference wiki page(s) via the MediaWiki API
   (raw wikitext) — every constant needs a live citation; cross-check the
   ambiguous wording against a second source (another wiki section/the
   real game's jar data/the Java behavior).
2. **Read the engine**: the crates the feature touches; the integration
   rules (§8); the era-locked constants (§3).
3. **Implement**: the data first (the registry windows/folds/tables),
   then the logic (the tick functions), then the wiring (the game layer),
   then the art (the clean-room painters) — each in its own commit
   (≤~300 lines; never parallel-write one file from two tool calls).
4. **Test**: the unit tests next to the feature (the verified anchors);
   the census pins update; the E2E leg if player-visible (its own
   linux-game run at that commit — never pile untested work forward).
5. **Gate**: cargo fmt → commit → push (never atop a running slice CI) →
   the 9 CI jobs → fix from the logs → ALL GREEN → the linux-game run
   for runtime behavior → V1 screenshots viewed → worklog + state.
6. **Verify in-game**: the Xvfb legs → the gated
   VERDICTs + the vision check for the pixel work (foreground
   xvfb-run + MangoHud only when CI can't answer).
7. **Log**: the worklog entry + THIS file's section update + AGENT-STATE.

## 19. The operating rules (PLAN-FINAL §0 + AGENTS.md, condensed)

Autonomy A1–A8 + continuation rule: one slice/commit, local
`cargo check/test -p <crate>` only on an idle machine (heavy work is
CI-only), commit→push→wait CI→green→worklog+state→next slice
immediately; reports live in files, never as chat closings. Turn ends
ONLY on a HARD STOP, a Part REVIEW PACKET, a STOP file, or a
rate/context limit. L1–L8, V1, R1, R3–R7, P1 always in force. Key docs:
PLAN-FINAL.md (the plan), AGENT-STATE.md (todo), BLOCKERS.md (soft/hard
blocks), CHECKPOINTS.md (Part reviews), OFFLOAD-AUDIT.md, F32-AUDIT-GEN.md.
