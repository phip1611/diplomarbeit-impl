mod aligned;
mod usr_ptr_or_embedded;

pub use aligned::*;
use libhedron::mem::PAGE_SIZE;
pub use usr_ptr_or_embedded::*;

/// Calculates the number of needed pages to cover all bytes.
/// Always rounds up to the next full page.
pub const fn calc_page_count(size: usize) -> usize {
    if size % PAGE_SIZE == 0 {
        size / PAGE_SIZE
    } else {
        (size / PAGE_SIZE) + 1
    }
}
