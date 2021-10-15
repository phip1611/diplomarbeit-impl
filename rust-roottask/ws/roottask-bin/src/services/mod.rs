use libhrstd::libhedron::hip::HIP;

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
pub fn init_services(hip: &HIP) {
    stdout::init_service(hip);
    // stderr::init_service(hip);
}
