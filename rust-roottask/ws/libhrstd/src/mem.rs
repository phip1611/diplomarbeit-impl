//! Helper functions to manage memory. Most popular/useful exports of this module are
//! - [`PageAlignedData`]
//! - [`PageAlignedByteBuf`]

use libhedron::mem::PAGE_SIZE;

/// Helper to create page-aligned data on the stack or in global memory. The data is guaranteed to
/// be page-aligned.
///
/// This struct has `Copy` semantics if `T` is `Copy`.
///
/// # Atention for Mutable Global Static Data
/// If you use this as static global variable to be referenced as stack or other writeable memory,
/// make sure to either mark the var as `mut` or manually place it in a writeable section of
/// the ELF file. Otherwise, the page where this is stored lies in a read-only segment.
#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAlignedData<T>(T);

impl<T> PageAlignedData<T> {
    /// Constructor.
    pub const fn new(t: T) -> Self {
        Self(t)
    }

    /// Return a pointer to self.
    pub const unsafe fn self_ptr(&self) -> *const Self {
        self as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        unsafe { self.self_ptr() as usize / PAGE_SIZE }
    }

    /// Returns a reference to the underlying data.
    pub const fn get(&self) -> &T {
        &self.0
    }

    /// Returns a reference to the underlying data.
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Copy> Copy for PageAlignedData<T> {}

/// Convenient wrapper around [`PageAlignedData`] for aligned stack-buffers, with exactly
/// the same restrictions and properties.
#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAlignedBuf<T, const N: usize>(PageAlignedData<[T; N]>);

impl<T: Copy, const N: usize> PageAlignedBuf<T, N> {
    /// Constructor that fills the default element into each index of the slice.
    /// Uses this approach in favor of `Default`, because this works in a const context.
    pub const fn new(default: T) -> Self {
        Self(PageAlignedData::new([default; N]))
    }

    /// Return a pointer to self.
    pub const unsafe fn self_ptr(&self) -> *const Self {
        self.0.self_ptr() as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        self.0.page_num()
    }

    /// Returns a reference to the underlying data.
    pub const fn get(&self) -> &[T; N] {
        self.0.get()
    }

    /// Returns a reference to the underlying data.
    pub const fn get_mut(&mut self) -> &mut [T; N] {
        self.0.get_mut()
    }
}

impl<T: Copy, const N: usize> Copy for PageAlignedBuf<T, N> {}

impl<const N: usize> PageAlignedBuf<u8, N> {
    /// New `u8` buffer that is initialized with zeroes.
    pub const fn new_zeroed() -> Self {
        Self::new(0)
    }
}

/// Convenient alias for [`PageAlignedBuf`].
pub type PageAlignedByteBuf<const N: usize> = PageAlignedBuf<u8, N>;

#[cfg(test)]
mod tests {
    use crate::mem::{
        PageAlignedBuf,
        PageAlignedData,
        PAGE_SIZE,
    };

    #[test]
    fn test_page_aligned_data() {
        let data = PageAlignedData::new(0);
        unsafe {
            assert_eq!(data.self_ptr() as usize % PAGE_SIZE, 0, "must be aligned");
        }
        assert_eq!(
            data.get() as *const _ as usize % PAGE_SIZE,
            0,
            "must be aligned"
        );

        let buf = PageAlignedBuf::<_, 1024>::new_zeroed();
        let buf_ptr = (&buf) as *const PageAlignedBuf<_, 1024> as usize;
        unsafe {
            assert_eq!(buf_ptr, buf.self_ptr() as usize);
        }
        assert_eq!(buf_ptr % 4096, 0);
    }
}
