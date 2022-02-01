use super::super::FD;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Data send via UTCB to Fs Read Portal.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsReadRequest {
    fd: FD,
    user_ptr: usize,
    count: usize,
}

impl FsReadRequest {
    pub fn new(fd: FD, user_ptr: usize, count: usize) -> Self {
        FsReadRequest {
            fd,
            user_ptr,
            count,
        }
    }

    pub fn fd(&self) -> FD {
        self.fd
    }
    pub fn user_ptr(&self) -> usize {
        self.user_ptr
    }
    pub fn count(&self) -> usize {
        self.count
    }
}
