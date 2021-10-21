//! User Address Space definitions.

use crate::libhedron::mem::{
    MAX_USER_ADDR,
    PAGE_SIZE,
};

/// Virtual address of the [`UTCB`] in user processes.
pub const VIRT_UTCB_ADDR: u64 = (MAX_USER_ADDR - PAGE_SIZE) as u64;

/// Page number of [`VIRT_UTCB_ADDR`].
pub const VIRT_UTCB_PAGE_NUM: u64 = VIRT_UTCB_ADDR / PAGE_SIZE as u64;

/// Virtual address of stack top in user processes.
/// -64: 512 bit alignment
/// + 8: stack offset for correct alignment
pub const VIRT_STACK_TOP: u64 = VIRT_UTCB_ADDR - 64 + 8;

/// The bottom address of the stack.
pub const VIRT_STACK_BOTTOM_ADDR: u64 = VIRT_UTCB_ADDR - USER_STACK_SIZE as u64;

/// The page number of [`VIRT_STACK_BOTTOM_ADDR`].
pub const VIRT_STACK_BOTTOM_PAGE_NUM: u64 = VIRT_STACK_BOTTOM_ADDR / PAGE_SIZE as u64;

/// 128KiB stack size for all Hedron-native apps. A multiple of [`PAGE_SIZE`].
pub const USER_STACK_SIZE: usize = 0x20000;
