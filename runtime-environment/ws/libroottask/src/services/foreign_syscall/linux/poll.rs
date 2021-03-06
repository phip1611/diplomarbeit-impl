use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;

/// Implementation of <https://man7.org/linux/man-pages/man2/poll.2.html>.
#[derive(Debug)]
#[allow(unused)]
pub struct PollSyscall {
    fds: *const *const PollFd,
    count: usize,
}

impl From<&GenericLinuxSyscall> for PollSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fds: syscall.arg0() as *const _,
            count: syscall.arg1() as usize,
        }
    }
}

impl LinuxSyscallImpl for PollSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs

        LinuxSyscallResult::new_success(0)
    }
}

#[repr(C)]
struct PollFd {
    /* file descriptor */
    fd: u32,
    /* requested events */
    events: u16,
    /* returned events */
    revents: u16,
}
