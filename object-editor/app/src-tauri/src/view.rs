//! The game's units with the open map's changes applied, shaped for the UI.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use wc3_data::game::ArtMode;
use wc3_data::guide::{FieldGuide, GuideEntry};
use wc3_data::meta::{FieldMeta, ObjectKind};
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;
use wc3_map::FourCC;
use wc3_map::plain::PlainObjects;
use wc3_map::units::{MapUnit, MapUnits};

/// Category order on a unit page: what people edit most comes first.
const CATEGORY_ORDER: &[&str] = &[
    "stats", "combat", "abil", "text", "art", "move", "path", "sound", "tech", "editor", "data",
];
const ENUM_LIST_SUFFIX: &str = "List";
const TEXT_CATEGORY: &str = "text";
/// Primary attribute codes and the fields holding each attribute.
const ATTRIBUTES: [(&str, &str); 3] = [("STR", "ustr"), ("AGI", "uagi"), ("INT", "uint")];
/// Text fields shown as multi-line boxes: unit and ability tooltips.
const LONG_TEXT_FIELDS: &[&str] = &["utub", "upro", "ides", "aub1", "auu1", "arut"];
const TRUE: &str = "1";
/// The field each list row's filter chips read: race for units and buffs,
/// class for items, category for destructables. The first one the database has wins.
const FILTER_FIELDS: &[&str] = &["urac", "icla", "frac", "bcat"];
/// The icon field: units, items, buffs. Destructables have none and show their category's.
const ICON_FIELDS: &[&str] = &["uico", "iico", "fart"];
const CATEGORY_FIELD: &str = "bcat";
/// The stats a page shows at the top, by object kind (units have their own).
const SUMMARIES: &[(&str, &[(&str, &str)])] = &[
    (
        "item",
        &[
            ("Class", "icla"),
            ("Level", "ilev"),
            ("Gold", "igol"),
            ("Lumber", "ilum"),
            ("Charges", "iuse"),
        ],
    ),
    ("buff", &[("Race", "frac"), ("Effect", "feff")]),
    (
        "destructable",
        &[
            ("Category", "bcat"),
            ("Hit points", "bhps"),
            ("Armor", "barm"),
            ("Variations", "bvar"),
        ],
    ),
];
const NOT_PLACEABLE: &str = "0";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitItem {
    pub id: String,
    pub base: String,
    pub name: String,
    pub kind: &'static str,
    pub race: String,
    /// `map` (custom or changed by the map), `melee`, `campaign`, `special` or `hidden`.
    pub group: &'static str,
    pub custom: bool,
    pub changed_fields: usize,
    pub icon: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitDetail {
    pub item: UnitItem,
    pub summary: Vec<Stat>,
    pub sections: Vec<Section>,
    /// Damage a hero adds to every attack from its primary attribute.
    pub damage_bonus: Option<DamageBonus>,
}

#[derive(Serialize)]
pub struct DamageBonus {
    pub amount: f64,
    /// The primary attribute's name, e.g. `Strength`.
    pub attribute: String,
}

#[derive(Serialize)]
pub struct Stat {
    pub label: &'static str,
    pub value: String,
}

#[derive(Serialize)]
pub struct Section {
    pub category: String,
    pub label: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub id: String,
    pub name: String,
    pub label: String,
    /// Formatted like the World Editor shows it.
    pub display: String,
    /// The stored value, as typed into an editor.
    pub raw: String,
    pub modified: bool,
    /// The base object's value, formatted like `display`: the game object a
    /// custom one copies, or the game's own value for a changed original.
    pub base: String,
    /// The base object's stored value, as `raw` is: which flags differ from it.
    pub base_raw: String,
    pub value_type: String,
    /// `bool`, `number`, `text`, `longText`, `select`, `flags` (bit flags),
    /// `flagList` (a list of choices, like targets), `codes` or `abilities`.
    pub control: &'static str,
    pub options: Vec<Choice>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub integer: bool,
    /// What the field does, from the field guide.
    pub description: Option<String>,
    /// Rarely needed; the page folds these into an "Advanced" group.
    pub advanced: bool,
    /// The description is not confirmed in game.
    pub unverified: bool,
    /// Text the game shows to players (names, tooltips): the editor previews
    /// it with colors, line breaks and placeholders filled in.
    pub preview: bool,
}

#[derive(Serialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
    /// Other spellings the game accepts, e.g. `invu` for `invulnerable`.
    pub aliases: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchHit {
    pub id: String,
    /// Why it matched, e.g. `Hit Points Maximum (Base) = 1000`; empty for name or code matches.
    pub reason: String,
}

/// A map's changes to one kind of plain object (no levels): units, or items.
pub trait PlainChanges {
    /// The objects the map adds or changes.
    fn changed(&self) -> Vec<MapUnit>;
    fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC>;
    fn raw_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String>;
    fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool;
    fn name(&self, db: &UnitDatabase, id: FourCC) -> String;
}

impl PlainChanges for MapUnits {
    fn changed(&self) -> Vec<MapUnit> {
        self.units()
    }

    fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        MapUnits::base_of(self, db, id)
    }

    fn raw_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        MapUnits::raw_value(self, db, id, field, mode)
    }

    fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        MapUnits::is_modified(self, id, field)
    }

    fn name(&self, db: &UnitDatabase, id: FourCC) -> String {
        MapUnits::name(self, db, id)
    }
}

/// A map's items with the string table their text lives in.
pub struct PlainObjectChanges<'a> {
    pub objects: &'a PlainObjects,
    pub strings: &'a MapStrings,
}

impl PlainChanges for PlainObjectChanges<'_> {
    fn changed(&self) -> Vec<MapUnit> {
        self.objects
            .objects()
            .into_iter()
            .map(|i| MapUnit {
                id: i.id,
                base: i.base,
                is_custom: i.is_custom,
                modified_fields: i.modified_fields,
            })
            .collect()
    }

    fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        self.objects.base_of(db, id)
    }

    fn raw_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        self.objects.raw_value(db, self.strings, id, field, mode)
    }

    fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        self.objects.is_modified(id, field)
    }

    fn name(&self, db: &UnitDatabase, id: FourCC) -> String {
        self.objects.name(db, self.strings, id)
    }
}

/// Read-only view of the game data with an optional map on top. Units by
/// default; items, buffs and destructables with [`PlainObjectChanges`] and their database.
pub struct View<'a, M = MapUnits> {
    pub db: &'a UnitDatabase,
    pub guide: &'a FieldGuide,
    pub map: Option<&'a M>,
    pub mode: ArtMode,
}

impl<M: PlainChanges> View<'_, M> {
    /// The map's units first, then every game unit, each once.
    pub fn ids(&self) -> Vec<FourCC> {
        self.ids_after(&self.map_units())
    }

    /// The units the map adds or changes. Rebuilt on every call, so callers
    /// going through many units compute it once.
    fn map_units(&self) -> Vec<MapUnit> {
        self.map.map(M::changed).unwrap_or_default()
    }

    /// The given map units first, then every game unit, each once.
    fn ids_after(&self, map_units: &[MapUnit]) -> Vec<FourCC> {
        let mut ids: Vec<FourCC> = map_units.iter().map(|u| u.id).collect();
        let mut seen: HashSet<FourCC> = ids.iter().copied().collect();
        for id in self.db.ids().iter().filter_map(|id| FourCC::parse(id)) {
            if seen.insert(id) {
                ids.push(id);
            }
        }

        ids
    }

    fn base(&self, id: FourCC) -> Option<FourCC> {
        match self.map {
            Some(map) => map.base_of(self.db, id),
            None => self.db.contains(&id.to_string()).then_some(id),
        }
    }

    fn raw(&self, id: FourCC, field: &FieldMeta) -> Option<String> {
        match self.map {
            Some(map) => map.raw_value(self.db, id, field, self.mode),
            None => self.db.raw_value(&id.to_string(), field, self.mode),
        }
    }

    pub(crate) fn raw_by_id(&self, id: FourCC, field_id: &str) -> Option<String> {
        self.db.field(field_id).and_then(|f| self.raw(id, f))
    }

    fn display(&self, id: FourCC, field: &FieldMeta) -> String {
        self.db.format_value(field, self.raw(id, field))
    }

    fn modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        self.map.is_some_and(|m| m.is_modified(id, field))
    }

    pub(crate) fn name(&self, id: FourCC) -> String {
        match self.map {
            Some(map) => map.name(self.db, id),
            None => self.db.name(&id.to_string()),
        }
    }

    fn fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        self.base(id)
            .map(|base| self.db.fields_for(&base.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn item(&self, id: FourCC) -> Option<UnitItem> {
        let map_unit = self.map_units().into_iter().find(|u| u.id == id);
        self.item_with(id, map_unit.as_ref())
    }

    /// The list row for a unit, given its entry in the map's units if it has one.
    fn item_with(&self, id: FourCC, map_unit: Option<&MapUnit>) -> Option<UnitItem> {
        let base = self.base(id)?;
        let race = FILTER_FIELDS
            .iter()
            .find_map(|field| self.db.field(field))
            .map(|f| self.display(id, f))
            .unwrap_or_default();

        Some(UnitItem {
            id: id.to_string(),
            base: base.to_string(),
            name: self.name(id),
            kind: kind_name(self.db.kind(&base.to_string())),
            race,
            group: self.group(id, map_unit.is_some()),
            custom: map_unit.is_some_and(|u| u.is_custom),
            changed_fields: map_unit.map_or(0, |u| u.modified_fields),
            icon: ICON_FIELDS
                .iter()
                .find_map(|field| self.raw_by_id(id, field))
                .or_else(|| self.category_icon(id)),
        })
    }

    /// The editor's icon for the object's category (destructables).
    fn category_icon(&self, id: FourCC) -> Option<String> {
        let field = self.db.field(CATEGORY_FIELD)?;
        let category = self.raw_by_id(id, CATEGORY_FIELD)?;
        self.db
            .choice_icon(&field.value_type, &category)
            .map(str::to_owned)
    }

    fn group(&self, id: FourCC, in_map: bool) -> &'static str {
        let flag = |field_id: &str| self.raw_by_id(id, field_id);

        if in_map {
            "map"
        } else if matches!(
            self.db.kind(&id.to_string()),
            ObjectKind::Item | ObjectKind::Buff | ObjectKind::Destructable
        ) {
            "game"
        } else if flag("uine").as_deref() == Some(NOT_PLACEABLE) {
            "hidden"
        } else if flag("ucam").as_deref() == Some(TRUE) {
            "campaign"
        } else if flag("uspe").as_deref() == Some(TRUE) {
            "special"
        } else {
            "melee"
        }
    }

    pub fn items(&self) -> Vec<UnitItem> {
        let map_units = self.map_units();
        let by_id: HashMap<FourCC, &MapUnit> = map_units.iter().map(|u| (u.id, u)).collect();

        self.ids_after(&map_units)
            .into_iter()
            .filter_map(|id| self.item_with(id, by_id.get(&id).copied()))
            .collect()
    }

    pub fn field(&self, id: FourCC, field: &FieldMeta) -> Field {
        let raw = self.raw(id, field);
        let base = self
            .base(id)
            .and_then(|base| self.db.raw_value(&base.to_string(), field, self.mode));
        build_field(
            self.db,
            field,
            self.guide.entry(&field.id),
            FieldValue {
                name: self.db.field_name(field),
                label: self.db.field_label(field),
                display: self.db.format_value(field, raw.clone()),
                raw: raw.unwrap_or_default(),
                modified: self.modified(id, field),
                base_raw: base.clone().unwrap_or_default(),
                base: base_text(self.db, field, base),
            },
        )
    }

    pub fn detail(&self, id: FourCC) -> Option<UnitDetail> {
        let item = self.item(id)?;
        let mut fields = self.fields(id);
        fields.sort_by_key(|f| {
            let order = CATEGORY_ORDER
                .iter()
                .position(|c| *c == f.category)
                .unwrap_or(CATEGORY_ORDER.len());
            // The guide's important fields first, then alphabetical.
            let rank = self.guide.rank(&f.id);
            (order, rank.is_none(), rank, self.db.field_name(f))
        });

        let mut sections: Vec<Section> = Vec::new();
        for meta in fields {
            let field = self.field(id, meta);
            match sections.last_mut() {
                Some(section) if section.category == meta.category => section.fields.push(field),
                _ => sections.push(Section {
                    category: meta.category.clone(),
                    label: self.db.category_label(&meta.category),
                    fields: vec![field],
                }),
            }
        }

        let damage_bonus = (item.kind == "hero")
            .then(|| self.damage_bonus(id))
            .flatten();

        Some(UnitDetail {
            summary: self.summary(id, item.kind),
            item,
            sections,
            damage_bonus,
        })
    }

    fn number(&self, id: FourCC, field_id: &str) -> f64 {
        self.raw_by_id(id, field_id)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_default()
    }

    fn shown(&self, id: FourCC, field_id: &str) -> String {
        self.db
            .field(field_id)
            .map(|f| self.display(id, f))
            .unwrap_or_default()
    }

    /// The numbers people check first. Heroes add their primary attribute to damage.
    /// A hero's primary attribute value, which every attack adds as damage.
    fn damage_bonus(&self, id: FourCC) -> Option<DamageBonus> {
        let primary = self.raw_by_id(id, "upra")?;
        let field = ATTRIBUTES
            .iter()
            .find(|(code, _)| primary.eq_ignore_ascii_case(code))
            .map(|(_, field)| *field)?;
        Some(DamageBonus {
            amount: self.number(id, field),
            attribute: self.shown(id, "upra"),
        })
    }

    fn summary(&self, id: FourCC, kind: &str) -> Vec<Stat> {
        if let Some((_, stats)) = SUMMARIES.iter().find(|(k, _)| *k == kind) {
            return stats
                .iter()
                .map(|(label, field)| Stat {
                    label,
                    value: self.shown(id, field),
                })
                .collect();
        }

        let attributes = ATTRIBUTES;
        let is_hero = kind == "hero";
        let hero_bonus = if is_hero {
            self.damage_bonus(id).map_or(0.0, |b| b.amount)
        } else {
            0.0
        };

        let base = self.number(id, "ua1b") + hero_bonus;
        let dice = self.number(id, "ua1d");
        let sides = self.number(id, "ua1s");
        let damage = format!("{}–{}", base + dice, base + dice * sides);

        let mut stats = vec![
            Stat {
                label: "Hit points",
                value: self.shown(id, "uhpm"),
            },
            Stat {
                label: "Damage",
                value: damage,
            },
            Stat {
                label: "Armor",
                value: format!("{} · {}", self.shown(id, "udef"), self.shown(id, "udty")),
            },
            Stat {
                label: "Move speed",
                value: self.shown(id, "umvs"),
            },
        ];

        if is_hero {
            stats.push(Stat {
                label: "Str / Agi / Int",
                value: attributes
                    .iter()
                    .map(|(_, field)| self.shown(id, field))
                    .collect::<Vec<_>>()
                    .join(" / "),
            });
        }

        stats
    }

    /// Units whose name or raw code contains the query, then units with a field
    /// whose shown value equals it (e.g. `1000`, `Metal`).
    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut by_name = Vec::new();
        let mut by_field = Vec::new();
        for id in self.ids() {
            let code = id.to_string();
            let base = self.base(id).map(|b| b.to_string()).unwrap_or_default();
            let names_match = self.name(id).to_lowercase().contains(&query)
                || code.to_lowercase().contains(&query)
                || base.to_lowercase() == query;

            if names_match {
                by_name.push(SearchHit {
                    id: code,
                    reason: String::new(),
                });
                continue;
            }

            let matching = self
                .fields(id)
                .into_iter()
                .find(|f| self.display(id, f).to_lowercase() == query);
            if let Some(field) = matching {
                by_field.push(SearchHit {
                    id: code,
                    reason: format!(
                        "{} = {}",
                        self.db.field_name(field),
                        self.display(id, field)
                    ),
                });
            }
        }

        by_name.extend(by_field);
        by_name
    }
}

/// A field's names and value, as the page shows them.
pub struct FieldValue {
    pub name: String,
    pub label: String,
    pub display: String,
    pub raw: String,
    pub modified: bool,
    pub base: String,
    /// The base object's stored value, to compare with `raw` piece by piece.
    pub base_raw: String,
}

/// How the base column shows a value: numbers as stored (`0.25`, not the
/// editor's `0.250`), so they compare with the value box; choices and lists
/// by their labels.
pub fn base_text(db: &UnitDatabase, field: &FieldMeta, raw: Option<String>) -> String {
    match field.value_type.as_str() {
        "int" | "real" | "unreal" => raw.unwrap_or_default(),
        _ => db.format_value(field, raw),
    }
}

/// A field row for any object kind. Choices come from the unit database,
/// which holds every enumerated type the editor knows.
pub fn build_field(
    db: &UnitDatabase,
    field: &FieldMeta,
    guide: Option<&GuideEntry>,
    value: FieldValue,
) -> Field {
    let options = db.enum_values(&field.value_type).unwrap_or_default();
    let is_list = field.value_type.ends_with(ENUM_LIST_SUFFIX);

    let control = match field.value_type.as_str() {
        "bool" => "bool",
        "abilityList" | "heroAbilityList" => "abilities",
        "int" | "real" | "unreal" => "number",
        _ if !options.is_empty() && field.is_bit_flags() => "flags",
        _ if !options.is_empty() && is_list => "flagList",
        _ if !options.is_empty() => "select",
        _ if is_list => "codes",
        _ if LONG_TEXT_FIELDS.contains(&field.id.as_str()) => "longText",
        _ => "text",
    };

    Field {
        id: field.id.clone(),
        name: value.name,
        label: value.label,
        display: value.display,
        raw: value.raw,
        modified: value.modified,
        base: value.base,
        base_raw: value.base_raw,
        value_type: field.value_type.clone(),
        control,
        options: options
            .iter()
            .map(|o| Choice {
                value: o.value.clone(),
                label: o.label.clone(),
                aliases: o.aliases.clone(),
            })
            .collect(),
        min: field.min.as_deref().and_then(|v| v.parse().ok()),
        max: field.max.as_deref().and_then(|v| v.parse().ok()),
        integer: field.value_type == "int",
        description: guide.map(|g| g.description.clone()),
        advanced: guide.is_some_and(|g| g.advanced),
        unverified: guide.is_some_and(|g| g.unverified),
        preview: field.category == TEXT_CATEGORY && field.value_type == "string",
    }
}

pub fn kind_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Hero => "hero",
        ObjectKind::Unit => "unit",
        ObjectKind::Building => "building",
        ObjectKind::Item => "item",
        ObjectKind::Buff => "buff",
        ObjectKind::Destructable => "destructable",
    }
}

#[cfg(test)]
mod tests {
    //! Against a real install; run with `WC3_GAME_DIR=... cargo test -- --ignored`.

    use std::path::PathBuf;

    use wc3_data::game::{GameFiles, GameOptions};
    use wc3_map::Map;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const MAX_FILES: u32 = 8;

    fn database() -> Result<UnitDatabase, Box<dyn std::error::Error>> {
        let dir = std::env::var_os("WC3_GAME_DIR")
            .map(PathBuf::from)
            .ok_or("set WC3_GAME_DIR")?;
        Ok(UnitDatabase::load(&GameFiles::open(
            &dir,
            GameOptions::default(),
        )?)?)
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn paladin_page_matches_the_game_data() -> TestResult {
        let db = database()?;
        let guide = FieldGuide::units()?;
        let view: View<'_> = View {
            db: &db,
            guide: &guide,
            map: None,
            mode: ArtMode::Sd,
        };
        let paladin = FourCC::parse("Hpal").ok_or("bad id")?;
        let detail = view.detail(paladin).ok_or("no detail")?;

        assert_eq!(detail.item.name, "Paladin");
        assert_eq!(detail.item.kind, "hero");
        assert_eq!(detail.item.group, "melee");
        assert_eq!(
            detail.sections.first().map(|s| s.category.as_str()),
            Some("stats")
        );
        let stat = |label: &str| {
            detail
                .summary
                .iter()
                .find(|s| s.label == label)
                .map(|s| s.value.clone())
        };
        assert_eq!(stat("Damage").as_deref(), Some("24–34"));
        assert_eq!(stat("Str / Agi / Int").as_deref(), Some("22 / 13 / 17"));

        let armor = detail
            .sections
            .iter()
            .flat_map(|s| &s.fields)
            .find(|f| f.id == "uarm")
            .ok_or("no uarm")?;
        assert_eq!(armor.control, "select");
        assert!(armor.options.iter().any(|o| o.value == "Metal"));
        assert!(
            armor
                .description
                .as_deref()
                .is_some_and(|d| d.contains("sound"))
        );

        let stats = detail.sections.first().ok_or("no sections")?;
        let first: Vec<&str> = stats.fields.iter().take(3).map(|f| f.id.as_str()).collect();
        assert_eq!(first, ["uhpm", "umpm", "umpi"]);
        assert!(
            detail
                .sections
                .iter()
                .flat_map(|s| &s.fields)
                .any(|f| f.advanced)
        );
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn search_finds_names_codes_and_values() -> TestResult {
        let db = database()?;
        let guide = FieldGuide::units()?;
        let view: View<'_> = View {
            db: &db,
            guide: &guide,
            map: None,
            mode: ArtMode::Sd,
        };

        assert_eq!(
            view.search("paladin").first().map(|h| h.id.as_str()),
            Some("Hpal")
        );
        assert_eq!(
            view.search("Hamg").first().map(|h| h.id.as_str()),
            Some("Hamg")
        );
        let by_value = view.search("Metal");
        assert!(
            by_value
                .iter()
                .any(|h| h.id == "Hpal" && h.reason.contains("Metal"))
        );
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn map_changes_show_as_modified_and_reset() -> TestResult {
        let db = database()?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3m");
        mpq::Archive::create(&path, MAX_FILES)?.save()?;
        let mut units = MapUnits::load(&Map::open(&path)?)?;
        let hero = units.create(&db, FourCC::parse("Hpal").ok_or("bad id")?)?;
        let hp = db.field("uhpm").ok_or("no uhpm")?;
        units.set(&db, hero, hp, "1000")?;

        let guide = FieldGuide::units()?;
        let view = View {
            db: &db,
            guide: &guide,
            map: Some(&units),
            mode: ArtMode::Sd,
        };
        assert_eq!(view.ids().first(), Some(&hero));
        let field = view.field(hero, hp);
        assert!(field.modified);
        assert_eq!(field.display, "1000");
        assert_eq!(view.item(hero).map(|i| i.group), Some("map"));
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn list_of_a_map_with_many_units_matches_each_row() -> TestResult {
        const CUSTOM_UNITS: usize = 1600;
        const BASES: [&str; 4] = ["Hpal", "hfoo", "ogru", "Edem"];

        let db = database()?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3m");
        mpq::Archive::create(&path, MAX_FILES)?.save()?;
        let mut units = MapUnits::load(&Map::open(&path)?)?;
        let name = db.field("unam").ok_or("no unam")?;
        let mut created = Vec::new();
        for n in 0..CUSTOM_UNITS {
            let base = FourCC::parse(BASES[n % BASES.len()]).ok_or("bad id")?;
            let unit = units.create(&db, base)?;
            units.set(&db, unit, name, &format!("Unit {n}"))?;
            created.push(unit);
        }

        let guide = FieldGuide::units()?;
        let view = View {
            db: &db,
            guide: &guide,
            map: Some(&units),
            mode: ArtMode::Sd,
        };
        // A refresh used to take seconds here: every row rebuilt the map's unit list.
        let items = view.items();

        let ids: Vec<String> = view.ids().iter().map(ToString::to_string).collect();
        let listed: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
        assert_eq!(listed, ids);
        for (item, unit) in items.iter().zip(&created) {
            assert_eq!(item.id, unit.to_string());
            assert_eq!(item.group, "map");
            assert!(item.custom);
            assert_eq!(item.changed_fields, 1);
        }

        for item in items.iter().step_by(CUSTOM_UNITS / BASES.len()) {
            let id = FourCC::parse(&item.id).ok_or("bad id")?;
            let row = view.item(id).ok_or("no row")?;
            assert_eq!(
                (row.name, row.group, row.custom, row.changed_fields),
                (
                    item.name.clone(),
                    item.group,
                    item.custom,
                    item.changed_fields
                )
            );
        }
        Ok(())
    }
}
