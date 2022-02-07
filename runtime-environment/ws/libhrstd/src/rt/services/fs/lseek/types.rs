use super::super::FD;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Data send via UTCB to Fs Read Portal.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsLseekRequest {
    fd: FD,
    offset: u64,
}

impl FsLseekRequest {
    pub fn new(fd: FD, offset: u64) -> Self {
        Self { fd, offset }
    }

    pub fn fd(&self) -> FD {
        self.fd
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
}
