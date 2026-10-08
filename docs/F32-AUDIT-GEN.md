# 1.0.4 — f32 sites in vc-world/src/gen.rs (feature placement) — Part 4 lookup

Census: 115 f32 mentions in the production region (lines < 5310; the
rest of the file is tests). Grouped by system, with the reference-game
parity question each one will have to answer in Part 4. NO CODE CHANGED
in this slice — this is the audit table the plan asked for.

## A. RNG draws (`rng.next_f32()`) — ~45 sites
vc-rng's next_f32 is our own integer→float conversion (fixed, not
libm), so these are already deterministic. The Part 4 question is never
"which libm" but "does the vanilla stage draw from the same stream in
the same order and with the same thresholds":
- cave worms: width/step rolls (1101, 1061 + pitch/yaw deltas)
- tree shape: lean/branch probability rolls (1632–1974)
- decorations: flower/mushroom/grass picks (2077, 2320, 2436, 2480,
  2586, 2619, 2772, 2802, 2819, 2826, 2847)
- dungeon/mob spawner rolls, desert well, igloo/pillager rolls
  (2973–2974, 3221–3266)
- village house ring jitter (4029: `rng.next_f32() * 0.6` angle jitter,
  `* 9.0` radius jitter)
- stronghold sector jitter (4716 area: `(rng.next_f32() - 0.5) * 0.5`)
- ravine top/depth rolls (4943)
- PARITY RELEVANCE: HIGH for all — vanilla worldgen's per-feature RNG
  streams (Java Random LCG, `setRegionSeed`/`setDecorationSeed` salt
  ladder) decide exact placement. Our hash3-per-position scheme is a
  DIFFERENT stream design; matching vanilla placement exactly (4.3/4.4)
  will require re-deriving the stream order per stage, not just the
  constants. Expect the oracle (2.2) to quantify this.

## B. Noise scaling constants (f32 simplex input scaling) — ~20 sites
- climate_fields (732–734): `x as f32 / 400.0` style scaling for
  temperature/humidity/continentalness/multifact
- mushroom fields gate (988, 1364, 1466): `/ 400.0`
- river width (1471): `/ 640.0`
- density_params (822–823), mountains mask, classify (865)
- PARITY RELEVANCE: HIGH — vanilla 1.16.5 uses the
  OverworldNoiseParameters octaves/scales (documented in the wiki's
  "Custom world generation" JSON, default_settings). Our scales
  (400/640/…) are [ESTIMATED / APPROXIMATION] today; Part 4.3 must
  replace them with the documented per-noise-parameter xz_scale values
  and switch the simplex to f64 (vanilla noise is f64). The f32→f64
  change WILL shift the golden hash — intentional re-baseline with
  Part 4's GO.

## C. Structural geometry constants — ~25 sites
- end island radius/taper (5087: dist < 60.0, / 30.0 taper)
- end pillar circle: TAU/10 spacing, 42.0 radius (5042–5051, 5140–5149)
- ravine width/depth (529–534: dx/dz/half_w f32)
- stronghold angle/dist bands (4746–4757: TAU/3, 1280.0..2816.0)
- village house ring (4029: TAU/n, r 10..19)
- nether fortress / bastion-ish grids (3904, 3961, 3987)
- shipwreck/ruin placement (4220–4231)
- PARITY RELEVANCE: HIGH where vanilla publishes the number
  (stronghold ring 1: 3 strongholds, 1280–2816 blocks [wiki/Stronghold —
  VERIFIED]; end pillars: 10 pillars r=42 [wiki/End — VERIFIED];
  village spacing 34/26 desert / 32/27 plains-style grids [wiki/Village
  structure spacing — must re-verify live in Part 4.2]). LOW where the
  number is our own placement heuristic inside authored templates.

## D. Simplex kernel internals — 2 constants + per-call math
- F2/G2/F3/G3 (242–243) + noise2/noise3 (312, 354): the classic
  Perlin/simplex skew constants.
- PARITY RELEVANCE: MEDIUM-HIGH — vanilla does NOT use simplex; it uses
  improved Perlin with 2^16-scale coordinate magnification (x/…·
  octave ladder). If 4.3 targets exact terrain identity, noise2/noise3
  get replaced by an f64 Perlin (vanilla's derivative approach); the
  simplex stays for our own extra layers only. [Code-only today; the
  swap is a reported golden-hash re-baseline]

## E. Wrapper functions (1.0.3 additions) — 5 sites
- dsin32/dcos32/dsqrt32/dround32 (256–283): deterministic libm
  shims. NOT parity-relevant by themselves (vanilla Java Math.sin is
  fdlibm-derived strictpath; Part 4.3 must compare our libm values vs
  Java's StrictMath on the actual argument ranges — both are portable
  pure implementations, so this is a checkable constant-level task).
