//! Tinkerbench Setup: installs Tinkerbench for the current user with what it
//! needs from the system, finds Warcraft III and hands the choices to the app.
//!
//! `--uninstall` opens on removal; `--dry-run` shows the system-package
//! command instead of running it.

#![windows_subsystem = "windows"]

mod app;
mod install;
mod payload;
mod platform;
mod requirements;
mod theme;

use eframe::egui;

const WINDOW_SIZE: [f32; 2] = [820.0, 600.0];
const MIN_WINDOW_SIZE: [f32; 2] = [700.0, 520.0];

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = app::Options {
        dry_run: args.iter().any(|a| a == "--dry-run"),
        uninstall: args.iter().any(|a| a == "--uninstall"),
    };
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Tinkerbench Setup")
        .with_app_id(wc3_install::APP_IDENTIFIER)
        .with_inner_size(WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);

    if let Ok(icon) = eframe::icon_data::from_png_bytes(payload::WINDOW_ICON) {
        viewport = viewport.with_icon(icon);
    }

    eframe::run_native(
        "Tinkerbench Setup",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::SetupApp::new(&cc.egui_ctx, options)))),
    )
}
