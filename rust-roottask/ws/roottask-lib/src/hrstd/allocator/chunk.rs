//! See [`ChunkAllocator`].

use core::alloc::{
    GlobalAlloc,
    Layout,
};

/// Possible errors for [`ChunkAllocator`].
#[derive(Debug)]
pub enum ChunkAllocatorError {
    /// The backing memory for the heap must be
    /// - an even multiple of [`ChunkAllocator::CHUNK_SIZE`], and
    /// - a multiple of 8 to be correctly represented by the bitmap.
    BadHeapMemory,
    /// The number of bits in the backing memory for the heap bitmap
    /// must match the number of chunks in the heap.
    BadBitmapMemory,
}

/// First-fit allocator that takes mutable references to arbitrary external memory
/// backing storages. It uses them to manage memory. It is mandatory to wrap
/// this allocator by a mutex or a similar primitive, if it should be used
/// in a global context. It can take (global) static memory arrays as backing
/// storage. It allocates memory in chunks of [`Self::CHUNK_SIZE`].
///
/// TODO: dealloc + zero new memory.. or at dealloc?! or at init?!
#[derive(Debug)]
pub struct ChunkAllocator<'a> {
    heap: &'a mut [u8],
    bitmap: &'a mut [u8],
}

impl<'a> ChunkAllocator<'a> {
    /// Unit of size in bytes for memory allocations.
    pub const CHUNK_SIZE: usize = 256;

    /// Creates a new allocator object. Verifies that the provided memory has the correct properties.
    /// - heap length must be a multiple of [`Self::CHUNK_SIZE`]
    /// - the heap must be >=
    pub const fn new(
        heap: &'a mut [u8],
        bitmap: &'a mut [u8],
    ) -> Result<Self, ChunkAllocatorError> {
        let is_empty = heap.len() == 0;
        let is_not_multiple_of_chunk_size = heap.len() % Self::CHUNK_SIZE != 0;
        let is_not_coverable_by_bitmap = heap.len() < 8 * Self::CHUNK_SIZE;
        if is_empty || is_not_multiple_of_chunk_size || is_not_coverable_by_bitmap {
            return Err(ChunkAllocatorError::BadHeapMemory);
        }

        // check bitmap memory has correct length
        let expected_bitmap_length = heap.len() / Self::CHUNK_SIZE / 8;
        if bitmap.len() != expected_bitmap_length {
            return Err(ChunkAllocatorError::BadBitmapMemory);
        }

        Ok(Self { heap, bitmap })
    }

    /// Capacity in bytes of the allocator.
    pub const fn capacity(&self) -> usize {
        self.heap.len()
    }

    /// Returns number of chunks.
    pub fn chunk_count(&self) -> usize {
        // size is a multiple of CHUNK_SIZE;
        // ensured in new()
        self.capacity() / Self::CHUNK_SIZE
    }

    /// Returns whether a chunk is free according to the bitmap.
    ///
    /// # Parameters
    /// - `chunk_index` describes the start chunk; i.e. the search space inside the backing storage
    fn chunk_is_free(&self, chunk_index: usize) -> bool {
        assert!(chunk_index < self.chunk_count());
        let (byte_i, bit) = self.chunk_index_to_bitmap_indices(chunk_index);
        let relevant_bit = (self.bitmap[byte_i] >> bit) & 1;
        relevant_bit == 0
    }

    /// Returns the indices into the bitmap array of a given chunk index.
    fn chunk_index_to_bitmap_indices(&self, chunk_index: usize) -> (usize, usize) {
        assert!(
            chunk_index < self.chunk_count(),
            "chunk_index out of range!"
        );
        (chunk_index / 8, chunk_index % 8)
    }

    /// Returns the indices into the bitmap array of a given chunk index.
    ///
    /// # Parameters
    /// - `borrowed_lock` Option; prevents nested locking (i.e. deadlock)
    #[allow(unused)]
    fn bitmap_indices_to_chunk_index(&self, bitmap_index: usize, bit: usize) -> usize {
        let chunk_index = bitmap_index * 8 + bit;
        assert!(
            chunk_index < self.chunk_count(),
            "chunk_index out of range!"
        );
        chunk_index
    }

    /// Returns the index of the next free chunk of memory.
    ///
    /// # Parameters
    /// - `start_chunk` describes the start chunk; i.e. the search space inside the backing storage
    /// - `borrowed_lock` Option; prevents nested locking (i.e. deadlock)
    ///
    /// # Return
    /// Returns the index of the chunk or `Err` for out of memory.
    fn find_next_free_chunk(&self, start_chunk: Option<usize>) -> Result<usize, ()> {
        let start_chunk = start_chunk.unwrap_or(0);

        if start_chunk >= self.chunk_count() {
            log::debug!("chunk_index out of range!");
            return Err(());
        }

        for i in start_chunk..self.chunk_count() {
            if self.chunk_is_free(i) {
                return Ok(i);
            }
        }

        // out of memory
        Err(())
    }

    /// Finds the next available chain of available chunks. Returns the
    /// beginning index.
    ///
    /// # Parameters
    /// - `chunk_num` number of chunks that must be all free without gap in-between
    /// - `borrowed_lock` Option; prevents nested locking (i.e. deadlock)
    fn find_free_coherent_chunks(&self, chunk_num: usize) -> Result<usize, ()> {
        let mut begin_chunk_i = self.find_next_free_chunk(Some(0))?;
        while begin_chunk_i + (chunk_num - 1) < self.chunk_count() {
            // this var counts how many coherent chunks we found while iterating the bitmap
            let mut coherent_chunk_count = 1;
            for chunk_chain_i in 1..=chunk_num {
                if coherent_chunk_count == chunk_num {
                    return Ok(begin_chunk_i);
                } else if self.chunk_is_free(begin_chunk_i + chunk_chain_i) {
                    coherent_chunk_count += 1;
                } else {
                    break;
                }
            }

            // check again at next free block
            // "+1" because we want to skip the just discovered non-free block
            begin_chunk_i = self
                .find_next_free_chunk(Some(begin_chunk_i + coherent_chunk_count + 1))
                .unwrap();
        }
        // out of memory
        Err(())
    }

    /// Returns the pointer to the beginning of the chunk.
    unsafe fn chunk_index_to_ptr(&self, chunk_index: usize) -> *mut u8 {
        assert!(
            chunk_index < self.chunk_count(),
            "chunk_index out of range!"
        );
        self.heap.as_ptr().add(chunk_index * Self::CHUNK_SIZE) as *mut u8
    }
}

unsafe impl<'a> GlobalAlloc for ChunkAllocator<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut required_chunks = layout.size() / ChunkAllocator::CHUNK_SIZE;
        let modulo = layout.size() % ChunkAllocator::CHUNK_SIZE;

        log::debug!("alloc: layout={:?}", layout);

        if modulo != 0 {
            required_chunks += 1;
        }

        let index = self
            .find_free_coherent_chunks(required_chunks)
            .expect("out of memory");

        self.chunk_index_to_ptr(index)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        log::debug!("dealloc not supported yet");
        // TODO rust is nice enough and gives us a dealloc layout
        //  therefore we don't have to check how big the
        //  memory allocaiton was
        // panic!("unsupported dealloc: ptr={:?}, layout={:?}", ptr, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiles() {
        const TEST_HEAP_SIZE: usize = ChunkAllocator::CHUNK_SIZE * 16;
        static mut HEAP: [u8; TEST_HEAP_SIZE] = [0_u8; TEST_HEAP_SIZE];
        static mut BITMAP: [u8; TEST_HEAP_SIZE / ChunkAllocator::CHUNK_SIZE / 8] =
            [0_u8; TEST_HEAP_SIZE / ChunkAllocator::CHUNK_SIZE / 8];

        // check that it compiles
        let mut _alloc = unsafe { ChunkAllocator::new(&mut HEAP, &mut BITMAP).unwrap() };
    }

    #[test]
    fn test_chunk_count() {
        // step by 8 => heap size must be dividable by 8 for the bitmap
        for chunk_count in (8..128).step_by(8) {
            let heap_size: usize = chunk_count * ChunkAllocator::CHUNK_SIZE;
            let mut heap = vec![0_u8; heap_size];
            let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
            let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();
            assert_eq!(chunk_count, alloc.chunk_count());
        }
    }

    #[test]
    fn test_indices_helpers_1() {
        let heap_size: usize = 16 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        assert_eq!((0, 3), alloc.chunk_index_to_bitmap_indices(3));
        assert_eq!((0, 7), alloc.chunk_index_to_bitmap_indices(7));

        assert_eq!(3, alloc.bitmap_indices_to_chunk_index(0, 3));
        assert_eq!(7, alloc.bitmap_indices_to_chunk_index(0, 7));
    }

    #[test]
    fn test_indices_helpers_2() {
        let heap_size: usize = 16 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        assert_eq!((0, 7), alloc.chunk_index_to_bitmap_indices(7));
        assert_eq!((1, 0), alloc.chunk_index_to_bitmap_indices(8));
        assert_eq!((1, 1), alloc.chunk_index_to_bitmap_indices(9));

        assert_eq!(7, alloc.bitmap_indices_to_chunk_index(0, 7));
        assert_eq!(8, alloc.bitmap_indices_to_chunk_index(1, 0));
        assert_eq!(9, alloc.bitmap_indices_to_chunk_index(1, 1));
    }

    #[test]
    fn test_chunk_is_free() {
        let heap_size: usize = 16 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
        bitmap[0] = 0x0f;
        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        assert!(!alloc.chunk_is_free(0));
        assert!(!alloc.chunk_is_free(1));
        assert!(!alloc.chunk_is_free(2));
        assert!(!alloc.chunk_is_free(3));
        assert!(alloc.chunk_is_free(4));
    }

    #[test]
    fn test_find_next_free_chunk() {
        let heap_size: usize = 16 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
        bitmap[0] = 0x0f;
        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        assert_eq!(4, alloc.find_next_free_chunk(None).unwrap());
        assert_eq!(4, alloc.find_next_free_chunk(Some(0)).unwrap());

        // the very last chunk is available
        assert_eq!(
            15,
            alloc
                .find_next_free_chunk(Some(alloc.chunk_count() - 1))
                .unwrap()
        );
        assert!(alloc
            .find_next_free_chunk(Some(alloc.chunk_count()))
            .is_err());
    }

    #[test]
    fn test_find_free_coherent_chunks() {
        let heap_size: usize = 24 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];

        bitmap[0] = 0x0f;
        bitmap[1] = 0x0f;

        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        assert_eq!(4, alloc.find_free_coherent_chunks(1).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(2).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(3).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(4).unwrap());
        assert_eq!(12, alloc.find_free_coherent_chunks(5).unwrap());
    }

    #[test]
    fn test_chunk_index_to_ptr() {
        let heap_size: usize = 8 * ChunkAllocator::CHUNK_SIZE;
        let mut heap = vec![0_u8; heap_size];
        let ptr = heap.as_ptr();
        let mut bitmap = vec![0_u8; heap_size / ChunkAllocator::CHUNK_SIZE / 8];
        let alloc = ChunkAllocator::new(&mut heap, &mut bitmap).unwrap();

        unsafe {
            assert_eq!(ptr, alloc.chunk_index_to_ptr(0));
            assert_eq!(
                ptr as usize + ChunkAllocator::CHUNK_SIZE,
                alloc.chunk_index_to_ptr(1) as usize
            );
        }
    }
}
