#!/usr/bin/env python3
"""vault_synth_sprites.py — sprite-family generators (items, plants,
particles, map marks). Original pixel-art constructions: same canvas,
same palette family (numeric facts), own silhouettes and detailing.

Template router: suffix/keyword -> procedural drawing at any power-of-2
scale. Unmatched names fall back to an organic-blob sprite sized by the
measured opaque-coverage fact.
"""
import numpy as np
from PIL import Image, ImageDraw
from vault_synth_core import (rng_for, canvas, fill, px, Pal, blob, disc,
                              line, sprite_from_fn)

def _pal_roles(pal):
    """Choose shading roles from the measured palette."""
    order = pal.order_by_luma()
    return dict(
        hi=pal.rgba(order[-1]),           # brightest — highlights
        mid=pal.rgba(order[len(order) // 2]),
        lo=pal.rgba(order[0]),            # darkest — outlines/shadow
        mid2=pal.rgba(order[max(0, len(order) // 2 - 1)]),
    )

def _outline(c, mask, color):
    m = mask.astype(bool)
    if not m.any():
        return
    dil = m.copy()
    dil[1:, :] |= m[:-1, :]
    dil[:-1, :] |= m[1:, :]
    dil[:, 1:] |= m[:, :-1]
    dil[:, :-1] |= m[:, 1:]
    ring = dil & ~m
    c[ring] = color

def _shaded(c, mask, roles, rng, axis="v"):
    """Fill a mask with 3-tone shading + own dithering."""
    m = mask.astype(bool)
    if not m.any():
        return
    H, W = c.shape[:2]
    yy, xx = np.mgrid[0:H, 0:W]
    if axis == "v":
        t = (xx - xx[m].min()) / max(1, xx[m].max() - xx[m].min())
    else:
        t = (yy - yy[m].min()) / max(1, yy[m].max() - yy[m].min())
    n = rng.random((H, W)) * 0.18
    t = np.clip(t + n, 0, 1)
    c[m & (t < 0.33)] = roles["lo"]
    c[m & (t >= 0.33) & (t < 0.72)] = roles["mid2"]
    c[m & (t >= 0.72)] = roles["hi"]

# ------------------------------------------------------------------ tools
def tpl_sword(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # blade: 3px diagonal band (bright edge / body / dark edge)
    x0, y0 = 4 * s, 12 * s
    x1, y1 = 13 * s, 3 * s
    for i in range((x1 - x0) + 1):
        t = i / max(1, (x1 - x0))
        xx = x0 + i
        yy = int(y0 + (y1 - y0) * t)
        d.point((xx, yy), fill=r["hi"])
        d.point((xx - 1, yy + 1), fill=r["mid2"])
        d.point((xx - 2, yy + 2), fill=r["mid"])
    # guard: cross bar perpendicular to the blade
    for k in range(-2, 3):
        d.point((x0 + k - 1, y0 + k), fill=r["lo"])
        d.point((x0 + k, y0 + k - 1), fill=r["lo"])
    d.point((x0, y0 - 1), fill=r["hi"])
    # grip (dark leather tones)
    for i in range(3 * s):
        gx, gy = x0 - 1 - i, y0 + 1 + i
        d.point((gx, gy), fill=r["lo"])
        d.point((gx + 1, gy), fill=r["mid"])
    # pommel
    d.point((x0 - 3, y0 + 4), fill=r["hi"])
    d.point((x0 - 4, y0 + 3), fill=r["mid2"])
    return np.asarray(im).copy()
def tpl_pickaxe(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # handle: diagonal 2px (body + dark edge)
    for i in range(10 * s):
        xx, yy = 3 * s + i, 12 * s - i
        d.point((xx, yy), fill=r["mid2"])
        d.point((xx, yy + 1), fill=r["mid"])
        d.point((xx + 1, yy + 1), fill=r["lo"])
    # head: two prongs sweeping down at both ends + center mount
    for k in range(4):
        d.point((9 * s - 1 - k, 3 * s + k), fill=r["hi"])
        d.point((9 * s - 2 - k, 3 * s + k), fill=r["mid2"])
        d.point((10 * s + k, 3 * s + k), fill=r["hi"])
        d.point((10 * s + k + 1, 3 * s + k), fill=r["mid2"])
    for x in range(9 * s - 1, 11 * s):
        d.point((x, 3 * s), fill=r["lo"])
        d.point((x, 4 * s), fill=r["mid"])
    return np.asarray(im).copy()
def tpl_axe(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    for i in range(10 * s):
        d.point((3 * s + i, 12 * s - i), fill=r["mid2"])
        d.point((3 * s + i, 12 * s - i + 1), fill=r["mid"])
        d.point((3 * s + i + 1, 12 * s - i + 1), fill=r["lo"])
    # blade: chunky wedge w/ bright cutting edge (upper-right)
    d.polygon([(9 * s, 4 * s), (13 * s, 2 * s), (15 * s, 5 * s),
               (13 * s, 8 * s), (10 * s, 8 * s)], fill=r["mid2"])
    d.line([(13 * s, 2 * s), (15 * s, 5 * s), (13 * s, 8 * s)], fill=r["hi"])
    d.line([(10 * s, 8 * s), (13 * s, 8 * s)], fill=r["lo"])
    d.line([(9 * s, 4 * s), (10 * s, 8 * s)], fill=r["lo"])
    return np.asarray(im).copy()
def tpl_shovel(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    for i in range(9 * s):
        d.point((5 * s + i, 11 * s - i), fill=r["mid2"])
        d.point((5 * s + i, 11 * s - i + 1), fill=r["mid"])
    # spade head: rounded wedge w/ bright face
    d.polygon([(11 * s, 6 * s), (12 * s, 2 * s), (15 * s, 3 * s),
               (15 * s, 5 * s), (13 * s, 7 * s)], fill=r["mid2"])
    d.line([(12 * s, 2 * s), (15 * s, 3 * s)], fill=r["hi"])
    d.line([(12 * s, 3 * s), (14 * s, 4 * s)], fill=r["hi"])
    d.line([(13 * s, 7 * s), (15 * s, 5 * s)], fill=r["lo"])
    return np.asarray(im).copy()
def tpl_hoe(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    for i in range(9 * s):
        d.point((5 * s + i, 11 * s - i), fill=r["mid2"])
        d.point((5 * s + i, 11 * s - i + 1), fill=r["mid"])
    # hoe head: horizontal bar + downward blade
    d.rectangle([10 * s, 3 * s, 14 * s, 4 * s], fill=r["mid2"])
    d.rectangle([10 * s, 3 * s, 14 * s, 3 * s], fill=r["hi"])
    d.rectangle([13 * s, 4 * s, 15 * s, 6 * s], fill=r["mid"])
    d.rectangle([13 * s, 4 * s, 15 * s, 4 * s], fill=r["hi"])
    return np.asarray(im).copy()
# ------------------------------------------------------------------ armor
def tpl_helmet(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # dome
    d.polygon([(4 * s, 7 * s), (5 * s, 3 * s), (11 * s, 3 * s),
               (12 * s, 7 * s), (12 * s, 10 * s), (4 * s, 10 * s)],
              fill=r["mid2"])
    # dome highlight band
    d.line([(5 * s, 4 * s), (11 * s, 4 * s)], fill=r["hi"])
    # face opening (dark)
    d.rectangle([6 * s, 7 * s, 10 * s, 9 * s], fill=r["lo"])
    # side cheek guards
    d.rectangle([4 * s, 7 * s, 5 * s, 10 * s], fill=r["mid"])
    d.rectangle([11 * s, 7 * s, 12 * s, 10 * s], fill=r["mid"])
    # nose guard
    d.rectangle([7 * s, 7 * s, 8 * s, 9 * s], fill=r["mid2"])
    d.line([(4 * s, 10 * s), (12 * s, 10 * s)], fill=r["lo"])
    return np.asarray(im).copy()
def tpl_chest(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # torso with shoulders
    d.polygon([(3 * s, 5 * s), (5 * s, 3 * s), (11 * s, 3 * s),
               (13 * s, 5 * s), (13 * s, 13 * s), (3 * s, 13 * s)],
              fill=r["mid2"])
    # shoulder caps
    d.rectangle([3 * s, 4 * s, 6 * s, 6 * s], fill=r["mid"])
    d.rectangle([10 * s, 4 * s, 13 * s, 6 * s], fill=r["mid"])
    d.line([(4 * s, 4 * s), (6 * s, 4 * s)], fill=r["hi"])
    d.line([(10 * s, 4 * s), (12 * s, 4 * s)], fill=r["hi"])
    # central straps + belt
    d.rectangle([7 * s, 3 * s, 8 * s, 13 * s], fill=r["lo"])
    d.rectangle([3 * s, 11 * s, 13 * s, 12 * s], fill=r["lo"])
    d.rectangle([3 * s, 11 * s, 13 * s, 11 * s], fill=r["mid"])
    return np.asarray(im).copy()
def tpl_leggings(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # waistband
    d.rectangle([3 * s, 3 * s, 13 * s, 5 * s], fill=r["mid2"])
    d.rectangle([3 * s, 3 * s, 13 * s, 3 * s], fill=r["hi"])
    # two legs separated by a 1px gap
    d.rectangle([4 * s, 6 * s, 7 * s, 13 * s], fill=r["mid"])
    d.rectangle([9 * s, 6 * s, 12 * s, 13 * s], fill=r["mid"])
    d.rectangle([4 * s, 6 * s, 7 * s, 6 * s], fill=r["hi"])
    d.rectangle([9 * s, 6 * s, 12 * s, 6 * s], fill=r["hi"])
    d.line([(4 * s, 13 * s), (7 * s, 13 * s)], fill=r["lo"])
    d.line([(9 * s, 13 * s), (12 * s, 13 * s)], fill=r["lo"])
    return np.asarray(im).copy()
def tpl_boots(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # two boot silhouettes: shaft + foot (L-shape)
    d.rectangle([3 * s, 6 * s, 6 * s, 11 * s], fill=r["mid2"])
    d.rectangle([2 * s, 11 * s, 7 * s, 13 * s], fill=r["mid"])
    d.rectangle([10 * s, 6 * s, 13 * s, 11 * s], fill=r["mid2"])
    d.rectangle([9 * s, 11 * s, 14 * s, 13 * s], fill=r["mid"])
    d.rectangle([3 * s, 6 * s, 6 * s, 6 * s], fill=r["hi"])
    d.rectangle([10 * s, 6 * s, 13 * s, 6 * s], fill=r["hi"])
    d.rectangle([2 * s, 13 * s, 7 * s, 13 * s], fill=r["lo"])
    d.rectangle([9 * s, 13 * s, 14 * s, 13 * s], fill=r["lo"])
    return np.asarray(im).copy()
# ------------------------------------------------------------------ materials
def tpl_ingot(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.polygon([(4 * s, 9 * s), (6 * s, 7 * s), (12 * s, 7 * s),
               (14 * s, 9 * s), (12 * s, 12 * s), (6 * s, 12 * s)],
              fill=r["mid2"])
    d.polygon([(6 * s, 7 * s), (12 * s, 7 * s), (13 * s, 9 * s),
               (7 * s, 9 * s)], fill=r["hi"])
    d.line([(7 * s, 10 * s), (12 * s, 10 * s)], fill=r["mid"])
    return np.asarray(im).copy()

def tpl_gem(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.polygon([(8 * s, 3 * s), (13 * s, 8 * s), (8 * s, 13 * s),
               (3 * s, 8 * s)], fill=r["mid2"])
    d.polygon([(8 * s, 3 * s), (13 * s, 8 * s), (8 * s, 8 * s)],
              fill=r["hi"])
    d.polygon([(8 * s, 13 * s), (3 * s, 8 * s), (8 * s, 8 * s)],
              fill=r["mid"])
    return np.asarray(im).copy()

def tpl_shard(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.polygon([(3 * s, 11 * s), (6 * s, 4 * s), (10 * s, 6 * s),
               (12 * s, 12 * s), (8 * s, 13 * s)], fill=r["mid2"])
    d.line([(6 * s, 4 * s), (8 * s, 12 * s)], fill=r["hi"])
    return np.asarray(im).copy()

def tpl_dust(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    n = max(8, (w * h) // 6)
    for _ in range(n):
        x = int(rng.integers(2, w - 2))
        y = int(rng.integers(4, h - 2))
        cy = 1.0 - (y - 4) / max(1, h - 6)
        rad = max(1, int(s_rad(rng, w) * (0.5 + cy)))
        disc(c, x, y, rad, r["hi"] if rng.random() < 0.25 else
             (r["mid2"] if rng.random() < 0.5 else r["mid"]))
    return c

def s_rad(rng, w):
    return max(0.8, w / 16 * (0.8 + rng.random() * 0.8))

def tpl_ball(w, h, pal, rng, highlight=True):
    c = canvas(w, h)
    r = _pal_roles(pal)
    cx, cy, rad = (w - 1) / 2, (h - 1) / 2, min(w, h) / 2 - 1
    m = disc(c, cx, cy, rad, r["mid2"])
    if highlight:
        yy, xx = np.mgrid[0:h, 0:w]
        hm = ((xx - (cx - rad * 0.28)) ** 2 + (yy - (cy - rad * 0.32)) ** 2) <= (rad * 0.42) ** 2
        c[hm] = r["hi"]
    # shade lower right
    yy, xx = np.mgrid[0:h, 0:w]
    sm = ((xx - cx) ** 2 + (yy - cy) ** 2 <= rad * rad) & ((xx - cx) + (yy - cy) > rad * 0.9)
    c[sm] = r["mid"]
    return c

def tpl_coal(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    for k in range(3):
        blob(c, w * (0.35 + 0.18 * k), h * (0.62 - 0.12 * k),
             w * 0.22, h * 0.20, rng, r["mid2"], wob=0.4)
    yy, xx = np.mgrid[0:h, 0:w]
    hm = (xx < w * 0.45) & (yy < h * 0.5)
    c[hm & (c[..., 3] > 0)] = r["mid"]
    px(c, int(w * 0.4), int(h * 0.35), r["hi"])
    return c

def tpl_raw_chunk(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    blob(c, w * 0.5, h * 0.6, w * 0.3, h * 0.26, rng, r["mid2"])
    blob(c, w * 0.35, h * 0.45, w * 0.18, h * 0.16, rng, r["mid"])
    blob(c, w * 0.65, h * 0.5, w * 0.15, h * 0.14, rng, r["hi"])
    return c

def tpl_pearl(w, h, pal, rng):
    return tpl_ball(w, h, pal, rng, highlight=True)

def tpl_egg(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    yy, xx = np.mgrid[0:h, 0:w]
    cx = (w - 1) / 2
    m = ((xx - cx) / (w * 0.30)) ** 2 + ((yy - h * 0.55) / (h * 0.42)) ** 2 <= 1
    c[m] = r["mid2"]
    c[m & (yy < h * 0.4)] = r["mid"]
    c[m & (xx < w * 0.4) & (yy < h * 0.5)] = r["hi"]
    for _ in range(6):
        x = int(rng.integers(4, w - 3))
        y = int(rng.integers(5, h - 3))
        px(c, x, y, r["lo"])
    return c

TOOL_ROUTES = [
    ("_sword", tpl_sword), ("sword", tpl_sword),
    ("_pickaxe", tpl_pickaxe), ("pickaxe", tpl_pickaxe),
    ("_axe", tpl_axe), ("_hatchet", tpl_axe),
    ("_shovel", tpl_shovel), ("_spade", tpl_shovel),
    ("_hoe", tpl_hoe),
    ("_helmet", tpl_helmet), ("helmet", tpl_helmet),
    ("_chestplate", tpl_chest), ("chestplate", tpl_chest),
    ("_leggings", tpl_leggings), ("leggings", tpl_leggings),
    ("_boots", tpl_boots),
    ("_ingot", tpl_ingot), ("nugget", tpl_ingot),
    ("_gem", tpl_gem), ("emerald", tpl_gem), ("diamond", tpl_gem),
    ("lapis_lazuli", tpl_gem), ("quartz", tpl_gem),
    ("_shard", tpl_shard), ("_crystal", tpl_shard),
    ("_dust", tpl_dust), ("_powder", tpl_dust),
    ("_ball", tpl_ball), ("snowball", tpl_ball),
    ("_pearl", tpl_pearl), ("_eye", tpl_ball),
    ("coal", tpl_coal), ("charcoal", tpl_coal),
    ("raw_", tpl_raw_chunk), ("_egg", tpl_egg),
]
