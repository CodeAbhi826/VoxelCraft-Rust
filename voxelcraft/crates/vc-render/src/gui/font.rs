//! `gui/font.rs` — Luanti-style runtime font engine (the FreeType
//! equivalent for VoxelCraft).
//!
//! Luanti rasterizes TTF/OTF fonts at runtime with FreeType into a
//! glyph cache and draws text as GPU quads at the resolution actually
//! on screen (`src/client/fontengine.cpp`). This module replicates
//! that architecture with `ab_glyph` (pure Rust, wasm-safe):
//!
//! * **Font**: Monocraft (SIL OFL 1.1 — see
//!   `assets/OFL-Monocraft.txt`), a Minecraft-style pixel font by
//!   IdreesInc, embedded via `include_bytes!` (G9 intact: no PNGs, no
//!   external files at runtime). The OFL permits embedding and
//!   redistribution.
//! * **Rasterization on demand**: glyphs rasterize with
//!   anti-aliasing at the DEVICE pixel size (UI cell × the canvas
//!   letterbox scale), so text stays crisp at any window size —
//!   Luanti's device-resolution glyph cache.
//! * **Glyph atlas**: rasterized glyphs pack into a 1024×1024 RGBA
//!   shelf atlas (white RGB × coverage alpha; vertex tint supplies
//!   the color). A full atlas resets and lazily re-rasters — the
//!   same strategy as Luanti's glyph-cache invalidation.
//! * **Proportional metrics, the Minecraft rule**: Monocraft is
//!   monospaced (720/1080 em advance), so the engine measures each
//!   glyph's ink width and lays out with `advance = ink + 1 px at the
//!   8-px reference cell` — the rule the vanilla font itself uses
//!   (and the rule Phase 5 already applies to the builtin bitmap
//!   font). Narrow letters pack tight; the UI reads like vanilla.
//! * **Fallback chain** (Luanti ships font fallbacks too): any char
//!   missing from the TTF cmap rasterizes from the builtin clean-room
//!   5×7 bitmap font instead, scaled to the same cell.
//!
//! Everything here is CPU-side and unit-testable without a GPU; the
//! renderer side (`GuiRenderer::sync_glyph_atlas`) drains the
//! pending uploads each frame.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use ab_glyph::{Font as _, FontRef, Glyph, PxScale};

use crate::ui::FONT;

/// The embedded OFL pixel font (~202 KB).
pub const MONOCRAFT_TTF: &[u8] = include_bytes!("../../assets/Monocraft.ttf");

/// Glyph atlas size (px). 1024² holds thousands of 24–40 px glyphs;
/// overflow resets the cache (`resets()`).
pub const GLYPH_ATLAS_PX: u32 = 1024;

/// The reference text cell: 8 UI px tall — the vanilla font's 8-px
/// line, the same cell the builtin bitmap font uses.
pub const REF_CELL: f32 = 8.0;

/// The vanilla cap proportion: caps occupy 7 of the 8 reference
/// cell pixels. The engine derives its raster scale from the measured
/// cap ratio (`raster_px = cell × 0.875 / cap_ratio`), so any font
/// lands on the vanilla proportion.
const CAP_PER_CELL: f32 = 0.875;

/// Ink-trim spacing: vanilla adds 1 px after the ink at the 8-px
/// reference cell (`ink + 1`), i.e. `cell / 8` at any cell size.
const SPACING_PER_CELL: f32 = 1.0 / 8.0;

/// canonical raster height for metric measurement (px)
const CANON_H: f32 = 96.0;

// ------------------------------------------------------------ types --

/// One pending atlas upload (the renderer drains these).
#[derive(Clone, Debug)]
pub struct PendingUpload {
    /// atlas-space rect (px), padding excluded
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// RGBA8 bytes, `w * h * 4` long (white RGB × coverage alpha)
    pub bytes: Vec<u8>,
}

/// A cached, rasterized glyph: atlas rect + geometry in DEVICE px.
#[derive(Copy, Clone, Debug)]
pub struct CachedGlyph {
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub w: u32,
    pub h: u32,
    /// ink top relative to the baseline (px; ≤ 0 above it)
    pub top_rel_baseline: f32,
}

/// Shelf packer: glyphs left→right, rows top→bottom, 1-px transparent
/// padding on every side (the glyph atlas samples LINEARLY — the
/// padding stops neighbor bleed at fractional device scales).
#[derive(Clone, Debug)]
struct ShelfPacker {
    w: u32,
    h: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
}

const PAD: u32 = 1;

impl ShelfPacker {
    fn new(w: u32, h: u32) -> Self {
        ShelfPacker {
            w,
            h,
            cursor_x: 0,
            cursor_y: 0,
            row_h: 0,
        }
    }

    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        let need_w = w + PAD * 2;
        let need_h = h + PAD * 2;
        if need_w > self.w {
            return None; // glyph wider than the atlas — never packable
        }
        if self.cursor_x + need_w > self.w {
            // wrap to the next shelf row
            self.cursor_y += self.row_h;
            self.cursor_x = 0;
            self.row_h = 0;
        }
        if self.cursor_y + need_h > self.h {
            return None; // atlas full → caller resets
        }
        let (x, y) = (self.cursor_x + PAD, self.cursor_y + PAD);
        self.cursor_x += need_w;
        self.row_h = self.row_h.max(need_h);
        Some((x, y))
    }

    fn reset(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_h = 0;
    }
}

// ----------------------------------------------------------- engine --

/// The runtime font engine — one process-wide instance behind a
/// `Mutex` in a `OnceLock` (thread-safe; the game loop is
/// single-threaded, so locks are uncontended).
pub struct FontEngine {
    font: FontRef<'static>,
    /// cap-height / font-height, measured from 'H' at init
    /// (Monocraft: 840/1440 ≈ 0.5833 — i.e. cap = 0.875 × cell)
    cap_ratio: f32,
    /// ink-width / font-height per char (measured lazily at the
    /// canonical raster; 0.0 for the space)
    ink: HashMap<char, f32>,
    /// rasterized glyphs keyed by (char, device cell px)
    glyphs: HashMap<(char, u32), CachedGlyph>,
    /// cached multi-glyph RUN strips keyed by (key string, device cell)
    /// — the splash text's colored outlined run, uploaded once and
    /// drawn as a single rotated quad. Cleared with the glyphs on
    /// every atlas reset (stale rects must never survive).
    runs: HashMap<(String, u32), CachedGlyph>,
    packer: ShelfPacker,
    pending: Vec<PendingUpload>,
    /// bumps on every rasterization/reset — the renderer re-uploads
    /// when its tracked version falls behind
    version: u64,
    resets: u32,
}

impl FontEngine {
    fn new(bytes: &'static [u8]) -> Option<Self> {
        let font = FontRef::try_from_slice(bytes).ok()?;
        let mut eng = FontEngine {
            font,
            cap_ratio: 0.78125, // patched from 'H' below (Monocraft: 840/1080 em)
            ink: HashMap::new(),
            glyphs: HashMap::new(),
            runs: HashMap::new(),
            packer: ShelfPacker::new(GLYPH_ATLAS_PX, GLYPH_ATLAS_PX),
            pending: Vec::new(),
            version: 0,
            resets: 0,
        };
        // cap ratio = the 'H' outline height / raster height
        if let Some((_, _, cap_h)) = eng.outline_coverage('H', CANON_H) {
            if cap_h > 0 {
                eng.cap_ratio = cap_h as f32 / CANON_H;
            }
        }
        Some(eng)
    }

    // ------------------------------------------------------- metrics --

    /// the ab_glyph PxScale raster size for a UI cell — derived from
    /// the measured cap ratio so the cap lands at `0.875 × cell`
    /// (Monocraft: 0.875 / 0.78125 ≈ 1.12 × cell)
    pub fn raster_px_for_cell(&self, cell: f32) -> f32 {
        cell * CAP_PER_CELL / self.cap_ratio.max(0.1)
    }

    /// baseline (px) below the cell top — centers the cap window in
    /// the cell (cap = 0.875 × cell for Monocraft)
    pub fn baseline_for_cell(&self, cell: f32) -> f32 {
        let cap = self.cap_ratio * self.raster_px_for_cell(cell);
        (cell + cap) / 2.0
    }

    /// does the TTF cmap cover this char? (glyph 0 = .notdef)
    fn has_glyph(&self, c: char) -> bool {
        c != '\0' && self.font.glyph_id(c) != ab_glyph::GlyphId(0)
    }

    /// ink-width ratio (against the raster height), measured once per
    /// char at the canonical raster
    fn ink_ratio(&mut self, c: char) -> f32 {
        if let Some(&r) = self.ink.get(&c) {
            return r;
        }
        let r = if c == ' ' {
            0.0
        } else if self.has_glyph(c) {
            self.outline_coverage(c, CANON_H)
                .map(|(bytes, w, _)| {
                    let stride = w as usize;
                    let rows = bytes.len() / (stride * 4);
                    let mut ink = 0usize;
                    'cols: for x in (0..stride).rev() {
                        for y in 0..rows {
                            if bytes[(y * stride + x) * 4 + 3] != 0 {
                                ink = x + 1;
                                break 'cols;
                            }
                        }
                    }
                    ink as f32 / CANON_H
                })
                .unwrap_or(0.0)
        } else {
            // bitmap fallback: ink at the 8-px reference cell
            let slot = crate::ui::smallcaps_slot(c);
            crate::ui::glyph_ink_width(&crate::ui::FONT[slot]) as f32 / REF_CELL
        };
        self.ink.insert(c, r);
        r
    }

    /// per-character advance (UI px) at a UI cell size — the vanilla
    /// proportional rule `ink + cell/8`; the space stays fixed at
    /// `cell / 2` (Phase 5's verified behavior)
    pub fn advance(&mut self, c: char, cell: f32) -> f32 {
        if c == ' ' {
            return cell * 0.5;
        }
        if self.has_glyph(c) {
            let ink = self.ink_ratio(c) * self.raster_px_for_cell(cell);
            ink + cell * SPACING_PER_CELL
        } else {
            // bitmap fallback keeps the bitmap font's exact rule
            let slot = crate::ui::smallcaps_slot(c);
            let w = crate::ui::glyph_ink_width(&crate::ui::FONT[slot]) as f32;
            (w + 1.0) * cell / REF_CELL
        }
    }

    /// measured width of `s` (UI px) at a UI cell size
    pub fn measure(&mut self, s: &str, cell: f32) -> f32 {
        s.chars().map(|c| self.advance(c, cell)).sum()
    }

    // ------------------------------------------------------ rasterize --

    /// rasterize `c`'s outline at font height `h`, baseline at the
    /// origin. Returns (RGBA bytes, w, h) — white × coverage.
    fn outline_coverage(&self, c: char, h: f32) -> Option<(Vec<u8>, u32, u32)> {
        let glyph = Glyph {
            id: self.font.glyph_id(c),
            scale: PxScale::from(h),
            position: ab_glyph::point(0.0, 0.0),
        };
        let outlined = self.font.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();
        let w = bounds.width().max(0.0).ceil() as u32;
        let hgt = bounds.height().max(0.0).ceil() as u32;
        if w == 0 || hgt == 0 {
            return None;
        }
        let mut bytes = vec![0u8; (w * hgt * 4) as usize];
        outlined.draw(|x, y, coverage| {
            let i = ((y * w + x) * 4) as usize;
            if i + 3 < bytes.len() {
                bytes[i] = 255;
                bytes[i + 1] = 255;
                bytes[i + 2] = 255;
                bytes[i + 3] = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        });
        Some((bytes, w, hgt))
    }

    /// the raw raster for a char at a DEVICE cell size: cropped to
    /// the ink bbox (the proportionalization), with the ink top
    /// relative to the baseline. Uncached, no atlas writes — the
    /// shared core of `rasterize` and `bake_bitmap`.
    fn glyph_bitmap(&mut self, c: char, cell_dev: u32) -> Option<(Vec<u8>, u32, u32, f32)> {
        if c == ' ' {
            return None;
        }
        if self.has_glyph(c) {
            self.rasterize_ttf(c, cell_dev)
        } else {
            self.rasterize_bitmap(c, cell_dev)
        }
    }

    /// TTF path: rasterize at device px, crop to the ink bbox, report
    /// the cropped ink top vs the baseline.
    fn rasterize_ttf(&mut self, c: char, cell_dev: u32) -> Option<(Vec<u8>, u32, u32, f32)> {
        let h = self.raster_px_for_cell(cell_dev as f32);
        let glyph = Glyph {
            id: self.font.glyph_id(c),
            scale: PxScale::from(h),
            position: ab_glyph::point(0.0, 0.0),
        };
        let outlined = self.font.outline_glyph(glyph)?;
        let bounds = outlined.px_bounds();
        let w = bounds.width().max(0.0).ceil() as u32;
        let hgt = bounds.height().max(0.0).ceil() as u32;
        if w == 0 || hgt == 0 {
            return None;
        }
        let mut bytes = vec![0u8; (w * hgt * 4) as usize];
        outlined.draw(|x, y, coverage| {
            let i = ((y * w + x) * 4) as usize;
            if i + 3 < bytes.len() {
                bytes[i] = 255;
                bytes[i + 1] = 255;
                bytes[i + 2] = 255;
                bytes[i + 3] = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        });
        // ink bbox scan
        let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0usize, usize::MAX, 0usize);
        for y in 0..hgt as usize {
            for x in 0..w as usize {
                if bytes[(y * w as usize + x) * 4 + 3] != 0 {
                    x0 = x0.min(x);
                    x1 = x1.max(x + 1);
                    y0 = y0.min(y);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x0 == usize::MAX {
            return None; // blank glyph
        }
        let cw = (x1 - x0) as u32;
        let ch = (y1 - y0) as u32;
        let mut out = vec![0u8; (cw * ch * 4) as usize];
        for y in 0..ch as usize {
            for x in 0..cw as usize {
                let si = ((y0 + y) * w as usize + x0 + x) * 4;
                let di = (y * cw as usize + x) * 4;
                out[di..di + 4].copy_from_slice(&bytes[si..si + 4]);
            }
        }
        // bounds.min.y is the glyph top vs the baseline (≤ 0 above it)
        Some((out, cw, ch, bounds.min.y + y0 as f32))
    }

    /// bitmap fallback: the builtin clean-room 5×7 glyph,
    /// nearest-upscaled to the device cell (solid alpha — pixel art),
    /// cap top aligned with the TTF cap top.
    fn rasterize_bitmap(&mut self, c: char, cell_dev: u32) -> Option<(Vec<u8>, u32, u32, f32)> {
        let slot = crate::ui::smallcaps_slot(c);
        let glyph = &FONT[slot];
        let (mut x0, mut x1) = (usize::MAX, 0usize);
        for gx in 0..5usize {
            if glyph.iter().any(|row| row & (1 << (4 - gx)) != 0) {
                x0 = x0.min(gx);
                x1 = x1.max(gx + 1);
            }
        }
        if x0 == usize::MAX {
            return None;
        }
        let scale = (cell_dev as f32 / REF_CELL).max(1.0);
        let step = scale.ceil() as u32;
        let cw = (((x1 - x0) as f32 * scale).ceil() as u32).max(1);
        let ch = ((8.0 * scale).ceil() as u32).max(1);
        let cap_dev = self.cap_ratio * self.raster_px_for_cell(cell_dev as f32);
        let mut out = vec![0u8; (cw * ch * 4) as usize];
        for gy in 0..8u32 {
            for gx in 0..5u32 {
                if glyph[gy as usize] & (1 << (4 - gx)) != 0 {
                    for sy in 0..step {
                        for sx in 0..step {
                            let px = ((gx as f32 - x0 as f32) * scale).floor() as i32 + sx as i32;
                            let py = (gy as f32 * scale).floor() as i32 + sy as i32;
                            if px < 0 || py < 0 || px >= cw as i32 || py >= ch as i32 {
                                continue;
                            }
                            let i = (py as usize * cw as usize + px as usize) * 4;
                            out[i] = 255;
                            out[i + 1] = 255;
                            out[i + 2] = 255;
                            out[i + 3] = 255;
                        }
                    }
                }
            }
        }
        Some((out, cw, ch, -cap_dev))
    }

    /// rasterize (or fetch from cache) `c` at a DEVICE cell size.
    /// `None` for ink-free chars (the space) — the advance carries
    /// the width.
    pub fn rasterize(&mut self, c: char, cell_dev: u32) -> Option<CachedGlyph> {
        if let Some(&g) = self.glyphs.get(&(c, cell_dev)) {
            return Some(g);
        }
        let (bytes, w, h, top_rel) = self.glyph_bitmap(c, cell_dev)?;
        // pack (reset + retry once when full)
        let (x, y) = match self.packer.alloc(w, h) {
            Some(p) => p,
            None => {
                // atlas full — Luanti-style cache reset, then re-pack
                self.glyphs.clear();
                self.runs.clear();
                self.packer.reset();
                self.resets += 1;
                self.version += 1;
                self.packer.alloc(w, h)?
            }
        };
        let g = CachedGlyph {
            atlas_x: x,
            atlas_y: y,
            w,
            h,
            top_rel_baseline: top_rel,
        };
        self.pending.push(PendingUpload {
            x,
            y,
            w,
            h,
            bytes,
        });
        self.version += 1;
        self.glyphs.insert((c, cell_dev), g);
        Some(g)
    }

    /// CPU text bitmap (RGBA, straight color) for splash-style
    /// compositors: bakes `s` at a UI cell size into a tight bitmap.
    pub fn bake_bitmap(&mut self, s: &str, cell: f32, color: [u8; 4]) -> (Vec<u8>, u32, u32) {
        let cell_dev = cell.max(2.0).round() as u32;
        let baseline = self.baseline_for_cell(cell_dev as f32);
        let mut placed: Vec<(f32, f32, u32, u32, Vec<u8>)> = Vec::new();
        let mut pen = 0f32;
        let mut top = f32::MAX;
        let mut bottom = f32::MIN;
        for c in s.chars() {
            if let Some((bytes, w, h, rel)) = self.glyph_bitmap(c, cell_dev) {
                let gy = baseline + rel;
                top = top.min(gy);
                bottom = bottom.max(gy + h as f32);
                placed.push((pen, gy, w, h, bytes));
            }
            pen += self.advance(c, cell_dev as f32);
        }
        if top == f32::MAX {
            return (Vec::new(), 0, 0);
        }
        let h = (bottom - top).ceil().max(1.0) as u32;
        let w = (pen.ceil() as u32).max(1);
        let mut out = vec![0u8; (w * h * 4) as usize];
        for (qx, qy, qw, qh, bytes) in placed {
            for y in 0..qh {
                for x in 0..qw {
                    let sx = qx as i32 + x as i32;
                    let sy = (qy - top) as i32 + y as i32;
                    if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                        continue;
                    }
                    let cov = bytes[(y * qw + x) as usize * 4 + 3];
                    let di = (sy as usize * w as usize + sx as usize) * 4;
                    out[di] = color[0];
                    out[di + 1] = color[1];
                    out[di + 2] = color[2];
                    out[di + 3] = (color[3] as u16 * cov as u16 / 255) as u8;
                }
            }
        }
        (out, w, h)
    }

    /// Cache a pre-baked RUN strip (arbitrary RGBA — the splash's
    /// colored outlined run) in the glyph atlas under `key`, sized
    /// `w × h` DEVICE px. `bake` only runs on a cache miss (or after
    /// an atlas reset re-cleared the strip) — it RECEIVES the engine
    /// (`&mut FontEngine`) so it can call `bake_bitmap` without
    /// fighting the outer `&mut self` borrow; the per-frame cost of a
    /// hit is one HashMap lookup. Returns the atlas rect for a
    /// rotated quad. `None` only when the strip cannot pack (larger
    /// than the atlas) or bakes empty.
    pub fn cache_run(
        &mut self,
        key: &str,
        cell_dev: u32,
        bake: impl FnOnce(&mut Self) -> (Vec<u8>, u32, u32),
    ) -> Option<CachedGlyph> {
        if let Some(&g) = self.runs.get(&(key.to_string(), cell_dev)) {
            return Some(g);
        }
        let (bytes, w, h) = bake(self);
        if w == 0 || h == 0 || bytes.len() != (w as usize) * (h as usize) * 4 {
            return None;
        }
        let (x, y) = match self.packer.alloc(w, h) {
            Some(p) => p,
            None => {
                // atlas full — the same Luanti-style reset as glyphs
                // (stale run rects are cleared with everything else);
                // if it still cannot pack it is bigger than a fresh
                // atlas — bail with None
                self.glyphs.clear();
                self.runs.clear();
                self.packer.reset();
                self.resets += 1;
                self.version += 1;
                self.packer.alloc(w, h)?
            }
        };
        let g = CachedGlyph {
            atlas_x: x,
            atlas_y: y,
            w,
            h,
            top_rel_baseline: 0.0, // run strips position by center, not baseline
        };
        self.pending.push(PendingUpload { x, y, w, h, bytes });
        self.version += 1;
        self.runs.insert((key.to_string(), cell_dev), g);
        Some(g)
    }

    // -------------------------------------------------------- upkeep --

    /// pending uploads since the last drain (renderer-side sync)
    pub fn take_pending(&mut self) -> Vec<PendingUpload> {
        std::mem::take(&mut self.pending)
    }

    /// monotonically increasing atlas version (renderer sync check)
    pub fn version(&self) -> u64 {
        self.version
    }

    /// how many times the atlas reset (full → cleared)
    pub fn resets(&self) -> u32 {
        self.resets
    }
}

// ------------------------------------------------------------ global --

static ENGINE: OnceLock<Option<Mutex<FontEngine>>> = OnceLock::new();

/// the process-wide font engine (`None` only if the embedded bytes
/// fail to parse — callers keep the bitmap-font canvas path then)
pub fn engine() -> Option<&'static Mutex<FontEngine>> {
    ENGINE
        .get_or_init(|| FontEngine::new(MONOCRAFT_TTF).map(Mutex::new))
        .as_ref()
}

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_loads_the_embedded_font() {
        let e = engine().expect("embedded Monocraft parses");
        let e = e.lock().unwrap();
        assert!(
            e.has_glyph('A') && e.has_glyph('a') && e.has_glyph('∞'),
            "Latin + infinity covered"
        );
        assert!(!e.has_glyph('\u{4E9C}'), "CJK falls back to bitmap");
        // cap ratio measured from 'H' ≈ 840/1080 em (ttf-parser reads
        // the OS/2 verticals: ascender 960 / descender -120)
        assert!((e.cap_ratio - 0.78125).abs() < 0.02, "cap ratio {}", e.cap_ratio);
    }

    #[test]
    fn proportional_advances_pack_narrow_letters_tighter() {
        let e = engine().unwrap();
        let mut e = e.lock().unwrap();
        let cell = 16.0;
        let aw = e.advance('W', cell);
        let ai = e.advance('i', cell);
        let ad = e.advance('.', cell);
        assert!(
            aw > ai && ai > ad,
            "W {aw} > i {ai} > . {ad} — ink-trim proportionalization"
        );
        // the space keeps the vanilla fixed advance: cell / 2
        assert_eq!(e.advance(' ', cell), cell * 0.5);
        // advance = ink + cell/8 for a TTF glyph
        let ink_w = e.ink_ratio('W') * e.raster_px_for_cell(cell);
        assert!((aw - ink_w - cell / 8.0).abs() < 0.01, "ink + cell/8");
    }

    #[test]
    fn measure_sums_the_per_char_advances() {
        let e = engine().unwrap();
        let mut e = e.lock().unwrap();
        for (s, cell) in [("VoxelCraft", 16.0), ("a i.", 12.0), ("", 24.0)] {
            let m = e.measure(s, cell);
            let sum: f32 = s.chars().map(|c| e.advance(c, cell)).sum();
            assert_eq!(m, sum, "{s:?}");
        }
        assert_eq!(e.measure("", 16.0), 0.0);
    }

    #[test]
    fn rasterize_crops_to_ink_and_reports_the_top() {
        let e = engine().unwrap();
        let mut e = e.lock().unwrap();
        let g = e.rasterize('A', 24).expect("A rasterizes");
        assert!(g.w > 0 && g.h > 0);
        // cap ≈ 0.875 × cell = 21 px at cell 24
        assert!(g.h <= 23, "cropped height {} ≤ 23", g.h);
        // top_rel_baseline ≤ 0 (the cap rises above the baseline)
        assert!(g.top_rel_baseline <= 0.0);
        // the pending upload carries the cropped byte count
        let p = e.take_pending();
        let mine = p.iter().find(|p| p.x == g.atlas_x && p.y == g.atlas_y);
        assert!(mine.is_some(), "rasterize queued an upload");
        assert_eq!(mine.unwrap().bytes.len(), (g.w * g.h * 4) as usize);
        // second rasterize is a cache hit — no new upload
        let before = e.version();
        let g2 = e.rasterize('A', 24).expect("cached");
        assert_eq!((g2.atlas_x, g2.atlas_y), (g.atlas_x, g.atlas_y));
        assert!(e.take_pending().is_empty());
        assert_eq!(e.version(), before, "cache hit does not bump the version");
    }

    #[test]
    fn shelf_packer_never_overlaps_and_pads() {
        let mut p = ShelfPacker::new(64, 64);
        let a = p.alloc(18, 10).unwrap();
        let b = p.alloc(18, 10).unwrap();
        let c = p.alloc(18, 10).unwrap();
        // same row: x advances by w + 2 (padding both sides)
        assert_eq!((a.0, b.0, c.0), (1, 21, 41));
        assert_eq!(a.1, b.1);
        // fill the rest — then the atlas fills up
        let mut n = 0;
        while p.alloc(18, 10).is_some() {
            n += 1;
            assert!(n < 64, "infinite loop guard");
        }
        p.reset();
        assert!(p.alloc(20, 10).is_some(), "reset frees everything");
    }

    #[test]
    fn atlas_full_resets_and_reports_it() {
        // a LOCAL engine (not the global) — this test wrecks the packer
        // and must not interleave with the shared-instance tests
        let mut e = FontEngine::new(MONOCRAFT_TTF).expect("parses");
        // a wide-but-short packer: the glyph fits the width, never the
        // height → the reset path runs, rasterize still fails cleanly
        e.packer = ShelfPacker::new(64, 8);
        let before = e.resets();
        assert!(e.rasterize('A', 24).is_none(), "unrasterizable when tiny");
        assert_eq!(e.resets(), before + 1, "one reset counted");
    }

    #[test]
    fn missing_chars_fall_back_to_the_bitmap_glyph() {
        let e = engine().unwrap();
        let mut e = e.lock().unwrap();
        let c = '\u{4E9C}'; // CJK — not in Monocraft
        let g = e.rasterize(c, 16).expect("bitmap fallback rasterizes");
        assert!(g.w > 0 && g.h > 0, "fallback has ink");
        // the fallback advance matches the bitmap font's Phase-5 rule
        let slot = crate::ui::smallcaps_slot(c);
        let w = crate::ui::glyph_ink_width(&crate::ui::FONT[slot]) as f32;
        let expect = (w + 1.0) * 16.0 / REF_CELL;
        assert!((e.advance(c, 16.0) - expect).abs() < 0.01);
    }

    #[test]
    fn bake_bitmap_produces_a_tight_colored_run() {
        let e = engine().unwrap();
        let mut e = e.lock().unwrap();
        let (bytes, w, h) = e.bake_bitmap("Hi!", 16.0, [255, 255, 0, 255]);
        assert!(w > 0 && h > 0);
        assert_eq!(bytes.len(), (w * h * 4) as usize);
        // yellow with coverage-derived alpha
        let ink_px = bytes
            .chunks(4)
            .filter(|c| c[3] > 0)
            .map(|c| (c[0], c[1], c[2]))
            .collect::<std::collections::HashSet<_>>();
        assert!(ink_px.contains(&(255, 255, 0)));
        assert!(!ink_px.is_empty());
    }

    #[test]
    fn height_and_baseline_map_the_vanilla_proportions() {
        // raster px ≈ 1.12 × cell (0.875 / 0.78125); the cap lands at
        // 0.875 × cell; the baseline centers the cap window in the cell
        let e = engine().unwrap();
        let e = e.lock().unwrap();
        let px = e.raster_px_for_cell(16.0);
        assert!((px - 17.92).abs() < 0.5, "raster px at cell 16 (got {px})");
        let cap = e.cap_ratio * px;
        assert!((cap - 14.0).abs() < 0.5, "cap ≈ 0.875 × cell (got {cap})");
        let b = e.baseline_for_cell(16.0);
        assert!((b - 15.0).abs() < 0.75, "baseline ≈ 15 at cell 16 (got {b})");
    }

    // ---- Luanti round 2: cached run strips (the splash) ------------

    #[test]
    fn cache_run_bakes_once_and_hits_thereafter() {
        // two calls with the same key+cell: ONE bake (closure count 1),
        // the SAME atlas rect back, and the second call adds no pending
        // upload (the renderer sync already drained the first)
        let eng = engine().expect("engine");
        let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
        let bakes = std::cell::Cell::new(0u32);
        let strip = |e: &mut FontEngine| -> (Vec<u8>, u32, u32) {
            bakes.set(bakes.get() + 1);
            e.bake_bitmap("Hi!", 16.0, [255, 255, 0, 255])
        };
        let g1 = e.cache_run("t1", 16, strip).expect("packs");
        let pending1 = e.take_pending();
        assert_eq!(pending1.len(), 1, "one upload on the miss");
        assert_eq!(pending1[0].w, g1.w);
        let g2 = e.cache_run("t1", 16, strip).expect("cached");
        assert_eq!(bakes.get(), 1, "bake ran exactly once");
        assert_eq!(
            (g1.atlas_x, g1.atlas_y, g1.w, g1.h),
            (g2.atlas_x, g2.atlas_y, g2.w, g2.h),
            "stable rect on the hit"
        );
        assert!(e.take_pending().is_empty(), "no upload on the hit");
    }

    #[test]
    fn cache_run_keys_on_cell_and_rejects_empty() {
        // a different cell is a different entry (device-scale change →
        // re-bake at the new resolution); an empty bake returns None
        let eng = engine().expect("engine");
        let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
        let a = e
            .cache_run("k", 16, |e| e.bake_bitmap("A", 16.0, [255, 255, 0, 255]))
            .expect("16 packs");
        let b = e
            .cache_run("k", 24, |e| e.bake_bitmap("A", 24.0, [255, 255, 0, 255]))
            .expect("24 packs");
        assert_ne!((a.w, a.h), (b.w, b.h), "cell-keyed: 24-cell run is larger");
        let none = e.cache_run("empty", 16, |_| (Vec::new(), 0, 0));
        assert!(none.is_none(), "empty bake rejected");
        let bad = e.cache_run("bad", 16, |_| (vec![9u8; 10], 2, 2));
        assert!(bad.is_none(), "byte-length mismatch rejected (10 != 2*2*4)");
    }
}

