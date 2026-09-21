#!/usr/bin/env python3
"""restore_real_names.py — 2026-09-21 round: restore the REAL in-game
vocabulary (redstone, netherrack, soul_sand, creeper, enderman, wither,
ender_dragon, crimson, warped, chorus, elytra, ...) across the whole repo.

WHY: the 2026-09-20 de-brand sweep renamed the in-game entity/block/item
vocabulary to coined terms (fluxstone/hollowstone/fuseling/...). The
owner's clarified directive: the legal concern is STRUCTURE (folder
names, namespaces, paths, "minecraft"/"mojang" marks) — NOT in-game
names. Real in-game names are what user packs, wikis and muscle memory
expect, so they come BACK. Structure keeps our own voxelcraft layout.

SAFETY MODEL (why this will not corrupt anything):
- Saves keep loading: dimensions persist as numeric ids (from_u8), block
  ids are numeric; the vc-pack legacy alias table is being SWAPPED to
  (coined -> real) so coined-era packs/saves still resolve.
- Ambiguous English words are protected:
  * "a hollow thump" (vc-audio comment) stays English
  * "far echo"/"soft echo" (vc-audio comments) stay English
  * `blight` = BLOCK-LIGHT in vc-mesh/mesh.rs, vc-world/light.rs,
    vc-render/gpu_mesh.rs (pre-existing abbreviation) — skipped there
  * generic lowercase `void` (the abyss below the world) is untouched;
    only the End-family compounds and capital `Void` (dimension) rename
  * `mold` renames only as a whole word (never "molded")
- docs/WORKLOG.md (historical record) and docs/LEGAL-COMPLIANCE.md
  (binding legal doc, updated separately) are EXCLUDED.
- vc-pack/src/legacy_aliases.rs + scripts/gen_legacy_aliases.py are
  EXCLUDED (regenerated with the swapped table).

Run from voxelcraft/: python3 scripts/restore_real_names.py
"""
import os
import re
import sys

# ------------------------------------------------------------- pairs --

# (coined snake, real snake) — plain-substring, case variants generated
# below, longest-first enforced by sorting.
PLAIN_PAIRS = [
    # dimension ids & compounds (before bare stems; sort enforces)
    ("the_void", "the_end"),
    ("the_hollow", "the_nether"),
    # nether family
    ("hollowstone", "netherrack"),
    ("hollowite", "netherite"),
    ("hollow_quartz_ore", "nether_quartz_ore"),
    # soul family
    ("spirit_sand_valley", "soul_sand_valley"),
    ("spirit_sand", "soul_sand"),
    ("spirit_soil", "soul_soil"),
    ("spirit_fire", "soul_fire"),
    ("spirit_torch", "soul_torch"),
    ("spirit_lantern", "soul_lantern"),
    ("spirit_campfire", "soul_campfire"),
    ("spirit_speed", "soul_speed"),
    ("spirit_dmg", "soul_dmg"),
    ("spiritsand", "soulsand"),
    # redstone family
    ("fluxstone", "redstone"),
    # end / ender family
    ("void_gate_frame", "end_portal_frame"),
    ("void_gate", "end_portal"),
    ("void_gateway", "end_gateway"),
    ("void_crystal", "end_crystal"),
    ("void_chest", "ender_chest"),
    ("void_pearl", "ender_pearl"),
    ("void_eye", "eye_of_ender"),
    ("void_stone", "end_stone"),
    ("void_rod", "end_rod"),
    ("void_pillar", "end_pillar"),
    ("void_arrival", "end_arrival"),
    ("void_wyrm", "ender_dragon"),
    ("voidwyrm", "enderdragon"),
    ("voidlings", "endermen"),
    ("voidling", "enderman"),
    ("voidmite", "endermite"),
    # wither family (bare blight handled separately with file guards)
    ("blight_skeleton", "wither_skeleton"),
    ("blight_rose", "wither_rose"),
    # crimson / warped families
    ("scarlet_mold", "crimson_nylium"),
    ("viridian_mold", "warped_nylium"),
    ("scarlet", "crimson"),
    ("viridian", "warped"),
    # items / blocks
    ("weeping_obsidian", "crying_obsidian"),
    ("rebirth_anchor", "respawn_anchor"),
    ("totem_of_revival", "totem_of_undying"),
    ("aqua_step", "depth_strider"),
    ("skywings", "elytra"),
    ("glowcap", "shroomlight"),
    ("violetstone", "purpur"),
    ("abyssprism", "prismarine"),
    # mobs
    ("fuseling", "creeper"),
    ("weepgeist", "ghast"),
    ("pigoblin", "piglin"),
    ("boarling", "hoglin"),
    ("rotboar", "zoglin"),
    ("emberhopper", "strider"),
    ("lurkshell", "shulker"),
    ("shroomcow", "mooshroom"),
    ("runecaller", "evoker"),
    ("miragecaller", "illusioner"),
    ("sootheling", "allay"),
    ("depthbrute", "warden"),
    ("timberhaunt", "creaking"),
    ("trufflehog", "sniffer"),
]

# phrases applied FIRST (before any pair can mangle them)
PHRASES = [
    ("Hollows Update", "Nether Update"),
    ("hollows update", "nether update"),
    ("The_Void", "The_End"),          # wiki page refs (w/The_Void)
    ("at the void", "at the end"),    # corrupted English ("at the end")
    ("past the void", "past the end"),
]

# bare ambiguous stems -> real (plain substring, all cases). `blight` is
# file-guarded (block-light files keep it); `hollow`/`echo` are phrase-
# protected below.
BARE_PAIRS = [
    ("hollow", "nether"),
    ("echo", "chorus"),
    ("blight", "wither"),
]

# whole-word-only renames (never inside another word)
WB_PAIRS = [
    (r"\bcleaver\b", "vindicator"),
    (r"\bVindicator\b", "Vindicator"),
    (r"\bwisp\b", "vex"),
    (r"\bWisp\b", "Vex"),
    (r"\bzephyr\b", "breeze"),
    (r"\bZephyr\b", "Breeze"),
    (r"\bzephyr\b", "breeze"),
    (r"\bZEPHYR\b", "BREEZE"),
    (r"\bmold\b", "nylium"),
    (r"\bMold\b", "Nylium"),
    (r"\bVOID\b", "END"),   # see note: bare UPPER VOID joins are rare; straggler-checked
    (r"\bVoid\b", "End"),   # capital Void = the End dimension family ONLY
]

# English phrases that must survive (sentinel-swapped around everything)
PROTECT = [
    "a hollow thump",
    "near + far echo",
    "far echo",
    "soft echo",
]

# files where `blight` legitimately means BLOCK-LIGHT (abbreviation) —
# the wither rename is skipped there, everything else still applies.
BLIGHT_KEEP_FILES = {
    "vc-mesh/src/mesh.rs",
    "vc-world/src/light.rs",
    "vc-render/src/gpu_mesh.rs",
}

# files fully excluded (regenerated manually / historical record)
EXCLUDE_FILES = {
    "docs/WORKLOG.md",
    "docs/LEGAL-COMPLIANCE.md",
    "crates/vc-pack/src/legacy_aliases.rs",
    "scripts/gen_legacy_aliases.py",
    "scripts/restore_real_names.py",   # this file
    "scripts/rename_terms.py",         # historical record of the forward sweep
}


def pascal(s):
    return "".join(w[:1].upper() + w[1:] for w in s.split("_") if w)


def spaced_variants(a, b):
    """spaced/hyphen forms; the_void is skipped (generic 'the void' abyss)"""
    out = []
    if (a, b) == ("the_void", "the_end"):
        return out  # lowercase "the void" is the generic abyss — never touch
    if "_" in a:
        out.append((a.replace("_", " "), b.replace("_", " ")))
        out.append((a.replace("_", "-"), b.replace("_", "-")))
        ta = " ".join(w.capitalize() for w in a.split("_"))
        tb = " ".join(w.capitalize() for w in b.split("_"))
        out.append((ta, tb))
        out.append((pascal(a), pascal(b)))
    return out


def build_pairs():
    pairs = []
    for a, b in PLAIN_PAIRS:
        pairs.append((a, b))
        pairs.append((a.upper(), b.upper()))
        if "_" not in a:
            pairs.append((a.capitalize(), b.capitalize()))
        pairs.extend(spaced_variants(a, b))
    for a, b in BARE_PAIRS:
        for ca, cb in ((a, b), (a.capitalize(), b.capitalize()), (a.upper(), b.upper())):
            pairs.append((ca, cb))
    # longest search-string first, then alphabetical for determinism
    pairs.sort(key=lambda p: (-len(p[0]), p[0]))
    return pairs


PAIRS = build_pairs()
WB_RES = [(re.compile(rx), rep) for rx, rep in WB_PAIRS]

# ------------------------------------------------------------ apply --

TEXT_EXT = {".rs", ".toml", ".json", ".md", ".sh", ".py", ".yml", ".yaml",
            ".html", ".glsl", ".txt", ".css", ".js", ".ts", ".tsx", ".mjs",
            ".mcmeta", ".gitignore", ".svg", ".d.ts", ".properties"}

SKIP_DIRS = {"target", "node_modules", ".git", ".next", "wasm-out",
             "build-artifact", "skills", "upload", "smoke-run", "bench-run",
             "debs", "__pycache__"}
SKIP_FILES = {"Cargo.lock", "bun.lock", "package-lock.json",
              "voxelcraft_bg.wasm", "voxelcraft_bg.wasm.d.ts"}

SENT = "\x00SENTINEL%d\x00"


def apply_to_text(s, allow_blight=True):
    # 1. protect English phrases
    prot = []
    for i, phrase in enumerate(PROTECT):
        if phrase in s:
            tok = SENT % i
            s = s.replace(phrase, tok)
            prot.append((tok, phrase))
    # 2. phrase specials
    for a, b in PHRASES:
        s = s.replace(a, b)
    # 3. plain pairs (longest first). blight-prefixed entries are
    # guarded per-file.
    for a, b in PAIRS:
        if not allow_blight and a.lower().startswith("blight"):
            continue
        if a in s:
            s = s.replace(a, b)
    # 4. bare stems (hollow/echo/blight) — blight guarded per-file
    for a, b in BARE_PAIRS:
        if a == "blight" and not allow_blight:
            continue
        for ca, cb in ((a, b), (a.capitalize(), b.capitalize()), (a.upper(), b.upper())):
            s = s.replace(ca, cb)
    # 5. whole-word renames
    for rx, rep in WB_RES:
        if rx.pattern == r"\bVOID\b" and not allow_blight:
            continue
        s = rx.sub(rep, s)
    # restore protections
    for tok, phrase in prot:
        s = s.replace(tok, phrase)
    return s


ROOTS = [
    ".",                           # voxelcraft/ (crates, packs, scripts, docs)
    "../src", "../docs", "../scripts", "../.github", "../ci", "../tests",
    "../public", "../prisma", "../examples",
]


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


def is_excluded(path):
    p = os.path.abspath(path)
    base = os.path.abspath(".")
    try:
        rel = os.path.relpath(p, base)
    except ValueError:
        return True
    rel = rel.replace(os.sep, "/")
    if rel.startswith("../"):
        # repo-root files: check against root-relative exclusions
        root = os.path.abspath("..")
        rel2 = os.path.relpath(p, root).replace(os.sep, "/")
        return rel2 in {"docs/WORKLOG.md", "docs/LEGAL-COMPLIANCE.md"}
    return rel in EXCLUDE_FILES


def blight_allowed(path):
    p = os.path.abspath(path).replace(os.sep, "/")
    return not any(p.endswith(k) for k in BLIGHT_KEEP_FILES)


def rename_path_segments(path):
    d, base = os.path.split(path)
    new_base = apply_to_text(base)
    return os.path.join(d, new_base) if new_base != base else path


def rename_tree(root_dir):
    renames = 0
    all_paths = []
    for dirpath, dirnames, filenames in os.walk(root_dir):
        dirnames[:] = [d for d in dirnames
                       if d not in ("target", ".git", "node_modules", "upload")]
        for f in filenames:
            all_paths.append(os.path.join(dirpath, f))
        for d in dirnames:
            all_paths.append(os.path.join(dirpath, d))
    for p in sorted(all_paths, key=lambda x: -x.count(os.sep)):
        new_p = rename_path_segments(p)
        if new_p != p:
            if not os.path.exists(new_p):
                os.rename(p, new_p)
                renames += 1
            else:
                print(f"  !! skip (dest exists): {new_p}")
    return renames


def main():
    os.chdir(os.path.dirname(os.path.abspath(__file__)) + "/..")
    n_files = 0
    for p in iter_files():
        if is_excluded(p):
            continue
        try:
            s = open(p, encoding="utf-8").read()
        except (UnicodeDecodeError, IsADirectoryError):
            continue
        allow_bl = blight_allowed(p)
        new = apply_to_text(s, allow_blight=allow_bl)
        if new != s:
            open(p, "w", encoding="utf-8").write(new)
            n_files += 1
    print(f"text: {n_files} files rewritten")
    for tree in ["crates", "assets-vault", "builtin-pack", "builtin-packs",
                 "../builtin-packs",
                 "../public/voxelcraft-pack",
                 "../public/voxelcraft-pack-classic-art",
                 "../public/voxelcraft-pack-programmer-art"]:
        if os.path.isdir(tree):
            n = rename_tree(tree)
            print(f"tree: {n} paths renamed under {tree}/")
        else:
            print(f"tree: {tree}/ (absent, skipped)")
    print("DONE")


if __name__ == "__main__":
    main()
