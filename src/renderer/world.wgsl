// World shader: texturing + per-vertex AO + fog + alpha test.

struct WorldUniforms {
    mvp: mat4x4<f32>,
    cam_pos: vec3<f32>,
    fog_color: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
};

@group(0) @binding(0) var<uniform> u: WorldUniforms;
@group(1) @binding(0) var tex_array: texture_2d_array<f32>;
@group(1) @binding(1) var tex_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tex_layer: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tex_layer: u32,
    @location(2) color: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u.mvp * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.tex_layer = in.tex_layer;
    out.color = in.color;
    out.world_pos = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex_array, tex_sampler, in.uv, i32(in.tex_layer));

    // Alpha test: discard nearly-transparent pixels (leaves, glass edges)
    if (tex_color.a < 0.5) {
        discard;
    }

    var col = tex_color;
    col = vec4<f32>(col.rgb * in.color, col.a);

    // Fog
    let dist = length(in.world_pos - u.cam_pos);
    let fog_factor = clamp((dist - u.fog_start) / (u.fog_end - u.fog_start), 0.0, 1.0);
    col = vec4<f32>(mix(col.rgb, u.fog_color, fog_factor), col.a);

    return col;
}
