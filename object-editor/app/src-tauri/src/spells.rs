//! The spell wizard: spell kinds with their suggested base abilities, and
//! removing a spell the wizard created when it is cancelled. The spell itself
//! is an ordinary custom ability, created and edited with the ability commands.

use serde::Serialize;
use tauri::State;
use wc3_data::spells::spell_kinds as kinds;

use crate::abilities::AbilityCard;
use crate::commands::{fourcc, with_abilities};
use crate::state::{AppState, NO_MAP, Session};

/// Images a map can import as icons.
const IMAGE_EXTENSIONS: &[&str] = &["blp", "dds", "tga"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpellKind {
    pub id: String,
    pub label: String,
    pub description: String,
    pub bases: Vec<SpellBase>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpellBase {
    pub card: AbilityCard,
    /// What it brings, and when to pick it over the others.
    pub why: String,
}

/// An icon the spell wizard offers for a spell's button.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconChoice {
    /// As data files name it, e.g. `ReplaceableTextures\CommandButtons\BTNStormBolt.blp`.
    pub path: String,
    /// Its file name without folders or extension, e.g. `BTNStormBolt`, for searching.
    pub name: String,
    /// Imported into the open map rather than from the game.
    pub from_map: bool,
}

/// The file name of a path without its folders and extension.
fn file_stem(path: &str) -> &str {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// Every icon the game's units, abilities, items and buffs use, and every
/// image the open map imports (first), each once.
fn icon_choices(session: &Session) -> Vec<IconChoice> {
    let mut seen = std::collections::HashSet::new();
    let mut choices = Vec::new();
    let mut add = |path: String, from_map: bool| {
        if seen.insert(path.to_ascii_lowercase().replace('/', "\\")) {
            choices.push(IconChoice {
                name: file_stem(&path).to_owned(),
                path,
                from_map,
            });
        }
    };

    if let Some(map) = &session.map {
        for path in map.imports.paths() {
            let extension = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
            if extension.is_some_and(|e| IMAGE_EXTENSIONS.contains(&e.as_str())) {
                add(path, true);
            }
        }
    }

    let mode = session.mode;
    for id in session.abilities.ids() {
        if let Some(icon) = session.abilities.info(id).and_then(|info| info.icon) {
            add(icon, false);
        }
    }
    for db in [&session.db, &session.items, &session.buffs] {
        for id in db.ids() {
            if let Some(icon) = db.icon_path(id, mode) {
                add(icon, false);
            }
        }
    }

    choices.sort_by(|a, b| {
        b.from_map.cmp(&a.from_map).then_with(|| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        })
    });
    choices
}

/// Icons for the spell wizard's button step.
#[tauri::command]
pub fn spell_icons(state: State<'_, AppState>) -> Result<Vec<IconChoice>, String> {
    state.with(|session| Ok(icon_choices(session)))
}

/// The wizard's first step: what a spell can do, each with its suggested bases.
#[tauri::command]
pub fn spell_kinds(state: State<'_, AppState>) -> Result<Vec<SpellKind>, String> {
    let kinds = kinds().map_err(|e| e.to_string())?;
    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            kinds
                .into_iter()
                .map(|kind| SpellKind {
                    id: kind.id,
                    label: kind.label,
                    description: kind.description,
                    bases: kind
                        .bases
                        .into_iter()
                        .map(|base| SpellBase {
                            card: abilities.card(&base.code),
                            why: base.why,
                        })
                        .collect(),
                })
                .collect()
        }))
    })
}

/// Removes a custom ability the wizard created, when the wizard is cancelled
/// or goes back to pick another base. Game abilities are never removed.
#[tauri::command]
pub fn discard_spell(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = fourcc(&id)?;
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        if map.abilities.remove(map.units.strings_mut(), id) {
            Ok(())
        } else {
            Err(format!("{id} is not a custom ability of this map."))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::file_stem;

    #[test]
    fn icon_names_drop_folders_and_extensions() {
        assert_eq!(
            file_stem("ReplaceableTextures\\CommandButtons\\BTNStormBolt.blp"),
            "BTNStormBolt"
        );
        assert_eq!(file_stem("war3mapImported/BTNFire.v2.dds"), "BTNFire.v2");
        assert_eq!(file_stem("NoFolder"), "NoFolder");
    }
}
