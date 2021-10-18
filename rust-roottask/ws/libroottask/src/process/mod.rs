use crate::capability_space::RootCapabilitySpace;
use crate::roottask_exception;
use crate::stack::StaticStack;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use bitflags::_core::hash::Hash;
use core::char::MAX;
use core::cmp::Ordering;
use core::hash::Hasher;
use core::pin::Pin;
use elf_rs::{
    Elf,
    ProgramType,
};
use libhrstd::capability_space::UserAppCapabilitySpace;
use libhrstd::libhedron::capability::{
    CrdMem,
    CrdObjPT,
    MemCapPermissions,
    PTCapPermissions,
};
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::event_offset::ExceptionEventOffset::HedronGlobalEcStartup;
use libhrstd::libhedron::mem::{
    MAX_USER_ADDR,
    PAGE_SIZE,
};
use libhrstd::libhedron::mtd::Mtd;
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
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::util::crd_bulk::CrdBulkLoopOrderOptimizer;

type ProcessId = u64;

pub static PROCESS_MNG: SimpleMutex<ProcessManager> = SimpleMutex::new(ProcessManager::new());

/// 128KiB stack size for all Hedron-native apps.
const STACK_SIZE: usize = 0x20000;

#[derive(Debug)]
pub struct Process {
    pid: ProcessId,
    elf_file: Pin<Box<[u8], PageAlignedAlloc>>,
    name: String,
    stack: Pin<Box<[u8; STACK_SIZE], PageAlignedAlloc>>,
    utcb: Pin<Box<Utcb, PageAlignedAlloc>>,
}

impl Process {
    pub fn new(
        pid: ProcessId,
        elf_file: Pin<Box<[u8], PageAlignedAlloc>>,
        name: String,
        stack: Pin<Box<[u8; STACK_SIZE], PageAlignedAlloc>>,
        utcb: Pin<Box<Utcb, PageAlignedAlloc>>,
    ) -> Self {
        Process {
            pid,
            elf_file,
            name,
            stack,
            utcb,
        }
    }
}

impl PartialEq for Process {
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
            pid_counter: 0,
            init: false,
        }
    }

    pub fn init(&mut self) {
        roottask_exception::register_specialized_exc_handler(
            ExceptionEventOffset::HedronGlobalEcStartup,
            startup_callback,
        );
        self.init = true;
    }

    pub fn start(
        &mut self,
        elf_file: Pin<Box<[u8], PageAlignedAlloc>>,
        program_name: String,
    ) -> ProcessId {
        if !self.init {
            panic!("call init() first!");
        }

        log::debug!("starting file {}", program_name);

        let pid = self.pid_counter;
        self.pid_counter += 1;

        let pd_cap = RootCapabilitySpace::ProcessEcBase.val() + pid;
        let ec_cap = RootCapabilitySpace::ProcessEcBase.val() + pid;
        let sc_cap = RootCapabilitySpace::ProcessScBase.val() + pid;

        // location in new PD
        let virt_utcb_addr = MAX_USER_ADDR - PAGE_SIZE;
        // -128 + 8: correct alignment for vector instructions
        let virt_stack_addr = virt_utcb_addr - 128 + 8;

        create_pd(false, pd_cap, RootCapabilitySpace::RootPd.val()).unwrap();
        log::debug!("created PD");
        create_global_ec(
            ec_cap,
            pd_cap,
            virt_stack_addr as u64,
            UserAppCapabilitySpace::ExceptionEventBase.val(),
            0,
            virt_utcb_addr as u64,
        )
        .unwrap();
        log::debug!("created global EC");
        create_sc(sc_cap, pd_cap, ec_cap).unwrap();
        log::debug!("created SC");

        // install all exception handling portals (including STARTUP portal) into new PD
        pd_ctrl_delegate(
            RootCapabilitySpace::RootPd.val(),
            pd_cap,
            CrdObjPT::new(
                RootCapabilitySpace::ExceptionEventBase.val(),
                // 2^5 = 32 => all portal caps at once for 32 exceptions
                5,
                PTCapPermissions::CALL,
            ),
            CrdObjPT::new(
                UserAppCapabilitySpace::ExceptionEventBase.val(),
                5,
                PTCapPermissions::CALL,
            ),
            DelegateFlags::new(false, false, false, false, 0),
        )
        .unwrap();

        log::debug!("installed all exception portals into new PD");

        let stack = Pin::new(Box::new_in([0_u8; STACK_SIZE], PageAlignedAlloc));
        let utcb = Pin::new(Box::new_in(Utcb::new(), PageAlignedAlloc));

        // Map UTCB into new PD
        pd_ctrl_delegate(
            RootCapabilitySpace::RootPd.val(),
            pd_cap,
            CrdMem::new(
                ((&utcb) as *const _ as usize / PAGE_SIZE) as u64,
                0,
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            ),
            CrdMem::new(
                (virt_utcb_addr / PAGE_SIZE) as u64,
                0,
                MemCapPermissions::READ | MemCapPermissions::WRITE,
            ),
            DelegateFlags::new(false, false, false, false, 0),
        )
        .unwrap();

        log::debug!("delegated mem for UTCB to new PD");

        // Map stack into new PD
        let base_root_stack_page_num = (stack.as_ptr() as usize / PAGE_SIZE) as u64;
        let base_vm_stack_page_num = (virt_stack_addr / PAGE_SIZE) as u64;
        CrdBulkLoopOrderOptimizer::new(
            base_root_stack_page_num,
            base_vm_stack_page_num,
            STACK_SIZE / PAGE_SIZE,
        )
        .for_each(|iter| {
            pd_ctrl_delegate(
                RootCapabilitySpace::RootPd.val(),
                pd_cap,
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
                DelegateFlags::new(false, false, false, false, 0),
            )
            .unwrap()
        });

        log::debug!("delegated mem for stack to new PD");

        // map all load segments
        let elf = elf_rs::Elf::from_bytes(&elf_file).unwrap();
        let elf64 = match elf {
            Elf::Elf64(elf) => elf,
            _ => panic!("unexpected elf 32"),
        };
        elf64.program_header_iter().for_each(|pr_hdr| {
            if pr_hdr.ph.ph_type() != ProgramType::LOAD {
                log::debug!("skipping ph_hdr {:?}", pr_hdr.ph);
            }
            CrdBulkLoopOrderOptimizer::new(
                base_root_stack_page_num,
                base_vm_stack_page_num,
                STACK_SIZE / PAGE_SIZE,
            )
            .for_each(|iter| {
                let base_vm_page_num = pr_hdr.ph.vaddr() / PAGE_SIZE as u64;
                let base_root_page_num = elf_file.as_ptr() as u64 / PAGE_SIZE as u64;
                CrdBulkLoopOrderOptimizer::new(
                    base_root_page_num,
                    base_vm_page_num,
                    pr_hdr.ph.filesz() as usize / PAGE_SIZE,
                )
                .for_each(|iter| {
                    pd_ctrl_delegate(
                        RootCapabilitySpace::RootPd.val(),
                        pd_cap,
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
                        DelegateFlags::new(false, false, false, false, 0),
                    )
                    .unwrap()
                });

                log::debug!("mapped mem for all load segments to new PD");
            });
        });

        let process = Process::new(pid, elf_file, program_name, stack, utcb);
        self.processes.insert(pid, process).unwrap();

        pid
    }

    pub fn terminate(&mut self, id: ProcessId) -> Result<(), ()> {
        todo!()
    }
}

fn startup_callback(id: ExceptionEventOffset, utcb: &mut Utcb) -> ! {
    log::info!("startup portal called! id={:?}", id);
    let mut process_mngr = PROCESS_MNG.lock();

    // TODO Q&D: need some kind of identifier here?!
    let pid = 0;
    let process = process_mngr.processes.get_mut(&pid).expect("unknown ID!");

    let elf = elf_rs::Elf::from_bytes(&process.elf_file).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("unexpected elf 32"),
    };

    // process.utcb.exception_data_mut().rip = elf64.header().entry_point();

    // process.utcb.exception_data_mut().mtd = Mtd::RSP;
    // process.utcb.exception_data_mut().

    utcb.exception_data_mut().rip = elf64.header().entry_point();
    utcb.exception_data_mut().mtd = Mtd::empty();

    reply();
}
