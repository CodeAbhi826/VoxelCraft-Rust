# Round 17 — the hunger-drain gameplay system: reference audit

Date: 2026-09-18 · the standing Round-16-parity-gap deferral ("Hunger-drain
gameplay system — the HUD displays the vanilla 20/20 spawn state; the
drain/regen simulation is a gameplay round of its own"), closed per the
user's "anything left to do then do it".

Source of truth: the reference wiki w/Food, fetched LIVE 2026-09-18
(`api.php?action=parse&page=Food&prop=wikitext` — the full §Exhaustion /
§Effects / §Hunger values / §Saturation / §Food saturation values /
§Starvation / §Sprinting sections captured below as extracts). Every
numeric in the code cites this page.

## 1. The variables (wiki §Hunger variables — exact quotes)

- `foodLevel`: "The player's current hunger level, shown on the hunger
  bar. Its initial value on world creation or respawn is 20."
- `foodSaturationLevel`: "Its maximum value always equals foodLevel's
  value and decreases with foodLevel. Its initial value on world
  creation or respawn is 5."
- `foodTickTimer`: "used when foodLevel either exceeds 17, or is at
  zero. It increases on every tick, and whenever it reaches 80
  (4 seconds), it resets" — the regen/starve cadence counter.
- `foodExhaustionLevel`: the §Exhaustion accumulator.

Engine form: `vc_gameplay::hunger::Hunger { food: i32, saturation: f32,
exhaustion: f32, food_tick_timer: i32 }`; `spawn()` = 20 / 5.0 / 0 / 0.

## 2. Exhaustion drain (wiki §Exhaustion — exact quote)

"Once the exhaustion level reaches 4.0, it resets to 0.0 and reduces the
saturation by 1 if there is any saturation remaining. If the saturation
is 0, it reduces the hunger by 1 instead."

**Disagreement, resolved and documented (the R9 note):** the live text
says "resets to 0.0"; the 1.16.5 Java shape subtracts 4.0 and keeps the
overshoot. The wiki's OWN §Saturation-boost row ("heals 1 HP by
consuming 1.5 saturation") mathematically corroborates subtract-4.0:
6.0 exhaustion per healed HP ⇒ 6.0/4.0 = 1.5 drains of saturation per
HP. A reset-to-zero rule would consume only 1.0 sat/HP and contradict
that row. Implemented: threshold `> 4.0`, subtract 4.0, saturation-first.
Also: no drain "on Peaceful difficulty" (the engine has no difficulty
selector — see §7).

## 3. Regeneration (wiki §Natural regeneration + §Saturation boost)

- Natural regeneration: "If the hunger value is at 18 or above, or the
  saturation value is non-zero, the player's health naturally
  regenerates every 4 seconds (80 ticks). Saturation is used first,
  and then once fully drained, hunger is used instead. When the hunger
  value drops to 17 or below, natural regeneration stops."
- Saturation boost (Java only): "regenerates health when the player's
  hunger bar is full (20). Saturation boost heals 1 HP by consuming
  1.5 saturation, and activates every 0.5 seconds (10 ticks) when at
  full hunger."
- Exhaustion cost: "Natural regeneration (requires 18 or higher...):
  6.0 per 1 HP healed."

Engine form (the 1.16.5 branch shape):
- saturated boost: food == 20 && saturation > 0 && shouldHeal → every
  10 ticks: heal `min(saturation, 6) / 6`, add `min(saturation, 6)`
  exhaustion (steady state sat ≥ 6: 1 HP / 0.5 s at 1.5 sat per HP —
  exactly the wiki row);
- natural regen: food ≥ 18 && shouldHeal → every 80 ticks: heal 1.0 HP,
  add 6.0 exhaustion;
- `shouldHeal` = health < 20 (vanilla `player.shouldHeal()`).

## 4. Starvation (wiki §Starvation — exact quotes)

"If the hunger value reaches 0, the player begins to lose health due to
starvation. Starvation damages the player by 1 HP every 4 seconds
(80 ticks). Starvation damage ignores armor and armor toughness, the
Protection enchantment, and the Resistance effect." Thresholds:
"On Easy difficulty, starvation damage stops when the player's health
is at 10 or below. On Normal difficulty, ... at 1 or below. On Hard
difficulty, starvation damage does not stop at any health threshold."

Engine form: `StarveRule` — Survival/Adventure worlds run Normal-class
rules (damage only while health > 1.0); Hardcore runs Hard-class (never
stops; the modes doc already locks Hardcore to Hard rules). Easy and
Peaceful do not exist in the engine's world model (no difficulty
selector — disclosed). The starve damage path bypasses armor and does
NOT add take-damage exhaustion (the unblockable damage class).

## 5. Sprint gate (wiki §Sprinting — exact quote)

"If the hunger value is at 6 or below, the player loses the ability to
sprint until the hunger value exceeds 7."

**Phrasing note (R9):** the sentence's two halves disagree by one point
("at 6 or below" blocked vs "exceeds 7" to regain would imply 8+). The
1.16.5 Java check is `foodLevel > 6.0F` (sprint requires 7+), which
reconciles both halves if "exceeds 7" is read as "passes the 6-point
gate". Implemented: `food > 6`; mayfly-class flight (the engine's
flying state — Creative/Spectator only) bypasses, matching the Java
`|| mayfly` arm.

## 6. Eating (wiki §Hunger values + §Food saturation values tables)

`food = min(food + nutrition, 20)`; `saturation = min(saturation +
saturation_restored, food)` (the Java `eat()` shape: the wiki's
"Saturation restored" column is exactly nutrition × satModifier × 2, so
adding the column value is identical). The full nutrition/saturation
table for every registered edible (both columns VERIFIED live
2026-09-18):

| item | nutrition | saturation |
|---|---|---|
| RABBIT_STEW | 10 | 12.0 |
| COOKED_PORKCHOP / STEAK | 8 | 12.8 |
| PUMPKIN_PIE | 8 | 4.8 |
| BEETROOT_SOUP / COOKED_CHICKEN / COOKED_MUTTON / COOKED_SALMON / GOLDEN_CARROT / HONEY_BOTTLE / MUSHROOM_STEW | 6 | 7.2 |
| BAKED_POTATO / BREAD / COOKED_COD / COOKED_RABBIT | 5 | 6.0 |
| APPLE / CHORUS_FRUIT / GOLDEN_APPLE / ROTTEN_FLESH | 4 | 2.4 (GOLDEN_APPLE 9.6) |
| CARROT / BEEF / PORKCHOP / RAW_RABBIT | 3 | 3.6 (raw meats 1.8) |
| COOKIE / MELON_SLICE / POISONOUS_POTATO / CHICKEN_RAW / RAW_FISH / MUTTON / RAW_SALMON / SPIDER_EYE / SWEET_BERRIES | 2 | 0.4 (raw meats + melon + berries + potato 1.2) |
| BEETROOT / DRIED_KELP / POTATO / PUFFERFISH / CLOWNFISH | 1 | 0.6 / 0.2 |

**The retired convention:** the pre-hunger `food_heal()` table mapped
hunger points to direct HP at hunger/2 (steak 8 → 4.0 HP instantly).
This round RETIRES that deviation — food feeds the FoodData model and
healing comes only through the natural-regen/saturation-boost ticks.
The raw meats' old 4.0-HP default (hunger 8) was the Phase-2 sandbox
convention; the live table's raw meats are 3 hunger (beef/porkchop/
rabbit) or 2 (chicken/mutton/fish) — the wiki wins, and the
direct-heal tests are replaced by nutrition/saturation pins (same
numbers, one column up: 4.0 HP ↔ 8 hunger only for the cooked meats).

## 7. Exhaustion sources (wiki §Energy-intensive actions — exact table)

| action | exhaustion | engine hook |
|---|---|---|
| Swimming | 0.01 per meter | player update, horizontal distance while in water |
| Breaking a block | 0.005 per block | `finish_break` |
| Sprinting | 0.1 per meter | player update, horizontal distance while sprinting (not in water, not flying) |
| Jumping | 0.05 per jump | the manual-jump + autojump launch sites |
| Attacking an entity | 0.1 per attack landed | the melee sites (`applied > 0`) |
| Taking damage (armor-protected class) | 0.1 per instance | `Player::damage` (applied > 0) |
| Hunger effect | 0.005 per tick, per level | the per-sim-tick effect block |
| Sprint-jumping | 0.2 per jump | the jump site's sprint branch |
| Natural regeneration | 6.0 per 1 HP healed | inside the hunger tick |

The old pasted spec's "Food poisoning (per tick) 0.1" row is
superseded by the live table's 0.005/tick/level — the wiki wins.

## 8. HUD consequences (wiki §Saturation — exact quote)

"when saturation reaches zero, the hunger bar starts to shake or jitter
periodically." — wired: `HudStatus.food_jitter` = saturation ≤ 0.0
(alongside the existing Hunger-effect recolor+jitter from round 9).
The food row itself now displays the LIVE foodLevel (the hardcoded
20/20 spawn display is retired with this round).

## 9. Cadence defect found (fixed this round)

The status-effect tick (`Effects::tick`) ran per FRAME in `update()`,
not per 20 Hz sim tick — at 60 fps effect durations expired ~3x too
fast (a latent defect from the Phase-E2 round; the unit tests call
`tick()` per-tick so they never caught the frame-cadence call). The
same lava-contact damage (`lava_t.is_multiple_of(10)` over a per-frame
counter) fired ~3x too often. Both now hang off the sim's 20 Hz
accumulator (per sim step, exactly once), which is also the cadence the
new hunger tick requires.

## 10. Disclosed deviations (with reasons)

- **No difficulty selector** exists in the engine's world model:
  Survival/Adventure run Normal-class starvation (stop at 1 HP),
  Hardcore runs Hard-class (never stops) per the modes doc's own
  "difficulty locked to Hard" row. Easy/Peaceful rules are unreachable
  until a difficulty setting exists.
- **Eating is instant** (no 1.6-s eat animation, no 3-variant
  `generic.eat` sounds, no burp) — the engine's standing eat model;
  deferred with the eat-duration system.
- **foodLevel/foodSaturationLevel/foodExhaustionLevel do not persist**
  in level.dat this round: the engine's player NBT carries no
  health/xp either — every world entry is a fresh 20 HP, so hunger
  rides the same wiki-verified entry semantics ("initial value on
  world creation or respawn is 20 / 5"). A future persistence round
  can carry the three vanilla keys.
- **Riding a mount** while sprint-gated: the mount's locomotion is the
  mount's (the ride-drive path) — the food gate applies to the
  player's own sprint (the `mayfly`-bypass arm covers creative
  flight), matching where the Java check lives.
