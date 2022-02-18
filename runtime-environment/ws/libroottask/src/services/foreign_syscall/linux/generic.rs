use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::arch_prctl::ArchPrctlSyscall;
use crate::services::foreign_syscall::linux::brk::BrkSyscall;
use crate::services::foreign_syscall::linux::close::CloseSyscall;
use crate::services::foreign_syscall::linux::fcntl::FcntlSyscall;
use crate::services::foreign_syscall::linux::fstat::FstatSyscall;
use crate::services::foreign_syscall::linux::ioctl::IoctlSyscall;
use crate::services::foreign_syscall::linux::lseek::LSeekSyscall;
use crate::services::foreign_syscall::linux::mmap::MMapSyscall;
use crate::services::foreign_syscall::linux::open::OpenSyscall;
use crate::services::foreign_syscall::linux::poll::PollSyscall;
use crate::services::foreign_syscall::linux::read::ReadSyscall;
use crate::services::foreign_syscall::linux::rtsigaction::RtSigactionSyscall;
use crate::services::foreign_syscall::linux::rtsigprocmask::RtSigProcMaskSyscall;
use crate::services::foreign_syscall::linux::set_tid_address::SetTidAddressSyscall;
use crate::services::foreign_syscall::linux::signalstack::SignalStackSyscall;
use crate::services::foreign_syscall::linux::syscall_num::LinuxSyscallNum;
use crate::services::foreign_syscall::linux::write::WriteSyscall;
use crate::services::foreign_syscall::linux::write_v::WriteVSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use core::fmt::Debug;
use libhrstd::libhedron::ipc_serde::__private::Formatter;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::UtcbDataException;

/// Generic Syscall. Bindings from registers
/// to argument number. See <https://github.com/torvalds/linux/blob/35776f10513c0d523c5dd2f1b415f642497779e2/arch/x86/entry/entry_64.S>
// #[derive(Debug)]
pub struct GenericLinuxSyscall {
    rax: LinuxSyscallNum,
    rdi_arg0: u64,
    rsi_arg1: u64,
    rdx_arg2: u64,
    r10_arg3: u64,
    r8_arg4: u64,
    r9_arg5: u64,
}

impl GenericLinuxSyscall {
    pub fn syscall_num(&self) -> LinuxSyscallNum {
        self.rax
    }
    pub fn arg0(&self) -> u64 {
        self.rdi_arg0
    }
    pub fn arg1(&self) -> u64 {
        self.rsi_arg1
    }
    pub fn arg2(&self) -> u64 {
        self.rdx_arg2
    }
    pub fn arg3(&self) -> u64 {
        self.r10_arg3
    }
    pub fn arg4(&self) -> u64 {
        self.r8_arg4
    }
    pub fn arg5(&self) -> u64 {
        self.r9_arg5
    }

    pub fn handle(&self, utcb_exc: &mut UtcbDataException, process: &Process) {
        // all Linux syscalls put their result in RAX => save general purpose registers
        utcb_exc.mtd |= Mtd::GPR_ACDB;

        #[rustfmt::skip]
        let res: LinuxSyscallResult = match self.rax {
            LinuxSyscallNum::Read => ReadSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Write => WriteSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Open => OpenSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Close => CloseSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Fstat => FstatSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Poll => PollSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::LSeek => LSeekSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::MMap => MMapSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::MProtect => todo!("LinuxSyscallNum::MProtect"),
            LinuxSyscallNum::MUnmap => todo!("LinuxSyscallNum::MUnmap"),
            LinuxSyscallNum::Brk => BrkSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::RtSigaction => RtSigactionSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::RtSigprocmask => RtSigProcMaskSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Ioctl => IoctlSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::WriteV => WriteVSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Clone => todo!("LinuxSyscallNum::Clone"),
            LinuxSyscallNum::Fcntl => FcntlSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::SigAltStack => SignalStackSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::ArchPrctl => ArchPrctlSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::Gettid => todo!("LinuxSyscallNum::Gettid"),
            LinuxSyscallNum::Futex => todo!("LinuxSyscallNum::Futex"),
            LinuxSyscallNum::SchedGetAffinity => todo!("LinuxSyscallNum::SchedGetAffinity"),
            LinuxSyscallNum::SetTidAddress => SetTidAddressSyscall::from(self).handle(utcb_exc, process),
            LinuxSyscallNum::ExitGroup => todo!("LinuxSyscallNum::ExitGroup"),
            LinuxSyscallNum::ReadLinkAt => todo!("LinuxSyscallNum::ReadLinkAt"),
            LinuxSyscallNum::PrLimit64 => todo!("LinuxSyscallNum::PrLimit64"),
        };
        utcb_exc.rax = res.val();
    }
}

impl Debug for GenericLinuxSyscall {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GenericLinuxSyscall")
            .field("rax", &self.rax)
            .field("rdi_arg0", &(self.rdi_arg0 as *const u8))
            .field("rsi_arg1", &(self.rsi_arg1 as *const u8))
            .field("rdx_arg2", &(self.rdx_arg2 as *const u8))
            .field("r10_arg3", &(self.r10_arg3 as *const u8))
            .field("r8_arg4", &(self.r8_arg4 as *const u8))
            .field("r9_arg5", &(self.r9_arg5 as *const u8))
            .finish()
    }
}

impl TryFrom<&UtcbDataException> for GenericLinuxSyscall {
    type Error = ();
    fn try_from(exc: &UtcbDataException) -> Result<Self, Self::Error> {
        let syscall_num = LinuxSyscallNum::try_from(exc.rax);
        if syscall_num.is_err() {
            log::debug!("unsupported syscall num: {}", exc.rax);
        }
        Ok(Self {
            rax: syscall_num?,
            rdi_arg0: exc.rdi,
            rsi_arg1: exc.rsi,
            rdx_arg2: exc.rdx,
            r10_arg3: exc.r10,
            r8_arg4: exc.r8,
            r9_arg5: exc.r9,
        })
    }
}
