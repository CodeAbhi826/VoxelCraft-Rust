# AGENT-STATE.md — handoff (PLAN-FINAL §0.A6: updated after every slice)

## Current
- Part: **1 PERFORMANCE CORE**. Current slice: **1.1v E2E phase proof** (next).
- HEAD: `30ad6a8` (1.0.5 done, CI 37743400037 green 933/0/2). Branch: `test/full-sweep-2026-09-25`.
- Last known CI: run 37731681836 at `f0d346a` — 916 passed / 0 failed / 2 ignored.

## TODO — next 5 slices
- [x] 1.0.5 per-function pinned `to_bits` tests + wide/targeted golden hashes (`30ad6a8`, CI 37743400037 green)
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

## TODO — approved licensing L1–L8 (APPLY ONLY AFTER slice 1.12; do not reorder)
Decision: code + art-generator scripts GPL-3.0-or-later; original art + aggregate spec CC BY-SA 4.0; Monocraft SIL OFL 1.1 (unchanged, own folder); Voxelfont MIT; no custom GPLv3 §7 terms; no dual licensing; no permissive crates for now. No history rewrites. Gameplay/rendering behaviour must not change (metadata+docs except About screen). Incompatible dependency ⇒ stop, record in BLOCKERS.md.
- [ ] L1 LICENSE (full GPL-3.0 text) + LICENSES/ texts + `license` fields in every Cargo.toml
- [ ] L2 REUSE: REUSE.toml bulk annotations + "Copyright (c) 2026 CodeAbhi826 and contributors" + SPDX identifiers (headers in new files only)
- [ ] L3 CI job: `reuse lint` + `cargo deny check licenses` (fail on GPL-3.0-or-later-incompatible dep)
- [ ] L4 NOTICE, TRADEMARKS.md, README credits, CITATION.cff, third-party list, generated-art licence statement (script+spec+provenance), CONTRIBUTING (GPL-3.0-or-later + DCO)
- [ ] L5 About screen (credit, warranty, source link, not-affiliated, third-party list; V1 screenshot)
- [ ] L6 AUTHORSHIP.md (human role + AI agents, honest, no overclaim)
- [ ] L7 provider output-ownership terms → finding in LEGAL.md
- [ ] L8 README/LEGAL docs off Apache-2.0 (keep every provenance statement, R4) + worklog entry

## Open blockers
- None. Soft blockers go to `docs/BLOCKERS.md` with exact ask + default; hard stops halt everything.

## Decisions in force (PLAN-FINAL §4, do not re-ask)
Vanilla mode AA off; FSR 1 only with Native/1.3/1.5/1.7/2.0/Custom; AA = SMAA-if-<=3ms-else-FXAA, forced on when scale<1; no GPU world-gen; original structure interiors; parity order biomes→positions→terrain→carvers; internal ids stay, display names from table; player skin layout mandatory; Round B parked to Part 5; small original entity roster first.

## Landmines
Chunk::get double-fold trap (~150 positional sites remain); greedy-key bitfield mirrored in WGSL (CPU `mesh.rs:615` / GPU `gpu_mesh.rs:385-386`); monoliths (game.rs 28k, blocks.rs 16k, mobs.rs 10k); CI quirks (clippy lib-only, match CI exactly; never pipe test output hiding exit code); reference hardware: no local release builds ever.

## Rules in force
L1–L8, V1, R1, R3–R7, P1. Autonomy A1–A7: one slice/commit (≤~300 lines), local `cargo check/test -p <crate>` only, commit→push→wait CI→green→worklog+state→next. No 1.0.x re-baselines without report. STOP file = halt clean.
