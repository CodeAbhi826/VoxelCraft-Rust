#!/usr/bin/env python3
"""vault_synth_sprites2.py — food / plant / utility sprite templates and
the master sprite router (falls back to a coverage-targeted organic blob)."""
import numpy as np
from PIL import Image, ImageDraw
from vault_synth_core import (rng_for, canvas, fill, px, Pal, blob, disc,
                              line, sprite_from_fn)
from vault_synth_sprites import (_pal_roles, TOOL_ROUTES, tpl_ball,
                                  tpl_ingot, tpl_dust)

# ------------------------------------------------------------------ food
def tpl_apple(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    blob(c, w / 2, h * 0.58, w * 0.32, h * 0.30, rng, r["mid2"], wob=0.15)
    m = c[..., 3] > 0
    yy, xx = np.mgrid[0:h, 0:w]
    c[m & (xx > w * 0.55) & (yy > h * 0.5)] = r["mid"]
    c[m & (xx < w * 0.42) & (yy < h * 0.5)] = r["hi"]
    # stem + leaf
    for i in range(3):
        px(c, int(w / 2 + i * 0.4), int(h * 0.28 - i), r["lo"])
    c[int(h*0.22):int(h*0.30), int(w*0.55):int(w*0.75)] = r["hi"]
    return c

def tpl_bread(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.polygon([(3 * s, 9 * s), (5 * s, 6 * s), (11 * s, 6 * s),
               (13 * s, 9 * s), (11 * s, 11 * s), (5 * s, 11 * s)],
              fill=r["mid2"])
    for k in range(3):
        x = (5 + k * 2.6) * s
        d.line([(int(x), 7 * s), (int(x + s), 8 * s)], fill=r["hi"])
        d.line([(int(x + s), 7 * s), (int(x), 8 * s)], fill=r["mid"])
    return np.asarray(im).copy()

def tpl_cookie(w, h, pal, rng):
    c = tpl_ball(w, h, pal, rng, highlight=False)
    r = _pal_roles(pal)
    for _ in range(8):
        x = int(rng.integers(3, w - 3))
        y = int(rng.integers(3, h - 3))
        disc(c, x, y, max(0.7, w / 32), r["lo"])
    return c

def tpl_bowl(w, h, pal, rng, stew=True):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # bowl shell
    d.polygon([(3 * s, 8 * s), (13 * s, 8 * s), (11 * s, 13 * s),
               (5 * s, 13 * s)], fill=r["mid2"])
    d.line([(3 * s, 8 * s), (13 * s, 8 * s)], fill=r["hi"])
    if stew:
        d.rectangle([4 * s, 8 * s, 12 * s, 9 * s], fill=r["lo"])
    return np.asarray(im).copy()

def tpl_meat(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    blob(c, w * 0.55, h * 0.5, w * 0.30, h * 0.26, rng, r["mid2"], wob=0.2)
    # bone toward bottom-left
    for i in range(4):
        px(c, int(w * 0.22 + i), int(h * 0.66 + i * 0.5), r["hi"])
    disc(c, w * 0.22, h * 0.66, max(1, w * 0.08), r["hi"])
    yy, xx = np.mgrid[0:h, 0:w]
    c[(c[..., 3] > 0) & (yy > h * 0.62)] = r["mid"]
    return c

def tpl_fish(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.ellipse([3 * s, 5 * s, 11 * s, 11 * s], fill=r["mid2"])
    d.polygon([(11 * s, 6 * s), (14 * s, 4 * s), (14 * s, 12 * s),
               (11 * s, 10 * s)], fill=r["mid"])
    d.point((5 * s, 7 * s), fill=r["lo"])
    d.line([(5 * s, 9 * s), (9 * s, 9 * s)], fill=r["mid"])
    return np.asarray(im).copy()

def tpl_carrot(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.polygon([(6 * s, 13 * s), (8 * s, 6 * s), (10 * s, 5 * s),
               (9 * s, 13 * s)], fill=r["mid2"])
    d.polygon([(8 * s, 6 * s), (10 * s, 5 * s), (10 * s, 8 * s)],
              fill=r["hi"])
    for k in range(2):
        d.line([(6 * s, 9 * s + k), (8 * s, 9 * s + k)], fill=r["mid"])
    # greens
    d.line([(8 * s, 6 * s), (7 * s, 2 * s)], fill=(90, 160, 60, 255))
    d.line([(9 * s, 5 * s), (9 * s, 1 * s)], fill=(70, 140, 50, 255))
    d.line([(9 * s, 6 * s), (11 * s, 3 * s)], fill=(90, 160, 60, 255))
    return np.asarray(im).copy()

def tpl_potato(w, h, pal, rng):
    c = tpl_ball(w, h, pal, rng, highlight=False)
    r = _pal_roles(pal)
    for _ in range(4):
        x = int(rng.integers(3, w - 3))
        y = int(rng.integers(3, h - 3))
        disc(c, x, y, max(0.6, w / 40), r["hi"])
    return c

def tpl_melon_slice(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.pieslice([1 * s, 1 * s, 15 * s, 15 * s], 200, 340, fill=r["mid2"])
    d.arc([1 * s, 1 * s, 15 * s, 15 * s], 200, 340, fill=r["hi"])
    for k in range(3):
        ang = 220 + k * 28
        import math
        x = 8 * s + 5 * s * math.cos(math.radians(ang))
        y = 8 * s + 5 * s * math.sin(math.radians(ang))
        d.point((int(x), int(y)), fill=r["lo"])
    return np.asarray(im).copy()

def tpl_berries(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    for _ in range(3):
        blob(c, rng.uniform(w * 0.3, w * 0.7), rng.uniform(h * 0.35, h * 0.7),
             w * 0.16, h * 0.16, rng, r["mid2"], wob=0.2)
        px(c, int(w * 0.45), int(h * 0.4), r["hi"])
    return c

# ------------------------------------------------------------------ plants
def tpl_flower(w, h, pal, rng, head="disc"):
    r = _pal_roles(pal)
    c = canvas(w, h)
    # stem
    for y in range(int(h * 0.45), int(h * 0.95)):
        px(c, int(w / 2 + (1 if (y // 3) % 2 else 0)), y, r["lo"])
    # leaves
    blob(c, w * 0.36, h * 0.72, w * 0.14, h * 0.06, rng, r["mid"], wob=0.3)
    blob(c, w * 0.64, h * 0.8, w * 0.14, h * 0.06, rng, r["mid"], wob=0.3)
    # head
    cx, cy = w / 2, h * 0.3
    if head == "disc":
        disc(c, cx, cy, w * 0.24, r["mid2"])
        disc(c, cx, cy, w * 0.10, r["hi"])
    elif head == "cup":
        c[int(cy - h*0.08):int(cy + h*0.08), int(cx - w*0.16):int(cx + w*0.16)] = r["mid2"]
        c[int(cy - h*0.02):int(cy + h*0.08), int(cx - w*0.16):int(cx + w*0.16)] = r["hi"]
    elif head == "plume":
        for k in range(4):
            blob(c, cx + (k - 1.5) * w * 0.08, cy + abs(k - 1.5) * h * 0.05,
                 w * 0.09, h * 0.1, rng, r["mid2"], wob=0.25)
    return c

def tpl_sapling(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    for y in range(int(h * 0.55), int(h * 0.95)):
        px(c, int(w / 2), y, r["lo"])
    blob(c, w / 2, h * 0.38, w * 0.3, h * 0.26, rng, r["mid2"], wob=0.35)
    yy, xx = np.mgrid[0:h, 0:w]
    c[(c[..., 3] > 0) & (yy < h * 0.35) & (xx < w * 0.5)] = r["mid"]
    c[(c[..., 3] > 0) & (yy > h * 0.5)] = r["mid"]
    return c

def tpl_grass_tuft(w, h, pal, rng):
    c = canvas(w, h)
    r = _pal_roles(pal)
    n = max(4, w // 3)
    for k in range(n):
        x0 = int(rng.integers(2, w - 2))
        x1 = x0 + int(rng.integers(-2, 3))
        line(c, x0, h - 1, x1, int(h * rng.uniform(0.2, 0.6)),
             r["mid2"] if k % 3 else r["mid"])
    return c

def tpl_torch(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    for y in range(int(h * 0.4), int(h * 0.98)):
        px(c, int(w / 2), y, r["lo"])
        px(c, int(w / 2) + 1, y, r["mid"])
    disc(c, w / 2, h * 0.3, w * 0.16, r["hi"])
    disc(c, w / 2, h * 0.36, w * 0.12, r["mid2"])
    return c

def tpl_rail(w, h, pal, rng, powered=False):
    r = _pal_roles(pal)
    c = canvas(w, h)
    # two horizontal rails (functional geometry)
    for y in (int(h * 0.25), int(h * 0.75)):
        for x in range(w):
            px(c, x, y, r["mid2"] if (x // 2) % 3 else r["mid"])
    # ties
    for x in range(1, w, max(2, w // 4)):
        for yy in range(int(h * 0.2), int(h * 0.85)):
            px(c, x, yy, r["lo"])
    if powered:
        disc(c, w / 2, h / 2, w * 0.15, r["hi"])
    return c

def tpl_door(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    fill(c, r["mid2"], 1, 1, w - 2, h - 2)
    fill(c, r["lo"], 0, 0, w, 1)
    fill(c, r["lo"], 0, 0, 1, h)
    fill(c, r["lo"], w - 1, 0, 1, h)
    # recessed panel + window
    fill(c, r["mid"], 3, 2, w - 6, h // 2 - 2)
    wx, wy, ww, wh = w // 4, h // 4, w // 2, h // 4
    fill(c, (255, 255, 255, 200), wx, wy, ww, wh)
    fill(c, r["hi"], wx - 1, wy - 1, ww + 2, 1)
    px(c, w - 3, h // 2, r["hi"])
    return c

# ------------------------------------------------------------------ utility
def tpl_bottle(w, h, pal, rng, liquid=None):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # flask body
    d.rounded_rectangle([4 * s, 7 * s, 12 * s, 14 * s], radius=2 * s,
                        outline=r["hi"], width=s)
    d.rectangle([7 * s, 3 * s, 9 * s, 7 * s], outline=r["hi"], width=s)
    # cork
    d.rectangle([6 * s, 1 * s, 10 * s, 3 * s], fill=(150, 110, 60, 255))
    # liquid
    d.rectangle([5 * s, 9 * s, 11 * s, 13 * s], fill=liquid or r["mid2"])
    return np.asarray(im).copy()

def tpl_bucket(w, h, pal, rng, content=None):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    # filled body with rim
    d.polygon([(4 * s, 6 * s), (12 * s, 6 * s), (11 * s, 13 * s),
               (5 * s, 13 * s)], fill=r["mid2"])
    d.line([(4 * s, 6 * s), (12 * s, 6 * s)], fill=r["hi"])
    d.line([(5 * s, 13 * s), (11 * s, 13 * s)], fill=r["lo"])
    d.line([(5 * s, 13 * s - 1), (11 * s, 13 * s - 1)], fill=r["lo"] if content is None else r["mid"])
    # handle arc
    d.arc([5 * s, 1 * s, 11 * s, 8 * s], 180, 360, fill=r["hi"], width=max(1, s))
    if content is not None:
        d.rectangle([5 * s, 6 * s, 11 * s, 8 * s], fill=content)
        d.rectangle([5 * s, 6 * s, 11 * s, 6 * s], fill=r["hi"] if False else r["mid"])
    return np.asarray(im).copy()

def tpl_rod(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    for i in range(9 * s):
        d.point((5 * s + i, 11 * s - i), fill=r["mid2"])
        d.point((5 * s + i, 11 * s - i + 1), fill=r["mid"])
    d.point((14 * s, 2 * s), fill=r["hi"])
    return np.asarray(im).copy()

def tpl_book(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.rectangle([4 * s, 3 * s, 12 * s, 13 * s], fill=r["mid2"])
    d.rectangle([4 * s, 3 * s, 5 * s, 13 * s], fill=r["lo"])
    d.rectangle([6 * s, 5 * s, 11 * s, 11 * s], fill=r["hi"])
    d.line([(6 * s, 5 * s), (6 * s, 11 * s)], fill=r["mid"])
    return np.asarray(im).copy()

def tpl_paper(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    d.rectangle([3 * s, 4 * s, 12 * s, 12 * s], fill=r["hi"])
    d.line([(3 * s, 4 * s), (12 * s, 4 * s)], fill=r["mid"])
    d.line([(3 * s, 12 * s), (12 * s, 12 * s)], fill=r["mid"])
    d.line([(5 * s, 7 * s), (10 * s, 7 * s)], fill=r["mid2"])
    d.line([(5 * s, 9 * s), (10 * s, 9 * s)], fill=r["mid2"])
    return np.asarray(im).copy()

def tpl_string(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    pts = [(3 * s, 4 * s), (9 * s, 2 * s), (13 * s, 6 * s), (7 * s, 9 * s),
           (10 * s, 13 * s), (3 * s, 11 * s)]
    col = r["mid2"]
    if 0.299 * col[0] + 0.587 * col[1] + 0.114 * col[2] < 80:
        col = r["hi"]
    d.line(pts, fill=col, width=s)
    return np.asarray(im).copy()

def tpl_banner_sheet(w, h, pal, rng):
    r = _pal_roles(pal)
    c = canvas(w, h)
    fill(c, r["mid2"], 1, 1, w - 2, int(h * 0.85))
    fill(c, r["lo"], 0, 0, 1, int(h * 0.85))
    fill(c, r["lo"], w - 1, 0, 1, int(h * 0.85))
    fill(c, r["lo"], 0, int(h * 0.85) - 1, w, 1)
    # swallow-tail bottom cut (our own banner cut)
    for x in range(w // 2):
        y = int(h * 0.85) + x
        px(c, x + w // 4, y, (0, 0, 0, 0))
    fill(c, r["hi"], 2, 2, w - 4, 2)
    return c

def tpl_arrow(w, h, pal, rng):
    r = _pal_roles(pal)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    s = max(1, w // 16)
    for i in range(8 * s):
        d.point((4 * s + i, 11 * s - i), fill=r["mid2"])
    d.polygon([(11 * s, 4 * s), (14 * s, 1 * s), (13 * s, 5 * s)],
              fill=r["hi"])
    d.polygon([(3 * s, 10 * s), (2 * s, 13 * s), (5 * s, 11 * s)],
              fill=r["lo"])
    return np.asarray(im).copy()

# ------------------------------------------------------------------ generic
def tpl_blob_generic(rec, name, pal):
    """Coverage-targeted organic sprite (coverage = measured opaque fraction)."""
    w, h = rec["w"], rec["h"]
    rng = rng_for(name, "sprite")
    r = _pal_roles(pal)
    c = canvas(w, h)
    cov = max(0.12, min(0.95, 1.0 - rec.get("ar", 0.5)))
    # choose blob radii to approximate coverage
    rx = w * 0.5 * (cov ** 0.5) * rng.uniform(0.85, 1.15)
    ry = h * 0.5 * (cov ** 0.5) * rng.uniform(0.85, 1.15)
    m = blob(c, w / 2, h / 2, rx, ry, rng, r["mid2"], wob=0.3)
    yy, xx = np.mgrid[0:h, 0:w]
    c[m & (yy > h * 0.6)] = r["mid"]
    c[m & (xx < w * 0.35) & (yy < h * 0.4)] = r["hi"]
    return c

def tpl_particle(rec, name, pal):
    """Soft radial particle with alpha falloff."""
    w, h = rec["w"], rec["h"]
    rng = rng_for(name, "part")
    r = _pal_roles(pal)
    c = canvas(w, h)
    yy, xx = np.mgrid[0:h, 0:w]
    d = np.sqrt((xx - (w - 1) / 2) ** 2 + (yy - (h - 1) / 2) ** 2)
    rad = min(w, h) / 2
    fall = np.clip(1 - d / rad, 0, 1)
    a = (fall ** 1.6) * 255
    base = np.array(r["mid2"][:3], np.float32)          # (3,)
    shade = (0.7 + 0.5 * fall)[..., None]               # (h, w, 1)
    rgb = np.clip(base[None, None, :] * shade, 0, 255).astype(np.uint8)
    c[..., :3] = rgb
    c[..., 3] = a.astype(np.uint8)
    return c

# ------------------------------------------------------------------ router
FOOD_ROUTES = [
    ("apple", tpl_apple), ("berries", tpl_berries), ("bread", tpl_bread),
    ("cookie", tpl_cookie), ("_stew", lambda w, h, p, r: tpl_bowl(w, h, p, r, True)),
    ("_soup", lambda w, h, p, r: tpl_bowl(w, h, p, r, True)),
    ("bowl", lambda w, h, p, r: tpl_bowl(w, h, p, r, False)),
    ("melon_slice", tpl_melon_slice), ("carrot", tpl_carrot),
    ("potato", tpl_potato), ("_chop", tpl_meat), ("beef", tpl_meat),
    ("mutton", tpl_meat), ("_fish", tpl_fish), ("cod", tpl_fish),
    ("salmon", tpl_fish),
]
PLANT_ROUTES = [
    ("_sapling", tpl_sapling), ("_tulip", lambda w, h, p, r: tpl_flower(w, h, p, r, "cup")),
    ("dandelion", lambda w, h, p, r: tpl_flower(w, h, p, r, "disc")),
    ("_orchid", lambda w, h, p, r: tpl_flower(w, h, p, r, "disc")),
    ("_poppy", lambda w, h, p, r: tpl_flower(w, h, p, r, "disc")),
    ("_daisy", lambda w, h, p, r: tpl_flower(w, h, p, r, "plume")),
    ("allium", lambda w, h, p, r: tpl_flower(w, h, p, r, "plume")),
    ("_flower", lambda w, h, p, r: tpl_flower(w, h, p, r, "disc")),
    ("pitcher_plant", lambda w, h, p, r: tpl_flower(w, h, p, r, "cup")),
    ("torchflower", lambda w, h, p, r: tpl_flower(w, h, p, r, "cup")),
    ("grass", tpl_grass_tuft), ("fern", tpl_grass_tuft),
    ("_roots", tpl_grass_tuft),
    ("_fungus", tpl_sapling), ("kelp", tpl_grass_tuft),
    ("seagrass", tpl_grass_tuft), ("vine", tpl_grass_tuft),
    ("torch", tpl_torch), ("_torch", tpl_torch),
    ("_rail", tpl_rail), ("_door", tpl_door),
    ("_trapdoor", tpl_door),
]
UTIL_ROUTES = [
    ("_bucket", lambda w, h, p, r: tpl_bucket(w, h, p, r)),
    ("water_bucket", lambda w, h, p, r: tpl_bucket(w, h, p, r, (60, 90, 200, 255))),
    ("lava_bucket", lambda w, h, p, r: tpl_bucket(w, h, p, r, (220, 110, 20, 255))),
    ("milk_bucket", lambda w, h, p, r: tpl_bucket(w, h, p, r, (245, 245, 245, 255))),
    ("_bottle", lambda w, h, p, r: tpl_bottle(w, h, p, r)),
    ("potion", lambda w, h, p, r: tpl_bottle(w, h, p, r, (200, 60, 160, 230))),
    ("_rod", tpl_rod), ("stick", tpl_rod),
    ("book", tpl_book), ("paper", tpl_paper), ("string", tpl_string),
    ("_banner", tpl_banner_sheet), ("arrow", tpl_arrow),
    ("leather", tpl_paper),
    ("feather", tpl_string), ("flint", tpl_string),
    ("_seeds", tpl_dust), ("wheat", tpl_grass_tuft),
    ("sugar", tpl_dust),
]

def _outline_pass(arr, pal):
    """Dark outline around the sprite silhouette (readability)."""
    out = arr.copy()
    m = out[..., 3] > 8
    if not m.any():
        return out
    dil = m.copy()
    dil[1:, :] |= m[:-1, :]
    dil[:-1, :] |= m[1:, :]
    dil[:, 1:] |= m[:, :-1]
    dil[:, :-1] |= m[:, 1:]
    ring = dil & ~m
    order = pal.order_by_luma()
    dark = pal.rgba(order[0])
    if 0.299 * dark[0] + 0.587 * dark[1] + 0.114 * dark[2] > 90:
        dark = (30, 30, 34, 255)
    out[ring] = dark
    return out

def synthesize_sprite(rec, name):
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    cat = rec["c"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name, "sprite")
    w, h = rec["w"], rec["h"]
    if cat == "particle":
        return tpl_particle(rec, name, pal)
    # route by suffix keyword (checked from specific -> generic)
    for routes in (TOOL_ROUTES, FOOD_ROUTES, PLANT_ROUTES, UTIL_ROUTES):
        for key, fn in routes:
            if fn is None:
                continue
            if key in base:
                try:
                    out = fn(w, h, pal, rng)
                    if out is not None and out.shape[:2] == (h, w):
                        return _outline_pass(out, pal)
                except Exception:
                    break
    return _outline_pass(tpl_blob_generic(rec, name, pal), pal)
