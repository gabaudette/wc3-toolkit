//! A map's unit changes on top of the game's unit data.
//!
//! Reforged stores a map's unit changes in two files: gameplay fields in
//! `war3map.w3u` and cosmetic ("netsafe") fields such as names and art in
//! `war3mapSkin.w3u`. Text fields refer to the map's string table.

use std::collections::HashMap;

use wc3_data::game::ArtMode;
use wc3_data::meta::{FieldMeta, ObjectKind};
use wc3_data::objects::{FourCC, Layout, ModSet, Modification, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};
use wc3_data::units::UnitDatabase;

use crate::values::{is_translatable, parse_value};
use crate::{Error, Map, Result, STRINGS_FILE, UNIT_SKINS_FILE, UNITS_FILE};

/// Characters the World Editor uses for the last three letters of new IDs (`H000`, `H00A`, ...).
const ID_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ID_SUFFIX_LEN: u32 = 3;
const NAME_FIELD: &str = "unam";

pub struct MapUnits {
    data: ObjectData,
    skins: ObjectData,
    strings: MapStrings,
    /// Strings this editor added since loading. Only these are removed again
    /// when nothing refers to them; strings already in the map are never touched.
    added_strings: Vec<u32>,
}

/// A unit the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct MapUnit {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

impl MapUnits {
    pub fn load(map: &Map) -> Result<Self> {
        let empty = || ObjectData::empty(Layout::Plain);

        Ok(Self {
            data: map.object_data(UNITS_FILE)?.unwrap_or_else(empty),
            skins: map.object_data(UNIT_SKINS_FILE)?.unwrap_or_else(empty),
            strings: map.strings()?,
            added_strings: Vec::new(),
        })
    }

    /// The map's string table. Units keep it; abilities borrow it to store
    /// their text.
    pub fn strings(&self) -> &MapStrings {
        &self.strings
    }

    pub fn strings_mut(&mut self) -> &mut MapStrings {
        &mut self.strings
    }

    /// The unit tables (gameplay, skin) and the string table, for moving
    /// objects between maps.
    pub(crate) fn parts(&self) -> (&ObjectData, &ObjectData, &MapStrings) {
        (&self.data, &self.skins, &self.strings)
    }

    #[cfg(test)]
    pub(crate) fn from_parts(data: ObjectData, skins: ObjectData, strings: MapStrings) -> Self {
        Self {
            data,
            skins,
            strings,
            added_strings: Vec::new(),
        }
    }

    pub(crate) fn parts_mut(&mut self) -> (&mut ObjectData, &mut ObjectData, &mut MapStrings) {
        (&mut self.data, &mut self.skins, &mut self.strings)
    }

    /// Custom units first, then changed original units, each listed once.
    pub fn units(&self) -> Vec<MapUnit> {
        let mut units: Vec<MapUnit> = Vec::new();
        let mut index: HashMap<FourCC, usize> = HashMap::new();
        let tables = [
            (&self.data.custom, true),
            (&self.skins.custom, true),
            (&self.data.original, false),
            (&self.skins.original, false),
        ];

        for (entries, is_custom) in tables {
            for entry in entries {
                let id = entry.effective_id();
                let mods = entry.mods().count();

                match index.get(&id) {
                    Some(&i) => units[i].modified_fields += mods,
                    None => {
                        index.insert(id, units.len());
                        units.push(MapUnit {
                            id,
                            base: entry.base,
                            is_custom,
                            modified_fields: mods,
                        });
                    }
                }
            }
        }

        units
    }

    /// The game unit a map unit is based on: the base of a custom unit, or the
    /// unit itself. `None` when neither the map nor the game has the ID.
    pub fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        if let Some(entry) = self.data.find(id).or_else(|| self.skins.find(id)) {
            return Some(entry.base);
        }

        db.contains(&id.to_string()).then_some(id)
    }

    pub fn kind(&self, db: &UnitDatabase, id: FourCC) -> Option<ObjectKind> {
        let base = self.base_of(db, id)?;
        Some(db.kind(&base.to_string()))
    }

    /// Fields the Object Editor shows for this unit (those of its base unit).
    pub fn fields<'a>(&self, db: &'a UnitDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        self.base_of(db, id)
            .map(|base| db.fields_for(&base.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta) -> Option<&Value> {
        let field_id = FourCC::parse(&field.id)?;
        let file = self.file_for(field);
        let other = if std::ptr::eq(file, &self.data) {
            &self.skins
        } else {
            &self.data
        };

        // Look in the file the field belongs to first; maps saved by older
        // editors keep every field in war3map.w3u.
        [file, other].into_iter().find_map(|data| {
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
    /// references resolved), otherwise the base unit's game data.
    pub fn raw_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        if let Some(value) = self.modification(id, field) {
            return Some(match value {
                Value::String(s) => self.strings.resolve(s).to_owned(),
                other => other.to_string(),
            });
        }

        let base = self.base_of(db, id)?;
        db.raw_value(&base.to_string(), field, mode)
    }

    pub fn display_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> String {
        db.format_value(field, self.raw_value(db, id, field, mode))
    }

    pub fn name(&self, db: &UnitDatabase, id: FourCC) -> String {
        let name = db
            .field(NAME_FIELD)
            .and_then(|field| self.modification(id, field))
            .map(|value| self.strings.resolve(&value.to_string()).to_owned());

        name.or_else(|| self.base_of(db, id).map(|base| db.name(&base.to_string())))
            .unwrap_or_else(|| id.to_string())
    }

    /// Sets a field from text as typed in an editor: numbers for numeric
    /// fields, `true`/`false` or `1`/`0` for booleans, raw codes for lists and
    /// choices. Text fields are stored in the map's string table.
    pub fn set(
        &mut self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> Result<()> {
        let field_id = FourCC::parse(&field.id)
            .ok_or_else(|| Error::Invalid(format!("bad field ID {}", field.id)))?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!("unknown unit {id}")));
        };

        // Setting the base unit's own value is no change at all: drop the
        // modification instead of storing a copy.
        let parsed = parse_value(field, text)?;
        if same_as_base(db, base, field, &parsed) {
            self.reset(id, field);
            return Ok(());
        }

        let value = match parsed {
            Value::String(s) if is_translatable(field) => {
                Value::String(self.store_string(db, id, field, &s))
            }
            other => other,
        };

        let in_skin = field.netsafe;
        let data = if in_skin {
            &mut self.skins
        } else {
            &mut self.data
        };
        let entry = entry_mut(data, id);
        let set = match entry.sets.first_mut() {
            Some(set) => set,
            None => {
                entry.sets.push(ModSet {
                    flags: 0,
                    mods: Vec::new(),
                });
                &mut entry.sets[0]
            }
        };

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

    /// Removes the map's change to a field so the base unit's value applies
    /// again. Returns `false` when the field was not changed.
    pub fn reset(&mut self, id: FourCC, field: &FieldMeta) -> bool {
        let Some(field_id) = FourCC::parse(&field.id) else {
            return false;
        };

        let mut removed = false;
        for data in [&mut self.data, &mut self.skins] {
            let entries = data.custom.iter_mut().chain(data.original.iter_mut());
            for entry in entries.filter(|e| e.effective_id() == id) {
                for set in &mut entry.sets {
                    let before = set.mods.len();
                    set.mods.retain(|m| m.field != field_id);
                    removed |= set.mods.len() != before;
                }
            }

            // A changed original unit with no changes left is dropped entirely.
            data.original.retain(|e| e.mods().next().is_some());
        }

        self.drop_unused_strings();
        removed
    }

    /// Removes strings this editor added that no field refers to anymore.
    fn drop_unused_strings(&mut self) {
        let referenced: Vec<u32> = [&self.data, &self.skins]
            .iter()
            .flat_map(|data| data.custom.iter().chain(&data.original))
            .flat_map(ObjectEntry::mods)
            .filter_map(|m| match &m.value {
                Value::String(s) => reference_id(s),
                _ => None,
            })
            .collect();

        let strings = &mut self.strings;
        self.added_strings.retain(|id| {
            if referenced.contains(id) {
                return true;
            }

            strings.remove(*id);
            false
        });
    }

    /// Creates a custom unit copying `base` and returns its new ID, numbered the
    /// way the World Editor does (`H000`, `H001`, ... for a hero based on `Hpal`).
    pub fn create(&mut self, db: &UnitDatabase, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!("unknown base unit {base}")));
        }

        let id = self
            .next_id(db, base)
            .ok_or_else(|| Error::Invalid(format!("no free ID for {base}")))?;
        for data in [&mut self.data, &mut self.skins] {
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

    fn next_id(&self, db: &UnitDatabase, base: FourCC) -> Option<FourCC> {
        let prefix = base.0[0];
        let radix = ID_ALPHABET.len();
        let count = radix.pow(ID_SUFFIX_LEN);

        (0..count)
            .map(|n| {
                FourCC([
                    prefix,
                    ID_ALPHABET[n / (radix * radix)],
                    ID_ALPHABET[(n / radix) % radix],
                    ID_ALPHABET[n % radix],
                ])
            })
            .find(|id| {
                self.data.find(*id).is_none()
                    && self.skins.find(*id).is_none()
                    && !db.contains(&id.to_string())
            })
    }

    /// Updates the string a text field already refers to, or adds a new one
    /// with a comment like the World Editor's: `Units: H000 (name), Name (Name)`.
    fn store_string(
        &mut self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> String {
        let current = self.modification(id, field).map(Value::to_string);
        let existing = current.and_then(|c| Some((reference_id(&c)?, c)));

        if let Some((number, reference)) = existing
            && self.strings.set(number, text)
        {
            return reference;
        }

        let comment = format!(
            "Units: {id} ({}), {} ({})",
            self.name(db, id),
            db.field_name(field),
            field.field
        );
        let reference = self.strings.add(text, Some(&comment));
        if let Some(id) = reference_id(&reference) {
            self.added_strings.push(id);
        }

        reference
    }

    /// The files to write into the map for the current state.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (UNITS_FILE, self.data.to_bytes()),
            (UNIT_SKINS_FILE, self.skins.to_bytes()),
            (STRINGS_FILE, self.strings.to_text().into_bytes()),
        ]
    }

    fn file_for(&self, field: &FieldMeta) -> &ObjectData {
        if field.netsafe {
            &self.skins
        } else {
            &self.data
        }
    }
}

fn entry_mut(data: &mut ObjectData, id: FourCC) -> &mut ObjectEntry {
    let index = data.custom.iter().position(|e| e.effective_id() == id);
    if let Some(i) = index {
        return &mut data.custom[i];
    }

    let index = data.original.iter().position(|e| e.effective_id() == id);
    match index {
        Some(i) => &mut data.original[i],
        None => {
            // A change to an original unit, or a custom unit missing from this file.
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

/// Whether `value` equals the base unit's game value in both art modes (some
/// art fields differ between SD and HD; those always keep the change).
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
        FourCC::parse(code).unwrap_or(FourCC([0; 4]))
    }

    /// An entry with one modification per given field.
    fn entry(base: &str, own_id: &str, fields: &[&str]) -> ObjectEntry {
        let mods = fields
            .iter()
            .map(|field| Modification {
                field: id(field),
                level: None,
                column: None,
                value: Value::Int(1),
                end: FourCC([0; 4]),
            })
            .collect();

        ObjectEntry {
            base: id(base),
            id: if own_id.is_empty() {
                FourCC([0; 4])
            } else {
                id(own_id)
            },
            sets: vec![ModSet { flags: 0, mods }],
        }
    }

    #[test]
    fn units_lists_custom_then_original_once_with_changes_from_both_files() {
        let mut data = ObjectData::empty(Layout::Plain);
        let mut skins = ObjectData::empty(Layout::Plain);
        data.custom = vec![
            entry("Hpal", "H000", &["uhpm"]),
            entry("hfoo", "h001", &["uhpm", "udef"]),
        ];
        skins.custom = vec![
            entry("hfoo", "h001", &["unam"]),
            entry("ogru", "o002", &["unam"]),
        ];
        data.original = vec![entry("hpea", "", &["ubld"])];
        skins.original = vec![
            entry("hpea", "", &["unam", "uico"]),
            entry("Hamg", "", &["unam"]),
        ];
        let units = MapUnits {
            data,
            skins,
            strings: MapStrings::default(),
            added_strings: Vec::new(),
        };

        let listed: Vec<(String, bool, usize)> = units
            .units()
            .iter()
            .map(|u| (u.id.to_string(), u.is_custom, u.modified_fields))
            .collect();
        let expected = [
            ("H000", true, 1),
            ("h001", true, 3),
            ("o002", true, 1),
            ("hpea", false, 3),
            ("Hamg", false, 1),
        ]
        .map(|(code, custom, changed)| (code.to_owned(), custom, changed));
        assert_eq!(listed, expected);
    }
}
