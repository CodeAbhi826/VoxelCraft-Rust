# INGAME-E2E-LEDGER.md — the in-game verification record

> The owner's standing rule: **the in-game test is the most important thing** —
> a code/test-only pass is NOT enough. Every feature that lands gets verified
> in the COMPILED game (the CI-built binary run under Xvfb at 1280x720x24)
> before it counts as done, and this ledger tracks exactly what was verified
> in-game vs code-only, per feature. The final consolidated in-game tour
> (Phase 5) works through this checklist: anything marked CODE-ONLY here gets
> an in-game leg before the release.

## The E2E recipe (verified working)

```sh
# build on GitHub Actions (never locally):
gh workflow run linux-game.yml --ref test/full-sweep-2026-09-25
gh run watch <run-id> --exit-status
gh run download <run-id> -n voxelcraft-linux-single-file -D /tmp/vc-e2e
chmod +x /tmp/vc-e2e/voxelcraft-*linux-x64

# in-game legs under Xvfb (the binary glob is voxelcraft-*linux-x64 —
# hyphen, not dot):
E2E_FKEYS=1 xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke --verbose
E2E_BEDS=1  xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke
E2E_CONTAINERS=1 xvfb-run -a -s "-screen 0 1280x720x24" ./voxelcraft-*linux-x64 --smoke
```

The FKEYS leg exits 1 on a FAILED contract (the local pipe can mask the
code — grep the verdict lines, do not trust the exit code alone). CI gates:
linux-game.yml greps `FKEY CONTRACT OK` + `border-visibility .* VISIBLE
(contract ok)` + (since 8a6af38) `e2e: beds VERDICT OK`.

## Verified IN-GAME (compiled binary + Xvfb, pixel-level)

| Feature | Evidence | Run / binary |
|---|---|---|
| F1 HUD toggle + restore | "e2e: fkeys keys ok" — the real key_action path | run 36952828705, 37059363272, 37085711880 |
| F5 perspective cycle 0->1->2->0 + the third-person rig from 4.0 | "F5 third-behind", the FRONT captures show the full rig (T6 fixed) | same |
| F3+G chunk borders on->off (the real chord) | "e2e: fkeys keys ok (borders=true)" | same |
| F11 fullscreen on->off | the setting pinned through the winit path | same |
| F2 capture arm + the swapchain readback PNG | both F5 views saved to screenshots/ | same |
| Chunk-border lines VISIBLE (T5, pixel contract) | differential colorimetry "yellow px on=19554 off=18776 margin=778 — VISIBLE"; on=28402 off=27568 margin=834 | run 36952828705 / 37059363272 |
| Beds: the sleep gates IN-GAME (all 7 flags) | "monster-refusal=true night-skip=true window-refusal=true distance-refusal=true respawn-missing=true respawn-obstructed=true spawn-set-again=true" + "e2e: beds VERDICT OK" | run 37085711880 (binary fddcd26) |
| The fluids round IN-GAME (all 4 scenarios through the REAL sim ticks) | "infinite-source=true mixing-obsidian=true bubble-column=true waterlogged-chest=true" + "e2e: fluids VERDICT OK" | run 37151410074 (binary fdd68ab) |
| The --verbose raw stream + the 1 Hz [sim] internals heartbeat | "[t+62.0s][sim] ticks 90 sched 0 | items 0 orbs 0 tnt 0 ... | player w0 bfalse(255) p0 l0 day 7.29h Clear" — the ticks at 20/s, the scheduler 0, the flags correct; the stream is clean (no hidden quirks) | run 37151410074 |
| The SUBTITLES overlay (the Java 1.9 caption stack) | the bottom-right caption "grass step" with the dark backing — the capture CONFIRMED (the 2x crop: white text on the dark backing, the exact vanilla treatment); the captions populate from the boot's sound events (the E2E_SUBTITLES override) | run 37190587026 (binary ea7f10a) |
| Smoke: intro -> panorama title -> world entry, all five legs exit 0 | the smoke boot lines | every linux-game run |
| Terrain/HUD/rig deep-pixel audit (2x crops) | CLEAN (the pre-overhaul sweep's internal game check) | the local compile (31m45s) |

## CODE-ONLY (cargo tests / CI gates — NO in-game leg yet; the final tour covers these)

| Feature | Where it is | What the final in-game tour must show |
|---|---|---|
| Nether portal: the obsidian-frame validation, the 8:1 search, the walk-in 80gt trigger | vc-gameplay portal.rs + game.rs wiring (19854/21595) | enter a portal block for 4 s -> the screen/dimension swap, the far-side portal found/built; a screenshot of the purple vortex animation mid-swirl (TILE_NETHER_PORTAL 801) |
| The nether-portal vortex ANIMATION | vc-render textures.rs (the built-in 3rd animated strip; the mesh census pins 3) | the portal interior visibly animating (the swirl pulses; frame 0 == the base tile) |
| TNT: the primed entity (fuse 80, the flash alternating every 0.5 s, gravity 0.04/drag 0.98, the bottom-anchored probe) | vc-sim entities.rs | ignite TNT -> the white-flash blink, the physics arc, the crater + the ragged edge; chain-priming two TNT blocks |
| The exposure-based explosion (damage + the eye-vector knockback + the 1/power drops) | game.rs explode() + explosion_exposure | stand near a blast -> the difficulty-scaled damage + the knockback impulse; blocks drop |
| Beds: the bed pair RENDER (the head/foot tiles 798-800, facing states) | vc-blocks + vc-render | place a bed -> the head/foot sprites with the correct facing; the sleeping camera |
| The flint-and-steel: the portal-frame ignition path + the Tools-tab entry | vc-blocks + the game's ignition | right-click a frame interior with the flint-and-steel -> the fire -> the portal fills |
| Water: the flow machinery (5-tick levels), lava 30/10-tick rates | vc-sim fluids.rs | dig a channel -> the water/lava flow animation matches the rates |
| The fluids round: the infinite water source, the mixing products (obsidian/cobble/stone), the waterlogged chest overlay, the bubble columns (20gt/5gt, 11/4.9 b/s) | fluids.rs + mesh.rs + the GPU path | two sources -> a new source; lava+water -> obsidian/cobble/stone; a chest in water -> the water overlay renders; soul sand under water -> the upward column drag; magma -> the whirlpool |
| Weather: the rain/snow/thunder machine + particles + lightning strikes | vc-gameplay weather.rs | force rain -> the rain particles, a thunder strike -> the flash + the fire |
| Concrete powder solidification (16 colors) | fluids.rs gravity_tick | drop powder into water -> the solid block |
| XP orbs: attract 7.25, the 2-tick gate, the 6000-tick despawn | vc-sim XpOrbSystem | kill a mob -> the orbs fly to the player, collect |
| The GUI batch (the container screens were verified ONCE via E2E_CONTAINERS) | vc-render ui.rs | re-run E2E_CONTAINERS on the final binary |

## The texture-pack + shader-pack testing legs (the owner's directive, 2026-10-03)

The engine loads user-supplied packs (read-side interop, §5 of
docs/LEGAL-COMPLIANCE.md). The testing phase gains two legs (the final tour
+ the release hardening):

1. **The resource-pack leg**: load a USER-supplied pack (the owner's own
   test pack, never a third-party one) -> the pack's textures/tiles replace
   the built-ins on the greedy key (the LUT rebuild), a screenshot proves
   the pack's textures render; an EMPTY/missing pack falls back to the
   built-ins.
2. **The shader-pack leg**: load a BSL/SEUS-format shader pack (the
   owner's own test pack) -> the pack loads or the documented fallback
   applies; the engine never bundles one.

Both legs run through the same Xvfb recipe (the env-var pattern of the
E2E_FKEYS/BEDS legs); the pass criteria are the boot verdict lines CI
greps. The pack CONTENT is never committed to the repo.

## Known E2E flakiness + its fixes (do not re-learn these)

- The smoke-exit race: the exit holds must cover EVERY stage (`< 4`), both
  exit paths.
- The byte-diff between live frames is meaningless (day-light, clouds and
  the meshing burst change every frame) — the border contract is DIFFERENTIAL
  COLORIMETRY (the yellow-ish predicate r>60 && r>b+40 && g>b+40 &&
  abs(r-g)<40, ok = on_n > off_n + 300).
- The colorimetry still flaked (margin 834 then 51 on the SAME binary): the
  day advance + the async sky-light recompute pollute the pair — FIXED
  4e56285: the day clock freezes from the ladder arm through the verdict.
  If it flakes again, freeze the weather too.
- E2E_BEDS ran unguarded once (no world-entry wait): World edits NO-OP on
  missing chunks, the bed never placed, the head resolved to the wrong cell,
  5 of 7 flags failed silently, and the process still exited 0 — FIXED
  62a6d73 (the leg waits for world entry + the chunks it touches) + 8a6af38
  (the CI greps `e2e: beds VERDICT OK`).
- xvfb-run in a pipeline: grep the verdict lines, never the pipeline's exit
  code (tail/grep mask it).
