# BLOCKERS.md — soft blockers (need owner input/checks) and hard stops

HARD STOP: GitHub Actions billing block (2026-10-09) — every CI job
fails unstarted in 2–5s: "recent account payments have failed or your
spending limit needs to be increased". No CI validation possible until
the owner clears billing. Letter of A4(f) is recorded here; work
continues ONLY on CI-independent tracks (local checks, design,
foreground runs with the existing binary) per the sleep-shift order —
no slice is marked done without CI green, and commits queue for
validation when runners return.

(A3: each entry carries the exact ask + the default the work continues under.
HARD STOP OPEN (see top): CI billing block. The STOP file is absent.)

## Soft blocker 5 — provider output-ownership terms (L7, owner confirm)
- Ask: confirm your model-provider terms permit shipping
  model-assisted output under GPL-3.0-or-later (the LEGAL.md L7
  finding records our DCO + GPL-entry side; provider-side ownership
  is not asserted). If they do not, say so and release re-scopes.
- Default (work continues under it): DCO + GPL-on-entry + AUTHORSHIP
  honesty stands; Part 9 re-verifies before release.

## Soft blocker 4 — docs license undecided (reuse-lint half of L3)
- Ask: pick ONE license for repo docs/prose (`docs/**`, root `*.md`
  except README claims handled in L8, workflow/config text): e.g.
  CC-BY-SA-4.0 (matches art/spec) or GPL-3.0-or-later (matches code).
  Until picked, `reuse lint` cannot gate (it fails on unannotated
  prose) and no docs license is asserted anywhere.
- Default (work continues under it): L3 ships the cargo-deny
  `check licenses` gate now; `reuse lint` gates in L3b right after the
  docs license lands (with L8). REUSE.toml already covers everything
  decided (code/art/fonts).

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
