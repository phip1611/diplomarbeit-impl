use enum_iterator::IntoEnumIterator;

/// Lists all services that are available by default in the runtime environment.
#[derive(Copy, Clone, Debug, IntoEnumIterator)]
#[repr(u64)]
pub enum ServiceId {
    StdoutService,
    StderrService,
    AllocateService,
    _Count,
}

impl ServiceId {
    /// Returns the number of variants.
    pub const fn count() -> u64 {
        Self::_Count.val()
    }

    /// Returns the numeric value.
    pub const fn val(self) -> u64 {
        self as _
    }
}
