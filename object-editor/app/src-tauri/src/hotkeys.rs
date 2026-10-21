//! Hotkey clashes: two buttons in the same menu with the same key.
//!
//! A unit has up to three menus: its command card (standard commands,
//! abilities, units it trains, upgrades it researches, buildings it upgrades
//! into), a hero's learn menu, and a worker's build menu.
//!
//! Trained units on the same slot replace each other, like the Headhunter and
//! its upgrade the Berserker, so a shared key there is fine. Abilities sharing
//! a slot are not reported: many of Blizzard's own creeps do that, so the game
//! evidently places them itself.

use serde::Serialize;
use wc3_data::abilities::AbilityDatabase;
use wc3_data::units::parse_hotkey;
use wc3_map::FourCC;

use crate::view::View;

const NO_ATTACKS: &str = "0";
const NO_MOVEMENT: &str = "0";

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Clash {
    pub key: char,
    /// `Command card`, `Learn menu` or `Build menu`.
    pub menu: &'static str,
    pub names: Vec<String>,
}

type Slot = Option<(u32, u32)>;

struct Button {
    key: Option<char>,
    name: String,
    /// `None` when the slot is not known; such buttons always count as separate.
    slot: Slot,
    /// Trained units and buildings on one slot replace each other (an upgraded
    /// version takes the original's place).
    replaces_on_slot: bool,
}

struct Menu {
    name: &'static str,
    buttons: Vec<Button>,
}

impl Menu {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            buttons: Vec::new(),
        }
    }

    fn add(&mut self, key: Option<char>, name: String, slot: Slot) {
        self.buttons.push(Button {
            key,
            name,
            slot,
            replaces_on_slot: false,
        });
    }

    fn add_replaceable(&mut self, key: Option<char>, name: String, slot: Slot) {
        self.buttons.push(Button {
            key,
            name,
            slot,
            replaces_on_slot: true,
        });
    }

    fn clashes(self) -> Vec<Clash> {
        let mut keys: Vec<char> = self.buttons.iter().filter_map(|b| b.key).collect();
        keys.sort_unstable();
        keys.dedup();

        keys.into_iter()
            .filter_map(|key| {
                let same_key: Vec<&Button> =
                    self.buttons.iter().filter(|b| b.key == Some(key)).collect();
                let first_slot = same_key.first()?.slot;
                let replacements = first_slot.is_some()
                    && same_key
                        .iter()
                        .all(|b| b.replaces_on_slot && b.slot == first_slot);
                if same_key.len() < 2 || replacements {
                    return None;
                }

                Some(Clash {
                    key,
                    menu: self.name,
                    names: same_key.iter().map(|b| b.name.clone()).collect(),
                })
            })
            .collect()
    }
}

fn codes(raw: Option<String>) -> Vec<FourCC> {
    raw.unwrap_or_default()
        .split(',')
        .filter_map(|code| FourCC::parse(code.trim()))
        .collect()
}

/// A unit's own hotkey: the map's value for map units, else the game's.
fn unit_hotkey(view: &View<'_>, id: FourCC) -> Option<char> {
    view.raw_by_id(id, "uhot").as_deref().and_then(parse_hotkey)
}

/// The unit's button slot on its producer's command card.
fn unit_slot(view: &View<'_>, id: FourCC) -> Slot {
    let coordinate = |field: &str| view.raw_by_id(id, field)?.trim().parse::<u32>().ok();
    Some((coordinate("ubpx")?, coordinate("ubpy")?))
}

fn ability_name(abilities: &AbilityDatabase, id: FourCC) -> String {
    abilities
        .info(&id.to_string())
        .map_or_else(|| id.to_string(), |a| a.name)
}

pub fn clashes(view: &View<'_>, abilities: &AbilityDatabase, id: FourCC) -> Vec<Clash> {
    let raw = |field: &str| view.raw_by_id(id, field);
    let is_building = raw("ubdg").as_deref() == Some("1");
    let hero_abilities = codes(raw("uhab"));
    let mut card = Menu::new("Command card");

    let mut command = |section: &str| {
        if let Some((name, key)) = view.db.command(section) {
            card.add(Some(key), name, None);
        }
    };
    if !is_building {
        command("CmdStop");
        if raw("umvs").is_some_and(|s| s != NO_MOVEMENT) {
            command("CmdMove");
            command("CmdHoldPos");
            command("CmdPatrol");
        }

        if raw("uaen").is_some_and(|a| a != NO_ATTACKS) {
            command("CmdAttack");
        }

        if !hero_abilities.is_empty() {
            command("CmdSelectSkill");
        }
    }

    let trains = codes(raw("utra"));
    if is_building && !trains.is_empty() {
        command("CmdRally");
    }

    for ability in codes(raw("uabi"))
        .into_iter()
        .chain(hero_abilities.iter().copied())
    {
        let info = abilities.info(&ability.to_string());
        let key = info
            .as_ref()
            .and_then(|a| a.hotkey.as_deref())
            .and_then(parse_hotkey);
        let slot = info.as_ref().and_then(|a| a.button);
        card.add(key, ability_name(abilities, ability), slot);
    }

    for unit in trains.into_iter().chain(codes(raw("uupt"))) {
        card.add_replaceable(
            unit_hotkey(view, unit),
            view.name(unit),
            unit_slot(view, unit),
        );
    }

    for upgrade in codes(raw("ures")) {
        let code = upgrade.to_string();
        card.add_replaceable(view.db.hotkey(&code), view.db.name(&code), None);
    }

    let mut learn = Menu::new("Learn menu");
    for ability in &hero_abilities {
        let info = abilities.info(&ability.to_string());
        let key = info
            .as_ref()
            .and_then(|a| a.learn_hotkey.as_deref())
            .and_then(parse_hotkey);
        let slot = info.as_ref().and_then(|a| a.learn_button);
        learn.add(key, ability_name(abilities, *ability), slot);
    }

    let mut build = Menu::new("Build menu");
    for structure in codes(raw("ubui")) {
        build.add_replaceable(
            unit_hotkey(view, structure),
            view.name(structure),
            unit_slot(view, structure),
        );
    }

    [card, learn, build]
        .into_iter()
        .flat_map(Menu::clashes)
        .collect()
}

#[cfg(test)]
mod tests {
    //! Against a real install; run with `WC3_GAME_DIR=... cargo test -- --ignored`.

    use std::path::PathBuf;

    use wc3_data::game::{ArtMode, GameFiles, GameOptions};
    use wc3_data::guide::FieldGuide;
    use wc3_data::units::UnitDatabase;
    use wc3_map::Map;
    use wc3_map::units::MapUnits;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const MAX_FILES: u32 = 8;

    fn load() -> Result<(UnitDatabase, AbilityDatabase, FieldGuide), Box<dyn std::error::Error>> {
        let dir = std::env::var_os("WC3_GAME_DIR")
            .map(PathBuf::from)
            .ok_or("set WC3_GAME_DIR")?;
        let files = GameFiles::open(&dir, GameOptions::default())?;
        Ok((
            UnitDatabase::load(&files)?,
            AbilityDatabase::load(&files)?,
            FieldGuide::units()?,
        ))
    }

    /// Blizzard's melee units are the reference: their menus have no clashes.
    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn melee_units_have_no_clashes() -> TestResult {
        let (db, abilities, guide) = load()?;
        let view = View {
            db: &db,
            guide: &guide,
            map: None,
            mode: ArtMode::Sd,
        };
        let mut found = Vec::new();

        for id in view.ids() {
            if view.item(id).is_some_and(|i| i.group == "melee") {
                for clash in clashes(&view, &abilities, id) {
                    found.push(format!(
                        "{id} {}: {} {} {:?}",
                        view.name(id),
                        clash.key,
                        clash.menu,
                        clash.names
                    ));
                }
            }
        }

        assert!(
            found.is_empty(),
            "{} clashes:\n{}",
            found.len(),
            found.join("\n")
        );
        Ok(())
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn storm_bolt_clashes_with_holy_light() -> TestResult {
        let (db, abilities, guide) = load()?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3m");
        mpq::Archive::create(&path, MAX_FILES)?.save()?;
        let mut units = MapUnits::load(&Map::open(&path)?)?;
        let hero = units.create(&db, FourCC::parse("Hpal").ok_or("bad id")?)?;
        let field = db.field("uhab").ok_or("no uhab")?;
        units.set(&db, hero, field, "AHhb,AHds,AHre,AHad,AHtb")?;

        let view = View {
            db: &db,
            guide: &guide,
            map: Some(&units),
            mode: ArtMode::Sd,
        };
        let found = clashes(&view, &abilities, hero);

        let both = |c: &Clash| {
            c.names.contains(&"Holy Light".to_owned()) && c.names.contains(&"Storm Bolt".to_owned())
        };
        assert!(
            found
                .iter()
                .any(|c| c.key == 'T' && c.menu == "Command card" && both(c)),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|c| c.key == 'T' && c.menu == "Learn menu" && both(c)),
            "{found:?}"
        );
        Ok(())
    }
}
