# VoxelCraft-Rust — Version Evolution Research: Supplement
*Adds mechanic-level detail beneath the feature-name list already provided. Does not
repeat that document — read both together. Scope note up front: exhaustively
researching mechanic-level detail for every one of ~700+ features across 16 major
versions is not achievable with full confidence in one pass. This supplement covers
a representative set of high-value mechanics that are commonly left out of
feature-name overviews, verified to the standard used throughout this project
(real citations, disagreements between sources flagged, nothing asserted on a
single low-quality source).

## Beacon (added 1.4) — full mechanic, cross-verified across two independent sources

| Pyramid tier | Blocks (that layer / total) | Effect range | Effect duration | Effects unlocked |
|---|---|---|---|---|
| 1 | 9 (3×3) | 20 blocks | 11s | Speed I, Haste I |
| 2 | 25 / 34 (5×5) | 30 blocks | 13s | + Resistance I, Jump Boost I |
| 3 | 49 / 83 (7×7) | 40 blocks | 15s | + Strength I |
| 4 | 81 / 164 (9×9) | 50 blocks | 17s | Secondary slot: Regeneration I, **or** upgrade primary to level II |

Duration formula: `9 + 2×tier` seconds. Effect re-applies every 4 seconds while in
range (this is why it feels continuous, not a one-shot buff) — confirmed
consistently across two independently-styled sources.

**Confirmed WRONG, seen in a low-quality source this round — do not implement**:
a claim that stacking multiple beacons of the same effect combines to a higher
effect level (e.g. "Speed II + Speed II = Speed IV"). This does not match how
Minecraft's status-effect system works (a player just gets the higher of any
overlapping same-type effects, not an additive stack) and was not corroborated by
any other source. Treat as fabricated until independently confirmed.

Requires clear sky access directly above (glass/water/leaves/slabs/bedrock don't
block it; any other solid block does) — real mechanic, low-risk to implement as
stated.

## Beds exploding outside the Overworld (added 1.16, part of the Nether Update)

Confirmed real and intentional (Mojang's own bug tracker explicitly resolves related
tickets as "Works As Intended," not a bug) — sleeping in or placing/using a bed in
the Nether or End triggers an explosion instead of setting spawn. This is a genuine
1.16 mechanic worth adding alongside the dimension work already in the repo.

**Not verified this round**: the exact explosion power/damage value. Sources this
round only described it qualitatively ("stronger than TNT," unverified blog claim).
Get the real value from minecraft.wiki's Bed page directly before hardcoding a
damage number — do not use the "stronger than TNT" claim as a number.

## Totem of Undying (added 1.11) — real source disagreement found, resolved

Checked across five independently-styled sources; they disagreed on exact numbers:

| Source | Regeneration | Fire Resistance | Absorption |
|---|---|---|---|
| Source A | II, 5s | I, 40s | I, 5s |
| Source B | II, 45s | I, 40s | II, 5s |
| Source C | II, 40s | (not mentioned) | II, 5s |
| Source D (notes Java/Bedrock split explicitly) | — | — | Java = 45s, Bedrock = 40s |

**Resolved reading**: the version-split source is the most technically careful one
(it's the only one that explains *why* the numbers disagree — a real Java/Bedrock
edition difference, not random error). For this project's Java 1.16.5 target, the
correct values are most likely **Regeneration II for 45 seconds, Fire Resistance I
for 40 seconds, Absorption II for 5 seconds**, restoring 1 HP and clearing all
negative status effects. **Flagging this as still not fully certain** — the
disagreement pattern (some sources say 5s, some say 40s, some say 45s) suggests at
least one is simply wrong rather than version-specific, and this should get a direct
minecraft.wiki confirmation before being hardcoded, not just this cross-referenced
best-guess.

Mechanically confirmed and low-risk across all sources: must be held in main hand or
offhand to trigger; triggers automatically on what would otherwise be fatal damage;
consumed on use (one-time); dropped exclusively by Evokers (Woodland Mansions or
raid waves 5/7), unaffected by Looting.

## Scope limitation — be honest about this before treating "the evolution list" as complete

Given the size of this task (763 blocks, 976 items, 102 entities, 71 advancements,
38 enchantments, 79 biomes, 18 structures, spread across 16 major versions), a
genuinely complete mechanic-level research pass is a multi-session undertaking, not
a single research turn. The right way to handle the remaining depth is the same
pattern already established in this project: **research and verify each version
bracket immediately before implementing it**, not all 16 versions up front. Trying
to front-load exhaustive mechanic detail for versions that won't be implemented for
weeks risks the exact staleness/drift problem already caught twice in this project
(GLM's own sweeping_edge regression, the dripstone version-contamination error) —
research done too far ahead of implementation has more time to go stale or be
misremembered before it's used.

**Recommendation**: treat the original evolution document's version-by-version
ordering as the correct *sequence*, but do the mechanic-level deep-dive for each
version bracket at the start of implementing that bracket, not all at once now.
