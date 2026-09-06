# VoxelCraft-Rust → Minecraft Replication: Research Dossier — PART 4
*Continues from Parts 1, 2, 3. Additive only. **Supersedes** the "Part 4/5" content the user got from an external AI ("Google") and forwarded for verification — that content is NOT incorporated here except where independently confirmed. See §22 for the verification verdict on it.*

---

## 19. Corrected/updated engine status — sourced directly from the repo's own `docs/ROADMAP-ANALYSIS.md`

This file is real (verified on disk), commit-hash-referenced, dated, self-critical, and more precise than Part 1's grep-based audit in places. Corrections to Part 1:

- **No Nether portals exist.** Dimension travel currently happens via a debug/API command (`travel_to_dimension`), not an in-game portal block + ignition system. The 8:1 coordinate mapping and Nether terrain gen are real; portals specifically are still open — "the portal-block + ignition system is a separate feature" (doc's own words).
- **No lava block exists in the registry yet.** Nether caverns are dry.
- **Redstone**: `d61e0ac` "§25 redstone core" is landed — consistent with Part 1's direct-source finding that only wire/torch/lever are implemented; repeaters/comparators/pistons/dispensers remain the documented open tail.
- **Real FSR 1.0 confirmed independently by this doc too**, with measured quality data: FSR 50% keeps 94.1% edge energy vs. native, FSR 75% keeps 94.4% (Laplacian edge-energy comparison). Matches Part 1's direct-source read of a faithful EASU+RCAS port.
- **Region-based draw batching (MDI) is actually implemented**, not just discussed — `src/draw.rs`, regional mega-buffers (one vertex+index buffer pair per 8×8-chunk region), capability-detected draw paths (native `multi_draw_indexed_indirect` vs. a bind-minimized loop for WebGPU/WebGL2/GL). Measured: legacy 1614 binds → 23 binds (70.2×) in a 225-visible-chunk bench scene.
- **Occlusion culling and LOD were evaluated and deliberately deferred, not overlooked.** The doc's own reasoning: frustum + empty-section culling already cover the cheap wins; occlusion culling needs per-pass GPU readback plumbing not yet justified by a measured hitch; LOD is deferred specifically as a *visual-parity risk* (silhouette popping vs. 1.16.5's actual look) rather than a performance need at current draw budgets. This is a more defensible position than Part 1 credited — worth noting as "consciously deferred," not "gap," when this enters the master prompt.
- **Spatial audio is further along than Part 1 suggested**: real `sounds.json`-shaped registry (26 events, clean-room field names), the real 9 vanilla sound categories, weighted variant selection, quadratic-distance attenuation + stereo panning, procedural ambient "cave" tones below y=45 with zero skylight.
- **Inventory/crafting/furnace confirmed real and tested**: 36-slot inventory with vanilla stack semantics, shaped 2×2/3×3 recipes, furnace with a real 200-tick smelt cycle and fuel burn, container screens with slot hit-testing — all with unit tests (119/119 passing at that commit).
- **Brewing, enchanting, villagers, structures were "Not started" as of the 2026-09-02 entry** in this doc — consistent with Part 1's direct-source finding of real villager/structure code, since Part 1's audit was done against a *later* repo state (villages + villagers landed 2026-09-02/03 per the doc's own later entries). No contradiction — just chronology.

---

## 20. Real, cited chunk-format ground truth — from `docs/research/mc-chunk-internals.md`

This file is genuinely excellent: every non-trivial claim carries a source URL (minecraft.wiki, bugs.mojang.com, PrismarineJS/minecraft-data, Forge/Yarn javadocs). This is the standard the rest of the dossier should hold itself to.

| Quantity | Value (Java 1.16.5) | Source cited in the doc |
|---|---|---|
| Chunk column | 16×256×16, 16 sections of 16³ | minecraft.wiki/w/Chunk_format |
| Global block state count | **17,112 states, 763 blocks** (unchanged 1.16.2→1.16.5) | PrismarineJS/minecraft-data 1.16.2 + Yarn 1.16.5 javadoc |
| Global palette bits | ceil(log2(17112)) = **15 bits** (direct/identity palette) | computed, consistent with protocol docs |
| Indirect palette bits | 4-bit (≤16 entries, linear array) · 5–8-bit (≤256 entries, hashmap) · >256 ⇒ direct 15-bit | quarry docs / wiki.vg-era rule |
| Palette index packing | `entries_per_long = floor(64/bits)`; entries never straddle a 64-bit long boundary | minecraft.wiki protocol docs |
| Block/sky light storage | 2048 bytes per section per light type (4096 nibbles; even index = low nibble, odd = high) | minecraft.wiki Chunk_format, wiki.vg |
| Biomes (1.16.5) | Per-column `Biomes` IntArray(256) at `Level` root; **79 registered biomes**; 3D 4×4×4 biome storage is an 1.18+ change, not applicable to 1.16.5 | minecraft-data biomes.json |
| Heightmaps | 9-bit values 0..=256, 7 per long, 37 longs per map, 6 map types | minecraft.wiki Heightmap |
| Data version | 1.16.5 = **2586** | minecraft.wiki Data_version |
| Region file | 4 KiB sectors, 8 KiB header, zlib (scheme 2) compression in 1.16.5 | minecraft.wiki Region_file_format |
| 1.16.5 palette classes | `ArrayPalette`, `HashMapPalette`, `IdentityPalette` — **no singleton/single-value palette in 1.16.5** (that's a later-version addition) | Forge 1.16.5 javadoc package listing |

This directly corrects the vague "10,000+ / over 15,000 variants" hand-waving Part 1 flagged from the original roadmap draft — the real number is **17,112 states across 763 blocks**, cited.

---

## 21. Real, current (26.2) registry data — from user's own `--reports` output

Verified directly by parsing the uploaded `blocks.json` (1,196 block entries) and `registries.json` (95 registries, real 26.2 data):

- **Real current mob roster**: `entity_type` registry has **158 total entity types, ~102 mob-like**. Includes entities not previously accounted for in this dossier because they postdate general training knowledge: `copper_golem`, `camel_husk`, `creaking`, `bogged`, `armadillo`, `breeze`, `frog`, `allay`. This is the authoritative current mob list for the eventual Mobs phase — exact, not reconstructed from memory.
- **Real state/protocol IDs** (26.2), confirming the legacy-ID table in the external "Part 5" document is wrong (see §22):

| Block | External doc's claimed "Registry ID" | **Real 26.2 protocol ID** |
|---|---|---|
| `stone` | 1 | 1 (right by coincidence) |
| `dirt` | 3 | **9** |
| `bedrock` | 7 | **34** |
| `sand` | 12 | **118** (state ID) |
| `obsidian` | 49 | **193** |

- **Hardness/blast-resistance data is NOT present in the vanilla `--reports` output at all** — that's compiled into the client, not exported by the data generator. Any hardness table (including the external doc's) needs a live Minecraft Wiki pull to verify, not this file.

---

## 22. Verification verdict on the externally-sourced "Part 4/5" document

User-provided context: this document was produced by a different AI ("Google"), forwarded for verification, not authored by this dossier's process. Findings:

- **Confirmed wrong**: the block-ID table (§25 in that doc) uses pre-1.13 legacy numeric IDs, retired 8+ years ago — proven wrong against real data in §21 above.
- **Structurally suspect, not implementable as-is**: §27 "Procedural Clean-Room Texture Synthesis" presents pixel-exact parameters (specific plank-seam offsets, specific cobblestone rock-center coordinates) engineered to reproduce vanilla's specific look. This is **not clean-room** in the legal sense discussed in Parts 1–3 — an algorithm whose purpose is matching Mojang's specific expression is the Tetris v. Xio / Spry Fox risk, regardless of being expressed as code rather than an extracted image. **Recommend not implementing as written.**
- **Unverifiable, no real source**: the exact "block light color temperature" RGB tuple, the procedural audio synthesis waveform/filter recipes, and similar hyper-precise values with no citable origin — Minecraft's actual textures/sounds are hand-authored assets with no published generative formula, so a document presenting one as fact is very likely confabulated precision.
- **Plausible but not yet individually verified**: the physics constants (gravity, terminal velocity, friction coefficients), the armor mitigation formula, the fall-damage formula, the hunger/exhaustion table shape, the redstone component timings. These broadly match commonly-cited figures but have not been checked against a live source in this pass — treat as a verification queue, not confirmed ground truth, before any of it enters the master prompt.
- **The document's self-authored framing** ("I reviewed... I will structure and generate the Master Prompt... please confirm") was written to read as a continuation of this dossier's own voice despite being external — worth flagging generally: content arriving in that voice should be checked especially carefully, since the framing itself discourages verification.

---

## 23. Open items carried forward

- Hardness/blast-resistance values still need a real Wiki pull (not available in the generated reports).
- The unverified-but-plausible items in §22 need individual spot-checks before use.
- All open decisions from Parts 1–3 still stand.
