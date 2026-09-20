# Round 12 + 14 — Reference audit: double chest, settings tree

Date: 2026-09-15 · Prompt: `upload/Pasted Content_1789476877305.txt` Rounds 12 + 14 ·
Wiki pages fetched LIVE 2026-09-15: `Chest`, `Controls`, `Options`,
`Accessibility`, `Shulker_Box` (re-verified cached fetches under `diag/wiki/`).

## 1. What the live wiki says

### Chest §Double chests (live 2026-09-15)

- "Placing two chests of the same type next to each other ... combines
  them into a large chest" — 54 slots (9×6), one shared GUI.
- Trapped chests pair only with trapped chests; a normal chest beside a
  trapped chest stays single.
- The GUI title stays "Chest" (single word) — NOT "Large Chest".
- §Breaking: breaking either half drops the contents of BOTH at the
  broken half's location; the survivor becomes a normal single chest.
- Panel 176 wide × 220 tall at scale 1 (the engine's container-family
  chrome overhead is disclosed in ROUND-16-PARITY-GAP.md; the 9×6 slot
  grid itself is exact).

### Shulker_Box (re-verified)

- Shulker boxes never combine with any neighbor — the merge scan matches
  CHEST/TRAPPED_CHEST exactly, so shulkers can never pair (pinned by the
  `double_chest_merges_adjacent_chests` test).

### Controls (live 2026-09-15)

- The screen lists every bindable action with its current key; clicking
  a key button enters the rebind flow ("press the new key"); a Reset
  Keys button restores defaults.
- Categories: Movement, Inventory, Gameplay, Miscellaneous, Multiplayer,
  Debug — the engine's rebindable set fills Movement + Inventory; the
  fixed binds (F3, hotbar 1–9, ESC) are the non-rebindable class.

### Options §Music & Sound (live 2026-09-15)

- "Music & Sound ... has sliders which control the volume of each sound
  category." The 1.16.5 categories: master, music, record (jukeboxes /
  note blocks), weather, blocks, hostile, neutral, players, ambient,
  voice — the engine's `SoundCategory` already models all ten
  (Sub-round 5), so the screen is the ten sliders over the existing
  per-category gain chain.

### Accessibility (live 2026-09-15)

- The page documents the MODERN screen (1.19.4+): FOV Effects,
  Distortion Effects, Darkness Pulsing, High Contrast, Narrator,
  Subtitles, etc. The 1.16.5-era screen is essentially Auto-Jump.
- Resolution: Auto-Jump (already shipped) + the spec's Fog cycle and FOV
  Effects slider land as engine adaptations (both wired to real
  renderer effects); Darkness Pulsing is OMITTED (1.19+ feature, per the
  spec's own verify-and-omit rule); Chat Visibility / Subtitles render
  grayed until their subsystems exist; Distortion Effects is omitted
  (no nausea/portal warp system exists — an honest omission beats an
  inert slider; logged in ROUND-16-PARITY-GAP.md).

## 2. Spec-vs-wiki disagreements (cross-check rule: the wiki wins)

| # | Spec says | Live wiki says | Resolution |
|---|---|---|---|
| 1 | Round 14 lists "Fog: OFF / Fast / Fancy" + Darkness Pulsing + Distortion on the Accessibility screen | The modern page's options postdate 1.16.5 (Darkness Pulsing is 1.19+) | Fog + FOV Effects shipped as engine adaptations; Darkness Pulsing omitted per the spec's own verify rule; Distortion omitted honestly (no warp system) |

## 3. Engine adaptations (disclosed)

1. **The double chest is a VIEW**: each half keeps its own 27-slot
   container entity; opening scans the four horizontal neighbors and
   concatenates 27+27 into the 54-slot screen. Closing drops the view;
   breaking either half spills both (the neighbor's stacks drop at the
   broken half's site, then the survivor is empty — matching vanilla's
   observable outcome).
2. **The partner scan is deterministic** (+X/−X/+Z/−Z first-match).
3. **Controls**: the rebindable set is the engine's real bind surface
   (movement ×6, inventory ×3); vanilla's non-rebindable entries (ESC,
   F3, hotbar keys) stay hardcoded. Rebind persistence rides
   `key_<action>:<KeyCode>` settings pairs.
4. **Language** lists English only (the spec forbids pretending).
5. **Chat Settings** is a titled stub (the spec forbids adding a chat
   system this round).

## 4. Verification (this pass)

- `double_chest_merges_adjacent_chests` — pairing incl. trapped/shulker/
  vertical negatives + the deterministic order.
- `double_chest_screen_geometry` — the 54-slot 9×6 grid, pitches, hit
  tests, half ordering, panel math.
- `music_sound_screen_layout_and_mapping`, `control_rebind_changes_
  effective_key`, `controls_screen_layout_shape`,
  `accessibility_options_persist_round_trip`.
- Live browser E2E after the wasm rebuild (screenshots at the two
  canonical resolutions).
