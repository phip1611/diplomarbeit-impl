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

/// Helper struct to map memory. It reserves a range inside the current address space on the
/// heap and this can be used as destination buffer. All addresses will remain valid but they
/// will point to the new data afterwards. In other words, the memory allocated by this struct
/// holds the destination buffer for memory mappings. By calling `map` the operation
/// gets commited.
#[derive(Debug)]
pub struct MappingHelper<'a> {
    src_mapping_base_address: Option<usize>,
    // length is a multiple of PAGE_SIZE
    memory: &'a mut [u8],
    order: u8,
}

impl<'a> MappingHelper<'a> {
    /// Creates a new object and prepares the memory/address range.
    /// Similar to CRDs, the order is used to describe a range from
    /// 0..2^order pages.
    ///
    /// Order 0 means: 2^0 pages = 1 page
    /// Order 1 means: 2^1 pages = 2 pages
    pub fn new(page_order: u8) -> Self {
        let pages = 2_usize.pow(page_order as u32);
        let size = pages * PAGE_SIZE;
        let memory = unsafe {
            let ptr = alloc(Layout::from_size_align(size, PAGE_SIZE).unwrap());
            core::slice::from_raw_parts_mut(ptr, size)
        };
        log::debug!(
            "New MappingHelper with backing memory from {:?} to {:?} ({} pages)",
            memory.as_ptr(),
            unsafe { memory.as_ptr().add(size) },
            pages
        );
        Self {
            memory,
            src_mapping_base_address: None,
            order: page_order,
        }
    }

    /// Performs a `pd_ctrl_delegate` syscall and transfers the given memory
    /// capability (= page number) from the source PD to the dest PD. This can be used to map
    /// physical memory addresses (such as a Multiboot module) to
    pub fn map(
        &mut self,
        src_pd: CapSel,
        dest_pd: CapSel,
        src_addr: usize,
        permissions: MemCapPermissions,
        delegate_flags: DelegateFlags,
    ) -> Result<(), SyscallStatus> {
        // TODO ist das so? oder will ich mehrfachnutzung erlauben?
        assert!(
            self.src_mapping_base_address.is_none(),
            "a MappingHelper can only be used once!"
        );

        let src_page_num = src_addr / PAGE_SIZE;
        let dest_page_num = self.memory.as_ptr() as usize / PAGE_SIZE;

        // permissions ignored here
        let src_crd = CrdMem::new(
            src_page_num as u64,
            self.order,
            // important; permissions must be set for SrcCrd and DestCrd (ask julian: why?)
            permissions,
        );

        let dest_crd = CrdMem::new(
            dest_page_num as u64,
            self.order,
            // important; permissions must be set for SrcCrd and DestCrd (ask julian: why?)
            permissions,
        );

        let base_addr = src_addr & !0xfff;
        self.src_mapping_base_address.replace(base_addr);

        log::debug!(
            "MappingHelper: mapping page {} ({:?}) from pd {} to page {} ({:?}) from pd {}",
            src_page_num,
            base_addr as *const usize,
            src_pd,
            dest_page_num,
            self.memory.as_ptr(),
            dest_pd,
        );

        pd_ctrl_delegate(src_pd, dest_pd, src_crd, dest_crd, delegate_flags)
    }

    /// Wrapper around [`mem_with_offset_as`].
    pub fn mem_as<T: Sized>(&self) -> &T {
        self.mem_with_offset_as(None)
    }

    /// Wrapper around [`mem_with_offset_as_mut`].
    pub fn mem_as_mut<T: Sized>(&mut self) -> &mut T {
        self.mem_with_offset_as_mut(None)
    }

    /// Wrapper around [`mem_as`].
    pub fn mem_as_ptr<T: Sized>(&self) -> *const T {
        self.mem_as() as *const T
    }

    /// Wrapper around [`mem_as_mut`].
    pub fn mem_as_ptr_mut<T: Sized>(&mut self) -> *mut T {
        self.mem_as_mut() as *mut T
    }

    /// Wrapper around [`mem_with_offset_as`].
    pub fn mem_with_offset_as_ptr<T: Sized>(&self, offset: Option<usize>) -> *const T {
        self.mem_with_offset_as(offset) as *const T
    }

    /// Wrapper around [`mem_with_offset_as_mut`].
    pub fn mem_with_offset_as_ptr_mut<T: Sized>(&mut self, offset: Option<usize>) -> *mut T {
        self.mem_with_offset_as_mut(offset) as *mut T
    }

    /// Helper to interpret the mapped memory at a given address as a special type.
    pub fn mem_with_offset_as<T: Sized>(&self, offset: Option<usize>) -> &T {
        self.assert_mem_as::<T>(offset);
        unsafe {
            self.memory
                .as_ptr()
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
            self.memory
                .as_mut_ptr()
                .add(offset.unwrap_or(0))
                .cast::<T>()
                .as_mut()
        }
        .unwrap()
    }

    /// Convenient helper. You can input the address that you just mapped
    /// and you get the new address back.
    pub fn old_to_new_addr(&self, addr: usize) -> usize {
        let base_addr = self
            .src_mapping_base_address
            .expect("Method only valid if `map()` was called");

        assert!(
            addr >= base_addr,
            "addr {:?} must be bigger than base addr {:?}",
            base_addr as *const usize,
            addr as *const usize
        );

        let offset = addr - base_addr;

        assert!(
            offset <= self.memory.len(),
            "provided addr {:?} out of memory range",
            addr as *const usize,
        );

        unsafe { self.memory.as_ptr().add(offset) as usize }
    }

    fn assert_mem_as<T: Sized>(&self, offset: Option<usize>) {
        let total_size = size_of::<T>();
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.memory.len() {
            panic!("the memory region is not big enough for the given type T with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.memory.len());
        }
    }
}

impl<'a> Drop for MappingHelper<'a> {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.memory.as_mut_ptr(),
                Layout::from_size_align(self.memory.len(), PAGE_SIZE).unwrap(),
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
        assert_eq!(foo.memory.len(), 4 * PAGE_SIZE);
        assert_eq!(foo.memory.as_ptr() as usize % PAGE_SIZE, 0);
    }
}
