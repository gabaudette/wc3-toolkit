use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use wc3_data::abilities::AbilityDatabase;
use wc3_data::doodads::DoodadDatabase;
use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::guide::FieldGuide;
use wc3_data::meta::FieldMeta;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::abilities::MapAbilities;
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, MapStrings, save_changes};

use crate::abilities::{AbilityCard, AbilityDetail, AbilitySummary, AbilityView};
use crate::hotkeys::{self, Clash};
use crate::interface;
use crate::loot;
use crate::platform;
use crate::state::{AppState, NO_MAP, OpenMap, Session};
use crate::view::{Field, SearchHit, UnitDetail, UnitItem, View};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub dir: String,
    pub units: usize,
    pub maps_dir: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSummary {
    pub path: String,
    pub name: String,
    pub unsaved: bool,
}

pub(crate) fn fourcc(code: &str) -> Result<FourCC, String> {
    FourCC::parse(code).ok_or(format!("“{code}” is not a four-character code."))
}

pub(crate) fn view(session: &Session) -> View<'_> {
    View {
        db: &session.db,
        guide: &session.guide,
        map: session.map.as_ref().map(|m| &m.units),
        mode: session.mode,
    }
}

pub(crate) fn with_abilities<T>(session: &Session, f: impl FnOnce(&AbilityView<'_>) -> T) -> T {
    let unchanged;
    let (map, strings) = match &session.map {
        Some(open) => (&open.abilities, open.units.strings()),
        None => {
            unchanged = (MapAbilities::empty(), MapStrings::default());
            (&unchanged.0, &unchanged.1)
        }
    };

    f(&AbilityView {
        db: &session.abilities,
        units: &session.db,
        guide: &session.ability_guide,
        map,
        strings,
        mode: session.mode,
    })
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[tauri::command]
pub fn detect_game_dir() -> Option<String> {
    platform::detect_game_dir().map(|p| path_text(&p))
}

#[tauri::command]
pub fn open_game(state: State<'_, AppState>, dir: String, hd: bool) -> Result<GameSummary, String> {
    let dir = PathBuf::from(dir);

    if !platform::is_game_dir(&dir) {
        return Err(format!(
            "{} is not a Warcraft III folder (it has no Data folder).",
            dir.display()
        ));
    }

    let files = GameFiles::open(&dir, GameOptions::default()).map_err(|e| e.to_string())?;
    let db = UnitDatabase::load(&files).map_err(|e| e.to_string())?;
    let guide = FieldGuide::units().map_err(|e| e.to_string())?;
    let abilities = AbilityDatabase::load(&files).map_err(|e| e.to_string())?;
    let ability_guide = FieldGuide::abilities().map_err(|e| e.to_string())?;
    let items = UnitDatabase::load_items(&files).map_err(|e| e.to_string())?;
    let item_guide = FieldGuide::items().map_err(|e| e.to_string())?;
    let buffs = UnitDatabase::load_buffs(&files).map_err(|e| e.to_string())?;
    let buff_guide = FieldGuide::buffs().map_err(|e| e.to_string())?;
    let destructables = UnitDatabase::load_destructables(&files).map_err(|e| e.to_string())?;
    let destructable_guide = FieldGuide::destructables().map_err(|e| e.to_string())?;
    let doodads = DoodadDatabase::load(&files).map_err(|e| e.to_string())?;
    let doodad_guide = FieldGuide::doodads().map_err(|e| e.to_string())?;
    let upgrades = UpgradeDatabase::load(&files).map_err(|e| e.to_string())?;
    let upgrade_guide = FieldGuide::upgrades().map_err(|e| e.to_string())?;
    let summary = GameSummary {
        dir: path_text(&dir),
        units: db.ids().len(),
        maps_dir: platform::maps_dir(&dir).map(|p| path_text(&p)),
    };

    let mode = if hd { ArtMode::Hd } else { ArtMode::Sd };

    state.replace(Session {
        files: Arc::new(files),
        db,
        guide,
        abilities,
        ability_guide,
        items,
        item_guide,
        buffs,
        buff_guide,
        destructables,
        destructable_guide,
        doodads,
        doodad_guide,
        upgrades,
        upgrade_guide,
        mode,
        map: None,
        icons: HashMap::new(),
    })?;

    Ok(summary)
}

#[tauri::command]
pub fn open_map(state: State<'_, AppState>, path: String) -> Result<MapSummary, String> {
    let open = OpenMap::load(PathBuf::from(path)).map_err(|e| e.to_string())?;

    state.with(|session| {
        session.map = Some(open);
        Ok(map_summary(session))
    })
}

pub(crate) fn map_summary(session: &Session) -> MapSummary {
    let map = session.map.as_ref();

    MapSummary {
        path: map.map(|m| path_text(&m.path)).unwrap_or_default(),
        name: map
            .and_then(|m| m.path.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        unsaved: map.is_some_and(OpenMap::unsaved),
    }
}

#[tauri::command]
pub fn map_status(state: State<'_, AppState>) -> Result<MapSummary, String> {
    state.with(|session| Ok(map_summary(session)))
}

#[tauri::command]
pub fn list_units(state: State<'_, AppState>) -> Result<Vec<UnitItem>, String> {
    state.with(|session| Ok(view(session).items()))
}

#[tauri::command]
pub fn unit_detail(state: State<'_, AppState>, id: String) -> Result<UnitDetail, String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        view(session)
            .detail(id)
            .ok_or(format!("Unknown unit {id}."))
    })
}

#[tauri::command]
pub fn list_abilities(state: State<'_, AppState>) -> Result<Vec<AbilitySummary>, String> {
    state.with(|session| Ok(with_abilities(session, |abilities| abilities.list())))
}

#[tauri::command]
pub fn ability_cards(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Vec<AbilityCard>, String> {
    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            ids.iter().map(|id| abilities.card(id.trim())).collect()
        }))
    })
}

#[tauri::command]
pub fn ability_detail(state: State<'_, AppState>, id: String) -> Result<AbilityDetail, String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        with_abilities(session, |abilities| abilities.detail(id))
            .ok_or(format!("Unknown ability {id}."))
    })
}

#[tauri::command]
pub fn search_abilities(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            abilities.search(&query)
        }))
    })
}

#[tauri::command]
pub fn hotkey_clashes(state: State<'_, AppState>, id: String) -> Result<Vec<Clash>, String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            hotkeys::clashes(&view(session), &|ability| abilities.info(ability), id)
        }))
    })
}

#[tauri::command]
pub fn fill_placeholders(state: State<'_, AppState>, text: String) -> Result<String, String> {
    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            abilities.fill_placeholders(&text)
        }))
    })
}

#[tauri::command]
pub fn search_units(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(view(session).search(&query)))
}

fn edit_field(
    state: &AppState,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &UnitDatabase,
        &mut MapUnits,
        FourCC,
        &wc3_data::meta::FieldMeta,
    ) -> Result<(), String>,
) -> Result<Field, String> {
    let id = fourcc(id)?;

    state.with(|session| {
        let field = session
            .db
            .field(field_id)
            .ok_or(format!("Unknown field {field_id}."))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        change(&session.db, &mut map.units, id, field)?;
        Ok(view(session).field(id, field))
    })
}

#[tauri::command]
pub fn set_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    value: String,
) -> Result<Field, String> {
    edit_field(&state, &id, &field, |db, units, id, meta| {
        units.set(db, id, meta, &value).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn reset_field(state: State<'_, AppState>, id: String, field: String) -> Result<Field, String> {
    edit_field(&state, &id, &field, |_, units, id, meta| {
        units.reset(id, meta);
        Ok(())
    })
}

#[tauri::command]
pub fn create_unit(state: State<'_, AppState>, base: String) -> Result<UnitItem, String> {
    let base = fourcc(&base)?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let id = map
            .units
            .create(&session.db, base)
            .map_err(|e| e.to_string())?;

        view(session)
            .item(id)
            .ok_or(format!("Created {id} but cannot show it."))
    })
}

fn edit_ability(
    state: &AppState,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &AbilityDatabase,
        &mut MapAbilities,
        &mut MapStrings,
        FourCC,
        &FieldMeta,
    ) -> Result<(), String>,
) -> Result<AbilityDetail, String> {
    let id = fourcc(id)?;

    state.with(|session| {
        let field = session
            .abilities
            .field(field_id)
            .ok_or(format!("Unknown field {field_id}."))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        change(
            &session.abilities,
            &mut map.abilities,
            map.units.strings_mut(),
            id,
            field,
        )?;
        with_abilities(session, |abilities| abilities.detail(id))
            .ok_or(format!("Unknown ability {id}."))
    })
}

#[tauri::command]
pub fn set_ability_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    level: u32,
    value: String,
) -> Result<AbilityDetail, String> {
    edit_ability(&state, &id, &field, |db, abilities, strings, id, meta| {
        abilities
            .set(db, strings, id, meta, level, &value)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn reset_ability_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    level: u32,
) -> Result<AbilityDetail, String> {
    edit_ability(&state, &id, &field, |_, abilities, strings, id, meta| {
        abilities.reset(strings, id, meta, level);
        Ok(())
    })
}

#[tauri::command]
pub fn create_ability(state: State<'_, AppState>, base: String) -> Result<AbilitySummary, String> {
    let base = fourcc(&base)?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let id = map
            .abilities
            .create(&session.abilities, base)
            .map_err(|e| e.to_string())?;

        with_abilities(session, |abilities| abilities.summary(id))
            .ok_or(format!("Created {id} but cannot show it."))
    })
}

#[tauri::command]
pub fn save_map(state: State<'_, AppState>) -> Result<MapSummary, String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let files = map.files();
        let imported = map.imports.files();
        let mut written = files.clone();

        written.extend(
            imported
                .iter()
                .map(|(path, bytes)| (path.as_str(), bytes.clone())),
        );
        let removed = map.imports.removed();
        let removed: Vec<&str> = removed.iter().map(String::as_str).collect();

        save_changes(&map.path, &map.path, &written, &removed).map_err(|e| e.to_string())?;
        if map.ui != map.saved_ui {
            interface::save(&map.path, &map.ui)?;
            map.saved_ui = map.ui.clone();
        }

        if map.cascades != map.saved_cascades {
            loot::save(&map.path, &map.cascades)?;
            map.saved_cascades = map.cascades.clone();
        }

        map.saved_files = files;
        map.imports.saved();
        Ok(map_summary(session))
    })
}
