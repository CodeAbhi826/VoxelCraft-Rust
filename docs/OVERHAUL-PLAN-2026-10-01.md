# OVERHAUL-PLAN-2026-10-01.md — the overhaul umbrella plan

**Purpose:** the umbrella plan for the complete overhaul of this project.
The previous plan of record — `docs/MASTER-PLAN.md` (19 rounds A–S, all
Phases 0–13/0–11 closed, the beyond-1.16.5 queue and the perf-parity notes)
— is incorporated here BY REFERENCE and stays untouched on disk. This file
holds what the master plan did not: the 2026-10-01 sweep's discovered
defects (Round T), and — to be dictated by the owner after the full report —
the rest of the overhaul structure.

**Baseline (2026-10-01):** all SIX CI gates green (run 36868455480: fmt,
wasm, test 4m17s, legal audit, clippy, bench); native game verified live
under Xvfb on the real Intel UHD 600 (Vulkan/Mesa); findings ledger
`docs/PRE-OVERHAUL-SWEEP-2026-10-01.md`; worklog Task 4.

**Standing policies (inherited from the master plan, every round):**
1. Nothing is skipped — verified live and marked done with evidence.
2. Native-first verification (Linux binary before wasm every round).
3. Clean-room rules + `docs/LEGAL-COMPLIANCE.md` apply to everything; the
   legal audit gate now runs in CI on every push.
4. Per-round commits: implement → unit tests → native E2E → vision-verify →
   commit → wait for CI green → next round.
5. Compiling gates run on GitHub Actions only (owner decision, 2026-10-01).
6. Parallel subagents sanctioned (owner decision, 2026-10-01) — quality
   never compromised for speed.

---

## Part 1 — Incorporated: the master plan of record

`docs/MASTER-PLAN.md` — Part I (Transformation Roadmap, 13 phases, CLOSED),
Part II (Master Engineering Spec Phases 0–11, CLOSED), Part III (execution
ladder: Rounds A–S with their gates), the beyond-1.16.5 queue, and the
Sodium/Lithium/Phosphor/Starlight/FerriteCore perf-parity notes. Execute
Rounds A–S as written there; Round T below executes FIRST.

## Part 2 — Round T: 2026-10-01 sweep burn-down (EXECUTES BEFORE ROUND A)

The 2026-10-01 full sweep (CI gates + internal game check with vision +
~141.7k-line code review by 4 agents; ledger:
`docs/PRE-OVERHAUL-SWEEP-2026-10-01.md`) found defects that fold into the
plan here. Every fix: implement → unit tests → native E2E under Xvfb →
vision-verify → commit → CI green. NOTHING is fixed ad hoc.

**Critical (meshing parity — states ≥ 256 render wrong tiles):**
- **T1** | mesh.rs:534,978 | The greedy solid key packs the 16-bit state at
  bit 28 but decodes only 8 bits (`((key >> 28) & 0xff)`), and the tint byte
  at bit 36 overlaps state bits 8+ — every world-stored state ≥ 256 meshed as
  a solid cube truncates (basalt 738 → 222 → Hopper tiles;
  stained-glass-white 400 → 144 → Repeater tiles; anvil 283 → 27 → Iron-Ore
  tiles); merge keys collide across states sharing a low byte (repeater
  142..=173 vs V2 400..=415 share low bytes). Reachable for ALL E2/E3/V2-V14
  world solids (`is_model_state` false for the windows); no test meshes any
  state ≥ 256. Fix: widen the key layout (state at 28 bits decoded as u16,
  tint moved above the state bits) + add the missing state ≥ 256 meshing test.
- **T2** | mesh.rs:534,543 | Same key-layout root cause, tint side: for
  tinted states ≥ 256 the state's bits 8-9 OR into the tint slot — acacia
  leaves 420 + Plains tint pack renders with the dark Forest water tint.
  Same fix as T1.

**Major:**
- **T3** | mesh.rs:448-456 | `above == b` compares a raw STATE id against the
  WATER BLOCK id (9): flowing-water states 89..=95 never match, so the
  "water above → full-height column" arm only fires for a source above —
  waterfalls/multi-level columns render at (8−level)/9 instead of full
  sheets, and the aw bit splits greedy runs. Fix: fold via `sb(above) == b`.
- **T4** | blocks.rs:12293-12313 + mesh.rs:435-436 | LAVA has no
  `face_visible` special case and the fluid step-face culling is WATER-only:
  every lava-lava boundary emits faces from BOTH cells (double faces) and
  lava-lava top faces render interior 0.875-height stripes inside lava
  bodies. Fix: extend the fluid culling to the lava block-id family.
- **T11** | beacon.rs:623-625 | `is_base_block` accepts only
  IRON/GOLD/DIAMOND blocks while the module's own VERIFIED citation
  (beacon.rs:586-587) lists iron/gold/emerald/diamond/netherite as valid
  pyramid materials — emerald/netherite beacon pyramids cannot be built
  (parity gap contradicting the in-code citation). Fix: accept the cited
  five materials + a pyramid test for each.
- **T5 (V1)** | render.rs:~5936-5969 | The F3+G chunk-border overlay renders
  NOTHING despite `borders=true` (measured on the 2026-10-01 captures):
  the 25-box loop rewrites ONE `LineUniform` via `queue.write_buffer`
  BETWEEN `pass.draw()` calls of the SAME render pass — wgpu applies
  write_buffer modifications before the whole command buffer executes, so
  all 25 boxes collapse to the LAST offset (bcx+2, bcz+2, off-screen). Fix:
  instance-rate vertex data (the engine's own origin_vb pattern) or
  per-instance bind groups written before the pass. ALSO fix the E2E
  contract gap: the FKEYS leg checks flags were armed, not that border lines
  are VISIBLE in the capture.
- **T6 (V2)** | game.rs:~22645 | The F5 front-view camera distance is 1.5 vs
  the behind view's 4.0 — the reference game's front view sits at ~4 blocks;
  at 1.5 the face fills the screen (measured). Fix: 4.0 with a live wiki
  citation for the F5 offset at implementation time, then a vision
  re-capture.

**Minor (grouped, one commit class each):**
- **T7** Stale docs/comments: gpu_mesh.rs:620 (STATE_COUNT 236 → 863),
  mesh.rs:13-14 header, chunk.rs:194-196 decode_flat docstring (it is the
  LIGHT engine's input), chunk.rs:364-373 Chunk::get doc, blocks.rs:819/991/
  1059/1188/6467 id windows, fluids.rs:16-18 + redstone.rs:15-18 module docs,
  gui/loader.rs:3, player.rs:1120 (elytra −3.2 vs the correct 2.5 constant),
  gen.rs:512,518 mojibake "â" (corrupted em-dash), mobs.rs:524-527
  duplicated doc fragment.
- **T8** Convention: std HashMap/HashSet for integer-keyed production maps →
  FxHashMap/FxHashSet — render.rs:1627 (regions), draw.rs:270-341 (occlusion
  flood), item_icon_cache.rs:273-376, ui.rs:2134/2325 (icon snapshots),
  game.rs:1515/1967/1556 (gen_inflight/pending_edits/shulker_positions),
  sim.rs:120 + ticks.rs:20 (conduits/pending_pos), world.rs:122,132
  (decorated/save_dirty — self-contradicting citation), villagers.rs:555
  (populated), model.rs:62,608 (by_state + dispatch map — the hottest
  registry lookups at mesh time).
- **T9** Particle shader (render.rs:891): PARTICLE_SHADER fs_main samples
  the block atlas with NO tile-safe inset/mip-cap — port the terrain/water
  bleed guards (gradient inset + mip cap) to the particle pass.
- **T10** Small fixes: unused `serde` dep in vc-nbt/Cargo.toml; modulo bias
  in vc-rng::next_range (rejection sampling); tautological Arc::ptr_eq
  assertion (chunk.rs:572-580); `el_affects_ao(_el)` ignores its parameter
  (mesh.rs:932-934, documented behavior not implemented); dead bookkeeping
  in gui/loader.rs:191; dead `snap` helper (bin/ui_snapshots.rs:14); dead
  double-XOR in Sim::new/XpOrbSystem (entities.rs:433); f3_held stuck-flag
  window across Esc-to-Pause (game.rs:3862-3882 — reset key state on screen
  transition); lever_tick empty-if shape (redstone.rs:294-297); datapack.rs
  798-799 duplicated set_count condition (bare-form functions silently
  dropped); craft.rs:1206-1207 duplicated MELON_SLICE arm; datapack.rs:1324
  load-time whole-set clone; mobs.rs:4364,4490 vestigial always-true
  `restore` flag; mobs.rs:649-652 wither egg falls through to Chicken;
  dump_chunk.rs:44 ungated std::time in the example; vanilla_noise.rs:140
  gradient `hash % 12` non-uniformity (parity completeness note).

Gate for Round T: all six CI gates green + a native E2E run showing the
chunk borders VISIBLE (T5) and the water/lava/greedy-state/beacon fixes
vision-verified against fresh captures.

## Part 3 — Owner directives (RESERVED)

The rest of the overhaul plan is dictated by the owner after the full
sweep report is delivered. This section is intentionally empty until then —
no work is planned or executed from it.
