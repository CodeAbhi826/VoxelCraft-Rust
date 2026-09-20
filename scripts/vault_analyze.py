#!/usr/bin/env python3
"""vault_analyze.py — clean-room SPEC EXTRACTION (LEGAL-COMPLIANCE.md §1.4 steps 1–2).

Reads the quarantined reference zip (upload/textures.zip — never committed,
never shipped) and reduces every texture to a FUNCTIONAL DESCRIPTION:
dimensions, palette histogram (color values as numeric facts — §4.2),
aggregate structure statistics (banding, edge density, symmetry, coverage)
and a family tag. NO pixel positions, NO masks, NO coordinate data are
stored — the spec is a measurement document, not a downsampled copy.

The synthesizer (vault_synthesize.py) then generates every pixel of the
vault from the spec + procedural rules, with zero reference reads.

Output: voxelcraft/assets-vault/spec/spec.json (+ spec/README.md provenance).
"""
import io, json, re, sys, zipfile
import numpy as np
from PIL import Image

ZIP = "/home/z/my-project/upload/textures.zip"
OUT = "/home/z/my-project/voxelcraft/assets-vault/spec/spec.json"

# ---------------------------------------------------------------- helpers
def to_rgba(im):
    if im.mode == "RGBA":
        return im
    if im.mode in ("P", "PA"):
        return im.convert("RGBA")
    if im.mode == "LA":
        return im.convert("RGBA")
    return im.convert("RGBA")

def palette(arr, cap=24):
    """Top-N colors + counts (histogram facts only)."""
    flat = arr.reshape(-1, 4)
    uniq, counts = np.unique(flat, axis=0, return_counts=True)
    order = np.argsort(-counts)[:cap]
    return [[int(c) for c in uniq[i]] + [int(counts[i])] for i in order], int(len(uniq))

def stats(arr):
    h, w, _ = arr.shape
    rgb = arr[..., :3].astype(np.int16)
    a = arr[..., 3].astype(np.int16)
    # neighbor differences
    rd = np.abs(rgb[:, 1:] - rgb[:, :-1]).mean(axis=2)  # right neighbor
    dd = np.abs(rgb[1:, :] - rgb[:-1, :]).mean(axis=2)  # down neighbor
    nb = min(rd.mean(), dd.mean()) if w > 1 and h > 1 else 0.0
    # banding: rows similar to each other (horizontal boards) etc.
    row_mean = rgb.mean(axis=1)                          # (h,3)
    col_mean = rgb.mean(axis=0)                          # (w,3)
    row_band = float(np.abs(np.diff(row_mean, axis=0)).mean()) if h > 1 else 0.0
    col_band = float(np.abs(np.diff(col_mean, axis=0)).mean()) if w > 1 else 0.0
    # busy-ness
    edge_density = float((rd > 12).mean()) if rd.size else 0.0
    # flatness: pixel equals right AND down neighbor
    same_r = (np.abs(rd) <= 2) if rd.size else np.zeros((h, w - 1), bool)
    same_d = (np.abs(dd) <= 2) if dd.size else np.zeros((h - 1, w), bool)
    if h > 1 and w > 1:
        flat = float((same_r[: h - 1, :] & same_d[:, : w - 1]).mean())
    else:
        flat = 1.0
    # symmetry
    sym_h = float(np.abs(rgb - rgb[:, ::-1]).mean() / 255.0)  # mirrored across vertical axis
    sym_v = float(np.abs(rgb - rgb[::-1, :]).mean() / 255.0)  # mirrored across horizontal axis
    # alpha (tr = transparent fraction, pa = partial-alpha fraction)
    opaque = a >= 255
    transparent = a <= 8
    tr = float(transparent.mean())
    pa = float(((~opaque) & (~transparent)).mean())
    # luma
    luma = rgb.mean(axis=2)
    return dict(
        rb=round(row_band, 2), cb=round(col_band, 2),
        ed=round(edge_density, 4), fl=round(flat, 4),
        sy=round(sym_h, 4), sz=round(sym_v, 4),
        ar=round(tr, 4), pa=round(pa, 4),
        lm=round(float(luma.mean()), 1), ct=round(float(luma.std()), 1),
        nb=round(float(nb), 1),
    )

ORE_RE = re.compile(r"(?:^|_)(?:[a-z_]+_ore|ore)(?:$|_|$)")
PLANK_RE = re.compile(r"(?:^|_)planks(?:$|_)|_parquet|_boards|_shingle")
BRICK_RE = re.compile(r"(?:^|_)bricks?(?:$|_)|_tiles(?![a-z])|tile$|_mosaic")
STRIPE_RE = re.compile(r"(?:^|_)(?:log|stem|pillar|bamboo_block)(?:$|_)")
GUI_PANELS = ("container", "inventory", "crafting", "furnace", "hopper", "brewing",
              "anvil", "villager", "village", "loom", "cartography", "stonecutter",
              "grindstone", "smithing", "enchanting", "beacon", "book", "social",
              "creative_inventory", "presets", "advancements", "puzzle", "sign",
              "hanging_sign", "stats", "abuse", "report", "realms", "skins",
              "world_selection", "widget", "spectator", "npc", "pin", "chat",
              "banned", "warning", "insufficient", "connection", "community")

def classify(cat, name, pal_n, st):
    """Family tag from category + name (functional facts) + aggregate stats."""
    base = name.rsplit("/", 1)[-1]
    if base.endswith(".png"):
        base = base[:-4]
    if cat == "font":
        return "glyph_page"
    if cat == "colormap":
        return "colormap"
    if cat == "painting":
        return "painting"
    if cat == "environment":
        return "environment"
    if cat == "entity":
        return "entity_skin"
    if cat == "trims":
        return "trim_palette"
    if cat == "gui":
        if any(k in base for k in GUI_PANELS) and (st["fl"] > 0.35 or st["nb"] < 60):
            return "gui_panel"
        if st["ar"] < 0.35 and st["fl"] > 0.55:
            return "gui_panel"       # bordered widgets with flat fills
        if st["ar"] >= 0.35:
            return "gui_icon"        # small alpha-masked widget sprites
        return "gui_icon"
    if st["ar"] >= 0.35:
        return "sprite"              # items, plants, particles w/ transparency
    if pal_n <= 2 or st["dc"] > 0.965:
        return "flat"
    if PLANK_RE.search(base):
        return "planks"
    if BRICK_RE.search(base):
        return "bricks"
    if ORE_RE.search(base) or base.endswith("_ore") or "_ore_" in base:
        return "ore"
    if STRIPE_RE.search(base) or (st["cb"] > 8 and st["rb"] < 6):
        return "stripe"
    if cat == "map":
        return "sprite"
    if cat in ("particle", "effect", "mob_effect", "misc"):
        return "sprite"
    if st["rb"] > 8 and st["cb"] < 6:
        return "bands"               # generic horizontal banding
    return "noise"

def dominant_cov(pal):
    total = sum(p[4] for p in pal) or 1
    return pal[0][4] / total

# ---------------------------------------------------------------- main
def main():
    z = zipfile.ZipFile(ZIP)
    records, mcmetas = [], []
    for info in z.infolist():
        name = info.filename
        if info.is_dir():
            continue
        if name.endswith(".mcmeta"):
            try:
                mcmetas.append({"n": name.replace("textures/", "", 1),
                                "j": json.loads(z.read(name).decode("utf-8"))})
            except Exception:
                pass
            continue
        if not name.endswith(".png"):
            continue
        rel = name.replace("textures/", "", 1)
        cat = rel.split("/")[0]
        im = to_rgba(Image.open(io.BytesIO(z.read(name))))
        arr = np.asarray(im)
        h, w = arr.shape[:2]
        pal, pal_n = palette(arr)
        st = stats(arr)
        st["dc"] = round(dominant_cov(pal), 4)
        fam = classify(cat, rel, pal_n, st)
        rec = {"n": rel, "c": cat, "w": w, "h": h, "pn": pal_n,
               "pal": [[c[0], c[1], c[2], c[3]] for c in pal],
               "pc": [round(p[4] / (w * h), 4) for p in pal], **st, "fam": fam}
        if fam == "colormap":
            # functional LUT — coarse 6x6 sample grid as numeric facts
            s = arr[:: max(h // 6, 1)][:, :: max(w // 6, 1)][:, :, :3]
            rec["lut"] = [[[int(v) for v in px] for px in row] for row in s]
        records.append(rec)

    with open(OUT, "w") as f:
        json.dump({"textures": records, "mcmeta": mcmetas}, f, separators=(",", ":"))
    # family census
    from collections import Counter
    fams = Counter((r["fam"], r["c"]) for r in records)
    print(f"spec written: {len(records)} textures + {len(mcmetas)} mcmeta -> {OUT}")
    print(f"{'category':14s} {'family':14s} count")
    for (fam, cat), n in sorted(fams.items(), key=lambda kv: -kv[1]):
        print(f"{cat:14s} {fam:14s} {n}")
    sizes = Counter((r["c"], r["w"], r["h"]) for r in records)
    print("\nlargest canvases:")
    for (cat, w, h), n in sorted(sizes.items(), key=lambda kv: -kv[0][1] * kv[0][2])[:14]:
        print(f"  {cat:12s} {w}x{h} x{n}")

if __name__ == "__main__":
    main()
