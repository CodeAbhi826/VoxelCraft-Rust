//! Blockstate + model JSON layer (Master Spec §5.2, Phase 1).
//!
//! Parses the 1.16.5 resource-pack JSON structures (`blockstates/*.json`,
//! `models/**/*.json`), resolves model inheritance and texture variables,
//! bakes element rotations, and precomputes a **per-blockstate model
//! dispatch** so the mesher never parses JSON at mesh time ("parse once,
//! validate, canonicalize and cache").
//!
//! VERIFIED facts (minecraft.wiki, 2026-09): pack_format 6 = 1.16.2–1.16.5;
//! variant keys are comma-separated `property=value` pairs ("" for a
//! single-variant block); variant configs rotate the whole model in 90°
//! steps on x/y with `uvlock` and `weight`; multipart cases apply when their
//! `when` conditions match (with OR/AND combinators); elements are cuboids
//! `from`/`to` in −16..32; element rotation is a single axis with ±45°/22.5°
//! steps; faces carry uv [x1,y1,x2,y2] in 0..16 texture units (y-down),
//! auto-generated from element position when absent; `cullface` names the
//! neighbor direction that suppresses the face; `tintindex` −1 = no tint.
//!
//! ENGINEERING RECOMMENDATIONS (not vanilla-verified, §0.2): the exact
//! auto-UV orientation per face and the per-face texture mirroring — our
//! builtin pack therefore specifies UVs explicitly wherever orientation
//! matters.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

// ---------------------------------------------------------------------------
// raw JSON shapes
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct BlockstateJson {
    #[serde(default)]
    pub variants: Option<HashMap<String, VariantSpec>>,
    #[serde(default)]
    pub multipart: Option<Vec<MultipartCase>>,
}

/// a variant's value: one model config or an array (random weighted)
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum VariantSpec {
    One(ModeledVariant),
    Many(Vec<ModeledVariant>),
}

impl VariantSpec {
    pub fn list(&self) -> Vec<ModeledVariant> {
        match self {
            VariantSpec::One(v) => vec![v.clone()],
            VariantSpec::Many(v) => v.clone(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModeledVariant {
    pub model: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub uvlock: bool,
    #[serde(default = "default_weight")]
    pub weight: i32,
}

fn default_weight() -> i32 {
    1
}

#[derive(Deserialize, Debug)]
pub struct MultipartCase {
    /// kept raw: {"north":"true"} | {"facing":["north","east"]} | {"OR":[...]}
    #[serde(default)]
    pub when: Option<serde_json::Value>,
    pub apply: VariantSpec,
}

#[derive(Deserialize, Debug)]
pub struct ModelJson {
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub textures: Option<HashMap<String, String>>,
    #[serde(default = "default_true")]
    pub ambientocclusion: bool,
    #[serde(default)]
    pub elements: Option<Vec<ElementJson>>,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Clone, Debug)]
pub struct ElementJson {
    pub from: [f32; 3],
    pub to: [f32; 3],
    #[serde(default)]
    pub rotation: Option<RotationJson>,
    #[serde(default = "default_true")]
    pub shade: bool,
    #[serde(default)]
    pub faces: HashMap<FaceDir, FaceJson>,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FaceDir {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl FaceDir {
    /// outward unit vector (model space == world axes)
    #[inline]
    pub fn normal(self) -> [f32; 3] {
        match self {
            FaceDir::Down => [0.0, -1.0, 0.0],
            FaceDir::Up => [0.0, 1.0, 0.0],
            FaceDir::North => [0.0, 0.0, -1.0],
            FaceDir::South => [0.0, 0.0, 1.0],
            FaceDir::West => [-1.0, 0.0, 0.0],
            FaceDir::East => [1.0, 0.0, 0.0],
        }
    }

    /// VC-16 normal index (0=+X 1=−X 2=+Y 3=−Y 4=+Z 5=−Z)
    #[inline]
    pub fn normal_index(self) -> u32 {
        match self {
            FaceDir::East => 0,
            FaceDir::West => 1,
            FaceDir::Up => 2,
            FaceDir::Down => 3,
            FaceDir::South => 4,
            FaceDir::North => 5,
        }
    }

    /// rotate this direction by the blockstate x/y rotation
    fn rotated(self, x_rot: i32, y_rot: i32) -> FaceDir {
        let mut d = self;
        // y rotation: N→E→S→W (matches rot_y_90 point mapping)
        for _ in 0..((y_rot / 90).rem_euclid(4)) {
            d = match d {
                FaceDir::North => FaceDir::East,
                FaceDir::East => FaceDir::South,
                FaceDir::South => FaceDir::West,
                FaceDir::West => FaceDir::North,
                v => v,
            };
        }
        // x rotation 180: Up↔Down, North↔South
        if (x_rot / 90).rem_euclid(4) == 2 {
            d = match d {
                FaceDir::Up => FaceDir::Down,
                FaceDir::Down => FaceDir::Up,
                FaceDir::North => FaceDir::South,
                FaceDir::South => FaceDir::North,
                v => v,
            };
        }
        d
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct RotationJson {
    #[serde(default)]
    pub origin: Option<[f32; 3]>,
    pub axis: String,
    pub angle: f32,
    #[serde(default)]
    pub rescale: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FaceJson {
    #[serde(default)]
    pub uv: Option<[f32; 4]>,
    pub texture: String,
    #[serde(default)]
    pub cullface: Option<FaceDir>,
    /// texture rotation on this face: 0/90/180/270 (permutes uv corners)
    #[serde(default)]
    pub rotation: Option<i32>,
    #[serde(default = "no_tint")]
    pub tintindex: i32,
}

fn no_tint() -> i32 {
    -1
}

// ---------------------------------------------------------------------------
// compiled (canonicalized) representation
// ---------------------------------------------------------------------------

/// A face ready for the mesher: 4 pre-rotated corners in model units
/// (0..16, y-up), uv rect in texture units (y-down like the JSON), the
/// resolved texture resource location, cullface direction, tint, shade.
#[derive(Clone, Debug)]
pub struct CompiledFace {
    pub dir: FaceDir,
    /// CCW when viewed from outside (front-face rule), after all rotations
    pub verts: [[f32; 3]; 4],
    /// corner uv pairs, aligned with `verts` (uv in 0..1 of the texture)
    pub uvs: [[f32; 2]; 4],
    pub texture: String,
    pub cullface: Option<FaceDir>,
    pub tintindex: i32,
    pub shade: bool,
}

#[derive(Clone, Debug)]
pub struct CompiledElement {
    pub faces: Vec<CompiledFace>,
}

#[derive(Clone, Debug)]
pub struct CompiledModel {
    pub ambientocclusion: bool,
    pub elements: Vec<CompiledElement>,
    /// source location (debug/F3)
    pub loc: String,
}

/// one weighted application of a compiled model for a blockstate
/// (x/y rotations already applied at compile time — zero JSON work per mesh)
#[derive(Clone)]
pub struct AppliedModel {
    pub model: Arc<CompiledModel>,
    pub weight: u32,
}

/// A per-state model CHOICE: one of `alts` is picked (weighted, hashed by
/// world position). Variant random-model arrays become one choice with N
/// alternatives; multipart cases become N independent choices (all apply).
#[derive(Clone)]
pub struct ModelChoice {
    pub alts: Vec<AppliedModel>,
}

/// The global model registry: state id → per-state model choices.
/// Built once at boot (before any mesh job), then immutable — safe to read
/// from rayon workers and the browser's inline job loop.
pub struct ModelSet {
    pub by_state: HashMap<u16, Vec<ModelChoice>>,
    /// resolved texture location → atlas tile index (filled by atlas merge)
    pub tiles: HashMap<String, u16>,
}

static MODELS: OnceLock<Arc<ModelSet>> = OnceLock::new();

/// global access for the mesher (None until boot compiles the pack)
pub fn models() -> Option<&'static Arc<ModelSet>> {
    MODELS.get()
}

/// install the compiled set (call once, before mesh jobs start)
pub fn install(set: ModelSet) {
    let _ = MODELS.set(Arc::new(set));
}

/// test helper: swap in a set (leaks the previous Arc intentionally —
/// tests build fresh registries per case; also used by vc-mesh's tests,
/// so it is compiled unconditionally (tiny test hook, public API))
pub fn install_for_tests(set: ModelSet) {
    MODELS.set(Arc::new(set)).ok(); // ignore double-install
}

// ---------------------------------------------------------------------------
// model resolution (parent chain + texture vars + rotation baking)
// ---------------------------------------------------------------------------

const PARENT_DEPTH_CAP: usize = 32;

/// Resolve a model by location (e.g. "minecraft:block/oak_slab"), walking the
/// parent chain. `read` fetches raw bytes for a location minus namespace
/// normalization (we keep the `minecraft:` prefix in loc keys).
pub fn resolve_model(
    loc: &str,
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<CompiledModel, String> {
    // walk the chain: child elements override parent elements; child
    // texture vars override parent vars (vanilla semantics)
    let mut chain: Vec<ModelJson> = Vec::new();
    let mut cur = loc.to_string();
    for _ in 0..PARENT_DEPTH_CAP {
        let bytes = read(&model_path(&cur))
            .ok_or_else(|| format!("model not found: {cur}"))?;
        let json: ModelJson = serde_json::from_slice(&bytes)
            .map_err(|e| format!("model {cur}: bad JSON: {e}"))?;
        let parent = json.parent.clone();
        chain.push(json);
        match parent {
            Some(p) => {
                cur = normalize_loc(&p);
            }
            None => break,
        }
    }
    // texture resolution: parents first, child last → child overrides
    // parent per key (vanilla rule)
    let mut textures: HashMap<String, String> = HashMap::new();
    let mut ambientocclusion = true;
    for json in chain.iter().rev() {
        if let Some(t) = &json.textures {
            for (k, v) in t {
                textures.insert(k.clone(), v.clone());
            }
        }
        if !json.ambientocclusion {
            ambientocclusion = false;
        }
    }
    // elements: the most-derived (first in chain order) non-None set wins
    let elements: Vec<ElementJson> = match chain.iter().find_map(|j| j.elements.clone()) {
        Some(e) => e,
        None => Vec::new(),
    };

    // resolve texture variables (#ref chains, depth cap)
    let resolve_var = |name: &str| -> String {
        let mut key = name.trim_start_matches('#').to_string();
        for _ in 0..8 {
            match textures.get(&key) {
                Some(v) if v.starts_with('#') => {
                    key = v.trim_start_matches('#').to_string();
                }
                Some(v) => return v.clone(),
                None => return format!("minecraft:missing/{name}"),
            }
        }
        format!("minecraft:missing/{name}")
    };

    let mut compiled_elements = Vec::with_capacity(elements.len());
    for el in elements {
        let mut faces = Vec::with_capacity(el.faces.len());
        for (dir, face) in &el.faces {
            let tex_loc = if face.texture.starts_with('#') {
                normalize_loc(&resolve_var(&face.texture))
            } else {
                normalize_loc(&face.texture)
            };
            let (verts, uvs) = compile_face(
                el.from,
                el.to,
                *dir,
                face.uv,
                face.rotation.unwrap_or(0),
                el.rotation.as_ref(),
            );
            faces.push(CompiledFace {
                dir: *dir,
                verts,
                uvs,
                texture: tex_loc,
                cullface: face.cullface,
                tintindex: face.tintindex,
                shade: el.shade,
            });
        }
        compiled_elements.push(CompiledElement { faces });
    }

    Ok(CompiledModel {
        ambientocclusion,
        elements: compiled_elements,
        loc: loc.to_string(),
    })
}

/// normalize "block/oak_slab" → "minecraft:block/oak_slab"
/// (already-namespaced locations pass through)
pub fn normalize_loc(loc: &str) -> String {
    if loc.contains(':') {
        loc.to_string()
    } else {
        format!("minecraft:{loc}")
    }
}

/// Build the 4 face corners (CCW from outside) + uv pairs for one face of a
/// cuboid, applying the element rotation and the face texture rotation.
fn compile_face(
    from: [f32; 3],
    to: [f32; 3],
    dir: FaceDir,
    uv: Option<[f32; 4]>,
    face_rot: i32,
    rot: Option<&RotationJson>,
) -> ([[f32; 3]; 4], [[f32; 2]; 4]) {
    let (x1, y1, z1) = (from[0], from[1], from[2]);
    let (x2, y2, z2) = (to[0], to[1], to[2]);

    // corner order: CCW viewed from outside; uv[0] of each pair maps to the
    // corner's (u,v). Side faces: v = 16 − y (texture y-down = block top).
    // Face texture rotation (0/90/180/270) permutes which corner gets which
    // uv — implemented as a corner permutation at the end.
    let (base_verts, base_uv): ([[f32; 3]; 4], [[f32; 2]; 4]) = match dir {
        FaceDir::Up => {
            let v = [
                [x1, y2, z1],
                [x1, y2, z2],
                [x2, y2, z2],
                [x2, y2, z1],
            ];
            let uv = uv.unwrap_or([x1, z1, x2, z2]);
            (v, uv_pairs(uv))
        }
        FaceDir::Down => {
            let v = [
                [x1, y1, z2],
                [x1, y1, z1],
                [x2, y1, z1],
                [x2, y1, z2],
            ];
            let uv = uv.unwrap_or([x1, z1, x2, z2]);
            (v, uv_pairs(uv))
        }
        FaceDir::North => {
            let v = [
                [x2, y1, z1],
                [x1, y1, z1],
                [x1, y2, z1],
                [x2, y2, z1],
            ];
            let uv = uv.unwrap_or([x1, 16.0 - y1, x2, 16.0 - y2]);
            (v, uv_pairs(uv))
        }
        FaceDir::South => {
            let v = [
                [x1, y1, z2],
                [x2, y1, z2],
                [x2, y2, z2],
                [x1, y2, z2],
            ];
            let uv = uv.unwrap_or([x1, 16.0 - y1, x2, 16.0 - y2]);
            (v, uv_pairs(uv))
        }
        FaceDir::West => {
            let v = [
                [x1, y1, z1],
                [x1, y1, z2],
                [x1, y2, z2],
                [x1, y2, z1],
            ];
            let uv = uv.unwrap_or([z1, 16.0 - y1, z2, 16.0 - y2]);
            (v, uv_pairs(uv))
        }
        FaceDir::East => {
            let v = [
                [x2, y1, z2],
                [x2, y1, z1],
                [x2, y2, z1],
                [x2, y2, z2],
            ];
            let uv = uv.unwrap_or([z1, 16.0 - y1, z2, 16.0 - y2]);
            (v, uv_pairs(uv))
        }
    };

    // element rotation: single-axis ±45/±22.5 around origin (default 8,8,8)
    let mut verts = base_verts;
    if let Some(r) = rot {
        let origin = r.origin.unwrap_or([8.0, 8.0, 8.0]);
        let (cos_a, sin_a) = {
            let a = r.angle.to_radians();
            (a.cos(), a.sin())
        };
        for v in verts.iter_mut() {
            *v = rotate_around(v, origin, &r.axis, cos_a, sin_a);
        }
    }

    // face texture rotation: rotate the UV assignment by permuting pairs
    let steps = ((face_rot / 90).rem_euclid(4)) as usize;
    let mut uvs = base_uv;
    if steps > 0 {
        // rotating the texture 90° CW moves corner uvs around the quad
        uvs = base_uv.map(|_| [0.0; 2]);
        for i in 0..4 {
            uvs[(i + steps) % 4] = base_uv[i];
        }
        // keep uv corners coherent with the (rotated) texture: swap u/v
        // inside pairs for 90/270 rotations
        if steps % 2 == 1 {
            for p in uvs.iter_mut() {
                *p = [p[1], p[0]];
            }
        }
    }

    (verts, uvs)
}

/// uv rect [x1,y1,x2,y2] (0..16 texture units, y-down) → per-corner pairs
/// aligned with the CCW vert order (v0→(u1,v2) bottom-left in texture space
/// etc. — corners go around the quad the same way the verts do).
#[inline]
fn uv_pairs(uv: [f32; 4]) -> [[f32; 2]; 4] {
    let (u1, v1, u2, v2) = (uv[0] / 16.0, uv[1] / 16.0, uv[2] / 16.0, uv[3] / 16.0);
    [
        [u1, v1],
        [u2, v1],
        [u2, v2],
        [u1, v2],
    ]
}

/// rotate one point around `origin` about a single axis
fn rotate_around(p: &[f32; 3], origin: [f32; 3], axis: &str, cos_a: f32, sin_a: f32) -> [f32; 3] {
    let (px, py, pz) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
    let (rx, ry, rz) = match axis {
        "x" => (px, py * cos_a - pz * sin_a, py * sin_a + pz * cos_a),
        "y" => (px * cos_a + pz * sin_a, py, -px * sin_a + pz * cos_a),
        _ => (px * cos_a - py * sin_a, px * sin_a + py * cos_a, pz),
    };
    [rx + origin[0], ry + origin[1], rz + origin[2]]
}

// ---------------------------------------------------------------------------
// blockstate dispatch compilation
// ---------------------------------------------------------------------------

/// rotate a compiled model by the blockstate x/y config (positions + dirs +
/// cullfaces). Winding is preserved (proper rotations). UVs stay attached to
/// their verts (uvlock behavior; exact non-uvlock texture rotation is a
/// documented parity TODO).
pub fn apply_variant_rotation(m: &CompiledModel, x_rot: i32, y_rot: i32) -> CompiledModel {
    if x_rot == 0 && y_rot == 0 {
        return m.clone();
    }
    let y_steps = ((y_rot / 90).rem_euclid(4)) as i32;
    let flip_y = (x_rot / 90).rem_euclid(4) == 2;
    let rot_pt = |p: &[f32; 3]| -> [f32; 3] {
        let mut x = p[0];
        let mut y = p[1];
        let mut z = p[2];
        // about the block center (8,8,8)
        for _ in 0..y_steps {
            let (nx, nz) = (16.0 - z, x);
            x = nx;
            z = nz;
        }
        if flip_y {
            y = 16.0 - y;
            z = 16.0 - z;
        }
        [x, y, z]
    };
    let elements = m
        .elements
        .iter()
        .map(|el| CompiledElement {
            faces: el
                .faces
                .iter()
                .map(|f| {
                    let verts = f.verts.map(|v| rot_pt(&v));
                    CompiledFace {
                        dir: f.dir.rotated(x_rot, y_rot),
                        verts,
                        uvs: f.uvs,
                        texture: f.texture.clone(),
                        cullface: f.cullface.map(|c| c.rotated(x_rot, y_rot)),
                        tintindex: f.tintindex,
                        shade: f.shade,
                    }
                })
                .collect(),
        })
        .collect();
    CompiledModel {
        ambientocclusion: m.ambientocclusion,
        elements,
        loc: m.loc.clone(),
    }
}

/// Compile the full dispatch table for one block: parse its blockstate JSON,
/// resolve every referenced model, evaluate multipart conditions for every
/// property combination of the block's declared properties.
pub struct BlockDispatchSpec {
    /// block's registry name (blockstates/<name>.json)
    pub name: &'static str,
    /// sorted property definitions (vanilla state-id order: alphabetical,
    /// last-sorted varies fastest)
    pub props: &'static [vc_blocks::blocks::PropDef],
    /// base state id for this block's property states
    pub base_state: u16,
    /// state count = product of property value counts
    pub state_count: u16,
}

/// evaluate a `when` clause against a state's property assignment
fn eval_when(when: &serde_json::Value, props: &[(String, String)]) -> bool {
    match when {
        serde_json::Value::Object(map) => {
            if let Some(or) = map.get("OR") {
                if let Some(arr) = or.as_array() {
                    return arr.iter().any(|c| eval_when(c, props));
                }
            }
            if let Some(and) = map.get("AND") {
                if let Some(arr) = and.as_array() {
                    return arr.iter().all(|c| eval_when(c, props));
                }
            }
            map.iter().all(|(k, v)| prop_matches(props, k, v))
        }
        _ => true, // null / missing → unconditional
    }
}

fn prop_matches(props: &[(String, String)], key: &str, value: &serde_json::Value) -> bool {
    let cur = props.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
    match (cur, value) {
        (Some(cur), serde_json::Value::String(s)) => cur == s,
        (Some(cur), serde_json::Value::Array(list)) => list
            .iter()
            .any(|v| v.as_str().map(|s| cur == s).unwrap_or(false)),
        _ => false,
    }
}

/// Build by_state entries for one block dispatch spec.
/// `read` fetches pack bytes by pack-relative path.
pub fn compile_block_dispatch(
    spec: &BlockDispatchSpec,
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<HashMap<u16, Vec<ModelChoice>>, String> {
    let bs_path = format!("assets/minecraft/blockstates/{}.json", spec.name);
    let bytes = read(&bs_path).ok_or_else(|| format!("blockstate not found: {bs_path}"))?;
    let bs: BlockstateJson = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{bs_path}: bad JSON: {e}"))?;

    // cache resolved (base) models by location
    let mut cache: HashMap<String, Arc<CompiledModel>> = HashMap::new();
    let mut get_model = |loc: &str| -> Result<Arc<CompiledModel>, String> {
        let loc = normalize_loc(loc);
        if let Some(m) = cache.get(&loc) {
            return Ok(Arc::clone(m));
        }
        let m = Arc::new(resolve_model(&loc, &|p| read(&p))?);
        cache.insert(loc, Arc::clone(&m));
        Ok(m)
    };

    let mut by_state: HashMap<u16, Vec<ModelChoice>> = HashMap::new();

    // enumerate every property combination (mixed radix, last prop fastest)
    let total = spec.state_count as usize;
    for idx in 0..total {
        let props = decode_props(spec, idx as u16);
        let state = spec.base_state + idx as u16;
        let mut choices: Vec<ModelChoice> = Vec::new();

        if let Some(variants) = &bs.variants {
            // vanilla: the variant key is the sorted full assignment; "" for
            // property-less blocks. A variant's model ARRAY is a weighted
            // random pick → ONE choice with N alternatives.
            let key = if props.is_empty() {
                String::new()
            } else {
                props
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            // exact match first; fall back to partial matches (vanilla logs
            // an error and picks "" — we accept exact-full only, then any
            // key whose assignments are a subset, preferring "")
            let spec_list = variants
                .get(&key)
                .or_else(|| {
                    if props.is_empty() {
                        variants.get("")
                    } else {
                        // partial-key fallback: all listed props match
                        variants
                            .iter()
                            .find(|(k, _)| {
                                !k.is_empty()
                                    && k.split(',')
                                        .all(|kv| {
                                            let mut it = kv.split('=');
                                            let pk = it.next().unwrap_or("");
                                            let pv = it.next().unwrap_or("");
                                            props.iter().any(|(k2, v2)| k2 == pk && v2 == pv)
                                        })
                            })
                            .map(|(_, v)| v)
                    }
                })
                .ok_or_else(|| {
                    format!("{bs_path}: no variant for state {state} (key {key:?})")
})?;
            let mut alts: Vec<AppliedModel> = Vec::new();
            for v in spec_list.list() {
                let base = get_model(&v.model)?;
                let m = if v.x != 0 || v.y != 0 {
                    Arc::new(apply_variant_rotation(&base, v.x, v.y))
                } else {
                    base
                };
                alts.push(AppliedModel { model: m, weight: v.weight.max(1) as u32 });
            }
            choices.push(ModelChoice { alts });
        }

        if let Some(cases) = &bs.multipart {
            // multipart: each matching case is an INDEPENDENT choice (all
            // matching models render together — post + sides, etc.)
            for case in cases {
                let ok = case
                    .when
                    .as_ref()
                    .map(|w| eval_when(w, &props))
                    .unwrap_or(true);
                if !ok {
                    continue;
                }
                let mut alts: Vec<AppliedModel> = Vec::new();
                for v in case.apply.list() {
                    let base = get_model(&v.model)?;
                    let m = if v.x != 0 || v.y != 0 {
                        Arc::new(apply_variant_rotation(&base, v.x, v.y))
                    } else {
                        base
                    };
                    alts.push(AppliedModel { model: m, weight: v.weight.max(1) as u32 });
                }
                if !alts.is_empty() {
                    choices.push(ModelChoice { alts });
                }
            }
        }

        if choices.is_empty() {
            return Err(format!("{bs_path}: state {state} resolves to no models"));
        }
        by_state.insert(state, choices);
    }
    Ok(by_state)
}

/// decode property assignment for state index idx (mixed radix, sorted
/// alphabetically, last property varies fastest — vanilla algorithm)
pub fn decode_props(spec: &BlockDispatchSpec, idx: u16) -> Vec<(String, String)> {
    let mut idx = idx as usize;
    let mut out: Vec<(String, String)> = Vec::with_capacity(spec.props.len());
    // radix products from the last property (fastest) to the first
    for p in spec.props.iter().rev() {
        let radix = p.values.len().max(1);
        let v = idx % radix;
        idx /= radix;
        out.push((p.name.to_string(), p.values[v].to_string()));
    }
    out.reverse();
    out
}

/// model location → pack-relative path:
/// "minecraft:block/oak_slab" → "assets/minecraft/models/block/oak_slab.json"
pub fn model_path(loc: &str) -> String {
    let (ns, path) = match loc.split_once(':') {
        Some((ns, p)) => (ns, p),
        None => ("minecraft", loc),
    };
    format!("assets/{ns}/models/{path}.json")
}

/// texture location → pack-relative path:
/// "minecraft:block/oak_planks" → "assets/minecraft/textures/block/oak_planks.png"
pub fn texture_path(loc: &str) -> String {
    let (ns, path) = match loc.split_once(':') {
        Some((ns, p)) => (ns, p),
        None => ("minecraft", loc),
    };
    // strip a possible .png suffix the pack author may have included
    let path = path.trim_end_matches(".png");
    format!("assets/{ns}/textures/{path}.png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as Map;

    /// in-memory pack for tests
    fn pack(files: &[(&str, &str)]) -> impl Fn(&str) -> Option<Vec<u8>> {
        let map: Map<String, Vec<u8>> = files
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_bytes().to_vec()))
            .collect();
        move |path| map.get(path).cloned()
    }

    const SLAB_BASE: &str = r##"{
        "parent": "block/block",
        "textures": {"top": "#top", "bottom": "#bottom", "side": "#side"},
        "elements": [{
            "from": [0, 0, 0], "to": [16, 8, 16],
            "faces": {
                "down":  {"texture": "#bottom", "cullface": "down"},
                "up":    {"texture": "#top", "cullface": "up"},
                "north": {"texture": "#side", "uv": [0, 8, 16, 16]},
                "south": {"texture": "#side", "uv": [0, 8, 16, 16]},
                "west":  {"texture": "#side", "uv": [0, 8, 16, 16]},
                "east":  {"texture": "#side", "uv": [0, 8, 16, 16]}
            }
        }]
    }"##;

    const OAK_SLAB: &str = r##"{
        "parent": "block/slab",
        "textures": {
            "top": "block/oak_planks",
            "bottom": "block/oak_planks",
            "side": "block/oak_planks"
        }
    }"##;

    const BLOCK_BLOCK: &str = r##"{"ambientocclusion": true}"##;

    #[test]
    fn resolves_parent_chain_and_texvars() {
        let p = pack(&[
            ("assets/minecraft/models/block/slab.json", SLAB_BASE),
            ("assets/minecraft/models/block/oak_slab.json", OAK_SLAB),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let m = resolve_model("minecraft:block/oak_slab", &p).unwrap();
        assert_eq!(m.elements.len(), 1);
        let faces = &m.elements[0].faces;
        assert_eq!(faces.len(), 6);
        // texture vars resolved through the parent's #top ← child's block/oak_planks
        for f in faces {
            assert_eq!(f.texture, "minecraft:block/oak_planks", "face {:?}", f.dir);
        }
        // slab geometry: top face at y=8
        let up = faces.iter().find(|f| f.dir == FaceDir::Up).unwrap();
        assert!(up.verts.iter().all(|v| (v[1] - 8.0).abs() < 0.001));
        // cullface survived the chain
        assert_eq!(up.cullface, Some(FaceDir::Up));
    }

    #[test]
    fn uv_autogen_flips_v_for_side_faces() {
        let p = pack(&[
            ("assets/minecraft/models/block/slab.json", SLAB_BASE),
            ("assets/minecraft/models/block/oak_slab.json", OAK_SLAB),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        // north face uv explicitly [0,8,16,16] → v range 0.5..1.0
        let m = resolve_model("minecraft:block/oak_slab", &p).unwrap();
        let north = m.elements[0]
            .faces
            .iter()
            .find(|f| f.dir == FaceDir::North)
            .unwrap();
        // corner uv pairs span the bottom half of the texture (y-down)
        for (i, uv) in north.uvs.iter().enumerate() {
            let _ = i;
            assert!(uv[1] >= 0.5 && uv[1] <= 1.0, "uv {uv:?}");
        }
        // and autogen (drop explicit uv): full height
        let auto = r##"{
            "parent": "block/block",
            "textures": {"all": "block/stone"},
            "elements": [{
                "from": [0, 0, 0], "to": [16, 16, 16],
                "faces": { "north": {"texture": "#all"} }
            }]
        }"##;
        let p2 = pack(&[
            ("assets/minecraft/models/block/auto.json", auto),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let m2 = resolve_model("minecraft:block/auto", &p2).unwrap();
        let n2 = &m2.elements[0].faces[0];
        // autogen north uv = [x1, 16−y1, x2, 16−y2] = [0,0,16,16] → full tile
        assert!(n2.uvs.iter().all(|uv| (uv[0] - 0.0).abs() < 0.01 || (uv[0] - 1.0).abs() < 0.01));
        assert!(n2.uvs.iter().all(|uv| (uv[1] - 0.0).abs() < 0.01 || (uv[1] - 1.0).abs() < 0.01));
    }

    #[test]
    fn variant_y_rotation_rotates_positions_and_dirs() {
        let p = pack(&[
            ("assets/minecraft/models/block/slab.json", SLAB_BASE),
            ("assets/minecraft/models/block/oak_slab.json", OAK_SLAB),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let m = resolve_model("minecraft:block/oak_slab", &p).unwrap();
        let r = apply_variant_rotation(&m, 0, 90);
        // top face stays a top face
        let up = r.elements[0].faces.iter().find(|f| f.dir == FaceDir::Up).unwrap();
        assert!(up.verts.iter().all(|v| (v[1] - 8.0).abs() < 0.001));
        // x=180 (top slab): top face becomes bottom at y=8
        let r2 = apply_variant_rotation(&m, 180, 0);
        let down = r2.elements[0].faces.iter().find(|f| f.dir == FaceDir::Down).unwrap();
        assert!(down.verts.iter().all(|v| (v[1] - 8.0).abs() < 0.001));
        assert!(down.cullface == Some(FaceDir::Down));
    }

    #[test]
    fn element_rotation_bakes_45_degrees() {
        let rotated = r##"{
            "parent": "block/block",
            "textures": {"all": "block/stone"},
            "elements": [{
                "from": [0, 0, 0], "to": [16, 16, 16],
                "rotation": {"origin": [8, 8, 8], "axis": "y", "angle": 45},
                "faces": { "up": {"texture": "#all"} }
            }]
        }"##;
        let p = pack(&[
            ("assets/minecraft/models/block/rot.json", rotated),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let m = resolve_model("minecraft:block/rot", &p).unwrap();
        let up = &m.elements[0].faces[0];
        // rotated corners: x²+z² pattern around center, distance 8/√2 from axis
        for v in up.verts.iter() {
            let dx = v[0] - 8.0;
            let dz = v[2] - 8.0;
            let d = (dx * dx + dz * dz).sqrt();
            assert!((d - 11.3137).abs() < 0.01, "corner distance {d} (want 8√2)");
        }
    }

    #[test]
    fn blockstate_variants_and_fallback() {
        let bs = r##"{
            "variants": {
                "half=bottom": {"model": "block/oak_slab"},
                "half=top":    {"model": "block/oak_slab", "x": 180}
            }
        }"##;
        let p = pack(&[
            ("assets/minecraft/blockstates/testslab.json", bs),
            ("assets/minecraft/models/block/slab.json", SLAB_BASE),
            ("assets/minecraft/models/block/oak_slab.json", OAK_SLAB),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let spec = BlockDispatchSpec {
            name: "testslab",
            props: &[vc_blocks::blocks::PropDef { name: "half", values: &["bottom", "top"] }],
            base_state: 100,
            state_count: 2,
        };
        let by_state = compile_block_dispatch(&spec, &p).unwrap();
        assert_eq!(by_state.len(), 2);
        let bottom = &by_state[&100];
        let top = &by_state[&101];
        assert_eq!(bottom.len(), 1);
        assert_eq!(top.len(), 1);
        // top variant is x=180-rotated: its Down face carries y=8 verts
        let up_m = &top[0].alts[0].model;
        let down = up_m.elements[0].faces.iter().find(|f| f.dir == FaceDir::Down).unwrap();
        assert!(down.verts.iter().all(|v| (v[1] - 8.0).abs() < 0.001));
    }

    #[test]
    fn multipart_when_matching() {
        let bs = r##"{
            "multipart": [
                { "apply": {"model": "block/fence_post"} },
                { "when": {"north": "true"}, "apply": {"model": "block/fence_side"} },
                { "when": {"OR": [{"east": "true"}, {"west": "true"}]},
                  "apply": {"model": "block/fence_side", "y": 90} }
            ]
        }"##;
        let post = r##"{
            "parent": "block/block",
            "textures": {"all": "block/planks"},
            "elements": [{"from": [6, 0, 6], "to": [10, 16, 10],
                          "faces": {"up": {"texture": "#all"}}}]
        }"##;
        let side = r##"{
            "parent": "block/block",
            "textures": {"all": "block/planks"},
            "elements": [{"from": [6, 12, 0], "to": [10, 15, 6],
                          "faces": {"up": {"texture": "#all"}}}]
        }"##;
        let p = pack(&[
            ("assets/minecraft/blockstates/fence.json", bs),
            ("assets/minecraft/models/block/fence_post.json", post),
            ("assets/minecraft/models/block/fence_side.json", side),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        // 2 props × 2 (north/east) = 4 states
        let spec = BlockDispatchSpec {
            name: "fence",
            props: &[
                vc_blocks::blocks::PropDef { name: "east", values: &["false", "true"] },
                vc_blocks::blocks::PropDef { name: "north", values: &["false", "true"] },
            ],
            base_state: 200,
            state_count: 4,
        };
        let by_state = compile_block_dispatch(&spec, &p).unwrap();
        // idx: east*2 + north (east sorted first = slower, north fastest)
        // idx 3 = east=true, north=true → 3 choices: post + side + OR side
        let both = &by_state[&203];
        assert_eq!(both.len(), 3, "post + side + rotated side (multipart additive)");
        // idx 0 = both false → post choice only
        assert_eq!(by_state[&200].len(), 1);
        // idx 2 = east=true, north=false → post + OR-matched rotated side
        assert_eq!(by_state[&202].len(), 2);
        // all choices have exactly 1 alternative here (no random arrays)
        assert!(both.iter().all(|c| c.alts.len() == 1));
    }

    #[test]
    fn weighted_random_variants_parse() {
        let bs = r##"{
            "variants": {
                "": [
                    {"model": "block/fence_post", "weight": 1},
                    {"model": "block/fence_side", "weight": 3}
                ]
            }
        }"##;
        let post = r##"{"parent": "block/block", "textures": {"all": "block/planks"},
            "elements": [{"from": [6, 0, 6], "to": [10, 16, 10],
                          "faces": {"up": {"texture": "#all"}}}]}"##;
        let p = pack(&[
            ("assets/minecraft/blockstates/rand.json", bs),
            ("assets/minecraft/models/block/fence_post.json", post),
            ("assets/minecraft/models/block/fence_side.json", post),
            ("assets/minecraft/models/block/block.json", BLOCK_BLOCK),
        ]);
        let spec = BlockDispatchSpec {
            name: "rand",
            props: &[],
            base_state: 300,
            state_count: 1,
        };
        let by_state = compile_block_dispatch(&spec, &p).unwrap();
        let choices = &by_state[&300];
        assert_eq!(choices.len(), 1, "variants = ONE choice");
        let alts = &choices[0].alts;
        assert_eq!(alts.len(), 2, "…with 2 weighted alternatives");
        assert_eq!(alts[0].weight, 1);
        assert_eq!(alts[1].weight, 3);
    }

    #[test]
    fn paths_map_correctly() {
        assert_eq!(
            model_path("minecraft:block/oak_slab"),
            "assets/minecraft/models/block/oak_slab.json"
        );
        assert_eq!(
            texture_path("minecraft:block/oak_planks"),
            "assets/minecraft/textures/block/oak_planks.png"
        );
        assert_eq!(
            texture_path("block/oak_planks.png"),
            "assets/minecraft/textures/block/oak_planks.png"
        );
    }
}
