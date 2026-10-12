//! Pack font providers (Part 5 §2B.4, completed Part 6).
//!
//! Schema facts per the public Font wiki (fetched 2026-10-12, L1-clean):
//! - `bitmap`: `file` PNG + `ascent` int + `chars` string grid +
//!   optional `height` (defaults 8). The texture divides evenly into
//!   rows = chars.len() by cols = longest string; glyph width is
//!   automatic (last column with alpha > 0). Cells may be any size;
//!   glyphs must not exceed 256×256.
//! - `legacy_unicode`: `template` (with `%s` over pages 00–FF) +
//!   `sizes` (0xFFFF-byte nibble widths). Deprecated but present
//!   in 1.16 — parsed here, rasterized in the next slice.
//! - Unknown `type` values are recorded as Unsupported, never
//!   silently dropped (same honesty rule as shaders.properties).

use serde::Deserialize;

/// One parsed font provider (owned, validated shapes only).
#[derive(Clone, Debug)]
pub enum FontProvider {
    Bitmap {
        file: String,
        ascent: Option<i32>,
        height: u32,
        chars: Vec<String>,
    },
    LegacyUnicode {
        template: String,
        sizes: String,
    },
    Unsupported {
        type_name: String,
    },
}

#[derive(Deserialize)]
struct FontJson {
    #[serde(default)]
    providers: Vec<ProviderJson>,
}

#[derive(Deserialize)]
struct ProviderJson {
    #[serde(rename = "type")]
    type_name: String,
    file: Option<String>,
    ascent: Option<i32>,
    height: Option<u32>,
    chars: Option<Vec<String>>,
    template: Option<String>,
    sizes: Option<String>,
}

/// Parse a font JSON (`assets/<ns>/font/*.json`) into providers.
/// Unknown types survive as Unsupported; malformed JSON errors.
pub fn parse_font(json: &[u8]) -> Result<Vec<FontProvider>, String> {
    let doc: FontJson =
        serde_json::from_slice(json).map_err(|e| format!("font json: bad JSON: {e}"))?;
    let mut out = Vec::with_capacity(doc.providers.len());
    for p in doc.providers {
        match p.type_name.as_str() {
            "bitmap" => {
                let file = p
                    .file
                    .ok_or_else(|| "font json: bitmap provider lacks file".to_string())?;
                let chars = p
                    .chars
                    .ok_or_else(|| "font json: bitmap provider lacks chars".to_string())?;
                if chars.is_empty() {
                    return Err("font json: bitmap provider has empty chars".to_string());
                }
                out.push(FontProvider::Bitmap {
                    file,
                    ascent: p.ascent,
                    height: p.height.unwrap_or(8),
                    chars,
                });
            }
            "legacy_unicode" => {
                let template = p
                    .template
                    .ok_or_else(|| "font json: legacy_unicode lacks template".to_string())?;
                if !template.contains("%s") {
                    return Err("font json: legacy_unicode template lacks %s".to_string());
                }
                out.push(FontProvider::LegacyUnicode {
                    template,
                    sizes: p
                        .sizes
                        .unwrap_or_else(|| "font/glyph_sizes.bin".to_string()),
                });
            }
            other => out.push(FontProvider::Unsupported {
                type_name: other.to_string(),
            }),
        }
    }
    Ok(out)
}

/// One rasterized bitmap glyph: alpha mask + measured width.
pub struct BitmapGlyph {
    pub ch: char,
    pub cell_w: u32,
    pub cell_h: u32,
    /// auto width: last column with alpha > 0 (min 1)
    pub width: u32,
    /// row-major alpha bytes, cell_w × cell_h
    pub alpha: Vec<u8>,
}

/// Rasterize a bitmap provider's glyphs from its PNG bytes.
/// Grid: rows = chars.len(), cols = longest string; `\0` cells are
/// padding (skipped). Never panics on corrupt input.
pub fn rasterize_bitmap(chars: &[String], file_bytes: &[u8]) -> Result<Vec<BitmapGlyph>, String> {
    let img = image::load_from_memory(file_bytes)
        .map_err(|e| format!("font bitmap: bad PNG: {e}"))?
        .to_rgba8();
    let (w, h) = img.dimensions();
    let rows = chars.len() as u32;
    let cols = chars
        .iter()
        .map(|s| s.chars().count() as u32)
        .max()
        .unwrap_or(0);
    if rows == 0 || cols == 0 {
        return Err("font bitmap: empty chars grid".to_string());
    }
    if w % cols != 0 || h % rows != 0 {
        return Err(format!(
            "font bitmap: {w}x{h} not divisible into {cols}x{rows}"
        ));
    }
    let (cw, ch) = (w / cols, h / rows);
    if cw == 0 || ch == 0 || cw > 256 || ch > 256 {
        return Err(format!("font bitmap: bad cell {cw}x{ch}"));
    }
    let mut out = Vec::new();
    for (r, line) in chars.iter().enumerate() {
        for (c, ch_) in line.chars().enumerate() {
            if ch_ == '\0' {
                continue;
            }
            let mut alpha = vec![0u8; (cw * ch) as usize];
            let mut width = 1u32;
            for y in 0..ch {
                for x in 0..cw {
                    let px = img.get_pixel(c as u32 * cw + x, r as u32 * ch + y);
                    alpha[(y * cw + x) as usize] = px[3];
                    if px[3] > 0 {
                        width = width.max(x + 1);
                    }
                }
            }
            out.push(BitmapGlyph {
                ch: ch_,
                cell_w: cw,
                cell_h: ch,
                width,
                alpha,
            });
        }
    }
    Ok(out)
}

/// One rasterized legacy-Unicode glyph: 16×16 alpha + nibble width.
pub struct LegacyGlyph {
    pub ch: char,
    pub width: u32,
    pub alpha: Vec<u8>,
}

/// Rasterize one legacy-Unicode template page (256×256 PNG, 16×16 grid
/// of 16px glyphs) using the sizes table (0xFFFF bytes; high nibble =
/// start col, low nibble = end col). Pages D8–DF are UTF-16 surrogates
/// and invalid — rejected, never rendered. Never panics on bad input.
pub fn rasterize_legacy(
    page: u8,
    page_png: &[u8],
    sizes: &[u8],
) -> Result<Vec<LegacyGlyph>, String> {
    if (0xD8..=0xDF).contains(&page) {
        return Err(format!("font legacy: page {page:02X} is a surrogate page"));
    }
    if sizes.len() != 0xFFFF {
        return Err(format!(
            "font legacy: sizes is {} bytes, want 65535",
            sizes.len()
        ));
    }
    let img = image::load_from_memory(page_png)
        .map_err(|e| format!("font legacy: bad page PNG: {e}"))?
        .to_rgba8();
    if img.dimensions() != (256, 256) {
        return Err(format!(
            "font legacy: page is {:?}, want 256x256",
            img.dimensions()
        ));
    }
    let mut out = Vec::with_capacity(256);
    for i in 0..256u32 {
        let code = (page as u32) * 256 + i;
        let Some(ch) = char::from_u32(code) else {
            continue;
        };
        let wbyte = sizes[code as usize];
        let (start, end) = (wbyte >> 4, wbyte & 0x0F);
        let width = if end >= start {
            (end - start + 1) as u32
        } else {
            1
        };
        let (gx, gy) = ((i % 16) * 16, (i / 16) * 16);
        let mut alpha = vec![0u8; 256];
        for y in 0..16u32 {
            for x in 0..16u32 {
                alpha[(y * 16 + x) as usize] = img.get_pixel(gx + x, gy + y)[3];
            }
        }
        out.push(LegacyGlyph { ch, width, alpha });
    }
    Ok(out)
}

/// Build the engine 96-glyph form ([u8; 8] per glyph, 5 ink bits per
/// row, alpha >= 128) for U+0020..=U+007E from bitmap providers plus
/// legacy page 00. Later providers win per char. Cells resample to 8×8
/// nearest; provider height/ascent do not survive the fixed-cell form
/// (documented limitation). `read` resolves pack-flat file keys
/// verbatim (vanilla convention: "font/ascii.png"). Returns None when
/// no ASCII glyph was produced.
pub fn build_ascii_glyphs(
    providers: &[FontProvider],
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Option<Box<[[u8; 8]; 96]>> {
    fn pack_cell(cell: &[u8], cw: u32, chh: u32) -> [u8; 8] {
        let mut g = [0u8; 8];
        if cw == 0 || chh == 0 {
            return g;
        }
        for gy in 0..8u32 {
            let mut row = 0u8;
            for gx in 0..5u32 {
                let sx = (gx * cw / 8).min(cw - 1);
                let sy = (gy * chh / 8).min(chh - 1);
                if cell[(sy * cw + sx) as usize] >= 128 {
                    row |= 1 << (4 - gx);
                }
            }
            g[gy as usize] = row;
        }
        g
    }
    let mut glyphs = Box::new([[0u8; 8]; 96]);
    let mut any = false;
    for p in providers {
        match p {
            FontProvider::Bitmap { file, chars, .. } => {
                let Some(bytes) = read(file) else {
                    continue;
                };
                let Ok(gs) = rasterize_bitmap(chars, &bytes) else {
                    continue;
                };
                for g in gs {
                    let idx = g.ch as usize;
                    if !(0x20..=0x7E).contains(&idx) {
                        continue;
                    }
                    glyphs[idx - 0x20] = pack_cell(&g.alpha, g.cell_w, g.cell_h);
                    any = true;
                }
            }
            FontProvider::LegacyUnicode { template, sizes } => {
                let page_key = template.replace("%s", "00");
                let (Some(page), Some(sizes)) = (read(&page_key), read(sizes)) else {
                    continue;
                };
                let Ok(gs) = rasterize_legacy(0x00, &page, &sizes) else {
                    continue;
                };
                for g in gs {
                    let idx = g.ch as usize;
                    if !(0x20..=0x7E).contains(&idx) {
                        continue;
                    }
                    glyphs[idx - 0x20] = pack_cell(&g.alpha, 16, 16);
                    any = true;
                }
            }
            FontProvider::Unsupported { .. } => {}
        }
    }
    if any {
        Some(glyphs)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2x1 grid PNG: 'A' solid 8x8, 'B' ink in left half only
    fn grid_png() -> Vec<u8> {
        let img = image::RgbaImage::from_fn(16, 8, |x, _| {
            if x < 12 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn bitmap_provider_parses_and_rasterizes() {
        let json = br#"{"providers": [
            {"type": "bitmap", "file": "font/ascii.png",
             "ascent": 7, "height": 8, "chars": ["AB"]},
            {"type": "legacy_unicode", "template": "font/glyph_%s.png"},
            {"type": "space", "advances": {" ": 4}}
        ]}"#;
        let ps = parse_font(json).unwrap();
        assert_eq!(ps.len(), 3);
        assert!(matches!(&ps[0], FontProvider::Bitmap { .. }));
        assert!(matches!(&ps[1], FontProvider::LegacyUnicode { .. }));
        // sizes defaults to glyph_sizes.bin
        let FontProvider::LegacyUnicode { sizes, .. } = &ps[1] else {
            unreachable!()
        };
        assert_eq!(sizes, "font/glyph_sizes.bin");
        // unknown types survive, recorded
        let FontProvider::Unsupported { type_name } = &ps[2] else {
            panic!("space must be Unsupported, got {:?}", ps[2])
        };
        assert_eq!(type_name, "space");

        // rasterize: A full-width, B half-width
        let FontProvider::Bitmap { chars, .. } = &ps[0] else {
            unreachable!()
        };
        let glyphs = rasterize_bitmap(chars, &grid_png()).unwrap();
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].ch, 'A');
        assert_eq!(glyphs[0].width, 8);
        assert_eq!(glyphs[1].ch, 'B');
        assert_eq!(glyphs[1].width, 4);
    }

    #[test]
    fn malformed_font_json_errors() {
        assert!(parse_font(b"not json").is_err());
        assert!(parse_font(br#"{"providers": [{"type": "bitmap"}]}"#).is_err());
        assert!(parse_font(
            br#"{"providers": [{"type": "legacy_unicode", "template": "no-pages"}]}"#
        )
        .is_err());
        assert!(rasterize_bitmap(&["AB".to_string()], b"garbage").is_err());
    }

    #[test]
    fn legacy_page_rasterizes_widths_and_rejects_surrogates() {
        // 256x256 page: cell (0,0) fully inked, cell (1,0) ink in x 0..5
        let img = image::RgbaImage::from_fn(256, 256, |x, y| {
            if x < 16 && y < 16 {
                image::Rgba([255, 255, 255, 255])
            } else if (16..22).contains(&x) && y < 16 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        // sizes: glyph 0 full (0x0F), glyph 1 cols 0..5 (0x05), rest empty
        let mut sizes = vec![0u8; 0xFFFF];
        sizes[0] = 0x0F;
        sizes[1] = 0x05;
        let glyphs = rasterize_legacy(0x00, &png, &sizes).unwrap();
        assert_eq!(glyphs.len(), 256);
        assert_eq!(glyphs[0].ch, '\0');
        assert_eq!(glyphs[0].width, 16);
        assert_eq!(glyphs[1].width, 6);
        // surrogate pages rejected; bad shapes error, never panic
        assert!(rasterize_legacy(0xD8, &png, &sizes).is_err());
        assert!(rasterize_legacy(0x00, &png, &sizes[..100]).is_err());
        assert!(rasterize_legacy(0x00, b"garbage", &sizes).is_err());
    }

    #[test]
    fn ascii_builder_maps_bitmap_and_legacy_with_later_wins() {
        // bitmap file: 16x8, 'A' solid + 'B' left-half (from grid_png)
        let font_json = br#"{"providers": [
            {"type": "bitmap", "file": "font/ascii.png",
             "ascent": 7, "chars": ["AB"]}
        ]}"#;
        let providers = parse_font(font_json).unwrap();
        let read = |k: &str| {
            if k == "font/ascii.png" {
                Some(grid_png())
            } else {
                None
            }
        };
        let g = build_ascii_glyphs(&providers, &read).expect("ascii glyphs");
        // 'A' (0x41): full 8x8 ink → rows 0b11111; 'B': ink at local
        // x 0..3 (global 8..11) → 0b11110
        assert_eq!(g[0x41 - 0x20][0], 0b11111);
        assert_eq!(g[0x42 - 0x20][0], 0b11110);
        // untouched slots stay blank
        assert_eq!(g[0x43 - 0x20][0], 0);

        // legacy page 00 overrides 'A' when listed later
        let img = image::RgbaImage::from_pixel(256, 256, image::Rgba([0, 0, 0, 0]));
        let mut page = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut page), image::ImageFormat::Png)
            .unwrap();
        let sizes = vec![0u8; 0xFFFF];
        let font_json2 = br#"{"providers": [
            {"type": "bitmap", "file": "font/ascii.png",
             "ascent": 7, "chars": ["AB"]},
            {"type": "legacy_unicode", "template": "font/glyph_%s.png",
             "sizes": "font/glyph_sizes.bin"}
        ]}"#;
        let providers2 = parse_font(font_json2).unwrap();
        let read2 = |k: &str| match k {
            "font/ascii.png" => Some(grid_png()),
            "font/glyph_00.png" => Some(page.clone()),
            "font/glyph_sizes.bin" => Some(sizes.clone()),
            _ => None,
        };
        let g2 = build_ascii_glyphs(&providers2, &read2).expect("ascii glyphs 2");
        // legacy blank 'A' (width 1, no ink) overwrote the bitmap 'A'
        assert_eq!(g2[0x41 - 0x20][0], 0);

        // nothing resolvable → None (callers keep current font)
        let none_read = |_: &str| None;
        assert!(build_ascii_glyphs(&providers, &none_read).is_none());
    }
}
