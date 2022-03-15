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

mod file_descriptor;
mod file_table;
mod in_mem_fs;
mod inode;
mod stat;

use crate::file_table::OpenFileTable;
use crate::in_mem_fs::{
    FileMetaData,
    InMemFile,
    InMemFilesystem,
};
use alloc::string::String;
use core::cmp::min;
pub use file_descriptor::FileDescriptor;
use libhrstd::process::consts::ProcessId;
use libhrstd::rt::services::fs::FsOpenFlags;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::global_counter::GlobalIncrementingCounter;
pub use stat::FileStat;

/// Public facade to the file system. See [`Filesystem`].
pub static FILESYSTEM: SimpleMutex<Filesystem> = SimpleMutex::new(Filesystem::new());

/// Counter to give unique inodes (=identifiers) to files. Currently, this is auto incrementing
/// for ever.
static INODE_COUNTER: GlobalIncrementingCounter = GlobalIncrementingCounter::new();

/// Facade over the virtual file system that contains the in-memory file system and possibly
/// others in the future.
#[derive(Debug)]
pub struct Filesystem {
    in_mem_fs: InMemFilesystem,
    open_file_table: OpenFileTable,
}

impl Filesystem {
    const fn new() -> Self {
        Self {
            in_mem_fs: InMemFilesystem::new(),
            open_file_table: OpenFileTable::new(),
        }
    }

    /// Public interface to the file system management data structures to open files.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX. On success, a new [`FD`] gets returned.
    pub fn open_or_create_file(
        &mut self,
        caller: ProcessId,
        path: &str,
        flags: FsOpenFlags,
        umode: u16,
    ) -> Result<FileDescriptor, ()> {
        if flags.is_empty() {
            return Err(());
        };
        if path.is_empty() {
            return Err(());
        }

        // the file either:
        // - does not exist and may be created
        // - or already exist
        let maybe_file = self.in_mem_fs.get_file_by_path(&path);

        if maybe_file.is_none() & flags.can_create() {
            // create new file
            let i_node = INODE_COUNTER.next().into();
            let new_file =
                InMemFile::new(i_node, String::from(path), FileMetaData::new(umode, caller));
            self.in_mem_fs.create_file(i_node, new_file)?;
            let fd = self.open_file_table.open(caller, i_node, flags)?;
            log::trace!("file creation successful: path={}, flags={:?}", path, flags);
            Ok(fd)
        } else if maybe_file.is_none() {
            // file doesn't exist or can't get created
            log::trace!("file open error: path={}, flags={:?}", path, flags);
            Err(())
        } else {
            let file = maybe_file.ok_or(())?;
            // open existing file
            let fd = self.open_file_table.open(caller, file.i_node(), flags)?;
            Ok(fd)
        }
    }

    /// Public interface to the file system management data structures to read from open files.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX. On success, a Vector with read bytes gets returned.
    pub fn read_file(
        &mut self,
        caller: ProcessId,
        fd: FileDescriptor,
        count: usize,
    ) -> Result<&[u8], ()> {
        let open_handle = self
            .open_file_table
            .lookup_handle_mut(caller, fd)
            .ok_or(())?;

        let file = self
            .in_mem_fs
            .get_file_by_inode(open_handle.i_node())
            .ok_or(())?;

        let from_index = open_handle.file_offset();
        let to_index = min(from_index + count, file.data().len());
        // update file offset is important! So that next read continues where the
        // previous read stopped
        open_handle.file_offset = to_index;
        let slice = &file.data()[from_index..to_index];
        Ok(slice)
    }

    /// Public interface to the file system management data structures to write to open files.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX. On success, the number of written bytes gets returned.
    pub fn write_file(
        &mut self,
        caller: ProcessId,
        fd: FileDescriptor,
        new_data: &[u8],
    ) -> Result<usize, ()> {
        let open_handle = self
            .open_file_table
            .lookup_handle_mut(caller, fd)
            .ok_or(())?;

        let file = self
            .in_mem_fs
            .get_file_by_inode_mut(open_handle.i_node())
            .ok_or(())?;

        // get offset; i.e.: the point where we start to append data
        // on UNIX, APPEND always appends; independent from the file offset
        let write_begin_offset = if open_handle.flags().is_append() {
            file.data().len()
        } else {
            open_handle.file_offset()
        };

        // This may truncate the vector but old data stay in memory unless overwritten.
        // This is no data-leak because at this point the capacity can never shrink
        unsafe {
            file.data_mut().set_len(write_begin_offset);
        }

        // the final file offset, after the new data got written.
        let new_length = write_begin_offset + new_data.len();
        open_handle.file_offset = new_length;

        // increase capacity if necessary
        let vec_current_capacity = file.data_mut().capacity();
        if new_data.len() > vec_current_capacity {
            file.data_mut()
                .reserve_exact(new_length - vec_current_capacity);
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
    pub fn lseek_file(
        &mut self,
        caller: ProcessId,
        fd: FileDescriptor,
        offset: usize,
    ) -> Result<(), ()> {
        let open_handle = self
            .open_file_table
            .lookup_handle_mut(caller, fd)
            .ok_or(())?;

        let file = self
            .in_mem_fs
            .get_file_by_inode(open_handle.i_node())
            .ok_or(())?;

        if offset > file.data().len() {
            log::warn!("offset >= file.data.len()");
            // TODO not sure how UNIX handles this
        }
        let offset = min(offset, file.data().len());
        open_handle.file_offset = offset;
        Ok(())
    }

    /// Public interface to the file system management data structures to get the fstat data structure.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX.
    pub fn fstat(&mut self, caller: ProcessId, fd: FileDescriptor) -> Result<FileStat, ()> {
        let open_handle = self
            .open_file_table
            .lookup_handle_mut(caller, fd)
            .ok_or(())?;

        let file = self
            .in_mem_fs
            .get_file_by_inode(open_handle.i_node())
            .ok_or(())?;

        Ok(FileStat::from(file))
    }

    /// Public interface to the file system management data structures to close open files.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX.
    pub fn close_file(&mut self, caller: ProcessId, fd: FileDescriptor) -> Result<(), ()> {
        self.open_file_table.close(caller, fd)
    }

    /// Public interface to the file system management data structures to unlink a file.
    ///
    /// This is not the public service API that gets exported via portals but the
    /// public service Portals will wrap around these functions.
    ///
    /// The interface is close to UNIX.
    pub fn unlink_file(&mut self, _caller: ProcessId, file: &str) -> Result<(), ()> {
        // TODO don't know yet how this interacts with files opened in the open file table
        if self.in_mem_fs.delete_file_by_path(file) {
            log::trace!("deletion successful");
            Ok(())
        } else {
            log::trace!("deletion failed");
            Err(())
        }
    }
}

// caution: tests will share the state from the globally shared variables
#[cfg(test)]
mod tests {
    use super::*;
    use libhrstd::time::Instant;
    use std::vec::Vec;

    #[test]
    fn test_fs_basic() {
        let mut fs = FILESYSTEM.lock();
        let fd = fs
            .open_or_create_file(
                1,
                "/foo/test1",
                FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
                0o777,
            )
            .unwrap();
        fs.write_file(1, fd, b"Hallo Welt!").unwrap();
        fs.lseek_file(1, fd, "Hallo ".len()).unwrap();
        let read = fs.read_file(1, fd, 100).unwrap();
        let read = String::from_utf8_lossy(read);
        // get rid of additional zeroes
        let read = read.trim_matches('\0');
        assert_eq!(read, "Welt!");

        fs.lseek_file(1, fd, 0).unwrap();
        let read = fs.read_file(1, fd, 100).unwrap();
        let read = String::from_utf8_lossy(read);
        // get rid of additional zeroes
        let read = read.trim_matches('\0');
        assert_eq!(read, "Hallo Welt!")
    }

    #[test]
    fn test_fs_lseek_write_size() {
        let mut fs = FILESYSTEM.lock();
        let payload = [0; 16384];
        for i in 0..10 {
            let fd = fs
                .open_or_create_file(
                    1,
                    "/foo/test2",
                    FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
                    0o777,
                )
                .unwrap();

            // is first iteration
            if i == 0 {
                assert_eq!(fs.fstat(1, fd).unwrap().st_size(), 0, "file size must be 0");
            } else {
                fs.lseek_file(1, fd, 0).unwrap();
            }

            fs.write_file(1, fd, &payload).unwrap();
            assert_eq!(
                fs.fstat(1, fd).unwrap().st_size(),
                16384,
                "the file size must match the previous write"
            );
            fs.lseek_file(1, fd, 0).unwrap();
            assert_eq!(fs.fstat(1, fd).unwrap().st_size(), 16384, "the file size must match the previous write even if the file pointer was reset to the beginning");
            fs.write_file(1, fd, &payload).unwrap();
            assert_eq!(fs.fstat(1, fd).unwrap().st_size(), 16384, "the file size must stay the same because the file offset was reset to the beginning.");
        }
    }

    #[test]
    fn test_fs_unlink() {
        let mut fs = FILESYSTEM.lock();
        let filename = "/foo/test3";
        let fd = fs
            .open_or_create_file(
                1,
                filename.clone(),
                FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
                0o777,
            )
            .unwrap();
        {
            assert!(fs.in_mem_fs.get_file_by_path(&filename).is_some());
        }

        fs.unlink_file(1, &filename).unwrap();
        {
            assert!(fs.in_mem_fs.get_file_by_path(&filename).is_none());

            assert!(
                fs.open_file_table.lookup_handle(1, fd).is_some(),
                "file must stay opened in open file table"
            )
        }
    }

    /// The tests above do basic functionality of read and write. This test checks with random
    /// data if the data written is actually the data read. Furthermore, it splits read and
    /// write operation into multiple chunks.
    #[test]
    fn test_fs_correctness() {
        for outer_iteration in 0..10 {
            const BYTE_COUNT: usize = 2049;
            const CHUNK_SIZE: usize = 1024;
            let random_data_2049 = (0..BYTE_COUNT)
                .map(|_| Instant::now().val())
                .flat_map(|x| x.to_ne_bytes())
                .take(BYTE_COUNT)
                .collect::<Vec<_>>();

            let bench_file_path = "/tmp_foobar_fs_correctness";

            let mut fs = FILESYSTEM.lock();
            let fd = fs
                .open_or_create_file(
                    1,
                    bench_file_path,
                    // make sure that we don't set the "append" flag :)
                    FsOpenFlags::O_CREAT | FsOpenFlags::O_RDWR,
                    0o777,
                )
                .unwrap();

            for inner_iteration in 0..100 {
                assert_eq!(
                    fs.in_mem_fs.get_file_by_path(bench_file_path).unwrap().inner_vec().capacity(),
                    InMemFile::DEFAULT_CAPACITY,
                    "the capacity should not grow across multiple iterations because the file offset gets resettet every time!"
                );

                // I execute this test multiple times. However, each iteration should start at
                // the "raw" state.
                fs.lseek_file(1, fd, 0).unwrap();

                // ############ BEGIN WRITE IN THREE STEPS ############
                let bytes_written = fs
                    .write_file(1, fd, &random_data_2049[..CHUNK_SIZE])
                    .unwrap();
                assert_eq!(bytes_written, CHUNK_SIZE, "must write all bytes");
                assert_eq!(
                    CHUNK_SIZE,
                    fs.in_mem_fs.get_file_by_path(bench_file_path).unwrap().inner_vec().len(),
                    "larger than expected! [inner_iteration={inner_iteration}, outer_iteration={outer_iteration}]"
                );

                let bytes_written = fs
                    .write_file(1, fd, &random_data_2049[CHUNK_SIZE..][..CHUNK_SIZE])
                    .unwrap();
                assert_eq!(bytes_written, CHUNK_SIZE, "must write all bytes");
                assert_eq!(
                    2 * CHUNK_SIZE,
                    fs.in_mem_fs.get_file_by_path(bench_file_path).unwrap().inner_vec().len(),
                    "larger than expected! [inner_iteration={inner_iteration}, outer_iteration={outer_iteration}]"
                );

                let bytes_written = fs
                    .write_file(1, fd, &random_data_2049[2 * CHUNK_SIZE..])
                    .unwrap();
                assert_eq!(bytes_written, 1, "only one byte is left");

                // with the "lseek(0)" at the beginning of each iteration I want to ensure that
                // the file content gets never longer and always stay the same. Here I check if
                // that actually works.
                assert_eq!(
                    BYTE_COUNT,
                    fs.fstat(1, fd).unwrap().st_size() as usize,
                    "the file must be as long as the data that was written to it and no longer\n\
                    [inner_iteration={inner_iteration}, outer_iteration={outer_iteration}]"
                );
                // ############ END WRITE IN THREE STEPS ############

                // ############ BEGIN READ IN THREE STEPS ############
                // create a readbuffer that is big enough
                let mut read_buf = Vec::with_capacity(random_data_2049.len());

                // make sure that read now starts at the beginning
                fs.lseek_file(1, fd, 0).unwrap();

                let read_bytes = fs.read_file(1, fd, CHUNK_SIZE).unwrap();
                assert_eq!(read_bytes.len(), CHUNK_SIZE, "must read {CHUNK_SIZE} bytes");
                read_buf.extend_from_slice(read_bytes);

                let read_bytes = fs.read_file(1, fd, CHUNK_SIZE).unwrap();
                assert_eq!(read_bytes.len(), CHUNK_SIZE, "must read {CHUNK_SIZE} bytes");
                read_buf.extend_from_slice(read_bytes);

                let read_bytes = fs.read_file(1, fd, CHUNK_SIZE).unwrap();
                assert_eq!(read_bytes.len(), 1, "must read exactly 1 byte that is left");
                read_buf.extend_from_slice(read_bytes);
                // ############ END READ IN THREE STEPS ############

                // make sure read and write data is equal
                assert_eq!(
                    read_buf, random_data_2049,
                    "read and write data must equal!"
                );
            }

            // Nope to unlink :) I want to re-use the file in succeeding iterations once it
            // got created to simulate/test a more realistic layout.
            // fs.unlink_file(1, bench_file_path).unwrap();
        }
    }
}
