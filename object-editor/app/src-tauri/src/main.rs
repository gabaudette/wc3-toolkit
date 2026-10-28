#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod abilities;
mod assets;
mod commands;
mod hotkeys;
mod icons;
mod interface;
mod leveled;
mod log;
mod loot;
mod mapinfo;
mod plain;
mod platform;
mod spells;
mod state;
mod transfer;
mod view;

use std::process::ExitCode;

fn main() -> ExitCode {
    platform::keep_threads_at_normal_priority();
    let context = tauri::generate_context!();
    log::start(&context.config().identifier);

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
            spells::spell_kinds,
            spells::discard_spell,
            spells::spell_icons,
            spells::spell_units,
            spells::spell_fit,
            spells::give_spell,
            spells::take_spell,
            spells::spell_templates,
            spells::create_triggered_spell,
            spells::set_triggered_values,
            spells::triggered_spell_lua,
            spells::write_triggered_spell_lua,
            spells::default_lua_folder,
            spells::build_test_copy,
            interface::ui_designs,
            interface::set_ui_designs,
            interface::ui_lua,
            interface::write_ui_lua,
            interface::ui_export_preview,
            interface::save_ui_file,
            interface::ui_import_preview,
            interface::import_ui_file,
            plain::list_plain,
            plain::plain_detail,
            plain::search_plain,
            plain::set_plain_field,
            plain::reset_plain_field,
            plain::create_plain,
            plain::discard_plain,
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
            log::log_failure,
            mapinfo::map_info,
            mapinfo::set_map_info,
            mapinfo::gameplay_constants,
            mapinfo::set_gameplay_constant,
            mapinfo::set_custom_constants,
            loot::list_placed_units,
            loot::set_placed_inventory,
            loot::set_placed_drops,
            loot::list_item_tables,
            loot::save_item_table,
            loot::remove_item_table,
            loot::item_choices,
            assets::list_assets,
            assets::asset_textures,
            assets::plan_import,
            assets::read_text_file,
            assets::import_assets,
            assets::remove_asset,
            assets::replace_asset,
            assets::rename_asset,
            assets::export_asset,
            assets::export_game_file,
        ])
        .run(context);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::record(
                log::Level::Error,
                "startup",
                &format!("Tinkerbench could not start: {e}"),
            );
            ExitCode::FAILURE
        }
    }
}
