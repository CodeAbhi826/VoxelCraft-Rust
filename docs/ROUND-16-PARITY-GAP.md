# ROUND-16-PARITY-GAP.md — the remaining known gaps

Date: 2026-09-18 (updated; first cut 2026-09-15) · the honest ledger
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
  batch (splash/bubble/drips/portal/void_rod/firework/squid_ink/dust/
  note/villager moods/snowflake/totem/spit); the sky exactness (sun
  30×30, moon 20×20 at distance 100, pinned star density).
- **Saved hotbars CLOSED (2026-09-17)** — the 12-tab creative strip
  (Search/Hotbars/Inventory), C+n save + X+n load (the vanilla
  defaults, held-state combos), options-store persistence
  (hotbar0..8), the tab grid with placeholder rows. E2E-verified
  live (both boot logs + the tab screen).
- **Round 17 — the hunger-drain gameplay system CLOSED (2026-09-18)**
  — the standing deferral closed: the full vanilla FoodData model
  (`vc_gameplay::hunger`: foodLevel / foodSaturationLevel /
  foodExhaustionLevel / foodTickTimer), the exhaustion-accumulation
  hooks (sprint 0.1/m, swim 0.01/m, jump 0.05, sprint-jump 0.2,
  attack 0.1 landed, damage 0.1, mining 0.005, Hunger effect
  0.005/tick/level, regen 6.0/HP), the natural regeneration (80-tick
  cadence) + the Java-only saturation boost (10-tick cadence), the
  starvation branch with the per-difficulty stop thresholds (Normal
  1 HP / Hardcore Hard-class never-stops), the sprint gate (food > 6,
  mayfly bypass), the live HUD food bar (replacing the hardcoded
  20/20) + the saturation-zero hunger-bar jitter, the FoodData eat
  path (the hunger/2 direct-heal convention RETIRED — the full
  nutrition/saturation table re-verified live against the Food
  page), the respawn reset (20/5), and the cadence defect fix (the
  effects tick + lava contact damage ran per FRAME — now per 20 Hz
  sim tick). All wiki-cited live 2026-09-18
  (docs/research/round-17-hunger-audit.md).

## Deferred (with reasons)

| Gap | Reason | Estimate |
|---|---|---|
| Round 13 loom / stonecutter / cartography / smithing / fletching | The SPEC'S OWN deferral clauses: banners, map items, the stone-family recipes, hollowite gear and arrows are not registered (audit §4 verdicts) | blocked on their subsystems |
| Container-panel family chrome (+~7 vanilla-eq-px vs the 176-wide panels) | A full panel-chrome rework touches every container screen (disclosed in the round-12 audit) | ½ round |
| Saved-hotbar counts in the tab grid | The creative grid is the u16 item grid; the X+n load preserves counts but the tab view does not render them | trivial, cosmetic |
| The in-game hollow portal (Phase 12A of the master plan) | Newly identified in the 2026-09-18 master-plan review: the Hollow dimension itself is COMPLETE (five biomes, fortresses, 8:1 coords, rebirth anchors, pigoblin/boarling content, hollow fog) and reachable through the §28 travel pipeline + the E2E `dim:` command, but the obsidian-frame portal block + the flint-and-steel item + the walk-in trigger do not exist (the Void portals are the only in-game dimension path; the fire block exists via lightning only). A portal round needs: the PORTAL block registration + frame validation + flint-and-steel item + portal particles/fog overlay + the walk-in travel + spawn-frame building on the far side | 1 round |
| Hunger NBT persistence (foodLevel/foodSaturationLevel/foodExhaustionLevel in the player block) | The engine's level.dat player NBT carries no health/xp either — every world entry is a fresh 20 HP / 20 food (the wiki's own world-creation semantics). Riding the existing convention this round; a future persistence round carries the whole vital-stat set | with the persistence round |
| Peaceful / Easy difficulty rules | The engine has no difficulty selector (worlds run Normal-class starvation; Hardcore runs Hard-class per the modes doc). The two unreachable rule rows stay disclosed | with a difficulty selector round |

## Known issues (observed, not fixed this pass)

- (none new — the three latent defects from the interrupted commits
  were fixed 2026-09-17: the name-pool underflow, the R13 state
  decode window, the missing music-slider tooltips; the frame-cadence
  effects/lava defect was fixed 2026-09-18 with the round-17
  per-sim-tick block)

## Legal audit notes (Round 16 [2])

- Grep for "the reference game/the original publisher/the original author/Fuseling" in user-facing strings: the
  mob display name "Fuseling Spawn Egg" (blocks.rs) is a pre-existing
  naming choice from the earlier audit16 rounds (vanilla's own item
  name, shown on VLM-verified screens); flagged for a dedicated
  string-freeze pass rather than a mid-round rename.
- The wasm bundle pair (voxelcraft.js + voxelcraft_bg.wasm) is the
  matched 2026-09-18 rebuild (wasm-bindgen 0.2.127, glue
  patched, packs rsynced from builtin-pack/ +
  builtin-packs/classic-art/ — procedural PNGs only).
- README disclaimer: the standing clean-room notice, unchanged.
