use crate::mem::{
    MappedMemory,
    MemLocation,
};
use crate::roottask_exception;
use alloc::boxed::Box;
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
use elf_rs::{
    Elf,
    ProgramType,
};
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{
    GlobalEcObject,
    PdObject,
    PortalIdentifier,
    PtObject,
    ScObject,
};
use libhrstd::libhedron::capability::{
    CapSel,
    CrdMem,
    CrdObjPT,
    MemCapPermissions,
    PTCapPermissions,
};
use libhrstd::libhedron::consts::NUM_EXC;

use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::qpd::Qpd;
use libhrstd::libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::PinnedPageAlignedHeapArray;
use libhrstd::process::consts::{
    ProcessId,
    NUM_PROCESSES,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::uaddress_space::{
    USER_STACK_SIZE,
    VIRT_STACK_BOTTOM_PAGE_NUM,
    VIRT_STACK_TOP,
    VIRT_UTCB_PAGE_NUM,
};
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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
    // todo theoretically I could remove the option, because we can now the memory for the mapped roottask too
    elf_file: Option<MappedMemory>,
    // stack with size USER_STACK_SIZE for the main global EC
    // todo so far not per thread
    stack: MemLocation<PinnedPageAlignedHeapArray<u8>>,
    // todo so far not per thread
    // UTCB for the main global EC. UTCB itself has align property already.
    utcb: MemLocation<Box<Utcb>>,
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
            stack: MemLocation::new_external(stack_btm_addr / PAGE_SIZE as u64, stack_size),
            utcb: MemLocation::new_external(utcb_addr / PAGE_SIZE as u64, PAGE_SIZE as u64),
            state: Cell::new(ProcessState::Created),
            parent: None,
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
            // utcb for the main global EC; Utcb type itself has already page align guarantee
            utcb: MemLocation::Owned(Box::new(Utcb::new())),
            stack: MemLocation::Owned(PinnedPageAlignedHeapArray::new(0_u8, USER_STACK_SIZE)),
            state: Cell::new(ProcessState::Created),
            parent: Some(Rc::downgrade(parent)),
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

        let stack_top_ptr = unsafe { self.stack.mem_ptr().add(USER_STACK_SIZE).sub(64).add(8) };

        let pd = PdObject::create(self.pid, &self.parent().unwrap().pd_obj(), pd_cap_in_root);
        self.pd_obj.borrow_mut().replace(pd.clone());

        let ec = GlobalEcObject::create(
            ec_cap_in_root,
            &pd,
            self.utcb.mem_ptr() as u64,
            stack_top_ptr as u64,
        );
        log::trace!("created EC for PID={}", self.pid);

        self.init_exc_portals(RootCapSpace::calc_exc_pt_sel_base(self.pid));

        self.init_map_utcb();
        self.init_map_stack();
        self.init_map_elf_load_segments();

        // create service pts for the new process
        crate::services::create_and_delegate_service_pts(self);

        // create SC-Object at the very end! Otherwise Hedron might schedule the new PD too early
        let _ = ScObject::create(sc_cap_in_root, &ec, Qpd::new(1, 333));
        log::trace!("created SC for PID={}", self.pid);
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
        // TODO use CrdDelegateOptimizer
        for exc_i in 0..NUM_EXC as u64 {
            let roottask_pt_sel = base_cap_sel_in_root + exc_i;
            let pt = roottask_exception::create_exc_pt_for_process(exc_i, roottask_pt_sel);
            pd_ctrl_delegate(
                self.parent().unwrap().pd_obj().cap_sel(),
                self.pd_obj().cap_sel(),
                // Must be callable for exceptions too
                CrdObjPT::new(roottask_pt_sel, 0, PTCapPermissions::CALL),
                CrdObjPT::new(exc_i, 0, PTCapPermissions::CALL),
                DelegateFlags::default(),
            )
            .unwrap();

            // TODO instead use something like `self.parent_process.map_portal(pt, dest_pd_obj)`?!
            pt.attach_delegated_to_pd(&self.pd_obj());
            self.pd_obj().attach_delegated_pt(pt)
        }
        log::trace!("created and mapped exception portals into new PD");
    }

    /// Maps the UTCB into the new PD.
    fn init_map_utcb(&self) {
        log::debug!("mapping utcb into new PD");
        log::trace!(
            "map page {} ({:?}) (pd={}) to page {} ({:?}) (pd={}), order={} (2^order={})",
            self.utcb.page_num(),
            (self.utcb.page_num() as usize * PAGE_SIZE) as *const u64,
            self.parent().unwrap().pd_obj().cap_sel(),
            VIRT_UTCB_PAGE_NUM,
            (VIRT_UTCB_PAGE_NUM as usize * PAGE_SIZE) as *const u64,
            self.pd_obj().cap_sel(),
            0,
            1
        );
        pd_ctrl_delegate(
            self.parent().unwrap().pd_obj().cap_sel(),
            self.pd_obj().cap_sel(),
            CrdMem::new(
                self.utcb.page_num(),
                // map exactly 1 single page
                0,
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            ),
            CrdMem::new(
                VIRT_UTCB_PAGE_NUM,
                // map exactly 1 single page
                0,
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            ),
            DelegateFlags::new(true, false, false, false, 0),
        )
        .unwrap();
    }

    /// Maps the stack into the new PD.
    fn init_map_stack(&self) {
        assert_eq!(
            USER_STACK_SIZE % PAGE_SIZE,
            0,
            "STACK-Size must be a multiple of PAGE_SIZE."
        );
        assert_eq!(
            self.stack.size_in_pages() as usize,
            USER_STACK_SIZE / PAGE_SIZE,
            "stack has wrong size?!"
        );
        log::debug!(
            "mapping stack (virt stack top=0x{:016x}) into new PD",
            VIRT_STACK_TOP
        );
        let src_stack_bottom_page_num = self.stack.page_num();
        CrdDelegateOptimizer::new(
            src_stack_bottom_page_num,
            VIRT_STACK_BOTTOM_PAGE_NUM,
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
        let elf64 = match elf {
            Elf::Elf64(elf) => elf,
            _ => panic!("unexpected elf 32"),
        };
        // log::debug!("ELF: {:#?}", elf64.header());
        log::debug!("mapping mem for all load segments to new PD");
        elf64.program_header_iter().for_each(|pr_hdr| {
            if pr_hdr.ph.ph_type() != ProgramType::LOAD {
                log::debug!("skipping ph_hdr {:?}", pr_hdr.ph);
            }
            assert_eq!(
                pr_hdr.ph.offset() as usize % PAGE_SIZE,
                0,
                "expects that all segments are page aligned inside the file!!"
            );
            assert_eq!(
                pr_hdr.ph.vaddr() as usize % PAGE_SIZE,
                0,
                "virtual address must be page-aligned!"
            );
            assert_eq!(
                pr_hdr.ph.vaddr(),
                pr_hdr.ph.paddr(),
                "virtual address must be physical address"
            );
            assert_eq!(
                pr_hdr.ph.filesz(),
                pr_hdr.ph.memsz(),
                "filesize must be memsize"
            );

            // mem in roottask: pointer/page into address space of the roottask
            let load_segment_src_page_num = pr_hdr.segment().as_ptr() as usize / PAGE_SIZE;
            // virt mem in dest PD / address space
            let load_segment_dest_page_num = pr_hdr.ph.vaddr() as usize / PAGE_SIZE;

            // number of pages to map
            let mut num_pages = pr_hdr.ph.filesz() as usize / PAGE_SIZE;
            if pr_hdr.ph.filesz() as usize % PAGE_SIZE != 0 {
                num_pages += 1;
            }

            // loop over all pages of the current segment
            CrdDelegateOptimizer::new(
                load_segment_src_page_num as u64,
                load_segment_dest_page_num as u64,
                num_pages as usize,
            )
            .mmap(
                RootCapSpace::RootPd.val(),
                self.pd_obj().cap_sel(),
                // TODO don't make this RWX :)
                MemCapPermissions::all(),
            );
        });
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
