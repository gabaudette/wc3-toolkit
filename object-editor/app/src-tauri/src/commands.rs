//! Commands the window calls. Errors come back as messages the UI shows.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;
use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::units::UnitDatabase;
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, Map, save_files};

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

fn fourcc(code: &str) -> Result<FourCC, String> {
    FourCC::parse(code).ok_or(format!("“{code}” is not a four-character code."))
}

fn view(session: &Session) -> View<'_> {
    View {
        db: &session.db,
        map: session.map.as_ref().map(|m| &m.units),
        mode: session.mode,
    }
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
    let summary = GameSummary {
        dir: path_text(&dir),
        units: db.ids().len(),
        maps_dir: platform::maps_dir(&dir).map(|p| path_text(&p)),
    };

    let mode = if hd { ArtMode::Hd } else { ArtMode::Sd };
    state.replace(Session {
        files,
        db,
        mode,
        map: None,
        icons: HashMap::new(),
    })?;
    Ok(summary)
}

#[tauri::command]
pub fn open_map(state: State<'_, AppState>, path: String) -> Result<MapSummary, String> {
    let path = PathBuf::from(path);
    let map = Map::open(&path).map_err(|e| e.to_string())?;
    let units = MapUnits::load(&map).map_err(|e| e.to_string())?;
    drop(map);

    state.with(|session| {
        session.map = Some(OpenMap::new(path.clone(), units));
        Ok(map_summary(session))
    })
}

fn map_summary(session: &Session) -> MapSummary {
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
pub fn search_units(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(view(session).search(&query)))
}

/// Applies `change` to the open map and returns the field's new state.
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

/// Saves into the open map file through a temporary copy.
#[tauri::command]
pub fn save_map(state: State<'_, AppState>) -> Result<MapSummary, String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let files = map.units.files();
        save_files(&map.path, &map.path, &files).map_err(|e| e.to_string())?;
        map.saved_files = files;
        Ok(map_summary(session))
    })
}
