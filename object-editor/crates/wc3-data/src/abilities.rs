//! The game's abilities: names, icons, tooltips and values per level, for
//! showing abilities as cards wherever a unit refers to them.
//!
//! Per-level values live in `AbilityData.slk` columns named after the level
//! (`Cool1`, `DataA2`, ...). `AbilityMetaData.slk` says which columns apply to
//! which ability and what they mean (`DataA` is "Amount Healed/Damaged" for
//! Holy Light). Tooltips refer to values with placeholders like `<AHhb,DataA1>`.

use crate::game::{GameFiles, decode_text};
use crate::meta::{FieldMeta, parse_metadata};
use crate::profile::{Profile, split_list, unquote};
use crate::slk::Slk;
use crate::units::resolve_string;
use crate::{Error, Result};

const METADATA: &str = "units\\abilitymetadata.slk";
const TABLE: &str = "units\\abilitydata.slk";
/// Tooltips also quote upgrade values, like `<Rhme,base1>`.
const UPGRADE_TABLE: &str = "units\\upgradedata.slk";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];
const SKIN: &str = "units\\abilityskin.txt";
const SKIN_STRINGS: &str = "units\\abilityskinstrings.txt";
const SOURCES: &[&str] = &[
    "campaign", "common", "human", "item", "neutral", "nightelf", "orc", "undead",
];

/// Generic per-level columns, in the order a card lists them.
const GENERIC_ORDER: &[&str] = &["Cost", "Cool", "Rng", "Area", "Dur", "HeroDur", "Cast"];
const NUMERIC_TYPES: &[&str] = &["int", "real", "unreal"];
const DATA_SOURCE: &str = "AbilityData";
const TRUE: &str = "1";
const PERCENT_SCALE: f64 = 100.0;

#[derive(Debug, Clone, PartialEq)]
pub struct AbilityInfo {
    pub id: String,
    /// The base ability this one copies (its own ID for originals).
    pub code: String,
    pub name: String,
    /// Editor-only text that tells similar abilities apart, like "(Neutral Hostile)".
    pub suffix: Option<String>,
    pub icon: Option<String>,
    pub hero: bool,
    pub item: bool,
    pub race: String,
    pub levels: u32,
    pub required_level: u32,
    pub hotkey: Option<String>,
    /// Key that learns the ability in the hero's learn menu.
    pub learn_hotkey: Option<String>,
    /// Button slot (column, row) on the command card and in the learn menu.
    pub button: Option<(u32, u32)>,
    pub learn_button: Option<(u32, u32)>,
}

/// One row of a card's table: a value for each level.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelValues {
    pub label: String,
    pub values: Vec<String>,
}

/// Another table that tooltip placeholders can refer to, such as unit data
/// for summoned units (`<osw1,realHP>`).
pub trait PlaceholderSource {
    fn placeholder(&self, id: &str, column: &str) -> Option<String>;
}

pub struct AbilityDatabase {
    fields: Vec<FieldMeta>,
    table: Slk,
    upgrades: Slk,
    profile: Profile,
    editor_strings: Profile,
    ids: Vec<String>,
}

impl AbilityDatabase {
    pub fn load(files: &GameFiles) -> Result<Self> {
        let metadata = load_slk(files.read_base(METADATA)?, METADATA)?;
        let fields = parse_metadata(&metadata).map_err(|e| e.in_file(METADATA))?;
        let table = load_slk(files.read_balanced(TABLE)?, TABLE)?;
        let upgrades = load_slk(files.read_balanced(UPGRADE_TABLE)?, UPGRADE_TABLE)?;
        let ids = table.keys().map(str::to_owned).collect();

        let mut profile = Profile::new();
        for source in SOURCES {
            let path = format!("units\\{source}abilityfunc.txt");
            merge_optional(&mut profile, files.read_balanced(&path), &path)?;
        }

        merge_optional(&mut profile, files.read_base(SKIN), SKIN)?;
        for source in SOURCES {
            let path = format!("units\\{source}abilitystrings.txt");
            merge_optional(&mut profile, files.read_localized(&path), &path)?;
        }

        merge_optional(
            &mut profile,
            files.read_localized(SKIN_STRINGS),
            SKIN_STRINGS,
        )?;

        let mut editor_strings = Profile::new();
        for path in EDITOR_STRINGS {
            editor_strings.merge_text(&decode_text(files.read_localized(path)?, path)?);
        }

        Ok(Self {
            fields,
            table,
            upgrades,
            profile,
            editor_strings,
            ids,
        })
    }

    /// Every ability ID in the game data, in table order.
    /// Every ability field the metadata defines.
    pub fn fields(&self) -> &[FieldMeta] {
        &self.fields
    }

    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn contains(&self, id: &str) -> bool {
        self.table.has_row(id)
    }

    fn text(&self, id: &str, key: &str) -> Option<String> {
        self.profile
            .get(id, key)
            .map(|v| unquote(v).trim().to_owned())
            .filter(|v| !v.is_empty())
    }

    fn cell(&self, id: &str, column: &str) -> Option<&str> {
        self.table
            .get(id, column)
            .map(str::trim)
            .filter(|v| !v.is_empty() && *v != "-" && *v != "_")
    }

    fn number(&self, id: &str, column: &str) -> u32 {
        self.cell(id, column)
            .and_then(|v| v.parse().ok())
            .unwrap_or_default()
    }

    pub fn info(&self, id: &str) -> Option<AbilityInfo> {
        if !self.contains(id) {
            return None;
        }

        Some(AbilityInfo {
            id: id.to_owned(),
            code: self.cell(id, "code").unwrap_or(id).to_owned(),
            name: self.text(id, "Name").unwrap_or_else(|| id.to_owned()),
            suffix: self.text(id, "EditorSuffix"),
            icon: self.text(id, "Art"),
            hero: self.cell(id, "hero") == Some(TRUE),
            item: self.cell(id, "item") == Some(TRUE),
            race: self.cell(id, "race").unwrap_or_default().to_owned(),
            levels: self.number(id, "levels").max(1),
            required_level: self.number(id, "reqLevel"),
            hotkey: self.text(id, "Hotkey"),
            learn_hotkey: self.text(id, "Researchhotkey"),
            button: self.text(id, "Buttonpos").as_deref().and_then(parse_button),
            learn_button: self
                .text(id, "Researchbuttonpos")
                .as_deref()
                .and_then(parse_button),
        })
    }

    fn per_level_text(
        &self,
        id: &str,
        key: &str,
        level: u32,
        others: &dyn PlaceholderSource,
    ) -> Option<String> {
        let raw = self.profile.get(id, key)?;
        let items = split_list(raw);
        let index = usize::try_from(level.saturating_sub(1)).ok()?;
        let text = items.get(index).or(items.last())?.trim().to_owned();
        (!text.is_empty()).then(|| self.fill_placeholders(&text, others))
    }

    /// The one-line tooltip at `level` (1-based), e.g. "Holy Light - [Level 1]".
    pub fn tooltip(&self, id: &str, level: u32, others: &dyn PlaceholderSource) -> Option<String> {
        self.per_level_text(id, "Tip", level, others)
    }

    /// The extended tooltip at `level`, with placeholders filled in.
    pub fn description(
        &self,
        id: &str,
        level: u32,
        others: &dyn PlaceholderSource,
    ) -> Option<String> {
        self.per_level_text(id, "Ubertip", level, others)
    }

    /// For hero abilities: the "learn" tooltip, which describes every level.
    pub fn learn_description(&self, id: &str, others: &dyn PlaceholderSource) -> Option<String> {
        self.text(id, "Researchubertip")
            .map(|t| self.fill_placeholders(&t, others))
    }

    /// Numeric values per level that apply to this ability: generic ones
    /// (mana, cooldown, range, ...) first, then its own data fields. Rows that
    /// are empty or zero on every level are left out.
    pub fn level_values(&self, id: &str) -> Vec<LevelValues> {
        let Some(info) = self.info(id) else {
            return Vec::new();
        };

        let mut fields: Vec<&FieldMeta> = self
            .fields
            .iter()
            .filter(|f| {
                f.per_level
                    && f.source == DATA_SOURCE
                    && f.applies_to_id(&info.code)
                    && NUMERIC_TYPES.contains(&f.value_type.as_str())
            })
            .collect();
        fields.sort_by_key(|f| {
            let generic = GENERIC_ORDER.iter().position(|g| *g == f.field);
            (generic.is_none(), generic, f.data_column, f.sort.clone())
        });

        let mut rows: Vec<LevelValues> = Vec::new();
        let mut seen_columns: Vec<String> = Vec::new();

        for field in fields {
            let column = field.column(1);
            if seen_columns.contains(&column) {
                continue;
            }

            seen_columns.push(column);
            let values: Vec<String> = (1..=info.levels)
                .map(|level| {
                    self.cell(id, &field.column(level))
                        .map(format_number)
                        .unwrap_or_default()
                })
                .collect();
            if values.iter().all(|v| v.is_empty() || v == "0") {
                continue;
            }

            let label = resolve_string(&self.editor_strings, &field.display_key)
                .unwrap_or_else(|| field.field.clone());
            rows.push(LevelValues { label, values });
        }

        rows
    }

    /// Replaces `<AHhb,DataA1>` (and `<AHhb,DataA1,%>` for percentages) with the
    /// value from the ability, upgrade or `others` tables. Unknown placeholders
    /// are left as they are.
    pub fn fill_placeholders(&self, text: &str, others: &dyn PlaceholderSource) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;

        while let Some(start) = rest.find('<') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find('>') else {
                out.push_str(&rest[start..]);
                return out;
            };

            let inner = &after[..end];
            match self.placeholder_value(inner, others) {
                Some(value) => out.push_str(&value),
                None => {
                    out.push('<');
                    out.push_str(inner);
                    out.push('>');
                }
            }

            rest = &after[end + 1..];
        }

        out.push_str(rest);
        out
    }

    fn placeholder_value(&self, inner: &str, others: &dyn PlaceholderSource) -> Option<String> {
        let mut parts = inner.split(',');
        let id = parts.next()?.trim();
        let column = parts.next()?.trim();
        let percent = parts.next().is_some_and(|p| p.trim() == "%");
        let raw = self
            .cell(id, column)
            .map(str::to_owned)
            .or_else(|| self.upgrades.get(id, column).map(|v| v.trim().to_owned()))
            .or_else(|| others.placeholder(id, column))?;

        if percent {
            let value: f64 = raw.parse().ok()?;
            return Some(format_number(&(value * PERCENT_SCALE).to_string()));
        }

        Some(format_number(&raw))
    }
}

/// No other tables, for callers without unit data.
pub struct NoPlaceholders;

impl PlaceholderSource for NoPlaceholders {
    fn placeholder(&self, _id: &str, _column: &str) -> Option<String> {
        None
    }
}

/// A button slot such as `0,2` → (column 0, row 2).
fn parse_button(value: &str) -> Option<(u32, u32)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// `200.00` → `200`, `5.50` → `5.5`; anything that is not a number stays as it is.
fn format_number(raw: &str) -> String {
    const MAX_DECIMALS: usize = 2;

    let Ok(value) = raw.trim().parse::<f64>() else {
        return raw.trim().to_owned();
    };

    let fixed = format!("{value:.MAX_DECIMALS$}");
    let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        return "0".to_owned();
    }

    trimmed.to_owned()
}

fn load_slk(data: Vec<u8>, path: &str) -> Result<Slk> {
    Slk::parse(&decode_text(data, path)?).map_err(|e| e.in_file(path))
}

/// Merges a text file that not every install or locale has.
fn merge_optional(profile: &mut Profile, data: Result<Vec<u8>>, path: &str) -> Result<()> {
    match data {
        Ok(bytes) => {
            profile.merge_text(&decode_text(bytes, path)?);
            Ok(())
        }
        Err(Error::Casc(casc::Error::NotFound(_))) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn formats_numbers_like_tooltips() {
        assert_eq!(format_number("200.00"), "200");
        assert_eq!(format_number("5.5"), "5.5");
        assert_eq!(format_number("0.333"), "0.33");
        assert_eq!(format_number("-0.0"), "0");
        assert_eq!(format_number("hfoo"), "hfoo");
    }
}
