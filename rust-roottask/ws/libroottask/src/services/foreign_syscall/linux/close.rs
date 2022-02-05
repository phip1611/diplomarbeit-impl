use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

#[derive(Debug)]
pub struct CloseSyscall {
    fd: FD,
}

impl From<&GenericLinuxSyscall> for CloseSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FD::new(syscall.arg0() as i32),
        }
    }
}

impl LinuxSyscallImpl for CloseSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        libfileserver::fs_close(process.pid(), self.fd).unwrap();

        LinuxSyscallResult::new_success(0)
    }
}
