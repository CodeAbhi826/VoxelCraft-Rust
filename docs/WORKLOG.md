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
