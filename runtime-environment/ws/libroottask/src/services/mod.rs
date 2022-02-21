//! All service implementations the roottask provides via portals.

use crate::mem::{
    MappedMemory,
    ROOT_MEM_MAPPER,
    VIRT_MEM_ALLOC,
};
use crate::process_mng::manager::PROCESS_MNG;
use crate::process_mng::process::Process;
use crate::stack::StaticStack;
use alloc::collections::{
    BTreeMap,
    BTreeSet,
};
use alloc::rc::Rc;
use core::alloc::Layout;
use libhrstd::cap_space::root::RootCapSpace;
use libhrstd::cap_space::user::UserAppCapSpace;
use libhrstd::kobjects::{
    LocalEcObject,
    PtObject,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::Utcb;
use libhrstd::libhedron::HIP;
use libhrstd::mem::calc_page_count;
use libhrstd::process::consts::ProcessId;
use libhrstd::service_ids::ServiceId;
use libhrstd::sync::mutex::SimpleMutex;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;

pub mod allocate;
pub mod echo;
pub mod foreign_syscall;
pub mod fs;
pub mod stderr;
pub mod stdout;

static mut LOCAL_EC_STACK: StaticStack<16> = StaticStack::new();

/// The stack top of the local EC that handles all exception calls.
pub static LOCAL_EC_STACK_TOP: StaticGlobalPtr<u8> =
    StaticGlobalPtr::new(unsafe { LOCAL_EC_STACK.get_stack_top_ptr() });

/// Holds a weak reference to the local EC object used for handling service calls the roottask.
static LOCAL_EC: SimpleMutex<Option<Rc<LocalEcObject>>> = SimpleMutex::new(None);

/// Helps to keep knowledge about mapped areas. This accelerates reads and writes if certain user
/// memory pages are mapped already. For example, Linux read and write calls require memory
/// mappings. Because they are expensive, I try to cache them to avoid repetitions.
///
/// The type reads as following: Binary Tree Map of (From Process) to Map from page aligned address
/// to Memory Mapping.
static MAPPED_AREAS: SimpleMutex<MappedAreas> = SimpleMutex::new(MappedAreas::new());

/// The type reads as follows: Binary Tree Map of (From Process) to Map from page aligned
/// address to Memory Mapping.
struct MappedAreas(BTreeMap<ProcessId, BTreeMap<u64, MappedMemory>>);

impl MappedAreas {
    const fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Convenient wrapper that service functions should use if they need access to certain
    /// user memory. It creates a mapping with an appropriate size.
    fn create_get_mapping(
        &mut self,
        process: &Rc<Process>,
        u_addr: u64,
        u_count: u64,
    ) -> MappedMemory {
        // TODO create cool mechanism that displaces old mappings and prevents resource overflow

        // Map of Mappings per Process
        let main_map = &mut self.0;

        if !main_map.contains_key(&process.pid()) {
            main_map.insert(process.pid(), BTreeMap::new());
        }

        let process_mappings = main_map.get_mut(&process.pid()).unwrap();

        let u_page_addr = u_addr & !0xfff;
        let u_page_offset = u_addr & 0xfff;
        let page_count = calc_page_count((u_page_offset + u_count) as usize) as u64;

        if !process_mappings.contains_key(&u_page_addr) {
            let mapped_memory = Self::create_mapped_memory(process, u_page_addr, page_count);
            process_mappings.insert(u_page_addr, mapped_memory);
        } /* else {
              log::info!("CONTAINED");
          }*/

        let mapped_mem = process_mappings.get(&u_page_addr).unwrap();

        // everything quick and dirty

        if mapped_mem.size_in_pages() < page_count {
            process_mappings.remove(&u_page_addr);
            // Create the old mapping and create a new, larger one.
            let mapped_memory = Self::create_mapped_memory(process, u_page_addr, page_count);
            process_mappings.insert(u_page_addr, mapped_memory.clone());
            mapped_memory
        } else {
            mapped_mem.clone()
        }
    }

    fn create_mapped_memory(
        process: &Rc<Process>,
        u_page_addr: u64,
        page_count: u64,
    ) -> MappedMemory {
        let mut mapper = ROOT_MEM_MAPPER.lock();
        let root_process = process.parent().unwrap();

        mapper.mmap(
            &process,
            &root_process,
            u_page_addr,
            None,
            page_count,
            MemCapPermissions::RW,
        )
    }
}

/// Initializes stdout and stderr writers.
/// See [`stdout::StdoutWriter`] and [`stderr::StderrWriter`].
pub fn init_writers(hip: &HIP) {
    stdout::init_writer(hip);
    stderr::init_writer(hip);
}

/// Inits the local EC used by the service portals. Now [`create_and_delegate_service_pts`] can be called.
pub fn init_services(root: &Process) {
    let mut ec_lock = LOCAL_EC.lock();
    assert!(ec_lock.is_none(), "init only allowed once!");

    let utcb_addr = VIRT_MEM_ALLOC
        .lock()
        .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());

    unsafe { LOCAL_EC_STACK.activate_guard_page(RootCapSpace::RootPd.val()) };
    // adds itself to the root process
    let ec = LocalEcObject::create(
        RootCapSpace::RootServiceLocalEc.val(),
        &root.pd_obj(),
        LOCAL_EC_STACK_TOP.val(),
        utcb_addr,
    );
    log::trace!(
        "Created local EC for all service calls (UTCB={:016x})",
        ec.utcb_addr()
    );

    ec_lock.replace(ec);

    // Additional setup out of the loop for the regular service PTs that gets multiplexed
    // via the shared PT entry.
    echo::init_echo_raw_service(root);
}

/// Entry for all services of the roottask.
pub fn handle_service_call(
    pt: &Rc<PtObject>,
    process: &Rc<Process>,
    utcb: &mut Utcb,
    do_reply: &mut bool,
) {
    log::trace!(
        "got service call for service {:?} from Process({}, {})",
        pt.ctx().service_id(),
        process.pid(),
        process.name()
    );
    let cb = match pt.ctx().service_id() {
        ServiceId::StdoutService => stdout::stdout_service_handler,
        ServiceId::StderrService => stderr::stderr_service_handler,
        ServiceId::AllocateService => allocate::allocate_service_handler,
        ServiceId::FileSystemService => fs::fs_service_handler,
        ServiceId::EchoService => echo::echo_service_handler,
        ServiceId::RawEchoService => panic!("the raw echo service is not covered by the PT multiplexing mechanism; has a dedicated entry"),
        _ => panic!("service not supported yet"),
    };
    cb(pt, process, utcb, do_reply);
}

/// Creates the service PTs for a process inside the roottask. Install the PTs in the
/// target PD at well-known locations.
///
/// Call [`init_services`] once first.
pub fn create_and_delegate_service_pts(process: &Process) {
    log::debug!(
        "creating service PTs for process {}, {}",
        process.pid(),
        process.name()
    );

    let cap_base_sel = RootCapSpace::calc_service_pt_sel_base(process.pid());

    // local EC for all service calls
    let ec_lock = LOCAL_EC.lock();
    let ec_lock = ec_lock.as_ref().unwrap();

    // Stdout Service PT
    {
        let stdout_pt = stdout::create_service_pt(cap_base_sel, ec_lock);
        PtObject::delegate(
            &stdout_pt,
            &process.pd_obj(),
            UserAppCapSpace::StdoutServicePT.val(),
        );
        log::trace!("delegated stdout service pt");
    }

    // Stderr Service PT
    {
        let stderr_pt = stderr::create_service_pt(cap_base_sel, ec_lock);
        PtObject::delegate(
            &stderr_pt,
            &process.pd_obj(),
            UserAppCapSpace::StderrServicePT.val(),
        );
    }

    // Alloc Service PT
    {
        let alloc_pt = allocate::create_service_pt(cap_base_sel, ec_lock);
        PtObject::delegate(
            &alloc_pt,
            &process.pd_obj(),
            UserAppCapSpace::AllocatorServicePT.val(),
        );
        log::trace!("delegated alloc service pt");
    }

    // FS Service PT
    {
        let fs_pt = fs::create_service_pt(cap_base_sel, ec_lock);
        PtObject::delegate(
            &fs_pt,
            &process.pd_obj(),
            UserAppCapSpace::FsServicePT.val(),
        );
        log::trace!("delegated fs service pt");
    }

    // ECHO Service PT & RAW ECHO Service PT
    {
        let (echo_service_pt, raw_echo_service_pt) =
            echo::create_service_pts(cap_base_sel, ec_lock);
        PtObject::delegate(
            &echo_service_pt,
            &process.pd_obj(),
            UserAppCapSpace::EchoServicePT.val(),
        );
        PtObject::delegate(
            &raw_echo_service_pt,
            &process.pd_obj(),
            UserAppCapSpace::RawEchoServicePt.val(),
        );
        log::trace!("delegated echo + raw echo service PTs");
    }
}

/// The roottask can use this to create and get the pair of (echo pt, raw echo pt).
/// Useful for benchmarking of PD-internal IPC costs.
pub fn init_roottask_echo_pts() -> (Rc<PtObject>, Rc<PtObject>) {
    let ec_lock = LOCAL_EC.lock();
    let ec_lock = ec_lock.as_ref().expect("call init_services first!");
    echo::create_service_pts_fot_roottask(ec_lock)
}
