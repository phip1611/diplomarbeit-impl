//! Module with wrappers for all syscalls.
//!
//! # Capability Selector Hint
//! Almost every syscall takes capability selectors. Capability selectors during object
//! creation/deletes always refers to the capability space of the caller! This means, that if
//! you install a capability for a global EC that belongs to a other PD
//! (`create_ec(pd_sel_in_root: CapSel, ec_sel_in_root: CapSel)`), each cap is only installed
//! in the capability space of the caller. To have the capability in the target PD, you need
//! to delegate the cap afterwards too!

use alloc::string::String;

mod create_ec;
pub use create_ec::*;
mod create_pd;
pub use create_pd::*;
mod create_pt;
pub use create_pt::*;
mod create_sc;
pub use create_sc::*;
mod generic;
pub use generic::*;
mod ipc;
pub use ipc::*;
mod pd_ctrl;
pub use pd_ctrl::*;
mod pt_ctrl;
pub use create_sm::*;
mod create_sm;
pub use sm_ctrl::*;
mod sm_ctrl;

pub use pt_ctrl::*;

/// Describes the possible results of system calls errors.
#[derive(Debug)]
pub enum SyscallError {
    /// The user provided illegal arguments and the syscall failed when
    /// the arguments where validated
    ClientArgumentError(String),
    /// Hedron returned an error.
    HedronStatusError(SyscallStatus),
}

/// Describes the result of all Hedron system calls.
pub type SyscallResult = Result<(), SyscallError>;
