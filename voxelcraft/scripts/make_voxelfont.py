#!/usr/bin/env python3
"""make_voxelfont.py — generate Voxelfont.ttf from the project's own
clean-room 5x8 bitmap glyph table (vc-render/src/ui.rs `FONT`).

Every glyph pixel in the output font comes from the in-project clean-room
bitmap set (classic 5x7-style industry shapes for ASCII uppercase/digits +
in-project designed lowercase, rows documented in ui.rs). The generator
wraps those pixels into a TrueType outline font (square outlines, one
contour per pixel) so the runtime engine (ab_glyph) can rasterize them at
device resolution.

100% original provenance: no third-party font data is read, bundled or
transformed anywhere in this pipeline. Output license: MIT (see
crates/vc-render/assets/VOXELFONT-NOTICE.txt).

Usage: python3 scripts/make_voxelfont.py   (from the voxelcraft/ root)
"""
import re
import sys

UI = "crates/vc-render/src/ui.rs"
OUT = "crates/vc-render/assets/Voxelfont.ttf"

# Extra symbols beyond ASCII — original in-project 5x8 designs
# (rows top->bottom, bit 4 = leftmost pixel, same grid as FONT).
EXTRAS = {
    0x221E: [0x00, 0x00, 0x15, 0x0E, 0x15, 0x00, 0x00, 0x00],  # infinity
    0x00D7: [0x00, 0x00, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x00],  # multiply
    0x2022: [0x00, 0x00, 0x00, 0x0E, 0x1F, 0x0E, 0x00, 0x00],  # bullet
    0x00B0: [0x0E, 0x11, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00],  # degree
    0x2192: [0x00, 0x00, 0x04, 0x02, 0x1F, 0x02, 0x04, 0x00],  # right arrow
    0x2190: [0x00, 0x00, 0x04, 0x08, 0x1F, 0x08, 0x04, 0x00],  # left arrow
    0x2191: [0x04, 0x0E, 0x15, 0x04, 0x04, 0x04, 0x00, 0x00],  # up arrow
    0x2193: [0x00, 0x00, 0x04, 0x04, 0x04, 0x15, 0x0E, 0x04],  # down arrow
    0x00B7: [0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00],  # middle dot
    0x2026: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x00],  # ellipsis
    0x2014: [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x00],  # em dash
    0x2013: [0x00, 0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],  # en dash
}

# ---------------------------------------------------------------- read --

def load_font_table():
    src = open(UI, encoding="utf-8").read()
    m = re.search(r"const FONT: \[\[u8; 8\]; 96\] = \[\n(.*?)\n\];", src, re.S)
    if not m:
        sys.exit("FONT table not found in ui.rs")
    glyphs = re.findall(r"\[([0-9xXa-fA-F,]+)\]", m.group(1))
    if len(glyphs) != 96:
        sys.exit(f"expected 96 glyphs, got {len(glyphs)}")
    table = []
    for s in glyphs:
        rows = [int(x, 16) for x in s.split(",")]
        if len(rows) != 8:
            sys.exit(f"expected 8 rows, got {len(rows)}")
        table.append(rows)
    return table

# ------------------------------------------------------------- build ---

def build_ttf(table):
    from fontTools.fontBuilder import FontBuilder
    from fontTools.pens.ttGlyphPen import TTGlyphPen
    from fontTools.pens.boundsPen import BoundsPen

    UPEM = 1024          # units per em
    P = 128              # one bitmap pixel = 128 units -> 8 rows = 1024
    BASE_ROW = 6         # bottom of row 6 sits on the baseline (y=0)

    # glyph order: .notdef + 96 ASCII slots (32..127) + symbol extras
    names = ([".notdef"] + [f"g{c}" for c in range(32, 128)]
             + [f"uni{cp:04X}" for cp in sorted(EXTRAS)])
    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(names)
    cmap = {c: f"g{c}" for c in range(32, 128)}
    cmap.update({cp: f"uni{cp:04X}" for cp in EXTRAS})
    fb.setupCharacterMap(cmap)

    # rows -> bit outlines. row r (0 = top) spans y in
    # [(BASE_ROW - r) * P, (BASE_ROW - r + 1) * P); rows 0..6 sit above
    # the baseline (cap height = 7 px), row 7 dips below it (-P..0).
    def glyph_pen(rows, pen):
        for r, bits in enumerate(rows):
            y0 = (BASE_ROW - r) * P
            y1 = y0 + P
            for col in range(5):
                if bits & (1 << (4 - col)):
                    x0 = col * P
                    x1 = x0 + P
                    # clockwise (TrueType convention, y-up space)
                    pen.moveTo((x0, y0))
                    pen.lineTo((x0, y1))
                    pen.lineTo((x1, y1))
                    pen.lineTo((x1, y0))
                    pen.closePath()

    glyphs = {}
    advances = {}
    for idx, rows in enumerate(table):
        name = f"g{idx + 32}"
        pen = TTGlyphPen(None)
        glyph_pen(rows, pen)
        glyphs[name] = pen.glyph()
        # ink width in pixels (rightmost on-column), min 1 for advance
        ink = 0
        for bits in rows:
            ink = max(ink, bits.bit_length())
        adv = (ink + 1) * P if ink > 0 else 2 * P  # space: 2px advance
        advances[name] = adv

    # .notdef: a single 3x5 nether box (our own placeholder mark)
    pen = TTGlyphPen(None)
    pen.moveTo((0, 0)); pen.lineTo((0, 5 * P)); pen.lineTo((3 * P, 5 * P))
    pen.lineTo((3 * P, 0)); pen.closePath()
    pen.moveTo((P, P)); pen.lineTo((2 * P, P)); pen.lineTo((2 * P, 4 * P))
    pen.lineTo((P, 4 * P)); pen.closePath()
    glyphs[".notdef"] = pen.glyph()
    advances[".notdef"] = 4 * P

    # extra symbols (original in-project designs)
    for cp, rows in EXTRAS.items():
        name = f"uni{cp:04X}"
        pen = TTGlyphPen(None)
        glyph_pen(rows, pen)
        glyphs[name] = pen.glyph()
        ink = max(bits.bit_length() for bits in rows)
        advances[name] = (ink + 1) * P if ink > 0 else 2 * P

    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics({n: (advances[n], 0) for n in names})
    # bounds for head/hhea — computed straight from the pixel grid
    all_rows = list(table) + list(EXTRAS.values())
    xmin = ymin = xmax = ymax = None
    for rows in all_rows:
        for r, bits in enumerate(rows):
            if not bits:
                continue
            x1 = bits.bit_length() * P
            y0 = (BASE_ROW - r) * P
            xmin = 0 if xmin is None else min(xmin, 0)
            xmax = x1 if xmax is None else max(xmax, x1)
            ymin = y0 if ymin is None else min(ymin, y0)
            ymax = y0 + P if ymax is None else max(ymax, y0 + P)
    fb.setupHorizontalHeader(
        ascent=7 * P, descent=-P, lineGap=2 * P,
        xMin=xmin or 0, yMin=ymin or 0, xMax=xmax or 0, yMax=ymax or 0)
    fb.setupOS2(
        sTypoAscender=7 * P, sTypoDescender=-P, sTypoLineGap=2 * P,
        usWinAscent=7 * P, usWinDescent=P,
        usWeightClass=400, usWidthClass=5, fsSelection=0x40)
    # name/notice: original in-project design, MIT
    fb.setupNameTable({
        "familyName": "Voxelfont",
        "styleName": "Regular",
        "uniqueFontIdentifier": "Voxelfont Regular 1.0",
        "fullName": "Voxelfont Regular",
        "version": "Version 1.0",
        "copyright": "Original clean-room glyph designs by the VoxelCraft "
                     "project. MIT license.",
        "designer": "The VoxelCraft Project",
        "description": "Pixel font generated from the project's own "
                       "clean-room 5x8 bitmap glyph set.",
        "licenseDescription": "MIT license, see VOXELFONT-NOTICE.txt",
        "licenseInfoURL": "https://opensource.org/licenses/MIT",
    })
    fb.setupPost()
    fb.setupMaxp()
    fb.save(OUT)
    print(f"wrote {OUT}: {len(names)} glyphs, upem={UPEM}, pixel={P}u")


if __name__ == "__main__":
    build_ttf(load_font_table())
