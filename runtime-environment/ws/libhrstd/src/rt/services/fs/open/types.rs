use alloc::string::String;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

bitflags::bitflags! {
    /// Flags that can be used for the `open()` system call. The interface is similar
    /// to the one by Linux.
    ///
    /// Flags that can be used here are specified in:
    /// - https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/fcntl.h
    /// - https://github.com/torvalds/linux/blob/master/include/linux/fcntl.h
    ///
    /// Most of these information are in the manpage: `$ man open`
    ///
    /// Linux defines each variant using the octal number format.
    // TODO It feels like this should belong to libfileserver
    #[derive(Serialize, Deserialize)]
    pub struct FsOpenFlags: u32 {
        /// Open for reading only.
        const O_RDONLY = 0o0;
        /// Open for writing only.
        const O_WRONLY = 0o1;
        /// Opens a file for reading and writing.
        const O_RDWR = 0o2;
        /// Create file if it doesn't exist.
        const O_CREAT = 0o100;
        /// Truncates the file
        const O_TRUNC = 0o1000;
        /// Append for all writes, regardless of the current file pointer.
        const O_APPEND = 0o2000;
        /// O_LARGEFILE should never be used directly by applications.
        /// It's to be used internally by the 64-bit-offset-compatible
        /// version of open in libc when it makes the syscall to the kernel
        /// (Linux, or possibly another kernel with this 64-bit-offset-mode-is-
        /// a-second-class-citizen nonsense). Just make sure to always include
        /// -D_FILE_OFFSET_BITS=64 in your CFLAGS and you'll never have to
        /// worry about anything.
        const O_LARGEFILE = 0o100000;
        /// On EXEC-Calls the FD must be closed.
        const O_CLOEXEC = 0o2000000;
    }
}

impl FsOpenFlags {
    pub fn can_read(self) -> bool {
        self.contains(Self::O_RDONLY) || self.contains(Self::O_RDWR)
    }
    pub fn can_write(self) -> bool {
        self.contains(Self::O_WRONLY) || self.contains(Self::O_RDWR)
    }
    pub fn is_append(self) -> bool {
        self.contains(Self::O_APPEND)
    }
    pub fn can_create(self) -> bool {
        self.contains(Self::O_CREAT)
    }
}

/// Data send via UTCB to Fs Open Portal.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsOpenRequest {
    path: String,
    flags: FsOpenFlags,
    umode: u16,
}

impl FsOpenRequest {
    pub fn new(path: String, flags: FsOpenFlags, umode: u16) -> Self {
        FsOpenRequest { path, flags, umode }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn flags(&self) -> FsOpenFlags {
        self.flags
    }
    pub fn umode(&self) -> u16 {
        self.umode
    }
}
