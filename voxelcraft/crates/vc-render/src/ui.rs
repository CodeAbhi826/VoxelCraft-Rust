//! UI canvas (960x540 RGBA) with hand-built 5x7 bitmap font, Minecraft-style
//! widgets (buttons + sliders), title / options / pause screens, and the
//! full 1.16.5-style HUD (hotbar, hearts, hunger, XP bar, crosshair, F3).
//! Redrawn only when state changes; uploaded to GPU as a texture.

use std::sync::OnceLock;

use crate::textures::blit_tile;
use vc_blocks::blocks::*;
use vc_inventory::inventory::ItemStack;

pub const UI_W: usize = 960;
pub const UI_H: usize = 540;

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
pub fn slider_h(
    id: u16,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    value: f32,
) -> Widget {
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
    let (cx, cy) = (UI_W as f32 / 2.0, UI_H as f32 / 2.0);
    for w in ws.iter_mut() {
        w.x = (cx + (w.x as f32 - cx) * s).round() as i32;
        w.y = (cy + (w.y as f32 - cy) * s).round() as i32;
        w.w = (w.w as f32 * s).round() as i32;
        w.h = (w.h as f32 * s).round() as i32;
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
/// maximum world entries the select screen lists (ids 60..60+n)
pub const MAX_LISTED_WORLDS: usize = 8;
pub const ID_OPT_FOV: u16 = 10;
pub const ID_OPT_SENS: u16 = 11;
pub const ID_OPT_RD: u16 = 12;
pub const ID_OPT_BRIGHT: u16 = 13;
pub const ID_OPT_VOL: u16 = 14;
pub const ID_OPT_SHADER: u16 = 15;
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
pub const ID_PACK_BASE: u16 = 110;
/// pack rows available (3 engine shader modes + up to 5 packs)
pub const MAX_PACK_ENTRIES: usize = 8;
/// 2026-09-14 round: the REAL Resource Packs screen (vanilla two-pane
/// Available/Selected — the old shader-mode list moved to a dedicated
/// Shader Packs screen reached from Video Settings, Iris-style).
pub const ID_OPT_SHADERS: u16 = 51; // Video Settings → SHADER PACKS...
/// vanilla "View Bobbing" toggle (Options screen, default ON)
pub const ID_OPT_BOB: u16 = 52;
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
    let cx = (UI_W as i32 - 300) / 2;
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
        btn_h(ID_OPT_CHAT, l, rows[2], bw, 30, "CHAT SETTINGS...", "", false),
        btn_h(ID_OPT_PACKS, r, rows[2], bw, 30, "RESOURCE PACKS...", "", true),
        btn_h(ID_OPT_LANG, l, rows[3], bw, 30, "LANGUAGE...", "", false),
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
        btn_h(ID_OPT_VIDEO, l, rows[4], bw, 30, "VIDEO SETTINGS...", "", true),
        btn_h(ID_OPT_CONTROLS, r, rows[4], bw, 30, "CONTROLS...", "", false),
        btn_h(ID_OPT_ENGINE, 248, 252, 465, 30, "ENGINE SETTINGS...", "", true),
        // vanilla 1.16.5 Options-screen option (default ON): the walk-cycle
        // camera/hand sway
        btn_h(ID_OPT_BOB, 248, 292, 465, 30, "VIEW BOBBING", "ON", true),
        btn_h(
            ID_OPT_DONE,
            (UI_W as i32 - 300) / 2,
            470,
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Video Settings — the EXACT vanilla 1.16.5 screen: full-width Render
/// Distance slider on top, four two-column cycling rows (Graphics |
/// Smooth Lighting, GUI Scale | Clouds, Particles | Full Screen, Use
/// VSync | Entity Shadows), the unlabeled full-width Brightness slider
/// (hover shows Moody/Bright), the full-width Biome Blend slider, Done.
pub fn layout_video() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [108, 144, 180, 216];
    vec![
        slider_h(ID_OPT_RD, 248, 72, 465, 30, "RENDER DISTANCE", 0.4),
        btn_h(ID_OPT_GRAPHICS, l, rows[0], bw, 30, "GRAPHICS", "FANCY", true),
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
        btn_h(ID_OPT_GUISCALE, l, rows[1], bw, 30, "GUI SCALE", "AUTO", true),
        btn_h(ID_OPT_CLOUDS, r, rows[1], bw, 30, "CLOUDS", "FANCY", true),
        btn_h(ID_OPT_PARTICLES, l, rows[2], bw, 30, "PARTICLES", "ALL", true),
        btn_h(ID_OPT_FULLSCREEN, r, rows[2], bw, 30, "FULL SCREEN", "OFF", true),
        btn_h(ID_OPT_VSYNC, l, rows[3], bw, 30, "USE VSYNC", "ON", true),
        btn_h(ID_OPT_ENTSHADOW, r, rows[3], bw, 30, "ENTITY SHADOWS", "ON", true),
        // vanilla brightness slider carries NO label; the hover tooltip
        // reads Moody/Bright from the live value
        slider_h(ID_OPT_BRIGHT, 248, 252, 465, 30, "", 0.1),
        slider_h(ID_OPT_BIOME, 248, 288, 465, 30, "BIOME BLEND", 0.5),
        // 2026-09-14: SHADER PACKS lives HERE (Iris-style — vanilla
        // 1.16.5 has no shader screen; Iris/OptiFine add theirs to Video
        // Settings), not on the Options page where it used to squat
        // mislabeled as "RESOURCE PACKS..."
        btn_h(
            ID_OPT_SHADERS,
            248,
            324,
            465,
            30,
            "SHADER PACKS...",
            "",
            true,
        ),
        btn_h(
            ID_OPT_DONE2,
            (UI_W as i32 - 300) / 2,
            470,
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Engine Settings — our extra subsystems (GPU meshing, occlusion,
/// texture/AA quality, sim distance, frame cap, upscaling, sun shadows)
/// live on their own page so the Video screen stays vanilla-exact.
pub fn layout_engine() -> Vec<Widget> {
    let (l, r, bw) = (248, 487, 225);
    let rows = [72, 108, 144, 180, 216];
    vec![
        slider_h(ID_OPT_SIMDIST, l, rows[0], bw, 30, "SIM DISTANCE", 0.25),
        btn_h(ID_OPT_MAXFPS, r, rows[0], bw, 30, "MAX FPS", "UNCAPPED", true),
        btn_h(ID_OPT_MIP, l, rows[1], bw, 30, "MIPMAP LEVELS", "4", true),
        btn_h(ID_OPT_ANISO, r, rows[1], bw, 30, "ANISOTROPIC", "4X", true),
        btn_h(ID_OPT_MSAA, l, rows[2], bw, 30, "MSAA", "OFF", true),
        btn_h(ID_OPT_OCCL, r, rows[2], bw, 30, "OCCLUSION CULLING", "ON", true),
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
        btn_h(ID_OPT_SHADOWS, r, rows[3], bw, 30, "SUN SHADOWS", "2K", true),
        btn_h(ID_OPT_UPSCALE, l, rows[4], bw, 30, "UPSCALING", "OFF", true),
        btn_h(
            ID_OPT_DONE2,
            (UI_W as i32 - 300) / 2,
            470,
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}

/// Shader Packs — the Iris-style shader selection screen (engine shader
/// modes OFF/VANILLA+/CINEMATIC + WGSL shader packs). Moved here from the
/// old mislabeled "RESOURCE PACKS" screen in the 2026-09-14 round: the
/// RESOURCE PACKS entry on the Options screen now opens the real
/// resource-pack manager (`layout_resource_packs`), because vanilla
/// 1.16.5 has NO shader-pack screen at all — shaders are an Iris/OptiFine
/// concept, so this list lives on its own page reached from Video
/// Settings. `selected` marks the active entry.
pub fn layout_packs(entries: &[String], selected: usize) -> Vec<Widget> {
    let mut v = Vec::new();
    for (i, name) in entries.iter().take(MAX_PACK_ENTRIES).enumerate() {
        v.push(btn_h(
            ID_PACK_BASE + i as u16,
            248,
            72 + i as i32 * 40,
            465,
            30,
            name,
            if i == selected { "SELECTED" } else { "" },
            true,
        ));
    }
    v.push(btn_h(
        ID_OPT_DONE2,
        (UI_W as i32 - 300) / 2,
        470,
        300,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Resource Packs — the vanilla 1.16.5 two-pane manager (VERIFIED live
/// 2026-09-14, minecraft.wiki/w/Resource_pack §Behavior: packs "can be
/// moved between 'Available' (disabled) and 'Selected' (enabled), and
/// reordered"; "The bottom-most pack loads first, then each pack above it
/// replaces or merges loaded assets"; Default is "Selected by default,
/// can't be unselected").
///
/// * LEFT pane — `avail`: disabled packs (Programmer Art + user packs
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
        (UI_W as i32 - 300) / 2,
        470,
        300,
        30,
        "DONE",
        "",
        true,
    ));
    v
}

/// Accessibility Settings — vanilla 1.16.5 home of the Auto-Jump toggle
/// (the screen's other entries land with their subsystems).
pub fn layout_access() -> Vec<Widget> {
    vec![
        btn_h(ID_OPT_AUTOJUMP, 248, 72, 465, 30, "AUTO-JUMP", "ON", true),
        btn_h(
            ID_OPT_DONE2,
            (UI_W as i32 - 300) / 2,
            470,
            300,
            30,
            "DONE",
            "",
            true,
        ),
    ]
}



pub fn layout_pause() -> Vec<Widget> {
    vec![
        btn(
            ID_PAUSE_BACK,
            (UI_W as i32 - 320) / 2,
            208,
            320,
            "BACK TO GAME",
            "",
            true,
        ),
        btn(
            ID_PAUSE_OPTIONS,
            (UI_W as i32 - 320) / 2,
            264,
            320,
            "OPTIONS...",
            "",
            true,
        ),
        btn(
            ID_PAUSE_QUIT,
            (UI_W as i32 - 320) / 2,
            320,
            320,
            "QUIT TO TITLE",
            "",
            true,
        ),
    ]
}

/// Phase 1: world-select layout (native). One button per saved world plus
/// create/delete/cancel. `dead` marks a hardcore world whose player died —
/// it can't be played but stays clickable so it can be selected + deleted.
pub fn layout_world_select(
    names: &[(String, String, bool)], // (name, mode label, dead)
) -> Vec<Widget> {
    let mut v = Vec::new();
    for (i, (name, mode, dead)) in names.iter().take(MAX_LISTED_WORLDS).enumerate() {
        let label = if *dead {
            format!("{name} - GAME OVER")
        } else {
            format!("{name} ({mode})")
        };
        v.push(btn(
            ID_WS_WORLD_BASE + i as u16,
            176,
            96 + i as i32 * 56,
            500,
            &label,
            "",
            true,
        ));
    }
    let y = 96 + names.len().min(MAX_LISTED_WORLDS) as i32 * 56 + 8;
    v.push(btn(ID_WS_CREATE, 176, y, 242, "CREATE NEW WORLD", "", true));
    v.push(btn(ID_WS_DELETE, 434, y, 242, "DELETE SELECTED", "", true));
    v.push(btn(ID_WS_CANCEL, 176, y + 56, 500, "CANCEL", "", true));
    v
}

/// Phase 1: world-create layout. Values refresh on every keystroke /
/// mode cycle from game.rs (widgets are rebuilt per state change).
pub fn layout_world_create(
    name: &str,
    seed_placeholder: &str,
    mode_label: &str,
    mode_desc: &str,
    type_label: &str,
) -> Vec<Widget> {
    let col = 176;
    let w = 500;
    vec![
        text_field(ID_WC_NAME, col, 96, w, "NAME", name, "New World"),
        text_field(ID_WC_SEED, col, 152, w, "SEED", "", seed_placeholder),
        btn(ID_WC_MODE, col, 208, w, "GAME MODE", mode_label, true),
        // Phase E3 (VERIFIED w/Superflat): world-type cycle Normal ↔
        // Superflat (classic preset — enabled now)
        btn(ID_WC_TYPE, col, 264, w, "WORLD TYPE", type_label, true),
        btn(ID_WC_CREATE, col, 336, w, "CREATE WORLD", mode_desc, true),
        btn(ID_WC_CANCEL, col, 392, w, "CANCEL", "", true),
    ]
}

/// Phase 1: death screen (Survival vs Hardcore variants).
pub fn layout_death(hardcore: bool) -> Vec<Widget> {
    let mut v = Vec::new();
    if !hardcore {
        v.push(btn(
            ID_DEATH_RESPAWN,
            (UI_W as i32 - 320) / 2,
            300,
            320,
            "RESPAWN",
            "",
            true,
        ));
        v.push(btn(
            ID_DEATH_TITLE,
            (UI_W as i32 - 320) / 2,
            356,
            320,
            "TITLE SCREEN",
            "",
            true,
        ));
    } else {
        // hardcore: death is final — vanilla's two options (delete world /
        // title screen, which leaves the locked world on disk)
        v.push(btn(
            ID_DEATH_DELETE,
            (UI_W as i32 - 320) / 2,
            300,
            320,
            "DELETE WORLD",
            "",
            true,
        ));
        v.push(btn(
            ID_DEATH_TITLE,
            (UI_W as i32 - 320) / 2,
            356,
            320,
            "TITLE SCREEN",
            "",
            true,
        ));
    }
    v
}

// ------------------------------------------------- Phase 5 font core --
// variable-width advance: glyph advance = measured ink width + 1
// (VERIFIED https://minecraft.wiki/w/Font — "the width of each
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
    pub dirty: bool,
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
fn splash_ink_at(
    s: &str,
    cell: f32,
    e: &mut crate::gui::font::FontEngine,
) -> (Vec<u8>, i32, i32) {
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
            widget_scale: 1.0,
            chrome_enabled: true,
            gui_frame: crate::gui_render::GuiFrame::default(),
            icon_cells: None,
            device_scale: 1.0,
        }
    }

    /// the device scale (device px per UI px) for the GPU text path —
    /// set per frame from the Renderer's letterbox uniform
    pub fn set_device_scale(&mut self, k: f32) {
        self.device_scale = k.max(0.05);
    }

    /// Phase 3: install the ready-icon snapshot (called by the game
    /// whenever the icon cache's version moves)
    pub fn set_icon_cells(&mut self, cells: std::sync::Arc<std::collections::HashMap<u16, [u8; 2]>>) {
        self.icon_cells = Some(cells);
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.px.iter_mut().for_each(|p| *p = 0);
        self.gui_frame.clear();
        self.dirty = true;
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
        if let Some(img) = image::RgbaImage::from_raw(UI_W as u32, UI_H as u32, self.px.clone())
        {
            let _ = img.save(path);
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        if x < 0 || x >= UI_W as i32 || y < 0 || y >= UI_H as i32 {
            return;
        }
        let i = (y as usize * UI_W + x as usize) * 4;
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
        self.text((UI_W as i32 - w) / 2, y, s, c, scale);
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
    /// up), pulsing at 2 Hz (VERIFIED minecraft.wiki/w/Splash: "yellow
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

    /// Minecraft-style button (gray body, bevel, hover tint).
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
        let body: Color = if enabled {
            [96, 96, 96, 235]
        } else {
            [56, 56, 56, 215]
        };
        self.rect(w.x, w.y, w.w, w.h, body);
        // bevel: light top/left, dark bottom/right
        self.rect(w.x + 2, w.y + 2, w.w - 4, 2, [140, 140, 140, 255]);
        self.rect(w.x + 2, w.y + 2, 2, w.h - 4, [130, 130, 130, 255]);
        self.rect(w.x + 2, w.y + w.h - 4, w.w - 4, 2, [58, 58, 58, 255]);
        self.rect(w.x + w.w - 4, w.y + 2, 2, w.h - 4, [58, 58, 58, 255]);
        // 2px black border
        self.frame(w.x, w.y, w.w, w.h, [12, 12, 12, 255]);
        self.frame(w.x + 1, w.y + 1, w.w - 2, w.h - 2, [42, 42, 42, 255]);
        if hover && enabled {
            let tint: Color = [130, 160, 255, 70];
            self.rect(w.x + 2, w.y + 2, w.w - 4, w.h - 4, tint);
            self.frame(w.x + 2, w.y + 2, w.w - 4, w.h - 4, [255, 255, 255, 130]);
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

    /// Minecraft-style slider: inset track + knob.
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
        // track: dark inset
        self.rect(w.x, ty, w.w, th, [30, 30, 30, 230]);
        self.frame(w.x, ty, w.w, th, [12, 12, 12, 255]);
        self.rect(w.x + 2, ty + 2, w.w - 4, th - 4, [86, 86, 86, 230]);
        self.rect(w.x + 2, ty + 2, w.w - 4, 2, [64, 64, 64, 255]);
        // knob (16 wide, button style)
        let kx = w.x + 8 + ((w.w - 16 - 16) as f32 * value) as i32;
        self.rect(kx, ty - 4, 16, th + 8, [110, 110, 110, 250]);
        self.frame(kx, ty - 4, 16, th + 8, [12, 12, 12, 255]);
        self.rect(kx + 2, ty - 2, 12, 2, [150, 150, 150, 255]);
        self.rect(kx + 2, ty + th, 12, 2, [58, 58, 58, 255]);
        if hover {
            self.frame(kx + 1, ty - 3, 14, th + 6, [255, 255, 255, 110]);
        }
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
                    self.rect(w.x + pad + tw + 1, w.y + (12.0 * self.widget_scale) as i32, 2, ch, [240, 240, 240, 255]);
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
    /// (VERIFIED minecraft.wiki/w/Title_screen + /w/Splash.)
    pub fn title_screen(&mut self, splash: &str, ws: &[Widget], hover: Option<u16>, time: f32) {
        // logo: big blocky wordmark over the panorama (vanilla draws its
        // logo with a dark outline — no dim band behind it)
        let scale = 12;
        let logo = "VOXELCRAFT";
        let lw = Self::text_width(logo, scale);
        let lx = (UI_W as i32 - lw) / 2;
        let ly = 18;
        // soft drop shadow
        self.text(lx + 4, ly + 6, logo, [0, 0, 0, 150], scale);
        // dark outline pass
        self.text_outlined(lx, ly, logo, [235, 235, 235, 255], [42, 42, 42, 255], scale);

        // splash: yellow, tilted -20 deg (right side up), pulsing 2 Hz,
        // tucked at the logo's bottom-right corner
        let sw = Self::text_width(splash, 2);
        let cx = (lx + lw - 30 - sw / 2).clamp(40, UI_W as i32 - 40);
        let cy = ly + 62;
        self.text_splash(cx, cy, splash, time);

        self.draw_widgets(ws, hover);

        self.text(
            8,
            UI_H as i32 - 20,
            "VoxelCraft 1.16.5",
            [220, 220, 220, 255],
            1,
        );
        let vr = "100% CLEAN-ROOM - NOT AN OFFICIAL GAME";
        let vw = Self::text_width(vr, 1);
        self.text(
            UI_W as i32 - vw - 8,
            UI_H as i32 - 20,
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
        // Phase 2: the vanilla options background (16x16 dirt tiles at
        // 0.25 brightness) rides the quad pass; the canvas dark rect
        // stays as the no-quads fallback
        self.gui_frame
            .dirt_background(UI_W as i32, UI_H as i32);
        self.rect(0, 0, UI_W as i32, UI_H as i32, [8, 8, 10, 110]);
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
        self.gui_frame
            .dirt_background(UI_W as i32, UI_H as i32);
        self.rect(0, 0, UI_W as i32, UI_H as i32, [8, 8, 10, 110]);
        self.text_center(18, "RESOURCE PACKS", [255, 255, 255, 255], 3);
        for (i, line) in tooltip.iter().take(2).enumerate() {
            self.text_center(46 + i as i32 * 12, line, [170, 170, 170, 255], 1);
        }
        // pane headers
        let aw = Self::text_width("AVAILABLE", 2);
        self.text((450 - aw) / 2, 62, "AVAILABLE", [255, 255, 255, 255], 2);
        let sw = Self::text_width("SELECTED", 2);
        self.text((720 - sw) / 2 + 210, 62, "SELECTED", [255, 255, 255, 255], 2);
        // dark inset panels behind the rows (vanilla's sunken list look)
        self.rect(26, 86, 428, 348, [0, 0, 0, 130]);
        self.rect(506, 86, 428, 348, [0, 0, 0, 130]);
        self.draw_widgets(ws, hover);
    }

    pub fn pause_screen(&mut self, ws: &[Widget], hover: Option<u16>) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [0, 0, 0, 130]);
        self.text_center(140, "GAME MENU", [255, 255, 255, 255], 3);
        self.draw_widgets(ws, hover);
    }

    /// Phase 1: world-select screen (native — the browser build creates
    /// worlds directly, no persistent list).
    pub fn world_select_screen(
        &mut self,
        ws: &[Widget],
        hover: Option<u16>,
        selected: Option<usize>,
        count_shown: usize,
        total: usize,
    ) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [8, 8, 10, 200]);
        self.text_center(18, "SELECT WORLD", [255, 255, 255, 255], 3);
        if total == 0 {
            self.text_center(
                64,
                "NO SAVED WORLDS YET - CREATE ONE BELOW",
                [170, 170, 170, 255],
                1,
            );
        } else if total > count_shown {
            let sub = format!("SHOWING {count_shown} OF {total} (OLDEST HIDDEN)");
            self.text_center(64, &sub, [170, 170, 170, 255], 1);
        }
        // highlight the selected row (vanilla-style white frame)
        if let Some(sel) = selected {
            if let Some(w) = ws.iter().find(|w| w.id == ID_WS_WORLD_BASE + sel as u16) {
                self.frame(w.x - 3, w.y - 3, w.w + 6, w.h + 6, [255, 255, 255, 200]);
            }
        }
        self.draw_widgets(ws, hover);
    }

    /// Phase 1: world-create screen (shared native/web).
    pub fn world_create_screen(&mut self, ws: &[Widget], hover: Option<u16>, time: f32) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [8, 8, 10, 200]);
        self.text_center(18, "CREATE NEW WORLD", [255, 255, 255, 255], 3);
        self.text_center(
            64,
            "SEED: NUMBER = ITSELF, TEXT = JAVA HASH, BLANK = RANDOM",
            [150, 150, 150, 255],
            1,
        );
        // focused-field hint + blinking caret handled per widget
        self.draw_widgets_caret(ws, hover, time);
    }

    /// Phase 1: death screen — red-tinged overlay, vanilla "You died!".
    pub fn death_screen(&mut self, ws: &[Widget], hover: Option<u16>, hardcore: bool, cause: &str) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [80, 0, 0, 150]);
        let title = if hardcore { "GAME OVER!" } else { "YOU DIED!" };
        let tw = Self::text_width(title, 5);
        self.text((UI_W as i32 - tw) / 2, 150, title, [255, 240, 240, 255], 5);
        let sub = if hardcore {
            "HARDCORE WORLD - DEATH IS PERMANENT"
        } else {
            cause
        };
        self.text_center(210, sub, [230, 200, 200, 255], 1);
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
    /// reference: minecraft.wiki/w/Crosshair — the classic
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
        let cx = (UI_W / 2) as i32;
        let cy = (UI_H / 2) as i32;
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
        let cxd = snap(UI_W as f32 * 0.5 * k);
        let cyd = snap(UI_H as f32 * 0.5 * k);
        let ha = (snap(8.0 * k) as i32).max(4); // half-arm (device px)
        let ht = (snap(2.0 * k) as i32).max(2); // arm thickness
        // bar top/left snapped so the arm covers whole device px
        // (odd thickness sits 1 px heavy toward +x/+y — invisible on
        // a symmetric-plus crosshair, and every edge stays crisp)
        let wy = cyd as i32 - (ht + 1) / 2;
        let wx = cxd as i32 - (ht + 1) / 2;
        // device-px rect → fractional UI rect (dst = device / k)
        let q = |x: i32, y: i32, w: i32, h: i32, f: &mut Self| {
            f.gui_frame.solid_invert(
                x as f32 / k,
                y as f32 / k,
                w as f32 / k,
                h as f32 / k,
            );
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

    /// Full 1.16.5-style status bars: hearts (left), hunger (right), XP
    /// bar, and the oxygen bubble row above hunger (VERIFIED —
    /// research-verdicts.md live round: 10 bubbles × 30 air; drawn only
    /// while the air supply is below full, right-aligned above hunger).
    pub fn status_bars(&mut self, health: f32, food: f32, xp: f32, level: u32, air: f32) {
        let hb_w = 9 * 40 + 4;
        let hb_x = (UI_W as i32 - hb_w) / 2;
        let hb_y = UI_H as i32 - 48;

        // hearts row
        let heart_pal: [(char, Color); 4] = [
            ('O', [46, 6, 6, 255]),
            ('R', [227, 27, 13, 255]),
            ('H', [255, 116, 116, 255]),
            ('W', [255, 255, 255, 255]),
        ];
        for i in 0..10i32 {
            let x = hb_x + 2 + i * 17;
            let y = hb_y - 26;
            // Phase 2: the 9x9 quad sprite (18x18 drawn) always pushed;
            // the legacy canvas sprite raster is gated
            let variant = if health >= (i + 1) as f32 / 10.0 {
                crate::textures::gui_art::HeartVariant::Full
            } else if health > i as f32 / 10.0 {
                crate::textures::gui_art::HeartVariant::Half
            } else {
                crate::textures::gui_art::HeartVariant::Empty
            };
            self.gui_frame.heart(x, y, variant);
            if !self.chrome_enabled {
                continue;
            }
            // background outline (empty heart) then fill
            if health >= (i + 1) as f32 / 10.0 {
                self.sprite(x, y, &Self::HEART, &heart_pal, 2);
            } else {
                let dim: [(char, Color); 4] = [
                    ('O', [30, 30, 30, 200]),
                    ('R', [70, 70, 70, 200]),
                    ('H', [90, 90, 90, 200]),
                    ('W', [110, 110, 110, 200]),
                ];
                self.sprite(x, y, &Self::HEART, &dim, 2);
            }
        }

        // hunger row (right aligned, mirrored order)
        let food_pal: [(char, Color); 4] = [
            ('O', [43, 26, 4, 255]),
            ('M', [186, 106, 38, 255]),
            ('W', [222, 222, 222, 255]),
            ('H', [255, 255, 255, 255]),
        ];
        for i in 0..10i32 {
            let x = hb_x + hb_w - 4 - (i + 1) * 17;
            let y = hb_y - 28;
            // Phase 2: quad sprite always pushed (right row mirrors)
            let variant = if food >= (i + 1) as f32 / 10.0 {
                crate::textures::gui_art::HungerVariant::Full
            } else if food > i as f32 / 10.0 {
                crate::textures::gui_art::HungerVariant::Half
            } else {
                crate::textures::gui_art::HungerVariant::Empty
            };
            self.gui_frame.hunger(x, y, variant);
            if !self.chrome_enabled {
                continue;
            }
            if food >= (i + 1) as f32 / 10.0 {
                self.sprite(x, y, &Self::FOOD, &food_pal, 2);
            } else {
                let dim: [(char, Color); 4] = [
                    ('O', [30, 30, 30, 200]),
                    ('M', [70, 70, 70, 200]),
                    ('W', [110, 110, 110, 200]),
                    ('H', [110, 110, 110, 200]),
                ];
                self.sprite(x, y, &Self::FOOD, &dim, 2);
            }
        }

        // oxygen bubbles (air supply < full): right-aligned row ABOVE
        // the hunger bar, mirrored order (vanilla position); ceil(air/30)
        // full bubbles — at the pop boundary the last one blinks out
        if air < 299.0 {
            let bubble_pal: [(char, Color); 4] = [
                ('o', [26, 46, 78, 255]),
                ('W', [235, 247, 255, 255]),
                ('B', [94, 158, 222, 255]),
                ('.', [0, 0, 0, 0]),
            ];
            let bubbles = (air.max(0.0) / 30.0).ceil() as i32;
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

        // XP bar — Luanti font round: Solid quads (pixel-crisp at any
        // window size, and immune to the canvas/quad z-order class of
        // bugs); the canvas raster stays as the no-GPU-pass fallback
        let xp_w = hb_w;
        let xp_x = hb_x;
        let xp_y = hb_y - 10;
        let fill = ((xp_w - 4) as f32 * xp.clamp(0.0, 1.0)) as i32;
        let ct = |c: Color| -> [f32; 4] {
            [
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                c[3] as f32 / 255.0,
            ]
        };
        self.gui_frame.solid_rect(xp_x, xp_y, xp_w, 8, ct([16, 16, 16, 220]));
        self.gui_frame.solid_rect(xp_x, xp_y, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame.solid_rect(xp_x, xp_y + 7, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame.solid_rect(xp_x, xp_y, 1, 8, ct([60, 60, 60, 255]));
        self.gui_frame
            .solid_rect(xp_x + xp_w - 1, xp_y, 1, 8, ct([60, 60, 60, 255]));
        if fill > 0 {
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 4, ct([128, 255, 32, 255]));
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 1, ct([190, 255, 130, 255]));
        }
        if self.chrome_enabled {
            self.rect(xp_x, xp_y, xp_w, 8, [16, 16, 16, 220]);
            self.frame(xp_x, xp_y, xp_w, 8, [60, 60, 60, 255]);
            if fill > 0 {
                self.rect(xp_x + 2, xp_y + 2, fill, 4, [128, 255, 32, 255]);
                self.rect(xp_x + 2, xp_y + 2, fill, 1, [190, 255, 130, 255]);
            }
        }
        if level > 0 {
            let s = format!("{}", level);
            let w = Self::text_width(&s, 2);
            self.text_outlined(
                (UI_W as i32 - w) / 2,
                xp_y - 20,
                &s,
                [128, 255, 32, 255],
                [20, 40, 8, 255],
                2,
            );
        }
    }

    /// Phase E1: the ender-dragon boss bar — VERIFIED w/Ender_Dragon:
    /// "a light purple health bar ... at the top of the player's screen",
    /// the name above it, width matches the hotbar band. `frac` = the
    /// dragon's remaining health fraction (0..1).
    pub fn boss_bar(&mut self, frac: f32) {
        let w = 9 * 40 + 4; // hotbar-width band (the vanilla boss-bar width)
        let x = (UI_W as i32 - w) / 2;
        let y = 24;
        // label
        let name = "ENDER DRAGON";
        let tw = name.len() as i32 * 8;
        self.text(
            (UI_W as i32 - tw) / 2,
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
        self.gui_frame.solid_rect(x, y, w, 12, ct([16, 12, 20, 220]));
        // 1-px frame (the canvas `frame` decomposition)
        self.gui_frame.solid_rect(x, y, w, 1, ct([90, 70, 110, 255]));
        self.gui_frame.solid_rect(x, y + 11, w, 1, ct([90, 70, 110, 255]));
        self.gui_frame.solid_rect(x, y, 1, 12, ct([90, 70, 110, 255]));
        self.gui_frame.solid_rect(x + w - 1, y, 1, 12, ct([90, 70, 110, 255]));
        let fill = ((w - 4) as f32 * frac.clamp(0.0, 1.0)) as i32;
        if fill > 0 {
            self.gui_frame.solid_rect(x + 2, y + 2, fill, 8, ct([190, 90, 220, 255]));
            self.gui_frame.solid_rect(x + 2, y + 2, fill, 2, ct([230, 150, 250, 255]));
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

    /// Phase 1: creative HUD — no hearts, no hunger, XP bar only
    /// (vanilla creative shows no status rows; levels still matter here
    /// because enchanting spends them), plus the oxygen bubbles
    /// (creative players still lose air visually — damage is gated by
    /// invulnerability).
    pub fn xp_bar_only(&mut self, xp: f32, level: u32, air: f32) {
        let hb_w = 9 * 40 + 4;
        let hb_x = (UI_W as i32 - hb_w) / 2;
        let hb_y = UI_H as i32 - 48;
        let xp_w = hb_w;
        let xp_x = hb_x;
        let xp_y = hb_y - 10;
        // Luanti round 2: solid quads — the exact XP block the survival
        // `status_bars` renders (pixel-crisp at any window size, immune
        // to the canvas/quad z-order bug class); canvas raster gated
        let ct = |c: Color| -> [f32; 4] {
            [
                c[0] as f32 / 255.0,
                c[1] as f32 / 255.0,
                c[2] as f32 / 255.0,
                c[3] as f32 / 255.0,
            ]
        };
        self.gui_frame.solid_rect(xp_x, xp_y, xp_w, 8, ct([16, 16, 16, 220]));
        self.gui_frame.solid_rect(xp_x, xp_y, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame.solid_rect(xp_x, xp_y + 7, xp_w, 1, ct([60, 60, 60, 255]));
        self.gui_frame.solid_rect(xp_x, xp_y, 1, 8, ct([60, 60, 60, 255]));
        self.gui_frame
            .solid_rect(xp_x + xp_w - 1, xp_y, 1, 8, ct([60, 60, 60, 255]));
        let fill = ((xp_w - 4) as f32 * xp.clamp(0.0, 1.0)) as i32;
        if fill > 0 {
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 4, ct([128, 255, 32, 255]));
            self.gui_frame
                .solid_rect(xp_x + 2, xp_y + 2, fill, 1, ct([190, 255, 130, 255]));
        }
        if self.chrome_enabled {
            self.rect(xp_x, xp_y, xp_w, 8, [16, 16, 16, 220]);
            self.frame(xp_x, xp_y, xp_w, 8, [60, 60, 60, 255]);
            if fill > 0 {
                self.rect(xp_x + 2, xp_y + 2, fill, 4, [128, 255, 32, 255]);
                self.rect(xp_x + 2, xp_y + 2, fill, 1, [190, 255, 130, 255]);
            }
        }
        if level > 0 {
            let s = format!("{}", level);
            let w = Self::text_width(&s, 2);
            self.text_outlined(
                (UI_W as i32 - w) / 2,
                xp_y - 20,
                &s,
                [128, 255, 32, 255],
                [20, 40, 8, 255],
                2,
            );
        }
        // oxygen bubbles also render in creative (vanilla shows them) —
        // the 9x9 quad sprite always pushed (the status_bars pattern),
        // canvas raster gated
        if air < 299.0 {
            let bubble_pal: [(char, Color); 4] = [
                ('o', [26, 46, 78, 255]),
                ('W', [235, 247, 255, 255]),
                ('B', [94, 158, 222, 255]),
                ('.', [0, 0, 0, 0]),
            ];
            let bubbles = (air.max(0.0) / 30.0).ceil() as i32;
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
        let hb_w = 9 * 40 + 4;
        let hb_x = (UI_W as i32 - hb_w) / 2;
        let hb_y = UI_H as i32 - 48;
        // above the XP level-number zone, left-aligned with the hotbar
        let x = hb_x + 4;
        let y = hb_y - 44;
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
        let x0 = (UI_W as i32 - bw) / 2;
        let y0 = UI_H as i32 - 48;
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
                (UI_W as i32 - w) / 2,
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
            match self
                .icon_cells
                .as_ref()
                .and_then(|m| m.get(&b).copied())
            {
                Some(cell) => self.gui_frame.icon_quad(sx + 2, sy + 2, cell),
                None => blit_tile(
                    atlas,
                    tile,
                    2,
                    (sx + 2) as usize,
                    (sy + 2) as usize,
                    &mut self.px,
                    UI_W,
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
        let x0 = (UI_W as i32 - grid_w) / 2;
        // top-area height per kind
        let top_h = match kind {
            ContainerKind::Inventory => 96, // 2x2 craft + arrow + output
            ContainerKind::Crafting => 140, // 3x3 craft + arrow + output
            ContainerKind::Chest => 132,    // 3 rows of 9 slots
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
        };
        let panel_h = top_h + 3 * 44 + 8 + 44 + 30; // + title + gaps + padding
        let y0 = (UI_H as i32 - panel_h) / 2;

        // panel chrome (the trade screen is wider: two 260px columns)
        let trade_wide = kind == ContainerKind::Trade;
        let pw = if trade_wide { 562 } else { grid_w + 28 };
        let px0 = if trade_wide {
            (UI_W as i32 - pw) / 2
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
            // 1.14: the barrel's own label (the vanilla GUI title)
            ContainerKind::Barrel => "BARREL",
            // VERIFIED vanilla GUI label: "Item Hopper"
            ContainerKind::Hopper => "ITEM HOPPER",
            ContainerKind::Furnace => "FURNACE",
            ContainerKind::Brewing => "BREWING STAND",
            ContainerKind::Enchant => "ENCHANT  (needs book + lapis + levels)",
            ContainerKind::Trade => "VILLAGER",
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
        };

        // ---- container-specific top area ----
        match kind {
            ContainerKind::Inventory => {
                // 2x2 grid + arrow + output, centered
                let total = 2 * 40 + 50 + 36;
                let cx = x0 + (grid_w - total) / 2;
                let cy = y0 + 8;
                for r in 0..2 {
                    for c in 0..2 {
                        let x = cx + c as i32 * 40;
                        let y = cy + r as i32 * 40;
                        self.slot_well(x, y, &view.grid[r * 2 + c], atlas);
                        geom.craft.push((x, y));
                    }
                }
                self.arrow(
                    cx + 84,
                    cy + 12,
                    if !view.craft_out.is_empty() { 1.0 } else { 0.0 },
                );
                let ox = cx + 134;
                let oy = cy + 2;
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
                    format!("{} (minecraft:{})", name(s.block), id)
                } else {
                    name(s.block).to_string()
                };
                let lw = Self::text_width(&label, 1);
                self.text(
                    (UI_W as i32 - lw) / 2,
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
            let half = UI_W as f32 / 2.0 - 12.0;
            if strip_quads {
                let line = fit_line(l, half, |s| measure(s));
                let w = measure(&line);
                // text ends 3px from the right edge; the strip extends
                // 1 UI px past both ends of the fractional text width
                let x = UI_W as f32 - 3.0 - w;
                self.gui_frame
                    .solid_over(x - 1.0, y as f32, w + 2.0, LINE_H as f32, bg_tint);
                self.text_flat_case(x.round() as i32, y + 1, &line, FG, 2);
            } else {
                let line = fit_line(l, half, |s| Self::text_width_case(s, 2) as f32);
                let w = Self::text_width_case(&line, 2);
                let x = UI_W as i32 - 3 - w; // text ends 3px from the right edge
                self.rect(x - 1, y, w + 2, LINE_H, BG);
                self.text_flat_case(x, y + 1, &line, FG, 2);
            }
        }
    }

    /// Vanilla F3+Q help overlay: the key-combination list in a centered
    /// box (like the real "Debug help" screen — rows of "F3 + X - action"),
    /// rendered with the same true-case font as the F3 overlay.
    pub fn debug_help(&mut self, rows: &[(String, String)]) {
        let w = 340;
        let row_h = 20;
        let h = rows.len() as i32 * row_h + 40;
        let x0 = (UI_W as i32 - w) / 2;
        let y0 = (UI_H as i32 - h) / 2;
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
            self.gui_frame
                .solid_over(x0 as f32, y as f32, (w + 4) as f32, (h + 4) as f32, ct([80, 80, 80, 110]));
            self.gui_frame
                .solid_over(x0 as f32 + 2.0, guide_y as f32, w as f32, 1.0, ct([255, 255, 255, 70]));
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

    /// Creative-style block picker (E key): centered grid of every placeable
    /// block; click → assigns to the selected hotbar slot. Returns the grid
    /// geometry so game.rs can hit-test clicks.
    pub fn picker(
        &mut self,
        cursor: (f32, f32),
        atlas: &[u8],
        scroll: usize,
        advanced_tooltips: bool,
    ) -> PickerGeom {
        let blocks = &PICKER_BLOCKS;
        // [merge scroll] the F-series (1.7.2-1.10) grew PICKER_BLOCKS to
        // 236 — 15 cols x 16 rows overflows the 540px canvas; the picker
        // is now a fixed 11-row window scrolled by the game layer (mouse
        // wheel, vanilla creative-grid behavior) instead of clipping.
        let cols = 15;
        let vis_rows = 11usize;
        let cell = 44i32;
        let total_rows = blocks.len().div_ceil(cols);
        let scroll = scroll.min(total_rows.saturating_sub(vis_rows));
        // Phase E1: 12 columns (was 8) — the picker grew past 68 entries
        // with the 1.0–1.2 bracket blocks + 16 spawn eggs.
        // Phase E3: 15 columns — the picker grew to 164 entries with the
        // 1.5–1.6 bracket (quartz family, 16 stained terracotta, carpets,
        // redstone components, items, 3 eggs); 15×11 stays inside the
        // 960×540 UI canvas (668×514 grid).
        // [merge] F-series (1.7.2-1.10) grows PICKER_BLOCKS to 236 —
        // 15 cols × 16 rows overflows the 540px canvas bottom; the last
        // rows clip (known issue, scrolling picker is future UI work,
        // documented in WORKLOG).
        let grid_w = cols as i32 * cell + 8;
        let grid_h = vis_rows as i32 * cell + 8 + 22;
        let x0 = (UI_W as i32 - grid_w) / 2;
        let y0 = (UI_H as i32 - grid_h) / 2;

        self.rect(x0 - 6, y0 - 26, grid_w + 12, grid_h + 32, [16, 16, 16, 210]);
        self.frame(x0 - 6, y0 - 26, grid_w + 12, grid_h + 32, [70, 70, 70, 255]);
        self.text(
            x0 - 6 + 10,
            y0 - 24,
            "SELECT BLOCK  (B / ESC to close)  -  wheel scrolls",
            [230, 230, 230, 255],
            1,
        );

        let first = scroll * cols;
        let last = (first + vis_rows * cols).min(blocks.len());
        let mut hovered: Option<u16> = None;
        for (i, b) in blocks[first..last].iter().enumerate() {
            let col = (i % cols) as i32;
            let row = (i / cols) as i32;
            let sx = x0 + 4 + col * cell;
            let sy = y0 + 4 + row * cell;
            self.rect(sx, sy, 40, 40, [58, 58, 58, 170]);
            self.frame(sx, sy, 40, 40, [90, 90, 90, 220]);
            let tile = def(*b).tiles[0];
            blit_tile(
                atlas,
                tile,
                2,
                (sx + 4) as usize,
                (sy + 4) as usize,
                &mut self.px,
                UI_W,
            );
            // hover highlight
            let cx = cursor.0 as i32;
            let cy = cursor.1 as i32;
            if cx >= sx && cx < sx + 40 && cy >= sy && cy < sy + 40 {
                self.frame(sx - 1, sy - 1, 42, 42, [255, 255, 255, 255]);
                hovered = Some(*b);
            }
        }

        // hovered block name on a bottom strip (F3+H appends the registry
        // id — vanilla advanced tooltips)
        let label = hovered
            .map(name)
            .map(|n| {
                if advanced_tooltips {
                    let id: String = n.to_lowercase().replace(' ', "_");
                    format!("{n} (minecraft:{id})")
                } else {
                    n.to_string()
                }
            })
            .unwrap_or_default();
        let lw = Self::text_width(&label, 1);
        self.text(x0 + 4, y0 + grid_h - 18, &label, [255, 255, 255, 255], 1);
        let _ = lw;

        PickerGeom { x0, y0, cell, cols, scroll, vis_rows }
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
            ("B", "Creative block picker"),
            ("ESC", "Pause menu / options"),
            ("F3", "Debug info"),
            ("H", "This help"),
            ("[ ]", "Render distance"),
            ("- =", "Volume"),
            ("V", "Toggle V-Sync"),
        ];
        let bw = 460;
        let bh = lines.len() as i32 * 20 + 50;
        let x0 = (UI_W as i32 - bw) / 2;
        let y0 = (UI_H as i32 - bh) / 2;
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
        self.text_center(UI_H as i32 / 2 - 40, title, [255, 255, 255, 255], 3);
        self.text_center(UI_H as i32 / 2, sub, [200, 200, 200, 255], 1);
    }

    /// Boot intro screen — the FIRST screen after opening the game, in the
    /// vanilla splash-screen structure: a solid studio-brand background, the
    /// studio wordmark centered above mid-frame, and a thin white progress
    /// bar (the only thing that animates). No panorama, no world, no text
    /// caption — exactly the real boot screen's composition, clean-room.
    pub fn intro_screen(&mut self, progress: f32) {
        // solid studio-brand red (clean-room color — not a sampled asset)
        self.rect(0, 0, UI_W as i32, UI_H as i32, [239, 50, 61, 255]);
        // studio wordmark: dark on the bright field, centered
        let scale = 8;
        let logo = "VOXELCRAFT";
        let lw = Self::text_width(logo, scale);
        let lx = (UI_W as i32 - lw) / 2;
        let ly = 196;
        self.text(lx + 3, ly + 4, logo, [120, 22, 28, 200], scale);
        self.text(lx, ly, logo, [54, 54, 54, 255], scale);
        // studio sub-brand, letter-spaced under the wordmark
        let sub = "S T U D I O S";
        let suw = Self::text_width(sub, 3);
        self.text(
            (UI_W as i32 - suw) / 2 + 2,
            ly + 70,
            sub,
            [54, 54, 54, 255],
            3,
        );
        // thin white progress bar, centered, just past mid-frame (the bar
        // tracks the settled intro beat — assets all load in GameApp::new)
        let bw = 200;
        let x0 = (UI_W as i32 - bw) / 2;
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
    /// minecraft.wiki/w/Loading_world_screen): "Loading world" centered at
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
        self.rect(0, 0, UI_W as i32, UI_H as i32, [10, 12, 16, 140]);
        self.text_center(84, "LOADING WORLD", [255, 255, 255, 255], 2);
        let pct = format!("{percent}%");
        self.text_center(118, &pct, [220, 220, 220, 255], 2);

        // 35x35 colormap, 4px cells (140px square, vanilla-proportioned)
        const N: i32 = 35;
        const CELL: i32 = 4;
        let x0 = (UI_W as i32 - N * CELL) / 2;
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

/// hit-test geometry for the picker grid (UI-space), returned by
/// `UiCanvas::picker` so game.rs can map clicks to picker slots.
pub struct PickerGeom {
    pub x0: i32,
    pub y0: i32,
    pub cell: i32,
    pub cols: usize,
    /// first visible row (the game layer's scroll state, clamped by the
    /// renderer to the real range)
    pub scroll: usize,
    /// visible rows in the fixed window
    pub vis_rows: usize,
}

impl PickerGeom {
    /// which picker slot (if any) is under this UI-space cursor position
    /// (absolute PICKER_BLOCKS index, scroll-aware)
    pub fn slot_at(&self, ux: i32, uy: i32) -> Option<usize> {
        let dx = ux - (self.x0 + 4);
        let dy = uy - (self.y0 + 4);
        if dx < 0 || dy < 0 {
            return None;
        }
        let col = dx / self.cell;
        let row = dy / self.cell;
        if col >= self.cols as i32
            || row >= self.vis_rows as i32
            || dx % self.cell >= 40
            || dy % self.cell >= 40
        {
            return None;
        }
        let idx = self.scroll * self.cols + row as usize * self.cols + col as usize;
        if idx < PICKER_BLOCKS.len() {
            Some(idx)
        } else {
            None
        }
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
    /// 1.14: barrel container (3 rows of 9 — VERIFIED w/Barrel: "the
    /// same as a single chest"; shares the chest grid geometry, own
    /// title)
    Barrel,
    /// hopper container: 5 slots in one row — the verdict-corrected
    /// 176×133 vanilla screen (research doc's blanket 176×166 was
    /// confirmed wrong; see docs/research/research-verdicts.md — a hopper
    /// has one content row, not three, so the panel is genuinely shorter)
    Hopper,
}

/// a logical slot in a container screen — the target of a mouse click
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotRef {
    /// player inventory slot (0..36; 0..9 = hotbar row)
    Inv(usize),
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

impl ContainerView {
    fn hovered_stack(&self, x: i32, y: i32, geom: &ContainerGeom) -> Option<ItemStack> {
        Some(match geom.slot_at(x, y)? {
            SlotRef::Inv(i) => self.inv[i],
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
}

impl ContainerGeom {
    fn hit(x: i32, y: i32, s: &(i32, i32)) -> bool {
        x >= s.0 && x < s.0 + 36 && y >= s.1 && y < s.1 + 36
    }

    /// which logical slot (if any) is under this UI-space cursor position
    pub fn slot_at(&self, x: i32, y: i32) -> Option<SlotRef> {
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
        ID_OPT_SHADER,
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
        ID_OPT_SHADERS,
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
        let rows: [(u16, u16, &str); 6] = [
            (ID_WS_WORLD_BASE, MAX_LISTED_WORLDS as u16, "world entries"),
            (ID_PACK_BASE, MAX_PACK_ENTRIES as u16, "shader pack rows"),
            (ID_RPACK_AVAIL_BASE, MAX_RPACK_ENTRIES as u16, "rpack available"),
            (ID_RPACK_SEL_BASE, MAX_RPACK_ENTRIES as u16, "rpack selected"),
            (ID_RPACK_UP_BASE, MAX_RPACK_ENTRIES as u16, "rpack up arrows"),
            (ID_RPACK_DOWN_BASE, MAX_RPACK_ENTRIES as u16, "rpack down arrows"),
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
            cursor: ItemStack::EMPTY,
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
        ui.status_bars(20.0, 20.0, 0.5, 5, 300.0);
        // full air -> the bubble band (right side, above hunger) stays empty
        let band = nonwhite(&ui, bubble_band_rect());
        let mut ui2 = UiCanvas::new();
        ui2.status_bars(20.0, 20.0, 0.5, 5, 150.0);
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
        assert_eq!(px(&c, x0, y0), [128, 178, 82], "generated cell = biomes green");
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
        assert_eq!(px(&c, UI_W as i32 - 3, y), [80, 80, 80], "right strip margin");
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
        assert!(!matches!(
            &multi.kind,
            WidgetKind::Button { enabled: false, .. }
        ) || true);
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
                cells[idx] = if r < 3.0 { 2 } else if r < 5.0 { 1 } else { 0 };
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
        ui.status_bars(0.8, 0.8, 0.5, 3, 300.0);
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
            .min_by(|a, b| a.dst.y.partial_cmp(&b.dst.y).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        assert_eq!(xp.dst.h, 8.0, "8-px bar");
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
        assert_eq!(ui.gui_frame.text_quads.len(), 3, "H split into 2 disjoint segments + 1 V bar");
        assert!(ui.gui_frame.quads.is_empty(), "crosshair never lands in the chrome layer");
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
                    let sep_x = a.dst.x + a.dst.w <= b.dst.x + 1e-6
                        || b.dst.x + b.dst.w <= a.dst.x + 1e-6;
                    let sep_y = a.dst.y + a.dst.h <= b.dst.y + 1e-6
                        || b.dst.y + b.dst.h <= a.dst.y + 1e-6;
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
        assert!(hw.iter().all(|q| q.dst.y * k <= cy_dev && (q.dst.y + q.dst.h) * k >= cy_dev));
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
        assert!(q2.dst.w > q.dst.w && q2.dst.h > q.dst.h, "pulse scales the dst");
        let c = |r: &crate::gui_render::RectF| (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let (ax, ay) = c(&q.dst);
        let (bx, by) = c(&q2.dst);
        assert!((ax - bx).abs() < 1e-3 && (ay - by).abs() < 1e-3, "center invariant");
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

    #[test]
    fn xp_bar_only_pushes_solid_quads_and_bubble_sprites() {
        // the creative XP block now matches status_bars: solid quads
        // always, canvas gated; low air also pushes bubble quads
        let mut ui = UiCanvas::new();
        ui.set_chrome_enabled(false);
        ui.clear();
        ui.xp_bar_only(0.75, 7, 150.0);
        let solids: Vec<_> = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Solid)
            .collect();
        // bg + 4 frame edges + fill pair = 7; plus 5 bubble sprites
        assert_eq!(solids.len(), 7, "XP track block");
        let bubbles: Vec<_> = ui
            .gui_frame
            .quads
            .iter()
            .filter(|q| q.texture == crate::gui_render::QuadTexture::Bubbles)
            .collect();
        assert_eq!(bubbles.len(), 5, "ceil(150/30) bubbles");
        // canvas XP track suppressed
        let xp_y = (crate::ui::UI_H as i32 - 48 - 10) as usize;
        let idx = (xp_y * crate::ui::UI_W + 480usize) * 4;
        assert_eq!(ui.px[idx + 3], 0, "canvas XP suppressed when chrome off");
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
