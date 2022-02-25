use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use core::mem::size_of;
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
pub struct SysinfoSyscall {
    sysinfo: *mut sysinfo,
}

impl From<&GenericLinuxSyscall> for SysinfoSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            sysinfo: syscall.arg0() as *mut _,
        }
    }
}

impl LinuxSyscallImpl for SysinfoSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        unsafe { core::ptr::write_bytes(self.sysinfo.cast::<u8>(), 0, size_of::<sysinfo>()) };
        LinuxSyscallResult::new_success(0)
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct sysinfo {
    /// Seconds since boot
    uptime: usize,
    /// 1, 5, and 15 minute load averages
    loads: [usize; 3],
    /// Total usable main memory size
    totalram: usize,
    /// Available memory size
    freeram: usize,
    /// Amount of shared memory
    sharedram: usize,
    /// Memory used by buffers
    bufferram: usize,
    /// Total swap space size
    totalswap: usize,
    /// Swap space still available
    freeswap: usize,
    /// Number of current processes
    procs: u16,
    /// Pads structure to 64 bytes
    _pad: [u8; 22],
}
