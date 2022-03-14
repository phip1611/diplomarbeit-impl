use crate::process::Process;
use crate::services::foreign_syscall::linux::consts::LINUX_PATH_MAX;
use crate::services::foreign_syscall::linux::error_code::LinuxErrorCode;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;
use alloc::string::ToString;
use libhrstd::cstr::CStr;
use libhrstd::libhedron::UtcbDataException;

#[derive(Debug)]
pub struct UnlinkSyscall {
    u_filename: *const u8,
}

impl From<&GenericLinuxSyscall> for UnlinkSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            u_filename: syscall.arg0() as *const _,
        }
    }
}

impl LinuxSyscallImpl for UnlinkSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        let mapping = MAPPED_AREAS
            .lock()
            .create_or_get_mapping(process, self.u_filename as u64, LINUX_PATH_MAX as u64)
            .clone();

        let u_page_offset = self.u_filename as usize & 0xfff;
        let filename = mapping.mem_with_offset_as_slice::<u8>(LINUX_PATH_MAX, u_page_offset);
        let filename = CStr::try_from(filename).unwrap();
        // remove null bytes
        let filename = filename.as_str().trim_matches('\0').to_string();

        if libfileserver::FILESYSTEM
            .lock()
            .unlink_file(process.pid(), &filename)
            .is_ok()
        {
            LinuxSyscallResult::new_success(0)
        } else {
            LinuxSyscallResult::new_error(LinuxErrorCode::EINVAL)
        }
    }
}
