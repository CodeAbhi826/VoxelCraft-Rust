# The 1.0–1.16.5 Completeness Audit — Research Record (bracket 15/16, 2026-09-08 → 2026-09-09)

Live-verification record for the full-era completeness audit round. The
user-facing mandate: *recheck every version from 1.0 to 1.16.5 and close
every gap, "without even missing a single minute stuff"*. The round ran
in two halves:

- **Half 1 (2026-09-08, recovered session)**: the V15 registry window —
  the cooked-meat smelting class (the standing 1.0-era deferral), the
  kitchen chain, the ghast/cave-spider/silverfish trio, chicken
  egg-laying, the leaping/regeneration potion rows, the purpur/end-rod
  crafts, and the F3/registry plumbing. Raw page captures:
  `voxelcraft/scripts/audit16_page_{Apple,Beetroot_Soup,Bowl,Cave_Spider,
  Egg,Food,Ghast,Mushroom_Stew,Poisonous_Potato,Popped_Chorus_Fruit,
  Potion,Pumpkin_Pie,Rabbit_Stew,Silverfish,Sugar}.json`.
- **Half 2 (2026-09-09, the independent recheck)**: a fresh
  version-by-version sweep over the engine's systems found five more
  food-side gaps and one whole missing mechanic family (the player
  throwables). Fresh captures:
  `audit16_page_{Rotten_Flesh,Spider_Eye,Chorus_Fruit,Golden_Apple,
  Melon_Slice,Melon,Ender_Pearl,Snowball,Java_Edition_1.0}.json`.

Every row below was checked against the live wiki at implementation
time (STRICT PROTOCOL — no value enters the code without a capture).

## Half 1 — the V15 window (ids 479..=504, states 776..=803, tiles 706..=734)

| Claim | Verdict | Source |
|---|---|---|
| Steak/Cooked Porkchop hunger 8; Cooked Chicken/Mutton/Salmon 6; Cooked Cod 5 (the smelting class of the raw meats) | ✅ | w/Food §Food values (the grouped hunger table) |
| All six raw meats smelt to their cooked forms; the smoker cooks the meats at half the furnace time | ✅ | w/Furnace + w/Smoker ("food twice as fast") |
| Apple: "Oak and dark oak leaves have a 0.5% (1/200) chance of dropping an apple when decayed or broken" | ✅ | w/Apple §Obtaining (the leaves' bonus roll) |
| Bowl: "Any Planks" ×3 → 4 bowls | ✅ | w/Bowl §Crafting |
| Mushroom stew = bowl + red + brown mushroom (shapeless); rabbit stew = cooked rabbit + carrot + baked potato + any mushroom + bowl; beetroot soup = 6 beetroot + bowl | ✅ | w/Mushroom_Stew, w/Rabbit_Stew, w/Beetroot_Soup §Crafting |
| Pumpkin pie = pumpkin + sugar + egg (shapeless) | ✅ | w/Pumpkin_Pie §Crafting |
| Sugar from a honey bottle ×3 (the 1.15 recipe; sugar cane absent — no farming system, disclosed) | ✅ | w/Sugar §Crafting |
| "Every adult chicken lays an egg item every 5-10 minutes … the theoretical average would be expected at 1 egg every 7.5 minutes (9000 game ticks)" | ✅ | w/Egg §Chicken — engine form: the stateless 1/9000 per-tick roll |
| Poisonous potato: "restores 2 hunger and 1.2 saturation and has a 60% chance of applying 5 seconds of Poison I" | ✅ | w/Poisonous_Potato infobox |
| Popped chorus fruit from smelting chorus fruit; "used to craft End rods and purpur blocks" | ✅ | w/Popped_Chorus_Fruit lead |
| Purpur block = 4 popped chorus (2x2); end rod = blaze rod + popped chorus pair | ✅ | w/Purpur_Block + w/End_Rod §Crafting |
| Ghast: 10 HP fireball at the 1/second cadence within 64 blocks, "after 3 seconds" of charging per volley cycle; drops "Ghast Tear 0-1 50%" + "Gunpowder 0-2 66.67%" | ✅ | w/Ghast §Behavior + §Drops |
| Cave spider: the venom payload — Poison I 7 s on Normal (1:30 Hard; the 140-tick Normal row) | ✅ | w/Cave_Spider §Attack |
| Silverfish: "Silverfish have no drops other than 5 XP" — spawner state in the 776..=803 window | ✅ | w/Silverfish §Drops |
| Potion of Leaping = awkward + rabbit's foot (3:00 Jump Boost I; glowstone → 1:30 II; redstone-ext absent — no redstone item, disclosed) | ✅ | w/Potion §Brewing |
| Potion of Regeneration = awkward + ghast tear (0:45 Regeneration I; glowstone → 0:22 II) | ✅ | w/Potion §Brewing |
| The bowl is a junk-table fishing catch ("Bowl: obtainable through fishing") | ✅ | w/Bowl §Fishing |
| Campfire cooks the six raw meats (the "food items" class; cooked items never re-cook) | ✅ | w/Campfire §Usage |

## Half 2 — the independent recheck (the sweep-2 rows: ids +505, states +804, tiles +735)

| Claim | Verdict | Source |
|---|---|---|
| Rotten flesh: hunger 4, saturation 0.8, "Hunger (0:30) (80% chance)" | ✅ | w/Rotten_Flesh infobox (live 2026-09-09) — the Phase-2 value (4.0 HP) was the raw-meat default and wrong; now 2.0 HP + the effect |
| Spider eye: hunger 2, saturation 3.2, "Poison (0:05)" always | ✅ | w/Spider_Eye infobox — the item existed (spider drops at 1/3, trades, the fermented craft) but was never edible |
| Chorus fruit: hunger 4, saturation 2.4, edible "even when full" | ✅ | w/Chorus_Fruit infobox + §Eating |
| Chorus teleport: "up to 16 attempts are made to choose a random destination within ±8 on all three axes in the same manner as enderman teleportation, with the exception that the entity may teleport into an area only 2 blocks high … If there are no valid blocks within this range, the teleportation attempt fails and the entity remains in place" | ✅ | w/Chorus_Fruit §Teleportation — engine form: `chorus_destination()` (16 attempts, ±8 all axes, solid floor + 2-air column, the silent failure) |
| Golden apple: hunger 4, saturation 9.6, "Absorption (2:00)" + "Regeneration II (0:05)", always consumable | ✅ | w/Golden_Apple infobox — the item existed (the horse-breeding food) but was never player-edible |
| Melon slice: hunger 2, saturation 1.2 | ✅ | w/Melon_Slice infobox |
| "When broken, a melon drops 3-7 melon slices with equal probability for an overall average of 5 slices per melon" | ✅ | w/Melon_Slice §Block loot |
| Melon crafts: 9 slices → the melon block; 1 slice → melon seeds | ✅ | w/Melon_Slice §Crafting ingredient |
| Ender pearl: "can be thrown by pressing the use button, which consumes the item and teleports the player to where the pearl lands, dealing 5 HP damage"; "cooldown of one second (20 ticks)"; "if the player throws an ender pearl before hitting the ground, the fall damage is negated" | ✅ | w/Ender_Pearl §Usage (live 2026-09-09) |
| Snowball: player-thrown, "do not deal damage except to blazes, but they still knock back any mobs that they hit" (3 HP vs blazes) | ✅ | w/Snowball + the standing w/Snow_Golem row (mob-side already shipped; the PLAYER throw was the gap) |
| Egg throw: "When thrown … an egg has a 1/8 (12.5%) chance of spawning a chick. If this occurs, there is a 1/32 (3.125%) chance of spawning three additional chicks (on average, 1 out of every 256 eggs spawns 4 chicks)" | ✅ | w/Egg §Spawning chickens |
| The 1.0-era attribution of the whole throwable + food class (snowballs/eggs/pearls/melons as the 1.0.0 release feature set) | ✅ | w/Java_Edition_1.0 changelog page (the closing-era cross-check) |

## The sweep method (the "minute stuff" pass)

The independent recheck swept the engine system-by-system against the
1.0.0 → 1.16.5 feature set: the registry greps (every food item on the
Food page vs `is_food`/`food_heal`), the mob-drop table, the eat-path
effect hooks, the use-path item branches (throwables), the block-break
loot chain, the craft matcher, and the brewing ingredient set. Each
candidate gap was then classified:

1. **In-capability and missing** → closed this round (all rows above).
2. **Out of engine capability** → disclosed below.
3. **Already faithful** → no action.

## The disclosed deferral class (out of engine capability, 1.0–1.16.5 era)

- **Bread/wheat** (the 3-wheat craft, farmland wheat crops): no farming
  system (no hoe, no farmland, no crop growth beyond nether
  wart/beetroot-item). WHEAT_SEEDS/MELON_SEEDS/PUMPKIN_SEEDS exist as
  trade/craft items only; the 1-slice → melon-seeds craft shipped, the
  planting did not.
- **Cake** (the 7-slice block-food): its craft needs 3 milk buckets —
  no bucket/fluid-holding items in the engine.
- **Milk bucket** (the effect-clearing drink): the same bucket class.
- **Glistering melon slice** (the healing-brew base): the engine's
  brewing uses the documented mushroom stand-in convention; a craftable
  item with no brewing role would be dead registry weight.
- **Golden apple/golden carrot crafts** (8 gold ingots / 8 gold
  nuggets): no gold ingot item — the disclosed GOLD_ORE stand-in
  convention; both foods are picker-obtainable and now edible.
- **Thrown-projectile gravity**: vanilla gives snowballs/eggs/pearls a
  0.03–0.04 gravity and a ~30 b/s arc; the engine's projectile class
  flies straight at 24 b/s (the standing fireball-class adaptation,
  disclosed in code comments at the throw site).
- **Egg-thrown at entities hatching mid-flight** on age-out (the 60 s
  flight cap): the void case — no hatch, no teleport (documented at the
  landing-queue push site).
- **Enchanting-adjacent** (Feather Falling reducing pearl damage,
  Fortune on melon slices, Looting on the drop rows): no enchantment
  application system.
- The standing per-bracket deferrals (villages/raids, redstone
  circuitry, minecarts/boats, doors, TNT, compass/clock/maps, armor
  wearing, mounts): unchanged from their rounds' README notes.

## Art

`audit16_art.rs` grew the sweep-2 row (`melon_slice_art` — the
green-rind wedge with the red flesh and pale seeds, the file's rows-API
style). The atlas coverage guard extends with TILE_MAX 735; the WGSL
LUT + clamps resynced for STATE_COUNT 805 / BLOCK_COUNT 506 (the drift
guards caught the bump exactly as designed — 11 count assertions
updated, the cumulative-window convention).

## Verification

- 593/593 workspace tests green (587 at round start + 6 new: the
  projectile-landings contract, the egg-hatch statistics, the chick
  maturity, the chorus-destination bounds + failure case, the melon
  crafts, and the sweep-2 food-value rows).
- `e2e_audit16` + `e2e_audit16b` stages ride the shared E2E_V116 gate
  (CI smoke greps both boot lines).
- The wasm bundle rebuilt from the new HEAD (matched pair,
  mtime-identical) and browser-verified: boot to the title screen in
  2.62 s with rendered content, zero page errors
  (`docs/screenshots/audit16-sweep2-title.png`).
