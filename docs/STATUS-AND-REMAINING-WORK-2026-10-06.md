# VoxelCraft — Verified Status & Remaining Work

**Written 2026-10-06 at `bcc47ff` (branch `test/full-sweep-2026-09-25`).**
Every number below was measured in this session unless tagged otherwise. Where I
could not measure something, it says **Unknown** — not a guess.

**Claim tags used throughout:** `Verified` = measured against a file/command.
`Tested` = proven by an executed test. `Code-only` = present in source, never
exercised. `Unknown` = not determined.

---

## 0. What this document is, and what it is not

It is a status report. It contains **no completion percentages**, because a
percentage of "100% vanilla 1.16.5" is not measurable without a byte-level
oracle I do not have. What *is* measurable is: how many systems exist, which
have tests, which are stubs, and which are absent. That is what follows.

**Hard structural ceilings** (these cap everything, verified):

| Ceiling | Evidence | Consequence |
|---|---|---|
| **Cannot open a vanilla world** | No byte-compat test exists in [vc-anvil](voxelcraft/crates/vc-anvil/src/). Packed BlockStates are *referenced* (50 hits) but there is no fixture round-trip against a real 1.16.5 save. | The headline promise "can read and write vanilla worlds" is **unmet**. |
| **Same seed ≠ vanilla world** | `gen.rs` carries a self-disclosure that vanilla's biome climate sampler needs the full multi-noise stack. | Same-seed parity is **not achieved** and cannot be without Phase 7's oracle. |

Everything below should be read against those two.

---

## 1. The engine as measured

| Metric | Value | Tag |
|---|---|---|
| Rust lines | **149,975** | `Verified` (wc -l, excl. target) |
| Crates | **15** (14 `vc-*` + `voxelcraft`) | `Verified` |
| Tests (CI) | **909 passed, 0 failed, 2 ignored** | `Tested` — [run 37414947052](https://github.com/CodeAbhi826/VoxelCraft-Rust/actions/runs/37414947052) |
| CI gates | **6, all green** | `Tested` |
| `BLOCK_COUNT` | **539** | `Tested` — asserted `blocks.rs:13723, 14694` |
| `STATE_COUNT` | **892** | `Tested` — asserted `blocks.rs:14695` |
| Biomes | **28** | `Verified` — `Biome` enum, `gen.rs:27` |
| Mob kinds | **65** (`MOB_DATA: [MobDef; 65]`) | `Verified` — `mobs.rs:1059` |
| Status effects | **32** | `Verified` |
| Production panics | **0** | `Verified` — per-file scan |
| Committed PNGs | **3,964** | `Verified` |

Note the correction: earlier docs said "66 MobKinds". The array is `[MobDef; 65]`.
The doc figure is wrong; the code is the truth.

### Largest files (maintainability, `Verified`)

| File | Lines |
|---|---|
| `voxelcraft/src/game.rs` | **27,763** |
| `vc-blocks/src/blocks.rs` | 16,122 |
| `vc-gameplay/src/mobs.rs` | 10,785 |
| `vc-render/src/ui.rs` | 8,800 |
| `vc-world/src/gen.rs` | 8,066 |
| `vc-render/src/render.rs` | 6,930 |

---

## 2. System-by-system, measured

`DONE` = implemented with tests. `PARTIAL` = implemented but incomplete. `STUB` =
placeholder. `ABSENT` = no implementation found.

### World generation
| Item | Status | Evidence |
|---|---|---|
| Terrain, ores, trees | PARTIAL | `gen.rs` 8,066 lines |
| Biomes (28) | PARTIAL | `Biome` enum — 28 variants vs vanilla's 66 |
| Caves | PARTIAL | carve pass present |
| **Multi-noise biome layers** | **STUB** | `gen.rs:518` self-disclosure |
| Ravines | PARTIAL | `RAVINE_CHANCE` self-labelled a tuning value |
| Structures | PARTIAL | 9 present; **`ocean_monument` = 0 hits**, **`end_city`/`ruined_portal` = 0 hits** `Verified` |
| Villages, strongholds, mineshafts | PARTIAL | generators exist, placement unproven against vanilla |

### Persistence
| Item | Status | Evidence |
|---|---|---|
| Own Anvil format + NBT | DONE | 23 tests in vc-anvil |
| `level.dat` read | PARTIAL | 57 references |
| Packed BlockStates (1.16) | PARTIAL | 50 references, no round-trip fixture |
| Heightmaps / POI / TileEntities | PARTIAL | 19 references |
| **Open a real vanilla save** | **ABSENT** | no fixture, no test `Verified` |

### Content registries
| Item | Status | Evidence |
|---|---|---|
| Blocks / states | PARTIAL | 539 / 892 of vanilla's ~1,000+ / ~20,000 |
| **Items (tools, swords, armor)** | **ABSENT** | `ItemDef`/`ITEM_COUNT` = **4 hits** total `Verified` |
| Mobs (65) | PARTIAL | roster complete, AI depth varies |
| Enchantments | PARTIAL | 38 declared; Looting unwired |
| Potions/effects | PARTIAL | 32/32 kinds; only 9 brewing families |

### Combat
| Item | Status | Evidence |
|---|---|---|
| Damage/armor/crits | DONE | `combat.rs` formula tests pass |
| **Weapons** | **ABSENT** | `held_attack` returns flat 1.0 HP; `tool_profile` returns `None` for every input |
| **Sweep attack** | **ABSENT** | 2 hits, both comments `Verified` |

### Entities & AI
| Item | Status | Evidence |
|---|---|---|
| `ai_tick` dispatch | PARTIAL | 25 references |
| Spawn categories + caps | PARTIAL | 17 references |
| **Pathfinding (A*)** | **ABSENT** | **0 hits for `astar`/`AStar`/`Pathfind`** `Verified` |
| Villager schedules | PARTIAL | `villagers.rs` exists |

### Redstone
| Item | Status | Evidence |
|---|---|---|
| Components | PARTIAL | 199 references |
| Quasi-connectivity | PARTIAL | 12 references — not full QC |
| Piston block entity, dispenser logic | STUB | matrix says deferral notes only |

### Commands / meta systems
| Item | Status | Evidence |
|---|---|---|
| **Slash-command parser** | **ABSENT** | **0 hits** `Verified` |
| Command block | STUB | 4 references — renders, no execution |
| **Scoreboard / teams** | **ABSENT** | **0 hits** `Verified` |
| **Recipe book** | **ABSENT** | **0 hits** `Verified` |
| Advancements | STUB | 10 references |
| Gamerules | DONE | verified earlier this session |

### Rendering
| Item | Status | Evidence |
|---|---|---|
| FSR 1.0 (EASU + RCAS) | DONE | 153 references; identity-at-1× test exists |
| GPU meshing (WGSL compute) | DONE | 13 references + CPU/GPU parity tests |
| GLSL→WGSL (naga) | PARTIAL | 113 references, feature coverage incomplete |
| MSAA | PARTIAL | 22 `sample_count` sites — not user-selectable |
| Sound events / `sounds.json` | PARTIAL | 8 references |
| **Lang key system** | **ABSENT** | **0 hits for `lang.json`/`LangKey`** `Verified` |

---

## 3. Checklist against the original plan (Parts I–IV)

| Plan part | Status | Note |
|---|---|---|
| **Part I** — engine foundation | DONE | 15 crates, CI green |
| **Part II** — Phase 0–11 ladder | CLOSED (per MASTER-PLAN) | but several Phase 2B items below are unmet |
| **Part III** — Rounds A–S | Round A shipped; Round B parked unlanded | Round B code is in `stash@{0}` + `/tmp/roundb-backup/` |
| **Part IV** — Phase 2B (pack compatibility) | **NOT STARTED** | added 2026-10-06, 8 slices planned |

**Open debt carried forward:**

| Debt | State |
|---|---|
| Round B menu vision audit | **Unlanded.** Patch applies clean; one known test-construction bug (`menu_layouts_survive_the_gui_scale_ladder` builds screens before `set_live_ui_size`). |
| `Chunk::get` migration | **Partial.** `LocalXZ` typed door added; ~186 raw call sites remain. |
| WASM bundle | **Stale**, marked in [public/README.md](public/README.md); workflow now builds on this branch. |
| Iconic-art review | **Blocked on owner verdicts.** 14-cell sheet at `/tmp/vc-contact/iconic_cast.png`. |

---

## 4. What remains, by size

Ordered by dependency, not by effort. `L` = weeks-to-months at this project's
observed pace for one developer + agent.

| # | Work | Size | Blocks |
|---|---|---|---|
| R1 | **Item/tool/armor system** — no swords exist, so no sweep, no loot depth | **L** | combat depth, creative inventory, progression |
| R2 | **Pathfinding** — mobs cannot navigate; 0 A* hits | **L** | mob behaviour, raids, village realism |
| R3 | **Commands** (parse, execute, permission levels, tab completion) | **L** | creative mode, automation, debugging |
| R4 | **Vanilla world compatibility** — read + write real 1.16.5 saves | **L** | the headline promise |
| R5 | **Worldgen parity** — multi-noise biomes, missing structures, oracle harness | **XL** | same-seed promise |
| R6 | Block/item long tail (539 → ~1,000 blocks) | **XL** | content completeness |
| R7 | Chat, scoreboard, advancements, recipe book | **M–L** | social/meta layer |
| R8 | Lang key system | **M** | Phase 2B.4 |
| R9 | MSAA/AF as real settings, per-device capability detection | **M** | Phase 1 |
| R10 | Redstone depth (QC, piston BE, dispenser logic) | **L** | technical play |
| R11 | Phase 2B.1–2B.8 pack compatibility | **L** | mod/pack ecosystem |
| R12 | Entity models/skeletons/animations (Phase 4) | **XL** | visual fidelity |
| R13 | Audio, menus, F3 fields (Phase 12) | **L** | product completeness |
| R14 | Release/legal hardening (Phase 14) | **M** | public release |

**Critical path:** R1 → R2 → R4 → R5. Tools first (they gate combat and mob
depth), then navigation, then the two promises that are currently unmet
(vanilla saves, same-seed worldgen). R3/R7 can run in parallel with R1–R2.

**Honest total:** R4 + R5 alone (the two unmet promises) are multiple months of
full-time work, and R5's ≥99% threshold needs oracle data only the owner can
supply.

---

## 5. Comparison to other clones

**This section is from general knowledge, not measured in this repo. Treat the
figures as directional.**

| | VoxelCraft | the established voxel engine / the surveyed community project |
|---|---|---|
| Language | Rust + WGSL | C++ engine + Lua game |
| Licence | Apache-2.0 (code) | GPL/LGPL + CC-BY-SA assets |
| Target | 1.16.5 parity, clean-room | the reference game, *like but not equal*; the surveyed community project ≈ 1.17 |
| Goal | byte-parity where measurable | playability and moddability |
| Scripting | none yet | Lua mods, the core strength |
| Content | 539 blocks, 65 mobs | the surveyed community project is content-rich by community contribution |

**What VoxelCraft has that they do not:** a hard parity target with a citation
discipline, and a clean-room legal chain that survives forensic audit.

**What they have that VoxelCraft does not:** a scripting layer (the established voxel engine mods are
why it has survived years), a community content pipeline, and shipping maturity.
Those are the two structural disadvantages here, and neither is fixed by adding
more Rust.

---

## 6. Skill rules that governed this pass

| Skill | Status this pass | Rule applied |
|---|---|---|
| `audit` | **Applied** | Four-dimension grading; CORRECTNESS capped because I did not exercise the game |
| `verification-before-completion` | **Applied** | Every number above came from a command run in this session |
| `review` | **Applied** | Adversarial read of the diff; found the stale `OUT_ROOT` and the over-broad script deletion |
| `find-skills` | **Applied** | Skill sources verified rather than recommended blind |
| `brainstorm` | **Partial** | Read-only; no files edited |
| `learn` | **Rule recorded, not executed** | Its SKILL.md body was not supplied in this invocation (only the name), so its actual instruction — persist learnings to `AGENTS.md` — could not be followed. The learnings this session produced are listed in §6a instead. |
| `rust-pro` | **Applied** | Type-level guards (`LocalXZ`), `Result` over `panic` |
| `frontend-design` | **Out of scope** | No UI work requested this turn |
| `deepen` | **Out of scope** | By its own rule: applies only when the user asked to *build* |
| `experience` | **Out of scope** | Same — no user-facing polish requested |
| `overhaul` | **Out of scope** | Same — no diff to restructure |
| `reenvision` | **Out of scope** | Same — no weakest-subsystem rebuild requested |
| `spec-brainstorm` | **Out of scope** | Same — reporting task, not a build |

Five of the twelve are explicitly build-only and correctly did not fire. That is
the rule working, not the rule being ignored.

### 6a. Non-obvious learnings this session produced
(What `learn` would persist to `AGENTS.md`; recorded here because its SKILL.md body was not supplied.)

1. `cargo clippy --workspace --all-targets -D warnings` **fails** on pre-existing
   test-code lints; CI runs bare `cargo clippy -- -D warnings` (lib only). Match CI
   exactly or you will chase warnings that are not gated.
2. A manual `gh workflow run` collides with the push-triggered run of the same
   commit under the concurrency group — the push run gets `cancelled`. Dispatch
   once and wait; don't re-dispatch on a cancelled status.
3. `.gitignore` has `/scripts/`, but those files were tracked before the rule.
   After any script is deleted and restored, `git add` silently fails — use
   `git add -f`.
4. `scripts/legal_audit.py` (not `ci/legal_audit.py`) is the gate, and it
   flags trademark terms in **docs** too. Any new doc containing the rights-holder
   or product name fails the build.
5. `build.rs` embeds `voxelcraft/builtin-packs/classic-art/` resolved from the
   cargo workspace root — not the repo root. Touch `build.rs` to force a re-run;
   a `cargo check` alone reports "Finished" from cache and proves nothing.
6. A Section index is packed `(y << 8) | (z << 4) | x` with x/z **unmasked**, so
   out-of-range local coords alias silently (x=32 -> local x=0; z=18 carries into
   y). This is the trap `LocalXZ` exists to close.

---

## 7. Limitations of this report

1. **No runtime verification.** I did not launch the game. Every claim is static
   analysis plus CI results. Rendering, input, and frame behaviour are `Code-only`.
2. **No percentages.** Deliberate — see §0.
3. **Grep-based absence proofs.** "0 hits for `astar`" means no code under that
   name. A differently-named pathfinder would be missed.
4. **The comparison section is unverified** — general knowledge, not measured.
5. **Unknown: the reference ZIP contents.** With the reading analyzer deleted,
   what the corpus actually held cannot be established.
6. **CI numbers are for `bcc47ff` only.** `game.rs` behaviour under real play is untested here.
---

## 8. Four-dimension audit of this session's work (Phase 0 FIX batch)

Graded as a demanding principal engineer would. Self-assessed, so read with the
usual discount.

### SPEC — **6/10**
Did the requested batch, one commit per risk item, nothing unrelated touched.
Deductions: I **over-deleted** `vault_contact_sheet.py` in risk #2 (it only read
our own output) and restored it — scope was temporarily wrong. I also added the
`OUT_ROOT` fix, which was outside the stated batch but a real defect in a file I
had already edited. Gaps: the iconic-art visual pass (#4) is **not done** — I
cannot view images, so I produced artifacts instead of the deliverable.

### DESIGN — **5/10**
Caps at 5 by the rule: a grown app still living in one or two files.
`game.rs` is **27,763 lines** — 18.5% of the entire codebase in a single file.
`blocks.rs` 16,122 and `mobs.rs` 10,785 compound it. State ownership is not
single: `held_attack`'s damage lives in `vc-gameplay/combat.rs` while the caller
in `game.rs` decides what is held. Gaps, in order:
1. Split `game.rs` — player, world-stream, and UI-routing concerns are separable
   without reading the whole file.
2. `blocks.rs` and `mobs.rs` are generated-registry-plus-logic hybrids; the logic
   belongs outside the table.
3. `Chunk::get`'s mixed convention (world-y, local x/z) is exactly the kind of
   thing that should not be expressible; I added a typed door but 186 call sites
   still use the old path.

### CORRECTNESS — **6/10** *(capped at 6: I did not exercise the real surface)*
CI proves 909 tests pass with 6 green gates. But the audit rule is explicit —
asserted-but-unexercised caps this at 6, and I have never launched the game in
this session. Every rendering, input and frame claim is `Code-only`. What I *did*
exercise: the legal guard (planted violations, both classes fired), the
synthesizer path resolution, `vc-rng`/`vc-chunk`/`vc-gameplay` tests, and the full
clippy/fmt/audit gate set. Gaps:
1. No run of the binary — renderer boot path changed signature, only compiled.
2. `wait_done`'s new `Result` branch is never hit by any test.
3. The duplicate-tree deletion was verified by re-running `build.rs`, not by
   launching the game.

### QUALITY — **7/10**
Net-positive: 419 lines of reference-reading tooling deleted, 53 duplicate files
removed, three colormap LUTs purged, 12 tests added. Over-engineering is low. The
debt is inherited, not mine — but I'm grading the result. Gaps:
1. `held_attack`'s `tool_profile` returns `None` for every input — a real
   function shape with no behaviour yet. Honest, but it is scaffolding.
2. `gpu_mesh::LostJobs` is a 1-line public typealias used once.
3. Three separate test modules in `ui.rs` (`tests`, `screen_tests`,
   `round13_station_tests`) with overlapping fixtures.

### Most valuable next pass overall

**Vanilla world compatibility (R4),** and not for feature reasons. It is the only
remaining item where the project makes a promise in its own README that it does
not currently keep, it unblocks Phase 15's oracle and Phase 5 acceptance testing,
and it is the one gap a user would discover in the first five minutes. Commands
(R3) are comparable in value and smaller, but they don't unblock anything else.

*Scoring note:* the audit skill's guidance is that a generous grade wastes the
budget that would have fixed the gap. These are my honest numbers, not a target.
