//! Helper functions to manage memory. Most popular/useful exports of this module are
//! - [`PageAligned`]
//! - [`PageAlignedBuf`]
//! - [`PageAlignedByteBuf`]
//! - [`AlignedAlloc`]

use alloc::alloc::alloc;
use alloc::alloc::dealloc;
use core::alloc::{
    AllocError,
    Allocator,
    Layout,
};
use core::ops::{
    Deref,
    DerefMut,
};
use core::ptr::NonNull;
use libhedron::mem::PAGE_SIZE;

/// Helper to create page-aligned data on the stack or in global memory. The data is guaranteed to
/// be page-aligned. Be aware, that properties referenced by the inner data are not necessarily
/// aligned too, for example the heap pointer inside a `Box`. For this, use [`PageAlignedBox`].
///
/// This struct has `Copy` semantics if `T` is `Copy`.
///
/// # Atention for Mutable Global Static Data
/// If you use this as static global variable to be referenced as stack or other writeable memory,
/// make sure to either mark the var as `mut` or manually place it in a writeable section of
/// the ELF file. Otherwise, the page where this is stored lies in a read-only segment.
#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAligned<T>(T);

impl<T> PageAligned<T> {
    /// Constructor.
    pub const fn new(t: T) -> Self {
        Self(t)
    }

    #[cfg(test)]
    const fn self_ptr(&self) -> *const Self {
        self as *const _
    }

    pub const fn data_ptr(&self) -> *const T {
        (&self.0) as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        self.data_ptr() as usize / PAGE_SIZE
    }

    /// Returns the address of this struct. Because this struct is page-aligned,
    /// the address is the address of a page.
    pub fn page_addr(&self) -> usize {
        self.data_ptr() as usize /*& !0xfff not relevant because aligned*/
    }

    /// Returns a reference to the underlying data.
    pub const fn get(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the underlying data.
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Consumes the struct and returns the owned, inner data.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Copy> Copy for PageAligned<T> {}

impl<T> From<T> for PageAligned<T> {
    fn from(data: T) -> Self {
        PageAligned::new(data)
    }
}

impl<T> Deref for PageAligned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for PageAligned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Convenient wrapper around [`PageAlignedData`] for aligned stack-buffers, with exactly
/// the same restrictions and properties.
#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAlignedBuf<T, const N: usize>(PageAligned<[T; N]>);

impl<T: Copy, const N: usize> PageAlignedBuf<T, N> {
    /// Constructor that fills the default element into each index of the slice.
    /// Uses this approach in favor of `Default`, because this works in a const context.
    pub const fn new(default: T) -> Self {
        Self(PageAligned::new([default; N]))
    }
}

impl<T, const N: usize> PageAlignedBuf<T, N> {
    /// Return a pointer to self.
    pub const fn self_ptr(&self) -> *const Self {
        self.0.data_ptr() as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        self.0.page_num()
    }

    /// Returns the page base address of this struct.
    pub fn page_bade_addr(&self) -> usize {
        self.0.page_addr()
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

impl<T, const N: usize> Deref for PageAlignedBuf<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T, const N: usize> DerefMut for PageAlignedBuf<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Convenient alias for [`PageAlignedBuf`].
pub type PageAlignedByteBuf<const N: usize> = PageAlignedBuf<u8, N>;

/// Local allocator that can be used in structs such as `Vec` or `Box`,
/// to enforce correct alignment. Works in situations, where [`PageAligned`]
/// doesn't work well.
///
/// See <https://stackoverflow.com/a/69544158/2891595> for more info.
#[derive(Debug)]
pub struct AlignedAlloc<const N: usize>;

unsafe impl<const N: usize> Allocator for AlignedAlloc<N> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { alloc(layout.align_to(N).unwrap()) };
        let ptr = NonNull::new(ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr.as_ptr(), layout.align_to(N).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::{
        AlignedAlloc,
        PageAligned,
        PageAlignedBuf,
        PAGE_SIZE,
    };
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    #[test]
    fn test_page_aligned() {
        let stack_data = PageAligned::new(0);
        assert_eq!(
            stack_data.data_ptr() as usize % PAGE_SIZE,
            0,
            "must be aligned on the stack"
        );
        assert_eq!(
            stack_data.get() as *const _ as usize % PAGE_SIZE,
            0,
            "must be aligned on the stack"
        );
        assert_eq!(
            stack_data.self_ptr() as usize,
            stack_data.data_ptr() as usize,
            "Rust Compiler must behave as expected and not add any padding to the struct"
        );
        let _inner_data = stack_data.into_inner();

        let heap_data = Box::new(PageAligned::new([0, 1, 2, 3, 4, 5, 6, 7, 8]));
        assert_eq!(
            heap_data.data_ptr() as usize % PAGE_SIZE,
            0,
            "must be aligned on the heap"
        );
        assert_eq!(
            heap_data.get() as *const _ as usize % PAGE_SIZE,
            0,
            "must be aligned on the heap"
        );
        let _inner_data = heap_data.into_inner();
        assert_eq!(
            stack_data.self_ptr() as usize,
            stack_data.data_ptr() as usize,
            "Rust Compiler must behave as expected and not add any padding to the struct"
        );

        // #######################################################################

        let buf = PageAlignedBuf::<_, 1024>::new_zeroed();
        let buf_ptr = (&buf) as *const PageAlignedBuf<_, 1024> as usize;
        assert_eq!(buf_ptr, buf.self_ptr() as usize);
        assert_eq!(buf_ptr % PAGE_SIZE, 0);
    }

    #[test]
    fn test_aligned_alloc() {
        let aligned_box = Box::new_in([1, 2, 3, 4, 5], AlignedAlloc::<PAGE_SIZE>);
        assert_eq!(
            aligned_box.as_ptr() as usize % PAGE_SIZE,
            0,
            "box must be aligned"
        );

        let mut aligned_vec = Vec::with_capacity_in(5, AlignedAlloc::<PAGE_SIZE>);
        aligned_vec.extend_from_slice(&*aligned_box);
        assert_eq!(
            aligned_vec.as_ptr() as usize % PAGE_SIZE,
            0,
            "vec must be aligned"
        );
    }
}
