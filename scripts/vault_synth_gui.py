#!/usr/bin/env python3
"""vault_synth_gui.py — GUI family generators (widgets, HUD icons,
container panels, backgrounds, signs). Widget geometry (button 200x20,
heart 9x9, bars 182x5, slot grids) is functional; bevel detailing, grain
and icon art are our own expression."""
import numpy as np
from PIL import Image, ImageDraw
from vault_synth_core import (rng_for, canvas, fill, px, Pal, blob, disc,
                              line, bevel_panel, fbm)

def _roles(pal):
    order = pal.order_by_luma()
    n = len(order)
    # fill = MOST FREQUENT entry (the body color), not the luma median
    fillc = pal.rgba(0) if len(pal.entries) else pal.rgba(order[n // 2])
    mid2 = pal.rgba(order[n // 2])
    return dict(hi=pal.rgba(order[-1]), lo=pal.rgba(order[0]),
                mid=fillc, mid2=mid2,
                bright=pal.rgba(order[-1]), dark=pal.rgba(order[0]))

def _grain(c, rng, color, density=0.06):
    h, w = c.shape[:2]
    m = rng.random((h, w)) < density
    st = c[m]
    for i in range(len(st)):
        st[i] = color
    c[m] = color

# ------------------------------------------------------------------ widgets
def gen_button(rec, name, state="normal"):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name, state)
    c = canvas(w, h)
    body = r["mid"] if state != "highlighted" else r["mid2"]
    if state == "disabled":
        body = tuple(int(v * 0.55) for v in r["mid"][:3]) + (255,)
    border = r["dark"]
    light = r["hi"] if state != "disabled" else tuple(int(v * 0.8) for v in r["hi"][:3]) + (255,)
    bevel_panel(c, 0, 0, w, h, border, light, r["lo"], body)
    # subtle inner grain (own dither)
    yy, xx = np.mgrid[0:h, 0:w]
    keep = (yy >= 3) & (yy < h - 3) & (xx >= 3) & (xx < w - 3)
    gm = rng.random((h, w)) < 0.08
    c[keep & gm] = r["mid2"]
    # top gradient band
    if state == "highlighted":
        c[2:h // 2, 2:w - 2] = np.clip(
            c[2:h // 2, 2:w - 2].astype(np.int32) + 10, 0, 255).astype(np.uint8)
    return c

def gen_slider_track(rec, name, state="normal"):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    bevel_panel(c, 0, 0, w, h, r["dark"], r["lo"], r["mid2"], body(r, state))
    # groove: center horizontal recess
    gy0 = h // 2 - 2
    fill(c, r["lo"], 3, gy0, w - 6, 4)
    fill(c, r["dark"], 3, gy0, w - 6, 1)
    fill(c, r["dark"], 3, gy0 + 3, w - 6, 1)
    return c

def body(r, state):
    if state == "highlighted":
        return r["mid2"]
    if state == "disabled":
        return tuple(int(v * 0.7) for v in r["mid"][:3]) + (255,)
    return r["mid"]

def gen_slider_handle(rec, name, state="normal"):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], body(r, state))
    # grip lines
    for y in range(4, h - 4, 3):
        fill(c, r["lo"], 2, y, w - 4, 1)
    return c

def gen_scroller(rec, name, bg=False):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    if bg:
        fill(c, r["dark"])
        fill(c, r["lo"], 0, 0, 1, h)
        fill(c, r["lo"], w - 1, 0, 1, h)
    else:
        bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], r["mid2"])
        fill(c, r["mid"], 1, h // 4, 1, h // 2)
        fill(c, r["mid"], w - 2, h // 4, 1, h // 2)
    return c

def gen_checkbox(rec, name, state="normal"):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name, state)
    c = canvas(w, h)
    sel = "selected" in name
    bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"],
                body(r, state))
    inner = tuple(int(v * 0.8) for v in r["mid"][:3]) + (255,)
    fill(c, inner, 2, 2, w - 4, h - 4)
    fill(c, r["dark"], 2, 2, w - 4, 1)
    fill(c, r["dark"], 2, h - 3, w - 4, 1)
    fill(c, r["dark"], 2, 2, 1, h - 4)
    fill(c, r["dark"], w - 3, 2, 1, h - 4)
    if sel:
        # our own check mark
        line(c, w // 3, h // 2, int(w * 0.45), int(h * 0.68), r["hi"])
        line(c, int(w * 0.45), int(h * 0.68), int(w * 0.72), int(h * 0.3), r["hi"])
        line(c, w // 3, h // 2 + 1, int(w * 0.45), int(h * 0.68) + 1, r["hi"])
    return c

def gen_tab(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    selected = "selected" in name
    c = canvas(w, h)
    fillc = r["mid"] if selected else tuple(int(v * 0.82) for v in r["mid"][:3]) + (255,)
    # trapezoid: narrow at top (attached at bottom edge)
    for y in range(h):
        inset = max(0, (h - 4 - y) // 4)
        fill(c, fillc, inset, y, w - 2 * inset, 1)
    y = 0
    # top edge treatment
    fill(c, r["dark"], 0, 0, w, 1)
    fill(c, r["hi"] if selected else r["mid2"], 0, 1, w, 1)
    return c

def gen_text_field(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    bevel_panel(c, 0, 0, w, h, r["dark"], r["lo"], r["hi"], r["dark"])
    field = tuple(int(v * 0.35) for v in r["hi"][:3]) + (255,)
    fill(c, field, 2, 2, w - 4, h - 4)
    return c

def gen_slot_frame(rec, name):
    """Modern slot frame = 4 corner brackets (functional)."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    arm = max(4, w // 8)
    th = 2
    for (cx, cy, dx, dy) in ((0, 0, 1, 1), (w, 0, -1, 1), (0, h, 1, -1), (w, h, -1, -1)):
        x0 = min(cx, cx + dx * arm)
        y0 = min(cy, cy + dy * arm)
        fill(c, r["hi"], x0, y0, arm if dx > 0 else arm, th)
        fill(c, r["hi"], x0, y0, th, arm if dy > 0 else arm)
    # dark offset shadow corners
    for (cx, cy, dx, dy) in ((0, 0, 1, 1), (w, 0, -1, 1), (0, h, 1, -1), (w, h, -1, -1)):
        fill(c, r["lo"], min(cx, cx + dx * arm), cy + dy, arm, 1)
        fill(c, r["lo"], cx + dx, min(cy, cy + dy * arm), 1, arm)
    return c

def gen_cross_button(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], r["mid2"])
    line(c, w // 3, h // 3, int(w * 0.67), int(h * 0.67), r["lo"])
    line(c, int(w * 0.67), h // 3, w // 3, int(h * 0.67), r["lo"])
    return c

def gen_lock_button(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    state = "disabled" if "disabled" in name else ("highlighted" if "highlighted" in name else "normal")
    bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], body(r, state))
    locked = "unlocked" not in name
    # padlock glyph (own construction)
    bw = w // 2
    bx = (w - bw) // 2
    if locked:
        # shackle arc
        for k in range(5):
            px(c, bx - 1 + k, h // 3 - abs(2 - k) // 2, r["lo"])
            px(c, bx + bw - 1 - k, h // 3 - abs(2 - k) // 2, r["lo"])
        fill(c, r["lo"], bx - 1, h // 3, 1, h // 4)
        fill(c, r["lo"], bx + bw, h // 3, 1, h // 4)
    else:
        fill(c, r["lo"], bx - 2, h // 3, 1, h // 4)
        fill(c, r["lo"], bx + bw + 1, h // 3, 1, h // 4)
    fill(c, r["lo"], bx, h // 2, bw, h // 3)
    fill(c, r["hi"], bx + 1, h // 2 + 1, bw - 2, 1)
    px(c, w // 2, int(h * 0.62), r["hi"])
    return c

def gen_page_arrow(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    forward = "forward" in name
    c = canvas(w, h)
    fill(c, r["mid"], 0, 1, w, h - 2)
    fill(c, r["hi"], 0, 1, w, 2)
    fill(c, r["lo"], 0, h - 3, w, 2)
    d = 1 if forward else -1
    x0 = w // 2 - d * 2
    for k in range(4):
        yy0 = h // 2 - k
        yy1 = h // 2 + k
        x = x0 + d * k
        line(c, x, yy0, x, yy1, r["dark"])
    # trim to the chevron silhouette (arrow shape only, transparent rest)
    yy, xx = np.mgrid[0:h, 0:w]
    cx = x0 + d * 2
    body = np.abs((xx - cx) * 1.0) <= (h / 2 + 1)
    c[~body] = (0, 0, 0, 0)
    return c

# ------------------------------------------------------------------ HUD
def _heart_mask(w, h):
    yy, xx = np.mgrid[0:h, 0:w]
    cx, cy = (w - 1) / 2, (h - 1) / 2
    r1 = ((xx - cx + 1.2) ** 2 + (yy - cy - 1.2) ** 2) <= (w * 0.30) ** 2
    r2 = ((xx - cx - 1.2) ** 2 + (yy - cy - 1.2) ** 2) <= (w * 0.30) ** 2
    tri = (np.abs(xx - cx) * 1.6 + (yy - cy) * 1.8 <= w * 0.42) & (yy >= cy)
    return r1 | r2 | tri

def gen_heart(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    m = _heart_mask(w, h)
    fillc = r["mid2"]
    if "container" in name or "empty" in name.replace("food", "x"):
        if "container" in name:
            fillc = (0, 0, 0, 0)
            ring = np.zeros_like(m)
            ring[1:, :] |= m[:-1, :]; ring[:-1, :] |= m[1:, :]
            ring[:, 1:] |= m[:, :-1]; ring[:, :-1] |= m[:, 1:]
            c[ring & ~m] = r["lo"]
            return c
    c[m] = fillc
    # variants: poisoned/withered/frozen/absorbing use their own palettes —
    # the spec palette already carries the right hues.
    if "half" in name:
        c[m & (np.mgrid[0:h, 0:w][1] < w // 2 + 1)] = fillc
        c[m & (np.mgrid[0:h, 0:w][1] >= w // 2 + 1)] = r["lo"]
    # highlight pixel
    yy, xx = np.mgrid[0:h, 0:w]
    c[m & (xx < w * 0.4) & (yy < h * 0.4)] = r["hi"]
    if "blinking" in name:
        c[m] = np.clip(c[m].astype(np.int32) + 40, 0, 255).astype(np.uint8)
    if "hardcore" in name:
        # own touch: darker rim
        yy, xx = np.mgrid[0:h, 0:w]
        c[m & ((yy == 0) | (yy == h - 1))] = r["lo"]
    return c

def gen_food_icon(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    # drumstick: shank + meat blob (own construction)
    empty = "empty" in name
    half = "half" in name
    yy, xx = np.mgrid[0:h, 0:w]
    meat = ((xx - w * 0.58) ** 2 / (w * 0.26) ** 2 + (yy - h * 0.42) ** 2 / (h * 0.24) ** 2) <= 1
    shank = (np.abs((yy - h * 0.72) - (xx - w * 0.30) * 0.8) <= 1.2) & (xx < w * 0.55) & (xx > w * 0.22)
    if empty:
        c[meat | shank] = r["lo"]
    else:
        c[meat] = r["mid2"]
        c[meat & (xx > w * 0.62)] = r["mid"]
        c[shank] = r["hi"]
        if half:
            c[meat & (yy > h * 0.5)] = r["lo"]
    return c

def gen_air_icon(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    disc(c, w / 2, h / 2, min(w, h) / 2 - 1.2, r["mid2"], outline=r["hi"])
    if "bursting" in name:
        rng = rng_for(name)
        for k in range(5):
            ang = rng.uniform(0, 6.28)
            x = int(w / 2 + (w * 0.45) * np.cos(ang))
            y = int(h / 2 + (h * 0.45) * np.sin(ang))
            px(c, x, y, r["mid2"])
    if "empty" in name:
        c[..., 3] = (c[..., 3] > 0).astype(np.uint8) * 120
    return c

def gen_armor_icon(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    # mini chestplate silhouette
    yy, xx = np.mgrid[0:h, 0:w]
    m = (np.abs(xx - w / 2) < w * 0.34) & (yy > h * 0.25) & (yy < h * 0.85)
    m |= (np.abs(xx - w / 2) < w * 0.5) & (yy > h * 0.2) & (yy < h * 0.4)
    hole = (np.abs(xx - w / 2) < w * 0.12) & (yy < h * 0.32)
    m &= ~hole
    if "empty" in name:
        c[m] = (r["lo"][0], r["lo"][1], r["lo"][2], 140)
    else:
        c[m] = r["mid2"]
        c[m & (yy < h * 0.35)] = r["hi"]
        if "half" in name:
            c[m & (yy > h * 0.55)] = r["lo"]
    return c

def gen_crosshair(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    cx = w // 2
    cy = h // 2
    # plus arms with gap (functional crosshair construction)
    for d in range(2, max(3, w // 2 + 2)):
        for k in (-1, 0, 1):
            px(c, cx + d, cy + k, r["hi"])
            px(c, cx - d, cy + k, r["hi"])
            px(c, cx + k, cy + d, r["hi"])
            px(c, cx + k, cy - d, r["hi"])
    px(c, cx, cy, r["lo"])
    return c

def gen_bar(rec, name, kind="xp"):
    """182x5 bars: xp / boss bars (opaque; the original authored variants functional)."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    is_bg = "background" in name
    def opaque(col):
        return (col[0], col[1], col[2], 255)
    fillc = opaque(r["lo"]) if is_bg else opaque(r["mid2"])
    fill(c, fillc, 0, 0, w, h)
    if not is_bg:
        fill(c, opaque(r["hi"]), 0, 0, w, 1)
        fill(c, opaque(r["mid"]), 0, h - 1, w, 1)
    if is_bg:
        fill(c, opaque(r["dark"]), 0, 0, w, 1)
    else:
        # segment ticks
        n = 10
        if "the original authored_" in name:
            n = int(name.split("the original authored_")[1].split("_")[0])
        seg = w / n
        for i in range(1, n):
            x = int(i * seg)
            fill(c, (0, 0, 0, 0), x, 0, max(1, int(seg * 0.18)), h)
    return c

def gen_hotbar(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], r["mid"])
    # 9 slots (functional layout)
    sw = w / 9
    for i in range(1, 9):
        x = int(i * sw)
        fill(c, r["lo"], x, 2, 1, h - 4)
        fill(c, r["hi"], x + 1, 2, 1, h - 4)
    # selection slot markers
    return c

def gen_effect_bg(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    fill(c, r["lo"], 1, 1, w - 2, h - 2)
    fill(c, r["dark"], 0, 0, w, 1)
    fill(c, r["dark"], 0, h - 1, w, 1)
    fill(c, r["dark"], 0, 0, 1, h)
    fill(c, r["dark"], w - 1, 0, 1, h)
    return c

# ------------------------------------------------------------------ containers
CONTAINER_GRIDS = {
    "crafting": [("grid", 3, 3, 0.13, 0.18), ("result", 1, 1, 0.68, 0.30)],
    "furnace": [("slot", 1, 1, 0.30, 0.16), ("slot", 1, 1, 0.30, 0.52),
                ("result", 1, 1, 0.68, 0.34)],
    "blast_furnace": [("slot", 1, 1, 0.30, 0.16), ("slot", 1, 1, 0.30, 0.52),
                      ("result", 1, 1, 0.68, 0.34)],
    "smoker": [("slot", 1, 1, 0.30, 0.16), ("slot", 1, 1, 0.30, 0.52),
               ("result", 1, 1, 0.68, 0.34)],
    "anvil": [("slot", 1, 1, 0.28, 0.30), ("slot", 1, 1, 0.50, 0.30),
              ("result", 1, 1, 0.72, 0.30)],
    "brewing": [("slot", 1, 1, 0.42, 0.16), ("slot", 3, 1, 0.30, 0.52),
                ("slot", 1, 1, 0.42, 0.82)],
    "hopper": [("grid", 5, 1, 0.22, 0.42)],
    "dispenser": [("grid", 3, 3, 0.28, 0.16)],
    "crafter": [("grid", 3, 3, 0.28, 0.14)],
    "loom": [("slot", 1, 1, 0.20, 0.28), ("slot", 1, 1, 0.34, 0.28),
             ("slot", 1, 1, 0.48, 0.28), ("result", 1, 1, 0.78, 0.28)],
    "cartography": [("slot", 1, 1, 0.24, 0.30), ("slot", 1, 1, 0.38, 0.30),
                    ("result", 1, 1, 0.74, 0.30)],
    "grindstone": [("slot", 1, 1, 0.30, 0.24), ("slot", 1, 1, 0.44, 0.24),
                   ("result", 1, 1, 0.70, 0.24)],
    "smithing": [("slot", 1, 1, 0.24, 0.26), ("slot", 1, 1, 0.38, 0.26),
                 ("slot", 1, 1, 0.52, 0.26), ("result", 1, 1, 0.76, 0.26)],
    "enchanting": [("slot", 1, 1, 0.14, 0.20), ("row", 3, 1, 0.50, 0.18),
                   ("row", 3, 1, 0.50, 0.42), ("row", 3, 1, 0.50, 0.66)],
    "beacon": [("slot", 1, 1, 0.42, 0.72), ("grid", 1, 1, 0.44, 0.42)],
    "villager": [("slot", 1, 1, 0.24, 0.30), ("slot", 1, 1, 0.38, 0.30),
                 ("result", 1, 1, 0.72, 0.30)],
    "horse": [("slot", 1, 1, 0.18, 0.22), ("slot", 1, 1, 0.18, 0.52)],
    "shulker_box": [("grid", 9, 3, 0.12, 0.16)],
    "generic_54": [("grid", 9, 6, 0.12, 0.10)],
    "nautilus": [("slot", 1, 1, 0.42, 0.40)],
}

def _slot(c, x, y, s, r):
    bevel_panel(c, x, y, s, s, r["dark"], r["lo"], r["hi"], r["mid"])
    inner = tuple(int(v * 0.6) for v in r["mid"][:3]) + (255,)
    fill(c, inner, x + 2, y + 2, s - 4, s - 4)

def gen_container_panel(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name, "panel")
    c = canvas(w, h)  # margins stay transparent (functional blit canvas)
    # panel area = measured opaque fraction; vanilla aspect 176:166
    opq = max(0.15, 1.0 - rec.get("ar", 0.45))
    area = opq * w * h
    pw = int(round(np.sqrt(area * (176.0 / 166.0))))
    ph = int(round(area / max(1, pw)))
    pw, ph = min(pw, w), min(ph, h)
    px, py = (w - pw) // 2, (h - ph) // 2
    # window frame + body
    bevel_panel(c, px, py, pw, ph, r["dark"], r["hi"], r["lo"], r["mid"])
    body = tuple(int(v * 0.92) for v in r["mid"][:3]) + (255,)
    fill(c, body, px + 2, py + 2, pw - 4, ph - 4)
    # subtle body grain (own dither)
    yy, xx = np.mgrid[0:h, 0:w]
    keep = (yy > py + 3) & (yy < py + ph - 3) & (xx > px + 3) & (xx < px + pw - 3)
    gm = rng.random((h, w)) < 0.02
    c[keep & gm] = r["mid2"]
    # container-specific grids (functional layouts), inside the panel
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    grid_key = None
    for k in CONTAINER_GRIDS:
        if k in base:
            grid_key = k
            break
    if grid_key:
        s = max(6, int(min(pw, ph) * 0.055))
        for kind, cols, rows, fx, fy in CONTAINER_GRIDS[grid_key]:
            gx = px + int(pw * fx)
            gy = py + int(ph * fy)
            for ry in range(rows):
                for rx in range(cols):
                    _slot(c, gx + rx * (s + 2), gy + ry * (s + 2), s, r)
    else:
        # player inventory: 9x3 + hotbar 9x1 (functional layout)
        s = max(6, int(min(pw, ph) * 0.055))
        for ry in range(3):
            for rx in range(9):
                _slot(c, px + int(pw * 0.12) + rx * (s + 2), py + int(ph * 0.28) + ry * (s + 2), s, r)
        for rx in range(9):
            _slot(c, px + int(pw * 0.12) + rx * (s + 2), py + int(ph * 0.62), s, r)
    return c

# ------------------------------------------------------------------ backgrounds
def gen_menu_bg(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name)
    c = canvas(w, h)
    # translucent dark veil w/ dither (the classic menu dimmer)
    base_a = 110
    for i, entry in enumerate(pal.entries):
        if entry[3] > 8 and entry[3] < 200:
            base_a = entry[3]
            break
    dark = (int(r["lo"][0] * 0.4), int(r["lo"][1] * 0.4), int(r["lo"][2] * 0.4), base_a)
    darker = (int(r["lo"][0] * 0.25), int(r["lo"][1] * 0.25), int(r["lo"][2] * 0.25), base_a)
    mid = (int(r["lo"][0] * 0.55), int(r["lo"][1] * 0.55), int(r["lo"][2] * 0.55), base_a)
    n = fbm(w, h, [4, 2], rng)
    c[n < 0.45] = darker
    c[(n >= 0.45) & (n < 0.75)] = dark
    c[n >= 0.75] = mid
    return c

def gen_separator(rec, name):
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    c = canvas(w, h)
    # 1px center gradient line, rest transparent
    y0 = h // 2
    for x in range(w):
        t = x / max(1, w - 1)
        v = tuple(int(a + (b - a) * t) for a, b in zip(r["mid2"][:3], r["hi"][:3])) + (255,)
        for y in range(max(0, y0 - 0), min(h, y0 + 1)):
            px(c, x, y, v)
    return c

def gen_sign_board(rec, name):
    """Sign/hanging-sign board: planks-style board with frame (own knots)."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name)
    c = canvas(w, h)
    fill(c, r["mid2"])
    # horizontal grain
    n = fbm(w, 1, [max(3, w // 4)], rng)[0]
    for x in range(w):
        v = n[x]
        if v < 0.35:
            col = r["mid"]
        elif v > 0.8:
            col = r["hi"]
        else:
            col = r["mid2"]
        for y in range(2, h - 2):
            if (x + y) % 7 == 0 and rng.random() < 0.15:
                continue
            px(c, x, y, col)
    fill(c, r["lo"], 0, 0, w, 2)
    fill(c, r["lo"], 0, h - 2, w, 2)
    fill(c, r["hi"], 0, 2, w, 1)
    fill(c, r["hi"], 0, h - 3, w, 1)
    return c

def gen_generic_panel_sprite(rec, name):
    """GUI icons without a template: transparent-aware emblem, chip or
    progress indicator."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name, "icon")
    c = canvas(w, h)
    tr = rec.get("ar", 0.0)
    if "progress" in name or "lit_" in name or "burn" in name:
        # opaque progress indicator (arrow/flame bar shapes)
        bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], r["mid2"])
        # fill fraction ~60% (progress look)
        fill(c, r["hi"], 2, 2, max(2, int(w * 0.6)) - 3, h - 4)
        return c
    if tr > 0.25 or w <= 24:
        # sprite-style: our own emblem on transparent background
        disc(c, w / 2, h / 2, min(w, h) / 2 - 1, r["mid2"])
        disc(c, w * 0.42, h * 0.42, min(w, h) * 0.18, r["hi"])
        ring = disc(np.zeros((h, w)), w / 2, h / 2, min(w, h) / 2 - 1.4, 0)
        c[ring] = r["lo"]
    else:
        bevel_panel(c, 0, 0, w, h, r["dark"], r["hi"], r["lo"], r["mid2"])
    return c

def gen_illustration(rec, name):
    """Realms/presets illustrations: ORIGINAL abstract compositions."""
    w, h = rec["w"], rec["h"]
    pal = Pal(rec["pal"], rec["pc"], name)
    r = _roles(pal)
    rng = rng_for(name, "art")
    c = canvas(w, h)
    # sky gradient
    for y in range(h):
        t = y / max(1, h - 1)
        col = tuple(int(a + (b - a) * t) for a, b in zip(r["hi"][:3], r["mid2"][:3])) + (255,)
        for x in range(w):
            px(c, x, y, col)
    # terrain silhouette bands
    for b in range(3):
        yb = int(h * (0.55 + b * 0.16))
        n = fbm(w, 1, [max(6, w // 8)], np.random.default_rng(rng.integers(1 << 32)))[0]
        for x in range(w):
            ytop = yb - int(n[x] * h * 0.12)
            for y in range(ytop, h):
                px(c, x, y, r["lo"] if b == 2 else r["mid"])
    # sun disc
    disc(c, w * rng.uniform(0.2, 0.8), h * rng.uniform(0.15, 0.3), min(w, h) * 0.1, r["hi"])
    return c
