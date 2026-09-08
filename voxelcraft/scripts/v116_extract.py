#!/usr/bin/env python3
"""Extract plain text from the v116 wiki captures and print key sections
for the Nether Update research record (values verification pass)."""
import json, re, html as htmllib, sys

PAGES = ["Respawn_Anchor", "Target", "Crying_Obsidian", "Soul_Soil",
         "Soul_Fire", "Basalt", "Blackstone", "Ancient_Debris",
         "Netherite_Scrap", "Netherite_Ingot", "Nether_Gold_Ore",
         "Chain", "Gilded_Blackstone", "Lodestone"]

def load_text(name):
    with open(f"/home/z/my-project/voxelcraft/scripts/v116_page_{name}.json") as f:
        d = json.load(f)
    data = d.get("data", d)
    h = data.get("html", "") or data.get("text", "")
    # strip tags
    t = re.sub(r"<script[^>]*>.*?</script>", " ", h, flags=re.S)
    t = re.sub(r"<style[^>]*>.*?</style>", " ", t, flags=re.S)
    t = re.sub(r"<[^>]+>", " ", t)
    t = htmllib.unescape(t)
    t = re.sub(r"\s+", " ", t)
    return t

# keyword windows per page: (page, [keywords], window chars)
WINDOWS = {
    "Respawn_Anchor": ["charge", "glowstone", "respawn point", "explode",
                       "maximum", "deplete", "Nether", "light", "craft",
                       "hardness", "blast resistance", "water"],
    "Target": ["redstone", "signal", "strength", "arrow", "projectile",
               "emits", "craft", "hardness", "accuracy", "power"],
    "Crying_Obsidian": ["light", "portal", "respawn", "glowstone",
                        "craft", "hardness", "obsidian", "piglin",
                        "barter", "blast"],
    "Soul_Soil": ["fire", "soul", "basalt", "blue", "nether", "campfire",
                  "spawn", "valley"],
    "Soul_Fire": ["soul sand", "soul soil", "blue", "damage", "light",
                  "flint", "eternal", "nether"],
    "Basalt": ["generat", "soul sand", "lava", "water", "polished",
               "column", "nether", "craft", "hardness"],
    "Blackstone": ["gilded", "polished", "chiseled", "craft", "nether",
                   "hardness", "gold", "bastion"],
    "Ancient_Debris": ["generat", "chunk", "vein", "netherite scrap",
                       "smelt", "blast", "y-level", "resistance",
                       "diamond", "spawn", "nether"],
    "Netherite_Scrap": ["smelt", "ancient debris", "netherite ingot",
                        "gold", "explosion", "float"],
    "Netherite_Ingot": ["scrap", "gold", "upgrade", "smithing",
                        "recipe", "explosion", "float", "fire"],
    "Nether_Gold_Ore": ["gold nugget", "drops", "fortune", "smelt",
                        "generat", "nether", "hardness", "experience"],
    "Chain": ["craft", "iron", "hardness", "lantern", "hang",
              "generate", "bastion", "piglin"],
    "Gilded_Blackstone": ["gold nugget", "drops", "chance", "generat",
                          "bastion", "hardness", "fortune"],
    "Lodestone": ["compass", "nether", "dimension", "craft", "chiseled",
                  "stone", "iron", "hardness"],
}

W = 700  # window chars per hit

def main():
    page = sys.argv[1] if len(sys.argv) > 1 else None
    for name in (PAGES if page is None else [page]):
        t = load_text(name)
        print(f"\n{'='*70}\nPAGE: {name}  ({len(t)} chars of text)\n{'='*70}")
        for kw in WINDOWS[name]:
            for m in re.finditer(re.escape(kw), t, re.I):
                s = max(0, m.start() - 120)
                print(f"\n--[{kw}]-- ...{t[s:s+W]}...")
                break  # first hit per keyword only (windows overlap heavily)

if __name__ == "__main__":
    main()
