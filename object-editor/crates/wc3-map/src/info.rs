use crate::binary::{Reader, put_count, put_u32};
use crate::placements::DropChance;
use crate::{Map, Result};

pub const INFO_FILE: &str = "war3map.w3i";

// War3map.w3i versions: 25 is The Frozen Throne (the first with item tables),
// 28 adds game versions and Lua, 31 Reforged's modes and enemy priorities,
// 32 and 33 camera distances, 39 (patch 2.0) loading screen sources, fog,
// water and player HUD skins.
const ITEM_TABLE_VERSION: u32 = 25;
const GAME_VERSION_VERSION: u32 = 28;
const REFORGED_VERSION: u32 = 31;
const CAMERA_VERSION: u32 = 32;
const MIN_CAMERA_VERSION: u32 = 33;
const PATCH_2_VERSION: u32 = 39;
const CAMERA_BOUNDS_BYTES: usize = 8 * 4 + 4 * 4;
const PLAYABLE_SIZE_BYTES: usize = 8;
const TILESET_BYTES: usize = 1;
const FOG_BYTES: usize = 4 * 3 + 4;
const PATCH_2_FOG_BYTES: usize = 4 * 6;
const LIGHT_AND_WATER_BYTES: usize = 1 + 4;
const PATCH_2_WATER_BYTES: usize = 4 * 10;
const PLAYER_POSITION_BYTES: usize = 8;
const PRIORITY_BYTES: usize = 8;
const UPGRADE_BYTES: usize = 16;
const TECH_BYTES: usize = 8;
const UNIT_TABLE_POSITION_BYTES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTable {
    pub number: u32,
    pub name: String,
    pub sets: Vec<Vec<DropChance>>,
}

/// The parts of `war3map.w3i` the editor changes: its item tables. The rest
/// is kept byte for byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapInfo {
    before: Vec<u8>,
    tables: Option<Vec<ItemTable>>,
    after: Vec<u8>,
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

        if version < ITEM_TABLE_VERSION {
            return Ok(Self::without_tables(data));
        }

        let hud_skins = version >= PATCH_2_VERSION;

        // A few of the game's own patch 2.0 maps leave out the players' HUD skins.
        Self::parse_tables(data, version, hud_skins)
            .or_else(|error| Self::parse_tables(data, version, !hud_skins).map_err(|_| error))
    }

    fn without_tables(data: &[u8]) -> Self {
        Self {
            before: data.to_vec(),
            tables: None,
            after: Vec::new(),
        }
    }

    fn parse_tables(data: &[u8], version: u32, hud_skins: bool) -> Result<Self> {
        let mut r = Reader::new(INFO_FILE, data);

        r.u32()?;
        skip_to_tables(&mut r, version, hud_skins)?;
        let before = data[..r.pos()].to_vec();

        if r.remaining() < 4 {
            return Ok(Self::without_tables(data));
        }

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

        let after = r.take(r.remaining())?.to_vec();

        Ok(Self {
            before,
            tables: Some(tables),
            after,
        })
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
        let mut out = self.before.clone();

        if let Some(tables) = &self.tables {
            put_count(&mut out, tables.len());
            for table in tables {
                put_u32(&mut out, table.number);
                out.extend_from_slice(table.name.as_bytes());
                out.push(0);
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

fn skip_strings(r: &mut Reader<'_>, n: usize) -> Result<()> {
    for _ in 0..n {
        r.c_string()?;
    }

    Ok(())
}

fn skip_to_tables(r: &mut Reader<'_>, version: u32, hud_skins: bool) -> Result<()> {
    r.take(8)?;
    if version >= GAME_VERSION_VERSION {
        r.take(16)?;
    }

    skip_strings(r, 4)?;
    r.take(CAMERA_BOUNDS_BYTES + PLAYABLE_SIZE_BYTES)?;
    r.u32()?;
    r.take(TILESET_BYTES)?;
    r.u32()?;
    if version >= PATCH_2_VERSION {
        r.u32()?;
    }

    skip_strings(r, 4)?;
    r.u32()?;
    skip_strings(r, 4)?;
    r.u32()?;
    r.take(FOG_BYTES)?;
    r.u32()?;
    if version >= PATCH_2_VERSION {
        r.take(PATCH_2_FOG_BYTES)?;
    }

    skip_strings(r, 1)?;
    r.take(LIGHT_AND_WATER_BYTES)?;
    if version >= GAME_VERSION_VERSION {
        r.u32()?;
    }

    if version >= REFORGED_VERSION {
        r.take(8)?;
    }

    if version >= CAMERA_VERSION {
        r.take(8)?;
    }

    if version >= MIN_CAMERA_VERSION {
        r.u32()?;
    }

    if version >= PATCH_2_VERSION {
        r.take(PATCH_2_WATER_BYTES)?;
    }

    for _ in 0..r.count(4)? {
        r.take(12)?;
        if hud_skins {
            r.u32()?;
        }

        r.u32()?;
        skip_strings(r, 1)?;
        r.take(PLAYER_POSITION_BYTES + PRIORITY_BYTES)?;
        if version >= REFORGED_VERSION {
            r.take(PRIORITY_BYTES)?;
        }
    }

    for _ in 0..r.count(4)? {
        r.take(8)?;
        skip_strings(r, 1)?;
    }

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
        skip_strings(r, 1)?;
        let positions = r.count(UNIT_TABLE_POSITION_BYTES)?;

        r.take(positions * UNIT_TABLE_POSITION_BYTES)?;
        for _ in 0..r.count(4)? {
            r.u32()?;
            r.take(positions * 4)?;
        }
    }

    Ok(())
}
