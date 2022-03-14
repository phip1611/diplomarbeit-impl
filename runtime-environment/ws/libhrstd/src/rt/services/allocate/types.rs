use core::alloc::Layout;
use libhedron::ipc_serde::{
    Deserialize,
    Serialize,
};

/// Describes an allocation request similar to mmap that Hedron-native
/// apps can trigger.
///
/// Like "Layout" but serializable.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum AllocRequest {
    Alloc { size: usize, align: usize },
    Dealloc { ptr: u64, size: usize, align: usize },
}

impl AllocRequest {
    pub fn new_alloc(layout: Layout) -> Self {
        Self::Alloc {
            size: layout.size(),
            align: layout.align(),
        }
    }

    pub fn new_delloc(ptr: u64, layout: Layout) -> Self {
        Self::Dealloc {
            ptr,
            size: layout.size(),
            align: layout.align(),
        }
    }

    pub fn to_layout(self) -> Layout {
        Layout::from_size_align(self.size(), self.align()).unwrap()
    }

    pub fn size(&self) -> usize {
        match self {
            AllocRequest::Alloc { size, .. } => *size,
            AllocRequest::Dealloc { size, .. } => *size,
        }
    }
    pub fn align(&self) -> usize {
        match self {
            AllocRequest::Alloc { align, .. } => *align,
            AllocRequest::Dealloc { align, .. } => *align,
        }
    }

    pub fn ptr(&self) -> Option<u64> {
        match self {
            AllocRequest::Dealloc { ptr, .. } => Some(*ptr),
            _ => None,
        }
    }

    pub fn is_allocation(&self) -> bool {
        matches!(self, Self::Alloc { .. })
    }

    pub fn is_deallocation(&self) -> bool {
        matches!(self, Self::Dealloc { .. })
    }
}
