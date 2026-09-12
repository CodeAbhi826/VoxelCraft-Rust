# Resource Packs

Drop vanilla-format resource packs here to override the built-in GUI
art. Both plain folders and `.zip` archives work.

## What can be overridden

| Logical name          | File in your pack                              | Size     |
|-----------------------|------------------------------------------------|----------|
| `gui/hearts`          | `assets/minecraft/textures/gui/hearts.png`     | 27 x 9   |
| `gui/hunger`          | `assets/minecraft/textures/gui/hunger.png`     | 27 x 9   |
| `gui/armor`           | `assets/minecraft/textures/gui/armor.png`      | 27 x 9   |
| `gui/bubbles`         | `assets/minecraft/textures/gui/bubbles.png`    | 18 x 9   |
| `gui/widgets`         | `assets/minecraft/textures/gui/widgets.png`    | 120 x 20 |
| `gui/hotbar`          | `assets/minecraft/textures/gui/hotbar.png`     | 182 x 22 |
| `gui/hotbar_sel`      | `assets/minecraft/textures/gui/hotbar_sel.png` | 24 x 22  |
| `gui/options_background` | `assets/minecraft/textures/gui/options_background.png` | 16 x 16 |
| `gui/font`            | `assets/minecraft/textures/gui/font.png`       | 128 x 48 |

Each sheet is a horizontal strip of tiles (hearts: Empty / Full / Half;
widgets: ButtonNormal, ButtonHover, ButtonDisabled, SlotEmpty,
SlotHover, Panel). A pack that provides only `hearts.png` overrides
hearts and nothing else.

## pack.mcmeta

```json
{
  "pack": {
    "pack_format": 6,
    "description": "My GUI pack"
  }
}
```

`pack_format` **6** is the 1.16.2-1.16.5 resource-pack format. A
mismatched number logs a warning but the pack still loads (the engine
deliberately never rejects an imperfect pack).

## Priority

Packs apply alphabetically, later names winning — the same
"top of the list wins" behavior as vanilla. The built-in procedural art
always sits below every user pack and fills in anything a pack does not
provide.

Restart the game after changing packs (no hot reload yet — tracked as
UI-overhaul backlog).
