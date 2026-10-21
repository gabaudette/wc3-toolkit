//! Object Editor field definitions from `*MetaData.slk`.
//!
//! The World Editor builds its Object Editor from these files: every field's
//! 4-letter ID (the one stored in maps), where its value lives, its category,
//! display name, type and limits.

use std::collections::HashMap;

use crate::slk::Slk;
use crate::{Error, Result};

/// Data rows start after the header row, which is row 1 in the file.
const FIRST_DATA_ROW: usize = 2;

/// Which object kinds a field applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Hero,
    Unit,
    Building,
    Item,
}

#[derive(Debug, Clone)]
pub struct FieldMeta {
    /// Field ID stored in map files, e.g. `uhpm`.
    pub id: String,
    /// Column (SLK) or key (profile) holding the value, e.g. `HP`.
    pub field: String,
    /// Source table: `UnitData`, `UnitBalance`, `UnitUI`, `UnitWeapons`, `UnitAbilities`, `Profile`, ...
    pub source: String,
    /// Position inside a comma-separated value, or `None` for the whole value.
    pub index: Option<usize>,
    pub category: String,
    /// `WESTRING_...` key of the editor's display name.
    pub display_key: String,
    pub sort: String,
    /// Value type, e.g. `int`, `real`, `unreal`, `bool`, `armorType`, `abilityList`.
    pub value_type: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub can_be_empty: bool,
    pub use_hero: bool,
    pub use_unit: bool,
    pub use_building: bool,
    pub use_item: bool,
    /// Restricts the field to specific object IDs when non-empty.
    pub use_specific: Vec<String>,
    /// Object IDs the field does not apply to.
    pub not_specific: Vec<String>,
    /// Per-level fields (abilities, upgrades): the column name gets the level
    /// appended, e.g. `Cool` → `Cool1`, `Cool2`, ...
    pub per_level: bool,
    /// Spell-specific data column: 1 = `DataA`, 2 = `DataB`, ... (field `Data`).
    pub data_column: Option<u32>,
    /// Cosmetic field that may differ between players. Reforged stores changes
    /// to these in the map's `Skin` object files.
    pub netsafe: bool,
}

impl FieldMeta {
    /// The table column holding this field's value at `level` (1-based) for
    /// per-level fields, e.g. `DataA2` or `Cool2`; the plain column otherwise.
    pub fn column(&self, level: u32) -> String {
        const FIRST_DATA_LETTER: u8 = b'A';

        let base = match self.data_column {
            Some(n) => {
                let letter = u8::try_from(n - 1)
                    .map_or('?', |offset| char::from(FIRST_DATA_LETTER + offset));
                format!("{}{letter}", self.field)
            }
            None => self.field.clone(),
        };

        if self.per_level {
            return format!("{base}{level}");
        }

        base
    }

    /// Whether the field applies to the object with this ID (or base code).
    pub fn applies_to_id(&self, code: &str) -> bool {
        let allowed = self.use_specific.is_empty() || self.use_specific.iter().any(|s| s == code);
        allowed && !self.not_specific.iter().any(|s| s == code)
    }

    pub fn applies_to(&self, kind: ObjectKind) -> bool {
        match kind {
            ObjectKind::Hero => self.use_hero,
            ObjectKind::Unit => self.use_unit,
            ObjectKind::Building => self.use_building,
            ObjectKind::Item => self.use_item,
        }
    }
}

pub fn parse_metadata(slk: &Slk) -> Result<Vec<FieldMeta>> {
    slk.records()
        .enumerate()
        .map(|(i, record)| parse_record(&record, i + FIRST_DATA_ROW))
        .collect()
}

fn parse_record(record: &HashMap<&str, &str>, row: usize) -> Result<FieldMeta> {
    let get = |column: &str| record.get(column).copied().filter(|v| !v.is_empty());
    let flag = |column: &str| get(column) == Some("1");
    let required = |column: &'static str| {
        get(column)
            .map(str::to_owned)
            .ok_or(Error::Field { row, column })
    };

    // A negative index means the value is not part of a list.
    let index = match get("index") {
        None => None,
        Some(raw) => {
            let parsed: i64 = raw.parse().map_err(|_| Error::Field {
                row,
                column: "index",
            })?;
            usize::try_from(parsed).ok()
        }
    };

    let id_list = |column: &str| -> Vec<String> {
        get(column)
            .map(|s| s.split(',').map(str::to_owned).collect())
            .unwrap_or_default()
    };
    let positive = |column: &str| {
        get(column)
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|n| *n > 0)
    };

    Ok(FieldMeta {
        id: required("ID")?,
        field: required("field")?,
        source: required("slk")?,
        index,
        category: get("category").unwrap_or_default().to_owned(),
        display_key: get("displayName").unwrap_or_default().to_owned(),
        sort: get("sort").unwrap_or_default().to_owned(),
        value_type: required("type")?,
        min: get("minVal").map(str::to_owned),
        max: get("maxVal").map(str::to_owned),
        can_be_empty: flag("canBeEmpty"),
        use_hero: flag("useHero"),
        use_unit: flag("useUnit"),
        use_building: flag("useBuilding"),
        use_item: flag("useItem"),
        use_specific: id_list("useSpecific"),
        not_specific: id_list("notSpecific"),
        per_level: positive("repeat").is_some(),
        data_column: positive("data"),
        netsafe: flag("netsafe"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_field_rows() {
        let text = "ID;P\nC;X1;Y1;K\"ID\"\nC;X2;K\"field\"\nC;X3;K\"slk\"\nC;X4;K\"index\"\n\
                    C;X5;K\"type\"\nC;X6;K\"useHero\"\n\
                    C;X1;Y2;K\"uhpm\"\nC;X2;K\"HP\"\nC;X3;K\"UnitBalance\"\nC;X4;K-1\n\
                    C;X5;K\"int\"\nC;X6;K1\n";
        let fields = parse_metadata(&Slk::parse(text).unwrap()).unwrap();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].id, "uhpm");
        assert_eq!(fields[0].index, None);
        assert!(fields[0].applies_to(ObjectKind::Hero));
        assert!(!fields[0].applies_to(ObjectKind::Item));
    }

    #[test]
    fn reports_missing_required_column() {
        let text = "ID;P\nC;X1;Y1;K\"ID\"\nC;X1;Y2;K\"uhpm\"\n";
        let err = parse_metadata(&Slk::parse(text).unwrap()).unwrap_err();

        assert!(matches!(err, Error::Field { row: 2, .. }), "{err}");
    }
}
