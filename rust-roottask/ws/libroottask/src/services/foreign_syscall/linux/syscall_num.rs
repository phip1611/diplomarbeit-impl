use enum_iterator::IntoEnumIterator;

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
#[repr(u64)]
pub enum LinuxSyscallNum {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Poll = 7,
    MMap = 9,
    MProtect = 10,
    MUnmap = 11,
    Brk = 12,
    RtSigaction = 13,
    RtSigprocmask = 14,
    Ioctl = 16,
    WriteV = 20,
    Clone = 56,
    Fcntl = 72,
    SigAltStack = 131,
    ArchPrctl = 158,
    Gettid = 186,
    Futex = 202,
    SchedGetAffinity = 204,
    SetTidAddress = 218,
    ExitGroup = 231,
    ReadLinkAt = 267,
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
        Err(())
    }
}
