#!/usr/bin/env python3
"""Focused extraction: strip HTML, search the plain text of each capture
for the exact sentences around the flagged mechanics."""
import json
import re
import html

FILES = {
    "sweetberry": "scripts/v114_page_sweetberrybush.json",
    "bamboo": "scripts/v114_page_bamboo.json",
    "campfire": "scripts/v114_page_campfire.json",
    "barrel": "scripts/v114_page_barrel.json",
    "fox": "scripts/v114_page_fox.json",
}

QUERIES = {
    "sweetberry": ["slow", "damage", "moving", "stage", "harvest", "growth", "mature",
                   "sweet berries,", "food", "hunger", "saturation", "compost", "breed"],
    "bamboo": ["fuel", "smelt", "grow", "random tick", "chance", "height", "jungle",
               "panda", "bone meal", "light"],
    "campfire": ["craft", "stick", "coal", "log", "cook", "30 seconds", "600",
                 "charcoal", "silk", "damage", "shovel", "smoke", "hay"],
    "barrel": ["craft", "plank", "slab", "slot", "27", "hardness", "drops", "hopper"],
    "fox": ["health", "attack", "chicken", "rabbit", "cod", "salmon", "tropical",
            "turtle", "breed", "sweet berries", "trust", "spawn", "group", "taiga",
            "hitbox", "height", "width", "speed", "immun"],
}

def plain_text(path):
    d = json.load(open(path))
    def hunt(obj):
        if isinstance(obj, str) and len(obj) > 5000:
            return obj
        if isinstance(obj, dict):
            for v in obj.values():
                r = hunt(v)
                if r:
                    return r
        if isinstance(obj, list):
            for v in obj:
                r = hunt(v)
                if r:
                    return r
        return None
    raw = hunt(d) or ""
    # strip scripts/styles then tags
    raw = re.sub(r"<(script|style)[^>]*>.*?</\1>", " ", raw, flags=re.S)
    raw = re.sub(r"<[^>]+>", " ", raw)
    raw = html.unescape(raw)
    raw = re.sub(r"\s+", " ", raw)
    return raw

for name, path in FILES.items():
    txt = plain_text(path)
    print(f"\n{'#'*72}\n# {name}: {len(txt)} chars of plain text\n{'#'*72}")
    for q in QUERIES[name]:
        # find up to 2 occurrences, print 200-char window
        for m in list(re.finditer(re.escape(q), txt, re.I))[:2]:
            s = max(0, m.start() - 160)
            e = min(len(txt), m.end() + 220)
            snippet = txt[s:e].strip()
            print(f"\n[{q}] ...{snippet}...")
