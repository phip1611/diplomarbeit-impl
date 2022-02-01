use super::super::FD;
use crate::mem::UserPtrOrEmbedded;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Data send via UTCB to Fs Write Portal.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsWriteRequest {
    fd: FD,
    data: UserPtrOrEmbedded<u8>,
    count: usize,
}

impl FsWriteRequest {
    pub fn new(fd: FD, data: UserPtrOrEmbedded<u8>, count: usize) -> Self {
        FsWriteRequest { fd, data, count }
    }

    pub fn fd(&self) -> FD {
        self.fd
    }
    pub fn data(&self) -> &UserPtrOrEmbedded<u8> {
        &self.data
    }
    pub fn count(&self) -> usize {
        self.count
    }
}
