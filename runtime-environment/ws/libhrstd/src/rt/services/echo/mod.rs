#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
use crate::cap_space::user::UserAppCapSpace;
#[cfg(feature = "foreign_rust_rt")]
use crate::rt::hybrid_rt::syscalls::sys_hybrid_call;
#[cfg(feature = "native_rust_rt")]
use libhedron::syscall::sys_call;

/// Calls the echo service. Useful to measure the full communication path costs of
/// my portal multiplexing mechanism (cross-PD IPC).
pub fn call_echo_service() {
    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::EchoServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::EchoServicePT.val()).unwrap();
}

/// Calls the raw echo service. Useful to measure the stripped down communication path
/// costs of a basic call and reply without additional logic (cross-PD IPC).
pub fn call_raw_echo_service() {
    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::RawEchoServicePt.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::RawEchoServicePt.val()).unwrap();
}
