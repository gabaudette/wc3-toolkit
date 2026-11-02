use std::collections::HashMap;

use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::ArtMode;
use wc3_data::meta::ObjectKind;
use wc3_data::units::UnitDatabase;
use wc3_map::info::{ItemTable, Properties};
use wc3_map::placements::UnitPlacements;
use wc3_map::script::{self, Language, UnitKinds, UnitSection};
use wc3_map::setup::{self, SetupSection};
use wc3_map::terrain::Terrain;
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, Map};

use crate::state::{OpenMap, Session};

const GOLD_MINE_ABILITY: &str = "Agld";
const ABILITIES_FIELD: &str = "uabi";
const TEAM_COLOR_FIELD: &str = "utco";
const ORDER_ON: &str = "Orderon";
const ORDER_OFF: &str = "Orderoff";
const LINE_END: &str = "\r\n";
const SETUP_UNREPRODUCIBLE: &str = "Tinkerbench cannot rewrite this map's player and environment script exactly as the World Editor does, so changes to players, forces, fog, weather, water and sound take effect in game only after you open and save the map in the World Editor.";
const UNREPRODUCIBLE: &str = "Tinkerbench cannot rewrite this map's unit script exactly as the World Editor does, so placed units and drops changed here take effect in game only after you open and save the map in the World Editor.";

pub struct GameKinds<'a> {
    pub db: &'a UnitDatabase,
    pub items: &'a UnitDatabase,
    pub abilities: &'a AbilityDatabase,
    pub units: &'a MapUnits,
}

impl GameKinds<'_> {
    fn raw(&self, id: FourCC, field_id: &str) -> Option<String> {
        let field = self.db.field(field_id)?;
        let base = self.units.base_of(self.db, id).unwrap_or(id);

        self.units
            .raw_value(self.db, id, field, ArtMode::Sd)
            .or_else(|| self.db.raw_value(&base.to_string(), field, ArtMode::Sd))
    }
}

impl UnitKinds for GameKinds<'_> {
    fn is_building(&self, id: FourCC) -> bool {
        self.units.kind(self.db, id) == Some(ObjectKind::Building)
    }

    fn is_hero(&self, id: FourCC) -> bool {
        self.units.kind(self.db, id) == Some(ObjectKind::Hero)
    }

    fn is_item(&self, id: FourCC) -> bool {
        let code = id.to_string();

        self.items.contains(&code)
            || (!self.db.contains(&code) && self.units.base_of(self.db, id).is_none())
    }

    fn is_gold_mine(&self, id: FourCC) -> bool {
        self.raw(id, ABILITIES_FIELD)
            .is_some_and(|list| list.split(',').any(|a| a.trim() == GOLD_MINE_ABILITY))
    }

    fn team_color(&self, id: FourCC) -> Option<i32> {
        self.raw(id, TEAM_COLOR_FIELD)
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

        Some((order(ORDER_ON)?, order(ORDER_OFF)?))
    }
}

pub struct ScriptState {
    pub file: &'static str,
    pub language: Language,
    pub text: String,
    crlf: bool,
    placements: Option<UnitPlacements>,
    tables: Vec<ItemTable>,
    properties: Option<Properties>,
    pub units_problem: Option<String>,
    pub setup_problem: Option<String>,
}

fn read_script(archive: &Map) -> Option<(&'static str, Language, Vec<u8>)> {
    if let Ok(Some(bytes)) = archive.read_file(script::LUA_FILE) {
        return Some((script::LUA_FILE, Language::Lua, bytes));
    }

    script::JASS_FILES.iter().find_map(|file| {
        archive
            .read_file(file)
            .ok()
            .flatten()
            .map(|bytes| (*file, Language::Jass, bytes))
    })
}

fn kinds<'a>(session: &'a Session, map: &'a OpenMap) -> GameKinds<'a> {
    GameKinds {
        db: &session.db,
        items: &session.items,
        abilities: &session.abilities,
        units: &map.units,
    }
}

fn reproduces(text: &str, language: Language, section: &UnitSection<'_>) -> bool {
    let Ok(generated) = script::unit_functions(language, text, section) else {
        return false;
    };
    let existing: HashMap<String, String> = script::existing_unit_functions(language, text)
        .into_iter()
        .collect();

    existing.len() == generated.len()
        && generated
            .iter()
            .all(|(name, text)| existing.get(name) == Some(text))
}

fn tables_of(map: &OpenMap) -> Vec<ItemTable> {
    map.info
        .as_ref()
        .map(|i| i.tables().to_vec())
        .unwrap_or_default()
}

fn properties_of(map: &OpenMap) -> Option<Properties> {
    map.info.as_ref().and_then(|i| i.properties()).cloned()
}

fn setup_section<'a>(map: &'a OpenMap, properties: &'a Properties) -> SetupSection<'a> {
    SetupSection {
        properties,
        bounds: map.terrain.as_ref().map(Terrain::bounds),
    }
}

pub fn load(session: &Session, map: &OpenMap) -> Option<ScriptState> {
    let archive = Map::open(&map.path).ok()?;
    let (file, language, bytes) = read_script(&archive)?;
    let raw = String::from_utf8_lossy(&bytes);
    let crlf = raw.contains(LINE_END);
    let text = raw.replace(LINE_END, "\n");
    let tables = tables_of(map);
    let kinds = kinds(session, map);
    let units_ok = map.placements.as_ref().is_some_and(|placements| {
        let section = UnitSection {
            placements,
            original: Some(placements),
            tables: &tables,
            kinds: &kinds,
        };

        reproduces(&text, language, &section)
    });
    let properties = properties_of(map);
    let setup_ok = properties
        .as_ref()
        .is_some_and(|p| setup::reproduces(language, &text, &setup_section(map, p)));

    Some(ScriptState {
        file,
        language,
        text,
        crlf,
        placements: map.placements.clone(),
        tables,
        properties,
        units_problem: (!units_ok).then(|| UNREPRODUCIBLE.to_owned()),
        setup_problem: (!setup_ok).then(|| SETUP_UNREPRODUCIBLE.to_owned()),
    })
}

pub fn updated(session: &Session, map: &OpenMap) -> Result<Option<(&'static str, String)>, String> {
    let Some(state) = &map.script else {
        return Ok(None);
    };
    let tables = tables_of(map);
    let properties = properties_of(map);
    let mut text = state.text.clone();
    let units_changed = map.placements != state.placements || tables != state.tables;
    let setup_changed = properties != state.properties;

    if let (true, None, Some(placements)) = (units_changed, &state.units_problem, &map.placements) {
        let kinds = kinds(session, map);
        let section = UnitSection {
            placements,
            original: state.placements.as_ref(),
            tables: &tables,
            kinds: &kinds,
        };
        let generated =
            script::unit_functions(state.language, &text, &section).map_err(|e| e.to_string())?;

        text =
            script::splice_units(state.language, &text, &generated).map_err(|e| e.to_string())?;
    }

    if let (true, None, Some(properties)) = (setup_changed, &state.setup_problem, &properties) {
        text = setup::rewrite(state.language, &text, &setup_section(map, properties))
            .map_err(|e| e.to_string())?;
    }

    Ok((text != state.text).then_some((state.file, text)))
}

impl ScriptState {
    pub fn bytes(&self, text: &str) -> Vec<u8> {
        if self.crlf {
            return text.replace('\n', LINE_END).into_bytes();
        }

        text.as_bytes().to_vec()
    }

    pub fn saved(&mut self, text: Option<String>, map: &OpenMap) {
        if let Some(text) = text {
            self.text = text;
        }

        if self.units_problem.is_none() {
            self.placements = map.placements.clone();
            self.tables = tables_of(map);
        }

        if self.setup_problem.is_none() {
            self.properties = properties_of(map);
        }
    }
}
