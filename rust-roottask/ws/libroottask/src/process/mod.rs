use crate::capability_space::RootCapSpace;
use crate::roottask_exception;
use crate::roottask_exception::LOCAL_EXC_EC_STACK_TOP;
use crate::stack::StaticStack;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::char::MAX;
use core::cmp::Ordering;
use core::hash::{
    Hash,
    Hasher,
};
use core::pin::Pin;
use elf_rs::{
    Elf,
    ProgramType,
};
use libhrstd::capability_space::UserAppCapSpace;
use libhrstd::libhedron::capability::{
    CapSel,
    CrdMem,
    CrdObjEC,
    CrdObjPD,
    CrdObjPT,
    CrdObjSC,
    ECCapPermissions,
    MemCapPermissions,
    PDCapPermissions,
    PTCapPermissions,
    SCCapPermissions,
};
use libhrstd::libhedron::consts::NUM_EXC;
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::event_offset::ExceptionEventOffset::HedronGlobalEcStartup;
use libhrstd::libhedron::mem::{
    MAX_USER_ADDR,
    PAGE_SIZE,
};
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::qpd::Qpd;
use libhrstd::libhedron::syscall::create_ec::{
    create_global_ec,
    create_local_ec,
};
use libhrstd::libhedron::syscall::create_pd::create_pd;
use libhrstd::libhedron::syscall::create_pt::create_pt;
use libhrstd::libhedron::syscall::create_sc::create_sc;
use libhrstd::libhedron::syscall::ipc::reply;
use libhrstd::libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::mem::{
    AlignedAlloc,
    PageAligned,
    PageAlignedAlloc,
    PageAlignedByteBuf,
};
use libhrstd::portal_identifier::PortalIdentifier;
use libhrstd::process::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::uaddress_space::{
    USER_STACK_SIZE,
    VIRT_STACK_BOTTOM_PAGE_NUM,
    VIRT_STACK_TOP,
    VIRT_UTCB_PAGE_NUM,
};
use libhrstd::util::crd_bulk::CrdBulkLoopOrderOptimizer;

pub static PROCESS_MNG: SimpleMutex<ProcessManager> = SimpleMutex::new(ProcessManager::new());

#[derive(Debug)]
pub struct Process {
    pid: ProcessId,
    elf_file: Box<[u8], PageAlignedAlloc>,
    name: String,
    // stack with size USER_STACK_SIZE
    stack: Box<[u8], PageAlignedAlloc>,
    utcb: Box<Utcb, PageAlignedAlloc>,
}

impl Process {
    /// Creates a new process.
    pub fn new(pid: ProcessId, elf_file: Box<[u8], PageAlignedAlloc>, name: String) -> Self {
        // The roottask itself (PID=0) is not inside this cap space => drop it
        let pid_cap_sel = pid - 1;

        let pd_cap_in_root = RootCapSpace::ProcessPdBase.val() + pid_cap_sel;
        let ec_cap_in_root = RootCapSpace::ProcessEcBase.val() + pid_cap_sel;
        let sc_cap_in_root = RootCapSpace::ProcessScBase.val() + pid_cap_sel;

        Self::init_create_pd(pd_cap_in_root);
        Self::init_exc_portals(
            RootCapSpace::ProcessExcPtBase.val() + pid_cap_sel,
            pid,
            pd_cap_in_root,
        );

        let utcb = Box::new_in(Utcb::new(), PageAlignedAlloc);

        // don't do "Box::new_in([0_u8; STACK_SIZE]" because at least in Debug build it copies
        // data first to the stack => can result in Page Fault (not enough stack mem)
        let mut stack = Vec::with_capacity_in(USER_STACK_SIZE, PageAlignedAlloc);
        unsafe { stack.set_len(USER_STACK_SIZE) };
        stack.fill(0_u8);
        let stack = stack.into_boxed_slice();

        Self::init_map_utcb(pd_cap_in_root, &utcb);
        Self::init_map_stack(pd_cap_in_root, &stack);
        Self::init_map_elf_load_segments(pd_cap_in_root, &elf_file);

        Self::init_install_service_portals(pd_cap_in_root);

        // Create the global EC at the end! The startup exception is fired immediately, but we
        // need to map the memory first!
        Self::init_create_global_ec(pd_cap_in_root, ec_cap_in_root);
        log::trace!("created global ec");
        Self::init_create_sc(pd_cap_in_root, ec_cap_in_root, sc_cap_in_root);
        log::trace!("created sc");

        Process {
            pid,
            elf_file,
            name,
            stack,
            utcb,
        }
    }

    /// Creates a new PD for the new process in the cap space of the caller PD/the roottask.
    /// It delegates the capability to the new PD afterwards too.
    ///
    /// # Parameters
    /// * `pd_sel_in_root` selector for the new PD in the cap space of the roottask
    fn init_create_pd(pd_sel_in_root: CapSel) {
        create_pd(false, pd_sel_in_root, RootCapSpace::RootPd.val()).unwrap();
        pd_ctrl_delegate(
            RootCapSpace::RootPd.val(),
            pd_sel_in_root,
            CrdObjPD::new(pd_sel_in_root, 0, PDCapPermissions::CREATE_EC),
            CrdObjPD::new(UserAppCapSpace::Pd.val(), 0, PDCapPermissions::CREATE_EC),
            DelegateFlags::new(true, false, false, false, 0),
        )
        .unwrap();
    }

    /// Creates [`NUM_EXC`] new portals inside the roottask, let them point
    /// to the common generic exception handler and delegate them to
    /// the new protection domain.
    ///
    /// # Parameters
    /// * `base_cap_sel_in_root`: Base cap sel into the roottask.
    /// * `pid`: Process ID of the new process.
    /// * `pd_sel_in_root`: Capability selector of the new PD inside the cap space of the roottask
    fn init_exc_portals(base_cap_sel_in_root: CapSel, pid: ProcessId, pd_sel_in_root: CapSel) {
        for exc_i in 0..NUM_EXC as u64 {
            let roottask_pt_sel = base_cap_sel_in_root + exc_i;

            // create a new portal inside root PD, associate it with generic exc callback
            roottask_exception::create_exc_handler_portal(
                roottask_pt_sel,
                PortalIdentifier::new(exc_i, pid, 0),
            );

            pd_ctrl_delegate(
                RootCapSpace::RootPd.val(),
                pd_sel_in_root,
                // Must be callable
                CrdObjPT::new(roottask_pt_sel, 0, PTCapPermissions::CALL),
                CrdObjPT::new(exc_i, 0, PTCapPermissions::CALL),
                DelegateFlags::new(true, false, false, false, 0),
            )
            .unwrap();
        }
        log::trace!("created and mapped exception portals into new PD");
    }

    /// Creates a new global EC that belongs to the PD for the new process in the cap space of the
    /// caller PD/the roottask. Delegates the capability to the new PD afterwards.
    ///
    /// # Parameters
    /// * `pd_sel_in_root`: PD CapSel of the new process in the cap space of the roottask
    /// * `ec_sel_in_root`: EC CapSel of the new process in the cap space of the roottask
    fn init_create_global_ec(pd_sel_in_root: CapSel, ec_sel_in_root: CapSel) {
        create_global_ec(
            ec_sel_in_root,
            pd_sel_in_root,
            // done in STARTUP portal.
            UserAppCapSpace::ExceptionEventBase.val(),
            0,
            VIRT_UTCB_PAGE_NUM,
        )
        .unwrap();
        pd_ctrl_delegate(
            RootCapSpace::RootPd.val(),
            pd_sel_in_root,
            CrdObjEC::new(pd_sel_in_root, 0, ECCapPermissions::empty()),
            CrdObjEC::new(UserAppCapSpace::Ec.val(), 0, ECCapPermissions::empty()),
            DelegateFlags::new(true, false, false, false, 0),
        )
        .unwrap();
    }

    /// Creates a new SC that belongs to the PD and the main global EC for the new process in the
    /// cap space of the caller PD/the roottask. Delegates the capability to the new PD afterwards.
    ///
    /// # Parameters
    /// * `pd_sel_in_root`: PD CapSel of the new process in cap space of the roottask.
    /// * `ec_sel_in_root`: EC CapSel of the new process in cap space of the roottask.
    /// * `sc_sel_in_root`: SC CapSel of the new process in cap space of the roottask.
    fn init_create_sc(pd_sel_in_root: CapSel, ec_sel_in_root: CapSel, sc_sel_in_root: CapSel) {
        create_sc(
            sc_sel_in_root,
            pd_sel_in_root,
            ec_sel_in_root,
            Qpd::new(1, 333),
        )
        .unwrap();
        pd_ctrl_delegate(
            RootCapSpace::RootPd.val(),
            pd_sel_in_root,
            CrdObjSC::new(pd_sel_in_root, 0, SCCapPermissions::empty()),
            CrdObjSC::new(UserAppCapSpace::Sc.val(), 0, SCCapPermissions::empty()),
            DelegateFlags::new(true, false, false, false, 0),
        )
        .unwrap();
    }

    /// Maps the UTCB into the new PD.
    fn init_map_utcb(pd_sel_in_root: CapSel, utcb: &Box<Utcb, PageAlignedAlloc>) {
        log::debug!("mapping utcb into new PD");
        pd_ctrl_delegate(
            RootCapSpace::RootPd.val(),
            pd_sel_in_root,
            CrdMem::new(
                utcb.page_num(),
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
    fn init_map_stack(pd_sel_in_root: CapSel, stack: &Box<[u8], PageAlignedAlloc>) {
        assert_eq!(stack.len(), USER_STACK_SIZE, "stack has wrong size?!");
        assert_eq!(
            USER_STACK_SIZE % PAGE_SIZE,
            0,
            "STACK-Size must be a multiple of PAGE_SIZE."
        );
        log::debug!(
            "mapping stack (virt stack top=0x{:016x}) into new PD",
            VIRT_STACK_TOP
        );
        let src_stack_bottom_page_num = stack.as_ptr() as u64 / PAGE_SIZE as u64;
        CrdBulkLoopOrderOptimizer::new(
            src_stack_bottom_page_num,
            VIRT_STACK_BOTTOM_PAGE_NUM,
            USER_STACK_SIZE / PAGE_SIZE,
        )
        .for_each(|iter| {
            log::debug!(
                    "mapping page for stack: page {} ({:?}) of pd {} to page {} ({:?}) of pd {} with order={} (2^order={})",
                    iter.src_base,
                    (iter.src_base as usize * PAGE_SIZE) as *const u64,
                    RootCapSpace::RootPd.val(),
                    iter.dest_base,
                    (iter.dest_base as usize * PAGE_SIZE) as *const u64,
                    pd_sel_in_root,
                    iter.order,
                    iter.power
                );
            pd_ctrl_delegate(
                RootCapSpace::RootPd.val(),
                pd_sel_in_root,
                CrdMem::new(
                    iter.src_base,
                    iter.order,
                    MemCapPermissions::READ | MemCapPermissions::WRITE,
                ),
                CrdMem::new(
                    iter.dest_base,
                    iter.order,
                    MemCapPermissions::READ | MemCapPermissions::WRITE,
                ),
                DelegateFlags::new(true, false, false, false, 0),
            )
            .unwrap();
        });

        // TODO last stack page without read or write permissions! => detect page fault
    }

    /// Maps all load segments into the new PD.
    fn init_map_elf_load_segments(pd_cap_in_root: CapSel, elf_file: &Box<[u8], PageAlignedAlloc>) {
        let elf = elf_rs::Elf::from_bytes(&elf_file).unwrap();
        let elf64 = match elf {
            Elf::Elf64(elf) => elf,
            _ => panic!("unexpected elf 32"),
        };
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
                pr_hdr.ph.vaddr() as usize  % PAGE_SIZE,
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
            CrdBulkLoopOrderOptimizer::new(
                load_segment_src_page_num as u64,
                load_segment_dest_page_num as u64,
                num_pages as usize,
            )
            .for_each(|iter| {
                log::debug!(
                    "mapping page for elf segment: page {} ({:?}) of pd {} to page {} ({:?}) of pd {} with order={} (2^order={})",
                    iter.src_base,
                    (iter.src_base as usize * PAGE_SIZE) as *const u64,
                    RootCapSpace::RootPd.val(),
                    iter.dest_base,
                    (iter.dest_base as usize * PAGE_SIZE) as *const u64,
                    pd_cap_in_root,
                    iter.order,
                    iter.power
                );
                pd_ctrl_delegate(
                    RootCapSpace::RootPd.val(),
                    pd_cap_in_root,
                    CrdMem::new(
                        iter.src_base,
                        iter.order,
                        // todo not all read and write but only as needed
                        MemCapPermissions::READ | MemCapPermissions::WRITE | MemCapPermissions::EXECUTE,
                    ),
                    CrdMem::new(
                        iter.dest_base,
                        iter.order,
                        // todo not all read and write but only as needed
                        MemCapPermissions::READ | MemCapPermissions::WRITE | MemCapPermissions::EXECUTE,
                    ),
                    DelegateFlags::new(true, false, false, false, 0),
                )
                .unwrap()
            });
        });
    }

    /// Delegates all the service portals of the roottask into the new pd.
    fn init_install_service_portals(pd_sel_in_root: CapSel) {
        /// helper function
        fn map(pd_sel: CapSel, from: CapSel, to: CapSel) {
            pd_ctrl_delegate(
                RootCapSpace::RootPd.val(),
                pd_sel,
                // Must be callable
                CrdObjPT::new(from, 0, PTCapPermissions::CALL),
                CrdObjPT::new(to, 0, PTCapPermissions::CALL),
                DelegateFlags::new(true, false, false, false, 0),
            )
            .unwrap();
        }

        map(
            pd_sel_in_root,
            RootCapSpace::RoottaskStdoutServicePortal.val(),
            UserAppCapSpace::StdoutServicePT.val(),
        );
        map(
            pd_sel_in_root,
            // TODO map to roottask#stderr and not stdout
            RootCapSpace::RoottaskStdoutServicePortal.val(),
            UserAppCapSpace::StderrServicePT.val(),
        );
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

/// Manager that holds information about all processes that are
/// started by the roottask. Doesn't contain info about the
/// roottask itself.
#[derive(Debug)]
pub struct ProcessManager {
    processes: BTreeMap<ProcessId, Process>,
    pid_counter: u64,
    init: bool,
}

impl ProcessManager {
    pub const fn new() -> Self {
        ProcessManager {
            processes: BTreeMap::new(),
            // the process manager doesn't manage the roottask
            pid_counter: ROOTTASK_PROCESS_PID + 1,
            init: false,
        }
    }

    /// Initializes the callback for the startup exception of
    /// processes.
    pub fn init(&mut self) {
        roottask_exception::register_specialized_exc_handler(
            ExceptionEventOffset::HedronGlobalEcStartup,
            startup_callback,
        );
        self.init = true;
    }

    pub fn start(
        &mut self,
        elf_file: Box<[u8], PageAlignedAlloc>,
        program_name: String,
    ) -> ProcessId {
        if !self.init {
            panic!("call init() first!");
        }
        log::debug!("starting file {}", program_name);

        let pid = self.pid_counter;

        // the process starts itself. the Mng just keeps track of it.
        let process = Process::new(pid, elf_file, program_name);

        let _ = self.processes.insert(pid, process);
        self.pid_counter += 1;

        pid
    }

    pub fn terminate(&mut self, id: ProcessId) -> Result<(), ()> {
        todo!()
    }
}

fn startup_callback(pid: PortalIdentifier, utcb: &mut Utcb) -> ! {
    log::info!("startup portal called! id={:?}", pid);
    let mut process_mngr = PROCESS_MNG.lock();

    let process = process_mngr
        .processes
        .get_mut(&pid.pid())
        .expect("unknown ID!");

    let elf = elf_rs::Elf::from_bytes(&process.elf_file).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("unexpected elf 32"),
    };

    utcb.exception_data_mut().rip = elf64.header().entry_point();
    // julian: pit fall: portals don't reset their RSP after reply, therefore
    // we do it manually
    utcb.exception_data_mut().rsp = VIRT_STACK_TOP;
    // utcb.exception_data_mut().mtd = Mtd::RSP;

    // this influences what information are transferred to the portal exception handler
    // WTF?! I guess we set the GENERAL MTD of the new utcb?! TODO Ask julian
    utcb.exception_data_mut().mtd = Mtd::all();
    // utcb.exception_data_mut().mtd = Mtd::RSP | Mtd::RIP_LEN;

    log::info!(
        "startup: setting rip to 0x{:x}",
        elf64.header().entry_point()
    );

    reply(LOCAL_EXC_EC_STACK_TOP.val());
}
