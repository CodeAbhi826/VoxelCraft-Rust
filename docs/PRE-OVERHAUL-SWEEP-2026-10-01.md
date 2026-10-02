# PRE-OVERHAUL-SWEEP-2026-10-01.md — the pre-overhaul findings ledger

**Purpose:** durable memory for the pre-overhaul sweep. If agent context is
ever lost, this file + `worklog.md` + `docs/MASTER-PLAN.md` carry everything
gathered. The overhaul plan builds on this file: every finding below is a
candidate work item — fold into MASTER-PLAN rounds or change the plan on the
fly, per the standing mid-round rule.

**Status: PHASE 0 IN EXECUTION (2026-10-02)** — Round T code work COMPLETE
and CI-verified (all six gates green at `34a536f`, run 36947278822):
T1–T4 (greedy key full 16-bit state + WGSL mirror + F_LAVA + water-above
fold + lava culling), T5 (the 25-box static grid + the pixel-based
border-visibility E2E contract), T6 (F5 front distance 4.0, VERIFIED
Third-person_view), T11 (netherite pyramid), T7 (stale docs), T8 (FxHash
everywhere), T9+T10 (particle guards + small fixes). Remaining: the E2E/
vision verification of T1–T6+T11 from the CI-built binary; the ledger
reconcile; Phase 1 next.

---

## 1. Session state snapshot (2026-10-01)

| Item | Value |
|---|---|
| Branch | `test/full-sweep-2026-09-25`, pushed to origin, remote verified |
| HEAD | `8a51ed0` — "fix(clippy): collapse the two nested ifs in game.rs" |
| Commit chain this round | `1d592cb` (rebased to `9336e59`: rustfmt 1.9.0 reflow, 77 files, zero logic change) → `2bb2ba2` (CuboidFace type alias, vc-sim clippy) → `219a118` (RenderFrame params struct + div_ceil + as_chunks) → `79f313b` (as_chunks `.0` tuple fix) → `8a51ed0` (collapsible-if fixes) |
| Engine size | ~141.7k lines of Rust, 15 crates (14 `vc-*` + `voxelcraft` app) |
| Local toolchain | rustc 1.98.1 / rustfmt 1.9.0 (Arch); 31m45s release build (fat LTO) |
| CI policy | NO local compiling — all compiling gates run on GitHub Actions only |
| Legal | `scripts/legal_audit.py` → **[PASS]** (after the docs/WORKLOG.md reword below) |

## 2. CI gate status (GitHub Actions, run 36837860178 for `8a51ed0` — ALL GREEN)

| Gate | Status |
|---|---|
| `cargo test --release --no-default-features --workspace` | ✓ GREEN (4m28s) |
| `cargo check --release --workspace` | ✓ GREEN (native, audio on) |
| wasm32 check (`--workspace --lib`) | ✓ GREEN (32s) |
| `cargo fmt --check` | ✓ GREEN (9s) |
| `cargo clippy -- -D warnings` | ✓ GREEN (57s) — fully green after 3 fix pushes |
| headless benchmark | ✓ GREEN (1m44s) |

**THE ENGINE VERIFICATION GATES ARE ALL GREEN ON GITHUB ACTIONS** (run
36837860178, 2026-10-01) — the definition of done for this sweep's gate work.

**Owner decision (2026-10-01):** two subagents ran cleanly in parallel —
parallel subagents are SANCTIONED going forward (previously serialize-only).

**CI changes this round** (`.github/workflows/ci.yml`): added `fmt` (7s) and
`clippy` (with ALSA headers + rust-cache) jobs — neither gate ran anywhere
before this round. CI triggers on push to any branch + PR to main +
workflow_dispatch. Remaining known gap: `docs/LEGAL-COMPLIANCE.md` §6 claims
"`ci.yml` runs the audit scanner on every push" — FALSE today: no workflow
runs `scripts/legal_audit.py` (the doc's `audit_tools.py` name is stale too;
the real scanner is `scripts/legal_audit.py`). Add an audit job to ci.yml.

**Clippy burn-down so far** (clippy never ran on this codebase before —
it stops at the first failing crate, so each fix reveals the next):
1. `vc-sim` `type_complexity` (entities.rs:230) → CuboidFace type alias — FIXED
2. `vc-render` 4 errors: manual `div_ceil` (render.rs:5287), unit let-binding
   (`let _ = buf.unmap()`), `chunks_exact` with constant size
   (`chunks_exact_to_as_chunks` → `as_chunks::<4>().0`), `too_many_arguments`
   (10/7) → `RenderFrame` params struct (render.rs + the single call site
   game.rs:~22816) — FIXED
3. `voxelcraft` app 2 errors: `collapsible_match` (fkeys stage 2 arm → match
   guard) and `collapsible_if` (screenshot readback gate) — FIXED, CI verifying

## 3. Workspace report (structural + rule compliance) — clean

- Manifest matches the preset spec: wgpu 22, glam 0.29, naga 22 (runtime WGSL),
  mimalloc (native-only), rustc-hash 2, web-time 0.2, release `lto = "fat"` /
  `codegen-units = 1` / `strip = true`.
- `unsafe`: exactly ONE block in the engine (game.rs, read-only world alias
  with `// SAFETY:`; line drifted 13787 → ~14045 after the reflow) + the
  exempt `unsafe impl GlobalAlloc` (alloc_stats.rs).
- No placeholders (todo!/unimplemented!/FIXME — zero; XXX hits are pixel-art
  sprite data). No non-English text anywhere in source files.
- Era-locked constants intact with VERIFIED citations: drag
  `v1 = (v0 − 0.08) × 0.98` (player.rs:1169), GRAVITY 32.0, SPRINT_JUMP_SPEED
  7.127 (player.rs:36), lava 30/10 (fluids.rs:202-203), TICK_HZ 20.
- 863 `#[test]` functions; CI runs the whole suite green.
- `std::time` uses are all correctly `#[cfg(not(target_arch = "wasm32"))]`
  gated with js_sys/web-time wasm arms (world.rs:555-563 verified).

## 4. Internal game check (vision) — the game RUNS

Recipe that works (record it — do not rediscover):
```sh
# --smoke enters a world through the REAL input path (title → play → create);
# E2E_FKEYS then runs the F5/chunk-border/F2 ladder and exits 0/1 with a verdict
mkdir -p /tmp/vc-run2/screenshots && cd /tmp/vc-run2 && \
E2E_FKEYS=1 xvfb-run -a -s "-screen 0 1280x720x24" \
  voxelcraft/target/release/voxelcraft --smoke
```
- Build: `cargo build --release --no-default-features --bin voxelcraft`
  (31m45s, fat LTO). Binary lands in `voxelcraft/target/release/voxelcraft`.
- Rendering: the game ran on the REAL Intel UHD 600 (Celeron N4000) via
  Vulkan/Mesa 26.2.3 — NOT software; 10–19 fps, 1280×696 surface.
- Verdict: FKEY contract OK, exit 0; screenshots:
  `screenshots/e2e_fkeys_behind_*.png` (855 KB) + `e2e_fkeys_front_*.png`
  (556 KB). First attempt WITHOUT `--smoke` sat at the title screen forever
  (E2E_FKEYS waits for world entry) — `--smoke` is REQUIRED.
- Rendering CLEAN at 2× crops: no greedy-quad seams, no atlas bleeding, no
  z-fighting; F3 two-column overlay correct; hearts/hunger/hotbar slot wells
  correct; player rig renders correctly from behind.

### Visual findings (from the captures)

| # | Severity | Finding | Evidence |
|---|---|---|---|
| V1 | major | **F3+G chunk-border overlay does not render.** `borders=true` was logged and the E2E contract passed, but NO border lines appear anywhere in the frame (measured: player area, right terrain, far-field bands at 2× — all absent). Root cause (code analysis): the 25-box loop in render.rs (~5936-5969) rewrites ONE `LineUniform` via `queue.write_buffer` BETWEEN `pass.draw()` calls of the same render pass — wgpu applies write_buffer modifications before the whole command buffer executes, so all 25 boxes collapse to the LAST offset (bcx+2, bcz+2, off-screen). Fix: instance-rate vertex data (the engine's own origin_vb pattern) or per-instance bind groups written before the pass. The E2E contract only checks flags were armed, not that lines are VISIBLE — contract gap. | Measured absence + wgpu documented semantics |
| V2 | major | **F5 front view camera distance is 1.5, not vanilla's ~4.0.** game.rs:22645 (`sign=1.0, dist=1.5` for mode 2; behind mode 1 uses 4.0). At 1.5 the face fills the screen (measured: head-only in the front capture, vs the full player in the behind capture). Needs a live wiki citation at implementation time, then the distance + a doc note. | Measured + code-confirmed |

## 5. Code-review findings ledger

### Agent 1 — core crates (vc-blocks, vc-chunk, vc-mesh, vc-nbt, vc-rng, vc-inventory, vc-particles) — 20,083 lines, 16 files. DONE.

**Critical (2):**
- C1 | mesh.rs:534,978 | Greedy solid key packs the 16-bit state at bit 28 but decodes only 8 bits (`((key >> 28) & 0xff)`), and the tint byte at bit 36 overlaps state bits 8+ — every world-stored state ≥ 256 meshed as a solid cube is truncated: basalt 738 → 222 (Hopper tiles), stained-glass-white 400 → 144 (Repeater tiles), anvil 283 → 27 (Iron-Ore tiles); merge keys also collide across states sharing a low byte. Reachable: `is_model_state` is false for all window states (blocks.rs:4350-4465), so ALL E2/E3/V2-V14 world solids take the greedy path; no test meshes any state ≥ 256.
- C2 | mesh.rs:534,543 | Same key-layout root cause: for tinted states ≥ 256 the state's bits 8-9 OR into the tint slot — acacia leaves 420 + Plains tint pack decodes tint 0xC3 = WATER slot → foliage renders with the dark Forest water tint; the tint field is never trustworthy for states ≥ 256.

**Major (2):**
- M1 | mesh.rs:448-456 | `above == b` compares a raw STATE id against the WATER BLOCK id (9): flowing-water states 89..=95 never equal 9, so the "water above → full-height column" arm only fires for a source above — waterfalls/multi-level columns render at (8−level)/9 instead of full sheets, and the aw bit splits greedy runs. Should fold via `sb(above) == b`.
- M2 | blocks.rs:12293-12313 + mesh.rs:435-436 | LAVA has no `face_visible` special case and the mesher's fluid step-face culling is WATER-only: every lava-lava boundary emits faces from BOTH cells (double faces), and lava-lava top faces render interior 0.875-height stripes inside lava bodies.

**Minor (12):** same-block culling missing for V2 stained glass (200..215) and
SLIME_BLOCK (243) (overdraw); tautological Arc::ptr_eq assertion
(chunk.rs:572-580); `el_affects_ao(_el)` ignores its parameter and always
returns true, contradicting its doc (mesh.rs:932-934); stale/contradictory
`Chunk::get` doc (chunk.rs:364-373); stale V2/V3/V4/V5 id-window comments
(blocks.rs:819, 991, 1059, 1188); stale redstone id comment (blocks.rs:6467);
stale `decode_flat` docstring (chunk.rs:194-196 — it is the LIGHT engine's
input now, not the mesher's); stale mesh.rs header doc ("≤ 62 today" — states
now span 0..=862); unused `serde` dependency in vc-nbt/Cargo.toml; modulo
bias in `vc-rng::next_range` (negligible at n ≤ 32, rejection sampling would
be uniform); `build_mesh_inputs` allocates ~1.7 MiB per mesh_sections call
even for 1-section remeshes (documented Phase-7 design — hot-path note only).

### Agent 2 — sim/game layer (voxelcraft app, vc-sim) — 32,865 lines, 13 files. DONE.

**Critical/major (0). Minor (10):**
- The sanctioned unsafe block now sits at game.rs:14045 (line drift, not a violation).
- player.rs:1120 | Elytra comment says "descent clamped at −3.2 b/s" but ELYTRA_DESCENT is 2.5 (constant correct — 10:1 glide ratio pinned by test; comment stale).
- fluids.rs:16-18 | Stale module doc: "lava is not in the block registry yet" contradicts the fully implemented Phase-E2 LAVA fluid.
- redstone.rs:15-18 | Stale module doc: "repeaters/comparators/pistons … not in the registry yet" contradicts the implemented Phase-3 components.
- redstone.rs:294-297 | `lever_tick` body is a documented no-op empty `if` — intentional but dead-code shape.
- sim.rs:149 + entities.rs:433 | Dead double-XOR: Sim::new passes `seed ^ 0x0DB_5ED` into XpOrbSystem::new, which XORs the SAME constant again → the orb seed collapses to exactly `seed` (no stream collision — item system uses a distinct salt).
- game.rs:1515,1967,1556 | Integer-keyed std maps in production code (`gen_inflight`, `pending_edits`, `shulker_positions`) — should be FxHashMap/FxHashSet (convention).
- sim.rs:120 + ticks.rs:20 | `conduits`/`pending_pos` std HashSets — convention deviation (order-insensitive consumption, no determinism impact).
- game.rs:3862-3882 | Stuck-flag window: F3 held → Esc to Pause → release F3 in the pause menu leaves `f3_held = true`; the next in-game F3 press is swallowed. No state reset on screen transition for this flag.
- bin/vc_bench.rs:31,49,54 | Dev-tool only: `percentile_ms` unwraps `partial_cmp`; args `parse().unwrap()` — benchmark binary, never in tick paths.

### Agent 3 — render layer (vc-render, 42,471 lines, 36 files) — DONE.

**Critical/major (0). Minor (10):**
- render.rs:891 | PARTICLE_SHADER fs_main samples the block atlas with `textureSample` + implicit derivatives and NO tile-safe inset/mip-cap — the block-particle pass lacks the bleed/fringe guards the terrain/water shaders carry (transient fringes possible when the random 4×4 quarter is flush with a tile boundary).
- render.rs:1627 | `regions: HashMap<(i32,i32), RegionArena>` — std HashMap for an integer-keyed production map (the same struct's `chunks` correctly uses FxHashMap).
- draw.rs:270-341 | occlusion flood uses std `HashSet<(ChunkPos,u8)>`/`HashSet<ChunkPos>` — convention deviation.
- item_icon_cache.rs:273-376 | std HashMaps for integer-keyed production maps (`entries`, `cell_owner`, `ready_cells`).
- ui.rs:2134,2325 | `icon_cells: Option<Arc<HashMap<u16,[u8;2]>>>` — std HashMap for the integer-keyed icon snapshot.
- gui/font.rs:166-173 | char-keyed std HashMaps (`ink`, `glyphs`, `runs`) — borderline (char ≈ u32 keys).
- gpu_mesh.rs:620 | stale doc: "STATE_COUNT=236, BLOCK_COUNT=103" contradicts the actual layout (863/533 — matches the WGSL header + the wgsl_lut_offsets_match_rust test); doc only, code correct.
- gui/loader.rs:3 | stale doc: calls the loader a "Phase 1 filesystem placeholder" while Phase 4 `load_from_pack` replaces that path; no code stub.
- gui/loader.rs:191 | `applied` pack-source list built per sheet then discarded — dead bookkeeping contradicting its own comment.
- bin/ui_snapshots.rs:14 | dead `snap` helper kept only via `let _ = snap;` — superseded by `snap_at`.

**CLEAN (verified):** zero unsafe; zero placeholders; the mid-pass
write_buffer collapse class scanned across EVERY render pass — the only
instance is the known chunk-border loop (V1); terrain/water atlas guards are
test-protected (LOD-aware gradient inset, textureSampleGrad, mip-2 cap);
font-grid placement/advance/shadow verified on all three paths with
device-exact tests; zero non-English script; std::time all gated correctly.

### Agent 4 — world/content layer (vc-world, vc-gameplay, vc-pack, vc-anvil, vc-audio — ~43,628 lines, 43 files) — DONE.

**Critical/major (1):**
- beacon.rs:623-625 | `is_base_block` accepts only IRON/GOLD/DIAMOND blocks while the module's own VERIFIED citation (beacon.rs:586-587) lists iron/gold/emerald/diamond/netherite as valid pyramid materials — emerald/netherite beacon pyramids cannot be built (parity gap contradicting the in-code citation).

**Minor (13):**
- mobs.rs:2509-2520 | Soul Sand Valley ghast spawn roll ≈4.9% effective vs the cited ≈2.5% — the inner 1/20 gate is applied twice on the 1..=20 arm and once on the 21..=70 arm (wastes and basalt-deltas rolls match exactly).
- datapack.rs:798-799 | duplicated identical condition `f_kind == Some("voxelcraft:set_count")` — the intended bare `"set_count"` arm never exists; bare-form set_count functions are silently dropped.
- craft.rs:1206-1207 | duplicated `MELON_SLICE` match arm in `match_kitchen` — second arm unreachable-with-guard, silent dead code.
- datapack.rs:1324 | `for pack in out.packs.clone()` clones the entire report set at load time (load-time only).
- world.rs:122,132 | `decorated`/`save_dirty` (i32,i32)-keyed std HashSets contradict the in-code citation claiming "every integer-keyed map in this crate swaps std SipHash for FxHash".
- villagers.rs:555 | `populated: HashSet<[i32; 2]>` — same FxHash-convention class.
- model.rs:62,608 | `by_state: HashMap<u16, Vec<ModelChoice>>` + compile_block_dispatch's return map — std SipHash on the hottest registry lookups at mesh time.
- gen.rs:512,518 | mojibake "â" (corrupted em-dash) in the nether_region_biome doc — the only encoding corruption in scope.
- mobs.rs:524-527 | duplicated doc fragment "neutral until provoked)." on `hostile()`.
- mobs.rs:4364,4490 | vestigial always-true `restore` flag in the bee day-phase branch — dead scaffolding.
- mobs.rs:649-652 | `from_egg(19)` (the E2 wither spawn egg) falls through to Chicken — spawning a chicken on wither-egg use (disclosed in-comment, still a wrong-mob gap).
- vanilla_noise.rs:140 | gradient pick `hash % 12` over the doubled 512-entry permutation table — accepted clean-room formulation of the cited Perlin 2002 algorithm, noted for parity completeness.
- dump_chunk.rs:44 | `std::time::Instant::now()` in the dump_chunk example without a wasm gate — native-only diagnostic binary; the only ungated std-time use in scope.

**CLEAN (verified):** zero unsafe; zero placeholders (panic! only in tests;
"[placeholder, disclosed in worklog]" strings are documented deterministic
values, not stubs); era-locked constants all live-cited with no
value/citation contradiction (mob data 49 rows, fuse 30/power 3/charged 6,
COOK_TICKS 200 + full fuel ladder, dragon phases + timeline, fishing
85/10/5, enchanting 38-set, brewing 400-tick cycle); 1.16.5 parity verified
against in-code citations; lifetimes sound (PackSource dyn traits, Arc COW,
documented Box::leak 12-rig cache); skylight propagation deviation from
vanilla's free-down rule is documented with a differential test oracle —
disclosed, not a violation.

### SWEEP TOTALS (agents 1–4, every engine line read)

- Lines reviewed: ~139,000+ across 4 agents (vc-blocks 15,670; vc-chunk 619;
  vc-mesh 1,824; vc-nbt 590; vc-rng 54; vc-inventory 403; vc-particles 923;
  voxelcraft app + vc-sim 32,865; vc-render 42,471; vc-world + vc-gameplay +
  vc-pack + vc-anvil + vc-audio 43,628).
- Findings: **2 critical, 4 major, 45 minor** — all ledgered here and folded
  into `docs/OVERHAUL-PLAN-2026-10-01.md` Round T (T1–T11).
- Clean across every checklist: zero unsafe beyond the one sanctioned block;
  zero placeholders; era-locked constants intact and cited everywhere; zero
  non-English script (one mojibake byte); determinism verified; the
  mid-pass write_buffer collapse class has exactly one instance (V1/T5).

## 6. Work done this round (chronological)

1. Full workspace inspection + report (structure, git, manifest, rule checks).
2. Local compile run cancelled at the owner's instruction; compiling gates
   moved to GitHub Actions.
3. `cargo fmt` applied (rustfmt 1.9.0 reflow, 77 files, zero logic change) —
   fmt gate green.
4. ci.yml: added fmt + clippy jobs; committed and pushed (rebased over the
   remote's Round A commit `b995992` — no force-push; one conflict in
   vc-render/src/render.rs resolved by taking the remote side + re-running
   cargo fmt).
5. Clippy burn-down via CI feedback: 3 pushes (CuboidFace, RenderFrame +
   3 small fixes, collapsible-ifs). test/wasm/fmt/bench all GREEN.
6. Internal game check: release binary built (31m45s), `--smoke` + E2E_FKEYS
   run under Xvfb on real Intel GPU — FKEY contract OK, 2 screenshots
   captured, deep-pixel audit (2× crops, deterministic tools) — terrain/HUD/
   rig CLEAN; V1 (borders) + V2 (front camera distance) found.
7. Legal audit: scanner run → 1 violation (docs/WORKLOG.md carried the
   publisher's built-in art-pack label) → reworded → **[PASS]**.
8. Code review agents 1-2 done; agent 3 relaunched; agent 4 pending.
9. This ledger created.

## 7. NOT done (explicitly)

- No engine logic/bitmasking/constant fixes (C1, C2, M1, M2 are LEDGERED for
  the overhaul plan, not fixed ad hoc).
- No local compiling gates (all compiling ran on GitHub Actions).
- No plan-of-record rewrite yet (MASTER-PLAN update happens when all four
  review agents have reported).
- No force-push, no branch switch, no CI job removals.
