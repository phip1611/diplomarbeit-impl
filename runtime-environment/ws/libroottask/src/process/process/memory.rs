use crate::process::Process;
use alloc::alloc::{
    Allocator,
    Global,
    Layout,
};
use alloc::collections::BTreeMap;
use core::cmp::Ordering;
use core::ptr::NonNull;
use elf_rs::{
    Elf,
    ElfFile,
    ProgramHeaderWrapper,
    ProgramType,
};
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::mem::calc_page_count;
use libhrstd::uaddress_space::{
    USER_STACK_BOTTOM_ADDR,
    USER_STACK_BOTTOM_PAGE_NUM,
    USER_STACK_SIZE,
};
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

/// Wrapper around `u64` that ensures that the inner value is a page address.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PageAddress(u64);

impl PageAddress {
    #[track_caller]
    pub fn new(val: u64) -> Self {
        assert_eq!(val as usize % PAGE_SIZE, 0, "must be page address");
        Self(val)
    }

    pub const fn val(self) -> u64 {
        self.0
    }
}

/// Structure that knows about the memory usage/layout of a process. This helps to identify
/// and manage the heap. Performs memory mappings.
///
///
/// TODO unify with the MappedMemory struct used in the roottask mapping mechanism
#[derive(Debug)]
pub struct ProcessMemoryManager {
    init: bool,
    /// Used for program break heap mechanism. Tells the beginning of the program break
    /// in the address space of the user.
    u_program_break_begin: PageAddress,
    /// Tells how many memory is mapped as read/write for usage as heap into the program
    /// regarding the beginning of the program break in the address space of the user.
    /// Never less than `program_break_begin`.
    u_program_break_current: PageAddress,
    /// Contains memory mappings for the ELF segments.
    elf_mappings: BTreeMap<PageAddress, MemoryMapping>,
    /// Contains the memory mappings for the stack.
    stack: Option<MemoryMapping>,
    /// Contains all additional memory mappings  This includes heap mappings from mmap() calls for
    /// example from Linux programs.
    memory_mappings: BTreeMap<PageAddress, MemoryMapping>,
    /// The next virtual memory address for a mmap mapping. Right now this grows until
    /// infinity (TODO!).
    u_next_mmap_addr: u64,
}

impl ProcessMemoryManager {
    /// The maximum memory break.
    pub const MEMORY_BREAK_MAX: usize = 0x40000000;

    /// Constructor. Saves the area used for the stack and the program break inside the structure.
    pub fn new(process: &Process) -> Self {
        let u_program_break_begin = Self::get_program_break_begin(process.elf_file_bytes());

        Self {
            init: false,
            u_program_break_begin,
            u_program_break_current: u_program_break_begin,
            u_next_mmap_addr: u_program_break_begin.val() + Self::MEMORY_BREAK_MAX as u64,
            elf_mappings: Default::default(),
            stack: None,
            memory_mappings: BTreeMap::new(),
        }
    }

    /// Determines the page-aligned begin of the program break used for the heap.
    fn get_program_break_begin(elf_bytes: &[u8]) -> PageAddress {
        let elf = elf_rs::Elf::from_bytes(elf_bytes).unwrap();

        // the maximum virtual address used by a program
        let elf_max_addr = elf
            .program_header_iter()
            .map(|hdr| hdr.vaddr() + hdr.memsz())
            .max()
            .unwrap();

        let page_offset = elf_max_addr & 0xfff;

        let program_break_begin = elf_max_addr + PAGE_SIZE as u64 - page_offset;

        PageAddress(program_break_begin)
    }

    /// Initializes the stack, the elf segments, and the heap for an application. Performs
    /// memory mappings/page table manipulations.
    pub fn init(&mut self, process: &Process) -> Result<(), ()> {
        assert!(!self.init, "init only permitted once!");
        self.init = true;

        self.init_stack(process).unwrap();
        self.init_elf_load_segments(process).unwrap();

        Ok(())
    }

    /// Initializes the stack and maps it to the user address space.
    fn init_stack(&mut self, process: &Process) -> Result<(), ()> {
        assert_eq!(
            USER_STACK_SIZE % PAGE_SIZE,
            0,
            "STACK-Size must be a multiple of PAGE_SIZE."
        );
        let r_layout = Layout::from_size_align(USER_STACK_SIZE, PAGE_SIZE).unwrap();
        let r_stack: NonNull<[u8]> = Global.allocate_zeroed(r_layout).unwrap();
        let r_stack = r_stack.as_ptr().as_mut_ptr() as u64;
        let stack_page_count = USER_STACK_SIZE / PAGE_SIZE;

        let r_stack_bottom_page_num = r_stack / PAGE_SIZE as u64;

        CrdDelegateOptimizer::new(
            r_stack_bottom_page_num,
            USER_STACK_BOTTOM_PAGE_NUM,
            stack_page_count,
        )
        .mmap(
            process.parent().unwrap().pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE,
        );

        let stack = MemoryMapping::new(
            PageAddress::new(r_stack),
            r_layout,
            PageAddress::new(USER_STACK_BOTTOM_ADDR),
            stack_page_count,
            MemoryKind::Stack,
            MemCapPermissions::RW,
        );

        self.stack.replace(stack);

        // TODO last stack page without read or write permissions! => detect page fault

        Ok(())
    }

    /// Maps the load elf segments to the user address space. If necessary,
    /// allocates additional memory from the heap for BSS (filesize != memsize in elf)
    fn init_elf_load_segments(&mut self, process: &Process) -> Result<(), ()> {
        let elf = Elf::from_bytes(process.elf_file_bytes()).unwrap();

        // log::debug!("ELF: {:#?}", elf64.header());
        log::debug!("mapping mem for all load segments to new PD");
        for segment in elf
            .program_header_iter()
            .filter(|pr_hrd| pr_hrd.ph_type() == ProgramType::LOAD)
        {
            log::trace!("next segment");
            assert_eq!(
                segment.align() as usize,
                PAGE_SIZE,
                "expects that all segments are page aligned inside the file!!"
            );
            if segment.memsz() == segment.filesz() {
                self.init_elf_load_segments__direct(&segment, process)?;
            } else {
                self.init_elf_load_segments__indirect(&segment, process)?;
            }
        }

        Ok(())
    }

    /// Maps a single load segment directly into the user address space.
    #[allow(non_snake_case)]
    fn init_elf_load_segments__direct(
        &mut self,
        segment: &ProgramHeaderWrapper,
        process: &Process,
    ) -> Result<(), ()> {
        assert_eq!(segment.offset() % PAGE_SIZE as u64, 0);
        // mem in roottask: pointer/page into address space of the roottask
        let load_segment_src_page_num = segment.content().as_ptr() as usize / PAGE_SIZE;
        // virt mem in dest PD / address space
        let load_segment_dest_page_num = segment.vaddr() as usize / PAGE_SIZE;

        // number of pages to map
        let num_pages = calc_page_count(segment.filesz() as usize);

        CrdDelegateOptimizer::new(
            load_segment_src_page_num as u64,
            load_segment_dest_page_num as u64,
            num_pages,
        )
        .mmap(
            RootCapSpace::RootPd.val(),
            process.pd_obj().cap_sel(),
            // works because Hedron and ELF use the same bits for RWX
            MemCapPermissions::from_elf_segment_permissions(segment.flags().bits() as u8),
        );

        Ok(())
    }

    /// Maps a single load segment indirectly into the user address space.
    /// This means, it allocates additional memory on the roottask heap
    /// and this is what gets mapped to the user.
    #[allow(non_snake_case)]
    fn init_elf_load_segments__indirect(
        &mut self,
        segment: &ProgramHeaderWrapper,
        process: &Process,
    ) -> Result<(), ()> {
        // memsize != file size
        // I can't map the ELF load segment directly

        // offset of load segment in first page (segment might not start at page aligned address)
        let first_page_offset = segment.offset() & 0xfff;
        // the total number we need in bytes (we always need to start at a page)
        let total_size = first_page_offset + segment.memsz();
        // how many pages we need
        let page_count = calc_page_count(total_size as usize);

        // TODO this will never be freed.. Q&D
        // roottask pointer that holds the elf segment (page aligned)
        let r_elf_segment_layout =
            Layout::from_size_align(page_count as usize * PAGE_SIZE, PAGE_SIZE).unwrap();
        let r_elf_segment_ptr: NonNull<[u8]> =
            Global.allocate_zeroed(r_elf_segment_layout).unwrap();

        let u_mem_permissions =
            MemCapPermissions::from_elf_segment_permissions(segment.flags().bits() as u8);

        let memory_mapping = MemoryMapping::new(
            PageAddress::new(r_elf_segment_ptr.as_mut_ptr() as u64),
            r_elf_segment_layout,
            PageAddress::new(segment.vaddr() & !0xfff),
            page_count,
            MemoryKind::Elf,
            u_mem_permissions,
        );
        self.elf_mappings
            .insert(memory_mapping.u_address, memory_mapping);

        // copy everything from the ELF file to the new memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                segment.content().as_ptr(),
                r_elf_segment_ptr
                    .as_ptr()
                    .cast::<u8>()
                    .add(first_page_offset as usize),
                segment.filesz() as usize,
            );
        }

        // mem in roottask: pointer/page into address space of the roottask
        let load_segment_src_page_num = r_elf_segment_ptr.as_mut_ptr() as usize / PAGE_SIZE;
        // virt mem in dest PD / address space
        let load_segment_dest_page_num = segment.vaddr() as usize / PAGE_SIZE;

        CrdDelegateOptimizer::new(
            load_segment_src_page_num as u64,
            load_segment_dest_page_num as u64,
            page_count as usize,
        )
        .mmap(
            RootCapSpace::RootPd.val(),
            process.pd_obj().cap_sel(),
            u_mem_permissions,
        );

        Ok(())
    }

    /// Increases the program break by providing either null or an address. This is similar to
    /// how Linux handles the program break. In Linux a program performs an initial BRK(NULL)
    /// call to find the program break beginning. Afterwards, it sends the `break + additional_len`
    /// address to the Kernel.
    ///
    /// Performs memory mappings.
    ///
    /// Address must be a page address.
    ///
    /// Returns the new current break on success. Returns the begin of the break if
    /// the provided address is zero.
    pub fn increase_break(&mut self, address: u64, process: &Process) -> u64 {
        if address == 0 {
            return self.u_program_break_current.val();
        }
        assert!(
            address > self.u_program_break_current.val(),
            "new address must be bigger than program break! brk=0x{:x}, address=0x{address:x}",
            self.u_program_break_current.val()
        );
        let address = PageAddress::new(address);
        let growth = address.val() - self.u_program_break_current.val();
        log::trace!(
            "increase_break: old_brk=0x{old_brk:x}, address=0x{address:x}, growth={growth:x}",
            old_brk = self.u_program_break_current.val(),
            address = address.val()
        );
        let page_count = calc_page_count(growth as usize);

        let layout = Layout::from_size_align(page_count * PAGE_SIZE, PAGE_SIZE).unwrap();
        let r_mapping_ptr: NonNull<[u8]> = Global.allocate_zeroed(layout).unwrap();
        let r_mapping_addr = r_mapping_ptr.as_mut_ptr() as u64;
        let perm = MemCapPermissions::RW;

        let mapping = MemoryMapping::new(
            PageAddress::new(r_mapping_addr),
            layout,
            self.u_program_break_current,
            page_count,
            MemoryKind::Heap,
            perm,
        );
        self.memory_mappings.insert(mapping.u_address, mapping);

        CrdDelegateOptimizer::new(
            r_mapping_addr / PAGE_SIZE as u64,
            self.u_program_break_current.val() as u64 / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            process.parent().unwrap().pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            perm,
        );

        let _old_break = self.u_program_break_current;
        self.u_program_break_current =
            PageAddress::new(self.u_program_break_current.val() + growth);
        self.u_program_break_current.val()
    }

    /// Increases the program break by providing a size that describes the
    /// growth in bytes. Uprounds the size to the next multiple of a page.
    ///
    /// Performs memory mappings.
    pub fn increase_break_by(&mut self, size: usize, process: &Process) -> u64 {
        assert!(size > 0, "size must be bigger than 0");
        log::trace!("size={}", size);
        let page_offset = size & 0xfff;
        let size = if page_offset == 0 {
            size
        } else {
            size + PAGE_SIZE - page_offset
        };
        assert_eq!(size % PAGE_SIZE, 0);
        assert!(size != 0);
        let new_brk_addr = self.u_program_break_current.val() + size as u64;
        self.increase_break(new_brk_addr, process)
    }

    /// Maps a memory area to the user (for heap usage). The heap is
    pub fn mmap(&mut self, layout: Layout, process: &Process) -> u64 {
        let layout = layout.align_to(PAGE_SIZE).unwrap();

        // upround to next multiple of page size
        let size = calc_page_count(layout.size()) * PAGE_SIZE;
        let layout = Layout::from_size_align(size, layout.align()).unwrap();

        let r_ptr: NonNull<[u8]> = Global.allocate_zeroed(layout).unwrap();
        let r_ptr = r_ptr.as_non_null_ptr().as_ptr();
        let r_addr = r_ptr as u64;
        let r_addr_page_num = r_addr / PAGE_SIZE as u64;

        let page_count = calc_page_count(layout.size());

        let perm = MemCapPermissions::RW;
        let mapping = MemoryMapping::new(
            PageAddress::new(r_addr),
            layout,
            PageAddress::new(self.u_next_mmap_addr),
            page_count,
            MemoryKind::Heap,
            perm,
        );
        self.memory_mappings.insert(mapping.u_address, mapping);

        CrdDelegateOptimizer::new(
            r_addr_page_num,
            self.u_next_mmap_addr / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            process.parent().unwrap().pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            perm,
        );

        let addr = self.u_next_mmap_addr;
        self.u_next_mmap_addr += layout.size() as u64;
        addr
    }

    pub fn munmap(&mut self, u_addr: u64, process: &Process) {
        let mapping = self
            .memory_mappings
            .iter()
            .find(|(mapping_u_addr, _mapping)| mapping_u_addr.val() == u_addr)
            .unwrap();

        let (u_addr, page_count) = (*mapping.0, mapping.1.page_count);
        drop(mapping);

        // downgrade rights
        CrdDelegateOptimizer::new(
            u_addr.val() / PAGE_SIZE as u64,
            u_addr.val() / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            process.pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            MemCapPermissions::empty(),
        );

        self.memory_mappings.remove(&u_addr);
    }

    pub fn stack(&self) -> &MemoryMapping {
        self.stack.as_ref().unwrap()
    }

    pub fn stack_mut(&mut self) -> &mut MemoryMapping {
        self.stack.as_mut().unwrap()
    }

    /// Returns the current program break in user address space.
    pub fn u_program_break_current(&self) -> PageAddress {
        self.u_program_break_current
    }

    pub fn u_program_break_begin(&self) -> PageAddress {
        self.u_program_break_begin
    }
}

/// Describes a memory mapping for a process. Allows access to it in roottask address space.
#[derive(Debug)]
pub struct MemoryMapping {
    /// The address of the mapping in the address space of the roottask. This points into the
    /// heap of the roottask and must be used to free heap usage of the roottask afterwards.
    r_address: PageAddress,
    /// The layout used for the allocation of this memory mapping inside the heap of the roottask.
    /// Required to free the memory properly afterwards.
    r_layout: core::alloc::Layout,
    /// The address of the mapping in the address space of the user app.
    u_address: PageAddress,
    /// Amount of pages that were mapped.
    page_count: usize,
    kind: MemoryKind,
    /// Permissions of the mapping in the address space of the user app.
    u_perm: MemCapPermissions,
}

impl MemoryMapping {
    const fn new(
        r_address: PageAddress,
        r_layout: Layout,
        u_address: PageAddress,
        page_count: usize,
        kind: MemoryKind,
        u_perm: MemCapPermissions,
    ) -> Self {
        // TODO invoke allocator here?!
        Self {
            r_address,
            r_layout,
            u_address,
            page_count,
            kind,
            u_perm,
        }
    }

    pub fn address(&self) -> PageAddress {
        self.u_address
    }
    pub fn page_count(&self) -> usize {
        self.page_count
    }
    pub fn kind(&self) -> &MemoryKind {
        &self.kind
    }
    pub fn perm(&self) -> MemCapPermissions {
        self.u_perm
    }
    pub fn len(&self) -> usize {
        self.page_count * PAGE_SIZE
    }

    /// Returns a pointer to the beginning of the mapping in the address space of the roottask.
    pub fn r_address_as_non_null(&self) -> NonNull<u8> {
        NonNull::new(self.r_address.val() as *mut _).unwrap()
    }

    /// Gives raw access to the memory mapping in the address space of the roottask.
    pub fn mem_as_ref(&self) -> &[u8] {
        let addr = self.r_address_as_non_null();
        unsafe { core::slice::from_raw_parts(addr.as_ref(), self.len()) }
    }

    /// Gives raw access to the memory mapping in the address space of the roottask.
    pub fn mem_as_mut(&mut self) -> &mut [u8] {
        let mut addr = self.r_address_as_non_null();
        unsafe { core::slice::from_raw_parts_mut(addr.as_mut(), self.len()) }
    }
}

impl Drop for MemoryMapping {
    fn drop(&mut self) {
        unsafe { Global.deallocate(self.r_address_as_non_null(), self.r_layout) }
    }
}

impl PartialOrd for MemoryMapping {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.u_address.val().partial_cmp(&other.u_address.val())
    }
}

impl PartialEq for MemoryMapping {
    fn eq(&self, other: &Self) -> bool {
        self.u_address.val().eq(&other.u_address.val())
    }
}

impl Ord for MemoryMapping {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for MemoryMapping {}

/// Describes the kind of a [`MemoryMapping`].
#[derive(Debug)]
pub enum MemoryKind {
    /// Memory is used for executable file.
    Elf,
    /// Memory is used as heap. Either program break or mmap-like mappings.
    Heap,
    /// Memory is used as stack.
    Stack,
}
