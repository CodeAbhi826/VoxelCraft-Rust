VOXELCRAFT FINAL MASTER PLAN (v4). Single source of truth. Read it fully at the start of every session, then read docs/AGENT-STATE.md. Refer to "the reference game" in repo files.

0. HOW YOU WORK (AUTONOMY PROTOCOL)
A1 Loop per slice: check for a STOP file in the repo root (gitignored; if present, make the tree clean, update AGENT-STATE.md and stop) -> read AGENT-STATE.md -> implement ONE slice (max about 300 changed lines) -> local cargo check/test -p <crate> only, no local release builds -> commit -> push -> wait for CI -> read the CI log -> only if green, append a worklog entry and update AGENT-STATE.md -> next slice. Never mark a slice done without CI green at that commit. Fix forward. Never re-baseline a pinned hash. Dispatch a workflow once, then wait (a manual dispatch collides with the push run).
A2 Continue through the Parts in order without waiting for a GO. Stop only at HARD STOPS.
A3 SOFT BLOCKERS (need my input or my own checks: sample worlds, numbers measured on my machine, art verdicts, in-game verification): record them in docs/BLOCKERS.md with the exact ask and a default, then continue with the next independent slice.
A4 HARD STOPS (stop everything, write docs/BLOCKERS.md, wait for me): (a) any possible breach or ambiguity under L1-L8 or any legal gate; (b) any destructive git operation (R1); (c) any change to a pinned golden hash; (d) CI red twice for the same cause after real fix attempts; (e) you are confused, looping, or near your context limit with uncommitted work (leave the tree clean, update state, stop); (f) a rate limit or tool failure that blocks progress; (g) anything that touches files outside the repo or any user world.
A5 CHECKPOINT at the end of every Part: append to docs/CHECKPOINTS.md and write a REVIEW PACKET (max 400 words): what landed with commit hashes, CI links with real test counts, measured numbers, images viewed and what you saw, open blockers, top 5 risks, and 5 claims I should spot-check with commands. Then do an adversarial self-review: re-verify 10 claims from your own docs against code and CI logs; fix any that fail. Then continue (unless a hard stop applies).
A6 docs/AGENT-STATE.md after every slice: current Part/slice, next 5 slices, open blockers, decisions made, landmines. It is the handoff for a new session or model.
A7 Rate or context limits: finish or revert the current slice, update AGENT-STATE.md, stop cleanly.
A8 CONTINUATION (owner directive 2026-10-08): do not end the turn after a slice — slice reports go into docs/WORKLOG.md and docs/AGENT-STATE.md, never into a closing chat message. After CI green + state update, immediately start the next slice from AGENT-STATE.md. A turn ends ONLY on a HARD STOP, a Part REVIEW PACKET, a STOP file, or a rate/context limit. On a HARD STOP the first line of docs/BLOCKERS.md reads "HARD STOP: <reason>".
A9 SLEEP-SHIFT (owner-declared, ~6h): never stop voluntarily inside the window. Always keep one background waiter armed (CI/quota/log watches) so completions resume work; do quota-independent work between notifications; batch questions for morning. Session suspension is covered by the state files — resume from AGENT-STATE.md.

1. TARGET AND POSITIONING
A clean-room recreation of the reference game Java 1.16.5 (DataVersion 2586, pack format 6), singleplayer, generic across CPUs and GPUs (the owner's low-end reference hardware is only the low-end test device). Aim to be better at: opening and saving real 1.16.5 worlds, vanilla-format resource-pack compatibility, a fast well-measured renderer (GPU meshing, FSR 1.0 only), vanilla behaviour backed by tests and a verified-vs-approximated ledger, and legal cleanliness with documented provenance. Non-goals: multiplayer, mods, Bedrock, other versions, FSR 2.0 or temporal upscalers, RNG-lockstep mob behaviour. Solo developer, public GitHub release.

2. RULES
L1 Never open, decompile or read the reference game's jars, class files, mappings or leaked source. The study corpus is off-limits permanently; report any path to it and do not open it.
L2 Never copy the rights holder's assets (textures, models, sounds, music, fonts, logo, splash texts, full lang files, advancement text). Procedural art is driven only by aggregate statistics, never per-pixel layouts or shapes.
L3 No rights-holder trademarks in project name, logo, repo description, window title or UI; a descriptive secondary mention with a "not affiliated" disclaimer is allowed.
L4 Asset modes: A built-in original art (the only art shipped); B import from the user's own game files at runtime, never bundled or committed; C third-party packs under their own licences.
L5 Facts are fine (mechanics, numbers, behaviour, file formats); expression is not (art, sounds, text, characters).
L6 Every committed data file has a provenance record.
L7 Other projects' mods, packs and code: behaviour documentation or optional runtime packs only, never copied; anything derived from the rights holder's art counts as theirs.
L8 Fidelity comes from aggregate measurements and original art direction; exact look comes from mode B.
V1 After any slice that changes textures, models, lighting, shaders or UI, capture screenshots and report exactly which images you viewed and what you saw; never claim visual correctness for an image you did not view.
R1 No history rewrites or force-pushes without my explicit confirmation of that exact operation.
R3 Estimates come from velocity measured in this repo's history, as ranges; no whole-game totals in weeks.
R4 Keep the honest provenance statements in LEGAL.md and docs (a study corpus was used for fact-level counts and aggregate statistics). Never scrub them.
R5 Determinism: world generation uses exact IEEE arithmetic on the CPU in the numeric type the reference behaviour requires; no FMA contraction, no fast-math; own fixed implementations (libm) of sin/cos/pow/exp/log; golden hashes identical on Linux, Windows and macOS. SIMD only if each value's operation order is preserved.
R6 User worlds: never modify an original; work on copies; keep automatic backups; preserve unknown data verbatim; corrupt input never panics.
R7 Tag every claim Verified / Tested / Code-only / Unknown. Test counts come from CI log "test result:" lines only. One slice per commit. Never pipe test output through tail or head in a way that hides the exit code.
P1 I am on a data-collecting free model tier: never ask me for reference captures, sample worlds, jars or reference material; I give numeric results only. No secrets, tokens or personal files in your work.

3. CURRENT STATE
Done and verified: Phase 0 batch; Part 0; slice 1.0 (libm on all 11 production transcendental sites; world-gen golden hash 3 seeds x 9 chunks x 3 dims, passing on ubuntu, windows and macos; f32 census in docs/F32-AUDIT-GEN.md); AGENTS.md rules; font and home-path fixes; parked/round-b branch.
Landed but only code-read: 1.1 (phase meter spans update+draw, unit test only) and 1.2 (--benchmark streaming; not in CI).
Last known CI: run 37731681836 at f0d346a, 916 passed / 0 failed / 2 ignored.
Measured: 127.5 ms avg per frame on the reference hardware at 720p in a static scene (MDI path); interactive frames of 2 to 5 s while streaming (before 1.1 fixed the meter blind spot); CI-runner gen about 11 ms per chunk (runner numbers, not reference-hardware numbers).
Absent: tool/weapon registry, commands, scoreboard, recipe book, graph pathfinding, lang keys, real-world import, same-seed parity.
Landmines: Chunk::get trap (about 150 positional sites remain), greedy-key bitfield mirrored in WGSL, monolith files (game.rs 28k lines), CI quirks (clippy is lib-only; match CI exactly).
Art verdicts: REDESIGN ghast, Nether portal, TNT, default player (palette plus torso/face mapping bug), Enderman (eye colour/shape). PENDING turntables: Creeper, Ender Dragon, Wither, Blaze, Piglin, villager, zombie villager, iron golem. Also: leaf texture is per-pixel noise, water is oversaturated cyan, crosshair colour inconsistent, confirm HUD icons come from our pipeline.

4. DECISIONS ALREADY MADE (use these defaults; do not ask)
- Vanilla mode at native: AA off. Upscaling: Native (FSR fully off, pass skipped), Ultra Quality 1.3, Quality 1.5, Balanced 1.7, Performance 2.0, Custom scale slider; FSR 1 only; render size = round(output / scale) per axis from the real output size (any aspect ratio), recomputed on resize/fullscreen/DPI change; UI at native after the upscale; sharpness control named "Sharpness" (never RCAS).
- AA: Off or ONE method: implement SMAA, measure on the reference hardware at 720p; if above 3 ms ship FXAA instead. No MSAA, no TAA. AA is forced on, before the upscale, whenever render scale < 1.0.
- No GPU world-gen; CPU speedups only, identical output.
- Structure interiors: author original templates by default.
- World-gen parity order: biomes, structure positions, terrain, then carvers and decoration. Stop and report instead of tuning to approximate.
- Internal ids stay; display names come from a data table.
- Player skin layout is mandatory; mob texture layouts are our own until decided.
- Round B stays parked until Part 5.
- Smaller, well-made original entity roster first.

5. PARTS (execute in order; independent slices may be interleaved while a soft blocker is open)

PART 1 PERFORMANCE CORE
1.0.5 Determinism coverage (approved): per-function pinned-value tests (exact to_bits) for each libm site (cave_worms_near, place_ores, village_houses, strongholds, ravines_near_chunk, end island sqrt, end pillars and end_pillar_tops, find_spawn); widen the golden hash to 5 seeds x 25 chunks per dimension plus targeted pins at village, stronghold, ravine and ocean chunks (forced where needed), in the existing 3-OS job.
1.1v Runtime proof: an E2E leg in the smoke run asserts sum(phases) >= 0.9 x frame_ms and prints a VERDICT line; show the F3 phase line in a screenshot (V1).
1.2v Add the streaming benchmark as a CI step under xvfb with lavapipe or software Vulkan (non-gating, JSON artifact with avg, p99, max frame ms, chunks generated). Have CI upload the game binary as an artifact so I can download and run it on the reference hardware (soft blocker for my numbers; never build a release locally).
1.3 Work budgets first: main thread never blocks on gen/mesh; per-frame time budget scaled to measured machine speed; workers = cores-1 (min 1) at low priority; nearest-first; cancel stale jobs when the player moves; cap uploads per frame. Target: no frame over 100 ms while streaming and 30 fps steady on the reference hardware at defaults (I measure; report honestly).
1.4 CPU world-gen speedups with IDENTICAL output (coarse-grid noise with interpolation, per-column caching, no per-block allocation, order-preserving SIMD, thread sizing). Golden hash change = HARD STOP.
1.5 GPU timestamp queries enabled whenever supported (flag --gpu-timing); never auto-disabled per adapter.
1.6 Upscaling per section 4. 1.7 AA per section 4. 1.8 Capability probe feeding defaults for any CPU/GPU plus CPU-side knobs (render distance, entity distance, particle limit, gen threads); Vanilla mode keeps the reference defaults; F3 shows CPU ms and GPU ms separately.
1.9 Offload audit (report only): classify per-system CPU work as visual-only / exact-integer / gameplay-or-double (CPU only), with estimates flagged.
1.10 Re-measure the matrix and streaming path; verdict CPU-bound or GPU-bound; CI perf gate on the JSON including max frame time.
1.11 Turntable capture tool (one character alone at about 3 m on flat ground, HUD hidden, neutral daylight; front, side, back, 3/4, close-up; CI artifacts). 1.12 Fix the player model torso/face mapping bug.
Interleave: Chunk::get migration batches B to G, compile-verified.
Part 1 ends with a REVIEW PACKET.

PART 2 WORLDS FIRST
2.1 Read-only importer for 1.16.5 worlds (a copy): level.dat, region/DIM-1/DIM1, chunk NBT with 1.16 packed BlockStates, palettes, heightmaps, biomes, tile entities, entities, structure starts; unknown blocks/items/entities preserved and shown as placeholders; refuse other versions; fuzzing; no panics. Develop first on synthetic fixtures built from public format documentation; real worlds are a soft blocker.
2.2 World-gen oracle harness over imported regions: per-chunk block and biome identity, structure position match, first-mismatch locator, seam test between imported and newly generated chunks; commit only numbers.
2.3 Engine/server architecture: 20 TPS fixed tick order (documented and tested), chunk tickets, spawn caps and despawn, light update queue.
2.4 Writer: worlds the reference game opens; unknown data preserved; atomic writes; backups. My verification in the reference game is a soft blocker.
REVIEW PACKET.

PART 3 GAMEPLAY CORE
Tool/weapon/armor registry (data-driven where cheap), mining speeds, durability, sweep, enchantment completion (Looting, anvil prior-work), tipped and lingering potions, recipe book, chat and commands (parser, selectors, NBT paths, execute, data, scoreboard, give, summon, fill, clone, setblock, tp, gamemode, time, weather, effect, enchant, loot, locate, tellraw, title, bossbar, team, advancement, datapack, function; tab completion; permission levels), scoreboard and teams, lang keys. REVIEW PACKET.

PART 4 WORLD-GEN PARITY
4.0 LEGAL GATE (HARD STOP): write a proposal of a black-box method under L1 (I generate worlds in my own copy and give you only numeric diffs; you implement from public documentation and behaviour; list every constant you would need and its source; no code ported from community reimplementations built from decompiled sources). Wait for my approval.
4.1 Biomes (default, large biomes, amplified, Nether, End): target 100% on oracle seeds. 4.2 Structure positions (spacing, separation, salt): target 100%. 4.3 Terrain shape and surface builders: target at least 99% blocks; report the seam score. 4.4 Carvers and decoration: report honestly; may remain approximate with my approval. REVIEW PACKET.

PART 5 COLOUR, LIGHTING, PACK COMPATIBILITY, BUILT-IN SHADER
Colour-space audit (document every sRGB/linear conversion); vanilla lightmap and per-face shade, light free-down rule, smooth AO, sky/fog/clouds/moon phases/weather darkening, biome tints (colormaps from public per-biome colour values), .mcmeta animation, pack loader to format 6.
2B pack compatibility: asset-path contract table (every path read, size, UV region, fallback, mode A replacement, public source per row); GUI/HUD layout compatibility at vanilla coordinates (widgets, icons, hotbar, hearts, XP, boss bars, containers, advancement frames, recipe book); grass/leaves tint overlays and colormap lookup from packs; model and item overrides; font providers, lang JSON, sounds.json with .ogg; player skin layout (64x64, 64x32 upgrade, slim arms); optional OptiFine-style compat layer from public docs only with a Supported/Partial/Unsupported matrix; shader packs staged (composite/final, then shadow, then gbuffers) with readable failures, correctness tested on CI, fps judged only on my hardware.
Built-in optional shader (after the benchmark harness): off by default; independent toggles for soft shadows, bloom, POM, PBR, SSR, SSAO, volumetric light, vignette/grain, grading presets; cost shown per toggle; Low/Medium/High/Ultra tiers sized from measured numbers. Shadows OFF in Vanilla mode.
Colour-accuracy harness: I run comparisons against my private reference captures on my machine and give you numbers only (soft blocker). REVIEW PACKET.

PART 6 ART AND MODELS
Original art direction; JSON block/item model system with display transforms, blockstates, overrides; entity ModelPart hierarchies and animation rules; player classic/slim; item and block render forms (GUI, held, dropped, frame/stand/head, placed, block entity); redesigns per section 3 (change at least 3 of silhouette, face/pattern, proportions, palette, signature pose; keep role, hitbox and ids); fix leaves, water, crosshair; entity table (model, animation, AI, tests) kept current. Use turntable captures; V1 applies. REVIEW PACKET.

PART 7 REMAINING GAMEPLAY AND CONTENT
Redstone gaps (button, tripwire, rails, piston block entity, dispenser behaviours), rails/minecarts/boats, per-shape collision, graph pathfinding, villager schedules, raids, beacon, fishing, spectator, crawl, missing GUIs, block long tail (prune vault art for content newer than 1.16.5), data-driven content (recipes, loot tables, tags, advancements, datapacks and /reload), audio/lang/menus (event-based sounds, positional audio, original built-in sounds and music, rebinding with conflict detection, stats and advancement screens, F3 vanilla-style fields). REVIEW PACKET.

PART 8 QUIRK PARITY
Documented quirk spec with public citations (update order, scheduled ticks, piston events, quasi-connectivity, comparator and observer timing, update suppression, explosion behaviour); contraption oracle with per-tick traces (my numbers, soft blocker); statistical tier for mob timings; game rule vanilla_quirks on by default; crash-class quirks reproduced safely with no panic. REVIEW PACKET.

PART 9 RELEASE
Packaging, crash reports, provenance report for every shipped file, third-party licence list, final legal sweep, screenshot audit, README claims re-verified. FINAL REVIEW PACKET.

6. REPORT FORMAT
Per slice: what changed, commit hash, before/after. CI link with real counts. Images viewed and what you saw. Claims tagged. What you could not do. Limitations. Keep it concise.

END OF PLAN
