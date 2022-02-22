use crate::process_mng::process::Process;
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
    _len: u64,
}

impl From<&GenericLinuxSyscall> for MUnMapSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0(),
            _len: syscall.arg1(),
        }
    }
}

impl LinuxSyscallImpl for MUnMapSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        if self.addr % PAGE_SIZE as u64 != 0 {
            log::debug!("Linux app send a not page aligned address. This is with high certainty illegal! How does Linux get that address?! Mappings with mmap should all be page aligned..");
        }
        log::debug!("MUnMap syscall currently doesn't do anything; TODO fix memory leak");
        LinuxSyscallResult::new_success(0)
    }
}
