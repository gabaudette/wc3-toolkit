//! The field guide: a description for every Object Editor field, which fields
//! come first in each category, and which are rarely needed ("advanced").
//!
//! The guide is written for this project and lives in `data/*.toml`.

use std::collections::HashMap;

use serde::Deserialize;

use crate::{Error, Result};

const UNIT_GUIDE: &str = include_str!("../data/unit-fields.toml");
const UNIT_GUIDE_FILE: &str = "data/unit-fields.toml";
const ABILITY_GUIDE: &str = include_str!("../data/ability-fields.toml");
const ABILITY_GUIDE_FILE: &str = "data/ability-fields.toml";
const ITEM_GUIDE: &str = include_str!("../data/item-fields.toml");
const ITEM_GUIDE_FILE: &str = "data/item-fields.toml";
const UPGRADE_GUIDE: &str = include_str!("../data/upgrade-fields.toml");
const UPGRADE_GUIDE_FILE: &str = "data/upgrade-fields.toml";
const BUFF_GUIDE: &str = include_str!("../data/buff-fields.toml");
const BUFF_GUIDE_FILE: &str = "data/buff-fields.toml";
const DESTRUCTABLE_GUIDE: &str = include_str!("../data/destructable-fields.toml");
const DESTRUCTABLE_GUIDE_FILE: &str = "data/destructable-fields.toml";
const DOODAD_GUIDE: &str = include_str!("../data/doodad-fields.toml");
const DOODAD_GUIDE_FILE: &str = "data/doodad-fields.toml";

#[derive(Debug, Clone, PartialEq)]
pub struct GuideEntry {
    pub description: String,
    /// Rarely needed; shown in a folded "Advanced" group.
    pub advanced: bool,
    /// Not confirmed in game; the editor marks it.
    pub unverified: bool,
}

#[derive(Debug, Clone)]
pub struct FieldGuide {
    entries: HashMap<String, GuideEntry>,
    /// Field ID → (category it is listed under, position in that list).
    ranks: HashMap<String, (String, usize)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuideFile {
    #[serde(default)]
    order: HashMap<String, Vec<String>>,
    #[serde(default)]
    field: Vec<FieldEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldEntry {
    ids: Vec<String>,
    description: String,
    #[serde(default)]
    advanced: bool,
    #[serde(default)]
    unverified: bool,
}

fn invalid(reason: String) -> Error {
    Error::Parse { line: 0, reason }
}

impl FieldGuide {
    /// The guide for units, heroes and buildings.
    pub fn units() -> Result<Self> {
        Self::parse(UNIT_GUIDE).map_err(|e| e.in_file(UNIT_GUIDE_FILE))
    }

    /// The guide for abilities' generic fields.
    pub fn abilities() -> Result<Self> {
        Self::parse(ABILITY_GUIDE).map_err(|e| e.in_file(ABILITY_GUIDE_FILE))
    }

    /// The guide for items.
    pub fn items() -> Result<Self> {
        Self::parse(ITEM_GUIDE).map_err(|e| e.in_file(ITEM_GUIDE_FILE))
    }

    /// The guide for upgrades.
    pub fn upgrades() -> Result<Self> {
        Self::parse(UPGRADE_GUIDE).map_err(|e| e.in_file(UPGRADE_GUIDE_FILE))
    }

    /// The guide for buffs and effects.
    pub fn buffs() -> Result<Self> {
        Self::parse(BUFF_GUIDE).map_err(|e| e.in_file(BUFF_GUIDE_FILE))
    }

    /// The guide for destructables.
    pub fn destructables() -> Result<Self> {
        Self::parse(DESTRUCTABLE_GUIDE).map_err(|e| e.in_file(DESTRUCTABLE_GUIDE_FILE))
    }

    /// The guide for doodads.
    pub fn doodads() -> Result<Self> {
        Self::parse(DOODAD_GUIDE).map_err(|e| e.in_file(DOODAD_GUIDE_FILE))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let file: GuideFile = toml::from_str(text).map_err(|e| Error::Parse {
            line: 0,
            reason: e.to_string(),
        })?;

        let mut entries = HashMap::new();

        for field in file.field {
            let entry = GuideEntry {
                description: field.description,
                advanced: field.advanced,
                unverified: field.unverified,
            };

            for id in field.ids {
                if entries.insert(id.clone(), entry.clone()).is_some() {
                    return Err(invalid(format!("field {id} is described twice")));
                }
            }
        }

        let mut ranks = HashMap::new();

        for (category, ids) in &file.order {
            for (rank, id) in ids.iter().enumerate() {
                if !entries.contains_key(id) {
                    return Err(invalid(format!(
                        "field {id} is ordered but has no description"
                    )));
                }

                if ranks.insert(id.clone(), (category.clone(), rank)).is_some() {
                    return Err(invalid(format!("field {id} is ordered twice")));
                }
            }
        }

        Ok(Self { entries, ranks })
    }

    pub fn entry(&self, field_id: &str) -> Option<&GuideEntry> {
        self.entries.get(field_id)
    }

    /// Position among the fields shown first in its category, if it is one of them.
    pub fn rank(&self, field_id: &str) -> Option<usize> {
        self.ranks.get(field_id).map(|(_, rank)| *rank)
    }

    /// Every described field ID.
    pub fn field_ids(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// Every field shown first, with the category it is listed under.
    pub fn ordered(&self) -> impl Iterator<Item = (&str, &str)> {
        self.ranks
            .iter()
            .map(|(id, (category, _))| (id.as_str(), category.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unit_guide_is_valid() {
        let guide = FieldGuide::units().unwrap();

        assert_eq!(guide.rank("uhpm"), Some(0));
        assert!(!guide.entry("uhpm").unwrap().advanced);
        assert!(guide.entry("uept").unwrap().advanced);
        assert_eq!(guide.entry("ua1b"), guide.entry("ua2b"));
    }

    #[test]
    fn the_ability_guide_is_valid() {
        let guide = FieldGuide::abilities().unwrap();

        assert_eq!(guide.rank("alev"), Some(0));
        assert!(guide.entry("aord").unwrap().advanced);
        assert_eq!(guide.entry("abpx"), guide.entry("abpy"));
    }

    #[test]
    fn rejects_duplicates_and_undescribed_order() {
        let twice = "[[field]]\nids = [\"a\"]\ndescription = \"x\"\n[[field]]\nids = [\"a\"]\ndescription = \"y\"\n";
        let undescribed = "[order]\nstats = [\"zzzz\"]\n";
        let unknown_key = "[[field]]\nids = [\"a\"]\ndescription = \"x\"\nadvnced = true\n";

        assert!(FieldGuide::parse(twice).is_err());
        assert!(FieldGuide::parse(undescribed).is_err());
        assert!(FieldGuide::parse(unknown_key).is_err());
    }
}
