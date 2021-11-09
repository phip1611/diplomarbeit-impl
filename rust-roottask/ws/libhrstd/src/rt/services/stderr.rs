use crate::cap_space::user::UserAppCapSpace;
use crate::rt::load_utcb::load_utcb_mut;
use crate::rt::services::stdout::msg_chunk_bulk_apply;
use core::cmp::min;
use libhedron::syscall::ipc::call;

/// Writes a message to STDERR. If the message is too long, it does so in multiple iterations.
pub fn stderr_write(msg: &str) {
    let utcb = load_utcb_mut();
    let step_size = 4000;
    msg_chunk_bulk_apply(msg, step_size, move |msg| {
        utcb.store_data(&msg).unwrap();
        call(UserAppCapSpace::StderrServicePT.val()).unwrap();
    });
}
