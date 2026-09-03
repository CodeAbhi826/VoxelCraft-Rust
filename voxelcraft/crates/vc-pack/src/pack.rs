//! Resource-pack source layer (Master Spec §19, Phase 1).
//!
//! Abstracts where pack bytes come from (§36 platform abstraction):
//! * native — `FolderSource` rooted at a real directory (the builtin pack at
//!   `voxelcraft/assets/`, or any user folder later);
//! * wasm — pack files are `fetch()`ed at boot into a `MemorySource` (the
//!   builtin pack is deployed to the web `public/assets/` by CI), so the
//!   compile pipeline stays identical across platforms.
//!
//! `pack.mcmeta` is validated: pack_format 6 = 1.16.2–1.16.5 (VERIFIED,
//! minecraft.wiki). Mismatched formats log a warning but never abort (§46 —
//! a user-supplied imperfect pack must not crash the engine).

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// 1.16.2 – 1.16.5 resource-pack format (VERIFIED)
pub const PACK_FORMAT_1_16_5: u32 = 6;

#[derive(Deserialize, Debug)]
struct McMeta {
    pack: PackInfo,
}

#[derive(Deserialize, Debug)]
struct PackInfo {
    pack_format: u32,
    #[serde(default)]
    description: String,
}

#[derive(Clone, Debug)]
pub struct PackMeta {
    pub pack_format: u32,
    pub description: String,
}

/// where pack files come from
pub trait PackSource: Send + Sync {
    /// read a pack-relative path, e.g. "assets/minecraft/blockstates/oak_slab.json"
    fn read(&self, path: &str) -> Option<Vec<u8>>;
    /// human-readable source name for logs
    fn name(&self) -> String;
    /// enumerate nothing yet (targeted reads only for Phase 1)
    fn is_folder(&self) -> bool {
        false
    }
}

/// open a pack: validate pack.mcmeta and return the usable source
pub fn open(source: Arc<dyn PackSource>) -> Result<(PackMeta, Arc<dyn PackSource>), String> {
    let bytes = source
        .read("pack.mcmeta")
        .ok_or_else(|| format!("pack.mcmeta not found in {}", source.name()))?;
    let meta: McMeta = serde_json::from_slice(&bytes)
        .map_err(|e| format!("pack.mcmeta: bad JSON: {e}"))?;
    let meta = PackMeta {
        pack_format: meta.pack.pack_format,
        description: meta.pack.description,
    };
    // §46 resilience: warn on version mismatch, do not fail
    if meta.pack_format != PACK_FORMAT_1_16_5 {
        log_warn(&format!(
            "pack {} declares pack_format {} (target is 6 for 1.16.5) — loading anyway",
            source.name(),
            meta.pack_format
        ));
    }
    Ok((meta, source))
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
        self.root.join("pack.mcmeta").is_file()
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
    // 1. pack.mcmeta + all blockstates
    let mut wanted: Vec<String> = vec!["pack.mcmeta".to_string()];
    for spec in specs {
        wanted.push(format!("assets/minecraft/blockstates/{}.json", spec.name));
    }
    // fetch blockstates first, parse model refs from them
    let mut models: Vec<String> = Vec::new();
    for spec in specs {
        let bs_path = format!("assets/minecraft/blockstates/{}.json", spec.name);
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
    // 3. fetch textures (and their .mcmeta animation metadata if present)
    for tex in textures {
        let path = crate::model::texture_path(&crate::model::normalize_loc(&tex));
        if seen.insert(path.clone()) {
            if let Some(bytes) = fetch_bytes(&path).await {
                mem.insert(&path, bytes);
            }
            let mcmeta = path.replace(".png", ".png.mcmeta");
            if seen.insert(mcmeta.clone()) {
                if let Some(bytes) = fetch_bytes(&mcmeta).await {
                    mem.insert(&mcmeta, bytes);
                }
            }
        }
    }
    // pack.mcmeta last (for open()) — fetch it too
    if let Some(bytes) = fetch_bytes("pack.mcmeta").await {
        mem.insert("pack.mcmeta", bytes);
    }
    let _ = wanted;
    Some(mem)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(path: &str) -> Option<Vec<u8>> {
    // same-origin fetch of the deployed builtin pack (public/voxelcraft-pack).
    // Returns None on any network/HTTP failure — callers fall back to the
    // missing-asset path (§46), never panic.
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let url = format!("/voxelcraft-pack/{path}");
    let Some(window) = web_sys::window() else { return None };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcmeta_validates_and_version_mismatch_only_warns() {
        let mut mem = MemorySource::new("test");
        mem.insert(
            "pack.mcmeta",
            br#"{"pack":{"pack_format":5,"description":"old pack"}}"#.to_vec(),
        );
        let src: Arc<dyn PackSource> = Arc::new(mem);
        let (meta, _) = open(src).unwrap();
        assert_eq!(meta.pack_format, 5);
        assert_eq!(meta.description, "old pack");
    }

    #[test]
    fn missing_mcmeta_errors_cleanly() {
        let mem = MemorySource::new("empty");
        let src: Arc<dyn PackSource> = Arc::new(mem);
        assert!(open(src).is_err());
    }

    #[test]
    fn memory_source_roundtrip() {
        let mut mem = MemorySource::new("t");
        mem.insert("a/b.txt", b"hello".to_vec());
        assert_eq!(mem.read("a/b.txt"), Some(b"hello".to_vec()));
        assert_eq!(mem.read("missing"), None);
        assert_eq!(mem.len(), 1);
    }
}
