//! create_pd syscall

use crate::capability::{
    CapSel,
    CrdNull,
};
use crate::consts::NUM_CAP_SEL;
use crate::syscall::{
    hedron_syscall_4,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// `create_pd` creates a PD kernel object and a capability pointing to
/// the newly created kernel object. Protection domains are security
/// boundaries. They consist of several capability spaces. The host,
/// guest, and DMA capability spaces are the address spaces that are used
/// for ECs belonging to this PD or DMA from assigned devices. Similarly,
/// port I/O from ECs in this PD is checked against the port I/O
/// capability space.
///
/// PDs are roughly analogous to processes.
///
/// There are two special kinds of PDs. The PD (roottask) initially
/// created by the microhypervisor is privileged in that it directly
/// delegates arbitrary physical memory, I/O ports, and interrupts. This
/// property cannot be passed on.
///
/// The other special kind of PD is a _passthrough_ PD that has special
/// hardware access. The roottask is such a passthrough PD and can pass
/// this right on via the corresponding flag.
///
/// **Passthrough access is inherently insecure and should not be granted to
/// untrusted userspace PDs.**
///
/// This function never panics.
///
/// # Parameters
/// - `has_passthrough_access` see description above
/// - `dest_cap_sel` Free capability selector in callers capability space
/// - `parent_pd_sel` The capability selector of the parent protection domain (e.g. root task)
/// - `foreign_syscall_base` Of some, this PD will be a foreign PD (syscalls handled as exceptions)
///                          with the given foreign syscall base.
#[inline]
pub fn sys_create_pd(
    passthrough_access: bool,
    cap_sel: CapSel,
    parent_pd_sel: CapSel,
    foreign_syscall_base: Option<CapSel>,
) -> SyscallResult {
    if cap_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `cap_sel` is too big".to_string(),
        ))
    } else if parent_pd_sel >= NUM_CAP_SEL {
        Err(SyscallError::ClientArgumentError(
            "Argument `parent_pd_sel` is too big".to_string(),
        ))
    } else {
        log::trace!(
            "syscall create_pd: pd={:?}, parent_pd={}",
            cap_sel,
            parent_pd_sel
        );
        let mut arg1 = 0;
        arg1 |= SyscallNum::CreatePd.val() & 0xff;
        if passthrough_access {
            arg1 |= 1 << 8;
        }
        arg1 |= cap_sel << 12;
        let arg2 = parent_pd_sel;
        // arg3 is poorly described in spec. What kind of capabilities should be delegated initially?
        // Object ones, memory ones, ..?
        // Since we have a dedicated pd_ctrl#delegate syscall, it is recommended to use that instead
        let arg3 = CrdNull::default().val();
        let arg4 = foreign_syscall_base.map(|x| (x << 1) | 1).unwrap_or(0);
        unsafe {
            hedron_syscall_4(arg1, arg2, arg3, arg4)
                .map(|_x| ())
                .map_err(|e| SyscallError::HedronStatusError(e.0))
        }
    }
}
