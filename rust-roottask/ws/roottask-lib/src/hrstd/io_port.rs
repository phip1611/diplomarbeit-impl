use crate::hedron::capability::{
    CapSel,
    CrdPortIO,
};
use crate::syscall::generic::{
    PdCtrlSubSyscall,
    SyscallStatus,
};
use crate::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};

/// Requests access to a single I/O port. It doesn't return a capability
/// selector because the kernel updates the I/O bitmap.
pub fn request_io_port(pd: CapSel, io_port: u16) -> Result<(), SyscallStatus> {
    let crd = CrdPortIO::new(io_port, 0);
    request_io_ports(pd, crd)
}

pub fn request_io_ports(pd: CapSel, io_capsel: CrdPortIO) -> Result<(), SyscallStatus> {
    pd_ctrl_delegate(
        PdCtrlSubSyscall::PdCtrlDelegate,
        pd,
        pd,
        io_capsel,
        io_capsel,
        DelegateFlags::new(false, false, false, true, 0),
    )
}
