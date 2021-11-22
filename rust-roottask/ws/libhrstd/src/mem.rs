//! Helper functions to manage memory. Most popular/useful exports of this module are
//! - [`PageAligned`]
//! - [`PageAlignedBuf`]
//! - [`PageAlignedByteBuf`]
//! - [`AlignedAlloc`]

use alloc::alloc::alloc;
use alloc::alloc::dealloc;
use alloc::vec::Vec;
use core::alloc::{
    AllocError,
    Allocator,
    Layout,
};
use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{
    Deref,
    DerefMut,
};
use core::ptr::NonNull;
use libhedron::ipc_serde::{
    Deserialize as DeriveDeserialize,
    Serialize as DeriveSerialize,
    Serialize,
};
use libhedron::mem::PAGE_SIZE;
use libhedron::utcb::UTCB_DATA_CAPACITY;

/// Calculates the number of needed pages to cover all bytes.
pub const fn calc_page_count(size: u64) -> u64 {
    if size % PAGE_SIZE as u64 == 0 {
        size / PAGE_SIZE as u64
    } else {
        (size / PAGE_SIZE as u64) + 1
    }
}

/// Wrapping struct that acts as a smart pointer to align owned data. Can be used to align data
/// on the stack, the heap (`Box<PageAligned<T>>`), or global static memory.
/// **BE AWARE** that this doesn't work for situations, like `PageAligned<Vec<...>>`.
/// `PageAligned<Vec<...>>` only aligns the managing structure of the Vector, but not the heap data
/// the vector allocates internally. For this, `AlignedAlloc` might be a better option.
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
    /// Constructor that takes ownership of the data. The data is guaranteed to be aligned.
    pub const fn new(t: T) -> Self {
        Self(t)
    }

    #[cfg(test)]
    const fn self_ptr(&self) -> *const Self {
        self as *const _
    }

    /// Returns the pointer to the data. The pointer is the address of a page, because
    /// the data is page-aligned.
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

/// Convenient wrapper around [`PageAligned`] for aligned stack-buffers, with exactly
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

/// Version of [`AlignedAlloc`], that works without const generics. Const generics
/// have to many bugs yet for this use case, including but not limited
/// to https://github.com/rust-lang/rust/issues/81698.
#[derive(Debug)]
pub struct PageAlignedAlloc;

unsafe impl Allocator for PageAlignedAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { alloc(layout.align_to(PAGE_SIZE).unwrap()) };
        let ptr = NonNull::new(ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr.as_ptr(), layout.align_to(PAGE_SIZE).unwrap());
    }
}

/// Page-aligned array on the heap, that ensures that the heap memory stays the same
/// throughout the lifetime of this object (pinned). The size is fixed.
///
/// Used in favor of `Box::new([0; USER_STACK_SIZE`), because Box copies memory first
/// to the stack and then to the heap (even in release build). I want to avoid stack
/// overflows, therefore the dedicated abstraction.
#[derive(Debug)]
pub struct PinnedPageAlignedHeapArray<T: Copy + Debug> {
    ptr: *mut T,
    len: usize,
    layout: Layout,
}

impl<T: Copy + Debug> PinnedPageAlignedHeapArray<T> {
    /// This is similar to `Box::new([0; 1024])` with the exception, that the
    /// array is not created on the stack at first but directly on the heap.
    pub fn new(fill_elem: T, len: usize) -> Self {
        assert!(len > 0, "length must be > 0");
        let total_size = size_of::<T>() * len;
        let layout = Layout::from_size_align(total_size, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc(layout.clone()) } as *mut T;
        assert!(!ptr.is_null());

        unsafe {
            for i in 0..len {
                core::ptr::write(ptr.add(i), fill_elem);
            }
        }

        Self { ptr, len, layout }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &[T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Returns the pointer to the begin of the array on the heap. The pointer is guaranteed
    /// to be page-aligned.
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T: Copy + Debug> Drop for PinnedPageAlignedHeapArray<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr.cast(), self.layout) }
    }
}

/// Used to transfer data through service portals either
/// via a user ptr or via embedded content. Data can be embedded,
/// if the data is less than [`UTCB_DATA_CAPACITY`] bytes long.
#[derive(Debug, DeriveSerialize, DeriveDeserialize)]
pub enum UserPtrOrEmbedded<T: Serialize + Clone> {
    // usize because raw ptrs are not serializable
    Ptr(usize),
    Embedded(T),
    EmbeddedSlice(Vec<T>),
}

impl<T: Serialize + Clone> UserPtrOrEmbedded<T> {
    // -2: "postcard" needs additional info for slices for example!
    const CAPACITY: usize = UTCB_DATA_CAPACITY - size_of::<Self>();
    const VEC_CAPACITY: usize = Self::CAPACITY - size_of::<Vec<T>>();

    /// Constructor.
    pub fn new(data: T) -> Self {
        if size_of::<T>() <= Self::CAPACITY {
            Self::Embedded(data)
        } else {
            Self::Ptr(&data as *const _ as usize)
        }
    }

    pub fn new_slice(data: &[T]) -> Self {
        let size_t = size_of::<T>();
        let size = size_t * data.len();
        if size <= Self::VEC_CAPACITY {
            Self::EmbeddedSlice(data.to_vec())
        } else {
            Self::Ptr(data.as_ptr() as usize)
        }
    }

    pub fn ptr(&self) -> *const T {
        self.ptr_mut() as *const _
    }

    #[track_caller]
    pub fn ptr_mut(&self) -> *mut T {
        match self {
            UserPtrOrEmbedded::Ptr(ptr) => *ptr as *mut _,
            _ => panic!("invalid type"),
        }
    }

    #[track_caller]
    pub fn embedded(&self) -> &T {
        match self {
            UserPtrOrEmbedded::Embedded(val) => val,
            _ => panic!("invalid type"),
        }
    }

    #[track_caller]
    pub fn embedded_slice(&self) -> &[T] {
        match self {
            UserPtrOrEmbedded::EmbeddedSlice(val) => val.as_slice(),
            _ => panic!("invalid type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::{
        AlignedAlloc,
        PageAligned,
        PageAlignedBuf,
        PAGE_SIZE,
    };
    use crate::uaddress_space::USER_STACK_SIZE;
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::pin::Pin;

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

        let aligned = Pin::new(PageAligned::from([1, 2, 3]));
        assert_eq!(
            (&aligned) as *const _ as usize % PAGE_SIZE,
            0,
            "pinned version must also be aligned"
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

    #[test]
    fn test_pinned_page_aligned_heap_array() {
        let stack = PinnedPageAlignedHeapArray::new(0_u8, USER_STACK_SIZE);
        assert_eq!(
            stack.as_ptr() as usize % PAGE_SIZE,
            0,
            "must be page aligned"
        );
        for i in 0..USER_STACK_SIZE {
            assert_eq!(stack.as_slice()[i], 0);
        }

        let stack = PinnedPageAlignedHeapArray::new(7_u8, USER_STACK_SIZE);
        assert_eq!(
            stack.as_ptr() as usize % PAGE_SIZE,
            0,
            "must be page aligned"
        );
        for i in 0..USER_STACK_SIZE {
            assert_eq!(stack.as_slice()[i], 7);
        }
    }

    #[test]
    fn test_user_ptr_or_embedded() {
        let data_small = [0_u8; 2048];
        let data_big = [0_u8; 4096];
        let data_big_ptr = data_big.as_ptr() as usize;

        let usptr_or_embedded_small = UserPtrOrEmbedded::new_slice(&data_small);
        assert_eq!(
            usptr_or_embedded_small.embedded_slice(),
            UserPtrOrEmbedded::EmbeddedSlice(data_small.to_vec()).embedded_slice(),
        );
        let usptr_or_embedded_big = UserPtrOrEmbedded::new_slice(&data_big);
        assert_eq!(
            usptr_or_embedded_big.ptr(),
            UserPtrOrEmbedded::Ptr(data_big_ptr).ptr()
        );

        // now test that everything is serializable as expected

        let mut serialized_small = [0; UTCB_DATA_CAPACITY];
        libhedron::ipc_postcard::to_slice(&usptr_or_embedded_small, &mut serialized_small).unwrap();
        let deserialized_small =
            libhedron::ipc_postcard::from_bytes::<UserPtrOrEmbedded<u8>>(&serialized_small)
                .unwrap();
        assert_eq!(
            usptr_or_embedded_small.embedded_slice(),
            deserialized_small.embedded_slice()
        );

        let mut serialized_big = [0; UTCB_DATA_CAPACITY];
        libhedron::ipc_postcard::to_slice(&usptr_or_embedded_big, &mut serialized_big).unwrap();
        let deserialized_small =
            libhedron::ipc_postcard::from_bytes::<UserPtrOrEmbedded<u8>>(&serialized_big).unwrap();
        assert_eq!(
            deserialized_small.ptr(),
            UserPtrOrEmbedded::Ptr(data_big_ptr).ptr()
        );
    }
}
