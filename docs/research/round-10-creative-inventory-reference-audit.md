# Round 10 — Reference audit: the vanilla 1.16.5 creative inventory (Ultimate Prompt sub-round 2)

Date: 2026-09-15 · Prompt: `upload/Pasted Content_1789400071647.txt` §B ·
Wiki pages fetched LIVE 2026-09-15 (saved under `diag/wiki/`):
`Creative_inventory`, `Heads-up_display`, `Inventory`, `Options`.

## 1. What the live wiki says (the facts this round is built on)

### Creative_inventory (minecraft.wiki, rev 3761325, fetched live 2026-09-15)

- The **current** wiki describes the modern (1.19.3+) screen: 11 sections
  (Building Blocks, Colored Blocks, Natural Blocks, Functional Blocks,
  Redstone Blocks, Tools & Utilities, Combat, Food & Drinks, Ingredients,
  Spawn Eggs, Operator Utilities) plus Search Items, Saved Hotbars and
  Survival Inventory tabs.
- The **classic** (pre-1.19.3 = our 1.16.5 target) tab set is pinned by
  the page's history + gallery: "The Creative inventory before Java
  Edition 1.12" caption; 1.7.2 13w36a "The displayed item for the
  'Decoration Blocks' tab ... has been changed from a rose to a peony
  flower"; 1.9 15w31a "The Survival inventory has been rearranged to
  accommodate the new off-hand slot"; 1.12 17w06a "Added Saved Toolbars".
  The classic set (also stated in the Ultimate Prompt §B, our second
  source per the cross-check rule):
  **Building Blocks, Decoration Blocks, Redstone, Transportation,
  Miscellaneous, Foodstuffs, Tools, Combat, Brewing** (9 content tabs,
  in this order) **+ Search Items + Inventory** (structural tabs).
- Grid: **9 columns × 5 rows = 45 slots per page** (the prompt §B states
  it; the wiki's screenshots show it; tab overflow scrolls) — scrollbar
  on the right.
- Click semantics (all quoted live):
  - "A single item can be grabbed using left-click, increasing with
    continued left-clicks on that item."
  - "Right-clicking an item also picks up one item, but the second click
    then puts it back down as opposed to picking up more."
  - "Left or right-clicking with an item or stack of items while
    hovering over any item other than the one held gets rid of the held
    item."
  - "Shift-clicking an item grabs a full stack of items."
  - "Pressing a number key while hovering over an item instantly places
    one full stack of that item into the hotbar slot that corresponds
    with the number."
- The Search tab has a working text field; "Using the search tab, one
  can access enchanted books of any level" (search across all items).
- Bottom row: the hotbar + the destroy slot ("get rid of the held item").

### Heads-up_display (fetched live 2026-09-14, re-verified this round)

- "In Creative mode, the health, hunger, oxygen, experience, and armor
  bars are hidden." — the **effect icons are NOT in the hidden list**;
  "All effects ... the player currently has are shown on the top-right
  of the screen". This round hoists the effect-icon draw out of the
  mode gate so Creative shows them (the user's "what about the effects"
  callout).

### Options (fetched live 2026-09-15)

- GUI-scale formula (for sub-round 6, recorded here): "The number of
  available GUI scales ... can be calculated using this formula:
  `max(1, min(floor(width / 320), floor(height / 240)))` ... Auto sets
  the GUI scale to the highest value available for the current
  resolution." **No cap.**

## 2. Prompt-vs-wiki disagreements (cross-check rule: the wiki wins)

| # | Prompt §B says | Live wiki says | Resolution |
|---|---|---|---|
| 1 | Creative "hides all of these except hotbar, XP bar, boss bar, held-item name, and crosshair" (Subsystem A table) | "In Creative mode, the health, hunger, oxygen, experience, and armor bars are hidden" — XP IS hidden; effect icons stay | Round 9 already resolved: XP hidden. This round: effect icons render in creative too |
| 2 | (modern-era page) | The wiki's current text describes the 1.19.3+ tab set | Classic 9+2 tab set used (pinned by history rows + the prompt as second source) |

## 3. Engine adaptations (disclosed)

1. **Tab membership** is engine-adapted: the 515-block registry is not
   item-for-item vanilla. The mapping is canonical-anchored (grass →
   Building, poppy → Decoration, redstone components → Redstone, saddle
   → Transportation, spawn eggs → Miscellaneous — their 1.16.5 home,
   bread → Foodstuffs, hoe → Tools, shield → Combat, potions +
   brewing stand → Brewing) and pinned by the `canonical_tab_anchors`
   test. Uncertain memberships (station blocks → Miscellaneous, bowl/
   egg → Foodstuffs, nether quartz item → Miscellaneous) are disclosed
   here as judgment calls.
2. **Tab icons**: engine substitutes where the registry lacks the
   vanilla icon item — peony ✓ (exists), redstone dust ✓ (exists),
   apple ✓ (exists), brewing stand ✓ (exists), grass block ✓ (exists);
   minecart → saddle, lava bucket → lava block, iron pickaxe → hoe,
   iron sword → shield, compass → eye of ender, player head →
   wither-skeleton skull.
3. **Transportation tab is thin** (saddle only): the registry has no
   rails/minecarts/boats. Honest content, not a stub.
4. **Redstone tab completeness**: the registry's redstone components
   (wire, torch, lever, repeater, comparator, piston, dispenser,
   dropper, observer) never joined PICKER_BLOCKS; they ride the tab via
   `CREATIVE_REDSTONE_EXTRA`.
5. **Inventory tab** swaps to the survival inventory container screen
   (vanilla keeps it in-tab; the engine's screen-swap achieves the same
   reachability — documented adaptation).
6. **The old 15×11 scroll picker is replaced** (the prompt's ask). Its
   "SELECT BLOCK" chrome, wheel-scroll window and direct-assign click
   are gone; clicks now follow the vanilla cursor-stack semantics above,
   including the destroy slot and outside-click destruction.
7. **Single membership**: each block lives in exactly one tab (vanilla
   allows a few items in multiple tabs, e.g. redstone dust also in
   Brewing). Redstone dust stays in Redstone (primary), disclosed.

## 4. Verification chain (this round)

- Tests: `tabs_are_complete_and_ordered` (9 tabs, vanilla order, all
  non-empty, per-tab census 155/52/17/1/184/37/1/5/25 = 467 + 10
  extras), `canonical_tab_anchors` (23 anchor assertions + icon
  validity), `tab_items_are_unique_and_valid`,
  `creative_screen_geometry` (11 tab rects in vanilla order, 6+5 folder
  rows, 9×5 grid hit-mapping, scrollbar present on the Building tab,
  hotbar + destroy hit rects), `creative_screen_search_tab` (search
  field rect, no scrollbar on a single-page tab),
  `effect_icons_render_without_status_rows_creative` (the user's
  effects-in-creative callout).
- 744 → 752 tests, 0 failures; clippy clean (native + wasm32).
- Live browser E2E + VLM: performed at the end of the full pass
  (creative tab bar, tab switching, search, hotbar pick, destroy slot).

## 5. Deferred (with reasons)

- Drag-and-drop with a held stack across tabs (mouse-down drag) — the
  engine's click-pickup/click-place covers the flow; the drag gesture
  needs a pointer-drag pipeline the input layer lacks (menus are
  click-based). Vanilla-shape preserved by the two-click flow.
- Saved hotbars (the 1.12 C tab row) — 1.16.5 has it, but the engine has
  no saved-toolbar store; registered for the deferred list.
- The creative item count badge/scroll-per-item granularity (engine
  scrolls by row, vanilla scrolls per item internally but the page model
  is identical at 45/page).
