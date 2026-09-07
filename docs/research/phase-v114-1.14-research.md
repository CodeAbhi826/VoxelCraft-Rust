# 1.14 "Village & Pillage" — research record (pre-implementation)

Date: 2026-09-07. Live captures: `voxelcraft/scripts/v114_page_*.json`
(bamboo, campfire, sweet_berry_bush, fox, barrel — minecraft.wiki, extracted
the same session). This is the research half of the round; the implementation
follows the established bracket pattern (registry window → art → gen →
gameplay → tests). Values below are the implementation contract.

## Scope decision (session-disclosed)

The full 1.14 bracket splits the same way the wiki does:

- **Nature half** — bamboo, sweet berry bushes/berries, fox, campfire, barrel,
  smooth stone, blast furnace/smoker, lantern, flowers. *Implementation target
  for the next round* (researched below).
- **Village half** — village rework (schedules/beds/work sites), pillager +
  outposts, ravager, raids (Bad Omen → waves → Hero of the Village), crossbow,
  bell, wandering trader, the crafting-station blocks (loom, stonecutter,
  fletching/cartography/smithing/grindstone). *Deferred until the engine has
  the village/raid scaffolding* — the same deferral class as prior brackets.

## Nature half — verified values

### Bamboo (block + shoot)
Source: `v114_page_bamboo.json` (minecraft.wiki/w/Bamboo).

- "A bamboo shoot is the initial non-solid sapling form of planted bamboo."
- Growth: "Upon receiving a random tick, bamboo has a 1/3 chance of growing;
  at default random tick speed (3), each plant grows on average every 4096
  game ticks (204.8 seconds)." Bone meal: +1–2 blocks. Max height 12–16.
- Light: "The top of a bamboo plant requires a client light level of 9 or
  above" to grow.
- Generation: "found primarily in jungles" (Jungle/Jungle variants; 20% chance
  per chunk on the world border of jungle in vanilla — verify at gen time).
- Pandas: "Bamboo items are eaten by pandas and can be used to speed up the
  growth of baby pandas... breed... when at least 1 bamboo block (not a
  shoot) is within 5 blocks" (panda itself is a 1.14 mob — see deferral
  note; breeding hook lands with pandas).
- Fuel: 0.25 items smelted per bamboo (50 ticks) — **RE-VERIFY against the
  Fuel page at implementation time** (value not quoted in this session's
  capture; engine fuel table lives in furnace.rs, DRIED_KELP_BLOCK = 4000
  ticks is the precedent row).
- Breaking: instant-ish (hardness 0); drops itself (shoot drops the item).

### Sweet berry bush + sweet berries
Source: `v114_page_sweetberrybush.json` (minecraft.wiki/w/Sweet_Berry_Bush).

- "Sweet berry bushes are quick-growing plants that grow sweet berries.
  Players and most mobs are slowed down and take constant damage while moving
  through them. They can be found naturally in taigas and snowy taigas."
  (+ old growth spruce/pine taiga in the current wiki — version-scope to
  1.16.5's biome set at gen time).
- Damage: "Sweet berry bushes deal 1 HP damage every tick (although damage
  immunity reduces this to once every half-second), only if the entity is
  MOVING in the hitbox of the bush. At stage 1 and higher, it causes
  damage. Foxes are immune to both characteristics."
- Slow: entities moving through are slowed (factor — re-verify the exact
  multiplier at implementation; the engine has the web/aura slow precedent
  from Slowness).
- Harvest: "A mature sweet berry bush yields 2–3 sweet berries. On its third
  growth stage, it yields 1–2 sweet berries. Each level of Fortune can
  increase the amount of drops by 1." Right-click harvest (regrows).
- Stages: 0 (sapling) → 1..3 → mature 4-stage growth (blockstate `age 0..3`:
  stage 3 = mature in 1.16.5 — the "third growth stage" = age 2; the
  "mature" = age 3. Map carefully: wiki prose says "third growth stage"
  yields 1–2 and "mature" yields 2–3 — with age 0..3 the third stage is
  age 2, mature is age 3).
- Food: sweet berries — Java: 2 hunger, 0.4 saturation ("0.4" hunger-icon
  bar capture). Also compost + fox breeding food; plant on grass/dirt to
  place a bush.

### Campfire
Source: `v114_page_campfire.json` (minecraft.wiki/w/Campfire).

- "A campfire is a block that can be used to cook food, pacify bees, or act
  as a spread-proof light source, a smoke signal, or a damaging trap."
- Hardness 2, blast resistance 2; "Yes (15) when lit" luminous; transparent;
  waterloggable; any tool (axe fastest).
- Damage: "Campfires deal 1 HP every tick (although damage immunity reduces
  this to once every half-second)." No lasting burning; mobs that die to it
  drop raw (not cooked) food.
- Cooking: "Food items take 30 seconds (600 ticks) to cook, compared to 10
  seconds for furnaces or 5 seconds for smokers" — up to 4 food items at
  once, no fuel needed.
- Breaking: "When mined regularly, a campfire drops 2 charcoal. If mined with
  a tool enchanted with Silk Touch, the campfire instead drops itself."
- Unlit state (extinguished by shovel/water; relit with flint&steel);
  the soul variant is 1.16-only — out of the 1.14 bracket.
- Craft: 3 sticks + 1 coal|charcoal + 3 logs (pattern rows: S-S-S top? —
  exact grid: sticks top, coal middle-center, logs bottom? **re-verify the
  3x3 pattern at implementation time** from the recipe capture).
- Smoke: rises ~10 blocks (24 with a hay bale beneath) — particle emitter
  per lit campfire near the player.

### Barrel
Source: `v114_page_barrel.json` (minecraft.wiki/w/Barrel).

- "A barrel is a solid block used to store items. Unlike a chest, it cannot
  connect to other barrels. It also serves as a fisherman's job site block."
- "Barrels have a container inventory with 27 slots, which is the same as a
  single chest. Unlike chests, the action of opening a barrel is never
  prevented." (No ocelot/cat check — a real simplification win over chests.)
- Breaking: drops contents + itself; hardness 2.5, axe fastest.
- Craft: "Any Planks + Any Wooden Slab" (6 planks + 2 slabs; grid pattern
  re-verify at implementation time).
- Hoppers interact (the engine's hopper path extends with the container).

### Fox
Source: `v114_page_fox.json` (minecraft.wiki/w/Fox).

- "Health points: 10 HP." Behavior: "Passive (wild or trusting)."
- Attack strength: "Easy and Normal: 2 HP, Hard: 3 HP" (the engine's
  difficulty_scale path).
- Hitbox: "Adult: Height 0.7 blocks, Width 0.6 blocks. Baby: Height 0.42,
  Width 0.36." Speed: 0.3.
- Spawning: "Foxes spawn in groups of two to four" in Taiga / Snowy Taiga /
  Old Growth Pine Taiga / Old Growth Spruce Taiga / Grove (version-scope:
  1.14 had taiga+snowy taiga; Grove is 1.17 — use the 1.16.5 set).
- Prey: "Foxes attack chickens, rabbits, cod, salmon and tropical fish, and
  baby turtles while they are on land."
- Sweet berry bush immunity (see above) — the interlock with this round's
  plant feature.
- Breeding food: sweet berries (glow berries are 1.17). Trust mechanics
  (baby foxes trust their breeder) — simplify to the llama-style tamed/trust
  pattern and disclose.
- Natural equipment: carries a spawned item in its mouth (5% emerald etc.)
  — optional hook, disclose if deferred.

## Implementation plan (next round)

1. **Registry V10 window** (blocks 417.., states 676.., tiles 619..):
   BAMBOO + BAMBOO_SHOOT (states: shoot + 0..15 age? — vanilla uses
   `age 0..1` leaf age + stalk property; simplify to shoot + stalk, growth
   via column scan), SWEET_BERRY_BUSH (age 0..3 states), SWEET_BERRIES
   (item), CAMPFIRE (+unlit, LIT blockstate), BARREL, + FOX egg if spawn
   eggs are in-scope for the round (kinds 40..).
2. **Art** (v114_art.rs): bamboo stalk tile (segmented green-yellow), shoot,
   berry bush ×4 stages (green shrub → red berries), campfire (logs +
   glowing coals; unlit variant), barrel (wood front + bands), fox sprite
   (orange + white chest/tail — the 1.13 mob-sprite pattern).
3. **Gen**: bamboo patches in Jungle (vanilla 20% on jungle-edge chunks —
   re-verify); berry bushes scattered in Taiga/Snowy Taiga.
4. **Gameplay**: bush damage/slow (moving entities only, fox immune),
   berry harvest (2–3 mature / 1–2 third-stage), berries food 2/0.4,
   bamboo random-tick growth (1/3 per tick, light 9+, max 12–16) + fuel,
   campfire standing damage + 4-slot 600-tick cooking + 2-charcoal drop +
   smoke particles, barrel 27-slot container on the chest path.
5. **Fox**: MobDef (10 HP, 2/3 dmg, 0.3 speed, 0.7/0.6 box), taiga group
   2–4 spawns, prey AI vs chicken/rabbit/cod/salmon/tropical_fish/baby
   turtle, sweet-berry breeding, bush immunity.
6. **Tests**: growth contract, damage gating (stage 1+, moving-only, fox
   immune), harvest counts, campfire cook timing (600 ticks) + drop,
   barrel slot count + craft, fox stats/spawns/prey.
