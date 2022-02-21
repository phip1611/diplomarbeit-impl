//! User Address Space definitions.

use crate::libhedron::mem::{
    PAGE_SIZE,
    USER_MAX_ADDR,
};

/// Virtual page-aligned address of the [`UTCB`] in user processes.
/// So far this is the UTCB of global EC 1. No further UTCBs supported yet.
pub const USER_UTCB_ADDR: u64 = (USER_MAX_ADDR - PAGE_SIZE) as u64;

/// Page number of [`VIRT_UTCB_ADDR`].
pub const USER_UTCB_PAGE_NUM: u64 = USER_UTCB_ADDR / PAGE_SIZE as u64;

/// The very top exclusive(!) address of the user stack. The first valid
/// and mapped byte is at this address minus 1.
pub const USER_STACK_VERY_TOP: u64 = USER_UTCB_ADDR;

/// Virtual address of initial stack top in user processes.
/// -64: 512 bit alignment
/// + 8: stack offset for correct alignment of first argument.
pub const USER_STACK_TOP: u64 = USER_STACK_VERY_TOP - 64 + 8;

/// The page-aligned bottom address of the stack.
pub const USER_STACK_BOTTOM_ADDR: u64 = USER_UTCB_ADDR - USER_STACK_SIZE as u64;

/// The page number of [`USER_STACK_BOTTOM_ADDR`].
pub const USER_STACK_BOTTOM_PAGE_NUM: u64 = USER_STACK_BOTTOM_ADDR / PAGE_SIZE as u64;

/// 2 MiB stack size for all Hedron user apps. A multiple of [`PAGE_SIZE`].
/// Linux default is 10MB, Windows default is 1MB. That big because I need to write large
/// amounts of data in my FS micro benchmark.
pub const USER_STACK_SIZE: usize = 512 * PAGE_SIZE;

/// Some libc implementation, such as musl, need to read the program headers of their
/// ELF file. This is the user address where the ELF file program headers shall be
/// mapped.
pub const USER_ELF_ADDR: u64 = USER_STACK_BOTTOM_ADDR - PAGE_SIZE as u64;

/// Begin of the heap. No text or data segment is allowed to clash with this.
pub const USER_HEAP_BEGIN: usize = 0x40000000;
