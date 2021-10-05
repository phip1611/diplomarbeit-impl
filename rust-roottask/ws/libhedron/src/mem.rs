//! Helper methods and constants for managing memory under Hedron.

/// Size of a page on x86_64. Also stands inside the HIP. As long as Hedron
/// only runs on x86_64, this will probably never change.
pub const PAGE_SIZE: usize = 4096;
