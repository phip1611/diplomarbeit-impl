use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use core::mem::size_of;
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
pub struct ClockGetTimeSyscall {
    _clk_id: ClockId,
    timespec: *mut timespec,
}

impl From<&GenericLinuxSyscall> for ClockGetTimeSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            _clk_id: unsafe { core::mem::transmute(syscall.arg0()) },
            timespec: syscall.arg1() as *mut _,
        }
    }
}

impl LinuxSyscallImpl for ClockGetTimeSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        log::trace!("ClockGetTime: {:?}", self);
        unsafe { core::ptr::write_bytes(self.timespec.cast::<u8>(), 0, size_of::<timespec>()) };
        LinuxSyscallResult::new_success(0)
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct timespec {
    /// seconds
    tv_sec: usize,
    /// nanoseconds
    tv_nsec: u64,
}

#[allow(unused)]
#[repr(u64)]
#[derive(Debug)]
enum ClockId {
    Realtime = 0,
    Monotonic = 1,
    ProcessCpuTimeId = 2,
    ThreadCpuTimeId = 3,
    MonotonicRaw = 4,
    RealtimeCoarse = 5,
    MonotonicCoarse = 6,
    Boottime = 7,
    Realtimealarm = 8,
    BoottimeAlarm = 9,
}
