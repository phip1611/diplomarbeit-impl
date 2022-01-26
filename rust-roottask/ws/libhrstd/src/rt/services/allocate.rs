use crate::cap_space::user::UserAppCapSpace;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use core::alloc::Layout;
use libhedron::syscall::sys_call;
use serde::{
    Deserialize,
    Serialize,
};

/// Allocates memory from the roottask allocator.
pub fn alloc(layout: Layout) -> *mut u8 {
    let utcb = user_load_utcb_mut();
    utcb.store_data(&AllocRequest::from(layout)).unwrap();
    sys_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();
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
