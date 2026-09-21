//! Resource-pack source layer (Master Spec §19, Phase 1).
//!
//! Abstracts where pack bytes come from (§36 platform abstraction):
//! * native — `FolderSource` rooted at a real directory (the builtin pack at
//!   `voxelcraft/builtin-pack/`, or any user folder later);
//! * wasm — pack files are `fetch()`ed at boot into a `MemorySource` (the
//!   builtin pack is deployed to the web `public/voxelcraft-pack/` by CI),
//!   so the compile pipeline stays identical across platforms.
//!
//! Packs are validated through their manifest: OUR packs carry `pack.json`
//! (`{"pack":{"format":1,…}}`); user-supplied packs from the wider
//! 1.16.5-era ecosystem carry `pack.mcmeta` (`{"pack":{"pack_format":N,…}}`)
//! — both are accepted. Mismatches log a warning but never abort (§46 —
//! a user-supplied imperfect pack must not crash the engine).
//!
//! Namespace interop: user packs may lay files out under ANY
//! `assets/<namespace>/…` (or `data/<namespace>/…`) prefix. At open time
//! the source's entry list is scanned once and every such entry is aliased
//! onto its namespace-stripped flat key — no hardcoded namespace strings,
//! every pack resolves onto the same flat key space as our builtin assets.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// OUR pack manifest format version (`pack.json` → `pack.format`).
pub const PACK_FORMAT_OURS: u32 = 1;

#[derive(Deserialize, Debug)]
struct Manifest {
    pack: ManifestInfo,
}

#[derive(Deserialize, Debug)]
struct ManifestInfo {
    /// our schema (`pack.json`)
    #[serde(default)]
    format: Option<u32>,
    /// the wider ecosystem's schema (`pack.mcmeta` in user packs)
    #[serde(default)]
    pack_format: Option<u32>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    name: String,
}

#[derive(Clone, Debug)]
pub struct PackMeta {
    pub pack_format: u32,
    pub description: String,
}

/// where pack files come from
pub trait PackSource: Send + Sync {
    /// read a pack-relative path, e.g. "blockstates/oak_slab.json"
    fn read(&self, path: &str) -> Option<Vec<u8>>;
    /// human-readable source name for logs
    fn name(&self) -> String;
    /// enumerate every pack-relative path (empty = not enumerable; used
    /// by the namespace-alias scan in [`open`])
    fn list(&self) -> Vec<String> {
        Vec::new()
    }
    /// is this a browsable folder pack? (folder packs extract natively)
    fn is_folder(&self) -> bool {
        false
    }
}

/// open a pack: validate the manifest (ours `pack.json`, the wider
/// ecosystem's `pack.mcmeta` in user packs — either is accepted) and
/// return the usable source wrapped in the namespace-alias shim.
pub fn open(source: Arc<dyn PackSource>) -> Result<(PackMeta, Arc<dyn PackSource>), String> {
    let (bytes, manifest_path) = match source.read("pack.json") {
        Some(b) => (b, "pack.json"),
        None => source
            .read("pack.mcmeta")
            .map(|b| (b, "pack.mcmeta"))
            .ok_or_else(|| {
                format!(
                    "pack manifest not found (pack.json / pack.mcmeta) in {}",
                    source.name()
                )
            })?,
    };
    let meta: Manifest = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{manifest_path}: bad JSON: {e}"))?;
    let pack_format = meta.pack.format.or(meta.pack.pack_format).unwrap_or(0);
    let description = if meta.pack.description.is_empty() {
        meta.pack.name.clone()
    } else {
        meta.pack.description
    };
    let meta = PackMeta {
        pack_format,
        description,
    };
    // §46 resilience: warn on version mismatch, do not fail
    if meta.pack_format != PACK_FORMAT_OURS {
        log_warn(&format!(
            "pack {} declares pack format {} (ours is {PACK_FORMAT_OURS}) — loading anyway",
            source.name(),
            meta.pack_format
        ));
    }
    // namespace-alias scan: map every "assets/<ns>/<rest>" (and
    // "data/<ns>/<rest>") entry onto its stripped flat key
    let mut aliases: HashMap<String, String> = HashMap::new();
    for entry in source.list() {
        for prefix in ["assets/", "data/"] {
            let Some(after_ns) = entry.strip_prefix(prefix) else {
                continue;
            };
            let Some((ns, rest)) = after_ns.split_once('/') else {
                continue;
            };
            if ns.is_empty() || rest.is_empty() {
                continue;
            }
            // flat key = the namespace-stripped path
            // ("textures/gui/hearts.png"), mapping to the real entry
            let flat = rest.to_string();
            // first entry wins; prefer our own namespace when present by
            // letting an exact "voxelcraft" hit override an earlier alias
            if !aliases.contains_key(&flat) || ns == crate::model::NS {
                aliases.insert(flat, entry.clone());
            }
        }
    }
    Ok((meta, Arc::new(NsAliasSource { inner: source, aliases })))
}

/// Read-side namespace shim: answers flat-key reads (`textures/gui/x.png`)
/// from the underlying source either directly (our layout), through the
/// alias map (packs laid out under `assets/<any-namespace>/…`), or
/// through the legacy-NAME map (packs that reference the wider
/// ecosystem's coined names — see `legacy_aliases`).
struct NsAliasSource {
    inner: Arc<dyn PackSource>,
    aliases: HashMap<String, String>,
}

impl NsAliasSource {
    /// remap each path segment through the legacy-name table
    /// (`textures/block/<legacy>.png` → `textures/block/<ours>.png`)
    fn remap_legacy_name(&self, path: &str) -> Option<String> {
        let mut parts: Vec<std::borrow::Cow<'_, str>> =
            path.split('/').map(std::borrow::Cow::Borrowed).collect();
        let mut changed = false;
        for part in parts.iter_mut() {
            let (stem, tail) = match part.rsplit_once('.') {
                Some((s, t)) if matches!(t, "png" | "json" | "mcmeta") => (s, ".".to_string() + t),
                _ => (part.as_ref(), String::new()),
            };
            if let Some(new_stem) = crate::legacy_aliases::legacy_name_alias(stem) {
                *part = std::borrow::Cow::Owned(format!("{new_stem}{tail}"));
                changed = true;
            }
        }
        if changed {
            Some(parts.join("/"))
        } else {
            None
        }
    }
}

impl PackSource for NsAliasSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        if let Some(bytes) = self.inner.read(path) {
            return Some(bytes);
        }
        if let Some(real) = self.aliases.get(path) {
            if let Some(bytes) = self.inner.read(real) {
                return Some(bytes);
            }
        }
        // legacy-NAME interop: retry the remapped path (direct, then
        // through the namespace alias) — the ecosystem's packs carry
        // its own coined names in their paths
        if let Some(remapped) = self.remap_legacy_name(path) {
            if let Some(bytes) = self.inner.read(&remapped) {
                return Some(bytes);
            }
            if let Some(real) = self.aliases.get(&remapped) {
                if let Some(bytes) = self.inner.read(real) {
                    return Some(bytes);
                }
            }
        }
        None
    }

    fn name(&self) -> String {
        self.inner.name()
    }

    fn list(&self) -> Vec<String> {
        self.inner.list()
    }

    fn is_folder(&self) -> bool {
        self.inner.is_folder()
    }
}

fn log_warn(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("[pack] {msg}").into());
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[pack] {msg}");
}

// ---------------------------------------------------------------- sources --

/// native: a plain directory (builtin pack or a user folder)
#[cfg(not(target_arch = "wasm32"))]
pub struct FolderSource {
    root: std::path::PathBuf,
    label: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl FolderSource {
    pub fn new(root: impl Into<std::path::PathBuf>, label: &str) -> Self {
        FolderSource {
            root: root.into(),
            label: label.to_string(),
        }
    }

    pub fn exists(&self) -> bool {
        self.root.join("pack.json").is_file() || self.root.join("pack.mcmeta").is_file()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PackSource for FolderSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        // path traversal safety: reject anything escaping the root
        if path.contains("..") || path.starts_with('/') {
            return None;
        }
        std::fs::read(self.root.join(path)).ok()
    }

    fn name(&self) -> String {
        format!("folder:{} ({})", self.root.display(), self.label)
    }

    fn list(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![(self.root.clone(), String::new())];
        while let Some((dir, rel)) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if path.is_dir() {
                    stack.push((path, format!("{rel}{name}/")));
                } else {
                    out.push(format!("{rel}{name}"));
                }
            }
        }
        out.sort();
        out
    }

    fn is_folder(&self) -> bool {
        true
    }
}

/// any platform: files already in memory (wasm fetch results, embedded zip
/// later)
pub struct MemorySource {
    files: HashMap<String, Arc<Vec<u8>>>,
    label: String,
}

impl MemorySource {
    pub fn new(label: &str) -> Self {
        MemorySource {
            files: HashMap::new(),
            label: label.to_string(),
        }
    }

    pub fn insert(&mut self, path: &str, bytes: Vec<u8>) {
        self.files.insert(path.to_string(), Arc::new(bytes));
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl PackSource for MemorySource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.files.get(path).map(|b| b.as_ref().clone())
    }

    fn name(&self) -> String {
        format!("memory:{} ({} files)", self.label, self.files.len())
    }

    fn list(&self) -> Vec<String> {
        let mut v: Vec<String> = self.files.keys().cloned().collect();
        v.sort();
        v
    }
}

// ----------------------------------------------------------- wasm fetches --

/// wasm: fetch the builtin pack's file set into memory at boot.
/// The file list is derived from the block-dispatch specs (blockstates →
/// models → textures), so no directory listing is needed.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_builtin_pack(
    specs: &[crate::model::BlockDispatchSpec],
) -> Option<MemorySource> {
    let mut mem = MemorySource::new("builtin (fetched)");
    // 1. pack.json + all blockstates
    let mut wanted: Vec<String> = vec!["pack.json".to_string()];
    for spec in specs {
        wanted.push(format!("blockstates/{}.json", spec.name));
    }
    // fetch blockstates first, parse model refs from them
    let mut models: Vec<String> = Vec::new();
    for spec in specs {
        let bs_path = format!("blockstates/{}.json", spec.name);
        let bytes = fetch_bytes(&bs_path).await?;
        collect_model_refs(&bytes, &mut models);
        mem.insert(&bs_path, bytes);
    }
    // 2. fetch models (following parents)
    let mut seen: std::collections::HashSet<String> = Default::default();
    let mut queue = models;
    let mut textures: Vec<String> = Vec::new();
    while let Some(loc) = queue.pop() {
        if !seen.insert(loc.clone()) {
            continue;
        }
        let path = crate::model::model_path(&crate::model::normalize_loc(&loc));
        let Some(bytes) = fetch_bytes(&path).await else {
            continue; // missing model → missing-model fallback at compile
        };
        collect_parent_and_texture_refs(&bytes, &mut queue, &mut textures);
        mem.insert(&path, bytes);
    }
    // 3. fetch textures (and their .png.json animation metadata if present)
    for tex in textures {
        let path = crate::model::texture_path(&crate::model::normalize_loc(&tex));
        if seen.insert(path.clone()) {
            if let Some(bytes) = fetch_bytes(&path).await {
                mem.insert(&path, bytes);
            }
            let anim = path.replace(".png", ".png.json");
            if seen.insert(anim.clone()) {
                if let Some(bytes) = fetch_bytes(&anim).await {
                    mem.insert(&anim, bytes);
                }
            }
        }
    }
    // pack.json last (for open()) — fetch it too
    if let Some(bytes) = fetch_bytes("pack.json").await {
        mem.insert("pack.json", bytes);
    }
    let _ = wanted;
    Some(mem)
}

/// The Classic Art builtin pack's file manifest (the 52 clean-room
/// retro look-alike textures + pack.json). Deployed to the web as
/// `/voxelcraft-pack-classic-art/**` by the bundle script; the exact
/// list is baked in because wasm has no directory listing.
pub const CLASSIC_ART_FILES: &[&str] = &[
    "textures/block/bedrock.png",
    "textures/block/black_wool.png",
    "textures/block/blue_wool.png",
    "textures/block/bookshelf.png",
    "textures/block/bricks.png",
    "textures/block/clay.png",
    "textures/block/coal_ore.png",
    "textures/block/cobblestone.png",
    "textures/block/crafting_table_side.png",
    "textures/block/crafting_table_top.png",
    "textures/block/dandelion.png",
    "textures/block/diamond_block.png",
    "textures/block/diamond_ore.png",
    "textures/block/dirt.png",
    "textures/block/emerald_ore.png",
    "textures/block/end_stone.png",
    "textures/block/furnace_front_on.png",
    "textures/block/furnace_side.png",
    "textures/block/furnace_top.png",
    "textures/block/glass.png",
    "textures/block/glowstone.png",
    "textures/block/gold_block.png",
    "textures/block/gold_ore.png",
    "textures/block/grass_block_side.png",
    "textures/block/grass_block_top.png",
    "textures/block/gravel.png",
    "textures/block/ice.png",
    "textures/block/iron_block.png",
    "textures/block/iron_ore.png",
    "textures/block/lapis_ore.png",
    "textures/block/mossy_cobblestone.png",
    "textures/block/nether_bricks.png",
    "textures/block/netherrack.png",
    "textures/block/oak_leaves.png",
    "textures/block/oak_log.png",
    "textures/block/oak_log_top.png",
    "textures/block/oak_planks.png",
    "textures/block/obsidian.png",
    "textures/block/poppy.png",
    "textures/block/red_wool.png",
    "textures/block/redstone_ore.png",
    "textures/block/sand.png",
    "textures/block/snow.png",
    "textures/block/soul_sand.png",
    "textures/block/spawner.png",
    "textures/block/stone.png",
    "textures/block/stone_bricks.png",
    "textures/block/tall_grass.png",
    "textures/block/tnt_side.png",
    "textures/block/tnt_top.png",
    "textures/block/white_wool.png",
    "textures/block/yellow_wool.png",
    "pack.json",
];

/// wasm: fetch the Classic Art builtin pack (the retro "classic look"
/// analog) into memory. Cached by the caller at boot so the resource-pack
/// screen can toggle it synchronously afterwards.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_classic_art_pack() -> Option<MemorySource> {
    let mut mem = MemorySource::new("classic-art (fetched)");
    let mut any = false;
    for path in CLASSIC_ART_FILES {
        if let Some(bytes) = fetch_bytes_base("/voxelcraft-pack-classic-art", path).await {
            mem.insert(path, bytes);
            any = true;
        }
    }
    if any {
        Some(mem)
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(path: &str) -> Option<Vec<u8>> {
    // same-origin fetch of the deployed builtin pack (public/voxelcraft-pack).
    // Returns None on any network/HTTP failure — callers fall back to the
    // missing-asset path (§46), never panic.
    fetch_bytes_base("/voxelcraft-pack", path).await
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes_base(base: &str, path: &str) -> Option<Vec<u8>> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let url = format!("{base}/{path}");
    let window = web_sys::window()?;
    let Ok(resp_val) = JsFuture::from(window.fetch_with_str(&url)).await else {
        return None;
    };
    let resp = resp_val.dyn_into::<web_sys::Response>().ok()?;
    if !resp.ok() {
        return None;
    }
    let Ok(buf_val) = JsFuture::from(resp.array_buffer().ok()?).await else {
        return None;
    };
    let buf = buf_val.dyn_into::<js_sys::ArrayBuffer>().ok()?;
    Some(js_sys::Uint8Array::new(&buf).to_vec())
}

#[cfg(target_arch = "wasm32")]
fn collect_model_refs(bytes: &[u8], out: &mut Vec<String>) {
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(bytes) {
        collect_str_field_recursive(&v, "model", out);
    }
}

#[cfg(target_arch = "wasm32")]
fn collect_parent_and_texture_refs(bytes: &[u8], models: &mut Vec<String>, textures: &mut Vec<String>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return;
    };
    if let Some(p) = v.get("parent").and_then(|p| p.as_str()) {
        models.push(p.to_string());
    }
    if let Some(t) = v.get("textures").and_then(|t| t.as_object()) {
        for tv in t.values() {
            if let Some(s) = tv.as_str() {
                if !s.starts_with('#') {
                    textures.push(s.to_string());
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn collect_str_field_recursive(v: &serde_json::Value, field: &str, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                if k == field {
                    if let Some(s) = val.as_str() {
                        out.push(s.to_string());
                    }
                }
                collect_str_field_recursive(val, field, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                collect_str_field_recursive(val, field, out);
            }
        }
        _ => {}
    }
}


// ------------------------------------------------------- UI Phase 4 --
// GUI texture overrides through the resource-pack pipeline (Master
// Prompt Phase 4: a WIRING task — this crate already owns pack
// discovery/validation; nothing here duplicates the resolver).

/// Ordered resource-pack stack, HIGHEST priority first. The builtin
/// procedural GUI set is NOT a member — it is the fallback BELOW the
/// stack (the loader merges user-pack sheets over builtin).
pub struct PackStack {
    sources: Vec<Arc<dyn PackSource>>,
}

impl Default for PackStack {
    fn default() -> Self {
        Self::new()
    }
}

impl PackStack {
    pub fn new() -> Self {
        PackStack { sources: Vec::new() }
    }

    /// add a source at the given priority position: index 0 = highest
    /// (applied last in scan order, vanilla-style "top of the list")
    pub fn push_front(&mut self, source: Arc<dyn PackSource>) {
        self.sources.insert(0, source);
    }

    /// read `path` from the highest-priority source that provides it
    /// (returns the bytes + the source name for logs). Sources opened
    /// through [`open`] already carry the namespace-alias shim, so a
    /// flat-key read resolves from any pack layout.
    pub fn read_first(&self, path: &str) -> Option<(Vec<u8>, String)> {
        for s in &self.sources {
            if let Some(bytes) = s.read(path) {
                return Some((bytes, s.name()));
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

/// A `.zip` resource pack. Reuses the Phase 9 zip reader (flate2, zero
/// new dependencies).
pub struct ZipSource {
    files: crate::zip::ZipFiles,
    label: String,
}

impl ZipSource {
    /// open a zip pack from disk (None when unreadable as a zip)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        let files = crate::zip::ZipFiles::from_bytes(&bytes)?;
        Some(ZipSource {
            files,
            label: path.display().to_string(),
        })
    }

    /// open from in-memory zip bytes (tests)
    pub fn from_bytes(bytes: &[u8], label: &str) -> Option<Self> {
        let files = crate::zip::ZipFiles::from_bytes(bytes)?;
        Some(ZipSource {
            files,
            label: label.to_string(),
        })
    }
}

impl PackSource for ZipSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.files.read_file(path)
    }

    fn name(&self) -> String {
        format!("zip:{}", self.label)
    }

    fn list(&self) -> Vec<String> {
        super::datapack::PackFiles::list(&self.files, "")
    }
}

/// Discover user packs under `dir`: every sub-folder WITH a pack.mcmeta
/// plus every `.zip`. Alphabetical order — the LAST name lands on top
/// (highest priority), matching vanilla's list-on-top-wins behavior.
/// Unreadable entries are skipped silently (§46: a broken pack never
/// breaks boot).
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_user_packs(dir: &std::path::Path) -> Vec<Arc<dyn PackSource>> {
    scan_user_packs_named(dir)
        .into_iter()
        .map(|(_name, src)| src)
        .collect()
}

/// The 2026-09-14 round: scan `resourcepacks/` and keep the NAMES (folder
/// or zip file name) so the Resource Packs screen can list/enable/disable
/// individual packs (vanilla lists "file/<name>" in options.txt). Returns
/// them ALPHABETICALLY — priority ordering is the caller's business (the
/// Selected list, not the disk order, decides precedence).
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_user_packs_named(dir: &std::path::Path) -> Vec<(String, Arc<dyn PackSource>)> {
    let mut found: Vec<(String, Arc<dyn PackSource>)> = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let name = name.to_string();
        if path.is_dir() {
            let folder = FolderSource::new(path.clone(), &name);
            if folder.exists() {
                if let Ok((_meta, src)) = open(Arc::new(folder)) {
                    found.push((name, src));
                }
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("zip") {
            if let Some(zip) = ZipSource::from_path(&path) {
                if let Ok((_meta, src)) = open(Arc::new(zip)) {
                    found.push((name, src));
                }
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// logical GUI texture name -> pack-relative path:
/// "hearts" -> "textures/gui/hearts.png" (our flat layout; user-supplied
/// packs laid out under any "assets/<namespace>/" prefix resolve through
/// the alias shim built in [`open`])
pub fn gui_texture_path(name: &str) -> String {
    crate::model::texture_path(&format!("gui/{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn our_manifest_validates() {
        let mut mem = MemorySource::new("test");
        mem.insert(
            "pack.json",
            br#"{"pack":{"format":1,"name":"t","description":"our pack"}}"#.to_vec(),
        );
        let src: Arc<dyn PackSource> = Arc::new(mem);
        let (meta, _) = open(src).unwrap();
        assert_eq!(meta.pack_format, 1);
        assert_eq!(meta.description, "our pack");
    }

    #[test]
    fn legacy_ecosystem_manifest_accepted() {
        // user-supplied packs from the wider 1.16.5-era ecosystem carry
        // pack.mcmeta with the pack_format key — accepted read-side
        let mut mem = MemorySource::new("user pack");
        mem.insert(
            "pack.mcmeta",
            br#"{"pack":{"pack_format":6,"description":"user pack"}}"#.to_vec(),
        );
        let src: Arc<dyn PackSource> = Arc::new(mem);
        let (meta, src) = open(src).unwrap();
        assert_eq!(meta.pack_format, 6);
        assert_eq!(meta.description, "user pack");
        // the opened source answers manifest reads through the shim
        assert!(src.read("pack.mcmeta").is_some());
    }

    #[test]
    fn missing_manifest_errors_cleanly() {
        let mem = MemorySource::new("empty");
        let src: Arc<dyn PackSource> = Arc::new(mem);
        assert!(open(src).is_err());
    }

    #[test]
    fn memory_source_roundtrip_and_list() {
        let mut mem = MemorySource::new("t");
        mem.insert("a/b.txt", b"hello".to_vec());
        assert_eq!(mem.read("a/b.txt"), Some(b"hello".to_vec()));
        assert_eq!(mem.read("missing"), None);
        assert_eq!(mem.len(), 1);
        assert_eq!(mem.list(), vec!["a/b.txt".to_string()]);
    }

    #[test]
    fn gui_texture_path_is_flat() {
        assert_eq!(gui_texture_path("hearts"), "textures/gui/hearts.png");
        assert_eq!(
            gui_texture_path("options_background"),
            "textures/gui/options_background.png"
        );
    }

    #[test]
    fn pack_stack_resolves_highest_priority_first() {
        let mut low = MemorySource::new("low");
        low.insert("textures/gui/hearts.png", b"low".to_vec());
        low.insert("textures/gui/hunger.png", b"low".to_vec());
        let mut high = MemorySource::new("high");
        high.insert("textures/gui/hearts.png", b"high".to_vec());
        let mut stack = PackStack::new();
        stack.push_front(Arc::new(high));
        stack.push_front(Arc::new(low)); // low pushed to front = now highest
        // low wins hearts AND hunger; high's hearts is shadowed
        let (hearts, name) = stack
            .read_first("textures/gui/hearts.png")
            .unwrap_or_else(|| (Vec::new(), String::new()));
        assert_eq!(hearts, b"low".to_vec());
        assert!(name.contains("low"));
        let (hunger, _) = stack
            .read_first("textures/gui/hunger.png")
            .unwrap_or_else(|| (Vec::new(), String::new()));
        assert_eq!(hunger, b"low".to_vec());
        // nothing provides widgets
        assert!(stack.read_first("textures/gui/widgets.png").is_none());
    }

    #[test]
    fn empty_stack_reads_nothing() {
        let stack = PackStack::new();
        assert!(stack.is_empty());
        assert!(stack.read_first("pack.json").is_none());
    }

    /// Namespace interop: a user-supplied pack laid out under ANY
    /// `assets/<namespace>/…` prefix resolves through the alias shim
    /// built at open() — flat-key reads find the namespaced entries,
    /// with zero hardcoded namespace strings anywhere.
    #[test]
    fn namespaced_pack_layout_resolves_through_the_alias_shim() {
        let mut ns_pack = MemorySource::new("namespaced-pack");
        ns_pack.insert(
            "assets/some-ecosystem-pack/textures/gui/hearts.png",
            b"namespaced".to_vec(),
        );
        ns_pack.insert(
            "pack.mcmeta",
            br#"{"pack":{"pack_format":6}}"#.to_vec(),
        );
        let (_meta, src) = open(Arc::new(ns_pack)).unwrap();
        // flat-key read resolves through the alias
        assert_eq!(
            src.read("textures/gui/hearts.png"),
            Some(b"namespaced".to_vec())
        );
        // direct reads of the real entry still work
        assert!(src.read("assets/some-ecosystem-pack/textures/gui/hearts.png").is_some());
    }

    /// a pack carrying BOTH layouts: the flat entry wins (alias only
    /// fills gaps, never shadows the pack's own flat files)
    #[test]
    fn flat_layout_wins_over_the_alias() {
        let mut both = MemorySource::new("both");
        both.insert("textures/gui/hearts.png", b"flat".to_vec());
        both.insert(
            "assets/some-ecosystem-pack/textures/gui/hearts.png",
            b"namespaced".to_vec(),
        );
        both.insert("pack.json", br#"{"pack":{"format":1}}"#.to_vec());
        let (_meta, src) = open(Arc::new(both)).unwrap();
        assert_eq!(src.read("textures/gui/hearts.png"), Some(b"flat".to_vec()));
    }

    /// a real zip built in-memory: the ZipSource reads the same paths a
    /// folder pack would (zip-vs-folder parity, D8)
    #[test]
    fn zip_source_resolves_the_same_paths_as_a_folder_pack() {
        fn le16(v: u16) -> [u8; 2] {
            [v as u8, (v >> 8) as u8]
        }
        fn le32(v: u32) -> [u8; 4] {
            [
                v as u8,
                (v >> 8) as u8,
                (v >> 16) as u8,
                (v >> 24) as u8,
            ]
        }
        let name = b"textures/gui/hearts.png";
        let data = b"zip-heart".to_vec();
        let crc = crc32(&data);
        let mut out: Vec<u8> = Vec::new();
        let local_offset = 0u32;
        out.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // local sig
        out.extend_from_slice(&le16(20)); // version
        out.extend_from_slice(&le16(0)); // flags
        out.extend_from_slice(&le16(0)); // method 0 = stored
        out.extend_from_slice(&le16(0)); // time
        out.extend_from_slice(&le16(0)); // date
        out.extend_from_slice(&le32(crc));
        out.extend_from_slice(&le32(data.len() as u32));
        out.extend_from_slice(&le32(data.len() as u32));
        out.extend_from_slice(&le16(name.len() as u16));
        out.extend_from_slice(&le16(0)); // extra len
        out.extend_from_slice(name);
        out.extend_from_slice(&data);
        let cd_offset = out.len() as u32;
        out.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]); // CD sig
        out.extend_from_slice(&le16(20)); // version made by
        out.extend_from_slice(&le16(20)); // version needed
        out.extend_from_slice(&le16(0)); // flags
        out.extend_from_slice(&le16(0)); // method
        out.extend_from_slice(&le16(0)); // time
        out.extend_from_slice(&le16(0)); // date
        out.extend_from_slice(&le32(crc));
        out.extend_from_slice(&le32(data.len() as u32));
        out.extend_from_slice(&le32(data.len() as u32));
        out.extend_from_slice(&le16(name.len() as u16));
        out.extend_from_slice(&le16(0)); // extra
        out.extend_from_slice(&le16(0)); // comment
        out.extend_from_slice(&le16(0)); // disk
        out.extend_from_slice(&le16(0)); // int attrs
        out.extend_from_slice(&le32(0)); // ext attrs
        out.extend_from_slice(&le32(local_offset));
        out.extend_from_slice(name);
        let eocd_offset = out.len() as u32;
        out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]); // EOCD
        out.extend_from_slice(&le16(0)); // disk
        out.extend_from_slice(&le16(0)); // cd disk
        out.extend_from_slice(&le16(1)); // entries this disk
        out.extend_from_slice(&le16(1)); // total entries
        out.extend_from_slice(&le32(eocd_offset - cd_offset)); // cd size
        out.extend_from_slice(&le32(cd_offset)); // cd offset
        out.extend_from_slice(&le16(0)); // comment len

        let Some(zip) = ZipSource::from_bytes(&out, "test.zip") else {
            panic!("hand-built zip must parse");
        };
        let src: Arc<dyn PackSource> = Arc::new(zip);
        assert_eq!(
            src.read("textures/gui/hearts.png"),
            Some(b"zip-heart".to_vec())
        );
        // folder parity: the same logical path through a MemorySource
        let mut folder_like = MemorySource::new("folder-like");
        folder_like.insert("textures/gui/hearts.png", b"zip-heart".to_vec());
        assert_eq!(
            folder_like.read("textures/gui/hearts.png"),
            src.read("textures/gui/hearts.png")
        );
    }

    /// CRC-32 (IEEE) for the hand-built zip above
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in data {
            crc ^= b as u32;
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        !crc
    }

    /// a future manifest format only WARNS (the repo's §46 policy) and
    /// the pack still opens
    #[test]
    fn future_manifest_format_warns_but_opens() {
        let mut mem = MemorySource::new("fmt");
        mem.insert(
            "pack.json",
            br#"{"pack":{"format":99,"description":"future"}}"#.to_vec(),
        );
        let src: Arc<dyn PackSource> = Arc::new(mem);
        let r = open(src);
        assert!(r.is_ok(), "mismatched format must not reject (§46)");
        let (meta, _) = r.unwrap_or((PackMeta {
            pack_format: 0,
            description: String::new(),
        }, Arc::new(MemorySource::new("x"))));
        assert_eq!(meta.pack_format, 99);
    }
}
