use std::collections::{HashMap, HashSet};

use serde::Serialize;
use wc3_data::abilities::{AbilityDatabase, AbilityInfo};
use wc3_data::game::ArtMode;
use wc3_data::guide::FieldGuide;
use wc3_data::meta::FieldMeta;
use wc3_data::units::UnitDatabase;
use wc3_map::FourCC;
use wc3_map::MapStrings;
use wc3_map::abilities::{MapAbilities, MapAbility, MapPlaceholders};

use crate::view::{Field, FieldValue, SearchHit, base_text, build_field};

const CATEGORY_ORDER: &[&str] = &["stats", "data", "text", "art", "tech", "sound"];
const DATA_SOURCE: &str = "AbilityData";
const NUMERIC_TYPES: &[&str] = &["int", "real", "unreal"];
const TOOLTIP: &str = "aub1";
const LEARN_TOOLTIP: &str = "arut";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilitySummary {
    pub id: String,
    /// The game ability it copies; its own ID for game abilities.
    pub base: String,
    pub name: String,
    pub suffix: Option<String>,
    pub icon: Option<String>,
    pub hero: bool,
    pub item: bool,
    pub race: String,
    pub levels: u32,
    pub custom: bool,
    pub in_map: bool,
    pub changed_fields: usize,
}

#[derive(Serialize)]
pub struct LevelRow {
    pub label: String,
    pub values: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityCard {
    pub summary: AbilitySummary,
    pub known: bool,
    pub hotkey: Option<String>,
    pub required_level: u32,
    /// The text a player reads: the "learn" tooltip for hero abilities (it
    /// covers every level), the level 1 tooltip otherwise. May contain color codes.
    pub description: Option<String>,
    pub rows: Vec<LevelRow>,
}

#[derive(Serialize)]
pub struct LevelCell {
    pub raw: String,
    pub display: String,
    pub modified: bool,
}

#[derive(Serialize)]
pub struct LevelField {
    pub field: Field,
    pub cells: Vec<LevelCell>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilitySection {
    pub category: String,
    pub label: String,
    pub fields: Vec<Field>,
    pub level_fields: Vec<LevelField>,
}

#[derive(Serialize)]
pub struct AbilityDetail {
    pub card: AbilityCard,
    pub levels: u32,
    pub sections: Vec<AbilitySection>,
}

pub struct AbilityView<'a> {
    pub db: &'a AbilityDatabase,
    pub units: &'a UnitDatabase,
    pub guide: &'a FieldGuide,
    pub map: &'a MapAbilities,
    pub strings: &'a MapStrings,
    pub mode: ArtMode,
}

impl AbilityView<'_> {
    pub fn info(&self, id: FourCC) -> Option<AbilityInfo> {
        self.map.info(self.db, self.strings, id, self.mode)
    }

    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, level, self.mode)
    }

    fn placeholders(&self) -> MapPlaceholders<'_> {
        MapPlaceholders {
            abilities: self.map,
            db: self.db,
            strings: self.strings,
            mode: self.mode,
            others: self.units,
        }
    }

    pub fn fill_placeholders(&self, text: &str) -> String {
        self.db.fill_placeholders(text, &self.placeholders())
    }

    fn summary_with(
        &self,
        id: FourCC,
        info: AbilityInfo,
        entry: Option<&MapAbility>,
    ) -> AbilitySummary {
        AbilitySummary {
            id: info.id,
            base: self.map.base_of(self.db, id).unwrap_or(id).to_string(),
            name: info.name,
            suffix: info.suffix,
            icon: info.icon,
            hero: info.hero,
            item: info.item,
            race: info.race,
            levels: info.levels,
            custom: entry.is_some_and(|e| e.is_custom),
            in_map: entry.is_some(),
            changed_fields: entry.map_or(0, |e| e.modified_fields),
        }
    }

    pub fn summary(&self, id: FourCC) -> Option<AbilitySummary> {
        let info = self.info(id)?;
        let entry = self.map.abilities().into_iter().find(|a| a.id == id);

        Some(self.summary_with(id, info, entry.as_ref()))
    }

    fn ids_after(&self, map_abilities: &[MapAbility]) -> Vec<FourCC> {
        let mut ids: Vec<FourCC> = map_abilities.iter().map(|a| a.id).collect();
        let mut seen: HashSet<FourCC> = ids.iter().copied().collect();

        for id in self.db.ids().iter().filter_map(|id| FourCC::parse(id)) {
            if seen.insert(id) {
                ids.push(id);
            }
        }

        ids
    }

    pub fn list(&self) -> Vec<AbilitySummary> {
        let map_abilities = self.map.abilities();
        let by_id: HashMap<FourCC, &MapAbility> = map_abilities.iter().map(|a| (a.id, a)).collect();

        self.ids_after(&map_abilities)
            .into_iter()
            .filter_map(|id| {
                let info = self.info(id)?;

                Some(self.summary_with(id, info, by_id.get(&id).copied()))
            })
            .collect()
    }

    pub fn card(&self, code: &str) -> AbilityCard {
        let info = FourCC::parse(code).and_then(|id| Some((id, self.info(id)?)));
        let Some((id, info)) = info else {
            return AbilityCard {
                summary: AbilitySummary {
                    id: code.to_owned(),
                    base: code.to_owned(),
                    name: code.to_owned(),
                    suffix: None,
                    icon: None,
                    hero: false,
                    item: false,
                    race: String::new(),
                    levels: 1,
                    custom: false,
                    in_map: false,
                    changed_fields: 0,
                },
                known: false,
                hotkey: None,
                required_level: 0,
                description: None,
                rows: Vec::new(),
            };
        };

        let text = |field_id: &str, level: u32| {
            let field = self.db.field(field_id)?;
            let text = self.raw(id, field, level)?;
            Some(self.fill_placeholders(&text))
        };
        let description = if info.hero {
            text(LEARN_TOOLTIP, 0).or_else(|| text(TOOLTIP, 1))
        } else {
            text(TOOLTIP, 1)
        };
        let rows = self
            .db
            .level_values_with(&info.code, info.levels, &|field, level| {
                self.raw(id, field, level)
            })
            .into_iter()
            .map(|row| LevelRow {
                label: row.label,
                values: row.values,
            })
            .collect();

        let entry = self.map.abilities().into_iter().find(|a| a.id == id);

        AbilityCard {
            known: true,
            hotkey: info.hotkey.clone(),
            required_level: info.required_level,
            description,
            rows,
            summary: self.summary_with(id, info, entry.as_ref()),
        }
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
            base_raw: base.clone().unwrap_or_default(),
            base: base_text(self.units, field, base),
        }
    }

    pub fn field(&self, id: FourCC, field: &FieldMeta, level: u32) -> Field {
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

    fn sorted_fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        let mut fields = self.map.fields(self.db, id);

        fields.sort_by_key(|f| {
            let order = CATEGORY_ORDER
                .iter()
                .position(|c| *c == f.category)
                .unwrap_or(CATEGORY_ORDER.len());
            let rank = self.guide.rank(&f.id);

            (order, rank.is_none(), rank, f.data_column, f.sort.clone())
        });

        fields
    }

    pub fn detail(&self, id: FourCC) -> Option<AbilityDetail> {
        let card = self.card(&id.to_string());

        if !card.known {
            return None;
        }

        let levels = self.map.levels(self.db, id);
        let mut sections: Vec<AbilitySection> = Vec::new();

        for meta in self.sorted_fields(id) {
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

        Some(AbilityDetail {
            card,
            levels,
            sections,
        })
    }

    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        let query = query.trim().to_lowercase();

        if query.is_empty() {
            return Vec::new();
        }

        let number: Option<f64> = query.parse().ok();
        let map_abilities = self.map.abilities();
        let in_map: HashSet<FourCC> = map_abilities.iter().map(|a| a.id).collect();
        let mut by_name = Vec::new();
        let mut by_value = Vec::new();

        for id in self.ids_after(&map_abilities) {
            let Some(info) = self.info(id) else {
                continue;
            };

            let code = id.to_string();
            let names_match = info.name.to_lowercase().contains(&query)
                || info
                    .suffix
                    .as_deref()
                    .is_some_and(|s| s.to_lowercase().contains(&query))
                || code.to_lowercase().contains(&query)
                || info.code.to_lowercase() == query;

            if names_match {
                by_name.push(SearchHit {
                    id: code,
                    reason: String::new(),
                });

                continue;
            }

            let Some(number) = number else {
                continue;
            };

            if let Some(reason) = self.number_match(id, in_map.contains(&id), info.levels, number) {
                by_value.push(SearchHit { id: code, reason });
            }
        }

        by_name.extend(by_value);
        by_name
    }

    fn number_match(&self, id: FourCC, in_map: bool, levels: u32, number: f64) -> Option<String> {
        let key = id.to_string();
        let raw = |field: &FieldMeta, level: u32| {
            if in_map {
                self.raw(id, field, level)
            } else {
                self.db.raw_value(&key, field, level, self.mode)
            }
        };

        self.map.fields(self.db, id).into_iter().find_map(|field| {
            let numeric = field.per_level
                && field.source == DATA_SOURCE
                && NUMERIC_TYPES.contains(&field.value_type.as_str());

            if !numeric {
                return None;
            }

            (1..=levels).find_map(|level| {
                let value = raw(field, level)?;
                let matches = value.trim().parse::<f64>().is_ok_and(|v| v == number);

                matches.then(|| format!("{} = {value} (level {level})", self.db.field_name(field)))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    //! Against a real install; run with `WC3_GAME_DIR=... cargo test -- --ignored`.

    use std::path::PathBuf;

    use wc3_data::game::{GameFiles, GameOptions};

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    type Databases = (AbilityDatabase, UnitDatabase, FieldGuide);

    fn load() -> Result<Databases, Box<dyn std::error::Error>> {
        let dir = std::env::var_os("WC3_GAME_DIR")
            .map(PathBuf::from)
            .ok_or("set WC3_GAME_DIR")?;
        let files = GameFiles::open(&dir, GameOptions::default())?;

        Ok((
            AbilityDatabase::load(&files)?,
            UnitDatabase::load(&files)?,
            FieldGuide::abilities()?,
        ))
    }

    fn view<'a>(
        dbs: &'a Databases,
        map: &'a MapAbilities,
        strings: &'a MapStrings,
    ) -> AbilityView<'a> {
        AbilityView {
            db: &dbs.0,
            units: &dbs.1,
            guide: &dbs.2,
            map,
            strings,
            mode: ArtMode::Sd,
        }
    }

    fn id(code: &str) -> Result<FourCC, String> {
        FourCC::parse(code).ok_or(format!("bad id {code}"))
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn holy_light_page_has_a_column_per_level() -> TestResult {
        let dbs = load()?;
        let (map, strings) = (MapAbilities::empty(), MapStrings::default());
        let view = view(&dbs, &map, &strings);
        let detail = view.detail(id("AHhb")?).ok_or("no detail")?;

        assert_eq!(detail.levels, 3);
        assert_eq!(detail.card.summary.name, "Holy Light");
        let order: Vec<&str> = detail
            .sections
            .iter()
            .map(|s| s.category.as_str())
            .collect();

        assert_eq!(order.first(), Some(&"stats"));
        let level_field = |field_id: &str| {
            detail
                .sections
                .iter()
                .flat_map(|s| &s.level_fields)
                .find(|f| f.field.id == field_id)
        };
        let amount = level_field("Hhb1").ok_or("no Hhb1")?;
        let values: Vec<&str> = amount.cells.iter().map(|c| c.raw.as_str()).collect();

        assert_eq!(values, ["200", "400", "600"]);
        assert!(level_field("atp1").is_some_and(|f| f.cells.len() == 3));
        assert!(
            detail
                .sections
                .iter()
                .flat_map(|s| &s.fields)
                .any(|f| f.id == "alev" && f.raw == "3"),
            "levels is a single field"
        );
        let targets = level_field("atar").ok_or("no atar")?;

        assert_eq!(targets.field.control, "flagList");
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn search_finds_names_codes_and_numbers() -> TestResult {
        let dbs = load()?;
        let (map, strings) = (MapAbilities::empty(), MapStrings::default());
        let view = view(&dbs, &map, &strings);

        assert_eq!(
            view.search("holy light").first().map(|h| h.id.as_str()),
            Some("AHhb")
        );
        assert_eq!(
            view.search("AHbz").first().map(|h| h.id.as_str()),
            Some("AHbz")
        );
        let by_value = view.search("600");

        assert!(
            by_value
                .iter()
                .any(|h| h.id == "AHhb" && h.reason.contains("level 3")),
            "{:?}",
            by_value
                .iter()
                .take(5)
                .map(|h| &h.reason)
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn custom_abilities_show_their_own_values() -> TestResult {
        let dbs = load()?;
        let mut map = MapAbilities::empty();
        let mut strings = MapStrings::default();
        let custom = map.create(&dbs.0, id("AHhb")?)?;
        let field = |field_id: &str| dbs.0.field(field_id).ok_or(format!("no {field_id}"));

        map.set(
            &dbs.0,
            &mut strings,
            custom,
            field("anam")?,
            0,
            "Holier Light",
        )?;
        map.set(&dbs.0, &mut strings, custom, field("Hhb1")?, 1, "250")?;
        map.set(&dbs.0, &mut strings, custom, field("alev")?, 0, "4")?;
        let view = view(&dbs, &map, &strings);

        let first = view.list().into_iter().next().ok_or("empty list")?;

        assert_eq!(
            (first.id.as_str(), first.custom, first.base.as_str()),
            ("A000", true, "AHhb")
        );
        let card = view.card("A000");

        assert!(card.known);
        assert_eq!(card.summary.name, "Holier Light");
        let amount = card
            .rows
            .iter()
            .find(|r| r.label == "Amount Healed/Damaged")
            .ok_or("no amount row")?;

        assert_eq!(amount.values, ["250", "400", "600", "600"]);

        let detail = view.detail(custom).ok_or("no detail")?;
        let cells = detail
            .sections
            .iter()
            .flat_map(|s| &s.level_fields)
            .find(|f| f.field.id == "Hhb1")
            .map(|f| f.cells.iter().map(|c| c.modified).collect::<Vec<_>>());

        assert_eq!(cells, Some(vec![true, false, false, false]));
        assert_eq!(view.fill_placeholders("<A000,DataA1>"), "250");
        Ok(())
    }
}
