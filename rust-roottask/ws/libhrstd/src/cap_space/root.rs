//! See [`CapabilitySpace`].

use crate::libhedron::capability::CapSel;
use crate::libhedron::consts::{
    NUM_CPUS,
    NUM_EXC,
};
use crate::process::consts::NUM_PROCESSES;
use enum_iterator::IntoEnumIterator;

const PROCESS_PD_BASE: u64 = 100;
const PROCESS_PD_END: u64 = PROCESS_PD_BASE + NUM_PROCESSES;
const PROCESS_EC_BASE: u64 = PROCESS_PD_END + 1;
const PROCESS_EC_END: u64 = PROCESS_EC_BASE + (NUM_PROCESSES * NUM_CPUS as u64);
const PROCESS_SC_BASE: u64 = PROCESS_EC_END + 1;
const PROCESS_SC_END: u64 = PROCESS_SC_BASE + NUM_PROCESSES;
const PROCESS_EXC_PT_BASE: u64 = PROCESS_SC_END + 1;
const PROCESS_EXC_PT_END: u64 = PROCESS_EXC_PT_BASE + (NUM_PROCESSES * NUM_EXC as u64);

/// Describes the capability space of the roottask. Party determinined by Hedron,
/// the rest is a choice by me. Some of the capabilities stand also inside the HIP.
/// Anyhow, we don't expect or support changing capability space layouts without recompilation.
///
/// The variant value corresponds to the [`crate::libhrstd::libhedron::capability::CapSel`]
/// that refers to the given capability.
#[repr(u64)]
#[derive(Copy, Clone, Debug, IntoEnumIterator)]
pub enum RootCapSpace {
    /// Used as event offset for exceptions.
    ExceptionEventBase = 0,
    /// Last inclusive index of exception events.
    ExceptionEnd = (NUM_EXC - 1) as u64,

    /// CapSel of the root PD.
    RootPd = 32,
    /// CapSel of the (global) root EC.
    RootEc = 33,
    /// CapSel of the root SC.
    RootSc = 34,

    /// Local EC used for exception handling inside the roottask and user apps.
    /// Exception-portals shall be attached to this local EC.
    RootExceptionLocalEc = 35,

    /// The CapSel for the local EC for the STDOUT service portal.
    RoottaskStdoutServiceLocalEc = 36,
    /// CapSel for the portal for the STDOUT service,
    RoottaskStdoutServicePortal = 37,

    /// Base CapSel for the PD of a process. This + PID => capability index offset
    ProcessPdBase = PROCESS_PD_BASE,
    /// Last inclusive index relative to [`ProcessPdBase`].
    ProcessPdEnd = PROCESS_PD_END,

    /// Base CapSel for the global EC of a process. This + PID + CPU => capability index offset
    ProcessEcBase = PROCESS_EC_BASE,
    /// Last inclusive index relative to [`ProcessEcBase`].
    ProcessEcEnc = PROCESS_EC_END,

    /// Base CapSel for the SC of a process. This + PID => capability index offset
    ProcessScBase = PROCESS_SC_BASE,
    /// Last inclusive index relative to [`ProcessScBase`].
    ProcessScEnd = PROCESS_SC_END,

    /// Base CapSel for the exception portals of a process. This + PID + NUM-EXC => cap index offset
    ProcessExcPtBase = PROCESS_EXC_PT_BASE,
    /// Last inclusive index relative to [`ProcessExcPtBase`].
    ProcessExcPtEnd = PROCESS_EXC_PT_END,
    _Max,
}

impl RootCapSpace {
    /// Returns the numeric value.
    pub const fn val(self) -> CapSel {
        self as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libhedron::consts::NUM_CAP_SEL;
    use alloc::vec::Vec;

    #[test]
    fn print_root_cap_space() {
        let variants: Vec<RootCapSpace> = RootCapSpace::into_enum_iter().collect::<Vec<_>>();
        let variants = variants
            .into_iter()
            .map(|x| (x, x.val()))
            .collect::<Vec<_>>();
        dbg!(variants);
    }

    #[test]
    fn test_assert_max_cap_sel() {
        assert!(RootCapSpace::_Max.val() <= NUM_CAP_SEL);
    }
}
