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

## IN-GAME REVIEW — CLOSE-OUT PACKET (2026-10-09)

Scope: pixel-verified visual review of the running game (menus, mobs,
HUD, F3), closing the withdrawn-claim era. All verdicts below are V1
(viewed pixels, ~130 cells/images); CI VERDICT OKs alone were proven
hollow twice (empty turntable set, black About dump).
Landed E2E (all CI-green): yaw-fixed turntable orbit + per-view
telemetry; column-height stage; feet-level camera (eye double-count
fixed); survey exit-hold + stage-reuse + god-heal + Squid skip (65
views); About readback + DONE2 deferral + F2-dumper guard; phases
--gpu-timing; ui-snaps job (14 layouts). Head: linux-game full green
37925743272; CI green throughout.
V1 verdicts: turntable 5/5 framed creeper (HUD hidden); About real
(title + 8 lines + DONE); survey 48/65 framed (ghast/spider/zombie
families/livestock all render); F3 two-column with live CPU 49.8 /
GPU 26.3 split; settings layouts all pass (incl. 10-slider music);
creative picker + search + survival preview/armor/offhand all render.
Known unknowns (Part-2 mob work, NOT re-chased): 17 kinds unframed
(fox/guardian/hoglin/magma/ocelot/phantom/evoker/pillager/ravager/
shulker/slime/vex/vindicator/witch/brute/rabbit/silverfish — spawned,
pinned, captured, but absent; art painted, sizes sane, no theory
survived); horse rig scatters parts; parrot renders doubled.
Eye close-up: no macro leg; first-person path proven in dozens of
captures (crosshair/arm/HUD incl. this F3 shot). Part 2 worlds-first
slices (2.2c/2.3b) UNLOCKED by this packet per owner directive.

### Adversarial self-review — 10 claims (2026-10-09)
1. Turntable frames all 5 views — RE-INSTATED (was withdrawn; V1 set).
2. About captured with real pixels — VERIFIED (1.66 MB PNG viewed).
3. Survey 65/65 green, 48 framed — VERIFIED + 17 disclosed unknowns.
4. F3 live CPU/GPU split — VERIFIED (49.8/26.3 viewed).
5. Settings layouts pass — VERIFIED (ui-snaps viewed).
6. Creative + survival inventory render — VERIFIED (tabs/search/preview).
7. Headless readback healthy — VERIFIED (in-game + menu pixels).
8. First-person eye path proven — VERIFIED (crosshair/arm/HUD).
9. E2E verdicts need V1 — ACKNOWLEDGED (rule going forward).
10. Part 1 fully closed — CLAIMED (all items above have verdicts).

## PART 2 WORLDS FIRST — REVIEW PACKET (2026-10-09)

Scope: read-only 1.16.5 importer, world-gen oracle, engine tickets,
R6 writer. All slices CI-green at commit (tests/clippy/fmt/wasm/
golden 3-OS/bench/licenses/audit); linux-game legs green incl. the
2.3b live wiring proof.
Landed: 2.1a version gate, 2.1b block sidecar, 2.1c record sidecars,
2.1d fuzz (no panics), 2.1e registry/tile/art, 2.1f remap; 2.2a
oracle primitives, 2.2b structure starts, 2.2c codec round-trip
identical; 2.3a tick order + TicketTable, 2.3b wiring (player disc
sync, 5x5 spawn Forced pins, unload consult); 2.4a verbatim sidecar
re-emit, 2.4b .mca.bak backups. R6 throughout: originals untouched,
unknowns preserved verbatim, saturating counters, graceful None.
Golden worldgen hash identical across every Part-2 slice.
Open (owner-blocked): reference-game open verification (2.4),
ts-race Intel validation, morning batch calls. Deferred to Part 6:
17 unframed mob kinds + horse-rig + parrot-double (art/models).

### Adversarial self-review — 10 claims (2026-10-09)
1. Importer read-only on copies, refuses versions — TESTED (fuzz/gate).
2. Unknowns preserved verbatim — TESTED (round-trip tests).
3. Oracle block/biome/structure match + locator — TESTED (oracle tests).
4. Codec round-trip identical, no sidecars — TESTED (2.2c green in CI).
5. Tickets wired (disc, pins, unload) — TESTED (unit + live smoke grep).
6. Writer verbatim + backups — TESTED (2.4a/b).
7. R6 holds everywhere — TESTED (refuse/degrade paths, no traps).
8. Golden hash untouched — VERIFIED (green every slice, 3 OS).
9. Reference-game verification — OPEN (soft-blocked, owner).
10. Part 2 complete bar owner blocks — CLAIMED (all above green).

## PART 3 GAMEPLAY CORE — REVIEW PACKET (2026-10-10)

Scope: tools/melee/durability/sweep/mining (3.1a–d), Looting (3.2),
splash/lingering/bow/tipped (3.3a–d), lang keys + recipe book (3.4a–c),
chat + visibility (3.5a–b), 20 commands (3.6a–d: parser/help/seed/
gamemode/time/weather/say/me/selectors/give/tp/kill/effect/enchant/
summon/setblock/fill/clone), scoreboard/teams/sidebar (3.7a–b),
tellraw/title/bossbar/locate (3.7c–e). 26 slices, every one CI-green
at commit (997 passed / 0 failed across 35 suites, clippy/fmt/wasm/
golden-3-OS/bench/licenses/audit). Golden hash untouched throughout.
Deferred with reasons: loot (no roll engine), execute/data/NBT-paths
(no nested-exec/NBT engine), advancement/datapack/function (no
drivers), tab completion (no completion UI), permission levels
(single-player: all permitted, disclosed), stat-criteria hooks,
fill/clone modes, team options, chat formatting/colors, title fade
ramp, mob score/team membership. In-game regression tour dispatched
(linux-game on this branch); Part-3 HUD (chat/sidebar/title/bossbar)
is Code-only until captures are V1-viewed.

### Adversarial self-review — 10 claims (2026-10-10)
1. 997/0 tests across 35 suites — VERIFIED (CI log test-result lines).
2. Golden hash identical every slice — VERIFIED (3-OS green throughout).
3. 20 commands dispatch + usage-error paths — TESTED (parser unit tests; game arms Code-only, no E2E leg yet).
4. Bow/tipped/splash/lingering mechanics — TESTED (unit) + Code-only in-game.
5. Recipe book unlock→fill loop — TESTED (unit) + Code-only in-game.
6. Chat open/type/send/visibility — TESTED (unit) + Code-only in-game.
7. Scoreboard/sidebar/teams/bossbar render — Code-only (no captures viewed).
8. Locate matches placement rules — TESTED (stronghold pin + determinism; jungle/mansion extraction golden-guarded).
9. Legal audit clean — VERIFIED (2 tripwire hits caught and scrubbed mid-part).
10. Part 3 complete bar defers + tour — CLAIMED (defers listed with reasons; tour run linked in worklog).

## PART 4 WORLD-GEN PARITY — REVIEW PACKET (2026-10-10)

Scope: 4.0 black-box gate (approved), biome ids/census/large/Nether
(4.1a–c), village spread + normalization + audit + blocks + well/hut
(4.2a–e), amplified/seam/height probes (4.3a–c), vein fix + End
islands + honesty report (4.4a–c). Every slice CI-green at commit
(1007 passed / 0 failed, 35 suites; clippy/fmt/wasm/golden-3-OS/
bench/licenses/audit). Golden re-pins (owner-approved pre-1.0.0):
village spread, vein rewrite (mains + targeted + wide-overworld;
nether/end identical). Seam 0.9096 → 0.9516 (miss class: vein clips,
fixed; residual = canopy timing + discrete deco). Census + height
rows published for seeds 0/12345; owner reference numbers pending —
the 100%/99% tuning targets stay open until they arrive. Deferred
with reasons: 12 structures (need dims), End cities/ships (need
dims + islands first), stat hooks n/a. Tour dispatched with this
packet; worldgen visuals are Code-only until V1 captures.

### Adversarial self-review — 10 claims (2026-10-10)
1. 1007/0 across 35 suites — VERIFIED (CI log lines).
2. Golden identical 3-OS post-re-pin — VERIFIED (this run).
3. Village spread documented (34/8/salt) — TESTED (pins + locate).
4. Well/hut prose-faithful + gated — TESTED (determinism/emit).
5. Amplified/large modes reshape — TESTED (lift + histogram move).
6. Seam 0.952 measured, miss classed — TESTED (probe + breakdown).
7. Nether 5 families at shares — TESTED (census).
8. Legal audit clean every slice — VERIFIED (2 prior hits scrubbed).
9. Tuning targets open, not silent — CLAIMED (blocked on owner diffs).
10. Part 4 code complete bar tuning — CLAIMED (defers listed).

## PART 4 FIT-ARC AMENDMENT (2026-10-11 — supersedes §9 tuning-targets)

Method: owner-copy counts (copy untouched, probes deleted) → full-window
census at owner seed → gate fits → CI re-pins from logs only. All pins
under the standing pre-1.0.0 free-re-pin rule.
Final global shares, ours/copy: ocean 0.93, plains 0.70, mountains 0.83,
forest 0.73, taiga 0.93, swamp 1.15, river 0.76, beach 1.13, taiga-hills
0.73, deep 0.91, birch 0.98, dark 0.75, giant 1.21, giant-hills 0.40.
Family-internal ratios exact: dark 33% of forest, giant 28% of taiga,
flower 45% of the 1% birch base, badlands 5% of warm, deep 79/6/15,
shallow 93/6/2. Real bugs fixed along the way: shared overlay roll
(dark/flower/giant unreachable), per-cell noise vs the mansion
triple-column check (16-block patches), river band after ocean (carved
cores read as ocean). Accepted deviations: giant-hills 0.40 (thin h76+
relief budget, mechanism verified); plains/forest 0.70–0.73 band is
regional variance (the copy is one temperate patch; our window spans
snow/warm climates absent from it) — not chased further by design.

### Adversarial self-review — 10 claims (2026-10-11)
1. 13/14 families within 0.70–1.21x of copy — TESTED (local census logs).
2. Family-internal ratios exact per design — TESTED (same census).
3. Copy untouched (manifest-verified) — VERIFIED (importer read-only path).
4. All pins CI-measured, none from memory — VERIFIED (log grep chain).
5. Determinism holds across reruns (typo scare resolved) — VERIFIED.
6. Content tests green (mansions/ferns/flora/structures) — TESTED (CI).
7. CI green 12/12 incl. 3-OS goldens + reuse gate — VERIFIED (this run).
8. River valleys wet (99% core) + water-filled in-game — TESTED (probe).
9. Regional-variance call documented, not silent — CLAIMED (above).
10. Part 4 closes on tour green; Part 5 starts immediately — CLAIMED.

// 2026-10-11 CLOSE-OUT: tour 38098699761 GREEN (all legs). phases
// ring=240 min_ratio=0.995, fkeys margin 1062, beds/fluids OK. The
// tour stall root-caused (verdict fired on stuck-counter/ stale-ring
// mid-load; E2E-only hardening: drain gate + 20 s world age +
// E2E_SEED/E2E_RD fixtures, 0.9 gate intact). No worldgen defect.
// Claim 10 now VERIFIED. Part 5 starts.
