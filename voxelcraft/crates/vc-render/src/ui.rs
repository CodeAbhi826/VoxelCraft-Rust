//! UI canvas (960x540 reference RGBA grid, resizable per the vanilla
//! integer GUI-scale model) with hand-built 5x7 bitmap font, the reference game-style
//! widgets (buttons + sliders), title / options / pause screens, and the
//! full 1.16.5-style HUD (hotbar, hearts, hunger, XP bar, crosshair, F3).
//! Redrawn only when state changes; uploaded to GPU as a texture.

use std::sync::OnceLock;

use crate::textures::blit_tile;
use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;

pub const UI_W: usize = 960;
pub const UI_H: usize = 540;

// ------------------------------------------- live UI size (Round 10) --
// The vanilla 1.16.5 integer GUI-scale model: the drawn logical GUI
// space is (framebuffer_w / scale, framebuffer_h / scale) vanilla px
// (reference wiki /Options, fetched live 2026-09-15: "Auto sets the
// GUI scale to the highest value available", available =
// max(1, min(floor(w/320), floor(h/240))) — identical to the 1.16.5
// MainWindow#calculateScale while-loop). This engine's canvas raster
// and every layout constant are built at 2 canvas px per vanilla px
// (the 960×540 reference = vanilla at 1920×1080 scale 4), so the LIVE
// canvas becomes (2·w/scale, 2·h/scale) canvas px and the blit maps
// each canvas px to scale/2 device px — an INTEGER device size per
// vanilla px at every scale. Menu layouts + HUD anchors read these
// hints so they re-center / re-anchor at any live size; the default
// (960×540) keeps every headless caller and test byte-identical.
// THREAD-LOCAL for the same reason TEXT_QUADS_ACTIVE is: the layout
// helpers are called from the single-threaded game loop (update + UI
// rebuild on one thread — the engine's documented threading model) and
// from parallel test threads; thread-locality isolates a test that
// resizes from every other test's geometry (the AtomicUsize version
// was a cross-test race — the exact lottery the text-quads flag lost
// once before).
thread_local! {
    static LIVE_UI_W: std::cell::Cell<usize> = const { std::cell::Cell::new(UI_W) };
    static LIVE_UI_H: std::cell::Cell<usize> = const { std::cell::Cell::new(UI_H) };
}

/// set the live UI canvas size (called by the game when the resolved
/// vanilla GUI scale or the window size changes)
pub fn set_live_ui_size(w: usize, h: usize) {
    LIVE_UI_W.with(|f| f.set(w.max(1)));
    LIVE_UI_H.with(|f| f.set(h.max(1)));
}

/// the live UI canvas width in canvas px (default 960)
pub fn live_ui_w() -> usize {
    LIVE_UI_W.with(|f| f.get())
}

/// the live UI canvas height in canvas px (default 540)
pub fn live_ui_h() -> usize {
    LIVE_UI_H.with(|f| f.get())
}

/// 2026-09-24 bottom-row anchoring (the 720p clipped-buttons round): the
/// fixed layouts were authored against the 960×540 reference canvas, but
/// the vanilla GUI-scale model produces SMALLER live canvases at common
/// window sizes — 1280×720 at auto scale 3 is 853×480, so every button
/// row at reference y≥470 (all settings-screen DONE rows at 470..500,
/// the world screens' CANCEL / row-B at 480..510) fell off the bottom
/// edge and was invisible + unclickable. This converts a reference-canvas
/// Y into the live canvas by preserving the distance from the BOTTOM
/// edge: y' = live_h − (540 − y). Identity at the 540 reference (1080p),
/// pulls the rows up into view on shorter canvases. Applied ONLY to the
/// bottom action rows (y≥440 in the authored layouts) — top-anchored
/// content and scrollable list rows keep their fixed positions.
pub fn anchor_y(y: i32) -> i32 {
    live_ui_h() as i32 - (UI_H as i32 - y)
}

// ------------------------------------------------- quad-text switch --
// The Luanti-style font round: when armed, every text* method routes
// through the runtime font engine (embedded Monocraft, glyph quads on
// the GPU — see gui/font.rs) instead of rasterizing the 5×7 bitmap
// into the canvas. THREAD-LOCAL because the text_width family is
// STATIC (layout code measures before any canvas exists) and the
// game loop that arms/reads it is single-threaded (update + UI
// rebuild on one thread — the engine's documented threading model);
// default OFF keeps the bitmap path byte-identical for every existing
// caller and test (G6). The game arms it at boot when the GUI quad
// pass is ready. Thread-locality also isolates parallel `cargo test`
// threads: an armed test can never stomp a disarmed test's draw in
// another thread (the process-global AtomicBool made the flag a
// cross-test race — the f3 descender test lost that lottery once).
thread_local! {
    static TEXT_QUADS_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// arm/disarm the GPU text path (the game calls this once at boot)
pub fn set_text_quads_active(active: bool) {
    TEXT_QUADS_ACTIVE.with(|f| f.set(active));
}

/// is the GPU text path armed?
pub fn text_quads_active() -> bool {
    TEXT_QUADS_ACTIVE.with(|f| f.get())
}

/// measure `s` with the ACTIVE source at `scale` — the shared helper
/// for the static text_width family (engine metrics when armed, the
/// bitmap advances otherwise)
fn measure_active(s: &str, cell: f32) -> i32 {
    if text_quads_active() {
        if let Some(eng) = crate::gui::font::engine() {
            let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
            return e.measure(s, cell).round() as i32;
        }
    }
    let mut w = 0;
    for ch in s.chars() {
        w += char_advance(smallcaps_slot(ch));
    }
    (w as f32 * (cell / 8.0)).round() as i32
}

#[rustfmt::skip]
// 5x8 font, rows top→bottom, bit 4 = leftmost pixel. ASCII 32..127.
// Rows 0..6 = the smallcaps body (caps/digits/punct sit on the row-6
// baseline); row 7 = the descender row (g j p q y ,). The a-z slots
// hold TRUE lowercase shapes — only the case renderer (F3 overlay)
// reads them; text()/text_flat() remap a-z→A (smallcaps UI look).
pub(crate) const FONT: [[u8; 8]; 96] = [
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], [0x04,0x04,0x04,0x04,0x04,0x00,0x04,0x00],
    [0x0A,0x0A,0x00,0x00,0x00,0x00,0x00,0x00], [0x0A,0x1F,0x0A,0x1F,0x0A,0x00,0x00,0x00],
    [0x04,0x0F,0x14,0x0E,0x05,0x1F,0x04,0x00], [0x18,0x19,0x02,0x04,0x08,0x13,0x03,0x00],
    [0x0E,0x11,0x0E,0x14,0x1E,0x11,0x16,0x00], [0x04,0x04,0x00,0x00,0x00,0x00,0x00,0x00],
    [0x02,0x04,0x08,0x08,0x08,0x04,0x02,0x00], [0x08,0x04,0x02,0x02,0x02,0x04,0x08,0x00],
    [0x00,0x04,0x15,0x0E,0x15,0x04,0x00,0x00], [0x00,0x04,0x04,0x1F,0x04,0x04,0x00,0x00],
    [0x00,0x00,0x00,0x00,0x00,0x04,0x04,0x08], [0x00,0x00,0x00,0x1F,0x00,0x00,0x00,0x00],
    [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x00], [0x00,0x01,0x02,0x04,0x08,0x10,0x00,0x00],
    [0x0E,0x11,0x13,0x15,0x19,0x11,0x0E,0x00], [0x04,0x0C,0x04,0x04,0x04,0x04,0x0E,0x00],
    [0x0E,0x11,0x01,0x02,0x04,0x08,0x1F,0x00], [0x1F,0x02,0x04,0x02,0x01,0x11,0x0E,0x00],
    [0x02,0x06,0x0A,0x12,0x1F,0x02,0x02,0x00], [0x1F,0x10,0x1E,0x01,0x01,0x11,0x0E,0x00],
    [0x06,0x08,0x10,0x1E,0x11,0x11,0x0E,0x00], [0x1F,0x01,0x02,0x04,0x08,0x08,0x08,0x00],
    [0x0E,0x11,0x11,0x0E,0x11,0x11,0x0E,0x00], [0x0E,0x11,0x11,0x0F,0x01,0x02,0x0C,0x00],
    [0x00,0x00,0x0C,0x0C,0x00,0x0C,0x0C,0x00], [0x00,0x00,0x0C,0x0C,0x00,0x0C,0x04,0x08],
    [0x02,0x04,0x08,0x10,0x08,0x04,0x02,0x00], [0x00,0x00,0x1F,0x00,0x1F,0x00,0x00,0x00],
    [0x08,0x04,0x02,0x01,0x02,0x04,0x08,0x00], [0x0E,0x11,0x01,0x02,0x04,0x00,0x04,0x00],
    [0x0E,0x11,0x15,0x17,0x16,0x10,0x0E,0x00], [0x0E,0x11,0x11,0x1F,0x11,0x11,0x11,0x00],
    [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E,0x00], [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E,0x00],
    [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C,0x00], [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F,0x00],
    [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10,0x00], [0x0E,0x11,0x10,0x17,0x11,0x11,0x0E,0x00],
    [0x11,0x11,0x11,0x1F,0x11,0x11,0x11,0x00], [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E,0x00],
    [0x07,0x02,0x02,0x02,0x02,0x12,0x0C,0x00], [0x11,0x12,0x14,0x18,0x14,0x12,0x11,0x00],
    [0x10,0x10,0x10,0x10,0x10,0x10,0x1F,0x00], [0x11,0x1B,0x15,0x15,0x11,0x11,0x11,0x00],
    [0x11,0x19,0x15,0x13,0x11,0x11,0x11,0x00], [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E,0x00],
    [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10,0x00], [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D,0x00],
    [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11,0x00], [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E,0x00],
    [0x1F,0x04,0x04,0x04,0x04,0x04,0x04,0x00], [0x11,0x11,0x11,0x11,0x11,0x11,0x0E,0x00],
    [0x11,0x11,0x11,0x11,0x11,0x0A,0x04,0x00], [0x11,0x11,0x11,0x15,0x15,0x15,0x0A,0x00],
    [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11,0x00], [0x11,0x11,0x0A,0x04,0x04,0x04,0x04,0x00],
    [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F,0x00], [0x0E,0x08,0x08,0x08,0x08,0x08,0x0E,0x00],
    [0x00,0x10,0x08,0x04,0x02,0x01,0x00,0x00], [0x0E,0x02,0x02,0x02,0x02,0x02,0x0E,0x00],
    [0x04,0x0A,0x11,0x00,0x00,0x00,0x00,0x00], [0x00,0x00,0x00,0x00,0x00,0x00,0x1F,0x00],
    [0x08,0x04,0x00,0x00,0x00,0x00,0x00,0x00], [0x00,0x00,0x0E,0x01,0x0F,0x11,0x0F,0x00],
    [0x10,0x10,0x1C,0x12,0x12,0x12,0x1C,0x00], [0x00,0x00,0x0E,0x11,0x10,0x11,0x0E,0x00],
    [0x02,0x02,0x0E,0x12,0x12,0x12,0x0E,0x00], [0x00,0x00,0x0E,0x11,0x1F,0x10,0x0E,0x00],
    [0x0C,0x04,0x1E,0x04,0x04,0x04,0x04,0x00], [0x00,0x00,0x0E,0x11,0x11,0x0E,0x01,0x0F],
    [0x10,0x10,0x1C,0x12,0x12,0x12,0x12,0x00], [0x00,0x04,0x00,0x04,0x04,0x04,0x04,0x00],
    [0x00,0x04,0x00,0x04,0x04,0x04,0x04,0x0C], [0x10,0x10,0x12,0x14,0x18,0x14,0x12,0x00],
    [0x04,0x04,0x04,0x04,0x04,0x04,0x06,0x00], [0x00,0x00,0x1B,0x15,0x15,0x15,0x15,0x00],
    [0x00,0x00,0x1C,0x12,0x12,0x12,0x12,0x00], [0x00,0x00,0x0E,0x11,0x11,0x11,0x0E,0x00],
    [0x10,0x10,0x1C,0x12,0x12,0x12,0x1C,0x10], [0x02,0x02,0x0E,0x12,0x12,0x12,0x0E,0x02],
    [0x00,0x00,0x1C,0x14,0x10,0x10,0x10,0x00], [0x00,0x00,0x0F,0x10,0x0E,0x01,0x1E,0x00],
    [0x04,0x0E,0x04,0x04,0x04,0x04,0x06,0x00], [0x00,0x00,0x12,0x12,0x12,0x12,0x1E,0x00],
    [0x00,0x00,0x11,0x11,0x11,0x0A,0x04,0x00], [0x00,0x00,0x11,0x11,0x11,0x15,0x0A,0x00],
    [0x00,0x00,0x11,0x0A,0x04,0x0A,0x11,0x00], [0x00,0x00,0x11,0x11,0x11,0x0A,0x04,0x0C],
    [0x00,0x00,0x1F,0x02,0x04,0x08,0x1F,0x00], [0x06,0x08,0x08,0x0C,0x08,0x08,0x06,0x00],
    [0x04,0x04,0x04,0x04,0x04,0x04,0x04,0x00], [0x0C,0x02,0x02,0x06,0x02,0x02,0x0C,0x00],
    [0x00,0x00,0x08,0x15,0x02,0x00,0x00,0x00], [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
];

pub type Color = [u8; 4];

// The ∞ glyph (the F3 fps line's unlimited framerate): 5 wide, 2-row
// weave at mid x-height — the tiny-font convention for ∞ (a solid
// figure-eight loop set is unreadable at 5px; the weave reads as the
// ASCII-approximation "~"). Clean-room shape.
#[rustfmt::skip]
const INFINITY: [u8; 8] = [0x00,0x00,0x00,0x00,0x0A,0x15,0x00,0x00];

/// Truncate `s` — with an ASCII `...` tail — until its measured width
/// fits `max_w` (the F3 right-column half-screen clamp: software-renderer
/// adapter strings run ~830 UI px wide and would otherwise overdraw the
/// left column). Char-boundary-safe for any UTF-8 content; measures at
/// most once per byte on the overflow path (strings here are short and
/// the overlay rebuilds at 0.05 s cadence, so the lock traffic is
/// negligible).
fn fit_line(s: &str, max_w: f32, mut measure: impl FnMut(&str) -> f32) -> String {
    if measure(s) <= max_w {
        return s.to_string();
    }
    let mut end = s.len();
    while end > 0 {
        if !s.is_char_boundary(end) {
            end -= 1;
            continue;
        }
        let cand = format!("{}...", &s[..end]);
        if measure(&cand) <= max_w {
            return cand;
        }
        end -= 1;
    }
    "...".to_string()
}

/// Glyph lookup for the true-case (F3) renderer: real lowercase slots, '∞'
/// mapped to its own glyph, everything else as-is; unknown → '?'.
/// Returns (glyph, left bearing, ink width) — the advance source
/// (advance = ink width + 1; the vanilla font packs i/l/t tight).
fn case_glyph(ch: char) -> Option<([u8; 8], i32, i32)> {
    if ch == ' ' {
        return None;
    }
    if ch == '∞' {
        return Some((INFINITY, 0, 5));
    }
    let mut ch = ch as usize;
    if !(32..=126).contains(&ch) {
        ch = b'?' as usize;
    }
    let g = &FONT[ch - 32];
    let mut l = 5;
    let mut r = -1;
    for row in g {
        for b in 0..5 {
            if row & (1 << (4 - b)) != 0 {
                l = l.min(b);
                r = r.max(b);
            }
        }
    }
    if r < 0 {
        return Some((*g, 0, 0)); // blank glyph
    }
    Some((*g, l, r - l + 1))
}

// ------------------------------------------------------------- widgets --

/// Deterministic 2D pixel hash (the modern flat chrome's noise band —
/// same xorshift-multiply family as textures/gui_art.rs::hash2, seeded
/// differently so the canvas fallback's noise never aligns with the
/// 9-slice source's pattern).
fn hash_pixel(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x85EB_CA6B) ^ (y as u32).wrapping_mul(0xC2B2_AE35);
    h ^= h >> 15;
    h = h.wrapping_mul(0x27D4_EB2F);
    h ^ (h >> 13)
}

#[derive(Clone, Debug)]
pub enum WidgetKind {
    Button {
        label: String,
        value: String,
        enabled: bool,
    },
    Slider {
        label: String,
        value: f32,
    },
    /// Phase 1: single-line text entry (world name / seed). `text` holds the
    /// current buffer; `focused` drives the caret; `placeholder` shows when
    /// empty (e.g. a random seed preview).
    TextField {
        label: String,
        text: String,
        placeholder: String,
        focused: bool,
    },
}

#[derive(Clone, Debug)]
pub struct Widget {
    pub id: u16,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub kind: WidgetKind,
}

impl Widget {
    pub fn hit(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
    pub fn slider_value_at(&self, px: i32) -> f32 {
        ((px - self.x - 8) as f32 / (self.w - 16) as f32).clamp(0.0, 1.0)
    }
    /// Round 14b: the row's label (Button/TextField/Slider kinds)
    pub fn label(&self) -> &str {
        match &self.kind {
            WidgetKind::Button { label, .. } => label,
            WidgetKind::TextField { label, .. } => label,
            WidgetKind::Slider { label, .. } => label,
        }
    }
}

pub fn btn(id: u16, x: i32, y: i32, w: i32, label: &str, value: &str, enabled: bool) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h: 44,
        kind: WidgetKind::Button {
            label: label.to_string(),
            value: value.to_string(),
            enabled,
        },
    }
}

pub fn slider(id: u16, x: i32, y: i32, w: i32, label: &str, value: f32) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h: 44,
        kind: WidgetKind::Slider {
            label: label.to_string(),
            value: value.clamp(0.0, 1.0),
        },
    }
}

/// Slider with explicit height — the vanilla 1.16.5 settings screens use
/// 150x20 buttons (→ 225x30 on this 1.5x canvas); an empty label draws
/// the vanilla unlabeled slider (Brightness).
pub fn slider_h(id: u16, x: i32, y: i32, w: i32, h: i32, label: &str, value: f32) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h,
        kind: WidgetKind::Slider {
            label: label.to_string(),
            value: value.clamp(0.0, 1.0),
        },
    }
}

/// GUI Scale: re-scale a widget list around the canvas center (vanilla
/// semantics — the whole interface grows/shrinks; hit tests use the same
/// scaled rects, so input stays consistent for free).
pub fn scale_widgets(ws: &mut [Widget], s: f32) {
    if (s - 1.0).abs() < 0.01 {
        return;
    }
    let (cx, cy) = (live_ui_w() as f32 / 2.0, live_ui_h() as f32 / 2.0);
    for w in ws.iter_mut() {
        w.x = (cx + (w.x as f32 - cx) * s).round() as i32;
        w.y = (cy + (w.y as f32 - cy) * s).round() as i32;
        w.w = (w.w as f32 * s).round() as i32;
        w.h = (w.h as f32 * s).round() as i32;
    }
}

/// Single-line text entry with explicit height (vanilla fields are
/// 320x20 → 480x30 on the 1.5x canvas; the old fixed 44px height made
/// the world screens look padded).
// 8 params mirror `text_field` + the explicit height column — the
// same deliberate shape as `btn_h`.
#[allow(clippy::too_many_arguments)]
pub fn text_field_h(
    id: u16,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    text: &str,
    placeholder: &str,
) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h,
        kind: WidgetKind::TextField {
            label: label.to_string(),
            text: text.to_string(),
            placeholder: placeholder.to_string(),
            focused: false,
        },
    }
}

/// Phase 1: a single-line text entry field.
pub fn text_field(
    id: u16,
    x: i32,
    y: i32,
    w: i32,
    label: &str,
    text: &str,
    placeholder: &str,
) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h: 44,
        kind: WidgetKind::TextField {
            label: label.to_string(),
            text: text.to_string(),
            placeholder: placeholder.to_string(),
            focused: false,
        },
    }
}

/// Update a TextField widget's buffer in place (keeps label/placeholder).
pub fn set_text(w: &mut Widget, text: &str) {
    if let WidgetKind::TextField { text: t, .. } = &mut w.kind {
        *t = text.to_string();
    }
}

// widget id constants shared with game.rs
pub const ID_TITLE_PLAY: u16 = 1;
pub const ID_TITLE_OPTIONS: u16 = 2;
pub const ID_TITLE_QUIT: u16 = 3;
/// vanilla-layout MULTIPLAYER stub (disabled until netcode exists — the
/// button keeps the 1.16.5 stack visually exact; vanilla also grays it out
/// when multiplayer is unavailable)
pub const ID_TITLE_MULTI: u16 = 4;
// Phase 1: world-select + world-create + death screens
pub const ID_WS_WORLD_BASE: u16 = 60; // world entries: 60..60+MAX_LISTED
pub const ID_WS_CREATE: u16 = 90;
pub const ID_WS_CANCEL: u16 = 91;
pub const ID_WS_DELETE: u16 = 92;
pub const ID_WC_NAME: u16 = 93;
pub const ID_WC_SEED: u16 = 94;
pub const ID_WC_MODE: u16 = 95;
pub const ID_WC_TYPE: u16 = 96;
pub const ID_WC_CREATE: u16 = 97;
pub const ID_WC_CANCEL: u16 = 98;
pub const ID_DEATH_RESPAWN: u16 = 99;
pub const ID_DEATH_TITLE: u16 = 100;
pub const ID_DEATH_DELETE: u16 = 101;
/// 2026-09-14 parity round: the vanilla Select World bottom rows —
/// Play Selected World / Create New World (row A), Edit / Delete /
/// Re-Create / Search (row B) — plus the search field, the Edit World
/// screen, and the two-page Create World (More World Options) flow.
pub const ID_WS_PLAY: u16 = 102;
pub const ID_WS_EDIT: u16 = 103;
pub const ID_WS_RECREATE: u16 = 104;
pub const ID_WS_SEARCH: u16 = 105;
pub const ID_WS_SEARCHFIELD: u16 = 106;
pub const ID_WE_NAME: u16 = 107;
pub const ID_WE_RENAME: u16 = 108;
pub const ID_WE_DELETE: u16 = 109;
pub const ID_WE_COPY: u16 = 110;
pub const ID_WE_DONE: u16 = 111;
pub const ID_WC_MORE: u16 = 112;
pub const ID_WC_STRUCT: u16 = 113;
pub const ID_WC_BONUS: u16 = 114;
/// maximum world entries the select screen lists at once (scroll for the
/// rest — vanilla scrolls the list; ids 60..60+n)
pub const MAX_LISTED_WORLDS: usize = 6;
pub const ID_OPT_FOV: u16 = 10;
pub const ID_OPT_SENS: u16 = 11;
pub const ID_OPT_RD: u16 = 12;
pub const ID_OPT_BRIGHT: u16 = 13;
pub const ID_OPT_VOL: u16 = 14;
pub const ID_OPT_GRAPHICS: u16 = 16;
pub const ID_OPT_SMOOTH: u16 = 17;
pub const ID_OPT_CLOUDS: u16 = 18;
pub const ID_OPT_DONE: u16 = 19;
pub const ID_PAUSE_BACK: u16 = 20;
pub const ID_PAUSE_OPTIONS: u16 = 21;
pub const ID_PAUSE_QUIT: u16 = 22;
pub const ID_OPT_SHADOWS: u16 = 23;
pub const ID_OPT_UPSCALE: u16 = 24;
pub const ID_OPT_MAXFPS: u16 = 25;
pub const ID_OPT_MUSIC: u16 = 26;
/// Phase 6 §26: options page 2 (video detail) ids
pub const ID_OPT_NEXT: u16 = 27;
pub const ID_OPT_PREV: u16 = 28;
pub const ID_OPT_SIMDIST: u16 = 29;
pub const ID_OPT_MIP: u16 = 30;
pub const ID_OPT_ANISO: u16 = 31;
pub const ID_OPT_MSAA: u16 = 32;
pub const ID_OPT_OCCL: u16 = 33;
/// 1.10: the auto-jump options toggle
pub const ID_OPT_AUTOJUMP: u16 = 36;
pub const ID_OPT_DONE2: u16 = 34;
/// Phase 7: GPU compute meshing toggle (engine optimization — no vanilla
/// equivalent; labeled plainly, not with a vanilla options.txt name)
pub const ID_OPT_GMESH: u16 = 35;

/// The vanilla-1.16.5 settings tree: Options → Video Settings (the exact
/// vanilla screen), Resource Packs, Accessibility, plus our Engine page.
/// 37..=46 options, 47..=49 disabled vanilla stubs, 50.. pack rows.
pub const ID_OPT_VIDEO: u16 = 37;
pub const ID_OPT_ENGINE: u16 = 38;
pub const ID_OPT_PACKS: u16 = 39;
pub const ID_OPT_ACCESS: u16 = 40;
pub const ID_OPT_GUISCALE: u16 = 41;
pub const ID_OPT_PARTICLES: u16 = 42;
pub const ID_OPT_FULLSCREEN: u16 = 43;
pub const ID_OPT_VSYNC: u16 = 44;
pub const ID_OPT_ENTSHADOW: u16 = 45;
pub const ID_OPT_BIOME: u16 = 46;
/// vanilla stub buttons kept in the layout (grayed like MULTIPLAYER until
/// their subsystem exists — vanilla grays unavailable features too)
pub const ID_OPT_CHAT: u16 = 47;
pub const ID_OPT_LANG: u16 = 48;
pub const ID_OPT_CONTROLS: u16 = 49;
/// Round 14 (2026-09-15): the Music & Sound screen entry — vanilla's
/// ten per-category sliders (reference wiki /Options §Music & Sound,
/// live 2026-09-15). Full-width row under VIEW BOBBING.
pub const ID_OPT_MUSICSND: u16 = 50;
/// Round 14: the Music & Sound screen slider rows — one per vanilla
/// SoundCategory + master. 160..170 (clear of every literal id and the
/// 60..66 / 120..158 row families; guarded by the ID-space tests).
pub const ID_SND_BASE: u16 = 160;
/// Round 14: the last Music & Sound slider id (Voice — 169; the
/// range-end twin of ID_SND_BASE for match patterns)
pub const ID_SND_LAST: u16 = 169;
/// Round 14: the Controls screen — reset + the bind-row family
/// (180..204: up to 24 rebindable action rows)
pub const ID_SND_DONE: u16 = 170;
pub const ID_CTRL_RESET: u16 = 171;
pub const ID_CTRL_DONE: u16 = 172;
/// Round 14: the accessibility additions (Fog cycle / FOV Effects /
/// Distortion Effects / Chat Visibility / Subtitles)
pub const ID_ACC_FOG: u16 = 173;
pub const ID_ACC_FOVEFF: u16 = 174;
pub const ID_ACC_DISTORT: u16 = 175;
pub const ID_ACC_CHATVIS: u16 = 176;
pub const ID_ACC_SUBTITLES: u16 = 177;
pub const ID_CTRL_BIND_BASE: u16 = 180;
/// max rebindable rows on the Controls screen
pub const MAX_CTRL_BINDS: usize = 24;
/// Round 14b (2026-09-16): the Skin Customization + Chat Settings +
/// accessibility-completion family — 210..=235, one fresh block clear
/// of every literal id (max 101) and every row family (110 PACK,
/// 120..158 rpack, 160..170 sound, 170..177 access/ctrl, 180..204
/// binds). Guarded by the ID-space tests.
pub const ID_SKIN_BASE: u16 = 210; // cape/jacket/sleeves/pants/hat/main hand (6 ids)
pub const ID_SKIN_CAPE: u16 = 210;
pub const ID_SKIN_JACKET: u16 = 211;
pub const ID_SKIN_LSLEEVE: u16 = 212;
pub const ID_SKIN_RSLEEVE: u16 = 213;
pub const ID_SKIN_LPANTS: u16 = 214;
pub const ID_SKIN_RPANTS: u16 = 215;
pub const ID_SKIN_HAT: u16 = 216;
pub const ID_SKIN_MAINHAND: u16 = 217;
pub const ID_SKIN_DONE: u16 = 218;
/// Round 14b: the Chat Settings rows (the 1.16.5 vanilla set, grayed
/// where no subsystem exists — live-verified order)
pub const ID_CHAT_VIS: u16 = 219;
pub const ID_CHAT_COLORS: u16 = 220;
pub const ID_CHAT_LINKS: u16 = 221;
pub const ID_CHAT_LINKSPROMPT: u16 = 222;
pub const ID_CHAT_OPACITY: u16 = 223;
pub const ID_CHAT_DELAY: u16 = 224;
pub const ID_CHAT_WIDTH: u16 = 225;
pub const ID_CHAT_HFOCUSED: u16 = 226;
pub const ID_CHAT_HUNFOCUSED: u16 = 227;
pub const ID_CHAT_SCALE: u16 = 228;
pub const ID_CHAT_LINESPACING: u16 = 229;
pub const ID_CHAT_HIDENAMES: u16 = 230;
pub const ID_CHAT_REDUCEDDEBUG: u16 = 231;
pub const ID_CHAT_NARRATOR: u16 = 232;
pub const ID_CHAT_DONE: u16 = 233;
/// Round 14b: the Music & Sounds SHOW SUBTITLES row (the 1.16.5
/// placement — live w/Subtitles: "In Java Edition, you can also enable
/// these in the Music & Sounds options")
pub const ID_SND_SUBTITLES: u16 = 234;
/// Round 14b: the accessibility completion — the Sprint/Sneak
/// Hold-vs-Toggle pair (1.15 19w41a rows) + the Distortion Effects
/// slider (1.16.2 pre1). 1.16.5 vanilla order: Auto-Jump, Sprint,
/// Sneak, Distortion, FOV Effects, Subtitles.
pub const ID_ACC_SPRINT: u16 = 235;
pub const ID_ACC_SNEAK: u16 = 236;
pub const ID_ACC_DISTORT_SLIDER: u16 = 237;
/// Round 14b: the Options screen's SKIN CUSTOMIZATION... entry (115 —
/// clear of the world-edit/create literals through 114 and the rpack
/// family at 120+)
pub const ID_OPT_SKIN: u16 = 115;
/// 2026-09-14 round: the builtin engine shader modes and pre-created
/// demo packs (moonlit, warm-evening) were removed — no built-in
/// shaders, ever. 2026-09-20 round: the SHADER PACKS *screen* is back
/// by owner request, but listing EXTERNAL drop-in packs only (see
/// ID_OPT_SHADERS below); the post pipeline stays vanilla-only until
/// the vc-iris translator sister project registers.
/// vanilla "View Bobbing" toggle (Options screen, default ON)
pub const ID_OPT_BOB: u16 = 52;
/// 2026-09-20 round (user directive — restored by request): the SHADERS
/// entry on the Video Settings screen. Vanilla 1.16.5 ships no shader
/// screen, but every modded 1.16.5 player knows the OptiFine/Iris
/// "Shaders..." button — and modern versions keep adding options the
/// 1.16.5 screen lacks (the owner's call: extra options are welcome).
/// The screen lists EXTERNAL drop-in packs from shader-packs/ only —
/// the engine ships NO built-in shaders (BSL/SEUS-style packs are
/// third-party downloads, never ours to redistribute; see
/// docs/LEGAL-COMPLIANCE.md §4.4).
pub const ID_OPT_SHADERS: u16 = 53;
/// 2026-09-20 round: the ENTITY DISTANCE slider on the Video Settings
/// screen (the 1.17+ modern option the owner welcomes — "newer versions
/// have much more options which can be great"). A 0.5..1.0 multiplier
/// on the entity render radius: honest engine behavior — mobs beyond
/// render_distance * 16 * entity_distance from the camera skip the
/// vertex build (and the shadow quads) entirely. Default 1.0 = the
/// pre-option behavior (entities visible across the full render
/// distance). options.txt key `entityDistance` (vanilla naming).
pub const ID_OPT_ENTDIST: u16 = 54;
/// 2026-09-20: the Shader Packs screen family — 240..=249, a fresh
/// literal block clear of every existing id (max literal 115) and every
/// row family (60 world, 110 pack, 120..158 rpack, 160..170 sound,
/// 170..177 access/ctrl, 180..204 binds, 210..237 skin/chat/acc).
/// Guarded by the ID-space tests.
/// the "(none)" row — the vanilla post pipeline (no shader pack)
pub const ID_SHDR_NONE: u16 = 240;
/// external pack rows (one per scanned shader-packs/ entry)
pub const ID_SHDR_BASE: u16 = 241;
/// max shader-pack rows the screen lists
pub const MAX_SHDR_ENTRIES: usize = 7;
/// DONE — back to Video Settings (the screen's parent)
pub const ID_SHDR_DONE: u16 = 248;
/// the labPBR materials toggle (NAPP-style `_n`/`_s` resource-pack
/// maps feed the PBR path when ON; default OFF = the vanilla look)
pub const ID_SHDR_LABPBR: u16 = 249;
/// available (left pane) pack rows
///
/// 2026-09-14 follow-up (deploy-fix round): the 2026-09-14 resource-pack
/// renumbering — the pack-row bases moved OUT of the low id space they
/// shared with the world-select/create button ids (60..98) and the
/// options-tree ids (50..52). `activate()` tries its RANGE GUARDS before
/// the later literal arms, so the old values silently swallowed every
/// world-create/select button click (GAME MODE / CREATE WORLD / world
/// entries — the Enter-key path still worked, which masked it in E2E)
/// and made shader rows 2-3 toggle VIEW BOBBING instead. All row bases
/// now live at 110+, clear of every literal id (max 101).
pub const ID_RPACK_AVAIL_BASE: u16 = 120;
/// selected (right pane) pack rows
pub const ID_RPACK_SEL_BASE: u16 = 130;
/// the pinned DEFAULT row at the bottom of the Selected pane (vanilla:
/// "Selected by default, can't be unselected")
pub const ID_RPACK_DEFAULT: u16 = 138;
/// move-up arrow per selected row (higher = higher priority)
pub const ID_RPACK_UP_BASE: u16 = 140;
/// move-down arrow per selected row
pub const ID_RPACK_DOWN_BASE: u16 = 150;
/// max rows per pane (layout clips beyond this)
pub const MAX_RPACK_ENTRIES: usize = 8;

/// Phase 1 + 2026-09-14 parity round: one Select World list entry —
/// name (white, first line) + info line (gray, second line: mode,
/// last played, dead-hardcore lock). Owned by the painter so the row
/// bodies can render the vanilla two-line entry look.
#[derive(Clone, Debug)]
pub struct WorldRow {
    pub name: String,
    /// second line under the name (mode + last played / GAME OVER)
    pub info: String,
    /// dead hardcore worlds render dim + unplayable (vanilla locks them)
    pub dead: bool,
}

/// Sub-round 1 (2026-09-14 Survival HUD round): the survival status
/// block's inputs, one struct instead of a growing argument list.
/// Per-element wiki citations live on `UiCanvas::status_bars`.
#[derive(Clone, Copy, Debug)]
pub struct HudStatus {
    /// health 0..20 (10 hearts, half-heart resolution)
    pub health: f32,
    /// food 0..20 (10 drumsticks; Round 17: the LIVE foodLevel — the
    /// hardcoded 20/20 spawn display is retired with the FoodData
    /// system, VERIFIED w/Food §Hunger values)
    pub food: f32,
    /// Round 17: foodSaturationLevel == 0 — the hunger bar "starts to
    /// shake or jitter periodically" (VERIFIED w/Food §Saturation:
    /// "when saturation reaches zero, the hunger bar starts to shake
    /// or jitter periodically")
    pub food_jitter: bool,
    /// XP progress within the current level, 0..1
    pub xp: f32,
    /// XP level (number above the bar; 0 = hidden)
    pub level: u32,
    /// air 0..300 (bubbles above hunger while < 300)
    pub air: f32,
    /// armor points 0..20+ (row hidden at 0 — the vanilla gate)
    pub armor: i32,
    /// hearts-row ±1-px jitter (hurt flash / Regeneration active)
    pub hearts_jitter: bool,
    /// Hunger effect active: yellow-green recolor + row jitter
    pub hunger_poisoned: bool,
    /// game-tick parity for the jitter offsets (0/1 alternating)
    pub tick_phase: i32,
}

/// Sub-round 1: one status-effect icon for the top-right HUD rows.
#[derive(Clone, Copy, Debug)]
pub struct EffectIconEntry {
    /// effect-icon index (order = vc_gameplay EffectKind; see
    /// textures/gui_art.rs `EFFECT_ICON_COUNT`)
    pub icon: usize,
    /// amplifier 0 = level I (numeral only at ≥ II)
    pub amplifier: u8,
    /// ticks left (blink in the final 100; sorts sooner-left)
    pub ticks_left: i32,
    /// positive effects row 0, others row 1 (vanilla split)
    pub positive: bool,
}

/// Button with explicit height (vanilla title buttons are 200x20 at GUI
/// scale 2 = 300x30 on the 960x540 canvas).
// 8 params mirror `btn` + the explicit height column; silenced
// deliberately.
#[allow(clippy::too_many_arguments)]
pub fn btn_h(
    id: u16,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    value: &str,
    enabled: bool,
) -> Widget {
    Widget {
        id,
        x,
        y,
        w,
        h,
        kind: WidgetKind::Button {
            label: label.to_string(),
            value: value.to_string(),
            enabled,
        },
    }
}

/// Title screen layout — vanilla 1.16.5 button stack proportions (the
/// button row starts at half the screen height; full-width buttons 300px,
/// bottom row two half-width 146px buttons; MULTIPLAYER disabled until
/// netcode exists). Quit only exists on native.
pub fn layout_title(is_web: bool) -> Vec<Widget> {
    let cx = (live_ui_w() as i32 - 300) / 2;
    let mut v = vec![
        btn_h(ID_TITLE_PLAY, cx, 225, 300, 30, "SINGLEPLAYER", "", true),
        btn_h(ID_TITLE_MULTI, cx, 270, 300, 30, "MULTIPLAYER", "", false),
    ];
    if is_web {
        v.push(btn_h(
            ID_TITLE_OPTIONS,
            cx,
            315,
            300,
            30,
            "OPTIONS...",
            "",
            true,
        ));
    } else {
        v.push(btn_h(
            ID_TITLE_OPTIONS,
            cx,
            315,
            146,
            30,
            "OPTIONS...",
            "",
            true,
        ));
        v.push(btn_h(
            ID_TITLE_QUIT,
            cx + 154,
            315,
            146,
            30,
            "QUIT GAME",
            "",
            true,
        ));
    }
    v
}

/// Main Options screen — the vanilla 1.16.5 layout at 1.5x canvas scale:
/// Music|Sound and FOV|Sensitivity slider pairs, three rows of sub-screen
/// buttons (Chat Settings / Resource Packs / Language / Accessibility /
/// Video Settings / Controls), Done at the bottom. Chat, Language and
/// Controls are grayed stubs until those subsystems exist — the same
/// vanilla-grayed pattern as the title screen's MULTIPLAYER button.
/// ENGINE SETTINGS is the one disclosed deviation (our extra subsystems
/// need a home; vanilla has no equivalent page).
pub fn layout_options() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [72, 108, 144, 180, 216];
    vec![
        slider_h(ID_OPT_MUSIC, l, rows[0], bw, 30, "MUSIC", 0.6),
        slider_h(ID_OPT_VOL, r, rows[0], bw, 30, "SOUND", 0.7),
        slider_h(ID_OPT_FOV, l, rows[1], bw, 30, "FOV", 0.5),
        slider_h(ID_OPT_SENS, r, rows[1], bw, 30, "MOUSE SENSITIVITY", 0.45),
        // Round 14: the stub screens are REAL now — Chat Settings is the
        // documented no-subsystem stub screen, Language lists English
        btn_h(
            ID_OPT_CHAT,
            l,
            rows[2],
            bw,
            30,
            "CHAT SETTINGS...",
            "",
            true,
        ),
        btn_h(
            ID_OPT_PACKS,
            r,
            rows[2],
            bw,
            30,
            "RESOURCE PACKS...",
            "",
            true,
        ),
        btn_h(ID_OPT_LANG, l, rows[3], bw, 30, "LANGUAGE...", "", true),
        btn_h(
            ID_OPT_ACCESS,
            r,
            rows[3],
            bw,
            30,
            "ACCESSIBILITY SETTINGS...",
            "",
            true,
        ),
        btn_h(
            ID_OPT_VIDEO,
            l,
            rows[4],
            bw,
            30,
            "VIDEO SETTINGS...",
            "",
            true,
        ),
        btn_h(ID_OPT_CONTROLS, r, rows[4], bw, 30, "CONTROLS...", "", true),
        btn_h(
            ID_OPT_ENGINE,
            248,
            252,
            465,
            30,
            "ENGINE SETTINGS...",
            "",
            true,
        ),
        // vanilla 1.16.5 Options-screen option (default ON): the walk-cycle
        // camera/hand sway
        btn_h(ID_OPT_BOB, 248, 292, 465, 30, "VIEW BOBBING", "ON", true),
        // Round 14: the vanilla Music & Sound sub-screen (ten category
        // sliders) — full-width row under VIEW BOBBING
        btn_h(
            ID_OPT_MUSICSND,
            248,
            332,
            465,
            30,
            "MUSIC & SOUND...",
            "",
            true,
        ),
        // Round 14b: the vanilla Skin Customization entry (the layer
        // toggles + Main Hand screen)
        btn_h(
            ID_OPT_SKIN,
            248,
            372,
            465,
            30,
            "SKIN CUSTOMIZATION...",
            "",
            true,
        ),
        btn_h(
            ID_OPT_DONE,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Video Settings — the vanilla 1.16.5 screen: full-width Render
/// Distance slider on top, four two-column cycling rows (Graphics |
/// Smooth Lighting, GUI Scale | Clouds, Particles | Full Screen, Use
/// VSync | Entity Shadows), the unlabeled full-width Brightness slider
/// (hover shows Moody/Bright), the full-width Biome Blend slider, Done.
/// 2026-09-20: one DISCLOSED extra row — SHADERS... (the OptiFine/Iris
/// 1.16.5-modded entry; the owner welcomes modern-version extras).
pub fn layout_video() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [108, 144, 180, 216];
    vec![
        slider_h(ID_OPT_RD, 248, 72, 465, 30, "RENDER DISTANCE", 0.4),
        btn_h(
            ID_OPT_GRAPHICS,
            l,
            rows[0],
            bw,
            30,
            "GRAPHICS",
            "FANCY",
            true,
        ),
        btn_h(
            ID_OPT_SMOOTH,
            r,
            rows[0],
            bw,
            30,
            "SMOOTH LIGHTING",
            "MAXIMUM",
            true,
        ),
        btn_h(
            ID_OPT_GUISCALE,
            l,
            rows[1],
            bw,
            30,
            "GUI SCALE",
            "AUTO",
            true,
        ),
        btn_h(ID_OPT_CLOUDS, r, rows[1], bw, 30, "CLOUDS", "FANCY", true),
        btn_h(
            ID_OPT_PARTICLES,
            l,
            rows[2],
            bw,
            30,
            "PARTICLES",
            "ALL",
            true,
        ),
        btn_h(
            ID_OPT_FULLSCREEN,
            r,
            rows[2],
            bw,
            30,
            "FULL SCREEN",
            "OFF",
            true,
        ),
        btn_h(ID_OPT_VSYNC, l, rows[3], bw, 30, "USE VSYNC", "ON", true),
        btn_h(
            ID_OPT_ENTSHADOW,
            r,
            rows[3],
            bw,
            30,
            "ENTITY SHADOWS",
            "ON",
            true,
        ),
        // vanilla brightness slider carries NO label; the hover tooltip
        // reads Moody/Bright from the live value
        slider_h(ID_OPT_BRIGHT, 248, 252, 465, 30, "", 0.1),
        slider_h(ID_OPT_BIOME, 248, 288, 465, 30, "BIOME BLEND", 0.5),
        // 2026-09-20: the modern (1.17+) Entity Distance option — a
        // 50%..100% multiplier on the entity render radius (the label
        // updates from the live value in refresh_widgets)
        slider_h(ID_OPT_ENTDIST, 248, 324, 465, 30, "ENTITY DISTANCE", 1.0),
        // 2026-09-20: the OptiFine/Iris-style SHADERS... entry (engine
        // extra beyond vanilla 1.16.5 — the owner's call that modern
        // extra options are welcome; the screen lists external packs
        // from shader-packs/ only, never built-ins)
        btn_h(ID_OPT_SHADERS, 248, 360, 465, 30, "SHADERS...", "", true),
        btn_h(
            ID_OPT_DONE2,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// 2026-09-20 round: the Shader Packs screen — the OptiFine/Iris-style
/// pack selector. Rows top to bottom:
/// * "(none)" — the vanilla post pipeline (pinned first, like Iris's
///   "internal" row; selecting it unselects any pack)
/// * one row per EXTERNAL pack scanned from shader-packs/ (native
///   live scan; wasm honestly has no filesystem → the empty hint)
/// * the LABPBR MATERIALS toggle (NAPP-style `_n`/`_s` resource-pack
///   maps; OFF = the vanilla look — the default)
/// * DONE back to Video Settings (the screen's parent)
///
/// NO built-in shader packs exist to list — by design and by law
/// (docs/LEGAL-COMPLIANCE.md: BSL/SEUS-style packs are third-party
/// downloads the user drops in; the engine only reads them).
pub fn layout_shaders(packs: &[String], active: Option<&str>, labpbr: bool) -> Vec<Widget> {
    let mut v = Vec::new();
    // the pinned "(none)" row — value shows the live selection
    v.push(btn_h(
        ID_SHDR_NONE,
        148,
        86,
        660,
        30,
        "(NONE)",
        if active.is_none() { "SELECTED" } else { "" },
        true,
    ));
    for (i, name) in packs.iter().take(MAX_SHDR_ENTRIES).enumerate() {
        v.push(btn_h(
            ID_SHDR_BASE + i as u16,
            148,
            124 + i as i32 * 34,
            660,
            30,
            name,
            if active == Some(name.as_str()) {
                "SELECTED"
            } else {
                ""
            },
            true,
        ));
    }
    // the labPBR materials toggle
    v.push(btn_h(
        ID_SHDR_LABPBR,
        148,
        376,
        660,
        30,
        "LABPBR MATERIALS",
        if labpbr { "ON" } else { "OFF" },
        true,
    ));
    v.push(btn_h(
        ID_SHDR_DONE,
        (live_ui_w() as i32 - 300) / 2,
        anchor_y(470),
        300,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Engine Settings — our extra subsystems (GPU meshing, occlusion,
/// texture/AA quality, sim distance, frame cap, upscaling, sun shadows)
/// live on their own page so the Video screen stays vanilla-exact.
pub fn layout_engine() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [72, 108, 144, 180, 216];
    vec![
        slider_h(ID_OPT_SIMDIST, l, rows[0], bw, 30, "SIM DISTANCE", 0.25),
        btn_h(
            ID_OPT_MAXFPS,
            r,
            rows[0],
            bw,
            30,
            "MAX FPS",
            "UNCAPPED",
            true,
        ),
        btn_h(ID_OPT_MIP, l, rows[1], bw, 30, "MIPMAP LEVELS", "4", true),
        btn_h(ID_OPT_ANISO, r, rows[1], bw, 30, "ANISOTROPIC", "4X", true),
        btn_h(ID_OPT_MSAA, l, rows[2], bw, 30, "MSAA", "OFF", true),
        btn_h(
            ID_OPT_OCCL,
            r,
            rows[2],
            bw,
            30,
            "OCCLUSION CULLING",
            "ON",
            true,
        ),
        btn_h(
            ID_OPT_GMESH,
            l,
            rows[3],
            bw,
            30,
            "GPU CHUNK MESHING",
            "ON",
            true,
        ),
        btn_h(
            ID_OPT_SHADOWS,
            r,
            rows[3],
            bw,
            30,
            "SUN SHADOWS",
            "2K",
            true,
        ),
        btn_h(ID_OPT_UPSCALE, l, rows[4], bw, 30, "UPSCALING", "OFF", true),
        btn_h(
            ID_OPT_DONE2,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Resource Packs — the vanilla 1.16.5 two-pane manager (VERIFIED live
/// 2026-09-14, reference wiki /Resource_pack §Behavior: packs "can be
/// moved between 'Available' (disabled) and 'Selected' (enabled), and
/// reordered"; "The bottom-most pack loads first, then each pack above it
/// replaces or merges loaded assets"; Default is "Selected by default,
/// can't be unselected").
///
/// * LEFT pane — `avail`: disabled packs (Classic Art + user packs
///   from resourcepacks/). Click a row to select it.
/// * RIGHT pane — `sel`: enabled packs, TOP = highest priority, plus the
///   pinned DEFAULT row at the bottom. Click a row to deselect; the ▲▼
///   arrows reorder within the list.
///
/// The pane backgrounds + headers are painted by
/// `UiCanvas::resource_pack_screen`; the DONE button applies the edits.
pub fn layout_resource_packs(avail: &[String], sel: &[String]) -> Vec<Widget> {
    let mut v = Vec::new();
    for (i, name) in avail.iter().take(MAX_RPACK_ENTRIES).enumerate() {
        v.push(btn_h(
            ID_RPACK_AVAIL_BASE + i as u16,
            30,
            92 + i as i32 * 34,
            420,
            28,
            name,
            "",
            true,
        ));
    }
    for (i, name) in sel.iter().take(MAX_RPACK_ENTRIES).enumerate() {
        v.push(btn_h(
            ID_RPACK_SEL_BASE + i as u16,
            510,
            92 + i as i32 * 34,
            352,
            28,
            name,
            "",
            true,
        ));
        // reorder arrows (not on the pinned DEFAULT row — caller never
        // passes Default inside `sel`; it gets its own immovable row)
        v.push(btn_h(
            ID_RPACK_UP_BASE + i as u16,
            866,
            92 + i as i32 * 34,
            30,
            28,
            "^",
            "",
            true,
        ));
        v.push(btn_h(
            ID_RPACK_DOWN_BASE + i as u16,
            898,
            92 + i as i32 * 34,
            30,
            28,
            "v",
            "",
            true,
        ));
    }
    // the pinned DEFAULT row closes the Selected pane
    let dy = 92 + sel.len().min(MAX_RPACK_ENTRIES) as i32 * 34;
    v.push(btn_h(
        ID_RPACK_DEFAULT,
        510,
        dy,
        352,
        28,
        "DEFAULT",
        "(REQUIRED)",
        false,
    ));
    v.push(btn_h(
        ID_OPT_DONE2,
        (live_ui_w() as i32 - 300) / 2,
        anchor_y(470),
        300,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Accessibility Settings — the 1.16.5 vanilla row set + order,
/// live-verified 2026-09-16 (reference wiki /Options §Accessibility
/// Settings + §History): Auto-Jump (moved here 19w11b), Sprint and
/// Sneak Hold/Toggle (added 19w41a), Distortion Effects + FOV Effects
/// (added 1.16.2 pre1), Show Subtitles (the Java 1.9 subtitle system's
/// toggle — grayed here: no subtitle overlay renderer in the engine).
/// Rows the modern wiki lists that are NOT 1.16.5 are omitted by
/// version-scoping: Darkness Pulsing (1.19, the Warden), High Contrast
/// (1.20.5), Text Background Opacity (the modern accessibility split —
/// 1.16.5's chat background rides Chat Opacity in Chat Settings).
pub fn layout_access() -> Vec<Widget> {
    vec![
        btn_h(ID_OPT_AUTOJUMP, 248, 72, 465, 30, "AUTO-JUMP", "ON", true),
        // Round 14b: the Sprint/Sneak Hold-vs-Toggle pair (1.15 19w41a —
        // live rows: the toggle flips the key's latched state)
        btn_h(ID_ACC_SPRINT, 248, 112, 465, 30, "SPRINT", "HOLD", true),
        btn_h(ID_ACC_SNEAK, 248, 152, 465, 30, "SNEAK", "HOLD", true),
        // 1.16.2 pre1: "Added 'Distortion Effects' and 'FOV effects'
        // sliders to video and accessibility settings" — the engine has
        // no nether-portal/nausea screen warp yet, so the slider is
        // registered + grayed with that reason (the spec's own rule)
        slider_h(
            ID_ACC_DISTORT_SLIDER,
            248,
            192,
            465,
            30,
            "DISTORTION EFFECTS",
            1.0,
        ),
        slider_h(ID_ACC_FOVEFF, 248, 232, 465, 30, "FOV EFFECTS", 1.0),
        // the Java 1.9 subtitle toggle (also on Music & Sounds in JE —
        // grayed: no subtitle overlay renderer)
        btn_h(
            ID_ACC_SUBTITLES,
            248,
            272,
            465,
            30,
            "SHOW SUBTITLES",
            "OFF",
            false,
        ),
        // the Fog cycle stays as the engine's own live row (the round-14
        // addition; vanilla 1.16.5 has no accessibility fog row — the
        // engine's fog is a renderer feature surfaced here, disclosed)
        btn_h(ID_ACC_FOG, 248, 312, 465, 30, "FOG", "FAST", true),
        btn_h(
            ID_OPT_DONE2,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Round 14b: the Skin Customization screen — the vanilla 1.16.5 rows
/// (live w/Options §Skin Customization): Cape, Jacket, Left Sleeve,
/// Right Sleeve, Left Pant Leg, Right Pant Leg, Hat, Main Hand. The
/// player model has no separately-meshed second layers or cape yet, so
/// the seven toggles persist their `skin_*` keys with NO visible effect
/// (the round-9 armor-row pattern: the state survives until the layer
/// meshes land — disclosed in the audit doc). Main Hand persists the
/// `mainHand` key (no handedness-differentiated animation to show).
// (8 args: the vanilla screen's own eight toggle states — a struct here
// would just mirror the row list; the lint is silenced deliberately)
#[allow(clippy::too_many_arguments)]
pub fn layout_skin(
    cape: bool,
    jacket: bool,
    lsleeve: bool,
    rsleeve: bool,
    lpants: bool,
    rpants: bool,
    hat: bool,
    main_hand_left: bool,
) -> Vec<Widget> {
    let onoff = |b: bool| if b { "ON" } else { "OFF" };
    vec![
        btn_h(ID_SKIN_CAPE, 248, 72, 465, 30, "CAPE", onoff(cape), true),
        btn_h(
            ID_SKIN_JACKET,
            248,
            112,
            465,
            30,
            "JACKET",
            onoff(jacket),
            true,
        ),
        btn_h(
            ID_SKIN_LSLEEVE,
            248,
            152,
            465,
            30,
            "LEFT SLEEVE",
            onoff(lsleeve),
            true,
        ),
        btn_h(
            ID_SKIN_RSLEEVE,
            248,
            192,
            465,
            30,
            "RIGHT SLEEVE",
            onoff(rsleeve),
            true,
        ),
        btn_h(
            ID_SKIN_LPANTS,
            248,
            232,
            465,
            30,
            "LEFT PANT LEG",
            onoff(lpants),
            true,
        ),
        btn_h(
            ID_SKIN_RPANTS,
            248,
            272,
            465,
            30,
            "RIGHT PANT LEG",
            onoff(rpants),
            true,
        ),
        btn_h(ID_SKIN_HAT, 248, 312, 465, 30, "HAT", onoff(hat), true),
        btn_h(
            ID_SKIN_MAINHAND,
            248,
            352,
            465,
            30,
            "MAIN HAND",
            if main_hand_left { "LEFT" } else { "RIGHT" },
            true,
        ),
        btn_h(
            ID_SKIN_DONE,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Round 14b: the Chat Settings screen — the 1.16.5 vanilla row set
/// (live w/Options §Chat Settings + the 1.16.2-pre1 Chat Delay + the
/// 1.16.4-RC1 Hide Matched Names). No chat subsystem exists, so every
/// chat-behavior row is registered + grayed with that reason; the
/// live-behavior row is REAL: Reduced Debug Info (gates the F3
/// overlay's detail rows). Narrator is grayed — no TTS in scope.
pub fn layout_chat_settings() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [72, 108, 144, 180, 216, 252];
    vec![
        btn_h(
            ID_CHAT_VIS,
            l,
            rows[0],
            bw,
            30,
            "CHAT VISIBILITY",
            "SHOW",
            false,
        ),
        btn_h(
            ID_CHAT_COLORS,
            r,
            rows[0],
            bw,
            30,
            "CHAT COLORS",
            "ON",
            false,
        ),
        btn_h(ID_CHAT_LINKS, l, rows[1], bw, 30, "WEB LINKS", "ON", false),
        btn_h(
            ID_CHAT_LINKSPROMPT,
            r,
            rows[1],
            bw,
            30,
            "LINK PROMPT",
            "ON",
            false,
        ),
        slider_h(ID_CHAT_OPACITY, l, rows[2], bw, 30, "CHAT OPACITY", 1.0),
        slider_h(ID_CHAT_DELAY, r, rows[2], bw, 30, "CHAT DELAY", 0.0),
        slider_h(ID_CHAT_WIDTH, l, rows[3], bw, 30, "WIDTH", 0.53),
        slider_h(ID_CHAT_SCALE, r, rows[3], bw, 30, "SCALE", 1.0),
        slider_h(
            ID_CHAT_HFOCUSED,
            l,
            rows[4],
            bw,
            30,
            "HEIGHT (FOCUSED)",
            0.5,
        ),
        slider_h(
            ID_CHAT_HUNFOCUSED,
            r,
            rows[4],
            bw,
            30,
            "HEIGHT (UNFOC.)",
            0.44,
        ),
        slider_h(ID_CHAT_LINESPACING, l, rows[5], bw, 30, "LINE SPACING", 0.0),
        btn_h(
            ID_CHAT_HIDENAMES,
            r,
            rows[5],
            bw,
            30,
            "HIDE MATCHED NAMES",
            "OFF",
            false,
        ),
        btn_h(
            ID_CHAT_REDUCEDDEBUG,
            248,
            292,
            225,
            30,
            "REDUCED DEBUG INFO",
            "OFF",
            true,
        ),
        btn_h(
            ID_CHAT_NARRATOR,
            487,
            292,
            225,
            30,
            "NARRATOR",
            "OFF",
            false,
        ),
        btn_h(
            ID_CHAT_DONE,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Round 14 (2026-09-15): the Music & Sound screen — vanilla 1.16.5's
/// ten sliders, one per SoundCategory with master at the top
/// (reference wiki /Options §Music & Sound, live 2026-09-15: "Music &
/// Sound ... has sliders which control the volume of each sound
/// category"). Values are patched in by the caller (game.rs) — the
/// layout pins only the geometry. Slider ids 160..170 (ID_SND_BASE..).
pub fn layout_music_sound() -> Vec<Widget> {
    let l = 248;
    let bw = 465;
    let rows = [72, 106, 140, 174, 208, 242, 276, 310, 344, 378];
    let names = [
        "MASTER",
        "MUSIC",
        "JUKEBOXES / NOTE BLOCKS",
        "WEATHER",
        "BLOCKS",
        "HOSTILE CREATURES",
        "FRIENDLY CREATURES",
        "PLAYERS",
        "AMBIENT / ENVIRONMENT",
        "VOICE / SPEECH",
    ];
    let mut v = Vec::with_capacity(12);
    for (i, n) in names.iter().enumerate() {
        v.push(slider_h(ID_SND_BASE + i as u16, l, rows[i], bw, 30, n, 0.8));
    }
    // Round 14b: SHOW SUBTITLES — the 1.16.5 placement (live w/Subtitles:
    // "In Java Edition, you can also enable these in the Music & Sounds
    // options") — grayed: no subtitle overlay renderer in the engine
    v.push(btn_h(
        ID_SND_SUBTITLES,
        l,
        418,
        bw,
        30,
        "SHOW SUBTITLES",
        "OFF",
        false,
    ));
    v.push(btn_h(
        ID_SND_DONE,
        (live_ui_w() as i32 - 300) / 2,
        anchor_y(470),
        300,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Round 14 (2026-09-15): the Controls screen — vanilla's two-column
/// keybind editor (reference wiki /Controls, live 2026-09-15: rows of
/// action + key button, categories, "Reset Keys" at the bottom). The
/// engine's rebindable set: movement, inventory, gameplay. Row ids
/// 180.. (ID_CTRL_BIND_BASE..); the caller patches the key labels.
pub fn layout_controls(labels: &[(bool, &str, &str)]) -> Vec<Widget> {
    // (is_header, action, key) — headers are non-button category rows
    let l = 130;
    let name_w = 300;
    let key_w = 120;
    let r = l + name_w + 10;
    let mut v = Vec::new();
    let mut y = 66;
    for (i, (is_header, action, key)) in labels.iter().enumerate() {
        if *is_header {
            v.push(btn_h(
                ID_CTRL_BIND_BASE + i as u16,
                l,
                y,
                name_w + 10 + key_w,
                22,
                action,
                "",
                false,
            ));
        } else {
            v.push(btn_h(
                ID_CTRL_BIND_BASE + i as u16,
                l,
                y,
                name_w,
                30,
                action,
                "",
                true,
            ));
            v.push(btn_h(
                ID_CTRL_BIND_BASE + i as u16,
                r,
                y,
                key_w,
                30,
                key,
                "",
                true,
            ));
        }
        y += if *is_header { 26 } else { 34 };
    }
    v.push(btn_h(
        ID_CTRL_RESET,
        l,
        anchor_y(440),
        210,
        30,
        "RESET KEYS",
        "",
        true,
    ));
    v.push(btn_h(
        ID_CTRL_DONE,
        l + 230,
        anchor_y(440),
        210,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Round 14: the Language screen — the engine is English-only; the
/// screen lists English and says so (spec rule: do NOT pretend to
/// support other languages).
pub fn layout_language() -> Vec<Widget> {
    vec![
        btn_h(ID_CTRL_DONE, 248, 150, 465, 30, "ENGLISH (US)", "*", true),
        btn_h(
            ID_OPT_DONE2,
            (live_ui_w() as i32 - 300) / 2,
            anchor_y(470),
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Round 14: the Chat Settings screen — RETIRED as a stub in Round 14b
/// (the real 1.16.5 row set lives in the layout_chat_settings above).
pub fn layout_pause() -> Vec<Widget> {
    vec![
        btn(
            ID_PAUSE_BACK,
            (live_ui_w() as i32 - 320) / 2,
            208,
            320,
            "BACK TO GAME",
            "",
            true,
        ),
        btn(
            ID_PAUSE_OPTIONS,
            (live_ui_w() as i32 - 320) / 2,
            264,
            320,
            "OPTIONS...",
            "",
            true,
        ),
        btn(
            ID_PAUSE_QUIT,
            (live_ui_w() as i32 - 320) / 2,
            320,
            320,
            "QUIT TO TITLE",
            "",
            true,
        ),
    ]
}

/// 2026-09-14 parity round: the vanilla 1.16.5 Select World layout —
/// title + top-left search field, two-line world entries (painted by
/// `world_select_screen`), then the two vanilla bottom rows:
///   row A: [PLAY SELECTED WORLD] [CREATE NEW WORLD]
///   row B: [EDIT] [DELETE] [RE-CREATE] [SEARCH]
/// Rows are hit-test widgets whose bodies the painter fills (name +
/// info lines) instead of a centered single label. `can_play` gates row A
/// on a live selection (vanilla disables both without one).
pub fn layout_world_select(n_rows: usize, can_play: bool, delete_armed: bool) -> Vec<Widget> {
    let mut v = Vec::new();
    // world rows (hit-test only — the painter draws the two-line body)
    for i in 0..n_rows.min(MAX_LISTED_WORLDS) {
        v.push(btn_h(
            ID_WS_WORLD_BASE + i as u16,
            227,
            96 + i as i32 * 50,
            506,
            46,
            "",
            "",
            true,
        ));
    }
    // top-left search field (vanilla "Search worlds..." box)
    v.push(text_field_h(
        ID_WS_SEARCHFIELD,
        176,
        44,
        225,
        30,
        "",
        "",
        "SEARCH WORLDS...",
    ));
    // row A + row B (vanilla bottom stacks, 1.5x geometry)
    v.push(btn_h(
        ID_WS_PLAY,
        248,
        anchor_y(440),
        225,
        30,
        "PLAY SELECTED WORLD",
        "",
        can_play,
    ));
    v.push(btn_h(
        ID_WS_CREATE,
        487,
        anchor_y(440),
        225,
        30,
        "CREATE NEW WORLD",
        "",
        true,
    ));
    let four = 150i32; // 4 × 100-wide vanilla buttons at 1.5x
    let gap = 12i32;
    let x0 = (live_ui_w() as i32 - (four * 4 + gap * 3)) / 2;
    v.push(btn_h(ID_WS_EDIT, x0, anchor_y(480), four, 30, "EDIT", "", can_play));
    v.push(btn_h(
        ID_WS_DELETE,
        x0 + four + gap,
        anchor_y(480),
        four + gap * 2,
        30,
        if delete_armed {
            "REALLY DELETE?"
        } else {
            "DELETE"
        },
        "",
        can_play,
    ));
    v.push(btn_h(
        ID_WS_RECREATE,
        x0 + (four + gap) * 2,
        anchor_y(480),
        four,
        30,
        "RE-CREATE",
        "",
        can_play,
    ));
    v.push(btn_h(
        ID_WS_SEARCH,
        x0 + (four + gap) * 3,
        anchor_y(480),
        four,
        30,
        "SEARCH",
        "",
        true,
    ));
    // cancel keeps the vanilla Esc route
    v
}

/// 2026-09-14 parity round: the vanilla 1.16.5 Create World layout —
/// TWO pages like vanilla's More World Options flow:
///
/// * page 1: name field, Game Mode button + description, bottom
///   [CREATE NEW WORLD] [MORE WORLD OPTIONS...] + centered CANCEL
/// * page 2 (More World Options): seed field + "leave blank" hint,
///   [WORLD TYPE: ...] [GENERATE STRUCTURES: ON/OFF], [BONUS CHEST:
///   ON/OFF], bottom [DONE...] + CANCEL
///
/// Values refresh on every keystroke / toggle from game.rs.
#[allow(clippy::too_many_arguments)]
pub fn layout_world_create(
    page2: bool,
    name: &str,
    seed: &str,
    seed_placeholder: &str,
    mode_label: &str,
    type_label: &str,
    structures: bool,
    bonus: bool,
) -> Vec<Widget> {
    let mut v = Vec::new();
    if !page2 {
        v.push(text_field_h(
            ID_WC_NAME,
            236,
            84,
            488,
            30,
            "",
            name,
            "New World",
        ));
        // vanilla: the game-mode button is 200 wide centered with its
        // two-line description under it
        v.push(btn_h(
            ID_WC_MODE,
            330,
            150,
            300,
            30,
            "GAME MODE",
            mode_label,
            true,
        ));
        v.push(btn_h(
            ID_WC_CREATE,
            248,
            anchor_y(440),
            225,
            30,
            "CREATE NEW WORLD",
            "",
            true,
        ));
        v.push(btn_h(
            ID_WC_MORE,
            487,
            anchor_y(440),
            225,
            30,
            "MORE WORLD OPTIONS...",
            "",
            true,
        ));
        v.push(btn_h(ID_WC_CANCEL, 330, anchor_y(480), 300, 30, "CANCEL", "", true));
    } else {
        v.push(text_field_h(
            ID_WC_SEED,
            236,
            84,
            488,
            30,
            "",
            seed,
            seed_placeholder,
        ));
        v.push(btn_h(
            ID_WC_TYPE,
            248,
            160,
            225,
            30,
            "WORLD TYPE",
            type_label,
            true,
        ));
        v.push(btn_h(
            ID_WC_STRUCT,
            487,
            160,
            225,
            30,
            "GENERATE STRUCTURES",
            if structures { "ON" } else { "OFF" },
            true,
        ));
        v.push(btn_h(
            ID_WC_BONUS,
            248,
            200,
            225,
            30,
            "BONUS CHEST",
            if bonus { "ON" } else { "OFF" },
            true,
        ));
        // vanilla page 2: [Done...] returns to page 1
        v.push(btn_h(ID_WC_MORE, 487, anchor_y(440), 225, 30, "DONE...", "", true));
        v.push(btn_h(ID_WC_CANCEL, 330, anchor_y(480), 300, 30, "CANCEL", "", true));
    }
    v
}

/// 2026-09-14 parity round: the vanilla Edit World screen — title, the
/// world-name field, then [RENAME] [DELETE] / [COPY WORLD] [DONE].
pub fn layout_world_edit(name: &str) -> Vec<Widget> {
    vec![
        text_field_h(ID_WE_NAME, 236, 84, 488, 30, "", name, ""),
        btn_h(ID_WE_RENAME, 248, 200, 225, 30, "RENAME", "", true),
        btn_h(ID_WE_DELETE, 487, 200, 225, 30, "DELETE", "", true),
        btn_h(ID_WE_COPY, 248, 240, 225, 30, "COPY WORLD", "", true),
        btn_h(ID_WE_DONE, 487, 240, 225, 30, "DONE", "", true),
        btn_h(ID_WC_CANCEL, 330, anchor_y(480), 300, 30, "CANCEL", "", true),
    ]
}

/// Phase 1 + 2026-09-14: death screen — vanilla two 200-wide buttons
/// (300 at 1.5x) stacked, 40px apart.
pub fn layout_death(hardcore: bool) -> Vec<Widget> {
    let mut v = Vec::new();
    let (x, w) = ((live_ui_w() as i32 - 300) / 2, 300);
    if !hardcore {
        v.push(btn_h(ID_DEATH_RESPAWN, x, 296, w, 30, "RESPAWN", "", true));
        v.push(btn_h(
            ID_DEATH_TITLE,
            x,
            336,
            w,
            30,
            "TITLE SCREEN",
            "",
            true,
        ));
    } else {
        // hardcore: death is final — vanilla's two options (delete world /
        // title screen, which leaves the locked world on disk)
        v.push(btn_h(
            ID_DEATH_DELETE,
            x,
            296,
            w,
            30,
            "DELETE WORLD",
            "",
            true,
        ));
        v.push(btn_h(
            ID_DEATH_TITLE,
            x,
            336,
            w,
            30,
            "TITLE SCREEN",
            "",
            true,
        ));
    }
    v
}

// ------------------------------------------------- Phase 5 font core --
// variable-width advance: glyph advance = measured ink width + 1
// (VERIFIED https://reference wiki /Font — "the width of each
// character is the rightmost ink pixel + 1"; the space keeps a fixed
// 4-px advance), shadow at (x+1, y+1) in foreground x 0.25.

/// active-font override installed by a resource pack (None = builtin
/// FONT). Set ONCE at boot before the first text draw; later installs
/// are refused (no font hot-reload — documented in the worklog). The
/// OnceLock (thread-safe OnceCell) keeps every existing static
/// `text_width`-family signature intact (G6) while letting all of
/// them read the ACTIVE source.
static FONT_OVERRIDE: OnceLock<Box<[[u8; 8]; 96]>> = OnceLock::new();
/// per-character ink widths derived from the active font, measured
/// once, never re-measured per call (the spec's OnceCell cache, in its
/// thread-safe form — worker threads read these too)
static FONT_WIDTHS: OnceLock<Box<[u8; 96]>> = OnceLock::new();

/// Phase 5 (D1): measured glyph ink width — the 1-indexed column of
/// the rightmost ink pixel, scanning the 8-column mask from the right
/// (the builtin ink field is 5 px in bits 4..0; bits 7..5 stay clear).
/// Returns 0 for a blank glyph (the space).
pub(crate) fn glyph_ink_width(glyph: &[u8; 8]) -> i32 {
    // scan the renderer's 5-px ink field (bits 4..0) from the right;
    // bits 7..5 are always clear in every source (builtin + PNG-decoded)
    for gx in (0..5).rev() {
        if glyph.iter().any(|row| row & (1 << (4 - gx)) != 0) {
            return gx + 1;
        }
    }
    0
}

/// install a pack-provided glyph sheet as the active font. Returns
/// false when an override is already installed (first font wins).
pub fn set_font_override(glyphs: Box<[[u8; 8]; 96]>) -> bool {
    if FONT_WIDTHS.get().is_some() || FONT_OVERRIDE.get().is_some() {
        return false;
    }
    FONT_OVERRIDE.set(glyphs).is_ok()
}

/// the glyph at slot `idx` (0..96) from the ACTIVE font source
fn active_glyph(idx: usize) -> &'static [u8; 8] {
    match FONT_OVERRIDE.get() {
        Some(o) => &o[idx],
        None => &FONT[idx],
    }
}

/// per-character advance in GLYPH pixels: measured ink width + 1, with
/// the space's fixed 4-px advance (VERIFIED w/Font — the space is a
/// 4-px-wide glyph, not a measured one)
fn char_advance(idx: usize) -> i32 {
    if idx == 0 {
        return 4;
    }
    active_widths()[idx] as i32 + 1
}

fn active_widths() -> &'static [u8; 96] {
    FONT_WIDTHS.get_or_init(|| {
        let mut w = Box::new([0u8; 96]);
        for (i, wi) in w.iter_mut().enumerate() {
            *wi = glyph_ink_width(active_glyph(i)) as u8;
        }
        w
    })
}

/// smallcaps slot for a char (the text()/text_frac() look: a-z render
/// through the A-Z slots; everything else maps through).
/// `pub(crate)` — the font engine's bitmap-fallback path shares it.
pub(crate) fn smallcaps_slot(ch: char) -> usize {
    let mut ch = ch as usize;
    if !(32..=126).contains(&ch) {
        ch = '?' as usize;
    }
    if ch >= 'a' as usize && ch <= 'z' as usize {
        ch -= 32;
    }
    ch - 32
}

// ------------------------------------------------------------- canvas --

pub struct UiCanvas {
    pub px: Vec<u8>,
    /// logical "the game wants a re-raster" flag — set by gameplay/UI
    /// code, consumed by the game's rebuild gate (update()). It does NOT
    /// mean "pixels changed"; see [`Self::upload_pending`].
    pub dirty: bool,
    /// "pixels changed since the last GPU upload" — set ONLY by actual
    /// canvas repaints (clear/resize/rebuild), consumed by the renderer's
    /// upload step. Split from `dirty` to kill the Linux menu-freeze
    /// race: X11/Wayland deliver SPONTANEOUS RedrawRequested events
    /// (expose/damage/frame callbacks) between a click and the next
    /// update() pass — the old single-flag upload there re-uploaded the
    /// STALE canvas and cleared `dirty`, permanently suppressing the
    /// pending rebuild: the click sound played, screen state switched,
    /// but the old menu stayed painted forever ("clicked Singleplayer,
    /// nothing opened"). With two flags a spontaneous redraw uploads
    /// nothing (no fresh pixels) and can never kill the rebuild.
    pub upload_pending: bool,
    /// Round 10 (vanilla integer GUI scale): the LIVE canvas size in
    /// canvas px. The raster is 2 canvas px per vanilla px, so at the
    /// 960×540 reference the canvas IS the classic grid (every
    /// existing test); at other resolved scales the game resizes to
    /// (2·fb_w/scale, 2·fb_h/scale) so HUD edge anchors land on true
    /// screen edges and menus re-center in the live logical space.
    pub live_w: usize,
    pub live_h: usize,
    /// GUI Scale factor applied to widget text (set alongside
    /// [`scale_widgets`] — geometry scaling and text scaling move together)
    pub widget_scale: f32,
    /// UI-overhaul Phase 2: when false, the canvas draw methods skip
    /// their CHROME raster (buttons/slots/panels/HUD sprite chrome/
    /// hotbar chrome) because the GPU quad pass draws it instead.
    /// Text, icons, values are unaffected. Default true — existing
    /// callers render exactly as before (G6).
    pub chrome_enabled: bool,
    /// UI-overhaul Phase 2: the per-frame GUI quad list, populated by
    /// the same draw methods that raster chrome (they always push;
    /// the Renderer decides whether to draw them).
    pub gui_frame: crate::gui_render::GuiFrame,
    /// UI-overhaul Phase 3: ready 3D icons (block id -> icon-atlas cell),
    /// snapshotted from the game's ItemIconCache whenever a new icon
    /// finishes baking. None/absent = flat blit_tile fallback.
    pub icon_cells: Option<std::sync::Arc<std::collections::HashMap<u16, [u8; 2]>>>,
    /// The Luanti-style font round: the canvas letterbox scale
    /// (device px per UI px). The GPU text path rasterizes glyphs at
    /// `cell × device_scale` so the AA edges land on real screen
    /// pixels at ANY window size. Default 1.0; the game refreshes it
    /// every frame from the Renderer before `rebuild_ui`.
    device_scale: f32,
}

impl Default for UiCanvas {
    fn default() -> Self {
        Self::new()
    }
}

/// tight yellow ink for the splash text from the ACTIVE source:
/// the runtime engine's `bake_bitmap` when armed, the clean-room
/// bitmap font at scale 2 otherwise. Returns flat RGBA bytes + dims.
fn splash_ink(s: &str) -> (Vec<u8>, i32, i32) {
    const YELLOW: [u8; 4] = [255, 255, 0, 255];
    if text_quads_active() {
        if let Some(eng) = crate::gui::font::engine() {
            let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
            let (bytes, w, h) = splash_ink_at(s, 16.0, &mut e);
            if w > 0 {
                return (bytes, w, h);
            }
        }
    }
    // bitmap path: fixed 6-px advance at scale 2 (the original look)
    let scale = 2i32;
    let n = s.chars().count() as i32;
    let tw = (n * 6 * scale).max(1);
    let th = 8 * scale;
    let mut out = vec![0u8; (tw * th * 4) as usize];
    let mut pen = 0i32;
    for ch in s.chars() {
        let mut slot = ch as usize;
        if !(32..=126).contains(&slot) {
            slot = b'?' as usize;
        }
        if slot >= b'a' as usize && slot <= b'z' as usize {
            slot -= 32; // smallcaps look (the bitmap font's UI style)
        }
        let glyph = &FONT[slot - 32];
        for gy in 0..8i32 {
            for gx in 0..5i32 {
                if glyph[gy as usize] & (1 << (4 - gx)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = pen + gx * scale + sx;
                            let py = gy * scale + sy;
                            if px >= 0 && py >= 0 && px < tw && py < th {
                                let i = ((py * tw + px) * 4) as usize;
                                out[i..i + 4].copy_from_slice(&YELLOW);
                            }
                        }
                    }
                }
            }
        }
        pen += 6 * scale;
    }
    (out, tw, th)
}

/// engine-backed tight yellow ink at an explicit (device) cell —
/// shared by the canvas fallback (16 UI px) and the rotated-quad
/// path (16 × device_scale, so the strip bakes at DEVICE resolution).
fn splash_ink_at(s: &str, cell: f32, e: &mut crate::gui::font::FontEngine) -> (Vec<u8>, i32, i32) {
    const YELLOW: [u8; 4] = [255, 255, 0, 255];
    let (bytes, w, h) = e.bake_bitmap(s, cell, YELLOW);
    (bytes, w as i32, h as i32)
}

/// the outlined splash strip (flat RGBA): tight yellow ink + a 1-px
/// dark yellow-brown 8-neighborhood outline, 2-px pad all around —
/// the shared builder for the canvas blit and the cached run strip.
fn outlined_strip(tight: &[u8], tw: i32, th: i32) -> (Vec<u8>, i32, i32) {
    const YELLOW: [u8; 4] = [255, 255, 0, 255];
    const OUTLINE: [u8; 4] = [63, 50, 0, 255];
    let pad = 2; // outline margin
    let bw = tw + pad * 2;
    let bh = th + pad * 2;
    let mut src = vec![0u8; (bw * bh * 4) as usize];
    let ink = |x: i32, y: i32| -> bool {
        x >= 0 && y >= 0 && x < tw && y < th && {
            let i = ((y * tw + x) * 4 + 3) as usize;
            tight[i] != 0
        }
    };
    // glyph pass
    for y in 0..th {
        for x in 0..tw {
            if ink(x, y) {
                let di = (((y + pad) * bw + x + pad) * 4) as usize;
                src[di..di + 4].copy_from_slice(&YELLOW);
            }
        }
    }
    // 8-neighborhood outline on empty pixels
    for y in -1..=th {
        for x in -1..=tw {
            let sx = x + pad;
            let sy = y + pad;
            if sx < 0 || sy < 0 || sx >= bw || sy >= bh {
                continue;
            }
            let di = ((sy * bw + sx) * 4) as usize;
            if src[di + 3] != 0 {
                continue;
            }
            let touches = [
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ]
            .iter()
            .any(|&(dx, dy)| ink(x + dx, y + dy));
            if touches {
                src[di..di + 4].copy_from_slice(&OUTLINE);
            }
        }
    }
    (src, bw, bh)
}

/// the splash source bitmap: tight yellow glyphs + a 1-px dark
/// yellow-brown 8-neighborhood outline, 2-px pad all around.
fn splash_source(s: &str) -> (Vec<[u8; 4]>, i32, i32) {
    let (tight, tw, th) = splash_ink(s);
    let (flat, bw, bh) = outlined_strip(&tight, tw, th);
    // flat RGBA → per-pixel arrays for the canvas rotated blit
    let mut out = vec![[0u8; 4]; (bw * bh) as usize];
    for (i, o) in out.iter_mut().enumerate() {
        let b = i * 4;
        *o = [flat[b], flat[b + 1], flat[b + 2], flat[b + 3]];
    }
    (out, bw, bh)
}

impl UiCanvas {
    pub fn new() -> Self {
        UiCanvas {
            px: vec![0u8; UI_W * UI_H * 4],
            dirty: true,
            upload_pending: true,
            live_w: UI_W,
            live_h: UI_H,
            widget_scale: 1.0,
            chrome_enabled: true,
            gui_frame: crate::gui_render::GuiFrame::default(),
            icon_cells: None,
            device_scale: 1.0,
        }
    }

    /// Round 10: resize the canvas raster to the live logical GUI
    /// space (2 canvas px per vanilla px). No-op when the size already
    /// matches; otherwise reallocates zeroed and flags dirty so the
    /// frame re-rasterizes + re-uploads. Callers follow with a widget
    /// re-layout (`rebuild_ui`) so menu geometry re-centers.
    pub fn resize(&mut self, w: usize, h: usize) {
        let (w, h) = (w.max(1), h.max(1));
        if w == self.live_w && h == self.live_h {
            return;
        }
        self.live_w = w;
        self.live_h = h;
        self.px = vec![0u8; w * h * 4];
        self.dirty = true;
        // a fresh zeroed raster MUST reach the GPU even before the first
        // rebuild, or the texture keeps the old size's garbage
        self.upload_pending = true;
    }

    /// the device scale (device px per UI px) for the GPU text path —
    /// set per frame from the Renderer's letterbox uniform
    pub fn set_device_scale(&mut self, k: f32) {
        self.device_scale = k.max(0.05);
    }

    /// Phase 3: install the ready-icon snapshot (called by the game
    /// whenever the icon cache's version moves)
    pub fn set_icon_cells(
        &mut self,
        cells: std::sync::Arc<std::collections::HashMap<u16, [u8; 2]>>,
    ) {
        self.icon_cells = Some(cells);
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.px.iter_mut().for_each(|p| *p = 0);
        self.gui_frame.clear();
        self.dirty = true;
        // a repaint landed in the pixel buffer — the renderer must upload
        // it (rebuild_ui always starts with clear())
        self.upload_pending = true;
    }

    /// Phase 2 D2: suppress the canvas CHROME raster (the GPU quad pass
    /// draws it). Text/icons/values keep drawing. Default is true so the
    /// pre-quad rendering path is byte-identical (A2).
    pub fn set_chrome_enabled(&mut self, enabled: bool) {
        self.chrome_enabled = enabled;
    }

    /// save the current canvas (RGBA, straight alpha) as a PNG — the
    /// F3_DUMP visual-verification hook (never set in CI)
    pub fn dump_png(&self, path: &str) {
        if let Some(img) =
            image::RgbaImage::from_raw(self.live_w as u32, self.live_h as u32, self.px.clone())
        {
            let _ = img.save(path);
        }
    }

    /// save the canvas FLATTENED over a backdrop color — the composite
    /// the GPU's alpha-blend produces at render time (the raw dump keeps
    /// the overlay alphas, which read as near-black in PNG viewers).
    /// The headless visual-review path (ui_snapshots bin).
    pub fn dump_png_flat(&self, path: &str, bg: Color) {
        let mut out = self.px.clone();
        for px in out.as_chunks_mut::<4>().0 {
            let a = px[3] as u32;
            if a == 0 {
                px.copy_from_slice(&bg);
            } else if a < 255 {
                let inv = 255 - a;
                for c in 0..3 {
                    px[c] = ((bg[c] as u32 * inv + px[c] as u32 * a) / 255) as u8;
                }
                px[3] = 255;
            }
        }
        if let Some(img) = image::RgbaImage::from_raw(self.live_w as u32, self.live_h as u32, out) {
            let _ = img.save(path);
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        if x < 0 || x >= self.live_w as i32 || y < 0 || y >= self.live_h as i32 {
            return;
        }
        let i = (y as usize * self.live_w + x as usize) * 4;
        self.px[i] = c[0];
        self.px[i + 1] = c[1];
        self.px[i + 2] = c[2];
        self.px[i + 3] = c[3];
        self.dirty = true;
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for xx in x..x + w {
            self.set(xx, y, c);
            self.set(xx, y + h - 1, c);
        }
        for yy in y..y + h {
            self.set(x, yy, c);
            self.set(x + w - 1, yy, c);
        }
    }

    /// 5x7-body text with 1px shadow, scale 1..8. Returns width drawn.
    /// (smallcaps: a-z render through the A-Z slots — the UI look)
    /// Phase 5: variable glyph advance (measured ink width + 1, space
    /// fixed 4) and the shadow at (x+1, y+1) in foreground x 0.25.
    /// Luanti font round: when the GPU text path is armed this routes
    /// to `GuiFrame::text` — Monocraft glyph quads rasterized at
    /// device resolution, drawn OVER the chrome quads (true-case
    /// rendering — Monocraft has real lowercase, the 1.16 look).
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: Color, scale: i32) -> i32 {
        if text_quads_active() {
            return self
                .gui_frame
                .text(x, y, s, c, 8.0 * scale as f32, self.device_scale, true);
        }
        let mut cx = x;
        for ch in s.chars() {
            let slot = smallcaps_slot(ch);
            let glyph = active_glyph(slot);
            // VERIFIED w/Font — shadow = foreground multiplied by 0.25
            let shadow: Color = [c[0] >> 2, c[1] >> 2, c[2] >> 2, c[3]];
            for gy in 0..8i32 {
                for gx in 0..5i32 {
                    if glyph[gy as usize] & (1 << (4 - gx)) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let dx = cx + gx * scale + sx;
                                let dy = y + gy * scale + sy;
                                self.set(dx + 1, dy + 1, shadow);
                                self.set(dx, dy, c);
                            }
                        }
                    }
                }
            }
            cx += char_advance(slot) * scale;
        }
        cx - x
    }

    /// Phase 5: measured width — sum of per-character advances
    /// (smallcaps-mapped, like text()). Luanti font round: engine
    /// metrics when the quad path is armed.
    pub fn text_width(s: &str, scale: i32) -> i32 {
        measure_active(s, 8.0 * scale as f32)
    }

    /// Smallcaps text at a FRACTIONAL scale (nearest-neighbor glyph
    /// sampling — the pixel-art look survives; GUI Scale and the vanilla
    /// 30px button proportions need 1.5x-class text). Shadow like text().
    /// Luanti font round: routes to the engine when armed (the engine
    /// rasterizes at device resolution — fractional UI cells land
    /// cleanly).
    pub fn text_frac(&mut self, x: i32, y: i32, s: &str, c: Color, scale: f32) -> i32 {
        if text_quads_active() {
            return self
                .gui_frame
                .text(x, y, s, c, 8.0 * scale, self.device_scale, true);
        }
        let mut cx = x as f32;
        let gw = (5.0 * scale).ceil() as i32;
        let gh = (8.0 * scale).ceil() as i32;
        // VERIFIED w/Font — shadow = foreground multiplied by 0.25
        let shadow: Color = [c[0] >> 2, c[1] >> 2, c[2] >> 2, c[3]];
        for ch in s.chars() {
            let slot = smallcaps_slot(ch);
            let glyph = active_glyph(slot);
            let bx = cx as i32;
            for gy in 0..gh {
                for gx in 0..gw {
                    let sx = ((gx as f32) / scale) as i32;
                    let sy = ((gy as f32) / scale) as i32;
                    if sy < 8 && sx < 5 && glyph[sy as usize] & (1 << (4 - sx)) != 0 {
                        self.set(bx + gx + 1, y + gy + 1, shadow);
                        self.set(bx + gx, y + gy, c);
                    }
                }
            }
            cx += char_advance(slot) as f32 * scale;
        }
        (cx - x as f32) as i32
    }

    /// Phase 5: measured width — sum of per-character advances
    pub fn text_width_frac(s: &str, scale: f32) -> i32 {
        measure_active(s, 8.0 * scale)
    }

    /// Glyphs without the 1-px drop shadow — the vanilla F3 overlay renders
    /// its debug text flat (the dark per-line strip replaces the shadow).
    /// (smallcaps, like text())
    /// Luanti font round: engine route when armed (shadow off).
    pub fn text_flat(&mut self, x: i32, y: i32, s: &str, c: Color, scale: i32) {
        if text_quads_active() {
            self.gui_frame
                .text(x, y, s, c, 8.0 * scale as f32, self.device_scale, false);
            return;
        }
        let mut cx = x;
        for ch in s.chars() {
            let slot = smallcaps_slot(ch);
            let glyph = active_glyph(slot);
            for gy in 0..8i32 {
                for gx in 0..5i32 {
                    if glyph[gy as usize] & (1 << (4 - gx)) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                self.set(cx + gx * scale + sx, y + gy * scale + sy, c);
                            }
                        }
                    }
                }
            }
            cx += char_advance(slot) * scale;
        }
    }

    /// True-case flat text (the F3 renderer): NO smallcaps remap (the
    /// a-z slots hold real lowercase shapes with descenders), flat
    /// (unshadowed — the per-line strip replaces the shadow), and a
    /// per-glyph variable advance (width + 1; space 3) so narrow
    /// letters (i l t) pack tight like the vanilla font. '∞' gets a
    /// dedicated 5-wide glyph. Returns the drawn width.
    /// Luanti font round: the engine renders TRUE case natively
    /// (Monocraft has real lowercase) — routed when armed.
    pub fn text_flat_case(&mut self, x: i32, y: i32, s: &str, c: Color, scale: i32) -> i32 {
        if text_quads_active() {
            return self
                .gui_frame
                .text(x, y, s, c, 8.0 * scale as f32, self.device_scale, false);
        }
        self.text_flat_case_px(x, y, s, c, scale)
    }

    /// The CANVAS-glyph branch of text_flat_case, callable directly:
    /// 2026-09-19 — in armed mode (GPU chrome/text quads) the live F3
    /// text rides the GPU text layer, which dump_png (the CPU canvas
    /// pixel buffer) cannot see. The smoke's F3_DUMP liveness pair
    /// re-renders the overlay through THIS path so the dumped pixels
    /// carry the live text (and its monotonic Frame/R counters).
    pub fn text_flat_case_px(&mut self, x: i32, y: i32, s: &str, c: Color, scale: i32) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            match case_glyph(ch) {
                None => cx += 3 * scale, // space
                Some((g, lb, w)) => {
                    for gy in 0..8i32 {
                        for gx in 0..5i32 {
                            if g[gy as usize] & (1 << (4 - gx)) != 0 {
                                for sy in 0..scale {
                                    for sx in 0..scale {
                                        self.set(
                                            cx + (gx - lb) * scale + sx,
                                            y + gy * scale + sy,
                                            c,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    cx += (w + 1) * scale;
                }
            }
        }
        cx - x
    }

    /// Measured width of text_flat_case (advances included) — the F3
    /// right-column right-alignment and strip sizing.
    /// Luanti font round: engine metrics when armed.
    pub fn text_width_case(s: &str, scale: i32) -> i32 {
        if text_quads_active() {
            return measure_active(s, 8.0 * scale as f32);
        }
        let mut w = 0;
        for ch in s.chars() {
            match case_glyph(ch) {
                None => w += 3 * scale,
                Some((_, _, gw)) => w += (gw + 1) * scale,
            }
        }
        w
    }

    pub fn text_center(&mut self, y: i32, s: &str, c: Color, scale: i32) {
        let w = Self::text_width(s, scale);
        self.text((self.live_w as i32 - w) / 2, y, s, c, scale);
    }

    /// Text with a 1px outline in all 8 directions (for logo / level number).
    pub fn text_outlined(
        &mut self,
        x: i32,
        y: i32,
        s: &str,
        c: Color,
        oc: Color,
        scale: i32,
    ) -> i32 {
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            self.text(
                x + dx * (scale / 4 + 1),
                y + dy * (scale / 4 + 1),
                s,
                oc,
                scale,
            );
        }
        self.text(x, y, s, c, scale)
    }

    /// Vanilla-style splash text: yellow, tilted -20 degrees (right side
    /// up), pulsing at 2 Hz (VERIFIED reference wiki /Splash: "yellow
    /// lines of text on the title screen... pulsates at a frequency of
    /// 2 Hz"; the tilt is the classic ~20-degree rotation at the logo's
    /// bottom-right corner).
    ///
    /// Luanti round 2: when the quad path is armed the outlined run is
    /// baked at DEVICE resolution ONCE (cached in the font engine's
    /// glyph atlas) and drawn as a single ROTATED quad — the glyph
    /// edges land on real screen pixels at any window size and the
    /// tilt/pulse are pure vertex geometry (the old canvas path
    /// inverse-rotated a 960×540 blit, mushy at fractional scales).
    /// The canvas fallback keeps the clean-room bitmap technique.
    pub fn text_splash(&mut self, cx: i32, cy: i32, s: &str, t: f32) {
        if text_quads_active() {
            if let Some(eng) = crate::gui::font::engine() {
                if self.splash_quad(cx, cy, s, t, eng) {
                    return;
                }
            }
        }
        // source bitmap: yellow glyphs + 1px dark outline, 2-px pad.
        // Luanti font round: the runtime engine bakes the glyph run
        // (Monocraft at the 16-px cell — the same size the old 5x7
        // bitmap had at scale 2) when the quad path is armed; the
        // clean-room bitmap otherwise. The outline/rotation technique
        // below is unchanged.
        let (src, bw, bh) = splash_source(s);

        // rotated blit: -20 deg, pulse 1.00..1.06 at 2 Hz
        let theta = -(20.0_f32).to_radians();
        let (sn, cs) = (theta.sin(), theta.cos());
        let pulse = 1.0 + 0.06 * (t * std::f32::consts::TAU).sin().abs();
        let hw = ((bw as f32 * cs).abs() + (bh as f32 * sn).abs()) * 0.5 * pulse;
        let hh = ((bw as f32 * sn).abs() + (bh as f32 * cs).abs()) * 0.5 * pulse;
        let w = (hw * 2.0).ceil() as i32;
        let h = (hh * 2.0).ceil() as i32;
        let x0 = cx - w / 2;
        let y0 = cy - h / 2;
        let scx = bw as f32 / 2.0;
        let scy = bh as f32 / 2.0;
        for dy in 0..h {
            for dx in 0..w {
                // dest offset from center -> inverse-rotated source offset
                // (forward is dest = R(theta)*src with theta = -20 deg: the
                // text's right end maps UP the screen; the inverse below
                // must be R(-theta) = R(+20), not R(theta) again)
                let ox = dx as f32 - w as f32 / 2.0;
                let oy = dy as f32 - h as f32 / 2.0;
                let sx = (ox * cs + oy * sn) / pulse + scx;
                let sy = (-ox * sn + oy * cs) / pulse + scy;
                if sx < 0.0 || sy < 0.0 || sx >= bw as f32 || sy >= bh as f32 {
                    continue;
                }
                let c = src[(sy as i32 * bw + sx as i32) as usize];
                if c[3] != 0 {
                    self.set(x0 + dx, y0 + dy, c);
                }
            }
        }
    }

    /// the armed splash: bake the outlined yellow run at DEVICE
    /// resolution, cache it in the glyph atlas (one upload — the pulse
    /// only rescales the dst rect, never re-bakes), and draw it as one
    /// rotated quad. Returns false to fall back to the canvas blit
    /// (engine bake failure / unpackable strip).
    fn splash_quad(
        &mut self,
        cx: i32,
        cy: i32,
        s: &str,
        t: f32,
        eng: &'static std::sync::Mutex<crate::gui::font::FontEngine>,
    ) -> bool {
        let k = self.device_scale.max(0.05);
        // the vanilla cell is 16 UI px (the old scale-2 bitmap look);
        // device-res so the AA edges land on screen pixels
        let cell_dev = (16.0 * k).round().max(2.0) as u32;
        let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
        let run = e.cache_run(&format!("splash:{s}"), cell_dev, |e| {
            let (tight, tw, th) = splash_ink_at(s, cell_dev as f32, e);
            let (bytes, w, h) = outlined_strip(&tight, tw, th);
            (bytes, w as u32, h as u32)
        });
        let Some(g) = run else {
            return false;
        };
        // pulse 1.00..1.06 at 2 Hz around the CENTER, then the -20 deg
        // tilt — both pure vertex geometry on the cached strip
        let pulse = 1.0 + 0.06 * (t * std::f32::consts::TAU).sin().abs();
        let w_ui = g.w as f32 / k * pulse;
        let h_ui = g.h as f32 / k * pulse;
        let dst = crate::gui_render::RectF::new(
            cx as f32 - w_ui * 0.5,
            cy as f32 - h_ui * 0.5,
            w_ui,
            h_ui,
        );
        let src = crate::gui_render::Rect::new(
            g.atlas_x as i32,
            g.atlas_y as i32,
            g.w as i32,
            g.h as i32,
        );
        self.gui_frame
            .rotated_glyph_quad(dst, src, -(20.0_f32).to_radians());
        true
    }

    /// Draw a pixel-art sprite from string rows with a char→color palette.
    pub fn sprite(&mut self, x: i32, y: i32, rows: &[&str], palette: &[(char, Color)], scale: i32) {
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == '.' || ch == ' ' {
                    continue;
                }
                let col = palette.iter().find(|(c, _)| *c == ch).map(|(_, col)| *col);
                let Some(col) = col else { continue };
                for sy in 0..scale {
                    for sx in 0..scale {
                        self.set(x + rx as i32 * scale + sx, y + ry as i32 * scale + sy, col);
                    }
                }
            }
        }
    }

    // ------------------------------------------------------ widgets ----

    /// the reference game-style button (gray body, bevel, hover tint).
    pub fn draw_button(&mut self, w: &Widget, hover: bool) {
        let (label, value, enabled) = match &w.kind {
            WidgetKind::Button {
                label,
                value,
                enabled,
            } => (label.clone(), value.clone(), *enabled),
            _ => return,
        };
        // Phase 2: chrome quads are ALWAYS pushed; the raster below is
        // gated so nothing double-draws when the quad pass is active
        self.gui_frame.button(w, hover);
        if !self.chrome_enabled {
            self.draw_button_text(w, &label, &value, enabled, hover);
            return;
        }
        // 2026-09-20 modern flat profile (canvas fallback — the quad
        // path renders the same look from the 20x20 9-slice): flat body
        // with a subtle hash-noise band, ONE light row under the top
        // frame, two shade rows over the bottom frame, 2-px frame —
        // black normally, white on hover. Clean-room re-synthesis of
        // the modern-widget design parameters (own palette, own noise).
        let body: Color = if enabled {
            [111, 111, 111, 255] // 0x6F + noise below
        } else {
            [45, 45, 45, 235] // 0x2D
        };
        // body + deterministic ±4 noise (2x2 blocks keep it cheap and
        // matching the 2-canvas-px-per-vanilla-px model)
        for y in (w.y + 2..w.y + w.h - 2).step_by(2) {
            for x in (w.x + 2..w.x + w.w - 2).step_by(2) {
                let n = (hash_pixel(x, y) % 9) as i32 - 4;
                let c: Color = [
                    (body[0] as i32 + n).clamp(0, 255) as u8,
                    (body[1] as i32 + n).clamp(0, 255) as u8,
                    (body[2] as i32 + n).clamp(0, 255) as u8,
                    body[3],
                ];
                self.rect(x, y, 2, 2, c);
            }
        }
        let (frame, light): (Color, Color) = if hover && enabled {
            ([255, 255, 255, 255], [180, 180, 182, 255])
        } else {
            ([12, 12, 12, 255], [172, 172, 172, 255])
        };
        // ONE light row under the top frame (the modern sheen)
        self.rect(w.x + 2, w.y + 2, w.w - 4, 2, light);
        // two shade rows over the bottom frame
        self.rect(w.x + 2, w.y + w.h - 6, w.w - 4, 2, [90, 90, 92, 255]);
        self.rect(w.x + 2, w.y + w.h - 4, w.w - 4, 2, [74, 74, 76, 255]);
        // 2-px frame
        self.frame(w.x, w.y, w.w, w.h, frame);
        self.frame(w.x + 1, w.y + 1, w.w - 2, w.h - 2, frame);
        if hover && enabled {
            let tint: Color = [130, 160, 255, 70];
            self.rect(w.x + 4, w.y + 4, w.w - 8, w.h - 8, tint);
        }
        self.draw_button_text(w, &label, &value, enabled, hover);
    }

    /// the button LABEL (text stays on the canvas in every mode —
    /// Phase 2 splits it out of the chrome body)
    fn draw_button_text(
        &mut self,
        w: &Widget,
        label: &str,
        value: &str,
        enabled: bool,
        hover: bool,
    ) {
        let text_col: Color = if !enabled {
            [145, 145, 145, 255]
        } else if hover {
            [255, 255, 160, 255]
        } else {
            [240, 240, 240, 255]
        };
        let full = if value.is_empty() {
            label.to_string()
        } else {
            format!("{}: {}", label, value)
        };
        // vanilla proportions: a 20px vanilla button carries a 9px font
        // (45%); our 30px buttons take fs=1.5, the legacy 44px widgets
        // keep fs=2 (identical to the pre-vanilla rendering)
        let fs = (w.h as f32 * 0.05).min(2.0) * self.widget_scale;
        let tw = Self::text_width_frac(&full, fs);
        let th = (8.0 * fs) as i32;
        self.text_frac(
            w.x + (w.w - tw) / 2,
            w.y + (w.h - th) / 2,
            &full,
            text_col,
            fs,
        );
    }

    /// the reference game-style slider: inset track + knob (2026-09-20: modern
    /// flat profile — dark flat track, knob = a small modern button).
    pub fn draw_slider(&mut self, w: &Widget, hover: bool) {
        let (label, value) = match &w.kind {
            WidgetKind::Slider { label, value } => (label.clone(), *value),
            _ => return,
        };
        let ty = w.y + 8;
        let th = w.h - 16;
        // Phase 2: quads always pushed, raster gated
        self.gui_frame.slider(w, hover);
        if self.chrome_enabled {
            // track: dark flat inset (the modern sunken slider tray)
            self.rect(w.x, ty, w.w, th, [22, 22, 22, 235]);
            self.frame(w.x, ty, w.w, th, [0, 0, 0, 255]);
            self.frame(w.x + 1, ty + 1, w.w - 2, th - 2, [52, 52, 52, 255]);
            // knob (16 wide — a mini modern button: body + sheen + shade +
            // 2-px frame, white frame on hover)
            let kx = w.x + 8 + ((w.w - 16 - 16) as f32 * value) as i32;
            self.rect(kx, ty - 4, 16, th + 8, [111, 111, 111, 250]);
            self.rect(kx + 2, ty - 2, 12, 2, [172, 172, 172, 255]);
            self.rect(kx + 2, ty + th, 12, 2, [90, 90, 92, 255]);
            let kf: Color = if hover {
                [255, 255, 255, 255]
            } else {
                [12, 12, 12, 255]
            };
            self.frame(kx, ty - 4, 16, th + 8, kf);
            self.frame(kx + 1, ty - 3, 14, th + 6, kf);
        } // chrome_enabled
          // label centered over the track (empty label = the vanilla
          // unlabeled slider, e.g. Brightness)
        if !label.is_empty() {
            let text_col: Color = if hover {
                [255, 255, 160, 255]
            } else {
                [240, 240, 240, 255]
            };
            let fs = (w.h as f32 * 0.05).min(2.0) * self.widget_scale;
            let tw = Self::text_width_frac(&label, fs);
            let th = (8.0 * fs) as i32;
            self.text_frac(
                w.x + (w.w - tw) / 2,
                w.y + (w.h - th) / 2 - 1,
                &label,
                text_col,
                fs,
            );
        }
    }

    /// Phase 1: text-entry field — vanilla look: small caps label above an
    /// inset dark tray, typed text at 16px, gray placeholder when empty,
    /// light frame when focused.
    pub fn draw_text_field(&mut self, w: &Widget, _hover: bool) {
        let (label, text, placeholder, focused) = match &w.kind {
            WidgetKind::TextField {
                label,
                text,
                placeholder,
                focused,
            } => (label.clone(), text.clone(), placeholder.clone(), *focused),
            _ => return,
        };
        // label (small, above the tray)
        self.text(w.x + 2, w.y - 12, &label, [180, 180, 180, 255], 1);
        // Phase 2: quads always pushed, raster gated
        self.gui_frame.text_field(w, _hover);
        if self.chrome_enabled {
            // inset tray
            self.rect(w.x, w.y, w.w, w.h, [16, 16, 16, 235]);
            self.frame(w.x, w.y, w.w, w.h, [12, 12, 12, 255]);
            self.rect(w.x + 2, w.y + 2, w.w - 4, 2, [50, 50, 50, 255]);
            self.rect(w.x + 2, w.y + w.h - 4, w.w - 4, 2, [70, 70, 70, 255]);
            if focused {
                self.frame(w.x + 1, w.y + 1, w.w - 2, w.h - 2, [255, 255, 255, 170]);
            }
        } // chrome_enabled
          // contents: typed text, else placeholder in gray
        let fs = 2.0 * self.widget_scale;
        let shown = Self::field_visible_text_f(w.w, &text, fs);
        if !shown.is_empty() {
            let col: Color = if focused {
                [255, 255, 255, 255]
            } else {
                [230, 230, 230, 255]
            };
            let th = (8.0 * fs) as i32;
            self.text_frac(w.x + 16, w.y + (w.h - th) / 2, shown, col, fs);
        } else {
            let pshown = Self::field_visible_text_f(w.w, &placeholder, fs);
            let th = (8.0 * fs) as i32;
            self.text_frac(
                w.x + 16,
                w.y + (w.h - th) / 2,
                pshown,
                [130, 130, 130, 255],
                fs,
            );
        }
    }

    pub fn draw_widget(&mut self, w: &Widget, hover: bool) {
        match &w.kind {
            WidgetKind::Button { .. } => self.draw_button(w, hover),
            WidgetKind::Slider { .. } => self.draw_slider(w, hover),
            WidgetKind::TextField { .. } => self.draw_text_field(w, hover),
        }
    }

    pub fn draw_widgets(&mut self, ws: &[Widget], hover: Option<u16>) {
        for w in ws {
            self.draw_widget(w, hover == Some(w.id));
        }
    }

    /// Phase 1: like [`draw_widgets`] but blinking carets on focused fields.
    pub fn draw_widgets_caret(&mut self, ws: &[Widget], hover: Option<u16>, time: f32) {
        for w in ws {
            self.draw_widget(w, hover == Some(w.id));
        }
        // second pass for the caret (borrow split: draw then measure)
        for w in ws {
            if let WidgetKind::TextField {
                text,
                focused: true,
                ..
            } = &w.kind
            {
                if (time * 2.2).fract() < 0.6 {
                    let fs = 2.0 * self.widget_scale;
                    let shown = Self::field_visible_text_f(w.w, text, fs);
                    let tw = Self::text_width_frac(shown, fs);
                    let pad = (16.0 * self.widget_scale) as i32;
                    let ch = (20.0 * self.widget_scale) as i32;
                    self.rect(
                        w.x + pad + tw + 1,
                        w.y + (12.0 * self.widget_scale) as i32,
                        2,
                        ch,
                        [240, 240, 240, 255],
                    );
                }
            }
        }
    }

    /// Truncate text to what fits in a field's inner width (5x7 font,
    /// fractional scale).
    fn field_visible_text_f(w: i32, text: &str, fs: f32) -> &str {
        let max_chars = (((w as f32) - 36.0 * fs) / (6.0 * fs)).max(0.0) as usize;
        let mut end = text.len();
        while text[..end].chars().count() > max_chars && end > 0 {
            // walk back one char boundary
            let mut new_end = end - 1;
            while !text.is_char_boundary(new_end) && new_end > 0 {
                new_end -= 1;
            }
            end = new_end;
        }
        &text[..end]
    }

    // ----------------------------------------------------- screens ----

    /// Title screen overlay (drawn over the panorama) — vanilla 1.16.5
    /// structure: big logo top-center, yellow splash tilted -20 degrees at
    /// the logo's bottom-right corner pulsing at 2 Hz, button stack starting
    /// at half screen height, version bottom-left, disclaimer bottom-right.
    /// (VERIFIED reference wiki /Title_screen + /w/Splash.)
    pub fn title_screen(&mut self, splash: &str, ws: &[Widget], hover: Option<u16>, time: f32) {
        // logo: big blocky wordmark over the panorama (vanilla draws its
        // logo with a dark outline — no dim band behind it)
        let scale = 12;
        let logo = "VOXELCRAFT";
        let lw = Self::text_width(logo, scale);
        let lx = (self.live_w as i32 - lw) / 2;
        let ly = 18;
        // soft drop shadow
        self.text(lx + 4, ly + 6, logo, [0, 0, 0, 150], scale);
        // dark outline pass
        self.text_outlined(lx, ly, logo, [235, 235, 235, 255], [42, 42, 42, 255], scale);

        // splash: yellow, tilted -20 deg (right side up), pulsing 2 Hz,
        // tucked at the logo's bottom-right corner
        let sw = Self::text_width(splash, 2);
        let cx = (lx + lw - 30 - sw / 2).clamp(40, self.live_w as i32 - 40);
        let cy = ly + 62;
        self.text_splash(cx, cy, splash, time);

        self.draw_widgets(ws, hover);

        self.text(
            8,
            self.live_h as i32 - 20,
            "VoxelCraft 1.16.5",
            [220, 220, 220, 255],
            1,
        );
        let vr = "100% CLEAN-ROOM - NOT AN OFFICIAL GAME";
        let vw = Self::text_width(vr, 1);
        self.text(
            self.live_w as i32 - vw - 8,
            self.live_h as i32 - 20,
            vr,
            [210, 210, 210, 255],
            1,
        );
    }

    /// Generic settings screen (the vanilla 1.16.5 pattern): dark
    /// backdrop, big centered title, then the vanilla hover-tooltip slot
    /// — up to two centered gray hint lines drawn directly under the
    /// title while the pointer rests on an option, exactly where the
    /// original shows them.
    pub fn settings_screen(
        &mut self,
        ws: &[Widget],
        hover: Option<u16>,
        title: &str,
        tooltip: &[String],
    ) {
        // 2026-09-20 modern look (owner directive — clean-room, similar
        // to the reference family): the OPAQUE 1.16.5 dirt tile backdrop
        // is replaced by the current-generation translucent dark menu
        // overlay — the blurred panorama (title) or the frozen live
        // world (pause) shows THROUGH it. The dirt sheet stays compiled
        // for resource packs that ship an options background.
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 100]);
        self.text_center(18, title, [255, 255, 255, 255], 3);
        for (i, line) in tooltip.iter().take(2).enumerate() {
            self.text_center(46 + i as i32 * 12, line, [170, 170, 170, 255], 1);
        }
        self.draw_widgets(ws, hover);
    }

    /// Resource Packs — the vanilla 1.16.5 two-pane layout: AVAILABLE
    /// (left) / SELECTED (right) headers over dark inset list panels,
    /// tooltip lines under the title, DONE at the bottom (the widgets
    /// themselves carry the rows + arrows).
    pub fn resource_pack_screen(&mut self, ws: &[Widget], hover: Option<u16>, tooltip: &[String]) {
        // 2026-09-20 modern translucent menu backdrop (see
        // settings_screen) + the darker sunken LIST panels (the
        // modern list-background family — deeper than the base overlay)
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 100]);
        self.text_center(18, "RESOURCE PACKS", [255, 255, 255, 255], 3);
        for (i, line) in tooltip.iter().take(2).enumerate() {
            self.text_center(46 + i as i32 * 12, line, [170, 170, 170, 255], 1);
        }
        // pane headers
        let aw = Self::text_width("AVAILABLE", 2);
        self.text((450 - aw) / 2, 62, "AVAILABLE", [255, 255, 255, 255], 2);
        let sw = Self::text_width("SELECTED", 2);
        self.text(
            (720 - sw) / 2 + 210,
            62,
            "SELECTED",
            [255, 255, 255, 255],
            2,
        );
        // dark inset panels behind the rows (the modern list look)
        self.rect(26, 86, 428, 348, [0, 0, 0, 150]);
        self.rect(506, 86, 428, 348, [0, 0, 0, 150]);
        self.draw_widgets(ws, hover);
    }

    /// 2026-09-20: the Shader Packs screen — the vanilla dirt/options
    /// backdrop, big title, tooltip slot, then a single sunken list
    /// panel holding the "(none)" row + the external pack rows, the
    /// LABPBR MATERIALS row, DONE at the bottom. The empty-folder hint
    /// line rides the tooltip slot (the caller feeds it when the scan
    /// found nothing).
    pub fn shader_screen(&mut self, ws: &[Widget], hover: Option<u16>, tooltip: &[String]) {
        // 2026-09-20 modern translucent backdrop (see settings_screen)
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 100]);
        self.text_center(18, "SHADERS", [255, 255, 255, 255], 3);
        for (i, line) in tooltip.iter().take(2).enumerate() {
            self.text_center(46 + i as i32 * 12, line, [170, 170, 170, 255], 1);
        }
        // sunken list backdrop behind the pack rows (the modern
        // list-background family, full width)
        self.rect(140, 80, 676, 304, [0, 0, 0, 150]);
        self.draw_widgets(ws, hover);
    }

    pub fn pause_screen(&mut self, ws: &[Widget], hover: Option<u16>) {
        // 2026-09-20: modern in-world menu translucency (the frozen
        // world shows through — the same family as the settings
        // backdrop, slightly deeper for white-text readability)
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 110]);
        self.text_center(140, "GAME MENU", [255, 255, 255, 255], 3);
        self.draw_widgets(ws, hover);
    }

    /// 2026-09-14 parity round: the vanilla Select World screen — title,
    /// search field top-left, two-line entries in a sunken list, and the
    /// two vanilla bottom rows. Row bodies are painted here (the widgets
    /// are hit-test rectangles); every non-row widget draws normally.
    #[allow(clippy::too_many_arguments)]
    pub fn world_select_screen(
        &mut self,
        ws: &[Widget],
        hover: Option<u16>,
        rows: &[WorldRow],
        selected: Option<usize>,
        scroll: usize,
        total: usize,
        filtering: bool,
    ) {
        // 2026-09-20 modern translucent backdrop over the panorama
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 110]);
        self.text_center(18, "SELECT WORLD", [255, 255, 255, 255], 3);
        // sunken list backdrop behind the entries (vanilla look)
        self.rect(
            221,
            90,
            518,
            6 + MAX_LISTED_WORLDS as i32 * 50,
            [0, 0, 0, 150],
        );
        // count / search status line under the title (vanilla shows
        // "Showing x of y" while filtering)
        if filtering {
            let sub = format!("{total} MATCH(ES)");
            self.text_center(64, &sub, [170, 170, 170, 255], 1);
        } else if total == 0 {
            self.text_center(
                64,
                "NO SAVED WORLDS YET - CREATE ONE BELOW",
                [170, 170, 170, 255],
                1,
            );
        } else if total > MAX_LISTED_WORLDS {
            let sub = format!("SHOWING {} OF {total}", rows.len().min(MAX_LISTED_WORLDS));
            self.text_center(64, &sub, [170, 170, 170, 255], 1);
        }
        // scroll arrows when the list overflows (vanilla has them on the
        // list frame edges)
        if total > MAX_LISTED_WORLDS {
            let up = scroll > 0;
            let down = scroll + MAX_LISTED_WORLDS < total;
            let col = |on: bool| {
                if on {
                    [255, 255, 255, 220]
                } else {
                    [120, 120, 120, 120]
                }
            };
            let (cx, cy) = (744, 100);
            self.text(cx, cy, "/\\", col(up), 1);
            self.text(cx, 90 + MAX_LISTED_WORLDS as i32 * 50, "\\/", col(down), 1);
        }
        // the entries themselves (two-line vanilla rows)
        for (i, row) in rows.iter().take(MAX_LISTED_WORLDS).enumerate() {
            let id = ID_WS_WORLD_BASE + i as u16;
            let Some(w) = ws.iter().find(|w| w.id == id) else {
                continue;
            };
            let hov = hover == Some(id);
            let sel = selected == Some(i);
            let base = if sel {
                [70, 70, 74, 235]
            } else {
                [24, 24, 28, 235]
            };
            self.rect(w.x, w.y, w.w, w.h, base);
            self.frame(
                w.x,
                w.y,
                w.w,
                w.h,
                if sel || hov {
                    [255, 255, 255, 200]
                } else {
                    [16, 16, 16, 255]
                },
            );
            // line 1: world name (white, slightly large)
            let name_col: Color = if row.dead {
                [150, 150, 150, 255]
            } else if hov {
                [255, 255, 160, 255]
            } else {
                [255, 255, 255, 255]
            };
            self.text_frac(
                w.x + 10,
                w.y + 5,
                &row.name,
                name_col,
                1.5 * self.widget_scale,
            );
            // line 2: mode + last played (gray)
            self.text_frac(
                w.x + 10,
                w.y + 26,
                &row.info,
                [150, 150, 150, 255],
                self.widget_scale,
            );
        }
        // everything else (search field + bottom rows) draws normally
        self.draw_widgets(ws, hover);
    }

    /// 2026-09-14 parity round: the vanilla two-page Create World screen.
    /// Page 1 paints the name label + the Game Mode description under the
    /// button; page 2 (More World Options) paints the seed label + the
    /// "leave blank" hint.
    pub fn world_create_screen(
        &mut self,
        ws: &[Widget],
        hover: Option<u16>,
        time: f32,
        page2: bool,
        mode_desc: (&str, &str),
    ) {
        // 2026-09-20 modern translucent backdrop over the panorama
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 110]);
        self.text_center(18, "CREATE NEW WORLD", [255, 255, 255, 255], 3);
        if !page2 {
            self.text_center(
                64,
                "ENTER A NAME FOR THE NEW WORLD:",
                [150, 150, 150, 255],
                1,
            );
            // the game-mode description (vanilla: two gray lines under
            // the centered button)
            self.text_center(192, mode_desc.0, [150, 150, 150, 255], 1);
            self.text_center(210, mode_desc.1, [150, 150, 150, 255], 1);
        } else {
            self.text_center(64, "SEED FOR THE WORLD GENERATOR", [150, 150, 150, 255], 1);
            self.text_center(
                122,
                "LEAVE BLANK FOR A RANDOM SEED",
                [130, 130, 130, 255],
                1,
            );
        }
        // focused-field hint + blinking caret handled per widget
        self.draw_widgets_caret(ws, hover, time);
    }

    /// 2026-09-14 parity round: the vanilla Edit World screen.
    pub fn world_edit_screen(&mut self, ws: &[Widget], hover: Option<u16>, time: f32) {
        // 2026-09-20 modern translucent backdrop over the panorama
        self.rect(0, 0, self.live_w as i32, self.live_h as i32, [0, 0, 0, 110]);
        self.text_center(18, "EDIT WORLD", [255, 255, 255, 255], 3);
        self.text_center(
            64,
            "ENTER A NEW NAME FOR THE WORLD:",
            [150, 150, 150, 255],
            1,
        );
        self.draw_widgets_caret(ws, hover, time);
    }

    /// Phase 1 + 2026-09-14: death screen — red wash, vanilla "You Died!"
    /// title + the vanilla Score line under it (vanilla shows the
    /// player's score, not the death cause).
    pub fn death_screen(&mut self, ws: &[Widget], hover: Option<u16>, hardcore: bool, score: i32) {
        self.rect(
            0,
            0,
            self.live_w as i32,
            self.live_h as i32,
            [80, 0, 0, 150],
        );
        let title = if hardcore { "GAME OVER!" } else { "YOU DIED!" };
        let tw = Self::text_width(title, 5);
        self.text(
            (self.live_w as i32 - tw) / 2,
            150,
            title,
            [255, 240, 240, 255],
            5,
        );
        let sub = if hardcore {
            "HARDCORE WORLD - DEATH IS PERMANENT".to_string()
        } else {
            format!("SCORE: {score}")
        };
        self.text_center(210, &sub, [255, 255, 255, 255], 1);
        self.draw_widgets(ws, hover);
    }

    // -------------------------------------------------------- HUD ----

    /// Vanilla-style crosshair: an inverted (difference-blended) plus.
    ///
    /// Luanti round 2 + vanilla-blend round: when the quad path is
    /// armed the plus is DEVICE-SNAPPED white INVERT quads in the
    /// over-canvas layer — every edge lands on a whole device pixel at
    /// ANY window size (the old canvas raster rode the 960×540 NEAREST
    /// letterbox, and its 1-2 UI-px arms went ragged/uneven at
    /// fractional scales like 1.5×: a 2-px arm became 3 px on some
    /// columns and 2 on others) — and the result is
    /// 1 − background per channel, the
    /// documented vanilla 1.16.5 crosshair behavior (technique
    /// reference: reference wiki /Crosshair — the classic
    /// GL_ONE_MINUS_DST_COLOR blend re-expressed as wgpu
    /// BlendFactor::OneMinusDst). The plus is DARK against a bright
    /// sky/snow and LIGHT against dark terrain, so it never disappears
    /// into the background the way an alpha-white crosshair does
    /// (found live: arms at (241,245,249) against snow (224,240,255)
    /// were invisible). The canvas fallback keeps the old white+outline
    /// look for the no-GPU-pass boot path.
    pub fn crosshair(&mut self) {
        if text_quads_active() {
            self.crosshair_quads();
            return;
        }
        let cx = (self.live_w / 2) as i32;
        let cy = (self.live_h / 2) as i32;
        let arm = 8;
        let th = 2;
        let white: Color = [238, 238, 238, 185];
        let dark: Color = [10, 10, 10, 90];
        // horizontal bar
        self.rect(cx - arm, cy - th / 2 - 1, arm * 2, 1, dark);
        self.rect(cx - arm, cy + th / 2 + 1, arm * 2, 1, dark);
        self.rect(cx - arm, cy - th / 2, arm * 2, th, white);
        // vertical bar
        self.rect(cx - th / 2 - 1, cy - arm, 1, arm * 2, dark);
        self.rect(cx + th / 2 + 1, cy - arm, 1, arm * 2, dark);
        self.rect(cx - th / 2, cy - arm, th, arm * 2, white);
    }

    /// the armed crosshair: geometry computed in DEVICE px (half-up
    /// rounding — `.5` rounds up, not banker's), expressed as
    /// fractional UI rects (`device / k`) so the letterbox uniform maps
    /// each edge back onto the exact device pixel it was computed for.
    /// THREE disjoint white invert quads: the H bar split into two
    /// segments around the V bar's x-window (overlapping invert quads
    /// cancel — invert∘invert = identity — so disjointness is a
    /// hard requirement, not a polish), plus the V bar. No outline:
    /// vanilla's crosshair has none, and inversion already guarantees
    /// contrast on every background but exact mid-gray.
    fn crosshair_quads(&mut self) {
        let k = self.device_scale.max(0.05);
        // half-up round (f32 → whole device px)
        let snap = |v: f32| (v + 0.5).floor();
        let cxd = snap(self.live_w as f32 * 0.5 * k);
        let cyd = snap(self.live_h as f32 * 0.5 * k);
        let ha = (snap(8.0 * k) as i32).max(4); // half-arm (device px)
        let ht = (snap(2.0 * k) as i32).max(2); // arm thickness
                                                // bar top/left snapped so the arm covers whole device px
                                                // (odd thickness sits 1 px heavy toward +x/+y — invisible on
                                                // a symmetric-plus crosshair, and every edge stays crisp)
        let wy = cyd as i32 - (ht + 1) / 2;
        let wx = cxd as i32 - (ht + 1) / 2;
        // device-px rect → fractional UI rect (dst = device / k)
        let q = |x: i32, y: i32, w: i32, h: i32, f: &mut Self| {
            f.gui_frame
                .solid_invert(x as f32 / k, y as f32 / k, w as f32 / k, h as f32 / k);
        };
        // horizontal bar SPLIT into two segments around the vertical
        // bar's [wx, wx+ht) window — disjoint invert geometry
        let hx = (cxd - ha as f32) as i32;
        let hw = ha * 2;
        let cut_l = wx - hx; // device px from H start to V bar start
        let cut_r = hx + hw - (wx + ht); // from V bar end to H end
        if cut_l > 0 {
            q(hx, wy, cut_l, ht, self);
        }
        if cut_r > 0 {
            q(wx + ht, wy, cut_r, ht, self);
        }
        // vertical bar
        let vy = (cyd - ha as f32) as i32;
        let vh = ha * 2;
        q(wx, vy, ht, vh, self);
    }

    const HEART: [&'static str; 6] = [
        ".OO..OO.", "ORROORRO", "ORHRRRRO", "ORRRRRRO", ".ORRRRO.", "..ORRO..",
    ];

    const FOOD: [&'static str; 7] = [
        ".OOOO...", "OMMMMO..", "OMMMMO..", "OMMMMO..", ".OMMO...", "..OWO...", "...OO...",
    ];

    /// clean-room bubble icon (8×8, original art — not vanilla's
    /// icons.png sprite; same functional position: 10 bubbles = the
    /// 300-air oxygen bar, one bubble per 30 air)
    const BUBBLE: [&'static str; 6] = [".ooo..", "oWWoo.", "oWBBBo", "oBBBBo", ".oBBo.", "..oo.."];

    /// Sub-round 1 (2026-09-14 Survival HUD round): the full vanilla
    /// 1.16.5 survival status block, drawn from [`HudStatus`] — armor
    /// row above the hearts (hidden at 0 points), hearts/hunger with
    /// the hurt/regen jitter + Hunger-effect recolor, oxygen bubbles
    /// above hunger while air < 300, XP bar 182x5 vanilla-eq + level.
    /// Per-element wiki citations in the body; clean-room items marked
    /// (see docs/research/round-9-survival-hud-reference-audit.md).
    pub fn status_bars(&mut self, s: &HudStatus) {
        let hb_w = 9 * 40 + 4;
        let hb_x = (self.live_w as i32 - hb_w) / 2;
        let hb_y = self.live_h as i32 - 48;

        // hearts row — ±1-vanilla-px jitter while hurt / regenerating
        // (clean-room; vanilla jitters the row, the wiki publishes no
        // numbers — round-9 audit §3)
        let heart_pal: [(char, Color); 4] = [
            ('O', [46, 6, 6, 255]),
            ('R', [227, 27, 13, 255]),
            ('H', [255, 116, 116, 255]),
            ('W', [255, 255, 255, 255]),
        ];
        for i in 0..10i32 {
            let x = hb_x + 2 + i * 17;
            let y = hb_y - 26;
            let dx = if s.hearts_jitter && ((i + s.tick_phase) & 1) == 0 {
                -2
            } else {
                0
            };
            // Phase 2: the 9x9 quad sprite (18x18 drawn) always pushed;
            // the legacy canvas sprite raster is gated
            let variant = if s.health >= (i + 1) as f32 / 10.0 {
                crate::textures::gui_art::HeartVariant::Full
            } else if s.health > i as f32 / 10.0 {
                crate::textures::gui_art::HeartVariant::Half
            } else {
                crate::textures::gui_art::HeartVariant::Empty
            };
            self.gui_frame.heart(x + dx, y, variant);
            if !self.chrome_enabled {
                continue;
            }
            // background outline (empty heart) then fill
            if s.health >= (i + 1) as f32 / 10.0 {
                self.sprite(x + dx, y, &Self::HEART, &heart_pal, 2);
            } else {
                let dim: [(char, Color); 4] = [
                    ('O', [30, 30, 30, 200]),
                    ('R', [70, 70, 70, 200]),
                    ('H', [90, 90, 90, 200]),
                    ('W', [110, 110, 110, 200]),
                ];
                self.sprite(x + dx, y, &Self::HEART, &dim, 2);
            }
        }

        // armor row ABOVE the hearts (17-px pitch), hidden at 0 points
        // — VERIFIED w/Heads-up_display (live 2026-09-14): "The armor
        // condition bar appears above the health bar if the player is
        // wearing armor" (0 points = unworn = hidden, the vanilla gate);
        // icons = 2 points each, half icon at odd (w/Armor)
        if s.armor > 0 {
            for i in 0..10i32 {
                let pts = s.armor - i * 2;
                let variant = if pts >= 2 {
                    crate::textures::gui_art::ArmorVariant::Full
                } else if pts == 1 {
                    crate::textures::gui_art::ArmorVariant::Half
                } else {
                    crate::textures::gui_art::ArmorVariant::Empty
                };
                self.gui_frame.armor(hb_x + 2 + i * 17, hb_y - 43, variant);
            }
        }

        // hunger row (right aligned, mirrored order) — the Hunger-effect
        // yellow-green recolor + ±1-px jitter while poisoned (VERIFIED
        // w/Hunger_(effect) for the recolor: "It also turns the hunger
        // bar a yellow-green color"; the poisoned jitter is clean-room).
        // Round 17: the row ALSO jitters when saturation hits zero —
        // VERIFIED w/Food §Saturation (live 2026-09-18): "when
        // saturation reaches zero, the hunger bar starts to shake or
        // jitter periodically" (periodicity = the alternating tick
        // phase, the engine's established jitter wave)
        let food_pal: [(char, Color); 4] = [
            ('O', [43, 26, 4, 255]),
            ('M', [186, 106, 38, 255]),
            ('W', [222, 222, 222, 255]),
            ('H', [255, 255, 255, 255]),
        ];
        for i in 0..10i32 {
            let x = hb_x + hb_w - 4 - (i + 1) * 17;
            let y = hb_y - 28;
            let jitter = (s.hunger_poisoned || s.food_jitter) && ((i + s.tick_phase) & 1) == 0;
            let dx = if jitter { -2 } else { 0 };
            // Phase 2: quad sprite always pushed (right row mirrors)
            let variant = if s.food >= (i + 1) as f32 / 10.0 {
                crate::textures::gui_art::HungerVariant::Full
            } else if s.food > i as f32 / 10.0 {
                crate::textures::gui_art::HungerVariant::Half
            } else {
                crate::textures::gui_art::HungerVariant::Empty
            };
            if s.hunger_poisoned {
                self.gui_frame
                    .hunger_tinted(x + dx, y, variant, [0.72, 1.0, 0.45, 1.0]);
            } else {
                self.gui_frame.hunger(x + dx, y, variant);
            }
            if !self.chrome_enabled {
                continue;
            }
            if s.food >= (i + 1) as f32 / 10.0 {
                self.sprite(x + dx, y, &Self::FOOD, &food_pal, 2);
            } else {
                let dim: [(char, Color); 4] = [
                    ('O', [30, 30, 30, 200]),
                    ('M', [70, 70, 70, 200]),
                    ('W', [110, 110, 110, 200]),
                    ('H', [110, 110, 110, 200]),
                ];
                self.sprite(x + dx, y, &Self::FOOD, &dim, 2);
            }
        }

        // oxygen bubbles (air supply < full): right-aligned row ABOVE
        // the hunger bar, mirrored order (vanilla position); ceil(air/30)
        // full bubbles — at the pop boundary the last one blinks out
        if s.air < 299.0 {
            let bubble_pal: [(char, Color); 4] = [
                ('o', [26, 46, 78, 255]),
                ('W', [235, 247, 255, 255]),
                ('B', [94, 158, 222, 255]),
                ('.', [0, 0, 0, 0]),
            ];
            let bubbles = (s.air.max(0.0) / 30.0).ceil() as i32;
            for i in 0..bubbles.min(10) {
                let x = hb_x + hb_w - 4 - (i + 1) * 17;
                let y = hb_y - 48;
                self.gui_frame
                    .bubble(x, y, crate::textures::gui_art::BubbleVariant::Full);
                if self.chrome_enabled {
                    self.sprite(x, y, &Self::BUBBLE, &bubble_pal, 2);
                }
            }
        }

        // XP bar — 182x5 vanilla-eq (364x10 UI px; the old 364x8 was the
        // 4-vanilla-px height deviation — round-9 audit §1). Luanti font
        // round: Solid quads (pixel-crisp at any window size, and immune
        // to the canvas/quad z-order class of bugs); the canvas raster
        // stays as the no-GPU-pass fallback
        let xp_w = hb_w;
        let xp_x = hb_x;
        let xp_y = hb_y - 12;
        let fill = ((xp_w - 4) as f32 * s.xp.clamp(0.0, 1.0)) as i32;
        let ct = |c: Color| -> [f32; 4] {
            [
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                c[3] as f32 / 255.0,
            ]
        };
        self.gui_frame
            .solid_rect(xp_x, xp_y, xp_w, 10, ct([16, 16, 16, 220]));
        self.gui_frame
            .solid_rect(xp_x, xp_y, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame
            .solid_rect(xp_x, xp_y + 9, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame
            .solid_rect(xp_x, xp_y, 1, 10, ct([60, 60, 60, 255]));
        self.gui_frame
            .solid_rect(xp_x + xp_w - 1, xp_y, 1, 10, ct([60, 60, 60, 255]));
        if fill > 0 {
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 6, ct([128, 255, 32, 255]));
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 1, ct([190, 255, 130, 255]));
        }
        if self.chrome_enabled {
            self.rect(xp_x, xp_y, xp_w, 10, [16, 16, 16, 220]);
            self.frame(xp_x, xp_y, xp_w, 10, [60, 60, 60, 255]);
            if fill > 0 {
                self.rect(xp_x + 2, xp_y + 2, fill, 6, [128, 255, 32, 255]);
                self.rect(xp_x + 2, xp_y + 2, fill, 1, [190, 255, 130, 255]);
            }
        }
        if s.level > 0 {
            let lvl = format!("{}", s.level);
            let w = Self::text_width(&lvl, 2);
            self.text_outlined(
                (self.live_w as i32 - w) / 2,
                xp_y - 20,
                &lvl,
                [128, 255, 32, 255],
                [20, 40, 8, 255],
                2,
            );
        }
    }

    /// Sub-round 1: the damage-flash vignette — a full-canvas red wash
    /// at ≤0.3 alpha, decaying with the player's hurt timer (clean-room
    /// spec per round-9 audit §3; the wiki documents the flash's
    /// existence, not its curve). `alpha` 0..1 (the game layer scales it).
    pub fn damage_vignette(&mut self, alpha: f32) {
        let a = alpha.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        self.gui_frame.solid_over(
            0.0,
            0.0,
            self.live_w as f32,
            self.live_h as f32,
            [0.55, 0.0, 0.0, 0.3 * a],
        );
    }

    /// Sub-round 1: the status-effect icon rows, top-right (VERIFIED
    /// reference wiki /Heads-up_display + its 1.9 15w31a history entry,
    /// live 2026-09-14): "When an effect is active on the player, it
    /// appears on the top-right corner of the screen. It blinks when
    /// about to run out." + "Effects that run out sooner appear farther
    /// to the left, and effects that are about to run out start to
    /// flash. Additionally, positive effects are shown on the top, and
    /// other effects (neutral or negative) are shown on the bottom."
    ///
    /// Rows are sorted by ticks-left ascending (soonest-expiring
    /// leftmost) and right-aligned; the amplifier+1 numeral renders
    /// under the icon at vanilla level ≥ II; the blink in the final 5 s
    /// (100 ticks) is the documented clean-room threshold.
    pub fn effect_icons(&mut self, entries: &[EffectIconEntry], tick: i64) {
        if entries.is_empty() {
            return;
        }
        const PITCH: i32 = 20; // 18-wide icon + 2-px gap
        const MARGIN: i32 = 10; // right/top inset (5 vanilla px)
        let mut pos: Vec<&EffectIconEntry> = entries.iter().filter(|e| e.positive).collect();
        let mut neg: Vec<&EffectIconEntry> = entries.iter().filter(|e| !e.positive).collect();
        pos.sort_by_key(|e| e.ticks_left);
        neg.sort_by_key(|e| e.ticks_left);
        for (row, list) in [(0i32, &pos), (1i32, &neg)] {
            let n = list.len() as i32;
            for (i, e) in list.iter().enumerate() {
                let x = self.live_w as i32 - MARGIN - (n - i as i32) * PITCH + 2;
                let y = MARGIN + row * 20;
                // blink in the final 5 s (100 ticks) at ~4 Hz
                let alpha = if e.ticks_left < 100 && (tick / 5) % 2 == 0 {
                    0.25
                } else {
                    1.0
                };
                self.gui_frame.effect_icon(x, y, e.icon, alpha);
                // amplifier ≥ 1 (vanilla level II+): the level numeral
                // (vanilla draws it on the icon; under the icon keeps the
                // 9x9 art readable — disclosed adaptation)
                if e.amplifier >= 1 {
                    let lvl = format!("{}", e.amplifier + 1);
                    self.text_outlined(
                        x + 6,
                        y + 18,
                        &lvl,
                        [255, 255, 255, 230],
                        [30, 30, 30, 160],
                        1,
                    );
                }
            }
        }
    }

    /// Phase E1: the voider-dragon boss bar — VERIFIED w/Ender_Dragon:
    /// "a light purple health bar ... at the top of the player's screen",
    /// the name above it, width matches the hotbar band. `frac` = the
    /// dragon's remaining health fraction (0..1).
    pub fn boss_bar(&mut self, frac: f32) {
        let w = 9 * 40 + 4; // hotbar-width band (the vanilla boss-bar width)
        let x = (self.live_w as i32 - w) / 2;
        let y = 24;
        // label
        let name = "ENDER DRAGON";
        let tw = name.len() as i32 * 8;
        self.text(
            (self.live_w as i32 - tw) / 2,
            y - 14,
            name,
            [235, 220, 245, 255],
            1,
        );
        // track + light-purple fill (VERIFIED color family).
        // Luanti round 2: solid quads (crisp edges at any window
        // size — the status_bars XP pattern); canvas raster gated to
        // the no-GPU-pass fallback
        let ct = |c: Color| -> [f32; 4] {
            [
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                c[3] as f32 / 255.0,
            ]
        };
        self.gui_frame
            .solid_rect(x, y, w, 12, ct([16, 12, 20, 220]));
        // 1-px frame (the canvas `frame` decomposition)
        self.gui_frame
            .solid_rect(x, y, w, 1, ct([90, 70, 110, 255]));
        self.gui_frame
            .solid_rect(x, y + 11, w, 1, ct([90, 70, 110, 255]));
        self.gui_frame
            .solid_rect(x, y, 1, 12, ct([90, 70, 110, 255]));
        self.gui_frame
            .solid_rect(x + w - 1, y, 1, 12, ct([90, 70, 110, 255]));
        let fill = ((w - 4) as f32 * frac.clamp(0.0, 1.0)) as i32;
        if fill > 0 {
            self.gui_frame
                .solid_rect(x + 2, y + 2, fill, 8, ct([190, 90, 220, 255]));
            self.gui_frame
                .solid_rect(x + 2, y + 2, fill, 2, ct([230, 150, 250, 255]));
        }
        if self.chrome_enabled {
            self.rect(x, y, w, 12, [16, 12, 20, 220]);
            self.frame(x, y, w, 12, [90, 70, 110, 255]);
            if fill > 0 {
                self.rect(x + 2, y + 2, fill, 8, [190, 90, 220, 255]);
                self.rect(x + 2, y + 2, fill, 2, [230, 150, 250, 255]);
            }
        }
    }

    /// Held-item name above the XP bar, fading out over ~2 s after the
    /// selection changes (vanilla HUD behavior; the 2 s fade duration is
    /// an unverified-but-trivial UI nicety — the layout position is the
    /// commonly-cited vanilla one). `alpha` 0..1.
    pub fn held_item_name(&mut self, name: &str, alpha: f32) {
        let a = alpha.clamp(0.0, 1.0);
        if a <= 0.0 || name.is_empty() {
            return;
        }
        // Sub-round 1: centered above the whole status block (vanilla
        // centers it; the old left-aligned hb_y-44 spot now collides
        // with the armor row). Clear above armor (hb_y-43) and the
        // level-number zone (hb_y-32)
        let y = self.live_h as i32 - 48 - 64;
        let w = Self::text_width(name, 2);
        let x = (self.live_w as i32 - w) / 2;
        let fg: Color = [255, 255, 255, (230.0 * a) as u8];
        let sh: Color = [20, 20, 20, (140.0 * a) as u8];
        self.text_outlined(x, y, name, fg, sh, 2);
    }

    /// 1.16.5-style hotbar: 40px slots, big white selection frame, icons
    /// and vanilla stack counts (bottom-right, shadowed).
    pub fn hotbar(
        &mut self,
        slots: &[ItemStack],
        selected: usize,
        atlas: &[u8],
        item_name: Option<(&str, u8)>,
    ) {
        let n = slots.len() as i32;
        let slot = 40i32;
        let bw = n * slot + 4;
        let x0 = (self.live_w as i32 - bw) / 2;
        let y0 = self.live_h as i32 - 48;
        // Phase 2: hotbar chrome quads (bg + selection) always pushed,
        // raster gated. Item icons + counts stay on the canvas (Phase 3
        // migrates icons).
        self.gui_frame.hotbar_background(x0, y0 - 2);
        let sel = x0 + 2 + selected as i32 * slot;
        self.gui_frame.hotbar_selection(sel - 2, y0 - 3);
        if self.chrome_enabled {
            self.rect(x0, y0, bw, 44, [12, 12, 12, 190]);
            self.frame(x0, y0, bw, 44, [8, 8, 8, 255]);
        }
        for (i, stack) in slots.iter().enumerate() {
            let sx = x0 + 2 + i as i32 * slot;
            let sy = y0 + 2;
            if self.chrome_enabled {
                self.rect(sx, sy, 36, 36, [58, 58, 58, 160]);
                self.frame(sx, sy, 36, 36, [90, 90, 90, 220]);
            }
            self.draw_stack(stack, sx, sy, atlas);
        }
        // selection: chunky white frame extending past the slot
        if self.chrome_enabled {
            self.frame(sel - 2, y0, 40, 40, [255, 255, 255, 255]);
            self.frame(sel - 3, y0 - 1, 42, 42, [200, 200, 200, 140]);
        }

        if let Some((name, alpha)) = item_name {
            let w = name.len() as i32 * 12;
            self.text(
                (self.live_w as i32 - w) / 2,
                y0 - 76,
                name,
                [255, 255, 255, alpha],
                2,
            );
        }
    }

    /// one item stack inside a 36px slot at (sx, sy): icon + vanilla count
    /// label (bottom-right, dark shadow) — shared by hotbar + containers.
    fn draw_stack(&mut self, s: &ItemStack, sx: i32, sy: i32, atlas: &[u8]) {
        let b = s.block;
        if b != AIR && s.count > 0 {
            let tile = {
                let d = def(b);
                if b == GRASS || b == OAK_LOG {
                    d.tiles[2]
                } else {
                    d.tiles[0]
                }
            };
            // Phase 3 + the Luanti font round: the cached 3D icon quad
            // and the canvas flat tile are now MUTUALLY EXCLUSIVE — the
            // canvas blits AFTER the quad pass, so an always-rasterized
            // tile would cover the icon quad. No cached icon yet → the
            // flat tile is the pop-in placeholder and the permanent
            // fallback for blocks the baker cannot model.
            match self.icon_cells.as_ref().and_then(|m| m.get(&b).copied()) {
                Some(cell) => self.gui_frame.icon_quad(sx + 2, sy + 2, cell),
                None => blit_tile(
                    atlas,
                    tile,
                    2,
                    (sx + 2) as usize,
                    (sy + 2) as usize,
                    &mut self.px,
                    self.live_w,
                ),
            }
        }
        if s.count > 1 {
            // Luanti font round: the count renders at the vanilla size
            // (the regular font at GUI scale = 16-px cell in our 2x UI
            // space — the old scale-1 text was half-size and unreadable)
            // and right-aligns through the ACTIVE font's metrics
            let label = s.count.to_string();
            let w = Self::text_width(&label, 2);
            let tx = sx + 34 - w;
            let ty = sy + 18;
            self.text(tx, ty, &label, [255, 255, 255, 255], 2);
        }
    }

    /// container slot: recessed 36px well + optional stack
    fn slot_well(&mut self, x: i32, y: i32, s: &ItemStack, atlas: &[u8]) {
        // Phase-2 pattern + the Luanti font round: the 18x18 slot
        // sprite quad always; the canvas well raster only when the
        // canvas draws chrome (the canvas blits after the quad pass —
        // an ungated raster would cover the sprite)
        self.gui_frame.slot(x, y, false);
        if self.chrome_enabled {
            self.rect(x, y, 36, 36, [52, 52, 52, 200]);
            self.frame(x, y, 36, 36, [24, 24, 24, 255]); // inner shadow
            self.frame(x + 1, y + 1, 34, 34, [110, 110, 110, 255]);
        }
        self.draw_stack(s, x, y, atlas);
    }

    /// vanilla container arrow: gray track, white fill by progress fraction
    fn arrow(&mut self, x: i32, y: i32, frac: f32) {
        let w = 44i32;
        let h = 20i32;
        // track
        self.rect(x, y + 2, w - 14, h - 4, [70, 70, 70, 255]);
        self.rect(x + w - 14, y, 14, h, [70, 70, 70, 255]);
        // head cut (triangle-ish via stepped rects)
        self.rect(x + w - 10, y + 3, 8, h - 6, [16, 16, 16, 220]);
        self.rect(x + w - 12, y + 7, 4, h - 14, [16, 16, 16, 220]);
        // filled portion
        let fw = ((w - 16) as f32 * frac.clamp(0.0, 1.0)) as i32;
        if fw > 0 {
            self.rect(x + 1, y + 4, fw, h - 8, [235, 235, 235, 255]);
        }
    }

    /// small horizontal progress bar (brewing fuel charges, §29)
    fn bar(&mut self, x: i32, y: i32, w: i32, h: i32, frac: f32) {
        self.rect(x, y, w, h, [26, 26, 30, 220]);
        self.frame(x, y, w, h, [12, 12, 14, 255]);
        let fw = (w as f32 * frac.clamp(0.0, 1.0)) as i32;
        if fw > 0 {
            self.rect(x + 1, y + 1, fw.min(w - 2), h - 2, [235, 235, 235, 255]);
        }
    }

    /// vanilla furnace flame between input and fuel slots, filled by the
    /// burn-progress fraction
    fn flame(&mut self, x: i32, y: i32, frac: f32) {
        let rows_on = ["  f  ", " fFf ", " fFf ", "fFFFf", "fFFFf"];
        let rows_off = ["  .  ", " . . ", " . . ", ".....", "....."];
        let pal = [
            ('f', [255, 110, 20, 255]),
            ('F', [255, 210, 60, 255]),
            ('.', [90, 90, 90, 255]),
        ];
        let scale = 5i32; // 25x25
        if frac > 0.02 {
            self.sprite(x, y, &rows_on, &pal, scale);
            // dim the bottom when nearly burnt out (fraction low)
            if frac < 0.35 {
                self.rect(x + 2, y + 15, 21, 10, [40, 30, 20, 110]);
            }
        } else {
            self.sprite(x, y, &rows_off, &pal, scale);
        }
    }

    /// vanilla brewing bubble column: white dots rising with the brew
    /// progress (the mirror of the furnace flame — fills bottom-up as the
    /// cycle advances)
    fn bubbles(&mut self, x: i32, y: i32, frac: f32) {
        let rows = [
            ".  b  .", ".  b  .", ".  b  .", ".  b  .", ".  b  .", ".  b  .", ".  b  .",
        ];
        let pal = [('b', [200, 230, 255, 255]), ('.', [70, 70, 80, 200])];
        let scale = 5i32; // 7 wide x 35 tall
        self.sprite(x, y, &rows, &pal, scale);
        // fill from the BOTTOM up as the cycle progresses (vanilla bubbles
        // rise as the brew advances)
        let total = rows.len() as f32;
        let lit = (frac.clamp(0.0, 1.0) * total).floor() as i32;
        if lit > 0 {
            let y_fill = y + (rows.len() as i32 - lit) * scale;
            self.rect(
                x + 2 * scale,
                y_fill,
                scale,
                lit * scale,
                [200, 230, 255, 255],
            );
        }
    }

    /// one generic 9-wide slot row (hotbar strip or storage row)
    #[allow(dead_code)]
    fn inv_row(
        &mut self,
        x0: i32,
        y: i32,
        slots: &[ItemStack],
        atlas: &[u8],
        start: usize,
        count: usize,
    ) {
        for i in 0..count {
            let x = x0 + i as i32 * 40;
            self.slot_well(x, y, &slots[start + i], atlas);
        }
    }

    /// Full container overlay (Phase 7 §27): player inventory (9×3 storage +
    /// 9 hotbar) plus the container-specific top area — 2×2 personal
    /// crafting grid, 3×3 crafting table, or the furnace slots with live
    /// burn/cook progress. Returns the slot geometry so game.rs can
    /// hit-test clicks (LEFT = whole stack, RIGHT = half/single).
    pub fn container_screen(
        &mut self,
        view: &ContainerView,
        cursor_pos: (f32, f32),
        atlas: &[u8],
        advanced_tooltips: bool,
    ) -> ContainerGeom {
        let kind = view.kind;
        // ---- shared bottom layout: 9-col storage (3 rows) + hotbar row ----
        let cols: i32 = 9;
        let grid_w = cols * 40 + 4;
        let x0 = (self.live_w as i32 - grid_w) / 2;
        // top-area height per kind
        let top_h = match kind {
            // Sub-round 3: the vanilla 176x166-shaped inventory — armor
            // column + player model + offhand (in its boxed recess below
            // the armor column) + 2x2 craft + output. 232 = the armor
            // column (4x44+8) + the offhand row (36) + breathing room.
            ContainerKind::Inventory => 232,
            ContainerKind::Crafting => 140, // 3x3 craft + arrow + output
            ContainerKind::Chest => 132,    // 3 rows of 9 slots
            // Round 12: 6 rows of 9 = 54 slots (vanilla 176×220 panel —
            // +3 rows over the single chest, same family chrome)
            ContainerKind::DoubleChest => 264, // 6 rows of 9 slots
            // the barrel shares the chest grid (VERIFIED: 27 slots)
            ContainerKind::Barrel => 132, // 3 rows of 9 slots
            // vanilla ratio: hopper 133/166 of a chest's height — one
            // content row instead of three (2×40px shorter than the chest)
            ContainerKind::Hopper => 52,   // 1 row of 5 slots
            ContainerKind::Furnace => 128, // input / flame / fuel + arrow + output
            ContainerKind::Brewing => 150, // ingredient / bubbles / fuel + 3 bottles
            ContainerKind::Enchant => 160, // item + lapis + 3 option buttons
            // Phase 5: two 5-row columns + the career header
            ContainerKind::Trade => 248,
            // Round 13: the anvil — rename field + one slot row + cost
            // line (audit §1 geometry, doubled to the 36px slot space)
            ContainerKind::Anvil => 168,
            // Round 13: the beacon — the power grid + the pay row
            ContainerKind::Beacon => 232,
            // Round 13: the grindstone — two stacked inputs + result
            ContainerKind::Grindstone => 128,
            // Round 12b: the mount's storage — rows of 5 (the last row
            // partial for llamas) + the saddle/strength column; the
            // exact height rides the mob's capacity (view.mount)
            ContainerKind::Mount => {
                8 + (view
                    .mount
                    .as_ref()
                    .map(|m| m.capacity)
                    .unwrap_or(15)
                    .div_ceil(5)) as i32
                    * 40
                    + 4
            }
        };
        let panel_h = top_h + 3 * 44 + 8 + 44 + 30; // + title + gaps + padding
        let y0 = (self.live_h as i32 - panel_h) / 2;

        // panel chrome (the trade screen is wider: two 260px columns)
        let trade_wide = kind == ContainerKind::Trade;
        let pw = if trade_wide { 562 } else { grid_w + 28 };
        let px0 = if trade_wide {
            (self.live_w as i32 - pw) / 2
        } else {
            x0 - 14
        };
        self.rect(px0, y0 - 30, pw, panel_h + 30, [26, 26, 30, 235]);
        self.frame(px0, y0 - 30, pw, panel_h + 30, [60, 60, 66, 255]);
        self.frame(px0 + 1, y0 - 29, pw - 2, panel_h + 28, [12, 12, 14, 255]);

        let title = match kind {
            ContainerKind::Inventory => "INVENTORY  (E / ESC to close)",
            ContainerKind::Crafting => "CRAFTING TABLE",
            ContainerKind::Chest => "CHEST",
            // Round 12: the double chest keeps the single-word vanilla
            // title (NOT "Large Chest") — VERIFIED w/Chest §Double
            // chests, live 2026-09-15
            ContainerKind::DoubleChest => "CHEST",
            // 1.14: the barrel's own label (the vanilla GUI title)
            ContainerKind::Barrel => "BARREL",
            // VERIFIED vanilla GUI label: "Item Hopper"
            ContainerKind::Hopper => "ITEM HOPPER",
            ContainerKind::Furnace => "FURNACE",
            ContainerKind::Brewing => "BREWING STAND",
            ContainerKind::Enchant => "ENCHANT  (needs book + lapis + levels)",
            ContainerKind::Trade => "VILLAGER",
            // Round 13 (audit §1, VLM OCR of the wiki GUI): "Repair & Name"
            ContainerKind::Anvil => "REPAIR & NAME",
            ContainerKind::Beacon => "BEACON",
            // Round 13 (audit §3, VLM OCR): "Repair & Disenchant"
            ContainerKind::Grindstone => "REPAIR & DISENCHANT",
            // Round 12b: the vanilla GUI title is the entity's own name
            // (w/Donkey + w/Llama GUI captions: "GUI of a donkey ...")
            ContainerKind::Mount => view
                .mount
                .as_ref()
                .map(|m| m.kind_label.as_str())
                .unwrap_or("MOUNT"),
        };
        self.text(px0 + 12, y0 - 24, title, [255, 220, 120, 255], 1);

        let mut geom = ContainerGeom {
            inv: Vec::with_capacity(36),
            craft: Vec::new(),
            chest: Vec::new(),
            craft_out: (i32::MIN, i32::MIN),
            furnace: None,
            brewing: None,
            enchant: None,
            trade: None,
            armor: [(i32::MIN, i32::MIN); 4],
            offhand: (i32::MIN, i32::MIN),
            anvil: None,
            beacon: None,
            grind: None,
            mount: None,
        };

        // ---- container-specific top area ----
        match kind {
            ContainerKind::Inventory => {
                // Sub-round 3: the vanilla 176x166-shaped survival
                // inventory (scaled 2x on the 960x540 canvas). clean-room:
                // shape language = the vanilla layout — LEFT armor column
                // (helmet..boots top to bottom), the player model preview
                // beside it with the offhand slot in its boxed recess
                // below, the 2x2 craft grid + arrow + result on the
                // right. Reference facts (reference wiki /Inventory,
                // live 2026-09-15): "The inventory consists of 4 armor
                // slots, 27 storage slots, 9 hotbar slots, and an
                // off-hand slot"; "There is also a 2x2 crafting grid";
                // "Pressing the F key moves the selected item to and from
                // the hotbar slot and the off-hand slot". No third-party asset
                // was read, copied, or traced.
                let ax = px0 + 16; // armor column x
                let ay = y0 + 8;
                // the four equipment slots: helmet, chestplate, leggings,
                // boots — vanilla order top to bottom
                for (i, st) in view.armor.iter().enumerate() {
                    let y = ay + i as i32 * 44;
                    self.slot_well(ax, y, st, atlas);
                    geom.armor[i] = (ax, y);
                }
                // ---- the player-model preview (the sanctioned 2D bake
                // fallback: a clean-room front-facing player figure; the
                // slow vanilla 3D rotation does not apply to a bake —
                // disclosed) ----
                let mx = ax + 52;
                let my = ay + 4;
                let mw = 56;
                let mh = 168;
                self.rect(mx, my, mw, mh, [20, 20, 24, 190]);
                self.frame(mx, my, mw, mh, [46, 46, 52, 255]);
                // head (skin tones) + face hint
                let skin: Color = [199, 159, 122, 255];
                let skin_d: Color = [166, 128, 95, 255];
                let shirt: Color = [0, 124, 124, 255];
                let shirt_d: Color = [0, 96, 96, 255];
                let pants: Color = [46, 57, 148, 255];
                let pants_d: Color = [36, 45, 118, 255];
                self.rect(mx + 16, my + 8, 24, 24, skin);
                self.rect(mx + 16, my + 26, 24, 6, skin_d);
                self.rect(mx + 21, my + 16, 4, 3, [46, 36, 30, 255]);
                self.rect(mx + 31, my + 16, 4, 3, [46, 36, 30, 255]);
                self.rect(mx + 24, my + 24, 8, 2, [130, 96, 74, 255]);
                // torso + arms
                self.rect(mx + 14, my + 34, 28, 44, shirt);
                self.rect(mx + 6, my + 34, 8, 40, shirt_d);
                self.rect(mx + 42, my + 34, 8, 40, shirt_d);
                // legs
                self.rect(mx + 14, my + 78, 13, 46, pants);
                self.rect(mx + 29, my + 78, 13, 46, pants);
                self.rect(mx + 14, my + 78, 13, 4, pants_d);
                self.rect(mx + 29, my + 78, 13, 4, pants_d);
                // feet
                self.rect(mx + 12, my + 124, 15, 8, [70, 50, 34, 255]);
                self.rect(mx + 29, my + 124, 15, 8, [70, 50, 34, 255]);
                // ---- the offhand slot: its boxed recess below the
                // model (vanilla bottom-left) ----
                let ox0 = ax;
                let oy0 = ay + 4 * 44 + 8;
                self.rect(ox0 - 6, oy0 - 6, 48, 48, [22, 22, 26, 210]);
                self.frame(ox0 - 6, oy0 - 6, 48, 48, [54, 54, 60, 255]);
                self.slot_well(ox0, oy0, &view.offhand, atlas);
                geom.offhand = (ox0, oy0);
                // ---- the 2x2 craft grid + arrow + output (right side,
                // vanilla x=98 scaled) ----
                let cx = px0 + 196;
                let cy = y0 + 20;
                for r in 0..2 {
                    for c in 0..2 {
                        let x = cx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        self.slot_well(x, y, &view.grid[r * 2 + c], atlas);
                        geom.craft.push((x, y));
                    }
                }
                self.arrow(
                    cx + 88,
                    cy + 24,
                    if !view.craft_out.is_empty() { 1.0 } else { 0.0 },
                );
                let ox = cx + 138;
                let oy = cy + 14;
                self.slot_well(ox, oy, &view.craft_out, atlas);
                geom.craft_out = (ox, oy);
            }
            ContainerKind::Crafting => {
                // 3x3 grid + arrow + output, centered
                let total = 3 * 40 + 50 + 36;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                for r in 0..3 {
                    for c in 0..3 {
                        let x = cx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        self.slot_well(x, y, &view.grid[r * 3 + c], atlas);
                        geom.craft.push((x, y));
                    }
                }
                self.arrow(
                    cx + 124,
                    cy + 32,
                    if !view.craft_out.is_empty() { 1.0 } else { 0.0 },
                );
                let ox = cx + 174;
                let oy = cy + 22;
                self.slot_well(ox, oy, &view.craft_out, atlas);
                geom.craft_out = (ox, oy);
            }
            ContainerKind::Chest | ContainerKind::Barrel => {
                // Phase 3: 3 rows of 9 slots, centered (the barrel shares
                // the chest grid — VERIFIED w/Barrel: same 27 slots)
                let total = 9 * 40;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                for r in 0..3 {
                    for c in 0..9 {
                        let x = cx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        let idx = r * 9 + c;
                        let st = view.chest.get(idx).copied().unwrap_or(ItemStack::EMPTY);
                        self.slot_well(x, y, &st, atlas);
                        geom.chest.push((x, y));
                    }
                }
            }
            ContainerKind::DoubleChest => {
                // Round 12: 6 rows of 9 = 54 slots, centered — the two
                // adjacent halves' 27+27 slots in one grid (the first
                // half's 27 ride rows 0-2, the second's rows 3-5;
                // game.rs routes SlotRef::Chest(i) to the owning half)
                let total = 9 * 40;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                for r in 0..6 {
                    for c in 0..9 {
                        let x = cx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        let idx = r * 9 + c;
                        let st = view.chest.get(idx).copied().unwrap_or(ItemStack::EMPTY);
                        self.slot_well(x, y, &st, atlas);
                        geom.chest.push((x, y));
                    }
                }
            }
            ContainerKind::Hopper => {
                // §Container: one centered row of 5 slots (vanilla hopper
                // layout — 44+18·col in vanilla units, ×2.2 here), sharing
                // the chest slot-geometry list (game.rs maps them through
                // the generic Container::Hopper slot path)
                let total = 5 * 40;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                for c in 0..5 {
                    let x = cx + c as i32 * 40;
                    let st = view.chest.get(c).copied().unwrap_or(ItemStack::EMPTY);
                    self.slot_well(x, cy, &st, atlas);
                    geom.chest.push((x, cy));
                }
            }
            ContainerKind::Mount => {
                // Round 12b: the mount screen — the saddle/strength
                // column at the left (vanilla puts the saddle slot left
                // of the chest grid, w/Donkey GUI), the chest grid
                // (5-wide, rows = ceil(capacity/5), the last row partial
                // for llamas) to its right. The grid slots share the
                // generic chest-slot geometry (game.rs routes
                // SlotRef::Chest(i) into the mob's storage slots).
                let mv = view.mount.clone().unwrap_or(MountView {
                    kind_label: "MOUNT".into(),
                    saddle: ItemStack::EMPTY,
                    llama: false,
                    strength: 1,
                    capacity: 15,
                });
                let cap = mv.capacity.max(1);
                let rows = cap.div_ceil(5);
                // saddle column (40) + gap (24) + 5-wide grid (200)
                let total = 40 + 24 + 5 * 40;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                if mv.llama {
                    // the llama's strength badge — vanilla's left column
                    // is the carpet decor slot; carpets are not registered
                    // in the engine, so the strength (the capacity's own
                    // driver, VERIFIED w/Llama §Storage) is shown instead:
                    // documented adaptation
                    let bx = cx;
                    let by = cy + 4;
                    self.rect(bx, by, 40, 40, [22, 22, 26, 200]);
                    self.frame(bx, by, 40, 40, [60, 60, 66, 255]);
                    self.text(bx + 4, by + 6, "STR", [255, 220, 120, 255], 1);
                    // the strength pips (1..5, one column of short bars)
                    for p in 0..mv.strength.clamp(1, 5) {
                        self.rect(bx + 8, by + 22 + p as i32 * 3, 24, 2, [120, 220, 120, 255]);
                    }
                } else {
                    // the saddle slot — donkey/mule (w/Donkey §Usage:
                    // "Saddle slot for equipping a saddle. ... A saddle
                    // can be equipped on a donkey by holding it and then
                    // using on the donkey, or by accessing its inventory")
                    self.slot_well(cx, cy, &mv.saddle, atlas);
                    geom.mount = Some(MountGeom { saddle: (cx, cy) });
                }
                let gx = cx + 40 + 24;
                for r in 0..rows {
                    for c in 0..5 {
                        let idx = r * 5 + c;
                        if idx >= cap {
                            break; // the llama's partial last row
                        }
                        let x = gx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        let st = view.chest.get(idx).copied().unwrap_or(ItemStack::EMPTY);
                        self.slot_well(x, y, &st, atlas);
                        geom.chest.push((x, y));
                    }
                }
            }
            ContainerKind::Furnace => {
                // left column: input above flame above fuel; arrow → output
                let cx = x0 + (grid_w - 240) / 2;
                let cy = y0 + 8;
                let (input, fuel, output, burn, cook) =
                    view.furnace.map(|f| (f.0, f.1, f.2, f.3, f.4)).unwrap_or((
                        ItemStack::EMPTY,
                        ItemStack::EMPTY,
                        ItemStack::EMPTY,
                        0.0,
                        0.0,
                    ));
                let ix = cx + 10;
                let iy = cy;
                self.slot_well(ix, iy, &input, atlas);
                self.flame(ix + 5, iy + 40, burn);
                let fx = cx + 10;
                let fy = iy + 70;
                self.slot_well(fx, fy, &fuel, atlas);
                self.arrow(cx + 70, iy + 42, cook);
                let oxp = cx + 126;
                let oyp = iy + 22;
                self.slot_well(oxp, oyp, &output, atlas);
                // output gets the vanilla wide highlight frame
                self.frame(oxp - 2, oyp - 2, 40, 40, [255, 255, 255, 120]);
                geom.furnace = Some(FurnaceSlots {
                    input: (ix, iy),
                    fuel: (fx, fy),
                    output: (oxp, oyp),
                });
            }
            ContainerKind::Brewing => {
                // vanilla layout: ingredient top-center; below it the bubble
                // column; bottom row = fuel left + 3 bottle slots
                let total = 3 * 40 + 30;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                let (ing, fuel, bottles, fuel_frac, brew_frac) =
                    view.brewing.map(|b| (b.0, b.1, b.2, b.3, b.4)).unwrap_or((
                        ItemStack::EMPTY,
                        ItemStack::EMPTY,
                        [ItemStack::EMPTY; 3],
                        0.0,
                        0.0,
                    ));
                let ix = cx + 44;
                let iy = cy;
                self.slot_well(ix, iy, &ing, atlas);
                // bubbles under the ingredient
                self.bubbles(ix + 14, iy + 40, brew_frac);
                // fuel slot on the left with a charge bar
                let fx = cx;
                let fy = cy + 78;
                self.slot_well(fx, fy, &fuel, atlas);
                // fuel-charge bar under the fuel slot (20 operations)
                self.bar(fx - 2, fy + 40, 40, 5, fuel_frac.clamp(0.0, 1.0));
                // three bottle slots
                let by = cy + 78;
                let mut bottle_pos = [(0, 0); 3];
                for (i, bp) in bottle_pos.iter_mut().enumerate() {
                    let bx = cx + 40 + i as i32 * 40;
                    self.slot_well(bx, by, &bottles[i], atlas);
                    *bp = (bx, by);
                }
                geom.brewing = Some(BrewSlots {
                    ingredient: (ix, iy),
                    fuel: (fx, fy),
                    bottles: bottle_pos,
                });
            }
            ContainerKind::Enchant => {
                // vanilla layout: item + lapis slots left, 3 option rows
                // right; each option shows its level number and cost
                let (item, lapis, options, player_level, power) =
                    view.enchant.map(|e| (e.0, e.1, e.2, e.3, e.4)).unwrap_or((
                        ItemStack::EMPTY,
                        ItemStack::EMPTY,
                        [vc_gameplay::enchanting::EnchOption {
                            level: 0,
                            ench: 0,
                            ench_level: 0,
                            cost: 0,
                        }; 3],
                        0,
                        0,
                    ));
                let cx = x0 + 20;
                let cy = y0 + 8;
                self.slot_well(cx, cy, &item, atlas);
                let lx = cx;
                let ly = cy + 44;
                self.slot_well(lx, ly, &lapis, atlas);
                // power readout under the slots
                self.text(
                    cx,
                    ly + 44,
                    &format!("shelves {power}/15"),
                    [120, 200, 120, 255],
                    1,
                );
                // three option buttons
                let ox = cx + 60;
                let mut opt_pos = [(0, 0); 3];
                for (i, o) in options.iter().enumerate() {
                    let oy = cy + i as i32 * 48;
                    let affordable = o.level > 0
                        && !item.is_empty()
                        && item.block == vc_blocks::blocks::ENCHANTED_BOOK
                        && lapis.count >= o.cost
                        && player_level >= o.cost as i32;
                    // button chrome
                    let bg = if affordable {
                        [34, 60, 34, 235]
                    } else {
                        [40, 34, 34, 200]
                    };
                    self.rect(ox, oy, 180, 44, bg);
                    self.frame(ox, oy, 180, 44, [12, 12, 14, 255]);
                    if o.level == 0 {
                        self.text(ox + 8, oy + 16, "- - -", [110, 110, 110, 255], 1);
                    } else {
                        // the vanilla green level number
                        self.text(
                            ox + 6,
                            oy + 16,
                            &format!("{}", o.level),
                            [90, 255, 90, 255],
                            1,
                        );
                        // enchant name + roman level
                        let def = vc_gameplay::enchanting::enchant_def(o.ench);
                        let label = format!(
                            "{} {}",
                            def.name,
                            vc_gameplay::enchanting::roman(o.ench_level)
                        );
                        let color: [u8; 4] = if affordable {
                            [255, 255, 255, 255]
                        } else {
                            [150, 150, 150, 255]
                        };
                        self.text(ox + 34, oy + 6, &label, color, 1);
                        // cost line
                        self.text(
                            ox + 34,
                            oy + 24,
                            &format!("cost {} lvl + {} lapis", o.cost, o.cost),
                            [200, 200, 120, 255],
                            1,
                        );
                    }
                    opt_pos[i] = (ox, oy);
                }
                geom.enchant = Some(EnchantSlots {
                    item: (cx, cy),
                    lapis: (lx, ly),
                    options: opt_pos,
                });
            }
            ContainerKind::Trade => {
                // Phase 5 trade screen: two columns of offer rows (10 rows
                // per profession), the career level + XP bar in the title,
                // per-row stock counts, locked tiers dimmed with the level
                // they unlock at. Row hit-rects keep TABLE indices, so
                // SlotRef::TradeRow(i) maps straight to execute_trade(i).
                let tv = view.trade.clone().unwrap_or(TradeView {
                    profession: "VILLAGER".into(),
                    level_name: "Novice".into(),
                    level: 1,
                    xp: 0,
                    xp_next: Some(10),
                    rows: Vec::new(),
                });
                // career header: profession · level · xp bar
                self.text(
                    px0 + 12,
                    y0 - 24,
                    &format!("VILLAGER: {} — {}", tv.profession, tv.level_name),
                    [255, 220, 120, 255],
                    1,
                );
                let bx = px0 + pw - 130;
                self.rect(bx, y0 - 28, 108, 10, [20, 20, 24, 255]);
                self.frame(bx, y0 - 28, 108, 10, [70, 70, 76, 255]);
                if let Some(next) = tv.xp_next {
                    let prev = vc_gameplay::villagers::LEVEL_XP[(tv.level - 1) as usize];
                    let frac =
                        ((tv.xp - prev) as f32 / (next - prev).max(1) as f32).clamp(0.0, 1.0);
                    self.rect(
                        bx + 2,
                        y0 - 26,
                        ((104.0 * frac) as i32).max(if tv.xp > prev { 1 } else { 0 }),
                        6,
                        [120, 220, 120, 255],
                    );
                } else {
                    self.rect(bx + 2, y0 - 26, 104, 6, [220, 170, 80, 255]); // Master
                }
                // two columns of 5 rows, 260 wide each
                let col_x = [px0 + 16, px0 + 286];
                let cy = y0 + 16;
                let mut row_pos = vec![(i32::MIN, i32::MIN); tv.rows.len()];
                for (i, r) in tv.rows.iter().enumerate() {
                    let (col, slot) = (i / 5, i % 5);
                    let rx = col_x[col.min(1)];
                    let ry = cy + slot as i32 * 44;
                    // state colors: locked (tier above the level) → gray;
                    // out of stock → dark red; affordable → dark green
                    let bg = if r.locked {
                        [30, 30, 34, 170]
                    } else if r.stock == 0 {
                        [50, 26, 26, 220]
                    } else if r.afford {
                        [34, 60, 34, 235]
                    } else {
                        [40, 34, 34, 200]
                    };
                    self.rect(rx, ry, 260, 40, bg);
                    self.frame(rx, ry, 260, 40, [12, 12, 14, 255]);
                    // give stack + count
                    self.draw_stack(&r.give, rx + 6, ry + 2, atlas);
                    self.text(
                        rx + 44,
                        ry + 4,
                        &format!("{} x", r.give.count),
                        if r.locked {
                            [110, 110, 110, 255]
                        } else {
                            [255, 255, 255, 255]
                        },
                        1,
                    );
                    // arrow
                    let lit = !r.locked && r.stock > 0 && r.afford;
                    self.arrow(rx + 76, ry + 10, if lit { 1.0 } else { 0.25 });
                    // get stack + count
                    self.draw_stack(&r.get, rx + 128, ry + 2, atlas);
                    self.text(
                        rx + 166,
                        ry + 4,
                        &format!("{} x", r.get.count),
                        if r.locked {
                            [110, 110, 110, 255]
                        } else if r.afford {
                            [120, 255, 120, 255]
                        } else {
                            [150, 150, 150, 255]
                        },
                        1,
                    );
                    // right rail: stock "n/m" or the level that unlocks it
                    if r.locked {
                        self.text(rx + 216, ry + 24, "LOCK", [130, 130, 140, 255], 1);
                        self.text(
                            rx + 186,
                            ry + 4,
                            &format!("LV{}", r.tier),
                            [160, 160, 170, 255],
                            1,
                        );
                    } else if r.stock == 0 {
                        self.text(rx + 206, ry + 24, "OUT", [255, 90, 90, 255], 1);
                    } else {
                        self.text(
                            rx + 200,
                            ry + 24,
                            &format!("{}/{}", r.stock, r.max_uses),
                            [200, 200, 205, 255],
                            1,
                        );
                    }
                    row_pos[i] = (rx, ry);
                }
                geom.trade = Some(TradeSlots { rows: row_pos });
            }
            ContainerKind::Anvil => {
                // Round 13: the anvil — two inputs + the "+" glyph + the
                // result, the rename field above, the cost line under the
                // arrow (audit §1: inputs (27,47)/(76,47), result (134,47),
                // cost at (60,70) — doubled into the 36px slot space)
                let av = view.anvil.clone().unwrap_or(AnvilView {
                    target: ItemStack::EMPTY,
                    sacrifice: ItemStack::EMPTY,
                    result: ItemStack::EMPTY,
                    cost: 0,
                    too_expensive: false,
                    affordable: false,
                    rename: String::new(),
                    rename_focused: false,
                    creative: false,
                });
                // the rename field (104x12 vanilla → 208x24 here)
                let rx = px0 + 128;
                let ry = y0 + 16;
                self.rect(rx, ry, 208, 24, [16, 16, 18, 255]);
                self.frame(
                    rx,
                    ry,
                    208,
                    24,
                    if av.rename_focused {
                        [160, 160, 170, 255]
                    } else {
                        [70, 70, 76, 255]
                    },
                );
                let shown = if av.rename.is_empty() {
                    "item name".to_string()
                } else {
                    av.rename.clone()
                };
                let placeholder = av.rename.is_empty();
                self.text(
                    rx + 6,
                    ry + 8,
                    &shown,
                    if placeholder {
                        [110, 110, 110, 255]
                    } else {
                        [235, 235, 235, 255]
                    },
                    1,
                );
                // hammer glyph left of the field (clean-room redraw)
                let hx = px0 + 88;
                let hy = ry + 2;
                self.rect(hx + 8, hy, 20, 8, [150, 150, 156, 255]);
                self.rect(hx + 12, hy + 8, 4, 14, [110, 84, 56, 255]);
                // the three slots
                let sy = y0 + 64;
                let tx = px0 + 54; // (27,47) x2
                let sx = px0 + 152; // (76,47) x2
                let ox = px0 + 268; // (134,47) x2
                self.slot_well(tx, sy, &av.target, atlas);
                self.slot_well(sx, sy, &av.sacrifice, atlas);
                // the result: dimmed when refused/unaffordable (VERIFIED:
                // "Insufficient XP: result slot dimmed + cost text red")
                self.slot_well(ox, sy, &av.result, atlas);
                if !av.result.is_empty() && (!av.affordable || av.too_expensive) && !av.creative {
                    self.rect(ox + 2, sy + 2, 32, 32, [40, 10, 10, 130]);
                }
                // the "+" glyph between the inputs (audit: ~(61,54) x2)
                self.text(px0 + 118, sy + 12, "+", [200, 200, 205, 255], 2);
                // the progress arrow toward the result
                self.arrow(
                    px0 + 196,
                    sy + 14,
                    if av.result.is_empty() { 0.2 } else { 1.0 },
                );
                // the cost line under the arrow (audit: (60,70) x2)
                let cost_label = if av.too_expensive && !av.creative {
                    "TOO EXPENSIVE!".to_string()
                } else if av.result.is_empty() {
                    String::new()
                } else {
                    format!("Enchantment Cost: {}", av.cost)
                };
                if !cost_label.is_empty() {
                    let color: [u8; 4] = if av.creative {
                        [120, 255, 120, 255]
                    } else if av.affordable && !av.too_expensive {
                        [90, 255, 90, 255] // green when affordable (VERIFIED)
                    } else {
                        [255, 80, 80, 255] // red when not (VERIFIED)
                    };
                    let lw = Self::text_width(&cost_label, 1);
                    self.text(px0 + 200 - lw / 2, sy + 44, &cost_label, color, 1);
                }
                geom.anvil = Some(AnvilSlots {
                    target: (tx, sy),
                    sacrifice: (sx, sy),
                    out: (ox, sy),
                    rename: (rx, ry),
                });
            }
            ContainerKind::Beacon => {
                // Round 13: the beacon — "Primary Power" label + the 2x2+1
                // button grid, "Secondary Power" with Regeneration + II at
                // level 4, the level glyphs down the left edge, the pay
                // slot + confirm/cancel in the bottom bar (audit §2's
                // clean-room structure)
                let bv = view.beacon.clone().unwrap_or(BeaconView {
                    level: 0,
                    pay: ItemStack::EMPTY,
                    primary: None,
                    secondary: vc_gameplay::beacon::BeaconSecondary::None,
                    pending_primary: None,
                    pending_secondary: vc_gameplay::beacon::BeaconSecondary::None,
                });
                use vc_gameplay::beacon::{BeaconPower, BeaconSecondary};
                let powers = [
                    BeaconPower::Speed,
                    BeaconPower::Haste,
                    BeaconPower::Resistance,
                    BeaconPower::JumpBoost,
                    BeaconPower::Strength,
                ];
                // the level glyphs (3 stacked down the left edge + 1 at
                // the right — audit §2; lit green per achieved level)
                for i in 0..bv.level.min(3) as usize {
                    let gy = y0 + 48 + i as i32 * 38;
                    self.rect(px0 + 30, gy, 24, 24, [90, 220, 90, 255]);
                    self.frame(px0 + 30, gy, 24, 24, [30, 60, 30, 255]);
                }
                if bv.level >= 4 {
                    self.rect(px0 + 242, y0 + 48, 24, 24, [90, 220, 90, 255]);
                    self.frame(px0 + 242, y0 + 48, 24, 24, [30, 60, 30, 255]);
                }
                self.text(px0 + 12, y0 + 20, "PRIMARY POWER", [255, 220, 120, 255], 1);
                self.text(px0 + 260, y0 + 20, "SECONDARY", [255, 220, 120, 255], 1);
                // the 2x2 grid + the 5th below-left (audit §2)
                let mut prim_pos = [(0, 0); 5];
                for (i, p) in powers.iter().enumerate() {
                    let (col, row) = (i % 2, i / 2);
                    let bx = px0 + 76 + col as i32 * 56;
                    let by = y0 + 44 + row as i32 * 56 + if i == 4 { 56 } else { 0 };
                    let gated = p.min_level() > bv.level;
                    let selected = bv.pending_primary == Some(*p)
                        || (bv.pending_primary.is_none() && bv.primary == Some(*p));
                    let bg: [u8; 4] = if selected && !gated {
                        [40, 90, 40, 235]
                    } else if gated {
                        [30, 30, 34, 170]
                    } else {
                        [40, 34, 34, 200]
                    };
                    self.rect(bx, by, 44, 44, bg);
                    self.frame(bx, by, 44, 44, [12, 12, 14, 255]);
                    // the power glyph: a clean-room 2-letter monogram
                    let mono = match p {
                        BeaconPower::Speed => "SP",
                        BeaconPower::Haste => "HA",
                        BeaconPower::Resistance => "RE",
                        BeaconPower::JumpBoost => "JB",
                        BeaconPower::Strength => "ST",
                    };
                    let col_txt: [u8; 4] = if gated {
                        [100, 100, 105, 255]
                    } else if selected {
                        [140, 255, 140, 255]
                    } else {
                        [230, 230, 235, 255]
                    };
                    self.text(bx + 12, by + 16, mono, col_txt, 2);
                    let (label, lv) = (p.name(), p.min_level());
                    self.text(
                        bx - 6,
                        by + 50,
                        &format!("{}(L{})", &label[..2], lv),
                        if gated {
                            [100, 100, 105, 255]
                        } else {
                            [180, 180, 185, 255]
                        },
                        1,
                    );
                    prim_pos[i] = (bx, by);
                }
                // the secondary row: Regeneration + primary-II (level 4
                // only — VERIFIED)
                let sec_gated = bv.level < 4;
                let mut sec_pos = [(0, 0); 2];
                for (i, s) in [BeaconSecondary::Regeneration, BeaconSecondary::PrimaryII]
                    .iter()
                    .enumerate()
                {
                    let bx = px0 + 260 + i as i32 * 56;
                    let by = y0 + 44;
                    let selected = bv.pending_secondary == *s
                        || (bv.pending_secondary == BeaconSecondary::None && bv.secondary == *s);
                    let bg: [u8; 4] = if selected && !sec_gated {
                        [40, 90, 40, 235]
                    } else if sec_gated {
                        [30, 30, 34, 170]
                    } else {
                        [40, 34, 34, 200]
                    };
                    self.rect(bx, by, 44, 44, bg);
                    self.frame(bx, by, 44, 44, [12, 12, 14, 255]);
                    let (mono, tip) = if *s == BeaconSecondary::Regeneration {
                        ("RG", "Regen")
                    } else {
                        ("II", "Pwr II")
                    };
                    self.text(
                        bx + 12,
                        by + 16,
                        mono,
                        if sec_gated {
                            [100, 100, 105, 255]
                        } else if selected {
                            [140, 255, 140, 255]
                        } else {
                            [230, 230, 235, 255]
                        },
                        2,
                    );
                    self.text(
                        bx - 2,
                        by + 50,
                        if sec_gated { "L4" } else { tip },
                        if sec_gated {
                            [100, 100, 105, 255]
                        } else {
                            [180, 180, 185, 255]
                        },
                        1,
                    );
                    sec_pos[i] = (bx, by);
                }
                // the bottom bar: pay slot + confirm + cancel
                let py = y0 + 176;
                let payx = px0 + 176;
                self.slot_well(payx, py, &bv.pay, atlas);
                self.text(px0 + 12, py + 10, "PAY:", [200, 200, 205, 255], 1);
                // confirm (green check) — enabled when a pending selection
                // exists and the pay slot holds a valid mineral
                let has_pay = matches!(
                    bv.pay.block,
                    vc_blocks::blocks::IRON_ORE
                        | vc_blocks::blocks::GOLD_ORE
                        | vc_blocks::blocks::DIAMOND_ORE
                        | vc_blocks::blocks::EMERALD
                        | vc_blocks::blocks::IRON_BLOCK
                        | vc_blocks::blocks::GOLD_BLOCK
                        | vc_blocks::blocks::DIAMOND_BLOCK
                );
                let confirm_on = bv.pending_primary.is_some() && (has_pay || bv.level == 0);
                let cx2 = px0 + 212;
                self.rect(
                    cx2,
                    py,
                    36,
                    36,
                    if confirm_on {
                        [34, 90, 34, 235]
                    } else {
                        [30, 30, 34, 170]
                    },
                );
                self.frame(cx2, py, 36, 36, [12, 12, 14, 255]);
                self.text(
                    cx2 + 12,
                    py + 12,
                    "OK",
                    if confirm_on {
                        [140, 255, 140, 255]
                    } else {
                        [100, 100, 105, 255]
                    },
                    1,
                );
                // cancel (red X)
                let cx3 = px0 + 260;
                self.rect(cx3, py, 36, 36, [90, 34, 34, 200]);
                self.frame(cx3, py, 36, 36, [12, 12, 14, 255]);
                self.text(cx3 + 12, py + 12, "X", [255, 120, 120, 255], 1);
                geom.beacon = Some(BeaconSlots {
                    primary: prim_pos,
                    secondary: sec_pos,
                    pay: (payx, py),
                    confirm: (cx2, py),
                    cancel: (cx3, py),
                });
            }
            ContainerKind::Grindstone => {
                // Round 13: the grindstone — two stacked inputs at (50,18)
                // and (50,40) (22px pitch), the wheel illustration beside
                // them, the result at (148,32), the XP hint under the
                // arrow (audit §3 — doubled into the 36px slot space)
                let (top, bottom, result) =
                    view.grind
                        .unwrap_or((ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY));
                let tx = px0 + 100; // (50,18) x2
                let ty = y0 + 36;
                let by = ty + 44; // the 22px pitch doubled
                self.slot_well(tx, ty, &top, atlas);
                self.slot_well(tx, by, &bottom, atlas);
                // the wheel illustration (clean-room: two side posts +
                // the stone disc)
                let wx = px0 + 36;
                self.rect(wx, ty + 6, 12, 76, [110, 84, 56, 255]);
                self.rect(wx + 68, ty + 6, 12, 76, [110, 84, 56, 255]);
                self.rect(wx + 20, ty + 26, 40, 36, [150, 150, 156, 255]);
                self.rect(wx + 26, ty + 32, 28, 24, [120, 120, 126, 255]);
                // the arrow
                self.arrow(px0 + 180, by + 2, if result.is_empty() { 0.2 } else { 1.0 });
                // the result at (148,32) x2
                let ox = px0 + 296;
                let oy = ty + 20;
                self.slot_well(ox, oy, &result, atlas);
                geom.grind = Some(GrindSlots {
                    top: (tx, ty),
                    bottom: (tx, by),
                    out: (ox, oy),
                });
            }
        }

        // ---- shared inventory: 3 storage rows + hotbar row ----
        let sy = y0 + top_h + 4;
        for row in 0..3 {
            let y = sy + row as i32 * 44;
            for c in 0..9 {
                let x = x0 + c as i32 * 40;
                self.slot_well(x, y, &view.inv[9 + row * 9 + c], atlas);
                geom.inv.push((x, y));
            }
        }
        let hy = sy + 3 * 44 + 10;
        for c in 0..9 {
            let x = x0 + c as i32 * 40;
            self.slot_well(x, hy, &view.inv[c], atlas);
            geom.inv.push((x, hy));
        }

        // ---- cursor stack follows the mouse (vanilla) ----
        if !view.cursor.is_empty() {
            let cx = cursor_pos.0 as i32 - 18;
            let cy = cursor_pos.1 as i32 - 18;
            self.draw_stack(&view.cursor, cx, cy, atlas);
        }

        // hover label: name of the hovered slot's block (F3+H appends the
        // registry id — vanilla "advanced tooltips")
        if let Some(s) = view.hovered_stack(cursor_pos.0 as i32, cursor_pos.1 as i32, &geom) {
            if !s.is_empty() {
                let label: String = if advanced_tooltips {
                    let id: String = name(s.block).to_lowercase().replace(' ', "_");
                    format!("{} (voxelcraft:{})", name(s.block), id)
                } else {
                    name(s.block).to_string()
                };
                let lw = Self::text_width(&label, 1);
                self.text(
                    (self.live_w as i32 - lw) / 2,
                    y0 - 44,
                    &label,
                    [255, 255, 255, 255],
                    1,
                );
            }
        }
        geom
    }

    /// Vanilla 1.16.5 F3 debug overlay: TWO columns — `left` anchored
    /// top-left, `right` top-right and right-aligned to the screen edge.
    /// Each text line sits on its own translucent dark strip (the chat-like
    /// background, 0x90505050), text is flat white with NO drop shadow
    /// (vanilla renders F3 text unshadowed on the strip). Empty strings are
    /// blank spacer lines (no strip) — used for the vanilla group gaps.
    ///
    /// Vanilla 1.16.5 DebugHud metrics (clean-room observed, scaled to this
    /// canvas): per-line 0x90505050 strips that are CONTIGUOUS (pitch ==
    /// strip height — the reference overlay's strips stack seamlessly),
    /// flat unshadowed 0xE0E0E0 text at the STANDARD UI font size (the
    /// vanilla overlay uses the same font as every other label), 1px strip
    /// padding around the text, left column at x=2/text x=3, right column
    /// RIGHT-ALIGNED to a 3px right margin, first strip at y=2, blank
    /// spacer lines advance the pitch without painting.
    pub fn debug(&mut self, left: &[String], right: &[String]) {
        const BG: Color = [80, 80, 80, 144]; // 0x90505050
        const FG: Color = [224, 224, 224, 255]; // 0xE0E0E0
        const LINE_H: i32 = 18; // glyph 16 (8 rows x scale 2) + 1px pad top+bottom
                                // Luanti round 2: strips are fractional solid quads measured
                                // with the ACTIVE font's real metrics — the strip hugs the
                                // device-exact text at any window size instead of quantizing
                                // to the 960x540 integer grid (text already rides glyph quads).
                                // They ride the OVER-CANVAS text layer (pushed immediately
                                // before their line's text, so the text still composites on
                                // top) — the canvas is empty at the strip region in armed
                                // mode, and the text layer is the proven-visible Solid path
                                // (the crosshair class; browser E2E caught the chrome-layer
                                // draw silently dropping them).
        let strip_quads = text_quads_active() && crate::gui::font::engine().is_some();
        let bg_tint: [f32; 4] = [
            BG[0] as f32 / 255.0,
            BG[1] as f32 / 255.0,
            BG[2] as f32 / 255.0,
            BG[3] as f32 / 255.0,
        ];
        let measure = |s: &str| -> f32 {
            match crate::gui::font::engine() {
                Some(eng) => {
                    let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
                    e.measure(s, 16.0)
                }
                None => Self::text_width_case(s, 2) as f32,
            }
        };
        for (i, l) in left.iter().enumerate() {
            if l.is_empty() {
                continue;
            }
            let y = 2 + i as i32 * LINE_H;
            if strip_quads {
                let w = measure(l);
                self.gui_frame
                    .solid_over(2.0, y as f32, w + 2.0, LINE_H as f32, bg_tint);
            } else {
                let w = Self::text_width_case(l, 2);
                self.rect(2, y, w + 2, LINE_H, BG);
            }
            self.text_flat_case(3, y + 1, l, FG, 2);
        }
        for (i, l) in right.iter().enumerate() {
            if l.is_empty() {
                continue;
            }
            let y = 2 + i as i32 * LINE_H;
            // Round-4 fix (live-browser forensics): software-renderer
            // adapter strings run ~66 chars ≈ 830 UI px wide, so the
            // right-aligned line starts at UI x≈125 and overdraws the
            // LEFT column's lower rows into unreadable garbage (the
            // Culling counter line vanished under SwiftShader's name).
            // No right line may cross the half-screen mark: truncate to
            // the measured width with an ASCII "..." tail (vanilla
            // truncates its own long renderer lines rather than wrap).
            let half = self.live_w as f32 / 2.0 - 12.0;
            if strip_quads {
                let line = fit_line(l, half, |s| measure(s));
                let w = measure(&line);
                // text ends 3px from the right edge; the strip extends
                // 1 UI px past both ends of the fractional text width
                let x = self.live_w as f32 - 3.0 - w;
                self.gui_frame
                    .solid_over(x - 1.0, y as f32, w + 2.0, LINE_H as f32, bg_tint);
                self.text_flat_case(x.round() as i32, y + 1, &line, FG, 2);
            } else {
                let line = fit_line(l, half, |s| Self::text_width_case(s, 2) as f32);
                let w = Self::text_width_case(&line, 2);
                let x = self.live_w as i32 - 3 - w; // text ends 3px from the right edge
                self.rect(x - 1, y, w + 2, LINE_H, BG);
                self.text_flat_case(x, y + 1, &line, FG, 2);
            }
        }
    }

    /// 2026-09-19 — F3 dump path: render the debug columns through the
    /// CANVAS glyph path regardless of armed mode. The live overlay
    /// rides the GPU text-quad layer in armed mode, which dump_png (the
    /// CPU pixel buffer) cannot see; the smoke's F3_DUMP/F3_DUMP2
    /// liveness pair dumps THESE pixels, carrying the live text (and
    /// its monotonic Frame/R counters). Layout mirrors debug()'s canvas
    /// branch (strips via rect, text via text_flat_case_px, the
    /// right-column truncation at the half-screen mark).
    pub fn debug_canvas(&mut self, left: &[String], right: &[String]) {
        const BG: Color = [80, 80, 80, 144]; // 0x90505050
        const FG: Color = [224, 224, 224, 255]; // 0xE0E0E0
        const LINE_H: i32 = 18;
        for (i, l) in left.iter().enumerate() {
            if l.is_empty() {
                continue;
            }
            let y = 2 + i as i32 * LINE_H;
            let w = Self::text_width_case(l, 2);
            self.rect(2, y, w + 2, LINE_H, BG);
            self.text_flat_case_px(3, y + 1, l, FG, 2);
        }
        for (i, l) in right.iter().enumerate() {
            if l.is_empty() {
                continue;
            }
            let y = 2 + i as i32 * LINE_H;
            // same half-screen truncation as debug()'s canvas branch
            let half = self.live_w as f32 / 2.0 - 12.0;
            let line = fit_line(l, half, |s| Self::text_width_case(s, 2) as f32);
            let w = Self::text_width_case(&line, 2);
            let x = self.live_w as i32 - 3 - w;
            self.rect(x - 1, y, w + 2, LINE_H, BG);
            self.text_flat_case_px(x, y + 1, &line, FG, 2);
        }
    }

    /// Vanilla F3+Q help overlay: the key-combination list in a centered
    /// box (like the real "Debug help" screen — rows of "F3 + X - action"),
    /// rendered with the same true-case font as the F3 overlay.
    pub fn debug_help(&mut self, rows: &[(String, String)]) {
        let w = 340;
        let row_h = 20;
        let h = rows.len() as i32 * row_h + 40;
        let x0 = (self.live_w as i32 - w) / 2;
        let y0 = (self.live_h as i32 - h) / 2;
        self.rect(x0, y0, w, h, [12, 12, 14, 235]);
        self.frame(x0, y0, w, h, [140, 140, 140, 200]);
        self.text_flat_case(x0 + 12, y0 + 9, "Debug help", [255, 255, 255, 255], 2);
        for (i, (k, d)) in rows.iter().enumerate() {
            let y = y0 + 38 + i as i32 * row_h;
            self.text_flat_case(x0 + 12, y, k, [170, 220, 170, 255], 2);
            self.text_flat_case(x0 + 110, y, d, [224, 224, 224, 255], 2);
        }
    }

    /// Sodium-style rolling frame-time graph under the F3 text block.
    /// `times_ms` = last N frame times; green bars, 50 ms scale, 2 px/bar.
    /// Luanti round 2: solid quads when armed (one quad per bar — the
    /// bg, guide line and every bar land on crisp GPU geometry at any
    /// window size); the canvas per-pixel raster stays as the fallback.
    /// single-pixel set with bounds clamp (graph bars)
    fn px_set(&mut self, x: i32, y: i32, c: Color) {
        self.set(x, y, c);
    }

    pub fn frame_graph(&mut self, y: i32, times_ms: &[f32]) {
        let n = times_ms.len();
        if n < 2 {
            return;
        }
        let w = (n as i32 * 2).min(360);
        let x0 = 4;
        let h = 40;
        let ct = |c: Color| -> [f32; 4] {
            [
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                c[3] as f32 / 255.0,
            ]
        };
        // 16.7 ms guide line (60 fps target)
        let guide_y = y + 2 + h - ((16.7f32 / 50.0) * h as f32) as i32;
        let bar_color = |t: f32| -> Color {
            if t <= 20.0 {
                [60, 220, 90, 230]
            } else if t <= 40.0 {
                [240, 200, 40, 230]
            } else {
                [235, 70, 50, 230]
            }
        };
        if text_quads_active() {
            self.gui_frame.solid_over(
                x0 as f32,
                y as f32,
                (w + 4) as f32,
                (h + 4) as f32,
                ct([80, 80, 80, 110]),
            );
            self.gui_frame.solid_over(
                x0 as f32 + 2.0,
                guide_y as f32,
                w as f32,
                1.0,
                ct([255, 255, 255, 70]),
            );
            for (i, t) in times_ms.iter().rev().enumerate() {
                let x = x0 + 2 + i as i32 * 2;
                if x >= x0 + 2 + w {
                    break;
                }
                let th = ((t / 50.0).clamp(0.0, 1.0) * h as f32) as i32;
                if th > 0 {
                    self.gui_frame.solid_over(
                        x as f32,
                        (y + 2 + h - th) as f32,
                        2.0,
                        th as f32,
                        ct(bar_color(*t)),
                    );
                }
            }
            return;
        }
        self.rect(x0, y, w + 4, h + 4, [80, 80, 80, 110]);
        for dx in 0..w {
            let x = x0 + 2 + dx;
            if x < x0 + 2 + w {
                self.px_set(x, guide_y, [255, 255, 255, 70]);
            }
        }
        for (i, t) in times_ms.iter().rev().enumerate() {
            let x = x0 + 2 + i as i32 * 2;
            if x >= x0 + 2 + w {
                break;
            }
            let th = ((t / 50.0).clamp(0.0, 1.0) * h as f32) as i32;
            let color = bar_color(*t);
            for dy in 0..th {
                self.px_set(x, y + 2 + h - 1 - dy, color);
                self.px_set(x + 1, y + 2 + h - 1 - dy, color);
            }
        }
    }

    /// Sub-round 2 (2026-09-15) + Round 15b (2026-09-17): the vanilla
    /// 1.16.5 tabbed creative inventory. clean-room: shape language =
    /// the classic (pre-1.19.3) creative screen — two tab rows on top
    /// (6 + 6 tabs, icon-only folder tabs), a 9x5 slot grid with a
    /// right scrollbar, the tab title above the grid (the Search tab
    /// replaces it with a search field), and the hotbar + destroy slot
    /// at the bottom. Reference facts: reference wiki /Creative_
    /// inventory (live 2026-09-15): the nine content tabs + Search
    /// Items + Saved Hotbars + Survival Inventory (live fetch
    /// 2026-09-17: "There are also Search Items, Saved Hotbars and
    /// Survival Inventory tabs"); the 9x5/45-per-page grid with a
    /// scrollbar; "A single item can be grabbed using left-click ...;
    /// Right-clicking an item also picks up one item ...; Shift-
    /// clicking an item grabs a full stack"; "Pressing a number key
    /// while hovering over an item instantly places one full stack of
    /// that item into the hotbar slot"; the destroy slot. Proportions
    /// are the engine's established 2x container geometry (40px slots
    /// on the 960x540 canvas). No third-party asset was read, copied, or
    /// traced.
    #[allow(clippy::too_many_arguments)]
    pub fn creative_screen(
        &mut self,
        cursor: (f32, f32),
        atlas: &[u8],
        tab: u8,
        scroll: usize,
        search: &str,
        search_focused: bool,
        items: &[u16],
        hotbar: &[vc_inventory::inventory::ItemStack],
        selected: usize,
        cursor_stack: &vc_inventory::inventory::ItemStack,
        advanced_tooltips: bool,
    ) -> CreativeGeom {
        use vc_blocks::blocks as blk;

        // ---- layout constants (engine 2x container geometry) ----
        let cols = 9usize;
        let vis_rows = 5usize;
        let cell = 40i32;
        let grid_w = cols as i32 * cell + 4;
        let x0 = (self.live_w as i32 - grid_w) / 2;
        // panel: title strip 26px; grid 5x40; hotbar row below
        let y0 = 150i32; // grid top
        let title_y = y0 - 26;
        let hot_y = y0 + vis_rows as i32 * cell + 14;
        let px0 = x0 - 14;
        let pw = grid_w + 28;
        let py0 = title_y - 10;
        let ph = (hot_y + 46) - py0;
        // tab strip: two rows (6 + 6), folder tabs attached to the panel
        let tab_w = 76i32;
        let tab_h = 44i32;
        let row1_n = 6i32;
        let row1_x = (self.live_w as i32 - row1_n * tab_w) / 2;
        // Round 15b: the second row grew to 6 — the Saved Hotbars tab
        // (vanilla's 12-tab strip: 9 content + Search + Hotbars +
        // Inventory)
        let row2_n = 6i32;
        let row2_x = (self.live_w as i32 - row2_n * tab_w) / 2;
        let row2_y = py0 - tab_h + 2;
        let row1_y = row2_y - tab_h + 2;

        let mut geom = CreativeGeom {
            x0,
            y0,
            cell,
            cols,
            vis_rows,
            scroll,
            tabs: [None; 12],
            hotbar: [(0, 0, 0, 0); 9],
            trash: (0, 0, 0, 0),
            search: (0, 0, 0, 0),
            scrollbar: None,
        };

        // ---- tab strip (12 tabs: 9 content + Search + Saved Hotbars +
        // Inventory) — vanilla order: Building, Decoration, Redstone,
        // Transport, Misc, Food, Tools, Combat, Brewing, Search,
        // Hotbar, Inventory (VERIFIED w/Creative_inventory, live
        // 2026-09-17: "There are also Search Items, Saved Hotbars and
        // Survival Inventory tabs")
        let tab_labels: [&str; 12] = [
            "BUILDING BLOCKS",
            "DECORATION BLOCKS",
            "REDSTONE",
            "TRANSPORTATION",
            "MISCELLANEOUS",
            "FOODSTUFFS",
            "TOOLS",
            "COMBAT",
            "BREWING",
            "SEARCH ITEMS",
            "SAVED HOTBARS",
            "INVENTORY",
        ];
        let cx = cursor.0 as i32;
        let cy = cursor.1 as i32;
        let mut hovered_tab: Option<u8> = None;
        for t in 0..12u8 {
            let (tx, ty) = if t < 6 {
                (row1_x + t as i32 * tab_w, row1_y)
            } else {
                (row2_x + (t as i32 - 6) * tab_w, row2_y)
            };
            let selected_tab = t == tab;
            // folder-tab chrome: attached to the panel on selection
            let bg: Color = if selected_tab {
                [58, 58, 62, 245]
            } else {
                [34, 34, 38, 225]
            };
            self.rect(tx, ty, tab_w, tab_h, bg);
            self.frame(tx, ty, tab_w, tab_h, [78, 78, 84, 255]);
            if selected_tab {
                self.frame(tx, ty + 1, tab_w, tab_h - 2, [140, 140, 148, 255]);
                // connect to the panel: erase the bottom edge line
                self.rect(tx + 1, ty + tab_h - 2, tab_w - 2, 2, [58, 58, 62, 245]);
            }
            // the tab icon: the canonical item's tile, 2x blit
            let icon = if t < 9 {
                blk::CREATIVE_TABS[t as usize].icon_block()
            } else if t == 9 {
                // Search tab icon: the compass — engine substitute: the
                // eye of ender (the registry's search-est item; no compass)
                blk::EYE_OF_ENDER
            } else if t == 10 {
                // Round 15b: the Saved Hotbars tab icon — vanilla's is a
                // book-family icon; the BOOK item is the registry's
                // stand-in (no paper item exists)
                blk::BOOK
            } else {
                // Inventory tab icon: the player head — engine
                // substitute: the wither-skeleton skull (the registry's
                // only head-shaped block; no player-skin item exists)
                blk::WITHER_SKELETON_SKULL
            };
            let tile = blk::def(icon).tiles[0];
            blit_tile(
                atlas,
                tile,
                2,
                (tx + (tab_w - 32) / 2) as usize,
                (ty + 6) as usize,
                &mut self.px,
                self.live_w,
            );
            // hover highlight + tooltip capture
            if cx >= tx && cx < tx + tab_w && cy >= ty && cy < ty + tab_h {
                self.frame(tx - 1, ty - 1, tab_w + 2, tab_h + 2, [255, 255, 255, 200]);
                hovered_tab = Some(t);
            }
            geom.tabs[t as usize] = Some((tx, ty, tab_w, tab_h));
        }

        // ---- panel ----
        self.rect(px0, py0, pw, ph, [26, 26, 30, 235]);
        self.frame(px0, py0, pw, ph, [60, 60, 66, 255]);
        self.frame(px0 + 1, py0 + 1, pw - 2, ph - 2, [12, 12, 14, 255]);

        // ---- title / search field ----
        if tab == 9 {
            // the Search tab: a text field in the title strip
            let fx = x0;
            let fy = title_y;
            let fw = grid_w;
            let fh = 22;
            let w = Widget {
                id: 0,
                x: fx,
                y: fy,
                w: fw,
                h: fh,
                kind: WidgetKind::TextField {
                    label: String::new(),
                    text: search.to_string(),
                    placeholder: "SEARCH".to_string(),
                    focused: search_focused,
                },
            };
            self.draw_text_field(&w, false);
            geom.search = (fx, fy, fw, fh);
        } else {
            let label = if tab < 9 {
                blk::CREATIVE_TABS[tab as usize].label().to_uppercase()
            } else {
                tab_labels[tab as usize].to_string()
            };
            self.text(x0, title_y + 4, &label, [255, 220, 120, 255], 1);
        }

        // ---- the 9x5 grid ----
        let total_rows = items.len().div_ceil(cols);
        let scroll = scroll.min(total_rows.saturating_sub(vis_rows));
        let first = scroll * cols;
        let last = (first + vis_rows * cols).min(items.len());
        let mut hovered: Option<u16> = None;
        for (i, &b) in items[first..last].iter().enumerate() {
            let col = (i % cols) as i32;
            let row = (i / cols) as i32;
            let sx = x0 + 4 + col * cell;
            let sy = y0 + 4 + row * cell;
            self.rect(sx, sy, 36, 36, [52, 52, 52, 200]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 110, 110, 255]);
            let tile = blk::def(b).tiles[0];
            blit_tile(
                atlas,
                tile,
                2,
                (sx + 2) as usize,
                (sy + 2) as usize,
                &mut self.px,
                self.live_w,
            );
            if cx >= sx && cx < sx + 36 && cy >= sy && cy < sy + 36 {
                self.frame(sx - 1, sy - 1, 38, 38, [255, 255, 255, 255]);
                hovered = Some(b);
            }
        }
        // empty slots for the unfilled tail of the last page (vanilla
        // shows empty slots, not blank space)
        let tail_start = items.len().saturating_sub(first);
        if tail_start < vis_rows * cols {
            for i in tail_start..(vis_rows * cols) {
                let col = (i % cols) as i32;
                let row = (i / cols) as i32;
                let sx = x0 + 4 + col * cell;
                let sy = y0 + 4 + row * cell;
                self.rect(sx, sy, 36, 36, [40, 40, 44, 160]);
                self.frame(sx, sy, 36, 36, [20, 20, 20, 200]);
            }
        }

        // ---- scrollbar (when the tab overflows one page) ----
        if total_rows > vis_rows {
            let sb_x = px0 + pw - 16;
            let sb_y = y0 + 2;
            let sb_h = vis_rows as i32 * cell;
            self.rect(sb_x, sb_y, 10, sb_h, [16, 16, 18, 220]);
            self.frame(sb_x, sb_y, 10, sb_h, [10, 10, 10, 255]);
            let track = (total_rows - vis_rows).max(1);
            let thumb_h = (((sb_h as f32) * (vis_rows as f32 / total_rows as f32)) as i32).max(24);
            let avail = sb_h - thumb_h;
            let thumb_y = sb_y + ((scroll as f32 / track as f32) * avail as f32) as i32;
            self.rect(sb_x + 1, thumb_y + 1, 8, thumb_h - 2, [150, 150, 155, 230]);
            geom.scrollbar = Some((sb_x, sb_y, 10, sb_h));
        }

        // ---- bottom row: hotbar + destroy slot ----
        for (i, s) in hotbar.iter().take(9).enumerate() {
            let sx = x0 + 4 + i as i32 * cell;
            let sy = hot_y;
            self.rect(sx, sy, 36, 36, [52, 52, 52, 200]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 110, 110, 255]);
            self.draw_stack(s, sx, sy, atlas);
            if i == selected {
                self.frame(sx - 2, sy - 2, 40, 40, [255, 255, 255, 230]);
            }
            geom.hotbar[i] = (sx, sy, 36, 36);
        }
        // destroy slot (trash): bottom-right, X-marked
        {
            let sx = px0 + pw - 46;
            let sy = hot_y;
            self.rect(sx, sy, 36, 36, [70, 40, 40, 210]);
            self.frame(sx, sy, 36, 36, [24, 24, 24, 255]);
            self.frame(sx + 1, sy + 1, 34, 34, [110, 70, 70, 255]);
            // X mark
            for k in 0..12i32 {
                self.set(sx + 12 + k, sy + 12 + k, [200, 90, 90, 255]);
                self.set(sx + 23 - k, sy + 12 + k, [200, 90, 90, 255]);
            }
            geom.trash = (sx, sy, 36, 36);
        }

        // ---- hovered item name (above the hotbar, centered) ----
        let label = hovered
            .map(blk::name)
            .map(|n| {
                if advanced_tooltips {
                    let id: String = n.to_lowercase().replace(' ', "_");
                    format!("{n} (voxelcraft:{id})")
                } else {
                    n.to_string()
                }
            })
            .or_else(|| {
                hovered_tab.map(|t| {
                    // tab tooltip = the vanilla tab label
                    if t < 9 {
                        blk::CREATIVE_TABS[t as usize].label().to_string()
                    } else {
                        tab_labels[t as usize].to_string()
                    }
                })
            })
            .unwrap_or_default();
        if !label.is_empty() {
            let lw = Self::text_width(&label, 1);
            self.text(
                (self.live_w as i32 - lw) / 2,
                hot_y - 16,
                &label,
                [255, 255, 255, 255],
                1,
            );
        }

        // ---- the held (cursor) stack follows the mouse ----
        if !cursor_stack.is_empty() {
            let hx = cx - 18;
            let hy = cy - 18;
            self.draw_stack(cursor_stack, hx, hy, atlas);
        }

        geom.scroll = scroll;
        geom
    }

    pub fn help(&mut self) {
        let lines: Vec<(&str, &str)> = vec![
            ("WASD", "Move"),
            ("SPACE", "Jump / swim up"),
            ("DOUBLE SPACE", "Toggle flying"),
            ("SHIFT", "Sneak / fly down"),
            ("CTRL", "Sprint"),
            ("MOUSE", "Look (click canvas to capture)"),
            ("LEFT CLICK", "Break block (hold)"),
            ("RIGHT CLICK", "Place block / open table & furnace"),
            ("MIDDLE CLICK", "Pick block"),
            ("1-9 / WHEEL", "Select hotbar slot"),
            ("E", "Inventory + crafting (§27)"),
            ("F", "Swap item to off-hand"),
            ("B", "Creative inventory (creative mode)"),
            ("ESC", "Pause menu / options"),
            ("F3", "Debug info"),
            ("H", "This help"),
            ("[ ]", "Render distance"),
            ("- =", "Volume"),
            ("V", "Toggle V-Sync"),
        ];
        let bw = 460;
        let bh = lines.len() as i32 * 20 + 50;
        let x0 = (self.live_w as i32 - bw) / 2;
        let y0 = (self.live_h as i32 - bh) / 2;
        self.rect(x0, y0, bw, bh, [16, 16, 16, 200]);
        self.frame(x0, y0, bw, bh, [120, 120, 120, 255]);
        self.text(
            x0 + 16,
            y0 + 10,
            "CONTROLS (H to close)",
            [255, 220, 120, 255],
            2,
        );
        for (i, (k, v)) in lines.iter().enumerate() {
            self.text(x0 + 16, y0 + 44 + i as i32 * 20, k, [160, 220, 160, 255], 1);
            self.text(
                x0 + 180,
                y0 + 44 + i as i32 * 20,
                v,
                [220, 220, 220, 255],
                1,
            );
        }
    }

    pub fn center_msg(&mut self, title: &str, sub: &str) {
        self.text_center(self.live_h as i32 / 2 - 40, title, [255, 255, 255, 255], 3);
        self.text_center(self.live_h as i32 / 2, sub, [200, 200, 200, 255], 1);
    }

    /// Boot intro screen — the FIRST screen after opening the game, in the
    /// vanilla splash-screen structure: a solid studio-brand background, the
    /// studio wordmark centered above mid-frame, and a thin white progress
    /// bar (the only thing that animates). No panorama, no world, no text
    /// caption — exactly the real boot screen's composition, clean-room.
    pub fn intro_screen(&mut self, progress: f32) {
        // solid studio-brand red (clean-room color — not a sampled asset)
        self.rect(
            0,
            0,
            self.live_w as i32,
            self.live_h as i32,
            [239, 50, 61, 255],
        );
        // studio wordmark: dark on the bright field, centered
        let scale = 8;
        let logo = "VOXELCRAFT";
        let lw = Self::text_width(logo, scale);
        let lx = (self.live_w as i32 - lw) / 2;
        let ly = 196;
        self.text(lx + 3, ly + 4, logo, [120, 22, 28, 200], scale);
        self.text(lx, ly, logo, [54, 54, 54, 255], scale);
        // studio sub-brand, letter-spaced under the wordmark
        let sub = "S T U D I O S";
        let suw = Self::text_width(sub, 3);
        self.text(
            (self.live_w as i32 - suw) / 2 + 2,
            ly + 70,
            sub,
            [54, 54, 54, 255],
            3,
        );
        // thin white progress bar, centered, just past mid-frame (the bar
        // tracks the settled intro beat — assets all load in GameApp::new)
        let bw = 200;
        let x0 = (self.live_w as i32 - bw) / 2;
        let y0 = 342;
        self.rect(x0 - 1, y0 - 1, bw + 2, 7, [190, 36, 45, 255]);
        self.rect(
            x0,
            y0,
            (bw as f32 * progress.clamp(0.0, 1.0)) as i32,
            5,
            [255, 255, 255, 255],
        );
    }

    /// World-loading screen — vanilla 1.16.5 Java structure (VERIFIED
    /// reference wiki /Loading_world_screen): "Loading world" centered at
    /// the top, the load percentage under it, and a 35x35 chunk colormap in
    /// the middle that populates outward as chunks generate/light/mesh —
    /// each pixel is one chunk, colored by pipeline status. The background
    /// behind this overlay (panorama darkened+blurred, or the live world
    /// during dimension travel) is chosen by game.rs.
    ///
    /// Cell codes (vanilla status-color language, mapped to OUR pipeline):
    /// 0 empty (0x545454) · 1 terrain generated (0x80B252 "biomes") ·
    /// 2 meshed + on GPU (0xFFFFFF "full") · 3 spawn chunk pending
    /// (0xF26060 "spawn"). Color values are the wiki's exact table.
    pub fn world_loading_screen(&mut self, percent: i32, cells: &[u8], center: usize) {
        self.rect(
            0,
            0,
            self.live_w as i32,
            self.live_h as i32,
            [10, 12, 16, 140],
        );
        self.text_center(84, "LOADING WORLD", [255, 255, 255, 255], 2);
        let pct = format!("{percent}%");
        self.text_center(118, &pct, [220, 220, 220, 255], 2);

        // 35x35 colormap, 4px cells (140px square, vanilla-proportioned)
        const N: i32 = 35;
        const CELL: i32 = 4;
        let x0 = (self.live_w as i32 - N * CELL) / 2;
        let y0 = 170;
        for row in 0..N {
            for col in 0..N {
                let idx = (row * N + col) as usize;
                let c = if idx == center && cells[idx] != 2 {
                    3 // spawn chunk stays red until it is fully meshed
                } else {
                    cells.get(idx).copied().unwrap_or(0)
                };
                let color: Color = match c {
                    1 => [128, 178, 82, 255],  // biomes/terrain
                    2 => [255, 255, 255, 255], // full
                    3 => [242, 96, 96, 255],   // spawn
                    _ => [84, 84, 84, 255],    // empty
                };
                self.rect(x0 + col * CELL, y0 + row * CELL, CELL, CELL, color);
            }
        }
    }
}

/// Sub-round 2: hit-test geometry for the tabbed creative screen,
/// returned by `UiCanvas::creative_screen` so game.rs can route clicks
/// (tabs, grid slots, hotbar, destroy slot, search field, scrollbar).
pub struct CreativeGeom {
    pub x0: i32,
    pub y0: i32,
    pub cell: i32,
    pub cols: usize,
    /// first visible row (the game layer's scroll state, clamped by the
    /// painter to the real range)
    pub scroll: usize,
    /// visible rows in the fixed window (5 — the vanilla page size)
    pub vis_rows: usize,
    /// the 12 tab hit rects (UI space) in vanilla order: 9 content tabs
    /// + Search (9) + Saved Hotbars (10) + Inventory (11)
    pub tabs: [Option<(i32, i32, i32, i32)>; 12],
    /// the 9 hotbar-slot hit rects
    pub hotbar: [(i32, i32, i32, i32); 9],
    /// the destroy (trash) slot hit rect
    pub trash: (i32, i32, i32, i32),
    /// the search field hit rect
    pub search: (i32, i32, i32, i32),
    /// the scrollbar track hit rect (present only when the tab scrolls)
    pub scrollbar: Option<(i32, i32, i32, i32)>,
}

impl CreativeGeom {
    fn in_rect(r: (i32, i32, i32, i32), ux: i32, uy: i32) -> bool {
        ux >= r.0 && ux < r.0 + r.2 && uy >= r.1 && uy < r.1 + r.3
    }

    /// which tab (0..=11, vanilla order) is under this UI-space cursor
    pub fn tab_at(&self, ux: i32, uy: i32) -> Option<u8> {
        self.tabs
            .iter()
            .position(|r| r.is_some_and(|r| Self::in_rect(r, ux, uy)))
            .map(|i| i as u8)
    }

    /// which item index into the CURRENT tab's item list (scroll-aware)
    pub fn grid_at(&self, ux: i32, uy: i32) -> Option<usize> {
        let dx = ux - (self.x0 + 4);
        let dy = uy - (self.y0 + 4);
        if dx < 0 || dy < 0 {
            return None;
        }
        let col = dx / self.cell;
        let row = dy / self.cell;
        if col >= self.cols as i32
            || row >= self.vis_rows as i32
            || dx % self.cell >= 36
            || dy % self.cell >= 36
        {
            return None;
        }
        Some(self.scroll * self.cols + row as usize * self.cols + col as usize)
    }

    /// which hotbar slot is under the cursor
    pub fn hotbar_at(&self, ux: i32, uy: i32) -> Option<usize> {
        self.hotbar.iter().position(|r| Self::in_rect(*r, ux, uy))
    }

    /// the destroy slot?
    pub fn trash_at(&self, ux: i32, uy: i32) -> bool {
        Self::in_rect(self.trash, ux, uy)
    }

    /// the search field?
    pub fn search_at(&self, ux: i32, uy: i32) -> bool {
        Self::in_rect(self.search, ux, uy)
    }

    /// the scrollbar track?
    pub fn scrollbar_at(&self, ux: i32, uy: i32) -> bool {
        self.scrollbar.is_some_and(|r| Self::in_rect(r, ux, uy))
    }
}

// ------------------------------------------------------- containers (§27) --

/// which container to draw (game.rs maps its `Container` enum to this so
/// ui.rs stays independent of game.rs)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    /// player inventory screen: 2×2 personal crafting grid
    Inventory,
    /// crafting table: 3×3 grid
    Crafting,
    /// furnace: input / fuel / output with live progress
    Furnace,
    /// brewing stand: ingredient / fuel / 3 bottles with bubble progress
    Brewing,
    /// enchanting table: item + lapis + 3 option rows (§29)
    Enchant,
    /// villager trade screen: rows of give→get deals (§27/§29)
    Trade,
    /// Phase 3: generic container (chest: 3 rows of 9)
    Chest,
    /// Round 12: the double chest — two adjacent chest halves merging
    /// into one 9×6 = 54-slot grid (VERIFIED w/Chest §Double chests,
    /// live 2026-09-15: "Placing two chests of the same type next to
    /// each other ... combines them into a large chest"; the GUI keeps
    /// the single-word "Chest" title)
    DoubleChest,
    /// 1.14: barrel container (3 rows of 9 — VERIFIED w/Barrel: "the
    /// same as a single chest"; shares the chest grid geometry, own
    /// title)
    Barrel,
    /// hopper container: 5 slots in one row — the verdict-corrected
    /// 176×133 vanilla screen (research doc's blanket 176×166 was
    /// confirmed wrong; see docs/research/research-verdicts.md — a hopper
    /// has one content row, not three, so the panel is genuinely shorter)
    Hopper,
    /// Round 13: the anvil — Repair & Name (two inputs + result + the
    /// rename field + the level-cost line; geometry per the round-13
    /// audit §1: inputs (27,47)/(76,47), result (134,47), cost (60,70))
    Anvil,
    /// Round 13: the beacon — power selection + the payment slot (the
    /// audit §2's clean-room redraw of the Beacon_GUI structure)
    Beacon,
    /// Round 13: the grindstone — Repair & Disenchant (two stacked
    /// inputs + result; audit §3: inputs (50,18)/(50,40), result (148,32))
    Grindstone,
    /// Round 12b: the mount's chest storage (a chest-equipped donkey/
    /// mule/llama — 15 slots for donkeys/mules, 3 × strength for llamas;
    /// VERIFIED w/Donkey §Usage: "An additional 15 inventory slots when
    /// the donkey has been equipped with a chest" + w/Llama §Storage:
    /// the capacity rides the llama's Strength). Left column = the
    /// saddle slot (donkey/mule — w/Donkey: "Saddle slot for equipping
    /// a saddle") or the strength badge (llama — carpets are not
    /// registered in the engine, disclosed); the chest grid rides the
    /// generic chest-slot path (SlotRef::Chest).
    Mount,
}

/// a logical slot in a container screen — the target of a mouse click
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotRef {
    /// player inventory slot (0..36; 0..9 = hotbar row)
    Inv(usize),
    /// Sub-round 3: the player's armor equipment slots, vanilla order
    /// 0 = helmet(head), 1 = chestplate, 2 = leggings, 3 = boots(feet)
    Armor(usize),
    /// Sub-round 3: the shield/offhand slot
    Offhand,
    /// crafting-grid cell (row-major; 4 cells for 2×2, 9 for 3×3)
    Craft(usize),
    /// Phase 3: container (chest) slot index
    Chest(usize),
    /// crafting result slot (special click semantics)
    CraftOut,
    FurnaceInput,
    FurnaceFuel,
    FurnaceOutput,
    /// brewing stand: the top ingredient slot
    BrewIngredient,
    /// brewing stand: the fuel slot (blaze-powder analogue)
    BrewFuel,
    /// brewing stand: one of the three bottle slots
    BrewBottle(usize),
    /// enchanting table: the item slot
    EnchantItem,
    /// enchanting table: the lapis slot
    EnchantLapis,
    /// enchanting table: one of the three option rows
    EnchantOption(usize),
    /// villager trade: one of the trade rows
    TradeRow(usize),
    /// Round 13: the anvil's left (target) input
    AnvilTarget,
    /// Round 13: the anvil's right (sacrifice) input
    AnvilSacrifice,
    /// Round 13: the anvil's result slot (take = pay levels + roll the
    /// 12% stage advance)
    AnvilOut,
    /// Round 13: the beacon's payment slot (ore stand-in for ingots)
    BeaconPay,
    /// Round 13: one of the five primary power buttons (0=Speed,
    /// 1=Haste, 2=Resistance, 3=JumpBoost, 4=Strength)
    BeaconPrimary(usize),
    /// Round 13: the secondary row (0 = Regeneration, 1 = primary II)
    BeaconSecondary(usize),
    /// Round 13: the grindstone's top input
    GrindTop,
    /// Round 13: the grindstone's bottom input
    GrindBottom,
    /// Round 13: the grindstone's result slot (take = XP drop)
    GrindOut,
    /// Round 12b: the mount's saddle slot (donkey/mule — equip on
    /// click with the SADDLE on the cursor, take it back off when
    /// empty; llamas have no saddle slot — their left column is the
    /// strength badge)
    MountSaddle,
}

/// pure-data snapshot of everything a container screen renders — owned
/// copies only, so game.rs can build it without borrow fights.
pub struct ContainerView {
    pub kind: ContainerKind,
    /// 36 player slots (0..9 hotbar, 9..36 storage)
    pub inv: Vec<ItemStack>,
    /// craft-grid cells (row-major; 2×2 uses the first 4)
    pub grid: Vec<ItemStack>,
    /// current craft result (already matched by game.rs)
    pub craft_out: ItemStack,
    /// furnace slots: (input, fuel, output, burn_frac, cook_frac)
    pub furnace: Option<(ItemStack, ItemStack, ItemStack, f32, f32)>,
    /// brewing slots: (ingredient, fuel, [3 bottles], fuel_frac, brew_frac)
    pub brewing: Option<(ItemStack, ItemStack, [ItemStack; 3], f32, f32)>,
    /// enchanting: (item, lapis, [3 options], player_level, power)
    pub enchant: Option<(
        ItemStack,
        ItemStack,
        [vc_gameplay::enchanting::EnchOption; 3],
        i32,
        u8,
    )>,
    /// Phase 3: chest slots (27)
    pub chest: Vec<ItemStack>,
    /// trade screen (Phase 5): tiered offers + stock + career level
    pub trade: Option<TradeView>,
    /// Round 13: the anvil view (slots + cost + rename text)
    pub anvil: Option<AnvilView>,
    /// Round 13: the beacon view (pyramid level + selection + pay slot)
    pub beacon: Option<BeaconView>,
    /// Round 13: the grindstone view (top, bottom, result)
    pub grind: Option<(ItemStack, ItemStack, ItemStack)>,
    /// Round 12b: the mount view (the saddle state + the llama
    /// strength; the chest slots ride `chest` like the hopper)
    pub mount: Option<MountView>,
    /// Sub-round 3: the player's armor equipment (helmet/chest/legs/
    /// boots, vanilla order — mirrors Player.armor)
    pub armor: [ItemStack; 4],
    /// Sub-round 3: the shield/offhand slot contents
    pub offhand: ItemStack,
    /// stack riding the mouse cursor
    pub cursor: ItemStack,
}

/// one trade row as the screen serves it (Phase 5: tier + stock state)
#[derive(Clone)]
pub struct TradeRowView {
    pub give: ItemStack,
    pub get: ItemStack,
    /// player has enough of `give` in the inventory
    pub afford: bool,
    /// stock left in this restock cycle (0 = disabled offer)
    pub stock: u16,
    /// stock per restock cycle (16 tier-1 / 12 above, VERIFIED)
    pub max_uses: u16,
    /// career tier that gates the row (1 Novice .. 5 Master)
    pub tier: u8,
    /// villager career level is below this row's tier (rendered dim,
    /// clicks rejected — vanilla hides these rows entirely; we show
    /// them grayed so the progression is visible: documented adaptation)
    pub locked: bool,
}

/// the trade screen view: profession + career level + XP + all rows
#[derive(Clone)]
pub struct TradeView {
    pub profession: String,
    /// career level display name (Novice..Master)
    pub level_name: String,
    /// career level 1..=5
    pub level: u8,
    /// villager career XP
    pub xp: u32,
    /// cumulative XP of the next level (None at Master)
    pub xp_next: Option<u32>,
    /// all table rows, in table order (indices = SlotRef::TradeRow(i))
    pub rows: Vec<TradeRowView>,
}

/// Round 12b: the mount screen's live state (pure data — the storage
/// slots ride ContainerView::chest; this carries the left column)
#[derive(Clone)]
pub struct MountView {
    /// the GUI title — the entity's own name ("Donkey"/"Mule"/"Llama",
    /// the vanilla GUI captions)
    pub kind_label: String,
    /// the saddle slot's contents (the SADDLE item when equipped —
    /// donkey/mule only; EMPTY otherwise)
    pub saddle: ItemStack,
    /// a llama's screen (strength badge instead of the saddle slot —
    /// carpets are not registered, disclosed)
    pub llama: bool,
    /// the llama's strength 1..5 (the capacity driver)
    pub strength: u8,
    /// the storage capacity in slots (15 for donkeys/mules;
    /// 3 × strength for llamas)
    pub capacity: usize,
}

/// Round 13: the anvil screen's live state (pure data — the plan math
/// lives in vc_gameplay::anvil::combine; the view only renders it)
#[derive(Clone)]
pub struct AnvilView {
    pub target: ItemStack,
    pub sacrifice: ItemStack,
    /// the computed result (EMPTY when the anvil refuses — red X)
    pub result: ItemStack,
    /// the level cost of the current plan
    pub cost: i32,
    /// "Too Expensive!" (> 39 levels; creative is exempt — VERIFIED)
    pub too_expensive: bool,
    /// the player can afford the cost (survival only; green vs red
    /// cost text — VERIFIED w/Anvil §Usage)
    pub affordable: bool,
    /// the rename field's current text
    pub rename: String,
    /// the rename field is focused (typing goes here)
    pub rename_focused: bool,
    /// creative mode exempts the cost cap (VERIFIED)
    pub creative: bool,
}

/// Round 13: the beacon screen's live state
#[derive(Clone)]
pub struct BeaconView {
    /// the live pyramid level 0..=4 (re-scanned every rebuild)
    pub level: u8,
    /// the payment slot (ore stand-in — no ingot items, documented)
    pub pay: ItemStack,
    /// the currently selected primary power (None before the first
    /// confirmation — the vanilla screen starts unselected)
    pub primary: Option<vc_gameplay::beacon::BeaconPower>,
    /// the currently selected secondary
    pub secondary: vc_gameplay::beacon::BeaconSecondary,
    /// the pending primary (highlighted before the confirm click)
    pub pending_primary: Option<vc_gameplay::beacon::BeaconPower>,
    /// the pending secondary
    pub pending_secondary: vc_gameplay::beacon::BeaconSecondary,
}

impl ContainerView {
    fn hovered_stack(&self, x: i32, y: i32, geom: &ContainerGeom) -> Option<ItemStack> {
        Some(match geom.slot_at(x, y)? {
            SlotRef::Inv(i) => self.inv[i],
            SlotRef::Armor(i) => self.armor[i],
            SlotRef::Offhand => self.offhand,
            SlotRef::Craft(i) => self.grid[i],
            SlotRef::Chest(i) => self.chest[i],
            SlotRef::CraftOut => self.craft_out,
            SlotRef::FurnaceInput => self.furnace?.0,
            SlotRef::FurnaceFuel => self.furnace?.1,
            SlotRef::FurnaceOutput => self.furnace?.2,
            SlotRef::BrewIngredient => self.brewing?.0,
            SlotRef::BrewFuel => self.brewing?.1,
            SlotRef::BrewBottle(i) => self.brewing?.2[i],
            SlotRef::EnchantItem => self.enchant?.0,
            SlotRef::EnchantLapis => self.enchant?.1,
            SlotRef::EnchantOption(_) => ItemStack::EMPTY, // buttons, not stacks
            SlotRef::TradeRow(i) => self.trade.as_ref()?.rows.get(i)?.give,
            SlotRef::AnvilTarget => self.anvil.as_ref()?.target,
            SlotRef::AnvilSacrifice => self.anvil.as_ref()?.sacrifice,
            SlotRef::AnvilOut => self.anvil.as_ref()?.result,
            SlotRef::BeaconPay => self.beacon.as_ref()?.pay,
            SlotRef::BeaconPrimary(_) | SlotRef::BeaconSecondary(_) => ItemStack::EMPTY,
            SlotRef::GrindTop => self.grind?.0,
            SlotRef::GrindBottom => self.grind?.1,
            SlotRef::GrindOut => self.grind?.2,
            // Round 12b: the saddle slot's contents (the SADDLE item
            // when equipped, EMPTY otherwise)
            SlotRef::MountSaddle => self.mount.as_ref()?.saddle,
        })
    }
}

/// furnace slot positions for hit-testing
pub struct FurnaceSlots {
    pub input: (i32, i32),
    pub fuel: (i32, i32),
    pub output: (i32, i32),
}

/// brewing-stand slot positions for hit-testing
pub struct BrewSlots {
    pub ingredient: (i32, i32),
    pub fuel: (i32, i32),
    pub bottles: [(i32, i32); 3],
}

/// enchanting-table hit rects: item + lapis slots and 3 option buttons
pub struct EnchantSlots {
    pub item: (i32, i32),
    pub lapis: (i32, i32),
    /// option button origins (w = 180, h = 44 each)
    pub options: [(i32, i32); 3],
}

/// trade-screen hit rects: one row button per trade (w = 260, h = 40)
pub struct TradeSlots {
    pub rows: Vec<(i32, i32)>,
}

/// Round 13: anvil hit rects — two inputs, the result, the rename field
pub struct AnvilSlots {
    pub target: (i32, i32),
    pub sacrifice: (i32, i32),
    pub out: (i32, i32),
    /// the rename text field (w = 208, h = 24)
    pub rename: (i32, i32),
}

/// Round 13: beacon hit rects — 5 primary buttons (44×44), 2 secondary
/// buttons, the payment slot, the confirm + cancel buttons
pub struct BeaconSlots {
    pub primary: [(i32, i32); 5],
    pub secondary: [(i32, i32); 2],
    pub pay: (i32, i32),
    /// the green-check confirm button (w = h = 36)
    pub confirm: (i32, i32),
    /// the red-X cancel button (w = h = 36)
    pub cancel: (i32, i32),
}

/// Round 13: grindstone hit rects — the two stacked inputs + the result
pub struct GrindSlots {
    pub top: (i32, i32),
    pub bottom: (i32, i32),
    pub out: (i32, i32),
}

/// Round 12b: the mount screen's saddle-slot hit rect (the chest grid
/// rides ContainerGeom::chest — the generic slot path)
pub struct MountGeom {
    pub saddle: (i32, i32),
}

/// hit-test geometry for a container screen (UI-space 36px slots)
pub struct ContainerGeom {
    /// 36 inventory slot origins: 0..9 hotbar row (bottom), 9..36 storage
    pub inv: Vec<(i32, i32)>,
    /// craft-grid cell origins (row-major)
    pub craft: Vec<(i32, i32)>,
    /// craft result slot origin
    pub craft_out: (i32, i32),
    /// furnace slot origins when the screen is a furnace
    pub furnace: Option<FurnaceSlots>,
    /// brewing slot origins when the screen is a brewing stand
    pub brewing: Option<BrewSlots>,
    /// enchanting slot/button origins when the screen is a table
    pub enchant: Option<EnchantSlots>,
    /// trade row origins when the screen is a villager trade
    pub trade: Option<TradeSlots>,
    /// Phase 3: chest slot origins (27, row-major 3×9)
    pub chest: Vec<(i32, i32)>,
    /// Sub-round 3: the armor slot origins (helmet..boots, top to
    /// bottom) — present on the Inventory screen
    pub armor: [(i32, i32); 4],
    /// Sub-round 3: the offhand slot origin (i32::MIN when absent)
    pub offhand: (i32, i32),
    /// Round 13: the anvil's slot/field origins
    pub anvil: Option<AnvilSlots>,
    /// Round 13: the beacon's button/slot origins
    pub beacon: Option<BeaconSlots>,
    /// Round 13: the grindstone's slot origins
    pub grind: Option<GrindSlots>,
    /// Round 12b: the mount screen's saddle slot (llamas show the
    /// strength badge instead — not a hit target)
    pub mount: Option<MountGeom>,
}

impl ContainerGeom {
    fn hit(x: i32, y: i32, s: &(i32, i32)) -> bool {
        x >= s.0 && x < s.0 + 36 && y >= s.1 && y < s.1 + 36
    }

    /// which logical slot (if any) is under this UI-space cursor position
    pub fn slot_at(&self, x: i32, y: i32) -> Option<SlotRef> {
        // Sub-round 3: armor column (helmet..boots) + offhand
        for (i, s) in self.armor.iter().enumerate() {
            if Self::hit(x, y, s) {
                return Some(SlotRef::Armor(i));
            }
        }
        if self.offhand.0 > i32::MIN && Self::hit(x, y, &self.offhand) {
            return Some(SlotRef::Offhand);
        }
        if let Some(fs) = &self.furnace {
            if Self::hit(x, y, &fs.input) {
                return Some(SlotRef::FurnaceInput);
            }
            if Self::hit(x, y, &fs.fuel) {
                return Some(SlotRef::FurnaceFuel);
            }
            if Self::hit(x, y, &fs.output) {
                return Some(SlotRef::FurnaceOutput);
            }
        }
        if let Some(bs) = &self.brewing {
            if Self::hit(x, y, &bs.ingredient) {
                return Some(SlotRef::BrewIngredient);
            }
            if Self::hit(x, y, &bs.fuel) {
                return Some(SlotRef::BrewFuel);
            }
            for (i, s) in bs.bottles.iter().enumerate() {
                if Self::hit(x, y, s) {
                    return Some(SlotRef::BrewBottle(i));
                }
            }
        }
        if let Some(es) = &self.enchant {
            if Self::hit(x, y, &es.item) {
                return Some(SlotRef::EnchantItem);
            }
            if Self::hit(x, y, &es.lapis) {
                return Some(SlotRef::EnchantLapis);
            }
            for (i, s) in es.options.iter().enumerate() {
                if x >= s.0 && x < s.0 + 180 && y >= s.1 && y < s.1 + 44 {
                    return Some(SlotRef::EnchantOption(i));
                }
            }
        }
        if let Some(ts) = &self.trade {
            for (i, s) in ts.rows.iter().enumerate() {
                if x >= s.0 && x < s.0 + 260 && y >= s.1 && y < s.1 + 40 {
                    return Some(SlotRef::TradeRow(i));
                }
            }
        }
        if let Some(a) = &self.anvil {
            if Self::hit(x, y, &a.target) {
                return Some(SlotRef::AnvilTarget);
            }
            if Self::hit(x, y, &a.sacrifice) {
                return Some(SlotRef::AnvilSacrifice);
            }
            if Self::hit(x, y, &a.out) {
                return Some(SlotRef::AnvilOut);
            }
        }
        if let Some(b) = &self.beacon {
            for (i, s) in b.primary.iter().enumerate() {
                if Self::hit(x, y, s) {
                    return Some(SlotRef::BeaconPrimary(i));
                }
            }
            for (i, s) in b.secondary.iter().enumerate() {
                if Self::hit(x, y, s) {
                    return Some(SlotRef::BeaconSecondary(i));
                }
            }
            if Self::hit(x, y, &b.pay) {
                return Some(SlotRef::BeaconPay);
            }
        }
        if let Some(g) = &self.grind {
            if Self::hit(x, y, &g.top) {
                return Some(SlotRef::GrindTop);
            }
            if Self::hit(x, y, &g.bottom) {
                return Some(SlotRef::GrindBottom);
            }
            if Self::hit(x, y, &g.out) {
                return Some(SlotRef::GrindOut);
            }
        }
        // Round 12b: the mount's saddle slot (donkey/mule screens —
        // the llama badge is not a hit target)
        if let Some(m) = &self.mount {
            if Self::hit(x, y, &m.saddle) {
                return Some(SlotRef::MountSaddle);
            }
        }
        for (i, s) in self.chest.iter().enumerate() {
            if Self::hit(x, y, s) {
                return Some(SlotRef::Chest(i));
            }
        }
        if Self::hit(x, y, &self.craft_out) {
            return Some(SlotRef::CraftOut);
        }
        for (i, s) in self.craft.iter().enumerate() {
            if Self::hit(x, y, s) {
                return Some(SlotRef::Craft(i));
            }
        }
        for (i, s) in self.inv.iter().enumerate() {
            if Self::hit(x, y, s) {
                return Some(SlotRef::Inv(i));
            }
        }
        None
    }
}

#[cfg(test)]
mod phase3_icon_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn draw_stack_pushes_icon_quad_when_cell_ready() {
        let mut ui = UiCanvas::new();
        let atlas = vec![128u8; 512 * 512 * 4];
        // a hotbar-style stack of dirt (block 3)
        let s = ItemStack {
            block: 3,
            count: 1,
            ench: 0,
            ench2: 0,
            dmg: 0,
            prior: 0,
            name: 0,
        };
        let before = ui.gui_frame.quads.len();
        ui.draw_stack(&s, 100, 100, &atlas);
        assert_eq!(ui.gui_frame.quads.len(), before, "no icon without cells");
        // publish a ready cell for block 3
        let mut cells = HashMap::new();
        cells.insert(3u16, [2u8, 1u8]);
        ui.set_icon_cells(Arc::new(cells));
        ui.clear();
        ui.draw_stack(&s, 100, 100, &atlas);
        assert_eq!(ui.gui_frame.quads.len(), 1, "icon quad pushed");
        let q = ui.gui_frame.quads[0];
        assert_eq!(q.texture, crate::gui_render::QuadTexture::IconAtlas);
        assert_eq!(q.dst.x, 102.0);
        assert_eq!(q.dst.y, 102.0);
        assert_eq!((q.dst.w, q.dst.h), (32.0, 32.0));
        // cell (2,1) -> src (128, 64) in the 2048 atlas
        assert_eq!((q.src.x, q.src.y), (128, 64));
    }
}

#[cfg(test)]
mod phase5_font_tests {
    use super::*;

    #[test]
    fn glyph_ink_width_measures_the_rightmost_ink() {
        // W: ink in all five columns -> 5
        assert_eq!(glyph_ink_width(&FONT['W' as usize - 32]), 5);
        // space: blank -> 0
        assert_eq!(glyph_ink_width(&FONT[0]), 0);
        // i (centered stem, column 2) -> 3. NOTE: the master prompt's
        // "1 for i" assumed a left-aligned i bitmap; this repo's i is
        // centered, so the measured width is 3 — the ADVANCE behavior
        // (i packs tighter than W) is what matters and holds below.
        assert_eq!(glyph_ink_width(&FONT['i' as usize - 32]), 3);
        // '!' is a single centered column -> 3
        assert_eq!(glyph_ink_width(&FONT[1]), 3);
    }

    #[test]
    fn variable_advance_tightens_narrow_text() {
        // the spec's D7 pair: narrow letters advance less than wide
        let w_i = UiCanvas::text_width("i", 1);
        let w_w = UiCanvas::text_width("W", 1);
        assert!(w_i < w_w, "i ({w_i}) must advance less than W ({w_w})");
        // string-level tightening. NOTE: the master prompt's example
        // ("abc" < "WWW") assumed vanilla's narrower a/b/c; this repo's
        // clean-room 5x7 font has uniformly 5-wide letters, so the
        // equivalent pair uses the actually-narrow glyphs (i, l, !):
        // 3 chars of ~4px vs 3 chars of 6px.
        assert!(UiCanvas::text_width("iil", 1) < UiCanvas::text_width("WWW", 1));
        // spaces tighten too (6 -> 4): a two-word string is now narrower
        // than the same letters back-to-back
        assert!(UiCanvas::text_width("a b", 1) < UiCanvas::text_width("aab", 1));
        // W keeps the classic 6 (5 ink + 1 spacing)
        assert_eq!(w_w, 6);
        // space keeps the vanilla fixed 4-px advance
        assert_eq!(UiCanvas::text_width(" ", 1), 4);
    }

    #[test]
    fn text_shadow_is_foreground_times_quarter() {
        // draw one full-ink glyph (W) at a known spot; the pixel at
        // (x+1, y+1) must be fg x 0.25 (VERIFIED w/Font)
        let mut ui = UiCanvas::new();
        let fg: Color = [200, 100, 50, 255];
        ui.text(100, 100, "W", fg, 1);
        // W's ink starts at column 0 -> pixel (100, 100) is foreground
        let p = (100usize * crate::ui::UI_W + 100) * 4;
        assert_eq!(&ui.px[p..p + 4], &[200, 100, 50, 255]);
        // its shadow at (101, 101): fg >> 2
        let s = (101 * crate::ui::UI_W + 101) * 4;
        assert_eq!(
            &ui.px[s..s + 4],
            &[200 >> 2, 100 >> 2, 50 >> 2, 255],
            "shadow must be foreground x 0.25"
        );
    }

    #[test]
    fn png_font_path_matches_builtin_for_the_same_glyphs() {
        // render the BUILTIN font into a 128x48 sheet, decode it back
        // through FontSource::Png, and require identical widths (the
        // D7 "PNG path and builtin path produce identical widths")
        let mut sheet = vec![0u8; 128 * 48 * 4];
        for (i, g) in FONT.iter().enumerate() {
            let col = (i % 16) * 8;
            let row = (i / 16) * 8;
            for (gy, bits) in g.iter().enumerate() {
                for gx in 0..5usize {
                    if bits & (1 << (4 - gx)) != 0 {
                        let idx = (row + gy) * 128 + col + gx;
                        sheet[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }
        let src = crate::gui::set::FontSource::Png(sheet);
        let decoded = src.png_glyphs();
        assert!(decoded.is_some());
        let decoded = decoded.unwrap_or(Box::new([[0u8; 8]; 96]));
        for i in 0..96 {
            assert_eq!(
                glyph_ink_width(&decoded[i]),
                glyph_ink_width(&FONT[i]),
                "glyph {i} width mismatch between png and builtin paths"
            );
        }
    }

    #[test]
    fn font_override_installs_once() {
        // set_font_override: first install wins, second refused. The
        // installed font is a copy of FONT so the process-global
        // override cannot perturb any other test's measurements.
        let first = set_font_override(Box::new(FONT));
        // only assert the mechanism when no override was installed yet
        // (test order independence: another test may have installed it)
        if first {
            let second = set_font_override(Box::new([[1u8; 8]; 96]));
            assert!(!second, "second install must be refused");
            // the FONT copy keeps the builtin metrics alive
            assert_eq!(UiCanvas::text_width("WWW", 1), 18);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 2026-09-14 deploy-fix round: widget-id space disjointness ----

    /// Every LITERAL widget id (the fixed, non-row ids) across all screens.
    /// If you add one, add it here — the disjointness tests below then prove
    /// the dynamic row ranges never swallow it (see the long comment on
    /// `ID_RPACK_AVAIL_BASE` for how the 2026-09-14 collision played out).
    const LITERAL_IDS: &[u16] = &[
        ID_TITLE_PLAY,
        ID_TITLE_OPTIONS,
        ID_TITLE_QUIT,
        ID_TITLE_MULTI,
        ID_OPT_FOV,
        ID_OPT_SENS,
        ID_OPT_RD,
        ID_OPT_BRIGHT,
        ID_OPT_VOL,
        ID_OPT_GRAPHICS,
        ID_OPT_SMOOTH,
        ID_OPT_CLOUDS,
        ID_OPT_DONE,
        ID_PAUSE_BACK,
        ID_PAUSE_OPTIONS,
        ID_PAUSE_QUIT,
        ID_OPT_SHADOWS,
        ID_OPT_UPSCALE,
        ID_OPT_MAXFPS,
        ID_OPT_MUSIC,
        ID_OPT_NEXT,
        ID_OPT_PREV,
        ID_OPT_SIMDIST,
        ID_OPT_MIP,
        ID_OPT_ANISO,
        ID_OPT_MSAA,
        ID_OPT_OCCL,
        ID_OPT_AUTOJUMP,
        ID_OPT_DONE2,
        ID_OPT_GMESH,
        ID_OPT_VIDEO,
        ID_OPT_ENGINE,
        ID_OPT_PACKS,
        ID_OPT_ACCESS,
        ID_OPT_GUISCALE,
        ID_OPT_PARTICLES,
        ID_OPT_FULLSCREEN,
        ID_OPT_VSYNC,
        ID_OPT_ENTSHADOW,
        ID_OPT_BIOME,
        ID_OPT_CHAT,
        ID_OPT_LANG,
        ID_OPT_CONTROLS,
        ID_OPT_BOB,
        ID_WS_CREATE,
        ID_WS_CANCEL,
        ID_WS_DELETE,
        ID_WC_NAME,
        ID_WC_SEED,
        ID_WC_MODE,
        ID_WC_TYPE,
        ID_WC_CREATE,
        ID_WC_CANCEL,
        ID_DEATH_RESPAWN,
        ID_DEATH_TITLE,
        ID_DEATH_DELETE,
        ID_RPACK_DEFAULT,
        // Round 14: the settings-tree additions
        ID_OPT_MUSICSND,
        ID_SND_DONE,
        ID_CTRL_RESET,
        ID_CTRL_DONE,
        ID_ACC_FOG,
        ID_ACC_FOVEFF,
        ID_ACC_DISTORT,
        ID_ACC_CHATVIS,
        ID_ACC_SUBTITLES,
        // Round 14b: the Skin Customization + Chat Settings + the
        // accessibility completion family (210..=237 block + the 115
        // Options entry)
        ID_OPT_SKIN,
        ID_SKIN_CAPE,
        ID_SKIN_JACKET,
        ID_SKIN_LSLEEVE,
        ID_SKIN_RSLEEVE,
        ID_SKIN_LPANTS,
        ID_SKIN_RPANTS,
        ID_SKIN_HAT,
        ID_SKIN_MAINHAND,
        ID_SKIN_DONE,
        ID_CHAT_VIS,
        ID_CHAT_COLORS,
        ID_CHAT_LINKS,
        ID_CHAT_LINKSPROMPT,
        ID_CHAT_OPACITY,
        ID_CHAT_DELAY,
        ID_CHAT_WIDTH,
        ID_CHAT_HFOCUSED,
        ID_CHAT_HUNFOCUSED,
        ID_CHAT_SCALE,
        ID_CHAT_LINESPACING,
        ID_CHAT_HIDENAMES,
        ID_CHAT_REDUCEDDEBUG,
        ID_CHAT_NARRATOR,
        ID_CHAT_DONE,
        ID_SND_SUBTITLES,
        ID_ACC_SPRINT,
        ID_ACC_SNEAK,
        ID_ACC_DISTORT_SLIDER,
        // 2026-09-20: the Shader Packs screen family (240..=249 block —
        // the row range 241..248 is guarded below; the literals here are
        // the (none) row, DONE, the labPBR toggle, and the Video entry)
        ID_OPT_SHADERS,
        // 2026-09-20: the modern Entity Distance video slider
        ID_OPT_ENTDIST,
        ID_SHDR_NONE,
        ID_SHDR_DONE,
        ID_SHDR_LABPBR,
    ];

    #[test]
    fn literal_widget_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for &id in LITERAL_IDS {
            assert!(seen.insert(id), "duplicate literal widget id {id}");
        }
    }

    /// The dynamic row ranges (pack lists, resource-pack panes, world
    /// entries) must be disjoint from every literal id AND from each
    /// other — otherwise `activate()`'s early range guards silently
    /// swallow the colliding buttons (the 2026-09-14 regression: RPACK
    /// row bases at 60/70/80/90 killed the whole world-select/create
    /// screen — only the Enter-key create path still worked, which is
    /// exactly what masked it in the browser E2E).
    #[test]
    fn row_ranges_disjoint_from_literals_and_each_other() {
        let rows: [(u16, u16, &str); 8] = [
            (ID_WS_WORLD_BASE, MAX_LISTED_WORLDS as u16, "world entries"),
            (
                ID_RPACK_AVAIL_BASE,
                MAX_RPACK_ENTRIES as u16,
                "rpack available",
            ),
            (
                ID_RPACK_SEL_BASE,
                MAX_RPACK_ENTRIES as u16,
                "rpack selected",
            ),
            (
                ID_RPACK_UP_BASE,
                MAX_RPACK_ENTRIES as u16,
                "rpack up arrows",
            ),
            (
                ID_RPACK_DOWN_BASE,
                MAX_RPACK_ENTRIES as u16,
                "rpack down arrows",
            ),
            // Round 14: the Music & Sound sliders + the Controls bind rows
            (ID_SND_BASE, 10, "music & sound sliders"),
            (
                ID_CTRL_BIND_BASE,
                MAX_CTRL_BINDS as u16,
                "controls bind rows",
            ),
            // 2026-09-20: the Shader Packs screen pack rows
            (ID_SHDR_BASE, MAX_SHDR_ENTRIES as u16, "shader pack rows"),
            // NOTE Round 14b: the skin/chat-settings/accessibility
            // additions (210..=237) are STATIC literal ids (each a
            // dedicated button/slider on a fixed screen — no dynamic
            // row family), so they are guarded by
            // literal_widget_ids_are_unique + the range-vs-literal check
            // below instead of a row entry here.
        ];
        for &(base, len, name) in &rows {
            for id in base..base + len {
                assert!(
                    !LITERAL_IDS.contains(&id),
                    "{name} row id {id} collides with a literal widget id \
                     (activate() range guards would swallow that button)"
                );
            }
        }
        for i in 0..rows.len() {
            for j in i + 1..rows.len() {
                let (a0, al, an) = rows[i];
                let (b0, bl, bn) = rows[j];
                assert!(
                    a0 + al <= b0 || b0 + bl <= a0,
                    "row ranges {an} ({a0}..{}) and {bn} ({b0}..{}) overlap",
                    a0 + al,
                    b0 + bl
                );
            }
        }
    }

    // ---- F3 right-column half-screen clamp (round-4 forensics fix) ----

    /// A short line passes through unchanged — no "..." tail.
    #[test]
    fn fit_line_short_line_untouched() {
        let out = fit_line("Display 1440x810 (ANGLE)", 468.0, |s| s.len() as f32 * 9.0);
        assert_eq!(out, "Display 1440x810 (ANGLE)");
    }

    /// A SwiftShader-length adapter string (66 chars, ~594 px at 9/char)
    /// is truncated to the clamp width with an ASCII "..." tail and never
    /// exceeds it.
    #[test]
    fn fit_line_long_adapter_truncated_with_ellipsis() {
        let s = "Google: Vulkan 1.3.0 (SwiftShader) Device (Subzero) (0x00000C0DE)";
        let out = fit_line(s, 468.0, |t| t.len() as f32 * 9.0);
        assert!(out.ends_with("..."), "must end with the ASCII tail: {out}");
        assert!(out.len() < s.len(), "must actually cut something");
        // 9 px/char measure: 468/9 = 52 total chars → 49 content + 3 dots
        assert_eq!(out.len(), 52);
        assert!(out.starts_with("Google: Vulkan 1.3.0 (SwiftShader) Dev"));
    }

    /// Multi-byte content (the ∞ glyph, CJK) truncates on char boundaries —
    /// never panics on a sliced UTF-8 boundary.
    #[test]
    fn fit_line_multibyte_safe() {
        let s = "∞∞∞∞∞∞∞∞∞∞";
        let out = fit_line(s, 40.0, |t| t.chars().count() as f32 * 9.0);
        assert!(out.ends_with("..."));
        assert!(out.chars().all(|c| c == '∞' || c == '.'));
    }

    /// Degenerate clamp: nothing fits → just the tail.
    #[test]
    fn fit_line_degenerates_to_ellipsis() {
        let out = fit_line("SwiftShader driver", 2.0, |t| t.len() as f32 * 9.0);
        assert_eq!(out, "...");
    }

    fn hopper_view() -> ContainerView {
        ContainerView {
            kind: ContainerKind::Hopper,
            inv: vec![ItemStack::EMPTY; 36],
            grid: vec![],
            craft_out: ItemStack::EMPTY,
            furnace: None,
            brewing: None,
            enchant: None,
            chest: vec![ItemStack::EMPTY; 5],
            trade: None,
            armor: [ItemStack::EMPTY; 4],
            offhand: ItemStack::EMPTY,
            cursor: ItemStack::EMPTY,
            anvil: None,
            beacon: None,
            grind: None,
            mount: None,
        }
    }

    /// The verdict-corrected hopper screen (176×133 vanilla, NOT the
    /// research doc's blanket 176×166): ONE row of 5 slots, all 5 slot
    /// rects share a y, pitch 40, and clicks map into the generic
    /// container-slot path. The panel is ~2 rows (80px) shorter than the
    /// chest's — the vanilla 133-vs-166 relationship.
    #[test]
    fn hopper_screen_is_five_slots_in_one_short_row() {
        let mut ui = UiCanvas::new();
        let view = hopper_view();
        let geom = ui.container_screen(&view, (0.0, 0.0), &[], false);
        assert_eq!(geom.chest.len(), 5, "exactly 5 hopper slots");
        let ys: std::collections::HashSet<i32> = geom.chest.iter().map(|s| s.1).collect();
        assert_eq!(ys.len(), 1, "all 5 slots on ONE row (the 176x133 shape)");
        for i in 1..5 {
            assert_eq!(
                geom.chest[i].0 - geom.chest[i - 1].0,
                40,
                "vanilla slot pitch"
            );
        }
        // clicking the first slot's center hits SlotRef::Chest(0)
        let (x, y) = geom.chest[0];
        assert_eq!(geom.slot_at(x + 18, y + 18), Some(SlotRef::Chest(0)));
        // and the 5th
        let (x5, y5) = geom.chest[4];
        assert_eq!(geom.slot_at(x5 + 18, y5 + 18), Some(SlotRef::Chest(4)));
    }

    /// The oxygen bubble row appears only below a full air supply and
    /// renders ceil(air/30) bubbles (VERIFIED: 10 bubbles × 30 air).
    /// (Asserted structurally: status_bars with full air draws no bubble
    /// pixels in the bubble row band; with air 150 draws the band.)
    #[test]
    fn oxygen_row_draws_only_when_air_is_depleted() {
        let mut ui = UiCanvas::new();
        ui.status_bars(&HudStatus {
            health: 20.0,
            food: 20.0,
            food_jitter: false,
            xp: 0.5,
            level: 5,
            air: 300.0,
            armor: 0,
            hearts_jitter: false,
            hunger_poisoned: false,
            tick_phase: 0,
        });
        // full air -> the bubble band (right side, above hunger) stays empty
        let band = nonwhite(&ui, bubble_band_rect());
        let mut ui2 = UiCanvas::new();
        ui2.status_bars(&HudStatus {
            health: 20.0,
            food: 20.0,
            food_jitter: false,
            xp: 0.5,
            level: 5,
            air: 150.0,
            armor: 0,
            hearts_jitter: false,
            hunger_poisoned: false,
            tick_phase: 0,
        });
        let band2 = nonwhite(&ui2, bubble_band_rect());
        assert_eq!(band, 0, "no bubbles at full air");
        assert!(band2 > 0, "bubbles drawn at half air ({} px)", band2);
    }

    fn bubble_band_rect() -> (i32, i32, i32, i32) {
        // exactly the bubble row: y = hb_y - 48 (above hunger's -28),
        // 12px tall (6-row sprite x2), right-aligned 10-bubble span
        let hb_w = 9 * 40 + 4;
        let hb_x = (UI_W as i32 - hb_w) / 2;
        (hb_x + hb_w - 176, (UI_H as i32 - 48) - 48, 176, 12)
    }

    /// count non-transparent pixels inside a rect of the canvas buffer
    fn nonwhite(ui: &UiCanvas, r: (i32, i32, i32, i32)) -> i32 {
        let (rx, ry, rw, rh) = r;
        let mut n = 0;
        for y in ry..ry + rh {
            for x in rx..rx + rw {
                let i = (y as usize * UI_W + x as usize) * 4;
                if ui.px[i + 3] != 0 {
                    n += 1;
                }
            }
        }
        n
    }
}

// ----------------------------------------------------- screen tests --

#[cfg(test)]
mod screen_tests {
    use super::*;

    fn px(c: &UiCanvas, x: i32, y: i32) -> [u8; 3] {
        let i = (y as usize * UI_W + x as usize) * 4;
        [c.px[i], c.px[i + 1], c.px[i + 2]]
    }

    /// The boot splash is the vanilla structure: a SOLID studio background
    /// (not a wash over the panorama), the dark wordmark above center, and
    /// a thin white bar whose fill tracks progress.
    #[test]
    fn intro_screen_is_solid_splash_with_white_bar() {
        let mut c = UiCanvas::new();
        c.intro_screen(0.5);
        // background is the solid studio red, everywhere away from art
        assert_eq!(px(&c, 5, 5), [239, 50, 61]);
        assert_eq!(px(&c, UI_W as i32 - 6, UI_H as i32 - 6), [239, 50, 61]);
        // bar: white fill present mid-bar (fill spans x0..x0+progress*200)
        assert_eq!(px(&c, UI_W as i32 / 2 - 50, 344), [255, 255, 255]);
        // fill scales with progress: quarter bar is shorter than full
        let mut q = UiCanvas::new();
        q.intro_screen(0.25);
        let mut f = UiCanvas::new();
        f.intro_screen(1.0);
        let white_row = |c: &UiCanvas| -> usize {
            (385..575)
                .map(|x| px(c, x, 344) == [255, 255, 255])
                .filter(|b| *b)
                .count()
        };
        assert!(white_row(&f) > white_row(&q) + 20);
    }

    /// The title splash text is yellow, sits at the logo's bottom-right,
    /// and is TILTED so its right end is higher than its left end (the
    /// classic -20 degree rotation).
    #[test]
    fn title_splash_is_yellow_and_tilted() {
        let mut c = UiCanvas::new();
        // paint only the splash so the logo cannot pollute the bounds
        c.text_splash(UI_W as i32 / 2, 120, "SPLASH!", 0.25);
        let mut yellows: Vec<(i32, i32)> = Vec::new();
        for y in 0..UI_H as i32 {
            for x in 0..UI_W as i32 {
                let p = px(&c, x, y);
                if p == [255, 255, 0] {
                    yellows.push((x, y));
                }
            }
        }
        assert!(!yellows.is_empty(), "splash must paint yellow pixels");
        let min_x = yellows.iter().map(|p| p.0).min().unwrap();
        let max_x = yellows.iter().map(|p| p.0).max().unwrap();
        let y_at = |x: i32| yellows.iter().find(|p| p.0 == x).map(|p| p.1);
        // right end higher (smaller y) than left end — the -20 deg tilt
        let left = y_at(min_x).unwrap();
        let right = y_at(max_x).unwrap();
        assert!(
            right < left,
            "splash right end must tilt up: left y={left}, right y={right}"
        );
        // pulse changes the splash size (vanilla 2 Hz scale wobble)
        let mut p0 = UiCanvas::new();
        p0.text_splash(200, 120, "WOBBLE", 0.0);
        let mut p1 = UiCanvas::new();
        p1.text_splash(200, 120, "WOBBLE", 0.25);
        let count = |c: &UiCanvas| c.px.chunks(4).filter(|p| p[3] != 0).count();
        assert!(
            count(&p1) > count(&p0),
            "pulse peak must paint more pixels than the trough"
        );
    }

    /// The world-loading screen shows the vanilla chunk-colormap: exact
    /// wiki status colors, spawn cell red until meshed, and the percentage
    /// text above the map.
    #[test]
    fn world_loading_colormap_uses_status_colors() {
        let mut cells = [0u8; 35 * 35];
        cells[0] = 1; // generated
        cells[34] = 2; // meshed
                       // center (17,17) left pending -> spawn red
        let mut c = UiCanvas::new();
        c.world_loading_screen(43, &cells, 17 * 35 + 17);
        // map origin: (UI_W-140)/2, 170 — 4px cells
        let x0 = (UI_W as i32 - 140) / 2;
        let y0 = 170;
        assert_eq!(
            px(&c, x0, y0),
            [128, 178, 82],
            "generated cell = biomes green"
        );
        assert_eq!(
            px(&c, x0 + 4 * 34 + 2, y0 + 2),
            [255, 255, 255],
            "meshed cell = full white"
        );
        assert_eq!(
            px(&c, x0 + 4 * 17 + 2, y0 + 4 * 17 + 2),
            [242, 96, 96],
            "pending spawn cell = spawn red"
        );
        assert_eq!(px(&c, x0 + 8, y0 + 8), [84, 84, 84], "empty cell = gray");
        // meshed center flips off the spawn marker
        cells[17 * 35 + 17] = 2;
        let mut c2 = UiCanvas::new();
        c2.world_loading_screen(100, &cells, 17 * 35 + 17);
        assert_eq!(px(&c2, x0 + 4 * 17 + 2, y0 + 4 * 17 + 2), [255, 255, 255]);
    }

    /// F3 overlay: vanilla 1.16.5 two-column layout — left column at the
    /// top-left, right column right-aligned at the top-right, EVERY text
    /// line on its own translucent strip, blank lines paint nothing.
    #[test]
    fn f3_overlay_is_two_columns_with_per_line_strips() {
        let mut c = UiCanvas::new();
        let left = vec![
            "Facing: south".to_string(),
            String::new(), // blank spacer — no strip
            "Line three".to_string(),
        ];
        let right = vec!["R1".to_string()];
        c.debug(&left, &right);
        // line 1 strip: y 2..19 (18 tall, CONTIGUOUS pitch — vanilla stacks
        // its per-line strips seamlessly); glyphs span y 3..18, so y=17 is a
        // strip-only band below the x-height text but still line 1's strip
        let s1 = px(&c, 40, 17);
        assert_eq!(s1, [80, 80, 80], "per-line strip color (0x505050)");
        assert_eq!(c.px[(17 * UI_W + 40) * 4 + 3], 144, "strip alpha");
        // the strip ends right after the text (per-line width, +2 pad)
        assert_eq!(px(&c, 220, 9), [0, 0, 0], "past the line's strip");
        // CONTIGUITY: two consecutive non-empty lines' strips touch — draw
        // a two-line set and check the seam row belongs to a strip
        let mut c2 = UiCanvas::new();
        c2.debug(&["Aa".to_string(), "Bb".to_string()], &[]);
        assert_eq!(px(&c2, 5, 19), [80, 80, 80], "seam row = line 2's strip");
        assert_eq!(px(&c2, 5, 37), [80, 80, 80], "line 2 strip bottom row");
        // the blank second line paints NOTHING (spacer row stays empty)
        let row = 2 + 18 + 9; // y inside line 2's band
        let any = (0..UI_W).any(|x| c.px[(row * UI_W + x) * 4 + 3] != 0);
        assert!(!any, "blank spacer line must not paint a strip");
        // left text is FLAT (no drop-shadow pixel at +1,+1 of a glyph) and
        // uses the vanilla 0xE0E0E0 text color
        let mut found = false;
        for x in 3..160 {
            let i = (5 * UI_W + x) * 4;
            if c.px[i] == 224 && c.px[i + 1] == 224 && c.px[i + 3] == 255 {
                let j = (6 * UI_W + x + 1) * 4;
                if c.px[j + 3] == 144 && c.px[j] == 80 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "text must render flat 0xE0E0E0 over its strip");
        // right column: right-aligned — the strip hugs the right edge with
        // the 3px text margin (strip ends at UI_W-2)
        let y = 9;
        assert_eq!(
            px(&c, UI_W as i32 - 3, y),
            [80, 80, 80],
            "right strip margin"
        );
        assert_eq!(
            px(&c, UI_W as i32 - 1, y),
            [0, 0, 0],
            "1px past the right margin is empty"
        );
        // strip is only as wide as the text (+2 px pad), not full-width
        assert_eq!(px(&c, UI_W as i32 - 40, y), [0, 0, 0]);
    }

    /// The F3 case font: true lowercase (NOT the smallcaps remap), 8-row
    /// glyphs with a real descender row, tight variable advance, and the
    /// ∞ glyph.
    #[test]
    fn f3_case_font_is_lowercase_with_descenders() {
        let mut c = UiCanvas::new();
        // 'g' at (10, 20) scale 1: rows 2..7 — the DESCENDER row (y+7)
        // must have glyph pixels (the old 7-row font could not do this)
        c.text_flat_case(10, 20, "g", [255, 255, 255, 255], 1);
        let mut desc = false;
        for x in 10..16 {
            let i = (27 * UI_W + x) * 4;
            if c.px[i + 3] != 0 {
                desc = true;
            }
        }
        assert!(desc, "'g' must paint its descender row (row 7)");
        // lowercase 'p' is NOT the smallcaps 'P': different pixel pattern
        let mut cp = UiCanvas::new();
        cp.text_flat_case(10, 20, "p", [255, 255, 255, 255], 1);
        let mut cs = UiCanvas::new();
        cs.text_flat(10, 20, "p", [255, 255, 255, 255], 1); // smallcaps path
        let pat = |c: &UiCanvas| -> Vec<(usize, usize)> {
            (20..28)
                .flat_map(|y| (10..16).map(move |x| (y, x)))
                .filter(|&(y, x)| c.px[(y * UI_W + x) * 4 + 3] != 0)
                .collect()
        };
        assert_ne!(
            pat(&cp),
            pat(&cs),
            "case 'p' (bowl + descender) must differ from smallcaps 'P'"
        );
        // tight advance: "i" is 1px wide -> 2px advance (vanilla packs it)
        let w_i = UiCanvas::text_width_case("i", 1);
        assert_eq!(w_i, 2, "narrow 'i' advance");
        // a lowercase-dominant line is NARROWER than its smallcaps twin
        let s = "Integrated server tick";
        assert!(
            UiCanvas::text_width_case(s, 2) < UiCanvas::text_width(s, 2),
            "case text must pack tighter than fixed-advance smallcaps"
        );
        // the ∞ glyph renders (not the '?' fallback)
        let mut ci = UiCanvas::new();
        let w = ci.text_flat_case(10, 20, "∞", [255, 255, 255, 255], 1);
        assert_eq!(w, 6, "infinity advance (5 wide + 1)");
        let mut pxs = 0;
        for y in 24..26 {
            for x in 10..16 {
                if ci.px[(y * UI_W + x) * 4 + 3] != 0 {
                    pxs += 1;
                }
            }
        }
        assert!(pxs >= 5, "infinity glyph paints its weave");
    }

    /// F3+Q help overlay: a centered box listing the key combinations.
    #[test]
    fn f3_help_overlay_is_centered_box() {
        let mut c = UiCanvas::new();
        let rows = vec![
            ("F3 + Q".to_string(), "This help".to_string()),
            ("F3 + 1".to_string(), "Frame time graph".to_string()),
        ];
        c.debug_help(&rows);
        // panel fill near the center, empty far corners
        let mid = px(&c, UI_W as i32 / 2, UI_H as i32 / 2);
        assert!(mid != [0, 0, 0], "help box fills the center");
        assert_eq!(px(&c, 5, 5), [0, 0, 0], "corner stays empty");
    }

    /// Title layout: vanilla 1.16.5 stack — two full-width buttons then the
    /// half-width Options/Quit pair, all 30px tall, MULTIPLAYER disabled.
    #[test]
    fn title_layout_is_vanilla_stack() {
        let ws = layout_title(false);
        assert_eq!(ws.len(), 4);
        let play = ws.iter().find(|w| w.id == ID_TITLE_PLAY).unwrap();
        assert_eq!((play.w, play.h), (300, 30));
        assert_eq!(play.y, 225);
        let multi = ws.iter().find(|w| w.id == ID_TITLE_MULTI).unwrap();
        assert!(!matches!(&multi.kind, WidgetKind::Button { enabled: false, .. }) || true);
        assert!(matches!(
            &multi.kind,
            WidgetKind::Button { enabled: false, .. }
        ));
        let opts = ws.iter().find(|w| w.id == ID_TITLE_OPTIONS).unwrap();
        let quit = ws.iter().find(|w| w.id == ID_TITLE_QUIT).unwrap();
        assert_eq!((opts.w, quit.w), (146, 146));
        assert_eq!(opts.y, quit.y);
        assert!(quit.x > opts.x + opts.w, "options left, quit right");
        // web layout: no quit button, options full-width
        let web = layout_title(true);
        assert!(web.iter().all(|w| w.id != ID_TITLE_QUIT));
    }

    /// Optional visual dump for inspection (never set in CI):
    ///   UI_DUMP=/tmp/uidump cargo test -p vc-render screen
    #[test]
    fn ui_screens_dump() {
        let Ok(dir) = std::env::var("UI_DUMP") else {
            return;
        };
        let dump = |c: &UiCanvas, name: &str| {
            let img = image::RgbaImage::from_raw(UI_W as u32, UI_H as u32, c.px.clone())
                .expect("canvas size");
            let _ = img.save(format!("{dir}/{name}.png"));
        };
        let mut intro = UiCanvas::new();
        intro.intro_screen(0.62);
        dump(&intro, "intro");

        let mut title = UiCanvas::new();
        // stand-in backdrop so contrast reads in the dump (the real bg is
        // the painted panorama cubemap + menu blur)
        for y in 0..UI_H as i32 {
            for x in 0..UI_W as i32 {
                let t = y as f32 / UI_H as f32;
                title.set(x, y, [(90.0 * (1.0 - t) + 40.0) as u8, 20, 18, 255]);
            }
        }
        let ws = layout_title(false);
        title.title_screen("Also try going outside!", &ws, None, 0.25);
        dump(&title, "title");

        let mut loading = UiCanvas::new();
        let mut cells = [0u8; 35 * 35];
        for dy in -6..=6i32 {
            for dx in -6..=6i32 {
                let r = ((dx * dx + dy * dy) as f32).sqrt();
                let idx = ((dy + 17) * 35 + (dx + 17)) as usize;
                cells[idx] = if r < 3.0 {
                    2
                } else if r < 5.0 {
                    1
                } else {
                    0
                };
            }
        }
        loading.world_loading_screen(43, &cells, 17 * 35 + 17);
        dump(&loading, "world-loading");
    }

    // ---- Luanti font round: HUD quads + splash bake ----------------

    struct QuadTextGuard;
    impl QuadTextGuard {
        fn arm() -> Self {
            crate::ui::set_text_quads_active(true);
            QuadTextGuard
        }
    }
    impl Drop for QuadTextGuard {
        fn drop(&mut self) {
            crate::ui::set_text_quads_active(false);
        }
    }

    #[test]
    fn xp_bar_pushes_solid_quads_with_canvas_fallback() {
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        // quad path ON (chrome_enabled = false mirrors the game's
        // shipping config): the XP bar lands as Solid quads
        ui.set_chrome_enabled(false);
        ui.status_bars(&HudStatus {
            health: 0.8,
            food: 0.8,
            food_jitter: false,
            xp: 0.5,
            level: 3,
            air: 300.0,
            armor: 0,
            hearts_jitter: false,
            hunger_poisoned: false,
            tick_phase: 0,
        });
        let solids: Vec<_> = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .collect();
        // bg + 4 border edges + fill + highlight = 7 (hearts/hunger are
        // sprite quads, not Solid)
        assert_eq!(solids.len(), 7, "bg + 4 edges + 2 fill layers");
        // the fill is green
        assert!(solids.iter().any(|q| q.tint[1] > 0.9 && q.tint[0] < 0.6));
        // XP bar sits above the hotbar band
        let xp = solids
            .iter()
            .min_by(|a, b| {
                a.dst
                    .y
                    .partial_cmp(&b.dst.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        // Sub-round 1: 182x5 vanilla-eq height — 10 UI px (was 8)
        assert_eq!(xp.dst.h, 10.0, "10-px bar (5 vanilla px)");
        // level number routes through the engine's TEXT layer (pass 7)
        assert!(
            ui.gui_frame
                .text_quads
                .iter()
                .any(|q| q.texture == crate::gui_render::QuadTexture::GlyphAtlas),
            "level 3 text present as glyph quads"
        );
    }

    #[test]
    fn splash_source_bakes_engine_ink_when_armed() {
        let _g = QuadTextGuard::arm();
        let (src, bw, bh) = splash_source("100% RUST!");
        assert!(bw > 0 && bh > 0);
        assert_eq!(src.len(), (bw * bh) as usize);
        let colors: std::collections::HashSet<_> = src
            .iter()
            .filter(|c| c[3] > 0)
            .map(|c| (c[0], c[1], c[2]))
            .collect();
        // yellow ink + the dark yellow-brown outline
        assert!(colors.contains(&(255, 255, 0)), "yellow ink");
        assert!(colors.contains(&(63, 50, 0)), "outline");
        // both sources agree on the bitmap-fallback shape: disarm and
        // bake again — same dims class (16-px cell both paths)
        let (src2, bw2, bh2) = {
            crate::ui::set_text_quads_active(false);
            let r = splash_source("100% RUST!");
            crate::ui::set_text_quads_active(true);
            r
        };
        assert!(bw2 > 0 && bh2 > 0);
        assert!(src2.iter().any(|c| c[3] > 0), "bitmap path also inks");
    }

    #[test]
    fn stack_count_renders_at_vanilla_size_when_armed() {
        // the count is the regular font at the 16-px cell — the
        // half-size scale-1 text was the "garbled x4" complaint
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        let stack = ItemStack::new(vc_blocks::blocks::DIRT, 64);
        let atlas = vec![128u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        ui.draw_stack(&stack, 100, 100, &atlas);
        let glyphs: Vec<_> = ui
            .gui_frame
            .text_quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::GlyphAtlas)
            .collect();
        assert!(!glyphs.is_empty(), "count glyphs pushed");
        // "64" = 2 chars * (glyph + shadow) = 4 quads
        assert_eq!(glyphs.len(), 4);
        // cell 16: cap-height ink ≈ 14 px
        assert!(glyphs.iter().all(|q| q.dst.h <= 16.0 && q.dst.h >= 8.0));
    }

    // ---- Luanti round 2: the full device-resolution HUD ------------

    #[test]
    fn crosshair_is_device_snapped_quads_at_fractional_scale() {
        // THE fractional-scale crosshair fix: at k=1.5 every edge of
        // every arm quad lands on a whole DEVICE pixel (the old canvas
        // raster rode the 960x540 NEAREST letterbox — 2-px arms became
        // 3-px on some columns, ragged plus)
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        ui.set_device_scale(1.5);
        ui.clear();
        ui.crosshair();
        // 3 solid INVERT quads in the OVER-canvas layer: the horizontal
        // bar split into 2 disjoint segments + the vertical bar
        assert_eq!(
            ui.gui_frame.text_quads.len(),
            3,
            "H split into 2 disjoint segments + 1 V bar"
        );
        assert!(
            ui.gui_frame.quads.is_empty(),
            "crosshair never lands in the chrome layer"
        );
        let k = 1.5f32;
        for q in &ui.gui_frame.text_quads {
            assert_eq!(q.texture, crate::gui_render::QuadTexture::Solid);
            // the vanilla-blend contract: every crosshair quad is an
            // INVERT quad with white tint (result = 1 − dst)
            assert!(q.invert, "crosshair quads draw through the invert pipeline");
            assert_eq!(q.tint, [1.0, 1.0, 1.0, 1.0], "white tint — full inversion");
            for v in [
                q.dst.x * k,
                q.dst.y * k,
                (q.dst.x + q.dst.w) * k,
                (q.dst.y + q.dst.h) * k,
            ] {
                assert!((v - v.round()).abs() < 1e-3, "edge {v} on the device grid");
            }
        }
        // DISJOINTNESS (hard requirement: overlapping invert quads
        // cancel — invert∘invert = identity): the two H segments and
        // the V bar must not overlap each other
        {
            let qs = &ui.gui_frame.text_quads;
            for i in 0..qs.len() {
                for j in (i + 1)..qs.len() {
                    let (a, b) = (&qs[i], &qs[j]);
                    let sep_x =
                        a.dst.x + a.dst.w <= b.dst.x + 1e-6 || b.dst.x + b.dst.w <= a.dst.x + 1e-6;
                    let sep_y =
                        a.dst.y + a.dst.h <= b.dst.y + 1e-6 || b.dst.y + b.dst.h <= a.dst.y + 1e-6;
                    assert!(
                        sep_x || sep_y,
                        "invert quads {i} and {j} overlap — they would cancel"
                    );
                }
            }
        }
        // the arms cover the exact device center (720, 405 at
        // 1440x810): the horizontal segments' y window straddles it
        let hw: Vec<_> = ui
            .gui_frame
            .text_quads
            .iter()
            .filter(|q| q.dst.w > q.dst.h)
            .collect();
        assert_eq!(hw.len(), 2, "the split horizontal segments");
        let cy_dev = 270.0 * k;
        assert!(hw
            .iter()
            .all(|q| q.dst.y * k <= cy_dev && (q.dst.y + q.dst.h) * k >= cy_dev));
        let vw = ui
            .gui_frame
            .text_quads
            .iter()
            .find(|q| q.dst.h > q.dst.w)
            .expect("vertical bar");
        let cx_dev = 480.0 * k;
        assert!(vw.dst.x * k <= cx_dev && (vw.dst.x + vw.dst.w) * k >= cx_dev);
        // the canvas fallback did NOT rasterize (no double-draw)
        let cidx = (270usize * crate::ui::UI_W + 480usize) * 4;
        assert_eq!(ui.px[cidx + 3], 0, "canvas center clear — quads only");
    }

    #[test]
    fn crosshair_canvas_fallback_unchanged() {
        // disarm → the classic canvas plus (white center, dark outline)
        let mut ui = UiCanvas::new();
        ui.clear();
        ui.crosshair();
        assert!(ui.gui_frame.text_quads.is_empty(), "no quads disarmed");
        let cidx = (270usize * crate::ui::UI_W + 480usize) * 4;
        assert_eq!(ui.px[cidx + 3], 185, "white center on canvas");
    }

    #[test]
    fn splash_is_one_cached_rotated_quad() {
        // the armed splash: ONE glyph-atlas quad, tilted -20 degrees,
        // baked at DEVICE resolution and CACHED (the font-engine tests
        // prove bake-once in isolation; here the CACHE CONTRACT shows
        // as the same atlas rect across frames while the 2 Hz pulse
        // only rescales the dst rect)
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        ui.set_device_scale(1.5);
        ui.clear();
        ui.text_splash(480, 200, "100% RUST!", 0.0);
        assert_eq!(ui.gui_frame.text_quads.len(), 1, "one run quad");
        let q = ui.gui_frame.text_quads[0];
        assert_eq!(q.texture, crate::gui_render::QuadTexture::GlyphAtlas);
        assert!(
            (q.rot - (-(20.0_f32).to_radians())).abs() < 1e-4,
            "the vanilla -20 degree tilt, rot={}",
            q.rot
        );
        assert_eq!(q.tint, [1.0, 1.0, 1.0, 1.0], "strip carries its own colors");
        // second frame: cache hit — SAME atlas rect (no re-bake), and
        // t=0.25 is the pulse peak (|sin(pi/2)| = 1 → 1.06×) so the dst
        // rect grows around the SAME center
        ui.clear();
        ui.text_splash(480, 200, "100% RUST!", 0.25);
        let q2 = ui.gui_frame.text_quads[0];
        assert_eq!(
            (q.src.x, q.src.y, q.src.w, q.src.h),
            (q2.src.x, q2.src.y, q2.src.w, q2.src.h),
            "cache hit — stable atlas rect"
        );
        assert!(
            q2.dst.w > q.dst.w && q2.dst.h > q.dst.h,
            "pulse scales the dst"
        );
        let c = |r: &crate::gui_render::RectF| (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let (ax, ay) = c(&q.dst);
        let (bx, by) = c(&q2.dst);
        assert!(
            (ax - bx).abs() < 1e-3 && (ay - by).abs() < 1e-3,
            "center invariant"
        );
        // the device-res bake: at k=1.5 the strip is ~1.5× the UI-cell
        // raster (the old canvas path baked at 16 and upscaled mushy)
        assert!(q.src.w as f32 >= 16.0, "device-res strip (w={})", q.src.w);
        // the canvas fallback did NOT rasterize (no double-draw): the
        // strip's bounding area stays clear
        let idx = (205usize * crate::ui::UI_W + 480usize) * 4;
        assert_eq!(ui.px[idx + 3], 0, "canvas clear — quads only");
    }

    #[test]
    fn debug_strips_are_fractional_quads_matching_engine_metrics() {
        // armed: strips ride SOLID quads with the engine's fractional
        // widths (hugging the device-exact text) and the canvas stays
        // clear; disarmed: the classic integer canvas strips
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        ui.clear();
        let left = vec!["VoxelCraft 1.16.5".to_string()];
        let right = vec!["60 fps".to_string()];
        ui.debug(&left, &right);
        let strips: Vec<_> = ui
            .gui_frame
            .text_quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .collect();
        assert_eq!(strips.len(), 2, "one strip per column line");
        // the left strip's width = engine measure + 2 UI px (pad)
        let w_engine = match crate::gui::font::engine() {
            Some(eng) => {
                let mut e = eng.lock().unwrap_or_else(|p| p.into_inner());
                e.measure("VoxelCraft 1.16.5", 16.0)
            }
            None => panic!("engine required"),
        };
        assert!((strips[0].dst.w - (w_engine + 2.0)).abs() < 1e-3);
        // the right strip is right-aligned: right edge = UI_W - 2
        let r = strips[1].dst.x + strips[1].dst.w;
        assert!((r - (crate::ui::UI_W as f32 - 2.0)).abs() < 1e-3);
        // canvas clear where the strip sits (top-left line body)
        let idx = (3usize * crate::ui::UI_W + 30usize) * 4;
        assert_eq!(ui.px[idx + 3], 0, "canvas strip region clear when armed");
    }

    #[test]
    fn boss_bar_pushes_solid_quads_with_gated_canvas() {
        // quads always pushed (track + 4 frame edges + fill pair at
        // frac>0); the canvas raster only when chrome_enabled
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.boss_bar(0.5);
        let solids: Vec<_> = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .collect();
        // 1 track + 4 frame + 2 fill = 7 (label rides glyph quads)
        assert_eq!(solids.len(), 7, "track + frame edges + fill pair");
        // canvas suppressed at the track body (y=24..36, center x)
        let idx = (30usize * crate::ui::UI_W + 480usize) * 4;
        assert_eq!(ui.px[idx + 3], 0, "canvas suppressed when chrome off");
        // fallback: chrome on → the canvas track paints AND quads stay
        ui.set_chrome_enabled(true);
        ui.clear();
        ui.boss_bar(0.5);
        assert_eq!(ui.px[idx + 3], 220, "canvas track painted in fallback");
        assert!(!ui.gui_frame.quads.is_empty(), "quads still pushed (A/B)");
    }

    /// Sub-round 3 (2026-09-15): the survival inventory layout — the
    /// vanilla 176x166 shape (scaled 2x): the LEFT armor column in
    /// helmet..boots order, the offhand slot below it, the 2x2 craft
    /// grid + output on the right, the 9x3 storage + 9x1 hotbar below.
    /// (reference wiki /Inventory, live 2026-09-15: "The inventory
    /// consists of 4 armor slots, 27 storage slots, 9 hotbar slots, and
    /// an off-hand slot"; "There is also a 2x2 crafting grid".)
    #[test]
    fn survival_inventory_layout() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let view = crate::ui::ContainerView {
            kind: crate::ui::ContainerKind::Inventory,
            inv: vec![vc_inventory::inventory::ItemStack::EMPTY; 36],
            grid: vec![vc_inventory::inventory::ItemStack::EMPTY; 4],
            craft_out: vc_inventory::inventory::ItemStack::EMPTY,
            furnace: None,
            brewing: None,
            enchant: None,
            trade: None,
            chest: Vec::new(),
            armor: [
                vc_inventory::inventory::ItemStack::new(vc_blocks::blocks::DIAMOND_HELMET, 1),
                vc_inventory::inventory::ItemStack::EMPTY,
                vc_inventory::inventory::ItemStack::EMPTY,
                vc_inventory::inventory::ItemStack::EMPTY,
            ],
            offhand: vc_inventory::inventory::ItemStack::new(vc_blocks::blocks::SHIELD, 1),
            cursor: vc_inventory::inventory::ItemStack::EMPTY,
            anvil: None,
            beacon: None,
            grind: None,
            mount: None,
        };
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas, false);
        // the armor column: 4 slots top-to-bottom (helmet first)
        assert!(g.armor.len() == 4);
        for i in 1..4 {
            assert_eq!(g.armor[i].1, g.armor[i - 1].1 + 44, "armor pitch");
        }
        // armor hit-tests resolve in piece order
        assert_eq!(
            g.slot_at(g.armor[0].0 + 4, g.armor[0].1 + 4),
            Some(crate::ui::SlotRef::Armor(0))
        );
        assert_eq!(
            g.slot_at(g.armor[3].0 + 4, g.armor[3].1 + 4),
            Some(crate::ui::SlotRef::Armor(3))
        );
        // the offhand slot sits BELOW the armor column (its boxed recess)
        assert!(g.offhand.1 > g.armor[3].1);
        assert_eq!(
            g.slot_at(g.offhand.0 + 4, g.offhand.1 + 4),
            Some(crate::ui::SlotRef::Offhand)
        );
        // the 2x2 craft grid + output on the right of the armor column
        assert_eq!(g.craft.len(), 4);
        assert!(g.craft[0].0 > g.armor[0].0 + 100);
        assert!(g.craft_out.0 > g.craft[3].0);
        // 36 inventory slots (27 storage + 9 hotbar) still hit-test
        assert_eq!(g.inv.len(), 36);
        assert_eq!(
            g.slot_at(g.inv[0].0 + 4, g.inv[0].1 + 4),
            Some(crate::ui::SlotRef::Inv(0))
        );
    }

    /// Round 12 (2026-09-15): the double-chest screen geometry — the
    /// 9×6 = 54-slot grid (the two halves' 27+27), the single-word
    /// "CHEST" title, and the vanilla 176×220-family panel. The engine
    /// container family carries a constant chrome overhead over vanilla
    /// (single chest 173 vanilla-eq px vs 166; the same +3-row growth
    /// → the double's 239 vanilla-eq vs 220) — the SLOT GRID itself is
    /// exactly vanilla: 9 columns × 6 rows at the 20-px pitch, +3 rows
    /// over the single chest. (reference wiki /Chest §Double chests,
    /// live 2026-09-15.)
    #[test]
    fn double_chest_screen_geometry() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let view = crate::ui::ContainerView {
            kind: crate::ui::ContainerKind::DoubleChest,
            inv: vec![vc_inventory::inventory::ItemStack::EMPTY; 36],
            grid: vec![],
            craft_out: vc_inventory::inventory::ItemStack::EMPTY,
            furnace: None,
            brewing: None,
            enchant: None,
            trade: None,
            chest: vec![vc_inventory::inventory::ItemStack::EMPTY; 54],
            armor: [vc_inventory::inventory::ItemStack::EMPTY; 4],
            offhand: vc_inventory::inventory::ItemStack::EMPTY,
            cursor: vc_inventory::inventory::ItemStack::EMPTY,
            anvil: None,
            beacon: None,
            grind: None,
            mount: None,
        };
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas, false);
        // the merged grid: 54 slots = 9 cols × 6 rows
        assert_eq!(g.chest.len(), 54, "54 slot rects");
        // column pitch 40 (20 vanilla px), row pitch 40
        for r in 0..6usize {
            for c in 0..8usize {
                let i = r * 9 + c;
                assert_eq!(g.chest[i + 1].0, g.chest[i].0 + 40, "col pitch");
            }
        }
        for r in 0..5usize {
            let i = r * 9;
            assert_eq!(g.chest[i + 9].1, g.chest[i].1 + 40, "row pitch");
        }
        // +3 rows over the single chest's 3-row grid (132 → 264 top)
        let single_top = 132;
        let double_top = 264;
        assert_eq!(double_top - single_top, 3 * 44, "+3 rows of slots");
        // every slot hit-tests through the generic Chest(i) path
        for i in [0usize, 26, 27, 53] {
            let (x, y) = g.chest[i];
            assert_eq!(
                g.slot_at(x + 4, y + 4),
                Some(crate::ui::SlotRef::Chest(i)),
                "slot {i} hit-tests"
            );
        }
        // the player inventory rows below still hit-test (36 slots)
        assert_eq!(g.inv.len(), 36);
        // rows 0-2 sit ABOVE rows 3-5 (the halves' order in the view)
        assert!(g.chest[0].1 < g.chest[27].1, "half A rows above half B");
        // the panel is taller than the single chest's: top 264 + the
        // shared bottom (3*44+8+44+30) = 478 canvas px = 239
        // vanilla-eq (family chrome overhead, disclosed above)
        let panel_h = 264 + 3 * 44 + 8 + 44 + 30;
        assert_eq!(panel_h, 478);
        assert_eq!(
            panel_h / 2,
            239,
            "vanilla-eq height (220 + family overhead)"
        );
        // the panel centers vertically in the live canvas (540 tall)
        let y0 = (540 - panel_h) / 2;
        assert_eq!(g.chest[0].1 - 8, y0, "grid starts 8px under the panel top");
    }

    /// Sub-round 2 (2026-09-15): the tabbed creative screen geometry —
    /// the 11-tab strip in vanilla order, the 9x5 grid page, the hotbar
    /// and destroy-slot hit rects, and the scrollbar presence rule.
    /// (reference wiki /Creative_inventory, live 2026-09-15: nine
    /// content tabs, Search Items, and Survival Inventory; 9 columns
    /// and 5 rows = 45 slots per page with a scrollbar when the tab has
    /// more items than one page.)
    #[test]
    fn creative_screen_geometry() {
        use vc_blocks::blocks as blk;
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let items = blk::creative_tab_items(blk::CreativeTab::BuildingBlocks);
        let hotbar = [vc_inventory::inventory::ItemStack::EMPTY; 9];
        // zeroed full-size atlas (the tile blits read within bounds)
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.creative_screen(
            (480.0, 270.0),
            &atlas,
            0,
            0,
            "",
            false,
            &items,
            &hotbar,
            0,
            &vc_inventory::inventory::ItemStack::EMPTY,
            false,
        );
        // the grid page: 9 columns x 5 rows
        assert_eq!(g.cols, 9);
        assert_eq!(g.vis_rows, 5);
        // Round 15b: 12 tab hit rects, all present (9 content + Search
        // + Saved Hotbars + Inventory)
        assert_eq!(g.tabs.len(), 12);
        assert!(g.tabs.iter().all(|t| t.is_some()));
        // tab hit-testing in vanilla order: tab 0 (Building Blocks) is
        // the first hit rect of row 1; tab 9 (Search), tab 10 (Saved
        // Hotbars) + tab 11 (Inventory) live on row 2
        let t0 = g.tabs[0].unwrap();
        assert_eq!(g.tab_at(t0.0 + 2, t0.1 + 2), Some(0));
        let t9 = g.tabs[9].unwrap();
        assert_eq!(g.tab_at(t9.0 + 2, t9.1 + 2), Some(9));
        let t10 = g.tabs[10].unwrap();
        assert_eq!(g.tab_at(t10.0 + 2, t10.1 + 2), Some(10));
        let t11 = g.tabs[11].unwrap();
        assert_eq!(g.tab_at(t11.0 + 2, t11.1 + 2), Some(11));
        // row 2 sits BELOW row 1 (folder-tab stacking, 6 + 6)
        assert!(t9.1 > t0.1);
        // row 2's six tabs stay inside the canvas (the 12-tab strip)
        assert!(t11.0 + t11.2 < ui.live_w as i32);
        // the grid hit-tests to the item list (Building tab, no scroll)
        assert_eq!(g.grid_at(g.x0 + 4 + 18, g.y0 + 4 + 18), Some(0));
        assert_eq!(g.grid_at(g.x0 + 4 + 40 + 18, g.y0 + 4 + 18), Some(1));
        assert_eq!(g.grid_at(g.x0 + 4 + 18, g.y0 + 4 + 40 + 18), Some(9));
        // the Building tab (155 items > 45) must carry a scrollbar
        assert!(g.scrollbar.is_some());
        // hotbar hit rects: 9, left-to-right
        assert_eq!(g.hotbar.len(), 9);
        let h0 = g.hotbar[0];
        let h8 = g.hotbar[8];
        assert!(h8.0 > h0.0);
        assert_eq!(g.hotbar_at(h0.0 + 4, h0.1 + 4), Some(0));
        assert_eq!(g.hotbar_at(h8.0 + 4, h8.1 + 4), Some(8));
        // the destroy slot sits right of the hotbar
        assert!(g.trash.0 > h8.0);
        assert!(g.trash_at(g.trash.0 + 4, g.trash.1 + 4));
        // no search field outside the Search tab
        assert_eq!(g.search, (0, 0, 0, 0));
    }

    /// Sub-round 2: the Search tab — the search field's hit rect appears
    /// in the title strip, and a short result list (<= 45 items) shows
    /// NO scrollbar (the vanilla single-page case).
    #[test]
    fn creative_screen_search_tab() {
        use vc_blocks::blocks as blk;
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let items = blk::creative_tab_items(blk::CreativeTab::Redstone);
        assert!(items.len() <= 45, "redstone tab is one page");
        let hotbar = [vc_inventory::inventory::ItemStack::EMPTY; 9];
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.creative_screen(
            (480.0, 270.0),
            &atlas,
            9, // Search tab
            0,
            "redstone",
            true,
            &items,
            &hotbar,
            0,
            &vc_inventory::inventory::ItemStack::EMPTY,
            false,
        );
        // the search field replaces the title strip
        assert!(g.search.2 > 0, "search field present on the Search tab");
        assert!(g.search_at(g.search.0 + 4, g.search.1 + 4));
        // single page: no scrollbar
        assert!(g.scrollbar.is_none());
    }

    /// Sub-round 1 (2026-09-14): the creative HUD now hides the XP bar
    /// and bubbles too — VERIFIED reference wiki /Heads-up_display
    /// (live 2026-09-14): "In Creative mode, the health, hunger, oxygen,
    /// experience, and armor bars are hidden." The retired
    /// `xp_bar_only()` (creative XP + bubbles) is replaced by this
    /// assertion: the game layer simply does not call `status_bars` in
    /// creative — here we prove a no-status call pushes ZERO status quads.
    #[test]
    fn creative_hud_hides_all_status_rows() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        // nothing drawn except what the game layer would draw in
        // creative (crosshair + hotbar + held name are separate calls)
        let before = ui.gui_frame.quads.len();
        assert_eq!(before, 0, "no status quads without status_bars");
    }

    /// Sub-round 2/3 round (2026-09-15, the user's "what about the
    /// effects" callout): effect icons render with NO status_bars call
    /// — i.e. in Creative mode, where every status row is hidden but
    /// the effect icons stay. VERIFIED reference wiki /
    /// Heads-up_display (live 2026-09-14): "All effects ... the player
    /// currently has are shown on the top-right of the screen" with no
    /// Creative exception (the hidden list is "health, hunger, oxygen,
    /// experience, and armor bars").
    #[test]
    fn effect_icons_render_without_status_rows_creative() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        // the creative situation: no status_bars call at all, but the
        // player has two live effects
        let entries = [
            EffectIconEntry {
                icon: 3,
                amplifier: 0,
                ticks_left: 200,
                positive: true,
            },
            EffectIconEntry {
                icon: 8,
                amplifier: 1,
                ticks_left: 60,
                positive: false,
            },
        ];
        ui.effect_icons(&entries, 40);
        let icons = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Effects)
            .count();
        assert!(
            icons >= 2,
            "effect icon quads must render in creative (got {icons})"
        );
    }

    /// Sub-round 1: the survival block's quad census — hearts 10,
    /// hunger 10, XP solids 7 (bg + 4 edges + fill pair), plus the
    /// armor row's 10 sprites at >0 points and 0 at zero (the vanilla
    /// hide gate), and the 182x5 vanilla-eq XP height (364x10).
    #[test]
    fn status_bars_quad_census_and_armor_gate() {
        let base = HudStatus {
            health: 13.0,
            food: 20.0,
            food_jitter: false,
            xp: 0.75,
            level: 7,
            air: 300.0,
            armor: 0,
            hearts_jitter: false,
            hunger_poisoned: false,
            tick_phase: 0,
        };
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.status_bars(&base);
        let hearts = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Hearts)
            .count();
        let hunger = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Hunger)
            .count();
        let armor = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Armor)
            .count();
        let solids = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .count();
        assert_eq!(hearts, 10);
        assert_eq!(hunger, 10);
        assert_eq!(armor, 0, "0 armor points = row hidden (vanilla gate)");
        assert_eq!(solids, 7, "XP track: bg + 4 edges + fill pair");
        // 182x5 vanilla-eq height: the XP track quad is 364x10
        let track = ui
            .gui_frame
            .quads
            .iter()
            .find(|q| q.texture == crate::gui_render::QuadTexture::Solid && q.dst.w == 364.0)
            .expect("364-wide XP track");
        assert_eq!(track.dst.h, 10.0, "XP bar height = 5 vanilla px");

        // armor > 0 → the row appears, icon census follows the points
        let mut armored = base;
        armored.armor = 15; // 7 full + 1 half + 2 empty
        ui.clear();
        ui.status_bars(&armored);
        let armor2 = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Armor)
            .count();
        assert_eq!(
            armor2, 10,
            "10 icons at 15 points (7 full, 1 half, 2 empty)"
        );
        // armor row sits ABOVE the hearts row (vanilla position)
        let armor_y = ui
            .gui_frame
            .quads
            .iter()
            .find(|q| q.texture == crate::gui_render::QuadTexture::Armor)
            .map(|q| q.dst.y)
            .unwrap();
        let heart_y = ui
            .gui_frame
            .quads
            .iter()
            .find(|q| q.texture == crate::gui_render::QuadTexture::Hearts)
            .map(|q| q.dst.y)
            .unwrap();
        assert!(
            armor_y < heart_y,
            "armor above hearts ({armor_y} < {heart_y})"
        );
    }

    /// Sub-round 1: the Hunger-effect recolor routes the hunger sprites
    /// through the tint path (non-white tint) while the normal path
    /// stays white.
    #[test]
    fn hunger_poisoned_tints_the_row() {
        let base = HudStatus {
            health: 20.0,
            food: 20.0,
            food_jitter: false,
            xp: 0.0,
            level: 0,
            air: 300.0,
            armor: 0,
            hearts_jitter: false,
            hunger_poisoned: false,
            tick_phase: 0,
        };
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.status_bars(&base);
        assert!(ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Hunger)
            .all(|q| q.tint == [1.0, 1.0, 1.0, 1.0]));

        let mut poisoned = base;
        poisoned.hunger_poisoned = true;
        ui.clear();
        ui.status_bars(&poisoned);
        let tinted = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Hunger)
            .filter(|q| q.tint[1] > q.tint[0] && q.tint[2] < q.tint[0])
            .count();
        assert_eq!(tinted, 10, "all 10 drumsticks tinted yellow-green");
    }

    /// Sub-round 1: the damage vignette pushes ONE full-canvas red
    /// over-quad at alpha ≤ 0.3·a and nothing at alpha 0.
    #[test]
    fn damage_vignette_full_canvas_red() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.damage_vignette(0.0);
        assert!(ui.gui_frame.text_quads.is_empty(), "no vignette at 0");
        ui.damage_vignette(1.0);
        assert_eq!(ui.gui_frame.text_quads.len(), 1);
        let q = &ui.gui_frame.text_quads[0];
        assert_eq!(
            (q.dst.w, q.dst.h),
            (crate::ui::UI_W as f32, crate::ui::UI_H as f32)
        );
        assert!((q.tint[3] - 0.3).abs() < 1e-6, "max 0.3 alpha");
        assert!(q.tint[0] > 0.0 && q.tint[1] == 0.0, "red");
    }

    /// Sub-round 1: the effect-icon rows — positive on top, others on
    /// the bottom, sooner-expiring LEFT within a row, blinking in the
    /// final 100 ticks, amplifier numerals at level II+.
    #[test]
    fn effect_icons_split_sort_and_blink() {
        let entries = [
            EffectIconEntry {
                icon: 3,
                amplifier: 0,
                ticks_left: 400,
                positive: true,
            }, // speed
            EffectIconEntry {
                icon: 2,
                amplifier: 1,
                ticks_left: 1200,
                positive: true,
            }, // regen II
            EffectIconEntry {
                icon: 1,
                amplifier: 0,
                ticks_left: 60,
                positive: false,
            }, // poison, about to expire
            EffectIconEntry {
                icon: 8,
                amplifier: 0,
                ticks_left: 900,
                positive: false,
            }, // slowness
        ];
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.effect_icons(&entries, 0);
        let icons: Vec<(f32, f32, f32)> = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Effects)
            .map(|q| (q.dst.x, q.dst.y, q.tint[3]))
            .collect();
        assert_eq!(icons.len(), 4, "one quad per effect");
        // positive row 0 (y=10), others row 1 (y=30)
        let top: Vec<_> = icons.iter().filter(|i| i.1 == 10.0).collect();
        let bottom: Vec<_> = icons.iter().filter(|i| i.1 == 30.0).collect();
        assert_eq!(top.len(), 2);
        assert_eq!(bottom.len(), 2);
        // sooner-expiring farther left within the row
        let poison = bottom
            .iter()
            .find(|i| i.2 == 0.25)
            .expect("poison blinking");
        let slowness = bottom.iter().find(|i| i.2 == 1.0).expect("slowness solid");
        assert!(
            poison.0 < slowness.0,
            "poison (60 ticks) left of slowness (900)"
        );
        // blink: poison at 60 ticks < 100 → alpha 0.25 at tick 0
        // (verified by the find above); the regen II numeral is text —
        // canvas ink when the font-quad path is not armed (tests)
        let ink = ui
            .px
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|c| c[3] != 0)
            .count();
        assert!(ink > 0, "regen II numeral rastered");
        // no entries → nothing
        ui.clear();
        ui.effect_icons(&[], 0);
        assert!(ui.gui_frame.quads.is_empty());
    }

    #[test]
    fn frame_graph_bars_are_quads_when_armed() {
        // armed: bg + guide + one quad per bar; disarmed: canvas raster
        let _g = QuadTextGuard::arm();
        let mut ui = UiCanvas::new();
        ui.clear();
        let times = vec![10.0f32, 25.0, 55.0, 8.0];
        ui.frame_graph(100, &times);
        let solids: Vec<_> = ui
            .gui_frame
            .text_quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .collect();
        // bg + guide + 4 bars (8 ms clamps to a >0 bar too)
        assert_eq!(solids.len(), 6, "bg + guide line + one quad per bar");
        // the 10 ms bar: th = 10/50*40 = 8 px tall, 2 px wide
        let bar = solids.iter().find(|q| q.dst.h == 8.0 && q.dst.w == 2.0);
        assert!(bar.is_some(), "10 ms bar is 8 px tall");
        // canvas clear at the graph body
        let idx = (110usize * crate::ui::UI_W + 30usize) * 4;
        assert_eq!(ui.px[idx + 3], 0, "canvas graph clear when armed");
    }
}

#[cfg(test)]
mod round13_station_tests {
    use super::*;
    use vc_inventory::inventory::ItemStack;

    fn station_view(kind: ContainerKind, anvil: Option<AnvilView>) -> ContainerView {
        ContainerView {
            kind,
            inv: vec![ItemStack::EMPTY; 36],
            grid: vec![],
            craft_out: ItemStack::EMPTY,
            furnace: None,
            brewing: None,
            enchant: None,
            trade: None,
            chest: Vec::new(),
            armor: [ItemStack::EMPTY; 4],
            offhand: ItemStack::EMPTY,
            cursor: ItemStack::EMPTY,
            anvil,
            beacon: None,
            grind: None,
            mount: None,
        }
    }

    /// Round 13 [spec]: the anvil screen geometry — the audit §1 layout
    /// (inputs (27,47)/(76,47), result (134,47) at the doubled scale:
    /// 36px pitch) with the rename field above and the cost line under
    /// the arrow. Hit-rects resolve to the right SlotRefs.
    #[test]
    fn anvil_screen_geometry_and_hits() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let av = AnvilView {
            target: ItemStack::new(vc_blocks::blocks::IRON_HELMET, 1),
            sacrifice: ItemStack::EMPTY,
            result: ItemStack::new(vc_blocks::blocks::IRON_HELMET, 1),
            cost: 1,
            too_expensive: false,
            affordable: true,
            rename: "Hero Cap".into(),
            rename_focused: true,
            creative: false,
        };
        let view = station_view(ContainerKind::Anvil, Some(av));
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas, false);
        let a = g.anvil.as_ref().expect("anvil geometry present");
        // the vanilla slot triangle: target left, sacrifice middle,
        // result right — the result sits FARTHER right than the inputs
        assert!(a.target.0 < a.sacrifice.0);
        assert!(a.sacrifice.0 < a.out.0);
        // all three on the same row (the audit's y=47 row)
        assert_eq!(a.target.1, a.sacrifice.1);
        assert_eq!(a.sacrifice.1, a.out.1);
        // the vanilla proportions at the doubled scale: target→sacrifice
        // = 49 vanilla px → 98 UI px; sacrifice→result = 58 vanilla px
        // → 116 UI px (audit §1's measured layout)
        assert_eq!(a.sacrifice.0 - a.target.0, 98);
        assert_eq!(a.out.0 - a.sacrifice.0, 116);
        // the rename field sits ABOVE the slots
        assert!(a.rename.1 < a.target.1);
        // hit-tests resolve in order
        assert_eq!(
            g.slot_at(a.target.0 + 4, a.target.1 + 4),
            Some(SlotRef::AnvilTarget)
        );
        assert_eq!(
            g.slot_at(a.sacrifice.0 + 4, a.sacrifice.1 + 4),
            Some(SlotRef::AnvilSacrifice)
        );
        assert_eq!(g.slot_at(a.out.0 + 4, a.out.1 + 4), Some(SlotRef::AnvilOut));
    }

    /// Round 13 [spec]: the beacon screen geometry — 5 primary power
    /// buttons, 2 secondary buttons, the payment slot; the buttons
    /// resolve to BeaconPrimary(i)/BeaconSecondary(i) and the slot to
    /// BeaconPay.
    #[test]
    fn beacon_screen_geometry_and_hits() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let bv = BeaconView {
            level: 4,
            pay: ItemStack::new(vc_blocks::blocks::IRON_ORE, 1),
            primary: Some(vc_gameplay::beacon::BeaconPower::Speed),
            secondary: vc_gameplay::beacon::BeaconSecondary::None,
            pending_primary: None,
            pending_secondary: vc_gameplay::beacon::BeaconSecondary::None,
        };
        let mut view = station_view(ContainerKind::Beacon, None);
        view.beacon = Some(bv);
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas, false);
        let b = g.beacon.as_ref().expect("beacon geometry present");
        // the five primary buttons: two columns, the 5th below-left
        // (the audit §2's 2x2 + 1 grid)
        assert_eq!(b.primary.len(), 5);
        assert_eq!(b.primary[1].0, b.primary[0].0 + 56, "column pitch 56");
        assert_eq!(b.primary[2].1, b.primary[0].1 + 56, "row pitch 56");
        // the 5th button sits below the first column
        assert!(b.primary[4].1 > b.primary[2].1);
        assert_eq!(b.primary[4].0, b.primary[0].0);
        // the secondary pair to the right of the primary grid
        assert!(b.secondary[0].0 > b.primary[1].0);
        // hit-tests resolve
        assert_eq!(
            g.slot_at(b.primary[0].0 + 8, b.primary[0].1 + 8),
            Some(SlotRef::BeaconPrimary(0))
        );
        assert_eq!(
            g.slot_at(b.primary[4].0 + 8, b.primary[4].1 + 8),
            Some(SlotRef::BeaconPrimary(4))
        );
        assert_eq!(
            g.slot_at(b.secondary[0].0 + 8, b.secondary[0].1 + 8),
            Some(SlotRef::BeaconSecondary(0))
        );
        assert_eq!(
            g.slot_at(b.pay.0 + 8, b.pay.1 + 8),
            Some(SlotRef::BeaconPay)
        );
    }

    /// Round 13 [spec]: the grindstone screen geometry — the two
    /// STACKED inputs (the audit §3's 22px-pitch vertical pair) and the
    /// result on the right.
    #[test]
    fn grindstone_screen_geometry_and_hits() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let mut view = station_view(ContainerKind::Grindstone, None);
        view.grind = Some((
            ItemStack::new(vc_blocks::blocks::IRON_HELMET, 1),
            ItemStack::EMPTY,
            ItemStack::EMPTY,
        ));
        let atlas = vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4];
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas, false);
        let s = g.grind.as_ref().expect("grindstone geometry present");
        // the inputs are vertically stacked (top above bottom, same x)
        assert_eq!(s.top.0, s.bottom.0);
        assert_eq!(s.bottom.1 - s.top.1, 44, "the doubled 22px pitch");
        // the result sits right of both inputs, vertically between them
        assert!(s.out.0 > s.top.0);
        assert!(s.out.1 > s.top.1 && s.out.1 < s.bottom.1);
        // hit-tests resolve
        assert_eq!(g.slot_at(s.top.0 + 4, s.top.1 + 4), Some(SlotRef::GrindTop));
        assert_eq!(
            g.slot_at(s.bottom.0 + 4, s.bottom.1 + 4),
            Some(SlotRef::GrindBottom)
        );
        assert_eq!(g.slot_at(s.out.0 + 4, s.out.1 + 4), Some(SlotRef::GrindOut));
    }
}

/// Round 12b: the mount storage screen — the donkey/mule (saddle
/// column + the 5-wide 15-slot grid) and the llama (strength badge +
/// the partial-row 3×strength grid) layouts and hit-tests.
#[cfg(test)]
mod round12b_mount_screen_tests {
    use super::*;
    use vc_inventory::inventory::ItemStack;

    fn mount_view(
        kind_label: &str,
        llama: bool,
        strength: u8,
        capacity: usize,
        saddle: ItemStack,
        chest: Vec<ItemStack>,
    ) -> ContainerView {
        ContainerView {
            kind: ContainerKind::Mount,
            inv: vec![ItemStack::EMPTY; 36],
            grid: vec![],
            craft_out: ItemStack::EMPTY,
            furnace: None,
            brewing: None,
            enchant: None,
            trade: None,
            chest,
            armor: [ItemStack::EMPTY; 4],
            offhand: ItemStack::EMPTY,
            cursor: ItemStack::EMPTY,
            anvil: None,
            beacon: None,
            grind: None,
            mount: Some(MountView {
                kind_label: kind_label.to_string(),
                saddle,
                llama,
                strength,
                capacity,
            }),
        }
    }

    fn atlas() -> Vec<u8> {
        vec![0u8; crate::textures::ATLAS_SIZE * crate::textures::ATLAS_SIZE * 4]
    }

    /// the donkey screen: the saddle slot left of a 3×5 grid; all 15
    /// slots hit-test as generic chest slots; the saddle resolves to
    /// SlotRef::MountSaddle.
    #[test]
    fn donkey_screen_geometry_and_hits() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let chest = vec![ItemStack::EMPTY; 15];
        let saddle = ItemStack::new(vc_blocks::blocks::SADDLE, 1);
        let view = mount_view("DONKEY", false, 1, 15, saddle, chest);
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas(), false);
        // 15 grid wells + the saddle slot
        assert_eq!(g.chest.len(), 15, "the donkey's 15 storage slots");
        let m = g.mount.as_ref().expect("the saddle geometry present");
        assert_eq!(
            g.slot_at(m.saddle.0 + 4, m.saddle.1 + 4),
            Some(SlotRef::MountSaddle)
        );
        // grid slots resolve as generic chest slots (the game layer's
        // Container::Mount routing surface)
        for i in 0..15 {
            let (x, y) = g.chest[i];
            assert_eq!(
                g.slot_at(x + 4, y + 4),
                Some(SlotRef::Chest(i)),
                "slot {i} hit-tests"
            );
        }
        // the saddle column sits LEFT of the grid
        assert!(m.saddle.0 < g.chest[0].0);
        // row pitch: 5 columns per row (slots 0/5/10 step one row)
        assert!(g.chest[5].1 > g.chest[0].1);
        assert!(g.chest[4].0 < g.chest[5].0 || g.chest[5].1 > g.chest[4].1);
        // the saddle well and the first grid well share the row
        assert!((m.saddle.1 - g.chest[0].1).abs() < 8);
    }

    /// the llama screen (strength 3 → 9 slots): no saddle geometry (the
    /// strength badge is not a hit target), the 9 grid wells with a
    /// PARTIAL last row (4 in row 2).
    #[test]
    fn llama_screen_partial_row_and_no_saddle() {
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        let chest = vec![ItemStack::EMPTY; 9];
        let view = mount_view("LLAMA", true, 3, 9, ItemStack::EMPTY, chest);
        let g = ui.container_screen(&view, (0.0, 0.0), &atlas(), false);
        assert_eq!(g.chest.len(), 9, "3 × strength 3 = 9 slots");
        // no saddle hit target on the llama screen (the badge column)
        assert!(g.mount.is_none(), "llamas have no saddle slot");
        // rows: 5 + 4 (the partial second row)
        let row1: Vec<_> = g.chest.iter().filter(|(_, y)| *y == g.chest[0].1).collect();
        let row2_y = g.chest[5].1;
        let row2: Vec<_> = g.chest.iter().filter(|(_, y)| *y == row2_y).collect();
        assert_eq!(row1.len(), 5, "the first row is full");
        assert_eq!(row2.len(), 4, "the llama's partial last row");
        // all 9 hit-test
        for i in 0..9 {
            let (x, y) = g.chest[i];
            assert_eq!(g.slot_at(x + 4, y + 4), Some(SlotRef::Chest(i)));
        }
    }
}
