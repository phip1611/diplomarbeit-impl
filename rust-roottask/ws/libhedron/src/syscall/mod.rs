//! Module with wrappers for all syscalls.
//!
//! # Capability Selector Hint
//! Almost every syscall takes capability selectors. Capability selectors during object
//! creation/deletes always refers to the capability space of the caller! This means, that if
//! you install a capability for a global EC that belongs to a other PD
//! (`create_ec(pd_sel_in_root: CapSel, ec_sel_in_root: CapSel)`), each cap is only installed
//! in the capability space of the caller. To have the capability in the target PD, you need
//! to delegate the cap afterwards too!

pub mod create_ec;
pub mod create_pd;
pub mod create_pt;
pub mod create_sc;
pub mod generic;
pub mod ipc;
pub mod pd_ctrl;
pub mod pt_ctrl;
