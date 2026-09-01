// egui UI: start screen, settings panel, logger panel, debug HUD, hotbar.
// All the polished UI that the C++ version couldn't have.

use egui::{Color32, RichText, Vec2, Rect, Pos2, Stroke};
use crate::logger::{self, LogLevel};
use crate::blocks::Block;
use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Loading,
    StartScreen,
    Playing,
    Paused,
    Settings,
    Logger,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub render_distance: i32,
    pub user_render_distance: i32,
    pub fov: f32,
    pub mouse_sensitivity: f32,
    pub day_night_speed: f32,
    pub fog_enabled: bool,
    pub clouds_enabled: bool,
    pub show_fps: bool,
    pub show_debug: bool,
    pub vsync: bool,
    pub fly_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 6,
            user_render_distance: 6,
            fov: 80.0,
            mouse_sensitivity: 0.0022,
            day_night_speed: 0.0,
            fog_enabled: true,
            clouds_enabled: true,
            show_fps: true,
            show_debug: true,
            vsync: true,
            fly_mode: false,
        }
    }
}

pub struct GameStats {
    pub fps: i32,
    pub frame_ms: f32,
    pub chunks_loaded: usize,
    pub chunks_ready: usize,
    pub triangles: usize,
    pub draw_calls: usize,
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub time_of_day: f32,
    pub looking_at: Option<(String, [i32; 3])>,
    pub on_ground: bool,
    pub in_water: bool,
    pub flying: bool,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            fps: 0,
            frame_ms: 16.0,
            chunks_loaded: 0,
            chunks_ready: 0,
            triangles: 0,
            draw_calls: 0,
            position: [0.0; 3],
            yaw: 0.0,
            pitch: 0.0,
            speed: 0.0,
            time_of_day: 0.3,
            looking_at: None,
            on_ground: false,
            in_water: false,
            flying: false,
        }
    }
}

/// Render the loading screen with a progress bar.
pub fn render_loading_screen(ctx: &egui::Context, progress: f32, loaded: usize, total: usize) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(12, 18, 32)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(150.0);
                ui.heading(RichText::new("VOXELCRAFT").size(56.0).color(Color32::WHITE));
                ui.add_space(20.0);
                ui.label(RichText::new("Generating World...").size(18.0).color(Color32::from_gray(150)));

                ui.add_space(30.0);
                let bar_width = 400.0;
                let progress_pct = progress.clamp(0.0, 1.0);
                ui.add(
                    egui::ProgressBar::new(progress_pct)
                        .desired_width(bar_width)
                        .text(format!("{} / {} chunks ({}%)", loaded, total, (progress_pct * 100.0) as i32))
                );

                ui.add_space(20.0);
                // Animated dots
                let dots = (ui.input(|i| i.time) as i32) % 4;
                let dot_str = "LOADING".to_string() + &".".repeat(dots as usize);
                ui.label(RichText::new(dot_str).size(14.0).color(Color32::from_gray(100)));
            });
        });
}

/// Render the start screen / main menu.
pub fn render_start_screen(ctx: &egui::Context, show_help: bool) -> bool {
    let mut clicked_play = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading(RichText::new("VOXELCRAFT").size(64.0).color(Color32::WHITE));
                ui.add_space(8.0);
                ui.label(RichText::new("A Minecraft-style voxel sandbox").size(16.0).color(Color32::from_gray(180)));

                ui.add_space(40.0);
                let btn_size = Vec2::new(280.0, 50.0);
                if ui.add_sized(btn_size, egui::Button::new(RichText::new("PLAY").size(20.0)).fill(Color32::from_rgb(40, 167, 69))).clicked() {
                    clicked_play = true;
                }
                ui.add_space(10.0);
                if ui.add_sized(btn_size, egui::Button::new(RichText::new("SETTINGS (O)").size(18.0)).fill(Color32::from_rgb(50, 50, 55))).clicked() {
                    // handled by key, but show hint
                }
                ui.add_space(10.0);
                if ui.add_sized(btn_size, egui::Button::new(RichText::new("LOGGER (L)").size(18.0)).fill(Color32::from_rgb(50, 50, 55))).clicked() {
                    // handled by key
                }

                ui.add_space(30.0);
                ui.label(RichText::new("Press O for Settings  |  L for Logger  |  H for Help").color(Color32::from_gray(140)));

                if show_help {
                    ui.add_space(20.0);
                    render_help_overlay(ui);
                }
            });
        });

    clicked_play
}

fn render_help_overlay(ui: &mut egui::Ui) {
    egui::Frame::popup(ui.style())
        .fill(Color32::from_rgba_unmultiplied(30, 30, 35, 240))
        .show(ui, |ui| {
            ui.set_width(400.0);
            ui.heading(RichText::new("How to Play").color(Color32::WHITE));
            ui.add_space(10.0);
            egui::Grid::new("help_grid").num_columns(2).spacing([20.0, 6.0]).show(ui, |ui| {
                let rows = [
                    ("Move", "WASD / Arrows"),
                    ("Look", "Mouse"),
                    ("Jump", "Space"),
                    ("Sprint", "Shift (while moving)"),
                    ("Break block", "Hold Left Click"),
                    ("Place block", "Right Click"),
                    ("Hotbar", "1-9 / scroll wheel"),
                    ("Fly toggle", "F"),
                    ("Fly descend", "Ctrl (in fly mode)"),
                    ("Settings", "O / ESC (pause)"),
                    ("Logger", "L"),
                    ("Release mouse", "ESC"),
                ];
                for (action, key) in rows {
                    ui.label(RichText::new(action).color(Color32::from_gray(150)));
                    ui.label(RichText::new(key).color(Color32::WHITE));
                    ui.end_row();
                }
            });
        });
}

/// Render the in-game debug HUD (FPS + stats, top-left).
pub fn render_debug_hud(ctx: &egui::Context, stats: &GameStats, settings: &Settings) {
    if !settings.show_fps && !settings.show_debug {
        return;
    }

    egui::Area::new(egui::Id::new("debug_hud"))
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 120))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    if settings.show_fps {
                        let fps_color = if stats.fps >= 55 { Color32::from_rgb(77, 230, 102) }
                                      else if stats.fps >= 30 { Color32::from_rgb(242, 178, 51) }
                                      else { Color32::from_rgb(242, 77, 77) };
                        ui.label(RichText::new(format!("{} FPS", stats.fps)).size(24.0).color(fps_color));
                    }
                    if settings.show_debug {
                        ui.add_space(4.0);
                        let state_str = if stats.on_ground { "[ground]" } else if stats.flying { "[fly]" } else { "[air]" };
                        let debug_text = format!(
                            "frame: {:.1} ms\nchunks: {} / {}\ntris: {}\ndraw calls: {}\npos: {:.1}, {:.1}, {:.1}\nyaw: {:.0} deg  pitch: {:.0} deg\nspeed: {:.1} b/s  {}\ntime: {:.1}h\nrd: {}  fov: {}",
                            stats.frame_ms,
                            stats.chunks_ready, stats.chunks_loaded,
                            stats.triangles,
                            stats.draw_calls,
                            stats.position[0], stats.position[1], stats.position[2],
                            stats.yaw.to_degrees(), stats.pitch.to_degrees(),
                            stats.speed,
                            state_str,
                            stats.time_of_day * 24.0,
                            settings.render_distance,
                            settings.fov as i32,
                        );
                        ui.label(RichText::new(debug_text).size(12.0).color(Color32::from_gray(220)).monospace());
                        if let Some((block, pos)) = &stats.looking_at {
                            ui.label(RichText::new(format!("looking: {} @ ({},{},{})", block, pos[0], pos[1], pos[2])).size(12.0).color(Color32::from_gray(180)).monospace());
                        }
                    }
                });
        });
}

/// Render the hotbar at the bottom of the screen.
pub fn render_hotbar(ctx: &egui::Context, selected_slot: usize, hotbar: &[Block]) {
    egui::Area::new(egui::Id::new("hotbar"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -10.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, block) in hotbar.iter().enumerate() {
                    let is_selected = i == selected_slot;
                    let color = block_color(*block);
                    let border = if is_selected { Stroke::new(3.0, Color32::WHITE) } else { Stroke::new(1.0, Color32::from_gray(80)) };
                    let bg = Color32::from_rgba_unmultiplied(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                        if is_selected { 240 } else { 160 },
                    );
                    egui::Frame::none()
                        .fill(bg)
                        .stroke(border)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(44.0, 44.0));
                            ui.vertical_centered(|ui| {
                                ui.add_space(2.0);
                                ui.label(RichText::new(block.name().chars().next().unwrap_or('?').to_string()).size(16.0).color(Color32::WHITE));
                                ui.label(RichText::new(format!("{}", i + 1)).size(9.0).color(Color32::from_gray(200)));
                            });
                        });
                }
            });
        });
}

/// Render the crosshair in the center of the screen.
pub fn render_crosshair(ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("crosshair"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::hover());
            let painter = ui.painter();
            let center = rect.center();
            // White crosshair with dark outline
            for offset in [1.0, 0.0] {
                let color = if offset > 0.0 { Color32::from_black_alpha(140) } else { Color32::WHITE };
                painter.line_segment([Pos2::new(center.x - 8.0 - offset, center.y), Pos2::new(center.x - 3.0 + offset, center.y)], Stroke::new(2.0, color));
                painter.line_segment([Pos2::new(center.x + 3.0 - offset, center.y), Pos2::new(center.x + 8.0 + offset, center.y)], Stroke::new(2.0, color));
                painter.line_segment([Pos2::new(center.x, center.y - 8.0 - offset), Pos2::new(center.x, center.y - 3.0 + offset)], Stroke::new(2.0, color));
                painter.line_segment([Pos2::new(center.x, center.y + 3.0 - offset), Pos2::new(center.x, center.y + 8.0 + offset)], Stroke::new(2.0, color));
            }
        });
}

/// Render the settings panel (modal).
pub fn render_settings(ctx: &egui::Context, settings: &mut Settings) -> bool {
    let mut close = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(30, 30, 35))
                    .stroke(Stroke::new(1.0, Color32::from_gray(80)))
                    .inner_margin(30.0)
                    .show(ui, |ui| {
                        ui.set_width(600.0);
                        ui.heading(RichText::new("Settings").size(28.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Tweak rendering, controls, and world").color(Color32::from_gray(150)));

                        ui.add_space(16.0);
                        ui.label(RichText::new("RENDERING").size(14.0).color(Color32::from_gray(150)));
                        ui.add_space(4.0);
                        egui::Grid::new("render_settings").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label("Render Distance");
                            ui.add(egui::Slider::new(&mut settings.user_render_distance, 2..=12).text("chunks"));
                            ui.end_row();
                            settings.render_distance = settings.user_render_distance;
                            ui.label("Field of View");
                            ui.add(egui::Slider::new(&mut settings.fov, 60.0..=110.0).text("°"));
                            ui.end_row();
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("CONTROLS").size(14.0).color(Color32::from_gray(150)));
                        ui.add_space(4.0);
                        egui::Grid::new("control_settings").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label("Mouse Sensitivity");
                            ui.add(egui::Slider::new(&mut settings.mouse_sensitivity, 0.001..=0.010).text(""));
                            ui.end_row();
                            ui.label("Start in Fly Mode");
                            ui.checkbox(&mut settings.fly_mode, "");
                            ui.end_row();
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("WORLD").size(14.0).color(Color32::from_gray(150)));
                        ui.add_space(4.0);
                        egui::Grid::new("world_settings").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label("Day/Night Speed");
                            ui.add(egui::Slider::new(&mut settings.day_night_speed, 0.0..=10.0).text("cycles/min"));
                            ui.end_row();
                            ui.label("Fog");
                            ui.checkbox(&mut settings.fog_enabled, "");
                            ui.end_row();
                            ui.label("Clouds");
                            ui.checkbox(&mut settings.clouds_enabled, "");
                            ui.end_row();
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("HUD").size(14.0).color(Color32::from_gray(150)));
                        ui.add_space(4.0);
                        egui::Grid::new("hud_settings").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                            ui.label("Show FPS Counter");
                            ui.checkbox(&mut settings.show_fps, "");
                            ui.end_row();
                            ui.label("Show Debug Stats");
                            ui.checkbox(&mut settings.show_debug, "");
                            ui.end_row();
                            ui.label("VSync");
                            ui.checkbox(&mut settings.vsync, "");
                            ui.end_row();
                        });

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.add_sized([120.0, 36.0], egui::Button::new("Close (Esc)").fill(Color32::from_rgb(60, 60, 65))).clicked() {
                                close = true;
                            }
                        });
                    });
            });
        });

    close
}

/// Render the logger panel.
pub fn render_logger_panel(ctx: &egui::Context, filter_level: &mut LogLevel, filter_scope: &mut String) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 20, 26))
                    .stroke(Stroke::new(1.0, Color32::from_gray(80)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_width(900.0);
                        ui.set_height(600.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new("Debug Logger").size(22.0).color(Color32::WHITE));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let entries = logger::logger().entries();
                                    ui.label(RichText::new(format!("{} entries", entries.len())).color(Color32::from_gray(150)));
                                });
                            });

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label("Level:");
                                let levels = [LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error];
                                egui::ComboBox::from_label("")
                                    .selected_text(filter_level.label())
                                    .show_ui(ui, |ui| {
                                        for lvl in levels {
                                            ui.selectable_value(filter_level, lvl, lvl.label());
                                        }
                                    });
                                ui.label("Scope:");
                                ui.text_edit_singleline(filter_scope);
                                if ui.button("Clear").clicked() {
                                    logger::logger().clear();
                                }
                            });

                            ui.add_space(8.0);
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let entries = logger::logger().entries();
                                let filtered: Vec<_> = entries.iter()
                                    .filter(|e| (*filter_level as u8) <= (e.level as u8))
                                    .filter(|e| filter_scope.is_empty() || e.scope.contains(filter_scope.as_str()))
                                    .collect();
                                for entry in filtered.iter().rev() {
                                    let color = entry.level.color();
                                    let color32 = Color32::from_rgb(
                                        (color[0] * 255.0) as u8,
                                        (color[1] * 255.0) as u8,
                                        (color[2] * 255.0) as u8,
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{:8.3}", entry.timestamp)).color(Color32::from_gray(100)).monospace());
                                        ui.label(RichText::new(entry.level.label()).color(color32).monospace());
                                        ui.label(RichText::new(format!("[{}]", entry.scope)).color(Color32::from_gray(160)).monospace());
                                        ui.label(RichText::new(&entry.message).color(Color32::from_gray(230)).monospace());
                                    });
                                }
                            });

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Copy All").clicked() {
                                    let entries = logger::logger().entries();
                                    let text: String = entries.iter()
                                        .map(|e| format!("[{}] [{}] [{}] {}", e.timestamp, e.level.label(), e.scope, e.message))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    ui.ctx().copy_text(text);
                                }
                                if ui.button("Download").clicked() {
                                    // In a real app, write to file. For now, copy.
                                }
                                if ui.button("Mirror to stderr: ON").clicked() {
                                    // toggle
                                }
                            });
                        });
                    });
            });
        });
}

fn block_color(block: Block) -> [f32; 3] {
    match block {
        Block::Grass => [0.36, 0.58, 0.20],
        Block::Dirt => [0.55, 0.45, 0.33],
        Block::Stone => [0.50, 0.50, 0.50],
        Block::Cobblestone => [0.39, 0.39, 0.39],
        Block::Wood => [0.63, 0.47, 0.31],
        Block::Planks => [0.63, 0.47, 0.31],
        Block::Leaves => [0.16, 0.43, 0.12],
        Block::Sand => [0.86, 0.81, 0.64],
        Block::Glass => [0.78, 0.86, 0.94],
        Block::Bedrock => [0.16, 0.16, 0.16],
        _ => [0.5, 0.5, 0.5],
    }
}

/// Pause menu shown when ESC is pressed during play.
pub fn render_pause_menu(ctx: &egui::Context) -> PauseAction {
    let mut action = PauseAction::None;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading(RichText::new("PAUSED").size(48.0).color(Color32::WHITE));
                ui.add_space(30.0);
                let btn_size = Vec2::new(240.0, 44.0);
                if ui.add_sized(btn_size, egui::Button::new("Resume").fill(Color32::from_rgb(40, 167, 69))).clicked() {
                    action = PauseAction::Resume;
                }
                ui.add_space(8.0);
                if ui.add_sized(btn_size, egui::Button::new("Settings").fill(Color32::from_rgb(50, 50, 55))).clicked() {
                    action = PauseAction::Settings;
                }
                ui.add_space(8.0);
                if ui.add_sized(btn_size, egui::Button::new("Logger").fill(Color32::from_rgb(50, 50, 55))).clicked() {
                    action = PauseAction::Logger;
                }
                ui.add_space(8.0);
                if ui.add_sized(btn_size, egui::Button::new("Quit to Menu").fill(Color32::from_rgb(120, 40, 40))).clicked() {
                    action = PauseAction::QuitToMenu;
                }
            });
        });

    action
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    None,
    Resume,
    Settings,
    Logger,
    QuitToMenu,
}
