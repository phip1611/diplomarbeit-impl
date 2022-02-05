use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Layout;
use core::cmp::min;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::{
    MemCapPermissions,
    UtcbDataException,
};
use libhrstd::rt::services::fs::FD;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        let data = libfileserver::fs_read(process.pid(), self.fd, self.count).unwrap();

        if data.is_empty() {
            return LinuxSyscallResult::new_success(0);
        }

        // roottask mapping destination
        let r_mapping = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(self.count, PAGE_SIZE).unwrap());

        let u_page_offset = self.user_buf as usize & 0xfff;
        let byte_amount = u_page_offset + self.count;
        let page_count = if byte_amount % PAGE_SIZE == 0 {
            byte_amount / PAGE_SIZE + 1
        } else {
            (byte_amount / PAGE_SIZE) + 1
        };
        CrdDelegateOptimizer::new(
            self.user_buf as u64 / PAGE_SIZE as u64,
            r_mapping / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            // from calling process
            process.pd_obj().cap_sel(),
            // to roottask
            process.parent().unwrap().pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE,
        );

        let bytes_written = min(self.count, data.len());

        unsafe {
            let w_ptr = (r_mapping + u_page_offset as u64) as *mut u8;
            core::ptr::copy(data.as_ptr(), w_ptr, bytes_written);
        }

        LinuxSyscallResult::new_success(bytes_written as u64)
    }
}
