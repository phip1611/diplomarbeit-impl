use crate::mem::UserPtrOrEmbedded;
use crate::rt::services::fs::fd::FD;
use crate::rt::services::fs::fs_close::{
    fs_service_close,
    FsCloseRequest,
};
use crate::rt::services::fs::fs_lseek::{
    fs_service_lseek,
    FsLseekRequest,
};
use crate::rt::services::fs::fs_open::{
    fs_service_open,
    FsOpenFlags,
    FsOpenRequest,
};
use crate::rt::services::fs::fs_read::{
    fs_service_read,
    FsReadRequest,
};
use crate::rt::services::fs::fs_write::{
    fs_service_write,
    FsWriteRequest,
};
use alloc::string::ToString;
use alloc::vec::Vec;
use libhedron::mem::PAGE_SIZE;

/// A file abstraction over the underlying service portals that talk to the file
/// system service.
#[derive(Debug)]
pub struct File {
    fd: FD,
}

impl File {
    /// Opens a file.
    pub fn open(path: &str, flags: FsOpenFlags, umode: u16) -> Self {
        let fd = fs_service_open(FsOpenRequest::new(path.to_string(), flags, umode));
        Self { fd }
    }

    /// Writes all bytes to the file.
    pub fn write_all(&mut self, bytes: &[u8]) -> usize {
        fs_service_write(FsWriteRequest::new(
            self.fd,
            UserPtrOrEmbedded::EmbeddedSlice(bytes.to_vec()),
            bytes.len(),
        ))
    }

    /// This returns all bytes until the file system returns EOF.
    pub fn read_to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::<u8>::with_capacity(PAGE_SIZE);
        let mut tmp_data = Vec::<u8>::with_capacity(PAGE_SIZE);
        loop {
            let read_bytes = fs_service_read(FsReadRequest::new(
                self.fd,
                tmp_data.as_mut_ptr() as usize,
                data.capacity(),
            ));
            log::info!("read_bytes = {}", read_bytes);
            if read_bytes == 0 {
                break;
            } else {
                unsafe { tmp_data.set_len(read_bytes) };
                data.extend_from_slice(tmp_data.as_slice());
            }
        }
        data
    }

    /// Updates the file offset of the opened file.
    pub fn lseek(&mut self, offset: u64) {
        fs_service_lseek(FsLseekRequest::new(self.fd, offset));
    }

    /// Closes a file.
    pub fn close(self) {
        fs_service_close(FsCloseRequest::new(self.fd));
    }
}
