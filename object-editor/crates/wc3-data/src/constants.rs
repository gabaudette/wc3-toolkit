//! Gameplay constants: the World Editor's Advanced > Gameplay Constants.
//!
//! `MiscMetaData.slk` lists them like an Object Editor's fields: the key and
//! section holding each one (`[Misc] MaxHeroLevel`), its type and limits. The
//! game's values come from `MiscData.txt` and `MiscGame.txt` (balance-layered);
//! the techtree equivalents (`[TWN1]`, `[TALT]`, ...) from the item profiles.

use crate::Result;
use crate::game::{GameFiles, decode_text};
use crate::meta::{FieldMeta, parse_metadata};
use crate::profile::Profile;
use crate::slk::Slk;
use crate::units::resolve_string;

const METADATA: &str = "units\\miscmetadata.slk";
const MISC_DATA: &str = "units\\miscdata.txt";
const MISC_GAME: &str = "units\\miscgame.txt";
const ITEM_FUNC: &str = "units\\itemfunc.txt";
const ITEM_STRINGS: &str = "units\\itemstrings.txt";
const EDITOR_STRINGS: &[&str] = &["ui\\worldeditstrings.txt", "ui\\worldeditgamestrings.txt"];
const COMMENT: &str = "//";
const MISC_SECTION: &str = "Misc";
// No game file holds these; they are the game's built-in values.
const BUILT_IN: &[(&str, &str)] = &[
    ("FoodCeiling", "100"),
    ("UpkeepUsage", "50,80"),
    ("UpkeepGoldTax", "0.00,0.30,0.60"),
    ("UpkeepLumberTax", "0.00,0.00,0.00"),
    ("ItemStackingEnabled", "0"),
];

#[derive(Debug, Clone)]
pub struct Constant {
    pub meta: FieldMeta,
    pub label: String,
    pub default: String,
}

#[derive(Debug, Clone)]
pub struct ConstantsDatabase {
    constants: Vec<Constant>,
}

fn merge(profile: &mut Profile, data: Result<Vec<u8>>, path: &str) -> Result<()> {
    profile.merge_text(&decode_text(data?, path)?);
    Ok(())
}

fn without_comment(value: &str) -> String {
    value
        .split(COMMENT)
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned()
}

impl ConstantsDatabase {
    pub fn load(files: &GameFiles) -> Result<Self> {
        let text = decode_text(files.read_base(METADATA)?, METADATA)?;
        let metadata = Slk::parse(&text).map_err(|e| e.in_file(METADATA))?;
        let fields = parse_metadata(&metadata).map_err(|e| e.in_file(METADATA))?;
        let mut values = Profile::new();
        let mut strings = Profile::new();

        merge(&mut values, files.read_base(MISC_DATA), MISC_DATA)?;
        merge(&mut values, files.read_balanced(MISC_GAME), MISC_GAME)?;
        merge(&mut values, files.read_balanced(ITEM_FUNC), ITEM_FUNC)?;
        merge(
            &mut values,
            files.read_localized(ITEM_STRINGS),
            ITEM_STRINGS,
        )?;
        for path in EDITOR_STRINGS {
            merge(&mut strings, files.read_localized(path), path)?;
        }

        let constants = fields
            .into_iter()
            .map(|meta| {
                let built_in = BUILT_IN
                    .iter()
                    .find(|(key, _)| meta.section == MISC_SECTION && *key == meta.field)
                    .map(|(_, value)| (*value).to_owned());
                let default = values
                    .get(&meta.section, &meta.field)
                    .map(without_comment)
                    .or(built_in)
                    .unwrap_or_default();

                Constant {
                    label: resolve_string(&strings, &meta.display_key)
                        .unwrap_or_else(|| meta.field.clone()),
                    default,
                    meta,
                }
            })
            .collect();

        Ok(Self { constants })
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn find(&self, id: &str) -> Option<&Constant> {
        self.constants.iter().find(|c| c.meta.id == id)
    }
}
