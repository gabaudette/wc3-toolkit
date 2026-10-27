//! Hotkey clashes: two buttons in the same menu with the same key. Also how
//! a spell would fit on a unit's menus before it is given to the unit.
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
use wc3_data::abilities::AbilityInfo;
use wc3_data::units::parse_hotkey;
use wc3_map::FourCC;

use crate::view::View;

const NO_ATTACKS: &str = "0";
const NO_MOVEMENT: &str = "0";
/// Spots on a command card or learn menu: 4 columns, 3 rows.
const SPOTS: usize = 12;
/// The learn menu's own button, bottom right.
const CANCEL: &str = "Cancel";
/// A unit's normal abilities and a hero's learnable ones.
pub const ABILITY_LIST: &str = "uabi";
pub const HERO_ABILITY_LIST: &str = "uhab";

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
    icon: Option<String>,
    /// The ability behind the button, for abilities.
    ability: Option<FourCC>,
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
            icon: None,
            ability: None,
        });
    }

    fn add_replaceable(
        &mut self,
        key: Option<char>,
        name: String,
        slot: Slot,
        icon: Option<String>,
    ) {
        self.buttons.push(Button {
            key,
            name,
            slot,
            replaces_on_slot: true,
            icon,
            ability: None,
        });
    }

    fn add_ability(&mut self, ability: FourCC, info: Option<&AbilityInfo>, learn: bool) {
        let (key, slot) = match (info, learn) {
            (Some(a), false) => (a.hotkey.as_deref(), a.button),
            (Some(a), true) => (a.learn_hotkey.as_deref(), a.learn_button),
            (None, _) => (None, None),
        };
        self.buttons.push(Button {
            key: key.and_then(parse_hotkey),
            name: ability_name(info, ability),
            slot,
            replaces_on_slot: false,
            icon: info.and_then(|a| a.icon.clone()),
            ability: Some(ability),
        });
    }

    fn clashes(&self) -> Vec<Clash> {
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

fn unit_icon(view: &View<'_>, id: FourCC) -> Option<String> {
    view.raw_by_id(id, "uico")
}

/// Looks up an ability by ID, with the map's changes applied.
pub type AbilityLookup<'a> = &'a dyn Fn(FourCC) -> Option<AbilityInfo>;

fn ability_name(info: Option<&AbilityInfo>, id: FourCC) -> String {
    info.map_or_else(|| id.to_string(), |a| a.name.clone())
}

/// The unit's ability lists as the map has them.
pub fn ability_lists(view: &View<'_>, id: FourCC) -> (Vec<FourCC>, Vec<FourCC>) {
    (
        codes(view.raw_by_id(id, ABILITY_LIST)),
        codes(view.raw_by_id(id, HERO_ABILITY_LIST)),
    )
}

/// The unit's command card, learn menu and build menu, with `extra` added to
/// its abilities (to the hero abilities when `extra_learned`) if it is not
/// there yet.
fn menus(
    view: &View<'_>,
    abilities: AbilityLookup<'_>,
    id: FourCC,
    extra: Option<(FourCC, bool)>,
) -> [Menu; 3] {
    let raw = |field: &str| view.raw_by_id(id, field);
    let is_building = raw("ubdg").as_deref() == Some("1");
    let (mut unit_abilities, mut hero_abilities) = ability_lists(view, id);
    if let Some((spell, learned)) = extra
        && !unit_abilities.contains(&spell)
        && !hero_abilities.contains(&spell)
    {
        if learned {
            hero_abilities.push(spell);
        } else {
            unit_abilities.push(spell);
        }
    }

    let mut card = Menu::new("Command card");

    let mut command = |section: &str| {
        if let Some((name, key)) = view.db.command(section) {
            card.add(Some(key), name, view.db.command_button(section));
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

    for ability in unit_abilities.iter().chain(&hero_abilities) {
        card.add_ability(*ability, abilities(*ability).as_ref(), false);
    }

    for unit in trains.into_iter().chain(codes(raw("uupt"))) {
        card.add_replaceable(
            unit_hotkey(view, unit),
            view.name(unit),
            unit_slot(view, unit),
            unit_icon(view, unit),
        );
    }

    for upgrade in codes(raw("ures")) {
        let code = upgrade.to_string();
        card.add_replaceable(view.db.hotkey(&code), view.db.name(&code), None, None);
    }

    let mut learn = Menu::new("Learn menu");
    if !hero_abilities.is_empty() {
        // Its key is Esc, which never clashes with a letter.
        learn.add(None, CANCEL.to_owned(), view.db.command_button("CmdCancel"));
    }
    for ability in &hero_abilities {
        learn.add_ability(*ability, abilities(*ability).as_ref(), true);
    }

    let mut build = Menu::new("Build menu");
    for structure in codes(raw("ubui")) {
        build.add_replaceable(
            unit_hotkey(view, structure),
            view.name(structure),
            unit_slot(view, structure),
            unit_icon(view, structure),
        );
    }

    [card, learn, build]
}

pub fn clashes(view: &View<'_>, abilities: AbilityLookup<'_>, id: FourCC) -> Vec<Clash> {
    menus(view, abilities, id, None)
        .iter()
        .flat_map(Menu::clashes)
        .collect()
}

/// A button on a menu preview.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardButton {
    pub name: String,
    pub icon: Option<String>,
    pub hotkey: Option<char>,
    pub x: u32,
    pub y: u32,
    /// The spell being placed.
    pub spell: bool,
}

/// How a spell sits on a unit's menus once given: what each menu would show,
/// and what the player would notice.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpellFit {
    /// The unit already has the spell.
    pub has: bool,
    /// The spell goes in the hero's learnable abilities rather than the normal ones.
    pub learned: bool,
    pub card: Vec<CardButton>,
    /// Empty unless the spell is learned.
    pub learn: Vec<CardButton>,
    pub warnings: Vec<String>,
}

fn preview(menu: &Menu, spell: FourCC) -> Vec<CardButton> {
    menu.buttons
        .iter()
        .filter_map(|b| {
            let (x, y) = b.slot?;
            Some(CardButton {
                name: b.name.clone(),
                icon: b.icon.clone(),
                hotkey: b.key,
                x,
                y,
                spell: b.ability == Some(spell),
            })
        })
        .collect()
}

/// `A and B`, `A, B and C`.
fn and_list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// "on" a card, "in" a menu.
fn menu_preposition(menu: &str) -> &'static str {
    if menu.ends_with("menu") { "in" } else { "on" }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// What the player would notice about the spell's button on one menu.
fn menu_warnings(menu: &Menu, spell: FourCC, warnings: &mut Vec<String>) {
    let Some(button) = menu.buttons.iter().find(|b| b.ability == Some(spell)) else {
        return;
    };
    let others: Vec<&Button> = menu
        .buttons
        .iter()
        .filter(|b| b.ability != Some(spell))
        .collect();
    let place = format!(
        "{} the {}",
        menu_preposition(menu.name),
        menu.name.to_lowercase()
    );

    if let Some(key) = button.key {
        let names: Vec<String> = others
            .iter()
            .filter(|b| b.key == Some(key))
            .map(|b| b.name.clone())
            .collect();
        if !names.is_empty() {
            warnings.push(format!(
                "{}, {key} is also the hotkey of {}.",
                capitalized(&place),
                and_list(&names)
            ));
        }
    }

    let mut taken: Vec<(u32, u32)> = others.iter().filter_map(|b| b.slot).collect();
    taken.sort_unstable();
    taken.dedup();
    if taken.len() >= SPOTS {
        warnings.push(format!("Every spot {place} is already taken."));
    } else if let Some(slot) = button.slot {
        let names: Vec<String> = others
            .iter()
            .filter(|b| b.slot == Some(slot))
            .map(|b| b.name.clone())
            .collect();
        if !names.is_empty() {
            warnings.push(format!(
                "{}, its button shares a spot with {}.",
                capitalized(&place),
                and_list(&names)
            ));
        }
    }
}

/// How `spell` fits on `unit`'s menus, as if it were given to the unit (or as
/// it is, when the unit has it). Hero spells go in a hero's learnable
/// abilities; everything else, hero spells on other units included, in the
/// normal ones.
pub fn spell_fit(
    view: &View<'_>,
    abilities: AbilityLookup<'_>,
    unit: FourCC,
    spell: FourCC,
    is_hero: bool,
) -> SpellFit {
    let info = abilities(spell);
    let hero_spell = info.as_ref().is_some_and(|a| a.hero);
    let (unit_abilities, hero_abilities) = ability_lists(view, unit);
    let has = unit_abilities.contains(&spell) || hero_abilities.contains(&spell);
    let learned = if has {
        hero_abilities.contains(&spell)
    } else {
        hero_spell && is_hero
    };

    let [card, learn, _] = menus(view, abilities, unit, Some((spell, learned)));
    let mut warnings = Vec::new();
    if hero_spell && !learned {
        warnings.push(
            "Not a hero, so the spell is a normal ability at level 1; only heroes learn levels."
                .to_owned(),
        );
    }
    menu_warnings(&card, spell, &mut warnings);
    if learned {
        menu_warnings(&learn, spell, &mut warnings);
    }

    SpellFit {
        has,
        learned,
        card: preview(&card, spell),
        learn: if learned {
            preview(&learn, spell)
        } else {
            Vec::new()
        },
        warnings,
    }
}

#[cfg(test)]
mod tests {
    //! Against a real install; run with `WC3_GAME_DIR=... cargo test -- --ignored`.

    use std::path::PathBuf;
    use wc3_data::abilities::AbilityDatabase;

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
                for clash in clashes(&view, &|a| abilities.info(&a.to_string()), id) {
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
    fn lists_read_naturally() {
        let names = |list: &[&str]| list.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
        assert_eq!(and_list(&names(&["Move"])), "Move");
        assert_eq!(and_list(&names(&["Move", "Stop"])), "Move and Stop");
        assert_eq!(
            and_list(&names(&["Move", "Stop", "Patrol"])),
            "Move, Stop and Patrol"
        );
    }

    #[test]
    #[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
    fn storm_bolt_fits_a_paladin_and_a_footman_differently() -> TestResult {
        let (db, abilities, guide) = load()?;
        let view = View {
            db: &db,
            guide: &guide,
            map: None,
            mode: ArtMode::Sd,
        };
        let lookup = |a: FourCC| abilities.info(&a.to_string());
        let id = |code: &str| FourCC::parse(code).ok_or("bad id");
        let storm_bolt = id("AHtb")?;

        let paladin = spell_fit(&view, &lookup, id("Hpal")?, storm_bolt, true);
        assert!(!paladin.has && paladin.learned, "{paladin:?}");
        assert!(paladin.learn.iter().any(|b| b.spell), "{paladin:?}");
        let holy_light = "On the command card, T is also the hotkey of Holy Light.";
        assert!(
            paladin.warnings.iter().any(|w| w == holy_light),
            "{paladin:?}"
        );
        assert!(
            paladin
                .card
                .iter()
                .any(|b| b.name == "Move" && (b.x, b.y) == (0, 0)),
            "{paladin:?}"
        );

        let footman = spell_fit(&view, &lookup, id("hfoo")?, storm_bolt, false);
        assert!(!footman.learned && footman.learn.is_empty(), "{footman:?}");
        assert!(footman.warnings[0].starts_with("Not a hero"), "{footman:?}");
        assert!(footman.card.iter().any(|b| b.spell), "{footman:?}");
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
        let found = clashes(&view, &|a| abilities.info(&a.to_string()), hero);

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
