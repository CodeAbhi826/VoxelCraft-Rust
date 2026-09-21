# Round 9 (prompt sub-round 1) — Survival HUD reference audit

**Task:** the Survival HUD completeness sub-round of the 2026-09-14
"Ultimate Prompt" parity pass. This document is the pre-implementation
reference audit required by that prompt's verification protocol.

**Sources fetched live 2026-09-14** (the reference wiki):
`w/Heads-up_display`, `w/Armor`, `w/Hunger_(effect)`, `w/Experience`.
Reused standing repo research: `upload/VANILLA-VALUES.md` (V/C-marked),
`upload/SCREEN-SPECS.md` §13, `docs/VERIFICATION-REPORT.md` §2 (stale
in parts — re-checked against code below).

---

## 1. Element inventory (vanilla 1.16.5, Survival)

| Element | Wiki-verified behavior | Current code state | Action |
|---|---|---|---|
| Hearts | 10 icons above hotbar left half; full/half/empty | ✅ drawn (9×9 quads, 18×18) | add damage flash + low-health/regen shake |
| Armor bar | "appears above the health bar if the player is wearing armor" — sum of piece armor points, 2 pts/icon, 10 icons | ❌ sprite + `GuiFrame::armor()` exist but never called; no armor slots on the player | wire to armor slots (Sub-round 3 prerequisite lands early) |
| Hunger | 10 icons, right half, mirrored | ✅ drawn; **food argument hardcoded 20.0** at the call site | keep visual; jitter is a Hunger-effect recolor question (see §3) |
| Oxygen | "breath meter displays above the hunger bar if the player's air supply unit is below 300" | ✅ drawn when air < 299, right-aligned above hunger | keep; pop-sound/wobble are 1.21+ — N/A for 1.16.5 |
| XP bar | 182×5 vanilla; level number above | ⚠️ drawn 8 UI px tall (= 4 vanilla-eq) | height → 10 UI px (= 5 vanilla-eq) |
| Hotbar | 182×22; 9 slots; selection frame | ✅ 364×44 + selection | keep |
| Crosshair | 15×15 invert-blend center | ✅ invert-blend with H/V split | keep |
| Boss bar | while dragon/wither live | ✅ both bosses wired | keep |
| Held-item name | shows after selection change, ~2 s fade | ⚠️ hidden in creative (`!invulnerable()` gate) | show in both modes |
| Status effect icons | 1.9 15w31a: "top-right corner… blinks when about to run out"; sooner-expiring farther left; positive top row, others bottom row | ❌ not drawn | add (16 effects exist engine-side) |
| Offhand slot | 1.9 15w31a: "a slot appears on the left of the hotbar" when holding an item there | ❌ no offhand slot exists | defer to Sub-round 3 (needs the offhand data model) — documented |

## 2. Per-mode element set (the wiki-vs-prompt disagreement)

Live wiki (Heads-up_display §Display, fetched 2026-09-14): "In Creative
mode, the health, hunger, oxygen, experience, and armor bars are
hidden."

The prompt's own Subsystem A table claims Creative keeps the XP bar.
Per the prompt's cross-check rule ("if sources disagree, implement the
more conservative interpretation and document the disagreement"):
**the wiki wins** — Creative shows only crosshair + hotbar + boss bar +
held-item name. The current `xp_bar_only()` (XP + bubbles in creative)
is a documented engine deviation and is retired this round. The old
rationale ("levels still matter for enchanting") stays recorded here.

## 3. Animation facts (what is actually documented)

- **Hunger effect recolor** — w/Hunger_(effect): "It also turns the
  hunger bar a yellow-green color." Verified. The prompt's "±1 px
  jitter" phrasing is NOT wiki-documented as a number → implemented as
  clean-room approximation: yellow-green tint + 1-px row jitter while
  the effect is active (marked C).
- **Damage flash** — not documented numerically on the wiki → the
  prompt's clean-room spec (red vignette, 0.3 alpha, ~4 ticks) is used
  and marked C.
- **Hearts shake** — vanilla jitters the row while Regeneration ticks
  and at low health; not wiki-documented as numbers → clean-room ±1 px
  jitter (marked C).
- **Bubble pop sound / wobble / empty state** — wiki history section
  dates these to 1.21.x → N/A for 1.16.5 (correctly absent).
- **Effect icon blink** — "blinks when about to run out" (verified) →
  blink in the final ~5 s (C for the exact threshold; vanilla is
  understood to be 5 s ≈ 100 ticks).

## 4. Numbers pinned for implementation (960×540 canvas = vanilla GUI 2)

- Heart row: 10 × (9×9) icons at 8-px vanilla pitch → 18×18 at 17-px
  pitch (current layout is already this; verified `status_bars`).
- Armor row: same geometry as hearts, one row higher (wiki: "above the
  health bar").
- XP bar: 182×5 vanilla → **364×10 canvas px** (current 364×8 is the
  4-px-tall deviation).
- Armor icon palette: body `#E1E1E1`, shade `#A5A5A5` (V/C table).
- Armor points per piece (vanilla helmets 1-3, chests 3-8, legs 2-6,
  boots 1-3 by tier): leather 1/3/2/1, gold 2/5/3/1, chain 2/5/4/1,
  iron 2/6/5/2, diamond 3/8/6/2, netherite 3/8/6/2 (w/Armor points
  table, live 2026-09-14). The engine has no wearable armor items yet
  (audit §2.5: armor formula ✅ but no equip layer) — the HUD row
  renders from the new armor-slot array, which the survival inventory
  screen (Sub-round 3) will fill; until items exist it stays hidden
  exactly like vanilla's "0 armor points = no row".

## 5. Drift flagged (prompt's "trust the code" rule)

- Prompt claims "the 2048² procedural atlas stays" — the block atlas is
  **512×512** (`ATLAS_SIZE = 512`, 32×32 grid of 16×16 tiles,
  `TILE_MAX = 774`); the 2048² atlas is the separate **item icon**
  cache (`ICON_ATLAS_PX = 2048`). No change made; recorded here.
- Audit-doc counts drifted: 506 blocks/805 states/41 sounds → live code
  says **515 blocks / 845 states / 43 sound events / 737 tests** (test
  count re-verified this round by the full workspace run).
- `docs/VERIFICATION-REPORT.md` §2/§4 is pre-UI-overhaul (Monocraft,
  quad sprites, GUI Auto all landed after it) — its ❌ rows for GUI
  Scale and armor icons are being closed across rounds 8-9.
