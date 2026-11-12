use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::ArtMode;
use wc3_data::meta::ObjectKind;
use wc3_data::units::UnitDatabase;
use wc3_map::script::UnitKinds;
use wc3_map::sync::{MapData, ScriptState};
use wc3_map::terrain::Terrain;
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, Map};

use crate::state::{OpenMap, Session};

const GOLD_MINE_ABILITY: &str = "Agld";
const ABILITIES_FIELD: &str = "uabi";
const TEAM_COLOR_FIELD: &str = "utco";
const ORDER_ON: &str = "Orderon";
const ORDER_OFF: &str = "Orderoff";
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

fn kinds<'a>(session: &'a Session, map: &'a OpenMap) -> GameKinds<'a> {
    GameKinds {
        db: &session.db,
        items: &session.items,
        abilities: &session.abilities,
        units: &map.units,
    }
}

/// The open map's data the script is generated from. Item tables live in the
/// `info` this borrows from, so they come as their own argument.
fn with_data<T>(map: &OpenMap, use_data: impl FnOnce(&MapData<'_>) -> T) -> T {
    let tables = map
        .info
        .as_ref()
        .map(|i| i.tables().to_vec())
        .unwrap_or_default();

    use_data(&MapData {
        placements: map.placements.as_ref(),
        tables: &tables,
        properties: map.info.as_ref().and_then(|i| i.properties()),
        bounds: map.terrain.as_ref().map(Terrain::bounds),
    })
}

pub fn load(session: &Session, map: &OpenMap) -> Option<ScriptState> {
    let archive = Map::open(&map.path).ok()?;

    with_data(map, |data| {
        ScriptState::load(&archive, data, &kinds(session, map))
    })
}

pub fn updated(session: &Session, map: &OpenMap) -> Result<Option<(&'static str, String)>, String> {
    let Some(state) = &map.script else {
        return Ok(None);
    };

    with_data(map, |data| state.updated(data, &kinds(session, map)))
        .map(|text| text.map(|text| (state.file, text)))
        .map_err(|e| e.to_string())
}

pub fn saved(state: &mut ScriptState, text: Option<String>, map: &OpenMap) {
    with_data(map, |data| state.saved(text, data));
}

/// Why placed units, drops and item tables changed here do not reach the game.
pub fn units_problem(state: &ScriptState) -> Option<String> {
    (!state.rewrites_units()).then(|| UNREPRODUCIBLE.to_owned())
}

/// Why players, forces and environment changed here do not reach the game.
pub fn setup_problem(state: &ScriptState) -> Option<String> {
    (!state.rewrites_setup()).then(|| SETUP_UNREPRODUCIBLE.to_owned())
}
