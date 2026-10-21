//! What the app has open: the game data, the map, and decoded icons.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use wc3_data::game::{ArtMode, GameFiles};
use wc3_data::guide::FieldGuide;
use wc3_data::units::UnitDatabase;
use wc3_map::units::MapUnits;

pub struct Session {
    pub files: GameFiles,
    pub db: UnitDatabase,
    pub guide: FieldGuide,
    pub mode: ArtMode,
    pub map: Option<OpenMap>,
    /// PNG bytes by `(art mode, game path)`, so each icon is decoded once.
    pub icons: HashMap<(ArtMode, String), Vec<u8>>,
}

pub struct OpenMap {
    pub path: PathBuf,
    pub units: MapUnits,
    /// The unit files as last loaded or saved, to tell whether anything changed.
    pub saved_files: Vec<(&'static str, Vec<u8>)>,
}

impl OpenMap {
    pub fn new(path: PathBuf, units: MapUnits) -> Self {
        let saved_files = units.files();
        Self {
            path,
            units,
            saved_files,
        }
    }

    /// Whether the map differs from the file on disk. Undoing every change
    /// by hand counts as no change.
    pub fn unsaved(&self) -> bool {
        self.units.files() != self.saved_files
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
