#!/usr/bin/env python3
"""Extract plain text from the v115 wiki captures and print key sections
for the Buzzy Bees research record (values verification pass)."""
import json, re, html as htmllib, sys

PAGES = ["Bee", "Beehive", "Bee_nest", "Honey_Block", "Honey_Bottle",
         "Honeycomb", "Honeycomb_Block"]

def load_text(name):
    with open(f"/home/z/my-project/voxelcraft/scripts/v115_page_{name}.json") as f:
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
    "Bee": ["health points", "poison", "sting", "pollinat", "angry", "breeding",
            "spawn", "hive", "campfire", "flower", "damage", "speed", "die",
            "honey level", "entity data", "Advancement", "drop"],
    "Beehive": ["hardness", "honey level", "shears", "bottle", "craft",
                "honeycomb", "collection", "pollinat", "note block"],
    "Bee_nest": ["generat", "chance", "plains", "forest", "flower", "shears",
                 "bottle", "honey level", "spawn", "biome"],
    "Honey_Block": ["slows", "slide", "jump", "craft", "fall damage",
                    "entity", "speed", "piston", "stuck"],
    "Honey_Bottle": ["restores", "saturation", "poison", "craft", "stack",
                     "hunger", "drink", "glass bottle"],
    "Honeycomb": ["shears", "wax", "craft", "obtain", "honey level"],
    "Honeycomb_Block": ["craft", "decorat", "fuel", "note block"],
}

page = sys.argv[1] if len(sys.argv) > 1 else None
for name in (PAGES if page is None else [page]):
    t = load_text(name)
    print(f"\n{'='*70}\n### {name} (len {len(t)})\n{'='*70}")
    seen = []
    for kw in WINDOWS.get(name, []):
        for m in re.finditer(re.escape(kw), t, re.I):
            s = max(0, m.start() - 160)
            e = min(len(t), m.end() + 260)
            # skip if overlapping an already-printed window
            if any(abs(s - ps) < 200 for ps in seen):
                continue
            seen.append(s)
            print(f"\n--[{kw}]-- {t[s:e].strip()}")
            if len(seen) > 18:
                break
        if len(seen) > 18:
            break
