//! Utilities to request I/O ports from the kern PD into the roottask PD.

use libhrstd::libhedron::syscall::SyscallResult;
use libhrstd::libhedron::syscall::{
    sys_pd_ctrl_delegate,
    DelegateFlags,
};
use libhrstd::libhedron::{
    CapSel,
    CrdPortIO,
};

/// Wrapper around [`request_io_ports`].
pub fn request_io_port(pd: CapSel, io_port: u16) -> SyscallResult {
    let crd = CrdPortIO::new(io_port, 0);
    request_io_ports(pd, crd)
}

/// Maps the requested I/O port capabilities from the kern PD into
/// the root pd. It requires no [`CapSel`] because the kernel updates just updates
/// the bitmap.
///
/// # Parameters
/// - `pd` The protection domain that is the target
pub fn request_io_ports(pd: CapSel, io_cdr: CrdPortIO) -> SyscallResult {
    sys_pd_ctrl_delegate(
        pd,
        pd,
        io_cdr,
        // Not sure if dest crd is used at all in this case
        io_cdr,
        DelegateFlags::new(true, false, false, true, 0),
    )
}
