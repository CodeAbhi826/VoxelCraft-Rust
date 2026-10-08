# AGENT-STATE.md — handoff (PLAN-FINAL §0.A6: updated after every slice)

## Current
- Part: **2 WORLDS FIRST**. Current slice: **2.1e placeholder registry** (committed; CI watch armed).
- HEAD: 2.1e fix-forward commit (count pins + decode skip; 48/48 local).
- Last known CI: run 37731681836 at `f0d346a` — 916 passed / 0 failed / 2 ignored.

## TODO — next 5 slices
- [x] 1.0.5 per-function pinned `to_bits` tests + wide/targeted golden hashes (`30ad6a8`, CI 37743400037 green)
- [x] 1.1v E2E phase proof (`29de86b`+`1ad23a4`; linux-game 37747380181 green, V1 F3 screenshot viewed)
- [x] 1.2v CI streaming step (non-gating) + world-chunks in bench JSON (`871d28f`+2 fixes; linux-game 37754542709 green, artifact: avg 268ms/p99 808ms, 877 chunks)
- [x] 1.3 work budgets (`8b4011c`+2 fixes; avg 268→52ms lavapipe, meshes land, all legs green)
- [x] 1.4 CPU world-gen speedups, IDENTICAL output (`8e0fcd6`; 3-OS + E2E green, −4% inside noise)

## TODO — queued (Part 1 remainder)
- [x] 1.5 GPU timestamp queries (`89a935a`; scene GPU 8.9ms vs CPU 83ms lavapipe, gpu_ms in artifacts)
- [x] 1.6 upscaling (§4 ladder + Sharpness + Native skip; `6b237ad`+fix; T5 margin 513)
- [x] 1.7 FXAA (`d9f0425`; CI green, all legs green in run 37786754613; job red on quota only)
- [x] 1.8 capability probe + F3 split + gen knob (`a7f233e`; legs all exit 0, uploads red on quota)
- [x] 1.9 offload audit (`OFFLOAD-AUDIT.md`; CPU-bound by exact/gameplay classes)
- [x] 1.10 re-measure + gate + verdict (`linux-game.yml` gate; CPU-bound, worst_ms gated)
- [x] 1.11 turntable capture tool (`7f47f74`; leg green, 5 creeper views; V1 viewing waits quota)
- [x] 1.12 player torso/face fix (`333b766`; pixel test + all legs green; visual waits foreground)
- [x] 1.8b vanilla preset + fresh-profile tiering (`7006495`; all legs exit 0; particle knob pre-existed)
- [ ] Chunk migration B–G (save.rs, game.rs, gen.rs ranges; A done)
- [ ] Interleaved: Chunk::get migration batches B–G (compile-verified)
- [ ] Part 1 REVIEW PACKET + adversarial self-review (10 claims) + foreground capture sweep (intro→panorama→every settings page→world select/create→loading→loaded→HUD/inventories/containers + 1fps boot+play bursts; pixel-by-pixel review of EVERY image for unreported issues, V1 described per image) + full-dimension coverage (Overworld/Nether/End), all textures/tiles, and everything visible in a full playthrough as if completing the game

## TODO — later parts (in order)
- [ ] Part 2 worlds-first (importer → oracle → arch → writer) — Part 3 gameplay core — Part 4 world-gen parity (4.0 LEGAL GATE hard stop first) — Part 5 colour/lighting/packs/shader — Part 6 art/models — Part 7 remaining gameplay — Part 8 quirk parity — Part 9 release

## TODO — approved licensing L1–L8 (APPLY ONLY AFTER slice 1.12; do not reorder)
Decision: code + art-generator scripts GPL-3.0-or-later; original art + aggregate spec CC BY-SA 4.0; Monocraft SIL OFL 1.1 (unchanged, own folder); Voxelfont MIT; no custom GPLv3 §7 terms; no dual licensing; no permissive crates for now. No history rewrites. Gameplay/rendering behaviour must not change (metadata+docs except About screen). Incompatible dependency ⇒ stop, record in BLOCKERS.md.
- [x] L1 LICENSE file (L1a) + Cargo `license` fields (L1b) + LICENSES/ texts (L1c)
- [x] L2 REUSE (committed; `reuse lint` gates in L3b after docs-license call)
- [x] L3 CI job: cargo-deny `check licenses` (green; caught + resolved symphonia MPL-2.0); reuse-lint half → L3b (BLOCKERS #4)
- [x] L4 NOTICE/TRADEMARKS/CITATION (L4a) + README credits + third-party list + generated-art statement + CONTRIBUTING (L4b)
- [x] L5 About screen (gated in CI; V1 screenshot waits fresh binary post-quota)
- [x] L6 AUTHORSHIP.md (green)
- [x] L7 provider output-ownership finding (green; owner confirms pre-release → BLOCKERS #5)
- [x] L8 README/LEGAL docs off Apache-2.0 (green; history intact, R4 kept)
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
L1–L8, V1, R1, R3–R7, P1. Autonomy A1–A8 + continuation rule: one slice/commit (≤~300 lines), local `cargo check/test -p <crate>` only, commit→push→wait CI→green→worklog+state→next slice immediately; slice reports live in WORKLOG/AGENT-STATE, never as chat closings. No 1.0.x re-baselines without report. STOP file = halt clean. Repo is PRIVATE.
Hardware privacy: no owner specs in repo/docs/commits (commit `47acd82` scrubbed
the tree; pushed history untouched pending explicit rewrite order).
