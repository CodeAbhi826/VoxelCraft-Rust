#!/usr/bin/env python3
"""vault_synth_special.py — font pages (rendered from OUR OFL-licensed
Monocraft), colormaps (functional LUT re-derivation from coarse sampled
facts), environment (celestial discs, clouds, weather strips), paintings
(original abstract art), map marks and misc overlays."""
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from voxel_synth_shim import *  # noqa: F403

FONT_PATH = "/home/z/my-project/voxelcraft/crates/vc-render/assets/Monocraft.ttf"

def _glyph_font(size):
    try:
        return ImageFont.truetype(FONT_PATH, size)
    except Exception:
        return ImageFont.load_default()

# ------------------------------------------------------------------ font pages
def gen_glyph_page(rec, name, base_cp):
    w, h = rec["w"], rec["h"]
    # cell pitch: prefer 8px grid, else largest common divisor <= 16
    for cell in (8, 6, 12, 9, 4, 16, 3):
        if w % cell == 0 and h % cell == 0 and cell <= max(w, h):
            break
    cols = max(1, w // cell)
    rows = max(1, h // cell)
    pal_entry = rec["pal"][0] if rec["pal"] else [255, 255, 255, 255]
    ink = (pal_entry[0], pal_entry[1], pal_entry[2], 255)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    f = _glyph_font(max(6, int(cell * 0.85)))
    for idx in range(cols * rows):
        cp = base_cp + idx
        if cp in (0, 10, 13):
            continue
        x = (idx % cols) * cell
        y = (idx // cols) * cell
        ch = chr(cp)
        try:
            d.text((x, y), ch, font=f, fill=ink)
        except Exception:
            continue
    return np.asarray(im).copy()

def gen_fictional_page(rec, name):
    """ascii_sga / asciillager: our own original fictional alphabet —
    per-codepoint deterministic 8x8 glyphs (no reference letters)."""
    w, h = rec["w"], rec["h"]
    cell = 8 if w % 8 == 0 and h % 8 == 0 else max(2, np.gcd(w, h))
    cols, rows = max(1, w // cell), max(1, h // cell)
    ink = tuple(rec["pal"][0][:3]) + (255,) if rec["pal"] else (255, 255, 255, 255)
    c = np.zeros((h, w, 4), np.uint8)
    rng = np.random.default_rng(0)
    for idx in range(cols * rows):
        # deterministic per-cell strokes (2-3 strokes within the cell)
        gr = np.random.default_rng(1000 + idx)
        x0 = (idx % cols) * cell
        y0 = (idx // cols) * cell
        nstrokes = 2 + (idx % 3)
        for _ in range(nstrokes):
            ax = x0 + int(gr.integers(0, cell))
            ay = y0 + int(gr.integers(0, cell))
            bx = x0 + int(gr.integers(0, cell))
            by = y0 + int(gr.integers(0, cell))
            # stroke as L-shaped path (pixel-script feel)
            for t in range(4):
                x = int(ax + (bx - ax) * t / 3)
                y = int(ay + (by - ay) * t / 3)
                if 0 <= x < w and 0 <= y < h:
                    c[y, x] = ink
            for dy in range(0, 3):
                x, y = bx, by + dy
                if 0 <= x < w and 0 <= y < h:
                    c[y, x] = ink
    return c

# ------------------------------------------------------------------ colormap
def gen_colormap(rec, name):
    w, h = rec["w"], rec["h"]
    lut = rec.get("lut")
    c = np.zeros((h, w, 4), np.uint8)
    if not lut:
        fill_col = rec["pal"][0] if rec["pal"] else [120, 200, 120, 255]
        c[:] = fill_col
        return c
    gh, gw = len(lut), len(lut[0])
    grid = np.array(lut, np.float32)  # (gh, gw, 3)
    gy = np.linspace(0, gh - 1, h)[:, None].repeat(w, 1)
    gx = np.linspace(0, gw - 1, w)[None, :].repeat(h, 0)
    y0 = np.clip(np.floor(gy).astype(int), 0, gh - 2)
    x0 = np.clip(np.floor(gx).astype(int), 0, gw - 2)
    fy = (gy - y0)[..., None]
    fx = (gx - x0)[..., None]
    a = grid[y0, x0]
    b = grid[y0, x0 + 1]
    cc = grid[y0 + 1, x0]
    dd = grid[y0 + 1, x0 + 1]
    out = (a * (1 - fy) * (1 - fx) + b * (1 - fy) * fx
           + cc * fy * (1 - fx) + dd * fy * fx)
    # subtle own jitter
    jitter = np.random.default_rng(1234).integers(-2, 3, size=(h, w, 3))
    out = np.clip(out + jitter, 0, 255).astype(np.uint8)
    c[..., :3] = out
    c[..., 3] = 255
    return c

# ------------------------------------------------------------------ environment
def gen_celestial(rec, name):
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    c = np.zeros((h, w, 4), np.uint8)
    rng = np.random.default_rng(abs(hash(name)) % (2 ** 31))
    base = pal.rgba(pal.mid()) if len(pal.entries) >= 3 else (250, 240, 200, 255)
    lite = pal.rgba(pal.extreme(True))
    dark = pal.rgba(pal.extreme(False))
    cx, cy, r = (w - 1) / 2, (h - 1) / 2, min(w, h) / 2 - 1
    yy, xx = np.mgrid[0:h, 0:w]
    d = np.sqrt((xx - cx) ** 2 + (yy - cy) ** 2)
    disc = d <= r
    c[disc] = base
    c[disc & (d < r * 0.75)] = lite
    # moon phases: shadow crescent (functional phase geometry)
    for kw in ("new_moon", "first_quarter", "third_quarter", "waxing_crescent",
               "waning_crescent", "waxing_gibbous", "waning_gibbous"):
        if kw in name:
            if kw == "new_moon":
                c[disc] = dark
                break
            sx = 1 if "first" in kw or "waxing" in kw else -1
            width_frac = 0.35 if "crescent" in kw else (0.5 if "quarter" in kw else 0.85)
            shadow = (((xx - cx) * sx) / (r * width_frac) + 0.35) ** 2 + ((yy - cy) / r) ** 2 <= 1.05
            c[disc & shadow] = dark
            break
    # pixel grain on the disc (own crater speckle)
    m = rng.random((h, w)) < 0.10
    c[disc & m] = dark
    return c

def gen_clouds(rec, name):
    w, h = rec["w"], rec["h"]
    c = np.zeros((h, w, 4), np.uint8)
    rng = np.random.default_rng(77)
    # blocky cloud blobs (own arrangement, white w/ alpha)
    for _ in range(90):
        cx = rng.integers(0, w)
        cy = rng.integers(0, h)
        bw = int(rng.integers(6, 24))
        bh = int(rng.integers(3, 8))
        for dy in range(bh):
            for dx in range(bw):
                x = int((cx + dx) % w)
                y = int((cy + dy) % h)
                c[y, x] = (255, 255, 255, 180)
    return c

def gen_starfield(rec, name):
    w, h = rec["w"], rec["h"]
    c = np.zeros((h, w, 4), np.uint8)
    rng = np.random.default_rng(99)
    c[:] = (8, 8, 14, 255)
    m = rng.random((h, w)) < 0.012
    c[m] = (190, 190, 210, 255)
    m2 = rng.random((h, w)) < 0.002
    c[m2] = (240, 240, 255, 255)
    return c

def gen_weather(rec, name):
    w, h = rec["w"], rec["h"]
    c = np.zeros((h, w, 4), np.uint8)
    rng = np.random.default_rng(abs(hash(name)) % (2 ** 31))
    snow = "snow" in name
    n_streaks = w // 2
    for k in range(n_streaks):
        x = int(rng.integers(0, w))
        col = (235, 240, 250, 180) if snow else (120, 140, 190, 130)
        if snow:
            # scattered flakes
            for _ in range(8):
                y = int(rng.integers(0, h))
                c[y, x] = col
        else:
            # vertical streak columns
            y0 = int(rng.integers(0, h))
            ln = int(rng.integers(6, 18))
            for y in range(y0, min(h, y0 + ln)):
                c[y, x] = col
    return c

# ------------------------------------------------------------------ paintings
def gen_painting(rec, name):
    """ORIGINAL abstract compositions — the most-protected category in the
    reference set, so the vault ships entirely original gallery works."""
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    rng = np.random.default_rng(abs(hash(name)) % (2 ** 31))
    c = np.zeros((h, w, 4), np.uint8)
    style = rng.integers(0, 4)
    order = pal.order_by_luma()
    lo, mid, hi = (pal.rgba(order[0]), pal.rgba(order[len(order) // 2]),
                   pal.rgba(order[-1]))
    if style == 0:
        # landscape: sky band + hills + sun
        for y in range(h):
            t = y / max(1, h - 1)
            col = tuple(int(a + (b - a) * t) for a, b in zip(hi[:3], mid[:3])) + (255,)
            c[y, :] = col
        n = __import__("voxel_synth_shim", fromlist=["fbm"]).fbm(w, 1, [max(4, w // 6)], rng)[0]
        for x in range(w):
            ytop = int(h * 0.6 - n[x] * h * 0.2)
            for y in range(ytop, h):
                c[y, x] = lo
        disc(c, w * 0.3, h * 0.25, min(w, h) * 0.12, hi)
    elif style == 1:
        # geometric composition (Mondrian-style: our own grid)
        fill(c, mid)
        for _ in range(rng.integers(6, 12)):
            x0, y0 = int(rng.integers(0, w // 2)), int(rng.integers(0, h // 2))
            bw, bh = int(rng.integers(2, w // 2)), int(rng.integers(2, h // 2))
            extra = pal.rgba(order[-2]) if len(order) > 2 else mid
            col = [lo, hi, mid, extra][int(rng.integers(0, 4))]
            fill(c, col, x0, y0, bw, bh)
    elif style == 2:
        # gradient + circles
        for x in range(w):
            t = x / max(1, w - 1)
            col = tuple(int(a + (b - a) * t) for a, b in zip(mid[:3], hi[:3])) + (255,)
            for y in range(h):
                c[y, x] = col
        for _ in range(int(rng.integers(2, 5))):
            disc(c, rng.integers(2, max(3, w - 2)), rng.integers(2, max(3, h - 2)),
                 min(w, h) * 0.14, lo)
    else:
        # pixel-creature blob (abstract mob portrait, own)
        blob(c, w / 2, h * 0.62, w * 0.3, h * 0.3, rng, mid, wob=0.3)
        c[int(h * 0.2):int(h * 0.5), int(w * 0.2):int(w * 0.8)] = hi
        disc(c, w * 0.4, h * 0.35, max(1, w * 0.06), lo)
        disc(c, w * 0.6, h * 0.35, max(1, w * 0.06), lo)
        fill(c, lo, 0, int(h * 0.08), w, int(h * 0.06))
    return c

# ------------------------------------------------------------------ map
def gen_map_bg(rec, name):
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    rng = np.random.default_rng(5)
    c = np.zeros((h, w, 4), np.uint8)
    order = pal.order_by_luma()
    mid = pal.rgba(order[len(order) // 2]) if order else (216, 202, 160, 255)
    hi = pal.rgba(order[-1]) if order else (234, 226, 190, 255)
    lo = pal.rgba(order[0]) if order else (180, 164, 120, 255)
    checker = "checkerboard" in name
    if checker:
        s = 8
        yy, xx = np.mgrid[0:h, 0:w]
        c[(yy // s + xx // s) % 2 == 0] = mid
        c[(yy // s + xx // s) % 2 == 1] = hi
        return c
    fill(c, mid)
    m = rng.random((h, w)) < 0.15
    c[m] = hi
    m2 = rng.random((h, w)) < 0.10
    c[m2] = lo
    # frame border
    b = max(2, w // 16)
    fill(c, lo, 0, 0, w, b)
    fill(c, lo, 0, h - b, w, b)
    fill(c, lo, 0, 0, b, h)
    fill(c, lo, w - b, 0, b, h)
    fill(c, hi, b, b, w - 2 * b, 1)
    fill(c, hi, b, b, 1, h - 2 * b)
    # grid lines (functional map grid)
    g = max(8, w // 8)
    for x in range(b, w - b, g):
        for y in range(b, h - b):
            if rng.random() < 0.5:
                c[y, x] = lo
    return c

def gen_map_decoration(rec, name):
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    c = np.zeros((h, w, 4), np.uint8)
    order = pal.order_by_luma()
    ink = pal.rgba(order[-1]) if order else (220, 40, 40, 255)
    dark = pal.rgba(order[0]) if order else (60, 20, 20, 255)
    if "banner" in name:
        fill(c, ink, 1, 1, w - 2, int(h * 0.7))
        fill(c, dark, 1, int(h * 0.7), w - 2, 1)
    elif "x" in name:
        for k in range(min(w, h)):
            c[k, k] = ink
            c[k, min(w - 1, k)] = ink
            c[min(h - 1, k), k] = ink
    elif "player" in name:
        # player arrow (functional)
        for y in range(h):
            t = y / max(1, h - 1)
            xw = max(1, int(w * (1 - t) * 0.9))
            fill(c, ink, w // 2 - xw // 2, y, xw, 1)
    elif "marker" in name or "target_point" in name:
        disc(c, w / 2, h / 2, min(w, h) / 2 - 1, ink)
        disc(c, w / 2, h / 2, min(w, h) / 4, dark)
    elif "village" in name or "temple" in name or "mansion" in name or "monument" in name or "chambers" in name:
        fill(c, ink, 1, 1, w - 2, h - 2)
        fill(c, dark, 2, 2, w - 4, h - 4)
        fill(c, ink, w // 2 - 1, h // 2 - 1, 2, 2)
    elif "frame" in name:
        fill(c, ink, 0, 0, w, 1)
        fill(c, ink, 0, h - 1, w, 1)
        fill(c, ink, 0, 0, 1, h)
        fill(c, ink, w - 1, 0, 1, h)
    else:
        disc(c, w / 2, h / 2, min(w, h) / 2 - 1, ink)
    return c

# ------------------------------------------------------------------ misc overlays
def gen_misc(rec, name):
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    rng = np.random.default_rng(abs(hash(name)) % (2 ** 31))
    c = np.zeros((h, w, 4), np.uint8)
    order = pal.order_by_luma()
    yy, xx = np.mgrid[0:h, 0:w]
    cx, cy = (w - 1) / 2, (h - 1) / 2
    d = np.sqrt((xx - cx) ** 2 + (yy - cy) ** 2)
    if "vignette" in name:
        a = np.clip((d / (max(w, h) / 2)) ** 2.2 * 255, 0, 255)
        c[..., 3] = a.astype(np.uint8)
        c[..., :3] = 0
        return c
    if "glint" in name:
        # enchanted glint: diagonal streak field (functional shimmer look)
        phase = (xx + yy) % 12
        m = (phase < 3) | ((xx - yy) % 17 < 2)
        col = pal.rgba(order[-1]) if order else (220, 160, 255, 120)
        c[m] = col
        c[~m] = (0, 0, 0, 0)
        c[m & ((phase + rng.integers(0, 1)) % 2 == 0)] = (
            pal.rgba(order[len(order) // 2]) if len(order) > 2 else (180, 120, 220, 90))
        return c
    if "shadow" in name:
        a = (np.clip(1 - d / (max(w, h) / 2), 0, 1) ** 1.8 * 160).astype(np.uint8)
        c[..., :3] = 0
        c[..., 3] = a
        return c
    if "nausea" in name:
        ang = np.arctan2(yy - cy, xx - cx)
        swirl = (np.sin(ang * 3 + d * 0.12) * 0.5 + 0.5)
        c[..., :3] = 20
        c[..., 3] = (swirl * 90).astype(np.uint8)
        return c
    if "pumpkin" in name:
        # pumpkin overlay: orange vignette + eye holes (functional)
        c[..., :3] = np.array([200, 120, 30])[None, None, :]
        a = np.clip((d / (max(w, h) / 2)) ** 2 * 200, 0, 255)
        c[..., 3] = a.astype(np.uint8)
        # eye holes
        e1 = ((xx - w * 0.33) ** 2 + (yy - h * 0.4) ** 2) <= (w * 0.07) ** 2
        e2 = ((xx - w * 0.67) ** 2 + (yy - h * 0.4) ** 2) <= (w * 0.07) ** 2
        c[e1 | e2] = (0, 0, 0, 235)
        return c
    if "scope" in name or "spyglass" in name:
        a = np.where(d <= min(w, h) * 0.35, 0, 235).astype(np.uint8)
        c[..., :3] = 10
        c[..., 3] = a
        line(c, w // 2, 0, w // 2, h - 1, (10, 10, 10, 160))
        line(c, 0, h // 2, w - 1, h // 2, (10, 10, 10, 160))
        return c
    if "underwater" in name:
        c[..., :3] = np.array([30, 60, 160])[None, None, :]
        c[..., 3] = 130
        return c
    if "unknown_pack" in name or "unknown_server" in name:
        # our cube icon
        s = min(w, h) // 2
        x0, y0 = (w - s) // 2, (h - s) // 2
        fill(c, pal.rgba(order[-1]) if order else (140, 140, 140, 255), x0, y0, s, s)
        fill(c, pal.rgba(order[0]) if order else (60, 60, 60, 255), x0, y0, s, 2)
        fill(c, pal.rgba(order[0]) if order else (60, 60, 60, 255), x0, y0, 2, s)
        c = np.rot90(c, 0)
        return c
    if "forcefield" in name:
        disc(c, w / 2, h / 2, min(w, h) / 2 - 1.5, (120, 200, 255, 120))
        ring = (d <= min(w, h) / 2 - 1.5) & (d > min(w, h) / 2 - 3.0)
        c[ring] = (180, 230, 255, 200)
        return c
    if "powder_snow" in name:
        ring = (d <= min(w, h) / 2) & (d > min(w, h) / 2 - 4)
        c[ring] = (235, 245, 255, 180)
        return c
    # fallback: soft blob
    disc(c, w / 2, h / 2, min(w, h) / 2 - 1, pal.rgba(order[len(order) // 2]) if order else (128, 128, 128, 255))
    return c

# ------------------------------------------------------------------ trims
def gen_trim(rec, name):
    """trims/items/*_trim.png — armor silhouettes tinted by trim palette."""
    from vault_synth_sprites import tpl_helmet, tpl_chest, tpl_leggings, tpl_boots
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    rng = np.random.default_rng(3)
    if "leggings" in name:
        return tpl_leggings(w, h, pal, rng)
    if "boots" in name:
        return tpl_boots(w, h, pal, rng)
    if "helmet" in name or "helmet" in name:
        return tpl_helmet(w, h, pal, rng)
    return tpl_chest(w, h, pal, rng)

# ------------------------------------------------------------------ mob effect icons
def gen_effect_icon(rec, name):
    """mob_effect/*: abstract emblems (own design, thematic keyword hints)."""
    w, h = rec["w"], rec["h"]
    pal = __import__("voxel_synth_shim", fromlist=["Pal"]).Pal(rec["pal"], rec["pc"], name)
    rng = np.random.default_rng(abs(hash(name)) % (2 ** 31))
    c = np.zeros((h, w, 4), np.uint8)
    order = pal.order_by_luma()
    bgc = pal.rgba(order[0]) if order else (20, 20, 40, 255)
    ink = pal.rgba(order[-1]) if order else (220, 220, 255, 255)
    accent = pal.rgba(order[len(order) // 2]) if len(order) > 2 else ink
    fill(c, bgc, 2, 2, w - 4, h - 4)
    fill(c, (0, 0, 0, 0), 0, 0, w, 2)
    fill(c, (0, 0, 0, 0), 0, h - 2, w, 2)
    fill(c, (0, 0, 0, 0), 0, 0, 2, h)
    fill(c, (0, 0, 0, 0), w - 2, 0, 2, h)
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    if "speed" in base or "haste" in base or "jump" in base:
        # chevron up (own motif)
        for k in range(w // 3):
            line(c, w // 2 - k, h // 2 + k - h // 6, w // 2 + k, h // 2 + k - h // 6, ink)
    elif "strength" in base or "power" in base:
        disc(c, w / 2, h / 2, min(w, h) * 0.28, ink)
        disc(c, w / 2, h / 2, min(w, h) * 0.12, accent)
    elif "regen" in base or "heal" in base or "health" in base:
        fill(c, ink, w // 2 - 1, h // 4, 2, h // 2)
        fill(c, ink, w // 4, h // 2 - 1, w // 2, 2)
    elif "poison" in base or "wither" in base or "decay" in base:
        for k in range(3):
            disc(c, w * (0.3 + k * 0.2), h * (0.65 - k * 0.1), min(w, h) * 0.12, ink)
    elif "night_vision" in base or "vision" in base or "glowing" in base:
        disc(c, w / 2, h / 2, min(w, h) * 0.3, ink, outline=accent)
        disc(c, w / 2, h / 2, min(w, h) * 0.1, accent)
    elif "fire" in base or "burn" in base:
        for k in range(4):
            fill(c, ink, w // 2 - 2 + k, h // 2 + h // 6 - k * 2, 1, 3, )
    else:
        # generic emblem: ring + dot
        disc(c, w / 2, h / 2, min(w, h) * 0.3, ink)
        disc(c, w / 2, h / 2, min(w, h) * 0.14, bgc)
    return c
