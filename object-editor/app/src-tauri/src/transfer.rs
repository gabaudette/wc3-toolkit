//! Commands that move custom objects between maps: share codes, bundle files
//! (`.w3obj`, which also carry imported models and icons) and imports from
//! another map file. The work happens in `wc3_map::transfer`; this module names
//! the objects for the window and picks where they come from.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_map::abilities::MapAbilities;
use wc3_map::imports::{FileStatus, ImportedFile, file_statuses};
use wc3_map::items::MapItems;
use wc3_map::transfer::{
    self, Bundle, BundledObject, Export, MapExtras, MapObjects, MapObjectsMut, ObjectType,
    PlannedObject, References,
};
use wc3_map::units::MapUnits;
use wc3_map::upgrades::MapUpgrades;
use wc3_map::{FourCC, Map};

use crate::commands::{fourcc, view};
use crate::state::{AppState, NO_MAP, Session};
use crate::view::UnitItem;

const UNIT_NAME: &str = "unam";
const UNIT_ICON: &str = "uico";
const ABILITY_NAME: &str = "anam";
const ABILITY_ICON: &str = "aart";
const BUFF_NAME: &str = "fnam";
const BUFF_ICON: &str = "fart";
const ITEM_NAME: &str = "unam";
const ITEM_ICON: &str = "iico";
const UPGRADE_NAME: &str = "gnam";
const UPGRADE_ICON: &str = "gar1";

/// An object about to move, as the window lists it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferObject {
    /// `unit`, `ability` or `buff`.
    pub kind: &'static str,
    pub id: String,
    /// The ID it gets in this map, when importing.
    pub new_id: Option<String>,
    pub base: String,
    pub name: String,
    pub icon: Option<String>,
    /// Brought along because a chosen object uses it.
    pub dependency: bool,
}

/// A file imported into a map (model, texture, icon) that travels with objects.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFile {
    pub path: String,
    pub size: usize,
    /// When importing: `new`, `same` (the map has this file already) or
    /// `different` (the map has another file at that path and keeps it).
    pub status: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePreview {
    pub code: String,
    pub objects: Vec<TransferObject>,
    /// Imported files the objects use: in bundle files, not in the code.
    pub files: Vec<TransferFile>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub objects: Vec<TransferObject>,
    pub files: Vec<TransferFile>,
    pub warnings: Vec<String>,
}

/// Where imported objects come from.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportSource {
    Code { code: String },
    Bundle { path: String },
    Map { path: String, ids: Vec<String> },
}

/// Another map's objects, read before taking the session lock, and the map
/// itself for reading its imported files.
struct OtherMap {
    map: Map,
    units: MapUnits,
    abilities: MapAbilities,
    items: MapItems,
    upgrades: MapUpgrades,
    extras: MapExtras,
}

impl OtherMap {
    fn load(path: &str) -> Result<Self, String> {
        let map = Map::open(&PathBuf::from(path)).map_err(|e| e.to_string())?;
        Ok(Self {
            units: MapUnits::load(&map).map_err(|e| e.to_string())?,
            abilities: MapAbilities::load(&map).map_err(|e| e.to_string())?,
            items: MapItems::load(&map).map_err(|e| e.to_string())?,
            upgrades: MapUpgrades::load(&map).map_err(|e| e.to_string())?,
            extras: MapExtras::load(&map).map_err(|e| e.to_string())?,
            map,
        })
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.map.read_file(path).ok().flatten()
    }

    fn objects(&self) -> MapObjects<'_> {
        MapObjects {
            units: &self.units,
            abilities: &self.abilities,
            items: &self.items,
            upgrades: &self.upgrades,
            extras: &self.extras,
        }
    }
}

fn references(session: &Session) -> References {
    References::new(
        session.db.fields(),
        session.abilities.fields(),
        session.items.fields(),
        session.upgrades.fields(),
    )
}

fn kind_name(ty: ObjectType) -> &'static str {
    match ty {
        ObjectType::Unit => "unit",
        ObjectType::Ability => "ability",
        ObjectType::Buff => "buff",
        ObjectType::Item => "item",
        ObjectType::Upgrade => "upgrade",
    }
}

/// Names an object from its own fields, falling back to its base object.
fn describe(
    session: &Session,
    bundle: &Bundle,
    object: &BundledObject,
    new_id: Option<FourCC>,
    chosen: &[FourCC],
) -> TransferObject {
    let base = object.base.to_string();
    let own = |field: &str| {
        bundle
            .text(object.ty, object.id, field)
            .filter(|t| !t.is_empty())
    };
    let (name, icon) = match object.ty {
        ObjectType::Unit => (
            own(UNIT_NAME).unwrap_or_else(|| session.db.name(&base)),
            own(UNIT_ICON).or_else(|| session.db.icon_path(&base, session.mode)),
        ),
        ObjectType::Ability => {
            let info = session.abilities.info(&base);
            (
                own(ABILITY_NAME)
                    .or_else(|| info.as_ref().map(|i| i.name.clone()))
                    .unwrap_or_else(|| base.clone()),
                own(ABILITY_ICON).or_else(|| info.and_then(|i| i.icon)),
            )
        }
        ObjectType::Buff => (
            own(BUFF_NAME).unwrap_or_else(|| base.clone()),
            own(BUFF_ICON),
        ),
        ObjectType::Item => (
            own(ITEM_NAME).unwrap_or_else(|| session.items.name(&base)),
            own(ITEM_ICON).or_else(|| session.items.icon_path(&base, session.mode)),
        ),
        ObjectType::Upgrade => (
            own(UPGRADE_NAME).unwrap_or_else(|| session.upgrades.name(&base)),
            own(UPGRADE_ICON).or_else(|| session.upgrades.icon_path(&base, session.mode)),
        ),
    };

    TransferObject {
        kind: kind_name(object.ty),
        id: object.id.to_string(),
        new_id: new_id.map(|id| id.to_string()),
        base,
        name,
        icon,
        dependency: !(object.ty == ObjectType::Unit && chosen.contains(&object.id)),
    }
}

fn parse_ids(ids: &[String]) -> Result<Vec<FourCC>, String> {
    if ids.is_empty() {
        return Err("Pick at least one unit.".to_owned());
    }
    ids.iter().map(|id| fourcc(id)).collect()
}

/// Bundles the open map's chosen custom units, what they use, and the
/// imported files among that.
fn export_open_map(session: &Session, chosen: &[FourCC]) -> Result<Export, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    let source = MapObjects {
        units: &map.units,
        abilities: &map.abilities,
        items: &map.items,
        upgrades: &map.upgrades,
        extras: &map.extras,
    };
    transfer::export(source, &references(session), chosen, &map.reader()).map_err(|e| e.to_string())
}

fn listed_files(files: &[ImportedFile], statuses: Option<&[FileStatus]>) -> Vec<TransferFile> {
    files
        .iter()
        .enumerate()
        .map(|(i, file)| TransferFile {
            path: file.path.clone(),
            size: file.bytes.len(),
            status: statuses.and_then(|s| s.get(i)).map(|status| match status {
                FileStatus::New => "new",
                FileStatus::Same => "same",
                FileStatus::Different => "different",
            }),
        })
        .collect()
}

/// A share code for the open map's chosen custom units and what they use.
#[tauri::command]
pub fn share_code(state: State<'_, AppState>, ids: Vec<String>) -> Result<SharePreview, String> {
    let chosen = parse_ids(&ids)?;
    state.with(|session| {
        let Export {
            bundle,
            objects,
            mut warnings,
        } = export_open_map(session, &chosen)?;
        let count = bundle.files().len();
        if count > 0 {
            let files = match count {
                1 => "1 imported file (model, texture or icon) is".to_owned(),
                n => format!("{n} imported files (models, textures, icons) are"),
            };
            warnings.insert(
                0,
                format!(
                    "{files} too large for a share code. Save a bundle file to bring them along."
                ),
            );
        }

        Ok(SharePreview {
            code: bundle.to_code(),
            objects: objects
                .iter()
                .map(|o| describe(session, &bundle, o, None, &chosen))
                .collect(),
            files: listed_files(bundle.files(), None),
            warnings,
        })
    })
}

/// Saves the chosen units, what they use and their imported files as a
/// bundle file (`.w3obj`).
#[tauri::command]
pub fn save_bundle(
    state: State<'_, AppState>,
    ids: Vec<String>,
    path: String,
) -> Result<(), String> {
    let chosen = parse_ids(&ids)?;
    let bytes = state.with(|session| Ok(export_open_map(session, &chosen)?.bundle.to_file()))?;
    std::fs::write(&path, bytes).map_err(|e| format!("Cannot write {path}: {e}"))
}

/// Another map's custom units, to pick what to import.
#[tauri::command]
pub fn map_custom_units(state: State<'_, AppState>, path: String) -> Result<Vec<UnitItem>, String> {
    let other = OtherMap::load(&path)?;
    state.with(|session| {
        let mut shown = view(session);
        shown.map = Some(&other.units);
        Ok(shown.items().into_iter().filter(|u| u.custom).collect())
    })
}

/// A code or file does not say which units were picked: all its units count.
fn bundled_units(bundle: &Bundle) -> Vec<FourCC> {
    bundle
        .objects()
        .into_iter()
        .filter(|o| o.ty == ObjectType::Unit)
        .map(|o| o.id)
        .collect()
}

/// How each bundled file compares with the open map's file at that path.
fn statuses(session: &Session, bundle: &Bundle) -> Result<Vec<FileStatus>, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    Ok(file_statuses(bundle.files(), &map.reader()))
}

/// The bundle a source holds, the units chosen in it, and warnings.
fn bundle_from(
    session: &Session,
    source: &ImportSource,
    other: Option<&OtherMap>,
) -> Result<(Bundle, Vec<FourCC>, Vec<String>), String> {
    match (source, other) {
        (ImportSource::Code { code }, _) => {
            let bundle = Bundle::from_code(code).map_err(|e| e.to_string())?;
            let units = bundled_units(&bundle);
            Ok((bundle, units, Vec::new()))
        }
        (ImportSource::Bundle { path }, _) => {
            let bytes = std::fs::read(path).map_err(|e| format!("Cannot read {path}: {e}"))?;
            let bundle = Bundle::from_file(&bytes).map_err(|e| e.to_string())?;
            let units = bundled_units(&bundle);
            Ok((bundle, units, Vec::new()))
        }
        (ImportSource::Map { ids, .. }, Some(other)) => {
            let chosen = parse_ids(ids)?;
            let export =
                transfer::export(other.objects(), &references(session), &chosen, &|path| {
                    other.read(path)
                })
                .map_err(|e| e.to_string())?;
            Ok((export.bundle, chosen, export.warnings))
        }
        (ImportSource::Map { .. }, None) => Err("The other map is not loaded.".to_owned()),
    }
}

fn load_other(source: &ImportSource) -> Result<Option<OtherMap>, String> {
    match source {
        ImportSource::Map { path, .. } => OtherMap::load(path).map(Some),
        ImportSource::Code { .. } | ImportSource::Bundle { .. } => Ok(None),
    }
}

/// Where each object would go in the open map.
fn planned(session: &Session, bundle: &Bundle) -> Result<Vec<PlannedObject>, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    let target = MapObjects {
        units: &map.units,
        abilities: &map.abilities,
        items: &map.items,
        upgrades: &map.upgrades,
        extras: &map.extras,
    };
    let in_game = |ty: ObjectType, id: FourCC| match ty {
        ObjectType::Unit => session.db.contains(&id.to_string()),
        ObjectType::Ability => session.abilities.contains(&id.to_string()),
        ObjectType::Item => session.items.contains(&id.to_string()),
        ObjectType::Upgrade => session.upgrades.contains(&id.to_string()),
        // Game buffs use letter codes (`BHbd`); custom ones use digits (`B000`).
        ObjectType::Buff => false,
    };
    transfer::plan(bundle, target, &in_game).map_err(|e| e.to_string())
}

fn listed(
    session: &Session,
    bundle: &Bundle,
    plan: &[PlannedObject],
    chosen: &[FourCC],
) -> Vec<TransferObject> {
    plan.iter()
        .map(|p| {
            let object = BundledObject {
                ty: p.ty,
                id: p.id,
                base: p.base,
            };
            describe(session, bundle, &object, Some(p.new_id), chosen)
        })
        .collect()
}

/// What an import would add to the open map, without changing it.
#[tauri::command]
pub fn preview_import(
    state: State<'_, AppState>,
    source: ImportSource,
) -> Result<ImportPreview, String> {
    let other = load_other(&source)?;
    state.with(|session| {
        let (bundle, chosen, warnings) = bundle_from(session, &source, other.as_ref())?;
        let plan = planned(session, &bundle)?;
        let statuses = statuses(session, &bundle)?;
        let mut warnings = warnings;
        for (file, status) in bundle.files().iter().zip(&statuses) {
            if *status == FileStatus::Different {
                warnings.push(format!(
                    "This map already has a different {}; it keeps its own.",
                    file.path
                ));
            }
        }

        Ok(ImportPreview {
            objects: listed(session, &bundle, &plan, &chosen),
            files: listed_files(bundle.files(), Some(&statuses)),
            warnings,
        })
    })
}

/// Adds the objects, and the imported files the map does not have yet, to the
/// open map. Nothing is written until the map is saved.
#[tauri::command]
pub fn import_objects(
    state: State<'_, AppState>,
    source: ImportSource,
) -> Result<Vec<TransferObject>, String> {
    let other = load_other(&source)?;
    state.with(|session| {
        let (bundle, chosen, _) = bundle_from(session, &source, other.as_ref())?;
        let plan = planned(session, &bundle)?;
        let refs = references(session);
        let added = listed(session, &bundle, &plan, &chosen);
        let statuses = statuses(session, &bundle)?;

        let map = session.map.as_mut().ok_or(NO_MAP)?;
        for (file, status) in bundle.files().iter().zip(statuses) {
            if status == FileStatus::New {
                map.imports.add(file.clone());
            }
        }
        transfer::apply(
            &bundle,
            &plan,
            &refs,
            MapObjectsMut {
                units: &mut map.units,
                abilities: &mut map.abilities,
                items: &mut map.items,
                upgrades: &mut map.upgrades,
                extras: &mut map.extras,
            },
        );
        Ok(added)
    })
}
