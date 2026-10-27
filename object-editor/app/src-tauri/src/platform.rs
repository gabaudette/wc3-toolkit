//! Where Warcraft III is usually installed, per platform (AGE-TECH-005:
//! platform-specific by requirement, kept in this one module).

use std::path::{Path, PathBuf};

/// Keeps WebKitGTK's threads at normal priority on Linux. WebKitGTK asks
/// RealtimeKit, over the system D-Bus, to make a few of its IPC threads
/// real-time (`SCHED_RR`), which RealtimeKit grants only under a CPU limit
/// (`RLIMIT_RTTIME`, 200 ms by default): a real-time thread that stays busy that
/// long gets its whole process killed with SIGKILL. Browsing models streams
/// enough data through those threads to reach it, so the app closed without a
/// word (AGE-46). Real-time priority gains this app nothing, so its processes
/// get a system bus address that cannot exist: the request fails and the
/// threads stay normal. Nothing else here uses the system bus (file dialogs
/// and accessibility use the session bus).
#[cfg(all(unix, not(target_os = "macos")))]
#[allow(unsafe_code)]
pub fn keep_threads_at_normal_priority() {
    const SYSTEM_BUS_VAR: &str = "DBUS_SYSTEM_BUS_ADDRESS";
    const NO_SYSTEM_BUS: &str = "unix:path=/dev/null/no-system-bus";

    // SAFETY: called first in `main`, before any other thread exists, so
    // nothing reads the environment while it changes. WebKit's processes
    // inherit it when they start.
    unsafe { std::env::set_var(SYSTEM_BUS_VAR, NO_SYSTEM_BUS) };
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
pub fn keep_threads_at_normal_priority() {}

/// Files a Reforged install folder contains.
const INSTALL_MARKERS: &[&str] = &["Data", ".build.info"];
const GAME_FOLDER: &str = "Warcraft III";

/// The most likely Warcraft III install. Several can exist (e.g. one per
/// Proton prefix); prefer one whose user folder has maps, then the most
/// recently updated (`.build.info` changes with every game update).
pub fn detect_game_dir() -> Option<PathBuf> {
    let updated = |dir: &Path| {
        std::fs::metadata(dir.join(".build.info"))
            .and_then(|m| m.modified())
            .ok()
    };

    candidates()
        .into_iter()
        .filter(|dir| is_game_dir(dir))
        .max_by_key(|dir| (maps_dir(dir).is_some(), updated(dir)))
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
