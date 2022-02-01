use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsWriteRequest;

/// Implements the fs write service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_write(request: &FsWriteRequest, utcb: &mut Utcb, process: &Process) {
    libfileserver::fs_write(
        process.pid(),
        request.fd(),
        // currently don't support user ptr read
        request.data().embedded_slice(),
    )
    .unwrap();

    utcb.store_data(&request.data().embedded_slice().len())
        .unwrap();
}
