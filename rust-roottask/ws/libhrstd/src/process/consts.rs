//! Common abstractions for processes with hrstd.

/// ID of a process. Each process inside the runtime environment is a wrapper around a PD,
/// with convenient methods and data attached to them. Similar to a process on UNIX.
///
/// The init process, the roottask, has the ID [`ROOTTASK_PROCESS_PID`].
pub type ProcessId = u64;

/// The PID of the roottask.
pub const ROOTTASK_PROCESS_PID: ProcessId = 0;

/// Max number of supported processes.
pub const NUM_PROCESSES: u64 = 2_u64.pow(6);
