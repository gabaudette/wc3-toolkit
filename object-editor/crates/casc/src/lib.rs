//! Read-only access to a local Blizzard CASC storage, such as the one in a
//! Warcraft III Reforged install. Wraps the bundled CascLib.

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;
use std::ptr;

type Handle = *mut c_void;

/// Accept files of every locale; Warcraft III selects locales by path instead.
const CASC_LOCALE_ALL: u32 = 0xFFFF_FFFF;
const CASC_OPEN_BY_NAME: u32 = 0;

/// CascLib reports a missing file as `ERROR_FILE_NOT_FOUND`, which is 2 on
/// Windows and maps to `ENOENT` (also 2) elsewhere.
const ERROR_FILE_NOT_FOUND: u32 = 2;

/// CascLib's `INVALID_HANDLE_VALUE`, i.e. `(HANDLE)-1`.
const INVALID_HANDLE_ADDR: usize = usize::MAX;

/// Matches CascLib's `MAX_PATH` on non-Windows platforms.
const NAME_CAPACITY: usize = 1024;
const READ_CHUNK_SIZE: u32 = 64 * 1024;

unsafe extern "C" {
    fn CascOpenStorage(params: *const c_char, locale_mask: u32, storage: *mut Handle) -> bool;
    fn CascCloseStorage(storage: Handle) -> bool;
    fn CascOpenFile(
        storage: Handle,
        name: *const c_void,
        locale: u32,
        flags: u32,
        file: *mut Handle,
    ) -> bool;
    fn CascGetFileSize64(file: Handle, size: *mut u64) -> bool;
    fn CascReadFile(file: Handle, buffer: *mut c_void, to_read: u32, read: *mut u32) -> bool;
    fn CascCloseFile(file: Handle) -> bool;
    fn CascFindClose(find: Handle) -> bool;
    fn GetCascError() -> u32;

    fn casc_shim_find_first(
        storage: Handle,
        mask: *const c_char,
        name: *mut c_char,
        capacity: usize,
        size: *mut u64,
    ) -> Handle;
    fn casc_shim_find_next(
        find: Handle,
        name: *mut c_char,
        capacity: usize,
        size: *mut u64,
    ) -> bool;
}

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

/// One entry from [`Storage::list`].
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub size: u64,
}

/// An open CASC storage. Not thread-safe; wrap it in a `Mutex` to share it.
pub struct Storage {
    handle: Handle,
}

// SAFETY: a CascLib storage handle is not tied to the thread that opened it.
// `Storage` is not `Sync`, so only one thread can use it at a time.
unsafe impl Send for Storage {}

fn last_error() -> u32 {
    // SAFETY: reads CascLib's thread-local last error; no arguments, no preconditions.
    unsafe { GetCascError() }
}

fn cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::BadPath(s.to_owned()))
}

impl Storage {
    /// Opens the storage of a game install, e.g. `.../Warcraft III`.
    pub fn open(game_dir: &Path) -> Result<Self> {
        let path = game_dir.to_string_lossy().into_owned();
        let c_path = cstring(&path)?;
        let mut handle = ptr::null_mut();

        // SAFETY: `c_path` is a valid NUL-terminated string that outlives the call,
        // and `handle` is a valid out-pointer.
        let opened = unsafe { CascOpenStorage(c_path.as_ptr(), CASC_LOCALE_ALL, &mut handle) };
        if !opened {
            return Err(Error::OpenStorage {
                path,
                code: last_error(),
            });
        }

        Ok(Self { handle })
    }

    /// Reads a whole file, e.g. `war3.w3mod:units\unitdata.slk`. Names are case-insensitive.
    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let c_name = cstring(&name.to_ascii_lowercase())?;
        let mut file = ptr::null_mut();

        // SAFETY: `self.handle` is an open storage, `c_name` is a valid NUL-terminated
        // string that outlives the call, and `file` is a valid out-pointer.
        let opened = unsafe {
            CascOpenFile(
                self.handle,
                c_name.as_ptr().cast(),
                0,
                CASC_OPEN_BY_NAME,
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
        unsafe { CascCloseFile(file) };
        result
    }

    /// Like [`Storage::read`], but returns `Ok(None)` when the file does not exist.
    pub fn read_optional(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self.read(name) {
            Ok(data) => Ok(Some(data)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Lists files matching a wildcard mask such as `*` or `war3.w3mod:units\*`.
    pub fn list(&self, mask: &str) -> Result<Vec<Entry>> {
        let c_mask = cstring(mask)?;
        let mut name: Vec<c_char> = vec![0; NAME_CAPACITY];
        let mut size = 0u64;
        let mut entries = Vec::new();

        // SAFETY: `self.handle` is an open storage, `c_mask` is NUL-terminated, and
        // `name` provides `NAME_CAPACITY` writable bytes; the shim always NUL-terminates.
        let find = unsafe {
            casc_shim_find_first(
                self.handle,
                c_mask.as_ptr(),
                name.as_mut_ptr(),
                NAME_CAPACITY,
                &mut size,
            )
        };
        if find.is_null() || find.addr() == INVALID_HANDLE_ADDR {
            return Ok(entries);
        }

        loop {
            // SAFETY: the shim wrote a NUL-terminated string into `name`.
            let entry_name = unsafe { CStr::from_ptr(name.as_ptr()) };
            entries.push(Entry {
                name: entry_name.to_string_lossy().into_owned(),
                size,
            });

            // SAFETY: `find` is a live search handle and `name` is writable as above.
            let found =
                unsafe { casc_shim_find_next(find, name.as_mut_ptr(), NAME_CAPACITY, &mut size) };
            if !found {
                break;
            }
        }

        // SAFETY: `find` is a live search handle, closed exactly once.
        unsafe { CascFindClose(find) };
        Ok(entries)
    }
}

fn read_open_file(file: Handle, name: &str) -> Result<Vec<u8>> {
    let read_error = || Error::Read {
        name: name.to_owned(),
        code: last_error(),
    };

    let mut size = 0u64;

    // SAFETY: `file` is an open CascLib file handle and `size` is a valid out-pointer.
    if !unsafe { CascGetFileSize64(file, &mut size) } {
        return Err(read_error());
    }

    let mut data = Vec::with_capacity(usize::try_from(size).unwrap_or_default());
    let mut chunk = vec![0u8; READ_CHUNK_SIZE as usize];

    loop {
        let mut got = 0u32;

        // SAFETY: `chunk` has `READ_CHUNK_SIZE` writable bytes and `got` is a valid out-pointer.
        let read =
            unsafe { CascReadFile(file, chunk.as_mut_ptr().cast(), READ_CHUNK_SIZE, &mut got) };
        if !read {
            return Err(read_error());
        }

        if got == 0 {
            break;
        }

        data.extend_from_slice(&chunk[..got as usize]);
    }

    Ok(data)
}

impl Drop for Storage {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is an open storage, closed exactly once here.
        unsafe { CascCloseStorage(self.handle) };
    }
}
