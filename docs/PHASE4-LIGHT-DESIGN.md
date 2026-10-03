# PHASE4-LIGHT-DESIGN.md — the light-engine init optimization (Phase 4, #1 cost)

> Grounded in the Phase-1 measurement (docs/BASELINE-PERF-2026-10-02.md): the
> light engine's `init_chunk` is THE #1 cost — 86.6 ms/chunk on the reference hardware,
> 15.6 ms on the GH runner — slower than meshing (49.1/10.6) and dominating
> chunk loading. Design only (no code landed yet); Phase 4 implements it
> ranked by measured payoff, every change gated by the CI bench.

## The measured target (Phase 1's proposal)

Stable 30 fps on the reference hardware (amortize the ~190 ms/chunk pipeline), 60+ on mid,
no hitches > 50 ms (the light-init spike is THE target), memory < 20 KiB/chunk
(today 10.3 KiB/chunk — healthy). The light init must land under ~15-20 ms/chunk
on the reference hardware for the 30 fps target; on the runner it is already 15.6.

## The cost structure (code-verified 2026-10-03)

`vc-world/src/light.rs` `init_chunk`:

1. **The sky column scan** — 16×16×256 = 65,536 cells, each a `chunk.get(
   lx, y, lz)` call: a section match + `state_block(s)` — and `state_block`
   is a LONG match chain (vc-blocks: two range `contains` checks, then
   ~40 match arms over the state windows) per cell.
2. **The block-light emissive scan** — another 65,536 cells, each
   `chunk.get_state` + `state_emissive(s)` (another match chain), regardless
   of whether the cell's SECTION exists (`get_state` on a None section
   returns 0 but the call + the chain walk still happen).
3. The BFS propagation (the pending sets) — proportional to the lit volume,
   not the chunk volume.

The two 65,536-cell scans dominate: ~131k match-chain walks per chunk on a
slow reference hardware core ≈ the 86.6 ms. Both scans are per-COLUMN over all 256 y —
they ignore section presence entirely.

## The optimizations (ranked)

- **O1 — iterate per-SECTION, skip empty bands**: for each of the 16 bands,
  `if section.is_none() { skip }` then the 4,096 cells of that band. Most
  generated chunks have only the surface bands populated (the world height
  is 256; the terrain top is ~70) — the upper bands are None. Cuts the cell
  iterations ~4-6× for BOTH scans. Zero semantic change: a None section is
  all-air, the column scan writes sky=15 there... CAREFUL: the sky scan's
  column descent crosses bands — the empty-band cells need sky=15 written
  (the vec default is 0, NOT 15) — so O1 applies cleanly to the EMISSIVE
  scan only; the sky scan needs the empty bands filled with 15 (a
  `vec![15u8; ...]` + the scan writing the attenuated cells... the scan
  descends top-down: empty bands above the terrain keep 15 — see O2).
- **O2 — the sky scan stops at the heightmap**: initialize the sky array to
  15 (all-air world default) and scan top-down writing only the ATTENUATED
  cells (water sub(2), leaves sub(1), opaque → 0 + everything below 0 —
  write a 0 run from the first opaque to the bottom... no: below the first
  opaque the column may re-enter air (a cave) — the vanilla column-scan
  reference semantics keep l=0 below the first opaque UNLESS the BFS
  propagates sideways into it (the established semantics: the scan sets
  l=0 at the first opaque and the cells below stay 0 — the BFS fills caves
  from the neighbors). So the scan CAN break at the first opaque: the cells
  below stay 15 from the init?? NO — they must be 0, not 15. The correct
  form: fill 15, scan down writing the attenuated values, and at the first
  opaque write 0 for the remaining column (a memset-like run, no per-cell
  match chain). Saves ~half the scan AND the per-cell fold for the whole
  below-heightmap column.
- **O3 — a static per-state lookup table**: a `const`-built or
  lazy-initialized `[u8; STATE_COUNT]` table (opaque | emissive | the
  attenuation class packed) turns BOTH match chains into indexed loads. The
  table is 874 entries × 1 byte — trivial memory. `state_block` itself stays
  for the cold paths; the hot scans read the table.
- **O4 (stretch) — a per-section emissive bitmap** maintained at
  generation/edit time: the emissive scan then touches only the sections
  that actually contain emissive blocks (usually 0-1 per chunk). Cuts the
  emissive scan to near-zero for most chunks.

## The expected payoff

O1 (emissive) + O2 + O3 together: the scans drop from ~131k match-chain
walks to ~surface-bands × 4096 indexed loads ≈ 5-10× fewer — the init
targets < 15-20 ms/chunk on the reference hardware. O4 cuts the emissive scan further.
The bench (the CI job's light-engine section, the glowstone-edit pump)
gates every change — no landing without a measured improvement and the
six gates green.

## The risk register

- The sky/blk semantics are the established reference semantics + the BFS —
  the differential test oracle (the skylight propagation deviation is
  documented) must stay green bit-for-bit; any constant change needs a
  live-verified citation.
- The column scan's `Chunk::get` fold is required per cell (states ≥ 256
  alias otherwise) — the table must fold the RAW state (the O3 design keeps
  this correct: the table is indexed BY the raw state).
- The mesher/mesh-jobs share the light arrays — the Arc COW pattern stays.
