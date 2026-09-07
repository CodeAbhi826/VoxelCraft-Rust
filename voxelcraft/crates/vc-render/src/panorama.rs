//! Title-screen panorama — a pre-rendered, slowly panning cubemap.
//!
//! VERIFIED 2026-09-07 live (minecraft.wiki/w/Panorama): the real title
//! screen background is NOT the live world — it is six square pre-rendered
//! images (`panorama_0..5.png`, four horizontal faces + up + down) shown as
//! a cubemap with a slowly rotating camera and a blur overlay, displayed
//! behind every menu screen that does not cover the whole background. No
//! chunks are generated, meshed or streamed while the menus are up; the
//! world only generates when the player enters it.
//!
//! This module reproduces that architecture clean-room:
//! - `paint_cubemap()` renders the six faces ONCE (at renderer init) from a
//!   deterministic procedural scene — Nether-Update theme to match 1.16.x
//!   (crimson fog sky, lava-glow horizon, netherrack ground, a crimson
//!   canopy tree belt and a glowing lava lake sector; no clouds — the
//!   Nether has none). It is "pre-rendered" in the exact sense that
//!   matters: the menus never touch the world/meshing pipeline, so a slow
//!   GPU mesher or heavy world gen can never stall the title screen again
//!   (the user-reported minute-long "loading" was exactly that coupling).
//! - `PanoResources` owns the cube texture + a fullscreen pass that ray
//!   casts per pixel from a (yaw, pitch, fov) camera and samples the cube.
//!   The existing post chain then applies the menu blur on top of it.
//!
//! Faces follow the standard cube conventions (layer order +X, −X, +Y, −Y,
//! +Z, −Z; u right, v down, top-left origin) so `textureSample` with a
//! world-space direction reconstructs the painted view without seams.

use bytemuck::{Pod, Zeroable};

/// Per-frame panorama camera uniform: (yaw, pitch, tan(fov/2), aspect).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct PanoUniform {
    pub p: [f32; 4],
}

/// The menu background view for one frame.
#[derive(Clone, Copy, Debug)]
pub struct PanoView {
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

/// The panorama pass WGSL (naga-validated by render.rs's shader test).
pub const PANO_SHADER: &str = r#"
struct PanoU { p: vec4<f32> };
@group(0) @binding(0) var<uniform> u: PanoU;
@group(0) @binding(1) var tex: texture_cube<f32>;
@group(0) @binding(2) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4f,
    // NDC-space coords (y up): @builtin(position) in the FRAGMENT stage is
    // framebuffer pixels, not NDC — so the ray basis interpolates this
    // varying instead of deriving from pos
    @location(0) ndc: vec2f,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VOut {
    // fullscreen triangle: (-1,-1) (3,-1) (-1,3)
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f( 3.0, -1.0),
        vec2f(-1.0,  3.0),
    );
    var o: VOut;
    o.pos = vec4f(pos[vi], 0.0, 1.0);
    o.ndc = pos[vi];
    return o;
}

@fragment
fn fs(v: VOut) -> @location(0) vec4f {
    // NDC → camera ray through the yaw/pitch camera, then cubemap
    // lookup. Same basis as the engine's Camera (yaw=0 faces −Z, +yaw
    // turns right; pitch +up).
    let ndc = v.ndc;
    let aspect = u.p.w;
    let tanf = u.p.z;
    let yaw = u.p.x;
    let pitch = u.p.y;
    let cy = cos(yaw); let sy = sin(yaw);
    let cp = cos(pitch); let sp = sin(pitch);
    let fwd = vec3f(sy * cp, sp, -cy * cp);
    let right = vec3f(cy, 0.0, sy);
    let up = cross(right, fwd);
    let dir = normalize(fwd + right * (ndc.x * aspect * tanf) + up * (ndc.y * tanf));
    let rgb = textureSample(tex, samp, dir).rgb;
    return vec4f(rgb, 1.0);
}
"#;

/// GPU resources for the panorama pass. Created once at renderer init; the
/// uniform is refreshed per frame from the render() panorama path.
pub struct PanoResources {
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    _tex: wgpu::Texture,
}

impl PanoResources {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const SIZE: u32 = 384;
        let data = paint_cubemap(SIZE);

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("panorama-cubemap"),
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        for face in 0..6u32 {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: face },
                    aspect: wgpu::TextureAspect::All,
                },
                &data[face as usize * (SIZE as usize * SIZE as usize * 4)
                    ..(face as usize + 1) * (SIZE as usize * SIZE as usize * 4)],
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(SIZE * 4),
                    rows_per_image: Some(SIZE),
                },
                wgpu::Extent3d {
                    width: SIZE,
                    height: SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }
        // 6 square array layers viewed as a cube — hardware face selection
        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("panorama-cube-view"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(6),
        });

        let samp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("panorama-samp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pano-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                        view_dimension: wgpu::TextureViewDimension::Cube,
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

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pano-uniform"),
            size: std::mem::size_of::<PanoUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pano-bg"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&samp),
                },
            ],
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("panorama"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PANO_SHADER)),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pano-pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pano-pipe"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    // renders into the linear offscreen scene texture; the
                    // post chain sRGB-encodes once at the end
                    format: wgpu::TextureFormat::Rgba8Unorm,
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
        });

        Self {
            uniform_buf,
            bind_group,
            pipeline,
            _tex: tex,
        }
    }
}

// ---------------------------------------------------------------- painter --

/// sRGB-ish palette entry → linear (the scene texture is linear; post
/// re-encodes to sRGB once at the composite).
fn lin(c: [f32; 3]) -> [f32; 3] {
    [c[0].powf(2.2), c[1].powf(2.2), c[2].powf(2.2)]
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// integer lattice hash → [0,1)
fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x.wrapping_mul(374761393))
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(z.wrapping_mul(1274126177));
    h = (h ^ (h >> 13)).wrapping_mul(1103515245);
    h ^= h >> 16;
    ((h as u32) & 0x00ff_ffff) as f32 / 0x00ff_ffff as f32
}

/// 3D value noise (seam-free on the sphere — sampled at raw directions)
fn vnoise3(x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let zf = z - zi as f32;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let c00 = lerp(hash3(xi, yi, zi), hash3(xi + 1, yi, zi), u);
    let c10 = lerp(hash3(xi, yi + 1, zi), hash3(xi + 1, yi + 1, zi), u);
    let c01 = lerp(hash3(xi, yi, zi + 1), hash3(xi + 1, yi, zi + 1), u);
    let c11 = lerp(
        hash3(xi, yi + 1, zi + 1),
        hash3(xi + 1, yi + 1, zi + 1),
        u,
    );
    lerp(lerp(c00, c10, v), lerp(c01, c11, v), w)
}

/// 4-octave fbm of the 3D value noise
fn fbm3(x: f32, y: f32, z: f32) -> f32 {
    let mut a = 0.5f32;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    let (mut fx, mut fy, mut fz) = (x, y, z);
    for _ in 0..4 {
        sum += a * vnoise3(fx, fy, fz);
        norm += a;
        a *= 0.5;
        fx *= 2.03;
        fy *= 1.97;
        fz *= 2.01;
    }
    sum / norm
}

/// periodic 2D fbm on the unit circle (azimuth-continuous, no seams across
/// the four horizontal faces)
fn fbm_circle(az: f32, scale: f32) -> f32 {
    fbm3(az.cos() * scale, az.sin() * scale, 0.5)
}

/// The sun direction (also drives the warm halo band).
const SUN: [f32; 3] = [0.30, 0.52, -0.80];

/// Paint the six cubemap faces (layer order +X, −X, +Y, −Y, +Z, −Z; u right,
/// v down, top-left origin — matches hardware cube face selection).
pub fn paint_cubemap(size: u32) -> Vec<u8> {
    let s = size as usize;
    let mut out = vec![0u8; 6 * s * s * 4];

    // palette (sRGB, converted to linear below) — NETHER-UPDATE THEME:
    // the 1.16.x title panorama reflects the Nether Update (VERIFIED
    // minecraft.wiki/w/Panorama history: "1.16 ... Changed panorama in all
    // released 1.16 snapshots to reflect the Nether Update"). Clean-room
    // approximation of that look: crimson fog sky, lava-glow horizon,
    // netherrack ground, a dark tree belt with crimson canopies (crimson
    // forest), and a glowing lava lake sector. No sampled assets.
    let zenith = lin([0.30, 0.06, 0.07]); // dark maroon void above
    let horizon_sky = lin([0.56, 0.14, 0.11]); // crimson fog at eye level
    let warm = lin([0.82, 0.34, 0.10]); // lava-glow band on the horizon
    let sun_disc = lin([1.0, 0.60, 0.22]); // distant lava-sea glow
    let sun_halo = lin([0.90, 0.40, 0.13]);
    let cloud_lit = lin([0.97, 0.98, 1.0]); // unused — the Nether has no
    let cloud_shade = lin([0.66, 0.71, 0.80]); // clouds (gated off below)
    let hill_far = lin([0.34, 0.10, 0.10]); // hazy crimson far ridge
    let hill_near = lin([0.26, 0.07, 0.08]); // netherrack near ridge
    let grass = [
        lin([0.40, 0.12, 0.12]), // netherrack light
        lin([0.33, 0.09, 0.10]), // netherrack dark
        lin([0.30, 0.06, 0.10]), // deep shade toward the nadir
    ];
    let tree_lit = lin([0.68, 0.10, 0.13]); // crimson canopy, lit
    let tree_dark = lin([0.52, 0.08, 0.11]); // crimson canopy, shade
    let trunk = lin([0.16, 0.09, 0.08]); // dark nether trunk
    let water = lin([0.88, 0.36, 0.07]); // glowing lava lake
    let sparkle = lin([1.0, 0.76, 0.30]); // bright lava crust glints

    let sun_len = (SUN[0] * SUN[0] + SUN[1] * SUN[1] + SUN[2] * SUN[2]).sqrt();

    for face in 0..6usize {
        for py in 0..s {
            for px in 0..s {
                // pixel → face uv → direction (inverse of cube face sel)
                let u = (px as f32 + 0.5) / s as f32 * 2.0 - 1.0;
                let v = (py as f32 + 0.5) / s as f32 * 2.0 - 1.0;
                let d = match face {
                    0 => [1.0, -v, -u],
                    1 => [-1.0, -v, u],
                    2 => [u, 1.0, v],
                    3 => [u, -1.0, -v],
                    4 => [u, -v, 1.0],
                    _ => [-u, -v, -1.0],
                };
                let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let d = [d[0] / dl, d[1] / dl, d[2] / dl];
                let az = d[0].atan2(-d[2]);
                let elev = d[1];

                // ---- sky gradient (t on elevation, warm band at horizon)
                let sky_t = smoothstep(-0.02, 0.85, elev);
                let mut col = mix(horizon_sky, zenith, sky_t);
                let warm_t = smoothstep(0.10, 0.0, elev.abs()) * 0.35;
                col = mix(col, warm, warm_t);

                // ---- sun disc + halo (reads as a distant lava glow in
                // the Nether theme — dimmer than the overworld sun)
                let dot = (d[0] * SUN[0] + d[1] * SUN[1] + d[2] * SUN[2]) / sun_len;
                if dot > 0.0 {
                    let halo = dot.powf(220.0) * 0.4;
                    col = mix(col, sun_halo, halo);
                    if dot > 0.999_35 {
                        col = sun_disc;
                    }
                }

                // ---- clouds: none in the Nether theme (the overworld
                // painter's fbm clouds are gated off; the palette entries
                // stay so the field remains documented)
                let cover = 0.0f32;
                if cover > 0.001 && elev > -0.02 {
                    let shade = smoothstep(0.50, 0.80, fbm3(d[0] * 4.0, d[1] * 4.0, d[2] * 4.0));
                    let cloud_col = mix(cloud_shade, cloud_lit, 0.35 + 0.65 * shade);
                    col = mix(col, cloud_col, cover * 0.9);
                }

                // ---- terrain: two ridge silhouettes on the azimuth (noise
                // on the unit circle → wraps seamlessly), tree belt cells,
                // a lake sector near the horizon
                let far_h = 0.016 + 0.050 * fbm_circle(az, 2.1);
                let near_h = 0.008 + 0.030 * fbm_circle(az + 2.0, 3.3);
                let below = -elev;
                if below > far_h {
                    if below > near_h {
                        // near hills → ground plane
                        let depth = smoothstep(0.0, 0.55, below - near_h);
                        let patch = fbm3(d[0] * 6.0, d[1] * 6.0 + 3.0, d[2] * 6.0);
                        let g = if patch < 0.42 { 0 } else { 1 };
                        let gcol = mix(grass[g], grass[2], depth * 0.7);
                        let mut col2 = gcol;
                        // lake sector: where the lake noise is high, the
                        // ground near the horizon is a glowing lava lake
                        // (crust glints stay quantized-blocky)
                        let lake = fbm_circle(az + 4.0, 1.6);
                        if lake > 0.52 && below < near_h + 0.055 {
                            let sp = vnoise3(d[0] * 40.0, d[1] * 40.0, d[2] * 40.0);
                            let sp2 = smoothstep(0.72, 0.95, sp) * 0.6;
                            col2 = mix(water, sparkle, sp2);
                        }
                        // ---- voxel tree belt standing on the near ridge
                        // (crimson-forest stand: dense, tall — the canopies
                        // must read as trees, not a hedge line)
                        let cells = 132.0;
                        let cell = ((az + std::f32::consts::PI)
                            / (std::f32::consts::TAU)
                            * cells)
                            .floor() as i32;
                        let h = hash3(cell, 77, 0);
                        if h < 0.78 {
                            let local = ((az + std::f32::consts::PI)
                                / std::f32::consts::TAU
                                * cells)
                                .fract();
                            let hgt = 0.04 + 0.14 * hash3(cell, 78, 0);
                            // canopy profile, quantized to two steps (blocky)
                            let prof =
                                (std::f32::consts::PI * local).sin().clamp(0.0, 1.0);
                            let prof = if prof > 0.55 { 1.0 } else { 0.6 };
                            let tree_top = near_h + hgt * prof;
                            let tree_base = near_h - 0.012;
                            if below > tree_base && below < tree_top {
                                let is_trunk =
                                    below < tree_base + 0.006 && prof == 1.0;
                                let shade = hash3(cell, 79, 0);
                                let tcol = if shade < 0.5 { tree_lit } else { tree_dark };
                                col2 = if is_trunk { trunk } else { tcol };
                            }
                        }
                        col = col2;
                    } else {
                        // far ridge: hazy blue-green band
                        let band = (below - far_h) / 0.012f32.max(1e-4);
                        col = mix(hill_far, hill_near, band.clamp(0.0, 1.0));
                    }
                }

                // ---- subtle horizon fog blend + dither (kills banding)
                let fog_t = smoothstep(0.10, 0.0, elev.abs()) * 0.25;
                col = mix(col, horizon_sky, fog_t);
                let dith = (hash3(px as i32, py as i32, face as i32) - 0.5) * 0.004;
                col = [
                    (col[0] + dith).clamp(0.0, 1.0),
                    (col[1] + dith).clamp(0.0, 1.0),
                    (col[2] + dith).clamp(0.0, 1.0),
                ];

                let base = (face * s * s + py * s + px) * 4;
                out[base] = (col[0] * 255.0).round() as u8;
                out[base + 1] = (col[1] * 255.0).round() as u8;
                out[base + 2] = (col[2] * 255.0).round() as u8;
                out[base + 3] = 255;
            }
        }
    }
    out
}

// ------------------------------------------------------------------ tests --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panorama_painter_is_deterministic_and_oriented() {
        let a = paint_cubemap(64);
        let b = paint_cubemap(64);
        assert_eq!(a, b, "painter must be deterministic (no RNG)");
        assert_eq!(a.len(), 6 * 64 * 64 * 4, "six 64px faces, RGBA");

        let px = |face: usize, x: usize, y: usize| -> [u8; 3] {
            let i = (face * 64 * 64 + y * 64 + x) * 4;
            [a[i], a[i + 1], a[i + 2]]
        };
        // side faces (+X, +Z): top rows are sky (red-dominant — the
        // Nether theme's crimson fog), bottom rows are ground (dark
        // netherrack: red dominant over green) — the cube orientation
        // contract the ray-cast shader's face selection relies on
        for face in [0usize, 4] {
            let sky = px(face, 32, 2);
            let ground = px(face, 32, 61);
            assert!(sky[0] > sky[2], "sky must be red-dominant, got {sky:?}");
            assert!(
                ground[0] > ground[1] && ground[0] > ground[2],
                "ground must be netherrack red-dominant, got {ground:?}"
            );
        }
        // up face center is sky-ish; down face center is ground-ish
        let up = px(2, 32, 32);
        assert!(up[0] > up[2], "up face must be crimson sky, got {up:?}");
        let down = px(3, 32, 32);
        assert!(down[0] > down[1], "down face must be netherrack, got {down:?}");

        // optional visual dump for inspection (never set in CI):
        //   PANORAMA_DUMP=/tmp/pano cargo test -p vc-render panorama
        // (the dump applies linear->sRGB so it matches what the in-game
        // sRGB surface shows — the raw face data is linear-space)
        if let Ok(dir) = std::env::var("PANORAMA_DUMP") {
            let data = paint_cubemap(384);
            let srgb = |v: f32| {
                if v <= 0.003_130_8 {
                    v * 12.92
                } else {
                    1.055 * v.powf(1.0 / 2.4) - 0.055
                }
            };
            for face in 0..6usize {
                let mut slice = data[face * 384 * 384 * 4..(face + 1) * 384 * 384 * 4].to_vec();
                for px in slice.chunks_exact_mut(4) {
                    for c in px.iter_mut().take(3) {
                        *c = (srgb(*c as f32 / 255.0) * 255.0).round() as u8;
                    }
                }
                let img = image::RgbaImage::from_raw(384, 384, slice)
                    .expect("face buffer size");
                let _ = img.save(format!("{dir}/pano_{face}.png"));
            }
        }
    }
}
