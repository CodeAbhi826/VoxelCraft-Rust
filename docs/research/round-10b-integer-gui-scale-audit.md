# Round 10 — Reference audit: the vanilla integer GUI-scale model

Date: 2026-09-15 · Prompt: `upload/Pasted Content_1789476877305.txt` Round 10 §[1] ·
Wiki pages fetched LIVE 2026-09-15 (the earlier fetches cached under
`diag/wiki/options.json`): `Options`, `Options.txt`, `Inventory`.

## 1. What the live wiki says (the facts this round is built on)

### Options (the reference wiki, fetched live 2026-09-15)

- "The number of available GUI scales ... can be calculated using this
  formula: `max(1, min(floor(width / 320), floor(height / 240)))`"
- "Auto sets the GUI scale to the highest value available for the current
  resolution."
- **No cap** on the scale at any resolution.
- The 1.16.5 client code (`MainWindow#calculateScale`) increments `i`
  while `width/(i+1) >= 320 && height/(i+1) >= 240` — mathematically
  identical to the wiki formula at every resolution tested.

### Options.txt (fetched live 2026-09-15)

- `gui_scale` = 0 (Auto) | 1 | 2 | 3 | 4 in 1.16.5; values above the
  available max clamp to the max at resolution time.

### Inventory (fetched live 2026-09-15 — re-verified from the sub-round-3 audit)

- The survival-inventory layout facts (176×166 panel, armor column,
  offhand, 2×2 grid, 9×3 + hotbar) shipped in sub-round 3; unchanged.

## 2. Spec-vs-wiki disagreements (cross-check rule: the wiki wins)

| # | Spec pack says | Live wiki + 1.16.5 code say | Resolution |
|---|---|---|---|
| 1 | Auto test table: 854×480→1, 960×540→1, 1280×720→2, 1440×810→2, 1920×1080→3, 2560×1440→4 "(capped)", 3840×2160→4 | 854×480→2, 960×540→2, 1280×720→3, 1440×810→3, 1920×1080→4, 2560×1440→6, 3840×2160→9 — no cap | **Wiki wins.** The spec table appears to subtract 1 from the formula and invent a cap; both authoritative sources agree on the plain formula. Documented here + in the `gui_scale_auto_matches_vanilla` test header |
| 2 | "Feed the GuiRenderer's existing scale uniform the integer scale" | The engine's canvas raster is 2 canvas px per vanilla px (the 960×540 reference grid) | The uniform receives the mapping derived from the integer scale: device px per canvas px = scale/2 — an INTEGER device size per vanilla px at every scale, preserving every existing 2× layout constant |
| 3 | "Logical GUI space = (fb_w/scale, fb_h/scale)" | same | Implemented literally: the live canvas resizes to ceil(2·fb/scale) canvas px; menus re-center and the HUD re-anchors at the live edges (= true screen edges) |

## 3. Engine adaptations (disclosed)

1. **The canvas raster stays 2 canvas px per vanilla px.** Every layout
   constant in the engine (400×40 cp buttons = vanilla 200×20, 364 cp
   hotbar = vanilla 182, 352×332 cp survival panel = vanilla 176×166)
   already encodes the 2× reference. Resizing the raster to
   (2·fb_w/scale, 2·fb_h/scale) makes the canvas BE the logical space
   and retires the fixed 960×540 letterbox — no layout constant moves.
2. **Odd scales map a canvas px to a half device px** (scale 3 → 1.5
   dpx/cp) — but every VANILLA px still lands on exactly `scale` device
   px, which is the property vanilla itself guarantees. The GPU text
   path rasterizes glyphs at the live device scale, so text stays crisp.
3. **Rounding**: the live size rounds UP (ceil) so content never clips
   at the last row/column; the blit fit absorbs the sub-pixel
   difference (≤1 device px).
4. **The default canvas (960×540) is unchanged** — every headless test
   and the 1920×1080@Auto(4) fullscreen case produce the exact classic
   grid.

## 4. Verification (this round)

- `gui_scale_auto_matches_vanilla` — the wiki-pinned table (8
  resolutions) + manual-setting clamps.
- `logical_gui_space_uses_integer_scale` — the live-canvas resize math,
  the px-grid indexing at a non-reference size, and the no-op
  same-size resize.
- `live_ui_size_hints_recenter_layouts` — menu rows re-center at a
  narrower live width while vanilla px sizes stay put.
- Live browser E2E: screenshots at 1280×720 (scale 2 canvas ≈ 854×480)
  and 1920×1080 (scale 3 canvas = 1280×720... at the preview iframe's
  device size) after the wasm rebuild.
