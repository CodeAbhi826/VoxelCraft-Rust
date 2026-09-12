//! `gen_gui_assets` — dev tool (UI-overhaul Phase 1, D7).
//!
//! Emits the procedural GUI sprites as PNGs for HUMAN INSPECTION ONLY.
//! The game NEVER reads these files — it repaints every sprite in
//! memory at boot (G9). This binary exists so a human can eyeball the
//! chrome without launching the game.
//!
//! Output goes to `target/gui-art/` (build output, gitignored, never
//! embedded into the binary — deliberately NOT `assets/` so the
//! emitted files can never leak into the runtime pack paths).
//!
//! Idempotent: the painters are deterministic, so two runs produce
//! byte-identical PNGs (A8 verifies this with sha256).
//!
//! Run: `cargo run -p vc-render --bin gen_gui_assets`

use std::path::{Path, PathBuf};

use vc_render::gui::GuiTextureSet;
use vc_render::textures::gui_art;

fn save(path: &Path, name: &str, px: &[u8], w: u32, h: u32) -> bool {
    if px.len() != (w as usize) * (h as usize) * 4 {
        return false;
    }
    let img = image::RgbaImage::from_raw(w, h, px.to_vec()).unwrap_or_default();
    img.save(path.join(format!("{name}.png"))).is_ok()
}

fn main() {
    let out_dir = {
        // target/gui-art next to whatever profile cargo picked
        let mut p = PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()));
        p.push("gui-art");
        p
    };
    if std::fs::create_dir_all(&out_dir).is_err() {
        eprintln!("[gen_gui_assets] cannot create {}", out_dir.display());
        return;
    }

    let set = GuiTextureSet::build_builtin();
    let mut n = 0usize;
    for (name, sheet) in [
        ("hearts", &set.hearts),
        ("hunger", &set.hunger),
        ("armor", &set.armor),
        ("bubbles", &set.bubbles),
        ("widgets", &set.widgets),
        ("hotbar", &set.hotbar_bg),
        ("hotbar_sel", &set.hotbar_sel),
        ("options_background", &set.dirt),
    ] {
        if save(&out_dir, name, &sheet.px, sheet.w as u32, sheet.h as u32) {
            n += 1;
        } else {
            eprintln!("[gen_gui_assets] FAILED to write {name}.png");
        }
    }

    // individual 9x9 sprites at 8x zoom for easier eyeballing
    let zoom = |px: &[u8], w: usize, h: usize| -> Vec<u8> {
        let mut big = vec![0u8; w * 8 * h * 8 * 4];
        for y in 0..h {
            for x in 0..w {
                let s = (y * w + x) * 4;
                if s + 3 >= px.len() {
                    continue;
                }
                for dy in 0..8usize {
                    for dx in 0..8usize {
                        let d = ((y * 8 + dy) * w * 8 + (x * 8 + dx)) * 4;
                        big[d..d + 4].copy_from_slice(&px[s..s + 4]);
                    }
                }
            }
        }
        big
    };

    use gui_art::{ArmorVariant, BubbleVariant, HeartVariant, HungerVariant};
    let sprites: [(&str, Vec<u8>, usize, usize); 11] = [
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_heart(&mut t, 9, HeartVariant::Full);
            ("heart_full_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_heart(&mut t, 9, HeartVariant::Half);
            ("heart_half_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_heart(&mut t, 9, HeartVariant::Empty);
            ("heart_empty_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_hunger(&mut t, 9, HungerVariant::Full);
            ("hunger_full_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_hunger(&mut t, 9, HungerVariant::Empty);
            ("hunger_empty_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_armor(&mut t, 9, ArmorVariant::Full);
            ("armor_full_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 9 * 9 * 4];
            gui_art::draw_bubble(&mut t, 9, BubbleVariant::Full);
            ("bubble_full_x8", zoom(&t, 9, 9), 72, 72)
        },
        {
            let mut t = vec![0u8; 20 * 20 * 4];
            gui_art::draw_widget(&mut t, 20, gui_art::WidgetVariant::ButtonNormal);
            ("button_normal_x8", zoom(&t, 20, 20), 160, 160)
        },
        {
            let mut t = vec![0u8; 18 * 18 * 4];
            gui_art::draw_widget(&mut t, 18, gui_art::WidgetVariant::SlotEmpty);
            ("slot_empty_x8", zoom(&t, 18, 18), 144, 144)
        },
        {
            let mut t = vec![0u8; 20 * 20 * 4];
            gui_art::draw_widget(&mut t, 20, gui_art::WidgetVariant::Panel);
            ("panel_x8", zoom(&t, 20, 20), 160, 160)
        },
        {
            let mut t = vec![0u8; 16 * 16 * 4];
            gui_art::draw_options_dirt(&mut t, 16);
            ("dirt_x8", zoom(&t, 16, 16), 128, 128)
        },
    ];
    for (name, px, w, h) in sprites {
        if save(&out_dir, name, &px, w as u32, h as u32) {
            n += 1;
        }
    }

    println!(
        "[gen_gui_assets] wrote {n} PNGs to {} (inspection only — the game regenerates these in memory)",
        out_dir.display()
    );
}
