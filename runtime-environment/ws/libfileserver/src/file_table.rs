use crate::inode::INode;
use crate::FileDescriptor;
use alloc::collections::BTreeMap;
use libhrstd::process::consts::ProcessId;
use libhrstd::rt::services::fs::FsOpenFlags;

/// Holds information about all files that are currently open inside the system.
#[derive(Debug)]
pub(crate) struct OpenFileTable {
    data: BTreeMap<OpenFileHandleId, OpenFileHandle>,
}

impl OpenFileTable {
    pub(crate) const fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    /// Marks a file as opened and returns a [`FileDescriptor`] that identifies that entry.
    pub(crate) fn open(
        &mut self,
        pid: ProcessId,
        inode: INode,
        flags: FsOpenFlags,
    ) -> Result<FileDescriptor, ()> {
        let fd = self.find_next_fd(pid);
        let key = (pid, fd);
        let value = OpenFileHandle::new(flags, inode);
        self.data.insert(key, value);
        Ok(fd)
    }

    /// Checks if the given process has an opened file with the given file descriptor.
    /// If so, it returns the handle to the open file.
    #[allow(unused)]
    pub(crate) fn lookup_handle(
        &self,
        pid: ProcessId,
        fd: FileDescriptor,
    ) -> Option<&OpenFileHandle> {
        self.data
            .iter()
            .find(|((id_pid, id_fd), _)| *id_pid == pid && *id_fd == fd)
            .map(|(_id, val)| val)
    }

    /// Checks if the given process has an opened file with the given file descriptor.
    /// If so, it returns the handle to the open file.
    pub(crate) fn lookup_handle_mut(
        &mut self,
        pid: ProcessId,
        fd: FileDescriptor,
    ) -> Option<&mut OpenFileHandle> {
        self.data
            .iter_mut()
            .find(|((id_pid, id_fd), _)| *id_pid == pid && *id_fd == fd)
            .map(|(_id, val)| val)
    }

    /// Closes a file.
    pub(crate) fn close(&mut self, caller: ProcessId, fd: FileDescriptor) -> Result<(), ()> {
        let key = (caller, fd);
        self.data.remove(&key).map(|_| ()).ok_or(())
    }

    /// Checks if the passed [`FileDescriptor`]
    fn check_fd_is_in_use(&self, pid: ProcessId, fd_to_check: FileDescriptor) -> bool {
        self.data
            .keys()
            .filter(|(process_id, _)| *process_id == pid)
            .map(|(_pid, fd)| fd.val())
            .any(|fd| fd == fd_to_check.val())
    }

    /// Returns the next available file descriptor for a process.
    fn find_next_fd(&self, pid: ProcessId) -> FileDescriptor {
        // 0-2 reserved for stdin, stdout, stderr
        const MIN_FD: u64 = 3;

        let fd = (MIN_FD..u64::MAX)
            .filter(|fd| !self.check_fd_is_in_use(pid, (*fd).into()))
            .take(1)
            .next()
            .expect("currently I do not expect to run out of FDs :)");

        FileDescriptor::new(fd)
    }
}

/// Combines a process ID that has opened a certain file descriptor.
/// Identifies objects of type [`OpenFileHandle`].
type OpenFileHandleId = (ProcessId, FileDescriptor);

/// Describes an opened file.
#[derive(Debug)]
pub(crate) struct OpenFileHandle {
    // used as ID
    i_node: INode,
    pub(crate) file_offset: usize,
    flags: FsOpenFlags,
}

impl OpenFileHandle {
    pub(crate) fn new(flags: FsOpenFlags, i_node: INode) -> Self {
        OpenFileHandle {
            file_offset: 0,
            flags,
            i_node,
        }
    }

    pub(crate) fn file_offset(&self) -> usize {
        self.file_offset
    }
    pub(crate) fn flags(&self) -> FsOpenFlags {
        self.flags
    }
    pub(crate) fn i_node(&self) -> INode {
        self.i_node
    }
}
