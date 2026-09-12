//! `gui/loader.rs` — GUI texture override loading (UI-overhaul Phase 1, D3).
//!
//! Phase 1 ships a filesystem placeholder with the right *shape*: it
//! reads `hearts.png` / `hunger.png` / … from a directory, validates
//! dimensions against [`SHEET_DIMS`], and merges whatever it finds over
//! the builtin set (a pack that provides only `hearts.png` overrides
//! hearts and nothing else). Phase 4 replaces the directory read with
//! the `vc-pack` resolver — same merge semantics, pack-priority order.
//!
//! PNG decode uses the crate's existing `image` dependency (the repo
//! has no `png` crate and none is being added). No `unwrap()`/`expect()`
//! anywhere: every failure is a typed [`GuiTextureError`] and boot
//! always continues with the builtin set.

use std::path::Path;

use super::set::{FontSource, GuiTextureError, GuiTextureSet, SpriteSheet, SHEET_DIMS};

/// decode a PNG file to (rgba bytes, w, h) — typed errors, no panics
fn decode_png(bytes: &[u8]) -> Result<(Vec<u8>, usize, usize), image::ImageError> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let w = rgba.width() as usize;
    let h = rgba.height() as usize;
    Ok((rgba.into_raw(), w, h))
}

/// build a single-tile sheet from decoded RGBA pixels (dimension-
/// validated by the caller)
fn sheet_from_rgba(px: Vec<u8>, w: usize, h: usize, tile_w: usize) -> SpriteSheet {
    SpriteSheet {
        px,
        w,
        h,
        tile_w,
        tile_h: h,
    }
}

/// Load a user-supplied override set from a directory.
///
/// * `Ok(None)` — the directory does not exist or holds no recognizable
///   GUI PNG (nothing overridden).
/// * `Ok(Some(set))` — builtin set with every valid PNG merged over it.
/// * `Err(WrongDimensions)` — a recognized PNG has the wrong size
///   (rejected loudly; the caller logs and keeps the builtin set).
///
/// The builtin set itself never goes through this path — it comes from
/// [`GuiTextureSet::build_builtin`] and never touches disk.
pub fn load_override(dir: &Path) -> Result<Option<GuiTextureSet>, GuiTextureError> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut set = GuiTextureSet::build_builtin();
    let mut overridden = 0usize;

    for &(name, ew, eh) in SHEET_DIMS {
        if name == "font" {
            continue; // handled by load_font_png_if_present
        }
        let file = dir.join(format!("{name}.png"));
        let Ok(bytes) = std::fs::read(&file) else {
            continue; // not provided by this override dir
        };
        let file_label: &'static str = match name {
            "hearts" => "hearts.png",
            "hunger" => "hunger.png",
            "armor" => "armor.png",
            "bubbles" => "bubbles.png",
            "widgets" => "widgets.png",
            "hotbar" => "hotbar.png",
            "hotbar_sel" => "hotbar_sel.png",
            "options_background" => "options_background.png",
            _ => "font.png",
        };
        let (px, w, h) = decode_png(&bytes).map_err(|_| GuiTextureError::Decode {
            file: file_label,
        })?;
        if (w, h) != (ew, eh) {
            return Err(GuiTextureError::WrongDimensions {
                file: file_label,
                expected: (ew, eh),
                got: (w, h),
            });
        }
        match name {
            "hearts" => set.hearts = sheet_from_rgba(px, w, h, 9),
            "hunger" => set.hunger = sheet_from_rgba(px, w, h, 9),
            "armor" => set.armor = sheet_from_rgba(px, w, h, 9),
            "bubbles" => set.bubbles = sheet_from_rgba(px, w, h, 9),
            "widgets" => set.widgets = sheet_from_rgba(px, w, h, 20),
            "hotbar" => set.hotbar_bg = sheet_from_rgba(px, w, h, 182),
            "hotbar_sel" => set.hotbar_sel = sheet_from_rgba(px, w, h, 24),
            "options_background" => set.dirt = sheet_from_rgba(px, w, h, 16),
            _ => {}
        }
        overridden += 1;
    }

    if overridden == 0 {
        return Ok(None);
    }
    Ok(Some(set))
}

/// If `path` exists and decodes as a 128x48 RGBA glyph sheet, replaces
/// `set.font` with [`FontSource::Png`] and returns `true`. If absent,
/// keeps [`FontSource::BuiltinArray`] and returns `false`. Never panics.
pub fn load_font_png_if_present(set: &mut GuiTextureSet, path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let decoded = match decode_png(&bytes) {
        Ok(d) => d,
        Err(_) => {
            // a broken font pack texture is logged by the caller; keep
            // the builtin font
            return false;
        }
    };
    let (px, w, h) = decoded;
    if (w, h) != (128, 48) {
        return false;
    }
    set.font = FontSource::Png(px);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// write a flat-color PNG of the given size to dir/name.png
    fn write_flat_png(dir: &Path, name: &str, w: u32, h: u32, color: [u8; 4]) {
        let img = image::RgbaImage::from_fn(w, h, |_, _| {
            image::Rgba([color[0], color[1], color[2], color[3]])
        });
        let _ = img.save(dir.join(format!("{name}.png")));
    }

    #[test]
    fn load_override_on_missing_dir_returns_none() {
        let r = load_override(Path::new("no/such/gui/dir"));
        assert!(r.is_ok());
        assert!(r.ok().flatten().is_none());
    }

    #[test]
    fn load_override_on_empty_dir_returns_none() {
        let tmp = std::env::temp_dir().join("vc_gui_test_empty");
        let _ = fs::create_dir_all(&tmp);
        let r = load_override(&tmp);
        assert!(r.is_ok());
        assert!(r.ok().flatten().is_none());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_override_wrong_sized_png_errors() {
        let tmp = std::env::temp_dir().join("vc_gui_test_wrongsize");
        let _ = fs::create_dir_all(&tmp);
        // hearts must be 27x9 — write a 9x9 instead
        write_flat_png(&tmp, "hearts", 9, 9, [255, 0, 0, 255]);
        let r = load_override(&tmp);
        match r {
            Err(GuiTextureError::WrongDimensions {
                file,
                expected,
                got,
            }) => {
                assert_eq!(file, "hearts.png");
                assert_eq!(expected, (27, 9));
                assert_eq!(got, (9, 9));
            }
            other => panic!("expected WrongDimensions, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_override_correct_png_returns_overridden_set() {
        let tmp = std::env::temp_dir().join("vc_gui_test_ok");
        let _ = fs::create_dir_all(&tmp);
        // solid red hearts strip (27x9)
        write_flat_png(&tmp, "hearts", 27, 9, [255, 0, 0, 255]);
        let r = load_override(&tmp);
        assert!(r.is_ok());
        let set = r.ok().flatten();
        assert!(set.is_some(), "override set expected");
        let set = set.unwrap_or_else(|| GuiTextureSet::build_builtin());
        // hearts overridden: first pixel is red, NOT the builtin shell
        assert_eq!(&set.hearts.px[0..4], &[255, 0, 0, 255]);
        // everything else still builtin: hunger's first pixel is NOT the
        // override red (builtin mask leaves it transparent)
        assert!(
            !(set.hunger.px[0] == 255 && set.hunger.px[1] == 0 && set.hunger.px[2] == 0
                && set.hunger.px[3] == 255),
            "hunger must not be overridden by a hearts-only pack"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_font_png_if_present_absent_keeps_builtin() {
        let mut set = GuiTextureSet::build_builtin();
        let changed = load_font_png_if_present(&mut set, Path::new("no/such/font.png"));
        assert!(!changed);
        assert!(matches!(set.font, FontSource::BuiltinArray));
    }

    #[test]
    fn load_font_png_if_present_valid_png_takes_over() {
        let tmp = std::env::temp_dir().join("vc_gui_test_font");
        let _ = fs::create_dir_all(&tmp);
        // a 128x48 sheet with a distinct first pixel
        let img = image::RgbaImage::from_fn(128, 48, |x, y| {
            if x == 0 && y == 0 {
                image::Rgba([1, 2, 3, 255])
            } else {
                image::Rgba([200, 200, 200, 255])
            }
        });
        let _ = img.save(tmp.join("font.png"));
        let mut set = GuiTextureSet::build_builtin();
        let changed = load_font_png_if_present(&mut set, &tmp.join("font.png"));
        assert!(changed);
        match &set.font {
            FontSource::Png(px) => {
                assert_eq!(&px[0..4], &[1, 2, 3, 255]);
                assert_eq!(px.len(), 128 * 48 * 4);
            }
            FontSource::BuiltinArray => panic!("font PNG did not take over"),
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
