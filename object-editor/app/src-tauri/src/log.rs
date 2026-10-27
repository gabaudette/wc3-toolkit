use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const FILE: &str = "tinkerbench.log";
const PREVIOUS_FILE: &str = "tinkerbench.previous.log";
const LOG_FOLDER: &str = "logs";
const MAX_BYTES: u64 = 1024 * 1024;

static PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warn,
    Error,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

pub fn folder(identifier: &str) -> Option<PathBuf> {
    Some(dirs::data_local_dir()?.join(identifier).join(LOG_FOLDER))
}

pub fn start(identifier: &str) {
    let Some(dir) = folder(identifier) else {
        return;
    };

    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }

    let path = dir.join(FILE);
    let too_big = std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES);

    if too_big && std::fs::rename(&path, dir.join(PREVIOUS_FILE)).is_err() {
        return;
    }

    if PATH.set(path).is_err() {
        return;
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        record(Level::Error, "panic", &info.to_string());
        default_hook(info);
    }));
}

pub fn record(level: Level, source: &str, message: &str) {
    let Some(path) = PATH.get() else {
        return;
    };

    let when = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    let message = message.replace(['\r', '\n'], " ");
    let line = format!("{when} {} {source}: {message}\n", level.label());
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .ok();
}

#[tauri::command]
pub fn log_failure(level: Level, source: String, message: String) {
    record(level, &source, &message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_lives_in_the_local_data_folder() {
        let dir = folder("io.example.app").unwrap();

        assert!(dir.ends_with("io.example.app/logs"), "{}", dir.display());
    }
}
