//! See [`GlobalStaticChunkAllocator`].

use crate::static_alloc::chunk::{
    ChunkAllocator,
    ChunkAllocatorError,
    DEFAULT_ALLOCATOR_CHUNK_SIZE,
};
use core::alloc::{
    GlobalAlloc,
    Layout,
};
use libhrstd::sync::mutex::SimpleMutex;

#[derive(Debug)]
pub enum GlobalStaticChunkAllocatorError {
    Uninitialized,
    AlreadyInitialized,
    /// Error in the inner allocator object.
    Inner(ChunkAllocatorError),
}

/// Wrapping struct around [`ChunkAllocator`] which enables the usage
/// of this allocator in a global context, i.e. as global allocator.
/// Memory is allocated in blocks/chunks with a size of
/// [`GlobalStaticChunkAllocator::CHUNK_SIZE`].
///
/// The struct synchronized accesses to the underlying memory.
/// It must be initialized by calling [`Self::init`], otherwise allocations
/// result in panics.
#[derive(Debug)]
pub struct GlobalStaticChunkAllocator<'a> {
    inner_allocator: SimpleMutex<Option<ChunkAllocator<'a, DEFAULT_ALLOCATOR_CHUNK_SIZE>>>,
}

impl<'a> GlobalStaticChunkAllocator<'a> {
    /// Publicly make the default chunk size available.
    pub const CHUNK_SIZE: usize = DEFAULT_ALLOCATOR_CHUNK_SIZE;

    /// Constructor.
    pub const fn new() -> Self {
        Self {
            inner_allocator: SimpleMutex::new(None),
        }
    }

    /// Initializes the allocator by feeding it with backing memory.
    /// This operation can be done once.
    pub fn init(
        &self,
        heap: &'a mut [u8],
        bitmap: &'a mut [u8],
    ) -> Result<(), GlobalStaticChunkAllocatorError> {
        let mut lock = self.inner_allocator.lock();
        if lock.is_some() {
            log::error!("Allocator already initialized!");
            Err(GlobalStaticChunkAllocatorError::AlreadyInitialized)
        } else {
            let alloc = ChunkAllocator::new(heap, bitmap)
                .map_err(|e| GlobalStaticChunkAllocatorError::Inner(e))?;
            log::debug!("initialized the allocator:");
            log::debug!("  chunks: {}", alloc.chunk_count());
            log::debug!("  heap: {} bytes", alloc.capacity());
            lock.replace(alloc);
            Ok(())
        }
    }

    /// Wrapper around [`ChunkAllocator::usage`].
    pub fn usage(&self) -> f64 {
        self.inner_allocator.lock().as_ref().unwrap().usage()
    }
}

unsafe impl<'a> GlobalAlloc for GlobalStaticChunkAllocator<'a> {
    #[track_caller]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // DON'T USE RECURSIVE ALLOCATING HERE
        // LIKE format!().. otherwise infinite loop because of the (dead)lock

        let mut lock = self.inner_allocator.lock();
        let lock = lock.as_mut().expect("allocator is uninitialized");
        let x = lock.alloc(layout);
        /*log::debug!(
            "allocated {} bytes at address 0x{:x}",
            layout.size(),
            x as usize
        );*/
        x
    }

    #[track_caller]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // DON'T USE RECURSIVE ALLOCATING HERE
        // LIKE format!().. otherwise infinite loop because of the (dead)lock

        // log::debug!("dealloc: ptr={:?}, layout={:?}", ptr, layout);
        let mut lock = self.inner_allocator.lock();
        let lock = lock.as_mut().expect("allocator is uninitialized");
        lock.dealloc(ptr, layout)
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use libhrstd::libhedron::mem::PAGE_SIZE;
    use libhrstd::mem::PageAlignedByteBuf;

    // must be a multiple of 8; 32 is equivalent to two pages
    const CHUNK_COUNT: usize = 32;
    const HEAP_SIZE: usize = DEFAULT_ALLOCATOR_CHUNK_SIZE * CHUNK_COUNT;
    static mut HEAP: PageAlignedByteBuf<HEAP_SIZE> = PageAlignedByteBuf::new_zeroed();
    const BITMAP_SIZE: usize = HEAP_SIZE / DEFAULT_ALLOCATOR_CHUNK_SIZE / 8;
    static mut BITMAP: PageAlignedByteBuf<BITMAP_SIZE> = PageAlignedByteBuf::new_zeroed();

    static ALLOCATOR: GlobalStaticChunkAllocator = GlobalStaticChunkAllocator::new();

    #[test]
    fn test_compiles() {
        unsafe {
            ALLOCATOR.init(HEAP.get_mut(), BITMAP.get_mut());
            let ptr = ALLOCATOR.alloc(Layout::from_size_align(256, PAGE_SIZE).unwrap());
            assert_eq!(ptr as u64 % PAGE_SIZE as u64, 0, "must be 4096-bit-aligned");
        };
    }
}
