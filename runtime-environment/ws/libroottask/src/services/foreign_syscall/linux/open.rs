use crate::process_mng::process::Process;
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
use libhrstd::rt::services::fs::FsOpenFlags;

// TODO find this value in Linux code and
// put the constant somewhere else
const MAX_FILE_NAME_LEN: usize = 255;

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
            .create_get_mapping(process, self.filename as u64, MAX_FILE_NAME_LEN as u64)
            .clone();

        let u_page_offset = self.filename as usize & 0xfff;
        let filename = mapping.mem_with_offset_as_slice::<u8>(MAX_FILE_NAME_LEN, u_page_offset);
        let filename = CStr::try_from(filename).unwrap();
        // remove null bytes
        let filename = filename.as_str().trim_matches('\0').to_string();

        let fd = libfileserver::fs_open(process.pid(), filename, self.flags, self.umode as u16);

        LinuxSyscallResult::new_success(fd.get().unwrap() as u64)
    }
}
