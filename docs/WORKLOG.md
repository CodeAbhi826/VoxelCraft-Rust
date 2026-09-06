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
