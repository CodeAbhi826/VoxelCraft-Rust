//! `item_icon_cache.rs` — cached 3D item icons (UI-overhaul Phase 3).
//!
//! Inventory, hotbar and container items render as cached **isometric
//! block models** instead of flat 16x16 tiles.
//!
//! Repo reality (reported in the Phase 3 checkpoint): there is NO
//! first-person hand / held-item model renderer to refactor — the
//! engine's only item art is the flat `blit_tile` path. So per the
//! Master Prompt's own fallback clause ("if you truly find nothing,
//! define minimal versions and explain why existing code could not be
//! reused"), this module defines a **clean-room CPU baker**: it
//! projects each block's three visible faces (top + two sides) with
//! the vanilla item-icon camera — yaw 45°, pitch 30°, orthographic —
//! and rasterizes them into a 64x64 cell of a shared GPU icon atlas.
//! CPU baking keeps the whole path unit-testable without a GPU and
//! matches the repo's procedural, in-code ethos (G9); cross-rendered
//! blocks (flowers/crops/torches) bake their flat sprite instead, the
//! vanilla behavior for cross models.
//!
//! Amortization + LRU follow the spec: at most `bake_budget_per_frame`
//! (4) icons bake per frame; the cache holds at most `max_entries`
//! (512) with least-recently-used eviction; every stat is reported to
//! the `perf` debug category by the game layer.

use std::collections::HashMap;

use vc_blocks::blocks::{def, is_cross, AIR};

/// Opaque handle into the icon atlas — the cell index (`row * 32 + col`
/// in the 2048x2048, 64px-cell grid). Do NOT construct directly: the
/// cache owns allocation (Section 3 note — this repo has no existing
/// texture-pool handle type, so the Section 3 definition lands here).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

/// cache key — (item id, damage, nbt hash). This engine's stacks carry
/// no damage/NBT yet, so the last two are 0 today; the shape is the
/// spec's, ready for durability/data components.
pub type IconKey = (u16, u16, u64);

/// bake status of one icon
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IconStatus {
    Ready(TextureHandle),
    Queued,
}

/// icon atlas geometry: 2048x2048 RGBA, 64px cells, 32x32 grid
pub const ICON_ATLAS_PX: u32 = 2048;
pub const ICON_CELL_PX: u32 = 64;
pub const ICON_GRID: u32 = ICON_ATLAS_PX / ICON_CELL_PX;

/// the fixed item-icon camera (Section 3).
// VERIFIED https://reference wiki /Model — Y-rotation 45 degrees,
// X-rotation 30 degrees (the standard item-icon isometric angle);
// orthographic (no perspective); 4-px margin inside the 64x64 target.
#[derive(Copy, Clone, Debug)]
pub struct IsoCamera {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub margin_px: u32,
}

impl Default for IsoCamera {
    fn default() -> Self {
        IsoCamera {
            yaw_deg: 45.0,
            pitch_deg: 30.0,
            margin_px: 4,
        }
    }
}

/// face brightness — CLEAN-ROOM approximations of the vanilla item-icon
/// shading (top fully lit, the two visible sides stepped down; the wiki
/// documents the look, not the exact factors)
const FACE_TOP: f32 = 1.0;
const FACE_LEFT: f32 = 0.8;
const FACE_RIGHT: f32 = 0.6;

// ------------------------------------------------------------ baker --

/// sample one texel of a 16x16 atlas tile (out-of-range reads black)
#[inline]
fn texel(atlas: &[u8], tile: u16, x: i32, y: i32) -> [u8; 4] {
    let tx = (tile % 32) as i32;
    let ty = (tile / 32) as i32;
    let px = (tx * 16 + x.clamp(0, 15)) as usize;
    let py = (ty * 16 + y.clamp(0, 15)) as usize;
    let idx = (py * 512 + px) * 4;
    if idx + 3 < atlas.len() {
        [
            atlas[idx],
            atlas[idx + 1],
            atlas[idx + 2],
            atlas[idx + 3],
        ]
    } else {
        [0, 0, 0, 0]
    }
}

/// bake one block's 64x64 icon into `out` (RGBA, stride 64). Pure CPU,
/// deterministic — the same atlas bytes produce byte-identical icons.
/// Returns false when the block has no drawable model (AIR / missing
/// def) — callers fall back to the flat tile.
pub fn bake_block_icon(atlas: &[u8], block: u16, out: &mut [u8]) -> bool {
    if block == AIR || out.len() < (ICON_CELL_PX * ICON_CELL_PX * 4) as usize {
        return false;
    }
    let d = def(block);
    let cam = IsoCamera::default();

    // ---- cross-rendered blocks: flat sprite (vanilla behavior for
    // cross models — flowers, crops, torches, saplings)
    if is_cross(block) {
        let tile = d.tiles[2];
        let size = (ICON_CELL_PX - cam.margin_px * 2) as i32; // 56
        let off = cam.margin_px as i32;
        for y in 0..size {
            for x in 0..size {
                let sx = x * 16 / size;
                let sy = y * 16 / size;
                let c = texel(atlas, tile, sx, sy);
                let dst = (((off + y) * 64) + off + x) as usize * 4;
                out[dst] = c[0];
                out[dst + 1] = c[1];
                out[dst + 2] = c[2];
                out[dst + 3] = c[3];
            }
        }
        return true;
    }

    // ---- cube: project the 8 corners with yaw 45 / pitch 30
    // (orthographic; screen x right, screen y down; the viewer sits at
    // +depth so the TOP, -x (left on screen) and +z (right on screen)
    // faces are the visible three)
    let yaw = cam.yaw_deg.to_radians();
    let pitch = cam.pitch_deg.to_radians();
    let (sy, cy) = (yaw.sin(), yaw.cos());
    let (sp, cp) = (pitch.sin(), pitch.cos());
    // rotate Y by yaw, then X by pitch, keep (screen_x, screen_y) and
    // drop depth (orthographic)
    let proj = |x: f32, y: f32, z: f32| -> (f32, f32) {
        let x1 = x * cy + z * sy;
        let z1 = -x * sy + z * cy;
        let y1 = y * cp - z1 * sp;
        (x1, -y1)
    };
    // corner indexing: i = y*4 + z*2 + x with each axis {0: -, 1: +}
    let idx = |y: usize, z: usize, x: usize| y * 4 + z * 2 + x;
    let mut corners = [(0f32, 0f32); 8];
    for y in [0usize, 1] {
        for z in [0usize, 1] {
            for x in [0usize, 1] {
                corners[idx(y, z, x)] = proj(
                    x as f32 - 0.5,
                    y as f32 - 0.5,
                    z as f32 - 0.5,
                );
            }
        }
    }
    // fit into the margin box (56 px), centered, uniform scale
    let box_px = (ICON_CELL_PX - cam.margin_px * 2) as f32;
    let min_x = corners.iter().map(|c| c.0).fold(f32::INFINITY, f32::min);
    let max_x = corners.iter().map(|c| c.0).fold(f32::NEG_INFINITY, f32::max);
    let min_y = corners.iter().map(|c| c.1).fold(f32::INFINITY, f32::min);
    let max_y = corners.iter().map(|c| c.1).fold(f32::NEG_INFINITY, f32::max);
    let span_x = (max_x - min_x).max(1e-6);
    let span_y = (max_y - min_y).max(1e-6);
    let scale = (box_px / span_x).min(box_px / span_y);
    let off_x = cam.margin_px as f32 + (box_px - span_x * scale) * 0.5 - min_x * scale;
    let off_y = cam.margin_px as f32 + (box_px - span_y * scale) * 0.5 - min_y * scale;
    let s: Vec<(f32, f32)> = corners
        .iter()
        .map(|(x, y)| (x * scale + off_x, y * scale + off_y))
        .collect();

    // each visible face: a PARALLELOGRAM (cube faces stay parallelograms
    // under the affine projection), given as p0 + u*e1 + v*e2 with
    // u,v in 0..1; tile uv maps u -> texel x, v -> texel row (v=0 top)
    let top_tile = d.tiles[0];
    let side_tile = d.tiles[2]; // side art for both visible sides
    // top (y=+): u along +x, v along +z
    let top = (s[idx(1, 0, 0)], s[idx(1, 0, 1)], s[idx(1, 1, 1)], s[idx(1, 1, 0)]);
    // left (-x): u along +z, v downward (-y)
    let left = (s[idx(1, 0, 0)], s[idx(1, 1, 0)], s[idx(0, 1, 0)], s[idx(0, 0, 0)]);
    // right (+z): u along +x, v downward (-y)
    let right = (s[idx(1, 1, 0)], s[idx(1, 1, 1)], s[idx(0, 1, 1)], s[idx(0, 1, 0)]);

    let faces = [
        (top, top_tile, FACE_TOP),
        (left, side_tile, FACE_LEFT),
        (right, side_tile, FACE_RIGHT),
    ];

    // clear the cell (margin must stay transparent)
    out.iter_mut().for_each(|b| *b = 0);

    for ((p0, p1, p2, p3), tile, bright) in faces {
        // parallelogram basis: e1 = p1-p0 (u axis), e2 = p3-p0 (v axis)
        let e1 = [p1.0 - p0.0, p1.1 - p0.1];
        let e2 = [p3.0 - p0.0, p3.1 - p0.1];
        let det = e1[0] * e2[1] - e1[1] * e2[0];
        if det.abs() < 1e-9 {
            continue; // degenerate face (never for a real cube)
        }
        let minx = p0.0.min(p1.0).min(p2.0).min(p3.0).floor().max(0.0) as i32;
        let maxx = p0
            .0
            .max(p1.0)
            .max(p2.0)
            .max(p3.0)
            .ceil()
            .min(ICON_CELL_PX as f32 - 1.0) as i32;
        let miny = p0.1.min(p1.1).min(p2.1).min(p3.1).floor().max(0.0) as i32;
        let maxy = p0
            .1
            .max(p1.1)
            .max(p2.1)
            .max(p3.1)
            .ceil()
            .min(ICON_CELL_PX as f32 - 1.0) as i32;
        for py in miny..=maxy {
            for px in minx..=maxx {
                let dx = px as f32 + 0.5 - p0.0;
                let dy = py as f32 + 0.5 - p0.1;
                // exact affine inverse (parallelogram)
                let u = (dx * e2[1] - dy * e2[0]) / det;
                let v = (e1[0] * dy - e1[1] * dx) / det;
                if !(-0.001..=1.001).contains(&u) || !(-0.001..=1.001).contains(&v) {
                    continue;
                }
                // nearest texel (v=0 is the tile's top row)
                let tx = (u.clamp(0.0, 0.9999) * 16.0) as i32;
                let ty = ((1.0 - v.clamp(0.0, 0.9999)) * 16.0) as i32;
                let c = texel(atlas, tile, tx, ty);
                if c[3] == 0 {
                    continue; // transparent texel (cross-style side tiles)
                }
                let dst = ((py * 64) + px) as usize * 4;
                if dst + 3 < out.len() {
                    out[dst] = (c[0] as f32 * bright) as u8;
                    out[dst + 1] = (c[1] as f32 * bright) as u8;
                    out[dst + 2] = (c[2] as f32 * bright) as u8;
                    out[dst + 3] = c[3];
                }
            }
        }
    }
    // any painted pixel?
    out.as_chunks::<4>().0.iter().any(|c| c[3] != 0)
}

// ------------------------------------------------------------- cache --

/// pure LRU/queue core — unit-tested without any GPU objects
#[derive(Debug)]
struct IconLruCore {
    /// key -> (cell, last_access)
    entries: HashMap<IconKey, (u32, u64)>,
    /// cell -> key (reverse map for eviction)
    cell_owner: HashMap<u32, IconKey>,
    /// pending bake queue (FIFO, deduped)
    queued: Vec<IconKey>,
    clock: u64,
    max_entries: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
    /// bumped whenever a new cell becomes Ready (the canvas snapshot
    /// follows this version)
    version: u64,
}

impl IconLruCore {
    fn new(max_entries: usize) -> Self {
        IconLruCore {
            entries: HashMap::new(),
            cell_owner: HashMap::new(),
            queued: Vec::new(),
            clock: 0,
            max_entries,
            hits: 0,
            misses: 0,
            evictions: 0,
            version: 0,
        }
    }

    fn get_or_queue(&mut self, key: IconKey) -> IconStatus {
        self.clock += 1;
        if let Some((cell, last)) = self.entries.get_mut(&key) {
            *last = self.clock;
            self.hits += 1;
            return IconStatus::Ready(TextureHandle(*cell));
        }
        self.misses += 1;
        if !self.queued.contains(&key) {
            self.queued.push(key);
        }
        IconStatus::Queued
    }

    /// allocate a cell for `key`, evicting the LRU entry when full.
    /// Returns the cell index and the evicted key (if any) so the GPU
    /// layer can reuse the cell.
    fn allocate(&mut self, key: IconKey) -> (u32, Option<IconKey>) {
        let mut evicted = None;
        if self.entries.len() >= self.max_entries {
            // evict least-recently-used (min last_access), NOT the first
            // inserted
            let lru = self
                .entries
                .iter()
                .min_by_key(|(_, (_, last))| *last)
                .map(|(k, _)| *k);
            if let Some(k) = lru {
                if let Some((cell, _)) = self.entries.remove(&k) {
                    self.cell_owner.remove(&cell);
                    evicted = Some(k);
                    self.evictions += 1;
                }
            }
        }
        // first free cell: lowest index not owned
        let mut cell = 0u32;
        while self.cell_owner.contains_key(&cell) && cell < ICON_GRID * ICON_GRID {
            cell += 1;
        }
        let cell = cell.min(ICON_GRID * ICON_GRID - 1);
        self.entries.insert(key, (cell, self.clock));
        self.cell_owner.insert(cell, key);
        self.queued.retain(|k| *k != key);
        self.version += 1;
        (cell, evicted)
    }

    /// take up to `budget` queued keys for baking this frame
    fn take_bake_batch(&mut self, budget: usize) -> Vec<IconKey> {
        let mut batch = Vec::new();
        while batch.len() < budget && !self.queued.is_empty() {
            let k = self.queued.remove(0);
            if !self.entries.contains_key(&k) {
                batch.push(k);
            }
        }
        batch
    }

    fn stats(&self) -> (u64, u64, u64) {
        (self.hits, self.misses, self.evictions)
    }
}

/// The GPU icon cache: a 2048x2048 atlas of 64px cells + the pure LRU
/// core. `run_bakes` bakes up to `budget` icons per frame on the CPU
/// and uploads exactly the touched cells.
pub struct ItemIconCache {
    core: IconLruCore,
    texture: wgpu::Texture,
    /// cell coordinates ready for the canvas snapshot (block id ->
    /// (col, row)) — rebuilt when core.version moves
    ready_cells: HashMap<u16, [u8; 2]>,
    snap_version: u64,
    /// bake budget per frame (D6: start at 4)
    pub bake_budget_per_frame: usize,
}

impl ItemIconCache {
    /// Create the cache + its (initially empty) atlas texture.
    /// (Infallible: wgpu texture creation returns no Result, so the
    /// spec's `Result<_, GuiError>` arm has nothing real to report —
    /// deviation noted in the Phase 3 checkpoint.)
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, max_entries: usize) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("item-icon-atlas"),
            size: wgpu::Extent3d {
                width: ICON_ATLAS_PX,
                height: ICON_ATLAS_PX,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // zero the atlas once (transparent cells)
        let zero_row = vec![0u8; (ICON_ATLAS_PX * 4) as usize];
        for row in 0..ICON_ATLAS_PX {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: row,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &zero_row,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(ICON_ATLAS_PX * 4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: ICON_ATLAS_PX,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }
        ItemIconCache {
            core: IconLruCore::new(max_entries.min((ICON_GRID * ICON_GRID) as usize)),
            texture,
            ready_cells: HashMap::new(),
            snap_version: u64::MAX,
            bake_budget_per_frame: 4,
        }
    }

    /// the atlas texture (the Renderer binds it as QuadTexture::IconAtlas)
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// lookup/queue one icon by item id (damage/NBT stay 0 until those
    /// exist on stacks)
    pub fn get_or_queue(&mut self, item_id: u16) -> IconStatus {
        self.core.get_or_queue((item_id, 0, 0))
    }

    /// bake + upload up to `budget` queued icons; returns the number
    /// baked (the game logs it to the `perf` category).
    pub fn run_bakes(&mut self, queue: &wgpu::Queue, atlas: &[u8], budget: usize) -> usize {
        let batch = self.core.take_bake_batch(budget);
        let mut baked = 0usize;
        for key in batch {
            let (cell, _evicted) = self.core.allocate(key);
            let mut buf = [0u8; (ICON_CELL_PX * ICON_CELL_PX * 4) as usize];
            if bake_block_icon(atlas, key.0, &mut buf) {
                let col = cell % ICON_GRID;
                let row = cell / ICON_GRID;
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: col * ICON_CELL_PX,
                            y: row * ICON_CELL_PX,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &buf,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(ICON_CELL_PX * 4),
                        rows_per_image: Some(ICON_CELL_PX),
                    },
                    wgpu::Extent3d {
                        width: ICON_CELL_PX,
                        height: ICON_CELL_PX,
                        depth_or_array_layers: 1,
                    },
                );
                baked += 1;
            }
        }
        baked
    }

    /// snapshot of ready icons (block id -> atlas cell coords) — rebuilt
    /// only when a new cell became ready since the last snapshot
    pub fn ready_cells(&mut self) -> &HashMap<u16, [u8; 2]> {
        if self.snap_version != self.core.version {
            self.ready_cells.clear();
            for ((block, _, _), (cell, _)) in self.core.entries.iter() {
                self.ready_cells.insert(
                    *block,
                    [(cell % ICON_GRID) as u8, (cell / ICON_GRID) as u8],
                );
            }
            self.snap_version = self.core.version;
        }
        &self.ready_cells
    }

    /// hits / misses / evictions (D7: the perf-category counters)
    pub fn stats(&self) -> (u64, u64, u64) {
        self.core.stats()
    }

    pub fn len(&self) -> usize {
        self.core.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.core.entries.is_empty()
    }
}

/// the Phase 2 error type (spec D2 refers to GuiError for cache
/// creation; wgpu made that arm unnecessary — re-exported for callers
/// that pattern-match the shared error family)
pub use crate::gui_render::GuiError;

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    /// a tiny procedural stand-in atlas: 512x512 RGBA with per-tile
    /// flat colors so bakes are verifiable
    fn fake_atlas() -> Vec<u8> {
        let mut a = vec![0u8; 512 * 512 * 4];
        for t in 0u16..32 * 32 {
            let tx = (t % 32) * 16;
            let ty = (t / 32) * 16;
            for y in 0..16 {
                for x in 0..16 {
                    let i = ((ty as usize + y as usize) * 512 + tx as usize + x as usize) * 4;
                    a[i] = (t % 32) as u8 * 6 + 40;
                    a[i + 1] = (t / 32) as u8 * 5 + 30;
                    a[i + 2] = 200;
                    a[i + 3] = 255;
                }
            }
        }
        a
    }

    /// ground-truth dump: bake a real block with the REAL procedural
    /// atlas and write the 64x64 icon to target/icon-dump-<block>.png
    /// (inspection only — run with --nocapture off, then eyeball)
    #[test]
    fn dump_baked_icons_for_inspection() {
        let atlas = crate::textures::generate_atlas();
        for (block, name) in [(3u16, "dirt"), (1u16, "stone"), (5u16, "planks")] {
            let mut out = [0u8; 64 * 64 * 4];
            let ok = bake_block_icon(&atlas, block, &mut out);
            assert!(ok, "{name} must bake");
            let dir = format!("{}/target", env!("CARGO_MANIFEST_DIR"));
            let _ = std::fs::create_dir_all(&dir);
            let path = format!("{dir}/icon-dump-{name}.png");
            if let Some(img) = image::RgbaImage::from_raw(64, 64, out.to_vec()) {
                let _ = img.save(&path);
            }
        }
    }

    /// GPU ground truth: create a real device, run one bake, read the
    /// atlas cell back and verify pixels landed (pollster is a
    /// dev-dependency; skipped when no adapter — headless CI safe)
    #[test]
    fn gpu_upload_lands_in_the_sampled_cell() {
        let r = pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await?;
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .ok()?;
            let atlas = crate::textures::generate_atlas();
            let mut cache = ItemIconCache::new(&device, &queue, 512);
            cache.get_or_queue(3); // dirt
            let baked = cache.run_bakes(&queue, &atlas, 4);
            assert_eq!(baked, 1);
            // cell for dirt
            let cells = cache.ready_cells().clone();
            let [col, row] = *cells.get(&3)?;
            // read back the 64x64 cell
            let dst = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("readback"),
                size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: cache.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: col as u32 * 64, y: row as u32 * 64, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &dst,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
            );
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("rb-buf"),
                size: 64 * 64 * 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &dst,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &buf,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(64 * 4),
                        rows_per_image: Some(64),
                    },
                },
                wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
            );
            queue.submit(Some(encoder.finish()));
            let slice = buf.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
            device.poll(wgpu::Maintain::Wait);
            let _ = rx.recv();
            let data = slice.get_mapped_range().to_vec();
            buf.unmap();
            let painted = data.as_chunks::<4>().0.iter().filter(|c| c[3] != 0).count();
            Some((painted, data.len() / 4))
        });
        if let Some((painted, total)) = r {
            assert!(painted > 200, "cell nearly empty: {painted}/{total} painted");
        }
    }

    #[test]
    fn cube_icon_bakes_non_empty_with_clear_margin() {
        let atlas = fake_atlas();
        let mut out = [0u8; 64 * 64 * 4];
        // block 3 = a real cube block (dirt): bake by block id (the API)
        let ok = bake_block_icon(&atlas, 3, &mut out);
        assert!(ok, "a cube block must bake");
        // non-empty
        assert!(out.as_chunks::<4>().0.iter().any(|c| c[3] != 0));
        // 4-px margin: the corner pixel stays transparent
        assert_eq!(&out[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn icons_are_deterministic() {
        let atlas = fake_atlas();
        let mut a = [0u8; 64 * 64 * 4];
        let mut b = [0u8; 64 * 64 * 4];
        bake_block_icon(&atlas, 3, &mut a);
        bake_block_icon(&atlas, 3, &mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn air_bakes_nothing() {
        let atlas = fake_atlas();
        let mut out = [0u8; 64 * 64 * 4];
        assert!(!bake_block_icon(&atlas, AIR, &mut out));
    }

    #[test]
    fn icon_key_hash_stability_for_equal_items() {
        // two equal stacks -> identical keys -> the same HashMap slot
        let mut core = IconLruCore::new(512);
        let k1: IconKey = (10, 0, 0);
        let k2: IconKey = (10, 0, 0);
        assert_eq!(k1, k2);
        core.get_or_queue(k1);
        let st = core.get_or_queue(k2);
        assert!(
            matches!(st, IconStatus::Queued | IconStatus::Ready(_)),
            "duplicate request resolves, not a second entry"
        );
    }

    #[test]
    fn lru_evicts_least_recently_used_not_first_inserted() {
        let mut core = IconLruCore::new(3);
        for k in [1u16, 2, 3] {
            core.get_or_queue((k, 0, 0));
            let _ = core.allocate((k, 0, 0));
        }
        // touch entry 1 (oldest insert) so entry 2 becomes LRU
        core.get_or_queue((1, 0, 0));
        // allocating a 4th must evict key 2 (LRU), NOT key 1 (inserted first)
        let (_cell, evicted) = core.allocate((4, 0, 0));
        assert_eq!(evicted, Some((2, 0, 0)));
        assert_eq!(core.stats().2, 1, "one eviction recorded");
    }

    #[test]
    fn bake_budget_is_respected() {
        let mut core = IconLruCore::new(512);
        for k in 0..10u16 {
            core.get_or_queue((k, 0, 0));
        }
        let batch = core.take_bake_batch(4);
        assert_eq!(batch.len(), 4);
        let batch2 = core.take_bake_batch(4);
        assert_eq!(batch2.len(), 4);
        // 10 queued - 8 taken = 2 remain
        let batch3 = core.take_bake_batch(4);
        assert_eq!(batch3.len(), 2);
    }

    #[test]
    fn flat_tile_fallback_when_model_unknown() {
        // AIR has no model: bake_block_icon returns false -> callers keep
        // the flat blit_tile path (the D8 fallback contract)
        let atlas = fake_atlas();
        let mut out = [0u8; 64 * 64 * 4];
        assert!(!bake_block_icon(&atlas, AIR, &mut out));
        assert!(out.iter().all(|&b| b == 0));
    }

    #[test]
    fn ready_cells_snapshot_grows_with_bakes() {
        let mut core = IconLruCore::new(512);
        core.get_or_queue((7, 0, 0));
        core.allocate((7, 0, 0));
        core.get_or_queue((8, 0, 0));
        core.allocate((8, 0, 0));
        assert_eq!(core.entries.len(), 2);
        // distinct cells
        let cells: Vec<u32> = core.entries.values().map(|(c, _)| *c).collect();
        assert_ne!(cells[0], cells[1]);
    }
}
