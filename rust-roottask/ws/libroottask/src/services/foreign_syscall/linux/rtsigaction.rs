use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use enum_iterator::IntoEnumIterator;
use libhrstd::libhedron::utcb::UtcbDataException;

/// Implementation of <https://man7.org/linux/man-pages/man2/sigaction.2.html>.
#[derive(Debug)]
pub struct RtSigactionSyscall {
    signum: u64,
    new_action: *const Sigaction,
    old_action: *const Sigaction,
}

impl From<&GenericLinuxSyscall> for RtSigactionSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            signum: syscall.arg0(),
            new_action: syscall.arg1() as *const _,
            old_action: syscall.arg2() as *const _,
        }
    }
}

impl LinuxSyscallImpl for RtSigactionSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}

#[repr(C)]
struct Sigaction {
    sa_handler: *const u8,
    sig_mask: usize,
    flags: u32,
    sa_sigaction: *const u8,
    sa_restorer: *const u8,
}
