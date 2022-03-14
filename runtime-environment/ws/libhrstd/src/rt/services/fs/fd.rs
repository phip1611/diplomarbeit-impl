use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Either a fd (>=0) or error code (<0)
///
/// TODO remove type; make file descriptor only positive
///  and use Result<> for errors: move this to a "Linux File Descriptor" type in the
///  OS personality of Linux! And replace this in the service with "FileDescriptor" type
///  from libfileserver
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Ord, Eq, Serialize, Deserialize)]
pub struct FD(i32);

impl FD {
    const ERROR_VALUE: i32 = -1;

    pub fn new(fd: i32) -> Self {
        Self(fd)
    }

    pub fn error() -> Self {
        Self::new(Self::ERROR_VALUE)
    }

    pub fn get(self) -> Result<i32, ()> {
        if self.raw() == -1 {
            Err(())
        } else {
            Ok(self.raw())
        }
    }

    pub fn raw(self) -> i32 {
        self.0
    }
}
