//! Spell kinds for the spell wizard: what a spell does, and the game abilities
//! that make the best base for it, with the reason for each. The list is
//! `data/spell-recipes.toml`, written for this project.

use serde::Deserialize;

use crate::{Error, Result};

const RECIPES: &str = include_str!("../data/spell-recipes.toml");
const RECIPES_FILE: &str = "spell-recipes.toml";

/// One choice on the wizard's first step, such as "Damage one target".
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellKind {
    pub id: String,
    pub label: String,
    pub description: String,
    /// Suggested base abilities, best first.
    #[serde(rename = "base")]
    pub bases: Vec<SuggestedBase>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedBase {
    /// The game ability's ID, e.g. `AHtb`.
    pub code: String,
    /// What it brings, and when to pick it over the others.
    pub why: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    kind: Vec<SpellKind>,
}

/// The spell kinds, in the order the wizard shows them.
pub fn spell_kinds() -> Result<Vec<SpellKind>> {
    parse(RECIPES).map_err(|e| e.in_file(RECIPES_FILE))
}

fn parse(text: &str) -> Result<Vec<SpellKind>> {
    let file: RecipeFile = toml::from_str(text).map_err(|e| Error::Parse {
        line: 0,
        reason: e.to_string(),
    })?;
    Ok(file.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_label_and_suggested_bases() {
        let kinds = spell_kinds().unwrap();
        assert!(kinds.len() >= 10, "only {} kinds", kinds.len());

        let mut ids: Vec<&str> = kinds.iter().map(|k| k.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), kinds.len(), "kind IDs repeat");

        for kind in &kinds {
            assert!(
                !kind.label.is_empty() && !kind.description.is_empty(),
                "{}",
                kind.id
            );
            assert!(!kind.bases.is_empty(), "{} suggests no base", kind.id);
            for base in &kind.bases {
                assert_eq!(base.code.len(), 4, "{}: {}", kind.id, base.code);
                assert!(!base.why.is_empty(), "{}: {}", kind.id, base.code);
            }
        }
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(
            parse("[[kind]]\nid = \"x\"\nlabel = \"X\"\ndescription = \"x\"\nbase = []\nextra = 1")
                .is_err()
        );
    }
}
