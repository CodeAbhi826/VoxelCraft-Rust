// VoxelCraft SHADER-PACK-API v1 — demo pack "Warm Evening" (clean-room)
//
// Contract: struct PackU + fn packGrade(uv, scene, bloom, u) -> vec3.
// Runs AFTER the engine's FSR RCAS + grade; returns final linear RGB.
struct PackU {
    params: vec4<f32>,   // (warmth, _, _, _) — settings row 0
    viewport: vec4<f32>, // (w, h, 1/w, 1/h)
    time: vec4<f32>,     // (seconds, day 0..1, underwater, skylight)
};

fn packGrade(uv: vec2<f32>, scene: vec3<f32>, bloom: vec3<f32>, u: PackU) -> vec3<f32> {
    let warmth = clamp(u.params.x, 0.0, 2.0);
    let day = u.time.y;

    // golden hour ramps in as the day fades (day 0.5 → 1.0)
    let evening = smoothstep(0.5, 0.95, 1.0 - day);
    // night keeps a faint ember tint
    let night = smoothstep(0.05, 0.0, day) * 0.15;

    // warm lift on R/G, cool press on B — scaled by the pack slider
    let tint = vec3<f32>(0.10, 0.03, -0.06) * (evening + night) * warmth;
    var col = scene + tint;

    // soft shoulder roll-off (filmic-ish, keeps highlights from clipping)
    col = col / (col + vec3<f32>(0.55)) * 1.35;

    // gentle bloom fold-in (the engine already scaled the bloom buffer)
    col += bloom * 0.5;

    // slight radial falloff toward the corners (portrait-lens feel)
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    col *= 1.0 - 0.22 * smoothstep(0.45, 0.95, d);

    return clamp(col, vec3<f32>(0.0), vec3<f32>(1.0));
}
