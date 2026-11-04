//! Installing, updating and removing Tinkerbench for the current user, without
//! administrator rights. `install.json` beside the app's settings lists every
//! file written, so removal deletes exactly those.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wc3_install::{APP_IDENTIFIER, ChoicesError, SetupChoices};

use crate::payload;
use crate::platform;

const RECORD_FILE: &str = "install.json";
const REPLACEMENT_SUFFIX: &str = ".new";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRecord {
    pub version: String,
    pub dir: PathBuf,
    pub binary: PathBuf,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error(
        "This copy of the setup does not contain Tinkerbench. Download Tinkerbench Setup from the releases page."
    )]
    NoPayload,
    #[error("This system has no folder for your programs and settings.")]
    NoUserFolders,
    #[error("Tinkerbench is running. Close it, then choose Install again.")]
    AppRunning,
    #[error("Cannot {action} {}: {source}", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[cfg(windows)]
    #[error("Cannot write the Add/Remove Programs entry ({key}): {message}")]
    Registry { key: String, message: String },
    #[cfg(windows)]
    #[error("Cannot create the shortcut {}: {message}", .path.display())]
    Shortcut { path: PathBuf, message: String },
    #[error("{} is not an install record the setup can read: {source}", .path.display())]
    Record {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(transparent)]
    Choices(#[from] ChoicesError),
}

pub fn io_error(action: &'static str, path: &Path) -> impl FnOnce(std::io::Error) -> InstallError {
    let path = path.to_path_buf();

    move |source| InstallError::Io {
        action,
        path,
        source,
    }
}

/// What the wizard collected.
#[derive(Debug, Clone)]
pub struct Plan {
    pub dir: PathBuf,
    pub desktop_shortcut: bool,
    pub choices: SetupChoices,
}

fn settings_folder() -> Result<PathBuf, InstallError> {
    dirs::config_dir()
        .map(|dir| dir.join(APP_IDENTIFIER))
        .ok_or(InstallError::NoUserFolders)
}

impl InstallRecord {
    fn file() -> Result<PathBuf, InstallError> {
        Ok(settings_folder()?.join(RECORD_FILE))
    }

    /// The current install, or `None` when Tinkerbench Setup has not installed it.
    pub fn load() -> Result<Option<Self>, InstallError> {
        let path = Self::file()?;
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_error("read", &path)(e)),
        };

        serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| InstallError::Record { path, source })
    }

    fn save(&self) -> Result<(), InstallError> {
        let path = Self::file()?;
        let text = serde_json::to_string_pretty(self).map_err(|source| InstallError::Record {
            path: path.clone(),
            source,
        })?;

        write_file(&path, text.as_bytes())
    }
}

pub fn default_dir() -> Option<PathBuf> {
    platform::default_dir()
}

/// Creates the parent folders, then writes the file.
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_error("create", dir))?;
    }

    std::fs::write(path, bytes).map_err(io_error("write", path))
}

/// Writes beside the old binary, then renames over it: Windows refuses to
/// replace a running program, which is how a running Tinkerbench shows up.
fn replace_binary(path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    let mut staged = path.as_os_str().to_owned();

    staged.push(REPLACEMENT_SUFFIX);

    let staged = PathBuf::from(staged);

    write_file(&staged, bytes)?;
    platform::make_executable(&staged)?;

    match std::fs::rename(&staged, path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            std::fs::remove_file(&staged).ok();
            Err(InstallError::AppRunning)
        }
        Err(e) => Err(io_error("replace", path)(e)),
    }
}

/// Installs or updates Tinkerbench. `report` hears each step as it starts.
pub fn install(plan: &Plan, report: &mut dyn FnMut(&str)) -> Result<InstallRecord, InstallError> {
    if !payload::has_app() {
        return Err(InstallError::NoPayload);
    }

    report("Unpacking Tinkerbench");

    let bytes = payload::app_binary().map_err(io_error("unpack", Path::new("Tinkerbench")))?;
    let binary = plan.dir.join(platform::BINARY);
    let license = plan.dir.join(payload::LICENSE_FILE);
    let notices = plan.dir.join(payload::NOTICES_FILE);

    replace_binary(&binary, &bytes)?;
    write_file(&license, payload::LICENSE.as_bytes())?;
    write_file(&notices, payload::APP_NOTICES.as_bytes())?;

    report("Adding Tinkerbench to your applications");

    let mut files = vec![binary.clone(), license, notices];

    platform::integrate(plan, &binary, &mut files)?;

    report("Saving your choices");
    plan.choices.save()?;

    let record = InstallRecord {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        dir: plan.dir.clone(),
        binary,
        files,
    };

    record.save()?;
    platform::refresh_menus();
    Ok(record)
}

/// Removes what the record lists. With `remove_settings`, also the app's
/// settings, browser data and logs.
pub fn uninstall(record: &InstallRecord, remove_settings: bool) -> Result<(), InstallError> {
    let running = std::env::current_exe()
        .and_then(|exe| exe.canonicalize())
        .ok();
    let mut running_in_record = None;

    for file in &record.files {
        if running.is_some() && file.canonicalize().ok() == running {
            running_in_record = Some(file);
            continue;
        }

        match std::fs::remove_file(file) {
            Err(e) if e.kind() != ErrorKind::NotFound => {
                return Err(io_error("remove", file)(e));
            }
            _ => {}
        }
    }

    std::fs::remove_dir(&record.dir).ok();

    let record_file = InstallRecord::file()?;

    match std::fs::remove_file(&record_file) {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            return Err(io_error("remove", &record_file)(e));
        }
        _ => {}
    }

    if remove_settings {
        for dir in app_data_folders() {
            match std::fs::remove_dir_all(&dir) {
                Err(e) if e.kind() != ErrorKind::NotFound => {
                    return Err(io_error("remove", &dir)(e));
                }
                _ => {}
            }
        }
    }

    platform::unregister()?;
    platform::refresh_menus();

    if let Some(file) = running_in_record {
        platform::remove_after_exit(file, &record.dir);
    }

    Ok(())
}

/// Where the app keeps settings, browser data and logs, all named after its identifier.
fn app_data_folders() -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = [
        dirs::config_dir(),
        dirs::data_dir(),
        dirs::data_local_dir(),
        dirs::cache_dir(),
    ]
    .into_iter()
    .flatten()
    .map(|dir| dir.join(APP_IDENTIFIER))
    .collect();

    folders.sort();
    folders.dedup();
    folders
}

pub fn launch(binary: &Path) -> std::io::Result<()> {
    let mut command = std::process::Command::new(binary);

    if let Some(home) = dirs::home_dir() {
        command.current_dir(home);
    }

    command.spawn().map(drop)
}
