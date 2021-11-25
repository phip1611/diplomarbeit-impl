use crate::services::foreign_syscall::linux::arch_prctl::ArchPrctlSyscall;
use crate::services::foreign_syscall::linux::syscall_num::LinuxSyscallNum;
use crate::services::foreign_syscall::linux::LinuxSyscallImpl;
use alloc::boxed::Box;
use libhrstd::libhedron::utcb::UtcbDataException;

/// Generic Syscall. Bindings from registers
/// to argument number. See <https://github.com/torvalds/linux/blob/35776f10513c0d523c5dd2f1b415f642497779e2/arch/x86/entry/entry_64.S>
#[derive(Debug)]
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

    pub fn handle(&self, utcb_exc: &mut UtcbDataException) {
        let syscall_impl: Box<dyn LinuxSyscallImpl> = match self.rax {
            LinuxSyscallNum::Read => todo!(),
            LinuxSyscallNum::Write => todo!(),
            LinuxSyscallNum::Open => todo!(),
            LinuxSyscallNum::Close => todo!(),
            LinuxSyscallNum::Poll => todo!(),
            LinuxSyscallNum::MMap => todo!(),
            LinuxSyscallNum::MProtect => todo!(),
            LinuxSyscallNum::MUnmap => todo!(),
            LinuxSyscallNum::Brk => todo!(),
            LinuxSyscallNum::RtSigaction => todo!(),
            LinuxSyscallNum::RtSigprocmask => todo!(),
            LinuxSyscallNum::Ioctl => todo!(),
            LinuxSyscallNum::WriteV => todo!(),
            LinuxSyscallNum::Clone => todo!(),
            LinuxSyscallNum::Fcntl => todo!(),
            LinuxSyscallNum::SigAltStack => todo!(),
            LinuxSyscallNum::ArchPrctl => Box::new(ArchPrctlSyscall::try_from(self).unwrap()),
            LinuxSyscallNum::Gettid => todo!(),
            LinuxSyscallNum::Futex => todo!(),
            LinuxSyscallNum::SchedGetAffinity => todo!(),
            LinuxSyscallNum::SetTidAddress => todo!(),
            LinuxSyscallNum::ExitGroup => todo!(),
            LinuxSyscallNum::ReadLinkAt => todo!(),
            LinuxSyscallNum::PrLimit64 => todo!(),
        };
        log::debug!("Linux syscall: {:?}", syscall_impl);
        utcb_exc.rax = syscall_impl.handle(utcb_exc).val();

        // syscall implementations may not change these values

        // see x86 spec:
        utcb_exc.rip = utcb_exc.rcx;
        // hedron saves user sp in r11
        utcb_exc.rsp = utcb_exc.r11;
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
