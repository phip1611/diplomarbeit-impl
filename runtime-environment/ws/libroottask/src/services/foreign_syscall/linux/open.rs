use crate::mem::VIRT_MEM_ALLOC;
use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::string::String;
use core::alloc::Layout;
use libhrstd::cstr::CStr;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::{
    MemCapPermissions,
    UtcbDataException,
};
use libhrstd::mem::calc_page_count;
use libhrstd::rt::services::fs::FsOpenFlags;
use libhrstd::util::crd_delegate_optimizer::CrdDelegateOptimizer;

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
    fn handle(&self, _utcb_exc: &mut UtcbDataException, process: &Process) -> LinuxSyscallResult {
        let u_page_offset = self.filename as usize & 0xfff;
        let byte_amount = u_page_offset + MAX_FILE_NAME_LEN;
        let page_count = calc_page_count(byte_amount);

        let r_mapping = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(byte_amount, PAGE_SIZE).unwrap());

        CrdDelegateOptimizer::new(
            self.filename as u64 / PAGE_SIZE as u64,
            r_mapping / PAGE_SIZE as u64,
            page_count,
        )
        .mmap(
            // from calling process
            process.pd_obj().cap_sel(),
            // to roottask
            process.parent().unwrap().pd_obj().cap_sel(),
            MemCapPermissions::READ,
        );

        let r_filename_ptr = (r_mapping + u_page_offset as u64) as *const u8;
        let filename = unsafe { core::slice::from_raw_parts(r_filename_ptr, MAX_FILE_NAME_LEN) };
        let filename = CStr::try_from(filename).unwrap();
        let filename = String::from(filename.as_str());

        let fd = libfileserver::fs_open(process.pid(), filename, self.flags, self.umode as u16);

        LinuxSyscallResult::new_success(fd.get().unwrap() as u64)
    }
}
