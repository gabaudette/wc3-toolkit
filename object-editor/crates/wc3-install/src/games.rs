//! Finding Warcraft III installs: Battle.net on Windows, and on Linux the Wine
//! prefixes of Steam (Proton), Heroic, Bottles, Lutris and plain Wine.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;

/// Files a Reforged install folder contains.
const INSTALL_MARKERS: &[&str] = &["Data", BUILD_INFO];
const BUILD_INFO: &str = ".build.info";
const GAME_FOLDER: &str = "Warcraft III";
const VERSION_COLUMN: &str = "Version";
const ACTIVE_COLUMN: &str = "Active";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GameSource {
    BattleNet,
    Folder,
    Steam,
    Heroic,
    Bottles,
    Lutris,
    Wine,
}

impl GameSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::BattleNet => "Battle.net",
            Self::Folder => "This computer",
            Self::Steam => "Steam (Proton)",
            Self::Heroic => "Heroic",
            Self::Bottles => "Bottles",
            Self::Lutris => "Lutris",
            Self::Wine => "Wine",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundGame {
    pub dir: PathBuf,
    pub source: GameSource,
    pub version: Option<String>,
    pub maps_dir: Option<PathBuf>,
}

pub fn is_game_dir(dir: &Path) -> bool {
    INSTALL_MARKERS.iter().all(|m| dir.join(m).exists())
}

/// The `Documents/Warcraft III/Maps` folder belonging to an install. Inside a
/// Wine prefix that is the prefix's user folder.
pub fn maps_dir(game_dir: &Path) -> Option<PathBuf> {
    let drive_c = game_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "drive_c"));
    let documents = match drive_c {
        Some(drive_c) => wine_documents(drive_c)?,
        None => dirs::document_dir()?,
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

/// The game version Battle.net installed, from the active row of `.build.info`.
pub fn game_version(dir: &Path) -> Option<String> {
    parse_build_info(&std::fs::read_to_string(dir.join(BUILD_INFO)).ok()?)
}

/// `.build.info` is a table: a header row of `Name!TYPE:size` columns, then one
/// row per region, separated by `|`.
fn parse_build_info(text: &str) -> Option<String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<&str> = lines
        .next()?
        .split('|')
        .map(|column| column.split('!').next().unwrap_or(column))
        .collect();
    let version = header.iter().position(|c| *c == VERSION_COLUMN)?;
    let active = header.iter().position(|c| *c == ACTIVE_COLUMN);
    let rows: Vec<Vec<&str>> = lines.map(|l| l.split('|').collect()).collect();
    let row = rows
        .iter()
        .find(|row| active.is_some_and(|a| row.get(a) == Some(&"1")))
        .or_else(|| rows.first())?;

    row.get(version)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// Every install found, the most likely first: one whose user folder has maps,
/// then the most recently updated (`.build.info` changes with every game update).
pub fn find_games() -> Vec<FoundGame> {
    found(candidates())
}

fn found(candidates: Vec<(PathBuf, GameSource)>) -> Vec<FoundGame> {
    let mut seen = HashSet::new();
    let mut games: Vec<(FoundGame, Option<SystemTime>)> = candidates
        .into_iter()
        .filter(|(dir, _)| is_game_dir(dir))
        .filter(|(dir, _)| seen.insert(dir.canonicalize().unwrap_or_else(|_| dir.clone())))
        .map(|(dir, source)| {
            let updated = std::fs::metadata(dir.join(BUILD_INFO))
                .and_then(|m| m.modified())
                .ok();
            let game = FoundGame {
                version: game_version(&dir),
                maps_dir: maps_dir(&dir),
                dir,
                source,
            };

            (game, updated)
        })
        .collect();

    games.sort_by(|(a, a_updated), (b, b_updated)| {
        (b.maps_dir.is_some(), b_updated).cmp(&(a.maps_dir.is_some(), a_updated))
    });

    games.into_iter().map(|(game, _)| game).collect()
}

#[cfg(windows)]
fn candidates() -> Vec<(PathBuf, GameSource)> {
    windows::candidates()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn candidates() -> Vec<(PathBuf, GameSource)> {
    dirs::home_dir()
        .map(|home| wine::candidates(&home))
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<(PathBuf, GameSource)> {
    vec![(
        Path::new("/Applications").join(GAME_FOLDER),
        GameSource::Folder,
    )]
}

#[cfg(windows)]
mod windows {
    use std::path::{Path, PathBuf};

    use super::{GAME_FOLDER, GameSource};

    /// Where Battle.net records the install. Unconfirmed on a Reforged install:
    /// the drive scan below finds the default locations either way.
    const REGISTRY_VALUES: &[(&str, &str)] = &[
        (
            r"SOFTWARE\WOW6432Node\Blizzard Entertainment\Warcraft III",
            "InstallPath",
        ),
        (
            r"SOFTWARE\Blizzard Entertainment\Warcraft III",
            "InstallPath",
        ),
        (
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Warcraft III",
            "InstallLocation",
        ),
        (
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Warcraft III",
            "InstallLocation",
        ),
    ];
    const DRIVE_FOLDERS: &[&str] = &["Program Files (x86)", "Program Files", "Games", ""];
    /// A and B are floppy drives; probing an empty one can stall.
    const FIRST_DRIVE: u8 = b'C';
    const LAST_DRIVE: u8 = b'Z';

    pub(super) fn candidates() -> Vec<(PathBuf, GameSource)> {
        let mut found: Vec<(PathBuf, GameSource)> = registry_installs()
            .into_iter()
            .map(|dir| (dir, GameSource::BattleNet))
            .collect();

        for letter in FIRST_DRIVE..=LAST_DRIVE {
            let drive = PathBuf::from(format!("{}:\\", char::from(letter)));

            if !drive.is_dir() {
                continue;
            }

            found.extend(
                DRIVE_FOLDERS
                    .iter()
                    .map(|folder| (drive.join(folder).join(GAME_FOLDER), GameSource::Folder)),
            );
        }

        found
    }

    fn registry_installs() -> Vec<PathBuf> {
        let mut found = Vec::new();

        for hive in [
            windows_registry::LOCAL_MACHINE,
            windows_registry::CURRENT_USER,
        ] {
            for (key, value) in REGISTRY_VALUES {
                let Ok(key) = hive.open(key) else {
                    continue;
                };

                if let Ok(dir) = key.get_string(value)
                    && !dir.trim().is_empty()
                {
                    found.push(Path::new(dir.trim()).to_path_buf());
                }
            }
        }

        found
    }
}

#[cfg(any(test, all(unix, not(target_os = "macos"))))]
mod wine {
    use std::path::{Path, PathBuf};

    use super::{GAME_FOLDER, GameSource};

    const STEAM_ROOTS: &[&str] = &[
        ".steam/steam",
        ".steam/debian-installation",
        ".local/share/Steam",
        ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        ".var/app/com.valvesoftware.Steam/data/Steam",
    ];
    const LIBRARY_LIST: &str = "steamapps/libraryfolders.vdf";
    const COMPAT_DATA: &str = "steamapps/compatdata";
    /// Folders holding Wine prefixes, with how deep below them a prefix can be.
    /// Heroic comes before Lutris, whose `~/Games` holds Heroic's folder too.
    const PREFIX_ROOTS: &[(&str, usize, GameSource)] = &[
        ("Games/Heroic/Prefixes", 3, GameSource::Heroic),
        (".local/share/bottles/bottles", 1, GameSource::Bottles),
        (
            ".var/app/com.usebottles.bottles/data/bottles/bottles",
            1,
            GameSource::Bottles,
        ),
        ("Games", 2, GameSource::Lutris),
        (".wine", 0, GameSource::Wine),
    ];
    /// `compatdata/<app id>/pfx`.
    const COMPAT_DEPTH: usize = 2;
    const PROGRAM_FOLDERS: &[&str] = &["Program Files (x86)", "Program Files"];

    pub(super) fn candidates(home: &Path) -> Vec<(PathBuf, GameSource)> {
        let mut found = Vec::new();

        for library in steam_libraries(home) {
            for prefix in prefixes(&library.join(COMPAT_DATA), COMPAT_DEPTH) {
                found.extend(games_in(&prefix, GameSource::Steam));
            }
        }

        for (root, depth, source) in PREFIX_ROOTS {
            for prefix in prefixes(&home.join(root), *depth) {
                found.extend(games_in(&prefix, *source));
            }
        }

        found
    }

    fn games_in(prefix: &Path, source: GameSource) -> Vec<(PathBuf, GameSource)> {
        PROGRAM_FOLDERS
            .iter()
            .map(|folder| {
                (
                    prefix.join("drive_c").join(folder).join(GAME_FOLDER),
                    source,
                )
            })
            .collect()
    }

    /// Each Steam install, and the other libraries it lists.
    fn steam_libraries(home: &Path) -> Vec<PathBuf> {
        let mut libraries = Vec::new();

        for root in STEAM_ROOTS.iter().map(|r| home.join(r)) {
            if !root.is_dir() {
                continue;
            }

            if let Ok(text) = std::fs::read_to_string(root.join(LIBRARY_LIST)) {
                libraries.extend(library_paths(&text));
            }

            libraries.push(root);
        }

        libraries
    }

    /// The `"path"` entries of `libraryfolders.vdf`.
    pub(super) fn library_paths(vdf: &str) -> Vec<PathBuf> {
        vdf.lines()
            .filter_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix("\"path\"")?.trim();
                let value = rest.strip_prefix('"')?.strip_suffix('"')?;

                Some(PathBuf::from(value.replace("\\\\", "\\")))
            })
            .collect()
    }

    /// The Wine prefixes (folders with a `drive_c`) at most `depth` levels below
    /// `root`. Symbolic links below the root are not followed, so loops cannot form.
    pub(super) fn prefixes(root: &Path, depth: usize) -> Vec<PathBuf> {
        let mut found = Vec::new();

        collect_prefixes(root, depth, &mut found);
        found
    }

    fn collect_prefixes(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if dir.join("drive_c").is_dir() {
            found.push(dir.to_path_buf());
            return;
        }

        if depth == 0 {
            return;
        }

        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                collect_prefixes(&entry.path(), depth - 1, found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const BUILD_INFO_TEXT: &str = "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\n\
        eu|0|aaaa|2.0.0.1|w3\n\
        us|1|bbbb|2.0.3.22988|w3\n";

    fn install(dir: &Path) {
        fs::create_dir_all(dir.join("Data")).unwrap();
        fs::write(dir.join(BUILD_INFO), BUILD_INFO_TEXT).unwrap();
    }

    fn prefix_game(prefix: &Path) -> PathBuf {
        prefix
            .join("drive_c")
            .join("Program Files (x86)")
            .join(GAME_FOLDER)
    }

    #[test]
    fn the_active_row_gives_the_version() {
        assert_eq!(
            parse_build_info(BUILD_INFO_TEXT).as_deref(),
            Some("2.0.3.22988")
        );
        assert_eq!(parse_build_info("Branch!STRING:0\nus\n"), None);
        assert_eq!(parse_build_info(""), None);
    }

    #[test]
    fn library_folders_list_every_steam_library() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"/home/me/.local/share/Steam\"\n\t\t\"label\"\t\t\"\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"/mnt/games/SteamLibrary\"\n\t}\n}\n";

        assert_eq!(
            wine::library_paths(vdf),
            vec![
                PathBuf::from("/home/me/.local/share/Steam"),
                PathBuf::from("/mnt/games/SteamLibrary"),
            ]
        );
    }

    #[test]
    fn games_are_found_in_every_kind_of_prefix() {
        let home = tempfile::tempdir().unwrap();
        let home = home.path();
        let library = home.join("other-disk/SteamLibrary");
        let steam = home.join(".local/share/Steam");

        fs::create_dir_all(steam.join("steamapps")).unwrap();
        fs::write(
            steam.join("steamapps/libraryfolders.vdf"),
            format!("\"path\" \"{}\"\n", library.display()),
        )
        .unwrap();

        let proton = prefix_game(&library.join("steamapps/compatdata/123/pfx"));
        let heroic = prefix_game(&home.join("Games/Heroic/Prefixes/default/Warcraft/pfx"));
        let bottles = prefix_game(&home.join(".local/share/bottles/bottles/Battlenet"));
        let lutris = prefix_game(&home.join("Games/battlenet"));
        let wine = prefix_game(&home.join(".wine"));
        let not_installed = home.join(".wine/drive_c/Program Files/Warcraft III");

        for dir in [&proton, &heroic, &bottles, &lutris, &wine] {
            install(dir);
        }

        fs::create_dir_all(&not_installed).unwrap();

        let games = found(wine::candidates(home));
        let sources: Vec<(PathBuf, GameSource)> =
            games.iter().map(|g| (g.dir.clone(), g.source)).collect();

        assert_eq!(games.len(), 5);
        for expected in [
            (proton, GameSource::Steam),
            (heroic, GameSource::Heroic),
            (bottles, GameSource::Bottles),
            (lutris, GameSource::Lutris),
            (wine, GameSource::Wine),
        ] {
            assert!(sources.contains(&expected), "missing {expected:?}");
        }

        assert!(
            games
                .iter()
                .all(|g| g.version.as_deref() == Some("2.0.3.22988"))
        );
    }

    #[test]
    fn an_install_with_maps_comes_first_and_duplicates_are_dropped() {
        let home = tempfile::tempdir().unwrap();
        let without_maps = prefix_game(&home.path().join("a"));
        let prefix = home.path().join("b");
        let with_maps = prefix_game(&prefix);

        install(&without_maps);
        install(&with_maps);
        fs::create_dir_all(prefix.join("drive_c/users/me/Documents/Warcraft III/Maps")).unwrap();

        let games = found(vec![
            (without_maps.clone(), GameSource::Wine),
            (with_maps.clone(), GameSource::Wine),
            (with_maps.clone(), GameSource::Lutris),
        ]);

        assert_eq!(games.len(), 2);
        assert_eq!(games[0].dir, with_maps);
        assert_eq!(
            games[0].maps_dir,
            Some(prefix.join("drive_c/users/me/Documents/Warcraft III/Maps"))
        );
        assert_eq!(games[1].dir, without_maps);
    }

    #[test]
    fn prefix_search_stops_at_its_depth() {
        let root = tempfile::tempdir().unwrap();
        let deep = root.path().join("a/b/c");

        fs::create_dir_all(deep.join("drive_c")).unwrap();

        assert!(wine::prefixes(root.path(), 2).is_empty());
        assert_eq!(wine::prefixes(root.path(), 3), vec![deep]);
    }
}
