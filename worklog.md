# VoxelCraft Worklog

Shared multi-agent work log. Append-only. English only.

---
Task ID: 1
Agent: main (Super Z)
Task: Crash recovery — rebuild toolchain + WASM bundle, restore the web preview

Work Log:
- Sandbox reset wiped: ~/.cargo (rustup, wasm32 target, wasm-bindgen), voxelcraft/target, wasm-out/, public/ bundle files. Git history (183 commits, slim) survived.
- Reinstalled rustup (rustc 1.98.1) + wasm32-unknown-unknown target.
- Downloaded prebuilt wasm-bindgen 0.2.127 (x86_64-unknown-linux-musl) to ~/.local/bin.
- cargo build --release --no-default-features --target wasm32-unknown-unknown --lib (2m31s), wasm-bindgen --target web, patch-wasm-glue.py, deployed to public/ (voxelcraft.js 136KB, voxelcraft_bg.wasm 5.4MB) + rsync'd builtin packs.
- Verified: all bundle URLs return 200 on localhost:3000.

Stage Summary:
- Preview restored; build recipe verified end-to-end from a cold toolchain.

---
Task ID: 2
Agent: main (Super Z)
Task: Full in-game (not code) test campaign — boot the game in a real headless browser and walk every menu/settings/world/mode path

Work Log:
- Installed agent-browser (npm) + Chromium; viewport 1280x720.
- Built a VLM-driven visual verification loop (z-ai vision on screenshots) + used the game's own E2E bridge (window.__vcCmds break/place/probe/give, window.__vcStats live state, boot-log console lines) for ground truth.
- VERIFIED WORKING: studio splash → title menu (splash text, buttons) → Select World (list, search, rows) → Create World p1/p2 (4 game modes with vanilla descriptions, WORLD TYPE DEFAULT/SUPERFLAT, structures, bonus chest) → world gen (superflat + default: plains y=65, oak trees, flowers, tall grass) → HUD (hearts/hunger/hotbar/creative-correct) → pointer lock lifecycle (incl. release on GUI open, re-lock on close) → mouselook + WASD + F3 debug overlay (full vanilla-parity data: XYZ/Block/Chunk/Facing/Light/Biome/CH caches) → block breaking (crack overlay, particles, survival timing) → 3D drop entities + vanilla feet-box pickup semantics (items in a 1-deep hole below the box floor are correctly NOT auto-collected) → inventory add + stack counts → give path → world persistence across full page reloads (position, inventory, containers) → drowning (air 15s then 2 dmg/s) → bubbles bar → underwater tint + surface line → SWIM-UP (Space rise verified with correctly-coded synthetic key events) → water flow (source → flowing states on wall break) → jump + gravity + fall + edge-standing physics → pause menu → creative flight (double-space toggle, Space rise, Shift descend) → creative HUD (no hearts/hunger).
- TEST-HARNESS PITFALLS DOCUMENTED (NOT game bugs): agent-browser `keydown "Space"` dispatches e.code="" (only `press` sends codes; single-char keys are saved by the shim's codeOf fallback) — all "dead keys/no swim" observations traced to this + input-eating death screens + Chrome's pointer-lock re-entry cooldown after Esc (headless synthetic clicks lack the required activation).
- REAL BUGS FOUND + FIXED THIS ROUND:
  1. 720p bottom-row clipping: every settings screen's DONE (y=470) and the world screens' CANCEL/row-B (y=480) fell off the live canvas at 1280x720 (auto GUI scale 3 -> 853x480 canvas; layouts authored for the 960x540 reference). FIX: anchor_y() in ui.rs preserves bottom-edge distance; applied to all 20+ bottom-row widgets. Regression test added (bottom_rows_anchor_into_short_canvases). LIVE-VERIFIED: DONE + CANCEL + EDIT/DELETE/RE-CREATE/SEARCH all visible at 720p.
  2. First-person view model invisible ("Steve right-hand missing"): the anchor was (-0.42,-0.36,0.55) — LEFT side, cube half below the bottom edge, arm ~95% below (NDC y ~ -0.94..-1.9). FIX: ax=+0.42 (vanilla bottom-right), enlarged the bare-arm box (y ay-0.55..ay+0.22, 0.20 wide) + raised the equipped-arm box. LIVE-VERIFIED: bare arm (skin + teal Steve sleeve) and held block both render bottom-right.
  3. HUD bleeding through GUI overlays: crosshair/hearts/hunger/world-hotbar drew under the open inventory. FIX: hud_hidden guard (container/picker open) in game.rs — vanilla hides the in-game HUD under GUI screens; F3 overlay intentionally stays.
  4. Shim pointer-lock promise gap: requestPointerLock() rejections that fire NO pointerlockerror event (e.g. NotAllowedError without user activation) left the drag-look fallback unengaged — canvas clicks dead until reload. FIX: the promise .catch in public/voxelcraft.html now routes into the same fallback as the event handler.
- Minor notes (no action): world list shows placeholder seed text in the seed field; superflat world type only (DEFAULT/SUPERFLAT — documented scope); the drag-look tap-break path was not conclusively verified in-headless (real pointer-locked clicks, the primary path, are verified working).

Stage Summary:
- The user-reported "bugs" (water/swim/inventory/3D drops/hand/breaking) are all confirmed FIXED in the latest build — the remaining real defects found were the 720p layout clipping, the invisible view model, the HUD overlay bleed, and the shim lock-catch gap, all fixed and live-verified this session. Test evidence: /home/z/my-project/ingame-test/*.png (57 screenshots).

---
Task ID: 3
Agent: main (Super Z)
Task: Test suite + git deliverable (commit, push)

Work Log:
- cargo test --workspace --no-default-features: 727+ tests green across vc-blocks(43) vc-chunk(7) vc-inventory(6) vc-particles(5) vc-audio(5) vc-sim(62) vc-gameplay(289) vc-render(193) voxelcraft(90, incl. the new anchor regression test) vc-world non-gen(17). ZERO failures.
- Known-slow (pre-existing, untouched this round): vc-world::gen statistical worldgen tests (~60 tests, 60s+ each on 2 debug cores) — not run to completion; several sampled individually green in earlier partial runs.
- Repo state: history ALREADY slim (7.73 MiB pack, 183 commits — the prior session's rewrite). No filter-repo needed. Bundle files in public/ are untracked (CI builds them) — correct policy.
- Committed: ui.rs + game.rs + public/voxelcraft.html + worklog + .gitignore(ingame-test/). Pushed main (1838d5d) to origin with the user's token — the remote already carried the slim history at 9100a78, so this round fast-forwarded on top. Remote verified via ls-remote: main = 1838d5d.

Stage Summary:
- All fixes committed and pushed; remote at the slim 183-commit history with this round's in-game-test fixes on top.

---
Task ID: 4
Agent: main (GLM-5.3-Flash, god-tier preset)
Task: Pre-overhaul sweep — CI gates to GitHub Actions, internal game check with vision, full code review (~141.7k lines), findings ledger

Work Log:
- Full workspace report: 15 crates (14 vc-* + app), manifest matches the preset spec, git clean, era-locked constants intact with VERIFIED citations, 863 tests, zero placeholders, zero non-English text, exactly one unsafe block (game.rs, // SAFETY:-commented).
- Local compile run CANCELLED at the owner's instruction — all compiling gates moved to GitHub Actions.
- cargo fmt applied (rustfmt 1.9.0 reflow, 77 files, 12,910 insertions, zero logic change); fmt gate green locally and in CI.
- ci.yml: added fmt + clippy jobs (neither gate ran anywhere before), then the legal_audit.py gate (LEGAL-COMPLIANCE.md §6 claimed CI ran the scanner — it never did; the doc's audit_tools.py name was stale, the real scanner is legal_audit.py).
- Clippy burn-down via CI feedback (clippy stops at the first failing crate; each fix reveals the next): vc-sim type_complexity (CuboidFace type alias); vc-render manual div_ceil / unit let-binding / chunks_exact_to_as_chunks / too_many_arguments (RenderFrame params struct, render.rs + the single call site in game.rs); voxelcraft collapsible_match + collapsible_if; vc-render single_element_loop (shaderpack fsh scan). RESULT: ALL SIX CI gates GREEN (run 36868455480: fmt 18s, wasm 38s, bench 2m17s, test 4m17s, legal audit 36s, clippy 1m0s).
- Internal game check with vision: release binary built (31m45s); discovered the working headless recipe — `--smoke` is REQUIRED to enter a world (E2E_FKEYS alone sits at the title forever); ran on the REAL Intel UHD 600 via Vulkan/Mesa (10-19 fps, 1280x696); FKEY contract OK, exit 0, 2 screenshots (behind + front third-person). Deep-pixel audit (2x crops + deterministic tools): terrain/HUD/player-rig/font CLEAN — no quad seams, no atlas bleeding, no z-fighting. TWO FINDINGS: (V1, major) the F3+G chunk-border overlay renders NOTHING despite borders=true — the 25-box loop (render.rs ~5936-5969) rewrites one LineUniform via queue.write_buffer BETWEEN pass.draw() calls of one render pass; wgpu applies write_buffer before the whole command buffer executes, so all 25 boxes collapse to the last offset — fix with the engine's own instance-rate origin_vb pattern; the E2E contract checks flags were armed, not that lines are VISIBLE (contract gap). (V2, major) the F5 front camera distance is 1.5 (game.rs ~22645) vs the behind view's 4.0 — vanilla's front view sits at ~4 blocks; at 1.5 the face fills the screen (measured). Both ledgered in docs/PRE-OVERHAUL-SWEEP-2026-10-01.md, NOT fixed ad hoc.
- Full code review, 4 review agents, every line read:
  - Agent 1 (core: vc-blocks/vc-chunk/vc-mesh/vc-nbt/vc-rng/vc-inventory/vc-particles — 20,083 lines): 2 CRITICAL (C1: the greedy solid key decodes only 8 state bits at bit 28 while packing 16 — every world-stored state >= 256 renders the WRONG tiles (basalt 738 -> 222 Hopper, stained-glass-white 400 -> 144 Repeater, anvil 283 -> 27 Iron-Ore) and merge keys collide across states sharing a low byte; C2: same key layout — for tinted states >= 256 the state's bits 8-9 OR into the tint slot, foliage renders with the water tint); 2 MAJOR (M1: `above == b` compares a raw state id against the WATER BLOCK id — flowing water 89..=95 never matches, waterfalls render at (8-level)/9 instead of full sheets; M2: LAVA has no face_visible special case and fluid culling is WATER-only — every lava-lava boundary emits double faces, interior 0.875 stripes render inside lava bodies); 12 minor (same-block culling for V2 stained glass/SLIME, tautological Arc::ptr_eq assertion, el_affects_ao ignoring its parameter, stale id-window docs, unused serde in vc-nbt, modulo bias in vc-rng::next_range, build_mesh_inputs ~1.7 MiB per call).
  - Agent 2 (sim/game: voxelcraft app + vc-sim — 32,865 lines): 0 critical/major, 10 minor (stale module docs fluids/redstone, elytra comment vs 2.5 constant, dead double-XOR in Sim::new/XpOrbSystem, std-integer-keyed maps in production code, f3_held stuck-flag window across Esc-to-Pause, bench-only unwraps).
  - Agent 3 (render: vc-render — 42,471 lines, 36 files): 0 critical/major, 10 minor (PARTICLE_SHADER samples the atlas with NO tile-safe inset/mip-cap — the bleed guards terrain/water carry are missing there; std-HashMap convention deviations in regions/occlusion/item_icon_cache/ui icon snapshots; stale gpu_mesh.rs STATE_COUNT doc (236 vs actual 863); dead bookkeeping in gui/loader.rs, dead snap helper in bin/ui_snapshots.rs). The mid-pass write_buffer collapse class scanned across EVERY pass — only instance is the known V1.
  - Agent 4 (world/content: vc-world/vc-gameplay/vc-pack/vc-anvil/vc-audio — ~43,628 lines, 43 files): 1 MAJOR (T11: is_base_block accepts only iron/gold/diamond while its own citation lists iron/gold/emerald/diamond/netherite — emerald/netherite beacon pyramids cannot be built), 13 minor (SSV ghast roll 4.9% vs cited 2.5%, duplicated set_count condition dropping bare-form functions, duplicated MELON_SLICE arm, load-time whole-set clone, std-integer-keyed maps contradicting the crate citation, gen.rs mojibake, vestigial restore flag, wither egg falls through to Chicken, ungated std-time in the dump example).
- Legal: scripts/legal_audit.py run — 1 violation (docs/WORKLOG.md carried the publisher's built-in art-pack label) — reworded — [PASS] clean. The audit gate now runs in CI on every push.
- docs/PRE-OVERHAUL-SWEEP-2026-10-01.md created: the durable findings ledger + working recipes + session state, updated as results land; the overhaul plan builds on it.

Stage Summary:
- The engine gates are ALL GREEN on GitHub Actions (6 jobs); the game runs and renders clean except the two ledgered visual findings; 141.7k lines reviewed by agents 1-3 with 2 critical + 4 major + 45 minor findings ledgered (all four agents reported). NOTHING was fixed ad hoc — C1/C2/M1/M2/T11/V1/V2 wait for the overhaul plan (docs/OVERHAUL-PLAN-2026-10-01.md Round T).
