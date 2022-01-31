use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::fs_lseek::FsLseekRequest;

pub(super) fn fs_service_lseek(request: &FsLseekRequest, _utcb: &mut Utcb, process: &Process) {
    libfileserver::fs_lseek(process.pid(), request.fd(), request.offset() as usize).unwrap();
}
