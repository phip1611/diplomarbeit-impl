use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Layout;
use core::mem::size_of;
use libfileserver::FileStat;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::{
    MemCapPermissions,
    UtcbDataException,
};
use libhrstd::rt::services::fs::FD;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        let fstat = libfileserver::fs_fstat(process.pid(), self.fd).unwrap();

        let u_page_offset = self.u_ptr_statbuf & 0xfff;
        let mapping_bytes = u_page_offset + size_of::<FileStat>() as u64;

        let page_count = if mapping_bytes % PAGE_SIZE as u64 == 0 {
            mapping_bytes / PAGE_SIZE as u64
        } else {
            (mapping_bytes / PAGE_SIZE as u64) + 1
        };

        let r_mapping = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(mapping_bytes as usize, PAGE_SIZE).unwrap());

        let u_page_num = self.u_ptr_statbuf / PAGE_SIZE as u64;
        let r_page_num = r_mapping / PAGE_SIZE as u64;

        CrdDelegateOptimizer::new(u_page_num, r_page_num, page_count as usize).mmap(
            process.pd_obj().cap_sel(),
            process.parent().unwrap().pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE,
        );

        let r_stat_ptr = r_mapping + u_page_offset;
        unsafe {
            core::ptr::write(r_stat_ptr as *mut _, fstat);
        }

        LinuxSyscallResult::new_success(0)
    }
}
