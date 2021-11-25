use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use enum_iterator::IntoEnumIterator;
use libhrstd::libhedron::utcb::UtcbDataException;

/// <https://elixir.bootlin.com/linux/latest/source/arch/x86/um/syscalls_64.c#L15>
#[derive(Debug)]
pub struct ArchPrctlSyscall {
    option: ArchPrctlOption,
    addr: u64,
}

impl From<&GenericLinuxSyscall> for ArchPrctlSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            option: ArchPrctlOption::try_from(syscall.arg0()).unwrap(),
            addr: syscall.arg1(),
        }
    }
}

impl LinuxSyscallImpl for ArchPrctlSyscall {
    fn handle(&self, utcb_exc: &mut UtcbDataException) -> LinuxSyscallResult {
        match self.option {
            ArchPrctlOption::ArchSetGs => utcb_exc.gs.base = self.addr,
            ArchPrctlOption::ArchSetFs => utcb_exc.fs.base = self.addr,
            ArchPrctlOption::ArchGetFs => {
                // TODO write into user address value
            }
            ArchPrctlOption::ArchGetGs => {
                // TODO write into user address value
            }
            ArchPrctlOption::ArchGetCpuid => {
                todo!()
            }
            ArchPrctlOption::ArchSetCpuid => {
                todo!()
            }
            ArchPrctlOption::ArchMapVdsoX32 => {
                todo!()
            }
            ArchPrctlOption::ArchMapVdso32 => {
                todo!()
            }
            ArchPrctlOption::ArchMapVdso64 => {
                todo!()
            }
        }

        LinuxSyscallResult::new_success(0)
    }
}

/// <https://elixir.bootlin.com/linux/latest/source/arch/x86/include/uapi/asm/prctl.h#L6>
#[derive(Clone, Copy, Debug, IntoEnumIterator)]
#[repr(u64)]
pub enum ArchPrctlOption {
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

impl ArchPrctlOption {
    pub fn val(self) -> u64 {
        self as u64
    }
}

impl TryFrom<u64> for ArchPrctlOption {
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
