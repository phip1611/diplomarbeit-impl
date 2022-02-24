//! File server lib. Currently this library only contains the internal interface of the
//! file system server. The public interface (exported via Portals) must be build around
//! these interfaces.

#![no_std]
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
// I see a benefit here: Even tho it might not be usable from the outside world,
// it may contain useful information about how the implementation works.
#![allow(rustdoc::private_intra_doc_links)]
#![allow(rustdoc::missing_doc_code_examples)]
#![feature(const_ptr_offset)]
#![feature(const_fmt_arguments_new)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(const_btree_new)]
#![feature(slice_ptr_get)]
#![allow(unused)]

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
use libhrstd::rt::services::fs::FsOpenFlags;
use libhrstd::rt::services::fs::FD;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::global_counter::GlobalIncrementingCounter;

/// Open file table with open files in [`IN_MEM_FS`].
static OPEN_FILE_TABLE: SimpleMutex<OpenFileTable> = SimpleMutex::new(OpenFileTable::new());

/// Contains the file system in memory.
static IN_MEM_FS: SimpleMutex<InMemFilesystem> = SimpleMutex::new(InMemFilesystem::new());

/// Counter to give unique inodes (=identifiers) to files. Currently, this is auto incrementing
/// for ever.
static INODE_COUNTER: GlobalIncrementingCounter = GlobalIncrementingCounter::new();

/// Holds information about all files that are currently open inside the system.
#[derive(Debug)]
struct OpenFileTable(BTreeMap<OpenFileHandleId, OpenFileHandle>);

impl OpenFileTable {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Opens a file and returns a [`FD`].
    pub fn open(
        &mut self,
        caller: ProcessId,
        path: &str,
        flags: FsOpenFlags,
        umode: u16,
    ) -> Result<FD, ()> {
        let fd = self.next_fd(caller);

        let mut in_mem_fs = IN_MEM_FS.lock();
        let maybe_file = in_mem_fs.get_file_by_path(&path);
        if maybe_file.is_none() & flags.can_create() {
            // create new file
            let path = String::from(path);
            let new_file = InMemFile::new(path.clone(), FileMetaData::new(umode, caller));
            let i_node = new_file.i_node();
            in_mem_fs.create_file(path, new_file).unwrap();
            self.data_mut()
                .insert((caller, fd), OpenFileHandle::new(flags, i_node));
            Ok(fd)
        } else if in_mem_fs.get_file_by_path(&path).is_none() {
            // file doesn't exist or can't get created
            Err(())
        } else {
            let file = maybe_file.unwrap();
            // open existing file
            self.data_mut()
                .insert((caller, fd), OpenFileHandle::new(flags, file.i_node()));
            Ok(fd)
        }
    }

    /// Closes a file.
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

    /// Returns the next available file descriptor for a process.
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

/// Combines a process ID that has opened a certain file descriptor.
/// Identifies objects of type [`OpenFileHandle`].
type OpenFileHandleId = (ProcessId, FD);

/// Describes an opened file.
#[derive(Debug)]
struct OpenFileHandle {
    // used as ID
    i_node: u64,
    file_offset: usize,
    flags: FsOpenFlags,
}

impl OpenFileHandle {
    pub fn new(flags: FsOpenFlags, i_node: u64) -> Self {
        OpenFileHandle {
            file_offset: 0,
            flags,
            i_node,
        }
    }

    pub fn file_offset(&self) -> usize {
        self.file_offset
    }
    pub fn flags(&self) -> FsOpenFlags {
        self.flags
    }

    pub fn i_node(&self) -> u64 {
        self.i_node
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
    i_node: u64,
    path: String,
    data: Vec<u8>,
    meta: FileMetaData,
}

impl InMemFile {
    pub fn new(path: String, meta: FileMetaData) -> Self {
        Self {
            i_node: INODE_COUNTER.next(),
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
    pub fn i_node(&self) -> u64 {
        self.i_node
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

    fn create_file(&mut self, filepath: String, file: InMemFile) -> Result<(), ()> {
        if self.files.contains_key(&filepath) {
            Err(())
        } else {
            self.files.insert(filepath, file);
            Ok(())
        }
    }

    fn get_file_by_inode(&self, i_node: u64) -> Option<&InMemFile> {
        self.files
            .iter()
            .map(|(_, file)| file)
            .find(|file| file.i_node() == i_node)
    }

    fn get_file_by_inode_mut(&mut self, i_node: u64) -> Option<&mut InMemFile> {
        self.files
            .iter_mut()
            .map(|(_, file)| file)
            .find(|file| file.i_node() == i_node)
    }

    fn get_file_by_path(&self, filepath: &str) -> Option<&InMemFile> {
        self.files.get(filepath)
    }

    fn get_file_by_path_mut(&mut self, filepath: &str) -> Option<&mut InMemFile> {
        self.files.get_mut(filepath)
    }

    fn delete_file_by_path(&mut self, filepath: &str) -> bool {
        self.files.remove(filepath).is_some()
    }
}

/// Public interface to the file system management data structures to open files.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX. On success, a new [`FD`] gets returned.
pub fn fs_open(caller: ProcessId, path: &str, flags: FsOpenFlags, umode: u16) -> FD {
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

/// Public interface to the file system management data structures to read from open files.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX. On success, a Vector with read bytes gets returned.
pub fn fs_read(caller: ProcessId, fd: FD, count: usize) -> Result<Vec<u8>, ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let in_mem_fs_lock = IN_MEM_FS.lock();
    let offset = handle.file_offset();
    let file = in_mem_fs_lock
        .get_file_by_inode(handle.i_node())
        .ok_or(())?;

    let mut data = Vec::new();
    let new_offset = min(file.data().len(), count + offset);
    handle.file_offset = new_offset;

    data.extend_from_slice(&file.data()[offset..new_offset]);
    Ok(data)
}

/// Public interface to the file system management data structures to write to open files.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX. On success, the number of written bytes gets returned.
pub fn fs_write(caller: ProcessId, fd: FD, new_data: &[u8]) -> Result<usize, ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let mut in_mem_fs_lock = IN_MEM_FS.lock();

    let file = in_mem_fs_lock
        .get_file_by_inode_mut(handle.i_node())
        .ok_or(())?;

    // get offset; i.e.: the point where we start to append data
    // on UNIX, APPEND always appends; independent from the file offset
    let offset = if handle.flags().is_append() {
        file.data.len() - 1
    } else {
        handle.file_offset()
    };

    // the final file offset, after the new data got written.
    let final_file_offset = offset + new_data.len();
    handle.file_offset = final_file_offset;

    // Q&D: increase capacity
    // Make sure the vector allocates enough memory, before I start to write data.
    for i in file.data.capacity()..final_file_offset {
        file.data_mut().push(0);
    }

    // Make sure "extend" starts at the right length
    unsafe {
        file.data_mut().set_len(offset);
    }
    file.data_mut().extend_from_slice(new_data);

    let written_bytes = new_data.len();
    Ok(written_bytes)
}

/// Public interface to the file system management data structures to set the internal
/// files offset of an open file
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX.
pub fn fs_lseek(caller: ProcessId, fd: FD, offset: usize) -> Result<(), ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let fs_lock = IN_MEM_FS.lock();
    let file = fs_lock.get_file_by_inode(handle.i_node()).ok_or(())?;
    if offset > file.data().len() {
        log::warn!("offset > file.data.len()");
        // TODO not sure how UNIX handles this
    }
    let offset = min(offset, file.data().len());
    handle.file_offset = offset;
    Ok(())
}

/// Public interface to the file system management data structures to get the fstat data structure.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX.
pub fn fs_fstat(caller: ProcessId, fd: FD) -> Result<FileStat, ()> {
    let key = (caller, fd);
    let mut open_file_table_lock = OPEN_FILE_TABLE.lock();
    let mut handle = open_file_table_lock.data_mut().get_mut(&key).ok_or(())?;
    let fs_lock = IN_MEM_FS.lock();
    let file = fs_lock.get_file_by_inode(handle.i_node()).ok_or(())?;
    Ok(FileStat::from(file))
}

/// Public interface to the file system management data structures to close open files.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX.
pub fn fs_close(caller: ProcessId, fd: FD) -> Result<(), ()> {
    let mut lock = OPEN_FILE_TABLE.lock();
    lock.close(caller, fd)?;
    Ok(())
}

/// Public interface to the file system management data structures to unlink a file.
///
/// This is not the public service API that gets exported via portals but the
/// public service Portals will wrap around these functions.
///
/// The interface is close to UNIX.
pub fn fs_unlink(caller: ProcessId, file: &str) -> Result<(), ()> {
    let mut fs_lock = IN_MEM_FS.lock();
    // TODO don't know yet how this interacts with files opened in the open file table
    if fs_lock.delete_file_by_path(file) {
        log::trace!("deletion successful");
        Ok(())
    } else {
        log::trace!("deletion failed");
        Err(())
    }
}

/// This is identical to the UNIX/libc stat type.
#[repr(C)]
#[derive(Debug)]
pub struct FileStat {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

impl FileStat {
    pub fn st_dev(&self) -> u64 {
        self.st_dev
    }
    pub fn st_ino(&self) -> u64 {
        self.st_ino
    }
    pub fn st_nlink(&self) -> u64 {
        self.st_nlink
    }
    pub fn st_mode(&self) -> u32 {
        self.st_mode
    }
    pub fn st_uid(&self) -> u32 {
        self.st_uid
    }
    pub fn st_gid(&self) -> u32 {
        self.st_gid
    }
    pub fn st_rdev(&self) -> u64 {
        self.st_rdev
    }
    pub fn st_size(&self) -> i64 {
        self.st_size
    }
    pub fn st_blksize(&self) -> i64 {
        self.st_blksize
    }
    pub fn st_blocks(&self) -> i64 {
        self.st_blocks
    }
    pub fn st_atime(&self) -> i64 {
        self.st_atime
    }
    pub fn st_atime_nsec(&self) -> i64 {
        self.st_atime_nsec
    }
    pub fn st_mtime(&self) -> i64 {
        self.st_mtime
    }
    pub fn st_mtime_nsec(&self) -> i64 {
        self.st_mtime_nsec
    }
    pub fn st_ctime(&self) -> i64 {
        self.st_ctime
    }
    pub fn st_ctime_nsec(&self) -> i64 {
        self.st_ctime_nsec
    }
}

impl From<&InMemFile> for FileStat {
    fn from(file: &InMemFile) -> Self {
        Self {
            st_dev: 0,
            st_ino: file.i_node(),
            st_nlink: 0,
            st_mode: file.meta().umode() as u32,
            st_uid: 0,
            st_gid: 0,
            __pad0: 0,
            st_rdev: 0,
            st_size: file.data().len() as i64,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused: [0; 3],
        }
    }
}

// caution: tests will share the state from the globally shared variables
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fs_basic() {
        let fd = fs_open(
            1,
            "/foo/test1",
            FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
            0o777,
        );
        fs_write(1, fd, b"Hallo Welt!").unwrap();
        fs_lseek(1, fd, "Hallo ".len()).unwrap();
        let read = fs_read(1, fd, 100).unwrap();
        let read = String::from_utf8(read).unwrap();
        // get rid of additional zeroes
        let read = read.trim_matches('\0');
        assert_eq!(read, "Welt!");

        fs_lseek(1, fd, 0).unwrap();
        let read = fs_read(1, fd, 100).unwrap();
        let read = String::from_utf8(read).unwrap();
        // get rid of additional zeroes
        let read = read.trim_matches('\0');
        assert_eq!(read, "Hallo Welt!")
    }

    #[test]
    fn test_fs_lseek_write_size() {
        let payload = [0; 16384];
        let fd = fs_open(
            1,
            "/foo/test2",
            FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
            0o777,
        );
        assert_eq!(fs_fstat(1, fd).unwrap().st_size(), 0);
        fs_write(1, fd, &payload).unwrap();
        assert_eq!(fs_fstat(1, fd).unwrap().st_size(), 16384);
        fs_lseek(1, fd, 0).unwrap();
        assert_eq!(fs_fstat(1, fd).unwrap().st_size(), 16384);
        fs_write(1, fd, &payload).unwrap();
        assert_eq!(fs_fstat(1, fd).unwrap().st_size(), 16384);
    }

    #[test]
    fn test_fs_unlink() {
        let filename = "/foo/test3";
        let fd = fs_open(
            1,
            filename.clone(),
            FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
            0o777,
        );
        {
            let fs_lock = IN_MEM_FS.lock();
            assert!(fs_lock.get_file_by_path(&filename).is_some());
        }

        fs_unlink(1, &filename).unwrap();
        {
            let fs_lock = IN_MEM_FS.lock();
            assert!(fs_lock.get_file_by_path(&filename).is_none());
            // without this, tests get stuck (because some methods lock
            // the open file table first and then we have a deadlock
            drop(fs_lock);

            let open_ft_lock = OPEN_FILE_TABLE.lock();
            assert!(
                open_ft_lock.data().get(&(1, fd)).is_some(),
                "file must stay opened in open file table"
            )
        }
    }
}
