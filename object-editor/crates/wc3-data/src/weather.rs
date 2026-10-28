//! Weather effects a map can use (Scenario > Map Options), from `Weather.slk`.

use crate::Result;
use crate::game::{GameFiles, decode_text};
use crate::profile::Profile;
use crate::slk::Slk;
use crate::units::resolve_string;

const TABLE: &str = "terrainart\\weather.slk";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];
const ID_COLUMN: &str = "effectID";
const NAME_COLUMN: &str = "name";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherEffect {
    pub id: String,
    pub name: String,
}

pub fn load(files: &GameFiles) -> Result<Vec<WeatherEffect>> {
    let table =
        Slk::parse(&decode_text(files.read_base(TABLE)?, TABLE)?).map_err(|e| e.in_file(TABLE))?;
    let mut strings = Profile::new();

    for path in EDITOR_STRINGS {
        strings.merge_text(&decode_text(files.read_localized(path)?, path)?);
    }

    let mut effects: Vec<WeatherEffect> = table
        .records()
        .filter_map(|record| {
            let id = record.get(ID_COLUMN)?.to_string();
            let key = record.get(NAME_COLUMN).copied().unwrap_or_default();

            Some(WeatherEffect {
                name: resolve_string(&strings, key).unwrap_or_else(|| id.clone()),
                id,
            })
        })
        .collect();

    effects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(effects)
}
