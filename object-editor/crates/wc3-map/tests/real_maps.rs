//! Checks against real maps and a real Warcraft III install. Ignored by default
//! because CI has neither; run with:
//!
//! `WC3_MAPS_DIR="/path/to/Maps" WC3_GAME_DIR="/path/to/Warcraft III" cargo test -- --ignored`

use std::path::{Path, PathBuf};

use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::units::UnitDatabase;
use wc3_map::units::MapUnits;
use wc3_map::{
    FourCC, Layout, Map, OBJECT_FILES, ObjectData, STRINGS_FILE, UNIT_SKINS_FILE, UNITS_FILE,
    save_files,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MAPS_DIR_VAR: &str = "WC3_MAPS_DIR";
const GAME_DIR_VAR: &str = "WC3_GAME_DIR";
const MAP_EXTENSIONS: &[&str] = &["w3m", "w3x"];
const MAX_FILES: u32 = 8;

fn env_dir(var: &str) -> Result<PathBuf, String> {
    std::env::var_os(var)
        .map(PathBuf::from)
        .ok_or(format!("set {var}"))
}

fn find_maps(dir: &Path, found: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            find_maps(&path, found)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| MAP_EXTENSIONS.iter().any(|m| m.eq_ignore_ascii_case(e)))
        {
            found.push(path);
        }
    }

    Ok(())
}

/// Every object data file and string table in every map rewrites byte for byte.
#[test]
#[ignore = "needs maps (WC3_MAPS_DIR)"]
fn every_map_round_trips() -> TestResult {
    let mut maps = Vec::new();
    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    let mut failures = Vec::new();
    let mut files = 0;

    for path in &maps {
        let map = Map::open(path)?;
        for name in OBJECT_FILES.iter().copied().chain([STRINGS_FILE]) {
            let Some(original) = map.read_file(name)? else {
                continue;
            };

            let rewritten = match Layout::for_file(name) {
                Some(layout) => ObjectData::parse(&original, layout).map(|d| d.to_bytes()),
                None => map
                    .strings()
                    .map(|s| s.to_text().into_bytes())
                    .map_err(|e| wc3_data::Error::Parse {
                        line: 0,
                        reason: e.to_string(),
                    }),
            };

            match rewritten {
                Ok(bytes) if bytes == original => files += 1,
                Ok(_) => failures.push(format!("{}: {name} differs", path.display())),
                Err(e) => failures.push(format!("{}: {name}: {e}", path.display())),
            }
        }
    }

    assert!(!maps.is_empty(), "no maps found");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(files > 0, "no object data in {} maps", maps.len());
    Ok(())
}

fn field_ids(map: &Map, file: &str, id: FourCC) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = map.object_data(file)?.ok_or(format!("{file} missing"))?;
    let entry = data.find(id).ok_or(format!("{id} missing from {file}"))?;
    Ok(entry.mods().map(|m| m.field.to_string()).collect())
}

/// Gameplay fields go to war3map.w3u, cosmetic ones to war3mapSkin.w3u with
/// their text in the string table, and everything reads back.
#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn edited_units_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("map.w3m");
    mpq::Archive::create(&path, MAX_FILES)?.save()?;

    let mut units = MapUnits::load(&Map::open(&path)?)?;
    let paladin = FourCC::parse("Hpal").ok_or("bad id")?;
    let hero = units.create(&db, paladin)?;
    let hp = db.field("uhpm").ok_or("no uhpm")?;
    let name = db.field("unam").ok_or("no unam")?;
    units.set(&db, hero, hp, "1000")?;
    units.set(&db, hero, name, "Holy Tank")?;
    save_files(&path, &path, &units.files())?;

    let map = Map::open(&path)?;
    let reloaded = MapUnits::load(&map)?;
    assert_eq!(hero.to_string(), "H000");
    assert_eq!(reloaded.display_value(&db, hero, hp, ArtMode::Sd), "1000");
    assert_eq!(reloaded.name(&db, hero), "Holy Tank");
    assert_eq!(field_ids(&map, UNITS_FILE, hero)?, ["uhpm"]);
    assert_eq!(field_ids(&map, UNIT_SKINS_FILE, hero)?, ["unam"]);
    assert_eq!(map.strings()?.resolve("TRIGSTR_001"), "Holy Tank");

    let mut edited = reloaded;
    let abilities = db.field("uhab").ok_or("no uhab")?;
    edited.set(&db, hero, abilities, "AHhb,AHds,AHre,AHad,AHtb")?;
    assert!(edited.is_modified(hero, abilities));
    edited.set(&db, hero, abilities, "AHhb,AHds,AHre,AHad")?;
    assert!(
        !edited.is_modified(hero, abilities),
        "setting the Paladin's own list is no change"
    );

    assert!(edited.reset(hero, hp));
    assert!(!edited.reset(hero, hp));
    assert_eq!(
        edited.display_value(&db, hero, hp, ArtMode::Sd),
        db.display_value("Hpal", hp, ArtMode::Sd)
    );
    Ok(())
}
