//! Keeping a map's script in step with its data when the map is saved.
//!
//! The game runs only the script, so placed units, drops, item tables, players,
//! forces and environment changed in the data files do nothing until the World
//! Editor's generated parts of the script are rewritten. A part is rewritten only
//! when generating it from the data as loaded gives back the script's own text
//! exactly: anything else means the script was changed by hand, and is left as it is.

use std::collections::HashMap;

use crate::info::{ItemTable, Properties};
use crate::placements::UnitPlacements;
use crate::script::{self, Language, UnitKinds, UnitSection};
use crate::setup::{self, SetupSection};
use crate::{Map, Result};

const LINE_END: &str = "\r\n";

/// The map's data the script is generated from, as it is now.
pub struct MapData<'a> {
    pub placements: Option<&'a UnitPlacements>,
    pub tables: &'a [ItemTable],
    pub properties: Option<&'a Properties>,
    /// The terrain bounds weather covers.
    pub bounds: Option<[f32; 4]>,
}

/// The script as last loaded or saved, the data it was generated from, and
/// which of its parts can be rewritten.
#[derive(Debug, Clone)]
pub struct ScriptState {
    pub file: &'static str,
    pub language: Language,
    pub text: String,
    crlf: bool,
    placements: Option<UnitPlacements>,
    tables: Vec<ItemTable>,
    properties: Option<Properties>,
    rewrites_units: bool,
    rewrites_setup: bool,
}

/// The map's script file, its language and bytes: `war3map.lua`, else `war3map.j`.
pub fn read_script(archive: &Map) -> Option<(&'static str, Language, Vec<u8>)> {
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

fn units_reproduce(text: &str, language: Language, section: &UnitSection<'_>) -> bool {
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

fn setup_section<'a>(properties: &'a Properties, data: &MapData<'_>) -> SetupSection<'a> {
    SetupSection {
        properties,
        bounds: data.bounds,
    }
}

impl ScriptState {
    /// Reads the script and checks which parts it can rewrite later.
    pub fn new(
        file: &'static str,
        language: Language,
        bytes: &[u8],
        data: &MapData<'_>,
        kinds: &dyn UnitKinds,
    ) -> Self {
        let raw = String::from_utf8_lossy(bytes);
        let crlf = raw.contains(LINE_END);
        let text = raw.replace(LINE_END, "\n");
        let rewrites_units = data.placements.is_some_and(|placements| {
            let section = UnitSection {
                placements,
                original: Some(placements),
                tables: data.tables,
                kinds,
            };

            units_reproduce(&text, language, &section)
        });
        let rewrites_setup = data
            .properties
            .is_some_and(|p| setup::reproduces(language, &text, &setup_section(p, data)));

        Self {
            file,
            language,
            text,
            crlf,
            placements: data.placements.cloned(),
            tables: data.tables.to_vec(),
            properties: data.properties.cloned(),
            rewrites_units,
            rewrites_setup,
        }
    }

    pub fn load(archive: &Map, data: &MapData<'_>, kinds: &dyn UnitKinds) -> Option<Self> {
        let (file, language, bytes) = read_script(archive)?;

        Some(Self::new(file, language, &bytes, data, kinds))
    }

    /// Whether placed units, drops and item tables reach the script on save.
    pub fn rewrites_units(&self) -> bool {
        self.rewrites_units
    }

    /// Whether players, forces and environment reach the script on save.
    pub fn rewrites_setup(&self) -> bool {
        self.rewrites_setup
    }

    /// The script with the parts whose data changed rewritten, or `None` when
    /// nothing in it changes.
    pub fn updated(&self, data: &MapData<'_>, kinds: &dyn UnitKinds) -> Result<Option<String>> {
        let mut text = self.text.clone();
        let units_changed =
            data.placements != self.placements.as_ref() || data.tables != self.tables.as_slice();
        let setup_changed = data.properties != self.properties.as_ref();

        if let (true, true, Some(placements)) =
            (units_changed, self.rewrites_units, data.placements)
        {
            let section = UnitSection {
                placements,
                original: self.placements.as_ref(),
                tables: data.tables,
                kinds,
            };
            let generated = script::unit_functions(self.language, &text, &section)?;

            text = script::splice_units(self.language, &text, &generated)?;
        }

        if let (true, true, Some(properties)) =
            (setup_changed, self.rewrites_setup, data.properties)
        {
            text = setup::rewrite(self.language, &text, &setup_section(properties, data))?;
        }

        Ok((text != self.text).then_some(text))
    }

    /// The text as the file stores it, with the line endings it had.
    pub fn bytes(&self, text: &str) -> Vec<u8> {
        if self.crlf {
            return text.replace('\n', LINE_END).into_bytes();
        }

        text.as_bytes().to_vec()
    }

    /// Records a save: the new text, if it changed, and the data the
    /// rewritable parts now come from.
    pub fn saved(&mut self, text: Option<String>, data: &MapData<'_>) {
        if let Some(text) = text {
            self.text = text;
        }

        if self.rewrites_units {
            self.placements = data.placements.cloned();
            self.tables = data.tables.to_vec();
        }

        if self.rewrites_setup {
            self.properties = data.properties.cloned();
        }
    }
}
