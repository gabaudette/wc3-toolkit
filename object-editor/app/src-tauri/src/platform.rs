//! Where Warcraft III is usually installed, per platform (AGE-TECH-005:
//! platform-specific by requirement, kept in this one module).

use std::path::{Path, PathBuf};

/// Files a Reforged install folder contains.
const INSTALL_MARKERS: &[&str] = &["Data", ".build.info"];
const GAME_FOLDER: &str = "Warcraft III";

/// The first folder that looks like a Warcraft III install.
pub fn detect_game_dir() -> Option<PathBuf> {
    candidates().into_iter().find(|dir| is_game_dir(dir))
}

pub fn is_game_dir(dir: &Path) -> bool {
    INSTALL_MARKERS.iter().all(|m| dir.join(m).exists())
}

/// The `Documents/Warcraft III/Maps` folder belonging to an install, for the
/// open-map dialog. Inside a Wine/Proton prefix that is the prefix's user folder.
pub fn maps_dir(game_dir: &Path) -> Option<PathBuf> {
    let drive_c = game_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "drive_c"));
    let documents = match drive_c {
        Some(drive_c) => wine_documents(drive_c)?,
        None => home()?.join("Documents"),
    };

    let maps = documents.join(GAME_FOLDER).join("Maps");
    maps.is_dir().then_some(maps)
}

fn wine_documents(drive_c: &Path) -> Option<PathBuf> {
    let users = std::fs::read_dir(drive_c.join("users")).ok()?;
    users
        .filter_map(|entry| entry.ok().map(|e| e.path().join("Documents")))
        .find(|docs| docs.join(GAME_FOLDER).is_dir())
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    ["C:\\Program Files (x86)", "C:\\Program Files"]
        .iter()
        .map(|p| Path::new(p).join(GAME_FOLDER))
        .collect()
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<PathBuf> {
    vec![Path::new("/Applications").join(GAME_FOLDER)]
}

/// Steam (Proton) prefixes and the default Wine prefix.
#[cfg(all(unix, not(target_os = "macos")))]
fn candidates() -> Vec<PathBuf> {
    const STEAM_ROOTS: &[&str] = &[
        ".steam/steam",
        ".steam/debian-installation",
        ".local/share/Steam",
    ];
    const PREFIX_GAME_DIR: &str = "pfx/drive_c/Program Files (x86)/Warcraft III";

    let Some(home) = home() else {
        return Vec::new();
    };

    let mut found = Vec::new();
    for root in STEAM_ROOTS {
        let Ok(prefixes) = std::fs::read_dir(home.join(root).join("steamapps/compatdata")) else {
            continue;
        };

        found.extend(prefixes.filter_map(|p| p.ok().map(|p| p.path().join(PREFIX_GAME_DIR))));
    }

    found.push(
        home.join(".wine/drive_c/Program Files (x86)")
            .join(GAME_FOLDER),
    );
    found
}
