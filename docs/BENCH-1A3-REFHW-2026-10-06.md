# 1A.3 — real wall-clock bench on the reference hardware (2026-10-06)

Slice deliverable: verify shadows-OFF-by-default in Vanilla mode (a code no-op —
the default was already `shadow_quality: 0`, `Settings::default`, game.rs) and
measure the engine per setting on the real target hardware against the
**30 fps @ 1280×720 goal**.

## Hardware / environment

reference low-end CPU (2 cores @ 1.10 GHz), Intel reference iGPU iGPU (Mesa the other ICD, Vulkan
1.4.354). Release binary built from commit `eb44519` (the two commits after it
are docs-only and cannot affect the binary). Headless X: `xvfb-run -a -s
"-screen 0 1280x720x24"`, `WGPU_BACKEND=Vulkan`.

## Method

Each config ran in a scratch working directory with a crafted `options.txt`
(the engine reads `options.txt` from the CWD; first-run defaults never clobber
an existing file):

```
defv=2;rd=<rd>;sd=<sd>;vsync=0;shadowq=<sq>;upscale=<up>;
```

Everything else is the shipped default (aniso 1/OFF, msaa 0, mip 4, occlusion
on, graphics fancy, MDI draw path). `vsync=0` forces the wall clock to be the
limiter. Command:

```
xvfb-run -a -s "-screen 0 1280x720x24" env WGPU_BACKEND=Vulkan \
  target/release/voxelcraft --benchmark frames=300 warmup=60 seed=12648430 json=bench.json
```

Stats window: the engine's rolling 240-frame ring
(`FramePhases::new(240)`), so every run reports exactly 240 measured frames —
the last 240 of the 300 measured. Deterministic scripted orbit camera, fixed
seed 12648430. Raw JSON per config: [`docs/bench/1a3-2026-10-06/`](bench/1a3-2026-10-06).

## Results (all times in ms; fps = 1000 / avg)

| Config | avg | median | 1%-low avg | worst | avg fps | JSON |
|---|---|---|---|---|---|---|
| rd=12, shadows OFF, FSR off (engine defaults) | 120.9 | 123.7 | 352.9 | 355.6 | 8.3 | [base.json](bench/1a3-2026-10-06/base.json) |
| rd=12, shadowq=3 | 199.8 | 206.5 | 401.7 | 402.6 | 5.0 | [shadow3.json](bench/1a3-2026-10-06/shadow3.json) |
| rd=12, upscale=1 (75% FSR) | 118.6 | 120.4 | 291.2 | 305.0 | 8.4 | [fsr75.json](bench/1a3-2026-10-06/fsr75.json) |
| rd=12, upscale=2 (50% FSR) | 105.8 | 106.3 | 402.1 | 478.3 | 9.5 | [fsr50.json](bench/1a3-2026-10-06/fsr50.json) |
| rd=6, shadows OFF, FSR off | 90.4 | 92.9 | 159.5 | 160.6 | 11.1 | [rd6.json](bench/1a3-2026-10-06/rd6.json) |
| rd=4, shadows OFF, FSR off | 78.8 | 81.7 | 144.1 | 148.9 | 12.7 | [rd4.json](bench/1a3-2026-10-06/rd4.json) |

Every run exited 0 and reported the MDI draw path.

## Verdict against the 30 fps goal

**Not met on this hardware.** The best measured configuration at the default
render distance is 105.8 ms avg (≈9.5 fps); even rd=4 is 78.8 ms (≈12.7 fps)
— roughly 2.4× the 33.3 ms frame budget. This confirms and extends the
2026-10-05 hardware caveat: the reference iGPU is a 2017 low-power iGPU and the
result is a *hardware floor for the current renderer*, not a finished-engine
ceiling. Nothing hung or panicked; all six runs exited 0.

Per-setting findings:

* **Shadows OFF by default (vanilla mode) is correct and matters**: the
  opt-in `shadowq=3` costs **+65% avg frame time** (120.9 → 199.8 ms) at
  rd=12 — shadows re-render the scene from the light's view.
* **FSR upscale helps, modestly**: 50% upscale is −12.5% avg (120.9 →
  105.8 ms), 75% is within noise of baseline. The draw phase is ~100% of the
  frame (draw 120.9 of 120.9 ms avg) and is dominated by CPU-side
  command-building/mesh-upload work rather than fill rate, so resolution
  scaling alone cannot reach the target.
* **Render distance is the strongest lever measured**: rd=12 → 6 is −25%,
  rd=12 → 4 is −35%.
* The 1%-low/worst values (≈300–478 ms) are mid-run chunk-mesh/upload hitches
  inside the measured window; they are recorded, not discarded (the §1A.1
  wall-clock metering change covers the F3 display; the bench ring has always
  counted every frame).

## What would close the gap (out of scope for 1A, recorded for the plan)

The draw phase's CPU-side cost on 2 slow cores is the wall. Candidate levers,
in the order the measurements above motivate them: threaded meshing already
exists (rayon) but upload submission is serial per frame; reducing per-region
re-binds; a lower-cost default preset (rd≤6 + upscale=2) as the "Low" preset
for weak hardware; GPU-timestamp profiling (1A.2) to split mesh-upload vs
draw submission inside the draw phase.
