/// Syscall ABI or OS Personality of a [`super::process::Process`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SyscallAbi {
    NativeHedron,
    Linux,
}

impl SyscallAbi {
    pub fn is_native(self) -> bool {
        matches!(self, Self::NativeHedron)
    }

    pub fn is_foreign(self) -> bool {
        !self.is_native()
    }
}

impl Default for SyscallAbi {
    fn default() -> Self {
        Self::NativeHedron
    }
}
