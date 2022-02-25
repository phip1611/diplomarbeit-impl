use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::{
    GenericLinuxSyscall,
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
//use core::alloc::Layout;
//use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::UtcbDataException;
//use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

#[derive(Debug)]
pub struct CloneSyscall {
    // poorly documented ... I took this from the musl code
    // maybe this is the right linux code: https://elixir.bootlin.com/linux/v5.16.10/source/kernel/fork.c#L2677
    _fnc_ptr: u64,
    _child_stack: u64,
    _flags: u64,
    // flags: CloneFlags,
    _args: *const u8,
    _ptid: u64,
    _tls: u64,
}

impl From<&GenericLinuxSyscall> for CloneSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            _fnc_ptr: syscall.arg0(),
            _child_stack: syscall.arg1(),
            // flags: CloneFlags::from_bits(syscall.arg2()).unwrap(),
            _flags: syscall.arg2(),
            _args: syscall.arg3() as *const _,
            _ptid: syscall.arg4(),
            _tls: syscall.arg5(),
        }
    }
}

impl LinuxSyscallImpl for CloneSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        _process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        log::info!("Clone: {:#?}", self);

        // Quick and dirty: afterwards, the Haskell binary wants to access
        // the memory behind the TLS address

        /*let r_heap =
            unsafe { alloc::alloc::alloc(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap()) };

        CrdDelegateOptimizer::new(
            r_heap as u64 / PAGE_SIZE as u64,
            self.tls / PAGE_SIZE as u64,
            1,
        )
        .mmap(
            process.parent().unwrap().pd_obj().cap_sel(),
            process.pd_obj().cap_sel(),
            MemCapPermissions::READ | MemCapPermissions::WRITE | MemCapPermissions::EXECUTE,
        );*/

        LinuxSyscallResult::new_success(0)
    }
}

bitflags::bitflags! {
    #[allow(unused)]
    struct CloneFlags: u64 {
        /// signal mask to be sent at exit
        const CSIGNAL = 0x000000ff;
        /// set if VM shared between processes
        const VM = 0x00000100;
        /// set if fs info shared between processes
        const FS = 0x00000200;
        /// set if open files shared between processes
        const FILES = 0x00000400;
        /// set if signal handlers and blocked signals shared
        const SIGHAND = 0x00000800;
        /// set if a pidfd should be placed in parent
        const PIDFD = 0x00001000;
        /// set if we want to let tracing continue on the child too
        const PTRACE = 0x00002000;
        /// set if the parent wants the child to wake it up on mm_release
        const VFORK = 0x00004000;
        /// set if we want to have the same parent as the cloner
        const PARENT = 0x00008000;
        /// Same thread group?
        const THREAD = 0x00010000;
        /// New mount namespace group
        const NEWNS = 0x00020000;
        /// share system V SEM_UNDO semantics
        const SYSVSEM = 0x00040000;
        /// create a new TLS for the child
        const SETTLS = 0x00080000;
        /// set the TID in the parent
        const PARENT_SETTID = 0x00100000;
        /// clear the TID in the child
        const CHILD_CLEARTID = 0x00200000;
        /// Unused, ignored
        const DETACHED = 0x00400000;
        /// set if the tracing process can't force PTRACE on this clone
        const UNTRACED = 0x00800000;
        /// set the TID in the child
        const CHILD_SETTID = 0x01000000;
        /// New cgroup namespace
        const NEWCGROUP = 0x02000000;
        /// New utsname namespace
        const NEWUTS = 0x04000000;
        /// New ipc namespace
        const NEWIPC = 0x08000000;
        /// New user namespace
        const NEWUSER = 0x10000000;
        /// New pid namespace
        const NEWPID = 0x20000000;
        /// New network namespace
        const NEWNET = 0x40000000;
        /// Clone io context
        const IO = 0x80000000;
    }
}
