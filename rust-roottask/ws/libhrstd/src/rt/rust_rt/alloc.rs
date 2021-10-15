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
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
