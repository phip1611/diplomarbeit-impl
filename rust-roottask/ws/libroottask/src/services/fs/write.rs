use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::fs_write::FsWriteRequest;

pub(super) fn fs_service_write(request: &FsWriteRequest, utcb: &mut Utcb, process: &Process) {
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
