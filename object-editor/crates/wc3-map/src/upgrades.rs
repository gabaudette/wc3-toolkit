//! A map's upgrade changes on top of the game's upgrade data.
//!
//! Gameplay fields go in `war3map.w3q` and cosmetic ("netsafe") fields such as
//! names, tooltips and icons in `war3mapSkin.w3q`. Every change stores a level:
//! 1, 2, ... for fields with a value per level, 0 for the others. The data
//! column is always 0.
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
use wc3_data::upgrades::UpgradeDatabase;

use crate::transfer::{Tables, next_free_id};
use crate::values::{is_translatable, parse_value};
use crate::{Error, Map, Result, UPGRADE_SKINS_FILE, UPGRADES_FILE};

/// Custom upgrades are numbered `R000`, `R001`, ... like the World Editor does.
const ID_PREFIX: u8 = b'R';

/// Fields the editor reads to describe an upgrade.
const NAME: &str = "gnam";
const ICON: &str = "gar1";
const RACE: &str = "grac";
const LEVELS: &str = "glvl";

pub struct MapUpgrades {
    tables: Tables,
    /// Upgrade files the map had when loaded; others are only written once
    /// they hold something, so untouched maps keep their files as they are.
    loaded: Vec<&'static str>,
    /// Strings this editor added since loading. Only these are removed again
    /// when nothing refers to them; strings already in the map are never touched.
    added_strings: Vec<u32>,
}

/// An upgrade the map adds or changes.
#[derive(Debug, Clone, PartialEq)]
pub struct MapUpgrade {
    pub id: FourCC,
    pub base: FourCC,
    pub is_custom: bool,
    pub modified_fields: usize,
}

/// What a list entry shows, with the map's changes applied.
#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeInfo {
    pub name: String,
    pub icon: Option<String>,
    pub race: String,
    pub levels: u32,
}

impl MapUpgrades {
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
            data: read(UPGRADES_FILE)?,
            skin: read(UPGRADE_SKINS_FILE)?,
        };
        Ok(Self {
            tables,
            loaded,
            added_strings: Vec::new(),
        })
    }

    /// No changes: the game's upgrades as they are.
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

    /// The upgrade tables (gameplay, skin), for moving objects between maps.
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

    /// Custom upgrades first, then changed game upgrades, each listed once.
    pub fn upgrades(&self) -> Vec<MapUpgrade> {
        let mut upgrades: Vec<MapUpgrade> = Vec::new();
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
                    Some(&i) => upgrades[i].modified_fields += mods,
                    None => {
                        index.insert(id, upgrades.len());
                        upgrades.push(MapUpgrade {
                            id,
                            base: entry.base,
                            is_custom,
                            modified_fields: mods,
                        });
                    }
                }
            }
        }

        upgrades
    }

    /// The upgrade's entries in both tables, found once for several lookups.
    fn find(&self, id: FourCC) -> Found<'_> {
        Found {
            id,
            data: self.tables.data.find(id),
            skin: self.tables.skin.find(id),
        }
    }

    /// Whether the map adds or changes this upgrade.
    pub fn has(&self, id: FourCC) -> bool {
        self.find(id).in_map()
    }

    /// The game upgrade a map upgrade is based on: the base of a custom
    /// upgrade, or the upgrade itself. `None` when neither the map nor the
    /// game has the ID.
    pub fn base_of(&self, db: &UpgradeDatabase, id: FourCC) -> Option<FourCC> {
        self.find(id).base(db)
    }

    /// The number of levels, at least 1.
    pub fn levels(&self, db: &UpgradeDatabase, id: FourCC) -> u32 {
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
    /// (with string references resolved), otherwise the base upgrade's game data.
    pub fn raw_value(
        &self,
        db: &UpgradeDatabase,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        self.find(id).raw_value(db, strings, field, level, mode)
    }

    /// The level 1 name.
    pub fn name(&self, db: &UpgradeDatabase, strings: &MapStrings, id: FourCC) -> String {
        db.field(NAME)
            .and_then(|field| self.raw_value(db, strings, id, field, 1, ArtMode::Sd))
            .unwrap_or_else(|| id.to_string())
    }

    /// What the list shows, with the map's changes applied. `None` when
    /// neither the map nor the game has the ID.
    pub fn info(
        &self,
        db: &UpgradeDatabase,
        strings: &MapStrings,
        id: FourCC,
        mode: ArtMode,
    ) -> Option<UpgradeInfo> {
        let found = self.find(id);
        found.base(db)?;
        let text = |field_id: &str, level: u32| {
            db.field(field_id)
                .and_then(|field| found.raw_value(db, strings, field, level, mode))
                .filter(|v| !v.is_empty())
        };

        Some(UpgradeInfo {
            name: text(NAME, 1).unwrap_or_else(|| id.to_string()),
            icon: text(ICON, 1),
            race: text(RACE, 0).unwrap_or_default(),
            levels: found.levels(db),
        })
    }

    /// Sets a field at `level` (ignored for fields without levels) from text
    /// as typed in an editor. Text fields are stored in the map's string table.
    /// Setting the base upgrade's own value removes the change instead.
    pub fn set(
        &mut self,
        db: &UpgradeDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        text: &str,
    ) -> Result<()> {
        let field_id = FourCC::parse(&field.id)
            .ok_or_else(|| Error::Invalid(format!("bad field ID {}", field.id)))?;
        let Some(base) = self.base_of(db, id) else {
            return Err(Error::Invalid(format!("unknown upgrade {id}")));
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
    /// fields without levels) so the base upgrade's value applies again.
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

            // A changed game upgrade with no changes left is dropped entirely.
            data.original.retain(|e| e.mods().next().is_some());
        }

        self.drop_unused_strings(strings);
        removed
    }

    /// Removes strings this editor added that no upgrade field refers to anymore.
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

    /// Creates a custom upgrade copying `base` and returns its new ID, the
    /// next free one of `R000`, `R001`, ...
    pub fn create(&mut self, db: &UpgradeDatabase, base: FourCC) -> Result<FourCC> {
        if !db.contains(&base.to_string()) {
            return Err(Error::Invalid(format!("unknown base upgrade {base}")));
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

    /// Updates the string a text field already refers to, or adds a new one
    /// with a comment like the World Editor's: `Upgrades: R000 (name), Name (Name), level 1`.
    fn store_string(
        &mut self,
        db: &UpgradeDatabase,
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
            "Upgrades: {id} ({}), {} ({})",
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

    /// The upgrade files to write: those the map had, and new ones that now
    /// hold something.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        [
            (UPGRADES_FILE, &self.tables.data),
            (UPGRADE_SKINS_FILE, &self.tables.skin),
        ]
        .into_iter()
        .filter(|(file, data)| {
            !data.custom.is_empty() || !data.original.is_empty() || self.loaded.contains(file)
        })
        .map(|(file, data)| (file, data.to_bytes()))
        .collect()
    }
}

/// An upgrade's entries in the gameplay and skin tables.
struct Found<'a> {
    id: FourCC,
    data: Option<&'a ObjectEntry>,
    skin: Option<&'a ObjectEntry>,
}

impl<'a> Found<'a> {
    fn in_map(&self) -> bool {
        self.data.is_some() || self.skin.is_some()
    }

    fn base(&self, db: &UpgradeDatabase) -> Option<FourCC> {
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
        // editors keep every field in war3map.w3q.
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
        db: &UpgradeDatabase,
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

    fn levels(&self, db: &UpgradeDatabase) -> u32 {
        let Some(field) = db.field(LEVELS) else {
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

/// Whether a stored change is the one for `level`. Fields without levels
/// match any level: maps store them at level 0, but not every editor did.
fn at_level(field: &FieldMeta, m: &Modification, level: u32) -> bool {
    !field.per_level || m.level == Some(level)
}

/// The upgrade's entry in the gameplay or skin table, created when missing.
/// A custom upgrade missing from this table gets a custom entry with its base.
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

/// Whether `value` equals the base upgrade's game value at `level` in both
/// art modes (some art differs between SD and HD; those always keep the change).
fn same_as_base(
    db: &UpgradeDatabase,
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
