//! The spell wizard: spell kinds with their suggested base abilities, and
//! removing a spell the wizard created when it is cancelled. The spell itself
//! is an ordinary custom ability, created and edited with the ability commands.

use serde::Serialize;
use tauri::State;
use wc3_data::spells::spell_kinds as kinds;

use crate::abilities::AbilityCard;
use crate::commands::{fourcc, with_abilities};
use crate::state::{AppState, NO_MAP};

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
