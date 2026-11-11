//! A map's changes to objects whose fields can have a value per level, on top
//! of the game's data: abilities and upgrades (per level) and doodads (per
//! model variation).
//!
//! Gameplay fields go in `war3map.w3a` (abilities), `.w3q` (upgrades) or
//! `.w3d` (doodads) and cosmetic ("netsafe") fields such as names, tooltips and
//! icons in the matching `war3mapSkin` file. Every change stores a level: 1, 2,
//! ... for fields with a value per level, 0 for the others, and a data column:
//! for abilities' per-level data fields `DataA` = 1, 0 for every other field.
//! Doodads store their variation as the level, the first variation being 1
//! (assumed like ability levels; no map seen so far changes a variation).
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use wc3_data::game::ArtMode;
use wc3_data::leveled::LeveledData;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, Value};
use wc3_data::strings::MapStrings;

use crate::store::{Found, MapObject, ObjectStore, Slot};
use crate::transfer::Tables;
use crate::{
    ABILITIES_FILE, ABILITY_SKINS_FILE, DOODAD_SKINS_FILE, DOODADS_FILE, Error, Map, Result,
    UPGRADE_SKINS_FILE, UPGRADES_FILE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeveledKind {
    files: [&'static str; 2],
    /// Custom objects are numbered `R000`, `R001`, ... like the World Editor does.
    id_prefix: u8,
    /// "Upgrades", as string comments read.
    label: &'static str,
    singular: &'static str,
    name_field: &'static str,
    icon_field: Option<&'static str>,
    filter_field: &'static str,
    levels_field: &'static str,
    /// Whether per-level fields store their data column.
    data_columns: bool,
}

pub(crate) const ABILITIES: LeveledKind = LeveledKind {
    files: [ABILITIES_FILE, ABILITY_SKINS_FILE],
    id_prefix: b'A',
    label: "Abilities",
    singular: "ability",
    name_field: "anam",
    icon_field: Some("aart"),
    filter_field: "arac",
    levels_field: "alev",
    data_columns: true,
};

pub const UPGRADES: LeveledKind = LeveledKind {
    files: [UPGRADES_FILE, UPGRADE_SKINS_FILE],
    id_prefix: b'R',
    label: "Upgrades",
    singular: "upgrade",
    name_field: "gnam",
    icon_field: Some("gar1"),
    filter_field: "grac",
    levels_field: "glvl",
    data_columns: false,
};

pub const DOODADS: LeveledKind = LeveledKind {
    files: [DOODADS_FILE, DOODAD_SKINS_FILE],
    id_prefix: b'D',
    label: "Doodads",
    singular: "doodad",
    name_field: "dnam",
    icon_field: None,
    filter_field: "dcat",
    levels_field: "dvar",
    data_columns: false,
};

pub struct LeveledObjects {
    kind: LeveledKind,
    store: ObjectStore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeveledInfo {
    pub name: String,
    pub icon: Option<String>,
    pub filter: String,
    pub levels: u32,
}

impl LeveledObjects {
    pub fn load(map: &Map, kind: LeveledKind) -> Result<Self> {
        Ok(Self {
            kind,
            store: ObjectStore::load(map, kind.files, Layout::Leveled)?,
        })
    }

    pub fn empty(kind: LeveledKind) -> Self {
        Self {
            kind,
            store: ObjectStore::empty(kind.files, Layout::Leveled),
        }
    }

    pub(crate) fn tables(&self) -> &Tables {
        self.store.tables()
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        self.store.tables_mut()
    }

    #[cfg(test)]
    pub(crate) fn from_tables(kind: LeveledKind, tables: Tables) -> Self {
        Self {
            kind,
            store: ObjectStore::from_tables(kind.files, tables),
        }
    }

    pub fn objects(&self) -> Vec<MapObject> {
        self.store.objects()
    }

    pub(crate) fn lookup<'a>(&'a self, db: &'a dyn LeveledData, id: FourCC) -> Lookup<'a> {
        Lookup {
            found: self.store.find(id),
            db,
        }
    }

    pub fn has(&self, id: FourCC) -> bool {
        self.store.has(id)
    }

    pub fn base_of(&self, db: &dyn LeveledData, id: FourCC) -> Option<FourCC> {
        self.lookup(db, id).base()
    }

    pub fn levels(&self, db: &dyn LeveledData, id: FourCC) -> u32 {
        self.lookup(db, id).levels(self.kind.levels_field)
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<&Value> {
        let slot = Slot::leveled(field, level).ok()?;

        self.store.find(id).modification(&slot)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.modification(id, field, level).is_some()
    }

    pub fn raw_value(
        &self,
        db: &dyn LeveledData,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        self.lookup(db, id).raw_value(strings, field, level, mode)
    }

    pub fn name(&self, db: &dyn LeveledData, strings: &MapStrings, id: FourCC) -> String {
        db.field(self.kind.name_field)
            .and_then(|field| {
                self.raw_value(db, strings, id, field, first_level(field), ArtMode::Sd)
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| id.to_string())
    }

    pub fn info(
        &self,
        db: &dyn LeveledData,
        strings: &MapStrings,
        id: FourCC,
        mode: ArtMode,
    ) -> Option<LeveledInfo> {
        let lookup = self.lookup(db, id);

        lookup.base()?;
        let text = |field_id: &str| {
            db.field(field_id)
                .and_then(|field| lookup.raw_value(strings, field, first_level(field), mode))
                .filter(|v| !v.is_empty())
        };

        Some(LeveledInfo {
            name: text(self.kind.name_field).unwrap_or_else(|| id.to_string()),
            icon: self.kind.icon_field.and_then(text),
            filter: text(self.kind.filter_field).unwrap_or_default(),
            levels: lookup.levels(self.kind.levels_field),
        })
    }

    pub fn set(
        &mut self,
        db: &dyn LeveledData,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        text: &str,
    ) -> Result<()> {
        let mut slot = Slot::leveled(field, level)?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!(
                "unknown {} {id}",
                self.kind.singular
            )));
        };

        if field.per_level && level == 0 {
            return Err(Error::Invalid(format!("{} needs a level", field.id)));
        }

        if self.kind.data_columns {
            slot = slot.in_column(field.data_column.unwrap_or_default());
        }

        let mut comment = format!(
            "{}: {id} ({}), {} ({})",
            self.kind.label,
            self.name(db, strings, id),
            db.field_name(field),
            field.field
        );

        if field.per_level {
            comment.push_str(&format!(", level {level}"));
        }

        let base = base.to_string();

        self.store.set(strings, id, &slot, text, &comment, |mode| {
            db.raw_value(&base, field, level, mode)
        })
    }

    pub fn reset(
        &mut self,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
    ) -> bool {
        Slot::leveled(field, level).is_ok_and(|slot| self.store.reset(strings, id, &slot))
    }

    pub fn remove(&mut self, strings: &mut MapStrings, id: FourCC) -> bool {
        self.store.remove(strings, id)
    }

    pub fn create(&mut self, db: &dyn LeveledData, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!(
                "unknown base {} {base}",
                self.kind.singular
            )));
        }

        self.store
            .create(base, self.kind.id_prefix, |id| db.contains(id))
            .ok_or_else(|| Error::Invalid(format!("no free ID for {base}")))
    }

    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        self.store.files()
    }
}

/// An object's changes in the map and the game data under them.
pub(crate) struct Lookup<'a> {
    found: Found<'a>,
    db: &'a dyn LeveledData,
}

impl Lookup<'_> {
    pub fn base(&self) -> Option<FourCC> {
        self.found.base(|id| self.db.contains(id))
    }

    pub fn raw_value(
        &self,
        strings: &MapStrings,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        if let Ok(slot) = Slot::leveled(field, level)
            && let Some(text) = self.found.text(&slot, strings)
        {
            return Some(text);
        }

        self.db
            .raw_value(&self.base()?.to_string(), field, level, mode)
    }

    /// The value of a field without levels, as stored: text stays a string
    /// reference.
    pub fn plain_value(&self, field_id: &str) -> Option<String> {
        let field = self.db.field(field_id)?;
        let changed = Slot::leveled(field, 0)
            .ok()
            .and_then(|slot| self.found.modification(&slot));

        if let Some(value) = changed {
            return Some(value.to_string());
        }

        self.db
            .raw_value(&self.base()?.to_string(), field, 0, ArtMode::Sd)
    }

    pub fn levels(&self, levels_field: &str) -> u32 {
        self.plain_value(levels_field)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or_default()
            .max(1)
    }
}

fn first_level(field: &FieldMeta) -> u32 {
    u32::from(field.per_level)
}
