/// Wrapper around a byte array. The type ensures that the byte array
/// is page aligned. Useful to serve as (global) static backing memory
/// for [`super::chunk::ChunkAllocator`]. It is page-aligned because this
/// will fit every situation where this is needed well.
///
/// MAKE SURE THAT THIS IS LANDS IN A WRITEABLE SECTION OF THE ELF FILE!
/// The simplest approach is `static mut MEM: StaticAlignedMem<....> = StaticAlignedMem::new()`.
#[derive(Debug)]
#[repr(align(4096))]
pub struct StaticAlignedMem<const N: usize>([u8; N]);

/*unsafe impl <const N: usize> Send for StaticAlignedMem<N> {}
unsafe impl <const N: usize> Sync for StaticAlignedMem<N> {}*/

impl<const N: usize> StaticAlignedMem<N> {
    pub const fn new() -> Self {
        Self([0; N])
    }

    /// Returns a mutable reference to the data.
    pub const fn data_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::static_alloc::static_aligned_mem::StaticAlignedMem;

    static mut TEST_MEM: StaticAlignedMem<4096> = StaticAlignedMem::new();

    #[test]
    fn test_aligned() {
        assert_eq!(unsafe { TEST_MEM.data_mut() }.as_mut_ptr() as u64 % 1024, 0);
    }
}
