//! A map's changes to objects without levels, on top of the game's data (a
//! [`UnitDatabase`] loaded for that catalog): units, items, buffs and
//! destructables.
//!
//! Reforged splits them in two files: gameplay fields in `war3map.w3u` (units),
//! `.w3t` (items), `.w3h` (buffs) or `.w3b` (destructables), and cosmetic
//! ("netsafe") fields such as names, tooltips and art in the matching
//! `war3mapSkin` file.
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use wc3_data::game::ArtMode;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, Value};
use wc3_data::strings::MapStrings;
use wc3_data::units::UnitDatabase;

use crate::store::{MapObject, ObjectStore, Slot};
use crate::transfer::Tables;
use crate::{
    BUFF_SKINS_FILE, BUFFS_FILE, DESTRUCTABLE_SKINS_FILE, DESTRUCTABLES_FILE, Error,
    ITEM_SKINS_FILE, ITEMS_FILE, Map, Result, UNIT_SKINS_FILE, UNITS_FILE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlainKind {
    files: [&'static str; 2],
    /// Custom objects are numbered `I000`, `I001`, ... like the World Editor
    /// does; `None` takes the base's first letter (`H000` for a hero based on `Hpal`).
    id_prefix: Option<u8>,
    /// "Items", as string comments read: `Items: I000 (name), Name (Name)`.
    label: &'static str,
    singular: &'static str,
    name_field: &'static str,
}

pub(crate) const UNITS: PlainKind = PlainKind {
    files: [UNITS_FILE, UNIT_SKINS_FILE],
    id_prefix: None,
    label: "Units",
    singular: "unit",
    name_field: "unam",
};

pub const ITEMS: PlainKind = PlainKind {
    files: [ITEMS_FILE, ITEM_SKINS_FILE],
    id_prefix: Some(b'I'),
    label: "Items",
    singular: "item",
    name_field: "unam",
};

pub const BUFFS: PlainKind = PlainKind {
    files: [BUFFS_FILE, BUFF_SKINS_FILE],
    id_prefix: Some(b'B'),
    label: "Buffs",
    singular: "buff",
    name_field: "fnam",
};

pub const DESTRUCTABLES: PlainKind = PlainKind {
    files: [DESTRUCTABLES_FILE, DESTRUCTABLE_SKINS_FILE],
    id_prefix: Some(b'B'),
    label: "Destructables",
    singular: "destructable",
    name_field: "bnam",
};

pub struct PlainObjects {
    kind: PlainKind,
    store: ObjectStore,
}

impl PlainObjects {
    pub fn load(map: &Map, kind: PlainKind) -> Result<Self> {
        Ok(Self {
            kind,
            store: ObjectStore::load(map, kind.files, Layout::Plain)?,
        })
    }

    pub fn empty(kind: PlainKind) -> Self {
        Self {
            kind,
            store: ObjectStore::empty(kind.files, Layout::Plain),
        }
    }

    pub(crate) fn tables(&self) -> &Tables {
        self.store.tables()
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        self.store.tables_mut()
    }

    #[cfg(test)]
    pub(crate) fn from_tables(kind: PlainKind, tables: Tables) -> Self {
        Self {
            kind,
            store: ObjectStore::from_tables(kind.files, tables),
        }
    }

    pub fn objects(&self) -> Vec<MapObject> {
        self.store.objects()
    }

    pub fn has(&self, id: FourCC) -> bool {
        self.store.has(id)
    }

    pub fn base_of(&self, db: &UnitDatabase, id: FourCC) -> Option<FourCC> {
        self.store.find(id).base(|id| db.contains(id))
    }

    pub fn fields<'a>(&self, db: &'a UnitDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        self.base_of(db, id)
            .map(|base| db.fields_for(&base.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta) -> Option<&Value> {
        let slot = Slot::plain(field).ok()?;

        self.store.find(id).modification(&slot)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta) -> bool {
        self.modification(id, field).is_some()
    }

    pub fn raw_value(
        &self,
        db: &UnitDatabase,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        mode: ArtMode,
    ) -> Option<String> {
        let found = self.store.find(id);

        if let Ok(slot) = Slot::plain(field)
            && let Some(text) = found.text(&slot, strings)
        {
            return Some(text);
        }

        let base = found.base(|id| db.contains(id))?;

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

    pub fn set(
        &mut self,
        db: &UnitDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        text: &str,
    ) -> Result<()> {
        let slot = Slot::plain(field)?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!(
                "unknown {} {id}",
                self.kind.singular
            )));
        };
        let comment = format!(
            "{}: {id} ({}), {} ({})",
            self.kind.label,
            self.name(db, strings, id),
            db.field_name(field),
            field.field
        );
        let base = base.to_string();

        self.store.set(strings, id, &slot, text, &comment, |mode| {
            db.raw_value(&base, field, mode)
        })
    }

    pub fn reset(&mut self, strings: &mut MapStrings, id: FourCC, field: &FieldMeta) -> bool {
        Slot::plain(field).is_ok_and(|slot| self.store.reset(strings, id, &slot))
    }

    pub fn remove(&mut self, strings: &mut MapStrings, id: FourCC) -> bool {
        self.store.remove(strings, id)
    }

    pub fn create(&mut self, db: &UnitDatabase, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!(
                "unknown base {} {base}",
                self.kind.singular
            )));
        }

        let prefix = self.kind.id_prefix.unwrap_or(base.0[0]);

        self.store
            .create(base, prefix, |id| db.contains(id))
            .ok_or_else(|| Error::Invalid(format!("no free ID for {base}")))
    }

    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        self.store.files()
    }
}
