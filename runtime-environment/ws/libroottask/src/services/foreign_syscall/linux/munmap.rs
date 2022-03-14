use crate::process::Process;
use crate::services::foreign_syscall::linux::{
    GenericLinuxSyscall,
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
pub struct MUnMapSyscall {
    addr: u64,
    len: u64,
}

impl From<&GenericLinuxSyscall> for MUnMapSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0(),
            len: syscall.arg1(),
        }
    }
}

impl LinuxSyscallImpl for MUnMapSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        log::trace!(
            "munmap: addr={:?}, len={}",
            self.addr as *const u8,
            self.len
        );
        if self.addr % PAGE_SIZE as u64 != 0 {
            log::debug!("Linux app did not send page aligned address. This is with high certainty illegal! How does Linux get that address?! Mappings with mmap should all be page aligned..");
        }
        process.memory_manager_mut().munmap(self.addr, process);
        LinuxSyscallResult::new_success(0)
    }
}
