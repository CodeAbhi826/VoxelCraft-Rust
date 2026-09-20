# Phase 1 Research Notes — MC 1.0–1.2 bracket (live round, 2026-09-06)

All values below were checked against live minecraft.wiki pages fetched this
session (raw HTML archived in /tmp/phase1/*.json at fetch time). Per the
STRICT PROTOCOL: re-verify at implementation time — these notes are the
record of THIS round, not a license to skip future re-checks.

## Mobs

| Value | Verdict | Source (live 2026-09-06) |
|---|---|---|
| Snow Golem HP 4; snowball 0 dmg, 3 dmg vs Blaze only | ✅ | minecraft.wiki/w/Snow_Golem infobox + §Behavior |
| Snow Golem: 2 snow blocks + carved pumpkin on top (pumpkin LAST); shear → 1 carved pumpkin | ✅ | §Spawning/§Drops |
| Snow Golem: throws 1 snowball/s at monsters up to 10 blocks | ✅ | §Behavior |
| Snow Golem: 1 HP/tick damage in temp > 1.0 biomes (savanna[JE], badlands, desert, Nether) + rain/water contact | ✅ | §Behavior |
| Snow Golem snow trail: page lead says temperature-gated; §Behavior says "any biome (Java)". ⚠ DISAGREEMENT within one page — implemented temp < 0.5 gate (Snowy Taiga/Mountains/Taiga), disclosed in worklog | ⚠ | same page, two readings |
| Magma Cube HP = size² (16/4/1), attack = size+2 (6/4/3 normal), armor = 3×size (12/6/3) | ✅ | minecraft.wiki/w/Magma_Cube §Combat |
| Magma Cube sizes 1/2/4 spawn naturally; splits into 2-4 on death; XP 4/2/1 by size | ✅ | §Spawning/§Drops/§Behavior |
| Magma Cube: seeks player/iron golem ≤ 16 blocks; idle jump every 40-120 ticks (target: 13-40); direction change 40-100 ticks; jump height = size blocks; fireproof | ✅ | §Behavior |
| Blaze HP 20; fireball dmg Easy 3.5/Normal 5/Hard 7.5; contact 4/6/9; fire 1 HP/s 5 s | ✅ | minecraft.wiki/w/Blaze infobox |
| Blaze: charge 3 s then 3 fireballs at 0.3 s intervals; fireball accelerates to ~38 b/s; floats up while targeting; follow range 42.5–53.5 | ✅ | §Behavior |
| Blaze: drops 0-1 blaze rod (50%, Looting scales); 10 XP; spawns in fortresses light ≤ 11, groups 2-3 (JE) | ✅ | infobox/§Spawning |
| Fortress: up to 2 blaze spawner platforms, nether brick fences, 3-block staircase | ✅ | §Spawning |
| Ocelot HP 10; passive; spawns in jungle (JE weight 2/93, group 1-3) on grass at sea level; attacks chickens ≤ 15 blocks; flees players; trust via raw cod/salmon (1/3 chance each); NOT tameable in 1.16; 1-3 XP | ✅ | minecraft.wiki/w/Ocelot |
| Iron Golem HP 100; attack Normal 7.5–21.5 (Easy 4.75–11.75); created 4 iron blocks T + pumpkin last; drops 3-5 iron ingots + 0-2 poppy (drop table unreadable via extraction — widely-cited values, flagged) | ✅/⚠ | minecraft.wiki/w/Iron_Golem |
| Iron Golem: village guard, attacks hostile mobs + low-reputation players; offers poppy | ✅ | §Behavior |
| Zombie Villager HP 20; dmg Easy 2.5/Normal 3/Hard 4.5; villager→zombie-villager conversion on zombie kill: Easy 0%/Normal 50%/Hard 100%; 5% of zombie spawns are zombie villagers (JE); cure = Weakness + golden apple, 3600–6000 ticks (3–5 min); XP 5 (baby 12) | ✅ | minecraft.wiki/w/Zombie_Villager |
| Mooshroom HP 10; spawns ONLY in Mushroom Fields (JE weight 8/8, group 4-8); shear → 5 mushrooms + cow; bowl → mushroom stew; breed wheat; 5% baby; red↔brown on lightning | ✅ | minecraft.wiki/w/Mooshroom |
| Ender Dragon HP 200; melee Easy 6/Normal 10/Hard 15; wings Easy 3.5/...; takes damage ONLY from players + explosions; immune fire/fall/drown/freeze/poison/lightning/void; non-head damage ~×0.25 (formula text garbled — documented approximation) | ✅/⚠ | minecraft.wiki/w/Ender_Dragon |
| Dragon healed 1 HP per 10 ticks (0.5 s) by nearest crystal within 32-block cuboid; destroying a healing crystal deals 10 HP | ✅ | w/Ender_Dragon + w/End_Crystal |
| Dragon first death: 12000 XP; XP page says "10 waves of 1000 + 1 of 2000", dragon page says "10 drops of 960 + 1 of 2400" — ⚠ INTRA-WIKI DISAGREEMENT (both sum 12000); implemented the dragon-page split (960×10+2400), disclosed | ⚠ | both pages live |
| Dragon death: 154 ticks after ascension start XP appears one per tick; at 200 ticks exit portal fills + dragon egg spawns above center; End gateway 96 blocks out at Y=75; max 20 gateways; re-summoned kills give 500 XP | ✅ | w/Ender_Dragon §Death and drops |
| End Crystal: crafted glass + eye of ender + ghast tear; place on obsidian/bedrock with 2 air above; explodes power 6 when damaged (charged-creeper power); damaged-by-explosion → just disappears (JE); fire block created at location when placed in the End | ✅ | minecraft.wiki/w/End_Crystal |
| 10 crystals on pillars, 2 in iron-bar cages | ✅ | w/End_Crystal + w/The_End |

## XP orbs

| Value | Verdict | Source |
|---|---|---|
| Orb split into base values 1, 3, 7, 17, 37, 73, 149, 307, 617, 1237, 2477 | ✅ | w/Experience §Orbs |
| Attraction: glide toward player up to 7.25 blocks (center-of-feet to orb center), speeding up nearer | ✅ | w/Experience |
| Pickup is gradual: max 10 orbs/second, collected at feet (15w46a) | ✅ | w/Experience |
| Despawn after 6000 ticks (5 min) | ✅ | w/Experience |
| Hitbox 0.25 blocks; green↔yellow fade; value ≥ 17 shows a dense core | ✅ | w/Experience |
| Orbs MERGE only from 20w45a (1.17) — 1.16.5 orbs DO NOT merge | ✅ (version-scoped) | w/Experience history |
| Mobs only drop XP if killed by player or within 100 ticks of a player hit | ✅ | w/Experience |

## Spawn eggs

| Value | Verdict | Source |
|---|---|---|
| Use (right-click) on any surface → mob with feet adjacent to surface; not thrown; consumed | ✅ | w/Spawn_Egg §Usage |
| Egg on same mob type with baby form → baby; egg on spawner changes its mob; creative-only item | ✅ | w/Spawn_Egg |

## Blocks / world gen

| Value | Verdict | Source |
|---|---|---|
| Mycelium: spreads to dirt 1 up / 1 sideways / 3 down; needs light ≥ 9 (dirt side ≥ 4); reverts to dirt under opaque cover with light < 4; Silk Touch to keep; snowy variant state | ✅ | w/Mycelium |
| Redstone lamp: light 15 when powered, 0 off; on instantly, off after 4 game ticks (0.2 s JE — history "2-tick delay" = 2 redstone ticks = 4 game ticks); opaque; craft 4 glowstone + 1 redstone | ✅ | w/Redstone_Lamp |
| Stone bricks: chiseled variant exists (1.2-era family); strongholds generate them | ✅ | w/Stone_Bricks |
| Sandstone variants: chiseled (2 sandstone slabs), cut (2×2 sandstone → 4), smooth (smelting; 18w43a made it smelt-only — valid for 1.16.5) | ✅ | w/Sandstone |
| Nether brick (item) smelted from netherrack; nether bricks block builds fortress | ✅ | w/Nether_Brick |
| Nether wart: 4 stages (age 0-3), 10% chance per random tick to advance; only on soul sand; fortress stairwell gardens (~20 plants); fully grown drops 2-4, immature 1; no bone meal; any light | ✅ | w/Nether_Wart |
| End stone: hardness 3, blast resistance 9; generates in the End | ✅ | w/End_Stone |
| Mushroom Fields: ~0.15% of overworld, mycelium surface, ocean islands, NO natural hostile spawns (mooshrooms + bats only); huge mushrooms generate abundantly; biome temp 0.9 | ✅ | w/Mushroom_Fields + w/Biome |
| Huge mushrooms: red = stalk + 5× 3×3 dome slabs; brown flat cap; growth min 5 blocks clear; all huge mushrooms have exactly 45 cap blocks besides the stalk | ✅ | w/Huge_mushroom |
| Nether Fortress: regions 432×432 (Java), nether brick bridges + corridors on pillars above lava; up to 2 blaze spawner platforms; up to 20 corridor turns with 1/3 loot-chest chance; nether wart near stairwells | ✅ | w/Nether_Fortress |
| The End: 5×5 portal-frame ring (12 frames, corners cut) in stronghold over lava; activates with 12 eyes; destroys central 3×3 → end portal blocks; entry point (X:100, Z:0) on 5×5 obsidian platform; central island of end stone; obsidian pillars on 42-radius circle down to y=0, bedrock + crystal on top; exit portal activates on dragon death | ✅ | w/The_End |
| Biome temps (live table): Plains 0.8, Forest 0.7, Birch Forest 0.6, Snowy Taiga -0.5, Jungle 0.95, Taiga 0.3, Mushroom Fields 0.9, Nether Wastes 2.0, Badlands 2.0, The End 1.5 | ✅ | w/Biome |
| Biome temps not directly extractable (Desert/Mountains/Ocean/Beach/Savanna rows): using widely-cited 2.0 / 0.2 / 0.5 / 0.8 / 0.95 — flagged ⚠ in worklog | ⚠ | extraction limitation |

## Deferred from this bracket (recorded, not implemented)

- Wither-skeleton spawning in fortresses (mob is a Phase-2 bracket item,
  1.4) — fortress code leaves the spawn hook ready.
- Beds (1.0 sleep mechanic) — the evolution list's Phase 1 does not
  include beds; the mushroom-fields/End work does not require them.
