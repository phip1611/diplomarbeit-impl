use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;

use core::cmp::min;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

#[derive(Debug)]
pub struct ReadSyscall {
    fd: FD,
    user_buf: *const u8,
    count: usize,
}

impl From<&GenericLinuxSyscall> for ReadSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FD::new(syscall.arg0() as i32),
            user_buf: syscall.arg1() as *const _,
            count: syscall.arg2() as usize,
        }
    }
}

impl LinuxSyscallImpl for ReadSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        let data = libfileserver::fs_read(process.pid(), self.fd, self.count).unwrap();

        let mapping = MAPPED_AREAS
            .lock()
            .create_get_mapping(process, self.user_buf as u64, self.count as u64)
            .clone();

        let bytes_read = min(self.count, data.len());

        unsafe {
            core::ptr::copy(data.as_ptr(), mapping.begin_ptr_mut(), bytes_read);
        }

        LinuxSyscallResult::new_success(bytes_read as u64)
    }
}
