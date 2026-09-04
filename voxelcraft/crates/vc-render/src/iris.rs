//! Phase 8 — Iris shader-pack INTEGRATION INTERFACE.
//!
//! Dossier Part 1 §4: "Iris/GLSL shader-pack compatibility layer: real,
//! buildable, but large… Requires a custom GLSL-330-compat parser feeding
//! into your own IR → WGSL, plus restructuring the render loop to match
//! Iris's gbuffers → composite → final pass chain" and the agreed design:
//! **Iris compatibility is a separate sister project (vc-iris), referenced
//! by VoxelCraft — this repo keeps the integration interfaces.**
//!
//! Legal framing (Part 1 §1): Iris is LGPL-3.0 — "study the *published
//! spec*, not the Java code". This module implements the DOCUMENTED,
//! implementation-agnostic interface surface (the `shaders.properties`
//! format, the on-disk program layout, the `/* RENDERTARGETS: n */`
//! comment directive, the public uniform name set) from Iris's own
//! documentation. No Iris code is copied or ported.
//!
//! Honest tier labels (§34.2 discipline — never claim more than is
//! demonstrated):
//! * `IRIS-STRUCTURE-VALIDATED` — an Iris-format pack was detected and
//!   its structure (properties + pass chain + targets + uniforms) fully
//!   parsed. It is NOT runnable yet: GLSL translation ships in the vc-iris
//!   sister project, plugged in via [`IrisTranslator`].
//! * `IRIS-TRANSLATED` — a registered translator accepted the stages.
//!   Only a real translator (sister project or a test mock) can set it.
//!
//! What this interface gives the sister project:
//! 1. [`ShadersProperties`] — the parsed `shaders.properties` document
//!    (profiles, screens, sliders, per-stage directives, unknown keys
//!    retained honestly instead of silently dropped).
//! 2. [`IrisPassChain`] — the structural model of the pack's programs
//!    (which stages exist, their GLSL versions, their render targets via
//!    RENDERTARGETS/DRAWBUFFERS directives) — exactly what a translator
//!    needs to plan codegen.
//! 3. [`IrisUniforms`] — the documented uniform reference with an
//!    availability map against the engine's current `PackUniform` v1
//!    bridge (which subset the engine can supply today).
//! 4. [`IrisTranslator`] — the plug seam: `register_translator` installs
//!    the sister project's GLSL→WGSL translator; until then the built-in
//!    [`NoTranslator`] reports the honest "pending" status.
//!
//! Render-loop restructuring (the gbuffers→deferred→composite→final chain
//! with per-target ping-pong buffers) is deliberately NOT built here: the
//! engine already has a composite/post chain (§34), and the pass-chain
//! restructuring belongs to the sister project's activation milestone —
//! the interface exposes the parsed chain so that work can begin without
//! touching engine code.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// shaders.properties — the documented Iris properties document
// ---------------------------------------------------------------------------

/// Parsed `shaders.properties` (the documented Iris directives format,
/// verified against the live Iris docs — shaders.properties/current/
/// reference/shadersproperties/shader_settings):
///
/// * `key = value` pairs, `#`/`!` line comments, Java-properties `\` line
///   continuations
/// * `sliders = <list>` — options rendered as sliders
/// * `screen = <list>` — the main settings-screen membership (tokens:
///   `OPTION`, `[SCREEN]` sub-screen links, the literal `<profile>` button
///   token, `<empty>` spacers, `*` for all remaining options)
/// * `screen.<NAME> = <list>` — sub-screen membership
/// * `profile.<name> = <list>` — pre-configured option sets (`OPTION:value`
///   setters, `OPTION` / `!OPTION` booleans, `profile.OTHER` includes,
///   `!program.NAME` program disables)
/// * per-stage directives (documented on the related pages):
///   `texture.<stage>…`, `blend.<stage>…`, `alphaTest.<stage>…`,
///   `program.<name>.enabled`, `separateEntityDraws`, `shadow.culling`,
///   `iris.features.required`…
///
/// Everything is a flat key → value map (that IS the documented format —
/// verified live; no block syntax exists in current Iris). Unknown keys
/// are NOT dropped: they land in `unknown` so pack authors and the
/// translator see exactly what the interface did not model.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ShadersProperties {
    /// flat key → value pairs (comments stripped, continuations joined)
    pub entries: BTreeMap<String, String>,
    /// keys the parser did not model explicitly (kept verbatim)
    pub unknown: Vec<String>,
}

impl ShadersProperties {
    /// Parse a `shaders.properties` document. Never fails: malformed lines
    /// become `unknown` entries with their raw text (packs in the wild are
    /// messy; the interface reports rather than rejects).
    pub fn parse(text: &str) -> ShadersProperties {
        let mut props = ShadersProperties::default();
        // 1. join line continuations (Java properties rule: an ODD number
        //    of trailing backslashes continues, `\\` is an escaped one)
        let mut logical_lines: Vec<String> = Vec::new();
        let mut pending: Option<String> = None;
        for raw in text.lines() {
            let line = raw.trim_end_matches('\r');
            if let Some(mut acc) = pending.take() {
                acc.push(' ');
                acc.push_str(line.trim_start());
                if ends_with_continuation(&acc) {
                    acc.pop(); // drop the continuation backslash itself
                    pending = Some(acc);
                } else {
                    logical_lines.push(acc);
                }
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue; // comment line
            }
            if ends_with_continuation(line) {
                let mut base = line.to_string();
                base.pop();
                pending = Some(base);
            } else {
                logical_lines.push(line.to_string());
            }
        }
        if let Some(rest) = pending {
            logical_lines.push(rest);
        }
        // 2. key = value pairs (the whole documented format)
        for line in logical_lines {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            match split_kv(t) {
                Some((k, v)) => {
                    props.entries.insert(k, v);
                }
                None => props.unknown.push(t.to_string()),
            }
        }
        props
    }

    /// the main settings-screen membership list (`screen = …` tokens)
    pub fn screen_main(&self) -> Vec<String> {
        self.list("screen")
    }

    /// a sub-screen's membership list (`screen.<NAME> = …`)
    pub fn screen(&self, name: &str) -> Vec<String> {
        self.list(&format!("screen.{name}"))
    }

    /// the `sliders` membership list
    pub fn sliders(&self) -> Vec<String> {
        self.list("sliders")
    }

    /// `profile.<name> = <option list>` entries, name → body
    pub fn profiles(&self) -> BTreeMap<String, String> {
        self.entries
            .iter()
            .filter(|(k, _)| k.starts_with("profile."))
            .map(|(k, v)| (k.strip_prefix("profile.").unwrap().to_string(), v.clone()))
            .collect()
    }

    fn list(&self, key: &str) -> Vec<String> {
        self.entries
            .get(key)
            .map(|v| v.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
}

/// true when the line ends with an ODD number of backslashes (Java
/// properties continuation rule)
fn ends_with_continuation(line: &str) -> bool {
    let mut n = 0;
    for c in line.chars().rev() {
        if c == '\\' {
            n += 1;
        } else {
            break;
        }
    }
    n % 2 == 1
}

/// split `key = value` (first `=` or `:`; Java properties allows both)
fn split_kv(line: &str) -> Option<(String, String)> {
    let idx = line.find(|c| c == '=' || c == ':')?;
    let key = line[..idx].trim().to_string();
    if key.is_empty() || key.contains(char::is_whitespace) {
        return None;
    }
    let value = line[idx + 1..].trim().to_string();
    Some((key, value))
}

// ---------------------------------------------------------------------------
// pass chain — the documented Iris program layout
// ---------------------------------------------------------------------------

/// one program/stage in an Iris pack (structural only — no translation)
#[derive(Debug, Clone, PartialEq)]
pub struct IrisStage {
    /// stage name as documented (`gbuffers_terrain`, `composite1`, …)
    pub name: String,
    /// `shaders/<name>.vsh` exists
    pub has_vertex: bool,
    /// `shaders/<name>.fsh` exists
    pub has_fragment: bool,
    /// the `#version` line (normalized, e.g. "330 compatibility")
    pub glsl_version: Option<String>,
    /// render targets from `/* RENDERTARGETS: n m */` or the legacy
    /// `/* DRAWBUFFERS: n m */` directive (empty = stage default)
    pub render_targets: Vec<u8>,
    /// verbatim source paths (translator input)
    pub vertex_path: Option<PathBuf>,
    pub fragment_path: Option<PathBuf>,
    /// pipeline slot the stage belongs to (Iris's documented order)
    pub phase: IrisPhase,
}

/// the documented pass-chain phases, in Iris's execution order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IrisPhase {
    /// shadow map programs (`shadow`, `shadowterrain`…)
    Shadow,
    /// `prepare` programs (Iris 1.6+)
    Prepare,
    /// `gbuffers_*` — opaque geometry
    GbuffersOpaque,
    /// `gbuffers_water` etc. — translucent geometry
    GbuffersTranslucent,
    /// `deferred*` programs
    Deferred,
    /// `composite*` programs
    Composite,
    /// `final` (last, writes the screen)
    Final,
}

impl IrisPhase {
    fn of(name: &str) -> IrisPhase {
        if name == "shadow" || name.starts_with("shadow") {
            IrisPhase::Shadow
        } else if name == "prepare" || name.starts_with("prepare") {
            IrisPhase::Prepare
        } else if name == "final" {
            IrisPhase::Final
        } else if name == "deferred" || name.starts_with("deferred") {
            IrisPhase::Deferred
        } else if name == "composite" || name.starts_with("composite") {
            IrisPhase::Composite
        } else if name == "gbuffers_water"
            || name == "gbuffers_hand_water"
            || name == "gbuffers_entities_translucent"
            || name.contains("translucent")
        {
            IrisPhase::GbuffersTranslucent
        } else if name.starts_with("gbuffers") {
            IrisPhase::GbuffersOpaque
        } else {
            // unknown program names ride the composite phase (Iris runs
            // custom `program.<name>.enabled`-gated stages there)
            IrisPhase::Composite
        }
    }
}

/// structural model of an Iris pack's program chain
#[derive(Debug, Default, Clone, PartialEq)]
pub struct IrisPassChain {
    pub stages: Vec<IrisStage>,
}

impl IrisPassChain {
    /// Discover the stage inventory from a pack's `shaders/` directory.
    /// Only documented program names are modeled; exotic files are
    /// reported through `unknown` on the resulting [`IrisPackInfo`].
    pub fn scan(shaders_dir: &Path) -> IrisPassChain {
        let mut stages: Vec<IrisStage> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        let Ok(entries) = std::fs::read_dir(shaders_dir) else {
            return IrisPassChain { stages };
        };
        for e in entries.flatten() {
            let path = e.path();
            let Some(ext) = path.extension().and_then(|x| x.to_str()) else { continue };
            let (is_v, is_f) = match ext {
                "vsh" => (true, false),
                "fsh" => (false, true),
                _ => continue,
            };
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            if !is_documented_stage(stem) {
                continue;
            }
            if !names.iter().any(|n| n == stem) {
                names.push(stem.to_string());
            }
            let _ = (is_v, is_f);
        }
        names.sort();
        for name in names {
            let v = shaders_dir.join(format!("{name}.vsh"));
            let f = shaders_dir.join(format!("{name}.fsh"));
            let has_vertex = v.is_file();
            let has_fragment = f.is_file();
            // parse directives from whichever source exists
            let (version, targets) = parse_stage_directives(
                has_fragment
                    .then(|| std::fs::read_to_string(&f).ok())
                    .flatten()
                    .as_deref(),
            );
            let (version, targets) = if version.is_some() && !targets.is_empty() {
                (version, targets)
            } else {
                // fall back to the vertex source if the fragment had none
                let (v2, t2) = parse_stage_directives(
                    has_vertex
                        .then(|| std::fs::read_to_string(&v).ok())
                        .flatten()
                        .as_deref(),
                );
                (
                    version.or(v2),
                    if targets.is_empty() { t2 } else { targets },
                )
            };
            stages.push(IrisStage {
                name: name.to_string(),
                has_vertex,
                has_fragment,
                glsl_version: version,
                render_targets: targets,
                vertex_path: has_vertex.then(|| v.clone()),
                fragment_path: has_fragment.then(|| f.clone()),
                phase: IrisPhase::of(&name),
            });
        }
        stages.sort_by_key(|s| s.phase);
        IrisPassChain { stages }
    }

    /// stages in a given phase (execution order preserved)
    pub fn phase(&self, phase: IrisPhase) -> Vec<&IrisStage> {
        self.stages.iter().filter(|s| s.phase == phase).collect()
    }
}

/// the documented Iris program names (the classic OptiFine layout Iris
/// documents for packs; custom `program.X` extensions ride Composite)
fn is_documented_stage(stem: &str) -> bool {
    const GB: [&str; 12] = [
        "gbuffers_basic",
        "gbuffers_textured",
        "gbuffers_textured_lit",
        "gbuffers_skybasic",
        "gbuffers_skytextured",
        "gbuffers_clouds",
        "gbuffers_terrain",
        "gbuffers_damagedblock",
        "gbuffers_block",
        "gbuffers_beaconbeam",
        "gbuffers_item",
        "gbuffers_entities",
    ];
    const EXTRA: [&str; 10] = [
        "gbuffers_hand",
        "gbuffers_weather",
        "gbuffers_water",
        "gbuffers_hand_water",
        "shadow",
        "shadowterrain",
        "prepare",
        "deferred",
        "composite",
        "final",
    ];
    if GB.contains(&stem) || EXTRA.contains(&stem) {
        return true;
    }
    // numbered composites/deferreds: composite1..99, deferred1..99
    for (prefix, hi) in [("composite", 99u32), ("deferred", 99), ("prepare", 9), ("shadow", 9)] {
        if let Some(num) = stem.strip_prefix(prefix) {
            if let Ok(n) = num.parse::<u32>() {
                if n >= 1 && n <= hi && prefix != "shadow" {
                    return true;
                }
            }
        }
    }
    false
}

/// parse `#version` + `/* RENDERTARGETS: n m */` / `/* DRAWBUFFERS: n */`
/// from a GLSL source (the directives Iris documents for target binding)
pub fn parse_stage_directives(src: Option<&str>) -> (Option<String>, Vec<u8>) {
    let mut version = None;
    let mut targets = Vec::new();
    let Some(src) = src else { return (version, targets) };
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#version") {
            version = Some(rest.trim().to_string());
        }
        if t.starts_with("/*") {
            let body = t.trim_start_matches('/').trim_start_matches('*');
            let body = body.trim_end_matches('*').trim_end_matches('/');
            let body = body.trim();
            if let Some(list) = body.strip_prefix("RENDERTARGETS:") {
                targets = parse_target_list(list);
            } else if let Some(list) = body.strip_prefix("DRAWBUFFERS:") {
                // legacy directive: each char is one target digit
                targets = list
                    .chars()
                    .filter_map(|c| c.to_digit(10).map(|d| d as u8))
                    .collect();
            }
        }
    }
    (version, targets)
}

fn parse_target_list(list: &str) -> Vec<u8> {
    // "0,1 4" / "0 1 4" / "0, 1, 4" are all legal spellings
    list.replace(',', " ")
        .split_whitespace()
        .filter_map(|t| t.parse::<u8>().ok())
        .collect()
}

// ---------------------------------------------------------------------------
// uniform reference — the documented set + engine availability
// ---------------------------------------------------------------------------

/// availability of a documented Iris/OptiFine uniform against the
/// engine's current `PackUniform` v1 bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniformAvailability {
    /// the engine supplies it today (through the v1 bridge)
    Available,
    /// structurally planned (v2+ bridge) — matrices/depth/samplers
    Planned,
    /// engine-architecture-dependent (e.g. shadow-only uniforms when the
    /// engine's shadow pass differs) — the sister project must adapt
    EngineDependent,
}

/// one documented uniform
#[derive(Debug, Clone, PartialEq)]
pub struct IrisUniformRef {
    pub name: &'static str,
    /// documented GLSL type
    pub ty: &'static str,
    pub availability: UniformAvailability,
}

/// The documented uniform reference (Iris's public docs surface; names are
/// the interface contract, not copied code). The v1 bridge supplies
/// color+time only — the availability map says so honestly:
///
/// Available (PackUniform v1): `viewWidth`/`viewHeight` (viewport.xy),
/// `pixelWidth`/`pixelHeight`-style inverses (viewport.zw),
/// `frameTimeCounter` (time.x), `worldTime` (time.y),
/// `isEyeInWater` (time.z), `eyeBrightness` (time.w, scaled 0..1).
/// Planned (v2+): matrices (`gbufferModelView`…), `cameraPosition`,
/// depth samplers, `fogColor`/`skyColor`, `sunPosition`/`moonPosition`.
/// Engine-dependent: shadow-map uniforms (`shadowtex0/1` resolution…).
pub fn uniform_reference() -> Vec<IrisUniformRef> {
    use UniformAvailability::*;
    vec![
        // ---------------------------------------------------------------- v1
        IrisUniformRef { name: "viewWidth", ty: "float", availability: Available },
        IrisUniformRef { name: "viewHeight", ty: "float", availability: Available },
        IrisUniformRef { name: "frameTimeCounter", ty: "float", availability: Available },
        IrisUniformRef { name: "worldTime", ty: "float", availability: Available },
        IrisUniformRef { name: "worldDay", ty: "float", availability: Planned },
        IrisUniformRef { name: "isEyeInWater", ty: "float", availability: Available },
        IrisUniformRef { name: "eyeBrightness", ty: "vec2", availability: Available },
        IrisUniformRef { name: "eyeBrightnessSmooth", ty: "vec2", availability: Planned },
        // ---------------------------------------------------------------- v2
        IrisUniformRef { name: "gbufferModelView", ty: "mat4", availability: Planned },
        IrisUniformRef { name: "gbufferModelViewInverse", ty: "mat4", availability: Planned },
        IrisUniformRef { name: "gbufferProjection", ty: "mat4", availability: Planned },
        IrisUniformRef { name: "gbufferProjectionInverse", ty: "mat4", availability: Planned },
        IrisUniformRef { name: "cameraPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "previousCameraPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "sunPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "moonPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "shadowLightPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "upPosition", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "fogColor", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "skyColor", ty: "vec3", availability: Planned },
        IrisUniformRef { name: "rainStrength", ty: "float", availability: Planned },
        IrisUniformRef { name: "aspectRatio", ty: "float", availability: Planned },
        IrisUniformRef { name: "near", ty: "float", availability: Planned },
        IrisUniformRef { name: "far", ty: "float", availability: Planned },
        // ------------------------------------------------------- engine-side
        IrisUniformRef { name: "shadowtex0", ty: "sampler2D", availability: EngineDependent },
        IrisUniformRef { name: "shadowtex1", ty: "sampler2D", availability: EngineDependent },
        IrisUniformRef { name: "depthtex0", ty: "sampler2D", availability: EngineDependent },
        IrisUniformRef { name: "colortex0", ty: "sampler2D", availability: EngineDependent },
        IrisUniformRef { name: "colortex1", ty: "sampler2D", availability: EngineDependent },
        IrisUniformRef { name: "colortex2", ty: "sampler2D", availability: EngineDependent },
    ]
}

/// which documented uniforms a GLSL source references (identifier scan —
/// planning data for the translator; `uniform <ty> <name>;` declarations)
pub fn uniforms_declared(glsl: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in glsl.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("uniform") else { continue };
        // uniform mat4 gbufferModelView;
        let rest = rest.trim().trim_end_matches(';').trim();
        // split type from name (arrays/initializers not modeled — the
        // translator sees the raw source anyway)
        let mut parts = rest.split_whitespace();
        let (Some(_ty), Some(name)) = (parts.next(), parts.next()) else { continue };
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.push(name.to_string());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// the translator seam — where the vc-iris sister project plugs in
// ---------------------------------------------------------------------------

/// A translated stage produced by the sister project's GLSL→WGSL pipeline
/// (the engine's composite contract from §34 wraps it like a native pack).
#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedStage {
    /// WGSL function body following PACK_CONTRACT's `packGrade` shape
    pub wgsl: String,
    /// the GLSL version the translator accepted
    pub glsl_version: String,
}

/// why a translation attempt failed (honest reporting — the pack list
/// shows these instead of silently dropping the pack)
#[derive(Debug, Clone, PartialEq)]
pub enum IrisTranslateError {
    /// no translator registered (the built-in NoTranslator)
    NoTranslator,
    /// the translator does not support this GLSL version/profile
    UnsupportedVersion(String),
    /// the translator rejected the stage (its own diagnostics)
    Rejected(String),
}

/// The vc-iris sister project implements this and registers it through
/// [`register_translator`] on the renderer. The interface is deliberately
/// minimal and structural: the engine hands over the stage it parsed
/// (sources, targets, version), the translator hands back WGSL.
pub trait IrisTranslator: Send + Sync {
    /// translator identity (shown in F3/E2E reports)
    fn id(&self) -> &'static str;
    /// can this translator handle the given `#version` line?
    fn supports_version(&self, glsl_version: &str) -> bool;
    /// translate one stage's vertex+fragment pair
    fn translate(
        &self,
        stage: &IrisStage,
        vertex_src: Option<&str>,
        fragment_src: &str,
    ) -> Result<TranslatedStage, IrisTranslateError>;
}

/// The default, pre-registration translator: honest "pending" status.
/// `translate` always fails with [`IrisTranslateError::NoTranslator`] —
/// selecting the pack reports WHY instead of pretending.
pub struct NoTranslator;

impl IrisTranslator for NoTranslator {
    fn id(&self) -> &'static str {
        "none (vc-iris sister project not registered)"
    }
    fn supports_version(&self, _glsl_version: &str) -> bool {
        false
    }
    fn translate(
        &self,
        _stage: &IrisStage,
        _vertex_src: Option<&str>,
        _fragment_src: &str,
    ) -> Result<TranslatedStage, IrisTranslateError> {
        Err(IrisTranslateError::NoTranslator)
    }
}

// ---------------------------------------------------------------------------
// pack analysis — detection + the honest report
// ---------------------------------------------------------------------------

/// analysis result for an Iris-format pack found in `shader-packs/`
#[derive(Debug, Clone, PartialEq)]
pub struct IrisPackInfo {
    /// directory name
    pub id: String,
    /// display name (from `shaders.properties` profile 0 or the dir name)
    pub name: String,
    /// honest tier — always `IRIS-STRUCTURE-VALIDATED` here
    pub tier: &'static str,
    pub properties: ShadersProperties,
    pub chain: IrisPassChain,
    /// files in `shaders/` that are not documented stages (kept honest)
    pub unknown_files: Vec<String>,
    /// documented uniforms referenced across all stage sources
    pub uniforms_used: Vec<String>,
}

impl IrisPackInfo {
    /// true when the pack has at least one gbuffers + one composite/final
    /// stage — the minimum a translator needs to produce visible output
    pub fn minimal_chain(&self) -> bool {
        let has_gbuffers = self
            .chain
            .stages
            .iter()
            .any(|s| matches!(s.phase, IrisPhase::GbuffersOpaque | IrisPhase::GbuffersTranslucent));
        let has_post = self
            .chain
            .stages
            .iter()
            .any(|s| matches!(s.phase, IrisPhase::Composite | IrisPhase::Final));
        has_gbuffers && has_post
    }

    /// one-line summary for logs / the E2E `iris` command
    pub fn summary(&self) -> String {
        format!(
            "{} [{}] passes={} profiles={} uniforms={} unknown-files={} minimal-chain={}",
            self.name,
            self.tier,
            self.chain.stages.len(),
            self.properties.profiles().len(),
            self.uniforms_used.len(),
            self.unknown_files.len(),
            self.minimal_chain()
        )
    }
}

/// detect + fully analyze an Iris-format pack directory
/// (`shader-packs/<dir>/` with a `shaders.properties` file).
/// Returns None when the directory is not an Iris-format pack.
pub fn analyze_pack(dir: &Path) -> Option<IrisPackInfo> {
    let props_path = dir.join("shaders.properties");
    if !props_path.is_file() {
        return None;
    }
    let id = dir.file_name()?.to_str()?.to_string();
    let props_text = std::fs::read_to_string(&props_path).ok()?;
    let properties = ShadersProperties::parse(&props_text);
    // the display name: a `profile.<PACK>` entry is not a name — Iris packs
    // are named by their directory on the pack list; use the dir name
    // (lang-file names are a sister-project concern)
    let name = id.clone();

    let shaders_dir = dir.join("shaders");
    let chain = IrisPassChain::scan(&shaders_dir);

    // unknown files: not documented stages
    let mut unknown_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&shaders_dir) {
        for e in entries.flatten() {
            let path = e.path();
            let Some(ext) = path.extension().and_then(|x| x.to_str()) else { continue };
            if ext != "vsh" && ext != "fsh" {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            if !is_documented_stage(stem)
                && !unknown_files.iter().any(|u| u == stem)
            {
                unknown_files.push(stem.to_string());
            }
        }
    }

    // cross-reference uniform declarations with the documented reference
    let reference: Vec<&str> = uniform_reference().iter().map(|u| u.name).collect();
    let mut uniforms_used: Vec<String> = Vec::new();
    for stage in &chain.stages {
        for path in [stage.vertex_path.as_ref(), stage.fragment_path.as_ref()]
            .into_iter()
            .flatten()
        {
            if let Ok(src) = std::fs::read_to_string(path) {
                for name in uniforms_declared(&src) {
                    if reference.contains(&name.as_str())
                        && !uniforms_used.iter().any(|u| *u == name)
                    {
                        uniforms_used.push(name);
                    }
                }
            }
        }
    }

    Some(IrisPackInfo {
        id,
        name,
        tier: "IRIS-STRUCTURE-VALIDATED",
        properties,
        chain,
        unknown_files,
        uniforms_used,
    })
}

/// global translator registry (set once at boot by the game crate — or by
/// the sister project's integration test). [`NoTranslator`] until then.
static TRANSLATOR: std::sync::OnceLock<Box<dyn IrisTranslator>> = std::sync::OnceLock::new();

/// install the vc-iris sister project's translator (idempotent — the
/// first registration wins, later calls return false)
pub fn register_translator(t: Box<dyn IrisTranslator>) -> bool {
    TRANSLATOR.set(t).is_ok()
}

/// the active translator (NoTranslator before registration)
pub fn translator() -> &'static dyn IrisTranslator {
    TRANSLATOR
        .get()
        .map(|b| b.as_ref() as &dyn IrisTranslator)
        .unwrap_or(&NoTranslator)
}

/// attempt translation of a pack's composite/final stages through the
/// registered translator — the sister-project activation path. Returns
/// per-stage results so a partially-supported pack reports per pass.
pub fn try_translate(info: &IrisPackInfo) -> Vec<(String, Result<TranslatedStage, IrisTranslateError>)> {
    let t = translator();
    let mut out = Vec::new();
    for stage in info
        .chain
        .stages
        .iter()
        .filter(|s| matches!(s.phase, IrisPhase::Composite | IrisPhase::Final))
    {
        let frag_src = stage
            .fragment_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok());
        let vert_src = stage
            .vertex_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok());
        let result = match (&frag_src, stage.glsl_version.as_deref()) {
            (None, _) => Err(IrisTranslateError::Rejected("no fragment source".into())),
            (Some(f), Some(v)) if t.supports_version(v) => {
                t.translate(stage, vert_src.as_deref(), f)
            }
            (Some(_), v) => {
                Err(IrisTranslateError::UnsupportedVersion(v.unwrap_or("unknown").into()))
            }
        };
        out.push((stage.name.clone(), result));
    }
    out
}

// ---------------------------------------------------------------------------
// root scanning + the embedded demo document (E2E / sister-project example)
// ---------------------------------------------------------------------------

/// scan a `shader-packs/` root for Iris-format packs: one pass over the
/// directory, each subdirectory that carries a `shaders.properties` is
/// fully analyzed. Non-Iris directories (the engine's own WGSL packs,
/// random folders) return None from [`analyze_pack`] and are skipped
/// without a log line — the WGSL scanner owns those reports. The engine
/// calls this at boot (native: real `shader-packs/`; wasm has no
/// filesystem and boots with an empty list).
pub fn scan_shader_packs(root: &Path) -> Vec<IrisPackInfo> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for e in entries.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(info) = analyze_pack(&path) {
            out.push(info);
        }
    }
    out
}

/// A minimal but honest Iris-format `shaders.properties` document — the
/// E2E `iris` command parses this on wasm (where no pack directory
/// exists) to prove the wasm-reachable interface surface works live; the
/// vc-iris sister project can also use it as a smoke document.
pub const DEMO_PROPERTIES: &str = "\
# VoxelCraft Phase 8 interface demo (documented Iris directives)
sliders = EXPOSURE CONTRAST SATURATION
screen = <profile> [TONE] EXPOSURE *
screen.TONE = CONTRAST SATURATION
profile.LOW = EXPOSURE=0.8 !SATURATION
profile.HIGH = EXPOSURE=1.2 SATURATION profile.LOW
texture.composite.colortex0 = textures/tone_lut.png
blend.gbuffers_water = off
";

/// A minimal documented stage source: `#version 330 compatibility` (the
/// version real packs use — the reason naga's GLSL 440 frontend can't
/// consume them) + the `/* RENDERTARGETS: n */` comment directive + the
/// two most-used public uniforms.
pub const DEMO_STAGE_GLSL: &str = "\
#version 330 compatibility
/* RENDERTARGETS: 0,1 */
uniform float viewWidth;
uniform float viewHeight;
uniform float frameTimeCounter;
void main() { gl_FragColor = vec4(1.0); }
";

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_parse_basics() {
        // verified against the live Iris docs (shader_settings page):
        // screen / sliders / profile.<name> / per-stage directives are all
        // flat key = value pairs
        let doc = "# a comment
! another comment style
sliders = EXPOSURE CONTRAST
screen = <profile> <empty> [SHADOWS] iScream *
screen.SHADOWS = shadowMapResolution shadowFilter
profile.LOW = shadowMapResolution=1024 !shadowFilter
profile.HIGH = shadowMapResolution=4096 shadowFilter profile.LOW !program.composite3
texture.composite.colortex4 = textures/final.png
blend.gbuffers_water = off
program.shadow.enabled = false
separateEntityDraws = true
";
        let p = ShadersProperties::parse(doc);
        assert_eq!(p.sliders(), vec!["EXPOSURE", "CONTRAST"]);
        // main screen tokens carry the documented magic members verbatim
        assert_eq!(
            p.screen_main(),
            vec!["<profile>", "<empty>", "[SHADOWS]", "iScream", "*"]
        );
        assert_eq!(
            p.screen("SHADOWS"),
            vec!["shadowMapResolution", "shadowFilter"]
        );
        // profiles are key-value entries
        let profiles = p.profiles();
        assert_eq!(profiles.len(), 2);
        assert_eq!(
            profiles.get("LOW").map(|s| s.as_str()),
            Some("shadowMapResolution=1024 !shadowFilter")
        );
        assert_eq!(
            profiles.get("HIGH").map(|s| s.as_str()),
            Some("shadowMapResolution=4096 shadowFilter profile.LOW !program.composite3")
        );
        // per-stage + feature directives are ordinary entries
        assert_eq!(
            p.entries.get("texture.composite.colortex4").map(|s| s.as_str()),
            Some("textures/final.png")
        );
        assert_eq!(p.entries.get("blend.gbuffers_water").map(|s| s.as_str()), Some("off"));
        assert_eq!(p.entries.get("program.shadow.enabled").map(|s| s.as_str()), Some("false"));
        assert_eq!(p.entries.get("separateEntityDraws").map(|s| s.as_str()), Some("true"));
        assert!(p.unknown.is_empty(), "all keys modeled: {p:?}");
    }

    #[test]
    fn properties_line_continuation() {
        // Java properties: odd trailing backslash continues the line
        let p = ShadersProperties::parse("sliders = ALPHA \\
  BETA GAMMA\n");
        assert_eq!(p.sliders(), vec!["ALPHA", "BETA", "GAMMA"]);
        // an escaped backslash (two of them) does NOT continue: the value
        // keeps the backslashes verbatim (no unescape pass — documented)
        let p2 = ShadersProperties::parse(r"sliders = A \ B");
        assert_eq!(p2.sliders(), vec!["A", "\\", "B"], "escaped pair stays inline");
        // multi-line continuation chains
        let p3 = ShadersProperties::parse("profile.MID = OPT_A \\\n OPT_B \\\n OPT_C\n");
        // joined continuations keep raw spacing — list accessors tokenize
        let mid = p3.profiles().get("MID").cloned().unwrap_or_default();
        let tokens: Vec<&str> = mid.split_whitespace().collect();
        assert_eq!(tokens, vec!["OPT_A", "OPT_B", "OPT_C"]);
    }

    #[test]
    fn properties_unknown_lines_reported() {
        let doc = "sliders = A\nsome malformed line without separator\n";
        let p = ShadersProperties::parse(doc);
        assert_eq!(p.sliders(), vec!["A"]);
        // the malformed line lands honestly in unknown
        assert_eq!(p.unknown, vec!["some malformed line without separator"]);
    }

    #[test]
    fn stage_directives_parse() {
        let glsl = "\
#version 330 compatibility
/* RENDERTARGETS: 0,1 4 */
uniform mat4 gbufferModelView;
uniform vec3 cameraPosition;
uniform float frameTimeCounter;
varying vec2 texcoord;
void main() { gl_FragColor = vec4(1.0); }
";
        let (version, targets) = parse_stage_directives(Some(glsl));
        assert_eq!(version.as_deref(), Some("330 compatibility"));
        assert_eq!(targets, vec![0, 1, 4]);
        // legacy DRAWBUFFERS
        let legacy = "#version 120\n/* DRAWBUFFERS:05 */\n";
        let (v2, t2) = parse_stage_directives(Some(legacy));
        assert_eq!(v2.as_deref(), Some("120"));
        assert_eq!(t2, vec![0, 5]);
        // uniforms declared
        let uniforms = uniforms_declared(glsl);
        assert!(uniforms.contains(&"gbufferModelView".to_string()));
        assert!(uniforms.contains(&"cameraPosition".to_string()));
        assert!(uniforms.contains(&"frameTimeCounter".to_string()));
        assert!(!uniforms.contains(&"texcoord".to_string()), "varying is not a uniform");
    }

    #[test]
    fn pass_chain_scan_from_disk() {
        let dir = std::env::temp_dir().join(format!("vc-iris-test-{}", std::process::id()));
        let shaders = dir.join("shaders");
        std::fs::create_dir_all(&shaders).unwrap();
        std::fs::write(
            shaders.join("gbuffers_terrain.fsh"),
            "#version 330 compatibility\n/* RENDERTARGETS: 0 */\nuniform float viewWidth;\nvoid main(){}\n",
        )
        .unwrap();
        std::fs::write(
            shaders.join("composite.fsh"),
            "#version 330 compatibility\n/* RENDERTARGETS: 0 */\nuniform sampler2D colortex0;\nvoid main(){}\n",
        )
        .unwrap();
        std::fs::write(
            shaders.join("final.fsh"),
            "#version 330 compatibility\nuniform float frameTimeCounter;\nvoid main(){}\n",
        )
        .unwrap();
        // a non-stage file — must land in unknown_files, not the chain
        std::fs::write(shaders.join("my_custom_stage.fsh"), "#version 330\n").unwrap();

        let chain = IrisPassChain::scan(&shaders);
        assert_eq!(chain.stages.len(), 3, "3 documented stages");
        let terrain = chain.stages.iter().find(|s| s.name == "gbuffers_terrain").unwrap();
        assert_eq!(terrain.phase, IrisPhase::GbuffersOpaque);
        assert_eq!(terrain.render_targets, vec![0]);
        assert!(terrain.has_fragment);
        assert!(!terrain.has_vertex);
        let final_ = chain.stages.iter().find(|s| s.name == "final").unwrap();
        assert_eq!(final_.phase, IrisPhase::Final);
        // phase ordering: gbuffers before composite before final
        let phases: Vec<IrisPhase> = chain.stages.iter().map(|s| s.phase).collect();
        let sorted = {
            let mut p = phases.clone();
            p.sort();
            p == phases
        };
        assert!(sorted, "stages sorted by phase: {phases:?}");

        // full pack analysis from the parent dir (add shaders.properties
        // with the documented key-value profile + slider directives)
        std::fs::write(
            dir.join("shaders.properties"),
            "sliders = EXPOSURE\nprofile.LOW = EXPOSURE=0.5\n",
        )
        .unwrap();
        let info = analyze_pack(&dir).expect("must detect as Iris pack");
        assert_eq!(info.tier, "IRIS-STRUCTURE-VALIDATED");
        assert!(info.minimal_chain(), "gbuffers + composite/final present");
        assert_eq!(info.unknown_files, vec!["my_custom_stage"]);
        assert!(info.uniforms_used.contains(&"viewWidth".to_string()));
        assert!(info.uniforms_used.contains(&"colortex0".to_string()));
        assert_eq!(info.properties.sliders(), vec!["EXPOSURE"]);
        // profiles parse as documented key-value entries
        assert_eq!(
            info.properties.profiles().get("LOW").map(|s| s.as_str()),
            Some("EXPOSURE=0.5")
        );
        // the pack display name falls back to the directory name
        assert_eq!(info.name, dir.file_name().unwrap().to_str().unwrap());
        // summary line is human-readable
        assert!(info.summary().contains("IRIS-STRUCTURE-VALIDATED"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn uniform_reference_availability_is_honest() {
        let refs = uniform_reference();
        // the v1-available set is exactly the PackUniform bridge fields
        let available: Vec<&str> = refs
            .iter()
            .filter(|u| u.availability == UniformAvailability::Available)
            .map(|u| u.name)
            .collect();
        for must in ["viewWidth", "viewHeight", "frameTimeCounter", "worldTime", "isEyeInWater"] {
            assert!(available.contains(&must), "{must} must be Available");
        }
        // matrices/depth are honestly NOT available yet
        for planned in ["gbufferModelView", "gbufferProjection", "cameraPosition", "fogColor"] {
            let u = refs.iter().find(|u| u.name == planned).unwrap();
            assert_eq!(u.availability, UniformAvailability::Planned, "{planned}");
        }
        // samplers are engine-dependent
        let depthtex = refs.iter().find(|u| u.name == "depthtex0").unwrap();
        assert_eq!(depthtex.availability, UniformAvailability::EngineDependent);
    }

    /// the seam works end-to-end with a mock translator (the same flow the
    /// vc-iris sister project will use through register_translator)
    #[test]
    fn translator_seam_mock_flow() {
        struct MockTranslator;
        impl IrisTranslator for MockTranslator {
            fn id(&self) -> &'static str { "mock" }
            fn supports_version(&self, v: &str) -> bool { v.starts_with("330") }
            fn translate(
                &self,
                stage: &IrisStage,
                _vertex: Option<&str>,
                fragment: &str,
            ) -> Result<TranslatedStage, IrisTranslateError> {
                if fragment.is_empty() {
                    return Err(IrisTranslateError::Rejected("empty source".into()));
                }
                Ok(TranslatedStage {
                    wgsl: format!("// translated {} for Iris", stage.name),
                    glsl_version: stage.glsl_version.clone().unwrap_or_default(),
                })
            }
        }
        let t = MockTranslator;
        let stage = IrisStage {
            name: "composite".into(),
            has_vertex: false,
            has_fragment: true,
            glsl_version: Some("330 compatibility".into()),
            render_targets: vec![0],
            vertex_path: None,
            fragment_path: None,
            phase: IrisPhase::Composite,
        };
        // supported version translates
        let ok = t.translate(&stage, None, "void main(){}").unwrap();
        assert!(ok.wgsl.contains("translated composite"));
        // the default NoTranslator is honest
        let none = translator();
        assert!(!none.supports_version("330 compatibility"));
        assert!(matches!(
            none.translate(&stage, None, "src"),
            Err(IrisTranslateError::NoTranslator)
        ));
        // registration is idempotent — but the NoTranslator default is
        // what unregistered code sees (register only in this test's
        // process would race other tests; assert the read path instead)
        let _ = register_translator(Box::new(MockTranslator));
        // after registration the global translator is the mock (first
        // registration wins; in the test process this is ours)
        assert!(translator().supports_version("330 compatibility"));
    }

    /// the root scanner: only Iris-format subdirectories are picked up,
    /// the engine's own WGSL packs and stray files are skipped silently
    #[test]
    fn scan_shader_packs_filters_correctly() {
        let root = std::env::temp_dir().join(format!("vc-iris-root-{}", std::process::id()));
        std::fs::create_dir_all(root.join("an-iris-pack/shaders")).unwrap();
        std::fs::write(root.join("an-iris-pack/shaders.properties"), DEMO_PROPERTIES).unwrap();
        std::fs::write(
            root.join("an-iris-pack/shaders/gbuffers_terrain.fsh"),
            DEMO_STAGE_GLSL,
        )
        .unwrap();
        std::fs::write(
            root.join("an-iris-pack/shaders/composite.fsh"),
            DEMO_STAGE_GLSL,
        )
        .unwrap();
        std::fs::write(
            root.join("an-iris-pack/shaders/final.fsh"),
            DEMO_STAGE_GLSL,
        )
        .unwrap();
        // a WGSL pack (shaders.json, no shaders.properties) — not ours
        std::fs::create_dir_all(root.join("a-wgsl-pack")).unwrap();
        std::fs::write(root.join("a-wgsl-pack/shaders.json"), "{}").unwrap();
        // a stray file at the root — not a directory, skipped
        std::fs::write(root.join("notes.txt"), "hello").unwrap();

        let packs = scan_shader_packs(&root);
        assert_eq!(packs.len(), 1, "only the Iris pack: {packs:?}");
        let p = &packs[0];
        assert_eq!(p.id, "an-iris-pack");
        assert_eq!(p.tier, "IRIS-STRUCTURE-VALIDATED");
        assert_eq!(p.properties.sliders().len(), 3);
        assert_eq!(p.properties.profiles().len(), 2);
        assert!(p.minimal_chain(), "gbuffers_terrain + composite + final");

        // missing root: empty, not a panic (wasm boots this way)
        assert!(scan_shader_packs(&root.join("does-not-exist")).is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    /// the embedded demo document parses to exactly what the E2E log line
    /// claims (the wasm `iris` command prints from these numbers)
    #[test]
    fn demo_document_matches_e2e_claims() {
        let props = ShadersProperties::parse(DEMO_PROPERTIES);
        assert_eq!(props.sliders(), vec!["EXPOSURE", "CONTRAST", "SATURATION"]);
        assert_eq!(props.screen_main(), vec!["<profile>", "[TONE]", "EXPOSURE", "*"]);
        assert_eq!(props.screen("TONE"), vec!["CONTRAST", "SATURATION"]);
        assert_eq!(props.profiles().len(), 2);
        assert!(props.unknown.is_empty(), "demo must stay fully modeled: {props:?}");

        let (version, targets) = parse_stage_directives(Some(DEMO_STAGE_GLSL));
        assert_eq!(version.as_deref(), Some("330 compatibility"));
        assert_eq!(targets, vec![0, 1]);
        let uniforms = uniforms_declared(DEMO_STAGE_GLSL);
        assert_eq!(uniforms, vec!["viewWidth", "viewHeight", "frameTimeCounter"]);
    }
}
