use crate::binary::{Reader, put_count, put_i32, put_u32};
use crate::{Error, FourCC, Map, Result};

pub const PLACEMENTS_FILE: &str = "war3mapUnits.doo";

const MAGIC: &[u8; 4] = b"W3do";
// Subversion 11 adds the map item table link and hero attributes to each unit.
const TABLE_SUBVERSION: u32 = 11;
// Version 12 adds a group, roll and pitch, and lights to each unit.
const GROUP_VERSION: u32 = 12;
const NO_TABLE: i32 = -1;
const NO_GROUP: i32 = -1;
const MIN_UNIT_BYTES: usize = 4 + 4 + 12 + 4 + 12;
const LIGHT_BYTES: usize = 36;
const DROP_ENTRY_BYTES: usize = 8;
const INVENTORY_ENTRY_BYTES: usize = 8;
const ABILITY_BYTES: usize = 12;
const RANDOM_ENTRY_BYTES: usize = 8;
// How a placed unit is randomized: from a level and class, from a map unit
// table column, or from its own list; other values mean a fixed unit.
const RANDOM_BY_LEVEL: i32 = 0;
const RANDOM_FROM_TABLE: i32 = 1;
const RANDOM_FROM_LIST: i32 = 2;
const FIXED_UNIT: i32 = -1;
// Placement flags: 2 is what the World Editor writes for every placed unit.
const DEFAULT_FLAGS: u8 = 2;
const DEFAULT_LIFE: i32 = -1;
const DEFAULT_MANA: i32 = -1;
const DEFAULT_ACQUISITION: f32 = -1.0;
const NO_COLOR: i32 = -1;
const NO_WAYGATE: i32 = -1;

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
pub struct AbilityLevel {
    pub ability: FourCC,
    pub autocast: bool,
    pub level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drops {
    None,
    Table(i32),
    Sets(Vec<Vec<DropChance>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomUnit {
    Fixed(i32),
    ByLevel([u8; 4]),
    FromTable { group: u32, column: u32 },
    FromList(Vec<(FourCC, u32)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub id: FourCC,
    pub variation: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Facing in radians, as stored; scripts use degrees.
    pub angle: f32,
    pub scale: [f32; 3],
    pub skin: Option<FourCC>,
    group: Option<i32>,
    pub flags: u8,
    pub player: u32,
    unknown: [u8; 2],
    /// Life in percent of the maximum, or -1 for full.
    pub life: i32,
    /// Mana in points, or -1 for the unit's starting mana.
    pub mana: i32,
    item_table: Option<i32>,
    sets: Vec<Vec<DropChance>>,
    pub gold: u32,
    /// Acquisition range: -1 normal, -2 creep camp, or a range.
    pub acquisition: f32,
    pub level: u32,
    /// Hero strength, agility and intelligence; 0 keeps the hero's own.
    pub attributes: Option<[i32; 3]>,
    inventory: Vec<InventorySlot>,
    pub abilities: Vec<AbilityLevel>,
    pub random: RandomUnit,
    pub color: i32,
    pub waygate: i32,
    pub creation_number: u32,
    pub roll_pitch: Option<[f32; 2]>,
    lights: Option<Vec<u8>>,
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

    pub fn item_table(&self) -> Option<i32> {
        self.item_table.filter(|t| *t != NO_TABLE)
    }

    pub fn item_sets(&self) -> &[Vec<DropChance>] {
        &self.sets
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
        let count = r.count(MIN_UNIT_BYTES)?;
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

    pub fn has_table_links(&self) -> bool {
        self.subversion >= TABLE_SUBVERSION
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();

        put_u32(&mut out, self.version);
        put_u32(&mut out, self.subversion);
        put_count(&mut out, self.units.len());
        for unit in &self.units {
            write_unit(&mut out, unit);
        }

        out
    }

    pub fn unit_mut(&mut self, index: usize) -> Result<&mut Placement> {
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

    pub fn set_attributes(&mut self, index: usize, attributes: [i32; 3]) -> Result<()> {
        self.link_tables();
        self.unit_mut(index)?.attributes = Some(attributes);
        Ok(())
    }

    fn link_tables(&mut self) {
        if self.subversion >= TABLE_SUBVERSION {
            return;
        }

        self.subversion = TABLE_SUBVERSION;
        for unit in &mut self.units {
            unit.item_table = Some(NO_TABLE);
            unit.attributes = Some([0; 3]);
        }
    }

    pub fn tables_removed(&mut self, removed: i32) {
        for unit in &mut self.units {
            if unit.item_table == Some(removed) {
                unit.item_table = Some(NO_TABLE);
            }
        }
    }

    /// Adds a unit as the World Editor places one: facing 270°, full life and
    /// mana, the next creation number. Returns its index.
    pub fn add(&mut self, id: FourCC, player: u32, x: f32, y: f32, z: f32) -> usize {
        let creation_number = self
            .units
            .iter()
            .map(|u| u.creation_number + 1)
            .max()
            .unwrap_or_default();
        let tables = self.subversion >= TABLE_SUBVERSION;
        let grouped = self.version >= GROUP_VERSION;

        self.units.push(Placement {
            id,
            variation: 0,
            x,
            y,
            z,
            angle: std::f32::consts::FRAC_PI_2 * 3.0,
            scale: [1.0; 3],
            skin: self.skins.then_some(id),
            group: grouped.then_some(NO_GROUP),
            flags: DEFAULT_FLAGS,
            player,
            unknown: [0; 2],
            life: DEFAULT_LIFE,
            mana: DEFAULT_MANA,
            item_table: tables.then_some(NO_TABLE),
            sets: Vec::new(),
            gold: 0,
            acquisition: DEFAULT_ACQUISITION,
            level: 1,
            attributes: tables.then_some([0; 3]),
            inventory: Vec::new(),
            abilities: Vec::new(),
            random: RandomUnit::Fixed(FIXED_UNIT),
            color: NO_COLOR,
            waygate: NO_WAYGATE,
            creation_number,
            roll_pitch: grouped.then_some([0.0; 2]),
            lights: grouped.then(|| 0u32.to_le_bytes().to_vec()),
        });

        self.units.len() - 1
    }

    pub fn remove(&mut self, index: usize) -> Result<Placement> {
        if index >= self.units.len() {
            return Err(Error::Invalid(format!("No placed unit number {index}.")));
        }

        Ok(self.units.remove(index))
    }
}

fn parse_unit(r: &mut Reader<'_>, layout: &Layout) -> Result<Placement> {
    let id = r.fourcc()?;
    let variation = r.u32()?;
    let (x, y, z) = (r.f32()?, r.f32()?, r.f32()?);
    let angle = r.f32()?;
    let scale = [r.f32()?, r.f32()?, r.f32()?];
    let skin = if layout.skins {
        Some(r.fourcc()?)
    } else {
        None
    };
    let group = if layout.version >= GROUP_VERSION {
        Some(r.i32()?)
    } else {
        None
    };
    let flags = r.u8()?;
    let player = r.u32()?;
    let unknown = [r.u8()?, r.u8()?];
    let life = r.i32()?;
    let mana = r.i32()?;
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

    let gold = r.u32()?;
    let acquisition = r.f32()?;
    let level = r.u32()?;
    let attributes = if layout.subversion >= TABLE_SUBVERSION {
        Some([r.i32()?, r.i32()?, r.i32()?])
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

    let n = r.count(ABILITY_BYTES)?;
    let mut abilities = Vec::with_capacity(n);

    for _ in 0..n {
        abilities.push(AbilityLevel {
            ability: r.fourcc()?,
            autocast: r.u32()? != 0,
            level: r.u32()?,
        });
    }

    let random = match r.i32()? {
        RANDOM_BY_LEVEL => {
            let bytes = r.take(4)?;

            RandomUnit::ByLevel([bytes[0], bytes[1], bytes[2], bytes[3]])
        }
        RANDOM_FROM_TABLE => RandomUnit::FromTable {
            group: r.u32()?,
            column: r.u32()?,
        },
        RANDOM_FROM_LIST => {
            let n = r.count(RANDOM_ENTRY_BYTES)?;
            let mut list = Vec::with_capacity(n);

            for _ in 0..n {
                list.push((r.fourcc()?, r.u32()?));
            }

            RandomUnit::FromList(list)
        }
        other => RandomUnit::Fixed(other),
    };
    let color = r.i32()?;
    let waygate = r.i32()?;
    let creation_number = r.u32()?;
    let (roll_pitch, lights) = if layout.version >= GROUP_VERSION {
        let roll_pitch = [r.f32()?, r.f32()?];
        let start = r.pos();
        let count = r.count(LIGHT_BYTES)?;

        r.take(count * LIGHT_BYTES)?;
        (Some(roll_pitch), Some(r.since(start)))
    } else {
        (None, None)
    };

    Ok(Placement {
        id,
        variation,
        x,
        y,
        z,
        angle,
        scale,
        skin,
        group,
        flags,
        player,
        unknown,
        life,
        mana,
        item_table,
        sets,
        gold,
        acquisition,
        level,
        attributes,
        inventory,
        abilities,
        random,
        color,
        waygate,
        creation_number,
        roll_pitch,
        lights,
    })
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_unit(out: &mut Vec<u8>, unit: &Placement) {
    out.extend_from_slice(&unit.id.0);
    put_u32(out, unit.variation);
    for value in [unit.x, unit.y, unit.z, unit.angle] {
        put_f32(out, value);
    }

    for value in unit.scale {
        put_f32(out, value);
    }

    if let Some(skin) = unit.skin {
        out.extend_from_slice(&skin.0);
    }

    if let Some(group) = unit.group {
        put_i32(out, group);
    }

    out.push(unit.flags);
    put_u32(out, unit.player);
    out.extend_from_slice(&unit.unknown);
    put_i32(out, unit.life);
    put_i32(out, unit.mana);
    if let Some(table) = unit.item_table {
        put_i32(out, table);
    }

    put_count(out, unit.sets.len());
    for set in &unit.sets {
        put_count(out, set.len());
        for drop in set {
            out.extend_from_slice(&drop.item.0);
            put_u32(out, drop.chance);
        }
    }

    put_u32(out, unit.gold);
    put_f32(out, unit.acquisition);
    put_u32(out, unit.level);
    if let Some(attributes) = unit.attributes {
        for value in attributes {
            put_i32(out, value);
        }
    }

    put_count(out, unit.inventory.len());
    for slot in &unit.inventory {
        put_u32(out, slot.slot);
        out.extend_from_slice(&slot.item.0);
    }

    put_count(out, unit.abilities.len());
    for ability in &unit.abilities {
        out.extend_from_slice(&ability.ability.0);
        put_u32(out, u32::from(ability.autocast));
        put_u32(out, ability.level);
    }

    match &unit.random {
        RandomUnit::Fixed(kind) => put_i32(out, *kind),
        RandomUnit::ByLevel(bytes) => {
            put_i32(out, RANDOM_BY_LEVEL);
            out.extend_from_slice(bytes);
        }
        RandomUnit::FromTable { group, column } => {
            put_i32(out, RANDOM_FROM_TABLE);
            put_u32(out, *group);
            put_u32(out, *column);
        }
        RandomUnit::FromList(list) => {
            put_i32(out, RANDOM_FROM_LIST);
            put_count(out, list.len());
            for (id, chance) in list {
                out.extend_from_slice(&id.0);
                put_u32(out, *chance);
            }
        }
    }

    put_i32(out, unit.color);
    put_i32(out, unit.waygate);
    put_u32(out, unit.creation_number);
    if let Some([roll, pitch]) = unit.roll_pitch {
        put_f32(out, roll);
        put_f32(out, pitch);
    }

    if let Some(lights) = &unit.lights {
        out.extend_from_slice(lights);
    }
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
    const ATTRIBUTE_BYTES: usize = 12;

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
            out.extend_from_slice(&[0; 8]);
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
