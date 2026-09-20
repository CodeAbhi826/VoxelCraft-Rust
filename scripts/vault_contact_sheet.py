#!/usr/bin/env python3
"""vault_contact_sheet.py — labeled contact sheets of the VAULT's own
output (no reference pixels) for visual QA critique. v2: proper fit."""
import os
import numpy as np
from PIL import Image, ImageDraw

V = "/home/z/my-project/voxelcraft/assets-vault/assets/the reference game/textures"
OUT = "/home/z/my-project/diag/vault"

def load_tile(path, tile):
    """First frame, fitted into the cell (max 8x upscale)."""
    t = Image.open(path).convert("RGBA")
    w, h = t.size
    if h > 1.5 * w:  # animated strip -> first frame
        t = t.crop((0, 0, w, w))
        h = w
    s = max(1, min(8, (tile - 12) // max(w, h)))
    t = t.resize((w * s, h * s), Image.NEAREST)
    if t.width > tile - 4 or t.height > tile - 4:  # safety clamp
        s2 = (tile - 4) / max(t.width, t.height)
        t = t.resize((max(1, int(t.width * s2)), max(1, int(t.height * s2))),
                     Image.NEAREST)
    return t

def sheet(name_list, out_name, cols=8, tile=84):
    os.makedirs(OUT, exist_ok=True)
    rows = (len(name_list) + cols - 1) // cols
    W, H = cols * tile, rows * tile
    im = Image.new("RGBA", (W, H), (38, 38, 44, 255))
    d = ImageDraw.Draw(im)
    for i, n in enumerate(name_list):
        p = os.path.join(V, n)
        if not os.path.exists(p):
            continue
        cx, cy = (i % cols) * tile, (i // cols) * tile
        # checker backdrop inside the cell
        for yy in range(cy, cy + tile - 14, 8):
            for xx in range(cx, cx + tile, 8):
                col = (58, 58, 64, 255) if ((xx - cx) // 8 + (yy - cy) // 8) % 2 else (48, 48, 54, 255)
                d.rectangle([xx, yy, min(xx + 8, cx + tile) - 1, min(yy + 8, cy + tile - 14) - 1], fill=col)
        t = load_tile(p, tile)
        x0 = cx + (tile - t.width) // 2
        y0 = cy + (tile - 14 - t.height) // 2
        im.paste(t, (x0, y0), t)
        label = n.rsplit("/", 1)[-1].replace(".png", "")[:13]
        d.text((cx + 3, cy + tile - 11), label, fill=(205, 205, 215, 255))
    im.convert("RGB").save(os.path.join(OUT, out_name))
    print("wrote", os.path.join(OUT, out_name))

BLOCKS = ["block/stone.png", "block/dirt.png", "block/grass_block_top.png",
          "block/grass_block_side.png", "block/oak_planks.png", "block/spruce_planks.png",
          "block/oak_log.png", "block/oak_log_top.png", "block/coal_ore.png",
          "block/iron_ore.png", "block/diamond_ore.png", "block/deepslate.png",
          "block/stone_bricks.png", "block/bricks.png", "block/hollowstone.png",
          "block/hollow_bricks.png", "block/sand.png", "block/gravel.png",
          "block/obsidian.png", "block/glass.png", "block/snow.png", "block/ice.png",
          "block/glowstone.png", "block/oak_leaves.png", "block/crafting_table_top.png",
          "block/furnace_front.png", "block/water_still.png", "block/lava_still.png",
          "block/red_wool.png", "block/bedrock.png", "block/cobblestone.png",
          "block/hollow_portal.png"]
ITEMS = ["item/diamond_sword.png", "item/iron_pickaxe.png", "item/iron_axe.png",
         "item/iron_shovel.png", "item/iron_hoe.png", "item/diamond_helmet.png",
         "item/diamond_chestplate.png", "item/diamond_leggings.png", "item/diamond_boots.png",
         "item/iron_ingot.png", "item/gold_ingot.png", "item/diamond.png",
         "item/emerald.png", "item/fluxstone.png", "item/coal.png",
         "item/apple.png", "item/bread.png", "item/cookie.png",
         "item/bowl.png", "item/mushroom_stew.png", "item/potion.png",
         "item/bucket.png", "item/water_bucket.png", "item/stick.png",
         "item/book.png", "item/paper.png", "item/string.png",
         "item/arrow.png", "item/wheat.png", "item/carrot.png",
         "item/potato.png", "item/void_pearl.png"]
GUI = ["gui/sprites/widget/button.png", "gui/sprites/widget/button_highlighted.png",
       "gui/sprites/widget/button_disabled.png", "gui/sprites/widget/checkbox.png",
       "gui/sprites/widget/checkbox_selected.png", "gui/sprites/widget/slider.png",
       "gui/sprites/widget/slider_handle.png", "gui/sprites/widget/scroller.png",
       "gui/sprites/widget/text_field.png", "gui/sprites/widget/tab.png",
       "gui/sprites/widget/tab_selected.png", "gui/sprites/widget/page_forward.png",
       "gui/sprites/hud/heart/full.png", "gui/sprites/hud/heart/container.png",
       "gui/sprites/hud/heart/half.png", "gui/sprites/hud/heart/poisoned_full.png",
       "gui/sprites/hud/food_full.png", "gui/sprites/hud/food_empty.png",
       "gui/sprites/hud/air.png", "gui/sprites/hud/armor_full.png",
       "gui/sprites/hud/crosshair.png", "gui/sprites/hud/hotbar.png",
       "gui/sprites/hud/experience_bar_background.png",
       "gui/sprites/hud/experience_bar_progress.png",
       "gui/sprites/boss_bar/red_background.png", "gui/sprites/boss_bar/red_progress.png",
       "gui/container/inventory.png", "gui/container/crafting_table.png",
       "gui/container/furnace.png", "gui/container/hopper.png",
       "gui/container/anvil.png", "gui/menu_background.png"]
ENT = ["entity/fuseling/fuseling.png", "entity/zombie/zombie.png",
       "entity/skeleton/skeleton.png", "entity/spider/spider.png",
       "entity/voidling/voidling.png", "entity/pig/pig.png",
       "entity/cow/cow.png", "entity/sheep/sheep.png",
       "entity/chicken.png", "entity/slime/slime.png",
       "entity/weepgeist/weepgeist.png", "entity/villager/villager.png",
       "entity/wolf/wolf.png", "entity/cat/all_black.png",
       "entity/iron_golem.png", "entity/snow_golem.png",
       "entity/bat.png", "entity/squid/squid.png",
       "entity/bee/bee.png", "entity/fox/fox.png"]
SPECIAL = ["environment/celestial/sun.png", "environment/celestial/moon/full_moon.png",
           "environment/celestial/moon/new_moon.png", "environment/clouds.png",
           "environment/end_sky.png", "environment/rain.png",
           "colormap/grass.png", "colormap/foliage.png",
           "font/ascii.png", "font/nonlatin_european.png",
           "map/map_background.png", "map/decorations/player.png",
           "map/decorations/red_x.png", "map/decorations/white_banner.png",
           "misc/shadow.png", "misc/vignette.png",
           "misc/enchanted_glint_item.png", "misc/pumpkinblur.png",
           "painting/kebab.png", "painting/alban.png",
           "painting/wanderer.png", "painting/sea.png",
           "mob_effect/speed.png", "mob_effect/strength.png",
           "particle/generic_0.png", "particle/big_smoke_0.png",
           "trims/items/leggings_trim.png", "trims/color_palettes/gold.png"]

if __name__ == "__main__":
    sheet(BLOCKS, "vault_blocks.png", cols=8, tile=84)
    sheet(ITEMS, "vault_items.png", cols=8, tile=84)
    sheet(GUI, "vault_gui.png", cols=4, tile=180)
    sheet(ENT, "vault_entities.png", cols=5, tile=160)
    sheet(SPECIAL, "vault_special.png", cols=6, tile=120)
