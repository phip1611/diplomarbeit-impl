use core::alloc::{Allocator, Layout, AllocError, GlobalAlloc};
use core::ptr::NonNull;
use core::sync::atomic::{compiler_fence, Ordering};

const PAGESIZE: u64 = 1024;
const HEAPSIZE: u64 = PAGESIZE * 1024;
const HEAP: [u8; HEAPSIZE as usize] = [0; HEAPSIZE as usize];

#[global_allocator]
static ALLOC: RoottaskAlloctor = RoottaskAlloctor;

struct RoottaskAlloctor;

unsafe impl GlobalAlloc for RoottaskAlloctor {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}

#[alloc_error_handler]
fn foo(info: Layout) -> ! {
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
