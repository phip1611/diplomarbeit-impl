//! Module for struct [`StaticGlobalPtr`].

use core::fmt::{
    Debug,
    Formatter,
};

/// This type enables us to share raw pointers as global static variables.
/// Be careful what you do with the pointers.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct StaticGlobalPtr<T: Sized>(*const T);

impl<T: Sized> StaticGlobalPtr<T> {
    /// Constructs a new wrapper type for a pointer.
    pub const fn new(ptr: *const T) -> Self {
        Self(ptr)
    }

    /// Returns the pointer as 64 bit value.
    pub fn val(self) -> u64 {
        self.0 as u64
    }

    /// Casts the type
    pub const unsafe fn get(self) -> *const T {
        self.0
    }
}

unsafe impl<T> Sync for StaticGlobalPtr<T> {}

impl<T> Debug for StaticGlobalPtr<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
