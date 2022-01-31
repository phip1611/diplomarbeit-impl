use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::fs_close::FsCloseRequest;

pub(super) fn fs_service_close(request: &FsCloseRequest, _utcb: &mut Utcb, process: &Process) {
    libfileserver::fs_close(process.pid(), request.fd()).unwrap();
}
