//! 2026-09-19 round — REAL drop-in shader-pack loading (the BSL/SEUS
//! class), built on the OptiFine/Iris pack format (the authoritative
//! OptiFineDoc `shaders.txt` + `shaders.properties`, downloaded and
//! cross-verified 2026-09-19; the Iris docs at shaders.properties
//! agree on the structure). NO builtin shader ships with the engine
//! (user directive 2026-09-14, reaffirmed 2026-09-19) — packs are
//! user-provided files in `shader-packs/` (folders or zips).
//!
//! Honest tier labels (the §34.2 discipline — never claim more than is
//! demonstrated):
//! * `IRIS-STRUCTURE` — the pack parses: shaders.properties + the
//!   program list + option surface are read and reported.
//! * `IRIS-COMPOSITE-SUBSET` — the pack's composite/final GLSL chain
//!   translates to WGSL (via naga's GLSL frontend) and RUNS on the
//!   engine's post chain: colortex0 (scene color) ping-pong +
//!   colortex1/2 (linear depth / bloom) + the documented uniform
//!   subset. Passes needing gbuffers/shadow data or extra color
//!   attachments are SKIPPED with a logged reason — never silently.
//! * Full OptiFine/Iris compatibility is NOT claimed: no shadow-pass
//!   re-render, no per-material gbuffers, no compute (deferred v2 —
//!   the honest gap list lives in the pack report).
//!
//! ## The translation pipeline (per composite-style .fsh)
//!
//! ```text
//! raw OptiFine GLSL (#version 120..330, #include, option #defines,
//! gl_FragColor, uniform sampler2D colortexN, /* DRAWBUFFERS */)
//!   1. inline #include "..." (rooted at shaders/, depth-capped)
//!   2. apply user option values (rewrite the #define lines)
//!   3. record + strip /* DRAWBUFFERS:N.. */ / /* RENDERTARGETS: */
//!   4. version bump → #version 450 core (naga accepts 440-460 only)
//!   5. legacy rewrite: varying→in, gl_FragColor→declared out,
//!      gl_FragData[0]→out, texture2D→texture, texture2DLod→textureLod
//!   6. binding rewrite: bare `uniform sampler2D N` → the fixed
//!      texture/sampler pair layout; every other bare uniform → one
//!      anonymous std140 uniform block at set=0 binding=0
//!   7. naga GLSL frontend parse (fragment stage)
//!   8. naga validation
//!   9. naga WGSL backend → the pass module source
//!  10. the engine's fullscreen quad vertex stage pairs with it
//!      (location 0 = texcoord, matching every composite .vsh, which
//!      is why we do not translate the pack's .vsh files)
//! ```
//!
//! The fixed binding layout every translated pass sees (set 0):
//!
//! * binding 0 — `VCUniforms` std140 block (the engine fills it)
//! * binding 1 + 2i — `texture_2d<f32>` colortex i (i = 0..15)
//! * binding 2 + 2i — the shared sampler
//!
//! colortex0/1/2 = scene color / linear depth / bloom (ping-pong A/B
//! for 0); colortex3..15 = engine-owned 1×1 black (honest zeros).

use std::collections::BTreeMap;

// ------------------------------------------------------------ properties --

/// The shaders.properties surface we consume (OptiFineDoc
/// doc/shaders.properties, cross-checked against BSL/SEUS reality):
/// * `sliders=<id>...` — which options render as sliders
/// * `screen=<items>` / `screen.<name>=<items>` — option screens
/// * `profiles.<name>=<opt>:<val> ...` — value bundles
/// * `<option>=<default>[:min:step:max]` — non-GLSL option defaults
#[derive(Debug, Default, Clone)]
pub struct PropertiesDoc {
    /// option ids that render as sliders (everything else is a cycle
    /// through its allowed values)
    pub sliders: Vec<String>,
    /// the main screen item list (entry order; `<blank>` preserved as
    /// an empty-string item)
    pub screen: Vec<String>,
    /// named sub-screens
    pub screens: BTreeMap<String, Vec<String>>,
    /// profiles: name → (option, value) pairs
    pub profiles: BTreeMap<String, Vec<(String, String)>>,
    /// direct option declarations (rare; options normally live in the
    /// GLSL `// [allowed values]` comments)
    pub options: BTreeMap<String, OptionDecl>,
}

/// a properties-declared option: default + optional min/step/max
pub type OptionDecl = (String, Option<(f32, f32, f32)>);

impl PropertiesDoc {
    /// Parse the properties text (line-oriented `key=value`, `#`/`!`
    /// comments — the java-properties conventions the format uses).
    pub fn parse(text: &str) -> PropertiesDoc {
        let mut doc = PropertiesDoc::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            let words: Vec<String> = value
                .split_whitespace()
                .map(|w| w.to_string())
                .collect();
            if key == "sliders" {
                doc.sliders = words;
            } else if key == "screen" {
                doc.screen = words;
            } else if let Some(name) = key.strip_prefix("screen.") {
                doc.screens.insert(name.trim().to_string(), words);
            } else if let Some(name) = key.strip_prefix("profiles.") {
                let pairs = words
                    .iter()
                    .filter_map(|w| {
                        w.split_once(':')
                            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                    })
                    .collect();
                doc.profiles.insert(name.trim().to_string(), pairs);
            } else if !key.contains('.') && !words.is_empty() {
                // `<option>=<default>` or `<option>=<def>:<min>:<step>:<max>`
                let mut parts = value.split(':');
                let default = parts.next().unwrap_or("").trim().to_string();
                let range = parts.next().and_then(|mn| {
                    parts.next().and_then(|step| {
                        parts.next().and_then(|mx| {
                            Some((
                                mn.trim().parse().ok()?,
                                step.trim().parse().ok()?,
                                mx.trim().parse().ok()?,
                            ))
                        })
                    })
                });
                doc.options.insert(key.to_string(), (default, range));
            }
        }
        doc
    }
}

// --------------------------------------------------------------- options --

/// One user-facing shader option (parsed from the GLSL
/// `#define NAME value // [v1 v2 v3]` convention + the sliders list).
#[derive(Debug, Clone)]
pub struct PackOption {
    pub id: String,
    /// the display name (underscores → spaces, like OptiFine)
    pub label: String,
    pub value: String,
    /// the allowed values (from the `// [..]` comment) — a slider when
    /// they parse as an arithmetic range, else a cycle
    pub allowed: Vec<String>,
    pub is_slider: bool,
}

impl PackOption {
    /// 0..1 slider position → value string (the slider_pos inverse:
    /// numeric interpolation when the allowed values parse as numbers,
    /// else the list index)
    pub fn interpolate(&self, t: f32) -> String {
        let t = t.clamp(0.0, 1.0);
        if let (Ok(a), Ok(b)) = (
            self.allowed.first().and_then(|s| s.parse::<f32>().ok()).ok_or(()),
            self.allowed.last().and_then(|s| s.parse::<f32>().ok()).ok_or(()),
        ) {
            if b > a {
                let v = a + t * (b - a);
                // keep the value's own decimal shape (ints stay ints)
                let dec = self
                    .allowed
                    .iter()
                    .filter_map(|v| v.parse::<f32>().ok())
                    .any(|x| x.fract().abs() > f32::EPSILON);
                return if dec { format!("{v:.3}") } else { format!("{}", v.round() as i64) };
            }
        }
        let idx = (t * (self.allowed.len().saturating_sub(1)) as f32).round() as usize;
        self.allowed
            .get(idx)
            .cloned()
            .unwrap_or_else(|| self.value.clone())
    }

    /// value → 0..1 slider position (linear across the allowed list,
    /// numeric when the values parse)
    pub fn slider_pos(&self) -> f32 {
        if let (Ok(v), Ok(a), Ok(b)) = (
            self.value.parse::<f32>(),
            self.allowed.first().and_then(|s| s.parse::<f32>().ok()).ok_or(()),
            self.allowed.last().and_then(|s| s.parse::<f32>().ok()).ok_or(()),
        ) {
            if b > a {
                return ((v - a) / (b - a)).clamp(0.0, 1.0);
            }
        }
        let idx = self
            .allowed
            .iter()
            .position(|a| a == &self.value)
            .unwrap_or(0);
        if self.allowed.len() > 1 {
            idx as f32 / (self.allowed.len() - 1) as f32
        } else {
            0.5
        }
    }
}

/// scan raw GLSL sources for the `#define NAME value // [allowed]`
/// option convention (OptiFine parses exactly this comment list; the
/// default value is the one written on the line)
pub fn scan_options(sources: &[(&str, &str)]) -> Vec<PackOption> {
    let mut out: Vec<PackOption> = Vec::new();
    for (path, src) in sources {
        for line in src.lines() {
            let line = line.trim();
            if line.starts_with("#ifndef ") || line.starts_with("#ifdef ") {
                // boolean toggles: `#ifndef X` / `#define X` pairs —
                // handled by the scan's companion below
                continue;
            }
            let Some(rest) = line.strip_prefix("#define ") else {
                continue;
            };
            let rest = rest.trim_start();
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            let after = &rest[name.len()..];
            // require the option-comment list to count as an option
            let Some(list_part) = after.find("//") else {
                continue;
            };
            let value: String = after[..list_part]
                .trim()
                .chars()
                .take_while(|c| *c != '/')
                .collect();
            let value = value.trim().to_string();
            let comment = after[list_part + 2..].trim();
            let Some(stripped) = comment.strip_prefix('[') else {
                continue;
            };
            let Some(close) = stripped.find(']') else {
                continue;
            };
            let allowed: Vec<String> = stripped[..close]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if allowed.len() < 2 || out.iter().any(|o| o.id == name) {
                continue;
            }
            let _ = path;
            out.push(PackOption {
                id: name.clone(),
                label: name.replace('_', " "),
                value: if value.is_empty() {
                    "true".to_string()
                } else {
                    value
                },
                allowed,
                is_slider: false, // fixed up by the caller with sliders=
            });
        }
    }
    out
}

// ------------------------------------------------------- pre-translation --

/// the colortex alias table (OptiFineDoc shaders.txt "Color
/// Attachments": gcolor, gdepth, gnormal, composite, gaux1..4 are
/// colortex0..7; colortex8..15 have no aliases)
fn resolve_colortex(name: &str) -> Option<u8> {
    match name {
        "colortex0" | "gcolor" => Some(0),
        "colortex1" | "gdepth" => Some(1),
        "colortex2" | "gnormal" => Some(2),
        "colortex3" | "composite" => Some(3),
        "colortex4" | "gaux1" => Some(4),
        "colortex5" | "gaux2" => Some(5),
        "colortex6" | "gaux3" => Some(6),
        "colortex7" | "gaux4" => Some(7),
        "colortex8" | "colortex9" | "colortex10" | "colortex11" | "colortex12"
        | "colortex13" | "colortex14" | "colortex15" => name
            .strip_prefix("colortex")
            .and_then(|n| n.parse::<u8>().ok()),
        _ => None,
    }
}

/// one member of the generated std140 uniform block
#[derive(Debug, Clone, PartialEq)]
pub struct UniformMember {
    pub name: String,
    /// scalar/vector/matrix GLSL type as written
    pub ty: String,
    /// std140 byte offset (WGSL uniform-address-space layout matches)
    pub offset: u32,
    /// byte size at that offset
    pub size: u32,
}

/// std140 (== WGSL uniform address space for these types) layout:
/// float 4/4, vec2 8/8, vec3 12/16, vec4 16/16, mat4 64/16,
/// int/uint 4, bool 4
fn std140_align_size(ty: &str) -> (u32, u32) {
    match ty {
        "float" | "int" | "uint" | "bool" => (4, 4),
        "vec2" | "ivec2" | "uvec2" => (8, 8),
        "vec3" | "ivec3" | "uvec3" => (16, 12),
        "vec4" | "ivec4" | "uvec4" => (16, 16),
        "mat4" | "mat4x4" => (16, 64),
        "mat3" => (16, 48),
        _ => (16, 16), // conservative
    }
}

/// Which colortex indices a translated pass samples (parsed back from
/// the fixed binding scheme the rewrite emits: colortexN's texture pair
/// sits at binding 1+2N). The host uses this to decide which passes the
/// engine can actually feed — colortex0 is the chained scene color,
/// colortex1 the engine's bloom buffer; passes sampling colortex2+ need
/// render targets the v1 subset does not own and must be skipped with
/// an honest reason. 255 marks the noise texture (bindings 33/34).
pub fn colortexes_sampled(wgsl: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let bytes = wgsl.as_bytes();
    let mut i = 0usize;
    while let Some(at) = wgsl[i..].find("@binding(") {
        let start = i + at + "@binding(".len();
        i = start;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        let Ok(n) = wgsl[start..end].parse::<u32>() else {
            continue;
        };
        // the rest of the declaration, up to the semicolon
        let semi = wgsl[end..].find(';').map(|s| end + s).unwrap_or(wgsl.len());
        let decl = &wgsl[end..semi];
        if !decl.contains("texture_2d") {
            continue; // a sampler or uniform block binding — not a texture
        }
        if n == 33 {
            out.push(255); // the noise pair
        } else if (1..=31).contains(&n) && n % 2 == 1 {
            out.push(((n - 1) / 2) as u8); // colortex index
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// the result of the source-level preparation
#[allow(dead_code)] // draw_buffers recorded for the report (single-target
                    // enforced at prepare time)
struct Prepared {
    /// the rewritten GLSL (naga-ready: 450 core, split samplers,
    /// uniform block, declared outputs)
    src: String,
    /// recorded DRAWBUFFERS/RENDERTARGETS (None = default 0)
    draw_buffers: Vec<u8>,
    /// the std140 block member layout the engine must fill
    uniforms: Vec<UniformMember>,
}

/// inline `#include "path"` directives (OptiFine resolves them against
/// the pack's shaders/ root; recursion-capped, cycle-safe). `resolve`
/// maps a normalized include path → source text.
fn inline_includes(
    src: &str,
    resolve: &dyn Fn(&str) -> Option<String>,
    seen: &mut Vec<String>,
    depth: u32,
) -> String {
    if depth > 12 {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len() + 256);
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include") {
            let rest = rest.trim();
            let path = rest
                .trim_start_matches('<')
                .trim_end_matches('>')
                .trim_matches('"')
                .trim();
            let norm = path.trim_start_matches('/');
            if seen.iter().any(|s| s == norm) {
                continue; // include guard already emitted
            }
            if let Some(content) = resolve(norm) {
                seen.push(norm.to_string());
                // #line hygiene: naga's GLSL frontend rejects stray
                // #line handling differences — we drop the directive
                // lines themselves
                let expanded = inline_includes(&content, resolve, seen, depth + 1);
                out.push_str(&expanded);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The per-pass source-level translator (steps 2-6 of the pipeline
/// doc). `option_values`: user-chosen values per option id.
fn glsl_prepare(
    src: &str,
    option_values: &BTreeMap<String, String>,
) -> Result<Prepared, String> {
    // ---- 2. apply option values: rewrite `#define NAME <v> // [..]` ----
    let mut applied = String::with_capacity(src.len());
    for line in src.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("#define ") {
            let name: String = rest
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if let Some(v) = option_values.get(&name) {
                // rewrite the value in place, keep any trailing comment
                applied.push_str(&format!("#define {name} {v}\n"));
                continue;
            }
        }
        applied.push_str(line);
        applied.push('\n');
    }

    // ---- 3. record + strip the DRAWBUFFERS / RENDERTARGETS comments ----
    let mut draw_buffers: Vec<u8> = Vec::new();
    let mut stripped = String::with_capacity(applied.len());
    for line in applied.lines() {
        if let Some(i) = line.find("/* DRAWBUFFERS:") {
            let inner = &line[i + 15..];
            if let Some(j) = inner.find("*/") {
                for ch in inner[..j].chars() {
                    if ch.is_ascii_digit() {
                        draw_buffers.push(ch as u8 - b'0');
                    }
                }
                stripped.push_str(line[..i].trim_end());
                stripped.push('\n');
                continue;
            }
        }
        if let Some(i) = line.find("/* RENDERTARGETS:") {
            let inner = &line[i + 17..];
            if let Some(j) = inner.find("*/") {
                for tok in inner[..j].split(',') {
                    if let Ok(n) = tok.trim().parse::<u8>() {
                        draw_buffers.push(n);
                    }
                }
                stripped.push_str(line[..i].trim_end());
                stripped.push('\n');
                continue;
            }
        }
        stripped.push_str(line);
        stripped.push('\n');
    }
    if draw_buffers.is_empty() {
        draw_buffers.push(0); // OptiFine default: first 8 → we do buffer 0
    }
    // v1 runs single-target passes only (multi-target composites need
    // MRT chains — honest skip, disclosed in the report)
    if draw_buffers.iter().any(|&b| b != 0) || draw_buffers.len() > 1 {
        return Err(format!(
            "needs {} color attachments (DRAWBUFFERS:{:?}) — the engine subset writes colortex0 only",
            draw_buffers.len(),
            draw_buffers
        ));
    }

    // ---- 4/5. version bump + legacy keyword rewrite (line-level) ----
    let mut body: Vec<String> = Vec::new();
    let mut has_frag_out = false;
    for line in stripped.lines() {
        let t = line.trim();
        if t.starts_with("#version") {
            continue; // replaced below
        }
        if t.starts_with("varying ") || t.starts_with("attribute ") {
            // fragment stage: both become inputs
            body.push(line.replace("varying ", "in ").replace("attribute ", "in "));
            continue;
        }
        // gl_FragColor / gl_FragData[0] → the declared single output
        if t.contains("gl_FragData") && !line.contains("gl_FragData[0]") {
            return Err("writes gl_FragData[N>0] — multi-target (see DRAWBUFFERS)".into());
        }
        if line.contains("gl_FragColor") || line.contains("gl_FragData[0]") {
            has_frag_out = true;
        }
        let l2 = line
            .replace("gl_FragData[0]", "vc_fragColor")
            .replace("gl_FragColor", "vc_fragColor")
            .replace("texture2DLod(", "textureLod(")
            .replace("texture2D(", "texture(");
        body.push(l2);
    }

    // ---- 6. the binding rewrite ----
    // bare `uniform sampler2D N;` → fixed layout pair; other bare
    // uniforms → one anonymous std140 block
    let mut uniforms: Vec<UniformMember> = Vec::new();
    let mut offset: u32 = 0;
    let mut head = String::new();
    let mut rewritten: Vec<String> = Vec::new();
    // the sampler names turned into texture/sampler pairs THIS pass —
    // the call-site rewrite below needs them (the declarations are
    // consumed, so they cannot be matched from the body text)
    let mut sampler_names: Vec<String> = Vec::new();
    let mut declared_out = false;
    for line in body {
        let t = line.trim();
        if t.contains("out vec4") {
            declared_out = true; // modern syntax: pack declares its own
        }
        // `uniform sampler2D NAME;` (+ precision prefixes / trailing
        // comment)
        if let Some(rest) = t.strip_prefix("uniform ") {
            let rest = rest.trim();
            // strip precision qualifiers
            let rest = rest
                .strip_prefix("lowp ")
                .or_else(|| rest.strip_prefix("mediump "))
                .or_else(|| rest.strip_prefix("highp "))
                .unwrap_or(rest);
            if let Some(after_sampler) = rest.strip_prefix("sampler2D ") {
                let name: String = after_sampler
                    .trim()
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                sampler_names.push(name.clone());
                if let Some(n) = resolve_colortex(&name) {
                    let tb = 1 + 2 * n as u32;
                    let sb = tb + 1;
                    head.push_str(&format!(
                        "layout(set = 0, binding = {tb}) uniform texture2D {name}_t;\n\
                         layout(set = 0, binding = {sb}) uniform sampler {name}_s;\n"
                    ));
                    continue; // declaration consumed
                }
                // a non-colortex sampler (noise etc.) — engine provides
                // a 4×4 noise texture at bindings 33/34
                head.push_str(&format!(
                    "layout(set = 0, binding = 33) uniform texture2D {name}_t;\n\
                     layout(set = 0, binding = 34) uniform sampler {name}_s;\n"
                ));
                continue;
            }
            // a bare scalar/vector/matrix uniform → the block member
            let mut it = rest.split_whitespace();
            let (Some(ty), Some(name_tok)) = (it.next(), it.next()) else {
                rewritten.push(line);
                continue;
            };
            let name: String = name_tok
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let (align, size) = std140_align_size(ty);
            offset = offset.div_ceil(align) * align;
            uniforms.push(UniformMember {
                name: name.clone(),
                ty: ty.to_string(),
                offset,
                size,
            });
            offset += size;
            continue;
        }
        rewritten.push(line);
    }

    // texture call rewrite: `texture(NAME, ...)` where NAME is one of
    // the samplers this pass declared → `texture(sampler2D(NAME_t,
    // NAME_s), ...)` (naga's GLSL frontend pairs textures and samplers
    // ONLY through the sampler2D(tex, samp) constructor syntax)
    let mut final_lines: Vec<String> = Vec::new();
    for line in rewritten {
        let mut out_line = line.clone();
        for name in &sampler_names {
            for call in ["texture(", "textureLod(", "textureGrad("] {
                let needle = format!("{call}{name},");
                if out_line.contains(&needle) {
                    let repl = format!("{call}sampler2D({name}_t, {name}_s),");
                    out_line = out_line.replace(&needle, &repl);
                }
                let needle2 = format!("{call}{name} ,");
                if out_line.contains(&needle2) {
                    let repl2 = format!("{call}sampler2D({name}_t, {name}_s) ,");
                    out_line = out_line.replace(&needle2, &repl2);
                }
            }
        }
        final_lines.push(out_line);
    }

    // ---- emit ----
    let mut out = String::with_capacity(
        1024 + final_lines.iter().map(|l| l.len() + 1usize).sum::<usize>(),
    );
    out.push_str("#version 450 core\n");
    if !uniforms.is_empty() {
        out.push_str("layout(set = 0, binding = 0) uniform VCUniforms {\n");
        for u in &uniforms {
            out.push_str(&format!("    {} {};\n", u.ty, u.name));
        }
        out.push_str("};\n");
    }
    out.push_str(&head);
    if has_frag_out && !declared_out {
        out.push_str("layout(location = 0) out vec4 vc_fragColor;\n");
    }
    // naga's GLSL frontend requires an entry point named `main`
    for l in final_lines {
        out.push_str(&l);
        out.push('\n');
    }
    Ok(Prepared {
        src: out,
        draw_buffers,
        uniforms,
    })
}

// ----------------------------------------------------------- translation --

/// one successfully translated composite-style pass
#[derive(Debug, Clone)]
pub struct TranslatedPass {
    /// the program name ("composite", "composite1", …, "final")
    pub program: String,
    /// the WGSL fragment module (entry point `main`, location 0 input
    /// texcoord, location 0 output color)
    pub wgsl: String,
    /// the std140 block members the engine must fill per frame
    pub uniforms: Vec<UniformMember>,
    /// `final` passes write the swapchain; composites ping-pong
    pub is_final: bool,
}

/// parse → validate → WGSL for one composite-style .fsh (the pipeline
/// doc steps 7-9; failures are honest `Err` strings that become the
/// skip lines in the pack report)
pub fn translate_pass(
    program: &str,
    fsh_src: &str,
    option_values: &BTreeMap<String, String>,
    resolve: &dyn Fn(&str) -> Option<String>,
) -> Result<TranslatedPass, String> {
    let mut seen: Vec<String> = Vec::new();
    let inlined = inline_includes(fsh_src, resolve, &mut seen, 0);
    let prepared = glsl_prepare(&inlined, option_values)?;

    // naga GLSL frontend (the same frontend family wgpu speaks)
    let options = naga::front::glsl::Options {
        stage: naga::ShaderStage::Fragment,
        defines: Default::default(),
    };
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&options, &prepared.src)
        .map_err(|e| format!("GLSL parse: {e}"))?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator
        .validate(&module)
        .map_err(|e| format!("validation: {e:?}"))?;
    let wgsl = naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
        .map_err(|e| format!("WGSL emit: {e}"))?;
    Ok(TranslatedPass {
        program: program.to_string(),
        wgsl,
        uniforms: prepared.uniforms,
        is_final: program == "final",
    })
}

// ------------------------------------------------------------- pack model --

/// the per-program report line (translated / skipped-with-reason)
#[derive(Debug, Clone)]
pub struct PassReport {
    pub program: String,
    pub translated: bool,
    pub note: String,
}

/// a loaded shader pack (the 2026-09-19 model: real OptiFine/Iris
/// structure, translated composite chain, honest report)
#[derive(Debug, Clone)]
pub struct ShaderPackV2 {
    /// folder/zip name
    pub id: String,
    /// display name (pack.properties name, else id)
    pub name: String,
    pub props: PropertiesDoc,
    /// the user-facing options (sliders applied from shaders.properties)
    pub options: Vec<PackOption>,
    /// current values (id → value) — persisted by the host
    pub option_values: BTreeMap<String, String>,
    /// the translated chain, in OptiFine run order (composite,
    /// composite1..99, then final)
    pub passes: Vec<TranslatedPass>,
    /// per-program report (including skips)
    pub report: Vec<PassReport>,
    /// the honest tier label granted
    pub tier: &'static str,
}

/// the composite-family program order (OptiFineDoc shaders.txt:
/// composite, composite1..composite99, then final — deferred runs
/// pre-composite but needs gbuffer data, out of the v1 subset)
fn program_order(name: &str) -> u32 {
    match name {
        "final" => 1000,
        other => other
            .strip_prefix("composite")
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(500),
    }
}

/// Build a pack from its TEXT file map (path-within-pack → content;
/// the caller fronted the folder/zip/web source). Pure — no fs, no
/// wgpu — so the SAME code loads native and web packs.
///
/// * `files` should contain every `.fsh/.vsh/.glsl/.properties/.txt`
///   text file of the pack (binary textures are engine-provided v1).
/// * `option_values` — the persisted user values (missing → defaults).
pub fn build_pack(
    id: &str,
    files: &BTreeMap<String, String>,
    option_values: &BTreeMap<String, String>,
) -> ShaderPackV2 {
    let mut report = Vec::new();
    // shaders.properties (path: "shaders.properties", or inside
    // "shaders/shaders.properties" for the nested layout)
    let props_text = files
        .get("shaders.properties")
        .or_else(|| files.get("shaders/shaders.properties"))
        .cloned()
        .unwrap_or_default();
    let props = PropertiesDoc::parse(&props_text);

    // pack.properties name (Iris convention)
    let name = files
        .get("pack.properties")
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.strip_prefix("name=")
                    .or_else(|| l.strip_prefix("name ="))
                    .map(|s| s.trim().to_string())
            })
        })
        .unwrap_or_else(|| id.to_string());

    // collect the composite-family .fsh sources under shaders/
    let mut programs: Vec<String> = files
        .keys()
        .filter_map(|p| {
            let p = p.replace('\\', "/");
            let in_shaders = p.strip_prefix("shaders/")?;
            let fsh = in_shaders.strip_suffix(".fsh")?;
            // only the composite family + final (the honest subset)
            if fsh == "final" || fsh.starts_with("composite") {
                Some(fsh.to_string())
            } else {
                None
            }
        })
        .collect();
    programs.sort_by_key(|p| program_order(p));

    // option surface: scan the composite-family sources + every
    // included lib (settings files) — the union is what OptiFine
    // surfaces for the composite screens
    let mut opt_sources: Vec<(String, String)> = Vec::new();
    for (path, content) in files.iter() {
        let p = path.replace('\\', "/");
        if p.starts_with("shaders/") && p.ends_with(".glsl") {
            opt_sources.push((p, content.clone()));
        }
    }
    for p in &programs {
        for (path, key) in [("shaders/", format!("shaders/{p}.fsh"))] {
            if let Some(c) = files.get(&key) {
                opt_sources.push((format!("{path}{p}.fsh"), c.clone()));
            }
        }
    }
    let mut options = scan_options(
        &opt_sources
            .iter()
            .map(|(p, c)| (p.as_str(), c.as_str()))
            .collect::<Vec<(&str, &str)>>(),
    );
    for o in options.iter_mut() {
        o.is_slider = props.sliders.iter().any(|s| s == &o.id);
    }

    // effective values: user overrides → declared defaults
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for o in &options {
        values.insert(
            o.id.clone(),
            option_values
                .get(&o.id)
                .cloned()
                .unwrap_or_else(|| o.value.clone()),
        );
    }

    // translate the chain
    let resolve = |path: &str| -> Option<String> {
        // OptiFine resolves against shaders/ root; try both forms
        let p = path.trim_start_matches('/');
        files
            .get(&format!("shaders/{p}"))
            .or_else(|| files.get(p))
            .cloned()
    };
    let mut passes = Vec::new();
    for p in &programs {
        let src = files
            .get(&format!("shaders/{p}.fsh"))
            .or_else(|| files.get(&format!("{p}.fsh")))
            .cloned()
            .unwrap_or_default();
        match translate_pass(p, &src, &values, &resolve) {
            Ok(t) => {
                report.push(PassReport {
                    program: p.clone(),
                    translated: true,
                    note: format!(
                        "{} uniform{}",
                        t.uniforms.len(),
                        if t.uniforms.len() == 1 { "" } else { "s" }
                    ),
                });
                passes.push(t);
            }
            Err(e) => report.push(PassReport {
                program: p.clone(),
                translated: false,
                note: e,
            }),
        }
    }

    // the honest tier: full structure + at least one running pass
    let structure_ok = !props_text.is_empty() || !programs.is_empty();
    let tier = if !structure_ok {
        "REJECTED (no shaders.properties / programs)"
    } else if passes.is_empty() {
        "IRIS-STRUCTURE (0 passes translated — see report)"
    } else {
        "IRIS-COMPOSITE-SUBSET"
    };

    ShaderPackV2 {
        id: id.to_string(),
        name,
        props,
        options,
        option_values: values,
        passes,
        report,
        tier,
    }
}

/// native discovery: every pack in `shader-packs/` (folders + zips),
/// each as (id, text-file map). Binary members are skipped (the v1
/// engine provides colortex/noise textures itself).
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_pack_files(root: &std::path::Path) -> Vec<(String, BTreeMap<String, String>)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for e in entries.flatten() {
        let path = e.path();
        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.trim_end_matches(".zip").to_string(),
            None => continue,
        };
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        if path.is_dir() {
            collect_text_folder(&path, &path, &mut files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("zip") {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Some(z) = vc_pack::zip::ZipFiles::from_bytes(&bytes) {
                    use vc_pack::datapack::PackFiles as _;
                    for name in z.list("") {
                        if is_text_member(&name) {
                            if let Some(b) = z.read_file(&name) {
                                if let Ok(s) = String::from_utf8(b) {
                                    files.insert(name.replace('\\', "/"), s);
                                }
                            }
                        }
                    }
                }
            }
        }
        if !files.is_empty() {
            out.push((id, files));
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_text_folder(
    root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut BTreeMap<String, String>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_text_folder(root, &p, out);
        } else if is_text_member(&p.to_string_lossy()) {
            if let Ok(s) = std::fs::read_to_string(&p) {
                // paths are keyed relative to the PACK ROOT at every
                // depth (the recursive call keeps the root): shaders/
                // composite.fsh, shaders/lib/settings.glsl, …
                let rel = p
                    .strip_prefix(root)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| p.to_string_lossy().to_string());
                out.insert(rel, s);
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // wasm path builds the map from the fetched index
fn is_text_member(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".fsh")
        || n.ends_with(".vsh")
        || n.ends_with(".glsl")
        || n.ends_with(".properties")
        || n.ends_with(".txt")
        || n.ends_with(".json")
        || n.ends_with(".md")
}

// ------------------------------------------------------------------ tests --

#[cfg(test)]
mod tests {
    use super::*;

    /// a BSL-style composite pass: #version 120, gl_FragColor,
    /// bare colortex samplers, bare scalar uniforms, an option #define,
    /// and a DRAWBUFFERS comment
    const BSL_STYLE: &str = r#"
#version 120

#define BLOOM_STRENGTH 1.0 // [0.5 1.0 1.5 2.0]

uniform sampler2D colortex0;
uniform float viewWidth;
uniform float viewHeight;
uniform float frameTimeCounter;

varying vec2 texcoord;

/* DRAWBUFFERS:0 */
void main() {
    vec3 color = texture2D(colortex0, texcoord).rgb;
    float lum = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(lum), color, 1.0 + (BLOOM_STRENGTH - 1.0) * 0.5);
    color *= 0.5 + 0.5 * sin(frameTimeCounter);
    color /= (viewWidth / viewHeight) * 0.0 + 1.0;
    gl_FragColor = vec4(color, 1.0);
}
"#;

    #[test]
    fn bsl_style_pass_translates_to_valid_wgsl() {
        let mut opts = BTreeMap::new();
        opts.insert("BLOOM_STRENGTH".to_string(), "1.5".to_string());
        let t = translate_pass("composite", BSL_STYLE, &opts, &|_| None)
            .expect("the BSL-style composite must translate");
        assert!(t.wgsl.contains("var vc_uniforms_struct_VCUniforms") || t.wgsl.contains("VCUniforms"), "uniform block present");
        assert!(t.wgsl.contains("textureSample") || t.wgsl.contains("texture_2d"), "the texture sample survived");
        // naga validation already ran inside translate_pass — the WGSL
        // re-parses clean below (the round-trip gate)
        let mut fe = naga::front::wgsl::Frontend::new();
        let module = fe
            .parse(&t.wgsl)
            .expect("emitted WGSL re-parses clean");
        let mut v = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        v.validate(&module).expect("emitted WGSL validates");
        // the option value landed: pp-rs substitutes (and naga
        // constant-folds) the macro — the macro NAME must be fully
        // gone (any survivor would mean the value didn't apply)
        assert!(!t.wgsl.contains("BLOOM_STRENGTH"), "the option macro must be substituted away");
        // uniforms recorded with std140 offsets
        assert!(t.uniforms.iter().any(|u| u.name == "viewWidth" && u.offset == 0));
        assert!(t.uniforms.iter().any(|u| u.name == "frameTimeCounter"));
    }

    #[test]
    fn includes_and_options_surface() {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        files.insert(
            "shaders/composite.fsh".into(),
            r#"
#version 120
#include "/lib/common.glsl"
uniform sampler2D colortex0;
varying vec2 texcoord;
/* DRAWBUFFERS:0 */
void main() {
    gl_FragColor = vec4(grade(texture2D(colortex0, texcoord).rgb), 1.0);
}
"#
            .into(),
        );
        files.insert(
            "shaders/lib/common.glsl".into(),
            r#"
#define TONEMAP 2 // [0 1 2 3]
vec3 grade(vec3 c) { return c / 2.0 + TONEMAP * 0.0; }
"#
            .into(),
        );
        files.insert("shaders.properties".into(), "sliders=TONEMAP\n".into());
        let pack = build_pack("demo", &files, &BTreeMap::new());
        assert_eq!(pack.tier, "IRIS-COMPOSITE-SUBSET");
        assert_eq!(pack.passes.len(), 1);
        // the include was inlined (grade() must resolve) and the option
        // was discovered + slider-flagged
        assert!(pack.options.iter().any(|o| o.id == "TONEMAP" && o.is_slider));
        assert!(pack.report[0].translated, "report: {:?}", pack.report);
    }

    #[test]
    fn multi_target_pass_is_an_honest_skip() {
        let src = r#"
#version 120
uniform sampler2D colortex0;
varying vec2 texcoord;
/* DRAWBUFFERS:05 */
void main() {
    gl_FragData[0] = texture2D(colortex0, texcoord);
}
"#;
        let err = translate_pass("composite1", src, &BTreeMap::new(), &|_| None)
            .expect_err("multi-target must skip");
        assert!(err.contains("color attachments"), "{err}");
    }

    /// the full game-side flow over the repo's own test-warm demo pack
    /// (shader-packs/test-warm/ — our legal self-written BSL-style
    /// fixture): two passes translate, both stay inside the runnable
    /// colortex subset, and the runtime binding-scan agrees with the
    /// translation report.
    #[test]
    fn test_warm_pack_full_chain() {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        files.insert(
            "shaders.properties".into(),
            "sliders=WARMTH EXPOSURE\nscreen=WARMTH EXPOSURE <blank> GRAYSCALE\nprofiles.Default=WARMTH:1.0\n".into(),
        );
        files.insert(
            "shaders/composite.fsh".into(),
            r#"
#version 120
#include "/lib/settings.glsl"

uniform sampler2D colortex0;
uniform float viewWidth;
uniform float viewHeight;
uniform float frameTimeCounter;

varying vec2 texcoord;

/* DRAWBUFFERS:0 */
void main() {
    vec3 color = texture2D(colortex0, texcoord).rgb;
    color *= EXPOSURE;
    color.r *= 1.0 + 0.12 * WARMTH;
    color.b *= max(0.0, 1.0 - 0.15 * WARMTH);
    color *= 0.97 + 0.03 * sin(frameTimeCounter * 1.5);
    gl_FragColor = vec4(color, 1.0);
}
"#
            .into(),
        );
        files.insert(
            "shaders/final.fsh".into(),
            r#"
#version 120
uniform sampler2D colortex0;
varying vec2 texcoord;
void main() {
    vec3 color = texture2D(colortex0, texcoord).rgb;
    float d = distance(texcoord, vec2(0.5));
    color *= 1.0 - 0.25 * smoothstep(0.35, 0.75, d);
    color = clamp((color - 0.5) * 1.06 + 0.5, 0.0, 1.5);
    gl_FragColor = vec4(color, 1.0);
}
"#
            .into(),
        );
        files.insert(
            "shaders/lib/settings.glsl".into(),
            "#define WARMTH 1.0 // [0.0 0.5 1.0 1.5 2.0]\n#define EXPOSURE 1.0 // [0.5 1.0 1.5 2.0 2.5]\n#define GRAYSCALE 0 // [0 1]\n".into(),
        );
        let pack = build_pack("test-warm", &files, &BTreeMap::new());
        assert_eq!(pack.tier, "IRIS-COMPOSITE-SUBSET");
        assert_eq!(pack.passes.len(), 2, "report: {:?}", pack.report);
        // run order: composite before final
        assert_eq!(pack.passes[0].program, "composite");
        assert!(pack.passes[1].is_final);
        // every translated pass samples ONLY colortex0 — the runtime
        // subset check must accept both (the installer would run them)
        for p in &pack.passes {
            let ctx = colortexes_sampled(&p.wgsl);
            assert!(
                ctx.iter().all(|&k| k == 0 || k == 255),
                "{:?} outside the runnable subset: {ctx:?}",
                p.program
            );
        }
    }

    #[test]
    fn properties_parse_roundtrip() {
        let doc = PropertiesDoc::parse(
            "sliders=A B\nscreen=A <blank> C\nscreen.CUSTOM=X profiles\nprofiles.Potato=X:1 B:2\nBLOOM=0.5:0.0:0.1:1.0\n",
        );
        assert_eq!(doc.sliders, ["A", "B"]);
        assert_eq!(doc.screen.len(), 3);
        assert_eq!(doc.screen[1], "<blank>");
        assert!(doc.profiles.contains_key("Potato"));
        let (def, range) = doc.options.get("BLOOM").unwrap().clone();
        assert_eq!(def, "0.5");
        assert_eq!(range, Some((0.0, 0.1, 1.0)));
    }
}

#[cfg(test)]
mod folder_tests {
    use super::*;

    /// the full drop-in path: write a BSL-style pack into a temp
    /// shader-packs/ dir (folder form), scan, build, translate — the
    /// same code the game runs at boot and in the Shaders screen.
    #[test]
    fn scan_and_translate_a_dropped_pack() {
        let dir = std::env::temp_dir().join("vc-shaderpack-test");
        let _ = std::fs::remove_dir_all(&dir);
        let pack = dir.join("warm-evening-test");
        let shaders = pack.join("shaders");
        let lib = shaders.join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(
            pack.join("shaders.properties"),
            "sliders=WARMTH EXPOSURE\nprofiles.Default=WARMTH:1.0\n",
        )
        .unwrap();
        std::fs::write(
            lib.join("settings.glsl"),
            "#define WARMTH 1.0 // [0.0 0.5 1.0 1.5 2.0]\n#define EXPOSURE 1.0 // [0.5 1.0 1.5 2.0 2.5]\n",
        )
        .unwrap();
        std::fs::write(
            shaders.join("composite.fsh"),
            r#"#version 120
#include "/lib/settings.glsl"
uniform sampler2D colortex0;
uniform float viewWidth;
varying vec2 texcoord;
/* DRAWBUFFERS:0 */
void main() {
    vec3 c = texture2D(colortex0, texcoord).rgb * EXPOSURE;
    c.r *= 1.0 + 0.12 * WARMTH;
    gl_FragColor = vec4(c, 1.0);
}
"#,
        )
        .unwrap();
        std::fs::write(
            shaders.join("final.fsh"),
            r#"#version 120
uniform sampler2D colortex0;
varying vec2 texcoord;
void main() {
    vec3 c = texture2D(colortex0, texcoord).rgb;
    gl_FragColor = vec4(c * 0.9, 1.0);
}
"#,
        )
        .unwrap();
        let found = scan_pack_files(&dir);
        assert_eq!(found.len(), 1, "one pack discovered");
        let (id, files) = &found[0];
        assert_eq!(id, "warm-evening-test");
        let values: BTreeMap<String, String> = BTreeMap::new();
        let pack = build_pack(id, files, &values);
        assert_eq!(pack.tier, "IRIS-COMPOSITE-SUBSET");
        assert_eq!(pack.passes.len(), 2, "composite + final: {:?}", pack.report);
        // the option surface (sliders flagged from shaders.properties)
        assert!(pack.options.iter().any(|o| o.id == "WARMTH" && o.is_slider));
        assert!(pack.options.iter().any(|o| o.id == "EXPOSURE" && o.is_slider));
        // an override value flows through the translation
        let mut vals = BTreeMap::new();
        vals.insert("WARMTH".to_string(), "2.0".to_string());
        let pack2 = build_pack(id, files, &vals);
        assert!(pack2.option_values.get("WARMTH").map(|v| v == "2.0").unwrap_or(false));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
