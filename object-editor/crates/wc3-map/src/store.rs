//! What a map's changes to every object type share: a gameplay table and a
//! cosmetic ("netsafe") skin table, the strings added for text fields, and
//! which of the two files the map had.
//!
//! Plain objects ([`PlainObjects`], and units through it) store changes without
//! a level; leveled ones ([`LeveledObjects`], and abilities through it) store
//! a level and a data column with each.
//!
//! [`PlainObjects`]: crate::plain::PlainObjects
//! [`LeveledObjects`]: crate::leveled::LeveledObjects

use std::collections::HashMap;

use wc3_data::game::ArtMode;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, ModSet, Modification, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};

use crate::transfer::{Tables, next_free_id};
use crate::values::{is_translatable, parse_value};
use crate::{Error, Map, Result};

/// An object the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

/// Which stored changes belong to a field, and how a new one is stored.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Slot<'a> {
    pub field: &'a FieldMeta,
    id: FourCC,
    /// `None` matches any level: fields without levels are stored at level 0,
    /// but not every editor did.
    level: Option<u32>,
    /// The level and data column of a new change; `None` in plain files.
    stored: Option<(u32, u32)>,
}

impl<'a> Slot<'a> {
    pub fn plain(field: &'a FieldMeta) -> Result<Self> {
        Ok(Self {
            field,
            id: field_id(field)?,
            level: None,
            stored: None,
        })
    }

    /// Data column 0; [`Slot::in_column`] sets another.
    pub fn leveled(field: &'a FieldMeta, level: u32) -> Result<Self> {
        let level = field.per_level.then_some(level);

        Ok(Self {
            field,
            id: field_id(field)?,
            level,
            stored: Some((level.unwrap_or_default(), 0)),
        })
    }

    /// The data column a per-level change is stored in (`DataA` = 1).
    pub fn in_column(self, column: u32) -> Self {
        let stored = match (self.level, self.stored) {
            (Some(level), Some(_)) => Some((level, column)),
            _ => self.stored,
        };

        Self { stored, ..self }
    }

    fn matches(&self, m: &Modification) -> bool {
        m.field == self.id && self.level.is_none_or(|level| m.level == Some(level))
    }
}

fn field_id(field: &FieldMeta) -> Result<FourCC> {
    FourCC::parse(&field.id).ok_or_else(|| Error::Invalid(format!("bad field ID {}", field.id)))
}

/// An object's entries in the map's two tables.
pub(crate) struct Found<'a> {
    pub id: FourCC,
    data: Option<&'a ObjectEntry>,
    skin: Option<&'a ObjectEntry>,
}

impl<'a> Found<'a> {
    pub fn in_map(&self) -> bool {
        self.data.is_some() || self.skin.is_some()
    }

    /// The object it copies, or itself when the game has it.
    pub fn base(&self, game_has: impl Fn(&str) -> bool) -> Option<FourCC> {
        match self.data.or(self.skin) {
            Some(entry) => Some(entry.base),
            None => game_has(&self.id.to_string()).then_some(self.id),
        }
    }

    pub fn modification(&self, slot: &Slot<'_>) -> Option<&'a Value> {
        let entries = if slot.field.netsafe {
            [self.skin, self.data]
        } else {
            [self.data, self.skin]
        };

        // Look in the file the field belongs to first; maps saved by older
        // editors keep every field in the gameplay file.
        entries.into_iter().flatten().find_map(|entry| {
            entry
                .mods()
                .filter(|m| slot.matches(m))
                .last()
                .map(|m| &m.value)
        })
    }

    /// The changed value, with text read from the string table.
    pub fn text(&self, slot: &Slot<'_>, strings: &MapStrings) -> Option<String> {
        self.modification(slot).map(|value| match value {
            Value::String(s) => strings.resolve(s).to_owned(),
            other => other.to_string(),
        })
    }
}

pub(crate) struct ObjectStore {
    files: [&'static str; 2],
    tables: Tables,
    loaded: Vec<&'static str>,
    added_strings: Vec<u32>,
}

impl ObjectStore {
    pub fn load(map: &Map, files: [&'static str; 2], layout: Layout) -> Result<Self> {
        let mut loaded = Vec::new();
        let mut read = |file: &'static str| -> Result<ObjectData> {
            match map.object_data(file)? {
                Some(data) => {
                    loaded.push(file);
                    Ok(data)
                }
                None => Ok(ObjectData::empty(layout)),
            }
        };

        let tables = Tables {
            data: read(files[0])?,
            skin: read(files[1])?,
        };

        Ok(Self {
            files,
            tables,
            loaded,
            added_strings: Vec::new(),
        })
    }

    pub fn empty(files: [&'static str; 2], layout: Layout) -> Self {
        Self::from_tables(
            files,
            Tables {
                data: ObjectData::empty(layout),
                skin: ObjectData::empty(layout),
            },
        )
    }

    pub fn from_tables(files: [&'static str; 2], tables: Tables) -> Self {
        Self {
            files,
            tables,
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    pub fn tables(&self) -> &Tables {
        &self.tables
    }

    pub fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
    }

    /// Custom objects first, then changed game objects, each once.
    pub fn objects(&self) -> Vec<MapObject> {
        let mut objects: Vec<MapObject> = Vec::new();
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
                    Some(&i) => objects[i].modified_fields += mods,
                    None => {
                        index.insert(id, objects.len());
                        objects.push(MapObject {
                            id,
                            base: entry.base,
                            is_custom,
                            modified_fields: mods,
                        });
                    }
                }
            }
        }

        objects
    }

    pub fn find(&self, id: FourCC) -> Found<'_> {
        Found {
            id,
            data: self.tables.data.find(id),
            skin: self.tables.skin.find(id),
        }
    }

    pub fn has(&self, id: FourCC) -> bool {
        self.find(id).in_map()
    }

    /// Stores `text` for the field, or drops the change when it is the base
    /// object's game value in both art modes (some art differs between SD and
    /// HD; those always keep the change). New text gets a string with
    /// `comment`, as the World Editor writes: `Units: H000 (name), Name (Name)`.
    pub fn set(
        &mut self,
        strings: &mut MapStrings,
        id: FourCC,
        slot: &Slot<'_>,
        text: &str,
        comment: &str,
        base_value: impl Fn(ArtMode) -> Option<String>,
    ) -> Result<()> {
        let field = slot.field;
        let parsed = parse_value(field, text)?;
        let same_as_base = [ArtMode::Sd, ArtMode::Hd].into_iter().all(|mode| {
            let base = base_value(mode).unwrap_or_default();

            parse_value(field, &base).is_ok_and(|value| value == parsed)
        });

        if same_as_base {
            self.reset(strings, id, slot);
            return Ok(());
        }

        let value = match parsed {
            Value::String(s) if is_translatable(field) => {
                Value::String(self.store_string(strings, id, slot, &s, comment))
            }
            other => other,
        };

        self.write(id, slot, value);
        Ok(())
    }

    fn write(&mut self, id: FourCC, slot: &Slot<'_>, value: Value) {
        let entry = entry_mut(&mut self.tables, slot.field.netsafe, id);

        if entry.sets.is_empty() {
            entry.sets.push(ModSet {
                flags: 0,
                mods: Vec::new(),
            });
        }

        let set = &mut entry.sets[0];

        match set.mods.iter_mut().find(|m| slot.matches(m)) {
            Some(existing) => existing.value = value,
            None => set.mods.push(Modification {
                field: slot.id,
                level: slot.stored.map(|(level, _)| level),
                column: slot.stored.map(|(_, column)| column),
                value,
                end: FourCC::ZERO,
            }),
        }
    }

    pub fn reset(&mut self, strings: &mut MapStrings, id: FourCC, slot: &Slot<'_>) -> bool {
        let mut removed = false;

        for data in [&mut self.tables.data, &mut self.tables.skin] {
            let entries = data.custom.iter_mut().chain(data.original.iter_mut());

            for entry in entries.filter(|e| e.effective_id() == id) {
                for set in &mut entry.sets {
                    let before = set.mods.len();

                    set.mods.retain(|m| !slot.matches(m));
                    removed |= set.mods.len() != before;
                }
            }

            data.original.retain(|e| e.mods().next().is_some());
        }

        self.drop_unused_strings(strings);
        removed
    }

    /// Removes a custom object; game objects stay.
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

    /// Adds a custom object copying `base`, numbered the way the World Editor
    /// does: `prefix` and three digits or letters (`A000`, `A001`, ...).
    pub fn create(
        &mut self,
        base: FourCC,
        prefix: u8,
        game_has: impl Fn(&str) -> bool,
    ) -> Option<FourCC> {
        let id = next_free_id(prefix, |id| !self.has(id) && !game_has(&id.to_string()))?;

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

        Some(id)
    }

    /// Updates the string a text field already refers to, or adds a new one.
    fn store_string(
        &mut self,
        strings: &mut MapStrings,
        id: FourCC,
        slot: &Slot<'_>,
        text: &str,
        comment: &str,
    ) -> String {
        let current = self.find(id).modification(slot).map(Value::to_string);
        let existing = current.and_then(|c| Some((reference_id(&c)?, c)));

        if let Some((number, reference)) = existing
            && strings.set(number, text)
        {
            return reference;
        }

        let reference = strings.add(text, Some(comment));

        if let Some(number) = reference_id(&reference) {
            self.added_strings.push(number);
        }

        reference
    }

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

    /// The files to write: those with changes, and those the map had (now
    /// emptied of changes).
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        self.files
            .into_iter()
            .zip(self.tables.both())
            .filter(|(file, data)| {
                !data.custom.is_empty() || !data.original.is_empty() || self.loaded.contains(file)
            })
            .map(|(file, data)| (file, data.to_bytes()))
            .collect()
    }
}

/// The object's entry in one table, added when missing: a custom entry when
/// the other table has the object as custom, else a change to a game object.
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

#[cfg(test)]
mod tests {
    use super::*;

    const FILES: [&str; 2] = ["war3map.w3a", "war3mapSkin.w3a"];

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

    fn store(data: Vec<ObjectEntry>, skin: Vec<ObjectEntry>) -> ObjectStore {
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

        ObjectStore::from_tables(FILES, tables)
    }

    #[test]
    fn lists_custom_then_changed_objects_once_with_changes_from_both_files() {
        let map = store(
            vec![
                entry("AHhb", "A000", vec![change("Hhb1", 1), change("Hhb1", 2)]),
                entry("AHds", "", vec![change("acdn", 1)]),
            ],
            vec![
                entry("AHhb", "A000", vec![change("anam", 0)]),
                entry("AHds", "", vec![change("anam", 0)]),
                entry("AHtb", "", vec![change("anam", 0)]),
            ],
        );

        let listed: Vec<(String, bool, usize)> = map
            .objects()
            .iter()
            .map(|a| (a.id.to_string(), a.is_custom, a.modified_fields))
            .collect();
        let expected = [("A000", true, 3), ("AHds", false, 2), ("AHtb", false, 1)]
            .map(|(code, custom, changed)| (code.to_owned(), custom, changed));

        assert_eq!(listed, expected);
        assert!(map.has(id("AHds")));
        assert!(!map.has(id("AHhb")));
    }

    #[test]
    fn removing_a_custom_object_drops_it_and_its_new_strings() {
        let mut map = store(
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
            "game objects are not removed"
        );
        assert!(map.remove(&mut strings, id("A000")));
        assert!(!map.has(id("A000")));
        assert!(map.objects().is_empty());
        assert_eq!(strings.get(added_id), None, "its new string is gone");
        let kept_id = reference_id(&kept).unwrap_or_default();

        assert_eq!(strings.get(kept_id), Some("Kept"), "other strings stay");
    }

    #[test]
    fn writes_files_only_when_needed() {
        let empty = store(Vec::new(), Vec::new());

        assert!(empty.files().is_empty());

        let changed = store(vec![entry("AHds", "", vec![change("acdn", 1)])], Vec::new());
        let names: Vec<&str> = changed.files().iter().map(|(n, _)| *n).collect();

        assert_eq!(names, [FILES[0]]);
    }

    #[test]
    fn a_custom_object_missing_from_a_table_gets_a_custom_entry() {
        let mut map = store(vec![entry("AHhb", "A000", Vec::new())], Vec::new());

        let created = entry_mut(&mut map.tables, true, id("A000"));

        assert_eq!((created.base, created.id), (id("AHhb"), id("A000")));
        assert!(map.tables.skin.original.is_empty());
    }
}
