//! Checks against real maps and a real Warcraft III install. Ignored by default
//! because CI has neither; run with:
//!
//! `WC3_MAPS_DIR="/path/to/Maps" WC3_GAME_DIR="/path/to/Warcraft III" cargo test -- --ignored`

use std::path::{Path, PathBuf};

use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::units::UnitDatabase;
use wc3_map::abilities::{MapAbilities, MapPlaceholders};
use wc3_map::imports::MapImports;
use wc3_map::transfer::{self, Bundle, MapExtras, MapObjects, ObjectType, References};
use wc3_map::units::MapUnits;
use wc3_map::{
    ABILITIES_FILE, ABILITY_SKINS_FILE, FourCC, Layout, Map, OBJECT_FILES, ObjectData,
    STRINGS_FILE, UNIT_SKINS_FILE, UNITS_FILE, save_files,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MAPS_DIR_VAR: &str = "WC3_MAPS_DIR";
const GAME_DIR_VAR: &str = "WC3_GAME_DIR";
const MAP_EXTENSIONS: &[&str] = &["w3m", "w3x"];
const MAX_FILES: u32 = 8;
/// Unit, ability and buff files with their skins, and the string table.
const MOVED_MAX_FILES: u32 = 16;

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

    // A text change that is reset again leaves no string behind.
    let untouched = units.files();
    let tooltip = db.field("utub").ok_or("no utub")?;
    units.set(&db, paladin, tooltip, "|cffffcc00New|r tooltip")?;
    assert_ne!(units.files(), untouched);
    units.reset(paladin, tooltip);
    assert_eq!(
        units.files(),
        untouched,
        "reset left a string or entry behind"
    );

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

fn level_and_column(
    map: &Map,
    file: &str,
    id: FourCC,
    field: &str,
) -> Result<Vec<(u32, u32)>, Box<dyn std::error::Error>> {
    let data = map.object_data(file)?.ok_or(format!("{file} missing"))?;
    let entry = data.find(id).ok_or(format!("{id} missing from {file}"))?;
    Ok(entry
        .mods()
        .filter(|m| m.field.to_string() == field)
        .map(|m| (m.level.unwrap_or_default(), m.column.unwrap_or_default()))
        .collect())
}

/// Ability changes go to war3map.w3a with their level and data column, text
/// to war3mapSkin.w3a and the string table, and everything reads back.
#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn edited_abilities_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = AbilityDatabase::load(&files)?;
    let units_db = UnitDatabase::load(&files)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("map.w3m");
    mpq::Archive::create(&path, MAX_FILES)?.save()?;

    let map = Map::open(&path)?;
    let mut units = MapUnits::load(&map)?;
    let mut abilities = MapAbilities::load(&map)?;
    drop(map);
    let holy_light = FourCC::parse("AHhb").ok_or("bad id")?;
    let field = |id: &str| db.field(id).ok_or(format!("no {id}"));
    let (amount, cooldown, levels, name, tooltip) = (
        field("Hhb1")?,
        field("acdn")?,
        field("alev")?,
        field("anam")?,
        field("atp1")?,
    );

    // A text change that is reset again leaves no string behind.
    let untouched = (units.files(), abilities.files());
    abilities.set(
        &db,
        units.strings_mut(),
        holy_light,
        tooltip,
        2,
        "|cffffcc00New|r",
    )?;
    assert_ne!((units.files(), abilities.files()), untouched);
    abilities.reset(units.strings_mut(), holy_light, tooltip, 2);
    assert_eq!(
        (units.files(), abilities.files()),
        untouched,
        "reset left a string or entry behind"
    );

    let custom = abilities.create(&db, holy_light)?;
    abilities.set(&db, units.strings_mut(), custom, amount, 2, "500")?;
    abilities.set(&db, units.strings_mut(), custom, cooldown, 3, "2.5")?;
    abilities.set(&db, units.strings_mut(), custom, levels, 0, "4")?;
    abilities.set(&db, units.strings_mut(), custom, name, 0, "Holier Light")?;
    let mut written = units.files();
    written.extend(abilities.files());
    save_files(&path, &path, &written)?;

    let map = Map::open(&path)?;
    let units = MapUnits::load(&map)?;
    let mut reloaded = MapAbilities::load(&map)?;
    let strings = units.strings();
    let raw = |abilities: &MapAbilities, field, level| {
        abilities.raw_value(&db, strings, custom, field, level, ArtMode::Sd)
    };
    assert_eq!(custom.to_string(), "A000");
    assert_eq!(raw(&reloaded, amount, 1).as_deref(), Some("200"));
    assert_eq!(raw(&reloaded, amount, 2).as_deref(), Some("500"));
    assert_eq!(raw(&reloaded, cooldown, 3).as_deref(), Some("2.5"));
    assert_eq!(reloaded.levels(&db, custom), 4);
    assert_eq!(reloaded.name(&db, strings, custom), "Holier Light");
    let info = reloaded
        .info(&db, strings, custom, ArtMode::Sd)
        .ok_or("no info")?;
    assert!(info.hero && info.code == "AHhb" && info.levels == 4);
    assert_eq!(
        level_and_column(&map, ABILITIES_FILE, custom, "Hhb1")?,
        [(2, 1)]
    );
    assert_eq!(
        level_and_column(&map, ABILITIES_FILE, custom, "alev")?,
        [(0, 0)]
    );
    assert_eq!(
        level_and_column(&map, ABILITY_SKINS_FILE, custom, "anam")?,
        [(0, 0)]
    );
    assert_eq!(map.strings()?.resolve("TRIGSTR_001"), "Holier Light");

    // Tooltips naming the ability show the map's values.
    let placeholders = MapPlaceholders {
        abilities: &reloaded,
        db: &db,
        strings,
        mode: ArtMode::Sd,
        others: &units_db,
    };
    assert_eq!(
        db.fill_placeholders("<A000,DataA2> and <A000,Cool3>", &placeholders),
        "500 and 2.5"
    );

    // Typing the base ability's value again is no change.
    let mut strings = units.strings().clone();
    reloaded.set(&db, &mut strings, custom, amount, 2, "400")?;
    assert!(!reloaded.is_modified(custom, amount, 2));
    assert!(reloaded.reset(&mut strings, custom, cooldown, 3));
    assert!(!reloaded.reset(&mut strings, custom, cooldown, 3));
    Ok(())
}

/// Every map's custom units, with what they use, survive a share code and an
/// import into a copy of the same map, where every ID is taken: all objects get
/// new IDs, references follow them, and the saved map reads back.
#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn custom_units_move_between_maps() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let refs = References::new(db.fields(), abilities.fields());
    let in_game = |ty: ObjectType, id: FourCC| match ty {
        ObjectType::Unit => db.contains(&id.to_string()),
        ObjectType::Ability => abilities.contains(&id.to_string()),
        ObjectType::Buff => false,
    };

    let mut maps = Vec::new();
    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    let dir = tempfile::tempdir()?;
    let (mut moved_maps, mut objects, mut files_moved) = (0, 0, 0);

    for path in &maps {
        let map = Map::open(path)?;
        let units = MapUnits::load(&map)?;
        let abilities = MapAbilities::load(&map)?;
        let extras = MapExtras::load(&map)?;
        let custom: Vec<FourCC> = units
            .units()
            .iter()
            .filter(|u| u.is_custom)
            .map(|u| u.id)
            .collect();
        if custom.is_empty() {
            continue;
        }

        let source = MapObjects {
            units: &units,
            abilities: &abilities,
            extras: &extras,
        };
        let export = transfer::export(source, &refs, &custom, &|path| {
            map.read_file(path).ok().flatten()
        })?;
        let bundle = Bundle::from_file(&export.bundle.to_file())?;
        assert_eq!(
            bundle,
            export.bundle,
            "{}: the bundle file changes the bundle",
            path.display()
        );
        // Share codes carry the objects; imported files only travel in bundle files.
        let from_code = Bundle::from_code(&export.bundle.to_code())?;
        assert_eq!(
            from_code.objects(),
            bundle.objects(),
            "{}: share code",
            path.display()
        );
        assert!(from_code.files().is_empty());
        files_moved += bundle.files().len();

        let mut target_units = MapUnits::load(&map)?;
        let mut target_abilities = MapAbilities::load(&map)?;
        let mut target_extras = MapExtras::load(&map)?;
        let target = MapObjects {
            units: &target_units,
            abilities: &target_abilities,
            extras: &target_extras,
        };
        let plan = transfer::plan(&bundle, target, &in_game)?;
        assert!(
            plan.iter().all(|p| p.new_id != p.id),
            "{}: an ID that is taken was kept",
            path.display()
        );
        transfer::apply(
            &bundle,
            &plan,
            &refs,
            &mut target_units,
            &mut target_abilities,
            &mut target_extras,
        );

        // Protected maps cannot be rewritten, so the result goes into a new archive.
        let copy = dir.path().join(format!("moved{moved_maps}.w3x"));
        let max_files = MOVED_MAX_FILES + u32::try_from(bundle.files().len())?;
        mpq::Archive::create(&copy, max_files)?.save()?;
        let mut imports = MapImports::load(&Map::open(&copy)?)?;
        for file in bundle.files() {
            imports.add(file.clone());
        }
        let mut written = target_units.files();
        written.extend(target_abilities.files());
        written.extend(target_extras.files());
        let import_files = imports.files();
        written.extend(import_files.iter().map(|(p, b)| (p.as_str(), b.clone())));
        save_files(&copy, &copy, &written)?;

        let saved = Map::open(&copy)?;
        MapImports::load(&saved)?;
        for file in bundle.files() {
            assert_eq!(
                saved.read_file(&file.path)?.as_deref(),
                Some(file.bytes.as_slice()),
                "{}: imported file {} did not come along",
                path.display(),
                file.path
            );
        }
        let reread = MapUnits::load(&saved)?;
        let reread_abilities = MapAbilities::load(&saved)?;
        MapExtras::load(&saved)?;
        for planned in plan.iter().filter(|p| p.ty == ObjectType::Ability) {
            assert!(
                reread_abilities
                    .abilities()
                    .iter()
                    .any(|a| a.id == planned.new_id && a.is_custom),
                "{}: ability {} is missing after saving",
                path.display(),
                planned.new_id
            );
        }
        let old_ids: Vec<FourCC> = plan
            .iter()
            .filter(|p| p.ty == ObjectType::Unit)
            .map(|p| p.id)
            .collect();
        for planned in plan.iter().filter(|p| p.ty == ObjectType::Unit) {
            assert!(
                reread
                    .units()
                    .iter()
                    .any(|u| u.id == planned.new_id && u.is_custom),
                "{}: {} is missing after saving",
                path.display(),
                planned.new_id
            );
            // Lists of units an imported unit trains or builds name only new IDs.
            for field in ["utra", "ubui", "uupt"].iter().filter_map(|f| db.field(f)) {
                let value = reread
                    .raw_value(&db, planned.new_id, field, ArtMode::Sd)
                    .unwrap_or_default();
                let stale = value.split(',').filter_map(FourCC::parse).find(|id| {
                    old_ids.contains(id)
                        && !reread
                            .units()
                            .iter()
                            .any(|u| u.id == *id && !plan.iter().any(|p| p.id == *id))
                });
                assert!(
                    stale.is_none(),
                    "{}: {} still names {stale:?}",
                    path.display(),
                    planned.new_id
                );
            }
        }

        moved_maps += 1;
        objects += plan.len();
    }

    assert!(moved_maps > 0 && objects > 0, "no map has custom units");
    assert!(files_moved > 0, "no custom unit uses an imported file");
    Ok(())
}
