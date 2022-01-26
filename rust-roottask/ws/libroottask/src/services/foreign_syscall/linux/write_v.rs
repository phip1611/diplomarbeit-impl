use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Layout;
use core::fmt::Write;
use libhrstd::libhedron::capability::MemCapPermissions;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::utcb::UtcbDataException;
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
    fn handle(&self, _utcb_exc: &mut UtcbDataException, _process: &Process) -> LinuxSyscallResult {
        let mut bytes_written = 0;

        // first: map the iovec itself
        let u_iovec_page_offset = self.usr_ptr as usize & 0xfff;
        let u_iovec_total_len = core::mem::size_of::<LinuxIoVec>() * self.count;
        let r_mapping_size = u_iovec_page_offset + u_iovec_total_len;
        let r_mapping_pages = if r_mapping_size % PAGE_SIZE == 0 {
            r_mapping_size / PAGE_SIZE
        } else {
            (r_mapping_size / PAGE_SIZE) + 1
        };

        let r_iovec_mapping_dest = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(r_mapping_size, PAGE_SIZE).unwrap());
        CrdDelegateOptimizer::new(
            self.usr_ptr as u64 / PAGE_SIZE as u64,
            r_iovec_mapping_dest / PAGE_SIZE as u64,
            r_mapping_pages,
        )
        .mmap(101, 32, MemCapPermissions::READ);

        let r_mapping_begin_ptr = r_iovec_mapping_dest as *const u8;
        let r_iovec_begin_ptr = unsafe {
            r_mapping_begin_ptr
                .add(u_iovec_page_offset)
                .cast::<LinuxIoVec>()
        };
        let r_iovec = unsafe { core::slice::from_raw_parts(r_iovec_begin_ptr, self.count) };
        dbg!(r_iovec);

        // still somehow buggy
        for io_vec_entry in r_iovec {
            let cstr_mapping_dest = VIRT_MEM_ALLOC
                .lock()
                .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());
            CrdDelegateOptimizer::new(
                io_vec_entry.u_iov_base as u64 / PAGE_SIZE as u64,
                cstr_mapping_dest / PAGE_SIZE as u64,
                1,
            )
            .mmap(101, 32, MemCapPermissions::READ);

            let u_page_offset = io_vec_entry.u_iov_base as usize & 0xfff;
            let r_bytes =
                unsafe { core::slice::from_raw_parts(cstr_mapping_dest as *const u8, PAGE_SIZE) };
            let r_cstr_bytes = &r_bytes[u_page_offset..u_page_offset + io_vec_entry.len as usize];
            let r_cstr = unsafe { core::str::from_utf8_unchecked(r_cstr_bytes) };

            bytes_written += io_vec_entry.len;
            dbg!(r_cstr);
            crate::services::stdout::writer_mut()
                .write_str(r_cstr)
                .unwrap();
        }

        dbg!(bytes_written);
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
