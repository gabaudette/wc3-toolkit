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
const FIXED_START: u32 = 1;
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

// A player's start location is fixed when the field is 1 or holds the text
// older editors left there; 0 and 2 leave it free.
fn fixes_start(raw: u32) -> bool {
    !matches!(raw, 0 | 2)
}

fn f32_at(bytes: &[u8], at: usize) -> f32 {
    bytes
        .get(at..at + 4)
        .map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

impl Player {
    pub fn start(&self) -> [f32; 2] {
        [f32_at(&self.rest, 0), f32_at(&self.rest, 4)]
    }

    /// Start locations (bits by player number) this player's allies prefer
    /// to start near: low and high priority.
    pub fn ally_priorities(&self) -> [u32; 2] {
        [
            u32_at(&self.rest, PLAYER_POSITION_BYTES).unwrap_or_default(),
            u32_at(&self.rest, PLAYER_POSITION_BYTES + 4).unwrap_or_default(),
        ]
    }

    /// The same for enemies; Reforged only.
    pub fn enemy_priorities(&self) -> Option<[u32; 2]> {
        let at = PLAYER_POSITION_BYTES + PRIORITY_BYTES;

        Some([u32_at(&self.rest, at)?, u32_at(&self.rest, at + 4)?])
    }

    pub fn hud_skin(&self) -> Option<u32> {
        self.hud_skin
    }
}

impl Properties {
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Patch 2.0's extra fog settings, as `SetTerrainFogExV` takes them, and
    /// the maximum linear density.
    pub fn fog_extra(&self) -> Option<([f32; 4], f32)> {
        (!self.fog_extra.is_empty()).then(|| {
            (
                [0, 4, 8, 12].map(|at| f32_at(&self.fog_extra, at)),
                f32_at(&self.fog_extra, 16),
            )
        })
    }

    pub fn fog_over_sky(&self) -> bool {
        u32_at(&self.fog_extra, 20).is_some_and(|v| v != 0)
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
    // Patch 2.0 maps store their extra fog settings before the weather.
    let fog_extra = if version >= PATCH_2_VERSION {
        r.take(PATCH_2_FOG_BYTES)?.to_vec()
    } else {
        Vec::new()
    };
    let weather = r.fourcc()?;
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
            fixed_start: fixes_start(fixed_start_raw),
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
    out.extend_from_slice(&p.fog_extra);
    out.extend_from_slice(&p.weather.0);
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

        let fixed = match (player.fixed_start, fixes_start(player.fixed_start_raw)) {
            (wanted, now) if wanted == now => player.fixed_start_raw,
            (true, _) => FIXED_START,
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

#[cfg(test)]
mod tests {
    use super::*;

    const VERSIONS: [u32; 6] = [
        TFT_VERSION,
        GAME_VERSION_VERSION,
        REFORGED_VERSION,
        CAMERA_VERSION,
        MIN_CAMERA_VERSION,
        PATCH_2_VERSION,
    ];
    const OPTIONS: u32 = 0x2040;
    const SOURCE_IMPORTED: u32 = 128;
    const USER: u32 = 1;
    const COMPUTER: u32 = 2;
    const HUMAN: u32 = 1;
    const ORC: u32 = 2;
    /// What older editors left in a player's fixed-start field: it still fixes the start.
    const LEFTOVER_TEXT: u32 = u32::from_le_bytes(*b"tool");
    const FOG_EXTRA: [f32; 5] = [0.1, 0.2, 0.3, 0.4, 0.9];
    const ALL_PLAYERS: u32 = 0b11;
    const FORCE_FLAGS: u32 = 0x3;
    const FILLER: u8 = 7;
    const PRE_TFT_VERSION: u32 = 18;

    fn put_f32(out: &mut Vec<u8>, value: f32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    /// A `war3map.w3i` of the given version, with two players, a force and an
    /// item table. Parts kept as bytes hold filler.
    fn w3i(version: u32, hud_skins: bool) -> Vec<u8> {
        let mut out = Vec::new();
        let filler = |out: &mut Vec<u8>, n: usize| out.extend(std::iter::repeat_n(FILLER, n));

        put_u32(&mut out, version);
        filler(&mut out, VERSION_INFO_BYTES);
        if version >= GAME_VERSION_VERSION {
            filler(&mut out, GAME_VERSION_BYTES);
        }

        for value in ["Sunwell", "Me", "A map", "1-2"] {
            put_text(&mut out, value);
        }

        filler(&mut out, BOUNDS_BYTES);
        put_u32(&mut out, OPTIONS);
        out.push(b'L');
        out.extend_from_slice(&(-1i32).to_le_bytes());
        if version >= PATCH_2_VERSION {
            put_u32(&mut out, SOURCE_IMPORTED);
        }

        for value in ["Load.mdx", "Text", "Title", "Subtitle"] {
            put_text(&mut out, value);
        }

        put_u32(&mut out, 0);
        for value in ["", "Prologue", "Prologue title", "Prologue subtitle"] {
            put_text(&mut out, value);
        }

        put_u32(&mut out, 1);
        for value in [1000.0, 5000.0, 0.5] {
            put_f32(&mut out, value);
        }

        out.extend_from_slice(&[10, 20, 30, 255]);
        if version >= PATCH_2_VERSION {
            for value in FOG_EXTRA {
                put_f32(&mut out, value);
            }

            put_u32(&mut out, 1);
        }

        out.extend_from_slice(b"RAhr");
        put_text(&mut out, "forest");
        out.push(b'L');
        out.extend_from_slice(&[200, 150, 100, 255]);
        if version >= GAME_VERSION_VERSION {
            put_u32(&mut out, LUA);
        }

        if version >= REFORGED_VERSION {
            filler(&mut out, MODE_BYTES);
        }

        if version >= CAMERA_VERSION {
            filler(&mut out, CAMERA_DISTANCE_BYTES);
        }

        if version >= MIN_CAMERA_VERSION {
            filler(&mut out, MIN_CAMERA_BYTES);
        }

        if version >= PATCH_2_VERSION {
            filler(&mut out, PATCH_2_WATER_BYTES);
        }

        put_u32(&mut out, 2);
        for (number, controller, race, fixed, name) in [
            (0, USER, HUMAN, FIXED_START, "Blue"),
            (1, COMPUTER, ORC, LEFTOVER_TEXT, "Red"),
        ] {
            put_u32(&mut out, number);
            put_u32(&mut out, controller);
            put_u32(&mut out, race);
            if hud_skins {
                put_u32(&mut out, 0);
            }

            put_u32(&mut out, fixed);
            put_text(&mut out, name);
            put_f32(&mut out, -1216.0);
            put_f32(&mut out, 576.0);
            put_u32(&mut out, 0);
            put_u32(&mut out, 0);
            if version >= REFORGED_VERSION {
                put_u32(&mut out, 0);
                put_u32(&mut out, 0);
            }
        }

        put_u32(&mut out, 1);
        put_u32(&mut out, FORCE_FLAGS);
        put_u32(&mut out, ALL_PLAYERS);
        put_text(&mut out, "Force 1");
        for _ in 0..3 {
            put_u32(&mut out, 0);
        }

        put_u32(&mut out, 1);
        put_u32(&mut out, 0);
        put_text(&mut out, "Loot");
        put_u32(&mut out, 1);
        put_u32(&mut out, 1);
        put_u32(&mut out, 100);
        out.extend_from_slice(b"phea");
        out
    }

    fn properties(info: &MapInfo) -> &Properties {
        info.properties().unwrap()
    }

    #[test]
    fn every_version_round_trips_byte_for_byte() {
        for version in VERSIONS {
            let bytes = w3i(version, version >= PATCH_2_VERSION);
            let info = MapInfo::parse(&bytes).unwrap();
            let p = properties(&info);

            assert_eq!(info.to_bytes(), bytes, "version {version}");
            assert_eq!(p.version(), version);
            assert_eq!(p.name, "Sunwell");
            assert_eq!(p.loading_screen.model, "Load.mdx");
            assert_eq!(p.prologue.title, "Prologue title");
            assert_eq!(p.weather, FourCC(*b"RAhr"));
            assert_eq!(p.sound_environment, "forest");
            assert_eq!(p.players.len(), 2);
            assert_eq!(p.forces[0].name, "Force 1");
            assert_eq!(info.tables()[0].sets[0][0].item, FourCC(*b"phea"));
        }
    }

    #[test]
    fn fields_are_read_from_the_versions_that_have_them() {
        let read = |version| MapInfo::parse(&w3i(version, version >= PATCH_2_VERSION)).unwrap();
        let tft = read(TFT_VERSION);
        let reforged = read(REFORGED_VERSION);
        let patch_2 = read(PATCH_2_VERSION);

        assert!(!properties(&tft).uses_lua());
        assert!(properties(&read(GAME_VERSION_VERSION)).uses_lua());
        assert_eq!(properties(&tft).players[0].enemy_priorities(), None);
        assert_eq!(
            properties(&reforged).players[0].enemy_priorities(),
            Some([0, 0])
        );
        assert_eq!(properties(&reforged).fog_extra(), None);
        assert_eq!(
            properties(&patch_2).fog_extra(),
            Some(([0.1, 0.2, 0.3, 0.4], 0.9))
        );
        assert!(properties(&patch_2).fog_over_sky());
        assert_eq!(properties(&patch_2).players[0].hud_skin(), Some(0));
        assert_eq!(properties(&patch_2).players[1].start(), [-1216.0, 576.0]);
    }

    #[test]
    fn patch_2_maps_without_hud_skins_still_read() {
        let bytes = w3i(PATCH_2_VERSION, false);
        let info = MapInfo::parse(&bytes).unwrap();

        assert_eq!(info.to_bytes(), bytes);
        assert_eq!(properties(&info).players[0].hud_skin(), None);
        assert_eq!(properties(&info).players[1].name, "Red");
    }

    #[test]
    fn a_fixed_start_keeps_its_field_until_it_changes() {
        let mut info = MapInfo::parse(&w3i(PATCH_2_VERSION, true)).unwrap();

        assert!(properties(&info).players[0].fixed_start);
        assert!(
            properties(&info).players[1].fixed_start,
            "leftover text fixes the start"
        );

        let p = info.properties_mut().unwrap();

        p.players[0].fixed_start = false;
        p.players[1].race = HUMAN;

        let freed = MapInfo::parse(&info.to_bytes()).unwrap();

        assert!(!properties(&freed).players[0].fixed_start);
        assert!(properties(&freed).players[1].fixed_start);
        assert_eq!(properties(&freed).players[1].race, HUMAN);

        let mut fixed = freed.clone();

        fixed.properties_mut().unwrap().players[0].fixed_start = true;
        assert!(properties(&MapInfo::parse(&fixed.to_bytes()).unwrap()).players[0].fixed_start);
    }

    #[test]
    fn item_tables_change_and_older_maps_keep_their_bytes() {
        let mut info = MapInfo::parse(&w3i(TFT_VERSION, false)).unwrap();
        let table = ItemTable {
            number: 3,
            name: "Rare".to_owned(),
            sets: vec![vec![DropChance {
                item: FourCC(*b"ratf"),
                chance: 25,
            }]],
        };

        info.set_tables(vec![table.clone()]);
        assert_eq!(
            MapInfo::parse(&info.to_bytes()).unwrap().tables(),
            std::slice::from_ref(&table)
        );

        let mut old = PRE_TFT_VERSION.to_le_bytes().to_vec();

        old.extend_from_slice(b"anything at all");

        let mut before_tft = MapInfo::parse(&old).unwrap();

        assert!(before_tft.properties().is_none());
        assert!(!before_tft.supports_tables());
        before_tft.set_tables(vec![table]);
        assert_eq!(before_tft.to_bytes(), old);
    }
}
