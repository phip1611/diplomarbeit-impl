use crate::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::{
    FsOpenRequest,
    FD,
};

/// Implements the fs open service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_open(request: &FsOpenRequest, utcb: &mut Utcb, process: &Process) {
    let fd = libfileserver::FILESYSTEM.lock().open_or_create_file(
        process.pid(),
        request.path(),
        request.flags(),
        request.umode(),
    );
    let fd = if let Ok(fd) = fd {
        FD::new(fd.val() as _)
    } else {
        FD::error()
    };
    utcb.store_data(&fd).unwrap();
}
