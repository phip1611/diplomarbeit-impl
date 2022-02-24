use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsOpenRequest;

/// Implements the fs open service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_open(request: &FsOpenRequest, utcb: &mut Utcb, process: &Process) {
    let fd = libfileserver::fs_open(
        process.pid(),
        request.path(),
        request.flags(),
        request.umode(),
    );
    utcb.store_data(&fd).unwrap();
}
