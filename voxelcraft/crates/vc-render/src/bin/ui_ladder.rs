//! Round B (MASTER-PLAN Part III): the menu VISION AUDIT — a headless
//! ladder that drives every layout helper through the same
//! `set_live_ui_size` path the game's GUI-scale resolution uses, at each
//! rung of the audit ladder, then:
//!   1. runs the pixel-audit rules on every produced widget set
//!      (all widgets inside the canvas, primary rows horizontally
//!      centered, uniform row pitch on the settings grids), failing the
//!      rung with a printed violation, and
//!   2. writes the FLATTENED PNG dumps (`dump_png_flat`, the composite
//!      the GPU alpha-blend would produce) for human/vision review —
//!      `<outdir>/<screen>_<W>x<H>.png`.
//!
//! Ladder (MASTER-PLAN Round B): 1280x720 → 1366x768 (primary) →
//! 1600x900 → 1920x1080 → 2560x1440 → 1280x1024 (letterbox — 5:4, the
//! aspect oddball). At each resolution EVERY available GUI scale
//! (vanilla `w/320 × h/240`, so 1280x1024 yields 3) is shot — the
//! live-canvas size is computed exactly like `refresh_gui_scale`:
//! cw = ceil(2*W/scale), ch = ceil(2*H/scale).
//!
//! Run: cargo run -p vc-render --bin ui_ladder -- <outdir>
//! Exit 0 = every rung clean; exit 1 = violations printed.

use vc_render::ui::{self, Widget, WidgetKind};

use vc_render::ui::ID_RPACK_AVAIL_BASE;
use vc_render::ui::ID_RPACK_DOWN_BASE;
use vc_render::ui::MAX_RPACK_ENTRIES;

/// one ladder rung: (window W, H, note)
const LADDER: &[(usize, usize, &str)] = &[
    (1280, 720, "baseline"),
    (1366, 768, "primary"),
    (1600, 900, ""),
    (1920, 1080, "1080p"),
    (2560, 1440, "1440p"),
    (1280, 1024, "letterbox 5:4"),
];

/// the audited screens: (name, layout builder). The dynamic screens
/// (world select / create / packs / controls) use representative
/// sample data — the same fixtures the snapshot tool ships.
fn screens() -> Vec<(&'static str, Vec<Widget>)> {
    let avail: Vec<String> = ["napp-1.16.zip", "Classic Art"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sel: Vec<String> = ["Default", "napp-1.16.zip"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let controls: Vec<(bool, &str, &str)> = [
        (false, "Forward", "W"),
        (false, "Back", "S"),
        (false, "Creative Item Picker", "B"),
    ]
    .to_vec();
    vec![
        ("title", ui::layout_title(false)),
        ("options", ui::layout_options()),
        ("video", ui::layout_video()),
        (
            "shaders",
            ui::layout_shaders(
                &["BSL-v8".to_string(), "SEUS-Renewed".to_string()],
                Some("BSL-v8"),
                false,
            ),
        ),
        ("engine", ui::layout_engine()),
        ("packs", ui::layout_resource_packs(&avail, &sel)),
        ("access", ui::layout_access()),
        ("chat", ui::layout_chat_settings()),
        ("musicsound", ui::layout_music_sound()),
        ("controls", ui::layout_controls(&controls)),
        ("language", ui::layout_language()),
        ("pause", ui::layout_pause()),
        ("worldselect", ui::layout_world_select(3, true, false)),
        (
            "worldcreate",
            ui::layout_world_create(
                false,
                "New World",
                "12345",
                "",
                "Game Mode: Survival",
                "World Type: Default",
                false,
                false,
            ),
        ),
        ("death", ui::layout_death(false)),
    ]
}

/// the Round B pixel-audit rules for one (screen, canvas) widget set.
/// Returns the violation list (empty = clean).
fn audit(name: &str, ws: &[Widget], w: i32, h: i32) -> Vec<String> {
    let mut bad = Vec::new();
    // fixed-width rows: (button id, authored width) — the vanilla pairs
    // that must share width and sit symmetric about the center line
    // (title: options|quit; the two-column settings grids are exempt —
    // their columns are symmetric BY construction).
    for wi in ws {
        let (x, y, bw, bh) = (wi.x, wi.y, wi.w, wi.h);
        // Rule 1: fully on-canvas (0 margin allowed — vanilla buttons
        // can touch the edge on tiny canvases, but never cross it)
        if x < 0 || y < 0 || x + bw > w || y + bh > h {
            bad.push(format!(
                "{name} @{w}x{h}: widget id={} at ({x},{y} {bw}x{bh}) crosses the canvas",
                wi.id
            ));
        }
        // Rule 2: no zero/negative-size widget
        if bw <= 0 || bh <= 0 {
            bad.push(format!(
                "{name} @{w}x{h}: widget id={} has degenerate size {bw}x{bh}",
                wi.id
            ));
        }
    }
    // Rule 3: primary single-column rows centered. A widget counts as a
    // "primary row" when its width is >= 40% of the canvas AND it is
    // neither (a) a cell sharing its y-band with another widget — those
    // belong to multi-column compositions (the controls name|key rows;
    // vanilla centers the column GROUP, never each cell on the canvas
    // midline) — nor (b) a resource-pack PANE row (the two-pane
    // manager's avail/sel/arrows/DEFAULT family, ids 120..158: vanilla
    // anchors those to the panes). Rules 1+2 still bind every one of
    // them. Every primary row must satisfy |(x + bw/2) - w/2| <= 2.
    let pane_family =
        |id: u16| id >= ID_RPACK_AVAIL_BASE && id < ID_RPACK_DOWN_BASE + MAX_RPACK_ENTRIES as u16;
    for (i, wi) in ws.iter().enumerate() {
        let shares_band = ws
            .iter()
            .enumerate()
            .any(|(j, o)| j != i && o.y < wi.y + wi.h && wi.y < o.y + o.h);
        if wi.w * 5 >= w * 2 && !shares_band && !pane_family(wi.id) {
            let center = wi.x + wi.w / 2;
            if (center - w / 2).abs() > 2 {
                bad.push(format!(
                    "{name} @{w}x{h}: wide widget id={} off-center by {}",
                    wi.id,
                    center - w / 2
                ));
            }
        }
    }
    // Rule 4: full-width slider row pitch uniform (the settings grids
    // lay full-width rows every 36px from y=72 — verify no full-width
    // row breaks the pitch). Collect the y of every canvas-width slider;
    // consecutive gaps must be 36 or a multiple (sub-screens skip rows).
    let mut slider_ys: Vec<i32> = ws
        .iter()
        .filter(|wi| wi.w >= w - 2 && matches!(wi.kind, WidgetKind::Slider { .. }))
        .map(|wi| wi.y)
        .collect();
    slider_ys.sort_unstable();
    slider_ys.dedup();
    for pair in slider_ys.windows(2) {
        let gap = pair[1] - pair[0];
        if gap != 36 && gap != 54 {
            bad.push(format!(
                "{name} @{w}x{h}: full-width slider pitch {gap} (want 36 or 54)"
            ));
        }
    }
    bad
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let _ = std::fs::create_dir_all(&out);
    let mut violations = 0usize;
    let mut shots = 0usize;

    for &(rw, rh, note) in LADDER {
        // vanilla's available-scale rule (Settings::gui_scale_available):
        // (w/320).min(h/240).max(1) — duplicated here because the helper
        // lives in the game crate and this audit runs against vc-render
        let avail = ((rw / 320).min(rh / 240)).max(1);
        for scale in 1..=avail {
            // the exact live-canvas math of refresh_gui_scale
            let cw = ((2.0 * rw as f32) / scale as f32).ceil() as usize;
            let ch = ((2.0 * rh as f32) / scale as f32).ceil() as usize;
            ui::set_live_ui_size(cw, ch);
            let tag_note = if note.is_empty() {
                String::new()
            } else {
                format!(" [{note}]")
            };
            println!("== {rw}x{rh} scale {scale} -> canvas {cw}x{ch}{tag_note}");
            for (name, ws) in screens() {
                for v in audit(name, &ws, cw as i32, ch as i32) {
                    println!("  VIOLATION: {v}");
                    violations += 1;
                }
                // dump the flattened composite for vision review
                let mut canvas = ui::UiCanvas::new();
                canvas.resize(cw, ch);
                // paint through the screen painter so the dump shows the
                // REAL chrome (labels, tooltips, footer), not bare widgets
                match name {
                    "title" => canvas.title_screen("Clean-room!", &ws, None, 0.0),
                    "pause" => canvas.pause_screen(&ws, None),
                    _ => {
                        let tt = vec!["Choose what to tune.".to_string()];
                        canvas.settings_screen(&ws, None, name.to_uppercase().as_str(), &tt)
                    }
                }
                let p = format!("{out}/{name}_{rw}x{rh}_s{scale}.png");
                canvas.dump_png_flat(&p, [96, 122, 146, 255]);
                shots += 1;
            }
        }
    }
    println!(
        "\nLADDER DONE: {shots} dumps, {violations} violations — {}",
        if violations == 0 {
            "CLEAN"
        } else {
            "FIX NEEDED"
        }
    );
    std::process::exit(if violations == 0 { 0 } else { 1 });
}
