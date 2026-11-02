//! Checks against real maps and a real Warcraft III install. Ignored by default
//! because CI has neither; run with:
//!
//! `WC3_MAPS_DIR="/path/to/Maps" WC3_GAME_DIR="/path/to/Warcraft III" cargo test -- --ignored`

use std::path::{Path, PathBuf};

use wc3_data::abilities::AbilityDatabase;
use wc3_data::doodads::DoodadDatabase;
use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::icon::decode_texture;
use wc3_data::leveled::LeveledData;
use wc3_data::meta::FieldMeta;
use wc3_data::model::preview_copy;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::abilities::{MapAbilities, MapPlaceholders};
use wc3_map::imports::MapImports;
use wc3_map::info::{INFO_FILE, MapInfo};
use wc3_map::leveled::{DOODADS, LeveledObjects, UPGRADES};
use wc3_map::placements::{PLACEMENTS_FILE, UnitPlacements};
use wc3_map::plain::{BUFFS, DESTRUCTABLES, ITEMS, PlainKind, PlainObjects};
use wc3_map::transfer::{self, Bundle, MapObjects, MapObjectsMut, ObjectType, References};
use wc3_map::units::MapUnits;
use wc3_map::{
    ABILITIES_FILE, ABILITY_SKINS_FILE, BUFF_SKINS_FILE, BUFFS_FILE, DESTRUCTABLE_SKINS_FILE,
    DESTRUCTABLES_FILE, DOODADS_FILE, FourCC, ITEM_SKINS_FILE, ITEMS_FILE, Layout, Map,
    OBJECT_FILES, ObjectData, STRINGS_FILE, UNIT_SKINS_FILE, UNITS_FILE, UPGRADE_SKINS_FILE,
    UPGRADES_FILE, save_files,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MAPS_DIR_VAR: &str = "WC3_MAPS_DIR";
const GAME_DIR_VAR: &str = "WC3_GAME_DIR";
const MAP_EXTENSIONS: &[&str] = &["w3m", "w3x"];
const MAX_FILES: u32 = 8;
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

fn placement_failures(map: &Map, name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut failures = Vec::new();

    if let Some(original) = map.read_file(PLACEMENTS_FILE)? {
        match UnitPlacements::parse(&original) {
            Ok(units) if units.to_bytes() == original => {}
            Ok(_) => failures.push(format!("{name}: {PLACEMENTS_FILE} differs")),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }

    if let Some(original) = map.read_file(INFO_FILE)? {
        match MapInfo::parse(&original) {
            Ok(info)
                if info.to_bytes() == original
                    && info.supports_tables()
                    && info.properties().is_some() => {}
            Ok(_) => failures.push(format!("{name}: {INFO_FILE} differs or has no item tables")),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }

    Ok(failures)
}

#[test]
#[ignore = "needs maps (WC3_MAPS_DIR)"]
fn every_map_round_trips_placements_and_item_tables() -> TestResult {
    let mut maps = Vec::new();
    let mut failures = Vec::new();

    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    for path in &maps {
        failures.extend(placement_failures(
            &Map::open(path)?,
            &path.display().to_string(),
        )?);
    }

    assert!(!maps.is_empty(), "no maps found");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn every_game_map_round_trips_placements_and_item_tables() -> TestResult {
    let storage = casc::Storage::open(&env_dir(GAME_DIR_VAR)?)?;
    let dir = std::env::temp_dir().join("wc3-toolkit-game-maps");
    let mut failures = Vec::new();
    let mut maps = 0;

    std::fs::create_dir_all(&dir)?;
    for entry in storage.list("*.w3x")? {
        let Ok(bytes) = storage.read(&entry.name) else {
            continue;
        };
        let path = dir.join("map.w3x");

        std::fs::write(&path, bytes)?;
        let Ok(map) = Map::open(&path) else {
            continue;
        };

        maps += 1;
        failures.extend(placement_failures(&map, &entry.name)?);
    }

    std::fs::remove_dir_all(&dir)?;
    assert!(maps > 0, "no game maps found");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn field_ids(map: &Map, file: &str, id: FourCC) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = map.object_data(file)?.ok_or(format!("{file} missing"))?;
    let entry = data.find(id).ok_or(format!("{id} missing from {file}"))?;

    Ok(entry.mods().map(|m| m.field.to_string()).collect())
}

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

#[test]
#[ignore = "needs WC3_GAME_DIR"]
fn edited_items_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load_items(&files)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("map.w3m");
    mpq::Archive::create(&path, MAX_FILES)?.save()?;

    let map = Map::open(&path)?;
    let mut units = MapUnits::load(&map)?;
    let mut items = PlainObjects::load(&map, ITEMS)?;

    drop(map);
    let claws = FourCC::parse("ratf").ok_or("bad id")?;
    let field = |id: &str| db.field(id).ok_or(format!("no {id}"));
    let (gold, name, tooltip) = (field("igol")?, field("unam")?, field("utub")?);

    let untouched = (units.files(), items.files());

    items.set(&db, units.strings_mut(), claws, tooltip, "|cffffcc00New|r")?;
    assert_ne!((units.files(), items.files()), untouched);
    items.reset(units.strings_mut(), claws, tooltip);
    assert_eq!(
        (units.files(), items.files()),
        untouched,
        "reset left a string or entry behind"
    );

    let custom = items.create(&db, claws)?;

    items.set(&db, units.strings_mut(), custom, gold, "250")?;
    items.set(&db, units.strings_mut(), custom, name, "Claws of Testing")?;
    let mut written = units.files();

    written.extend(items.files());
    save_files(&path, &path, &written)?;

    let map = Map::open(&path)?;
    let reloaded = PlainObjects::load(&map, ITEMS)?;
    let strings = map.strings()?;

    assert_eq!(custom.to_string(), "I000");
    assert_eq!(reloaded.base_of(&db, custom), Some(claws));
    assert_eq!(
        reloaded
            .raw_value(&db, &strings, custom, gold, ArtMode::Sd)
            .as_deref(),
        Some("250")
    );
    assert_eq!(reloaded.name(&db, &strings, custom), "Claws of Testing");
    assert_eq!(field_ids(&map, ITEMS_FILE, custom)?, ["igol"]);
    assert_eq!(field_ids(&map, ITEM_SKINS_FILE, custom)?, ["unam"]);

    let mut edited = reloaded;
    let mut strings = strings;

    edited.set(&db, &mut strings, custom, gold, "800")?;
    assert!(
        !edited.is_modified(custom, gold),
        "setting the base's own value is no change"
    );
    assert!(edited.reset(&mut strings, custom, name));
    assert!(!edited.reset(&mut strings, custom, name));
    Ok(())
}

struct PlainCase {
    kind: PlainKind,
    database: fn(&GameFiles) -> wc3_data::Result<UnitDatabase>,
    base: &'static str,
    number: (&'static str, &'static str),
    name: &'static str,
    files: (&'static str, &'static str),
    first_id: &'static str,
}

#[test]
#[ignore = "needs WC3_GAME_DIR"]
fn edited_buffs_and_destructables_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let cases = [
        PlainCase {
            kind: BUFFS,
            database: UnitDatabase::load_buffs,
            base: "BHbd",
            number: ("feff", "1"),
            name: "fnam",
            files: (BUFFS_FILE, BUFF_SKINS_FILE),
            first_id: "B000",
        },
        PlainCase {
            kind: DESTRUCTABLES,
            database: UnitDatabase::load_destructables,
            base: "LTlt",
            number: ("bhps", "123"),
            name: "bnam",
            files: (DESTRUCTABLES_FILE, DESTRUCTABLE_SKINS_FILE),
            first_id: "B000",
        },
    ];

    for case in cases {
        let db = (case.database)(&files)?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3m");
        mpq::Archive::create(&path, MAX_FILES)?.save()?;

        let map = Map::open(&path)?;
        let mut units = MapUnits::load(&map)?;
        let mut objects = PlainObjects::load(&map, case.kind)?;

        drop(map);
        let base = FourCC::parse(case.base).ok_or("bad id")?;
        let (number, name) = (field(&db, case.number.0)?, field(&db, case.name)?);

        let custom = objects.create(&db, base)?;

        objects.set(&db, units.strings_mut(), custom, number, case.number.1)?;
        objects.set(&db, units.strings_mut(), custom, name, "Testing Name")?;
        let mut written = units.files();

        written.extend(objects.files());
        save_files(&path, &path, &written)?;

        let map = Map::open(&path)?;
        let reloaded = PlainObjects::load(&map, case.kind)?;
        let strings = map.strings()?;

        assert_eq!(custom.to_string(), case.first_id);
        assert_eq!(reloaded.base_of(&db, custom), Some(base));
        assert_eq!(
            reloaded
                .raw_value(&db, &strings, custom, number, ArtMode::Sd)
                .as_deref(),
            Some(case.number.1),
            "{}",
            case.base
        );
        assert_eq!(reloaded.name(&db, &strings, custom), "Testing Name");
        assert_eq!(field_ids(&map, case.files.0, custom)?, [case.number.0]);
        assert_eq!(field_ids(&map, case.files.1, custom)?, [case.name]);
    }

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

    let mut strings = units.strings().clone();

    reloaded.set(&db, &mut strings, custom, amount, 2, "400")?;
    assert!(!reloaded.is_modified(custom, amount, 2));
    assert!(reloaded.reset(&mut strings, custom, cooldown, 3));
    assert!(!reloaded.reset(&mut strings, custom, cooldown, 3));
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn custom_units_move_between_maps() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let items_db = UnitDatabase::load_items(&files)?;
    let upgrades_db = UpgradeDatabase::load(&files)?;
    let refs = References::new(
        db.fields(),
        abilities.fields(),
        items_db.fields(),
        upgrades_db.fields(),
    );
    let in_game = |ty: ObjectType, id: FourCC| match ty {
        ObjectType::Unit => db.contains(&id.to_string()),
        ObjectType::Ability => abilities.contains(&id.to_string()),
        ObjectType::Item => items_db.contains(&id.to_string()),
        ObjectType::Upgrade => upgrades_db.contains(&id.to_string()),
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
        let items = PlainObjects::load(&map, ITEMS)?;
        let upgrades = LeveledObjects::load(&map, UPGRADES)?;
        let buffs = PlainObjects::load(&map, BUFFS)?;
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
            items: &items,
            upgrades: &upgrades,
            buffs: &buffs,
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
        let mut target_items = PlainObjects::load(&map, ITEMS)?;
        let mut target_upgrades = LeveledObjects::load(&map, UPGRADES)?;
        let mut target_buffs = PlainObjects::load(&map, BUFFS)?;
        let target = MapObjects {
            units: &target_units,
            abilities: &target_abilities,
            items: &target_items,
            upgrades: &target_upgrades,
            buffs: &target_buffs,
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
            MapObjectsMut {
                units: &mut target_units,
                abilities: &mut target_abilities,
                items: &mut target_items,
                upgrades: &mut target_upgrades,
                buffs: &mut target_buffs,
            },
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
        written.extend(target_items.files());
        written.extend(target_upgrades.files());
        written.extend(target_buffs.files());
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
        let reread_items = PlainObjects::load(&saved, ITEMS)?;
        let reread_upgrades = LeveledObjects::load(&saved, UPGRADES)?;
        PlainObjects::load(&saved, BUFFS)?;
        for planned in plan.iter().filter(|p| p.ty == ObjectType::Upgrade) {
            assert!(
                reread_upgrades
                    .objects()
                    .iter()
                    .any(|u| u.id == planned.new_id && u.is_custom),
                "{}: upgrade {} is missing after saving",
                path.display(),
                planned.new_id
            );
        }

        for planned in plan.iter().filter(|p| p.ty == ObjectType::Item) {
            assert!(
                reread_items
                    .objects()
                    .iter()
                    .any(|i| i.id == planned.new_id && i.is_custom),
                "{}: item {} is missing after saving",
                path.display(),
                planned.new_id
            );
        }

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

fn field<'a>(db: &'a UnitDatabase, id: &str) -> Result<&'a FieldMeta, String> {
    db.field(id).ok_or(format!("no {id}"))
}

#[test]
#[ignore = "needs WC3_GAME_DIR"]
fn custom_items_and_upgrades_move_with_the_units_that_use_them() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let items_db = UnitDatabase::load_items(&files)?;
    let upgrades_db = UpgradeDatabase::load(&files)?;
    let refs = References::new(
        db.fields(),
        abilities.fields(),
        items_db.fields(),
        upgrades_db.fields(),
    );
    let dir = tempfile::tempdir()?;

    let source_path = dir.path().join("source.w3m");
    mpq::Archive::create(&source_path, MAX_FILES)?.save()?;
    let map = Map::open(&source_path)?;
    let (mut units, mut items) = (MapUnits::load(&map)?, PlainObjects::load(&map, ITEMS)?);
    let source_abilities = MapAbilities::load(&map)?;
    let mut upgrades = LeveledObjects::load(&map, UPGRADES)?;
    let buffs = PlainObjects::load(&map, BUFFS)?;
    let swords = FourCC::parse("Rhme").ok_or("bad id")?;
    let upgrade = upgrades.create(&upgrades_db, swords)?;
    let upgrade_name = upgrades_db.field("gnam").ok_or("no gnam")?;

    upgrades.set(
        &upgrades_db,
        units.strings_mut(),
        upgrade,
        upgrade_name,
        2,
        "Swords of Sharing",
    )?;
    let item = items.create(&items_db, FourCC::parse("ratf").ok_or("bad id")?)?;

    items.set(
        &items_db,
        units.strings_mut(),
        item,
        field(&items_db, "unam")?,
        "Claws of Sharing",
    )?;
    let shop = units.create(&db, FourCC::parse("ngme").ok_or("bad id")?)?;

    units.set(&db, shop, field(&db, "usei")?, &format!("{item},stwp"))?;
    units.set(&db, shop, field(&db, "ures")?, &upgrade.to_string())?;

    let source = MapObjects {
        units: &units,
        abilities: &source_abilities,
        items: &items,
        upgrades: &upgrades,
        buffs: &buffs,
    };
    let export = transfer::export(source, &refs, &[shop], &|_| None)?;

    assert!(export.warnings.is_empty(), "{:?}", export.warnings);
    let bundle = Bundle::from_code(&export.bundle.to_code())?;

    // The target already has its own I000 and R000.
    let target_path = dir.path().join("target.w3m");
    mpq::Archive::create(&target_path, MAX_FILES)?.save()?;
    let map = Map::open(&target_path)?;
    let mut target_units = MapUnits::load(&map)?;
    let mut target_abilities = MapAbilities::load(&map)?;
    let mut target_items = PlainObjects::load(&map, ITEMS)?;
    let mut target_upgrades = LeveledObjects::load(&map, UPGRADES)?;
    let mut target_buffs = PlainObjects::load(&map, BUFFS)?;

    drop(map);
    target_items.create(&items_db, FourCC::parse("phea").ok_or("bad id")?)?;
    target_upgrades.create(&upgrades_db, swords)?;
    let target = MapObjects {
        units: &target_units,
        abilities: &target_abilities,
        items: &target_items,
        upgrades: &target_upgrades,
        buffs: &target_buffs,
    };
    let in_game = |ty: ObjectType, id: FourCC| match ty {
        ObjectType::Unit => db.contains(&id.to_string()),
        ObjectType::Ability => abilities.contains(&id.to_string()),
        ObjectType::Item => items_db.contains(&id.to_string()),
        ObjectType::Upgrade => upgrades_db.contains(&id.to_string()),
        ObjectType::Buff => false,
    };
    let plan = transfer::plan(&bundle, target, &in_game)?;
    let new_item = plan
        .iter()
        .find(|p| p.ty == ObjectType::Item)
        .ok_or("the item did not come along")?
        .new_id;
    let new_shop = plan
        .iter()
        .find(|p| p.ty == ObjectType::Unit)
        .ok_or("the shop is missing")?
        .new_id;
    let new_upgrade = plan
        .iter()
        .find(|p| p.ty == ObjectType::Upgrade)
        .ok_or("the upgrade did not come along")?
        .new_id;

    assert_eq!(
        (new_item.to_string(), new_upgrade.to_string()),
        ("I001".into(), "R001".into())
    );
    transfer::apply(
        &bundle,
        &plan,
        &refs,
        MapObjectsMut {
            units: &mut target_units,
            abilities: &mut target_abilities,
            items: &mut target_items,
            upgrades: &mut target_upgrades,
            buffs: &mut target_buffs,
        },
    );
    let mut written = target_units.files();

    written.extend(target_items.files());
    written.extend(target_upgrades.files());
    save_files(&target_path, &target_path, &written)?;

    let saved = Map::open(&target_path)?;
    let (reread_units, reread_items) =
        (MapUnits::load(&saved)?, PlainObjects::load(&saved, ITEMS)?);
    let reread_upgrades = LeveledObjects::load(&saved, UPGRADES)?;
    let strings = saved.strings()?;

    assert_eq!(
        reread_units
            .raw_value(&db, new_shop, field(&db, "usei")?, ArtMode::Sd)
            .as_deref(),
        Some("I001,stwp")
    );
    assert_eq!(
        reread_items.name(&items_db, &strings, new_item),
        "Claws of Sharing"
    );
    assert_eq!(
        reread_items.base_of(&items_db, new_item),
        FourCC::parse("ratf")
    );
    assert_eq!(
        reread_units
            .raw_value(&db, new_shop, field(&db, "ures")?, ArtMode::Sd)
            .as_deref(),
        Some("R001")
    );
    assert_eq!(
        reread_upgrades
            .raw_value(
                &upgrades_db,
                &strings,
                new_upgrade,
                upgrade_name,
                2,
                ArtMode::Sd
            )
            .as_deref(),
        Some("Swords of Sharing")
    );
    Ok(())
}

#[test]
#[ignore = "needs WC3_GAME_DIR"]
fn edited_doodads_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = DoodadDatabase::load(&files)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("map.w3m");
    mpq::Archive::create(&path, MAX_FILES)?.save()?;

    let map = Map::open(&path)?;
    let mut units = MapUnits::load(&map)?;
    let mut doodads = LeveledObjects::load(&map, DOODADS)?;

    drop(map);
    let mushrooms = FourCC::parse("APms").ok_or("bad id")?;
    let field = |id: &str| db.field(id).ok_or(format!("no {id}"));
    let (red, scale) = (field("dvr1")?, field("ddes")?);

    let custom = doodads.create(&db, mushrooms)?;

    doodads.set(&db, units.strings_mut(), custom, red, 2, "100")?;
    doodads.set(&db, units.strings_mut(), custom, scale, 0, "1.5")?;
    let mut written = units.files();

    written.extend(doodads.files());
    save_files(&path, &path, &written)?;

    let map = Map::open(&path)?;
    let reloaded = LeveledObjects::load(&map, DOODADS)?;
    let strings = map.strings()?;

    assert_eq!(custom.to_string(), "D000");
    assert_eq!(reloaded.levels(&db, custom), 4);
    let value = |field, level| reloaded.raw_value(&db, &strings, custom, field, level, ArtMode::Sd);

    assert_eq!(value(red, 2).as_deref(), Some("100"));
    assert_eq!(value(red, 1).as_deref(), Some("255"));
    assert_eq!(value(scale, 0).as_deref(), Some("1.5"));
    assert_eq!(
        level_and_column(&map, DOODADS_FILE, custom, "dvr1")?,
        [(2, 0)]
    );
    assert_eq!(
        level_and_column(&map, DOODADS_FILE, custom, "ddes")?,
        [(0, 0)]
    );
    Ok(())
}

#[test]
#[ignore = "needs WC3_GAME_DIR"]
fn edited_upgrades_save_and_read_back() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UpgradeDatabase::load(&files)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("map.w3m");
    mpq::Archive::create(&path, MAX_FILES)?.save()?;

    let map = Map::open(&path)?;
    let mut units = MapUnits::load(&map)?;
    let mut upgrades = LeveledObjects::load(&map, UPGRADES)?;

    drop(map);
    let swords = FourCC::parse("Rhme").ok_or("bad id")?;
    let field = |id: &str| db.field(id).ok_or(format!("no {id}"));
    let (name, gold, levels, requirements) = (
        field("gnam")?,
        field("gglb")?,
        field("glvl")?,
        field("greq")?,
    );

    let untouched = (units.files(), upgrades.files());

    upgrades.set(&db, units.strings_mut(), swords, name, 2, "|cffffcc00New|r")?;
    assert_ne!((units.files(), upgrades.files()), untouched);
    upgrades.reset(units.strings_mut(), swords, name, 2);
    assert_eq!(
        (units.files(), upgrades.files()),
        untouched,
        "reset left a string or entry behind"
    );

    let custom = upgrades.create(&db, swords)?;

    upgrades.set(&db, units.strings_mut(), custom, gold, 0, "250")?;
    upgrades.set(&db, units.strings_mut(), custom, levels, 0, "4")?;
    upgrades.set(
        &db,
        units.strings_mut(),
        custom,
        name,
        4,
        "Adamantite Swords",
    )?;
    upgrades.set(&db, units.strings_mut(), custom, requirements, 4, "hcas")?;
    let mut written = units.files();

    written.extend(upgrades.files());
    save_files(&path, &path, &written)?;

    let map = Map::open(&path)?;
    let reloaded = LeveledObjects::load(&map, UPGRADES)?;
    let strings = map.strings()?;

    assert_eq!(custom.to_string(), "R000");
    assert_eq!(reloaded.levels(&db, custom), 4);
    let value = |field, level| reloaded.raw_value(&db, &strings, custom, field, level, ArtMode::Sd);

    assert_eq!(value(gold, 0).as_deref(), Some("250"));
    assert_eq!(value(name, 1).as_deref(), Some("Iron Forged Swords"));
    assert_eq!(value(name, 4).as_deref(), Some("Adamantite Swords"));
    assert_eq!(value(requirements, 4).as_deref(), Some("hcas"));
    assert_eq!(
        level_and_column(&map, UPGRADES_FILE, custom, "glvl")?,
        [(0, 0)]
    );
    assert_eq!(
        level_and_column(&map, UPGRADE_SKINS_FILE, custom, "gnam")?,
        [(4, 0)]
    );

    let mut edited = reloaded;
    let mut strings = strings;

    edited.set(&db, &mut strings, custom, gold, 0, "100")?;
    assert!(
        !edited.is_modified(custom, gold, 0),
        "setting the base's own value is no change"
    );
    assert!(edited.reset(&mut strings, custom, name, 4));
    assert!(!edited.reset(&mut strings, custom, name, 4));
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn imported_art_decodes() -> TestResult {
    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let items = UnitDatabase::load_items(&files)?;
    let upgrades_db = UpgradeDatabase::load(&files)?;
    let refs = References::new(
        db.fields(),
        abilities.fields(),
        items.fields(),
        upgrades_db.fields(),
    );

    let mut maps = Vec::new();

    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    let (mut models, mut textures) = (0, 0);
    let mut failures = Vec::new();

    for path in &maps {
        let map = Map::open(path)?;
        let units = MapUnits::load(&map)?;
        let custom: Vec<FourCC> = units
            .units()
            .iter()
            .filter(|u| u.is_custom)
            .map(|u| u.id)
            .collect();
        let source = MapObjects {
            units: &units,
            abilities: &MapAbilities::load(&map)?,
            items: &PlainObjects::load(&map, ITEMS)?,
            upgrades: &LeveledObjects::load(&map, UPGRADES)?,
            buffs: &PlainObjects::load(&map, BUFFS)?,
        };
        let export = transfer::export(source, &refs, &custom, &|file| {
            map.read_file(file).ok().flatten()
        })?;

        for file in export.bundle.files() {
            let lower = file.path.to_ascii_lowercase();
            let decoded = if lower.ends_with(".mdx") {
                models += 1;
                preview_copy(&file.bytes).map(drop)
            } else if lower.ends_with(".blp") || lower.ends_with(".dds") {
                textures += 1;
                decode_texture(&file.bytes).map(drop)
            } else {
                continue;
            };

            if let Err(e) = decoded {
                failures.push(format!("{}: {}: {e}", path.display(), file.path));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(models > 0 && textures > 0, "no map imports art");
    Ok(())
}

struct GameKinds<'a> {
    db: &'a UnitDatabase,
    items: &'a UnitDatabase,
    abilities: &'a AbilityDatabase,
    units: MapUnits,
}

impl wc3_map::script::UnitKinds for GameKinds<'_> {
    fn is_building(&self, id: FourCC) -> bool {
        self.units.kind(self.db, id) == Some(wc3_data::meta::ObjectKind::Building)
    }

    fn is_hero(&self, id: FourCC) -> bool {
        self.units.kind(self.db, id) == Some(wc3_data::meta::ObjectKind::Hero)
    }

    fn is_item(&self, id: FourCC) -> bool {
        self.items.contains(&id.to_string())
            || (!self.db.contains(&id.to_string()) && self.units.base_of(self.db, id).is_none())
    }

    fn is_gold_mine(&self, id: FourCC) -> bool {
        let Some(field) = self.db.field("uabi") else {
            return false;
        };
        let base = self.units.base_of(self.db, id).unwrap_or(id);

        self.units
            .raw_value(self.db, id, field, ArtMode::Sd)
            .or_else(|| self.db.raw_value(&base.to_string(), field, ArtMode::Sd))
            .is_some_and(|abilities| abilities.split(',').any(|a| a.trim() == "Agld"))
    }

    fn team_color(&self, id: FourCC) -> Option<i32> {
        let field = self.db.field("utco")?;
        let base = self.units.base_of(self.db, id).unwrap_or(id);

        self.units
            .raw_value(self.db, id, field, ArtMode::Sd)
            .or_else(|| self.db.raw_value(&base.to_string(), field, ArtMode::Sd))
            .and_then(|v| v.trim().parse().ok())
            .filter(|c: &i32| *c >= 0)
    }

    fn autocast_orders(&self, ability: FourCC) -> Option<(String, String)> {
        let code = ability.to_string();
        let order = |name: &str| {
            let field = self
                .abilities
                .fields()
                .iter()
                .find(|f| f.field.eq_ignore_ascii_case(name))?;

            self.abilities
                .raw_value(&code, field, 1, ArtMode::Sd)
                .filter(|order| !order.is_empty())
        };

        Some((order("Orderon")?, order("Orderoff")?))
    }
}

const PERCENT: usize = 100;
const UNIT_SCRIPT_MATCH: usize = 90;
const SETUP_SCRIPT_MATCH: usize = 98;

fn script_maps() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut maps = Vec::new();

    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    let storage = casc::Storage::open(&env_dir(GAME_DIR_VAR)?)?;
    let game_maps = std::env::temp_dir().join("wc3-toolkit-script-maps");

    std::fs::create_dir_all(&game_maps)?;
    for (n, entry) in storage.list("*.w3x")?.iter().enumerate() {
        if let Ok(bytes) = storage.read(&entry.name) {
            let path = game_maps.join(format!("{n}.w3x"));

            std::fs::write(&path, bytes)?;
            maps.push(path);
        }
    }

    Ok(maps)
}

fn map_script(
    map: &Map,
) -> Result<Option<(wc3_map::script::Language, String)>, Box<dyn std::error::Error>> {
    use wc3_map::script::{self, Language};

    let jass = match map.read_file(script::JASS_FILES[0])? {
        Some(text) => Some(text),
        None => map.read_file(script::JASS_FILES[1])?,
    };
    let found = match (map.read_file(script::LUA_FILE)?, jass) {
        (Some(lua), _) => Some((Language::Lua, lua)),
        (None, Some(jass)) => Some((Language::Jass, jass)),
        _ => None,
    };

    Ok(found.map(|(language, bytes)| {
        (
            language,
            String::from_utf8_lossy(&bytes).replace("\r\n", "\n"),
        )
    }))
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn generated_setup_matches_the_world_editor() -> TestResult {
    use wc3_map::info::MapInfo;
    use wc3_map::setup::{self, SetupSection};
    use wc3_map::terrain::Terrain;

    let report_path = std::env::var_os("WC3_SCRIPT_REPORT").map(PathBuf::from);
    let mut report = String::new();
    let (mut same, mut checked) = (0, 0);

    for path in &script_maps()? {
        let Ok(map) = Map::open(path) else {
            continue;
        };
        let Ok(Some((language, text))) = map_script(&map) else {
            continue;
        };
        let Ok(Some(info)) = MapInfo::load(&map) else {
            continue;
        };
        let Some(properties) = info.properties() else {
            continue;
        };
        let bounds = Terrain::load(&map).ok().flatten().map(|t| t.bounds());
        let section = SetupSection { properties, bounds };

        checked += 1;
        match setup::rewrite(language, &text, &section) {
            Ok(rewritten) if rewritten == text => same += 1,
            Ok(rewritten) => {
                let old: Vec<&str> = text.lines().collect();
                let new: Vec<&str> = rewritten.lines().collect();
                let at = old
                    .iter()
                    .zip(&new)
                    .position(|(a, b)| a != b)
                    .unwrap_or(old.len().min(new.len()));
                let window = |lines: &[&str]| {
                    lines[at.saturating_sub(2)..(at + 4).min(lines.len())].join("\n")
                };

                report.push_str(&format!(
                    "== {}\n-- world editor\n{}\n-- generated\n{}\n",
                    path.display(),
                    window(&old),
                    window(&new)
                ));
            }
            Err(e) => report.push_str(&format!("== {}\n!! {e}\n", path.display())),
        }
    }

    report.insert_str(0, &format!("{same}/{checked} maps identical\n"));
    if let Some(path) = report_path {
        std::fs::write(path, &report)?;
    }

    assert!(checked > 0, "no maps checked");
    assert!(
        same * PERCENT >= checked * SETUP_SCRIPT_MATCH,
        "{}",
        report.lines().next().unwrap_or_default()
    );
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn generated_unit_functions_match_the_world_editor() -> TestResult {
    use wc3_map::info::MapInfo;
    use wc3_map::script::{self, UnitSection};

    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let items = UnitDatabase::load_items(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let maps = script_maps()?;
    let report_path = std::env::var_os("WC3_SCRIPT_REPORT").map(PathBuf::from);
    let mut report = String::new();
    let (mut same, mut total, mut maps_same, mut maps_checked) = (0, 0, 0, 0);

    for path in &maps {
        let Ok(map) = Map::open(path) else {
            continue;
        };
        let Some((language, text)) = map_script(&map)? else {
            continue;
        };
        let Ok(Some(placements)) = UnitPlacements::load(&map) else {
            continue;
        };
        let Ok(units) = MapUnits::load(&map) else {
            continue;
        };
        let info = MapInfo::load(&map).ok().flatten();
        let tables = info
            .as_ref()
            .map(|i| i.tables().to_vec())
            .unwrap_or_default();
        let kinds = GameKinds {
            db: &db,
            items: &items,
            abilities: &abilities,
            units,
        };
        let section = UnitSection {
            placements: &placements,
            original: Some(&placements),
            tables: &tables,
            kinds: &kinds,
        };
        let Ok(generated) = script::unit_functions(language, &text, &section) else {
            continue;
        };
        let existing = script::existing_unit_functions(language, &text);
        let mut map_ok = existing.len() == generated.len();

        maps_checked += 1;
        for (name, original) in &existing {
            total += 1;
            match generated.iter().find(|(n, _)| n == name) {
                Some((_, text)) if text == original => same += 1,
                Some((_, text)) => {
                    map_ok = false;
                    if report.len() < 400_000 {
                        let line = original
                            .lines()
                            .zip(text.lines())
                            .position(|(a, b)| a != b)
                            .unwrap_or_else(|| original.lines().count().min(text.lines().count()));

                        report.push_str(&format!(
                            "== {} {name} line {line}\n-- world editor:\n{}\n-- generated:\n{}\n",
                            path.display(),
                            original
                                .lines()
                                .skip(line.saturating_sub(2))
                                .take(6)
                                .collect::<Vec<_>>()
                                .join("\n"),
                            text.lines()
                                .skip(line.saturating_sub(2))
                                .take(6)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ));
                    }
                }
                None => {
                    map_ok = false;
                    report.push_str(&format!("== {} {name} not generated\n", path.display()));
                }
            }
        }

        for (name, _) in &generated {
            if !existing.iter().any(|(n, _)| n == name) {
                map_ok = false;
                report.push_str(&format!(
                    "== {} {name} generated but not in the script\n",
                    path.display()
                ));
            }
        }

        if map_ok {
            maps_same += 1;
        }
    }

    report.insert_str(
        0,
        &format!("{same}/{total} functions, {maps_same}/{maps_checked} maps identical\n"),
    );
    if let Some(path) = report_path {
        std::fs::write(path, &report)?;
    }

    assert!(maps_checked > 0, "no maps with scripts");
    assert!(
        maps_same * PERCENT >= maps_checked * UNIT_SCRIPT_MATCH,
        "{}",
        report.lines().next().unwrap_or_default()
    );
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn edited_placements_rewrite_the_unit_script() -> TestResult {
    use std::collections::HashMap;

    use wc3_map::info::MapInfo;
    use wc3_map::placements::{DropChance, Drops};
    use wc3_map::script::{self, Language, UnitSection};

    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let items = UnitDatabase::load_items(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let mut maps = Vec::new();
    let mut checked = HashMap::new();

    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    for path in &maps {
        let map = Map::open(path)?;
        let (language, bytes) = match (
            map.read_file(script::LUA_FILE)?,
            map.read_file(script::JASS_FILES[0])?,
        ) {
            (Some(lua), _) => (Language::Lua, lua),
            (None, Some(jass)) => (Language::Jass, jass),
            _ => continue,
        };

        if checked
            .get(&(language == Language::Lua))
            .copied()
            .unwrap_or(0)
            >= 3
        {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
        let (Ok(Some(original)), Ok(units)) = (UnitPlacements::load(&map), MapUnits::load(&map))
        else {
            continue;
        };
        let tables = MapInfo::load(&map)?
            .map(|i| i.tables().to_vec())
            .unwrap_or_default();
        let kinds = GameKinds {
            db: &db,
            items: &items,
            abilities: &abilities,
            units,
        };
        let unchanged = UnitSection {
            placements: &original,
            original: Some(&original),
            tables: &tables,
            kinds: &kinds,
        };
        let Ok(regenerated) = script::unit_functions(language, &text, &unchanged) else {
            continue;
        };

        if regenerated != script::existing_unit_functions(language, &text) {
            continue;
        }

        let editable = |u: &wc3_map::placements::Placement| {
            u.id != FourCC(*b"sloc")
                && u.id.0[1..] != *b"DNR"
                && !items.contains(&u.id.to_string())
                && !text.contains(&format!("gg_unit_{}_{:04}", u.id, u.creation_number))
        };
        let Some(first) = original.units().iter().position(editable) else {
            continue;
        };
        let mut edited = original.clone();
        let moved_x = original.units()[first].x + 128.0;

        edited.unit_mut(first)?.x = moved_x;
        edited.unit_mut(first)?.life = 50;
        edited.set_drops(
            first,
            Drops::Sets(vec![vec![DropChance {
                item: FourCC(*b"phea"),
                chance: 100,
            }]]),
        )?;
        if let Some(last) = edited
            .units()
            .iter()
            .rposition(editable)
            .filter(|i| *i != first)
        {
            edited.remove(last)?;
        }

        edited.add(FourCC(*b"hfoo"), 0, 0.0, 0.0, 0.0);

        let section = UnitSection {
            placements: &edited,
            original: Some(&original),
            tables: &tables,
            kinds: &kinds,
        };
        let generated = script::unit_functions(language, &text, &section)?;
        let rewritten = script::splice_units(language, &text, &generated)?;
        let moved = format!("{moved_x:.1}");

        assert!(
            rewritten.contains(&moved),
            "{}: moved unit missing",
            path.display()
        );
        assert!(
            rewritten.contains("hfoo"),
            "{}: added footman missing",
            path.display()
        );
        assert!(
            rewritten.contains("0.50 * life"),
            "{}: life missing",
            path.display()
        );
        assert_eq!(
            rewritten.matches("CreateAllUnits").count(),
            2,
            "{}: CreateAllUnits is not defined and called once each",
            path.display()
        );
        if language == Language::Lua {
            full_moon::parse_fallible(&rewritten, full_moon::LuaVersion::lua53())
                .into_result()
                .map_err(|e| format!("{}: {e:?}", path.display()))?;
        }

        let again = UnitSection {
            placements: &edited,
            original: Some(&edited),
            tables: &tables,
            kinds: &kinds,
        };

        assert_eq!(
            script::unit_functions(language, &rewritten, &again)?,
            script::existing_unit_functions(language, &rewritten),
            "{}: the rewritten script does not reproduce itself",
            path.display()
        );
        *checked.entry(language == Language::Lua).or_insert(0) += 1;
    }

    assert_eq!(checked.get(&true), Some(&3), "too few Lua maps");
    assert_eq!(checked.get(&false), Some(&3), "too few JASS maps");
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn edited_properties_rewrite_the_setup_script() -> TestResult {
    use wc3_map::info::{Force, MapInfo};
    use wc3_map::script::Language;
    use wc3_map::setup::{self, SetupSection};
    use wc3_map::terrain::Terrain;

    const TERRAIN_FOG: u32 = 0x2000;
    const WATER_TINTING: u32 = 0x1_0000;
    const CUSTOM_FORCES: u32 = 0x0040;
    const ALLIED: u32 = 0x01;

    let mut checked = [0, 0];

    for path in &script_maps()? {
        let Ok(map) = Map::open(path) else {
            continue;
        };
        let Ok(Some((language, text))) = map_script(&map) else {
            continue;
        };
        let Ok(Some(info)) = MapInfo::load(&map) else {
            continue;
        };
        let Some(original) = info.properties() else {
            continue;
        };
        let bounds = Terrain::load(&map).ok().flatten().map(|t| t.bounds());
        let slot = usize::from(language == Language::Lua);

        if checked[slot] >= 3
            || original.players.len() < 2
            || !setup::reproduces(
                language,
                &text,
                &SetupSection {
                    properties: original,
                    bounds,
                },
            )
        {
            continue;
        }

        let mut edited = original.clone();

        edited.flags |= TERRAIN_FOG | WATER_TINTING | CUSTOM_FORCES;
        edited.weather = FourCC(*b"RAlr");
        edited.water_color = [0x40, 0x80, 0xc0, 0xff];
        edited.sound_environment = "cave".to_owned();
        edited.players[0].race = 2;
        edited.forces = vec![Force {
            flags: ALLIED,
            players: edited
                .players
                .iter()
                .fold(0, |mask, p| mask | (1 << p.number)),
            name: "Everyone".to_owned(),
        }];

        let section = SetupSection {
            properties: &edited,
            bounds,
        };
        let rewritten = setup::rewrite(language, &text, &section)?;
        let first = edited.players[0].number;
        let second = edited.players[1].number;

        for expected in [
            "SetTerrainFog",
            "SetWaterBaseColor",
            "AddWeatherEffect",
            "RAlr",
            "RACE_PREF_ORC",
            "NewSoundEnvironment(\"cave\")",
            "InitCustomTeams",
            &format!("SetPlayerAllianceStateAllyBJ(Player({first}), Player({second}), true)"),
        ] {
            let found = rewritten.contains(expected)
                || rewritten
                    .replace("( ", "(")
                    .replace(" )", ")")
                    .contains(expected);

            assert!(found, "{}: {expected} missing", path.display());
        }

        if language == Language::Lua {
            full_moon::parse_fallible(&rewritten, full_moon::LuaVersion::lua53())
                .into_result()
                .map_err(|e| format!("{}: {e:?}", path.display()))?;
        }

        assert!(
            setup::reproduces(language, &rewritten, &section),
            "{}: the rewritten script does not reproduce itself",
            path.display()
        );
        checked[slot] += 1;
    }

    assert_eq!(checked, [3, 3], "too few maps checked (JASS, Lua)");
    Ok(())
}

#[test]
#[ignore = "needs maps and an install (WC3_MAPS_DIR, WC3_GAME_DIR)"]
fn exports_every_object_type_with_filled_tooltips() -> TestResult {
    use wc3_map::export::{self, CsvValues, ExportType, GameData, LoadedMap, Origin, Selection};

    const BLIZZARD: &str = "AHbz";
    const EXTENDED_TOOLTIP: &str = "aub1";

    let files = GameFiles::open(&env_dir(GAME_DIR_VAR)?, GameOptions::default())?;
    let units = UnitDatabase::load(&files)?;
    let items = UnitDatabase::load_items(&files)?;
    let buffs = UnitDatabase::load_buffs(&files)?;
    let destructables = UnitDatabase::load_destructables(&files)?;
    let abilities = AbilityDatabase::load(&files)?;
    let upgrades = UpgradeDatabase::load(&files)?;
    let doodads = DoodadDatabase::load(&files)?;
    let game = GameData {
        units: &units,
        items: &items,
        buffs: &buffs,
        destructables: &destructables,
        abilities: &abilities,
        upgrades: &upgrades,
        doodads: &doodads,
        mode: ArtMode::Sd,
    };
    let everything = Selection {
        types: ExportType::ALL.to_vec(),
        ..Selection::default()
    };
    let empty = LoadedMap::empty();
    let exported = export::export(&game, &empty.data(), None, &everything);

    for kind in ExportType::ALL {
        let objects = exported
            .objects
            .get(&kind)
            .map(Vec::len)
            .unwrap_or_default();
        let rows = exported
            .to_csv(kind, CsvValues::Raw)
            .matches("\r\n")
            .count();

        assert!(objects > 0, "no {}", kind.name());
        assert!(
            rows > objects,
            "{}: {rows} CSV lines for {objects} objects",
            kind.name()
        );
    }

    let blizzard = exported
        .objects
        .get(&ExportType::Abilities)
        .and_then(|list| list.iter().find(|o| o.id == BLIZZARD))
        .ok_or("no Blizzard")?;
    let tooltip = blizzard
        .values
        .get(EXTENDED_TOOLTIP)
        .ok_or("no extended tooltip")?
        .levels();

    assert_eq!(tooltip.len(), 3);
    assert!(tooltip.iter().all(|v| !v.display.contains('<')));
    assert!(
        exported
            .fields
            .get(&ExportType::Abilities)
            .is_some_and(|f| f.get(EXTENDED_TOOLTIP).is_some_and(|i| i.per_level))
    );
    assert_eq!(
        exported,
        export::export(&game, &empty.data(), None, &everything)
    );

    let mut maps = Vec::new();

    find_maps(&env_dir(MAPS_DIR_VAR)?, &mut maps)?;
    let changed = Selection {
        origin: Origin::Changed,
        ..everything.clone()
    };
    let custom = Selection {
        origin: Origin::Custom,
        ..everything
    };
    let with_changes = maps.iter().find_map(|path| {
        let loaded = LoadedMap::load(&Map::open(path).ok()?).ok()?;
        let exported = export::export(&game, &loaded.data(), None, &changed);

        (exported.object_count() > 0).then_some(exported)
    });
    let exported = with_changes.ok_or("no map with changed objects")?;

    assert!(exported.objects.values().flatten().all(|o| o.changed));
    assert!(
        export::export(&game, &empty.data(), None, &custom)
            .objects
            .values()
            .all(Vec::is_empty)
    );
    assert!(
        exported
            .objects
            .values()
            .flatten()
            .any(|o| !o.modified.is_empty())
    );
    Ok(())
}
