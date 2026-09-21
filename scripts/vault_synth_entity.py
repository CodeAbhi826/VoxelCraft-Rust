#!/usr/bin/env python3
"""vault_synth_entity.py — entity-skin generators.

The skin-file LAYOUT (which canvas region maps to head/body/limb faces)
is the de-facto skin format — functional geometry. The pixels are our
own: region fills from the measured palette with our noise, and faces
(eyes, mouths) drawn procedurally with our own shapes."""
import numpy as np
from vault_synth_core import (rng_for, fbm, canvas, fill, px, Pal, blob,
                              disc, line)

def _roles(pal):
    order = pal.order_by_luma()
    n = len(order)
    return dict(hi=pal.rgba(order[-1]), lo=pal.rgba(order[0]),
                mid=pal.rgba(order[n // 2]), mid2=pal.rgba(order[max(0, n // 2 - 1)]))

def _opaque_pal(pal, name):
    """Sub-palette of opaque entries only (the transparent entry belongs
    to the UNUSED canvas regions of a skin, never inside a body region)."""
    ids = [i for i in range(len(pal.entries)) if pal.entries[i][3] >= 10]
    if len(ids) < 2:
        return pal
    sub = Pal([list(pal.rgba(i)) for i in ids],
              [max(float(pal.freq[i]), 0.01) for i in ids], name + "#op")
    return sub

def _region_noise(c, x, y, w, h, pal, rng, cells=(4, 2), target_luma=None):
    part = np.zeros((h, w, 4), np.uint8)
    op = _opaque_pal(pal, "region")
    n = fbm(w, h, list(cells), rng)
    n = (n - n.min()) / max(1e-6, float(n.max() - n.min()))
    r = rng.random((h, w))
    v = np.clip(n * 0.5 + r * 0.5, 0, 1)
    order, cum = op.thresholds()
    if target_luma is not None:
        # luma-targeted weighting: pull the tone distribution toward the
        # measured mean-luma fact (weighted toward matching entries)
        lum = np.array([op.luma(i) for i in order], np.float64)
        wgt = np.exp(-((lum - target_luma) ** 2) / (2 * 55.0 ** 2))
        base = np.maximum(np.asarray([op.freq[i] for i in order], np.float64), 0.004)
        eff = base * wgt
        eff = eff / max(1e-9, eff.sum())
        cum = np.cumsum(eff)
    k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
    idx = np.array(order)[k]
    for i in range(len(op.entries)):
        part[idx == i] = op.rgba(i)
    c[y:y + h, x:x + w] = part
    return part

def _eye_color(r):
    """Dark eyes; fall back to near-black if the palette's darkest is
    still bright (invisible on light mobs)."""
    e = r["lo"]
    if 0.299 * e[0] + 0.587 * e[1] + 0.114 * e[2] > 60:
        return (28, 28, 34, 255)
    return e

def _eyes(c, x, y, s, color, style="pair", gap=2):
    """Draw a face's eyes in region starting (x, y) sized s x s (own shapes)."""
    if style == "pair":
        ey = y + s // 2 - 1
        ex0 = x + s // 2 - gap - 1
        ex1 = x + s // 2 + gap
        for dx in range(2):
            fill(c, color, ex0 + dx, ey, 1, 2)
            fill(c, color, ex1 + dx, ey, 1, 2)
    elif style == "wide":
        for k, (ex, dy) in enumerate(((x + s // 2 - 4, -1), (x + s // 2 + 2, -1),
                                      (x + s // 2 - 4, 2), (x + s // 2 + 2, 2))):
            fill(c, color, ex, y + s // 2 + dy, 2, 1)
    elif style == "single":
        fill(c, color, x + s // 2 - 1, y + s // 2 - 1, 2, 2)

def _mouth(c, x, y, s, color, style="flat"):
    my = y + int(s * 0.68)
    if style == "flat":
        fill(c, color, x + s // 2 - 2, my, 4, 1)
    elif style == "open":
        fill(c, color, x + s // 2 - 1, my - 1, 2, 3)
    elif style == "sad":
        fill(c, color, x + s // 2 - 2, my + 1, 1, 1)
        fill(c, color, x + s // 2 + 1, my + 1, 1, 1)
        fill(c, color, x + s // 2 - 1, my, 2, 1)

# ------------------------------------------------------------------ layouts
# 64x64 modern skin regions (functional format facts):
#  head: (0,0,32,16) faces: top(8,0) bottom(16,0) right(0,8) front(8,8) left(16,8) back(24,8)
#  body: (16,16,24,16) | arms (40,16) & (32,48) | legs (0,16) & (16,48)
HEAD64 = dict(whole=(0, 0, 32, 16), front=(8, 8, 8, 8), hat=(32, 0, 32, 16))
BODY64 = (16, 16, 24, 16)
ARM_R = (40, 16, 16, 16)
ARM_L = (32, 48, 16, 16)
LEG_R = (0, 16, 16, 16)
LEG_L = (16, 48, 16, 16)

def gen_humanoid64(rec, name, pal, rng, face_style="pair", eye_color=None):
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.3))
    c = canvas(64, 64)
    r = _roles(pal)
    eye = eye_color or r["lo"]
    # limbs/body: region noise
    for (x, y, w, h) in (BODY64, ARM_R, ARM_L, LEG_R, LEG_L):
        _region_noise(c, x, y, w, h, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
    # head band + face
    hx, hy, hw, hh = 0, 0, 32, 16
    _region_noise(c, hx, hy, hw, hh, pal, np.random.default_rng(rng.integers(1 << 32)))
    fx, fy = 8, 8
    _eyes(c, fx, fy, 8, eye, style=face_style)
    _mouth(c, fx, fy, 8, eye, style="flat")
    legacy_canvas = rec.get("ar", 0.0) > 0.5  # legacy-region canvas fact
    if not legacy_canvas:
        # hat layer: sparse translucent same-noise (classic overlay look)
        _region_noise(c, 32, 0, 32, 16, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
        overlay = c[0:16, 32:64].copy()
        mask = rng.random((16, 32)) < 0.35
        overlay[mask, 3] = 0
        c[0:16, 32:64] = overlay
        # jacket/sleeve overlays (second layer regions)
        for (x, y, w, h) in ((16, 32, 24, 16), (40, 32, 16, 16), (0, 32, 16, 16), (48, 48, 16, 16)):
            if x + w <= 64 and y + h <= 64:
                _region_noise(c, x, y, w, h, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
                m = rng.random((h, w)) < 0.5
                c[y:y + h, x:x + w][m, 3] = 0
    return c

def gen_humanoid32(rec, name, pal, rng, face_style="pair", eye_color=None,
                   camo=False):
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.3))
    c = canvas(64, 32)
    r = _roles(pal)
    eye = eye_color or r["lo"]
    # 64x32 layout: head (0,0,32,16); body (16,16); arm (40,16); leg (0,16)
    _region_noise(c, 0, 0, 32, 16, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
    for (x, y, w, h) in ((16, 16, 24, 16), (40, 16, 16, 16), (0, 16, 16, 16)):
        _region_noise(c, x, y, w, h, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
    fx, fy = 8, 8
    if camo:
        # camo face: extra dark splats on head-front (creeper-family style)
        frng = np.random.default_rng(rng.integers(1 << 32))
        for _ in range(6):
            blob(c, 8 + frng.integers(0, 8), 8 + frng.integers(0, 8),
                 1.5 + frng.random(), 1.5 + frng.random(), frng, r["lo"])
    _eyes(c, fx, fy, 8, eye, style=face_style)
    _mouth(c, fx, fy, 8, eye, style="open" if camo else "flat")
    return c

def gen_quadruped(rec, name, pal, rng, size=(64, 32)):
    """Generic quadruped layout: head band with face + body/leg noise."""
    w, h = size
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.3))
    c = canvas(w, h)
    # head region (0,0,w/2,h/2) with face at left quarter
    _region_noise(c, 0, 0, w, 6, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
    _region_noise(c, 0, 6, w, h - 6, pal, np.random.default_rng(rng.integers(1 << 32)), target_luma=tgt)
    r = _roles(pal)
    # face box: place eyes in upper-left region (functional head-front position)
    fs = max(6, h // 4)
    _eyes(c, 2, 2, fs, _eye_color(r), style="pair", gap=1)
    _mouth(c, 2, 2, fs, _eye_color(r), style="flat")
    # snout shading
    fill(c, r["mid"], 2, 2 + fs, fs // 2, fs // 4)
    return c

def gen_spider(rec, name, pal, rng):
    c = gen_quadruped(rec, name, pal, rng, size=(64, 32))
    r = _roles(pal)
    # many eyes (spider construction: rows of small dots — functional look)
    glow = (200, 60, 40, 255) if 0.299*r["hi"][0] < 120 else r["hi"]
    for (ex, ey) in ((11, 9), (13, 9), (10, 11), (12, 11), (14, 11), (16, 9), (18, 10)):
        px(c, ex, ey, glow)
        px(c, ex + 1, ey, (20, 20, 20, 255))
    return c

def gen_blob_mob(rec, name, pal, rng):
    """Slime/ghast/magma-style: full noise + big face (own expression)."""
    w, h = rec["w"], rec["h"]
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.3))
    c = canvas(w, h)
    _region_noise(c, 0, 0, w, h, pal, rng, cells=(max(4, w // 4), max(3, h // 4)), target_luma=tgt)
    r = _roles(pal)
    fs = max(6, w // 2)
    _eyes(c, w // 4, h // 4, fs, _eye_color(r), style="pair", gap=max(2, fs // 6))
    _mouth(c, w // 4, h // 4, fs, _eye_color(r), style="open")
    return c

def gen_chest(rec, name, pal, rng):
    """Chest layout: vanilla chest canvases are 64x64-ish (front/side/top)."""
    w, h = rec["w"], rec["h"]
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.1))
    c = canvas(w, h)
    r = _roles(pal)
    _region_noise(c, 0, 0, w, h, pal, rng, cells=(max(4, w // 6), max(4, h // 6)), target_luma=tgt)
    # frame
    fill(c, r["lo"], 0, 0, w, 2)
    fill(c, r["lo"], 0, h - 2, w, 2)
    fill(c, r["lo"], 0, 0, 2, h)
    fill(c, r["lo"], w - 2, 0, 2, h)
    # latch (functional: front latch block)
    lx = w // 2 - 3
    ly = int(h * 0.42)
    fill(c, r["hi"], lx, ly, 6, 7)
    fill(c, r["lo"], lx - 1, ly - 1, 8, 1)
    fill(c, r["lo"], lx - 1, ly + 7, 8, 1)
    px(c, lx + 2, ly + 2, r["lo"])
    return c

# ------------------------------------------------------------------ patterns
PATTERN_KINDS = ("stripe_left", "stripe_right", "stripe_center", "stripe_down",
                 "cross", "straight_cross", "diagonal_left", "diagonal_right",
                 "gradient", "gradient_up", "bricks", "globe", "creeper",
                 "flower", "the original publisher", "skull", "triangle", "triangles_bottom",
                 "triangles_top", "square", "circle", "rhombus", "bordure",
                 "chief", "pale", "saltire", "chevron", "loom", "flow",
                 "guster", "field_masoned")

def gen_pattern(rec, name, pal, rng):
    """Banner/shield/decorated-pot heraldry — geometric pattern renderers
    (the pattern grammar is functional data-driven design)."""
    w, h = rec["w"], rec["h"]
    c = canvas(w, h)
    r = _roles(pal)
    fill(c, r["mid2"])
    kind = "plain"
    for k in PATTERN_KINDS:
        if k in name:
            kind = k
            break
    dark = r["lo"]
    lite = r["hi"]
    if kind in ("stripe_left", "pale"):
        fill(c, dark, 0, 0, max(1, w // 4), h)
    elif kind == "stripe_right":
        fill(c, dark, w - max(1, w // 4), 0, w, h)
    elif kind == "stripe_center":
        fill(c, dark, w // 2 - max(1, w // 8), 0, max(1, w // 4), h)
    elif kind == "stripe_down":
        fill(c, dark, 0, 0, w, max(1, h // 4))
    elif kind == "chief":
        fill(c, dark, 0, 0, w, max(1, h // 4))
    elif kind == "gradient":
        for y in range(h):
            t = y / max(1, h - 1)
            col = tuple(int(a + (b - a) * t) for a, b in zip(r["mid2"][:3], dark[:3])) + (255,)
            fill(c, col, 0, y, w, 1)
    elif kind == "gradient_up":
        for y in range(h):
            t = 1 - y / max(1, h - 1)
            col = tuple(int(a + (b - a) * t) for a, b in zip(r["mid2"][:3], dark[:3])) + (255,)
            fill(c, col, 0, y, w, 1)
    elif kind in ("cross", "straight_cross"):
        fill(c, dark, w // 2 - max(1, w // 8), 0, max(1, w // 4), h)
        fill(c, dark, 0, h // 2 - max(1, h // 8), w, max(1, h // 4))
    elif kind in ("diagonal_left", "diagonal_right"):
        d = 1 if kind == "diagonal_left" else -1
        for k in range(-w, w):
            for b in range(max(1, w // 10)):
                x = k + b
                y = (k * h // w) if d > 0 else (h - k * h // w)
                px(c, x, y, dark)
                px(c, x, y + 1, dark)
    elif kind == "bricks":
        for y in range(0, h, max(2, h // 8)):
            fill(c, dark, 0, y, w, 1)
        for y0 in range(0, h, max(2, h // 4)):
            off = (y0 // max(2, h // 4)) % 2
            for x in range(off * (w // 6), w, max(2, w // 3)):
                fill(c, dark, x, y0, 1, max(2, h // 8))
    elif kind == "circle":
        disc(c, w / 2, h / 2, min(w, h) * 0.28, dark)
    elif kind == "square":
        fill(c, dark, w // 4, h // 4, w // 2, h // 2)
    elif kind == "rhombus":
        for k in range(h):
            t = abs(k - h / 2) / (h / 2)
            xw = int((1 - t) * w / 2)
            fill(c, dark, w // 2 - xw // 2, k, max(1, xw), 1)
    elif kind == "bordure":
        bw = max(2, w // 10)
        fill(c, dark, 0, 0, w, bw)
        fill(c, dark, 0, h - bw, w, bw)
        fill(c, dark, 0, 0, bw, h)
        fill(c, dark, w - bw, 0, bw, h)
    elif kind == "chevron":
        for x in range(w):
            y = abs(x - w / 2) * h // w
            fill(c, dark, x, int(h * 0.3) + y, 1, max(2, h // 6))
    elif kind in ("triangle", "triangles_bottom"):
        for x in range(w):
            fill(c, dark, x, h - int(abs(x - w / 2) * 2 * h / w) - 1, 1, int(abs(x - w / 2) * 2 * h / w) + 1)
    elif kind == "triangles_top":
        for x in range(w):
            hh2 = int(abs(x - w / 2) * 2 * h / w)
            fill(c, dark, x, 0, 1, hh2)
    elif kind == "saltire":
        for k in range(min(w, h) * 2):
            px(c, k * w // min(w, h) // 2, k * h // min(w, h) // 2, dark)
            px(c, w - k * w // min(w, h) // 2, k * h // min(w, h) // 2, dark)
    # speckle detail (own dither)
    frng = np.random.default_rng(rng.integers(1 << 32))
    m = frng.random((h, w)) < 0.05
    c[m] = lite
    return c

def gen_bigcanvas(rec, name, pal, rng):
    """Dragon/warden/portal/large canvases: scale-noise compositions."""
    w, h = rec["w"], rec["h"]
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.1))
    c = canvas(w, h)
    _region_noise(c, 0, 0, w, h, pal, rng, cells=(max(6, w // 10), max(6, h // 10)), target_luma=tgt)
    if "portal" in name:
        # starfield portal (own specks)
        frng = np.random.default_rng(rng.integers(1 << 32))
        m = frng.random((h, w)) < 0.04
        c[m] = (220, 230, 255, 255)
    if "warden" in name or "glow" in name or "spot" in name or "eyes" in name:
        frng = np.random.default_rng(rng.integers(1 << 32))
        for _ in range(max(4, (w * h) // 900)):
            x = int(frng.integers(2, w - 2))
            y = int(frng.integers(2, h - 2))
            disc(c, x, y, 1.4 + frng.random() * 2, (90, 200, 190, 255))
    return c

def gen_equipment_layer(rec, name, pal, rng):
    """Armor equipment layers (trim-style): humanoid layout single-tone +
    trim motif, scaled to the actual canvas dims."""
    w, h = rec["w"], rec["h"]
    s = max(1, w // 64)
    # scale factors may differ per axis for non-square canvases
    sx, sy = max(1, w // 64), max(1, h // 64)
    c = canvas(w, h)
    r = _roles(pal)
    def R(x, y, ww, hh):
        return (x * sx, y * sy, ww * sx, hh * sy)
    for (x, y, ww, hh) in (R(8, 8, 8, 8), R(0, 8, 8, 8), R(16, 8, 8, 8),
                           R(24, 8, 8, 8), R(8, 0, 8, 8), R(16, 0, 8, 8),
                           R(16, 16, 24, 16), R(40, 16, 16, 16),
                           R(32, 48, 16, 16), R(0, 16, 16, 16), R(16, 48, 16, 16)):
        x, y, ww, hh = int(x), int(y), int(ww), int(hh)
        x, y = min(x, w - 1), min(y, h - 1)
        ww, hh = min(ww, w - x), min(hh, h - y)
        if ww > 0 and hh > 0:
            fill(c, r["mid2"], x, y, ww, hh)
    # trim motif: geometric edges on the body/head front (own motif)
    trim = r["hi"]
    def F(color, x, y, ww, hh):
        x, y = int(x * sx), int(y * sy)
        ww, hh = int(ww * sx), int(hh * sy)
        if ww > 0 and hh > 0 and x < w and y < h:
            fill(c, color, x, y, min(ww, w - x), min(hh, h - y))
    F(trim, 16, 16, 24, 1)
    F(trim, 16, 31, 24, 1)
    F(trim, 16, 16, 1, 16)
    F(trim, 39, 16, 1, 16)
    F(trim, 8, 8, 8, 1)
    F(trim, 8, 15, 8, 1)
    F(trim, 8, 8, 1, 8)
    F(trim, 15, 8, 1, 8)
    px(c, int(11 * sx), int(11 * sy), trim)
    px(c, int(12 * sx), int(12 * sy), trim)
    return c

# ------------------------------------------------------------------ router
HUMANOID64_MOBS = ("zombie", "husk", "drowned", "skeleton", "wither_skeleton",
                   "piglin", "zombified", "villager", "illager", "player",
                   "witch", "evoker", "vindicator", "pillager", "guardian",
                   "iron_golem", "snow_golem", "golem", "wandering", "stray",
                   "zombie_villager", "nitwit", "creeper_armor")
HUMANOID32_MOBS = ("creeper", "enderman", "blaze", "silverfish", "endermite")
QUADRUPED_MOBS = ("cow", "pig", "sheep", "horse", "llama", "wolf", "fox",
                  "cat", "ocelot", "rabbit", "panda", "polar_bear", "goat",
                  "mooshroom", "donkey", "mule", "strider", "hoglin",
                  "zoglin", "camel", "sniffer", "happy_ghast", "armadillo",
                  "copper_golem", "allay", "vex", "breeze", "frog", "tadpole")
BLOB_MOBS = ("slime", "magma", "ghast", "shulker", "phantom", "parrot",
             "bat", "bee", "chicken", "axolotl", "salmon", "cod", "squid",
             "glow_squid", "dolphin", "turtle", "tropical_fish", "pufferfish")

def synthesize_entity(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name, "entity")
    path = name.lower()
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    frames = max(1, h // w) if h > w * 1.5 else 1
    if frames > 1:
        # animated entity strip: synthesize each frame
        fh = h // frames
        out = canvas(w, h)
        sub = dict(rec)
        sub["h"] = fh
        for f in range(frames):
            part = synthesize_entity(sub, name)
            out[f * fh:(f + 1) * fh] = part[:fh]
        return out
    if "banner" in path or "shield" in path or "decorated_pot" in path or "pattern" in path:
        return gen_pattern(rec, name, pal, rng)
    if "equipment" in path or "trim" in path:
        return gen_equipment_layer(rec, name, pal, rng)
    if "chest" in path and "ender" not in path and w <= 64:
        return gen_chest(rec, name, pal, rng)
    if any(m in path for m in HUMANOID64_MOBS) and (w, h) in ((64, 64), (64, 32)):
        if (w, h) == (64, 32):
            return gen_humanoid32(rec, name, pal, rng)
        return gen_humanoid64(rec, name, pal, rng,
                              face_style="pair",
                              eye_color=(40, 40, 60, 255) if "zombie" in path or "drowned" in path else None)
    if "creeper" in path:
        return gen_humanoid32(rec, name, pal, rng, face_style="pair", camo=True)
    if "enderman" in path:
        c = gen_humanoid32(rec, name, pal, rng)
        fx, fy = 8, 8
        fill(c, (200, 100, 240, 255), fx + 1, fy + 3, 6, 2)  # glowing eye band (own)
        return c
    if "spider" in path:
        return gen_spider(rec, name, pal, rng)
    if "dragon" in path or "warden" in path or "portal" in path or w >= 128:
        return gen_bigcanvas(rec, name, pal, rng)
    if any(m in path for m in QUADRUPED_MOBS):
        return gen_quadruped(rec, name, pal, rng, size=(w, h))
    if any(m in path for m in BLOB_MOBS) or w <= 32:
        return gen_blob_mob(rec, name, pal, rng)
    # default: region noise + face
    tgt = rec["lm"] / max(0.15, 1.0 - rec.get("ar", 0.3))
    c = canvas(w, h)
    _region_noise(c, 0, 0, w, h, pal, rng, cells=(max(4, w // 5), max(4, h // 5)), target_luma=tgt)
    r = _roles(pal)
    fs = max(6, min(w, h) // 2)
    _eyes(c, max(1, w // 5), max(1, h // 5), fs, _eye_color(r), style="pair")
    return c
