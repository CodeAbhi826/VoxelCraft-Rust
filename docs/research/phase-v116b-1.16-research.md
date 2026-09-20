# 1.16 "Nether Update" — research record, PART 2 (the forest families)

Date: 2026-09-08. Live captures: `voxelcraft/scripts/v116b_page_*.json`
(Strider, Piglin, Hoglin, Crimson_Stem, Crimson_Hyphae,
Crimson_Planks, Crimson_Fungus, Warped_Fungus, Crimson_Roots,
Warped_Roots, Weeping_Vines, Twisting_Vines, Nether_Sprouts,
Shroomlight, Warped_Wart_Block, Soul_Torch, Soul_Lantern,
Polished_Basalt, Polished_Blackstone, Polished_Blackstone_Bricks,
Crimson_Forest, Warped_Forest, Java_Edition_1.16, Bartering —
minecraft.wiki, extracted the same session). Values below are the
implementation contract.

## Scope decision (session-disclosed)

Part 2 is the second half of the 1.16 bracket: **the mobs (strider,
piglin, hoglin) and the crimson/warped wood families**, the two
forest BIOMES that host them, piglin bartering, and the polished
stone families that part 1's BASALT/BLACKSTONE docs deferred to
"the part-2 wood round". **Out of scope (with reasons)**: zoglin
(zombification needs mob dimension transfer — the engine has none);
piglin brutes (bastion interiors — no bastion structures);
zombified piglins (a 1.16 rename of a mob the engine never had);
bastion remnants (structure round, standing deferral); ruined
portals (no portal-ruin structures); lodestone (still needs the
compass item); soul-speed enchantment (no enchant system);
piglin-riding striders + saddles (no strider mount system — the
1-in-10 jockey's baby half rides alongside, disclosed).

## Strider (VERIFIED w/Strider)

- 20 HP passive Animal; hitbox 1.7 h x 0.9 w (baby 0.85/0.45);
  speed 0.175.
- "Lava does not damage striders, and they can walk on top of it
  without sinking" → feet-in-lava + air-above = standing (gravity
  off, on_ground); "If a strider spawns under lava, it rises out of
  the lava" → submerged = buoyant ascent.
- Water damage: "deal damage by 1 HP per splash water bottle or
  half-second in water or rain" (no rain in the engine — the water
  half rides the 0.5 s hazard window).
- Spawning: "Striders can spawn in every Nether biome. Groups of 2
  to 4 striders spawn on spaces of lava that have an air block
  above. In Java Edition, striders are the only passive mob in the
  Nether, so spawning attempts are made every 400 game ticks." The
  1-in-10 jockey row: baby-half implemented, riding-half trimmed.
- "They can be fed warped fungus to breed"; "All babies obtained
  through breeding take 20 minutes to grow up" (24000 ticks); drops
  "String 2-5" at 100%.

## Piglin (VERIFIED w/Piglin + w/Bartering + w/Crimson_Forest)

- 16 HP "Neutral (adult)" Monster; hitbox 1.95 x 0.6; speed 0.35.
- Attack: "Melee: Golden Sword: ... Normal: 8 HP" (the engine's
  melee row; the crossbow's 2-5 ranged row is melee-only here,
  disclosed).
- "It is hostile to players unless they wear at least one piece of
  golden armor" — no wearable armor in the engine: the adaptation
  is neutral-until-provoked (the enderman class) + the gold-mining
  anger hook, disclosed.
- Spawning: Nether Wastes + Crimson Forest; "Piglins are often seen
  in this biome in groups of 3-4" (w/Crimson_Forest).
- Bartering: "Adult piglins take gold ingots, whether dropped
  nearby or when a player uses one while looking at them, and barter
  certain items for them. The piglin 'examines' the ingot for six
  [JE] seconds, then drops a random item from the chart." → 120-gt
  countdown, then pending_drops (one item entity per unit).
- The trimmed barter chart (VERIFIED w/Bartering, the current
  wiki's /469 weights; potions/books/boots/spectral arrows/water
  bottles/dried ghast need absent systems — trimmed, ratios kept):
  the 40-class (obsidian 1, crying obsidian 1-3, gravel 8-16,
  blackstone 8-16, leather 2-4, soul sand 2-8), the 20-class
  (string 3-9, nether quartz 5-12), the 10-class (iron nugget
  10-36, ender pearl 2-4). Gold = the iron-ore/nugget stand-ins
  (the disclosed convention).
- "Soul torches repel piglins" (VERIFIED w/Soul_Torch) — the soul
  flame family (torch/lantern/fire) repels within 8 blocks.
- Aggravation: mining nether gold ore / gilded blackstone angers
  nearby piglins (16 blocks, the medium-aggravation class).

## Hoglin (VERIFIED w/Hoglin + w/Crimson_Forest)

- 40 HP hostile Animal "Monster"; hitbox 1.4 h x 1.3965 w (JE);
  speed 0.3; knockback resistance 60% (no knockback stat —
  disclosed).
- "Attack strength Adult in Java Edition: ... Normal: 3 HP to 8 HP"
  — engine takes the 5.5 midpoint, disclosed.
- "Hoglins avoid being within 7 blocks of warped fungi ... Nether
  portals and respawn anchors" — warped fungi + respawn anchors
  (portals absent); the flee outranks fighting and breeding.
- Spawning: Crimson Forest only ("the only biome where hoglins
  naturally spawn outside of bastion remnants"), 3-4 packs, "20% of
  hoglins spawn as babies" (JE).
- "Hoglins can be bred with crimson fungi"; babies flee when hit.
- Drops: "Raw Porkchop 2-4 100.00%" + "Leather 0-1 50.00%"
  (cooked-if-on-fire is the no-fire-on-entities deferral);
  "5 XP if killed by a player".

## The crimson/warped families (VERIFIED w/Crimson_Stem, w/Crimson_
## Hyphae, w/Crimson_Planks, w/Crimson_Fungus, w/Warped_Fungus,
## w/Crimson_Roots, w/Warped_Roots, w/Weeping_Vines, w/Twisting_
## Vines, w/Nether_Sprouts, w/Shroomlight, w/Warped_Wart_Block)

- Stems/hyphae: hardness 2 / blast 2 (the log class); hyphae is the
  all-sides "bark" form (the jungle-log-bark pattern).
- Planks: hardness 2 / blast 3; crafted 1:4 from stem or hyphae
  (the universal log→planks rule).
- Fungi: hardness 0, any tool, transparent cross plants. "It can
  also be fed to hoglins" (crimson) / "fed to striders or be
  planted to repel hoglins" (warped).
- Roots: hardness 0, transparent; "no longer require shears to drop
  themselves" (the 1.16.2-pre2 revert — plain self-drops).
- Nether sprouts: hardness 0, transparent; drops nothing without
  shears (no tool-gated drops in the engine — the empty-handed
  result, disclosed).
- Weeping vines: crimson forest, "downwards-growing", "These blocks
  have a 1/3 chance of dropping themselves"; twisting vines: warped
  forest, "upward-growing", the same 1/3 drop.
- Shroomlight: hardness 1 / blast 1, "Luminous Yes (15)", "generate
  in huge fungi".
- Warped wart block: hardness 1 / blast 1 (the huge-warped-fungus
  cap; crimson's nether wart block exists since 1.10).

## The forest biomes (VERIFIED w/Crimson_Forest + w/Warped_Forest)

- Crimson forest: "the second most common Nether biome, making up
  22% of the Nether by volume"; piglins 3-4 groups + hoglins +
  striders in lava lakes; "The forest floor is mostly covered with
  crimson nylium, with some netherrack and Nether wart blocks
  generating on the surface"; vegetation: crimson fungi + roots +
  huge crimson fungi + weeping vines.
- Warped forest: "the rarest of the five biomes in the Nether,
  making up around 8% of the Nether by volume"; "Endermen are
  common in this biome, and hostile mobs do not spawn naturally";
  "The floor of the biome is composed mostly of warped nylium";
  vegetation: warped fungi + roots + nether sprouts + huge warped
  fungi + "twisting vines growing from the ground".
- Engine adaptation (disclosed): region cells of 2x2 chunks (a
  deterministic hash) instead of vanilla's 3D multi-noise climate
  sampler; nylium + both forests are the engine's first nether
  sub-biomes.

## Soul torch / soul lantern (VERIFIED w/Soul_Torch, w/Soul_Lantern)

- Soul torch: "Luminous Yes (10)", hardness 0, any tool; crafted
  from charcoal-or-coal + stick + soul-soil-or-soul-sand → 4
  (shapeless). "Soul torches repel piglins but not piglin brutes."
- Soul lantern: "Luminous Yes (10)", hardness 3.5 / blast 3.5,
  transparent; "To hang a soul lantern from the bottom of a block,
  aim at the block's bottom face, and press use" (the lantern/
  chain face-matched pair). Craft: 8 iron nuggets + 1 soul torch
  (VERIFIED w/Soul_Torch §Crafting ingredient table).

## The polished stone families (VERIFIED w/Polished_Basalt,
## w/Polished_Blackstone, w/Polished_Blackstone_Bricks)

- Polished basalt: hardness 1.25 / blast 4.2; 4 basalt → 4 (the
  2x2 family part 1's BASALT doc deferred here).
- Polished blackstone: hardness 2 / blast 6; 4 blackstone → 4.
- Polished blackstone bricks: hardness 1.5 / blast 6; 4 polished
  blackstone → 4. Chiseled/cracked forms are the stonecutter
  family — no stonecutter in the engine, disclosed.

## Engine-side adaptations (the disclosed list)

- Piglin gold-armor pacification → neutral-until-provoked (no
  wearable armor).
- Piglin crossbow → melee-only (the golden-sword row).
- Hoglin knockback resistance 60% → absent (no knockback stat).
- Strider riding/saddle/jockey → trimmed (no mount system for
  striders); the jockey's 1-in-10 baby half rides alongside.
- Zoglin/piglin-brute/zombified-piglin mobs → trimmed with their
  systems (no dimension transfer, no bastions, no pre-1.16 pigman).
- The barter chart trimmed to engine-feasible items (ratios kept).
- Bastions/ruined portals → no structures (standing deferral).
- Nylium "drops netherrack when mined" (the grass-to-dirt class);
  stripped stems → no axe-strip mechanic (standing deferral).
- The hoglin zombification + drowned-style conversions → N/A.
