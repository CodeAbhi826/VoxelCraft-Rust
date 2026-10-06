#!/usr/bin/env python3
"""vault_contact_sheet.py — labeled contact sheets of the VAULT's own
output (no reference pixels) for visual QA critique.

Phase 0 restore: this tool was removed alongside the reference-reading
scripts, but that was over-broad — it only ever reads files this project
generated. Its one real defect was a hardcoded `/home/z/...` path outside
the repo. Now repo-relative, so scripts/legal_audit.py's external-path
rule passes.

Usage:
    python3 scripts/vault_contact_sheet.py <out_dir> [group ...]

Groups: mobs, blocks, all. Output is PNG contact sheets, never committed.
"""
import os
import sys
from PIL import Image, ImageDraw

# Repo-relative: the vault's own generated output, nothing outside the tree.
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
V = os.path.join(REPO, "voxelcraft", "assets-vault", "textures")

# The iconic cast the owner asked to review. Paths are the generated ones.
ICONIC = [
    ("creeper", "entity/creeper/creeper.png"),
    ("enderman", "entity/enderman/enderman.png"),
    ("ender dragon", "entity/enderdragon/dragon.png"),
    ("villager", "entity/villager/villager.png"),
    ("zombie villager", "entity/zombie_villager/zombie_villager.png"),
    ("iron golem", "entity/iron_golem/iron_golem.png"),
    ("wither", "entity/wither/wither.png"),
    ("ghast", "entity/ghast/ghast.png"),
    ("blaze", "entity/blaze/blaze.png"),
    ("piglin", "entity/piglin/piglin.png"),
    ("player classic", "entity/player/wide/nova.png"),
    ("player slim", "entity/player/slim/nova.png"),
    ("tnt", "block/tnt_side.png"),
    ("nether portal", "block/nether_portal.png"),
]


def load_tile(rel, tile):
    """First frame, nearest-neighbour fitted into the cell (max 8x upscale)."""
    p = os.path.join(V, rel)
    if not os.path.isfile(p):
        return None
    t = Image.open(p).convert("RGBA")
    w, h = t.size
    if h > 1.5 * w:  # animated strip -> first frame
        t = t.crop((0, 0, w, w))
        h = w
    s = max(1, min(8, (tile - 12) // max(w, h)))
    t = t.resize((w * s, h * s), Image.NEAREST)
    if t.width > tile - 4 or t.height > tile - 4:
        s2 = (tile - 4) / max(t.width, t.height)
        t = t.resize(
            (max(1, int(t.width * s2)), max(1, int(t.height * s2))), Image.NEAREST
        )
    return t


def sheet(items, out_path, cols=5, tile=200):
    rows = (len(items) + cols - 1) // cols
    W, H = cols * tile, rows * tile
    im = Image.new("RGBA", (W, H), (38, 38, 44, 255))
    d = ImageDraw.Draw(im)
    missing = []
    for i, (name, rel) in enumerate(items):
        cx, cy = (i % cols) * tile, (i // cols) * tile
        d.rectangle([cx + 2, cy + 2, cx + tile - 3, cy + tile - 3],
                    outline=(90, 90, 100, 255))
        t = load_tile(rel, tile)
        if t is None:
            missing.append(name)
            d.text((cx + 8, cy + tile // 2), f"MISSING\n{name}", fill=(220, 120, 120, 255))
        else:
            im.alpha_composite(t, (cx + (tile - t.width) // 2, cy + 6))
        d.text((cx + 6, cy + tile - 16), name, fill=(235, 235, 240, 255))
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    im.convert("RGB").save(out_path)
    return missing


def main():
    out_dir = sys.argv[1] if len(sys.argv) > 1 else "/tmp/vc-contact"
    groups = sys.argv[2:] or ["iconic"]
    if "all" in groups or "iconic" in groups:
        miss = sheet(ICONIC, os.path.join(out_dir, "iconic_cast.png"))
        print(f"iconic_cast.png written; missing: {miss or 'none'}")
    if "all" in groups or "blocks" in groups:
        blocks = [
            (f[:-4], f"block/{f}")
            for f in sorted(os.listdir(os.path.join(V, "block")))[:40]
            if f.endswith(".png")
        ]
        sheet(blocks, os.path.join(out_dir, "blocks.png"))
        print("blocks.png written")


if __name__ == "__main__":
    main()