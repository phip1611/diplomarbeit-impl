//! Initial stack for the roottask. References in `assembly.S`.

use libhrstd::libhedron::hip::HIP;
use libhrstd::sync::static_global_ptr::StaticGlobalPtr;
use libroottask::stack::StaticStack;

/// Marks the guard-page of the corresponding [`StaticStack`] as not
/// read- and writeable, i.e. not present. Performs a syscall for that.
pub fn init(hip: &HIP) {
    unsafe { ROOTTASK_STACK.activate_guard_page(hip.root_pd()) }
    log::debug!(
        "guard page for root task stack is active! Stackoverflow will result in PF exception now."
    );
}

// The stack of the roottask is 32 pages in size, which equals 128 Kibibyte.
pub const STACK_SIZE_128KIB: usize = 0x20000;
const PAGE_SIZE: usize = 4096;
const STACK_SIZE_PAGES: usize = STACK_SIZE_128KIB / PAGE_SIZE;

/// Pointer to the stack top of the stack of the Roottask (inclusive!).
pub static STACK_TOP_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(ROOTTASK_STACK.get_stack_top_ptr()) };
/// Pointer to the stack bottom of the stack of the Roottask (inclusive!).
pub static STACK_BOTTOM_PTR: StaticGlobalPtr<u8> =
    unsafe { StaticGlobalPtr::new(ROOTTASK_STACK.get_stack_btm_ptr()) };

/// Put into the load section of the ELF. Marked as read and write.
#[no_mangle]
#[used]
static mut ROOTTASK_STACK: StaticStack<STACK_SIZE_PAGES> = StaticStack::new();

/// Referenced by assembly.S.
#[no_mangle]
#[used]
static ROOTTASK_STACK_TOP_PTR: StaticGlobalPtr<u8> =
    StaticGlobalPtr::new(unsafe { ROOTTASK_STACK.get_stack_top_ptr() });
