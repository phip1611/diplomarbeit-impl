//! Common abstractions for processes with hrstd.

/// ID of a process.
pub type ProcessId = u64;

/// The PID of the roottask.
pub const ROOTTASK_PROCESS_PID: ProcessId = 0;

/// Max number of supported processes.
pub const NUM_PROCESSES: u64 = 2_u64.pow(32);
