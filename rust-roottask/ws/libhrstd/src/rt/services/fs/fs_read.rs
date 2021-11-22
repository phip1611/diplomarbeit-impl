use crate::cap_space::user::UserAppCapSpace;
use crate::rt::services::fs::fd::FD;
use crate::rt::services::fs::service::FsServiceRequest;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};
use libhedron::syscall::ipc::call;

pub fn fs_read(request: FsReadRequest) -> usize {
    let utcb = user_load_utcb_mut();
    let request = FsServiceRequest::Read(request);
    utcb.store_data(&request).unwrap();
    call(UserAppCapSpace::FsServicePT.val()).unwrap();
    utcb.load_data().unwrap()
}

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
