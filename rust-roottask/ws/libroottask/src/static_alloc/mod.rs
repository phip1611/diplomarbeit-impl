//! Allocator for the Roottask. The roottask gets its initial heap from a static array. This
//! memory is available because its part of the LOAD section of the ELF and the kernel loads
//! the ELF as expected into memory.
//!
//! The roottask has all memory anyway. But this approach simplifies things a bit because
//! I don't have to look for free pages manually. The approach is good enough for this thesis.
//!
//! The allocator from this module must be stored in a global static variable and
//! reference memory from the global static backing storage.
//!
//! The two most important types are [`StaticAlignedMem`] and [`GlobalStaticChunkAllocator`].

mod chunk;
mod global_static_alloc;
mod static_aligned_mem;

pub use global_static_alloc::{
    GlobalStaticChunkAllocator,
    GlobalStaticChunkAllocatorError,
};
pub use static_aligned_mem::StaticAlignedMem;
