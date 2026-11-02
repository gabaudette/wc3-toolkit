use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_map::export::{self, CsvValues, Format, GameData, LoadedMap, MapData, Selection};

use crate::state::{AppState, OpenMap, Session};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub selection: Selection,
    pub format: Format,
    pub values: CsvValues,
    pub path: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub objects: usize,
    pub files: Vec<String>,
}

fn map_data(map: &OpenMap) -> MapData<'_> {
    MapData {
        units: &map.units,
        items: &map.items,
        buffs: &map.buffs,
        destructables: &map.destructables,
        abilities: &map.abilities,
        upgrades: &map.upgrades,
        doodads: &map.doodads,
    }
}

fn file_name(path: &Path) -> Option<String> {
    path.file_name().map(|n| n.to_string_lossy().into_owned())
}

fn exported(session: &Session, selection: &Selection) -> export::Export {
    let game = GameData {
        units: &session.db,
        items: &session.items,
        buffs: &session.buffs,
        destructables: &session.destructables,
        abilities: &session.abilities,
        upgrades: &session.upgrades,
        doodads: &session.doodads,
        mode: session.mode,
    };

    match &session.map {
        Some(map) => export::export(&game, &map_data(map), file_name(&map.path), selection),
        None => export::export(&game, &LoadedMap::empty().data(), None, selection),
    }
}

#[tauri::command]
pub fn export_objects(
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<ExportResult, String> {
    let export = state.with(|session| Ok(exported(session, &request.selection)))?;
    let files = export::write(&export, request.format, request.values, &request.path)
        .map_err(|e| e.to_string())?;

    Ok(ExportResult {
        objects: export.object_count(),
        files: files
            .iter()
            .map(|f| f.to_string_lossy().into_owned())
            .collect(),
    })
}
