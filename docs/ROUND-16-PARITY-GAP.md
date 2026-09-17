# ROUND-16-PARITY-GAP.md — the remaining known gaps

Date: 2026-09-17 (updated; first cut 2026-09-15) · the honest ledger
of what is NOT yet vanilla-parity, with size estimates (the spec's
Round 16 [1] deliverable).

## Shipped since the 2026-09-15 cut

- **Round 10 [1]** — the vanilla integer GUI-scale model (Auto =
  max(1, min(⌊w/320⌋, ⌊h/240⌋)), live-canvas logical space, integer
  device-px-per-vanilla-px blit; the fractional 0.72/0.86/1.0 path
  removed). Items [2]–[7] (armor registry, survival inventory, armor
  HUD wiring) shipped in the prior sub-rounds 2–3.
- **Round 11** — the 11-tab creative inventory (prior session, commit
  b477d24; re-verified).
- **Round 12 [1]** — the double chest (partner scan, 54-slot merged
  screen, spill-both-halves on break).
- **Round 12b — mount storage CLOSED (2026-09-17)** — the
  per-entity MountStorage model (donkey/mule 15; llama 3×strength),
  the mount SCREEN (saddle slot + 5-wide grid / STR badge + partial
  rows), chest-equip through the use-interaction, E-while-riding, and
  the death spill (chest + contents). E2E-verified live.
- **Round 13 — station GUIs CLOSED** — anvil (Repair & Name: combine
  math, prior-work penalty, rename, Too Expensive!), grindstone
  (Repair & Disenchant + XP), beacon (power selection + payment +
  level gates). E2E-verified live 2026-09-17.
- **Round 14 (partial) + 14b** — Music & Sound (10 sliders + the
  2026-09-17 tooltip completion), Controls (rebind table), Language,
  Chat Settings (real screen), Accessibility completion (Hold/Toggle,
  Distortion/FOV Effects), Skin Customization (8 layers + Main Hand).
- **Round 15 (partial) + 15b** — biome fog; the 20-kind particle
  batch (splash/bubble/drips/portal/end_rod/firework/squid_ink/dust/
  note/villager moods/snowflake/totem/spit); the sky exactness (sun
  30×30, moon 20×20 at distance 100, pinned star density).
- **Saved hotbars CLOSED (2026-09-17)** — the 12-tab creative strip
  (Search/Hotbars/Inventory), C+n save + X+n load (the vanilla
  defaults, held-state combos), options-store persistence
  (hotbar0..8), the tab grid with placeholder rows. E2E-verified
  live (both boot logs + the tab screen).

## Deferred (with reasons)

| Gap | Reason | Estimate |
|---|---|---|
| Round 13 loom / stonecutter / cartography / smithing / fletching | The SPEC'S OWN deferral clauses: banners, map items, the stone-family recipes, netherite gear and arrows are not registered (audit §4 verdicts) | blocked on their subsystems |
| Hunger-drain gameplay system | The HUD displays the vanilla 20/20 spawn state; the drain/regen simulation is a gameplay round of its own | 1 round |
| Container-panel family chrome (+~7 vanilla-eq-px vs the 176-wide panels) | A full panel-chrome rework touches every container screen (disclosed in the round-12 audit) | ½ round |
| Saved-hotbar counts in the tab grid | The creative grid is the u16 item grid; the X+n load preserves counts but the tab view does not render them | trivial, cosmetic |

## Known issues (observed, not fixed this pass)

- (none new — the three latent defects from the interrupted commits
  were fixed 2026-09-17: the name-pool underflow, the R13 state
  decode window, the missing music-slider tooltips)

## Legal audit notes (Round 16 [2])

- Grep for "Minecraft/Mojang/Notch/Creeper" in user-facing strings: the
  mob display name "Creeper Spawn Egg" (blocks.rs) is a pre-existing
  naming choice from the earlier audit16 rounds (vanilla's own item
  name, shown on VLM-verified screens); flagged for a dedicated
  string-freeze pass rather than a mid-round rename.
- The wasm bundle pair (voxelcraft.js + voxelcraft_bg.wasm) is the
  matched 2026-09-17 16:43 rebuild (wasm-bindgen 0.2.127, glue
  patched, packs rsynced from builtin-pack/ +
  builtin-packs/programmer-art/ — procedural PNGs only).
- README disclaimer: the standing clean-room notice, unchanged.
