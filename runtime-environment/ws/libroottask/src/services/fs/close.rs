use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsCloseRequest;

/// Implements the fs close service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_close(request: &FsCloseRequest, _utcb: &mut Utcb, process: &Process) {
    libfileserver::fs_close(process.pid(), request.fd()).unwrap();
}