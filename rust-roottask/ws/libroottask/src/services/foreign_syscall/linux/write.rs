use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Layout;
use core::fmt::Write;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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

impl LinuxSyscallImpl for WriteSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        let cstr_mapping_dest = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(self.count, PAGE_SIZE).unwrap());

        let u_page_offset = self.usr_ptr as usize & 0xfff;
        let byte_amount = u_page_offset + self.count;
        let page_count = if byte_amount % PAGE_SIZE == 0 {
            byte_amount / PAGE_SIZE + 1
        } else {
            (byte_amount / PAGE_SIZE) + 1
        };
        CrdDelegateOptimizer::new(
            self.usr_ptr as u64 / PAGE_SIZE as u64,
            cstr_mapping_dest / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            process.pd_obj().cap_sel(),
            process.parent().unwrap().pd_obj().cap_sel(),
            MemCapPermissions::READ,
        );

        let r_bytes = unsafe {
            core::slice::from_raw_parts(cstr_mapping_dest as *const u8, PAGE_SIZE * page_count)
        };
        let r_cstr_bytes = &r_bytes[u_page_offset..u_page_offset + self.count];
        let r_cstr = unsafe { core::str::from_utf8_unchecked(r_cstr_bytes) };

        crate::services::stdout::writer_mut()
            .write_str(r_cstr)
            .unwrap();

        LinuxSyscallResult::new_success(self.count as u64)
    }
}

#[derive(Debug)]
#[repr(C)]
struct LinuxIoVec {
    /// User address.
    u_iov_base: *const u8,
    len: u64,
}
