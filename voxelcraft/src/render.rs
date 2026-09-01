//! wgpu renderer: Vulkan/DX12/Metal/WebGPU via a single code path.
//! Pipelines: sky (gradient + sun/moon/stars), terrain (alpha-test, smooth
//! light), water (blend + waves), selection wireframe, UI (bitmap canvas).

use crate::mesh::{MeshData, Vertex};
use crate::textures;
use crate::ui::{UiCanvas, UI_H, UI_W};
use crate::world::ChunkPos;
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;
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
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    p: [f32; 4],
    q: [f32; 4],
}

/// blur direction + texel step for the separable blur passes
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AuxUniform {
    dir: [f32; 4],
}

/// offscreen scene + bloom pyramid textures (recreated on resize)
struct PostTargets {
    scene: wgpu::Texture,
    scene_view: wgpu::TextureView,
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
    fn new(device: &wgpu::Device, w: u32, h: u32, _format: wgpu::TextureFormat) -> Self {
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
                size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let v = t.create_view(&wgpu::TextureViewDescriptor::default());
            (t, v)
        };
        let (scene, scene_view) = make(w, h);
        let (q, q_view) = make(w / 4, h / 4);
        let (b1, b1_view) = make(w / 8, h / 8);
        let (b2, b2_view) = make(w / 8, h / 8);
        PostTargets { scene, scene_view, q, q_view, b1, b1_view, b2, b2_view }
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
}

/// Post-processing request for a frame.
pub struct PostParams {
    /// 0 = off, 1 = vanilla+ (bloom/vig/sat), 2 = cinematic (+chroma, ACES)
    pub mode: u8,
    /// menu/panorama background blur 0..1
    pub menu_blur: f32,
}

#[derive(Default)]
pub struct RenderStats {
    pub chunks: u32,
    pub tris: u32,
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

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec2<f32>,
    @location(3) light: f32,
    @location(4) sky: f32,
};

@vertex
fn vs_main(
    @location(0) in_pos: vec3<f32>,
    @location(1) in_uv: vec2<f32>,
    @location(2) in_tile: vec2<f32>,
    @location(3) in_light: f32,
    @location(4) in_sky: f32,
) -> VsOut {
    var out: VsOut;
    out.pos = G.view_proj * vec4<f32>(in_pos, 1.0);
    out.world = in_pos;
    out.uv = in_uv;
    out.tile = in_tile;
    out.light = in_light;
    out.sky = in_sky;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tuv = (in.tile + fract(in.uv)) / vec2<f32>(16.0, 16.0);
    let c = textureSample(atlas_tex, atlas_samp, tuv);
    if (c.a < 0.5) { discard; }
    let day = G.misc.x;
    // G.misc.w = min-light floor (brightness setting, kills pitch-black caves)
    let sky_l = max(in.sky * day, G.misc.w);
    var rgb = c.rgb * in.light * sky_l;
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

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile: vec2<f32>,
    @location(3) light: f32,
    @location(4) sky: f32,
};

@vertex
fn vs_main(
    @location(0) in_pos: vec3<f32>,
    @location(1) in_uv: vec2<f32>,
    @location(2) in_tile: vec2<f32>,
    @location(3) in_light: f32,
    @location(4) in_sky: f32,
) -> VsOut {
    var out: VsOut;
    var p = in_pos;
    let is_top = abs(fract(p.y) - 0.875) < 0.01;
    let wob = sin(G.misc.y * 1.6 + p.x * 0.7 + p.z * 1.1) * 0.045
            + sin(G.misc.y * 1.1 + p.x * 1.9 - p.z * 0.6) * 0.025;
    p.y = p.y + select(0.0, wob, is_top);
    out.pos = G.view_proj * vec4<f32>(p, 1.0);
    out.world = p;
    out.uv = in_uv;
    out.tile = in_tile;
    out.light = in_light;
    out.sky = in_sky;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scroll = vec2<f32>(G.misc.y * 0.06, G.misc.y * 0.025);
    let tuv = (in.tile + fract(in.uv + scroll)) / vec2<f32>(16.0, 16.0);
    let c = textureSample(atlas_tex, atlas_samp, tuv);
    let day = G.misc.x;
    // G.misc.w = min-light floor (brightness setting)
    let sky_l = max(in.sky * day, G.misc.w);
    var rgb = c.rgb * in.light * sky_l * 1.05;
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
struct PostU { p: vec4<f32>, q: vec4<f32> };
// p = (mode, menu_blur, time, aspect)  q = (bloom, vignette, saturation, exposure)
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
    } else {
        col = textureSample(scene, samp, uv).rgb;
        // cinematic chromatic aberration
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

// ---------------------------------------------------------------- renderer

pub(crate) struct ChunkGpu {
    pub v: wgpu::Buffer,
    pub i: wgpu::Buffer,
    pub n: u32,
    pub w: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    atlas_tex: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    globals_buf: wgpu::Buffer,
    world_bg: wgpu::BindGroup,
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
    // post-processing
    post_targets: PostTargets,
    post_samp: wgpu::Sampler,
    post_buf: wgpu::Buffer,
    // blur directions: SEPARATE buffers per axis. A single buffer written
    // mid-encoder (between the h-blur and v-blur passes) does NOT work —
    // queue.write_buffer applies at the next submit, so both passes read the
    // last-written value (horizontal) and the vertical blur never runs →
    // bloom smears into vertical streaks (looked like sky "tearing").
    aux_h_buf: wgpu::Buffer,
    aux_v_buf: wgpu::Buffer,
    post_bgl: wgpu::BindGroupLayout,
    comp_bgl: wgpu::BindGroupLayout,
    bg_scene: wgpu::BindGroup,
    bg_q: wgpu::BindGroup,
    bg_b1: wgpu::BindGroup,
    bg_comp: wgpu::BindGroup,
    bright_pipe: wgpu::RenderPipeline,
    blur_pipe: wgpu::RenderPipeline,
    post_pipe: wgpu::RenderPipeline,
    pub chunks: HashMap<ChunkPos, ChunkGpu>,
    present_modes: Vec<wgpu::PresentMode>,
    pub vsync: bool,
    /// diagnostic counter: frames successfully submitted (logged first 3)
    submitted_frames: u32,
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

        // WebGL2 (downlevel) can't satisfy default limits (no compute);
        // retry with downlevel limits in that case.
        let (device, queue) = match adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("voxelcraft"),
                    required_features: wgpu::Features::empty(),
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
        let alpha_mode = if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
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

        let depth = Self::make_depth(&device, config.width, config.height);

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
            ],
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
            ],
        });

        let terrain_vbl = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 20, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 28, shader_location: 3, format: wgpu::VertexFormat::Float32 },
                wgpu::VertexAttribute { offset: 32, shader_location: 4, format: wgpu::VertexFormat::Float32 },
            ],
        };

        let depth_state = |write: bool, cmp: wgpu::CompareFunction| wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: write,
            depth_compare: cmp,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("world-pl"),
            bind_group_layouts: &[&world_bgl],
            push_constant_ranges: &[],
        });

        // scene pipelines target the LINEAR offscreen scene texture
        let scene_format = wgpu::TextureFormat::Rgba8Unorm;
        let make_pipe = |module: &wgpu::ShaderModule, entry: &str, cull: Option<wgpu::Face>, blend: Option<wgpu::BlendState>, depth: wgpu::DepthStencilState| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("pipe"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: entry,
                    compilation_options: Default::default(),
                    buffers: &[terrain_vbl.clone()],
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
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        // Blending for translucent surfaces that keeps the canvas OPAQUE:
        // the alpha channel uses One/One so dst.a stays 1. With plain
        // ALPHA_BLENDING, water/cloud pixels end with a < 1 and the browser
        // composites the page background through them (see-through water /
        // flicker on WebGL2 and premultiplied WebGPU surfaces).
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
        let terrain_pipe = make_pipe(&terrain_mod, "vs_main", Some(wgpu::Face::Back), None, depth_state(true, wgpu::CompareFunction::Less));
        let water_pipe = make_pipe(&water_mod, "vs_main", None, opaque_blend, depth_state(false, wgpu::CompareFunction::Less));
        let sky_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky-pipe"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_mod,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_mod,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: scene_format,
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
            depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Always)),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
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
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
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
        let line_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lines"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(LINE_SHADER)),
        });
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
        let line_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line-pl"),
            bind_group_layouts: &[&line_bgl],
            push_constant_ranges: &[],
        });
        let line_vbl = wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 }],
        };
        let line_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line-pipe"),
            layout: Some(&line_pl),
            vertex: wgpu::VertexState {
                module: &line_mod,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[line_vbl],
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // unit cube edges (slightly inflated to avoid z-fighting)
        let e = 0.002;
        let lo = -e;
        let hi = 1.0 + e;
        let c = [
            [lo, lo, lo], [hi, lo, lo], [hi, lo, hi], [lo, lo, hi],
            [lo, hi, lo], [hi, hi, lo], [hi, hi, hi], [lo, hi, hi],
        ];
        let edges: Vec<[f32; 3]> = vec![
            c[0], c[1], c[1], c[2], c[2], c[3], c[3], c[0],
            c[4], c[5], c[5], c[6], c[6], c[7], c[7], c[4],
            c[0], c[4], c[1], c[5], c[2], c[6], c[3], c[7],
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
        let cloud_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("clouds"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(CLOUD_SHADER)),
        });
        let cloud_vbl = wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        };
        let cloud_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cloud-pl"),
            bind_group_layouts: &[&cloud_bgl],
            push_constant_ranges: &[],
        });
        let cloud_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cloud-pipe"),
            layout: Some(&cloud_pl),
            vertex: wgpu::VertexState {
                module: &cloud_mod,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[cloud_vbl],
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
            depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Less)),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        // cloud quad: big XZ plane centered on the camera (recentered in vs)
        let s = 2400.0f32;
        let cloud_verts: [[f32; 2]; 6] = [
            [-s, -s], [s, -s], [s, s],
            [-s, -s], [s, s], [-s, s],
        ];
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
        // write once here; rewritten on resize (texel steps are size-dependent)
        let bw = (config.width / 8).max(1);
        let bh = (config.height / 8).max(1);
        queue.write_buffer(
            &aux_h_buf,
            0,
            bytemuck::bytes_of(&AuxUniform { dir: [1.0 / bw as f32, 0.0, 0.0, 0.0] }),
        );
        queue.write_buffer(
            &aux_v_buf,
            0,
            bytemuck::bytes_of(&AuxUniform { dir: [0.0, 1.0 / bh as f32, 0.0, 0.0] }),
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
        let make_fs_pipe = |module: &wgpu::ShaderModule, layout: &wgpu::PipelineLayout, out_format: wgpu::TextureFormat| {
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
        // composite writes the final srgb-encoded image to the surface
        let post_pipe = make_fs_pipe(&post_mod, &comp_pl, format);

        // offscreen targets + bind groups at the real size
        // (blur-h reads q with the H-step aux; blur-v reads b1 with the V-step)
        let post_targets = PostTargets::new(&device, config.width, config.height, format);
        let bg_scene = Self::single_tex_bg(&device, &post_bgl, &post_targets.scene_view, &post_samp, &post_buf, &aux_v_buf);
        let bg_q = Self::single_tex_bg(&device, &post_bgl, &post_targets.q_view, &post_samp, &post_buf, &aux_h_buf);
        let bg_b1 = Self::single_tex_bg(&device, &post_bgl, &post_targets.b1_view, &post_samp, &post_buf, &aux_v_buf);
        let bg_comp = Self::comp_bg(&device, &comp_bgl, &post_targets.scene_view, &post_targets.b2_view, &post_samp, &post_buf);

        let renderer = Renderer {
            surface,
            device,
            queue,
            config,
            depth,
            atlas_tex,
            atlas_view,
            sampler,
            globals_buf,
            world_bg,
            terrain_pipe,
            water_pipe,
            sky_pipe,
            cloud_bg,
            cloud_pipe,
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
            line_pipe,
            line_bg,
            post_targets,
            post_samp,
            post_buf,
            aux_h_buf,
            aux_v_buf,
            post_bgl,
            comp_bgl,
            bg_scene,
            bg_q,
            bg_b1,
            bg_comp,
            bright_pipe,
            blur_pipe,
            post_pipe,
            chunks: HashMap::new(),
            present_modes,
            vsync,
            submitted_frames: 0,
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
        let t = PostTargets::new(&self.device, w, h, format);
        let bg_scene = Self::single_tex_bg(&self.device, &self.post_bgl, &t.scene_view, &self.post_samp, &self.post_buf, &self.aux_v_buf);
        let bg_q = Self::single_tex_bg(&self.device, &self.post_bgl, &t.q_view, &self.post_samp, &self.post_buf, &self.aux_h_buf);
        let bg_b1 = Self::single_tex_bg(&self.device, &self.post_bgl, &t.b1_view, &self.post_samp, &self.post_buf, &self.aux_v_buf);
        let bg_comp = Self::comp_bg(&self.device, &self.comp_bgl, &t.scene_view, &t.b2_view, &self.post_samp, &self.post_buf);
        self.post_targets = t;
        self.bg_scene = bg_scene;
        self.bg_q = bg_q;
        self.bg_b1 = bg_b1;
        self.bg_comp = bg_comp;
        // refresh blur texel steps for the new size (1/8 targets)
        let bw = (w / 8).max(1);
        let bh = (h / 8).max(1);
        self.queue.write_buffer(
            &self.aux_h_buf,
            0,
            bytemuck::bytes_of(&AuxUniform { dir: [1.0 / bw as f32, 0.0, 0.0, 0.0] }),
        );
        self.queue.write_buffer(
            &self.aux_v_buf,
            0,
            bytemuck::bytes_of(&AuxUniform { dir: [0.0, 1.0 / bh as f32, 0.0, 0.0] }),
        );
    }

    fn make_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
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
        self.depth = Self::make_depth(&self.device, w, h);
        self.rebuild_post_targets();
    }

    /// current surface size in physical px (UI coordinate mapping)
    pub fn size(&self) -> (f32, f32) {
        (self.config.width as f32, self.config.height as f32)
    }

    /// drop all GPU chunk meshes (full re-mesh, e.g. smooth-lighting toggle)
    pub fn clear_meshes(&mut self) {
        self.chunks.clear();
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

    pub fn set_chunk_mesh(&mut self, pos: ChunkPos, md: &MeshData) {
        let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST;
        let make = |data: (&Vec<Vertex>, &Vec<u32>)| -> Option<(wgpu::Buffer, wgpu::Buffer, u32)> {
            if data.1.is_empty() {
                return None;
            }
            let v = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (data.0.len() * std::mem::size_of::<Vertex>()) as u64,
                usage,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&v, 0, bytemuck::cast_slice(data.0));
            let i = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (data.1.len() * 4) as u64,
                usage,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&i, 0, bytemuck::cast_slice(data.1));
            Some((v, i, data.1.len() as u32))
        };
        let solid = make((&md.solid.0, &md.solid.1));
        let water = make((&md.water.0, &md.water.1));
        let (v, i, n) = solid.unwrap_or_else(|| (self.empty_buf(), self.empty_buf(), 0));
        self.chunks.insert(pos, ChunkGpu { v, i, n, w: water });
    }

    fn empty_buf(&self) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        })
    }

    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
    }

    pub fn has_chunk(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
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
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

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
            fog_color: [sky.fog_color[0], sky.fog_color[1], sky.fog_color[2], sky.fog_start],
            sun_dir: [sky.sun_dir.x, sky.sun_dir.y, sky.sun_dir.z, sky.fog_end],
            misc: [
                sky.day_light,
                sky.time,
                if sky.underwater { 1.0 } else { 0.0 },
                sky.min_light,
            ],
        };
        self.queue.write_buffer(&self.globals_buf, 0, bytemuck::bytes_of(&globals));

        // UI uniform: letterboxed mapping canvas → screen
        let sw = self.config.width as f32;
        let sh = self.config.height as f32;
        let scale = (sw / UI_W as f32).min(sh / UI_H as f32);
        let x0 = (sw - UI_W as f32 * scale) * 0.5;
        let y0 = (sh - UI_H as f32 * scale) * 0.5;
        let ui_map = UiUniform {
            map: [2.0 * scale / sw, 2.0 * x0 / sw - 1.0, -2.0 * scale / sh, 1.0 - 2.0 * y0 / sh],
        };
        self.queue.write_buffer(&self.ui_buf, 0, bytemuck::bytes_of(&ui_map));

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
        let (bloom, vig, sat, exp) = match post.mode {
            1 => (0.55, 0.14, 1.07, 1.0),   // vanilla+
            2 => (0.85, 0.32, 1.16, 1.06),  // cinematic
            _ => (0.0, 0.0, 1.0, 1.0),      // off
        };
        let post_u = PostUniform {
            p: [post.mode as f32, post.menu_blur.clamp(0.0, 1.0), sky.time, aspect],
            q: [bloom, vig, sat, exp],
        };
        self.queue.write_buffer(&self.post_buf, 0, bytemuck::bytes_of(&post_u));

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
        self.queue.write_buffer(&self.line_buf, 0, bytemuck::bytes_of(&line_u));

        // NOTE: blur direction uniforms (aux_h_buf / aux_v_buf) are written
        // once at init and refreshed in rebuild_post_targets() — per-frame
        // mid-encoder writes cannot take effect between passes (see struct
        // field comment).

        // frustum planes from vp (rows)
        let rows: [Vec4; 4] = [vp.row(0), vp.row(1), vp.row(2), vp.row(3)];
        let combine = |a: &Vec4, b: &Vec4| [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
        let planes: [[f32; 4]; 6] = [
            combine(&rows[3], &rows[0]), // left
            combine(&rows[3], &-rows[0]), // right
            combine(&rows[3], &rows[1]), // bottom
            combine(&rows[3], &-rows[1]), // top
            combine(&rows[3], &rows[2]), // near
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        // ─────────────────────────────────────────────── pass 1: scene ──
        // (sky + terrain + selection + water + clouds → offscreen LINEAR
        // scene texture — the composite encodes to srgb once at the end)
        let clear = wgpu::RenderPassColorAttachment {
            view: &self.post_targets.scene_view,
            resolve_target: None,
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
        let depth_att = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(clear)],
                depth_stencil_attachment: Some(depth_att),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. sky
            pass.set_pipeline(&self.sky_pipe);
            pass.set_bind_group(0, &self.world_bg, &[]);
            pass.draw(0..3, 0..1);

            // 2. terrain (near → far for early-z)
            let mut sorted = visible.clone();
            sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            pass.set_pipeline(&self.terrain_pipe);
            pass.set_bind_group(0, &self.world_bg, &[]);
            for (pos, _) in sorted.iter() {
                let g = self.chunks.get(pos).unwrap();
                if g.n == 0 {
                    continue;
                }
                pass.set_vertex_buffer(0, g.v.slice(..));
                pass.set_index_buffer(g.i.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..g.n, 0, 0..1);
                stats.chunks += 1;
                stats.tris += g.n / 3;
            }

            // 3. selection wireframe
            if selection.is_some() {
                pass.set_pipeline(&self.line_pipe);
                pass.set_bind_group(0, &self.line_bg, &[]);
                pass.set_vertex_buffer(0, self.line_vb.slice(..));
                pass.draw(0..24, 0..1);
            }

            // 4. water (far → near, blended)
            pass.set_pipeline(&self.water_pipe);
            pass.set_bind_group(0, &self.world_bg, &[]);
            let mut far_sorted = sorted;
            far_sorted.reverse();
            for (pos, _) in far_sorted.iter() {
                let Some(g) = self.chunks.get(pos) else { continue };
                let Some((v, i, n)) = &g.w else { continue };
                pass.set_vertex_buffer(0, v.slice(..));
                pass.set_index_buffer(i.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..*n, 0, 0..1);
            }

            // 5. clouds (translucent plane above the world)
            if clouds {
                pass.set_pipeline(&self.cloud_pipe);
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

        // ───────────────────────────────── pass 4: composite → surface ──
        {
            let att = wgpu::RenderPassColorAttachment {
                view: &frame_view,
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
            pass.set_pipeline(&self.post_pipe);
            pass.set_bind_group(0, &self.bg_comp, &[]);
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
                self.submitted_frames, self.config.format, self.config.alpha_mode, self.config.width, self.config.height,
                post.mode, post.menu_blur
            ));
        }

        stats
    }
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
    let Some(window) = web_sys::window() else { return false };
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
    let Ok(func) = request_adapter.dyn_into::<js_sys::Function>() else { return false };

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
    let Ok(promise) = result.dyn_into::<js_sys::Promise>() else { return false };
    let Ok(value) = wasm_bindgen_futures::JsFuture::from(promise).await else {
        return false;
    };
    !value.is_null() && !value.is_undefined()
}

/// Surface a fatal init error on the page (wasm) / stderr (native),
/// instead of silently panicking into a blank screen.
#[allow(dead_code)]
pub(crate) fn report_boot_error(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::wasm_entry::boot_error(msg);
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[voxelcraft] {msg}");
}

/// Best-effort diagnostic log (wasm: JS console, native: stderr).
#[allow(dead_code)]
pub(crate) fn report_boot_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::wasm_entry::boot_log(msg);
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[voxelcraft] {msg}");
}
