//! PD CTRL-syscalls.

use crate::capability::{
    CapSel,
    Crd,
};
use crate::syscall::generic::{
    generic_syscall,
    PdCtrlSubSyscall,
    SyscallNum,
    SyscallStatus,
};

/// Carries additional infos for a transfer or delegation call including some flags.
/// Can also be understood typed item
/// (Partly described by 4.6.2.2 Typed Items of original NOVA spec.)
#[derive(Copy, Clone, Debug)]
pub struct DelegateFlags(u64);

impl DelegateFlags {
    /// # Parameters
    /// - `host`
    /// - `device`
    /// - `guest`
    /// - `hypervisor` Only valid in roottask, silently ignored otherwise. Means "take cap from kern".
    /// - `hotspot` A hotspot is used to disambiguate send and receive windows for
    ///             delegations. The hotspot carries additional information for some types
    ///             of mappings as well.
    pub fn new(host: bool, device: bool, guest: bool, hypervisor: bool, hotspot: u64) -> Self {
        let mut base = 0;
        // refers to Typed Item Kind "Delegate"
        // "Translate" (0x0) will eventually be removed. Therefore we hard-code this bit here.
        base |= 1;
        // flags
        if host {
            base |= 1 << 8;
        }
        if device {
            base |= 1 << 9;
        }
        if guest {
            base |= 1 << 10;
        }
        if hypervisor {
            base |= 1 << 11;
        }
        // hotspot
        base |= hotspot << 12;
        Self(base)
    }

    pub fn val(self) -> u64 {
        self.0
    }

    // legacy; always true
    /*pub fn typ(self) -> bool {
        // taken from kernel-interface.md
        true
    }*/

    /// Mapping needs to go into (0) / not into (1) host page table. Only valid for memory and I/O delegations.
    pub fn host(self) -> bool {
        (self.0 & 0x100) != 0
    }

    /// Mapping needs to go into (1) / not into (0) guest page table / IO space. Valid for memory and I/O delegations.
    pub fn guest(self) -> bool {
        (self.0 & 0x200) != 0
    }

    /// Mapping needs to go into (1) / not into (0) device page table. Only valid for memory delegations.
    pub fn device(self) -> bool {
        (self.0 & 0x400) != 0
    }

    /// Source is actually hypervisor PD. Only valid when used by the roottask, silently ignored otherwise
    pub fn hypervisor(self) -> bool {
        (self.0 & 0x800) != 0
    }

    /// The hotspot used to disambiguate send and receive windows.
    pub fn hotspot(self) -> bool {
        (self.0 & 0xfffffffffffff000) != 0
    }
}

/// System call `pd_ctrl_delegate` transfers memory, port I/O and object capabilities
/// from one protection domain to another. It allows the same functionality as rights
/// delegation via IPC.
///
/// # Memory Delegations
/// SrcCRD and DestCRD ([`crate::capability::CrdMem`]) refer to virtual page numbers. If the
/// `hypervisor` flag of [`DelegateFlags`] is set and the source_pd is `0` (the one of the roottask),
/// all memory is identity mapped. Hence, virtual address is physical address.
///
/// # Parameters
/// - `source_crd` A [`Crd`] range descriptor describing the send window in the source PD.
/// - `dest_crd` A [`Crd`] describing the receive window in the destination PD.
pub fn pd_ctrl_delegate<Perm, Spec, ObjSpec>(
    source_pd: CapSel,
    dest_pd: CapSel,
    source_crd: Crd<Perm, Spec, ObjSpec>,
    dest_crd: Crd<Perm, Spec, ObjSpec>,
    flags: DelegateFlags,
) -> Result<(), SyscallStatus> {
    const SYSCALL_BITMASK: u64 = 0xf;
    const SUB_SYSCALL_BITMASK: u64 = 0x30;
    const SUB_SYSCALL_BITSHIFT: u64 = 4;
    const SOURCE_PD_BITMASK: u64 = !0xff;
    const SOURCE_PD_BITSHIFT: u64 = 8;

    let mut arg1 = 0;
    arg1 |= SyscallNum::PdCtrl.val() & SYSCALL_BITMASK;
    arg1 |= (PdCtrlSubSyscall::PdCtrlDelegate.val() << SUB_SYSCALL_BITSHIFT) & SUB_SYSCALL_BITMASK;
    arg1 |= (source_pd << SOURCE_PD_BITSHIFT) & SOURCE_PD_BITMASK;

    let arg2 = dest_pd;
    let arg3 = source_crd.val();
    let arg4 = flags;
    let arg5 = dest_crd.val();

    unsafe {
        generic_syscall(arg1, arg2, arg3, arg4.val(), arg5)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
