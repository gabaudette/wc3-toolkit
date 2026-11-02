use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wc3_data::abilities::{AbilityDatabase, PlaceholderSource};
use wc3_data::doodads::DoodadDatabase;
use wc3_data::game::ArtMode;
use wc3_data::leveled::LeveledData;
use wc3_data::meta::FieldMeta;
use wc3_data::strings::{MapStrings, plain_text};
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;

use crate::abilities::{MapAbilities, MapPlaceholders};
use crate::leveled::{DOODADS, LeveledObjects, UPGRADES};
use crate::plain::{BUFFS, DESTRUCTABLES, ITEMS, PlainObjects};
use crate::units::MapUnits;
use crate::{FourCC, Map, Result};

pub const FORMAT: &str = "tinkerbench-objects";
pub const FORMAT_VERSION: u32 = 1;

const LEVEL_OF_PLAIN_FIELDS: u32 = 0;
const FIRST_LEVEL: u32 = 1;
const PLACEHOLDER_START: char = '<';
const BYTE_ORDER_MARK: &str = "\u{feff}";
const CSV_LINE_END: &str = "\r\n";
const CSV_EXTENSION: &str = "csv";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportType {
    Units,
    Items,
    Buffs,
    Destructables,
    Abilities,
    Upgrades,
    Doodads,
}

impl ExportType {
    pub const ALL: [Self; 7] = [
        Self::Units,
        Self::Items,
        Self::Buffs,
        Self::Destructables,
        Self::Abilities,
        Self::Upgrades,
        Self::Doodads,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Units => "units",
            Self::Items => "items",
            Self::Buffs => "buffs",
            Self::Destructables => "destructables",
            Self::Abilities => "abilities",
            Self::Upgrades => "upgrades",
            Self::Doodads => "doodads",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|t| t.name().eq_ignore_ascii_case(name))
    }
}

pub struct GameData<'a> {
    pub units: &'a UnitDatabase,
    pub items: &'a UnitDatabase,
    pub buffs: &'a UnitDatabase,
    pub destructables: &'a UnitDatabase,
    pub abilities: &'a AbilityDatabase,
    pub upgrades: &'a UpgradeDatabase,
    pub doodads: &'a DoodadDatabase,
    pub mode: ArtMode,
}

pub struct MapData<'a> {
    pub units: &'a MapUnits,
    pub items: &'a PlainObjects,
    pub buffs: &'a PlainObjects,
    pub destructables: &'a PlainObjects,
    pub abilities: &'a MapAbilities,
    pub upgrades: &'a LeveledObjects,
    pub doodads: &'a LeveledObjects,
}

pub struct LoadedMap {
    units: MapUnits,
    items: PlainObjects,
    buffs: PlainObjects,
    destructables: PlainObjects,
    abilities: MapAbilities,
    upgrades: LeveledObjects,
    doodads: LeveledObjects,
}

impl LoadedMap {
    pub fn load(map: &Map) -> Result<Self> {
        Ok(Self {
            units: MapUnits::load(map)?,
            items: PlainObjects::load(map, ITEMS)?,
            buffs: PlainObjects::load(map, BUFFS)?,
            destructables: PlainObjects::load(map, DESTRUCTABLES)?,
            abilities: MapAbilities::load(map)?,
            upgrades: LeveledObjects::load(map, UPGRADES)?,
            doodads: LeveledObjects::load(map, DOODADS)?,
        })
    }

    pub fn empty() -> Self {
        Self {
            units: MapUnits::empty(),
            items: PlainObjects::empty(ITEMS),
            buffs: PlainObjects::empty(BUFFS),
            destructables: PlainObjects::empty(DESTRUCTABLES),
            abilities: MapAbilities::empty(),
            upgrades: LeveledObjects::empty(UPGRADES),
            doodads: LeveledObjects::empty(DOODADS),
        }
    }

    pub fn data(&self) -> MapData<'_> {
        MapData {
            units: &self.units,
            items: &self.items,
            buffs: &self.buffs,
            destructables: &self.destructables,
            abilities: &self.abilities,
            upgrades: &self.upgrades,
            doodads: &self.doodads,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub types: Vec<ExportType>,
    pub ids: Option<Vec<String>>,
    pub origin: Origin,
    pub plain_text: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    #[default]
    All,
    Changed,
    Custom,
}

impl Origin {
    fn keeps(self, changed: Option<bool>) -> bool {
        match self {
            Self::All => true,
            Self::Changed => changed.is_some(),
            Self::Custom => changed == Some(true),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Export {
    pub format: &'static str,
    pub version: u32,
    pub map: Option<String>,
    pub fields: BTreeMap<ExportType, BTreeMap<String, FieldInfo>>,
    pub objects: BTreeMap<ExportType, Vec<Object>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    pub name: String,
    pub category: String,
    pub per_level: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub id: String,
    pub base: String,
    pub name: String,
    pub custom: bool,
    pub changed: bool,
    pub levels: u32,
    pub modified: Vec<String>,
    pub values: BTreeMap<String, Values>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Values {
    One(Value),
    PerLevel(Vec<Value>),
}

impl Values {
    pub fn levels(&self) -> &[Value] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::PerLevel(values) => values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Value {
    pub raw: String,
    pub display: String,
}

trait Source {
    fn changed(&self) -> Vec<(FourCC, bool)>;
    fn game_ids(&self) -> &[String];
    fn base(&self, id: FourCC) -> Option<FourCC>;
    fn name(&self, id: FourCC) -> String;
    fn fields(&self, id: FourCC) -> Vec<&FieldMeta>;
    fn field_name(&self, field: &FieldMeta) -> String;
    fn category(&self, field: &FieldMeta) -> String;
    fn levels(&self, id: FourCC) -> u32;
    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String>;
    fn modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool;
}

struct Units<'a> {
    db: &'a UnitDatabase,
    map: &'a MapUnits,
    mode: ArtMode,
}

impl Source for Units<'_> {
    fn changed(&self) -> Vec<(FourCC, bool)> {
        self.map
            .units()
            .into_iter()
            .map(|u| (u.id, u.is_custom))
            .collect()
    }

    fn game_ids(&self) -> &[String] {
        self.db.ids()
    }

    fn base(&self, id: FourCC) -> Option<FourCC> {
        self.map.base_of(self.db, id)
    }

    fn name(&self, id: FourCC) -> String {
        self.map.name(self.db, id)
    }

    fn fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        self.map.fields(self.db, id)
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        self.db.field_name(field)
    }

    fn category(&self, field: &FieldMeta) -> String {
        self.db.category_label(&field.category)
    }

    fn levels(&self, _: FourCC) -> u32 {
        FIRST_LEVEL
    }

    fn raw(&self, id: FourCC, field: &FieldMeta, _: u32) -> Option<String> {
        self.map.raw_value(self.db, id, field, self.mode)
    }

    fn modified(&self, id: FourCC, field: &FieldMeta, _: u32) -> bool {
        self.map.is_modified(id, field)
    }
}

struct Plain<'a> {
    db: &'a UnitDatabase,
    map: &'a PlainObjects,
    strings: &'a MapStrings,
    mode: ArtMode,
}

impl Source for Plain<'_> {
    fn changed(&self) -> Vec<(FourCC, bool)> {
        self.map
            .objects()
            .into_iter()
            .map(|o| (o.id, o.is_custom))
            .collect()
    }

    fn game_ids(&self) -> &[String] {
        self.db.ids()
    }

    fn base(&self, id: FourCC) -> Option<FourCC> {
        self.map.base_of(self.db, id)
    }

    fn name(&self, id: FourCC) -> String {
        self.map.name(self.db, self.strings, id)
    }

    fn fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        self.map.fields(self.db, id)
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        self.db.field_name(field)
    }

    fn category(&self, field: &FieldMeta) -> String {
        self.db.category_label(&field.category)
    }

    fn levels(&self, _: FourCC) -> u32 {
        FIRST_LEVEL
    }

    fn raw(&self, id: FourCC, field: &FieldMeta, _: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, self.mode)
    }

    fn modified(&self, id: FourCC, field: &FieldMeta, _: u32) -> bool {
        self.map.is_modified(id, field)
    }
}

struct Abilities<'a> {
    db: &'a AbilityDatabase,
    map: &'a MapAbilities,
    strings: &'a MapStrings,
    mode: ArtMode,
}

impl Source for Abilities<'_> {
    fn changed(&self) -> Vec<(FourCC, bool)> {
        self.map
            .abilities()
            .into_iter()
            .map(|a| (a.id, a.is_custom))
            .collect()
    }

    fn game_ids(&self) -> &[String] {
        self.db.ids()
    }

    fn base(&self, id: FourCC) -> Option<FourCC> {
        self.map.base_of(self.db, id)
    }

    fn name(&self, id: FourCC) -> String {
        self.map.name(self.db, self.strings, id)
    }

    fn fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        self.map.fields(self.db, id)
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        self.db.field_name(field)
    }

    fn category(&self, field: &FieldMeta) -> String {
        self.db.category_label(&field.category)
    }

    fn levels(&self, id: FourCC) -> u32 {
        self.map.levels(self.db, id)
    }

    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, level, self.mode)
    }

    fn modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.map.is_modified(id, field, level)
    }
}

struct Leveled<'a> {
    db: &'a dyn LeveledData,
    map: &'a LeveledObjects,
    strings: &'a MapStrings,
    // Doodads name themselves with editor string keys (`WESTRING_DOOD_...`).
    units: &'a UnitDatabase,
    mode: ArtMode,
}

impl Source for Leveled<'_> {
    fn changed(&self) -> Vec<(FourCC, bool)> {
        self.map
            .objects()
            .into_iter()
            .map(|o| (o.id, o.is_custom))
            .collect()
    }

    fn game_ids(&self) -> &[String] {
        self.db.ids()
    }

    fn base(&self, id: FourCC) -> Option<FourCC> {
        self.map.base_of(self.db, id)
    }

    fn name(&self, id: FourCC) -> String {
        self.units
            .display_text(&self.map.name(self.db, self.strings, id))
    }

    fn fields(&self, id: FourCC) -> Vec<&FieldMeta> {
        match self.base(id) {
            Some(_) => self.db.fields().iter().collect(),
            None => Vec::new(),
        }
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        self.db.field_name(field)
    }

    fn category(&self, field: &FieldMeta) -> String {
        self.db.category_label(&field.category)
    }

    fn levels(&self, id: FourCC) -> u32 {
        self.map.levels(self.db, id)
    }

    fn raw(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<String> {
        self.map
            .raw_value(self.db, self.strings, id, field, level, self.mode)
    }

    fn modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.map.is_modified(id, field, level)
    }
}

struct Writer<'a> {
    units: &'a UnitDatabase,
    abilities: &'a AbilityDatabase,
    placeholders: &'a dyn PlaceholderSource,
    selection: &'a Selection,
}

impl Writer<'_> {
    fn display(&self, field: &FieldMeta, raw: Option<String>) -> String {
        let mut text = self.units.format_value(field, raw);

        if text.contains(PLACEHOLDER_START) {
            text = self.abilities.fill_placeholders(&text, self.placeholders);
        }

        if self.selection.plain_text {
            text = plain_text(&text);
        }

        text
    }

    fn value(&self, source: &dyn Source, id: FourCC, field: &FieldMeta, level: u32) -> Value {
        let raw = source.raw(id, field, level);

        Value {
            display: self.display(field, raw.clone()),
            raw: raw.unwrap_or_default(),
        }
    }

    fn object(
        &self,
        source: &dyn Source,
        id: FourCC,
        changed: Option<bool>,
        infos: &mut BTreeMap<String, FieldInfo>,
    ) -> Option<Object> {
        let base = source.base(id)?;
        let levels = source.levels(id).max(FIRST_LEVEL);
        let mut modified = Vec::new();
        let mut values = BTreeMap::new();

        for field in source.fields(id) {
            infos.entry(field.id.clone()).or_insert_with(|| FieldInfo {
                name: source.field_name(field),
                category: source.category(field),
                per_level: field.per_level,
            });

            let value = match field.per_level {
                true => Values::PerLevel(
                    (FIRST_LEVEL..=levels)
                        .map(|level| {
                            if source.modified(id, field, level) {
                                modified.push(format!("{}:{level}", field.id));
                            }

                            self.value(source, id, field, level)
                        })
                        .collect(),
                ),
                false => {
                    if source.modified(id, field, LEVEL_OF_PLAIN_FIELDS) {
                        modified.push(field.id.clone());
                    }

                    Values::One(self.value(source, id, field, LEVEL_OF_PLAIN_FIELDS))
                }
            };

            values.insert(field.id.clone(), value);
        }

        modified.sort();
        modified.dedup();
        Some(Object {
            id: id.to_string(),
            base: base.to_string(),
            name: source.name(id),
            custom: changed.unwrap_or_default(),
            changed: changed.is_some(),
            levels,
            modified,
            values,
        })
    }

    fn objects(&self, source: &dyn Source) -> (Vec<Object>, BTreeMap<String, FieldInfo>) {
        let changed: BTreeMap<FourCC, bool> = source.changed().into_iter().collect();
        let wanted: Option<HashSet<&str>> = self
            .selection
            .ids
            .as_ref()
            .map(|ids| ids.iter().map(String::as_str).collect());
        let mut ids: Vec<FourCC> = changed.keys().copied().collect();

        if self.selection.origin == Origin::All {
            ids.extend(source.game_ids().iter().filter_map(|id| FourCC::parse(id)));
        }

        ids.sort_by_key(ToString::to_string);
        ids.dedup();
        let mut infos = BTreeMap::new();
        let objects = ids
            .into_iter()
            .filter(|id| {
                wanted
                    .as_ref()
                    .is_none_or(|w| w.contains(id.to_string().as_str()))
            })
            .filter(|id| self.selection.origin.keeps(changed.get(id).copied()))
            .filter_map(|id| self.object(source, id, changed.get(&id).copied(), &mut infos))
            .collect();

        (objects, infos)
    }
}

pub fn export(
    game: &GameData<'_>,
    map: &MapData<'_>,
    map_name: Option<String>,
    selection: &Selection,
) -> Export {
    let strings = map.units.strings();
    let placeholders = MapPlaceholders {
        abilities: map.abilities,
        db: game.abilities,
        strings,
        mode: game.mode,
        others: game.units,
    };
    let writer = Writer {
        units: game.units,
        abilities: game.abilities,
        placeholders: &placeholders,
        selection,
    };
    let plain = |db, objects| Plain {
        db,
        map: objects,
        strings,
        mode: game.mode,
    };
    let leveled = |db, objects| Leveled {
        db,
        map: objects,
        strings,
        units: game.units,
        mode: game.mode,
    };
    let mut fields = BTreeMap::new();
    let mut objects = BTreeMap::new();

    for kind in &selection.types {
        let (list, infos) = match kind {
            ExportType::Units => writer.objects(&Units {
                db: game.units,
                map: map.units,
                mode: game.mode,
            }),
            ExportType::Items => writer.objects(&plain(game.items, map.items)),
            ExportType::Buffs => writer.objects(&plain(game.buffs, map.buffs)),
            ExportType::Destructables => {
                writer.objects(&plain(game.destructables, map.destructables))
            }
            ExportType::Abilities => writer.objects(&Abilities {
                db: game.abilities,
                map: map.abilities,
                strings,
                mode: game.mode,
            }),
            ExportType::Upgrades => writer.objects(&leveled(game.upgrades, map.upgrades)),
            ExportType::Doodads => writer.objects(&leveled(game.doodads, map.doodads)),
        };

        fields.insert(*kind, infos);
        objects.insert(*kind, list);
    }

    Export {
        format: FORMAT,
        version: FORMAT_VERSION,
        map: map_name,
        fields,
        objects,
    }
}

impl Export {
    pub fn to_json(&self) -> Result<String> {
        let mut text = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::Invalid(format!("Cannot write the export: {e}")))?;

        text.push('\n');
        Ok(text)
    }

    pub fn to_csv(&self, kind: ExportType, values: CsvValues) -> String {
        let objects = self
            .objects
            .get(&kind)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let no_fields = BTreeMap::new();
        let infos = self.fields.get(&kind).unwrap_or(&no_fields);
        let columns: Vec<(&str, &FieldInfo, usize)> = infos
            .iter()
            .map(|(id, info)| {
                let count = objects
                    .iter()
                    .filter_map(|o| o.values.get(id))
                    .map(|v| v.levels().len())
                    .max()
                    .unwrap_or(1);

                (id.as_str(), info, count)
            })
            .collect();
        let mut header: Vec<String> = ["id", "base", "name", "custom", "changed", "levels"]
            .map(str::to_owned)
            .to_vec();

        for (id, info, count) in &columns {
            for level in 1..=*count {
                header.push(match info.per_level {
                    true => format!("{} [{id}:{level}]", info.name),
                    false => format!("{} [{id}]", info.name),
                });
            }
        }

        let mut out = String::from(BYTE_ORDER_MARK);

        push_row(&mut out, &header);
        for object in objects {
            let mut row = vec![
                object.id.clone(),
                object.base.clone(),
                object.name.clone(),
                object.custom.to_string(),
                object.changed.to_string(),
                object.levels.to_string(),
            ];

            for (id, _, count) in &columns {
                let levels = object
                    .values
                    .get(*id)
                    .map(Values::levels)
                    .unwrap_or_default();

                for level in 0..*count {
                    row.push(match (levels.get(level), values) {
                        (Some(v), CsvValues::Display) => v.display.clone(),
                        (Some(v), CsvValues::Raw) => v.raw.clone(),
                        (None, _) => String::new(),
                    });
                }
            }

            push_row(&mut out, &row);
        }

        out
    }

    pub fn object_count(&self) -> usize {
        self.objects.values().map(Vec::len).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    Csv,
}

impl Format {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

pub fn write(
    export: &Export,
    format: Format,
    values: CsvValues,
    out: &Path,
) -> Result<Vec<PathBuf>> {
    let io = |path: &Path, e: std::io::Error| {
        crate::Error::Invalid(format!("Cannot write {}: {e}", path.display()))
    };

    match format {
        Format::Json => {
            std::fs::write(out, export.to_json()?).map_err(|e| io(out, e))?;
            Ok(vec![out.to_path_buf()])
        }
        Format::Csv if export.objects.len() == 1 => {
            let kind = export
                .objects
                .keys()
                .copied()
                .next()
                .unwrap_or(ExportType::Units);

            std::fs::write(out, export.to_csv(kind, values)).map_err(|e| io(out, e))?;
            Ok(vec![out.to_path_buf()])
        }
        Format::Csv => {
            std::fs::create_dir_all(out).map_err(|e| io(out, e))?;
            export
                .objects
                .keys()
                .map(|kind| {
                    let path = out.join(format!("{}.{CSV_EXTENSION}", kind.name()));

                    std::fs::write(&path, export.to_csv(*kind, values))
                        .map_err(|e| io(&path, e))?;
                    Ok(path)
                })
                .collect()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CsvValues {
    #[default]
    Display,
    Raw,
}

fn push_row(out: &mut String, cells: &[String]) {
    let quoted: Vec<String> = cells.iter().map(|c| csv_cell(c)).collect();

    out.push_str(&quoted.join(","));
    out.push_str(CSV_LINE_END);
}

fn csv_cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        return format!("\"{}\"", text.replace('"', "\"\""));
    }

    text.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(raw: &str) -> Value {
        Value {
            raw: raw.to_owned(),
            display: format!("shown {raw}"),
        }
    }

    fn object(id: &str, values: Vec<(&str, Values)>) -> Object {
        Object {
            id: id.to_owned(),
            base: id.to_owned(),
            name: format!("Name, \"{id}\""),
            custom: false,
            changed: false,
            levels: 2,
            modified: Vec::new(),
            values: values.into_iter().map(|(f, v)| (f.to_owned(), v)).collect(),
        }
    }

    fn info(name: &str, per_level: bool) -> FieldInfo {
        FieldInfo {
            name: name.to_owned(),
            category: "Stats".to_owned(),
            per_level,
        }
    }

    fn sample() -> Export {
        let kind = ExportType::Abilities;

        Export {
            format: FORMAT,
            version: FORMAT_VERSION,
            map: None,
            fields: BTreeMap::from([(
                kind,
                BTreeMap::from([
                    ("acdn".to_owned(), info("ACDN", true)),
                    ("anam".to_owned(), info("ANAM", false)),
                ]),
            )]),
            objects: BTreeMap::from([(
                kind,
                vec![
                    object(
                        "AHbz",
                        vec![
                            ("acdn", Values::PerLevel(vec![value("6"), value("7")])),
                            ("anam", Values::One(value("Blizzard"))),
                        ],
                    ),
                    object("AHhb", vec![("anam", Values::One(value("Holy\nLight")))]),
                ],
            )]),
        }
    }

    #[test]
    fn csv_has_a_column_per_level_and_quotes_what_needs_it() {
        let csv = sample().to_csv(ExportType::Abilities, CsvValues::Raw);
        let lines: Vec<&str> = csv
            .trim_start_matches(BYTE_ORDER_MARK)
            .split(CSV_LINE_END)
            .collect();

        assert_eq!(
            lines[0],
            "id,base,name,custom,changed,levels,ACDN [acdn:1],ACDN [acdn:2],ANAM [anam]"
        );
        assert_eq!(
            lines[1],
            "AHbz,AHbz,\"Name, \"\"AHbz\"\"\",false,false,2,6,7,Blizzard"
        );
        assert_eq!(
            lines[2],
            "AHhb,AHhb,\"Name, \"\"AHhb\"\"\",false,false,2,,,\"Holy\nLight\""
        );
        assert!(csv.starts_with(BYTE_ORDER_MARK));
    }

    #[test]
    fn csv_can_hold_display_values() {
        let csv = sample().to_csv(ExportType::Abilities, CsvValues::Display);

        assert!(csv.contains(",shown 6,shown 7,shown Blizzard"));
    }

    #[test]
    fn json_names_the_format_and_types() -> Result<()> {
        let json = sample().to_json()?;

        assert!(json.contains("\"format\": \"tinkerbench-objects\""));
        assert!(json.contains("\"abilities\": ["));
        assert!(json.contains("\"perLevel\": true"));
        assert!(json.contains("\"anam\": {\n"));
        assert!(json.contains("\"acdn\": [\n"));
        Ok(())
    }

    #[test]
    fn origins_keep_game_changed_or_custom_objects() {
        let game = None;
        let changed = Some(false);
        let custom = Some(true);

        assert!(
            [game, changed, custom]
                .into_iter()
                .all(|o| Origin::All.keeps(o))
        );
        assert_eq!(
            [game, changed, custom].map(|o| Origin::Changed.keeps(o)),
            [false, true, true]
        );
        assert_eq!(
            [game, changed, custom].map(|o| Origin::Custom.keeps(o)),
            [false, false, true]
        );
    }

    #[test]
    fn types_parse_by_name() {
        assert_eq!(ExportType::parse("Upgrades"), Some(ExportType::Upgrades));
        assert_eq!(ExportType::parse("quests"), None);
    }
}
