use core::alloc::Layout;
use libhrstd::sync::mutex::SimpleMutex;

/// Address bound, from that all virtual memory addresses are guaranteed to be unused,
/// except [`VirtMemAllocator`] hands them out.
const VIRT_FREE_ADDR_BEGIN: VirtAddr = 0x40000000;

pub type VirtAddr = u64;

pub static VIRT_MEM_ALLOC: SimpleMutex<VirtMemAllocator> =
    SimpleMutex::new(VirtMemAllocator::new(VIRT_FREE_ADDR_BEGIN));

/// Allocates virtual memory addresses. Doesn't affect the heap, memory capabilities,
/// or the page table. Only hands out addresses, which can be used for further steps.
///
/// Currently: fast and pragmatic solution (no dealloc/free)
#[derive(Debug)]
pub struct VirtMemAllocator {
    next_available_addr: VirtAddr,
}

impl VirtMemAllocator {
    const fn new(begin_addr: VirtAddr) -> Self {
        Self {
            next_available_addr: begin_addr,
        }
    }

    /// Returns the next free/available virtual address.
    pub fn next_addr(&mut self, layout: Layout) -> VirtAddr {
        let align = layout.align() as u64;
        let addr = if self.next_available_addr % align == 0 {
            self.next_available_addr
        } else {
            self.next_available_addr + align - self.next_available_addr % align as u64
        };
        assert_eq!(addr % layout.align() as u64, 0, "must be aligned");
        self.next_available_addr = addr + layout.size() as u64;
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;
    use libhrstd::libhedron::mem::PAGE_SIZE;

    #[test]
    fn test_virt_mem_alloc() {
        let first = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());
        assert_eq!(first, VIRT_FREE_ADDR_BEGIN);

        let second = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap());
        assert_eq!(
            second,
            VIRT_FREE_ADDR_BEGIN + PAGE_SIZE as u64,
            "{:016x} != {:016x}",
            first,
            VIRT_FREE_ADDR_BEGIN + PAGE_SIZE as u64,
        );

        let one_mib = 0x100000;
        let third = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(PAGE_SIZE, one_mib).unwrap());
        assert_eq!(third, VIRT_FREE_ADDR_BEGIN + one_mib as u64);
    }
}
