//! Generic allocator, independent from the Rust core runtime allocator.
//! Also independent from the Kernel.
//!
use crate::hrstd::allocator::chunk::{
    ChunkAllocator,
    ChunkAllocatorError,
};
use crate::hrstd::sync::mutex::SimpleMutex;
use core::alloc::{
    GlobalAlloc,
    Layout,
};

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
/// It must be initialized first by calling [`Self::init`].
#[derive(Debug)]
pub struct GlobalStaticChunkAllocator<'a> {
    // data: SimpleMutex<Option<ChunkAllocator<'a>>>,
    data: SimpleMutex<Option<ChunkAllocator<'a>>>,
}

impl<'a> GlobalStaticChunkAllocator<'a> {
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
        let lock = self.data.lock();
        let lock = lock.as_ref().expect("allocator is uninitialized");
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
        let lock = self.data.lock();
        let lock = lock.as_ref().expect("allocator is uninitialized");
        lock.dealloc(ptr, layout)
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_compiles() {}
}
