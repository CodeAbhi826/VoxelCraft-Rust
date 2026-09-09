#!/usr/bin/env python3
"""Post wasm-bindgen patch: harden the generated JS glue against browser /
automation quirks. Two patches, each idempotent and independently applied:

1. pointerType: synthetic / CDP-dispatched events lack pointerType.
2. exitFullscreen: Document.exitFullscreen() can throw (headless /
   automation contexts report "Document not active" even when the call
   would be a no-op in a real browser). The game treats fullscreen as
   best-effort, so the throw is swallowed at the API boundary.

wasm-bindgen regenerates voxelcraft.js on every rebuild, wiping manual edits,
so this script re-applies the patch. Run it AFTER wasm-bindgen, BEFORE
copying the bundle into public/.

Usage: python3 patch-wasm-glue.py [path/to/voxelcraft.js]
       (defaults to ./wasm-out/voxelcraft.js, patches in place)

Why regex, and why TWO body shapes:
* the import identifier is `__wbg_pointerType_<hash>` — the hash covers the
  module's whole import set, so any round adding a web-sys/js-sys binding
  can change it;
* the argument handling depends on the wasm-bindgen codegen mode, which
  follows the rustc toolchain (observed live: rustc 1.98.0 -> 1.98.1 flipped
  this getter between externref-style `arg1.pointerType` and heap-index
  style `getObject(arg1).pointerType`). Matching both keeps the patch
  stable across toolchain drift; the exact import id found in the file is
  preserved in the replacement.
"""
import re
import sys

# name + typed-args header, hash-agnostic, whitespace-tolerant; body accepts
# BOTH codegen modes (externref: arg1 is the object; heap: arg1 is an index
# that getObject() unwraps)
PATTERN = re.compile(
    r"(__wbg_pointerType_[0-9a-f]+: function\(arg0, arg1\) \{\s*\n)"
    r"(\s*)const ret = (getObject\(arg1\)|arg1)\.pointerType;"
)

PATCHED_BODY = """{header}{indent}// PATCHED: synthetic/automation events (CDP Input.dispatchMouseEvent,
{indent}// dispatched MouseEvent with type 'pointer*') lack pointerType —
{indent}// winit's own canvas handlers then crashed with
{indent}// "Cannot read properties of undefined (reading 'length')".
{indent}// Default to '' (winit classifies as generic pointer; the game's
{indent}// input shim handles actual gameplay input anyway).
{indent}const ret = {expr}.pointerType || '';"""

# Document.exitFullscreen() import — hash-agnostic (same drift reasoning
# as the pointerType import). Body is a single statement in every observed
# wasm-bindgen codegen mode, so one shape suffices.
EXIT_FS_PATTERN = re.compile(
    r"(__wbg_exitFullscreen_[0-9a-f]+: function\(arg0\) \{\s*\n)"
    r"(\s*)(getObject\(arg0\))\.exitFullscreen\(\);"
)

EXIT_FS_BODY = """{header}{indent}// PATCHED: exitFullscreen can throw in headless/automation contexts
{indent}// ("Document not active") where a real browser would no-op it. The
{indent}// game treats fullscreen as best-effort (winit Result ignored on the
{indent}// Rust side) — swallow here instead of surfacing a page error.
{indent}try {{ {expr}.exitFullscreen(); }} catch (_) {{}}"""


def main(path: str) -> int:
    with open(path, "r", encoding="utf-8") as f:
        s = f.read()
    rc = 0

    # --- patch 1: pointerType hardening ---------------------------------
    if re.search(r"const ret = (?:getObject\(arg1\)|arg1)\.pointerType \|\| ''", s):
        print(f"[patch-wasm-glue] {path}: pointerType already patched")
    else:
        m = PATTERN.search(s)
        if not m:
            print(f"[patch-wasm-glue] {path}: WARNING — pointerType pattern "
                  "not found (wasm-bindgen output shape changed?). Skipping.")
            rc = 2
        else:
            header, indent, expr = m.group(1), m.group(2), m.group(3)
            s = s[: m.start()] + PATCHED_BODY.format(
                header=header, indent=indent, expr=expr) + s[m.end():]
            print(f"[patch-wasm-glue] {path}: patched pointerType glue "
                  f"(import id: {m.group(1).split(':')[0]}, mode: {expr})")

    # --- patch 2: exitFullscreen best-effort -----------------------------
    if "// PATCHED: exitFullscreen can throw" in s:
        print(f"[patch-wasm-glue] {path}: exitFullscreen already patched")
    else:
        m = EXIT_FS_PATTERN.search(s)
        if not m:
            print(f"[patch-wasm-glue] {path}: WARNING — exitFullscreen "
                  "pattern not found (no fullscreen binding?). Skipping.")
        else:
            header, indent, expr = m.group(1), m.group(2), m.group(3)
            s = s[: m.start()] + EXIT_FS_BODY.format(
                header=header, indent=indent, expr=expr) + s[m.end():]
            print(f"[patch-wasm-glue] {path}: patched exitFullscreen glue "
                  f"(import id: {m.group(1).split(':')[0]})")

    with open(path, "w", encoding="utf-8") as f:
        f.write(s)
    return rc


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "wasm-out/voxelcraft.js"
    sys.exit(main(target))
