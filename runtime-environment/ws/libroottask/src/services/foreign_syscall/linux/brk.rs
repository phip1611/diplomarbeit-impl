use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;

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
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        log::trace!("BRK  in={:?}", self.addr);
        let brk = process
            .memory_manager_mut()
            .increase_break(self.addr as u64, process);
        log::trace!("BRK  out={:?}", brk as *const u8);
        LinuxSyscallResult::new_success(brk)
    }
}
