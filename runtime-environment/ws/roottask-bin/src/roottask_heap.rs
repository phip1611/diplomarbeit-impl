//! Allocator for the roottask - the HEAP. The roottask uses a statically allocated array
//! as backing storage for the HEAP. The memory is mapped and available after Hedron starts the
//! roottask.

use core::alloc::Layout;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;
use simple_chunk_allocator::{
    heap,
    heap_bitmap,
    GlobalChunkAllocator,
};

// times chunk_size==256 => 24MiB
// I need a relatively large heap for the in-mem file system benchmark
// The benchmark itself requires lots of heap but also the in-mem file system
// additionally, fragmentation makes this hard .. so yeah.. big heap required
const CHUNK_AMOUNT: usize = 98304;
static mut HEAP: simple_chunk_allocator::PageAligned<[u8; 25165824]> = heap!(chunks = CHUNK_AMOUNT);
static mut BITMAP: simple_chunk_allocator::PageAligned<[u8; 12288]> =
    heap_bitmap!(chunks = CHUNK_AMOUNT);

pub static HEAP_SIZE: usize = unsafe { HEAP.deref_const().len() };

/// Begin address of the heap.
pub static HEAP_BEGIN_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(HEAP.deref_const().as_ptr()) };

/// End address of the heap (exclusive!)
pub static HEAP_END_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(HEAP_BEGIN_PTR.get().add(HEAP_SIZE)) };

#[global_allocator]
static ALLOC: GlobalChunkAllocator =
    unsafe { GlobalChunkAllocator::new(HEAP.deref_mut_const(), BITMAP.deref_mut_const()) };

/// Wrapper around [`GlobalStaticChunkAllocator::usage`].
#[allow(unused)]
pub fn usage() -> f32 {
    ALLOC.usage()
}

#[alloc_error_handler]
fn alloc_error_handler(err: Layout) -> ! {
    panic!("Alloc Error, aborting program. layout={:#?}", err);
}
