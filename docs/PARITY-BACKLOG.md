# VoxelCraft vs. the reference game 1.16.5 — Comprehensive Parity Backlog & Architecture Audit

This document tracks the clean-room architectural comparison, current implementation status, and technical backlog between VoxelCraft-Rust and **vanilla the reference game 1.16.5 (The Nether Update)**.

All mechanics, numbers, and constants documented here are verified against authoritative public documentation (`the reference game.wiki`, vendor specifications like AMD GPUOpen, or technical the reference game community research) or explicitly flagged as `[ESTIMATED / APPROXIMATION]`.

---

## 1. Executive Parity Status Matrix

| Subsystem | Vanilla 1.16.5 Scope | VoxelCraft Status | Backlog Gap & Clean-Room Action Items |
|---|---|---|---|
| **Blocks & Collision Shapes** | 700+ block IDs, multi-part states, non-cube voxel bounds | 506 block entries, 805 states (`vc-blocks`) | Complex shapes (stairs, slabs, fences, walls, lanterns) need sub-block AABB list. Full waterlogging flag. |
| **Villager Life & Jobs** | 15 professions, work stations, sleeping, golem summoning, trading tiers | 15 professions, 5 trading tiers, restock | Daily schedule AI (work, gossip at bell, sleep in bed), field harvesting/replanting, iron golem panic summoning. |
| **Farming Systems** | Wheat, carrots, potatoes, beetroot, melons, pumpkins, sugarcane, sweet berries, cocoa, nether wart | Basic crop growth, berry bushes, sweet berries | Hydration radius (4 blocks), bone meal growth stages, bee pollination acceleration, villager farmer automation. |
| **Technical Mechanics & Exploits** | TNT duping (BUD + coral notifier), carpet duping, sand duping, update suppression | Pure tick simulation | Deliberate parity for community farm designs: piston push limit (12 blocks), slime/honey separation, falling block portal transition timing. |
| **Entity Models & Recoil** | The Pioneer/The Scout humanoid models, quadrupeds, winged, multi-jointed entities | Procedural bounding box & hand renderer | 3D multi-part box models for all mobs, walking leg/arm swing cycle, head pitch/yaw tracking, clean-room damage recoil tilt. |
| **Enchantments & Books** | 38 enchantments, anvil combining, prior work penalty, librarian book trading | 38 enchantments implemented in `vc-gameplay` | Enchanted book item rendering with glint shader, anvil level progression & XP cost curve, librarian trade rolling. |
| **Vehicles (Boats & Minecarts)** | Rowable 2-seat boats, minecarts (hopper, chest, furnace, TNT) on 4 rail types | Basic entity movement | Boat rowing oar animation + splash sounds; minecart momentum physics, curves, sloped rails, and rail clatter audio. |
| **Dimensions & World Gen** | Overworld (79 biomes), Nether (5 biomes + bastions/fortresses), End (islands + cities + ships) | 25 Overworld biomes, Nether & End basic terrain | Nether Bastion Remnant variants (bridge, housing, stables, treasure), Ruined Portals, End gateway beam teleportation, End cities & Elytra ships. |
| **UI, Input & Menus** | Mouse release activation, drag cancellation, integer GUI scaling, FSR presets | UI canvas 960×540, mouse release activation, letterbox color match | 1.16.5 button release activation parity verified; X11/Wayland input deduplicated; pillarbox clear normalized (#EF323D Intro, #38281B Loading); integer GUI scaling. |
| **Weather & Atmosphere** | Rain, thunderstorms, snow, particle splashes, sky darkness, thunder claps | Sky cycle, daylight cycle | Weather state machine (Clear 12k–180k ticks, Rain 12k–24k ticks, Thunder 3.6k–15.6k ticks), rain/snow particle streaks, sky darkening curve. |

---

## 2. Deep-Dive Subsystem Specifications & Backlog

### A. Villager Economy, Schedules & Iron Golem Summoning
*(Source: `the reference game.wiki/w/Villager`, `the reference game.wiki/w/Iron_Golem`)*

1. **Daily Diurnal Schedule (24,000 Ticks / 20 Minutes)**:
   - `0 – 2000 ticks`: Morning gathering and wandering around village boundaries.
   - `2000 – 9000 ticks`: Working at their claimed job site block (Composter, Lectern, Blast Furnace, Smoker, Barrel, Loom, Stonecutter, Smithing Table, Grindstone, Cartography Table, Brewing Stand, Cauldron, Fletching Table). Checks workstation presence every 300 ticks to perform restock (up to 2 restocks/day).
   - `9000 – 11000 ticks`: Gathering around the village Bell to gossip, share gossip tags (`minor_positive`, `major_positive`, `minor_negative`, `major_negative`), and trade food.
   - `12000 – 0 ticks`: Retiring to their claimed Bed to sleep.
2. **Farmer Auto-Harvesting**:
   - Farmer villagers seek mature crops (Wheat stage 7, Carrots stage 7, Potatoes stage 7), harvest them into inventory, and replant seeds in tilled farmland.
3. **Iron Golem Summoning**:
   - **Panic Trigger**: In Java Edition 1.16.5, 3 or more villagers within 10 blocks of each other who have slept within the last 20 minutes (24,000 ticks) and are panicked by a line-of-sight hostile mob within 16 blocks will attempt to summon an Iron Golem.
   - **Exclusion Zone**: Villagers scan every 10 seconds and will not summon if an existing Iron Golem is detected within a 16-block radius ($\pm16$ on X, Y, Z) in the last 30 seconds.
   - **Player Built**: 4 Iron Blocks in a T-shape + 1 Carved Pumpkin / Jack o'Lantern on top.

### B. Technical Mechanics, Glitches & Vanilla Farming Systems
*(Source: Technical the reference game community consensus & `the reference game.wiki`)*

1. **Piston & Slime Block Dynamics**:
   - Piston push limit: exactly 12 blocks.
   - Slime blocks and honey blocks do not stick to each other.
   - Immovable blocks: Obsidian, Crying Obsidian, Bedrock, Extended Pistons.
2. **TNT Duplication (Java 1.13–1.16.5 Standard)**:
   - Technical machines use an unignited TNT block in a BUD-powered configuration attached to a slime block alongside a dead coral fan and a rail.
   - When the sticky piston moves the assembly, the dead coral fan emits a notifier block update immediately before the intact TNT block transitions into the moving piston tile entity state, causing the TNT to ignite a primed entity while the original block is preserved and moved.
3. **Gravity Block / Sand Duplication**:
   - When falling block entities (sand, gravel, concrete powder, anvils) cross an End Portal or Nether Portal chunk border, the dimension teleportation logic clones the falling entity at the destination coordinates while tick boundary conditions delay destruction of the source entity, generating an extra block.
4. **Zero-Tick & Rapid Farming Mechanics**:
   - Piston extension/retraction cycles causing immediate block update notifications forced instant crop growth ticks in pre-1.16 versions; in 1.16, standard piston observers trigger automated harvesting upon maturity.

### C. 3D Entity Models, Recoil & Animations
*(Source: Client damage rendering standards & clean-room behavioral specification)*

1. **Model Hierarchy (Voxel Boxes)**:
   - **Humanoid (The Pioneer & The Scout)**: Head (8×8×8), Torso (8×12×4), Left/Right Arms (4×12×4 or 3×12×4 for The Scout), Left/Right Legs (4×12×4).
   - **Quadrupeds (Cow, Pig, Sheep)**: Horizontal Torso, 4 independent legs, Head, Snout.
   - **Iron Golem**: Heavy Torso (18×12×11), Long swinging arms (6×30×6), Legs, Protruding nose.
2. **Damage Recoil & Procedural Animation**:
   - **Hurt Recoil**: During the 10-tick invulnerability window (`hurt_t > 0`), the entity model or billboard tilts horizontally using a quadratic ease into a sinusoidal oscillation peaking at $\approx 14^\circ$, combined with the red damage color tint (`[Clean-room Behavioral Approximation]`).
   - **Walking Cycle**: Sinusoidal opposing pitch for left/right legs and arms ($\sin(t \cdot \omega)$).
   - **Head Tracking**: Clamped pitch $[-60^\circ, 60^\circ]$ and yaw relative to torso $[-75^\circ, 75^\circ]$ towards camera/target.
   - **Death Animation**: $90^\circ$ side roll fall + white smoke puff particles.

### D. Enchantments, Enchanted Books & Anvil System
*(Source: `the reference game.wiki/w/Anvil_mechanics`, `the reference game.wiki/w/Enchanting_mechanics`)*

1. **Enchanted Books**:
   - Books can hold any of the 38 vanilla enchantments.
   - Procedural purple/cyan glint shader pass scrolling over 2D item icon or 3D held mesh.
2. **Anvil Combining Rules**:
   - Prior Work Penalty: exponential penalty cost $2^n - 1$ levels added to each subsequent repair or combination.
   - Renaming cost: +1 level.
   - "Too Expensive!" hard limit at 40 levels in Survival mode.
3. **Enchanting Table**:
   - Requires 0 to 15 bookshelves placed in a 5×5 perimeter 1 block away.
   - Lapis lazuli cost (1–3) + XP levels deducted (1–3), requiring player level 1–30.
   - Runic glyph hover revealing one guaranteed enchantment outcome.

### E. Dimensions: Nether & The End Parity
*(Source: `the reference game.wiki/w/The_Nether`, `the reference game.wiki/w/The_End`)*

1. **The Nether (1.16.5 Parity)**:
   - **Bedrock Ceiling**: Flat bedrock ceiling at Y=127; Lava sea surface at Y=31.
   - **5 Biomes**: Nether Wastes, Crimson Forest, Warped Forest, Soul Sand Valley, Basalt Deltas.
   - **Bastion Remnants**: 4 structural layouts (Bridge, Hoglin Stables, Housing Units, Treasure Room) with Piglin Brutes and gilded blackstone chests.
   - **Nether Fortresses**: Blaze spawners, Wither Skeleton spawning on nether bricks, Nether Wart rooms.
   - **Respawn Anchor**: Charges 1–4 with Glowstone; sets spawn in Nether; explodes with strength 5 in Overworld or End.
   - **Piglin Bartering**: Gold Ingot barter loot table (ender pearls, fire resistance potions, obsidian, soul speed books).
2. **The End (1.16.5 Parity)**:
   - **Central Island**: 10 Obsidian Pillars with Ender Crystals (some protected by iron bars), Bedrock Exit Portal with Dragon Egg podium.
   - **Ender Dragon Boss**: Circling, strafing dragon breath fireballs, perching on bedrock podium, healing tethers to end crystals.
   - **End Gateway Portals**: 20 circular bedrock portals opening around perimeter after dragon defeat, teleporting player to outer islands (1000 blocks distance).
   - **Outer Islands**: Chorus fruit plants, End Cities with branching rooms, Shulkers, and End Ships containing the Elytra in an item frame.

### F. Vehicles: Boats & Minecarts
*(Source: `the reference game.wiki/w/Boat`, `the reference game.wiki/w/Minecart`)*

1. **Boats**:
   - 2-passenger capacity (player can transport villagers, pigs, hostile mobs).
   - Dual-paddle rowing animation synchronized with Left/Right turn keys.
   - Water wake and paddle splash sound effects.
   - Extreme acceleration sliding on Ice, Packed Ice, and Blue Ice.
2. **Minecarts & Rail Network**:
   - Standard Rail, Powered Rail (boost when powered, brake when unpowered), Detector Rail (emits redstone signal when cart is present), Activator Rail (shakes passengers out, primes TNT carts).
   - Dynamic 90-degree track curves and sloped incline ramps.
   - Momentum conservation around loops.
   - Metallic wheel friction hum and rail joint click-clack procedural audio.

### G. FSR 1.0 Presets, GUI Scale & Weather Systems
*(Source: `AMD GPUOpen FidelityFX-FSR 1.0 Specs`, `the reference game.wiki/w/Weather`)*

1. **FSR 1.0 Quality Modes (GPUOpen Canonical Specifications)**:
   - **Ultra Quality**: 1.3× per-dimension scale factor ($\approx 77\%$ linear render scale).
   - **Quality**: 1.5× per-dimension scale factor ($\approx 67\%$ linear render scale).
   - **Balanced**: 1.7× per-dimension scale factor ($\approx 59\%$ linear render scale).
   - **Performance**: 2.0× per-dimension scale factor ($50\%$ linear render scale).
   - Dynamic RCAS sharpening lobe scaling: softer (0.4) on Ultra Quality to crisp (0.8) on Performance mode.
2. **GUI Scale Settings**:
   - Options: `Auto`, `1x`, `2x`, `3x`, `4x`.
   - Integer scaling calculating the maximum pixel multiple that fits window bounds without stretching fonts or blurring borders.
3. **Weather Engine**:
   - **Clear**: 12,000 to 180,000 ticks (0.5 to 7.5 the reference game days).
   - **Rain**: 12,000 to 24,000 ticks (10 to 20 minutes).
   - **Thunderstorm**: 3,600 to 15,600 ticks (3 to 13 minutes), active only during rain.
   - **Transition Intensity**: Smooth linear ramp over 100 ticks (0.01 per tick) for `rain_level` and `thunder_level`.
   - Darkening sky light curve, rain streak particles, splash particles on top block faces, spatial thunder claps.

## 4.2c Structure coverage audit (2026-10-10, Part 4)

Placement provenance per structure (1.16.5 scope). "Documented" =
public data/doc source cited in code; [ESTIMATED] = engine tuning.

| Structure | Status | Placement rule |
|---|---|---|
| Village | Present | Documented (spacing 34 / separation 8 / salt 10387312); margin convention [ESTIMATED] |
| Desert pyramid | Present | 32-chunk regions [ESTIMATED]; salt engine-local |
| Jungle temple | Present | 32-chunk regions [ESTIMATED]; salt engine-local |
| Woodland mansion | Present | 8-chunk regions, 1/5 gate [ESTIMATED] |
| Mineshaft | Present | 0.4%/chunk VERIFIED + bounds/corridors tested |
| Stronghold | Present | 3-ring layout, pinned |
| Dungeon (monster room) | Present | Pre-existing |
| Nether fortress | Present | Pre-existing region roll |
| Nether/overworld fossil | Present | 1/64 per chunk VERIFIED |
| Witch hut | Missing | Needs documented dimensions → 4.2d candidate or Part 6 |
| Igloo | Missing | Same |
| Ocean monument | Missing | Same (needs water-aware emit) |
| Pillager outpost | Missing | Same |
| Bastion remnant | Missing | Same (4 variants) |
| Ruined portal | Missing | Same |
| Shipwreck / ocean ruin / buried treasure | Missing | Same (ocean set) |
| Desert well | Missing | Same (trivial shape, needs docs) |
| End city + ship | Missing | Blocked on End outer islands (4.3/4.4) |

Rule: no missing structure is built from estimated dimensions without
owner approval — shapes need documented sources first (4.0 method).

## 4.4c Carver/decoration honesty report (2026-10-10, Part 4)

| System | Status | Provenance |
|---|---|---|
| Cave worms | Present: 1/7 per chunk, 10–28 steps, width 1–5, lava ≤ y10 | Documented (configured_carver/cave.json, data extraction) |
| Ravines | Present: 1/50 per chunk | [ESTIMATED] (unpublished) |
| Veins | Present, cross-chunk resolved (seam 0.952) | Counts/sizes engine-tuned [ESTIMATED]; salt engine-local |
| Trees | Present: per-biome counts (forest 8, jungle 10, dark 14, plains ~0.5 …) | Prose-cited where noted, else [ESTIMATED] |
| Vegetation (flowers/grass) | Present, per-chunk rolls | [ESTIMATED] |
| Desert well / witch hut | Present, prose-faithful | Materials/placement documented; cells approximate |
| End outer islands + chorus | Present | Ring/gate/heights [ESTIMATED]; cities/ships deferred (need dims) |
| 12 missing structures | Absent | Need documented dims (4.2c audit) |

## 5.1m Pack model/item override audit (2026-10-11, Part 5)

| System | Status | Provenance |
|---|---|---|
| Block models (`models/block/*.json` + parent chains + texture vars) | Present, vanilla semantics | resolve_model: child-overrides-parent, `#var` chains depth-capped, 7+ unit tests |
| Pack blockstate dispatch → mesher | Present end-to-end | compile_block_dispatch → installed ModelSet → mesher by_state reads |
| Item models (`models/item/*.json`) | Partial | `resolve_item_model` entry point tested (chains + display merge + missing→fallback rule); no builtin content and no baker/held callers yet |
| Entity textures (`textures/entity/*.png`) | Absent | No pack path exists; mobs render procedural art (turntable-proven). Pack reskinning of entities is a queued epic |
| Custom player skins (64×64, slim) | Absent | Procedural sprite only; needs loader + layout + rig variants (queued epic) |

Approval ask (PLAN-FINAL 4.4): leave the [ESTIMATED] rows approximate
until owner numeric diffs arrive; tune then, not now.

## 5.1n 2B-remainder status (2026-10-11, Part 5)

| System | Status | Provenance |
|---|---|---|
| Font providers (bitmap/ttf/`legacy_unicode`) | Partial | `vc-pack::font`: bitmap + legacy_unicode parse/rasterize, ASCII-range wiring into `FontSource::Providers` (explicit font.png wins); TTF/space/unihex/reference recorded-unsupported |
| `lang/*.json` | Absent, Part 7 | Zero `lang/` reads; Language screen English-only, honestly labeled |
| Pack sounds end-to-end | Tested (headless) | Registry→pick→resample→wav→spatialize deterministic test (fixed seed, exact asserts); physical playback still needs an audio device |
| AO curve public source | Sourced (algorithm) + [ESTIMATED] (curve) | vertexAO + quad-flip match the public 0fps write-up (verified 2026-10-12); curve [0.42,0.62,0.80,1.0] engine-tuned, cited in-code, owner diffs via colour harness |
| Colour-accuracy harness | Partial (tooling) | `colordiff` bin prints numbers-only cell diffs for the owner to run on private captures; awaiting owner numeric diffs (P1) |
| Engine shadow pass | Present | Sun shadow map + 3x3 PCF, strength 0..1 (0 disables); OFF in Vanilla mode |
| Pack shadow-stage + gbuffers (2B.7) | Epic, arch-deferred | Chain design + honest EngineDependent uniform matrix; the gbuffers→deferred→composite→final restructuring belongs to the vc-iris sister project's activation milestone by documented design (iris.rs), not to this repo |
| OptiFine-style compat matrix (2B.6) | Absent | Plan-only references; no shipped Supported/Partial/Unsupported matrix |
| Per-toggle costs + Low/Med/High/Ultra tiers | Procedure-defined, awaiting owner numbers | No local release builds (thermal) and lavapipe is noise, so costs are measured owner-side on the downloadable build: fixed scene + seed 95, `--gpu-timing`, 5 runs per toggle on/off, median `gpu_ms` delta from bench JSON; tiers sized from those deltas when they arrive (P1: numeric results only) |

## 5.1o OptiFine-style compat matrix (2026-10-11, Part 5)

2B.6 support state, audited against code (not claimed from docs):

| Feature | Status | Provenance |
|---|---|---|
| Connected textures (CTM `.properties`) | Unsupported | Zero handling in tree (repo-wide grep, no hits) |
| Custom item properties | Unsupported | Item models absent (§5.1m) |
| Custom entity models / random entities | Unsupported | Procedural mobs, no pack path (§5.1m) |
| Custom sky | Unsupported | Zero hits in tree |
| Normal/specular maps (`_n`/`_s`, labPBR) | Partial | Channel decode in `pbr.rs`, behind the labPBR toggle (default off) |
| Emissive (`_s` alpha) | Partial | Same labPBR path; pack emissive maps beyond it unverified |
| Custom colours (pack colormaps) | Partial | Pack grass/foliage lookup works; per-biome custom colours unsupported |
| Shader uniform aliases | Partial | Documented subset bridge (`shaders.rs`); full compatibility NOT claimed |
| `shaders.properties` options surface | Present | Parsed (profiles/screens/sliders/unknowns retained) in `shaderpack.rs` |

## 5.1p Pack animation + format-warning status (2026-10-11, Part 5)

| System | Status | Provenance |
|---|---|---|
| Pack `.mcmeta` animation (2B.3) | Present, V1'd (agent) | Pack strips honored (`<path>.json` or `<path>.mcmeta`: frametime + frame order + `interpolate` blending); blend proven in-game — uniform (161,0,0) grass tops = mid-blend frame × red tint (tour 38165153925, pixel-verified) |
| `pack_format` 6 warning | Present | Mismatch warns and loads anyway (§46 resilience, tested: "mismatched format must not reject") |

## 5.1q Built-in shader toggle inventory (2026-10-11, Part 5)

PLAN-FINAL §5 toggle list, audited against code:

| Toggle | Status | Provenance |
|---|---|---|
| Soft sun shadows | Present | Shadow-map + 3×3 PCF, §17 sizes (1024/2048/4096), default OFF (vanilla has no sun pass); fabulous tier |
| Entity ground shadows | Present, V1'd (agent) | Setting-gated `push_mob_shadows` emits ground quads into the billboard stream; 2026-10-11 bug (glass texel → invisible) fixed by resampling to black-stained-glass fill; blob visible under turntable creeper (tour 38165153925) |
| Bloom | Present | Bloom pyramid + composite WGSL, grade-driven |
| POM | Present | Raymarch in `pbr.rs` |
| PBR | Partial | labPBR decode behind toggle, default off |
| SSR / SSAO / volumetric light | Absent | Zero word-level hits in tree |
| Vignette / grain | Present | Grade-driven composite; grain covered by test |
| Grading presets | Present | Table + Shaders-screen row + unit test + in-game V1 (modes 1/2) |
| Tiers fast / fancy / fabulous | Present | `graphics` setting (fabulous = soft shadows + full post) |

## 5.1r Lighting status (2026-10-11, Part 5)

| System | Status | Provenance |
|---|---|---|
| Lightmap texture | Absent by architecture | Per-vertex packed light (`sky_pack`/block-light in vertices); no lightmap texture in vc-render (zero hits); vanilla tint curve unverified |
| Per-face shade | Present, exact | Verified exact (prior session) |
| Smooth AO | Present | Mesher AO pack/unpack + smooth flag (§5.1n) |
| Free-down rule | Deviated, disclosed | `light.rs` docs: engine lights upward too — "not vanilla's free-down rule" |
| Weather darkening | Present + tested | Rain/thunder gray-blend constants + `sky_factor` tests |
| Sky / fog / clouds / moon | Present + V1 | Moon disc V1'd in-game; fog end/start wiki-approximated (disclosed in code) |

## 6.1 Item/block render forms (2026-10-12, Part 6)

| Form | Status | Provenance |
|---|---|---|
| Placed blocks | Present | Mesher path (established) |
| GUI / inventory / hotbar icons | Present | Isometric CPU baker (`item_icon_cache.rs`) |
| Held first-person | Present | `push_held_item`: textured cube or bare arm + swing/equip/bob, last in billboard stream |
| Dropped items | Present | 3D mini-block cuboid + spin/bob (`entities.rs`) |
| `display` transforms | Partial | Parsed + merged child-over-parent into `CompiledModel` (identity defaults, tested); application needs item-model resolution first — the icon baker is block-def driven, not model-driven (re-scoped 2026-10-12) |
| Item-frame display / armor stand | Absent | Item id maps only; no display renderer, no stand entity |
| Custom player skins | Partial (loader→FaceTex table) | Loader/regions/sampler/mapping/blit done; FaceTex per-face tiles + `player_skin_face_tex` (tested); settings + stow hookup + selection + tour queued |

Note: `item_icon_cache.rs` module docs still claim "NO first-person
hand renderer" — stale (predates `push_held_item`); queued for a
one-line doc fix with the next code slice touching that file.

## 6.2 Hi-res texture tiles (2026-10-12, Part 6 — owner-ordered)

Today every pack texture is nearest-downsampled to 16×16 (`TILE_PX`,
512² atlas); mipmaps 0–4 exist but resolution scaling does not. Plan:
parameterized tile size (16→512) + atlas growth + mesher UV math +
mip regen (3–5 slices) + icon baker/GUI cells (1–2) + tour V1 (1) =
~7–12 slices (R3 range). Risks: atlas VRAM/mip cost needs real-GPU
numbers; pack-override edge cases multiply with tile sizes. Legal:
user files stay local-only (L3); nothing bundled.

## 6.3 Wider gaps vs the real game (2026-10-12 — matrix pointer)

Source of truth for breadth is `docs/PARITY-MATRIX-1.16.5.md` (Phase 2);
below is the short queue with freshness corrections — re-verify each
row against code before building, the matrix predates Part 5/6 work:
- Rendering/packs: hi-res tiles (§6.2), CTM/custom-sky (§5.1o
  Unsupported), gbuffers (sister project), SSR/SSAO/volumetric
  (absent), ~70 particle types + sprites, music/discs/jukebox,
  entity-model loading (48 kinds billboards), F3 combos + pie,
  subtitles overlay, chat subsystem, full keybind list, languages,
  container GUIs (smoker/blast/stonecutter/loom/smithing/cartography/
  lectern/dispenser/dropper/shulker), skin-layer meshes.
- Gameplay: crawl/spectator caps, sweep (needs swords),
  rails/minecarts, buttons/tripwire, plates, piston-block entity,
  16 effects, raids, advancements, commands + bridge, scoreboard,
  A* pathfinding, datapack-driven drops + Looting, stonecutter/
  smithing/recipe-book/tipped/fireworks/maps/books.
- World: biomes 28/66, structures (bastion/end-city/igloo/outpost/
  portal-wrecks/shipwrecks/monument/hut + ocean set), blocks 533/846.
- Fresh since the matrix: pack ogg decode, font providers (Partial),
  interpolate, entity shadows, crosshair 15, display parse,
  item-model entry — do NOT rebuild these.
