#!/usr/bin/env python3
"""vault_synthesize.py — main entry: generate the FULL clean-room asset
vault from spec.json.

Usage: python3 scripts/vault_synthesize.py [--only NAME_SUBSTR]

Output tree (NOT referenced by the engine — a library for future
versions, per the owner's directive):
  voxelcraft/assets-vault/assets/minecraft/textures/**.png (+ .mcmeta)

Clean-room chain: spec (measurements) + procedural rules -> pixels.
The reference zip is never read here.
"""
import json, os, sys, time
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import vault_synth_blocks as B
import vault_synth_sprites2 as S2
import vault_synth_gui as G
import vault_synth_entity as E
import vault_synth_special as X
from voxel_synth_shim import Pal, canvas, fill, disc, rng_for, fbm

SPEC = "/home/z/my-project/voxelcraft/assets-vault/spec/spec.json"
OUT_ROOT = "/home/z/my-project/voxelcraft/assets-vault/assets/minecraft/textures"

# ------------------------------------------------------------------ font pages
def synth_font_page(rec, name):
    base = name.rsplit("/", 1)[-1].replace(".png", "")
    if "sga" in base or "illager" in base:
        return X.gen_fictional_page(rec, name)
    if base == "ascii":
        return X.gen_glyph_page(rec, name, base_cp=0x00)
    if base == "accented":
        return X.gen_glyph_page(rec, name, base_cp=0x00C0)
    if base == "nonlatin_european":
        return X.gen_glyph_page(rec, name, base_cp=0x0100)
    return X.gen_glyph_page(rec, name, base_cp=0x00)

# ------------------------------------------------------------------ environment
def synth_environment(rec, name):
    base = name.lower()
    if "moon" in base or "sun" in base or "celestial" in base:
        return X.gen_celestial(rec, name)
    if "cloud" in base:
        return X.gen_clouds(rec, name)
    if "end_sky" in base or "sky" in base:
        return X.gen_starfield(rec, name)
    if "rain" in base or "snow" in base:
        return X.gen_weather(rec, name)
    return X.gen_starfield(rec, name)

# ------------------------------------------------------------------ gui router
def synth_gui(rec, name):
    base = name.lower()
    n = name.rsplit("/", 1)[-1].replace(".png", "")
    rel = name.replace("gui/", "").replace(".png", "")
    if rel.startswith("sprites/widget/") or rel.startswith("sprites/container/") \
       or rel.startswith("sprites/recipe_book/") or rel.startswith("sprites/toast/") \
       or rel.startswith("sprites/gamemode_switcher/") or rel.startswith("world_list") \
       or rel.startswith("server_list") or rel.startswith("friends") \
       or rel.startswith("transferable_list") or rel.startswith("social_interactions") \
       or rel.startswith("pause_menu") or rel.startswith("dialog") or rel.startswith("popup"):
        # widget-family routing by leaf name
        leaf = n
        if "button" in leaf:
            state = ("disabled" if "disabled" in name else
                     "highlighted" if "highlighted" in name else "normal")
            if "locked" in leaf or "unlocked" in leaf:
                return G.gen_lock_button(rec, name)
            if "cross_button" in leaf:
                return G.gen_cross_button(rec, name)
            return G.gen_button(rec, name, state=state)
        if "checkbox" in leaf:
            state = "highlighted" if "highlighted" in name else "normal"
            return G.gen_checkbox(rec, name, state=state)
        if "slider_handle" in leaf:
            state = "highlighted" if "highlighted" in name else "normal"
            return G.gen_slider_handle(rec, name, state=state)
        if "slider" in leaf:
            state = "highlighted" if "highlighted" in name else "normal"
            return G.gen_slider_track(rec, name, state=state)
        if "scroller" in leaf:
            return G.gen_scroller(rec, name, bg="background" in leaf)
        if "text_field" in leaf or "preedit" in leaf:
            return G.gen_text_field(rec, name)
        if "tab" in leaf:
            return G.gen_tab(rec, name)
        if "slot" in leaf and "frame" in leaf:
            return G.gen_slot_frame(rec, name)
        if "page_forward" in leaf or "page_backward" in leaf:
            return G.gen_page_arrow(rec, name)
        # progress bars / bubbles / misc container sprites
        return G.gen_generic_panel_sprite(rec, name)
    if rel.startswith("sprites/hud/"):
        leaf = n
        if "heart" in leaf:
            return G.gen_heart(rec, name)
        if "food" in leaf:
            return G.gen_food_icon(rec, name)
        if "air" in leaf or "bubble" in leaf:
            return G.gen_air_icon(rec, name)
        if "armor" in leaf:
            return G.gen_armor_icon(rec, name)
        if "crosshair" in leaf:
            if "background" in leaf or "progress" in leaf:
                return G.gen_bar(rec, name)
            return G.gen_crosshair(rec, name)
        if "experience" in leaf or "xp" in leaf:
            return G.gen_bar(rec, name)
        if "hotbar" in leaf:
            if "attack_indicator" in leaf and ("background" in leaf or "progress" in leaf):
                return G.gen_bar(rec, name)
            return G.gen_hotbar(rec, name)
        if "effect_background" in leaf:
            return G.gen_effect_bg(rec, name)
        return G.gen_generic_panel_sprite(rec, name)
    if rel.startswith("sprites/boss_bar/"):
        return G.gen_bar(rec, name)
    if rel.startswith("sprites/icon/"):
        return G.gen_generic_panel_sprite(rec, name)
    if rel.startswith("sprites/advancements/"):
        return G.gen_menu_bg(rec, name)
    if rel.startswith("sprites/spectator/") or rel.startswith("sprites/statistics/") \
       or rel.startswith("sprites/notification/") or rel.startswith("sprites/pending_invite") \
       or rel.startswith("sprites/realm_status") or rel.startswith("sprites/player_list"):
        return G.gen_generic_panel_sprite(rec, name)
    if rel.startswith("container/") and rec["w"] >= 128:
        return G.gen_container_panel(rec, name)
    if "menu_background" in base or "list_background" in base:
        return G.gen_menu_bg(rec, name)
    if "separator" in base:
        return G.gen_separator(rec, name)
    if rel.startswith("signs/") or rel.startswith("hanging_signs/"):
        return G.gen_sign_board(rec, name)
    if rel.startswith("realms/") or rel.startswith("presets/"):
        return G.gen_illustration(rec, name)
    if "title" in rel:
        # Title-screen art: our OWN voxel emblem — never a wordmark imitation
        # (trademark boundary, LEGAL-COMPLIANCE.md §1.3).
        return G.gen_illustration(rec, name)
    if "advancements/backgrounds" in rel:
        return G.gen_menu_bg(rec, name)
    # book.png, recipe_book.png and remaining panels
    if rec["w"] >= 128:
        return G.gen_container_panel(rec, name)
    return G.gen_generic_panel_sprite(rec, name)

# ------------------------------------------------------------------ main
def main():
    only = None
    if "--only" in sys.argv:
        only = sys.argv[sys.argv.index("--only") + 1]
    spec = json.load(open(SPEC))
    os.makedirs(OUT_ROOT, exist_ok=True)
    t0 = time.time()
    counts = {}
    errors = []
    for rec in spec["textures"]:
        name = rec["n"]
        if only and only not in name:
            continue
        out_path = os.path.join(OUT_ROOT, name)
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        try:
            cat = rec["c"]
            if cat == "font":
                arr = synth_font_page(rec, name)
            elif cat == "colormap":
                arr = X.gen_colormap(rec, name)
            elif cat == "painting":
                arr = X.gen_painting(rec, name)
            elif cat == "environment":
                arr = synth_environment(rec, name)
            elif cat == "entity":
                arr = E.synthesize_entity(rec, name)
            elif cat == "trims":
                if "color_palette" in name:
                    arr = B.gen_flat(rec, name)
                elif "/entity/" in name:
                    arr = E.gen_equipment_layer(rec, name, Pal(rec["pal"], rec["pc"], name), np.random.default_rng(7))
                else:
                    arr = X.gen_trim(rec, name)
            elif cat == "gui":
                arr = synth_gui(rec, name)
            elif cat == "map":
                if "background" in name:
                    arr = X.gen_map_bg(rec, name)
                else:
                    arr = X.gen_map_decoration(rec, name)
            elif cat == "misc":
                arr = X.gen_misc(rec, name)
            elif cat == "mob_effect":
                arr = X.gen_effect_icon(rec, name)
            elif cat == "effect":
                arr = X.gen_effect_icon(rec, name)
            elif cat == "block":
                arr = B.synthesize_block(rec, name)
                if arr is None:
                    if "_sign" in name or "_fence" in name:
                        arr = G.gen_sign_board(rec, name)
                    else:
                        arr = S2.synthesize_sprite(rec, name)
            else:  # item / particle sprites
                arr = S2.synthesize_sprite(rec, name)
            assert arr.shape[:2] == (rec["h"], rec["w"]), \
                f"size mismatch {arr.shape} vs {rec['w']}x{rec['h']}"
            from PIL import Image
            Image.fromarray(arr, "RGBA").save(out_path)
            counts[cat] = counts.get(cat, 0) + 1
        except Exception as e:
            errors.append((name, repr(e)))
    # mcmeta re-emission (functional JSON values, our formatting)
    mc_n = 0
    for m in spec.get("mcmeta", []):
        if only and only not in m["n"]:
            continue
        p = os.path.join(OUT_ROOT, m["n"])
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w") as f:
            f.write(json.dumps(m["j"], separators=(", ", ": ")))
        mc_n += 1
    dt = time.time() - t0
    print(f"generated {sum(counts.values())} PNGs + {mc_n} mcmeta in {dt:.1f}s")
    for k in sorted(counts):
        print(f"  {k:12s} {counts[k]}")
    if errors:
        print(f"\n{len(errors)} ERRORS:")
        for n, e in errors[:40]:
            print(f"  {n}: {e}")
        with open("/home/z/my-project/voxelcraft/assets-vault/spec/synth_errors.txt", "w") as f:
            for n, e in errors:
                f.write(f"{n}: {e}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
