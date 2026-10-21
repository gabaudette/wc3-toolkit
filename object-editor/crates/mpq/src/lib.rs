//! Read and write MPQ archives, the container format of Warcraft III maps.
//! Wraps the bundled StormLib.

use std::ffi::{CString, c_char, c_void};
use std::path::Path;
use std::ptr;

type Handle = *mut c_void;

const STREAM_FLAG_READ_ONLY: u32 = 0x0000_0100;
const MPQ_FILE_COMPRESS: u32 = 0x0000_0200;
const MPQ_FILE_REPLACE_EXISTING: u32 = 0x8000_0000;
const MPQ_COMPRESSION_ZLIB: u32 = 0x02;
const SFILE_INVALID_SIZE: u32 = 0xFFFF_FFFF;
const SEARCH_SCOPE_DEFAULT: u32 = 0;
const LOCALE_NEUTRAL: u32 = 0;
const NO_FILE_TIME: u64 = 0;
const PRIORITY_DEFAULT: u32 = 0;
/// MPQ format version 1 with a `(listfile)`, the layout Warcraft III maps use.
const CREATE_FLAGS: u32 = 0x0010_0000;

/// StormLib reports errors as Windows error codes, mapped to errno values elsewhere.
const ERROR_FILE_NOT_FOUND: u32 = 2;
#[cfg(windows)]
const ERROR_DISK_FULL: u32 = 112;
#[cfg(not(windows))]
const ERROR_DISK_FULL: u32 = 28; // ENOSPC

/// The file list StormLib maintains inside an archive.
const LISTFILE: &str = "(listfile)";

unsafe extern "C" {
    fn SFileOpenArchive(name: *const c_char, priority: u32, flags: u32, mpq: *mut Handle) -> bool;
    fn SFileCreateArchive(
        name: *const c_char,
        flags: u32,
        max_files: u32,
        mpq: *mut Handle,
    ) -> bool;
    fn SFileCloseArchive(mpq: Handle) -> bool;
    fn SFileFlushArchive(mpq: Handle) -> bool;
    fn SFileGetMaxFileCount(mpq: Handle) -> u32;
    fn SFileSetMaxFileCount(mpq: Handle, count: u32) -> bool;
    fn SFileHasFile(mpq: Handle, name: *const c_char) -> bool;
    fn SFileOpenFileEx(mpq: Handle, name: *const c_char, scope: u32, file: *mut Handle) -> bool;
    fn SFileGetFileSize(file: Handle, size_high: *mut u32) -> u32;
    fn SFileReadFile(
        file: Handle,
        buffer: *mut c_void,
        to_read: u32,
        read: *mut u32,
        overlapped: *mut c_void,
    ) -> bool;
    fn SFileCloseFile(file: Handle) -> bool;
    fn SFileCreateFile(
        mpq: Handle,
        name: *const c_char,
        file_time: u64,
        size: u32,
        locale: u32,
        flags: u32,
        file: *mut Handle,
    ) -> bool;
    fn SFileWriteFile(file: Handle, data: *const c_void, size: u32, compression: u32) -> bool;
    fn SFileFinishFile(file: Handle) -> bool;
    fn SErrGetLastError() -> u32;
}

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

fn last_error() -> u32 {
    // SAFETY: reads StormLib's thread-local last error; no arguments, no preconditions.
    unsafe { SErrGetLastError() }
}

fn cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::BadPath(s.to_owned()))
}

/// An open MPQ archive.
///
/// Archives opened with [`Archive::open_writable`] are modified in place; the
/// changes are saved by [`Archive::save`] or when the archive is dropped. To
/// keep the original safe, open a copy.
pub struct Archive {
    handle: Handle,
}

// SAFETY: a StormLib archive handle is not tied to the thread that opened it.
// `Archive` is not `Sync`, so only one thread can use it at a time.
unsafe impl Send for Archive {}

impl Archive {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with(path, STREAM_FLAG_READ_ONLY)
    }

    pub fn open_writable(path: &Path) -> Result<Self> {
        Self::open_with(path, 0)
    }

    /// Creates a new, empty archive with room for `max_files` files (it grows when full).
    pub fn create(path: &Path, max_files: u32) -> Result<Self> {
        let display = path.to_string_lossy().into_owned();
        let c_path = cstring(&display)?;
        let mut handle = ptr::null_mut();

        // SAFETY: `c_path` is NUL-terminated and outlives the call; `handle` is a valid out-pointer.
        let created =
            unsafe { SFileCreateArchive(c_path.as_ptr(), CREATE_FLAGS, max_files, &mut handle) };
        if !created {
            return Err(Error::Open {
                path: display,
                code: last_error(),
            });
        }

        Ok(Self { handle })
    }

    fn open_with(path: &Path, flags: u32) -> Result<Self> {
        let display = path.to_string_lossy().into_owned();
        let c_path = cstring(&display)?;
        let mut handle = ptr::null_mut();

        // SAFETY: `c_path` is NUL-terminated and outlives the call; `handle` is a valid out-pointer.
        let opened =
            unsafe { SFileOpenArchive(c_path.as_ptr(), PRIORITY_DEFAULT, flags, &mut handle) };
        if !opened {
            return Err(Error::Open {
                path: display,
                code: last_error(),
            });
        }

        Ok(Self { handle })
    }

    pub fn contains(&self, name: &str) -> bool {
        let Ok(c_name) = cstring(name) else {
            return false;
        };

        // SAFETY: `self.handle` is an open archive and `c_name` is NUL-terminated.
        unsafe { SFileHasFile(self.handle, c_name.as_ptr()) }
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let c_name = cstring(name)?;
        let mut file = ptr::null_mut();

        // SAFETY: `self.handle` is an open archive, `c_name` is NUL-terminated and
        // outlives the call, and `file` is a valid out-pointer.
        let opened = unsafe {
            SFileOpenFileEx(
                self.handle,
                c_name.as_ptr(),
                SEARCH_SCOPE_DEFAULT,
                &mut file,
            )
        };
        if !opened {
            let code = last_error();
            if code == ERROR_FILE_NOT_FOUND {
                return Err(Error::NotFound(name.to_owned()));
            }

            return Err(Error::Read {
                name: name.to_owned(),
                code,
            });
        }

        let result = read_open_file(file, name);

        // SAFETY: `file` was opened above and is closed exactly once.
        unsafe { SFileCloseFile(file) };
        result
    }

    /// Like [`Archive::read`], but returns `Ok(None)` when the file does not exist.
    pub fn read_optional(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self.read(name) {
            Ok(data) => Ok(Some(data)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// File names from the archive's `(listfile)`. Archives without one return an empty list.
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

    /// Adds or replaces a file, compressed with zlib. Grows the archive's file
    /// table when it is full.
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
        // SAFETY: `self.handle` is an open, writable archive.
        let current = unsafe { SFileGetMaxFileCount(self.handle) };
        let wanted = current.saturating_mul(2).max(1);

        // SAFETY: as above; StormLib rebuilds its tables for the new size.
        if !unsafe { SFileSetMaxFileCount(self.handle, wanted) } {
            return Err(Error::Write {
                name: name.to_owned(),
                code: last_error(),
            });
        }

        Ok(())
    }

    fn try_write(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let write_error = || Error::Write {
            name: name.to_owned(),
            code: last_error(),
        };
        let size = u32::try_from(data.len()).map_err(|_| Error::TooLarge {
            name: name.to_owned(),
            size: data.len(),
        })?;
        let c_name = cstring(name)?;
        let mut file = ptr::null_mut();

        // SAFETY: `self.handle` is an open archive, `c_name` is NUL-terminated and
        // outlives the call, and `file` is a valid out-pointer.
        let created = unsafe {
            SFileCreateFile(
                self.handle,
                c_name.as_ptr(),
                NO_FILE_TIME,
                size,
                LOCALE_NEUTRAL,
                MPQ_FILE_COMPRESS | MPQ_FILE_REPLACE_EXISTING,
                &mut file,
            )
        };
        if !created {
            return Err(write_error());
        }

        // SAFETY: `file` is the handle created above and `data` has exactly `size` bytes.
        let written =
            unsafe { SFileWriteFile(file, data.as_ptr().cast(), size, MPQ_COMPRESSION_ZLIB) };

        // SAFETY: `file` is finished (and thereby closed) exactly once, whether or not writing succeeded.
        let finished = unsafe { SFileFinishFile(file) };
        if !written || !finished {
            return Err(write_error());
        }

        Ok(())
    }

    /// Writes all changes to disk and closes the archive.
    pub fn save(self) -> Result<()> {
        let handle = self.handle;
        std::mem::forget(self);

        // SAFETY: `handle` is an open archive; ownership moved out of `self`, so
        // `Drop` will not close it a second time.
        let flushed = unsafe { SFileFlushArchive(handle) };
        let flush_code = last_error();

        // SAFETY: as above; the handle is closed exactly once.
        let closed = unsafe { SFileCloseArchive(handle) };
        if !flushed {
            return Err(Error::Save { code: flush_code });
        }

        if !closed {
            return Err(Error::Save { code: last_error() });
        }

        Ok(())
    }
}

fn read_open_file(file: Handle, name: &str) -> Result<Vec<u8>> {
    let read_error = || Error::Read {
        name: name.to_owned(),
        code: last_error(),
    };

    // SAFETY: `file` is an open StormLib file; a null high-size pointer is allowed.
    let size = unsafe { SFileGetFileSize(file, ptr::null_mut()) };
    if size == SFILE_INVALID_SIZE {
        return Err(read_error());
    }

    let mut data = vec![0u8; size as usize];
    let mut read = 0u32;

    // SAFETY: `data` has `size` writable bytes, `read` is a valid out-pointer and
    // StormLib allows a null `overlapped`.
    let ok = unsafe {
        SFileReadFile(
            file,
            data.as_mut_ptr().cast(),
            size,
            &mut read,
            ptr::null_mut(),
        )
    };
    if !ok || read != size {
        return Err(read_error());
    }

    Ok(data)
}

impl Drop for Archive {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is an open archive, closed exactly once here.
        // Closing also flushes pending writes; errors cannot be reported from Drop,
        // which is why `save` exists.
        unsafe { SFileCloseArchive(self.handle) };
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
