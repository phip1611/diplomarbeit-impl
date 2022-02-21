use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
pub struct MAdviseSyscall {
    _addr: *const u8,
    _length: u64,
    _advice: MAdvise,
}

impl From<&GenericLinuxSyscall> for MAdviseSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            _addr: syscall.arg0() as *const _,
            _length: syscall.arg1(),
            _advice: unsafe { core::mem::transmute(syscall.arg2()) },
        }
    }
}

impl LinuxSyscallImpl for MAdviseSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // log::info!("{:#?}", self);
        LinuxSyscallResult::new_success(0)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
#[allow(unused)]
pub enum MAdvise {
    /// no further special treatment
    Normal = 0,
    /// expect random page references
    Random = 1,
    /// expect sequential page references
    Sequential = 2,
    ///  will need these pages
    WillNeed = 3,
    /// don't need these pages
    DontNeed = 4,
    /// free pages only if memory pressure
    Free = 8,
    /// remove these pages & resources
    Remove = 9,
    /// don't inherit across fork
    DontFork = 10,
    /// do inherit across fork
    DoFork = 11,
    ///  poison a page for testing
    HwPoison = 100,
    /// soft offline page for testing
    SoftOffline = 101,
    /// KSM may merge identical pages
    Mergeable = 12,
    /// KSM may not merge identical pages
    UnMergeable = 13,
    /// Worth backing with hugepages
    HugePage = 14,
    /// Not worth backing with hugepages
    NoHugePage = 15,
    /// Explicity exclude from the core dump, overrides the coredump filter bits
    DontDump = 16,
    /// Clear the MADV_DONTDUMP flag.
    DoDump = 17,
    /// Zero memory on fork, child only
    WipeOnFork = 18,
    /// Undo WIPEONFORK
    KeepOnFork = 19,
    /// deactivate these pages
    Cold = 20,
    /// reclaim these pages
    PageOut = 21,
    /// populate (prefault) page tables readable
    PopulateRead = 22,
    /// populate (prefault) page tables writable
    PopulateWrite = 23,
}

impl MAdvise {
    pub const fn val(self) -> u64 {
        self as _
    }
}
