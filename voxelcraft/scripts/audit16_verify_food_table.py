#!/usr/bin/env python3
"""Re-verify the audit16 food values straight from the Food page capture.

Extracts the hunger/saturation table rows for every food the engine now
serves, printing the live-captured values next to the coded ones so a
human can eyeball the match (the tests assert the same numbers).
"""
import json
import re
import html as htmllib

RAW = json.load(open("scripts/audit16_page_Food.json"))["data"]["html"]

# strip tags -> text
text = re.sub(r"<style.*?</style>", " ", RAW, flags=re.S)
text = re.sub(r"<script.*?</script>", " ", text, flags=re.S)
text = re.sub(r"<[^>]+>", "|", text)
text = htmllib.unescape(text)
text = re.sub(r"\|+", "|", text)

WANT = [
    "Apple", "Baked Potato", "Beetroot", "Beetroot Soup", "Bread",
    "Carrot", "Chorus Fruit", "Cooked Chicken", "Cooked Cod",
    "Cooked Mutton", "Cooked Porkchop", "Cooked Salmon", "Cookie",
    "Dried Kelp", "Glow Berries", "Golden Apple", "Golden Carrot",
    "Honey Bottle", "Melon Slice", "Mushroom Stew", "Poisonous Potato",
    "Pumpkin Pie", "Rabbit Stew", "Raw Beef", "Raw Chicken", "Raw Cod",
    "Raw Mutton", "Raw Porkchop", "Raw Salmon", "Rotten Flesh",
    "Spider Eye", "Steak", "Sweet Berries", "Tropical Fish",
]

for food in WANT:
    # find the row: |Food|...|hunger|...|saturation|...
    i = text.find("|" + food + "|")
    if i < 0:
        print(f"{food:18s} NOT-FOUND-IN-CAPTURE")
        continue
    row = text[i:i + 220]
    # hunger is the first small number after the name cell; the table
    # columns are: Food | Restores (hunger) | Saturation | ...
    m = re.match(r"\|[^|]+\|([^|]+)\|([^|]+)\|", row)
    if m:
        print(f"{food:18s} hunger={m.group(1).strip():>6s}  sat={m.group(2).strip():>6s}")
    else:
        print(f"{food:18s} ROW:{row[:80]}")
