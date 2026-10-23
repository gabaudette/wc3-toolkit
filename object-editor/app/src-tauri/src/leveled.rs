//! The Upgrades and Doodads tabs. List rows take the unit list's shape
//! (`UnitItem`, with race or category as the filter) and pages the ability
//! page's (`AbilityDetail`), whose per-level fields are tables: an upgrade's
//! name, tooltips, icon and requirements have a value per level, a doodad's
//! tint one per model variation.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use tauri::State;
use wc3_data::game::ArtMode;
use wc3_data::guide::FieldGuide;
use wc3_data::leveled::LeveledData;
use wc3_data::meta::FieldMeta;
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;
use wc3_map::FourCC;
use wc3_map::leveled::{DOODADS, LeveledObject, LeveledObjects, UPGRADES};

use crate::abilities::{
    AbilityCard, AbilityDetail, AbilitySection, AbilitySummary, LevelCell, LevelField,
};
use crate::commands::fourcc;
use crate::state::{AppState, NO_MAP, OpenMap, Session};
use crate::view::{Field, FieldValue, SearchHit, UnitItem, base_text, build_field};

/// Category order on a page: costs and levels, then what it does, then looks.
const CATEGORY_ORDER: &[&str] = &["stats", "data", "text", "art", "path", "tech", "editor"];

/// Which tab a command is for.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Catalog {
    Upgrades,
    Doodads,
}

/// The fields that describe an object on its page, where the kind has them.
struct Describe {
    /// The kind, as list rows name it.
    kind: &'static str,
    singular: &'static str,
    name: &'static str,
    filter: &'static str,
    suffix: Option<&'static str>,
    hotkey: Option<&'static str>,
    tooltip: Option<&'static str>,
}

impl Catalog {
    fn describe(self) -> Describe {
        match self {
            Self::Upgrades => Describe {
                kind: "upgrade",
                singular: "upgrade",
                name: "gnam",
                filter: "grac",
                suffix: Some("gnsf"),
                hotkey: Some("ghk1"),
                tooltip: Some("gub1"),
            },
            Self::Doodads => Describe {
                kind: "doodad",
                singular: "doodad",
                name: "dnam",
                filter: "dcat",
                suffix: None,
                hotkey: None,
                tooltip: None,
            },
        }
    }

    fn data(self, session: &Session) -> (&dyn LeveledData, &FieldGuide) {
        match self {
            Self::Upgrades => (&session.upgrades, &session.upgrade_guide),
            Self::Doodads => (&session.doodads, &session.doodad_guide),
        }
    }

    fn objects(self, map: &OpenMap) -> &LeveledObjects {
        match self {
            Self::Upgrades => &map.upgrades,
            Self::Doodads => &map.doodads,
        }
    }
}

/// The game's upgrades or doodads with a map's changes on top (none without a map).
pub struct LeveledView<'a> {
    pub catalog: Catalog,
    pub db: &'a dyn LeveledData,
    pub units: &'a UnitDatabase,
    pub guide: &'a FieldGuide,
    pub map: &'a LeveledObjects,
    pub strings: &'a MapStrings,
    pub mode: ArtMode,
}

impl LeveledView<'_> {
    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, level, self.mode)
    }

    fn text(&self, id: FourCC, field_id: &str, level: u32) -> Option<String> {
        self.raw(id, self.db.field(field_id)?, level)
            .filter(|t| !t.is_empty())
    }

    /// The map's objects first, then every game object, each once.
    fn ids_after(&self, map_upgrades: &[LeveledObject]) -> Vec<FourCC> {
        let mut ids: Vec<FourCC> = map_upgrades.iter().map(|u| u.id).collect();
        let mut seen: HashSet<FourCC> = ids.iter().copied().collect();
        for id in self.db.ids().iter().filter_map(|id| FourCC::parse(id)) {
            if seen.insert(id) {
                ids.push(id);
            }
        }

        ids
    }

    fn item_with(&self, id: FourCC, entry: Option<&LeveledObject>) -> Option<UnitItem> {
        let info = self.map.info(self.db, self.strings, id, self.mode)?;
        let base = self.map.base_of(self.db, id)?;
        let describe = self.catalog.describe();
        let race = self
            .db
            .field(describe.filter)
            .map(|field| self.units.format_value(field, Some(info.filter.clone())))
            .unwrap_or_default();

        Some(UnitItem {
            id: id.to_string(),
            base: base.to_string(),
            // Doodads name themselves with editor string keys.
            name: self.units.display_text(&info.name),
            kind: describe.kind,
            race,
            group: if entry.is_some() { "map" } else { "game" },
            custom: entry.is_some_and(|e| e.is_custom),
            changed_fields: entry.map_or(0, |e| e.modified_fields),
            icon: info.icon,
        })
    }

    /// The map's objects first, then the game's.
    pub fn list(&self) -> Vec<UnitItem> {
        let map_upgrades = self.map.objects();
        let by_id: HashMap<FourCC, &LeveledObject> =
            map_upgrades.iter().map(|u| (u.id, u)).collect();
        self.ids_after(&map_upgrades)
            .into_iter()
            .filter_map(|id| self.item_with(id, by_id.get(&id).copied()))
            .collect()
    }

    pub fn item(&self, id: FourCC) -> Option<UnitItem> {
        let entry = self.map.objects().into_iter().find(|u| u.id == id);
        self.item_with(id, entry.as_ref())
    }

    fn value(&self, id: FourCC, field: &FieldMeta, level: u32) -> FieldValue {
        let raw = self.raw(id, field, level);
        let base = self.map.base_of(self.db, id).and_then(|base| {
            self.db
                .raw_value(&base.to_string(), field, level, self.mode)
        });
        FieldValue {
            name: self.db.field_name(field),
            label: format!(
                "{} - {}",
                self.db.category_label(&field.category),
                self.db.field_name(field)
            ),
            display: self.units.format_value(field, raw.clone()),
            raw: raw.unwrap_or_default(),
            modified: self.map.is_modified(id, field, level),
            base: base_text(self.units, field, base),
        }
    }

    fn field(&self, id: FourCC, field: &FieldMeta, level: u32) -> Field {
        build_field(
            self.units,
            field,
            self.guide.entry(&field.id),
            self.value(id, field, level),
        )
    }

    fn level_field(&self, id: FourCC, field: &FieldMeta, levels: u32) -> LevelField {
        LevelField {
            field: self.field(id, field, 1),
            cells: (1..=levels)
                .map(|level| {
                    let value = self.value(id, field, level);
                    LevelCell {
                        raw: value.raw,
                        display: value.display,
                        modified: value.modified,
                    }
                })
                .collect(),
        }
    }

    /// Fields in page order: by category, then the guide's important fields,
    /// then the game's order.
    fn sorted_fields(&self) -> Vec<&FieldMeta> {
        let mut fields: Vec<&FieldMeta> = self.db.fields().iter().collect();
        fields.sort_by_key(|f| {
            let order = CATEGORY_ORDER
                .iter()
                .position(|c| *c == f.category)
                .unwrap_or(CATEGORY_ORDER.len());
            let rank = self.guide.rank(&f.id);
            (order, rank.is_none(), rank, f.sort.clone())
        });
        fields
    }

    /// The page: shaped like an ability page, with a column per level.
    pub fn detail(&self, id: FourCC) -> Option<AbilityDetail> {
        let item = self.item(id)?;
        let levels = self.map.levels(self.db, id);
        let mut sections: Vec<AbilitySection> = Vec::new();
        for meta in self.sorted_fields() {
            if !matches!(sections.last(), Some(s) if s.category == meta.category) {
                sections.push(AbilitySection {
                    category: meta.category.clone(),
                    label: self.db.category_label(&meta.category),
                    fields: Vec::new(),
                    level_fields: Vec::new(),
                });
            }

            let Some(section) = sections.last_mut() else {
                continue;
            };
            if meta.per_level {
                section
                    .level_fields
                    .push(self.level_field(id, meta, levels));
            } else {
                section.fields.push(self.field(id, meta, 0));
            }
        }

        let describe = self.catalog.describe();
        let text = |field: Option<&str>| field.and_then(|f| self.text(id, f, 1));
        let summary = AbilitySummary {
            id: item.id,
            base: item.base,
            name: item.name,
            suffix: text(describe.suffix),
            icon: item.icon,
            hero: false,
            item: false,
            race: item.race,
            levels,
            custom: item.custom,
            in_map: item.group == "map",
            changed_fields: item.changed_fields,
        };
        Some(AbilityDetail {
            card: AbilityCard {
                summary,
                known: true,
                hotkey: text(describe.hotkey),
                required_level: 0,
                description: text(describe.tooltip),
                rows: Vec::new(),
            },
            levels,
            sections,
        })
    }

    /// Objects whose code or name contains the query, then upgrades with a
    /// later level's name that does (`Mithril` finds Iron Forged Swords).
    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut by_name = Vec::new();
        let mut by_level = Vec::new();
        for upgrade in self.list() {
            let names_match = upgrade.name.to_lowercase().contains(&query)
                || upgrade.id.to_lowercase().contains(&query)
                || upgrade.base.to_lowercase() == query;
            if names_match {
                by_name.push(SearchHit {
                    id: upgrade.id,
                    reason: String::new(),
                });
                continue;
            }

            let Some(id) = FourCC::parse(&upgrade.id) else {
                continue;
            };
            let name_field = self.catalog.describe().name;
            let names_have_levels = self.db.field(name_field).is_some_and(|f| f.per_level);
            let last = if names_have_levels {
                self.map.levels(self.db, id)
            } else {
                1
            };
            let level_name = (2..=last).find_map(|level| {
                let name = self.text(id, name_field, level)?;
                name.to_lowercase()
                    .contains(&query)
                    .then(|| format!("level {level}: {name}"))
            });
            if let Some(reason) = level_name {
                by_level.push(SearchHit {
                    id: upgrade.id,
                    reason,
                });
            }
        }

        by_name.extend(by_level);
        by_name
    }
}

/// Runs `f` with a view of the catalog, with the open map's changes if there is one.
fn with_view<T>(session: &Session, catalog: Catalog, f: impl FnOnce(&LeveledView<'_>) -> T) -> T {
    let unchanged;
    let (map, strings) = match &session.map {
        Some(open) => (catalog.objects(open), open.units.strings()),
        None => {
            let kind = match catalog {
                Catalog::Upgrades => UPGRADES,
                Catalog::Doodads => DOODADS,
            };
            unchanged = (LeveledObjects::empty(kind), MapStrings::default());
            (&unchanged.0, &unchanged.1)
        }
    };

    let (db, guide) = catalog.data(session);
    f(&LeveledView {
        catalog,
        db,
        units: &session.db,
        guide,
        map,
        strings,
        mode: session.mode,
    })
}

#[tauri::command]
pub fn list_leveled(state: State<'_, AppState>, catalog: Catalog) -> Result<Vec<UnitItem>, String> {
    state.with(|session| Ok(with_view(session, catalog, |view| view.list())))
}

#[tauri::command]
pub fn leveled_detail(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
) -> Result<AbilityDetail, String> {
    let id = fourcc(&id)?;
    state.with(|session| {
        with_view(session, catalog, |view| view.detail(id))
            .ok_or(format!("Unknown {} {id}.", catalog.describe().singular))
    })
}

#[tauri::command]
pub fn search_leveled(
    state: State<'_, AppState>,
    catalog: Catalog,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(with_view(session, catalog, |view| view.search(&query))))
}

/// Applies `change` to the open map's objects and returns the new page.
fn edit(
    state: &AppState,
    catalog: Catalog,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &dyn LeveledData,
        &mut LeveledObjects,
        &mut MapStrings,
        FourCC,
        &FieldMeta,
    ) -> Result<(), String>,
) -> Result<AbilityDetail, String> {
    let id = fourcc(id)?;
    let singular = catalog.describe().singular;
    state.with(|session| {
        // The databases and the open map are separate fields of the session,
        // and the objects and string table separate fields of the map.
        let db: &dyn LeveledData = match catalog {
            Catalog::Upgrades => &session.upgrades,
            Catalog::Doodads => &session.doodads,
        };
        let field = db
            .field(field_id)
            .ok_or(format!("Unknown {singular} field {field_id}."))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let OpenMap {
            units,
            upgrades,
            doodads,
            ..
        } = map;
        let objects = match catalog {
            Catalog::Upgrades => upgrades,
            Catalog::Doodads => doodads,
        };
        change(db, objects, units.strings_mut(), id, field)?;
        with_view(session, catalog, |view| view.detail(id))
            .ok_or(format!("Unknown {singular} {id}."))
    })
}

/// `level` is 0 for fields without levels.
#[tauri::command]
pub fn set_leveled_field(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
    field: String,
    level: u32,
    value: String,
) -> Result<AbilityDetail, String> {
    edit(
        &state,
        catalog,
        &id,
        &field,
        |db, objects, strings, id, meta| {
            objects
                .set(db, strings, id, meta, level, &value)
                .map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn reset_leveled_field(
    state: State<'_, AppState>,
    catalog: Catalog,
    id: String,
    field: String,
    level: u32,
) -> Result<AbilityDetail, String> {
    edit(
        &state,
        catalog,
        &id,
        &field,
        |_, objects, strings, id, meta| {
            objects.reset(strings, id, meta, level);
            Ok(())
        },
    )
}

/// A new custom object copying `base`, numbered like the World Editor (`R000`, `D000`, ...).
#[tauri::command]
pub fn create_leveled(
    state: State<'_, AppState>,
    catalog: Catalog,
    base: String,
) -> Result<UnitItem, String> {
    let base = fourcc(&base)?;
    state.with(|session| {
        let db: &dyn LeveledData = match catalog {
            Catalog::Upgrades => &session.upgrades,
            Catalog::Doodads => &session.doodads,
        };
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let objects = match catalog {
            Catalog::Upgrades => &mut map.upgrades,
            Catalog::Doodads => &mut map.doodads,
        };
        let id = objects.create(db, base).map_err(|e| e.to_string())?;
        with_view(session, catalog, |view| view.item(id))
            .ok_or(format!("Created {id} but cannot show it."))
    })
}

#[cfg(test)]
mod tests {
    //! Against a real install; run with `WC3_GAME_DIR=... cargo test -- --ignored`.

    use std::path::PathBuf;

    use wc3_data::game::{GameFiles, GameOptions};

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn upgrade_pages_have_a_column_per_level() -> TestResult {
        let dir = std::env::var_os("WC3_GAME_DIR")
            .map(PathBuf::from)
            .ok_or("set WC3_GAME_DIR")?;
        let files = GameFiles::open(&dir, GameOptions::default())?;
        let (db, units, guide) = (
            wc3_data::upgrades::UpgradeDatabase::load(&files)?,
            UnitDatabase::load(&files)?,
            FieldGuide::upgrades()?,
        );
        let (mut map, mut strings) = (LeveledObjects::empty(UPGRADES), MapStrings::default());
        let swords = FourCC::parse("Rhme").ok_or("bad id")?;
        let custom = map.create(&db, swords)?;
        let levels = db.field("glvl").ok_or("no glvl")?;
        map.set(&db, &mut strings, custom, levels, 0, "4")?;
        let view = LeveledView {
            catalog: Catalog::Upgrades,
            db: &db,
            units: &units,
            guide: &guide,
            map: &map,
            strings: &strings,
            mode: ArtMode::Sd,
        };

        let first = view.list().into_iter().next().ok_or("empty list")?;
        assert_eq!(
            (first.id.as_str(), first.custom, first.race.as_str()),
            ("R000", true, "Human")
        );
        let detail = view.detail(custom).ok_or("no detail")?;
        assert_eq!(detail.levels, 4);
        assert_eq!(detail.card.summary.name, "Iron Forged Swords");
        let order: Vec<&str> = detail
            .sections
            .iter()
            .map(|s| s.category.as_str())
            .collect();
        assert_eq!(order, ["stats", "data", "text", "art", "tech"]);
        let names = detail
            .sections
            .iter()
            .flat_map(|s| &s.level_fields)
            .find(|f| f.field.id == "gnam")
            .ok_or("no gnam")?;
        let raws: Vec<&str> = names.cells.iter().map(|c| c.raw.as_str()).collect();
        assert_eq!(
            raws,
            [
                "Iron Forged Swords",
                "Steel Forged Swords",
                "Mithril Forged Swords",
                ""
            ]
        );
        assert_eq!(
            view.search("mithril").first().map(|h| h.id.as_str()),
            Some("R000")
        );
        Ok(())
    }
}
