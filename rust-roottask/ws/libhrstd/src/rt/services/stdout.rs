use crate::cap_space::user::UserAppCapSpace;
use crate::rt::load_utcb::load_utcb_mut;
use core::cmp::min;
use libhedron::syscall::ipc::call;

/// Writes a message to STDOUT. If the message is too long, it does so in multiple iterations.
pub fn stdout_write(msg: &str) {
    let utcb = load_utcb_mut();
    let step_size = 4000;
    msg_chunk_bulk_apply(msg, step_size, move |msg| {
        utcb.store_data(&msg).unwrap();
        call(UserAppCapSpace::StdoutServicePT.val()).unwrap();
    });
}

/// Splits a message into multiple chunks and applies the function step by step. This is useful
/// because the message may be to large to fit into the UTCB.
pub(super) fn msg_chunk_bulk_apply(msg: &str, step_size: usize, fnc: impl FnMut(&str) -> ()) {
    (0..msg.len())
        .step_by(step_size)
        .map(|step| &msg[step..min(msg.len(), step + step_size)])
        .for_each(fnc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use alloc::vec::Vec;
    use std::cell::RefCell;
    use std::prelude::v1::String;

    #[test]
    fn test_msg_to_chunk_splitter() {
        let msg = "Hallo Welt!\n";
        let msgs = Vec::with_capacity(3);
        let msgs = Rc::new(RefCell::new(msgs));
        let msgs_closure = msgs.clone();
        msg_chunk_bulk_apply(msg, 5, move |msg| {
            msgs_closure.borrow_mut().push(String::from(msg))
        });
        assert_eq!(msgs.borrow()[0], "Hallo");
        assert_eq!(msgs.borrow()[1], " Welt");
        assert_eq!(msgs.borrow()[2], "!\n");
    }
}
