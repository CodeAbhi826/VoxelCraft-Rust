#!/usr/bin/env python3
"""vault_synth_blocks.py — block-family generators (v2: MC-style speckle).

Style study findings (clean-room step 1/2 — functional/aggregate facts):
- Reference block textures are PER-PIXEL DITHERED palette noise (classic
  MC speckle), with only a subtle coarse blotch bias — NOT smooth noise.
- Planks: 4 boards + 1px dark seams + staggered vertical joints.
- Ore: dithered base + chunky 2-4px accent clusters with own dither+rim.
- Bricks: mortar = 2nd-frequency palette entry (~25-35% coverage).
- Logs: clustered vertical column tones + per-pixel dither; *_top rings.
- Glass: frame border + transparent interior + sparse diagonal streaks.
- Leaves: speckle + random cutout holes; portal/water: partial alpha.
"""
import numpy as np
from voxel_synth_shim import rng_for, value_noise, fbm, Pal, canvas, fill, px, blob, disc

def _mc_speckle(w, h, pal, rng, bias_cells=0, corr=0.55):
    """MC-style dithered palette noise. Returns index map.

    value = coarse_bias*(1-corr) + per-pixel random*corr, quantized by
    frequency-weighted cumulative thresholds (luma-ordered palette).
    """
    if bias_cells:
        bias = fbm(w, h, [bias_cells, max(2, bias_cells // 2)], rng)
    else:
        bias = np.full((h, w), 0.5, np.float32)
    r = rng.random((h, w))
    v = np.clip(bias * (1 - corr) + r * corr, 0, 1)
    # re-spread to full [0,1] so every palette bucket gets its share
    v = (v - v.min()) / max(1e-6, float(v.max() - v.min()))
    order = pal.order_by_luma()
    order, cum = pal.thresholds()
    k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
    return np.array(order)[k]

def _apply_pal(c, idx, pal):
    for i in range(len(pal.entries)):
        c[idx == i] = pal.rgba(i)

def _anim_frames(rec):
    w, h = rec["w"], rec["h"]
    if h <= w * 1.25:
        return 1
    return max(1, int(round(h / w)))

def _frames_canvas(rec, fn):
    w, h = rec["w"], rec["h"]
    frames = _anim_frames(rec)
    fh = h // frames
    c = canvas(w, h)
    for f in range(frames):
        sub = dict(rec)
        sub["h"] = fh
        part = fn(sub, f)
        c[f * fh:(f + 1) * fh] = part[:fh]
    return c

# ------------------------------------------------------------------ families
def _opaque_alpha(pal):
    """Alpha of the most frequent substantially-opaque palette entry."""
    best = 255
    for i, (c, f) in enumerate(zip(pal.entries, pal.freq)):
        if c[3] >= 128:
            best = c[3]
            break
    return best

def _palette_transparent_frac(pal):
    """Fraction of pixels the palette itself already maps to transparent."""
    return float(sum(f for c, f in zip(pal.entries, pal.freq) if c[3] < 10))

def gen_noise(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    return _frames_canvas(rec, lambda sub, f: _gen_noise_frame(sub, name, pal, rng, f))

def _gen_noise_frame(sub, name, pal, rng, f):
    w, h = sub["w"], sub["h"]
    frng = np.random.default_rng(rng.integers(1 << 32) + f)
    idx = _mc_speckle(w, h, pal, frng, bias_cells=max(3, w // 4))
    c = canvas(w, h)
    _apply_pal(c, idx, pal)
    return c

def gen_flat(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    fill(c, pal.rgba(0))
    if len(pal.entries) > 1 and pal.freq[1] > 0.005:
        m = rng.random((h, w)) < pal.freq[1]
        c[m] = pal.rgba(1)
    return c

def gen_bands(rec, name):
    """Horizontal strata; weak banding = plain dithered noise."""
    rb = rec.get("rb", 0)
    if rb < 13:
        return gen_noise(rec, name)
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    dark = pal.rgba(pal.extreme(False))
    scale = max(1, round(h / 16))
    nb = max(3, h // (4 * scale))
    bh = h // nb
    for b in range(nb):
        y0 = b * bh
        frng = np.random.default_rng(rng.integers(1 << 32))
        part = canvas(w, bh)
        tilt = rng.uniform(-0.15, 0.15)
        order = pal.order_by_luma()
        order, cum = pal.thresholds()
        bias = np.full((bh, w), 0.5 + tilt, np.float32)
        bias += fbm(w, bh, [max(3, w // 4), 2], frng) * 0.2
        r = frng.random((bh, w))
        v = np.clip(bias * 0.45 + r * 0.55, 0, 1)
        v = (v - v.min()) / max(1e-6, float(v.max() - v.min()))
        k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
        _apply_pal(part, np.array(order)[k], pal)
        c[y0:y0 + bh] = part
        if b > 0:
            fill(c, dark, 0, y0, w, 1)
    return c

def gen_planks(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    scale = max(1, round(min(w, h) / 16))
    boards = max(3, h // (4 * scale))
    bh = h // boards
    dark = pal.rgba(pal.extreme(False))
    order = pal.order_by_luma()
    for b in range(boards):
        y0 = b * bh
        frng = np.random.default_rng(rng.integers(1 << 32))
        # per-board tone tilt + per-pixel dither (board grain)
        tilt = rng.uniform(-0.14, 0.14)
        order, cum = pal.thresholds()
        bias = np.full((bh, w), 0.5 + tilt, np.float32)
        # long horizontal streaks: 1px-high noise rows
        streak = value_noise(1, w, max(2, w // 6), frng)[0]
        bias += (streak[None, :] - 0.5) * 0.5
        r = frng.random((bh, w))
        v = np.clip(bias * 0.5 + r * 0.5, 0, 1)
        k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
        part = canvas(w, bh)
        _apply_pal(part, np.array(order)[k], pal)
        # staggered vertical joint (dark 1px cut)
        jx = int(frng.integers(2, max(3, w - 2)))
        fill(part, dark, jx, 0, 1, bh)
        # sparse knot
        if frng.random() < 0.4:
            kx = int(frng.integers(2, max(3, w - 3)))
            ky = int(frng.integers(1, max(2, bh - 2)))
            part[ky, kx] = dark
            part[ky, min(kx + 1, w - 1)] = dark
            if ky + 1 < bh:
                part[ky + 1, kx] = dark
        c[y0:y0 + bh] = part
        if b > 0:
            fill(c, dark, 0, y0, w, 1)
    return c

def gen_bricks(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    scale = max(1, round(min(w, h) / 16))
    bh = 4 * scale
    bw = 8 * scale
    # mortar = 2nd most frequent entry (measured fact: seams ~25-35%)
    mortar_i = 1 if len(pal.entries) > 1 else 0
    mortar = pal.rgba(mortar_i)
    fill(c, mortar)
    body_ids = [i for i in range(len(pal.entries)) if i != mortar_i]
    if len(body_ids) >= 2:
        body_pal = Pal([list(pal.rgba(i)) for i in body_ids],
                       [max(pal.freq[i], 0.01) for i in body_ids], name + "#body")
        order, cum = body_pal.thresholds()
        lookup_pal = body_pal
    else:
        order, cum = pal.thresholds()
        lookup_pal = pal
    rows = int(np.ceil(h / bh))
    for r in range(rows):
        y0 = r * bh
        off = (bw // 2) if r % 2 else 0
        frng = np.random.default_rng(rng.integers(1 << 32))
        for x in range(-bw + off, w + off, bw):
            x0 = x + 1
            bwid = bw - 1
            xx0, xx1 = max(0, x0), min(x0 + bwid, w)
            yy0, yy1 = y0, min(y0 + bh - 1, h)
            if xx1 <= xx0 or yy1 <= yy0 or cum is None:
                continue
            # per-brick tone + per-pixel dither
            tilt = frng.uniform(-0.18, 0.18)
            n = frng.random((yy1 - yy0, xx1 - xx0))
            v = np.clip((0.5 + tilt) * 0.55 + n * 0.45, 0, 1)
            v = (v - v.min()) / max(1e-6, float(v.max() - v.min())) if v.max() > v.min() else v
            k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
            part = canvas(xx1 - xx0, yy1 - yy0)
            _apply_pal(part, np.array(order)[k], lookup_pal)
            c[yy0:yy1, xx0:xx1] = part
    return c

def gen_stripe(rec, name):
    """Vertical bark striations; _top -> concentric rings."""
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    order = pal.order_by_luma()
    order, cum = pal.thresholds()

    def frame(sub, f):
        w, h = sub["w"], sub["h"]
        frng = np.random.default_rng(rng.integers(1 << 32) + f)
        part = canvas(w, h)
        if base.endswith("_top") or "_top" in base:
            yy, xx = np.mgrid[0:h, 0:w]
            cx, cy = (w - 1) / 2, (h - 1) / 2
            d = np.sqrt((xx - cx) ** 2 + (yy - cy) ** 2)
            ring = np.sin(d / max(1.5, min(w, h) / 9) + frng.uniform(0, 3)) * 0.5 + 0.5
            v = np.clip(ring * 0.6 + frng.random((h, w)) * 0.4, 0, 1)
        else:
            # clustered column tones + per-pixel dither = vertical bark
            cols = value_noise(1, w, max(2, w // 5), frng)[0]
            v = np.clip(cols[None, :] * 0.62 + frng.random((h, w)) * 0.38, 0, 1)
        k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
        _apply_pal(part, np.array(order)[k], pal)
        return part
    return _frames_canvas(rec, frame)

def gen_ore(rec, name):
    """Dithered base + chunky dithered accent clusters (own placement)."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    base_ids = [i for i in range(len(pal.entries)) if pal.freq[i] >= 0.08][:5] or list(range(min(3, len(pal.entries))))
    accent_ids = [i for i in range(len(pal.entries)) if i not in base_ids]
    base_pal = Pal([list(pal.rgba(i)) for i in base_ids],
                   [max(pal.freq[i], 0.05) for i in base_ids], name + "#base")
    frng = np.random.default_rng(rng.integers(1 << 32))
    idx = _mc_speckle(w, h, base_pal, frng, bias_cells=max(3, w // 4))
    _apply_pal(c, idx, pal)
    # accent area target
    area_t = sum(pal.freq[i] for i in accent_ids) * w * h
    placed = 0
    tries = 0
    while placed < area_t * 0.8 and tries < 40 and accent_ids:
        tries += 1
        i = accent_ids[int(rng.integers(0, len(accent_ids)))]
        cx = int(rng.integers(2, w - 2))
        cy = int(rng.integers(2, h - 2))
        # chunky cluster: random-walk growth, 3-7 px
        cells = {(cx, cy)}
        for _ in range(int(rng.integers(2, 6))):
            x, y = list(cells)[int(rng.integers(0, len(cells)))]
            dx, dy = [(1, 0), (-1, 0), (0, 1), (0, -1)][int(rng.integers(0, 4))]
            nx, ny = x + dx, y + dy
            if 0 <= nx < w and 0 <= ny < h:
                cells.add((nx, ny))
        for (x, y) in cells:
            # internal dither among accent tones
            ai = i if rng.random() < 0.7 else accent_ids[int(rng.integers(0, len(accent_ids)))]
            c[y, x] = pal.rgba(ai)
        # dark rim pixel (own convention)
        for (x, y) in cells:
            if rng.random() < 0.35:
                for dx, dy in ((1, 0), (0, 1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < w and 0 <= ny < h and (nx, ny) not in cells:
                        c[ny, nx] = pal.rgba(pal.extreme(False))
        placed += len(cells)
    return c

# ------------------------------------------------------------------ specials
def gen_glass(rec, name):
    """Glass family: frame + transparent interior + sparse streaks."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)  # transparent
    order = pal.order_by_luma()
    lite = pal.rgba(order[-1])
    mid = pal.rgba(order[0]) if len(order) < 2 else pal.rgba(order[len(order) // 4])
    # frame: 1px border, corner accents
    fill(c, lite, 0, 0, w, 1)
    fill(c, lite, 0, h - 1, w, 1)
    fill(c, lite, 0, 0, 1, h)
    fill(c, lite, w - 1, 0, 1, h)
    for (cx, cy) in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)):
        c[cy, cx] = mid
    # sparse diagonal streaks (upper-left, lower-right)
    n = max(2, w // 8)
    for k in range(n):
        sx = int(rng.integers(2, max(3, w // 2)))
        sy = int(rng.integers(1, max(2, h // 3)))
        ln = int(rng.integers(2, max(3, w // 3)))
        for t in range(ln):
            x, y = sx + t, sy + t
            if 0 <= x < w - 1 and 0 <= y < h - 1:
                c[y, x] = mid
    for k in range(max(1, n // 2)):
        sx = int(rng.integers(w // 2, w - 2))
        sy = int(rng.integers(h // 2, h - 2))
        ln = int(rng.integers(2, max(3, w // 4)))
        for t in range(ln):
            x, y = sx + t, sy + t
            if 0 <= x < w - 1 and 0 <= y < h - 1:
                c[y, x] = mid
    return c

def gen_leaves(rec, name):
    """Leaves: dithered speckle + random cutout holes (tr fraction)."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    c = canvas(w, h)
    idx = _mc_speckle(w, h, pal, rng, bias_cells=max(3, w // 4))
    _apply_pal(c, idx, pal)
    # extra cutout holes only beyond what the palette already provides
    already = _palette_transparent_frac(pal)
    tr = max(0.05, min(0.6, rec.get("ar", 0.25)))
    extra = max(0.0, tr - already)
    holes = rng.random((h, w)) < extra * 0.85
    c[holes] = (0, 0, 0, 0)
    return c

def gen_fire(rec, name, alpha_vary=True):
    """Fire/portal strips: dithered frames w/ vertical falloff + alpha.
    The transparent fraction is calibrated to the measured tr fact."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    order = pal.order_by_luma()
    order, cum = pal.thresholds()
    base_alpha = _opaque_alpha(pal)
    tr = max(0.05, min(0.8, rec.get("ar", 0.45)))

    def frame(sub, f):
        w, h = sub["w"], sub["h"]
        frng = np.random.default_rng(rng.integers(1 << 32) + f)
        yy, xx = np.mgrid[0:h, 0:w]
        # tall flame cells (coarse vertically), fine horizontally;
        # portals get a swirl bias (rotational drift)
        n = fbm(w, h, [max(3, w // 4), max(5, h // 3)], frng)
        if "portal" in name:
            import math
            ang = np.arctan2(yy - h / 2, xx - w / 2)
            swirl = np.sin(ang * 2 + (xx + yy) * 0.55 + f * 0.6) * 0.5 + 0.5
            n = np.clip(n * 0.55 + swirl * 0.45, 0, 1)
        fall = 1.0 - (yy / max(1, h - 1)) * 0.7
        v = np.clip(n * fall * 1.25 + 0.1, 0, 1)
        k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
        part = canvas(w, h)
        _apply_pal(part, np.array(order)[k], pal)
        # hard-transparent cut ONLY if the palette itself carries no
        # transparent entry (avoid double-counting transparency)
        if _palette_transparent_frac(pal) < 0.02:
            thr = float(np.quantile(v, tr))
            part[v <= thr] = (0, 0, 0, 0)
        if alpha_vary and base_alpha < 250:
            keep = part[..., 3] > 0
            a = np.clip(base_alpha + (frng.random((h, w)) - 0.4) * 70, 30, 255)
            part[..., 3] = np.where(keep, a.astype(np.uint8), 0).astype(np.uint8)
        return part
    return _frames_canvas(rec, frame)

def gen_water(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    rng = rng_for(name)
    base_alpha = _opaque_alpha(pal)
    order = pal.order_by_luma()
    order, cum = pal.thresholds()

    def frame(sub, f):
        w, h = sub["w"], sub["h"]
        frng = np.random.default_rng(rng.integers(1 << 32) + f)
        n = fbm(w, h, [max(3, w // 2), max(2, h // 3)], frng)
        v = np.clip(n * 0.6 + frng.random((h, w)) * 0.4, 0, 1)
        k = np.clip(np.searchsorted(cum, v), 0, len(order) - 1)
        part = canvas(w, h)
        _apply_pal(part, np.array(order)[k], pal)
        part[..., 3] = base_alpha
        return part
    return _frames_canvas(rec, frame)

def gen_web(rec, name):
    """Cobweb: radial + spiral-ish lines on transparent."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    c = canvas(w, h)
    order = pal.order_by_luma()
    lite = pal.rgba(order[-1])
    from voxel_synth_shim import line
    cx, cy = 0, 0  # corner-anchored web
    for ang_d in (0, 45, 90, 135):
        import math
        ex = int(cx + (w - 1) * math.cos(math.radians(ang_d)))
        ey = int(cy + (h - 1) * math.sin(math.radians(abs(ang_d))))
        line(c, cx, cy, min(ex, w - 1), min(ey, h - 1), lite)
    # arcs
    for r in (w // 4, w // 2, 3 * w // 4):
        for x in range(w):
            y = int(cy + abs(r - x) * 0.6)
            if 0 <= y < h and x <= r:
                c[y, x] = lite
    return c

FAMILY_FUNCS = {
    "noise": gen_noise,
    "flat": gen_flat,
    "bands": gen_bands,
    "planks": gen_planks,
    "bricks": gen_bricks,
    "stripe": gen_stripe,
    "ore": gen_ore,
}

def synth_block_special(rec, name):
    """Name-routed special block constructions; None if not special."""
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    n = base.lower()
    if "stained_glass" in n or "tinted_glass" in n:
        return gen_noise(rec, name)  # full translucent panel (no cutout)
    if "glass" in n or "window" in n:
        return gen_glass(rec, name)
    if "_leaves" in n or n.endswith("_leaf") or "azalea" in n and "plant" not in n:
        return gen_leaves(rec, name)
    if "fire" in n and "campfire" not in n and "firefly" not in n and "wildfire" not in n:
        return gen_fire(rec, name)
    if "portal" in n:
        return gen_fire(rec, name)
    if "water" in n or "bubble" in n:
        return gen_water(rec, name)
    if "web" in n or "cobweb" in n:
        return gen_web(rec, name)
    return None

def synthesize_block(rec, name):
    special = synth_block_special(rec, name)
    if special is not None:
        return special
    fam = rec["fam"]
    fn = FAMILY_FUNCS.get(fam)
    if fn is not None:
        return fn(rec, name)
    return None  # sprite-family -> handled by the sprite router
