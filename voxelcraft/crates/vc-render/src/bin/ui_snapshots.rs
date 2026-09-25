//! Headless UI-snapshot tool (2026-09-20 round): renders the key menu
//! screens through the REAL UiCanvas painters (the same code the game
//! loop drives) and writes FLATTENED PNGs for visual review — the
//! canvas is composited over a stand-in backdrop the way the GPU's
//! alpha-blend composites it over the blurred panorama at render time.
//!
//! Run: cargo run -p vc-render --bin ui_snapshots -- <outdir>

use vc_render::ui::{self, UiCanvas};

/// a neutral mid-tone stand-in for the blurred panorama
const PANO: [u8; 4] = [96, 122, 146, 255];

fn snap(name: &str, paint: &dyn Fn(&mut UiCanvas)) {
    let mut ui = UiCanvas::new();
    ui.resize(ui::UI_W, ui::UI_H);
    paint(&mut ui);
    let p = format!("{name}.png");
    ui.dump_png_flat(&p, PANO);
    println!("wrote {p}");
}

fn main() {
    // `--size WxH` shoots the whole set at a different live canvas size
    // (e.g. 1920x1080, 2560x1440) through the SAME set_live_ui_size path
    // the game uses — the visual-proof source for the multi-resolution
    // centering contract. Default (no flag) stays 960x540 with unchanged
    // filenames.
    let mut w = ui::UI_W;
    let mut h = ui::UI_H;
    if let Some(pos) = std::env::args().position(|a| a == "--size") {
        if let Some(spec) = std::env::args().nth(pos + 1) {
            let mut it = spec.splitn(2, 'x');
            if let (Some(pw), Some(ph)) = (it.next(), it.next()) {
                if let (Ok(pw), Ok(ph)) = (pw.parse::<usize>(), ph.parse::<usize>()) {
                    w = pw;
                    h = ph;
                }
            }
        }
    }
    let tag = if (w, h) == (ui::UI_W, ui::UI_H) {
        String::new()
    } else {
        format!("_{w}x{h}")
    };
    // drive layouts through the same live-size path the game does —
    // widget geometry (cx2 centering, anchor_y bottom rows) reads this
    ui::set_live_ui_size(w, h);
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let _ = std::fs::create_dir_all(&out);
    let cwd = std::env::current_dir().unwrap_or_default();
    let base = if out.starts_with('/') {
        out
    } else {
        format!("{}/{}", cwd.display(), out)
    };
    let snap_at = |name: &str, paint: &dyn Fn(&mut UiCanvas)| {
        let mut ui = UiCanvas::new();
        ui.resize(w, h);
        paint(&mut ui);
        let p = format!("{base}/{name}{tag}.png");
        ui.dump_png_flat(&p, PANO);
        println!("wrote {p}");
    };
    let _ = snap;

    // title
    let ws = ui::layout_title(false);
    snap_at("title", &|ui| {
        ui.title_screen("Clean-room!", &ws, None, 0.0);
    });
    // title with hover on SINGLEPLAYER (the white hover frame)
    snap_at("title-hover", &|ui| {
        ui.title_screen("Clean-room!", &ws, Some(ui::ID_TITLE_PLAY), 0.0);
    });
    // options
    let opts = ui::layout_options();
    snap_at("options", &|ui| {
        let tt = vec!["Choose what to tune.".to_string()];
        ui.settings_screen(&opts, None, "OPTIONS", &tt);
    });
    // video (with the new ENTITY DISTANCE row)
    let video = ui::layout_video();
    snap_at("video", &|ui| {
        ui.settings_screen(&video, None, "VIDEO SETTINGS", &[]);
    });
    // video hovering the entity distance slider
    let ed = video
        .iter()
        .find(|w| w.id == ui::ID_OPT_ENTDIST)
        .map(|w| w.id);
    snap_at("video-hover-entdist", &|ui| {
        ui.settings_screen(
            &video,
            ed,
            "VIDEO SETTINGS",
            &[
                "How far away creatures render, as a share of the render distance.".to_string(),
                "50% hides distant mobs and speeds up crowded scenes.".to_string(),
            ],
        );
    });
    // shaders screen
    let shd = ui::layout_shaders(
        &["BSL-v8".to_string(), "SEUS-Renewed".to_string()],
        Some("BSL-v8"),
        false,
    );
    snap_at("shaders", &|ui| {
        ui.shader_screen(&shd, None, &["Select a pack to activate it.".to_string()]);
    });
    // pause
    let pause = ui::layout_pause();
    snap_at("pause", &|ui| {
        ui.pause_screen(&pause, None);
    });
    // resource packs
    let avail: Vec<String> = ["napp-1.16.zip", "Classic Art"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sel: Vec<String> = ["Default", "napp-1.16.zip"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let packs = ui::layout_resource_packs(&avail, &sel);
    snap_at("packs", &|ui| {
        ui.resource_pack_screen(&packs, None, &[]);
    });
}
