//! The game's doodads: decoration without gameplay (props, plants, rocks,
//! lights). A few art fields (the tint) have one value per model variation;
//! the map files store them by variation the way abilities store levels.

use std::collections::HashMap;

use crate::Result;
use crate::abilities::load_slk;
use crate::game::{ArtMode, GameFiles, decode_text};
use crate::leveled::LeveledData;
use crate::meta::{FieldMeta, parse_metadata};
use crate::profile::{Profile, split_list, unquote};
use crate::slk::Slk;
use crate::units::{CATEGORY_NAMES, resolve_string};

const METADATA: &str = "doodads\\doodadmetadata.slk";
const TABLE: &str = "doodads\\doodads.slk";
const SKINS: &str = "doodads\\doodadskins.txt";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];
const TABLE_SOURCE: &str = "DoodadData";
const NAME_COLUMN: &str = "Name";
const VARIATIONS_KEY: &str = "numVar";
const STRING_KEY_PREFIX: &str = "WESTRING_";
/// Value types holding a whole list: the metadata still gives them index 0.
const LIST_SUFFIX: &str = "List";

pub struct DoodadDatabase {
    fields: Vec<FieldMeta>,
    field_index: HashMap<String, usize>,
    table: Slk,
    profile: Profile,
    editor_strings: Profile,
    ids: Vec<String>,
}

impl DoodadDatabase {
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

        profile.merge_text(&decode_text(files.read_base(SKINS)?, SKINS)?);
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

    /// The doodad's name, with editor string keys (`WESTRING_DOOD_APMS`) resolved.
    pub fn name(&self, id: &str) -> String {
        let raw = self
            .table
            .get(id, NAME_COLUMN)
            .or_else(|| self.profile.get(id, NAME_COLUMN))
            .map(unquote)
            .unwrap_or(id);

        if raw.starts_with(STRING_KEY_PREFIX) {
            return resolve_string(&self.editor_strings, raw).unwrap_or_else(|| raw.to_owned());
        }

        raw.to_owned()
    }

    pub fn variations(&self, id: &str) -> u32 {
        self.whole(id, VARIATIONS_KEY, TABLE_SOURCE, ArtMode::Sd)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1)
            .max(1)
    }

    /// A field's whole stored value: the table first, then the skins, where
    /// Reforged moved most doodad art.
    fn whole(&self, id: &str, key: &str, source: &str, mode: ArtMode) -> Option<String> {
        let from_table = (source == TABLE_SOURCE)
            .then(|| self.table.get(id, key))
            .flatten();
        let variant = format!("{key}:{}", mode.key_suffix());

        from_table
            .or_else(|| self.profile.get(id, &variant))
            .or_else(|| self.profile.get(id, key))
            .map(|v| unquote(v.trim()).to_owned())
            .filter(|v| present(v))
    }
}

impl LeveledData for DoodadDatabase {
    fn fields(&self) -> &[FieldMeta] {
        &self.fields
    }

    fn field(&self, field_id: &str) -> Option<&FieldMeta> {
        self.fields.get(*self.field_index.get(field_id)?)
    }

    fn field_name(&self, field: &FieldMeta) -> String {
        resolve_string(&self.editor_strings, &field.display_key)
            .unwrap_or_else(|| field.field.clone())
    }

    fn category_label(&self, category: &str) -> String {
        CATEGORY_NAMES
            .iter()
            .find(|(c, _)| *c == category)
            .and_then(|(_, key)| resolve_string(&self.editor_strings, key))
            .unwrap_or_else(|| category.to_owned())
    }

    fn ids(&self) -> &[String] {
        &self.ids
    }

    fn contains(&self, id: &str) -> bool {
        self.table.has_row(id)
    }

    /// Per-variation fields are lists with one entry per variation
    /// (`vertR=255,255,...`): variation 1 is the first.
    fn raw_value(&self, id: &str, field: &FieldMeta, level: u32, mode: ArtMode) -> Option<String> {
        let whole = self.whole(id, &field.field, &field.source, mode)?;
        let position = if field.per_level {
            Some(usize::try_from(level.checked_sub(1)?).ok()?)
        } else {
            // List types (tilesets) also say index 0; for them it means the whole list.
            field
                .index
                .filter(|i| *i > 0 || !field.value_type.ends_with(LIST_SUFFIX))
        };

        match position {
            Some(i) => split_list(&whole)
                .into_iter()
                .nth(i)
                .map(|v| v.trim().to_owned())
                .filter(|v| present(v)),
            None => Some(whole),
        }
    }
}

/// Whether a value is set: the game marks empty values with `-` or `_`.
fn present(value: &str) -> bool {
    !value.is_empty() && value != "-" && value != "_"
}
