//! Module for [`ChunkAllocator`].

use core::alloc::Layout;

/// Possible errors for [`ChunkAllocator`].
/// TODO make more generic ?! later use in roottask and native hedron app with different allocator frontends?
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
/// It is a generic allocator but can be wrapped to be used as the allocator for the Rust runtime,
/// i.e. the functionality of the `alloc` crate.
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

    /// Marks a chunk as used, i.e. write a 1 into the bitmap at the right position.
    fn mark_chunk_as_used(&mut self, chunk_index: usize) {
        assert!(chunk_index < self.chunk_count());
        if !self.chunk_is_free(chunk_index) {
            panic!(
                "tried to mark chunk {} as used but it is already used",
                chunk_index
            );
        }
        let (byte_i, bit) = self.chunk_index_to_bitmap_indices(chunk_index);
        // xor => keep all bits, except bitflip at relevant position
        self.bitmap[byte_i] = self.bitmap[byte_i] ^ (1 << bit);
    }

    /// Marks a chunk as free, i.e. write a 0 into the bitmap at the right position.
    fn mark_chunk_as_free(&mut self, chunk_index: usize) {
        assert!(chunk_index < self.chunk_count());
        if self.chunk_is_free(chunk_index) {
            panic!(
                "tried to mark chunk {} as free but it is already free",
                chunk_index
            );
        }
        let (byte_i, bit) = self.chunk_index_to_bitmap_indices(chunk_index);
        // xor => keep all bits, except bitflip at relevant position
        let updated_byte = self.bitmap[byte_i] ^ (1 << bit);
        self.bitmap[byte_i] = updated_byte;
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
    #[allow(unused)]
    fn bitmap_indices_to_chunk_index(&self, bitmap_index: usize, bit: usize) -> usize {
        let chunk_index = bitmap_index * 8 + bit;
        assert!(
            chunk_index < self.chunk_count(),
            "chunk_index out of range!"
        );
        chunk_index
    }

    /// Returns the index of the next free chunk of memory that is correctly aligned.
    ///
    /// # Parameters
    /// - `start_chunk` describes the start chunk; i.e. the search space inside the backing storage
    /// - `alignment` required alignment of the chunk in memory
    ///
    /// # Return
    /// Returns the index of the chunk or `Err` for out of memory.
    fn find_next_free_chunk(
        &self,
        start_chunk: Option<usize>,
        alignment: u32,
    ) -> Result<usize, ()> {
        let start_chunk = start_chunk.unwrap_or(0);

        if start_chunk >= self.chunk_count() {
            log::debug!("chunk_index out of range!");
            return Err(());
        }

        for i in start_chunk..self.chunk_count() {
            // given the fact that the first chunk is page-aligned (see struct definition)
            let is_aligned = (i * Self::CHUNK_SIZE) as u32 % alignment == 0;
            if is_aligned && self.chunk_is_free(i) {
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
    /// - `alignment` required alignment of the chunk in memory
    fn find_free_coherent_chunks(&self, chunk_num: usize, alignment: u32) -> Result<usize, ()> {
        let mut begin_chunk_i = self.find_next_free_chunk(Some(0), alignment)?;
        let out_of_mem_cond = begin_chunk_i + (chunk_num - 1) >= self.chunk_count();
        while !out_of_mem_cond {
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
                .find_next_free_chunk(Some(begin_chunk_i + coherent_chunk_count + 1), alignment)
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

    /// Returns the chunk index of the given pointer (which points to the beginning of a chunk).
    unsafe fn ptr_to_chunk_index(&self, ptr: *const u8) -> usize {
        let heap_begin_inclusive = self.heap.as_ptr();
        let heap_end_exclusive = self.heap.as_ptr().add(self.chunk_count());
        assert!(
            heap_begin_inclusive <= ptr && ptr < heap_end_exclusive,
            "pointer {:?} is out of range {:?}..{:?} of the allocators backing storage",
            ptr,
            heap_begin_inclusive,
            heap_end_exclusive
        );
        (ptr as usize - heap_begin_inclusive as usize) / Self::CHUNK_SIZE
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut required_chunks = layout.size() / ChunkAllocator::CHUNK_SIZE;
        let modulo = layout.size() % ChunkAllocator::CHUNK_SIZE;

        log::debug!("alloc: layout={:?} ({} chunks]", layout, required_chunks);

        if modulo != 0 {
            required_chunks += 1;
        }

        let index = self
            .find_free_coherent_chunks(required_chunks, layout.align() as u32)
            .expect("out of memory");

        for i in index..index + required_chunks {
            self.mark_chunk_as_used(i);
        }

        self.chunk_index_to_ptr(index)
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut required_chunks = layout.size() / ChunkAllocator::CHUNK_SIZE;
        let modulo = layout.size() % ChunkAllocator::CHUNK_SIZE;
        if modulo != 0 {
            required_chunks += 1;
        }
        log::debug!("dealloc: layout={:?} ({} chunks]", layout, required_chunks);

        let index = self.ptr_to_chunk_index(ptr as *const u8);
        for i in index..index + required_chunks {
            self.mark_chunk_as_free(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::static_alloc::static_aligned_mem::StaticAlignedMem;

    #[test]
    fn test_compiles() {
        // must be a multiple of 8
        const CHUNK_COUNT: usize = 16;
        const HEAP_SIZE: usize = ChunkAllocator::CHUNK_SIZE * CHUNK_COUNT;
        static mut HEAP: StaticAlignedMem<HEAP_SIZE> = StaticAlignedMem::new();
        const BITMAP_SIZE: usize = HEAP_SIZE / ChunkAllocator::CHUNK_SIZE / 8;
        static mut BITMAP: StaticAlignedMem<BITMAP_SIZE> = StaticAlignedMem::new();

        // check that it compiles
        let mut _alloc =
            unsafe { ChunkAllocator::new(HEAP.data_mut(), BITMAP.data_mut()).unwrap() };
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

        assert_eq!(4, alloc.find_next_free_chunk(None, 1).unwrap());
        assert_eq!(4, alloc.find_next_free_chunk(Some(0), 1).unwrap());

        // the very last chunk is available
        assert_eq!(
            15,
            alloc
                .find_next_free_chunk(Some(alloc.chunk_count() - 1), 1)
                .unwrap()
        );
        assert!(alloc
            .find_next_free_chunk(Some(alloc.chunk_count()), 1)
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

        assert_eq!(4, alloc.find_free_coherent_chunks(1, 1).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(2, 1).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(3, 1).unwrap());
        assert_eq!(4, alloc.find_free_coherent_chunks(4, 1).unwrap());
        assert_eq!(12, alloc.find_free_coherent_chunks(5, 1).unwrap());
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

    #[test]
    fn test_alloc() {
        // must be a multiple of 8; 32 is equivalent to two pages
        const CHUNK_COUNT: usize = 32;
        const HEAP_SIZE: usize = ChunkAllocator::CHUNK_SIZE * CHUNK_COUNT;
        static mut HEAP: StaticAlignedMem<HEAP_SIZE> = StaticAlignedMem::new();
        const BITMAP_SIZE: usize = HEAP_SIZE / ChunkAllocator::CHUNK_SIZE / 8;
        static mut BITMAP: StaticAlignedMem<BITMAP_SIZE> = StaticAlignedMem::new();

        // check that it compiles
        let mut alloc = unsafe { ChunkAllocator::new(HEAP.data_mut(), BITMAP.data_mut()).unwrap() };

        let layout1 = Layout::from_size_align(1, 1).unwrap();
        let layout2 = Layout::from_size_align(4096, 4096).unwrap();

        // allocate 1 single byte
        let ptr1 = {
            unsafe {
                let ptr = alloc.alloc(layout1.clone());
                assert_eq!(
                    ptr as u64 % 4096,
                    0,
                    "the first allocation must be always page-aligned"
                );
                assert!(!alloc.chunk_is_free(0), "the first chunk is taken now!");
                assert!(
                    alloc.chunk_is_free(1),
                    "the second chunk still must be free!"
                );
                ptr
            }
        };

        // allocate 1 page (consumes now the higher half of the available memory)
        let _ptr2 = {
            let ptr;
            unsafe {
                ptr = alloc.alloc(layout2.clone());
                assert_eq!(
                    ptr as u64 % 4096,
                    0,
                    "the second allocation must be page-aligned because this was requested!"
                );
            }
            (0..CHUNK_COUNT)
                .into_iter()
                .skip(CHUNK_COUNT / 2)
                .for_each(|i| {
                    assert!(!alloc.chunk_is_free(i), "chunk must be in use!");
                });
            ptr
        };

        // free the very first allocation; allocate again; now we should have two allocations
        // of two full pages
        {
            unsafe {
                alloc.dealloc(ptr1, layout1);
                let ptr3 = alloc.alloc(layout2);
                assert_eq!(ptr1, ptr3);
            }

            // assert that all chunks are taken
            for i in 0..CHUNK_COUNT {
                assert!(!alloc.chunk_is_free(i), "all chunks must be in use!");
            }
        }
    }
}
