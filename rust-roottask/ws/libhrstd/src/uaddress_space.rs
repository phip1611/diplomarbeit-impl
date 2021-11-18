//! User Address Space definitions.

use crate::libhedron::mem::{
    PAGE_SIZE,
    USER_MAX_ADDR,
};

/// Virtual page-aligned address of the [`UTCB`] in user processes.
pub const USER_UTCB_ADDR: u64 = (USER_MAX_ADDR - PAGE_SIZE) as u64;

/// Page number of [`VIRT_UTCB_ADDR`].
pub const USER_UTCB_PAGE_NUM: u64 = USER_UTCB_ADDR / PAGE_SIZE as u64;

/// Virtual address of initial stack top in user processes.
/// -64: 512 bit alignment
/// + 8: stack offset for correct alignment of first argument
pub const USER_STACK_TOP: u64 = USER_UTCB_ADDR - 64 + 8;

/// The page-aligned bottom address of the stack.
pub const USER_STACK_BOTTOM_ADDR: u64 = USER_UTCB_ADDR - USER_STACK_SIZE as u64;

/// The page number of [`VIRT_STACK_BOTTOM_ADDR`].
pub const USER_STACK_BOTTOM_PAGE_NUM: u64 = USER_STACK_BOTTOM_ADDR / PAGE_SIZE as u64;

/// 128KiB stack size for all Hedron-native apps. A multiple of [`PAGE_SIZE`].
pub const USER_STACK_SIZE: usize = 64 * PAGE_SIZE;

/// Begin of the heap. No text or data segment is allowed to clash with this.
pub const USER_HEAP_BEGIN: usize = 0x40000000;
