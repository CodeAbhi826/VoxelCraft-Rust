# ROUND-16-PARITY-GAP.md — the remaining known gaps after the Round 10–16 pass

Date: 2026-09-15 · the honest ledger of what is NOT yet vanilla-parity,
with size estimates (the spec's Round 16 [1] deliverable).

## Shipped this pass

- **Round 10 [1]** — the vanilla integer GUI-scale model (Auto =
  max(1, min(⌊w/320⌋, ⌊h/240⌋)), live-canvas logical space, integer
  device-px-per-vanilla-px blit; the fractional 0.72/0.86/1.0 path
  removed). Items [2]–[7] (armor registry, survival inventory, armor
  HUD wiring) shipped in the prior sub-rounds 2–3.
- **Round 11** — the 11-tab creative inventory (prior session, commit
  b477d24; re-verified this pass).
- **Round 12 [1]** — the double chest: partner scan (pure + tested),
  54-slot merged screen, single-word CHEST title, spill-both-halves on
  break, shulker/trapped no-cross-merge.
- **Round 14 (partial)** — Music & Sound screen (10 category sliders,
  live mixer gains), Controls screen (the KeyBinds table, click-to-
  rebind with ESC cancel, Reset Keys, persistence), Language screen
  (English-only, honest), Chat Settings (documented stub),
  Accessibility additions (Fog Fast/Fancy/OFF live, FOV Effects
  slider, grayed Chat Visibility/Subtitles).
- **Round 15 (partial)** — biome-tinted fog (20% blend at the player's
  biome), re-verified weather darkening + nether fog (already shipped).

## Deferred (with reasons)

| Gap | Reason | Estimate |
|---|---|---|
| Round 12 [2] mount storage (donkey/llama screens) | The entity system has Horse/Donkey/Llama mobs and riding, but NO entity-side inventories — needs a per-entity container model + the chest-equip interaction + a mount screen (a full round of work on its own) | 1 round |
| Round 13 anvil GUI | anvil.rs ships the block physics + prior-work penalty but no repair/combine/rename logic or GUI; needs enchantment-merge rules + a new container kind + level-cost formula | 1 round |
| Round 13 beacon GUI | beacon.rs ships the pyramid scan + effect application; needs the power-selection GUI (primary/secondary) + the payment slot | ½ round |
| Round 13 grindstone / loom / stonecutter / cartography / smithing / fletching GUIs | Each needs its station logic first (disenchant-XP, banner system, recipe lists, map items, netherite gear registration, arrow-recipe index) — the spec's own deferral clauses apply (banners and maps are not registered; netherite gear is not registered) | 1–2 rounds |
| Round 14 Skin Customization screen | The player model has no separate cape/jacket/sleeve layers to toggle — the spec allows showing inert toggles, but that pretends at parity; deferred honestly instead | ¼ round |
| Round 15 particle types (splash, bubble, drips, portal, end_rod, firework, squid_ink, dust, note, happy/angry villager, snowflake, totem, spit) | The particle system is block-texture billboards; each new type needs a procedural texture + spawn rules through the gameplay paths (water entry, mob events) — a batch of small but individually fiddly additions | 1 round |
| Round 15 sky ramp exactness (sun 30×30 / moon 20×20 cells, ~1500 stars) | The shader sky (sun disc + glow + hash stars + sunset band) is visually equivalent; the exact cell sizes are sub-pixel at the engine's rendering scale | ¼ round |
| Saved hotbars (the creative C-tab row, 1.12+) | No saved-toolbar store exists | ¼ round |

## Known issues (observed, not fixed this pass)

- The container-panel family chrome carries a constant ~+7 vanilla-eq-px
  overhead vs vanilla's 176-wide panels (all container screens uniformly;
  the slot grids are exactly vanilla-geometry). Disclosed in the round-12
  audit; a full panel-chrome rework would touch every container screen.

## Legal audit notes (Round 16 [2], this pass)

- Grep for "Minecraft/Mojang/Notch/Creeper" in user-facing strings: the
  mob display name "Creeper Spawn Egg" (blocks.rs) is a pre-existing
  naming choice from the earlier audit16 rounds (vanilla's own item
  name, shown on VLM-verified screens); flagged here rather than
  renamed mid-pass — a rename belongs to a dedicated string-freeze pass.
- The wasm bundle pair (voxelcraft.js + voxelcraft_bg.wasm) is the
  matched 14:45 Sep-15 rebuild; packs rsynced from builtin-pack/ +
  builtin-packs/programmer-art/ (procedural PNGs only).
- README disclaimer: untouched this pass (still the standing clean-room
  notice from the earlier rounds).
