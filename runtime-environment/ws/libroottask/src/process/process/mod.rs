mod memory;
mod syscall_abi;

pub use memory::*;
pub use syscall_abi::*;

use crate::mem::MappedMemory;
use crate::roottask_exception;
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
use core::cell::{
    Ref,
    RefMut,
};
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::{
    Hash,
    Hasher,
};
use elf_rs::ElfFile;
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
use libhrstd::process::consts::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use libhrstd::uaddress_space::{
    USER_ELF_ADDR,
    USER_STACK_BOTTOM_ADDR,
    USER_STACK_SIZE,
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
    /// Currently the process memory manager is only available for user processes
    /// but not the roottask.
    memory_manager: Option<RefCell<ProcessMemoryManager>>,

    /// Syscall ABI used by this process.
    syscall_abi: SyscallAbi,
}

impl Process {
    /// Creates the object for the root process. Doesn't create any system calls.
    pub fn root(utcb_addr: u64, stack_top_addr: u64) -> Rc<Self> {
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
            state: Cell::new(ProcessState::Created),
            parent: None,
            syscall_abi: SyscallAbi::NativeHedron,
            memory_manager: None,
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
    ) -> Self {
        assert_eq!(
            elf_file.perm(),
            MemCapPermissions::all(),
            "memory needs RXW permission, because permissions can only be downgraded, not upgraded"
        );
        Self {
            pid,
            pd_obj: RefCell::new(None),
            elf_file: Some(elf_file),
            name: program_name,
            state: Cell::new(ProcessState::Created),
            parent: Some(Rc::downgrade(parent)),
            syscall_abi,
            memory_manager: None,
        }
    }

    /// Starts a process. This will
    /// - trigger syscalls for new PDs, ECs and SCs
    /// - map UTCB, STACK, and the LOAD segments from the ELF into the new process.
    ///
    /// This will result in a STARTUP exception.
    pub fn init(&mut self) {
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
        log::trace!("created global EC for PID={}", self.pid);

        self.init_exc_portals(RootCapSpace::calc_exc_pt_sel_base(self.pid));

        let mut memory_manager = ProcessMemoryManager::new(self);
        memory_manager.init(self).unwrap();
        self.memory_manager.replace(RefCell::new(memory_manager));

        crate::services::create_and_delegate_service_pts(self);
        if self.syscall_abi.is_foreign() {
            crate::services::foreign_syscall::create_and_delegate_syscall_handler_pts(self);
        }

        // create SC-Object at the very end! Otherwise Hedron might schedule the new PD too early
        // (i.e.: before startup exception portal is set)
        let _ = ScObject::create(sc_cap_in_root, &ec, Qpd::new(1, None));

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

        let mut memory_manager = self.memory_manager_mut();
        let stack = memory_manager.stack_mut();
        // whole memory that is stack for user; in roottask address space
        let r_mem_stack = stack.mem_as_mut();

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

    // TODO this should not return a Result, because:
    // - the only exception is the roottask that does not has a parent object
    //   This adds inconvenience to all users of the API
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
        elf.mem_as_slice(elf.size() as usize)
    }

    pub fn syscall_abi(&self) -> SyscallAbi {
        self.syscall_abi
    }

    pub fn elf_file(&self) -> &Option<MappedMemory> {
        &self.elf_file
    }

    pub fn memory_manager(&self) -> Ref<ProcessMemoryManager> {
        self.memory_manager.as_ref().unwrap().borrow()
    }

    pub fn memory_manager_mut(&self) -> RefMut<ProcessMemoryManager> {
        self.memory_manager.as_ref().unwrap().borrow_mut()
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
