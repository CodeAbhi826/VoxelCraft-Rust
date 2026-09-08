# 1.16 "Nether Update" — research record, PART 1 (the anchor family)

Date: 2026-09-08. Live captures: `voxelcraft/scripts/v116_page_*.json`
(Respawn_Anchor, Target, Crying_Obsidian, Soul_Soil, Soul_Fire, Basalt,
Blackstone, Ancient_Debris, Netherite_Scrap, Netherite_Ingot,
Nether_Gold_Ore, Chain, Gilded_Blackstone, Lodestone — minecraft.wiki,
extracted the same session). Values below are the implementation contract.

## Scope decision (session-disclosed)

1.16 is the biggest bracket left. Following the 1.14 precedent (nature
half first), this round is **part 1 — the anchor family**: the
signature respawn-anchor mechanic, the target block, the soul-fire
family, the basalt/blackstone terrain, and the ancient-debris →
netherite material path. **Part 2 (next)**: the mobs (strider, piglin,
hoglin) and the crimson/warped wood families. **Out of scope (with
reasons)**: lodestone (needs the compass item — the engine has none);
piglin bartering (no piglins yet — part 2); ruined-portal generation
(no portal-ruin structures); soul-speed enchantment (no enchant
system); soul torch/lantern (torch family exists — can ride part 2 with
the wood rounds); Bastion remnants (structure — with the piglin round).

## Respawn Anchor — the signature mechanic (VERIFIED w/Respawn_Anchor)

- Craft: **6 crying obsidian + 3 glowstone**.
- Hardness 50, blast resistance 1,200, pickaxe.
- Charging: **glowstone adds one charge, max 4**. Luminous by charge:
  **charge 1/2/3/4 → light 3/7/11/15** (each glowstone after the first
  +4, cap 15); uncharged → no light.
- Setting respawn: use it (like a bed) — **needs ≥ 1 charge AND must be
  in the Nether**; a confirmation appears; using it overrides any other
  spawn. Each respawn **consumes one charge**.
- Using it in the Overworld/End: **the block explodes, power 5, sets
  fire to surrounding blocks** (the bed-in-nether pattern).
- Redstone: a charged anchor emits a comparator signal = its charge
  count (the E3 viewer-signal pattern can carry this).

## Target (VERIFIED w/Target)

- Craft: **4 redstone dust + 1 hay bale**. Hardness 0.5, blast 0.5.
  (Redstone dust is absent in this engine — the REDSTONE BLOCK stand-in
  is the recipe's dust slot, disclosed.)
- Hit by a projectile → **redstone power for 8 game ticks; arrows and
  tridents → 20 ticks (1 s, the stone-button row)**.
- **Strength 1–15 by how close the hit is to the block's center**.
- The engine has one projectile family in flight (skeleton arrows,
  llama spit, blaze fireballs, snowballs, tridents) — arrows + tridents
  get the 20-tick window, the others 8 ticks (verified list includes
  llama spit and fireballs).
- JE blockstate `power` 0..15 — the E3 blockstate-encoded-signal
  pattern (daylight sensor precedent).

## Soul fire / soul soil (VERIFIED w/Soul_Fire, w/Soul_Soil)

- Soul soil: hardness 0.5, shovel; "burns indefinitely when manually
  ignited on the top side only, creating soul fire". Soul sand also
  hosts soul fire (the fire page's §Soul fire).
- Soul fire: **light 10**; **"the fire inflicts damage at a rate of 2
  HP per tick, twice as many as with the normal fire (although damage
  immunity reduces this to once every half-second)"** → in-engine:
  2 HP per half-second through the shared damage-immunity window —
  exactly the campfire's rate doubled (the engine's fire-contact rate).
  It does not spread. Fire/flint ignition isn't in the engine — soul
  fire generates on soul patches (disclosed; no flint-and-steel item).

## Crying obsidian (VERIFIED w/Crying_Obsidian)

- **Light 10**, hardness 50 (the obsidian class), blast 1,200.
- Obtained: bastion loot (part 2) and **piglin bartering (~8.53%,
  40/469, 1–3)** — no piglins yet: craft-only + world-gen trace this
  round (a few vein spots in nether blackstone, disclosed adaptation;
  the vanilla source is ruined portals/bastions, neither exists yet).
- The purple drip particles are visual polish (the engine's particle
  set can carry a drip) — optional, disclosed if skipped.

## Basalt / blackstone terrain (VERIFIED w/Basalt, w/Blackstone)

- Basalt: hardness 1.25, blast 4.2. Generates as **pillars in soul
  sand valleys**; the engine's nether has no sub-biomes — basalt
  appears as **pillar clusters + blobs** on the netherrack body
  (disclosed adaptation), including the lava-on-soul-soil basalt
  generator rule where the engine's lava contact allows.
- Blackstone: hardness 1.5, blast 6. Vanilla sources are bastions +
  ruined portals + "small patches in all Nether biomes" (20w19a row,
  VERIFIED) — this round takes the small-patches generation (floor
  patches in the nether, more at low y).
- Gilded blackstone: **10% chance to drop 2–5 gold nuggets when mined
  with any pickaxe; otherwise drops itself** (Fortune raises the
  CHANCE, not the count — no Fortune in engine, disclosed). Generated
  inside blackstone patches (the bastion-native ore, our patch-based
  adaptation).

## The netherite path (VERIFIED w/Ancient_Debris, w/Netherite_Scrap,
## w/Netherite_Ingot)

- Ancient debris: hardness 30, **blast 1,200**; **"never naturally
  exposed to air"**; replaces netherrack/basalt/blackstone. Java gen:
  **one cluster of 0–3, triangle distribution y 8–24 (peak 16), + one
  cluster of 0–2, even y 8–119, per chunk**. Smelting: ancient debris
  → **1 netherite scrap + 2 XP**.
- Netherite scrap: "resistant to fire and lava in item form" (the
  engine's item-burn path is lava-contact erasure — scrap + ingot +
  block are exempted there, the same fire-immunity class as... this is
  the engine's first item with the property; disclosed mechanism: no
  item fire in this engine, so the property documents itself).
- Netherite ingot: **craft 4 netherite scrap + 4 gold ingots**. Gold
  is the engine's **iron-ingot stand-in** (the disclosed convention
  since golden-apple/golden-carrot). Netherite gear upgrading (the
  smithing template) needs the tool/armor system — standing 1.11+
  deferral; the ingot exists as a material + crafts the block.
- Block of netherite: 9 ingots ↔ 8... vanilla is 9:1 both ways with
  1.16.5 having... VERIFIED row: "9 netherite ingots" for the block,
  block → 9 ingots. Blast resistance 1,200.

## Nether gold ore (VERIFIED w/Nether_Gold_Ore)

- Drops **2–6 gold nuggets when mined with any pickaxe** (Fortune
  multiplies — absent, disclosed); smelts to a gold ingot (0.1 XP class).
- The nugget is the engine's **iron-nugget stand-in** (1.14b's item —
  the disclosed gold convention; 9 nuggets ↔ 1 ingot both ways, the
  lantern precedent).

## Chain (VERIFIED w/Chain — version-scoped)

- The 1.16 recipe: **iron nuggets + iron ingot** (the current wiki
  table shows "Iron Nugget or Copper Nugget + Iron Ingot or Copper
  Ingot" — the copper halves are 1.21 additions, out of bracket; the
  1.16 form is the iron-only row, VERIFIED by the §History absence of
  any 1.16–1.16.5 change). Shape: 2 nuggets over/under 1 ingot.
- Hardness 0.5, blast 6; hangs lanterns (the engine's lantern already
  hangs from undersides — chains extend the same face-matched states).
- The 9-variant chain list on the page (copper chains) is 1.21+ — out
  of bracket.

## Engine-side adaptations (the disclosed list)

- Gold → iron-ingot stand-in, gold nugget → iron-nugget stand-in (the
  pre-existing convention).
- Redstone dust → redstone block in the target recipe (no dust item).
- Piglin anger on gold-ore mining: part 2 (no piglins).
- Explosion fire-spread: the engine's creeper-explosion path carries
  the anchor's power-5 overworld blast (fire-spread disclosed if the
  explosion path doesn't ignite).
- Crying obsidian world-gen trace: a few blackstone-embedded spots
  (vanilla: ruined portals/bastions/barter — all part 2 or absent).
