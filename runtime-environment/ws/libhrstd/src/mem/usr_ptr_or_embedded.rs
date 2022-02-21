use alloc::vec::Vec;
use core::mem::size_of;
use libhedron::ipc_serde::{
    Deserialize as DeriveDeserialize,
    Serialize as DeriveSerialize,
};
use libhedron::UTCB_DATA_CAPACITY;

/// Used to transfer data through service portals either
/// via a user ptr or via embedded content. Data can be embedded,
/// if the data is less than [`UTCB_DATA_CAPACITY`] bytes long.
#[derive(Debug, DeriveSerialize, DeriveDeserialize)]
pub enum UserPtrOrEmbedded<T: DeriveSerialize + Clone> {
    // usize because raw ptrs are not serializable
    Ptr(usize),
    Embedded(T),
    EmbeddedSlice(Vec<T>),
}

impl<T: DeriveSerialize + Clone> UserPtrOrEmbedded<T> {
    // -2: "postcard" needs additional info for slices for example!
    const CAPACITY: usize = UTCB_DATA_CAPACITY - size_of::<Self>();
    const VEC_CAPACITY: usize = Self::CAPACITY - size_of::<Vec<T>>();

    /// Constructor.
    pub fn new(data: T) -> Self {
        if size_of::<T>() <= Self::CAPACITY {
            Self::Embedded(data)
        } else {
            Self::Ptr(&data as *const _ as usize)
        }
    }

    pub fn new_slice(data: &[T]) -> Self {
        let size_t = size_of::<T>();
        let size = size_t * data.len();
        if size <= Self::VEC_CAPACITY {
            Self::EmbeddedSlice(data.to_vec())
        } else {
            Self::Ptr(data.as_ptr() as usize)
        }
    }

    pub fn ptr(&self) -> *const T {
        self.ptr_mut() as *const _
    }

    #[track_caller]
    pub fn ptr_mut(&self) -> *mut T {
        match self {
            UserPtrOrEmbedded::Ptr(ptr) => *ptr as *mut _,
            _ => panic!("invalid type"),
        }
    }

    #[track_caller]
    pub fn embedded(&self) -> &T {
        match self {
            UserPtrOrEmbedded::Embedded(val) => val,
            _ => panic!("invalid type"),
        }
    }

    #[track_caller]
    pub fn embedded_slice(&self) -> &[T] {
        match self {
            UserPtrOrEmbedded::EmbeddedSlice(val) => val.as_slice(),
            _ => panic!("invalid type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_ptr_or_embedded() {
        let data_small = [0_u8; 2048];
        let data_big = [0_u8; 4096];
        let data_big_ptr = data_big.as_ptr() as usize;

        let usptr_or_embedded_small = UserPtrOrEmbedded::new_slice(&data_small);
        assert_eq!(
            usptr_or_embedded_small.embedded_slice(),
            UserPtrOrEmbedded::EmbeddedSlice(data_small.to_vec()).embedded_slice(),
        );
        let usptr_or_embedded_big = UserPtrOrEmbedded::new_slice(&data_big);
        assert_eq!(
            usptr_or_embedded_big.ptr(),
            UserPtrOrEmbedded::Ptr(data_big_ptr).ptr()
        );

        // now test that everything is serializable as expected

        let mut serialized_small = [0; UTCB_DATA_CAPACITY];
        libhedron::ipc_postcard::to_slice(&usptr_or_embedded_small, &mut serialized_small).unwrap();
        let deserialized_small =
            libhedron::ipc_postcard::from_bytes::<UserPtrOrEmbedded<u8>>(&serialized_small)
                .unwrap();
        assert_eq!(
            usptr_or_embedded_small.embedded_slice(),
            deserialized_small.embedded_slice()
        );

        let mut serialized_big = [0; UTCB_DATA_CAPACITY];
        libhedron::ipc_postcard::to_slice(&usptr_or_embedded_big, &mut serialized_big).unwrap();
        let deserialized_small =
            libhedron::ipc_postcard::from_bytes::<UserPtrOrEmbedded<u8>>(&serialized_big).unwrap();
        assert_eq!(
            deserialized_small.ptr(),
            UserPtrOrEmbedded::Ptr(data_big_ptr).ptr()
        );
    }
}
