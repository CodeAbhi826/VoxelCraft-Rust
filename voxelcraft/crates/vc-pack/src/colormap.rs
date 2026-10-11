//! Pack colormaps (Part 5, 2B): `textures/colormap/grass.png` and
//! `foliage.png` overrides read from the enabled pack stack (vanilla
//! paths, namespace-stripped flat keys). Decoded to raw RGBA; sampling
//! (climate UV → linear tint) lives in `vc_blocks::tint` next to the
//! engine LUT it overrides. Absent files = engine constants (no-op).

use std::sync::Arc;

/// A decoded colormap image (vanilla maps are 256×256 RGBA).
pub struct Colormap {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub source: String,
}

/// Load both colormaps from the stack (missing files stay `None`).
pub fn load_colormaps(stack: &crate::pack::PackStack) -> (Option<Colormap>, Option<Colormap>) {
    (
        load_one(stack, "textures/colormap/grass.png"),
        load_one(stack, "textures/colormap/foliage.png"),
    )
}

fn load_one(stack: &crate::pack::PackStack, path: &str) -> Option<Colormap> {
    let (bytes, source) = stack.read_first(path)?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return None;
    }
    Some(Colormap {
        w,
        h,
        rgba: img.into_raw(),
        source,
    })
}

/// Nearest sample as sRGB bytes (linearization happens in tint sampling).
pub fn sample_bytes(map: &Colormap, u: f32, v: f32) -> [u8; 3] {
    let x = (u.clamp(0.0, 1.0) * (map.w - 1) as f32).round() as u32;
    let y = (v.clamp(0.0, 1.0) * (map.h - 1) as u32 as f32).round() as u32;
    let i = ((y * map.w + x) * 4) as usize;
    [map.rgba[i], map.rgba[i + 1], map.rgba[i + 2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_source(files: Vec<(&str, Vec<u8>)>) -> Arc<dyn crate::pack::PackSource> {
        let mut m = crate::pack::MemorySource::new("test");
        for (p, b) in files {
            m.insert(p, b);
        }
        Arc::new(m)
    }

    fn png_bytes(px: &[[u8; 3]]) -> Vec<u8> {
        let img: image::RgbaImage = image::ImageBuffer::from_fn(2, 2, |x, y| {
            let p = px[(y * 2 + x) as usize];
            image::Rgba([p[0], p[1], p[2], 255])
        });
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn loads_grass_and_foliage_by_flat_key() {
        let px = [[10u8, 20, 30], [40, 50, 60], [70, 80, 90], [100, 110, 120]];
        let mut stack = crate::pack::PackStack::new();
        stack.push_front(mem_source(vec![(
            "textures/colormap/grass.png",
            png_bytes(&px),
        )]));
        let (grass, foliage) = load_colormaps(&stack);
        let g = grass.expect("grass loads");
        assert_eq!((g.w, g.h), (2, 2));
        assert!(foliage.is_none());
        // nearest corners round-trip
        assert_eq!(sample_bytes(&g, 0.0, 0.0), [10, 20, 30]);
        assert_eq!(sample_bytes(&g, 1.0, 1.0), [100, 110, 120]);
    }

    #[test]
    fn missing_or_garbage_is_none() {
        let stack = crate::pack::PackStack::new();
        let (g, f) = load_colormaps(&stack);
        assert!(g.is_none() && f.is_none());
        let mut stack2 = crate::pack::PackStack::new();
        stack2.push_front(mem_source(vec![(
            "textures/colormap/grass.png",
            b"not a png".to_vec(),
        )]));
        assert!(load_colormaps(&stack2).0.is_none());
    }
}
