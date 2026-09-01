#!/usr/bin/env python3
"""Post wasm-bindgen patch: harden the generated JS glue against events that
lack pointerType (synthetic / CDP-dispatched events).

wasm-bindgen regenerates voxelcraft.js on every rebuild, wiping manual edits,
so this script re-applies the patch. Run it AFTER wasm-bindgen, BEFORE
copying the bundle into public/.

Usage: python3 patch-wasm-glue.py [path/to/voxelcraft.js]
       (defaults to ./wasm-out/voxelcraft.js, patches in place)
"""
import sys

MARKER = "__wbg_pointerType_b3dafa8fb9c97016: function(arg0, arg1) {\n            const ret = arg1.pointerType;"

PATCH = """__wbg_pointerType_b3dafa8fb9c97016: function(arg0, arg1) {
            // PATCHED: synthetic/automation events (CDP Input.dispatchMouseEvent,
            // dispatched MouseEvent with type 'pointer*') lack pointerType —
            // winit's own canvas handlers then crashed with
            // "Cannot read properties of undefined (reading 'length')".
            // Default to '' (winit classifies as generic pointer; the game's
            // input shim handles actual gameplay input anyway).
            const ret = arg1.pointerType || '';"""

def main(path: str) -> int:
    with open(path, "r", encoding="utf-8") as f:
        s = f.read()
    if "const ret = arg1.pointerType || ''" in s:
        print(f"[patch-wasm-glue] {path}: already patched, nothing to do")
        return 0
    if MARKER not in s:
        print(f"[patch-wasm-glue] {path}: WARNING — glue pattern not found "
              "(wasm-bindgen version changed?). Skipping.")
        return 2
    s = s.replace(MARKER, PATCH, 1)
    with open(path, "w", encoding="utf-8") as f:
        f.write(s)
    print(f"[patch-wasm-glue] {path}: patched pointerType glue")
    return 0

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "wasm-out/voxelcraft.js"
    sys.exit(main(target))
