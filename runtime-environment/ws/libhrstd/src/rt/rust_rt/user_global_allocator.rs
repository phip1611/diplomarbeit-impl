use crate::rt::services::allocate::{
    alloc_service,
    dealloc_service,
};
use core::alloc::{
    GlobalAlloc,
    Layout,
};

#[global_allocator]
static GLOBAL_ALLOC: UserGlobalAllocator = UserGlobalAllocator::new();

/// Global Allocator for User Hedron-native User Apps. Currently it is dumb.
/// It allocates whole portions of pages (minimum allocation). THis is really inefficient.
struct UserGlobalAllocator {}

impl UserGlobalAllocator {
    const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for UserGlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = alloc_service(layout);
        log::trace!("alloc: layout={:?} ptr={:?}", layout, ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dealloc_service(ptr as u64, layout);
        log::trace!("dealloc: layout={:?} ptr={:?}", layout, ptr);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
