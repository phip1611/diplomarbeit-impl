use crate::cap_space::user::UserAppCapSpace;
#[cfg(feature = "foreign_rust_rt")]
use crate::rt::hybrid_rt::syscalls::sys_hybrid_call;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use core::alloc::Layout;
#[cfg(feature = "native_rust_rt")]
use libhedron::syscall::sys_call;
use serde::{
    Deserialize,
    Serialize,
};

/// Allocates memory from the roottask allocator.
#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
pub fn alloc_service(layout: Layout) -> *mut u8 {
    let utcb = user_load_utcb_mut();
    utcb.store_data(&AllocRequest::from(layout)).unwrap();

    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();

    utcb.load_data::<u64>().unwrap() as *mut u8
}

/// Like "Layout" but serializable.
#[derive(Serialize, Deserialize, Debug)]
pub struct AllocRequest {
    size: usize,
    align: usize,
}

impl AllocRequest {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn align(&self) -> usize {
        self.align
    }
}

impl From<Layout> for AllocRequest {
    fn from(layout: Layout) -> Self {
        Self {
            size: layout.size(),
            align: layout.align(),
        }
    }
}
