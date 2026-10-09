# OFFLOAD-AUDIT.md — per-system CPU/GPU classification (slice 1.9, report only)

Rule: VISUAL-ONLY may move to the GPU; EXACT (bit-identity required)
and GAMEPLAY (sim-visible state) stay on the CPU. A GPU double of a
GAMEPLAY system needs a bit-exact parity proof first. Estimates are
flagged [ESTIMATED]; measured numbers cite their artifact.

## Measured baselines (CI runner, NOT reference hardware)
- Headless bench (run 37777978599): gen 8.43, mesh 7.83, remesh 1.95,
  sim 0.015 ms; light init 1147 ms/96 chunks (~12 ms/chunk);
  drawprep 2.54 µs.
- Streaming walk, lavapipe (run 37772129669): frame avg 83.2 ms —
  sim 0.08, stream 17.2, results 17.0, ui 0.01, draw 65.8 ms;
  scene GPU 8.9 ms vs CPU frame → CPU-bound on the runner.
- Series note: gen 11.44 → 10.96 → 8.43 ms across runs (1.4 landed
  between); runner variance is ±20%, so no single delta is claimed —
  the direction is consistent, the proof waits for reference-hardware numbers.

## Classification (code refs are the current tree)
| System | Cost | Class | Notes |
|---|---|---|---|
| Terrain density lattice + trilinear fill (`gen.rs`) | ~8–11 ms/chunk | EXACT | R5: f64/f32 op order + libm; golden hashes pin it. CPU only. |
| Structures/features/ores (`gen.rs` emits) | inside gen | EXACT | Gameplay-visible blocks; same determinism contract. CPU only. |
| Greedy meshing, CPU path (`mesh.rs`) | ~8–13 ms/chunk | VISUAL-ONLY | Rendering data only; GPU path already bit-identical (hybrid). |
| GPU compute mesher (`gpu_mesh.rs`) | — | (already GPU) | 2-strike watchdog → CPU fallback. |
| Light init + pump (`light.rs`) | ~12–20 ms/chunk | GAMEPLAY | Levels feed spawning/growth; a GPU mirror needs exact-BFS proof first. |
| 20 Hz sim: scheduler, fluids, redstone, mobs, block entities (`vc-sim`) | ~0.02–0.04 ms/tick | GAMEPLAY | Tick-ordered sim state. CPU only. |
| Mob AI/steering, drops (`mobs.rs`, `entities.rs`) | inside sim | GAMEPLAY | RNG-lockstep is an explicit non-goal; stays CPU regardless. |
| Particles spawn + vertex build (`vc-particles`, game layer) | [ESTIMATED] <0.5 ms/frame | VISUAL-ONLY | Visual seeds only; GPU particles possible later. |
| Item-icon baking (`item_icon_cache.rs`) | inside ui ~0.03 ms | VISUAL-ONLY | 4 bakes/frame cap; GPU render-to-texture possible later. |
| Occlusion flood + frustum cull (`draw.rs`) | inside drawprep µs | VISUAL-ONLY | Already GPU-assisted; flood stays CPU (integer graph walk). |
| GUI rebuild + canvas upload (`ui.rs`) | ~0.01–0.03 ms | VISUAL-ONLY | Small; no action. |
| Streaming orchestration (stream/submit/apply) | 11–130 ms under load | CPU STRUCTURAL | Scheduling, not compute; budgets (1.3) bound it. |
| Save/journal/anvil I/O | I/O-bound | CPU | Correctness (R6) dominates; never the hot path. |
| Audio mixing (rodio) | negligible | CPU | No action. |
| Atlas generation at boot | one-time ~1 s | CPU | Fine as-is. |
| EASU/RCAS/FXAA/composite/shadow/sky/clouds | GPU (8.9 ms scene) | (already GPU) | 1.5 timestamps cover the scene pass. |

## Conclusions for 1.10+
1. The frame is CPU-bound by worldgen + lighting + mesh-apply on every
   measured tier — the exact/gameplay classes. No further GPU offload
   is available without breaking bit-identity or sim determinism.
2. The remaining lever is cheaper CPU work (1.4-class transforms) and
   tighter budgets (1.3-class), judged on reference hardware — 1.10.
3. Particle/icon GPU migration is the only visual-only work left, and
   it is sub-millisecond today: explicitly not worth it before 1.10.
