use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;
use core::cmp::min;
use libfileserver::FileDescriptor;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::mem::PageAlignedBuf;

// Nils: for the evaluation I should simulate a more realistic scenario.
// This is that the Linux OS Personality and the FS-Service use an
// shared page-aligned buffer. It should not work like this that the fs service
// gets access to for example the stack or the heap of a Linux app directly
// for security reasons
static mut SIMULATED_READ_WINDOW: PageAlignedBuf<u8, 0x100000> =
    PageAlignedBuf::<u8, 0x100000>::new(0);

#[derive(Debug)]
pub struct ReadSyscall {
    fd: FileDescriptor,
    user_buf: *mut u8,
    count: usize,
}

impl From<&GenericLinuxSyscall> for ReadSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FileDescriptor::new(syscall.arg0()),
            user_buf: syscall.arg1() as *mut _,
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
        let mut fs_lock = libfileserver::FILESYSTEM.lock();
        let data = fs_lock
            .read_file(process.pid(), self.fd, self.count)
            .unwrap();

        let bytes_read = min(self.count, data.len());

        // simulate: copy to receive/send window
        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                SIMULATED_READ_WINDOW.as_mut_ptr(),
                bytes_read,
            );
            let _ = core::ptr::read_volatile(SIMULATED_READ_WINDOW.as_ptr());
        }

        let mapping = MAPPED_AREAS
            .lock()
            .create_or_get_mapping(process, self.user_buf as u64, bytes_read as u64)
            .clone();

        let r_write_ptr = mapping.old_to_new_ptr_mut(self.user_buf);

        unsafe {
            core::ptr::copy(SIMULATED_READ_WINDOW.as_ptr(), r_write_ptr, bytes_read);
        }

        LinuxSyscallResult::new_success(bytes_read as u64)
    }
}
