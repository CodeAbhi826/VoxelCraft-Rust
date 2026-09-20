# Phase E2 live-research record — MC 1.3–1.4 bracket (evolution Phase 2)

Round date: 2026-09-06. Every row below was fetched LIVE this round from
minecraft.wiki (page_reader fetches archived under `tool-results/phase2/`).
Nothing is copied from the old AI research dumps; `evolution-research.md`
was used only to decide bracket ORDER (which items belong to 1.3–1.4),
never for values.

## Wither (boss) — w/Wither
- Health: **Java 300** regardless of difficulty (BE 300/450/600 by difficulty —
  Java row implemented).
- Armor 4. Undead Monster (Smite works; Instant Health harms / Instant Damage
  heals). Immune to fire, lava, drowning, freezing, and all other status
  effects.
- Summon: 4 soul sand and/or soul soil in a **T shape** + 3 wither skeleton
  skulls on the three upper blocks; the LAST block placed must be a skull.
  Charge-up: **invulnerable, no movement/attacks, 11 s / 220 game ticks**
  (infobox "11 second spawn delay" §Spawning; prose "12 seconds to fully
  charge" is the boss-bar animation — 220 ticks is the mechanical value; the
  1 s gap between them noted as an intra-page phrasing disagreement, the
  entity-active value is 220 ticks).
- Birth explosion after charge: Java max damage Easy 35.5 / Normal 69 /
  Hard 103.5 (proximity-scaled).
- Attacks: aggro range **40 blocks**; flies ~5 blocks above target; main
  head fires a **black wither skull every 2 s** (0.1% blue instead); side
  heads 2–3 s interval (multi-head AI compressed in engine adaptation).
- Wither skull projectile: **8 HP** damage on Normal + **Wither II** effect,
  10 s (Normal) / 40 s (Hard), 1 HP per second, can kill.
- Passive regeneration: **1 HP per 20 ticks (1 s)**; a direct killing blow
  on a target instantly heals **5 HP**.
- On taking damage: breaks all blocks in a **3×4×3** area around itself,
  including obsidian (wither_immune exceptions: bedrock, end portal stuff,
  reinforced deepslate etc. — engine has bedrock/portal blocks immune).
- Below half health: gains "wither armor" (projectile immunity) — Java
  behavior section.
- Drops: **1 Nether Star, always 100%**; **50 XP** when player/tamed-wolf
  killed (Java). Nether star despawn 10 min (Java).
- Hitbox: 3.5 H × 0.9 W (Java).

## Wither Skeleton — w/Wither_Skeleton
- Health 20. Attack (stone sword): Java Easy 5 / Normal 8 / Hard 12.
- Inflicts **Wither effect for 10 s on ANY difficulty** (1 HP per 2 s — i.e.
  0.5 HP/s; unlike poison, can kill).
- Immune to fire + wither effect; does not burn in sunlight.
- Spawns: **Nether fortresses, light 0–7, groups of 5 (JE)**, spawn weight
  8/28 in the fortress category (28.57%).
- Equipment: stone sword 95% right hand (engine: always armed).
- Drops (Java): Coal 0–1 @ 33.33%; Bone 0–2 @ 66.67%; **Wither Skeleton
  Skull 0–1 @ 2.50%** (+1%/Looting level: 3.5/4.5/5.5%); skull guaranteed
  only via charged-creeper kills (engine has no charged creepers yet —
  noted for a later bracket).
- Hitbox 2.4 H × 0.7 W.

## Witch — w/Witch
- Health **26**. Hitbox 0.6 W. Speed 0.25.
- Attack: **Splash Potion of Harming — damage varies by proximity, max 6**;
  Splash Potion of Poison — up to 1 HP per 1.25 s for max 45 s.
- Drinks potions defensively: Healing, Fire Resistance, Swiftness, Water
  Breathing (8.5% chance to drop the potion being drunk when killed).
- Spawns: Overworld light 0, swamp huts, raids, villagers struck by
  lightning (engine: dark Overworld spawns + swamp-hut-adjacent biome rule).
- Drops (Java, per-item 0–2 range, 18.5% × 3 rolls average 4/7): redstone,
  glowstone, gunpowder, spider eye, sugar, glass bottle, stick (avg row
  0.29–0.57 each); engine implements the per-item 0–2 roll.

## Bat — w/Bat
- Health 6. Passive/ambient (does NOT count toward the passive mob cap).
- Hitbox 0.9 H × 0.5 W.
- Spawns: Overworld, **light ≤ 3**, groups of **8 (JE)**, **below sea level
  (the "any height" change is 1.21.2 24w33a — NOT 1.16.5)**, not directly
  exposed to sky, solid block below (24w33a history confirms the pre-1.21.2
  rule was exactly these constraints).
- Drops: nothing.

## Emerald Ore — w/Emerald_Ore
- Hardness 3, blast resistance 3. Non-renewable. Iron pickaxe or better.
- Generates **only in mountains/windswept-hills family biomes** (1.16.5:
  Mountains/Gravelly/Mountain Edge family); **attempts 100 times per chunk
  in blobs of 0–3 ores**; single-block veins (12w22a history: "blob size
  reduced to 1"). Can be exposed to the sky.
- Drops: 1 emerald (up to 4 with Fortune — engine has no Fortune: 1 flat,
  documented), **3–7 XP** when mined. Deepslate variant is 1.17+ (skipped).

## Ender Chest — w/Ender_Chest
- Hardness **22.5** (pickaxe). Drops **8 obsidian** without Silk Touch,
  itself with Silk Touch (engine adaptation: always 8 obsidian — no Silk
  Touch).
- Craft: **8 obsidian + 1 eye of ender**.
- **27 slots**, contents **per-player and shared across every ender chest
  in every dimension** — the same items appear everywhere for that player;
  other players see their own. No double-chest combining; hoppers don't
  interact.
- Light source **level 7**. JE-only note: is a container block.

## Anvil — w/Anvil
- Hardness 5, blast resistance 1200. Recipe: **3 blocks of iron + 4 iron
  ingots (31 iron total)**.
- **Gravity block** (falls like sand); a falling anvil deals **2 HP per
  block fallen after the first**, **capped at 40 HP**; helmets reduce 25%
  and take 2× durability damage. Falls >600 ticks → drops as item. Landing
  on a non-replaceable-with-solid-top (torch/slab) → breaks as item.
- Damage states: **12% chance per use** to degrade one stage (anvil →
  chipped → damaged → destroyed); average 25 uses.
- Repair: combine two same-type items (keeps enchantments, +some durability)
  OR **1 material = 25% of max durability**. Rename: 1 level + prior-work
  penalty. **Cost cap 39 levels ("Too Expensive!")** — not in Creative.
  Prior-work penalty **doubles** each repair (minimum cost doubles).
- 1.16.5-relevant cap: 39 (pre-1.8 was 40 — history row "1.8: cap 40→39";
  implementing 39).

## Beacon — w/Beacon
- Hardness 3, light **15** (even without beam). Craft: **5 glass + 1 nether
  star + 3 obsidian**.
- Pyramid: 1–4 levels of iron/gold/emerald/diamond/netherite blocks (type
  purely cosmetic; mixing allowed): **9 / 34 / 83 / 164 blocks**
  (3×3; 5×5+3×3; 7×7+5×5+3×3; 9×9+7×7+5×5+3×3).
- Requires unobstructed sky view for the BEAM (engine: beam drawn when sky
  above is clear).
- Primary powers: Speed I, Haste I (level 1+); Resistance I, Jump Boost I
  (level 2+); Strength I (level 3+). Secondary (level 4): Regeneration I
  OR primary at level II.
- Feed: 1 iron/gold/emerald/diamond/netherite ingot-or-gem per power
  selection change.
- Range (Java): **20 / 30 / 40 / 50 blocks** radius (cuboid, down + out;
  up by range + dimension height).
- Duration: applied **every 4 s**, lasts **9 + 2×level s** → **11 / 13 /
  15 / 17 s** (Java table; the Bedrock table's 10/12/14/16 with asymmetric
  radii is NOT used — Java row implemented; prose formula ×2+9 matches the
  Java table exactly).
- Multiple beacons with the same effect do **not** stack the level (the
  known-fabricated "stacking" claim stays banned; vanilla: effects refresh
  independently, highest amplifier wins per refresh).

## Item Frame — w/Item_Frame
- Java: an ENTITY (not block); punchable even in Adventure mode.
- Craft: **8 sticks + 1 leather**.
- Drops itself + contained item; punching pops the item first.
- 8 rotation steps (45° each) via interaction.
- Glow Item Frame is 1.17+ — skipped.

## Flower Pot — w/Flower_Pot
- Hardness 0, instant break, drops itself + the plant separately.
- Craft: **3 bricks** (brick ITEM in vanilla; engine adaptation: 3 brick
  BLOCKS — engine has no brick item, documented).
- Can contain flowers/saplings/mushrooms/cacti/fungi.

## Wall (cobblestone) — w/Wall
- Craft: **6 matching blocks → 6 walls** (cobblestone for the 1.3 bracket).
- Decorative boundary block like fences: **1.5-block-tall collision**
  (players/most mobs cannot jump over); connects to adjacent walls/fences/
  solid blocks (engine: fence-style connection rules at mesh time).

## Tripwire Hook — w/Tripwire_Hook
- Craft: **1 iron ingot + 1 stick + 2 planks → 2 hooks** (engine
  adaptation: iron ore + planks — no stick/ingot items, documented).
- A valid circuit: **two hooks + a straight horizontal line of 1–40
  string**; both hooks emit redstone power while the line is tripped by
  a mob/item/player. Destroys itself if lava flows in (JE).

## Food values — w/Food (hunger / saturation, per item)
- Potato: **1 / 0.6**. Baked Potato: **5 / 6.0**. Carrot: **3 / 3.6**.
- Pumpkin Pie: **8 / 4.8**. (Poisonous Potato 2/1.2 — 60% poison; engine
  poison-effect path exists via effects system added this bracket, but
  poisonous potato item deferred — no potato-farming system to distinguish
  it from normal drops.)
- Baked potato via smelting (furnace map: POTATO → BAKED_POTATO).
- Pumpkin Pie recipe (vanilla): pumpkin + sugar + egg — engine has NO
  sugar/egg items → **picker-only, recipe deferred with disclosure**.

## Adventure Mode — w/Adventure
- Cannot break blocks (Java: only with can_break component — engine has no
  item components → plain no-break), cannot place blocks (same), CAN still:
  interact with mobs/entities, use levers/buttons/doors/containers, craft,
  fight, take damage. Hunger advances; world-create mode option.

## Command Block — w/Command_Block
- Not craftable, not in the survival pick list; /give or creative pick.
- Impulse variant only in 1.3–1.4 (chain/repeating are 1.9 — deferred).
- Executes a chat command on redstone pulse. Engine adaptation: executes
  the engine's E2E command bridge (the engine's command set), triggered by
  redstone power — full player chat-command system documented as deferred.

## Mob heads — w/Skeleton_Skull (+Wither page)
- Wither skeleton skull: the summon component (drops covered above).
- Skeleton/zombie/creeper/player heads drop only from **charged-creeper**
  kills (skeleton skull hardness 1) — engine has no charged creepers yet:
  deferred to the lightning bracket; wither skeleton skull IS implemented
  (drop + summon use).
- Note [2]: BE all mobs drop heads, JE one random — irrelevant while
  deferred.

## Book and Quill — w/Book_and_Quill
- Craft: book + ink sac + feather; up to 100 pages; not stackable.
- Engine has NO book/paper/ink items and NO text-editor GUI → **deferred
  entirely this bracket** (registered as a known deferral, not a half-item;
  re-evaluate with the writing/farming brackets).

## Bracket-scoped mechanical fixes (from VERIFICATION-REPORT.md)
- Render-distance slider range **2–32** (engine had 2–16).
- **Lava fluid sim**: Overworld 1 spread / 30 ticks, max 3; Nether 1 / 10,
  max 7 (values previously live-verified w/Lava, recorded in
  research-verdicts.md; re-cited here).
- Sprint-jump **7.127 b/s** emergence regression test.

## Intra-page / cross-source disagreements noted this round
1. Wither charge time: infobox "11 s spawn delay" vs prose "12 s to fully
   charge the boss bar". The mechanical entity-active window is 220 ticks
   (11 s) per the Spawning section; the boss-bar fill animation reaches
   full at 12 s in the prose description. Engine implements 220 ticks;
   disclosed.
2. Beacon duration: prose formula (9 + 2×level) and the Java table agree
   (11/13/15/17); the Bedrock table (10/12/14/16, asymmetric radius) is a
   Bedrock-only split — Java row implemented, no unresolved conflict.
3. Witch drop table: "Redstone Dust 6.00" parsing artifact in extraction —
   the per-item expectation is 0.29–0.57 (3 rolls of 1–3 range at 18.5% /
   Looting-scaled); engine implements the per-item 0–2 roll semantics.

## Engine adaptations (disclosed in code comments + worklog)
- No iron ingot / stick / brick-item / sugar / egg / paper / ink-sac items
  → anvil craft 3 iron blocks + 4 iron ore; tripwire hook craft iron ore +
  planks; flower pot craft 3 brick blocks; pumpkin pie + book-and-quill
  deferred (no ingredient items); beacon feed accepts iron/gold/emerald/
  diamond ore-or-block forms (engine's ore-as-material convention).
- Charged creeper absent → mob heads (skeleton/zombie/creeper/player)
  deferred; wither skeleton skull implemented via its 2.5% drop.
- Command block executes the engine's E2E command bridge (no player chat
  command system yet).
- Multi-head wither AI compressed to a single-head model with the main
  head's 2 s cadence + side-head volley adaptation; dragon-style billboard
  rendering for the boss.
