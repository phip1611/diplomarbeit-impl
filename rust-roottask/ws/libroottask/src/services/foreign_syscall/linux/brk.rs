use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use enum_iterator::IntoEnumIterator;
use libhrstd::libhedron::utcb::UtcbDataException;

/// Implementation of <https://man7.org/linux/man-pages/man2/brk.2.html>.
#[derive(Debug)]
pub struct BrkSyscall {
    addr: *const u8,
}

impl From<&GenericLinuxSyscall> for BrkSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0() as *const _,
        }
    }
}

impl LinuxSyscallImpl for BrkSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}
