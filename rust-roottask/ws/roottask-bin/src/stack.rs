//! Initial stack for the roottask. References in `assembly.S`.

use libhrstd::hip::HIP;
use libroottask::stack::{
    StaticStack,
    TrustedStackPtr,
};

/// Marks the guard-page of the corresponding [`StaticStack`] as not
/// read- and writeable, i.e. not present. Performs a syscall for that.
pub fn init(hip: &HIP) {
    unsafe { ROOTTASK_STACK.activate_guard_page(hip.root_pd()) }
    log::debug!("guard page for root task stack is active!");
}

// 32 pages equals 128 Kibibyte
const STACK_SIZE_128KIB: usize = 32;

/// Put into the load section of the ELF. Marked as read and write.
#[no_mangle]
#[used]
pub static mut ROOTTASK_STACK: StaticStack<STACK_SIZE_128KIB> = StaticStack::new();

/// Referenced by assembly.S.
#[no_mangle]
#[used]
pub static ROOTTASK_STACK_TOP_PTR: TrustedStackPtr =
    TrustedStackPtr::new(unsafe { ROOTTASK_STACK.get_stack_top_ptr() });
