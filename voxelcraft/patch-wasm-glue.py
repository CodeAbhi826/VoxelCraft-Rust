#!/usr/bin/env python3
"""Post wasm-bindgen patch: harden the generated JS glue against events that
lack pointerType (synthetic / CDP-dispatched events).

wasm-bindgen regenerates voxelcraft.js on every rebuild, wiping manual edits,
so this script re-applies the patch. Run it AFTER wasm-bindgen, BEFORE
copying the bundle into public/.

Usage: python3 patch-wasm-glue.py [path/to/voxelcraft.js]
       (defaults to ./wasm-out/voxelcraft.js, patches in place)

Why regex: the import identifier is `__wbg_pointerType_<hash>` where the
hash covers the wasm module's whole import set — every round that adds a
new web-sys/js-sys binding changes it (the 1.12 round did). Matching the
name + body shape (not the literal hash) keeps the patch stable across
rounds; the hash found in the file is preserved in the replacement.
"""
import re
import sys

# name + typed-args header, hash-agnostic, whitespace-tolerant
PATTERN = re.compile(
    r"(__wbg_pointerType_[0-9a-f]+: function\(arg0, arg1\) \{\s*\n)"
    r"(\s*)const ret = arg1\.pointerType;"
)

PATCHED_BODY = """{header}{indent}// PATCHED: synthetic/automation events (CDP Input.dispatchMouseEvent,
{indent}// dispatched MouseEvent with type 'pointer*') lack pointerType —
{indent}// winit's own canvas handlers then crashed with
{indent}// "Cannot read properties of undefined (reading 'length')".
{indent}// Default to '' (winit classifies as generic pointer; the game's
{indent}// input shim handles actual gameplay input anyway).
{indent}const ret = arg1.pointerType || '';"""


def main(path: str) -> int:
    with open(path, "r", encoding="utf-8") as f:
        s = f.read()
    if re.search(r"const ret = arg1\.pointerType \|\| ''", s):
        print(f"[patch-wasm-glue] {path}: already patched, nothing to do")
        return 0
    m = PATTERN.search(s)
    if not m:
        print(f"[patch-wasm-glue] {path}: WARNING — glue pattern not found "
              "(wasm-bindgen output shape changed?). Skipping.")
        return 2
    header, indent = m.group(1), m.group(2)
    s = s[: m.start()] + PATCHED_BODY.format(header=header, indent=indent) + s[m.end():]
    with open(path, "w", encoding="utf-8") as f:
        f.write(s)
    print(f"[patch-wasm-glue] {path}: patched pointerType glue "
          f"(import id: {m.group(1).split(':')[0]})")
    return 0


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "wasm-out/voxelcraft.js"
    sys.exit(main(target))
