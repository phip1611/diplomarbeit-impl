use libhrstd::libhedron::hip::HIP;
use libroottask::process_mng::process::Process;

pub mod stderr;
pub mod stdout;

/// Initializes stdout and stderr writers.
/// See [`stdout::StdoutWriter`] and [`stderr::StderrWriter`].
pub fn init_writers(hip: &HIP) {
    stdout::init_writer(hip);
    stderr::init_writer(hip);
}

/// Initializes stdout and stderr writers.
/// See [`stdout::StdoutWriter`] and [`stderr::StderrWriter`].
pub fn init_services(roottask: &Process) {
    stdout::init_service(roottask);
    // stderr::init_service(hip);
}
