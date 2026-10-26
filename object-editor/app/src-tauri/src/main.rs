//! WC3 Object Editor: a desktop editor for Warcraft III map object data.

// Release builds on Windows run without a console window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Startup failures are reported on stderr (AGE-QUAL-003: documented, file-scoped).
#![allow(clippy::print_stderr)]

mod abilities;
mod commands;
mod hotkeys;
mod icons;
mod leveled;
mod plain;
mod platform;
mod state;
mod transfer;
mod view;

use std::process::ExitCode;

fn main() -> ExitCode {
    platform::keep_threads_at_normal_priority();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .register_asynchronous_uri_scheme_protocol(icons::ICON_SCHEME, icons::handle_icon)
        .register_asynchronous_uri_scheme_protocol(icons::TEXTURE_SCHEME, icons::handle_texture)
        .register_asynchronous_uri_scheme_protocol(icons::MODEL_SCHEME, icons::handle_model)
        .invoke_handler(tauri::generate_handler![
            commands::detect_game_dir,
            commands::open_game,
            commands::open_map,
            commands::map_status,
            commands::list_units,
            commands::unit_detail,
            commands::search_units,
            commands::list_abilities,
            commands::ability_cards,
            commands::ability_detail,
            commands::search_abilities,
            commands::set_ability_field,
            commands::reset_ability_field,
            commands::create_ability,
            commands::fill_placeholders,
            commands::hotkey_clashes,
            commands::set_field,
            commands::reset_field,
            commands::create_unit,
            commands::save_map,
            plain::list_plain,
            plain::plain_detail,
            plain::search_plain,
            plain::set_plain_field,
            plain::reset_plain_field,
            plain::create_plain,
            leveled::list_leveled,
            leveled::leveled_detail,
            leveled::search_leveled,
            leveled::set_leveled_field,
            leveled::reset_leveled_field,
            leveled::create_leveled,
            transfer::share_code,
            transfer::save_bundle,
            transfer::map_custom_units,
            transfer::preview_import,
            transfer::import_objects,
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
