//! Player-skin loading (Part 5 skins epic, slice 1 — load + validate).
//!
//! Layout facts per the public Skin wiki (fetched 2026-10-12, L1-clean):
//! - Java accepts 64×64 or legacy 64×32 PNGs; the inner layer is
//!   forced opaque, the outer layer may be transparent.
//! - "Slim" = 3px arms (vs 4px classic); the model flag travels with
//!   the skin, not in the pixels.
//! - Legacy 64×32 upgrades by transparent fill (no geometry change).
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
}
