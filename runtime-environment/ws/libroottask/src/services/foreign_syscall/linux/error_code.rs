/// <https://elixir.bootlin.com/linux/latest/source/include/uapi/asm-generic/errno-base.h#L5>
#[derive(Debug)]
#[repr(u64)]
#[allow(unused)]
pub enum LinuxErrorCode {
    /// Operation not permitted
    EPERM = 1,
    /// No such file or directory
    ENOENT = 2,
    /// No such process
    ESRCH = 3,
    /// Interrupted system call
    EINTR = 4,
    /// I/O error
    EIO = 5,
    /// No such device or address
    ENXIO = 6,
    /// Argument list too long
    E2BIG = 7,
    /// Exec format error
    ENOEXEC = 8,
    /// Bad file number
    EBADF = 9,
    /// No child processes
    ECHILD = 10,
    /// Try again
    EAGAIN = 11,
    /// Out of memory
    ENOMEM = 12,
    /// Permission denied
    EACCES = 13,
    /// Bad address
    EFAULT = 14,
    /// Block device required
    ENOTBLK = 15,
    /// Device or resource busy
    EBUSY = 16,
    /// File exists
    EEXIST = 17,
    /// Cross-device link
    EXDEV = 18,
    /// No such device
    ENODEV = 19,
    /// Not a directory
    ENOTDIR = 20,
    /// Is a directory
    EISDIR = 21,
    /// Invalid argument
    EINVAL = 22,
    /// File table overflow
    ENFILE = 23,
    /// Too many open files
    EMFILE = 24,
    /// Not a typewriter
    ENOTTY = 25,
    /// Text file busy
    ETXTBSY = 26,
    /// File too large
    EFBIG = 27,
    /// No space left on device
    ENOSPC = 28,
    /// Illegal seek
    ESPIPE = 29,
    /// Read-only file system
    EROFS = 30,
    /// Too many links
    EMLINK = 31,
    /// Broken pipe
    EPIPE = 32,
    /// Math argument out of domain of func
    EDOM = 33,
    /// Math result not representable
    ERANGE = 34,
}

impl LinuxErrorCode {
    pub fn val(self) -> u64 {
        self as _
    }
}
