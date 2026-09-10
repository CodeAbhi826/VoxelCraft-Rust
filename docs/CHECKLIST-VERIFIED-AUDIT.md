# VoxelCraft — The Verified Master Checklist (1.16.5 target)

> **What this is.** An independent, source-checked audit of the project against
> a 100%-replica target (Minecraft Java Edition 1.16.5), built by cross-checking
> the two uploaded third-party checklists (`chat-Minecraft Clone Development
> Checklist.txt`, ~1,200 items; `checklist_MASTER_CHECKLIST.md`, S00–S22 + L)
> against (a) the actual code at HEAD `61a92f9` and (b) the live primary sources
> (minecraft.wiki pages fetched 2026-09-09, minecraft.net Usage Guidelines
> fetched 2026-09-09). Neither source file was trusted blindly — both were
> fact-checked, and both were found to contain errors in *opposite directions*.
>
> **Legend:** ✅ done & verified · 🟡 partial (what exists + what's missing) ·
> ❌ not done (with the documented reason) · ⚠️ a claim in a source file that
> needed correction.

---

## Part 1 — Verdict on the two source checklists

### 1.1 File A: `chat-Minecraft Clone Development Checklist.txt`

**Character:** AI-generated chat transcript, 25 sections, ~1,200 items.
**Snapshot date:** the 185-commit era (claims 575 tests, 479 blocks, 14 biomes,
"3 mobs with AI", 2 dimensions, no The End, "15–20% coverage").

| Problem class | Finding |
|---|---|
| **Stale status** | Every "Repo Status" cell reflects a repo state ~9 rounds old. Current truth (updated 2026-09-10 forensics round): 193 commits, **628/628 lib tests green** (re-run locally on a fresh toolchain), 515 blocks / 845 states, 27 overworld+nether biomes + the End, 49 mob defs (50 MobKind — the backlog round added the zombified piglin), 3 dimensions, weather + farming landed. Dozens of its "❌ Missing" items are ✅ now (The End + dragon fight, combat cooldown/crits/sweep, armor formula, weather-independent spawning rules, 15 villager types, brewing, enchanting, most redstone components, pistons, observers, daylight sensors, comparators, campfires, sweet berries, coral, kelp, bees, the entire 1.16 set…). |
| **Version anachronisms** | Lists **1.17+/1.19 content as 1.16.5**: glow squid (6.1.9), goat (6.2.9), frog/tadpole (6.1.32/33), powder-snow bucket (4.3.13), freeze overlay (12.1.22), allay in pillager outposts (2.4.12), "drowned drop copper" (6.2.10 — copper is 1.17). |
| **Factual errors** | Oxygen "300 (15 bubbles)" (3.1.14) — the HUD shows **10** bubbles (its own §12.1.6 agrees); weather "rain 0.5–7.5 days" (10.4.6) — rain *lasts 10–20 min*; the *clear* period is 0.5–7.5 in-game days (live wiki, Weather §Java Edition mechanics); "30 painting variants" (4.3.37) — 1.16.5 has **26**; villager "13 professions" (6.1.16/7.1) — 13 *employable* + nitwit + unemployed = **15 types** (the repo's own framing). |
| **What it got right** | The villager **gossip table values are exactly correct** (verified against the live wiki — see §6.2 below); Ghast **10 HP** (File B says 16 — wrong); crit conditions (falling + ≥84.8% cooldown) correct; sword damage table, fall-damage formula, terminal velocity, ladder speeds all correct; the legal section's framework is sound. |

### 1.2 File B: `checklist_MASTER_CHECKLIST.md`

**Character:** "MiniMax Agent" checklist, S00–S22 + Legal, current-era snapshot
(knows about brackets 13–14). Bottom line claims **"~70% done"**.

| Problem class | Finding |
|---|---|
| **Systematic over-claiming** | The "Repo coverage" column says "In repo" for entire subsystems that do **not exist**. Worst offenders, all verified false at HEAD: **S18 raids/patrols/outposts/wandering trader** (every row — the code itself says "the village half of 1.14 (villages/pillager/raids/crossbow/bell/wandering trader/loom/stonecutter) is deferred"); **S16.01 "32 status effects — In repo"** (16 exist); **S21.F01 "600+ sound events"** (41 synthesized events); **S21.F07 "music disc 13 — In repo"** (no discs, no jukebox; two procedural music pads); **S21.D12–D15 F3+G/F3+P/F3+T/F3+1..4/F3+Esc** (only F3+Q, F3+1, F3+H exist); **S21.E controls** (F1/F2/F5/F11/Q-drop/Tab/T-chat/R/F-offhand/C-zoom/F4 — none exist); **S05.03/S05.06 Soul Sand Valley & Basalt Deltas "In repo"** (no trace in code); **S04.30 bastion remnants** (a gilded-blackstone *blob stand-in*, disclosed — not the structure); **S20.32 "entity animations: limb swing, head yaw"** (mobs are 2D billboard sprites). |
| **Invented/garbled vanilla values** | Ghast 16 HP (real: **10**); zombie "0.5 attack" (real: 2.5/3/4.5 by difficulty); moon "4 day cycle" (real: **8**); Nether "1.16 doubled [height]" (**false** — the Nether has always been 128 blocks tall; 1.16 did not double it); bedrock "Y=0 ceiling + Y=127 floor" (inverted — floor at bottom, ceiling at top); beacon "level 1–9, range 20–100" (real: levels 1–4, range 20/30/40/50); conduit "16-block range" (real: 32–96 by frame size); target block "comparator output 1-21" (impossible — redstone max is 15; the repo correctly does 1–15); golden apple "regen I 30s, absorption II 5 min" (real: Regeneration II 5 s, Absorption I 2:00); raid wave counts "5 waves: 3-8, 5-9, 8-12…" (invented — Easy 3 / Normal 5 / Hard 7 waves, composition scales with Bad Omen); witch "1.16.5 new shrinking potion" (no such thing); iron ingot as brewing ingredient (no); enchant-power formula (garbled); villager gossip table (garbled — see §6.2); "default font: M+ (Open Source)" (vanilla's ASCII font is a custom pixel font, with GNU Unifont as the Unicode fallback); "illusioner … not in 1.16.5 (1.14+ removed)" (**false** — the illusioner exists in 1.16.5, unused; this repo even implements it); "Boat: 0.4 base speed" (boats travel ~8 m/s); "tropical fish 22,000 variants" (2,700 natural). |
| **What it got right** | Paintings = 26 in 1.16.5 (File A says 30 — wrong); Wither 300 HP JE; 38 enchantments; 15 villager professions framing; Pigstep as the 1.16 disc (13 discs total in 1.16.5); the day-length suspicion ("10 min day in current impl") pointed at a **real README staleness** — the *code* is correct (20-minute day, unit-tested `DAY_LEN_SECS == 1200`), the README said 10; axolotl/bundle/sculk/bogged correctly marked 1.17+/1.21 skips; End Poem = Julian Gough + copyright warning (correct, and the engine ships no End credits — that's the right call legally); the L.1 legal landscape table is largely sound (see Part 3 for the 2026-current corrections). |

### 1.3 The one-sentence verdict on each

- **File A** is *factually decent but 9 rounds stale* — useful as a vanilla-behavior
  reference, dangerous as a status report; ~40% of its "Missing" rows are done.
- **File B** is *current-aware but systematically over-optimistic* — its
  "70% done" and its S18/S16/S21 "In repo" rows would send someone hunting for
  features that don't exist. Its real contribution is the item-count scale
  (a bit-exact replica really is 10⁴–10⁵ items).
- **Neither file** notices the engine's *disclosed deferral architecture* — the
  code deliberately documents every out-of-capability class at the deferral
  site (`docs/research/audit16-completeness-research.md` §deferral class).
  That inventory is the honest gap list, and Part 2 of this document
  consolidates it.

---

## Part 2 — The verified checklist, subsystem by subsystem

Ground truth at HEAD `61a92f9` (2026-09-09): **188 commits**, 15 crates
(14 libraries + app), ~96,400 lines of Rust, 593/593 tests green
(0 failed, 1 ignored — re-run locally today with `--no-default-features`;
CI runs the full-featured build on ALSA), `cargo check` zero-warning on
native + wasm32, 113 screenshots, 15 research documents.

### 2.1 Engine core, rendering & post-processing — ✅ (strongest layer)

| Item | Status | Notes / why-not |
|---|---|---|
| wgpu renderer (Vulkan/DX12/Metal + WebGPU/WebGL2 fallback) | ✅ | One codebase, two targets; the same source runs native and browser. |
| 16×256×16 chunks, paletted, copy-on-write | ✅ | Correct 1.16.5 dimensions (16×384×16 is 1.18+ — File A wisely pinned this). |
| Greedy meshing + per-vertex AO (OFF/Minimum/Maximum) | ✅ | Smooth-Lighting *Minimum* is the `(a+3)/2` remap in both meshers, bit-identical CPU/GPU by parity test. |
| Skylight + block light BFS engines | ✅ | 15→0 propagation both channels. |
| Mipmaps, aniso, MSAA, FSR 1.0, occlusion-flood cache, frustum culling | ✅ | All present and user-togglable. |
| Texture atlas, half-texel inset, `textureSampleGrad` | ✅ | Tile-safe filtering — no atlas bleed. |
| GPU compute mesher (WGSL) | ✅ | Bit-identical to the CPU mesher (drift guards enforce STATE_COUNT sync). |
| Shader-pack pipeline (Iris-format structure) | ✅ | 2 built-in packs (`moonlit`, `warm-evening`); WGSL-validated at runtime. |
| Render/simulation distance split | ✅ | Independent settings. |
| Sky: gradient, sun, moon (8 phases), stars, fog, clouds (Fast/Fancy), sunset band | ✅ | Moon 8-phase cycle = 8 in-game days (File B's "4 day" is wrong). |
| First-person hand / held-item 3D render; F5 third person | ❌ | No player model at all (billboard mobs, billboard-free player). Why not: the renderer has no skeletal/entity mesh pipeline yet — the single largest visual-fidelity gap. |
| Weather rendering (rain/thunder/lightning/snow) | ✅ | The two-flag Java weather machine (2026-09-09 backlog round): rain/thunder at the wiki cadences, lightning strikes with mob conversions, rain/snow particles, storm sky-darkening. |
| Post-chain extras (bloom/vignette/ACES/chromatic aberration) | ✅ | Cinematic mode set. |

### 2.2 World generation

| Item | Status | Notes / why-not |
|---|---|---|
| Seeded deterministic terrain (simplex 2D/3D, multi-octave) | ✅ | Java-LCG-compatible RNG crate (`vc-rng`). |
| Overworld biomes — 22 of ~61 | 🟡 | Have: ocean, beach, plains, forest, desert, snowy, mountains, taiga, birch, jungle, savanna, swamp, badlands, mushroom fields, flower forest, sunflower plains, ice spikes, dark forest, warm/lukewarm/cold/frozen ocean. Missing: river, snowy taiga, snowy beach, stone shore, giant taiga, windswept variants, jungle edge, deep-ocean variants (disclosed as folded into the temperature families). |
| Nether biomes — 5 of 5 | ✅ | All five: nether wastes, crimson forest, warped forest, **Soul Sand Valley, Basalt Deltas** (the last two landed in the 2026-09-09 backlog round — soul floor + fossils, the basalt floor trio, biome fog/spawn rows). |
| The End dimension | ✅ | Central island, obsidian pillars, end crystals, dragon fight, gateway, dragon egg. End *cities/ships* ❌ (no chorus-fruit outer islands, no shulkers, no elytra-in-frame). |
| Structures — 10 of ~19 | 🟡 | ✅ villages, mineshafts, ravines, desert pyramids, jungle temples, strongholds (12-frame end portal), dungeons, woodland mansions, nether fortresses, the End island. ❌ bastion remnants (blob stand-in), ruined portals (obsidian-trace stand-in), ocean monuments, ocean ruins, shipwrecks, buried treasure, pillager outposts, igloos, witch huts, fossils, end cities. |
| Ore distribution | ✅ | Coal/iron/gold/redstone/lapis/diamond/emerald/nether quartz/nether gold/ancient debris — per-wiki y-ranges and vein sizes, ancient debris blast-resistance honored. |
| Trees & flora | 🟡 | Oak/spruce/birch/jungle/acacia/dark-oak leaves+logs, huge crimson/warped fungi, 8 small flowers + 5 tall, grass/fern, dead bush, vines, weeping/twisting vines, kelp, seagrass, sea pickles, coral families (live+dead), cactus, sugar cane item, bamboo, sweet berry bushes, nether sprouts/roots. ❌ dark-oak *2×2 trunks as gen*, lily pads, cocoa, huge mushrooms (mushroom blocks exist as blocks). |
| World types (default/superflat/flat presets) | ✅ | Superflat customization supported. |

### 2.3 Player, physics, movement

| Item | Status | Notes / why-not |
|---|---|---|
| Hitbox 0.6×1.8, eye 1.62, sneak 1.5, walk 4.317, sprint 5.612, sneak ~1.295, jump 0.42/1.25 blocks | ✅ | All verified constants, unit-pinned. |
| Gravity/drag formula `(v−0.08)×0.98`, terminal 3.92 b/t | ✅ | Exact vanilla per-tick math. |
| Fall damage (−3 grace), drowning (300 air, 10 bubbles), suffocation, void | ✅ | File A's "15 bubbles" is wrong (10). |
| Sprint-jump momentum, sprint-swim, underwater walk | ✅ | |
| Honey-block physics (2.508 m/s clamp, 85% jump cut) | ✅ | 1.15 round, wiki-verified values. |
| Ladders/vines climbing | 🟡 | Vines climb (2.35 up/3.0 down, fall reset); actual *ladder blocks* not in registry. |
| Elytra gliding | ✅ | 432 durability, glide mechanics, firework boost (rockets as items). |
| Mounts | ✅ | Horse/donkey/mule with per-instance stats, taming temper, breeding; ride-drive steering with the 43.17 speed conversion. Strider riding ❌ (disclosed — no lava-mount steering). |
| Sneak edge-guard, auto-step, auto-jump (1.10 default ON) | ✅ | |
| Boats / minecarts / rails | ❌ | The standing transport deferral — no vehicle physics class. Why not: each needs an entity-physics + control layer the sim doesn't have yet. |
| Souls speed slowdown, ice friction, slime bounce | 🟡 | Soul sand slows (1.16 round); slime-block bounce (1.8 round); frosted/blue-ice friction ❌. |

### 2.4 Combat, damage, effects

| Item | Status | Notes / why-not |
|---|---|---|
| Attack cooldown (`0.2 + 0.8·p²` scaling), attack-speed attribute | ✅ | |
| Critical hits | ✅ | Falling + cooldown ≥ 84.8% + **not sprinting** (sprint ⇒ knockback attack) — exactly the live wiki conditions; File A and File B both got this right; the code comment is right. |
| Sprint-knockback, sweep (sword, ≥84.8%) | ✅ | |
| Armor formula `min(20, max(a/5, a−4d/(t+8)))/25 · 100%` | ✅ | Exact vanilla damage-reduction math. |
| Invulnerability frames (10 ticks), knockback impulse | ✅ | |
| 16 of 32 status effects | 🟡 | Have: wither, poison, regeneration, speed, haste, resistance, jump boost, strength, slowness, hunger, absorption, blindness, water breathing, slow falling, conduit power, dolphin's grace. ❌: nausea, instant health/damage, fire resistance, invisibility, night vision, weakness, health boost, saturation, glowing, levitation, luck, unluck, bad omen, hero of the village. Why not: each absent effect lacks a consumer system (potions cover 6 of the brewing families; bad omen/hero need raids). |
| Damage types (melee/projectile/explosion/fire/fall/void/magic/drown) | ✅ | |
| Bosses: Ender Dragon (200 HP, perch, breath, crystals, bossbar, 12,000 XP) and Wither (300 HP, 3 phases, skull projectiles, nether star) | ✅ | Both bossbar'd; wither build pattern 4 sand + 3 skulls verified. |

### 2.5 Items, inventory, crafting, containers

| Item | Status | Notes / why-not |
|---|---|---|
| Registry: 506 entries / 805 states / 735 atlas tiles | 🟡 | A curated subset of vanilla's ~740 block types + ~1,000 items — the engine deliberately uses the pre-flattening flat-id design; per-item coverage is the long tail. File B's stale "479" is from the V14 era. |
| Inventory 41 slots + hotbar + stacking rules | ✅ | |
| Creative picker (search + categorized) | ✅ | Grew with every round. |
| Crafting: 81 recipe rows (shaped/shapeless) + smelting (furnace/blast/smoker) + brewing | 🟡 | All the *engine-representable* recipes of the era's brackets; full vanilla is 379 recipes — the gap is the item long tail + the farming/bucket-dependent crafts. |
| Enchanting: all 38 enchantments, 3-slot table, lapis, bookshelves, glint, conflicts | ✅ | Verified count (matches both files). |
| Brewing: awkward/mundane/water + 6 effect families (healing, harming, leaping, regeneration, slow falling, turtle master), extended/II variants | 🟡 | 6 of ~15 vanilla effect potions; splash/lingering ❌. |
| Anvil (3 damage stages, combine/repair/rename) | ✅ | |
| Containers: chest, trapped chest, ender chest, barrel, hopper, dispenser, dropper, shulker box, furnace GUIs | ✅ | Hopper transfer at the verified cadence. |
| Tools/weapons: 24+ tier set incl. netherite, shield, trident, bow-class, arrows, totem, elytra | ✅ | |
| Food: ~45 items incl. the cooked-meat family, stews, honey bottle, golden apple (edible + effects) | 🟡 | Cake/milk-bucket/glistering-melon ❌ (no bucket class — disclosed). |
| Buckets, flint-and-steel, compass, clock, maps, fishing rod | ❌ | The standing item-class deferral (fishing *loot tables* exist with verified 85/10/5 splits, but no rod item). Why not: buckets need fluid-carrying state, compass/clock need NBT-tracked targets, maps need a render-to-texture pipeline. |
| Durability bar, rarity colors, item entities (despawn) | ✅ | |
| Drop key (Q), offhand (F) | ❌ | No drop/offhand input layer. |

### 2.6 Mobs & AI — 52 of ~72

| Category | Status | Detail |
|---|---|---|
| Implemented (52 kinds) | ✅ | Zombie, skeleton, creeper, spider, cave spider, silverfish, enderman, cow, pig, sheep, chicken, squid, bat, ocelot, wolf-class absent… see full list below. |
| Rendering | ⚠️ | **All mobs are 2D billboard sprites** (procedural art), not 3D cuboid models. File B's "limb swing, head yaw, walk anim — In repo" is misleading. Why not: no entity mesh/model pipeline; the billboard layer is the disclosed stand-in. |
| AI behaviors | ✅ | Wander/panic/flee, chase-melee, ranged (skeleton/ghast/blaze/witch), light-gated & biome-aware spawning, despawn rules, breeding (fungi/wheat-class foods), taming (horse temper), anger propagation (piglins, bees, zombie-villagers), group AI, baby variants, jockey-style spawns (chicken jockeys in dungeons). |
| Missing mobs (20) | ❌ | Slime, guardian, elder guardian, shulker *mob*, pillager, ravager, wandering trader, trader llama, zombified piglin, zoglin, piglin brute, wolf, cat, endermite, glow squid-class (1.17, correctly N/A), etc. Why not: the village/raid half (1.14) and several 1.13 aquatic predators are the disclosed deferral class. |

**Full implemented list** (49 MobKind + 3 module mobs): zombie, skeleton,
creeper, spider, cave spider, silverfish, enderman, squid, bat, cow, pig,
sheep, chicken, mooshroom, snow golem, iron golem, zombie villager, ocelot,
horse, donkey, mule, rabbit, polar bear, stray, husk, llama, wither skeleton,
witch, blaze, magma cube, ghast, vindicator, evoker, vex, illusioner,
parrot, drowned, phantom, dolphin, cod, salmon, pufferfish, tropical fish,
turtle, fox, bee, strider, piglin, hoglin + **ender dragon, wither, villager**.

### 2.7 Villagers — the deep system, correctly done

| Item | Status | Notes |
|---|---|---|
| 15 types: 13 professions + unemployed + nitwit, job-site blocks | ✅ | Correct framing (File A's "13" and File B's "15" are the same fact). |
| 5 trade tiers (Novice→Master) with per-tier tables | ✅ | |
| **Gossip — all five types, exact live-wiki table** | ✅ | `(gain, decay, share-cost, max, multiplier)`: trading (4, 2, 20, 25, 1) · major_positive (20, 0, 100, 20, 5) · minor_positive (25, 1, 5, 25, 1) · minor_negative (25, 20, 20, 200, −1) · major_negative (25, 10, 10, 100, −5). Verified today against minecraft.wiki/w/Villager §Gossiping — **five-for-five identical**. Share-cost semantics (major_positive unshareable) and 20-minute decay included. |
| Reputation → price | ✅ | `clamp(base − floor(rep × 0.05), 1, 64)` — the reputation term of the live wiki's full sale-price formula, with demand (d) and Hero-of-Village (h) terms zeroed because those systems don't exist (disclosed). |
| Zombie-villager curing → major_positive gossip | ✅ | Curing countdown 3600–6000 ticks, difficulty-gated conversion chances. |
| Schedules/beds/farming/breeding AI | ❌ | No villager daily schedule, bed claiming, or crop-tending loop — the biggest behavioral gap in an otherwise gold-standard system. Why not: needs a village-blocks claiming layer (beds don't exist as blocks). |
| Iron golem: mob + player-build pattern + hostile-target AI | ✅ | Village-summoned golems & reputation-based hostility ❌ (no village scheduling layer). |

### 2.8 Redstone, fluids, block mechanics

| Item | Status | Notes |
|---|---|---|
| Dust (0–15 decay), torch, repeater (1–4 delay, lock), comparator (compare/subtract, container reading), lever, both weighted plates, buttons | ✅ | |
| Pistons + sticky (12-block push), observer (update detection), daylight detector (inverted mode), target (1–15 by proximity, 8/20-gt pulse), trapped chest, tripwire hooks | ✅ | Target values verified — File B's "1-21" impossible. |
| Dispenser/dropper per-item actions | ✅ | |
| Quasi-connectivity | 🟡 | Documented behavior differences where the update graph differs — disclosed. |
| Fluids: water/lava 8-4-3 source rules, flow levels, obsidian/cobble/stone interactions, waterlogging, bubble columns, lava light | ✅ | 20 Hz sim; File B's claim that this is "too fast" vs vanilla is unproven — the tick cadence is documented per-recipe. |
| Interactive blocks: doors, trapdoors (iron trapdoor places), beds, TNT, campfire cooking, respawn anchor, lodestone, soul fire, sponge | 🟡 | Iron trapdoor ✅ (redstone-gated opening is the documented deferral); campfire ✅ (4-slot 600-tick cooking, soul variant); respawn anchor ✅ (4 charges, comparator signal, Nether-only respawn); sponge ❌ (no absorb system); **wood doors ❌, beds ❌, TNT ❌** — the standing disclosed deferral. Why not: multi-block placeable entities need a block-entity + multi-cell state layer. |
| Farmland/crops (wheat/carrot/potato/beetroot) | ✅ | The farming system landed (2026-09-09 backlog round): hoe tilling, farmland moisture 0..7 with the 4-block hydration boundary, wheat/carrot/potato age 0..7 + beetroot age 0..3 growth at the wiki denominators, trampling, dry decay, bread/hay-bale/hoe crafts. |

### 2.9 World events, time, weather

| Item | Status | Notes |
|---|---|---|
| 24000-tick day = 20 real minutes | ✅ | **Code correct and unit-tested** (`DAY_LEN_SECS == 1200`); the README's "10 min" line was stale doc text — fixed in this round. File B caught the README symptom, wrong about the code. |
| Sun/moon/stars, 8-phase moon over 8 days, sleep… | 🟡 | Day/night + moon ✅; **sleeping to skip night ❌, beds ❌**; phantoms spawn on insomnia timer ✅ (their anti-sleep prey loop is N/A without beds). |
| Weather: rain, thunder, lightning, snow | ✅ | Implemented (2026-09-09 backlog round): the two-flag state machine at the exact wiki cadences, 5 HP lightning with creeper-charging / pig→zombified-piglin / mooshroom-flip conversions and fire ignition, thunderstorm all-day hostile spawning, rain/snow particles + sound + sky factors. (File B's S19 "In repo" claim became true a day after this audit first wrote "false" — the round closed it.) |
| Raids, bad omen, patrols, hero of the village, wandering traders, zombie sieges | ❌ | All deferred (the 1.14 "village half"). File B's entire S18 is false for this repo. |
| Game rules, commands, /time etc. | 🟡 | No chat/commands; gamerules as engine settings only. |
| Difficulty (peaceful→hard) with damage/spawning scaling | ✅ | |

### 2.10 UI, HUD, menus, F3, audio, packs

| Item | Status | Notes |
|---|---|---|
| Boot flow: studio intro → panorama title (rotating cubemap, procedural Nether theme) → splash (14 **original** texts, 2 Hz pulse, yellow, tilted) → world-create → chunk-colormap loading | ✅ | Verified against screenshots; splash originality confirmed (no vanilla strings). |
| Settings tree = exact vanilla 1.16.5 screens with functional options + hover hints | ✅ | Video/Options/Accessibility/Resource Packs/Engine page; all 11 video options functional. |
| HUD: hotbar, hearts, hunger, XP, bubbles, bossbar (dragon/wither), held-item name, toasts, crosshair | ✅ | Armor icons ❌ (no armor-wearing layer), effect icons 🟡 (16 effects exist, HUD icons for the ambient set). |
| F3 debug overlay (vanilla two-column layout, translucent strips, targeted-block lines, biome, light, memory, FPS) | ✅ | Combos: **F3+Q (help), F3+1 (frame graph — engine extension), F3+H (advanced tooltips)** — exactly the honest set; F3+B/G/A/T/P/D/N ❌. |
| Font: hand-built 5×7→8px-case bitmap, shadow, per-glyph advance | ✅ | Clean-room (vanilla's is a custom pixel font + Unifont; File B's "M+" claim wrong). No §-code formatting engine yet 🟡. |
| Audio: 41-event synthesized bank in vanilla `sounds.json` shape, 9 categories + music, spatial pan/attenuation, day/night music pads, cave ambience | 🟡 | vs vanilla's ~600 events & 13 discs — the disclosed clean-room scale. No jukebox, no discs, no note blocks. |
| Resource packs: vanilla-format pack loading (blockstates/models/textures) | ✅ | `pack_format 6`-era JSON pipeline + the builtin pack. |
| Data packs: recipes, loot tables, tags (folder + zip) | ✅ | Verified against a genuine 1.16.5 server jar's pack format. Advancements/structures/functions: detected, honestly reported unsupported. |
| Save format: Anvil `.mca` + NBT + level.dat | ✅ | Reads/writes real vanilla-format saves. |
| Chat, Tab list, multiplayer, skins, servers | ❌ | Entirely absent (S22 honestly "optional" in File B). |

### 2.11 QA & infrastructure — ✅

593 tests (verified green today), 6+ E2E stages (`E2E_MENU`, `E2E_V114/114b/114c/115/116/116b`, `e2e_audit16/16b`), CI on 4 workflows with smoke greps, drift guards on every registry count, wasm32 clean-build gate, browser-verified bundles (113 screenshots), 15 research documents with live wiki captures. Fuzz testing ❌ (File A's one fair QA gap).

---

## Part 3 — Legal & compliance (verified against 2026-current sources)

**Sources fetched live 2026-09-09:** minecraft.net/en-us/usage-guidelines
("Usage Guidelines for Fans and Creators"), the 2023-updated EULA context,
minecraft.wiki. This is engineering research, **not legal advice** — the
README already says consult an IP attorney before commercial release, and
that advice stands.

### 3.1 Where the project stands (all verified at HEAD)

1. **Assets** — zero Mojang files. Every texture, sound, glyph, logo and the
   panorama are synthesized procedurally at boot (`docs/README` "Zero asset
   files"; art modules per bracket). The Usage Guidelines' hardest rule —
   "do not redistribute our games or any alterations of our games or game
   files" — is not even implicated: nothing of theirs is present.
2. **Disclaimer** — the README carries **Mojang's exact requested wording**:
   "NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH
   MOJANG OR MICROSOFT." The live guidelines page asks for precisely this
   string family on "your product, listing, description, website/webpage,
   and all other related materials." The in-game title screen carries the
   short form ("100% CLEAN-ROOM — NOT AN OFFICIAL GAME").
   **Gap found in this audit:** the *game binary itself* shows only the short
   form; if this is ever distributed as a standalone download, the full
   sentence should appear in-game or in the store listing. Low effort, worth
   doing at next release.
3. **Naming** — brand is "VoxelCraft"; "Minecraft" appears only as a
   *secondary, descriptive* term ("Minecraft-1.16.5-style engine"), which is
   exactly the pattern the live guidelines permit ("Kotoba Miners: a
   Minecraft server…" ✅ vs "Minecraft — the ultimate…" ❌). The repo URL and
   project name don't embed the trademark.
4. **`minecraft:` namespace in code/data** — interop convention for
   save/pack compatibility (every world editor and data tool does this);
   the namespace appears in F3 IDs and datapack keys, never as branding.
   Defensible; documented in the README legal notes.
5. **License** — Apache-2.0; no GPL code copied (Sodium/Lithium/Iris used as
   *technique references* only; Cuberite is Apache-2.0 and credited).
6. **Music** — no C418/Lena Raine material anywhere; the two pads are
   synthesized.
7. **End Poem** — correctly absent (no End credits screen). Julian Gough's
   poem is a copyright landmine (its authorship/ownership history is
   genuinely unusual) — never paste it; if End credits are ever added, write
   original text.

### 3.2 Corrections to the source files' legal sections

- File B's L.1 is broadly sound but two rows are overstated: "The name
  'Creeper' is a trademark for that mob shape" (mob *names* as such aren't
  registered trademarks; trade-dress/confusion is the real doctrine to mind —
  keep names distinct anyway, which the engine does) and "EULA forbids…"
  rows phrased as if the EULA binds this project (the EULA is a contract for
  people who *run Minecraft*; a clean-room engine never agreed to it. The
  *Usage Guidelines* are the relevant enforcement posture, and they govern
  use of Mojang's name/brand/assets — which we don't use).
- File B's L.1.13 "gameplay is not patented — safe" — the conclusion is
  reasonable (game *rules* are excluded from copyright under 17 USC §102(b)
  and the idea/expression doctrine; Mojang/Microsoft hold no known
  gameplay patents covering these mechanics) but absolute "safe" language is
  stronger than any non-lawyer should certify. Keep the attorney line.
- File A's §24 checklist is a good compliance framework; its 24.11
  "trademark search for VoxelCraft" remains open — worth a real USPTO/EUIPO
  search before any commercial distribution, since several small projects
  have used similar names over the years.

### 3.3 The 2026-current picture that neither file has

- The **Aug 2023 EULA/guidelines rewrite** (Mojang's own announcement, "Minecraft EULA and Commercial Usage Guidelines Updates") restructured the language, folded the old Commercial Usage Guidelines into the Usage Guidelines, and hardened the "any sharing with the community is a *commercial thing*" framing — i.e., even free fan distributions are evaluated under the commercial-use rules. For this project that changes nothing materially (no Mojang assets/brand are being distributed), but it kills the casual "it's non-commercial so it's fine" argument some clones lean on.
- The **15th-anniversary painting additions (2024)** are why the live wiki says 47 paintings; 1.16.5 is 26. Irrelevant to us (0 implemented) but a good example of why era-pinned checklists must cite *versioned* facts.
- Practical posture unchanged since the files were written: Mojang's
  enforcement pattern targets (a) asset redistribution, (b) brand/confusion
  abuse, (c) servers selling gameplay advantage — not clean-room engines
  (Minetest/Luanti, ClassiCube, Terasology etc. have operated for 15+ years).
  The project's clean-room discipline + original assets + disclaimer is the
  correct posture; keep the C&D-response plan (rename, pull, comply) as
  README insurance.

---

## Part 4 — Coverage scorecard (the honest numbers)

| Subsystem | Vanilla-1.16.5 scale | VoxelCraft | Coverage |
|---|---|---|---|
| Engine/rendering infrastructure | — | complete-for-scope | strongest layer |
| Blocks/items (types) | ~740 blocks + ~1,000 items | 506 flat-registry entries | ~50–55% of the combined space |
| Block *states* (1.16.5 sum) | ~10,000+ state combos | 805 (compact-state design) | deliberate simplification |
| Mobs | ~72 kinds | 52 kinds (billboards) | ~72% kinds, low visual fidelity |
| Biomes | ~61 (61 registry ids) | 25 + End | ~41% |
| Structures | ~19 families | 10 + 2 stand-ins | ~55% |
| Status effects | 32 | 16 | 50% |
| Enchantments | 38 | 38 | 100% |
| Crafting recipes | 379 | 81 rows | ~21% |
| Brewing (effect potions) | ~15 | 6 | 40% |
| Sound events | ~600 | 41 synthesized | ~7% (clean-room scale) |
| Music discs | 13 | 0 | 0% |
| HUD/menus/settings | full set | core set exact-vanilla | strong |
| Redstone components | ~25 | ~18 | ~70% |
| Weather/raids/events | full | 0 | 0% |
| Multiplayer | full | 0 | 0% |
| Save/format/packs | Anvil+NBT+packs | full | 100% of chosen scope |

**Overall: ~35–45% of a 1.16.5 behavioral replica by feature count, with the
engine/format/UI layer near-complete and the content/long-tail layer the
remaining bulk.** File A's "15–20%" was true of its era and undercounts
today; File B's "~70%" counts checkboxes, not features — real parity is
closer to the middle. (A *bit-exact* replica — every state combo, pixel,
sound and tick — remains a 10⁴–10⁵-item effort, as File B's scale section
correctly says.)

---

## Part 5 — Priority backlog (what "next" actually means)

1. **Weather** (rain/thunder/lightning) — ✅ **DONE** (2026-09-09 backlog
   round `62ca070`, verified 2026-09-10: 628/628 lib tests, the 5
   mob-weather tests by name).
2. **The two missing Nether biomes** (Soul Sand Valley, Basalt Deltas) —
   ✅ **DONE** (same round; the five-biome set is complete).
3. **Farming** (hoe→farmland→wheat/carrot/potato/beetroot growth) — ✅
   **DONE** (2026-09-09 farming round inside `b0529ca`; bread/hay-bale/hoe
   crafts included, villager-farmer AI remains part of item 6).
4. **Doors, beds, TNT** — the multi-block entity class; beds unlock sleep,
   spawn points, and the phantom prey loop.
5. **Remaining 16 status effects + splash/lingering potions** — most are
   formula-only once potion apply paths exist.
6. **Village half of 1.14**: pillagers, patrols, raids, bells, wandering
   trader — the biggest single gameplay system still missing (and the one
   the user's original brief called out by name).
7. **Boats/minecarts** — the vehicle physics class.
8. **Entity mesh pipeline** — replaces billboards with cuboid models;
   biggest visual-fidelity jump, biggest renderer lift.
9. **Sound-event scale-up** (discs, note blocks, per-material steps) —
   mechanical work once priorities settle.
10. **Multiplayer** — only if the goal shifts from replica to game.

The repo's own era plan already queued 1.17+; this backlog is the
1.16.5-closure list before that frontier opens.
