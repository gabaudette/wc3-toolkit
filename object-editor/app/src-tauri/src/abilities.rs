//! Abilities shaped for the UI: picker entries and cards.

use serde::Serialize;
use wc3_data::abilities::{AbilityDatabase, AbilityInfo};
use wc3_data::units::UnitDatabase;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilitySummary {
    pub id: String,
    pub name: String,
    pub suffix: Option<String>,
    pub icon: Option<String>,
    pub hero: bool,
    pub item: bool,
    pub race: String,
    pub levels: u32,
}

#[derive(Serialize)]
pub struct LevelRow {
    pub label: String,
    pub values: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityCard {
    pub summary: AbilitySummary,
    /// Whether the game data has this ability. Map-made abilities are not
    /// loaded yet, so their cards only show the code.
    pub known: bool,
    pub hotkey: Option<String>,
    pub required_level: u32,
    /// The text a player reads: the "learn" tooltip for hero abilities (it
    /// covers every level), the level 1 tooltip otherwise. May contain color codes.
    pub description: Option<String>,
    pub rows: Vec<LevelRow>,
}

fn summary(info: AbilityInfo) -> AbilitySummary {
    AbilitySummary {
        id: info.id,
        name: info.name,
        suffix: info.suffix,
        icon: info.icon,
        hero: info.hero,
        item: info.item,
        race: info.race,
        levels: info.levels,
    }
}

pub fn list(abilities: &AbilityDatabase) -> Vec<AbilitySummary> {
    abilities
        .ids()
        .iter()
        .filter_map(|id| abilities.info(id))
        .map(summary)
        .collect()
}

pub fn card(abilities: &AbilityDatabase, units: &UnitDatabase, id: &str) -> AbilityCard {
    let Some(info) = abilities.info(id) else {
        return AbilityCard {
            summary: AbilitySummary {
                id: id.to_owned(),
                name: id.to_owned(),
                suffix: None,
                icon: None,
                hero: false,
                item: false,
                race: String::new(),
                levels: 1,
            },
            known: false,
            hotkey: None,
            required_level: 0,
            description: None,
            rows: Vec::new(),
        };
    };

    let description = if info.hero {
        abilities
            .learn_description(id, units)
            .or_else(|| abilities.description(id, 1, units))
    } else {
        abilities.description(id, 1, units)
    };

    AbilityCard {
        known: true,
        hotkey: info.hotkey.clone(),
        required_level: info.required_level,
        description,
        rows: abilities
            .level_values(id)
            .into_iter()
            .map(|row| LevelRow {
                label: row.label,
                values: row.values,
            })
            .collect(),
        summary: summary(info),
    }
}
