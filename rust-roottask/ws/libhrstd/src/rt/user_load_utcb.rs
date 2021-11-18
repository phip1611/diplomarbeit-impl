//! Helper methods to load the UTCB in Hedron user apps.
//! It is mapped at a well-known location.

use crate::libhedron::utcb::Utcb;
use crate::uaddress_space::USER_UTCB_ADDR;

#[allow(unused)]
pub fn user_load_utcb() -> &'static Utcb {
    unsafe { (USER_UTCB_ADDR as *const Utcb).as_ref().unwrap() }
}

/// Loads the UTCB from the well-known location in user apps.
/// TODO currently this allows multiple mutuable references and ignores guarantees Rust wants to give.
pub fn user_load_utcb_mut() -> &'static mut Utcb {
    unsafe { (USER_UTCB_ADDR as *mut Utcb).as_mut().unwrap() }
}
