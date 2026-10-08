# BLOCKERS.md — soft blockers (need owner input/checks) and hard stops

(A3: each entry carries the exact ask + the default the work continues under.
No hard stops open. The STOP file is absent.)

## Soft blocker 3 — phases 0.9 gate fails on the reference machine (2026-10-09)
- Ask: E2E_PHASES min_ratio scored 0.879 then 0.892 on the reference
  machine (old CI binary, real GPU, loaded dual-core) vs 0.983–0.988 on
  CI lavapipe. Decide: (a) keep the 0.9 CI gate and accept a lower
  local number as environmental, (b) recalibrate the gate
  machine-relative, or (c) investigate the ~11% outside
  begin/end_frame (present/acquire + event pump suspects).
- Default (work continues under it): CI gate unchanged (green there);
  no E2E threshold touched in code until your call.

## Soft blocker 2 — Part 1 foreground capture sweep (2026-10-09, partial)
- DONE locally: F3 pair, MENU tree, FKEYS views (8 images viewed, V1 in
  WORKLOG 2026-10-09d), beds/fluids/containers green. Quota prune 104→44.
- STILL OPEN: (a) turntable 5 views (needs fresh CI binary post-quota —
  Oct-8 binary predates 1.11); (b) settings/title/inventory screens —
  X capture black + xdotool undelivered (no WM), engine dumps don't
  exist for menu screens (follow-up slice: dumps in E2E_MENU);
  (c) face-eye symmetry + arm-design checks need close-ups.
- Default: licensing → Part 2 through CI; sweep remainder runs when a
  fresh binary is downloadable.

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
