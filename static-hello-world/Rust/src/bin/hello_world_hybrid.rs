use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::PdObject;
use libhrstd::libhedron::mem::{PAGE_SIZE, USER_MAX_ADDR};
use libhrstd::libhedron::syscall::create_ec::create_global_ec;
use libhrstd::libhedron::syscall::create_pd::create_pd;
use libhrstd::libhedron::utcb::Utcb;
use std::env::var;
use std::rc::Rc;

const USER_UTCB_ADDR: u64 = (USER_MAX_ADDR - PAGE_SIZE) as u64;
const HEDRON_NATIVE_SYSCALL_MAGIC: u64 = 1 << 63;

/// Wraps a single Hedron syscall. The code inside `actions`
/// should not do any more than the raw syscall. Things as
/// unwrapping on an `Err` will already break everything,
/// because the panic handler might write to stderr whereas
/// Hedron still things, the application tries to make Hedron-
/// native syscalls.
fn wrap_hedron_syscall<T, R>(actions: T) -> R
where
    T: Fn() -> R,
{
    utcb_mut().set_head_tls(HEDRON_NATIVE_SYSCALL_MAGIC);
    let res = actions();
    utcb_mut().set_head_tls(0);
    res
}

fn main() {
    println!("Hello, world!");

    if var("LINUX_UNDER_HEDRON").is_ok() {
        println!("This Linux binary executes under Hedron");
        let new_pd = create_new_pd();
        println!("created new PD: {:#?}", new_pd);
    } else {
        println!("This Linux binary executes under native Linux");
    }
}

/// Loads the UTCB from the well-known location in user apps.
/// Currently: Shortcut, only one UTCB for main thread.
fn utcb_mut() -> &'static mut Utcb {
    unsafe { (USER_UTCB_ADDR as *mut Utcb).as_mut().unwrap() }
}

fn create_new_pd() -> Rc<PdObject> {
    let self_pd = PdObject::self_in_user_cap_space(1);
    let pd = PdObject::new(self_pd.pid() + 1, Some(&self_pd), 1000);
    wrap_hedron_syscall(|| create_pd(false, pd.cap_sel(), self_pd.cap_sel(), None)).unwrap();
    pd
}
