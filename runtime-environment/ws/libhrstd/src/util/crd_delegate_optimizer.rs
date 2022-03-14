//! Module for [`OrderLoopOptimizerIterator`].

use crate::cap_space::root::RootCapSpace;
use crate::libhedron::mem::PAGE_SIZE;
use core::cmp::min;
use libhedron::syscall::{
    sys_pd_ctrl_delegate,
    DelegateFlags,
};
use libhedron::{
    CapSel,
    CrdMem,
    CrdObjPT,
    MemCapPermissions,
    PTCapPermissions,
};

/// An iterator that helps to delegate multiple capabilities via
/// [`crate::libhedron::Crd`] objects in a as optimal as it can be bulk operation.
/// Helps you to iterate over the optimal syscall parameters (regarding base and order)
/// to reduce total syscalls.
///
/// NOVA/HEDRON use a "order" (2^order) parameter for capabilities when one wants
/// to transfer/delegate multiple capabilities at once. This reduces the number
/// of required syscalls by a lot.
///
/// The problem is that not every amount of capabilities to transfer (for example mem pages)
/// is a power of two. In this case, it is required to use multiple iterations with
/// a descending order. This iterator helps with that, so that not every piece of code has
/// to implement the loop by itself.
///
/// **An additional complexity is, that for all CRD types, the base must be aligned
/// regarding the order.** Example: We want to map 16 pages, starting at page 15.
/// 1) base 15, order 1 ( 1/16 pages)
/// 2) base 16, order 3 ( 9/16 pages)
/// 3) base 25, order 2 (13/16 pages)
/// 4) base 29, order 1 (15/16 pages)
/// 4) base 31, order 0 (16/16 pages)
///
/// # Notes
/// For Multiboot-Modules the optimization is not applicable from my observations, because
/// they are only page-aligned but often not more. Therefore a delegate syscall for each page.
#[derive(Debug)]
pub struct CrdDelegateOptimizer {
    /// Describes the amount of items/capabilities.
    /// For example amount of pages or port capabilities.
    item_amount: u64,
    items_processed: u64,
    start_src_base: u64,
    start_dest_base: u64,
}

impl CrdDelegateOptimizer {
    const MAX_ORDER: u64 = u8::MAX as u64;

    /// Creates a new [`CrdDelegateOptimizer`]. The base is either a page-num,
    /// a I/O port number or an index into the capability space.
    #[must_use]
    pub const fn new(start_src_base: u64, start_dest_base: u64, item_amount: usize) -> Self {
        Self {
            item_amount: item_amount as u64,
            items_processed: 0,
            start_src_base,
            start_dest_base,
        }
    }

    /// Finds the highest order for a base (regarding power of 2), where the base
    /// is aligned to. 64 is the maximum value.
    fn find_highest_order_for_base_alignment(base: u64) -> u64 {
        for order in (1..=Self::MAX_ORDER).rev() {
            let power = libm::pow(2.0, order as f64) as u64;
            if base % power == 0 {
                return order;
            }
        }
        0
    }

    /// Iterates over all elements of [`Self`] and delegates memory capabilites
    /// from the src Pd to the dest Pd. If SRC_PD = DEST_PD and SRC_PD == ROOTTASK_PD,
    /// the Hypervisor-flag in the DelegateFlags gets true.
    pub fn mmap(self, src_pd: CapSel, dest_pd: CapSel, perm: MemCapPermissions) {
        let is_roottask = src_pd == RootCapSpace::RootPd.val();
        let is_roottask_to_roottask_mapping = is_roottask && src_pd == dest_pd;

        if is_roottask_to_roottask_mapping {
            log::debug!("is roottask to roottask mapping (hypervisorflag true)");
        }

        self.for_each(|params| {
            log::trace!(
                "map page {} ({:?}) (pd={}) to page {} ({:?}) (pd={}), order={} (2^order={}, perm={:?})",
                params.src_base,
                (params.src_base as usize * PAGE_SIZE) as *const u64,
                src_pd,
                params.dest_base,
                (params.dest_base as usize * PAGE_SIZE) as *const u64,
                dest_pd,
                params.order,
                params.power,
                perm,
            );

            // currently in Hedron: needs twice the same permissions (this will be removed soon)
            let src_crd = CrdMem::new(params.src_base, params.order, perm);
            let dest_crd = CrdMem::new(params.dest_base, params.order, perm);
            sys_pd_ctrl_delegate(
                src_pd,
                dest_pd,
                src_crd,
                dest_crd,
                DelegateFlags::new(true, false, false, is_roottask_to_roottask_mapping, 0),
            )
            .unwrap();
        });
    }

    /// Map PTs to other PTs.
    pub fn pts(self, src_pd: CapSel, dest_pd: CapSel) {
        self.for_each(|params| {
            log::trace!(
                "map PT sel {} (pd={}) to PT sel {} (pd={}), order={} (2^order={})",
                params.src_base,
                src_pd,
                params.dest_base,
                dest_pd,
                params.order,
                params.power
            );

            let perm = PTCapPermissions::CALL;

            // currently in Hedron: needs twice the same permissions (this will be removed soon)
            let src_crd = CrdObjPT::new(params.src_base, params.order, perm);
            let dest_crd = CrdObjPT::new(params.dest_base, params.order, perm);
            sys_pd_ctrl_delegate(src_pd, dest_pd, src_crd, dest_crd, DelegateFlags::default())
                .unwrap();
        });
    }
}

impl Iterator for CrdDelegateOptimizer {
    type Item = CrdStepParams;

    fn next(&mut self) -> Option<Self::Item> {
        let items_left = self.item_amount - self.items_processed;
        if items_left == 0 {
            return None;
        }

        let src_base = self.start_src_base + self.items_processed;
        let dest_base = self.start_dest_base + self.items_processed;

        // next lower 2^order, that fits the total items
        let order_items = libm::log2(items_left as f64) as u8;

        let max_order_src = Self::find_highest_order_for_base_alignment(src_base);
        let max_order_dest = Self::find_highest_order_for_base_alignment(dest_base);

        // we now search the minimum order we can work with. It is determined by the bases
        // as well as the amount of left items.
        let min_order_src_dest = min(max_order_src, max_order_dest);
        let order = min(min_order_src_dest, order_items as u64) as u8;

        // Count of items (i.e. capabilities) , that we map in this iteration step.
        let amount_of_items_this_iteration = libm::pow(2 as f64, order as f64) as u64;

        // subtract iteration condition
        let old_items_processed = self.items_processed;
        self.items_processed += amount_of_items_this_iteration;

        Some(CrdStepParams {
            order,
            power: libm::pow(2 as f64, order as f64) as u64,
            src_base,
            dest_base,
            items_processed: old_items_processed,
        })
    }
}

/// Iterator-item for [`CrdDelegateOptimizer`].
#[derive(Debug, Copy, Clone)]
pub struct CrdStepParams {
    /// The power of the current iteration. Order for src & dest CRD.
    pub order: u8,
    /// The number of items processed in this iteration step (2^power).
    pub power: u64,
    /// The base for the src CRD in this iteration step.
    pub src_base: u64,
    /// The base for the dest CRD in this iteration step.
    pub dest_base: u64,
    /// Total items processed. The sum of all `power` values, except the current one.
    pub items_processed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_loop_optimizer_basic() {
        let mut optimizer = CrdDelegateOptimizer::new(0, 0, 0);
        assert!(optimizer.next().is_none());

        let mut optimizer = CrdDelegateOptimizer::new(0, 0, 1);
        {
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 0);
            assert_eq!(item.power, 1);
            assert_eq!(item.items_processed, 0);
            /*assert_eq!(item.items_left, 0);
            assert_eq!(item.items_processed, 1);*/
            assert!(optimizer.next().is_none());
        }

        let mut optimizer = CrdDelegateOptimizer::new(0, 0, 9);
        {
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 3);
            assert_eq!(item.power, 8);
            assert_eq!(item.items_processed, 0);
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 0);
            assert_eq!(item.power, 1);
            assert_eq!(item.items_processed, 8);
            assert!(optimizer.next().is_none());
        }

        let mut optimizer = CrdDelegateOptimizer::new(0, 0, 23);
        {
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 4);
            assert_eq!(item.power, 16);
            assert_eq!(item.items_processed, 0);
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 2);
            assert_eq!(item.power, 4);
            assert_eq!(item.items_processed, 16);
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 1);
            assert_eq!(item.power, 2);
            assert_eq!(item.items_processed, 20);
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 0);
            assert_eq!(item.power, 1);
            assert_eq!(item.items_processed, 22);
            assert!(optimizer.next().is_none());
        }
    }

    #[test]
    fn test_find_highest_order_for_base_alignment() {
        let fnc = CrdDelegateOptimizer::find_highest_order_for_base_alignment;
        assert_eq!(fnc(0), CrdDelegateOptimizer::MAX_ORDER);
        assert_eq!(fnc(1), 0);
        assert_eq!(fnc(2), 1);
        assert_eq!(fnc(3), 0);
        assert_eq!(fnc(4), 2);
        assert_eq!(fnc(5), 0);
        assert_eq!(fnc(6), 1);
        assert_eq!(fnc(512), 9);
        assert_eq!(fnc(1024), 10);
    }

    #[test]
    fn test_order_loop_optimizer_complex() {
        // pretend we want to map 15 pages
        // from src-page 15 to dest-page 0.
        let optimizer = CrdDelegateOptimizer::new(16, 4, 32);
        let entries = optimizer.collect::<alloc::vec::Vec<_>>();
        dbg!(entries);
        /*{
            let item = optimizer.next().unwrap();
            assert_eq!(item.order, 0);
            assert_eq!(item.power, 1);
            assert_eq!(item.items_processed, 0);
        }*/
    }
}
