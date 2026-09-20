# 1.15 "Buzzy Bees" — research record (pre-implementation)

Date: 2026-09-08. Live captures: `voxelcraft/scripts/v115_page_*.json`
(Bee, Beehive, Bee_nest, Honey_Block, Honey_Bottle, Honeycomb,
Honeycomb_Block — minecraft.wiki, extracted the same session). This is the
research half of the round; the implementation follows the established
bracket pattern (registry window → art → gen → gameplay → tests). Values
below are the implementation contract.

## Scope decision (session-disclosed)

1.15 was a small, thematically-unified update — the bee system plus the
honey family. The whole bracket fits in one round:

- **In**: the bee (mob), bee nest + beehive (honey_level 0–5 blocks),
  honeycomb + honey bottle (items), honey block + honeycomb block,
  shears (a legacy item the engine never had — the stick/charcoal
  precedent: "LEGACY item added in the 1.15 window"), hive/nest world
  generation, pollination, campfire pacification, honey-block movement
  physics, all four crafting recipes.
- **Out (post-1.16.5 content in the current wiki text)**: copper waxing
  (1.17), candles (1.17), sugar-from-honey (1.21.2+), trial-chamber loot
  (1.21), meadow/cherry/mangrove spawn table rows (1.17+/1.19+/1.20+
  biomes), the pet-bee April-Fools variants, dispenser-shears harvesting
  (no dispenser shears interaction yet — disclosed).

## The bee (mob) — verified values

Source: `v115_page_Bee.json` (w/Bee).

- Infobox: **10 HP**, "Neutral", Animal, Arthropod; adult hitbox
  **Height 0.5 / Width 0.55** (baby 0.275 width); **Speed 0.6**.
  (The bee's Java movement attribute is `0.3`; the wiki infobox row is
  0.6 — the engine's `speed_attr` field takes the infobox row and
  converts at ×10.5 b/s like every other mob; both readings are within
  the documented adaptation class.)
- "Bees are the only arthropod mob that can spawn in any difficulty
  including Peaceful." Peaceful: "Bees do not deal any damage in Peaceful
  difficulty and are completely passive."
- Attack strength: "Melee: Easy: 2 HP Normal: 2 HP Hard: 3 HP. Venom:
  Normal: Poison I for 10 sec. Hard: Poison I for 18 sec."
- Sting death: "A bee loses its stinger after a successful attack,
  cannot attack further and does not retreat to its nest (even at
  night), and dies approximately one minute later." → 1200-tick death
  timer, one sting per bee.
- Anger: "All bees nearby are angered when an individual bee is attacked
  (unless the bee attacked is killed in one hit), honey or honeycombs
  are collected (unless a campfire is placed under the nest), or a bee
  nest/beehive is destroyed." "Bees attack and swarm the player as a
  group when angered, and the eyes of angered bees turn red." "Bees
  become neutral if they fail to land a hit on their target until anger
  counter runs out. **Anger duration is randomly selected between 20 and
  39 seconds, inclusive.**" Campfire scope: "unless a campfire is placed
  **within five blocks below** the hive" (also "can still avoid fire
  damage if the campfire is placed 4–5 blocks below the nest").
- Spawning: "Naturally generated bee nests generate with **2-3 bees** in
  them." Tree chance table (JE): Plains/Sunflower Plains **5%**, Flower
  Forest **2%**, Forest/Birch Forest/Old Growth Birch Forest **0.2%**
  (Meadow 100%, Mangrove Swamp 5%, Cherry Grove 5% are post-1.16.5
  biomes — out of scope; our biome set: Plains, SunflowerPlains,
  FlowerForest, Forest, BirchForest).
- Behavior: "Bees do not fly (like ghasts or the ender dragon), but
  instead hover a few blocks above the ground similar to bats and
  parrots." (→ the existing `flies()` classification, like the bat.)
- Pollinating: "Bees leave their nest during the day." "After circling a
  flower for more than **400 game ticks (20 seconds)**, a bee collects
  nectar." "A bee carrying nectar has nectar spots on its abdomen and
  drops nectar particles to fertilize plants below the bee." "To
  pollinate a plant, a bee must be **1 to 2 blocks directly above** the
  plant and must have a valid home hive. A bee can fertilize plants
  **10 times** each time they have nectar. There is an approximately
  **5% chance each tick** to attempt fertilization." Pollinated plants:
  "wheat crops, potato crops, carrot crops [etc.] … advances to another
  growth stage, similar to using bone meal."
- Hive cycle: "Afterward, the bee flies back into its hive/nest to make
  honey. **It takes about 2 minutes** for the bee to do this." "When a
  bee that has nectar enters and then leaves its nest or hive, the
  honey level of the nest/hive is increased by one; there is a **1%
  chance it is increased by two**."
- Housing: "One bee nest/beehive can house **up to 3 bees**." "Bees can
  enter a beehive from any side, but exit only from the front."
  "Bees return to their nest when it rains and during the night … They
  stay in their nest or hive for **at least 2400 game ticks (2 minutes)**
  before coming back out."
- Breeding: "Bees follow players holding any 1- or 2-block tall flowers"
  (usable-items row: flowers). "When two bees breed and produce an
  offspring, **1–7 XP is dropped**." (Engine: flowers are the feed item —
  poppy/dandelion/cornflower/lily of the valley.)
- Drops: none (no item drops listed; "Upon death, adult bees drop 1–3
  XP experience when killed by a player or a tamed wolf").

## Bee nest / beehive — verified values

Source: `v115_page_Bee_nest.json`, `v115_page_Beehive.json`.

- Beehive: hardness **0.6**, blast resistance **0.6**, tool axe,
  flammable 5, transparent **No**, note-block instrument Bass, renewable
  (craftable). "Stackable Yes (64) without bees inside."
- Bee nest: hardness **0.3**, blast resistance **0.3**, flammable 30 —
  otherwise the same rows; only natural generation (not craftable).
- Breaking: "If a bee nest is broken with a tool **not enchanted with
  Silk Touch, it drops nothing** and any bees inside emerge angry."
  "If a Silk Touch tool is used, the bee nest drops itself and any bees
  inside remain inside." Beehive always drops itself (breaking with any
  tool — the standard block rule; bees inside are released angry).
  (Engine has no Silk Touch — the disclosed adaptation: nests drop
  nothing, hives drop themselves, both release bees angry.)
- honey_level blockstate **0..=5**: "Once it has the maximum honey level
  of 5, it changes its appearance to show honey oozing out and, if the
  block below it is not a full solid block, starts dripping honey
  particles. (The dripping honey is decorative; it cannot be
  collected.)"
- Harvesting: "When a bee nest or beehive at honey_level 5 is sheared,
  it **drops 3 honeycombs** and angers any bees inside … **Having a lit
  campfire or soul campfire or lighting a fire underneath** the nest or
  hive prevents the bees from becoming hostile." Glass bottle at level 5
  → honey bottle, "The honey level resets back to 0 when a honeycomb or
  honey bottle is harvested."
- Redstone: "Beehives and bee nests have a **redstone comparator output
  with a strength equal to the honey level**" (max 5).
- Beehive craft: "Any Planks + Honeycomb" — the grid is
  planks×3 / honeycomb×3 / planks×3 (6 planks + 3 honeycomb), matching
  the engine's 3×3 recipe rows.

## Honey bottle — verified values

Source: `v115_page_Honey_Bottle.json`.

- "Drinking one restores **6 hunger and 1.2 hunger saturation** and
  returns a glass bottle. Consuming the item also has the benefit of
  **removing any Poison effect** applied to the player. Unlike drinking
  milk, other applied effects are not removed."
- "Consumption time 40 game ticks (2 seconds)"; "Stackable Yes (16)";
  "obtainable by using a glass bottle on a full beehive or bee nest";
  "can be used to craft honey blocks". (The engine has no eating-speed
  or stack-limit-per-item system — disclosed; stack limit follows the
  engine's 64 default with a disclosed note.)

## Honeycomb — verified values

Source: `v115_page_Honeycomb.json`.

- "When a bee nest or beehive at honey_level 5 is sheared, it drops 3
  honeycombs" (the only 1.16.5-obtainable path).
- Used to craft "beehives, candles, and honeycomb blocks" (candles =
  1.17 — out of scope).

## Honey block — verified values

Source: `v115_page_Honey_Block.json`.

- "A … storage block equivalent to the contents of four honey bottles.
  It is sticky, can prevent jumping, and can be used in conjunction
  with pistons to move blocks and adhered entities." (Piston adhesion:
  the engine has no piston block-moving of entities — the slow/jump/
  fall/slide effects are the in-scope part.)
- Hardness **0**, blast resistance 0, any tool, flammable No, note
  block Harp, "JE: Partial (diffuses sky light)" transparency.
- Slow: "Entities … walking on honey blocks move at **2.508 m/s, about
  a 60% reduction** from the normal walking speed." Jump: "Players, who
  can ordinarily jump about 1 1/4 blocks high, can jump about
  **3/16 blocks high** on honey; this is an **85% reduction**."
- Sliding: "Entities pressed against the sides of a honey block slide
  down at a slow speed and do not take fall damage, similar to going
  down a ladder."
- Falling: "As with hay bales, falling onto a honey block **reduces
  fall damage by 80%**. For example, if a player or mob falls from a
  height that would normally cause 10 HP fall damage, the fall causes
  2 HP damage instead."
- Redstone: "Unlike slime blocks, honey blocks in Java Edition are
  non-conductive." (The engine has no slime/honey conduction either
  way — N/A, noted.)
- Crafting: "Honey Bottle [×4] … Empty bottles remain in the crafting
  grid after crafting the honey block." 4 bottles → 1 block; and honey
  block → 4 honey bottles (the reverse recipe — vanilla both ways).

## Honeycomb block — verified values

Source: `v115_page_Honeycomb_Block.json`.

- "Honeycomb blocks are decorative blocks crafted from honeycombs."
  Grid: 4 honeycomb (2×2).
- Hardness **0.6**, blast resistance 0.6, any tool, transparent No,
  flammable No, note block Harp (JE).

## Shears (LEGACY item added in the 1.15 window)

- Vanilla: crafted from 2 iron ingots (diagonal); the engine's
  iron-material convention = IRON_ORE items (the disclosed blast-
  furnace precedent). Durability 238 — the engine has no tool-durability
  system (disclosed; the stick/charcoal item precedent carries no
  durability either).
- 1.16.5 uses: shearing mooshrooms, harvesting honeycomb from full
  hives, fast-breaking leaves/wools/cobwebs. This round implements the
  honeycomb harvest (the round's need); leaves fast-shear and sheep
  wool are noted as follow-ups (the sheep currently regrow wool on
  their own timer — existing engine behavior).

## Implementation contract summary

| Thing | Value |
|---|---|
| Bee HP / damage / speed | 10 / 2 (E,N) 3 (H) + Poison I 10 s (N) 18 s (H) / 0.6 attr |
| Bee hitbox | 0.5 h × 0.55 w |
| Sting | once, then 1200-tick death timer, no further attacks |
| Anger | swarm on: bee hit (not one-shot), harvest w/o campfire, hive broken; 20–39 s |
| Campfire pacify | lit campfire ≤ 5 blocks below the hive |
| Nest spawn | plains/sunflower 5%, flower forest 2%, forest-family 0.2%; 2–3 bees |
| Pollination | 400-tick flower visit; 10 charges; ~5%/tick; 1–2 blocks above crop; +1 growth stage |
| Hive work | ~2400 ticks inside (2 min); +1 honey level on exit (1%: +2) |
| Hive capacity | 3 bees; enter any side, exit front |
| Honey level | 0..=5; level 5 = dripping art + particles |
| Shears on level 5 | 3 honeycomb, reset to 0, anger (unless campfire below) |
| Bottle on level 5 | 1 honey bottle, reset to 0, anger (unless campfire below) |
| Beehive craft | 6 planks + 3 honeycomb |
| Honey block craft | 4 honey bottles ↔ 4 bottles (both directions) |
| Honeycomb block craft | 4 honeycomb |
| Honey bottle food | 6 hunger, 1.2 sat, clears Poison, returns glass bottle |
| Honey block physics | 60% walk slow, 85% jump reduction, 80% fall-damage cut, side-slide |
| Nest/hive hardness | 0.3 / 0.6 (blast same); axe; flammable 30 / 5 |
| Honeycomb block | hardness 0.6, decorative |
| Comparator | output = honey level (max 5) |
