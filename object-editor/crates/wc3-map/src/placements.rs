use crate::binary::{Reader, put_count, put_i32, put_u32};
use crate::{Error, FourCC, Map, Result};

pub const PLACEMENTS_FILE: &str = "war3mapUnits.doo";

const MAGIC: &[u8; 4] = b"W3do";
// Subversion 11 adds the map item table link and hero attributes to each unit.
const TABLE_SUBVERSION: u32 = 11;
// Version 12 adds a group, roll and pitch, and lights to each unit.
const GROUP_VERSION: u32 = 12;
const NO_TABLE: i32 = -1;
const POSITION_BYTES: usize = 4 + 12 + 4 + 12;
const PLAYER_BYTES: usize = 2 + 4 + 4;
const ATTRIBUTE_BYTES: usize = 12;
const ABILITY_BYTES: usize = 12;
const RANDOM_ENTRY_BYTES: usize = 8;
const ROLL_PITCH_BYTES: usize = 8;
const LIGHT_BYTES: usize = 36;
const DROP_ENTRY_BYTES: usize = 8;
const INVENTORY_ENTRY_BYTES: usize = 8;
// How a placed unit is randomized: from a level and class, from a map unit
// table column, or from its own list; other values mean a fixed unit.
const RANDOM_BY_LEVEL: i32 = 0;
const RANDOM_FROM_TABLE: i32 = 1;
const RANDOM_FROM_LIST: i32 = 2;
const RANDOM_BY_LEVEL_BYTES: usize = 4;
const RANDOM_FROM_TABLE_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropChance {
    pub item: FourCC,
    pub chance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventorySlot {
    pub slot: u32,
    pub item: FourCC,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drops {
    None,
    Table(i32),
    Sets(Vec<Vec<DropChance>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub id: FourCC,
    pub x: f32,
    pub y: f32,
    pub player: u32,
    pub level: u32,
    pub creation_number: u32,
    item_table: Option<i32>,
    sets: Vec<Vec<DropChance>>,
    inventory: Vec<InventorySlot>,
    head: Vec<u8>,
    middle: Vec<u8>,
    attributes: Option<Vec<u8>>,
    tail: Vec<u8>,
}

impl Placement {
    pub fn inventory(&self) -> &[InventorySlot] {
        &self.inventory
    }

    pub fn drops(&self) -> Drops {
        match self.item_table {
            Some(table) if table != NO_TABLE => Drops::Table(table),
            _ if !self.sets.is_empty() => Drops::Sets(self.sets.clone()),
            _ => Drops::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPlacements {
    version: u32,
    subversion: u32,
    skins: bool,
    units: Vec<Placement>,
}

struct Layout {
    version: u32,
    subversion: u32,
    // Reforged (1.32) adds a skin ID after each unit's scale.
    skins: bool,
}

impl UnitPlacements {
    pub fn load(map: &Map) -> Result<Option<Self>> {
        map.read_file(PLACEMENTS_FILE)?
            .map(|bytes| Self::parse(&bytes))
            .transpose()
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut header = Reader::new(PLACEMENTS_FILE, data);

        if header.take(4)? != MAGIC {
            return Err(header.damaged("not a unit placement file"));
        }

        let version = header.u32()?;
        let subversion = header.u32()?;
        let with_skins = Layout {
            version,
            subversion,
            skins: true,
        };

        match Self::parse_units(data, &with_skins) {
            Ok(units) => Ok(Self::from_units(&with_skins, units)),
            Err(skin_error) => {
                let without = Layout {
                    skins: false,
                    ..with_skins
                };

                Self::parse_units(data, &without)
                    .map(|units| Self::from_units(&without, units))
                    .map_err(|_| skin_error)
            }
        }
    }

    fn from_units(layout: &Layout, units: Vec<Placement>) -> Self {
        Self {
            version: layout.version,
            subversion: layout.subversion,
            skins: layout.skins,
            units,
        }
    }

    fn parse_units(data: &[u8], layout: &Layout) -> Result<Vec<Placement>> {
        let mut r = Reader::new(PLACEMENTS_FILE, data);

        r.take(12)?;
        let count = r.count(POSITION_BYTES)?;
        let mut units = Vec::with_capacity(count);

        for _ in 0..count {
            units.push(parse_unit(&mut r, layout)?);
        }

        if r.remaining() != 0 {
            return Err(r.damaged("unexpected data after the last unit"));
        }

        Ok(units)
    }

    pub fn units(&self) -> &[Placement] {
        &self.units
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();

        put_u32(&mut out, self.version);
        put_u32(&mut out, self.subversion);
        put_count(&mut out, self.units.len());
        for unit in &self.units {
            out.extend_from_slice(&unit.head);
            if let Some(table) = unit.item_table {
                put_i32(&mut out, table);
            }

            put_count(&mut out, unit.sets.len());
            for set in &unit.sets {
                put_count(&mut out, set.len());
                for drop in set {
                    out.extend_from_slice(&drop.item.0);
                    put_u32(&mut out, drop.chance);
                }
            }

            out.extend_from_slice(&unit.middle);
            if let Some(attributes) = &unit.attributes {
                out.extend_from_slice(attributes);
            }

            put_count(&mut out, unit.inventory.len());
            for slot in &unit.inventory {
                put_u32(&mut out, slot.slot);
                out.extend_from_slice(&slot.item.0);
            }

            out.extend_from_slice(&unit.tail);
        }

        out
    }

    fn unit_mut(&mut self, index: usize) -> Result<&mut Placement> {
        self.units
            .get_mut(index)
            .ok_or_else(|| Error::Invalid(format!("No placed unit number {index}.")))
    }

    pub fn set_inventory(&mut self, index: usize, inventory: Vec<InventorySlot>) -> Result<()> {
        self.unit_mut(index)?.inventory = inventory;
        Ok(())
    }

    pub fn set_drops(&mut self, index: usize, drops: Drops) -> Result<()> {
        if matches!(drops, Drops::Table(_)) {
            self.link_tables();
        }

        let unit = self.unit_mut(index)?;
        let (table, sets) = match drops {
            Drops::None => (NO_TABLE, Vec::new()),
            Drops::Table(table) => (table, Vec::new()),
            Drops::Sets(sets) => (NO_TABLE, sets),
        };

        if unit.item_table.is_some() {
            unit.item_table = Some(table);
        }

        unit.sets = sets;
        Ok(())
    }

    fn link_tables(&mut self) {
        if self.subversion >= TABLE_SUBVERSION {
            return;
        }

        self.subversion = TABLE_SUBVERSION;
        for unit in &mut self.units {
            unit.item_table = Some(NO_TABLE);
            unit.attributes = Some(vec![0; ATTRIBUTE_BYTES]);
        }
    }

    pub fn tables_removed(&mut self, removed: i32) {
        for unit in &mut self.units {
            if unit.item_table == Some(removed) {
                unit.item_table = Some(NO_TABLE);
            }
        }
    }
}

fn parse_unit(r: &mut Reader<'_>, layout: &Layout) -> Result<Placement> {
    let start = r.pos();
    let id = r.fourcc()?;

    r.u32()?;
    let x = r.f32()?;
    let y = r.f32()?;

    r.take(POSITION_BYTES - 12)?;
    if layout.skins {
        r.fourcc()?;
    }

    if layout.version >= GROUP_VERSION {
        r.u32()?;
    }

    r.u8()?;
    let player = r.u32()?;

    r.take(PLAYER_BYTES)?;
    let head = r.since(start);
    let item_table = if layout.subversion >= TABLE_SUBVERSION {
        Some(r.i32()?)
    } else {
        None
    };
    let set_count = r.count(4)?;
    let mut sets = Vec::with_capacity(set_count);

    for _ in 0..set_count {
        let n = r.count(DROP_ENTRY_BYTES)?;
        let mut set = Vec::with_capacity(n);

        for _ in 0..n {
            set.push(DropChance {
                item: r.fourcc()?,
                chance: r.u32()?,
            });
        }

        sets.push(set);
    }

    let middle_start = r.pos();

    r.u32()?;
    r.f32()?;
    let level = r.u32()?;
    let middle = r.since(middle_start);
    let attributes = if layout.subversion >= TABLE_SUBVERSION {
        Some(r.take(ATTRIBUTE_BYTES)?.to_vec())
    } else {
        None
    };
    let n = r.count(INVENTORY_ENTRY_BYTES)?;
    let mut inventory = Vec::with_capacity(n);

    for _ in 0..n {
        inventory.push(InventorySlot {
            slot: r.u32()?,
            item: r.fourcc()?,
        });
    }

    let tail_start = r.pos();
    let abilities = r.count(ABILITY_BYTES)?;

    r.take(abilities * ABILITY_BYTES)?;
    match r.i32()? {
        RANDOM_BY_LEVEL => {
            r.take(RANDOM_BY_LEVEL_BYTES)?;
        }
        RANDOM_FROM_TABLE => {
            r.take(RANDOM_FROM_TABLE_BYTES)?;
        }
        RANDOM_FROM_LIST => {
            let n = r.count(RANDOM_ENTRY_BYTES)?;

            r.take(n * RANDOM_ENTRY_BYTES)?;
        }
        _ => {}
    }

    r.u32()?;
    r.u32()?;
    let creation_number = r.u32()?;

    if layout.version >= GROUP_VERSION {
        r.take(ROLL_PITCH_BYTES)?;
        let lights = r.count(LIGHT_BYTES)?;

        r.take(lights * LIGHT_BYTES)?;
    }

    let tail = r.since(tail_start);

    Ok(Placement {
        id,
        x,
        y,
        player,
        level,
        creation_number,
        item_table,
        sets,
        inventory,
        head,
        middle,
        attributes,
        tail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    const HERO: FourCC = FourCC(*b"Hpal");
    const POTION: FourCC = FourCC(*b"phea");
    const CLAWS: FourCC = FourCC(*b"ratc");
    const NO_RANDOM: i32 = -1;
    const HOSTILE: u32 = 24;

    fn unit(version: u32, subversion: u32) -> Vec<u8> {
        let mut out = HERO.0.to_vec();

        put_u32(&mut out, 0);
        for value in [128.0f32, -256.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
            out.extend_from_slice(&value.to_le_bytes());
        }

        out.extend_from_slice(&HERO.0);
        if version >= GROUP_VERSION {
            put_i32(&mut out, -1);
        }

        out.push(2);
        put_u32(&mut out, HOSTILE);
        out.extend_from_slice(&[0, 0]);
        put_i32(&mut out, -1);
        put_i32(&mut out, -1);
        if subversion >= TABLE_SUBVERSION {
            put_i32(&mut out, NO_TABLE);
        }

        put_u32(&mut out, 1);
        put_u32(&mut out, 1);
        out.extend_from_slice(&CLAWS.0);
        put_u32(&mut out, 40);
        put_u32(&mut out, 0);
        out.extend_from_slice(&200.0f32.to_le_bytes());
        put_u32(&mut out, 3);
        if subversion >= TABLE_SUBVERSION {
            out.extend_from_slice(&[0; ATTRIBUTE_BYTES]);
        }

        put_u32(&mut out, 1);
        put_u32(&mut out, 0);
        out.extend_from_slice(&POTION.0);
        put_u32(&mut out, 0);
        put_i32(&mut out, NO_RANDOM);
        put_i32(&mut out, -1);
        put_i32(&mut out, -1);
        put_u32(&mut out, 7);
        if version >= GROUP_VERSION {
            out.extend_from_slice(&[0; ROLL_PITCH_BYTES]);
            put_u32(&mut out, 0);
        }

        out
    }

    fn file(version: u32, subversion: u32) -> Vec<u8> {
        let mut out = MAGIC.to_vec();

        put_u32(&mut out, version);
        put_u32(&mut out, subversion);
        put_u32(&mut out, 1);
        out.extend_from_slice(&unit(version, subversion));
        out
    }

    #[test]
    fn every_layout_round_trips_and_reads_inventory_and_drops() -> TestResult {
        for (version, subversion) in [(8, 11), (13, 9), (13, 11)] {
            let data = file(version, subversion);
            let placements = UnitPlacements::parse(&data)?;
            let hero = &placements.units()[0];

            assert_eq!(placements.to_bytes(), data);
            assert_eq!((hero.id, hero.player, hero.level), (HERO, HOSTILE, 3));
            assert_eq!(hero.creation_number, 7);
            assert_eq!(hero.inventory()[0].item, POTION);
            assert_eq!(
                hero.drops(),
                Drops::Sets(vec![vec![DropChance {
                    item: CLAWS,
                    chance: 40
                }]])
            );
        }

        Ok(())
    }

    #[test]
    fn linking_a_table_adds_the_table_fields_to_older_subversions() -> TestResult {
        let mut placements = UnitPlacements::parse(&file(13, 9))?;

        placements.set_drops(0, Drops::Table(2))?;
        let written = placements.to_bytes();
        let reread = UnitPlacements::parse(&written)?;

        assert_eq!(reread.units()[0].drops(), Drops::Table(2));
        assert_eq!(reread.units()[0].inventory()[0].item, POTION);
        assert_eq!(&written[8..12], &TABLE_SUBVERSION.to_le_bytes());
        Ok(())
    }

    #[test]
    fn removing_a_table_unlinks_its_units() -> TestResult {
        let mut placements = UnitPlacements::parse(&file(8, 11))?;

        placements.set_drops(0, Drops::Table(1))?;
        placements.tables_removed(1);
        placements.set_inventory(0, Vec::new())?;
        let reread = UnitPlacements::parse(&placements.to_bytes())?;

        assert_eq!(reread.units()[0].drops(), Drops::None);
        assert!(reread.units()[0].inventory().is_empty());
        Ok(())
    }
}
