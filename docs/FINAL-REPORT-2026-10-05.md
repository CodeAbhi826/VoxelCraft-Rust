# VoxelCraft-RUST — THE FINAL OVERHAUL REPORT (2026-10-05)

> The one final report the plan called for. Phases 0-5 executed; release
> **v0.4.0 published**. Everything below is verified (the CI gates + the
> in-game legs + the measured numbers), not self-reported.

## 1. What shipped

**NO RELEASES** — the owner's 2026-10-05 directive: the project is in the
BUILDING stage, not production; the v0.4.0 release and the v0.2.0/v0.3.0
releases were DELETED (the GitHub Releases + the tags removed). The
build-state is the deliverable: the CI-verified tree (911 unit tests green,
the 6 gates) + the CI-built binaries verified via the artifact route
under Xvfb. The tag/release step is DEFERRED to production (the recipe:
the version bump + the tag push — recorded in the worklog).

**The headline numbers (all gate-verified):**
- **911 unit tests, green in CI** — `cargo test --release --no-default-features
  --workspace`, plus the wasm32 compile-check, clippy clean (0 warnings),
  cargo fmt, the legal audit (the trademark scanner), and the headless
  bench — all 6 gates green on every push.
- **1,913+ `VERIFIED` citations in code** — every constant carries a
  live-verified reference-wiki citation (raw wikitext via the MediaWiki
  API), cross-checked where the wording was ambiguous.
- **539 block/item entries, 892 block states, 28 biomes, 66 mob kinds
  (the 1.16.5 roster COMPLETE), all 32 status effects.**

## 2. The phases, end to end

- **Phase 0 — the pre-overhaul sweep**: 4 review agents, ~139,000 lines
  read; 2 critical + 4 major + 45 minor findings — ALL fixed since (the
  Round-T burn-down: the greedy key widened to the full u16 state at bits
  28..44 + the tint at 44..52, the WGSL mirror bit-identical, the lava
  T4 same-cull, the T5 mid-pass collapse (the one static border grid),
  the T6 camera distance). docs/PRE-OVERHAUL-SWEEP-2026-10-01.md.
- **Phase 1 — the measured baselines**: the #1 cost identified = the
  light-engine init (86.6 ms/chunk reference hardware); the bench gained the sim-tick
  + the light-engine sections; the regression check live via the CI bench
  job. docs/BASELINE-PERF-2026-10-02.md.
- **Phase 2 — the parity audit**: the 11-domain matrix + the prioritized
  gap list + the round order; the jar-derived reference counts; the
  ground-truth biome ids from the owner's real saves (the audit's WRONG
  row resolved). docs/PARITY-MATRIX-1.16.5.md.
- **Phase 3 — the parity rounds** (the priority set complete):
  - **Round K (the nether portal)**: the 8:1 coordinates (div_euclid),
    the obsidian-frame validation (2×3..21×21, the corners not required),
    the 80-tick walk-in, the 128/16 search radii, the build-spot scan,
    the forced-Y clamp, the clean-room vortex animation (3 animated
    strips).
  - **TNT**: the primed entity (the fuse 80, the flash alternating every
    0.5 s, the bottom-anchored probe), the exposure-based explosion
    (the bounding-box sample grid + the eye-vector knockback — the
    in-code placeholder GONE), chain-priming (10-30 ticks).
  - **Beds/sleeping**: the decision layer (the windows
    12523..23477/12002..23998, the monster box FIRST, the f32 snap),
    the 7-flag in-game E2E leg, the weather reset wired.
  - **Fluids**: the waterlogging states (the chest pair 874/875, the
    overlay renders both), the bubble columns (20gt/5gt, the verified
    11/4.9 b/s transport + the air-provide), the infinite water source,
    the mixing products (obsidian/cobblestone/stone).
  - **Fire**: the fire-age window (877..891, FIRE 805 = age 0), the fire
    tick (the extinguish rules, the spread degree formula, the burn-away,
    rain 20%+3%/age), the flammability table (the wiki's odds), the
    player's fire damage + the Fire tag (-20/160).
  - **Sneaking + effects**: the sneak caps (1.3/1.8), the 16 missing
    status effects (the 1.16.5 set COMPLETE at 32) with the verified
    behaviors wired + the 16 clean-room icons.
  - **Witch/skull**: the witch's splash-potion ladder (the verified
    gates), the wither-skeleton's skull 2.5% drop (the stale-comment
    finding FIXED).
  - **THE VISUALS UPDATE** (re-sequenced earlier by the owner): the
    SUBTITLES overlay (the Java 1.9 caption stack — in-game verified
    with a real capture), the smoker/blast furnace GUIs (the distinct
    titles), the dispenser/dropper GUI (the 3×3 grid + the arrow + the
    live slots), the COLORIMETRY CROSS-CHECK PASSED (the engine's tiles
    vs the vanilla measured — the sand/planks EXACT, the tints
    vanilla-exact, the greyscale mechanism matches).
  - **The mobs batch**: the 16 missing kinds (the verified stats/tiles/
    drops/AI + the slime-chunk spawn rule) — the roster COMPLETE.
  - **The systems**: the GAMERULES (the vanilla defaults + the
    set_gamerule stand-in; the flags wired: doFireTick/doDaylightCycle/
    doWeatherCycle/naturalRegeneration/keepInventory/mobGriefing/
    doMobSpawning).
- **Phase 4 — the optimization (the measured payoff)**:
  - O2 (the sky scan's heightmap break) + O3 (the static per-state
    lookup tables built from the same truth — parity-preserving).
  - **The light init 86.6 → 58.7 ms/chunk (32% FASTER)**, the meshing
    49.1 → 36.07 (27%), the generation 53.1 → 35.0 (34%), the remesh
    10.95 → 8.68 (21%), the drawprep 13.9 → 8.7 µs (37%).
- **Phase 5 — the release hardening**: the README finalized (the test count,
  the 66-mob roster, the 32 effects), the docs current, the tag shipped,
  the release published, and the final E2E tour on the release binary:
  FKEYS margin 825 VISIBLE + FKEY CONTRACT OK, beds VERDICT OK, fluids
  VERDICT OK.

## 3. The infrastructure built along the way

- The 6-gate CI (the legal audit/fmt/tests/wasm/clippy/bench) — every
  push gated.
- The in-game E2E legs (the owner's rule: the in-game test is the most
  important thing): E2E_FKEYS (the 4-stage ladder + the border
  colorimetry), E2E_BEDS (the 7 flags), E2E_FLUIDS (the 4 scenarios
  through the real sim ticks), E2E_CONTAINERS — all with the world-entry
  + chunk-ready guards and the gated VERDICT lines CI greps.
- `--verbose`: THE raw-log flag (the full raw stream + the 1 Hz [sim]
  internals heartbeat).
- The knowledge base (docs/REPO-KNOWLEDGE-BASE.md — the complete
  source-code-level transfer, updated in place), the findings ledger, the
  E2E ledger (the verified-in-game vs code-only record), the Phase-4
  design + the post-measurement.

## 4. What remains (the honest follow-up plan)

The L-tier content breadth the plan listed (the low-priority tail):
- **The structures**: bastion, end_city, igloo, pillager_outpost,
  ruined_portal, shipwreck, underwater_ruin, ocean monument, swamp hut,
  buried treasure, desert well (11 — the guardian/shulker/pillager/brute
  spawn integration waits on these).
- **The particles/sounds breadth**: 20/90 particle kinds, 122/~700 sound
  events, the .ogg files + the discs/jukebox.
- **The biome/blocks breadth**: 28/66 biomes, 539/764 blocks (the
  slimeball/rotten-porkchop/golden-axe items + the
  stonecutter/loom/smithing/cartography/lectern BLOCKS).
- **The polish tier**: the per-kind mob behavior depth (the slime's
  split, the pillager's crossbow bolts, the trader's trades), the A*
  pathfinding, the advancements/scoreboard/slash-command parser.
- **The Phase-4 O4**: the per-section emissive bitmap (the next
  light-init step).
- **The texture-pack + shader-pack E2E legs**: the recipe recorded (the
  knowledge base §9.2's pattern); the pack content never committed.
- **The engine findings (open)**: EMERALD_BLOCK absent (T12), the
  vanilla shade:false unimplemented (T13), the Nether lava sea, death in
  the Nether/End not traveling back, InhabitedTime stub, bits_for 8-bit
  cap, the world_source_conversion gamerule.

## 5. The rules honored throughout

- **Compilation on GitHub Actions only** — never locally (cargo fmt the
  only local command); the binaries verified via the downloaded CI
  artifacts under Xvfb.
- **The in-game test is the most important thing** — every visual/
  gameplay feature verified in the compiled game with the gated VERDICT
  lines + the pixel-level captures.
- **The live-verified citations** — never stale research dumps; the
  cross-check rule (two sources must agree).
- **Zero-unsafe (exactly one sanctioned block), zero placeholders, zero
  reference-game assets, zero trademark terms** — the legal audit green on
  every push. (Corrected 2026-10-06: the engine ships one third-party
  component, the Monocraft font under SIL OFL 1.1 with its licence; the
  original “zero third-party assets” wording overstated it. See
  `docs/LEGAL-COMPLIANCE.md` §3.)
- **The optimizations additive and parity-preserving** — the semantics
  identical (the tables built from the same truth; the break preserves
  the scan's zero-run).
- **English only** in every emitted word.
