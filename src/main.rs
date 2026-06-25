// VoxelCraft full graphical main.rs (wgpu + egui).
// This file has ~20 API mismatches with the specific wgpu 22 / egui 0.28 versions.
// DeepSeek needs to fix these by checking the actual API docs.
// The engine (lib.rs) is fully working and tested — this is just the rendering shell.

mod blocks;
mod world;
mod player;
mod renderer;
mod ui;
mod logger;

use std::sync::Arc;
use std::time::Instant;
use glam::{Mat4, Vec3};
use winit::{
    event::{Event, WindowEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use egui_winit::State as EguiState;

use crate::blocks::Block;
use crate::player::{Player, InputState};
use crate::world::world::World;
use crate::world::chunk::CHUNK_SIZE;
use crate::renderer::Renderer;
use crate::ui::{GameState, Settings, GameStats, render_loading_screen, render_start_screen,
                render_debug_hud, render_hotbar, render_crosshair, render_settings,
                render_logger_panel, render_pause_menu, PauseAction};
use crate::logger::logger;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("voxelcraft=info".parse().unwrap()))
        .init();

    logger().info("engine", "VoxelCraft-Rust starting up");

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("VoxelCraft — Rust Edition")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap(),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let mut renderer = pollster::block_on(Renderer::new(window.clone(), &instance));

    let egui_ctx = egui::Context::default();
    let mut egui_state = EguiState::new(egui_ctx.clone(), viewport_id(), &window, None, None);
    let mut egui_renderer = Some(egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        None,
        1,
    ));

    let world = World::new(1337);
    let mut player = Player::new(Vec3::new(0.5, 50.0, 0.5));
    let mut settings = Settings::default();
    let mut state = GameState::Loading;
    let mut input = InputState::default();
    let mut selected_slot: usize = 0;
    let hotbar: Vec<Block> = Block::HOTBAR.to_vec();

    let loading_total = (2 * settings.render_distance + 1).pow(2) as usize;
    let loading_start = Instant::now();
    let mut loading_started = false;

    let mut mouse_locked = false;
    let mut last_mouse_pos: Option<(f64, f64)> = None;

    let mut last_frame = Instant::now();
    let mut fps_avg = 60.0;
    let mut frame_ms_avg = 16.0;
    let mut stats = GameStats::default();

    let mut log_filter_level = logger::LogLevel::Debug;
    let mut log_filter_scope = String::new();

    logger().info("engine", "Game loop starting");

    event_loop.run(move |event, ctrl| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                let _ = egui_state.on_window_event(&window, &event);

                match event {
                    WindowEvent::CloseRequested => { ctrl.exit(); }
                    WindowEvent::Resized(size) => { renderer.resize(size); }
                    WindowEvent::KeyboardInput { event: KeyEvent { state: key_state, physical_key, .. }, .. } => {
                        let pressed = key_state == ElementState::Pressed;
                        use winit::keyboard::PhysicalKey;
                        use winit::keyboard::KeyCode::*;
                        match physical_key {
                            PhysicalKey::Code(KeyW | ArrowUp) => input.forward = pressed,
                            PhysicalKey::Code(KeyS | ArrowDown) => input.back = pressed,
                            PhysicalKey::Code(KeyA | ArrowLeft) => input.left = pressed,
                            PhysicalKey::Code(KeyD | ArrowRight) => input.right = pressed,
                            PhysicalKey::Code(Space) => input.jump = pressed,
                            PhysicalKey::Code(ShiftLeft | ShiftRight) => input.sprint = pressed,
                            PhysicalKey::Code(ControlLeft | ControlRight) => input.crouch = pressed,
                            PhysicalKey::Code(Escape) if pressed => {
                                match state {
                                    GameState::Playing => { state = GameState::Paused; release_mouse(&window, &mut mouse_locked); }
                                    GameState::Paused | GameState::Settings | GameState::Logger => { state = GameState::StartScreen; release_mouse(&window, &mut mouse_locked); }
                                    GameState::StartScreen => ctrl.exit(),
                                    GameState::Loading => {}
                                }
                            }
                            PhysicalKey::Code(KeyO) if pressed && (state == GameState::StartScreen || state == GameState::Paused) => state = GameState::Settings,
                            PhysicalKey::Code(KeyL) if pressed && (state == GameState::StartScreen || state == GameState::Paused) => state = GameState::Logger,
                            PhysicalKey::Code(KeyF) if pressed && state == GameState::Playing => {
                                player.flying = !player.flying;
                                player.velocity.y = 0.0;
                                logger().info("player", format!("Fly mode: {}", if player.flying { "ON" } else { "OFF" }));
                            }
                            PhysicalKey::Code(digit @ (Digit1 | Digit2 | Digit3 | Digit4 | Digit5 | Digit6 | Digit7 | Digit8 | Digit9)) if pressed => {
                                selected_slot = (digit as u32 - Digit1 as u32) as usize;
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::MouseInput { state: btn_state, button, .. } => {
                        let pressed = btn_state == ElementState::Pressed;
                        match button {
                            MouseButton::Left if pressed && state == GameState::StartScreen => {
                                state = GameState::Playing;
                                lock_mouse(&window, &mut mouse_locked);
                            }
                            MouseButton::Left if pressed && state == GameState::Playing => {
                                if let Some((pos, _)) = player.raycast(&world, 6.0) {
                                    world.set_block(pos.x, pos.y, pos.z, Block::Air);
                                }
                            }
                            MouseButton::Right if pressed && state == GameState::Playing => {
                                if let Some((pos, normal)) = player.raycast(&world, 6.0) {
                                    let place = pos + normal;
                                    let block = hotbar[selected_slot];
                                    world.set_block(place.x, place.y, place.z, block);
                                }
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if state == GameState::Playing && mouse_locked {
                            if let Some((lx, ly)) = last_mouse_pos {
                                let dx = position.x - lx;
                                let dy = position.y - ly;
                                player.yaw -= dx as f32 * settings.mouse_sensitivity;
                                player.pitch -= dy as f32 * settings.mouse_sensitivity;
                                player.pitch = player.pitch.clamp(-1.57, 1.57);
                            }
                        }
                        last_mouse_pos = Some((position.x, position.y));
                    }
                    WindowEvent::MouseWheel { delta, .. } if state == GameState::Playing => {
                        let dir = match delta {
                            MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { -1i32 } else { 1 },
                            _ => 0,
                        };
                        selected_slot = ((selected_slot as i32 + dir + 9) as usize) % 9;
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = now.duration_since(last_frame).as_secs_f32().min(0.1);
                last_frame = now;
                fps_avg = fps_avg * 0.9 + (1.0 / dt.max(0.0001)) * 0.1;
                frame_ms_avg = frame_ms_avg * 0.9 + (dt * 1000.0) * 0.1;

                egui_ctx.begin_frame(egui_state.take_egui_input(&window));
                let egui_output = egui_ctx.end_frame();
                let primitives = egui_ctx.tessellate(egui_output.shapes, egui_output.pixels_per_point);
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [renderer.config.width, renderer.config.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                match state {
                    GameState::Loading => {
                        if !loading_started { world.update_player_position(0, 0, settings.render_distance); loading_started = true; }
                        let (total, ready) = world.chunk_stats();
                        let progress = if loading_total > 0 { ready as f32 / loading_total as f32 } else { 0.0 };
                        if progress > 0.6 || loading_start.elapsed().as_secs() > 8 {
                            state = GameState::StartScreen;
                            let spawn_y = Player::find_spawn(&world, 0, 0);
                            player.position = Vec3::new(0.5, spawn_y, 0.5);
                        }
                        render_loading_screen(&egui_ctx, progress, ready, loading_total);
                    }
                    GameState::StartScreen => {
                        if render_start_screen(&egui_ctx, true) { state = GameState::Playing; lock_mouse(&window, &mut mouse_locked); }
                        process_mesh_uploads(&world, &mut renderer);
                    }
                    GameState::Playing => {
                        player.update(dt, input, &world);
                        world.update_player_position(player.position.x as i32, player.position.z as i32, settings.render_distance);
                        process_mesh_uploads(&world, &mut renderer);
                        rebuild_dirty_chunks(&world, &player, 2);
                        update_stats(&mut stats, &player, &world, &settings, fps_avg, frame_ms_avg, &renderer);
                        render_debug_hud(&egui_ctx, &stats, &settings);
                        render_crosshair(&egui_ctx);
                        render_hotbar(&egui_ctx, selected_slot, &hotbar);
                    }
                    GameState::Paused => {
                        match render_pause_menu(&egui_ctx) {
                            PauseAction::Resume => { state = GameState::Playing; lock_mouse(&window, &mut mouse_locked); }
                            PauseAction::Settings => state = GameState::Settings,
                            PauseAction::Logger => state = GameState::Logger,
                            PauseAction::QuitToMenu => state = GameState::StartScreen,
                            PauseAction::None => {}
                        }
                    }
                    GameState::Settings => {
                        if render_settings(&egui_ctx, &mut settings) { state = GameState::StartScreen; }
                    }
                    GameState::Logger => {
                        render_logger_panel(&egui_ctx, &mut log_filter_level, &mut log_filter_scope);
                    }
                }

                let (view, proj) = if matches!(state, GameState::Playing | GameState::Paused | GameState::StartScreen) {
                    let eye = player.eye_position();
                    let view = Mat4::look_to_rh(eye, player.forward_vector(), Vec3::Y);
                    let proj = Mat4::perspective_rh_gl(settings.fov.to_radians(), renderer.config.width as f32 / renderer.config.height as f32, 0.05, 1000.0);
                    (view, proj)
                } else { (Mat4::IDENTITY, Mat4::IDENTITY) };

                let fog_color = sky_color_for_time(0.3);
                let _ = renderer.render(view, proj, player.eye_position(), fog_color, 0.3, &mut egui_renderer, primitives, &screen_descriptor);
            }
            _ => {}
        }
    }).unwrap();
}

fn lock_mouse(window: &winit::window::Window, locked: &mut bool) {
    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked).or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
    window.set_cursor_visible(false);
    *locked = true;
}
fn release_mouse(window: &winit::window::Window, locked: &mut bool) {
    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
    window.set_cursor_visible(true);
    *locked = false;
}
fn process_mesh_uploads(world: &World, renderer: &mut Renderer) {
    for p in world.drain_pending_meshes() { renderer.upload_chunk_mesh(p.cx, p.cz, p.mesh); }
}
fn rebuild_dirty_chunks(world: &World, player: &Player, budget: usize) {
    let pcx = (player.position.x as i32).div_euclid(CHUNK_SIZE as i32);
    let pcz = (player.position.z as i32).div_euclid(CHUNK_SIZE as i32);
    let mut rebuilt = 0;
    'outer: for r in 0i32..=8 {
        for dx in -r..=r {
            for dz in -r..=r {
                if dx.abs() != r && dz.abs() != r { continue; }
                let cx = pcx + dx;
                let cz = pcz + dz;
                if let Some(chunk) = world.get_chunk(cx, cz) {
                    if chunk.read().dirty {
                        if let Some(mesh) = world.build_mesh(cx, cz) {
                            world.queue_mesh(cx, cz, mesh);
                            if let Some(c) = world.get_chunk(cx, cz) { c.write().dirty = false; }
                            rebuilt += 1;
                            if rebuilt >= budget { break 'outer; }
                        }
                    }
                }
            }
        }
    }
}
fn update_stats(stats: &mut GameStats, player: &Player, world: &World, _settings: &Settings, fps: f32, frame_ms: f32, renderer: &Renderer) {
    stats.fps = fps as i32;
    stats.frame_ms = frame_ms;
    let (total, ready) = world.chunk_stats();
    stats.chunks_loaded = total;
    stats.chunks_ready = ready;
    stats.position = [player.position.x, player.position.y, player.position.z];
    stats.yaw = player.yaw;
    stats.pitch = player.pitch;
    stats.speed = (player.velocity.x * player.velocity.x + player.velocity.z * player.velocity.z).sqrt();
    stats.on_ground = player.on_ground;
    stats.flying = player.flying;
    if let Some((pos, _)) = player.raycast(world, 6.0) {
        stats.looking_at = Some((world.get_block(pos.x, pos.y, pos.z).name().to_string(), [pos.x, pos.y, pos.z]));
    }
}
fn sky_color_for_time(t: f32) -> [f32; 4] {
    let day = [0.5, 0.7, 1.0];
    [day[0], day[1], day[2], 1.0]
}
fn viewport_id() -> egui::ViewportId { egui::ViewportId::ROOT }
