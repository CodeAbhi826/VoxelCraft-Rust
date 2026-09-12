//! `gui/set.rs` — the GUI texture set (UI-overhaul Phase 1, D2).
//!
//! One [`GuiTextureSet`] holds every sprite sheet the quad renderer
//! needs: HUD icons (hearts/hunger/armor/bubbles), widget chrome,
//! hotbar chrome, the options dirt tile, and the active font source.
//! The builtin set is painted procedurally at boot by
//! [`GuiTextureSet::build_builtin`] (G9 — in-code art, never disk); a
//! user resource pack can replace any sheet later (Phase 4).
//!
//! All sheets are RGBA8, tightly packed, tiles laid out horizontally.
//! Sprite art is painted at true texture size (9x9, 18x18, 20x20,
//! 182x22, 16x16) — the quad renderer scales by the engine's 2x UI
//! factor at draw time.

use crate::textures::gui_art::{
    draw_armor, draw_bubble, draw_heart, draw_hotbar_bg, draw_hotbar_sel, draw_hunger,
    draw_options_dirt, draw_widget, ArmorVariant, BubbleVariant, HeartVariant, HungerVariant,
    WidgetVariant,
};

/// One sprite sheet: RGBA pixels + dimensions + tile grid.
#[derive(Clone, Debug)]
pub struct SpriteSheet {
    /// tightly-packed RGBA8 pixels (`w * h * 4` bytes)
    pub px: Vec<u8>,
    pub w: usize,
    pub h: usize,
    pub tile_w: usize,
    pub tile_h: usize,
}

impl SpriteSheet {
    /// tile count across the horizontal strip
    pub fn tile_count(&self) -> usize {
        if self.tile_w == 0 {
            return 0;
        }
        self.w / self.tile_w
    }

    /// normalized (u0, v0, uw, vh) of tile `i` (None when out of range)
    pub fn tile_uv(&self, i: usize) -> Option<(f32, f32, f32, f32)> {
        if i >= self.tile_count() {
            return None;
        }
        let x0 = (i * self.tile_w) as f32;
        Some((
            x0 / self.w as f32,
            0.0,
            self.tile_w as f32 / self.w as f32,
            1.0,
        ))
    }
}

/// Where the UI font's glyphs come from. `BuiltinArray` wraps
/// `ui.rs::FONT` unchanged; `Png` is a decoded 128x48 RGBA glyph sheet
/// (16x6 grid of 8x8 cells, 5x7 ink) supplied by a resource pack.
#[derive(Clone, Debug)]
pub enum FontSource {
    BuiltinArray,
    /// decoded RGBA bytes, 128x48, row stride 128
    Png(Vec<u8>),
}

impl FontSource {
    /// decode the PNG sheet into the engine's `[[u8; 8]; 96]` glyph
    /// form (Phase 5 D5 — the text renderer reads the ACTIVE source):
    /// glyph i lives in the 8x8 cell at (col*8, row*8) with
    /// col = i % 16, row = i / 16; ink = alpha >= 128; cell columns
    /// 0..4 map to the engine's 5-px ink field (the renderer only
    /// draws five columns — the builtin's 5x7-in-8 design). Returns
    /// None for BuiltinArray (callers keep `ui::FONT`).
    pub fn png_glyphs(&self) -> Option<Box<[[u8; 8]; 96]>> {
        let FontSource::Png(px) = self else {
            return None;
        };
        if px.len() != 128 * 48 * 4 {
            return None;
        }
        let mut glyphs = Box::new([[0u8; 8]; 96]);
        for (i, g) in glyphs.iter_mut().enumerate() {
            let col = (i % 16) * 8;
            let row = (i / 16) * 8;
            for (gy, grow) in g.iter_mut().enumerate() {
                for gx in 0..5usize {
                    let idx = (row + gy) * 128 + col + gx;
                    if px.get(idx..idx + 4).map(|p| p[3] >= 128).unwrap_or(false) {
                        *grow |= 1 << (4 - gx);
                    }
                }
            }
        }
        Some(glyphs)
    }
}

/// Typed, non-panicking GUI texture errors (loader + pack override).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiTextureError {
    Missing(&'static str),
    WrongDimensions {
        file: &'static str,
        expected: (usize, usize),
        got: (usize, usize),
    },
    Decode {
        file: &'static str,
    },
    Io {
        file: &'static str,
    },
}

/// The complete GUI texture set, built once at boot and replaced (in
/// whole or per-sheet) when a resource pack provides overrides.
#[derive(Clone, Debug)]
pub struct GuiTextureSet {
    /// 3 tiles of 9x9 (Empty, Full, Half) — 27x9
    pub hearts: SpriteSheet,
    /// 3 tiles of 9x9 (Empty, Full, Half) — 27x9
    pub hunger: SpriteSheet,
    /// 3 tiles of 9x9 (Empty, Full, Half) — 27x9
    pub armor: SpriteSheet,
    /// 2 tiles of 9x9 (Full, Gone) — 18x9
    pub bubbles: SpriteSheet,
    /// 6 cells of 20x20 (ButtonNormal, ButtonHover, ButtonDisabled,
    /// SlotEmpty, SlotHover, Panel) — 120x20. Slots paint centered
    /// 18x18 in their cell (the vanilla 1-px buffer).
    pub widgets: SpriteSheet,
    /// hotbar background — single 182x22 tile
    pub hotbar_bg: SpriteSheet,
    /// hotbar selection frame — single 24x22 tile
    pub hotbar_sel: SpriteSheet,
    /// options-screen dirt tile — single 16x16 tile (0.25 brightness)
    pub dirt: SpriteSheet,
    pub font: FontSource,
}

/// expected pixel dimensions per logical GUI texture name (shared by
/// the builtin builder, the Phase 1 filesystem loader and the Phase 4
/// pack resolver — one table, no drift)
pub(crate) const SHEET_DIMS: &[(&str, usize, usize)] = &[
    ("hearts", 27, 9),
    ("hunger", 27, 9),
    ("armor", 27, 9),
    ("bubbles", 18, 9),
    ("widgets", 120, 20),
    ("hotbar", 182, 22),
    ("hotbar_sel", 24, 22),
    ("options_background", 16, 16),
    ("font", 128, 48),
];

/// copy a 9x9 tile's rows into a horizontal strip at tile index `i`
fn put_tile_9(strip: &mut [u8], i: usize, tile: &[u8]) {
    let stride = 4 * 9 * 3; // 27 px per row across the strip
    let off = i * 9 * 4;
    for row in 0..9usize {
        let dst = row * stride + off;
        let src = row * 9 * 4;
        if dst + 9 * 4 <= strip.len() && src + 9 * 4 <= tile.len() {
            strip[dst..dst + 9 * 4].copy_from_slice(&tile[src..src + 9 * 4]);
        }
    }
}

impl GuiTextureSet {
    /// Paint every builtin sprite (the default + the fallback — never
    /// fails, never touches disk).
    pub fn build_builtin() -> Self {
        let mut hearts_px = vec![0u8; 27 * 9 * 4];
        for (i, variant) in [HeartVariant::Empty, HeartVariant::Full, HeartVariant::Half]
            .into_iter()
            .enumerate()
        {
            let mut tile = [0u8; 9 * 9 * 4];
            draw_heart(&mut tile, 9, variant);
            put_tile_9(&mut hearts_px, i, &tile);
        }
        let hearts = SpriteSheet {
            px: hearts_px,
            w: 27,
            h: 9,
            tile_w: 9,
            tile_h: 9,
        };

        let mut hunger_px = vec![0u8; 27 * 9 * 4];
        for (i, variant) in [HungerVariant::Empty, HungerVariant::Full, HungerVariant::Half]
            .into_iter()
            .enumerate()
        {
            let mut tile = [0u8; 9 * 9 * 4];
            draw_hunger(&mut tile, 9, variant);
            put_tile_9(&mut hunger_px, i, &tile);
        }
        let hunger = SpriteSheet {
            px: hunger_px,
            w: 27,
            h: 9,
            tile_w: 9,
            tile_h: 9,
        };

        let mut armor_px = vec![0u8; 27 * 9 * 4];
        for (i, variant) in [ArmorVariant::Empty, ArmorVariant::Full, ArmorVariant::Half]
            .into_iter()
            .enumerate()
        {
            let mut tile = [0u8; 9 * 9 * 4];
            draw_armor(&mut tile, 9, variant);
            put_tile_9(&mut armor_px, i, &tile);
        }
        let armor = SpriteSheet {
            px: armor_px,
            w: 27,
            h: 9,
            tile_w: 9,
            tile_h: 9,
        };

        let mut bubbles_px = vec![0u8; 18 * 9 * 4];
        for (i, variant) in [BubbleVariant::Full, BubbleVariant::Gone].into_iter().enumerate() {
            let mut tile = [0u8; 9 * 9 * 4];
            draw_bubble(&mut tile, 9, variant);
            put_tile_9(&mut bubbles_px, i, &tile);
        }
        let bubbles = SpriteSheet {
            px: bubbles_px,
            w: 18,
            h: 9,
            tile_w: 9,
            tile_h: 9,
        };

        // widgets: 6 cells of 20x20, laid out horizontally
        let variants = [
            WidgetVariant::ButtonNormal,
            WidgetVariant::ButtonHover,
            WidgetVariant::ButtonDisabled,
            WidgetVariant::SlotEmpty,
            WidgetVariant::SlotHover,
            WidgetVariant::Panel,
        ];
        let mut widgets_px = vec![0u8; 120 * 20 * 4];
        for (i, variant) in variants.into_iter().enumerate() {
            let mut cell = [0u8; 20 * 20 * 4];
            draw_widget(&mut cell, 20, variant);
            let off = i * 20 * 4;
            for row in 0..20usize {
                let dst = row * 120 * 4 + off;
                let src = row * 20 * 4;
                if dst + 20 * 4 <= widgets_px.len() {
                    widgets_px[dst..dst + 20 * 4].copy_from_slice(&cell[src..src + 20 * 4]);
                }
            }
        }
        let widgets = SpriteSheet {
            px: widgets_px,
            w: 120,
            h: 20,
            tile_w: 20,
            tile_h: 20,
        };

        let mut hotbar_px = vec![0u8; 182 * 22 * 4];
        draw_hotbar_bg(&mut hotbar_px, 182);
        let hotbar_bg = SpriteSheet {
            px: hotbar_px,
            w: 182,
            h: 22,
            tile_w: 182,
            tile_h: 22,
        };

        let mut sel_px = vec![0u8; 24 * 22 * 4];
        draw_hotbar_sel(&mut sel_px, 24);
        let hotbar_sel = SpriteSheet {
            px: sel_px,
            w: 24,
            h: 22,
            tile_w: 24,
            tile_h: 22,
        };

        let mut dirt_px = vec![0u8; 16 * 16 * 4];
        draw_options_dirt(&mut dirt_px, 16);
        let dirt = SpriteSheet {
            px: dirt_px,
            w: 16,
            h: 16,
            tile_w: 16,
            tile_h: 16,
        };

        GuiTextureSet {
            hearts,
            hunger,
            armor,
            bubbles,
            widgets,
            hotbar_bg,
            hotbar_sel,
            dirt,
            font: FontSource::BuiltinArray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(px: &[u8]) -> usize {
        px.chunks_exact(4).filter(|c| c[3] != 0).count()
    }

    #[test]
    fn build_builtin_returns_fully_painted_set() {
        let set = GuiTextureSet::build_builtin();
        // every sprite non-empty
        for (name, sheet) in [
            ("hearts", &set.hearts),
            ("hunger", &set.hunger),
            ("armor", &set.armor),
            ("bubbles", &set.bubbles),
            ("widgets", &set.widgets),
            ("hotbar_bg", &set.hotbar_bg),
            ("hotbar_sel", &set.hotbar_sel),
            ("dirt", &set.dirt),
        ] {
            assert!(painted(&sheet.px) > 20, "{name} sheet is blank");
            assert_eq!(sheet.px.len(), sheet.w * sheet.h * 4, "{name} size");
        }
        assert!(matches!(set.font, FontSource::BuiltinArray));
    }

    #[test]
    fn sprite_tile_dimensions_match_spec() {
        // VERIFIED sprite dims: hearts/hunger/armor 3x 9x9, bubbles 2x
        // 9x9, widgets 6x 20x20, hotbar 182x22, sel 24x22, dirt 16x16
        let set = GuiTextureSet::build_builtin();
        for (sheet, tw, th, count) in [
            (&set.hearts, 9, 9, 3),
            (&set.hunger, 9, 9, 3),
            (&set.armor, 9, 9, 3),
            (&set.bubbles, 9, 9, 2),
            (&set.widgets, 20, 20, 6),
            (&set.hotbar_bg, 182, 22, 1),
            (&set.hotbar_sel, 24, 22, 1),
            (&set.dirt, 16, 16, 1),
        ] {
            assert_eq!((sheet.tile_w, sheet.tile_h), (tw, th));
            assert_eq!(sheet.tile_count(), count);
        }
    }

    #[test]
    fn tile_uv_maps_tiles_across_the_strip() {
        let set = GuiTextureSet::build_builtin();
        let (u0, v0, uw, vh) = set.hearts.tile_uv(0).unwrap_or((-1.0, 0.0, 0.0, 0.0));
        assert_eq!((u0, v0, uw, vh), (0.0, 0.0, 9.0 / 27.0, 1.0));
        let (u1, _, _, _) = set.hearts.tile_uv(1).unwrap_or((-1.0, 0.0, 0.0, 0.0));
        assert!((u1 - 9.0 / 27.0).abs() < 1e-6);
        assert!(set.hearts.tile_uv(3).is_none());
    }

    #[test]
    fn builtin_set_is_deterministic() {
        let a = GuiTextureSet::build_builtin();
        let b = GuiTextureSet::build_builtin();
        assert_eq!(a.hearts.px, b.hearts.px);
        assert_eq!(a.widgets.px, b.widgets.px);
        assert_eq!(a.dirt.px, b.dirt.px);
    }
}
