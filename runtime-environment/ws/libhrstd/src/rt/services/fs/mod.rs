mod close;
mod fd;
mod lseek;
mod open;
mod read;
mod request;
mod write;

// types
#[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
pub use close::fs_service_close;
pub use close::FsCloseRequest;
pub use fd::FD;
#[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
pub use lseek::fs_service_lseek;
pub use lseek::FsLseekRequest;
#[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
pub use open::fs_service_open;
pub use open::{
    FsOpenFlags,
    FsOpenRequest,
};
#[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
pub use read::fs_service_read;
pub use read::FsReadRequest;
pub use request::FsServiceRequest;
#[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
pub use write::fs_service_write;
pub use write::FsWriteRequest;
