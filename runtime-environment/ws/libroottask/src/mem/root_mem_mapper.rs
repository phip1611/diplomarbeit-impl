use crate::mem::VIRT_MEM_ALLOC;
use crate::process::Process;
use alloc::rc::{
    Rc,
    Weak,
};
use core::alloc::Layout;
use core::mem::size_of;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// Public instance of the root task memory mapper.
pub static ROOT_MEM_MAPPER: SimpleMutex<RootMemMapper> = SimpleMutex::new(RootMemMapper);

type Address = u64;

/// Type constructed by [`RootMemMapper`] that describes mapped memory by the roottask.
/// Mappings always begin at a page-aligned address.
///
/// See [`RootMemMapper`] for more details.
///
/// Current Q&D approach: can never be dropped/invalidated.
/// TODO: remove Clone; add drop trait
///
/// TODO unify with the MemoryMapping struct used in the process module
#[derive(Debug, Clone)]
pub struct MappedMemory {
    /// The origin of the mapping.
    origin_process: Weak<Process>,
    /// The destination process of the mapping.
    to_process: Weak<Process>,
    /// The original address in the address space of the origin that we mapped to the target.
    original_addr: Address,
    /// The new mapping-destination address in the address space of the target.
    mapped_addr: Address,
    /// Size of the mapping in pages.
    size_in_pages: u64,
    /// Rights of the memory mapping.
    perm: MemCapPermissions,
}

impl MappedMemory {
    /// Size of the mapping in bytes.
    pub fn size(&self) -> u64 {
        self.size_in_pages * PAGE_SIZE as u64
    }
    /// Permissions of the mapping.
    pub fn perm(&self) -> MemCapPermissions {
        self.perm
    }
    /// The original address in the address space of the origin.
    pub fn original_addr(&self) -> Address {
        self.original_addr
    }
    /// The new address in the address space of the code that executes this.
    pub fn mapped_addr(&self) -> Address {
        self.mapped_addr
    }
    /// Size of the mapping in pages.
    pub fn size_in_pages(&self) -> u64 {
        self.size_in_pages
    }
    /// Returns a pointer to the mapped memory in the address space of the caller.
    pub fn begin_ptr(&self) -> *const u8 {
        self.mapped_addr as _
    }
    /// Returns a mut pointer to the mapped memory in the address space of the caller.
    pub fn begin_ptr_mut(&self) -> *mut u8 {
        self.mapped_addr as _
    }

    pub fn origin_process(&self) -> &Weak<Process> {
        &self.origin_process
    }
    pub fn to_process(&self) -> &Weak<Process> {
        &self.to_process
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

        let new_addr = self.mapped_addr + offset;

        log::debug!(
            "old address {:#?} => new {:#?}",
            old_addr as *const usize,
            new_addr as *const usize
        );

        new_addr
    }

    /// Like [`Self::old_to_new_addr`] but with pointers.
    pub fn old_to_new_ptr(&self, old_ptr: *const u8) -> *const u8 {
        self.old_to_new_addr(old_ptr as u64) as *const u8
    }

    /// Like [`Self::old_to_new_addr`] but mutable pointers.
    pub fn old_to_new_ptr_mut(&self, old_ptr: *mut u8) -> *mut u8 {
        self.old_to_new_addr(old_ptr as u64) as *mut u8
    }

    /// Creates a slice of data from the underlying memory of Type T.
    pub fn mem_as_slice<T: Sized>(&self, length: usize) -> &[T] {
        self.mem_with_offset_as_slice(length, 0)
    }

    /// Creates a slice of data from the underlying memory of Type T at the
    /// given offset. **The offset is in bytes!**
    pub fn mem_with_offset_as_slice<T: Sized>(&self, length: usize, offset: usize) -> &[T] {
        self.assert_mem_as_slice::<T>(Some(offset), length);
        unsafe {
            let ptr = self.begin_ptr().add(offset).cast();
            core::slice::from_raw_parts(ptr, length)
        }
    }

    /// Wrapper around [`Self::mem_with_offset_as`].
    pub fn mem_as<T: Sized>(&self) -> &T {
        self.mem_with_offset_as(0)
    }

    /// Wrapper around [`Self::mem_with_offset_as_mut`].
    pub fn mem_as_mut<T: Sized>(&mut self) -> &mut T {
        self.mem_with_offset_as_mut(0)
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
    pub fn mem_with_offset_as_ptr<T: Sized>(&self, offset: usize) -> *const T {
        self.mem_with_offset_as(offset) as *const T
    }

    /// Wrapper around [`Self::mem_with_offset_as_mut`].
    pub fn mem_with_offset_as_ptr_mut<T: Sized>(&mut self, offset: usize) -> *mut T {
        self.mem_with_offset_as_mut(offset) as *mut T
    }

    /// Helper to interpret the mapped memory at a given address as a special type.
    pub fn mem_with_offset_as<T: Sized>(&self, offset: usize) -> &T {
        self.assert_mem_as::<T>(Some(offset));
        unsafe { self.begin_ptr().add(offset).cast::<T>().as_ref() }.unwrap()
    }

    /// Helper to interpret the mapped memory at a given address as a special type.
    pub fn mem_with_offset_as_mut<T: Sized>(&mut self, offset: usize) -> &mut T {
        self.assert_mem_as::<T>(Some(offset));
        unsafe { self.begin_ptr_mut().add(offset).cast::<T>().as_mut() }.unwrap()
    }

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as<T: Sized>(&self, offset: Option<usize>) {
        let total_size = size_of::<T>();
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.size() as usize {
            panic!("the memory region is not big enough for the given type T with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.size() as usize);
        }
    }

    /// Common assertion method for `mem_*_as*`-functions.
    fn assert_mem_as_slice<T: Sized>(&self, offset: Option<usize>, length: usize) {
        let total_size = size_of::<T>() * length;
        let offset = offset.unwrap_or(0);
        if total_size + offset > self.size() as usize {
            panic!("the memory region is not big enough for the given type T as slice with size {} at offset {}. Needs {} more bytes",
                   total_size, offset, total_size + offset - self.size() as usize);
        }
    }
}

// TODO remove "Clone"; add drop
/*impl Drop for MappedMemory {
    fn drop(&mut self) {
        log::debug!("Drop not implemented for MappedMemory yet");
    }
}*/

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
    /// High level wrapper around [`CrdDelegateOptimizer::mmap`]. Maps memory from the origin
    /// process to the caller process. If origin==to==Roottask, then the Hypervisor
    /// flag will be set and all addresses will be treated as physical addresses, because
    /// the memory will be identity mapped.
    #[track_caller]
    pub fn mmap(
        // Required to force the use of a lock when this is used in a static variable.
        &mut self,
        src_process: &Rc<Process>,
        dest_process: &Rc<Process>,
        src_addr: Address,
        preferred_dest_addr: Option<Address>,
        page_count: u64,
        perm: MemCapPermissions,
    ) -> MappedMemory {
        assert!(page_count > 0, "page_count must be not null");
        assert_eq!(
            src_addr % PAGE_SIZE as u64,
            0,
            "src addr must be page-aligned"
        );

        if let Some(preferred_dest_addr) = preferred_dest_addr {
            assert_eq!(
                preferred_dest_addr % PAGE_SIZE as u64,
                0,
                "dest addr must be page-aligned"
            );
        }

        let dest_addr = preferred_dest_addr.unwrap_or_else(|| {
            // next power of two; this will accelerate memory delegations because the
            // Crd order optimization is applicable
            let align = (page_count as usize * PAGE_SIZE).next_power_of_two();
            VIRT_MEM_ALLOC.lock().next_addr(
                // optimize alignment for faster delegate calls (use Crd order optimization)
                Layout::from_size_align(page_count as usize * PAGE_SIZE, align).unwrap(),
            )
        });

        if src_process == dest_process {
            assert_ne!(
                src_addr, dest_addr,
                "src == dest, not allowed! can't upgrade rights in Hedron this way"
            );
        }

        let src_page_num = src_addr / PAGE_SIZE as u64;
        let dest_page_num = dest_addr / PAGE_SIZE as u64;

        CrdDelegateOptimizer::new(src_page_num, dest_page_num, page_count as usize).mmap(
            src_process.pd_obj().cap_sel(),
            dest_process.pd_obj().cap_sel(),
            perm,
        );

        MappedMemory {
            origin_process: Rc::downgrade(src_process),
            to_process: Rc::downgrade(dest_process),
            original_addr: src_addr,
            mapped_addr: dest_addr,
            size_in_pages: page_count,
            perm,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::MappedMemory;
    use crate::process::Process;
    use alloc::rc::Rc;

    #[test]
    fn test_mapped_memory() {
        // some arbitrary values juts to create the object
        let root = Process::root(4096, 4096);
        let root = Rc::from(root);

        let mapped_memory = MappedMemory {
            origin_process: Rc::downgrade(&root),
            to_process: Rc::downgrade(&root),
            original_addr: 0x1000,
            mapped_addr: 0x2000,
            size_in_pages: 1,
            perm: Default::default(),
        };
        assert_eq!(mapped_memory.old_to_new_addr(0x1000), 0x2000);
        assert_eq!(mapped_memory.old_to_new_addr(0x1337), 0x2337);
        assert_eq!(mapped_memory.old_to_new_addr(0x1fff), 0x2fff);

        let bytes = [0_u8, 1_u8, 3_u8, 3_u8, 7_u8];

        let mapped_memory = MappedMemory {
            origin_process: Rc::downgrade(&root),
            to_process: Rc::downgrade(&root),
            original_addr: 0x1000,
            mapped_addr: bytes.as_ptr() as u64,
            size_in_pages: 1,
            perm: Default::default(),
        };
        assert_eq!(mapped_memory.mem_as_slice::<u8>(5), bytes);
    }
}
