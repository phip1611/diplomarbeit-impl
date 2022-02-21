use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;

/// Implementation of <https://man7.org/linux/man-pages/man2/sigprocmask.2.html>.
#[derive(Debug)]
pub struct RtSigProcMaskSyscall {}

impl From<&GenericLinuxSyscall> for RtSigProcMaskSyscall {
    fn from(_syscall: &GenericLinuxSyscall) -> Self {
        Self {}
    }
}

impl LinuxSyscallImpl for RtSigProcMaskSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}
