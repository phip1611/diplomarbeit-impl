//! Relevant information and constants when the kernel initially gives control
//! to the root task.

use roottask_lib::hedron::capability::CapSel;

/// The root task has 0 as event selector base. This means, initially
/// capability selectors 0..32 refer to a null capability, but can be used
/// for exception handling. To get the offset for the corresponding
/// event, see [`roottask_lib::hedron::event_offset::ExceptionEventOffset`].
///
/// The number of exceptions is also in [`roottask_lib::hedron::hip::HIP`] (field `num_exc_sel`).
pub const ROOT_EXC_EVENT_BASE: CapSel = 0;
