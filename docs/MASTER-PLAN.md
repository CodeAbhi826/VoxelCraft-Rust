# MASTER-PLAN.md — the merged plan of record (2026-09-26, execution copy)

**Baseline:** CI 3/3 green at `9780af4`. Native-first verification is the
standing directive (Linux binary before wasm every round). Clean-room rules
and `docs/LEGAL-COMPLIANCE.md` apply to every round.

**This file merges all three plans into one:**
- **Part I** — the 1.16.5 Transformation Roadmap's 13-phase ladder (original
  `upload/VoxelCraft-MC1.16.5-Transformation-Roadmap.md`, audited in
  `docs/ROADMAP-ANALYSIS.md` §3) — CLOSED.
- **Part II** — the Master Engineering Spec's Phase 0–11 ladder (original
  `upload/VoxelCraft-Rust_1.16.5_Master_Engineering_Spec_FINAL.md`) — CLOSED.
- **Part III** — the live execution plan: **19 rounds A–S** built from the
  2026-09-26 code audit (`ROUND-16-PARITY-GAP.md`), the mod feature-set
  survey (Sodium/Lithium/Phosphor/Starlight/FerriteCore), and the test-report
  leftovers.

**Standing policies (every round, no exceptions):**
1. **Nothing is skipped** — if an item is already built, it is verified live
   and marked done with evidence, never silently dropped.
2. **Settings dedup** — no control may appear in two layouts; every option
   lives on exactly one vanilla-style page (enforced by Round O's test).
3. **Repo hygiene** — dead code, debug scratch, and illegal-content scans are
   scripted and kept green (Round Q), then re-run in Round S.
4. **Per-round commits** — implement → unit tests → native E2E under Xvfb →
   vision-verify dumps → update TEST-REPORT + `ROUND-16-PARITY-GAP.md` →
   commit → wait for CI green → next round.
5. **Native-first, wasm secondary** — wasm bundle refreshed in Round D and
   re-verified at the end.
6. **Mid-round rule** — anything discovered missing or broken mid-round gets
   verified live, fixed or scoped, and ledgered the same round.

---

# Part I — Transformation Roadmap: 13-phase ladder ✅ CLOSED

Status per phase (the roadmap was "a good requirements document and a poor
implementation spec" — its vertex bit-budget and AtomicPtr sketches were
replaced by VC-16 + paletted CoW):

| # | Phase | Status | Evidence |
|---|---|---|---|
| 1 | GameSettings + persistence | ✅ | options.txt/localStorage; vanilla Options tree |
| 2 | BlockState + JSON models | ✅ | state registry, slab/stairs/fence models |
| 3 | Paletted 16³ sections | ✅ | 4b→direct ladder, ~0.5 KiB air chunks |
| 4 | Packed vertex + mesher | ✅ | VC-16, Sodium 0.5.1 parity |
| 5 | Resource-pack atlas loader | ✅ | vc-pack: folder/zip, blocks + GUI reskin |
| 6 | Deferred G-Buffer | ⏸ deferred | forward+ ships shadows/light/FSR; revisit at shader-pack v2 |
| 7 | Shadow pass | ✅ | 2048² sun, 3×3 PCF |
| 8 | Region MDI | ✅ | binds 1614→23; MDI/instance-loop paths |
| 9 | FSR 1.0 | ✅ | faithful EASU+RCAS WGSL ports |
| 10 | Enhanced F3 | ✅ | vanilla two-column; sub-hotkeys in Round A |
| 11 | Iris shader-pack loader | ✅ to gate | scan + tiers + GLSL→WGSL composite/final chain |
| 12 | Spatial audio + sounds.json | ✅ | registry, categories, spatialize(), music pads |
| 13 | Polish | ◐ | menu/tint/clouds done; mipmap slider in Round F |

# Part II — Master Engineering Spec: Phase 0–11 ladder ✅ CLOSED

| Phase | Item | Status | Evidence |
|---|---|---|---|
| P0 | Bench harness | ✅ | `--benchmark`, `vc_bench`, F3 timing |
| P1 | Block/asset foundation | ✅ | parsers, builtin pack, animated textures |
| P2 | World data | ✅ | paletted sections + Anvil/level.dat |
| P3 | Mesh system | ✅ | VC-16 + arenas + batching |
| P4 | Lighting | ✅ | incremental cross-chunk LightEngine |
| P5 | Vanilla rendering | ✅ | tint, particles, shadows, FSR |
| P6 | Simulation | ✅ | 20 Hz deterministic; fluids; redstone core |
| P7 | Gameplay | ✅ | containers, brewing/enchanting/anvil/grindstone/beacon, villagers, structures, dimensions, hunger, mobs+dragon |
| P8 | Audio + UI | ✅ | sound registry; full menu/HUD surface |
| P9 | Draw submission | ✅ | arenas, MDI, near→far region order |
| P10 | FSR | ✅ | real EASU+RCAS |
| P11 | Shader-pack API | ✅ | §34.2 tier; Iris-format loading |

---

# Part III — Execution plan: 19 rounds A–S

Order: A–C evidence first → D–E debts/polish → I–N content (player-visible)
→ F–H quality → M perf → O–R hardening → S final adversarial QA.

## Round A — Input & camera parity (F-keys + bindings)
- **F5 third-person camera** (absent entirely): behind/front views with the
  player model rendered (multi-box rig exists; reuses the view-model).
- **F2 screenshot** to `screenshots/` with toast feedback.
- **Full function-key contract test**: F1–F5, F3+G chunk borders (new),
  F3+Q/F3+H (exist), F11 fullscreen — each via the real input path.
- B-4 (`B` creative binding per Help card), B-5 (F3 Targeted Block to
  vanilla bottom-left), B-8 (first-boot render-distance floor), README
  test-count fix.
Gate: contract E2E greps each key's effect; vision dumps of both F5 views.

## Round B — Menu vision audit
Ladder **1280×720 → 1366×768 (primary) → 1600×900 → 1920×1080 → 2560×1440 →
1280×1024 (letterbox)** + GUI-scale Auto/2/3/4 sweep across every screen;
pixel-audit centering/skew (the 2026-09-25 helper contract).

## Round C — In-game world captures
Terrain/water/cave/day-night/F3 at ladder sizes; doubles as the glitch-hunt
net for Rounds F–G.

## Round D — Debts
Bench re-run (gen/mesh/remesh/drawprep), extra container dumps, wasm bundle
refresh + browser re-verify, B-3 glue re-check.

## Round E — Polish
8 remaining container screens at vanilla geometry, intro-beat polish, pitch
decision doc.

## Round F — Rendering quality
Texture-crispness/mip audit + **mipmap 0–4 slider**, MSAA + FSR quality modes
(FSR2 = separate honest project), gamma/RGB tuning pass, glitch sweep vs
Round C captures.

## Round G — Entities & animation
G1 mob models/textures (strider/drowned/trident class), G2 items-in-hand +
drops + break animations, G3 walk/run/swim cycles + scripted live gameplay
test.

## Round H — Terrain realism
Generation audit vs 1.16.5 surface rules, carve/feature density, biome
transitions; improvements with golden-seed before/after captures.

## Round I — TNT & explosions
TNT block + primed entity (4 s fuse, physics), terrain damage + drops, chain
reactions; shares the ignition path with Round K.

## Round J — Beds
2-part multi-state block, sleep → night skip (thunder check), spawn-point
set + respawn wiring, phantom gate (`weather.sleep_reset` exists).

## Round K — Nether portal
PORTAL block (animated purple), obsidian-frame validation, flint-and-steel,
walk-in trigger via §28 travel (8:1 exists), shimmer particles + fog,
far-side spawn frame; End portal flow verified end-to-end. Gate: round trip,
save/reload keeps both sides.

## Round L — Difficulty + persistence
Peaceful/Easy/Normal/Hard selector + rules (hostile purge, starvation
thresholds, conversion odds); player NBT: health/xp/food/saturation/spawn.
Gate: difficulty E2E + save/reload keeps vitals.

## Round M — Weather presentation
Rain/snow streaks (biome-picked), surface splashes, spatial thunder clap,
sky-darkening check (`sky_factor` wired). Gate: `force_rain` E2E + night-rain
vision dumps.

## Round N — Small items
Bone Meal (bone→3 meal, crop stage advance, grass), compass/clock (real
angle/time), map item (chunk-color canvas). Jukebox/note blocks scoped in
Round N-2 if the audio path has capacity, else next cycle.

## Round O — Settings overhaul
Remove the Options MUSIC/SOUND slider pair (pre-1.14 style, duplicates the
10-slider Music & Sound screen; both patch the same channels) → vanilla
"Music & Sounds..." button. Then a **full 17-layout audit**: dedupe every
control, re-categorize pages, verify wiring after moves, regression test
pinning "no option id appears in two layouts."

## Round P — Pack & datapack coverage audit
Install a real example pack: every loader path reskins (textures, gui,
sounds, models); craft/tag/loot datapack features end-to-end; shader-pack
demo loads in the pipeline; malformed pack errors gracefully (never panics);
hot-reload documented (present or absent).

## Round Q — Repo hygiene
Dead code sweep (`#[allow(dead_code)]`, unused fns/consts), debug scratch
(`diag/`), stale TODOs, **illegal/flagged-content scan** (no copied
assets/trademark strings; license headers on clean-room art) — scripted for
CI.

## Round R — Save & world robustness
Save-format versioning/migration tests, corrupted-save recovery (truncated
region, bad level.dat → graceful error screen, never a panic), autosave
cadence + save-on-quit verification, large-world open (memory/load time),
COPY WORLD re-verified on a big world.

## Round S — Final adversarial QA
A dedicated flaw-finding pass: menu random-input fuzzing, save/load
round-trips, every block/entity/interaction spot-check, edge cases (empty/
full inventory, death mid-action, reload mid-animation), re-run every prior
round's regression suite as one gate. Round ends only when a full session
produces zero new defects.

---

## Beyond-1.16.5 queue (post-parity)
1. Shader-pack v2 (texture/depth + gbuffers) → re-evaluate G-buffer.
2. 1.17+ bracket (caves & cliffs, copper, amethyst, glow squid/axolotl/goat).
3. Multiplayer skeleton behind the 20 Hz core.

## Perf-parity standing notes (Sodium/Lithium/Phosphor/Starlight/FerriteCore)
Core Sodium wins already shipped (compute mesher, MDI, binds 459→27,
per-vertex AO, culling). Round M-class audits fold into H/F/O as encountered:
occlusion/entity culling, Starlight-class light queue, Lithium-class tick
batching, FerriteCore-class blockstate dedup — each ends "already optimal"
or gets fixed, bench-guardrailed.
