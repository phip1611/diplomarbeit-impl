//! See [`GlobalStaticChunkAllocator`].

use crate::static_alloc::chunk::{
    ChunkAllocator,
    ChunkAllocatorError,
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
///
/// The struct synchronized accesses to the underlying memory.
///
/// It must be initialized first by calling [`Self::init`].
#[derive(Debug)]
pub struct GlobalStaticChunkAllocator<'a> {
    data: SimpleMutex<Option<ChunkAllocator<'a>>>,
}

impl<'a> GlobalStaticChunkAllocator<'a> {
    // Re-Exports the chunk size of the underlying allocator implementation.
    pub const CHUNK_SIZE: usize = ChunkAllocator::CHUNK_SIZE;

    pub const fn new() -> Self {
        Self {
            data: SimpleMutex::new(None),
        }
    }

    /// Initializes the allocator by feeding it with backing memory.
    /// This operation can be done once.
    pub fn init(
        &self,
        heap: &'a mut [u8],
        bitmap: &'a mut [u8],
    ) -> Result<(), GlobalStaticChunkAllocatorError> {
        let mut lock = self.data.lock();
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
}

unsafe impl<'a> GlobalAlloc for GlobalStaticChunkAllocator<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // DON'T USE RECURSIVE ALLOCATING HERE
        // LIKE format!().. otherwise infinite loop because of the (dead)lock

        log::debug!("alloc: {:?}", layout);
        let mut lock = self.data.lock();
        let lock = lock.as_mut().expect("allocator is uninitialized");
        let x = lock.alloc(layout);
        log::debug!(
            "allocated {} bytes at address 0x{:x}",
            layout.size(),
            x as usize
        );
        x
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // DON'T USE RECURSIVE ALLOCATING HERE
        // LIKE format!().. otherwise infinite loop because of the (dead)lock

        log::debug!("dealloc: {:?}", layout);
        let mut lock = self.data.lock();
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
    const HEAP_SIZE: usize = ChunkAllocator::CHUNK_SIZE * CHUNK_COUNT;
    static mut HEAP: PageAlignedByteBuf<HEAP_SIZE> = PageAlignedByteBuf::new_zeroed();
    const BITMAP_SIZE: usize = HEAP_SIZE / ChunkAllocator::CHUNK_SIZE / 8;
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
