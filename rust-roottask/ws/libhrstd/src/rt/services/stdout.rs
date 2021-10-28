use crate::cap_space::user::UserAppCapSpace;
use crate::rt::load_utcb::load_utcb_mut;
use libhedron::syscall::ipc::call;

pub fn stdout_write(msg: &str) {
    load_utcb_mut().store_data(&msg).unwrap();
    call(UserAppCapSpace::StdoutServicePT.val()).unwrap();
}
