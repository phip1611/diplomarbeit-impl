use crate::mem::VIRT_MEM_ALLOC;
use core::alloc::Layout;
use core::mem::size_of;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::libhedron::capability::MemCapPermissions;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

pub static ROOT_MEM_MAPPER: SimpleMutex<RootMemMapper> = SimpleMutex::new(RootMemMapper);

type VirtAddr = u64;
type PhysAddr = u64;

/// Type constructed by [`RootMemMapper`] that describes mapped memory by the roottask.
/// The begin addr is guaranteed to be page-aligned.
///
/// See [`RootMemMapper`] for more details.
///
/// Current fast and pragmatic approach: can never be dropped/invalidated
#[derive(Debug, Copy, Clone)]
pub struct MappedMemory {
    /// The original address that we mapped somewhere.
    original_addr: VirtAddr,
    /// The new mapping-destination address.
    new_addr: VirtAddr,
    size_in_pages: usize,
    perm: MemCapPermissions,
}

impl MappedMemory {
    pub fn size(&self) -> usize {
        self.size_in_pages * PAGE_SIZE
    }
    pub fn perm(&self) -> MemCapPermissions {
        self.perm
    }
    pub fn original_addr(&self) -> VirtAddr {
        self.original_addr
    }
    pub fn new_addr(&self) -> VirtAddr {
        self.new_addr
    }
    pub fn size_in_pages(&self) -> usize {
        self.size_in_pages
    }
    /// Returns a pointer to the mapped memory.
    pub fn begin_ptr(&self) -> *const u8 {
        self.new_addr as _
    }
    /// Returns a pointer to the mapped memory.
    pub fn begin_ptr_mut(&self) -> *mut u8 {
        self.new_addr as _
    }

    /// Returns the corresponding address of a old address in the new, mapped region.
    pub fn old_to_new_addr(&self, old_addr: u64) -> u64 {
        assert!(
            old_addr >= self.original_addr,
            "addr {:?} must be bigger than base addr {:?}",
            self.original_addr as *const usize,
            old_addr as *const usize
        );

        let offset = old_addr - self.original_addr;

        assert!(
            offset < self.size() as u64,
            "provided addr {:?} out of memory range",
            old_addr as *const usize,
        );

        let new_addr = self.new_addr + offset;

        log::debug!(
            "old address {:#?} => new {:#?}",
            old_addr as *const usize,
            new_addr as *const usize
        );

        new_addr
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

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as<T: Sized>(&self, offset: Option<usize>) {
        let total_size = size_of::<T>();
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.size() {
            panic!("the memory region is not big enough for the given type T with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.size());
        }
    }

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as_slice<T: Sized>(&self, offset: Option<usize>, length: usize) {
        let total_size = size_of::<T>() * length;
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.size() {
            panic!("the memory region is not big enough for the given type T as slice with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.size());
        }
    }
}

/// Helps the roottask to map memory to a specific location and set the rights in the page-table
/// as desired. Under Hedron, rights never can be upgraded. The work-a-round is that the roottask,
/// self-maps things with the desired rights to a new location (i.e. MEM(R)@0x1000 to MEM(RXW)@0x2000.
///
/// Usecases:
/// - map memory of ELF-File from MB module as RWX into the roottask and load the segments
///   to the PD with the correct rights later
#[derive(Debug)]
pub struct RootMemMapper;

impl RootMemMapper {
    /// Maps the pages from src to a free virtual address inside the roottask. SRC and DEST must be page aligned!
    /// SRC can be VIRT or PHYS addr, because the Hypervisor-flag will be true, which means all
    /// memory is identity mapped!
    #[track_caller]
    pub fn mmap(
        &mut self,
        src: PhysAddr,
        page_count: u64,
        perm: MemCapPermissions,
    ) -> MappedMemory {
        let dest = VIRT_MEM_ALLOC.lock().next_addr(
            Layout::from_size_align(page_count as usize * PAGE_SIZE, PAGE_SIZE).unwrap(),
        );

        assert_eq!(src % PAGE_SIZE as u64, 0, "src addr must be page-aligned");
        assert_eq!(dest % PAGE_SIZE as u64, 0, "dest addr must be page-aligned");
        assert_ne!(src, dest, "src == dest, not allowed! can't upgrade rights");
        assert!(page_count > 0, "page_count must be not null");

        let src_page_num = (src / PAGE_SIZE as u64) as u64;
        let dest_page_num = (dest / PAGE_SIZE as u64) as u64;

        CrdDelegateOptimizer::new(src_page_num, dest_page_num, page_count as usize).mmap(
            RootCapSpace::RootPd.val(),
            RootCapSpace::RootPd.val(),
            perm,
        );

        MappedMemory {
            original_addr: src,
            new_addr: dest,
            size_in_pages: page_count as usize,
            perm,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::MappedMemory;

    #[test]
    fn test_mapped_memory() {
        let mapped_memory = MappedMemory {
            original_addr: 0x1000,
            new_addr: 0x2000,
            size_in_pages: 1,
            perm: Default::default(),
        };
        assert_eq!(mapped_memory.old_to_new_addr(0x1000), 0x2000);
        assert_eq!(mapped_memory.old_to_new_addr(0x1337), 0x2337);
        assert_eq!(mapped_memory.old_to_new_addr(0x1fff), 0x2fff);

        let bytes = [0_u8, 1_u8, 3_u8, 3_u8, 7_u8];

        let mapped_memory = MappedMemory {
            original_addr: 0x1000,
            new_addr: bytes.as_ptr() as u64,
            size_in_pages: 1,
            perm: Default::default(),
        };
        assert_eq!(mapped_memory.mem_as_slice::<u8>(5), bytes);
    }
}
