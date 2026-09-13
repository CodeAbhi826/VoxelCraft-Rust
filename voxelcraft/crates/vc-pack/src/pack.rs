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

/// The Programmer Art builtin pack's file manifest (the 52 clean-room
/// retro look-alike textures + pack.mcmeta). Deployed to the web as
/// `/voxelcraft-pack-programmer-art/**` by the bundle script; the exact
/// list is baked in because wasm has no directory listing.
pub const PROGRAMMER_ART_FILES: &[&str] = &[
    "assets/minecraft/textures/block/bedrock.png",
    "assets/minecraft/textures/block/black_wool.png",
    "assets/minecraft/textures/block/blue_wool.png",
    "assets/minecraft/textures/block/bookshelf.png",
    "assets/minecraft/textures/block/bricks.png",
    "assets/minecraft/textures/block/clay.png",
    "assets/minecraft/textures/block/coal_ore.png",
    "assets/minecraft/textures/block/cobblestone.png",
    "assets/minecraft/textures/block/crafting_table_side.png",
    "assets/minecraft/textures/block/crafting_table_top.png",
    "assets/minecraft/textures/block/dandelion.png",
    "assets/minecraft/textures/block/diamond_block.png",
    "assets/minecraft/textures/block/diamond_ore.png",
    "assets/minecraft/textures/block/dirt.png",
    "assets/minecraft/textures/block/emerald_ore.png",
    "assets/minecraft/textures/block/end_stone.png",
    "assets/minecraft/textures/block/furnace_front_on.png",
    "assets/minecraft/textures/block/furnace_side.png",
    "assets/minecraft/textures/block/furnace_top.png",
    "assets/minecraft/textures/block/glass.png",
    "assets/minecraft/textures/block/glowstone.png",
    "assets/minecraft/textures/block/gold_block.png",
    "assets/minecraft/textures/block/gold_ore.png",
    "assets/minecraft/textures/block/grass_block_side.png",
    "assets/minecraft/textures/block/grass_block_top.png",
    "assets/minecraft/textures/block/gravel.png",
    "assets/minecraft/textures/block/ice.png",
    "assets/minecraft/textures/block/iron_block.png",
    "assets/minecraft/textures/block/iron_ore.png",
    "assets/minecraft/textures/block/lapis_ore.png",
    "assets/minecraft/textures/block/mossy_cobblestone.png",
    "assets/minecraft/textures/block/nether_bricks.png",
    "assets/minecraft/textures/block/netherrack.png",
    "assets/minecraft/textures/block/oak_leaves.png",
    "assets/minecraft/textures/block/oak_log.png",
    "assets/minecraft/textures/block/oak_log_top.png",
    "assets/minecraft/textures/block/oak_planks.png",
    "assets/minecraft/textures/block/obsidian.png",
    "assets/minecraft/textures/block/poppy.png",
    "assets/minecraft/textures/block/red_wool.png",
    "assets/minecraft/textures/block/redstone_ore.png",
    "assets/minecraft/textures/block/sand.png",
    "assets/minecraft/textures/block/snow.png",
    "assets/minecraft/textures/block/soul_sand.png",
    "assets/minecraft/textures/block/spawner.png",
    "assets/minecraft/textures/block/stone.png",
    "assets/minecraft/textures/block/stone_bricks.png",
    "assets/minecraft/textures/block/tall_grass.png",
    "assets/minecraft/textures/block/tnt_side.png",
    "assets/minecraft/textures/block/tnt_top.png",
    "assets/minecraft/textures/block/white_wool.png",
    "assets/minecraft/textures/block/yellow_wool.png",
    "pack.mcmeta",
];

/// wasm: fetch the Programmer Art builtin pack (the vanilla Programmer
/// Art analog — "the old pre-1.14 textures", minecraft.wiki/w/Programmer_
/// Art) into memory. Cached by the caller at boot so the resource-pack
/// screen can toggle it synchronously afterwards.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_programmer_art_pack() -> Option<MemorySource> {
    let mut mem = MemorySource::new("programmer-art (fetched)");
    let mut any = false;
    for path in PROGRAMMER_ART_FILES {
        if let Some(bytes) = fetch_bytes_base("/voxelcraft-pack-programmer-art", path).await {
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
    /// (returns the bytes + the source name for logs)
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
/// "hearts" -> "assets/minecraft/textures/gui/hearts.png" (the
/// `minecraft` namespace so packs written for MC 1.16.5 work unchanged;
/// custom namespaces resolve only through explicit "ns:name" requests)
pub fn gui_texture_path(name: &str) -> String {
    crate::model::texture_path(&format!("gui/{name}"))
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

    #[test]
    fn gui_texture_path_uses_the_minecraft_namespace() {
        assert_eq!(
            gui_texture_path("hearts"),
            "assets/minecraft/textures/gui/hearts.png"
        );
        assert_eq!(
            gui_texture_path("options_background"),
            "assets/minecraft/textures/gui/options_background.png"
        );
    }

    #[test]
    fn pack_stack_resolves_highest_priority_first() {
        let mut low = MemorySource::new("low");
        low.insert("assets/minecraft/textures/gui/hearts.png", b"low".to_vec());
        low.insert("assets/minecraft/textures/gui/hunger.png", b"low".to_vec());
        let mut high = MemorySource::new("high");
        high.insert("assets/minecraft/textures/gui/hearts.png", b"high".to_vec());
        let mut stack = PackStack::new();
        stack.push_front(Arc::new(high));
        stack.push_front(Arc::new(low)); // low pushed to front = now highest
        // low wins hearts AND hunger; high's hearts is shadowed
        let (hearts, name) = stack
            .read_first("assets/minecraft/textures/gui/hearts.png")
            .unwrap_or_else(|| (Vec::new(), String::new()));
        assert_eq!(hearts, b"low".to_vec());
        assert!(name.contains("low"));
        let (hunger, _) = stack
            .read_first("assets/minecraft/textures/gui/hunger.png")
            .unwrap_or_else(|| (Vec::new(), String::new()));
        assert_eq!(hunger, b"low".to_vec());
        // nothing provides widgets
        assert!(stack
            .read_first("assets/minecraft/textures/gui/widgets.png")
            .is_none());
    }

    #[test]
    fn empty_stack_reads_nothing() {
        let stack = PackStack::new();
        assert!(stack.is_empty());
        assert!(stack.read_first("pack.mcmeta").is_none());
    }

    /// a real zip built in-memory: the ZipSource reads the same paths a
    /// folder pack would (zip-vs-folder parity, D8)
    #[test]
    fn zip_source_resolves_the_same_paths_as_a_folder_pack() {
        // build a minimal zip via the `zip` writer? we have no writer —
        // hand-assemble a STORED (method 0) zip with one file.
        // layout: local header + data + central directory + EOCD
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
        let name = b"assets/minecraft/textures/gui/hearts.png";
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
            src.read("assets/minecraft/textures/gui/hearts.png"),
            Some(b"zip-heart".to_vec())
        );
        // folder parity: the same logical path through a MemorySource
        let mut folder_like = MemorySource::new("folder-like");
        folder_like.insert(
            "assets/minecraft/textures/gui/hearts.png",
            b"zip-heart".to_vec(),
        );
        assert_eq!(
            folder_like.read("assets/minecraft/textures/gui/hearts.png"),
            src.read("assets/minecraft/textures/gui/hearts.png")
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

    /// a wrong pack_format only WARNS (the repo's §46 policy — VERIFIED
    /// minecraft.wiki: pack_format 6 is 1.16.2-1.16.5, NOT the 5 the
    /// master prompt claims) and the pack still opens
    #[test]
    fn wrong_pack_format_warns_but_opens() {
        let mut mem = MemorySource::new("fmt");
        mem.insert(
            "pack.mcmeta",
            br#"{"pack":{"pack_format":99,"description":"future"}}"#.to_vec(),
        );
        let src: Arc<dyn PackSource> = Arc::new(mem);
        let r = open(src);
        assert!(r.is_ok(), "mismatched pack_format must not reject (§46)");
        let (meta, _) = r.unwrap_or((PackMeta {
            pack_format: 0,
            description: String::new(),
        }, Arc::new(MemorySource::new("x"))));
        assert_eq!(meta.pack_format, 99);
    }
}
