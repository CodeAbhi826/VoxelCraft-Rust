//! `gui_render.rs` — GPU textured-quad renderer for GUI chrome
//! (UI-overhaul Phase 2).
//!
//! Draws widget chrome (buttons/slots/panels), HUD sprites (hearts,
//! hunger, armor, bubbles), hotbar chrome and the options dirt
//! background as textured quads in screen space, AFTER the software
//! `UiCanvas` blit. The canvas keeps text, item icons and every other
//! pixel — this pass only replaces chrome, and only when the game
//! enables it (`GuiRenderConfig`).
//!
//! Geometry: all quads live in the 960x540 UI pixel space and ride the
//! SAME letterbox uniform the canvas blit uses (`ui_buf`), so quads and
//! canvas pixels stay perfectly aligned at any window size/aspect.
//! Sprite art is painted at true texture size (9x9 … 182x22) and
//! scaled 2x here — the engine's established UI scale (hotbar 364x44 =
//! 2x the vanilla 182x22 logical frame).
//!
//! The WGSL follows the repo's inline-const convention in render.rs
//! (there is no .wgsl-file aggregator for builtin shaders; shaders.rs
//! is the shader-PACK api — substitution noted in the worklog).

use crate::gui::set::GuiTextureSet;
use crate::ui::{Widget, WidgetKind};

// ------------------------------------------------------------- types --

/// A rectangle in UI pixels (960x540 logical space) — `i32` to match
/// the canvas coordinate type (Section 3 note).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

/// Which sheet a quad samples (or `Solid` = plain tint).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum QuadTexture {
    Widgets,
    Hearts,
    Hunger,
    Armor,
    Bubbles,
    Hotbar,
    HotbarSel,
    Dirt,
    /// ignore texture, use tint only (1x1 white binding)
    Solid,
}

/// One textured quad. `src` is in TEXTURE pixels (integer source rect);
/// `dst` is in UI pixels; `tint` multiplies the sampled texel; `z` is
/// the ascending draw order (the pass draws in submission order).
#[derive(Copy, Clone, Debug)]
pub struct GuiQuad {
    pub texture: QuadTexture,
    pub dst: Rect,
    pub src: Rect,
    pub tint: [f32; 4],
    pub z: f32,
}

/// Typed GUI renderer failures (no panics — G2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiError {
    BufferAlloc,
    PipelineCreate,
    TooManyQuads,
}

/// The per-frame quad list — built by the canvas draw methods
/// (button/slot/panel/HUD chrome) alongside their raster work.
#[derive(Clone, Debug, Default)]
pub struct GuiFrame {
    pub quads: Vec<GuiQuad>,
}

/// Master switches for the migration (spec D5):
/// * `quads_enabled` — the GPU pass draws chrome quads
/// * `chrome_in_canvas` — the software canvas ALSO rasterizes chrome
///
/// Shipping default after Phase 2: quads on, canvas chrome off. During
/// A/B development both may be true (chrome double-draws, quads on top).
#[derive(Copy, Clone, Debug)]
pub struct GuiRenderConfig {
    pub quads_enabled: bool,
    pub chrome_in_canvas: bool,
}

impl Default for GuiRenderConfig {
    fn default() -> Self {
        GuiRenderConfig {
            quads_enabled: true,
            chrome_in_canvas: false,
        }
    }
}

// -------------------------------------------------- Section 2 values --
// The engine's UI scale: sprite art at true vanilla texture size is
// rendered at 2x — 9x9 sprites draw 18x18, 18x18 slots draw 36x36,
// the 182x22 hotbar draws 364x44 (the repo's established chrome
// geometry — verified against the existing canvas layout constants).

/// UI-scale factor: repo chrome = 2x vanilla logical pixels
pub const GUI_SCALE: i32 = 2;

/// VERIFIED https://minecraft.wiki — button chrome 20x20 9-slice source
/// with 4-px corners (2-px bevels inset 1 px inside the 1-px outlines).
const BTN_SRC: i32 = 20;
const BTN_CORNER: i32 = 4;
/// VERIFIED https://minecraft.wiki — hover overlay #FFFFFF at alpha
/// 51 (= 0.2).
const HOVER_TINT: [f32; 4] = [1.0, 1.0, 1.0, 51.0 / 255.0];
/// VERIFIED https://minecraft.wiki — slot 18x18 (rendered 36x36),
/// panel 20x20 9-slice.
const SLOT_SRC: i32 = 18;
/// HUD sprites 9x9 → 18x18.
const HUD_SRC: i32 = 9;
const HUD_DST: i32 = 18;
/// VERIFIED https://minecraft.wiki — hotbar background 182x22,
/// selection frame 24x22.
const HOTBAR_BG_SRC: i32 = 182;
const HOTBAR_BG_DST: i32 = 364;
const HOTBAR_SEL_SRC: i32 = 24;
const HOTBAR_SEL_DST: i32 = 48;
/// VERIFIED https://minecraft.wiki — options background tiles a 16x16
/// dirt tile at 0.25 brightness (darkness baked into the sprite).
const DIRT_SRC: i32 = 16;
const DIRT_DST: i32 = 32;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

impl GuiFrame {
    /// reset for a new frame
    pub fn clear(&mut self) {
        self.quads.clear();
    }

    fn push(&mut self, texture: QuadTexture, dst: Rect, src: Rect, tint: [f32; 4]) {
        let z = self.quads.len() as f32;
        self.quads.push(GuiQuad {
            texture,
            dst,
            src,
            tint,
            z,
        });
    }

    /// Widget chrome for a button: 9-slice of the matching variant
    /// (Normal/Hover/Disabled) plus the verified hover overlay quad
    /// when hovered and enabled.
    pub fn button(&mut self, w: &Widget, hover: bool) {
        let enabled = match &w.kind {
            WidgetKind::Button { enabled, .. } => *enabled,
            _ => true,
        };
        let cell = if !enabled {
            2 // ButtonDisabled
        } else if hover {
            1 // ButtonHover
        } else {
            0 // ButtonNormal
        };
        let base_x = cell * BTN_SRC;
        self.nine_slice(
            QuadTexture::Widgets,
            w.x,
            w.y,
            w.w,
            w.h,
            base_x,
            0,
            BTN_SRC,
            BTN_CORNER * GUI_SCALE,
            WHITE,
        );
        // VERIFIED chrome: hover overlay #FFFFFF at alpha 51
        if hover && enabled {
            self.push(
                QuadTexture::Solid,
                Rect::new(w.x, w.y, w.w, w.h),
                Rect::new(0, 0, 1, 1),
                HOVER_TINT,
            );
        }
    }

    /// Slider chrome: inset track (panel-tinted) + knob quad.
    pub fn slider(&mut self, w: &Widget, hover: bool) {
        let _ = hover;
        // track: the button body darkened, inset like the canvas version
        let ty = w.y + w.h / 2 - 5 * GUI_SCALE;
        let th = 10 * GUI_SCALE;
        self.nine_slice(
            QuadTexture::Widgets,
            w.x,
            ty,
            w.w,
            th,
            0,
            0,
            BTN_SRC,
            BTN_CORNER * GUI_SCALE,
            [0.55, 0.55, 0.55, 1.0],
        );
        // knob: 8x16 vanilla-sized handle (16x32 at our scale) from the
        // panel cell (light chrome)
        let value = match &w.kind {
            WidgetKind::Slider { value, .. } => *value,
            _ => 0.5,
        };
        let knob_w = 8 * GUI_SCALE;
        let kx = w.x + ((w.w - knob_w) as f32 * value.clamp(0.0, 1.0)) as i32;
        self.push(
            QuadTexture::Widgets,
            Rect::new(kx, w.y + w.h / 2 - 8 * GUI_SCALE, knob_w, 16 * GUI_SCALE),
            Rect::new(5 * 20, 4, 10, 12), // panel cell center strip
            WHITE,
        );
    }

    /// Text-field chrome: recessed panel 9-slice at the widget rect.
    pub fn text_field(&mut self, w: &Widget, hover: bool) {
        let _ = hover;
        self.nine_slice(
            QuadTexture::Widgets,
            w.x,
            w.y,
            w.w,
            w.h,
            5 * 20, // Panel cell
            0,
            BTN_SRC,
            BTN_CORNER * GUI_SCALE,
            [0.7, 0.7, 0.7, 1.0],
        );
    }

    /// One inventory slot: 18x18 chrome (rendered 36x36), hover variant
    /// when requested.
    pub fn slot(&mut self, x: i32, y: i32, hover: bool) {
        let cell = if hover { 4 } else { 3 }; // SlotHover / SlotEmpty
        self.push(
            QuadTexture::Widgets,
            Rect::new(x, y, SLOT_SRC * GUI_SCALE, SLOT_SRC * GUI_SCALE),
            Rect::new(cell * 20 + 1, 1, SLOT_SRC, SLOT_SRC),
            WHITE,
        );
    }

    /// Container panel 9-slice from the Panel cell.
    pub fn panel(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.nine_slice(
            QuadTexture::Widgets,
            x,
            y,
            w,
            h,
            5 * 20, // Panel cell
            0,
            BTN_SRC,
            BTN_CORNER * GUI_SCALE,
            WHITE,
        );
    }

    /// HUD sprites — 9x9 sources drawn 18x18.
    pub fn heart(&mut self, x: i32, y: i32, variant: crate::textures::gui_art::HeartVariant) {
        let tile = match variant {
            crate::textures::gui_art::HeartVariant::Empty => 0,
            crate::textures::gui_art::HeartVariant::Full => 1,
            crate::textures::gui_art::HeartVariant::Half => 2,
        };
        self.push(
            QuadTexture::Hearts,
            Rect::new(x, y, HUD_DST, HUD_DST),
            Rect::new(tile * HUD_SRC, 0, HUD_SRC, HUD_SRC),
            WHITE,
        );
    }

    pub fn hunger(&mut self, x: i32, y: i32, variant: crate::textures::gui_art::HungerVariant) {
        let tile = match variant {
            crate::textures::gui_art::HungerVariant::Empty => 0,
            crate::textures::gui_art::HungerVariant::Full => 1,
            crate::textures::gui_art::HungerVariant::Half => 2,
        };
        self.push(
            QuadTexture::Hunger,
            Rect::new(x, y, HUD_DST, HUD_DST),
            Rect::new(tile * HUD_SRC, 0, HUD_SRC, HUD_SRC),
            WHITE,
        );
    }

    pub fn armor(&mut self, x: i32, y: i32, variant: crate::textures::gui_art::ArmorVariant) {
        let tile = match variant {
            crate::textures::gui_art::ArmorVariant::Empty => 0,
            crate::textures::gui_art::ArmorVariant::Full => 1,
            crate::textures::gui_art::ArmorVariant::Half => 2,
        };
        self.push(
            QuadTexture::Armor,
            Rect::new(x, y, HUD_DST, HUD_DST),
            Rect::new(tile * HUD_SRC, 0, HUD_SRC, HUD_SRC),
            WHITE,
        );
    }

    pub fn bubble(&mut self, x: i32, y: i32, variant: crate::textures::gui_art::BubbleVariant) {
        let tile = match variant {
            crate::textures::gui_art::BubbleVariant::Full => 0,
            crate::textures::gui_art::BubbleVariant::Gone => 1,
        };
        self.push(
            QuadTexture::Bubbles,
            Rect::new(x, y, HUD_DST, HUD_DST),
            Rect::new(tile * HUD_SRC, 0, HUD_SRC, HUD_SRC),
            WHITE,
        );
    }

    /// Hotbar background: one stretched 182x22 quad (364x44).
    pub fn hotbar_background(&mut self, x: i32, y: i32) {
        self.push(
            QuadTexture::Hotbar,
            Rect::new(x, y, HOTBAR_BG_DST, 22 * GUI_SCALE),
            Rect::new(0, 0, HOTBAR_BG_SRC, 22),
            WHITE,
        );
    }

    /// Hotbar selection frame: 24x22 quad (48x44) centered on the slot.
    pub fn hotbar_selection(&mut self, x: i32, y: i32) {
        self.push(
            QuadTexture::HotbarSel,
            Rect::new(x, y, HOTBAR_SEL_DST, 22 * GUI_SCALE),
            Rect::new(0, 0, HOTBAR_SEL_SRC, 22),
            WHITE,
        );
    }

    /// Options-screen dirt background: 16x16 tiles at 32px, full-canvas.
    /// The 0.25 brightness is baked into the sprite (tint stays white).
    pub fn dirt_background(&mut self, w: i32, h: i32) {
        let ty = (h as usize).div_ceil(DIRT_DST as usize) as i32;
        let tx = (w as usize).div_ceil(DIRT_DST as usize) as i32;
        for y in 0..ty {
            for x in 0..tx {
                self.push(
                    QuadTexture::Dirt,
                    Rect::new(x * DIRT_DST, y * DIRT_DST, DIRT_DST, DIRT_DST),
                    Rect::new(0, 0, DIRT_SRC, DIRT_SRC),
                    WHITE,
                );
            }
        }
    }

    /// 9-slice a `src`-sized cell (at `sx, sy` in the widgets sheet)
    /// into the dst rect, corners at `corner` dst px. Degenerate pieces
    /// (zero width/height) collapse away — a 1:1-scale dst emits fewer
    /// than 9 quads.
    // 11 params: the 9-slice takes the full src+dst geometry — this is
    // the chrome vocabulary, positional reads best; silenced deliberately.
    #[allow(clippy::too_many_arguments)]
    fn nine_slice(
        &mut self,
        tex: QuadTexture,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
        sx: i32,
        sy: i32,
        s_size: i32,
        corner_dst: i32,
        tint: [f32; 4],
    ) {
        // source corner size (texture px)
        let c_src = BTN_CORNER;
        // clamp: dst smaller than 2x corners -> single stretched quad
        if dw < corner_dst * 2 || dh < corner_dst * 2 {
            self.push(tex, Rect::new(dx, dy, dw, dh), Rect::new(sx, sy, s_size, s_size), tint);
            return;
        }
        let mid_w = dw - corner_dst * 2;
        let mid_h = dh - corner_dst * 2;
        let s_mid = s_size - c_src * 2;
        for row in 0..3 {
            for col in 0..3 {
                let (dw_, dh_) = match (row, col) {
                    (0, 0) | (0, 2) | (2, 0) | (2, 2) => (corner_dst, corner_dst),
                    (0, 1) | (2, 1) => (mid_w, corner_dst),
                    (1, 0) | (1, 2) => (corner_dst, mid_h),
                    _ => (mid_w, mid_h),
                };
                let (sw, sh) = match (row, col) {
                    (0, 0) | (0, 2) | (2, 0) | (2, 2) => (c_src, c_src),
                    (0, 1) | (2, 1) => (s_mid, c_src),
                    (1, 0) | (1, 2) => (c_src, s_mid),
                    _ => (s_mid, s_mid),
                };
                let ox = match col {
                    0 => 0,
                    1 => corner_dst,
                    _ => corner_dst + mid_w,
                };
                let oy = match row {
                    0 => 0,
                    1 => corner_dst,
                    _ => corner_dst + mid_h,
                };
                let sox = match col {
                    0 => 0,
                    1 => c_src,
                    _ => c_src + s_mid,
                };
                let soy = match row {
                    0 => 0,
                    1 => c_src,
                    _ => c_src + s_mid,
                };
                if dw_ > 0 && dh_ > 0 {
                    self.push(
                        tex,
                        Rect::new(dx + ox, dy + oy, dw_, dh_),
                        Rect::new(sx + sox, sy + soy, sw, sh),
                        tint,
                    );
                }
            }
        }
    }
}

// ------------------------------------------------------------ GPU side --

/// The GPU quad pass: one pipeline (same blend as the UI blit), one
/// dynamic vertex buffer, per-sheet bind groups sharing the canvas's
/// letterbox uniform buffer. Owned by `Renderer` as `Option<GuiRenderer>`
/// — `None` until `set_gui_textures` runs, and the game then keeps the
/// canvas chrome on (self-healing fallback).
pub struct GuiRenderer {
    pipe: wgpu::RenderPipeline,
    /// bind groups indexed by QuadTexture discriminant order
    bind_groups: [Option<wgpu::BindGroup>; 9],
    sheet_dims: [(u32, u32); 9],
    vb: Option<wgpu::Buffer>,
    ib: wgpu::Buffer,
    capacity: u32,
    vertex_staging: Vec<GuiVertex>,
}

/// One quad corner: UI-pixel position, UV, tint.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct GuiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    tint: [f32; 4],
}

/// hard cap per frame (dirt bg ~510 + widgets; generous margin)
const MAX_QUADS: usize = 8192;

impl GuiRenderer {
    /// Build the pipeline. `ui_bgl`/`ui_buf` come from the Renderer —
    /// sharing the letterbox uniform keeps quads aligned with the canvas.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_bgl: &wgpu::BindGroupLayout,
        out_format: wgpu::TextureFormat,
    ) -> Result<Self, GuiError> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gui-quad"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(GUI_QUAD_SHADER)),
        });
        let vbl = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GuiVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gui-pl"),
            bind_group_layouts: &[ui_bgl],
            push_constant_ranges: &[],
        });
        let pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gui-pipe"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[vbl],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: out_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        // static index buffer: 6 indices per quad, pattern 0,1,2,2,3,0
        const MAX_INDICES: u32 = MAX_QUADS as u32 * 6;
        let mut indices = Vec::with_capacity(MAX_INDICES as usize);
        for q in 0..MAX_QUADS as u32 {
            let b = q * 4;
            indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
        }
        let ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gui-ib"),
            size: MAX_INDICES as u64 * 4,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&ib, 0, bytemuck::cast_slice(&indices));

        Ok(GuiRenderer {
            pipe,
            bind_groups: std::array::from_fn(|_| None),
            sheet_dims: [(1, 1); 9],
            vb: None,
            ib,
            capacity: 0,
            vertex_staging: Vec::new(),
        })
    }

    /// Upload every sprite sheet and create the per-sheet bind groups
    /// (plus the 1x1 white `Solid` binding). Shared sampler + the
    /// Renderer's letterbox uniform.
    pub fn set_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_bgl: &wgpu::BindGroupLayout,
        ui_buf: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
        set: &GuiTextureSet,
    ) -> Result<(), GuiError> {
        let sheets: [(&crate::gui::set::SpriteSheet, QuadTexture); 8] = [
            (&set.widgets, QuadTexture::Widgets),
            (&set.hearts, QuadTexture::Hearts),
            (&set.hunger, QuadTexture::Hunger),
            (&set.armor, QuadTexture::Armor),
            (&set.bubbles, QuadTexture::Bubbles),
            (&set.hotbar_bg, QuadTexture::Hotbar),
            (&set.hotbar_sel, QuadTexture::HotbarSel),
            (&set.dirt, QuadTexture::Dirt),
        ];
        for (sheet, tex) in sheets {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("gui-sheet"),
                size: wgpu::Extent3d {
                    width: sheet.w as u32,
                    height: sheet.h as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            if sheet.px.len() != sheet.w * sheet.h * 4 {
                return Err(GuiError::PipelineCreate);
            }
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &sheet.px,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(sheet.w as u32 * 4),
                    rows_per_image: Some(sheet.h as u32),
                },
                wgpu::Extent3d {
                    width: sheet.w as u32,
                    height: sheet.h as u32,
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gui-bg"),
                layout: ui_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: ui_buf,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });
            let idx = tex as usize;
            self.bind_groups[idx] = Some(bg);
            self.sheet_dims[idx] = (sheet.w as u32, sheet.h as u32);
        }
        // Solid: 1x1 opaque white
        let white = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gui-solid"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &white,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let view = white.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gui-solid-bg"),
            layout: ui_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: ui_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        self.bind_groups[QuadTexture::Solid as usize] = Some(bg);
        Ok(())
    }

    /// clear the staging vertices (start of frame)
    pub fn begin(&mut self) {
        self.vertex_staging.clear();
    }

    /// append one quad's 4 vertices (positions in UI px, UVs from src /
    /// sheet dims). Returns TooManyQuads past the cap.
    pub fn push(&mut self, q: &GuiQuad) -> Result<(), GuiError> {
        if self.vertex_staging.len() / 4 >= MAX_QUADS {
            return Err(GuiError::TooManyQuads);
        }
        let idx = q.texture as usize;
        let (tw, th) = self.sheet_dims[idx];
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;
        let u0 = q.src.x as f32 / tw;
        let v0 = q.src.y as f32 / th;
        let u1 = (q.src.x + q.src.w) as f32 / tw;
        let v1 = (q.src.y + q.src.h) as f32 / th;
        let x0 = q.dst.x as f32;
        let y0 = q.dst.y as f32;
        let x1 = (q.dst.x + q.dst.w) as f32;
        let y1 = (q.dst.y + q.dst.h) as f32;
        let corners = [
            ([x0, y0], [u0, v0]),
            ([x1, y0], [u1, v0]),
            ([x1, y1], [u1, v1]),
            ([x0, y1], [u0, v1]),
        ];
        for (pos, uv) in corners {
            self.vertex_staging.push(GuiVertex {
                pos,
                uv,
                tint: q.tint,
            });
        }
        Ok(())
    }

    /// upload the staging vertices (growing the buffer on demand) and
    /// record the draw into `pass`, grouped by texture to minimize bind
    /// group switches.
    pub fn draw<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'a>,
        quads: &[GuiQuad],
    ) -> Result<(), GuiError> {
        if quads.is_empty() {
            return Ok(());
        }
        self.begin();
        let mut groups: Vec<(QuadTexture, u32, u32)> = Vec::new(); // (tex, first quad, count)
        for (i, q) in quads.iter().enumerate() {
            match groups.last_mut() {
                Some((tex, _, count)) if *tex == q.texture => {
                    *count += 1;
                }
                _ => groups.push((q.texture, i as u32, 1)),
            }
            self.push(q)?;
        }
        let needed = self.vertex_staging.len() as u64;
        if needed == 0 {
            return Ok(());
        }
        if self.vb.is_none() || (self.capacity as u64) < needed {
            let new_cap = needed.next_power_of_two().max(1024);
            self.vb = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("gui-vb"),
                size: new_cap * std::mem::size_of::<GuiVertex>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.capacity = new_cap as u32;
        }
        let vb = match &self.vb {
            Some(b) => b,
            None => return Err(GuiError::BufferAlloc),
        };
        queue.write_buffer(
            vb,
            0,
            bytemuck::cast_slice(&self.vertex_staging),
        );
        pass.set_pipeline(&self.pipe);
        pass.set_index_buffer(self.ib.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, vb.slice(..));
        for (tex, first, count) in groups {
            let bg = match &self.bind_groups[tex as usize] {
                Some(bg) => bg,
                None => continue, // sheet not uploaded — skip cleanly
            };
            pass.set_bind_group(0, bg, &[]);
            let first_v = first * 4;
            let n_idx = count * 6;
            pass.draw_indexed(
                first_v..first_v + n_idx,
                0,
                0..1,
            );
        }
        Ok(())
    }
}

/// The GUI quad shader — samples the sheet, multiplies by vertex tint,
/// premultiplied-style output over the same letterbox uniform the
/// canvas blit uses.
const GUI_QUAD_SHADER: &str = r#"
struct UiU { map: vec4<f32> };
@group(0) @binding(0) var gui_tex: texture_2d<f32>;
@group(0) @binding(1) var gui_samp: sampler;
@group(0) @binding(2) var<uniform> U: UiU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) px: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(px.x * U.map.x + U.map.y, px.y * U.map.z + U.map.w, 0.0, 1.0);
    out.uv = uv;
    out.tint = tint;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(gui_tex, gui_samp, in.uv);
    let a = c.a * in.tint.a;
    if (a < 0.004) { discard; }
    return vec4<f32>(c.rgb * in.tint.rgb, a);
}
"#;

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{btn_h, UiCanvas, UI_H, UI_W};

    #[test]
    fn button_produces_chrome_quads_matching_spec_dims() {
        // a 400x40 button (vanilla 200x20 at 2x): exactly 9 9-slice
        // quads, union = the widget rect
        let w = btn_h(1, 280, 100, 400, 40, "Singleplayer", "", true);
        let mut f = GuiFrame::default();
        f.button(&w, false);
        assert_eq!(f.quads.len(), 9, "3x3 9-slice");
        let x0 = f.quads.iter().map(|q| q.dst.x).min();
        let y0 = f.quads.iter().map(|q| q.dst.y).min();
        let x1 = f
            .quads
            .iter()
            .map(|q| q.dst.x + q.dst.w)
            .max();
        let y1 = f
            .quads
            .iter()
            .map(|q| q.dst.y + q.dst.h)
            .max();
        assert_eq!(
            (x0, y0, x1, y1),
            (
                Some(280),
                Some(100),
                Some(280 + 400),
                Some(100 + 40)
            )
        );
        // all from the widgets sheet, plain tint
        assert!(f.quads.iter().all(|q| q.texture == QuadTexture::Widgets));
        assert!(f.quads.iter().all(|q| q.tint == WHITE));
    }

    #[test]
    fn button_quad_count_is_width_independent() {
        // 9-slice at various widths: same quad count (9)
        for width in [40usize, 80, 200, 400, 600] {
            let w = btn_h(1, 0, 0, width as i32, 40, "OK", "", true);
            let mut f = GuiFrame::default();
            f.button(&w, false);
            assert_eq!(f.quads.len(), 9, "width {width}");
        }
    }

    #[test]
    fn hovered_button_adds_the_verified_overlay_tint() {
        let w = btn_h(1, 0, 0, 400, 40, "Multiplayer", "", true);
        let mut f = GuiFrame::default();
        f.button(&w, true);
        // 9 slice + 1 overlay
        assert_eq!(f.quads.len(), 10);
        let overlay = f.quads.last().cloned();
        if let Some(q) = overlay {
            assert_eq!(q.texture, QuadTexture::Solid);
            assert_eq!(q.dst, Rect::new(0, 0, 400, 40));
            // VERIFIED chrome: hover overlay #FFFFFF at alpha 51
            assert_eq!(q.tint, [1.0, 1.0, 1.0, 51.0 / 255.0]);
        } else {
            panic!("missing overlay quad");
        }
    }

    #[test]
    fn disabled_button_uses_the_disabled_cell() {
        let w = btn_h(1, 0, 0, 400, 40, "Grayed", "", false);
        let mut f = GuiFrame::default();
        f.button(&w, false);
        // cell 2 = ButtonDisabled: every src rect inside x 40..60
        assert!(f.quads.iter().all(|q| q.src.x >= 40 && q.src.x + q.src.w <= 60));
        // no hover overlay on disabled
        assert!(!f.quads.iter().any(|q| q.texture == QuadTexture::Solid));
    }

    #[test]
    fn slot_quad_is_36x36_from_the_18x18_cell() {
        let mut f = GuiFrame::default();
        f.slot(10, 20, false);
        assert_eq!(f.quads.len(), 1);
        let q = f.quads[0];
        assert_eq!(q.dst, Rect::new(10, 20, 36, 36));
        // SlotEmpty = cell 3, sprite at (61, 1) 18x18 inside the cell
        assert_eq!(q.src, Rect::new(3 * 20 + 1, 1, 18, 18));
        f.slot(10, 20, true);
        assert_eq!(f.quads[1].src.x, 4 * 20 + 1, "SlotHover = cell 4");
    }

    #[test]
    fn hud_sprites_draw_18x18_from_9x9_tiles() {
        use crate::textures::gui_art::*;
        let mut f = GuiFrame::default();
        f.heart(1, 2, HeartVariant::Full);
        f.hunger(3, 4, HungerVariant::Half);
        f.armor(5, 6, ArmorVariant::Empty);
        f.bubble(7, 8, BubbleVariant::Gone);
        for (q, tex) in [
            (f.quads[0], QuadTexture::Hearts),
            (f.quads[1], QuadTexture::Hunger),
            (f.quads[2], QuadTexture::Armor),
            (f.quads[3], QuadTexture::Bubbles),
        ] {
            assert_eq!(q.texture, tex);
            assert_eq!(q.dst.w, 18);
            assert_eq!(q.dst.h, 18);
            assert_eq!(q.src.w, 9);
            assert_eq!(q.src.h, 9);
        }
        // variant tile picks: Full=1, Half=2, Empty=0, Gone=1
        assert_eq!(f.quads[0].src.x, 9);
        assert_eq!(f.quads[1].src.x, 18);
        assert_eq!(f.quads[2].src.x, 0);
        assert_eq!(f.quads[3].src.x, 9);
    }

    #[test]
    fn dirt_background_tiles_32px_across_the_canvas() {
        let mut f = GuiFrame::default();
        f.dirt_background(UI_W as i32, UI_H as i32);
        // 960/32 = 30 columns, 540/32 = 16.875 -> 17 rows
        assert_eq!(f.quads.len(), 30 * 17);
        assert!(f.quads.iter().all(|q| q.dst.w == 32 && q.dst.h == 32));
        assert!(f.quads.iter().all(|q| q.texture == QuadTexture::Dirt));
        // plain tint: the 0.25 brightness lives in the sprite
        assert!(f.quads.iter().all(|q| q.tint == WHITE));
        // covers the full canvas
        let max_x = f.quads.iter().map(|q| q.dst.x + q.dst.w).max();
        let max_y = f.quads.iter().map(|q| q.dst.y + q.dst.h).max();
        assert_eq!(max_x, Some(960));
        assert_eq!(max_y, Some(544));
    }

    #[test]
    fn hotbar_chrome_matches_2x_vanilla_dims() {
        let mut f = GuiFrame::default();
        f.hotbar_background(298, 492);
        f.hotbar_selection(298, 490);
        assert_eq!(f.quads[0].dst, Rect::new(298, 492, 364, 44));
        assert_eq!(f.quads[0].src, Rect::new(0, 0, 182, 22));
        assert_eq!(f.quads[1].dst, Rect::new(298, 490, 48, 44));
        assert_eq!(f.quads[1].src, Rect::new(0, 0, 24, 22));
    }

    #[test]
    fn quads_carry_ascending_z_order() {
        let mut f = GuiFrame::default();
        f.panel(0, 0, 100, 100);
        f.slot(10, 10, false);
        f.heart(0, 0, crate::textures::gui_art::HeartVariant::Full);
        let zs: Vec<f32> = f.quads.iter().map(|q| q.z).collect();
        let mut sorted = zs.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        assert_eq!(zs, sorted);
    }

    #[test]
    fn chrome_suppression_flag_gates_raster_not_quads() {
        // D2 contract: set_chrome_enabled(false) stops the canvas chrome
        // raster; the quad frame is still populated
        let mut ui = UiCanvas::new();
        assert!(ui.chrome_enabled, "default is true (A2 back-compat)");
        let w = btn_h(7, 100, 100, 400, 40, "Buttons", "", true);
        ui.draw_button(&w, false);
        let quads_with_chrome = ui.gui_frame.quads.len();
        assert!(quads_with_chrome >= 9, "quads pushed alongside raster");
        // raster painted the body color at a mid-left body pixel (clear
        // of the centered label text)
        let (cx, cy) = (140usize, 121usize);
        let idx = (cy * crate::ui::UI_W + cx) * 4;
        assert!(ui.px[idx + 3] != 0, "raster chrome drawn while enabled");

        ui.set_chrome_enabled(false);
        ui.clear();
        ui.draw_button(&w, false);
        assert_eq!(ui.gui_frame.quads.len(), quads_with_chrome);
        // raster chrome suppressed: body pixel stays clear
        let idx = (cy * crate::ui::UI_W + cx) * 4;
        assert_eq!(ui.px[idx + 3], 0, "raster chrome suppressed");
    }
}
