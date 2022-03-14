/// New version of file descriptor that replaces [`libhrstd::rt::services::fs::FD`].
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub struct FileDescriptor(u64);

impl FileDescriptor {
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    pub const fn val(self) -> u64 {
        self.0
    }
}

impl<T> From<T> for FileDescriptor
where
    T: Into<u64>,
{
    fn from(val: T) -> Self {
        FileDescriptor::new(val.into())
    }
}
