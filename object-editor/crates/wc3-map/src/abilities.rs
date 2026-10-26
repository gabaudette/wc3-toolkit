//! A map's ability changes on top of the game's ability data.
//!
//! Like units, Reforged splits them in two files: gameplay fields in
//! `war3map.w3a` and cosmetic ("netsafe") fields such as names, tooltips and
//! art in `war3mapSkin.w3a`. Every change also stores a level (0 for fields
//! without levels) and a data column (`DataA` = 1, 0 for other fields).
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use std::collections::HashMap;

use wc3_data::abilities::{AbilityDatabase, AbilityInfo, PlaceholderSource};
use wc3_data::game::ArtMode;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, ModSet, Modification, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};

use crate::transfer::{Tables, next_free_id};
use crate::values::{is_translatable, parse_value};
use crate::{ABILITIES_FILE, ABILITY_SKINS_FILE, Error, Map, Result};

/// Custom abilities are numbered `A000`, `A001`, ... like the World Editor does.
const ID_PREFIX: u8 = b'A';
const TRUE: &str = "1";

/// Fields the editor reads to describe an ability.
const NAME: &str = "anam";
const SUFFIX: &str = "ansf";
const ICON: &str = "aart";
const HERO: &str = "aher";
const ITEM: &str = "aite";
const RACE: &str = "arac";
const LEVELS: &str = "alev";
const REQUIRED_LEVEL: &str = "arlv";
const HOTKEY: &str = "ahky";
const LEARN_HOTKEY: &str = "arhk";
const BUTTON: (&str, &str) = ("abpx", "abpy");
const LEARN_BUTTON: (&str, &str) = ("arpx", "arpy");

pub struct MapAbilities {
    tables: Tables,
    /// Ability files the map had when loaded; others are only written once
    /// they hold something, so untouched maps keep their files as they are.
    loaded: Vec<&'static str>,
    /// Strings this editor added since loading. Only these are removed again
    /// when nothing refers to them; strings already in the map are never touched.
    added_strings: Vec<u32>,
}

/// An ability the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct MapAbility {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

impl MapAbilities {
    pub fn load(map: &Map) -> Result<Self> {
        let mut loaded = Vec::new();
        let mut read = |file: &'static str| -> Result<ObjectData> {
            match map.object_data(file)? {
                Some(data) => {
                    loaded.push(file);
                    Ok(data)
                }
                None => Ok(ObjectData::empty(Layout::Leveled)),
            }
        };

        let tables = Tables {
            data: read(ABILITIES_FILE)?,
            skin: read(ABILITY_SKINS_FILE)?,
        };
        Ok(Self {
            tables,
            loaded,
            added_strings: Vec::new(),
        })
    }

    /// No changes: the game's abilities as they are.
    pub fn empty() -> Self {
        Self {
            tables: Tables {
                data: ObjectData::empty(Layout::Leveled),
                skin: ObjectData::empty(Layout::Leveled),
            },
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    /// The ability tables (gameplay, skin), for moving objects between maps.
    pub(crate) fn tables(&self) -> &Tables {
        &self.tables
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
    }

    #[cfg(test)]
    pub(crate) fn from_tables(tables: Tables) -> Self {
        Self {
            tables,
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    /// Custom abilities first, then changed game abilities, each listed once.
    pub fn abilities(&self) -> Vec<MapAbility> {
        let mut abilities: Vec<MapAbility> = Vec::new();
        let mut index: HashMap<FourCC, usize> = HashMap::new();
        let (data, skin) = (&self.tables.data, &self.tables.skin);
        let tables = [
            (&data.custom, true),
            (&skin.custom, true),
            (&data.original, false),
            (&skin.original, false),
        ];

        for (entries, is_custom) in tables {
            for entry in entries {
                let id = entry.effective_id();
                let mods = entry.mods().count();

                match index.get(&id) {
                    Some(&i) => abilities[i].modified_fields += mods,
                    None => {
                        index.insert(id, abilities.len());
                        abilities.push(MapAbility {
                            id,
                            base: entry.base,
                            is_custom,
                            modified_fields: mods,
                        });
                    }
                }
            }
        }

        abilities
    }

    /// The ability's entries in both tables, found once for several lookups.
    fn find(&self, id: FourCC) -> Found<'_> {
        Found {
            id,
            data: self.tables.data.find(id),
            skin: self.tables.skin.find(id),
        }
    }

    /// Whether the map adds or changes this ability.
    pub fn has(&self, id: FourCC) -> bool {
        self.find(id).in_map()
    }

    /// The game ability a map ability is based on: the base of a custom
    /// ability, or the ability itself. `None` when neither the map nor the
    /// game has the ID.
    pub fn base_of(&self, db: &AbilityDatabase, id: FourCC) -> Option<FourCC> {
        self.find(id).base(db)
    }

    /// The base code that decides the ability's data fields (`AHhb`).
    pub fn code(&self, db: &AbilityDatabase, id: FourCC) -> Option<String> {
        db.code(&self.base_of(db, id)?.to_string())
    }

    /// Fields the Object Editor shows for this ability: the generic ones for
    /// its kind (hero, item or unit ability) and its base code's data fields.
    pub fn fields<'a>(&self, db: &'a AbilityDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        let found = self.find(id);
        let Some(code) = found.base(db).and_then(|base| db.code(&base.to_string())) else {
            return Vec::new();
        };

        let flag = |field_id| found.plain_value(db, field_id).as_deref() == Some(TRUE);
        db.fields_for(&code, flag(HERO), flag(ITEM))
    }

    /// The number of levels, at least 1.
    pub fn levels(&self, db: &AbilityDatabase, id: FourCC) -> u32 {
        self.find(id).levels(db)
    }

    /// The map's change to a field at `level` (ignored for fields without levels).
    pub fn modification(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<&Value> {
        self.find(id).modification(field, level)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.modification(id, field, level).is_some()
    }

    /// The field's raw value at `level`: the map's change if there is one
    /// (with string references resolved), otherwise the base ability's game data.
    pub fn raw_value(
        &self,
        db: &AbilityDatabase,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        self.find(id).raw_value(db, strings, field, level, mode)
    }

    pub fn name(&self, db: &AbilityDatabase, strings: &MapStrings, id: FourCC) -> String {
        db.field(NAME)
            .and_then(|field| self.raw_value(db, strings, id, field, 0, ArtMode::Sd))
            .unwrap_or_else(|| id.to_string())
    }

    /// What an ability card shows, with the map's changes applied. `None`
    /// when neither the map nor the game has the ID.
    pub fn info(
        &self,
        db: &AbilityDatabase,
        strings: &MapStrings,
        id: FourCC,
        mode: ArtMode,
    ) -> Option<AbilityInfo> {
        let found = self.find(id);
        let code = db.code(&found.base(db)?.to_string())?;
        let text = |field_id: &str| {
            db.field(field_id)
                .and_then(|field| found.raw_value(db, strings, field, 0, mode))
                .filter(|v| !v.is_empty())
        };
        let number = |field_id: &str| {
            found
                .plain_value(db, field_id)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or_default()
        };
        let button = |(x, y): (&str, &str)| {
            let x = found.plain_value(db, x)?.trim().parse().ok()?;
            let y = found.plain_value(db, y)?.trim().parse().ok()?;
            Some((x, y))
        };

        Some(AbilityInfo {
            id: id.to_string(),
            code,
            name: text(NAME).unwrap_or_else(|| id.to_string()),
            suffix: text(SUFFIX),
            icon: text(ICON),
            hero: found.plain_value(db, HERO).as_deref() == Some(TRUE),
            item: found.plain_value(db, ITEM).as_deref() == Some(TRUE),
            race: text(RACE).unwrap_or_default(),
            levels: found.levels(db),
            required_level: number(REQUIRED_LEVEL),
            hotkey: text(HOTKEY),
            learn_hotkey: text(LEARN_HOTKEY),
            button: button(BUTTON),
            learn_button: button(LEARN_BUTTON),
        })
    }

    /// Sets a field at `level` (ignored for fields without levels) from text
    /// as typed in an editor. Text fields are stored in the map's string table.
    /// Setting the base ability's own value removes the change instead.
    pub fn set(
        &mut self,
        db: &AbilityDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        text: &str,
    ) -> Result<()> {
        let field_id = FourCC::parse(&field.id)
            .ok_or_else(|| Error::Invalid(format!("bad field ID {}", field.id)))?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!("unknown ability {id}")));
        };

        if field.per_level && level == 0 {
            return Err(Error::Invalid(format!("{} needs a level", field.id)));
        }

        let parsed = parse_value(field, text)?;
        if same_as_base(db, base, field, level, &parsed) {
            self.reset(strings, id, field, level);
            return Ok(());
        }

        let value = match parsed {
            Value::String(s) if is_translatable(field) => {
                Value::String(self.store_string(db, strings, id, field, level, &s))
            }
            other => other,
        };

        let (stored_level, column) = stored_position(field, level);
        let entry = entry_mut(&mut self.tables, field.netsafe, id);
        if entry.sets.is_empty() {
            entry.sets.push(ModSet {
                flags: 0,
                mods: Vec::new(),
            });
        }

        let set = &mut entry.sets[0];
        match set
            .mods
            .iter_mut()
            .find(|m| m.field == field_id && at_level(field, m, level))
        {
            Some(existing) => existing.value = value,
            None => set.mods.push(Modification {
                field: field_id,
                level: Some(stored_level),
                column: Some(column),
                value,
                end: FourCC::ZERO,
            }),
        }

        Ok(())
    }

    /// Removes the map's change to a field at `level` (every change, for
    /// fields without levels) so the base ability's value applies again.
    /// Returns `false` when the field was not changed.
    pub fn reset(
        &mut self,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
    ) -> bool {
        let Some(field_id) = FourCC::parse(&field.id) else {
            return false;
        };

        let mut removed = false;
        for data in [&mut self.tables.data, &mut self.tables.skin] {
            let entries = data.custom.iter_mut().chain(data.original.iter_mut());
            for entry in entries.filter(|e| e.effective_id() == id) {
                for set in &mut entry.sets {
                    let before = set.mods.len();
                    set.mods
                        .retain(|m| !(m.field == field_id && at_level(field, m, level)));
                    removed |= set.mods.len() != before;
                }
            }

            // A changed game ability with no changes left is dropped entirely.
            data.original.retain(|e| e.mods().next().is_some());
        }

        self.drop_unused_strings(strings);
        removed
    }

    /// Removes strings this editor added that no ability field refers to anymore.
    fn drop_unused_strings(&mut self, strings: &mut MapStrings) {
        let referenced: Vec<u32> = self
            .tables
            .both()
            .iter()
            .flat_map(|data| data.custom.iter().chain(&data.original))
            .flat_map(ObjectEntry::mods)
            .filter_map(|m| match &m.value {
                Value::String(s) => reference_id(s),
                _ => None,
            })
            .collect();

        self.added_strings.retain(|id| {
            if referenced.contains(id) {
                return true;
            }

            strings.remove(*id);
            false
        });
    }

    /// Creates a custom ability copying `base` and returns its new ID, the
    /// next free one of `A000`, `A001`, ...
    pub fn create(&mut self, db: &AbilityDatabase, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!("unknown base ability {base}")));
        }

        let id = next_free_id(ID_PREFIX, |id| {
            !self.has(id) && !db.contains(&id.to_string())
        })
        .ok_or_else(|| Error::Invalid(format!("no free ID for {base}")))?;
        for data in [&mut self.tables.data, &mut self.tables.skin] {
            data.custom.push(ObjectEntry {
                base,
                id,
                sets: vec![ModSet {
                    flags: 0,
                    mods: Vec::new(),
                }],
            });
        }

        Ok(id)
    }

    /// Removes a custom ability and the strings this editor added for it, as
    /// if it had never been created. Returns `false` for game abilities (their
    /// changes are reset field by field) and unknown IDs.
    pub fn remove(&mut self, strings: &mut MapStrings, id: FourCC) -> bool {
        let mut removed = false;
        for data in [&mut self.tables.data, &mut self.tables.skin] {
            let before = data.custom.len();
            data.custom.retain(|e| e.id != id);
            removed |= data.custom.len() != before;
        }

        self.drop_unused_strings(strings);
        removed
    }

    /// Updates the string a text field already refers to, or adds a new one
    /// with a comment like the World Editor's: `Abilities: A000 (name), Name (Name)`.
    fn store_string(
        &mut self,
        db: &AbilityDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        text: &str,
    ) -> String {
        let current = self.modification(id, field, level).map(Value::to_string);
        let existing = current.and_then(|c| Some((reference_id(&c)?, c)));

        if let Some((number, reference)) = existing
            && strings.set(number, text)
        {
            return reference;
        }

        let mut comment = format!(
            "Abilities: {id} ({}), {} ({})",
            self.name(db, strings, id),
            db.field_name(field),
            field.field
        );
        if field.per_level {
            comment.push_str(&format!(", level {level}"));
        }

        let reference = strings.add(text, Some(&comment));
        if let Some(number) = reference_id(&reference) {
            self.added_strings.push(number);
        }

        reference
    }

    /// The ability files to write: those the map had, and new ones that now
    /// hold something.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        [
            (ABILITIES_FILE, &self.tables.data),
            (ABILITY_SKINS_FILE, &self.tables.skin),
        ]
        .into_iter()
        .filter(|(file, data)| {
            !data.custom.is_empty() || !data.original.is_empty() || self.loaded.contains(file)
        })
        .map(|(file, data)| (file, data.to_bytes()))
        .collect()
    }
}

/// An ability's entries in the gameplay and skin tables. Finding them once
/// keeps listing every ability fast on maps that change hundreds of them.
struct Found<'a> {
    id: FourCC,
    data: Option<&'a ObjectEntry>,
    skin: Option<&'a ObjectEntry>,
}

impl<'a> Found<'a> {
    fn in_map(&self) -> bool {
        self.data.is_some() || self.skin.is_some()
    }

    fn base(&self, db: &AbilityDatabase) -> Option<FourCC> {
        match self.data.or(self.skin) {
            Some(entry) => Some(entry.base),
            None => db.contains(&self.id.to_string()).then_some(self.id),
        }
    }

    fn modification(&self, field: &FieldMeta, level: u32) -> Option<&'a Value> {
        let field_id = FourCC::parse(&field.id)?;
        let entries = if field.netsafe {
            [self.skin, self.data]
        } else {
            [self.data, self.skin]
        };

        // Look in the file the field belongs to first; maps saved by older
        // editors keep every field in war3map.w3a.
        entries.into_iter().flatten().find_map(|entry| {
            entry
                .mods()
                .filter(|m| m.field == field_id && at_level(field, m, level))
                .last()
                .map(|m| &m.value)
        })
    }

    fn raw_value(
        &self,
        db: &AbilityDatabase,
        strings: &MapStrings,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        if let Some(value) = self.modification(field, level) {
            return Some(match value {
                Value::String(s) => strings.resolve(s).to_owned(),
                other => other.to_string(),
            });
        }

        let base = self.base(db)?;
        db.raw_value(&base.to_string(), field, level, mode)
    }

    /// A value that is never text and has no levels, by field ID: no string
    /// table needed.
    fn plain_value(&self, db: &AbilityDatabase, field_id: &str) -> Option<String> {
        let field = db.field(field_id)?;
        if let Some(value) = self.modification(field, 0) {
            return Some(value.to_string());
        }

        let base = self.base(db)?;
        db.raw_value(&base.to_string(), field, 0, ArtMode::Sd)
    }

    fn levels(&self, db: &AbilityDatabase) -> u32 {
        self.plain_value(db, LEVELS)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or_default()
            .max(1)
    }
}

/// Whether a stored change is the one for `level`. Fields without levels
/// match any level: maps store them at level 0, but not every editor did.
fn at_level(field: &FieldMeta, m: &Modification, level: u32) -> bool {
    !field.per_level || m.level == Some(level)
}

/// The level and data column a change is stored with.
fn stored_position(field: &FieldMeta, level: u32) -> (u32, u32) {
    if field.per_level {
        (level, field.data_column.unwrap_or_default())
    } else {
        (0, 0)
    }
}

/// The ability's entry in the gameplay or skin table, created when missing.
/// A custom ability missing from this table gets a custom entry with its base.
fn entry_mut(tables: &mut Tables, in_skin: bool, id: FourCC) -> &mut ObjectEntry {
    let custom_base = tables
        .both()
        .iter()
        .flat_map(|t| &t.custom)
        .find(|e| e.effective_id() == id)
        .map(|e| e.base);
    let data = if in_skin {
        &mut tables.skin
    } else {
        &mut tables.data
    };

    if let Some(i) = data.custom.iter().position(|e| e.effective_id() == id) {
        return &mut data.custom[i];
    }

    if let Some(base) = custom_base {
        data.custom.push(ObjectEntry {
            base,
            id,
            sets: Vec::new(),
        });
        let last = data.custom.len() - 1;
        return &mut data.custom[last];
    }

    let index = data.original.iter().position(|e| e.effective_id() == id);
    match index {
        Some(i) => &mut data.original[i],
        None => {
            data.original.push(ObjectEntry {
                base: id,
                id: FourCC::ZERO,
                sets: Vec::new(),
            });
            let last = data.original.len() - 1;
            &mut data.original[last]
        }
    }
}

/// Whether `value` equals the base ability's game value at `level` in both
/// art modes (some art differs between SD and HD; those always keep the change).
fn same_as_base(
    db: &AbilityDatabase,
    base: FourCC,
    field: &FieldMeta,
    level: u32,
    value: &Value,
) -> bool {
    let base = base.to_string();
    [ArtMode::Sd, ArtMode::Hd].iter().all(|mode| {
        let base_value = db.raw_value(&base, field, level, *mode).unwrap_or_default();
        parse_value(field, &base_value).is_ok_and(|parsed| &parsed == value)
    })
}

/// Tooltip placeholders (`<A000,DataA1>`) filled from the map's abilities
/// where the map adds or changes them, then from `others`.
pub struct MapPlaceholders<'a> {
    pub abilities: &'a MapAbilities,
    pub db: &'a AbilityDatabase,
    pub strings: &'a MapStrings,
    pub mode: ArtMode,
    pub others: &'a dyn PlaceholderSource,
}

impl PlaceholderSource for MapPlaceholders<'_> {
    fn placeholder(&self, id: &str, column: &str) -> Option<String> {
        let in_map = FourCC::parse(id).filter(|code| self.abilities.has(*code));
        if let Some(ability) = in_map
            && let Some(code) = self.abilities.code(self.db, ability)
            && let Some((field, level)) = self.db.column_field(&code, column)
        {
            return self.abilities.raw_value(
                self.db,
                self.strings,
                ability,
                field,
                level,
                self.mode,
            );
        }

        self.others.placeholder(id, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(code: &str) -> FourCC {
        FourCC::parse(code).unwrap_or(FourCC::ZERO)
    }

    fn change(field: &str, level: u32) -> Modification {
        Modification {
            field: id(field),
            level: Some(level),
            column: Some(0),
            value: Value::Int(1),
            end: FourCC::ZERO,
        }
    }

    fn entry(base: &str, own: &str, mods: Vec<Modification>) -> ObjectEntry {
        ObjectEntry {
            base: id(base),
            id: if own.is_empty() {
                FourCC::ZERO
            } else {
                id(own)
            },
            sets: vec![ModSet { flags: 0, mods }],
        }
    }

    fn abilities(data: Vec<ObjectEntry>, skin: Vec<ObjectEntry>) -> MapAbilities {
        let mut tables = Tables {
            data: ObjectData::empty(Layout::Leveled),
            skin: ObjectData::empty(Layout::Leveled),
        };
        for (table, entries) in [(&mut tables.data, data), (&mut tables.skin, skin)] {
            for e in entries {
                if e.id.is_zero() {
                    table.original.push(e);
                } else {
                    table.custom.push(e);
                }
            }
        }

        MapAbilities::from_tables(tables)
    }

    #[test]
    fn removing_a_custom_ability_drops_it_and_its_new_strings() {
        let mut map = abilities(
            vec![entry("AHhb", "A000", vec![change("Hhb1", 1)])],
            vec![entry("AHhb", "A000", Vec::new())],
        );
        let mut strings = MapStrings::default();
        let kept = strings.add("Kept", None);
        let added = strings.add("Fire Bolt", None);
        let added_id = reference_id(&added).unwrap_or_default();
        map.added_strings.push(added_id);
        map.tables.skin.custom[0].sets[0].mods.push(Modification {
            field: id("anam"),
            level: Some(0),
            column: Some(0),
            value: Value::String(added),
            end: FourCC::ZERO,
        });

        assert!(
            !map.remove(&mut strings, id("AHds")),
            "game abilities are not removed"
        );
        assert!(map.remove(&mut strings, id("A000")));
        assert!(!map.has(id("A000")));
        assert!(map.abilities().is_empty());
        assert_eq!(strings.get(added_id), None, "its new string is gone");
        let kept_id = reference_id(&kept).unwrap_or_default();
        assert_eq!(strings.get(kept_id), Some("Kept"), "other strings stay");
    }

    #[test]
    fn lists_custom_then_changed_abilities_once() {
        let map = abilities(
            vec![
                entry("AHhb", "A000", vec![change("Hhb1", 1), change("Hhb1", 2)]),
                entry("AHds", "", vec![change("acdn", 1)]),
            ],
            vec![entry("AHhb", "A000", vec![change("anam", 0)])],
        );

        let listed: Vec<(String, bool, usize)> = map
            .abilities()
            .iter()
            .map(|a| (a.id.to_string(), a.is_custom, a.modified_fields))
            .collect();
        assert_eq!(
            listed,
            [("A000".to_owned(), true, 3), ("AHds".to_owned(), false, 1)]
        );
        assert!(map.has(id("AHds")));
        assert!(!map.has(id("AHhb")));
    }

    #[test]
    fn writes_ability_files_only_when_needed() {
        let empty = abilities(Vec::new(), Vec::new());
        assert!(empty.files().is_empty());

        let changed = abilities(vec![entry("AHds", "", vec![change("acdn", 1)])], Vec::new());
        let names: Vec<&str> = changed.files().iter().map(|(n, _)| *n).collect();
        assert_eq!(names, [ABILITIES_FILE]);
    }

    #[test]
    fn a_custom_ability_missing_from_a_table_gets_a_custom_entry() {
        let mut map = abilities(vec![entry("AHhb", "A000", Vec::new())], Vec::new());

        let created = entry_mut(&mut map.tables, true, id("A000"));
        assert_eq!((created.base, created.id), (id("AHhb"), id("A000")));
        assert!(map.tables.skin.original.is_empty());
    }
}
