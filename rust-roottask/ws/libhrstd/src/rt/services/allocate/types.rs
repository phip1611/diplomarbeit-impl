use core::alloc::Layout;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Like "Layout" but serializable.
#[derive(Serialize, Deserialize, Debug)]
pub struct AllocRequest {
    size: usize,
    align: usize,
}

impl AllocRequest {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn align(&self) -> usize {
        self.align
    }
}

impl From<Layout> for AllocRequest {
    fn from(layout: Layout) -> Self {
        Self {
            size: layout.size(),
            align: layout.align(),
        }
    }
}
