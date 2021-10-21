//! Helper methods to load the UTCB in Hedron user apps.
//! It is mapped at a well-known location.

use crate::libhedron::utcb::Utcb;
use crate::uaddress_space::VIRT_UTCB_ADDR;

pub fn load_utcb() -> &'static Utcb {
    unsafe { (VIRT_UTCB_ADDR as *const Utcb).as_ref().unwrap() }
}

pub fn load_utcb_mut() -> &'static mut Utcb {
    unsafe { (VIRT_UTCB_ADDR as *mut Utcb).as_mut().unwrap() }
}
