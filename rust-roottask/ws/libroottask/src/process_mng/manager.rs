use crate::mem::MappedMemory;
use crate::process_mng::process::Process;
use crate::process_mng::syscall_abi::SyscallAbi;
use crate::roottask_exception;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use elf_rs::ElfFile;

use libhrstd::kobjects::{
    PortalIdentifier,
    PtObject,
};
use libhrstd::libhedron::ExceptionEventOffset;
use libhrstd::libhedron::Mtd;
use libhrstd::libhedron::Utcb;
use libhrstd::process::consts::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::uaddress_space::USER_STACK_TOP;

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
    pub fn start_process(
        &mut self,
        elf_file: MappedMemory,
        program_name: String,
        syscall_abi: SyscallAbi,
    ) -> ProcessId {
        if !self.init {
            panic!("call init() first!");
        }
        log::info!("starting program {}", program_name);

        let pid = self.pid_counter;
        self.pid_counter += 1;

        // the process starts itself. the Mng just keeps track of it.
        let process = Process::new(pid, elf_file, program_name, self.root(), syscall_abi);
        let _ = self.processes.insert(pid, process.clone());

        // actually start
        process.init();
        log::debug!("process init done!");

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

    /// Registers [`Self::startup_exception_handler`] as the specialized handler for
    /// the startup exception in the `roottask_exception` module.
    pub fn register_startup_exc_callback(&self) {
        roottask_exception::register_specialized_exc_handler(
            ExceptionEventOffset::HedronGlobalEcStartup,
            Self::startup_exception_handler,
        );
    }

    /// Prepares the UTCB of the calling portal with the initial machine state to startup
    /// the thread.
    pub fn startup_exception_handler(
        _pt: &Rc<PtObject>,
        process: &Process,
        utcb: &mut Utcb,
        do_reply: &mut bool,
    ) {
        log::debug!("startup exception handler");

        let elf = elf_rs::Elf::from_bytes(process.elf_file_bytes()).unwrap();

        let utcb = utcb.exception_data_mut();
        utcb.mtd = Mtd::RIP_LEN | Mtd::RSP;
        // todo future work: figure out what global EC triggered this (multithreading, multiple stacks)
        utcb.rip = elf.entry_point();

        if matches!(process.syscall_abi(), SyscallAbi::Linux) {
            utcb.rsp = process.init_stack_libc_aux_vector() as u64;
        } else {
            utcb.rsp = USER_STACK_TOP;
        }

        *do_reply = true;
    }
}
