# VoxelCraft — Full-Repo Test Sweep Report (2026-09-25)

**Branch:** `test/full-sweep-2026-09-25` (main untouched) · **Engine under test:** HEAD `a094987` (includes the Sep 23 HUD fixes)
**Rules honored:** zero local Rust compilation; every compile ran in GitHub Actions; all binaries/bundles are official CI artifacts, tested locally under Xvfb / Chromium.

---

## 1. CI verdict — all three workflows GREEN at HEAD

| Workflow | Run | Result | What it proves |
|---|---|---|---|
| `ci.yml` | 36049727154 | ✅ success | **849 tests passed, 0 failed** across the 15-crate workspace (native + wasm32 compile-check + headless benchmark) |
| `linux-game.yml` | 36049728309 | ✅ success | Single-file binary builds, embedded pack verified, full smoke suite green on the runner (boot → world → 1.14/1.15/1.16 E2E → settings tree → F3 liveness → first-run profile) |
| `wasm-build.yml` | 36049732416 | ✅ success | Fresh HEAD wasm pair + builtin pack built and uploaded |

**Context that matters:** every push to `main` since Sep 20 was RED. The sweep found **three** root causes — none of them engine bugs:

1. `chorus: command not found` — a sandbox shell helper leaked into all 4 workflow YAMLs (27 call sites); builds died at the first echo-step **after** a fully successful Rust compile. → replaced with `echo`.
2. `vc-render` lib-test E0425 on wasm32: `mod folder_tests` used the native-only `scan_pack_files` un-gated (same class as the Sep 19 "ravine" gate). → `#[cfg(all(test, not(target_arch = "wasm32")))]`.
3. `voxelcraft` lib-test E0433/E0425 on wasm32: six pointer-watchdog tests referenced the native-only `PointerLockMode` / `PointerEnvOverride` / `should_demote_to_delta`. → per-test `#[cfg(not(target_arch = "wasm32"))]`.

Plus one stale CI assertion fixed: the first-run profile check demanded `builtin-pack/pack.mcmeta`, but the pack descriptor migrated to the clean-room `pack.json` — the engine's extraction was correct all along (16/16 files).

**Commits:** `537d614` (workflows + shaderpack gate), `a094987` (pointer-watchdog gates + pack.mcmeta→pack.json).

---

## 2. Native binary — full CI smoke suite, re-run locally on the HEAD artifact

Binary: `voxelcraft-main-a0949875eca7-linux-x64` (11.9 MB, ELF x86-64, pack embedded). Runs under `xvfb-run -s "-screen 0 1280x720x24"`, `WGPU_BACKEND=Vulkan` (lavapipe software).

### Run 1 — `--smoke --debug` + `E2E_V114=1 E2E_V115=1 E2E_V116=1` → **exit 0**
Every contract line verified against the CI grep set:
- Boot: first-run extraction (16-pack + 53 classic-art files + options.txt) → intro complete 2.60 s → `intro -> title`
- Menu path via the real input route: title → worldselect → create → loading → `loading complete: 44 chunks on GPU in 1.0s` → game
- `e2e: v114 campfire lit+fed=true … cooked 1 item(s) after 650 ticks` (600-tick cook contract)
- `e2e: v114b blast furnace out=1 lit=true (100-tick 2x cook), smoker out=1 … lantern … popped on support break=true`
- `e2e: v114c flowers … f3-corn="Cornflower" f3-lily="Lily of the Valley" blue-dye=true white-dye=true instant-break=true`
- `e2e: v115 hive level0=0 level5=5 full=true … lifecycle=true … campfire-pacify=true`
- `e2e: v116 anchor=true(ladder=true charge=4 drain=true) … spiritfire-dmg=2.0`
- `e2e: v116b family=true lantern-pair=true strider-lava=true hoglin-flee=true barter=true`
- `audit16` ×2 present; pointer ladder (`pointer: confined…`), `[perf] fps`, `[screen]` transitions, `[exit] uptime 5s … 51 edits`

### Run 2 — `E2E_MENU=1` settings-tree E2E → **exit 0**
Full round trip through the REAL click path: `options → video → shaders → video → engine → packs → access → title`, `e2e: settings tree ok`.

### Run 3 — F3 liveness + first-run profile → **exit 0**
- `F3_DUMP`/`F3_DUMP2` written; `cmp` confirms the two dumps **differ** (overlay is live: fps/XYZ/memory move)
- First-run materialization ✓: `builtin-pack/pack.json`, `options.txt`, `logs/latest.log`, `saves/`, `resourcepacks/`, `shader-packs/`, `builtin-packs/classic-art/`

Artifacts archived: `ingame-test/native/{smoke.log, menu.log, f3.log, f3a.png, f3b.png}`.

---

## 3. Browser E2E — HEAD wasm bundle, live clicks/keys

Served via the Next.js wrapper (`bun run dev`, port 3000; `tsc --noEmit` clean except 2 pre-existing errors in the unused `examples/websocket` scaffold). Bundle: the fresh HEAD `voxelcraft.js` + `voxelcraft_bg.wasm` (locked pair, matching mtimes) + `voxelcraft-pack/`.

### Verified live
| Area | Result |
|---|---|
| Boot → Intro → Title | ✅ WebGL2 fallback engaged cleanly (adapter lacks compute — expected headless); panorama + pixel logo + button stack |
| SINGLEPLAYER → Select World | ✅ search box, action bar (both worlds persisted in localStorage with correct modes) |
| CREATE NEW WORLD, both pages | ✅ name/seed (page 1), world type + structures + bonus chest (page 2, "DONE..." toggles back) — vanilla two-page flow |
| GAME MODE toggle | ✅ Survival world created, then Creative world |
| Loading gate → gameplay | ✅ `loading complete: 169 chunks`, world interactive |
| HUD | ✅ hotbar + selection frame, hearts/hunger rows, XP bar (creative), crosshair, first-person view-model visible |
| F3 overlay | ✅ two-column vanilla layout: XYZ 3-dec, Block/Chunk, Facing+yaw/pitch, Client/Server Light, Biome (`voxelcraft:plains`/`voxelcraft:beach` — own namespace), Local Difficulty, chunk caches, GPU line, **Targeted Block decoding live** (`voxelcraft:grass_block`) |
| Help (H) | ✅ full two-column controls card |
| Inventory (E) | ✅ creative picker: icon grid, tabs, hotbar row + trash slot |
| Pause (Esc) | ✅ game↔pause toggle |
| Render distance `[` `]` | ✅ D: 2 → 9 (F3 evidence), chunk meshing followed |
| Hotbar wheel | ✅ selection frame moved slot 1 → 9 |
| Movement/look | ✅ WASD displacement + yaw change in F3; drag-look fallback engaged when pointer lock is unavailable |
| Break/place/middle-click | ✅ events delivered through the shim (native runs prove the block-edit pipeline: 51 edits) |
| Day-night cycle | ✅ sky darkened during the session |
| Death screen | ✅ renders correctly (YOU DIED!, score, RESPAWN/TITLE) — but see Bug B-1 |
| Standalone `/voxelcraft.html` | ✅ boots identically outside the iframe (shim + live canvas + overlay hidden) |

Console/network: no engine errors; one caught JS TypeError (Bug B-3). The settings-tree walkthrough (Video/Shaders/Controls/Accessibility/Chat/Packs/Language/Engine/Music/Skin + DONE round trip) was recorded: `browser-recordings/1790282152764-….webm`.

---

## 4. Bug list

**Fixed this sweep (CI blockers):**
- F-1 `chorus` in workflows (all 4 files) → `echo`
- F-2 wasm32 test gate, `vc-render::shaderpack::folder_tests`
- F-3 wasm32 test gates, 6 pointer-watchdog tests in `game.rs`
- F-4 stale `pack.mcmeta` assertion → `pack.json`

**Open (ranked):**
- **B-1 (HIGH, parity): unsafe spawn → suffocation death loop.** `vc-world/src/gen.rs find_spawn()` returns `heightmap + 3.0` without validating the real generated blocks (leaves/canopy/water/carves). Reproduced live: survival player idle-died ~60 s after spawn with the *default* death cause (no message set), and respawn re-died instantly. Vanilla guarantees a verified-safe spawn. *Fix sketch:* after chunk load, scan the spawn column top-down for two air blocks above a solid block; nudge outward in a spiral if not found; set `death_cause` for any residual case.
- **B-2 (MED): creative mode shows hearts/hunger/XP** — vanilla hides the survival status bars in Creative (help screenshot evidence).
- **B-3 (LOW):** unguarded `document.exitFullscreen` throws a TypeError at boot inside iframes (`voxelcraft.js` glue) — needs a try/catch or feature check in the patched glue.
- **B-4 (LOW):** Help card documents `B` = creative inventory, but `B` is unbound (E works).
- **B-5 (LOW):** F3 Targeted Block renders top-right; vanilla places it bottom-left above the F3 help line.
- **B-6 (LOW):** Select World: search box overlaps the empty-state hint text.
- **B-7 (LOW):** F3 right column `Mem: 0MB` looks stubbed on wasm.
- **B-8 (LOW):** first world boot picks render distance 2 in the wrapper (auto-detect on llvmpipe); native CI picks higher.

---

## 5. Parity snapshot vs the reference (clean-room, wiki-cited values)

**Matches (verified this sweep):** 20 Hz sim, menu-flow structure incl. the two-page world create, F3 two-column layout with the core line set + light engine numbers, hotbar/hearts/hunger/XP layout, death screen structure, settings tree coverage (Video 19-option layout, Shaders screen, 10-channel-capable sound surface, Controls, Language, Chat, Accessibility, Skin, Packs), day-night sky behavior, biome namespaced ids, creative picker structure, drag-look/pointer-capture ladder, world persistence.

**Known gaps (documented in `docs/VERIFICATION-REPORT.md`, still open):** GUI Scale option, exact 176-vanilla-eq container panel widths + light-grey theme + armor slots + player preview, font (5×7 smallcaps vs 8 px proportional with descenders), 15×15 crosshair, 24×23 selection frame, 10-channel sound sliders (engine has 2 channels on the options screen), F3 right-column detail + F3+subhotkeys.

**Measured this session:** ~10 fps on SwiftShader-class GPU with D:9 — headless-environment cost, not engine cost (CI runner reports 32–185 fps in the same suite).

## 6. Asset & format validation (static, compile-free)
- builtin-pack: 14/14 JSON valid, 2/2 PNG magic-valid, `pack.json` clean-room descriptor ✓
- classic-art: 1/1 JSON valid, 52 PNGs ✓
- WGSL: all shaders are inline WGSL validated by naga in the green CI test suite ✓
- Registry: `BLOCK_COUNT = 533` (README's 533 entries ✓); repo contains **851** `#[test]`s vs README's "818" — README drift, CI actually runs 849 (+ gated native-only).

## 7. Legal posture check
Zero third-party assets in the tested tree; all textures/fonts/sounds procedural; namespace is `voxelcraft:*` throughout the live UI and F3; the only reference-game trademarks in user-facing surfaces are neutral "1.16.5-era reference-style" descriptors. `voxelcraft:pack.mcmeta`-style naming is gone (own `pack.json`). Status: **clean**.

---

## 8. Recommended next round (to hit 90–95 %)
1. **Fix B-1 spawn safety** (highest player-facing impact, small diff).
2. Creative HUD hiding (B-2) + F3 Targeted Block placement (B-5) + help `B` binding (B-4) — small UI wins.
3. Container theme + armor slots + player preview (VERIFICATION item 6) — biggest single visual-parity jump.
4. GUI Scale option (VERIFICATION item 5) — fixes integer-scaling polish across every screen.
5. Font upgrade to 8 px proportional with descenders + 25 %-shadow (item 7).

---

# Round 2 — Vanilla-UI parity + B-1 spawn fix (commit `cad2662`, 2026-09-25)

**Scope shipped:** dirt backdrops (Settings/ResourcePacks/Shaders/Select/Create/Edit), `#C6C6C6` 9-slice container chrome + dark-grey titles on all containers + creative picker, canvas slot-well overlay removed, B-6 select-screen hint overlap fixed, **B-1 water-spawn fix** (column validation + ±8 spiral relocation + respawn-point replacement) with regression test `spawn_column_under_water_is_rejected_and_neighbor_is_accepted`.

## 9. Round-2 CI verdict — 3/3 GREEN at `cad2662`

| Workflow | Run | Result |
|---|---|---|
| `ci.yml` (849 tests + wasm32 + bench) | 36093871214 | ✅ |
| `linux-game.yml` (single-file + smoke) | 36093877489 | ✅ |
| `wasm-build.yml` (HEAD bundle) | 36093879947 | ✅ |

## 10. Round-2 live browser verification (HEAD bundle installed in `public/`)

| Surface | Result |
|---|---|
| Title (panorama, logo, splash, corner texts) | ✅ unchanged-good |
| Select World | ✅ **dirt backdrop live**, B-6 overlap **gone** (search row clean, both saved worlds listed) |
| Options | ✅ **dirt backdrop live** (all tabs inherit it) |
| Create New World | ✅ dirt backdrop, Survival default, vanilla two-page flow |
| Loading → gameplay | ✅ 10.5 s to 5 GPU chunks (first-world compile); survival, **spawned on dry land, full hearts — B-1 holding** |
| Inventory (E) | ✅ **vanilla-grey `#C6C6C6` chrome live**: armor column, player preview, 2×2 craft + arrow, off-hand slot, dark bevels |
| F3 | ✅ biome `voxelcraft:forest`, Targeted Block decode, day counter, light engine numbers |
| Movement / look | ✅ XYZ advanced, chunks streamed 169→183, drag-look fallback |
| Break + place | ✅ flower broken, input path confirmed; counters proven by native suite (51 edits) |
| Day-night | ✅ sky darkened to night during the sweep |
| Pause / Quit-to-title | ✅ blurred-world dim (vanilla), world saved |

## 11. Round-2 scaling + performance findings

- **Canvas scaling:** backing store tracks the viewport exactly (1280×720 → 1600×900 verified via computed CSS); menus/HUD recenter correctly at every size tested.
- **Panel-compositor limit:** the preview panel's software compositor stops producing frames ≥1600×900 with 3 live WebGL contexts — an environment limit, not an engine bug (DOM stays fully interactive; earlier 1920×1080 native-scope runs passed in CI's Xvfb).
- **CI bench (`bench-headless.json`, 4 threads, 96 chunks):** gen p50 **10.8 ms**/chunk, mesh p50 **10.5 ms**/chunk, remesh **2.4 ms** deterministic, draw-prep **2.57 µs/frame** — region-arena loop cuts binds 459→**27** (94 %); MDI path: 12 draws/27 binds.
- **Live wasm fps:** 9–12 fps in the software-ANGLE panel at D:2 (environment cost; native CI reports 32–185 fps on the same code).
- **B-3 note:** the boot-time `exitFullscreen` throw is already patched in the artifact glue (`try/catch`, line 754) — the one log line seen is a harmless iframe-document cosmetic; no engine change needed.

## 12. Remaining known gaps (unchanged, for the next round)

1. GUI Scale option (integer-scaling polish on every screen).
2. 10-channel sound sliders (engine surfaces 2 channels on the options screen).
3. Font upgrade to 8 px proportional with descenders + 25 % shadow.
4. F3 right-column detail (`Mem: 0MB` stub) + F3+sub-hotkeys.
5. Container panel widths matched to the 176-vanilla-eq pixel spec (theme itself is now correct).

## 13. Night-round 4 addendum — UI geometry + wiring + native-first verification (`4ce1d51`→`9780af4`)

**User-reported: "options hug one corner instead of centering" — CONFIRMED and FIXED.**
The settings-family layouts hardcoded x from the 960×540 reference (`(960−464)/2=248`),
so at any other live canvas width the widget block slid left — at the CI runner's
2560×1440 it sat literally in the corner (proven from the runner's own `widget table`
log). All ~12 settings-family layouts + 2 painter backdrops now re-anchor x via a
centered-block helper; geometry is identical at 960 (identity) and pixel-verified
centered (skew=0) at 1920×1080 and 2560×1440. Regression test
`settings_layouts_stay_centered_at_any_live_width` pins the contract at 2560.

**Vision pass (ui_snapshots --size, new):** the headless snapshot tool now accepts
`--size WxH` and drives `set_live_ui_size` exactly like the game, so every menu can be
shot at any resolution. Pixel audit: options/video L=R margins at both 1920 and 2560.

**Canvas-fallback fidelity (GPU-vs-CPU divergence the user warned about):**
- Menu backdrop: the canvas path painted a flat `[24,20,16]` fill while the GPU path
  tiles darkened dirt — the fallback now tiles the same 16×16 sprite at 32 px
  (regression test `settings_backdrop_tiles_dirt_not_flat_fill`).
- Container slots: the fallback painted no slot wells (the GPU quad layer owns them).
  It now paints the wiki-exact 32 px well (body #8B8B8B, top/left #373737,
  bottom/right #FFFFFF), so the self-healing no-GPU path is vanilla-correct.
- New `VC_GUI_CANVAS=1` knob forces canvas chrome for headless screenshot review;
  with it, native chest+furnace dumps under Xvfb show the full #C6C6C6 panel + wells
  (previously the dump captured a stale frame because quads own the chrome).

**Settings wiring audit (user: "a lot unwired"):** scripted diff of all 116 widget IDs
across 17 layouts vs game handlers — every ID is wired. The one real dead control
found and FIXED: **COPY WORLD on native** (was web-only + a "not implemented" log).
New `save::copy_world_dir` clones level.dat + region/ recursively, rewrites LevelName
to vanilla's `"<name> copy"` (dir sanitized, display space preserved, `-2` suffix on
repeats), refreshes the list with the copy selected; covered by
`copy_world_dir_clones_level_dat_and_regions`. The 10-slider Music & Sound screen and
the GUI Scale cycle (Auto/1–4) were confirmed fully wired end-to-end: sliders →
`set_sound_slider` → `cat_volumes` → `play_event` category gains → `soundCategory_*`
persistence (report §12 items 1–2 were stale; closed).

**Native-first verification (per user directive, Linux binary before wasm):**
- E2E suites on the artifact binary under Xvfb(lavapipe): v114/v114b/v114c/v115/
  v116/v116b/audit16/audit16b/containers — ALL green (campfire cook contract, blast
  furnace 2× cook, bees lifecycle, respawn anchor, bartering, smelting, chest/furnace
  screens with grey chrome + PNG dumps).
- Settings-tree menu walk (`E2E_MENU`): title→options→video→engine→shaders→packs→
  access→musicsound→…→title, exiting 0, across two live sizes (1280×696 and
  2560×1392 mid-walk via a GUI-scale change) — the centering fix holds under real
  clicks.
- F3 dynamism pair (F3_DUMP/F3_DUMP2): 2560×1392 dumps differ; overlay columns intact.
- Native bench (this host): remesh 11 ms deterministic, drawprep 12.4 µs/frame,
  binds 459→27 (17×), 10.3 KiB/chunk; `--benchmark` GPU run: MDI path (16 calls/34
  binds), stream 0.00 ms; 10 fps avg is lavapipe software-Vulkan in this VM (the GPU
  readback watchdog fired once and correctly degraded one batch to the CPU path).
- Wasm HEAD bundle reinstalled and booted: title/panorama/splash render, `__vcStats`
  pipeline live (fps/draw-path/counters), console clean. Wasm secondary per directive.
- CI: 3/3 green at `9780af4` (also `4ce1d51`, `f56fb9a`).

## 14. Night-round 5 addendum — §12 item 5 closed + vanilla HUD parity sweep

**§12 item 5 (container geometry vs the 176-vanilla-px spec) — CLOSED, verified not
just claimed.** `ContainerGeom` is constructed *inside* `container_screen()` — every
`geom.*.push((x, y))` rides the exact coordinates just drawn, so hit-test and draw
cannot drift by construction. Pixel-spec audit: hit box 36 == quad sprite 36
(`SLOT_SRC 18 × GUI_SCALE 2`) == pitch 40 − 4 gutter (vanilla 18 px × 2.2 ≈ 39.6),
panel 392 ≈ vanilla 176 × 2.2 = 387; inventory rows pitch 44, hotbar +54. New
regression test `container_hit_box_matches_drawn_slot_rect` pins: full-diagonal
occupancy of slot 0, exclusive edges, a 4-px gutter owned by *neither* neighbor,
row-wrap non-overlap, the shared-inventory pitch chain, and the canvas well pixels.
One real defect found and FIXED: the canvas-fallback slot well was 32 px while the
quad sprite and hit box were 36 (a 4-px panel-grey halo around every fallback slot);
now 36 px on both paths.

**Container E2E dump bug found by vision and FIXED:** the `E2E_CONTAINERS` chest/furnace
PNG dumps ran while `screen` was still Intro — the "chest" dump was the red studio
splash with the container composited on top (quad assertions unaffected). The leg now
forces `Screen::Game` for its rebuilds and restores it after; fresh dumps vision-
verified at 2560×1392: full #C6C6C6 panel, 36-px wells with gutters, seeded stone in
chest slot 0, furnace input/flame/fuel/arrow row, shared inventory block.

**Vanilla HUD parity sweep (clean-room, wiki-cited painters audited one by one):**
crosshair (device-snapped invert quads), hotbar (364-wide, selection frame),
hearts/hunger/armor/air (armor gated at 0, mirrored rows, jitter rules), XP bar
364×10 + level, held-item-name fade, boss bars (dragon + wither), effect icons
(split/sort/blink) — all present and quad+canvas dual-path. Three gaps found and
implemented:
- **Attack indicator (1.9+)**: was missing entirely though `swing_t`/cooldown math
  already existed for damage scaling. New `attack_indicator(p)` painter (below-
  crosshair gauge, hidden at full charge per vanilla 1.16.5 default) wired in the HUD
  path; test `attack_indicator_fills_then_hides_at_full_charge`.
- **Underwater screen tint**: audio/physics handled submersion but the camera had no
  wash. New `underwater_tint()` (full-canvas over-canvas blue solid, vanilla fluid
  overlay class) wired to `head_in_water`; test
  `underwater_tint_pushes_fullscreen_blue_only_when_submerged`.
- **F1 hide-HUD**: vanilla's HUD toggle was unbound. New `hide_hud` flag + F1 handler
  gating the whole HUD block (world keeps rendering).
Bonus: the last workspace warning (unused `System` import) cfg-gated to wasm32 —
native check is now 0 warnings.

**Native-first verification (Linux before wasm, per directive), this round:**
- `cargo test --workspace`: all green — vc-render 197/0/1-ign, vc-sim 62, vc-pack 32,
  vc-mesh 11, voxelcraft lib 92, plus 367 across anvil/audio/world/chunk/gameplay/
  blocks/rng. Native check 0 warnings.
- Native E2E under Xvfb(lavapipe) on the fresh release binary: smoke+v114/v114b/
  v114c/v115/v116/v116b/audit16/audit16b exit 0 with all CI grep assertions;
  settings-tree walk ok; containers leg ok (geom 27+36, panel quads, furnace slots).
- Vision: chest36/furnace36 dumps (canvas fallback) inspected in-browser at 2 zoom
  levels — geometry, chrome, wells, gutters, seeded items all correct.
- Wasm secondary: no UI/layout divergence remains (the wasm32 splits are platform
  seams only: file-IO vs fetch, MDI vs emulation, DPR twins); the one real divergence
  (corner-hug centering) was fixed and pinned last round.
