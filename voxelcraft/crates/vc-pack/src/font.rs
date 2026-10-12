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
                    return Err(
                        "font json: legacy_unicode template lacks %s".to_string()
                    );
                }
                out.push(FontProvider::LegacyUnicode {
                    template,
                    sizes: p.sizes.unwrap_or_else(|| "font/glyph_sizes.bin".to_string()),
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
pub fn rasterize_bitmap(
    chars: &[String],
    file_bytes: &[u8],
) -> Result<Vec<BitmapGlyph>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 2x1 grid PNG: 'A' solid 8x8, 'B' right-half ink
    fn grid_png() -> Vec<u8> {
        let img = image::RgbaImage::from_fn(16, 8, |x, _| {
            if x < 8 {
                image::Rgba([255, 255, 255, 255])
            } else if x >= 12 {
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
        assert!(matches!(
            &ps[1],
            FontProvider::LegacyUnicode { .. }
        ));
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
        assert!(parse_font(br#"{"providers": [{"type": "legacy_unicode", "template": "no-pages"}]}"#)
            .is_err());
        assert!(rasterize_bitmap(&["AB".to_string()], b"garbage").is_err());
    }
}
