# Phase 1.13 research record — MC 1.13 "Update Aquatic" bracket (live round, 2026-09-07)

The strict protocol's per-bracket research record: what was checked,
against which live page, and every disagreement/adaptation. Raw page
fetches: `voxelcraft/scripts/v113_page_*.json` (+ `_text.txt`
extracts), fetched live 2026-09-07 pre-implementation — **including
independent Fandom captures** (`v113_page_x_fandom_{113,drowned,
phantom,trident}.*`) per the user's standing multi-source verification
directive; every headline mob/weapon value was cross-checked across
the two wikis.

## Verified live (minecraft.wiki unless noted)

| Value | Page | Note |
|---|---|---|
| 1.13 "Update Aquatic", released 2018-07-18; headline: the ocean overhaul, drowned, turtles, fish, phantoms, dolphins, conduits, data packs | w/Java_Edition_1.13 (changelog capture) + Fandom 1.13 | both sources agree on the bracket's scope |
| Ocean split: warm/lukewarm/cold/frozen ocean biomes added; "minecraft:frozen_ocean ... now generates again" | changelog §World generation | the temp-field selection + `is_ocean()` family gate |
| Kelp: "Generate in ocean biomes, **except warm oceans**"; "Can grow multiple blocks high"; smelts into dried kelp | changelog §Blocks | the exclusion is tested both ways (present cold, absent warm) |
| Seagrass: "Generates in oceans (including underwater caves), rivers, and swamplands" | changelog §Blocks | swamp pools carry their own 20% roll |
| Coral reefs: "composed of coral, coral blocks and coral fans"; 5 colors tube/brain/bubble/fire/horn; warm oceans only | changelog §Blocks + w/Coral_Block | patch-noise fields; dead variants carried as registry rows |
| Sea pickles: "generate in warm oceans, especially around coral reefs"; "Up to 4 of them can be placed on a block"; each adds 3 light underwater; smelts into lime dye | changelog §Blocks + w/Sea_Pickle | 6/9/12/15 light ladder carried in blocks.rs; clusters 1–4 |
| Blue ice: "Generates in icebergs"; "Slippier than ice and packed ice"; crafted from 9 packed ice | changelog §Blocks + w/Blue_Ice | simplified iceberg mounds (disclosed); slipperiness out of scope (no friction ladder on ice) |
| Dried kelp block: "Can be used as a fuel in a furnace. **Smelts 20 items**"; crafted from / back into dried kelp | changelog §Blocks + w/Dried_Kelp_Block | 20 × 200 = 4000 ticks |
| Dried kelp: "restoring 1 ( ) hunger point"; "eaten faster than other food" | changelog §Items | 0.5 HP on the engine's food→HP scale; eating speed has no system (disclosed) |
| Drowned: 20 HP zombie-parity; melee 2.5/3/4.5 E/N/H; **"If a zombie's head ... is continuously submerged for 30 seconds, it begins to convert into a drowned"**; 6.25% trident hold; trident throw "every 1.5 seconds, sending it up to 20 blocks away" | w/Drowned + changelog §Mobs | 600-tick accumulator + 30-tick cooldown + 8-HP projectile; 8.5% trident drop on a player kill (w/Trident) |
| Phantom: "spawn ... above a player whose Time Since Last Rest is 1 hour (72000 ticks)"; undead; 0–1 membrane at 50%; attack 2 E/N (the **1.14-pre3 reduction** the current wiki lists — the 1.13 original was 6, version-scoped, disclosed); "circles ... at a height of approximately 12 blocks" | w/Phantom (both wikis agree on the insomnia gate) | reset on death wired into respawn; 12–20-block spawn band; local pack cap 4; orbit-and-swoop with a dedicated altitude controller |
| Dolphin: 10 HP neutral; pods 1–2 (JE); all oceans **except frozen/cold**; "Players who sprint-swim within a 9 block spherical radius of a dolphin receive a swimming speed boost for 5 seconds, replenished" | w/Dolphin | 1/s refresh cadence; ×2 scalar is the engine's documented approximation (no scalar published) |
| Cod: 3 HP; "cannot survive out of water ... they start suffocating"; schools; drops 1 cod; spawns cold/normal/lukewarm | w/Cod + changelog §Mobs | out-of-water flop + 1 HP/s suffocation |
| Salmon: 3 HP; 3 size variants; frozen/cold/river; drops 1 salmon | w/Salmon | variant byte carried on the registry row |
| Pufferfish: 3 HP neutral; inflate toward players; contact 2 HP + 3 s poison semi / 3 HP + 6 s fully (Java); warm/lukewarm | w/Pufferfish | one-step-per-20-ticks inflation; contact band 1.15 blocks |
| Tropical fish: 3 HP; 2,700 visual variants (2 shapes × 15 base × 6 patterns × 15 pattern colors); lukewarm/warm; drops 1 tropical fish | w/Tropical_Fish | variant math documented on `aquatic()` |
| Turtle: 30 HP passive; "spawn on the sand ... in groups of up to 5"; 5% babies; bred with seagrass; "lay eggs in their home beach"; "0–2 seagrass" on death | w/Turtle + changelog §Mobs | beach nesting + egg queue + scute maturity (w/Scute: "Dropped when baby turtles grow up") |
| Turtle shell: 5 scutes (helmet shape); helmet = 2 armor; brews the Turtle Master from awkward | changelog §Items + w/Turtle_Shell | armor wear deferred with the armor system (disclosed) |
| Potion of the Turtle Master: "Gives **Slowness IV and Resistance III for 1 minute**"; glowstone → "Slowness VI and Resistance IV" | changelog §Items | drink-time windows applied for real; redstone 3:00 extension redstone-item-gated (disclosed) |
| Potion of Slow Falling: "Brewed with phantom membrane"; "Slow Falling status effect for 1:30"; "Prevents all fall damage" | changelog §Items | 1800-tick window; redstone 4:00 extension gated (disclosed) |
| Conduit: 26-water 3×3×3 waterlogged core; frame 16–42 of prismarine/dark prismarine/prismarine bricks/sea lanterns; "effective radius of the conduit is 16 blocks for every seven blocks in the frame ... 48 at 21 blocks, 64 at 28, 80 at 35, and 96 with a complete frame of 42"; "A minimum of 16 blocks are required"; full frame "attacks hostile mob within 8 blocks, **dealing 4 HP magic damage every 2 seconds** if they are in contact with water"; "attack only one mob at a time"; light 15; "A conduit won't be activated if not waterlogged" | w/Conduit (§Usage, all rows) | the range ladder formula reproduces every published data point; hostile hunt = one target, 40-tick cadence, wet-mob gate |
| Conduit Power: "The effect has the same benefits as Water Breathing, Night Vision, and Haste" | w/Conduit + changelog §Gameplay | air-frozen half implemented; NV + Haste halves ride the standing deferrals (no darkness / no mining-time system) |
| Slow Falling physics: "terminal velocity of 9.8 m/s, and is unable to take fall damage" | w/Slow_Falling | −9.8 b/s clamp + fall-damage negation |
| Water Breathing: "the breath meter does not run out" | w/Effect §Water Breathing | the air-drain freeze |
| Trident: melee 9 HP; thrown by drowned; 8.5% drop | changelog §Items + w/Trident + Fandom Trident | melee row deferred with the fists-only `held_attack` (no player-weapon system — standing deferral) |
| Data packs ("Items, blocks and functions can be 'tagged'") | changelog §Gameplay | the engine's Phase 9 data-pack system (Mojang's 1.16.5 format) ALREADY covers this — the 1.13 flagship was forward-ported in Phase 9; noted as satisfied |
| Movement: "When sprinting while in water, the player now swims on the surface. Much faster than walking" | changelog §Gameplay | the engine's verified sprint-swim 3.918 b/s (research-verdicts round) |
| 8 spawn eggs (drowned/phantom/dolphin/cod/salmon/pufferfish/tropical fish/turtle) | changelog §Items | V9 egg window, kinds 32..=39 |

## Disagreements / version traps caught

- **Phantom attack damage**: the current wiki lists 2 E/N / 3 H with
  the note that 1.14-pre3 halved it from the 1.13 original 6. The
  engine implements the CURRENT wiki value (2) — the engine's 1.16.5
  target frame — with the version history cited in code (the same
  pattern as the 1.11 totem's Fire Resistance scoping).
- **The conduit's "16 blocks for every seven blocks in the frame"**:
  the phrasing contradicts the page's own data points (48 at 21 = +5
  blocks for the first step, then +7 each). The formula reproduces
  the DATA POINTS, not the phrasing — noted in `conduit_range`'s doc.
- **The changelog's drowned neutral framing** vs the mob pages'
  "Behavior: Hostile" — treated hostile (zombie parity), disclosed on
  the enum.
- **Fandom cross-check**: the Fandom 1.13/drowned/phantom/trident
  captures agree with minecraft.wiki on every cross-checked value
  (insomnia gate, conversion timer, trident throw cadence/damage,
  membrane drop) — no source disagreements to resolve on the headline
  mechanics.

## Deferred with reasons

- Advancements (three new triggers + the whole tree — no advancement
  engine; the standing 1.12 deferral).
- Commands/Brigadier/data-get (no command parser).
- Trident enchantments: Channeling, Impaling, Loyalty, Riptide (no
  enchant-on-weapon mechanics).
- Player trident throwing + the 9-HP melee row (no player-projectile
  or weapon-damage system — `held_attack` stays fists-only).
- Map markers (no map items). Buckets of fish (no bucket capture).
- Redstone-extended potions (no redstone-dust item; extended ITEM
  rows exist with correct windows).
- Turtle-egg hatch stages + zombie/drowned trampling (needs random
  block ticks — no such scheduler for world blocks yet).
- Coral death when out of water (same tick system).
- Bubble columns (magma/soul sand water pushes — needs waterlogged
  block states).
- Stripped logs, carved pumpkin, debug stick, cave_air/void_air,
  buffet world type, End biome split (palette- or system-absent).
- Squid implementation (pre-1.13 legacy — classification marker only;
  a future backfill bracket could add it with its Beta-1.2 semantics).

## Adaptations (engine-shaped forms of wiki facts)

- The conduit frame: the vanilla ring (three 5×5 open squares per
  axis) is approximated by counting frame-material cells in the
  5×5×5 shell minus the 3×3×3 core, clamped at 42 — a player
  replicating the real ring counts identically; a shell-filler hits
  the clamp.
- Icebergs: simplified pack-ice mounds with blue-ice cores at 25% per
  frozen chunk (the vanilla shape grammar is unpublished).
- Conduit effect refresh: applied continuously while in range+water
  with a 40-tick expiry grace (vanilla re-evaluates continuously).
- Water Breathing's turtle-shell source: the helmet wear is deferred
  with the armor system (the effect + air gate are real — the
  conduit's Conduit Power bundles water breathing for a live source).
- The squid: `aquatic()` gates the 1.13 physics set, so the legacy
  water mob is deliberately NOT in it — the enum marker encodes that
  verdict (its own test asserts it).
