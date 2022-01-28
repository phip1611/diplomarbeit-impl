use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::error_code::LinuxErrorCode;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::alloc::Layout;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// * <https://man7.org/linux/man-pages/man2/mmap.2.html>
#[derive(Debug)]
#[allow(unused)]
pub struct MMapSyscall {
    addr: *const u8,
    len: u64,
    prot: MMapProt,
    flags: MMapFlags,
    fd: u64,
    offset: u64,
}

impl From<&GenericLinuxSyscall> for MMapSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            addr: syscall.arg0() as _,
            len: syscall.arg1(),
            prot: MMapProt::from_bits(syscall.arg2()).unwrap(),
            flags: MMapFlags::from_bits(syscall.arg3()).unwrap(),
            fd: syscall.arg4(),
            offset: syscall.arg5(),
        }
    }
}

impl LinuxSyscallImpl for MMapSyscall {
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        // two most popular combinations
        let mut ptr = None;
        if self.flags.contains(MMapFlags::ANONYMOUS) && self.flags.contains(MMapFlags::PRIVATE) {
            let layout = Layout::from_size_align(self.len as usize, PAGE_SIZE).unwrap();
            ptr.replace(unsafe { alloc::alloc::alloc_zeroed(layout) } as u64);
        } else if self.flags.contains(MMapFlags::ANONYMOUS)
            && self.flags.contains(MMapFlags::SHARED)
        {
            // TODO what to do different?
            let layout = Layout::from_size_align(self.len as usize, PAGE_SIZE).unwrap();
            ptr.replace(unsafe { alloc::alloc::alloc_zeroed(layout) } as u64);
        } else {
            todo!("unimplemented for flag combination: {:?}", self.flags);
        }

        // TODO keep track of memory in process; no dealloc so far

        let src_page_num = ptr.unwrap() as usize / PAGE_SIZE;
        // TODO look into process object to see where the heap
        //  ptr is and don't map to static location
        let dest_page_num = 0x1234567;

        let page_num = if self.len as usize % PAGE_SIZE == 0 {
            self.len as usize / PAGE_SIZE
        } else {
            (self.len as usize / PAGE_SIZE) + 1
        };

        // map into PD
        CrdDelegateOptimizer::new(src_page_num as u64, dest_page_num, page_num).mmap(
            process.parent().unwrap().pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE,
        );

        // ptr: roottask mem address
        ptr.map(|_x| 0x1234567000)
            .map(LinuxSyscallResult::new_success)
            .unwrap_or(LinuxSyscallResult::new_error(LinuxErrorCode::ENOMEM))
    }
}

bitflags::bitflags! {
    /// Don't know why iti s called PROT but it describes the permissions.
    /// <https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/mman-common.h#L12>
    struct MMapProt: u64 {
        /// page can be read
        const READ = 0x1;
        /// page can be written
        const WRITE = 0x2;
        /// page can be executed
        const EXEC = 0x4;
    }
}

bitflags::bitflags! {
    /// * <https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/mman-common.h#L12>
    /// * <https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/mman-common.h#L22>
    struct MMapFlags: u64 {
        const SHARED = 0x1;
        const PRIVATE = 0x2;
        const SHARED_VALIDATE = 0x3;
        /*/// Put the mapping into the first 2 Gigabytes of the process
        /// address space.  This flag is supported only on x86-64, for
        /// 64-bit programs.  It was added to allow thread stacks to
        /// be allocated somewhere in the first 2 GB of memory, so as
        /// to improve context-switch performance on some early 64-bit
        /// processors.  Modern x86-64 processors no longer have this
        /// performance problem, so use of this flag is not required
        /// on those systems.  The MAP_32BIT flag is ignored when
        /// MAP_FIXED is set.
        Map32Bit = 0x40,*/
        /*/// Synonym for [`Self::Anonymous`] for legacy reasons.
        Anon,*/
        const ANONYMOUS = 0x20;
        /*/// Old, nut used anymore/removed.
        DenyWrite,
        /// Ignored, legacy.
        Executable,
        /// Ignored, legacy.
        File,*/
        /// Don't interpret addr as a hint but place the mapping
        /// at exactly that address.
        const FIXED = 0x10;
        /*FixedNoReplace,
        GrowsDown,
        HugeTlb,
        Huge2MB,
        Huge1GB,
        Locked,
        Nonblock,
        NoReserve,
        Populate,
        Stack,
        Sync,
        Uninitialized,*/
    }
}
