//! StormLib's C functions behind owned handles: the only unsafe code in this crate.
#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr;

type Handle = *mut c_void;

const SFILE_INVALID_SIZE: u32 = 0xFFFF_FFFF;

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

pub fn last_error() -> u32 {
    // SAFETY: reads StormLib's thread-local last error; no arguments, no preconditions.
    unsafe { SErrGetLastError() }
}

pub struct ArchiveHandle(Handle);

// SAFETY: a StormLib archive handle is not tied to the thread that opened it.
// `ArchiveHandle` is not `Sync`, so only one thread can use it at a time.
unsafe impl Send for ArchiveHandle {}

pub struct NewFile<'name> {
    pub name: &'name CStr,
    pub file_time: u64,
    pub locale: u32,
    pub flags: u32,
    pub compression: u32,
}

impl ArchiveHandle {
    pub fn open(path: &CStr, priority: u32, flags: u32) -> Result<Self, u32> {
        let mut handle = ptr::null_mut();

        // SAFETY: `path` is NUL-terminated and outlives the call; `handle` is a valid out-pointer.
        if !unsafe { SFileOpenArchive(path.as_ptr(), priority, flags, &mut handle) } {
            return Err(last_error());
        }

        Ok(Self(handle))
    }

    pub fn create(path: &CStr, flags: u32, max_files: u32) -> Result<Self, u32> {
        let mut handle = ptr::null_mut();

        // SAFETY: `path` is NUL-terminated and outlives the call; `handle` is a valid out-pointer.
        if !unsafe { SFileCreateArchive(path.as_ptr(), flags, max_files, &mut handle) } {
            return Err(last_error());
        }

        Ok(Self(handle))
    }

    pub fn has_file(&self, name: &CStr) -> bool {
        // SAFETY: `self.0` is an open archive and `name` is NUL-terminated.
        unsafe { SFileHasFile(self.0, name.as_ptr()) }
    }

    pub fn open_file(&self, name: &CStr, scope: u32) -> Result<FileHandle<'_>, u32> {
        let mut file = ptr::null_mut();

        // SAFETY: `self.0` is an open archive, `name` is NUL-terminated and outlives
        // the call, and `file` is a valid out-pointer.
        if !unsafe { SFileOpenFileEx(self.0, name.as_ptr(), scope, &mut file) } {
            return Err(last_error());
        }

        Ok(FileHandle {
            handle: file,
            archive: PhantomData,
        })
    }

    pub fn max_file_count(&self) -> u32 {
        // SAFETY: `self.0` is an open archive.
        unsafe { SFileGetMaxFileCount(self.0) }
    }

    pub fn set_max_file_count(&mut self, count: u32) -> Result<(), u32> {
        // SAFETY: `self.0` is an open, writable archive; StormLib rebuilds its tables.
        if !unsafe { SFileSetMaxFileCount(self.0, count) } {
            return Err(last_error());
        }

        Ok(())
    }

    pub fn add_file(&mut self, file: &NewFile<'_>, data: &[u8]) -> Result<(), u32> {
        let size = u32::try_from(data.len()).unwrap_or(u32::MAX);
        let mut handle = ptr::null_mut();

        // SAFETY: `self.0` is an open archive, `file.name` is NUL-terminated and
        // outlives the call, and `handle` is a valid out-pointer.
        let created = unsafe {
            SFileCreateFile(
                self.0,
                file.name.as_ptr(),
                file.file_time,
                size,
                file.locale,
                file.flags,
                &mut handle,
            )
        };

        if !created {
            return Err(last_error());
        }

        // SAFETY: `handle` is the file created above and `data` has at least `size` bytes.
        let written =
            unsafe { SFileWriteFile(handle, data.as_ptr().cast(), size, file.compression) };
        let write_code = last_error();

        // SAFETY: `handle` is finished (and thereby closed) exactly once, whether or not writing succeeded.
        let finished = unsafe { SFileFinishFile(handle) };

        if !written {
            return Err(write_code);
        }

        if !finished {
            return Err(last_error());
        }

        Ok(())
    }

    pub fn save(self) -> Result<(), u32> {
        let archive = ManuallyDrop::new(self);

        // SAFETY: `archive.0` is an open archive; `ManuallyDrop` keeps `Drop` from closing it again.
        let flushed = unsafe { SFileFlushArchive(archive.0) };
        let flush_code = last_error();

        // SAFETY: as above; the handle is closed exactly once.
        let closed = unsafe { SFileCloseArchive(archive.0) };

        if !flushed {
            return Err(flush_code);
        }

        if !closed {
            return Err(last_error());
        }

        Ok(())
    }
}

impl Drop for ArchiveHandle {
    fn drop(&mut self) {
        // SAFETY: `self.0` is an open archive, closed exactly once here. Closing also
        // flushes pending writes; errors cannot be reported from Drop, which is why
        // `save` exists.
        unsafe { SFileCloseArchive(self.0) };
    }
}

pub struct FileHandle<'archive> {
    handle: Handle,
    archive: PhantomData<&'archive ArchiveHandle>,
}

impl FileHandle<'_> {
    pub fn read_all(&self) -> Result<Vec<u8>, u32> {
        // SAFETY: `self.handle` is an open file; a null high-size pointer is allowed.
        let size = unsafe { SFileGetFileSize(self.handle, ptr::null_mut()) };

        if size == SFILE_INVALID_SIZE {
            return Err(last_error());
        }

        let mut data = vec![0u8; size as usize];
        let mut read = 0u32;

        // SAFETY: `data` has `size` writable bytes, `read` is a valid out-pointer and
        // StormLib allows a null `overlapped`.
        let ok = unsafe {
            SFileReadFile(
                self.handle,
                data.as_mut_ptr().cast(),
                size,
                &mut read,
                ptr::null_mut(),
            )
        };

        if !ok || read != size {
            return Err(last_error());
        }

        Ok(data)
    }
}

impl Drop for FileHandle<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is an open file, closed exactly once here.
        unsafe { SFileCloseFile(self.handle) };
    }
}
