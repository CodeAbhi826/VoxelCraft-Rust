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

## 2026-09-06 — MC 1.0–1.2 bracket (version-evolution Phase 1: Core World Content) — commit d60e62f

**Task:** first bracket of the 1.0 → 1.16.5 version-evolution ordering
(`evolution-research.md` Part 3 Phase 1). All values live-verified at
implementation time per the STRICT PROTOCOL — the round's research record
(what was checked, against which wiki page, including intra-page and
inter-page disagreements) is `docs/research/phase1-1.0-1.2-research.md`;
204 `VERIFIED` citations live in the code comments.

### Implemented

- **The End dimension**: 5×5 obsidian entry platform at (100, 0), central
  end-stone island, 10 obsidian pillars on the 42-radius circle down to
  y=0 with bedrock caps, 10 end crystals (2 in iron-bar cages),
  deterministic per-seed. Strongholds got the 5×5 end-portal-frame ring
  (12 frames, corners cut) over lava; eye-of-ender filling activates the
  central 3×3 into end-portal blocks; dimension travel both directions
  (portal room → the End; exit fountain → home).
- **Ender Dragon fight** (`dragon.rs`, 444 lines): 200 HP, damage only
  from players + explosions, crystal healing (1 HP / 10 ticks within
  32-block cuboid), 10-HP backlash when a healing crystal is destroyed,
  power-6 crystal explosions, death timeline (XP at 154 ticks into the
  ascension, exit portal + dragon egg at 200 ticks), 12,000 first-kill
  XP / 500 re-summoned. The dragon + crystals render as End billboards
  in-game.
- **Nether Fortress**: 432×432 regions (Java), deterministic per-region
  rolls, nether-brick bridges/corridors on pillars, up to 2 blaze-spawner
  platforms, nether-wart gardens near stairwells.
- **Mushroom Fields biome**: mycelium surface, ocean-island placement,
  no natural hostile spawns, huge red/brown mushrooms (exactly 45 cap
  blocks + stalk), mooshrooms (JE weight 8/8, groups 4–8).
- **Mobs**: Snow Golem (2-snow-blocks + pumpkin-last build, 1 snowball/s
  at hostiles within 10 blocks, 1 HP/tick melt in hot biomes + rain),
  Magma Cube (HP = size², attack = size+2, armor = 3×size, splits into
  2–4 on death, fireproof, 16-block aggro), Blaze (HP 20, 3-fireball
  burst after 3 s charge, fortress spawner at light ≤ 11, 50% blaze rod),
  Ocelot (flees players, hunts chickens ≤ 15 blocks, jungle-only),
  Iron Golem (HP 100, village guard, 4-blocks-T + pumpkin build), Zombie
  Villager (infection Easy 0%/Normal 50%/Hard 100%, cure = Weakness +
  golden apple over 3600–6000 ticks), Mooshroom (shear → 5 mushrooms +
  cow, bowl → mushroom stew).
- **XP orb system**: vanilla value ladder 1/3/7/17/37/73/149/307/617/
  1237/2477, 7.25-block attraction accelerating near the player, 10
  orbs/s pickup gate (2-tick), 6000-tick despawn, green↔yellow fade,
  no merging (merging is 1.17+ — version-scoped check), mob XP only on
  player kill or within 100 ticks of a player hit.
- **Spawn eggs**: use-on-surface spawn (feet adjacent), spawner
  retarget, baby form on same-type, creative-picker-only item.
- **Blocks**: mycelium (spread 1-up/1-side/3-down, revert under opaque
  cover at light < 4), redstone lamp (light 15 when powered, 4-game-tick
  off delay, 4-glowstone + 1-redstone craft), chiseled stone bricks,
  chiseled/cut/smooth sandstone (smooth = smelt-only, 1.14-valid),
  nether-wart crop (4 age stages, 10%/random tick, soul-sand only,
  2–4 mature drops), end stone (hardness 3, blast 9).
- **Clean-room art** (`e1_art.rs`, 747 lines) for every new block; zero
  extracted/recreated Mojang assets.

### Verified

- Every constant above carries a `VERIFIED w/<page>` comment from this
  round's live wiki fetch (record: research doc above).
- Test suite: **339 passed / 0 failed** (was 310; +29: dragon fight
  timeline, crystal-heal rules, golem build patterns, zombie-villager
  cure lifecycle, ocelot AI, magma scaling, XP ladder/attract/despawn,
  mycelium spread/revert, nether-wart stages, lamp toggle, End geometry
  (42-radius pillar circle, central island), fortress determinism,
  huge-mushroom cap counts, registry rows, picker entries).

### Placeholder-unresolved

- **Snow Golem snow-trail biome gate**: the wiki page contradicts itself
  (lead paragraph = temperature-gated; §Behavior = "any biome, Java").
  Implemented the temperature-gated reading (temp < 0.5) and disclosed
  here; revisit if a better source lands.
- **Dragon first-kill XP split**: Ender_Dragon page says 10×960 + 1×2400;
  the Experience page says 10×1000 + 1×2000 (both = 12,000 total — an
  intra-wiki disagreement). The dragon-page split is implemented; the
  total is what actually matters mechanically since orbs use the ladder.
- **Iron Golem drops (3–5 iron + 0–2 poppy)**: drop-table section was
  unreadable via live extraction this round; widely-cited values
  implemented and flagged, not live-confirmed.
- **Biome temps for Desert/Mountains/Ocean/Beach/Savanna** (2.0 / 0.2 /
  0.5 / 0.8 / 0.95): not extractable from the live biome table this
  round; widely-cited values, flagged.

### Deferred

- Dragon breath attack / lingering-area fire (dragon fireball damage
  row is cited; the breath *system* rides the 1.9-style effects work).
- Re-summon ritual (4 side crystals + dragon spawn via the end-portal
  sequence) — the re-fight XP value is already in place.
- Wither-skeleton fortress spawns (mob itself is a 1.4 bracket item).
- Beds / sleep-to-morning (1.0 feature not on the evolution Phase-1
  list; bed *explosions* in the Nether/End are recorded in the supplement
  for the dimension brackets).

### Known issues & regressions

- None observed this bracket: wasm32 target still compiles clean
  (`--no-default-features`), full suite green, no new clippy lints
  introduced (the pre-existing `never_loop` in vc-pack remains).
- Engine graphics for the new content use the billboard/sprite path for
  the dragon + crystals (no articulated dragon model — acceptable for
  this bracket; revisit with a mesh pass if the user wants closer
  visual parity).

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

---

## 2026-09-06 — visual & mechanical verification round (the 7-section test) — commit pending-push

**Task:** execute the user's 200+ item visual/mechanical verification test
(§1 font, §2 HUD, §3 containers, §4 settings, §5 F3, §6 mechanics, §7
rendering) against the live engine, under STRICT PROTOCOL discipline —
every asserted number re-verified live this round (minecraft.wiki),
nothing copied from the old research dumps. Full verdict table:
`docs/VERIFICATION-REPORT.md`.

**Environment honesty note:** the headless verifier renders via
SwiftShader (software raster, ~10 fps, sim advancing sub-realtime), so the
in-game "10 s walk = 43 blocks" timing checks were replaced by
code-constant + convergence-test verification (the constants themselves
live-verified). Pixel measurements taken at the default 1280×720 window.
The user's 7 reference screenshots (options / video settings / resource
packs / select world / F3 gameplay / creative inventory / survival
inventory) were all reviewed via VLM and are recorded as the visual
target set.

### Implemented (this round's fixes — live-confirmed corrections only)

- **Day-night cycle 600 s → 1200 s**: vanilla 1.16.5 = 24000 ticks @
  20 tps = 20 min (live: w/Daylight_cycle, w/Tick). BOTH the old engine
  value AND the checklist's "10-min" claim were wrong — noted as a
  checklist error, not just an engine one. Extracted `DAY_LEN_SECS`
  constant + regression test.
- **Wooden-slab fuel 300 → 150 ticks** (live: w/Smelting fuel table).
  Planks/logs/table/fence stay 300 (verified). Tests:
  `fuel_table_matches_the_live_wiki`, `slab_burns_half_as_long_as_planks`.

### Verified

- Live-verified this round: walk 4.317 / sprint 5.612 / sprint-jump 7.127
  (w/Walking, w/Sprinting, w/Transportation); smelting 200 ticks + coal
  1600 (w/Smelting, w/Furnace); wooden slab 150; day cycle 20 min; lava
  spread Nether 7 blocks/10 ticks + Overworld 3 blocks/30 ticks (w/Lava —
  for when lava sim lands); guiScale semantics (w/Options.txt: 0=Auto or
  integer). Existing verified rows (gravity formula, MC-12357 fall
  damage, combat constants, water 5-tick spread, hopper 176×133) were
  re-checked, not assumed.
- Suite: **342/342 green** (339 → +3 tests), wasm32 clean, wasm bundle
  rebuilt from the bracket-1 tree and live-tested in-browser (world
  create → game entry → HUD/F3/inventory screenshots → VLM inspection).
- In-game live checks: HUD (9 slots / 10 hearts / 10 drumsticks /
  bubbles logic / held-item fade), F3 overlay (~24 left lines, all core
  vanilla lines present), inventory screen functional.

### Placeholder-unresolved

- None new. (Carried: snow-golem trail gate, dragon XP split, iron-golem
  drops, 5 biome temps — all disclosed in the bracket-1 entry above.)

### Deferred (from the report's priority list, awaiting user direction)

- Mechanical (small): render-distance slider 2–32 (engine 2–16); lava
  fluid sim; sprint-jump 7.127 emergence test; coal item + 1600-tick fuel.
- Visual (design-sized): GUI Scale option + integer UI scaling (the fixed
  960×540 canvas currently yields a non-integer 2.67× effective scale at
  1280×720); vanilla light-grey #C6C6C6 container theme + exact 176-wide
  panels + armor slots + player model; font upgrade (8 px descenders,
  proportional, 25 %-color shadow, § codes) or Monocraft adoption;
  selection-frame/XP-bar/crosshair micro-sizes; 10-channel audio; F3
  right column + sub-hotkeys.

### Known issues & regressions

- §1/§2/§3 carry structural deviations from vanilla styling (dark
  container theme, 5×7 smallcaps font, no armor slots/player model,
  non-integer GUI scale). These are pre-existing design decisions, now
  formally measured and disclosed in the report rather than silently
  kept. No gameplay regressions: 342/342, world create/play/E2E paths
  all live.

---

## 2026-09-06 — MC 1.3–1.4 bracket (version-evolution Phase 2: Adventure Features) — commit e9e79de

**Task:** second bracket of the 1.0 → 1.16.5 version-evolution ordering
(`evolution-research.md` Part 3 Phase 2). All values live-verified this
round against minecraft.wiki — the round's research record (what was
checked, against which page, including the disagreements) is
`docs/research/phase2-1.3-1.4-research.md`; ~120 `VERIFIED` citations
live in the code comments.

### Implemented

- **Structural: the block-state space widened u8 → u16** (`Chunk::get`,
  `get_idx`). The E1 bracket had exhausted every state id ≤ 255; E2
  world blocks live at 283+. All call sites either already folded via
  `state_block` (the `as u16` casts became no-ops) or compared against
  identity-mapped ids ≤ 56 — every comparison site now folds explicitly.
- **The Wither boss** (`wither.rs`, ~370 lines): summon = 4 soul sand in
  a T + 3 wither-skeleton skulls, last block must be a skull; 220-tick
  invulnerable charge with the boss bar filling; birth explosion
  (power-6-class, proximity damage); 300 HP Java row; passive regen
  1 HP/20 ticks; black skulls every 2 s (8 HP + Wither II 10 s Normal /
  40 s Hard via the new effects system); 40-block aggro, hovers 5 above
  the target; breaks a 3×4×3 box of blocks on taking damage (bedrock +
  portal blocks immune); drops 1 nether star (100%) + 50 XP; billboard
  sprite render + boss bar. Side-head multi-target AI compressed to the
  main-head cadence + a 2–3 s volley (disclosed adaptation).
- **Three mobs** (mobs.rs + drops + spawns): Wither Skeleton (20 HP,
  stone sword 8 Normal, Wither I 10 s on hit, coal/bone/skull-2.5%
  drops; fortress spawner platform #2), Witch (26 HP, splash-potion
  attack 6, joins the dark monster pool at the verified ~0.97% share,
  per-item 0–2 drops), Bat (6 HP, ambient, light ≤ 3 below sea level,
  groups of 8, ambient cap 10, no passive-cap pressure, empty drop
  table).
- **Effects system** (`effects.rs`): Wither (40/20-tick periods, can
  kill), Poison (25-tick, floors at 1 HP), Regeneration (50-tick), plus
  the beacon stat effects (Speed +20%/level, Strength +3/level,
  Resistance −20%/level floor 20%, Jump Boost +0.1/level). Applied to
  the player every tick; beacons refresh through the same path.
- **Beacon** (`beacon.rs` + game wiring): pyramid scan 1–4 levels
  (9/34/83/164 blocks, mixed materials allowed), powers gated by level
  (Speed/Haste 1+, Resistance/Jump 2+, Strength 3+), secondary at 4 =
  Regeneration or primary II; effects every 4 s for 9+2×level s at
  20/30/40/50 range; feed via iron/gold/diamond ore-or-block/emerald
  (adaptation: no ingot/gem items); light 15; feed-cycles the powers
  (adaptation: no beacon GUI, disclosed).
- **Ender Chest**: craft 8 obsidian + eye of ender; right-click opens
  the shared 27-slot container (sentinel-keyed — every ender chest opens
  the same inventory, the single-player form of the vanilla per-player
  rule); breaks into 8 obsidian, contents never spill; light 7.
- **Adventure mode** (modes.rs): vanilla GameType 2, saved/round-tripped;
  no direct block break or place (Java needs item components — plain
  denial, disclosed); all interactions (mobs, levers, containers,
  crafting) stay available; everything else = Survival rules.
- **Anvil** (block family + `anvil.rs`): 3 damage stages (12% per use,
  pristine→chipped→damaged→destroyed), gravity block (falls like sand),
  craft 3 iron blocks + 4 iron ore (adaptation). Falling damage on
  entities + repair/combine/rename costs deferred (no damageable items,
  no item names, no anvil GUI — the verified constants are recorded in
  anvil.rs for the tools/armor bracket).
- **Lava fluid** (`fluids.rs`): the LAVA block (light 15, fluid) + flow
  levels; dimension-aware spread (Overworld/End: level drop 2 → 3 blocks
  per 30-tick step; Nether: drop 1 → 7 blocks per 10-tick step);
  source-removal drains; meshes through the fluid-quad path with the
  fixed lava tint (SLOT_LAVA); contact damage 4 HP per 10 ticks (the
  half-second immunity window). Post-lava fire (300 ticks) deferred —
  no fire system.
- **Emerald ore generation**: Mountains-family columns only (the engine's
  per-column biome gate), single blocks, y 4–31, hash-gated ~a few per
  chunk; drops 1 emerald + the ore's 3–7 XP (existing ore-XP path);
  Fortune deferred.
- **Foods**: potato (0.5 HP), carrot (1.5), baked potato (2.5, smelted
  from potato), pumpkin pie (4.0) — heal = hunger/2 per the engine's
  food convention (steak 8 hunger → 4 HP); pumpkin pie picker-only
  (recipe needs sugar + egg, absent — documented).
- **Blocks/items registry**: cobblestone wall (craft 6→6, fence-class),
  flower pot (craft 3 bricks-blocks, cross-rendered), item frame (craft
  8 planks + leather — the stick adaptation), tripwire hook (craft → 2),
  wither-skeleton skull (cross-rendered summon component), command
  block (creative-pick only), emerald + nether star items; 4 new spawn
  eggs (kinds 17–20); 29 new clean-room art tiles (e2_art.rs); creative
  picker + E2E `give:` entries.
- **Mechanical fix (VERIFICATION-REPORT)**: render-distance slider range
  2–32 (was 2–16).
- **Latent E1 bug fixed**: `World::set_block` stored raw block IDS as
  states (END_PORTAL placed via set_block read back as FURNACE;
  DRAGON_EGG as REDSTONE_TORCH; OAK_SLAB as OAK_LOG[axis=x]). It now
  routes through `default_state`, matching the generator-side rule.

### Verified

- Every constant above carries a `VERIFIED w/<page>` comment from this
  round's live wiki fetches (research doc above; raw JSON archived under
  `tool-results/phase2/`).
- Suite: **372/372 green** (was 342; +30: wither fight timeline/charge/
  regen/skull cadence/death, summon pattern, effects periods/stat
  modifiers, beacon pyramid/range/levels/reapply/duration, adventure
  mode rules, anvil ladder/falling formula/12% gate, lava
  rates/spread-by-dimension/drain, anvil gravity, emerald
  mountains-only, registry folds).
- wasm32 target clean (`--no-default-features --lib`).

### Placeholder-unresolved

- Witch spawn weight implemented as a 1/100 roll ≈ the verified 5/515
  (~0.97%) share — the engine's 5-kind monster roll has no weight table
  (disclosed approximation of a VERIFIED number, not an unverified one).

### Deferred (with disclosure in code + here)

- Book and Quill (no paper/ink items, no text editor GUI).
- Pumpkin pie recipe (needs sugar + egg items).
- Anvil repair/combine/rename GUI + falling-anvil entity damage (no
  damageable items; the falling-block sim is block-wise without fall
  tracking).
- Item frame contents/rotation, flower pot planting, tripwire circuit
  signaling, command block execution (blocks + recipes exist; the deep
  wiring rides later brackets — item-frame entity storage, redstone
  signal routing, the command bridge).
- Charged-creeper mob heads (no charged creepers yet — wither skeleton
  skull IS in via its 2.5% drop).
- Wither "wither armor" below half health (projectile immunity — the
  engine's arrows route through melee damage).
- Post-lava fire ticks (no fire system).

### Known issues & regressions

- None observed: 372/372, wasm clean, no new clippy lints (the
  pre-existing `never_loop` in vc-pack remains).
- The wither's block-breaking on damage can carve terrain fast in a
  long fight (vanilla-accurate behavior; the 3×4×3 box is the VERIFIED
  rule).

## 2026-09-06 — verification follow-up: mechanical priority items 3+4 (+ item-1 slider completion) — commit 3b56274

**Task:** close the remaining mechanical items from
`docs/VERIFICATION-REPORT.md`'s priority list. Items 1 (render distance)
and 2 (lava fluid) had already folded into the E2 bracket; this round
implements **item 3 (sprint-jump 7.127 b/s emergence)** and **item 4
(coal item + 1600-tick fuel)**, plus the options-screen slider mapping
that item 1's E2 fix had missed.

### Implemented

- **Sprint-jump mechanics** (`player.rs`): the vanilla input — jumping
  while sprinting accelerates the player **+0.2 blocks/tick toward their
  facing** (VERIFIED live: mcpk.wiki/wiki/Sprinting "when the player
  jumps while sprinting, they accelerate by 0.2 towards their facing";
  minecraft.wiki/w/Jumping "jumping can be combined with sprinting to
  increase the player's movement speed") — now exists as
  `SPRINT_JUMP_BOOST = 4.0` (0.2 b/t × 20), applied at the jump.
- **Excess air drag** (`SPRINT_JUMP_EXTRA_DRAG = 2.2`): a documented
  adaptation. Vanilla gets the sustained 7.127 b/s figure from its
  0.91×/tick air drag on an impulse model; this engine's smoothed
  velocity model decays the speed EXCESS over the movement target at
  the tuned rate instead (calibrated, not guessed: measured 7.129 b/s
  with the 60 Hz test harness).
- **Emergence test** `sprint_jump_averages_vanilla_7_127`: 4 s settle +
  30 s measured displacement on a 3×19-chunk flat runway (the shared
  3×3 test world runs out of floor in 4 s at sprint-jump speed);
  asserts |avg − 7.127| < 0.1, avg > sprint+0.5, y stays in the
  jump-corridor. VERIFIED live: minecraft.wiki/w/Sprinting "jumping
  while sprinting allows the player to move with an average speed of
  7.127 m/s"; w/Transportation "Sprint-jumping, flat terrain, 7.127
  m/s".
- **The coal item** (id 162, tile 206, state 316 — the E2 item-block
  pattern): `COAL` with `fuel_ticks = 1600` (VERIFIED live:
  minecraft.wiki/w/Furnace "a piece of coal burns for 80 seconds and
  can process eight items"; w/Smelting fuel table "Coal 1600 ticks /
  8 items"), clean-room lump art in `e2_art.rs`, picker entry.
- **Coal ore → coal smelting recipe** (`smelt_result(COAL_ORE) =
  Some(COAL)`, `smelt_xp 0.1` — VERIFIED live: w/Smelting "smelting 1
  coal ore and removing the coal, the value is 0.1"). This is how coal
  is obtained in survival.
- **The COAL_ORE 800 ore-as-fuel stopgap is retired**: vanilla coal
  ore is not a fuel; the stand-in sites swapped to the real item —
  wither-skeleton drop `(COAL, 1)`, the three villager "buys coal"
  trades (armorer/toolsmith/weaponsmith), the dungeon-chest loot entry.
- **Registry ripple** (all guarded by tests): BLOCK_COUNT 163,
  STATE_COUNT 317, `COAL_STATE = 316` wired through `default_state` /
  `state_block` / `is_model_state` / the prop-roundtrip test; the WGSL
  mesh-compute LUT offsets resynced (L_FL 317 / L_TC 480 / L_ST 643,
  sb clamp 316 — `wgsl_lut_offsets_match_rust` guards the pair).
- **RD slider completion**: `apply_slider(ID_OPT_RD)` and both
  `refresh_widgets` inverse mappings now use the 2–32 range
  (2 + t·30 / (rd−2)/30) — the E2 round had fixed the clamp and the
  in-game ± keys but left the options screen itself mapping 2–16.

### Verified

- Live, this round: 7.127 b/s (w/Sprinting, w/Transportation);
  +0.2 b/t sprint-jump boost (mcpk.wiki/w/Sprinting); coal 1600 t /
  80 s / 8 items (w/Furnace, w/Smelting, Template:Smelting_table);
  coal ore → coal 0.1 XP (w/Smelting). Search transcripts saved under
  `scripts/verify_*.json`.
- Suite: **375/375** green (372 → +3: the emergence test, the
  eight-items-per-coal test, the ore→coal recipe test), wasm32 lib
  target clean in both feature configs, no new warnings, zero
  todo!/unimplemented!/unsafe in the touched files.

### Deferred (with disclosure)

- Sprint-jumping's 4× hunger cost (w/Jumping "A single jump while
  sprinting costs four times as much hunger as a normal jump") — the
  engine has no exhaustion/sprint-hunger system yet; noted for the
  hunger milestone.
- 45° diagonal sprint-jumping (vanilla is ~2 % faster again) — out of
  scope for the straight-line observable; revisit with a turning model.
- Block of Coal (16000 ticks / 80 items, live value recorded) — a
  block + recipe, not just an item; rides a later bracket.

### Known issues & regressions

- None observed: 375/375, wasm clean, WGSL LUT drift guard green.

---

## 2026-09-06 — MC 1.5–1.6 bracket (version-evolution Phase 3: Transport & Building) + full worklog↔evolution audit — commit 420818b

**Task:** third bracket of the 1.0 → 1.16.5 version-evolution ordering
(`evolution-research.md` Part 3 Phase 3, the Redstone/Horse updates),
plus the user-requested audit of everything implemented from the
worklog to the evolution plan. All values live-verified this round
against minecraft.wiki — search transcripts saved under
`voxelcraft/scripts/verify_e3_*.json` + `scripts/e3_page_*.json`
(~35 live citations in the code comments this round).

### The audit (worklog ↔ evolution ↔ code)

- **Suite**: 375/375 pass on the pre-E3 tree (matches every worklog
  claim; run with `--no-default-features` on this box — alsa-sys needs
  missing ALSA headers, the audio feature gates rodio only).
- **Everything the worklog claims exists in code**: E1/E2 module line
  counts match (dragon.rs 444, e1_art.rs 747, wither.rs 430,
  beacon.rs 349, effects.rs 305); BLOCK_COUNT/STATE_COUNT/COAL_STATE
  matched the claimed 163/317/316; 575 VERIFIED citations; zero
  todo!/unimplemented!/unsafe; 15 biomes (14 + MushroomFields);
  6 structures; local == origin/main (0 ahead, clean tree); CI
  workflows present (ci/release/wasm-build).
- **Audit finding #1 — SUPERFLAT (1.1 item) was silently absent**:
  never implemented, never deferred. FIXED this round (below).
- **Audit finding #2 — the evolution doc's 1.5 "already have" row was
  wrong**: daylight sensor, trapped chest, weighted pressure plates,
  block of redstone, and activator rail were NOT in the code (only
  comparator/dispenser/dropper/hopper). Four of the five land this
  round; activator rail is deferred (no rail/minecart system in the
  engine — riding arrives with horses instead).
- **Audit finding #3 — the "Wither Spawn Egg" (egg index 19) stub**:
  `from_egg(19)` has no arm → falls through to Chicken (the wither is
  a boss entity outside MobSystem; pre-existing E2 behavior, now
  documented at the from_egg NOTE).
- The user's 7 reference screenshots were VLM-reviewed (Options /
  Video Settings / Resource Packs / Select World / jungle F3 / creative
  inventory / survival inventory) — they inform the still-open visual
  priority list (GUI scale, #C6C6C6 theme, 176-wide panels, armor
  slots + player model, F3 right column) in VERIFICATION-REPORT.md.

### Implemented

- **Blocks/items (37 new ids, BLOCK_COUNT 163→200, STATE_COUNT
  317→400)**: Block of Coal (fuel 16000 t = 80 items, 9↔ coal
  crafts); Block of Quartz (4 quartz) + Chiseled Quartz (picker-only —
  no quartz-slab system) + Quartz Pillar (2 blocks → 2 pillars);
  16 stained terracotta (vanilla dye order); 5 carpets (the engine
  wool palette, 1/16-block non-solid overlay adaptation); Hay Bale;
  Daylight Sensor; Trapped Chest; Light/Heavy Weighted Pressure
  Plates; Block of Redstone; Nether Quartz item (the quartz-ore drop);
  Lead; Saddle; 3 spawn eggs (horse/donkey/mule at ids 197..=199,
  kinds 20..=22 — the legacy 124..=143 egg window was full).
- **POWER-state architecture (the round's key design fix)**: the
  first cut fed signals straight into wire states — the stateless
  wire re-derivation ERASED them on the next tick (caught by the new
  unit tests). Redesigned the vanilla way: sensor power 1..15
  (states 355..=369), trapped-chest OPEN (354), plate powers
  (370..=399) live in blockstates; `power_at`/`direct_feed` read them
  as real sources. Wire re-derivation now agrees by construction.
- **Horse/Donkey/Mule** (mobs.rs): per-instance stats (health 15–30,
  speed 0.1125–0.3375 internal / donkey-mule 0.175, jump 0.4–1.0,
  20% babies); temper taming (threshold 0–99 at first mount, +5 per
  failed mount); saddle gates control; ridden mount's AI suspends
  (physics still ticks); the ride drive steers at attr×43.17 b/s with
  the jump launch velocity solved by binary search over the engine
  integrator to hit the jump-strength clear height (the quadratic fit
  through the three VERIFIED anchors 0.4→1.153 / 0.7→3.124 /
  1.0→5.9197); breeding via golden apple on two tamed adults (foal
  stats via the VERIFIED 5-step bred formula; horse×donkey → mule);
  hay feeds/heals; plains herds (5/46 ≈ 1/9) + savanna (1/52 ≈ 1/26
  split horse/donkey), herds 2–6; drops 0–2 leather + 1–3 XP + the
  saddle when equipped.
- **Lead**: item; right-click a mob → leash (1.16.5 stretch max 10
  blocks — version-scoped: the current wiki's 12 is the 2025
  "Chase the Skies" buff); right-click a fence → knot anchor; pulled
  toward the anchor past 4 blocks; breaks at 10 + drops the item;
  re-use on the mob unleashes.
- **Redstone components**: daylight sensor (sky light × day-phase
  brightness, self-rescheduling every 20 gt); trapped chest (1 viewer
  while the GUI is open, back to 0 on close — wired at open/close);
  weighted plates (entity-count sweep every 10 gt: light = count,
  heavy = ceil(count/10), max 15); block of redstone (always-on weak
  15 in power_at + direct_feed).
- **Superflat** (audit finding #1): TerrainGen flat mode + the Gen
  job carries the flag + the WORLD TYPE button in world-create now
  cycles NORMAL/SUPERFLAT (was a disabled "NORMAL" stub). Classic
  preset: bedrock + 2 dirt + grass at y=3, plains, no structures
  (JE village/stronghold generation disclosed as out of scope).
- **Badlands terracotta banding**: the surface + top-16 strata band
  through the stained colors by absolute y with a per-seed offset
  (vanilla's exact seed-shifted layer table is unpublished —
  deterministic clean-room banding, disclosed).
- **Hay fall-damage reduction**: landing on a hay bale takes 20% of
  the normal damage (player.rs landing site).
- **WGSL mesh LUT resync** (twice — once per STATE_COUNT change):
  L_SB/L_FL/L_TC/L_ST = 0/400/600/800, sb clamp 399, fl clamp 199;
  `wgsl_lut_offsets_match_rust` green.
- 29 new clean-room art tiles (e3_art.rs); carpets reuse the wool
  tiles; picker widened to 15 columns (164 entries, 668×514 grid).

### Verified

- Live this round: coal block 16000 t/80 items (w/Block_of_Coal);
  quartz family + recipes (w/Block_of_Quartz, w/Quartz_Pillar, a
  2nd source for the output count, w/Chiseled_Quartz_Block,
  w/Nether_Quartz_Ore drops + 2–5 XP); carpets 2 wool → 3 + 1/16
  hitbox (w/Carpet 13w17a/14w29a); terracotta 16 colors + badlands
  (w/Terracotta, w/Badlands); hay −80% fall damage (w/Hay_Bale);
  daylight recipe + signal factors (w/Daylight_Detector); trapped
  chest recipe + viewers-signal (w/Trapped_Chest); plate formulas
  (w/Light_Weighted_Pressure_Plate + the heavy page); redstone block
  weak-15 (w/Block_of_Redstone); horse stats/taming/breeding/spawning
  /drops (w/Horse §Health/§Movement_speed/§Jump_strength/§Taming/
  §Bred_values/§Spawning/§Drops, w/Donkey, w/Mule); lead 10 blocks in
  1.16.5 (w/Lead + §History — the version-scoping catch); superflat
  classic preset (w/Superflat); horse 0–2 leather (search round).
- Suite: **396/396 green** (375 → +21: registry roundtrips + counts +
  picker, coal-block fuel + burn-outpaces-output, 6 recipe families,
  day-brightness curve, plate formulas, redstone-block wire power,
  daylight-sensor day/night, trapped-chest open/close, superflat
  layers, badlands banding, horse spawn stats, temper taming, saddle
  gating, bred-stat formula, jump-clear anchors, foal kind rules,
  ridden-AI suspension). wasm32 lib target clean in both feature
  configs; zero todo!/unimplemented!/unsafe.
- Environment note: this container cannot run the audio backend
  (alsa-sys needs missing system headers) — native build/test runs
  `--no-default-features` (audio is feature-gated; no test coverage
  lost). Browser E2E for the riding/lead flows rides the next
  wasm-bundle round (CI auto-rebuilds it on push).

### Placeholder-unresolved

- **Quartz pillar output count 2**: the wiki recipe table shows
  "Block of Quartz 2" (count column unreadable in the text extract);
  a second live source states "produces 2 Quartz Pillars per craft" —
  implemented as 2 with both citations, flagged as lightly-sourced.
- **Badlands band sequence**: vanilla's per-seed layer table is not
  published; the clean-room orange-dominant strata sequence is
  deterministic and disclosed as an approximation.

### Deferred (with disclosure in code + here)

- **Activator Rail** (1.5): the engine has no rail/minecart system
  (riding ships with horses this bracket); riding a later
  transport bracket if rails land.
- **Scoreboard** (1.5): its whole interface is the command system —
  the engine has no commands yet (74 = 0 implemented, the evolution
  table's own row).
- **Name Tag** (1.6): requires anvil renaming (the anvil GUI +
  damageable-items deferral from E2).
- **Horse armor** (iron/gold/diamond): no armor items in the engine.
- **Donkey/mule chest storage** (15 slots, VERIFIED number recorded):
  needs chest-item + per-mob container UI; rides the container pass.
- **Chiseled quartz + hay bale recipes** (2 quartz slabs / 9 wheat):
  no quartz-slab model, no wheat/farming.
- **Stained terracotta crafting** (terracotta + dye): no dye system —
  Badlands banding is the acquisition path.
- **Lead recipe** (4 string + 1 slimeball): no slimeballs (no slimes);
  picker + item exist.
- Vanilla horse traits (jump-charge hold, saddle-less steering
  prohibition is faithful; the rider's +7-block safe-fall rides the
  mount's landing instead — disclosed in game.rs).

### Known issues & regressions

- None observed: 396/396, wasm clean, no new clippy lints beyond the
  pre-existing set (the vc-pack `never_loop` + pre-existing unused
  warnings). The first-cut E3 redstone design (feeding wires
  directly) was caught and redesigned BEFORE commit by the new
  unit tests — the POWER-state architecture is the vanilla pattern.

---

## 2026-09-06 — version bracket 1.7.2 ("The Update that Changed the World") — Phase 1.7

**Task:** user directive — continue the remaining work as version phases
from the current position (1.7) through 1.10, checking every change in
detail (mechanics AND visuals) against live sources, implementing the
bracket content, and reporting parity per change.

**Live verification round (minecraft.wiki, 2026-09-06):** full changelog
pages fetched and parsed for 1.7.2, 1.8, 1.9, 1.10 (scripts/verify/*.txt)
plus targeted pages for Fishing (85/10/5 roll, 5–30 s wait, Lure −5 s/level
off both bounds), Poison (L4 = 3 ticks/HP raw, 10-tick hurt-immunity
effective floor, cannot kill — floors at 1 HP), and the new-biome color
pages (Flower Forest #79C05A, Dark Forest #507A32, Sunflower Plains
#91BD59, Ice Spikes #80B497/#60A17B). Key doc correction caught: the
evolution-research plan lists Stray under 1.9 — the live 1.10 page puts
strays in 1.10 (with husks and polar bears).

**Registry foundation (the V2 window):**
- State space extended past the historical 255 ceiling: block ids
  103..=161, dedicated states 236..=294 (+4 log-axis states 295..=298 for
  acacia/dark oak). `Chunk::get` now FOLDS states through `state_block`
  (the old `as u8` truncation would alias high states); a new raw
  `Chunk::get_state` accessor serves `World::get_state`/
  `set_block_state`. All historical double-fold call sites fixed (gen.rs
  nether decorations + tests, light.rs column scans, anvil test, the
  Phase-10 pyramid test).
- GPU mesher LUT re-derived (STATE_COUNT 236→299, BLOCK_COUNT 103→162;
  WGSL offsets + clamps updated; tint classes 7/8 for the new leaves).
  Caught a sneaky compile trap: un-imported `ACACIA_LEAVES` in gpu_mesh.rs
  silently became a *binding* match arm (class 7 for EVERY block) —
  fixed by importing the constants; the LUT-mirror test caught it.

**1.7.2 content implemented:**
- 59 new registrations: 16 stained glass, 16 stained terracotta, red
  sand, packed ice (OPAQUE — the changelog's signature difference vs
  ice), podzol, acacia/dark oak log+leaves (leaves reuse the oak tile —
  the changelog itself says both are "visually identical to regular oak
  leaves"), 8 small flowers, 4 two-block flowers as lower+upper id pairs,
  4 fish items (raw fish/salmon/clownfish/pufferfish).
- 60 new procedural clean-room tiles (glass tints, terracotta grain, red
  sand, packed-ice fractures, podzol, acacia "silver outside, orange
  inside" bark, dark oak near-black bark, all flower art, fish icons).
- 4 new biomes + worldgen: Flower Forest (dense new-flower flora, no
  sunflowers), Sunflower Plains (sunflower pairs), Ice Spikes (packed-
  ice spires 5–15 tall + snow-block surface), Dark Forest (dense 2×2
  dark-oak trunks). Badlands: red-sand floor over SEVEN banded terracotta
  colors ("normal, orange, red, yellow, white, light gray and brown" per
  the changelog) with jittered band edges. Taiga: mega-taiga podzol
  patches. Acacia trees: vertical base + diagonal (axis-state) segment +
  flat disc canopy.
- Tint parity: new-biome grass/foliage/water colors (live-verified hex),
  acacia/dark-oak leaves biome-foliage-tinted; tint LUT loop extended
  0..18 — which FIXED a latent Phase-10 bug: biomes 8..=13 were never
  written into the shader LUT, so taiga/jungle/savanna/swamp/badlands
  grass rendered untinted (white) since Phase 10.
- Mechanics: fishing loot system (85/10/5 fish/junk/treasure, Lure wait
  math, Luck of the Sea monotone treasure shift — vc-gameplay/fishing.rs
  with the full vanilla table shapes including palette-missing rows as
  named placeholders); pufferfish eating applies Poison IV 1:00 with the
  10-tick observable cadence and the 1-HP cannot-kill floor (new
  StatusEffects on the player, ticking in the fixed 20 Hz step); raw
  fish/salmon/clownfish join the eat path; red sand smelts to glass.
- Creative picker: +55 entries (123 blocks), grid widened 8→12 columns
  to stay inside the 540-px UI canvas.

**Verification:** 325/325 tests green (299 library + 26 game-crate;
was 310, +15: fishing ×6, worldgen ×6, poison ×3). The game crate's test
run uses `--no-default-features` in THIS container only — ALSA dev
libraries are absent (no root), so the rodio audio backend can't link
locally; CI builds it normally. Two pre-existing tests updated for the
new `Chunk::get` fold contract (anvil foreign-chunk, pyramid chest pit).

**Deferred (documented, carried to later brackets/registry phases):**
dye items + stained-glass/clay crafting recipes (dye economy absent),
fishing rod/bow/name-tag/bowl/stick/lily-pad/saddle items (loot rows
listed as placeholders), acacia/dark-oak planks, saplings, tall-grass/
fern bone-meal growth, infested block variants, grassless dirt
(1.8 coarse dirt supersedes), minecart-with-command-block, /tellraw /
/summon / /setblock / /testforblock commands (no chat-command system),
stained-glass panes, custom 23×23 nether portals, pufferfish→Water
Breathing brewing (brewing stands take block-id ingredients; pufferfish
item is now in the registry for a future recipe), 1.7 sound set.

**Commit:** this entry (bracket 1.7.2).

---

## 2026-09-06 — version bracket 1.8 ("Bountiful Update") — Phase 1.8

**Live verification:** minecraft.wiki/w/Java_Edition_1.8 parsed (2026-09-06)
+ targeted live checks for rabbit (3 HP, "avoid all players within 8
blocks", 0–1 raw rabbit + 0–1 hide, 10% rabbit's-foot player-kill roll).

**V3 registry window:** ids 162..=180, states 299..=317 (after the V2
log-axis states), STATE_COUNT 318, BLOCK_COUNT 181. GPU mesher LUT
re-derived (WGSL offsets 318/499/680 + clamps).

**1.8 content implemented:**
- Blocks: slime block (translucent), coarse dirt, polished
  granite/diorite/andesite, red sandstone + smooth variant, prismarine ×3,
  sea lantern (emissive 15, wiki-verified), iron trapdoor, barrier
  (near-invisible solid — the wiki's "completely transparent").
- Items: raw/cooked rabbit, rabbit hide, rabbit's foot, prismarine shard +
  crystals.
- Rabbit mob: 3 HP, skittish AI (bolts within 8 blocks of the player —
  the wiki's avoidance rule), joins the passive herd roll, drops 0–1 raw
  rabbit + 0–1 hide + the 10% foot roll, sprite + item art.
- Slime-block bounce physics: landing on slime negates fall damage and
  rebounds at the wiki's "up to 60% of initial height" ratio
  (v = sqrt(0.6)·impact); sneaking keeps the damage and cancels the
  rebound, exactly per the changelog.
- Spectator mode: GameType 3 round-trips through the save schema;
  always-flying, no-clip (move_axis bypass), no break/place/use, mob hits
  absorbed (invulnerable), not offered in the create-screen cycle
  (vanilla enters it only via /gamemode — which this engine lacks,
  documented).
- Worldgen: coarse-dirt patches in savanna (the 1.8 replacement for 1.7's
  grassless dirt), red-sandstone filler directly under badlands red sand.
- Recipes: 2×2 polished trio, 2×2 dirt+gravel checker → 4 coarse dirt,
  2×2 red sand → red sandstone, 2×2 shards → prismarine, 2×2 crystals →
  sea lantern (all per the 1.8 changelog text). Smelting: red rabbit →
  cooked rabbit.
- Picker: +19 (142 blocks).

**Verification:** 328/328 tests green (301 lib + 27 game; +3:
slime-bounce physics, spectator rules, rabbit data). The 1.7 badlands
banding test updated for the new red-sandstone filler (it now checks the
filler explicitly).

**Deferred (documented):** guardians + elder guardians + ocean monuments
(the era's flagship structure — beam attack, Mining Fatigue aura and
monument worldgen are a full phase of their own), armor stands, banners,
endermite, wet sponge, wood-specific doors/fences/fence gates, world
border + /clone /fill /title /execute /trigger /stats commands (no
command system), enchanting-lapis rework, customized/debug world types,
rabbit stew + Potion of Leaping (brewing needs the rabbit's-foot recipe
hook), door 3-tall models.

**Commit:** this entry (bracket 1.8).

---

## 2026-09-06 — version bracket 1.9 ("Combat Update") — Phase 1.9

**Live verification:** minecraft.wiki/w/Java_Edition_1.9 parsed
(2026-09-06): blocks (grass path 15/16 + shovel-use, purpur family, end
stone bricks, end rods "same brightness as torches", chorus plant/flower),
items (chorus fruit 4-hunger + random teleport, elytra "hang glider
aerodynamics" + chest slot, shield 6 planks + 1 iron), the combat
mechanics list, and the Elytra §Flight 10:1 glide ratio claim.

**Prior state confirmed (✅ already 1.9):** the engine's combat.rs was
built on 1.9 formulas from the start — attack cooldown `0.2 + 0.8p²`,
`20 / attack_speed` ticks, crits ×1.5 at ≥84.8% charge + falling +
not-sprinting, armor-toughness damage reduction, difficulty scaling.
Frost Walker and Mending are in the 38-enchant registry (pinned by a new
test).

**V4 registry window:** ids 181..=190, states 318..=327, STATE_COUNT 328,
BLOCK_COUNT 191, GPU LUT re-derived.

**1.9 content implemented:**
- Blocks: grass path (trodden top + lip side; full-cube simplification
  documented), purpur block + pillar, end stone bricks, end rod
  (emissive 14), chorus plant + flower.
- Items: chorus fruit, elytra, shield (clean-room art).
- Elytra GLIDE: while the selected item is the elytra and the player is
  airborne, falling, holding jump — horizontal velocity steers toward
  the look vector up to 25 b/s with descent clamped to 2.5 b/s,
  preserving the wiki's 10:1 glide ratio (documented adaptation: vanilla
  uses chest-slot equipping + pitch-driven per-tick aerodynamics; no
  armor slots or firework boost in scope).
- Shield BLOCKING: while the shield is selected and right-click is held,
  mob melee and arrows are absorbed entirely (adaptation: vanilla's
  partial-damage window, axe-disable and deflection angles deferred).
- Chorus FRUIT: eats (4 HP, our hunger-less deviation), then the vanilla
  teleport — up to 16 attempts in a ±8 cube for a grounded 2-air spot,
  enderman teleport pop.
- Grass path: block + picker + recipe path documented (survival
  obtaining needs the shovel item — tools are a deferred registry).

**Verification:** 331/331 tests green (302 lib + 29 game; +3: glide-ratio
constants, elytra gate, shield/elytra/enchant registration pins).

**Deferred (documented):** the End overhaul (outer islands, end cities,
end ships, end gateways, shulker + shulker boxes, dragon-fight rework —
the End DIMENSION itself is the still-open 1.0 bracket), dual wielding /
offhand slot (inventory rework), lingering potions + tipped/spectral
arrows (needs splash-potion brewing + arrow effects), dragon's breath,
grass-path shovel interaction (no tool items), shield crafting recipe
and axe-disable.

**Commit:** this entry (bracket 1.9).

---

## 2026-09-06 — version bracket 1.10 ("Frostburn Update") — Phase 1.10

**Live verification:** minecraft.wiki/w/Java_Edition_1.10 parsed
(2026-09-06 round, saved as scripts/verify/page_110.txt) + fresh live
rounds for /w/Magma_Block §Damage (api.php wikitext fetch) and the
Polar_Bear/Stray/Husk rows. Full-changelog sweep below.

**V5 registry window:** ids 191..=194, states 328..=331, STATE_COUNT 332,
BLOCK_COUNT 195. GPU mesher LUT re-derived (WGSL offsets 332/527/722 +
clamps 331/194).

**1.10 content implemented (each item checked against the live page):**
- Blocks: magma block (light 3 — wiki /w/Magma_Block; contact damage 1 HP
  per second per the 1.10 changelog "Mobs and players take 1 HP damage
  every second while touching it"; sneaking immune — "If the player is
  sneaking ... they do not take damage"; Frost Walker / Fire Resistance
  immunity documented out-of-scope, no boots/fire-effect registries yet;
  side-contact does not damage — "Walking into the side of a magma block
  doesn't cause damage"; death message "DISCOVERED FLOOR WAS LAVA" per
  the changelog's "[Player] discovered floor was lava."), nether wart
  block, red nether bricks, bone block.
- MAGMA contact-damage wiring: per-frame feet-below probe + 1-s
  accumulator → pending_magma_dmg drained by the game layer (creative
  invulnerable). FOUND+FIXED during this bracket's audit: the in-progress
  code had declared pending_magma_dmg/magma_accum but never SET them —
  dead code, zero damage at runtime. Also noted: the modern /w/Magma_
  Block page describes a per-tick/half-second cadence (damage-immunity
  gated) — a later-bracket value; the 1.10 changelog's per-second rate is
  the bracket-correct one, re-verify at the bracket where it changed.
- MAGMA FLOWING ANIMATION (visual): the changelog's "Has a flowing magma
  animation" — clean-room 4-frame shimmer registered as a BUILT-IN
  AnimatedTile (pulses only r>140 crack pixels, frametime 8 ticks, frame
  0 == the atlas tile for a seamless loop), independent of resource
  packs; rides the §20 update_atlas_frame path (no geometry rebuild).
- Worldgen: nether magma blobs ("4 blobs per chunk between Y=27 and
  Y=36", embedded in netherrack only — never floating); fossils
  ("generates 15–24 blocks underground in deserts, swampland and their M
  and hills variants. Each chunk has a 1/64 chance", "composed of bone
  blocks and some coal ore" — skull 3×3 with coal eye sockets + spine
  chain).
- Mobs: polar bear (30 HP, neutral-not-hostile, 4/6/9 HP melee by
  difficulty → 6 base, icy-family spawner, drops "0–2 raw fish (75%
  chance) or 0–2 salmon (25% chance)"), stray ("80% of skeletons spawned
  above ground in ice plains, ice mountains and ice plains spikes biomes
  are strays"; arrows apply Slowness 600 ticks = 0:30; "50% chance to
  drop 1 tipped arrow of Slowness when killed by the player" — our
  adaptation drops a plain arrow until a tipped-arrow registry exists),
  husk ("80% of zombies spawned above ground in desert ... are husks";
  melee applies Hunger for 7 × floor(regional difficulty) seconds —
  regional-difficulty proxy is the difficulty tier, documented; "does not
  burn in sunlight" — trivially satisfied, no zombie sunlight-burning
  system exists at this bracket).
- Icy-biome passive restriction (wiki §World generation: ice plains /
  ice mountains / ice plains spikes "don't spawn any passive mobs other
  than rabbits and the new polar bears"): the passive herd roll now
  yields ONLY Rabbit + PolarBear in biomes 5/16. FOUND+FIXED during the
  audit: the in-progress code still rolled cow/pig/sheep/chicken there
  70% of the time. (The same section's 7%-vs-10% worldgen-pass rate is
  N/A — we have no worldgen animal pass, documented.)
- Auto-jump ("A new 'Auto-jump' toggle ... automatically makes the player
  jump when running towards a one-block-tall obstacle. Enabled by
  default; can be disabled in options" — from Pocket Edition): Settings
  toggle default ON + AUTO-JUMP options button + player hop. FOUND+FIXED
  during the audit: two bugs in the in-progress hop — (1) autojump_cd
  was set but never decremented (one hop then locked), (2) the hspeed
  gate read the velocity AFTER the blocked move zeroed it, so a pressed
  player could never re-hop, and the hop missed the manual jump's
  tick_accum phase reset (apex 0.85 instead of 1.25 — feet never cleared
  the step). Fix: probe along the WISH direction gated on has_input +
  tick_accum reset; pinned by test auto_jump_hops_one_block_step.

**Verification:** 340/340 tests green (308 lib + 32 game; +9 over the
1.9 bracket: v5 window + magma light pin, Frostburn mob data, magma
damage 1 HP/s, magma sneak immunity, auto-jump hop, magma builtin
animation, nether magma blobs, fossils, + the nether-mass test updated
to admit magma).

**Deferred (documented, with reasons):**
- Structure blocks + structure voids (the bracket's headline feature):
  a creative/technical save-load-structures system (GUI, 4 modes,
  32-block limit, redstone activation) — no structure-system scope in
  the engine; full phase of its own if ever taken.
- All four 1.10 crafting recipes: magma block (4 magma cream — no magma
  cream item; magma cubes don't exist yet), nether wart block (9 nether
  wart — no nether wart crop block; brewing uses a documented red-
  mushroom substitution), red nether bricks (2×2 checkerboard of nether
  brick + nether wart — no nether brick BLOCK either), bone block (9
  bone meal; reverse 1 → 9 — no bone meal item). Also the 1.10 recipe
  FIX "End stone bricks now again gives four blocks instead of one" —
  no END_STONE block exists (End dimension is the open 1.0 bracket), so
  no recipe to fix yet.
- Spawn eggs (polar bear/stray/husk) — creative-mode items, no spawn-egg
  registry.
- Stray-on-spider jockeys, husk chicken jockeys / baby husks — no mob
  riding or baby-mob system.
- Looting interactions on the new drops (chance "2×level+1/2×level+2") —
  no Looting application to drops yet.
- Nether spawn-weight changes (endermen "1/153" vs pigmen "100/153",
  magma cubes "2/153 ... twice as often") — none of those Nether mobs
  exist in the registry yet (1.16.5-era content); re-check at the
  bracket that adds them.
- Husk/stray/polar-bear sound events + cave ambience (cave15/16) +
  splashes — synthesized bank uses generic hurt/step families; per-mob
  event names are a data-registry concern deferred.
- Magma behavioral details: mob pathing avoidance, the no-spawn-on-
  magma rule (exceptions magma cubes/pigmen/squid — none exist yet),
  water-removal-on-random-tick (N/A — magma only generates in the
  Nether, no water there in scope), smoke particles under rain.
- /teleport command, loot-table `limit` tag, FallFlying/ZombieType/
  ParticleParam NBT tags, fallingdust particle, F3+G chunk borders —
  no command system / datapack looting hooks / NBT schema for these /
  particle type / debug-outline renderer respectively.
- Changes-section items: dispenser-shield equipping, chorus-fruit/
  ender-pearl rider teleportation, fishing-rod item pulling (no mob-
  rider or item-entity fishing interaction), firework 3× recipe (no
  fireworks), skeleton off-hand tipped arrows + flaming arrows at
  regional difficulty ≥ 3, witch fire-resistance drinking, wolf
  no-despawn (no despawn system — trivially satisfied, documented),
  zombie fire-chance regional-difficulty rework (no burning zombies),
  mesa mineshafts (dark oak, MST type) + village wood variants (taiga
  spruce, savanna acacia, biome-boundary spread, blacksmith/well
  cobblestone swaps), plains 5%-tree worldgen, huge-mushroom 1/12 double
  height, hardened-clay rename ("Red Hardened Clay" — no stained clay
  exists yet, 1.6-era content), rails full-block bounding box.
- Visual: magma animation cadence/frame count is our adaptation (4
  frames, frametime 8); vanilla ships an 8-frame strip.

**Commit:** this entry (bracket 1.10).

---

## 2026-09-07 — merge completion: evolution e1–e3 (remote) ⊕ 1.7–1.10 (local) — the reconciliation round

**Task:** the 1.7–1.10 session built its four brackets on a stale base
(7b8b836) that predated the e1–e3 evolution commits; its final `git pull`
died mid-merge with 15 conflicted files and the session ended there. This
session completed the merge: resolved every conflict, fixed the semantic
collisions between the two lines, finished the half-done u16 registry
migration, and got the whole workspace green again before pushing.

**Starting state (honesty note):** local = 6 commits ahead (1.7.2 / 1.8 /
1.9 / 1.10 + docs), 13 behind (e1–e3 + verify rounds + CI); merge index
carried all three stages for 15 files; the prior session's hand-unioned
working tree compiled NOWHERE (unclosed delimiters, duplicate fields,
u8/u16 type splits, 152+ errors) — nothing was green.

### Implemented (the reconciliation)

- **WORKLOG.md conflict** → both sides kept, sections reordered
  chronologically (e1 → verify → e2 → priority-3+4 → e3 → 1.7.2 → 1.8 →
  1.9 → 1.10). LIBRARIES.md test-count conflict resolved to the real
  merged count.
- **game.rs / player.rs brace-loss unions repaired** — the MagmaCube
  split block lost 3 closing braces at the 1.10-Stray insertion point;
  the fall-damage region had the E3 hay-bale gate fused INSIDE the 1.8
  slime-bounce branch (double damage application) — reconstructed as:
  slime bounce takes precedence, else damage with the hay-bale ×0.2
  reduction inside the damage branch.
- **Effects system unified** (the biggest semantic collision): local
  1.7.2 `StatusEffects` (poison/slowness/hunger ad-hoc fields) DELETED in
  favor of the E2 `vc_gameplay::effects::Effects` table, extended with
  `Slowness` + `Hunger` kinds and `slowness_multiplier`. Poison period
  now `max(25 >> amplifier, 10)` — the live-verified hurt-immunity floor
  (w/Poison: L4 raw 3 ticks/HP, observable 10). Pufferfish eat =
  `apply(Poison, 3, 1200)` + `apply(Hunger, 0, 300)`; stray hit =
  Slowness 600 ticks; husk hit = Hunger 7×difficulty s. The separate
  1.7.2 tick call is gone — one effects tick in the E2 block, with
  poison-aware death cause + hurt sound. Beacon Speed (+20%/level) and
  the new Slowness (−15%/level) both now scale the movement target.
- **u16 registry migration completed** (the prior session started it,
  unstaged): block ids passed 255 states in the union, so `ItemStack`,
  container `kind`, anvil `ContainerMeta` (NBT Kind/Block → Short),
  `Job`/`JobResult` gen plumbing, `raycast`, combat `held_attack`/
  `player_melee`, drop tables, `is_food`/`food_heal`, fence probes, and
  ~20 test-site annotations all widened. `food_heal` had TWO bodies
  fused together (E2 match + 1.7.2/1.8 `matches!`) — reconstructed with
  the hunger/2 mapping for fish + rabbit.
- **Atlas grown 256²→512² (16→32 tile grid)**: merged TILE_MAX=325 broke
  the 256-slot atlas (index panic at TILE 256+). put/blit_tile/
  write_atlas_tile, both WGSL terrain passes (`% 16u`→`% 32u`, tile-UV
  divisor 16→32), and the CPU-side particle sub-tile UVs all moved to
  the 32-grid. `PACK_TILE_MAX` 255→1023 (it had silently locked ALL pack
  textures out of the atlas once TILE_MAX passed 255 — real bug, fixed).
- **Creative picker is now scrollable**: 236 entries × 15 cols no longer
  fit the 960×540 canvas (the prior session left the tail rows clipping
  as a "known issue"). Implemented the vanilla-correct fix: a fixed
  11-row window + mouse-wheel scroll (picker eats the wheel while
  open), scroll-aware hit-testing, top-of-grid on open. The E1 fit test
  now asserts the scroll-window invariant.
- **Badlands profile corrected**: the E3 code overrode the SURFACE with
  stained terracotta; 1.7.2's live-verified floor is red sand ("floor
  similar to a desert, but made of red sand"). Merged profile: red sand
  floor → 1.8 red-sandstone filler → E3 banded stained terracotta
  strata (16 deep). Both bracket tests now assert this one profile.

### Verified

- Suite: **426/426 green** (native; 396-e3-era + 340-f-era tests
  reconciled, duplicates merged, stale-registry expectations updated).
- wasm32-unknown-unknown: clean on the CI path
  (`cargo check --release --no-default-features --lib`) — the wasm
  bundle CI auto-rebuilds on push.
- All state windows re-checked against the merged table: E-series ends
  354; V2 400..=442 + log axis 443..=446; V3 447..=465; V4 466..=475;
  V5 476..=479; STATE_COUNT 480, BLOCK_COUNT 276, TILE_MAX 325 —
  non-overlapping (roundtrip tests pass on both sides of the mapping).

### Stale-test updates (registry-era invariants, not behavior changes)

- e1 picker fit → scroll-window invariant; e3 counts 200/400 → 276/480;
  v5 window 328..=331 → 476..=479; elytra state 326 → 474; MOB_DATA 22
  → 26 (+rabbit/stray/polar bear/husk); mip chain sizes → 512²-era;
  LUT tint arms `3|7|8` → `3|8|9` (the old arm swallowed the E2 lava
  class 7 before its own arm — merge artifact, unreachable-code bug in
  the test).

### Placeholder-unresolved / Deferred

- F-series spawn eggs (rabbit/polar-bear/stray/husk): vanilla 1.16.5
  HAS these egg items; the engine has none (egg_id 255 sentinel,
  roundtrip test skips them). Deferred until a later bracket touches
  the egg registry — disclosed, on the list.
- wgsl LUT drift test still pins L_TC = 756 (BLOCK_COUNT 276) — fine
  until the next registry growth.
- Headless environment note: this container has no rust toolchain
  preinstalled (installed via rustup this session) and no ALSA headers
  (user-prefix .deb extraction workaround, disclosed for
  reproducibility; CI unaffected).

### Known issues & regressions

- None new from the merge; the picker clipping "known issue" from the
  prior session is now FIXED (scrolling), not carried.

**Commit:** this merge commit (e9e9abf ⊕ fded9ef → unified tree).

---

## 2026-09-07 — Phase-1/2 evolution audit + audit-fix round (user-requested "especially the 1 and 2") — commit pending

**Task:** the user asked to verify the evolution work done so far is
accurate and complete — with explicit attention on Phases 1 and 2
(the e1/e2 brackets, which never got a dedicated "continue 1 and 2"
pass; only 3 and 4 were directed), then to re-check the standing
rules, then continue the bracket run. This round: the audit itself,
the fix round for what it found, and the push-discipline repair
(everything was 1 commit behind + a meaningless-UUID research commit
— amended to a real message and pushed before this round started).

### The audit (evolution-research.md Part 2/3 ↔ WORKLOG ↔ code ↔ tests)

- **Suite**: 426/426 green on the pre-round tree (matches the merge
  claim exactly; workspace-wide run, `--no-default-features`).
- **Zero todo!/unimplemented!/unsafe** — re-verified by grep across
  all crates.
- **Push state**: the branch sat 1 commit ahead of origin with the
  prior session's 1.11 research captures committed under a UUID
  message (`b9e2c68`) — amended to a descriptive research-commit
  message and pushed (`99b7f90`); docs/WORKLOG.md + docs/research/
  + docs/screenshots/ all confirmed tracked on the remote (102
  files). Commit→push discipline established from here on.
- **Phase e1 (1.0–1.2) cross-check — everything the WORKLOG claims
  exists in code**: End/dragon/fortress/mushroom-fields/mobs/XP/
  spawn-eggs/mycelium/lamp/sandstone variants confirmed via registry
  + module reads; sunrise/sunset colors ARE implemented (game.rs
  sunset fog band + render.rs horizon band + the day-brightness
  curve) even though the e1 WORKLOG never listed it — a
  documentation gap, not a code gap; Beach biome exists (Phase-10
  set); superflat was already caught + fixed by the e3 audit.
- **Phase e2 (1.3–1.4) cross-check**: wither/witch/bat/wither-
  skeleton, effects, beacon, ender chest, adventure mode, anvil
  ladder, lava fluid, emerald ore, foods, cobble wall, flower pot,
  item frame, tripwire hook, command block — all confirmed present
  with their cited constants.
- **Genuine audit findings — silently absent, never deferred (all
  FIXED this round or formally deferred below):**
  1. **Golden Carrot** (1.4 evolution-plan item) — missing entirely:
     not implemented, not deferred, not in the phase-2 research doc.
  2. **Jungle wood family** (1.2: "Jungle wood/leaves/sapling") —
     the Jungle biome existed as an oak-canopy ADAPTATION (disclosed
     only in a gen.rs comment); the block family never landed.
  3. **Vines** (1.2) and **ferns** (1.2) — absent; the only "fern"
     mention anywhere was the 1.7.2 bracket's "fern bone-meal
     growth" deferral.
  4. (Minor, formally deferred this round — see below.)

### Implemented (the fix round — all values live-verified 2026-09-07,
minecraft.wiki page captures archived under
`voxelcraft/scripts/auditfix_page_*.json` + one search round)

- **Registry V6 window (ids 276..=281, states 480..=485; BLOCK_COUNT
  276→282, STATE_COUNT 480→486, TILE_MAX 325→332, PICKER 236→242;
  WGSL LUT resync L_FL 486 / L_TC 768 / L_ST 1050 + clamp 485/281)**:
  GOLDEN_CARROT (276), JUNGLE_LOG (277), JUNGLE_LEAVES (278),
  JUNGLE_PLANKS (279), VINE (280), FERN (281).
- **Golden Carrot** — food 6 / 14.4 (VERIFIED w/Golden_Carrot
  infobox: "Hunger 6", "Saturation 14.4"; consumption 32 game
  ticks; added Java 1.4.2 12w34a per §History); heal 3.0 HP under
  the engine's hunger/2 convention; picker-only (craft = gold
  nugget + carrot — no gold nuggets in engine, documented);
  **equine feed**: golden carrot joins golden apple/hay in
  `try_feed` — love mode on two tamed adults (VERIFIED w/Horse
  §Breeding, the E3-round citation: "Feeding two tamed horses golden
  apples or golden carrots activates love mode") + the +4 heal arm
  (the engine's e3-verified per-food mapping; w/Golden_Carrot §Usage
  "used to tame, breed, lead, grow, and heal horses, donkeys, and
  mules").
- **Jungle trees** (gen.rs): the species switch now grows JUNGLE_LOG
  + JUNGLE_LEAVES in the Jungle biome (the oak-adaptation comment
  retired); 1×1 trunk height 5..10 (VERIFIED search round
  w/Jungle_Tree: "Regular jungle trees... 1×1 trunk, which can
  extend up to 10 blocks tall"; trees added 1.2.1 12w03a per
  w/Tree §History); **vines on trunks** (VERIFIED w/Vines: "Jungle
  trees of both sizes have vines on their trunks and canopy edges" —
  cross-rendered adaptation, ~60%/side); **jungle bushes** (~25% of
  jungle trees): "a single jungle log surrounded by oak leaves"
  (VERIFIED w/Tree) — the exact vanilla detail.
- **Vine physics** (player.rs): climbable "collisionless ladder"
  (VERIFIED w/Vines §History 12w04a + w/Vines: "Vines are climbable
  non-solid vegetation blocks that grow on walls"; "If there is a
  solid block behind the vines, the walk forward key can also be
  used"): up **2.35 b/s** (VERIFIED w/Ladder §Climbing "moves
  upward at about 2.35 blocks per second"), descent capped **3 b/s**
  ("maximum downward speed is reduced... at about 3 blocks per
  second"), sneak hangs ("grab hold of the ladder and not fall
  off"), jump-key climbing, fall distance zeroed while engaged,
  gravity substep frozen (ladder semantics), sprint cancelled
  (w/Vines §Behavior: "Vines cancel a sprint if the player is
  sprinting").
- **Ferns** (gen.rs flora pass): jungle/taiga flora arms (VERIFIED
  w/Fern §Natural generation: "Ferns occur naturally only in jungle,
  taiga, snowy taiga and old growth taiga biomes and their variants,
  scattered with short grass" — the live source CORRECTED the first
  draft, which wrongly included swamp).
- **Craft + fuel**: jungle log → 4 jungle planks recipe (the
  universal log→planks rule); JUNGLE_LOG/JUNGLE_PLANKS join the
  300-tick wood fuels (VERIFIED w/Log §Fuel).
- **Clean-room art** (`auditfix_art.rs`, 7 tiles): golden-carrot
  sprite, jungle bark/rings/leaves/planks, vine strands, fern
  fronds — zero Mojang assets.

### Verified

- Suite: **437/437 green** (426 + 11 new: V6 registry + state
  roundtrips, jungle-wood/vine/fern worldgen (incl. the bush
  signature + jungle-dominates-oak), taiga ferns, vine climb speed /
  descent cap / sneak-hang, golden-carrot breed + heal + food values,
  jungle-planks craft). wasm32 clean on the CI path. Zero
  todo!/unimplemented!/unsafe.

### Placeholder-unresolved

- **Vine fall-damage absorption**: the current wiki's Behavior
  section says "Vines absorb all fall damage, even without a solid
  surface nearby" — but the page mixes JE/BE without an edition tag
  at that sentence, and the Java ladder-climb reset (which we
  implement) is the 1.2-era VERIFIED behavior. NOT implemented as a
  blanket contact rule; climbing zeroes fall distance (the ladder
  semantics). Revisit if an edition-tagged source lands.
- Fern drop (12.5% wheat seeds, w/Fern) and vine/leaf shears
  collection — no seeds item / no shears in engine; both drop
  nothing (documented in the block comments).

### Deferred (formal, with reasons — closing the audit's
"silently-absent" findings that need engine systems first)

- **Jungle sapling** (1.2 item): no sapling system exists in the
  engine (oak saplings absent since Phase 0; the 1.7.2 bracket
  already deferred "saplings" as a class). Acquired-jungle-wood
  works via worldgen + the picker.
- **Carrot on a Stick** (1.4 item): needs pig riding + a fishing
  rod with durability — neither system exists (fishing is rod-less
  by design this engine; pigs are not rideable). The deferral reason
  is now recorded here (it was previously implicit).
- **Language support** (1.1 item): single-language engine (English);
  a translation layer has no scope. Recorded as N/A rather than
  silently absent.
- **Glass silk-touch pickup** (1.2 change): Silk Touch exists in the
  38-enchant registry but block-drop routing never consults
  enchantments (glass drops nothing, vanilla-without-silk-faithful).
  Needs the enchant→drops bridge from a tools/drops pass.
- **Jungle log axis X/Z placement states**: vertical placement
  unaffected (vanilla placement rule follows the clicked face — the
  default Y state covers top/bottom faces); sideways placement of
  jungle logs falls back to axis-Y (disclosed simplification; oak/
  birch/spruce/acacia/dark-oak have their X/Z states).
- **Golden-carrot rabbit breeding** (w/Golden_Carrot: "to breed,
  lead, and grow rabbits"): rabbits exist (1.8) but have no breeding
  path (equine-only breeding system); rides a future animal-breeding
  pass.
- Vine spread (random-tick growth, the ≤4-neighbors rule) — the
  current-wiki Behavior section's spread rules were not separately
  verified for the 1.2-era; deferred with the note.

### Known issues & regressions

- None: 437/437, wasm clean, no new clippy lints beyond the
  pre-existing set. The engine's snowy-taiga flora gate (SNOW_GRASS
  surface excludes the plant pass — pre-existing) means snowy-taiga
  ferns from the flora arm can't place; disclosed in the taiga fern
  test comment (vanilla snowy taiga has ferns; the engine's surface
  convention blocks it).

**Commit:** this entry (audit-fix round).

## 2026-09-07 — MC 1.11 bracket "Exploration Update" (Phase 1.11) — recovered from an interrupted session + completed — commit this-entry

**Task:** the 1.11 (Exploration Update) version bracket. **Honesty note
on the round's shape:** an earlier session had begun the bracket and
died mid-work — an unpushed local commit carrying ~1,500 lines of
implementation (four mobs, shulker box/shell/totem, woodland mansions,
1.11 research captures) with a UUID for a commit message, no WORKLOG
entry, five tests of which one failed, and three wiring gaps that made
real features inert (the new spawn eggs failed the `is_spawn_egg`
use-gate so using them did nothing; mansion chests never reached a loot
table; the mansion worldgen test scanned FOLDED block ids and always
read zero spawners/chests). This session recovered the work, audited
every changelog row against the live wiki captures, closed the gaps,
fixed a latent engine bug the new tests exposed, and completed the
bracket to the standard protocol (tests + WORKLOG + docs + one commit).

**Sources:** the live changelog capture
(`voxelcraft/scripts/v111_changelog_text.txt` +
`v111_page_changelog.json`), per-feature page captures
(`v111_page_{llama,vindicator,evoker,vex,shulker_box,shulker_shell,
totem,woodland_mansion}.json`), the earlier round's search verdicts
(`verify_v111_*.json`), plus two fresh live captures this round
(`v111_search_carpetfuel.json`,
`v111_search_mansionloot.json`).

### Implemented (this round)

- **Llama** (w/Llama, live): Mountains herds 4–6 (the changelog
  "Spawn in extreme hills"), health 15–30 per instance, speed 0.175,
  **strength 1–5** in the variant byte with the wild distribution
  32.8/32.8/32.8/0.8/0.8% (w/Llama §Strength), temper-taming via the
  equine infrastructure (repetitive riding, w/Llama §Taming), **hay-bale
  breeding** on two tamed adults (changelog: "Tamed llamas can be bred
  with hay bales"), leather 0–2 drops (66.67% roll rides the count max
  — engine convention), the 1⁄900 per-tick 1-HP regen, neutral class,
  and **spit retaliation** — provoked llamas fire a `LlamaSpit`
  projectile: 1 HP Easy/Normal, 1.5 Hard via difficulty scale. **Llama
  caravans** (changelog: "If the player puts a leash on one, up to 10
  llamas are attracted and try to form a caravan"): a leashed llama
  attracts up to 10 llamas within 9 blocks; the follow-the-leader
  chain (vanilla caravans leash-to-leash) is the disclosed
  simplification.
- **Vindicator** (w/Vindicator): 24 HP, iron-axe 13 HP Normal
  (7.5/19.5 E/H via scale), sprint speed 5.612 b/s, emerald 0–1 @ 50%,
  hostile; mansion spawner placement. Johnny tag and the
  attacks-villagers row are N/A (no name tags / no villager entities —
  disclosed below).
- **Evoker** (w/Evoker): 24 HP spell-caster, the two spells — **fangs**
  6 HP armor-ignoring ("not mitigated by armor" — rides the raw-damage
  path) and the **vex summon ring** (queued through
  `pending_summons`, drained by the game layer), 100% totem drop +
  emerald 0–1, XP 10, mansion upper-two-floors spawners. The
  blue→red sheep conversion is N/A (engine sheep are colorless —
  disclosed).
- **Vex** (w/Vex): 14 HP, iron sword 9 HP Normal, no-clip physics
  ("pass through any block, including water and lava" — the
  collision-skip path, tested), summoned-only (never in the spawn
  tables), 5 XP, no item drops (HandDropChances 0).
- **Totem of Undying** (w/Totem_of_Undying, live): held-item revival on
  lethal damage — restores 1 HP, clears all effects, Regeneration II
  45 s + Absorption II 5 s; **the absorption buffer** (8 points for
  Absorption II) now eats damage BEFORE health in `Player::damage`.
  Fire Resistance I (0:40) is a 1.16.2 addition (§History 20w28a) —
  version-scoped OUT. "Either hand" = the selected hotbar item (no
  offhand — disclosed). The revival payload is extracted into
  `apply_totem_revival` (unit-tested).
- **Shulker box + shell** (w/Shulker_Box, w/Shulker_Shell): 27 slots
  ("the same as a barrel, a single chest, or an ender chest"), solid
  placeable container, the **no-nesting rule** ("cannot be placed
  inside another" shulker box — the insert gate), the column recipe
  (shell/chest/shell — changelog §Blocks; the square matcher places it
  in the middle column, side columns are a disclosed placement
  constraint), break spills contents (vanilla keeps them inside the
  item — needs item-NBT, disclosed). Shell is picker-only (no shulkers
  / End cities in the engine — the 50% drop is N/A, disclosed).
- **Spawn eggs** (changelog §Items): the four new eggs
  llama/vindicator/evoker/vex (kinds 23..=26) **and the re-added
  husk/stray eggs** (kinds 27/28 — "Eggs that were removed in Java
  Edition 1.10-pre2 are re-added ... including: ... Husk spawn egg,
  Stray spawn egg"). All seven render **egg-shaped tiles**
  (`TILE_V7_EGG_BASE..=+5`, the E1/E2/E3 `egg_art` convention) — the
  interrupted round's mob-sprite reuse for egg items was replaced. The
  **zombie-villager egg** — the changelog's 5th new egg — is the
  engine's PRE-EXISTING E2-era item at id 129 (kind 5): an engine
  anachronism (added with the 1.4-era cure round) that satisfies the
  1.11 requirement without a duplicate; disclosed. The wither-skeleton
  / donkey / mule re-adds were already covered (kinds 16/21/22).
  **Wiring fix:** `is_spawn_egg` now includes the V7 window — the
  interrupted round's eggs failed the use-path gate and did nothing.
- **Woodland mansions** (w/Woodland_Mansion, live): rare dark-forest
  placement (8×8-chunk regions, ~1/5 hash gate, dark-forest + height
  checks), 13×13 footprint, three floors ("The top floor is about
  half the size" — 13/13/7), cobblestone shell + plank floors +
  full-coverage **cobblestone foundation** ("generate a cobblestone
  foundation underneath the entire structure"), cross-corridor rooms,
  south entrance, **vindicator spawners on the lower two floors +
  evoker spawners on the two upper floors** (the engine-native
  no-respawn adaptation of vanilla's generation-time spawns), two loot
  chests. Chunk-arrival registration rides the existing
  `register_block_entities` seam (spawner kinds 5/6 decode via
  `spawner_mob`; fortress blaze/wither-skeleton decode fixes were the
  interrupted round's, kept).
- **Mansion loot table** (`minecraft:chests/woodland_mansion`): the
  live page's four-pool structure with the page's own §History
  version-scoping — Vex Armor Trim (1.20, 23w04a) and Resin Clump
  (1.21.4, 24w44a) scoped OUT; the name tag (removed 26.1 snap11) and
  the palette-absent rows (diamond hoe, chainmail, music discs,
  diamond chestplate, enchanted golden apple, wheat, bread, redstone
  dust, seeds, iron/gold ingots, bucket) simply don't roll (the
  established honest policy). Present: pool 1 lead 20 / golden apple
  15 / enchanted book 10; pool 2 coal 15 (1–4); pool 3 bone /
  gunpowder / rotten flesh / string 10 each (1–8); pool 4 the empty
  partner. `chest_table_for` routes mansion chunks to it (priority
  dungeon > mineshaft > pyramid > jungle temple > **mansion** >
  stronghold).
- **Fuel** (changelog §Fuel + live verdict): wool 100 t (0.5 items)
  and **carpet 67 t (0.335 items** — live search verdict
  minecraft.wiki/w/Carpet, `v111_search_carpetfuel.json`; the
  changelog's "0.3 items" is the rounded form — the interrupted
  round's deferral was resolved by the live check). The other 1.11
  fuel rows are palette-absent (deferred below).
- **Curse of Vanishing** (changelog §Gameplay): cursed items are
  filtered from death drops ("makes the item disappear if the player
  dies"); both curse rows verified in the enchant registry.
- **Registry growth**: V7 window 282..=290 (9 blocks — shulker box,
  shell, totem, the four new eggs, husk/stray eggs), mansion spawner
  states **495..=496** (renumbered up from the interrupted round's
  493..=494 to clear the extended V7 window — nothing was ever pushed
  under the old numbering), BLOCK_COUNT 291, STATE_COUNT 497,
  TILE_MAX 345, WGSL mesh LUT resynced (L_FL 497 / L_TC 788 / L_ST
  1079, clamps 496/290).
- **LATENT ENGINE BUG FIXED (pre-1.11, found by this round's
  llama-spit test):** `attack_cd` is i32 and its per-tick decrement
  used `saturating_sub` — which for SIGNED types floors at i32::MIN,
  not 0. A fresh mob's cooldown walked 0 → −1 → −2 … forever, so every
  `attack_cd == 0` attack gate (skeleton arrows, melee swings, llama
  spit, even the game layer's zombie-vs-villager swings) could NEVER
  fire for a mob that had not already attacked — in the live game only
  creeper fuses and e2e debug hooks ever dealt damage. The existing
  mob tests masked it by calling `ai_tick` directly (no decrement).
  Now floored at 0 (`(cd - 1).max(0)`): fresh mobs can strike
  immediately, and a 20-tick cooldown re-fires on the 20th tick.

### Verified

- Suite: **454/454 green** (437 + 17: llama herd/strength/spit/
  breeding/vex-physics/egg-map tests, the mansion worldgen test
  (fixed to scan RAW states through `get_state` — the folded `get_idx`
  scan was the interrupted round's failure) + a spawner-state decode
  test, the mansion loot-table test, V7 registry/egg/roundtrip tests,
  totem/absorption/curse/shulker-recipe tests, carpet fuel asserts).
- wasm32 release build clean. Clippy: no NEW lints from this round's
  code; the pre-existing warning set stands, including the
  pre-existing `never_loop` error at vc-pack `datapack.rs:362`
  (present since the Phase 9 pattern matcher — CI runs tests + builds,
  not clippy; recorded here rather than silently ignored).
- Carpet fuel + mansion table rows verified live this round (the two
  new search captures).

### Deferred (formal, with reasons)

- **Observer behavior**: the block pre-exists (id 94, tiles 111/116 —
  an earlier bracket); its block-update detection needs a redstone
  signal system. The changelog's "Ported from the Pocket and Windows
  10 Editions. May have slight behavior differences" row is satisfied
  by the existing block; the observing function deferred.
- **Explorer maps** (cartographer trade → ocean monument / woodland
  mansion maps): no map items or map rendering in the engine.
- **Vindicator specifics**: Johnny tag (no name-tag/anvil-rename
  system), attacks-villagers (no villager entities — the villager
  system is gossip+trades around zombie villagers), axe-disables-
  shield (rides the disclosed 1.9 shield deferral — the engine's
  shield has no durability/disable mechanics to begin with).
- **Evoker blue→red sheep**: engine sheep have no color variants —
  the rule has no input (vacuously N/A).
- **Llama specifics**: chest storage (3×strength slots — no
  mob-equip UI), carpet decoration (same), multiple skins (single
  clean-room sprite), wolf aggression (no wolves — disclosed in
  code), leash-chain caravans (single-leash engine — the 10-follower
  simplification is implemented instead).
- **Spawn-egg re-adds for absent mobs**: skeleton horse / zombie
  horse / elder guardian eggs (no such mobs). Rabbit + polar bear
  eggs remain the standing 1.8/1.10 deferrals (out of 1.11 scope).
- **Fuel rows for absent items**: ladder 1.5 / wooden button 0.5 /
  bow 1.5 / fishing rod 1.5 / sign 1 / bowl 0.5 / wooden door 1 /
  boat 2 (none of these items exist in the palette).
- **Exhaustion-value changes** (the 1.11 rebalanced table —
  breaking a block 0.025→0.005 etc.): the engine is hunger-bar-less
  (no exhaustion treadmill; a disclosed design since Phase 2) — N/A.
- **Other N/A rows**: /locate (no commands), chat length (no chat),
  doWeatherCycle + maxEntityCramming gamerules (no gamerule system),
  entity-ID renames (inherently satisfied — the engine used the
  flattened `minecraft:*` string ids from day one), End-gateway
  regeneration (no End gateways), the fishing overhaul (fishing is
  rod-less by design), 1.11.1/1.11.2 (bugfix micro-releases, out of
  the bracket unit).

### Known issues & regressions

- None new: 454/454, wasm clean. (The attack_cd fix makes mob melee/
  ranged attacks live in gameplay for the first time — a behavioral
  change, but the restoration of intended Phase-2+ behavior, pinned
  by the round's tests.)

**Commit:** this entry (1.11 bracket, recovered + completed).

## 2026-09-07 — MC 1.12 bracket "World of Color Update" (Phase 1.12) — recovered from an interrupted session + completed — commit this-entry

**Task:** the 1.12 (World of Color Update) version bracket. **Honesty
note on the round's shape:** the implementation session died mid-work —
an unpushed local commit (`2c87f3b`, message a bare UUID) carried the
whole bracket body (V8 registry window, two mobs, the concrete/powder/
glazed-terracotta families, blindness, 17 inline tests, all research
captures) but no WORKLOG entry, no README row, and no research record
in docs/research/. This session audited the carried code line-by-line
against the live captures, verified the design decisions, completed the
round's documentation to the standard protocol, and prepared the push
(the standing CI gate — 471/471 tests, wasm clean — runs on push).

**Sources:** the live changelog capture
(`voxelcraft/scripts/v112_page_changelog_text.txt` +
`v112_page_changelog.json`), per-feature page captures
(`v112_page_{concrete,concrete_powder,glazed_terracotta,effect,
illusioner,parrot}.json` + `_text.txt`), all fetched live
2026-09-07 (pre-implementation) — recorded in
`docs/research/phase-v112-1.12-research.md`.

**Implemented (all constants live-verified against the captures):**

- **Blocks — V8 registry window (ids 291..=360):** the bracket's
  decorative core, 48 full blocks + the egg item:
  - **concrete ×16** — hardness 1.8, flat vibrant colors (the
    changelog's headline palette item; w/Concrete);
  - **concrete powder ×16** — hardness 0.5, gravity-affected like
    sand/gravel (w/Concrete_Powder), and the signature mechanic:
    **solidifies to concrete when touching water, checked BEFORE
    falling** (w/Concrete_Powder §Usage: "when it touches water, it
    turns into a concrete block"; covers both falling-into and
    placed-next-to water);
  - **glazed terracotta ×16** — hardness 1.4, obtained by smelting
    stained terracotta (0.1 XP per — the enchanting/furnace XP row),
    4-directional facing with per-rotation top/bottom art;
  - **parrot spawn egg** (kind 30).
- **Mobs (MOB_DATA 30→32):**
  - **parrot** — 6 HP passive, speed 0.2 (infobox), 5 variants
    (red/blue/lime/cyan/gray — the Variant NBT table), jungle spawn
    weight 40/93 = 43.01% over leaves+grass, groups of 1–2, drops
    1–3 XP, **taming**: 1/10 per seed feed (w/Parrot §Taming),
    seeds heal, **a cookie is instant death** (the engine's
    poison-free form of vanilla's fatal cookie), right-click sit
    toggle, follows the tamer with a 12-block teleport
    (cat-parity), gentle vex-style steering while flying;
  - **illusioner** — 32 HP hostile, speed 0.5, **no natural spawns
    and no spawn egg** (vanilla parity: raid-only in Java 1.12+;
    palette-only here), the spell kit: **Blindness** on the player
    (20 s — w/Illusioner §Casting_Blindness) queued through a
    game-layer pending-spell vector, and the defensive
    **Invisibility + 4 false duplicates** refresh cycle.
- **Status effect — Blindness** (id 15, negative): close black fog
  at the render layer + sprinting blocked (w/Effect §Blindness).
  Critical hits while blinded are a 1.9-combat detail the engine
  does not model — disclosed.
- **Crafting — the engine's first truly SHAPELESS 9-slot recipe:**
  4 sand + 4 gravel + any dye → 8 concrete powder of the dye's
  color (w/Concrete_Powder §Crafting + the changelog's own
  "shapeless" callout; the matcher ignores grid position, rejects
  wrong counts and multiple dyes).
- **Art (v112_art.rs, 379 lines, clean-room):** 16 flat vibrant
  concrete, 16 grainy powder aggregates, glazed terracotta as 16
  4-rotation top/bottom pairs + 16 shared side tiles, parrot 5
  variant tiles, illusioner tile — plus the **atlas row-math fix**
  (32-tile rows; the old `%16/16` indexing wrapped at 16 and would
  have smeared the 1.12 tiles across neighbors).

**Tests:** +17 inline — `v112_v8_registry_window`,
`v112_block_flags_and_sounds`, `v112_names_and_tiles`,
`v112_concrete_powder_shapeless_recipe`,
`v112_powder_recipe_rejects_wrong_counts`,
`v112_parrot_stats_and_variants`,
`v112_parrot_taming_roll_and_sit_toggle`,
`v112_parrot_cookie_is_instant_death`,
`v112_parrot_follows_and_teleports_at_12_blocks`,
`v112_illusioner_stats_and_blindness_spell`,
`v112_illusioner_never_spawns_naturally`, `v112_powder_falls_like_sand`,
`v112_powder_touching_water_solidifies`,
`v112_powder_without_water_stays_powder`,
`v112_all_powder_colors_solidify`, + the art/atlas tests. Suite
expected 471/471 (454 + 17) pending the push's CI gate.

**Deferred with reasons (recorded here, per the standing protocol):**
advancements (1.12's flagship system — no advancement engine), the
function/command system (no command parser), colored beds (no bed
block), the recipe book + knowledge book (no recipe-UI system),
crafting-tweaks gamerules (no gamerule system), dye ACQUISITION as an
economy (dyes are palette items feeding the powder recipe — the
standing disclosure since the 1.12 code comment), the iron nugget
(no nugget item), the "sound of milk" and mob-particle rows (N/A
to this engine's scope).

### Known issues & regressions

- None new expected. The atlas row-math change (`%16/16` → 32-tile
  rows) touches shared indexing — pinned by the art/atlas tests and
  the drift test on the existing tile layout.

**Commit:** this entry (1.12 bracket, recovered + completed; the
interrupted session's body amended to carry this documentation and a
real commit message).

## 2026-09-07 — MC 1.13 bracket "Update Aquatic" (Phase 1.13) — recovered from an interrupted session + completed — commit this-entry

**Task:** the 1.13 (Update Aquatic) version bracket. **Honesty note on
the round's shape:** the implementation session died mid-work AGAIN —
two unpushed local commits (`2717898` and `7f18519`, messages bare
UUIDs) carried the bracket body (the V9 registry window with
coral/pickle/kelp/conduit blocks + items, the eight aquatic mobs'
registry rows and spawn tables, four new effects, the ocean biome
split, ~32k lines of research captures, the game-layer queues and ten
TDD-style test functions) — but the work did not COMPILE: the tests
referenced a `MobKind::Squid` classification marker and a
`registry_id()` accessor that were never written, and seven of the ten
new tests failed on behavioral gaps. This session audited the carried
code line-by-line against the live captures, finished the
implementation, fixed the failures (three real engine bugs among
them), wired the remaining bracket systems the carried code had
prepared but not connected (world-gen ocean flora, the conduit, the
brewing/crafting/furnace/food rows, drink-time effect windows), and
completed the documentation to the standard protocol.

**Sources:** the live changelog capture
(`voxelcraft/scripts/v113_page_changelog_text.txt` +
`v113_page_changelog.json`), per-feature page captures
(`v113_page_{blue_ice,conduit,coral_block,dead_coral_block,dolphin,
dried_kelp_block,drowned,heart_of_the_sea,kelp,nautilus_shell,
phantom,phantom_membrane,pufferfish,salmon,scute,sea_pickle,
slow_falling,trident,tropical_fish,turtle,turtle_shell}.json` +
`_text.txt`) — **cross-checked against the independent Fandom
captures** (`v113_page_x_fandom_{113,drowned,phantom,trident}.*`, the
user's standing multi-source verification directive), all fetched live
2026-09-07 (pre-implementation).

**Implemented (all constants live-verified against the captures):**

- **World — the ocean temperature split** (changelog §World
  generation): Warm/Lukewarm/Cold/Frozen ocean families (internal ids
  19..=22) selected off the EXISTING climate temp field (VERIFIED:
  "Added minecraft:warm_ocean ... minecraft:frozen_ocean now generates
  again"); warm-side sand floors (the coral-reef substrate) vs
  cold-side gravel; `Biome::is_ocean()` as the family gate.
- **Ocean flora (this session):** kelp 2–4-block columns at 8%/floor
  cell in every ocean family EXCEPT warm (VERIFIED: "Generate in ocean
  biomes, except warm oceans ... Can grow multiple blocks high");
  seagrass at 12%/cell + the swamp-pool 20% roll (VERIFIED: "Generates
  in oceans ..., rivers, and swamplands"); coral reefs as patch-noise
  fields in warm oceans (coral blocks ×5 as floor surface, coral
  plants and fans ×5 above, at 35%/25%/18% of in-patch columns —
  VERIFIED: "Naturally generate in coral reefs ... composed of coral,
  coral blocks and coral fans"); sea-pickle clusters 1–4 counts on
  12% of in-patch columns (VERIFIED: "generate in warm oceans,
  especially around coral reefs ... Up to 4 of them can be placed on a
  block"); the frozen-ocean ICE surface sheet; icebergs at 25%/chunk —
  simplified pack-ice mounds with blue-ice cores (the full vanilla
  iceberg shape grammar is out of scope, DISCLOSED).
- **Eight new mobs (registry + spawns carried; behaviors completed):**
  - **drowned** (w/Drowned): ocean/river water-column hostile
    spawner in 1–2 packs; **zombies convert after 600 ticks of
    continuous head submersion** (VERIFIED §Conversion: "If a zombie's
    head ... is continuously submerged for 30 seconds"); 6.25% spawn
    armed with a trident (§Equipment) thrown every 30 ticks at up to
    20 blocks for 8 HP (VERIFIED §Attacking: "can throw it every 1.5
    seconds, sending it up to 20 blocks away"), the trident dropping
    at 8.5% on a player kill (w/Trident);
  - **phantom** (w/Phantom): the insomnia spawner — packs above
    players whose "Time Since Last Rest" ≥ 72000 ticks (1 in-game
    hour), 12–20 blocks up, local pack cap 4, statistic reset by
    player death (note_rest wired into the respawn path); the
    **orbit-and-swoop cycle**: 200-tick orbit at 12 blocks above the
    player (a dedicated damped altitude-hold controller — see the bug
    list) then a 60-tick dive with the 2-HP swoop bite (the current
    wiki's 1.14-pre3 value; the 1.13 original was 6 — version-scoped,
    disclosed);
  - **dolphin** (w/Dolphin): neutral pods of 1–2 (JE) spawning only
    in non-frozen/cold oceans at Y 50–64; **Dolphin's Grace 5 s
    banked for sprint-swimmers within 9 blocks**, replenished at most
    1/s (VERIFIED);
  - **cod / salmon / tropical fish** (w/ pages): the 3-HP passive
    water-ambient pool with the verified biome families (warm =
    tropical + pufferfish, lukewarm = cod + tropical, cold/frozen =
    cod + salmon split; group sizes 3–6 / 3–5 / 1–2 per the pages),
    out-of-water flopping + 1 HP/s suffocation (VERIFIED w/Cod:
    "cannot survive out of water"), school 3D-wander;
  - **pufferfish** (w/Pufferfish): neutral, inflates 0→1→2 one step
    per 20 ticks as a player closes within 3 blocks; contact 2 HP +
    3 s poison semi / 3 HP + 6 s fully (VERIFIED Java rows);
  - **turtle** (w/Turtle + w/Scute): beach sand nester (groups ≤ 5,
    5% babies with the 20-min maturity countdown); bred females
    (variant bit 7) queue real TURTLE_EGG world edits on sand; babies
    mature and queue SCUTE drops (VERIFIED w/Scute: "Dropped when baby
    turtles grow up"). **Environmental execution:** egg-laying and
    maturity run BEFORE the player-anchor early-return in ai_tick —
    they are player-independent (the carried code had them after it,
    dead with no player in range — this session's restructure).
  - **MobKind::Squid** — a classification-only marker (no MOB_DATA
    row, never spawns): the aquatic() gate is the 1.13
    Update-Aquatic physics set, and the squid is pre-1.13 legacy
    (Beta 1.2) that must NOT receive it (the carried tests' own
    verdict: "the squid is pre-1.13 legacy").
- **Aquatic physics:** buoyancy + drag for the aquatic set in water
  (fish hover, turtles/dolphins glide), fish suffocation on land, the
  water-ambient spawn category SEPARATE from the passive cap
  (passives_alive() now excludes aquatic kinds — vanilla's
  water_ambient/water_creature split, VERIFIED §Spawning).
- **The conduit (this session):** placed conduits register in
  `sim.conduits`; the game layer scans each for the 26-water 3×3×3
  core (the waterlogged gate — VERIFIED w/Conduit: "A conduit won't
  be activated if not waterlogged") and counts prismarine-family
  frame blocks in the 5×5×5 shell (16 minimum to activate — VERIFIED:
  "A minimum of 16 blocks are required"); players in water inside the
  **32→96-block range ladder** (48@21/64@28/80@35/96@42 — the wiki's
  own data points; the "every seven blocks" phrasing is its rounding)
  receive **Conduit Power** (air frozen via the existing
  water_breathing gate); a complete 42-block frame **hunts ONE wet
  hostile within 8 blocks at 4 HP every 40 ticks** (VERIFIED:
  "dealing 4 HP magic damage every 2 seconds", "attack only one mob
  at a time"). The ring-shaped vanilla frame is approximated by the
  shell count (clamped at 42) — DISCLOSED.
- **Four status effects** (carried): Water Breathing (id 13 — the air
  meter freezes), Slow Falling (id 28 — the −9.8 b/s terminal clamp +
  fall-damage negation, VERIFIED w/Slow_Falling: "terminal velocity of
  9.8 m/s, and is unable to take fall damage"), Conduit Power (id 29),
  Dolphin's Grace (id 30 — ×2 swim multiplier; the wiki publishes no
  scalar, DISCLOSED approximation).
- **Brewing (this session):** awkward + phantom membrane → Slow
  Falling (VERIFIED: "Brewed with phantom membrane"); awkward + turtle
  shell → Turtle Master (VERIFIED: "from an awkward potion");
  glowstone → Turtle Master II (VERIFIED: "Slowness VI and Resistance
  IV"); `potion_effects()` maps the drink-time windows (slow falling
  1:30 / extended 4:00 item row; turtle master Slowness IV +
  Resistance III 1:00, II = VI + IV) — the right-click drink path
  applies them for real.
- **Crafting/furnace/food (this session):** turtle shell (5 scutes,
  the helmet shape), dried kelp block (9 kelp) + reverse (→ 9 kelp),
  the conduit ring (8 nautilus shells + heart of the sea), blue ice
  (9 packed ice); kelp → dried kelp smelting, sea pickle → lime dye;
  dried kelp block fuel = 4000 ticks (VERIFIED: "Smelts 20 items");
  dried kelp food = 1 hunger (the engine's 0.5 HP row).

**Bugs found and fixed by this session (the carried round's tests
exposed them):**

1. **Beach-turtle spawn scan used an empty range** —
   `(p[1]+12 .. p[1]-12)` is `82..58`, an EMPTY range (start > end),
   so the beach branch of try_spawn_aquatic NEVER fired. Bounds
   swapped to scan down from +12 to −12.
2. **Flying mobs took gravity** — vanilla FlyingMobs (phantom/vex/
   bat/parrot) have none; the constant −1.568 b/s pull dragged the
   phantom's orbit ~3 blocks below its 12-block spec height and
   forced the vex/bat/parrot to fight gravity with velocity lerps.
   New `MobKind::flies()`: no gravity, gentle 0.98 flight drag, no
   fall-distance accumulation (a swooping phantom is flight, not a
   fall).
3. **The phantom's orbit altitude rode the shared 3D steer** — the
   tangential chase ate the vertical authority; the orbit now uses a
   dedicated damped P-controller (asymptotic approach — never sags
   below the target height).

**Test-setup corrections (the carried tests' own arithmetic/world
slips, fixed in place with the intent preserved):** the phantom
insomnia test's below-threshold window started 40 ticks short of the
threshold (an 80-tick loop crossed it); the pufferfish test beached
its fish on a stone flat-world (a fish out of water suffocates 1 HP/s
and flops — moved to its native warm ocean with the wander pinned for
determinism); the zombie conversion test's zombie could random-walk
off the single loaded test chunk (head reads AIR outside it — wander
pinned); the phantom aux assert sampled mid-countdown packs.

**Tests:** +18 this round — the ten carried v113 mob tests
(registry/flags/trident throw/conversion/orbit-swoop/pufferfish
inflate-sting/dolphin grace/turtle eggs-scutes/insomnia
spawning/drowned water spawns/water-ambient families) now green, plus
this session's eight: `conduit_range_matches_the_java_ladder`,
`v113_aquatic_recipes`, `v113_aquatic_smelting_and_fuel`,
`v113_aquatic_brews`, `v113_potion_effect_windows`,
`v113_ocean_flora_matches_the_biome_families`,
`v113_kelp_columns_grow_multiple_blocks`,
`v113_ocean_families_present_and_roundtrip`. Suite: **489/489** green
(471 + 18), wasm32 lib check clean.

**Deferred with reasons (recorded per the standing protocol):**
advancements (the bracket's flagship system — no advancement engine),
commands/Brigadier (no command parser), trident enchantments
(Channeling/Impaling/Loyalty/Riptide — no enchantment-on-weapon
mechanics), player trident throwing + the 9-HP melee row (no
player-weapon or player-projectile system — the standing fists-only
`held_attack` deferral), map markers (no map items), the
redstone-extended potion brews (no redstone-dust item; the extended
ITEM rows exist with correct windows), turtle-egg hatch stages +
trampling (needs random block ticks), coral death-out-of-water (same
tick system), bubble columns (needs waterlogged block states),
stripped logs / debug stick / carved pumpkin / buffet world type
(palette- and system-absent), buckets of fish (no mob-in-bucket
items), Conduit Power's Night Vision + Haste halves (no darkness
system; no per-block mining-time system — the standing deferrals),
and the water-bucket-on-fish interactions (no bucket capture path).

---

## 2026-09-07 — startup flow rebuilt to the vanilla architecture (intro → panorama title → world entry)

**Task:** user asked why the game renders the whole world in the intro
instead of a pre-rendered vignette like the real title screen, asked for a
web-verified implementation of the real startup sequence (first screen after
opening + the loading screens), with no third-party names in-game. Also:
verify all previous jobs were landed, and update docs.

**Research (live, 2026-09-07):** minecraft.wiki/w/Panorama — the title/menu
background is a **slowly panning wide-angle view shown as a cubemap of six
pre-rendered square images** (four horizontal faces + up + down, ~1.08k
square), displayed behind every menu that does not cover the whole
background, with an adjustable blur; it is **not the live world**. The wiki's
loading-screen pages confirm the boot order: logo/progress screen while
resources load (no world), then the panorama title; chunks generate only
when a world is entered, behind its own progress screen.

**What the code did wrong:** `GameApp::new` created a world at boot and the
Loading gate held the title screen until 5+ chunks around spawn were meshed
and uploaded; the "panorama" camera was the live player world (`player.pos
+ 14`, rotating yaw, blur 0.9) and streaming ran in EVERY screen. So the
title screen was coupled to the full generate/mesh/GPU-upload pipeline — on
the user's machine that meant a minute+ before the home screen (and a dead
mesher could stall it outright — the earlier 14bc3b7 watchdog papered over
the symptom without removing the coupling).

**Fix (vanilla architecture, clean-room):**

1. **`Screen::Intro`** — the first screen after opening the game: logo +
   asset progress bar over a near-opaque dark wash (a faint blurred-panorama
   glow reads through). Assets (builtin pack, atlas, pipelines, audio) all
   load in `GameApp::new` before the first frame; the intro is the settle
   beat, then hands over to Title. No world generation anywhere in it.
2. **Pre-rendered panorama** (`vc-render/src/panorama.rs`) — six
   procedurally painted 384px cubemap faces (sky gradient, blocky sun + halo,
   fbm clouds on the direction sphere — seam-free, a two-layer hill
   silhouette from periodic azimuth noise, a voxel tree belt, a lake sector
   with sparkle), painted ONCE at renderer init into a cube-viewed texture,
   drawn per frame by a fullscreen ray-cast pass (yaw/pitch/fov/aspect
   uniform → world ray → `textureSample` cube). Slow pan (~3.5 min per
   revolution), existing menu blur on top via the post chain. Title,
   Options-from-title, WorldSelect and WorldCreate all render it; the world
   pass (terrain/water/clouds/particles/shadows/MSAA) is skipped entirely.
3. **Streaming gated to active worlds** — `stream()` runs only in
   Loading/Game/Pause/Death/in-game-Options. Menus do zero world work.
   The world-entry loading screen keeps the chunk gate + 15 s timeout but
   rides the panorama dimmed behind its terrain progress bar; §28 dimension
   travel keeps the live-world blurred view (that world exists and streams).
4. **CI smoke extended end-to-end** — `--smoke` now runs the REAL boot path
   (intro → title panorama), then enters a world through the real pipeline
   (`reset_world` → Loading gate → `start_game`) and exits 0 only in
   gameplay. linux-game.yml greps `intro complete`, `smoke: title reached`,
   `loading (complete|timeout)`, `smoke: game entered` — the workflow now
   covers the exact path the original loading hang lived in, not just the
   title.
5. **In-game text scrubbed of third-party marks** — title corner text and
   one splash (research citations in comments/docs unchanged).

**Verified:** 43 tests green; `cargo check` native (--no-default-features,
ALSA-less box) + wasm32 lib clean; smoke/world-entry path exercised by CI
on lavapipe (see the linux-game run for this commit).

**Art notes:** the panorama painter is deterministic (integer-hash value
noise + fbm), paints linear-space values into the linear scene texture
(post re-encodes sRGB once), and follows the standard cube face conventions
(layer order +X −X +Y −Y +Z −Z, u right / v down, top-left origin) so
hardware face selection reconstructs the view without seams.

---

## 2026-09-07 (b) — follow-up: two live bugs the CI smoke + browser run flushed out

**Task:** turn the startup-flow CI round green; verify the browser build
visually (screenshots for the README).

**Bug 1 — native clock frozen (THE root cause of the original
"stuck on loading >1 minute"):** `now_secs()` returned UNIX-epoch seconds
as **f32**. At ~1.79e9 the 24-bit mantissa resolves only ~128–216 s, so
every `dt` computed from it was 0 — frozen clock: no physics, no menu
timers, no fps, no toasts, and every "X seconds" timeout silently became
minutes. The hoisted 15 s loading escape hatch could NEVER fire on native
(only `ready` paths worked, which is why lavapipe CI passed while the
user's machine hung). The intro's 1.1 s handover exposed it immediately
(CI: frames submitted, loop alive, intro never completed, `timeout 124`).
Irony: the wasm branch of the same function already carried the fix with
a comment describing exactly this symptom — native never got it. Fixed
with process uptime via `Instant` (monotonic, exact in f32 for days).

**Bug 2 — stale-UI race on screen transitions (live-observed in the
browser):** after the world-entry gate handed over to gameplay, the
"VOXELCRAFT / BUILDING TERRAIN…" overlay + a partial progress bar stayed
STUCK over sharp live terrain — no crosshair, no hotbar, forever (loop
alive, fps=10). Cause: the UI rebuild gate throttles on cadence; right
after a 20 Hz progress-screen rebuild it SKIPS the repaint, and render()
then uploads the STALE canvas and clears `ui.dirty` → nothing ever
repaints. The 20 Hz loading heartbeat made the race fire every time
(previously it only fired sometimes). Fix: `set_screen()` backdates
`last_ui_t` so a transition forces the repaint in the same frame.

**Verification:** CI green end-to-end at 6ef8992 (lavapipe smoke:
`intro complete … title in 1.12s` → `smoke: title reached — entering
world` → `loading complete: 8 chunks on GPU in 0.3s` → `smoke: game
entered — exiting 0`, exit 0); local wasm rebuild (wasm-bindgen 0.2.127
+ glue patch) clicked through in headless Chromium with the input-event
queue — crosshair/hotbar/hearts verified, overlay gone. Screenshots
saved as `docs/screenshots/startup-{intro,title-panorama,world-loading,
gameplay}.png`; README gallery + maintenance note 4 updated. Panorama
art itself VLM-verified (six faces: continuous horizon, no seams).

## 2026-09-07 — exact 1.16.5 boot flow (structure-matched, clean-room) + 1.14 research round

Commit c4a1d22. User ask: make the intro/panorama/loading EXACT like the
real game (referencing replica-project practice), verify all prior jobs,
keep everything legal, update docs, continue the plan.

- **Live research** (minecraft.wiki, captured in-session): Panorama history
  ("1.16 ... Changed panorama ... to reflect the Nether Update" — the 1.16.5
  title background is NETHER-themed; 1.13 pre1 removed live gaussian blur;
  slow 360-degree pan); Title screen (logo + splash + panorama + version
  bottom-left + copyright bottom-right); Splash (yellow, 2 Hz pulse, tilted
  ~20 degrees at the logo's bottom-right); Loading world screen (Java:
  "Loading world" + percentage + a 35×35 chunk colormap over a
  blurred+darkened panorama, with the page's EXACT status color table —
  Empty 545454 / Biomes 80B252 / Full FFFFFF / Spawn F26060).
- **Intro** → the real splash structure: solid studio-red field, dark
  VOXELCRAFT STUDIOS wordmark, thin WHITE bar. No panorama behind it (was:
  dark wash over blurred pano).
- **Title** → vanilla layout: logo scale 12 (no dim band), new
  `text_splash()` (glyph bitmap → rotated blit, −20 deg, right end up,
  2 Hz sub-pixel pulse) — **my own new test caught the blit using R(θ)
  instead of R(−θ): the splash tilted the WRONG WAY; fixed**; button stack
  rebuilt to vanilla geometry (300×30 from half screen height:
  SINGLEPLAYER / MULTIPLAYER [disabled until netcode] / half-width
  OPTIONS + QUIT row); version "VoxelCraft 1.16.5" bottom-left.
- **Loading** → `world_loading_screen()`: LOADING WORLD + percent + the
  35×35 colormap (4px cells, wiki-exact colors, spawn cell red until
  meshed), driven from the real pipeline (world.chunk()=generated →
  renderer.has_chunk()=meshed).
- **Panorama** → rethemed Nether (crimson fog sky, lava-glow horizon,
  netherrack ground, denser+taller crimson-canopy tree belt, glowing lava
  lake, no clouds); tests updated; PANORAMA_DUMP now sRGB-converts (dumps
  previously showed raw linear — VLM under-read them; pixel-scan confirmed
  3893 canopy pixels + lava band before believing the "flat" verdict).
- **Menu blur** re-tuned: title/options/world screens 0.45 (the soft
  pre-blurred-image look), world-entry loading 0.75 ("blurred and
  darkened" per the wiki), travel keeps 0.35.
- **Legal pass**: README Disclaimer rewritten to the Mojang fan-guidelines
  wording ("NOT AN OFFICIAL MINECRAFT PRODUCT...") + ClassiCube precedent;
  grep audit — no third-party names in user-facing strings; `minecraft:`
  namespaced ids documented as format interop (never in the UI).
- **Verification**: +5 screen tests (55 vc-render), 495/495 workspace
  green, wasm32 lib clean; VLM pass on the dumps (fixed: em-dash rendered
  as '?' in the 5×7 font → hyphen; disabled-button contrast). CI on
  c4a1d22: CI + Build WASM + Linux Game (lavapipe smoke: intro → title →
  world entry → gameplay, exit 0) ALL GREEN.
- **1.14 Village & Pillage — research round landed** (nature half):
  live captures for Bamboo / Campfire / Sweet Berry Bush / Fox / Barrel
  saved (`voxelcraft/scripts/v114_page_*.json`) and distilled into
  `docs/research/phase-v114-1.14-research.md` — the implementation
  contract (growth 1/3 random-tick, bush damage 1 HP/half-second
  moving-only + fox immunity, harvest 2–3/1–2, campfire 600-tick cooking
  + 2-charcoal drop, barrel 27 slots, fox 10 HP/2–3 dmg/0.7×0.6 box,
  taiga groups 2–4 + prey list). Implementation (registry V10 window →
  art → gen → gameplay → tests) is the next round, per the repo's
  research-then-implement discipline.

## 2026-09-07 (session 16) — F3 vanilla overlay (live values), native pointer-capture ladder, staged intro, first-run profile folder

**User reports addressed:**
1. "F3 values were static / don't work" — every F3 value now live.
2. "Mouse clicking and stuff not working in the latest executable" —
   root cause: blind `set_cursor_grab(Locked)` with the error DISCARDED
   while the cursor was hidden anyway. On lock-less compositors
   (WSLg/Wayland, RDP, some X11) that left an invisible cursor with no
   relative-motion events — camera frozen, clicks seemingly dead.
3. "Increase the intro loading a bit so assets properly load, like the
   real game" — splash now runs a staged ~2.6 s asset bar.
4. "Why isn't our asset getting created when we run the game" — first
   run now materializes the vanilla-profile-style folder next to the
   executable.

**F3 debug overlay (vanilla 1.16.5 structure):**
- ui.rs: `debug()` → two columns, per-line 0x90505050 strips, FLAT
  (unshadowed) text; `debug_help()` (F3+Q box); `text_flat()`;
  `frame_graph()` extension (F3+1); F3+H advanced tooltips on the
  picker + container hover labels (registry ids appended).
- game.rs `f3_lines()`: left = version / `fps T: D:` / `Integrated
  server @ N ms ticks` (live phase_ms(PHASE_SIM)) / `C: drawn/loaded
  (s) D: rd, pC: pU: aB:` (jobs in flight + GPU buffers) / `E:
  visible/loaded B:` / `F: I:` culling / client+server chunk caches /
  XYZ (3/5/3 decimals) / Block / six-value Chunk line / Facing
  (engine→vanilla yaw wrap + pitch negation) / Client+Server Light
  (real light engine) / CH S+CH H (live column scans) / Biome /
  Local Difficulty (0.75 + day ramp + moon phase × mode multiplier) /
  SC mob-cap categories (live sim scan) / Sounds 1-s window + registry
  / vanilla footer hints. Right = Rust 64bit release / Mem (% rss/sys
  from /proc, 4 Hz) / Allocated (% of RSS, counting allocator) / CPU
  (brand + cores) / Display WxH (adapter) / GPU name + driver line;
  Targeted Block/Fluid with full blockstate property lines when the
  crosshair hits.
- alloc_stats.rs: process-wide counting allocator (>=4 KiB counts
  only — small Vecs stay off the atomic path; chunk meshes/region
  arenas dominate). Installed in main.rs (native) / lib.rs (wasm).
- Liveness heartbeat: 0.05 s UI rebuild while the overlay is open.
- Verification: F3_DUMP/F3_DUMP2 pair (frames 0.6 s / 1.6 s into
  gameplay) — pixel-diff proves live values; VLM read of the dump
  confirms the vanilla two-column layout. Screenshot:
  docs/screenshots/f3-vanilla-live.png.

**Native pointer capture (the input regression fix):**
- `PointerLockMode { Locked, Confined, Delta }` + capture ladder
  `capture_pointer()` / `release_pointer()`; every grab site
  (set_screen, first in-game click, picker open/close, container
  open/close) goes through it. Cursor hides ONLY on a successful grab.
- Delta-look fallback: CursorMoved position deltas feed
  `input.add_mouse` while in the game screen (menus/containers
  unaffected); DeviceEvent raw motion is gated off in Delta mode so
  the two never double-count.
- Ladder re-attempted on the first in-game click (Wayland compositors
  that only lock on user gesture). A `pointer:` boot-log line names
  the mode (CI greps for it).

**Intro pacing + first-run profile:**
- `INTRO_SECS = 2.6` + `intro_progress()` staged waypoints (pack →
  atlas/pipelines → audio → save scan, with settling holds).
- `bootstrap_game_dir()` (native, GameApp::new): creates saves/,
  resourcepacks/, shader-packs/, logs/; EXTRACTS the embedded builtin
  pack to builtin-pack/ on first run (folder source wins over the
  baked copy — editable re-skin path); mirrors every boot line to
  logs/latest.log (init_file_log in vc-render); writes default
  options.txt.
- Settings persistence on native: load at boot / save on change
  (options.txt = the localStorage analog; corrupt → defaults + log).

**CI (linux-game.yml):**
- Smoke now greps the `pointer:` ladder line.
- Second smoke run in a fresh dir: F3=1 F3_DUMP=a F3_DUMP2=b —
  asserts "F3 liveness pair written", the pair DIFFERS (cmp), and the
  first-run folder structure materializes (builtin-pack/pack.mcmeta,
  options.txt, logs/latest.log, saves/, resourcepacks/,
  shader-packs/).

**Verification:** 500/500 tests green, wasm32 clean, local lavapiipe
smoke: boot 467 ms → intro 2.64 s → title → world entry (loading
complete 8 chunks 0.3 s) → pointer: confined (ladder fallback engaged
under Xvfb exactly as designed) → in-game click → F3 liveness pair →
exit 0. First-run directory verified in a scratch folder (pack
extracted + folder pack source loaded + options.txt + log mirror).

**Next:** 1.14 Village & Pillage implementation round (research
contract already landed: docs/research/phase-v114-1.14-research.md).

## 2026-09-07 (session 16, addendum) — 1.13 art-gap fix: the V9 window finally painted

**Found:** starting the 1.14 round's registry prep, an atlas-coverage
audit revealed the 1.13 recovery round (bracket 10/16) shipped the V9
tile window (550..=618) with TILE_MAX raised but NO painters in
textures.rs — every 1.13 block, item, egg and ALL EIGHT aquatic mob
billboards rendered blank (invisible drowned/phantom/dolphin/cod/
salmon/pufferfish/tropical-fish/turtle; invisible coral/kelp/seagrass/
pickles/conduit/eggs in oceans). Undisclosed in the 1.13 deferral list
— a true gap, now closed.

**Fix:** `vc-render/src/textures/v113_art.rs` (the v112_art.rs child-
module pattern):
- coral blocks ×5 + dead ×5 (polyp noise + darker flecks; the five
  families asserted pairwise-distinct, the dead forms asserted gray)
- coral plants ×5 + dead ×5, coral fans ×5 + dead ×5 (string-grid art)
- sea pickle 1..4 count tiles (rod count grows; pixel-count test)
- blue ice (deep blue + bright streaks), dried kelp block (strand rows)
- kelp (frond cross), seagrass (blade tuft cross)
- the conduit (shell frame + glowing heart core)
- turtle eggs ×3 hatch stages (crack pixels deepen per stage — test)
- 11 item icons (heart of the sea, nautilus shell, scute, trident,
  phantom membrane, dried kelp food, turtle shell)
- 4 potions on the existing potion_art bottle painter
- 8 spawn eggs on the e1_art::egg_art convention + V9_EGG_PALETTES
- 8 mob billboards: drowned (teal zombie), phantom (blue manta),
  dolphin (gray-blue, pale belly), cod, salmon, pufferfish (spiked
  ball), tropical fish (striped), turtle (shelled)

**Guards:** `v113_tiles_all_painted` (every tile 550..=TILE_MAX must
have >= 4 painted pixels — a blank tile is a missing painter),
`coral_palette_is_five_distinct_colors`, `sea_pickle_counts_grow`,
`turtle_egg_cracks_deepen`; an ATLAS_DUMP env hook on the coverage
test for visual inspection. VLM-read of the cropped atlas region
confirms the families/items/sprites read correctly.

**Verified:** 504/504 workspace tests green (+4), wasm32 clean,
release smoke green (boot → intro 2.64 s → title → world entry →
pointer: confined → in-game click → exit 0).

**Next:** the 1.14 Village & Pillage nature-half implementation
(bamboo/sweet-berry-bush/campfire/barrel/fox) on the researched
contract in docs/research/phase-v114-1.14-research.md.

## 2026-09-08 (session 17) — 1.14 Village & Pillage, the NATURE HALF (bracket 11/16)

**Task:** continue the main plan — the 1.14 implementation round on the
research contract landed last session
(docs/research/phase-v114-1.14-research.md, raw captures
`voxelcraft/scripts/v114_page_*.json`). The village half (villages/
pillager/raids/crossbow/bell/trader/crafting-stations) stays deferred
until the engine has village+raid scaffolding — the same deferral class
as prior brackets; this round ships everything that grows, burns,
stores, and prowls.

**Re-verification pass (the research doc's own flagged items):** the
captures already on disk answered every "re-verify at implementation
time" flag — bush slow **"34.05% of their normal speed"** (the doc had
"re-verify the exact multiplier"), bamboo fuel **"smelts 0.25 items"**
(50 ticks), campfire ingredients **"Stick + Coal or Charcoal + Any
Log"** (3/1/3 grid), barrel **"6 wood planks and 2 wood slabs"** (the
18w50a history row), berry-bush gen **"a 1/12 chance"** per chunk,
bamboo **"widely scattered single shoots within jungle biomes"** (the
20% jungle-edge figure replaced by the patch-roll adaptation),
harvest **"1–2 in its third growth stage, 2–3 in its final"** with the
**revert to the second growth stage** after harvest, and the bush
damage **"1 HP every tick (although damage immunity reduces this to
once every half-second), only if the entity is moving"**.

**Registry (V10 window — blocks 417..=425, states 676..=688, tiles
619..=633):**
- BAMBOO (417, stalk) + BAMBOO_SHOOT (418) — cross-rendered
  non-solid columns (vanilla's 2-px stalk collision can't be expressed
  in the engine's binary-solid model — the kelp-column adaptation,
  disclosed)
- SWEET_BERRY_BUSH (419) — 4 age states (0..3)
- CAMPFIRE (420) — unlit/lit states; **placed LIT**; `state_emissive`
  carries 15 on the lit state (the REDSTONE_LAMP_LIT pattern)
- BARREL (421) — solid + opaque (w/Barrel infobox "Transparent No")
- SWEET_BERRIES (422, item), SPAWN_EGG_FOX (423, kind 40)
- STICK (424) + CHARCOAL (425) — **two legacy items added late**:
  the stick is an Alpha-era item the engine never needed until the
  campfire recipe consumed one; charcoal until log-smelting + the
  campfire drop produced it. Both disclosed as late arrivals in the
  1.14 window.
- BLOCK_COUNT 417→426, STATE_COUNT 676→689, TILE_MAX 618→633, the
  WGSL mesh LUT resynced (L_FL/L_TC/L_ST + the min() clamps — the
  drift test caught it immediately, working as designed)
- F3: `state_description` now decodes the V9 pickle/egg AND V10
  bush/campfire property states for the Targeted Block lines
  ("Sweet Berry Bush[age=2]", "Campfire[lit=true]")

**Art (v114_art.rs, painted from day one):** bamboo stalk (segmented
culm + node rings + leaf blades), shoot, berry bush ×4 stages (the
berry pixels must GROW with the stage — test), lit/unlit campfire
(the lit one must show glowing coals, the unlit none — test), barrel
lid (plank cross + hoop band) + stave side, sweet berries icon, fox
sprite (orange/white/black), stick + charcoal icons; the fox egg on
the e1 egg-art convention. Guards: `v114_tiles_all_painted` (619..=633
must have ≥4 painted pixels — the 1.13 blank-window lesson baked in
WITH the window), `v114_berry_bush_berries_grow`,
`v114_campfire_lit_glows`.

**Gen:** bamboo patches (20%/chunk in Jungle, 4–10 shoots 1..4 tall on
grass/dirt/podzol — the no-bamboo-jungle-sub-biome adaptation,
disclosed); berry-bush patches in Taiga + Snowy at the VERIFIED 1/12
chunk roll, 3–6 bushes at bearing ages 1..=3. Both biome-scoped both
ways (present in-family, absent outside — tests).

**Gameplay:**
- random ticks: shoot→stalk (1/3 + light 9), stalk column growth
  (top-cell-only roll, 1/3, cap 16, light gate at the would-be top),
  bush aging (20% per tick, terminal at 3) — the per-position hash
  rolls of the nether-wart pattern
- berry bush full contract: player — moving-only, stage-1+, 1 HP per
  0.5 s on the SHARED hazard accumulator (vanilla's global immunity
  window — a bush + a campfire together still cost 1 HP per 0.5 s),
  horizontal velocity scaled 34.05%; mobs — the same through
  `hazard_tick` (AI steering → hazard scale → physics move, so the
  steady-state pace is the verified row); FOXES IMMUNE to both halves
- campfire: 4-slot fuel-less 600-tick cooking
  (`vc-gameplay/src/campfire.rs`, cooking gated on the LIT world
  state), completions auto-eject on top (the no-campfire-UI
  adaptation — disclosed; vanilla extracts via hopper), breaking
  drops the raw food + 2 charcoal (no Silk Touch — the self-drop row
  unreachable, disclosed), water contact extinguishes (BOTH the
  falling and horizontal flow arms — VERIFIED "waterlogging it"),
  smoke particles (10 blocks / 24 over a hay bale — the signal fire,
  tinted + taller)
- the player's standing-on-lit-campfire damage rides the same shared
  accumulator (no sneak exemption — unlike magma)
- barrel: 27 slots on the containers path (`slot_count(BARREL)`), own
  `Container::Barrel` + `ContainerKind::Barrel` (the BARREL title, the
  chest grid geometry), hoppers interact for free through the
  containers map
- crafting: stick ×2 orientations (the no-rotation-pass constraint
  disclosed per the shulker-column precedent), campfire ×2 rows (coal
  + charcoal fuels, AnyWood = the 6 logs, AnyPlanks = oak + jungle),
  barrel (6 any-planks + 2 OAK_SLAB caps — the single-slab registry
  is the "any slab" stand-in, disclosed)
- furnace: BAMBOO 50-tick fuel, CHARCOAL 1600 (coal parity), the 6
  logs smelt into charcoal
- food: SWEET_BERRIES (2 hunger → 1.0 HP); planting on
  grass/dirt/podzol/snow-grass before the eat branch (the plant-first
  interaction order)
- bamboo placement: soil → SHOOT, on bamboo → stalk, other supports
  DENIED (the u16::MAX sentinel through the placement chain)

**The fox (MobKind::Fox, MOB_DATA row 41):** 10 HP / 2 HP (E/N — 3 on
Hard via difficulty_scale) / 0.3 speed / 0.7×0.6 / 1 XP. Spawning:
taiga packs at a 25% roll + the Snowy 20% share (the 1.10
icy-family-only restriction's one later-bracket exception —
disclosed). AI (the ocelot pattern): wild flee (the 6-block ocelot
scare radius — the wiki gives no fox-specific figure, disclosed
approximation; trusting bit 0 stays), prey scan for chickens +
rabbits anywhere and beached cod/salmon/tropical-fish + baby turtles
("while they are on land" — the exact vanilla gate), 2-HP bites on
contact. Life-cycle clocks (environmental, the turtle precedent):
baby 0x40 matures on the 24000-tick aux countdown; love 0x80 expires
at 600. Breeding: `try_feed_fox` — first feeding arms love, a second
feeding with a loving adult within 8 blocks pairs them (both loves
clear; the vanilla 5-min cooldown simplified to a clean clear —
disclosed), the game layer spawns the trusting cub (0x41 + aux
24000). Death drops nothing (the body's loot is carried-item
equipment — the standing carried-items deferral).

**The picker gap fix that rode along:** the 1.13 rounds shipped the
V9 blocks but never added creative-picker rows — this round adds
BOTH windows (V9's 61 + V10's 9; PICKER_BLOCKS 321→386, every entry
reachable through the scrollable grid).

**E2E/CI:** the wasm command console grew `v114:<ticks>`; the native
smoke grew the `E2E_V114=1` env stage (the same sequence at world
entry — campfire fed a potato, barrel, mature bush, shoot, fox; a
650-tick full-scope fast-forward), and linux-game.yml now greps
"e2e: v114 campfire lit+fed=true" + "cooked 1 item(s)" as blocking
assertions.

**Deferred with reasons (recorded per the standing protocol):** the
village half of 1.14 (village rework, pillager + outposts, ravager,
raids/Bad Omen/Hero of the Village, crossbow, bell, wandering trader,
loom/stonecutter/fletching/cartography/smithing/grindstone — needs
village/raid scaffolding), smooth stone / blast furnace / smoker /
lantern / new flowers (unresearched — the next nature rounds),
sweet-berry composting (no composter), pandas (no mob; bamboo
breeding lands with them), fox carried-item loot + pounce animation
(standing carried-items/animation deferrals), fox flee radius exact
value (no wiki figure), post-breed 5-minute cooldown, campfire
shovel-extinguish (no shovel item) + Silk-Touch self-drop,
water-bucket-on-campfire (no bucket capture path), berry-bush
Fortune rows (no Fortune), the new mob sounds (the parrot/horse
precedent — unregistered events are silent no-ops until an audio
round adds recipes).

**Verification:** 527/527 workspace tests green (+23 this round:
vc-blocks 2, vc-render 3, vc-world 2, vc-sim 3, vc-gameplay 16 —
campfire 5 + mob 8 + craft 1 + furnace 1 + containers implicit —
and the updated registry bounds), wasm32 lib check clean, release
profile clean. Local checks ran with `--no-default-features` (the
sandbox lacks the ALSA dev headers for the rodio backend; CI builds
the full audio path — the audio feature is untouched this round).

---

## Session 2026-09-08 (b) — F3 vanilla-look overhaul, --debug raw log, verification sweep

Task ID: 1
Agent: main (Z)
Task: user asked to (a) verify everything previously built still
works, (b) add a debugging switch to ALL versions ("attach --debug to
activate it in the terminal to see the log"), (c) make the F3 overlay
look like the real vanilla version "everything including the
placement, font" (clean-room), same across versions, and (d) continue
the main job.

**Verification sweep (previous work all confirmed live):** GitHub
Actions API: CI + Linux Game (single file) + Build WASM all green on
the head commit 34a02a0 (the 1.14 nature half). Full workspace test
suite re-run: **529/529 green** (527 + 2 new this session). wasm32
lib check clean. Local end-to-end lavapipe smoke re-established in
this sandbox (see below) and passes: first-run profile extraction,
intro 2.66 s, title → worldselect → create → loading → game, pointer
ladder (confined), in-game click, e2e v114 campfire/barrel/bush/fox,
F3 liveness pair (two PNG dumps differ), exit 0.

**F3 vanilla-look overhaul (the "placement, font" round):** the
overlay's VALUES were already live (previous session); what still
separated it from the vanilla 1.16.5 reference look was the TYPE
RENDERING: the shared UI font is smallcaps (a-z remap to the A-Z
slots, fixed 6 px advance) so F3 rendered ALL-CAPS at half the UI
text size with 2 px gaps between the per-line strips. This round:
- FONT extended 7 → **8 rows** (baseline row 6, descender row 7 —
  vanilla font metrics; all existing smallcaps text unchanged, the
  extra row is empty for every non-descender glyph)
- the a-z slots now hold **true lowercase shapes** (x-height rows
  2..6, ascenders 0..6, descenders 2..7; narrow b/d/h/n/u/p/q arches,
  1-px i, footed l/t) — read ONLY by the new case renderer; the
  smallcaps remap in text()/text_flat()/text_splash() is untouched so
  the rest of the UI renders exactly as before
- new `text_flat_case`/`text_width_case`: true-case, flat, and a
  per-glyph VARIABLE advance (ink-extent width + 1; space 3) so
  lowercase packs at vanilla density; '∞' (the fps line's unlimited
  framerate) gets a dedicated 5-wide clean-room glyph
- `debug()` restyled to the vanilla DebugHud metrics: per-line
  0x90505050 strips now **CONTIGUOUS** (pitch == strip height —
  vanilla stacks them seamlessly), flat **0xE0E0E0** text at the
  standard UI font size (scale 2 — vanilla uses the same font as
  every label; the old scale-1 was half size), 1 px strip padding,
  left column strip x=2/text x=3, right column **right-aligned** to a
  3 px margin, first strip at y=2; blank spacer rows advance the
  pitch without painting (vanilla group gaps)
- F3+Q help box restyled to the same case font; frame-time graph
  repositioned for the 18 px pitch
- tests: `f3_overlay_is_two_columns_with_per_line_strips` rewritten
  for the new metrics (contiguity seam assertion, 0xE0E0E0 flat-text
  check, right-margin check) + new
  `f3_case_font_is_lowercase_with_descenders` (descender row must
  paint; case 'p' ≠ smallcaps 'P'; 'i' 2 px advance; lowercase line
  narrower than its smallcaps twin; ∞ paints)
- overlay VLM-verified against the vanilla layout (two columns, right
  column right-aligned, lowercase, contiguous strips, descenders
  below baseline, no clipping); `docs/screenshots/f3-vanilla-live.png`
  regenerated. The ∞ glyph's 5-px weave reads as the ASCII
  approximation "~" (a full loop set is unreadable at 5 px —
  disclosed here; a 3-row variant read worse, as an asterisk)

**--debug (the raw diagnostic log, all versions):**
- `vc-render`: `set_verbose`/`is_verbose` (AtomicBool, default OFF) +
  `report_debug_log(cat, msg)` → `[t+  12.3s][cat] msg` through the
  existing boot-log sinks (native stderr + logs/latest.log; wasm JS
  console), timestamps anchored when the flag is set; unit test
  `verbose_flag_defaults_off_and_toggles`
- native: `--debug` parsed in main.rs before any boot line; `--help`
  / `-h` prints a usage card (no-args / --debug / --smoke / --benchmark)
- wasm: `?debug`/`?debug=1`/`?debug=true` URL param (read via the
  same js interop as boot_log — no new web-sys features);
  play.html's voxelcraftLog passes the raw [t+ lines un-prefixed
- game.rs instrumentation (all zero-cost when the flag is off):
  [screen] every set_screen transition; [input] every click routed
  with screen + coords + what's under it (crosshair target block
  in-game, widget id in menus — the input-regression detector);
  [world] world entry (name/seed/spawn/mode, wasm uses the player
  position — level_spawn is native-gated); [perf] first sample at
  world entry + 1 Hz heartbeat (fps envelope, frame/sim ms, chunk
  mesh/loaded/drawn, gen+mesh queue depths, mobs, edits); [save]
  autosave ms; [f3] overlay + combo toggles; [exit] uptime/frames/
  fps/edit summary on window close AND both smoke exits
- linux-game.yml: the first smoke now runs `--smoke --debug` and
  greps all five categories as blocking assertions

**Local lavapiipe harness re-established** (the sandbox /tmp was
wiped): scripts/mesa-vk + vk-icd (lavapipe ICD) restored, and the
driver's missing transitive deps re-fetched and extracted into
scripts/*-extract (lib-shim libxkbcommon-x11 + libasound + libX11-xcb,
xkb-extract, llvm-extract libLLVM-21, di-extract libdisplay-info,
xml-extract libxml2-16, wl-extract libwayland 1.26 — the lvp ICD
needs the newer wl_fixes_interface symbol, which is why it silently
dropped its surface extensions and wgpu failed with
FailedToCreateSurfaceForAnyBackend before). Xvfb must run detached
(setsid) — it dies with its parent shell otherwise, and a full /tmp
once silently broke every background job (disk 100%).

Stage Summary:
- F3 overlay now vanilla-look (lowercase case-font, contiguous
  strips, 0xE0E0E0 at standard UI size, right-aligned right column)
  with all values live; same overlay across all version brackets
- --debug / ?debug raw log on native + browser, five categories,
  CI-grepped; --help usage card
- 529/529 tests green, wasm32 clean, local smoke + F3 liveness green,
  CI green at head, README + play.html document the debug channel
- main plan: next round queued (1.14 deferred nature blocks — smooth
  stone family / blast furnace / smoker / lantern / new flowers, then
  1.15 Buzzy Bees)

---

## Session 2026-09-08 (c) — 1.14 nature half, part 2: the smelting trio + lantern

Task ID: 1 (continuation)
Agent: main (Z)
Task: continue the main plan — the 1.14 deferred nature blocks
(smooth stone / blast furnace / smoker / lantern, the "next nature
rounds" from the V10 deferral list).

**Research:** the four wiki pages captured via the minecraft.wiki
API into scripts/v114b_page_{Smooth_Stone,Blast_Furnace,Smoker,
Lantern}.json. Verified contracts: stone smelts into smooth stone
(0.1 XP); blast furnace = 5 iron + furnace + 3 smooth stone, smelts
ONLY the ore/metal class at 2x speed with fuel burning at double
rate (same items-per-fuel), lit state light 13, drops contents when
broken; smoker = 4 logs (cross) around a furnace, cooks ONLY food at
2x (5 s per item), lit 13; lantern = 8 iron nuggets + torch, light 15
("brighter than a torch"), sits on tops or hangs from undersides,
pops on invalid support at the next block update.

**Registry (V11 window, ids 426..=429, states 689..=695, tiles
634..=639):** BLAST_FURNACE, SMOKER, LANTERN + IRON_NUGGET (the
1.11-era item the lantern recipe needed); 7 states (smelters unlit/lit
pairs, lantern sitting/hanging, nugget item). BLOCK_COUNT 426→430,
STATE_COUNT 689→696, TILE_MAX 633→639, PICKER_BLOCKS 386→389, WGSL
mesh LUT + clamps resynced (the drift test caught it, working as
designed). F3 targeted-block property lines decode
("Blast Furnace[lit=true]", "Lantern[hanging=false]").

**Furnace family (vc-gameplay/furnace.rs):** FurnaceKind
(Furnace/Blast/Smoker) — cook_ticks 200/100/100, burn_rate 1x/2x, and
a class filter (blast accepts is_ore_smelting = COAL_ORE today; smoker
accepts is_food_smelting = potato/raw rabbit/kelp; rejected inputs
never ignite and never spend fuel). The world lit-swap is per-kind
(FURNACE_STATE/FURNACE_LIT for the base; the V11 pairs for the
smelters). The STONE → SMOOTH_STONE smelting row (0.1 XP) + the
smelters' contents-spill on break ride the existing paths.

**Crafting (VERIFIED grids):** blast furnace (iron top+sides,
furnace center, smooth stone bottom), smoker (the 4-log cross),
lantern (nugget ring + torch), and the 9:1 nugget ↔ iron round-trip.
Disclosed stand-ins: IRON_ORE items for the iron ingots (the engine's
existing convention), REDSTONE_TORCH (light 7) for the recipe's torch
(the engine's only torch).

**Lantern gameplay:** placement writes sitting (top face) or hanging
(underside — the slab-half face pattern); light 15 through the block
row in both forms; breaking a support pops the lantern above/below
(the verified invalid-surface rule, recursion-chained down lantern
columns); water does NOT break lanterns (they are waterloggable in
vanilla — the no-waterlogging-path deferral covers it).

**Art (v114b_art.rs, painted from day one):** the blast furnace's
dark-iron face (unlit + glowing lit), the smoker's log-walled face
(unlit + lit), the lantern sprite (chain handle, iron frame, warm
glass + flame — cross-rendered like the torch; the sitting/hanging
model difference is disclosed future work), the iron nugget icon.
Guards: v114b_tiles_all_painted (the art-gap coverage) +
v114b_smelter_lit_glows (lit faces must carry warm-glow pixels, unlit
none).

**E2E/CI:** the E2E_V114 smoke stage gained e2e_v114b (blast furnace
fed coal ore + smoker fed potato — both out=1 lit=true at the 100-tick
2x cook; lantern sitting + hanging + block-light; the support-break
pop) — includes the light-engine pump() call the game loop normally
makes (a direct-sim fast-forward leaves seeds pending, light reads 0).
linux-game.yml greps the three v114b lines as blocking assertions.

**Deferred with reasons:** lantern waterlogging (no waterlogging
placement path), the smelters as villager job-site blocks (armorer /
butcher professions — the village half), blast-furnace metal-tool/
armor smelting (no tool items), gold ore → gold ingot (no gold ingot
item; the iron-ore-as-ingot stand-in convention makes iron-ore
smelting degenerate — the ore class grows with a future ingot round),
smoker chorus-fruit row (no chorus), lantern chain-connect rendering.

**Verification:** 537/537 workspace tests green (+8: furnace 3 + craft
1 + render 2 + blocks 1 + the smooth-stone smelt), wasm32 lib clean,
release build clean, local lavapiipe smoke green end-to-end with the
v114b stage (out=1/lit=true/100-tick cooks, neighbor-block-light=15,
popped=true).

---

## Session 2026-09-08 (d) — web-preview panic fix + 1.14 part-3 flowers

Task ID: 1 (panic fix) + 2 (main-plan continuation)
Agent: main (Z)
Task: (1) fix the website preview dying at boot with "Engine error:
panicked … time not implemented on this platform"; (2) continue the
main plan — the last deferred 1.14 nature item, the flowers.

**Reproduction:** headless browser against the dev server (port 3000)
— the boot overlay showed `Engine error: panicked at
library/std/src/sys/time/unsupported.rs:13:9: time not implemented on
this platform`, with the stack pointing into the wasm bundle.

**Root cause:** `std::time::Instant::now()` COMPILES on
wasm32-unknown-unknown but PANICS at runtime. Two wasm-reachable call
sites shipped in commit 2c29757's auto-rebuilt bundle: (a) the boot
diagnostics timer `t_boot = std::time::Instant::now()` in
`GameApp::new` (added with the staged intro — fires on EVERY boot,
the reported crash), and (b) the `--debug`/`?debug` timestamp anchor
`DEBUG_START.get_or_init(std::time::Instant::now)` in vc-render
(fires only with the flag on). The CI compile gate can't catch this
class (it compiles), the native smoke runs the same code where
std::time works, and the wasm auto-rebuild workflow pushes its bundle
with [skip ci] — no browser replay anywhere. All other time sites
were already cfg-gated correctly (boot_uptime, random_seed, autosave,
the gpu_mesh stall watchdog, bench micros — each with its wasm arm).

**Fix (web-time):** the `web-time` crate — wgpu's own solution, a
`std` re-export on native and `performance.now()`/`Date` on wasm —
already in Cargo.lock 0.2.4 via wgpu's dependency tree, so no new
resolution. Added as a workspace dependency and swapped in at:
- `voxelcraft/game.rs` — the boot timer (t_boot)
- `vc-render/render.rs` — the DEBUG_START anchor (type + both
  get_or_init sites)
- `vc-anvil` `now_secs()` + the level.dat LastPlayed stamp (native-
  only today, but the same trap if saves ever un-gate — defense in
  depth)

**Local toolchain (this sandbox had none):** rustup stable +
wasm32-unknown-unknown target + wasm-bindgen-cli 0.2.127 (pinned to
the crate), matching CI. Local wasm build via
scripts/build-wasm-bundle.sh (the CI recipe verbatim: cargo release
lib -> wasm-bindgen --target web -> patch-wasm-glue.py -> public/).
Native tests needed ALSA headers without root: apt-get download
libasound2-dev + dpkg -x into ~/alsa-dev, patched alsa.pc paths,
PKG_CONFIG_PATH (alsa-sys is pkg-config-only).

**Browser verification (agent-browser, after the fix):** clean boot
(overlay hides, canvas 1280x577 live, "intro complete … title in
2.62s"), ZERO panics in the console, `?debug` streams live
`[t+ 8.3s][screen] intro -> title` (the second panic site fixed), and
a full interactive replay through the input shim: title -> options
-> title -> create -> loading -> **world entered** ("New World" seed
12846350442853681743 spawn (0,19,0) mode Survival) with the [perf]
heartbeat live (fps/chunk/mob counters). Screenshots:
docs/screenshots/web-preview-fixed-{boot,gameplay}.png. (Also
root-caused a display artifact along the way: the Bash output pipeline
eats literal "[h" sequences — "Lantern[hanging=true]" READS as
"Lanternanging=true]" in terminal output; the files were always
correct. Do not "fix" strings from terminal display — verify with the
Read tool first.)

**1.14 part-3 flowers (the main-plan continuation):** research
captures scripts/v114c_page_{Cornflower,Lily_of_the_Valley}.json
(minecraft.wiki, clean-room). Verified contracts: both 18w43a,
non-solid cross plants, instant-break, drop themselves, plantable on
the grass/dirt family; cornflower crafts 1:1 into blue dye and
generates in plains/sunflower plains/flower forest; lily of the
valley crafts 1:1 into white dye and generates in the forest family
(forest, birch, flower forest; dark forest out of bracket).

Registry: V11 window grown to ids 426..=431 (CORNFLOWER 430,
LILY_OF_THE_VALLEY 431), states 689..=697 (one per flower, no
properties), tiles 640..=641; BLOCK_COUNT 432, STATE_COUNT 698,
TILE_MAX 641, PICKER_BLOCKS 391 (both flowers in the creative
picker; not item-blocks). The WGSL gpu-mesh LUT offsets + defensive
clamps resynced (L_FL 698, L_TC 1130, L_ST 1562, min(s,697),
min(b,431)) — the wgsl_lut_offsets_match_rust drift test green. All
14 registry-count drift asserts bumped (the audit trail's 430/696
hardcodes).

Gen: the flower-forest small-flower mix grew 8-way -> 10-way
(cornflower slot 8, lily slot 9); a dedicated Plains arm (tall grass
72% / poppy / dandelion / cornflower 10%); a Forest|BirchForest arm
(lily 10%). First test draft flaked (10 plant attempts x 8% lily over
4 chunks rolled zero on the fixed world seed) — lily slice raised to
10% and the test scans 8 chunks; green 4/4.

Crafting: the flower->dye pair as 1x1 shaped recipes (the
log->planks convention): CORNFLOWER -> DYE_BASE+11 (blue, the lapis
row), LILY_OF_THE_VALLEY -> DYE_BASE (white). The engine's first
flower dye recipes — the 1.7 flowers remain unwired (disclosed
deferral).

Art: vc-render/src/textures/v114c_art.rs — the allium cross-plant
layout; cornflower (deep-blue petal cluster D/B/W tones, stem +
paired leaves), lily (white bell florets W/w along the stem, broad
basal leaves). Dispatch arms in textures.rs; the
v114b_tiles_all_painted coverage guard auto-extends to 641.

E2E: e2e_v114c stage (rides the shared E2E_V114 gate) — both flowers
planted on grass, states round-trip, F3 targeted-block lines decode
the plain names, both dye crafts resolve via match_grid, and the
instant-break contract; one consolidated boot-log line, with four
new blocking greps in linux-game.yml.

Deferred with reasons: wither rose (needs the entity-damage aura +
wither-kill acquisition), suspicious stew (no stew system),
bone-meal post-generation (no composter), flower pots for the new
pair (the engine's pot is decorative-only), 1.7-flower dye recipes.

**Verification:** 539/539 workspace tests green (+2: the flower dye
pair + the biome gen test; the v11 registry test extended in place),
wasm32 lib clean, the browser preview verified end-to-end as above.
Bundle in public/ rebuilt locally (CI rebuilds on push). Pushed with
README bracket 11c + maintenance note 8.

Stage Summary:
- Web preview fixed: web-time replaces every wasm-reachable
  std::time call; preview verified boot -> title -> world entry in
  a real browser with zero panics and ?debug live
- 1.14 nature half COMPLETE (parts 1-3): bamboo/berry/campfire/
  barrel/fox + smelters/lantern/nugget + cornflower/lily — next
  main-plan round: 1.15 Buzzy Bees
- Standing lesson recorded: wasm32 "compiles but panics" APIs need a
  browser replay in CI, not just a compile gate (candidate follow-up)

---

## Session 2026-09-08 (e) — the exact vanilla 1.16.5 settings tree + hover tooltips

User report: the settings screens had FEWER options than the real game, no
hover hints, and two engine options (GPU chunk meshing, occlusion culling)
nobody could look up. This round rebuilds the whole settings UI to the
vanilla architecture — "not similar, exact".

**Video Settings = the exact vanilla 1.16.5 screen** (`layout_video`): the
full-width Render Distance slider, the four two-column cycling rows
(Graphics Fast/Fancy/Fabulous! | Smooth Lighting
OFF/Minimum/Maximum, GUI Scale Auto/1/2/3 | Clouds OFF/Fast/Fancy,
Particles All/Decreased/Minimal | Full Screen ON/OFF, Use VSync ON/OFF |
Entity Shadows ON/OFF), the UNLABELED Brightness slider (hover tooltip
reads Moody/Bright from the live value — the vanilla behavior), the
full-width Biome Blend slider, Done. 11 options, vanilla proportions on
the 1.5x canvas: 150x20 vanilla buttons → 225x30, 310x20 → 465x30, 36px
row pitch from y=72. New `text_frac` (fractional-scale nearest-neighbor
font renderer) supplies the 1.5x-class widget text; the old 44px widgets
render identically to before (fs = h*0.05 capped at 2).

**Every option is functional:**
- Smooth Lighting Minimum: the new half-strength AO level — the
  `(a+3)/2` corner remap in the CPU mesher AND the WGSL compute mesher
  (`smooth: bool` → `smooth: u8` through Job/GpuMeshPending/meta/params);
  the gpu_mesh parity test now cycles all three levels and stays
  bit-identical, and the golden terrain hash is unchanged at maximum.
- Clouds Fast/Fancy: Fast keeps the solid opaque plane; Fancy is a new
  `cloud_pipe_blend` (ALPHA_BLENDING) — the shader's 0.55 alpha finally
  blends. MSAA pipe set carries both variants.
- GUI Scale: `scale_widgets` re-scales every menu's widget rects around
  the canvas center; `UiCanvas::widget_scale` drives matching fractional
  text; hit tests share the scaled rects so input stays exact. Deferral
  disclosed: the HUD's edge-anchored scaling is the remaining half.
- Particles All/Decreased/Minimal: `ParticleSystem::density` (1.0/0.5/
  0.25) as rejection sampling in spawn_block_break + spawn_hit.
- Full Screen: real borderless winit fullscreen, applied at boot + on
  toggle (persisted).
- Use VSync: `Renderer::set_vsync` (Fifo vs AutoNoVsync/Mailbox/
  Immediate fallback); MAX FPS left the vsync role (now UNCAPPED/30/60/
  120 on the engine page).
- Entity Shadows: `push_mob_shadows` — one blended ground quad per
  visible mob (glass texel alpha + dark tint; ground scan ≤ 8 below the
  feet), through the billboard pipeline.
- Biome Blend: `blended_biome_pad` — the mesh-time tint pad becomes the
  nearest-LUT-slot average of the neighborhood biomes' grass colors
  (radius 1/2/3 for 3x3/5x5/7x7, world-side sampling via get_biome);
  injected in `Job::Mesh::biomes` so the CPU and GPU paths share one
  blend pass; remesh_all on change; default 3x3 (the vanilla look).
- Graphics: the existing 0/1/2 Fast/Fancy/Fabulous! cycle IS the vanilla
  1.16.5 cycle — relabeled and kept.

**The hover hint system** (the user's ask): resting the pointer on any
option draws 1-2 centered gray lines directly under the screen title —
the vanilla hint slot. `tooltip_for` covers every option on every
screen (unit-tested: `every_settings_option_has_tooltip`), with clean-
room wording describing what each option does in THIS engine (GPU chunk
meshing and occlusion culling finally explain themselves in-game).

**Tree restructure**: main Options = the vanilla layout (Music|Sound,
FOV|Sensitivity — "FOV: Quake Pro" at 110 — and the six sub-screen
buttons; Chat/Language/Controls grayed stubs until their subsystems
exist, the MULTIPLAYER pattern); **Resource Packs** = a real selectable
list (engine shader modes + packs — replaces the old SHADERS cycle
button); **Accessibility** hosts Auto-Jump (its true vanilla 1.16.5
home); **Engine Settings** is the one disclosed extra page (GPU meshing,
occlusion, mip/aniso/MSAA, sim distance, frame cap, FSR, sun shadows)
so the vanilla screen stays exact. ESC returns sub-screens to Options;
Done follows the vanilla parent chain; options_from drives the panorama
backdrop consistently.

**Persistence**: seven new serialize keys (smoothl/cloudsl/gui/part/fs/
vsync/eshad/bblend; the legacy smooth/clouds bools still parse →
levels, and the legacy clouds=1 maps to Fancy). Round-trip + legacy
tests extended; the stats pairs expose the new values; wasm
localStorage verified live in the browser.

**E2E + CI**: the smoke suite gained `E2E_MENU=1` — a scripted click
through Options → Video (toggles) → Engine → Packs → Access → back to
the title via the REAL input path, exiting clean with
"e2e: settings tree ok (video/engine/packs/access)". linux-game.yml
runs it as a second blocking smoke stage with screen-transition greps.
`UI_DUMP_DIR` renders every settings screen headlessly (pure-CPU UI
canvas) — the docs screenshots
`vanilla-{options-main,video-settings,engine-settings,resource-packs,
accessibility}.png` come from the test, VLM-verified line-by-line
(exact option sets, tooltip text, hover highlight, unlabeled
brightness).

**Verified end-to-end in the real browser** (agent-browser + local
bundle rebuild → public/): boot → title → options → video → hover
GRAPHICS (two-line hint live) → toggle to FABULOUS! → settings persisted
with all new keys (`voxelcraft.settings`) → world entry with the
default 3x3 biome-blended meshing, full HUD, zero page errors.

Deferred with reasons: HUD edge-anchored GUI scaling (menus scale
today), vanilla's two-pane resource-pack layout (single list),
entity-shadow softness (quad, not the blob texture), full screen on
web (winit canvas API — preference persists), the Chat/Language/
Controls subsystems themselves.

**Verification:** 543/543 workspace tests green (+4: vanilla video
layout, tooltip coverage, GUI-scale math, new-key round trip, the dump
helper), wasm32 lib clean, gpu-mesh parity extended to 3 levels, the
golden terrain hash unchanged, browser replay green. Bundle rebuilt
into public/ locally; CI rebuilds on push.

Stage Summary:
- The settings UI is now the vanilla 1.16.5 tree: exact Video screen
  (11 functional options), vanilla main Options, Resource Packs,
  Accessibility, one disclosed Engine page — every option with a hover
  hint
- Smooth Lighting is a real three-state AO level in BOTH meshers;
  Biome Blend blends the tint pad for both meshers; Clouds Fast/Fancy
  are two pipelines; GUI Scale, Particles, Full Screen, VSync, Entity
  Shadows all live
- E2E_MENU settings-tree click stage is a blocking CI gate; UI_DUMP_DIR
  regenerates the docs screenshots headlessly; browser replay verified
  boot → settings → toggle → persist → world entry with zero panics
- Next main-plan round: 1.15 "Buzzy Bees" (the bracket's nature half —
  bees, hives, honey)

---

## Session 2026-09-08 (f) — 1.15 "Buzzy Bees" complete (recovered from an interrupted session)

The round was implemented in one push but committed by the session
auto-save without its docs; this entry records what landed (all values
live-verified against the captures `voxelcraft/scripts/v115_page_*.json`:
Bee, Bee_nest, Beehive, Honey_Block, Honey_Bottle, Honeycomb,
Honeycomb_Block; research record `docs/research/phase-v115-1.15-research.md`).

**Registry — the V12 window (blocks 432..=439, states 698..=715, tiles
642..=654, BLOCK_COUNT 440 / STATE_COUNT 716, WGSL mesh LUT resynced):**
BEE_NEST + BEEHIVE (honey_level 0..=5 blockstates each — 12 states),
HONEY_BLOCK, HONEYCOMB_BLOCK, and the items HONEYCOMB, HONEY_BOTTLE,
SHEARS (the LEGACY-item precedent — first consumer is the harvest), and
the bee spawn egg (mob kind 41).

**The hive system** (`vc-gameplay/src/bees.rs`): lazy round-robin
registration of nests/hives in the sim ring (one chunk per 20 ticks,
the full 17x17 ring guaranteed), generated nests carry 2-3 bees with
staggered work timers; the in-hive work clock (2400 ticks) with the
daylight release gate; honey_level bumps ONLY for pollinated bees
(+1, the 1% +2); capacity 3 with refusal; anger (the harvest/break
swarm releases stored bees immediately); campfire pacification (the
5-below check); honey_level readback for the harvest path.

**The bee mob** (`mobs.rs` kind 41): 10 HP arthropod, spawns "in any
difficulty including Peaceful", sting 2 HP (Easy/Normal) / 3 HP Hard
with Poison I 10 s (Normal) / 18 s (Hard); one sting per bee then the
1200-tick stinger-less death; anger propagation to nearby bees; the
pollination loop (flower seek, crop pollination queues the growth
bump); flies() — no gravity.

**World gen**: bee nests on generated oak/birch trees at the verified
per-biome chances (plains 5%, flower forest 2%, forest 0.2%) — placed
against the trunk with the hash roll `0xBEE5`.

**Honey-block physics** (`player.rs`): the 2.508 m/s walk clamp (0.58
per-tick factor, the bush-slow pattern) + the 85% jump-height cut
(0.39 velocity factor). Honey-bottle food + the honey-sliding
disclosure: no slide-boost (the horizontal-velocity trick), only the
verified slow/jump rows.

**Crafting** (`craft.rs`): honeycomb block (4 combs), honey bottle x4
(honey block + 4 glass bottles), shears (2 iron — the LEGACY window).

**Art** (`v115_art.rs`): nest/hive front+top faces (the honey drip
grows with honey_level), honey block, honeycomb block, the comb /
bottle / shears / egg items, the bee billboard — coverage-guarded.

**E2E**: the `E2E_V115=1` smoke stage (2600 ticks — hive registration,
level 0/5 reads, the craft set, an entered bee's full work cycle,
campfire pacify, swarm) prints `e2e: v115 ...` boot lines.

**Test fix this session**: `hive_work_cycle_bumps_honey` entered two of
its three bees WITHOUT nectar but asserted three honey bumps — vanilla
bumps only for pollinated bees ("Every pollinated bee that leaves the
hive after working increases the honey level by one"). The engine was
right; the test setup now uses three nectar bees. 558/558 green.

Deferred with reasons (the standing classes): piglin-less bartering is
1.16; dispensers-with-shears (no dispenser interactions); waxing/
candles (1.17); sugar-from-honey (1.21.2+); meadow/cherry/mangrove
spawn-table rows (post-1.16.5 biomes); the pet-bee April-Fools forms.

Stage Summary:
- 1.15 Buzzy Bees is complete: hive system + bee mob + nest gen +
  honey physics + recipes + art + E2E, all wiki-verified
- The interrupted session's commit is now fully documented (this
  entry + the README bracket note)
- Full suite 558/558 green (the one failing test was a test-authoring
  bug, fixed)

---

## Session 2026-09-08 (g) — web preview "loading engine" fix + verification sweep

User report: the website preview stuck at "Loading engine…" with no
error. Root cause (git archaeology): the settings commit e0bdf0f
regenerated `public/voxelcraft.js` (new fullscreen imports + the
`__wasm_bindgen_func_elem_*` table indices shifting with the new
binding set) but never copied the matching `voxelcraft_bg.wasm` — the
last binary was two commits older (ff7f244). The mismatched pair
INSTANTIATES (extra glue imports are harmless) but winit's event-loop
setup then calls a wrong-index function element and the engine dies
silently before window creation: canvas stuck at 300x150, boot overlay
never hides, zero console output — exactly the symptom. GitHub's Pages
copy was healthy (CI's own rebuild f582eab is a matched pair); only
THIS container's preview server (which serves the repo's public/ via
Next.js) had the broken pair.

**Fix**: full bundle rebuild from HEAD (settings tree + 1.15): wasm32
release lib -> wasm-bindgen 0.2.127 -> `patch-wasm-glue.py` (the
pointerType hardening, import id `__wbg_pointerType_b3dafa8fb9c97016`,
getObject mode) -> all four files + the builtin pack copied to
public/ -> committed as a proper fix (31cdc1c after rebase onto the
CI bundle commit, -X theirs to keep the HEAD-built pair) -> pushed.

**Browser-verified end to end on the preview server (localhost:3000,
the same static path the public preview URL serves)**: boot -> title
2.6 s -> SINGLEPLAYER click -> CREATE NEW WORLD click -> CREATE WORLD
click (the REAL input pipeline, canvas 1280x577 scaled from the 960x540
UI space — the play button centers at (640,256)) -> loading complete
(5 chunks on GPU in 2.3 s) -> game screen, boot overlay hidden, zero
console errors/panics. The mouse-click path the earlier regression
complained about is live-verified working in the browser.

**Toolchain note for future sessions**: this container started with NO
Rust toolchain — rustup stable 1.98.1 + wasm32-unknown-unknown +
wasm-bindgen-cli 0.2.127 (the lockfile pin; 0.2.108 was installed
first and replaced — versions MUST match the crate or the glue hashes
drift).

Stage Summary:
- Web preview fixed, verified, pushed (31cdc1c)
- 558/558 tests green; the 1.15 test-authoring bug fixed (0088a75)
- Next: the 1.16 Nether Update bracket (nothing of it exists yet —
  no basalt/blackstone/soul soil/target/anchor/striders/piglins)
