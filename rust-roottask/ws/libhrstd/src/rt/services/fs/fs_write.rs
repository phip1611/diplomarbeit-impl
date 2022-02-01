use crate::cap_space::user::UserAppCapSpace;
use crate::mem::UserPtrOrEmbedded;
use crate::rt::services::fs::fd::FD;
use crate::rt::services::fs::service::FsServiceRequest;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};
use libhedron::syscall::sys_call;
use crate::rt::hybrid_rt::syscalls::{sys_hybrid_call, sys_hybrid_create_sc};

/// Wrapper around the FS service portal to write to files.
/// Returns the number of written bytes.
pub fn fs_service_write(request: FsWriteRequest) -> usize {
    let utcb = user_load_utcb_mut();
    let request = FsServiceRequest::Write(request);
    utcb.store_data(&request).unwrap();

    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::FsServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::FsServicePT.val()).unwrap();

    (UserAppCapSpace::FsServicePT.val()).unwrap();
    utcb.load_data().unwrap()
}

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
