#!/usr/bin/env python3
"""Extract plain text from the v116b wiki captures and print key sections
for the Nether Update PART 2 research record (mobs + crimson/warped
families), values verification pass."""
import json, re, html as htmllib, sys

PAGES = ["Strider", "Piglin", "Hoglin", "Crimson_Stem",
         "Crimson_Hyphae", "Crimson_Planks", "Crimson_Fungus",
         "Warped_Fungus", "Crimson_Roots", "Warped_Roots",
         "Weeping_Vines", "Twisting_Vines", "Nether_Sprouts",
         "Shroomlight", "Warped_Wart_Block", "Soul_Torch",
         "Soul_Lantern", "Polished_Basalt", "Polished_Blackstone",
         "Polished_Blackstone_Bricks", "Crimson_Forest",
         "Warped_Forest", "Java_Edition_1.16"]

def load_text(name):
    with open(f"/home/z/my-project/voxelcraft/scripts/v116b_page_{name}.json") as f:
        d = json.load(f)
    data = d.get("data", d)
    h = data.get("html", "") or data.get("text", "")
    t = re.sub(r"<script[^>]*>.*?</script>", " ", h, flags=re.S)
    t = re.sub(r"<style[^>]*>.*?</style>", " ", t)
    t = re.sub(r"<[^>]+>", " ", t)
    t = htmllib.unescape(t)
    t = re.sub(r"\s+", " ", t)
    return t

WINDOWS = {
    "Strider": ["Health", "Hitbox", "damage", "speed", "lava", "breed",
                "warped fungus", "saddle", "ride", "cold", "shiver",
                "spawn", "drops", "zombified", "XP", "Behavior"],
    "Piglin": ["Health", "Hitbox", "damage", "speed", "barter", "gold",
               "angry", "attack", "spawn", "crossbow", "armor",
               "behavior", "hostile", "neutral", "portal", "drown",
               "convert", "zombified", "flee", "fear", "XP", "drops"],
    "Hoglin": ["Health", "Hitbox", "damage", "attack", "spawn",
               "crimson forest", "warped fungus", "fear", "repel",
               "flee", "breed", "crimson fungus", "baby", "drops",
               "porkchop", "zoglin", "XP", "Behavior", "passive",
               "retreat"],
    "Crimson_Stem": ["Hardness", "Blast", "tool", "craft", "planks",
                     "stripped", "smoker", "fuel", "hyphae"],
    "Crimson_Hyphae": ["Hardness", "Blast", "tool", "craft", "planks",
                       "stripped", "fuel"],
    "Crimson_Planks": ["Hardness", "Blast", "craft", "flammable",
                       "fuel", "stripped"],
    "Crimson_Fungus": ["Hardness", "craft", "hoglin", "repell",
                       "breed", "spawn", "crimson forest", "bone meal",
                       "tool", "fungi", "tree", "place"],
    "Warped_Fungus": ["Hardness", "craft", "hoglin", "repell",
                      "breed", "strider", "spawn", "warped forest",
                      "tool", "fungi", "tree", "place"],
    "Crimson_Roots": ["Hardness", "tool", "drops", "shears", "spawn",
                      "biome", "bone meal"],
    "Warped_Roots": ["Hardness", "tool", "drops", "shears", "spawn",
                     "biome", "bone meal"],
    "Weeping_Vines": ["Hardness", "tool", "drops", "growth", "grow",
                      "down", "biome", "shears", "bone meal", "age"],
    "Twisting_Vines": ["Hardness", "tool", "drops", "growth", "grow",
                       "up", "biome", "shears", "bone meal", "age",
                       "climb"],
    "Nether_Sprouts": ["Hardness", "tool", "drops", "shears",
                       "bone meal", "biome", "light"],
    "Shroomlight": ["Hardness", "Blast", "light", "tool", "craft",
                    "generate", "huge fungi", "wart", "shears",
                    "hoe", "drops"],
    "Warped_Wart_Block": ["Hardness", "Blast", "tool", "craft",
                          "generate", "wart", "silk", "hoe"],
    "Soul_Torch": ["light", "craft", "coal", "charcoal", "stick",
                   "soul sand", "soul soil", "piglin", "repel"],
    "Soul_Lantern": ["light", "craft", "nugget", "hang", "iron",
                     "tool", "hardness", "water", "soul torch"],
    "Polished_Basalt": ["Hardness", "Blast", "craft", "tool", "generate",
                        "chiseled", "smooth"],
    "Polished_Blackstone": ["Hardness", "Blast", "craft", "tool",
                            "generate", "slab", "stairs"],
    "Polished_Blackstone_Bricks": ["Hardness", "Blast", "craft", "tool",
                                   "generate", "cracked", "chiseled"],
    "Crimson_Forest": ["piglin", "hoglin", "generate", "fungi", "weird",
                       "vegetation", "fog", "nether", "biome", "spawn",
                       "ambient"],
    "Warped_Forest": ["piglin", "enderman", "generate", "fungi",
                      "vegetation", "fog", "nether", "biome", "spawn",
                      "strider"],
    "Java_Edition_1.16": ["hoglin", "piglin", "strider", "zoglin",
                          "crimson", "warped", "zombified piglin",
                          "soul fire", "soul torch", "new blocks",
                          "shroomlight", "weeping", "twisting"],
}

W = 700

def main():
    page = sys.argv[1] if len(sys.argv) > 1 else None
    for name in (PAGES if page is None else [page]):
        t = load_text(name)
        print(f"\n{'='*70}\nPAGE: {name}  ({len(t)} chars of text)\n{'='*70}")
        for kw in WINDOWS[name]:
            for m in re.finditer(re.escape(kw), t, re.I):
                s = max(0, m.start() - 120)
                print(f"\n--[{kw}]-- ...{t[s:s+W]}...")
                break

if __name__ == "__main__":
    main()
