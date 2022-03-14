use crate::in_mem_fs::InMemFile;

/// This is identical to the UNIX/libc stat type.
#[repr(C)]
#[derive(Debug)]
pub struct FileStat {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

impl FileStat {
    pub fn st_dev(&self) -> u64 {
        self.st_dev
    }
    pub fn st_ino(&self) -> u64 {
        self.st_ino
    }
    pub fn st_nlink(&self) -> u64 {
        self.st_nlink
    }
    pub fn st_mode(&self) -> u32 {
        self.st_mode
    }
    pub fn st_uid(&self) -> u32 {
        self.st_uid
    }
    pub fn st_gid(&self) -> u32 {
        self.st_gid
    }
    pub fn st_rdev(&self) -> u64 {
        self.st_rdev
    }
    pub fn st_size(&self) -> i64 {
        self.st_size
    }
    pub fn st_blksize(&self) -> i64 {
        self.st_blksize
    }
    pub fn st_blocks(&self) -> i64 {
        self.st_blocks
    }
    pub fn st_atime(&self) -> i64 {
        self.st_atime
    }
    pub fn st_atime_nsec(&self) -> i64 {
        self.st_atime_nsec
    }
    pub fn st_mtime(&self) -> i64 {
        self.st_mtime
    }
    pub fn st_mtime_nsec(&self) -> i64 {
        self.st_mtime_nsec
    }
    pub fn st_ctime(&self) -> i64 {
        self.st_ctime
    }
    pub fn st_ctime_nsec(&self) -> i64 {
        self.st_ctime_nsec
    }
}

impl From<&InMemFile> for FileStat {
    fn from(file: &InMemFile) -> Self {
        Self {
            st_dev: 0,
            st_ino: file.i_node().val(),
            st_nlink: 0,
            st_mode: file.meta().umode() as u32,
            st_uid: 0,
            st_gid: 0,
            __pad0: 0,
            st_rdev: 0,
            st_size: file.data().len() as i64,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused: [0; 3],
        }
    }
}
