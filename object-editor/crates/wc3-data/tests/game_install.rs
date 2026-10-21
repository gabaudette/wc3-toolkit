//! Checks against a real Warcraft III: Reforged install. Ignored by default
//! because CI has no game files; run with:
//!
//! `WC3_GAME_DIR="/path/to/Warcraft III" cargo test --workspace -- --ignored`
//!
//! Expected values come from the World Editor's Object Editor for the Paladin
//! (`Hpal`) with the base game data.

use std::path::PathBuf;

use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::guide::FieldGuide;
use wc3_data::icon::load_texture;
use wc3_data::meta::ObjectKind;
use wc3_data::units::UnitDatabase;

const GAME_DIR_VAR: &str = "WC3_GAME_DIR";
const SD_ICON_SIZE: u32 = 64;
const HD_ICON_SIZE: u32 = 256;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn open(options: GameOptions) -> TestResult<(GameFiles, UnitDatabase)> {
    let dir = std::env::var_os(GAME_DIR_VAR)
        .map(PathBuf::from)
        .ok_or(format!(
            "set {GAME_DIR_VAR} to the Warcraft III install folder"
        ))?;
    let files = GameFiles::open(&dir, options)?;
    let db = UnitDatabase::load(&files)?;
    Ok((files, db))
}

fn value(db: &UnitDatabase, id: &str, field_id: &str, mode: ArtMode) -> TestResult<String> {
    let field = db.field(field_id).ok_or(format!("no field {field_id}"))?;
    Ok(db.display_value(id, field, mode))
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn paladin_matches_the_world_editor() -> TestResult {
    let (_, db) = open(GameOptions::default())?;
    let expected = [
        ("unam", "Paladin"),
        (
            "uhab",
            "Holy Light, Divine Shield, Resurrection, Devotion Aura",
        ),
        ("uabi", "Inventory (Hero)"),
        ("ucbs", "1.670"),
        ("ucpt", "0.500"),
        ("uwal", "250.000"),
        ("ubpx", "2"),
        ("ussc", "1.250"),
        ("ushw", "170.000"),
        ("uver", "None"),
        ("uarm", "Metal"),
        ("ua1c", "2.000"),
        ("ua1d", "2"),
        ("ua1s", "6"),
        ("ua1r", "100"),
        ("ua1g", "Ground, Structure, Debris, Item, Ward"),
        ("urac", "Human"),
        ("ustr", "22"),
    ];

    assert_eq!(db.kind("Hpal"), ObjectKind::Hero);
    for (field_id, want) in expected {
        assert_eq!(
            value(&db, "Hpal", field_id, ArtMode::Sd)?,
            want,
            "field {field_id}"
        );
    }

    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn hd_mode_uses_hd_values() -> TestResult {
    let (_, db) = open(GameOptions::default())?;

    assert_eq!(value(&db, "Hpal", "uwal", ArtMode::Hd)?, "290.000");

    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn balance_layer_overrides_base_data() -> TestResult {
    let options = GameOptions {
        balance: Some("custom_v1".to_owned()),
        ..GameOptions::default()
    };
    let (_, db) = open(options)?;

    assert_eq!(value(&db, "Hpal", "ua1c", ArtMode::Sd)?, "2.200");

    Ok(())
}

/// Every icon the data refers to loads in both modes. A few campaign units
/// (`Hleo`, `Scth`) have no icon in the game data, and units without an HD icon
/// fall back to the 64 px classic one.
#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn every_unit_icon_loads() -> TestResult {
    let (files, db) = open(GameOptions::default())?;
    let mut failures = Vec::new();
    let mut loaded = 0;

    for id in db.ids() {
        for mode in [ArtMode::Sd, ArtMode::Hd] {
            let Some(path) = db.icon_path(id, mode) else {
                continue;
            };

            match load_texture(&files, &path, mode) {
                Ok(image) if [SD_ICON_SIZE, HD_ICON_SIZE].contains(&image.width) => loaded += 1,
                Ok(image) => failures.push(format!("{id} {mode:?}: {}px", image.width)),
                Err(e) => failures.push(format!("{id} {mode:?}: {e}")),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(loaded > db.ids().len(), "only {loaded} icons loaded");

    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn every_field_of_every_unit_displays() -> TestResult {
    let (_, db) = open(GameOptions::default())?;
    let mut count = 0;

    for id in db.ids() {
        for field in db.fields_for(id) {
            db.display_value(id, field, ArtMode::Sd);
            count += 1;
        }
    }

    assert!(count > db.ids().len() * 100, "only {count} values");

    Ok(())
}

/// The field guide describes every unit field and nothing else, and lists
/// each ordered field under its real category.
#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn the_field_guide_covers_every_unit_field() -> TestResult {
    let (_, db) = open(GameOptions::default())?;
    let guide = FieldGuide::units()?;

    let missing: Vec<&str> = db
        .fields()
        .iter()
        .map(|f| f.id.as_str())
        .filter(|id| guide.entry(id).is_none())
        .collect();
    let unknown: Vec<&str> = guide
        .field_ids()
        .filter(|id| db.field(id).is_none())
        .collect();
    let misplaced: Vec<String> = guide
        .ordered()
        .filter_map(|(id, category)| {
            let field = db.field(id)?;
            (field.category != category)
                .then(|| format!("{id} is {} not {category}", field.category))
        })
        .collect();

    assert!(missing.is_empty(), "no description: {missing:?}");
    assert!(unknown.is_empty(), "not unit fields: {unknown:?}");
    assert!(misplaced.is_empty(), "{misplaced:?}");
    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn holy_light_card_data() -> TestResult {
    let dir = std::env::var_os(GAME_DIR_VAR)
        .map(PathBuf::from)
        .ok_or("set WC3_GAME_DIR")?;
    let files = GameFiles::open(&dir, GameOptions::default())?;
    let abilities = wc3_data::abilities::AbilityDatabase::load(&files)?;
    let info = abilities.info("AHhb").ok_or("no AHhb")?;

    assert_eq!(info.name, "Holy Light");
    assert!(info.hero);
    assert_eq!(info.levels, 3);
    assert!(
        info.icon
            .as_deref()
            .is_some_and(|i| i.contains("BTNHolyBolt"))
    );
    let none = wc3_data::abilities::NoPlaceholders;
    assert!(
        abilities
            .description("AHhb", 2, &none)
            .is_some_and(|d| d.contains("400") && !d.contains('<'))
    );

    let rows = abilities.level_values("AHhb");
    let row = |label: &str| {
        rows.iter()
            .find(|r| r.label == label)
            .map(|r| r.values.join("/"))
    };
    assert_eq!(row("Mana Cost").as_deref(), Some("65/65/65"));
    assert_eq!(row("Cast Range").as_deref(), Some("800/800/800"));
    assert_eq!(row("Amount Healed/Damaged").as_deref(), Some("200/400/600"));
    Ok(())
}

/// Every placeholder in every ability tooltip resolves to a value.
#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn ability_tooltip_placeholders_resolve() -> TestResult {
    let dir = std::env::var_os(GAME_DIR_VAR)
        .map(PathBuf::from)
        .ok_or("set WC3_GAME_DIR")?;
    let files = GameFiles::open(&dir, GameOptions::default())?;
    let abilities = wc3_data::abilities::AbilityDatabase::load(&files)?;
    let units = UnitDatabase::load(&files)?;
    let mut unresolved = Vec::new();

    for id in abilities.ids() {
        let levels = abilities.info(id).map_or(1, |i| i.levels);
        let texts = (1..=levels)
            .filter_map(|level| abilities.description(id, level, &units))
            .chain(abilities.learn_description(id, &units));
        for text in texts {
            let mut rest = text.as_str();
            while let Some(start) = rest.find('<') {
                let Some(end) = rest[start..].find('>') else {
                    break;
                };
                unresolved.push(format!("{id}: {}", &rest[start..start + end + 1]));
                rest = &rest[start + end + 1..];
            }
        }
    }

    unresolved.sort();
    unresolved.dedup();
    assert!(
        unresolved.is_empty(),
        "{} unresolved, e.g. {:?}",
        unresolved.len(),
        unresolved
            .iter()
            .map(|u| u.split(',').nth(1).unwrap_or_default().to_owned())
            .collect::<std::collections::BTreeSet<_>>()
    );
    Ok(())
}
