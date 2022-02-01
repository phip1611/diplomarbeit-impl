//! See [`UserAppCapabilitySpace`].

use crate::libhedron::consts::NUM_EXC;
use crate::libhedron::CapSel;

/// User application capability space.
/// Describes the capability space of the PD of Hedron-native Apps.
/// Each process has a 1:1 mapping to a PD.
///
/// The variant value corresponds to the [`crate::libhrstd::libhedron::CapSel`]
/// that refers to the given capability.
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum UserAppCapSpace {
    /// Used as event offset for exceptions.
    ExceptionEventBase = 0,
    /// Last inclusive index of exception events.
    ExceptionEnd = (NUM_EXC - 1) as u64,
    /// The capability to "self"/the protection domain of the PD, that belongs to a user app.
    Pd = 32,
    /// The capability to the main global EC, that belongs to a user app.
    Ec = 33,
    /// The capability to the main SC, that belongs to a user app.
    Sc = 34,
    /// CapSel for the allocator service portal.
    AllocatorServicePT = 35,
    /// CapSel for the stdout service portal.
    StdoutServicePT = 36,
    /// CapSel for the stderr service portal.
    StderrServicePT = 37,
    /// Service PT that multiplexes all file operations through a single portal.
    FsServicePT = 38,
}

impl UserAppCapSpace {
    /// Returns the numeric value.
    pub fn val(self) -> CapSel {
        self as _
    }
}

/// This is only an addition to [`UserAppCapSpace`] for foreign apps.
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum ForeignUserAppCapSpace {
    /// Begin value. This plus CPU_NUM equals the actual PT selector.
    SyscallBasePt = 50,
}

impl ForeignUserAppCapSpace {
    /// Returns the numeric value.
    pub fn val(self) -> CapSel {
        self as _
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_syscall_base_ot() {
        dbg!(ForeignUserAppCapSpace::SyscallBasePt.val());
    }
}
