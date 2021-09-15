use core::alloc::{
    GlobalAlloc,
    Layout,
};

use core::sync::atomic::{
    compiler_fence,
    Ordering,
};

const PAGESIZE: u64 = 1024;
const HEAPSIZE: u64 = PAGESIZE * 1024;
const HEAP: [u8; HEAPSIZE as usize] = [0; HEAPSIZE as usize];

#[global_allocator]
static ALLOC: RoottaskAlloctor = RoottaskAlloctor;

struct RoottaskAlloctor;

unsafe impl GlobalAlloc for RoottaskAlloctor {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}

#[alloc_error_handler]
fn foo(_info: Layout) -> ! {
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
