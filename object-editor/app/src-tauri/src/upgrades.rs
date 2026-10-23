//! The Upgrades tab. List rows take the unit list's shape (`UnitItem`, with
//! the race as the filter) and pages the ability page's (`AbilityDetail`), whose
//! per-level fields are tables: an upgrade's name, tooltips, icon and
//! requirements have a value per level.

use std::collections::{HashMap, HashSet};

use tauri::State;
use wc3_data::game::ArtMode;
use wc3_data::guide::FieldGuide;
use wc3_data::meta::FieldMeta;
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::FourCC;
use wc3_map::upgrades::{MapUpgrade, MapUpgrades};

use crate::abilities::{
    AbilityCard, AbilityDetail, AbilitySection, AbilitySummary, LevelCell, LevelField,
};
use crate::commands::fourcc;
use crate::state::{AppState, NO_MAP, Session};
use crate::view::{Field, FieldValue, SearchHit, UnitItem, base_text, build_field};

/// Category order on an upgrade page: costs and levels, then what it does.
const CATEGORY_ORDER: &[&str] = &["stats", "data", "text", "art", "tech"];
const TOOLTIP: &str = "gub1";
const HOTKEY: &str = "ghk1";
const SUFFIX: &str = "gnsf";
const NAME: &str = "gnam";
const KIND: &str = "upgrade";

/// The game's upgrades with a map's changes on top (none without a map).
pub struct UpgradeView<'a> {
    pub db: &'a UpgradeDatabase,
    pub units: &'a UnitDatabase,
    pub guide: &'a FieldGuide,
    pub map: &'a MapUpgrades,
    pub strings: &'a MapStrings,
    pub mode: ArtMode,
}

impl UpgradeView<'_> {
    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, level, self.mode)
    }

    fn text(&self, id: FourCC, field_id: &str, level: u32) -> Option<String> {
        self.raw(id, self.db.field(field_id)?, level)
            .filter(|t| !t.is_empty())
    }

    /// The map's upgrades first, then every game upgrade, each once.
    fn ids_after(&self, map_upgrades: &[MapUpgrade]) -> Vec<FourCC> {
        let mut ids: Vec<FourCC> = map_upgrades.iter().map(|u| u.id).collect();
        let mut seen: HashSet<FourCC> = ids.iter().copied().collect();
        for id in self.db.ids().iter().filter_map(|id| FourCC::parse(id)) {
            if seen.insert(id) {
                ids.push(id);
            }
        }

        ids
    }

    fn item_with(&self, id: FourCC, entry: Option<&MapUpgrade>) -> Option<UnitItem> {
        let info = self.map.info(self.db, self.strings, id, self.mode)?;
        let base = self.map.base_of(self.db, id)?;
        let race = self
            .db
            .field("grac")
            .map(|field| self.units.format_value(field, Some(info.race.clone())))
            .unwrap_or_default();

        Some(UnitItem {
            id: id.to_string(),
            base: base.to_string(),
            name: info.name,
            kind: KIND,
            race,
            group: if entry.is_some() { "map" } else { "game" },
            custom: entry.is_some_and(|e| e.is_custom),
            changed_fields: entry.map_or(0, |e| e.modified_fields),
            icon: info.icon,
        })
    }

    /// The map's upgrades first, then the game's.
    pub fn list(&self) -> Vec<UnitItem> {
        let map_upgrades = self.map.upgrades();
        let by_id: HashMap<FourCC, &MapUpgrade> = map_upgrades.iter().map(|u| (u.id, u)).collect();
        self.ids_after(&map_upgrades)
            .into_iter()
            .filter_map(|id| self.item_with(id, by_id.get(&id).copied()))
            .collect()
    }

    pub fn item(&self, id: FourCC) -> Option<UnitItem> {
        let entry = self.map.upgrades().into_iter().find(|u| u.id == id);
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

        let summary = AbilitySummary {
            id: item.id,
            base: item.base,
            name: item.name,
            suffix: self.text(id, SUFFIX, 1),
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
                hotkey: self.text(id, HOTKEY, 1),
                required_level: 0,
                description: self.text(id, TOOLTIP, 1),
                rows: Vec::new(),
            },
            levels,
            sections,
        })
    }

    /// Upgrades whose code or name contains the query, then upgrades with a
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
            let level_name = (2..=self.map.levels(self.db, id)).find_map(|level| {
                let name = self.text(id, NAME, level)?;
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

/// Runs `f` with a view of the upgrades, with the open map's changes if there is one.
fn with_upgrades<T>(session: &Session, f: impl FnOnce(&UpgradeView<'_>) -> T) -> T {
    let unchanged;
    let (map, strings) = match &session.map {
        Some(open) => (&open.upgrades, open.units.strings()),
        None => {
            unchanged = (MapUpgrades::empty(), MapStrings::default());
            (&unchanged.0, &unchanged.1)
        }
    };

    f(&UpgradeView {
        db: &session.upgrades,
        units: &session.db,
        guide: &session.upgrade_guide,
        map,
        strings,
        mode: session.mode,
    })
}

#[tauri::command]
pub fn list_upgrades(state: State<'_, AppState>) -> Result<Vec<UnitItem>, String> {
    state.with(|session| Ok(with_upgrades(session, |upgrades| upgrades.list())))
}

#[tauri::command]
pub fn upgrade_detail(state: State<'_, AppState>, id: String) -> Result<AbilityDetail, String> {
    let id = fourcc(&id)?;
    state.with(|session| {
        with_upgrades(session, |upgrades| upgrades.detail(id))
            .ok_or(format!("Unknown upgrade {id}."))
    })
}

#[tauri::command]
pub fn search_upgrades(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    state.with(|session| Ok(with_upgrades(session, |upgrades| upgrades.search(&query))))
}

/// Applies `change` to the open map's upgrades and returns the new page.
fn edit_upgrade(
    state: &AppState,
    id: &str,
    field_id: &str,
    change: impl FnOnce(
        &UpgradeDatabase,
        &mut MapUpgrades,
        &mut MapStrings,
        FourCC,
        &FieldMeta,
    ) -> Result<(), String>,
) -> Result<AbilityDetail, String> {
    let id = fourcc(id)?;
    state.with(|session| {
        let field = session
            .upgrades
            .field(field_id)
            .ok_or(format!("Unknown upgrade field {field_id}."))?;
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        change(
            &session.upgrades,
            &mut map.upgrades,
            map.units.strings_mut(),
            id,
            field,
        )?;
        with_upgrades(session, |upgrades| upgrades.detail(id))
            .ok_or(format!("Unknown upgrade {id}."))
    })
}

/// `level` is 0 for fields without levels.
#[tauri::command]
pub fn set_upgrade_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    level: u32,
    value: String,
) -> Result<AbilityDetail, String> {
    edit_upgrade(&state, &id, &field, |db, upgrades, strings, id, meta| {
        upgrades
            .set(db, strings, id, meta, level, &value)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn reset_upgrade_field(
    state: State<'_, AppState>,
    id: String,
    field: String,
    level: u32,
) -> Result<AbilityDetail, String> {
    edit_upgrade(&state, &id, &field, |_, upgrades, strings, id, meta| {
        upgrades.reset(strings, id, meta, level);
        Ok(())
    })
}

/// A new custom upgrade copying `base`, numbered like the World Editor (`R000`, ...).
#[tauri::command]
pub fn create_upgrade(state: State<'_, AppState>, base: String) -> Result<UnitItem, String> {
    let base = fourcc(&base)?;
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let id = map
            .upgrades
            .create(&session.upgrades, base)
            .map_err(|e| e.to_string())?;
        with_upgrades(session, |upgrades| upgrades.item(id))
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
            UpgradeDatabase::load(&files)?,
            UnitDatabase::load(&files)?,
            FieldGuide::upgrades()?,
        );
        let (mut map, mut strings) = (MapUpgrades::empty(), MapStrings::default());
        let swords = FourCC::parse("Rhme").ok_or("bad id")?;
        let custom = map.create(&db, swords)?;
        let levels = db.field("glvl").ok_or("no glvl")?;
        map.set(&db, &mut strings, custom, levels, 0, "4")?;
        let view = UpgradeView {
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
