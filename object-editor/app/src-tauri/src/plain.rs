//! Commands for the tabs of objects without levels other than units: items,
//! buffs and destructables. They take the unit page's shapes: list rows are
//! `UnitItem`s and pages `UnitDetail`s, built by the same [`View`] over the
//! catalog's database and the map's objects.

use serde::Deserialize;
use tauri::State;
use wc3_data::guide::FieldGuide;
use wc3_data::meta::FieldMeta;
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;
use wc3_map::FourCC;
use wc3_map::plain::PlainObjects;

use crate::commands::fourcc;
use crate::state::{AppState, NO_MAP, OpenMap, Session};
use crate::view::{Field, PlainObjectChanges, SearchHit, UnitDetail, UnitItem, View};

/// Which tab a command is for.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Catalog {
    Items,
    Buffs,
    Destructables,
}

impl Catalog {
    fn data(self, session: &Session) -> (&UnitDatabase, &FieldGuide) {
        match self {
            Self::Items => (&session.items, &session.item_guide),
            Self::Buffs => (&session.buffs, &session.buff_guide),
            Self::Destructables => (&session.destructables, &session.destructable_guide),
        }
    }

    fn objects(self, map: &OpenMap) -> &PlainObjects {
        match self {
            Self::Items => &map.items,
            Self::Buffs => &map.buffs,
            Self::Destructables => &map.destructables,
        }
    }

    /// The catalog's database, from the session's fields one by one, so it can
    /// be used while the open map is borrowed mutably.
    fn database<'a>(
        self,
        items: &'a UnitDatabase,
        buffs: &'a UnitDatabase,
        destructables: &'a UnitDatabase,
    ) -> &'a UnitDatabase {
        match self {
            Self::Items => items,
            Self::Buffs => buffs,
            Self::Destructables => destructables,
        }
    }

    fn singular(self) -> &'static str {
        match self {
            Self::Items => "item",
            Self::Buffs => "buff",
            Self::Destructables => "destructable",
        }
    }
}

/// Runs `f` with a view of the catalog, with the open map's changes if there is one.
fn with_view<T>(
    session: &Session,
    catalog: Catalog,
    f: impl FnOnce(&View<'_, PlainObjectChanges<'_>>) -> T,
) -> T {
    let changes = session.map.as_ref().map(|map| PlainObjectChanges {
        objects: catalog.objects(map),
        strings: map.units.strings(),
    });
    let (db, guide) = catalog.data(session);
    let view = View {
        db,
        guide,
        map: changes.as_ref(),
        mode: session.mode,
    };

    f(&view)
}

/// The map's objects first, then the game's.
#[tauri::command]
pub fn list_plain(state: State<'_, AppState>, catalog: Catalog) -> Result<Vec<UnitItem>, String> {
    state.with(|session| Ok(with_view(session, catalog, |view| view.items())))
}

#[tauri::command]
pub fn plain_detail(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
) -> Result<UnitDetail, String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        with_view(session, catalog, |view| view.detail(id))
            .ok_or(format!("Unknown {} {id}.", catalog.singular()))
    })
}

/// Objects whose name or code contains the query, then objects with a field
/// whose shown value equals it.
#[tauri::command]
pub fn search_plain(
    state: State<'_, AppState>,
    catalog: Catalog,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(with_view(session, catalog, |view| view.search(&query))))
}

/// Applies `change` to the open map's objects and returns the field's new state.
fn edit_field(
    state: &AppState,
    catalog: Catalog,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &UnitDatabase,
        &mut PlainObjects,
        &mut MapStrings,
        FourCC,
        &FieldMeta,
    ) -> Result<(), String>,
) -> Result<Field, String> {
    let id = fourcc(id)?;

    state.with(|session| {
        let db = catalog.database(&session.items, &session.buffs, &session.destructables);
        let field = db
            .field(field_id)
            .ok_or(format!("Unknown {} field {field_id}.", catalog.singular()))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        // The catalog's objects and the units' string table are separate
        // fields of the map, so both can be borrowed mutably at once.
        let OpenMap {
            units,
            items,
            buffs,
            destructables,
            ..
        } = map;
        let objects = match catalog {
            Catalog::Items => items,
            Catalog::Buffs => buffs,
            Catalog::Destructables => destructables,
        };

        change(db, objects, units.strings_mut(), id, field)?;
        Ok(with_view(session, catalog, |view| view.field(id, field)))
    })
}

#[tauri::command]
pub fn set_plain_field(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
    field: String,
    value: String,
) -> Result<Field, String> {
    edit_field(
        &state,
        catalog,
        &id,
        &field,
        |db, objects, strings, id, meta| {
            objects
                .set(db, strings, id, meta, &value)
                .map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn reset_plain_field(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
    field: String,
) -> Result<Field, String> {
    edit_field(
        &state,
        catalog,
        &id,
        &field,
        |_, objects, strings, id, meta| {
            objects.reset(strings, id, meta);
            Ok(())
        },
    )
}

/// A new custom object copying `base`, numbered like the World Editor (`I000`, `B000`, ...).
#[tauri::command]
pub fn create_plain(
    state: State<'_, AppState>,
    catalog: Catalog,
    base: String,
) -> Result<UnitItem, String> {
    let base = fourcc(&base)?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let OpenMap {
            items,
            buffs,
            destructables,
            ..
        } = map;
        let objects = match catalog {
            Catalog::Items => items,
            Catalog::Buffs => buffs,
            Catalog::Destructables => destructables,
        };
        let db = catalog.database(&session.items, &session.buffs, &session.destructables);
        let id = objects.create(db, base).map_err(|e| e.to_string())?;

        with_view(session, catalog, |view| view.item(id))
            .ok_or(format!("Created {id} but cannot show it."))
    })
}

/// Removes a custom object, such as a buff the spell wizard created when the
/// wizard is cancelled. Game objects are never removed.
#[tauri::command]
pub fn discard_plain(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
) -> Result<(), String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let OpenMap {
            items,
            buffs,
            destructables,
            units,
            ..
        } = map;
        let objects = match catalog {
            Catalog::Items => items,
            Catalog::Buffs => buffs,
            Catalog::Destructables => destructables,
        };

        if objects.remove(units.strings_mut(), id) {
            Ok(())
        } else {
            Err(format!(
                "{id} is not a custom {} of this map.",
                catalog.singular()
            ))
        }
    })
}
