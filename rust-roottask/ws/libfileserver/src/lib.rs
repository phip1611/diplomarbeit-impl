//! File server lib.

#![cfg_attr(not(test), no_std)]
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(asm)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(const_btree_new)]
#![feature(slice_ptr_get)]

#[allow(unused)]
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate std;

#[allow(unused)]
#[macro_use]
extern crate alloc;

#[allow(unused)]
#[macro_use]
extern crate libhrstd;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::min;
use libhrstd::process::consts::ProcessId;
use libhrstd::rt::services::fs::fd::FD;
use libhrstd::rt::services::fs::fs_open::FsOpenFlags;
use libhrstd::sync::mutex::SimpleMutex;

static OPEN_FILE_TABLE: SimpleMutex<OpenFileTable> = SimpleMutex::new(OpenFileTable::new());

static IN_MEM_FS: SimpleMutex<InMemFilesystem> = SimpleMutex::new(InMemFilesystem::new());

/// Holds information about all files that are currently open.
#[derive(Debug)]
struct OpenFileTable(BTreeMap<OpenFileHandleId, OpenFileHandle>);

impl OpenFileTable {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn open(
        &mut self,
        caller: ProcessId,
        path: String,
        flags: FsOpenFlags,
        umode: u16,
    ) -> Result<FD, ()> {
        let fd = self.next_fd(caller);

        let mut in_mem_fs = IN_MEM_FS.lock();
        if in_mem_fs.file(&path).is_none() & flags.can_create() {
            // create new file
            in_mem_fs
                .create(
                    path.clone(),
                    InMemFile::new(path.clone(), FileMetaData::new(umode, caller)),
                )
                .unwrap();
            self.data_mut()
                .insert((caller, fd), OpenFileHandle::new(flags, path));
            Ok(fd)
        } else if in_mem_fs.file(&path).is_none() {
            Err(())
        } else {
            // open existing file
            self.data_mut()
                .insert((caller, fd), OpenFileHandle::new(flags, path));
            Ok(fd)
        }
    }

    pub fn close(&mut self, caller: ProcessId, fd: FD) -> Result<(), ()> {
        let key = (caller, fd);
        self.data_mut().remove(&key).map(|_| ()).ok_or(())
    }

    fn data(&self) -> &BTreeMap<OpenFileHandleId, OpenFileHandle> {
        &self.0
    }

    fn data_mut(&mut self) -> &mut BTreeMap<OpenFileHandleId, OpenFileHandle> {
        &mut self.0
    }

    fn next_fd(&self, pid: ProcessId) -> FD {
        let mut i = 3;
        // all fds that the PID is using
        let fds_in_use = self
            .data()
            .keys()
            .filter(|(process_id, _)| *process_id == pid)
            .map(|(_pid, fd)| fd.raw())
            .collect::<Vec<_>>();
        loop {
            if fds_in_use.contains(&i) {
                i += 1;
            } else {
                return FD::new(i);
            }
        }
    }
}

type OpenFileHandleId = (ProcessId, FD);
#[derive(Debug)]
struct OpenFileHandle {
    file_offset: usize,
    flags: FsOpenFlags,
    // used as ID
    file_path: String,
}

impl OpenFileHandle {
    pub fn new(flags: FsOpenFlags, file_id: String) -> Self {
        OpenFileHandle {
            file_offset: 0,
            flags,
            file_path: file_id,
        }
    }

    pub fn file_offset(&self) -> usize {
        self.file_offset
    }
    pub fn flags(&self) -> FsOpenFlags {
        self.flags
    }
    pub fn file_path(&self) -> &String {
        &self.file_path
    }
}

#[derive(Debug)]
struct FileMetaData {
    umode: u16,
    owner: ProcessId,
}

impl FileMetaData {
    pub fn new(umode: u16, owner: ProcessId) -> Self {
        FileMetaData { umode, owner }
    }

    pub fn umode(&self) -> u16 {
        self.umode
    }
    pub fn owner(&self) -> ProcessId {
        self.owner
    }
}

/// An in-memory file.
#[derive(Debug)]
struct InMemFile {
    // used as ID
    path: String,
    data: Vec<u8>,
    meta: FileMetaData,
}

impl InMemFile {
    pub fn new(path: String, meta: FileMetaData) -> Self {
        Self {
            path,
            data: vec![],
            meta,
        }
    }
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    pub fn path(&self) -> &String {
        &self.path
    }
    pub fn meta(&self) -> &FileMetaData {
        &self.meta
    }
}

#[derive(Debug)]
struct InMemFilesystem {
    files: BTreeMap<String, InMemFile>,
}

impl InMemFilesystem {
    const fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    fn create(&mut self, filepath: String, file: InMemFile) -> Result<(), ()> {
        if self.files.contains_key(&filepath) {
            Err(())
        } else {
            self.files.insert(filepath, file);
            Ok(())
        }
    }

    fn file(&self, filepath: &String) -> Option<&InMemFile> {
        self.files.get(filepath)
    }

    fn file_mut(&mut self, filepath: &String) -> Option<&mut InMemFile> {
        self.files.get_mut(filepath)
    }

    fn delete(&mut self, filepath: &String) -> bool {
        self.files.remove(filepath).is_some()
    }
}

pub fn fs_open(caller: ProcessId, path: String, flags: FsOpenFlags, umode: u16) -> FD {
    if flags.is_empty() {
        return FD::error();
    };
    if path.is_empty() {
        return FD::error();
    }
    OPEN_FILE_TABLE
        .lock()
        .open(caller, path, flags, umode)
        .unwrap_or(FD::error())
}

pub fn fs_read(caller: ProcessId, fd: FD, count: usize) -> Result<Vec<u8>, ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let in_mem_fs_lock = IN_MEM_FS.lock();
    let offset = handle.file_offset();
    let file = in_mem_fs_lock.file(handle.file_path()).ok_or(())?;

    let mut data = Vec::new();
    let to = min(file.data().len(), count + offset);
    let bytes_read = to - 1 - offset;
    handle.file_offset += bytes_read;

    data.extend_from_slice(&file.data()[offset..to]);
    Ok(data)
}

pub fn fs_write(caller: ProcessId, fd: FD, new_data: &[u8]) -> Result<(), ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let mut in_mem_fs_lock = IN_MEM_FS.lock();

    let file = in_mem_fs_lock.file_mut(handle.file_path()).ok_or(())?;

    let offset = if handle.flags().is_append() {
        file.data.len() - 1
    } else {
        handle.file_offset()
    };

    handle.file_offset = offset + new_data.len();

    // currently I only support append..
    file.data_mut().extend_from_slice(new_data);

    Ok(())
}

pub fn fs_lseek(caller: ProcessId, fd: FD, offset: usize) -> Result<(), ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let fs_lock = IN_MEM_FS.lock();
    let file = fs_lock.file(handle.file_path()).ok_or(())?;
    let offset = min(offset, file.data().len());
    handle.file_offset = offset;
    Ok(())
}

pub fn fs_close(caller: ProcessId, fd: FD) -> Result<(), ()> {
    let mut lock = OPEN_FILE_TABLE.lock();
    lock.close(caller, fd)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fs() {
        let fd = fs_open(
            1,
            String::from("/foo/bar"),
            FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
            0o777,
        );
        fs_write(1, fd, b"Hallo Welt!").unwrap();
        fs_lseek(1, fd, "Hallo ".len()).unwrap();
        let read = fs_read(1, fd, 100).unwrap();
        let read = String::from_utf8(read).unwrap();
        assert_eq!(read, "Welt!");

        fs_lseek(1, fd, 0).unwrap();
        let read = fs_read(1, fd, 100).unwrap();
        let read = String::from_utf8(read).unwrap();
        assert_eq!(read, "Hallo Welt!")
    }
}
