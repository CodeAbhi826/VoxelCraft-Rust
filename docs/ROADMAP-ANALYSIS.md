# Roadmap Full-Read Audit & Evaluation

**Source reviewed:** `upload/VoxelCraft-MC1.16.5-Transformation-Roadmap.md` — 91,832 bytes / 1,339 lines, read in full, top to bottom.
**Audited against:** actual codebase at commit `f08c21a` (2026-09-01).
**Purpose:** (1) confirm full coverage, (2) map the internal duplication, (3) judge whether the roadmap's claims are right, (4) record true implementation status.

---

## 1. What the file actually is

It is **four pasted LLM outputs concatenated**, not one document:

| Section | Lines | What it is |
|---|---|---|
| **A** | 1–158 | First draft — condensed file-by-file assessment (blocks/chunk/world/mesh/textures/sounds/render + vertex packing + settings + dir blueprint). |
| **B** | 159–1125 | **The authoritative core** — "Complete Transformation Roadmap": 12 file-by-file sections with code sketches, new-files table, 13-phase priority order, "what you can keep", bottom line. |
| **C** | 1127–1262 | "Deep research" gap analysis — restates B's findings in report form (same 5 Critical gaps, same per-file detail). |
| **D** | 1264–1339 | Final prose essay — restates B's phases as narrative + adds a cross-platform/WASM-compatibility table and implementation-order guidance. |

### Duplication map
- **B ⊃ A**: A is a strict summary of B. ~100% redundant.
- **C ≈ B**: every finding in C exists in B (18→10k states, HashMap→paletted+lock-free, per-chunk→MDI, procedural→zip-atlas, forward→deferred). Unique to C: "What Already Aligns Well" list, effort/priority table, honest "Skipped Research Areas" note.
- **D ≈ B (narrative)**: unique to D is the **WASM cross-platform compatibility table** (zip/image/rodio/compute-shader caveats per platform) and the fragment-shader fallback advice for FSR on WebGL2.
- Line-1 also embeds the original user prompt (requirements source).

**Verdict:** roughly 60–70% of the file's bytes restate the same 10 findings three times. The canonical content = **B** + C's alignment list + D's compatibility table. Everything else can be dropped without losing information.

---

## 2. Is the roadmap right? (Evaluation)

### Correct and confirmed against the code
- ✅ Current-state claims all verified: flat `Box<[u8; 65536]>` chunk storage (`chunk.rs`), `HashMap<ChunkPos, Arc<Chunk>>` world, per-chunk buffers from the greedy mesher, forward renderer, procedural atlas, `Settings{render_distance,sensitivity,volume}` hardcoded — all true as of the audited commit (post-f08c21a: settings expanded + persisted, 57 blocks, shadow pass, FSR-lite, enhanced F3 — see §3).
- ✅ "Keep" list is fair: single native+WASM codebase, Rayon streaming, copy-on-write edits, frustum culling, greedy meshing foundation, procedural assets as zero-asset fallback, WebGPU→WebGL2 fallback chain — all genuinely present and load-bearing.
- ✅ The phased priority order (settings → BlockState → paletted sections → packed vertices → atlas loader → deferred → shadow → MDI → FSR → F3 → shader packs → audio → polish) is sound: data layer before render layer, foundation before features.
- ✅ The WASM caveats (rodio unavailable, `zip`/`image` feature gating, compute-shader flakiness on WebGL2 translation) are real; the fragment-shader FSR fallback advice is exactly what we shipped (FSR-lite via scaled render targets + RCAS-style sharpen in the composite pass).
- ✅ Directionally right on magnitudes: ~400 block types / ~15k state variants in 1.16.5, 36-byte current vertex vs Sodium-style ~8-byte, memory-shrink math for paletted sections (order-of-magnitude correct; exact figures are illustrative, not measured).

### Flawed or overstated (do NOT copy blindly)
1. **`UltraPackedVertex` bit layout is broken as written.** 5-bit X and 5-bit Z only address 0–31 → only valid as **section-relative** coords (fine, but the doc presents it as chunk-global). Worse: 6-bit U + 6-bit V = 64×64 = 4096 UV steps — **cannot address a 2048² atlas with 16px tiles** (needs ≥ 7–8 bits each, ideally per-tile index + tile-local fract). A correct design: `tile_index:u16 + uv fract in tile:u8 each + section-local pos + normal/ao/light`. The u64 packing idea itself is valid; the given bit budget is not.
2. **`AtomicPtr` WorldGrid sketch is unsafe as written**: no Drop/dropback, no reclamation story for replaced chunks (use-after-free on swap), and `Box::into_raw` in `insert` leaks on overwrite. Real design needs epoch reclamation or `ArcSwap`-style semantics.
3. **"10,000+ variants … over 15,000"** figures vary across the four sections and count all state combinations — fine as a scale signal, not as a spec.
4. **Paletted container sketch omits the hard parts**: palette growth (4b→8b→16b) requires repacking with locking; the nibble `set_block_light` code shown has its mask math backwards for odd indices.
5. **Version pins** (`winit 0.29`, `wgpu 22`, `zip 2`, `image 0.25`) match today's Cargo.toml for the first two but the new-crate versions are untested against our tree — treat as starting points, not gospel.
6. **Deferred rendering is presented as mandatory** for Iris parity. In practice, the shipped forward+ (shadow pass + emissive block-light + composite) already delivers the user-visible features (sun shadows, block light, FSR); a full G-Buffer rewrite is justified only when real Iris-pack loading lands. This is the single biggest scope trap in the doc — 1–2 weeks of work with no player-visible change until shader packs exist.
7. **Estimates are 8–12 weeks of focused full-time work** for one developer — honest, and confirms the "multi-session" framing; the Critical rows are not one-session items.

**Bottom line:** the roadmap is a **good requirements document and a poor implementation spec**. Its diagnosis is right; ~30% of its code sketches are wrong or incomplete in ways that would cost days if followed literally. Use it as the source of truth for *what* to build, not *how*.

---

## 3. True implementation status (audited, not aspirational)

| Roadmap phase | Item | Status | Evidence |
|---|---|---|---|
| 1 | `GameSettings` + deps | ✅ **Done** (settings + persistence, 3-state graphics, maxfps, shadows, upscale; web = localStorage) | `game.rs`/`ui.rs`, commit `f08c21a` |
| 2 | BlockState system + JSON models | ◐ **Registry done** — 1.16.5-pattern state ids, `log[axis=x\|y\|z]` variants proven through the full palette→mesh→vertex pipeline; JSON model parsing still pending (needs non-cube geometry) | commit `8dde4e5`, 18 unit tests |
| 3 | Paletted 16³ ChunkSections | ✅ **Done** — vanilla palette ladder (4b→direct), entries never straddle words, section-granularity CoW; ~0.5 KiB air chunks | commit `6039a9e` |
| 4 | Packed vertex + new mesher | ✅ **Done** — VC-16 (16 B, Sodium 0.5.1 parity), chunk-relative + instance-rate origins, corrected bit layout (roadmap's 8 B is mathematically impossible) | commit `9ae6fb9` |
| 5 | zip/JAR resource-pack atlas loader | ❌ Not started (procedural, 44 tiles) — *kept as designed fallback*; wasm path researched (`zip`+`image` pure-Rust features) | `textures.rs` |
| 6 | Deferred G-Buffer renderer | ❌ Not started (forward+; **recommended deferral** until shader packs exist — see §2.6) | `render.rs` |
| 7 | Shadow map pass | ✅ **Done** (2048² sun pass, 3×3 PCF, slope+normal bias, texel-snapped light camera, strength by graphics preset) | commit `f08c21a` |
| 8 | Region MDI + global buffer | ❌ Not started — research shows MDI native-only + indirect PANICS on WebGL2; needs the 3-path draw abstraction (MDI / WebGPU-indirect-loop / GL draw-loop) | `docs/research/sodium-meshing.md` §8.4 |
| 9 | FSR 1.0 | ✅ **Done as FSR-lite** (scaled targets 75/50% + RCAS-style sharpen) — the doc's own endorsed WASM-compatible variant | commit `f08c21a` |
| 10 | Enhanced F3 | ✅ **Done** (min/avg/max fps, frame ms, frame-time graph, targeted block **+ blockstate**, backend name, chunks/verts) | commits `f08c21a`, `8dde4e5` |
| 11 | Iris shader-pack loader | ❌ Not started | — |
| 12 | Spatial audio + sounds.json | ◐ Partial (synth bank, distance/pan attenuation; no JSON registry, no categories) | `sounds.rs` |
| 13 | Polish (settings menu, biome tint, mipmap, clouds) | ◐ Partial (settings menu ✅, picker ✅, block light ✅; mipmap ❌, clouds mode ❌) | `f08c21a` |

Also shipped beyond the roadmap's letter: E-key creative picker (52 blocks), emissive block-light BFS + vertex channel, 18→57 block registry with ores/tree species/glowstone caves, web hardening commits `157324d`/`f08c21a`.

**Honest score vs. the doc's own 13-phase plan: 7.5 / 13 phases** — the full data-layer trilogy (2-registry, 3-paletted sections, 4-VC-16 vertices) plus the feature phases (1, 7, 9, 10, settings/picker extras) are done, each with unit tests and browser E2E. Remaining: resource-pack loader (5), G-buffer (6 — keep deferred), region MDI (8 — needs 3-path draw abstraction), Iris packs (11), sounds.json spatial audio (12), mipmap/clouds polish (13).

### Update 2026-09-01 (Master-Spec session — Phases 0 + 1)

The authoritative plan is now `upload/VoxelCraft-Rust_1.16.5_Master_Engineering_Spec_FINAL.md` (its own Phase 0–11 ladder). Status against it:

| Spec phase | Status | Evidence |
|---|---|---|
| P0 baseline (bench harness, frame instrumentation, regression) | ✅ Done | commits `4dbc6cf` — `--benchmark` in-game mode, `vc_bench` headless CPU benchmark (in CI), §44 phase timing in F3, golden mesh/light tests |
| P1 block/asset foundation | ✅ Done (code) | commits `e8b771a`/`4b5e0b1` — blockstate/model JSON parsers + compiler, builtin pack, property states (slab/stairs/fence), animated textures, pack texture merge; 40/40 tests; browser E2E pending CI |
| P2 world data (sections/palettes) | ✅ Done | paletted sections `6039a9e`; serialization (Anvil/NBT) still open |
| P3 mesh system | ◐ VC-16 + model path done; upload batching/MDI open | `9ae6fb9`, Phase-1 model emission |
| P4 lighting | ◐ skylight+block light done (per-mesh); incremental/cross-chunk propagation open | `mesh.rs` |
| P5–P8 | partial per table above | — |
| P10 FSR | ◐ FSR-lite; spec §33 demands real EASU+RCAS — labelled, not claimed as FSR 1 | `render.rs` |

Clean-room rule honored: the builtin pack's JSONs follow the vanilla **format**; every texture is our own procedural art (regenerable via `cargo test write_builtin_pack_pngs -- --ignored`).

### Update 2026-09-02 (Master-Spec session — Phases 4 + 5 + 6 + 7§27)

| Spec phase | Status | Evidence |
|---|---|---|
| P4 lighting — incremental + cross-chunk | ✅ Done | `40eeba7` — LightEngine with incremental `on_block_changed`/`pump`, differential light tests |
| P5 vanilla rendering (tint, particles, shadows) | ✅ Done | `b5e280c` — biome tint, block-break particles, shadow-quality presets; FSR-lite + settings from `f08c21a` |
| P6 simulation (ticks, fluids, gravity, items) | ✅ Done | `b711ed9` — deterministic 20 Hz sim core; `2dc2e51` physics regression; `d61e0ac` §25 redstone core |
| P7 gameplay — inventories/crafting/furnaces (§27/§29 subset) | ✅ Done | `inventory.rs` (36-slot, vanilla stack semantics, 4 tests), `craft.rs` (shaped recipes, 2×2/3×3, 4 tests), `furnace.rs` (200-tick smelt, fuel burn, lit-state swap, 4 tests), container screens (inventory/crafting/furnace with slot hit-testing), E-key inventory + B-key picker, right-click opens table/furnace, furnace ticking in the 20 Hz sim, E2E commands (`open`/`cclick`/`give`/`craft`/`smelt`) — browser E2E green: 119/119 unit tests, pixel-verified screens, real right-click interaction |
| P7 gameplay — brewing/enchanting/villagers/structures/dimensions | ❌ Not started (scope-gated: payoff-gated on item/entity systems) | — |

Key fix shipped alongside: `Furnaces` moved into `Sim` (ticks at exactly 20 Hz like vanilla); `default_state()` helper prevents the FURNACE=63 identity-state collision with MODEL_STATE_BASE (a raw identity placement would have rendered as the oak-slab model); LockChange guard so opening a container no longer drops into the pause menu on WASM.

### Update 2026-09-02 (P10 — real FSR 1.0)

**Phase 10 is DONE properly**: faithful WGSL ports of the official AMD FidelityFX FSR 1.0 reference (`ffx_fsr1.h` from GPUOpen-Effects/FidelityFX-FSR, float paths):

- **EASU** (`FsrEasuF`): the full 12-tap edge-adaptive kernel — luma-based directional analysis (`FsrEasuSetF` over 4 bilinear-weighted quads), direction rotation, anisotropic stretch, Lanczos-2-approximation window, dering clamp against the 4 nearest. New fullscreen pass: scene (render scale) → `up` (full surface res). textureLoad replaces gather4; exact rcp/rsqrt replace the FP16-speed approximations with epsilon guards for the degenerate flat neighborhoods.
- **RCAS** (`FsrRcasF`): the canonical 5-tap sharpener with per-channel hit-limiters, peak-range clamps, the `−RCAS_LIMIT..0` lobe and `4·lobe+1` normalization, replacing the old 5-tap approximation. Runs in the composite on the EASU output before the grade (AMD's canonical ordering: EASU → RCAS → everything else). The optional FSR_RCAS_DENOISE gate stays disabled (AMD's default).
- EASU is mathematically identity at 1:1 (unit-proven — center tap weight 1, neighbors 0), so the pass runs at every setting including native.
- **Measured quality** (same scene, |Laplacian| edge energy vs native reference): FSR 50% keeps **94.1%**, FSR 75% keeps **94.4%** — vs the old FSR-lite bilinear+sharpen path this is the difference the spec §33 demanded ("real EASU, real RCAS" — no longer labelled approximations).
- **Regression net**: new naga-based unit test validates ALL 12 embedded WGSL shaders parse+type-check with wgpu 22's exact naga (already caught one real bug: WGSL has no ternary operator); EASU identity-at-1x test; container screens + F3 re-verified in-browser; no console errors.

| Spec phase | Status | Evidence |
|---|---|---|
| P10 FSR | ✅ **Done (real FSR 1.0)** | EASU_SHADER + RCAS in `render.rs`, FsrRcasCon-mapped sharpen setting (0.6 default when upscaling), options label "75%/50% FSR", F3 "FSR1: … EASU+RCAS" |

---

## 4. Recommended remaining order (revised, risk-aware)

1. **Packed vertex + sectional dirty tracking** (phase 4, minus MDI) — biggest perf/Δ for the least architectural risk; unblocks MDI later.
2. **Paletted ChunkSections** (phase 3) — memory + mesh invalidation correctness; can land behind the existing `Chunk` API to avoid a big-bang rewrite.
3. **BlockState system** (phase 2) — needed before any resource-pack/model work; start with property-bits in a `u16`/`u32` state ID, not the doc's full JSON stack.
4. **Resource-pack loader** (phase 5) — only after 2–3; `zip`+`image` gated off on WASM unless a web pack-import path is built.
5. **Region MDI** (phase 8) then **shader packs** (phase 11) — both last, both payoff-gated on real content.
- *Defer* the full deferred/G-Buffer rewrite (phase 6) until Iris packs actually exist to load (see §2.6).
