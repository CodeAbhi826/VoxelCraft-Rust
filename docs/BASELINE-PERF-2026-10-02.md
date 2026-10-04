# BASELINE-PERF-2026-10-02.md — the measured performance baseline (Phase 1)

**Purpose:** the before/after oracle for every Phase-4 optimization. Committed
per the plan's Phase 1: date, commit, hardware, and the measurement method.

- **Commit:** `9a8aada` (the bench with the sim-tick + light-engine sections;
  the colorimetry fix lineage `bb2cb43..9a8aada` is performance-neutral)
- **Method:** `vc_bench chunks=96` (seed 0xC0FFEE) — the headless CPU
  pipeline: generation, meshing (incl. light BFS), partial remesh, the 20 Hz
  sim tick, the real light engine (init + a glowstone-edit pump), drawprep,
  draw-call accounting, paletted-section memory. The artifact route: built on
  GitHub Actions (no local compiling), the binary downloaded and run locally.
- **Tiers:** the reference hardware iGPU machine (2 rayon threads) and the GitHub
  ubuntu-latest runner (4 threads — the "mid desktop" proxy). The wasm tier:
  **UNAVAILABLE** (no wasm bench harness exists — disclosed, not a failure).

## The baseline table (96 chunks, seed 0xC0FFEE)

| Metric | reference hardware iGPU (2 threads) | GH runner (4 threads) |
|---|---|---|
| generation avg / p95 (ms/chunk, 1-thread) | 53.1 / 99.0 | 11.0 / 12.7 |
| generation parallel total (96 chunks) | 3722 ms (1.4× speedup) | 396 ms |
| meshing avg / p95 (ms/chunk, 1-thread, incl. light BFS) | 49.1 / 80.1 | 10.6 / 11.6 |
| meshing parallel total (58 chunks) | 12 410 ms | 1763 ms |
| partial remesh (3-section edit job) | 10.95 ms | 2.48 ms |
| **sim tick avg / p95 (ms, 20 Hz, 96 chunks)** | **0.041 / 0.051** | **0.015 / 0.022** |
| **light engine init (96 chunks)** | **8313 ms — 86.6 ms/chunk** | **1500 ms — 15.6 ms/chunk** |
| light engine pump (1 glowstone edit) | 12.4 ms (3559 nodes) | 1.1 ms |
| drawprep (µs/frame) | 13.9 | 2.60 |
| draw calls per frame (legacy → loop → MDI) | 153/459 → 153/27 → 12/27 binds | same |
| paletted-section memory | 10.3 KiB/chunk | 10.2 KiB/chunk |
| geometry | 212 932 verts / 106 466 tris | same |

## POST-PHASE-4 MEASUREMENT (2026-10-05: O2+O3 landed — the measured payoff)

The reference hardware tier (the CI-built vc_bench binary run locally, chunks=96, the
same seed): the light init 86.6 → **58.7 ms/chunk (32% FASTER)**, the
meshing 49.1 → 36.07 (27%), the generation 53.1 → 35.0 (34%), the remesh
10.95 → 8.68 (21%), the drawprep 13.9 → 8.7 µs (37%), the sim tick
0.041 → 0.035. The GH-runner tier: the light init 1500 → 1750 ms (the
runner variance ±20% — the reference hardware's before/after is the honest pair).
The Phase-4 work: O2 (the sky scan's heightmap break — the semantics
identical) + O3 (the static per-state lookup tables built from the SAME
truth — Chunk::get's fold + the emissive scan became indexed loads; the
gen/mesh paths speed up through Chunk::get too). The remaining light-init
cost: the emissive scan's per-cell get_state + the BFS + the border
exchange (O4's per-section emissive bitmap is the next step, not landed).

## What the numbers say (the optimization ranking, Phase 4 input)

1. **The light engine's INIT is the #1 cost** — 86.6 ms/chunk on the reference hardware
   (8.3 s for 96 chunks), 15.6 ms/chunk on the runner. It is slower than
   meshing and dominates chunk loading. The in-game load path pays it per
   chunk (init_chunk) — the startup spike and the loading hitches live here.
2. **Meshing is #2** — 49.1 ms/chunk (reference hardware) / 10.6 (runner), already
   parallel; the greedy mesher + skylight + block-light BFS per chunk.
3. **Generation is #3** — 53.1 ms/chunk (reference hardware), parallel speedup only 1.4×
   on the reference hardware (2 threads, thermally throttled) vs 4× on the runner — the
   worker scheduling/queue is fine; the per-chunk cost is the noise stack.
4. **The sim tick and drawprep are NON-TARGETS** — 0.041 ms/tick and 13.9
   µs/frame are noise-level; leave them alone.
5. **Memory is healthy** — 10.3 KiB/chunk paletted; a 289-chunk view
   (render distance 8) ≈ 3 MB of sections. Non-target.

## Targets (the plan's starting proposal — applied tentatively, owner confirms at the final report)

1. **Stable 30 fps on the reference hardware at modest render distance** (frame budget
   33 ms): the sim (0.04 ms) + drawprep (14 µs) fit trivially; the budget is
   the streaming pipeline — gen 53 + light 87 + mesh 49 ≈ 190 ms per NEW
   chunk single-threaded must be amortized across frames and threads so no
   frame carries it whole.
2. **60+ fps on mid hardware**: the runner-tier numbers amortize fine
   (10.6–15.6 ms/chunk across 4 threads).
3. **No hitches over 50 ms during chunk loading**: today a cold chunk costs
   ~190 ms of pipeline work (reference hardware) — the light-init cost must drop or be
   budgeted per frame (the pump's budget parameter exists; the init path is
   the spike).
4. **Memory ceiling per render distance**: < 20 KiB/chunk (today 10.3 —
   2× headroom before any compaction work is warranted).
