#!/usr/bin/env python3
"""rename_terms.py — Phase 5: rename all ecosystem-coined terms to the
project's own original vocabulary.

Applies an ordered, case-aware replacement map (snake/Pascal/UPPER/Title)
across every text file in the repo, then renames asset files/folders that
carry coined names. Order matters: longer compounds run before their
shorter stems (hollowstone before hollow, void_gate_frame before
void_gate, aqua_step before emberhopper, ...).

Run from voxelcraft/: python3 scripts/rename_terms.py
"""
import os
import re
import sys

# ------------------------------------------------------------------ map --

# (snake_or_plain, replacement) — longest/most-specific FIRST is enforced
# below by sorting; this list documents the vocabulary.
SNAKE_PAIRS = [
    # dimension & marketing-era labels
    ("the_void", "the_void"), ("the_hollow", "the_hollow"),
    ("hollows update", "hollows update"),
    # end / ender family (never bare 'end')
    ("voidwyrm", "voidwyrm"), ("void_wyrm", "void_wyrm"),
    ("voidlings", "voidlings"), ("voidling", "voidling"),
    ("voidmite", "voidmite"),
    ("void_chest", "void_chest"), ("void_pearl", "void_pearl"),
    ("void_eye", "void_eye"),
    ("void_gate_frame", "void_gate_frame"), ("void_gate", "void_gate"),
    ("void_gateway", "void_gateway"), ("void_crystal", "void_crystal"),
    ("void_stone", "void_stone"), ("void_rod", "void_rod"),
    ("void_pillar", "void_pillar"), ("void_arrival", "void_arrival"),
    # hollow family
    ("hollowstone", "hollowstone"), ("hollowite", "hollowite"),
    ("hollow", "hollow"),
    # blight family
    ("blight_skeleton", "blight_skeleton"), ("blight_rose", "blight_rose"),
    ("blight", "blight"),
    # soul family
    ("spirit_sand_valley", "spirit_sand_valley"),
    ("spirit_sand", "spirit_sand"), ("spirit_soil", "spirit_soil"),
    ("spirit_fire", "spirit_fire"), ("spirit_torch", "spirit_torch"),
    ("spirit_lantern", "spirit_lantern"), ("spirit_campfire", "spirit_campfire"),
    ("spirit_speed", "spirit_speed"), ("spirit_dmg", "spirit_dmg"),
    ("spiritsand", "spiritsand"),
    # scarlet / viridian / mold
    ("scarlet", "scarlet"), ("viridian", "viridian"), ("mold", "mold"),
    # fluxstone
    ("fluxstone", "fluxstone"),
    # mobs — coined portmanteaus & distinctive coinages
    ("fuseling", "fuseling"), ("weepgeist", "weepgeist"),
    ("pigoblin", "pigoblin"), ("boarling", "boarling"), ("rotboar", "rotboar"),
    ("aqua_step", "aqua_step"), ("emberhopper", "emberhopper"),
    ("lurkshell", "lurkshell"), ("shroomcow", "shroomcow"),
    ("runecaller", "runecaller"), ("cleaver", "cleaver"),
    ("miragecaller", "miragecaller"),
    ("sootheling", "sootheling"), ("depthbrute", "depthbrute"),
    ("zephyr", "zephyr"), ("timberhaunt", "timberhaunt"),
    ("trufflehog", "trufflehog"),
    # blocks / items
    ("glowcap", "glowcap"), ("violetstone", "violetstone"),
    ("abyssprism", "abyssprism"), ("echo", "echo"),
    ("weeping_obsidian", "weeping_obsidian"),
    ("rebirth_anchor", "rebirth_anchor"),
    ("totem_of_revival", "totem_of_revival"),
    ("skywings", "skywings"),
]

# non-derivable specials, applied FIRST (longest first within the list)
SPECIALS = [
    ("Dimension::Void", "Dimension::Void"),
    ("Dimension::Hollow", "Dimension::Hollow"),
    ("Hollows Update", "Hollows Update"),
    ("Aquatic-era update", "Aquatic-era update"),
    ("Bountiful-era update", "Bountiful-era update"),
    ("Exploration-era update", "Exploration-era update"),
    ("Caves-era updates", "Caves-era updates"),
    ("Caves-era updates", "Caves-era updates"),
    ("the Void", "the Void"), ("The Void", "The Void"),
    ("Void Crystal", "Void Crystal"),
]

def pascal(s):
    return "".join(w[:1].upper() + w[1:] for w in s.split("_") if w)

def build_pairs():
    pairs = list(SPECIALS)
    for a, b in SNAKE_PAIRS:
        variants = {(a, b)}
        # PascalCase
        if "_" in a:
            variants.add((pascal(a), pascal(b)))
            # space forms
            variants.add((a.replace("_", " "), b.replace("_", " ")))
            variants.add((pascal(a).replace("", ""), pascal(b)))  # noop guard
            ta = " ".join(w.capitalize() for w in a.split("_"))
            tb = " ".join(w.capitalize() for w in b.split("_"))
            variants.add((ta, tb))
        else:
            variants.add((a.capitalize(), b.capitalize()))
        # UPPER (identifiers-with-underscores form)
        variants.add((a.upper(), b.upper()))
        pairs.extend(variants)
    # longest search-string first, then alphabetical for determinism
    pairs.sort(key=lambda p: (-len(p[0]), p[0]))
    return pairs

PAIRS = build_pairs()
# word-boundary-only regexes (substring-dangerous terms)
BOUNDARIED = [(re.compile(r"\bvex\b"), "wisp"),
              (re.compile(r"\bVex\b"), "Wisp")]

# ---------------------------------------------------------------- apply --

TEXT_EXT = {".rs", ".toml", ".json", ".md", ".sh", ".py", ".yml", ".yaml",
            ".html", ".glsl", ".txt", ".css", ".js", ".ts", ".tsx", ".mjs",
            ".mcmeta", ".gitignore", ".svg", ".d.ts"}

def apply_to_text(s):
    for a, b in PAIRS:
        if a and a in s:
            s = s.replace(a, b)
    for rx, b in BOUNDARIED:
        s = rx.sub(b, s)
    return s

ROOTS = [
    ".",                          # voxelcraft/ (crates, packs, scripts, docs)
    "../src", "../docs", "../scripts", "../.github", "../ci", "../tests",
    "../public", "../prisma", "../db", "../examples", "../mini-services",
]
SKIP_DIRS = {"target", "node_modules", ".git", ".next", "wasm-out",
             "build-artifact", "skills", "..", "upload"}
SKIP_FILES = {"Cargo.lock", "bun.lock", "package-lock.json"}

def iter_files():
    for root_dir in ROOTS:
        base = os.path.abspath(root_dir)
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for f in filenames:
                ext = os.path.splitext(f)[1].lower()
                if ext not in TEXT_EXT and f not in (".gitignore",):
                    continue
                if f in SKIP_FILES:
                    continue
                yield os.path.join(dirpath, f)

def rename_path_segments(path):
    """apply the term map to each segment of a path; return new path"""
    d, base = os.path.split(path)
    new_base = apply_to_text(base)
    return os.path.join(d, new_base) if new_base != base else path

def rename_tree(root_dir):
    """rename files & dirs (deepest first) under root_dir"""
    renames = 0
    all_paths = []
    for dirpath, dirnames, filenames in os.walk(root_dir):
        dirnames[:] = [d for d in dirnames if d not in ("target", ".git")]
        for f in filenames:
            all_paths.append(os.path.join(dirpath, f))
        for d in dirnames:
            all_paths.append(os.path.join(dirpath, d))
    # deepest first so children rename before their parents
    for p in sorted(all_paths, key=lambda x: -x.count(os.sep)):
        new_p = rename_path_segments(p)
        if new_p != p:
            if not os.path.exists(new_p):
                os.rename(p, new_p)
                renames += 1
            else:
                print(f"  !! skip (dest exists): {new_p}")
    return renames

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)) + "/..")
    n_files = 0
    for p in iter_files():
        try:
            s = open(p, encoding="utf-8").read()
        except (UnicodeDecodeError, IsADirectoryError):
            continue
        new = apply_to_text(s)
        if new != s:
            open(p, "w", encoding="utf-8").write(new)
            n_files += 1
    print(f"text: {n_files} files rewritten")
    for tree in ["assets-vault", "builtin-pack", "builtin-packs"]:
        n = rename_tree(tree)
        print(f"tree: {n} paths renamed under {tree}/")
    print("DONE")
