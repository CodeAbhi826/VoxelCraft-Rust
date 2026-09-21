#!/usr/bin/env python3
"""Bracket 16/16 (final polish pass): make the workspace warning-clean.

42 warnings, all mechanical, each resolved with the smallest honest fix:
- unused `mut` / unused vars / unused imports / dead initializers -> removed
- GPU keep-alive struct fields (views/samplers/textures held only so the
  bind groups' resources stay alive) -> #[allow(dead_code)] + doc line
- wasm-only-constructed enum variant -> #[allow(dead_code)] + doc line
- platform-gated state fields (wasm read paths) -> #[allow(dead_code)] + doc
- non-snake-case fns -> renamed (defs + the 2 call sites)
- duplicate match arm (ROTTEN_FLESH twice in is_food) -> second arm removed
- irrefutable if-let (single-variant LootFn) -> exhaustive match
- always-true e2e self-check (usize >= 0) -> `> 0` (the intended check)
- dead L_SB constant -> actually used in build_lut (L_SB + s), mirroring WGSL
- v115_world (test-only helper outside the tests mod) -> #[cfg(test)]
- unused `///` on statements -> `//`
- pointless drop(&ref) -> removed
- unreachable PresentMode wildcard -> kept + allow (wgpu drift guard)
- dead art palette constants -> deleted; LILY_w -> LILY_W_SHADE

Two-phase: every edit is validated against exact file content BEFORE
anything is applied; any mismatch aborts with no changes.
"""

import re
import sys
from pathlib import Path

ROOT = Path("/home/z/my-project/voxelcraft")
assert ROOT.is_dir(), f"workspace root missing: {ROOT}"


def rd(rel):
    return (ROOT / rel).read_text(encoding="utf-8").split("\n")


def wr(rel, lines):
    (ROOT / rel).write_text("\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------- line edits ----
# (relpath, 1-based line, expected old content, new content or None to delete)
LINE_EDITS = [
    # vc-world: unused mut x3, snake_case renames x2 (+ call sites)
    ("crates/vc-world/src/gen.rs", 887,
     "        for z in 0..CHUNK_Z_CHUNK() {",
     "        for z in 0..chunk_z_chunk() {"),
    ("crates/vc-world/src/gen.rs", 888,
     "            for x in 0..CHUNK_X_CHUNK() {",
     "            for x in 0..chunk_x_chunk() {"),
    ("crates/vc-world/src/gen.rs", 995,
     "        let mut set_dec = |chunk: &mut Chunk,",
     "        let set_dec = |chunk: &mut Chunk,"),
    ("crates/vc-world/src/gen.rs", 3794,
     "        let mut put_state = |chunk: &mut Chunk, x: i32, y: i32, z: i32, st: u16| {",
     "        let put_state = |chunk: &mut Chunk, x: i32, y: i32, z: i32, st: u16| {"),
    ("crates/vc-world/src/gen.rs", 3923,
     "        let mut room =",
     "        let room ="),
    ("crates/vc-world/src/gen.rs", 4502,
     "fn CHUNK_X_CHUNK() -> usize {",
     "fn chunk_x_chunk() -> usize {"),
    ("crates/vc-world/src/gen.rs", 4506,
     "fn CHUNK_Z_CHUNK() -> usize {",
     "fn chunk_z_chunk() -> usize {"),

    # vc-gameplay: unused param
    ("crates/vc-gameplay/src/villagers.rs", 1025,
     "    up: [f32; 3],",
     "    _up: [f32; 3],"),

    # vc-render: parens x6, dead let, dead palette consts
    ("crates/vc-render/src/textures/v113_art.rs", 210,
     "                    (GREEN[0]),", "                    GREEN[0],"),
    ("crates/vc-render/src/textures/v113_art.rs", 211,
     "                    (GREEN[1]),", "                    GREEN[1],"),
    ("crates/vc-render/src/textures/v113_art.rs", 212,
     "                    (GREEN[2]),", "                    GREEN[2],"),
    ("crates/vc-render/src/textures/v113_art.rs", 224,
     "                (LIGHT[0]),", "                LIGHT[0],"),
    ("crates/vc-render/src/textures/v113_art.rs", 225,
     "                (LIGHT[1]),", "                LIGHT[1],"),
    ("crates/vc-render/src/textures/v113_art.rs", 226,
     "                (LIGHT[2]),", "                LIGHT[2],"),
    ("crates/vc-render/src/textures.rs", 3509,
     "            let (bx, by) = (x / 4, y / 8);", None),
    ("crates/vc-render/src/textures/v116b_art.rs", 62,
     "const SF_OUT: [i32; 3] = [44, 96, 200];", None),
    ("crates/vc-render/src/textures/v116b_art.rs", 63,
     "/// iron gray (the soul lantern frame).", None),
    ("crates/vc-render/src/textures/v116b_art.rs", 64,
     "const IRON_M: [i32; 3] = [146, 150, 156];", None),
    ("crates/vc-render/src/textures/v116b_art.rs", 65,
     "const IRON_D: [i32; 3] = [96, 100, 106];", None),

    # app crate
    ("crates/voxelcraft/src/game.rs", 3179,
     "        /// hand\" = the selected/hotbar item (no offhand slot \u2014",
     "        // hand\" = the selected/hotbar item (no offhand slot \u2014"),
    ("crates/voxelcraft/src/game.rs", 3385,
     "                    use vc_gameplay::mobs;", None),
    ("crates/voxelcraft/src/game.rs", 3464,
     "                        use vc_gameplay::mobs;", None),
    ("crates/voxelcraft/src/game.rs", 6031,
     "                let (get, get_n) = tr.get;", None),
    ("crates/voxelcraft/src/game.rs", 6139,
     "            Some(Container::Chest { pos }) => (ContainerKind::Chest, None, None, None, None),",
     "            Some(Container::Chest { .. }) => (ContainerKind::Chest, None, None, None, None),"),
    ("crates/voxelcraft/src/game.rs", 6140,
     "            Some(Container::Barrel { pos }) => (ContainerKind::Barrel, None, None, None, None),",
     "            Some(Container::Barrel { .. }) => (ContainerKind::Barrel, None, None, None, None),"),
    ("crates/voxelcraft/src/game.rs", 6904,
     "            crafts_ok, smelt_ok, gilded_drop >= 0, nugget_roll, gold_ok, gold_drop, soul_dmg",
     "            crafts_ok, smelt_ok, gilded_drop > 0, nugget_roll, gold_ok, gold_drop, soul_dmg"),
    ("crates/voxelcraft/src/game.rs", 12267,
     "                        drop(held);", None),
    ("crates/voxelcraft/src/game.rs", 14188,
     "        let mut x = 0u32;", None),
    ("crates/voxelcraft/src/game.rs", 14217,
     "        x = self.sim.mobs.arrows.len() as u32;",
     "        let x = self.sim.mobs.arrows.len() as u32;"),
    ("crates/voxelcraft/src/game.rs", 15181,
     "            | ROTTEN_FLESH", None),
    ("crates/voxelcraft/src/player.rs", 333,
     "        /// w/Effect \u00a7Absorption \u2014 the yellow hearts absorb incoming",
     "        // w/Effect \u00a7Absorption \u2014 the yellow hearts absorb incoming"),
    ("crates/voxelcraft/src/player.rs", 1185,
     "    let mut t = 0.0f32;",
     "    let mut t: f32;"),
]

# ------------------------------------------------- insert-before edits ----
# (relpath, regex for the anchor line, lines to insert above it, expect count)
INSERT_EDITS = [
    ("crates/vc-gameplay/src/mobs.rs",
     r"^    fn v115_world\(\) -> World \{$",
     ["    #[cfg(test)]  // used only by the tests below (the per-module convention)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    up: wgpu::Texture,$",
     ["    #[allow(dead_code)] // GPU keep-alive (the _view / bind groups hold it)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    pack: wgpu::Texture,$",
     ["    #[allow(dead_code)] // GPU keep-alive (pack_view / the composite pass)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    q: wgpu::Texture,$",
     ["    #[allow(dead_code)] // GPU keep-alive (q_view / the bright pass)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    b1: wgpu::Texture,$",
     ["    #[allow(dead_code)] // GPU keep-alive (b1_view / blur ping)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    b2: wgpu::Texture,$",
     ["    #[allow(dead_code)] // GPU keep-alive (b2_view / blur pong)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    ui_view: wgpu::TextureView,$",
     ["    #[allow(dead_code)] // GPU keep-alive (bound via ui_bg at init)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    ui_samp: wgpu::Sampler,$",
     ["    #[allow(dead_code)] // GPU keep-alive (bound via ui_bg at init)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    tint_tex: ",
     ["    #[allow(dead_code)] // GPU keep-alive (the scene bind group owns it)"],
     1),
    ("crates/vc-render/src/render.rs",
     r"^    pub fn present_mode_name\(&self\) -> String \{$",
     ["    #[allow(unreachable_patterns)] // `_` arm kept for wgpu PresentMode drift"],
     1),
    ("crates/voxelcraft/src/game.rs",
     r"^    Inline \{$",
     ["    /// the wasm32-only backend (native uses the threaded pool above)",
     "    #[allow(dead_code)]"],
     1),
    ("crates/voxelcraft/src/game.rs",
     r"^    stats_t: f32,$",
     ["    #[allow(dead_code)] // read only on wasm32 (the E2E stats publisher)"],
     1),
    ("crates/voxelcraft/src/game.rs",
     r"^    ever_locked: bool,$",
     ["    #[allow(dead_code)] // read only on wasm32 (the web pointer-lock path)"],
     1),
    ("crates/voxelcraft/src/game.rs",
     r"^    iris_packs: Vec<vc_render::iris::IrisPackInfo>,$",
     ["    #[allow(dead_code)] // scanned at boot, not yet surfaced in a screen"],
     1),
    ("crates/voxelcraft/src/game.rs",
     r"^    web_shift: bool,$",
     ["    #[allow(dead_code)] // read only on wasm32 (the web keyboard path)"],
     1),
]

# ------------------------------------------------------ string edits ------
STRING_EDITS = [
    # datapack: irrefutable if-let -> exhaustive match (single-variant enum)
    ("crates/vc-pack/src/datapack.rs",
     """                        for f in functions {
                            if let LootFn::SetCount { min, max } = f {
                                let t = rng.next_f32();
                                let v = min + (max - min) * t;
                                count = v.round().clamp(1.0, 64.0) as u8;
                            }
                        }""",
     """                        for f in functions {
                            match f {
                                LootFn::SetCount { min, max } => {
                                    let t = rng.next_f32();
                                    let v = min + (max - min) * t;
                                    count = v.round().clamp(1.0, 64.0) as u8;
                                }
                            }
                        }"""),
    # auditfix art: unused rng params -> _rng
    ("crates/vc-render/src/textures/auditfix_art.rs",
     "pub(super) fn golden_carrot(a: &mut [u8], t: u16, rng: &mut Rng) {",
     "pub(super) fn golden_carrot(a: &mut [u8], t: u16, _rng: &mut Rng) {"),
    ("crates/vc-render/src/textures/auditfix_art.rs",
     "pub(super) fn vine(a: &mut [u8], t: u16, rng: &mut Rng) {",
     "pub(super) fn vine(a: &mut [u8], t: u16, _rng: &mut Rng) {"),
    ("crates/vc-render/src/textures/auditfix_art.rs",
     "pub(super) fn fern(a: &mut [u8], t: u16, rng: &mut Rng) {",
     "pub(super) fn fern(a: &mut [u8], t: u16, _rng: &mut Rng) {"),
    # v114c art: LILY_w -> LILY_W_SHADE (upper case + descriptive)
    ("crates/vc-render/src/textures/v114c_art.rs",
     "const LILY_w: [i32; 3] = [212, 216, 224];",
     "const LILY_W_SHADE: [i32; 3] = [212, 216, 224];"),
    ("crates/vc-render/src/textures/v114c_art.rs",
     "'w' => Some((LILY_w[0], LILY_w[1], LILY_w[2], 255)),",
     "'w' => Some((LILY_W_SHADE[0], LILY_W_SHADE[1], LILY_W_SHADE[2], 255)),"),
    # gpu_mesh: make L_SB live (mirrors the WGSL reader lut[L_SB + ...])
    ("crates/vc-render/src/gpu_mesh.rs",
     "        lut[s] = state_block(s as u16) as u32;",
     "        lut[L_SB + s] = state_block(s as u16) as u32;"),
]


def main() -> int:
    # ---- phase 1: validate everything, change nothing ----
    errors = []

    # line edits: group by file, check exact content at the line
    by_file = {}
    for rel, line, old, new in LINE_EDITS:
        by_file.setdefault(rel, []).append((line, old, new))
    for rel, items in by_file.items():
        lines = rd(rel)
        for line, old, new in items:
            idx = line - 1
            if idx >= len(lines):
                errors.append(f"{rel}:{line}: line beyond EOF")
                continue
            actual = lines[idx]
            if actual != old:
                errors.append(f"{rel}:{line}: content mismatch\n  want: {old!r}\n  got:  {actual!r}")
    if errors:
        print("VALIDATION FAILED (line edits) — nothing applied:")
        for e in errors:
            print(" ", e)
        return 1

    for rel, pattern, insert, expect in INSERT_EDITS:
        lines = rd(rel)
        hits = [i for i, l in enumerate(lines) if re.match(pattern, l)]
        if len(hits) != expect:
            errors.append(f"{rel}: pattern {pattern!r} matched {len(hits)} lines (want {expect})")
    if errors:
        print("VALIDATION FAILED (insert anchors) — nothing applied:")
        for e in errors:
            print(" ", e)
        return 1

    for rel, old, new in STRING_EDITS:
        text = "\n".join(rd(rel))
        n = text.count(old)
        if n != 1:
            errors.append(f"{rel}: string edit matched {n} times (want 1): {old[:60]!r}...")
    if errors:
        print("VALIDATION FAILED (string edits) — nothing applied:")
        for e in errors:
            print(" ", e)
        return 1

    print(f"validation ok: {len(LINE_EDITS)} line edits, "
          f"{len(INSERT_EDITS)} inserts, {len(STRING_EDITS)} string edits")

    # ---- phase 2: apply (line edits bottom-up per file) ----
    for rel, items in by_file.items():
        lines = rd(rel)
        for line, old, new in sorted(items, key=lambda it: -it[0]):
            idx = line - 1
            assert lines[idx] == old
            if new is None:
                del lines[idx]
            else:
                lines[idx] = new
        wr(rel, lines)
        print(f"applied {len(items)} line edits: {rel}")

    for rel, pattern, insert, expect in INSERT_EDITS:
        lines = rd(rel)
        hits = [i for i, l in enumerate(lines) if re.match(pattern, l)]
        assert len(hits) == expect, (rel, pattern, len(hits))
        for i in reversed(hits):
            lines[i:i] = insert
        wr(rel, lines)
        print(f"applied insert: {rel} :: {pattern}")

    for rel, old, new in STRING_EDITS:
        text = "\n".join(rd(rel))
        assert text.count(old) == 1, (rel, old[:50])
        wr(rel, text.replace(old, new).split("\n"))
        print(f"applied string edit: {rel}")

    print("ALL EDITS APPLIED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
