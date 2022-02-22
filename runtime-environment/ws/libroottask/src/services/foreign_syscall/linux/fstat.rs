use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;
use core::mem::size_of;
use libfileserver::FileStat;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

#[derive(Debug)]
pub struct FstatSyscall {
    fd: FD,
    u_ptr_statbuf: u64,
}

impl From<&GenericLinuxSyscall> for FstatSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FD::new(syscall.arg0() as i32),
            u_ptr_statbuf: syscall.arg1(),
        }
    }
}

impl LinuxSyscallImpl for FstatSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        let fstat = libfileserver::fs_fstat(process.pid(), self.fd).unwrap();

        let u_page_offset = self.u_ptr_statbuf & 0xfff;
        let mut mapping = MAPPED_AREAS
            .lock()
            .create_get_mapping(process, self.u_ptr_statbuf, size_of::<FileStat>() as u64)
            .clone();

        let r_write_ptr = mapping.mem_with_offset_as_ptr_mut(u_page_offset as usize);
        unsafe {
            core::ptr::write(r_write_ptr as *mut _, fstat);
        }

        LinuxSyscallResult::new_success(0)
    }
}
