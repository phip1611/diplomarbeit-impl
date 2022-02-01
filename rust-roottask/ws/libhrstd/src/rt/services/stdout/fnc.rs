use crate::cap_space::user::UserAppCapSpace;
#[cfg(feature = "foreign_rust_rt")]
use crate::rt::hybrid_rt::syscalls::sys_hybrid_call;
use crate::rt::services::stdout::msg_chunk_bulk_apply;
use crate::rt::user_load_utcb::user_load_utcb_mut;
#[cfg(feature = "native_rust_rt")]
use libhedron::syscall::sys_call;

/// Writes a message to STDOUT. If the message is too long, it does so in multiple iterations.
#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
pub fn stdout_service(msg: &str) {
    let utcb = user_load_utcb_mut();
    let step_size = 4000;
    msg_chunk_bulk_apply(msg, step_size, move |msg| {
        utcb.store_data(&msg).unwrap();

        #[cfg(feature = "native_rust_rt")]
        sys_call(UserAppCapSpace::StdoutServicePT.val()).unwrap();
        #[cfg(feature = "foreign_rust_rt")]
        sys_hybrid_call(UserAppCapSpace::StdoutServicePT.val()).unwrap();
    });
}
