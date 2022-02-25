use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

/// Syscall that immediately returns success.
#[derive(Debug)]
pub struct FakeJustReturnSyscall {}

impl From<&GenericLinuxSyscall> for FakeJustReturnSyscall {
    fn from(_syscall: &GenericLinuxSyscall) -> Self {
        Self {}
    }
}

impl LinuxSyscallImpl for FakeJustReturnSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        LinuxSyscallResult::new_success(0)
    }
}
