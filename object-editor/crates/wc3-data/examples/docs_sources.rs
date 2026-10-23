//! Export local documentation inputs. Never commit the exported game scripts.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::{GameFiles, GameOptions};
use wc3_data::meta::FieldMeta;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;

#[derive(Serialize)]
struct Field {
    id: String,
    name: String,
    value_type: String,
    min: Option<String>,
    max: Option<String>,
    per_level: bool,
    cosmetic: bool,
    applies_to: Vec<String>,
    excludes: Vec<String>,
    choices: Vec<String>,
}

fn describe(meta: &FieldMeta, name: String, units: &UnitDatabase) -> Field {
    let choices = units
        .enum_values(&meta.value_type)
        .unwrap_or_default()
        .iter()
        .map(|choice| format!("{}: {}", choice.value, choice.label))
        .collect();

    Field {
        id: meta.id.clone(),
        name,
        value_type: meta.value_type.clone(),
        min: meta.min.clone(),
        max: meta.max.clone(),
        per_level: meta.per_level,
        cosmetic: meta.netsafe,
        applies_to: meta.use_specific.clone(),
        excludes: meta.not_specific.clone(),
        choices,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let game = PathBuf::from(
        args.next()
            .ok_or("usage: docs_sources GAME_DIR OUTPUT_DIR")?,
    );
    let output = PathBuf::from(args.next().ok_or("missing OUTPUT_DIR")?);
    if args.next().is_some() {
        return Err("usage: docs_sources GAME_DIR OUTPUT_DIR".into());
    }

    let files = GameFiles::open(&game, GameOptions::default())?;
    std::fs::create_dir_all(&output)?;
    for script in ["common.j", "blizzard.j"] {
        let bytes = files.read_base(&format!("scripts\\{script}"))?;
        std::fs::write(output.join(script), bytes)?;
    }

    let units = UnitDatabase::load(&files)?;
    let items = UnitDatabase::load_items(&files)?;
    let buffs = UnitDatabase::load_buffs(&files)?;
    let destructables = UnitDatabase::load_destructables(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let upgrades = UpgradeDatabase::load(&files)?;
    let mut catalogs = BTreeMap::new();

    for (kind, database) in [
        ("unit", &units),
        ("item", &items),
        ("buff", &buffs),
        ("destructable", &destructables),
    ] {
        let fields: Vec<_> = database
            .fields()
            .iter()
            .map(|meta| describe(meta, database.field_name(meta), &units))
            .collect();
        catalogs.insert(kind, fields);
    }

    catalogs.insert(
        "ability",
        abilities
            .fields()
            .iter()
            .map(|meta| describe(meta, abilities.field_name(meta), &units))
            .collect(),
    );
    catalogs.insert(
        "upgrade",
        upgrades
            .fields()
            .iter()
            .map(|meta| describe(meta, upgrades.field_name(meta), &units))
            .collect(),
    );

    std::fs::write(output.join("fields.toml"), toml::to_string(&catalogs)?)?;
    Ok(())
}
