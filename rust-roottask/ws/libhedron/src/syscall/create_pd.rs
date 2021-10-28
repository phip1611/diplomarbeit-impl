//! create_pd syscall

use crate::capability::{
    CapSel,
    CrdNull,
};
use crate::syscall::generic::{
    generic_syscall,
    SyscallNum,
    SyscallStatus,
};

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
/// # Parameters
/// - `has_passthrough_access` see description above
/// - `dest_cap_sel` Free capability selector in callers capability space
/// - `parent_pd_sel` The capability selector of the parent protection domain (e.g. root task)
///
pub fn create_pd(
    passthrough_access: bool,
    cap_sel: CapSel,
    parent_pd_sel: CapSel,
) -> Result<(), SyscallStatus> {
    log::trace!(
        "syscall create_pd: pd={:?}, parent_pd={}",
        cap_sel,
        parent_pd_sel
    );
    let mut arg1 = 0;
    arg1 |= SyscallNum::CreatePd.val() & 0xf;
    if passthrough_access {
        arg1 |= 1 << 4;
    }
    arg1 |= cap_sel << 8;
    let arg2 = parent_pd_sel;
    // arg3 is poorly described in spec. What kind of capabilities should be delegated initially?
    // Object ones, memory ones, ..?
    // Since we have a dedicated pd_ctrl#delegate syscall, it is recommended to use that instead
    let arg3 = CrdNull::default().val();
    unsafe {
        generic_syscall(arg1, arg2, arg3, 0, 0)
            .map(|_x| ())
            .map_err(|e| e.0)
    }
}
