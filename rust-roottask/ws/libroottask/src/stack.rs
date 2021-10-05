//! Used as stack for the roottask. It is convenient to this in Rust
//! because it reduces distribution of responsibility/functionality across Rust code,
//! assembler code and the linker script.

use libhedron::capability::{
    CapSel,
    CrdMem,
    MemCapPermissions,
};
use libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};

/// Size of a page on x86_64.
const PAGE_SIZE: usize = 4096;

/// SSE feature requires 128 bit/16 byte stack alignment on x86_64.
/// In the spec I found instructions, such as movaps, that also want
/// 64-byte alignments for 512 bit registers. Therefore, I picked the
/// lowest, save alignment value, which is 64.
/// This value is save for all kinds of scenarios/used features.
const STACK_ALIGNMENT: usize = 64;

/// The stack of a typical function has a 8 byte return address and a
/// 8 byte frame pointer. After that, the first stack variable follows.
///
/// Therefore, one should optimize the stack in a way, that the very
/// first stack argument has a alignment of [`STACK_ALIGNMENT`] bytes.
/// Because a function pushes its framepointer itself, we are at a point
/// that the stack pointer has to be 8 bytes off the alignment.
///
/// Without this, instructions such as `movaps` fail.
const ALIGNMENT_LOAD_OFFSET: usize = 8;

/// Helper struct for [`StaticStack`].
#[derive(Copy, Clone, Debug)]
#[repr(align(4096), C)]
pub struct Page([u8; PAGE_SIZE]);

impl Page {
    /// Constructor.
    pub const fn new() -> Self {
        Self([0; PAGE_SIZE])
    }

    /// Returns the pointer to this page. It is the first byte of the page
    /// and page aligned.
    pub fn get_ptr(&self) -> *const u8 {
        let self_ptr = self as *const Page as *const u8;
        let data_ptr = self.0.as_ptr();

        // check if my assumptions work (and the compiler does what I think it does)
        debug_assert_eq!(self_ptr, data_ptr, "there is no padding allowed");
        debug_assert!(
            self_ptr as usize % PAGE_SIZE == 0,
            "page must be page-aligned"
        );

        data_ptr
    }

    /// Returns the number of this page (in the virtual address space),
    pub fn get_num(&self) -> usize {
        self.get_ptr() as usize / 4096
    }
}

/// A static stack object (assigned to a global static variable) helps us
/// to define the initial stack for the roottask from Rust. The symbol to
/// the stack begin itself can be exported and referenced by the assembly code.
/// The stack will be 128 byte aligned. An requirement for SSE instructions.
///
/// It contains space for a guard page below the stack. This is an easy and
/// pragmatic solution to have some kind of memory there, which can be marked as
/// not readable eventually (or be unmapped, depends on what works better).
///
/// We don't need linker magic or other utilities this way to guarantee, that
/// Rust or the linker don't place other things right below the stack.
///
/// This brings two benefits:
/// - I can relatively easy track stack memory usage in Rust
/// - there is no need for hacky linker script magic
#[derive(Debug)]
#[repr(align(4096), C)]
pub struct StaticStack<const PAGE_NUM: usize> {
    // C-layout: keep in mind: guard page lies below the stack; stack grows downwards
    /// Property itself is useless, but its address/page number can be used
    /// to tell the kernel to either unmap this page or to mark it as not readable.
    guard_page: Page,
    /// The stack itself.
    data: [Page; PAGE_NUM],
}

impl<const PAGE_NUM: usize> StaticStack<PAGE_NUM> {
    pub const fn new() -> Self {
        Self {
            guard_page: Page::new(),
            data: [Page::new(); PAGE_NUM],
        }
    }

    /// Returns the pointer (inclusive!) to the top of the stack. The pointer is PAGE-aligned.
    /// From there, the stack can grow downwards.
    /// We waste almost a full page here, because we have the following problem:
    ///
    /// 0x1000 Guard Page
    /// 0x2000 Page 1 begin
    /// 0x2fff Page 1 end
    /// 0x3000 Page 2 begin
    /// 0x3f80 is 128 byte aligned and highest address we can use as stack
    ///        (128 alignment required for SSE)
    ///        f80 comes from 0xfff - 127
    /// 0x3fff Page 2 end   --> stack top --> not page aligned --> bad
    /// 0x4000 out of bounds (in this example)
    ///        but 0x4000 - 128 is also the correct base address for the 128 byte aligned stack
    ///
    /// Therefore, given the above example, this function would return 0x3000 as stack top.
    pub const fn get_stack_top_ptr(&self) -> *const u8 {
        // references the byte right above the reserved space in "self.data";
        // therefore, the start byte of the next page (which is not there)
        // => good base to calculate actual stack top; see comments on method above
        let ptr = unsafe { self.data.as_ptr().add(PAGE_NUM) };
        let ptr = ptr as *const u8;
        unsafe { ptr.sub(STACK_ALIGNMENT).add(ALIGNMENT_LOAD_OFFSET) }
    }

    /// Returns the pointer (inclusive!) to the bottom of the stack.
    pub const fn get_stack_btm_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const u8
    }

    /// Returns a reference to the memory representing the guard page.
    pub const fn get_guard_page(&self) -> &Page {
        &self.guard_page
    }

    /// Marks the guard page as unmapped by performing a syscall
    /// and revoking the rights of this page.
    ///
    /// Must provide the capability selector for this protection domain.
    pub unsafe fn activate_guard_page(&self, pd_sel: CapSel) {
        // CRD for exactly one single page
        let crd = CrdMem::new(
            self.get_guard_page().get_num() as u64,
            0,
            MemCapPermissions::empty(),
        );
        pd_ctrl_delegate(
            pd_sel,
            pd_sel,
            crd,
            // ignored here I think
            crd,
            DelegateFlags::new(false, false, false, true, 0),
        )
        .unwrap();
    }

    /// TEST/Debug method.
    /// Performs a read and a write operation on the guard page.
    /// If this doesn't result in a general protection fault,
    /// the guard page is not active yet.
    #[allow(unused)]
    pub unsafe fn test_rw_guard_page(&self) -> u8 {
        let ptr = self.guard_page.get_ptr() as *mut u8;
        let val = *ptr;
        let mut ptr = ptr.add(1);
        *ptr = 0xff;
        val
    }

    /// Returns the size in bytes of the stack. It doesn't include the guard page below the stack.
    pub const fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libhrstd::sync::static_global_ptr::StaticGlobalPtr;

    static TEST_STACK: StaticStack<1> = StaticStack::new();

    #[test]
    fn test_stack() {
        let ptr = TEST_STACK.get_stack_top_ptr();
        println!("stack_top {:#?}", ptr);
        assert_eq!(
            (ptr as usize - ALIGNMENT_LOAD_OFFSET) % STACK_ALIGNMENT,
            0,
            "stack must be {} byte aligned",
            STACK_ALIGNMENT
        );

        // test compiles
        let _trusted_stack_ptr = StaticGlobalPtr::new(ptr);
    }
}
