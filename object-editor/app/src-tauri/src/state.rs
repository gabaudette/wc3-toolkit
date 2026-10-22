//! What the app has open: the game data, the map, and decoded icons.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use wc3_data::abilities::AbilityDatabase;
use wc3_data::game::{ArtMode, GameFiles};
use wc3_data::guide::FieldGuide;
use wc3_data::units::UnitDatabase;
use wc3_map::Map;
use wc3_map::abilities::MapAbilities;
use wc3_map::imports::MapImports;
use wc3_map::transfer::MapExtras;
use wc3_map::units::MapUnits;

pub struct Session {
    pub files: GameFiles,
    pub db: UnitDatabase,
    pub guide: FieldGuide,
    pub abilities: AbilityDatabase,
    pub ability_guide: FieldGuide,
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
    /// Buffs: only imports change them.
    pub extras: MapExtras,
    /// The import list, and files imports queued for the next save.
    pub imports: MapImports,
    /// The files as last loaded or saved, to tell whether anything changed.
    pub saved_files: Vec<(&'static str, Vec<u8>)>,
}

impl OpenMap {
    pub fn new(
        path: PathBuf,
        units: MapUnits,
        abilities: MapAbilities,
        extras: MapExtras,
        imports: MapImports,
    ) -> Self {
        let mut map = Self {
            path,
            units,
            abilities,
            extras,
            imports,
            saved_files: Vec::new(),
        };
        map.saved_files = map.files();
        map
    }

    /// Every file the editor writes into the map.
    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut files = self.units.files();
        files.extend(self.abilities.files());
        files.extend(self.extras.files());
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
