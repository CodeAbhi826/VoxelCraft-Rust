# AGENT-STATE.md — handoff (PLAN-FINAL §0.A6: updated after every slice)

## Current
- Part: **1 PERFORMANCE CORE**. Current slice: **1.0.5 determinism coverage** (approved, in progress).
- HEAD: `4895596` (STOP in .gitignore). Branch: `test/full-sweep-2026-09-25`.
- Last known CI: run 37731681836 at `f0d346a` — 916 passed / 0 failed / 2 ignored.

## TODO — next 5 slices
- [ ] 1.0.5 per-function pinned `to_bits` tests for the 9 libm-site functions + widen golden hash (5 seeds x 25 chunks/dim + targeted village/stronghold/ravine/ocean pins), existing 3-OS job
- [ ] 1.1v E2E leg asserting sum(phases) >= 0.9 x frame_ms + VERDICT line + F3 phase-line screenshot (V1)
- [ ] 1.2v streaming benchmark as non-gating CI step under xvfb + game-binary artifact upload
- [ ] 1.3 work budgets first (no frame over 100 ms streaming; 30 fps steady reference hardware at defaults)
- [ ] 1.4 CPU world-gen speedups, IDENTICAL output (hash change = HARD STOP)

## TODO — queued (Part 1 remainder)
- [ ] 1.5 GPU timestamp queries (`--gpu-timing`) — 1.6 upscaling (§4) — 1.7 AA (§4) — 1.8 capability probe + knobs — 1.9 offload audit (report) — 1.10 re-measure matrix + CI perf gate — 1.11 turntable tool — 1.12 player torso/face mapping fix
- [ ] Interleaved: Chunk::get migration batches B–G (compile-verified)
- [ ] Part 1 REVIEW PACKET + adversarial self-review (10 claims)

## TODO — later parts (in order)
- [ ] Part 2 worlds-first (importer → oracle → arch → writer) — Part 3 gameplay core — Part 4 world-gen parity (4.0 LEGAL GATE hard stop first) — Part 5 colour/lighting/packs/shader — Part 6 art/models — Part 7 remaining gameplay — Part 8 quirk parity — Part 9 release

## Open blockers
- None. Soft blockers go to `docs/BLOCKERS.md` with exact ask + default; hard stops halt everything.

## Decisions in force (PLAN-FINAL §4, do not re-ask)
Vanilla mode AA off; FSR 1 only with Native/1.3/1.5/1.7/2.0/Custom; AA = SMAA-if-<=3ms-else-FXAA, forced on when scale<1; no GPU world-gen; original structure interiors; parity order biomes→positions→terrain→carvers; internal ids stay, display names from table; player skin layout mandatory; Round B parked to Part 5; small original entity roster first.

## Landmines
Chunk::get double-fold trap (~150 positional sites remain); greedy-key bitfield mirrored in WGSL (CPU `mesh.rs:615` / GPU `gpu_mesh.rs:385-386`); monoliths (game.rs 28k, blocks.rs 16k, mobs.rs 10k); CI quirks (clippy lib-only, match CI exactly; never pipe test output hiding exit code); reference hardware: no local release builds ever.

## Rules in force
L1–L8, V1, R1, R3–R7, P1. Autonomy A1–A7: one slice/commit (≤~300 lines), local `cargo check/test -p <crate>` only, commit→push→wait CI→green→worklog+state→next. No 1.0.x re-baselines without report. STOP file = halt clean.
