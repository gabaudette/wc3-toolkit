//! Layered access to the game's files.
//!
//! Reforged stores its data in layers inside one CASC storage:
//! `war3.w3mod:` is the base, `_balance\<set>.w3mod:` overrides game data for a
//! balance version, `_locales\<locale>.w3mod:` holds text and `_hd.w3mod:` holds
//! HD art. A file in a more specific layer replaces the base file as a whole.
//! `_de.w3mod:` holds HD art that exists nowhere else, such as the models of
//! newer campaign units (Roderick, the Scarlet Crusade, the Forsaken); art
//! lookups use it last.

use std::path::Path;

use crate::{Error, Result};

const ROOT: &str = "war3.w3mod:";
const HD_LAYER: &str = "_hd.w3mod:";
const EXTRA_ART_LAYER: &str = "_de.w3mod:";
const DEFAULT_LOCALE: &str = "enus";

/// Which art the user works with. Reforged keeps separate values for some
/// fields (`walk:sd`, `walk:hd`) and separate icons and models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArtMode {
    #[default]
    Sd,
    Hd,
}

impl ArtMode {
    pub fn key_suffix(self) -> &'static str {
        match self {
            Self::Sd => "sd",
            Self::Hd => "hd",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameOptions {
    /// Balance data set such as `custom_v1` or `melee_v0`, or `None` for the base data.
    /// Maps choose this in their `war3map.w3i`.
    pub balance: Option<String>,
    pub locale: String,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            balance: None,
            locale: DEFAULT_LOCALE.to_owned(),
        }
    }
}

pub struct GameFiles {
    storage: casc::Storage,
    options: GameOptions,
}

impl GameFiles {
    pub fn open(game_dir: &Path, options: GameOptions) -> Result<Self> {
        let storage = casc::Storage::open(game_dir)?;

        Ok(Self { storage, options })
    }

    pub fn options(&self) -> &GameOptions {
        &self.options
    }

    pub fn storage(&self) -> &casc::Storage {
        &self.storage
    }

    pub fn read_base(&self, path: &str) -> Result<Vec<u8>> {
        Ok(self.storage.read(&format!("{ROOT}{path}"))?)
    }

    pub fn read_balanced(&self, path: &str) -> Result<Vec<u8>> {
        if let Some(set) = &self.options.balance {
            let layered = format!("{ROOT}_balance\\{set}.w3mod:{path}");

            if let Some(data) = self.storage.read_optional(&layered)? {
                return Ok(data);
            }
        }

        self.read_base(path)
    }

    pub fn read_localized(&self, path: &str) -> Result<Vec<u8>> {
        let layered = format!("{ROOT}_locales\\{}.w3mod:{path}", self.options.locale);

        if let Some(data) = self.storage.read_optional(&layered)? {
            return Ok(data);
        }

        self.read_base(path)
    }

    /// Reads an art file (icon, texture, model) for the given art mode. When that
    /// mode has no version of the file, the other mode's version is used: some
    /// art, such as several campaign icons, only exists in HD. Art found in
    /// neither comes from the `_de.w3mod:` layer.
    pub fn read_art(&self, path: &str, mode: ArtMode) -> Result<Vec<u8>> {
        let base = format!("{ROOT}{path}");
        let hd = format!("{ROOT}{HD_LAYER}{path}");
        let extra = format!("{ROOT}{EXTRA_ART_LAYER}{path}");
        let order = match mode {
            ArtMode::Sd => [&base, &hd, &extra],
            ArtMode::Hd => [&hd, &base, &extra],
        };

        for name in order {
            if let Some(data) = self.storage.read_optional(name)? {
                return Ok(data);
            }
        }

        Err(casc::Error::NotFound(base).into())
    }
}

pub fn decode_text(data: Vec<u8>, path: &str) -> Result<String> {
    let text = String::from_utf8(data).map_err(|e| {
        Error::Parse {
            line: 0,
            reason: format!("not valid UTF-8 at byte {}", e.utf8_error().valid_up_to()),
        }
        .in_file(path)
    })?;

    Ok(text
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(text))
}
