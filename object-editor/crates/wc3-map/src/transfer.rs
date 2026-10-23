//! Moving custom objects between maps: share codes and imports.
//!
//! A [`Bundle`] holds custom units, abilities and buffs exactly as map files
//! store them, plus the strings they use. [`export`] follows each object's
//! references (a unit's abilities, an ability's buffs and summoned units, the
//! units a building trains) and brings along the custom objects they point at.
//! [`plan`] gives every object a free ID in the target map, and [`apply`]
//! adds them with every reference rewritten to the new IDs.
//!
//! A share code is the bundle's object files, compressed and base64-encoded
//! behind a `W3OBJ1:` prefix, so it can be pasted anywhere text goes. A bundle
//! file (`.w3obj`) also carries the files the objects use that were imported
//! into the source map (models, their textures, icons): those are too large
//! for text.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};

use base64::Engine;
use base64::alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use wc3_data::meta::FieldMeta;
use wc3_data::objects::{FourCC, Layout, ObjectData, ObjectEntry, Value};
use wc3_data::strings::{MapStrings, reference_id};

use crate::abilities::MapAbilities;
use crate::imports::{ImportedFile, files_used};
use crate::plain::PlainObjects;
use crate::units::MapUnits;
use crate::upgrades::MapUpgrades;
use crate::{
    ABILITIES_FILE, ABILITY_SKINS_FILE, BUFF_SKINS_FILE, BUFFS_FILE, Error, ITEM_SKINS_FILE,
    ITEMS_FILE, Result, STRINGS_FILE, UNIT_SKINS_FILE, UNITS_FILE, UPGRADE_SKINS_FILE,
    UPGRADES_FILE,
};

const CODE_PREFIX: &str = "W3OBJ1:";
/// Start of a bundle file, before the compressed payload.
const FILE_MAGIC: &[u8; 7] = b"W3OBJ1\0";
const PAYLOAD_MAGIC: &[u8; 4] = b"W3OB";
const PAYLOAD_VERSION: u8 = 1;
/// Largest payload a share code may unpack to; real codes are a few kilobytes.
const MAX_CODE_PAYLOAD: u64 = 16 * 1024 * 1024;
/// Largest payload a bundle file may unpack to: imported models add megabytes.
const MAX_FILE_PAYLOAD: u64 = 512 * 1024 * 1024;
/// Payload section holding the imported files.
const IMPORTS_SECTION: &str = "imports";
const LIST_SEPARATOR: char = ',';
/// Characters the World Editor uses for the last three letters of new IDs.
const ID_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ID_SUFFIX_LEN: u32 = 3;
const ABILITY_PREFIX: u8 = b'A';
const BUFF_PREFIX: u8 = b'B';
const ITEM_PREFIX: u8 = b'I';
const UPGRADE_PREFIX: u8 = b'R';

/// Share codes may be wrapped or padded by whatever carried them.
const BASE64: GeneralPurpose = GeneralPurpose::new(
    &alphabet::URL_SAFE,
    GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// The kinds of objects that move between maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Unit,
    Ability,
    Buff,
    Item,
    Upgrade,
}

impl ObjectType {
    pub const ALL: [Self; 5] = [
        Self::Unit,
        Self::Ability,
        Self::Buff,
        Self::Item,
        Self::Upgrade,
    ];

    /// The gameplay file and the Reforged skin file holding this kind.
    fn files(self) -> (&'static str, &'static str) {
        match self {
            Self::Unit => (UNITS_FILE, UNIT_SKINS_FILE),
            Self::Ability => (ABILITIES_FILE, ABILITY_SKINS_FILE),
            Self::Buff => (BUFFS_FILE, BUFF_SKINS_FILE),
            Self::Item => (ITEMS_FILE, ITEM_SKINS_FILE),
            Self::Upgrade => (UPGRADES_FILE, UPGRADE_SKINS_FILE),
        }
    }

    fn layout(self) -> Layout {
        match self {
            Self::Ability | Self::Upgrade => Layout::Leveled,
            Self::Unit | Self::Buff | Self::Item => Layout::Plain,
        }
    }

    /// Plural label, as the World Editor writes it in string comments.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unit => "Units",
            Self::Ability => "Abilities",
            Self::Buff => "Buffs",
            Self::Item => "Items",
            Self::Upgrade => "Upgrades",
        }
    }

    /// First letter of new IDs. Units keep theirs: it tells heroes (`H000`)
    /// from other units (`h000`).
    fn id_prefix(self, id: FourCC) -> u8 {
        match self {
            Self::Unit => id.0[0],
            Self::Ability => ABILITY_PREFIX,
            Self::Buff => BUFF_PREFIX,
            Self::Item => ITEM_PREFIX,
            Self::Upgrade => UPGRADE_PREFIX,
        }
    }
}

/// One kind's two tables: gameplay fields and skin (names, art) fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Tables {
    pub data: ObjectData,
    pub skin: ObjectData,
}

impl Tables {
    fn empty(ty: ObjectType) -> Self {
        Self {
            data: ObjectData::empty(ty.layout()),
            skin: ObjectData::empty(ty.layout()),
        }
    }

    pub(crate) fn both(&self) -> [&ObjectData; 2] {
        [&self.data, &self.skin]
    }

    fn is_custom(&self, id: FourCC) -> bool {
        self.both()
            .iter()
            .any(|t| t.custom.iter().any(|e| e.effective_id() == id))
    }

    /// Whether the map changes an original game object with this ID.
    fn is_changed(&self, id: FourCC) -> bool {
        self.both()
            .iter()
            .any(|t| t.original.iter().any(|e| e.effective_id() == id))
    }

    fn is_empty(&self) -> bool {
        self.both()
            .iter()
            .all(|t| t.custom.is_empty() && t.original.is_empty())
    }

    /// Custom object IDs in file order, each once.
    fn custom_ids(&self) -> Vec<(FourCC, FourCC)> {
        let mut ids: Vec<(FourCC, FourCC)> = Vec::new();
        for entry in self.both().iter().flat_map(|t| &t.custom) {
            let id = entry.effective_id();
            if !ids.iter().any(|(known, _)| *known == id) {
                ids.push((id, entry.base));
            }
        }
        ids
    }
}

/// A map's movable objects, borrowed from where the editor keeps them.
#[derive(Clone, Copy)]
pub struct MapObjects<'a> {
    pub units: &'a MapUnits,
    pub abilities: &'a MapAbilities,
    pub items: &'a PlainObjects,
    pub upgrades: &'a MapUpgrades,
    pub buffs: &'a PlainObjects,
}

impl MapObjects<'_> {
    fn has(&self, ty: ObjectType, id: FourCC) -> bool {
        match ty {
            ObjectType::Unit => {
                let (data, skin, _) = self.units.parts();
                data.find(id).is_some() || skin.find(id).is_some()
            }
            ObjectType::Ability => self.abilities.has(id),
            ObjectType::Buff => self.buffs.has(id),
            ObjectType::Item => self.items.has(id),
            ObjectType::Upgrade => self.upgrades.has(id),
        }
    }
}

/// A map's movable objects, borrowed to add imported objects to.
pub struct MapObjectsMut<'a> {
    pub units: &'a mut MapUnits,
    pub abilities: &'a mut MapAbilities,
    pub items: &'a mut PlainObjects,
    pub upgrades: &'a mut MapUpgrades,
    pub buffs: &'a mut PlainObjects,
}

/// The kinds of objects a field's value can name, from its metadata type.
/// Requirements (`techList`) name units and upgrades alike.
fn targets_for(value_type: &str) -> Option<&'static [ObjectType]> {
    Some(match value_type {
        "unitList" | "unitCode" | "unitSkinList" => &[ObjectType::Unit],
        "abilityList" | "heroAbilityList" | "abilCode" => &[ObjectType::Ability],
        "buffList" | "effectList" => &[ObjectType::Buff],
        "techList" => &[ObjectType::Unit, ObjectType::Upgrade],
        "itemList" => &[ObjectType::Item],
        "upgradeList" | "upgradeCode" => &[ObjectType::Upgrade],
        _ => return None,
    })
}

/// Which fields of units, abilities, items and upgrades hold IDs of other objects.
pub struct References {
    fields: HashMap<(ObjectType, FourCC), &'static [ObjectType]>,
}

impl References {
    pub fn new(
        unit_fields: &[FieldMeta],
        ability_fields: &[FieldMeta],
        item_fields: &[FieldMeta],
        upgrade_fields: &[FieldMeta],
    ) -> Self {
        let mut fields = HashMap::new();
        for (ty, metas) in [
            (ObjectType::Unit, unit_fields),
            (ObjectType::Ability, ability_fields),
            (ObjectType::Item, item_fields),
            (ObjectType::Upgrade, upgrade_fields),
        ] {
            for meta in metas {
                if let (Some(id), Some(targets)) =
                    (FourCC::parse(&meta.id), targets_for(&meta.value_type))
                {
                    fields.insert((ty, id), targets);
                }
            }
        }
        Self { fields }
    }

    fn targets(&self, ty: ObjectType, field: FourCC) -> &'static [ObjectType] {
        self.fields.get(&(ty, field)).copied().unwrap_or_default()
    }
}

fn list_ids(value: &Value) -> Vec<FourCC> {
    match value {
        Value::String(list) => list
            .split(LIST_SEPARATOR)
            .filter_map(|code| FourCC::parse(code.trim()))
            .collect(),
        _ => Vec::new(),
    }
}

/// Custom objects ready to move to another map.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    units: Tables,
    abilities: Tables,
    buffs: Tables,
    items: Tables,
    upgrades: Tables,
    strings: MapStrings,
    /// Files imported into the source map that the objects use.
    files: Vec<ImportedFile>,
}

/// An object in a bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BundledObject {
    pub ty: ObjectType,
    pub id: FourCC,
    pub base: FourCC,
}

/// A bundle and what it could not bring along.
pub struct Export {
    pub bundle: Bundle,
    /// The selected objects first, then the ones they depend on.
    pub objects: Vec<BundledObject>,
    pub warnings: Vec<String>,
}

impl Bundle {
    fn new() -> Self {
        Self {
            units: Tables::empty(ObjectType::Unit),
            abilities: Tables::empty(ObjectType::Ability),
            buffs: Tables::empty(ObjectType::Buff),
            items: Tables::empty(ObjectType::Item),
            upgrades: Tables::empty(ObjectType::Upgrade),
            strings: MapStrings::default(),
            files: Vec::new(),
        }
    }

    /// Imported files the objects use; empty for bundles read from a share code.
    pub fn files(&self) -> &[ImportedFile] {
        &self.files
    }

    fn tables(&self, ty: ObjectType) -> &Tables {
        match ty {
            ObjectType::Unit => &self.units,
            ObjectType::Ability => &self.abilities,
            ObjectType::Buff => &self.buffs,
            ObjectType::Item => &self.items,
            ObjectType::Upgrade => &self.upgrades,
        }
    }

    fn tables_mut(&mut self, ty: ObjectType) -> &mut Tables {
        match ty {
            ObjectType::Unit => &mut self.units,
            ObjectType::Ability => &mut self.abilities,
            ObjectType::Buff => &mut self.buffs,
            ObjectType::Item => &mut self.items,
            ObjectType::Upgrade => &mut self.upgrades,
        }
    }

    /// Every object in the bundle, units first.
    pub fn objects(&self) -> Vec<BundledObject> {
        ObjectType::ALL
            .into_iter()
            .flat_map(|ty| {
                self.tables(ty)
                    .custom_ids()
                    .into_iter()
                    .map(move |(id, base)| BundledObject { ty, id, base })
            })
            .collect()
    }

    /// A field's value for an object, with string references resolved.
    pub fn text(&self, ty: ObjectType, id: FourCC, field: &str) -> Option<String> {
        let field = FourCC::parse(field)?;
        self.tables(ty)
            .both()
            .iter()
            .flat_map(|t| &t.custom)
            .filter(|e| e.effective_id() == id)
            .flat_map(ObjectEntry::mods)
            .filter(|m| m.field == field)
            .last()
            .map(|m| self.strings.resolve(&m.value.to_string()).to_owned())
    }

    /// The object files and strings, and the imported files when asked.
    fn payload(&self, with_files: bool) -> Vec<u8> {
        let mut payload = PAYLOAD_MAGIC.to_vec();
        payload.push(PAYLOAD_VERSION);

        for ty in ObjectType::ALL {
            let tables = self.tables(ty);
            let (data_file, skin_file) = ty.files();
            for (file, data) in [(data_file, &tables.data), (skin_file, &tables.skin)] {
                if !data.custom.is_empty() {
                    put_section(&mut payload, file, &data.to_bytes());
                }
            }
        }
        if !self.strings.entries.is_empty() {
            put_section(
                &mut payload,
                STRINGS_FILE,
                self.strings.to_text().as_bytes(),
            );
        }
        if with_files && !self.files.is_empty() {
            let mut files = Vec::new();
            for file in &self.files {
                // Archive paths are at most 260 bytes; files far below 4 GiB.
                files.extend_from_slice(
                    &u16::try_from(file.path.len())
                        .unwrap_or(u16::MAX)
                        .to_le_bytes(),
                );
                files.extend_from_slice(file.path.as_bytes());
                files.extend_from_slice(
                    &u32::try_from(file.bytes.len())
                        .unwrap_or(u32::MAX)
                        .to_le_bytes(),
                );
                files.extend_from_slice(&file.bytes);
            }
            put_section(&mut payload, IMPORTS_SECTION, &files);
        }

        compress(&payload)
    }

    /// The share code: `W3OBJ1:` and the compressed object files,
    /// base64-encoded. Imported files only travel in a bundle file.
    pub fn to_code(&self) -> String {
        format!("{CODE_PREFIX}{}", BASE64.encode(self.payload(false)))
    }

    /// The bundle file (`.w3obj`): the objects and the imported files they use.
    pub fn to_file(&self) -> Vec<u8> {
        let mut out = FILE_MAGIC.to_vec();
        out.extend(self.payload(true));
        out
    }

    /// Reads a share code. Spaces and line breaks inside it are ignored.
    pub fn from_code(code: &str) -> Result<Self> {
        let invalid = |reason: &str| Error::Invalid(format!("Not a valid share code: {reason}."));
        let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
        let body = code
            .strip_prefix(CODE_PREFIX)
            .ok_or_else(|| invalid("it should start with W3OBJ1:"))?;
        let compressed = BASE64
            .decode(body)
            .map_err(|_| invalid("it is cut off or has extra characters"))?;
        let payload = decompress(&compressed, MAX_CODE_PAYLOAD)
            .ok_or_else(|| invalid("it is cut off or has extra characters"))?;
        Self::from_payload(&payload, &invalid)
    }

    /// Reads a bundle file (`.w3obj`).
    pub fn from_file(bytes: &[u8]) -> Result<Self> {
        let invalid = |reason: &str| Error::Invalid(format!("Not a valid bundle file: {reason}."));
        let compressed = bytes
            .strip_prefix(FILE_MAGIC.as_slice())
            .ok_or_else(|| invalid("it is not a .w3obj file"))?;
        let payload = decompress(compressed, MAX_FILE_PAYLOAD)
            .ok_or_else(|| invalid("it is cut off or damaged"))?;
        Self::from_payload(&payload, &invalid)
    }

    fn from_payload(payload: &[u8], invalid: &dyn Fn(&str) -> Error) -> Result<Self> {
        let rest = payload
            .strip_prefix(PAYLOAD_MAGIC.as_slice())
            .and_then(|r| r.split_first())
            .filter(|(version, _)| **version == PAYLOAD_VERSION)
            .map(|(_, rest)| rest)
            .ok_or_else(|| invalid("it was made by a different version of the editor"))?;

        let mut bundle = Self::new();
        let mut pos = 0;
        while pos < rest.len() {
            let (name, bytes, next) =
                read_section(rest, pos).ok_or_else(|| invalid("its contents are damaged"))?;
            pos = next;

            if name == STRINGS_FILE {
                let text =
                    std::str::from_utf8(bytes).map_err(|_| invalid("its text is not UTF-8"))?;
                bundle.strings = MapStrings::parse(text)?;
                continue;
            }

            if name == IMPORTS_SECTION {
                bundle.files = read_files(bytes).ok_or_else(|| invalid("its files are damaged"))?;
                continue;
            }

            let (ty, is_skin) = ObjectType::ALL
                .into_iter()
                .find_map(|ty| {
                    let (data_file, skin_file) = ty.files();
                    (name == data_file)
                        .then_some((ty, false))
                        .or((name == skin_file).then_some((ty, true)))
                })
                .ok_or_else(|| invalid("it holds an unknown file"))?;
            let mut data = ObjectData::parse(bytes, ty.layout())?;
            // Only custom objects move between maps.
            data.original.clear();
            let tables = bundle.tables_mut(ty);
            if is_skin {
                tables.skin = data;
            } else {
                tables.data = data;
            }
        }

        if ObjectType::ALL
            .iter()
            .all(|ty| bundle.tables(*ty).is_empty())
        {
            return Err(invalid("it holds no objects"));
        }

        Ok(bundle)
    }

    /// Every text value of the bundled objects, as stored (string references
    /// unresolved): art fields among them name the files objects use.
    fn values(&self) -> impl Iterator<Item = &str> {
        ObjectType::ALL
            .into_iter()
            .flat_map(|ty| self.tables(ty).both())
            .flat_map(|t| &t.custom)
            .flat_map(ObjectEntry::mods)
            .filter_map(|m| match &m.value {
                Value::String(s) if reference_id(s).is_none() => Some(s.as_str()),
                _ => None,
            })
    }
}

fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    // Writing to a Vec cannot fail.
    encoder
        .write_all(bytes)
        .and_then(|()| encoder.finish())
        .unwrap_or_default()
}

/// Inflates at most `limit` bytes; `None` when the data is damaged.
fn decompress(bytes: &[u8], limit: u64) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    DeflateDecoder::new(bytes)
        .take(limit)
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

/// The imported files section: path length (u16), path, size (u32), bytes.
fn read_files(data: &[u8]) -> Option<Vec<ImportedFile>> {
    let mut files = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let path_len = usize::from(u16::from_le_bytes([*data.get(pos)?, *data.get(pos + 1)?]));
        let path_end = pos + 2 + path_len;
        let path = std::str::from_utf8(data.get(pos + 2..path_end)?)
            .ok()?
            .to_owned();
        let size = data.get(path_end..path_end + 4)?;
        let size = u32::from_le_bytes([size[0], size[1], size[2], size[3]]) as usize;
        let start = path_end + 4;
        let bytes = data.get(start..start + size)?.to_vec();
        files.push(ImportedFile { path, bytes });
        pos = start + size;
    }
    Some(files)
}

fn put_section(out: &mut Vec<u8>, name: &str, bytes: &[u8]) {
    // File names here are short constants and bundles are far below 4 GiB.
    out.push(u8::try_from(name.len()).unwrap_or(u8::MAX));
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(bytes);
}

/// The section at `pos`: its name, its bytes and where the next one starts.
fn read_section(data: &[u8], pos: usize) -> Option<(&str, &[u8], usize)> {
    let name_len = usize::from(*data.get(pos)?);
    let name_end = pos + 1 + name_len;
    let name = std::str::from_utf8(data.get(pos + 1..name_end)?).ok()?;
    let mut len = [0u8; 4];
    len.copy_from_slice(data.get(name_end..name_end + 4)?);
    let start = name_end + 4;
    let end = start + u32::from_le_bytes(len) as usize;
    Some((name, data.get(start..end)?, end))
}

/// Bundles the given custom units with every custom unit, ability, buff,
/// item and upgrade they depend on, and the files imported into the source map that they use
/// (`read` reads a file from it by path). Warnings name what could not come along.
pub fn export(
    source: MapObjects<'_>,
    refs: &References,
    units: &[FourCC],
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Export> {
    let (unit_data, unit_skin, strings) = source.units.parts();
    let source_units = Tables {
        data: unit_data.clone(),
        skin: unit_skin.clone(),
    };
    let tables = |ty: ObjectType| match ty {
        ObjectType::Unit => &source_units,
        ObjectType::Ability => source.abilities.tables(),
        ObjectType::Buff => source.buffs.tables(),
        ObjectType::Item => source.items.tables(),
        ObjectType::Upgrade => source.upgrades.tables(),
    };

    if let Some(id) = units.iter().find(|id| !source_units.is_custom(**id)) {
        return Err(Error::Invalid(format!(
            "{id} is not a custom unit; only custom objects can be shared."
        )));
    }

    let mut bundle = Bundle::new();
    let mut objects = Vec::new();
    let mut left_behind: Vec<LeftBehind> = Vec::new();
    let mut queue: VecDeque<(ObjectType, FourCC)> =
        units.iter().map(|id| (ObjectType::Unit, *id)).collect();
    let mut seen = HashSet::new();

    while let Some((ty, id)) = queue.pop_front() {
        if !seen.insert((ty, id)) {
            continue;
        }

        let from = tables(ty);
        let mut base = None;
        for (is_skin, input) in [(false, &from.data), (true, &from.skin)] {
            for entry in input.custom.iter().filter(|e| e.effective_id() == id) {
                base = Some(entry.base);
                let mut entry = entry.clone();
                for m in entry.sets.iter_mut().flat_map(|s| s.mods.iter_mut()) {
                    if let Value::String(s) = &m.value
                        && let Some(number) = reference_id(s)
                    {
                        let text = strings.get(number).unwrap_or_default();
                        m.value = Value::String(bundle.strings.add(text, None));
                    }

                    let targets = refs.targets(ty, m.field);
                    for code in list_ids(&m.value) {
                        if let Some(other) = targets.iter().find(|t| tables(**t).is_custom(code)) {
                            queue.push_back((*other, code));
                            continue;
                        }

                        if !targets.iter().any(|t| tables(*t).is_changed(code)) {
                            continue;
                        }
                        match left_behind.iter_mut().find(|l| l.code == code) {
                            Some(known) if !known.users.contains(&id) => known.users.push(id),
                            Some(_) => {}
                            None => left_behind.push(LeftBehind {
                                code,
                                users: vec![id],
                            }),
                        }
                    }
                }

                let output = bundle.tables_mut(ty);
                let output = if is_skin {
                    &mut output.skin
                } else {
                    &mut output.data
                };
                output.custom.push(entry);
            }
        }

        if let Some(base) = base {
            objects.push(BundledObject { ty, id, base });
        }
    }

    bundle.files = files_used(bundle.values(), read);

    Ok(Export {
        bundle,
        objects,
        warnings: left_behind.iter().map(LeftBehind::message).collect(),
    })
}

/// A game object the source map changes that a shared object refers to: it
/// does not come along, so the target map gets the game's version.
struct LeftBehind {
    code: FourCC,
    users: Vec<FourCC>,
}

/// Users named in a warning before the rest are counted.
const NAMED_USERS: usize = 3;

impl LeftBehind {
    fn message(&self) -> String {
        let what = format!(
            "{} is changed in this map; the game's version applies",
            self.code
        );
        let mut users: Vec<String> = self
            .users
            .iter()
            .take(NAMED_USERS)
            .map(FourCC::to_string)
            .collect();
        if self.users.len() > NAMED_USERS {
            users.push(format!("{} more", self.users.len() - NAMED_USERS));
        }
        format!("{what}. Used by {}.", users.join(", "))
    }
}

/// Where a bundled object goes in the target map.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedObject {
    pub ty: ObjectType,
    /// Its ID in the bundle.
    pub id: FourCC,
    /// Its ID in the target map: the same unless that one is taken.
    pub new_id: FourCC,
    pub base: FourCC,
}

/// Picks an ID for every bundled object: its own when the target map and the
/// game do not use it, otherwise the next free one with the same first letter
/// (`A000`, `A001`, ... for abilities).
pub fn plan(
    bundle: &Bundle,
    target: MapObjects<'_>,
    in_game: &dyn Fn(ObjectType, FourCC) -> bool,
) -> Result<Vec<PlannedObject>> {
    let mut taken: HashSet<(ObjectType, FourCC)> = HashSet::new();
    let mut planned = Vec::new();

    for object in bundle.objects() {
        let ty = object.ty;
        let free = |id: FourCC, taken: &HashSet<(ObjectType, FourCC)>| {
            !target.has(ty, id) && !in_game(ty, id) && !taken.contains(&(ty, id))
        };
        let new_id = if free(object.id, &taken) {
            object.id
        } else {
            next_free_id(ty.id_prefix(object.id), |id| free(id, &taken))
                .ok_or_else(|| Error::Invalid(format!("No free ID left for {}.", object.id)))?
        };

        taken.insert((ty, new_id));
        planned.push(PlannedObject {
            ty,
            id: object.id,
            new_id,
            base: object.base,
        });
    }

    Ok(planned)
}

pub(crate) fn next_free_id(prefix: u8, free: impl Fn(FourCC) -> bool) -> Option<FourCC> {
    let radix = ID_ALPHABET.len();
    (0..radix.pow(ID_SUFFIX_LEN))
        .map(|n| {
            FourCC([
                prefix,
                ID_ALPHABET[n / (radix * radix)],
                ID_ALPHABET[(n / radix) % radix],
                ID_ALPHABET[n % radix],
            ])
        })
        .find(|id| free(*id))
}

/// Adds the planned objects to the target map, with references between them
/// and their text rewritten for the target.
pub fn apply(
    bundle: &Bundle,
    plan: &[PlannedObject],
    refs: &References,
    target: MapObjectsMut<'_>,
) {
    let MapObjectsMut {
        units,
        abilities,
        items,
        upgrades,
        buffs,
    } = target;
    let renamed: HashMap<(ObjectType, FourCC), FourCC> =
        plan.iter().map(|p| ((p.ty, p.id), p.new_id)).collect();
    let (unit_data, unit_skin, strings) = units.parts_mut();

    for planned in plan {
        let ty = planned.ty;
        let from = bundle.tables(ty);
        let (to_data, to_skin) = match ty {
            ObjectType::Unit => (&mut *unit_data, &mut *unit_skin),
            ObjectType::Ability => {
                let tables = abilities.tables_mut();
                (&mut tables.data, &mut tables.skin)
            }
            ObjectType::Buff => {
                let tables = buffs.tables_mut();
                (&mut tables.data, &mut tables.skin)
            }
            ObjectType::Item => {
                let tables = items.tables_mut();
                (&mut tables.data, &mut tables.skin)
            }
            ObjectType::Upgrade => {
                let tables = upgrades.tables_mut();
                (&mut tables.data, &mut tables.skin)
            }
        };
        let comment = format!("{}: {}", ty.label(), planned.new_id);

        for (input, output) in [(&from.data, to_data), (&from.skin, to_skin)] {
            for entry in input
                .custom
                .iter()
                .filter(|e| e.effective_id() == planned.id)
            {
                let mut entry = entry.clone();
                entry.id = planned.new_id;
                for m in entry.sets.iter_mut().flat_map(|s| s.mods.iter_mut()) {
                    if m.end == planned.id {
                        m.end = planned.new_id;
                    }

                    if let Value::String(s) = &m.value
                        && let Some(number) = reference_id(s)
                    {
                        let text = bundle.strings.get(number).unwrap_or_default();
                        m.value = Value::String(strings.add(text, Some(&comment)));
                    }

                    let targets = refs.targets(ty, m.field);
                    if !targets.is_empty()
                        && let Value::String(list) = &m.value
                    {
                        let rewritten: Vec<String> = list
                            .split(LIST_SEPARATOR)
                            .map(|code| {
                                FourCC::parse(code.trim())
                                    .and_then(|id| {
                                        targets.iter().find_map(|t| renamed.get(&(*t, id)))
                                    })
                                    .map_or_else(|| code.to_owned(), FourCC::to_string)
                            })
                            .collect();
                        m.value = Value::String(rewritten.join(","));
                    }
                }
                output.custom.push(entry);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wc3_data::objects::{ModSet, Modification};

    use super::*;
    use crate::plain;

    const NAME: &str = "unam";
    const ABILITIES: &str = "uabi";
    const TRAINS: &str = "utra";
    const ITEMS: &str = "umki";
    const RESEARCHES: &str = "ures";
    const ITEM_ABILITIES: &str = "iabi";
    const UPGRADE_REQUIREMENTS: &str = "greq";
    const BUFFS: &str = "abuf";
    const ABILITY_NAME: &str = "anam";
    const MODEL: &str = "umdl";

    /// A source map with no imported files.
    fn no_files(_: &str) -> Option<Vec<u8>> {
        None
    }

    fn id(code: &str) -> FourCC {
        FourCC::parse(code).unwrap()
    }

    fn text(field: &str, value: &str, end: FourCC) -> Modification {
        Modification {
            field: id(field),
            level: None,
            column: None,
            value: Value::String(value.to_owned()),
            end,
        }
    }

    fn entry(base: &str, own: &str, mods: Vec<Modification>) -> ObjectEntry {
        ObjectEntry {
            base: id(base),
            id: id(own),
            sets: vec![ModSet { flags: 0, mods }],
        }
    }

    fn table(layout: Layout, custom: Vec<ObjectEntry>) -> ObjectData {
        let mut data = ObjectData::empty(layout);
        data.custom = custom;
        data
    }

    fn refs() -> References {
        let fields: [((ObjectType, &str), &'static [ObjectType]); 6] = [
            ((ObjectType::Unit, ABILITIES), &[ObjectType::Ability]),
            ((ObjectType::Unit, TRAINS), &[ObjectType::Unit]),
            ((ObjectType::Unit, ITEMS), &[ObjectType::Item]),
            ((ObjectType::Unit, RESEARCHES), &[ObjectType::Upgrade]),
            ((ObjectType::Ability, BUFFS), &[ObjectType::Buff]),
            ((ObjectType::Item, ITEM_ABILITIES), &[ObjectType::Ability]),
        ];
        let mut fields: HashMap<(ObjectType, FourCC), &'static [ObjectType]> = fields
            .into_iter()
            .map(|((ty, field), targets)| ((ty, id(field)), targets))
            .collect();
        fields.insert(
            (ObjectType::Upgrade, id(UPGRADE_REQUIREMENTS)),
            &[ObjectType::Unit, ObjectType::Upgrade],
        );
        References { fields }
    }

    /// A map's movable objects, owned, for tests to borrow as [`MapObjects`].
    struct Objects {
        units: MapUnits,
        abilities: MapAbilities,
        items: PlainObjects,
        upgrades: MapUpgrades,
        buffs: PlainObjects,
    }

    impl Objects {
        fn new(
            units: MapUnits,
            mut abilities: Vec<ObjectEntry>,
            buffs: Vec<ObjectEntry>,
            items: Vec<ObjectEntry>,
            mut upgrades: Vec<ObjectEntry>,
        ) -> Self {
            // Leveled files store a level and data column with every value.
            for m in abilities
                .iter_mut()
                .chain(upgrades.iter_mut())
                .flat_map(|e| &mut e.sets)
                .flat_map(|s| &mut s.mods)
            {
                m.level = Some(1);
                m.column = Some(0);
            }

            Self {
                units,
                abilities: MapAbilities::from_tables(Tables {
                    data: table(Layout::Leveled, abilities),
                    skin: ObjectData::empty(Layout::Leveled),
                }),
                items: PlainObjects::from_tables(
                    plain::ITEMS,
                    Tables {
                        data: table(Layout::Plain, items),
                        skin: ObjectData::empty(Layout::Plain),
                    },
                ),
                upgrades: MapUpgrades::from_tables(Tables {
                    data: table(Layout::Leveled, upgrades),
                    skin: ObjectData::empty(Layout::Leveled),
                }),
                buffs: PlainObjects::from_tables(
                    plain::BUFFS,
                    Tables {
                        data: table(Layout::Plain, buffs),
                        skin: ObjectData::empty(Layout::Plain),
                    },
                ),
            }
        }

        fn borrow(&self) -> MapObjects<'_> {
            MapObjects {
                units: &self.units,
                abilities: &self.abilities,
                items: &self.items,
                upgrades: &self.upgrades,
                buffs: &self.buffs,
            }
        }
    }

    /// A barracks (`h000`) training a custom footman (`h001`) whose ability
    /// `A000` applies buff `B000`; the barracks also makes custom item `I000`,
    /// which gives ability `A000`, and researches custom upgrade `R000`, which
    /// requires the footman. The map also changes the game's `AHhb`, which the
    /// footman has.
    fn source() -> Objects {
        let mut strings = MapStrings::default();
        let name = strings.add("Elite Footman", None);
        let units = MapUnits::from_parts(
            table(
                Layout::Plain,
                vec![
                    entry(
                        "hbar",
                        "h000",
                        vec![
                            text(TRAINS, "h001,hfoo", FourCC::ZERO),
                            text(ITEMS, "I000", FourCC::ZERO),
                            text(RESEARCHES, "R000", FourCC::ZERO),
                        ],
                    ),
                    entry(
                        "hfoo",
                        "h001",
                        vec![text(ABILITIES, "A000,AHhb", id("h001"))],
                    ),
                    entry("hfoo", "h002", vec![]),
                ],
            ),
            table(
                Layout::Plain,
                vec![entry(
                    "hfoo",
                    "h001",
                    vec![
                        text(NAME, &name, id("h001")),
                        text(MODEL, "Elite.mdl", id("h001")),
                    ],
                )],
            ),
            strings,
        );
        let mut objects = Objects::new(
            units,
            vec![entry(
                "AHhb",
                "A000",
                vec![text(BUFFS, "B000", FourCC::ZERO)],
            )],
            vec![entry("BHbd", "B000", vec![])],
            vec![entry(
                "ratf",
                "I000",
                vec![text(ITEM_ABILITIES, "A000", FourCC::ZERO)],
            )],
            vec![entry(
                "Rhme",
                "R000",
                vec![text(UPGRADE_REQUIREMENTS, "h001", FourCC::ZERO)],
            )],
        );
        let mut changed = entry("AHhb", "AHhb", vec![]);
        changed.id = FourCC::ZERO;
        objects.abilities.tables_mut().data.original.push(changed);
        objects
    }

    fn no_units() -> MapUnits {
        MapUnits::from_parts(
            ObjectData::empty(Layout::Plain),
            ObjectData::empty(Layout::Plain),
            MapStrings::default(),
        )
    }

    /// A map with nothing in it but one custom ability and one custom item.
    fn target_with(ability: &str, item: &str, units: MapUnits) -> Objects {
        Objects::new(
            units,
            vec![entry(
                "AHhb",
                ability,
                vec![text(ABILITY_NAME, "Mine", FourCC::ZERO)],
            )],
            Vec::new(),
            vec![entry("ratf", item, vec![])],
            Vec::new(),
        )
    }

    #[test]
    fn exports_dependencies_and_warns_about_the_rest() {
        let source = source();
        let export = export(source.borrow(), &refs(), &[id("h000")], &no_files).unwrap();

        let objects: Vec<(ObjectType, String)> = export
            .objects
            .iter()
            .map(|o| (o.ty, o.id.to_string()))
            .collect();
        assert_eq!(
            objects,
            [
                (ObjectType::Unit, "h000".into()),
                (ObjectType::Unit, "h001".into()),
                (ObjectType::Item, "I000".into()),
                (ObjectType::Upgrade, "R000".into()),
                (ObjectType::Ability, "A000".into()),
                (ObjectType::Buff, "B000".into()),
            ]
        );
        assert_eq!(export.warnings.len(), 1);
        assert!(export.warnings[0].starts_with("AHhb is changed"));
        assert_eq!(
            export
                .bundle
                .text(ObjectType::Unit, id("h001"), NAME)
                .as_deref(),
            Some("Elite Footman")
        );
    }

    #[test]
    fn only_custom_units_can_be_shared() {
        let source = source();
        assert!(export(source.borrow(), &refs(), &[id("hfoo")], &no_files).is_err());
    }

    #[test]
    fn share_codes_round_trip_and_reject_damage() {
        let source = source();
        let bundle = export(source.borrow(), &refs(), &[id("h000")], &no_files)
            .unwrap()
            .bundle;
        let code = bundle.to_code();

        assert!(code.starts_with(CODE_PREFIX));
        let wrapped: String = code
            .chars()
            .enumerate()
            .flat_map(|(i, c)| if i % 40 == 39 { vec![c, '\n'] } else { vec![c] })
            .collect();
        assert_eq!(Bundle::from_code(&wrapped).unwrap(), bundle);

        assert!(Bundle::from_code("hello").is_err());
        assert!(Bundle::from_code(&code[..code.len() - 8]).is_err());
        assert!(Bundle::from_code(&format!("{CODE_PREFIX}AAAA")).is_err());
    }

    #[test]
    fn bundle_files_carry_imported_files_and_codes_do_not() {
        let source = source();
        let read = |path: &str| (path == "Elite.mdx").then(|| b"model".to_vec());
        let bundle = export(source.borrow(), &refs(), &[id("h001")], &read)
            .unwrap()
            .bundle;

        let paths: Vec<&str> = bundle.files().iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["Elite.mdx"]);
        assert_eq!(Bundle::from_file(&bundle.to_file()).unwrap(), bundle);
        assert!(
            Bundle::from_code(&bundle.to_code())
                .unwrap()
                .files()
                .is_empty()
        );

        let file = bundle.to_file();
        assert!(Bundle::from_file(&file[..file.len() - 4]).is_err());
        assert!(Bundle::from_file(bundle.to_code().as_bytes()).is_err());
    }

    #[test]
    fn imports_with_free_ids_and_rewritten_references() {
        let source = source();
        let bundle = export(source.borrow(), &refs(), &[id("h000")], &no_files)
            .unwrap()
            .bundle;

        // The target already has its own h001, A000 and I000.
        let mut target = target_with(
            "A000",
            "I000",
            MapUnits::from_parts(
                table(Layout::Plain, vec![entry("hpea", "h001", vec![])]),
                ObjectData::empty(Layout::Plain),
                MapStrings::default(),
            ),
        );
        let planned = plan(&bundle, target.borrow(), &|_, _| false).unwrap();

        let new_ids: Vec<(String, String)> = planned
            .iter()
            .map(|p| (p.id.to_string(), p.new_id.to_string()))
            .collect();
        assert_eq!(
            new_ids,
            [
                ("h000".into(), "h000".into()),
                ("h001".into(), "h002".into()),
                ("A000".into(), "A001".into()),
                ("B000".into(), "B000".into()),
                ("I000".into(), "I001".into()),
                ("R000".into(), "R000".into()),
            ]
        );

        apply(
            &bundle,
            &planned,
            &refs(),
            MapObjectsMut {
                units: &mut target.units,
                abilities: &mut target.abilities,
                items: &mut target.items,
                upgrades: &mut target.upgrades,
                buffs: &mut target.buffs,
            },
        );

        let (data, skin, strings) = target.units.parts();
        let barracks: Vec<Value> = data
            .find(id("h000"))
            .unwrap()
            .mods()
            .map(|m| m.value.clone())
            .collect();
        assert_eq!(
            barracks,
            [
                Value::String("h002,hfoo".into()),
                Value::String("I001".into()),
                Value::String("R000".into()),
            ]
        );
        let imported = data.find(id("h002")).unwrap();
        let abilities = imported.mods().next().unwrap();
        assert_eq!(abilities.value, Value::String("A001,AHhb".into()));
        let name = skin.find(id("h002")).unwrap().mods().next().unwrap();
        assert_eq!(name.end, id("h002"));
        assert_eq!(strings.resolve(&name.value.to_string()), "Elite Footman");
        let ability = target.abilities.tables().data.find(id("A001")).unwrap();
        assert_eq!(
            ability.mods().next().unwrap().value,
            Value::String("B000".into())
        );
        let item = target.items.tables().data.find(id("I001")).unwrap();
        assert_eq!(item.base, id("ratf"));
        assert_eq!(
            item.mods().next().unwrap().value,
            Value::String("A001".into())
        );
        let upgrade = target.upgrades.tables().data.find(id("R000")).unwrap();
        assert_eq!(
            upgrade.mods().next().unwrap().value,
            Value::String("h002".into())
        );
        assert!(target.buffs.tables().data.find(id("B000")).is_some());
    }

    #[test]
    fn writes_buff_files_only_when_needed() {
        let empty = Objects::new(no_units(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        assert!(empty.buffs.files().is_empty());

        let with_buff = Objects::new(
            no_units(),
            Vec::new(),
            vec![entry("BHbd", "B000", vec![])],
            Vec::new(),
            Vec::new(),
        );
        let names: Vec<&str> = with_buff.buffs.files().iter().map(|(n, _)| *n).collect();
        assert_eq!(names, [BUFFS_FILE]);
    }
}
