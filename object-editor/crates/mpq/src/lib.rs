mod ffi;

use std::ffi::CString;
use std::path::Path;

use ffi::{ArchiveHandle, NewFile};

const STREAM_FLAG_READ_ONLY: u32 = 0x0000_0100;
const MPQ_FILE_COMPRESS: u32 = 0x0000_0200;
const MPQ_FILE_REPLACE_EXISTING: u32 = 0x8000_0000;
const MPQ_COMPRESSION_ZLIB: u32 = 0x02;
const SEARCH_SCOPE_DEFAULT: u32 = 0;
const LOCALE_NEUTRAL: u32 = 0;
const NO_FILE_TIME: u64 = 0;
const PRIORITY_DEFAULT: u32 = 0;
// MPQ format version 1 with a `(listfile)`, the layout Warcraft III maps use.
const CREATE_FLAGS: u32 = 0x0010_0000;

const ERROR_FILE_NOT_FOUND: u32 = 2;

#[cfg(windows)]
const ERROR_DISK_FULL: u32 = 112;

#[cfg(not(windows))]
const ERROR_DISK_FULL: u32 = 28; // ENOSPC

const LISTFILE: &str = "(listfile)";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot open archive {path} (StormLib error {code})")]
    Open { path: String, code: u32 },
    #[error("file not found in archive: {0}")]
    NotFound(String),
    #[error("cannot read {name} (StormLib error {code})")]
    Read { name: String, code: u32 },
    #[error("cannot write {name} (StormLib error {code})")]
    Write { name: String, code: u32 },
    #[error("cannot save archive (StormLib error {code})")]
    Save { code: u32 },
    #[error("{name} is too large for an MPQ archive ({size} bytes)")]
    TooLarge { name: String, size: usize },
    #[error("path contains a NUL byte: {0}")]
    BadPath(String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::BadPath(s.to_owned()))
}

pub struct Archive {
    handle: ArchiveHandle,
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with(path, STREAM_FLAG_READ_ONLY)
    }

    pub fn open_writable(path: &Path) -> Result<Self> {
        Self::open_with(path, 0)
    }

    pub fn create(path: &Path, max_files: u32) -> Result<Self> {
        let display = path.to_string_lossy().into_owned();
        let c_path = cstring(&display)?;
        let handle = ArchiveHandle::create(&c_path, CREATE_FLAGS, max_files).map_err(|code| {
            Error::Open {
                path: display,
                code,
            }
        })?;

        Ok(Self { handle })
    }

    fn open_with(path: &Path, flags: u32) -> Result<Self> {
        let display = path.to_string_lossy().into_owned();
        let c_path = cstring(&display)?;
        let handle =
            ArchiveHandle::open(&c_path, PRIORITY_DEFAULT, flags).map_err(|code| Error::Open {
                path: display,
                code,
            })?;

        Ok(Self { handle })
    }

    pub fn contains(&self, name: &str) -> bool {
        let Ok(c_name) = cstring(name) else {
            return false;
        };

        self.handle.has_file(&c_name)
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let c_name = cstring(name)?;
        let read_error = |code| Error::Read {
            name: name.to_owned(),
            code,
        };
        let file = self
            .handle
            .open_file(&c_name, SEARCH_SCOPE_DEFAULT)
            .map_err(|code| {
                if code == ERROR_FILE_NOT_FOUND {
                    return Error::NotFound(name.to_owned());
                }

                read_error(code)
            })?;

        file.read_all().map_err(read_error)
    }

    pub fn read_optional(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self.read(name) {
            Ok(data) => Ok(Some(data)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn file_names(&self) -> Result<Vec<String>> {
        let Some(data) = self.read_optional(LISTFILE)? else {
            return Ok(Vec::new());
        };

        Ok(String::from_utf8_lossy(&data)
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect())
    }

    pub fn write(&mut self, name: &str, data: &[u8]) -> Result<()> {
        match self.try_write(name, data) {
            Err(Error::Write { code, .. }) if code == ERROR_DISK_FULL => {
                self.grow_file_table(name)?;
                self.try_write(name, data)
            }
            other => other,
        }
    }

    fn grow_file_table(&mut self, name: &str) -> Result<()> {
        let wanted = self.handle.max_file_count().saturating_mul(2).max(1);

        self.handle
            .set_max_file_count(wanted)
            .map_err(|code| Error::Write {
                name: name.to_owned(),
                code,
            })
    }

    fn try_write(&mut self, name: &str, data: &[u8]) -> Result<()> {
        if u32::try_from(data.len()).is_err() {
            return Err(Error::TooLarge {
                name: name.to_owned(),
                size: data.len(),
            });
        }

        let c_name = cstring(name)?;
        let file = NewFile {
            name: &c_name,
            file_time: NO_FILE_TIME,
            locale: LOCALE_NEUTRAL,
            flags: MPQ_FILE_COMPRESS | MPQ_FILE_REPLACE_EXISTING,
            compression: MPQ_COMPRESSION_ZLIB,
        };

        self.handle
            .add_file(&file, data)
            .map_err(|code| Error::Write {
                name: name.to_owned(),
                code,
            })
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let c_name = cstring(name)?;

        match self.handle.remove_file(&c_name, SEARCH_SCOPE_DEFAULT) {
            Err(code) if code == ERROR_FILE_NOT_FOUND => Ok(()),
            Err(code) => Err(Error::Write {
                name: name.to_owned(),
                code,
            }),
            Ok(()) => Ok(()),
        }
    }

    pub fn save(self) -> Result<()> {
        self.handle.save().map_err(|code| Error::Save { code })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_TABLE: u32 = 4;

    #[test]
    fn writes_and_reads_back_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.mpq");

        let mut archive = Archive::create(&path, SMALL_TABLE).unwrap();

        archive
            .write("war3map.wts", b"STRING 1\n{\nhello\n}\n")
            .unwrap();
        archive.save().unwrap();

        let mut archive = Archive::open_writable(&path).unwrap();

        archive.write("war3map.wts", b"replaced").unwrap();
        archive.save().unwrap();

        let archive = Archive::open(&path).unwrap();

        assert_eq!(archive.read("war3map.wts").unwrap(), b"replaced");
        assert!(archive.contains("war3map.wts"));
        assert!(matches!(
            archive.read("missing.txt"),
            Err(Error::NotFound(_))
        ));
        assert!(
            archive
                .file_names()
                .unwrap()
                .contains(&"war3map.wts".to_owned())
        );
    }

    #[test]
    fn removes_files_and_ignores_missing_ones() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("remove.mpq");
        let mut archive = Archive::create(&path, SMALL_TABLE).unwrap();

        archive.write("a.txt", b"a").unwrap();
        archive.write("b.txt", b"b").unwrap();
        archive.save().unwrap();

        let mut archive = Archive::open_writable(&path).unwrap();

        archive.remove("a.txt").unwrap();
        archive.remove("never-there.txt").unwrap();
        archive.save().unwrap();

        let archive = Archive::open(&path).unwrap();

        assert!(!archive.contains("a.txt"));
        assert_eq!(archive.read("b.txt").unwrap(), b"b");
    }

    #[test]
    fn grows_a_full_file_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("full.mpq");
        let mut archive = Archive::create(&path, SMALL_TABLE).unwrap();

        for i in 0..SMALL_TABLE * 4 {
            archive
                .write(&format!("file{i}.txt"), i.to_string().as_bytes())
                .unwrap();
        }

        archive.save().unwrap();

        let archive = Archive::open(&path).unwrap();

        assert_eq!(archive.read("file15.txt").unwrap(), b"15");
    }
}
