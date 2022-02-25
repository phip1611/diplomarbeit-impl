use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::process::consts::ProcessId;

#[derive(Debug)]
pub struct SchedGetAffinitySyscall {
    _pid: ProcessId,
    _len: usize,
    user_mask_ptr: *mut u64,
}

impl From<&GenericLinuxSyscall> for SchedGetAffinitySyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            _pid: syscall.arg0(),
            _len: syscall.arg1() as usize,
            user_mask_ptr: syscall.arg2() as *mut _,
        }
    }
}

impl LinuxSyscallImpl for SchedGetAffinitySyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        unsafe { core::ptr::write(self.user_mask_ptr, !0) };
        LinuxSyscallResult::new_success(0)
    }
}
