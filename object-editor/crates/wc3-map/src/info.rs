use crate::binary::{Reader, put_count, put_u32};
use crate::placements::DropChance;
use crate::{FourCC, Map, Result};

pub const INFO_FILE: &str = "war3map.w3i";

// War3map.w3i versions: 25 is The Frozen Throne (the first with item tables),
// 28 adds game versions and Lua, 31 Reforged's modes and enemy priorities,
// 32 and 33 camera distances, 39 (patch 2.0) loading screen sources, fog,
// water and player HUD skins.
const TFT_VERSION: u32 = 25;
const GAME_VERSION_VERSION: u32 = 28;
const REFORGED_VERSION: u32 = 31;
const CAMERA_VERSION: u32 = 32;
const MIN_CAMERA_VERSION: u32 = 33;
const PATCH_2_VERSION: u32 = 39;
const VERSION_INFO_BYTES: usize = 8;
const GAME_VERSION_BYTES: usize = 16;
const BOUNDS_BYTES: usize = 8 * 4 + 4 * 4 + 8;
const PATCH_2_FOG_BYTES: usize = 4 * 6;
const MODE_BYTES: usize = 8;
const CAMERA_DISTANCE_BYTES: usize = 8;
const MIN_CAMERA_BYTES: usize = 4;
const PATCH_2_WATER_BYTES: usize = 4 * 10;
const PLAYER_POSITION_BYTES: usize = 8;
const PRIORITY_BYTES: usize = 8;
const UPGRADE_BYTES: usize = 16;
const TECH_BYTES: usize = 8;
const UNIT_TABLE_POSITION_BYTES: usize = 4;
const LUA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTable {
    pub number: u32,
    pub name: String,
    pub sets: Vec<Vec<DropChance>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    pub model: String,
    pub text: String,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fog {
    pub style: u32,
    pub start: f32,
    pub end: f32,
    pub density: f32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub number: u32,
    pub controller: u32,
    pub race: u32,
    pub fixed_start: bool,
    pub name: String,
    hud_skin: Option<u32>,
    fixed_start_raw: u32,
    rest: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Force {
    pub flags: u32,
    pub players: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Properties {
    version: u32,
    pub name: String,
    pub author: String,
    pub description: String,
    pub suggested_players: String,
    pub flags: u32,
    pub loading_screen_number: i32,
    pub loading_screen: Screen,
    pub game_data_set: u32,
    pub prologue: Screen,
    pub fog: Fog,
    pub weather: FourCC,
    pub sound_environment: String,
    pub light_tileset: u8,
    pub water_color: [u8; 4],
    pub players: Vec<Player>,
    pub forces: Vec<Force>,
    version_info: Vec<u8>,
    bounds: Vec<u8>,
    tileset: u8,
    loading_screen_source: Option<u32>,
    fog_extra: Vec<u8>,
    script: Option<u32>,
    options_tail: Vec<u8>,
}

impl Properties {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn uses_lua(&self) -> bool {
        self.script == Some(LUA)
    }
}

/// `war3map.w3i`: the map's properties, players and forces, and item tables.
/// Upgrade and tech availability and random unit tables are kept byte for byte.
#[derive(Debug, Clone, PartialEq)]
pub struct MapInfo {
    properties: Option<Properties>,
    middle: Vec<u8>,
    tables: Option<Vec<ItemTable>>,
    after: Vec<u8>,
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn put_text(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

fn screen(r: &mut Reader<'_>, with_model: bool) -> Result<Screen> {
    let model = if with_model {
        text(r.c_string()?)
    } else {
        String::new()
    };

    Ok(Screen {
        model,
        text: text(r.c_string()?),
        title: text(r.c_string()?),
        subtitle: text(r.c_string()?),
    })
}

fn put_screen(out: &mut Vec<u8>, screen: &Screen) {
    for value in [&screen.model, &screen.text, &screen.title, &screen.subtitle] {
        put_text(out, value);
    }
}

fn parse_properties(r: &mut Reader<'_>, version: u32, hud_skins: bool) -> Result<Properties> {
    let start = r.pos();

    r.take(VERSION_INFO_BYTES)?;
    if version >= GAME_VERSION_VERSION {
        r.take(GAME_VERSION_BYTES)?;
    }

    let version_info = r.since(start);
    let name = text(r.c_string()?);
    let author = text(r.c_string()?);
    let description = text(r.c_string()?);
    let suggested_players = text(r.c_string()?);
    let bounds = r.take(BOUNDS_BYTES)?.to_vec();
    let flags = r.u32()?;
    let tileset = r.u8()?;
    let loading_screen_number = r.i32()?;
    let loading_screen_source = if version >= PATCH_2_VERSION {
        Some(r.u32()?)
    } else {
        None
    };
    let loading_screen = screen(r, true)?;
    let game_data_set = r.u32()?;
    let prologue = screen(r, true)?;
    let fog = Fog {
        style: r.u32()?,
        start: r.f32()?,
        end: r.f32()?,
        density: r.f32()?,
        color: [r.u8()?, r.u8()?, r.u8()?, r.u8()?],
    };
    let weather = r.fourcc()?;
    let fog_extra = if version >= PATCH_2_VERSION {
        r.take(PATCH_2_FOG_BYTES)?.to_vec()
    } else {
        Vec::new()
    };
    let sound_environment = text(r.c_string()?);
    let light_tileset = r.u8()?;
    let water_color = [r.u8()?, r.u8()?, r.u8()?, r.u8()?];
    let script = if version >= GAME_VERSION_VERSION {
        Some(r.u32()?)
    } else {
        None
    };
    let tail_start = r.pos();

    if version >= REFORGED_VERSION {
        r.take(MODE_BYTES)?;
    }

    if version >= CAMERA_VERSION {
        r.take(CAMERA_DISTANCE_BYTES)?;
    }

    if version >= MIN_CAMERA_VERSION {
        r.take(MIN_CAMERA_BYTES)?;
    }

    if version >= PATCH_2_VERSION {
        r.take(PATCH_2_WATER_BYTES)?;
    }

    let options_tail = r.since(tail_start);
    let mut players = Vec::new();

    for _ in 0..r.count(4)? {
        let number = r.u32()?;
        let controller = r.u32()?;
        let race = r.u32()?;
        let hud_skin = if hud_skins { Some(r.u32()?) } else { None };
        let fixed_start_raw = r.u32()?;
        let name = text(r.c_string()?);
        let rest_start = r.pos();

        r.take(PLAYER_POSITION_BYTES + PRIORITY_BYTES)?;
        if version >= REFORGED_VERSION {
            r.take(PRIORITY_BYTES)?;
        }

        players.push(Player {
            number,
            controller,
            race,
            fixed_start: fixed_start_raw != 0,
            name,
            hud_skin,
            fixed_start_raw,
            rest: r.since(rest_start),
        });
    }

    let mut forces = Vec::new();

    for _ in 0..r.count(4)? {
        forces.push(Force {
            flags: r.u32()?,
            players: r.u32()?,
            name: text(r.c_string()?),
        });
    }

    Ok(Properties {
        version,
        name,
        author,
        description,
        suggested_players,
        flags,
        loading_screen_number,
        loading_screen,
        game_data_set,
        prologue,
        fog,
        weather,
        sound_environment,
        light_tileset,
        water_color,
        players,
        forces,
        version_info,
        bounds,
        tileset,
        loading_screen_source,
        fog_extra,
        script,
        options_tail,
    })
}

fn write_properties(out: &mut Vec<u8>, p: &Properties) {
    put_u32(out, p.version);
    out.extend_from_slice(&p.version_info);
    for value in [&p.name, &p.author, &p.description, &p.suggested_players] {
        put_text(out, value);
    }

    out.extend_from_slice(&p.bounds);
    put_u32(out, p.flags);
    out.push(p.tileset);
    out.extend_from_slice(&p.loading_screen_number.to_le_bytes());
    if let Some(source) = p.loading_screen_source {
        put_u32(out, source);
    }

    put_screen(out, &p.loading_screen);
    put_u32(out, p.game_data_set);
    put_screen(out, &p.prologue);
    put_u32(out, p.fog.style);
    for value in [p.fog.start, p.fog.end, p.fog.density] {
        out.extend_from_slice(&value.to_le_bytes());
    }

    out.extend_from_slice(&p.fog.color);
    out.extend_from_slice(&p.weather.0);
    out.extend_from_slice(&p.fog_extra);
    put_text(out, &p.sound_environment);
    out.push(p.light_tileset);
    out.extend_from_slice(&p.water_color);
    if let Some(script) = p.script {
        put_u32(out, script);
    }

    out.extend_from_slice(&p.options_tail);
    put_count(out, p.players.len());
    for player in &p.players {
        put_u32(out, player.number);
        put_u32(out, player.controller);
        put_u32(out, player.race);
        if let Some(hud) = player.hud_skin {
            put_u32(out, hud);
        }

        let fixed = match (player.fixed_start, player.fixed_start_raw) {
            (true, 0) => 1,
            (true, raw) => raw,
            (false, _) => 0,
        };

        put_u32(out, fixed);
        put_text(out, &player.name);
        out.extend_from_slice(&player.rest);
    }

    put_count(out, p.forces.len());
    for force in &p.forces {
        put_u32(out, force.flags);
        put_u32(out, force.players);
        put_text(out, &force.name);
    }
}

fn skip_availability(r: &mut Reader<'_>) -> Result<()> {
    for entry in [UPGRADE_BYTES, TECH_BYTES] {
        if r.remaining() < 4 {
            return Ok(());
        }

        let n = r.count(entry)?;

        r.take(n * entry)?;
    }

    if r.remaining() < 4 {
        return Ok(());
    }

    for _ in 0..r.count(4)? {
        r.u32()?;
        r.c_string()?;
        let positions = r.count(UNIT_TABLE_POSITION_BYTES)?;

        r.take(positions * UNIT_TABLE_POSITION_BYTES)?;
        for _ in 0..r.count(4)? {
            r.u32()?;
            r.take(positions * 4)?;
        }
    }

    Ok(())
}

fn parse_tables(r: &mut Reader<'_>) -> Result<Vec<ItemTable>> {
    let count = r.count(4)?;
    let mut tables = Vec::with_capacity(count);

    for _ in 0..count {
        let number = r.u32()?;
        let name = String::from_utf8(r.c_string()?.to_vec())
            .map_err(|_| r.damaged("an item table name is not UTF-8"))?;
        let set_count = r.count(4)?;
        let mut sets = Vec::with_capacity(set_count);

        for _ in 0..set_count {
            let n = r.count(8)?;
            let mut set = Vec::with_capacity(n);

            for _ in 0..n {
                let chance = r.u32()?;

                set.push(DropChance {
                    item: r.fourcc()?,
                    chance,
                });
            }

            sets.push(set);
        }

        tables.push(ItemTable { number, name, sets });
    }

    Ok(tables)
}

impl MapInfo {
    pub fn load(map: &Map) -> Result<Option<Self>> {
        map.read_file(INFO_FILE)?
            .map(|bytes| Self::parse(&bytes))
            .transpose()
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(INFO_FILE, data);
        let version = r.u32()?;

        if version < TFT_VERSION {
            return Ok(Self {
                properties: None,
                middle: data.to_vec(),
                tables: None,
                after: Vec::new(),
            });
        }

        let hud_skins = version >= PATCH_2_VERSION;

        // A few of the game's own patch 2.0 maps leave out the players' HUD skins.
        Self::parse_layout(data, version, hud_skins)
            .or_else(|error| Self::parse_layout(data, version, !hud_skins).map_err(|_| error))
    }

    fn parse_layout(data: &[u8], version: u32, hud_skins: bool) -> Result<Self> {
        let mut r = Reader::new(INFO_FILE, data);

        r.u32()?;
        let properties = parse_properties(&mut r, version, hud_skins)?;
        let middle_start = r.pos();

        skip_availability(&mut r)?;
        if r.remaining() < 4 {
            return Ok(Self {
                properties: Some(properties),
                middle: data[middle_start..].to_vec(),
                tables: None,
                after: Vec::new(),
            });
        }

        let middle = r.since(middle_start);
        let tables = parse_tables(&mut r)?;
        let after = r.take(r.remaining())?.to_vec();

        Ok(Self {
            properties: Some(properties),
            middle,
            tables: Some(tables),
            after,
        })
    }

    pub fn properties(&self) -> Option<&Properties> {
        self.properties.as_ref()
    }

    pub fn properties_mut(&mut self) -> Option<&mut Properties> {
        self.properties.as_mut()
    }

    pub fn tables(&self) -> &[ItemTable] {
        self.tables.as_deref().unwrap_or_default()
    }

    pub fn supports_tables(&self) -> bool {
        self.tables.is_some()
    }

    pub fn set_tables(&mut self, tables: Vec<ItemTable>) {
        if self.tables.is_some() {
            self.tables = Some(tables);
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        if let Some(properties) = &self.properties {
            write_properties(&mut out, properties);
        }

        out.extend_from_slice(&self.middle);
        if let Some(tables) = &self.tables {
            put_count(&mut out, tables.len());
            for table in tables {
                put_u32(&mut out, table.number);
                put_text(&mut out, &table.name);
                put_count(&mut out, table.sets.len());
                for set in &table.sets {
                    put_count(&mut out, set.len());
                    for drop in set {
                        put_u32(&mut out, drop.chance);
                        out.extend_from_slice(&drop.item.0);
                    }
                }
            }
        }

        out.extend_from_slice(&self.after);
        out
    }
}
