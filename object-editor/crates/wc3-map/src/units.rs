//! A map's unit changes on top of the game's unit data, and the map's string
//! table, which text fields of every object type refer to.
//!
//! Reforged stores a map's unit changes in two files: gameplay fields in
//! `war3map.w3u` and cosmetic ("netsafe") fields such as names and art in
//! `war3mapSkin.w3u`.

use wc3_data::game::ArtMode;
use wc3_data::meta::{FieldMeta, ObjectKind};
use wc3_data::objects::{FourCC, ObjectData, Value};
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;

use crate::plain::{PlainObjects, UNITS};
use crate::store::MapObject;
use crate::{Map, Result, STRINGS_FILE};

const NAME_FIELD: &str = "unam";

pub struct MapUnits {
    objects: PlainObjects,
    strings: MapStrings,
}

impl MapUnits {
    pub fn load(map: &Map) -> Result<Self> {
        Ok(Self {
            objects: PlainObjects::load(map, UNITS)?,
            strings: map.strings()?,
        })
    }

    pub fn empty() -> Self {
        Self {
            objects: PlainObjects::empty(UNITS),
            strings: MapStrings::default(),
        }
    }

    pub fn strings(&self) -> &MapStrings {
        &self.strings
    }

    pub fn strings_mut(&mut self) -> &mut MapStrings {
        &mut self.strings
    }

    pub(crate) fn parts(&self) -> (&ObjectData, &ObjectData, &MapStrings) {
        let tables = self.objects.tables();

        (&tables.data, &tables.skin, &self.strings)
    }

    #[cfg(test)]
    pub(crate) fn from_parts(data: ObjectData, skin: ObjectData, strings: MapStrings) -> Self {
        Self {
            objects: PlainObjects::from_tables(UNITS, crate::transfer::Tables { data, skin }),
            strings,
        }
    }

    pub(crate) fn parts_mut(&mut self) -> (&mut ObjectData, &mut ObjectData, &mut MapStrings) {
        let tables = self.objects.tables_mut();

        (&mut tables.data, &mut tables.skin, &mut self.strings)
    }

    pub fn units(&self) -> Vec<MapObject> {
        self.objects.objects()
    }

    pub fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        self.objects.base_of(db, id)
    }

    pub fn kind(&self, db: &UnitDatabase, id: FourCC) -> Option<ObjectKind> {
        let base = self.base_of(db, id)?;

        Some(db.kind(&base.to_string()))
    }

    pub fn fields<'a>(&self, db: &'a UnitDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        self.objects.fields(db, id)
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta) -> Option<&Value> {
        self.objects.modification(id, field)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        self.objects.is_modified(id, field)
    }

    pub fn raw_value(
        &self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        self.objects.raw_value(db, &self.strings, id, field, mode)
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

    pub fn set(
        &mut self,
        db: &UnitDatabase,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> Result<()> {
        self.objects.set(db, &mut self.strings, id, field, text)
    }

    pub fn reset(&mut self, id: FourCC, field: &FieldMeta) -> bool {
        self.objects.reset(&mut self.strings, id, field)
    }

    /// Creates a custom unit copying `base` and returns its new ID, numbered the
    /// way the World Editor does (`H000`, `H001`, ... for a hero based on `Hpal`).
    pub fn create(&mut self, db: &UnitDatabase, base: FourCC) -> Result<FourCC> {
        self.objects.create(db, base)
    }

    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut files = self.objects.files();

        files.push((STRINGS_FILE, self.strings.to_text().into_bytes()));
        files
    }
}

#[cfg(test)]
mod tests {
    use wc3_data::objects::{Layout, ModSet, Modification, ObjectEntry};

    use super::*;

    fn id(code: &str) -> FourCC {
        FourCC::parse(code).unwrap_or(FourCC([0; 4]))
    }

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
        let units = MapUnits::from_parts(data, skins, MapStrings::default());

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
