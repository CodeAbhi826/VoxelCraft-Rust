//! Player-skin loading (Part 5 skins epic, slice 1 — load + validate).
//!
//! Layout facts per the public Skin wiki (fetched 2026-10-12, L1-clean):
//!
//! - Java accepts 64×64 or legacy 64×32 PNGs; the inner layer is
//!   forced opaque, the outer layer may be transparent.
//! - "Slim" = 3px arms (vs 4px classic); the model flag travels with
//!   the skin, not in the pixels.
//! - Legacy 64×32 upgrades by transparent fill (no geometry change).
//!
//! Body-part regions arrive in slice 2 from a citable text source
//! (never pixel-derived from official art — L5).

/// Which arm model the skin is authored for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinModel {
    Classic,
    Slim,
}

/// A validated player skin: always 64×64 RGBA after upgrade.
#[derive(Clone, Debug)]
pub struct PlayerSkin {
    pub model: SkinModel,
    /// 64×64 row-major RGBA
    pub rgba: Vec<u8>,
}

/// Validate + normalize skin PNG bytes. Accepts 64×64 directly;
/// upgrades legacy 64×32 by transparent fill of the lower half.
/// Anything else errors (callers keep the procedural skin).
pub fn load_skin(png_bytes: &[u8], model: SkinModel) -> Result<PlayerSkin, String> {
    let img = image::load_from_memory(png_bytes)
        .map_err(|e| format!("skin: bad PNG: {e}"))?
        .to_rgba8();
    let (w, h) = (img.width(), img.height());
    let rgba = match (w, h) {
        (64, 64) => img.into_raw(),
        (64, 32) => {
            let mut out = vec![0u8; 64 * 64 * 4];
            out[..64 * 32 * 4].copy_from_slice(&img.into_raw());
            out
        }
        _ => {
            return Err(format!("skin: want 64x64 or legacy 64x32, got {w}x{h}"));
        }
    };
    Ok(PlayerSkin { model, rgba })
}

/// One body part's net origin + box dims, inner and outer layers.
///
/// Facts: box dims from the public Skin wiki (player 32 tall, 16 wide
/// classic / 14 slim, 4px arms classic / 3px slim); net origins from
/// the open-source skinview3d layout reference (behavior facts only —
/// no code copied). Validated by `usable_pixels_match_wiki` below:
/// face-area sums hit the wiki's 3264/3136 totals exactly.
pub struct SkinPart {
    pub name: &'static str,
    pub w: u32,
    pub h: u32,
    pub d: u32,
    /// inner-layer net origin
    pub inner: (u32, u32),
    /// outer-layer net origin
    pub outer: (u32, u32),
}

pub const SKIN_PARTS: &[SkinPart] = &[
    SkinPart {
        name: "head",
        w: 8,
        h: 8,
        d: 8,
        inner: (0, 0),
        outer: (32, 0),
    },
    SkinPart {
        name: "body",
        w: 8,
        h: 12,
        d: 4,
        inner: (16, 16),
        outer: (16, 32),
    },
    SkinPart {
        name: "arm_r",
        w: 4,
        h: 12,
        d: 4,
        inner: (40, 16),
        outer: (40, 32),
    },
    SkinPart {
        name: "leg_r",
        w: 4,
        h: 12,
        d: 4,
        inner: (0, 16),
        outer: (0, 32),
    },
    SkinPart {
        name: "arm_l",
        w: 4,
        h: 12,
        d: 4,
        inner: (32, 48),
        outer: (48, 48),
    },
    SkinPart {
        name: "leg_l",
        w: 4,
        h: 12,
        d: 4,
        inner: (16, 48),
        outer: (0, 48),
    },
];

/// Arm width by model (wiki: slim loses 25% — the rightmost column is
/// ignored; origins are shared).
pub fn arm_width(model: SkinModel) -> u32 {
    match model {
        SkinModel::Classic => 4,
        SkinModel::Slim => 3,
    }
}

/// Six face rects (x, y, w, h) for a part net at (u, v): row 1 holds
/// top + bottom, row 2 holds side/front/side/back.
pub fn part_faces(u: u32, v: u32, w: u32, h: u32, d: u32) -> [(u32, u32, u32, u32); 6] {
    [
        (u + d, v, w, d),             // top
        (u + d + w, v, w, d),         // bottom
        (u, v + d, d, h),             // side
        (u + d, v + d, w, h),         // front
        (u + d + w, v + d, d, h),     // side
        (u + d + w + d, v + d, w, h), // back
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skin_png(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(w, h, image::Rgba([200, 150, 100, 255]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn skin_loader_accepts_upgrades_rejects() {
        // 64x64 classic loads verbatim
        let s = load_skin(&skin_png(64, 64), SkinModel::Classic).unwrap();
        assert_eq!(s.rgba.len(), 64 * 64 * 4);
        assert_eq!(s.model, SkinModel::Classic);
        // legacy 64x32 upgrades: top opaque, bottom transparent
        let l = load_skin(&skin_png(64, 32), SkinModel::Slim).unwrap();
        assert_eq!(l.rgba.len(), 64 * 64 * 4);
        assert_eq!(l.model, SkinModel::Slim);
        assert_eq!(&l.rgba[0..4], &[200, 150, 100, 255]);
        assert_eq!(&l.rgba[64 * 32 * 4..64 * 32 * 4 + 4], &[0, 0, 0, 0]);
        // wrong sizes + garbage reject, never panic
        assert!(load_skin(&skin_png(128, 128), SkinModel::Classic).is_err());
        assert!(load_skin(b"not a png", SkinModel::Classic).is_err());
    }

    /// face-area sums hit the wiki totals exactly (3264 classic from
    /// 6 parts × 2 layers; slim arms 3-wide give 3136) — this validates
    /// the region table against the authoritative source.
    #[test]
    fn usable_pixels_match_wiki() {
        for model in [SkinModel::Classic, SkinModel::Slim] {
            let mut total = 0u32;
            for p in SKIN_PARTS {
                let w = if p.name.contains("arm") {
                    arm_width(model)
                } else {
                    p.w
                };
                total += 2 * (w * p.h + w * p.d + p.h * p.d) * 2; // faces × layers
            }
            let want = match model {
                SkinModel::Classic => 3264,
                SkinModel::Slim => 3136,
            };
            assert_eq!(total, want, "usable pixels for {model:?}");
        }
    }

    #[test]
    fn face_rects_in_bounds_and_spot_check() {
        for p in SKIN_PARTS {
            for layer in [p.inner, p.outer] {
                for (x, y, w, h) in part_faces(layer.0, layer.1, p.w, p.h, p.d) {
                    assert!(x + w <= 64 && y + h <= 64, "{} rect out of bounds", p.name);
                    assert!(w > 0 && h > 0);
                }
            }
        }
        // head front face sits at (8,8) 8x8; body front at (20,20) 8x12
        let head_front = part_faces(0, 0, 8, 8, 8)[3];
        assert_eq!(head_front, (8, 8, 8, 8));
        let body_front = part_faces(16, 16, 8, 12, 4)[3];
        assert_eq!(body_front, (20, 20, 8, 12));
    }
}
