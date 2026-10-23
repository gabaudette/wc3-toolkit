//! What the app has open: the game data, the map, and decoded icons.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::{ArtMode, GameFiles};
use wc3_data::guide::FieldGuide;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::Map;
use wc3_map::abilities::MapAbilities;
use wc3_map::imports::{MapImports, candidates};
use wc3_map::plain::{BUFFS, DESTRUCTABLES, ITEMS, PlainObjects};
use wc3_map::units::MapUnits;
use wc3_map::upgrades::MapUpgrades;

pub struct Session {
    pub files: GameFiles,
    pub db: UnitDatabase,
    pub guide: FieldGuide,
    pub abilities: AbilityDatabase,
    pub ability_guide: FieldGuide,
    /// Items: the unit metadata's item fields over the item tables.
    pub items: UnitDatabase,
    pub item_guide: FieldGuide,
    pub buffs: UnitDatabase,
    pub buff_guide: FieldGuide,
    pub destructables: UnitDatabase,
    pub destructable_guide: FieldGuide,
    pub upgrades: UpgradeDatabase,
    pub upgrade_guide: FieldGuide,
    pub mode: ArtMode,
    pub map: Option<OpenMap>,
    /// PNG bytes by `(art mode, game path)`, so each icon is decoded once.
    pub icons: HashMap<(ArtMode, String), Vec<u8>>,
}

pub struct OpenMap {
    pub path: PathBuf,
    /// Units, and the map's string table.
    pub units: MapUnits,
    pub abilities: MapAbilities,
    pub items: PlainObjects,
    pub upgrades: MapUpgrades,
    pub buffs: PlainObjects,
    pub destructables: PlainObjects,
    /// The import list, and files imports queued for the next save.
    pub imports: MapImports,
    /// The files as last loaded or saved, to tell whether anything changed.
    pub saved_files: Vec<(&'static str, Vec<u8>)>,
    /// PNG bytes of icons the map holds, by the path asked for. Files in a
    /// map never change while it is open; imports only add new ones.
    pub icons: HashMap<String, Vec<u8>>,
}

impl OpenMap {
    /// Reads every object file, the string table and the import list.
    pub fn load(path: PathBuf) -> wc3_map::Result<Self> {
        let archive = Map::open(&path)?;
        let mut map = Self {
            units: MapUnits::load(&archive)?,
            abilities: MapAbilities::load(&archive)?,
            items: PlainObjects::load(&archive, ITEMS)?,
            upgrades: MapUpgrades::load(&archive)?,
            buffs: PlainObjects::load(&archive, BUFFS)?,
            destructables: PlainObjects::load(&archive, DESTRUCTABLES)?,
            imports: MapImports::load(&archive)?,
            path,
            saved_files: Vec::new(),
            icons: HashMap::new(),
        };
        map.saved_files = map.files();
        Ok(map)
    }

    /// Every file the editor writes into the map.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut files = self.units.files();
        files.extend(self.abilities.files());
        files.extend(self.items.files());
        files.extend(self.upgrades.files());
        files.extend(self.buffs.files());
        files.extend(self.destructables.files());
        files
    }

    /// Whether the map differs from the file on disk. Undoing every change
    /// by hand counts as no change.
    pub fn unsaved(&self) -> bool {
        self.files() != self.saved_files || self.imports.has_pending()
    }

    /// Reads a file from the map: one an import queued, or else from the
    /// file on disk (opened once per reader).
    pub fn reader(&self) -> impl Fn(&str) -> Option<Vec<u8>> + '_ {
        let archive = Map::open(&self.path).ok();
        move |path| {
            self.imports
                .pending(path)
                .map(<[u8]>::to_vec)
                .or_else(|| archive.as_ref()?.read_file(path).ok().flatten())
        }
    }

    /// A model or texture the map holds for an art path as data or a model
    /// names it (`Haramad.mdl` → `Haramad.mdx`, `x.blp` → also `x.dds`), with
    /// the path it was found at. `None` when the map has none, so the game's
    /// file applies.
    pub fn art(&self, path: &str) -> Option<(String, Vec<u8>)> {
        let read = self.reader();
        candidates(path)
            .into_iter()
            .find_map(|file| read(&file).map(|bytes| (file, bytes)))
    }
}

/// Shared by all commands. `None` until a game folder is opened.
#[derive(Default)]
pub struct AppState(Mutex<Option<Session>>);

pub const NO_GAME: &str = "Open your Warcraft III folder first.";
pub const NO_MAP: &str = "Open a map first. Changes are saved into a map.";

impl AppState {
    pub fn replace(&self, session: Session) -> Result<(), String> {
        *self.0.lock().map_err(|_| poisoned())? = Some(session);
        Ok(())
    }

    /// Runs `f` with the open session, or fails with [`NO_GAME`].
    pub fn with<T>(&self, f: impl FnOnce(&mut Session) -> Result<T, String>) -> Result<T, String> {
        let mut guard = self.0.lock().map_err(|_| poisoned())?;
        let session = guard.as_mut().ok_or(NO_GAME)?;
        f(session)
    }
}

/// A command panicked while holding the lock; the data may be half-updated.
fn poisoned() -> String {
    "The editor hit an internal error. Restart it; unsaved changes are lost.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3_map::imports::ImportedFile;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const MAX_FILES: u32 = 8;

    #[test]
    fn art_comes_from_the_map_or_its_queued_imports() -> TestResult {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3x");
        let mut archive = mpq::Archive::create(&path, MAX_FILES)?;
        archive.write("Haramad.mdx", b"model")?;
        archive.write("Textures\\Skin.dds", b"texture")?;
        archive.save()?;

        let mut open = OpenMap::load(path.clone())?;
        open.imports.add(ImportedFile {
            path: "war3mapImported\\BTNHero.blp".to_owned(),
            bytes: b"icon".to_vec(),
        });

        let found = |art: &str| open.art(art).map(|(file, bytes)| (file, bytes.len()));
        assert_eq!(found("Haramad.mdl"), Some(("Haramad.mdx".to_owned(), 5)));
        assert_eq!(
            found("Textures/Skin.blp"),
            Some(("Textures\\Skin.dds".to_owned(), 7))
        );
        assert_eq!(
            found("war3mapImported\\BTNHero.blp"),
            Some(("war3mapImported\\BTNHero.blp".to_owned(), 4))
        );
        assert_eq!(found("units\\human\\Footman\\Footman"), None);
        Ok(())
    }
}
