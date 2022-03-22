use crate::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;
use core::fmt::Write;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::mem::PageAlignedBuf;

// Nils: for the evaluation I should simulate a more realistic scenario.
// This is that the Linux OS Personality and the FS-Service use an
// shared page-aligned buffer. It should not work like this that the fs service
// gets access to for example the stack or the heap of a Linux app directly
// for security reasons
static mut SIMULATED_WRITE_WINDOW: PageAlignedBuf<u8, 0x100000> =
    PageAlignedBuf::<u8, 0x100000>::new(0);

#[derive(Debug)]
pub struct WriteSyscall {
    fd: u64,
    usr_ptr: *const u8,
    // number of bytes
    count: usize,
}

impl From<&GenericLinuxSyscall> for WriteSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: syscall.arg0(),
            usr_ptr: syscall.arg1() as _,
            count: syscall.arg2() as _,
        }
    }
}

impl WriteSyscall {
    pub(super) fn new(
        fd: u64,
        usr_ptr: *const u8,
        // number of bytes
        count: usize,
    ) -> Self {
        Self { fd, usr_ptr, count }
    }
}

impl LinuxSyscallImpl for WriteSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // either create mapping or re-use if the page is already mapped
        let mapping = MAPPED_AREAS
            .lock()
            .create_or_get_mapping(process, self.usr_ptr as u64, self.count as u64)
            .clone();
        let u_page_offset = self.usr_ptr as usize & 0xfff;
        let u_write_data = mapping.mem_with_offset_as_slice::<u8>(self.count, u_page_offset);

        log::trace!(
            "write: fd={}, u_page_offset={}, count={}, page_count={}",
            self.fd,
            u_page_offset,
            self.count,
            if self.count % PAGE_SIZE == 0 {
                self.count / PAGE_SIZE
            } else {
                (self.count / PAGE_SIZE) + 1
            }
        );

        match self.fd {
            0 => panic!("write to stdin currently not supported"),
            1 | 2 => {
                let r_cstr = core::str::from_utf8(u_write_data).unwrap();
                if self.fd == 1 {
                    crate::services::stdout::writer_mut()
                        .write_str(r_cstr)
                        .unwrap();
                } else {
                    crate::services::stderr::writer_mut()
                        .write_str(r_cstr)
                        .unwrap();
                }

                LinuxSyscallResult::new_success(self.count as u64)
            }
            fd => {
                // simulate: copy to receive/send window
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        u_write_data.as_ptr(),
                        SIMULATED_WRITE_WINDOW.as_mut_ptr(),
                        u_write_data.len(),
                    );
                    let _ = core::ptr::read_volatile(SIMULATED_WRITE_WINDOW.as_ptr());
                }

                let written_bytes = libfileserver::FILESYSTEM
                    .lock()
                    .write_file(process.pid(), (fd as u64).into(), unsafe {
                        &SIMULATED_WRITE_WINDOW[0..u_write_data.len()]
                    })
                    .unwrap();
                LinuxSyscallResult::new_success(written_bytes as u64)
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct LinuxIoVec {
    /// User address.
    u_iov_base: *const u8,
    len: u64,
}
