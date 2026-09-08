#!/usr/bin/env python3
"""Extract the wikitext from the v114 research captures and print the
recipe / mechanics sections flagged for re-verification:
- sweet berry bush slow multiplier + damage wording
- bamboo fuel ticks
- campfire crafting grid
- barrel crafting grid
- bamboo jungle generation rate
"""
import json
import re
import sys

FILES = [
    "scripts/v114_page_sweetberrybush.json",
    "scripts/v114_page_bamboo.json",
    "scripts/v114_page_campfire.json",
    "scripts/v114_page_barrel.json",
]

KEYS = [
    "crafting", "recipe", "fuel", "smelt", "slow", "damage", "stage",
    "generat", "spawn", "jungle", "20%", "chance", "tick", "growth",
    "light level", "drops", "harvest", "grid",
]

def wikitext(path):
    d = json.load(open(path))
    # find the wikitext content field
    data = d.get("data", {})
    for k, v in data.items():
        if isinstance(v, str) and len(v) > 5000:
            return v
    # fallback: search nested
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
    return hunt(d)

for path in FILES:
    txt = wikitext(path)
    print(f"\n{'='*70}\n=== {path} ({len(txt) if txt else 0} chars of wikitext)\n{'='*70}")
    if not txt:
        print("NO WIKITEXT FOUND — keys:", list(json.load(open(path)).get("data", {}).keys())[:20])
        continue
    lines = txt.split("\n")
    for i, line in enumerate(lines):
        low = line.lower()
        if any(k in low for k in KEYS) and len(line.strip()) > 3:
            print(f"{i:5d}| {line[:300]}")
