//! GameApp: owns world/player/renderer/UI/audio; handles winit events
//! (native) and the JS input shim (wasm); screen flow:
//! Loading → Title ⇄ Options, Game ⇄ Pause/Options.
//! Streams chunks (rayon worker pool on native, time-budgeted inline on wasm).

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
use crate::ui::{self, UiCanvas, Widget, WidgetKind, UI_H, UI_W};
use crate::world::{ChunkPos, World};
use glam::Vec3;
use std::collections::{HashSet, VecDeque};
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
    pub fancy: bool,
    pub shader: u8, // 0 = off, 1 = vanilla+, 2 = cinematic
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
            fancy: true,
            shader: 1,
        }
    }
}

impl Settings {
    /// serialize as k=v; pairs (parsed without serde)
    pub fn serialize(&self) -> String {
        format!(
            "rd={};sens={:.3};vol={:.3};fov={:.1};bright={:.3};smooth={};clouds={};fancy={};shader={}",
            self.render_distance,
            self.sensitivity,
            self.volume,
            self.fov,
            self.brightness,
            self.smooth_lighting as u8,
            self.clouds as u8,
            self.fancy as u8,
            self.shader
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
                "fancy" => st.fancy = v == "1",
                "shader" => st.shader = v.parse().unwrap_or(st.shader).min(2),
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
    Mesh { pos: ChunkPos, snap: [Option<Arc<crate::chunk::Chunk>>; 9], smooth: bool },
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
        Job::Mesh { pos, snap, smooth } => {
            let mesh = mesh_chunk(pos, &snap, smooth);
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
    item_toast: Option<(String, f32)>,
    last_ui_t: f32,
    last_frame_t: f32,
    fps: f32,
    frames: u32,
    fps_t: f32,
    stats: RenderStats,
    spawn_snapped: bool,
    faced_land: bool,
    load_start: f32,
    edits: u32,
    stats_t: f32,
    pub pointer_locked: bool,
    pub drag_look: bool,
    ever_locked: bool,
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

impl GameApp {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        let atlas = crate::textures::generate_atlas();
        let renderer = Renderer::new(window, &atlas).await;
        let bank = SoundBank::generate();
        let world = World::new(crate::world::World::random_seed());
        let spawn = world.find_spawn();
        let player = Player::new(Vec3::new(spawn.0, spawn.1 + 20.0, spawn.2));

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
        let mut player = player;
        player.fov = settings.fov.to_radians();
        player.fov_cur = player.fov;

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
            mesh_inflight: HashSet::new(),
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
            item_toast: None,
            last_ui_t: -1.0,
            last_frame_t: now_secs(),
            fps: 0.0,
            frames: 0,
            fps_t: now_secs(),
            stats: RenderStats::default(),
            spawn_snapped: false,
            faced_land: false,
            load_start: 0.0,
            edits: 0,
            stats_t: 0.0,
            pointer_locked: false,
            drag_look: false,
            ever_locked: false,
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
                    if self.screen == Screen::Game {
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
                    if self.screen == Screen::Game {
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
                    if self.screen == Screen::Game {
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
                    } else if self.screen == Screen::Game {
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
        let in_game = self.screen == Screen::Game;
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
                        Screen::Game => self.enter_pause(),
                        Screen::Pause => self.resume_game(),
                        Screen::Options => self.close_options(),
                        _ => {}
                    }
                }
            }
            KeyCode::F3 => {
                if pressed && !repeat && in_game {
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

    fn quit_to_title(&mut self) {
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
                self.settings.fancy = !self.settings.fancy;
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
                        ID_OPT_GRAPHICS => set_button_value(w, if s.fancy { "FANCY" } else { "FAST" }),
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
            self.world.dirty.insert(p);
        }
        self.renderer.clear_meshes();
    }

    // ------------------------------------------------------------ update --

    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.day_time = (self.day_time + dt / 600.0) % 1.0; // 10-minute day

        // stream chunks (also during title/menus: the panorama keeps loading)
        self.stream();

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
                        self.world.set_block(pos[0], pos[1], pos[2], AIR);
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
                if let Some((_, _, prev)) = self.target {
                    let b = self.player.hotbar[self.player.selected];
                    if b != AIR {
                        let pb = self.world.get_block(prev[0], prev[1], prev[2]);
                        let replaceable = pb == AIR || pb == WATER || is_cross(pb);
                        let collides_player = is_solid(b) && self.player.block_intersects_player(prev);
                        if replaceable && !collides_player {
                            self.world.set_block(prev[0], prev[1], prev[2], b);
                            self.audio.play(&self.bank, def(b).sound, 0.55 * self.settings.volume, 1.15);
                            self.place_timer = 0.24;
                            self.edits += 1;
                        }
                    }
                }
            }
        }

        // toasts
        if let Some((_, t)) = self.item_toast.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.item_toast = None;
                self.ui.dirty = true;
            }
        }

        // UI rebuild cadence: snappier in menus (hover), relaxed in game
        let cadence = if self.screen == Screen::Game { 0.15 } else { 0.05 };
        if self.ui.dirty && self.time - self.last_ui_t > cadence {
            self.rebuild_ui();
        }

        // publish debug stats for E2E tests (wasm)
        #[cfg(target_arch = "wasm32")]
        {
            if self.time - self.stats_t > 0.25 {
                self.stats_t = self.time;
                self.publish_stats();
            }
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
            ("rd", StatsVal::F(self.settings.render_distance as f32)),
            ("fov", StatsVal::F(self.settings.fov)),
            ("sens", StatsVal::F(self.settings.sensitivity)),
            ("vol", StatsVal::F(self.settings.volume)),
            ("bright", StatsVal::F(self.settings.brightness)),
            ("shader", StatsVal::F(self.settings.shader as f32)),
            ("clouds", StatsVal::B(self.settings.clouds)),
            ("smooth", StatsVal::B(self.settings.smooth_lighting)),
            ("fancy", StatsVal::B(self.settings.fancy)),
            ("edits", StatsVal::F(self.edits as f32)),
            ("fwd", StatsVal::B(self.input.fwd)),
            ("back", StatsVal::B(self.input.back)),
            ("left", StatsVal::B(self.input.left)),
            ("right", StatsVal::B(self.input.right)),
            ("jump", StatsVal::B(self.input.jump)),
            ("breakHold", StatsVal::B(self.input.break_hold)),
            ("placeHold", StatsVal::B(self.input.place_hold)),
            ("hasTarget", StatsVal::B(self.target.is_some())),
            ("breakTimer", StatsVal::F(self.break_timer)),
            ("hover", StatsVal::F(self.hover.map(|h| h as f32).unwrap_or(-1.0))),
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
                self.submit(Job::Mesh { pos, snap, smooth: self.settings.smooth_lighting });
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
                format!("VOXELCRAFT (Rust + wgpu) {} fps", self.fps as i32),
                format!("XYZ: {:.1} / {:.1} / {:.1}", p.pos.x, p.pos.y, p.pos.z),
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
                    "RD: {}  FOV: {:.0}  Vol: {:.0}%  Bright: {:.0}%",
                    self.settings.render_distance,
                    self.settings.fov,
                    self.settings.volume * 100.0,
                    self.settings.brightness * 100.0
                ),
                format!(
                    "Shader: {}  Clouds: {}  Smooth: {}  VSync: {}",
                    ["off", "vanilla+", "cinematic"][self.settings.shader as usize],
                    if self.settings.clouds { "on" } else { "off" },
                    if self.settings.smooth_lighting { "on" } else { "off" },
                    if self.renderer.vsync { "on" } else { "off" }
                ),
                format!("Edits: {} (xp lvl {})  Seed: {}", self.edits, level, self.world.seed),
            ];
            self.ui.debug(&lines);
        }

        if self.show_help {
            self.ui.help();
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
        // fps = real RENDERED frame rate (draws ride RAF on the web)
        self.frames += 1;
        if self.time - self.fps_t > 0.5 {
            self.fps = self.frames as f32 / (self.time - self.fps_t);
            self.frames = 0;
            self.fps_t = self.time;
            self.ui.dirty = true;
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

        self.stats = self.renderer.render(
            &cam,
            &sky,
            &mut self.ui,
            selection,
            &crate::render::PostParams { mode: self.settings.shader, menu_blur },
            self.settings.clouds && self.settings.fancy,
        );
    }
}

// ---------------------------------------------------------------- helpers --

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
