//! See [`CapabilitySpace`].

use libhrstd::libhedron::capability::CapSel;

/// Describes the capability space of the roottask. Party determinined by Hedron,
/// the rest is a choice by me. Some of the capabilities stand also inside the HIP.
/// Anyhow, we don't expect or support changing capability space layouts without recompilation.
///
/// The variant value corresponds to the [`crate::libhrstd::libhedron::capability::CapSel`]
/// that refers to the given capability.
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum RootCapabilitySpace {
    /// Used as event offset for exceptions.
    ExceptionEventBase = 0,
    /// Last inclusive index of exception events.
    ExceptionEnd = 31,
    RootPd = 32,
    RootEc = 33,
    RootSc = 34,
    /// Local EC that handles exceptions of the roottask itself.
    RootExceptionLocalEc = 35,
    RoottaskStdoutLocalEc = 36,
    RoottaskStdoutPortal = 37,
}

impl RootCapabilitySpace {
    /// Returns the numeric value.
    pub fn val(self) -> CapSel {
        self as _
    }
}
