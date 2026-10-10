# Part 4 climate re-architecture — parameter source doc (4.0 gate)

Goal: replace the fBm climate predicates with a clean-room biome
layer stack, because measurement (owner Survival copy, seed
7998960918674860355) shows 4.8% biome agreement: the layout, not the
thresholds, is wrong.

## Source triage (L1/L4)

REFUSED (decompiled or derived from decompiled code — never opened
for implementation, listed here only as a refusal record):
- MCP-style `GenLayer*.java` reimplementations and mapping-repo
  mirrors (class/method structure mirrors decompilation).
- Per-layer salt/seed constants from any code-derived source.

CLEAN (public behavioral documentation, citable):
- Wiki "Biome/Before 1.18" §Generation: island start → zoom ×2 to
  1:256 scale → jagged-coast layer; hills stack scaled ×2, applied at
  1:64; river stack scaled ×4, merged at 1:4; smooth layer; shore
  layers; documented edge transitions (cool/warm, heat/ice, special);
  documented hill/mutation pairs per base biome (taiga→snowy/giant,
  beach→snowy beach/mushroom shore, ocean→deep variants, etc.).
- Wiki Large_Biomes: ×4 scale variant (already implemented).
- Our own oracle measurements (fit source, 4.0 method).

## Build plan (no decompiled logic)

1. New `layers` module: island grid, doubling zoom (behavioral:
   each zoom doubles resolution with jittered picks), edge rules
   (documented transitions only), river init/mix at 1:4, smooth,
   shore assignment, mushroom-island rarity. Every rule cites the
   wiki behavior or our measurements; every free number is marked
   FIT.
2. Parameters to FIT against the copy (automated oracle loop):
   island density, zoom-stage counts within documented ranges,
   add-island probabilities, hill/mutation injection rates,
   river frequency, shore width. Engine-local salts throughout.
3. classify() consults the stack first; surfaces/trees/depth pairs
   follow existing tables. Heights stay density-driven.
4. Golden re-pins expected (new layout) — covered by the standing
   pre-1.0.0 rule; nether/end must stay byte-identical.

## Approval ask

Approve (a) building from behavioral descriptions above while
refusing code-derived constants, (b) fitting free parameters against
copy measurements, (c) the re-pins this entails. Anything that needs
a decompiled-only value stops with a question instead of a guess.
