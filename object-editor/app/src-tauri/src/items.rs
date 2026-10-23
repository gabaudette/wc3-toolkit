//! Commands for the Items tab. Items take the unit page's shapes: list rows
//! are `UnitItem`s and pages `UnitDetail`s, built by the same [`View`] over
//! the item database and the map's items.

use tauri::State;
use wc3_data::meta::FieldMeta;
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;
use wc3_map::FourCC;
use wc3_map::items::MapItems;

use crate::commands::fourcc;
use crate::state::{AppState, NO_MAP, Session};
use crate::view::{Field, ItemChanges, SearchHit, UnitDetail, UnitItem, View};

/// Runs `f` with a view of the items, with the open map's changes if there is one.
fn with_view<T>(session: &Session, f: impl FnOnce(&View<'_, ItemChanges<'_>>) -> T) -> T {
    let changes = session.map.as_ref().map(|map| ItemChanges {
        items: &map.items,
        strings: map.units.strings(),
    });
    let view = View {
        db: &session.items,
        guide: &session.item_guide,
        map: changes.as_ref(),
        mode: session.mode,
    };
    f(&view)
}

/// The map's items first, then the game's.
#[tauri::command]
pub fn list_items(state: State<'_, AppState>) -> Result<Vec<UnitItem>, String> {
    state.with(|session| Ok(with_view(session, |view| view.items())))
}

#[tauri::command]
pub fn item_detail(state: State<'_, AppState>, id: String) -> Result<UnitDetail, String> {
    let id = fourcc(&id)?;
    state.with(|session| {
        with_view(session, |view| view.detail(id)).ok_or(format!("Unknown item {id}."))
    })
}

/// Items whose name or code contains the query, then items with a field
/// whose shown value equals it.
#[tauri::command]
pub fn search_items(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(with_view(session, |view| view.search(&query))))
}

/// Applies `change` to the open map's items and returns the field's new state.
fn edit_item_field(
    state: &AppState,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &UnitDatabase,
        &mut MapItems,
        &mut MapStrings,
        FourCC,
        &FieldMeta,
    ) -> Result<(), String>,
) -> Result<Field, String> {
    let id = fourcc(id)?;
    state.with(|session| {
        let field = session
            .items
            .field(field_id)
            .ok_or(format!("Unknown item field {field_id}."))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        change(
            &session.items,
            &mut map.items,
            map.units.strings_mut(),
            id,
            field,
        )?;
        Ok(with_view(session, |view| view.field(id, field)))
    })
}

#[tauri::command]
pub fn set_item_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    value: String,
) -> Result<Field, String> {
    edit_item_field(&state, &id, &field, |db, items, strings, id, meta| {
        items
            .set(db, strings, id, meta, &value)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn reset_item_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
) -> Result<Field, String> {
    edit_item_field(&state, &id, &field, |_, items, strings, id, meta| {
        items.reset(strings, id, meta);
        Ok(())
    })
}

/// A new custom item copying `base`, numbered like the World Editor (`I000`, ...).
#[tauri::command]
pub fn create_item(state: State<'_, AppState>, base: String) -> Result<UnitItem, String> {
    let base = fourcc(&base)?;
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let id = map
            .items
            .create(&session.items, base)
            .map_err(|e| e.to_string())?;
        with_view(session, |view| view.item(id)).ok_or(format!("Created {id} but cannot show it."))
    })
}
