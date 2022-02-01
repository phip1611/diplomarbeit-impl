use crate::rt::services::fs::FsCloseRequest;
use crate::rt::services::fs::FsLseekRequest;
use crate::rt::services::fs::FsOpenRequest;
use crate::rt::services::fs::FsReadRequest;
use crate::rt::services::fs::FsWriteRequest;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Used to multiplex all FS requests through a single portal.
#[derive(Serialize, Deserialize, Debug)]
pub enum FsServiceRequest {
    Open(FsOpenRequest),
    Read(FsReadRequest),
    LSeek(FsLseekRequest),
    Write(FsWriteRequest),
    Close(FsCloseRequest),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rt::services::fs::FsOpenFlags;

    #[test]
    fn test_compiles() {
        let _ = FsServiceRequest::Open(FsOpenRequest::new(
            String::from("/foo/bar"),
            FsOpenFlags::O_CREAT | FsOpenFlags::O_WRONLY,
            0o777,
        ));
    }

    #[test]
    fn test_serialization() {
        let obj = FsServiceRequest::Open(FsOpenRequest::new(
            String::from("/foo/bar"),
            FsOpenFlags::O_RDWR,
            0o777,
        ));
        let mut buf = vec![0; 16];
        let serialized = libhedron::ipc_postcard::to_slice(&obj, buf.as_mut_slice()).unwrap();
        let deserialized =
            libhedron::ipc_postcard::from_bytes::<FsServiceRequest>(serialized).unwrap();
        dbg!(deserialized);
    }
}
