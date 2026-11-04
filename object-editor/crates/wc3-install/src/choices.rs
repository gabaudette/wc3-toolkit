//! The choices Tinkerbench Setup makes for the app: `setup.json` in the app's
//! config folder, read on the app's first launch.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The app's Tauri identifier, which names its config folder.
pub const APP_IDENTIFIER: &str = "io.github.gabaudette.wc3toolkit";
const CHOICES_FILE: &str = "setup.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupChoices {
    pub game_dir: PathBuf,
    #[serde(default)]
    pub hd: bool,
    #[serde(default)]
    pub maps_dir: Option<PathBuf>,
    /// One folder for every map's Lua modules; `None` keeps `<map> Lua` beside each map.
    #[serde(default)]
    pub lua_folder: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChoicesError {
    #[error("This system has no config folder for Tinkerbench's settings.")]
    NoConfigFolder,
    #[error("Cannot read {}: {source}", .path.display())]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Cannot write {}: {source}", .path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{} is not a settings file Tinkerbench can read: {source}", .path.display())]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl SetupChoices {
    pub fn file() -> Result<PathBuf, ChoicesError> {
        dirs::config_dir()
            .map(|dir| dir.join(APP_IDENTIFIER).join(CHOICES_FILE))
            .ok_or(ChoicesError::NoConfigFolder)
    }

    /// The saved choices, or `None` when Tinkerbench Setup has not run.
    pub fn load() -> Result<Option<Self>, ChoicesError> {
        Self::load_from(&Self::file()?)
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, ChoicesError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ChoicesError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };

        serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| ChoicesError::Parse {
                path: path.to_path_buf(),
                source,
            })
    }

    pub fn save(&self) -> Result<PathBuf, ChoicesError> {
        let path = Self::file()?;

        self.save_to(&path)?;
        Ok(path)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), ChoicesError> {
        let write = |source| ChoicesError::Write {
            path: path.to_path_buf(),
            source,
        };

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(write)?;
        }

        let text = serde_json::to_string_pretty(self).map_err(|source| ChoicesError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        std::fs::write(path, text).map_err(write)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_survive_a_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config").join(CHOICES_FILE);
        let choices = SetupChoices {
            game_dir: PathBuf::from("/games/Warcraft III"),
            hd: true,
            maps_dir: Some(PathBuf::from("/docs/Warcraft III/Maps")),
            lua_folder: None,
        };

        assert_eq!(SetupChoices::load_from(&path).unwrap(), None);
        choices.save_to(&path).unwrap();
        assert_eq!(SetupChoices::load_from(&path).unwrap(), Some(choices));
    }

    #[test]
    fn only_the_game_folder_is_required() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CHOICES_FILE);

        std::fs::write(&path, r#"{ "gameDir": "/games/Warcraft III" }"#).unwrap();

        let choices = SetupChoices::load_from(&path).unwrap().unwrap();

        assert!(!choices.hd);
        assert_eq!(choices.maps_dir, None);
    }

    #[test]
    fn a_broken_file_names_itself() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CHOICES_FILE);

        std::fs::write(&path, "{").unwrap();

        let error = SetupChoices::load_from(&path).unwrap_err();

        assert!(error.to_string().contains(CHOICES_FILE));
    }
}
