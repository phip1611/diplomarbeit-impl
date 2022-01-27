use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
#[allow(unused)]
pub struct IoctlSyscall {
    fd: u64,
    request: u64,
}

impl From<&GenericLinuxSyscall> for IoctlSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: syscall.arg0(),
            request: syscall.arg1(),
        }
    }
}

impl LinuxSyscallImpl for IoctlSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException, _process: &Process) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}
