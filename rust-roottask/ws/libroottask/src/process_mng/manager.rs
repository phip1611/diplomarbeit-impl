use crate::mem::MappedMemory;
use crate::process_mng::process::Process;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use elf_rs::Elf;
use libhrstd::kobjects::{
    PortalIdentifier,
    PtObject,
};
use libhrstd::libhedron::event_offset::ExceptionEventOffset;
use libhrstd::libhedron::mtd::Mtd;
use libhrstd::libhedron::utcb::Utcb;
use libhrstd::process::consts::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::uaddress_space::VIRT_STACK_TOP;

/// The global instance for the roottask to manage all processes.
pub static PROCESS_MNG: SimpleMutex<ProcessManager> = SimpleMutex::new(ProcessManager::new());

/// Manager that holds information about all processes that are
/// started by the current PD. Can be used in the roottask or by
/// user-apps, that start other apps.
///
/// See [`Process`].
#[derive(Debug)]
pub struct ProcessManager {
    processes: BTreeMap<ProcessId, Rc<Process>>,
    pid_counter: u64,
    init: bool,
}

impl ProcessManager {
    /// Creates a new process manager. Caller must call [`init`] next.
    pub const fn new() -> Self {
        ProcessManager {
            processes: BTreeMap::new(),
            pid_counter: ROOTTASK_PROCESS_PID,
            init: false,
        }
    }

    /// Initializes the process object for the roottask
    pub fn init(
        &mut self,
        utcb_addr: u64,
        stack_btm_addr: u64,
        stack_size_pages: u64,
        stack_top_ptr: u64,
    ) {
        assert!(!self.init);
        // only creates the struct, without syscalls or so
        let process = Process::root(utcb_addr, stack_btm_addr, stack_size_pages, stack_top_ptr);
        self.pid_counter += 1;
        self.processes.insert(process.pid(), process);
        self.init = true;
    }

    /// Returns the Process object of the rootttask.
    pub fn root(&self) -> &Rc<Process> {
        assert!(self.init);
        self.processes.get(&ROOTTASK_PROCESS_PID).unwrap()
    }

    /// Starts a new process.
    pub fn start_process(&mut self, elf_file: MappedMemory, program_name: String) -> ProcessId {
        if !self.init {
            panic!("call init() first!");
        }
        log::debug!("starting program {}", program_name);

        let pid = self.pid_counter;
        self.pid_counter += 1;

        // the process starts itself. the Mng just keeps track of it.
        let process = Process::new(pid, elf_file, program_name, self.root());
        let _ = self.processes.insert(pid, process.clone());

        // actually start
        process.init();

        log::debug!("process init done!");

        // make sure startup exception is handled properly
        // TODO really quick & dirty
        let portals = process.parent().unwrap().portals();
        let startup_exc_pt = portals
            .iter()
            .find(|x| {
                x.ctx().unwrap().exc_pid().0 == ExceptionEventOffset::HedronGlobalEcStartup.val()
                    && x.ctx().unwrap().exc_pid().1 == process.pid()
            })
            .unwrap();
        crate::pt_multiplex::add_callback_hook(
            startup_exc_pt.portal_id(),
            ProcessManager::startup_exception_handler,
        );

        log::debug!("startup portal delegated!!");

        pid
    }

    pub fn terminate_prog(&mut self, _id: ProcessId) -> Result<(), ()> {
        todo!()
    }

    pub fn processes(&self) -> &BTreeMap<ProcessId, Rc<Process>> {
        &self.processes
    }

    /// Returns the process for a given PID.
    pub fn find_process_by_pid(&self, pid: ProcessId) -> Option<Rc<Process>> {
        self.processes.get(&pid).map(|x| x.clone())
    }

    /// Lookup for a portal in all processes.
    pub fn lookup_portal(&self, pid: PortalIdentifier) -> Option<Rc<PtObject>> {
        self.processes
            .iter()
            .map(|(_, x)| x.lookup_portal(pid))
            .filter(|x| x.is_some())
            .next()
            .unwrap()
    }

    /// Looks up the process by process ID.
    pub fn lookup_process(&self, pid: ProcessId) -> Option<&Rc<Process>> {
        self.processes.get(&pid)
    }

    /// Prepares the UTCB of the calling portal with the initial machine state to startup
    /// the thread.
    pub fn startup_exception_handler(
        pt: &Rc<PtObject>,
        process: &Process,
        utcb: &mut Utcb,
        do_reply: &mut bool,
    ) {
        let (_exc, _process_id) = pt.ctx().unwrap().exc_pid();
        log::debug!("startup exception handler");

        let elf = elf_rs::Elf::from_bytes(process.elf_file_bytes()).unwrap();
        let elf = match elf {
            Elf::Elf64(elf) => elf,
            Elf::Elf32(_) => panic!("only supports ELF64"),
        };

        let utcb = utcb.exception_data_mut();
        utcb.mtd = Mtd::RIP_LEN | Mtd::RSP;
        // todo future work: figure out what global EC triggered this (multithreading, multiple stacks)
        utcb.rsp = VIRT_STACK_TOP;
        utcb.rip = elf.header().entry_point();
        *do_reply = true;
    }
}
