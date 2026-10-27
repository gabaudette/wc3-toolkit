#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};
use std::marker::PhantomData;
use std::ptr;

type Handle = *mut c_void;

const INVALID_HANDLE_ADDR: usize = usize::MAX;
const NAME_CAPACITY: usize = 1024;

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

pub fn last_error() -> u32 {
    // SAFETY: reads CascLib's thread-local last error; no arguments, no preconditions.
    unsafe { GetCascError() }
}

pub struct StorageHandle(Handle);

// SAFETY: a CascLib storage handle is not tied to the thread that opened it.
unsafe impl Send for StorageHandle {}
// SAFETY: CascLib guards the state reads share: opening data archives takes the
// storage lock (`CascReadFile.cpp`) and every file stream has its own lock
// (`FileStream.cpp`). Each read opens and closes its own file handle, each
// search its own find handle, and the last error is thread-local. CascLib's own
// test extracts files from one storage on several worker threads
// (`RunExtractWorkers` in `CascTest.cpp`). Closing the storage needs ownership
// (`Drop`), and files and searches borrow it, so closing never races a read.
unsafe impl Sync for StorageHandle {}

impl StorageHandle {
    pub fn open(path: &CStr, locale_mask: u32) -> Result<Self, u32> {
        let mut handle = ptr::null_mut();

        // SAFETY: `path` is NUL-terminated and outlives the call; `handle` is a valid out-pointer.
        let opened = unsafe { CascOpenStorage(path.as_ptr(), locale_mask, &mut handle) };

        if !opened {
            return Err(last_error());
        }

        Ok(Self(handle))
    }

    pub fn open_file(&self, name: &CStr, flags: u32) -> Result<FileHandle<'_>, u32> {
        let mut file = ptr::null_mut();

        // SAFETY: `self.0` is an open storage, `name` is NUL-terminated and outlives
        // the call, and `file` is a valid out-pointer.
        let opened = unsafe { CascOpenFile(self.0, name.as_ptr().cast(), 0, flags, &mut file) };

        if !opened {
            return Err(last_error());
        }

        Ok(FileHandle {
            handle: file,
            storage: PhantomData,
        })
    }

    pub fn search(&self, mask: &CStr) -> Search<'_> {
        let mut name: Vec<c_char> = vec![0; NAME_CAPACITY];
        let mut size = 0u64;

        // SAFETY: `self.0` is an open storage, `mask` is NUL-terminated, and `name`
        // provides `NAME_CAPACITY` writable bytes; the shim always NUL-terminates.
        let find = unsafe {
            casc_shim_find_first(
                self.0,
                mask.as_ptr(),
                name.as_mut_ptr(),
                NAME_CAPACITY,
                &mut size,
            )
        };
        let found = !find.is_null() && find.addr() != INVALID_HANDLE_ADDR;

        Search {
            handle: find,
            name,
            size,
            current: found,
            storage: PhantomData,
        }
    }
}

impl Drop for StorageHandle {
    fn drop(&mut self) {
        // SAFETY: `self.0` is an open storage, closed exactly once here.
        unsafe { CascCloseStorage(self.0) };
    }
}

pub struct FileHandle<'storage> {
    handle: Handle,
    storage: PhantomData<&'storage StorageHandle>,
}

impl FileHandle<'_> {
    pub fn size(&self) -> Result<u64, u32> {
        let mut size = 0u64;

        // SAFETY: `self.handle` is an open file and `size` is a valid out-pointer.
        if !unsafe { CascGetFileSize64(self.handle, &mut size) } {
            return Err(last_error());
        }

        Ok(size)
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, u32> {
        let capacity = u32::try_from(buffer.len()).unwrap_or(u32::MAX);
        let mut got = 0u32;

        // SAFETY: `buffer` has at least `capacity` writable bytes and `got` is a valid out-pointer.
        let read =
            unsafe { CascReadFile(self.handle, buffer.as_mut_ptr().cast(), capacity, &mut got) };

        if !read {
            return Err(last_error());
        }

        Ok(got as usize)
    }
}

impl Drop for FileHandle<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is an open file, closed exactly once here.
        unsafe { CascCloseFile(self.handle) };
    }
}

pub struct Search<'storage> {
    handle: Handle,
    name: Vec<c_char>,
    size: u64,
    current: bool,
    storage: PhantomData<&'storage StorageHandle>,
}

impl Iterator for Search<'_> {
    type Item = (String, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.current {
            return None;
        }

        // SAFETY: the shim wrote a NUL-terminated string into `self.name`.
        let name = unsafe { CStr::from_ptr(self.name.as_ptr()) };
        let entry = (name.to_string_lossy().into_owned(), self.size);

        // SAFETY: `self.handle` is a live search handle and `self.name` has
        // `NAME_CAPACITY` writable bytes, NUL-terminated by the shim.
        self.current = unsafe {
            casc_shim_find_next(
                self.handle,
                self.name.as_mut_ptr(),
                NAME_CAPACITY,
                &mut self.size,
            )
        };

        Some(entry)
    }
}

impl Drop for Search<'_> {
    fn drop(&mut self) {
        if self.handle.is_null() || self.handle.addr() == INVALID_HANDLE_ADDR {
            return;
        }

        // SAFETY: `self.handle` is a live search handle, closed exactly once here.
        unsafe { CascFindClose(self.handle) };
    }
}
