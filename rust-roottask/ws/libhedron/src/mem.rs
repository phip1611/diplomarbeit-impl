//! Helper methods and constants for managing memory under Hedron.

/// Size of a page on x86_64. Also stands inside the HIP. As long as Hedron
/// only runs on x86_64, this will probably never change.
pub const PAGE_SIZE: usize = 4096;

/// Maximum virtual address inside the address space of user applications (page-aligned).
pub const MAX_USER_ADDR: usize = 0x7ffffffff000;
