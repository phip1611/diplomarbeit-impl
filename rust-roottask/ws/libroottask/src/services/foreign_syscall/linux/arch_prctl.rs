use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use enum_iterator::IntoEnumIterator;
use libhrstd::libhedron::utcb::UtcbDataException;

/// * <https://man7.org/linux/man-pages/man2/arch_prctl.2.html>
/// * <https://elixir.bootlin.com/linux/latest/source/arch/x86/um/syscalls_64.c#L15>
///
/// arch_prctl() sets architecture-specific process or thread state.
/// code selects a subfunction and passes argument addr to it; addr
/// is interpreted as either an unsigned long for the "set"
/// operations, or as an unsigned long *, for the "get" operations.
#[derive(Debug)]
pub struct ArchPrctlSyscall {
    subfunction: ArchPrctlSubfunction,
    /// integer for the set operations or pointer for get operations.
    addr: *const u8,
}

impl From<&GenericLinuxSyscall> for ArchPrctlSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            subfunction: ArchPrctlSubfunction::try_from(syscall.arg0()).unwrap(),
            addr: syscall.arg1() as _,
        }
    }
}

impl LinuxSyscallImpl for ArchPrctlSyscall {
    fn handle(&self, utcb_exc: &mut UtcbDataException) -> LinuxSyscallResult {
        /*match self.subfunction {
            ArchPrctlSubfunction::ArchSetGs => utcb_exc.gs.base = self.addr as _,
            ArchPrctlSubfunction::ArchSetFs => utcb_exc.fs.base = self.addr as _,
            ArchPrctlSubfunction::ArchGetFs => {
                // TODO write into user address
            }
            ArchPrctlSubfunction::ArchGetGs => {
                // TODO write into user address
            }
            ArchPrctlSubfunction::ArchGetCpuid => {
                todo!()
            }
            ArchPrctlSubfunction::ArchSetCpuid => {
                todo!()
            }
            ArchPrctlSubfunction::ArchMapVdsoX32 => {
                todo!()
            }
            ArchPrctlSubfunction::ArchMapVdso32 => {
                todo!()
            }
            ArchPrctlSubfunction::ArchMapVdso64 => {
                todo!()
            }
        }*/

        LinuxSyscallResult::new_success(0)
    }
}

/// <https://elixir.bootlin.com/linux/latest/source/arch/x86/include/uapi/asm/prctl.h#L6>
#[derive(Clone, Copy, Debug, IntoEnumIterator)]
#[repr(u64)]
pub enum ArchPrctlSubfunction {
    ArchSetGs = 0x1001,
    ArchSetFs = 0x1002,
    ArchGetFs = 0x1003,
    ArchGetGs = 0x1004,

    ArchGetCpuid = 0x1011,
    ArchSetCpuid = 0x1012,

    ArchMapVdsoX32 = 0x2001,
    ArchMapVdso32 = 0x2002,
    ArchMapVdso64 = 0x2003,
}

impl ArchPrctlSubfunction {
    pub fn val(self) -> u64 {
        self as u64
    }
}

impl TryFrom<u64> for ArchPrctlSubfunction {
    type Error = ();
    fn try_from(val: u64) -> Result<Self, ()> {
        // generated during compile time; probably not recognized by IDE
        for variant in Self::into_enum_iter() {
            if variant.val() == val {
                return Ok(variant);
            }
        }
        Err(())
    }
}
