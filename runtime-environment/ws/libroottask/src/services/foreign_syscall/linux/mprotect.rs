use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::{
    GenericLinuxSyscall,
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::UtcbDataException;

/// set protection on a region of memory
#[derive(Debug)]
pub struct MProtectSyscall {
    addr: u64,
    _len: u64,
    _prot: MProtect,
}

impl From<&GenericLinuxSyscall> for MProtectSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0(),
            _len: syscall.arg1(),
            _prot: MProtect::from_bits(syscall.arg2()).unwrap(),
        }
    }
}

impl LinuxSyscallImpl for MProtectSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        log::info!("MProtect: {:#?}", self);
        log::debug!("MUnMap syscall currently doesn't do anything; TODO fix memory leak");
        LinuxSyscallResult::new_success(0)
    }
}

bitflags::bitflags! {
    #[allow(unused)]
    struct MProtect: u64 {
        const None = 0x0;
        const Read = 0x1;
        const Write = 0x2;
        const Exec = 0x4;
        const Sem = 0x8;
        const GrowsUp = 0x2000000;
        const GrowsDown = 0x1000000;
    }
}
