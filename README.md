# VoxelCraft

A standalone, high-performance **Minecraft-1.16.5-style voxel game engine** written in **Rust** on top of `wgpu` — the same graphics abstraction that powers Bevy and Veloren. One codebase, two targets:

- **Native** (Windows / Linux / macOS) → Vulkan · DirectX 12 · Metal
- **Browser** (WASM + WebGPU, with automatic WebGL2 fallback) → served as static files, **prebuilt and included** so it can be run instantly

## Play it — one file, zero build (Linux)

Every push (and every release) is compiled by [**linux-game.yml**](.github/workflows/linux-game.yml) into a **single self-contained executable** — the whole engine with the builtin resource pack **embedded in the binary**, so it runs directly with no Rust toolchain, no crates to download, no companion files:

```sh
# grab "voxelcraft-*-linux-x64" from the latest run's artifacts (or a Release)
chmod +x voxelcraft-*-linux-x64
./voxelcraft-*-linux-x64
```

Built on `ubuntu-22.04` (glibc 2.35), so the same file runs on Debian 12+, Ubuntu 22.04+, Fedora 36+, Arch and friends. Audio is compiled in (ALSA) and degrades to silent if no sound device exists; everything else the game needs is inside the one file. Full releases (Windows / macOS / arm64 / browser bundle / per-library sources) are on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page.

## Reusable engine libraries — download them separately (no all-in-one bundle)

The engine is split into **14 independent libraries** (`vc-nbt`, `vc-blocks`, `vc-world`, `vc-mesh`, `vc-render`, …), each one a normal Rust crate with its own README, usage example and test suite. On the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page **every library ships as its own archive** (`vc-nbt-0.3.0-source.tar.gz`, `vc-blocks-0.3.0-source.tar.gz`, …) next to the single-file game and the per-architecture binaries — there is deliberately **no AIO zip**. Grab exactly what you need and drop it into your project as a path dependency.

→ Full index with per-library instructions: **[`voxelcraft/LIBRARIES.md`](voxelcraft/LIBRARIES.md)**

![VoxelCraft title screen](docs/screenshots/v112-release-title.png)

![VoxelCraft first person view](docs/screenshots/v112-release-game-first.png)

![VoxelCraft F3 debug overlay](docs/screenshots/v112-release-game-f3.png)

## Quick start — run it in the browser (dev preview)

No toolchain needed. The repo ships the prebuilt WASM bundle — this is the fastest way to **run and verify** the engine (it is a development preview of the native build, not an end-user product):

```sh
cd voxelcraft
python3 -m http.server 8080
# open http://localhost:8080/play.html  (Chromium 113+ or any WebGPU browser)
```

Or just open `voxelcraft/play.html` through any static file server.

## Quick start — native build

The quickest native path is the single-file build above. Building from source:

```sh
# 1. Install Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable   # (see BUILD.md)
source "$HOME/.cargo/env"

# 2. Run
cd voxelcraft
cargo run --release
```

- Linux audio wants ALSA dev headers: `sudo apt install -y libasound2-dev` (otherwise `cargo run --release --no-default-features` builds without sound)
- macOS / Windows work out of the box (CoreAudio / WASAPI)
- wgpu auto-picks the best GPU backend per platform (Vulkan / DX12 / Metal)

## Why it is fast

The previous prototype of this game was JavaScript/Chromium and suffered heavy frame drops. This engine fixes all of it:

- **Greedy meshing** — up to hundreds of merged quads per chunk; an entire flat ocean is one quad
- **Per-vertex ambient occlusion + smooth skylight** flood-fill (BFS) — the exact "smooth lighting" look, while still allowing greedy merging (equal AO/sky corner tuples)
- **Multi-threaded** chunk generation and meshing on native (Rayon), time-budgeted inline streaming on WASM so the browser stays at 60 fps
- **Frustum culling** + per-chunk `draw_indexed` calls, one texture atlas = one bind group
- **Occlusion-flood cache** — the chunk-graph visibility flood recomputes only when the camera crosses a section or a mesh uploads, not every frame
- **Tile-safe atlas sampling** — half-texel UV inset + analytic gradients (`textureSampleGrad`) so bilinear/mipmap/anisotropic filtering can never bleed neighboring atlas tiles (no texture seams, correct LOD at every block boundary)
- **Copy-on-write chunk edits** — in-flight mesh jobs with old snapshots stay consistent while you build

## Zero asset files

All 16×16 textures and every sound are **synthesized procedurally at startup** (in code). They are in the style of Minecraft 1.16.5 but were made from scratch — no Mojang assets were copied, bit-for-bit, anywhere.

## Controls

| Key                | Action                                      |
|--------------------|---------------------------------------------|
| WASD               | Move                                        |
| Mouse              | Look around (cursor is locked)              |
| Space              | Jump / swim up / fly up                     |
| **Double Space**   | Toggle flying (**Creative mode only** — see game modes) |
| Shift              | Fly down / sneak in water                   |
| Ctrl               | Sprint (FOV widens)                         |
| Left click (hold)  | Break blocks                                |
| Right click (hold) | Place blocks                                |
| Middle click       | Pick block into hotbar                      |
| 1–9 / wheel        | Select hotbar slot                          |
| F3                 | Debug overlay (fps, XYZ, biome, tris...)    |
| H                  | Help screen                                 |
| `[` `]`            | Render distance – / +                       |
| `-` `=`            | Volume – / +                                |
| V                  | Toggle V-Sync                               |
| Esc                | Pause / release mouse                       |

## World

- Procedural terrain: simplex 2D/3D noise, biomes (plains / forest / desert / snow / ocean), caves, trees
- Day/night cycle (10 min), dynamic sun/moon/stars, sunset band, fog
- Water with wave animation, glass, 18 block types
- 16×256×16 chunks streamed around the player

## Roadmap progress

The engine is being completed phase by phase (one commit per phase, values verified against minecraft.wiki; unverifiable values are explicitly marked placeholders):

| Phase | Scope | Status |
|---|---|---|
| 0 | Apache-2.0 license + README | ✅ done |
| 1 | Game modes (Creative/Survival/Hardcore), world creation, death & respawn | ✅ done |
| 2 | Mobs + combat (attack cooldown, armor, crits, light-gated spawning) | ✅ done |
| 3 | Full redstone (repeaters, comparators, pistons, containers) | ✅ done |
| 4 | Enchanting (38 entries) + brewing chain | ✅ done |
| 5 | Villager trading depth (15 professions, 5 tiers) + dungeons with spawners | ✅ done |
| 6 | Rendering optimization (mipmaps, aniso, MSAA, occlusion culling, simulation distance) | ✅ done |
| 7 | GPU compute meshing (WGSL greedy mesher, bit-identical to CPU) | ✅ done |
| 8 | Iris integration interface (shader-pack structure validation + translator seam) | ✅ done |
| 9 | Datapacks (Mojang official format: recipes, loot tables, tags; folder + zip packs) | ✅ done |
| 10 | Content breadth (6 new biomes, mineshafts, ravines, desert pyramids, jungle temples, strongholds) | ✅ done |

**Version-evolution bracket 1/16 (MC 1.0–1.2, "Core World Content"):** per the 1.0 → 1.16.5 version-evolution plan, this repo now implements the first historical bracket — **The End dimension** (entry platform at (100, 0), central end-stone island, 10 obsidian pillars on the 42-radius circle, 10 end crystals — 2 caged, stronghold 12-frame end-portal ring with eye-of-ender activation, dimension travel both ways), the **Ender Dragon fight** (200 HP, player-only damage, crystal healing 1 HP/10 ticks in a 32-block cuboid with 10-HP destruction backlash, power-6 crystal explosions, death timeline — XP at 154 ticks, exit portal + egg at 200, 12,000 first-kill XP), the **Nether Fortress** (432×432 Java regions, nether-brick bridges, up to 2 blaze-spawner platforms, nether-wart gardens), the **Mushroom Fields** biome (mycelium, huge mushrooms with exactly 45 cap blocks, mooshrooms, no hostile spawns), **7 new mobs** (Snow Golem, Magma Cube with size-scaled stats, Blaze, Ocelot, Iron Golem, Zombie Villager with the full cure lifecycle, Mooshroom), the **XP orb system** (vanilla 1/3/7/…/2477 ladder, 7.25-block attraction, 10 orbs/s gate, no merging — a 1.17 mechanic), **spawn eggs**, and the bracket's blocks (mycelium spread/revert, redstone lamp with the 4-game-tick off delay, chiseled stone bricks + sandstone variants, nether-wart crop, end stone) with clean-room art for all of it. Every constant was **live-verified against minecraft.wiki at implementation time** (204 `VERIFIED` citations in code; the round's research record, including disclosed wiki self-contradictions, is `docs/research/phase1-1.0-1.2-research.md`). Verified: **339/339 tests** green, wasm32 clean. Progress log: `docs/WORKLOG.md`.

**Version-evolution bracket 2/16 (MC 1.3–1.4, "Adventure Features"):** the second historical bracket lands the **Wither boss fight** (summon = 4 soul sand in a T + 3 wither-skeleton skulls with the last block a skull; 220-tick invulnerable charge; 300 HP Java row; 1 HP/20-tick passive regen; black skulls every 2 s at 8 HP + Wither II 10 s Normal / 40 s Hard; 40-block aggro, hovers 5 above the target; breaks a 3×4×3 box of blocks on damage; drops 1 nether star 100% + 50 XP), **three new mobs** (Wither Skeleton — 20 HP, stone sword, Wither on hit, 2.5% skull drop, fortress spawner platform; Witch — 26 HP, splash potions, ~0.97% monster-pool share; Bat — 6 HP ambient, light ≤ 3 below sea level in groups of 8), a **timed status-effect system** (Wither/Poison/Regeneration + the beacon stat effects), the **beacon** (pyramid 1–4 levels of 9/34/83/164 mixed mineral blocks, Speed/Haste at 1+, Resistance/Jump Boost at 2+, Strength at 3+, Regeneration or primary II at 4; effects every 4 s for 9+2×level s at 20/30/40/50 blocks; light 15), the **ender chest** (8 obsidian + eye of ender, shared 27 slots across every ender chest, breaks into 8 obsidian), **Adventure mode** (vanilla GameType 2 — no block break/place, interactions open), the **anvil** (3 damage stages, 12% degrade per use, gravity block), **lava as a fluid** (dimension-aware: 3-block spread per 30-tick step in the Overworld/End, 7-block spread per 10-tick step in the Nether; light 15; 4 HP per 10-tick contact damage), **emerald ore** (Mountains-family columns only, single blocks, y 4–31), and the bracket's **foods** (potato/carrot/baked potato/pumpkin pie at the verified hunger values). Structural: the block-state space widened u8 → u16 (the ≤255 window was exhausted; future brackets scale without constraint), and a latent E1 bug was fixed — `World::set_block` stored raw block ids as states, so END_PORTAL placed via the game layer read back as FURNACE. Every constant was **live-verified against minecraft.wiki at implementation time** (~120 `VERIFIED` citations; the round's research record, including the disclosed adaptation set, is `docs/research/phase2-1.3-1.4-research.md`). Verified: **372/372 tests** green, wasm32 clean. Progress log: `docs/WORKLOG.md`.

**Version-evolution bracket 3/16 (MC 1.5–1.6, "Transport & Building"):** the third historical bracket + a full worklog↔evolution audit. **Horses, donkeys, mules**: per-instance stats (health 15–30, speed 0.1125–0.3375 internal ≈ 4.86–14.57 b/s, jump 0.4–1.0 clearing 1.153–5.9197 blocks), the vanilla **temper taming rule** (threshold 0–99 drawn at the first mount, +5 per failed mount), the **saddle requirement for control**, full **riding** (steer at the horse's attribute speed, jump launch solved over the engine integrator, sneak-dismount, ridden mounts' AI suspended), **breeding** (golden apple on two tamed adults; foal stats via the wiki's 5-step bred formula; horse×donkey → mule), and herds (plains 5/46, savanna 1/52, groups 2–6, 20% babies). **The lead**: 10-block stretch max in 1.16.5 (the current wiki's 12 is the 2025 buff — version-scoped, both cited), fence knots, break-and-drop. **Redstone components**: daylight sensor (sky light × day-phase brightness in a vanilla-style `power` blockstate), trapped chest (viewer signal — 1 while its GUI is open), light/heavy weighted pressure plates (entity count / ceil(count/10), max 15), block of redstone (always-on weak 15). **Blocks/items**: block of coal (16000 ticks / 80 items), the quartz family, 16 stained terracotta (Badlands banded strata), 5 carpets, hay bale (−80% fall damage), nether quartz, saddle — 37 new registry rows (BLOCK_COUNT 200, STATE_COUNT 400, WGSL mesh LUT resynced). **Superflat** (a 1.1 audit gap, now closed): classic preset — bedrock + 2 dirt + grass, plains, via a real WORLD TYPE toggle in world-create. The round's design lesson: E3 signal sources encode power in blockstates (the vanilla pattern) after the first-cut direct wire-feed was caught being erased by re-derivation — caught by the new tests before commit. Every constant **live-verified against minecraft.wiki at implementation time** (~35 citations; transcripts in `voxelcraft/scripts/verify_e3_*`). Audit results: every prior worklog claim verified in code (375/375 pre-round, module counts, registries, 575 cumulative citations, zero todo!/unsafe); silent gaps found and fixed (Superflat) or disclosed (activator rail — no rail system; scoreboard — needs commands; name tag — needs anvil rename; wither egg stub). Verified: **396/396 tests** green (+21), wasm32 clean. Progress log: `docs/WORKLOG.md`.

**Version-evolution bracket 8/16 (MC 1.11, "Exploration Update" — recovered from an interrupted session + completed, 2026-09-07):** the illager bracket. **Woodland mansions** generate rarely in dark forests (8×8-chunk regions, 1/5 hash gate; three floors — the top half-size, cobblestone shell + plank floors + full-coverage foundation, cross corridors, south entrance) with **vindicator spawners on the lower two floors and evoker spawners on the two upper floors** (the no-respawn adaptation of vanilla's generation-time spawns) and two loot chests rolling the live-verified **`chests/woodland_mansion` table** (4 pools; the Vex Armor Trim 1.20 / Resin Clump 1.21.4 / name-tag rows version-scoped or palette-absent per the page's own §History). **Four new mobs** (all live-verified w/Llama, w/Vindicator, w/Evoker, w/Vex): the llama (Mountains herds 4–6, 15–30 HP, strength 1–5 with the 32.8/32.8/32.8/0.8/0.8% wild distribution, temper taming, hay-bale breeding, spit retaliation 1 HP E/N / 1.5 H, 1⁄900 regen, caravans — a leashed llama attracts up to 10 followers), the vindicator (24 HP, 13-HP iron axe, 5.612 b/s sprint), the evoker (24 HP caster — armor-ignoring 6-HP fangs + the vex summon ring, 100% totem drop), and the vex (14 HP no-clip flyer, summoned only). **The totem of undying**: held-item revival — 1 HP, effects cleared, Regeneration II 45 s + Absorption II 5 s, with a real **absorption buffer** eating damage before health (Fire Resistance scoped out — a 1.16.2 addition). **Shulker box + shell**: 27 slots, the no-nesting insert rule, the shell+chest column recipe, break-spills (disclosed — no item-NBT). **Spawn eggs** for all the new mobs + the re-added husk/stray eggs, all egg-shaped tiles (the zombie-villager egg — the changelog's 5th new — is the engine's pre-existing E2-era item, disclosed anachronism). **Fuel**: wool 0.5 items + carpet 0.335 items (67 t — the live Carpet-page verdict; the changelog's "0.3" is its rounded form). **Curse of Vanishing** filters death drops. Plus a **latent engine bug fix** the round's tests exposed: `attack_cd`'s i32 `saturating_sub` decrement walked fresh mobs' cooldowns below zero forever, so skeleton arrows, melee swings, and the llama spit could NEVER fire in the live game (masked by tests that called `ai_tick` directly) — now floored at 0, mob combat is live. Every constant live-verified against minecraft.wiki at implementation time (research: `voxelcraft/scripts/v111_*`). Verified: **454/454 tests** green (+17), wasm32 clean. Progress log: `docs/WORKLOG.md`.

**Version-evolution bracket 9/16 (MC 1.12, "World of Color Update" — recovered from an interrupted session + completed, 2026-09-07):** the color bracket. **Blocks — the V8 registry window (ids 291..=360):** **concrete ×16** (hardness 1.8, the changelog's headline palette item), **concrete powder ×16** (hardness 0.5, gravity-affected like sand, and the signature mechanic — solidifies to concrete the moment it touches water, checked before falling), **glazed terracotta ×16** (hardness 1.4, smelt any stained terracotta at 0.1 XP, 4-directional facing with per-rotation art), and the parrot spawn egg. **Two new mobs** (both live-verified w/Parrot + w/Illusioner): the **parrot** (6 HP passive, 5 color variants with the Variant NBT table, jungle spawn weight 40/93 = 43.01%, groups 1–2, drops 1–3 XP, seed taming at the 1/10 roll, sit toggle, 12-block-teleport following — and **a cookie is instant death**, the engine's poison-free form of vanilla's fatal cookie) and the **illusioner** (32 HP hostile caster, no natural spawns and no spawn egg — vanilla raid parity; **Blindness** on the player 20 s + the defensive **Invisibility + 4 false duplicates** refresh). **The Blindness effect** itself is real (id 15: close black fog + sprinting blocked). **Crafting:** the engine's first truly **shapeless 9-slot recipe** — 4 sand + 4 gravel + any dye → 8 concrete powder of that color (position-independent, multiple-dye-rejecting). **Art** is all clean-room (`v112_art.rs`: 16 flat vibrant concretes, 16 grainy powders, 16×4-rotation glazed tops + shared sides, 5 parrot variants) plus an **atlas row-math fix** (32-tile rows — the old `%16/16` indexing would have smeared the new tiles across neighbors). Every constant live-verified against the wiki captures at implementation time (research record: `docs/research/phase-v112-1.12-research.md`, transcripts in `voxelcraft/scripts/v112_page_*`). Verified: **471/471 tests** green (+17), wasm clean. Progress log: `docs/WORKLOG.md`.

**Version-evolution bracket 10/16 (MC 1.13, "Update Aquatic" — recovered from an interrupted session + completed, 2026-09-07):** the ocean bracket. **World — the ocean temperature split** (VERIFIED changelog §World generation): warm/lukewarm/cold/frozen ocean families off the existing climate-noise fields, with sand floors on the warm side and gravel on the cold; the **frozen-ocean ice sheet**; **icebergs** (25%/chunk — simplified pack-ice mounds with blue-ice cores, disclosed); **kelp forests** ("ocean biomes, except warm oceans", 2–4-block columns), **seagrass meadows** (oceans + swamp pools), and **coral reefs** (patch-noise fields in warm oceans mixing coral blocks ×5 colors, coral plants, coral fans, and 1–4-count **sea pickle** clusters at the verified 6/9/12/15 underwater light levels). **Eight new mobs** (all live-verified w/Drowned, w/Phantom, w/Dolphin, w/Cod, w/Salmon, w/Pufferfish, w/Tropical_Fish, w/Turtle): the **drowned** (ocean/river water-column spawner + the **zombie→drowned conversion** — "a zombie's head continuously submerged for 30 seconds"; 6.25% hold a **trident** thrown every 1.5 s at up to 20 blocks for 8 HP, dropped at 8.5% on a player kill), the **phantom** (insomnia spawner above players with ≥ 72000 ticks since last rest, resets on death; the 12-block **orbit-and-swoop** cycle with a dedicated altitude-hold controller), the **dolphin** (pods 1–2, warm-side oceans only, **Dolphin's Grace** banking for sprint-swimmers within 9 blocks), the **cod/salmon/tropical fish/pufferfish** water-ambient pool (biome-family spawn tables, 3 HP, out-of-water flop + suffocation), and the **turtle** (beach nesting — bred females lay eggs on sand, babies mature and drop **scutes**, egg-laying/maturity run environmentally with no player anchor). **Aquatic physics**: swim buoyancy + drag for the aquatic set (fish hover, turtles/dolphins glide), and a **flies() classification** — the phantom/vex/bat/parrot take NO gravity (a latent sag that dragged the phantom's orbit 3 blocks below spec, and broke flight for every flying mob, now fixed). **The conduit**: place it in a 26-water core, feed it a 16–42-block prismarine/sea-lantern frame, and players in water within the 32–96-block ladder get **Conduit Power** (air frozen); a complete 42-block frame **hunts wet hostiles within 8 blocks** at 4 HP/2 s, one target at a time. **Four status effects**: Water Breathing, Slow Falling (−9.8 b/s descent clamp + fall-damage immunity), Conduit Power, Dolphin's Grace (×2 swim multiplier, disclosed approximation). **Brewing**: slow falling (awkward + phantom membrane) and turtle master (awkward + turtle shell → Slowness IV + Resistance III 1:00; glowstone → VI + IV), with real drink-time effect windows. **Crafting/furnace**: turtle shell (5 scutes, helmet shape), dried kelp block ×2-way + 20-item fuel, the conduit ring (8 nautilus shells + heart of the sea), blue ice (9 packed ice), kelp → dried kelp smelting (1-hunger food), sea pickle → lime dye. **Registry**: the V9 window (blocks 361..=416, states 615..=675 — BLOCK_COUNT 417, STATE_COUNT 676, WGSL mesh LUT resynced) with the 8 spawn eggs (kinds 32..=39). Deferral class (disclosed in the WORKLOG): advancements (no engine), commands/Brigadier (no parser), trident enchantments + player weapon damage (no weapon/enchant mechanics — the standing 1.11+ deferral), map markers (no maps), redstone-extended potions (no redstone-dust item), turtle-egg hatching stages + trampling (needs random block ticks), stripped logs / debug stick / buffet world type (palette- and system-absent). Every constant live-verified against minecraft.wiki at implementation time (research transcripts in `voxelcraft/scripts/v113_page_*`, cross-checked against the Fandom captures `v113_page_x_fandom_*`). Verified: **489/489 tests** green (+18), wasm32 clean. Progress log: `docs/WORKLOG.md`.

**Evolution audit-fix round (Phases 1–2 completeness, 2026-09-07):** the user-requested audit of the early brackets (e1 = 1.0–1.2, e2 = 1.3–1.4) against the evolution plan found every worklog claim real in code (426/426 green, zero todo!/unsafe) but four silently-absent items — now fixed or formally deferred. **Fixed this round**: the **jungle wood family** (JUNGLE_LOG/LEAVES/PLANKS, registry V6 window 276..=281 — jungle trees now grow their own species with 5..10-block 1×1 trunks, vines on trunks, and the vanilla jungle bush = "a single jungle log surrounded by oak leaves", all VERIFIED live w/Tree + w/Vines + w/Jungle_Tree), **vine climbing** (the "collisionless ladder": up 2.35 b/s, descent capped 3 b/s, sneak hangs, sprint cancelled, fall distance reset — VERIFIED w/Vines + w/Ladder §Climbing), **ferns** (jungle/taiga flora, VERIFIED w/Fern: "only in jungle, taiga, snowy taiga and old growth taiga"), the **golden carrot** (food 6/14.4 VERIFIED w/Golden_Carrot; equine love-mode/heal feed per w/Horse §Breeding "golden apples **or golden carrots**"), jungle log→planks crafting + 300-tick wood fuel, and 7 new clean-room art tiles. **Formally deferred with reasons** (recorded in the WORKLOG): jungle sapling (no sapling system), carrot-on-a-stick (no pig riding/fishing rod), language support (single-language engine, N/A), glass silk-touch pickup (enchant→drop bridge pending), jungle log X/Z placement states, vine spread, golden-carrot rabbit breeding. One source-disagreement disclosed: the current wiki's "vines absorb all fall damage" is edition-untagged — the Java 1.2-era ladder-climb semantics are implemented instead. Verified: **437/437 tests** green (+11), wasm32 clean. Progress log: `docs/WORKLOG.md`.

Post-Phase 10 maintenance note 4 (verification follow-ups — mechanical parity closed): the two remaining mechanical items from the 7-section verification round are implemented and live-verified. **Sprint-jumping** now carries the vanilla input mechanic (+0.2 blocks/tick toward the facing on sprint jumps, per mcpk.wiki) plus a calibrated excess air drag, and an emergence test pins the sustained average at the wiki-observable **7.127 b/s** (measured 7.129). **Coal** is now a real item (the E2 item-block pattern): 1600 burn ticks / 80 s / 8 items per piece, obtained by smelting coal ore (0.1 XP per — the vanilla recipe), dropped by wither skeletons, bought by villagers, and found in dungeon chests — the old "coal-ore-as-fuel" stopgap (800 ticks) is retired since vanilla coal ore is not a fuel. The options-screen render-distance slider also finishes its E2 fix (full 2–32 range). Verified: **375/375 tests** green, wasm32 clean. Full verdict table + priority list: `docs/VERIFICATION-REPORT.md`.

Post-Phase 10 maintenance note 3 (mechanics + visuals, verdict-gated): the two AI-generated research documents (extended mechanics + UI/visuals) were implemented under the standing `docs/research/research-verdicts.md` gate after a **live re-verification round** against minecraft.wiki (outcomes recorded in that file). Mechanics: the **exact vanilla gravity drag `v1 = (v0 − 0.08) × 0.98`** now integrates on a fixed 20 Hz substep for the player (move-then-gravity tick order; jumps re-align the substep phase and rise the vanilla 1.25 blocks), mobs (fixing a latent 20× unit bug), villagers, and item entities; mob fall damage is distance-based MC-12357 (the old impact-speed path was dead code) with substepped terminal falls that can no longer tunnel floors; swimming uses the verified speeds (sprint-swim 3.918, underwater 1.97, surface 2.20 b/s); **drowning** (air 300 → 2 HP/s at −20, 10 HUD bubbles) and the **villager gossip system** (full verified table: trade +4, attack +25, kill broadcast in a 16-block box, 20-min decay, proximity sharing, reputation = Σ value × multiplier) now drive **reputation-priced trades** (`clamp(base − floor(rep × 0.05), 1, 64)`); villagers are attackable (20 HP, no armor) so the hooks are live. Visuals: the **hopper container screen at the verdict-corrected 176×133** (one row of 5 slots — NOT the research doc's blanket 176×166, which was confirmed wrong), the **oxygen bubble row** above hunger, the **held-item name fade** above the XP bar, and **vanilla-parity F3 lines** (XYZ 3-decimals, in-chunk Block/Chunk, `Facing: south (Towards positive Z) (yaw / pitch)`, `Client Light: L (S sky, B block)` from the real light engine, `Looking at block/fluid` split). Verified: 310/310 tests green, wasm32 clean, live browser E2E (hopper screen + F3 VLM-verified — `docs/screenshots/e2e-hopper-screen.png`, `e2e-f3-lines.png`). Confirmed-wrong rows were skipped, unverified rows without an engine system were NOT stubbed (data recorded in the verdicts doc for future phases).

Phase 10 note: the world now carries **14 biomes** (Ocean, Beach, Plains, Forest, Desert, Snowy, Mountains, Nether Wastes + new: Taiga, Birch Forest, Jungle, Savanna, Swamp, Badlands — vanilla save ids and wiki grass/foliage/water tint colors, live-verified) and **5 new structures**: mineshafts (parlors + corridor networks with support beams), ravines (wiki-verified shape grammar: 85-127 long, under 15 wide, up to 62 deep), desert pyramids (21×21 stepped tiers, terracotta checkerboard floor, hidden pit to a 4-chest treasure room), jungle temples, and strongholds (ring 1 = 3 strongholds at the verified 1280-2816 distance band, library + store room + portal room with the 12-frame end-portal ring). Every structure's chests roll loot through the Phase 9 data-pack pipeline — a chunk's owning structure (dungeon > mineshaft > pyramid > jungle temple > stronghold) picks the vanilla loot table, so data-pack overrides reach every chest in the world.

Post-Phase 10 maintenance note (rendering QA): a full engine diagnosis fixed the reported **texture seams / "textures connect"** artifact and trimmed per-frame CPU cost. Root cause (two bugs compounding in the terrain and water fragment shaders): `fract(uv)` tile repetition fed straight into `textureSample`, so (1) bilinear/mipmap/aniso footprints sampled past atlas-tile boundaries — neighboring tiles' texels bled along every block edge — and (2) `fract`'s derivative discontinuity made the GPU's implicit LOD/aniso gradients explode at every integer UV, selecting the coarsest mip along every seam (grid of dark/blurry lines; aniso streaks across tiles; the water scroll dragged a moving seam). Fix: **half-texel UV inset** (`clamp(fract(uv), 0.03125, 0.96875)` — the vanilla stitched-atlas trick, NEAREST unaffected) + **`textureSampleGrad` with analytic gradients from the pre-fract UV** (derivative of fract is 1 a.e.), guarded by the `terrain_water_seam_guards_present` drift test. Also cached the §26 occlusion flood (recomputed only on camera-section change or mesh upload via a `mesh_rev` revision counter) and removed a dead per-frame `Vec` clone. Verified: 295 tests green (WGSL naga-validated), wasm32 clean, live browser E2E on WebGL2/ANGLE SwiftShader — zero console errors, mipmap+aniso path active, terrain pixel-verified (screenshots in `docs/screenshots/seamfix-e2e-*.png`).

Post-Phase 10 maintenance note 2 (fall-through-world + F3 stats): a second diagnosis pass against the reported "lag when rendering" traced the remaining symptom to the **spawn pipeline racing the mesh backlog on slow machines** — the Loading timeout could enter the game before the spawn chunk was meshed, the mesh-gated spawn snap never ran, and the player free-fell through not-yet-generated chunks into the void (observed live at y = −2312, 0 chunks drawn). Fix: the spawn snap now keys on chunk **data** instead of the GPU mesh, and player physics is **held while the player's own chunk is unloaded** (vanilla semantics: entities in unloaded chunks do not tick — also covers creative flight outrunning the generation frontier). Also fixed the F3 header's max-FPS stat (swapped fold initializers printed i32::MAX), and raised the wasm mesh-job cap 2 → 4/frame (the 6 ms inline budget stays the real frame guard). Regression tests: `physics_freezes_until_own_chunk_exists`, `fps_min_max_orders_the_folds`. Before/after: `docs/screenshots/bugfix-void-fall-before.png` → `bugfix-void-fall-after.png` (player at y = 65.89 on the surface, 38 chunks drawn). A research-verdict gate for the mechanics/UI research documents now lives at `docs/research/research-verdicts.md`, and the session-by-session history is tracked in `docs/WORKLOG.md`.

Phase 9 note: data packs follow **Mojang's official 1.16.5 format** (pack_format 6). Drop a pack — a folder or a `.zip` — into your world's `datapacks/` directory (next to `level.dat`) and the engine loads its `recipes/`, `loot_tables/` and `tags/` on world start: datapack crafting recipes appear in the crafting table, `minecraft:chests/simple_dungeon` overrides change dungeon-chest loot, and item tags drive ingredient matching. Advancements, structures and `.mcfunction` files are detected and reported honestly as not-yet-supported. Every format fact was verified against the genuine vanilla 1.16.5 server jar's own data pack.

Phase 8 note: Iris/GLSL compatibility is a **separate sister project** (`vc-iris`, per the clean-room legal boundary — the LGPL Iris source is never copied, only its published documentation). This repo ships the integration surface: drop an Iris-format pack (a folder with `shaders.properties` + `shaders/*.vsh/fsh`) into `shader-packs/` and the engine boots it through structure validation, reporting the pass chain, render targets and uniforms it found.

## Repository layout

```
voxelcraft/                     Cargo workspace (the engine + the game)
  Cargo.toml                    workspace manifest (shared versions, profile)
  LIBRARIES.md                  index of all 14 libraries + download instructions
  crates/
    vc-nbt/                     NBT codec (read/write, all 13 tag types)
    vc-blocks/                  block registry + BlockState + biome tint
    vc-rng/                     deterministic RNG
    vc-chunk/                   16×256×16 chunks, paletted sections
    vc-pack/                    resource-pack / blockstate+model JSON pipeline
    vc-inventory/               items, stacks, containers
    vc-world/                   world grid + terrain gen + light engine
    vc-mesh/                    greedy mesher (AO/skylight-aware merging)
    vc-particles/               vanilla-style break/hit particles
    vc-gameplay/                crafting, furnaces, brewing, enchanting, villagers
    vc-sim/                     20 Hz simulation: fluids, redstone, entities
    vc-anvil/                   vanilla 1.16.5 save/load (.mca + level.dat)
    vc-render/                  wgpu renderer, FSR 1.0, shader packs, UI
    vc-audio/                   synthesized sound bank + spatial audio
    voxelcraft/                 the APPLICATION (game, vc_bench, wasm entry)
  builtin-pack/                 1.16.5-format resource pack (blockstates/models/PNGs)
  shader-packs/                 demo shader packs (moonlit, warm-evening)
  wasm-out/                     prebuilt wasm-bindgen output (run instantly)
  play.html                     standalone browser loader
  BUILD.md                      full build instructions (native + wasm + all-arch)
docs/                           roadmap analysis, session logs
docs/screenshots/               in-game screenshots
public/                         same wasm build, wired into the Next.js preview wrapper
src/app/page.tsx                Next.js wrapper that serves the game at /
scripts/                        build-all.sh (all-arch one-shot builder)
```

The Next.js app in this repo root is only a thin preview wrapper (it iframes `public/voxelcraft.html`); the game itself is entirely in `voxelcraft/`.

## Rebuilding the browser bundle

```sh
cd voxelcraft                                   # workspace root
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --version 0.2.127 --target web \
  --out-dir ./wasm-out target/wasm32-unknown-unknown/release/voxelcraft.wasm
python3 patch-wasm-glue.py wasm-out/voxelcraft.js
```

See `voxelcraft/BUILD.md` for details.

## Architecture notes

- **Chunk = 16×256×16**, `u8` block ids, stored as `Arc<Chunk>` in a `HashMap`. Player edits use copy-on-write so in-flight mesh jobs with old snapshots stay consistent.
- **Mesh job** snapshots the 3×3 neighborhood into a padded 48×256×48 buffer, computes skylight (column scan + lateral BFS), then greedy-merges per `(block, AO corner tuple, corner sky level, face light)` key.
- **Water** = separate blended pipeline (no face culling, depth-write off, top surface at y+0.875 with a vertex wave).
- **Sky** = fullscreen triangle, inverse view-proj ray, gradient + sun disc + glow + moon + twinkling stars.
- **wgpu backend selection on the web**: WebGPU when `navigator.gpu` exists, otherwise WebGL2 with downlevel limits — verified working in headless Chromium (SwiftShader).

## Disclaimer

Not affiliated with Mojang or Microsoft. "Minecraft" is a trademark of Mojang Synergies AB. This is an independent Rust engine written in the visual style of Minecraft 1.16.5; all textures and sounds are generated procedurally from scratch — none are copied from Minecraft's asset files.

## License

Licensed under the **Apache License 2.0** — see [`LICENSE`](LICENSE) for the full text.

In short: you are free to use, copy, modify, and distribute this project (including commercially), as long as you retain the license notice and state significant changes. Game *mechanics and data* (formulas, timings, recipe/loot schemas, registry names) are not copyrightable and are replicated from published documentation; all *assets* (textures, sounds, UI art) are independently authored and contain no Mojang material.
