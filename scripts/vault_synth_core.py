#!/usr/bin/env python3
"""vault_synth_core.py — procedural pixel-art synthesis core.

Clean-room chain, step 3 (LEGAL-COMPLIANCE.md §1.4): every pixel of the
vault is generated from (a) the functional spec (dims, palette histogram,
aggregate stats — numeric facts) and (b) procedural rules seeded per
texture name. The reference zip is NEVER opened by any of these modules.

Palette values enter as numeric facts (§4.2) and receive a small seeded
perturbation (±2/channel) so even our palette entries are our own.
"""
import hashlib
import numpy as np
from PIL import Image, ImageDraw

# ------------------------------------------------------------------ rng
def rng_for(name, salt=""):
    seed = int(hashlib.md5(f"{salt}|{name}".encode()).hexdigest()[:16], 16)
    return np.random.default_rng(seed)

# ------------------------------------------------------------------ noise
def _smoothstep(t):
    return t * t * (3.0 - 2.0 * t)

def value_noise(w, h, cell, rng, tileable=True):
    """Smooth tileable value noise in [0,1]."""
    gx = max(int(np.ceil(w / cell)) + 1, 2)
    gy = max(int(np.ceil(h / cell)) + 1, 2)
    lattice = rng.random((gy, gx))
    if tileable:
        lattice[:, -1] = lattice[:, 0]
        lattice[-1, :] = lattice[0, :]
    ys = np.arange(h, dtype=np.float32) / cell
    xs = np.arange(w, dtype=np.float32) / cell
    yi = np.floor(ys).astype(int) % (gy - 1)
    xi = np.floor(xs).astype(int) % (gx - 1)
    grid = np.take(np.take(lattice, yi, axis=0), xi, axis=1)
    grid_b = np.take(np.take(lattice, (yi + 1) % gy, axis=0), xi, axis=1)
    grid_r = np.take(np.take(lattice, yi, axis=0), (xi + 1) % gx, axis=1)
    grid_br = np.take(np.take(lattice, (yi + 1) % gy, axis=0), (xi + 1) % gx, axis=1)
    fy = _smoothstep(ys - np.floor(ys))[:, None]
    fx = _smoothstep(xs - np.floor(xs))[None, :]
    out = (grid * (1 - fy) * (1 - fx) + grid_b * fy * (1 - fx)
           + grid_r * (1 - fy) * fx + grid_br * fy * fx)
    return out

def fbm(w, h, cells, rng, weight=0.5):
    """Multi-octave noise: cells = [coarse, fine, ...]."""
    out = np.zeros((h, w), np.float32)
    total = 0.0
    for i, c in enumerate(cells):
        out += (weight ** i) * value_noise(w, h, c, rng)
        total += weight ** i
    return out / total

# ------------------------------------------------------------------ palette
class Pal:
    """Spec palette (color facts) with our own seeded perturbation."""
    def __init__(self, pal, pc, name):
        self.raw = [(c[0], c[1], c[2], c[3]) for c in pal]
        self.pc = list(pc)
        rng = rng_for(name, "pal")
        self.entries = []
        for c in self.raw:
            jitter = rng.integers(-2, 3, size=4)
            a = int(np.clip(c[3] + jitter[3] * 0, 0, 255))  # alpha kept exact (functional)
            self.entries.append((
                int(np.clip(c[0] + jitter[0], 0, 255)),
                int(np.clip(c[1] + jitter[1], 0, 255)),
                int(np.clip(c[2] + jitter[2], 0, 255)), a))
        # freq stays RAW (sums < 1 when the spec palette is truncated);
        # Pal.thresholds() redistributes the unrepresented tail correctly.
        self.freq = np.array(self.pc, np.float32)

    def __len__(self):
        return len(self.entries)

    def luma(self, i):
        r, g, b, _ = self.entries[i]
        return 0.299 * r + 0.587 * g + 0.114 * b

    def order_by_luma(self):
        return sorted(range(len(self.entries)), key=self.luma)

    def rgb(self, i):
        return self.entries[i][:3]

    def rgba(self, i):
        return self.entries[i]

    def pick(self, i):
        """i may exceed range -> clamp."""
        return self.entries[min(i, len(self.entries) - 1)]

    def main_alpha(self):
        return self.entries[0][3] if self.entries else 255

    def by_luma_index(self, v):
        """Map noise value [0,1] -> palette index, honoring frequency."""
        order = self.order_by_luma()
        cum = np.cumsum(self.freq[order])
        k = int(np.searchsorted(cum, v * cum[-1]))
        return order[min(k, len(order) - 1)]

    def thresholds(self):
        """Proportion-preserving cumulative thresholds over [0,1].

        The spec palette is truncated (top-N); the unrepresented tail mass
        is redistributed proportionally onto the opaque entries so the
        true pixel fractions survive (e.g. fire: 45% transparent stays
        45%, not 67% of the truncated mass)."""
        order = self.order_by_luma()
        raw = np.asarray(self.freq, np.float64)
        covered = float(raw.sum())
        opaque = np.array([self.entries[i][3] >= 10 for i in range(len(order))])
        eff = raw[order].copy()
        if covered < 0.999 and opaque.any():
            tail = 1.0 - covered
            share = eff * opaque / float((eff * opaque).sum())
            eff = eff + tail * share
        elif covered > 0:
            eff = eff / covered
        cum = np.cumsum(eff)
        if cum[-1] <= 0:
            cum = np.linspace(0, 1, len(order) + 1)[1:]
        return order, cum

    def accent_indices(self, max_pc=0.10):
        return [i for i in range(len(self.entries)) if self.freq[i] < max_pc]

    def extreme(self, high=True):
        order = self.order_by_luma()
        return order[-1] if high else order[0]

    def mid(self):
        order = self.order_by_luma()
        return order[len(order) // 2]

# ------------------------------------------------------------------ canvas
def canvas(w, h):
    return np.zeros((h, w, 4), np.uint8)

def fill(c, color, x=0, y=0, w=None, h=None):
    H, W = c.shape[:2]
    w = W if w is None else w
    h = H if h is None else h
    x2, y2 = min(x + w, W), min(y + h, H)
    if x2 > x and y2 > y:
        c[max(y, 0):y2, max(x, 0):x2] = color

def px(c, x, y, color):
    H, W = c.shape[:2]
    if 0 <= x < W and 0 <= y < H:
        c[y, x] = color

def to_image(c):
    return Image.fromarray(c, "RGBA")

def save(c, path):
    to_image(c).save(path)

# ------------------------------------------------------------------ primitives
def disc(c, cx, cy, r, color, outline=None):
    H, W = c.shape[:2]
    yy, xx = np.mgrid[0:H, 0:W]
    d = (xx - cx) ** 2 + (yy - cy) ** 2
    m = d <= r * r
    if outline is not None:
        mo = (d <= r * r) & (d > (r - 1.2) ** 2)
        c[mo] = outline
    else:
        c[m] = color
    return m

def blob(c, cx, cy, rx, ry, rng, color, wob=0.35):
    """Organic blob: radial polygon with seeded wobble."""
    H, W = c.shape[:2]
    yy, xx = np.mgrid[0:H, 0:W]
    ang = np.arctan2(yy - cy, xx - cx)
    wobn = 1.0 + wob * (np.sin(ang * 3 + rng.uniform(0, 6.3)) * 0.5
                        + np.sin(ang * 5 + rng.uniform(0, 6.3)) * 0.3
                        + np.sin(ang * 2 + rng.uniform(0, 6.3)) * 0.2)
    d = ((xx - cx) / (rx * wobn)) ** 2 + ((yy - cy) / (ry * wobn)) ** 2
    c[d <= 1.0] = color
    return d <= 1.0

def line(c, x0, y0, x1, y1, color, width=1):
    dx, dy = abs(x1 - x0), abs(y1 - y0)
    sx = 1 if x0 < x1 else -1
    sy = 1 if y0 < y1 else -1
    err = dx - dy
    x, y = x0, y0
    while True:
        for ox in range(width):
            for oy in range(width):
                px(c, x + ox, y + oy, color)
        if x == x1 and y == y1:
            break
        e2 = 2 * err
        if e2 > -dy:
            err -= dy; x += sx
        if e2 < dx:
            err += dx; y += sy

def bevel_panel(c, x, y, w, h, border, light, dark, fillc, inset=False):
    """Classic raised/inset bevel — functional widget construction."""
    fill(c, fillc, x, y, w, h)
    fill(c, border, x, y, w, 1)
    fill(c, border, x, y + h - 1, w, 1)
    fill(c, border, x, y, 1, h)
    fill(c, border, x + w - 1, y, 1, h)
    a, b = (light, dark) if not inset else (dark, light)
    fill(c, a, x + 1, y + 1, w - 2, 1)
    fill(c, a, x + 1, y + 1, 1, h - 2)
    fill(c, b, x + 1, y + h - 2, w - 2, 1)
    fill(c, b, x + w - 2, y + 1, 1, h - 2)

def sprite_from_fn(w, h, fn):
    """Draw on a PIL canvas via ImageDraw, return numpy array."""
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    fn(d)
    return np.asarray(im).copy()
