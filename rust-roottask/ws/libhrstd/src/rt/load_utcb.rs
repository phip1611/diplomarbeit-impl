//! Helper methods to load the UTCB in Hedron user apps.
//! It is mapped at a well-known location.

use crate::libhedron::utcb::Utcb;
use crate::uaddress_space::USER_UTCB_ADDR;

#[allow(unused)]
pub fn load_utcb() -> &'static Utcb {
    unsafe { (USER_UTCB_ADDR as *const Utcb).as_ref().unwrap() }
}

/// Loads the UTCB from the well-known location in user apps.
pub fn load_utcb_mut() -> &'static mut Utcb {
    unsafe { (USER_UTCB_ADDR as *mut Utcb).as_mut().unwrap() }
}
