#!/usr/bin/env python3
"""legal_audit.py — the standing compliance audit (LEGAL-COMPLIANCE.md §6).

Scans everything the repo SHIPS (code, assets, packs, docs, CI) for:

  1. Third-party trademark names (the de-branded vocabulary check)
  2. Files that must never exist in our tree (third-party fonts, the
     ecosystem's manifest/sidecar conventions in OUR packs, scraped
     wiki dumps)
  3. Optional byte/pixel-identity check against a reference set, when
     one is supplied via --reference <path-or-zip> (the reference set
     itself must NEVER live inside the repository)

Must print [PASS] before any deploy. Exit code 0 = pass, 1 = fail.
"""
import argparse
import hashlib
import io
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# ------------------------------------------------------------- rules --

# Trademark/coinage scan: any of these in a SHIPPED file is a failure.
# (The interop table + migration tooling are the only exempted files,
# declared below — they never render to users.)
# bare persona/short names need word boundaries (avoids matching
# inside identifiers like CrystalExplosion); compounds are substring-safe
TERM_RX = re.compile(
    r"minecraft|mojang|herobrine|monocraft|"
    r"creeper|enderman|endermen|endermite|mooshroom|netherite|netherrack|"
    r"nether|redstone|piglin|hoglin|zoglin|strider|shulker|ghast|"
    r"purpur|prismarine|nylium|shroomlight|chorus|elytra|warden|allay|"
    r"vindicator|evoker|illusioner|crimson|warped|soulsand|soul_sand|"
    r"soul_soil|respawn.anchor|crying.obsidian|totem.of.undying|"
    r"the_end|the_nether|programmer",
    re.IGNORECASE,
)
# "notch" is a dictionary word (notch filter in DSP) — not scanned
PERSONA_RX = re.compile(r"\b(steve|alex)\b", re.IGNORECASE)

EXEMPT_FILES = {
    # read-side format-interop data (generated; never user-visible)
    "voxelcraft/crates/vc-pack/src/legacy_aliases.rs",
    # the rename tooling itself (audit trail of the de-branding)
    "voxelcraft/scripts/rename_terms.py",
    "voxelcraft/scripts/gen_legacy_aliases.py",
    "voxelcraft/scripts/restructure_packs.py",
    # this script
    "scripts/legal_audit.py",
}

# Files/conventions that must never exist in OUR trees
FORBIDDEN_PATTERNS = [
    ("voxelcraft/crates/vc-render/assets/Monocraft.ttf",
     "third-party font (use the generated Voxelfont.ttf)"),
    ("voxelcraft/crates/vc-render/assets/OFL-Monocraft.txt",
     "third-party font license"),
]
FORBIDDEN_SUFFIXES_IN_OUR_PACKS = [
    (".mcmeta", "the ecosystem's manifest/sidecar convention in our packs "
                "(ours: pack.json + .png.json)"),
]
PACK_ROOTS = [
    "voxelcraft/builtin-pack",
    "voxelcraft/builtin-packs",
    "voxelcraft/assets-vault",
]

TEXT_EXT = {".rs", ".toml", ".json", ".md", ".sh", ".py", ".yml", ".yaml",
            ".ts", ".tsx", ".js", ".html", ".css", ".txt", ".glsl", ".mjs"}

SCAN_DIRS = ["voxelcraft/crates", "voxelcraft/scripts", "voxelcraft/diag",
             "voxelcraft/builtin-pack", "voxelcraft/builtin-packs",
             "voxelcraft/assets-vault/spec", "voxelcraft/*.md",
             "voxelcraft/*.toml", "docs", "src", "public", "scripts", "ci",
             ".github", "tests", "README.md", "GEMINI.md", "Caddyfile"]

SKIP_DIRS = {"target", "node_modules", ".git", ".next", "wasm-out",
             "build-artifact", "upload", "skills", "resourcepacks",
             "shader-packs", "saves", "logs"}

# ------------------------------------------------------------------ --


def iter_shipped_files():
    for entry in SCAN_DIRS:
        base = os.path.join(ROOT, entry)
        if base.endswith(".md") or base.endswith(".toml") or entry == "Caddyfile":
            if os.path.isfile(base):
                yield os.path.relpath(base, ROOT)
            continue
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for f in filenames:
                rel = os.path.relpath(os.path.join(dirpath, f), ROOT)
                if os.path.splitext(f)[1] in TEXT_EXT or f == ".gitignore":
                    yield rel


def audit_terms():
    fails = 0
    for rel in iter_shipped_files():
        rel_norm = rel.replace(os.sep, "/")
        if rel_norm in EXEMPT_FILES:
            continue
        try:
            s = open(os.path.join(ROOT, rel), encoding="utf-8",
                     errors="ignore").read()
        except OSError:
            continue
        m = TERM_RX.search(s)
        if not m:
            m = PERSONA_RX.search(s)
        if m:
            print(f"[FAIL] trademark term {m.group(0)!r} in {rel_norm}")
            fails += 1
    return fails


def audit_forbidden_files():
    fails = 0
    for path, why in FORBIDDEN_PATTERNS:
        if os.path.exists(os.path.join(ROOT, path)):
            print(f"[FAIL] forbidden file {path} ({why})")
            fails += 1
    for root_dir in PACK_ROOTS:
        base = os.path.join(ROOT, root_dir)
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            for f in filenames:
                for suffix, why in FORBIDDEN_SUFFIXES_IN_OUR_PACKS:
                    if f.endswith(suffix):
                        print(f"[FAIL] {why}: "
                              f"{os.path.relpath(os.path.join(dirpath, f), ROOT)}")
                        fails += 1
    return fails


def audit_reference_identity(reference):
    """byte + pixel identity against an EXTERNAL reference set"""
    fails = 0
    ref_hashes = {}
    ref_pixels = set()
    try:
        from PIL import Image
    except ImportError:
        print("[WARN] pillow not available — pixel check skipped")
        Image = None

    def add_bytes(name, data):
        ref_hashes[hashlib.sha256(data).hexdigest()] = name
        if Image is not None and name.endswith(".png"):
            try:
                img = Image.open(io.BytesIO(data)).convert("RGBA")
                ref_pixels.add(hashlib.sha256(img.tobytes()).hexdigest())
            except Exception:
                pass

    if os.path.isfile(reference) and reference.endswith(".zip"):
        import zipfile
        with zipfile.ZipFile(reference) as z:
            for info in z.infolist():
                if not info.is_dir():
                    add_bytes(info.filename, z.read(info))
    elif os.path.isdir(reference):
        for dirpath, _, filenames in os.walk(reference):
            for f in filenames:
                p = os.path.join(dirpath, f)
                add_bytes(os.path.relpath(p, reference),
                          open(p, "rb").read())
    else:
        print(f"[FAIL] --reference {reference} not readable")
        return 1

    for root_dir in PACK_ROOTS + ["public", "voxelcraft/crates"]:
        base = os.path.join(ROOT, root_dir)
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for f in filenames:
                p = os.path.join(dirpath, f)
                rel = os.path.relpath(p, ROOT)
                try:
                    data = open(p, "rb").read()
                except OSError:
                    continue
                if hashlib.sha256(data).hexdigest() in ref_hashes:
                    print(f"[FAIL] byte-identical copy of reference: {rel}")
                    fails += 1
                if Image is not None and f.endswith(".png"):
                    try:
                        img = Image.open(p).convert("RGBA")
                        if hashlib.sha256(img.tobytes()).hexdigest() in ref_pixels:
                            print(f"[FAIL] pixel-identical copy: {rel}")
                            fails += 1
                    except Exception:
                        pass
    return fails


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reference",
                    help="EXTERNAL reference set (zip or dir) for the "
                         "byte/pixel identity check; never inside the repo")
    args = ap.parse_args()

    fails = audit_terms()
    fails += audit_forbidden_files()
    if args.reference:
        fails += audit_reference_identity(args.reference)

    if fails:
        print(f"\n[FAIL] {fails} violation(s) — see above")
        sys.exit(1)
    print("[PASS] legal audit clean: no trademark terms, no forbidden "
          "files" + (" , zero reference-identity copies"
                     if args.reference else ""))


if __name__ == "__main__":
    main()
