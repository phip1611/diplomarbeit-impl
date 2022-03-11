use crate::inode::INode;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use libhrstd::process::consts::ProcessId;

#[derive(Debug)]
pub(crate) struct FileMetaData {
    umode: u16,
    owner: ProcessId,
}

impl FileMetaData {
    pub(crate) fn new(umode: u16, owner: ProcessId) -> Self {
        FileMetaData { umode, owner }
    }

    pub(crate) fn umode(&self) -> u16 {
        self.umode
    }
    #[allow(unused)]
    pub(crate) fn owner(&self) -> ProcessId {
        self.owner
    }
}

/// An in-memory file.
#[derive(Debug)]
pub(crate) struct InMemFile {
    // used as ID
    i_node: INode,
    path: String,
    data: Vec<u8>,
    meta: FileMetaData,
}

impl InMemFile {
    /// Each file has a default capacity of 64 KiB. This prevents relatively expensive
    /// allocations for small file operations.
    pub(crate) const DEFAULT_CAPACITY: usize = 0x10000;

    pub(crate) fn new(i_node: INode, path: String, meta: FileMetaData) -> Self {
        Self {
            i_node,
            path,
            data: Vec::with_capacity(Self::DEFAULT_CAPACITY),
            meta,
        }
    }
    pub(crate) fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub(crate) fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    pub(crate) fn path(&self) -> &String {
        &self.path
    }
    pub(crate) fn meta(&self) -> &FileMetaData {
        &self.meta
    }
    pub(crate) fn i_node(&self) -> INode {
        self.i_node
    }
    #[cfg(test)]
    pub(crate) fn inner_vec(&self) -> &Vec<u8> {
        &self.data
    }
}

/// The in-memory file system is implemented as a binary tree map
/// from [`INode`] to [`InMemFile`].
#[derive(Debug)]
pub(crate) struct InMemFilesystem {
    files: BTreeMap<INode, InMemFile>,
}

impl InMemFilesystem {
    pub(crate) const fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub(crate) fn create_file(&mut self, i_node: INode, file: InMemFile) -> Result<(), ()> {
        if self.files.contains_key(&i_node) {
            Err(())
        } else {
            self.files.insert(i_node, file);
            Ok(())
        }
    }

    pub(crate) fn get_file_by_inode(&self, i_node: INode) -> Option<&InMemFile> {
        self.files
            .iter()
            .map(|(_, file)| file)
            .find(|file| file.i_node() == i_node)
    }

    pub(crate) fn get_file_by_inode_mut(&mut self, i_node: INode) -> Option<&mut InMemFile> {
        self.files
            .iter_mut()
            .map(|(_, file)| file)
            .find(|file| file.i_node() == i_node)
    }

    fn get_entry_by_path(&self, filepath: &str) -> Option<(&INode, &InMemFile)> {
        self.files.iter().find(|(_, file)| file.path() == filepath)
    }

    fn get_entry_by_path_mut(&mut self, filepath: &str) -> Option<(&INode, &mut InMemFile)> {
        self.files
            .iter_mut()
            .find(|(_, file)| file.path() == filepath)
    }

    pub(crate) fn get_file_by_path(&self, filepath: &str) -> Option<&InMemFile> {
        self.get_entry_by_path(filepath).map(|(_, value)| value)
    }

    #[allow(unused)]
    pub(crate) fn get_file_by_path_mut(&mut self, filepath: &str) -> Option<&mut InMemFile> {
        self.get_entry_by_path_mut(filepath).map(|(_, value)| value)
    }

    pub(crate) fn delete_file_by_path(&mut self, filepath: &str) -> bool {
        let key = self
            .get_entry_by_path(filepath)
            .map(|(key, _)| key)
            // prevents borrow issue; copy is cheap here
            .copied();

        key.map(|key| self.files.remove(&key).is_some())
            .unwrap_or(false)
    }
}
