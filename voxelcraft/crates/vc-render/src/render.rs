//! wgpu renderer: Vulkan/DX12/Metal/WebGPU via a single code path.
//! Pipelines: sky (gradient + sun/moon/stars), terrain (alpha-test, smooth
//! light), water (blend + waves), selection wireframe, UI (bitmap canvas).

use crate::draw::{self, ChunkGpu, DrawCmd, IndirectArgs, MeshSlot, SlotAlloc, VisEntry};
use crate::textures;
use crate::ui::{UiCanvas, UI_H, UI_W};
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;
use vc_mesh::mesh::{MeshData, Vertex};
use vc_world::world::ChunkPos;
use wgpu::util::DeviceExt;

// ---------------------------------------------------------------- uniforms

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    cam: [f32; 4],
    fog_color: [f32; 4], // rgb + fog_start
    sun_dir: [f32; 4],   // xyz + fog_end
    misc: [f32; 4],      // day_light, time, underwater, _
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UiUniform {
    map: [f32; 4], // (a, b, c, d): ndc.x = x*a+b, ndc.y = y*c+d
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LineUniform {
    vp: [[f32; 4]; 4],
    offset: [f32; 4],
    color: [f32; 4],
}

/// post uniforms: p = (mode, menu_blur, time, aspect), q = (bloom, vig, sat, exposure)
/// s = (rcas_amount, _, _, _) — FSR 1.0 RCAS lobe scale (1.0 = FsrRcasCon(0) max)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    p: [f32; 4],
    q: [f32; 4],
    s: [f32; 4],
}

/// FSR 1.0 EASU size constants: (src_w, src_h, dst_w, dst_h) — the
/// FsrEasuCon() setup, inlined into the shader as pp = uv*src - 0.5
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct EasuUniform {
    con: [f32; 4],
}

/// blur direction + texel step for the separable blur passes
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AuxUniform {
    dir: [f32; 4],
}

/// shadow-map globals: light view-projection + params
/// params = (enabled, strength, fade_start, fade_end)
/// size = (map_px, _, _, _) — §17 quality (1024/2048/4096)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowGlobals {
    shadow_vp: [[f32; 4]; 4],
    params: [f32; 4],
    size: [f32; 4],
}

/// offscreen scene + bloom pyramid textures (recreated on resize)
struct PostTargets {
    scene: wgpu::Texture,
    scene_view: wgpu::TextureView,
    /// FSR 1.0 EASU output — FULL surface resolution (the composite + RCAS
    /// read this; scene stays at the internal render scale)
    up: wgpu::Texture,
    up_view: wgpu::TextureView,
    /// shader-pack composite handoff target (full res, LINEAR) — the engine
    /// composite writes here when a pack stage is active, the pack pass
    /// then writes the srgb surface (Phase 11 §34)
    pack: wgpu::Texture,
    pack_view: wgpu::TextureView,
    /// bright-pass output (1/4 res)
    q: wgpu::Texture,
    q_view: wgpu::TextureView,
    /// blur ping (1/8 res)
    b1: wgpu::Texture,
    b1_view: wgpu::TextureView,
    /// blur pong (1/8 res)
    b2: wgpu::Texture,
    b2_view: wgpu::TextureView,
}

impl PostTargets {
    fn new(
        device: &wgpu::Device,
        w: u32,
        h: u32,
        _format: wgpu::TextureFormat,
        scale: f32,
    ) -> Self {
        // LINEAR (Unorm) intermediates: the scene shaders output linear
        // color; the post chain reads/writes raw linear values and the
        // final composite encodes once into the srgb surface. This avoids
        // srgb texture sampling in the post chain entirely (a real-world
        // WebGL2/SwiftShader srgb-sampling corruption hit the washed-out
        // gray sky).
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let make = |w: u32, h: u32| {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("post"),
                size: wgpu::Extent3d {
                    width: w.max(1),
                    height: h.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let v = t.create_view(&wgpu::TextureViewDescriptor::default());
            (t, v)
        };
        // FSR 1.0 (§33): scene + bloom pyramid live at the internal render
        // scale; the EASU pass upscales scene → `up` at FULL surface
        // resolution, and the composite (with RCAS) reads `up`.
        let sw = ((w as f32) * scale).round().max(1.0) as u32;
        let sh = ((h as f32) * scale).round().max(1.0) as u32;
        let (scene, scene_view) = make(sw, sh);
        let (up, up_view) = make(w.max(1), h.max(1));
        let (pack, pack_view) = make(w.max(1), h.max(1));
        let (q, q_view) = make(sw / 4, sh / 4);
        let (b1, b1_view) = make(sw / 8, sh / 8);
        let (b2, b2_view) = make(sw / 8, sh / 8);
        PostTargets {
            scene,
            scene_view,
            up,
            up_view,
            pack,
            pack_view,
            q,
            q_view,
            b1,
            b1_view,
            b2,
            b2_view,
        }
    }

    /// internal (scaled) size of the scene target
    fn scene_size(&self) -> (u32, u32) {
        (self.scene.width(), self.scene.height())
    }
}

pub struct Camera {
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

pub struct SkyState {
    pub day_light: f32,
    pub sun_dir: Vec3,
    pub fog_color: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
    pub time: f32,
    pub underwater: bool,
    /// minimum light floor (brightness setting) — G.misc.w in shaders
    pub min_light: f32,
    /// §28: skip the sky/sun entirely — the Nether's fog-colored clear IS
    /// the sky (no gradient, no sun disc, no clouds)
    pub skyless: bool,
}

/// Post-processing request for a frame.
pub struct PostParams {
    /// 0 = off, 1 = vanilla+ (bloom/vig/sat), 2 = cinematic (+chroma, ACES)
    pub mode: u8,
    /// menu/panorama background blur 0..1
    pub menu_blur: f32,
    /// sun shadow strength 0..1 (0 disables the shadow pass entirely)
    pub shadows: f32,
    /// RCAS sharpen amount for the FSR-lite upscale (0 = off)
    pub sharpen: f32,
}

#[derive(Default)]
pub struct RenderStats {
    pub chunks: u32,
    pub tris: u32,
    /// particles drawn this frame
    pub particles: u32,
    /// draw_indexed-family API calls issued this frame (all passes) —
    /// the Phase-9 metric: loop path = drawn chunks, MDI path = region runs
    pub draws: u32,
    /// buffer binds issued for chunk drawing (§14/§37 diagnostics)
    pub binds: u32,
    /// Phase 6 §26: frustum-visible chunks removed by the occlusion flood
    pub culled: u32,
}

// ---------------------------------------------------------------- shaders

const TERRAIN_SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    misc: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;
@group(0) @binding(3) var shadow_tex: texture_2d<f32>;
@group(0) @binding(4) var shadow_samp: sampler;
@group(0) @binding(5) var<uniform> SH: ShadowG;
// §18 biome tint LUT (row = kind, col = slot; textureLoad is legal with
// per-vertex indices on every backend — incl. Vulkan's UBO rule)
@group(0) @binding(6) var tint_tex: texture_2d<f32>;

struct ShadowG {
    shadow_vp: mat4x4<f32>,
    // x = enabled, y = strength, zw = distance fade start/end
    params: vec4<f32>,
    // x = shadow map size in px (1024/2048/4096 — §17 quality)
    size: vec4<f32>,
};

fn unpackShadowDepth(c: vec4<f32>) -> f32 {
    return c.r + c.g / 255.0;
}

/// 3x3 PCF shadow test. Uses textureLoad (not textureSample) so it is legal
/// inside non-uniform control flow and works on every backend (incl. WebGL2).
/// Normal-offset pushes the sample point out of the surface to kill acne.
fn sampleShadow(world: vec3<f32>, nrm: vec3<f32>) -> f32 {
    if (SH.params.x < 0.5) { return 0.0; }
    let n = select(-nrm, nrm, dot(nrm, G.cam.xyz - world) >= 0.0);
    let sp = world + n * 0.10;
    let clip = SH.shadow_vp * vec4<f32>(sp, 1.0);
    let uv = clip.xy * 0.5 + 0.5;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) { return 0.0; }
    let d = clip.z;
    let ts = vec2<i32>(i32(SH.size.x), i32(SH.size.x));
    let base = vec2<i32>(uv * SH.size.xy);
    var acc = 0.0;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            let c = vec2<i32>(base.x + dx, base.y + dy);
            let s = unpackShadowDepth(textureLoad(shadow_tex, c, 0));
            acc = acc + select(1.0, 0.0, d <= s + 0.0006);
        }
    }
    var sh = acc / 9.0;
    let dist = distance(world, G.cam.xyz);
    sh = sh * (1.0 - smoothstep(SH.params.z, SH.params.w, dist));
    return sh;
}

// ---- VC-16 packed-vertex decode (bit layout documented in mesh.rs) ----
fn vc16_pos(v: vec4<u32>, origin: vec2<f32>) -> vec3<f32> {
    let x = f32(v.x & 0xFFFFu) / 2048.0 - 8.0;
    let z = f32(v.x >> 16u) / 2048.0 - 8.0;
    let y = f32(v.y & 0xFFFFu) / 128.0;
    return vec3<f32>(origin.x + x, y, origin.y + z);
}
fn face_shade(n: u32) -> f32 {
    if (n == 2u) { return 1.0; }       // +Y
    if (n == 3u) { return 0.5; }       // -Y
    if (n == 4u || n == 5u) { return 0.8; } // ±Z
    if (n == 6u) { return 0.85; }      // cross plants
    return 0.6;                        // ±X
}
fn ao_factor(a: u32) -> f32 {
    if (a == 0u) { return 0.42; }
    if (a == 1u) { return 0.62; }
    if (a == 2u) { return 0.80; }
    return 1.0;
}

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec2<f32>,
    @location(3) light: f32,
    @location(4) sky: f32,
    @location(5) block: f32,
    @location(6) tintcol: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) v_data: vec4<u32>,
    @location(1) origin: vec2<f32>,
) -> VsOut {
    let flags = v_data.y >> 16u;
    let nrm = flags & 7u;
    let ao = (flags >> 3u) & 3u;
    let tile_i = (v_data.z >> 18u) & 0x3FFFu;
    let uv = vec2<f32>(f32((v_data.z >> 10u) & 0xFFu), f32((v_data.z >> 2u) & 0xFFu)) / 16.0;
    let tile = vec2<f32>(f32(tile_i % 16u), f32(tile_i / 16u));
    let sky = f32((v_data.w >> 4u) & 0xFu) / 15.0;
    let block = f32(v_data.w & 0xFu) / 15.0;
    let world = vc16_pos(v_data, origin);
    // biome tint (§18): resolve the packed index HERE — the COLOR
    // interpolates across the face, the index must not
    let tint = (v_data.w >> 8u) & 0xFFu;
    let tintcol = select(
        vec3<f32>(1.0, 1.0, 1.0),
        textureLoad(tint_tex, vec2<i32>(i32(tint & 63u), i32(tint >> 6u)), 0).rgb,
        tint != 0u,
    );
    var out: VsOut;
    out.pos = G.view_proj * vec4<f32>(world, 1.0);
    out.world = world;
    out.uv = uv;
    out.tile = tile;
    out.light = face_shade(nrm) * ao_factor(ao);
    out.sky = sky;
    out.block = block;
    out.tintcol = tintcol;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // ---- atlas tile-safety: half-texel inset + analytic gradients ----
    // Two seam bugs lived here (the "textures connect/bleed" artifact):
    // (1) BOUNDARY BLEED: fract(uv) spans the tile exactly, so bilinear/
    //     mipmap/aniso filtering near tile edges sampled the NEIGHBORING
    //     atlas tile's texels. The half-texel inset (0.5/16 tile = 0.03125)
    //     keeps every bilinear footprint inside the tile (vanilla's stitched
    //     atlas uses the same trick; NEAREST sampling is unaffected since
    //     texel centers survive a half-texel clamp).
    // (2) LOD EXPLOSION: fract() is discontinuous at every integer UV, so
    //     the implicit dpdx/dpdy the GPU derives for mip/aniso selection
    //     jump to ~the full tile width at every block boundary — the GPU
    //     picked the coarsest mip along every seam line (dark/blurry grid
    //     over the world) and aniso footprints streaked across tiles.
    //     Fix: sample with EXPLICIT gradients taken from the PRE-fract uv
    //     (fract' has derivative 1 a.e., so d(uv)/16 is the analytic
    //     gradient of the atlas coordinate everywhere except the seam
    //     itself — exactly what the LOD computation wants).
    // Deep-distance note: at mip 3/4 (2px/1px per tile) bilinear still
    // mixes neighboring tiles — same residual vanilla 1.16.5 has (the
    // reason its mipmap slider stops at 4); covered by fog at that range.
    let fuv = clamp(fract(in.uv), vec2<f32>(0.03125), vec2<f32>(0.96875));
    let tuv = (in.tile + fuv) / vec2<f32>(16.0, 16.0);
    let gdx = dpdx(in.uv) / vec2<f32>(16.0, 16.0);
    let gdy = dpdy(in.uv) / vec2<f32>(16.0, 16.0);
    let c = textureSampleGrad(atlas_tex, atlas_samp, tuv, gdx, gdy);
    if (c.a < 0.5) { discard; }
    let day = G.misc.x;
    // geometric face normal from derivatives (camera-facing sign)
    var nrm = normalize(cross(dpdx(in.world), dpdy(in.world)));
    let shadow = sampleShadow(in.world, nrm);
    let sun_factor = 1.0 - shadow * SH.params.y;
    // block light (glowstone etc.) is independent of day/night and shadows
    let dyn_l = in.sky * day * sun_factor;
    // G.misc.w = min-light floor (brightness setting, kills pitch-black caves)
    let sky_l = max(max(dyn_l, in.block), G.misc.w);
    var rgb = c.rgb * in.tintcol * in.light * sky_l;
    let d = distance(in.world, G.cam.xyz);
    let f = smoothstep(G.fog_color.w, G.sun_dir.w, d);
    rgb = mix(rgb, G.fog_color.rgb, f);
    if (G.misc.z > 0.5) {
        rgb = rgb * vec3<f32>(0.60, 0.78, 1.10);
    }
    return vec4<f32>(rgb, 1.0);
}
"#;

const WATER_SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    misc: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;
@group(0) @binding(3) var shadow_tex: texture_2d<f32>;
@group(0) @binding(4) var shadow_samp: sampler;
@group(0) @binding(5) var<uniform> SH: ShadowG;
// §18 biome tint LUT (water takes the biome water color)
@group(0) @binding(6) var tint_tex: texture_2d<f32>;

struct ShadowG {
    shadow_vp: mat4x4<f32>,
    // x = enabled, y = strength, zw = distance fade start/end
    params: vec4<f32>,
    // x = shadow map size in px (§17 quality)
    size: vec4<f32>,
};

fn unpackShadowDepth(c: vec4<f32>) -> f32 {
    return c.r + c.g / 255.0;
}

fn sampleShadow(world: vec3<f32>, nrm: vec3<f32>) -> f32 {
    if (SH.params.x < 0.5) { return 0.0; }
    let sp = world + nrm * 0.10;
    let clip = SH.shadow_vp * vec4<f32>(sp, 1.0);
    let uv = clip.xy * 0.5 + 0.5;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) { return 0.0; }
    let d = clip.z;
    let base = vec2<i32>(uv * SH.size.xy);
    var acc = 0.0;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            let c = vec2<i32>(base.x + dx, base.y + dy);
            let s = unpackShadowDepth(textureLoad(shadow_tex, c, 0));
            acc = acc + select(1.0, 0.0, d <= s + 0.0006);
        }
    }
    var sh = acc / 9.0;
    let dist = distance(world, G.cam.xyz);
    sh = sh * (1.0 - smoothstep(SH.params.z, SH.params.w, dist));
    return sh;
}

// ---- VC-16 packed-vertex decode (bit layout documented in mesh.rs) ----
fn vc16_pos(v: vec4<u32>, origin: vec2<f32>) -> vec3<f32> {
    let x = f32(v.x & 0xFFFFu) / 2048.0 - 8.0;
    let z = f32(v.x >> 16u) / 2048.0 - 8.0;
    let y = f32(v.y & 0xFFFFu) / 128.0;
    return vec3<f32>(origin.x + x, y, origin.y + z);
}
fn face_shade(n: u32) -> f32 {
    if (n == 2u) { return 1.0; }
    if (n == 3u) { return 0.5; }
    if (n == 4u || n == 5u) { return 0.8; }
    if (n == 6u) { return 0.85; }
    return 0.6;
}
fn ao_factor(a: u32) -> f32 {
    if (a == 0u) { return 0.42; }
    if (a == 1u) { return 0.62; }
    if (a == 2u) { return 0.80; }
    return 1.0;
}

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec2<f32>,
    @location(3) light: f32,
    @location(4) sky: f32,
    @location(5) block: f32,
    @location(6) tintcol: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) v_data: vec4<u32>,
    @location(1) origin: vec2<f32>,
) -> VsOut {
    let flags = v_data.y >> 16u;
    let nrm = flags & 7u;
    let ao = (flags >> 3u) & 3u;
    let tile_i = (v_data.z >> 18u) & 0x3FFFu;
    let uv = vec2<f32>(f32((v_data.z >> 10u) & 0xFFu), f32((v_data.z >> 2u) & 0xFFu)) / 16.0;
    let tile = vec2<f32>(f32(tile_i % 16u), f32(tile_i / 16u));
    let sky = f32((v_data.w >> 4u) & 0xFu) / 15.0;
    let block = f32(v_data.w & 0xFu) / 15.0;
    // §18 water tint: resolve the packed index in the vertex stage
    let tint = (v_data.w >> 8u) & 0xFFu;
    let tintcol = select(
        vec3<f32>(1.0, 1.0, 1.0),
        textureLoad(tint_tex, vec2<i32>(i32(tint & 63u), i32(tint >> 6u)), 0).rgb,
        tint != 0u,
    );
    var p = vc16_pos(v_data, origin);
    let is_top = abs(fract(p.y) - 0.875) < 0.01;
    let wob = sin(G.misc.y * 1.6 + p.x * 0.7 + p.z * 1.1) * 0.045
            + sin(G.misc.y * 1.1 + p.x * 1.9 - p.z * 0.6) * 0.025;
    p.y = p.y + select(0.0, wob, is_top);
    var out: VsOut;
    out.pos = G.view_proj * vec4<f32>(p, 1.0);
    out.world = p;
    out.uv = uv;
    out.tile = tile;
    out.light = face_shade(nrm) * ao_factor(ao);
    out.sky = sky;
    out.block = block;
    out.tintcol = tintcol;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // scroll + fract(uv + scroll): the scroll offset is uniform across the
    // surface (contributes ZERO to derivatives), so the analytic gradients
    // still come from the pre-fract `in.uv`. Same tile-safety treatment as
    // TERRAIN_SHADER: half-texel inset kills cross-tile bleed (the moving
    // seam line the scroll used to drag across the surface), explicit
    // gradients fix the LOD explosion at every fract discontinuity.
    let scroll = vec2<f32>(G.misc.y * 0.06, G.misc.y * 0.025);
    let fuv = clamp(fract(in.uv + scroll), vec2<f32>(0.03125), vec2<f32>(0.96875));
    let tuv = (in.tile + fuv) / vec2<f32>(16.0, 16.0);
    let gdx = dpdx(in.uv) / vec2<f32>(16.0, 16.0);
    let gdy = dpdy(in.uv) / vec2<f32>(16.0, 16.0);
    let c = textureSampleGrad(atlas_tex, atlas_samp, tuv, gdx, gdy);
    let day = G.misc.x;
    // water is a flat plane — the up normal is exact
    let shadow = sampleShadow(in.world, vec3<f32>(0.0, 1.0, 0.0));
    let sun_factor = 1.0 - shadow * SH.params.y;
    // block light is independent of day/night and shadows
    let dyn_l = in.sky * day * sun_factor;
    // G.misc.w = min-light floor (brightness setting)
    let sky_l = max(max(dyn_l, in.block), G.misc.w);
    var rgb = c.rgb * in.tintcol * in.light * sky_l * 1.05;
    let d = distance(in.world, G.cam.xyz);
    let f = smoothstep(G.fog_color.w, G.sun_dir.w, d);
    rgb = mix(rgb, G.fog_color.rgb, f);
    return vec4<f32>(rgb, 0.62);
}
"#;

const SKY_SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    misc: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    out.pos = vec4<f32>(p[vi], 1.0, 1.0);
    out.ndc = p[vi];
    return out;
}

fn hash3(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3(12.9898, 78.233, 37.719))) * 43758.5453);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let v = G.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let dir = normalize(v.xyz / v.w - G.cam.xyz);
    let day = G.misc.x;
    let sun = normalize(G.sun_dir.xyz);
    let y = clamp(dir.y, -1.0, 1.0);

    let zen_day = vec3(0.26, 0.46, 0.88);
    let hor_day = vec3(0.77, 0.86, 0.99);
    let zen_night = vec3(0.012, 0.018, 0.055);
    let hor_night = vec3(0.035, 0.045, 0.10);
    var zen = mix(zen_night, zen_day, day);
    var hor = mix(hor_night, hor_day, day);

    // sunset band near the horizon
    let ss = clamp(1.0 - abs(sun.y) * 4.0, 0.0, 1.0) * clamp(day * 1.8, 0.0, 1.0);
    hor = mix(hor, vec3(0.99, 0.55, 0.26), ss * 0.5);

    var col = mix(hor, zen, pow(clamp(y, 0.0, 1.0), 0.6));
    if (y < 0.0) {
        col = mix(hor, hor * 0.35, clamp(-y * 3.0, 0.0, 1.0));
    }

    // sun disc + glow
    let sdot = dot(dir, sun);
    let disc = smoothstep(0.99930, 0.99965, sdot);
    let glow = pow(clamp(sdot, 0.0, 1.0), 250.0) * 0.30;
    col += (disc * 1.35 + glow) * vec3(1.0, 0.95, 0.82) * clamp(day * 2.0, 0.0, 1.0);

    // moon
    let mdot = dot(dir, -sun);
    let mdisc = smoothstep(0.9993, 0.9997, mdot);
    col += mdisc * vec3(0.85, 0.9, 1.0) * clamp(1.0 - day, 0.0, 1.0) * 0.7;

    // stars
    let star_h = hash3(floor(dir * 260.0));
    let star = select(0.0, 1.0, star_h > 0.9972) * clamp(1.0 - day * 2.5, 0.0, 1.0);
    col += star * (0.55 + 0.45 * sin(G.misc.y * 4.0 + star_h * 60.0));

    // blend into fog at horizon
    let fogf = pow(1.0 - clamp(abs(dir.y) * 2.0, 0.0, 1.0), 2.5);
    col = mix(col, G.fog_color.rgb, clamp(fogf * 0.85, 0.0, 1.0));
    return vec4<f32>(col, 1.0);
}
"#;

const UI_SHADER: &str = r#"
struct UiU { map: vec4<f32> };
@group(0) @binding(0) var ui_tex: texture_2d<f32>;
@group(0) @binding(1) var ui_samp: sampler;
@group(0) @binding(2) var<uniform> U: UiU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) p: vec2<f32>, @location(1) uv: vec2<f32>) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(p.x * U.map.x + U.map.y, p.y * U.map.z + U.map.w, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(ui_tex, ui_samp, in.uv);
    if (c.a < 0.004) { discard; }
    return c;
}
"#;

const LINE_SHADER: &str = r#"
struct LineU {
    vp: mat4x4<f32>,
    offset: vec4<f32>,
    color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> L: LineU;

@vertex
fn vs_main(@location(0) p: vec3<f32>) -> @builtin(position) vec4<f32> {
    return L.vp * vec4<f32>(p + L.offset.xyz, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return L.color;
}
"#;

// ---------------------------------------------------------------- clouds --

const CLOUD_SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    misc: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var cloud_tex: texture_2d<f32>;
@group(0) @binding(2) var cloud_samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
};

@vertex
fn vs_main(@location(0) xz: vec2<f32>) -> VsOut {
    var out: VsOut;
    let world = vec3<f32>(xz.x + G.cam.x, 168.0, xz.y + G.cam.z);
    out.pos = G.view_proj * vec4<f32>(world, 1.0);
    out.world = world;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = (in.world.xz + vec2<f32>(G.misc.y * 6.0, G.misc.y * 2.5)) / vec2<f32>(512.0, 512.0);
    let c = textureSample(cloud_tex, cloud_samp, uv);
    let day = G.misc.x;
    var col = vec3<f32>(1.0, 1.0, 1.0) * (0.30 + 0.70 * day);
    // fade far clouds into the fog
    let d = distance(in.world, G.cam.xyz);
    let f = smoothstep(G.fog_color.w, G.sun_dir.w, d);
    col = mix(col, G.fog_color.rgb, f * 0.9);
    return vec4<f32>(col, c.a * 0.55);
}
"#;

/// Block particles (§16.2 pass 4: after the translucent water pass, before
/// clouds). Billboard quads built CPU-side against the camera basis — the
/// vertex carries ABSOLUTE atlas UVs (a random 4×4-px quarter of the block
/// tile) and a baked light × tint color.
const PARTICLE_SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    misc: vec4<f32>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec3<f32>,
) -> VsOut {
    var out: VsOut;
    out.pos = G.view_proj * vec4<f32>(pos, 1.0);
    out.world = pos;
    out.uv = uv;
    out.col = col;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(atlas_tex, atlas_samp, in.uv);
    if (c.a < 0.1) { discard; }
    var rgb = c.rgb * in.col;
    let d = distance(in.world, G.cam.xyz);
    let f = smoothstep(G.fog_color.w, G.sun_dir.w, d);
    rgb = mix(rgb, G.fog_color.rgb, f);
    return vec4<f32>(rgb, c.a);
}
"#;

// -------------------------------------------------------- post-processing --

// bright-pass (threshold + downsample to 1/4)
const BRIGHT_SHADER: &str = r#"
struct PostU { p: vec4<f32>, q: vec4<f32> };
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> U: PostU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    // CRITICAL: sampling a previously-rendered-to texture requires the NDC→UV
    // V-flip (NDC y=+1 = top of viewport = texture row 0 = v=0). Without it
    // the composited world renders upside-down while the UI (own mapping)
    // stays right-side up.
    out.uv = vec2<f32>(p[vi].x * 0.5 + 0.5, 0.5 - p[vi].y * 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, in.uv);
    let l = max(c.rgb - vec3<f32>(0.82), vec3<f32>(0.0));
    return vec4<f32>(l * 1.5, 1.0);
}
"#;

// separable 9-tap gaussian blur (direction from aux uniform)
const BLUR_SHADER: &str = r#"
struct PostU { p: vec4<f32>, q: vec4<f32> };
struct AuxU { dir: vec4<f32> };
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> U: PostU;
@group(0) @binding(3) var<uniform> A: AuxU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    // NDC→UV V-flip (see BRIGHT_SHADER note) — keeps the bloom pyramid
    // orientation-stable relative to the scene texture.
    out.uv = vec2<f32>(p[vi].x * 0.5 + 0.5, 0.5 - p[vi].y * 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let off = A.dir.xy;
    var acc = textureSample(tex, samp, in.uv).rgb * 0.2270270270;
    acc += (textureSample(tex, samp, in.uv + off * 1.3846153846).rgb
          + textureSample(tex, samp, in.uv - off * 1.3846153846).rgb) * 0.3162162162;
    acc += (textureSample(tex, samp, in.uv + off * 3.2307692308).rgb
          + textureSample(tex, samp, in.uv - off * 3.2307692308).rgb) * 0.0702702703;
    return vec4<f32>(acc, 1.0);
}
"#;

// composite: menu blur + bloom + grade + vignette + chroma + tonemap
const POST_SHADER: &str = r#"
struct PostU { p: vec4<f32>, q: vec4<f32>, s: vec4<f32> };
// p = (mode, menu_blur, time, aspect)  q = (bloom, vignette, saturation, exposure)
// s = (rcas_amount, _, _, _) — AMD FSR 1.0 RCAS lobe scale: 1.0 = maximum
// sharpness (FsrRcasCon(0)), 0 = off. RCAS runs on the EASU-upscaled image
// before the grade (AMD canonical ordering: EASU → RCAS → everything else).
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var bloom: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> U: PostU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    // NDC→UV V-flip (see BRIGHT_SHADER note) — this pass writes to the
    // SWAPCHAIN, so without the flip the whole world appears upside-down.
    out.uv = vec2<f32>(p[vi].x * 0.5 + 0.5, 0.5 - p[vi].y * 0.5);
    return out;
}

// ----------------------------------------------------- FSR 1.0 RCAS ----
// Faithful WGSL port of AMD FidelityFX FSR 1.0 "FsrRcasF" (ffx_fsr1.h,
// float path). 5-tap cross on the OUTPUT-resolution EASU image, with the
// canonical per-channel hit-limiters, peak-range clamping and the
// 4*lobe+1 normalization. `scene` here is the EASU-upscaled target.
// Port notes: exact division replaces ARcpF1 approximations (GPU-friendly
// in f32); denominators are epsilon-guarded so flat-black/flat-white
// neighborhoods can't produce NaN (the FP16 approximations returned large
// finite values; exact math needs the explicit guard).
const RCAS_LIMIT: f32 = 0.25 - (1.0 / 16.0);

fn rcasLoad(o: vec2<i32>) -> vec3<f32> {
    let dim = textureDimensions(scene);
    let p = clamp(o, vec2<i32>(0, 0), vec2<i32>(i32(dim.x) - 1, i32(dim.y) - 1));
    return textureLoad(scene, p, 0).rgb;
}

fn rcas(uv: vec2<f32>, lobeScale: f32) -> vec3<f32> {
    // Algorithm uses minimal 3x3 pixel neighborhood:
    //    b
    //  d e f
    //    h
    let dim = textureDimensions(scene);
    let ip = vec2<i32>(floor(uv * vec2<f32>(f32(i32(dim.x)), f32(i32(dim.y))) - vec2<f32>(0.5)));
    let b = rcasLoad(ip + vec2<i32>( 0, -1));
    let d = rcasLoad(ip + vec2<i32>(-1,  0));
    let e = rcasLoad(ip);
    let f = rcasLoad(ip + vec2<i32>( 1,  0));
    let h = rcasLoad(ip + vec2<i32>( 0,  1));
    // luma times 2 — NOT used in the default RCAS path (only the optional
    // FSR_RCAS_DENOISE gate needs it, which AMD ships disabled); the loads
    // stay out entirely (dead-code eliminated by the compiler)
    // min and max of the ring (per channel). NOTE: the optional
    // FSR_RCAS_DENOISE noise gate is NOT enabled — AMD's reference default
    // ships without it (their comment: better to add grain after RCAS).
    let mnR = min(min(b.r, d.r), min(f.r, h.r));
    let mnG = min(min(b.g, d.g), min(f.g, h.g));
    let mnB = min(min(b.b, d.b), min(f.b, h.b));
    let mxR = max(max(b.r, d.r), max(f.r, h.r));
    let mxG = max(max(b.g, d.g), max(f.g, h.g));
    let mxB = max(max(b.b, d.b), max(f.b, h.b));
    // limiters (epsilon-guarded exact rcp). hitMin: the all-black ring makes
    // 4*mx = 0 → guard the denominator (AMD's approx rcp returned large
    // finite; exact math would 0/0 → NaN). hitMax: the all-white ring makes
    // 4*mn−4 = 0 with numerator ≤ 0 → shortcut to 0 (the same value the
    // approximation produced); the normal path keeps AMD's exact signs.
    let hitMinR = min(mnR, e.r) / (4.0 * max(mxR, 1e-6));
    let hitMinG = min(mnG, e.g) / (4.0 * max(mxG, 1e-6));
    let hitMinB = min(mnB, e.b) / (4.0 * max(mxB, 1e-6));
    let hitMaxR = select((1.0 - max(mxR, e.r)) / (4.0 * mnR - 4.0), 0.0, mnR > 0.9999);
    let hitMaxG = select((1.0 - max(mxG, e.g)) / (4.0 * mnG - 4.0), 0.0, mnG > 0.9999);
    let hitMaxB = select((1.0 - max(mxB, e.b)) / (4.0 * mnB - 4.0), 0.0, mnB > 0.9999);
    let lobeR = max(-hitMinR, hitMaxR);
    let lobeG = max(-hitMinG, hitMaxG);
    let lobeB = max(-hitMinB, hitMaxB);
    var lobe = max(-RCAS_LIMIT, min(max(max(lobeR, lobeG), lobeB), 0.0)) * lobeScale;
    // resolve
    let rcpL = 1.0 / (4.0 * lobe + 1.0);
    return vec3<f32>(
        (lobe * b.r + lobe * d.r + lobe * h.r + lobe * f.r + e.r) * rcpL,
        (lobe * b.g + lobe * d.g + lobe * h.g + lobe * f.g + e.g) * rcpL,
        (lobe * b.b + lobe * d.b + lobe * h.b + lobe * f.b + e.b) * rcpL,
    );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let mode = U.p.x;
    let blur = U.p.y;
    var col: vec3<f32>;

    if (blur > 0.001) {
        // menu/panorama blur: 13-tap disc in uv space (resolution independent)
        var offs = array<vec2<f32>, 12>(
            vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0),
            vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0),
            vec2<f32>(0.7071, 0.7071), vec2<f32>(-0.7071, 0.7071),
            vec2<f32>(0.7071, -0.7071), vec2<f32>(-0.7071, -0.7071),
            vec2<f32>(1.0, 0.7071), vec2<f32>(-1.0, -0.7071),
            vec2<f32>(0.7071, 1.0), vec2<f32>(-0.7071, -1.0),
        );
        let r = blur * 0.012;
        var acc = textureSample(scene, samp, uv).rgb;
        for (var i = 0; i < 12; i = i + 1) {
            acc += textureSample(scene, samp, uv + offs[i] * r).rgb;
        }
        col = acc / 13.0;
    } else if (U.s.x > 0.001) {
        // FSR 1.0: RCAS sharpening of the EASU-upscaled image (AMD ordering:
        // EASU → RCAS → grade)
        col = rcas(uv, U.s.x);
        // cinematic chromatic aberration (a lens effect riding on top — the
        // shifted taps resample bilinearly, post-RCAS)
        if (mode > 1.5) {
            let d = (uv - vec2<f32>(0.5, 0.5)) * 0.004;
            col.r = textureSample(scene, samp, uv + d).r;
            col.b = textureSample(scene, samp, uv - d).b;
        }
    } else {
        col = textureSample(scene, samp, uv).rgb;
        if (mode > 1.5) {
            let d = (uv - vec2<f32>(0.5, 0.5)) * 0.004;
            col.r = textureSample(scene, samp, uv + d).r;
            col.b = textureSample(scene, samp, uv - d).b;
        }
    }

    // bloom
    if (U.q.x > 0.001) {
        col += textureSample(bloom, samp, uv).rgb * U.q.x;
    }

    // exposure
    col = col * U.q.w;

    // saturation
    let lum = dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
    col = mix(vec3<f32>(lum), col, U.q.z);

    // vignette
    let d2 = distance(uv, vec2<f32>(0.5, 0.5));
    col = col * (1.0 - U.q.y * smoothstep(0.35, 0.85, d2));

    // cinematic: ACES-ish filmic curve
    if (mode > 1.5) {
        col = clamp((col * (2.51 * col + 0.03)) / (col * (2.43 * col + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
    }

    return vec4<f32>(col, 1.0);
}
"#;

// ----------------------------------------------------------- FSR 1.0 EASU
// Faithful WGSL port of AMD FidelityFX FSR 1.0 "FsrEasuF" (ffx_fsr1.h,
// float path) — the 12-tap edge-adaptive spatial upsampling kernel.
// Reads the low-res scene target, writes the full-res upscaled target.
// Port notes:
//  - textureLoad replaces gather4 (same 12 texels: 4 gathers → 12 loads);
//  - exact rcp/rsqrt replace the FP16-speed approximations (APrxLoRcpF1 &c)
//    — division sites are epsilon-guarded so degenerate flat neighborhoods
//    cannot produce NaN (the approximations returned large finite values);
//  - the kernel is mathematically identity at 1:1 scaling (verified: the
//    center tap weight is 1 and all neighbors window to 0), so EASU runs
//    at every upscale setting including native.
// Uniform con = (src_w, src_h, dst_w, dst_h) — the FsrEasuCon() setup
// inlined: pp = (dst_px + 0.5) * src/dst - 0.5.
const EASU_SHADER: &str = r#"
struct EasuU { con: vec4<f32> };
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> U: EasuU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    // NDC→UV V-flip — same mapping as every fullscreen pass (BRIGHT_SHADER)
    out.uv = vec2<f32>(p[vi].x * 0.5 + 0.5, 0.5 - p[vi].y * 0.5);
    return out;
}

fn loadC(o: vec2<i32>) -> vec3<f32> {
    let dim = textureDimensions(tex);
    let p = clamp(o, vec2<i32>(0, 0), vec2<i32>(i32(dim.x) - 1, i32(dim.y) - 1));
    return textureLoad(tex, p, 0).rgb;
}

// luma × 2 — AMD's 2-FMA approximation of 2*(0.25R + 0.5G + 0.25B)
fn luma2(c: vec3<f32>) -> f32 {
    return c.b * 0.5 + (c.r * 0.5 + c.g);
}

// FsrEasuSetF: accumulate gradient direction + length from a '+'
// neighborhood, weighted by the bilinear corner weight of the quad the
// samples come from.
fn easuSet(dir: ptr<function, vec2<f32>>, len: ptr<function, f32>, w: f32,
           lA: f32, lB: f32, lC: f32, lD: f32, lE: f32) {
    //    a
    //  b c d
    //    e
    let dc = lD - lC;
    let cb = lC - lB;
    var lenX = max(abs(dc), abs(cb));
    lenX = 1.0 / max(lenX, 1e-8);
    let dirX = lD - lB;
    (*dir).x += dirX * w;
    lenX = clamp(abs(dirX) * lenX, 0.0, 1.0);
    lenX *= lenX;
    *len += lenX * w;
    // repeat for the y axis
    let ec = lE - lC;
    let ca = lC - lA;
    var lenY = max(abs(ec), abs(ca));
    lenY = 1.0 / max(lenY, 1e-8);
    let dirY = lE - lA;
    (*dir).y += dirY * w;
    lenY = clamp(abs(dirY) * lenY, 0.0, 1.0);
    lenY *= lenY;
    *len += lenY * w;
}

// FsrEasuTapF: one weighted tap — offset rotated into the gradient
// direction, anisotropically scaled, Lanczos-2-approximation window.
fn easuTap(aC: ptr<function, vec3<f32>>, aW: ptr<function, f32>,
           off: vec2<f32>, dir: vec2<f32>, len: vec2<f32>,
           lob: f32, clp: f32, c: vec3<f32>) {
    // rotate offset by direction
    var v: vec2<f32>;
    v.x = (off.x * dir.x) + (off.y * dir.y);
    v.y = (off.x * (-dir.y)) + (off.y * dir.x);
    // anisotropy
    v = v * len;
    // distance², limited to the window
    var d2 = v.x * v.x + v.y * v.y;
    d2 = min(d2, clp);
    // Lanczos-2 approximation without sin()/rcp()/sqrt():
    //   (25/16 * (2/5 * x² - 1)² - 9/16) * (lob*x² - 1)²
    var wB = (2.0 / 5.0) * d2 - 1.0;
    var wA = lob * d2 - 1.0;
    wB = wB * wB;
    wA = wA * wA;
    wB = (25.0 / 16.0) * wB - (25.0 / 16.0 - 1.0);
    let w = wB * wA;
    *aC = *aC + c * w;
    *aW = *aW + w;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let src = vec2<f32>(U.con.x, U.con.y);
    // output pixel center → input pixel coords (FsrEasuCon inlined)
    let ppFull = in.uv * src - vec2<f32>(0.5);
    let fp = floor(ppFull);
    let pp = ppFull - fp;
    let ip = vec2<i32>(fp);

    // 12-tap kernel (ffx_fsr1.h diagram):
    //      b  c
    //   e  f  g  h
    //   i  j  k  l
    //      n  o
    let bC = loadC(ip + vec2<i32>( 0, -1));
    let cC = loadC(ip + vec2<i32>( 1, -1));
    let eC = loadC(ip + vec2<i32>(-1,  0));
    let fC = loadC(ip + vec2<i32>( 0,  0));
    let gC = loadC(ip + vec2<i32>( 1,  0));
    let hC = loadC(ip + vec2<i32>( 2,  0));
    let iC = loadC(ip + vec2<i32>(-1,  1));
    let jC = loadC(ip + vec2<i32>( 0,  1));
    let kC = loadC(ip + vec2<i32>( 1,  1));
    let lC = loadC(ip + vec2<i32>( 2,  1));
    let nC = loadC(ip + vec2<i32>( 0,  2));
    let oC = loadC(ip + vec2<i32>( 1,  2));

    // directional analysis — four quads, bilinear-weighted
    var dir = vec2<f32>(0.0);
    var len = 0.0;
    easuSet(&dir, &len, (1.0 - pp.x) * (1.0 - pp.y), luma2(bC), luma2(eC), luma2(fC), luma2(gC), luma2(jC));
    easuSet(&dir, &len,        pp.x  * (1.0 - pp.y), luma2(cC), luma2(fC), luma2(gC), luma2(hC), luma2(kC));
    easuSet(&dir, &len, (1.0 - pp.x) *        pp.y , luma2(fC), luma2(iC), luma2(jC), luma2(kC), luma2(nC));
    easuSet(&dir, &len,        pp.x  *        pp.y , luma2(gC), luma2(jC), luma2(kC), luma2(lC), luma2(oC));

    // normalize with cleanup close to zero (WGSL has no ternary — select())
    let dir2 = dir * dir;
    var dirR = dir2.x + dir2.y;
    let zro = dirR < (1.0 / 32768.0);
    dirR = select(inverseSqrt(dirR), 1.0, zro);
    dir.x = select(dir.x, 1.0, zro);
    dir = dir * dirR;
    // transform from {0 to 2} to {0 to 1} range, and shape with square
    len = len * 0.5;
    len = len * len;
    // stretch kernel {1.0 vert|horz, to sqrt(2.0) on diagonal}
    let stretch = (dir.x * dir.x + dir.y * dir.y) / max(abs(dir.x), abs(dir.y));
    // anisotropic length after rotation
    let len2 = vec2<f32>(1.0 + (stretch - 1.0) * len, 1.0 + (-0.5) * len);
    // negative lobe strength from the edge amount
    let lob = 0.5 + (0.25 - 0.04 - 0.5) * len;
    // distance² clipping point at the end of the adjustable window
    let clp = 1.0 / max(lob, 1e-4);

    // 12-tap accumulation
    var aC = vec3<f32>(0.0);
    var aW = 0.0;
    easuTap(&aC, &aW, vec2<f32>( 0.0, -1.0) - pp, dir, len2, lob, clp, bC); // b
    easuTap(&aC, &aW, vec2<f32>( 1.0, -1.0) - pp, dir, len2, lob, clp, cC); // c
    easuTap(&aC, &aW, vec2<f32>( 0.0,  0.0) - pp, dir, len2, lob, clp, fC); // f
    easuTap(&aC, &aW, vec2<f32>(-1.0,  0.0) - pp, dir, len2, lob, clp, eC); // e
    easuTap(&aC, &aW, vec2<f32>( 1.0,  0.0) - pp, dir, len2, lob, clp, gC); // g
    easuTap(&aC, &aW, vec2<f32>( 0.0,  1.0) - pp, dir, len2, lob, clp, jC); // j
    easuTap(&aC, &aW, vec2<f32>(-1.0,  1.0) - pp, dir, len2, lob, clp, iC); // i
    easuTap(&aC, &aW, vec2<f32>( 1.0,  1.0) - pp, dir, len2, lob, clp, kC); // k
    easuTap(&aC, &aW, vec2<f32>( 2.0,  1.0) - pp, dir, len2, lob, clp, lC); // l
    easuTap(&aC, &aW, vec2<f32>( 2.0,  0.0) - pp, dir, len2, lob, clp, hC); // h
    easuTap(&aC, &aW, vec2<f32>( 1.0,  2.0) - pp, dir, len2, lob, clp, oC); // o
    easuTap(&aC, &aW, vec2<f32>( 0.0,  2.0) - pp, dir, len2, lob, clp, nC); // n

    // normalize + dering against the 4 nearest (f, g, j, k)
    let min4 = min(min(fC, kC), min(jC, gC));
    let max4 = max(max(fC, kC), max(jC, gC));
    var pix = aC / max(aW, 1e-6);
    pix = min(max4, max(min4, pix));
    return vec4<f32>(pix, 1.0);
}
"#;

// ---------------------------------------------------------------- renderer

/// Sun shadow-map pass: renders terrain geometry from an orthographic camera
/// at the sun, packing linear clip-space depth (0..1, glam orthographic_rh
/// convention) into two RGBA8 channels (16-bit split: r = hi/255, g = lo).
/// A plain RGBA8 color target + textureLoad PCF works on EVERY backend
/// (WebGL2 has no guaranteed comparison samplers / float render targets),
/// which depth-texture sampling or R32Float targets would not.
const SHADOW_SHADER: &str = r#"
struct ShadowG {
    shadow_vp: mat4x4<f32>,
    params: vec4<f32>,
    size: vec4<f32>,
};
@group(0) @binding(5) var<uniform> SH: ShadowG;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) v_data: vec4<u32>,
    @location(1) origin: vec2<f32>,
) -> VsOut {
    // VC-16 packed position decode (see mesh.rs for the bit layout)
    let x = f32(v_data.x & 0xFFFFu) / 2048.0 - 8.0;
    let z = f32(v_data.x >> 16u) / 2048.0 - 8.0;
    let y = f32(v_data.y & 0xFFFFu) / 128.0;
    var out: VsOut;
    out.pos = SH.shadow_vp * vec4<f32>(origin.x + x, y, origin.y + z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // orthographic w = 1 → clip.z is already the 0..1 depth
    let d = clamp(in.pos.z, 0.0, 1.0);
    let v = d * 255.0;
    let hi = floor(v);
    let lo = v - hi;
    return vec4<f32>(hi / 255.0, lo, 0.0, 1.0);
}
"#;

// ChunkGpu / MeshSlot / draw-list types live in draw.rs (pure CPU,
// unit-testable + benchable headless — Phase 9 §14/§37).

/// GPU buffers of one 8×8-chunk mesh region (spec §14 item 3: regional
/// mega-buffers). Vertex + index storage is sub-allocated to chunks via
/// `SlotAlloc` ranges; remeshes that fit write in place (no realloc —
/// §14 in-place reuse), growth is a doubling realloc + GPU→GPU copy
/// submitted before the new data (§43).
struct RegionArena {
    v: wgpu::Buffer,
    i: wgpu::Buffer,
    /// buffer capacities in elements (16 B vertices / 4 B indices)
    v_elems: u32,
    i_elems: u32,
    va: SlotAlloc,
    ia: SlotAlloc,
}

const VERT_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;

/// Bake a chunk's mesh-relative indices into ABSOLUTE arena indices
/// (i + v_off). Uploaded once per (re)mesh; lets every draw use
/// base_vertex = 0 — mandatory on WebGL2/GL (glow has no
/// draw_elements_instanced_base_vertex), harmless on Vulkan/DX12/Metal.
#[inline]
fn bake_absolute_indices(idx: &[u32], v_off: u32) -> Vec<u32> {
    if v_off == 0 {
        return idx.to_vec();
    }
    idx.iter().map(|&i| i + v_off).collect()
}

#[inline]
fn arena_v_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}
#[inline]
fn arena_i_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}
/// initial region arena capacities: 65 536 B vertices / 16 384 B indices
const REGION_START_V_ELEMS: u32 = 4096;
const REGION_START_I_ELEMS: u32 = 4096;

/// capacity of the per-frame chunk-origin instance buffer (one Float32x2
/// entry per visible chunk drawn; render distance 64 would need ~16k —
/// 2048 covers every setting the options screen allows at 60fps draw budgets)
const MAX_DRAW_CHUNKS: usize = 2048;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    atlas_tex: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    /// CPU copy of the atlas (mip-chain rebuilds + E2E pixel reads)
    atlas_data: Vec<u8>,
    /// Phase 6 §26: mipmap levels in use (0 = off; vanilla default 4)
    mip_levels: u8,
    /// Phase 6 §26: anisotropy clamp in use (1 = off; OptiFine default 4)
    aniso: u8,
    globals_buf: wgpu::Buffer,
    world_bgl: wgpu::BindGroupLayout,
    world_bg: wgpu::BindGroup,
    /// per-frame instance-rate chunk origins (x,z in world blocks) — slot 1
    /// of the VC-16 terrain/water/shadow pipelines
    origin_vb: wgpu::Buffer,
    terrain_pipe: wgpu::RenderPipeline,
    water_pipe: wgpu::RenderPipeline,
    sky_pipe: wgpu::RenderPipeline,
    // clouds
    cloud_bg: wgpu::BindGroup,
    cloud_pipe: wgpu::RenderPipeline,
    cloud_vb: wgpu::Buffer,
    // ui
    ui_tex: wgpu::Texture,
    ui_view: wgpu::TextureView,
    ui_samp: wgpu::Sampler,
    ui_buf: wgpu::Buffer,
    ui_bg: wgpu::BindGroup,
    ui_pipe: wgpu::RenderPipeline,
    ui_vb: wgpu::Buffer,
    ui_ib: wgpu::Buffer,
    // selection lines
    line_buf: wgpu::Buffer,
    line_vb: wgpu::Buffer,
    line_pipe: wgpu::RenderPipeline,
    line_bg: wgpu::BindGroup,
    /// Phase 6: shared scene bind group layouts — both the 1x and MSAA
    /// pipeline sets reference the SAME objects, so bind groups stay
    /// compatible across a quality toggle without recreation
    line_bgl: wgpu::BindGroupLayout,
    cloud_bgl: wgpu::BindGroupLayout,
    part_bgl: wgpu::BindGroupLayout,
    // post-processing
    post_targets: PostTargets,
    post_samp: wgpu::Sampler,
    post_buf: wgpu::Buffer,
    /// composite pipeline variant targeting the LINEAR pack handoff target
    /// (used instead of `post_pipe` when a shader-pack stage is active —
    /// Phase 11 §34: the pack sees linear color, its pass does the sRGB
    /// encode; caught as a wgpu validation error in browser E2E)
    post_pipe_linear: wgpu::RenderPipeline,
    // blur directions: SEPARATE buffers per axis. A single buffer written
    // mid-encoder (between the h-blur and v-blur passes) does NOT work —
    // queue.write_buffer applies at the next submit, so both passes read the
    // last-written value (horizontal) and the vertical blur never runs →
    // bloom smears into vertical streaks (looked like sky "tearing").
    aux_h_buf: wgpu::Buffer,
    aux_v_buf: wgpu::Buffer,
    /// FSR 1.0 EASU size constants (src/dst) — written on resize/upscale
    easu_buf: wgpu::Buffer,
    post_bgl: wgpu::BindGroupLayout,
    comp_bgl: wgpu::BindGroupLayout,
    bg_scene: wgpu::BindGroup,
    bg_q: wgpu::BindGroup,
    bg_b1: wgpu::BindGroup,
    bg_comp: wgpu::BindGroup,
    /// EASU pass bind group (scene view + size constants)
    bg_easu: wgpu::BindGroup,
    bright_pipe: wgpu::RenderPipeline,
    blur_pipe: wgpu::RenderPipeline,
    /// FSR 1.0 EASU upscale pass (scene → up)
    easu_pipe: wgpu::RenderPipeline,
    post_pipe: wgpu::RenderPipeline,
    // sun shadow map
    shadow_tex: wgpu::TextureView,
    shadow_depth: wgpu::TextureView,
    shadow_samp: wgpu::Sampler,
    shadow_buf: wgpu::Buffer,
    shadow_bg: wgpu::BindGroup,
    shadow_pipe: wgpu::RenderPipeline,
    /// current shadow map resolution (px per side) — §17 quality setting
    pub shadow_px: u32,
    // §18 biome tint LUT (row = kind, col = slot)
    tint_tex: wgpu::Texture,
    tint_view: wgpu::TextureView,
    // particles (§16.2 pass 4)
    part_bg: wgpu::BindGroup,
    part_pipe: wgpu::RenderPipeline,
    particle_vb: wgpu::Buffer,
    // internal render scale (FSR-lite): scene/bloom/depth sized w*scale
    pub upscale: f32,
    /// adapter/backend description for the F3 overlay (e.g. "WebGPU (SwiftShader)")
    pub backend_name: String,
    pub chunks: HashMap<ChunkPos, ChunkGpu>,
    /// 8×8-chunk mesh-region arenas (Phase 9 §14: regional mega-buffers)
    regions: HashMap<(i32, i32), RegionArena>,
    /// true when the device exposes MULTI_DRAW_INDIRECT +
    /// INDIRECT_FIRST_INSTANCE (native Vulkan/DX12/Metal) — one
    /// multi_draw_indexed_indirect per region run; false → zero-rebind
    /// loop path (WebGPU/WebGL2/GL, §14 capability detection)
    draw_mdi: bool,
    /// per-frame indirect draw records: [terrain | shadow | water] segments
    args_buf: wgpu::Buffer,
    // ------------------------------------------------- Phase 6 §26 quality --
    /// MSAA sample count in use (0 = off — vanilla-faithful default; the
    /// setting itself is an OptiFine-parity extension, dossier Part 1 §3:
    /// ofAaLevel. Only 4x/8x — WebGPU guarantees 4x on renderable formats,
    /// 2x has no guaranteed path)
    msaa: u8,
    /// device-supported maximum (capability detection, not assumption):
    /// 0/4/8 from the scene format's MULTISAMPLE_X4/X8 flags
    msaa_max: u8,
    /// MSAA scene color target (resolve → post_targets.scene_view)
    msaa_view: Option<wgpu::TextureView>,
    /// MSAA depth target (replaced by `depth` when off)
    msaa_depth: Option<wgpu::TextureView>,
    /// lazily-built 4x/8x scene pipeline set (built on first enable —
    /// one-time hitch like vanilla's settings-apply, zero boot cost off)
    msaa_pipes: Option<ScenePipes>,
    /// chunk-graph occlusion culling (§26; OptiFine ofOcclusionFancy parity
    /// — default ON)
    pub occlusion: bool,
    /// §26 rendering-cost fix: mesh-set revision — bumped by EVERY mesh
    /// upload/removal/clear; the occlusion flood cache keys on it so the
    /// BFS runs only when the world or camera section actually changes
    /// (see draw::OcclCache)
    mesh_rev: u64,
    /// §26 rendering-cost fix: cached occlusion flood (per-frame reuse)
    occl_cache: draw::OcclCache,
    // ------------------------------------------------ Phase 11 (§34) packs --
    /// active shader-pack stage (pipeline + bind group over the pack
    /// handoff target) — None = engine composite writes the surface directly
    pack_pipe: Option<(wgpu::RenderPipeline, wgpu::BindGroup)>,
    /// PackUniform bridge buffer (params/viewport/time)
    pack_buf: wgpu::Buffer,
    /// active pack's engine-side grade row (PostUniform.q override)
    pack_grade: Option<[f32; 4]>,
    /// active pack id (F3/E2E) + tier label
    pub pack_id: Option<String>,
    pub pack_tier: String,
    /// clone of the active pack (settings defaults feed PackUniform)
    active_pack_src: Option<crate::shaders::ShaderPack>,
    /// composite pipeline layout (pack composites reuse it, §34)
    comp_pl: wgpu::PipelineLayout,
    present_modes: Vec<wgpu::PresentMode>,
    pub vsync: bool,
    /// diagnostic counter: frames successfully submitted (logged first 3)
    submitted_frames: u32,
    /// Phase 7: GPU compute mesher — None when the adapter lacks compute
    /// (WebGL2 fallback) or pipeline creation failed. All mesher state is
    /// main-thread-only (the zero-Mutex design holds: rayon jobs build
    /// inputs, a channel hands them back here)
    pub gpu_mesh: Option<crate::gpu_mesh::GpuMesher>,
}

/// Phase 6: the six scene-pass pipelines (everything that renders into the
/// offscreen scene target + depth). Built per sample count — 1x at boot,
/// the MSAA set lazily on first enable. All variants share the Renderer's
/// bind group layouts, so the same bind groups bind to either set.
struct ScenePipes {
    terrain: wgpu::RenderPipeline,
    water: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    line: wgpu::RenderPipeline,
    part: wgpu::RenderPipeline,
    cloud: wgpu::RenderPipeline,
}

/// build one full scene-pipeline set at `samples` (1 or 4/8). Shader
/// sources are the module consts; layouts are the Renderer's shared bgl's.
fn build_scene_pipes(
    device: &wgpu::Device,
    world_bgl: &wgpu::BindGroupLayout,
    line_bgl: &wgpu::BindGroupLayout,
    cloud_bgl: &wgpu::BindGroupLayout,
    part_bgl: &wgpu::BindGroupLayout,
    samples: u32,
) -> ScenePipes {
    let scene_format = wgpu::TextureFormat::Rgba8Unorm;

    // VC-16 packed vertex (slot 0) + per-chunk origin (slot 1, instance rate)
    let terrain_vbl = [
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32x4,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            }],
        },
    ];
    let depth_state = |write: bool, cmp: wgpu::CompareFunction| wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: write,
        depth_compare: cmp,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };
    // Blending for translucent surfaces that keeps the canvas OPAQUE:
    // alpha uses One/One so dst.a stays 1 (see the 1x-site comment).
    let opaque_blend = Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    });

    let terrain_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("terrain"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(TERRAIN_SHADER)),
    });
    let water_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("water"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WATER_SHADER)),
    });
    let sky_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("sky"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SKY_SHADER)),
    });
    let line_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lines"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(LINE_SHADER)),
    });
    let part_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("particles"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PARTICLE_SHADER)),
    });
    let cloud_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("clouds"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(CLOUD_SHADER)),
    });

    let world_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("world-pl"),
        bind_group_layouts: &[world_bgl],
        push_constant_ranges: &[],
    });
    let line_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("line-pl"),
        bind_group_layouts: &[line_bgl],
        push_constant_ranges: &[],
    });
    let cloud_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("cloud-pl"),
        bind_group_layouts: &[cloud_bgl],
        push_constant_ranges: &[],
    });
    let part_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("part-pl"),
        bind_group_layouts: &[part_bgl],
        push_constant_ranges: &[],
    });

    let ms = wgpu::MultisampleState {
        count: samples,
        ..Default::default()
    };
    let make_world_pipe = |module: &wgpu::ShaderModule,
                           entry: &str,
                           cull: Option<wgpu::Face>,
                           blend: Option<wgpu::BlendState>,
                           depth: wgpu::DepthStencilState,
                           buffers: &[wgpu::VertexBufferLayout]| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipe"),
            layout: Some(&world_pl),
            vertex: wgpu::VertexState {
                module,
                entry_point: entry,
                compilation_options: Default::default(),
                buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: scene_format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: cull,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(depth),
            multisample: ms,
            multiview: None,
            cache: None,
        })
    };

    // sky: no vertex buffers, no cull, depth-tested Always (paints first)
    static NO_BUFS: [wgpu::VertexBufferLayout; 0] = [];
    let sky = make_world_pipe(
        &sky_mod,
        "vs_main",
        None,
        None,
        depth_state(false, wgpu::CompareFunction::Always),
        &NO_BUFS,
    );
    let terrain = make_world_pipe(
        &terrain_mod,
        "vs_main",
        Some(wgpu::Face::Back),
        None,
        depth_state(true, wgpu::CompareFunction::Less),
        &terrain_vbl,
    );
    let water = make_world_pipe(
        &water_mod,
        "vs_main",
        None,
        opaque_blend,
        depth_state(false, wgpu::CompareFunction::Less),
        &terrain_vbl,
    );

    // selection lines: LineList, alpha blend, depth-test no write
    let line_vbl = [wgpu::VertexBufferLayout {
        array_stride: 12,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }];
    let line = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("line-pipe"),
        layout: Some(&line_pl),
        vertex: wgpu::VertexState {
            module: &line_mod,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &line_vbl,
        },
        fragment: Some(wgpu::FragmentState {
            module: &line_mod,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: scene_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Less)),
        multisample: ms,
        multiview: None,
        cache: None,
    });

    // particles: billboards, alpha blend, depth-test no write
    let part_vbl = [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<vc_particles::particles::ParticleVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 20,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    }];
    let part = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("part-pipe"),
        layout: Some(&part_pl),
        vertex: wgpu::VertexState {
            module: &part_mod,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &part_vbl,
        },
        fragment: Some(wgpu::FragmentState {
            module: &part_mod,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: scene_format,
                blend: opaque_blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // billboards: winding varies with view angle
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Less)),
        multisample: ms,
        multiview: None,
        cache: None,
    });

    // clouds: 2-vertex xz plane
    let cloud_vbl = [wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        }],
    }];
    let cloud = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("cloud-pipe"),
        layout: Some(&cloud_pl),
        vertex: wgpu::VertexState {
            module: &cloud_mod,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &cloud_vbl,
        },
        fragment: Some(wgpu::FragmentState {
            module: &cloud_mod,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: scene_format,
                blend: opaque_blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        // clouds sit above the world: drawn with depth-test no write
        depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Less)),
        multisample: ms,
        multiview: None,
        cache: None,
    });

    ScenePipes {
        terrain,
        water,
        sky,
        line,
        part,
        cloud,
    }
}

impl Renderer {
    pub async fn new(window: &'static winit::window::Window, atlas: &[u8]) -> Self {
        // Backend selection: probe for a *working* WebGPU adapter first,
        // mirroring EXACTLY the requestAdapter() options wgpu will use.
        // (navigator.gpu existing is not enough — headless Chromium exposes
        // the API but returns null adapters, and wgpu locks an instance to
        // WebGPU-only mode whenever it detects the API at all.
        // Those fall back to the WebGL2/GL backend.)
        // Chain: hardware adapter → fallback (software) adapter → WebGL2.
        #[cfg(target_arch = "wasm32")]
        let (backends, force_fallback_adapter) = match choose_webgpu_mode().await {
            // false = real GPU adapter, true = software adapter (SwiftShader)
            Some(force_fallback) => (wgpu::Backends::BROWSER_WEBGPU, force_fallback),
            None => (wgpu::Backends::GL, false),
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (backends, force_fallback_adapter) = (wgpu::Backends::all(), false);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::from(window))
            .expect("create surface");
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter,
            })
            .await
        {
            Some(a) => a,
            None => {
                report_boot_error("no suitable GPU adapter (WebGPU and WebGL2 both unavailable)");
                panic!("no suitable GPU adapter");
            }
        };

        // Phase 9 §14: request multi-draw-indirect where the adapter has it
        // (native-only feature; intersected so unsupported adapters still
        // create the device — capability detection, not assumption).
        let mdi_wanted = if cfg!(not(target_arch = "wasm32")) {
            adapter.features()
                & (wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::INDIRECT_FIRST_INSTANCE)
        } else {
            wgpu::Features::empty()
        };

        // WebGL2 (downlevel) can't satisfy default limits (no compute);
        // retry with downlevel limits in that case.
        let (device, queue) = match adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("voxelcraft"),
                    required_features: mdi_wanted,
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
        {
            Ok(dq) => dq,
            Err(first_err) => {
                report_boot_log(&format!(
                    "default device limits rejected ({first_err:?}) — retrying with downlevel limits"
                ));
                match adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: Some("voxelcraft-downlevel"),
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                            ..Default::default()
                        },
                        None,
                    )
                    .await
                {
                    Ok(dq) => dq,
                    Err(second_err) => {
                        report_boot_error(&format!(
                            "GPU device request failed: {first_err:?} / {second_err:?}"
                        ));
                        panic!("request device (downlevel)");
                    }
                }
            }
        };

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        // The game renders a fully opaque 3D scene — force Opaque compositing
        // where supported. With `Auto`/premultiplied, browsers composite the
        // canvas with the page background, which can yield a see-through
        // canvas if any pass leaves alpha < 1 (WebGPU honors it strictly,
        // unlike the WebGL2 path).
        let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            wgpu::CompositeAlphaMode::Auto
        };
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        report_boot_log("surface configured, building pipelines…");

        let present_modes = caps.present_modes.clone();
        let vsync = present_modes.contains(&wgpu::PresentMode::Fifo);

        // backend description for the F3 debug overlay
        let adapter_info = adapter.get_info();
        let backend_name = format!("{:?} ({})", adapter_info.backend, {
            let n = adapter_info.name.clone();
            if n.is_empty() {
                "generic".to_string()
            } else {
                n
            }
        });

        // FSR-lite: start at native scale (1.0); the game raises it via
        // set_upscale() from the saved settings once running.
        let upscale = 1.0f32;
        let depth = Self::make_depth(
            &device,
            ((config.width as f32) * upscale).round().max(1.0) as u32,
            ((config.height as f32) * upscale).round().max(1.0) as u32,
        );

        // atlas
        let atlas_size = wgpu::Extent3d {
            width: textures::ATLAS_SIZE as u32,
            height: textures::ATLAS_SIZE as u32,
            depth_or_array_layers: 1,
        };
        let atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(textures::ATLAS_SIZE as u32 * 4),
                rows_per_image: Some(textures::ATLAS_SIZE as u32),
            },
            atlas_size,
        );
        let atlas_view = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut samp = wgpu::SamplerDescriptor::default();
        samp.mag_filter = wgpu::FilterMode::Nearest;
        samp.min_filter = wgpu::FilterMode::Nearest;
        let sampler = device.create_sampler(&samp);

        // world bind group
        let world_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("world-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // sun shadow map (sampled with textureLoad PCF — NEAREST,
                // packed depth: bilinear filtering would corrupt the hi/lo split)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // light view-projection + params (VERTEX: the shadow depth
                // pass vertex shader projects with it)
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // §18 biome tint LUT — textureLoad in the VERTEX stage
                // (per-vertex indices are only guaranteed for texture
                // loads, not uniform array indexing, on Vulkan/WebGL2)
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // shadow map resources: RGBA8 packed-depth target + its own depth
        // buffer (color targets cannot depth-test, and the depth texture is
        // what keeps the FRONT-MOST surface in the map when geometry
        // overlaps in the light view). §17 quality: 1024/2048/4096 selectable
        // — CLAMPED to the device limit (downlevel/SwiftShader adapters cap
        // at 2048; requesting more is a validation error, not a fallback).
        let shadow_px: u32 = 2048.min(device.limits().max_texture_dimension_2d);
        let shadow_extent = wgpu::Extent3d {
            width: shadow_px,
            height: shadow_px,
            depth_or_array_layers: 1,
        };
        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: shadow_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_depth_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-depth"),
            size: shadow_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let shadow_depth = shadow_depth_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut shadow_samp_d = wgpu::SamplerDescriptor::default();
        shadow_samp_d.mag_filter = wgpu::FilterMode::Nearest;
        shadow_samp_d.min_filter = wgpu::FilterMode::Nearest;
        shadow_samp_d.address_mode_u = wgpu::AddressMode::ClampToEdge;
        shadow_samp_d.address_mode_v = wgpu::AddressMode::ClampToEdge;
        let shadow_samp = device.create_sampler(&shadow_samp_d);
        let shadow_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow-globals"),
            size: std::mem::size_of::<ShadowGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_globals = ShadowGlobals {
            shadow_vp: Mat4::IDENTITY.to_cols_array_2d(),
            params: [0.0, 0.0, 0.0, 0.0],
            size: [shadow_px as f32, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&shadow_buf, 0, bytemuck::bytes_of(&shadow_globals));

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // per-frame instance-rate chunk origins for the VC-16 vertex format
        let origin_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chunk-origins"),
            size: (MAX_DRAW_CHUNKS * 8) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // §18 biome tint LUT: 64×4 RGBA8 (row = kind, col = slot). Static
        // engine constants — written once, never touched again.
        let tint_data = vc_blocks::tint::lut_rgba();
        let tint_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tint-lut"),
            size: wgpu::Extent3d {
                width: vc_blocks::tint::LUT_W,
                height: vc_blocks::tint::LUT_H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tint_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &tint_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(vc_blocks::tint::LUT_W * 4),
                rows_per_image: Some(vc_blocks::tint::LUT_H),
            },
            wgpu::Extent3d {
                width: vc_blocks::tint::LUT_W,
                height: vc_blocks::tint::LUT_H,
                depth_or_array_layers: 1,
            },
        );
        let tint_view = tint_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let world_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world-bg"),
            layout: &world_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&shadow_samp),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &shadow_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&tint_view),
                },
            ],
        });

        // VC-16 packed vertex (slot 0) + per-chunk origin (slot 1, instance
        // rate). The origin reaches the vertex shader as a plain instance
        // attribute — portable to native, WebGPU AND WebGL2 (no push
        // constants, no dynamic uniform offsets, no SSBOs; see
        // docs/research/wgpu-web-assets.md).
        let terrain_vbl = [
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32x4,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            },
        ];
        // (terrain/water/sky/line/particles/clouds pipelines are built by
        // `build_scene_pipes` — Phase 6 made the set sample-count-parametric
        // for MSAA; see its comment for blend/depth rationale)

        // shadow depth pipeline: same vertex layout as terrain, writes packed
        // depth into RGBA8. Slope-scaled depth bias (mapped to polygon
        // offset by the GL backend) supplements the shader-side normal
        // offset to kill self-shadow acne.
        //
        // NOTE: the shadow pass gets its OWN bind group containing ONLY the
        // light matrix uniform — binding the world group here would make the
        // shadow texture a RESOURCE inside the very pass that uses it as a
        // COLOR_TARGET (exclusive) → wgpu validation error.
        let shadow_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADOW_SHADER)),
        });
        let shadow_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let shadow_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow-pl"),
            bind_group_layouts: &[&shadow_bgl],
            push_constant_ranges: &[],
        });
        let shadow_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow-bg"),
            layout: &shadow_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let shadow_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow-pipe"),
            layout: Some(&shadow_pl),
            vertex: wgpu::VertexState {
                module: &shadow_mod,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &terrain_vbl,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shadow_mod,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: -1.5,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // --------------------------------------------- particles (§16.2)
        // billboard quads, alpha blend, depth-TEST but no depth write —
        // drawn after the translucent water pass, before clouds
        // (pipeline itself: build_scene_pipes)
        let part_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("part-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let part_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("part-bg"),
            layout: &part_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        // dynamic billboard buffer: MAX_PARTICLES quads × 6 verts × 32 B
        let particle_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle-vb"),
            size: (vc_particles::particles::MAX_PARTICLES * 6 * 32) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ---------------------------------------------------------- UI
        let ui_size = wgpu::Extent3d {
            width: UI_W as u32,
            height: UI_H as u32,
            depth_or_array_layers: 1,
        };
        let ui_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ui"),
            size: ui_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let ui_view = ui_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut usamp = wgpu::SamplerDescriptor::default();
        usamp.mag_filter = wgpu::FilterMode::Nearest;
        usamp.min_filter = wgpu::FilterMode::Nearest;
        let ui_samp = device.create_sampler(&usamp);

        let ui_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let ui_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui-uniform"),
            size: std::mem::size_of::<UiUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ui_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui-bg"),
            layout: &ui_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ui_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ui_samp),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ui_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let ui_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(UI_SHADER)),
        });

        let ui_vbl = wgpu::VertexBufferLayout {
            array_stride: 16,
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
            ],
        };

        let ui_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui-pl"),
            bind_group_layouts: &[&ui_bgl],
            push_constant_ranges: &[],
        });

        let ui_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui-pipe"),
            layout: Some(&ui_pl),
            vertex: wgpu::VertexState {
                module: &ui_mod,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[ui_vbl],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_mod,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // UI pass renders without a depth attachment (drawn last, on top)
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // UI quad (canvas px, uv)
        let ui_verts: [[f32; 4]; 4] = [
            [0.0, 0.0, 0.0, 0.0],
            [UI_W as f32, 0.0, 1.0, 0.0],
            [UI_W as f32, UI_H as f32, 1.0, 1.0],
            [0.0, UI_H as f32, 0.0, 1.0],
        ];
        let ui_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui-vb"),
            contents: bytemuck::cast_slice(&ui_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ui_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui-ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });

        // ------------------------------------------------- selection lines
        // (pipeline: build_scene_pipes; the bgl stays shared)
        let line_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("line-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let line_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line-uniform"),
            size: std::mem::size_of::<LineUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let line_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("line-bg"),
            layout: &line_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &line_buf,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // unit cube edges (slightly inflated to avoid z-fighting)
        let e = 0.002;
        let lo = -e;
        let hi = 1.0 + e;
        let c = [
            [lo, lo, lo],
            [hi, lo, lo],
            [hi, lo, hi],
            [lo, lo, hi],
            [lo, hi, lo],
            [hi, hi, lo],
            [hi, hi, hi],
            [lo, hi, hi],
        ];
        let edges: Vec<[f32; 3]> = vec![
            c[0], c[1], c[1], c[2], c[2], c[3], c[3], c[0], c[4], c[5], c[5], c[6], c[6], c[7],
            c[7], c[4], c[0], c[4], c[1], c[5], c[2], c[6], c[3], c[7],
        ];
        let line_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("line-vb"),
            contents: bytemuck::cast_slice(&edges),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // ---------------------------------------------------------- clouds
        let cloud_atlas = crate::textures::generate_cloud_atlas();
        let cloud_size = wgpu::Extent3d {
            width: crate::textures::CLOUD_TEX as u32,
            height: crate::textures::CLOUD_TEX as u32,
            depth_or_array_layers: 1,
        };
        let cloud_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("clouds"),
            size: cloud_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &cloud_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &cloud_atlas,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(crate::textures::CLOUD_TEX as u32 * 4),
                rows_per_image: Some(crate::textures::CLOUD_TEX as u32),
            },
            cloud_size,
        );
        let cloud_view = cloud_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut csamp = wgpu::SamplerDescriptor::default();
        csamp.mag_filter = wgpu::FilterMode::Nearest; // crisp vanilla-style cloud edges
        csamp.min_filter = wgpu::FilterMode::Nearest;
        csamp.address_mode_u = wgpu::AddressMode::Repeat;
        csamp.address_mode_v = wgpu::AddressMode::Repeat;
        let cloud_samp = device.create_sampler(&csamp);

        let cloud_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cloud-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let cloud_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cloud-bg"),
            layout: &cloud_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cloud_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&cloud_samp),
                },
            ],
        });
        // cloud quad: big XZ plane centered on the camera (recentered in vs)
        let s = 2400.0f32;
        let cloud_verts: [[f32; 2]; 6] = [[-s, -s], [s, -s], [s, s], [-s, -s], [s, s], [-s, s]];
        let cloud_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cloud-vb"),
            contents: bytemuck::cast_slice(&cloud_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // ------------------------------------------------- post-processing
        let post_samp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post-samp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let post_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post-uniform"),
            size: std::mem::size_of::<PostUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let aux_h_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post-aux-h"),
            size: std::mem::size_of::<AuxUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let aux_v_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post-aux-v"),
            size: std::mem::size_of::<AuxUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // FSR 1.0 EASU size constants (rewritten on resize/upscale change)
        let easu_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fsr-easu"),
            size: std::mem::size_of::<EasuUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // write once here; rewritten on resize (texel steps are size-dependent)
        let bw = (config.width / 8).max(1);
        let bh = (config.height / 8).max(1);
        queue.write_buffer(
            &aux_h_buf,
            0,
            bytemuck::bytes_of(&AuxUniform {
                dir: [1.0 / bw as f32, 0.0, 0.0, 0.0],
            }),
        );
        queue.write_buffer(
            &aux_v_buf,
            0,
            bytemuck::bytes_of(&AuxUniform {
                dir: [0.0, 1.0 / bh as f32, 0.0, 0.0],
            }),
        );

        let uniform_entry = |binding: u32, vis: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vis,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        // single-texture layout (bright + blur passes)
        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post-bgl"),
            entries: &[
                texture_entry(0),
                sampler_entry(1),
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT),
                uniform_entry(3, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        // two-texture layout (composite pass)
        let comp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("comp-bgl"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                sampler_entry(2),
                uniform_entry(3, wgpu::ShaderStages::FRAGMENT),
            ],
        });

        let bright_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bright"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BRIGHT_SHADER)),
        });
        let blur_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blur"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLUR_SHADER)),
        });
        let post_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(POST_SHADER)),
        });
        let easu_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fsr-easu"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(EASU_SHADER)),
        });

        let post_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post-pl"),
            bind_group_layouts: &[&post_bgl],
            push_constant_ranges: &[],
        });
        let comp_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("comp-pl"),
            bind_group_layouts: &[&comp_bgl],
            push_constant_ranges: &[],
        });
        let make_fs_pipe = |module: &wgpu::ShaderModule,
                            layout: &wgpu::PipelineLayout,
                            out_format: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fs-pipe"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: out_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };
        let linear = wgpu::TextureFormat::Rgba8Unorm;
        let bright_pipe = make_fs_pipe(&bright_mod, &post_pl, linear);
        let blur_pipe = make_fs_pipe(&blur_mod, &post_pl, linear);
        // FSR 1.0 EASU: scene (render scale) → up (full surface res)
        let easu_pipe = make_fs_pipe(&easu_mod, &post_pl, linear);
        // composite writes the final srgb-encoded image to the surface
        let post_pipe = make_fs_pipe(&post_mod, &comp_pl, format);
        // Phase 11 §34: linear-target variant for the pack handoff path
        let post_pipe_linear = make_fs_pipe(&post_mod, &comp_pl, wgpu::TextureFormat::Rgba8Unorm);

        // offscreen targets + bind groups at the real size
        // (blur-h reads q with the H-step aux; blur-v reads b1 with the V-step)
        let post_targets = PostTargets::new(&device, config.width, config.height, format, upscale);
        let bg_scene = Self::single_tex_bg(
            &device,
            &post_bgl,
            &post_targets.scene_view,
            &post_samp,
            &post_buf,
            &aux_v_buf,
        );
        let bg_q = Self::single_tex_bg(
            &device,
            &post_bgl,
            &post_targets.q_view,
            &post_samp,
            &post_buf,
            &aux_h_buf,
        );
        let bg_b1 = Self::single_tex_bg(
            &device,
            &post_bgl,
            &post_targets.b1_view,
            &post_samp,
            &post_buf,
            &aux_v_buf,
        );
        // EASU reads the SCENE; binding 3 = the size constants buffer
        let bg_easu = Self::single_tex_bg(
            &device,
            &post_bgl,
            &post_targets.scene_view,
            &post_samp,
            &post_buf,
            &easu_buf,
        );
        // the composite (+ RCAS) reads the EASU-UPScaled target
        let bg_comp = Self::comp_bg(
            &device,
            &comp_bgl,
            &post_targets.up_view,
            &post_targets.b2_view,
            &post_samp,
            &post_buf,
        );
        // initial EASU size constants (rewritten on resize/upscale change)
        let (sc_w, sc_h) = post_targets.scene_size();
        queue.write_buffer(
            &easu_buf,
            0,
            bytemuck::bytes_of(&EasuUniform {
                con: [
                    sc_w as f32,
                    sc_h as f32,
                    config.width as f32,
                    config.height as f32,
                ],
            }),
        );

        // Phase 9 §14: per-frame indirect draw records — [terrain | shadow |
        // water] segments, one record per drawn chunk (20 B each, ≤ 2048 ×3)
        let args_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("draw-indirect-args"),
            size: 3 * MAX_DRAW_CHUNKS as u64 * IndirectArgs::SIZE,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let draw_mdi = device.features().contains(
            wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::INDIRECT_FIRST_INSTANCE,
        );
        if draw_mdi {
            report_boot_log("draw path: MDI (multi_draw_indexed_indirect per region run)");
        } else {
            report_boot_log("draw path: region-arena loop (zero per-chunk binds)");
        }

        // Phase 6 §26: MSAA capability — the scene color format must support
        // multisampling + automatic resolve, the depth format multisampling.
        // (Rgba8Unorm 4x + resolve is guaranteed by WebGPU on every
        // renderable format; 8x is checked, 2x has no guaranteed WebGPU
        // path so the setting only offers off/4/8.)
        let fmt_feats = |f| adapter.get_texture_format_features(f).flags;
        let color_ms = fmt_feats(wgpu::TextureFormat::Rgba8Unorm);
        let depth_ms = fmt_feats(wgpu::TextureFormat::Depth32Float);
        let msaa_max = if color_ms.contains(
            wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4
                | wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE,
        ) && depth_ms.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4)
        {
            if color_ms.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
                8
            } else {
                4
            }
        } else {
            0
        };

        // Phase 6: the 1x scene pipeline set (the MSAA set builds lazily)
        let scene = build_scene_pipes(&device, &world_bgl, &line_bgl, &cloud_bgl, &part_bgl, 1);

        // Phase 11 §34: pack-uniform bridge buffer
        let pack_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pack-uniforms"),
            size: std::mem::size_of::<crate::shaders::PackUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Phase 7: construct the compute mesher BEFORE the struct literal
        // moves device/queue into the Renderer (wgpu 22 handles aren't Clone)
        let gpu_mesh = if adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            Some(crate::gpu_mesh::GpuMesher::new(&device, &queue))
        } else {
            report_boot_log("gpu meshing unavailable: adapter lacks compute (WebGL2-class)");
            None
        };
        let renderer = Renderer {
            surface,
            device,
            queue,
            config,
            depth,
            atlas_tex,
            atlas_view,
            sampler,
            atlas_data: atlas.to_vec(),
            mip_levels: 0,
            aniso: 1,
            globals_buf,
            world_bgl,
            world_bg,
            origin_vb,
            terrain_pipe: scene.terrain,
            water_pipe: scene.water,
            sky_pipe: scene.sky,
            cloud_bg,
            cloud_pipe: scene.cloud,
            cloud_vb,
            ui_tex,
            ui_view,
            ui_samp,
            ui_buf,
            ui_bg,
            ui_pipe,
            ui_vb,
            ui_ib,
            line_buf,
            line_vb,
            line_pipe: scene.line,
            line_bg,
            line_bgl,
            cloud_bgl,
            part_bgl,
            post_targets,
            post_samp,
            post_buf,
            post_pipe_linear,
            aux_h_buf,
            aux_v_buf,
            easu_buf,
            post_bgl,
            comp_bgl,
            bg_scene,
            bg_q,
            bg_b1,
            bg_comp,
            bg_easu,
            bright_pipe,
            blur_pipe,
            easu_pipe,
            post_pipe,
            shadow_tex: shadow_view,
            shadow_depth,
            shadow_samp,
            shadow_buf,
            shadow_bg,
            shadow_pipe,
            shadow_px,
            tint_tex,
            tint_view,
            part_bg,
            part_pipe: scene.part,
            particle_vb,
            upscale,
            backend_name,
            chunks: HashMap::new(),
            regions: HashMap::new(),
            draw_mdi,
            args_buf,
            msaa: 0,
            msaa_max,
            msaa_view: None,
            msaa_depth: None,
            msaa_pipes: None,
            occlusion: true,
            mesh_rev: 0,
            occl_cache: draw::OcclCache::default(),
            pack_pipe: None,
            pack_buf,
            pack_grade: None,
            pack_id: None,
            pack_tier: String::new(),
            active_pack_src: None,
            comp_pl,
            present_modes,
            vsync,
            submitted_frames: 0,
            gpu_mesh,
        };
        report_boot_log("renderer ready (pipelines + atlas + clouds + post chain)");
        renderer
    }

    fn single_tex_bg(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        samp: &wgpu::Sampler,
        post_buf: &wgpu::Buffer,
        aux_buf: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post-bg"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(samp),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: post_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: aux_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }

    fn comp_bg(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        scene: &wgpu::TextureView,
        bloom: &wgpu::TextureView,
        samp: &wgpu::Sampler,
        post_buf: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("comp-bg"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(samp),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: post_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        })
    }

    /// recreate offscreen targets + bind groups after a resize
    fn rebuild_post_targets(&mut self) {
        let w = self.config.width.max(1);
        let h = self.config.height.max(1);
        let format = self.config.format;
        let t = PostTargets::new(&self.device, w, h, format, self.upscale);
        let bg_scene = Self::single_tex_bg(
            &self.device,
            &self.post_bgl,
            &t.scene_view,
            &self.post_samp,
            &self.post_buf,
            &self.aux_v_buf,
        );
        let bg_q = Self::single_tex_bg(
            &self.device,
            &self.post_bgl,
            &t.q_view,
            &self.post_samp,
            &self.post_buf,
            &self.aux_h_buf,
        );
        let bg_b1 = Self::single_tex_bg(
            &self.device,
            &self.post_bgl,
            &t.b1_view,
            &self.post_samp,
            &self.post_buf,
            &self.aux_v_buf,
        );
        let bg_easu = Self::single_tex_bg(
            &self.device,
            &self.post_bgl,
            &t.scene_view,
            &self.post_samp,
            &self.post_buf,
            &self.easu_buf,
        );
        let bg_comp = Self::comp_bg(
            &self.device,
            &self.comp_bgl,
            &t.up_view,
            &t.b2_view,
            &self.post_samp,
            &self.post_buf,
        );
        // Phase 11: the pack bind group follows the (resized) pack target
        let bg_pack = Self::comp_bg(
            &self.device,
            &self.comp_bgl,
            &t.pack_view,
            &t.b2_view,
            &self.post_samp,
            &self.pack_buf,
        );
        if let Some((pipe, _)) = self.pack_pipe.take() {
            self.pack_pipe = Some((pipe, bg_pack));
        }
        // refresh the FSR EASU size constants for the new src/dst sizes
        let (sc_w, sc_h) = t.scene_size();
        self.queue.write_buffer(
            &self.easu_buf,
            0,
            bytemuck::bytes_of(&EasuUniform {
                con: [sc_w as f32, sc_h as f32, w as f32, h as f32],
            }),
        );
        self.post_targets = t;
        self.bg_scene = bg_scene;
        self.bg_q = bg_q;
        self.bg_b1 = bg_b1;
        self.bg_comp = bg_comp;
        self.bg_easu = bg_easu;
        // refresh blur texel steps for the new size (1/8 targets)
        let bw = (w / 8).max(1);
        let bh = (h / 8).max(1);
        self.queue.write_buffer(
            &self.aux_h_buf,
            0,
            bytemuck::bytes_of(&AuxUniform {
                dir: [1.0 / bw as f32, 0.0, 0.0, 0.0],
            }),
        );
        self.queue.write_buffer(
            &self.aux_v_buf,
            0,
            bytemuck::bytes_of(&AuxUniform {
                dir: [0.0, 1.0 / bh as f32, 0.0, 0.0],
            }),
        );
        // Phase 6: the MSAA targets track the scene scale — resize with it
        if self.msaa > 0 {
            self.rebuild_msaa_targets();
        }
    }

    fn make_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: w.max(1),
                height: h.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        tex.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth = Self::make_depth(
            &self.device,
            ((w as f32) * self.upscale).round().max(1.0) as u32,
            ((h as f32) * self.upscale).round().max(1.0) as u32,
        );
        self.rebuild_post_targets();
    }

    /// FSR 1.0 (§33): set the internal render scale (1.0 = native, 0.75/0.5
    /// = EASU-upscaled). Rebuilds the offscreen targets + scene depth
    /// immediately.
    pub fn set_upscale(&mut self, scale: f32) {
        let scale = scale.clamp(0.5, 1.0);
        if (scale - self.upscale).abs() < 1e-3 {
            return;
        }
        self.upscale = scale;
        self.depth = Self::make_depth(
            &self.device,
            ((self.config.width as f32) * scale).round().max(1.0) as u32,
            ((self.config.height as f32) * scale).round().max(1.0) as u32,
        );
        self.rebuild_post_targets();
    }

    // ------------------------------------------------- Phase 6 §26 quality --

    /// Texture quality: `mip_levels` 0..=4 (vanilla `mipmapLevels`, VERIFIED
    /// default 4 / range 0-4, wiki Options.txt) and `aniso` 1/2/4/8/16
    /// (OptiFine `ofAfLevel` parity — vanilla 1.16.5 has no aniso setting;
    /// default 4 comes from the dossier Part 1 §3 captured optionsof.txt).
    /// Rebuilds the atlas texture (mip chain), the world sampler, and the
    /// two atlas-bound bind groups.
    pub fn set_texture_quality(&mut self, mip_levels: u8, aniso: u8) {
        let mips = mip_levels.min(textures::MAX_MIP_LEVELS);
        let aniso = aniso.clamp(1, 16);
        if mips == self.mip_levels && aniso == self.aniso {
            return;
        }
        self.mip_levels = mips;
        self.aniso = aniso;
        self.rebuild_atlas();
    }

    /// current quality settings (F3 / E2E stats)
    pub fn texture_quality(&self) -> (u8, u8) {
        (self.mip_levels, self.aniso)
    }

    /// world bind group over a given atlas view + sampler (binding set:
    /// globals, atlas, sampler, shadow map, shadow sampler, shadow
    /// uniforms, biome-tint LUT — see world_bgl)
    fn make_world_bg(
        &self,
        atlas_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world-bg"),
            layout: &self.world_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.shadow_tex),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_samp),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.shadow_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&self.tint_view),
                },
            ],
        })
    }

    /// particle bind group over a given atlas view + sampler
    fn make_part_bg(
        &self,
        atlas_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("part-bg"),
            layout: &self.part_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// the world sampler for the current quality:
    /// * mips off, aniso 1: Nearest/Nearest (the crisp vanilla default)
    /// * mips on: min Linear + mipmap Linear, mag Nearest — the classic
    ///   vanilla distance blur (documented adaptation: the exact GL
    ///   constants of 1.16.5's GlStateManager can't be verified from the
    ///   public wiki; nearest magnification is certain, trilinear
    ///   minification matches the vanilla look)
    /// * aniso > 1: all filters Linear (WebGPU REQUIRES linear filters for
    ///   anisotropy_clamp > 1 — a spec rule, wgpu validates it) +
    ///   anisotropy_clamp set
    fn world_sampler(&self) -> wgpu::Sampler {
        let mut d = wgpu::SamplerDescriptor::default();
        if self.mip_levels > 0 {
            d.min_filter = wgpu::FilterMode::Linear;
            d.mipmap_filter = wgpu::FilterMode::Linear;
            d.mag_filter = wgpu::FilterMode::Nearest;
        } else {
            d.min_filter = wgpu::FilterMode::Nearest;
            d.mag_filter = wgpu::FilterMode::Nearest;
        }
        if self.aniso > 1 {
            d.anisotropy_clamp = self.aniso as u16;
            d.mag_filter = wgpu::FilterMode::Linear;
            d.min_filter = wgpu::FilterMode::Linear;
            d.mipmap_filter = wgpu::FilterMode::Linear;
        }
        self.device.create_sampler(&d)
    }

    /// recreate the atlas texture with the active mip count, upload the
    /// base image + generated mip levels, refresh the sampler + the two
    /// atlas-bound bind groups
    fn rebuild_atlas(&mut self) {
        let levels = 1 + self.mip_levels as u32;
        let size = wgpu::Extent3d {
            width: textures::ATLAS_SIZE as u32,
            height: textures::ATLAS_SIZE as u32,
            depth_or_array_layers: 1,
        };
        let atlas_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size,
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(textures::ATLAS_SIZE as u32 * 4),
                rows_per_image: Some(textures::ATLAS_SIZE as u32),
            },
            size,
        );
        // mip levels 1..=N: CPU-generated per-tile-safe chain (§26)
        for (i, mip) in textures::generate_mips(&self.atlas_data, self.mip_levels)
            .iter()
            .enumerate()
        {
            let level = (i + 1) as u32;
            let side = (textures::ATLAS_SIZE >> level) as u32;
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &atlas_tex,
                    mip_level: level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                mip,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(side * 4),
                    rows_per_image: Some(side),
                },
                wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: 1,
                },
            );
        }
        let atlas_view = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.world_sampler();
        let world_bg = self.make_world_bg(&atlas_view, &sampler);
        let part_bg = self.make_part_bg(&atlas_view, &sampler);
        self.atlas_tex = atlas_tex;
        self.atlas_view = atlas_view;
        self.sampler = sampler;
        self.world_bg = world_bg;
        self.part_bg = part_bg;
    }

    /// MSAA setting (0 = off — the vanilla-faithful default; 4/8 gated on
    /// device support via `msaa_supported`). Rebuilds the multisampled
    /// targets and lazily builds the scene pipeline variant.
    pub fn set_msaa(&mut self, samples: u8) {
        let want = if samples >= 8 && self.msaa_max >= 8 {
            8
        } else if samples >= 4 && self.msaa_max >= 4 {
            4
        } else {
            0
        };
        if want == self.msaa {
            return;
        }
        self.msaa = want;
        if want == 0 {
            self.msaa_view = None;
            self.msaa_depth = None;
            self.msaa_pipes = None; // free the variant set
        } else {
            self.rebuild_msaa_targets();
            if self.msaa_pipes.is_none() {
                let pipes = build_scene_pipes(
                    &self.device,
                    &self.world_bgl,
                    &self.line_bgl,
                    &self.cloud_bgl,
                    &self.part_bgl,
                    want as u32,
                );
                self.msaa_pipes = Some(pipes);
            }
        }
    }

    /// current MSAA sample count (0/4/8 — E2E + F3)
    pub fn msaa(&self) -> u8 {
        self.msaa
    }

    /// device max MSAA (0/4/8) — settings UI hides unsupported options
    pub fn msaa_supported(&self) -> u8 {
        self.msaa_max
    }

    /// (re)create the multisampled color + depth targets at the current
    /// scene scale — called on enable, resize, and upscale changes
    fn rebuild_msaa_targets(&mut self) {
        let (w, h) = self.post_targets.scene_size();
        let samples = self.msaa as u32;
        let color = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa-color"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa-depth"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.msaa_view = Some(color.create_view(&wgpu::TextureViewDescriptor::default()));
        self.msaa_depth = Some(depth.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    /// chunk-graph occlusion culling toggle (§26)
    pub fn set_occlusion(&mut self, on: bool) {
        self.occlusion = on;
    }

    /// §17 shadow quality: rebuild the shadow map at a new resolution
    /// (1024/2048/4096). Recreates the packed-depth target + depth buffer
    /// and rebinds it into the world bind group (the terrain/water shaders
    /// read the size from the ShadowGlobals uniform each frame).
    pub fn set_shadow_quality(&mut self, px: u32) {
        // clamp to the DEVICE limit — a 4096 request on a 2048-max adapter
        // (SwiftShader/downlevel) is a validation PANIC, so it degrades to
        // the largest supported size instead of dying
        let max = self.device.limits().max_texture_dimension_2d;
        let px = px.clamp(1024, 4096).min(max);
        if px == self.shadow_px {
            return;
        }
        self.shadow_px = px;
        let extent = wgpu::Extent3d {
            width: px,
            height: px,
            depth_or_array_layers: 1,
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.shadow_tex = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let dtex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.shadow_depth = dtex.create_view(&wgpu::TextureViewDescriptor::default());
        // world_bg binds the shadow VIEW (binding 3) — rebuild it
        self.world_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world-bg"),
            layout: &self.world_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.globals_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.shadow_tex),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_samp),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.shadow_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&self.tint_view),
                },
            ],
        });
    }

    /// internal (scaled) scene size in px
    pub fn scene_size(&self) -> (u32, u32) {
        self.post_targets.scene_size()
    }

    /// current surface size in physical px (UI coordinate mapping)
    pub fn size(&self) -> (f32, f32) {
        (self.config.width as f32, self.config.height as f32)
    }

    /// drop all GPU chunk meshes (full re-mesh, e.g. smooth-lighting toggle)
    pub fn clear_meshes(&mut self) {
        self.chunks.clear();
        // dropping every RegionArena destroys its buffers (§43: no leaks)
        self.regions.clear();
        // the mesh set changed → occlusion flood cache is stale (§26)
        self.mesh_rev = self.mesh_rev.wrapping_add(1);
    }

    pub fn toggle_vsync(&mut self) {
        let target = if self.vsync {
            if self.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
                wgpu::PresentMode::AutoNoVsync
            } else {
                wgpu::PresentMode::Fifo
            }
        } else {
            wgpu::PresentMode::Fifo
        };
        self.vsync = target == wgpu::PresentMode::Fifo;
        self.config.present_mode = target;
        self.surface.configure(&self.device, &self.config);
    }

    /// current present mode, human-readable (benchmark report context)
    pub fn present_mode_name(&self) -> String {
        let name = match self.config.present_mode {
            wgpu::PresentMode::Fifo => "Fifo (vsync)",
            wgpu::PresentMode::FifoRelaxed => "FifoRelaxed (vsync, relaxed)",
            wgpu::PresentMode::Mailbox => "Mailbox",
            wgpu::PresentMode::Immediate => "Immediate (no vsync)",
            wgpu::PresentMode::AutoVsync => "AutoVsync",
            wgpu::PresentMode::AutoNoVsync => "AutoNoVsync",
            _ => "other",
        };
        format!("{name}{}", if self.vsync { "" } else { "" })
    }

    /// overwrite one 16×16 atlas tile (animated textures: frame update path —
    /// geometry is never rebuilt for a texture frame change, §20). With
    /// mipmaps active the tile's mip levels are refreshed too (vanilla
    /// uploads each animation frame's own mip chain — same behavior).
    pub fn write_atlas_tile(&mut self, tile: u16, rgba: &[u8]) {
        let tx = (tile % 16) as u32;
        let ty = (tile / 16) as u32;
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: tx * textures::TILE_PX as u32,
                    y: ty * textures::TILE_PX as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(textures::TILE_PX as u32 * 4),
                rows_per_image: Some(textures::TILE_PX as u32),
            },
            wgpu::Extent3d {
                width: textures::TILE_PX as u32,
                height: textures::TILE_PX as u32,
                depth_or_array_layers: 1,
            },
        );
        // keep the CPU atlas in sync (future full rebuilds stay correct)
        let base = ((ty as usize * textures::TILE_PX) * textures::ATLAS_SIZE
            + tx as usize * textures::TILE_PX)
            * 4;
        if base + rgba.len() <= self.atlas_data.len() {
            self.atlas_data[base..base + rgba.len()].copy_from_slice(rgba);
        }
        // mip refresh: per-tile chain from THIS frame (vanilla parity)
        if self.mip_levels > 0 {
            for (i, mip) in textures::generate_mips(rgba, self.mip_levels)
                .iter()
                .enumerate()
            {
                let level = (i + 1) as u32;
                let tile_px = (textures::TILE_PX as u32) >> level;
                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.atlas_tex,
                        mip_level: level,
                        origin: wgpu::Origin3d {
                            x: (tx * textures::TILE_PX as u32) >> level,
                            y: (ty * textures::TILE_PX as u32) >> level,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    mip,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(tile_px * 4),
                        rows_per_image: Some(tile_px),
                    },
                    wgpu::Extent3d {
                        width: tile_px,
                        height: tile_px,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    /// update an animated tile from its precomputed frames
    pub fn update_atlas_frame(&mut self, anim: &crate::textures::AnimatedTile, frame: usize) {
        if let Some(f) = anim.frames.get(frame) {
            self.write_atlas_tile(anim.tile, f);
        }
    }

    /// Upload one chunk's merged mesh (Phase 9 §14/§43: regional
    /// mega-buffers + slot sub-allocation). A remesh that fits the chunk's
    /// existing slot writes IN PLACE — repeated edits never reallocate;
    /// write_buffer calls coalesce into the next submit. Arena growth is
    /// a doubling realloc whose GPU→GPU copy is submitted strictly before
    /// the new data write (disjoint ranges either way — see grow()),
    pub fn set_chunk_mesh(&mut self, pos: ChunkPos, md: &MeshData, occl: draw::ChunkOccl) {
        let region = draw::region_of(pos);
        let v_len = md.solid.0.len() as u32;
        let i_len = md.solid.1.len() as u32;
        let w_v_len = md.water.0.len() as u32;
        let w_i_len = md.water.1.len() as u32;

        let mut solid_old = self.chunks.get(&pos).map(|c| c.solid);
        let mut water_old = self.chunks.get(&pos).and_then(|c| c.water);

        let solid = self.place_slot(
            region,
            solid_old.take(),
            v_len,
            i_len,
            &md.solid.0,
            &md.solid.1,
        );
        let water = if w_i_len == 0 {
            // no water: release any old water allocation (None = not drawn)
            if let Some(o) = water_old.take() {
                if o.i_cap > 0 || o.v_cap > 0 {
                    self.free_slot(o);
                }
            }
            None
        } else {
            Some(self.place_slot(
                region,
                water_old.take(),
                w_v_len,
                w_i_len,
                &md.water.0,
                &md.water.1,
            ))
        };

        match self.chunks.get_mut(&pos) {
            Some(c) => {
                c.solid = solid;
                c.water = water;
                c.occl = occl;
            }
            None => {
                self.chunks.insert(pos, ChunkGpu { solid, water, occl });
            }
        }
        // occl bits may have changed → occlusion flood cache is stale (§26)
        self.mesh_rev = self.mesh_rev.wrapping_add(1);
    }

    /// fit-or-replace one mesh slot in its region arena and upload the data.
    /// `old` is the chunk's previous slot (None on first upload). Returns
    /// the (possibly new) slot. Empty meshes get the null slot.
    /// Indices are baked ABSOLUTE (+ v_off) so every backend draws with
    /// base_vertex = 0 — WebGL2/GL cannot instanced-draw with a non-zero
    /// base vertex (§14 capability rules, empirically verified).
    fn place_slot(
        &mut self,
        region: (i32, i32),
        old: Option<MeshSlot>,
        v_len: u32,
        i_len: u32,
        verts: &[Vertex],
        idx: &[u32],
    ) -> MeshSlot {
        if i_len == 0 {
            // no indices → nothing drawn; release any old allocation
            if let Some(o) = old {
                if o.i_cap > 0 || o.v_cap > 0 {
                    self.free_slot(o);
                }
            }
            return MeshSlot::EMPTY;
        }
        // ---- in-place reuse (§14): new data fits the old slot
        if let Some(o) = &old {
            if v_len <= o.v_cap && i_len <= o.i_cap {
                let r = self.regions.get(&region).unwrap();
                self.queue.write_buffer(
                    &r.v,
                    o.v_off as u64 * VERT_SIZE,
                    bytemuck::cast_slice(verts),
                );
                let baked = bake_absolute_indices(idx, o.v_off);
                self.queue
                    .write_buffer(&r.i, o.i_off as u64 * 4, bytemuck::cast_slice(&baked));
                return MeshSlot {
                    region,
                    v_off: o.v_off,
                    v_cap: o.v_cap,
                    i_off: o.i_off,
                    i_cap: o.i_cap,
                    n: i_len,
                };
            }
        }
        // ---- doesn't fit (or first upload): allocate the new slot BEFORE
        // releasing the old one — if this was the region's only live slot,
        // releasing first would destroy the arena out from under us. The
        // region arena itself is created lazily here (only when a mesh
        // actually needs GPU space).
        if !self.regions.contains_key(&region) {
            self.regions.insert(
                region,
                RegionArena {
                    v: self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("region-v"),
                        size: REGION_START_V_ELEMS as u64 * VERT_SIZE,
                        usage: arena_v_usage(),
                        mapped_at_creation: false,
                    }),
                    i: self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("region-i"),
                        size: REGION_START_I_ELEMS as u64 * 4,
                        usage: arena_i_usage(),
                        mapped_at_creation: false,
                    }),
                    v_elems: REGION_START_V_ELEMS,
                    i_elems: REGION_START_I_ELEMS,
                    va: SlotAlloc::default(),
                    ia: SlotAlloc::default(),
                },
            );
        }
        let (v_off, _) = self.regions.get_mut(&region).unwrap().va.alloc(v_len);
        let (i_off, _) = self.regions.get_mut(&region).unwrap().ia.alloc(i_len);
        self.grow_region_if_needed(region);
        if let Some(o) = old {
            if o.i_cap > 0 || o.v_cap > 0 {
                self.free_slot(o);
            }
        }
        let r = self.regions.get(&region).unwrap();
        self.queue
            .write_buffer(&r.v, v_off as u64 * VERT_SIZE, bytemuck::cast_slice(verts));
        let baked = bake_absolute_indices(idx, v_off);
        self.queue
            .write_buffer(&r.i, i_off as u64 * 4, bytemuck::cast_slice(&baked));
        MeshSlot {
            region,
            v_off,
            v_cap: v_len,
            i_off,
            i_cap: i_len,
            n: i_len,
        }
    }

    /// grow the region's buffers when the freshly bump-allocated slots
    /// exceed capacity: new doubled buffers, GPU→GPU copy of the old
    /// contents submitted FIRST, then the caller's write_buffer (staged
    /// for the next submit) lands strictly after — so the fresh data
    /// always wins in any overlapping range (§43: no synchronized stall,
    /// no host round-trip).
    fn grow_region_if_needed(&mut self, region: (i32, i32)) {
        let r = self.regions.get(&region).unwrap();
        let need_v = r.va.used();
        let need_i = r.ia.used();
        if need_v <= r.v_elems && need_i <= r.i_elems {
            return;
        }
        // copy the whole old capacity — every live slot's data lands at
        // the identical offsets in the new buffer
        let copy_v = r.v_elems as u64 * VERT_SIZE;
        let copy_i = r.i_elems as u64 * 4;
        let new_v_elems = draw::grow_plan(need_v, r.v_elems);
        let new_i_elems = draw::grow_plan(need_i, r.i_elems);
        let new_v = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("region-v"),
            size: new_v_elems as u64 * VERT_SIZE,
            usage: arena_v_usage(),
            mapped_at_creation: false,
        });
        let new_i = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("region-i"),
            size: new_i_elems as u64 * 4,
            usage: arena_i_usage(),
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("arena-grow"),
            });
        {
            let r = self.regions.get(&region).unwrap();
            if copy_v > 0 {
                enc.copy_buffer_to_buffer(&r.v, 0, &new_v, 0, copy_v);
            }
            if copy_i > 0 {
                enc.copy_buffer_to_buffer(&r.i, 0, &new_i, 0, copy_i);
            }
        }
        // submit the copy BEFORE the caller's write_buffer stages for the
        // NEXT submit → deterministic ordering (§43). The old buffers are
        // destroyed by the field swap below — wgpu defers actual GPU-side
        // destruction until the submitted copy completes.
        self.queue.submit(Some(enc.finish()));
        let r = self.regions.get_mut(&region).unwrap();
        r.v = new_v;
        r.i = new_i;
        r.v_elems = new_v_elems;
        r.i_elems = new_i_elems;
    }

    /// return a slot's ranges to the region's free pools; destroy the
    /// region's buffers entirely when no live slot remains (§43)
    fn free_slot(&mut self, s: MeshSlot) {
        let mut dead = false;
        if let Some(r) = self.regions.get_mut(&s.region) {
            r.va.release(s.v_off, s.v_cap);
            r.ia.release(s.i_off, s.i_cap);
            dead = r.va.is_empty() && r.ia.is_empty();
        }
        if dead {
            self.regions.remove(&s.region);
        }
    }

    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        if let Some(c) = self.chunks.remove(&pos) {
            if c.solid.i_cap > 0 || c.solid.v_cap > 0 {
                self.free_slot(c.solid);
            }
            if let Some(w) = c.water {
                if w.i_cap > 0 || w.v_cap > 0 {
                    self.free_slot(w);
                }
            }
            // the mesh set changed → occlusion flood cache is stale (§26)
            self.mesh_rev = self.mesh_rev.wrapping_add(1);
        }
    }

    pub fn has_chunk(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Phase 9 §14 — active submission path (F3 / bench context)
    pub fn draw_path_name(&self) -> &'static str {
        if self.draw_mdi {
            "MDI (multi-draw-indirect)"
        } else {
            "region-loop (zero per-chunk binds)"
        }
    }

    /// Phase 11 §34 — install (or clear) a shader pack. Grade-only packs
    /// just override the engine grade; packs with a composite stage get a
    /// pipeline built from the WRAPPED, naga-validated WGSL. Invalid packs
    /// are rejected with a boot-log line and the previous state kept (§46
    /// resilience — a bad pack never takes the renderer down).
    pub fn set_shader_pack(&mut self, pack: Option<&crate::shaders::ShaderPack>) {
        let Some(p) = pack else {
            self.pack_pipe = None;
            self.pack_grade = None;
            self.pack_id = None;
            self.pack_tier.clear();
            self.active_pack_src = None;
            return;
        };
        self.pack_grade = Some(p.grade.as_q());
        self.pack_id = Some(p.id.clone());
        self.pack_tier = p.tier.clone();
        self.active_pack_src = Some(p.clone());
        match p.composite.as_deref() {
            None => {
                self.pack_pipe = None; // grade-only pack
            }
            Some(src) => {
                let wrapped = crate::shaders::wrap_composite(src);
                if let Err(e) = crate::shaders::validate_wgsl(&wrapped) {
                    report_boot_log(&format!("shader pack {} rejected at install: {}", p.id, e));
                    self.pack_pipe = None;
                    return;
                }
                let module = self
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("pack-composite"),
                        source: wgpu::ShaderSource::Wgsl(wrapped.into()),
                    });
                let pipe =
                    make_fullscreen_pipe(&self.device, &module, &self.comp_pl, self.config.format);
                let bg = Self::comp_bg(
                    &self.device,
                    &self.comp_bgl,
                    &self.post_targets.pack_view,
                    &self.post_targets.b2_view,
                    &self.post_samp,
                    &self.pack_buf,
                );
                self.pack_pipe = Some((pipe, bg));
            }
        }
    }

    /// pack settings row 0 → PackUniform.params (v1: the pack's declared
    /// defaults — first slider's default, or 1.0 = neutral)
    fn pack_params_row(p: &crate::shaders::ShaderPack) -> [f32; 4] {
        let x = p
            .settings
            .first()
            .map(|s| s.default.clamp(s.min, s.max))
            .unwrap_or(1.0);
        [x, 0.0, 0.0, 0.0]
    }

    /// Phase 9 §14 — submit one pass's draw list. The origin instance
    /// buffer must already be bound at slot 1 (whole buffer). Issues either
    /// MDI (one multi_draw per region run) or the zero-rebind loop (one
    /// draw per chunk, arena re-bound only at region transitions).
    /// `count_chunks` keeps the legacy F3 semantics (terrain pass only).
    fn issue_draws(
        &self,
        pass: &mut wgpu::RenderPass,
        list: &[DrawCmd],
        args_off: u64,
        stats: &mut RenderStats,
    ) {
        if self.draw_mdi {
            let runs = draw::region_runs(list);
            let mut cur: Option<(i32, i32)> = None;
            for &(region, start, count) in runs.iter() {
                if cur != Some(region) {
                    let r = self.regions.get(&region).expect("draw region missing");
                    pass.set_vertex_buffer(0, r.v.slice(..));
                    pass.set_index_buffer(r.i.slice(..), wgpu::IndexFormat::Uint32);
                    cur = Some(region);
                    stats.binds += 2;
                }
                pass.multi_draw_indexed_indirect(
                    &self.args_buf,
                    args_off + start as u64 * IndirectArgs::SIZE,
                    count as u32,
                );
                stats.draws += 1;
            }
        } else {
            let mut cur: Option<(i32, i32)> = None;
            for c in list.iter() {
                if cur != Some(c.region) {
                    let r = self.regions.get(&c.region).expect("draw region missing");
                    pass.set_vertex_buffer(0, r.v.slice(..));
                    pass.set_index_buffer(r.i.slice(..), wgpu::IndexFormat::Uint32);
                    cur = Some(c.region);
                    stats.binds += 2;
                }
                pass.draw_indexed(
                    c.i_first..c.i_first + c.i_count,
                    0, // base_vertex always 0 — arena indices are absolute
                    c.origin..c.origin + 1,
                );
                stats.draws += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        cam: &Camera,
        sky: &SkyState,
        ui: &mut UiCanvas,
        selection: Option<(i32, i32, i32)>,
        post: &PostParams,
        clouds: bool,
        particles: &[vc_particles::particles::ParticleVertex],
    ) -> RenderStats {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                // NOTE: this was silently swallowing per-frame failures
                report_boot_log(&format!("get_current_texture failed: {e:?}"));
                self.surface.configure(&self.device, &self.config);
                return RenderStats::default();
            }
        };
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let aspect = self.config.width as f32 / self.config.height as f32;
        let dir = Vec3::new(
            cam.yaw.sin() * cam.pitch.cos(),
            cam.pitch.sin(),
            -cam.yaw.cos() * cam.pitch.cos(),
        );
        let view_m = Mat4::look_at_rh(cam.eye, cam.eye + dir, Vec3::Y);
        let proj = Mat4::perspective_rh(cam.fov, aspect, 0.1, sky.fog_end + 96.0);
        let vp = proj * view_m;
        let inv_vp = vp.inverse();

        let globals = Globals {
            view_proj: vp.to_cols_array_2d(),
            inv_view_proj: inv_vp.to_cols_array_2d(),
            cam: [cam.eye.x, cam.eye.y, cam.eye.z, 0.0],
            fog_color: [
                sky.fog_color[0],
                sky.fog_color[1],
                sky.fog_color[2],
                sky.fog_start,
            ],
            sun_dir: [sky.sun_dir.x, sky.sun_dir.y, sky.sun_dir.z, sky.fog_end],
            misc: [
                sky.day_light,
                sky.time,
                if sky.underwater { 1.0 } else { 0.0 },
                sky.min_light,
            ],
        };
        self.queue
            .write_buffer(&self.globals_buf, 0, bytemuck::bytes_of(&globals));

        // UI uniform: letterboxed mapping canvas → screen
        let sw = self.config.width as f32;
        let sh = self.config.height as f32;
        let scale = (sw / UI_W as f32).min(sh / UI_H as f32);
        let x0 = (sw - UI_W as f32 * scale) * 0.5;
        let y0 = (sh - UI_H as f32 * scale) * 0.5;
        let ui_map = UiUniform {
            map: [
                2.0 * scale / sw,
                2.0 * x0 / sw - 1.0,
                -2.0 * scale / sh,
                1.0 - 2.0 * y0 / sh,
            ],
        };
        self.queue
            .write_buffer(&self.ui_buf, 0, bytemuck::bytes_of(&ui_map));

        // upload ui canvas if dirty
        if ui.dirty {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.ui_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &ui.px,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(UI_W as u32 * 4),
                    rows_per_image: Some(UI_H as u32),
                },
                wgpu::Extent3d {
                    width: UI_W as u32,
                    height: UI_H as u32,
                    depth_or_array_layers: 1,
                },
            );
            ui.dirty = false;
        }

        // post uniform: mode, menu_blur, time, aspect | bloom, vig, sat, exp
        // | sharpen, scene texel size (FSR-lite RCAS uses the SCENE-resolution
        // texel so sharpening matches the upscaled pixels, not surface px)
        // Phase 11 §34: an active shader pack OVERRIDES the grade row with
        // its declared preset (packs without a composite are grade-only)
        let (bloom, vig, sat, exp) = if let Some([b, v, s, e]) = self.pack_grade {
            (b, v, s, e)
        } else {
            match post.mode {
                1 => (0.55, 0.14, 1.07, 1.0),  // vanilla+
                2 => (0.85, 0.32, 1.16, 1.06), // cinematic
                _ => (0.0, 0.0, 1.0, 1.0),     // off
            }
        };
        let post_u = PostUniform {
            p: [
                post.mode as f32,
                post.menu_blur.clamp(0.0, 1.0),
                sky.time,
                aspect,
            ],
            q: [bloom, vig, sat, exp],
            // FSR 1.0 RCAS lobe scale: post.sharpen maps 0..1 → the
            // FsrRcasCon factor (1.0 = maximum sharpness, 0 = off)
            s: [post.sharpen.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.post_buf, 0, bytemuck::bytes_of(&post_u));

        // Phase 11 §34: bridge engine state → the active pack's uniforms
        // (OptiFine-style alias subset — see shaders.rs PackUniform docs)
        if let Some(p) = &self.active_pack_src {
            let u = crate::shaders::PackUniform {
                params: Self::pack_params_row(p),
                viewport: [sw, sh, 1.0 / sw.max(1.0), 1.0 / sh.max(1.0)],
                time: [
                    sky.time,
                    sky.day_light,
                    if sky.underwater { 1.0 } else { 0.0 },
                    sky.min_light,
                ],
            };
            self.queue
                .write_buffer(&self.pack_buf, 0, bytemuck::bytes_of(&u));
        }

        // ──────────────────────────────── sun shadow camera + globals ──
        // Ortho box following the player, aligned to the sun. The light-space
        // center is snapped to shadow-map texels so camera movement doesn't
        // make the shadows swim. Disabled at night / when strength = 0.
        let sun_up = sky.sun_dir.y > 0.06;
        let sh_strength = if sun_up {
            post.shadows.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let mut sh_globals = ShadowGlobals {
            shadow_vp: Mat4::IDENTITY.to_cols_array_2d(),
            params: [0.0, 0.0, 0.0, 0.0],
            size: [self.shadow_px as f32, 0.0, 0.0, 0.0],
        };
        if sh_strength > 0.0 {
            const SHADOW_R: f32 = 110.0;
            const SHADOW_FAR: f32 = 420.0;
            let texel = 2.0 * SHADOW_R / self.shadow_px as f32;
            let center0 = Vec3::new(cam.eye.x, 0.0, cam.eye.z);
            let light_pos0 = center0 + sky.sun_dir * (SHADOW_FAR * 0.5);
            let view0 = Mat4::look_at_rh(light_pos0, center0, Vec3::Y);
            // snap the center in light space to texel grid (kills shimmer)
            let c_l = view0 * Vec4::new(center0.x, center0.y, center0.z, 1.0);
            let snapped = Vec3::new(
                (c_l.x / texel).round() * texel,
                (c_l.y / texel).round() * texel,
                c_l.z,
            );
            let center =
                (view0.inverse() * Vec4::new(snapped.x, snapped.y, snapped.z, 1.0)).truncate();
            let light_pos = center + sky.sun_dir * (SHADOW_FAR * 0.5);
            let view = Mat4::look_at_rh(light_pos, center, Vec3::Y);
            let ortho =
                Mat4::orthographic_rh(-SHADOW_R, SHADOW_R, -SHADOW_R, SHADOW_R, 0.1, SHADOW_FAR);
            let sh_vp = ortho * view;
            sh_globals = ShadowGlobals {
                shadow_vp: sh_vp.to_cols_array_2d(),
                params: [1.0, sh_strength, 90.0, 110.0],
                size: [self.shadow_px as f32, 0.0, 0.0, 0.0],
            };
        }
        self.queue
            .write_buffer(&self.shadow_buf, 0, bytemuck::bytes_of(&sh_globals));
        let shadows_on = sh_strength > 0.0;

        // selection line uniform
        let line_u = LineUniform {
            vp: vp.to_cols_array_2d(),
            offset: [
                selection.map(|s| s.0 as f32).unwrap_or(0.0),
                selection.map(|s| s.1 as f32).unwrap_or(0.0),
                selection.map(|s| s.2 as f32).unwrap_or(0.0),
                0.0,
            ],
            color: [0.0, 0.0, 0.0, 0.55],
        };
        self.queue
            .write_buffer(&self.line_buf, 0, bytemuck::bytes_of(&line_u));

        // NOTE: blur direction uniforms (aux_h_buf / aux_v_buf) are written
        // once at init and refreshed in rebuild_post_targets() — per-frame
        // mid-encoder writes cannot take effect between passes (see struct
        // field comment).

        // frustum planes from vp (rows)
        let rows: [Vec4; 4] = [vp.row(0), vp.row(1), vp.row(2), vp.row(3)];
        let combine = |a: &Vec4, b: &Vec4| [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
        let planes: [[f32; 4]; 6] = [
            combine(&rows[3], &rows[0]),  // left
            combine(&rows[3], &-rows[0]), // right
            combine(&rows[3], &rows[1]),  // bottom
            combine(&rows[3], &-rows[1]), // top
            combine(&rows[3], &rows[2]),  // near
            combine(&rows[3], &-rows[2]), // far
        ];

        let visible: Vec<(ChunkPos, f32)> = self
            .chunks
            .iter()
            .filter(|(pos, _)| {
                let min = [pos.0 as f32 * 16.0, 0.0, pos.1 as f32 * 16.0];
                let max = [min[0] + 16.0, 256.0, min[2] + 16.0];
                for p in planes.iter() {
                    // p-vertex test
                    let px = if p[0] >= 0.0 { max[0] } else { min[0] };
                    let py = if p[1] >= 0.0 { max[1] } else { min[1] };
                    let pz = if p[2] >= 0.0 { max[2] } else { min[2] };
                    if p[0] * px + p[1] * py + p[2] * pz + p[3] < 0.0 {
                        return false;
                    }
                }
                true
            })
            .map(|(pos, _)| {
                let dx = pos.0 as f32 * 16.0 + 8.0 - cam.eye.x;
                let dz = pos.1 as f32 * 16.0 + 8.0 - cam.eye.z;
                (*pos, dx * dx + dz * dz)
            })
            .collect();

        let mut stats = RenderStats::default();

        // Sort once (near → far) — the per-frame origin rows are indexed by
        // this order (identical in all three passes, as before). `visible`
        // is not used afterwards — moved, not cloned (rendering-cost fix).
        let mut sorted = visible;
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // per-frame instance-rate origins: one Float32x2 per visible chunk
        let draw_count = sorted.len().min(MAX_DRAW_CHUNKS);
        let mut origin_data: Vec<[f32; 2]> = Vec::with_capacity(draw_count);
        for (pos, _) in sorted.iter().take(draw_count) {
            origin_data.push([pos.0 as f32 * 16.0, pos.1 as f32 * 16.0]);
        }
        if draw_count > 0 {
            self.queue
                .write_buffer(&self.origin_vb, 0, bytemuck::cast_slice(&origin_data));
        }

        // ──────────────────────── Phase 6 §26: occlusion culling ──
        // The chunk-graph flood (see draw::occlusion_visible) runs on the
        // SECTION grid from the camera's own section; a column is drawable
        // when any of its geometry bands is reachable. None = camera chunk
        // not meshed yet → skip (draw everything frustum-visible). The
        // SHADOW list deliberately stays unfiltered: sun shadows of
        // camera-occluded columns are still visible in view.
        let vis_full: Vec<VisEntry> = sorted
            .iter()
            .take(draw_count)
            .enumerate()
            .map(|(i, &(pos, d))| (pos, d, i as u32))
            .collect();
        let vis: Vec<VisEntry> = if self.occlusion {
            let cam_chunk = (
                cam.eye.x.div_euclid(16.0) as i32,
                cam.eye.z.div_euclid(16.0) as i32,
            );
            let cam_band = (cam.eye.y.clamp(0.0, 255.99) as u32 >> 4) as u8;
            // §26 rendering-cost fix: cached flood — the BFS re-runs only
            // when the camera section or the mesh set (mesh_rev) changes,
            // not every frame (see draw::OcclCache).
            match draw::occlusion_visible_cached(
                &self.chunks,
                cam_chunk,
                cam_band,
                self.mesh_rev,
                &mut self.occl_cache,
            ) {
                Some(vis_set) => {
                    let before = vis_full.len();
                    let v: Vec<VisEntry> = vis_full
                        .iter()
                        .copied()
                        .filter(|(pos, _, _)| vis_set.contains(pos))
                        .collect();
                    stats.culled = (before - v.len()) as u32;
                    v
                }
                None => vis_full.clone(),
            }
        } else {
            vis_full.clone()
        };

        // ─────────────────────── Phase 9: region-grouped draw lists (§14) ──
        // origin row = index in `sorted`; draw order = region-major near→far
        // so chunks of one 8×8 region are contiguous → one arena bind per
        // region run, zero per-chunk buffer binds (water: far→near).
        let cam2 = (cam.eye.x, cam.eye.z);
        let terrain_order = draw::order_by_region(&vis, cam2, false);
        let water_order: Vec<VisEntry> = terrain_order.iter().rev().copied().collect();
        let terrain_list = draw::build_draw_list(&self.chunks, &terrain_order, false, None);
        let water_list = draw::build_draw_list(&self.chunks, &water_order, true, None);
        // 110 u shadow radius + one chunk margin (16√2 ≈ 23) — built from
        // the FULL frustum-visible set (not occlusion-filtered: shadows of
        // hidden columns still land in view)
        let shadow_list = if shadows_on {
            let shadow_order = draw::order_by_region(&vis_full, cam2, false);
            draw::build_draw_list(
                &self.chunks,
                &shadow_order,
                false,
                Some((110.0 + 23.0) * (110.0 + 23.0)),
            )
        } else {
            Vec::new()
        };
        // same counting semantics as the old inline loops (drawn chunks of
        // the terrain pass; empty/missing were skipped there, skipped here)
        stats.chunks = terrain_list.len() as u32;
        stats.tris = terrain_list.iter().map(|c| c.i_count / 3).sum();

        // args buffer segments: [terrain | shadow | water]
        let args_shadow_off = MAX_DRAW_CHUNKS as u64 * IndirectArgs::SIZE;
        let args_water_off = 2 * args_shadow_off;
        if self.draw_mdi {
            let t = draw::pack_args(&terrain_list);
            let s = draw::pack_args(&shadow_list);
            let w = draw::pack_args(&water_list);
            if !t.is_empty() {
                self.queue
                    .write_buffer(&self.args_buf, 0, bytemuck::cast_slice(&t));
            }
            if !s.is_empty() {
                self.queue
                    .write_buffer(&self.args_buf, args_shadow_off, bytemuck::cast_slice(&s));
            }
            if !w.is_empty() {
                self.queue
                    .write_buffer(&self.args_buf, args_water_off, bytemuck::cast_slice(&w));
            }
        }

        // ────────────────────────────────────────── pass 0: sun shadows ──
        // Depth-only re-render of the terrain from the light's ortho camera
        // into the 2048² packed-depth map. This runs in its OWN command
        // encoder + submit: a wgpu command buffer is one usage scope, so the
        // shadow texture can be COLOR_TARGET here and RESOURCE (sampled via
        // the world bind group) only in the NEXT encoder, after a queue-order
        // barrier between the two submits.
        if shadows_on {
            let mut sh_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("shadow"),
                    });
            let sh_att = wgpu::RenderPassColorAttachment {
                view: &self.shadow_tex,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE), // depth 1.0
                    store: wgpu::StoreOp::Store,
                },
            };
            let sh_depth_att = wgpu::RenderPassDepthStencilAttachment {
                view: &self.shadow_depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            };
            {
                let mut pass = sh_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow"),
                    color_attachments: &[Some(sh_att)],
                    depth_stencil_attachment: Some(sh_depth_att),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.shadow_pipe);
                pass.set_bind_group(0, &self.shadow_bg, &[]);
                // Phase 9: whole origin buffer + region-arena draws — no
                // per-chunk buffer binds
                pass.set_vertex_buffer(1, self.origin_vb.slice(..));
                self.issue_draws(&mut pass, &shadow_list, args_shadow_off, &mut stats);
                stats.binds += 1; // the origin bind
            }
            self.queue.submit([sh_encoder.finish()]);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        // ─────────────────────────────────────────────── pass 1: scene ──
        // (sky + terrain + selection + water + clouds → offscreen LINEAR
        // scene texture — the composite encodes to srgb once at the end)
        // Phase 6 §26: with MSAA on, the scene renders into a multisampled
        // color+depth pair and resolves into the scene target automatically
        // at pass end (driver resolve — the Rgba8Unorm+X4+RESOLVE support
        // was capability-checked at boot); the post chain stays 1x.
        let msaa_on = self.msaa > 0 && self.msaa_view.is_some();
        let color_view: &wgpu::TextureView = if msaa_on {
            self.msaa_view.as_ref().unwrap()
        } else {
            &self.post_targets.scene_view
        };
        let resolve: Option<&wgpu::TextureView> = if msaa_on {
            Some(&self.post_targets.scene_view)
        } else {
            None
        };
        let clear = wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target: resolve,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: sky.fog_color[0] as f64,
                    g: sky.fog_color[1] as f64,
                    b: sky.fog_color[2] as f64,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        };
        let depth_view: &wgpu::TextureView = if msaa_on {
            self.msaa_depth.as_ref().unwrap()
        } else {
            &self.depth
        };
        let depth_att = wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };

        // pipeline set: the MSAA variants when active, else the 1x set
        let (sky_p, terrain_p, line_p, water_p, part_p, cloud_p) =
            if let (true, Some(p)) = (msaa_on, &self.msaa_pipes) {
                (&p.sky, &p.terrain, &p.line, &p.water, &p.part, &p.cloud)
            } else {
                (
                    &self.sky_pipe,
                    &self.terrain_pipe,
                    &self.line_pipe,
                    &self.water_pipe,
                    &self.part_pipe,
                    &self.cloud_pipe,
                )
            };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(clear)],
                depth_stencil_attachment: Some(depth_att),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. sky (§28: skipped in skyless dimensions — the nether's
            // fog-colored clear color IS the sky)
            if !sky.skyless {
                pass.set_pipeline(sky_p);
                pass.set_bind_group(0, &self.world_bg, &[]);
                pass.draw(0..3, 0..1);
            }

            // 2. terrain — Phase 9: region-grouped near→far (approximately
            // front-to-back for early-z), whole origin buffer bound once,
            // arena re-bound once per region run; MDI where supported
            pass.set_pipeline(terrain_p);
            pass.set_bind_group(0, &self.world_bg, &[]);
            pass.set_vertex_buffer(1, self.origin_vb.slice(..));
            self.issue_draws(&mut pass, &terrain_list, 0, &mut stats);
            stats.binds += 1; // the origin bind

            // 3. selection wireframe
            if selection.is_some() {
                pass.set_pipeline(line_p);
                pass.set_bind_group(0, &self.line_bg, &[]);
                pass.set_vertex_buffer(0, self.line_vb.slice(..));
                pass.draw(0..24, 0..1);
            }

            // 4. water (far → near, blended) — reversed region-major order,
            // same origin rows, same zero-rebind submission
            pass.set_pipeline(water_p);
            pass.set_bind_group(0, &self.world_bg, &[]);
            pass.set_vertex_buffer(1, self.origin_vb.slice(..));
            self.issue_draws(&mut pass, &water_list, args_water_off, &mut stats);
            stats.binds += 1; // the origin bind

            // 4.5 particles (§16.2 pass 4): billboard quads uploaded per
            // frame, alpha-blended, depth-tested but not written — after
            // the translucent water pass, before clouds
            if !particles.is_empty() {
                let bytes = bytemuck::cast_slice(particles);
                self.queue.write_buffer(&self.particle_vb, 0, bytes);
                pass.set_pipeline(part_p);
                pass.set_bind_group(0, &self.part_bg, &[]);
                pass.set_vertex_buffer(0, self.particle_vb.slice(..));
                let n =
                    (particles.len() as u32).min(vc_particles::particles::MAX_PARTICLES as u32 * 6);
                pass.draw(0..n, 0..1);
                stats.particles += n / 6;
            }

            // 5. clouds (translucent plane above the world)
            if clouds {
                pass.set_pipeline(cloud_p);
                pass.set_bind_group(0, &self.cloud_bg, &[]);
                pass.set_vertex_buffer(0, self.cloud_vb.slice(..));
                pass.draw(0..6, 0..1);
            }
        }

        // ─────────────────────────────────────── pass 2/3: bloom pyramid ──
        if post.mode > 0 {
            // bright: scene → q (1/4)
            {
                let att = wgpu::RenderPassColorAttachment {
                    view: &self.post_targets.q_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bright"),
                    color_attachments: &[Some(att)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bright_pipe);
                pass.set_bind_group(0, &self.bg_scene, &[]);
                pass.draw(0..3, 0..1);
            }
            // blur h: q → b1 (1/8)
            {
                let att = wgpu::RenderPassColorAttachment {
                    view: &self.post_targets.b1_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blur-h"),
                    color_attachments: &[Some(att)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.blur_pipe);
                pass.set_bind_group(0, &self.bg_q, &[]);
                pass.draw(0..3, 0..1);
            }
            // blur v: b1 → b2 (1/8) — bg_b1 binds aux_v_buf (vertical step)
            {
                let att = wgpu::RenderPassColorAttachment {
                    view: &self.post_targets.b2_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blur-v"),
                    color_attachments: &[Some(att)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.blur_pipe);
                pass.set_bind_group(0, &self.bg_b1, &[]);
                pass.draw(0..3, 0..1);
            }
        }

        // ───────────────────── pass 3.5: FSR 1.0 EASU: scene → up ──
        // Edge-adaptive spatial upsampling to the full surface resolution.
        // Mathematically identity at 1:1 scale (verified in EASU_SHADER
        // notes), so it runs at every upscale setting.
        {
            let att = wgpu::RenderPassColorAttachment {
                view: &self.post_targets.up_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fsr-easu"),
                color_attachments: &[Some(att)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.easu_pipe);
            pass.set_bind_group(0, &self.bg_easu, &[]);
            pass.draw(0..3, 0..1);
        }

        // ─────────── pass 4: composite → surface (or pack handoff) ──
        // (reads the EASU-upscaled target; applies RCAS sharpening when
        // enabled, then bloom + grade + vignette). With an active pack
        // STAGE the composite writes the LINEAR pack handoff target and
        // pass 4.5 encodes to srgb on the surface (Phase 11 §34).
        {
            let pack_active = self.pack_pipe.is_some();
            let out_view: &wgpu::TextureView = if pack_active {
                &self.post_targets.pack_view
            } else {
                &frame_view
            };
            let att = wgpu::RenderPassColorAttachment {
                view: out_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[Some(att)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // pack path: LINEAR pipeline writing the LINEAR handoff target
            if pack_active {
                pass.set_pipeline(&self.post_pipe_linear);
            } else {
                pass.set_pipeline(&self.post_pipe);
            }
            pass.set_bind_group(0, &self.bg_comp, &[]);
            pass.draw(0..3, 0..1);
        }

        // ───────── pass 4.5: shader-pack composite → surface (§34) ──
        // The pack's packGrade() runs on the LINEAR composite output + the
        // bloom buffer; its result is srgb-encoded by the surface write.
        if let Some((pipe, bg)) = &self.pack_pipe {
            let att = wgpu::RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shader-pack"),
                color_attachments: &[Some(att)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // ───────────────────────────────────── pass 5: UI → surface ──
        // (crisp, unblurred, alpha-blended over the final composited image)
        {
            let att = wgpu::RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui"),
                color_attachments: &[Some(att)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.ui_pipe);
            pass.set_bind_group(0, &self.ui_bg, &[]);
            pass.set_vertex_buffer(0, self.ui_vb.slice(..));
            pass.set_index_buffer(self.ui_ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..6, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        if self.submitted_frames < 3 {
            self.submitted_frames += 1;
            report_boot_log(&format!(
                "frame submitted #{}: format={:?} alpha={:?} {}x{} post_mode={} blur={:.2}",
                self.submitted_frames,
                self.config.format,
                self.config.alpha_mode,
                self.config.width,
                self.config.height,
                post.mode,
                post.menu_blur
            ));
        }

        stats
    }
}

/// full-screen triangle pipeline (shared by the post chain and Phase-11
/// pack composites — extracted from Renderer::new's make_fs_pipe)
fn make_fullscreen_pipe(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    out_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fs-pipe"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: out_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

/// Choose the GPU backend on wasm by ACTUALLY requesting adapters with the
/// same options wgpu will use later:
///   1. hardware adapter:  {powerPreference: 'high-performance', forceFallbackAdapter: false}
///   2. software adapter:  {forceFallbackAdapter: true}   (blocklisted/absent GPUs)
///   3. else: WebGL2 (GL backend)
/// Returns Some(force_fallback_adapter) when WebGPU is viable, None → GL.
/// (navigator.gpu presence alone is insufficient — headless Chromium exposes
/// the API but returns null adapters, and wgpu locks an instance to
/// WebGPU-only mode whenever it detects the API at all.)
#[cfg(target_arch = "wasm32")]
async fn choose_webgpu_mode() -> Option<bool> {
    if request_adapter_js(Some("high-performance"), false).await {
        return Some(false);
    }
    if request_adapter_js(None, true).await {
        return Some(true);
    }
    None
}

/// Call navigator.gpu.requestAdapter(options) via JS reflection (avoids
/// extra web-sys features) and report whether an adapter came back.
#[cfg(target_arch = "wasm32")]
async fn request_adapter_js(power_preference: Option<&str>, force_fallback: bool) -> bool {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return false;
    };
    let window_val: wasm_bindgen::JsValue = window.into();
    let navigator = match js_sys::Reflect::get(&window_val, &"navigator".into()) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let gpu = match js_sys::Reflect::get(&navigator, &"gpu".into()) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if gpu.is_null() || gpu.is_undefined() {
        return false;
    }
    let request_adapter = match js_sys::Reflect::get(&gpu, &"requestAdapter".into()) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let Ok(func) = request_adapter.dyn_into::<js_sys::Function>() else {
        return false;
    };

    // build the options object exactly like wgpu will
    let opts = js_sys::Object::new();
    if let Some(pref) = power_preference {
        let _ = js_sys::Reflect::set(
            &opts,
            &"powerPreference".into(),
            &wasm_bindgen::JsValue::from_str(pref),
        );
    }
    let _ = js_sys::Reflect::set(
        &opts,
        &"forceFallbackAdapter".into(),
        &wasm_bindgen::JsValue::from_bool(force_fallback),
    );

    let result = match func.call1(&gpu, &opts) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let Ok(promise) = result.dyn_into::<js_sys::Promise>() else {
        return false;
    };
    let Ok(value) = wasm_bindgen_futures::JsFuture::from(promise).await else {
        return false;
    };
    !value.is_null() && !value.is_undefined()
}

/// Surface a fatal init error on the page (wasm) / stderr (native),
/// instead of silently panicking into a blank screen.
#[allow(dead_code)]
pub fn report_boot_error(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        // page-level boot overlay hook (same JS contract as the app's
        // wasm_entry::boot_error; kept here so the library stays app-free)
        use wasm_bindgen::{JsCast, JsValue};
        if let Some(window) = web_sys::window() {
            let w: JsValue = window.into();
            if let Ok(f) = js_sys::Reflect::get(&w, &"voxelcraftBootError".into()) {
                if let Ok(f) = f.dyn_into::<js_sys::Function>() {
                    let _ = f.call1(&w, &JsValue::from_str(msg));
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[voxelcraft] {msg}");
}

/// Best-effort diagnostic log (wasm: JS console, native: stderr).
#[allow(dead_code)]
pub fn report_boot_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[voxelcraft] {msg}");
}

#[cfg(test)]
mod shader_tests {
    use super::*;

    /// All embedded WGSL must parse AND type-check with the exact naga
    /// version wgpu 22 ships (dev-dependency) — catches shader errors in
    /// `cargo test` instead of at device-creation time on a real GPU.
    #[test]
    fn wgsl_shaders_validate() {
        let shaders: &[(&str, &str)] = &[
            ("terrain", TERRAIN_SHADER),
            ("water", WATER_SHADER),
            ("sky", SKY_SHADER),
            ("cloud", CLOUD_SHADER),
            ("ui", UI_SHADER),
            ("line", LINE_SHADER),
            ("bright", BRIGHT_SHADER),
            ("blur", BLUR_SHADER),
            ("post", POST_SHADER),
            ("fsr-easu", EASU_SHADER),
            ("shadow", SHADOW_SHADER),
            ("particle", PARTICLE_SHADER),
        ];
        let mut frontend = naga::front::wgsl::Frontend::new();
        for (name, src) in shaders {
            let module = frontend
                .parse(src)
                .unwrap_or_else(|e| panic!("{name} WGSL parse failed: {e}"));
            let mut validator = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            );
            validator
                .validate(&module)
                .unwrap_or_else(|e| panic!("{name} WGSL validation failed: {e:?}"));
        }
    }

    /// §26 texture-seam drift guard: the atlas tile-safety fix (the
    /// "textures connect/bleed" bug) has THREE required ingredients in
    /// BOTH the terrain and water fragment shaders —
    /// 1. `textureSampleGrad` (explicit gradients: no implicit-derivative
    ///    LOD explosion at fract() discontinuities),
    /// 2. the half-texel inset `clamp(fract(...), 0.03125, 0.96875)`
    ///    (bilinear/mipmap/aniso footprints stay inside the tile),
    /// 3. gradients taken from the PRE-fract uv (`dpdx(in.uv)`, not of the
    ///    clamped/fract'ed coordinate).
    /// A refactor that drops any one of them resurrects the seam bug —
    /// this test fails loudly instead.
    #[test]
    fn terrain_water_seam_guards_present() {
        for (name, src) in [("terrain", TERRAIN_SHADER), ("water", WATER_SHADER)] {
            assert!(
                src.contains("textureSampleGrad(atlas_tex, atlas_samp, tuv, gdx, gdy)"),
                "{name}: explicit-gradient atlas sampling missing"
            );
            assert!(
                src.contains("clamp(fract("),
                "{name}: half-texel inset clamp missing"
            );
            assert!(
                src.contains("vec2<f32>(0.03125), vec2<f32>(0.96875)"),
                "{name}: inset bounds are not the half-texel pair (0.5/16, 1-0.5/16)"
            );
            assert!(
                src.contains("dpdx(in.uv) / vec2<f32>(16.0, 16.0)"),
                "{name}: gradients must come from the PRE-fract uv"
            );
            assert!(
                !src.contains("textureSample(atlas_tex"),
                "{name}: implicit-derivative atlas sampling is the seam bug — do not reintroduce"
            );
        }
    }

    /// FSR 1.0 EASU at EXACT 1:1 input/output scale must be the identity:
    /// the center tap weight is 1 and every neighbor windows to 0 (this is
    /// the property that lets the EASU pass run at native resolution). Runs
    /// the WGSL kernel math on the CPU by re-implementing the tap function
    /// — kept in lockstep with EASU_SHADER by the constants below.
    #[test]
    fn easu_is_identity_at_1x() {
        // kernel state for a flat neighborhood (dir=(1,0) after the zero
        // guard, len=0): len2=(1,1), lob=0.5, clp=1/0.5=2
        let dir = (1.0f32, 0.0f32);
        let len2 = (1.0f32, 1.0f32);
        let lob = 0.5f32;
        let clp = 2.0f32;
        let pp = (0.0f32, 0.0f32); // exact texel alignment at 1:1
        let tap = |off: (f32, f32)| -> f32 {
            let v = (
                (off.0 * dir.0 + off.1 * dir.1) * len2.0,
                (off.0 * -dir.1 + off.1 * dir.0) * len2.1,
            );
            let d2 = (v.0 * v.0 + v.1 * v.1).min(clp);
            let wb = (2.0 / 5.0 * d2 - 1.0).powi(2);
            let wa = (lob * d2 - 1.0).powi(2);
            (25.0 / 16.0 * wb - (25.0 / 16.0 - 1.0)) * wa
        };
        // center tap (0,0)-pp: weight exactly 1
        let center = tap((0.0, 0.0));
        assert!((center - 1.0).abs() < 1e-6, "center weight {center}");
        // every neighbor tap: 0 (windowed out)
        for off in [
            (0.0, -1.0),
            (1.0, -1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, 1.0),
            (0.0, 1.0),
            (1.0, 1.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 2.0),
            (0.0, 2.0),
        ] {
            let w = tap((off.0 - pp.0, off.1 - pp.1));
            assert!(w.abs() < 1e-6, "tap {off:?} weight {w} (must be 0)");
        }
    }
}
