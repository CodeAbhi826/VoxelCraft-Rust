#!/usr/bin/env python3
"""measure_monocraft.py — measure Monocraft.ttf metrics for the engine
font swap (cap ratio, coverage of the glyphs the tests assert, advance
behavior). Read-only inspection of the OFL font we embed.
"""
from fontTools.ttLib import TTFont

P = "/home/z/my-project/voxelcraft/crates/vc-render/assets/Monocraft.ttf"
f = TTFont(P)
upm = f["head"].unitsPerEm
cmap = f.getBestCmap()
glyf = f["glyf"]
hmtx = f["hmtx"]

print(f"unitsPerEm: {upm}, glyphs in cmap: {len(cmap)}")

# cap height from 'H' bbox
h = glyf[cmap[ord("H")]]
cap = (h.yMax - h.yMin) / upm
print(f"'H' bbox yMax={h.yMax} yMin={h.yMin} -> cap_ratio = {cap:.4f}")

# x-height from 'x'
x = glyf[cmap[ord("x")]]
print(f"'x' height ratio = {(x.yMax - x.yMin) / upm:.4f}")

# digits '0' bbox for reference
d = glyf[cmap[ord("0")]]
print(f"'0' height ratio = {(d.yMax - d.yMin) / upm:.4f}")

# advances (is it monospace?)
adv = {c: hmtx[cmap[ord(c)]][0] for c in "HWi. "}
print("advances (units):", adv)
print(f"advance ratio W: {adv['W'] / upm:.4f}")

# coverage of the test-asserted chars
for ch in ["A", "a", "∞", "→", "←", "↑", "↓", "°", "·", "§", "Ä", "ö", "ß", "€",
           "¼", "×", "÷", "±", "≈", "≠", "≤", "≥", "♥", "☑", "⌂", "¶", "µ",
           "\u4E9C"]:
    print(f"  U+{ord(ch):04X} {ch!r}: {'YES' if ord(ch) in cmap else 'no'}")
