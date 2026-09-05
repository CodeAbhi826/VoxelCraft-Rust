# VoxelCraft-Rust — Worklog

Append-only progress log. One section per work unit; newest at the bottom.
Commit references use the short hash.

---

## 2026-09-05 — blocking-bug triage: "connection in textures" + "lag when rendering"

**Task:** user reported graphical issues (texture seams / "connection in
textures") and rendering lag; requested a full every-part evaluation of all
changes so far.

**First finding — repo state honesty check:** the user was RIGHT that all
phases were complete. The remote (`origin/main`) carries Phases 0–10 plus a
post-Phase-10 texture-seam fix (`fe70cd9`) — 17 commits ahead of this
machine's stale checkout (`e99d4df`, Phase 4). This session's initial live
diagnosis ran against the stale local `public/` bundle, which reproduced the
user's symptoms in full. After fetching, the still-unfixed remainder was
re-diagnosed against the real Phase-10 code and fixed (below).

**Diagnosis (live, in-browser WebGL2 + SwiftShader, F3 overlay + VLM screenshot
analysis; initially against the Phase-4-era bundle):**

1. **Fall-through-world (root cause of the "graphical issues")** — F3 showed
   the player at **y = −2311.75** falling through the void, 0 chunks drawn.
   Chain: on slow devices the 15 s Loading timeout expires before the spawn
   chunk is *meshed*; the spawn snap was gated on the GPU mesh
   (`renderer.has_chunk`), so it never ran; the game starts with the player at
   spawn+20 in mid-air; `World::get_block` returns AIR over not-yet-generated
   chunks, so gravity free-falls the player below y=0 where nothing can ever
   collide again. No void damage ⇒ falls forever at terminal velocity. **Still
   present in the Phase-10 remote code — fixed by this commit.**
2. **Neighbor-tile atlas bleed ("connection in textures")** — terrain/water
   fragment shaders sampled `(tile + fract(uv))/16` with no edge guard:
   tiny negative interpolation epsilon wraps `fract` to ~1.0, and
   `tile + 0.99999994` rounds up to the next integer in f32 — the sample lands
   exactly on the tile boundary where Nearest filtering returns the
   NEIGHBORING tile's first texel (1-px wrong-texture lines at block joints).
   Independently diagnosed here; **already fixed on the remote by `fe70cd9`**
   with the same half-texel inset (0.03125/0.96875) plus
   `textureSampleGrad` analytic gradients (LOD explosion at fract
   discontinuities) and an occlusion-flood cache — strictly more complete
   than this session's shader draft, so the remote version is kept verbatim.
3. **F3 overlay FPS stats corruption** — "2147483547 max" in the F3 header:
   the min/max folds had swapped initializers (`fold(0.0, f32::min)` collapses
   to 0.0 → fps_max = 1000/0 = inf → saturates to i32::MAX; `fold(INF,
   f32::max)` collapses to INF → fps_min = 0). **Still present on the remote —
   fixed by this commit** + extracted into a tested `fps_min_max()` helper.
4. **Wasm initial-fill slowness (contributor to perceived lag)** — mesh-job
   cap was 2/frame on wasm while the real frame guard is the 6 ms inline
   budget; raised to 4 (slow devices unaffected — the budget loop always
   breaks after the first job that crosses it).

**Fixes in this commit (all on top of the fetched Phase-10 remote state):**

- `game.rs`: spawn snap now keys on chunk **data** (not the GPU mesh),
  extracted into `try_snap_to_surface()` and also runs from the first Game
  frames as defense in depth for the timeout path.
- `game.rs`: player physics is **held while the player's own chunk is not
  generated** (`physics_frozen`) — vanilla semantics: entities in unloaded
  chunks do not tick. Also covers fast creative flight outrunning the
  generation frontier.
- `game.rs`: F3 min/max FPS fold fix + `fps_min_max()` helper.
- `game.rs`: wasm mesh-job cap 2 → 4.
- Regression tests: `physics_freezes_until_own_chunk_exists`,
  `fps_min_max_orders_the_folds`.

**Verification (stale-bundle pre/post + merged-state rerun):**

- Before (Phase-4-era bundle): F3 `XYZ 40.58 / −2311.75 / −39.59`, **0 chunks
  drawn**, tris 0, "max 2147483547 fps"
  (`docs/screenshots/bugfix-void-fall-before.png`).
- After (merged Phase-10 + fixes, rebuilt wasm, live browser E2E): stats
  bridge reports **y = 65.0 standing at world spawn**, **35 chunks drawn /
  71,210 tris**, fps stat ordered and finite, mip 4 + aniso 4 + occlusion
  culling active, Phase-5 villagers present (4), sim ticking (469 ticks)
  (`docs/screenshots/bugfix-void-fall-after.png`).
- Close-up terrain VLM inspection after the shader inset: **no wrong-colored
  1-px boundary lines, clean block-to-block tiling, no z-fighting**
  (`docs/screenshots/bugfix-texture-seams-verified.png`).
- Interaction E2E: `break:9:64:9` → `probe:9:64:9` reports AIR — mining,
  mesh invalidation and probe read-back live.
- Full workspace suite on the merged state: **297 passed / 0 failed** (295
  baseline + the two new regression tests
  `physics_freezes_until_own_chunk_exists` and
  `fps_min_max_orders_the_folds`).

**Lag note:** in the headless verifier the frame is ~100 ms because
SwiftShader rasterizes in software — that is this test box's floor, not the
engine's. On real GPUs the frame cost is GPU-bound; engine-side factors
addressed here are the initial-fill mesh rate and the void-fall freeze.
Shadow mapping (Settings → SHADOWS, default ON, vanilla has none) remains the
biggest optional frame cost on weak GPUs — turn it OFF in Options for
headroom on WebGL2.

**Also this session:** research-verdict gate added at
`docs/research/research-verdicts.md` — the confirmed/confirmed-wrong/
unverified categorization for the two AI-generated research documents
(mechanics + UI/visuals). Key outcomes: pointed-dripstone section is 1.17
content, **deleted from scope**; the "all 18 container screens are 176×166"
table is wrong (hopper is 176×133 — every screen must be individually
verified); the `(height−3)×0.2` fall-damage formula circulating in SEO
sources is wrong (real: `fall_distance − 3` half-hearts, already implemented
per MC-12357); Monocraft OFL 1.1 + GPL 3 and the gravity formula
`v1 = (v0 − 0.08) × 0.98` are confirmed usable.

---

## 2026-09-05 — mechanics + visuals implementation round (research documents, verdict-gated)

**Task:** the user asked whether the "visuals and mechanics update" from the
two AI-generated research documents had been implemented; if not, read the
full documents, verify against live sources where needed, and implement.

**Verification pass first** (per the standing research-verdicts gate): every
"unverified" row that touches an existing engine system was checked against
the live wiki; outcomes appended to
`docs/research/research-verdicts.md` (live round table). Highlights:
- CONFIRMED: sprint-swim 3.918 b/s (surface 2.20 / underwater 1.97 — the
  doc's "downstream 1.81 / upstream 0.39" labels were mislabeled), drowning
  (air 300, 2 HP/s at −20, 10 bubbles × 30, regen 30/4 ticks), villager
  gossip table + trade-price rule, passive spawn cycle (1 per 400 ticks,
  chunk-gen spawn ignores the cap), falling-block entity physics
  (gravity 0.04, Drag-Y 0.98 — items share it), scaffolding falls at
  distance ≥ 7, hopper container (5 slots, "Item Hopper", 8-tick transfer
  cooldown), F3 "Looking at fluid" split (1.13 18w22c — valid for 1.16.5).
- CONTRADICTED: firework boost 33.5 b/s → current wiki says 35.5 (elytra
  not in engine; recorded only).
- STILL UNVERIFIED (no engine system, no live confirmation): minecart
  friction 0.01, Nether biome spawn weights as 1.16.5-exact, falling-block
  2/5-tick spawn delay. None were implemented; verified data lives in the
  verdicts doc for future phases.

**Mechanics implemented (all verdict-cited in code comments):**
- Exact vanilla gravity drag `v1 = (v0 − 0.08) × 0.98` on a fixed 20 Hz
  substep for the PLAYER (move-then-gravity ordering — vanilla tick order;
  jump re-aligns the substep phase so the 0.42 b/t launch rises the
  vanilla 1.25 blocks), for MOBS (b/s units: `(v − 1.6) × 0.98`; also
  fixes a latent 20× unit bug that made mobs fall 20× too slow), for
  VILLAGERS (b/tick, non-vanilla −0.5 clamp removed), and item entities
  gained the missing air Drag-Y 0.98.
- Mob fall damage rewritten distance-based (MC-12357: fall − 3) — the old
  impact-speed path was provably dead code; terminal falls (78.4 b/s) now
  substep the vertical probe so they cannot tunnel floors.
- Swimming speeds from the verified table (sprint-swim 3.918, underwater
  1.97, surface 2.20).
- Air supply + drowning: 300 air (−1/tick submerged), damage 2 HP when air
  hits −20 then reset (≈1 damage per second), regen 30 air / 4 ticks out of
  water; creative drains air visually but is damage-immune.
- Villager GOSSIP system: full verified table (trading 4/2/20/25/×1,
  major_positive 20/0/100/20/×5, minor_positive 25/1/5/25/×1,
  minor_negative 25/20/20/200/×−1, major_negative 25/10/10/100/×−5),
  per-trade +4, attack +25 (targeted), kill broadcast to the 16-block box,
  decay every 24000 ticks, proximity sharing (shared value − sharing cost,
  major_positive unshareable), reputation = Σ value × multiplier, and
  reputation-priced trades: `clamp(base − floor(rep × 0.05), 1, 64)`
  (Java Math.floor semantics — floor(−1.25) = −2). Villagers gained 20 HP
  + a player melee path (armor 0) so the hooks are reachable.

**Visuals implemented:**
- Hopper container screen at the verdict-corrected 176×133 proportions
  (ONE row of 5 slots — not the blanket 176×166), vanilla "Item Hopper"
  title, wired to right-click + break-spill + the generic container slot
  path; `open:hopper` E2E command added.
- Oxygen bubble row (10 clean-room bubbles above hunger, ceil(air/30),
  creative included); drawn only below full air.
- Held-item name above the XP bar, ~2 s fade on selection change.
- F3 vanilla-parity lines: XYZ 3-decimals, Block/Chunk with in-chunk
  coords, `Facing: south (Towards positive Z) (yaw / pitch)` with the
  vanilla yaw/pitch conventions, `Client Light: L (S sky, B block)` from
  the real light engine, `Looking at block/fluid` split (water → the fluid
  line). JVM-specific lines stay engine-adapted (Rust + wgpu backend row).

**Verification:** 310/310 workspace tests green (was 297; +13), wasm32
target clean, fresh wasm bundle deployed and live browser E2E on
WebGL2/SwiftShader: hopper screen VLM-verified (title/slots/seeded
item), F3 overlay VLM-verified (Facing/Client Light formats), survival
gameplay stable. Pre-existing clippy lint (`never_loop` in vc-pack
datapack pattern matcher) noted for a future pass — not touched
(minimal-change discipline).

Screenshots: `docs/screenshots/e2e-hopper-screen.png`,
`docs/screenshots/e2e-f3-lines.png`.

---

## Prior phases (from git history)

- Phase 0 — Apache-2.0 LICENSE + README license section (`4f11030`)
- Phase 1 — game modes + world creation + death/respawn (`d3bd25b`)
- Phase 2 — mobs + combat, live-verified data (`61a5de6`)
- Phase 3 — redstone full component set + containers (`5356fd2`)
- Phase 4 — enchanting (38-entry registry) + corruption brewing (`e99d4df`)
- Phase 5 — villager trading depth + dungeons with spawners (`05ff5d7`)
- Phase 6 — rendering optimization suite (`d6583dc`)
- Phase 7 — GPU compute greedy mesher, WGSL bit-identical (`b6d744e`)
- Phase 8 — Iris shader-pack integration interface (`8ff9722`)
- Phase 9 — Mojang-official data packs (`feec9b4`)
- Phase 10 — content breadth: 14 biomes, 5 structures, loot attribution
  (`7625f92`)
- Post-10 — texture-seam fix + occlusion-flood cache (`fe70cd9`)
- Workspace split into 14 library crates + per-library release archives +
  all-arch CI (`7378c08`, `f4b68a6`)
