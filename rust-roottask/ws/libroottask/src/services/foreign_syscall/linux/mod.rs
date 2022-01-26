mod arch_prctl;
mod brk;
mod error_code;
mod generic;
mod ioctl;
mod mmap;
mod poll;
mod rtsigaction;
mod rtsigprocmask;
mod set_tid_address;
mod signalstack;
mod syscall_num;
mod write;
mod write_v;

use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::error_code::LinuxErrorCode;
use core::fmt::Debug;
pub use generic::GenericLinuxSyscall;
use libhrstd::libhedron::UtcbDataException;

pub struct LinuxSyscallResult(i64);

impl LinuxSyscallResult {
    fn new_success(success_value: u64) -> Self {
        assert_eq!(success_value >> 63 & 1, 0, "bit 63 must be negative!");
        Self(success_value as i64)
    }

    fn new_error(error: LinuxErrorCode) -> Self {
        Self(-(error.val() as i64))
    }

    /// Returns the value for the RAX register, which holds the syscall return code.
    pub fn val(self) -> u64 {
        self.0 as _
    }
}

pub trait LinuxSyscallImpl: Debug {
    /// Must make sure, that the handler sets the correct return code in the correct register.
    fn handle(&self, utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult;
}
