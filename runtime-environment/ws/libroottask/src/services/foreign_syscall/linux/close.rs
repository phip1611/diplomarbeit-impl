use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libfileserver::FileDescriptor;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

#[derive(Debug)]
pub struct CloseSyscall {
    // TODO refactor this into a "Linux File Descriptor" that is cldoe to the existing FD type
    fd: FileDescriptor,
}

impl From<&GenericLinuxSyscall> for CloseSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FileDescriptor::new(syscall.arg0()),
        }
    }
}

impl LinuxSyscallImpl for CloseSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        libfileserver::FILESYSTEM
            .lock()
            .close_file(process.pid(), self.fd)
            .unwrap();

        LinuxSyscallResult::new_success(0)
    }
}
