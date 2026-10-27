use std::path::PathBuf;

use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::{GameFiles, GameOptions};
use wc3_data::meta::FieldMeta;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::abilities::MapAbilities;
use wc3_map::leveled::{LeveledObjects, UPGRADES};
use wc3_map::plain::{BUFFS, ITEMS, PlainObjects};
use wc3_map::transfer::{self, MapObjects, References};
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, Map};

const HERO_FIELDS: &[(&str, &str)] = &[
    ("unam", "Sunwell Warden"),
    ("ustr", "24"),
    ("uagi", "15"),
    ("uint", "20"),
    ("ustp", "2"),
    ("uagp", "1"),
    ("uinp", "3"),
];
const ITEM_FIELDS: &[(&str, &str)] = &[
    ("unam", "Wayfarer Draught"),
    ("igol", "125"),
    ("iuse", "3"),
    ("isst", "0"),
    ("utip", "Purchase Wayfarer Draught"),
    ("utub", "Restores hit points when used. Contains 3 charges."),
];
const SHOP_NAME: &str = "Wayfarer Supplies";
const MAP_CAPACITY: u32 = 16;

fn id(code: &str) -> Result<FourCC, String> {
    FourCC::parse(code).ok_or_else(|| format!("invalid raw code {code}"))
}

fn field<'a>(db: &'a UnitDatabase, code: &str) -> Result<&'a FieldMeta, String> {
    db.field(code)
        .ok_or_else(|| format!("missing field {code}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let game = PathBuf::from(
        args.next()
            .ok_or("usage: starter_examples GAME_DIR OUTPUT_DIR")?,
    );
    let output = PathBuf::from(args.next().ok_or("missing OUTPUT_DIR")?);

    if args.next().is_some() {
        return Err("usage: starter_examples GAME_DIR OUTPUT_DIR".into());
    }

    let files = GameFiles::open(&game, GameOptions::default())?;
    let db = UnitDatabase::load(&files)?;
    let item_db = UnitDatabase::load_items(&files)?;
    let ability_db = AbilityDatabase::load(&files)?;
    let upgrade_db = UpgradeDatabase::load(&files)?;
    let refs = References::new(
        db.fields(),
        ability_db.fields(),
        item_db.fields(),
        upgrade_db.fields(),
    );

    let temp = tempfile::tempdir()?;
    let path = temp.path().join("starters.w3m");
    mpq::Archive::create(&path, MAP_CAPACITY)?.save()?;
    let map = Map::open(&path)?;
    let mut units = MapUnits::load(&map)?;
    let abilities = MapAbilities::load(&map)?;
    let mut items = PlainObjects::load(&map, ITEMS)?;
    let upgrades = LeveledObjects::load(&map, UPGRADES)?;
    let buffs = PlainObjects::load(&map, BUFFS)?;

    let hero = units.create(&db, id("Hpal")?)?;

    for (code, value) in HERO_FIELDS {
        units.set(&db, hero, field(&db, code)?, value)?;
    }

    let item = items.create(&item_db, id("phea")?)?;

    for (code, value) in ITEM_FIELDS {
        items.set(
            &item_db,
            units.strings_mut(),
            item,
            field(&item_db, code)?,
            value,
        )?;
    }

    let shop = units.create(&db, id("ngme")?)?;

    units.set(&db, shop, field(&db, "unam")?, SHOP_NAME)?;
    units.set(&db, shop, field(&db, "usei")?, &item.to_string())?;

    if hero != id("H000")? || item != id("I000")? || shop != id("n000")? {
        return Err("starter IDs changed; review the sources and documentation".into());
    }

    std::fs::create_dir_all(&output)?;
    for (name, root, expected) in [
        ("sunwell-warden-v1", hero, 1),
        ("wayfarer-draught-v1", shop, 2),
    ] {
        let export = transfer::export(
            MapObjects {
                units: &units,
                abilities: &abilities,
                items: &items,
                upgrades: &upgrades,
                buffs: &buffs,
            },
            &refs,
            &[root],
            &|_| None,
        )?;

        if !export.warnings.is_empty()
            || !export.bundle.files().is_empty()
            || export.objects.len() != expected
        {
            return Err(format!("unexpected dependencies or assets in {name}").into());
        }
        std::fs::write(
            output.join(format!("{name}.w3obj")),
            export.bundle.to_file(),
        )?;
        std::fs::write(
            output.join(format!("{name}.txt")),
            format!("{}\n", export.bundle.to_code()),
        )?;
    }

    Ok(())
}
