//! Allocator for the roottask. The roottask uses a statically allocated array. The memory
//! is mapped and available after Hedron starts the roottask.

use core::alloc::Layout;
use libroottask::static_alloc::{
    GlobalStaticChunkAllocator,
    StaticAlignedMem,
};

// must be a multiple of 8; 512MiB
const HEAP_SIZE: usize = 0x20000000;
static mut HEAP: StaticAlignedMem<HEAP_SIZE> = StaticAlignedMem::new();
const BITMAP_SIZE: usize = HEAP_SIZE / GlobalStaticChunkAllocator::CHUNK_SIZE / 8;
static mut BITMAP: StaticAlignedMem<BITMAP_SIZE> = StaticAlignedMem::new();

#[global_allocator]
static ALLOC: GlobalStaticChunkAllocator = GlobalStaticChunkAllocator::new();

/// Initializes the global static rust allocator. It uses static memory already available
/// inside the address space.
pub fn init() {
    unsafe { ALLOC.init(HEAP.data_mut(), BITMAP.data_mut()).unwrap() }
    log::debug!("initialized allocator");
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
