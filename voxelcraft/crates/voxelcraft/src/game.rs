//! GameApp: owns world/player/renderer/UI/audio; handles winit events
//! (native) and the JS input shim (wasm); screen flow:
//! Loading → Title ⇄ Options, Game ⇄ Pause/Options.
//! Streams chunks (rayon worker pool on native, time-budgeted inline on wasm).

use vc_blocks::blocks::*;
use vc_world::gen::Biome;
use vc_mesh::mesh::{mesh_sections, MeshData};
use crate::player::{raycast, Input, Player};
use vc_render::render::{Camera, RenderStats, Renderer, SkyState};
use vc_audio::sounds::{AudioBackend, SoundBank};
#[cfg(not(target_arch = "wasm32"))]
use vc_audio::sounds::native_audio;
#[cfg(target_arch = "wasm32")]
use vc_audio::sounds::web_audio;
use vc_render::ui::{self, UiCanvas, Widget, WidgetKind, UI_H, UI_W};
use vc_world::world::{ChunkPos, World};
use glam::Vec3;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

// --------------------------------------------------------------- settings --

#[derive(Clone)]
pub struct Settings {
    pub render_distance: i32,
    /// Phase 6 §26: simulation distance (chunk radius for the sim ring).
    /// NOT a 1.16.5 feature (dossier: Mojang 1.18+/26.x) — opt-in
    /// optimization. VERIFIED modern-vanilla range 5–32, default 12 ≥ the
    /// default render distances → 1.16.5-identical behavior by default.
    pub sim_distance: i32,
    pub sensitivity: f32,
    pub volume: f32,
    pub fov: f32,        // degrees, 30..110
    pub brightness: f32, // 0..1
    pub smooth_lighting: bool,
    pub clouds: bool,
    /// 0 = fast, 1 = fancy, 2 = fabulous (fancy + soft shadows + full post)
    pub graphics: u8,
    pub shader: u8, // 0 = off, 1 = vanilla+, 2 = cinematic
    /// §17 sun shadows: 0 = off, 1 = 1024px, 2 = 2048px, 3 = 4096px
    pub shadow_quality: u8,
    /// FSR 1.0 internal render scale index: 0 = 100%, 1 = 75%, 2 = 50%
    pub upscale: u8,
    /// §21 music category volume (0..1, master = `volume`)
    pub music_volume: f32,
    /// frame limiter: 0 = uncapped, else a fps ceiling (30/60/120)
    pub maxfps: u8,
    // ------------------------------------------ Phase 6 §26: rendering --
    /// mipmap levels 0–4 (vanilla `mipmapLevels`; VERIFIED default 4,
    /// range 0-4, exists since 1.7.2 — wiki Options.txt)
    pub mipmap_levels: u8,
    /// anisotropic filtering 1/2/4/8/16 (OptiFine `ofAfLevel` parity —
    /// vanilla 1.16.5 has no aniso setting; default 4 from the dossier
    /// Part 1 §3 captured optionsof.txt). 1 = effectively off
    pub aniso: u8,
    /// MSAA: 0 = off (vanilla-faithful), 4/8 (OptiFine `ofAaLevel` parity;
    /// 2x has no guaranteed WebGPU path — off/4/8 only, device-gated)
    pub msaa: u8,
    /// chunk-graph occlusion culling (OptiFine `ofOcclusionFancy` parity,
    /// default on)
    pub occlusion: bool,
    // ------------------------------------------------------ Phase 7 --
    /// GPU compute meshing (dossier Part 1 §2 gap "GPU compute: zero
    /// compute shaders"; §4 "bleeding-edge"). Engine optimization, NOT a
    /// vanilla 1.16.5 setting — no vanilla-parity default exists. Default:
    /// ON natively, OFF on wasm (SwiftShader compute measured slower than
    /// the inline CPU path; enable via options/E2E — see the Phase 7
    /// measurements). Falls back to CPU automatically when the adapter
    /// lacks compute or a chunk needs the cross/model special paths.
    pub gpu_meshing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            #[cfg(target_arch = "wasm32")]
            render_distance: 6,
            #[cfg(not(target_arch = "wasm32"))]
            render_distance: 10,
            sim_distance: 12,
            sensitivity: 1.0,
            volume: 0.7,
            fov: 70.0,
            brightness: 0.10,
            smooth_lighting: true,
            clouds: true,
            graphics: 1,
            shader: 1,
            shadow_quality: 2,
            upscale: 0,
            music_volume: 0.6,
            maxfps: 0,
            mipmap_levels: 4,
            aniso: 4,
            msaa: 0,
            occlusion: true,
            #[cfg(target_arch = "wasm32")]
            gpu_meshing: false,
            #[cfg(not(target_arch = "wasm32"))]
            gpu_meshing: true,
        }
    }
}

impl Settings {
    /// §17: shadow map resolution for the current quality (0 = off)
    pub fn shadow_map_px(&self) -> u32 {
        match self.shadow_quality {
            1 => 1024,
            2 => 2048,
            3 => 4096,
            _ => 2048, // quality 0 + graphics "fast" still renders at 2048
        }
    }
    /// effective shadow strength (fabulous = softer/stronger, fast = off)
    pub fn shadow_strength(&self) -> f32 {
        if self.shadow_quality == 0 || self.graphics == 0 {
            0.0
        } else if self.graphics == 2 {
            0.72
        } else {
            0.55
        }
    }
    /// effective internal render scale
    pub fn upscale_factor(&self) -> f32 {
        match self.upscale {
            1 => 0.75,
            2 => 0.5,
            _ => 1.0,
        }
    }
    /// effective frame cap (0 = uncapped)
    pub fn fps_cap(&self) -> f32 {
        match self.maxfps {
            1 => 30.0,
            2 => 60.0,
            3 => 120.0,
            _ => 0.0,
        }
    }
    /// serialize as k=v; pairs (parsed without serde)
    pub fn serialize(&self) -> String {
        format!(
            "rd={};sd={};sens={:.3};vol={:.3};mvol={:.3};fov={:.1};bright={:.3};smooth={};clouds={};graphics={};shader={};shadowq={};upscale={};maxfps={};mip={};aniso={};msaa={};occl={};gmesh={}",
            self.render_distance,
            self.sim_distance,
            self.sensitivity,
            self.volume,
            self.music_volume,
            self.fov,
            self.brightness,
            self.smooth_lighting as u8,
            self.clouds as u8,
            self.graphics,
            self.shader,
            self.shadow_quality,
            self.upscale,
            self.maxfps,
            self.mipmap_levels,
            self.aniso,
            self.msaa,
            self.occlusion as u8,
            self.gpu_meshing as u8
        )
    }
    pub fn deserialize(s: &str) -> Settings {
        let mut st = Settings::default();
        for pair in s.split(';') {
            let mut kv = pair.splitn(2, '=');
            let k = kv.next().unwrap_or("");
            let v = kv.next().unwrap_or("");
            match k {
                "rd" => st.render_distance = v.parse().unwrap_or(st.render_distance).clamp(2, 16),
                "sd" => st.sim_distance = v.parse().unwrap_or(st.sim_distance).clamp(5, 32),
                "sens" => st.sensitivity = v.parse().unwrap_or(st.sensitivity).clamp(0.1, 2.0),
                "vol" => st.volume = v.parse().unwrap_or(st.volume).clamp(0.0, 1.0),
                "mvol" => st.music_volume = v.parse().unwrap_or(st.music_volume).clamp(0.0, 1.0),
                "fov" => st.fov = v.parse().unwrap_or(st.fov).clamp(30.0, 110.0),
                "bright" => st.brightness = v.parse().unwrap_or(st.brightness).clamp(0.0, 1.0),
                "smooth" => st.smooth_lighting = v == "1",
                "clouds" => st.clouds = v == "1",
                // legacy key from older saves
                "fancy" => st.graphics = if v == "1" { 1 } else { 0 },
                "graphics" => st.graphics = v.parse().unwrap_or(st.graphics).min(2),
                "shader" => st.shader = v.parse().unwrap_or(st.shader).min(2),
                "shadowq" => st.shadow_quality = v.parse().unwrap_or(2).min(3),
                "upscale" => st.upscale = v.parse().unwrap_or(st.upscale).min(2),
                "maxfps" => st.maxfps = v.parse().unwrap_or(st.maxfps).min(3),
                "mip" => st.mipmap_levels = v.parse().unwrap_or(4).min(4),
                "aniso" => st.aniso = v.parse().unwrap_or(4).clamp(1, 16),
                "msaa" => {
                    // valid sample counts: 0 (off), 4, 8 — snap anything else
                    let v = v.parse::<u8>().unwrap_or(0);
                    st.msaa = if v >= 6 { 8 } else if v >= 2 { 4 } else { 0 };
                }
                "occl" => st.occlusion = v == "1",
                "gmesh" => st.gpu_meshing = v == "1",
                _ => {}
            }
        }
        st
    }
}

// ---------------------------------------------------------------- screens --

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Loading,
    Title,
    Options,
    Game,
    Pause,
    /// Phase 1: saved-world list (native)
    WorldSelect,
    /// Phase 1: new-world creation (name / seed / mode)
    WorldCreate,
    /// Phase 1: death screen (respawn vs hardcore game-over)
    Death,
}

/// open container screens (Phase 7 §27/§29)
#[derive(Clone, Copy, PartialEq)]
pub enum Container {
    /// player inventory with its 2×2 craft grid
    Inventory,
    /// crafting table at a world position with a 3×3 grid
    Crafting { pos: [i32; 3] },
    /// furnace UI bound to a block entity
    Furnace { pos: [i32; 3] },
    /// brewing-stand UI bound to a block entity (§29)
    Brewing { pos: [i32; 3] },
    /// enchanting-table UI bound to a block entity (§29)
    Enchant { pos: [i32; 3] },
    /// villager trade screen bound to a villager entity id (§27/§29)
    Trade { villager: u32 },
    /// Phase 3: chest container screen (27 slots)
    Chest { pos: [i32; 3] },
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Loading => "loading",
            Screen::Title => "title",
            Screen::Options => "options",
            Screen::Game => "game",
            Screen::Pause => "pause",
            Screen::WorldSelect => "worldselect",
            Screen::WorldCreate => "create",
            Screen::Death => "death",
        }
    }

    /// Full-screen UI screens that own the cursor (menus proper).
    pub fn is_menu(self) -> bool {
        matches!(
            self,
            Screen::Title | Screen::Options | Screen::Pause | Screen::WorldSelect | Screen::WorldCreate | Screen::Death
        )
    }
}

const SPLASHES: [&str; 14] = [
    "100% Rust!",
    "Also try Minecraft!",
    "Procedural everything!",
    "wgpu powered!",
    "Zero assets copied!",
    "Greedy meshed!",
    "Now with shaders!",
    "60 fps or bust!",
    "Made of cubes!",
    "WebGPU + WebGL2!",
    "Single-file engine!",
    "No build step... wait",
    "Blobs are friends!",
    "Ctrl+W to sprint!",
];

// ------------------------------------------------------------------ jobs --

enum Job {
    Gen { pos: ChunkPos, seed: u64, dim: vc_world::world::Dimension, inbound: Vec<(u16, u8)> },
    Mesh {
        pos: ChunkPos,
        snap: [Option<Arc<vc_chunk::chunk::Chunk>>; 9],
        lsnap: [Option<Arc<vc_world::light::LightData>>; 9],
        smooth: bool,
        /// sections to rebuild (§12 bitset; 0xFFFF = full chunk)
        mask: u16,
        /// cached section meshes to reuse for unmasked sections
        prev: Vec<Option<Arc<MeshData>>>,
        /// Phase 7: route through the GPU compute mesher (the submit site
        /// checks the setting + device capability; run_job falls back to
        /// the CPU path when the snapshot needs the cross/model paths)
        gpu: bool,
    },
}

enum JobResult {
    Gen { pos: ChunkPos, chunk: Arc<vc_chunk::chunk::Chunk>, outbound: Vec<(i32, i32, i32, u8)> },
    Mesh {
        pos: ChunkPos,
        /// the mask this job covered (dirty-bit clearing)
        mask: u16,
        /// new 16-slot section cache (fresh for masked, Arc clones for rest)
        sections: Vec<Option<Arc<MeshData>>>,
        /// merged per-chunk mesh for upload (§14 per-chunk merged buffers)
        mesh: Box<MeshData>,
        /// Phase 6 §26: occlusion-graph bits for this column (§26)
        occl: vc_render::draw::ChunkOccl,
    },
    /// Phase 7: the worker built the padded inputs and the chunk is
    /// greedy-path-eligible — the main thread hands it to the GPU
    /// compute mesher (all mesher state is main-thread-only, preserving
    /// the repo's zero-Mutex concurrency model)
    GpuMeshPending {
        pos: ChunkPos,
        mask: u16,
        smooth: bool,
        prev: Vec<Option<Arc<MeshData>>>,
        center: Option<Arc<vc_chunk::chunk::Chunk>>,
        inputs: vc_mesh::mesh::MeshInputs,
    },
}

/// Phase 6 §26: occlusion-graph bits for one chunk column, computed at
/// mesh time from the center snapshot + the fresh section meshes.
/// * walls: a section's 16×16×1 side face has ≥1 non-opaque cell
///   (empty/air sections are fully open — default to open, conservative)
/// * planes: the y = s·16+15 plane between bands s/s+1 has ≥1 non-opaque cell
/// * geo: the band's fresh mesh has indices (empty bands have nothing to
///   hide — the mesher culled interior faces against opaque neighbors)
fn chunk_occl(
    center: Option<&Arc<vc_chunk::chunk::Chunk>>,
    sections: &[Option<Arc<MeshData>>],
) -> vc_render::draw::ChunkOccl {
    use vc_render::draw::{ChunkOccl, FACE_NX, FACE_NZ, FACE_PX, FACE_PZ};
    let mut occl = ChunkOccl::default();
    // geometry bits (any meshed triangles → the band is worth drawing)
    for (b, s) in sections.iter().enumerate().take(16) {
        if let Some(m) = s {
            if !m.solid.1.is_empty() || !m.water.1.is_empty() {
                occl.geo |= 1u16 << b;
            }
        }
    }
    let Some(c) = center else {
        // no center chunk (defensive — the mesh job requires it): treat the
        // column as fully open so the cull can never hide it
        occl.sides = u64::MAX;
        occl.planes = u16::MAX;
        return occl;
    };
    for b in 0usize..16 {
        // empty section = all-air band: every wall + adjacent planes open
        if c.sections[b].is_none() {
            for f in 0u32..4 {
                occl.sides |= 1u64 << (b as u32 * 4 + f);
            }
            if b > 0 {
                occl.planes |= 1u16 << (b - 1); // plane between b-1 and b
            }
            if b < 15 {
                occl.planes |= 1u16 << b; // plane between b and b+1
            }
            continue;
        }
        let y0 = b * 16;
        // +X / -X walls: 16×16 cells each (x fixed, y × z varies)
        if (0..16usize).any(|dy| (0..16usize).any(|z| !is_opaque(c.get(15, y0 + dy, z)))) {
            occl.sides |= 1u64 << (b as u32 * 4 + FACE_PX as u32);
        }
        if (0..16usize).any(|dy| (0..16usize).any(|z| !is_opaque(c.get(0, y0 + dy, z)))) {
            occl.sides |= 1u64 << (b as u32 * 4 + FACE_NX as u32);
        }
        // +Z / -Z walls: 16×16 cells each (z fixed, y × x varies)
        if (0..16usize).any(|dy| (0..16usize).any(|x| !is_opaque(c.get(x, y0 + dy, 15)))) {
            occl.sides |= 1u64 << (b as u32 * 4 + FACE_PZ as u32);
        }
        if (0..16usize).any(|dy| (0..16usize).any(|x| !is_opaque(c.get(x, y0 + dy, 0)))) {
            occl.sides |= 1u64 << (b as u32 * 4 + FACE_NZ as u32);
        }
        // ceiling plane of this band (y = b·16+15) — only for b < 15
        if b < 15
            && (0..16usize).any(|x| (0..16usize).any(|z| !is_opaque(c.get(x, y0 + 15, z))))
        {
            occl.planes |= 1u16 << b;
        }
    }
    occl
}

fn run_job(job: Job) -> JobResult {
    match job {
        Job::Gen { pos, seed, dim, inbound } => {
            let gen = vc_world::gen::TerrainGen::for_dimension(seed, dim);
            let (chunk, outbound) = gen.generate_chunk(pos.0, pos.1, inbound);
            JobResult::Gen { pos, chunk, outbound }
        }
        Job::Mesh { pos, snap, lsnap, smooth, mask, prev, gpu } => {
            // Phase 7: GPU route — build the shared padded inputs on the
            // worker; greedy-eligible snapshots go to the compute mesher,
            // anything with cross plants / JSON-model states falls back to
            // the full CPU mesh (the special paths stay CPU — documented
            // hybrid scope)
            if gpu {
                let inputs = vc_mesh::mesh::build_mesh_inputs(&snap, &lsnap);
                if !inputs.has_cross && !inputs.has_models {
                    return JobResult::GpuMeshPending {
                        pos,
                        mask,
                        smooth,
                        prev,
                        center: snap[4].clone(),
                        inputs,
                    };
                }
            }
            let out = mesh_sections(pos, &snap, &lsnap, smooth, mask, &prev);
            // Phase 6 §26: occlusion-graph data rides the mesh result
            let occl = chunk_occl(snap[4].as_ref(), &out.sections);
            JobResult::Mesh {
                pos,
                mask,
                sections: out.sections,
                mesh: Box::new(out.merged),
                occl,
            }
        }
    }
}

enum WorkBackend {
    Threading {
        tx: std::sync::mpsc::Sender<JobResult>,
        rx: std::sync::mpsc::Receiver<JobResult>,
        inflight: usize,
    },
    Inline {
        jobs: VecDeque<Job>,
    },
}

// ------------------------------------------------------------------- app --

pub struct GameApp {
    pub window: &'static winit::window::Window,
    pub renderer: Renderer,
    pub world: World,
    pub player: Player,
    pub ui: UiCanvas,
    pub atlas: Vec<u8>,
    pub bank: SoundBank,
    /// §21 data-driven sound-event registry (parsed from sounds::SOUNDS_JSON)
    pub sounds: vc_audio::sounds::SoundRegistry,
    /// rng for weighted variant picks + pitch rolls + schedulers
    audio_rng: vc_rng::rng::Rng,
    /// sounds played this session (stats/E2E)
    pub sounds_played: u32,
    /// §21: next game-time a music pad starts (first at ~12 s, then every
    /// 2.5–4 min; day/night pick the progression)
    music_next: f32,
    /// §21: next game-time for the ambient cave-sound roll
    ambient_next: f32,
    pub audio: Box<dyn AudioBackend>,
    pub settings: Settings,
    work: WorkBackend,
    gen_inflight: HashSet<ChunkPos>,
    /// in-flight mesh jobs: pos → submitted section mask (bits added while
    /// a job runs survive via §12 clear_dirty_mask semantics)
    mesh_inflight: HashMap<ChunkPos, u16>,
    /// per-chunk cache of the 16 section meshes (§12 fine-grained remesh —
    /// worker jobs rebuild only dirty sections and reuse the rest)
    section_meshes: HashMap<ChunkPos, Vec<Option<Arc<MeshData>>>>,
    /// incremental light engine (Phase 4 §18)
    light: vc_world::light::LightEngine,
    /// fixed-step simulation (Phase 6: scheduled ticks, fluids, gravity,
    /// random ticks, item entities)
    sim: vc_sim::sim::Sim,
    /// open container screen (Phase 7): inventory crafting grid, crafting
    /// table, or furnace
    container: Option<Container>,
    /// hit-test geometry of the open container screen
    container_geom: Option<vc_render::ui::ContainerGeom>,
    /// stack held by the cursor in a container screen
    cursor_stack: vc_inventory::inventory::ItemStack,
    /// open crafting grid (2×2 uses [0..4] row-major on a 2-wide layout,
    /// 3×3 uses all 9)
    craft_grid: [vc_inventory::inventory::ItemStack; 9],
    /// block particles (Phase 5 §16.2 pass 4)
    particles: vc_particles::particles::ParticleSystem,
    /// billboard vertex scratch (rebuilt per frame against the camera basis)
    particle_verts: Vec<vc_particles::particles::ParticleVertex>,
    input: Input,
    pub screen: Screen,
    options_from: Screen, // where Options was opened from
    /// Phase 6 §26: options page (0 = general, 1 = video details)
    options_page: u8,
    widgets: Vec<Widget>,
    hover: Option<u16>,
    dragging: Option<u16>,
    cursor: (f32, f32), // UI-canvas coords
    quit_requested: bool,
    audio_unlocked: bool,
    day_time: f32,
    time: f32,
    target: Option<([i32; 3], u8, [i32; 3])>,
    break_timer: f32,
    place_timer: f32,
    show_debug: bool,
    show_help: bool,
    /// creative-style block picker overlay (E key)
    picker_open: bool,
    /// last pickr grid geometry for hit-testing clicks
    picker_geom: Option<vc_render::ui::PickerGeom>,
    /// rolling frame times (ms) for the F3 frame-time graph
    frame_times: std::collections::VecDeque<f32>,
    /// rolling (draw calls, buffer binds) per frame — Phase 9 §37 metric
    draw_calls_ring: std::collections::VecDeque<(u32, u32)>,
    item_toast: Option<(String, f32)>,
    last_ui_t: f32,
    last_frame_t: f32,
    last_draw_t: f32,
    fps: f32,
    frames: u32,
    fps_t: f32,
    /// rolling 100-frame window: min / avg / max fps + last frame ms
    fps_min: f32,
    fps_avg: f32,
    fps_max: f32,
    frame_ms: f32,
    /// game-time of the previous draw() (for the frame-time history)
    draw_game_t: f32,
    stats: RenderStats,
    spawn_snapped: bool,
    faced_land: bool,
    load_start: f32,
    edits: u32,
    stats_t: f32,
    pub pointer_locked: bool,
    pub drag_look: bool,
    ever_locked: bool,
    /// Phase-0 baseline instrumentation (§44): per-frame CPU phases
    pub phases: crate::bench::FramePhases,
    /// active in-game benchmark (§37/§48 Phase 0) — None in normal play
    pub bench: Option<crate::bench::BenchState>,
    /// spawn position captured at world init (bench camera orbits it)
    bench_spawn: glam::Vec3,
    /// Phase 11 §34: discovered shader packs (builtin + external)
    shader_packs: Vec<vc_render::shaders::ShaderPack>,
    /// Phase 8: Iris-format packs found in `shader-packs/` (native scan;
    /// wasm has no filesystem and boots empty). Structure-validated only —
    /// they are deliberately NOT in `shader_packs` because they cannot be
    /// applied: GLSL translation ships in the vc-iris sister project and
    /// plugs in through the IrisTranslator seam (vc-render/src/iris.rs).
    iris_packs: Vec<vc_render::iris::IrisPackInfo>,
    /// Phase 9: the active world's data packs (Mojang official format —
    /// recipes + loot tables + tags; scanned from `<world>/datapacks/`,
    /// folders AND zips). Wasm has no filesystem: boots empty and the
    /// E2E `dpdemo` command exercises the in-memory demo pack instead.
    data: vc_pack::datapack::LoadedData,
    /// pack-driven animated textures (frame updates only, no re-mesh)
    animations: Vec<vc_render::textures::AnimatedTile>,
    /// §28: root save dir (world root); `world_dir` is the CURRENT
    /// dimension's dir (overworld = root, nether = DIM-1)
    #[cfg(not(target_arch = "wasm32"))]
    save_root: std::path::PathBuf,
    /// world save directory (native, §28 — browsers get OPFS later)
    #[cfg(not(target_arch = "wasm32"))]
    world_dir: std::path::PathBuf,
    /// §28: a dimension travel is waiting for the spawn chunk (Loading)
    traveling: bool,
    /// Phase 1: a created/loaded world is waiting for the spawn chunk —
    /// Loading then goes straight into the game (not back to the title)
    pending_play: bool,
    /// Phase 1: the active game mode (rules gate, see vc-gameplay::modes)
    mode: vc_gameplay::modes::GameMode,
    /// Phase 1: display name of the active world (level.dat LevelName)
    world_name: String,
    /// Phase 1: a hardcore world whose player died — locked, no respawn
    hardcore_dead: bool,
    /// Phase 1: world spawn for respawn (both targets — web has no
    /// level.dat but still needs a respawn point)
    respawn_pos: glam::Vec3,
    /// Phase 1: last death cause shown on the death screen
    death_cause: String,
    /// Phase 1: world-create screen state (buffers + selected mode + the
    /// random seed preview shown as the placeholder)
    wc_name: String,
    wc_seed: String,
    wc_mode: vc_gameplay::modes::GameMode,
    wc_seed_preview: u64,
    /// Phase 1: cached world list for the select screen + selection
    /// (native only — the save module is fs-based and cfg'd out on wasm;
    /// the web build goes straight to world-create)
    #[cfg(not(target_arch = "wasm32"))]
    worlds: Vec<vc_anvil::save::WorldEntry>,
    ws_selected: Option<usize>,
    /// Phase 1: web text-entry shift state (codes arrive without case)
    web_shift: bool,
    /// Phase 2: seconds since the last melee swing (attack-cooldown
    /// recovery — feeds combat::cooldown_damage_scale)
    swing_t: f32,
    /// persisted spawn point (level.dat SpawnX/Y/Z)
    #[cfg(not(target_arch = "wasm32"))]
    level_spawn: (i32, i32, i32),
    /// seconds until the next autosave flush (20 s cadence, vanilla-like)
    #[cfg(not(target_arch = "wasm32"))]
    autosave_in: f32,
}

pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f32())
            .unwrap_or(0.0)
    }
    // CRITICAL: epoch seconds (~1.79e9) cannot be represented in f32 — the
    // 24-bit mantissa gives ~128 s resolution there, so every dt computed
    // from it is 0 (frozen clock: no physics, no menus, no fps, no toasts).
    // Use page uptime instead (starts at ~0, exact in f32 for days).
    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::OnceLock;
        static START: OnceLock<f64> = OnceLock::new();
        let start = *START.get_or_init(js_sys::Date::now);
        ((js_sys::Date::now() - start) / 1000.0) as f32
    }
}

/// Compile the builtin resource pack into a ModelSet and merge its textures
/// into a fresh procedural atlas (Phase 1, Master Spec §5.2/§19).
///
/// Native: reads `voxelcraft/assets/` from the working directory. Wasm:
/// fetches the same file set from `/assets/` (deployed by CI). Any failure
/// degrades to the procedural-only path with the missing-texture fallback
/// (§46 — an imperfect pack must never crash the engine).
async fn load_builtin_pack_assets() -> (Vec<u8>, Vec<vc_render::textures::AnimatedTile>) {
    let mut atlas = vc_render::textures::generate_atlas();

    // 1. acquire the pack source
    #[cfg(not(target_arch = "wasm32"))]
    let source: Option<std::sync::Arc<dyn vc_pack::pack::PackSource>> = {
        let folder = vc_pack::pack::FolderSource::new("builtin-pack", "builtin");
        if folder.exists() {
            match vc_pack::pack::open(std::sync::Arc::new(folder)) {
                Ok((meta, src)) => {
                    vc_render::render::report_boot_log(&format!(
                        "builtin pack: {} (format {}, {})",
                        src.name(),
                        meta.pack_format,
                        meta.description
                    ));
                    Some(src)
                }
                Err(e) => {
                    vc_render::render::report_boot_log(&format!("builtin pack unavailable: {e}"));
                    None
                }
            }
        } else {
            vc_render::render::report_boot_log("no builtin pack folder (builtin-pack/) — procedural fallback");
            None
        }
    };
    #[cfg(target_arch = "wasm32")]
    let source: Option<std::sync::Arc<dyn vc_pack::pack::PackSource>> = {
        let specs: Vec<vc_pack::model::BlockDispatchSpec> = vc_blocks::blocks::PROP_BLOCKS
            .iter()
            .map(|pb| vc_pack::model::BlockDispatchSpec {
                name: pb.name,
                props: pb.props,
                base_state: pb.base_state,
                state_count: pb.state_count,
            })
            .collect();
        match vc_pack::pack::fetch_builtin_pack(&specs).await {
            Some(mem) => match vc_pack::pack::open(std::sync::Arc::new(mem)) {
                Ok((meta, src)) => {
                    vc_render::render::report_boot_log(&format!(
                        "builtin pack fetched: {} (format {})",
                        src.name(),
                        meta.pack_format
                    ));
                    Some(src)
                }
                Err(e) => {
                    vc_render::render::report_boot_log(&format!("builtin pack fetch failed: {e}"));
                    None
                }
            },
            None => {
                vc_render::render::report_boot_log("no builtin pack on server — procedural fallback");
                None
            }
        }
    };

    let Some(source) = source else {
        // no pack: still install an empty ModelSet so model-state blocks
        // render the missing texture instead of being skipped silently
        vc_pack::model::install(vc_pack::model::ModelSet {
            by_state: Default::default(),
            tiles: Default::default(),
        });
        return (atlas, Vec::new());
    };

    // 2. compile per-block dispatches (parse once, canonicalize, cache)
    let mut by_state = std::collections::HashMap::new();
    for pb in vc_blocks::blocks::PROP_BLOCKS.iter() {
        let spec = vc_pack::model::BlockDispatchSpec {
            name: pb.name,
            props: pb.props,
            base_state: pb.base_state,
            state_count: pb.state_count,
        };
        match vc_pack::model::compile_block_dispatch(&spec, &|p| source.read(p)) {
            Ok(map) => {
                by_state.extend(map);
            }
            Err(e) => {
                // §46: one bad blockstate must not take the engine down
                vc_render::render::report_boot_log(&format!(
                    "blockstate {name} failed: {e} — block will use the missing model",
                    name = pb.name
                ));
            }
        }
    }
    let mut set = vc_pack::model::ModelSet {
        by_state,
        tiles: Default::default(),
    };

    // 3. merge pack textures into the atlas (fills set.tiles + animations)
    let animations = vc_render::textures::merge_pack_textures(&mut atlas, &mut set, source.as_ref());
    let n_models: usize = set.by_state.values().map(|v| v.len()).sum();
    vc_render::render::report_boot_log(&format!(
        "model dispatch: {} states, {} applied models, {} pack textures, {} animations",
        set.by_state.len(),
        n_models,
        set.tiles.len(),
        animations.len()
    ));
    vc_pack::model::install(set);
    (atlas, animations)
}

impl GameApp {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        // ---------------------------------------------------- Phase 1 assets
        // Compile the builtin resource pack (blockstates → models → textures)
        // BEFORE any mesh job can run; merge its textures into the atlas.
        let (mut atlas, animations) = crate::game::load_builtin_pack_assets().await;
        vc_render::textures::draw_missing_tile(&mut atlas);

        let mut renderer = Renderer::new(window, &atlas).await;
        let bank = SoundBank::generate();
        let sounds = vc_audio::sounds::SoundRegistry::from_json(vc_audio::sounds::SOUNDS_JSON)
            .unwrap_or_else(|e| {
                vc_render::render::report_boot_log(&format!("sound registry broken: {e}"));
                // empty registry = silent game rather than a boot failure
                vc_audio::sounds::SoundRegistry { events: Default::default() }
            });
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut world = World::new(vc_world::world::World::random_seed());
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut spawn = world.find_spawn();
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));
        // Phase 1: default state until a world is created/loaded
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut mode = vc_gameplay::modes::GameMode::Survival;
        let mut world_name = String::from("VoxelCraft");

        // native: scan every saved world (Phase 1) and restore the most
        // recently played one (panorama background + fast re-entry); the
        // legacy single save at saves/VoxelCraft is simply one entry.
        // §28: the overworld saves at the world root (boot always starts
        // there, like vanilla); the nether dir is derived on travel.
        #[cfg(not(target_arch = "wasm32"))]
        let save_root = {
            let worlds = vc_anvil::save::list_worlds();
            if let Some(newest) = worlds.first() {
                let meta = &newest.meta;
                world = World::new(meta.seed);
                spawn = world.find_spawn();
                mode = vc_gameplay::modes::GameMode::from_save(meta.game_type, meta.hardcore);
                world_name = meta.name.clone();
                if let Some(p) = &meta.player {
                    player = Player::new(Vec3::new(
                        p.pos[0] as f32,
                        p.pos[1] as f32,
                        p.pos[2] as f32,
                    ));
                    player.yaw = p.yaw;
                    player.pitch = p.pitch;
                } else {
                    player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));
                }
                newest.dir.clone()
            } else {
                vc_anvil::save::default_world_dir()
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        let world_dir = vc_anvil::save::dimension_dir(
            &save_root,
            vc_world::world::Dimension::Overworld,
        );
        #[cfg(not(target_arch = "wasm32"))]
        let mut level_spawn = (spawn.0 as i32, spawn.1 as i32, spawn.2 as i32);
        #[cfg(not(target_arch = "wasm32"))]
        {
            // the restored save's own spawn point wins over find_spawn()
            if let Ok(Some(meta)) = vc_anvil::save::read_level_dat(&save_root) {
                level_spawn = meta.spawn;
            }
        }

        // persisted settings (web: localStorage)
        let settings = {
            #[cfg(target_arch = "wasm32")]
            {
                crate::web_input::load_settings()
                    .map(|s| Settings::deserialize(&s))
                    .unwrap_or_default()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                Settings::default()
            }
        };
        // (player was already built above — restored from level.dat on native
        // when a save exists, else spawn-positioned for a fresh world)
        player.fov = settings.fov.to_radians();
        player.fov_cur = player.fov;

        // apply persisted render scale (FSR 1.0 EASU) before the first frame
        renderer.set_upscale(settings.upscale_factor());
        // §17: apply persisted shadow quality
        renderer.set_shadow_quality(settings.shadow_map_px());
        // Phase 6 §26: apply persisted texture quality (mipmaps + aniso),
        // MSAA, and the occlusion-culling toggle before frame 1
        renderer.set_texture_quality(settings.mipmap_levels, settings.aniso);
        renderer.set_msaa(settings.msaa);
        renderer.set_occlusion(settings.occlusion);

        // Phase 11 §34: discover shader packs (builtin embedded + native
        // external dir) and apply the persisted selection before frame 1
        let mut shader_packs = vc_render::shaders::builtin_packs();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut ext = vc_render::shaders::external_packs();
            shader_packs.append(&mut ext);
        }
        if let Some(n) = shader_mode_pack_index(settings.shader, shader_packs.len()) {
            renderer.set_shader_pack(shader_packs.get(n).map(|p| p));
            if let Some(p) = shader_packs.get(n) {
                vc_render::render::report_boot_log(&format!(
                    "shader pack active: {} ({})",
                    p.name, p.tier
                ));
            }
        }

        // Phase 8: scan the same shader-packs/ root for Iris-format packs
        // (dirs carrying shaders.properties). Each is fully analyzed and
        // reported HONESTLY: structure-validated, not selectable — the
        // GLSL-330 translation lives in the vc-iris sister project and
        // registers itself through the IrisTranslator seam. Web builds
        // have no filesystem: the list stays empty and the E2E `iris`
        // command exercises the wasm-reachable surface instead.
        #[cfg(not(target_arch = "wasm32"))]
        let iris_packs = {
            let packs = vc_render::iris::scan_shader_packs(std::path::Path::new("shader-packs"));
            for p in &packs {
                vc_render::render::report_boot_log(&format!(
                    "iris pack detected: {} — structure-validated, not selectable \
                     (GLSL translation ships in the sister project vc-iris)",
                    p.summary()
                ));
            }
            packs
        };
        #[cfg(target_arch = "wasm32")]
        let iris_packs = Vec::new();

        // Phase 9: scan the restored world's data packs (recipes + loot
        // tables + tags, Mojang's official format — folders AND zips).
        // Must happen before the world starts generating: dungeon-chest
        // loot rolls through the loaded tables. wasm has no filesystem —
        // `data` boots empty and the E2E `dpdemo` command runs the
        // in-memory demo pack through the same code path.
        #[cfg(not(target_arch = "wasm32"))]
        let data = {
            let root = save_root.join("datapacks");
            let loaded = vc_pack::datapack::scan_datapacks(&root);
            report_datapacks(&loaded);
            loaded
        };
        #[cfg(target_arch = "wasm32")]
        let data = vc_pack::datapack::LoadedData::default();

        let work = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = std::sync::mpsc::channel();
                WorkBackend::Threading { tx, rx, inflight: 0 }
            }
            #[cfg(target_arch = "wasm32")]
            {
                WorkBackend::Inline { jobs: VecDeque::new() }
            }
        };

        let audio: Box<dyn AudioBackend> = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                match native_audio::RodioOut::new() {
                    Some(o) => Box::new(o),
                    None => Box::new(vc_audio::sounds::SilentOut),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                Box::new(web_audio::WebAudioOut::new())
            }
        };

        let mut app = GameApp {
            window,
            renderer,
            world,
            player,
            ui: UiCanvas::new(),
            atlas,
            bank,
            sounds,
            shader_packs,
            iris_packs,
            data,
            audio_rng: vc_rng::rng::Rng::new(0x50_0D_5EED),
            sounds_played: 0,
            music_next: 12.0,
            ambient_next: 4.0,
            audio,
            settings,
            work,
            gen_inflight: HashSet::new(),
            mesh_inflight: HashMap::new(),
            section_meshes: HashMap::new(),
            light: vc_world::light::LightEngine::new(),
            sim: vc_sim::sim::Sim::new(0xC0FF_EE01),
            container: None,
            container_geom: None,
            cursor_stack: vc_inventory::inventory::ItemStack::EMPTY,
            craft_grid: [vc_inventory::inventory::ItemStack::EMPTY; 9],
            particles: vc_particles::particles::ParticleSystem::new(0x5EED_0042),
            particle_verts: Vec::new(),
            input: Input::default(),
            screen: Screen::Loading,
            options_from: Screen::Title,
            options_page: 0,
            widgets: Vec::new(),
            hover: None,
            dragging: None,
            cursor: (UI_W as f32 / 2.0, UI_H as f32 / 2.0),
            quit_requested: false,
            audio_unlocked: false,
            day_time: 0.30,
            time: now_secs(),
            target: None,
            break_timer: 0.0,
            place_timer: 0.0,
            show_debug: false,
            show_help: false,
            picker_open: false,
            picker_geom: None,
            frame_times: std::collections::VecDeque::new(),
            draw_calls_ring: std::collections::VecDeque::new(),
            item_toast: None,
            last_ui_t: -1.0,
            last_frame_t: now_secs(),
            last_draw_t: 0.0,
            fps: 0.0,
            frames: 0,
            fps_t: now_secs(),
            fps_min: 0.0,
            fps_avg: 0.0,
            fps_max: 0.0,
            frame_ms: 0.0,
            draw_game_t: 0.0,
            stats: RenderStats::default(),
            spawn_snapped: false,
            faced_land: false,
            load_start: 0.0,
            edits: 0,
            stats_t: 0.0,
            pointer_locked: false,
            drag_look: false,
            ever_locked: false,
            phases: crate::bench::FramePhases::new(240),
            bench: None,
            bench_spawn: spawn.into(),
            animations,
            #[cfg(not(target_arch = "wasm32"))]
            save_root,
            #[cfg(not(target_arch = "wasm32"))]
            world_dir,
            traveling: false,
            pending_play: false,
            mode,
            world_name,
            hardcore_dead: false,
            respawn_pos: {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    glam::Vec3::new(
                        level_spawn.0 as f32 + 0.5,
                        level_spawn.1 as f32 + 0.5,
                        level_spawn.2 as f32 + 0.5,
                    )
                }
                #[cfg(target_arch = "wasm32")]
                {
                    glam::Vec3::new(spawn.0, spawn.1 + 1.0, spawn.2)
                }
            },
            death_cause: String::new(),
            wc_name: String::from("New World"),
            wc_seed: String::new(),
            wc_mode: vc_gameplay::modes::GameMode::Survival,
            wc_seed_preview: vc_world::world::World::random_seed(),
            #[cfg(not(target_arch = "wasm32"))]
            worlds: Vec::new(),
            ws_selected: None,
            web_shift: false,
            swing_t: 99.0,
            #[cfg(not(target_arch = "wasm32"))]
            level_spawn,
            #[cfg(not(target_arch = "wasm32"))]
            autosave_in: 20.0,
        };
        // Phase 5: restore container inventories (dungeon loot + the
        // player's touched chests/hoppers) into the fresh sim — native
        // only (web sessions regenerate; containers there are transient)
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(Some(meta)) = vc_anvil::save::read_level_dat(&app.world_dir) {
            for c in meta.containers {
                let inv = app.sim.containers.entry(c.pos, c.kind);
                for (slot, block, count) in c.slots {
                    if let Some(s) = inv.slots.get_mut(slot as usize) {
                        *s = vc_inventory::inventory::ItemStack::new(block, count);
                    }
                }
            }
        }
        app.load_start = app.time;
        app.refresh_widgets();
        app
    }

    // ------------------------------------------------------------- events --

    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        use winit::event::{Event, WindowEvent};
        #[cfg(not(target_arch = "wasm32"))]
        use winit::event::{ElementState, MouseScrollDelta};
        #[cfg(not(target_arch = "wasm32"))]
        use winit::keyboard::PhysicalKey;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    #[cfg(not(target_arch = "wasm32"))]
                    if self.bench.is_none() {
                        self.save_world(); // final flush on window close (§28)
                    }
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    self.renderer.resize(size.width, size.height);
                    self.ui.dirty = true;
                }
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    // Phase 1: text fields eat printable characters first
                    // (world name / seed entry on the create screen)
                    if pressed
                        && self.screen == Screen::WorldCreate
                        && self.text_field_focused().is_some()
                    {
                        if let winit::keyboard::Key::Character(s) = &event.logical_key {
                            let mut ate = false;
                            for ch in s.chars() {
                                if self.type_char(ch) {
                                    ate = true;
                                }
                            }
                            if ate {
                                return;
                            }
                        }
                    }
                    let code = match event.physical_key {
                        PhysicalKey::Code(c) => c,
                        _ => return,
                    };
                    self.key_action(code, pressed, false);
                }
                #[cfg(target_arch = "wasm32")]
                WindowEvent::KeyboardInput { .. } => {
                    // handled by the JS input shim (focus-independent)
                }
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = state == ElementState::Pressed;
                    let (cx, cy) = (self.cursor.0 as i32, self.cursor.1 as i32);
                    if self.container.is_some() && self.screen == Screen::Game {
                        if pressed {
                            let right = button == winit::event::MouseButton::Right;
                            self.container_click(cx, cy, right);
                        }
                    } else if self.picker_open && self.screen == Screen::Game {
                        if pressed {
                            self.picker_click(cx, cy);
                        }
                    } else if self.screen == Screen::Game {
                        self.game_mouse(button, pressed);
                    } else {
                        self.menu_mouse(button, pressed, cx, cy);
                    }
                }
                #[cfg(target_arch = "wasm32")]
                WindowEvent::MouseInput { .. } => {
                    // handled by the JS input shim
                }
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::CursorMoved { position, .. } => {
                    let (ux, uy) = self.phys_to_ui(position.x as f32, position.y as f32);
                    self.cursor = (ux, uy);
                    self.update_hover();
                    if self.dragging.is_some() {
                        self.drag_slider(ux);
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::MouseWheel { delta, .. } => {
                    let d = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                    };
                    self.wheel(d);
                }
                #[cfg(target_arch = "wasm32")]
                WindowEvent::MouseWheel { .. } => {
                    // handled by the JS input shim
                }
                WindowEvent::RedrawRequested => {
                    // optional frame limiter: skip draws that arrive too soon
                    // (update still runs at full RAF rate — only drawing is
                    // throttled, which is where the GPU time goes)
                    let cap = self.settings.fps_cap();
                    let now = now_secs();
                    if cap > 0.0 && now - self.last_draw_t < (1.0 / cap) - 0.001 {
                        self.window.request_redraw();
                        return;
                    }
                    self.last_draw_t = now;
                    self.draw();
                }
                WindowEvent::Focused(false) => {
                    self.input = Input::default();
                    if self.screen == Screen::Game {
                        self.enter_pause();
                    }
                }
                _ => {}
            },
            #[cfg(not(target_arch = "wasm32"))]
            Event::DeviceEvent { event, .. } => {
                use winit::event::DeviceEvent;
                if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                    if self.screen == Screen::Game {
                        self.input.add_mouse(dx as f32, dy as f32);
                    }
                }
            }
            Event::AboutToWait => {
                let now = now_secs();
                let dt = (now - self.last_frame_t).clamp(0.0, 0.1);
                self.last_frame_t = now;
                #[cfg(target_arch = "wasm32")]
                self.poll_web_events();
                self.update(dt);
                if self.quit_requested {
                    #[cfg(not(target_arch = "wasm32"))]
                    if self.bench.is_none() {
                        self.save_world(); // QUIT GAME button — final flush
                    }
                    elwt.exit();
                    return;
                }
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    // -------------------------------------------------- web input bridge --

    #[cfg(target_arch = "wasm32")]
    fn poll_web_events(&mut self) {
        use crate::web_input::{self, WebEvent};
        use winit::event::MouseButton;
        for ev in web_input::drain_events() {
            match ev {
                WebEvent::Key { code, pressed, repeat } => {
                    // Phase 1: shift state for web text case mapping
                    if code == "ShiftLeft" || code == "ShiftRight" {
                        self.web_shift = pressed;
                    }
                    // text fields eat printable keys (codes → chars, with
                    // shift for case — the shim sends physical codes)
                    if pressed
                        && !repeat
                        && self.screen == Screen::WorldCreate
                        && self.text_field_focused().is_some()
                    {
                        if let Some(ch) = web_char_from_code(&code, self.web_shift) {
                            if self.type_char(ch) {
                                continue;
                            }
                        }
                    }
                    if let Some(kc) = keycode_from_web(&code) {
                        self.key_action(kc, pressed, repeat);
                    }
                }
                WebEvent::MouseDelta { dx, dy } => {
                    if self.screen == Screen::Game && !self.picker_open && self.container.is_none() {
                        self.input.add_mouse(dx, dy);
                    }
                }
                WebEvent::Cursor { x, y } => {
                    let (ux, uy) = self.css_to_ui(x, y);
                    self.cursor = (ux, uy);
                    if self.screen != Screen::Game {
                        self.update_hover();
                        if self.dragging.is_some() {
                            self.drag_slider(ux);
                        }
                    }
                }
                WebEvent::Button { button, pressed, x, y } => {
                    if self.container.is_some() && self.screen == Screen::Game {
                        if pressed {
                            let (ux, uy) = self.css_to_ui(x, y);
                            self.container_click(ux as i32, uy as i32, button == 2);
                        }
                    } else if self.picker_open && self.screen == Screen::Game {
                        if pressed {
                            let (ux, uy) = self.css_to_ui(x, y);
                            self.picker_click(ux as i32, uy as i32);
                        }
                    } else if self.screen == Screen::Game {
                        // drag-look fallback path (pointer lock unavailable)
                        let b = match button {
                            0 => MouseButton::Left,
                            1 => MouseButton::Middle,
                            2 => MouseButton::Right,
                            _ => continue,
                        };
                        self.game_mouse(b, pressed);
                    } else {
                        let (ux, uy) = self.css_to_ui(x, y);
                        self.cursor = (ux, uy);
                        let b = match button {
                            0 => MouseButton::Left,
                            1 => MouseButton::Middle,
                            2 => MouseButton::Right,
                            _ => continue,
                        };
                        self.menu_mouse(b, pressed, ux as i32, uy as i32);
                    }
                }
                WebEvent::Wheel { dir } => self.wheel(dir),
                WebEvent::LockChange { locked } => {
                    self.pointer_locked = locked;
                    if locked {
                        self.ever_locked = true;
                    } else if self.screen == Screen::Game
                        && !self.picker_open
                        && self.container.is_none()
                    {
                        // browser released the lock (Esc) → pause menu.
                        // (opening a container releases the lock on purpose —
                        // that is NOT a pause)
                        self.enter_pause();
                    }
                }
                WebEvent::LockError => {
                    // If we have NEVER locked successfully, the browser is
                    // blocking pointer lock (permissions policy in a nested
                    // iframe) → engage the drag-to-look fallback. If we have
                    // locked before, this is a transient failure (e.g. Esc
                    // keypress has no activation) — the next click re-locks.
                    if !self.ever_locked {
                        self.drag_look = true;
                    }
                }
                WebEvent::Resize { w, h } => {
                    self.renderer.resize(w, h);
                    self.ui.dirty = true;
                    self.draw(); // redraw in the same tick (anti-flicker)
                }
                WebEvent::Blur => {
                    self.input = Input::default();
                }
                WebEvent::Visibility { hidden } => {
                    if hidden && self.screen == Screen::Game {
                        self.enter_pause();
                    }
                }
            }
        }
    }

    // ------------------------------------------------------- coordinates --

    fn phys_to_ui(&self, x: f32, y: f32) -> (f32, f32) {
        let (sw, sh) = self.renderer.size();
        let scale = (sw / UI_W as f32).min(sh / UI_H as f32);
        let x0 = (sw - UI_W as f32 * scale) * 0.5;
        let y0 = (sh - UI_H as f32 * scale) * 0.5;
        ((x - x0) / scale, (y - y0) / scale)
    }

    #[cfg(target_arch = "wasm32")]
    fn css_to_ui(&self, x: f32, y: f32) -> (f32, f32) {
        let dpr = self.window.scale_factor() as f32;
        self.phys_to_ui(x * dpr, y * dpr)
    }

    // ------------------------------------------------------------ input --

    fn key_action(&mut self, code: winit::keyboard::KeyCode, pressed: bool, repeat: bool) {
        use winit::keyboard::KeyCode;
        // movement only when actually in the game world (not in the picker
        // or a container screen)
        let in_game = self.screen == Screen::Game && !self.picker_open && self.container.is_none();
        match code {
            KeyCode::KeyW => self.input.fwd = pressed && in_game,
            KeyCode::KeyS => self.input.back = pressed && in_game,
            KeyCode::KeyA => self.input.left = pressed && in_game,
            KeyCode::KeyD => self.input.right = pressed && in_game,
            KeyCode::Space => {
                self.input.jump = pressed && in_game;
                // Phase 1: double-space flight is a Creative-only mechanic
                if pressed && in_game && self.mode.allows_flight() {
                    self.player.try_fly_toggle(self.time);
                }
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.input.sneak = pressed && in_game,
            KeyCode::ControlLeft | KeyCode::ControlRight => self.input.sprint = pressed && in_game,
            KeyCode::Backspace => {
                // Phase 1: text-field editing (world name / seed)
                if pressed && self.screen == Screen::WorldCreate {
                    self.backspace_field();
                }
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                // Phase 1: Enter on the create screen = CREATE
                if pressed && self.screen == Screen::WorldCreate {
                    self.create_world();
                }
            }
            KeyCode::Escape => {
                if pressed {
                    match self.screen {
                        Screen::Game => {
                            if self.container.is_some() {
                                self.close_container();
                            } else if self.picker_open {
                                self.close_picker();
                            } else {
                                self.enter_pause();
                            }
                        }
                        Screen::Pause => self.resume_game(),
                        Screen::Options => self.close_options(),
                        Screen::WorldCreate => self.cancel_world_create(),
                        Screen::WorldSelect => self.set_screen(Screen::Title),
                        _ => {}
                    }
                }
            }
            KeyCode::KeyE => {
                if pressed && !repeat && self.screen == Screen::Game {
                    if self.container.is_some() {
                        self.close_container();
                    } else if self.picker_open {
                        self.close_picker();
                    } else {
                        self.open_container(Container::Inventory);
                    }
                }
            }
            KeyCode::KeyB => {
                if pressed && !repeat && self.screen == Screen::Game {
                    if self.picker_open {
                        self.close_picker();
                    } else {
                        self.open_picker();
                    }
                }
            }
            KeyCode::F3 => {
                if pressed && !repeat && self.screen == Screen::Game {
                    self.show_debug = !self.show_debug;
                    self.ui.dirty = true;
                }
            }
            KeyCode::KeyH => {
                if pressed && !repeat && in_game {
                    self.show_help = !self.show_help;
                    self.ui.dirty = true;
                }
            }
            KeyCode::BracketLeft => {
                if pressed && in_game && self.settings.render_distance > 3 {
                    self.settings.render_distance -= 1;
                    self.ui.dirty = true;
                }
            }
            KeyCode::BracketRight => {
                if pressed && in_game && self.settings.render_distance < 16 {
                    self.settings.render_distance += 1;
                    self.ui.dirty = true;
                }
            }
            KeyCode::Minus => {
                if pressed && in_game {
                    self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                    self.ui.dirty = true;
                }
            }
            KeyCode::Equal => {
                if pressed && in_game {
                    self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                    self.ui.dirty = true;
                }
            }
            KeyCode::KeyV => {
                if pressed && in_game {
                    self.renderer.toggle_vsync();
                    self.ui.dirty = true;
                }
            }
            KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4
            | KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8
            | KeyCode::Digit9 => {
                if pressed && in_game {
                    let n = code as u8 - KeyCode::Digit1 as u8;
                    self.player.selected = n as usize;
                    let b = self.player.inv.slots[n as usize].block;
                    self.item_toast = Some((name(b).to_string(), 2.0));
                    self.ui.dirty = true;
                }
            }
            _ => {}
        }
    }

    /// mouse buttons while in-game
    fn game_mouse(&mut self, button: winit::event::MouseButton, pressed: bool) {
        use winit::event::MouseButton;
        match button {
            MouseButton::Left => {
                if pressed {
                    self.unlock_audio();
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let _ = self
                            .window
                            .set_cursor_grab(winit::window::CursorGrabMode::Locked);
                        self.window.set_cursor_visible(false);
                    }
                    // Phase 2: a mob under the crosshair takes swing
                    // priority over block breaking (vanilla ordering)
                    if self.try_attack_mob() {
                        return;
                    }
                }
                self.input.break_hold = pressed;
            }
            MouseButton::Right => self.input.place_hold = pressed,
            MouseButton::Middle => {
                if pressed {
                    if let Some((_, b, _)) = self.target {
                        if let Some(slot) = self.player.inv.slots.iter().position(|h| h.block == b && h.count > 0) {
                            self.player.selected = slot.min(8);
                        } else {
                            self.player.inv.slots[self.player.selected] =
                                vc_inventory::inventory::ItemStack::new(b, 64);
                        }
                        self.item_toast = Some((name(b).to_string(), 2.0));
                        self.ui.dirty = true;
                    }
                }
            }
            _ => {}
        }
    }

    // ------------------------------------------------------ block picker --

    fn open_picker(&mut self) {
        self.picker_open = true;
        self.input = Input::default();
        // release the pointer so the cursor can select blocks; tell the JS
        // shim we're in a "picker" state so canvas clicks are forwarded as
        // button events instead of lock requests
        #[cfg(target_arch = "wasm32")]
        {
            crate::web_input::release_pointer_lock();
            crate::web_input::set_screen("picker");
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::None);
            self.window.set_cursor_visible(true);
        }
        self.ui.dirty = true;
    }

    fn close_picker(&mut self) {
        self.picker_open = false;
        self.picker_geom = None;
        // back to the plain game state in the shim
        #[cfg(target_arch = "wasm32")]
        crate::web_input::set_screen("game");
        // re-capture the mouse (the E keypress counts as user activation)
        #[cfg(target_arch = "wasm32")]
        crate::web_input::request_pointer_lock();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked);
            self.window.set_cursor_visible(false);
        }
        self.ui.dirty = true;
    }

    /// click inside the picker grid → assign that block to the selected slot
    fn picker_click(&mut self, ux: i32, uy: i32) {
        self.unlock_audio();
        let Some(g) = &self.picker_geom else { return };
        if let Some(idx) = g.slot_at(ux, uy) {
            let b = PICKER_BLOCKS[idx];
            self.player.inv.slots[self.player.selected] = vc_inventory::inventory::ItemStack::new(b, 64);
            self.item_toast = Some((name(b).to_string(), 2.0));
            self.ui.dirty = true;
        }
    }

    /// mouse buttons while in a menu (buttons + sliders + text fields)
    fn menu_mouse(&mut self, _button: winit::event::MouseButton, pressed: bool, x: i32, y: i32) {
        if pressed {
            self.unlock_audio();
            if let Some(w) = self.widgets.iter().find(|w| w.hit(x, y)) {
                match &w.kind {
                    WidgetKind::Slider { .. } => {
                        self.dragging = Some(w.id);
                        let t = w.slider_value_at(x);
                        self.apply_slider(w.id, t);
                        self.click_sound();
                    }
                    WidgetKind::Button { enabled, .. } => {
                        if *enabled {
                            self.activate(w.id);
                            self.click_sound();
                        }
                    }
                    WidgetKind::TextField { .. } => {
                        // Phase 1: clicking a field focuses it (exclusively)
                        self.focus_field(w.id);
                        self.click_sound();
                    }
                }
            } else {
                // click outside any widget: drop text-field focus
                if self.text_field_focused().is_some() {
                    self.focus_field(0);
                }
            }
        } else if self.dragging.is_some() {
            self.dragging = None;
        }
    }

    fn wheel(&mut self, d: f32) {
        if self.screen != Screen::Game || d.abs() <= 0.01 {
            return;
        }
        let n = vc_inventory::inventory::INV_SLOTS.min(9) as i32;
        let cur = self.player.selected as i32;
        let next = ((cur - d.signum() as i32).rem_euclid(n)) as usize;
        self.player.selected = next;
        let b = self.player.inv.slots[next].block;
        self.item_toast = Some((name(b).to_string(), 2.0));
        self.ui.dirty = true;
    }

    fn update_hover(&mut self) {
        let (x, y) = (self.cursor.0 as i32, self.cursor.1 as i32);
        let h = self
            .widgets
            .iter()
            .find(|w| w.hit(x, y))
            .map(|w| w.id);
        if h != self.hover {
            self.hover = h;
            self.ui.dirty = true;
        }
    }

    fn drag_slider(&mut self, ux: f32) {
        let Some(id) = self.dragging else { return };
        if let Some(w) = self.widgets.iter().find(|w| w.id == id) {
            let t = w.slider_value_at(ux as i32);
            self.apply_slider(id, t);
        }
    }

    fn click_sound(&mut self) {
        self.play_event("ui.click", None, 1.0);
    }

    /// §21: play a sound EVENT through the data-driven registry —
    /// weighted variant pick, pitch range roll, category gain (master ×
    /// music), distance attenuation and stereo pan relative to the player.
    /// `pos` = world position (None = non-positional: UI, music, ambient).
    fn play_event(&mut self, event: &str, pos: Option<[f32; 3]>, volume_scale: f32) {
        let listener = self.player.eye().to_array();
        let yaw = self.player.yaw;
        let master = self.settings.volume;
        let music = self.settings.music_volume;
        let Some(r) = self.sounds.pick(event, &mut self.audio_rng, &self.bank) else {
            return;
        };
        // category gain: music rides its own slider; the other seven
        // categories default to full (their content volumes already encode
        // the mix); everything is scaled by the master volume
        let cat_gain = match r.category {
            vc_audio::sounds::SoundCategory::Music => music,
            _ => 1.0,
        };
        let (att, pan) = vc_audio::sounds::spatialize(pos, listener, yaw, r.attenuation);
        let vol = r.volume * volume_scale * att * cat_gain * master;
        if vol > 0.004 {
            self.sounds_played += 1;
            self.audio.play(&self.bank, r.recipe, vol, r.pitch, pan);
        }
    }

    fn unlock_audio(&mut self) {
        if !self.audio_unlocked {
            self.audio.unlock(&self.bank);
            self.audio_unlocked = true;
        }
    }

    // ------------------------------------------------------ screen flow --

    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        #[cfg(target_arch = "wasm32")]
        crate::web_input::set_screen(screen.name());
        // native cursor modes
        #[cfg(not(target_arch = "wasm32"))]
        {
            let game = matches!(screen, Screen::Game);
            let _ = self.window.set_cursor_grab(if game {
                winit::window::CursorGrabMode::Locked
            } else {
                winit::window::CursorGrabMode::None
            });
            self.window.set_cursor_visible(!game);
        }
        #[cfg(target_arch = "wasm32")]
        {
            if screen != Screen::Game {
                // release pointer lock when leaving gameplay
                crate::web_input::release_pointer_lock();
            }
        }
        self.refresh_widgets();
        self.ui.dirty = true;
    }

    pub fn enter_pause(&mut self) {
        if self.screen != Screen::Game {
            return;
        }
        self.set_screen(Screen::Pause);
    }

    pub fn resume_game(&mut self) {
        if self.screen != Screen::Pause {
            return;
        }
        self.set_screen(Screen::Game);
        self.input = Input::default();
        #[cfg(target_arch = "wasm32")]
        {
            if self.container.is_some() {
                // a container was open when we paused: keep the shim in the
                // click-forwarding state and leave the pointer free
                crate::web_input::set_screen("picker");
            } else if !self.drag_look {
                // works when called from a click (transient activation)
                crate::web_input::request_pointer_lock();
            }
        }
    }

    fn open_options(&mut self, from: Screen) {
        self.options_from = from;
        self.options_page = 0; // always land on the general page
        self.set_screen(Screen::Options);
    }

    fn close_options(&mut self) {
        let back = if self.options_from == Screen::Pause { Screen::Pause } else { Screen::Title };
        self.set_screen(back);
    }

    fn start_game(&mut self) {
        // Face the most interesting direction on first entry: sample terrain
        // height around the spawn and aim the camera at LAND, not the ocean
        // (spawning while staring at open water reads as a blank/void world).
        if !self.faced_land {
            self.faced_land = true;
            let eye = self.player.pos;
            let mut best_yaw = 0.0f32;
            let mut best_score = f32::MIN;
            for k in 0..8 {
                let yaw = k as f32 * std::f32::consts::TAU / 8.0;
                let dx = yaw.sin();
                let dz = -yaw.cos();
                let mut score = 0.0f32;
                for d in [20.0f32, 40.0, 64.0] {
                    let bx = (eye.x + dx * d) as i32;
                    let bz = (eye.z + dz * d) as i32;
                    let mut top = -1i32;
                    for y in (0..140).rev() {
                        let b = self.world.get_block(bx, y, bz);
                        if b != AIR && b != WATER && !is_cross(b) {
                            top = y;
                            break;
                        }
                    }
                    score += if top < 0 { -3.0 } else { (top - crate::SEA_LEVEL) as f32 };
                }
                if score > best_score {
                    best_score = score;
                    best_yaw = yaw;
                }
            }
            self.player.yaw = best_yaw;
        }
        self.set_screen(Screen::Game);
        self.input = Input::default();
        #[cfg(target_arch = "wasm32")]
        {
            if !self.drag_look {
                // called from the SINGLEPLAYER click → user activation is live
                crate::web_input::request_pointer_lock();
            }
        }
    }

    /// Enter deterministic benchmark mode (§37/§48 Phase 0): rebuilds the
    /// world with the fixed bench seed, arms the scripted camera, skips the
    /// title flow. Loading still runs normally (streaming + first meshes are
    /// part of what the warmup window absorbs).
    pub fn start_bench(&mut self, bench: crate::bench::BenchState) {
        let seed = bench.seed;
        self.bench = Some(bench);
        // fixed-seed world replaces the random one from GameApp::new
        self.world = World::new(seed);
        let spawn = self.world.find_spawn();
        self.bench_spawn = spawn.into();
        self.player.pos = Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2);
        self.player.vel = Vec3::ZERO;
        // uncapped frames for the measurement window
        self.settings.maxfps = 0;
        // start immediately: camera is scripted, not user-driven
        self.faced_land = true;
        self.set_screen(Screen::Game);
        self.input = Input::default();
        vc_render::render::report_boot_log(&format!(
            "benchmark armed: seed={seed}, orbit camera, fixed timestep"
        ));
    }

    fn quit_to_title(&mut self) {
        // leaving the world → flush unsaved chunks + level.dat (native, §28)
        #[cfg(not(target_arch = "wasm32"))]
        if self.bench.is_none() {
            self.save_world();
        }
        self.set_screen(Screen::Title);
        self.input = Input::default();
        self.target = None;
    }

    // --------------------------------------------- Phase 1: world flow --

    /// Open the saved-world list (native). Rescans `saves/` every time —
    /// the world set may have changed since boot.
    #[cfg(not(target_arch = "wasm32"))]
    fn open_world_select(&mut self) {
        self.worlds = vc_anvil::save::list_worlds();
        // preselect the most recent playable world (vanilla behavior)
        self.ws_selected = self.worlds.iter().position(|w| !w.meta.hardcore_dead);
        self.set_screen(Screen::WorldSelect);
    }

    /// Open the create-world screen (shared native/web). Fresh random seed
    /// preview each time; buffers reset to defaults.
    fn open_world_create(&mut self) {
        self.wc_name = String::from("New World");
        self.wc_seed = String::new();
        self.wc_mode = vc_gameplay::modes::GameMode::Survival;
        self.wc_seed_preview = vc_world::world::World::random_seed();
        self.set_screen(Screen::WorldCreate);
    }

    /// Cancel create: native goes back to the list, web straight to title.
    fn cancel_world_create(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.set_screen(Screen::WorldSelect);
        #[cfg(target_arch = "wasm32")]
        self.set_screen(Screen::Title);
    }

    /// CREATE WORLD: seed parse (vanilla: number = itself, text = Java
    /// hash, blank = random), fresh world + spawn, then the Loading →
    /// spawn-snap → game pipeline (`pending_play`).
    fn create_world(&mut self) {
        let name = {
            let n = self.wc_name.trim();
            if n.is_empty() { "New World" } else { n }
        }
        .to_string();
        let seed = vc_gameplay::modes::parse_seed(&self.wc_seed)
            .unwrap_or_else(|| vc_world::world::World::random_seed());
        let mode = self.wc_mode;
        // native: a fresh directory per world (unique-ified on collision)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dir = vc_anvil::save::unique_world_dir(&name);
            self.save_root = dir;
            self.world_dir = vc_anvil::save::dimension_dir(
                &self.save_root,
                vc_world::world::Dimension::Overworld,
            );
        }
        self.reset_world(seed, mode, name, None);
        self.load_datapacks();
        vc_render::render::report_boot_log(&format!(
            "world created: \"{}\" seed={} mode={}",
            self.world_name, self.world.seed, self.mode.label()
        ));
    }

    /// Phase 9: craft-grid matching with data packs — datapack recipes
    /// first (mirroring vanilla's "later pack overrides" semantics: a
    /// pack recipe can shadow a builtin shape), then the builtin static
    /// registry. Zero packs loaded → behavior identical to before.
    fn craft_result(
        &self,
        grid: &[vc_inventory::inventory::ItemStack],
        size: usize,
    ) -> Option<vc_inventory::inventory::ItemStack> {
        // adapt ItemStacks to name-based GridItems (the datapack matcher
        // is name/tag-driven; the builtin matcher is id-driven)
        let items: Vec<vc_pack::datapack::GridItem> = grid
            .iter()
            .map(|s| {
                if s.is_empty() {
                    vc_pack::datapack::GridItem::empty()
                } else {
                    match vc_pack::datapack::item_name_by_id(s.block) {
                        Some(n) => vc_pack::datapack::GridItem::item(n, s.count),
                        // palette-absent items match no datapack recipe
                        // (recipes reference only bridge-known names)
                        None => vc_pack::datapack::GridItem::item("", s.count),
                    }
                }
            })
            .collect();
        if let Some((block, count)) = self.data.match_grid(&items, size) {
            return Some(vc_inventory::inventory::ItemStack::new(block, count));
        }
        vc_gameplay::craft::match_grid(grid, size)
    }

    /// Play the `idx`-th world from the cached select list (native).
    #[cfg(not(target_arch = "wasm32"))]
    fn play_world(&mut self, idx: usize) {
        let Some(entry) = self.worlds.get(idx).cloned() else { return };
        // a dead hardcore world is unplayable — the list disables its
        // button, but keep the guard here too (defense in depth)
        if entry.meta.hardcore_dead {
            return;
        }
        let seed = entry.meta.seed;
        let mode = vc_gameplay::modes::GameMode::from_save(
            entry.meta.game_type,
            entry.meta.hardcore,
        );
        let name = entry.meta.name.clone();
        let player = entry.meta.player.clone().map(|p| {
            (p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32, p.yaw, p.pitch)
        });
        self.save_root = entry.dir;
        self.world_dir = vc_anvil::save::dimension_dir(
            &self.save_root,
            vc_world::world::Dimension::Overworld,
        );
        self.reset_world(seed, mode, name, player);
        self.load_datapacks();
        vc_render::render::report_boot_log(&format!(
            "world loaded: \"{}\" seed={} mode={}",
            self.world_name, self.world.seed, self.mode.label()
        ));
    }

    /// Phase 9: (re)scan the active world's `datapacks/` directory —
    /// called on world create AND on world load, after `save_root` is
    /// set and before generation fills dungeon chests. Native only;
    /// the wasm build has no filesystem (the E2E `dpdemo` command runs
    /// the embedded demo pack through the same code path instead).
    #[cfg(not(target_arch = "wasm32"))]
    fn load_datapacks(&mut self) {
        let loaded =
            vc_pack::datapack::scan_datapacks(&self.save_root.join("datapacks"));
        report_datapacks(&loaded);
        self.data = loaded;
    }
    #[cfg(target_arch = "wasm32")]
    fn load_datapacks(&mut self) {
        self.data = vc_pack::datapack::LoadedData::default();
    }

    /// DELETE SELECTED on the world-select screen (native). No confirm
    /// dialog yet — vanilla has one; noted as a follow-up.
    #[cfg(not(target_arch = "wasm32"))]
    fn delete_selected_world(&mut self) {
        if let Some(idx) = self.ws_selected {
            if let Some(entry) = self.worlds.get(idx).cloned() {
                if vc_anvil::save::delete_world(&entry.dir) {
                    self.worlds.remove(idx);
                    self.ws_selected = None;
                    vc_render::render::report_boot_log(&format!(
                        "world deleted: {}",
                        entry.dir.display()
                    ));
                }
            }
        }
        self.refresh_widgets();
        self.ui.dirty = true;
    }

    /// Swap the entire engine into a different world: fresh terrain,
    /// fresh sim/particles, fresh player (or restored from `level.dat`).
    /// Mirrors `travel_to_dimension`'s reset list — every world-local
    /// system restarts. The inventory reset is a documented deviation:
    /// vanilla starts Survival empty, we keep the starter palette so the
    /// sandbox stays playable before mobs/food exist (Phase 2).
    ///
    /// `restore` = (x, y, z, yaw, pitch) — a plain tuple so the signature
    /// is identical on wasm (PlayerMeta lives in the native save module).
    #[allow(clippy::too_many_arguments)]
    fn reset_world(
        &mut self,
        seed: u64,
        mode: vc_gameplay::modes::GameMode,
        name: String,
        restore: Option<(f32, f32, f32, f32, f32)>,
    ) {
        // flush the outgoing world first (native, §28)
        #[cfg(not(target_arch = "wasm32"))]
        if self.bench.is_none() && self.screen != Screen::Loading {
            self.save_world();
        }
        self.world = World::new(seed);
        let spawn = self.world.find_spawn();
        // Phase 1: mode + identity take effect BEFORE the player exists
        self.mode = mode;
        self.world_name = name;
        self.hardcore_dead = false;
        // overworld respawn point (the world's own spawn)
        self.respawn_pos = Vec3::new(spawn.0, spawn.1 + 1.0, spawn.2);
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.level_spawn = (spawn.0 as i32, spawn.1 as i32, spawn.2 as i32);
        }
        self.player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));
        if let Some((x, y, z, yaw, pitch)) = restore {
            self.player.pos = Vec3::new(x, y, z);
            self.player.yaw = yaw;
            self.player.pitch = pitch;
        }
        // creative starts hovering; survival rides the Loading snap
        self.player.flying = mode.allows_flight();
        self.player.vel = Vec3::ZERO;
        self.player.reset_fall();

        // world-local system reset (same list as dimension travel)
        self.renderer.clear_meshes();
        self.section_meshes.clear();
        self.mesh_inflight.clear();
        self.gen_inflight.clear();
        self.light = vc_world::light::LightEngine::new();
        self.sim = vc_sim::sim::Sim::new(seed);
        self.particles = vc_particles::particles::ParticleSystem::new(seed ^ 0x7EED);
        self.particle_verts.clear();
        self.container = None;
        self.container_geom = None;
        self.cursor_stack = vc_inventory::inventory::ItemStack::EMPTY;
        self.craft_grid = [vc_inventory::inventory::ItemStack::EMPTY; 9];
        self.target = None;
        self.break_timer = 0.0;
        self.place_timer = 0.0;
        self.day_time = 0.30;
        self.edits = 0;

        // the Loading pipeline snaps the player to the surface, then
        // pending_play routes straight into the game
        self.pending_play = true;
        self.traveling = false;
        self.spawn_snapped = false;
        self.faced_land = false;
        self.load_start = self.time;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.autosave_in = 20.0;
        }
        self.set_screen(Screen::Loading);
    }

    // ---------------------------------------------- Phase 1: text fields --

    /// Which text field (widget id) currently holds focus, if any.
    fn text_field_focused(&self) -> Option<u16> {
        self.widgets.iter().find_map(|w| {
            if let WidgetKind::TextField { focused: true, .. } = &w.kind {
                Some(w.id)
            } else {
                None
            }
        })
    }

    /// Focus exactly one text field (`id` 0 = none). Rebuilds widgets so
    /// the focus frame + caret appear immediately.
    fn focus_field(&mut self, id: u16) {
        for w in self.widgets.iter_mut() {
            if let WidgetKind::TextField { focused, .. } = &mut w.kind {
                *focused = w.id == id;
            }
        }
        self.ui.dirty = true;
    }

    /// Type one character into the focused field (ASCII 32..=126 only —
    /// that is the entire range the 5x7 font renders). Returns true when
    /// the character was consumed.
    fn type_char(&mut self, ch: char) -> bool {
        if !(32..=126).contains(&(ch as u32)) {
            return false;
        }
        let Some(id) = self.text_field_focused() else { return false };
        let (buf, max) = match id {
            ui::ID_WC_NAME => (&mut self.wc_name, 32),
            ui::ID_WC_SEED => (&mut self.wc_seed, 24),
            _ => return false,
        };
        if buf.chars().count() >= max {
            return true; // full, but consumed
        }
        buf.push(ch);
        self.sync_field_widgets();
        true
    }

    /// Backspace on the focused field.
    fn backspace_field(&mut self) {
        let Some(id) = self.text_field_focused() else { return };
        let buf = match id {
            ui::ID_WC_NAME => &mut self.wc_name,
            ui::ID_WC_SEED => &mut self.wc_seed,
            _ => return,
        };
        buf.pop();
        self.sync_field_widgets();
    }

    /// Push the live buffers into the widget copies (widgets own their
    /// render state; game.rs owns the truth).
    fn sync_field_widgets(&mut self) {
        let (name, seed) = (self.wc_name.clone(), self.wc_seed.clone());
        for w in self.widgets.iter_mut() {
            match w.id {
                ui::ID_WC_NAME => ui::set_text(w, &name),
                ui::ID_WC_SEED => ui::set_text(w, &seed),
                _ => {}
            }
        }
        self.ui.dirty = true;
    }

    // -------------------------------------------------- Phase 1: death --

    /// Death check + transition. Creative can never die (invulnerable).
    fn check_death(&mut self) {
        if self.mode.invulnerable() || self.screen != Screen::Game {
            return;
        }
        if self.player.health > 0.0 {
            return;
        }
        self.die();
    }

    /// The player died: scatter the inventory (Survival/Hardcore —
    /// vanilla drops it all), zero XP, freeze, show the death screen.
    /// Hardcore additionally latches `hardcore_dead` and flushes the
    /// save IMMEDIATELY so the lock survives a window close.
    fn die(&mut self) {
        let cause = if self.death_cause.is_empty() {
            "YOU DIED".to_string()
        } else {
            std::mem::take(&mut self.death_cause)
        };
        self.player.vel = Vec3::ZERO;
        self.player.flying = false;
        // scatter the inventory as item drops at the death spot
        if self.mode.drops_inventory_on_death() {
            let (bx, by, bz) = (
                self.player.pos.x.floor() as i32,
                self.player.pos.y.floor() as i32,
                self.player.pos.z.floor() as i32,
            );
            let mut dropped = 0usize;
            for slot in self.player.inv.slots.iter_mut() {
                if !slot.is_empty() {
                    for _ in 0..slot.count {
                        self.sim.items.drop_block(bx, by, bz, slot.block, 2, 15, 0);
                    }
                    *slot = vc_inventory::inventory::ItemStack::EMPTY;
                    dropped += 1;
                }
            }
            let _ = dropped;
        }
        // vanilla: XP is lost on death (dropped as orbs — we have none yet,
        // so it just zeroes; documented deviation)
        self.player.xp_points = 0;
        self.player.xp_level = 0;
        self.play_event("entity.player.hurt", None, 1.0);

        if self.mode.permadeath() {
            self.hardcore_dead = true;
            // persist the lock right now — closing the window must not
            // resurrect a dead hardcore world
            #[cfg(not(target_arch = "wasm32"))]
            if self.bench.is_none() {
                self.save_world();
            }
        }
        self.death_cause = cause;
        self.set_screen(Screen::Death);
    }

    /// RESPAWN (Survival only — the button doesn't exist for hardcore).
    /// Full health at the world spawn, empty fall accumulator.
    fn respawn(&mut self) {
        if self.mode.permadeath() || self.hardcore_dead {
            return; // unreachable via UI; guard stays for safety
        }
        self.player.health = 20.0;
        self.player.vel = Vec3::ZERO;
        self.player.pos = self.respawn_pos;
        self.player.reset_fall();
        self.death_cause.clear();
        // re-run the surface snap pipeline from the spawn column
        self.spawn_snapped = false;
        self.pending_play = true;
        self.load_start = self.time;
        self.set_screen(Screen::Loading);
    }

    /// Death-screen exit: TITLE (keeps a locked hardcore world on disk)
    /// or DELETE WORLD (hardcore's clean-slate option).
    fn death_quit_to_title(&mut self, delete: bool) {
        if delete && self.hardcore_dead {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = vc_anvil::save::delete_world(&self.save_root);
            }
        }
        self.death_cause.clear();
        self.quit_to_title();
    }

    // ----------------------------------------------- Phase 2: mob combat --

    /// Swing at the mob under the crosshair. Full vanilla combat math
    /// (verified formulas in vc_gameplay::combat): cooldown scaling
    /// 0.2 + 0.8·p², crits ×1.5 (falling + ≥84.8% + not sprinting),
    /// armor reduction. Returns true when a mob was hit (block breaking
    /// then yields this click).
    fn try_attack_mob(&mut self) -> bool {
        if self.screen != Screen::Game || self.picker_open || self.container.is_some() {
            return false;
        }
        use vc_gameplay::combat;
        let eye = self.player.eye().to_array();
        let dir = self.player.look_dir().to_array();
        let Some(id) = self.sim.mobs.ray_hit(eye, dir, crate::player::REACH) else {
            return false;
        };
        let Some(m) = self.sim.mobs.by_id(id) else { return false };
        let kind = m.kind;
        let armor = vc_gameplay::mobs::def(kind).armor;
        // cooldown recovery fraction p
        let (_, atk_speed) = combat::held_attack(self.player.held().block);
        let period = combat::attack_cooldown_ticks(atk_speed) / 20.0; // seconds
        let p = (self.swing_t / period).min(1.0);
        let falling = !self.player.on_ground && self.player.vel.y < 0.0;
        let sprinting = self.input.sprint
            && (self.input.fwd || self.input.back || self.input.left || self.input.right);
        let outcome = combat::player_melee(
            self.player.held().block,
            p,
            falling,
            sprinting,
            armor,
            0.0,
        );
        let applied = self.sim.mobs.damage(id, outcome.damage);
        if applied > 0.0 {
            self.play_event("entity.generic.death", None, 0.6); // hurt grunt
            vc_render::render::report_boot_log(&format!(
                "e2e: swing p={:.2}{} -> {:.2} dmg to {} (hp now serving)",
                p,
                if outcome.critical { " CRIT" } else { "" },
                outcome.damage,
                kind.name()
            ));
        }
        // every swing resets the recovery clock (weak spam allowed —
        // vanilla's 0.2× floor comes through the same formula)
        self.swing_t = 0.0;
        true
    }

    /// Drain the sim's mob queues after the fixed-step tick: player hits
    /// (difficulty-scaled + knockback), mob deaths (drops + XP), and
    /// creeper explosions (world edits + light + entity damage).
    fn drain_mob_events(&mut self) {
        use vc_gameplay::combat::{difficulty_scale, Difficulty};
        use vc_gameplay::mobs;
        // ---- 1. hits on the player ----
        let hits: Vec<mobs::PlayerHit> = self.sim.mobs.hits.drain(..).collect();
        for h in hits {
            if self.mode.invulnerable() || self.screen != Screen::Game {
                continue; // creative absorbs everything
            }
            let difficulty = if self.mode.permadeath() {
                Difficulty::Hard
            } else {
                Difficulty::Normal
            };
            let dmg = difficulty_scale(h.damage, difficulty);
            let applied = self.player.damage(dmg);
            if applied > 0.0 {
                self.play_event("entity.player.hurt", None, 1.0);
                // knockback: horizontal impulse away from the source +
                // a lift (documented adaptation of vanilla's 0.4 base)
                let k = &h.knockback_dir;
                self.player.vel[0] += k[0] * 6.0;
                self.player.vel[2] += k[1] * 6.0;
                if self.player.on_ground {
                    self.player.vel[1] = 4.2;
                }
                self.death_cause = format!("SLAIN BY A {}", h.source.name());
                self.ui.dirty = true;
            }
        }
        // ---- 2. mob deaths → drops + XP ----
        let deaths: Vec<(mobs::MobKind, [f32; 3])> = self.sim.mobs.deaths.drain(..).collect();
        for (kind, pos) in deaths {
            let d = mobs::def(kind);
            // vanilla-common loot ranges [adaptation: fixed min..max per
            // kind, no weighted loot tables yet — Phase 9 territory]
            let drops: &[(u8, u8)] = match kind {
                mobs::MobKind::Zombie => &[(ROTTEN_FLESH, 2)],
                mobs::MobKind::Skeleton => &[(BONE, 2), (ARROW_ITEM, 2)],
                mobs::MobKind::Creeper => &[(GUNPOWDER, 2)],
                mobs::MobKind::Spider => &[(STRING, 2)],
                mobs::MobKind::Enderman => &[(ENDER_PEARL, 1)],
                mobs::MobKind::Cow => &[(BEEF, 3), (LEATHER, 2)],
                mobs::MobKind::Pig => &[(PORKCHOP, 3)],
                mobs::MobKind::Sheep => &[(MUTTON, 2), (WOOL_WHITE, 1)],
                mobs::MobKind::Chicken => &[(CHICKEN_RAW, 1), (FEATHER, 2)],
            };
            for (block, max_n) in drops {
                let n = 1 + (self.audio_rng.next_f32() * *max_n as f32) as u8;
                for _ in 0..n {
                    self.sim.items.drop_block(
                        pos[0].floor() as i32,
                        pos[1].floor() as i32,
                        pos[2].floor() as i32,
                        *block,
                        2,
                        15,
                        0,
                    );
                }
            }
            // Phase 4 §26: spiders additionally have a 1/3 chance to drop
            // one spider eye (VERIFIED, 1.16.5-era Spider page — only when
            // killed by a player, which drain_mob_events is)
            if kind == mobs::MobKind::Spider && self.audio_rng.next_f32() < 1.0 / 3.0 {
                self.sim.items.drop_block(
                    pos[0].floor() as i32,
                    pos[1].floor() as i32,
                    pos[2].floor() as i32,
                    SPIDER_EYE,
                    2,
                    15,
                    0,
                );
            }
            // XP through the real curve
            if d.xp > 0 {
                let gained = self.player.add_xp(d.xp);
                if gained > 0 {
                    self.play_event("entity.player.levelup", None, 1.0);
                }
            }
            self.play_event(
                "entity.generic.death",
                Some([pos[0], pos[1] + 0.5, pos[2]]),
                1.0,
            );
            self.edits += 1;
        }
        // ---- 3. creeper explosions ----
        let booms = mobs::take_explosions(&mut self.sim.mobs);
        for (center, power) in booms {
            self.explode(center, power);
        }
    }

    /// One explosion: probabilistic sphere of block destruction (bedrock
    /// and obsidian resist), light updates per edit, particles + sound,
    /// distance-scaled damage to the player and every mob.
    /// [placeholder: damage = 24·(1 − dist/(power·2)) capped — vanilla's
    /// exact exposure-based formula was not verified this pass]
    fn explode(&mut self, center: [f32; 3], power: f32) {
        let r = power as i32;
        let (cx, cy, cz) = (
            center[0].floor() as i32,
            center[1].floor() as i32,
            center[2].floor() as i32,
        );
        let mut destroyed = 0u32;
        for dy in -r..=r {
            for dz in -r..=r {
                for dx in -r..=r {
                    let dist = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                    if dist > power as f32 {
                        continue;
                    }
                    let (x, y, z) = (cx + dx, cy + dy, cz + dz);
                    let b = self.world.get_block(x, y, z);
                    if b == AIR || b == BEDROCK || b == OBSIDIAN || b == WATER {
                        continue; // resistant / already gone
                    }
                    // vanilla-ish ragged edge: 70% + 30%·random survival
                    let edge = 0.7 + self.audio_rng.next_f32() * 0.3;
                    if dist / power as f32 > edge {
                        continue;
                    }
                    if let Some((old, new)) = self.world.set_block(x, y, z, AIR) {
                        self.light.on_block_changed(&self.world, x, y, z, old, new);
                    }
                    destroyed += 1;
                }
            }
        }
        // entity damage: distance-scaled (see placeholder note)
        let blast_damage = |ex: f32, ey: f32, ez: f32| -> f32 {
            let d = ((ex - center[0]).powi(2) + (ey - center[1]).powi(2) + (ez - center[2]).powi(2)).sqrt();
            (24.0 * (1.0 - d / (power * 2.0)).max(0.0)).max(0.0)
        };
        // player (mode-gated, knockback away from the blast)
        if !self.mode.invulnerable() && self.screen == Screen::Game {
            let dmg = blast_damage(
                self.player.pos.x,
                self.player.pos.y + 0.9,
                self.player.pos.z,
            );
            if dmg > 0.0 {
                let applied = self.player.damage(dmg);
                if applied > 0.0 {
                    let dir = glam::Vec3::new(
                        self.player.pos.x - center[0],
                        0.0,
                        self.player.pos.z - center[2],
                    )
                    .normalize_or_zero();
                    self.player.vel[0] += dir.x * 10.0;
                    self.player.vel[2] += dir.z * 10.0;
                    self.player.vel[1] += 6.0;
                    self.death_cause = "BLOWN UP BY A CREEPER".into();
                    self.play_event("entity.player.hurt", None, 1.0);
                    self.ui.dirty = true;
                }
            }
        }
        // every other mob in range takes the same blast (armor applies)
        use vc_gameplay::combat::armor_reduce;
        let mob_ids: Vec<(u32, f32)> = self
            .sim
            .mobs
            .list
            .iter()
            .map(|m| {
                (
                    m.id,
                    blast_damage(m.pos[0], m.pos[1] + 0.9, m.pos[2]),
                )
            })
            .collect();
        for (id, dmg) in mob_ids {
            if dmg > 0.0 {
                let armor = self
                    .sim
                    .mobs
                    .by_id(id)
                    .map(|m| vc_gameplay::mobs::def(m.kind).armor)
                    .unwrap_or(0.0);
                let through = armor_reduce(dmg, armor, 0.0);
                self.sim.mobs.damage(id, through);
            }
        }
        // explosion visual + sound
        for _ in 0..24 {
            self.particles.spawn_block_break(
                cx,
                cy,
                cz,
                COBBLE,
                2,
                14,
                10,
            );
        }
        self.play_event("entity.generic.explode", Some(center), 1.0);
        let _ = destroyed;
    }

    // ------------------------------------------------------ menu actions --

    fn activate(&mut self, id: u16) {
        use ui::*;
        match id {
            ID_TITLE_PLAY => {
                // Phase 1: SINGLEPLAYER opens the world flow — native picks
                // from the save list, web (no persistence) creates directly
                #[cfg(not(target_arch = "wasm32"))]
                self.open_world_select();
                #[cfg(target_arch = "wasm32")]
                self.open_world_create();
            }
            ID_TITLE_OPTIONS => self.open_options(Screen::Title),
            ID_TITLE_QUIT => self.quit_requested = true,
            ID_OPT_DONE | ID_OPT_DONE2 => self.close_options(),
            // Phase 6 §26: options page navigation (vanilla-style Video
            // Settings split)
            ID_OPT_NEXT => {
                self.options_page = 1;
                self.refresh_widgets();
                self.ui.dirty = true;
            }
            ID_OPT_PREV => {
                self.options_page = 0;
                self.refresh_widgets();
                self.ui.dirty = true;
            }
            // ---- Phase 6 §26: video-detail buttons ----
            ID_OPT_MIP => {
                // vanilla mipmapLevels cycle: 0 → 4 (VERIFIED range 0–4)
                self.settings.mipmap_levels = (self.settings.mipmap_levels + 1) % 5;
                self.after_settings_change();
            }
            ID_OPT_ANISO => {
                // OptiFine ofAfLevel cycle: 1 → 2 → 4 → 8 → 16
                self.settings.aniso = match self.settings.aniso {
                    1 => 2,
                    2 => 4,
                    4 => 8,
                    8 => 16,
                    _ => 1,
                };
                self.after_settings_change();
            }
            ID_OPT_MSAA => {
                // off → 4x → 8x, device-gated: an unsupported 8x request
                // snaps to the device max (4x on most hardware)
                let wanted = match self.settings.msaa {
                    0 => 4,
                    4 => 8,
                    _ => 0,
                };
                self.settings.msaa = if wanted == 0 {
                    0
                } else {
                    self.renderer.msaa_supported().min(wanted)
                };
                self.after_settings_change();
            }
            ID_OPT_OCCL => {
                self.settings.occlusion = !self.settings.occlusion;
                self.after_settings_change();
            }
            ID_OPT_GMESH => {
                // Phase 7: GPU compute meshing toggle — remeshes everything
                // through the new backend (mirrors the smooth-lighting
                // toggle's remesh_all semantics: cached section meshes
                // were built by the other backend). N/A (no compute
                // adapter): flip the stored preference but skip the
                // remesh — nothing renders differently.
                let avail = self.renderer.gpu_mesh.is_some();
                self.settings.gpu_meshing = !self.settings.gpu_meshing && avail;
                if avail {
                    self.remesh_all();
                }
                self.after_settings_change();
            }
            ID_PAUSE_BACK => self.resume_game(),
            ID_PAUSE_OPTIONS => self.open_options(Screen::Pause),
            ID_PAUSE_QUIT => self.quit_to_title(),
            // ---- Phase 1: world select / create / death screens ----
            ID_WS_CREATE => self.open_world_create(),
            ID_WS_CANCEL => self.set_screen(Screen::Title),
            ID_WS_DELETE => {
                #[cfg(not(target_arch = "wasm32"))]
                self.delete_selected_world();
            }
            ID_WC_MODE => {
                self.wc_mode = self.wc_mode.next();
                self.refresh_widgets();
                self.ui.dirty = true;
            }
            ID_WC_CREATE => self.create_world(),
            ID_WC_CANCEL => self.cancel_world_create(),
            ID_DEATH_RESPAWN => self.respawn(),
            ID_DEATH_TITLE => self.death_quit_to_title(false),
            ID_DEATH_DELETE => self.death_quit_to_title(true),
            _ if (ID_WS_WORLD_BASE..ID_WS_WORLD_BASE + MAX_LISTED_WORLDS as u16)
                .contains(&id) =>
            {
                // clicking a row selects it; a live world also plays
                // (WorldSelect is native-only — unreachable on wasm)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let idx = (id - ID_WS_WORLD_BASE) as usize;
                    self.ws_selected = Some(idx);
                    let dead = self
                        .worlds
                        .get(idx)
                        .map(|w| w.meta.hardcore_dead)
                        .unwrap_or(false);
                    if !dead {
                        self.play_world(idx);
                    } else {
                        self.refresh_widgets();
                    }
                }
            }
            ID_OPT_SHADER => {
                // Phase 11: off → vanilla+ → cinematic → <pack 0> … <pack N>
                let n = 3 + self.shader_packs.len() as u8;
                self.settings.shader = (self.settings.shader + 1) % n.max(3);
                self.after_settings_change();
            }
            ID_OPT_GRAPHICS => {
                self.settings.graphics = (self.settings.graphics + 1) % 3;
                self.after_settings_change();
            }
            ID_OPT_SHADOWS => {
                // §17 quality cycle: OFF → 1024 → 2048 → 4096
                self.settings.shadow_quality = (self.settings.shadow_quality + 1) % 4;
                self.renderer.set_shadow_quality(self.settings.shadow_map_px());
                self.after_settings_change();
            }
            ID_OPT_UPSCALE => {
                self.settings.upscale = (self.settings.upscale + 1) % 3;
                self.renderer.set_upscale(self.settings.upscale_factor());
                self.after_settings_change();
            }
            ID_OPT_MAXFPS => {
                self.settings.maxfps = (self.settings.maxfps + 1) % 4;
                self.after_settings_change();
            }
            ID_OPT_SMOOTH => {
                self.settings.smooth_lighting = !self.settings.smooth_lighting;
                self.remesh_all();
                self.after_settings_change();
            }
            ID_OPT_CLOUDS => {
                self.settings.clouds = !self.settings.clouds;
                self.after_settings_change();
            }
            _ => {}
        }
    }

    fn apply_slider(&mut self, id: u16, t: f32) {
        use ui::*;
        let t = t.clamp(0.0, 1.0);
        match id {
            ID_OPT_FOV => self.settings.fov = 30.0 + t * 80.0,
            ID_OPT_SENS => self.settings.sensitivity = 0.1 + t * 1.9,
            ID_OPT_RD => self.settings.render_distance = 2 + (t * 14.0).round() as i32,
            // Phase 6 §26: VERIFIED range 5–32 (wiki simulationDistance)
            ID_OPT_SIMDIST => self.settings.sim_distance = 5 + (t * 27.0).round() as i32,
            ID_OPT_BRIGHT => self.settings.brightness = t,
            ID_OPT_VOL => self.settings.volume = t,
            ID_OPT_MUSIC => self.settings.music_volume = t,
            _ => {}
        }
        self.after_settings_change();
    }

    /// persist + refresh widget labels + player fov
    fn after_settings_change(&mut self) {
        self.player.fov = self.settings.fov.to_radians();
        // Phase 11 §34: re-apply the shader selection (pack pipeline swap)
        self.apply_shader_selection();
        // Phase 6 §26: texture quality (mipmaps + aniso), MSAA, occlusion
        self.renderer.set_texture_quality(self.settings.mipmap_levels, self.settings.aniso);
        self.renderer.set_msaa(self.settings.msaa);
        self.renderer.set_occlusion(self.settings.occlusion);
        #[cfg(target_arch = "wasm32")]
        crate::web_input::save_settings(&self.settings.serialize());
        self.refresh_widgets();
        self.ui.dirty = true;
    }

    /// Phase 11 §34: settings.shader → display name (engine modes + packs)
    fn shader_mode_name(&self, mode: u8) -> &str {
        match mode {
            0 => "OFF",
            1 => "VANILLA+",
            2 => "CINEMATIC",
            i => self
                .shader_packs
                .get((i - 3) as usize)
                .map(|p| p.name.as_str())
                .unwrap_or("?"),
        }
    }

    /// Phase 11 §34: map settings.shader → renderer pack state. 0..2 are
    /// the engine modes (pack cleared); 3.. = pack index (clamped — a
    /// persisted selection outliving a removed pack falls back cleanly).
    fn apply_shader_selection(&mut self) {
        let idx = shader_mode_pack_index(self.settings.shader, self.shader_packs.len());
        let pack = idx.and_then(|i| self.shader_packs.get(i));
        self.renderer.set_shader_pack(pack);
    }

    /// rebuild widget list from current settings (labels carry values)
    fn refresh_widgets(&mut self) {
        use ui::*;
        let s = self.settings.clone();
        match self.screen {
            Screen::Title => {
                self.widgets = layout_title(cfg!(target_arch = "wasm32"));
            }
            Screen::Pause => {
                self.widgets = layout_pause();
            }
            Screen::Options if self.options_page == 1 => {
                // Phase 6 §26: page 2 — video details
                let mut ws = layout_options2();
                let max_msaa = self.renderer.msaa_supported();
                for w in ws.iter_mut() {
                    match w.id {
                        ID_OPT_SIMDIST => set_slider(
                            w,
                            &format!("SIM DIST: {} CHUNKS", s.sim_distance),
                            (s.sim_distance - 5) as f32 / 27.0,
                        ),
                        ID_OPT_RD => set_slider(w, &format!("RENDER DIST: {} CHUNKS", s.render_distance), (s.render_distance - 2) as f32 / 14.0),
                        ID_OPT_MIP => set_button_value(w, &format!("{}", s.mipmap_levels)),
                        ID_OPT_ANISO => set_button_value(
                            w,
                            &if s.aniso > 1 { format!("{}X", s.aniso) } else { "OFF".into() },
                        ),
                        ID_OPT_MSAA => {
                            let label = if self.renderer.msaa() == 0 {
                                "OFF".to_string()
                            } else {
                                format!("{}X{}", self.renderer.msaa(),
                                    if (self.renderer.msaa() as u8) < max_msaa { " (MAX)" } else { "" })
                            };
                            set_button_value(w, &label);
                        }
                        ID_OPT_OCCL => set_button_value(w, if s.occlusion { "ON" } else { "OFF" }),
                        ID_OPT_GMESH => set_button_value(
                            w,
                            if s.gpu_meshing && self.renderer.gpu_mesh.is_some() {
                                "ON"
                            } else if self.renderer.gpu_mesh.is_some() {
                                "OFF"
                            } else {
                                "N/A"
                            },
                        ),
                        _ => {}
                    }
                }
                self.widgets = ws;
            }
            Screen::Options => {
                let mut ws = layout_options();
                for w in ws.iter_mut() {
                    match w.id {
                        ID_OPT_FOV => set_slider(w, &format!("FOV: {}", s.fov.round() as i32), (s.fov - 30.0) / 80.0),
                        ID_OPT_SENS => set_slider(w, &format!("MOUSE SENS: {}%", (s.sensitivity * 100.0).round() as i32), (s.sensitivity - 0.1) / 1.9),
                        ID_OPT_RD => set_slider(w, &format!("RENDER DIST: {} CHUNKS", s.render_distance), (s.render_distance - 2) as f32 / 14.0),
                        ID_OPT_BRIGHT => {
                            let label = if s.brightness < 0.05 { "MOODY".to_string() } else { format!("{}%", (s.brightness * 100.0).round() as i32) };
                            set_slider(w, &format!("BRIGHTNESS: {}", label), s.brightness)
                        }
                        ID_OPT_VOL => set_slider(w, &format!("VOLUME: {}%", (s.volume * 100.0).round() as i32), s.volume),
                        ID_OPT_MUSIC => {
                            set_slider(w, &format!("MUSIC: {}%", (s.music_volume * 100.0).round() as i32), s.music_volume)
                        }
                        ID_OPT_SHADER => set_button_value(w, &self.shader_mode_name(s.shader)),
                        ID_OPT_GRAPHICS => set_button_value(w, match s.graphics { 0 => "FAST", 2 => "FABULOUS!", _ => "FANCY" }),
                        ID_OPT_SHADOWS => set_button_value(
                            w,
                            match s.shadow_quality {
                                0 => "OFF",
                                1 => "1K",
                                2 => "2K",
                                _ => "4K",
                            },
                        ),
                        ID_OPT_UPSCALE => set_button_value(w, match s.upscale { 1 => "75% FSR", 2 => "50% FSR", _ => "OFF" }),
                        ID_OPT_MAXFPS => set_button_value(w, match s.maxfps { 1 => "30", 2 => "60", 3 => "120", _ => "VSYNC" }),
                        ID_OPT_SMOOTH => set_button_value(w, if s.smooth_lighting { "ON" } else { "OFF" }),
                        ID_OPT_CLOUDS => set_button_value(w, if s.clouds { "ON" } else { "OFF" }),
                        _ => {}
                    }
                }
                self.widgets = ws;
            }
            #[cfg(not(target_arch = "wasm32"))]
            Screen::WorldSelect => {
                let names: Vec<(String, String, bool)> = self
                    .worlds
                    .iter()
                    .take(MAX_LISTED_WORLDS)
                    .map(|w| {
                        let mode = vc_gameplay::modes::GameMode::from_save(
                            w.meta.game_type,
                            w.meta.hardcore,
                        );
                        (w.meta.name.clone(), mode.label().to_string(), w.meta.hardcore_dead)
                    })
                    .collect();
                self.widgets = layout_world_select(&names);
            }
            Screen::WorldCreate => {
                // live buffers → widgets (focus state preserved via the
                // rebuild: the focused id is re-set from the last state)
                let focused = self.text_field_focused().unwrap_or(0);
                let mut ws = layout_world_create(
                    &self.wc_name,
                    &format!("{}", self.wc_seed_preview),
                    self.wc_mode.label(),
                    self.wc_mode.describe(),
                );
                for w in ws.iter_mut() {
                    if let WidgetKind::TextField { focused: f, .. } = &mut w.kind {
                        *f = w.id == focused;
                    }
                }
                self.widgets = ws;
            }
            Screen::Death => {
                self.widgets = layout_death(self.mode.permadeath());
            }
            _ => self.widgets = Vec::new(),
        }
    }

    fn remesh_all(&mut self) {
        let positions: Vec<ChunkPos> = self.renderer.chunks.keys().copied().collect();
        for p in positions {
            self.world.mark_all_dirty(p, vc_world::world::CAUSE_GEOMETRY | vc_world::world::CAUSE_LIGHT);
        }
        // cached section meshes embed the old baking (e.g. smooth-lighting AO)
        self.section_meshes.clear();
        self.renderer.clear_meshes();
    }


    // ------------------------------------------- containers (Phase 7) --

    /// open a container screen (inventory / crafting table / furnace)
    fn open_container(&mut self, c: Container) {
        self.container = Some(c);
        self.container_geom = None;
        self.input = Input::default();
        self.unlock_audio();
        // release the pointer so the cursor can click slots; tell the JS
        // shim we're in a picker-like state (canvas clicks forwarded as
        // button events, not lock requests)
        #[cfg(target_arch = "wasm32")]
        {
            crate::web_input::release_pointer_lock();
            crate::web_input::set_screen("picker");
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::None);
            self.window.set_cursor_visible(true);
        }
        self.ui.dirty = true;
    }

    /// close the container: craft-grid leftovers return to the inventory
    /// (vanilla behavior), the cursor stack drops back in too
    fn close_container(&mut self) {
        if let Some(c) = self.container.take() {
            match c {
                Container::Inventory => {
                    for s in self.craft_grid.iter_mut().take(4) {
                        if !s.is_empty() {
                            let left = self.player.inv.add(s.block, s.count);
                            if left > 0 {
                                // inventory full → drop into the world
                                self.sim.items.drop_block(
                                    self.player.pos.x.floor() as i32,
                                    self.player.pos.y.floor() as i32,
                                    self.player.pos.z.floor() as i32,
                                    s.block, 2, 15, 0,
                                );
                            }
                            *s = vc_inventory::inventory::ItemStack::EMPTY;
                        }
                    }
                }
                Container::Crafting { pos } => {
                    for s in self.craft_grid.iter_mut() {
                        if !s.is_empty() {
                            let left = self.player.inv.add(s.block, s.count);
                            if left > 0 {
                                self.sim.items.drop_block(pos[0], pos[1] + 1, pos[2], s.block, 2, 15, 0);
                            }
                            *s = vc_inventory::inventory::ItemStack::EMPTY;
                        }
                    }
                }
                Container::Chest { .. } => {
                    // chest contents live in the block entity, not the
                    // player — nothing to return (vanilla behavior)
                }
                Container::Furnace { .. } => {}
                Container::Brewing { .. } => {}
                Container::Enchant { pos } => {
                    // vanilla: the table's item + lapis return to the player
                    if let Some(e) = self.sim.enchants.map.get(&pos) {
                        for s in [&e.item, &e.lapis] {
                            if !s.is_empty() {
                                let left = self.player.inv.add(s.block, s.count);
                                if left > 0 {
                                    self.sim.items.drop_block(
                                        pos[0], pos[1] + 1, pos[2], s.block, 2, 15, 0,
                                    );
                                }
                            }
                        }
                    }
                }
                Container::Trade { .. } => {}
            }
        }
        // cursor returns to the inventory
        if !self.cursor_stack.is_empty() {
            let left = self.player.inv.add(self.cursor_stack.block, self.cursor_stack.count);
            if left > 0 {
                let b = self.cursor_stack.block;
                self.sim.items.drop_block(
                    self.player.pos.x.floor() as i32,
                    self.player.pos.y.floor() as i32,
                    self.player.pos.z.floor() as i32,
                    b, 2, 15, 0,
                );
            }
            self.cursor_stack = vc_inventory::inventory::ItemStack::EMPTY;
        }
        self.container_geom = None;
        // re-capture the mouse (the keypress counts as user activation)
        #[cfg(target_arch = "wasm32")]
        {
            crate::web_input::set_screen("game");
            crate::web_input::request_pointer_lock();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked);
            self.window.set_cursor_visible(false);
        }
        self.ui.dirty = true;
    }

    /// LEFT/RIGHT click inside the open container screen: vanilla slot
    /// semantics (LEFT = whole stack / swap / merge, RIGHT = half-take /
    /// place-one), plus the special craft-result and furnace-output rules.
    fn container_click(&mut self, ux: i32, uy: i32, right: bool) {
        self.unlock_audio();
        // resolve the slot first, then mutate (geom borrow must not overlap)
        let slot = match self.container_geom.as_ref().and_then(|g| g.slot_at(ux, uy)) {
            Some(s) => s,
            None => return,
        };
        use vc_inventory::inventory::Inventory;
        use vc_render::ui::SlotRef;
        match slot {
            SlotRef::Inv(i) if i < vc_inventory::inventory::INV_SLOTS => {
                Inventory::slot_click(
                    &mut self.player.inv.slots[i],
                    &mut self.cursor_stack,
                    right,
                );
            }
            SlotRef::Craft(i) => {
                let n_cells = self.craft_grid_cells();
                if i < n_cells {
                    Inventory::slot_click(
                        &mut self.craft_grid[i],
                        &mut self.cursor_stack,
                        right,
                    );
                }
            }
            SlotRef::Chest(i) => {
                // Phase 3: chest slots click like inventory slots
                if let Some(Container::Chest { pos }) = self.container {
                    if let Some(inv) = self.sim.containers.get_mut(&pos) {
                        if i < inv.slots.len() {
                            let inv = &mut inv.slots[i];
                            Inventory::slot_click(inv, &mut self.cursor_stack, right);
                        }
                    }
                }
            }
            SlotRef::CraftOut => {
                // take the crafted result: consume one of every ingredient,
                // land the output in the cursor (merge if it matches)
                let size = self.craft_grid_size();
                let grid: Vec<vc_inventory::inventory::ItemStack> =
                    self.craft_grid.iter().take(size * size).copied().collect();
                if let Some(out) = self.craft_result(&grid, size) {
                    let fits = self.cursor_stack.is_empty()
                        || (self.cursor_stack.block == out.block
                            && self.cursor_stack.count + out.count
                                <= vc_inventory::inventory::STACK_MAX);
                    if fits {
                        if self.cursor_stack.is_empty() {
                            self.cursor_stack = out;
                        } else {
                            self.cursor_stack.count += out.count;
                        }
                        vc_gameplay::craft::consume_grid(
                            &mut self.craft_grid[..size * size],
                        );
                        self.play_event("block.wood.dig", None, 0.8);
                    }
                }
            }
            SlotRef::FurnaceInput | SlotRef::FurnaceFuel => {
                let Some(Container::Furnace { pos }) = self.container else {
                    return;
                };
                let Some(f) = self.sim.furnaces.map.get_mut(&pos) else {
                    return;
                };
                if slot == SlotRef::FurnaceFuel
                    && !self.cursor_stack.is_empty()
                    && vc_gameplay::furnace::fuel_ticks(self.cursor_stack.block) == 0
                {
                    return; // vanilla: only burnable items in the fuel slot
                }
                let target = if slot == SlotRef::FurnaceInput {
                    &mut f.input
                } else {
                    &mut f.fuel
                };
                Inventory::slot_click(target, &mut self.cursor_stack, right);
            }
            SlotRef::FurnaceOutput => {
                let Some(Container::Furnace { pos }) = self.container else {
                    return;
                };
                // §29: collecting smelted output grants the pooled XP
                // (vanilla: xp accrues per smelt, pays on collect) —
                // self.player is a disjoint field so this stays borrow-clean
                let grant = self
                    .sim
                    .furnaces
                    .map
                    .get_mut(&pos)
                    .map(|f| {
                        let g = f.xp_pool.floor() as i32;
                        f.xp_pool -= g as f32;
                        g
                    })
                    .unwrap_or(0);
                let leveled = if grant > 0 {
                    self.player.add_xp(grant)
                } else {
                    0
                };
                let Some(f) = self.sim.furnaces.map.get_mut(&pos) else {
                    return;
                };
                // take-only: whole stack on LEFT, half on RIGHT
                if !f.output.is_empty() {
                    if !right || f.output.count == 1 {
                        if self.cursor_stack.is_empty() {
                            self.cursor_stack = f.output;
                            f.output = vc_inventory::inventory::ItemStack::EMPTY;
                        } else if self.cursor_stack.block == f.output.block {
                            let room =
                                vc_inventory::inventory::STACK_MAX - self.cursor_stack.count;
                            let take = room.min(f.output.count);
                            self.cursor_stack.count += take;
                            f.output.count -= take;
                            if f.output.count == 0 {
                                f.output = vc_inventory::inventory::ItemStack::EMPTY;
                            }
                        }
                    } else {
                        let half = f.output.split();
                        if self.cursor_stack.is_empty() {
                            self.cursor_stack = half;
                        }
                    }
                }
                if leveled > 0 {
                    self.play_event("entity.player.levelup", None, 1.0);
                }
            }
            SlotRef::BrewIngredient => {
                let Some(Container::Brewing { pos }) = self.container else {
                    return;
                };
                let Some(b) = self.sim.brewing.map.get_mut(&pos) else {
                    return;
                };
                Inventory::slot_click(&mut b.ingredient, &mut self.cursor_stack, right);
            }
            SlotRef::BrewFuel => {
                let Some(Container::Brewing { pos }) = self.container else {
                    return;
                };
                // vanilla: only fuel items in the fuel slot
                if !self.cursor_stack.is_empty()
                    && !vc_gameplay::brewing::is_fuel(self.cursor_stack.block)
                {
                    return;
                }
                let Some(b) = self.sim.brewing.map.get_mut(&pos) else {
                    return;
                };
                Inventory::slot_click(&mut b.fuel, &mut self.cursor_stack, right);
            }
            SlotRef::BrewBottle(i) => {
                let Some(Container::Brewing { pos }) = self.container else {
                    return;
                };
                // vanilla: bottle slots accept only bottles/potions
                if !self.cursor_stack.is_empty()
                    && !is_item_block(self.cursor_stack.block)
                {
                    return;
                }
                let Some(b) = self.sim.brewing.map.get_mut(&pos) else {
                    return;
                };
                let mut slot = b.bottles[i];
                Inventory::slot_click(&mut slot, &mut self.cursor_stack, right);
                b.bottles[i] = slot;
            }
            SlotRef::EnchantItem => {
                let Some(Container::Enchant { pos }) = self.container else {
                    return;
                };
                // vanilla: only books go in the item slot
                if !self.cursor_stack.is_empty()
                    && self.cursor_stack.block != ENCHANTED_BOOK
                {
                    return;
                }
                let seed = self.world.seed;
                let Some(e) = self.sim.enchants.map.get_mut(&pos) else {
                    return;
                };
                let changed = e.item != self.cursor_stack;
                let mut slot = e.item;
                Inventory::slot_click(&mut slot, &mut self.cursor_stack, right);
                e.item = slot;
                // vanilla: the offer list re-rolls when the item changes
                if changed {
                    e.reroll(&self.world, pos, seed);
                }
            }
            SlotRef::EnchantLapis => {
                let Some(Container::Enchant { pos }) = self.container else {
                    return;
                };
                // vanilla: only lapis goes in the lapis slot
                if !self.cursor_stack.is_empty()
                    && self.cursor_stack.block != LAPIS_ORE
                {
                    return;
                }
                let Some(e) = self.sim.enchants.map.get_mut(&pos) else {
                    return;
                };
                let mut slot = e.lapis;
                Inventory::slot_click(&mut slot, &mut self.cursor_stack, right);
                e.lapis = slot;
            }
            SlotRef::EnchantOption(row) => {
                // §29: pay levels + lapis, enchant the book, re-roll offers
                let Some(Container::Enchant { pos }) = self.container else {
                    return;
                };
                let player_level = self.player.xp_level;
                let Some(e) = self.sim.enchants.map.get_mut(&pos) else {
                    return;
                };
                if !e.can_apply(row, player_level) {
                    return;
                }
                let before = e.options[row];
                let Some(cost) = e.apply(row) else {
                    return;
                };
                // pay: lapis from the slot, levels from the player
                if e.lapis.count >= cost {
                    e.lapis.count -= cost;
                    if e.lapis.count == 0 {
                        e.lapis = vc_inventory::inventory::ItemStack::EMPTY;
                    }
                }
                self.player.spend_levels(cost as i32);
                self.sim.enchants.total_enchanted += 1;
                let seed = self.world.seed;
                e.reroll(&self.world, pos, seed);
                self.play_event(
                    "block.enchantment_table.use",
                    Some([pos[0] as f32 + 0.5, pos[1] as f32 + 0.5, pos[2] as f32 + 0.5]),
                    1.0,
                );
                let def = vc_gameplay::enchanting::enchant_def(before.ench);
                vc_render::render::report_boot_log(&format!(
                    "e2e: enchanted {} {} (lvl {}) cost {cost} lvl + {cost} lapis → xp lvl {}",
                    def.name,
                    vc_gameplay::enchanting::roman(before.ench_level),
                    before.level,
                    self.player.xp_level
                ));
            }
            SlotRef::TradeRow(i) => {
                // §29 trading (Phase 5 depth): tier gating + stock +
                // villager XP/level-ups live in execute_trade; the item
                // movement happens here through the REAL inventory
                // consume/add path (emerald ore = our emerald)
                let Some(Container::Trade { villager }) = self.container else {
                    return;
                };
                // read-only preflight: the row must be a visible offer
                // (tier ≤ level) with stock left, and affordable
                let vpos = self.sim.villagers.by_id(villager).map(|v| v.pos);
                let row = self.sim.villagers.by_id(villager).and_then(|v| {
                    let t = *vc_gameplay::villagers::trades(v.profession).get(i)?;
                    let tier_ok = t.tier <= v.level();
                    let stock = v.stock_left(i).unwrap_or(0);
                    (tier_ok && stock > 0).then_some((t, v.level()))
                });
                let (Some(vpos), Some((tr, level))) = (vpos, row) else {
                    return; // locked tier / out of stock — no sound, no trade
                };
                let (give, give_n) = tr.give;
                let (get, get_n) = tr.get;
                if (self.player.inv.count_of(give) as u8) < give_n {
                    self.click_sound();
                    return; // cannot afford
                }
                // the authoritative consume: stock--, villager XP++, maybe level-up
                let Some((tr, leveled)) = self.sim.villagers.execute_trade(villager, i) else {
                    return;
                };
                let (give, give_n) = tr.give;
                let (get, get_n) = tr.get;
                if self.player.inv.consume(give, give_n) {
                    let left = self.player.inv.add(get, get_n);
                    if left > 0 {
                        self.sim.items.drop_block(
                            self.player.pos.x.floor() as i32,
                            self.player.pos.y.floor() as i32,
                            self.player.pos.z.floor() as i32,
                            get, 2, 15, 0,
                        );
                    }
                    if leveled {
                        // villager career level-up: the pleased grunt +
                        // the log line the E2E harness greps for
                        self.play_event(
                            "entity.villager.trade",
                            Some([vpos[0], vpos[1] + 0.9, vpos[2]]),
                            1.15,
                        );
                        let v = self.sim.villagers.by_id(villager).unwrap();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: villager leveled up -> {} (xp {})",
                            vc_gameplay::villagers::level_name(v.level()),
                            v.xp
                        ));
                    }
                    self.play_event(
                        "entity.villager.trade",
                        Some([vpos[0], vpos[1] + 0.9, vpos[2]]),
                        1.0,
                    );
                    let stock = self
                        .sim
                        .villagers
                        .by_id(villager)
                        .and_then(|v| v.stock_left(i))
                        .unwrap_or(0);
                    vc_render::render::report_boot_log(&format!(
                        "e2e: traded {}x {} for {}x {} (lvl {}, stock left {}/{}, total {})",
                        give_n,
                        name(give),
                        get_n,
                        name(get),
                        level,
                        stock,
                        tr.max_uses,
                        self.sim.villagers.trades_done
                    ));
                } else {
                    self.click_sound();
                }
            }
            SlotRef::Inv(_) => {}
        }
        self.click_sound();
        self.ui.dirty = true;
    }

    /// craft grid width per open container: 2 (inventory) or 3 (table)
    fn craft_grid_size(&self) -> usize {
        match self.container {
            Some(Container::Crafting { .. }) => 3,
            _ => 2,
        }
    }

    /// craft grid cell count (4 for 2×2, 9 for 3×3)
    fn craft_grid_cells(&self) -> usize {
        let s = self.craft_grid_size();
        s * s
    }

    /// owned snapshot of everything the container screen renders (§27) —
    /// pure data, built fresh every UI rebuild
    fn container_view(&self) -> vc_render::ui::ContainerView {
        use vc_render::ui::{ContainerKind, ContainerView};
        let (kind, furnace, brewing, enchant, trade) = match self.container {
            Some(Container::Inventory) => (ContainerKind::Inventory, None, None, None, None),
            Some(Container::Crafting { .. }) => (ContainerKind::Crafting, None, None, None, None),
            Some(Container::Chest { pos }) => (ContainerKind::Chest, None, None, None, None),
            Some(Container::Furnace { pos }) => {
                // live slots + progress fractions for the flame/arrow
                let f = self
                    .sim
                    .furnaces
                    .map
                    .get(&pos)
                    .cloned()
                    .unwrap_or_default();
                let burn = if f.burn_max > 0 {
                    f.burn_left as f32 / f.burn_max as f32
                } else {
                    0.0
                };
                let cook = f.cook_left as f32 / vc_gameplay::furnace::COOK_TICKS as f32;
                (
                    ContainerKind::Furnace,
                    Some((f.input, f.fuel, f.output, burn, cook)),
                    None,
                    None,
                    None,
                )
            }
            Some(Container::Brewing { pos }) => {
                // live slots + progress fractions for the bubbles/charge bar
                let b = self
                    .sim
                    .brewing
                    .map
                    .get(&pos)
                    .cloned()
                    .unwrap_or_default();
                let fuel_frac = b.fuel_charges as f32
                    / vc_gameplay::brewing::FUEL_OPERATIONS as f32;
                let brew_frac = b.progress();
                (
                    ContainerKind::Brewing,
                    None,
                    Some((b.ingredient, b.fuel, b.bottles, fuel_frac, brew_frac)),
                    None,
                    None,
                )
            }
            Some(Container::Enchant { pos }) => {
                let e = self
                    .sim
                    .enchants
                    .map
                    .get(&pos)
                    .cloned()
                    .unwrap_or_default();
                (
                    ContainerKind::Enchant,
                    None,
                    None,
                    Some((e.item, e.lapis, e.options, self.player.xp_level, e.power)),
                    None,
                )
            }
            Some(Container::Trade { villager }) => {
                // Phase 5 trade view: all table rows (table-order indices
                // match SlotRef::TradeRow(i) → execute_trade(i)), the
                // career level + XP header, per-row stock + lock state
                let tv = self
                    .sim
                    .villagers
                    .by_id(villager)
                    .map(|v| {
                        let prof = vc_gameplay::villagers::PROFESSIONS
                            [(v.profession as usize).min(vc_gameplay::villagers::PROFESSIONS.len() - 1)];
                        let level = v.level();
                        let rows: Vec<vc_render::ui::TradeRowView> =
                            vc_gameplay::villagers::trades(v.profession)
                                .iter()
                                .enumerate()
                                .map(|(i, t)| {
                                    let give = vc_inventory::inventory::ItemStack::new(t.give.0, t.give.1);
                                    let get = vc_inventory::inventory::ItemStack::new(t.get.0, t.get.1);
                                    let afford =
                                        self.player.inv.count_of(t.give.0) >= t.give.1 as u32;
                                    let stock = v.stock_left(i).unwrap_or(0);
                                    vc_render::ui::TradeRowView {
                                        give,
                                        get,
                                        afford,
                                        stock,
                                        max_uses: t.max_uses,
                                        tier: t.tier,
                                        locked: t.tier > level,
                                    }
                                })
                                .collect();
                        vc_render::ui::TradeView {
                            profession: prof.to_string(),
                            level_name: vc_gameplay::villagers::level_name(level).to_string(),
                            level,
                            xp: v.xp,
                            xp_next: vc_gameplay::villagers::LEVEL_XP
                                .get(level as usize)
                                .copied()
                                .filter(|_| level < 5),
                            rows,
                        }
                    });
                (
                    ContainerKind::Trade,
                    None,
                    None,
                    None,
                    tv,
                )
            }
            None => (ContainerKind::Inventory, None, None, None, None),
        };
        // Phase 3: live chest slots (the container entity is created on
        // open; an absent entity renders as an empty 27-slot chest)
        let chest = match self.container {
            Some(Container::Chest { pos }) => self
                .sim
                .containers
                .get(&pos)
                .map(|c| c.slots.clone())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let size = self.craft_grid_size();
        let grid: Vec<vc_inventory::inventory::ItemStack> =
            self.craft_grid.iter().take(size * size).copied().collect();
        let craft_out =
            self.craft_result(&grid, size).unwrap_or(vc_inventory::inventory::ItemStack::EMPTY);
        ContainerView {
            kind,
            inv: self.player.inv.slots.clone(),
            grid,
            craft_out,
            furnace,
            brewing,
            enchant,
            trade,
            chest,
            cursor: self.cursor_stack,
        }
    }

    // ------------------------------------------------------------ update --

    /// E2E hook: break a block at world coords through the full interactive
    /// path (state edit → light update → fence re-link → particles → item
    /// drop → sim notification → invalidation), without requiring pointer
    /// lock / raycast targeting.
    fn test_break(&mut self, x: i32, y: i32, z: i32) {
        let b = self.world.get_block(x, y, z);
        if b == AIR || b == BEDROCK {
            return;
        }
        let (biome, sky, blk) = light_at(&self.world, &self.light, x, y, z);
        if let Some((old, new)) = self.world.set_block(x, y, z, AIR) {
            self.light.on_block_changed(&self.world, x, y, z, old, new);
        }
        update_fence_neighbors(&mut self.world, x, y, z);
        self.particles
            .spawn_block_break(x, y, z, b, biome, sky, blk);
        self.sim.items.drop_block(x, y, z, b, biome, sky, blk);
        notify_sim(&self.world, &mut self.sim.sched, x, y, z);
        // §27/§29: container contents spill + entity cleanup
        self.drop_container_contents([x, y, z], b);
        // §29: mining ores grants XP (vanilla amounts, fixed midpoint)
        let ore_xp = vc_gameplay::enchanting::ore_xp(b);
        if ore_xp > 0 {
            let gained = self.player.add_xp(ore_xp);
            if gained > 0 {
                self.play_event("entity.player.levelup", None, 1.0);
            }
        }
        // §21: the dig event, same as the interactive path
        self.play_event(
            vc_audio::sounds::family_event(vc_blocks::blocks::def(b).sound, true),
            Some([x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5]),
            1.0,
        );
        self.edits += 1;
    }

    /// §27/§29/§26: breaking a container block drops its contents and
    /// removes the block entity (vanilla behavior — also fixes the latent
    /// entity leak where broken furnaces stayed in the sim map forever)
    fn drop_container_contents(&mut self, pos: [i32; 3], broke: u8) {
        // Phase 3 §26: chests / dispensers / droppers / hoppers — the
        // containers module queues the spill, we turn it into item drops
        // (and if the player is mid-screen on this very container, close
        // it — the block is gone)
        if matches!(broke, CHEST | DISPENSER | DROPPER | HOPPER) {
            if matches!(
                self.container,
                Some(Container::Chest { pos: p }) if p == pos
            ) {
                self.close_container();
            }
            self.sim.containers.remove(&pos);
            self.drain_container_spills();
        }
        // Phase 5 §27: a broken spawner drops its block entity state
        if broke == SPAWNER {
            self.sim.spawners.remove(pos);
        }
        if broke == FURNACE {
            if let Some(f) = self.sim.furnaces.map.remove(&pos) {
                let (biome, sky, blk) = light_at(&self.world, &self.light, pos[0], pos[1] + 1, pos[2]);
                for s in [&f.input, &f.fuel, &f.output] {
                    if !s.is_empty() {
                        for _ in 0..s.count {
                            self.sim.items.drop_block(pos[0], pos[1] + 1, pos[2], s.block, biome, sky, blk);
                        }
                    }
                }
            }
        } else if broke == BREWING_STAND {
            if let Some(b) = self.sim.brewing.map.remove(&pos) {
                let (biome, sky, blk) = light_at(&self.world, &self.light, pos[0], pos[1] + 1, pos[2]);
                let mut slots = vec![b.ingredient, b.fuel];
                slots.extend(b.bottles.iter().copied());
                for s in &slots {
                    if !s.is_empty() {
                        for _ in 0..s.count {
                            self.sim.items.drop_block(pos[0], pos[1] + 1, pos[2], s.block, biome, sky, blk);
                        }
                    }
                }
            }
        }
    }

    /// Phase 3 §26: turn queued container spills (broken
    /// chests/dispensers/droppers/hoppers) into world item drops
    fn drain_container_spills(&mut self) {
        let spilled = std::mem::take(&mut self.sim.containers.spilled);
        for (pos, items) in spilled {
            let (biome, sky, blk) = light_at(&self.world, &self.light, pos[0], pos[1] + 1, pos[2]);
            for s in items {
                for _ in 0..s.count {
                    self.sim.items.drop_block(pos[0], pos[1] + 1, pos[2], s.block, biome, sky, blk);
                }
            }
        }
    }

    /// E2E hook: place a block / water source / redstone component.
    fn test_place(&mut self, block: u8, x: i32, y: i32, z: i32) {
        use vc_blocks::blocks::*;
        let state = match block {
            WATER => water_state(0),
            _ => default_state(block),
        };
        if let Some((old, new)) = self.world.set_block_state(x, y, z, state) {
            self.light.on_block_changed(&self.world, x, y, z, old, new);
        }
        notify_sim(&self.world, &mut self.sim.sched, x, y, z);
        self.edits += 1;
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.day_time = (self.day_time + dt / 600.0) % 1.0; // 10-minute day

        // --- bench mode: scripted camera, frame bookkeeping (§48 Phase 0)
        if let Some(bs) = self.bench.as_mut() {
            bs.t += dt;
            if self.screen == Screen::Game {
                let spawn = self.bench_spawn;
                let (pos, yaw, pitch) = bs.camera(spawn);
                self.player.pos = pos;
                self.player.vel = glam::Vec3::ZERO;
                self.player.yaw = yaw;
                self.player.pitch = pitch;
                self.player.flying = true;
                self.player.on_ground = false;
            }
        }

        // Phase 4 §18: settle incremental light updates, then fold the
        // engine's EXACT changed sections into the §12 dirty map (replaces
        // the heuristic light regions from Phase 3)
        self.light.pump(&mut self.world, 8_000);
        for (pos, mask) in self.light.take_changed() {
            self.world
                .mark_sections_dirty(pos, mask, vc_world::world::CAUSE_LIGHT);
        }

        // stream chunks (also during title/menus: the panorama keeps loading)
        crate::phase!(self.phases, crate::bench::PHASE_STREAM, self.stream());

        // particles: fixed 20 Hz sim against the live world (§16.2 pass 4)
        crate::phase!(self.phases, crate::bench::PHASE_SIM, {
            self.particles.update(dt, &self.world);
        });

        // Phase 6 simulation: scheduled ticks (fluids/gravity), random
        // ticks, item entities — same fixed-step accumulator
        crate::phase!(self.phases, crate::bench::PHASE_SIM, {
            // Phase 2: anchor the mob system before the tick (spawns/AI
            // need the player; creative flight holds all fire)
            self.sim.mobs.player = if self.screen == Screen::Game {
                Some(self.player.pos.to_array())
            } else {
                None
            };
            self.sim.mobs.player_invulnerable = self.mode.invulnerable();
            // Phase 6 §26: the sim ring follows the player chunk; radius =
            // the simulation-distance setting (default 12 covers everything
            // loaded at the default render distances — 1.16.5 behavior)
            let scope = vc_sim::sim::TickScope {
                center: self.player_chunk(),
                radius: self.settings.sim_distance,
            };
            self.sim.update(dt, &mut self.world, &mut self.light, &scope);
            // Phase 2: drain mob hits/deaths/explosions on the game thread
            self.drain_mob_events();
        });

        // Phase 2: melee cooldown recovery clock
        self.swing_t += dt;

        // §29: brewing completions → bubble sound at the stand (drained
        // here so the audio path stays on the game thread, not the sim)
        if !self.sim.brewing.completed.is_empty() {
            let done: Vec<[i32; 3]> = self.sim.brewing.completed.drain(..).collect();
            for pos in done {
                self.play_event(
                    "block.brewing_stand.bubble",
                    Some([pos[0] as f32 + 0.5, pos[1] as f32 + 0.5, pos[2] as f32 + 0.5]),
                    1.0,
                );
            }
        }

        // item pickup: entities in radius land in the hotbar
        if self.screen == Screen::Game {
            for b in self.sim.collect_items(self.player.eye().to_array()) {
                let leftover = self.player.inv.add(b, 1);
                if leftover == 0 {
                    let toast = name(b);
                    self.item_toast = Some((toast.to_string(), 2.0));
                    self.play_event("entity.item.pickup", None, 1.0);
                    self.ui.dirty = true;
                }
            }
        }

        // §21 music + ambient scheduling: a procedural pad every 2.5–4 min
        // (day/night progressions), and cave "eerie" tones when the player
        // is deep with no skylight. Both ride their own categories.
        if self.screen == Screen::Game {
            if self.time >= self.music_next {
                self.music_next = self.time + 150.0 + self.audio_rng.next_f32() * 90.0;
                let ev = if self.day_time < 0.55 {
                    "music.pad.day"
                } else {
                    "music.pad.night"
                };
                self.play_event(ev, None, 1.0);
            }
            if self.time >= self.ambient_next {
                self.ambient_next = self.time + 8.0;
                let p = &self.player.pos;
                if p.y < 45.0 {
                    let (_, sky, _) = light_at(
                        &self.world,
                        &self.light,
                        p.x.floor() as i32,
                        p.y.floor() as i32,
                        p.z.floor() as i32,
                    );
                    if sky == 0 && self.audio_rng.next_f32() < 0.12 {
                        self.play_event("ambient.eerie", None, 1.0);
                    }
                }
            }
        }

        // E2E test commands (wasm only): break/place/water/drop → full
        // interactive paths (mesh invalidation + light + particles + sim)
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(cmd) = crate::web_input::pop_test_cmd() {
                let parts: Vec<&str> = cmd.split(':').collect();
                let coords = || {
                    parts.iter()
                        .skip(1)
                        .filter_map(|v| v.parse::<i32>().ok())
                        .collect::<Vec<i32>>()
                };
                match parts.first().copied() {
                    Some("break") => {
                        let p = coords();
                        if p.len() == 3 {
                            self.test_break(p[0], p[1], p[2]);
                        }
                    }
                    Some("place") => {
                        // place:block_name:x:y:z (sand|gravel|dirt|stone|water
                        // | Phase 3: repeater|comparator|piston|sticky_piston
                        // |observer|chest|dispenser|dropper|hopper)
                        let p = coords();
                        let b = match parts.get(1).copied() {
                            Some("sand") => Some(SAND),
                            Some("gravel") => Some(GRAVEL),
                            Some("dirt") => Some(DIRT),
                            Some("stone") => Some(STONE),
                            Some("water") => Some(WATER),
                            Some("repeater") => Some(REPEATER),
                            Some("comparator") => Some(COMPARATOR),
                            Some("piston") => Some(PISTON),
                            Some("sticky_piston") => Some(STICKY_PISTON),
                            Some("observer") => Some(OBSERVER),
                            Some("chest") => Some(CHEST),
                            Some("dispenser") => Some(DISPENSER),
                            Some("dropper") => Some(DROPPER),
                            Some("hopper") => Some(HOPPER),
                            _ => None,
                        };
                        if p.len() == 3 && b.is_some() {
                            // coords() skipped the name — re-parse the tail
                            let q: Vec<i32> = parts[2..]
                                .iter()
                                .filter_map(|v| v.parse().ok())
                                .collect();
                            if q.len() == 3 {
                                self.test_place(b.unwrap(), q[0], q[1], q[2]);
                            }
                        }
                    }
                    Some("fplace") => {
                        // fplace:block:facing:x:y:z — Phase 3 components
                        // with an explicit facing (0=N,1=E,2=S,3=W);
                        // repeater/comparator delay defaults to 1 rt
                        let f: usize = parts
                            .get(2)
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0);
                        // coords after the facing field
                        let q: Vec<i32> = parts[3..]
                            .iter()
                            .filter_map(|v| v.parse().ok())
                            .collect();
                        let state = match parts.get(1).copied() {
                            Some("repeater") => Some(repeater_state(f, 1, false)),
                            Some("comparator") => Some(comparator_state(f, false, false)),
                            Some("piston") => Some(piston_state(f, false)),
                            Some("sticky_piston") => Some(sticky_piston_state(f, false)),
                            Some("observer") => Some(observer_state(f, false)),
                            _ => None,
                        };
                        if q.len() == 3 {
                            if let Some(st) = state {
                                if let Some((old, new)) = self.world.set_block_state(q[0], q[1], q[2], st) {
                                    self.light.on_block_changed(&self.world, q[0], q[1], q[2], old, new);
                                }
                                notify_sim(&self.world, &mut self.sim.sched, q[0], q[1], q[2]);
                                self.edits += 1;
                            }
                        }
                    }
                    Some("probe") => {
                        // probe:x:y:z — Phase 3 E2E read-back: decode the
                        // redstone state at a cell and report via boot log
                        let p = coords();
                        if p.len() == 3 {
                            let s = self.world.get_state(p[0], p[1], p[2]);
                            let b = vc_blocks::blocks::state_block(s);
                            let msg = match b {
                                REDSTONE_WIRE => format!(
                                    "e2e: probe ({},{},{}) WIRE power={}",
                                    p[0], p[1], p[2],
                                    vc_blocks::blocks::wire_power(s)
                                ),
                                REPEATER => {
                                    let (f, d, pw) = vc_blocks::blocks::repeater_decode(s);
                                    format!(
                                        "e2e: probe ({},{},{}) REPEATER facing={} delay={} powered={}",
                                        p[0], p[1], p[2], f, d, pw
                                    )
                                }
                                COMPARATOR => {
                                    let (f, sub, pw) = vc_blocks::blocks::comparator_decode(s);
                                    format!(
                                        "e2e: probe ({},{},{}) COMPARATOR facing={} subtract={} powered={}",
                                        p[0], p[1], p[2], f, sub, pw
                                    )
                                }
                                PISTON | STICKY_PISTON => {
                                    let (f, ext) = vc_blocks::blocks::piston_decode(s);
                                    format!(
                                        "e2e: probe ({},{},{}) PISTON facing={} extended={}",
                                        p[0], p[1], p[2], f, ext
                                    )
                                }
                                OBSERVER => {
                                    let (f, pw) = vc_blocks::blocks::observer_decode(s);
                                    format!(
                                        "e2e: probe ({},{},{}) OBSERVER facing={} powered={}",
                                        p[0], p[1], p[2], f, pw
                                    )
                                }
                                CHEST => {
                                    let key = [p[0], p[1], p[2]];
                                    let filled = self
                                        .sim
                                        .containers
                                        .get(&key)
                                        .map(|c| c.slots.iter().filter(|s| !s.is_empty()).count())
                                        .unwrap_or(0);
                                    format!(
                                        "e2e: probe ({},{},{}) CHEST filled_slots={}",
                                        p[0], p[1], p[2], filled
                                    )
                                }
                                _ => format!(
                                    "e2e: probe ({},{},{}) block={} state={}",
                                    p[0], p[1], p[2], b, s
                                ),
                            };
                            vc_render::render::report_boot_log(&msg);
                        }
                    }
                    Some("water") => {
                        let p = coords();
                        if p.len() == 3 {
                            self.test_place(WATER, p[0], p[1], p[2]);
                        }
                    }
                    Some("lever") => {
                        let p = coords();
                        if p.len() == 3 {
                            self.test_place(LEVER, p[0], p[1], p[2]);
                        }
                    }
                    Some("wire") => {
                        let p = coords();
                        if p.len() == 3 {
                            self.test_place(REDSTONE_WIRE, p[0], p[1], p[2]);
                        }
                    }
                    Some("torch") => {
                        let p = coords();
                        if p.len() == 3 {
                            self.test_place(REDSTONE_TORCH, p[0], p[1], p[2]);
                        }
                    }
                    Some("toggle") => {
                        let p = coords();
                        if p.len() == 3 {
                            vc_sim::redstone::toggle_lever(
                                &mut self.world,
                                &mut self.sim.sched,
                                p[0],
                                p[1],
                                p[2],
                            );
                            self.edits += 1;
                        }
                    }
                    // ---- Phase 7 §27 E2E: containers / crafting / smelting --
                    Some("open") => {
                        // open:<inventory|crafting|furnace> — the crafting/
                        // furnace variants need a position (defaults to
                        // two blocks below the player)
                        let pos = [
                            self.player.pos.x.floor() as i32,
                            self.player.pos.y.floor() as i32 - 2,
                            self.player.pos.z.floor() as i32,
                        ];
                        match parts.get(1).copied() {
                            Some("inventory") => {
                                self.open_container(Container::Inventory);
                                vc_render::render::report_boot_log("e2e: inventory screen open");
                            }
                            Some("crafting") => {
                                self.test_place(CRAFTING_TABLE, pos[0], pos[1], pos[2]);
                                self.open_container(Container::Crafting { pos });
                                vc_render::render::report_boot_log("e2e: crafting screen open");
                            }
                            Some("furnace") => {
                                self.test_place(FURNACE, pos[0], pos[1], pos[2]);
                                self.sim.furnaces.map.entry(pos).or_default();
                                self.open_container(Container::Furnace { pos });
                                vc_render::render::report_boot_log("e2e: furnace screen open");
                            }
                            Some("chest") => {
                                // Phase 3 §26: place a chest, seed one item
                                // into slot 0, open — the harness then reads
                                // the view back via the screenshot path
                                self.test_place(CHEST, pos[0], pos[1], pos[2]);
                                let e = self.sim.containers.entry(pos, CHEST);
                                e.slots[0] = vc_inventory::inventory::ItemStack::new(STONE, 7);
                                self.open_container(Container::Chest { pos });
                                vc_render::render::report_boot_log("e2e: chest screen open (27 slots)");
                            }
                            Some("brewing") => {
                                self.test_place(BREWING_STAND, pos[0], pos[1], pos[2]);
                                self.sim.brewing.map.entry(pos).or_default();
                                self.open_container(Container::Brewing { pos });
                                vc_render::render::report_boot_log("e2e: brewing screen open");
                            }
                            Some("enchant") => {
                                // place the table + the vanilla 15-bookshelf
                                // ring, then open with a fresh offer list
                                self.test_place(ENCHANT_TABLE, pos[0], pos[1], pos[2]);
                                for dz in [-2, 2] {
                                    for dx in -2i32..=2 {
                                        self.test_place(BOOKSHELF, pos[0] + dx, pos[1], pos[2] + dz);
                                        self.test_place(BOOKSHELF, pos[0] + dx, pos[1] + 1, pos[2] + dz);
                                    }
                                }
                                for dx in [-2, 2] {
                                    for dz in -1i32..=1 {
                                        self.test_place(BOOKSHELF, pos[0] + dx, pos[1], pos[2] + dz);
                                        self.test_place(BOOKSHELF, pos[0] + dx, pos[1] + 1, pos[2] + dz);
                                    }
                                }
                                let seed = self.world.seed;
                                let (power, offers) = {
                                    let e = self.sim.enchants.map.entry(pos).or_default();
                                    e.reroll(&self.world, pos, seed);
                                    (e.power, e.options.iter().map(|o| o.level).collect::<Vec<_>>())
                                };
                                self.open_container(Container::Enchant { pos });
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: enchant screen open (power {}/15, offers {:?})",
                                    power, offers
                                ));
                            }
                            Some("brew") => {
                                // brew:<ticks> — scripted §29 flow: place a
                                // stand, load it through REAL slot
                                // semantics (3 water bottles, wart
                                // ingredient, netherrack fuel), sim N ticks,
                                // report the bottle contents.
                                // brew:corrupt:<ticks> — Phase 4 §26 flow:
                                // a HEALING bottle + a fermented spider eye
                                // → the corruption cycle (→ Harming)
                                let corrupt = parts.get(2).copied() == Some("corrupt");
                                let n_ticks: i32 = if corrupt {
                                    // open:brew:corrupt:<ticks>
                                    parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(
                                        vc_gameplay::brewing::BREW_TICKS,
                                    )
                                } else {
                                    // open:brew:<ticks>
                                    parts
                                        .get(2)
                                        .and_then(|s| s.parse().ok())
                                        .unwrap_or(vc_gameplay::brewing::BREW_TICKS)
                                };
                                self.test_place(BREWING_STAND, pos[0], pos[1], pos[2]);
                                let entry = self.sim.brewing.map.entry(pos).or_default();
                                use vc_inventory::inventory::Inventory;
                                if corrupt {
                                    // one healing bottle + the corrupted eye
                                    let mut slot = entry.bottles[0];
                                    let mut cursor =
                                        vc_inventory::inventory::ItemStack::new(POTION_HEALING, 1);
                                    Inventory::slot_click(&mut slot, &mut cursor, false);
                                    entry.bottles[0] = slot;
                                    entry.ingredient =
                                        vc_inventory::inventory::ItemStack::new(FERMENTED_SPIDER_EYE, 1);
                                } else {
                                    // bottles through slot_click semantics
                                    for i in 0..3 {
                                        let mut slot = entry.bottles[i];
                                        let mut cursor =
                                            vc_inventory::inventory::ItemStack::new(POTION_WATER, 1);
                                        Inventory::slot_click(&mut slot, &mut cursor, false);
                                        entry.bottles[i] = slot;
                                    }
                                    entry.ingredient = vc_inventory::inventory::ItemStack::new(MUSHROOM_RED, 1);
                                }
                                entry.fuel = vc_inventory::inventory::ItemStack::new(NETHERRACK, 1);
                                drop(entry);
                                // advance the sim deterministically (the
                                // full 1.16.5-unticked scope — E2E brew
                                // fast-forward must behave like live play)
                                for _ in 0..n_ticks {
                                    self.sim.step(
                                        &mut self.world,
                                        &mut self.light,
                                        &vc_sim::sim::TickScope::everything(),
                                    );
                                }
                                let describe = |s: &vc_inventory::inventory::ItemStack| {
                                    if s.is_empty() {
                                        "-".to_string()
                                    } else {
                                        format!("{}x{}", name(s.block), s.count)
                                    }
                                };
                                let b = self
                                    .sim
                                    .brewing
                                    .map
                                    .get(&pos)
                                    .cloned()
                                    .unwrap_or_default();
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: brew {}t -> bottles [{}, {}, {}] brewed={} charges={}",
                                    n_ticks,
                                    describe(&b.bottles[0]),
                                    describe(&b.bottles[1]),
                                    describe(&b.bottles[2]),
                                    self.sim.brewing.total_brewed,
                                    b.fuel_charges
                                ));
                            }
                            _ => {}
                        }
                    }
                    Some("cclick") => {
                        // cclick:ux:uy:<l|r> — synthesize a slot click
                        let v: Vec<i32> = parts
                            .iter()
                            .skip(1)
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if v.len() >= 2 {
                            let right = parts.get(3).copied() == Some("r");
                            self.container_click(v[0], v[1], right);
                        }
                    }
                    Some("give") => {
                        // give:<block>:<count> — survival-style acquisition
                        let b = match parts.get(1).copied() {
                            Some("oak_log") => Some(OAK_LOG),
                            Some("planks") => Some(PLANKS),
                            Some("cobble") => Some(COBBLE),
                            Some("sand") => Some(SAND),
                            // §29 brewing chain
                            Some("brewing_stand") => Some(BREWING_STAND),
                            Some("glass_bottle") => Some(POTION_EMPTY),
                            Some("potion_water") => Some(POTION_WATER),
                            Some("potion_awkward") => Some(POTION_AWKWARD),
                            Some("potion_mundane") => Some(POTION_MUNDANE),
                            Some("potion_healing") => Some(POTION_HEALING),
                            Some("potion_healing_2") => Some(POTION_HEALING_II),
                            Some("mushroom_red") => Some(MUSHROOM_RED),
                            Some("mushroom_brown") => Some(MUSHROOM_BROWN),
                            Some("netherrack") => Some(NETHERRACK),
                            Some("glowstone") => Some(GLOWSTONE),
                            // Phase 4 §26: corruption chain
                            Some("potion_harming") => Some(POTION_HARMING),
                            Some("potion_harming_2") => Some(POTION_HARMING_II),
                            Some("spider_eye") => Some(SPIDER_EYE),
                            Some("fermented_eye") => Some(FERMENTED_SPIDER_EYE),
                            // §29 enchanting chain
                            Some("book") => Some(ENCHANTED_BOOK),
                            Some("lapis") => Some(LAPIS_ORE),
                            Some("enchant_table") => Some(ENCHANT_TABLE),
                            Some("bookshelf") => Some(BOOKSHELF),
                            // Phase 5: trade-payment items (E2E trade flows)
                            Some("rotten_flesh") => Some(ROTTEN_FLESH),
                            Some("emerald") => Some(EMERALD_ORE),
                            _ => None,
                        };
                        let n: u8 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
                        if let Some(b) = b {
                            let left = self.player.inv.add(b, n);
                            vc_render::render::report_boot_log(&format!(
                                "e2e: gave {n} x {} (leftover {left})",
                                name(b)
                            ));
                            self.ui.dirty = true;
                        }
                    }
                    Some("xp") => {
                        // xp:<points> — E2E shortcut: grant points through the
                        // REAL level-up curve and report level/progress
                        let pts: i32 = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(100);
                        let gained = self.player.add_xp(pts);
                        if gained > 0 {
                            self.play_event("entity.player.levelup", None, 1.0);
                        }
                        vc_render::render::report_boot_log(&format!(
                            "e2e: +{pts} xp -> level {} (+{}/{})",
                            self.player.xp_level,
                            self.player.xp_points,
                            vc_gameplay::enchanting::xp_to_next(self.player.xp_level)
                        ));
                        self.ui.dirty = true;
                    }
                    Some("enchant") => {
                        // enchant:<row> — scripted §29 flow: table + full
                        // bookshelf ring, book + lapis through the REAL slot
                        // semantics, grant levels, click the option row
                        let row: usize = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(2);
                        let pos = [
                            self.player.pos.x.floor() as i32,
                            self.player.pos.y.floor() as i32 - 2,
                            self.player.pos.z.floor() as i32,
                        ];
                        self.test_place(ENCHANT_TABLE, pos[0], pos[1], pos[2]);
                        for dz in [-2, 2] {
                            for dx in -2i32..=2 {
                                self.test_place(BOOKSHELF, pos[0] + dx, pos[1], pos[2] + dz);
                                self.test_place(BOOKSHELF, pos[0] + dx, pos[1] + 1, pos[2] + dz);
                            }
                        }
                        for dx in [-2, 2] {
                            for dz in -1i32..=1 {
                                self.test_place(BOOKSHELF, pos[0] + dx, pos[1], pos[2] + dz);
                                self.test_place(BOOKSHELF, pos[0] + dx, pos[1] + 1, pos[2] + dz);
                            }
                        }
                        let seed = self.world.seed;
                        let e = self.sim.enchants.map.entry(pos).or_default();
                        e.reroll(&self.world, pos, seed);
                        e.item = vc_inventory::inventory::ItemStack::new(ENCHANTED_BOOK, 1);
                        e.lapis = vc_inventory::inventory::ItemStack::new(LAPIS_ORE, 3);
                        // make sure the player can pay (vanilla: needs the
                        // levels — grant enough for the cost)
                        let cost = e.options.get(row).map(|o| o.cost as i32).unwrap_or(0);
                        while self.player.xp_level < cost {
                            self.player.add_xp(vc_gameplay::enchanting::xp_to_next(self.player.xp_level));
                        }
                        let before = e.options[row];
                        let level_before = self.player.xp_level;
                        drop(e);
                        self.open_container(Container::Enchant { pos });
                        // apply through the same can_apply/apply logic the
                        // option click uses (geometry-driven click is
                        // verified separately via cclick)
                        let affordable = self
                            .sim
                            .enchants
                            .map
                            .get(&pos)
                            .map(|e| e.can_apply(row, self.player.xp_level))
                            .unwrap_or(false);
                        if affordable {
                            let Some(e) = self.sim.enchants.map.get_mut(&pos) else {
                                return;
                            };
                            let Some(cost) = e.apply(row) else {
                                return;
                            };
                            if e.lapis.count >= cost {
                                e.lapis.count -= cost;
                                if e.lapis.count == 0 {
                                    e.lapis = vc_inventory::inventory::ItemStack::EMPTY;
                                }
                            }
                            self.player.spend_levels(cost as i32);
                            self.sim.enchants.total_enchanted += 1;
                            e.reroll(&self.world, pos, seed);
                            let def = vc_gameplay::enchanting::enchant_def(before.ench);
                            vc_render::render::report_boot_log(&format!(
                                "e2e: enchanted {} {} (row lvl {}) cost {cost} lvl + {cost} lapis, xp {} -> {}, book ench={}",
                                def.name,
                                vc_gameplay::enchanting::roman(before.ench_level),
                                before.level,
                                level_before,
                                self.player.xp_level,
                                self.sim.enchants.map[&pos].item.ench
                            ));
                        } else {
                            vc_render::render::report_boot_log("e2e: enchant offer not affordable");
                        }
                    }
                    Some("spawn") => {
                        // spawn:villager[:profession] — E2E: a villager near
                        // the player (auto-spawn at villages is separate)
                        let prof = parts.get(1).copied().and_then(|p| {
                            vc_gameplay::villagers::PROFESSIONS
                                .iter()
                                .position(|n| *n == p)
                                .map(|i| i as u8)
                        });
                        let pos = self.player.pos;
                        match self.sim.villagers.spawn_at(
                            pos.x.floor() as i32 + 2,
                            pos.y.floor() as i32,
                            pos.z.floor() as i32,
                            prof,
                        ) {
                            Some(id) => {
                                let v = self.sim.villagers.by_id(id).unwrap();
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: spawned villager #{id} {} at ({:.0},{:.0},{:.0})",
                                    vc_gameplay::villagers::PROFESSIONS[v.profession as usize],
                                    v.pos[0],
                                    v.pos[1],
                                    v.pos[2]
                                ));
                            }
                            None => {
                                vc_render::render::report_boot_log("e2e: villager cap reached");
                            }
                        }
                    }
                    Some("trade") => {
                        // trade:<idx> — scripted §29 flow: spawn a cleric (sells
                        // healing potions — the §29 cross-link), grant the
                        // give items, open the screen, execute the trade
                        // through the same tier/stock/XP path the TradeRow
                        // click uses (geometry-driven click verified via cclick)
                        let idx: usize = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        let pos = self.player.pos;
                        let id = self
                            .sim
                            .villagers
                            .spawn_at(
                                pos.x.floor() as i32 + 2,
                                pos.y.floor() as i32,
                                pos.z.floor() as i32,
                                Some(3), // Cleric (15-registry index)
                            )
                            .expect("villager cap");
                        let table = vc_gameplay::villagers::trades(3);
                        let idx = idx.min(table.len() - 1);
                        let tr = table[idx];
                        // grant the payment through the real add path
                        self.player.inv.add(tr.give.0, tr.give.1);
                        self.open_container(Container::Trade { villager: id });
                        // execute through the REAL tier/stock/XP path
                        if let Some((tr, _leveled)) = self.sim.villagers.execute_trade(id, idx) {
                            if self.player.inv.consume(tr.give.0, tr.give.1) {
                                let left = self.player.inv.add(tr.get.0, tr.get.1);
                                if left > 0 {
                                    self.sim.items.drop_block(
                                        pos.x.floor() as i32,
                                        pos.y.floor() as i32,
                                        pos.z.floor() as i32,
                                        tr.get.0, 2, 15, 0,
                                    );
                                }
                                self.play_event("entity.villager.trade", None, 1.0);
                                let v = self.sim.villagers.by_id(id).unwrap();
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: trade flow done - inv has {}x {} (trades {}, villager lvl {} xp {})",
                                    self.player.inv.count_of(tr.get.0),
                                    name(tr.get.0),
                                    self.sim.villagers.trades_done,
                                    v.level(),
                                    v.xp
                                ));
                            } else {
                                vc_render::render::report_boot_log("e2e: trade payment missing");
                            }
                        } else {
                            vc_render::render::report_boot_log("e2e: trade rejected (tier/stock)");
                        }
                    }
                    Some("tradelevel") => {
                        // tradelevel:<n> — Phase 5 career flow: spawn a
                        // cleric, run n tier-1 trades through the real
                        // path, report the level-ups (5 trades × 2 XP =
                        // Apprentice at the VERIFIED 10-XP threshold)
                        let n: usize = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(5);
                        let pos = self.player.pos;
                        let id = self
                            .sim
                            .villagers
                            .spawn_at(
                                pos.x.floor() as i32 + 2,
                                pos.y.floor() as i32,
                                pos.z.floor() as i32,
                                Some(3), // Cleric
                            )
                            .expect("villager cap");
                        self.open_container(Container::Trade { villager: id });
                        let mut done = 0;
                        for _ in 0..n {
                            // row 0: Rotten Flesh 12 → Emerald 1 (tier 1)
                            let Some((tr, _)) = self.sim.villagers.execute_trade(id, 0) else {
                                break;
                            };
                            self.player.inv.add(tr.give.0, tr.give.1); // grant payment
                            if self.player.inv.consume(tr.give.0, tr.give.1) {
                                let _ = self.player.inv.add(tr.get.0, tr.get.1);
                                done += 1;
                            }
                        }
                        let v = self.sim.villagers.by_id(id).unwrap();
                        let offers = v.offers();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: tradelevel {done} trades -> level {} ({}), xp {}, visible offers {} (tier-2 unlocked: {})",
                            v.level(),
                            vc_gameplay::villagers::level_name(v.level()),
                            v.xp,
                            offers.len(),
                            offers.contains(&2)
                        ));
                    }
                    Some("spawner") => {
                        // spawner:<mob>:<ticks> — Phase 5 §27 flow: place a
                        // spawner 5 blocks from the player, register it,
                        // step the sim through ticks with the player in
                        // range, report the spawned mobs (the harness also
                        // verifies the 6-mob cap by reading F3 stats)
                        let mob = match parts.get(1).copied() {
                            Some("skeleton") => SPAWNER_SKELETON,
                            Some("spider") => SPAWNER_SPIDER,
                            _ => SPAWNER_ZOMBIE,
                        };
                        let n_ticks: i32 = parts
                            .get(2)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(30);
                        let pos = self.player.pos;
                        let p = [
                            pos.x.floor() as i32 + 5,
                            pos.y.floor() as i32,
                            pos.z.floor() as i32,
                        ];
                        self.test_place(SPAWNER, p[0], p[1], p[2]);
                        self.world.set_block_state(
                            p[0],
                            p[1],
                            p[2],
                            vc_blocks::blocks::spawner_state(mob),
                        );
                        self.sim.spawners.register(p, mob);
                        self.sim.spawners.map.get_mut(&p).unwrap().delay = 0;
                        let kind = vc_gameplay::spawners::mob_kind(mob);
                        let before = self
                            .sim
                            .mobs
                            .list
                            .iter()
                            .filter(|m| m.kind == kind)
                            .count();
                        for _ in 0..n_ticks {
                            self.sim.step(&mut self.world, &mut self.light, &vc_sim::sim::TickScope::everything());
                        }
                        let after = self
                            .sim
                            .mobs
                            .list
                            .iter()
                            .filter(|m| m.kind == kind)
                            .count();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: spawner ({}) {n_ticks}t -> mobs {} -> {} (cycles {}, spawned total {})",
                            match mob {
                                SPAWNER_SKELETON => "skeleton",
                                SPAWNER_SPIDER => "spider",
                                _ => "zombie",
                            },
                            before,
                            after,
                            self.sim.spawners.map.get(&p).unwrap().cycles,
                            self.sim.spawners.spawned_total
                        ));
                    }
                    Some("dungeon") => {
                        // dungeon — Phase 5 §27 E2E: find the nearest
                        // dungeon roll near the player (±8 chunks), force
                        // its chunk through the real generator, register
                        // entities + loot, teleport the player inside
                        let gen = vc_world::gen::TerrainGen::for_dimension(
                            self.world.seed,
                            self.world.dimension,
                        );
                        let pcx = (self.player.pos.x.floor() as i32) >> 4;
                        let pcz = (self.player.pos.z.floor() as i32) >> 4;
                        let mut found = None;
                        'scan: for r in 0..=8i32 {
                            for dz in -r..=r {
                                for dx in -r..=r {
                                    if dx.abs() != r && dz.abs() != r {
                                        continue; // ring walk
                                    }
                                    if let Some(room) = gen.dungeon_in_chunk(pcx + dx, pcz + dz) {
                                        found = Some(((pcx + dx, pcz + dz), room));
                                        break 'scan;
                                    }
                                }
                            }
                        }
                        match found {
                            None => {
                                vc_render::render::report_boot_log(
                                    "e2e: no dungeon within ±8 chunks (regenerate for a denser roll)",
                                );
                            }
                            Some(((cx, cz), room)) => {
                                // force-generate the chunk (the real path)
                                let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
                                let pos = (cx, cz);
                                self.world.insert_generated(pos, chunk.clone(), Vec::new());
                                self.light.init_chunk(&mut self.world, pos);
                                for (lpos, lmask) in self.light.take_changed() {
                                    self.world
                                        .mark_sections_dirty(lpos, lmask, vc_world::world::CAUSE_LIGHT);
                                }
                                self.register_block_entities(pos, &chunk, true);
                                // teleport into the room center
                                self.player.pos.x = (room.x0 + room.size / 2) as f32 + 0.5;
                                self.player.pos.y = room.y0 as f32 + 0.2;
                                self.player.pos.z = (room.z0 + room.size / 2) as f32 + 0.5;
                                // stand spot: the spawner is the center —
                                // offset onto free floor next to it
                                self.player.pos.x += 2.0;
                                let mob = match room.mob {
                                    SPAWNER_SKELETON => "skeleton",
                                    SPAWNER_SPIDER => "spider",
                                    _ => "zombie",
                                };
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: dungeon at chunk ({cx},{cz}) center ({},{},{}) size {} mob {mob} chests {}",
                                    room.x0 + room.size / 2,
                                    room.y0,
                                    room.z0 + room.size / 2,
                                    room.size,
                                    room.chest_count
                                ));
                            }
                        }
                    }
                    // ---- Phase 10 E2E: structures + biomes ----
                    Some("mineshaft") => {
                        // mineshaft — find the nearest shaft roll within
                        // ±10 chunks, teleport into its parlor, report the
                        // layout (corridors + y)
                        let gen = &self.world.gen;
                        let pcx = (self.player.pos.x.floor() as i32) >> 4;
                        let pcz = (self.player.pos.z.floor() as i32) >> 4;
                        let mut found = None;
                        'scan: for r in 0..=10i32 {
                            for dz in -r..=r {
                                for dx in -r..=r {
                                    if dx.abs() != r && dz.abs() != r {
                                        continue;
                                    }
                                    for ms in
                                        gen.mineshafts_near((pcx + dx) * 16, (pcz + dz) * 16)
                                    {
                                        found = Some(((pcx + dx, pcz + dz), ms));
                                        break 'scan;
                                    }
                                }
                            }
                        }
                        match found {
                            None => vc_render::render::report_boot_log("e2e: no mineshaft within ±10 chunks (0.4%/chunk — try more area)"),
                            Some(((cx, cz), ms)) => {
                                let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
                                let pos = (cx, cz);
                                self.world.insert_generated(pos, chunk.clone(), Vec::new());
                                self.light.init_chunk(&mut self.world, pos);
                                for (lpos, lmask) in self.light.take_changed() {
                                    self.world
                                        .mark_sections_dirty(lpos, lmask, vc_world::world::CAUSE_LIGHT);
                                }
                                self.register_block_entities(pos, &chunk, true);
                                self.player.pos.x = ms.x as f32 + 0.5;
                                self.player.pos.y = (ms.y + 1) as f32 + 0.2;
                                self.player.pos.z = ms.z as f32 + 0.5;
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: mineshaft at chunk ({cx},{cz}) parlor ({},{},{}) corridors {} (lens {:?})",
                                    ms.x, ms.y, ms.z, ms.corridors.len(),
                                    ms.corridors.iter().map(|c| c.2).collect::<Vec<_>>()
                                ));
                            }
                        }
                    }
                    Some("pyramid") => {
                        // pyramid — locate the nearest desert pyramid,
                        // teleport to its hidden treasure room, report
                        let gen = &self.world.gen;
                        let px = self.player.pos.x.floor() as i32;
                        let pz = self.player.pos.z.floor() as i32;
                        // search outward region by region
                        let mut found = None;
                        'scan: for r in 0..=6i32 {
                            for rrz in -r..=r {
                                for rrx in -r..=r {
                                    if rrx.abs() != r && rrz.abs() != r {
                                        continue;
                                    }
                                    let rx = (px / (32 * 16)) + rrx;
                                    let rz = (pz / (32 * 16)) + rrz;
                                    if let Some((wx, wz)) = gen.pyramid_center_pub(rx, rz) {
                                        found = Some((wx, wz));
                                        break 'scan;
                                    }
                                }
                            }
                        }
                        match found {
                            None => vc_render::render::report_boot_log("e2e: no desert pyramid nearby (desert-gated, 1 per 32×32-chunk region)"),
                            Some((wx, wz)) => {
                                // read everything off `gen` FIRST (its
                                // borrow ends before the world mutations —
                                // held-across-mutation fails borrowck on
                                // the wasm target)
                                let base = gen.column(wx, wz).height as i32;
                                let floor = base - 11;
                                let cx = wx >> 4;
                                let cz = wz >> 4;
                                let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
                                let pos = (cx, cz);
                                self.world.insert_generated(pos, chunk.clone(), Vec::new());
                                self.light.init_chunk(&mut self.world, pos);
                                for (lpos, lmask) in self.light.take_changed() {
                                    self.world
                                        .mark_sections_dirty(lpos, lmask, vc_world::world::CAUSE_LIGHT);
                                }
                                self.register_block_entities(pos, &chunk, true);
                                self.player.pos.x = wx as f32 + 0.5;
                                self.player.pos.y = floor as f32 + 0.2;
                                self.player.pos.z = wz as f32 + 0.5;
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: desert pyramid at ({wx},{wz}) pit floor y={floor} (4 chests, desert_pyramid loot)"
                                ));
                            }
                        }
                    }
                    Some("chestloot") => {
                        // chestloot:<x>:<y>:<z> — report what rolled into
                        // the structure chest at a position: the Phase 10
                        // loot seam (fresh chunk → register_block_entities
                        // → chest_table_for attribution → fill_structure_
                        // chest through the data-pack pipeline) verified
                        // live, without needing crosshair aim
                        let c = coords();
                        if c.len() == 3 {
                            let p = [c[0], c[1], c[2]];
                            match self.sim.containers.map.get(&p) {
                                None => vc_render::render::report_boot_log(&format!(
                                    "e2e: chestloot ({},{},{}) — no container entity",
                                    p[0], p[1], p[2]
                                )),
                                Some(inv) => {
                                    let items: Vec<String> = inv
                                        .slots
                                        .iter()
                                        .filter(|s| !s.is_empty())
                                        .map(|s| {
                                            format!(
                                                "{} x{}",
                                                vc_blocks::blocks::name(s.block),
                                                s.count
                                            )
                                        })
                                        .collect();
                                    vc_render::render::report_boot_log(&format!(
                                        "e2e: chestloot ({},{},{}) — {} stacks: [{}]",
                                        p[0],
                                        p[1],
                                        p[2],
                                        items.len(),
                                        items.join(", ")
                                    ));
                                }
                            }
                        } else {
                            vc_render::render::report_boot_log(
                                "e2e: chestloot:<x>:<y>:<z> — report the loot in the chest at a position",
                            );
                        }
                    }
                    Some("stronghold") => {
                        // stronghold — report ring-1 positions (VERIFIED:
                        // 3 strongholds, 1280-2816 blocks, ~120° apart) and
                        // teleport to the first one's portal room. The room
                        // centers 17 blocks WEST of the anchor, so the
                        // RING-CENTER chunk's 3×3 neighborhood is what holds
                        // the room (each nearby chunk emits its own part of
                        // the layout, the village/mineshaft discipline).
                        let gen = &self.world.gen;
                        let sh = gen.strongholds();
                        let (sx, sz) = sh[0];
                        let (rcx, rcz) = ((sx - 17) >> 4, sz >> 4);
                        // phase 1: generate the 3×3 neighborhood while `gen`
                        // is borrowed (the player's world insert needs &mut)
                        let mut made = Vec::new();
                        for dcx in -1..=1i32 {
                            for dcz in -1..=1i32 {
                                let (chunk, _) = gen.generate_chunk(rcx + dcx, rcz + dcz, Vec::new());
                                made.push(((rcx + dcx, rcz + dcz), chunk));
                            }
                        }
                        // phase 2: insert + light + register entities
                        for (pos, chunk) in made {
                            self.world.insert_generated(pos, chunk.clone(), Vec::new());
                            self.light.init_chunk(&mut self.world, pos);
                            for (lpos, lmask) in self.light.take_changed() {
                                self.world
                                    .mark_sections_dirty(lpos, lmask, vc_world::world::CAUSE_LIGHT);
                            }
                            self.register_block_entities(pos, &chunk, true);
                        }
                        // the portal room center (portal ring + frames)
                        self.player.pos.x = (sx - 17) as f32 + 0.5;
                        self.player.pos.y = 21.0;
                        self.player.pos.z = sz as f32 + 0.5;
                        let dist = ((sx * sx + sz * sz) as f32).sqrt();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: stronghold ring 1: {} at dist {:.0} ({}..{} verified) — teleported to portal room of #1 at ({sx},{sz})",
                            sh.len(), dist, 1280, 2816
                        ));
                    }
                    Some("biome") => {
                        // biome — report the biome under the player
                        let x = self.player.pos.x.floor() as i32;
                        let z = self.player.pos.z.floor() as i32;
                        let col = self.world.gen.column(x, z);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: biome at ({x},{z}) = {} (h {}) — 14 biomes total (Phase 10: Taiga/Birch Forest/Jungle/Savanna/Swamp/Badlands)",
                            col.biome.name(), col.height
                        ));
                    }
                    Some("fill") => {
                        // fill: — E2E shortcut for the bottle-at-water
                        // interaction: every empty glass bottle in the
                        // inventory becomes a water bottle (the interactive
                        // path requires submersion, verified separately)
                        let empties = self.player.inv.count_of(POTION_EMPTY);
                        if empties > 0 {
                            self.player.inv.consume(POTION_EMPTY, empties as u8);
                            let left = self.player.inv.add(POTION_WATER, empties as u8);
                            vc_render::render::report_boot_log(&format!(
                                "e2e: filled {empties} bottles (leftover {left})"
                            ));
                            self.ui.dirty = true;
                        } else {
                            vc_render::render::report_boot_log("e2e: no glass bottles to fill");
                        }
                    }
                    Some("drink") => {
                        // drink:<potion> — consume one potion from the
                        // inventory, report the health effect (§29)
                        let b = match parts.get(1).copied() {
                            Some("potion_water") => Some(POTION_WATER),
                            Some("potion_awkward") => Some(POTION_AWKWARD),
                            Some("potion_mundane") => Some(POTION_MUNDANE),
                            Some("potion_healing") => Some(POTION_HEALING),
                            Some("potion_healing_2") => Some(POTION_HEALING_II),
                            Some("potion_harming") => Some(POTION_HARMING),
                            Some("potion_harming_2") => Some(POTION_HARMING_II),
                            _ => None,
                        };
                        match b {
                            None => vc_render::render::report_boot_log(
                                "e2e: drink:<potion_water|potion_awkward|potion_mundane|potion_healing|potion_healing_2|potion_harming|potion_harming_2>",
                            ),
                            Some(b) if self.player.inv.consume(b, 1) => {
                                let before = self.player.health;
                                // Phase 4 §26: signed instant-effect amounts —
                                // healing restores, harming damages (through
                                // the same mode rules as the hurt command)
                                if let Some(h) = vc_gameplay::brewing::potion_heal(b) {
                                    if h > 0.0 {
                                        self.player.heal(h);
                                    } else if !self.mode.invulnerable() {
                                        let _ = self.player.damage(-h);
                                        self.check_death();
                                    }
                                }
                                self.player.inv.add(POTION_EMPTY, 1);
                                self.play_event("entity.generic.drink", None, 0.9);
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: drank {} hp {:.1} -> {:.1}",
                                    name(b),
                                    before,
                                    self.player.health
                                ));
                                self.ui.dirty = true;
                            }
                            Some(_) => vc_render::render::report_boot_log("e2e: no such potion"),
                        }
                    }
                    Some("craft") => {
                        // craft:<log|table|furnace> — full flow: opens the
                        // right container, moves ingredients from the
                        // inventory into the grid through slot-click
                        // semantics, matches + consumes, lands the result
                        let recipe = parts.get(1).copied().unwrap_or("log");
                        let cells: &[usize] = match recipe {
                            "table" => &[0, 1, 2, 3],
                            "furnace" => &[0, 1, 2, 3, 5, 6, 7, 8],
                            _ => &[0], // log → planks (1x1 recipe)
                        };
                        let ing: u8 = match recipe {
                            "table" => PLANKS,
                            "furnace" => COBBLE,
                            _ => OAK_LOG,
                        };
                        // the 3×3 recipes need the crafting table open
                        if recipe == "furnace"
                            && !matches!(self.container, Some(Container::Crafting { .. }))
                        {
                            let pos = [
                                self.player.pos.x.floor() as i32,
                                self.player.pos.y.floor() as i32 - 2,
                                self.player.pos.z.floor() as i32,
                            ];
                            self.open_container(Container::Crafting { pos });
                        } else if self.container.is_none() {
                            self.open_container(Container::Inventory);
                        }
                        // ensure ingredients exist
                        if self.player.inv.count_of(ing) < cells.len() as u32 {
                            self.player.inv.add(ing, cells.len() as u8);
                        }
                        // move one item into each grid cell through the REAL
                        // slot_click semantics (cursor round-trip per cell)
                        use vc_inventory::inventory::Inventory;
                        for &c in cells {
                            if let Some(i) = self
                                .player
                                .inv
                                .slots
                                .iter()
                                .position(|s| s.block == ing && s.count > 0)
                            {
                                self.cursor_stack = vc_inventory::inventory::ItemStack::new(ing, 1);
                                self.player.inv.slots[i].count -= 1;
                                if self.player.inv.slots[i].count == 0 {
                                    self.player.inv.slots[i] = vc_inventory::inventory::ItemStack::EMPTY;
                                }
                                let mut grid = self.craft_grid[c];
                                Inventory::slot_click(&mut grid, &mut self.cursor_stack, false);
                                self.craft_grid[c] = grid;
                            }
                        }
                        self.cursor_stack = vc_inventory::inventory::ItemStack::EMPTY;
                        // match + consume through the real recipe engine,
                        // land the result in the inventory (the CraftOut
                        // click path — verified separately by cclick tests)
                        let size = self.craft_grid_size();
                        let grid: Vec<vc_inventory::inventory::ItemStack> =
                            self.craft_grid.iter().take(size * size).copied().collect();
                        let msg = match self.craft_result(&grid, size) {
                            Some(out) => {
                                vc_gameplay::craft::consume_grid(&mut self.craft_grid[..size * size]);
                                let left = self.player.inv.add(out.block, out.count);
                                format!(
                                    "e2e: crafted {} x {} (leftover {left})",
                                    out.count,
                                    name(out.block)
                                )
                            }
                            None => "e2e: craft FAILED — no recipe match".to_string(),
                        };
                        vc_render::render::report_boot_log(&msg);
                        self.ui.dirty = true;
                    }
                    Some("smelt") => {
                        // smelt:<x>:<y>:<z> — furnace block entity + world
                        // block, input SAND + fuel PLANKS, fast-forward 260
                        // sim steps, report the output + lit state swap
                        let p = coords();
                        let pos = if p.len() == 3 {
                            [p[0], p[1], p[2]]
                        } else {
                            [
                                self.player.pos.x.floor() as i32,
                                self.player.pos.y.floor() as i32 - 2,
                                self.player.pos.z.floor() as i32,
                            ]
                        };
                        self.test_place(FURNACE, pos[0], pos[1], pos[2]);
                        let mut f = vc_gameplay::furnace::FurnaceState::default();
                        f.input = vc_inventory::inventory::ItemStack::new(SAND, 2);
                        f.fuel = vc_inventory::inventory::ItemStack::new(PLANKS, 2);
                        self.sim.furnaces.map.insert(pos, f);
                        // fast-forward: 260 ticks = ignite + 200 cook + slack
                        let mut lit = false;
                        for _ in 0..260 {
                            let changed = self.sim.furnaces.tick(&mut self.world);
                            if !changed.is_empty() {
                                lit = true;
                            }
                        }
                        let out = self
                            .sim
                            .furnaces
                            .map
                            .get(&pos)
                            .map(|f| (f.output, f.is_burning()))
                            .unwrap_or_default();
                        let state = self.world.get_state(pos[0], pos[1], pos[2]);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: smelt output={} x {} burning={} lit_swapped={} state={}",
                            out.0.count,
                            name(out.0.block),
                            out.1,
                            lit,
                            state == vc_blocks::blocks::FURNACE_LIT,
                        ));
                        self.edits += 1;
                        self.ui.dirty = true;
                    }
                    Some("shader") => {
                        // Phase 11 §34: shader:<mode> — set the shader mode
                        // (0..2 engine grades, 3.. packs) exactly like the
                        // options row; E2E verifies via stats + pixels
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<u8>().ok()) {
                            self.settings.shader = v;
                            self.after_settings_change();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: shader mode {v} = {}",
                                self.shader_mode_name(v)
                            ));
                        }
                    }
                    Some("dim") => {
                        // §28 E2E: dim:<0|1> — dimension travel through the
                        // full pipeline (world swap, 8:1 coords, mesh reset,
                        // Loading snap); stats `dim`/`dimName` + the nether
                        // fog verify it. Travel lands asynchronously — the
                        // Loading screen holds the player until the spawn
                        // chunk meshes, then returns to the game.
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<u8>().ok()) {
                            let dim = vc_world::world::Dimension::from_u8(v);
                            let changed = dim != self.world.dimension;
                            if changed {
                                self.travel_to_dimension(dim);
                            }
                            vc_render::render::report_boot_log(&format!(
                                "e2e: dim {} ({}) changed={} traveling={}",
                                dim.id(),
                                dim.name(),
                                changed,
                                self.traveling
                            ));
                        }
                    }
                    // ---- Phase 6 §26 E2E: rendering-quality settings --
                    Some("sd") => {
                        // sd:<chunks> — simulation distance (stats `sd`)
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<i32>().ok()) {
                            self.settings.sim_distance = v.clamp(5, 32);
                            vc_render::render::report_boot_log(&format!(
                                "e2e: sim distance {} (rd {}, ring center follows player)",
                                self.settings.sim_distance,
                                self.settings.render_distance
                            ));
                        }
                    }
                    Some("mip") => {
                        // mip:<0-4> — mipmap levels (stats `mip`; renderer
                        // atlas rebuild + sampler change verified via pixels)
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<u8>().ok()) {
                            self.settings.mipmap_levels = v.min(4);
                            self.after_settings_change();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: mipmap levels {} (atlas rebuilt)",
                                self.settings.mipmap_levels
                            ));
                        }
                    }
                    Some("aniso") => {
                        // aniso:<1|2|4|8|16> — anisotropic filtering
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<u8>().ok()) {
                            self.settings.aniso = v.clamp(1, 16);
                            self.after_settings_change();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: anisotropy {}x",
                                self.settings.aniso
                            ));
                        }
                    }
                    Some("msaa") => {
                        // msaa:<0|4|8> — MSAA (device-gated; stats `msaa`
                        // reports the ACTIVE count)
                        if let Some(v) = parts.get(1).and_then(|s| s.parse::<u8>().ok()) {
                            self.settings.msaa = if v >= 6 { 8 } else if v >= 2 { 4 } else { 0 };
                            self.after_settings_change();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: msaa {} (device max {}, active {})",
                                self.settings.msaa,
                                self.renderer.msaa_supported(),
                                self.renderer.msaa()
                            ));
                        }
                    }
                    Some("occl") => {
                        // occl:<0|1> — chunk-graph occlusion culling toggle
                        // (stats `culled` responds: 0 when off)
                        if let Some(v) = parts.get(1) {
                            self.settings.occlusion = *v != "0";
                            self.after_settings_change();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: occlusion {} (culled counter: {})",
                                self.settings.occlusion,
                                self.stats.culled
                            ));
                        }
                    }
                    Some("gmesh") => {
                        // gmesh:<0|1|2> — Phase 7 GPU compute meshing toggle.
                        // 2 = force-GPU even on SwiftShader (same as 1 today —
                        // documented; the flag exists so E2E scripts can
                        // express intent). Toggling remeshes every loaded
                        // chunk through the NEW backend; the e2e log reports
                        // the mesher's completed-job counter (gpumesh stat)
                        // so the harness can verify chunks actually flowed
                        // through the compute path.
                        if let Some(v) = parts.get(1) {
                            self.settings.gpu_meshing = *v != "0";
                            self.remesh_all();
                            self.after_settings_change();
                            let backend = match (&self.renderer.gpu_mesh, self.settings.gpu_meshing) {
                                (Some(m), true) => {
                                    format!("GPU (done {})", m.jobs_done)
                                }
                                (Some(_), false) => "CPU (setting off)".to_string(),
                                (None, _) => "CPU (no compute adapter)".to_string(),
                            };
                            vc_render::render::report_boot_log(&format!(
                                "e2e: gpu meshing {backend}"
                            ));
                        }
                    }
                    // ---- Phase 8 E2E: Iris integration interface ----
                    Some("iris") => {
                        // iris — report the Phase 8 interface state honestly:
                        // * native: every pack the boot scan structure-validated
                        //   (full summary line per pack) + the translator seam
                        // * wasm: no filesystem → no packs, and the
                        //   wasm-reachable surface (properties document parse,
                        //   stage-directive parse, translator status) exercised
                        //   LIVE on the embedded demo so the harness proves the
                        //   interface itself works on the web build
                        let packs = self.iris_packs.len();
                        let trans = vc_render::iris::translator().id();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: iris interface — packs={packs} translator={}",
                            if trans == "none (vc-iris sister project not registered)" {
                                "none"
                            } else {
                                trans
                            }
                        ));
                        if packs > 0 {
                            for p in &self.iris_packs {
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: iris pack: {}",
                                    p.summary()
                                ));
                            }
                        }
                        // live wasm-reachable surface check (same on native —
                        // it proves the interface is wired, not just compiled)
                        let props = vc_render::iris::ShadersProperties::parse(
                            vc_render::iris::DEMO_PROPERTIES,
                        );
                        let (version, targets) = vc_render::iris::parse_stage_directives(
                            Some(vc_render::iris::DEMO_STAGE_GLSL),
                        );
                        vc_render::render::report_boot_log(&format!(
                            "e2e: iris demo parse — profiles={} sliders={} stage=GLSL-{} targets={:?}",
                            props.profiles().len(),
                            props.sliders().len(),
                            version.as_deref().unwrap_or("?"),
                            targets
                        ));
                    }
                    // ---- Phase 9 E2E: data packs ----
                    Some("dp") => {
                        // dp — report the active world's data packs
                        // (native scan results; wasm: honestly empty)
                        if self.data.packs.is_empty() {
                            vc_render::render::report_boot_log(
                                "e2e: data packs — 0 (no filesystem on wasm; use dpdemo)",
                            );
                        } else {
                            for p in &self.data.packs {
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: data pack: {}",
                                    p.summary()
                                ));
                            }
                            vc_render::render::report_boot_log(&format!(
                                "e2e: data packs — {} packs, {} recipes, {} loot tables, {} tags",
                                self.data.packs.len(),
                                self.data.recipes.len(),
                                self.data.loot_tables.len(),
                                self.data.tags.len()
                            ));
                        }
                    }
                    Some("dpdemo") => {
                        // dpdemo — run the EMBEDDED demo data pack (the
                        // genuine 1.16.5 JSON grammar) through the REAL
                        // scan→parse→match→roll code path. Works on every
                        // platform (the wasm proof that the whole
                        // pipeline executes, not just compiles).
                        let files = vc_pack::datapack::MemoryFiles::demo();
                        match vc_pack::datapack::scan_pack("demo", &files) {
                            Some(report) => {
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: dpdemo scan — {}",
                                    report.summary()
                                ));
                                let loaded = vc_pack::datapack::LoadedData::from_reports(
                                    vec![report],
                                );
                                // shaped recipe: 2x2 cobble -> 4 stone bricks
                                let grid = vec![
                                    vc_pack::datapack::GridItem::item(
                                        "minecraft:cobblestone",
                                        5,
                                    ),
                                    vc_pack::datapack::GridItem::item(
                                        "minecraft:cobblestone",
                                        5,
                                    ),
                                    vc_pack::datapack::GridItem::item(
                                        "minecraft:cobblestone",
                                        5,
                                    ),
                                    vc_pack::datapack::GridItem::item(
                                        "minecraft:cobblestone",
                                        5,
                                    ),
                                ];
                                let craft = loaded
                                    .match_grid(&grid, 2)
                                    .map(|(b, c)| format!("{} x{c}", name(b)))
                                    .unwrap_or_else(|| "NO MATCH".into());
                                // tag-driven shapeless: red wool -> string
                                let wool = vec![vc_pack::datapack::GridItem::item(
                                    "minecraft:red_wool",
                                    1,
                                )];
                                let craft2 = loaded
                                    .match_grid(&wool, 1)
                                    .map(|(b, c)| format!("{} x{c}", name(b)))
                                    .unwrap_or_else(|| "NO MATCH".into());
                                // loot table: one roll of the weighted table
                                let mut rng = vc_rng::rng::Rng::new(99);
                                let loot = loaded
                                    .roll("demo:demo_loot", &mut rng)
                                    .map(|stacks| {
                                        stacks
                                            .iter()
                                            .map(|(b, c)| format!("{} x{c}", name(*b)))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_else(|| "NO TABLE".into());
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: dpdemo craft 2x2 cobble -> {craft}; red wool -> {craft2}"
                                ));
                                vc_render::render::report_boot_log(&format!(
                                    "e2e: dpdemo loot demo:demo_loot -> [{loot}]"
                                ));
                            }
                            None => {
                                vc_render::render::report_boot_log(
                                    "e2e: dpdemo scan FAILED (demo pack invalid)",
                                );
                            }
                        }
                    }
                    Some("dloot") => {
                        // dloot[:n] — roll the dungeon seam n times
                        // (default 5) through the ACTIVE data (pack
                        // override if the world ships one, else the
                        // palette-limited builtin default)
                        let n: u32 = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(5)
                            .clamp(1, 50);
                        let mut rng = vc_rng::rng::Rng::new(1234);
                        let mut total = 0usize;
                        for i in 0..n {
                            let stacks = self
                                .data
                                .roll("minecraft:chests/simple_dungeon", &mut rng)
                                .unwrap_or_default();
                            total += stacks.len();
                            let items = stacks
                                .iter()
                                .map(|(b, c)| format!("{} x{c}", name(*b)))
                                .collect::<Vec<_>>()
                                .join(", ");
                            vc_render::render::report_boot_log(&format!(
                                "e2e: dloot roll {} -> [{}]",
                                i + 1,
                                items
                            ));
                        }
                        vc_render::render::report_boot_log(&format!(
                            "e2e: dloot {} rolls, {} stacks total (packs {}, tables {})",
                            n,
                            total,
                            self.data.packs.len(),
                            self.data.loot_tables.len()
                        ));
                    }
                    // ---- Phase 1 E2E: game modes / world creation / death --
                    Some("world") => {
                        // world:<survival|creative|hardcore>[:seed] — create a
                        // world through the REAL create pipeline (seed parse
                        // + reset_world + Loading snap) without UI clicks
                        let mode = match parts.get(1).copied() {
                            Some("creative") => {
                                vc_gameplay::modes::GameMode::Creative
                            }
                            Some("hardcore") => {
                                vc_gameplay::modes::GameMode::Hardcore
                            }
                            _ => vc_gameplay::modes::GameMode::Survival,
                        };
                        let seed = parts
                            .get(2)
                            .and_then(|s| vc_gameplay::modes::parse_seed(s))
                            .unwrap_or(12345);
                        self.reset_world(seed, mode, "E2E World".into(), None);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: world mode={} seed={} (loading snap follows)",
                            mode.label(),
                            seed
                        ));
                    }
                    Some("hurt") => {
                        // hurt:<hp> — direct damage through the mode rules
                        // (creative absorbs it — immunity is observable)
                        let dmg: f32 = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(4.0);
                        if self.mode.invulnerable() {
                            vc_render::render::report_boot_log(&format!(
                                "e2e: hurt {dmg} absorbed (creative immunity), health {}",
                                self.player.health
                            ));
                        } else {
                            let applied = self.player.damage(dmg);
                            self.play_event("entity.player.hurt", None, 1.0);
                            self.death_cause = "E2E DAMAGE".into();
                            vc_render::render::report_boot_log(&format!(
                                "e2e: hurt {dmg} applied {applied}, health {} mode {}",
                                self.player.health,
                                self.mode.label()
                            ));
                            self.ui.dirty = true;
                        }
                    }
                    Some("fall") => {
                        // fall:<blocks> — teleport up, let real gravity +
                        // the real fall-damage path apply (MC-12357)
                        let blocks: f32 = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(10.0);
                        self.player.pos.y += blocks;
                        self.player.vel.y = 0.0;
                        self.player.reset_fall();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: fall {blocks} armed (health {})",
                            self.player.health
                        ));
                    }
                    Some("respawn") => {
                        // respawn:<hp> — set health then trigger the death
                        // check manually; the stats screen/mode/health
                        // fields verify the outcome
                        let hp: f32 = parts
                            .get(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        self.player.health = hp;
                        self.check_death();
                        vc_render::render::report_boot_log(&format!(
                            "e2e: respawn-check hp={hp} -> screen {} mode {} dead-lock {}",
                            self.screen.name(),
                            self.mode.label(),
                            self.hardcore_dead
                        ));
                    }
                    // ---- Phase 2 E2E: mob spawning + combat probes --
                    Some("mob") => {
                        // mob:<kind>[:count] — spawn near the player through
                        // the REAL MobSystem (light rules apply to natural
                        // spawning only; explicit spawns are unconditional)
                        let kind = parts.get(1).copied().and_then(vc_gameplay::mobs::MobKind::from_name);
                        let n: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
                        if let Some(kind) = kind {
                            let (px, py, pz) = (
                                self.player.pos.x.floor() as i32 + 2,
                                self.player.pos.y.floor() as i32,
                                self.player.pos.z.floor() as i32,
                            );
                            let mut spawned = 0;
                            for i in 0..n.min(16) {
                                let dx = ((i % 4) as i32) - 1;
                                let dz = ((i / 4) as i32) - 1;
                                if self
                                    .sim
                                    .mobs
                                    .spawn_at(kind, px + dx * 2, py, pz + dz * 2)
                                    .is_some()
                                {
                                    spawned += 1;
                                }
                            }
                            vc_render::render::report_boot_log(&format!(
                                "e2e: spawned {spawned} x {} (alive {})",
                                kind.name(),
                                self.sim.mobs.len()
                            ));
                        }
                    }
                    Some("attack") => {
                        // attack:<p> — E2E: swing at the crosshair mob with a
                        // forced cooldown fraction (bypasses the mouse)
                        let p: f32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1.0);
                        self.swing_t = p * 10.0; // big enough = charged
                        if self.try_attack_mob() {
                            vc_render::render::report_boot_log(&format!(
                                "e2e: attack p={p} -> mob hit (alive {})",
                                self.sim.mobs.len()
                            ));
                        } else {
                            vc_render::render::report_boot_log("e2e: attack — no mob in reach");
                        }
                    }
                    _ => {}
                }
            }
        }

        // loading → wait for spawn chunk, then snap to surface → title screen
        // BLOCKING-BUG FIX (user report: fall-through-world): the snap now
        // keys on chunk DATA, not the GPU mesh. On slow devices meshing can
        // trail the 15 s Loading timeout — the old mesh-gated snap then
        // never ran, the game started with the player at spawn+20 in
        // mid-air, and physics over not-yet-generated chunks (get_block =
        // AIR) free-fell the player below y=0 into the void (observed live
        // at y = −2312 in the WebGL2 build). Block data is all the snap
        // needs; meshes catch up in view. The Game arm is defense in depth
        // for the timeout path that enters before meshing finishes.
        if (self.screen == Screen::Loading || self.screen == Screen::Game)
            && !self.spawn_snapped
        {
            self.try_snap_to_surface();
        }
        if self.screen == Screen::Loading {
            let pc = self.player_chunk();
            if self.renderer.has_chunk(pc) {
                let ready = {
                    let mut count = 0;
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            if self.renderer.has_chunk((pc.0 + dx, pc.1 + dz)) {
                                count += 1;
                            }
                        }
                    }
                    count >= 5 && self.mesh_near_count(pc) > 4
                };
                if ready || self.time - self.load_start > 15.0 {
                    // boot → title screen; §28 travel → straight back to play;
                    // Phase 1 world create/play/respawn → into the game
                    if self.traveling {
                        self.set_screen(Screen::Game);
                    } else if self.pending_play {
                        self.pending_play = false;
                        self.start_game();
                    } else {
                        self.set_screen(Screen::Title);
                    }
                    self.traveling = false;
                }
            }
        }

        let in_game = self.screen == Screen::Game;

        // player physics
        let t_sim = crate::bench::micros();
        if in_game {
            // BLOCKING-BUG FIX (fall-through-world, same report): hold the
            // player integration while their own chunk is not generated —
            // get_block() returns AIR over missing chunks, so gravity would
            // sink the player below y=0 where NO block can ever collide
            // again (the observed y = −2312 void fall). Vanilla freezes
            // entities in unloaded chunks (they do not tick); streaming and
            // meshing keep working around the frozen player meanwhile. This
            // also covers fast creative flight outrunning the generation
            // frontier.
            if !physics_frozen(&self.world, self.player.pos) {
                let sounds = self.player.update(
                    dt,
                    self.time,
                    &self.world,
                    &mut self.input,
                    self.settings.sensitivity,
                    true,
                );
                for s in sounds {
                    // footsteps + water-entry: the registry's step/splash events
                    // carry their own volume + pitch ranges (§21)
                    let ev = vc_audio::sounds::family_event(s.family, false);
                    self.play_event(ev, None, 1.0);
                }
            }

            // Phase 1: fall damage (MC-12357: fall − 3 HP) — creative is
            // invulnerable, the queued damage drains away instead
            let fall = self.player.take_pending_fall_damage();
            if fall > 0.0 {
                if self.mode.invulnerable() {
                    // creative: nothing happens (immunity includes falls)
                } else {
                    let applied = self.player.damage(fall);
                    if applied > 0.0 {
                        self.play_event("entity.player.hurt", None, 1.0);
                        self.death_cause = "FELL FROM A HIGH PLACE".into();
                        self.ui.dirty = true;
                    }
                }
            }

            // targeting
            self.target = raycast(&self.world, self.player.eye(), self.player.look_dir(), crate::player::REACH);

            // interactions
            self.break_timer -= dt;
            self.place_timer -= dt;
            if self.input.break_hold && self.break_timer <= 0.0 {
                if let Some((pos, b, _)) = self.target {
                    if b != BEDROCK {
                        let broke = self.world.get_block(pos[0], pos[1], pos[2]);
                        let (biome, sky, blk) = light_at(&self.world, &self.light, pos[0], pos[1], pos[2]);
                        if let Some((old, new)) = self.world.set_block(pos[0], pos[1], pos[2], AIR) {
                            self.light.on_block_changed(&self.world, pos[0], pos[1], pos[2], old, new);
                        }
                        // fences: removing a block changes neighbor connections
                        update_fence_neighbors(&mut self.world, pos[0], pos[1], pos[2]);
                        // §5 break burst: vanilla 4×4×4 particle grid, baked
                        // biome tint + light
                        self.particles.spawn_block_break(
                            pos[0], pos[1], pos[2], broke, biome, sky, blk,
                        );
                        // §22/§24: item drop + neighbor sim notification
                        // (water flows, sand falls). Phase 1: creative
                        // breaking yields NO drops (infinite inventory —
                        // blocks just vanish, vanilla behavior)
                        if self.mode.drops_blocks() {
                            self.sim.items.drop_block(
                                pos[0], pos[1], pos[2], broke, biome, sky, blk,
                            );
                        }
                        notify_sim(&self.world, &mut self.sim.sched, pos[0], pos[1], pos[2]);
                        // §27/§29: container contents spill + entity cleanup
                        self.drop_container_contents(pos, broke);
                        // §29: mining ores grants XP (vanilla amounts)
                        let ore_xp = vc_gameplay::enchanting::ore_xp(broke);
                        if ore_xp > 0 {
                            let gained = self.player.add_xp(ore_xp);
                            if gained > 0 {
                                self.play_event("entity.player.levelup", None, 1.0);
                            }
                        }
                        self.play_event(
                            vc_audio::sounds::family_event(def(b).sound, true),
                            Some([pos[0] as f32 + 0.5, pos[1] as f32 + 0.5, pos[2] as f32 + 0.5]),
                            1.0,
                        );
                        self.break_timer = 0.24;
                        self.edits += 1;
                    }
                }
            }
            if self.input.place_hold && self.place_timer <= 0.0 {
                // §27/§29: a villager under the crosshair opens the trade
                // screen FIRST (vanilla interaction priority over blocks)
                if let Some(vid) = self.sim.villagers.ray_hit(
                    self.player.eye().to_array(),
                    self.player.look_dir().to_array(),
                    crate::player::REACH,
                ) {
                    if let Some(v) = self.sim.villagers.by_id(vid) {
                        let pos = v.pos;
                        self.play_event(
                            "entity.villager.ambient",
                            Some([pos[0], pos[1] + 0.9, pos[2]]),
                            1.0,
                        );
                    }
                    self.open_container(Container::Trade { villager: vid });
                    self.place_timer = 0.3;
                } else if let Some((tpos, tb, _)) = self.target {
                    // §25: right-click a lever toggles it (vanilla interaction)
                    if tb == LEVER {
                        vc_sim::redstone::toggle_lever(
                            &mut self.world,
                            &mut self.sim.sched,
                            tpos[0],
                            tpos[1],
                            tpos[2],
                        );
                        self.play_event(
                            "block.lever.click",
                            Some([tpos[0] as f32 + 0.5, tpos[1] as f32 + 0.5, tpos[2] as f32 + 0.5]),
                            1.0,
                        );
                        self.place_timer = 0.24;
                    } else if tb == CRAFTING_TABLE {
                        // §27: right-click opens the 3×3 crafting screen
                        self.open_container(Container::Crafting { pos: tpos });
                        self.place_timer = 0.3;
                    } else if tb == FURNACE {
                        // §27: right-click opens the furnace screen; the
                        // block entity is created on first use (empty state)
                        self.sim.furnaces.map.entry(tpos).or_default();
                        self.open_container(Container::Furnace { pos: tpos });
                        self.place_timer = 0.3;
                    } else if tb == BREWING_STAND {
                        // §29: right-click opens the brewing screen; the
                        // block entity is created on first use (empty state)
                        self.sim.brewing.map.entry(tpos).or_default();
                        self.open_container(Container::Brewing { pos: tpos });
                        self.place_timer = 0.3;
                    } else if tb == ENCHANT_TABLE {
                        // §29: right-click opens the enchanting screen; the
                        // entity + offer list generate on first use
                        let seed = self.world.seed;
                        let e = self.sim.enchants.map.entry(tpos).or_default();
                        e.reroll(&self.world, tpos, seed);
                        self.open_container(Container::Enchant { pos: tpos });
                        self.place_timer = 0.3;
                    } else if tb == CHEST {
                        // Phase 3: right-click opens the chest screen; the
                        // 27-slot container entity is created on first use
                        // (empty state) — §26 containers
                        self.sim.containers.entry(tpos, CHEST);
                        self.open_container(Container::Chest { pos: tpos });
                        self.place_timer = 0.3;
                    } else if !self.player.held().is_empty()
                        && self.player.held().block == POTION_EMPTY
                        && (self.player.in_water || self.player.head_in_water)
                    {
                        // §29: right-click while submerged fills a glass
                        // bottle (vanilla bottle-on-water interaction; our
                        // raycast skips water, so the submersion check is
                        // the playable trigger)
                        let held = self.player.held_mut();
                        held.count -= 1;
                        let empty = held.count == 0;
                        if empty {
                            *held = vc_inventory::inventory::ItemStack::EMPTY;
                        }
                        let left = self.player.inv.add(POTION_WATER, 1);
                        if left > 0 {
                            self.sim.items.drop_block(
                                self.player.pos.x.floor() as i32,
                                self.player.pos.y.floor() as i32,
                                self.player.pos.z.floor() as i32,
                                POTION_WATER, 2, 15, 0,
                            );
                        }
                        self.play_event("liquid.splash", None, 0.8);
                        self.place_timer = 0.3;
                        self.ui.dirty = true;
                    } else if !self.player.held().is_empty()
                        && is_food(self.player.held().block)
                    {
                        // Phase 2: right-click eats raw meat. Documented
                        // deviation: no hunger system yet, so food heals
                        // directly (4 HP ≈ the meats' satiating weight);
                        // stacks deplete in Survival only
                        let b = self.player.held().block;
                        let heal_amt = 4.0;
                        if self.mode.depletes_items() {
                            let held = self.player.held_mut();
                            held.count -= 1;
                            if held.count == 0 {
                                *held = vc_inventory::inventory::ItemStack::EMPTY;
                            }
                        }
                        if !self.mode.invulnerable() {
                            self.player.heal(heal_amt);
                        }
                        self.play_event("entity.generic.drink", None, 0.8);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: ate {} (+{heal_amt} hp -> {})",
                            name(b),
                            self.player.health
                        ));
                        self.place_timer = 0.5;
                        self.ui.dirty = true;
                    } else if !self.player.held().is_empty()
                        && is_item_block(self.player.held().block)
                        && self.player.held().block != POTION_EMPTY
                    {
                        // §29: right-click drinks a potion — instant health
                        // heals; water/awkward/mundane do nothing (vanilla);
                        // the glass bottle comes back
                        let b = self.player.held().block;
                        let heal = vc_gameplay::brewing::potion_heal(b);
                        let held = self.player.held_mut();
                        held.count -= 1;
                        let empty = held.count == 0;
                        if empty {
                            *held = vc_inventory::inventory::ItemStack::EMPTY;
                        }
                        drop(held);
                        if let Some(h) = heal {
                            // Phase 1: potions never damage Creative (mode
                            // immunity); positive healing still applies
                            if h >= 0.0 || !self.mode.invulnerable() {
                                self.player.heal(h);
                                if h < 0.0 {
                                    self.death_cause = "DIED FROM MAGIC".into();
                                    self.play_event("entity.player.hurt", None, 1.0);
                                }
                            }
                            vc_render::render::report_boot_log(&format!(
                                "e2e: drank {} (+{h} hp → {})",
                                name(b),
                                self.player.health
                            ));
                        }
                        // vanilla: the empty glass bottle returns
                        let left = self.player.inv.add(POTION_EMPTY, 1);
                        if left > 0 {
                            self.sim.items.drop_block(
                                self.player.pos.x.floor() as i32,
                                self.player.pos.y.floor() as i32,
                                self.player.pos.z.floor() as i32,
                                POTION_EMPTY, 2, 15, 0,
                            );
                        }
                        self.play_event("entity.generic.drink", None, 0.9);
                        self.place_timer = 0.3;
                        self.ui.dirty = true;
                    } else if !self.player.held().is_empty() {
                        let b = self.player.held().block;
                        if is_item_block(b) {
                            // potions/bottles are never placeable (§29) —
                            // the drink/fill branches above catch the real
                            // interactions; empty bottle out of water = no-op
                        } else if let Some((_, _, prev)) = self.target {
                        let pb = self.world.get_block(prev[0], prev[1], prev[2]);
                        let replaceable = pb == AIR || pb == WATER || is_cross(pb);
                        let collides_player = is_solid(b) && self.player.block_intersects_player(prev);
                        if replaceable && !collides_player {
                            // vanilla placement rules per block family
                            let state = if is_log(b) {
                                // vanilla log placement: the axis follows the
                                // clicked face (top/bottom → axis Y, ±X → X, ±Z → Z)
                                let axis = if prev[1] != tpos[1] {
                                    1
                                } else if prev[0] != tpos[0] {
                                    0
                                } else {
                                    2
                                };
                                log_axis_state(b, axis)
                            } else if b == OAK_SLAB {
                                // vanilla slabs: clicking the TOP of a block →
                                // bottom slab; the UNDERSIDE → top slab
                                let half = if prev[1] < tpos[1] { "top" } else { "bottom" };
                                prop_state_encode(b, &[("half", half)]).unwrap_or(b as u16)
                            } else if b == COBBLE_STAIRS {
                                // vanilla stairs: face AWAY from the player
                                // (the ascent direction); half like slabs
                                let yaw = ((self.player.yaw.to_degrees() % 360.0) + 360.0) % 360.0;
                                let facing = match yaw {
                                    315.0..=360.0 | 0.0..=45.0 => "south",
                                    45.0..=135.0 => "west",
                                    135.0..=225.0 => "north",
                                    _ => "east",
                                };
                                let half = if prev[1] < tpos[1] { "top" } else { "bottom" };
                                prop_state_encode(
                                    b,
                                    &[("facing", facing), ("half", half)],
                                )
                                .unwrap_or(b as u16)
                            } else if b == OAK_FENCE {
                                // connections computed from the current world
                                fence_state_for(&self.world, prev[0], prev[1], prev[2])
                                    .unwrap_or(b as u16)
                            } else {
                                // sim blocks (wire/furnace/…) get their proper
                                // default STATE — never the identity slot
                                default_state(b)
                            };
                            if let Some((old, new)) =
                                self.world.set_block_state(prev[0], prev[1], prev[2], state)
                            {
                                self.light.on_block_changed(
                                    &self.world, prev[0], prev[1], prev[2], old, new,
                                );
                            }
                            // fences: neighbors recompute their connections
                            update_fence_neighbors(&mut self.world, prev[0], prev[1], prev[2]);
                            // §24/§25: a new block notifies the sim
                            notify_sim(&self.world, &mut self.sim.sched, prev[0], prev[1], prev[2]);
                            self.play_event(
                                vc_audio::sounds::family_event(def(b).sound, true),
                                Some([
                                    prev[0] as f32 + 0.5,
                                    prev[1] as f32 + 0.5,
                                    prev[2] as f32 + 0.5,
                                ]),
                                1.0,
                            );
                            // Phase 1: placement depletes the stack in
                            // Survival/Hardcore only — creative stacks are
                            // infinite (vanilla behavior)
                            if self.mode.depletes_items() {
                                let held = self.player.held_mut();
                                held.count -= 1;
                                if held.count == 0 {
                                    *held = vc_inventory::inventory::ItemStack::EMPTY;
                                }
                            }
                            self.place_timer = 0.24;
                            self.edits += 1;
                        }
                        }
                    }
                }
            }

            // Phase 1: death gate, END of the gameplay tick — fall damage
            // and potion damage have applied by now. Runs after the
            // interactions (not around them) so a death this tick still
            // reaches the UI-cadence code below: the death screen needs the
            // rebuild to draw at all.
            self.check_death();
        }
        self.phases.add(crate::bench::PHASE_SIM, crate::bench::micros() - t_sim);

        // toasts
        if let Some((_, t)) = self.item_toast.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.item_toast = None;
                self.ui.dirty = true;
            }
        }

        // animated pack textures: atlas region updates only (§20 — no
        // geometry rebuilds when a texture frame changes)
        if !self.animations.is_empty() {
            let updates = vc_render::textures::tick_animations(&mut self.animations, dt);
            for (tile, frame) in updates {
                if let Some(a) = self
                    .animations
                    .iter()
                    .find(|a| a.tile == tile)
                {
                    self.renderer.update_atlas_frame(a, frame as usize);
                }
            }
        }

        // UI rebuild cadence: snappier in menus (hover) + picker + live F3
        // + open containers (furnace progress arrows animate at 20 Hz —
        // 0.05 s cadence shows them smoothly)
        let live_debug = self.screen == Screen::Game && self.show_debug;
        let container_live = self.container.is_some();
        let cadence = if self.screen == Screen::Game
            && !self.picker_open
            && !live_debug
            && !container_live
        {
            0.15
        } else {
            0.05
        };
        // containers animate: mark the UI dirty on a 5 Hz heartbeat while a
        // furnace screen is open (craft/inventory screens are static between
        // clicks — clicks already set dirty)
        if container_live && matches!(self.container, Some(Container::Furnace { .. })) {
            if self.time - self.last_ui_t > 0.2 {
                self.ui.dirty = true;
            }
        }
        if self.ui.dirty && self.time - self.last_ui_t > cadence {
            crate::phase!(self.phases, crate::bench::PHASE_UI, self.rebuild_ui());
        }

        // publish debug stats for E2E tests (wasm)
        #[cfg(target_arch = "wasm32")]
        {
            if self.time - self.stats_t > 0.25 {
                self.stats_t = self.time;
                self.publish_stats();
            }
        }

        // native autosave (§28): 20 s cadence while a world is in play.
        // Benchmarks never touch the save dir.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let in_world = self.screen == Screen::Game || self.screen == Screen::Pause;
            if in_world && self.bench.is_none() {
                self.autosave_in -= dt;
                if self.autosave_in <= 0.0 {
                    self.autosave_in = 20.0;
                    self.save_world();
                }
            }
        }
    }

    /// Flush all unsaved chunks + level.dat to the save dir (native, §28).
    /// One compact-and-rewrite per touched region file; the player state
    /// rides in level.dat (vanilla keys + a `voxelcraft` sub-compound).
    #[cfg(not(target_arch = "wasm32"))]
    fn save_world(&mut self) {
        let dirty: Vec<ChunkPos> = self.world.save_dirty.drain().collect();
        let entries: Vec<(ChunkPos, Arc<vc_chunk::chunk::Chunk>)> = dirty
            .into_iter()
            .filter_map(|p| self.world.chunks.get(&p).map(|c| (p, Arc::clone(c))))
            .collect();
        let tick = ((self.time - self.load_start) * 20.0).max(0.0) as i64;
        if !entries.is_empty() {
            let refs: Vec<(i32, i32, &vc_chunk::chunk::Chunk, Option<&Arc<vc_world::light::LightData>>)> =
                entries
                    .iter()
                    .map(|(p, c)| (p.0, p.1, c.as_ref(), self.world.light.get(p)))
                    .collect();
            if let Err(e) = vc_anvil::save::store_chunks(&self.world_dir, &refs, tick) {
                vc_render::render::report_boot_log(&format!("autosave failed: {e}"));
            }
        }
        let meta = vc_anvil::save::WorldMeta {
            seed: self.world.seed,
            name: self.world_name.clone(),
            spawn: self.level_spawn,
            player: Some(vc_anvil::save::PlayerMeta {
                pos: [self.player.pos.x as f64, self.player.pos.y as f64, self.player.pos.z as f64],
                yaw: self.player.yaw,
                pitch: self.player.pitch,
            }),
            game_time: tick,
            // Phase 1: the real mode + hardcore state (vanilla schema)
            game_type: self.mode.vanilla_game_type(),
            hardcore: self.mode.vanilla_hardcore(),
            hardcore_dead: self.hardcore_dead,
            // Phase 5: container inventories (dungeon loot + touched
            // chests/hoppers) — they restore on load via read_level_dat
            containers: self
                .sim
                .containers
                .map
                .iter()
                .map(|(pos, inv)| {
                    let slots = inv
                        .slots
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| !s.is_empty())
                        .map(|(i, s)| (i as u16, s.block, s.count))
                        .collect();
                    // kind: live block when the chunk is loaded; otherwise
                    // inferred from the slot count (27 chest / 5 hopper /
                    // 9 dispenser-dropper) — containers in unloaded chunks
                    // keep their inventory shape
                    let live = vc_blocks::blocks::state_block(
                        self.world.get_state(pos[0], pos[1], pos[2]),
                    );
                    let kind = match live {
                        CHEST | DISPENSER | DROPPER | HOPPER => live,
                        _ => match inv.slots.len() {
                            27 => CHEST,
                            5 => HOPPER,
                            _ => DISPENSER,
                        },
                    };
                    vc_anvil::save::ContainerMeta {
                        pos: *pos,
                        kind,
                        slots,
                    }
                })
                .collect(),
        };
        if let Err(e) = vc_anvil::save::write_level_dat(&self.world_dir, &meta) {
            vc_render::render::report_boot_log(&format!("level.dat write failed: {e}"));
        }
    }

    /// §28 dimension travel (P7): swap the entire world for a fresh one in
    /// the target dimension — same seed, dimension-salted generator — and
    /// reset every dimension-local system:
    ///
    /// * player position follows the vanilla 8:1 coordinate rule
    ///   (overworld → nether divides by 8; nether → overworld multiplies);
    ///   the exact landing spot is refined when the spawn chunk arrives
    ///   (the Loading snap), like vanilla's portal search
    /// * GPU meshes, section-mesh caches, generation/mesh queues, light
    ///   engine, sim (block entities never cross dimensions), particles,
    ///   open containers are all reset
    /// * the inventory travels with the player (vanilla behavior)
    /// * native: the outgoing dimension's dirty chunks flush to its own
    ///   save dir first (overworld = world root, nether = DIM-1)
    pub fn travel_to_dimension(&mut self, dim: vc_world::world::Dimension) {
        if dim == self.world.dimension {
            return;
        }
        // native: flush the outgoing dimension before swapping the dir
        #[cfg(not(target_arch = "wasm32"))]
        if self.bench.is_none() {
            self.save_world();
        }

        // 8:1 horizontal mapping (vanilla nether portals)
        let cur = self.world.dimension;
        let (nx, nz) = cur.map_coords(dim, self.player.pos.x.floor() as i32, self.player.pos.z.floor() as i32);

        // fresh world in the target dimension
        self.world = World::new_in_dimension(self.world.seed, dim);
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.world_dir = vc_anvil::save::dimension_dir(&self.save_root, dim);
        }

        // reset every dimension-local system
        self.renderer.clear_meshes();
        self.section_meshes.clear();
        self.mesh_inflight.clear();
        self.gen_inflight.clear();
        self.light = vc_world::light::LightEngine::new();
        self.sim = vc_sim::sim::Sim::new(self.world.seed ^ dim.seed_salt());
        self.particles = vc_particles::particles::ParticleSystem::new(self.world.seed ^ 0x7EED);
        self.particle_verts.clear();
        self.container = None;
        self.container_geom = None;
        self.cursor_stack = vc_inventory::inventory::ItemStack::EMPTY;
        self.craft_grid = [vc_inventory::inventory::ItemStack::EMPTY; 9];
        self.target = None;
        self.break_timer = 0.0;
        self.place_timer = 0.0;

        // player: inventory persists, position rescales; y waits for the snap
        let y = if dim == vc_world::world::Dimension::Nether { 90.0 } else { 120.0 };
        self.player.pos = Vec3::new(nx as f32 + 0.5, y, nz as f32 + 0.5);
        self.player.vel = Vec3::ZERO;
        self.player.flying = false;

        // wait for the spawn chunk through the Loading screen (vanilla shows
        // a loading screen on travel too), then return to the game
        self.traveling = true;
        self.spawn_snapped = false;
        self.load_start = self.time;
        self.set_screen(Screen::Loading);
        vc_render::render::report_boot_log(&format!(
            "dimension travel: {} -> {} (coords {},{})",
            cur.id(),
            dim.id(),
            nx,
            nz
        ));
    }

    /// §28: nether floor search for the travel snap — a cavern cell with a
    /// solid floor and 2 blocks of headroom, nearest to the target height.
    /// (top_solid_y is wrong in the nether: the bedrock ROOF is the top.)
    fn nether_floor_y(&self, chunk: &vc_chunk::chunk::Chunk, lx: usize, lz: usize) -> Option<i32> {
        use vc_blocks::blocks::{is_solid, state_block};
        let target = self.player.pos.y;
        let mut best: Option<i32> = None;
        let mut best_dist = f32::MAX;
        for y in 6..120usize {
            let feet = state_block(chunk.get(lx, y, lz) as u16);
            let head = state_block(chunk.get(lx, y + 1, lz) as u16);
            let floor = state_block(chunk.get(lx, y - 1, lz) as u16);
            if !is_solid(floor) || is_solid(feet) || is_solid(head) {
                continue;
            }
            let d = (y as f32 - target).abs();
            if d < best_dist {
                best_dist = d;
                best = Some(y as i32);
            }
        }
        best
    }

    #[cfg(target_arch = "wasm32")]
    fn publish_stats(&self) {
        use crate::web_input::{publish_stats, StatsVal};
        publish_stats(&[
            ("screen", StatsVal::S(self.screen.name().to_string())),
            ("loading", StatsVal::B(self.screen == Screen::Loading)),
            ("locked", StatsVal::B(self.pointer_locked)),
            ("dragLook", StatsVal::B(self.drag_look)),
            ("x", StatsVal::F(self.player.pos.x)),
            ("y", StatsVal::F(self.player.pos.y)),
            ("z", StatsVal::F(self.player.pos.z)),
            ("yaw", StatsVal::F(self.player.yaw)),
            ("pitch", StatsVal::F(self.player.pitch)),
            ("fps", StatsVal::F(self.fps)),
            ("chunksLoaded", StatsVal::F(self.world.chunks.len() as f32)),
            ("chunksDrawn", StatsVal::F(self.stats.chunks as f32)),
            ("tris", StatsVal::F(self.stats.tris as f32)),
            ("drawCalls", StatsVal::F(self.stats.draws as f32)),
            ("bufferBinds", StatsVal::F(self.stats.binds as f32)),
            ("drawPath", StatsVal::S(self.renderer.draw_path_name().into())),
            // §28: current dimension (0 = overworld, 1 = nether) + name
            ("dim", StatsVal::F(self.world.dimension as u8 as f32)),
            ("dimName", StatsVal::S(self.world.dimension.id().into())),
            ("traveling", StatsVal::B(self.traveling)),
            ("shaderMode", StatsVal::F(self.settings.shader as f32)),
            (
                "shaderPack",
                StatsVal::S(self.renderer.pack_id.clone().unwrap_or_default()),
            ),
            ("packTier", StatsVal::S(self.renderer.pack_tier.clone())),
            // §12 evidence: section-granular invalidation state
            ("dirtySections", StatsVal::F(self.world.dirty_section_count() as f32)),
            ("dirtyChunks", StatsVal::F(self.world.dirty.len() as f32)),
            (
                "dirtyCauses",
                StatsVal::F(
                    self.world.dirty_causes.values().fold(0u8, |a, b| a | b) as f32
                ),
            ),
            ("sectionCache", StatsVal::F(
                self.section_meshes
                    .values()
                    .map(|v| v.iter().filter(|s| s.is_some()).count())
                    .sum::<usize>() as f32,
            )),
            ("rd", StatsVal::F(self.settings.render_distance as f32)),
            // Phase 6 §26: rendering-quality settings + occlusion counters
            ("sd", StatsVal::F(self.settings.sim_distance as f32)),
            ("mip", StatsVal::F(self.settings.mipmap_levels as f32)),
            ("aniso", StatsVal::F(self.settings.aniso as f32)),
            ("msaa", StatsVal::F(self.renderer.msaa() as f32)),
            ("msaaMax", StatsVal::F(self.renderer.msaa_supported() as f32)),
            ("occl", StatsVal::B(self.settings.occlusion)),
            ("culled", StatsVal::F(self.stats.culled as f32)),
            // Phase 7: GPU meshing backend + throughput counters
            ("gmesh", StatsVal::B(
                self.settings.gpu_meshing && self.renderer.gpu_mesh.is_some()
            )),
            ("gmeshAvail", StatsVal::B(self.renderer.gpu_mesh.is_some())),
            ("gmeshDone", StatsVal::F(
                self.renderer.gpu_mesh.as_ref().map(|m| m.jobs_done as f32).unwrap_or(0.0)
            )),
            ("gmeshQueue", StatsVal::F(
                self.renderer.gpu_mesh.as_ref().map(|m| m.queued() as f32).unwrap_or(0.0)
            )),
            // Phase 8: Iris interface — detected packs (native scan; the
            // wasm build boots empty by design, no filesystem)
            ("irisPacks", StatsVal::F(self.iris_packs.len() as f32)),
            // Phase 9: data packs — counts for E2E assertions
            ("dpacks", StatsVal::F(self.data.packs.len() as f32)),
            ("drecipes", StatsVal::F(self.data.recipes.len() as f32)),
            ("dloots", StatsVal::F(self.data.loot_tables.len() as f32)),
            ("dtags", StatsVal::F(self.data.tags.len() as f32)),
            ("fov", StatsVal::F(self.settings.fov)),
            ("sens", StatsVal::F(self.settings.sensitivity)),
            ("vol", StatsVal::F(self.settings.volume)),
            ("bright", StatsVal::F(self.settings.brightness)),
            ("shader", StatsVal::F(self.settings.shader as f32)),
            ("clouds", StatsVal::B(self.settings.clouds)),
            ("smooth", StatsVal::B(self.settings.smooth_lighting)),
            ("fancy", StatsVal::B(self.settings.graphics >= 1)),
            ("graphics", StatsVal::F(self.settings.graphics as f32)),
            ("shadowQuality", StatsVal::F(self.settings.shadow_quality as f32)),
            ("shadowMap", StatsVal::F(self.renderer.shadow_px as f32)),
            ("upscale", StatsVal::F(self.settings.upscale_factor())),
            ("edits", StatsVal::F(self.edits as f32)),
            ("particles", StatsVal::F(self.particles.len() as f32)),
            ("particlesDrawn", StatsVal::F(self.stats.particles as f32)),
            ("particlesTotal", StatsVal::F(self.particles.spawned_total as f32)),
            ("simTicks", StatsVal::F(self.sim.ticks as f32)),
            ("schedPending", StatsVal::F(self.sim.sched.pending() as f32)),
            ("items", StatsVal::F(self.sim.items.len() as f32)),
            ("itemsDropped", StatsVal::F(self.sim.items.dropped_total as f32)),
            ("itemsPicked", StatsVal::F(self.sim.items.picked_total as f32)),
            ("sounds", StatsVal::F(self.sounds_played as f32)),
            // §29: real player health + brewing counters
            ("health", StatsVal::F(self.player.health)),
            // Phase 1: mode + world identity for E2E assertions
            ("mode", StatsVal::S(self.mode.label().into())),
            ("modeIdx", StatsVal::F(self.mode.index() as f32)),
            ("worldName", StatsVal::S(self.world_name.clone())),
            ("hardcoreDead", StatsVal::B(self.hardcore_dead)),
            ("furnaces", StatsVal::F(self.sim.furnaces.map.len() as f32)),
            ("brewStands", StatsVal::F(self.sim.brewing.map.len() as f32)),
            ("potionsBrewed", StatsVal::F(self.sim.brewing.total_brewed as f32)),
            // §29: XP + enchanting + villagers
            ("xpLevel", StatsVal::F(self.player.xp_level as f32)),
            ("xpPoints", StatsVal::F(self.player.xp_points as f32)),
            ("enchApplied", StatsVal::F(self.sim.enchants.total_enchanted as f32)),
            ("villagers", StatsVal::F(self.sim.villagers.list.len() as f32)),
            ("tradesDone", StatsVal::F(self.sim.villagers.trades_done as f32)),
            // Phase 2: mobs + combat
            ("mobs", StatsVal::F(self.sim.mobs.len() as f32)),
            ("mobsSpawned", StatsVal::F(self.sim.mobs.spawned_total as f32)),
            ("mobsKilled", StatsVal::F(self.sim.mobs.killed_total as f32)),
            ("arrows", StatsVal::F(self.sim.mobs.arrows.len() as f32)),
            ("swingT", StatsVal::F(self.swing_t)),
            ("fwd", StatsVal::B(self.input.fwd)),
            ("back", StatsVal::B(self.input.back)),
            ("left", StatsVal::B(self.input.left)),
            ("right", StatsVal::B(self.input.right)),
            ("jump", StatsVal::B(self.input.jump)),
            ("breakHold", StatsVal::B(self.input.break_hold)),
            ("placeHold", StatsVal::B(self.input.place_hold)),
            ("hasTarget", StatsVal::B(self.target.is_some())),
            ("targetState", StatsVal::F(
                self.target.map(|(t, _, _)| self.world.get_state(t[0], t[1], t[2]) as f32).unwrap_or(-1.0)
            )),
            ("targetDesc", StatsVal::S(
                self.target
                    .map(|(t, _, _)| state_description(self.world.get_state(t[0], t[1], t[2])))
                    .unwrap_or_default()
            )),
            ("modelStates", StatsVal::F(
                vc_pack::model::models().map(|m| m.by_state.len() as f32).unwrap_or(0.0)
            )),
            ("packTextures", StatsVal::F(
                vc_pack::model::models().map(|m| m.tiles.len() as f32).unwrap_or(0.0)
            )),
            ("animations", StatsVal::F(self.animations.len() as f32)),
            ("breakTimer", StatsVal::F(self.break_timer)),
            ("hover", StatsVal::F(self.hover.map(|h| h as f32).unwrap_or(-1.0))),
            ("picker", StatsVal::B(self.picker_open)),
            ("frameMs", StatsVal::F(self.frame_ms)),
            ("histLen", StatsVal::F(self.frame_times.len() as f32)),
            ("dragging", StatsVal::F(self.dragging.map(|d| d as f32).unwrap_or(-1.0))),
        ]);
    }

    fn time_since_load(&self) -> f32 {
        self.time - self.load_start
    }

    fn mesh_near_count(&self, pc: ChunkPos) -> u32 {
        let mut n = 0;
        for dz in -2..=2 {
            for dx in -2..=2 {
                if self.renderer.has_chunk((pc.0 + dx, pc.1 + dz)) {
                    n += 1;
                }
            }
        }
        n
    }

    /// Place the player on the first solid floor of their column the moment
    /// the column's chunk DATA exists — mesh-independent (see the
    /// fall-through-world fix note at the Loading handler). Runs from both
    /// the Loading pipeline and (as a safety net) the first Game frames if
    /// the Loading timeout entered the game before meshing finished.
    fn try_snap_to_surface(&mut self) {
        let pc = self.player_chunk();
        let lx = (self.player.pos.x - pc.0 as f32 * 16.0).floor().clamp(0.0, 15.0) as usize;
        let lz = (self.player.pos.z - pc.1 as f32 * 16.0).floor().clamp(0.0, 15.0) as usize;
        let Some(c) = self.world.chunk(pc) else {
            return; // data not generated yet — retry next frame
        };
        // §28: the snap depends on the dimension — the overworld snaps to
        // the topmost solid block; the nether needs a CAVERN floor
        // (top_solid_y there is the bedrock roof). Travel keeps flying on
        // until a spot exists so the player never spawns inside rock.
        let snap = if self.world.dimension == vc_world::world::Dimension::Nether {
            self.nether_floor_y(c, lx.min(15), lz.min(15))
        } else {
            let t = c.top_solid_y(lx.min(15), lz.min(15));
            if t >= 0 { Some(t + 1) } else { None }
        };
        if let Some(y) = snap {
            self.player.pos.y = y as f32;
            // Phase 1: survival lands on its feet (snap = no fall damage,
            // fall accumulator resets); creative only "arrives" flying if
            // it arrived flying
            self.player.flying = false;
            self.player.reset_fall();
        } else if self.traveling {
            // no open floor in this column — a creative player arrives
            // flying and glides to a cavern; a survival player stays put
            // and waits for the timeout path (documented deviation: vanilla
            // does a full portal search; our travel is the debug/API path)
            self.player.flying = self.mode.allows_flight();
        }
        self.spawn_snapped = true;
    }

    fn player_chunk(&self) -> ChunkPos {
        (
            self.player.pos.x.div_euclid(16.0) as i32,
            self.player.pos.z.div_euclid(16.0) as i32,
        )
    }

    // ---------------------------------------------------------- streaming --

    fn stream(&mut self) {
        let pc = self.player_chunk();
        let rd = self.settings.render_distance;

        // 1. collect + apply results (collect first to release the borrow)
        let t_results = crate::bench::micros();
        let mut results: Vec<JobResult> = Vec::new();
        let mut done = 0usize;
        match &mut self.work {
            WorkBackend::Threading { rx, inflight, .. } => {
                while let Ok(res) = rx.try_recv() {
                    results.push(res);
                    done += 1;
                }
                *inflight = inflight.saturating_sub(done);
            }
            WorkBackend::Inline { jobs } => {
                let budget = 0.006; // 6ms per frame keeps 60fps on the browser build
                let start = now_secs();
                while let Some(job) = jobs.pop_front() {
                    results.push(run_job(job));
                    if now_secs() - start > budget {
                        break;
                    }
                }
            }
        }
        // Phase 7: drive the GPU compute mesher — completions become
        // ordinary Mesh results; pendings go back to the mesher; lost jobs
        // (readback failure) release their inflight markers so the §12
        // dirty bits (still set) trigger a CPU remesh
        {
            let renderer = &mut self.renderer;
            if let Some(m) = renderer.gpu_mesh.as_mut() {
                let (gpu_done, lost) = m.advance(&renderer.device, &renderer.queue);
                for d in gpu_done {
                    let occl = chunk_occl(d.center.as_ref(), &d.sections);
                    results.push(JobResult::Mesh {
                        pos: d.pos,
                        mask: d.mask,
                        sections: d.sections,
                        mesh: Box::new(d.mesh),
                        occl,
                    });
                }
                for (pos, _) in lost {
                    self.mesh_inflight.remove(&pos);
                }
                // route pendings collected this frame into the next batch
                let mut pendings: Vec<JobResult> = Vec::new();
                for res in results.drain(..) {
                    match res {
                        JobResult::GpuMeshPending { pos, mask, smooth, prev, center, inputs } => {
                            m.enqueue(
                                vc_render::gpu_mesh::GpuMeshJobMeta {
                                    pos,
                                    mask,
                                    smooth,
                                    prev,
                                    center,
                                },
                                inputs,
                            );
                        }
                        other => pendings.push(other),
                    }
                }
                results = pendings;
            } else {
                // no mesher (WebGL2): pendings can't occur (gpu flag is
                // false at submit) — drain nothing
            }
        }
        for res in results {
            self.apply_result(res);
        }
        self.phases
            .add(crate::bench::PHASE_RESULTS, crate::bench::micros() - t_results);

        // 2. queue generation jobs (radius rd+1, nearest first)
        let mut want_gen: Vec<ChunkPos> = Vec::new();
        for dz in -(rd + 1)..=(rd + 1) {
            for dx in -(rd + 1)..=(rd + 1) {
                let pos = (pc.0 + dx, pc.1 + dz);
                if !self.world.chunks.contains_key(&pos) && !self.gen_inflight.contains(&pos) {
                    want_gen.push(pos);
                }
            }
        }
        want_gen.sort_by_key(|p| (p.0 - pc.0).abs() + (p.1 - pc.1).abs());
        let max_gen = if cfg!(target_arch = "wasm32") { 4 } else { 16 };
        for pos in want_gen.into_iter().take(max_gen) {
            // native: try the save dir first (§28) — a stored chunk skips
            // generation entirely; pending edits queued while the chunk was
            // absent replay on top. Sync disk read bounded by max_gen/frame.
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(Some((mut chunk, light))) =
                vc_anvil::save::load_chunk(&self.world_dir, pos.0, pos.1)
            {
                let inbound = self.world.take_pending(pos);
                let edited = !inbound.is_empty();
                for (idx, id) in inbound {
                    chunk.set_idx(idx as usize, id);
                }
                let chunk = Arc::new(chunk);
                self.world.insert_generated(pos, chunk.clone(), Vec::new());
                // Phase 5 §27: re-register the spawner block entities a
                // save carries (chunk data keeps only states), and re-seed
                // the villagers of any village whose chunks arrived from
                // disk (vanilla persists villager NBT; ours re-populate
                // at the well — the populated set keeps it once/session)
                self.register_block_entities(pos, &chunk, false);
                self.sim.villagers.populate_villages(&self.world, pos.0, pos.1);
                match light {
                    Some(ld) => {
                        self.world.light.insert(pos, Arc::new(ld));
                    }
                    None => {
                        // pre-Phase-4 save: re-light on load
                        self.light.init_chunk(&mut self.world, pos);
                        for (lpos, lmask) in self.light.take_changed() {
                            self.world.mark_sections_dirty(
                                lpos,
                                lmask,
                                vc_world::world::CAUSE_LIGHT,
                            );
                        }
                    }
                }
                if !edited {
                    // pristine content straight from disk — no need to
                    // rewrite it at the next autosave
                    self.world.save_dirty.remove(&pos);
                }
                continue;
            }
            let inbound = self.world.take_pending(pos);
            self.gen_inflight.insert(pos);
            let job = Job::Gen { pos, seed: self.world.seed, dim: self.world.dimension, inbound };
            self.submit(job);
        }

        // 3. queue mesh jobs (radius rd, nearest first, dirty first).
        // §12: dirty bits are SECTION masks — a job rebuilds only the stale
        // sections, reusing the cached meshes for the rest.
        let mut want_mesh: Vec<(ChunkPos, bool, u16)> = Vec::new();
        for dz in -rd..=rd {
            for dx in -rd..=rd {
                let pos = (pc.0 + dx, pc.1 + dz);
                if self.mesh_inflight.contains_key(&pos) {
                    continue;
                }
                let dirty_mask = self.world.dirty.get(&pos).copied().unwrap_or(0);
                let meshed = self.renderer.has_chunk(pos);
                let mask = if !meshed {
                    u16::MAX // first mesh of the chunk: all 16 sections
                } else {
                    dirty_mask
                };
                if mask != 0 && self.world.meshable(pos.0, pos.1) {
                    want_mesh.push((pos, dirty_mask != 0, mask));
                }
            }
        }
        want_mesh.sort_by(|a, b| {
            let da = (a.0 .0 - pc.0).abs() + (a.0 .1 - pc.1).abs();
            let db = (b.0 .0 - pc.0).abs() + (b.0 .1 - pc.1).abs();
            b.1.cmp(&a.1).then(da.cmp(&db)) // dirty chunks first
        });
        // wasm: 4 mesh jobs/frame (was 2 — the initial world fill crawled on
        // capable browsers; the 6 ms inline budget below is the REAL frame
        // guard: the loop always breaks after the first job that crosses it,
        // so slow devices are unaffected by the higher cap). Native keeps
        // the rayon pool cap.
        let max_mesh = if cfg!(target_arch = "wasm32") { 4 } else { 16 };
        for (pos, _, mask) in want_mesh.into_iter().take(max_mesh) {
            if let Some(snap) = self.world.snapshot3x3(pos.0, pos.1) {
                let lsnap = self
                    .world
                    .snapshot3x3_light(pos.0, pos.1)
                    .unwrap_or_default();
                let prev = self
                    .section_meshes
                    .get(&pos)
                    .cloned()
                    .unwrap_or_else(|| vec![None; 16]);
                self.mesh_inflight.insert(pos, mask);
                // Phase 7: GPU route when the setting is on AND the device
                // has compute; run_job still falls back per-snapshot
                let gpu = self.settings.gpu_meshing
                    && self.renderer.gpu_mesh.is_some();
                self.submit(Job::Mesh {
                    pos,
                    snap,
                    lsnap,
                    smooth: self.settings.smooth_lighting,
                    mask,
                    prev,
                    gpu,
                });
            }
        }

        // 4. unload far GPU meshes + their section caches
        let unload: Vec<ChunkPos> = self
            .renderer
            .chunks
            .keys()
            .filter(|p| {
                let d = (p.0 - pc.0).abs().max(p.1 - pc.1);
                d > rd + 3
            })
            .copied()
            .collect();
        for pos in unload {
            self.renderer.remove_chunk(pos);
            self.section_meshes.remove(&pos);
            self.world.dirty.remove(&pos);
            self.world.dirty_causes.remove(&pos);
        }
    }

    fn submit(&mut self, job: Job) {
        match &mut self.work {
            WorkBackend::Threading { tx, inflight, .. } => {
                let tx = tx.clone();
                *inflight += 1;
                #[cfg(not(target_arch = "wasm32"))]
                rayon::spawn(move || {
                    let res = run_job(job);
                    let _ = tx.send(res);
                });
                #[cfg(target_arch = "wasm32")]
                let _ = (tx, job);
            }
            WorkBackend::Inline { jobs } => {
                jobs.push_back(job);
            }
        }
    }

    /// Phase 5 §27: scan a newly-arrived chunk for block-entity blocks —
    /// spawners register into the sim (mob type decoded from the state),
    /// and on fresh generation dungeon chests get their loot roll. Called
    /// from BOTH chunk-arrival paths (generated + loaded-from-disk);
    /// `fill_loot` is true only for fresh generation (loaded inventories
    /// restore from level.dat instead — double-filling would dupe items).
    fn register_block_entities(&mut self, pos: ChunkPos, chunk: &Arc<vc_chunk::chunk::Chunk>, fill_loot: bool) {
        let ox = pos.0 * 16;
        let oz = pos.1 * 16;
        // Phase 10: which structure's loot table owns this chunk's fresh
        // chests — per-chunk primary-structure attribution (dungeon exact;
        // the others region queries; documented approximation: a chest in
        // a chunk claimed by two structures follows the higher priority)
        let table = if fill_loot {
            Some(self.chest_table_for(pos))
        } else {
            None
        };
        for y in 0..256usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    // Chunk::get returns the raw STATE id (as u8), and the
                    // dedicated-state blocks this scan hunts for never
                    // equal their block ids raw (CHEST_STATE 227 vs block
                    // 96, SPAWNER states 232..=234 vs block 101) — so the
                    // comparison must decode through state_block, the same
                    // mapping World::get_block applies. [Phase 5 carried a
                    // latent miss here: chunk-arrival spawners/chests never
                    // actually registered, because the fast-skip compared
                    // raw states against block ids. Fixed with the Phase 10
                    // loot seam that builds on this scan.]
                    let b = state_block(chunk.get(x, y, z) as u16);
                    if b != SPAWNER && b != CHEST {
                        continue; // fast skip — `get` on empty sections is cheap
                    }
                    let p = [ox + x as i32, y as i32, oz + z as i32];
                    if b == SPAWNER {
                        let s = chunk
                            .sections[y >> 4]
                            .as_ref()
                            .map(|sec| sec.get(x, y & 15, z))
                            .unwrap_or(0);
                        self.sim.spawners.register(p, vc_blocks::blocks::spawner_mob(s));
                    } else if fill_loot {
                        // structure loot chest: fill ONLY if the container
                        // is untouched (fresh generation creates it here; a
                        // re-arriving chunk with loot already inside never
                        // refills — the all-empty guard)
                        let inv = self.sim.containers.entry(p, CHEST);
                        if inv.slots.iter().all(|s| s.is_empty()) {
                            fill_structure_chest(&self.data, table.unwrap_or("minecraft:chests/simple_dungeon"), inv, self.world.seed, p);
                        }
                    }
                }
            }
        }
    }

    /// Phase 10: the loot-table seam a fresh chest in chunk (cx, cz)
    /// rolls from — structure-attribution priority dungeon > mineshaft >
    /// desert pyramid > jungle temple > stronghold (the per-chunk
    /// primary-structure approximation, documented). Stronghold chests
    /// split by position: the library chest sits north of the portal
    /// room's center, the store-room chest south of it.
    fn chest_table_for(&self, pos: ChunkPos) -> &'static str {
        let gen = &self.world.gen;
        let cx = pos.0;
        let cz = pos.1;
        if gen.dungeon_in_chunk(cx, cz).is_some() {
            return "minecraft:chests/simple_dungeon";
        }
        if !gen.mineshafts_near(cx * 16 + 8, cz * 16 + 8).is_empty() {
            return "minecraft:chests/abandoned_mineshaft";
        }
        if !gen.pyramids_near(cx * 16 + 8, cz * 16 + 8).is_empty() {
            return "minecraft:chests/desert_pyramid";
        }
        if !gen.jungle_temples_near(cx * 16 + 8, cz * 16 + 8).is_empty() {
            return "minecraft:chests/jungle_temple";
        }
        for &(sx, sz) in gen.strongholds().iter() {
            if (sx - (cx * 16 + 8)).abs() <= 40 && (sz - (cz * 16 + 8)).abs() <= 40 {
                return if sz - (cz * 16 + 8) > 0 {
                    // chest north of the stronghold center → library
                    "minecraft:chests/stronghold_library"
                } else {
                    "minecraft:chests/stronghold_corridor"
                };
            }
        }
        "minecraft:chests/simple_dungeon"
    }

    fn apply_result(&mut self, res: JobResult) {
        match res {
            // pendings are intercepted in stream() before apply_result —
            // reaching here would double-route a GPU job; treat as a bug
            JobResult::GpuMeshPending { pos, .. } => {
                debug_assert!(false, "GpuMeshPending reached apply_result ({pos:?})");
                self.mesh_inflight.remove(&pos);
            }
            JobResult::Gen { pos, chunk, outbound } => {
                self.gen_inflight.remove(&pos);
                let leftover = self.world.pending.remove(&pos).unwrap_or_default();
                let chunk = if leftover.is_empty() {
                    chunk
                } else {
                    let mut c = (*chunk).clone();
                    for (idx, id) in leftover {
                        if c.get_idx(idx as usize) == AIR {
                            c.set_idx(idx as usize, id);
                        }
                    }
                    Arc::new(c)
                };
                self.world.insert_generated(pos, chunk.clone(), outbound);
                // §27: villagers spawn with their village — populate the
                // wells whose reach covers this chunk (guarded, once)
                self.sim.villagers.populate_villages(&self.world, pos.0, pos.1);
                // Phase 5 §27: register spawner entities + fill dungeon
                // chest loot (fresh generation only — loaded chunks take
                // the no-fill path and their inventories arrive from
                // level.dat)
                self.register_block_entities(pos, &chunk, true);
                // Phase 4: initial lighting for the new chunk (column scan +
                // border exchange, settled synchronously) — the engine's
                // changed map feeds precise §12 dirty bits below
                self.light.init_chunk(&mut self.world, pos);
                for (lpos, lmask) in self.light.take_changed() {
                    self.world
                        .mark_sections_dirty(lpos, lmask, vc_world::world::CAUSE_LIGHT);
                }
                // the new chunk changes border face culling in its 8
                // neighbors — mark the sections whose y-bands touch the new
                // chunk's non-air border cells (NOT all 16: §12)
                let bands = neighbor_geometry_bands(&self.world, pos);
                for (npos, band) in bands {
                    self.world
                        .mark_sections_dirty(npos, band, vc_world::world::CAUSE_GEOMETRY);
                }
            }
            JobResult::Mesh { pos, mask, sections, mesh, occl } => {
                self.mesh_inflight.remove(&pos);
                // clear only the bits this job covered — edits that arrived
                // after its snapshot re-queue the chunk (§12)
                self.world.clear_dirty_mask(pos, mask);
                self.section_meshes.insert(pos, sections);
                self.renderer.set_chunk_mesh(pos, &mesh, occl);
            }
        }
    }

    // ---------------------------------------------------------------- ui --

    fn rebuild_ui(&mut self) {
        self.last_ui_t = self.time;
        self.ui.clear();

        match self.screen {
            Screen::Loading => {
                let pc = self.player_chunk();
                let mut have = 0.0;
                for dz in -2..=2 {
                    for dx in -2..=2 {
                        if self.renderer.has_chunk((pc.0 + dx, pc.1 + dz)) {
                            have += 1.0;
                        }
                    }
                }
                let progress = (have / 9.0_f32).min(1.0);
                self.ui.vignette_loading("Building terrain...", progress);
                return;
            }
            Screen::Title => {
                let splash = splash_for(self.time);
                self.ui.title_screen(splash, &self.widgets, self.hover, self.time);
                return;
            }
            Screen::Options => {
                let sub = if self.options_page == 1 {
                    "VIDEO DETAILS - 2/2"
                } else {
                    "SETTINGS APPLY INSTANTLY AND ARE SAVED"
                };
                self.ui.options_screen(&self.widgets, self.hover, sub);
                return;
            }
            Screen::Pause => {
                self.ui.pause_screen(&self.widgets, self.hover);
                return;
            }
            #[cfg(not(target_arch = "wasm32"))]
            Screen::WorldSelect => {
                let total = self.worlds.len();
                let shown = total.min(ui::MAX_LISTED_WORLDS);
                self.ui.world_select_screen(
                    &self.widgets,
                    self.hover,
                    self.ws_selected,
                    shown,
                    total,
                );
                return;
            }
            // wasm: the select screen is unreachable (no save list); the
            // arm keeps the match exhaustive
            #[cfg(target_arch = "wasm32")]
            Screen::WorldSelect => {
                self.widgets = Vec::new();
            }
            Screen::WorldCreate => {
                self.ui.world_create_screen(&self.widgets, self.hover, self.time);
                return;
            }
            Screen::Death => {
                self.ui.death_screen(
                    &self.widgets,
                    self.hover,
                    self.mode.permadeath(),
                    &self.death_cause,
                );
                return;
            }
            Screen::Game => {}
        }

        // in-game HUD
        self.ui.crosshair();
        let toast = self.item_toast.as_ref().map(|(s, t)| (s.as_str(), (*t * 200.0).clamp(0.0, 220.0) as u8));
        self.ui.hotbar(
            &self.player.inv.slots[..vc_inventory::inventory::INV_SLOTS.min(9)],
            self.player.selected,
            &self.atlas,
            toast,
        );
        let xp = self.player.xp_fraction();
        let level = self.player.xp_level.max(0) as u32;
        // §29: the health bar is REAL now — potions heal it, damage lowers it;
        // the XP bar shows the real in-level progress + level.
        // Phase 1: creative hides hearts + hunger (vanilla), XP stays
        // (creative still earns and spends enchanting levels here)
        if self.mode.invulnerable() {
            self.ui.xp_bar_only(xp, level);
        } else {
            self.ui.status_bars(self.player.health, 20.0, xp, level);
        }

        if self.show_debug {
            let p = &self.player;
            let pc = self.player_chunk();
            let biome = self
                .world
                .chunk(pc)
                .map(|c| {
                    let lx = (p.pos.x - pc.0 as f32 * 16.0).floor().clamp(0.0, 15.0) as usize;
                    let lz = (p.pos.z - pc.1 as f32 * 16.0).floor().clamp(0.0, 15.0) as usize;
                    Biome::from_u8(c.biome[lz * 16 + lx]).name().to_string()
                })
                .unwrap_or_else(|| "…".into());
            let facing = {
                let yaw = ((p.yaw.to_degrees() % 360.0) + 360.0) % 360.0;
                match yaw {
                    315.0..=360.0 | 0.0..=45.0 => "north (-Z)",
                    45.0..=135.0 => "east (+X)",
                    135.0..=225.0 => "south (+Z)",
                    _ => "west (-X)",
                }
            };
            let lines = vec![
                format!(
                    "VOXELCRAFT (Rust + wgpu)  {} fps  ({} min / {} avg / {} max)",
                    self.fps as i32,
                    self.fps_min as i32,
                    self.fps_avg as i32,
                    self.fps_max as i32
                ),
                format!(
                    "Frame: {:.2} ms  Chunks: {} drawn / {} loaded  Tris: {}",
                    self.frame_ms,
                    self.stats.chunks,
                    self.world.chunks.len(),
                    self.stats.tris
                ),
                // Phase 6 §26: quality row (occlusion culling counter + the
                // four new settings, MSAA shows the ACTIVE device count) +
                // the Phase 7 meshing backend
                format!(
                    "Culled: {}  Sim: {}  Mip: {}  Aniso: {}x  MSAA: {}{}  Mesh: {}",
                    self.stats.culled,
                    self.settings.sim_distance,
                    self.settings.mipmap_levels,
                    self.settings.aniso,
                    if self.renderer.msaa() == 0 { "off".to_string() } else { self.renderer.msaa().to_string() },
                    if self.settings.occlusion { "" } else { "  (occl off)" },
                    match (&self.renderer.gpu_mesh, self.settings.gpu_meshing) {
                        (Some(m), true) => format!("GPU ({})", m.jobs_done),
                        (Some(_), false) => "CPU (off)".to_string(),
                        (None, _) => "CPU".to_string(),
                    }
                ),
                format!(
                    "Draws: {} avg  Binds: {} avg  Path: {}",
                    self.draw_calls_ring.iter().map(|d| d.0).sum::<u32>()
                        / self.draw_calls_ring.len().max(1) as u32,
                    self.draw_calls_ring.iter().map(|d| d.1).sum::<u32>()
                        / self.draw_calls_ring.len().max(1) as u32,
                    self.renderer.draw_path_name()
                ),
                format!("XYZ: {:.2} / {:.2} / {:.2}", p.pos.x, p.pos.y, p.pos.z),
                format!("Block: {} {} {}  ({})", p.pos.x as i32, p.pos.y as i32, p.pos.z as i32, ""),
                format!("Chunk: {} {}  Facing: {}  Light: sky {}",
                    pc.0, pc.1, facing,
                    "-"),
                format!("Biome: {}  Dimension: {}", biome, self.world.dimension.id()),
                // Phase 1: active game mode + world identity
                format!(
                    "Mode: {}  World: \"{}\"  Seed: {}",
                    self.mode.label(),
                    self.world_name,
                    self.world.seed
                ),
                // Phase 2: mob system state
                format!(
                    "Mobs: {} alive / {} spawned / {} killed  Arrows: {}  Swing: {:.0}%",
                    self.sim.mobs.len(),
                    self.sim.mobs.spawned_total,
                    self.sim.mobs.killed_total,
                    self.sim.mobs.arrows.len(),
                    self.swing_t.min(9.99) * 10.0
                ),
                format!(
                    "Day cycle: {:.0}%  Fly: {}",
                    self.day_time * 100.0,
                    if self.player.flying { "on" } else { "off" }
                ),
                format!(
                    "Targeted: {}",
                    self.target
                        .map(|(t, _, _)| {
                            // vanilla F3 parity: exact blockstate, e.g.
                            // "Oak Slab[half=top]" / "Oak Fence[north=true]"
                            state_description(self.world.get_state(t[0], t[1], t[2]))
                        })
                        .unwrap_or("none".into())
                ),
                format!(
                    "RD: {}  FOV: {:.0}  Vol: {:.0}%  Bright: {:.0}%",
                    self.settings.render_distance,
                    self.settings.fov,
                    self.settings.volume * 100.0,
                    self.settings.brightness * 100.0
                ),
                format!(
                    "Graphics: {}  Shadows: {}  FSR1: {}  MaxFPS: {}",
                    ["fast", "fancy", "fabulous"][self.settings.graphics as usize],
                    match self.settings.shadow_quality {
                        0 => "off",
                        1 => "1K",
                        2 => "2K",
                        _ => "4K",
                    },
                    match self.settings.upscale {
                        0 => "native",
                        1 => "75% EASU+RCAS",
                        _ => "50% EASU+RCAS",
                    },
                    match self.settings.maxfps { 0 => "vsync", 1 => "30", 2 => "60", _ => "120" }
                ),
                format!(
                    "Dirty: {} sections in {} chunks (§12 fine-grained invalidation)",
                    self.world.dirty_section_count(),
                    self.world.dirty.len()
                ),
                format!(
                    "Shader: {}  Clouds: {}  Smooth: {}  VSync: {}",
                    self.shader_mode_name(self.settings.shader),
                    if self.settings.clouds { "on" } else { "off" },
                    if self.settings.smooth_lighting { "on" } else { "off" },
                    if self.renderer.vsync { "on" } else { "off" }
                ),
                format!(
                    "Pack: {}",
                    match (&self.renderer.pack_id, &self.renderer.pack_tier) {
                        (Some(id), tier) if !tier.is_empty() => format!("{id} ({tier})"),
                        _ => "none".to_string(),
                    }
                ),
                format!("Edits: {} (xp lvl {})  Seed: {}", self.edits, level, self.world.seed),
                format!("Backend: {}  Scene: {}x{}", self.renderer.backend_name, self.renderer.scene_size().0, self.renderer.scene_size().1),
                // Phase-0 (§44): per-frame CPU phase breakdown
                self.phases.f3_line(),
            ];
            self.ui.debug(&lines);
            // Sodium-style frame-time graph right under the text block
            self.ui.frame_graph(7 + lines.len() as i32 * 14 + 8, self.frame_times.as_slices().0);
        }

        if self.show_help {
            self.ui.help();
        }

        // container overlay (§27) — sits above the HUD. Owned snapshot so
        // the renderer never borrows game state.
        if self.container.is_some() {
            let view = self.container_view();
            let g = self.ui.container_screen(&view, self.cursor, &self.atlas);
            self.container_geom = Some(g);
        } else {
            self.container_geom = None;
        }

        // block picker overlay (B) — sits above the HUD
        if self.picker_open {
            let g = self.ui.picker(self.cursor, &self.atlas);
            self.picker_geom = Some(g);
        } else {
            self.picker_geom = None;
        }

        // mouse-capture hint when unlocked and no drag-look fallback
        #[cfg(target_arch = "wasm32")]
        {
            if !self.pointer_locked && !self.drag_look {
                self.ui.center_msg("", "CLICK THE CANVAS TO CAPTURE THE MOUSE");
            }
        }
    }

    // -------------------------------------------------------------- draw --

    fn draw(&mut self) {
        // Phase-0 instrumentation: frame phases (§44)
        self.phases.begin_frame();
        let t_draw0 = crate::bench::micros();
        // fps = real RENDERED frame rate (draws ride RAF on the web)
        self.frames += 1;
        // rolling frame-time history (Sodium-style F3 graph + min/avg/max).
        // Uses the GAME-TIME delta between draws: the wall-clock
        // `last_draw_t` is stamped after the last `self.time` advance in the
        // same RAF tick, so time - last_draw_t reads ≈ 0 on wasm.
        let t_draw = self.time;
        self.frame_ms = (t_draw - self.draw_game_t).max(0.0) * 1000.0;
        if self.frame_ms > 1.0 && self.frame_ms < 500.0 {
            self.frame_times.push_back(self.frame_ms);
            while self.frame_times.len() > 180 {
                self.frame_times.pop_front();
            }
        }
        // Phase 9: draw-call/bind history (F3 + benchmark §37)
        self.draw_calls_ring.push_back((self.stats.draws, self.stats.binds));
        while self.draw_calls_ring.len() > 64 {
            self.draw_calls_ring.pop_front();
        }
        self.draw_game_t = t_draw;
        if self.time - self.fps_t > 0.5 {
            self.fps = self.frames as f32 / (self.time - self.fps_t);
            self.frames = 0;
            self.fps_t = self.time;
            self.ui.dirty = true;
        }
        // recompute the rolling min/avg/max once per FPS window
        if !self.frame_times.is_empty() {
            let n = self.frame_times.len() as f32;
            let total: f32 = self.frame_times.iter().sum();
            self.fps_avg = 1000.0 / (total / n);
            // BLOCKING-BUG FIX (user report — F3 showed "max 2147483547
            // fps"): the folds had swapped initializers — fold(0.0, min)
            // collapses to 0.0 (fps_max = 1000/0 = inf → saturates to
            // i32::MAX in the overlay) and fold(INF, max) collapses to INF
            // (fps_min = 0). Max frame time folds up from 0.0; min frame
            // time folds down from INFINITY.
            let (lo, hi) = fps_min_max(&self.frame_times);
            self.fps_min = lo;
            self.fps_max = hi;
        }
        // Only log the first frames of each actual game instance — the FPS
        // window counter (`self.frames`) resets every 0.5 s, so without the
        // time gate this spams "frame #1..#3" twice a second and makes
        // remounts impossible to distinguish from normal windows.
        if self.frames < 3 && self.time_since_load() < 2.0 {
            vc_render::render::report_boot_log(&format!(
                "draw() frame #{}: chunks_gpu={}, screen={:?}",
                self.frames + 1,
                self.renderer.chunks.len(),
                self.screen
            ));
        }
        // day/night state — §28: the Nether has no sky: constant dim
        // ambient (vanilla's flat nether light), thick dark-red fog close
        // in, no sun/shadows/clouds (the skyless flag drops the sky pass)
        let nether = self.world.dimension == vc_world::world::Dimension::Nether;
        let (sun_dir, day_light, fog) = if nether {
            (
                Vec3::new(0.0, 1.0, 0.0), // cosmetic only — skyless
                0.30,
                [0.16, 0.02, 0.02],
            )
        } else {
            let theta = self.day_time * std::f32::consts::TAU;
            let sun_dir = Vec3::new(theta.cos() * 0.85, theta.sin(), -0.4).normalize();
            let day_light = 0.16 + 0.84 * smoothstep(-0.10, 0.14, sun_dir.y);
            let sunset = (1.0 - (sun_dir.y * 4.0).abs()).clamp(0.0, 1.0)
                * (day_light.clamp(0.2, 0.8) - 0.2) / 0.6;
            let day_fog = [0.75, 0.85, 1.0];
            let night_fog = [0.02, 0.03, 0.07];
            let mut fog = [
                day_fog[0] * day_light + night_fog[0] * (1.0 - day_light),
                day_fog[1] * day_light + night_fog[1] * (1.0 - day_light),
                day_fog[2] * day_light + night_fog[2] * (1.0 - day_light),
            ];
            fog[0] += 0.65 * sunset;
            fog[1] += 0.22 * sunset;
            fog[2] -= 0.02 * sunset;
            (sun_dir, day_light, fog)
        };

        let rd = self.settings.render_distance;
        let (fog_start, fog_end, fog_col) = if self.player.head_in_water && self.screen == Screen::Game {
            (2.0, 28.0, [0.11, 0.22, 0.45])
        } else if nether {
            // thick nether fog well inside any render distance
            (8.0, 44.0, fog)
        } else {
            let end = (rd * 16 - 12) as f32;
            (end * 0.55, end, fog)
        };

        // camera: panorama on the title screen, player otherwise
        let (cam, menu_blur, selection) = match self.screen {
            Screen::Title => {
                let cam = Camera {
                    eye: Vec3::new(self.player.pos.x, self.player.pos.y + 14.0, self.player.pos.z),
                    yaw: self.time * 0.06,
                    pitch: -0.42,
                    fov: 1.2217,
                };
                (cam, 0.9, None)
            }
            Screen::Options if self.options_from == Screen::Title => {
                let cam = Camera {
                    eye: Vec3::new(self.player.pos.x, self.player.pos.y + 14.0, self.player.pos.z),
                    yaw: self.time * 0.06,
                    pitch: -0.42,
                    fov: 1.2217,
                };
                (cam, 0.9, None)
            }
            // Phase 1: world screens ride the (blurred) panorama like the
            // title/options screens — the world list previews the world
            Screen::WorldSelect | Screen::WorldCreate => {
                let cam = Camera {
                    eye: Vec3::new(self.player.pos.x, self.player.pos.y + 14.0, self.player.pos.z),
                    yaw: self.time * 0.06,
                    pitch: -0.42,
                    fov: 1.2217,
                };
                (cam, 0.9, None)
            }
            // Phase 1: death screen — frozen first-person view behind a
            // heavy red wash (the UI overlay paints it)
            Screen::Death => {
                let cam = Camera {
                    eye: self.player.eye(),
                    yaw: self.player.yaw,
                    pitch: self.player.pitch,
                    fov: self.player.fov_cur,
                };
                (cam, 0.55, None)
            }
            Screen::Loading => {
                let cam = Camera {
                    eye: self.player.eye(),
                    yaw: self.player.yaw,
                    pitch: self.player.pitch,
                    fov: self.player.fov_cur,
                };
                (cam, 0.35, None)
            }
            Screen::Pause | Screen::Options => {
                let cam = Camera {
                    eye: self.player.eye(),
                    yaw: self.player.yaw,
                    pitch: self.player.pitch,
                    fov: self.player.fov_cur,
                };
                (cam, 0.55, None)
            }
            Screen::Game => {
                let cam = Camera {
                    eye: self.player.eye(),
                    yaw: self.player.yaw,
                    pitch: self.player.pitch,
                    fov: self.player.fov_cur,
                };
                let sel = self.target.map(|(pos, _, _)| (pos[0], pos[1], pos[2]));
                (cam, 0.0, sel)
            }
        };

        let sky = SkyState {
            day_light,
            sun_dir,
            fog_color: fog_col,
            fog_start,
            fog_end,
            time: self.time,
            underwater: self.player.head_in_water && self.screen == Screen::Game,
            min_light: 0.05 + self.settings.brightness * 0.25,
            // §28: no sky pass in the Nether — the fog-colored clear is the
            // whole "sky" (dark red haze, no sun, no gradient)
            skyless: nether,
        };

        // particle billboards: camera basis from the active camera (game
        // camera or menu panorama camera)
        {
            let dir = [
                cam.yaw.sin() * cam.pitch.cos(),
                cam.pitch.sin(),
                -cam.yaw.cos() * cam.pitch.cos(),
            ];
            // right = normalize(dir × world-up) = (-dz, 0, dx)
            let rx = -dir[2];
            let rz = dir[0];
            let rl = (rx * rx + rz * rz).sqrt().max(1e-6);
            let right = [rx / rl, 0.0, rz / rl];
            let up = [
                right[1] * dir[2] - right[2] * dir[1],
                right[2] * dir[0] - right[0] * dir[2],
                right[0] * dir[1] - right[1] * dir[0],
            ];
            self.particles
                .build_vertices(right, up, &mut self.particle_verts);
            // item entities share the billboard pipeline (§22 progressive)
            self.sim
                .items
                .build_vertices(self.time, right, up, &mut self.particle_verts);
            // §27/§29 villagers: crossed-quad sprites, villager scale
            vc_gameplay::villagers::build_vertices(
                &self.sim.villagers.list,
                self.time,
                right,
                up,
                &mut self.particle_verts,
            );
            // Phase 2: mobs + skeleton arrows share the billboard pipeline
            vc_gameplay::mobs::build_vertices(
                &self.sim.mobs.list,
                right,
                &mut self.particle_verts,
            );
            vc_gameplay::mobs::build_arrow_vertices(
                &self.sim.mobs.arrows,
                right,
                up,
                &mut self.particle_verts,
            );
        }

        self.stats = self.renderer.render(
            &cam,
            &sky,
            &mut self.ui,
            selection,
            &vc_render::render::PostParams {
                mode: self.settings.shader,
                menu_blur,
                // §28: the Nether has no sun — no shadow pass
                shadows: if nether { 0.0 } else { self.settings.shadow_strength() },
                // FSR 1.0: RCAS lobe factor when the internal scale is below
                // native (0.6 ≈ FsrRcasCon(~0.7 stops) — sharp without halos;
                // EASU already reconstructs most of the edge contrast)
                sharpen: if self.settings.upscale > 0 { 0.6 } else { 0.0 },
            },
            self.settings.clouds && self.settings.graphics >= 1 && !nether,
            &self.particle_verts,
        );
        self.phases
            .add(crate::bench::PHASE_DRAW, crate::bench::micros() - t_draw0);

        // --- bench bookkeeping: count measured frames, finish + exit (§37)
        if let Some(bs) = self.bench.as_mut() {
            if self.screen == Screen::Game {
                bs.seen += 1;
                if bs.seen == bs.warmup + 1 {
                    vc_render::render::report_boot_log("benchmark: warmup done, measuring");
                }
                if bs.seen >= bs.warmup + bs.frames {
                    let (stats, report, mode) = {
                        let times: Vec<u64> = self.phases.frame_times_us().iter().copied().collect();
                        let fs = crate::bench::FrameStats::from_us(&times);
                        let pr = self.phases.report();
                        (fs, pr, self.renderer.present_mode_name())
                    };
                    if let Some(fs) = stats {
                        crate::bench::print_report(&fs, &report, &mode);
                        // Phase 9 §37/§48 gate: draw-call + bind counts travel
                        // with the frame-time report (rolling 64-frame means)
                        let n = self.draw_calls_ring.len().max(1) as u32;
                        let d_avg = self.draw_calls_ring.iter().map(|d| d.0).sum::<u32>() / n;
                        let b_avg = self.draw_calls_ring.iter().map(|d| d.1).sum::<u32>() / n;
                        let json = format!(
                            "{{\"benchmark\":{{\"frame\":{},\"phases\":{},\"draw\":{{\"calls_avg\":{},\"binds_avg\":{},\"path\":\"{}\"}}}}}}",
                            fs.to_json(),
                            report.to_json(),
                            d_avg,
                            b_avg,
                            self.renderer.draw_path_name()
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(path) = &self.bench.as_ref().and_then(|b| b.json_path.clone()) {
                            let _ = std::fs::write(path, json.clone());
                            println!("benchmark JSON written to {path}");
                        }
                        println!("benchmark JSON: {json}");
                    }
                    self.quit_requested = true;
                }
            }
        }
        self.phases.end_frame();
    }
}

// ---------------------------------------------------------------- helpers --

/// does a block connect a fence? (vanilla rule: solid blocks + fences)
#[inline]
/// Phase 11 §34: shader-mode index → pack list index. Modes 0..2 are the
/// engine's own grades; 3.. selects pack i-3 when it exists.
fn shader_mode_pack_index(mode: u8, n_packs: usize) -> Option<usize> {
    let i = mode as usize;
    if i >= 3 && i - 3 < n_packs {
        Some(i - 3)
    } else {
        None
    }
}

fn fence_connects_to(b: u8) -> bool {
    is_solid(b) || b == OAK_FENCE
}

/// compute the fence state (connection booleans) for a position from the
/// current world (vanilla: connect to solid blocks and other fences)
fn fence_state_for(world: &World, wx: i32, wy: i32, wz: i32) -> Option<u16> {
    let north = fence_connects_to(world.get_block(wx, wy, wz - 1));
    let east = fence_connects_to(world.get_block(wx + 1, wy, wz));
    let south = fence_connects_to(world.get_block(wx, wy, wz + 1));
    let west = fence_connects_to(world.get_block(wx - 1, wy, wz));
    prop_state_encode(
        OAK_FENCE,
        &[
            ("east", if east { "true" } else { "false" }),
            ("north", if north { "true" } else { "false" }),
            ("south", if south { "true" } else { "false" }),
            ("west", if west { "true" } else { "false" }),
        ],
    )
}


/// block-change notification for the whole sim (fluids + gravity +
/// redstone) — the §25 ordering backbone entry point
fn notify_sim(world: &World, sched: &mut vc_sim::ticks::TickScheduler, x: i32, y: i32, z: i32) {
    vc_sim::fluids::on_block_changed(sched, world, x, y, z);
    vc_sim::redstone::on_block_changed(sched, world, x, y, z);
}

/// Phase 5 §27: fill a freshly generated dungeon chest. Palette-bounded
/// adaptation of the vanilla "simple dungeon" table (bones, string,
/// gunpowder, rotten flesh, arrows, occasional iron): 3..=7 stacks of
/// 1..=4, deterministic from the world seed + chest position. Vanilla
/// saddle/music-disc/golden-apple slots are palette-absent and simply
/// don't roll (documented, not substituted with lookalikes).
/// Phase 9/10: fresh structure chests roll through the data-pack loot
/// system (vanilla table-name seam): a pack that ships the table
/// overrides the palette-limited builtin; either way the distribution is
/// pools/rolls/weights/set_count — the exact vanilla model, with our own
/// palette values in the builtin defaults (saddle/music-disc/golden-apple
/// slots are palette-absent and simply don't roll — documented, not
/// substituted with lookalikes).
fn fill_structure_chest(
    data: &vc_pack::datapack::LoadedData,
    table: &str,
    inv: &mut vc_sim::containers::ContainerInv,
    seed: u64,
    pos: [i32; 3],
) {
    let mut rng = vc_rng::rng::Rng::new(vc_rng::rng::Rng::hash3(
        seed ^ 0xDCC_E5,
        pos[0],
        pos[1],
        pos[2],
    ));
    let stacks = data.roll(table, &mut rng).unwrap_or_default();
    let mut slot = rng.next_range(27) as usize;
    for (item, count) in stacks {
        // walk to the next free slot (chests are fresh — always one)
        for _ in 0..27 {
            if inv.slots[slot].is_empty() {
                inv.slots[slot] =
                    vc_inventory::inventory::ItemStack::new(item, count);
                break;
            }
            slot = (slot + 1) % 27;
        }
        slot = (slot + 1 + rng.next_range(3) as usize) % 27;
    }
}

/// biome + (sky, block) light levels at a world position — for baking
/// particle tint/brightness at spawn (Phase 5)
/// Phase 2: edible mob drops (right-click to eat — heals directly until
/// the hunger system exists; documented deviation)
fn is_food(b: u8) -> bool {
    matches!(b, BEEF | PORKCHOP | MUTTON | CHICKEN_RAW | ROTTEN_FLESH)
}

fn light_at(world: &World, light: &vc_world::light::LightEngine, wx: i32, wy: i32, wz: i32) -> (u8, u8, u8) {
    let _ = light; // engine state lives in world.light (LightData map)
    let cx = wx.div_euclid(16);
    let cz = wz.div_euclid(16);
    let lx = (wx - cx * 16) as usize;
    let lz = (wz - cz * 16) as usize;
    let biome = world
        .chunk((cx, cz))
        .map(|c| c.biome[lz * 16 + lx])
        .unwrap_or(2);
    let (sky, blk) = world
        .light
        .get(&(cx, cz))
        .and_then(|ld| {
            let sec = (wy.clamp(0, 255) / 16) as usize;
            let yy = (wy.clamp(0, 255) % 16) as usize;
            let idx = (yy << 8) | (lz << 4) | lx;
            ld.sections[sec].as_ref().map(|s| (s.sky[idx], s.blk[idx]))
        })
        .unwrap_or((15, 0));
    (biome, sky, blk)
}

/// after an edit at (wx, wy, wz), refresh the connection states of any fence
/// blocks among the 4 horizontal neighbors (and the position itself if the
/// edit placed a fence — handled by the caller passing its own state).
fn update_fence_neighbors(world: &mut World, wx: i32, wy: i32, wz: i32) {
    for (dx, dz) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
        let nx = wx + dx;
        let nz = wz + dz;
        if world.get_block(nx, wy, nz) == OAK_FENCE {
            if let Some(s) = fence_state_for(world, nx, wy, nz) {
                // skip no-op writes (avoids dirty churn on repeated edits)
                if world.get_state(nx, wy, nz) != s {
                    world.set_block_state(nx, wy, nz, s);
                }
            }
        }
    }
    // the fence at the edit position itself (placed or revealed)
    if world.get_block(wx, wy, wz) == OAK_FENCE {
        if let Some(s) = fence_state_for(world, wx, wy, wz) {
            if world.get_state(wx, wy, wz) != s {
                world.set_block_state(wx, wy, wz, s);
            }
        }
    }
}

/// §12 streaming geometry bands: for each of the 8 neighbors of a newly
/// generated chunk, the sections whose y-bands touch the new chunk's
/// non-air cells along the shared face (face culling + AO read ±1 cells).
/// Replaces the old mark-all-16 — a surface chunk (y ≤ ~90) dirties ≤ 6
/// sections per neighbor instead of 16.
fn neighbor_geometry_bands(world: &World, pos: ChunkPos) -> Vec<(ChunkPos, u16)> {
    let mut out = Vec::with_capacity(8);
    let Some(chunk) = world.chunk(pos) else { return out };
    // per-direction shared-face columns (in the NEW chunk's local coords)
    let faces: [(i32, i32, Vec<(usize, usize)>); 8] = [
        (1, 0, (0..16).map(|t| (15, t)).collect()),
        (-1, 0, (0..16).map(|t| (0, t)).collect()),
        (0, 1, (0..16).map(|t| (t, 15)).collect()),
        (0, -1, (0..16).map(|t| (t, 0)).collect()),
        (1, 1, vec![(15, 15)]),
        (1, -1, vec![(15, 0)]),
        (-1, 1, vec![(0, 15)]),
        (-1, -1, vec![(0, 0)]),
    ];
    for (dx, dz, cols) in faces {
        let mut y_min = 256i32;
        let mut y_max = -1i32;
        for (lx, lz) in cols {
            for sy in (0..16usize).rev() {
                let Some(sec) = &chunk.sections[sy] else { continue };
                if sec.is_empty() {
                    continue;
                }
                let flat = sec.decode_flat();
                let base = sy * 16;
                for yy in (0..16usize).rev() {
                    if flat[(yy << 8) | (lz << 4) | lx] != 0 {
                        let y = (base + yy) as i32;
                        y_max = y_max.max(y);
                        y_min = y_min.min(y);
                    }
                }
            }
        }
        if y_max < 0 {
            continue; // nothing to cull against on this face
        }
        // ±1 for AO corner reads
        let lo = (y_min - 1).max(0) / 16;
        let hi = (y_max + 1).min(255) / 16;
        let mut band = 0u16;
        for s in lo..=hi {
            band |= 1 << s;
        }
        out.push(((pos.0 + dx, pos.1 + dz), band));
    }
    out
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn splash_for(time: f32) -> &'static str {
    let idx = ((time / 7.0).floor() as usize) % SPLASHES.len();
    SPLASHES[idx]
}

/// map a KeyboardEvent.code string to a winit KeyCode (web input path)
#[cfg(target_arch = "wasm32")]
fn keycode_from_web(code: &str) -> Option<winit::keyboard::KeyCode> {
    use winit::keyboard::KeyCode;
    Some(match code {
        "KeyW" => KeyCode::KeyW,
        "KeyA" => KeyCode::KeyA,
        "KeyS" => KeyCode::KeyS,
        "KeyD" => KeyCode::KeyD,
        "Space" => KeyCode::Space,
        "ShiftLeft" => KeyCode::ShiftLeft,
        "ShiftRight" => KeyCode::ShiftRight,
        "ControlLeft" => KeyCode::ControlLeft,
        "ControlRight" => KeyCode::ControlRight,
        "Escape" => KeyCode::Escape,
        "F3" => KeyCode::F3,
        "Backspace" => KeyCode::Backspace,
        "Enter" => KeyCode::Enter,
        "KeyE" => KeyCode::KeyE,
        "KeyB" => KeyCode::KeyB,
        "KeyH" => KeyCode::KeyH,
        "KeyV" => KeyCode::KeyV,
        "BracketLeft" => KeyCode::BracketLeft,
        "BracketRight" => KeyCode::BracketRight,
        "Minus" => KeyCode::Minus,
        "Equal" => KeyCode::Equal,
        "Digit1" => KeyCode::Digit1,
        "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3,
        "Digit4" => KeyCode::Digit4,
        "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6,
        "Digit7" => KeyCode::Digit7,
        "Digit8" => KeyCode::Digit8,
        "Digit9" => KeyCode::Digit9,
        _ => return None,
    })
}

fn set_slider(w: &mut Widget, label: &str, value: f32) {
    if let WidgetKind::Slider { label: l, value: v } = &mut w.kind {
        *l = label.to_string();
        *v = value.clamp(0.0, 1.0);
    }
}

/// Phase 1 (web): map a physical key code + shift state to the character
/// it types — enough for world names / seeds (a–z, 0–9, symbols, space,
/// dash, underscore, dot). The native path uses winit's logical_key
/// (full Unicode) instead.
#[cfg(target_arch = "wasm32")]
fn web_char_from_code(code: &str, shift: bool) -> Option<char> {
    if code == "Space" {
        return Some(' ');
    }
    if code == "Minus" {
        return Some(if shift { '_' } else { '-' });
    }
    if code == "Period" {
        return Some('.');
    }
    if let Some(rest) = code.strip_prefix("Key") {
        let c = rest.chars().next()?;
        if c.is_ascii_alphabetic() {
            return Some(if shift {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            });
        }
        return None;
    }
    if let Some(rest) = code.strip_prefix("Digit") {
        let d = rest.chars().next()?;
        let sym = match d {
            '1' => Some(('1', '!')),
            '2' => Some(('2', '@')),
            '3' => Some(('3', '#')),
            '4' => Some(('4', '$')),
            '5' => Some(('5', '%')),
            '6' => Some(('6', '^')),
            '7' => Some(('7', '&')),
            '8' => Some(('8', '*')),
            '9' => Some(('9', '(')),
            '0' => Some(('0', ')')),
            _ => None,
        }?;
        return Some(if shift { sym.1 } else { sym.0 });
    }
    None
}

fn set_button_value(w: &mut Widget, value: &str) {
    if let WidgetKind::Button { value: v, .. } = &mut w.kind {
        *v = value.to_string();
    }
}

/// Phase 9: honest boot-log reporting for the scanned data packs — one
/// summary line per pack, plus every skipped file (parse failures,
/// palette gaps, unsupported content) as its own reason line. Never
/// fatal: vanilla prompts Safe Mode for broken packs; the engine has no
/// pack-selection screen, so it degrades to the working parts + reports.
fn report_datapacks(loaded: &vc_pack::datapack::LoadedData) {
    if loaded.packs.is_empty() {
        return; // no packs — no log noise
    }
    for pack in &loaded.packs {
        if pack.pack_format != vc_pack::datapack::PACK_FORMAT_1_16_5 {
            vc_render::render::report_boot_log(&format!(
                "data pack {}: pack_format {} (1.16.5 wants 6) — loading anyway",
                pack.id, pack.pack_format
            ));
        }
        vc_render::render::report_boot_log(&format!(
            "data pack: {}{}",
            pack.summary(),
            if pack.description.is_empty() {
                String::new()
            } else {
                format!(" ({})", pack.description)
            },
        ));
        for (kind, count) in &pack.unsupported {
            vc_render::render::report_boot_log(&format!(
                "data pack {}: {count}x {kind}/ entries detected — not supported yet, reported honestly",
                pack.id
            ));
        }
        for s in &pack.skipped {
            vc_render::render::report_boot_log(&format!(
                "data pack {}: skipped {s}",
                pack.id
            ));
        }
    }
}

#[cfg(test)]
mod settings_tests {
    use super::Settings;

    /// Phase 6 §26: the new quality settings survive a serialize →
    /// deserialize round trip, and a legacy settings string (pre-Phase-6
    /// save, no new keys) parses to the DEFAULTS for those keys
    #[test]
    fn quality_settings_roundtrip() {
        let mut s = Settings::default();
        s.sim_distance = 7;
        s.mipmap_levels = 2;
        s.aniso = 8;
        s.msaa = 4;
        s.occlusion = false;
        let restored = Settings::deserialize(&s.serialize());
        assert_eq!(restored.sim_distance, 7);
        assert_eq!(restored.mipmap_levels, 2);
        assert_eq!(restored.aniso, 8);
        assert_eq!(restored.msaa, 4);
        assert!(!restored.occlusion);
        // the pre-existing keys still round trip
        assert_eq!(restored.render_distance, s.render_distance);
        assert_eq!(restored.fov, s.fov);
        assert_eq!(restored.smooth_lighting, s.smooth_lighting);
    }

    /// legacy settings strings (Phase 5 era) keep parsing; the Phase 6 keys
    /// fall back to their defaults
    #[test]
    fn legacy_settings_string_parses() {
        let legacy = "rd=8;sens=1.200;vol=0.500;mvol=0.400;fov=80.0;bright=0.250;smooth=1;clouds=0;graphics=2;shader=1;shadowq=3;upscale=1;maxfps=2";
        let s = Settings::deserialize(legacy);
        assert_eq!(s.render_distance, 8);
        assert_eq!(s.fov, 80.0);
        assert_eq!(s.graphics, 2);
        assert!(!s.clouds);
        assert_eq!(s.upscale, 1);
        // Phase 6 keys → defaults
        assert_eq!(s.sim_distance, 12);
        assert_eq!(s.mipmap_levels, 4);
        assert_eq!(s.aniso, 4);
        assert_eq!(s.msaa, 0);
        assert!(s.occlusion);
    }

    /// Phase 7: the GPU-meshing flag round trips and legacy strings fall
    /// back to the platform default
    #[test]
    fn gpu_meshing_flag_roundtrips() {
        let mut s = Settings::default();
        let native_default = s.gpu_meshing;
        s.gpu_meshing = !native_default;
        let restored = Settings::deserialize(&s.serialize());
        assert_eq!(restored.gpu_meshing, !native_default);
        // legacy string (no gmesh key) → platform default
        let legacy = "rd=8;smooth=1;clouds=0";
        let s2 = Settings::deserialize(legacy);
        assert_eq!(s2.gpu_meshing, native_default, "legacy save keeps the platform default");
    }

    /// garbage msaa values snap to the valid set (0/4/8)
    #[test]
    fn msaa_values_snap_to_valid_counts() {
        for (raw, want) in [(0u8, 0u8), (1, 0), (2, 4), (3, 4), (4, 4), (5, 4), (6, 8), (8, 8), (16, 8)] {
            let s = Settings::deserialize(&format!("msaa={raw}"));
            assert_eq!(s.msaa, want, "raw {raw} should snap to {want}");
        }
    }

    /// Phase 8: the Iris interface is wired into this crate — the demo
    /// document the E2E `iris` command parses must produce exactly the
    /// numbers the e2e log line claims, and the translator seam must
    /// report its honest default (the sister project is not registered).
    /// Mirrors the vc-render `demo_document_matches_e2e_claims` test at
    /// the app level so drift breaks one of the two.
    #[test]
    fn iris_interface_e2e_claims_hold() {
        let props = vc_render::iris::ShadersProperties::parse(vc_render::iris::DEMO_PROPERTIES);
        assert_eq!(props.profiles().len(), 2);
        assert_eq!(props.sliders().len(), 3);
        assert!(props.unknown.is_empty());
        let (version, targets) =
            vc_render::iris::parse_stage_directives(Some(vc_render::iris::DEMO_STAGE_GLSL));
        assert_eq!(version.as_deref(), Some("330 compatibility"));
        assert_eq!(targets, vec![0, 1]);
        // honest default: no translator until the sister project registers
        assert!(!vc_render::iris::translator().supports_version("330 compatibility"));
        // missing scan root → empty (how the wasm build boots)
        assert!(vc_render::iris::scan_shader_packs(std::path::Path::new(
            "no-such-dir-iris"
        ))
        .is_empty());
    }

    /// Phase 9: the data-pack pipeline the `dpdemo` E2E command runs must
    /// produce exactly the numbers its log lines claim — the demo pack
    /// scans to (2 recipes, 1 loot table, 1 tag, 1 unsupported
    /// advancement), the 2×2-cobble grid crafts Stone Bricks x4, red wool
    /// crafts String (tag-driven shapeless), and the demo loot table
    /// rolls inside its declared grammar. Mirrors the vc-pack
    /// `demo_pack_end_to_end` test so drift breaks one of the two.
    #[test]
    fn datapack_demo_e2e_claims_hold() {
        use vc_pack::datapack::{GridItem, MemoryFiles, PackFiles};
        let files = MemoryFiles::demo();
        let report = vc_pack::datapack::scan_pack("demo", &files).expect("demo pack valid");
        assert_eq!(report.pack_format, vc_pack::datapack::PACK_FORMAT_1_16_5);
        assert_eq!(report.recipes.len(), 2);
        assert_eq!(report.loot_tables.len(), 1);
        assert_eq!(report.tags.len(), 1);
        assert_eq!(report.unsupported, vec![("advancements".to_string(), 1)]);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);

        let loaded = vc_pack::datapack::LoadedData::from_reports(vec![report]);
        // dpdemo line 1: "craft 2x2 cobble -> Stone Bricks x4"
        let grid = vec![GridItem::item("minecraft:cobblestone", 5); 4];
        let (b, c) = loaded.match_grid(&grid, 2).expect("cobble grid matches");
        assert_eq!((b, c), (vc_blocks::blocks::STONE_BRICKS, 4));
        // dpdemo line 1: "red wool -> String x1"
        let wool = vec![GridItem::item("minecraft:red_wool", 1)];
        let (b, c) = loaded.match_grid(&wool, 1).expect("wool matches via tag");
        assert_eq!((b, c), (vc_blocks::blocks::STRING, 1));
        // dpdemo line 2: loot rolls within 2..=4 stacks of palette items
        let mut rng = vc_rng::rng::Rng::new(99);
        let stacks = loaded.roll("demo:demo_loot", &mut rng).expect("table rolls");
        assert!((2..=4).contains(&stacks.len()));
        for (id, count) in stacks {
            assert!(
                [vc_blocks::blocks::IRON_ORE, vc_blocks::blocks::GOLD_ORE, vc_blocks::blocks::BONE]
                    .contains(&id)
            );
            if id == vc_blocks::blocks::IRON_ORE {
                assert!((1..=2).contains(&count));
            }
        }
    }

    /// Phase 10 E2E claims hold: the `mineshaft` / `pyramid` /
    /// `stronghold` / `biome` command log lines state structure facts
    /// that must stay true (corridor lens 24..=48, the 4-chest pit +
    /// desert_pyramid loot, ring-1 = 3 strongholds in 1280..=2816, 14
    /// biomes) — this mirrors those log lines so drift breaks either
    /// the live E2E or this test, the same discipline as the P8/P9
    /// claims tests.
    #[test]
    fn phase10_structure_e2e_claims_hold() {
        use vc_world::gen::{Biome, TerrainGen};
        use vc_world::world::Dimension;
        let gen = TerrainGen::for_dimension(0x10C0_C0DE, Dimension::Overworld);
        // "e2e: stronghold ring 1: 3 at dist ... (1280..2816 verified)"
        let sh = gen.strongholds();
        assert_eq!(sh.len(), 3, "ring 1 has 3 strongholds");
        for &(x, z) in &sh {
            let dist = ((x * x + z * z) as f32).sqrt();
            assert!((1280.0..=2816.0).contains(&dist), "band claim, got {dist}");
        }
        // "e2e: desert pyramid at ... (4 chests, desert_pyramid loot)"
        let mut pyr = None;
        'p: for rx in -8..8 {
            for rz in -8..8 {
                if let Some(c) = gen.pyramid_center_pub(rx, rz) {
                    pyr = Some(c);
                    break 'p;
                }
            }
        }
        let (wx, wz) = pyr.expect("a pyramid within ±8 regions");
        let (cx, cz) = (wx >> 4, wz >> 4);
        let (chunk, _) = gen.generate_chunk(cx, cz, Vec::new());
        let base = gen.column(wx, wz).height as i32;
        let floor = (base - 11) as usize;
        let mut chests = 0;
        for (dx, dz) in [(-1i32, -1i32), (1, -1), (-1, 1), (1, 1)] {
            let x = ((wx + dx) - cx * 16) as usize;
            let z = ((wz + dz) - cz * 16) as usize;
            // Chunk::get yields the raw state — route through state_block
            if vc_blocks::blocks::state_block(chunk.get(x, floor, z) as u16)
                == vc_blocks::blocks::CHEST
            {
                chests += 1;
            }
        }
        assert_eq!(chests, 4, "the 4-chest pit claim");
        assert!(
            vc_pack::datapack::builtin_structure_table("minecraft:chests/desert_pyramid").is_some(),
            "the desert_pyramid loot-table claim"
        );
        // "e2e: mineshaft at chunk ... corridors N (lens [24..=48])"
        let mut shaft = None;
        'm: for dcx in -10..10i32 {
            for dcz in -10..10i32 {
                let v = gen.mineshafts_near(dcx * 16, dcz * 16);
                if let Some(m) = v.first() {
                    shaft = Some(m.clone());
                    break 'm;
                }
            }
        }
        let ms = shaft.expect("a mineshaft within ±10 chunks");
        assert!(!ms.corridors.is_empty() && ms.corridors.len() <= 4);
        for &(_, _, len) in &ms.corridors {
            assert!((24..=48).contains(&len), "corridor lens claim");
        }
        // "14 biomes total (Phase 10: Taiga/Birch Forest/Jungle/Savanna/
        // Swamp/Badlands)" — from_u8 maps 0..=13 to distinct names
        let mut names: Vec<&str> = (0u8..=13).map(Biome::from_u8).map(|b| b.name()).collect();
        names.dedup();
        assert_eq!(names.len(), 14, "the 14-biomes-total claim");
    }
}

// ---------------------------------------------------- regression helpers --

/// True when the player's own chunk column is not generated yet — physics
/// must hold until it is (vanilla: entities in unloaded chunks do not tick).
/// REGRESSION guard for the user-reported fall-through-world bug:
/// `World::get_block` returns AIR for missing chunks, so gravity over an
/// unloaded column free-falls the player below y=0 where nothing can ever
/// collide again.
pub(crate) fn physics_frozen(world: &vc_world::world::World, pos: Vec3) -> bool {
    !world
        .chunks
        .contains_key(&(pos.x.div_euclid(16.0) as i32, pos.z.div_euclid(16.0) as i32))
}

/// Rolling min/max FPS from the frame-time history (ms). Max frame time
/// folds UP from 0.0 (→ the minimum FPS); min frame time folds DOWN from
/// INFINITY (→ the maximum FPS). REGRESSION guard for the F3 overlay bug
/// (the swapped initializers printed "max" as i32::MAX = 2147483547 fps).
pub(crate) fn fps_min_max(times: &std::collections::VecDeque<f32>) -> (f32, f32) {
    let max_ms = times.iter().cloned().fold(0.0_f32, f32::max);
    let min_ms = times.iter().cloned().fold(f32::INFINITY, f32::min);
    (1000.0 / max_ms, 1000.0 / min_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_freezes_until_own_chunk_exists() {
        let mut w = vc_world::world::World::new(7);
        let p = Vec3::new(8.5, 90.0, 8.5);
        assert!(
            physics_frozen(&w, p),
            "no chunk at (0,0) yet → the player must be held (fall-through-world guard)"
        );
        let c = std::sync::Arc::new(vc_chunk::chunk::Chunk::empty());
        w.insert_generated((0, 0), c, vec![]);
        assert!(
            !physics_frozen(&w, p),
            "chunk (0,0) present → physics is live"
        );
        assert!(
            physics_frozen(&w, Vec3::new(-0.5, 90.0, 8.5)),
            "x=-0.5 is chunk (-1,0) — still unloaded → frozen there"
        );
    }

    #[test]
    fn fps_min_max_orders_the_folds() {
        // 8 / 16 / 33 ms frames → slowest 33 ms = 30.3 fps min,
        // fastest 8 ms = 125 fps max
        let mut t = std::collections::VecDeque::new();
        t.push_back(8.0);
        t.push_back(16.0);
        t.push_back(33.0);
        let (lo, hi) = fps_min_max(&t);
        assert!(
            (lo - 1000.0 / 33.0).abs() < 0.01,
            "min FPS must come from the SLOWEST frame, got {lo}"
        );
        assert!(
            (hi - 1000.0 / 8.0).abs() < 0.01,
            "max FPS must come from the FASTEST frame, got {hi}"
        );
        // the old swapped-init bug: hi would be inf (→ i32::MAX in F3),
        // lo would be 0
        assert!(hi.is_finite() && hi < 1000.0);
        assert!(lo > 0.0);
    }
}
