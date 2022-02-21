use crate::process_mng::process::Process;
use crate::services::foreign_syscall::linux::generic::GenericLinuxSyscall;
use crate::services::foreign_syscall::linux::{
    LinuxSyscallImpl,
    LinuxSyscallResult,
};
use alloc::rc::Rc;
use libhrstd::libhedron::UtcbDataException;
use libhrstd::rt::services::fs::FD;

#[derive(Debug)]
pub struct LSeekSyscall {
    fd: FD,
    offset: u64,
    _whence: LSeekWhence,
}

impl From<&GenericLinuxSyscall> for LSeekSyscall {
    fn from(syscall: &GenericLinuxSyscall) -> Self {
        Self {
            fd: FD::new(syscall.arg0() as i32),
            offset: syscall.arg1(),
            _whence: LSeekWhence::from(syscall.arg2()),
        }
    }
}

impl LinuxSyscallImpl for LSeekSyscall {
    fn handle(
        &self,
        _utcb_exc: &mut UtcbDataException,
        process: &Rc<Process>,
    ) -> LinuxSyscallResult {
        // TODO whence not considered yet
        libfileserver::fs_lseek(process.pid(), self.fd, self.offset as usize).unwrap();

        LinuxSyscallResult::new_success(0)
    }
}

#[derive(Debug)]
enum LSeekWhence {
    /// The file offset is set to offset bytes.
    SeekSet = 0,
    /// The file offset is set to its current location plus offset bytes.
    SeekCur = 1,
    /// The file offset is set to the size of the file plus offset bytes.
    SeekEnd = 2,
    /// Adjust the file offset to the next location in the file
    /// greater than or equal to offset containing data.  If
    /// offset points to data, then the file offset is set to
    /// offset.
    SeekData = 3,
    /// Adjust the file offset to the next hole in the file
    /// greater than or equal to offset.  If offset points into
    /// the middle of a hole, then the file offset is set to
    /// offset.  If there is no hole past offset, then the file
    /// offset is adjusted to the end of the file (i.e., there is
    /// an implicit hole at the end of any file).
    SeekHole = 4,
}

impl From<u64> for LSeekWhence {
    fn from(val: u64) -> Self {
        match val {
            0 => Self::SeekSet,
            1 => Self::SeekCur,
            2 => Self::SeekEnd,
            3 => Self::SeekData,
            4 => Self::SeekHole,
            _ => panic!("unknown variant"),
        }
    }
}
