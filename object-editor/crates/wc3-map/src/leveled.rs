//! A map's changes to objects whose fields can have a value per level, on top
//! of the game's data: upgrades (per research level) and doodads (per model
//! variation). Abilities, whose data columns also matter, have their own code.
//!
//! Gameplay fields go in `war3map.w3q` (upgrades) or `.w3d` (doodads) and
//! cosmetic ("netsafe") fields such as names, tooltips and icons in the
//! matching `war3mapSkin` file. Every change stores a level: 1, 2, ... for
//! fields with a value per level, 0 for the others. The data column is always 0.
//! Doodads store their variation as the level, the first variation being 1
//! (assumed like ability levels; no map seen so far changes a variation).
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use std::collections::HashMap;

use wc3_data::game::ArtMode;
use wc3_data::leveled::LeveledData;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, ModSet, Modification, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};

use crate::transfer::{Tables, next_free_id};
use crate::values::{is_translatable, parse_value};
use crate::{
    DOODAD_SKINS_FILE, DOODADS_FILE, Error, Map, Result, UPGRADE_SKINS_FILE, UPGRADES_FILE,
};

/// One kind of object with levels: its files, how custom IDs start, how the
/// World Editor names it in string comments, and the fields that describe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeveledKind {
    data_file: &'static str,
    skin_file: &'static str,
    /// Custom objects are numbered `R000`, `R001`, ... like the World Editor does.
    id_prefix: u8,
    /// "Upgrades", as string comments read.
    label: &'static str,
    /// "upgrade", for messages.
    singular: &'static str,
    name_field: &'static str,
    icon_field: Option<&'static str>,
    /// The field list rows are filtered by: race, category.
    filter_field: &'static str,
    /// The field holding the number of levels or variations.
    levels_field: &'static str,
}

pub const UPGRADES: LeveledKind = LeveledKind {
    data_file: UPGRADES_FILE,
    skin_file: UPGRADE_SKINS_FILE,
    id_prefix: b'R',
    label: "Upgrades",
    singular: "upgrade",
    name_field: "gnam",
    icon_field: Some("gar1"),
    filter_field: "grac",
    levels_field: "glvl",
};

pub const DOODADS: LeveledKind = LeveledKind {
    data_file: DOODADS_FILE,
    skin_file: DOODAD_SKINS_FILE,
    id_prefix: b'D',
    label: "Doodads",
    singular: "doodad",
    name_field: "dnam",
    icon_field: None,
    filter_field: "dcat",
    levels_field: "dvar",
};

pub struct LeveledObjects {
    kind: LeveledKind,
    tables: Tables,
    /// Files the map had when loaded; others are only written once
    /// they hold something, so untouched maps keep their files as they are.
    loaded: Vec<&'static str>,
    /// Strings this editor added since loading. Only these are removed again
    /// when nothing refers to them; strings already in the map are never touched.
    added_strings: Vec<u32>,
}

/// An object the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct LeveledObject {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

/// What a list entry shows, with the map's changes applied.
#[derive(Debug, Clone, PartialEq)]
pub struct LeveledInfo {
    pub name: String,
    pub icon: Option<String>,
    /// The raw value list rows are filtered by (race, category).
    pub filter: String,
    pub levels: u32,
}

impl LeveledObjects {
    pub fn load(map: &Map, kind: LeveledKind) -> Result<Self> {
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
    pub fn empty(kind: LeveledKind) -> Self {
        Self {
            kind,
            tables: Tables {
                data: ObjectData::empty(Layout::Leveled),
                skin: ObjectData::empty(Layout::Leveled),
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
    pub(crate) fn from_tables(kind: LeveledKind, tables: Tables) -> Self {
        Self {
            kind,
            tables,
            loaded: Vec::new(),
            added_strings: Vec::new(),
        }
    }

    /// Custom objects first, then changed game objects, each listed once.
    pub fn objects(&self) -> Vec<LeveledObject> {
        let mut objects: Vec<LeveledObject> = Vec::new();
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
                        objects.push(LeveledObject {
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

    /// The object's entries in both tables, found once for several lookups.
    fn find(&self, id: FourCC) -> Found<'_> {
        Found {
            id,
            data: self.tables.data.find(id),
            skin: self.tables.skin.find(id),
        }
    }

    /// Whether the map adds or changes this object.
    pub fn has(&self, id: FourCC) -> bool {
        self.find(id).in_map()
    }

    /// The game object a map object is based on: the base of a custom one, or
    /// the object itself. `None` when neither the map nor the
    /// game has the ID.
    pub fn base_of(&self, db: &dyn LeveledData, id: FourCC) -> Option<FourCC> {
        self.find(id).base(db)
    }

    /// The number of levels, at least 1.
    pub fn levels(&self, db: &dyn LeveledData, id: FourCC) -> u32 {
        self.find(id).levels(db, self.kind.levels_field)
    }

    /// The map's change to a field at `level` (ignored for fields without levels).
    pub fn modification(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<&Value> {
        self.find(id).modification(field, level)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.modification(id, field, level).is_some()
    }

    /// The field's raw value at `level`: the map's change if there is one
    /// (with string references resolved), otherwise the base object's game data.
    pub fn raw_value(
        &self,
        db: &dyn LeveledData,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        self.find(id).raw_value(db, strings, field, level, mode)
    }

    /// The name (the level 1 name where names have levels).
    pub fn name(&self, db: &dyn LeveledData, strings: &MapStrings, id: FourCC) -> String {
        db.field(self.kind.name_field)
            .and_then(|field| {
                self.raw_value(db, strings, id, field, first_level(field), ArtMode::Sd)
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| id.to_string())
    }

    /// What the list shows, with the map's changes applied. `None` when
    /// neither the map nor the game has the ID.
    pub fn info(
        &self,
        db: &dyn LeveledData,
        strings: &MapStrings,
        id: FourCC,
        mode: ArtMode,
    ) -> Option<LeveledInfo> {
        let found = self.find(id);
        found.base(db)?;
        let text = |field_id: &str| {
            db.field(field_id)
                .and_then(|field| found.raw_value(db, strings, field, first_level(field), mode))
                .filter(|v| !v.is_empty())
        };

        Some(LeveledInfo {
            name: text(self.kind.name_field).unwrap_or_else(|| id.to_string()),
            icon: self.kind.icon_field.and_then(text),
            filter: text(self.kind.filter_field).unwrap_or_default(),
            levels: found.levels(db, self.kind.levels_field),
        })
    }

    /// Sets a field at `level` (ignored for fields without levels) from text
    /// as typed in an editor. Text fields are stored in the map's string table.
    /// Setting the base object's own value removes the change instead.
    pub fn set(
        &mut self,
        db: &dyn LeveledData,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
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

        let stored_level = if field.per_level { level } else { 0 };
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
                column: Some(0),
                value,
                end: FourCC::ZERO,
            }),
        }

        Ok(())
    }

    /// Removes the map's change to a field at `level` (every change, for
    /// fields without levels) so the base object's value applies again.
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

    /// Creates a custom object copying `base` and returns its new ID, the
    /// next free one of `R000`, `R001`, ... (by the kind's first letter).
    pub fn create(&mut self, db: &dyn LeveledData, base: FourCC) -> Result<FourCC> {
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
    /// with a comment like the World Editor's: `Upgrades: R000 (name), Name (Name), level 1`.
    fn store_string(
        &mut self,
        db: &dyn LeveledData,
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
            "{}: {id} ({}), {} ({})",
            self.kind.label,
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

    /// The files to write: those the map had, and new ones that now
    /// hold something.
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

/// An object's entries in the gameplay and skin tables.
struct Found<'a> {
    id: FourCC,
    data: Option<&'a ObjectEntry>,
    skin: Option<&'a ObjectEntry>,
}

impl<'a> Found<'a> {
    fn in_map(&self) -> bool {
        self.data.is_some() || self.skin.is_some()
    }

    fn base(&self, db: &dyn LeveledData) -> Option<FourCC> {
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
        // editors keep every field in the gameplay file.
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
        db: &dyn LeveledData,
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

    fn levels(&self, db: &dyn LeveledData, levels_field: &str) -> u32 {
        let Some(field) = db.field(levels_field) else {
            return 1;
        };
        let changed = self.modification(field, 0).map(Value::to_string);
        changed
            .or_else(|| db.raw_value(&self.base(db)?.to_string(), field, 0, ArtMode::Sd))
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or_default()
            .max(1)
    }
}

/// The level a field is read at to describe an object: 1 where it has levels.
fn first_level(field: &FieldMeta) -> u32 {
    u32::from(field.per_level)
}

/// Whether a stored change is the one for `level`. Fields without levels
/// match any level: maps store them at level 0, but not every editor did.
fn at_level(field: &FieldMeta, m: &Modification, level: u32) -> bool {
    !field.per_level || m.level == Some(level)
}

/// The object's entry in the gameplay or skin table, created when missing.
/// A custom object missing from this table gets a custom entry with its base.
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

/// Whether `value` equals the base object's game value at `level` in both
/// art modes (some art differs between SD and HD; those always keep the change).
fn same_as_base(
    db: &dyn LeveledData,
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
