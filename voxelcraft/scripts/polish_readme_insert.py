#!/usr/bin/env python3
"""Insert the bracket 16/16 (final polish) README entry + the era-coverage
table right after the bracket 15/16 entry (line 166)."""

from pathlib import Path

README = Path("/home/z/my-project/README.md")

ENTRY = """
**Version-evolution bracket 16/16 (the FINAL POLISH PASS — era closeout, 2026-09-09):** the closing round of the 16-bracket plan — with the era feature-complete (brackets 1–15 including the completeness audit), this pass hardens quality instead of content. **A zero-warning workspace on both targets**: all 50 accumulated lint diagnostics eliminated (42 native + the wasm32 mirror set of 8), each fixed at the root rather than blanket-muted — the GPU keep-alive fields (the composite pipeline's texture/view pairs and the UI view/sampler/tint triple, held only so the bind groups' resources stay alive) now carry `#[allow(dead_code)]` with the reason written at the field; the target-gated construction/read splits (the `Inline` vs `Threading` mesh backends; `stats_t`/`ever_locked`/`web_shift` read only on wasm32, `ws_selected`/`iris_packs` only on the native side) are annotated per field; the file's own `#[cfg_attr(target_arch = "wasm32", …)]` convention covers the two scan-gated `mut`s; genuinely dead code deleted outright — a duplicated `Some("emerald")` arm in the E2E give-parser that could never fire (the earlier item arm always won), two no-op `drop(&mut)` calls, a dead `(bx, by)` destructuring, three unused art palette constants, and `LILY_w` renamed `LILY_W_SHADE`; the single-variant `LootFn` `if let` became an exhaustive `match` (future variants now fail loudly at the compiler instead of being silently unreachable); the LUT constant `L_SB` is finally exercised by `build_lut` itself (`lut[L_SB + s]`, mirroring the WGSL reader); and `present_mode_name`'s PresentMode wildcard keeps its wgpu-drift guard with the lint acknowledged at the function. **Two real latent bugs the sweep flushed out**: the e2e self-check `gilded_drop >= 0` on a `usize` was vacuously always-true — now `> 0`, the check it was always meant to be; and the in-browser boot raised exactly one page error (`TypeError: exitFullscreen … Document not active` — winit's boot-time fullscreen reset hitting a headless/automation quirk) — `patch-wasm-glue.py` now hardens a SECOND import (best-effort `try/catch` at the API boundary, the pointerType patch's exact philosophy; game logic untouched, real browsers unaffected). Verified: **593/593 tests** green (unchanged — polish only), `cargo check` **zero warnings on native AND wasm32**, the wasm bundle rebuilt from the new HEAD (matched pair, mtime-identical) and browser-verified live: title screen in 2.66 s with rendered content and **zero page errors** (`docs/screenshots/polish16-title.png`). The era-coverage table below closes the 16-bracket plan. Progress log: `docs/WORKLOG.md`.

**The 1.0 → 1.16.5 era at a glance — the version-evolution plan, closed (16/16):**

| # | Versions | Update | Headline | Record |
|---|---|---|---|---|
| 1 | 1.0–1.2 | Core World Content | The End + Ender Dragon fight, Nether Fortress, Mushroom Fields, 7 mobs, XP orbs | `phase1-1.0-1.2-research.md` |
| 2 | 1.3–1.4 | Adventure Features | Wither boss + beacon + ender chest, Adventure mode, anvil, lava fluid, timed status effects | `phase2-1.3-1.4-research.md` |
| 3 | 1.5–1.6 | Transport & Building | Horses/donkeys/mules (stats, taming, breeding), leads, name tags, hay bales, carpets | `phase3-1.5-1.6-research.md` |
| 4 | 1.7.2 | The Update that Changed the World | Acacia + dark oak, the new biome set with verified grass colors, the status-effect foundation | WORKLOG 2026-09-06 |
| 5 | 1.8 | Bountiful Update | Granite/diorite/andesite, red sandstone, prismarine, rabbits, slime-block bounce | WORKLOG 2026-09-06 |
| 6 | 1.9 | Combat Update | Attack cooldown, grass paths, the purpur family + end rods, elytra gliding | WORKLOG 2026-09-06 |
| 7 | 1.10 | Frostburn Update | The frosty trio — stray, husk, polar bear — with biome-locked spawning | WORKLOG 2026-09-06 |
| 8 | 1.11 | Exploration Update | Woodland mansions with illager spawners, vindicator/evoker/vex, totem of undying | `verify_v111_*` captures |
| 9 | 1.12 | World of Color Update | Concrete ×16 + powder solidification, glazed terracotta ×16, parrot + illusioner | `phase-v112-1.12-research.md` |
| 10 | 1.13 | Update Aquatic | Ocean temperature split, the aquatic mob set, coral, sea pickles, kelp | `phase-v113-1.13-research.md` |
| 11 | 1.14 | Village & Pillage (nature half) | Bamboo, sweet berry bushes, campfires, barrels, foxes, the two small flowers | `phase-v114-1.14-research.md` |
| 12 | 1.15 | Buzzy Bees | The full bee lifecycle — hives, honey, flowering pollination | `phase-v115-1.15-research.md` |
| 13 | 1.16 | Nether Update (parts 1+2) | Respawn anchors, targets, lodestones, piglin bartering, hoglins, striders, the crimson/warped families, soul fire | `phase-v116-1.16-research.md` + `phase-v116b-1.16-research.md` |
| 15 | 1.0–1.16.5 | COMPLETENESS AUDIT | The full-era recheck — every in-capability gap closed, the deferral inventory itemized | `audit16-completeness-research.md` |
| 16 | — | FINAL POLISH PASS | Zero-warning build on both targets, two latent bugs fixed, bundle re-verified | this entry |

(Brackets 4–7 shipped through the 2026-09-07 reconciliation merge — the
README bracket entries pick up at 8/16; their research lives in the WORKLOG
dated sections and the `verify_*` capture sets.) Every era gap outside the
engine's disclosed capability class is itemized in the audit's research
record; the engine is now 1.16.5-era complete and clean.
"""

lines = README.read_text(encoding="utf-8").split("\n")
# bracket 15/16 entry is line 166 (1-based); insert after it + its blank line
assert lines[165].startswith("**Version-evolution bracket 15/16"), lines[165][:60]
assert lines[166] == "", repr(lines[166])
insert_at = 167  # 0-based index of maintenance note 8 — insert before it
assert lines[insert_at].startswith("Post-Phase 10 maintenance note 8"), lines[insert_at][:60]
new = lines[:insert_at] + ENTRY.strip().split("\n") + ["", ""] + lines[insert_at:]
README.write_text("\n".join(new), encoding="utf-8")
print(f"inserted {len(ENTRY.strip().split(chr(10)))} lines before note 8")
