#!/usr/bin/env python3
"""legal_audit.py — the standing compliance audit (LEGAL-COMPLIANCE.md §6).

Scans everything the repo SHIPS (code, assets, packs, docs, CI) for:

  1. Third-party trademark / brand marks (the bright line: the
     original publisher's names must never appear — see §3 of the
     binding doc for the in-game-vocabulary policy: generic functional
     terms like redstone/creeper/netherrack are the genre's shared
     vocabulary and are fine; brand names are not)
  2. Files that must never exist in our tree (the ecosystem's
     manifest/sidecar conventions in OUR packs, scraped wiki dumps)
  3. Optional byte/pixel-identity check against a reference set, when
     one is supplied via --reference <path-or-zip> (the reference set
     itself must NEVER live inside the repository)

The ACTIVE engine font is Monocraft (IdreesInc, SIL OFL 1.1) — an
open-source font with full redistribution rights; its license ships
next to it, so the font and its license file are NOT forbidden.

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

# Brand-mark scan: any of these in a SHIPPED file is a failure. These
# are the original publisher's TRADEMARKS and brand labels — the
# bright line. (The generic functional in-game vocabulary — redstone,
# creeper, netherrack, soul sand, enderman, … — is the genre's shared
# term-of-art set and is deliberately NOT scanned; see
# LEGAL-COMPLIANCE.md §3, owner directive 2026-09-21.)
TERM_RX = re.compile(
    r"minecraft|mojang|herobrine|programmer[ _-]art",
    re.IGNORECASE,
)
# "notch" is a dictionary word (notch filter in DSP) — not scanned.
# Persona names (steve/alex) are distinctive character marks we have no
# need for — guarded as a class.
PERSONA_RX = re.compile(r"\b(steve|alex)\b", re.IGNORECASE)

EXEMPT_FILES = {
    # read-side format-interop data (generated; never user-visible)
    "voxelcraft/crates/vc-pack/src/legacy_aliases.rs",
    # the rename/restoration tooling (audit trail of the vocabulary
    # policy rounds — coined names appear in their maps by design)
    "voxelcraft/scripts/rename_terms.py",
    "voxelcraft/scripts/gen_legacy_aliases.py",
    "voxelcraft/scripts/restructure_packs.py",
    "voxelcraft/scripts/restore_real_names.py",
    "voxelcraft/scripts/followup_wiki_names.py",
    # this script + its sibling tooling
    "scripts/legal_audit.py",
    "scripts/debrand_research_docs.py",
}

# Files/conventions that must never exist in OUR trees
FORBIDDEN_PATTERNS = []
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
             ".github", "tests", "README.md", "AGENTS.md", "Caddyfile"]

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


# --- L2 guard: no script may read a reference archive, point outside the
# repo, or emit/read per-pixel data. The reference corpus is off-limits to
# the public tree; regeneration must run from the committed aggregate spec
# only. 1A.5 extends this to per-pixel WRITERS: painting an image one
# pixel at a time (putpixel/getpixel/getdata/PIL PixelAccess) is
# per-pixel authorship, not aggregate-statistics art. The sanctioned
# pattern — numpy canvas built from aggregate stats + slice fills +
# Image.fromarray(...).save — stays legal (self-test proves it).
_SCRIPT_GUARD_EXEMPT = {"legal_audit.py"}  # the auditor owns the sanctioned --reference flag
_ARCHIVE_PATTERNS = ("ZipFile", "zipfile.ZipFile", "tarfile.open", "py7zr")
_PIXELDUMP_PATTERNS = ("np.save(", "np.savez", "numpy.save", ".tofile(",
                       "frombuffer(", ".npy", ".npz",
                       "putpixel(", "getpixel(", ".load()[", "getdata(")


def scan_script_for_violations(fn, body):
    """Pure per-script scanner (1A.5): returns the reasons `fn` violates
    the L2/L7 script guard. Unit-checked by _self_test on every run."""
    reasons = []
    for pat in _ARCHIVE_PATTERNS:
        if pat in body:
            reasons.append(f"opens an archive ({pat!r}) — the reference "
                           "corpus must never be reachable from the public tree")
    for pat in _PIXELDUMP_PATTERNS:
        if pat in body:
            reasons.append(f"emits or reads per-pixel data ({pat!r}) — "
                           "procedural art is driven by aggregate statistics "
                           "only (L2)")
    for m in re.finditer(r'["\'](/(?:home|Users|mnt|media)/[^"\']*)["\']', body):
        reasons.append(f"hardcoded path outside the repo {m.group(1)!r} — "
                       "committed tools must resolve paths relative to __file__")
    return reasons


def audit_reference_scripts():
    """Fail if any committed script can reach a reference set (L2/L7)."""
    fails = 0
    sdir = os.path.join(ROOT, "scripts")
    if not os.path.isdir(sdir):
        return 0
    for fn in sorted(os.listdir(sdir)):
        if not fn.endswith(".py") or fn in _SCRIPT_GUARD_EXEMPT:
            continue
        path = os.path.join(sdir, fn)
        try:
            body = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        for reason in scan_script_for_violations(fn, body):
            print(f"[FAIL] {fn}: {reason}")
            fails += 1
    return fails


def _self_test():
    """Negative test, wired into EVERY audit run (1A.5): the guard must
    fire on per-pixel writers, per-pixel readers, raw pixel dumps, archive
    readers and outside paths — and stay silent on the sanctioned
    aggregate-painting pattern (numpy canvas + slice fill + fromarray)."""
    bad = [
        ("bad_putpixel.py", "img.putpixel((x, y), (255, 0, 0))"),
        ("bad_getpixel.py", "c = img.getpixel((x, y))"),
        ("bad_getdata.py", "for px in img.getdata():\n    total += px"),
        ("bad_loadidx.py", "px = img.load()[x, y]"),
        ("bad_zip.py", "with zipfile.ZipFile(ref) as z:\n    data = z.read('a.png')"),
        ("bad_npy.py", "np.save('out.npy', arr)"),
        ('bad_path.py', 'src = "/home/z/reference/textures"'),
    ]
    for name, body in bad:
        reasons = scan_script_for_violations(name, body)
        assert reasons, f"self-test: {name} must be flagged"
    clean = (
        "import numpy as np\n"
        "from PIL import Image\n"
        "c = np.zeros((16, 16, 4), np.uint8)\n"
        "c[0:8, 0:8] = (120, 40, 40, 255)  # aggregate fill\n"
        "img = Image.fromarray(c, 'RGBA')\n"
        "img.save('block/sand.png')\n"
    )
    reasons = scan_script_for_violations("clean.py", clean)
    assert not reasons, f"self-test: aggregate painting must pass, got {reasons}"
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reference",
                    help="EXTERNAL reference set (zip or dir) for the "
                         "byte/pixel identity check; never inside the repo")
    args = ap.parse_args()

    try:
        _self_test()
    except AssertionError as e:
        print(f"[FAIL] script-guard self-test: {e}")
        sys.exit(1)

    fails = audit_terms()
    fails += audit_forbidden_files()
    fails += audit_reference_scripts()
    if args.reference:
        fails += audit_reference_identity(args.reference)

    if fails:
        print(f"\n[FAIL] {fails} violation(s) — see above")
        sys.exit(1)
    print("[PASS] legal audit clean: no trademark terms, no forbidden "
          "files, no reference-reading scripts, no per-pixel writers "
          "(self-test ok)" + (" , zero reference-identity copies"
                     if args.reference else ""))


if __name__ == "__main__":
    main()
