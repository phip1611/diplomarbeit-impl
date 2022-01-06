use std::env::var;
use libhedron::mem::{PAGE_SIZE, USER_MAX_ADDR};
use libhedron::syscall::create_ec::create_global_ec;
use libhedron::syscall::create_pd::create_pd;
use libhedron::utcb::Utcb;

const USER_UTCB_ADDR: u64 = (USER_MAX_ADDR - PAGE_SIZE) as u64;
const HEDRON_NATIVE_SYSCALL_MAGIC: u64 = 1 << 63;

fn main() {
    println!("Hello, world!");

    if var("LINUX_UNDER_HEDRON").is_ok() {
        println!("This Linux binary executes under Hedron");
        // Set "TLS"-Field in UTCB head.
        // Hedron will now treat all foreign syscalls as its own syscalls.
        utcb_mut().set_head_tls(HEDRON_NATIVE_SYSCALL_MAGIC);
        create_pd(false, 2, 1, None).unwrap();
        create_global_ec(64, 1, 0, 0, 0).unwrap();
        // Reset mandatory!
        utcb_mut().set_head_tls(0 << 63);
    } else {
        println!("This Linux binary executes under native Linux");
    }
}

/// Loads the UTCB from the well-known location in user apps.
/// Currently: Shortcut, only one UTCB for main thread.
pub fn utcb_mut() -> &'static mut Utcb {
    unsafe { (USER_UTCB_ADDR as *mut Utcb).as_mut().unwrap() }
}
