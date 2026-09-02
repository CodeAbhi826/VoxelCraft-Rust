// VoxelCraft SHADER-PACK-API v1 — demo pack "Moonlit" (clean-room)
//
// Contract: struct PackU + fn packGrade(uv, scene, bloom, u) -> vec3.
// Demonstrates TIME-VARYING effects: the grain animates via time.x
// (frameTimeCounter alias) — proving the uniform bridge flows per frame.
struct PackU {
    params: vec4<f32>,   // (moonlight, _, _, _) — settings row 0
    viewport: vec4<f32>, // (w, h, 1/w, 1/h)
    time: vec4<f32>,     // (seconds, day 0..1, underwater, skylight)
};

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + vec3<f32>(33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn packGrade(uv: vec2<f32>, scene: vec3<f32>, bloom: vec3<f32>, u: PackU) -> vec3<f32> {
    let moon = clamp(u.params.x, 0.0, 2.0);
    let day = u.time.y;
    // full effect at night, fades to a whisper at noon
    let night = smoothstep(0.35, 0.0, day);

    // luma-first desaturation toward moon-blue
    let lum = dot(scene, vec3<f32>(0.2126, 0.7152, 0.0722));
    var col = mix(scene, vec3<f32>(lum), 0.55 * night * moon);
    let blue = vec3<f32>(-0.06, -0.02, 0.12) * night * moon;
    col += blue * lum;

    // stars-of-noise: animated monochrome grain (time.x = seconds).
    // viewport.xy = surface pixels; hash runs per output pixel.
    let px = uv * u.viewport.xy;
    let g = hash12(px + vec2<f32>(u.time.x * 61.7, u.time.x * -43.3)) - 0.5;
    col += g * 0.045 * night * moon;

    // moon bloom rides high-luma pixels only (cheap luma gate)
    let gate = smoothstep(0.55, 0.9, lum);
    col += bloom * gate * 0.8 * moon;

    return clamp(col, vec3<f32>(0.0), vec3<f32>(1.0));
}
