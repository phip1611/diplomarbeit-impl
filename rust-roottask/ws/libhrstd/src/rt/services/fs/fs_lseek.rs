use crate::cap_space::user::UserAppCapSpace;
use crate::rt::services::fs::fd::FD;
use crate::rt::services::fs::service::FsServiceRequest;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};
use libhedron::syscall::ipc::call;

pub fn fs_lseek(request: FsLseekRequest) -> FD {
    let utcb = user_load_utcb_mut();
    let request = FsServiceRequest::LSeek(request);
    utcb.store_data(&request).unwrap();
    call(UserAppCapSpace::FsServicePT.val()).unwrap();
    utcb.load_data().unwrap()
}

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
