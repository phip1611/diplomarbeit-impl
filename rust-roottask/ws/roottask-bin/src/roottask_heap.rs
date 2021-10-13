//! Allocator for the roottask - the HEAP. The roottask uses a statically allocated array
//! as backing storage for the HEAP. The memory is mapped and available after Hedron starts the
//! roottask.

use core::alloc::Layout;
use libhrstd::mem::PageAlignedByteBuf;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;
use libroottask::static_alloc::GlobalStaticChunkAllocator;

/// 32768 chunks -> 8 MiB Heap
pub const HEAP_SIZE: usize = GlobalStaticChunkAllocator::CHUNK_SIZE * 32768;
static mut HEAP: PageAlignedByteBuf<HEAP_SIZE> = PageAlignedByteBuf::new_zeroed();
// always make sure, that the division is "clean", i.e. no remainder
const BITMAP_SIZE: usize = HEAP_SIZE / GlobalStaticChunkAllocator::CHUNK_SIZE / 8;
static mut BITMAP: PageAlignedByteBuf<BITMAP_SIZE> = PageAlignedByteBuf::new_zeroed();

/// Begin address of the heap.
pub static HEAP_BEGIN_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(HEAP.get_mut().as_ptr()) };

/// End address of the heap (exclusive!)
pub static HEAP_END_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(HEAP_BEGIN_PTR.get().add(HEAP_SIZE)) };

#[global_allocator]
static ALLOC: GlobalStaticChunkAllocator = GlobalStaticChunkAllocator::new();

/// Initializes the global static rust allocator. It uses static memory already available
/// inside the address space.
pub fn init() {
    unsafe { ALLOC.init(HEAP.get_mut(), BITMAP.get_mut()).unwrap() }
    log::debug!("initialized allocator");
}

/// Wrapper around [`GlobalStaticChunkAllocator::usage`].
pub fn usage() -> f64 {
    ALLOC.usage()
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
