# VoxelCraft-Rust → Minecraft Replication: Research Dossier — PART 6
*Continues from Parts 1-5. This is a verification/correction layer on top of the large multi-turn GLM research dump the user relayed (referred to below as "the GLM dump") — not a re-copy of it. The GLM dump itself should be kept as a companion file; Part 6 tells you what in it to trust, what to fix, and what's still open.*

---

## 29. Context on the GLM dump's reliability — read this before using it

The user's process: gave GLM a 26.2 jar first, GLM researched against that, GLM caught mid-conversation that it was the wrong version, user then supplied a real 1.16.5 jar and research continued, and later the full repo was shared too. That explains and partly resolves the version-contamination concern raised initially. However, direct reading of the full dump surfaced two concrete, provable problems that are independent of jar version:

1. **A confirmed factual error that survived its own earlier correction.** Early in the dump, GLM correctly wrote `sweeping` "(not `sweeping_edge` — that ID was introduced later)" as a `[CORRECTION]`. Later in the *same* dump, its "final, complete" 38-enchantment table lists `sweeping_edge` with no caveat, reverting the earlier fix. **Confirmed via Mojang's own bug tracker (MC-271039, Fixed status): `minecraft:sweeping` was renamed to `minecraft:sweeping_edge` starting in snapshot 24w03a (2024)** — years after 1.16.5. The correct ID for this project is **`sweeping`**, not `sweeping_edge`. Apply this correction wherever the enchantment registry is used.
2. **Internally inconsistent entity-count sub-lists.** One "Passive mobs" list is labeled 30 but contains 32 named entries; a later one is labeled 28 but contains 30. The final "102 entities, confirmed" total is assembled by concatenating these mismatched lists rather than each being independently recount-checked. The registry *total* (102) is plausible and likely came from a real `jq` count, but the *category breakdown* (which mobs are "passive" vs "hostile" vs "neutral") should not be trusted at the sub-list level as given.

**Recommended confidence tiers for using the rest of the dump:**
- **High confidence** — raw registry key lists produced by piping the user's real `registries.json`/`commands.json`/`blocks.json` through `jq` (block list, item list, entity list, enchantment list, command list, biome list, structure list, recipe type list, villager profession list, mob effect list, potion list, particle list, attribute list, menu list, POI list, custom stat list, and the raw counts: 763 blocks, 976 items, 79 biomes, 102 entities, 38 enchantments, 41 potions, 74 commands, 18 structures). This is mechanical key extraction from real local files — low fabrication risk, assuming the jar was genuinely 1.16.5 by that point in the conversation.
- **Needs individual spot-checking before implementation** — anything with a *value* attached that required wiki synthesis rather than raw key extraction: hardness/blast-resistance numbers, mob health/damage/speed stats, villager trade prices/stock/XP, loot table weights, enchantment max levels, potion durations, redstone-adjacent claims not already resolved in Part 5. This is exactly the category where real errors were already caught (Qwen's obsidian error in Part 5, GLM's own sweeping_edge slip here).
- **Confirmed wrong, already fixed above:** the `sweeping`/`sweeping_edge` ID.

---

## 30. New verified data this round — closes a gap the GLM dump left open

**Full enchantment-level-by-bookshelf-count formula (the GLM dump explicitly said this wasn't fully extracted — now confirmed from minecraft.wiki directly):**

```
base = randomInt(1,8) + floor(b/2) + randomInt(0,b)
```
where `b` = nearby bookshelf count (capped at 15). Per-slot results:
- Top slot level = `floor(max(base/3, 1))`
- Middle slot level = `floor(base×2/3 + 1)`
- Bottom slot level = `floor(max(base, b×2))`

Confirms the GLM dump's "15 bookshelves → level 30 max" claim was correct, and replaces its "not fully extracted" placeholder with the actual formula. Reference table (bookshelves → level ranges) is available on minecraft.wiki's `Enchanting mechanics` page if exact per-count ranges are needed for testing.

---

## 31. Recommended next action — get an authoritative recount directly, not by re-reading the dump

Since the entity/mob category breakdown in the GLM dump is internally inconsistent, the cheapest fix is having GLM (or anyone with the real files) re-run a direct, unambiguous query rather than trying to reconcile the prose lists by hand:

```bash
# Exact, unambiguous entity list — no category guessing
jq -r '.["minecraft:entity_type"].entries | keys[]' registries.json | sort

# Exact enchantment list (will show "sweeping", confirming §29 correction)
jq -r '.["minecraft:enchantment"].entries | keys[]' registries.json | sort
```
This sidesteps the whole "which prose list do I trust" problem — the registry file itself is the source of truth, and `jq` against it can't misremember a category the way free-form generation can.

---

## 32. Open items carried forward

- Villager trade tables, loot table weights, and per-mob stat values in the GLM dump are usable as a *starting draft* but should be spot-checked against minecraft.wiki directly before being treated as final, per the tiering in §29.
- Whether the blockstate/asset-format research (the "dump3.txt" analysis in the GLM transcript) was done against the corrected 1.16.5 jar or still reflects contamination from the earlier wrong-version upload is unconfirmed — worth a direct check if the `vc-pack` blockstate work depends on it being version-exact.
- All open decisions from Parts 1-5 still stand.
