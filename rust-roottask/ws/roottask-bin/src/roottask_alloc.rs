//! Allocator for the roottask.

use core::alloc::Layout;
use libhrstd::allocator::chunk::ChunkAllocator;
use libhrstd::allocator::static_chunk::GlobalStaticChunkAllocator;

/// 2 MiB
const HEAPSIZE: usize = 1024 * 8 * ChunkAllocator::CHUNK_SIZE;
static mut HEAP: [u8; HEAPSIZE] = [0; HEAPSIZE];
const BITMAPSIZE: usize = HEAPSIZE / ChunkAllocator::CHUNK_SIZE / 8;
static mut HEAP_BITMAP: [u8; BITMAPSIZE] = [0; BITMAPSIZE];

#[global_allocator]
static ALLOC: GlobalStaticChunkAllocator = GlobalStaticChunkAllocator::new();

/// Initializes the global static rust allocator. It uses static memory already available
/// inside the address space.
pub fn init() {
    unsafe { ALLOC.init(&mut HEAP, &mut HEAP_BITMAP).unwrap() }
    log::debug!("initialized allocator");
}

#[alloc_error_handler]
fn alloc_error_handler(_err: Layout) -> ! {
    panic!("Alloc Error, aborting program.");
}
