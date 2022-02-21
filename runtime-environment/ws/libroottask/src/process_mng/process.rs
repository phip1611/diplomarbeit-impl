use crate::mem::{
    MappedMemory,
    MemLocation,
};
use crate::process_mng::syscall_abi::SyscallAbi;
use crate::process_mng::syscall_abi::SyscallAbi::NativeHedron;
use crate::roottask_exception;
use alloc::alloc::Layout;
use alloc::collections::BTreeSet;
use alloc::rc::{
    Rc,
    Weak,
};
use alloc::string::{
    String,
    ToString,
};
use alloc::vec::Vec;
use core::cell::{
    Cell,
    RefCell,
};
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::{
    Hash,
    Hasher,
};
use core::sync::atomic::AtomicU64;
use elf_rs::{
    ElfFile,
    ProgramType,
};
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::cap_space::user::{
    ForeignUserAppCapSpace,
    UserAppCapSpace,
};
use libhrstd::kobjects::{
    GlobalEcObject,
    PdObject,
    PortalIdentifier,
    PtObject,
    ScObject,
};
use libhrstd::libhedron::consts::NUM_EXC;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::Qpd;
use libhrstd::libhedron::{
    CapSel,
    MemCapPermissions,
};
use libhrstd::mem::{
    calc_page_count,
    PinnedPageAlignedHeapArray,
};
use libhrstd::process::consts::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::uaddress_space::{
    USER_ELF_ADDR,
    USER_HEAP_BEGIN,
    USER_STACK_BOTTOM_ADDR,
    USER_STACK_BOTTOM_PAGE_NUM,
    USER_STACK_SIZE,
    USER_STACK_TOP,
    USER_UTCB_ADDR,
};
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;
use linux_libc_auxv::{
    AuxVar,
    InitialLinuxLibcStackLayoutBuilder,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessState {
    /// Processes that are created but not yet started.
    Created,
    /// Processes that are started properly.
    Running,
}

/// A process is a wrapper around a [`PdObject`]. The process is responsible for
/// providing a stack, and a UTCB (it owns the memory).
#[derive(Debug)]
pub struct Process {
    pid: ProcessId,
    name: String,
    state: Cell<ProcessState>,
    parent: Option<Weak<Self>>,
    pd_obj: RefCell<Option<Rc<PdObject>>>,
    // todo theoretically I could remove the option, because I have the memory for the mapped
    //  roottask too from the hip
    elf_file: Option<MappedMemory>,
    // stack with size USER_STACK_SIZE for the main global EC
    // todo so far not per thread
    stack: RefCell<MemLocation<PinnedPageAlignedHeapArray<u8>>>,
    // Describes where the heap pointer is. This belongs to a Q&D approach of "fire and forget" allocations.
    // Similar to the "program break" in UNIX/Linux.
    heap_ptr: AtomicU64,

    syscall_abi: SyscallAbi,
}

impl Process {
    /// Creates the object for the root process. Doesn't create any system calls.
    pub fn root(
        utcb_addr: u64,
        stack_btm_addr: u64,
        stack_size: u64,
        stack_top_addr: u64,
    ) -> Rc<Self> {
        // the ::new-constructors don't trigger syscalls but just create the objects
        let root_pd_obj = PdObject::new(ROOTTASK_PROCESS_PID, None, RootCapSpace::RootPd.val());
        let root_ec_obj = GlobalEcObject::new(
            RootCapSpace::RootEc.val(),
            &root_pd_obj,
            utcb_addr,
            stack_top_addr,
        );
        let _ = ScObject::new(RootCapSpace::RootSc.val(), &root_ec_obj, None);

        Rc::new(Self {
            pid: ROOTTASK_PROCESS_PID,
            pd_obj: RefCell::new(Some(root_pd_obj)),
            elf_file: None,
            name: "roottask".to_string(),
            stack: RefCell::new(MemLocation::new_external(
                stack_btm_addr / PAGE_SIZE as u64,
                stack_size,
            )),
            state: Cell::new(ProcessState::Created),
            parent: None,
            heap_ptr: AtomicU64::new(0),
            syscall_abi: NativeHedron,
        })
    }

    /// Creates a new process object. Doesn't create kernel objects or trigger syscalls.
    /// Already allcoates memory for UTCB and stack.
    ///
    /// Invoke [`Self::init`] next.
    pub fn new(
        pid: u64,
        elf_file: MappedMemory,
        program_name: String,
        parent: &Rc<Self>,
        syscall_abi: SyscallAbi,
    ) -> Rc<Self> {
        assert_eq!(
            elf_file.perm(),
            MemCapPermissions::all(),
            "memory needs RXW permission, because permissions can only be downgraded, not upgraded"
        );
        Rc::new(Self {
            pid,
            pd_obj: RefCell::new(None),
            elf_file: Some(elf_file),
            name: program_name,
            stack: RefCell::new(MemLocation::Owned(PinnedPageAlignedHeapArray::new(
                0_u8,
                USER_STACK_SIZE,
            ))),
            state: Cell::new(ProcessState::Created),
            parent: Some(Rc::downgrade(parent)),
            heap_ptr: AtomicU64::new(USER_HEAP_BEGIN as u64),
            syscall_abi,
        })
    }

    /// Starts a process. This will
    /// - trigger syscalls for new PDs, ECs and SCs
    /// - map UTCB, STACK, and the LOAD segments from the ELF into the new process.
    ///
    /// This will result in a STARTUP exception.
    pub fn init(&self) {
        // state will be altered by the startup exception handler
        assert_eq!(self.state.get(), ProcessState::Created);
        log::debug!(
            "Create new process: pid={}, program_name={}",
            self.pid,
            self.name
        );

        let pd_cap_in_root = RootCapSpace::calc_pd_sel(self.pid);
        let ec_cap_in_root = RootCapSpace::calc_gl_ec_sel(self.pid);
        let sc_cap_in_root = RootCapSpace::calc_sc_sel(self.pid);

        let foreign_syscall_base = if self.syscall_abi.is_foreign() {
            Some(ForeignUserAppCapSpace::SyscallBasePt.val())
        } else {
            None
        };

        let pd = PdObject::create(
            self.pid,
            &self.parent().unwrap().pd_obj(),
            pd_cap_in_root,
            foreign_syscall_base,
        );
        self.pd_obj.borrow_mut().replace(pd.clone());

        let ec = GlobalEcObject::create(
            ec_cap_in_root,
            &pd,
            USER_UTCB_ADDR,
            // set in Startup-Exception anyway
            0,
        );
        log::trace!("created EC for PID={}", self.pid);

        self.init_exc_portals(RootCapSpace::calc_exc_pt_sel_base(self.pid));

        self.init_map_stack();
        self.init_map_elf_load_segments();

        crate::services::create_and_delegate_service_pts(self);
        if self.syscall_abi.is_foreign() {
            crate::services::foreign_syscall::create_and_delegate_syscall_handler_pts(self);
        }

        // create SC-Object at the very end! Otherwise Hedron might schedule the new PD too early
        let _ = ScObject::create(sc_cap_in_root, &ec, Qpd::new(1, 333));

        log::trace!(
            "Init process done: PID={}, name={}, utcb_addr={:x?}",
            self.pid,
            self.name,
            USER_UTCB_ADDR
        );
    }

    /// Creates [`NUM_EXC`] new portals inside the roottask, let them point
    /// to the common generic exception handler and delegate them to
    /// the new protection domain.
    ///
    /// # Parameters
    /// * `base_cap_sel_in_root`: Base cap sel into the roottask for the exception
    /// * `pid`: Process ID of the new process.
    /// * `pd_obj`: PdObject of this process.
    fn init_exc_portals(&self, base_cap_sel_in_root: CapSel) {
        for exc_i in 0..NUM_EXC as u64 {
            let roottask_pt_sel = base_cap_sel_in_root + exc_i;
            let pt = roottask_exception::create_exc_pt_for_process(exc_i, roottask_pt_sel);

            // delegate each exception portal to the pd of the new process
            PtObject::delegate(
                &pt,
                &self.pd_obj(),
                UserAppCapSpace::ExceptionEventBase.val() + exc_i,
            )
        }

        log::trace!("created and mapped exception portals into new PD");
    }

    /// Maps the stack into the new PD.
    fn init_map_stack(&self) {
        assert_eq!(
            USER_STACK_SIZE % PAGE_SIZE,
            0,
            "STACK-Size must be a multiple of PAGE_SIZE."
        );
        assert_eq!(
            self.stack.borrow().size_in_pages() as usize,
            USER_STACK_SIZE / PAGE_SIZE,
            "stack has wrong size?!"
        );
        log::debug!(
            "mapping stack (virt stack top=0x{:016x}) into new PD",
            USER_STACK_TOP
        );
        let src_stack_bottom_page_num = self.stack.borrow().page_num();
        CrdDelegateOptimizer::new(
            src_stack_bottom_page_num,
            USER_STACK_BOTTOM_PAGE_NUM,
            USER_STACK_SIZE / PAGE_SIZE,
        )
        .mmap(
            self.parent().unwrap().pd_obj().cap_sel(),
            self.pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE,
        );

        // TODO last stack page without read or write permissions! => detect page fault
    }

    /// Maps all load segments into the new PD.
    fn init_map_elf_load_segments(&self) {
        let elf = elf_rs::Elf::from_bytes(self.elf_file_bytes()).unwrap();

        // log::debug!("ELF: {:#?}", elf64.header());
        log::debug!("mapping mem for all load segments to new PD");
        elf
            .program_header_iter()
            .filter(|pr_hrd| pr_hrd.ph_type() == ProgramType::LOAD)
            .for_each(|pr_hdr| {
                log::trace!("next segment");
                assert_eq!(
                    pr_hdr.align() as usize,
                    PAGE_SIZE,
                    "expects that all segments are page aligned inside the file!!"
                );

                // there are ELF-files, where the offset is not the begin of the page, but
                // somewhere in the middle (i.e. to save space in the file). Thus, content from
                // the same page can be mapped to different virtual pages, one with R and one
                // with R+W. I experienced this for example with default gcc binaries for Linux.

                // TODO really Q&D
                if pr_hdr.memsz() == pr_hdr.filesz() {
                    assert_eq!(pr_hdr.offset() % PAGE_SIZE as u64, 0);
                    // mem in roottask: pointer/page into address space of the roottask
                    let load_segment_src_page_num = pr_hdr.content().as_ptr() as usize / PAGE_SIZE;
                    // virt mem in dest PD / address space
                    let load_segment_dest_page_num = pr_hdr.vaddr() as usize / PAGE_SIZE;

                    // number of pages to map
                    let num_pages = calc_page_count(pr_hdr.filesz() as usize);

                    CrdDelegateOptimizer::new(
                        load_segment_src_page_num as u64,
                        load_segment_dest_page_num as u64,
                        num_pages,
                    )
                    .mmap(
                        RootCapSpace::RootPd.val(),
                        self.pd_obj().cap_sel(),
                        // works because Hedron and ELF use the same bits for RWX
                        MemCapPermissions::from_elf_segment_permissions(pr_hdr.flags().bits() as u8),
                    );
                } else {
                    // memsize != file size
                    // I can't map the ELF load segment directly

                    // offset of load segment in first page (segment might not start at page aligned address)
                    let first_page_offset = pr_hdr.offset() & 0xfff;
                    // the total number we need in bytes (we always need to start at a page)
                    let total_size = first_page_offset + pr_hdr.memsz();
                    // how many pages we need
                    let page_count = calc_page_count(total_size as usize);

                    // TODO this will never be freed.. Q&D
                    // roottask pointer that holds the elf segment (page aligned)
                    let r_elf_segment_ptr = unsafe {
                        alloc::alloc::alloc_zeroed(
                            Layout::from_size_align(page_count as usize * PAGE_SIZE, PAGE_SIZE)
                                .unwrap(),
                        )
                    };

                    // copy everything from the ELF file to the new memory
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            pr_hdr.content().as_ptr(),
                            r_elf_segment_ptr.add(first_page_offset as usize),
                            pr_hdr.filesz() as usize,
                        );
                    }

                    // mem in roottask: pointer/page into address space of the roottask
                    let load_segment_src_page_num = r_elf_segment_ptr as usize / PAGE_SIZE;
                    // virt mem in dest PD / address space
                    let load_segment_dest_page_num = pr_hdr.vaddr() as usize / PAGE_SIZE;

                    CrdDelegateOptimizer::new(
                        load_segment_src_page_num as u64,
                        load_segment_dest_page_num as u64,
                        page_count as usize,
                    )
                    .mmap(
                        RootCapSpace::RootPd.val(),
                        self.pd_obj().cap_sel(),
                        // works because Hedron and ELF use the same bits for RWX
                        MemCapPermissions::from_elf_segment_permissions(pr_hdr.flags().bits() as u8),
                    );
                }
            });
    }

    /// Libc-Programs expect a certain data structure on the stack, when the program starts
    /// running ("_start" symbol). The layout is described here: https://lwn.net/Articles/631631/
    ///
    /// Returns the new, actual stack pointer.
    pub fn init_stack_libc_aux_vector(&self) -> usize {
        let elf_bytes = self.elf_file_bytes();
        let elf = elf_rs::Elf::from_bytes(elf_bytes).unwrap();
        let pr_hdr_off = elf.elf_header().program_header_offset();
        dbg!(pr_hdr_off);

        // page aligned
        let elf_bytes_addr = elf_bytes.as_ptr() as u64;

        // map program header
        CrdDelegateOptimizer::new(
            elf_bytes_addr / PAGE_SIZE as u64,
            USER_ELF_ADDR / PAGE_SIZE as u64,
            1,
        )
        .mmap(
            self.parent().unwrap().pd_obj().cap_sel(),
            self.pd_obj().cap_sel(),
            MemCapPermissions::READ,
        );

        let stack_layout = InitialLinuxLibcStackLayoutBuilder::new()
            .add_arg_v("./executable")
            .add_arg_v("10.123")
            .add_arg_v("first")
            .add_arg_v("second")
            .add_env_v("FOO=BAR")
            // application can use this to check if it runs under hedron
            .add_env_v("LINUX_UNDER_HEDRON=true")
            .add_aux_v(AuxVar::ExecFn("./executable"))
            .add_aux_v(AuxVar::Platform("x86_64"))
            // libc (at least musl) expects all of this values to be present
            .add_aux_v(AuxVar::Phdr((USER_ELF_ADDR + pr_hdr_off) as *const u8))
            .add_aux_v(AuxVar::Phnum(
                elf.elf_header().program_header_entry_num() as usize
            ))
            .add_aux_v(AuxVar::Phent(
                elf.elf_header().program_header_entry_size() as usize
            ))
            .add_aux_v(AuxVar::Pagesz(PAGE_SIZE));

        let mut stack = self.stack.borrow_mut();
        // whole memory that is stack for user; in roottask address space
        let r_mem_stack = stack.as_slice_mut();

        // "r_addr": roottask address
        // "u_addr": user address

        let r_addr_stack_btm_inc = r_mem_stack.as_ptr() as usize;
        let r_addr_stack_top_excl = r_addr_stack_btm_inc + USER_STACK_SIZE;

        // - 1: to inclusive addr; - 8 because later we might need to add + 8 for correct alignment
        let mut r_addr_crt0_layout_btm = r_addr_stack_top_excl - 1 - stack_layout.total_size() - 8;
        if r_addr_crt0_layout_btm % 64 != 0 {
            r_addr_crt0_layout_btm -= r_addr_crt0_layout_btm % 64;
        }
        // stack must be 64-byte aligned + 8 byte offset => first arg will be correctly aligned
        r_addr_crt0_layout_btm += 8;

        // offset from bottom of stack to begin of crt0 data
        let r_offset_crt0_layout = r_addr_crt0_layout_btm - r_addr_stack_btm_inc;

        // RSP of user
        let u_addr_crt0_btm = USER_STACK_BOTTOM_ADDR + r_offset_crt0_layout as u64;

        let r_mem_crt0 = &mut r_mem_stack[r_offset_crt0_layout..];

        // write crt0 data
        unsafe {
            stack_layout.serialize_into_buf(r_mem_crt0, u_addr_crt0_btm);
        }

        u_addr_crt0_btm as usize
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter for [`PdObject`].
    pub fn pd_obj(&self) -> Rc<PdObject> {
        self.pd_obj
            .borrow()
            .as_ref()
            .expect("call init() first!")
            .clone()
    }

    /// Wrapper around [`PdObject::lookup_portal`].
    pub fn lookup_portal(&self, pid: PortalIdentifier) -> Option<Rc<PtObject>> {
        self.pd_obj().lookup_portal(pid)
    }

    pub fn state(&self) -> ProcessState {
        self.state.clone().into_inner()
    }
    pub fn parent(&self) -> Option<Rc<Self>> {
        self.parent.as_ref().map(|x| x.upgrade()).flatten()
    }

    /// Wrapper around [`PdObject::portals`].
    pub fn portals(&self) -> Vec<Rc<PtObject>> {
        self.pd_obj.borrow().as_ref().unwrap().portals()
    }

    /// Wrapper around [`PdObject::delegated_pts`].
    pub fn delegated_pts(&self) -> BTreeSet<Rc<PtObject>> {
        // self.pd_obj.borrow().as_ref().unwrap().delegated_pts()
        // todo inefficient but works
        self.pd_obj
            .borrow()
            .as_ref()
            .unwrap()
            .delegated_pts()
            .clone()
    }

    /// Gets the bytes of the page-aligned ELF file.
    pub fn elf_file_bytes(&self) -> &[u8] {
        let elf = self.elf_file.as_ref().unwrap();
        elf.mem_as_slice(elf.size())
    }

    pub fn heap_ptr(&self) -> &AtomicU64 {
        &self.heap_ptr
    }

    pub fn syscall_abi(&self) -> SyscallAbi {
        self.syscall_abi
    }
}

impl PartialEq for Process {
    /// Two processes are equal, if their ID is equal.
    fn eq(&self, other: &Self) -> bool {
        self.pid.eq(&other.pid)
    }
}

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pid.partial_cmp(&other.pid)
    }
}

impl Hash for Process {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pid.hash(state)
    }
}
