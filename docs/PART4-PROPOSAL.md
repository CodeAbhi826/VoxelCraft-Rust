# Part 4 World-Gen Parity — 4.0 Black-Box Proposal (approval gate)

## Method (L1 black-box only)

1. The owner generates reference worlds in their own copy and hands
   over **numeric diffs only**: oracle match rates, per-column height
   deltas, biome-ID agreement, structure-center distances, seam scores.
2. I implement **only from public documentation** (reference wiki
   pages, archived Customized tables, data-file extractions such as
   misode/mcmeta JSON, Ken Perlin's published algorithm) **plus** the
   numeric diffs, which tell me *how far off* a documented parameter
   is — never *what code* produces it.
3. No decompiled output is used at any step. No community
   reimplementation built from decompiled sources is read or ported —
   not for structure, naming, or constants. Anything that cannot be
   sourced to a public document is marked `[ESTIMATED]` or dropped.
4. The existing oracle (2.2a–c) is the sole measuring instrument; the
   existing clean-room stack (`vanilla_noise.rs`, simplex/fBm,
   Java-Random-equivalent LCG) is the sole implementation base.

## Constants needed, with sources

| Slice | Constants | Public source |
|---|---|---|
| 4.1 biomes | temperature/humidity sampler octaves, scales; biome depth/scale weights | noise_settings JSON (data extraction), Customized defaults table (archived wiki) |
| 4.2 structures | spacing, separation, salts, scatter counts | reference wiki structure pages (documented values); salts only if published — else jitter-shape matching on numeric diffs |
| 4.3 terrain | cell sizes, sampling factors, density offset/factor, slide params, sea level | noise_settings JSON + Customized table (already cited in `vanilla_noise.rs`) |
| 4.4 carvers/deco | cave/ravine frequency bands, tree/decorator densities | reference wiki world-gen pages; approximate allowed only with owner approval |

Rule: a constant enters the tree only with its source cited in a
code comment. Unsourced tuning is flagged `[ESTIMATED]` and needs
explicit approval per constant.

## Acceptance gates

- 4.1/4.2: 100% structure/biome agreement on oracle seeds.
- 4.3: ≥99% blocks + seam score reported.
- 4.4: honest report; approximate only with approval.
- Golden worldgen hash: any change = HARD STOP (existing rule).

## Approval ask

Approve (a) the method above, (b) tuning documented-but-unsourced
constants against numeric diffs, (c) the 4.1→4.4 slice order. On
approval I start 4.1; any L1 ambiguity mid-part stops the line.
