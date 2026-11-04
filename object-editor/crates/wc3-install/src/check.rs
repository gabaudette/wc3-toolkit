//! Checking that an install's data can be read, with reasons a map maker can act on.

use std::path::{Path, PathBuf};

use wc3_data::game::{GameFiles, GameOptions};

use crate::games::{game_version, is_game_dir};

const UNIT_TABLE: &str = "units\\abilitydata.slk";
const EDITOR_TEXT: &str = "ui\\worldeditstrings.txt";
const HD_ICON: &str = "replaceabletextures\\commandbuttons\\btnfootman.dds";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameCheck {
    pub version: Option<String>,
    /// Whether HD art is installed; the classic-graphics install leaves it out.
    pub hd_art: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GameProblem {
    #[error(
        "{} is not a Warcraft III folder: it has no Data folder and .build.info file. Choose the folder the game is installed in, not its Maps folder.",
        .0.display()
    )]
    NotAGameFolder(PathBuf),
    #[error(
        "Tinkerbench could not open the game data in {}. If Battle.net is updating or repairing the game, wait for it to finish, then try again. Details: {source}",
        .dir.display()
    )]
    Open {
        dir: PathBuf,
        source: wc3_data::Error,
    },
    #[error(
        "The game data opened, but its {what} could not be read. Repair the game in Battle.net (the gear next to Play, then Scan and Repair), then try again. Details: {source}"
    )]
    Read {
        what: &'static str,
        source: wc3_data::Error,
    },
}

pub fn open_game_files(dir: &Path) -> Result<GameFiles, GameProblem> {
    if !is_game_dir(dir) {
        return Err(GameProblem::NotAGameFolder(dir.to_path_buf()));
    }

    GameFiles::open(dir, GameOptions::default()).map_err(|source| GameProblem::Open {
        dir: dir.to_path_buf(),
        source,
    })
}

/// Opens the install and reads a unit table, the editor's text and an HD icon.
pub fn check_game(dir: &Path) -> Result<GameCheck, GameProblem> {
    let files = open_game_files(dir)?;

    files
        .read_base(UNIT_TABLE)
        .map_err(|source| GameProblem::Read {
            what: "unit tables",
            source,
        })?;
    files
        .read_localized(EDITOR_TEXT)
        .map_err(|source| GameProblem::Read {
            what: "text",
            source,
        })?;

    Ok(GameCheck {
        version: game_version(dir),
        hd_art: files.read_hd(HD_ICON).is_ok_and(|icon| icon.is_some()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_maps_folder_is_not_a_game() {
        let maps = tempfile::tempdir().unwrap();

        let problem = check_game(maps.path()).unwrap_err();

        assert!(matches!(problem, GameProblem::NotAGameFolder(_)));
        assert!(problem.to_string().contains("not its Maps folder"));
    }
}
