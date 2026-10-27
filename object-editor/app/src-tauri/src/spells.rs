//! The spell wizard: spell kinds with their suggested base abilities, the
//! icons to pick from, giving the spell to units, and removing a spell the
//! wizard created when it is cancelled. The spell itself is an ordinary custom
//! ability, created and edited with the ability commands.

use serde::Serialize;
use tauri::State;
use wc3_data::spells::spell_kinds as kinds;

use wc3_map::FourCC;

use crate::abilities::AbilityCard;
use crate::commands::{fourcc, view, with_abilities};
use crate::hotkeys::{self, ABILITY_LIST, HERO_ABILITY_LIST, SpellFit};
use crate::state::{AppState, NO_MAP, Session};

/// Images a map can import as icons.
const IMAGE_EXTENSIONS: &[&str] = &["blp", "dds", "tga"];
const HERO_KIND: &str = "hero";

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

/// How `spell` sits, or would sit, on `unit`'s command card and learn menu.
fn fit(session: &Session, spell: FourCC, unit: FourCC) -> Result<SpellFit, String> {
    let view = view(session);
    let item = view.item(unit).ok_or(format!("Unknown unit {unit}."))?;
    Ok(with_abilities(session, |abilities| {
        hotkeys::spell_fit(
            &view,
            &|ability| abilities.info(ability),
            unit,
            spell,
            item.kind == HERO_KIND,
        )
    }))
}

/// Replaces one of a unit's ability lists in the open map.
fn set_list(
    session: &mut Session,
    unit: FourCC,
    field_id: &str,
    list: &[FourCC],
) -> Result<(), String> {
    let field = session
        .db
        .field(field_id)
        .ok_or(format!("Unknown field {field_id}."))?;
    let map = session.map.as_mut().ok_or(NO_MAP)?;
    let text: Vec<String> = list.iter().map(ToString::to_string).collect();
    map.units
        .set(&session.db, unit, field, &text.join(","))
        .map_err(|e| e.to_string())
}

/// Takes `spell` out of both of the unit's ability lists.
fn take(session: &mut Session, spell: FourCC, unit: FourCC) -> Result<(), String> {
    let (normal, learned) = hotkeys::ability_lists(&view(session), unit);
    for (field_id, list) in [(ABILITY_LIST, normal), (HERO_ABILITY_LIST, learned)] {
        if list.contains(&spell) {
            let kept: Vec<FourCC> = list.into_iter().filter(|a| *a != spell).collect();
            set_list(session, unit, field_id, &kept)?;
        }
    }

    Ok(())
}

/// The units, heroes and buildings that have `spell`, the map's first.
fn holders(session: &Session, spell: FourCC) -> Vec<FourCC> {
    let view = view(session);
    view.ids()
        .into_iter()
        .filter(|id| {
            let (normal, learned) = hotkeys::ability_lists(&view, *id);
            normal.contains(&spell) || learned.contains(&spell)
        })
        .collect()
}

#[tauri::command]
pub fn spell_units(state: State<'_, AppState>, spell: String) -> Result<Vec<String>, String> {
    let spell = fourcc(&spell)?;
    state.with(|session| {
        Ok(holders(session, spell)
            .iter()
            .map(ToString::to_string)
            .collect())
    })
}

#[tauri::command]
pub fn spell_fit(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);
    state.with(|session| fit(session, spell, unit))
}

/// Adds the spell to the unit's learnable hero abilities or its normal ones,
/// as `spell_fit` says.
#[tauri::command]
pub fn give_spell(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);
    state.with(|session| {
        let before = fit(session, spell, unit)?;
        if !before.has {
            let (mut normal, mut learned) = hotkeys::ability_lists(&view(session), unit);
            if before.learned {
                learned.push(spell);
                set_list(session, unit, HERO_ABILITY_LIST, &learned)?;
            } else {
                normal.push(spell);
                set_list(session, unit, ABILITY_LIST, &normal)?;
            }
        }

        fit(session, spell, unit)
    })
}

#[tauri::command]
pub fn take_spell(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);
    state.with(|session| {
        take(session, spell, unit)?;
        fit(session, spell, unit)
    })
}

/// Removes a custom ability the wizard created, when the wizard is cancelled
/// or goes back to pick another base, and takes it from the units it was
/// given to. Game abilities are never removed.
#[tauri::command]
pub fn discard_spell(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = fourcc(&id)?;
    state.with(|session| {
        for unit in holders(session, id) {
            take(session, id, unit)?;
        }

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
