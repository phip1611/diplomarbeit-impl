use crate::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsCloseRequest;

/// Implements the fs close service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_close(request: &FsCloseRequest, _utcb: &mut Utcb, process: &Process) {
    libfileserver::FILESYSTEM
        .lock()
        .close_file(process.pid(), (request.fd().raw() as u64).into())
        .unwrap();
}
