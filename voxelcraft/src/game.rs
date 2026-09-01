//! GameApp: owns world/player/renderer/UI/audio; handles winit events;
//! streams chunks (rayon worker pool on native, time-budgeted inline on wasm).

use crate::blocks::*;
use crate::gen::Biome;
use crate::mesh::{mesh_chunk, MeshData};
use crate::player::{raycast, Input, Player};
use crate::render::{Camera, RenderStats, Renderer, SkyState};
use crate::sounds::{AudioBackend, SoundBank};
#[cfg(not(target_arch = "wasm32"))]
use crate::sounds::native_audio;
#[cfg(target_arch = "wasm32")]
use crate::sounds::web_audio;
use crate::ui::UiCanvas;
use crate::world::{ChunkPos, World};
use glam::Vec3;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

pub struct Settings {
    pub render_distance: i32,
    pub sensitivity: f32,
    pub volume: f32,
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
        }
    }
}

// ------------------------------------------------------------------ jobs --

enum Job {
    Gen { pos: ChunkPos, seed: u64, inbound: Vec<(u16, u8)> },
    Mesh { pos: ChunkPos, snap: [Option<Arc<crate::chunk::Chunk>>; 9] },
}

enum JobResult {
    Gen { pos: ChunkPos, chunk: Arc<crate::chunk::Chunk>, outbound: Vec<(i32, i32, i32, u8)> },
    Mesh { pos: ChunkPos, mesh: Box<MeshData> },
}

fn run_job(job: Job) -> JobResult {
    match job {
        Job::Gen { pos, seed, inbound } => {
            let gen = crate::gen::TerrainGen::new(seed);
            let (chunk, outbound) = gen.generate_chunk(pos.0, pos.1, inbound);
            JobResult::Gen { pos, chunk, outbound }
        }
        Job::Mesh { pos, snap } => {
            let mesh = mesh_chunk(pos, &snap);
            JobResult::Mesh { pos, mesh: Box::new(mesh) }
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
    mesh_inflight: HashSet<ChunkPos>,
    input: Input,
    pub paused: bool,
    pub loading: bool,
    audio_unlocked: bool,
    day_time: f32,
    time: f32,
    target: Option<([i32; 3], u8, [i32; 3])>,
    break_timer: f32,
    place_timer: f32,
    show_debug: bool,
    show_help: bool,
    item_toast: Option<(String, f32)>,
    last_ui_t: f32,
    last_frame_t: f32,
    fps: f32,
    frames: u32,
    fps_t: f32,
    stats: RenderStats,
    spawn_snapped: bool,
    load_start: f32,
}

pub fn now_secs() -> f32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f32())
            .unwrap_or(0.0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as f32
    }
}

impl GameApp {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        let atlas = crate::textures::generate_atlas();
        let renderer = Renderer::new(window, &atlas).await;
        let bank = SoundBank::generate();
        let world = World::new(crate::world::World::random_seed());
        let spawn = world.find_spawn();
        let player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));

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
            settings: Settings::default(),
            work,
            gen_inflight: HashSet::new(),
            mesh_inflight: HashSet::new(),
            input: Input::default(),
            paused: false,
            loading: true,
            audio_unlocked: false,
            day_time: 0.30,
            time: now_secs(),
            target: None,
            break_timer: 0.0,
            place_timer: 0.0,
            show_debug: false,
            show_help: false,
            item_toast: None,
            last_ui_t: -1.0,
            last_frame_t: now_secs(),
            fps: 0.0,
            frames: 0,
            fps_t: now_secs(),
            stats: RenderStats::default(),
            spawn_snapped: false,
            load_start: 0.0,
        };
        app.load_start = app.time;
        app
    }

    // ------------------------------------------------------------- events --

    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
        use winit::keyboard::{KeyCode, PhysicalKey};

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    self.renderer.resize(size.width, size.height);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    let code = match event.physical_key {
                        PhysicalKey::Code(c) => c,
                        _ => return,
                    };
                    match code {
                        KeyCode::KeyW => self.input.fwd = pressed,
                        KeyCode::KeyS => self.input.back = pressed,
                        KeyCode::KeyA => self.input.left = pressed,
                        KeyCode::KeyD => self.input.right = pressed,
                        KeyCode::Space => {
                            self.input.jump = pressed;
                            if pressed && !self.paused && !self.loading {
                                self.player.try_fly_toggle(self.time);
                            }
                        }
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => self.input.sneak = pressed,
                        KeyCode::ControlLeft | KeyCode::ControlRight => self.input.sprint = pressed,
                        KeyCode::Escape => {
                            if pressed {
                                self.set_paused(!self.paused);
                            }
                        }
                        KeyCode::F3 => {
                            if pressed {
                                self.show_debug = !self.show_debug;
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::KeyH => {
                            if pressed {
                                self.show_help = !self.show_help;
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::BracketLeft => {
                            if pressed && self.settings.render_distance > 3 {
                                self.settings.render_distance -= 1;
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::BracketRight => {
                            if pressed && self.settings.render_distance < 16 {
                                self.settings.render_distance += 1;
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::Minus => {
                            if pressed {
                                self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::Equal => {
                            if pressed {
                                self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::KeyV => {
                            if pressed {
                                self.renderer.toggle_vsync();
                                self.ui.dirty = true;
                            }
                        }
                        KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4
                        | KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::Digit7
                        | KeyCode::Digit8 | KeyCode::Digit9 => {
                            if pressed {
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
                WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => {
                            if pressed {
                                if self.paused {
                                    self.set_paused(false);
                                }
                                // ensure pointer lock (wasm needs a user gesture)
                                if !self.loading {
                                    let _ = self.window
                                        .set_cursor_grab(winit::window::CursorGrabMode::Locked);
                                    self.window.set_cursor_visible(false);
                                }
                                if !self.audio_unlocked {
                                    self.audio.unlock(&self.bank);
                                    self.audio_unlocked = true;
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
                WindowEvent::MouseWheel { delta, .. } => {
                    let d = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                    };
                    if d.abs() > 0.01 {
                        let n = self.player.hotbar.len() as i32;
                        let cur = self.player.selected as i32;
                        let next = ((cur - d.signum() as i32).rem_euclid(n)) as usize;
                        self.player.selected = next;
                        let b = self.player.hotbar[next];
                        self.item_toast = Some((name(b).to_string(), 2.0));
                        self.ui.dirty = true;
                    }
                }
                WindowEvent::RedrawRequested => {
                    self.draw();
                }
                WindowEvent::Focused(false) => {
                    self.input = Input::default();
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                    if !self.paused {
                        self.input.add_mouse(dx as f32, dy as f32);
                    }
                }
            }
            Event::AboutToWait => {
                let now = now_secs();
                let dt = (now - self.last_frame_t).clamp(0.0, 0.1);
                self.last_frame_t = now;
                self.update(dt);
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        self.input = Input::default();
        let _ = self.window.set_cursor_grab(if paused {
            winit::window::CursorGrabMode::None
        } else {
            winit::window::CursorGrabMode::Locked
        });
        self.window.set_cursor_visible(paused);
        self.ui.dirty = true;
    }

    // ------------------------------------------------------------ update --

    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.day_time = (self.day_time + dt / 600.0) % 1.0; // 10-minute day

        // stream chunks
        self.stream();

        // loading → wait for spawn chunk, then snap to surface
        if self.loading {
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
                // small delay so surroundings exist
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
                    self.loading = false;
                    self.set_paused(false);
                }
            }
        }

        let loaded = !self.loading;

        // player physics
        if !self.paused && loaded {
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
                        self.world.set_block(pos[0], pos[1], pos[2], AIR);
                        self.audio.play(
                            &self.bank,
                            def(b).sound,
                            0.55 * self.settings.volume,
                            0.95 + (self.time * 7.13).sin().fract().abs() * 0.15,
                        );
                        self.break_timer = 0.24;
                    }
                }
            }
            if self.input.place_hold && self.place_timer <= 0.0 {
                if let Some((_, _, prev)) = self.target {
                    let b = self.player.hotbar[self.player.selected];
                    if b != AIR {
                        let pb = self.world.get_block(prev[0], prev[1], prev[2]);
                        let replaceable = pb == AIR || pb == WATER || is_cross(pb);
                        let collides_player = is_solid(b) && self.player.block_intersects_player(prev);
                        if replaceable && !collides_player {
                            self.world.set_block(prev[0], prev[1], prev[2], b);
                            self.audio.play(
                                &self.bank,
                                def(b).sound,
                                0.55 * self.settings.volume,
                                1.15,
                            );
                            self.place_timer = 0.24;
                        }
                    }
                }
            }
        } else if self.loading {
            // gentle fall while streaming
            self.player.update(dt, self.time, &self.world, &mut Input::default(), self.settings.sensitivity, false);
        }

        // toasts
        if let Some((_, t)) = self.item_toast.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.item_toast = None;
                self.ui.dirty = true;
            }
        }

        // fps
        self.frames += 1;
        if self.time - self.fps_t > 0.5 {
            self.fps = self.frames as f32 / (self.time - self.fps_t);
            self.frames = 0;
            self.fps_t = self.time;
            self.ui.dirty = true;
        }

        // UI rebuild every 150ms max (plus dirty events)
        if self.ui.dirty && self.time - self.last_ui_t > 0.15 {
            self.rebuild_ui();
        }
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
            let inbound = self.world.take_pending(pos);
            self.gen_inflight.insert(pos);
            let job = Job::Gen { pos, seed: self.world.seed, inbound };
            self.submit(job);
        }

        // 3. queue mesh jobs (radius rd, nearest first, dirty first)
        let mut want_mesh: Vec<(ChunkPos, bool)> = Vec::new();
        for dz in -rd..=rd {
            for dx in -rd..=rd {
                let pos = (pc.0 + dx, pc.1 + dz);
                if self.mesh_inflight.contains(&pos) {
                    continue;
                }
                let dirty = self.world.dirty.contains(&pos);
                let meshed = self.renderer.has_chunk(pos);
                if (dirty || !meshed) && self.world.meshable(pos.0, pos.1) {
                    want_mesh.push((pos, dirty));
                }
            }
        }
        want_mesh.sort_by(|a, b| {
            let da = (a.0 .0 - pc.0).abs() + (a.0 .1 - pc.1).abs();
            let db = (b.0 .0 - pc.0).abs() + (b.0 .1 - pc.1).abs();
            b.1.cmp(&a.1).then(da.cmp(&db)) // dirty chunks first
        });
        let max_mesh = if cfg!(target_arch = "wasm32") { 2 } else { 16 };
        for (pos, _) in want_mesh.into_iter().take(max_mesh) {
            if let Some(snap) = self.world.snapshot3x3(pos.0, pos.1) {
                self.mesh_inflight.insert(pos);
                self.submit(Job::Mesh { pos, snap });
            }
        }

        // 4. unload far GPU meshes
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
            self.world.dirty.remove(&pos);
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
                // fold in any pending edits that arrived while generating
                let leftover = self.world.pending.remove(&pos).unwrap_or_default();
                let chunk = if leftover.is_empty() {
                    chunk
                } else {
                    let mut c = (*chunk).clone();
                    for (idx, id) in leftover {
                        if c.blocks[idx as usize] == AIR {
                            c.blocks[idx as usize] = id;
                        }
                    }
                    Arc::new(c)
                };
                self.world.insert_generated(pos, chunk, outbound);
                // neighbors may now be meshable — drop their stale meshes if any
                for dz in -1..=1 {
                    for dx in -1..=1 {
                        let np = (pos.0 + dx, pos.1 + dz);
                        if dx != 0 || dz != 0 {
                            if !self.mesh_inflight.contains(&np) {
                                self.world.dirty.insert(np);
                            }
                        }
                    }
                }
            }
            JobResult::Mesh { pos, mesh } => {
                self.mesh_inflight.remove(&pos);
                self.world.dirty.remove(&pos);
                self.renderer.set_chunk_mesh(pos, &mesh);
            }
        }
    }

    // ---------------------------------------------------------------- ui --

    fn rebuild_ui(&mut self) {
        self.last_ui_t = self.time;
        self.ui.clear();

        if self.loading {
            let pc = self.player_chunk();
            let need = ((self.settings.render_distance * 2 + 1).max(1)) as f32;
            let mut have = 0.0;
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if self.renderer.has_chunk((pc.0 + dx, pc.1 + dz)) {
                        have += 1.0;
                    }
                }
            }
            let progress = (have / 9.0_f32).min(1.0);
            let _ = need;
            self.ui.vignette_loading("Building terrain...", progress);
            return;
        }

        self.ui.crosshair();
        let toast = self.item_toast.as_ref().map(|(s, t)| (s.as_str(), (*t * 200.0).clamp(0.0, 220.0) as u8));
        self.ui.hotbar(&self.player.hotbar, self.player.selected, &self.atlas, toast);

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
                format!("VOXELCRAFT (Rust + wgpu) {} fps", self.fps as i32),
                format!(
                    "XYZ: {:.1} / {:.1} / {:.1}",
                    p.pos.x, p.pos.y, p.pos.z
                ),
                format!("Chunk: {} {}  Facing: {}", pc.0, pc.1, facing),
                format!(
                    "Chunks: {} drawn / {} loaded  Tris: {}",
                    self.stats.chunks,
                    self.world.chunks.len(),
                    self.stats.tris
                ),
                format!("Biome: {}", biome),
                format!(
                    "Day cycle: {:.0}%  Fly: {}",
                    self.day_time * 100.0,
                    if self.player.flying { "on" } else { "off" }
                ),
                format!(
                    "Render dist: {}  Vol: {:.0}%  VSync: {}",
                    self.settings.render_distance,
                    self.settings.volume * 100.0,
                    if self.renderer.vsync { "on" } else { "off" }
                ),
                format!("Seed: {}", self.world.seed),
            ];
            self.ui.debug(&lines);
        }

        if self.show_help {
            self.ui.help();
        }

        if self.paused {
            self.ui.center_msg("PAUSED", "Click to play  -  ESC to toggle  -  H for controls");
        }
    }

    // -------------------------------------------------------------- draw --

    fn draw(&mut self) {
        // first-frames instrumentation (diagnoses event-loop / pipeline stalls)
        if self.frames < 3 {
            crate::render::report_boot_log(&format!(
                "draw() frame #{}: chunks_gpu={}, meshes drawn from {}",
                self.frames + 1,
                self.renderer.chunks.len(),
                0
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
        let (fog_start, fog_end, fog_col) = if self.player.head_in_water {
            (2.0, 28.0, [0.11, 0.22, 0.45])
        } else {
            let end = (rd * 16 - 12) as f32;
            (end * 0.55, end, fog)
        };

        let cam = Camera {
            eye: self.player.eye(),
            yaw: self.player.yaw,
            pitch: self.player.pitch,
            fov: self.player.fov_cur,
        };
        let sky = SkyState {
            day_light,
            sun_dir,
            fog_color: fog_col,
            fog_start,
            fog_end,
            time: self.time,
            underwater: self.player.head_in_water,
        };

        let selection = self.target.map(|(pos, _, _)| (pos[0], pos[1], pos[2]));
        self.stats = self.renderer.render(&cam, &sky, &mut self.ui, selection);
    }
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
