use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::consts::LINUX_PATH_MAX;
use crate::services::foreign_syscall::linux::error_code::LinuxErrorCode;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use crate::services::MAPPED_AREAS;
use alloc::rc::Rc;
use libhrstd::cstr::CStr;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FsOpenFlags;

#[derive(Debug)]
pub struct OpenSyscall {
    // null terminated file name
    filename: *const u8,
    flags: FsOpenFlags,
    umode: u64,
}

impl From<&GenericLinuxSyscall> for OpenSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            filename: syscall.arg0() as *const _,
            flags: FsOpenFlags::from_bits(syscall.arg1() as u32).unwrap(),
            umode: syscall.arg2(),
        }
    }
}

impl LinuxSyscallImpl for OpenSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        let mapping = MAPPED_AREAS
            .lock()
            .create_or_get_mapping(process, self.filename as u64, LINUX_PATH_MAX as u64)
            .clone();

        let u_page_offset = self.filename as usize & 0xfff;
        let filename = mapping.mem_with_offset_as_slice::<u8>(LINUX_PATH_MAX, u_page_offset);
        let filename = CStr::try_from(filename).unwrap();
        // remove null bytes
        let filename = filename.as_str().trim_matches('\0');

        let fd = libfileserver::FILESYSTEM.lock().open_or_create_file(
            process.pid(),
            filename,
            self.flags,
            self.umode as u16,
        );

        if let Ok(fd) = fd {
            LinuxSyscallResult::new_success(fd.val())
        } else {
            LinuxSyscallResult::new_error(LinuxErrorCode::EINVAL)
        }
    }
}
