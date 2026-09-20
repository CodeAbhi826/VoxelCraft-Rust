#!/usr/bin/env python3
"""vault_study.py — READ-ONLY reference study (functional facts only).

Part of the clean-room chain (LEGAL-COMPLIANCE.md §1.4 step 1/2):
extracts DIMENSIONS, palette statistics and category structure from the
quarantined reference zip. Prints aggregate facts + writes a study note.
No pixels are stored anywhere. Reference never leaves upload/.
"""
import io, json, zipfile
from collections import Counter, defaultdict
from PIL import Image

ZIP = "/home/z/my-project/upload/textures.zip"

def walk():
    with zipfile.ZipFile(ZIP) as z:
        for info in z.infolist():
            if info.is_dir():
                continue
            yield info.filename, z.read(info.filename)

def facts():
    dims = Counter()
    pal_sizes = Counter()
    per_cat = defaultdict(lambda: Counter())
    modes = Counter()
    font_files, colormap_files, env_files = [], [], []
    for name, blob in walk():
        if not name.endswith(".png"):
            continue
        cat = name.split("/")[1]
        try:
            im = Image.open(io.BytesIO(blob)).convert("RGBA")
        except Exception:
            continue
        w, h = im.size
        dims[(w, h)] += 1
        per_cat[cat][f"{w}x{h}"] += 1
        colors = im.getcolors(maxcolors=1 << 24)
        pal_sizes[len(colors) if colors else 4096] += 1
        modes[(cat, "has_alpha")] if False else None
        if cat == "font":
            font_files.append((name, w, h, len(colors) if colors else -1))
        if cat == "colormap":
            colormap_files.append((name, w, h, len(colors) if colors else -1))
        if cat == "environment":
            env_files.append((name, w, h, len(colors) if colors else -1))
    return dims, pal_sizes, per_cat, font_files, colormap_files, env_files

if __name__ == "__main__":
    dims, pal, per_cat, fonts, cmaps, envs = facts()
    print("TOP DIMENSIONS:", dims.most_common(25))
    print("\nPALETTE SIZES (distinct colors -> count):")
    for k in sorted(pal):
        if pal[k] >= 5 or k < 40:
            print(f"  {k:5d} colors -> {pal[k]} textures")
    print("\nPER-CATEGORY top dims:")
    for cat in sorted(per_cat):
        print(f"  {cat:12s} {per_cat[cat].most_common(6)}  total={sum(per_cat[cat].values())}")
    print("\nFONT FILES:", fonts)
    print("\nCOLORMAP FILES:", cmaps)
    print("\nENVIRONMENT FILES:", envs)
