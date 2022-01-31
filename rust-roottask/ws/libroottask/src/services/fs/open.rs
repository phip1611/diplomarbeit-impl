use crate::process_mng::process::Process;
use alloc::string::ToString;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::fs_open::FsOpenRequest;

pub(super) fn fs_service_open(request: &FsOpenRequest, utcb: &mut Utcb, process: &Process) {
    let fd = libfileserver::fs_open(
        process.pid(),
        request.path().to_string(),
        request.flags(),
        request.umode(),
    );
    utcb.store_data(&fd).unwrap();
}
