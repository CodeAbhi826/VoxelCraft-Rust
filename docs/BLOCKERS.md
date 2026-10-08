# BLOCKERS.md — soft blockers (need owner input/checks) and hard stops

(A3: each entry carries the exact ask + the default the work continues under.
No hard stops open. The STOP file is absent.)

## Soft blocker 2 — Part 1 foreground capture sweep (2026-10-09)
- Ask: EITHER (a) wait for the Actions artifact-quota recalculation
  (6–12h; quota-probe loop armed, reruns failed CI legs hourly) so the
  CI-built binary + turntable/F3 PNGs become downloadable, OR (b) grant
  a one-time exception for a local debug build (`cargo build -p
  voxelcraft`, NOT release) run when the machine is idle (load < ~1.5),
  under `xvfb-run` + MangoHud on this machine.
- Why blocked: no runnable binary exists anywhere (tree has none;
  `voxelcraft/target` was cleared; CI uploads fail on quota so no
  CI-built binary can be downloaded). Local full builds are otherwise
  prohibited by the compilation guardrail, and current load (~2.5 on
  the reference dual-core) makes a build unsafe right now. Display side
  is ready (`xvfb-run`, `Xvfb`, `mangohud` all present).
- Default (work continues under it): licensing L1–L8 then Part 2
  proceed through CI; the sweep executes the moment a binary is
  obtainable, before the Part 2 review packet. No visual claim beyond
  the one viewed F3 capture is made until then.

## Soft blocker 1 — SMAA-vs-FXAA 3 ms call (slice 1.7, PLAN-FINAL §4)
- Ask: on the reference hardware at 720p, read the FXAA pass cost (as built in 1.7) and
  decide: keep FXAA, or implement SMAA (3-pass + LUTs) for comparison.
  Anything I can measure here (lavapipe) does not transfer to the reference hardware,
  and 1.5's gpu_ms needs a real GPU to mean anything.
- How to measure: download any recent `voxelcraft-linux-single-file`
  artifact, run `./voxelcraft-*-linux-x64 --benchmark streaming
  frames=120 warmup=30 json=bench.json --gpu-timing` on the reference hardware, then
  compare with AA off vs on (engine page ANTI-ALIAS). Or read the in-game
  F3 phase row + gpu_ms from 1.8's split.
- Default (work continues under it): FXAA ships as the ONE method; no
  MSAA, no TAA; AA forced on pre-upscale below 1.0 render scale. SMAA
  lands only on your explicit ask with your numbers attached.
