//! A map's ability changes on top of the game's ability data: leveled
//! objects ([`LeveledObjects`]) whose per-level data fields also store their
//! data column, with what the Object Editor shows for abilities.
//!
//! Text fields refer to the map's string table, which [`MapUnits`] keeps:
//! the methods that read or write text take it as an argument.
//!
//! [`MapUnits`]: crate::units::MapUnits

use wc3_data::abilities::{AbilityDatabase, AbilityInfo, PlaceholderSource};
use wc3_data::game::ArtMode;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Value};
use wc3_data::strings::MapStrings;

use crate::leveled::{ABILITIES, LeveledObjects};
use crate::store::MapObject;
use crate::transfer::Tables;
use crate::{Map, Result};

const TRUE: &str = "1";

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
    objects: LeveledObjects,
}

impl MapAbilities {
    pub fn load(map: &Map) -> Result<Self> {
        Ok(Self {
            objects: LeveledObjects::load(map, ABILITIES)?,
        })
    }

    pub fn empty() -> Self {
        Self {
            objects: LeveledObjects::empty(ABILITIES),
        }
    }

    pub(crate) fn tables(&self) -> &Tables {
        self.objects.tables()
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        self.objects.tables_mut()
    }

    #[cfg(test)]
    pub(crate) fn from_tables(tables: Tables) -> Self {
        Self {
            objects: LeveledObjects::from_tables(ABILITIES, tables),
        }
    }

    pub fn abilities(&self) -> Vec<MapObject> {
        self.objects.objects()
    }

    pub fn has(&self, id: FourCC) -> bool {
        self.objects.has(id)
    }

    pub fn base_of(&self, db: &AbilityDatabase, id: FourCC) -> Option<FourCC> {
        self.objects.base_of(db, id)
    }

    /// The base code that decides the ability's data fields (`AHhb`).
    pub fn code(&self, db: &AbilityDatabase, id: FourCC) -> Option<String> {
        db.code(&self.base_of(db, id)?.to_string())
    }

    /// Fields the Object Editor shows for this ability: the generic ones for
    /// its kind (hero, item or unit ability) and its base code's data fields.
    pub fn fields<'a>(&self, db: &'a AbilityDatabase, id: FourCC) -> Vec<&'a FieldMeta> {
        let lookup = self.objects.lookup(db, id);
        let Some(code) = lookup.base().and_then(|base| db.code(&base.to_string())) else {
            return Vec::new();
        };

        let flag = |field_id| lookup.plain_value(field_id).as_deref() == Some(TRUE);

        db.fields_for(&code, flag(HERO), flag(ITEM))
    }

    pub fn levels(&self, db: &AbilityDatabase, id: FourCC) -> u32 {
        self.objects.levels(db, id)
    }

    pub fn modification(&self, id: FourCC, field: &FieldMeta, level: u32) -> Option<&Value> {
        self.objects.modification(id, field, level)
    }

    pub fn is_modified(&self, id: FourCC, field: &FieldMeta, level: u32) -> bool {
        self.objects.is_modified(id, field, level)
    }

    pub fn raw_value(
        &self,
        db: &AbilityDatabase,
        strings: &MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        self.objects.raw_value(db, strings, id, field, level, mode)
    }

    pub fn name(&self, db: &AbilityDatabase, strings: &MapStrings, id: FourCC) -> String {
        self.objects.name(db, strings, id)
    }

    pub fn info(
        &self,
        db: &AbilityDatabase,
        strings: &MapStrings,
        id: FourCC,
        mode: ArtMode,
    ) -> Option<AbilityInfo> {
        let lookup = self.objects.lookup(db, id);
        let code = db.code(&lookup.base()?.to_string())?;
        let text = |field_id: &str| {
            db.field(field_id)
                .and_then(|field| lookup.raw_value(strings, field, 0, mode))
                .filter(|v| !v.is_empty())
        };
        let number = |field_id: &str| {
            lookup
                .plain_value(field_id)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or_default()
        };
        let button = |(x, y): (&str, &str)| {
            let x = lookup.plain_value(x)?.trim().parse().ok()?;
            let y = lookup.plain_value(y)?.trim().parse().ok()?;
            Some((x, y))
        };

        Some(AbilityInfo {
            id: id.to_string(),
            code,
            name: text(NAME).unwrap_or_else(|| id.to_string()),
            suffix: text(SUFFIX),
            icon: text(ICON),
            hero: lookup.plain_value(HERO).as_deref() == Some(TRUE),
            item: lookup.plain_value(ITEM).as_deref() == Some(TRUE),
            race: text(RACE).unwrap_or_default(),
            levels: lookup.levels(LEVELS),
            required_level: number(REQUIRED_LEVEL),
            hotkey: text(HOTKEY),
            learn_hotkey: text(LEARN_HOTKEY),
            button: button(BUTTON),
            learn_button: button(LEARN_BUTTON),
        })
    }

    pub fn set(
        &mut self,
        db: &AbilityDatabase,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
        text: &str,
    ) -> Result<()> {
        self.objects.set(db, strings, id, field, level, text)
    }

    pub fn reset(
        &mut self,
        strings: &mut MapStrings,
        id: FourCC,
        field: &FieldMeta,
        level: u32,
    ) -> bool {
        self.objects.reset(strings, id, field, level)
    }

    pub fn create(&mut self, db: &AbilityDatabase, base: FourCC) -> Result<FourCC> {
        self.objects.create(db, base)
    }

    pub fn remove(&mut self, strings: &mut MapStrings, id: FourCC) -> bool {
        self.objects.remove(strings, id)
    }

    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        self.objects.files()
    }
}

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
