use crate::cap_space::user::UserAppCapSpace;
#[cfg(feature = "foreign_rust_rt")]
use crate::rt::hybrid_rt::syscalls::sys_hybrid_call;
use crate::rt::services::fs::fd::FD;
use crate::rt::services::fs::service::FsServiceRequest;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};
#[cfg(feature = "native_rust_rt")]
use libhedron::syscall::sys_call;

/// Wrapper around the FS service portal to update the file offset.
#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
pub fn fs_service_lseek(request: FsLseekRequest) -> FD {
    let utcb = user_load_utcb_mut();
    let request = FsServiceRequest::LSeek(request);
    utcb.store_data(&request).unwrap();

    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::FsServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::FsServicePT.val()).unwrap();

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
