# Audit-Fix Round Research Record (2026-09-07)

Live-verification record for the Phase-1/2 evolution-audit fix round
(the 1.2 jungle family + vines + ferns, and the 1.4 golden carrot).
Raw page captures: `voxelcraft/scripts/auditfix_page_*.json` +
`auditfix_search_jungletree.json`. Every row below was checked
against the live wiki at implementation time (STRICT PROTOCOL).

| Claim | Verdict | Source |
|---|---|---|
| Golden Carrot hunger 6, saturation 14.4, consumption 32 game ticks | ✅ | minecraft.wiki/w/Golden_Carrot infobox (live) |
| Golden Carrot added Java 1.4.2 12w34a (bracket attribution) | ✅ | w/Golden_Carrot §History |
| Golden Carrot craft = gold nugget + carrot | ✅ | w/Golden_Carrot §Crafting (nuggets absent in engine → picker-only, documented) |
| "Golden carrots are used to tame, breed, lead, grow, and heal horses, donkeys, and mules, and to breed, lead, and grow rabbits" | ✅ | w/Golden_Carrot §Usage (live) |
| "Feeding two tamed horses golden apples or golden carrots activates love mode" | ✅ | w/Horse §Breeding (live in the E3 round; re-cited) |
| Log (incl. Jungle Log): hardness 2, blast 2, flammable (5), axe-quickest, smelts to charcoal, fuel | ✅ | w/Log ("Redirected from Jungle Log") infobox + §Fuel |
| Leaves: hardness 0.2, blast 0.2, transparent, flammable (30) | ✅ | w/Leaves infobox |
| Jungle sapling drop rate 2.5% (1/40); sticks 2% | ✅ | w/Leaves §Breaking (neither item exists in engine — drops nothing, documented) |
| "Jungle trees of both sizes have vines on their trunks and canopy edges" | ✅ | w/Vines §Natural generation |
| Vines: "climbable non-solid vegetation blocks that grow on walls" | ✅ | w/Vines lead |
| "Vines can be climbed by standing next to them and holding the jump key. If there is a solid block behind the vines, the walk forward key can also be used" | ✅ | w/Vines §Behavior |
| "Vines cancel a sprint if the player is sprinting" | ✅ | w/Vines §Behavior |
| "Players are now slowed when going through vines due to their nature of being a collisionless ladder" (1.2.1 12w04a) | ✅ | w/Vines §History — the version-scoped (1.2-era) anchor |
| "Vines are now climbable provided they are climbed against a solid block" (1.2.1 12w03a) | ✅ | w/Vines §History |
| Ladder climb: "moves upward at about 2.35 blocks per second" | ✅ | w/Ladder §Climbing |
| Ladder descent: "maximum downward speed is reduced to a 'descending ladder' speed, at about 3 blocks per second" | ✅ | w/Ladder §Climbing |
| Ladder sneak: "Holding the sneak key while climbing a ladder causes the player to grab hold" | ✅ | w/Ladder §Climbing |
| "Vines absorb all fall damage, even without a solid surface nearby" | ⚠️ edition-untagged | w/Vines §Behavior — current page, no JE/BE tag on that sentence; NOT implemented as a blanket rule (climbing resets fall distance — the 1.2-era ladder semantics). Placeholder-unresolved, disclosed in the WORKLOG |
| Fern: "non-solid plant blocks... same characteristics as grass", hardness 0, shears to obtain, 12.5% wheat-seed drop | ✅ | w/Fern (live; seeds item absent → drops nothing, documented) |
| "Ferns occur naturally only in jungle, taiga, snowy taiga and old growth taiga biomes and their variants, scattered with short grass" | ✅ | w/Fern §Natural generation — CORRECTED the first draft (which wrongly included swamp) |
| "Added jungle trees" — Java 1.2.1 12w03a | ✅ | w/Tree §History |
| Jungle bushes: "featuring a single jungle log surrounded by oak leaves" | ✅ | w/Tree |
| Jungle trees "1×1 trunk, which can extend up to 10 blocks tall"; mega 2×2 "over 30 blocks" | ✅ | search round: minecraft.wiki w/Jungle_Tree + w/Jungle (cross-checked 3 sources) |
| Logs → 300-tick furnace fuel (incl. jungle); "Logs, but not stems, can be used as a fuel in furnaces" | ✅ | w/Log §Fuel + the existing engine fuel table |
| Universal 1 log → 4 planks recipe | ✅ | w/Log §Crafting (the "Wood/Hyphae 4 / 3 / 75%" family table) |

## Audit findings (the round's cause)

Cross-check of `evolution-research.md` Part 2 (1.0–1.4 sections) ↔
`docs/WORKLOG.md` ↔ code, with the e1/e2 research docs:

- Silently absent (never implemented, never deferred): golden
  carrot; jungle wood/leaves family; vines; ferns. All fixed this
  round.
- Silently absent, formally deferred this round with reasons:
  carrot-on-a-stick, language support (N/A), glass silk-touch
  pickup, jungle sapling, jungle log X/Z placement states, vine
  spread, golden-carrot rabbit breeding.
- Documentation-only gaps (code was right): sunrise/sunset colors
  implemented but never listed in the e1 WORKLOG; the jungle-oak
  adaptation was disclosed only in a gen.rs comment rather than a
  deferral entry.
- Everything the e1/e2 WORKLOGs claimed was confirmed present in
  code; 426/426 pre-round, 437/437 post-round; zero
  todo!/unimplemented!/unsafe.
