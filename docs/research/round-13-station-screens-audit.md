# Round 13 — Station GUIs research audit (anvil / beacon / grindstone)

All facts live-fetched 2026-09-15 (UTC) from minecraft.wiki via the
page_reader pipeline. Sources: w/Anvil, w/Anvil_mechanics, w/Beacon,
w/Grindstone, w/Enchanting, w/Armor. GUI geometry cross-derived from
the wiki's own GUI screenshots (Beacon_GUI.png, Grindstone_GUI.png,
Anvil_GUI.png — public CC BY-NC-SA wiki images, used as reference
only; every engine asset remains procedural clean-room).

## 1. Anvil (w/Anvil + w/Anvil_mechanics, fetched 2026-09-15)

### GUI geometry (spec + Anvil_GUI.png pixel analysis)
- Panel 176x166 (the standard container family).
- Slot layout (panel-local, 18x18 each): left input (27, 47), right
  input (76, 47), result (134, 47). [spec values; screenshot pixel
  analysis measured (28,46)/(78,46)/(138,46) at 2px sampling = same.]
- Merge "+" glyph between the inputs at ~(61, 54); progress arrow at
  ~(106..128, 54) pointing at the result.
- Cost text: "Enchantment Cost: <n>" drawn at panel-local (60, 70),
  green when affordable, red when not ("Insufficient XP: result slot
  dimmed + cost text red").
- Rename text field: 104x12 vanilla at panel-local ~(64, 20), sits in
  the panel's top strip above the slots (screenshot: field spans
  x 64..168, y 20..32). Title text reads "Repair & Name" (VLM OCR of
  the wiki screenshot); the engine draws its own "REPAIR & NAME".
- Hammer illustration top-left of the field (clean-room redraw).

### Mechanics (w/Anvil_mechanics — the exact formulas)
- COMBINE REPAIR: "If the target item is damaged it is repaired,
  adding the durability of the sacrifice item plus a bonus of 12% of
  the maximum durability. This repairs up to the item's maximum
  durability. The cost for this repair is 2 levels."
- ENCHANT TRANSFER per sacrifice enchant:
  * sacrifice level > target level -> target raised to match;
  * equal AND not already at max -> target gains one level;
  * lower -> unaffected.
- INCOMPATIBLE: "+1 level for every incompatible enchantment on the
  target (in Java Edition)" — the sacrifice's incompatible enchant is
  NOT transferred (the pre-1.12.2 hard refusal is historical; the
  1.16.5 behavior allows the combine and drops the enchant).
- COST = priorWorkPenalty(target) + priorWorkPenalty(sacrifice)
  + [1 if renaming] + [2 if target damaged] + enchantCost.
- ENCHANT COST (Java): for each sacrifice enchant that can apply:
  final level on the result x multiplier ("from item" column; "from
  book" = half). Multipliers (w/Anvil_mechanics table): Protection 1,
  Fire Protection 2, Feather Falling 2, Blast Protection 4,
  Projectile Protection 2, Thorns 8, Respiration 4, Depth Strider 4,
  Aqua Affinity 4, Sharpness 1, Smite 2, Bane of Arthropods 2,
  Knockback 2, Fire Aspect 4, Looting 4, Efficiency 1, Silk Touch 8,
  Unbreaking 2, Fortune 4, Power 1, Punch 4, Flame 4, Infinity 8,
  Luck of the Sea 4, Lure 4, Frost Walker 4, Mending 4, Curse of
  Binding 8, Curse of Vanishing 8, Impaling 4, Riptide 4, Loyalty 1,
  Channeling 8, Multishot 4, Piercing 1, Quick Charge 2, Soul Speed 8,
  Sweeping Edge 4.
- PRIOR WORK PENALTY: "The cost increases exponentially by the formula
  2^c-1 where c is the anvil use count" — table 0/1/3/7/15/31 for
  use counts 0..5. New use count after an operation = max of both
  items' use counts + 1. Renaming alone does NOT accumulate.
  **DISAGREEMENT (documented)**: the engine's old deferred-rules
  record `prior_work_penalty(repairs) = 1 << repairs` (1/2/4/8) does
  NOT match the live wiki (1/3/7/15). Wiki wins; the constant record
  and its test are corrected this round.
- RENAME: "Any item or stack of items can be renamed at a cost of one
  level plus any prior-work penalty"; max 50 chars (Java); blank or
  unchanged name => red X refusal; "Renamed items are italicized by
  default"; "Named items do not stack with unnamed or differently-
  named items of the same type".
- REFUSALS: total cost > 39 => "Too Expensive!" (creative exempt);
  target at full durability AND sacrifice has no enchantments AND no
  valid rename => red X.
- CAP: "The anvil has a limit of 39 levels; beyond that, repairs are
  refused." (COST_CAP = 39, already recorded.)

### Armor durability (w/Armor live tables, fetched 2026-09-15)
| piece | leather | golden | iron | diamond |
|---|---|---|---|---|
| helmet | 55 | 77 | 165 | 363 |
| chestplate | 80 | 112 | 240 | 528 |
| leggings | 75 | 105 | 225 | 495 |
| boots | 65 | 91 | 195 | 429 |

**DISAGREEMENT (documented)**: the historical values widely quoted for
iron/diamond boots are 209/461; the LIVE wiki tables read 195/429 —
and the live page's own rule ("boots a multiple of 13": 195=13x15,
429=13x33) confirms 195/429 while 209/461 are not multiples of 13.
Wiki wins: 195/429.

### Anvil damage stage (already in anvil.rs — re-verified)
- 12% chance per use to advance one stage; 3 stages; wired into the
  take-result path this round.

## 2. Beacon (w/Beacon, fetched 2026-09-15)

- Powers: Speed I / Haste I at level 1+; Resistance I / Jump Boost I
  at 2+; Strength I at 3+; secondary (Regeneration I or primary II)
  only at level 4. [matches the existing beacon.rs constants]
- Payment: "the player places the item to be fed in the empty slot and
  clicks an effect ... The user clicks the 'Done' button (green
  checkmark), the item is consumed". The engine has no ingot items;
  the standing documented adaptation (beacon.rs header) is that ORE
  items stand in for ingots. **Round-13 spec adaptation (documented)**:
  clicking a power may also pay "from the pyramid" — consume one base
  block from the pyramid itself when the payment slot is empty. The
  engine implements BOTH: slot payment (ore stand-in) preferred, else
  the pyramid-mineral payment.
- Pyramid break: "If the pyramid is broken, effects deactivate or
  weaken depending on the level ... Upon restoration of the pyramid,
  the originally selected power returns without the need to spend
  another item."
- GUI (Beacon_GUI.png analysis): dark upper section; "Primary Power"
  label left, "Secondary Power" right; 5 primary power buttons in a
  2x2 grid + a 5th below-left (bottom-right cell empty); secondary =
  Regeneration heart button + the primary-II slot; 4 level glyphs —
  three stacked down the left edge (~x15, y 24/43/62) + one at the
  right (~x121..135, y 24..31), lit green per achieved level; payment
  hint icons + payment slot (~x88..106, y 82..100) + green checkmark
  (~x106..124) + red X (~x124..142) in the bottom bar. All engine
  pixels are clean-room redraws of this structure (disclosed as
  approximate; the wiki publishes no coordinate table).

## 3. Grindstone (w/Grindstone, fetched 2026-09-15)

- GUI title (VLM OCR of Grindstone_GUI.png): "Repair & Disenchant".
- Two input slots STACKED at panel-local (50, 18) and (50, 40) (22px
  pitch), the grindstone-wheel illustration between them, the result
  slot at (148, 32), an arrow at ~(90..110, 36..46).
- "When one enchanted item is placed in either input slot, a
  disenchanted copy of the item appears in the output slot. The output
  item has the same durability ... If an enchanted book is placed in
  the input, a normal book appears in the output. Removing the item
  from the output slot deletes the input item and causes the
  grindstone to drop some experience."
- Two same-type items: output durability = sum + 5% of max (rounded
  down), capped at max. ("plus 5% of the maximum durability of the
  output item" — NOT the anvil's 12%.)
- "Disenchanting an item in the grindstone does not remove curse
  enchantments or an item's custom name." Cursed items therefore
  never fully disenchant — with ONLY a curse on the input the output
  would be identical to the input (the wiki's cursed-row behavior:
  the grindstone refuses; the engine refuses the combine for
  curse-only items per the spec's "Cursed items return nothing —
  the grindstone refuses").
- "Unenchanted items [Java] placed inside will have a red cross over
  the arrow and no item in the output slot."
- Grindstone RESETS the prior work penalty.
- XP: "a uniformly distributed pseudorandom number between 50% and
  100% (rounded up) of all the non-curse enchantments' minimum
  modified enchantment levels combined". Per-enchant published XP
  ranges (max = the "minimum modified level" M) — encoded per
  registry enchant from the live table: Protection 1/11/23/33,
  Fire Protection 9/17/25/33, Feather Falling 5/11/17/23, Blast
  Protection 5/13/21/29, Projectile Protection 3/9/15/21, Respiration
  9/19/29, Aqua Affinity 1, Thorns 9/29/49, Depth Strider 9/19/29,
  Frost Walker 9/19, Soul Speed 9/19/29, Sharpness 1/11/23/33/45,
  Smite 5/13/21/29/37, Bane of Arthropods 5/13/21/29/37, Knockback
  5/25, Fire Aspect 9/29, Looting 15/23, Sweeping Edge 5/13/23,
  Power 1/11/21/31/41, Punch 11/31, Flame 19, Infinity 19, Efficiency
  1/11/21/31/41, Silk Touch 15, Fortune 15/23/33, Luck of the Sea
  15/23/33, Lure 15/23/33, Unbreaking 5/13/21, Mending 25, Channeling
  25, Impaling 1/9/17/25/33, Loyalty 11/19/25, Riptide 17/23/31,
  Multishot 19, Piercing 1/11/21/31, Quick Charge 11/31/51. Curses:
  excluded (0 XP).
- The grindstone BLOCK does not exist in the engine (the 1.14 village
  half was deferred). Round 13 registers it: hardness 2.0 (w/Grindstone
  §Breaking: "2.0"), drops itself, crafted 2 sticks + 1 stone slab +
  2 planks (w/Grindstone §Crafting; the engine's OAK_SLAB remains the
  "any slab" stand-in — documented craft.rs line ~680). No other
  1.14-village content is registered.

## 4. Reachability verdicts (the spec's [4]/[5]/[6])

- STONECUTTER — **DEFERRED**: the craft registry has no stone-family
  recipes and the block family is absent. Missing specifically:
  stone -> stone slab (1 -> 2), stone -> stone stairs, stone -> stone
  bricks (1 -> 4), stone -> chiseled stone bricks, AND the target
  block types stone slab / stone bricks / stone stairs themselves
  (only OAK_SLAB 57 and COBBLE_STAIRS 58 exist). The spec forbids
  registering new recipes this round.
- FLETCHING TABLE — **DEFERRED**: no arrow, bow, spectral arrow, or
  tipped arrow items exist; no fletching recipes (flint + stick +
  feather -> arrow etc.). Nothing to list.
- LOOM — **DEFERRED** (spec-mandated): depends on the banner system
  (16 colored banners + the pattern family). Not present.
- CARTOGRAPHY TABLE — **DEFERRED** (spec-mandated): depends on map
  items (base map, filled map, map scaling). Not present.
- SMITHING TABLE — **DEFERRED** (spec-mandated): depends on netherite
  gear items (netherite sword/pickaxe/axe/shovel/hoe + 4 armor
  pieces). Netherite ingot exists; the gear family does not.

## 5. Engine model notes (the honest-support decisions)

- ItemStack (vc-inventory) carries only block/count/ench (ONE
  enchant). The anvil's combine mode merges the sacrifice's enchant
  onto the target; with a single-enchant model a two-different-enchant
  merge cannot be represented. Round 13 extends ItemStack with:
  `dmg: u16` (damage taken), `prior: u8` (anvil use count), `ench2:
  u16` (a second enchant slot — enough for item+item and item+book
  combines; the cap is disclosed), `name: u16` (id into the game
  layer's name pool; 0 = registry name). Same reasoning as Round 12b's
  entity-inventory mandate: the station GUIs cannot exist without the
  model. Vanilla stacks >2 enchants; the engine caps at 2 (disclosed).
- The enchant registry gains `cost_mult: u8` (the anvil "multiplier
  from item"; the book multiplier is half, rounded down — verified
  from the wiki table's exact halves).
- Renamed items stack only with same-name same-penalty stacks; damaged
  items stack only at equal damage (vanilla). The merge condition in
  Inventory::slot_click gains these guards.
- The rename surcharge/costs flow through the pure anvil math; the
  name itself renders in the hover label through ContainerView's name
  pool snapshot.
