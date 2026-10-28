use std::path::{Path, PathBuf};

pub use wc3_data::objects::{FourCC, Layout, ObjectData, Value};
pub use wc3_data::strings::MapStrings;

pub mod abilities;
mod binary;
pub mod imports;
pub mod info;
pub mod leveled;
pub mod lua;
pub mod placements;
pub mod plain;
pub mod references;
pub mod transfer;
pub mod units;
mod values;

pub const STRINGS_FILE: &str = "war3map.wts";
pub const UNITS_FILE: &str = "war3map.w3u";
pub const UNIT_SKINS_FILE: &str = "war3mapSkin.w3u";
pub const ABILITIES_FILE: &str = "war3map.w3a";
pub const ABILITY_SKINS_FILE: &str = "war3mapSkin.w3a";
pub const BUFFS_FILE: &str = "war3map.w3h";
pub const BUFF_SKINS_FILE: &str = "war3mapSkin.w3h";
pub const ITEMS_FILE: &str = "war3map.w3t";
pub const ITEM_SKINS_FILE: &str = "war3mapSkin.w3t";
pub const DESTRUCTABLES_FILE: &str = "war3map.w3b";
pub const DESTRUCTABLE_SKINS_FILE: &str = "war3mapSkin.w3b";
pub const UPGRADES_FILE: &str = "war3map.w3q";
pub const UPGRADE_SKINS_FILE: &str = "war3mapSkin.w3q";
pub const DOODADS_FILE: &str = "war3map.w3d";
pub const DOODAD_SKINS_FILE: &str = "war3mapSkin.w3d";

/// Object data files. Reforged splits each kind into
/// gameplay data and a `Skin` file with names and art.
pub const OBJECT_FILES: &[&str] = &[
    "war3map.w3u",
    "war3map.w3t",
    "war3map.w3b",
    "war3map.w3d",
    "war3map.w3a",
    "war3map.w3h",
    "war3map.w3q",
    "war3mapSkin.w3u",
    "war3mapSkin.w3t",
    "war3mapSkin.w3b",
    "war3mapSkin.w3d",
    "war3mapSkin.w3a",
    "war3mapSkin.w3h",
    "war3mapSkin.w3q",
];

const TEMP_SUFFIX: &str = ".wc3-toolkit-tmp";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Mpq(#[from] mpq::Error),
    #[error(transparent)]
    Data(#[from] wc3_data::Error),
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{0} is not an object data file")]
    NotObjectFile(String),
    #[error("{0}")]
    Invalid(String),
    #[error("{file}:{line}:{column}: Lua syntax error: {message}")]
    Lua {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> Error + '_ {
    move |source| Error::Io {
        path: path.to_owned(),
        source,
    }
}

pub struct Map {
    path: PathBuf,
    archive: mpq::Archive,
}

impl Map {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            path: path.to_owned(),
            archive: mpq::Archive::open(path)?,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_names(&self) -> Result<Vec<String>> {
        Ok(self.archive.file_names()?)
    }

    pub fn read_file(&self, name: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.archive.read_optional(name)?)
    }

    pub fn object_data(&self, name: &str) -> Result<Option<ObjectData>> {
        let layout = Layout::for_file(name).ok_or_else(|| Error::NotObjectFile(name.to_owned()))?;
        let Some(bytes) = self.read_file(name)? else {
            return Ok(None);
        };

        let data = ObjectData::parse(&bytes, layout).map_err(|e| e.in_file(name))?;

        Ok(Some(data))
    }

    pub fn strings(&self) -> Result<MapStrings> {
        let Some(bytes) = self.read_file(STRINGS_FILE)? else {
            return Ok(MapStrings::default());
        };

        let text = String::from_utf8(bytes).map_err(|e| {
            Error::Invalid(format!(
                "{STRINGS_FILE} is not UTF-8 at byte {}",
                e.utf8_error().valid_up_to()
            ))
        })?;

        Ok(MapStrings::parse(&text).map_err(|e| e.in_file(STRINGS_FILE))?)
    }
}

pub fn save_files(source: &Path, dest: &Path, files: &[(&str, Vec<u8>)]) -> Result<()> {
    save_changes(source, dest, files, &[])
}

pub fn save_changes(
    source: &Path,
    dest: &Path,
    files: &[(&str, Vec<u8>)],
    removed: &[&str],
) -> Result<()> {
    let mut temp_name = dest.file_name().unwrap_or_default().to_os_string();

    temp_name.push(TEMP_SUFFIX);
    let temp = dest.with_file_name(temp_name);

    std::fs::copy(source, &temp).map_err(io_error(&temp))?;
    let result = write_into(&temp, files, removed)
        .and_then(|()| std::fs::rename(&temp, dest).map_err(io_error(dest)));

    if result.is_err() {
        // Best effort: the temporary copy is useless after a failure. The original
        // error is what matters to the caller, so a cleanup failure is not reported.
        let _ = std::fs::remove_file(&temp);
    }

    result
}

fn write_into(path: &Path, files: &[(&str, Vec<u8>)], removed: &[&str]) -> Result<()> {
    let mut archive = mpq::Archive::open_writable(path)?;

    for name in removed {
        archive.remove(name)?;
    }

    for (name, data) in files {
        archive.write(name, data)?;
    }

    Ok(archive.save()?)
}
