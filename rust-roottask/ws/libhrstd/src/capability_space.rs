//! See [`UserAppCapabilitySpace`].

use crate::libhedron::capability::CapSel;
use crate::libhedron::consts::NUM_EXC;

/// User application capability space. Describes the capability space of Hedron-native Apps.
///
/// The variant value corresponds to the [`crate::libhrstd::libhedron::capability::CapSel`]
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
}

impl UserAppCapSpace {
    /// Returns the numeric value.
    pub fn val(self) -> CapSel {
        self as _
    }
}
