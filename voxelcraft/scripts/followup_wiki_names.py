#!/usr/bin/env python3
"""followup_wiki_names.py — pass 2 of the 2026-09-21 restoration: the
PascalCase-underscore wiki page references (w/Ender_Chest, w/Respawn_Anchor,
backlog_page_Soul_Sand_Valley.json, ...) that the snake-case pair set
did not cover, plus sentence-case display-name leftovers.

Run from voxelcraft/: python3 scripts/followup_wiki_names.py
"""
import os

REPL = [
    # wiki page names (PascalCase + underscore) — coined -> real
    ("Soul_Sand_Valley", "Soul_Sand_Valley"),
    ("Totem_of_Undying", "Totem_of_Undying"),
    ("Crying_Obsidian", "Crying_Obsidian"),
    ("Respawn_Anchor", "Respawn_Anchor"),
    ("End_Crystal", "End_Crystal"),
    ("End_Portal_Frame", "End_Portal_Frame"),
    ("End_Gateway", "End_Gateway"),
    ("End_Pillar", "End_Pillar"),
    ("End_Arrival", "End_Arrival"),
    ("Ender_Chest", "Ender_Chest"),
    ("Ender_Pearl", "Ender_Pearl"),
    ("Eye_of_Ender", "Eye_of_Ender"),
    ("End_Stairs", "End_Stairs"),        # guard: none expected
    ("End_Stone", "End_Stone"),
    ("End_Rod", "End_Rod"),
    ("End_Portal", "End_Portal"),
    ("Ender_Dragon", "Ender_Dragon"),
    ("Enderman", "Enderman"),
    ("Endermite", "Endermite"),
    ("Enderdragon", "Enderdragon"),
    ("End_Fire", "End_Fire"),            # guard: none expected
    ("Chorus_Fruit", "Chorus_Fruit"),       # guard: none expected
    ("Nether_Star", "Nether_Star"),
    ("Nether_Wastes", "Nether_Wastes"),
    ("Nether_Fortress", "Nether_Fortress"),
    ("Nether_Bricks", "Nether_Bricks"),
    ("Netherrack", "Netherrack"),
    ("Netherite", "Netherite"),
    ("Nether_Quartz_Ore", "Nether_Quartz_Ore"),
    ("Redstone", "Redstone"),
    ("Elytra", "Elytra"),
    ("Shroomlight", "Shroomlight"),
    ("Purpur", "Purpur"),
    ("Prismarine", "Prismarine"),
    ("Wither_Skeleton", "Wither_Skeleton"),
    ("Wither_Rose", "Wither_Rose"),
    ("The_Wither", "The_Wither"),
    ("Crimson_Nylium", "Crimson_Nylium"),
    ("Warped_Nylium", "Warped_Nylium"),
    ("Crimson_Forest", "Crimson_Forest"),
    ("Warped_Forest", "Warped_Forest"),
    ("Creeper", "Creeper"),
    ("Ghast", "Ghast"),
    ("Piglin", "Piglin"),
    ("Hoglin", "Hoglin"),
    ("Zoglin", "Zoglin"),
    ("Strider", "Strider"),
    ("Shulker", "Shulker"),
    ("Mooshroom", "Mooshroom"),
    ("Evoker", "Evoker"),
    ("Illusioner", "Illusioner"),
    ("Allay", "Allay"),
    ("Warden", "Warden"),
    ("Creaking", "Creaking"),
    ("Sniffer", "Sniffer"),
    ("Vindicator", "Vindicator"),
    ("Soulsand", "Soulsand"),
    ("Soul_Soil", "Soul_Soil"),
    ("Soul_Sand", "Soul_Sand"),
    ("Soul_Fire", "Soul_Fire"),
    ("Soul_Torch", "Soul_Torch"),
    ("Soul_Lantern", "Soul_Lantern"),
    ("Soul_Campfire", "Soul_Campfire"),
    ("Soul_Speed", "Soul_Speed"),
    ("Depth_Strider", "Depth_Strider"),
    # sentence-case display-name leftovers
    ("Totem of undying", "Totem of undying"),
    ("totem of undying", "totem of undying"),
    ("the Nether Update", "the Nether Update"),
]
# longest first
REPL.sort(key=lambda p: (-len(p[0]), p[0]))

TEXT_EXT = {".rs", ".toml", ".json", ".md", ".sh", ".py", ".yml", ".yaml",
            ".html", ".glsl", ".txt", ".css", ".js", ".ts", ".tsx", ".mjs"}
SKIP_DIRS = {"target", "node_modules", ".git", ".next", "wasm-out",
             "build-artifact", "skills", "upload", "__pycache__"}
EXCLUDE = {"docs/WORKLOG.md", "docs/LEGAL-COMPLIANCE.md",
           "crates/vc-pack/src/legacy_aliases.rs",
           "scripts/gen_legacy_aliases.py", "scripts/rename_terms.py",
           "scripts/restore_real_names.py", "scripts/followup_wiki_names.py"}
ROOTS = [".", "../src", "../docs", "../scripts", "../.github", "../ci",
         "../tests", "../public", "../prisma", "../examples"]


def main():
    os.chdir(os.path.dirname(os.path.abspath(__file__)) + "/..")
    base = os.path.abspath(".")
    root = os.path.abspath("..")
    n = 0
    for r in ROOTS:
        rbase = os.path.abspath(r)
        if not os.path.isdir(rbase):
            continue
        for dirpath, dirnames, filenames in os.walk(rbase):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for f in filenames:
                if os.path.splitext(f)[1].lower() not in TEXT_EXT:
                    continue
                p = os.path.join(dirpath, f)
                rel = os.path.relpath(p, base).replace(os.sep, "/")
                rel_root = os.path.relpath(p, root).replace(os.sep, "/")
                if rel in EXCLUDE or rel_root in EXCLUDE or rel.startswith("../"):
                    if rel_root in EXCLUDE:
                        continue
                try:
                    s = open(p, encoding="utf-8").read()
                except (UnicodeDecodeError, IsADirectoryError):
                    continue
                t = s
                for a, b in REPL:
                    if a in t:
                        t = t.replace(a, b)
                if t != s:
                    open(p, "w", encoding="utf-8").write(t)
                    n += 1
    print(f"pass2: {n} files rewritten")


if __name__ == "__main__":
    main()
