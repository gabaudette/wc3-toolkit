//! A map's changes to objects without levels other than units: items, buffs
//! and destructables, on top of the game's data (a [`UnitDatabase`] loaded
//! for that catalog).
//!
//! Like units, Reforged splits them in two files: gameplay fields in
//! `war3map.w3t` (items), `.w3h` (buffs) or `.w3b` (destructables), and
//! cosmetic ("netsafe") fields such as names, tooltips and art in the
//! matching `war3mapSkin` file.
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use std::collections::HashMap;

use wc3_data::game::ArtMode;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, ModSet, Modification, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};
use wc3_data::units::UnitDatabase;

use crate::transfer::{Tables, next_free_id};
use crate::values::{is_translatable, parse_value};
use crate::{
    BUFF_SKINS_FILE, BUFFS_FILE, DESTRUCTABLE_SKINS_FILE, DESTRUCTABLES_FILE, Error,
    ITEM_SKINS_FILE, ITEMS_FILE, Map, Result,
};

/// One kind of object without levels: its files, how custom IDs start and
/// how the World Editor names it in string comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlainKind {
    data_file: &'static str,
    skin_file: &'static str,
    /// Custom objects are numbered `I000`, `I001`, ... like the World Editor does.
    id_prefix: u8,
    /// "Items", as string comments read: `Items: I000 (name), Name (Name)`.
    label: &'static str,
    /// "item", for messages.
    singular: &'static str,
    /// The field holding the object's name.
    name_field: &'static str,
}

pub const ITEMS: PlainKind = PlainKind {
    data_file: ITEMS_FILE,
    skin_file: ITEM_SKINS_FILE,
    id_prefix: b'I',
    label: "Items",
    singular: "item",
    name_field: "unam",
};

pub const BUFFS: PlainKind = PlainKind {
    data_file: BUFFS_FILE,
    skin_file: BUFF_SKINS_FILE,
    id_prefix: b'B',
    label: "Buffs",
    singular: "buff",
    name_field: "fnam",
};

pub const DESTRUCTABLES: PlainKind = PlainKind {
    data_file: DESTRUCTABLES_FILE,
    skin_file: DESTRUCTABLE_SKINS_FILE,
    id_prefix: b'B',
    label: "Destructables",
    singular: "destructable",
    name_field: "bnam",
};

pub struct PlainObjects {
    kind: PlainKind,
    tables: Tables,
    /// Files the map had when loaded; others are only written once they
    /// hold something, so untouched maps keep their files as they are.
    loaded: Vec<&'static str>,
    /// Strings this editor added since loading. Only these are removed again
    /// when nothing refers to them; strings already in the map are never touched.
    added_strings: Vec<u32>,
}

/// An object the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct PlainObject {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

impl PlainObjects {
    pub fn load(map: &Map, kind: PlainKind) -> Result<Self> {
        let mut loaded = Vec::new();
        let mut read = |file: &'static str| -> Result<ObjectData> {
            match map.object_data(file)? {
                Some(data) => {
                    loaded.push(file);
                    Ok(data)
                }
                None => Ok(ObjectData::empty(Layout::Plain)),
            }
        };

        let tables = Tables {
            data: read(kind.data_file)?,
            skin: read(kind.skin_file)?,
        };
        Ok(Self {
            kind,
            tables,
            loaded,
            added_strings: Vec::new(),
        })
    }

    /// No changes: the game's objects as they are.
    pub fn empty(kind: PlainKind) -> Self {
        Self {
            kind,
            tables: Tables {
                data: ObjectData::empty(Layout::Plain),
                skin: ObjectData::empty(Layout::Plain),
            },
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    /// The tables (gameplay, skin), for moving objects between maps.
    pub(crate) fn tables(&self) -> &Tables {
        &self.tables
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
    }

    #[cfg(test)]
    pub(crate) fn from_tables(kind: PlainKind, tables: Tables) -> Self {
        Self {
            kind,
            tables,
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    /// Custom objects first, then changed game objects, each listed once.
    pub fn objects(&self) -> Vec<PlainObject> {
        let mut objects: Vec<PlainObject> = Vec::new();
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
                        objects.push(PlainObject {
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

    /// Whether the map adds or changes this object.
    pub fn has(&self, id: FourCC) -> bool {
        self.tables.data.find(id).is_some() || self.tables.skin.find(id).is_some()
    }

    /// The game object a map object is based on: the base of a custom one, or
    /// the object itself. `None` when neither the map nor the game has the ID.
    pub fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        match self
            .tables
            .data
            .find(id)
            .or_else(|| self.tables.skin.find(id))
        {
            Some(entry) => Some(entry.base),
            None => db.contains(&id.to_string()).then_some(id),
        }
    }

    /// Fields the Object Editor shows for this object.
    pub fn fields<'a>(&self, db: &'a UnitDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        self.base_of(db, id)
            .map(|base| db.fields_for(&base.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta) -> Option<&Value> {
        let field_id = FourCC::parse(&field.id)?;
        let tables = if field.netsafe {
            [&self.tables.skin, &self.tables.data]
        } else {
            [&self.tables.data, &self.tables.skin]
        };

        // Look in the file the field belongs to first; maps saved by older
        // editors keep every field in the gameplay file.
        tables.into_iter().find_map(|data| {
            data.find(id)?
                .mods()
                .filter(|m| m.field == field_id)
                .last()
                .map(|m| &m.value)
        })
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        self.modification(id, field).is_some()
    }

    /// The field's raw value: the map's change if there is one (with string
    /// references resolved), otherwise the base object's game data.
    pub fn raw_value(
        &self,
        db: &UnitDatabase,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        if let Some(value) = self.modification(id, field) {
            return Some(match value {
                Value::String(s) => strings.resolve(s).to_owned(),
                other => other.to_string(),
            });
        }

        let base = self.base_of(db, id)?;
        db.raw_value(&base.to_string(), field, mode)
    }

    pub fn name(&self, db: &UnitDatabase, strings: &MapStrings, id: FourCC) -> String {
        db.field(self.kind.name_field)
            .and_then(|field| self.raw_value(db, strings, id, field, ArtMode::Sd))
            .filter(|name| !name.is_empty())
            .map(|name| db.display_text(&name))
            .or_else(|| self.base_of(db, id).map(|base| db.name(&base.to_string())))
            .unwrap_or_else(|| id.to_string())
    }

    /// Sets a field from text as typed in an editor. Text fields are stored in
    /// the map's string table. Setting the base object's own value removes the
    /// change instead.
    pub fn set(
        &mut self,
        db: &UnitDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> Result<()> {
        let field_id = FourCC::parse(&field.id)
            .ok_or_else(|| Error::Invalid(format!("bad field ID {}", field.id)))?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!(
                "unknown {} {id}",
                self.kind.singular
            )));
        };

        let parsed = parse_value(field, text)?;
        if same_as_base(db, base, field, &parsed) {
            self.reset(strings, id, field);
            return Ok(());
        }

        let value = match parsed {
            Value::String(s) if is_translatable(field) => {
                Value::String(self.store_string(db, strings, id, field, &s))
            }
            other => other,
        };

        let entry = entry_mut(&mut self.tables, field.netsafe, id);
        if entry.sets.is_empty() {
            entry.sets.push(ModSet {
                flags: 0,
                mods: Vec::new(),
            });
        }

        let set = &mut entry.sets[0];
        match set.mods.iter_mut().find(|m| m.field == field_id) {
            Some(existing) => existing.value = value,
            None => set.mods.push(Modification {
                field: field_id,
                level: None,
                column: None,
                value,
                end: FourCC::ZERO,
            }),
        }

        Ok(())
    }

    /// Removes the map's change to a field so the base object's value applies
    /// again. Returns `false` when the field was not changed.
    pub fn reset(&mut self, strings: &mut MapStrings, id: FourCC, field: &FieldMeta) -> bool {
        let Some(field_id) = FourCC::parse(&field.id) else {
            return false;
        };

        let mut removed = false;
        for data in [&mut self.tables.data, &mut self.tables.skin] {
            let entries = data.custom.iter_mut().chain(data.original.iter_mut());
            for entry in entries.filter(|e| e.effective_id() == id) {
                for set in &mut entry.sets {
                    let before = set.mods.len();
                    set.mods.retain(|m| m.field != field_id);
                    removed |= set.mods.len() != before;
                }
            }

            // A changed game object with no changes left is dropped entirely.
            data.original.retain(|e| e.mods().next().is_some());
        }

        self.drop_unused_strings(strings);
        removed
    }

    /// Removes strings this editor added that no field refers to anymore.
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

    /// Removes a custom object and the strings this editor added for it, as
    /// if it had never been created. Returns `false` for game objects (their
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

    /// Creates a custom object copying `base` and returns its new ID, the
    /// next free one of `I000`, `I001`, ... (by the kind's first letter).
    pub fn create(&mut self, db: &UnitDatabase, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!(
                "unknown base {} {base}",
                self.kind.singular
            )));
        }

        let id = next_free_id(self.kind.id_prefix, |id| {
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

    /// Updates the string a text field already refers to, or adds a new one
    /// with a comment like the World Editor's: `Items: I000 (name), Name (Name)`.
    fn store_string(
        &mut self,
        db: &UnitDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> String {
        let current = self.modification(id, field).map(Value::to_string);
        let existing = current.and_then(|c| Some((reference_id(&c)?, c)));

        if let Some((number, reference)) = existing
            && strings.set(number, text)
        {
            return reference;
        }

        let comment = format!(
            "{}: {id} ({}), {} ({})",
            self.kind.label,
            self.name(db, strings, id),
            db.field_name(field),
            field.field
        );
        let reference = strings.add(text, Some(&comment));
        if let Some(number) = reference_id(&reference) {
            self.added_strings.push(number);
        }

        reference
    }

    /// The files to write: those the map had, and new ones that now hold something.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        [
            (self.kind.data_file, &self.tables.data),
            (self.kind.skin_file, &self.tables.skin),
        ]
        .into_iter()
        .filter(|(file, data)| {
            !data.custom.is_empty() || !data.original.is_empty() || self.loaded.contains(file)
        })
        .map(|(file, data)| (file, data.to_bytes()))
        .collect()
    }
}

/// The object's entry in the gameplay or skin table, created when missing. A
/// custom object missing from this table gets a custom entry with its base.
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

/// Whether `value` equals the base object's game value in both art modes.
fn same_as_base(db: &UnitDatabase, base: FourCC, field: &FieldMeta, value: &Value) -> bool {
    let base = base.to_string();
    [ArtMode::Sd, ArtMode::Hd].iter().all(|mode| {
        let base_value = db.raw_value(&base, field, *mode).unwrap_or_default();
        parse_value(field, &base_value).is_ok_and(|parsed| &parsed == value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(code: &str) -> FourCC {
        FourCC::parse(code).unwrap_or(FourCC::ZERO)
    }

    #[test]
    fn removing_a_custom_buff_drops_it_and_its_new_strings() {
        let mut tables = Tables {
            data: ObjectData::empty(Layout::Plain),
            skin: ObjectData::empty(Layout::Plain),
        };
        let mut strings = MapStrings::default();
        let kept = strings.add("Kept", None);
        let added = strings.add("Fire Bolt", None);
        let added_id = reference_id(&added).unwrap_or_default();
        for (table, mods) in [
            (&mut tables.data, Vec::new()),
            (
                &mut tables.skin,
                vec![Modification {
                    field: id("ftip"),
                    level: None,
                    column: None,
                    value: Value::String(added),
                    end: FourCC::ZERO,
                }],
            ),
        ] {
            table.custom.push(ObjectEntry {
                base: id("BPSE"),
                id: id("B000"),
                sets: vec![ModSet { flags: 0, mods }],
            });
        }
        let mut buffs = PlainObjects::from_tables(BUFFS, tables);
        buffs.added_strings.push(added_id);

        assert!(
            !buffs.remove(&mut strings, id("BPSE")),
            "game buffs are not removed"
        );
        assert!(buffs.remove(&mut strings, id("B000")));
        assert!(buffs.objects().is_empty());
        assert_eq!(strings.get(added_id), None, "its new string is gone");
        let kept_id = reference_id(&kept).unwrap_or_default();
        assert_eq!(strings.get(kept_id), Some("Kept"), "other strings stay");
    }
}
