use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::write::WriteSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use core::alloc::Layout;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::mem::calc_page_count;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

#[derive(Debug)]
pub struct WriteVSyscall {
    fd: u64,
    usr_ptr: *const LinuxIoVec,
    // number of io vecs
    count: usize,
}

impl From<&GenericLinuxSyscall> for WriteVSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: syscall.arg0(),
            usr_ptr: syscall.arg1() as _,
            count: syscall.arg2() as _,
        }
    }
}

impl LinuxSyscallImpl for WriteVSyscall {
    fn handle(
        &self,
        utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // first: map the iovec itself
        let u_iovec_page_offset = self.usr_ptr as usize & 0xfff;
        let u_iovec_total_len = core::mem::size_of::<LinuxIoVec>() * self.count;
        let r_mapping_size = u_iovec_page_offset + u_iovec_total_len;
        let r_mapping_pages = calc_page_count(r_mapping_size);

        let r_iovec_mapping_dest = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(r_mapping_size, PAGE_SIZE).unwrap());
        CrdDelegateOptimizer::new(
            self.usr_ptr as u64 / PAGE_SIZE as u64,
            r_iovec_mapping_dest / PAGE_SIZE as u64,
            r_mapping_pages,
        )
        .mmap(
            process.pd_obj().cap_sel(),
            process.parent().unwrap().pd_obj().cap_sel(),
            MemCapPermissions::READ,
        );

        let r_mapping_begin_ptr = r_iovec_mapping_dest as *const u8;
        let r_iovec_begin_ptr = unsafe {
            r_mapping_begin_ptr
                .add(u_iovec_page_offset)
                .cast::<LinuxIoVec>()
        };
        let r_iovec = unsafe { core::slice::from_raw_parts(r_iovec_begin_ptr, self.count) };

        // I reuse the functionality of the write system call for every IO VEC
        let bytes_written = r_iovec
            .iter()
            .map(|x| WriteSyscall::new(self.fd, x.u_iov_base, x.len as usize))
            .map(|x| x.handle(utcb_exc, process))
            .map(|x| x.val())
            .sum();

        LinuxSyscallResult::new_success(bytes_written)
    }
}

#[derive(Debug)]
#[repr(C)]
struct LinuxIoVec {
    /// User address.
    u_iov_base: *const u8,
    len: u64,
}
