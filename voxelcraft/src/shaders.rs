//! Phase 11 (Master Spec §34/§48) — native shader-pack API, tier
//! `SHADER-PACK-API`.
//!
//! §34.2 is explicit: a generic WGSL shader loader is NOT Iris
//! compatibility, and compatibility tiers must be labeled honestly. This
//! module is the **native VoxelCraft shader framework** (§34.1): packs
//! declare a grade preset and/or a custom composite stage in WGSL, get
//! naga-validated before any pipeline exists, and are cached per pack.
//!
//! Compatibility labels used (never claim more than is demonstrated):
//! * `NATIVE-SHADER` — the engine's built-in WGSL (FSR/EASU/RCAS, terrain…)
//! * `SHADER-PACK-API` — THIS: packs written against the documented
//!   VoxelCraft pack contract (see `PACK_CONTRACT`). Two clean-room demo
//!   packs ship with the engine and are explicitly tested (§48 gate).
//! * `OPTIFINE-COMPATIBLE-SUBSET` … `FULL-COMPATIBILITY` — NOT claimed.
//!   The uniform bridge exposes OptiFine-*style* alias names (the
//!   documented transformation subset) but loading an actual
//!   OptiFine/Iris pack is a separate, future project (§34.2's own rule).
//!
//! §34.1 coverage: configurable passes (composite stage + grade),
//! WGSL shaders (pack stage, naga-validated), material definitions (grade
//! presets), post-processing (rides the existing chain), custom effects
//! (packGrade body), recompilation/caching (pipeline cache keyed by pack
//! id + source hash; packs reload through `set_shader_pack`).

use serde::Deserialize;

/// Uniforms the engine bridges into every pack stage (§34.2
/// "transformation layer" — the documented subset, with the OptiFine-style
/// alias names packs can expect):
///
/// | field          | engine source          | OptiFine-style alias        |
/// |----------------|------------------------|-----------------------------|
/// | params         | pack settings row 0    | (pack-defined)              |
/// | viewport.x/.y  | surface px             | `viewWidth` / `viewHeight`  |
/// | viewport.z/.w  | 1/w, 1/h               | `pixelWidth`-style inverse  |
/// | time.x         | seconds since boot     | `frameTimeCounter`          |
/// | time.y         | day cycle 0..1         | `worldTime` (normalized)    |
/// | time.z         | underwater flag        | `isEyeInWater`              |
/// | time.w         | sky-light floor        | `eyeBrightness` (scaled)    |
///
/// v1 deliberately exposes COLOR + TIME state only (no matrices, no
/// depth) — a pack cannot break geometry, bindings or parity by design.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, PartialEq)]
pub struct PackUniform {
    /// pack settings row 0 (v1: declared defaults)
    pub params: [f32; 4],
    /// (w, h, 1/w, 1/h)
    pub viewport: [f32; 4],
    /// (seconds, day 0..1, underwater, skylight)
    pub time: [f32; 4],
}

/// one pack setting (v1: fixed at `default` — the JSON knob exists, the
/// options-screen wiring is the documented v2; §32 integration deferred)
#[derive(Clone, Debug, Deserialize)]
pub struct PackSetting {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String, // "slider" | "toggle"
    #[serde(default)]
    pub min: f32,
    #[serde(default = "one")]
    pub max: f32,
    pub default: f32,
}

fn one() -> f32 {
    1.0
}

/// JSON-only color grade a pack can apply without any WGSL (§34.1
/// "material definitions" — engine-side knobs, zero shader code)
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct GradePreset {
    pub saturation: f32,
    pub bloom: f32,
    pub exposure: f32,
    pub vignette: f32,
}

impl GradePreset {
    /// engine PostUniform.q row, clamped to engine-safe ranges
    pub fn as_q(&self) -> [f32; 4] {
        [
            self.bloom.clamp(0.0, 2.0),
            self.vignette.clamp(0.0, 1.0),
            self.saturation.clamp(0.0, 2.0),
            self.exposure.clamp(0.05, 4.0),
        ]
    }
}

/// a parsed shader pack (from JSON manifest + optional embedded WGSL)
#[derive(Clone, Debug)]
pub struct ShaderPack {
    /// stable identifier (directory name / registration key)
    pub id: String,
    pub name: String,
    /// honest compatibility tier — v1 packs are `SHADER-PACK-API`
    pub tier: String,
    #[allow(dead_code)]
    pub description: String,
    /// JSON grade preset (applied by the engine's composite)
    pub grade: GradePreset,
    /// custom composite WGSL (must satisfy PACK_CONTRACT)
    pub composite: Option<String>,
    #[allow(dead_code)]
    pub settings: Vec<PackSetting>,
}

/// the documented v1 pack contract, embedded in every error message so
/// pack authors see it exactly where they fail
pub const PACK_CONTRACT: &str = "\
VoxelCraft shader-pack API v1 (tier: SHADER-PACK-API)
The composite WGSL must define:
  struct PackU { params: vec4<f32>, viewport: vec4<f32>, time: vec4<f32> };
  fn packGrade(uv: vec2<f32>, scene: vec3<f32>, bloom: vec3<f32>, u: PackU) -> vec3<f32>;
The engine wraps packGrade into the composite pass AFTER FSR RCAS: it
receives the graded scene color and the bloom buffer color for the texel,
returns the final linear RGB (0..1 is a safe output range; the engine
encodes to sRGB). Textures/samplers are engine-owned; packs only compute
on the given inputs (v1 cannot sample, cannot touch geometry).";

/// parse + validate a pack manifest (JSON) with optional composite WGSL.
/// Validation is naga-based (real parse + type-check of the WRAPPED
/// module) so a broken pack can never reach pipeline creation (§46).
pub fn parse_pack(id: &str, manifest_json: &str, composite_wgsl: Option<&str>) -> Result<ShaderPack, String> {
    #[derive(Deserialize)]
    struct Manifest {
        name: String,
        #[serde(default = "default_tier")]
        tier: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        grade: GradePreset,
        composite: Option<String>,
        #[serde(default)]
        settings: Vec<PackSetting>,
    }
    fn default_tier() -> String {
        "SHADER-PACK-API".into()
    }
    let m: Manifest =
        serde_json::from_str(manifest_json).map_err(|e| format!("pack {id}: manifest JSON error: {e}"))?;
    if m.name.trim().is_empty() {
        return Err(format!("pack {id}: empty name"));
    }
    // honest tier labeling (§34.2): packs may not self-declare above the
    // API tier — the engine decides what it actually demonstrated
    let tier = if m.tier.is_empty() { default_tier() } else { m.tier };
    if tier != "SHADER-PACK-API" {
        return Err(format!(
            "pack {id}: tier '{tier}' is not granted by this engine (only SHADER-PACK-API is; see spec §34.2)"
        ));
    }
    let composite = match composite_wgsl.or(m.composite.as_deref()) {
        None => None,
        Some(src) => {
            let wrapped = wrap_composite(src);
            validate_wgsl(&wrapped).map_err(|e| format!("pack {id}: {e}\n{PACK_CONTRACT}"))?;
            Some(src.to_string())
        }
    };
    Ok(ShaderPack {
        id: id.to_string(),
        name: m.name,
        tier,
        description: m.description,
        grade: m.grade,
        composite,
        settings: m.settings,
    })
}

/// wrap a pack's packGrade into a complete WGSL module with the engine's
/// bindings. The pack source is embedded VERBATIM (no preprocessor, no
/// include resolution — §34.1 keep-it-simple, packs are self-contained).
pub fn wrap_composite(pack_src: &str) -> String {
    format!(
        r#"{pack_src}

// ---- engine wrapper (VoxelCraft SHADER-PACK-API v1) ----
// Bindings are engine-owned; packGrade is called after FSR RCAS + the
// engine grade with the texel's scene and bloom colors.
@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> U: PackU;

struct VsOut {{
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {{
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[vi], 0.0, 1.0);
    // writes to the SWAPCHAIN → NDC→UV V-flip (matches POST_SHADER)
    out.uv = vec2<f32>(p[vi].x * 0.5 + 0.5, 0.5 - p[vi].y * 0.5);
    return out;
}}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {{
    let scene = textureSample(scene_tex, samp, in.uv).rgb;
    let bloom = textureSample(bloom_tex, samp, in.uv).rgb;
    let col = packGrade(in.uv, scene, bloom, U);
    return vec4<f32>(col, 1.0);
}}
"#
    )
}

/// parse + type-check a full WGSL module with naga (the same frontend
/// wgpu 22 uses) — usable at runtime AND in tests
pub fn validate_wgsl(src: &str) -> Result<(), String> {
    let mut frontend = naga::front::wgsl::Frontend::new();
    let module = frontend
        .parse(src)
        .map_err(|e| format!("WGSL parse error: {e}"))?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator
        .validate(&module)
        .map_err(|e| format!("WGSL type-check error: {e:?}"))?;
    // the wrapper must actually call packGrade with the exact signature —
    // a pack that renames or re-types it fails VALIDATION (not at boot)
    let entry_ok = module
        .entry_points
        .iter()
        .any(|e| e.name == "fs_main");
    if !entry_ok {
        return Err("WGSL error: engine wrapper lost its fs_main entry point".into());
    }
    let _ = info;
    Ok(())
}

// ------------------------------------------------------------- builtin --

/// demo pack 1 — pure grade + warmth curve (clean-room, our own art)
const WARM_EVENING_JSON: &str = include_str!("../shader-packs/warm-evening/shaders.json");
const WARM_EVENING_WGSL: &str = include_str!("../shader-packs/warm-evening/composite.wgsl");
/// demo pack 2 — time-varying grain + blue shift (proves uniforms flow)
const MOONLIT_JSON: &str = include_str!("../shader-packs/moonlit/shaders.json");
const MOONLIT_WGSL: &str = include_str!("../shader-packs/moonlit/composite.wgsl");

/// the engine's clean-room demo packs — explicitly tested (§48 Phase-11
/// gate: "demonstrated compatibility with explicitly tested packs")
pub fn builtin_packs() -> Vec<ShaderPack> {
    let mut out = Vec::new();
    match parse_pack("warm-evening", WARM_EVENING_JSON, Some(WARM_EVENING_WGSL)) {
        Ok(p) => out.push(p),
        Err(e) => crate::render::report_boot_log(&format!("builtin pack warm-evening failed validation: {e}")),
    }
    match parse_pack("moonlit", MOONLIT_JSON, Some(MOONLIT_WGSL)) {
        Ok(p) => out.push(p),
        Err(e) => crate::render::report_boot_log(&format!("builtin pack moonlit failed validation: {e}")),
    }
    out
}

/// native-only: also load packs from a `shader-packs/` directory next to
/// the executable/cwd (§34.1 recompilation path — external packs without
/// rebuilding). Invalid packs are skipped with a log line (§46), never
/// fatal.
#[cfg(not(target_arch = "wasm32"))]
pub fn external_packs() -> Vec<ShaderPack> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("shader-packs") else {
        return out;
    };
    for e in entries.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let manifest = match std::fs::read_to_string(path.join("shaders.json")) {
            Ok(m) => m,
            Err(_) => {
                crate::render::report_boot_log(&format!("shader pack {id}: no shaders.json, skipped"));
                continue;
            }
        };
        let wgsl = std::fs::read_to_string(path.join("composite.wgsl")).ok();
        match parse_pack(&id, &manifest, wgsl.as_deref()) {
            Ok(p) => out.push(p),
            Err(err) => crate::render::report_boot_log(&format!("shader pack {id} rejected: {err}")),
        }
    }
    out
}

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
struct PackU { params: vec4<f32>, viewport: vec4<f32>, time: vec4<f32> };
fn packGrade(uv: vec2<f32>, scene: vec3<f32>, bloom: vec3<f32>, u: PackU) -> vec3<f32> {
    return scene + bloom * u.params.x;
}
"#;

    #[test]
    fn builtin_packs_parse_and_validate() {
        let packs = builtin_packs();
        assert_eq!(packs.len(), 2, "both demo packs must load");
        for p in &packs {
            assert_eq!(p.tier, "SHADER-PACK-API");
            assert!(p.composite.is_some());
            // the WRAPPED module must pass the real naga validation
            validate_wgsl(&wrap_composite(p.composite.as_ref().unwrap())).unwrap();
        }
        assert!(packs.iter().any(|p| p.id == "warm-evening"));
        assert!(packs.iter().any(|p| p.id == "moonlit"));
    }

    #[test]
    fn manifest_parse_and_grade_clamp() {
        let json = r#"{
            "name": "Test Pack",
            "description": "x",
            "grade": { "saturation": 5.0, "bloom": -1.0, "exposure": 2.0, "vignette": 0.3 }
        }"#;
        let p = parse_pack("test", json, None).unwrap();
        assert_eq!(p.name, "Test Pack");
        assert_eq!(p.tier, "SHADER-PACK-API");
        assert!(p.composite.is_none());
        let q = p.grade.as_q();
        assert_eq!(q, [0.0, 0.3, 2.0, 2.0], "clamped to engine-safe ranges");
    }

    #[test]
    fn tier_escalation_rejected() {
        let json = r#"{ "name": "Liar", "tier": "FULL-COMPATIBILITY" }"#;
        let err = parse_pack("liar", json, None).unwrap_err();
        assert!(err.contains("§34.2"), "must cite the spec rule: {err}");
    }

    #[test]
    fn bad_wgsl_rejected_by_validation() {
        // missing packGrade entirely
        let err = parse_pack("bad", r#"{ "name": "Bad" }"#, Some("fn nothing() {}")).unwrap_err();
        assert!(err.contains("packGrade") || err.contains("parse"), "{err}");
        // wrong signature
        let wrong = r#"
struct PackU { params: vec4<f32>, viewport: vec4<f32>, time: vec4<f32> };
fn packGrade(a: f32) -> vec3<f32> { return vec3<f32>(a); }
"#;
        let err = parse_pack("bad2", r#"{ "name": "Bad2" }"#, Some(wrong)).unwrap_err();
        assert!(err.contains("error"), "{err}");
    }

    #[test]
    fn wrapper_carries_pack_source_and_bindings() {
        let wrapped = wrap_composite(GOOD);
        assert!(wrapped.contains("fn packGrade"));
        assert!(wrapped.contains("@group(0) @binding(0) var scene_tex"));
        assert!(wrapped.contains("@fragment"));
        validate_wgsl(&wrapped).unwrap();
    }

    #[test]
    fn pack_uniform_layout() {
        // 3 vec4 rows, Pod-safe
        assert_eq!(std::mem::size_of::<PackUniform>(), 48);
        let u = PackUniform {
            params: [1.0, 0.0, 0.0, 0.0],
            viewport: [1280.0, 720.0, 1.0 / 1280.0, 1.0 / 720.0],
            time: [12.5, 0.75, 0.0, 0.9],
        };
        let bytes: &[u8] = bytemuck::bytes_of(&u);
        assert_eq!(bytes.len(), 48);
    }
}
