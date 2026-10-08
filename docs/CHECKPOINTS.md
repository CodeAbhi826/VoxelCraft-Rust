# CHECKPOINTS.md — Part review packets (PLAN-FINAL §0.A5, ≤400 words each)

## Part 1 PERFORMANCE CORE — REVIEW PACKET (2026-10-09)

Landed (code): 1.0.5 libm pins + wide/targeted hashes (`30ad6a8`); 1.1v
phase proof (`29de86b`+`1ad23a4`); 1.2v streaming bench + step
(`871d28f`+2 fixes); 1.3 budgets (`8b4011c`+2 fixes); 1.4 cell-major fill
(`8e0fcd6`); 1.5 timestamps (`89a935a`); 1.6 upscale ladder (`6b237ad`+fix);
1.7 FXAA (`d9f0425`); 1.8 probe/F3/knob (`a7f233e`); 1.9 audit
(`OFFLOAD-AUDIT.md`); 1.10 gate (`567c915`); 1.11 turntable (`7f47f74`+fmt);
1.12 player sprite (`333b766`); 1.8b preset/tiering (`7006495`); Chunk B–G
(`7576870`,`398002c`,`b2d9f63`,`94bbf84`,`ac15a8f`,`de69a25`); docs/state
throughout. CI: 9/9 code-green every slice (tests/clippy/fmt/wasm/golden
3-OS); linux-game legs green per slice (FKEYS margins 513–597, beds/fluids/
phases/turntable VERDICT OK). Measured: streaming avg 268→52–83 ms
lavapipe; scene GPU 9–31 ms; gen −4% (inside noise); verdict CPU-bound.
Images viewed: F3 phase row (live, V1). Open V1: turntable set, player rig,
engine screen, F3 split (quota-blocked PNGs; foreground session covers all).
Open blockers: SMAA-vs-FXAA 3 ms call (needs reference numbers);
quota-gated formal close-outs; branch rename proposal awaiting decision.
Top risks: lavapipe≠hardware numbers over-read; E2E leg count growth vs
job time; migration double-fold sites left untouched (noted, unproven
harmless); FXAA quality unjudged (no eyes yet); artifact quota discipline.
Spot-checks (run these): `gh run view <id> --log | grep -a "PERF GATE OK"`;
`grep -rn "chunk\.get(" voxelcraft/crates/*/src/*.rs` (expect only trap
tests/internals); `cargo test -p vc-world --lib golden` (12 green);
`grep -a "capability:" smoke-*.log`; `python3 -m json.tool` on any
streaming-bench.json (camera/world/gpu_ms keys).

### Adversarial self-review — 10 claims re-verified (2026-10-09)
1. Phase meter spans update+draw — VERIFIED (`begin_frame` AboutToWait,
   `end_frame` end of draw; E2E min_ratio 0.983–0.988 in logs).
2. Streaming JSON carries camera/world/gpu_ms — VERIFIED (downloaded
   artifacts parsed).
3. Narrow golden 9 values never re-baselined — VERIFIED (GOLDEN const
   diff vs 1.0.1 commit: identical).
4. F3 shows phase row + CPU/GPU split — VERIFIED (code + viewed capture).
5. Turntable covers 5 views solo — VERIFIED (code + VERDICT OK + byte sizes).
6. Dispatch caps (4,4)–(16,16) by workers — VERIFIED (unit test green in CI).
7. Upscale ladder 6 modes + migration 1→1, 2→4 — VERIFIED (unit test green).
8. FXAA shader validates + runs pre-upscale — VERIFIED (naga test green;
   legs green with pass in chain).
9. Player torso cyan + symmetric eyes — VERIFIED (pixel test green in CI).
10. Zero positional production `Chunk::get` remain — VERIFIED (final sweep:
    only trap tests/internals/NBT gets).
No failures. Two honest corrections folded in above: lavapipe GPU variance
(~3x) stated, gen −4% stated inside noise.
