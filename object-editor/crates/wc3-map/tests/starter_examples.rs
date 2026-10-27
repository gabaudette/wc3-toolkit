//! Validate the actual downloadable artifacts without a Warcraft installation.

use wc3_map::FourCC;
use wc3_map::transfer::{Bundle, ObjectType};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const HERO_FILE: &[u8] =
    include_bytes!("../../../../website/public/downloads/starters/sunwell-warden-v1.w3obj");
const HERO_CODE: &str =
    include_str!("../../../../website/public/downloads/starters/sunwell-warden-v1.txt");
const ITEM_FILE: &[u8] =
    include_bytes!("../../../../website/public/downloads/starters/wayfarer-draught-v1.w3obj");
const ITEM_CODE: &str =
    include_str!("../../../../website/public/downloads/starters/wayfarer-draught-v1.txt");

fn id(code: &str) -> Result<FourCC, String> {
    FourCC::parse(code).ok_or_else(|| format!("invalid raw code {code}"))
}

#[test]
fn downloadable_files_and_share_codes_carry_the_same_objects_without_assets() -> TestResult {
    for (file, code) in [(HERO_FILE, HERO_CODE), (ITEM_FILE, ITEM_CODE)] {
        let bundle = Bundle::from_file(file)?;
        assert_eq!(bundle, Bundle::from_code(code)?);
        assert!(bundle.files().is_empty());
        assert_eq!(Bundle::from_file(&bundle.to_file())?, bundle);
    }
    Ok(())
}

#[test]
fn downloads_contain_the_documented_hero_and_merchant_item_dependency() -> TestResult {
    let hero = Bundle::from_file(HERO_FILE)?;
    let item = Bundle::from_file(ITEM_FILE)?;
    let inventory = |bundle: &Bundle| {
        bundle
            .objects()
            .into_iter()
            .map(|o| (o.ty, o.id.to_string(), o.base.to_string()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        inventory(&hero),
        vec![(ObjectType::Unit, "H000".into(), "Hpal".into())]
    );
    assert_eq!(
        inventory(&item),
        vec![
            (ObjectType::Unit, "n000".into(), "ngme".into()),
            (ObjectType::Item, "I000".into(), "phea".into()),
        ]
    );
    for (field, expected) in [
        ("unam", "Sunwell Warden"),
        ("ustr", "24"),
        ("uagi", "15"),
        ("uint", "20"),
        ("ustp", "2"),
        ("uagp", "1"),
        ("uinp", "3"),
    ] {
        assert_eq!(
            hero.text(ObjectType::Unit, id("H000")?, field).as_deref(),
            Some(expected)
        );
    }
    for (field, expected) in [
        ("unam", "Wayfarer Draught"),
        ("igol", "125"),
        ("iuse", "3"),
        ("isst", "0"),
        ("utip", "Purchase Wayfarer Draught"),
        ("utub", "Restores hit points when used. Contains 3 charges."),
    ] {
        assert_eq!(
            item.text(ObjectType::Item, id("I000")?, field).as_deref(),
            Some(expected)
        );
    }
    assert_eq!(
        item.text(ObjectType::Unit, id("n000")?, "unam").as_deref(),
        Some("Wayfarer Supplies")
    );
    assert_eq!(
        item.text(ObjectType::Unit, id("n000")?, "usei").as_deref(),
        Some("I000")
    );
    Ok(())
}
