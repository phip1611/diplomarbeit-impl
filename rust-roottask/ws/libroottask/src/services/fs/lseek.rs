use crate::process_mng::process::Process;
use libhrstd::libhedron::Utcb;
use libhrstd::rt::services::fs::FsLseekRequest;

/// Implements the fs lseek service functionality that is accessible via the FS portal.
pub(super) fn fs_service_impl_lseek(request: &FsLseekRequest, _utcb: &mut Utcb, process: &Process) {
    libfileserver::fs_lseek(process.pid(), request.fd(), request.offset() as usize).unwrap();
}
