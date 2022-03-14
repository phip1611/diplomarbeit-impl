use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

/// Manipulates file descriptors.
#[derive(Debug)]
pub struct FcntlSyscall {
    // null terminated file name
    _fd: FD,
    _cmd: FcntlCmd,
    _arg: u64,
}

impl From<&GenericLinuxSyscall> for FcntlSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            _fd: FD::new(syscall.arg0() as i32),
            _cmd: FcntlCmd::from(syscall.arg1()),
            _arg: syscall.arg2(),
        }
    }
}

impl LinuxSyscallImpl for FcntlSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // for now it looks like this is enough to make simple
        // Rust programs work
        LinuxSyscallResult::new_success(0 as u64)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
#[allow(unused)]
enum FcntlCmd {
    DupFd = 0,
    GetFd = 1,
    SetFd = 2,
    GetFl = 3,
    SetFl = 4,
    GetLk = 5,
    SetLk = 6,
    SetLkw = 7,
    SetOwn = 8,
    GetOwn = 9,
    SetSig = 10,
    GetLk64 = 12,
    SetLk64 = 13,
    SetLkw64 = 14,
    SetOwnEx = 15,
    GetOwnEx = 16,
    GetOwnerUids = 17,
}

impl FcntlCmd {
    #[allow(unused)]
    pub const fn val(self) -> u64 {
        self as _
    }
}

impl From<u64> for FcntlCmd {
    fn from(val: u64) -> Self {
        if val > 17 {
            panic!("invalid variant");
        }
        let val = unsafe { core::mem::transmute(val) };
        val
    }
}
