use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_map::abilities::MapAbilities;
use wc3_map::imports::{FileStatus, ImportedFile, file_statuses};
use wc3_map::leveled::{LeveledObjects, UPGRADES};
use wc3_map::plain::{BUFFS, ITEMS, PlainObjects};
use wc3_map::transfer::{
    self, Bundle, BundledObject, Export, MapObjects, MapObjectsMut, ObjectType, PlannedObject,
    References,
};
use wc3_map::units::MapUnits;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferObject {
    pub kind: &'static str,
    pub id: String,
    pub new_id: Option<String>,
    pub base: String,
    pub name: String,
    pub icon: Option<String>,
    pub dependency: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFile {
    pub path: String,
    pub size: usize,
    pub status: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePreview {
    pub code: String,
    pub objects: Vec<TransferObject>,
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

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportSource {
    Code { code: String },
    Bundle { path: String },
    Map { path: String, ids: Vec<String> },
}

struct OtherMap {
    map: Map,
    units: MapUnits,
    abilities: MapAbilities,
    items: PlainObjects,
    upgrades: LeveledObjects,
    buffs: PlainObjects,
}

impl OtherMap {
    fn load(path: &str) -> Result<Self, String> {
        let map = Map::open(&PathBuf::from(path)).map_err(|e| e.to_string())?;

        Ok(Self {
            units: MapUnits::load(&map).map_err(|e| e.to_string())?,
            abilities: MapAbilities::load(&map).map_err(|e| e.to_string())?,
            items: PlainObjects::load(&map, ITEMS).map_err(|e| e.to_string())?,
            upgrades: LeveledObjects::load(&map, UPGRADES).map_err(|e| e.to_string())?,
            buffs: PlainObjects::load(&map, BUFFS).map_err(|e| e.to_string())?,
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
            buffs: &self.buffs,
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
            own(BUFF_NAME).unwrap_or_else(|| session.buffs.name(&base)),
            own(BUFF_ICON).or_else(|| session.buffs.icon_path(&base, session.mode)),
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

fn export_open_map(session: &Session, chosen: &[FourCC]) -> Result<Export, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    let source = MapObjects {
        units: &map.units,
        abilities: &map.abilities,
        items: &map.items,
        upgrades: &map.upgrades,
        buffs: &map.buffs,
    };
    transfer::export(source, &references(session), chosen, &map.reader()).map_err(|e| e.to_string())
}

pub(crate) fn listed_files(
    files: &[ImportedFile],
    statuses: Option<&[FileStatus]>,
) -> Vec<TransferFile> {
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

#[tauri::command]
pub fn map_custom_units(state: State<'_, AppState>, path: String) -> Result<Vec<UnitItem>, String> {
    let other = OtherMap::load(&path)?;

    state.with(|session| {
        let mut shown = view(session);

        shown.map = Some(&other.units);
        Ok(shown.items().into_iter().filter(|u| u.custom).collect())
    })
}

fn bundled_units(bundle: &Bundle) -> Vec<FourCC> {
    bundle
        .objects()
        .into_iter()
        .filter(|o| o.ty == ObjectType::Unit)
        .map(|o| o.id)
        .collect()
}

fn statuses(session: &Session, bundle: &Bundle) -> Result<Vec<FileStatus>, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;

    Ok(file_statuses(bundle.files(), &map.reader()))
}

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

fn planned(session: &Session, bundle: &Bundle) -> Result<Vec<PlannedObject>, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    let target = MapObjects {
        units: &map.units,
        abilities: &map.abilities,
        items: &map.items,
        upgrades: &map.upgrades,
        buffs: &map.buffs,
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
                buffs: &mut map.buffs,
            },
        );
        Ok(added)
    })
}
