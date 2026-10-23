//! The game's upgrades (researches such as Iron Forged Swords): names, icons,
//! costs, effects and values per level.
//!
//! Costs and effects live in `UpgradeData.slk` with one value for every level:
//! a base and a step per level (`goldbase`, `goldmod`). Names, tooltips, icons
//! and hotkeys are profile lists with one entry per level
//! (`Name=Iron Forged Swords,Steel Forged Swords,...`); requirements use one
//! key per level instead (`Requires`, `Requires1`, ...).

use std::collections::HashMap;

use crate::Result;
use crate::abilities::{load_slk, merge_optional};
use crate::game::{ArtMode, GameFiles, decode_text};
use crate::leveled::LeveledData;
use crate::meta::{FieldMeta, parse_metadata};
use crate::profile::{Profile, split_list, unquote};
use crate::slk::Slk;
use crate::units::{CATEGORY_NAMES, resolve_string};

const METADATA: &str = "units\\upgrademetadata.slk";
const TABLE: &str = "units\\upgradedata.slk";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];
const SOURCES: &[&str] = &[
    "campaign", "human", "item", "neutral", "nightelf", "orc", "undead",
];
const PROFILE_SOURCE: &str = "Profile";
const LEVELS_COLUMN: &str = "maxlevel";
const NAME_KEY: &str = "Name";
const ICON_KEY: &str = "Art";
/// What each column of an effect holds (`base1`, `mod1`, `code1`).
const EFFECT_PARTS: &[(&str, &str)] = &[("base", "Base"), ("mod", "Increment"), ("code", "Code")];

pub struct UpgradeDatabase {
    fields: Vec<FieldMeta>,
    /// Field ID → position in `fields`.
    field_index: HashMap<String, usize>,
    table: Slk,
    profile: Profile,
    editor_strings: Profile,
    ids: Vec<String>,
}

impl UpgradeDatabase {
    pub fn load(files: &GameFiles) -> Result<Self> {
        let metadata = load_slk(files.read_base(METADATA)?, METADATA)?;
        let fields = parse_metadata(&metadata).map_err(|e| e.in_file(METADATA))?;
        let table = load_slk(files.read_balanced(TABLE)?, TABLE)?;
        let ids = table.keys().map(str::to_owned).collect();
        let field_index = fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id.clone(), i))
            .collect();

        let mut profile = Profile::new();
        for source in SOURCES {
            let path = format!("units\\{source}upgradefunc.txt");
            merge_optional(&mut profile, files.read_balanced(&path), &path)?;
        }

        for source in SOURCES {
            let path = format!("units\\{source}upgradestrings.txt");
            merge_optional(&mut profile, files.read_localized(&path), &path)?;
        }

        let mut editor_strings = Profile::new();
        for path in EDITOR_STRINGS {
            editor_strings.merge_text(&decode_text(files.read_localized(path)?, path)?);
        }

        Ok(Self {
            fields,
            field_index,
            table,
            profile,
            editor_strings,
            ids,
        })
    }

    /// Every upgrade field the metadata defines; all apply to every upgrade.
    pub fn fields(&self) -> &[FieldMeta] {
        &self.fields
    }

    pub fn field(&self, field_id: &str) -> Option<&FieldMeta> {
        self.fields.get(*self.field_index.get(field_id)?)
    }

    /// A field's display name, e.g. `Gold Base`. The World Editor fills the
    /// `%s` in effect names (`Effect 1 - %s`) itself; this names the part.
    pub fn field_name(&self, field: &FieldMeta) -> String {
        let name = resolve_string(&self.editor_strings, &field.display_key)
            .unwrap_or_else(|| field.field.clone());
        let part = EFFECT_PARTS
            .iter()
            .find(|(column, _)| field.field.starts_with(column))
            .map_or("", |(_, part)| part);
        name.replace("%s", part)
    }

    /// The editor's name for a metadata category, e.g. `data` → `Data`.
    pub fn category_label(&self, category: &str) -> String {
        CATEGORY_NAMES
            .iter()
            .find(|(c, _)| *c == category)
            .and_then(|(_, key)| resolve_string(&self.editor_strings, key))
            .unwrap_or_else(|| category.to_owned())
    }

    /// Every upgrade ID in the game data, in table order.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn contains(&self, id: &str) -> bool {
        self.table.has_row(id)
    }

    /// The game's number of levels, at least 1.
    pub fn levels(&self, id: &str) -> u32 {
        self.table
            .get(id, LEVELS_COLUMN)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or_default()
            .max(1)
    }

    /// The level 1 name, or the ID when the game has none.
    pub fn name(&self, id: &str) -> String {
        self.level_text(id, NAME_KEY, 1)
            .unwrap_or_else(|| id.to_owned())
    }

    /// The level 1 icon.
    pub fn icon_path(&self, id: &str, mode: ArtMode) -> Option<String> {
        let variant = format!("{ICON_KEY}:{}", mode.key_suffix());
        let whole = self
            .profile
            .get(id, &variant)
            .or_else(|| self.profile.get(id, ICON_KEY))?;
        list_item(whole, 0)
    }

    fn level_text(&self, id: &str, key: &str, level: u32) -> Option<String> {
        let whole = self.profile.get(id, key)?;
        list_item(whole, usize::try_from(level.checked_sub(1)?).ok()?)
    }

    /// The game's raw value of a field at `level` (1-based; ignored for fields
    /// without levels), or `None` when the upgrade leaves it empty.
    pub fn raw_value(
        &self,
        id: &str,
        field: &FieldMeta,
        level: u32,
        mode: ArtMode,
    ) -> Option<String> {
        if field.source != PROFILE_SOURCE {
            return self
                .table
                .get(id, &field.field)
                .and_then(|v| present(v.trim()));
        }

        // Per-level fields without an index (requirements) have a key per
        // level: `Requires` for level 1, `Requires1` for level 2, ...
        if field.per_level && field.index.is_none() {
            let key = match level.checked_sub(1)? {
                0 => field.field.clone(),
                n => format!("{}{n}", field.field),
            };
            return self
                .profile
                .get(id, &key)
                .and_then(|v| present(unquote(v).trim()));
        }

        let variant = format!("{}:{}", field.field, mode.key_suffix());
        let whole = self
            .profile
            .get(id, &variant)
            .or_else(|| self.profile.get(id, &field.field))?;
        let position = if field.per_level {
            usize::try_from(level.checked_sub(1)?).ok()?
        } else {
            field.index.unwrap_or_default()
        };
        list_item(whole, position)
    }
}

/// Item `index` of a profile list, or `None` when it is missing or empty.
fn list_item(whole: &str, index: usize) -> Option<String> {
    split_list(whole)
        .into_iter()
        .nth(index)
        .and_then(|v| present(v.trim()))
}

/// The value, unless the game marks it empty (`-`, `_` or nothing).
fn present(value: &str) -> Option<String> {
    (!value.is_empty() && value != "-" && value != "_").then(|| value.to_owned())
}

impl LeveledData for UpgradeDatabase {
    fn fields(&self) -> &[FieldMeta] {
        UpgradeDatabase::fields(self)
    }

    fn field(&self, field_id: &str) -> Option<&FieldMeta> {
        UpgradeDatabase::field(self, field_id)
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        UpgradeDatabase::field_name(self, field)
    }

    fn category_label(&self, category: &str) -> String {
        UpgradeDatabase::category_label(self, category)
    }

    fn ids(&self) -> &[String] {
        UpgradeDatabase::ids(self)
    }

    fn contains(&self, id: &str) -> bool {
        UpgradeDatabase::contains(self, id)
    }

    fn raw_value(&self, id: &str, field: &FieldMeta, level: u32, mode: ArtMode) -> Option<String> {
        UpgradeDatabase::raw_value(self, id, field, level, mode)
    }
}
