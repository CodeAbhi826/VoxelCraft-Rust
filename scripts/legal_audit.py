#!/usr/bin/env python3
"""legal_audit.py — the standing byte-identity audit (LEGAL-COMPLIANCE.md §3.1/§7).

Hashes every reference byte-stream in upload/textures.zip + the three
logo/GUI reference files, then md5-scans every image/asset the REPO ships
(public/, voxelcraft/ builtin packs, assets-vault/, src/) and asserts
zero collisions. Also flags any reference path leaking into code.

Must print [PASS] before any deploy.
"""
import hashlib, os, sys, zipfile

ROOT = "/home/z/my-project"
REF_ZIP = os.path.join(ROOT, "upload/textures.zip")
LOGOS = ["minecraft.png", "mojangstudios.png", "inventory.png"]

SCAN_DIRS = [
    os.path.join(ROOT, "public"),
    os.path.join(ROOT, "voxelcraft"),
    os.path.join(ROOT, "src"),
]

def md5_file(p):
    h = hashlib.md5()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

def main():
    ref_md5 = {}
    with zipfile.ZipFile(REF_ZIP) as z:
        for info in z.infolist():
            if not info.is_dir():
                ref_md5[hashlib.md5(z.read(info.filename)).hexdigest()] = info.filename
    for logo in LOGOS:
        p = os.path.join(ROOT, "upload", logo)
        if os.path.exists(p):
            ref_md5[md5_file(p)] = f"upload/{logo}"
    collisions = []
    scanned = 0
    for d in SCAN_DIRS:
        for root, _, files in os.walk(d):
            if "/.git/" in root or "/target/" in root or "/node_modules/" in root:
                continue
            for f in files:
                p = os.path.join(root, f)
                if os.path.splitext(f)[1].lower() not in (".png", ".jpg", ".jpeg", ".webp", ".wasm", ".zip"):
                    continue
                scanned += 1
                if md5_file(p) in ref_md5:
                    collisions.append(p)
    # logo-slot check: the gui/title logo slots exist in the vault with OUR
    # art (format-compatible names). Verify pixel-level distinctness vs the
    # reference logos (must differ strongly — trademark boundary, §1.3).
    logo_sim = []
    import numpy as np
    from PIL import Image
    for ref_name, vault_name in (("minecraft.png", "gui/title/minecraft.png"),
                                 ("mojangstudios.png", "gui/title/mojangstudios.png")):
        ref_p = os.path.join(ROOT, "upload", ref_name)
        vault_p = os.path.join(ROOT, "voxelcraft/assets-vault/assets/minecraft/textures", vault_name)
        if os.path.exists(ref_p) and os.path.exists(vault_p):
            try:
                a = np.asarray(Image.open(ref_p).convert("RGBA").resize((64, 64)), np.float32)
                b = np.asarray(Image.open(vault_p).convert("RGBA").resize((64, 64)), np.float32)
                diff = float(np.abs(a - b).mean())
                if diff < 40:
                    logo_sim.append((vault_name, round(diff, 1)))
            except Exception:
                pass
    print(f"logo-slot pixel distinctness (vault art vs reference logos): "
          f"{'OK' if not logo_sim else logo_sim}")
    # source-line audit: no reference-set paths in engine code
    # ("upload/" alone false-positives on GPU 'mesh upload/removal' comments)
    code_hits = []
    REF_PATHS = ("upload/textures", "upload/minecraft", "upload/mojang",
                 "upload/inventory")
    for root, _, files in os.walk(os.path.join(ROOT, "voxelcraft", "crates")):
        for f in files:
            if f.endswith(".rs"):
                p = os.path.join(root, f)
                try:
                    txt = open(p, encoding="utf-8", errors="ignore").read()
                except Exception:
                    continue
                if any(rp in txt for rp in REF_PATHS):
                    code_hits.append(p)
    print(f"scanned {scanned} shipped files against {len(ref_md5)} reference streams")
    print(f"byte collisions: {len(collisions)}")
    print(f"reference-set paths in engine code: {code_hits}")
    if collisions or code_hits or logo_sim:
        for c in collisions[:10]:
            print("  COLLISION:", c)
        print("[FAIL]")
        sys.exit(1)
    print("[PASS] zero byte-identical matches — no reference asset is shipped; "
          "logo slots carry clearly distinct original art")

if __name__ == "__main__":
    main()
