use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use enum_iterator::IntoEnumIterator;
use libhrstd::libhedron::utcb::UtcbDataException;

/// Implementation of <https://man7.org/linux/man-pages/man2/signalstack.2.html>.
#[derive(Debug)]
pub struct SignalStackSyscall {}

impl From<&GenericLinuxSyscall> for SignalStackSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {}
    }
}

impl LinuxSyscallImpl for SignalStackSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}
