# AGENT-STATE.md — handoff (PLAN-FINAL §0.A6: updated after every slice)

## Current (2026-10-17, branch test/full-sweep-2026-09-25 — Part 6)
- Part 6 landed slices: z187–z199 display transforms / atlas-grid consts / shader grid templates / tile_uv + arrow-grid bug fix / model-driven icons / E2E_DISPLAY leg / skin outer layer (all CI-green).
- Part 6 §6.2b owner-verdict art redesigns LANDED: z200 TNT (palette+pattern), z201 enderman (eyes/head/body pattern), z202 ghast (silhouette/face/palette), z204 default player skin (palette) — each with unit tests, `cargo fmt --check`, clippy, tile captures agent-viewed (before/after real painted pixels).
- z203 leaves de-noise (structured clumps; owner "per-pixel noise" fix) — landed with measured before/after numbers.
- z205 verification: crosshair colour = vanilla difference blend by design (owner question queued on pinning a colour); HUD heart/food icons confirmed procedural (gui_art painters + tests; built-in set never touches disk; the override API is test-only).
- z208 in-game tour (dispatched linux-game at 68bd316, run 38960239643 — all legs green, no art regression). Agent V1 from the artifacts: leaves de-noise CONFIRMED in-game (structured canopy, cut-outs); enderman CONFIRMED (884 magenta eye pixels in a compact head region). Still PENDING in-game: ghast (survey does not frame it — known gap) and the player-skin palette (the FKEYS leg runs E2E_SKIN=1, so it never shows the default skin). Both are owed at the checkpoint tour (`TURNTABLE_MOB=ghast|enderman` dispatch + a no-skin third-person capture).
- z209 §6.1 item-frame wall-plate render form (`0615352`, CI 38994123411 + WASM 38994123419 both green): the frame is no longer a cross sprite — ITEM_FRAME joined PROP_BLOCKS with a 4-value facing window (states 940..=943, STATE_COUNT 940→944), placement attaches it to the clicked side face, the builtin pack ships `blockstates/item_frame.json` + `models/block/item_frame.json` (12×12×1 plate), and `procedural_tile_for()` lets the model address the painted frame tile instead of a duplicate PNG. Legacy state 290 kept as the pre-facing alias (folds to the block, off the model path). **No in-game V1 yet** — the plate's in-world look is unverified and is owed at the checkpoint tour.
- z210 §6.1 item-frame contents (store + rotate + pop + draw; code, tests+clippy green, not yet pushed/verified in-game): the frame's one item now rides the phase-3 container store (`slot_count(ITEM_FRAME) = 1`, so it is saved with `level.dat` and spilled on break), `ContainerInv.rot` records the 45° display step (persisted as `Rot`), right-click places (empty frame) or rotates (filled frame), a punch pops the item out as a world drop, and the item draws as a flat quad on the frame's plate through the same blended vertex stream the item entities use (`push_frame_items` / `frame_item_quad`, geometry pinned by `frame_item_quad_hangs_on_the_plate_per_facing`). **No in-game V1 yet** — owed at the checkpoint tour with z209's plate.
- z211 §6.1 item-frame in-game proof (E2E_FRAMES leg, wired into `linux-game.yml` as a gating step + artifact): pins a cardinal camera, builds a stone wall + three frames through the real state path, captures empty → filled → 45°-rotated, and exits on two mean-abs diffs (item appears, rotation moves pixels). Dispatch + capture V1 pending this push.
- Remaining Part 6: §6.1 armor stand entity (absent — needs a new item id + entity + render; planned as two slices: id/tile/registration, then entity + equipment + poses), §6.2 actual hi-res tile growth (needs real-GPU VRAM numbers; groundwork done), then the Part-6 REVIEW PACKET + full in-game tour, then Part 7.
- Landmines (new): ALWAYS run `cargo fmt --check` before committing (z200 went red on it); on this branch the push-triggered gates are `ci.yml` + `Build WASM` — wait for BOTH to finish before the next push (concurrency cancels in-flight branch runs; a docs-only push may trigger neither). `linux-game.yml` (the real-GPU E2E legs) triggers on `main` only, so per-commit E2E coverage needs a `workflow_dispatch` at the next Part checkpoint (never alongside a push run). The repo-root `builtin-pack/` is a **gitignored first-run extraction** — new pack assets go in `voxelcraft/builtin-pack/`. Growing `STATE_COUNT` also requires re-deriving the **WGSL LUT offsets** in `vc-render/src/gpu_mesh.rs` (`wgsl_lut_offsets_match_rust` drift test).
- (Note: older sections below describe the pre-rendering-stage tree; the branch has moved on.)

## Current (pre-rendering-stage, kept for history)
- Part: **2 CLOSED** (packet z60, 2026-10-09) — all slices green; only owner-blocked items remain.
- Owner directive 2026-10-09: when the plan is over, ship a downloadable build for testing on the owner's PC (local verification pass after CI).
- HEAD: survey V1 done (48 framed / 17 unknowns logged); ui-snaps slice staged (uncommitted).
- Repo is PUBLIC since 2026-10-09 (owner-ordered). Default branch still stale `main`.
- Last known CI: run 37907738410 (linux-game) — FULL SUCCESS incl. SURVEY VERDICT OK.
- V1 tally: ~110 cells/images. Claim 5 RE-INSTATED (z42). Part-1 packet written (z56).

## TODO — now (in order)
- [x] About capture: dumper-guard fix CI-green; real About PNG V1-PASSED (2560×1440, title + 8 lines + DONE)
- [ ] Turntable re-verdict: feet-level camera fix committed; verify on next linux-game, V1-view all 5, re-instate or rewrite claim 5
- [ ] Survey leg: stage-reuse optimization (this slice) → full 66 fronts → V1-view batch
- [ ] Part 1 remaining screens: settings/title/inventory/player-eye/F3-gpu to 30+ viewed, then packet close-out
- [ ] 2.2c codec fidelity (committed `3aad7f6`; validates on next green CI)
- [ ] ts-race fix slice (design in /tmp/opencode/ts-race-design.md; needs owner build call for Intel validation)
- [ ] E2E turntable gate hardening (assert mob-in-frame, not just files-saved)

## TODO — Part 2 queue status
- [x] 2.1a version gate, 2.1b block sidecar, 2.1c record sidecars, 2.1d fuzz, 2.1e registry/tile/art, 2.1f remap (all CI-green)
- [x] 2.2a oracle primitives, 2.2b structure starts (CI-green)
- [x] 2.3a tick order + tickets (CI-green)
- [x] 2.4a writer re-emit, 2.4b region backups (CI-green via rerun 37869400104)
- [x] 2.2c codec round-trip (test green in CI 37925048470)
- [ ] 2.3b ticket wiring (staged, in CI), 2.4 writer verification in reference game (soft-blocked)

## TODO — morning batch (owner calls)
- [ ] Branch layout decision (rename → dev? default branch still stale `main` on now-public repo)
- [ ] Docs-license call (BLOCKERS #4 — reuse-lint waits on it)
- [ ] Provider-terms confirm (BLOCKERS #5, pre-release)
- [ ] Phases-gate call (BLOCKERS #3 — 0.9 vs 0.88 local)
- [ ] package.json license scope question
- [ ] gmail in commit history: leave or explicit rewrite order (R1)

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
- [x] Chunk migration B–G (complete workspace-wide; only trap-pinning tests/internals remain)
- [x] Part 1 REVIEW PACKET + adversarial self-review (10 claims; claim 5 WITHDRAWN after V1 viewing — see re-verdict item above)

## TODO — next 5 slices (Part 6 remainder → Part 7)
- [x] §6.1 item-frame render form slice 1 (`0615352`, z209 — facing window 940..=943, placement attaches to the clicked side face, thin-plate builtin model, procedural-tile-by-location).
- [x] §6.1 item-frame contents slice 2 (z210 — 1-slot container store + `rot` persisted in `level.dat`, right-click place/rotate, punch pops the item out, break spills it, and the item draws as a flat quad on the plate through the shared blended vertex stream). Slices 2 and 3 collapsed: break/drop/persistence came free from the container store. Remaining: in-game V1 of both frame slices at the checkpoint tour.
- [ ] §6.1 armor stand entity (model + pose rules + tests) — absent; needs a new entity (mobs.rs/vc-sim) plus render.
- [ ] §6.2 hi-res tile growth (owner-ordered): parameterise TILE_PX beyond 16 (atlas 512→1024², mip regen, icon cells, skin stow bounds), then measure real-GPU VRAM/frametime deltas on the reference iGPU (local debug runs are allowed; no release builds).
- [ ] Part 6 REVIEW PACKET (docs/CHECKPOINTS.md, ≤400 words) + adversarial 10-claim self-review, then Part 7 (remaining gameplay: redstone gaps, rails/minecarts/boats, per-shape collision, pathfinding, villager schedules, raids, beacon, fishing, spectator/crawl, missing GUIs, block long tail, data-driven content, audio/lang/menus).
- [ ] Owner question queue (silent): POM/PBR pack status, flicker/z-fight specifics, UI/HUD specifics, frametime scope, cinematic discovery, night brightness, matrix content V1 (at full-plan end), crosshair colour (parity vs pinned), nether-portal "frame" reading, §6.2 hi-res tile target size, and whether a frame should keep showing a block FACE or get a real 2-D item sprite.

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

- Owner directive 2026-10-09: 4.0 legal-gate proposal POSTPONED — start Part 4 at 4.1 (biomes) instead. L1 black-box rules stay in force throughout; the 4.0 proposal is deferred debt, not a waiver.

- 2026-10-10: Part 3 progress — recipe book (3.4a-c) + chat (3.5a-b) CI-green. Next: 3.6a command parser + simple commands.

- 2026-10-10: PART 3 CLOSED (packet z20). 20 commands, chat, recipe book, scoreboard/teams/bossbar, locate. Tour run 38021703354 dispatched. Next: Part 4 starting at 4.0 (owner reinstated 4.0 over 4.1) — write the 4.0 black-box proposal after tour verdict.

- 2026-10-10: PART 4 at 4.0 gate — proposal written (docs/PART4-PROPOSAL.md). AWAITING OWNER APPROVAL; no Part-4 code until approved.

- 2026-10-10: Part 4 status — 4.1 probes/large/Nether done (tuning awaits owner diffs), 4.2 spread/audit/well/hut done, 4.3 amplified/seam/height probes done. Next: 4.4a vein-seam fix (needs main-hash re-pin approval).

- 2026-10-10: PART 4 CODE CLOSED (packet z43, tour 38036732269 running). Tuning targets open pending owner diffs. Next: Part 5 colour/lighting/packs (doc-driven slices first).

- 2026-10-10: 4.1k in CI (badlands-restore + mushroom targeting + window widening); golden re-pins expected from CI log values only. Next: re-pin batch, then height-fit round.

- 2026-10-10: 4.1k re-pin slice in CI (5 hashes from CI logs only + 2 test-only flake fixes). Next: height-fit round on the owner seed.

- 2026-10-10: 4.1l ratio slice in CI (temperate/ocean pick-splits; re-pins expected). Next: mountain/river measurement slice when machine idle, then height means.

- 2026-10-10: 4.1m re-pin slice in CI (9 hashes from CI logs + 3 test-only hardenings). Next: remaining targeted pins if any, then mountain/river measurement slice.

- 2026-10-10: village-targeted re-pin in CI. Next: ravine/ocean pins if moved, then mountain/river measurement.

- 2026-10-10 owner batch (all recorded): statistical parity YES (Part 4 close-out target); Part 5 verified by agent from docs + own V1, owner reviews only at full-plan end (no interim review asks); docs/prose GPL-3.0-or-later + art CC-BY-SA-4.0 (BLOCKERS #4 closed, L3b queued); provider terms still OPEN in plain words (BLOCKERS #5); phases gate keep (BLOCKERS #3 closed); real-GPU AA measurement authorized on this system (BLOCKERS #1); branch/history left as-is; reference-game + Intel checks deferred (owner won't run).

- 2026-10-10: z59 shell-removal + L3b in CI (reuse 4386/4386 local). Next: mountain/river measurement when idle, then height means.

- 2026-10-10: z60 fit-round-3 in CI (re-pins expected). Next: re-pin batch, then relief-histogram slice for beach/deep/plains.

- 2026-10-10: z61 overlay-roll fix + re-pins in CI. Next: remaining targeted pins, then verification census.

- 2026-10-10 owner comms directive: ask NOTHING unprompted — queue all questions silently and keep working; owner asks "anything?" when they want the queue. No stopping for anything.

- 2026-10-10 owner directive: Part 4 done goes straight to Part 5, no asking. Ask nothing unprompted, queue silently.
- 2026-10-10: z62 patch-overlay + sunflower harden in CI. Next: full re-pin harvest, then relief-histogram slice.

- 2026-10-10: z63 re-pins in CI. Next: ravine/ocean pins, then relief-histogram slice.

- 2026-10-10: z64 ravine pin in CI. Next: ocean pin, then relief-histogram slice.

- 2026-10-11: z65 relief gates in CI. Next: re-pin harvest + verification census.

- 2026-10-11: z66 re-pins in CI. Next: ravine/ocean pins if drifted, then verification census.

- 2026-10-11: z67 ravine pin in CI. Next: ocean pin, then verification census.

- 2026-10-11: z68 ocean pin in CI (chain complete if green). Next: verification census.

- 2026-10-11: z69 shelf/hills rebalance in CI. Next: re-pin harvest + verification census.

- 2026-10-11: z70 hardenings + harvest in CI. Next: ravine/ocean pins, then verification census.

- 2026-10-11: z71 ravine pin in CI. Next: ocean pin, then verification census.

- 2026-10-11: z72 ocean pin in CI (chain complete if green). Next: verification census.

- 2026-10-11: z73 hill gates in CI. Next: re-pin harvest, verification census, river-carve slice.

- 2026-10-11: z74 hill-gate pins in CI. Next: verification census + river carve.

- 2026-10-11: z75 giant-hills gate in CI. Next: pins, census, river-carve slice.

- 2026-10-11: z76 q2 pins in CI. Next: verification census + river carve.

- 2026-10-11: z77 river precedence in CI. Next: harvest + verification census.

- 2026-10-11: z78 river harvest in CI. Next: ocean pin + verification census.

- 2026-10-11: z79 typo fix in CI. Next: verification census grades river fix.

- 2026-10-11: z83 fixture seed in CI+tour. Next: tour verdict, then Part 4 close + Part 5.

- 2026-10-11: z84 drain-gate + seed95 in CI+tour. Next: tour verdict, Part 4 close, Part 5.

- 2026-10-11: PART 4 CLOSED (z88, tour green). Next: Part 5 colour/lighting/packs, doc-driven slices first.

- 2026-10-11: z92 tint fix in CI. Next: color rows 12/15, then packs.

- 2026-10-11: color audit closed 15/15. Next: Part 5 packs (builtin-pack content + pack overlays).

- 2026-10-11: z96 colormap lib in CI. Next: renderer wiring slice.

- 2026-10-11: z98 colormap wiring in CI. Next: in-game colormap proof or next Part 5 item.

- 2026-10-11: z100 moon phases in CI. Next: in-game moon proof (batched tour) or next Part 5 item.

- 2026-10-11: built-in shader UN-PARKED w/ corrected scope (owner: optional FEATURE shader with bloom/POM/PBR/etc, not a vanilla clone). Binding: default vanilla path frozen identical to real game; feature shader OFF by default; Iris support retained; vanilla-style options UI. Remaining Part 5: feature-shader toggles, 2B block-texture contract, shadow staging epic, proofs batch, AO source.
- 2026-10-11: tour batching policy (owner): no tour-per-slice; batch CI-green slices, tour at batch boundaries or when a slice needs in-game proof.
- 2026-10-11: z131 2B-remainder 5.1n table CI-green. Next: graceful-errors slice (malformed pack inputs never panic) or Part 5 packet scoping.
- 2026-10-11: z132 5.1o compat matrix CI-green. Next: mcmeta-animation + pack_format-6 warning audit.
- 2026-10-11: z133 5.1p animation/format CI-green. Next: built-in-shader toggle inventory (5.1q).
- 2026-10-11: z134 closed green; z135 5.1r lighting table in CI.
- 2026-10-11: z136 interpolate closed green (fmt lesson logged). Next: entity_shadows GPU wiring.
- 2026-10-12: z139 interp-proof stage closed green. Next: batch-tour V1 (interp + shadow texel).
- 2026-10-12: z140 batch tour green, both V1s closed. Next: Part 5 REVIEW PACKET (A5) then Part 6.
- 2026-10-12: PART 5 CLOSED (packet, CI pending). Next: Part 6 first slice (crosshair dims measure).
- 2026-10-12: Part 6 opens (crosshair slice in CI). Next: close z142, then item/block render-forms audit.
- 2026-10-12: z142 crosshair closed green. Next: Part 6 render-forms matrix.
- 2026-10-12: z143 6.1 matrix closed green. Next: display-transform parsing (model.rs).
- 2026-10-12: z144 display-parse closed green, locally verified too. Next: item-model resolution slice.
- 2026-10-12: z145 item-model entry closed green. Next: AO public source.
- 2026-10-12: z146 AO closed (algorithm VERIFIED via 0fps, curve ESTIMATED). Next: font providers.
- 2026-10-12: z147 font parse+rasterize closed green after fix (fixture widths). Next: legacy rasterize.
- 2026-10-12: z148 legacy_unicode closed green. Next: font ASCII wiring.
- 2026-10-12: z149 font wiring closed green after fix (early-return gate + clippy). Next: sounds proof.
- 2026-10-12: z150 sounds headless proof closed green. Next: colour harness.
- 2026-10-12: z151 colordiff closed green. Owner order: COMPLETE Part 5 now, then Part 6 (Part 6 pauses). Firecrawl adopted for research (quota-limited; wiki fallback).
- 2026-10-12: z152 hi-res epic (6.2) + gap queue (6.3) in CI. Part 5 remaining: skins epic, toggle-costs (procedure done), SSR/SSAO/volumetric. Part 6 queue: hi-res slices, display application.
- 2026-10-12: z155 sampler closed green. Next: skins slice 4 (atlas stow + FaceTex rewrite).
- 2026-10-12: z161 skins V1 closed (agent). Part 5 remaining: SSR/SSAO/volumetric epics. Next: volumetric slice (cheapest first).
- 2026-10-12: z164 SSAO fix closed green. Next: SSAO kernel slice.
- 2026-10-12: z165 SSAO kernel closed green. Tour dispatched for ssaodebug+ssao V1.
- 2026-10-12: z168 clear-probe closed green. Discriminator tour dispatched (mid-gray => scene never wrote; white => sampling path).
- 2026-10-12: z169 0.99-probe closed green. Discriminator tour dispatched.
- 2026-10-12: z170 linear-reachability fix closed green. Validation tour dispatched (expect near-black ground = reconstruction proven on GPU).
- 2026-10-13: z171 translation-proof closed green. Bytes-vs-multiply tour dispatched.
- 2026-10-13: z172 constant-multiply band closed green. Multiply-vs-input tour dispatched.
- 2026-10-13: z173 analytic closed green. Tour dispatched (builds z173 binary for local full sweep).\n- 2026-10-13: z175 timestamps closed green. Verification via local phases run (CI cannot reach path). Owner hardware report triaged; local debug build started; real-GPU gate now primary.
- 2026-10-13: z174/z176 closed green; analytic SSAO proven on real GPU (linear ramp correct); brightness parity fix (0.5) verified day-neutral locally. Next: matrix leg, water leg, damage/HUD.
- 2026-10-13: matrix part 2 closed; readiness slice verified (62/62 x4). Volumetric exonerated by sun geometry; sun-facing rows next. Rendering-stage gate: SSAO closed, kernel measured, modes proven, MSAA fixed, brightness fixed, phases fixed, fkeys fixed.
