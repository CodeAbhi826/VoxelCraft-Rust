//! UI canvas (960x540 RGBA) with hand-built 5x7 bitmap font, Minecraft-style
//! widgets (buttons + sliders), title / options / pause screens, and the
//! full 1.16.5-style HUD (hotbar, hearts, hunger, XP bar, crosshair, F3).
//! Redrawn only when state changes; uploaded to GPU as a texture.

use crate::blocks::*;
use crate::inventory::ItemStack;
use crate::textures::blit_tile;

pub const UI_W: usize = 960;
pub const UI_H: usize = 540;

// 5x7 font, rows top→bottom, bit 4 = leftmost pixel. ASCII 32..127.
#[rustfmt::skip]
const FONT: [[u8; 7]; 96] = [
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00], [0x04,0x04,0x04,0x04,0x04,0x00,0x04],
    [0x0A,0x0A,0x00,0x00,0x00,0x00,0x00], [0x0A,0x1F,0x0A,0x1F,0x0A,0x00,0x00],
    [0x04,0x0F,0x14,0x0E,0x05,0x1F,0x04], [0x18,0x19,0x02,0x04,0x08,0x13,0x03],
    [0x0E,0x11,0x0E,0x14,0x1E,0x11,0x16], [0x04,0x04,0x00,0x00,0x00,0x00,0x00],
    [0x02,0x04,0x08,0x08,0x08,0x04,0x02], [0x08,0x04,0x02,0x02,0x02,0x04,0x08],
    [0x00,0x04,0x15,0x0E,0x15,0x04,0x00], [0x00,0x04,0x04,0x1F,0x04,0x04,0x00],
    [0x00,0x00,0x00,0x00,0x04,0x04,0x08], [0x00,0x00,0x00,0x1F,0x00,0x00,0x00],
    [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C], [0x00,0x01,0x02,0x04,0x08,0x10,0x00],
    [0x0E,0x11,0x13,0x15,0x19,0x11,0x0E], [0x04,0x0C,0x04,0x04,0x04,0x04,0x0E],
    [0x0E,0x11,0x01,0x02,0x04,0x08,0x1F], [0x1F,0x02,0x04,0x02,0x01,0x11,0x0E],
    [0x02,0x06,0x0A,0x12,0x1F,0x02,0x02], [0x1F,0x10,0x1E,0x01,0x01,0x11,0x0E],
    [0x06,0x08,0x10,0x1E,0x11,0x11,0x0E], [0x1F,0x01,0x02,0x04,0x08,0x08,0x08],
    [0x0E,0x11,0x11,0x0E,0x11,0x11,0x0E], [0x0E,0x11,0x11,0x0F,0x01,0x02,0x0C],
    [0x00,0x0C,0x0C,0x00,0x0C,0x0C,0x00], [0x00,0x0C,0x0C,0x00,0x0C,0x04,0x08],
    [0x02,0x04,0x08,0x10,0x08,0x04,0x02], [0x00,0x00,0x1F,0x00,0x1F,0x00,0x00],
    [0x08,0x04,0x02,0x01,0x02,0x04,0x08], [0x0E,0x11,0x01,0x02,0x04,0x00,0x04],
    [0x0E,0x11,0x15,0x17,0x16,0x10,0x0E], [0x0E,0x11,0x11,0x1F,0x11,0x11,0x11],
    [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E], [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E],
    [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C], [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F],
    [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10], [0x0E,0x11,0x10,0x17,0x11,0x11,0x0F],
    [0x11,0x11,0x11,0x1F,0x11,0x11,0x11], [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E],
    [0x07,0x02,0x02,0x02,0x02,0x12,0x0C], [0x11,0x12,0x14,0x18,0x14,0x12,0x11],
    [0x10,0x10,0x10,0x10,0x10,0x10,0x1F], [0x11,0x1B,0x15,0x15,0x11,0x11,0x11],
    [0x11,0x19,0x15,0x13,0x11,0x11,0x11], [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E],
    [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10], [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D],
    [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11], [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E],
    [0x1F,0x04,0x04,0x04,0x04,0x04,0x04], [0x11,0x11,0x11,0x11,0x11,0x11,0x0E],
    [0x11,0x11,0x11,0x11,0x11,0x0A,0x04], [0x11,0x11,0x11,0x15,0x15,0x15,0x0A],
    [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11], [0x11,0x11,0x0A,0x04,0x04,0x04,0x04],
    [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F], [0x0E,0x08,0x08,0x08,0x08,0x08,0x0E],
    [0x00,0x10,0x08,0x04,0x02,0x01,0x00], [0x0E,0x02,0x02,0x02,0x02,0x02,0x0E],
    [0x04,0x0A,0x11,0x00,0x00,0x00,0x00], [0x00,0x00,0x00,0x00,0x00,0x00,0x1F],
    [0x08,0x04,0x00,0x00,0x00,0x00,0x00],
    [0x0E,0x11,0x11,0x1F,0x11,0x11,0x11], [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E],
    [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E], [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C],
    [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F], [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10],
    [0x0E,0x11,0x10,0x17,0x11,0x11,0x0F], [0x11,0x11,0x11,0x1F,0x11,0x11,0x11],
    [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E], [0x07,0x02,0x02,0x02,0x02,0x12,0x0C],
    [0x11,0x12,0x14,0x18,0x14,0x12,0x11], [0x10,0x10,0x10,0x10,0x10,0x10,0x1F],
    [0x11,0x1B,0x15,0x15,0x11,0x11,0x11], [0x11,0x19,0x15,0x13,0x11,0x11,0x11],
    [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E], [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10],
    [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D], [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11],
    [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E], [0x1F,0x04,0x04,0x04,0x04,0x04,0x04],
    [0x11,0x11,0x11,0x11,0x11,0x11,0x0E], [0x11,0x11,0x11,0x11,0x11,0x0A,0x04],
    [0x11,0x11,0x11,0x15,0x15,0x15,0x0A], [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11],
    [0x11,0x11,0x0A,0x04,0x04,0x04,0x04], [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F],
    [0x06,0x08,0x08,0x0C,0x08,0x08,0x06], [0x04,0x04,0x04,0x04,0x04,0x04,0x04],
    [0x0C,0x02,0x02,0x06,0x02,0x02,0x0C], [0x00,0x00,0x08,0x15,0x02,0x00,0x00],
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00],
];

pub type Color = [u8; 4];

// ------------------------------------------------------------- widgets --

#[derive(Clone, Debug)]
pub enum WidgetKind {
    Button { label: String, value: String, enabled: bool },
    Slider { label: String, value: f32 },
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
        let t = ((px - self.x - 8) as f32 / (self.w - 16) as f32).clamp(0.0, 1.0);
        t
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
        kind: WidgetKind::Slider { label: label.to_string(), value: value.clamp(0.0, 1.0) },
    }
}

// widget id constants shared with game.rs
pub const ID_TITLE_PLAY: u16 = 1;
pub const ID_TITLE_OPTIONS: u16 = 2;
pub const ID_TITLE_QUIT: u16 = 3;
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

/// Title screen layout (quit button only exists on native).
pub fn layout_title(is_web: bool) -> Vec<Widget> {
    let mut v = vec![
        btn(ID_TITLE_PLAY, (UI_W as i32 - 320) / 2, 296, 320, "SINGLEPLAYER", "", true),
        btn(ID_TITLE_OPTIONS, (UI_W as i32 - 320) / 2, 352, 320, "OPTIONS...", "", true),
    ];
    if !is_web {
        v.push(btn(ID_TITLE_QUIT, (UI_W as i32 - 320) / 2, 408, 320, "QUIT GAME", "", true));
    }
    v
}

/// Options screen layout. Values are 0..1 for sliders (game.rs normalizes).
pub fn layout_options() -> Vec<Widget> {
    let col1 = 72;
    let col2 = 496;
    let w = 392;
    let rows = [62, 110, 158, 206, 254, 302, 350];
    vec![
        slider(ID_OPT_FOV, col1, rows[0], w, "FOV", 0.5),
        slider(ID_OPT_BRIGHT, col2, rows[0], w, "BRIGHTNESS", 0.1),
        slider(ID_OPT_SENS, col1, rows[1], w, "MOUSE SENSITIVITY", 0.45),
        slider(ID_OPT_VOL, col2, rows[1], w, "MASTER VOLUME", 0.7),
        slider(ID_OPT_RD, col1, rows[2], w, "RENDER DISTANCE", 0.4),
        btn(ID_OPT_SHADER, col2, rows[2], w, "SHADERS", "OFF", true),
        btn(ID_OPT_GRAPHICS, col1, rows[3], w, "GRAPHICS", "FANCY", true),
        btn(ID_OPT_SHADOWS, col2, rows[3], w, "SHADOWS", "ON", true),
        btn(ID_OPT_SMOOTH, col1, rows[4], w, "SMOOTH LIGHTING", "ON", true),
        btn(ID_OPT_UPSCALE, col2, rows[4], w, "UPSCALING", "OFF", true),
        btn(ID_OPT_CLOUDS, col1, rows[5], w, "CLOUDS", "ON", true),
        btn(ID_OPT_MAXFPS, col2, rows[5], w, "MAX FPS", "VSYNC", true),
        // §21: the music category rides its own slider (master still scales it)
        slider(ID_OPT_MUSIC, col1, rows[6], w, "MUSIC", 0.6),
        btn(ID_OPT_DONE, (UI_W as i32 - 320) / 2, 470, 320, "DONE", "", true),
    ]
}

pub fn layout_pause() -> Vec<Widget> {
    vec![
        btn(ID_PAUSE_BACK, (UI_W as i32 - 320) / 2, 208, 320, "BACK TO GAME", "", true),
        btn(ID_PAUSE_OPTIONS, (UI_W as i32 - 320) / 2, 264, 320, "OPTIONS...", "", true),
        btn(ID_PAUSE_QUIT, (UI_W as i32 - 320) / 2, 320, 320, "QUIT TO TITLE", "", true),
    ]
}

// ------------------------------------------------------------- canvas --

pub struct UiCanvas {
    pub px: Vec<u8>,
    pub dirty: bool,
}

impl UiCanvas {
    pub fn new() -> Self {
        UiCanvas {
            px: vec![0u8; UI_W * UI_H * 4],
            dirty: true,
        }
    }

    pub fn clear(&mut self) {
        self.px.iter_mut().for_each(|p| *p = 0);
        self.dirty = true;
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

    /// 5x7 text with 1px shadow, scale 1..8. Returns width drawn.
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: Color, scale: i32) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            let mut ch = ch as usize;
            if ch < 32 || ch > 126 {
                ch = '?' as usize;
            }
            if ch >= 'a' as usize && ch <= 'z' as usize {
                ch -= 32; // smallcaps look
            }
            let glyph = &FONT[ch - 32];
            for gy in 0..7i32 {
                for gx in 0..5i32 {
                    if glyph[gy as usize] & (1 << (4 - gx)) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let dx = cx + gx * scale + sx;
                                let dy = y + gy * scale + sy;
                                self.set(dx + 1, dy + 1, [0, 0, 0, c[3]]);
                                self.set(dx, dy, c);
                            }
                        }
                    }
                }
            }
            cx += 6 * scale;
        }
        cx - x
    }

    pub fn text_width(s: &str, scale: i32) -> i32 {
        s.chars().count() as i32 * 6 * scale
    }

    pub fn text_center(&mut self, y: i32, s: &str, c: Color, scale: i32) {
        let w = Self::text_width(s, scale);
        self.text((UI_W as i32 - w) / 2, y, s, c, scale);
    }

    /// Text with a 1px outline in all 8 directions (for logo / level number).
    pub fn text_outlined(&mut self, x: i32, y: i32, s: &str, c: Color, oc: Color, scale: i32) -> i32 {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
            self.text(x + dx * (scale / 4 + 1), y + dy * (scale / 4 + 1), s, oc, scale);
        }
        self.text(x, y, s, c, scale)
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
            WidgetKind::Button { label, value, enabled } => (label.clone(), value.clone(), *enabled),
            _ => return,
        };
        let body: Color = if enabled { [96, 96, 96, 235] } else { [70, 70, 70, 200] };
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
        let text_col: Color = if !enabled {
            [160, 160, 160, 255]
        } else if hover {
            [255, 255, 160, 255]
        } else {
            [240, 240, 240, 255]
        };
        let full = if value.is_empty() { label } else { format!("{}: {}", label, value) };
        let tw = Self::text_width(&full, 2);
        self.text(w.x + (w.w - tw) / 2, w.y + (w.h - 14) / 2, &full, text_col, 2);
    }

    /// Minecraft-style slider: inset track + knob.
    pub fn draw_slider(&mut self, w: &Widget, hover: bool) {
        let (label, value) = match &w.kind {
            WidgetKind::Slider { label, value } => (label.clone(), *value),
            _ => return,
        };
        let ty = w.y + 8;
        let th = w.h - 16;
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
        // label centered over the track
        let text_col: Color = if hover { [255, 255, 160, 255] } else { [240, 240, 240, 255] };
        let tw = Self::text_width(&label, 2);
        self.text(w.x + (w.w - tw) / 2, w.y + (w.h - 14) / 2 - 1, &label, text_col, 2);
    }

    pub fn draw_widget(&mut self, w: &Widget, hover: bool) {
        match w.kind {
            WidgetKind::Button { .. } => self.draw_button(w, hover),
            WidgetKind::Slider { .. } => self.draw_slider(w, hover),
        }
    }

    pub fn draw_widgets(&mut self, ws: &[Widget], hover: Option<u16>) {
        for w in ws {
            self.draw_widget(w, hover == Some(w.id));
        }
    }

    // ----------------------------------------------------- screens ----

    /// Title screen overlay (drawn over the panorama).
    pub fn title_screen(&mut self, splash: &str, ws: &[Widget], hover: Option<u16>, time: f32) {
        // dim band behind logo area so it reads over bright panorama
        self.rect(0, 30, UI_W as i32, 120, [0, 0, 0, 60]);
        // logo with outline + drop shadow
        let scale = 7;
        let logo = "VOXELCRAFT";
        let lw = Self::text_width(logo, scale);
        let lx = (UI_W as i32 - lw) / 2;
        let ly = 52;
        // soft drop shadow
        self.text(lx + 3, ly + 4, logo, [0, 0, 0, 160], scale);
        // dark outline pass
        self.text_outlined(lx, ly, logo, [235, 235, 235, 255], [42, 42, 42, 255], scale);
        self.text_center(ly + 58, "A 1.16.5-STYLE VOXEL ENGINE", [200, 200, 200, 255], 1);

        // splash: yellow, pulsing, tucked at the logo's right
        let pulse = 0.5 + 0.5 * (time * 3.2).sin();
        let alpha = (150.0 + 105.0 * pulse) as u8;
        let sw = Self::text_width(splash, 2);
        let sx = (lx + lw - sw / 2).min(UI_W as i32 - sw - 8).max(8);
        self.text_outlined(sx, ly + 44, splash, [255, 255, 60, alpha], [60, 50, 0, alpha], 2);

        self.draw_widgets(ws, hover);

        self.text(8, UI_H as i32 - 20, "VoxelCraft 2.0_beta (Rust + wgpu)", [220, 220, 220, 255], 1);
        let vr = "100% PROCEDURAL — NO MOJANG ASSETS";
        let vw = Self::text_width(vr, 1);
        self.text(UI_W as i32 - vw - 8, UI_H as i32 - 20, vr, [210, 210, 210, 255], 1);
    }

    pub fn options_screen(&mut self, ws: &[Widget], hover: Option<u16>, sub: &str) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [8, 8, 10, 110]);
        self.text_center(18, "OPTIONS", [255, 255, 255, 255], 3);
        let sw = Self::text_width(sub, 1);
        self.text((UI_W as i32 - sw) / 2, 46, sub, [150, 150, 150, 255], 1);
        self.draw_widgets(ws, hover);
    }

    pub fn pause_screen(&mut self, ws: &[Widget], hover: Option<u16>) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [0, 0, 0, 130]);
        self.text_center(140, "GAME MENU", [255, 255, 255, 255], 3);
        self.draw_widgets(ws, hover);
    }

    // -------------------------------------------------------- HUD ----

    /// Vanilla-style crosshair: white plus with dark outline.
    pub fn crosshair(&mut self) {
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

    const HEART: [&'static str; 6] = [
        ".OO..OO.",
        "ORROORRO",
        "ORHRRRRO",
        "ORRRRRRO",
        ".ORRRRO.",
        "..ORRO..",
    ];

    const FOOD: [&'static str; 7] = [
        ".OOOO...",
        "OMMMMO..",
        "OMMMMO..",
        "OMMMMO..",
        ".OMMO...",
        "..OWO...",
        "...OO...",
    ];

    /// Full 1.16.5-style status bars: hearts (left), hunger (right), XP bar.
    pub fn status_bars(&mut self, health: f32, food: f32, xp: f32, level: u32) {
        const HB: Color = [20, 20, 20, 255]; // hotbar base coords
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

        // XP bar
        let xp_w = hb_w;
        let xp_x = hb_x;
        let xp_y = hb_y - 10;
        self.rect(xp_x, xp_y, xp_w, 8, [16, 16, 16, 220]);
        self.frame(xp_x, xp_y, xp_w, 8, [60, 60, 60, 255]);
        let fill = ((xp_w - 4) as f32 * xp.clamp(0.0, 1.0)) as i32;
        if fill > 0 {
            self.rect(xp_x + 2, xp_y + 2, fill, 4, [128, 255, 32, 255]);
            self.rect(xp_x + 2, xp_y + 2, fill, 1, [190, 255, 130, 255]);
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

    /// 1.16.5-style hotbar: 40px slots, big white selection frame, icons
    /// and vanilla stack counts (bottom-right, shadowed).
    pub fn hotbar(&mut self, slots: &[ItemStack], selected: usize, atlas: &[u8], item_name: Option<(&str, u8)>) {
        let n = slots.len() as i32;
        let slot = 40i32;
        let bw = n * slot + 4;
        let x0 = (UI_W as i32 - bw) / 2;
        let y0 = UI_H as i32 - 48;
        self.rect(x0, y0, bw, 44, [12, 12, 12, 190]);
        self.frame(x0, y0, bw, 44, [8, 8, 8, 255]);
        for i in 0..n as usize {
            let sx = x0 + 2 + i as i32 * slot;
            let sy = y0 + 2;
            self.rect(sx, sy, 36, 36, [58, 58, 58, 160]);
            self.frame(sx, sy, 36, 36, [90, 90, 90, 220]);
            self.draw_stack(&slots[i], sx, sy, atlas);
        }
        // selection: chunky white frame extending past the slot
        let sel = x0 + 2 + selected as i32 * slot;
        self.frame(sel - 2, y0, 40, 40, [255, 255, 255, 255]);
        self.frame(sel - 3, y0 - 1, 42, 42, [200, 200, 200, 140]);

        if let Some((name, alpha)) = item_name {
            let w = name.len() as i32 * 12;
            self.text((UI_W as i32 - w) / 2, y0 - 76, name, [255, 255, 255, alpha], 2);
        }
    }

    /// one item stack inside a 36px slot at (sx, sy): icon + vanilla count
    /// label (bottom-right, dark shadow) — shared by hotbar + containers.
    fn draw_stack(&mut self, s: &ItemStack, sx: i32, sy: i32, atlas: &[u8]) {
        let b = s.block;
        if b != AIR && s.count > 0 {
            let tile = {
                let d = def(b);
                if b == GRASS || b == OAK_LOG { d.tiles[2] } else { d.tiles[0] }
            };
            blit_tile(atlas, tile, 2, (sx + 2) as usize, (sy + 2) as usize, &mut self.px, UI_W);
        }
        if s.count > 1 {
            let label = s.count.to_string();
            let w = label.len() as i32 * 6;
            let tx = sx + 34 - w;
            let ty = sy + 27;
            self.text(tx + 1, ty + 1, &label, [0, 0, 0, 190], 1);
            self.text(tx, ty, &label, [255, 255, 255, 255], 1);
        }
    }

    /// container slot: recessed 36px well + optional stack
    fn slot_well(&mut self, x: i32, y: i32, s: &ItemStack, atlas: &[u8]) {
        self.rect(x, y, 36, 36, [52, 52, 52, 200]);
        self.frame(x, y, 36, 36, [24, 24, 24, 255]); // inner shadow
        self.frame(x + 1, y + 1, 34, 34, [110, 110, 110, 255]);
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
        let rows_on = [
            "  f  ",
            " fFf ",
            " fFf ",
            "fFFFf",
            "fFFFf",
        ];
        let rows_off = [
            "  .  ",
            " . . ",
            " . . ",
            ".....",
            ".....",
        ];
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
            ".  b  .",
            ".  b  .",
            ".  b  .",
            ".  b  .",
            ".  b  .",
            ".  b  .",
            ".  b  .",
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
            self.rect(x + 2 * scale, y_fill, scale, lit * scale, [200, 230, 255, 255]);
        }
    }


    /// one generic 9-wide slot row (hotbar strip or storage row)
    #[allow(dead_code)]
    fn inv_row(&mut self, x0: i32, y: i32, slots: &[ItemStack], atlas: &[u8], start: usize, count: usize) {
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
    ) -> ContainerGeom {
        let kind = view.kind;
        // ---- shared bottom layout: 9-col storage (3 rows) + hotbar row ----
        let cols: i32 = 9;
        let grid_w = cols * 40 + 4;
        let x0 = (UI_W as i32 - grid_w) / 2;
        // top-area height per kind
        let top_h = match kind {
            ContainerKind::Inventory => 96,   // 2x2 craft + arrow + output
            ContainerKind::Crafting => 140,   // 3x3 craft + arrow + output
            ContainerKind::Furnace => 128,    // input / flame / fuel + arrow + output
            ContainerKind::Brewing => 150,   // ingredient / bubbles / fuel + 3 bottles
        };
        let panel_h = top_h + 3 * 44 + 8 + 44 + 30; // + title + gaps + padding
        let y0 = (UI_H as i32 - panel_h) / 2;

        // panel chrome
        let px0 = x0 - 14;
        let pw = grid_w + 28;
        self.rect(px0, y0 - 30, pw, panel_h + 30, [26, 26, 30, 235]);
        self.frame(px0, y0 - 30, pw, panel_h + 30, [60, 60, 66, 255]);
        self.frame(px0 + 1, y0 - 29, pw - 2, panel_h + 28, [12, 12, 14, 255]);

        let title = match kind {
            ContainerKind::Inventory => "INVENTORY  (E / ESC to close)",
            ContainerKind::Crafting => "CRAFTING TABLE",
            ContainerKind::Furnace => "FURNACE",
            ContainerKind::Brewing => "BREWING STAND",
        };
        self.text(px0 + 12, y0 - 24, title, [255, 220, 120, 255], 1);

        let mut geom = ContainerGeom {
            inv: Vec::with_capacity(36),
            craft: Vec::new(),
            craft_out: (i32::MIN, i32::MIN),
            furnace: None,
            brewing: None,
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
                self.arrow(cx + 84, cy + 12, if !view.craft_out.is_empty() { 1.0 } else { 0.0 });
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
                self.arrow(cx + 124, cy + 32, if !view.craft_out.is_empty() { 1.0 } else { 0.0 });
                let ox = cx + 174;
                let oy = cy + 22;
                self.slot_well(ox, oy, &view.craft_out, atlas);
                geom.craft_out = (ox, oy);
            }
            ContainerKind::Furnace => {
                // left column: input above flame above fuel; arrow → output
                let cx = x0 + (grid_w - 240) / 2;
                let cy = y0 + 8;
                let (input, fuel, output, burn, cook) = view.furnace
                    .map(|f| (f.0, f.1, f.2, f.3, f.4))
                    .unwrap_or((ItemStack::EMPTY, ItemStack::EMPTY, ItemStack::EMPTY, 0.0, 0.0));
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
                let (ing, fuel, bottles, fuel_frac, brew_frac) = view.brewing
                    .map(|b| (b.0, b.1, b.2, b.3, b.4))
                    .unwrap_or((
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

        // hover label: name of the hovered slot's block
        if let Some(s) = view.hovered_stack(cursor_pos.0 as i32, cursor_pos.1 as i32, &geom) {
            if !s.is_empty() {
                let label = name(s.block);
                let lw = Self::text_width(label, 1);
                self.text(
                    (UI_W as i32 - lw) / 2,
                    y0 - 44,
                    label,
                    [255, 255, 255, 255],
                    1,
                );
            }
        }
        geom
    }

    pub fn debug(&mut self, lines: &[String]) {
        let mut max_w = 0i32;
        for l in lines {
            max_w = max_w.max(Self::text_width(l, 1) + 8);
        }
        let line_h = 14;
        self.rect(4, 4, max_w, lines.len() as i32 * line_h + 6, [80, 80, 80, 110]);
        for (i, l) in lines.iter().enumerate() {
            self.text(7, 7 + i as i32 * line_h, l, [235, 235, 235, 255], 1);
        }
    }

    /// Sodium-style rolling frame-time graph under the F3 text block.
    /// `times_ms` = last N frame times; green bars, 50 ms scale, 2 px/bar.
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
        self.rect(x0, y, w + 4, h + 4, [80, 80, 80, 110]);
        // 16.7 ms guide line (60 fps target)
        let guide_y = y + 2 + h - ((16.7f32 / 50.0) * h as f32) as i32;
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
            let color: Color = if *t <= 20.0 {
                [60, 220, 90, 230]
            } else if *t <= 40.0 {
                [240, 200, 40, 230]
            } else {
                [235, 70, 50, 230]
            };
            for dy in 0..th {
                self.px_set(x, y + 2 + h - 1 - dy, color);
                self.px_set(x + 1, y + 2 + h - 1 - dy, color);
            }
        }
    }

    /// Creative-style block picker (E key): centered grid of every placeable
    /// block; click → assigns to the selected hotbar slot. Returns the grid
    /// geometry so game.rs can hit-test clicks.
    pub fn picker(&mut self, cursor: (f32, f32), atlas: &[u8]) -> PickerGeom {
        let blocks = &PICKER_BLOCKS;
        let cols = 8;
        let cell = 44i32;
        let rows = (blocks.len() + cols - 1) / cols;
        let grid_w = cols as i32 * cell + 8;
        let grid_h = rows as i32 * cell + 8 + 22;
        let x0 = (UI_W as i32 - grid_w) / 2;
        let y0 = (UI_H as i32 - grid_h) / 2;

        self.rect(x0 - 6, y0 - 26, grid_w + 12, grid_h + 32, [16, 16, 16, 210]);
        self.frame(x0 - 6, y0 - 26, grid_w + 12, grid_h + 32, [70, 70, 70, 255]);
        self.text(
            x0 - 6 + 10,
            y0 - 24,
            "SELECT BLOCK  (B / ESC to close)",
            [230, 230, 230, 255],
            1,
        );

        let mut hovered: Option<u8> = None;
        for (i, b) in blocks.iter().enumerate() {
            let col = (i % cols) as i32;
            let row = (i / cols) as i32;
            let sx = x0 + 4 + col * cell;
            let sy = y0 + 4 + row * cell;
            self.rect(sx, sy, 40, 40, [58, 58, 58, 170]);
            self.frame(sx, sy, 40, 40, [90, 90, 90, 220]);
            let tile = def(*b).tiles[0];
            blit_tile(atlas, tile, 2, (sx + 4) as usize, (sy + 4) as usize, &mut self.px, UI_W);
            // hover highlight
            let cx = cursor.0 as i32;
            let cy = cursor.1 as i32;
            if cx >= sx && cx < sx + 40 && cy >= sy && cy < sy + 40 {
                self.frame(sx - 1, sy - 1, 42, 42, [255, 255, 255, 255]);
                hovered = Some(*b);
            }
        }

        // hovered block name on a bottom strip
        let label = hovered.map(name).unwrap_or("");
        let lw = Self::text_width(label, 1);
        self.text(x0 + 4, y0 + grid_h - 18, label, [255, 255, 255, 255], 1);
        let _ = lw;

        PickerGeom { x0, y0, cell, cols }
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
        self.text(x0 + 16, y0 + 10, "CONTROLS (H to close)", [255, 220, 120, 255], 2);
        for (i, (k, v)) in lines.iter().enumerate() {
            self.text(x0 + 16, y0 + 44 + i as i32 * 20, k, [160, 220, 160, 255], 1);
            self.text(x0 + 180, y0 + 44 + i as i32 * 20, v, [220, 220, 220, 255], 1);
        }
    }

    pub fn center_msg(&mut self, title: &str, sub: &str) {
        self.text_center(UI_H as i32 / 2 - 40, title, [255, 255, 255, 255], 3);
        self.text_center(UI_H as i32 / 2, sub, [200, 200, 200, 255], 1);
    }

    pub fn vignette_loading(&mut self, msg: &str, progress: f32) {
        self.rect(0, 0, UI_W as i32, UI_H as i32, [10, 12, 16, 120]);
        self.text_center(UI_H as i32 / 2 - 20, "VOXELCRAFT", [255, 255, 255, 255], 4);
        self.text_center(UI_H as i32 / 2 + 30, msg, [210, 210, 210, 255], 1);
        let bw = 360;
        let x0 = (UI_W as i32 - bw) / 2;
        let y0 = UI_H as i32 / 2 + 60;
        self.frame(x0, y0, bw, 12, [255, 255, 255, 200]);
        self.rect(x0 + 2, y0 + 2, ((bw - 4) as f32 * progress.clamp(0.0, 1.0)) as i32, 8, [110, 200, 90, 255]);
    }
}

/// hit-test geometry for the picker grid (UI-space), returned by
/// `UiCanvas::picker` so game.rs can map clicks to picker slots.
pub struct PickerGeom {
    pub x0: i32,
    pub y0: i32,
    pub cell: i32,
    pub cols: usize,
}

impl PickerGeom {
    /// which picker slot (if any) is under this UI-space cursor position
    pub fn slot_at(&self, ux: i32, uy: i32) -> Option<usize> {
        let dx = ux - (self.x0 + 4);
        let dy = uy - (self.y0 + 4);
        if dx < 0 || dy < 0 {
            return None;
        }
        let col = dx / self.cell;
        let row = dy / self.cell;
        if col >= self.cols as i32 || dx % self.cell >= 40 || dy % self.cell >= 40 {
            return None;
        }
        let idx = row as usize * self.cols + col as usize;
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
}

/// a logical slot in a container screen — the target of a mouse click
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotRef {
    /// player inventory slot (0..36; 0..9 = hotbar row)
    Inv(usize),
    /// crafting-grid cell (row-major; 4 cells for 2×2, 9 for 3×3)
    Craft(usize),
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
    /// stack riding the mouse cursor
    pub cursor: ItemStack,
}

impl ContainerView {
    fn hovered_stack(&self, x: i32, y: i32, geom: &ContainerGeom) -> Option<ItemStack> {
        Some(match geom.slot_at(x, y)? {
            SlotRef::Inv(i) => self.inv[i],
            SlotRef::Craft(i) => self.grid[i],
            SlotRef::CraftOut => self.craft_out,
            SlotRef::FurnaceInput => self.furnace?.0,
            SlotRef::FurnaceFuel => self.furnace?.1,
            SlotRef::FurnaceOutput => self.furnace?.2,
            SlotRef::BrewIngredient => self.brewing?.0,
            SlotRef::BrewFuel => self.brewing?.1,
            SlotRef::BrewBottle(i) => self.brewing?.2[i],
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
