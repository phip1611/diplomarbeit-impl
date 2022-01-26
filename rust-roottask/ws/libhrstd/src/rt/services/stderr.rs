use crate::cap_space::user::UserAppCapSpace;
use crate::rt::services::stdout::msg_chunk_bulk_apply;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use libhedron::syscall::sys_call;

/// Writes a message to STDERR. If the message is too long, it does so in multiple iterations.
pub fn stderr_write(msg: &str) {
    let utcb = user_load_utcb_mut();
    let step_size = 4000;
    msg_chunk_bulk_apply(msg, step_size, move |msg| {
        utcb.store_data(&msg).unwrap();
        sys_call(UserAppCapSpace::StderrServicePT.val()).unwrap();
    });
}
