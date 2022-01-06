use crate::mem::{
    VirtMemAllocator,
    VIRT_MEM_ALLOC,
};
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::arch_prctl::ArchPrctlSyscall;
use crate::services::foreign_syscall::linux::brk::BrkSyscall;
use crate::services::foreign_syscall::linux::ioctl::IoctlSyscall;
use crate::services::foreign_syscall::linux::mmap::MMapSyscall;
use crate::services::foreign_syscall::linux::poll::PollSyscall;
use crate::services::foreign_syscall::linux::rtsigaction::RtSigactionSyscall;
use crate::services::foreign_syscall::linux::rtsigprocmask::RtSigProcMaskSyscall;
use crate::services::foreign_syscall::linux::set_tid_address::SetTidAddressSyscall;
use crate::services::foreign_syscall::linux::signalstack::SignalStackSyscall;
use crate::services::foreign_syscall::linux::syscall_num::LinuxSyscallNum;
use crate::services::foreign_syscall::linux::syscall_num::LinuxSyscallNum::MMap;
use crate::services::foreign_syscall::linux::write::WriteSyscall;
use crate::services::foreign_syscall::linux::write_v::WriteVSyscall;
use crate::services::foreign_syscall::linux::LinuxSyscallImpl;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::fmt::Debug;
use libhrstd::libhedron::capability::MemCapPermissions;
use libhrstd::libhedron::ipc_serde::__private::Formatter;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::utcb::UtcbDataException;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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

        /*let mapping_dest = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());
        CrdDelegateOptimizer::new(
            utcb_exc.rip / PAGE_SIZE as u64,
            mapping_dest / PAGE_SIZE as u64,
            1,
        )
        .mmap(101, 32, MemCapPermissions::all());
        let ptr = mapping_dest as *const u8;
        let bytes = unsafe { core::slice::from_raw_parts(ptr, PAGE_SIZE) };

        let rip_offset = utcb_exc.rip & 0xfff;
        let first_16_bytes = &bytes[rip_offset as usize..16 + rip_offset as usize];
        for byte in first_16_bytes {
            log::debug!("{:x}", *byte);
        }*/

        let syscall_impl: Box<dyn LinuxSyscallImpl> = match self.rax {
            LinuxSyscallNum::Read => todo!(),
            LinuxSyscallNum::Write => Box::new(WriteSyscall::from(self)),
            LinuxSyscallNum::Open => todo!(),
            LinuxSyscallNum::Close => todo!(),
            LinuxSyscallNum::Poll => Box::new(PollSyscall::from(self)),
            LinuxSyscallNum::MMap => Box::new(MMapSyscall::from(self)),
            LinuxSyscallNum::MProtect => todo!(),
            LinuxSyscallNum::MUnmap => todo!(),
            LinuxSyscallNum::Brk => Box::new(BrkSyscall::from(self)),
            LinuxSyscallNum::RtSigaction => Box::new(RtSigactionSyscall::from(self)),
            LinuxSyscallNum::RtSigprocmask => Box::new(RtSigProcMaskSyscall::from(self)),
            LinuxSyscallNum::Ioctl => Box::new(IoctlSyscall::from(self)),
            LinuxSyscallNum::WriteV => Box::new(WriteVSyscall::from(self)),
            LinuxSyscallNum::Clone => todo!(),
            LinuxSyscallNum::Fcntl => todo!(),
            LinuxSyscallNum::SigAltStack => Box::new(SignalStackSyscall::from(self)),
            LinuxSyscallNum::ArchPrctl => Box::new(ArchPrctlSyscall::from(self)),
            LinuxSyscallNum::Gettid => todo!(),
            LinuxSyscallNum::Futex => todo!(),
            LinuxSyscallNum::SchedGetAffinity => todo!(),
            LinuxSyscallNum::SetTidAddress => Box::new(SetTidAddressSyscall::from(self)),
            LinuxSyscallNum::ExitGroup => todo!(),
            LinuxSyscallNum::ReadLinkAt => todo!(),
            LinuxSyscallNum::PrLimit64 => todo!(),
        };
        log::debug!("Linux syscall: {:?}", syscall_impl);
        utcb_exc.rax = syscall_impl.handle(utcb_exc, process).val();
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
