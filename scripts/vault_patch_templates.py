#!/usr/bin/env python3
"""vault_patch_templates.py — replace tool/armor template function bodies
by function-boundary regex (robust to whitespace)."""
import re

P = "/home/z/my-project/scripts/vault_synth_sprites.py"
src = open(P).read()

TEMPLATES = {}

TEMPLATES["tpl_sword"] = '''def tpl_sword(w, h, pal, rng):
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
'''

TEMPLATES["tpl_pickaxe"] = '''def tpl_pickaxe(w, h, pal, rng):
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
'''

TEMPLATES["tpl_axe"] = '''def tpl_axe(w, h, pal, rng):
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
'''

TEMPLATES["tpl_shovel"] = '''def tpl_shovel(w, h, pal, rng):
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
'''

TEMPLATES["tpl_hoe"] = '''def tpl_hoe(w, h, pal, rng):
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
'''

TEMPLATES["tpl_helmet"] = '''def tpl_helmet(w, h, pal, rng):
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
'''

TEMPLATES["tpl_chest"] = '''def tpl_chest(w, h, pal, rng):
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
'''

TEMPLATES["tpl_leggings"] = '''def tpl_leggings(w, h, pal, rng):
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
'''

TEMPLATES["tpl_boots"] = '''def tpl_boots(w, h, pal, rng):
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
'''

for fn_name, body in TEMPLATES.items():
    pattern = re.compile(rf"def {fn_name}\(.*?\n(?=(?:def |# ---|TOOL_ROUTES))", re.S)
    m = pattern.search(src)
    assert m, f"function not found: {fn_name}"
    src = pattern.sub(body, src, count=1)
    print("replaced", fn_name)

open(P, "w").write(src)
print("all templates upgraded")
