use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Allocator;
use core::alloc::Layout;
use core::ptr::null;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// Implementation of <https://man7.org/linux/man-pages/man2/brk.2.html>.
#[derive(Debug)]
pub struct BrkSyscall {
    addr: *const u8,
}

impl From<&GenericLinuxSyscall> for BrkSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0() as *const _,
        }
    }
}

impl LinuxSyscallImpl for BrkSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        // do nothing; it's okay for simple Linux programs
        if self.addr == null() {
            LinuxSyscallResult::new_success(process.heap_ptr().load(Ordering::SeqCst))
        } else {
            let size = self.addr as u64 - process.heap_ptr().load(Ordering::SeqCst);

            // ensure that we only map whole pages
            let layout = Layout::from_size_align(size as usize, PAGE_SIZE).unwrap();

            let ptr: NonNull<[u8]> = alloc::alloc::Global.allocate_zeroed(layout).unwrap();

            // map the page directly,
            let page_num = ptr.as_ptr() as *const u8 as usize / PAGE_SIZE;

            // map the right amount of pages
            let page_count = if layout.size() % PAGE_SIZE == 0 {
                layout.size() / PAGE_SIZE
            } else {
                (layout.size() / PAGE_SIZE) + 1
            };

            CrdDelegateOptimizer::new(
                page_num as u64,
                process.heap_ptr().load(Ordering::SeqCst) / PAGE_SIZE as u64,
                page_count,
            )
            .mmap(
                process.parent().unwrap().pd_obj().cap_sel(),
                process.pd_obj().cap_sel(),
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            );

            // update heap pointer/program break in process
            process.heap_ptr().store(
                process.heap_ptr().load(Ordering::SeqCst) + page_count as u64 * PAGE_SIZE as u64,
                Ordering::SeqCst,
            );

            log::debug!(
                "old brk: {:?}, new_brk: {:?}",
                (self.addr as u64 - size) as *const u8,
                process.heap_ptr().load(Ordering::SeqCst) as *const u8
            );

            LinuxSyscallResult::new_success(self.addr as u64)
        }
    }
}
