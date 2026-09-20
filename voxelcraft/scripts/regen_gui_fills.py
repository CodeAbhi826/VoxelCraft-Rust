#!/usr/bin/env python3
"""regen_gui_fills.py — regenerate the flat GUI dim-overlay textures.

The three menu-dim overlays are functional UI elements (a plain
semi-transparent black fill used to dim backgrounds behind menus).
This script generates them from scratch in-project so every file in
the asset vault has clean provenance: a flat fill carries no creative
expression, and these bytes come from THIS generator alone.

Run from voxelcraft/: python3 scripts/regen_gui_fills.py
"""
from PIL import Image

FILLS = {
    # (path, alpha): a flat black fill at the given opacity — the
    # functional "dim the world behind the menu" overlay
    "assets-vault/textures/gui/menu_background.png": 64,
    "assets-vault/textures/gui/inworld_menu_background.png": 64,
    "assets-vault/textures/gui/inworld_menu_list_background.png": 112,
}

for path, alpha in FILLS.items():
    img = Image.new("RGBA", (16, 16), (0, 0, 0, alpha))
    img.save(path, optimize=True)
    print(f"regenerated {path} (flat black, alpha {alpha})")
