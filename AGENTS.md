# VoxelCraft-Rust — Agent Guidelines & Rules

## 1. Hardware & Compilation Guardrails (STRICT)
- **Reference hardware**: low-end dual-core Linux test hardware (owner's machine — make/model/specs are private and must never appear in the repo, docs, commits, or logs).
- **NEVER run `cargo build --release` or compile release binaries locally on the user's host machine**. Local release compilation risks thermal throttling and host freezing.
- **No local compilation except quick test verification** (owner directive 2026-10-08): a single filtered `cargo test -p <crate> --lib <filter>` when a fast signal is genuinely needed. Everything heavy — full builds, full test suites, benchmarks, wasm targets, binaries — runs ONLY on GitHub Actions CI. Check the machine load (`/proc/loadavg`, free memory, thermal zones) before even a quick run; skip it if the machine is busy.
- **Local machine usage is strictly restricted to** (and only when the machine is idle):
  - a single filtered `cargo test -p <crate> --lib <filter>` for a fast signal
  - `cargo check -p <crate> --lib` only to catch syntax-level breakage pre-push
  - Fast single-file or non-release inspection commands (grep, logs, image viewing)
  - Git operations
- **All production executables, release packages, and standalone release binaries must be compiled and distributed via GitHub Actions CI**.

## 2. Clean-Room Legal Compliance
- **Zero Copied Assets**: Do NOT extract, copy, or bundle proprietary the reference game .jar assets, textures, sounds, or the original publisher code.
- **Procedural Art & Textures**: All 16x16 tiles, fonts, and particle textures must be synthesized procedurally at boot or generated cleanly via non-infringing math.
- **Independent Clean-Room Architecture**: Re-implement vanilla mechanics using clean-room specifications and open architectural comparisons with the established voxel engine (the upstream voxel platform).

## 3. Visual & Rendering Standards
- **Seamless Tiling**: Block faces must tile continuously without artificial border insets, clamping gaps, or "chocolate bar" seams.
- **Explicit Atlas Gradients**: Atlas UV gradients must be explicitly scaled to the 32x32 atlas dimensions (`dpdx(in.uv) / 32.0`) to avoid LOD explosion and coarse mipmap bleeding across tile seams.
- **the reference game 1.16.5 Fidelity**: Smooth lighting curve, ambient occlusion, water flow/transparency, 15x15 crosshair, and lower-right first-person 3D player arm with walk bobbing and attack swing animation.

## 4. UI & Menu Layout Parity
- **Creative Inventory**: 12 top/bottom category tabs, 3D player preview avatar tracking the cursor, working armor & offhand slots, 9x5 block grid with vertical scrollbar, live search filter box, 9-slot hotbar, and trash slot.
- **Video Settings**: Exact 19-option 1.16.5 layout with 2 top wide sliders, 2-column 16-option grid, and bottom Done button.
- **World Selection**: Darkened dirt background, top search box, scrollable world cards, and bottom action bar.
- **F3 Debug Screen**: Two-column layout with real-time FPS, coordinates, biome, chunk cache, and Targeted Block/Fluid properties.

## 5. Continuous Documentation, Worklog Sync & Clean-Room Verification
- **Continuous Worklog Maintenance**: Append detailed, factual work units to `docs/WORKLOG.md` in every session, documenting all architectural changes, bug fixes, test results, and benchmark metrics.
- **Parity Backlog Maintenance**: Keep `docs/PARITY-BACKLOG.md` synchronized with the verified clean-room parity status against the reference game 1.16.5.
- **Strict Clean-Room Verification & Source Citations**:
  - Every numerical constant, formula, tick timing, or gameplay mechanic added to code or documentation MUST cite an authoritative public source (e.g., live `the reference game.wiki`, vendor specifications like AMD GPUOpen, or open clean-room implementations like the established voxel engine).
  - Any values without an authoritative public source MUST be explicitly flagged as `[ESTIMATED / APPROXIMATION]` rather than asserted as fact.
  - **Zero Decompiled Code Transcription**: Strictly forbid copying, reproducing, or transcribing proprietary decompiled source code, variable names, or pseudocode into repository comments or documentation; express all mechanics purely as clean-room behavioral descriptions.
- **Repository Documentation Sync**: All documentation files (`docs/WORKLOG.md`, `docs/PARITY-BACKLOG.md`, `AGENTS.md`, and README) must be committed and pushed to GitHub alongside code changes.

## 6. Standing Session Rules (PLAN v3.1 — transcribed 2026-10-08 from the owner's plan briefs)

### Legal rules L1–L8 (numbering rests from the plan briefs; meanings are the documented repo rules)
- **L1 — Black-box only**: parity work against the reference game is built from public documentation plus our own oracle/behavior probes. No decompiled output, and no community reimplementation built from decompiled sources, is consulted for parity code. Propose the method before using it (Part 4.0 gate).
- **L2 — Read-side only**: the ecosystem's formats (packs, saves, NBT) are accepted as inputs for interop; we never emit or mirror another project's layout in our own shipped data. No reference set is committed, bundled, or distributed in any form.
- **L3 — Zero copied assets**: no byte of any rights-holder asset (texture, sound, model, font, panorama) enters the repo; all art/audio is synthesized procedurally in-repo. Verified per-pixel by `scripts/legal_audit.py`.
- **L4 — Zero decompiled transcription**: strict forbid on copying, reproducing, or transcribing proprietary decompiled source, variable names, or pseudocode into code, comments, or docs; mechanics are expressed purely as clean-room behavioral descriptions.
- **L5 — Study corpus = facts only**: owner-granted read-only reference sets are used only for fact-level counts and aggregate statistics (dimensions, palette histograms, banding/edge/symmetry/alpha statistics, animation metadata). No pixel positions, masks, silhouettes, waveforms, or code are stored or derived. This provenance statement must be kept visible in `LEGAL.md`/docs and never scrubbed.
- **L6 — Trademark & branding**: no third-party trademarks, names, logos, splash texts, folder conventions, or edition/update marketing labels; in-game functional vocabulary is generic terms-of-art (the 2026-09-21 owner directive in `docs/LEGAL-COMPLIANCE.md` §3).
- **L7 — Script enforcement**: committed scripts must not read or reach any reference set; `scripts/legal_audit.py` enforces the L2/L7 script guard on every run and in CI, with its self-test.
- **L8 — Honest claims**: Verified/Tested/Code-only/Unknown tags; every numerical constant, formula, timing, or mechanic cites an authoritative public source; unsourced values are flagged `[ESTIMATED / APPROXIMATION]`.

### Verification rule V1
- **V1**: any visual verdict must be based on actually viewing submitted screenshots — the verdict text describes exactly what was seen (never what was expected to be seen). Before trusting any visual verdict engine, describe test images supplied by the owner first.

### Process rules R1–R7
- **R1**: no history rewrites, force-pushes, or destructive git operations without the owner's explicit confirmation of that exact operation.
- **R2**: stop and report at the end of every Part; wait for "GO PART N" before starting the next.
- **R3**: estimates are velocity ranges (slices/week, CI rounds/slice) computed from repo history; no whole-game totals in weeks.
- **R4**: keep the honest provenance statements (the study corpus was used for fact-level counts and aggregate statistics) in `LEGAL.md` and docs; never scrub them.
- **R5 DETERMINISM**: world generation uses exact IEEE arithmetic on the CPU, in the numeric type the reference behaviour requires (f64 vs f32). No FMA contraction, no fast-math. World-gen math functions (sin, cos, pow, exp, log) are our own fixed implementations so results are identical on every OS and CPU. SIMD is allowed only if it keeps each value's exact operation order. The golden terrain hash must be identical on Linux, Windows and macOS CI runners.
- **R6 USER WORLDS**: never modify an original world; import works on a copy; automatic backups; unknown blocks, items, entities, tile entities and NBT are preserved verbatim through load and save; corrupt input never panics.
- **R7**: one slice per commit (max about 300 changed lines); keep L1–L8, V1 in force at all times.

### Privacy rule P1 (free-tier privacy, owner directive 2026-10-08)
- **P1**: the owner is on a data-collecting free model tier. Never ask the owner for reference captures, sample worlds, jars, or any reference material; the owner gives numeric results only. Never put secrets, tokens, or personal files in work (code, docs, logs, or commits).

### Continuation rule (owner directive 2026-10-08 — overrides R2's per-Part wait and §6 chat reporting)
- Do NOT end the turn after a slice. A slice report goes into `docs/WORKLOG.md` and `docs/AGENT-STATE.md`, not into a closing chat message.
- After CI is green and the state files are updated, immediately start the next slice from `docs/AGENT-STATE.md`. Never write "Next: ..." as a closing message; write it into AGENT-STATE.md and keep working.
- A turn ends ONLY when: a HARD STOP applies, a Part's REVIEW PACKET is written, a STOP file exists, or a rate/context limit is hit.
- On a HARD STOP, write "HARD STOP: \<reason\>" as the first line of `docs/BLOCKERS.md`.
- **Hardware privacy**: never record the owner's hardware make, model, specs, device IDs, driver versions, or distro in the repo, docs, commit messages, or CI-visible strings. Perf numbers stay; the machine identity goes ("reference hardware" / "reference iGPU").
- **Worklog convention**: append every slice to `docs/WORKLOG.md`; update `docs/AGENT-STATE.md` (todo tracker) in the same flow; write `docs/CHECKPOINTS.md` + REVIEW PACKET at each Part end. Root `worklog.md` is the prior era's closed log (Tasks 1–25) — read-only, never append.
- **In-game review rule**: every slice that touches gameplay/rendering gets its own linux-game E2E run at its commit; every Part review ends with a full in-game tour (all legs re-run on the final binary, fresh captures viewed under V1). Brief verification after each major Part — never pile untested work into a later review.

