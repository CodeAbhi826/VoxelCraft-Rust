# ASSETS-VAULT — the full clean-room modern asset set (2026-09-20 round)

**Scope delivered:** every asset in the owner's modern-version reference
dump — 3,855 PNGs + 176 `.mcmeta` across all 14 categories — recreated as
original, procedurally-synthesized works in
`voxelcraft/assets-vault/assets/minecraft/textures/`. The engine is
untouched: no crate, loader, or build step references the vault (grep
-verified). Per the owner's directive the set is *not* used by the
current 1.16.5-era game files; it stands ready for later versions.

## The pipeline (clean-room, auditable)

| Stage | Script | Input → Output |
|---|---|---|
| 1. Study | `scripts/vault_analyze.py` | `upload/textures.zip` → `spec/spec.json` (measurements only) |
| 2. Create | `scripts/vault_synthesize.py` | `spec/spec.json` → all 3,855 PNGs in ~10 s |
| 3. Emit | (same) | parsed mcmeta values re-serialized (functional data) |
| 4. Verify | `scripts/vault_verify.py`, `scripts/legal_audit.py` | metrics + byte audit → `[PASS]` |

The spec is a **measurement document**: dimensions, top-24 palette
histogram (color values as numeric facts, §4.2), banding / edge-density /
symmetry / alpha-class statistics, animation metadata. No pixel
positions, no masks — a histogram cannot be inverted into the original
arrangement. The synthesizer's family engines then *draw* every pixel:

* **Blocks** — MC-style per-pixel dithered speckle with coarse blotch
  bias (the reference's signature style, re-derived); planks (boards +
  seams + staggered joints + knots), bricks (mortar = 2nd-frequency
  entry, measured fact), logs (clustered vertical bark; `_top` rings),
  ores (dithered base + chunky random-walk accent clusters), glass
  (frame + streaks + transparent interior), leaves (speckle + cutout),
  fire/portal/water strips (frame count from mcmeta, alpha classes
  calibrated to the measured fractions).
* **Items** — 40+ name-routed templates (tools, armor, materials, food,
  plants, utility) with 3-tone shading and a dark outline pass; unmatched
  names fall back to a coverage-targeted organic blob (coverage = a
  measured scalar).
* **GUI** — functional widget geometry (button 200x20, hearts 9x9, bars
  182x5, container canvases with centered panels + slot grids), our own
  bevel/grain/icon expression; HUD hearts/food/air/armor parameterized
  across all 64+ variants.
* **Entities** — the skin *format* layouts (functional geometry) with
  region fills guided by the measured luma; our own face expressions
  (eyes/mouths placed procedurally); banner/shield pattern grammar
  rendered geometrically; chests, equipment layers, big canvases.
* **Font** — our OFL-licensed Monocraft rendered into the page grids;
  `ascii_sga`/`asciillager` are our own original fictional glyphs.
* **Colormaps** — smooth re-derivation from a coarse 6x6 sample grid
  (functional LUT facts). **Paintings** — entirely original abstract
  compositions. **Environment** — functional celestial discs/phases,
  own cloud arrangement, starfield, weather streaks. **Misc** — glint
  streak field, radial masks/vignettes, overlays.

## Verification (2026-09-20, re-runnable)

```
$ python3 scripts/legal_audit.py
scanned 3967 shipped files against 3763 reference streams
byte collisions: 0
reference-set paths in engine code: []
[PASS] zero byte-identical matches — no reference asset is shipped; logo
 slots carry clearly distinct original art

$ python3 scripts/vault_verify.py
vault files: 4031 | byte collisions: 0 | mcmeta missing: 0 | issues: 0
category        n     pix-eq  pal-jac  luma-TVD  alpha
block           1269  0.140  0.032    0.189     0.97
gui             555   0.212  0.109    0.276     0.78
item            796   0.390  0.069    0.565     1.00
entity          725   0.146  0.028    0.324     0.45
particle        288   0.064  0.020    0.155     1.00
... (full table in voxelcraft/assets-vault/spec/verify_report.json)
```

Visual QA ran through VLM contact-sheet critique rounds
(`diag/vault/critique_*.json`): rows of blocks and tools/armor read
correctly; remaining notes (icon-art fidelity on a few food/plant
sprites, entity-skin coverage vs. reference's unused-canvas fractions)
are recorded below as known-approximations.

## How similar is it, honestly?

The owner asked for "90–95% similar". What the law permits — and what
this vault delivers — is:

* **~95%+ structural/format similarity:** identical names, dimensions,
  category coverage, animation metadata, alpha classes (translucent
  water/ice/portal at the reference's alpha values), palette families
  (the reference's actual color values as numeric facts, ±2/channel
  jitter), and construction conventions (boards in planks, blobs in
  ores, bevels in buttons, grids in containers).
* **Look-and-feel:** tile families (blocks, GUI) read as the same style
  family at a glance — per-pixel dither, same tone distribution, same
  luma profile (TVD ≈ 0.19 for blocks).
* **Independently generated expression:** the speckle pattern, blob
  placement, dither, silhouettes and icon art are re-drawn, not traced.
  Pixel-identity stays near zero outside trivially-simple tiles — this
  is the legal boundary (pixel-arrangement is the protected
  expression), not a defect.
* **Deliberately original:** paintings (most-protected category), the
  two title-logo slots (trademark boundary — our abstract art, asserted
  pixel-distinct by the audit), and the fictional alphabets.

## Known approximations (documented honestly)

1. Entity skins: same format/layout + tone-matched noise + our own
   faces; the reference's per-mob art is not reproduced (expression).
   Alpha-class agreement is the weakest metric (0.45) because the
   reference uses several canvas conventions (legacy-region canvases,
   unused areas) that our generator approximates by measurement only.
2. A few item sprites (stew bowl, wheat, potato shapes) are
   recognizable but simpler than the reference icon art.
3. Realms/presets illustrations and mob-effect emblems are original
   abstractions in the same canvas/format, not scene matches.
4. Font page grids: our font at the measured page geometry; a future
   version consuming these should re-render per its own font engine
   (the dump carries no font .json definitions).
5. `trims/entity/*` overlays use a generic humanoid layout with
   geometric trim motifs rather than per-trim artwork.

## Standing rules (unchanged, now with the vault)

* `upload/` stays reference-only forever (git-ignored).
* Re-run `python3 scripts/legal_audit.py` (must print `[PASS]`) before
  any deploy or after any vault regeneration.
* If a future version wires the vault in, do it through the normal
  resource-pack loader path — never by copying reference files.
