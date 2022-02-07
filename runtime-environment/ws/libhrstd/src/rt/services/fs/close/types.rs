use super::super::FD;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Data send via UTCB to Fs Close Portal.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsCloseRequest {
    fd: FD,
}

impl FsCloseRequest {
    pub fn new(fd: FD) -> Self {
        FsCloseRequest { fd }
    }

    pub fn fd(&self) -> FD {
        self.fd
    }
}
