//! Object Editor data in maps: `war3map.w3u` (units), `.w3t` (items),
//! `.w3b` (destructibles), `.w3d` (doodads), `.w3a` (abilities), `.w3h` (buffs),
//! `.w3q` (upgrades), and their Reforged `war3mapSkin.*` counterparts.
//!
//! Every file has the same layout: a version, then a table of changed original
//! objects and a table of custom objects. Each object lists field modifications.
//! Version 3 (Reforged 2.0) groups modifications into sets with a flags word.
//! Doodads, abilities and upgrades also store a level and a data column per
//! modification, after the value type:
//!
//! ```text
//! field: [u8; 4], type: u32, (level: u32, column: u32), value, end: [u8; 4]
//! ```
//!
//! Parsing keeps everything needed to write the file back byte for byte.

use std::fmt;

use crate::{Error, Result};

const SETS_VERSION: u32 = 3;
const TYPE_INT: u32 = 0;
const TYPE_REAL: u32 = 1;
const TYPE_UNREAL: u32 = 2;
const TYPE_STRING: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    pub const ZERO: Self = Self([0; 4]);

    pub fn parse(code: &str) -> Option<Self> {
        let bytes: [u8; 4] = code.as_bytes().try_into().ok()?;

        Some(Self(bytes))
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.0))
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Which file family is being read: doodads, abilities and upgrades store a
/// level and data column with every modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Plain,
    Leveled,
}

impl Layout {
    pub fn for_file(name: &str) -> Option<Self> {
        let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();

        match ext.as_str() {
            "w3u" | "w3t" | "w3b" | "w3h" => Some(Self::Plain),
            "w3d" | "w3a" | "w3q" => Some(Self::Leveled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Real(f32),
    /// A real the editor keeps non-negative.
    Unreal(f32),
    String(String),
}

impl Value {
    fn type_code(&self) -> u32 {
        match self {
            Self::Int(_) => TYPE_INT,
            Self::Real(_) => TYPE_REAL,
            Self::Unreal(_) => TYPE_UNREAL,
            Self::String(_) => TYPE_STRING,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Real(v) | Self::Unreal(v) => write!(f, "{v}"),
            Self::String(v) => f.write_str(v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modification {
    pub field: FourCC,
    /// Level (abilities, upgrades) or variation (doodads); `None` for plain files.
    pub level: Option<u32>,
    /// Data column (`DataA` = 1, ...) for leveled files; `None` for plain files.
    pub column: Option<u32>,
    pub value: Value,
    /// Trailing marker the editor writes after each value (0 or an object ID).
    pub end: FourCC,
}

/// A group of modifications. Version 3 files can hold several per object.
#[derive(Debug, Clone, PartialEq)]
pub struct ModSet {
    pub flags: u32,
    pub mods: Vec<Modification>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectEntry {
    /// The object it is based on (or the changed original object).
    pub base: FourCC,
    /// The custom object's own ID, or zero in the original-objects table.
    pub id: FourCC,
    pub sets: Vec<ModSet>,
}

impl ObjectEntry {
    pub fn effective_id(&self) -> FourCC {
        if self.id.is_zero() {
            self.base
        } else {
            self.id
        }
    }

    pub fn mods(&self) -> impl Iterator<Item = &Modification> {
        self.sets.iter().flat_map(|s| s.mods.iter())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectData {
    pub version: u32,
    pub layout: Layout,
    pub original: Vec<ObjectEntry>,
    pub custom: Vec<ObjectEntry>,
}

impl ObjectData {
    pub fn empty(layout: Layout) -> Self {
        Self {
            version: SETS_VERSION,
            layout,
            original: Vec::new(),
            custom: Vec::new(),
        }
    }

    pub fn parse(data: &[u8], layout: Layout) -> Result<Self> {
        let mut reader = Reader { data, pos: 0 };
        let version = reader.u32()?;
        let original = reader.table(version, layout)?;
        let custom = reader.table(version, layout)?;

        if reader.pos != data.len() {
            return Err(reader.error(format!(
                "{} unexpected trailing bytes",
                data.len() - reader.pos
            )));
        }

        Ok(Self {
            version,
            layout,
            original,
            custom,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend_from_slice(&self.version.to_le_bytes());
        self.write_table(&mut out, &self.original);
        self.write_table(&mut out, &self.custom);
        out
    }

    fn write_table(&self, out: &mut Vec<u8>, entries: &[ObjectEntry]) {
        put_len(out, entries.len());

        for entry in entries {
            out.extend_from_slice(&entry.base.0);
            out.extend_from_slice(&entry.id.0);

            if self.version >= SETS_VERSION {
                put_len(out, entry.sets.len());
                for set in &entry.sets {
                    out.extend_from_slice(&set.flags.to_le_bytes());
                    self.write_mods(out, &set.mods);
                }
            } else {
                let mods: Vec<Modification> = entry.mods().cloned().collect();

                self.write_mods(out, &mods);
            }
        }
    }

    fn write_mods(&self, out: &mut Vec<u8>, mods: &[Modification]) {
        put_len(out, mods.len());

        for m in mods {
            out.extend_from_slice(&m.field.0);
            out.extend_from_slice(&m.value.type_code().to_le_bytes());
            if self.layout == Layout::Leveled {
                out.extend_from_slice(&m.level.unwrap_or_default().to_le_bytes());
                out.extend_from_slice(&m.column.unwrap_or_default().to_le_bytes());
            }

            match &m.value {
                Value::Int(v) => out.extend_from_slice(&v.to_le_bytes()),
                Value::Real(v) | Value::Unreal(v) => out.extend_from_slice(&v.to_le_bytes()),
                Value::String(v) => {
                    out.extend_from_slice(v.as_bytes());
                    out.push(0);
                }
            }

            out.extend_from_slice(&m.end.0);
        }
    }

    pub fn find(&self, id: FourCC) -> Option<&ObjectEntry> {
        self.custom
            .iter()
            .chain(&self.original)
            .find(|e| e.effective_id() == id)
    }
}

fn put_len(out: &mut Vec<u8>, len: usize) {
    // Counts in these files are 32-bit; maps never come close to the limit.
    let len = u32::try_from(len).unwrap_or(u32::MAX);

    out.extend_from_slice(&len.to_le_bytes());
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn error(&self, reason: String) -> Error {
        Error::Parse {
            line: 0,
            reason: format!("at byte {}: {reason}", self.pos),
        }
    }

    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len());
        let Some(end) = end else {
            return Err(self.error(format!("needs {n} more bytes, file ends")));
        };

        let bytes = &self.data[self.pos..end];

        self.pos = end;
        Ok(bytes)
    }

    fn u32(&mut self) -> Result<u32> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    fn fourcc(&mut self) -> Result<FourCC> {
        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(self.take(4)?);
        Ok(FourCC(bytes))
    }

    fn string(&mut self) -> Result<String> {
        let rest = &self.data[self.pos..];
        let Some(len) = rest.iter().position(|&b| b == 0) else {
            return Err(self.error("unterminated string".to_owned()));
        };

        let text = std::str::from_utf8(&rest[..len])
            .map_err(|e| self.error(format!("string is not UTF-8: {e}")))?
            .to_owned();

        self.pos += len + 1;
        Ok(text)
    }

    fn count(&mut self) -> Result<usize> {
        let n = self.u32()? as usize;

        // Every entry takes at least 4 bytes, so a larger count means corrupt data.
        if n > (self.data.len() - self.pos) / 4 {
            return Err(self.error(format!("count {n} exceeds the remaining data")));
        }

        Ok(n)
    }

    fn table(&mut self, version: u32, layout: Layout) -> Result<Vec<ObjectEntry>> {
        let count = self.count()?;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let base = self.fourcc()?;
            let id = self.fourcc()?;
            let sets = if version >= SETS_VERSION {
                let set_count = self.count()?;
                let mut sets = Vec::with_capacity(set_count);
                for _ in 0..set_count {
                    let flags = self.u32()?;
                    sets.push(ModSet {
                        flags,
                        mods: self.mods(layout)?,
                    });
                }
                sets
            } else {
                vec![ModSet {
                    flags: 0,
                    mods: self.mods(layout)?,
                }]
            };

            entries.push(ObjectEntry { base, id, sets });
        }

        Ok(entries)
    }

    fn mods(&mut self, layout: Layout) -> Result<Vec<Modification>> {
        let count = self.count()?;
        let mut mods = Vec::with_capacity(count);

        for _ in 0..count {
            let field = self.fourcc()?;
            let type_code = self.u32()?;
            let (level, column) = match layout {
                Layout::Leveled => (Some(self.u32()?), Some(self.u32()?)),
                Layout::Plain => (None, None),
            };
            let value = match type_code {
                TYPE_INT => Value::Int(i32::from_le_bytes(self.u32()?.to_le_bytes())),
                TYPE_REAL => Value::Real(f32::from_bits(self.u32()?)),
                TYPE_UNREAL => Value::Unreal(f32::from_bits(self.u32()?)),
                TYPE_STRING => Value::String(self.string()?),
                other => {
                    return Err(self.error(format!("unknown value type {other} for field {field}")));
                }
            };
            let end = self.fourcc()?;

            mods.push(Modification {
                field,
                level,
                column,
                value,
                end,
            });
        }

        Ok(mods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `war3mapSkin.w3u` from the test map: custom hero `H000` based on the
    /// Paladin with its name set to string 7.
    const SKIN_W3U: &[u8] = b"\x03\0\0\0\0\0\0\0\x01\0\0\0HpalH000\x01\0\0\0\0\0\0\0\x01\0\0\0\
                              unam\x03\0\0\0TRIGSTR_007\0\0\0\0\0";

    #[test]
    fn reads_the_test_hero() {
        let data = ObjectData::parse(SKIN_W3U, Layout::Plain).unwrap();
        let hero = &data.custom[0];

        assert_eq!(data.version, 3);
        assert!(data.original.is_empty());
        assert_eq!(hero.base.to_string(), "Hpal");
        assert_eq!(hero.effective_id().to_string(), "H000");

        let name = hero.mods().next().unwrap();

        assert_eq!(name.field.to_string(), "unam");
        assert_eq!(name.value, Value::String("TRIGSTR_007".to_owned()));
    }

    #[test]
    fn writes_back_byte_for_byte() {
        let data = ObjectData::parse(SKIN_W3U, Layout::Plain).unwrap();

        assert_eq!(data.to_bytes(), SKIN_W3U);
    }

    /// The first two objects of a real `war3map.w3a` (Legion TD): `Sloa` with
    /// an empty string field, and `Awrs` with a level-1 cooldown of 0.0.
    const LEVELED_W3A: &[u8] = b"\x03\0\0\0\0\0\0\0\x02\0\0\0\
        Sloa\0\0\0\0\x01\0\0\0\0\0\0\0\x01\0\0\0\
        arac\x03\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
        Awrs\0\0\0\0\x01\0\0\0\0\0\0\0\x01\0\0\0\
        acdn\x02\0\0\0\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    #[test]
    fn reads_leveled_files_with_the_type_before_the_level() {
        let data = ObjectData::parse(LEVELED_W3A, Layout::Leveled).unwrap();
        let cooldown = data
            .find(FourCC::parse("Awrs").unwrap())
            .unwrap()
            .mods()
            .next()
            .unwrap();

        assert_eq!(cooldown.field.to_string(), "acdn");
        assert_eq!(cooldown.value, Value::Unreal(0.0));
        assert_eq!(cooldown.level, Some(1));
        assert_eq!(cooldown.column, Some(0));
        assert_eq!(data.to_bytes(), LEVELED_W3A);
    }

    #[test]
    fn round_trips_leveled_mods_and_old_versions() {
        let mut data = ObjectData::empty(Layout::Leveled);

        data.version = 2;
        data.custom.push(ObjectEntry {
            base: FourCC::parse("AHhb").unwrap(),
            id: FourCC::parse("A000").unwrap(),
            sets: vec![ModSet {
                flags: 0,
                mods: vec![Modification {
                    field: FourCC::parse("Hhb1").unwrap(),
                    level: Some(2),
                    column: Some(1),
                    value: Value::Unreal(400.0),
                    end: FourCC::ZERO,
                }],
            }],
        });

        let bytes = data.to_bytes();

        assert_eq!(ObjectData::parse(&bytes, Layout::Leveled).unwrap(), data);
    }

    #[test]
    fn reports_truncated_and_corrupt_data() {
        assert!(ObjectData::parse(&SKIN_W3U[..SKIN_W3U.len() - 3], Layout::Plain).is_err());
        assert!(ObjectData::parse(b"\x03\0\0\0\xff\xff\xff\x7f", Layout::Plain).is_err());

        let mut trailing = SKIN_W3U.to_vec();

        trailing.push(0);
        assert!(ObjectData::parse(&trailing, Layout::Plain).is_err());
    }
}
