# VoxelCraft-Rust — Agent Guidelines & Rules

## 1. Machine & Compilation Guardrails (STRICT)
- **Reference hardware**: low-end dual-core Linux test hardware. Its make/model/specs, device IDs, driver versions, and distro must NEVER appear in the repo, docs, commits, or CI-visible strings. Perf numbers stay; the machine identity goes ("reference hardware" / "reference iGPU").
- **NEVER run `cargo build --release` locally.** Local release compilation risks thermal throttling and host freezing.
- **No local compilation except quick verification** (owner directive 2026-10-08): a single filtered `cargo test -p <crate> --lib <filter>`, or `cargo check -p <crate> --lib` for syntax-level breakage pre-push — only when the machine is idle (check `/proc/loadavg` first; skip if busy). Everything heavy (full builds/suites, benches, wasm targets, binaries) runs ONLY on GitHub Actions CI.
- Allowed local ops: the above checks, fast read-only inspection (grep, logs, image viewing), git operations.

## 2. Workspace Layout (verified)
- Rust workspace root is **`voxelcraft/`** — every cargo command runs there (CI uses `working-directory: voxelcraft`). 15 crates under `voxelcraft/crates/*`; default member `crates/voxelcraft`. License: **GPL-3.0-or-later** (`deny.toml` allow-list enforced by cargo-deny `check licenses`).
- (2026-10-10, owner-ordered: the Next.js template shell + its scaffold — `src/`, `package.json`, configs, `examples/websocket/`, `.zscripts/`, `tests/*.sh`, `Caddyfile`, `mini-services/` — was removed from the tree. `public/` stays: it is the game's WASM front.)

## 3. CI Gates (what must stay green)
Per push, `ci.yml`: `python3 scripts/legal_audit.py` (must print `[PASS]`) · `cargo fmt --check` · `cargo test --release --no-default-features --workspace` · golden hash `cargo test -p vc-world --lib golden_determinism` (exactly 9 `GOLDEN seed=` lines, identical on ubuntu/windows/macos) · wasm32 check · `cargo clippy -- -D warnings` · bench (`--features bench-bin --bin vc_bench`, uploads JSON + binary artifacts) · cargo-deny licenses.
- Concurrency cancels in-progress runs on the same ref: one push → one CI run; never push atop a running slice CI.
- `linux-game.yml` builds the release binary and runs scripted E2E legs headless: `xvfb-run -a -s "-screen 0 1280x720x24"`, `WGPU_BACKEND=Vulkan`, `"./$BIN_NAME" --smoke`, one leg per env var (`E2E_V114`/`115`/`116`, `E2E_MENU`, `E2E_CONTAINERS`, `E2E_FKEYS`, `E2E_BEDS`, `E2E_FLUIDS`, `E2E_PHASES`, `E2E_TURNTABLE`). lavapipe timings are environment noise — report, never gate on thresholds.

## 4. Clean-Room Legal Compliance (L1–L8)
- **L1 black-box only**: parity from public docs + our own oracle/behavior probes. No decompiled output, and no community reimplementation built from decompiled sources, for parity code. Propose the method before using it (Part 4.0 gate).
- **L2 read-side only**: ecosystem formats (packs, saves, NBT) accepted as inputs for interop; never emit or mirror another project's layout in shipped data. No reference set committed, bundled, or distributed in any form.
- **L3 zero copied assets**: no byte of any rights-holder asset (texture, sound, model, font, panorama) enters the repo; all art/audio synthesized procedurally in-repo.
- **L4 zero decompiled transcription**: never copy proprietary decompiled source, variable names, or pseudocode into code, comments, or docs; mechanics as clean-room behavioral descriptions only.
- **L5 study corpus = facts only**: reference sets used solely for fact-level counts and aggregate statistics (dimensions, palette histograms, banding/edge/symmetry/alpha stats, animation metadata). No pixel positions, masks, silhouettes, waveforms, or code stored or derived. Keep this provenance statement visible in `LEGAL.md`/docs; never scrub it.
- **L6 trademarks**: no third-party trademarks, names, logos, splash texts, folder conventions, or edition/update labels. In-game functional vocabulary is generic terms-of-art (`docs/LEGAL-COMPLIANCE.md` §3).
- **L7 script enforcement**: committed scripts must not read or reach any reference set; `scripts/legal_audit.py` enforces this in CI, with its self-test.
- **L8 honest claims**: tag every claim Verified / Tested / Code-only / Unknown. Test counts come from CI log `test result:` lines only. Numerical constants/formulas/timings cite an authoritative public source; unsourced values flagged `[ESTIMATED / APPROXIMATION]`.
- **Audit tripwires**: `scripts/legal_audit.py` fails the tree on certain trademarked-namespace strings — see the script itself for the exact list (do NOT quote the listed terms anywhere in the tree, docs included; this line previously quoted them and failed CI). Generic genre vocabulary (`redstone`, `creeper`, `netherrack`, …) is explicitly FINE — do not "fix" it.

## 5. Rendering & UI Fidelity Targets
- Seamless tiling (no border insets/clamping gaps); atlas UV gradients explicitly scaled to atlas dims; smooth lighting + AO; water flow/transparency; 15×15 crosshair; lower-right first-person 3D arm with walk bob + attack swing.
- Creative inventory (12 category tabs, player preview, armor/offhand slots, 9×5 grid + scrollbar, search, hotbar, trash); 19-option video settings layout; darkened-dirt world selection; two-column F3 debug screen.

## 6. Determinism & User Worlds (R5–R6)
- **R5**: worldgen uses exact IEEE arithmetic on CPU in the required numeric type; no FMA/fast-math; libm (not libc) for sin/cos/pow/exp/log/sqrt so results are identical on every OS/CPU. Golden hash change = HARD STOP.
- **R6**: never modify an original world; import works on a copy; automatic backups; unknown blocks/items/entities/NBT preserved verbatim through load and save; corrupt input never panics.

## 7. Session Protocol
- **R1**: no history rewrites, force-pushes, or destructive git ops without the owner's explicit confirmation of that exact operation.
- **R2 superseded**: do NOT wait for per-Part GO — work continuously through the plan; stop only at hard stops.
- **R3**: estimates are velocity ranges from repo history; no whole-game week totals.
- **R4**: keep honest provenance statements; never scrub them.
- **R7**: one slice per commit (~300 changed lines max); keep L1–L8, V1 in force at all times.
- **V1**: visual verdicts only for images actually viewed; describe exactly what was seen, never what was expected.
- **P1**: never ask the owner for reference captures/worlds/jars; numeric results only. No secrets, tokens, or personal files in work.
- **Turn flow**: slice reports go to `docs/WORKLOG.md` + `docs/AGENT-STATE.md` (live todo tracker, update after every slice), never closing chat messages. After CI green + state update, start the next slice immediately. A turn ends ONLY on: HARD STOP, Part REVIEW PACKET (`docs/CHECKPOINTS.md`, ≤400 words + adversarial 10-claim self-review), STOP file, or rate/context limit.
- **Sleep-shift** (owner-declared ~6h): never end voluntarily inside the window; keep a background waiter armed (CI watches/quota probes/log pulls); do quota-independent work between notifications; batch owner questions for morning. State files carry the handoff if suspended.
- **HARD STOP**: first line of `docs/BLOCKERS.md` reads `HARD STOP: <reason>`; triggers: possible L1–L8 breach/ambiguity, destructive git op, golden-hash change, CI red twice for the same cause after real fixes, confusion/looping/context pressure with uncommitted work, rate/tool failure, touching files outside the repo or any user world.
- **Docs sync**: `docs/WORKLOG.md` (append-only per slice), `docs/AGENT-STATE.md`, `docs/PARITY-BACKLOG.md`, README ship with code changes. Root `worklog.md` is the closed prior era — read-only.
- **In-game review**: slices touching gameplay/rendering get their own linux-game E2E run at their commit; Part reviews end with a full in-game tour (all legs re-run, fresh captures viewed under V1).
