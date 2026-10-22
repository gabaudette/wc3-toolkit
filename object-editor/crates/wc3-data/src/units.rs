//! The game's units, heroes and buildings with every Object Editor field.
//!
//! Field definitions come from `UnitMetaData.slk`. Each one names the table
//! (`UnitBalance`, `UnitWeapons`, ...) or profile key holding its value, so this
//! module only needs to load those sources and look values up.

use std::collections::HashMap;

use crate::game::{ArtMode, GameFiles, decode_text};
use crate::meta::{FieldMeta, ObjectKind, parse_metadata};
use crate::profile::{Profile, split_list, unquote};
use crate::slk::Slk;
use crate::{Error, Result};

const METADATA: &str = "units\\unitmetadata.slk";
const EDITOR_DATA: &str = "ui\\uniteditordata.txt";
/// Standard commands (Move, Attack, ...) with their hotkeys.
const COMMAND_STRINGS: &str = "units\\commandstrings.txt";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];

/// Game data tables and the metadata source name that refers to each one.
const TABLES: &[(&str, &str)] = &[
    ("UnitData", "units\\unitdata.slk"),
    ("UnitBalance", "units\\unitbalance.slk"),
    ("UnitUI", "units\\unitui.slk"),
    ("UnitWeapons", "units\\unitweapons.slk"),
    ("UnitAbilities", "units\\unitabilities.slk"),
];

const PROFILE_SOURCE: &str = "Profile";
const RACES: &[&str] = &["campaign", "human", "neutral", "nightelf", "orc", "undead"];

/// Text files that only name other objects (abilities, items, upgrades), used
/// to show readable names in list fields.
const NAME_SOURCES: &[&str] = &[
    "units\\commonabilitystrings.txt",
    "units\\itemabilitystrings.txt",
    "units\\abilityskinstrings.txt",
    "units\\itemstrings.txt",
];

/// Metadata category → `WorldEditStrings` key of its display name.
pub(crate) const CATEGORY_NAMES: &[(&str, &str)] = &[
    ("abil", "WESTRING_OE_CAT_ABILITIES"),
    ("art", "WESTRING_OE_CAT_ART"),
    ("combat", "WESTRING_OE_CAT_COMBAT"),
    ("data", "WESTRING_OE_CAT_DATA"),
    ("editor", "WESTRING_OE_CAT_EDITOR"),
    ("move", "WESTRING_OE_CAT_MOVEMENT"),
    ("path", "WESTRING_OE_CAT_PATHING"),
    ("sound", "WESTRING_OE_CAT_SOUND"),
    ("stats", "WESTRING_OE_CAT_STATS"),
    ("tech", "WESTRING_OE_CAT_TECHTREE"),
    ("text", "WESTRING_OE_CAT_TEXT"),
];

/// Editor strings may point at another string (`WESTRING_UE_UNITRACE_HUMAN=WESTRING_RACE_HUMAN`).
const STRING_KEY_PREFIX: &str = "WESTRING_";
const MAX_STRING_REDIRECTS: usize = 4;
/// Flag labels read "Ground, &Ground": the display label, then the menu label.
const MENU_LABEL_SEPARATOR: &str = ", &";
const NONE_KEY: &str = "WESTRING_NONE";
/// `UnitEditorData.txt` lists other spellings of choice `01` under `01_Alt`
/// (keys are stored lowercase).
const ALIAS_SUFFIX: &str = "_alt";

/// Values the game uses for "no value" in tables.
const EMPTY_MARKERS: &[&str] = &["", "-", "_"];
const REAL_DECIMALS: usize = 3;

/// One choice of an enumerated field type such as `armorType`.
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// Value stored in data and maps, e.g. `Metal`.
    pub value: String,
    /// Label shown in the editor, e.g. `Metal` or `Heavy`.
    pub label: String,
    /// Other spellings the game accepts, e.g. `invu` for `invulnerable`.
    pub aliases: Vec<String>,
}

impl EnumValue {
    /// Whether `text` is this choice's value or one of its other spellings.
    pub fn matches(&self, text: &str) -> bool {
        self.value.eq_ignore_ascii_case(text)
            || self.aliases.iter().any(|a| a.eq_ignore_ascii_case(text))
    }
}

pub struct UnitDatabase {
    fields: Vec<FieldMeta>,
    tables: HashMap<String, Slk>,
    profile: Profile,
    names: Profile,
    editor_strings: Profile,
    enums: HashMap<String, Vec<EnumValue>>,
    commands: Profile,
    ids: Vec<String>,
}

impl UnitDatabase {
    pub fn load(files: &GameFiles) -> Result<Self> {
        let metadata = load_slk(files.read_base(METADATA)?, METADATA)?;
        let fields: Vec<FieldMeta> = parse_metadata(&metadata)
            .map_err(|e| e.in_file(METADATA))?
            .into_iter()
            .filter(|f| f.use_hero || f.use_unit || f.use_building)
            .collect();

        let mut tables = HashMap::new();
        for (source, path) in TABLES {
            let table = load_slk(files.read_balanced(path)?, path)?;
            tables.insert(source.to_ascii_lowercase(), table);
        }

        let ids = tables
            .get("unitdata")
            .map(|t| t.keys().map(str::to_owned).collect())
            .unwrap_or_default();

        let editor_strings = load_profile(files, EDITOR_STRINGS, GameFiles::read_localized)?;
        let editor_data = load_profile(files, &[EDITOR_DATA], GameFiles::read_base)?;
        let enums = parse_enums(&editor_data, &editor_strings);
        let commands = load_profile(files, &[COMMAND_STRINGS], GameFiles::read_localized)?;

        Ok(Self {
            fields,
            tables,
            profile: load_unit_profile(files)?,
            names: load_name_profile(files)?,
            editor_strings,
            enums,
            commands,
            ids,
        })
    }

    /// Every unit ID in the game data, in table order.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn contains(&self, id: &str) -> bool {
        self.tables.get("unitdata").is_some_and(|t| t.has_row(id))
    }

    /// All unit, hero and building fields.
    pub fn fields(&self) -> &[FieldMeta] {
        &self.fields
    }

    pub fn field(&self, field_id: &str) -> Option<&FieldMeta> {
        self.fields.iter().find(|f| f.id == field_id)
    }

    /// Fields the Object Editor shows for this unit.
    pub fn fields_for(&self, id: &str) -> impl Iterator<Item = &FieldMeta> {
        let kind = self.kind(id);
        self.fields.iter().filter(move |f| {
            f.applies_to(kind)
                && (f.use_specific.is_empty() || f.use_specific.iter().any(|s| s == id))
        })
    }

    /// Buildings are flagged in `UnitBalance`; hero IDs start with an uppercase letter.
    pub fn kind(&self, id: &str) -> ObjectKind {
        let is_building = self
            .tables
            .get("unitbalance")
            .and_then(|t| t.get(id, "isbldg"))
            == Some("1");
        if is_building {
            return ObjectKind::Building;
        }

        if id.starts_with(|c: char| c.is_ascii_uppercase()) {
            return ObjectKind::Hero;
        }

        ObjectKind::Unit
    }

    /// Display name of a unit, or of any object a unit field refers to (ability,
    /// item, upgrade), with the editor suffix that tells campaign variants apart.
    pub fn name(&self, id: &str) -> String {
        object_name(&self.profile, id)
            .or_else(|| object_name(&self.names, id))
            .unwrap_or_else(|| id.to_owned())
    }

    /// The editor's label for a field, e.g. `Art - Model File`.
    pub fn field_label(&self, field: &FieldMeta) -> String {
        format!(
            "{} - {}",
            self.category_label(&field.category),
            self.field_name(field)
        )
    }

    /// The editor's name for a metadata category, e.g. `art` → `Art`.
    pub fn category_label(&self, category: &str) -> String {
        CATEGORY_NAMES
            .iter()
            .find(|(c, _)| *c == category)
            .and_then(|(_, key)| self.editor_string(key))
            .unwrap_or_else(|| category.to_owned())
    }

    /// A field's display name without its category, e.g. `Model File`.
    pub fn field_name(&self, field: &FieldMeta) -> String {
        self.editor_string(&field.display_key)
            .unwrap_or_else(|| field.field.clone())
    }

    /// The raw stored value of a field, or `None` when the unit leaves it empty.
    pub fn raw_value(&self, id: &str, field: &FieldMeta, mode: ArtMode) -> Option<String> {
        let whole = if field.source == PROFILE_SOURCE {
            self.profile_value(id, &field.field, mode)
        } else {
            // Reforged moved some table columns into the skin files, so fall back to them.
            self.tables
                .get(&field.source.to_ascii_lowercase())
                .and_then(|t| t.get(id, &field.field))
                .filter(|v| !is_empty(v))
                .map(str::to_owned)
                .or_else(|| self.profile_value(id, &field.field, mode))
        }?;

        match field.index {
            None => Some(whole),
            Some(i) => split_list(&whole)
                .into_iter()
                .nth(i)
                .filter(|v| !is_empty(v)),
        }
    }

    /// A field's value formatted like the Object Editor shows it.
    pub fn display_value(&self, id: &str, field: &FieldMeta, mode: ArtMode) -> String {
        self.format_value(field, self.raw_value(id, field, mode))
    }

    /// Formats a raw value (as stored in game data or a map) like the Object Editor.
    pub fn format_value(&self, field: &FieldMeta, raw: Option<String>) -> String {
        let Some(raw) = raw else {
            return self.empty_display(field);
        };

        match field.value_type.as_str() {
            "bool" => match raw.as_str() {
                "1" => "True".to_owned(),
                "0" => "False".to_owned(),
                _ => raw,
            },
            "real" | "unreal" => raw
                .trim()
                .parse::<f64>()
                .map(|v| format!("{v:.REAL_DECIMALS$}"))
                .unwrap_or(raw),
            "string" | "char" => unquote(&raw).to_owned(),
            _ if field.is_bit_flags() => self.bit_flags(&field.value_type, &raw),
            "abilityList" | "heroAbilityList" | "abilitySkinList" | "unitList" | "itemList"
            | "upgradeList" | "techList" => self.list_names(&raw),
            other => match self.enums.get(other) {
                Some(values) => enum_labels(values, &raw),
                None => raw,
            },
        }
    }

    /// Choices of an enumerated field type such as `armorType`, if it is one.
    pub fn enum_values(&self, value_type: &str) -> Option<&[EnumValue]> {
        self.enums.get(value_type).map(Vec::as_slice)
    }

    /// Hotkey of a unit or upgrade from the game's text files, e.g. `F` for the Footman.
    pub fn hotkey(&self, id: &str) -> Option<char> {
        [&self.profile, &self.names]
            .iter()
            .find_map(|p| p.get(id, "Hotkey"))
            .and_then(parse_hotkey)
    }

    /// A standard command's name and hotkey, e.g. `CmdMove` → ("Move", `M`).
    pub fn command(&self, section: &str) -> Option<(String, char)> {
        let key = self
            .commands
            .get(section, "Hotkey")
            .and_then(parse_hotkey)?;
        let name = self
            .commands
            .get(section, "Tip")
            .map_or(section, unquote)
            .to_owned();
        Some((name, key))
    }

    /// The unit's icon path as stored in the data, e.g. `ReplaceableTextures\...\BTNHeroPaladin.blp`.
    pub fn icon_path(&self, id: &str, mode: ArtMode) -> Option<String> {
        self.profile_value(id, "Art", mode)
    }

    fn profile_value(&self, id: &str, key: &str, mode: ArtMode) -> Option<String> {
        let variant = format!("{key}:{}", mode.key_suffix());
        self.profile
            .get(id, &variant)
            .or_else(|| self.profile.get(id, key))
            .filter(|v| !is_empty(v))
            .map(str::to_owned)
    }

    fn editor_string(&self, key: &str) -> Option<String> {
        resolve_string(&self.editor_strings, key)
    }

    /// What the editor shows for a field the unit leaves empty.
    fn empty_display(&self, field: &FieldMeta) -> String {
        match field.value_type.as_str() {
            "int" => "0".to_owned(),
            "real" | "unreal" => format!("{:.REAL_DECIMALS$}", 0.0),
            "bool" => "False".to_owned(),
            "model" | "shadowTexture" => self.none_label(),
            _ => String::new(),
        }
    }

    fn none_label(&self) -> String {
        self.editor_string(NONE_KEY)
            .unwrap_or_else(|| "None".to_owned())
    }

    /// Bit flags: entry `n` of the type's enum labels bit `n`.
    fn bit_flags(&self, value_type: &str, raw: &str) -> String {
        let Ok(bits) = raw.trim().parse::<u32>() else {
            return raw.to_owned();
        };

        let labels: Vec<String> = self
            .enums
            .get(value_type)
            .into_iter()
            .flatten()
            .filter_map(|v| {
                let bit = v.value.parse::<u32>().ok()?;
                (bits & (1 << bit) != 0).then(|| v.label.clone())
            })
            .collect();

        if labels.is_empty() {
            return self.none_label();
        }

        labels.join(", ")
    }

    fn list_names(&self, raw: &str) -> String {
        split_list(raw)
            .iter()
            .filter(|id| !is_empty(id))
            .map(|id| self.name(id.trim()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl crate::abilities::PlaceholderSource for UnitDatabase {
    /// A unit's value from any unit table, e.g. `<osw1,realHP>` for a summoned wolf.
    fn placeholder(&self, id: &str, column: &str) -> Option<String> {
        self.tables
            .values()
            .find_map(|t| t.get(id, column))
            .filter(|v| !is_empty(v))
            .map(|v| v.trim().to_owned())
    }
}

/// A hotkey as the game stores it: one letter or digit. Values like `27` or
/// `512` are key codes for special keys and do not count.
pub fn parse_hotkey(value: &str) -> Option<char> {
    let mut chars = unquote(value).trim().chars();
    let key = chars.next()?;
    (chars.next().is_none() && key.is_ascii_alphanumeric()).then(|| key.to_ascii_uppercase())
}

fn is_empty(value: &str) -> bool {
    EMPTY_MARKERS.contains(&value.trim())
}

/// Looks up an editor string, following redirects to other strings and keeping
/// only the display part of flag labels.
pub(crate) fn resolve_string(strings: &Profile, key: &str) -> Option<String> {
    let mut value = strings.section_names().find_map(|s| strings.get(s, key))?;

    for _ in 0..MAX_STRING_REDIRECTS {
        let target = unquote(value);
        if !target.starts_with(STRING_KEY_PREFIX) {
            break;
        }

        match strings.section_names().find_map(|s| strings.get(s, target)) {
            Some(next) => value = next,
            None => break,
        }
    }

    let text = unquote(value);
    let display = text
        .split_once(MENU_LABEL_SEPARATOR)
        .map_or(text, |(label, _)| label);
    Some(display.replace('&', ""))
}

fn object_name(profile: &Profile, id: &str) -> Option<String> {
    let name = unquote(profile.get(id, "Name")?);
    match profile.get(id, "EditorSuffix").map(unquote) {
        Some(suffix) if !suffix.is_empty() => Some(format!("{name} {}", suffix.trim())),
        _ => Some(name.to_owned()),
    }
}

fn enum_labels(values: &[EnumValue], raw: &str) -> String {
    split_list(raw)
        .iter()
        .map(|item| {
            values
                .iter()
                .find(|v| v.matches(item.trim()))
                .map_or_else(|| item.clone(), |v| v.label.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Reads `UnitEditorData.txt`: `[armorType]` sections with `00=Metal,WESTRING_...` entries.
fn parse_enums(editor_data: &Profile, editor_strings: &Profile) -> HashMap<String, Vec<EnumValue>> {
    editor_data
        .section_names()
        .filter_map(|name| {
            let section = editor_data.section(name)?;
            let mut entries: Vec<(&String, &String)> = section
                .iter()
                .filter(|(k, _)| k.chars().all(|c| c.is_ascii_digit()))
                .collect();
            entries.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(u32::MAX));

            let values: Vec<EnumValue> = entries
                .into_iter()
                .filter_map(|(key, v)| {
                    let mut parts = split_list(v).into_iter();
                    let value = parts.next()?;
                    let label = parts
                        .next()
                        .and_then(|k| resolve_string(editor_strings, &k))
                        .unwrap_or_else(|| value.clone());
                    let aliases = section
                        .get(&format!("{key}{ALIAS_SUFFIX}"))
                        .map(|list| {
                            split_list(list)
                                .into_iter()
                                .map(|a| a.trim().to_owned())
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(EnumValue {
                        value,
                        label,
                        aliases,
                    })
                })
                .collect();

            (!values.is_empty()).then(|| (name.to_owned(), values))
        })
        .collect()
}

fn load_slk(data: Vec<u8>, path: &str) -> Result<Slk> {
    Slk::parse(&decode_text(data, path)?).map_err(|e| e.in_file(path))
}

type Reader = fn(&GameFiles, &str) -> Result<Vec<u8>>;

fn load_profile(files: &GameFiles, paths: &[&str], read: Reader) -> Result<Profile> {
    let mut profile = Profile::new();
    for path in paths {
        profile.merge_text(&decode_text(read(files, path)?, path)?);
    }

    Ok(profile)
}

/// Unit data, art (skin) and text files. Their keys never overlap, so the merge
/// order only matters for balance layers, which replace whole files.
fn load_unit_profile(files: &GameFiles) -> Result<Profile> {
    let func: Vec<String> = RACES
        .iter()
        .map(|r| format!("units\\{r}unitfunc.txt"))
        .collect();
    let strings: Vec<String> = RACES
        .iter()
        .map(|r| format!("units\\{r}unitstrings.txt"))
        .collect();
    let mut profile = Profile::new();

    for path in func
        .iter()
        .map(String::as_str)
        .chain(["units\\unitweaponsfunc.txt"])
    {
        profile.merge_text(&decode_text(files.read_balanced(path)?, path)?);
    }

    for path in ["units\\unitskin.txt", "units\\unitweaponsskin.txt"] {
        profile.merge_text(&decode_text(files.read_base(path)?, path)?);
    }

    for path in strings
        .iter()
        .map(String::as_str)
        .chain(["units\\unitskinstrings.txt"])
    {
        profile.merge_text(&decode_text(files.read_localized(path)?, path)?);
    }

    Ok(profile)
}

fn load_name_profile(files: &GameFiles) -> Result<Profile> {
    let race_files: Vec<String> = RACES
        .iter()
        .flat_map(|r| {
            [
                format!("units\\{r}abilitystrings.txt"),
                format!("units\\{r}upgradestrings.txt"),
            ]
        })
        .collect();
    let mut profile = Profile::new();

    for path in race_files
        .iter()
        .map(String::as_str)
        .chain(NAME_SOURCES.iter().copied())
    {
        match files.read_localized(path) {
            Ok(data) => profile.merge_text(&decode_text(data, path)?),
            // Not every race has every kind of strings file.
            Err(Error::Casc(casc::Error::NotFound(_))) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::parse_hotkey;

    #[test]
    fn hotkeys_are_single_letters_or_digits() {
        assert_eq!(parse_hotkey("t"), Some('T'));
        assert_eq!(parse_hotkey("\"F\""), Some('F'));
        assert_eq!(parse_hotkey("27"), None);
        assert_eq!(parse_hotkey(""), None);
        assert_eq!(parse_hotkey("-"), None);
    }
}
