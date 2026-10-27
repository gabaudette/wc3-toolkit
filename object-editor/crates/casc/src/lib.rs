mod ffi;

use std::ffi::CString;
use std::path::Path;

use ffi::{FileHandle, StorageHandle};

// Warcraft III selects locales by path (`_locales\enus.w3mod:`), not by CASC locale.
const CASC_LOCALE_ALL: u32 = 0xFFFF_FFFF;
const CASC_OPEN_BY_NAME: u32 = 0;

const ERROR_FILE_NOT_FOUND: u32 = 2;

const READ_CHUNK_SIZE: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot open CASC storage at {path} (CascLib error {code})")]
    OpenStorage { path: String, code: u32 },
    #[error("file not found in storage: {0}")]
    NotFound(String),
    #[error("cannot read {name} (CascLib error {code})")]
    Read { name: String, code: u32 },
    #[error("path contains a NUL byte: {0}")]
    BadPath(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub size: u64,
}

pub struct Storage {
    handle: StorageHandle,
}

fn cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::BadPath(s.to_owned()))
}

impl Storage {
    pub fn open(game_dir: &Path) -> Result<Self> {
        let path = game_dir.to_string_lossy().into_owned();
        let c_path = cstring(&path)?;
        let handle = StorageHandle::open(&c_path, CASC_LOCALE_ALL)
            .map_err(|code| Error::OpenStorage { path, code })?;

        Ok(Self { handle })
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let c_name = cstring(&name.to_ascii_lowercase())?;
        let file = self
            .handle
            .open_file(&c_name, CASC_OPEN_BY_NAME)
            .map_err(|code| {
                if code == ERROR_FILE_NOT_FOUND {
                    return Error::NotFound(name.to_owned());
                }

                Error::Read {
                    name: name.to_owned(),
                    code,
                }
            })?;

        read_open_file(file, name)
    }

    pub fn read_optional(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self.read(name) {
            Ok(data) => Ok(Some(data)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn list(&self, mask: &str) -> Result<Vec<Entry>> {
        let c_mask = cstring(mask)?;

        Ok(self
            .handle
            .search(&c_mask)
            .map(|(name, size)| Entry { name, size })
            .collect())
    }
}

fn read_open_file(mut file: FileHandle<'_>, name: &str) -> Result<Vec<u8>> {
    let read_error = |code| Error::Read {
        name: name.to_owned(),
        code,
    };
    let size = file.size().map_err(read_error)?;
    let mut data = Vec::with_capacity(usize::try_from(size).unwrap_or_default());
    let mut chunk = vec![0u8; READ_CHUNK_SIZE];

    loop {
        let got = file.read(&mut chunk).map_err(read_error)?;

        if got == 0 {
            break;
        }

        data.extend_from_slice(&chunk[..got]);
    }

    Ok(data)
}
