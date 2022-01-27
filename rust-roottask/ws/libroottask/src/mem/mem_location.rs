use alloc::boxed::Box;
use core::fmt::Debug;
use core::mem::size_of;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::Utcb;
use libhrstd::mem::PinnedPageAlignedHeapArray;

/// Trait for owned data in [`MemLocation`].
pub trait MemLocationOwned {
    /// Returns the pointer to the begin of the data (which is page-aligned).
    fn page_ptr(&self) -> *const u8;
    /// Returns the number of the page.
    fn page_num(&self) -> u64;
    /// Returns the size of selth.
    fn size(&self) -> usize
    where
        Self: Sized,
    {
        size_of::<Self>()
    }
}

impl<T: core::alloc::Allocator> MemLocationOwned for Box<Utcb, T> {
    fn page_ptr(&self) -> *const u8 {
        let ptr = Utcb::self_ptr(self);
        debug_assert_eq!(ptr as usize % PAGE_SIZE, 0, "must be page aligned!");
        ptr.cast()
    }

    fn page_num(&self) -> u64 {
        Utcb::page_num(self)
    }
}

impl<T: Copy + Debug> MemLocationOwned for PinnedPageAlignedHeapArray<T> {
    fn page_ptr(&self) -> *const u8 {
        PinnedPageAlignedHeapArray::as_ptr(self) as *const u8
    }

    fn page_num(&self) -> u64 {
        self.as_ptr() as u64 / PAGE_SIZE as u64
    }

    fn size(&self) -> usize {
        self.len()
    }
}

/// Abstraction over the location of page-aligned memory. Relevant because Heap and Utcb
/// can be owned on the heap (for non-root processes) or be provided externally
/// (roottask).
#[derive(Debug, PartialEq)]
pub enum MemLocation<T: MemLocationOwned> {
    /// The data behind T MUST be page-aligned.
    Owned(T),
    /// Address of the external UTCB (for the roottask).
    External { page_num: u64, size_in_pages: u64 },
}

impl<T: MemLocationOwned> MemLocation<T> {
    /// Convenient constructor for [`PageAlignedMemLocation::External`].
    pub fn new_external(page_num: u64, size_bytes: u64) -> Self {
        let size_in_pages = if size_bytes % PAGE_SIZE as u64 == 0 {
            size_bytes / PAGE_SIZE as u64
        } else {
            (size_bytes / PAGE_SIZE as u64) + 1
        };
        Self::External {
            page_num,
            size_in_pages,
        }
    }

    /// Returns a page-aligned pointer to the underlying data.
    pub fn mem_ptr(&self) -> *const T {
        let ptr = match self {
            Self::Owned(data) => MemLocationOwned::page_ptr(data).cast(),
            Self::External { page_num, .. } => (*page_num * PAGE_SIZE as u64) as *const T,
        };
        debug_assert_eq!(ptr as usize % PAGE_SIZE, 0, "must be page aligned!");
        ptr
    }

    /// Returns the page number of the underlying memory.
    pub fn page_num(&self) -> u64 {
        match self {
            MemLocation::Owned(data) => MemLocationOwned::page_num(data),
            MemLocation::External { page_num, .. } => *page_num,
        }
    }

    /// Returns the size in pages.
    pub fn size_in_pages(&self) -> u64 {
        match self {
            MemLocation::Owned(data) => {
                let size = data.size();
                (if size % PAGE_SIZE == 0 {
                    size / PAGE_SIZE
                } else {
                    (size / PAGE_SIZE) + 1
                }) as u64
            }
            MemLocation::External {
                page_num: _page_num,
                size_in_pages,
            } => *size_in_pages,
        }
    }
}

impl MemLocation<PinnedPageAlignedHeapArray<u8>> {
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        match self {
            MemLocation::Owned(val) => val.as_slice_mut(),
            MemLocation::External {
                page_num,
                size_in_pages,
            } => unsafe {
                core::slice::from_raw_parts_mut(
                    (*page_num * PAGE_SIZE as u64) as *mut u8,
                    (*size_in_pages) as usize,
                )
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libhrstd::libhedron::mem::PAGE_SIZE;
    use libhrstd::mem::PinnedPageAlignedHeapArray;
    use libhrstd::uaddress_space::USER_STACK_SIZE;

    #[test]
    fn test_page_aligned_mem_location() {
        let mem = MemLocation::Owned(PinnedPageAlignedHeapArray::new(0_u8, USER_STACK_SIZE));
        assert_eq!(mem.size_in_pages() as usize, USER_STACK_SIZE / PAGE_SIZE);
        assert!(mem.page_num() > 0);
    }
}
