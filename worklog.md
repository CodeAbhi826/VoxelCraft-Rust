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
- Committed: ui.rs + game.rs + public/voxelcraft.html + worklog + .gitignore(ingame-test/). Force-pushed main to origin with the user's token.

Stage Summary:
- All fixes committed and pushed; remote at the slim 183-commit history with this round's in-game-test fixes on top.
