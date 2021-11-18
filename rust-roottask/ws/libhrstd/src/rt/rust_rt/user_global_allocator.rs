use core::alloc::{
    GlobalAlloc,
    Layout,
};

#[global_allocator]
static GLOBAL_ALLOC: GlobalAllocator = GlobalAllocator::new();

struct GlobalAllocator {}

impl GlobalAllocator {
    const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        log::trace!("alloc: layout={:?}", layout);
        let ptr = super::super::services::allocate::alloc(layout);
        log::trace!("alloc: layout={:?} ptr={:?}", layout, ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        log::warn!(
            "dealloc not implemented yet :D; ptr={:?}, layout={:?}",
            ptr,
            layout
        );
    }
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
