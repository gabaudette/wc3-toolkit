//! WC3 Object Editor: a desktop editor for Warcraft III map object data.

// Release builds on Windows run without a console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Startup failures are reported on stderr (AGE-QUAL-003: documented, file-scoped).
#![allow(clippy::print_stderr)]

mod commands;
mod icons;
mod platform;
mod state;
mod view;

use std::process::ExitCode;

fn main() -> ExitCode {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .register_asynchronous_uri_scheme_protocol(icons::SCHEME, icons::handle)
        .invoke_handler(tauri::generate_handler![
            commands::detect_game_dir,
            commands::open_game,
            commands::open_map,
            commands::list_units,
            commands::unit_detail,
            commands::search_units,
            commands::set_field,
            commands::reset_field,
            commands::create_unit,
            commands::save_map,
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("WC3 Object Editor could not start: {e}");
            ExitCode::FAILURE
        }
    }
}
