//! GameApp: owns world/player/renderer/UI/audio; handles winit events
//! (native) and the JS input shim (wasm); screen flow:
//! Loading → Title ⇄ Options, Game ⇄ Pause/Options.
//! Streams chunks (rayon worker pool on native, time-budgeted inline on wasm).

use crate::blocks::*;
use crate::gen::Biome;
use crate::mesh::{mesh_sections, MeshData};
use crate::player::{raycast, Input, Player};
use crate::render::{Camera, RenderStats, Renderer, SkyState};
use crate::sounds::{AudioBackend, SoundBank};
#[cfg(not(target_arch = "wasm32"))]
use crate::sounds::native_audio;
#[cfg(target_arch = "wasm32")]
use crate::sounds::web_audio;
use crate::ui::{self, UiCanvas, Widget, WidgetKind, UI_H, UI_W};
use crate::world::{ChunkPos, World};
use glam::Vec3;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

// --------------------------------------------------------------- settings --

#[derive(Clone)]
pub struct Settings {
    pub render_distance: i32,
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
    /// FSR-lite internal render scale index: 0 = 100%, 1 = 75%, 2 = 50%
    pub upscale: u8,
    /// frame limiter: 0 = uncapped, else a fps ceiling (30/60/120)
    pub maxfps: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            #[cfg(target_arch = "wasm32")]
            render_distance: 6,
            #[cfg(not(target_arch = "wasm32"))]
            render_distance: 10,
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
            maxfps: 0,
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
            "rd={};sens={:.3};vol={:.3};fov={:.1};bright={:.3};smooth={};clouds={};graphics={};shader={};shadowq={};upscale={};maxfps={}",
            self.render_distance,
            self.sensitivity,
            self.volume,
            self.fov,
            self.brightness,
            self.smooth_lighting as u8,
            self.clouds as u8,
            self.graphics,
            self.shader,
            self.shadow_quality,
            self.upscale,
            self.maxfps
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
                "sens" => st.sensitivity = v.parse().unwrap_or(st.sensitivity).clamp(0.1, 2.0),
                "vol" => st.volume = v.parse().unwrap_or(st.volume).clamp(0.0, 1.0),
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
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Loading => "loading",
            Screen::Title => "title",
            Screen::Options => "options",
            Screen::Game => "game",
            Screen::Pause => "pause",
        }
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
    Gen { pos: ChunkPos, seed: u64, inbound: Vec<(u16, u8)> },
    Mesh {
        pos: ChunkPos,
        snap: [Option<Arc<crate::chunk::Chunk>>; 9],
        lsnap: [Option<Arc<crate::light::LightData>>; 9],
        smooth: bool,
        /// sections to rebuild (§12 bitset; 0xFFFF = full chunk)
        mask: u16,
        /// cached section meshes to reuse for unmasked sections
        prev: Vec<Option<Arc<MeshData>>>,
    },
}

enum JobResult {
    Gen { pos: ChunkPos, chunk: Arc<crate::chunk::Chunk>, outbound: Vec<(i32, i32, i32, u8)> },
    Mesh {
        pos: ChunkPos,
        /// the mask this job covered (dirty-bit clearing)
        mask: u16,
        /// new 16-slot section cache (fresh for masked, Arc clones for rest)
        sections: Vec<Option<Arc<MeshData>>>,
        /// merged per-chunk mesh for upload (§14 per-chunk merged buffers)
        mesh: Box<MeshData>,
    },
}

fn run_job(job: Job) -> JobResult {
    match job {
        Job::Gen { pos, seed, inbound } => {
            let gen = crate::gen::TerrainGen::new(seed);
            let (chunk, outbound) = gen.generate_chunk(pos.0, pos.1, inbound);
            JobResult::Gen { pos, chunk, outbound }
        }
        Job::Mesh { pos, snap, lsnap, smooth, mask, prev } => {
            let out = mesh_sections(pos, &snap, &lsnap, smooth, mask, &prev);
            JobResult::Mesh {
                pos,
                mask,
                sections: out.sections,
                mesh: Box::new(out.merged),
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
    light: crate::light::LightEngine,
    /// block particles (Phase 5 §16.2 pass 4)
    particles: crate::particles::ParticleSystem,
    /// billboard vertex scratch (rebuilt per frame against the camera basis)
    particle_verts: Vec<crate::particles::ParticleVertex>,
    input: Input,
    pub screen: Screen,
    options_from: Screen, // where Options was opened from
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
    picker_geom: Option<crate::ui::PickerGeom>,
    /// rolling frame times (ms) for the F3 frame-time graph
    frame_times: std::collections::VecDeque<f32>,
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
    /// pack-driven animated textures (frame updates only, no re-mesh)
    animations: Vec<crate::textures::AnimatedTile>,
    /// world save directory (native, §28 — browsers get OPFS later)
    #[cfg(not(target_arch = "wasm32"))]
    world_dir: std::path::PathBuf,
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
async fn load_builtin_pack_assets() -> (Vec<u8>, Vec<crate::textures::AnimatedTile>) {
    let mut atlas = crate::textures::generate_atlas();

    // 1. acquire the pack source
    #[cfg(not(target_arch = "wasm32"))]
    let source: Option<std::sync::Arc<dyn crate::pack::PackSource>> = {
        let folder = crate::pack::FolderSource::new("builtin-pack", "builtin");
        if folder.exists() {
            match crate::pack::open(std::sync::Arc::new(folder)) {
                Ok((meta, src)) => {
                    crate::render::report_boot_log(&format!(
                        "builtin pack: {} (format {}, {})",
                        src.name(),
                        meta.pack_format,
                        meta.description
                    ));
                    Some(src)
                }
                Err(e) => {
                    crate::render::report_boot_log(&format!("builtin pack unavailable: {e}"));
                    None
                }
            }
        } else {
            crate::render::report_boot_log("no builtin pack folder (builtin-pack/) — procedural fallback");
            None
        }
    };
    #[cfg(target_arch = "wasm32")]
    let source: Option<std::sync::Arc<dyn crate::pack::PackSource>> = {
        let specs: Vec<crate::model::BlockDispatchSpec> = crate::blocks::PROP_BLOCKS
            .iter()
            .map(|pb| crate::model::BlockDispatchSpec {
                name: pb.name,
                props: pb.props,
                base_state: pb.base_state,
                state_count: pb.state_count,
            })
            .collect();
        match crate::pack::fetch_builtin_pack(&specs).await {
            Some(mem) => match crate::pack::open(std::sync::Arc::new(mem)) {
                Ok((meta, src)) => {
                    crate::render::report_boot_log(&format!(
                        "builtin pack fetched: {} (format {})",
                        src.name(),
                        meta.pack_format
                    ));
                    Some(src)
                }
                Err(e) => {
                    crate::render::report_boot_log(&format!("builtin pack fetch failed: {e}"));
                    None
                }
            },
            None => {
                crate::render::report_boot_log("no builtin pack on server — procedural fallback");
                None
            }
        }
    };

    let Some(source) = source else {
        // no pack: still install an empty ModelSet so model-state blocks
        // render the missing texture instead of being skipped silently
        crate::model::install(crate::model::ModelSet {
            by_state: Default::default(),
            tiles: Default::default(),
        });
        return (atlas, Vec::new());
    };

    // 2. compile per-block dispatches (parse once, canonicalize, cache)
    let mut by_state = std::collections::HashMap::new();
    for pb in crate::blocks::PROP_BLOCKS.iter() {
        let spec = crate::model::BlockDispatchSpec {
            name: pb.name,
            props: pb.props,
            base_state: pb.base_state,
            state_count: pb.state_count,
        };
        match crate::model::compile_block_dispatch(&spec, &|p| source.read(p)) {
            Ok(map) => {
                by_state.extend(map);
            }
            Err(e) => {
                // §46: one bad blockstate must not take the engine down
                crate::render::report_boot_log(&format!(
                    "blockstate {name} failed: {e} — block will use the missing model",
                    name = pb.name
                ));
            }
        }
    }
    let mut set = crate::model::ModelSet {
        by_state,
        tiles: Default::default(),
    };

    // 3. merge pack textures into the atlas (fills set.tiles + animations)
    let animations = crate::textures::merge_pack_textures(&mut atlas, &mut set, source.as_ref());
    let n_models: usize = set.by_state.values().map(|v| v.len()).sum();
    crate::render::report_boot_log(&format!(
        "model dispatch: {} states, {} applied models, {} pack textures, {} animations",
        set.by_state.len(),
        n_models,
        set.tiles.len(),
        animations.len()
    ));
    crate::model::install(set);
    (atlas, animations)
}

impl GameApp {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        // ---------------------------------------------------- Phase 1 assets
        // Compile the builtin resource pack (blockstates → models → textures)
        // BEFORE any mesh job can run; merge its textures into the atlas.
        let (mut atlas, animations) = crate::game::load_builtin_pack_assets().await;
        crate::textures::draw_missing_tile(&mut atlas);

        let mut renderer = Renderer::new(window, &atlas).await;
        let bank = SoundBank::generate();
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut world = World::new(crate::world::World::random_seed());
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut spawn = world.find_spawn();
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));

        // native: restore a persisted world (seed, spawn, player) if one
        // exists in the save dir (§28) — otherwise a fresh random world
        #[cfg(not(target_arch = "wasm32"))]
        let world_dir = crate::save::default_world_dir();
        #[cfg(not(target_arch = "wasm32"))]
        let mut level_spawn = (spawn.0 as i32, spawn.1 as i32, spawn.2 as i32);
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(Some(meta)) = crate::save::read_level_dat(&world_dir) {
            world = World::new(meta.seed);
            spawn = world.find_spawn();
            level_spawn = meta.spawn;
            if let Some(p) = &meta.player {
                player = Player::new(Vec3::new(
                    p.pos[0] as f32,
                    p.pos[1] as f32,
                    p.pos[2] as f32,
                ));
                player.yaw = p.yaw;
                player.pitch = p.pitch;
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

        // apply persisted render scale (FSR-lite) before the first frame
        renderer.set_upscale(settings.upscale_factor());
        // §17: apply persisted shadow quality
        renderer.set_shadow_quality(settings.shadow_map_px());

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
                    None => Box::new(crate::sounds::SilentOut),
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
            audio,
            settings,
            work,
            gen_inflight: HashSet::new(),
            mesh_inflight: HashMap::new(),
            section_meshes: HashMap::new(),
            light: crate::light::LightEngine::new(),
            particles: crate::particles::ParticleSystem::new(0x5EED_0042),
            particle_verts: Vec::new(),
            input: Input::default(),
            screen: Screen::Loading,
            options_from: Screen::Title,
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
            world_dir,
            #[cfg(not(target_arch = "wasm32"))]
            level_spawn,
            #[cfg(not(target_arch = "wasm32"))]
            autosave_in: 20.0,
        };
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
        use winit::event::{ElementState, MouseButton, MouseScrollDelta};
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
                    if self.picker_open && self.screen == Screen::Game {
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
                    if let Some(kc) = keycode_from_web(&code) {
                        self.key_action(kc, pressed, repeat);
                    }
                }
                WebEvent::MouseDelta { dx, dy } => {
                    if self.screen == Screen::Game && !self.picker_open {
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
                    if self.picker_open && self.screen == Screen::Game {
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
                    } else if self.screen == Screen::Game && !self.picker_open {
                        // browser released the lock (Esc) → pause menu
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
        // movement only when actually in the game world (not in the picker)
        let in_game = self.screen == Screen::Game && !self.picker_open;
        match code {
            KeyCode::KeyW => self.input.fwd = pressed && in_game,
            KeyCode::KeyS => self.input.back = pressed && in_game,
            KeyCode::KeyA => self.input.left = pressed && in_game,
            KeyCode::KeyD => self.input.right = pressed && in_game,
            KeyCode::Space => {
                self.input.jump = pressed && in_game;
                if pressed && in_game {
                    self.player.try_fly_toggle(self.time);
                }
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.input.sneak = pressed && in_game,
            KeyCode::ControlLeft | KeyCode::ControlRight => self.input.sprint = pressed && in_game,
            KeyCode::Escape => {
                if pressed {
                    match self.screen {
                        Screen::Game => {
                            if self.picker_open {
                                self.close_picker();
                            } else {
                                self.enter_pause();
                            }
                        }
                        Screen::Pause => self.resume_game(),
                        Screen::Options => self.close_options(),
                        _ => {}
                    }
                }
            }
            KeyCode::KeyE => {
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
                    let b = self.player.hotbar[n as usize];
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
                }
                self.input.break_hold = pressed;
            }
            MouseButton::Right => self.input.place_hold = pressed,
            MouseButton::Middle => {
                if pressed {
                    if let Some((_, b, _)) = self.target {
                        if let Some(slot) = self.player.hotbar.iter().position(|&h| h == b) {
                            self.player.selected = slot;
                        } else {
                            self.player.hotbar[self.player.selected] = b;
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
            self.player.hotbar[self.player.selected] = b;
            self.item_toast = Some((name(b).to_string(), 2.0));
            self.ui.dirty = true;
        }
    }

    /// mouse buttons while in a menu (buttons + sliders)
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
        let n = self.player.hotbar.len() as i32;
        let cur = self.player.selected as i32;
        let next = ((cur - d.signum() as i32).rem_euclid(n)) as usize;
        self.player.selected = next;
        let b = self.player.hotbar[next];
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
        self.audio.play(&self.bank, SoundFamily::Wood, 0.30 * self.settings.volume, 1.62);
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
            if !self.drag_look {
                // works when called from a click (transient activation)
                crate::web_input::request_pointer_lock();
            }
        }
    }

    fn open_options(&mut self, from: Screen) {
        self.options_from = from;
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
        crate::render::report_boot_log(&format!(
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

    // ------------------------------------------------------ menu actions --

    fn activate(&mut self, id: u16) {
        use ui::*;
        match id {
            ID_TITLE_PLAY => self.start_game(),
            ID_TITLE_OPTIONS => self.open_options(Screen::Title),
            ID_TITLE_QUIT => self.quit_requested = true,
            ID_OPT_DONE => self.close_options(),
            ID_PAUSE_BACK => self.resume_game(),
            ID_PAUSE_OPTIONS => self.open_options(Screen::Pause),
            ID_PAUSE_QUIT => self.quit_to_title(),
            ID_OPT_SHADER => {
                self.settings.shader = (self.settings.shader + 1) % 3;
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
            ID_OPT_BRIGHT => self.settings.brightness = t,
            ID_OPT_VOL => self.settings.volume = t,
            _ => {}
        }
        self.after_settings_change();
    }

    /// persist + refresh widget labels + player fov
    fn after_settings_change(&mut self) {
        self.player.fov = self.settings.fov.to_radians();
        #[cfg(target_arch = "wasm32")]
        crate::web_input::save_settings(&self.settings.serialize());
        self.refresh_widgets();
        self.ui.dirty = true;
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
                        ID_OPT_SHADER => set_button_value(w, ["OFF", "VANILLA+", "CINEMATIC"][s.shader as usize]),
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
                        ID_OPT_UPSCALE => set_button_value(w, match s.upscale { 1 => "75%", 2 => "50%", _ => "OFF" }),
                        ID_OPT_MAXFPS => set_button_value(w, match s.maxfps { 1 => "30", 2 => "60", 3 => "120", _ => "VSYNC" }),
                        ID_OPT_SMOOTH => set_button_value(w, if s.smooth_lighting { "ON" } else { "OFF" }),
                        ID_OPT_CLOUDS => set_button_value(w, if s.clouds { "ON" } else { "OFF" }),
                        _ => {}
                    }
                }
                self.widgets = ws;
            }
            _ => self.widgets = Vec::new(),
        }
    }

    fn remesh_all(&mut self) {
        let positions: Vec<ChunkPos> = self.renderer.chunks.keys().copied().collect();
        for p in positions {
            self.world.mark_all_dirty(p, crate::world::CAUSE_GEOMETRY | crate::world::CAUSE_LIGHT);
        }
        // cached section meshes embed the old baking (e.g. smooth-lighting AO)
        self.section_meshes.clear();
        self.renderer.clear_meshes();
    }

    // ------------------------------------------------------------ update --

    /// E2E hook: break a block at world coords through the full interactive
    /// path (state edit → light update → fence re-link → particles →
    /// invalidation), without requiring pointer lock / raycast targeting.
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
                .mark_sections_dirty(pos, mask, crate::world::CAUSE_LIGHT);
        }

        // stream chunks (also during title/menus: the panorama keeps loading)
        crate::phase!(self.phases, crate::bench::PHASE_STREAM, self.stream());

        // particles: fixed 20 Hz sim against the live world (§16.2 pass 4)
        crate::phase!(self.phases, crate::bench::PHASE_SIM, {
            self.particles.update(dt, &self.world);
        });

        // E2E test commands (wasm only): "break:x:y:z" → same path as an
        // interactive break (mesh invalidation + light + particles)
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(cmd) = crate::web_input::pop_test_cmd() {
                if let Some(rest) = cmd.strip_prefix("break:") {
                    let p: Vec<i32> = rest.split(':').filter_map(|v| v.parse().ok()).collect();
                    if p.len() == 3 {
                        self.test_break(p[0], p[1], p[2]);
                    }
                }
            }
        }

        // loading → wait for spawn chunk, then snap to surface → title screen
        if self.screen == Screen::Loading {
            let pc = self.player_chunk();
            if self.renderer.has_chunk(pc) {
                if !self.spawn_snapped {
                    let lx = (self.player.pos.x - pc.0 as f32 * 16.0).floor() as usize;
                    let lz = (self.player.pos.z - pc.1 as f32 * 16.0).floor() as usize;
                    if let Some(c) = self.world.chunk(pc) {
                        let top = c.top_solid_y(lx.min(15), lz.min(15));
                        if top >= 0 {
                            self.player.pos.y = top as f32 + 1.0;
                            self.player.flying = false;
                        }
                    }
                    self.spawn_snapped = true;
                }
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
                    self.set_screen(Screen::Title);
                }
            }
        }

        let in_game = self.screen == Screen::Game;

        // player physics
        let t_sim = crate::bench::micros();
        if in_game {
            let sounds = self.player.update(
                dt,
                self.time,
                &self.world,
                &mut self.input,
                self.settings.sensitivity,
                true,
            );
            for s in sounds {
                self.audio.play(&self.bank, s.family, s.volume * self.settings.volume, s.pitch);
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
                        self.audio.play(
                            &self.bank,
                            def(b).sound,
                            0.55 * self.settings.volume,
                            0.95 + (self.time * 7.13).sin().fract().abs() * 0.15,
                        );
                        self.break_timer = 0.24;
                        self.edits += 1;
                    }
                }
            }
            if self.input.place_hold && self.place_timer <= 0.0 {
                if let Some((tpos, _, prev)) = self.target {
                    let b = self.player.hotbar[self.player.selected];
                    if b != AIR {
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
                                b as u16
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
                            self.audio.play(&self.bank, def(b).sound, 0.55 * self.settings.volume, 1.15);
                            self.place_timer = 0.24;
                            self.edits += 1;
                        }
                    }
                }
            }
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
            let updates = crate::textures::tick_animations(&mut self.animations, dt);
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
        let live_debug = self.screen == Screen::Game && self.show_debug;
        let cadence = if self.screen == Screen::Game
            && !self.picker_open
            && !live_debug
        {
            0.15
        } else {
            0.05
        };
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
        let entries: Vec<(ChunkPos, Arc<crate::chunk::Chunk>)> = dirty
            .into_iter()
            .filter_map(|p| self.world.chunks.get(&p).map(|c| (p, Arc::clone(c))))
            .collect();
        let tick = ((self.time - self.load_start) * 20.0).max(0.0) as i64;
        if !entries.is_empty() {
            let refs: Vec<(i32, i32, &crate::chunk::Chunk, Option<&Arc<crate::light::LightData>>)> =
                entries
                    .iter()
                    .map(|(p, c)| (p.0, p.1, c.as_ref(), self.world.light.get(p)))
                    .collect();
            if let Err(e) = crate::save::store_chunks(&self.world_dir, &refs, tick) {
                crate::render::report_boot_log(&format!("autosave failed: {e}"));
            }
        }
        let meta = crate::save::WorldMeta {
            seed: self.world.seed,
            name: "VoxelCraft".into(),
            spawn: self.level_spawn,
            player: Some(crate::save::PlayerMeta {
                pos: [self.player.pos.x as f64, self.player.pos.y as f64, self.player.pos.z as f64],
                yaw: self.player.yaw,
                pitch: self.player.pitch,
            }),
            game_time: tick,
        };
        if let Err(e) = crate::save::write_level_dat(&self.world_dir, &meta) {
            crate::render::report_boot_log(&format!("level.dat write failed: {e}"));
        }
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
                crate::model::models().map(|m| m.by_state.len() as f32).unwrap_or(0.0)
            )),
            ("packTextures", StatsVal::F(
                crate::model::models().map(|m| m.tiles.len() as f32).unwrap_or(0.0)
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
                crate::save::load_chunk(&self.world_dir, pos.0, pos.1)
            {
                let inbound = self.world.take_pending(pos);
                let edited = !inbound.is_empty();
                for (idx, id) in inbound {
                    chunk.set_idx(idx as usize, id);
                }
                self.world.insert_generated(pos, Arc::new(chunk), Vec::new());
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
                                crate::world::CAUSE_LIGHT,
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
            let job = Job::Gen { pos, seed: self.world.seed, inbound };
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
        let max_mesh = if cfg!(target_arch = "wasm32") { 2 } else { 16 };
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
                self.submit(Job::Mesh {
                    pos,
                    snap,
                    lsnap,
                    smooth: self.settings.smooth_lighting,
                    mask,
                    prev,
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

    fn apply_result(&mut self, res: JobResult) {
        match res {
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
                self.world.insert_generated(pos, chunk, outbound);
                // Phase 4: initial lighting for the new chunk (column scan +
                // border exchange, settled synchronously) — the engine's
                // changed map feeds precise §12 dirty bits below
                self.light.init_chunk(&mut self.world, pos);
                for (lpos, lmask) in self.light.take_changed() {
                    self.world
                        .mark_sections_dirty(lpos, lmask, crate::world::CAUSE_LIGHT);
                }
                // the new chunk changes border face culling in its 8
                // neighbors — mark the sections whose y-bands touch the new
                // chunk's non-air border cells (NOT all 16: §12)
                let bands = neighbor_geometry_bands(&self.world, pos);
                for (npos, band) in bands {
                    self.world
                        .mark_sections_dirty(npos, band, crate::world::CAUSE_GEOMETRY);
                }
            }
            JobResult::Mesh { pos, mask, sections, mesh } => {
                self.mesh_inflight.remove(&pos);
                // clear only the bits this job covered — edits that arrived
                // after its snapshot re-queue the chunk (§12)
                self.world.clear_dirty_mask(pos, mask);
                self.section_meshes.insert(pos, sections);
                self.renderer.set_chunk_mesh(pos, &mesh);
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
                let sub = "SETTINGS APPLY INSTANTLY AND ARE SAVED";
                self.ui.options_screen(&self.widgets, self.hover, sub);
                return;
            }
            Screen::Pause => {
                self.ui.pause_screen(&self.widgets, self.hover);
                return;
            }
            Screen::Game => {}
        }

        // in-game HUD
        self.ui.crosshair();
        let toast = self.item_toast.as_ref().map(|(s, t)| (s.as_str(), (*t * 200.0).clamp(0.0, 220.0) as u8));
        self.ui.hotbar(&self.player.hotbar, self.player.selected, &self.atlas, toast);
        let xp = (self.edits % 50) as f32 / 50.0;
        let level = self.edits / 50;
        self.ui.status_bars(20.0, 20.0, xp, level);

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
                format!("XYZ: {:.2} / {:.2} / {:.2}", p.pos.x, p.pos.y, p.pos.z),
                format!("Block: {} {} {}  ({})", p.pos.x as i32, p.pos.y as i32, p.pos.z as i32, ""),
                format!("Chunk: {} {}  Facing: {}  Light: sky {}",
                    pc.0, pc.1, facing,
                    "-"),
                format!("Biome: {}", biome),
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
                    "Graphics: {}  Shadows: {}  Upscale: {:.0}%  MaxFPS: {}",
                    ["fast", "fancy", "fabulous"][self.settings.graphics as usize],
                    match self.settings.shadow_quality {
                        0 => "off",
                        1 => "1K",
                        2 => "2K",
                        _ => "4K",
                    },
                    self.settings.upscale_factor() * 100.0,
                    match self.settings.maxfps { 0 => "vsync", 1 => "30", 2 => "60", _ => "120" }
                ),
                format!(
                    "Dirty: {} sections in {} chunks (§12 fine-grained invalidation)",
                    self.world.dirty_section_count(),
                    self.world.dirty.len()
                ),
                format!(
                    "Shader: {}  Clouds: {}  Smooth: {}  VSync: {}",
                    ["off", "vanilla+", "cinematic"][self.settings.shader as usize],
                    if self.settings.clouds { "on" } else { "off" },
                    if self.settings.smooth_lighting { "on" } else { "off" },
                    if self.renderer.vsync { "on" } else { "off" }
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

        // block picker overlay (E) — sits above the HUD
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
            self.fps_min = 1000.0 / self.frame_times.iter().cloned().fold(f32::INFINITY, f32::max);
            self.fps_max = 1000.0 / self.frame_times.iter().cloned().fold(0.0, f32::min);
        }
        // Only log the first frames of each actual game instance — the FPS
        // window counter (`self.frames`) resets every 0.5 s, so without the
        // time gate this spams "frame #1..#3" twice a second and makes
        // remounts impossible to distinguish from normal windows.
        if self.frames < 3 && self.time_since_load() < 2.0 {
            crate::render::report_boot_log(&format!(
                "draw() frame #{}: chunks_gpu={}, screen={:?}",
                self.frames + 1,
                self.renderer.chunks.len(),
                self.screen
            ));
        }
        // day/night state
        let theta = self.day_time * std::f32::consts::TAU;
        let sun_dir = Vec3::new(theta.cos() * 0.85, theta.sin(), -0.4).normalize();
        let day_light = 0.16 + 0.84 * smoothstep(-0.10, 0.14, sun_dir.y);

        let sunset = (1.0 - (sun_dir.y * 4.0).abs()).clamp(0.0, 1.0) * (day_light.clamp(0.2, 0.8) - 0.2) / 0.6;
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

        let rd = self.settings.render_distance;
        let (fog_start, fog_end, fog_col) = if self.player.head_in_water && self.screen == Screen::Game {
            (2.0, 28.0, [0.11, 0.22, 0.45])
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
        }

        self.stats = self.renderer.render(
            &cam,
            &sky,
            &mut self.ui,
            selection,
            &crate::render::PostParams {
                mode: self.settings.shader,
                menu_blur,
                shadows: self.settings.shadow_strength(),
                sharpen: if self.settings.upscale > 0 { 0.35 } else { 0.0 },
            },
            self.settings.clouds && self.settings.graphics >= 1,
            &self.particle_verts,
        );
        self.phases
            .add(crate::bench::PHASE_DRAW, crate::bench::micros() - t_draw0);

        // --- bench bookkeeping: count measured frames, finish + exit (§37)
        if let Some(bs) = self.bench.as_mut() {
            if self.screen == Screen::Game {
                bs.seen += 1;
                if bs.seen == bs.warmup + 1 {
                    crate::render::report_boot_log("benchmark: warmup done, measuring");
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
                        let json = format!(
                            "{{\"benchmark\":{{\"frame\":{},\"phases\":{}}}}}",
                            fs.to_json(),
                            report.to_json()
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

/// biome + (sky, block) light levels at a world position — for baking
/// particle tint/brightness at spawn (Phase 5)
fn light_at(world: &World, light: &crate::light::LightEngine, wx: i32, wy: i32, wz: i32) -> (u8, u8, u8) {
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
        "KeyE" => KeyCode::KeyE,
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

fn set_button_value(w: &mut Widget, value: &str) {
    if let WidgetKind::Button { value: v, .. } = &mut w.kind {
        *v = value.to_string();
    }
}
