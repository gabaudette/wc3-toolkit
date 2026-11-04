//! Checks against a real Warcraft III: Reforged install. Ignored by default
//! because CI has no game files; run with:
//!
//! `WC3_GAME_DIR="/path/to/Warcraft III" cargo test -p wc3-install -- --ignored`

use std::path::PathBuf;

use wc3_install::{check_game, find_games};

const GAME_DIR_VAR: &str = "WC3_GAME_DIR";

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn game_dir() -> Result<PathBuf, String> {
    std::env::var_os(GAME_DIR_VAR)
        .map(PathBuf::from)
        .ok_or(format!(
            "set {GAME_DIR_VAR} to the Warcraft III install folder"
        ))
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn the_install_is_found_with_its_version_and_maps() -> TestResult {
    let dir = game_dir()?.canonicalize()?;
    let games = find_games();
    let game = games
        .iter()
        .find(|g| g.dir.canonicalize().is_ok_and(|found| found == dir))
        .ok_or(format!("{} was not found among {games:?}", dir.display()))?;

    assert!(game.version.is_some());
    assert!(game.maps_dir.is_some());
    Ok(())
}

#[test]
#[ignore = "needs a Warcraft III install (WC3_GAME_DIR)"]
fn the_install_reads_with_hd_art() -> TestResult {
    let check = check_game(&game_dir()?)?;

    assert!(check.version.is_some());
    assert!(check.hd_art);
    Ok(())
}
