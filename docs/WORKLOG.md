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
