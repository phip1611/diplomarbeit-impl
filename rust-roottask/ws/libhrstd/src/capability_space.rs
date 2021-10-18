//! See [`UserAppCapabilitySpace`].

use crate::libhedron::capability::CapSel;

/// Describes the capability space of Hedron-native Apps.
///
/// The variant value corresponds to the [`crate::libhrstd::libhedron::capability::CapSel`]
/// that refers to the given capability.
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum UserAppCapabilitySpace {
    /// Used as event offset for exceptions.
    ExceptionEventBase = 0,
    StartupExceptionPortal = 30,
    /// Last inclusive index of exception events.
    ExceptionEnd = 31,
    Pd = 32,
    Ec = 33,
    Sc = 34,
    AllocatorService = 35,
    StdoutService = 36,
    StderrService = 37,
}

impl UserAppCapabilitySpace {
    /// Returns the numeric value.
    pub fn val(self) -> CapSel {
        self as _
    }
}
