use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;

/// * <https://man7.org/linux/man-pages/man2/set_tid_address.2.html>
/// * <https://github.com/torvalds/linux/blob/master/kernel/fork.c#L1718>
///
/// For each thread, the kernel maintains two attributes (addresses)
/// called set_child_tid and clear_child_tid.  These two attributes
/// contain the value NULL by default.
///
/// * `set_child_tid`: \
///   If a thread is started using clone(2) with the
///   CLONE_CHILD_SETTID flag, set_child_tid is set to the value
///   passed in the ctid argument of that system call.
///
///   When set_child_tid is set, the very first thing the new
///   thread does is to write its thread ID at this address.
///
/// * `clear_child_tid`: \
///    If a thread is started using clone(2) with the
///    CLONE_CHILD_CLEARTID flag, clear_child_tid is set to the
///    value passed in the ctid argument of that system call.
///
///    The system call set_tid_address() sets the clear_child_tid value
///    for the calling thread to tidptr.
#[derive(Debug)]
#[allow(unused)]
pub struct SetTidAddressSyscall {
    tid_ptr: *const u8,
}

impl From<&GenericLinuxSyscall> for SetTidAddressSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            tid_ptr: syscall.arg0() as *const u8,
        }
    }
}

impl LinuxSyscallImpl for SetTidAddressSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        // this syscall always succeeds and returns always returns the caller's thread ID
        LinuxSyscallResult::new_success(0)
    }
}
