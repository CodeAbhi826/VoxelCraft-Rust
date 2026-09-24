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
