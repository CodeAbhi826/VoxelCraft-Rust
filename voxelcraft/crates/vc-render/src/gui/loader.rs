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


/// Phase 4 (D2): resolve GUI textures through the `vc-pack` stack.
/// Resolution order per sheet: highest-priority user pack first, then
/// lower packs, then the builtin set. A pack that provides only
/// `gui/hearts` overrides hearts and nothing else. Decode failures are
/// typed errors (the caller logs + keeps builtin); a stack providing
/// nothing returns Ok(None).
pub fn load_from_pack(
    stack: &vc_pack::pack::PackStack,
) -> Result<Option<GuiTextureSet>, GuiTextureError> {
    if stack.is_empty() {
        return Ok(None);
    }
    let mut set = GuiTextureSet::build_builtin();
    let mut overridden = 0usize;
    let mut applied: Vec<String> = Vec::new();

    for (name, ew, eh) in SHEET_DIMS {
        let path = vc_pack::pack::gui_texture_path(name);
        let Some((bytes, source)) = stack.read_first(&path) else {
            continue; // no pack provides this texture -> builtin stays
        };
        let file_label: &'static str = match *name {
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
        if (w, h) != (*ew, *eh) {
            return Err(GuiTextureError::WrongDimensions {
                file: file_label,
                expected: (*ew, *eh),
                got: (w, h),
            });
        }
        match *name {
            "hearts" => set.hearts = sheet_from_rgba(px, w, h, 9),
            "hunger" => set.hunger = sheet_from_rgba(px, w, h, 9),
            "armor" => set.armor = sheet_from_rgba(px, w, h, 9),
            "bubbles" => set.bubbles = sheet_from_rgba(px, w, h, 9),
            "widgets" => set.widgets = sheet_from_rgba(px, w, h, 20),
            "hotbar" => set.hotbar_bg = sheet_from_rgba(px, w, h, 182),
            "hotbar_sel" => set.hotbar_sel = sheet_from_rgba(px, w, h, 24),
            "options_background" => set.dirt = sheet_from_rgba(px, w, h, 16),
            _ => {
                // font.png: 128x48 glyph sheet takes over as FontSource
                set.font = FontSource::Png(px);
            }
        }
        overridden += 1;
        applied.push(format!("{name} <- {source}"));
    }

    if overridden == 0 {
        return Ok(None);
    }
    // the caller logs which sheets came from which pack
    let _ = &applied;
    Ok(Some(set))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// encode a flat-color RGBA PNG sheet of the given size
    fn flat_png(w: u32, h: u32, color: [u8; 4]) -> Vec<u8> {
        let img = image::RgbaImage::from_fn(w, h, |_, _| image::Rgba(color));
        let mut out = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut out));
        let _ = image::ImageEncoder::write_image(
            enc,
            img.as_raw(),
            w,
            h,
            image::ExtendedColorType::Rgba8,
        );
        out
    }

    /// write a flat-color PNG to dir/name.png
    fn write_flat_png(dir: &Path, name: &str, w: u32, h: u32, color: [u8; 4]) {
        let _ = std::fs::write(dir.join(format!("{name}.png")), flat_png(w, h, color));
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
        match load_override(&tmp) {
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
        let set = set.unwrap_or_else(GuiTextureSet::build_builtin);
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
        let mut px = vec![200u8, 200, 200, 255];
        px.reserve(128 * 48 * 4 - 4);
        let mut img = image::RgbaImage::from_pixel(128, 48, image::Rgba([200, 200, 200, 255]));
        img.put_pixel(0, 0, image::Rgba([1, 2, 3, 255]));
        let mut png = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png));
        let _ = image::ImageEncoder::write_image(
            enc,
            img.as_raw(),
            128,
            48,
            image::ExtendedColorType::Rgba8,
        );
        let _ = std::fs::write(tmp.join("font.png"), png);
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

    // -------------------------------------------------- Phase 4 ----

    /// a MemorySource pack carrying one gui sheet as a real PNG
    fn pack_with_sheet(name: &str, w: u32, h: u32, color: [u8; 4]) -> vc_pack::pack::MemorySource {
        let mut pack = vc_pack::pack::MemorySource::new("test-pack");
        pack.insert(
            &vc_pack::pack::gui_texture_path(name),
            flat_png(w, h, color),
        );
        pack
    }

    #[test]
    fn load_from_pack_overrides_only_provided_textures() {
        // a pack providing ONLY gui/hearts: hearts overridden, hunger
        // stays builtin, result is Some(set)
        let mut stack = vc_pack::pack::PackStack::new();
        stack.push_front(std::sync::Arc::new(pack_with_sheet(
            "hearts",
            27,
            9,
            [10, 200, 90, 255],
        )));
        let r = load_from_pack(&stack);
        assert!(r.is_ok());
        let set = r.ok().flatten();
        assert!(set.is_some(), "partial override returns a set");
        let set = set.unwrap_or_else(GuiTextureSet::build_builtin);
        assert_eq!(&set.hearts.px[0..4], &[10, 200, 90, 255]);
        // hunger untouched by the hearts-only pack
        assert!(
            !(set.hunger.px[0] == 10 && set.hunger.px[1] == 200 && set.hunger.px[3] == 255),
            "hunger must stay builtin"
        );
    }

    #[test]
    fn load_from_pack_on_empty_stack_returns_none() {
        let stack = vc_pack::pack::PackStack::new();
        let r = load_from_pack(&stack);
        assert!(r.is_ok());
        assert!(r.ok().flatten().is_none());
    }

    #[test]
    fn load_from_pack_wrong_dimensions_is_a_typed_error() {
        let mut stack = vc_pack::pack::PackStack::new();
        // armor must be 27x9 — a 9x9 sheet is rejected loudly
        stack.push_front(std::sync::Arc::new(pack_with_sheet(
            "armor",
            9,
            9,
            [1, 2, 3, 255],
        )));
        match load_from_pack(&stack) {
            Err(GuiTextureError::WrongDimensions { file, expected, got }) => {
                assert_eq!(file, "armor.png");
                assert_eq!(expected, (27, 9));
                assert_eq!(got, (9, 9));
            }
            other => panic!("expected WrongDimensions, got {other:?}"),
        }
    }

    #[test]
    fn load_from_pack_priority_highest_pack_wins() {
        // two packs both provide hearts: the one at the FRONT of the
        // stack wins
        let mut stack = vc_pack::pack::PackStack::new();
        // blue first, then red lands AT the front = highest priority
        stack.push_front(std::sync::Arc::new(pack_with_sheet(
            "hearts", 27, 9, [0, 0, 255, 255],
        )));
        stack.push_front(std::sync::Arc::new(pack_with_sheet(
            "hearts", 27, 9, [255, 0, 0, 255],
        )));
        let set = load_from_pack(&stack)
            .ok()
            .flatten()
            .unwrap_or_else(GuiTextureSet::build_builtin);
        assert_eq!(&set.hearts.px[0..4], &[255, 0, 0, 255], "front pack wins");
    }

    #[test]
    fn load_from_pack_font_sheet_takes_over_as_png_source() {
        let mut stack = vc_pack::pack::PackStack::new();
        stack.push_front(std::sync::Arc::new(pack_with_sheet(
            "font",
            128,
            48,
            [9, 8, 7, 255],
        )));
        let set = load_from_pack(&stack)
            .ok()
            .flatten()
            .unwrap_or_else(GuiTextureSet::build_builtin);
        match &set.font {
            FontSource::Png(px) => {
                assert_eq!(px.len(), 128 * 48 * 4);
                assert_eq!(&px[0..4], &[9, 8, 7, 255]);
            }
            FontSource::BuiltinArray => panic!("pack font.png did not take over"),
        }
    }
}
