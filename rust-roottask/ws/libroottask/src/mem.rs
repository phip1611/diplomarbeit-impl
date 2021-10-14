//! Memory utilities only for the roottask.

use alloc::alloc::*;
use core::alloc::Layout;
use core::mem::size_of;
use libhrstd::libhedron::capability::{
    CapSel,
    CrdMem,
    MemCapPermissions,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::syscall::generic::SyscallStatus;
use libhrstd::libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};
use libhrstd::util::crd_bulk::CrdBulkLoopOrderOptimizer;

type Page = [u8; PAGE_SIZE];

/// Helper struct to map memory. It reserves a range inside the current address space on the
/// heap and this can be used as destination buffer for memory mappings (memory delegations).
/// ALERT: All addresses will remain valid but they will point to the new data afterwards. In other
/// words, the memory allocated by this struct holds the destination buffer for memory mappings.
/// By calling `map` the mapping gets executed.
///
/// TODO: After talking with Julian & Nils, this approach is really strange and should not be done,
///  because it changes existing virtual memory (here: the heap) for ever and will lead to
///  overwritten physical memory (for example the mb modules). Instead, my whole memory allocation
///  approach should be refactored so something like:
///    - physical page frame allocator,
///    - virtual address space manager,
///
/// Because my approach is good enough for this work probably, I keep it as it is for now.
#[derive(Debug)]
pub struct MappingHelper<'a> {
    src_mapping_base_address: Option<usize>,
    // - allocated heap memory used as mapping destination
    // - byte array that is a multiple of page size
    // - we must be careful: for pointer arithmetic we need to use `.begin_ptr()` (Byte pointer!)
    // - the reference is valid as long as this instance is valid
    // - we don't need Pin<>, because only owned values are protected by Pin,
    //   but this reference (in the end, a pointer) stays constant during the lifetime of this obj
    // - I use this in favor of Pin<Box<>>, because this way I can guarantee the alignment
    //   for the allocation!
    aligned_mem_pages: &'a mut [Page],
}

impl<'a> MappingHelper<'a> {
    /// Creates a new object and allocates memory on the heap, that
    /// will be the mapping destination.
    pub fn new(page_count: usize) -> Self {
        assert!(page_count > 0, "must use at least one page!");

        let size = page_count * PAGE_SIZE;
        let aligned_mem_pages = unsafe {
            // I use manual alloc/dealloc in favor of Vec or Box, because this way I have
            // full control over the alignment (Page Alignment)

            // this way I can profit from fewer syscalls, because I can use "order"
            // optimization for delegations
            let order = libhrstd::libm::log2((page_count * PAGE_SIZE) as f64);
            let order = libhrstd::libm::trunc(order);
            let alignment = libhrstd::libm::pow(2.0, order) as usize;

            let ptr = alloc(Layout::from_size_align(size, alignment).unwrap());
            let ptr = ptr as *mut Page;
            core::slice::from_raw_parts_mut(ptr, page_count)
        };
        log::debug!(
            "New MappingHelper with backing memory {:?}..{:?} ({} pages)",
            aligned_mem_pages.as_ptr(),
            unsafe { aligned_mem_pages.as_ptr().add(size) },
            page_count
        );
        Self {
            aligned_mem_pages,
            src_mapping_base_address: None,
        }
    }

    /// Performs a `pd_ctrl_delegate` syscall and transfers the given memory
    /// capability (= page number(s)) from the source PD to the dest PD. This
    /// can be used to map physical memory addresses (such as a Multiboot module)
    /// to the virtual address space of the given PD.
    ///
    /// It maps as many pages as the instance was created with in the constructor.
    pub fn map(
        &mut self,
        src_pd: CapSel,
        dest_pd: CapSel,
        src_addr: usize,
        permissions: MemCapPermissions,
        delegate_flags: DelegateFlags,
    ) -> Result<(), SyscallStatus> {
        if self.src_mapping_base_address.is_some() {
            log::debug!("MappingHelper reused!");
        }

        let base_src_page_num = src_addr / PAGE_SIZE;
        let base_dest_page_num = self.aligned_mem_pages.as_ptr() as usize / PAGE_SIZE;

        let base_addr = src_addr & !0xfff;
        self.src_mapping_base_address.replace(base_addr);

        let iterator = CrdBulkLoopOrderOptimizer::new(
            base_src_page_num as u64,
            base_dest_page_num as u64,
            self.aligned_mem_pages.len(),
        );
        for mapping_step in iterator {
            let src_crd = CrdMem::new(
                mapping_step.src_base,
                mapping_step.order,
                // important; permissions must be set for SrcCrd and DestCrd (ask julian: why?)
                permissions,
            );

            let dest_crd = CrdMem::new(
                mapping_step.dest_base,
                mapping_step.order,
                // important; permissions must be set for SrcCrd and DestCrd (ask julian: why?)
                permissions,
            );

            log::debug!(
                "MappingHelper: mapping page {} ({:?}) of pd {} to page {} ({:?}) of pd {} with order={} (2^order={})",
                mapping_step.src_base,
                (mapping_step.src_base as usize * PAGE_SIZE) as *const u64,
                src_pd,
                mapping_step.dest_base,
                (mapping_step.dest_base as usize * PAGE_SIZE) as *const u64,
                dest_pd,
                mapping_step.order,
                mapping_step.power
            );

            // We map all pages in a loop (and don't use the order-field optimization),
            // because overhead here is negligible and simpler code is more important
            pd_ctrl_delegate(src_pd, dest_pd, src_crd, dest_crd, delegate_flags)?;
        }

        Ok(())
    }

    /// Returns the length in bytes of the mapping aea.
    pub fn len(&self) -> usize {
        self.aligned_mem_pages.len() * PAGE_SIZE
    }

    /// Returns the byte pointer to the begin of the data.
    fn begin_ptr(&self) -> *const u8 {
        self.aligned_mem_pages.as_ptr() as *const _
    }

    /// Returns the byte pointer to the begin of the data.
    fn begin_ptr_mut(&mut self) -> *mut u8 {
        self.aligned_mem_pages.as_mut_ptr() as *mut _
    }

    /// Creates a slice of data from the underlying memory of Type T.
    pub fn mem_as_slice<T: Sized>(&self, length: usize) -> &[T] {
        self.mem_with_offset_as_slice(length, None)
    }

    /// Creates a slice of data from the underlying memory of Type T at the
    /// given offset. **The offset is in bytes!**
    pub fn mem_with_offset_as_slice<T: Sized>(&self, length: usize, offset: Option<usize>) -> &[T] {
        self.assert_mem_as_slice::<T>(offset, length);
        unsafe {
            let ptr = self.begin_ptr() as *const _;
            core::slice::from_raw_parts(ptr, length)
        }
    }

    /// Wrapper around [`Self::mem_with_offset_as`].
    pub fn mem_as<T: Sized>(&self) -> &T {
        self.mem_with_offset_as(None)
    }

    /// Wrapper around [`Self::mem_with_offset_as_mut`].
    pub fn mem_as_mut<T: Sized>(&mut self) -> &mut T {
        self.mem_with_offset_as_mut(None)
    }

    /// Wrapper around [`Self::mem_as`].
    pub fn mem_as_ptr<T: Sized>(&self) -> *const T {
        self.mem_as() as *const T
    }

    /// Wrapper around [`Self::mem_as_mut`].
    pub fn mem_as_ptr_mut<T: Sized>(&mut self) -> *mut T {
        self.mem_as_mut() as *mut T
    }

    /// Wrapper around [`Self::mem_with_offset_as`].
    pub fn mem_with_offset_as_ptr<T: Sized>(&self, offset: Option<usize>) -> *const T {
        self.mem_with_offset_as(offset) as *const T
    }

    /// Wrapper around [`Self::mem_with_offset_as_mut`].
    pub fn mem_with_offset_as_ptr_mut<T: Sized>(&mut self, offset: Option<usize>) -> *mut T {
        self.mem_with_offset_as_mut(offset) as *mut T
    }

    /// Helper to interpret the mapped memory at a given address as a special type.
    pub fn mem_with_offset_as<T: Sized>(&self, offset: Option<usize>) -> &T {
        self.assert_mem_as::<T>(offset);
        unsafe {
            self.begin_ptr()
                .add(offset.unwrap_or(0))
                .cast::<T>()
                .as_ref()
        }
        .unwrap()
    }

    /// Helper to interpret the mapped memory at a given address as a special type.
    pub fn mem_with_offset_as_mut<T: Sized>(&mut self, offset: Option<usize>) -> &mut T {
        self.assert_mem_as::<T>(offset);
        unsafe {
            self.begin_ptr_mut()
                .add(offset.unwrap_or(0))
                .cast::<T>()
                .as_mut()
        }
        .unwrap()
    }

    /// Convenient helper. You can input the address that you just mapped
    /// and you get the new address back.
    pub fn old_to_new_addr(&self, old_addr: usize) -> usize {
        let base_addr = self
            .src_mapping_base_address
            .expect("Method only valid if `map()` was called");

        assert!(
            old_addr >= base_addr,
            "addr {:?} must be bigger than base addr {:?}",
            base_addr as *const usize,
            old_addr as *const usize
        );

        let offset = old_addr - base_addr;

        assert!(
            offset <= self.len(),
            "provided addr {:?} out of memory range",
            old_addr as *const usize,
        );

        let new_addr = unsafe { self.begin_ptr().add(offset) as usize };

        log::debug!(
            "old address {:#?} => new {:#?}",
            old_addr as *const usize,
            new_addr as *const usize
        );

        new_addr
    }

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as<T: Sized>(&self, offset: Option<usize>) {
        let total_size = size_of::<T>();
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.len() {
            panic!("the memory region is not big enough for the given type T with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.aligned_mem_pages.len());
        }
    }

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as_slice<T: Sized>(&self, offset: Option<usize>, length: usize) {
        let total_size = size_of::<T>() * length;
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.len() {
            panic!("the memory region is not big enough for the given type T as slice with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.aligned_mem_pages.len());
        }
    }
}

impl<'a> Drop for MappingHelper<'a> {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.aligned_mem_pages.as_mut_ptr() as *mut u8,
                Layout::from_size_align(self.len(), PAGE_SIZE).unwrap(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::MappingHelper;
    use libhrstd::libhedron::mem::PAGE_SIZE;

    #[test]
    fn test_mapping_helper() {
        let foo = MappingHelper::new(2);
        assert_eq!(foo.len(), 2 * PAGE_SIZE);
        assert_eq!(foo.aligned_mem_pages.len(), 2);
        assert_eq!(foo.mem_as_ptr::<u8>() as usize % PAGE_SIZE, 0);
    }

    #[test]
    fn test_mapping_helper_slice() {
        let mut mapping_helper = MappingHelper::new(1);
        let mut original_data = [7_u8; PAGE_SIZE];
        original_data[73] = 73;
        unsafe {
            core::ptr::write(
                mapping_helper.begin_ptr_mut() as *mut [u8; PAGE_SIZE],
                original_data,
            );
        }

        let reference = mapping_helper.mem_as_slice::<u8>(PAGE_SIZE);
        assert_eq!(reference.len(), PAGE_SIZE);
        assert_eq!(reference[0], 7);
        assert_eq!(reference[72], 7);
        assert_eq!(reference[73], 73);
        assert_eq!(reference[74], 7);
    }
}
