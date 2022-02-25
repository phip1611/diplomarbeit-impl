use enum_iterator::IntoEnumIterator;

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
#[repr(u64)]
pub enum LinuxSyscallNum {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Fstat = 5,
    Poll = 7,
    LSeek = 8,
    MMap = 9,
    MProtect = 10,
    MUnmap = 11,
    Brk = 12,
    RtSigaction = 13,
    RtSigprocmask = 14,
    Ioctl = 16,
    MAdvise = 28,
    WriteV = 20,
    Clone = 56,
    Fcntl = 72,
    Unlink = 87,
    SigAltStack = 131,
    ArchPrctl = 158,
    Gettid = 186,
    Futex = 202,
    SchedGetAffinity = 204,
    SetTidAddress = 218,
    ExitGroup = 231,
    ReadLinkAt = 267,
    ClockGetTime = 228,
    PrLimit64 = 302,
}

impl LinuxSyscallNum {
    pub fn val(self) -> u64 {
        self as u64
    }
}

impl TryFrom<u64> for LinuxSyscallNum {
    type Error = ();
    fn try_from(val: u64) -> Result<Self, Self::Error> {
        // generated during compile time; probably not recognized by IDE
        for variant in Self::into_enum_iter() {
            if variant.val() == val {
                return Ok(variant);
            }
        }
        log::warn!("linux syscall {} not typed yet!", val);
        Err(())
    }
}
