//! Helper structures to manage capability selectors.
//! Different to UNIX, capability selectors are managed
//! by the userland app itself. The app needs to know what
//! selector can point to the next capability inside its
//! kernel capability space.

use crate::hedron::capability::CapSel;
use crate::hrstd::sync::mutex::SimpleMutex;
use alloc::collections::BTreeSet;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::ops::Deref;

/// This structure helps to manage capability
/// selectors inside userspace, requesting new
/// ones, i.e. new free numbers, and also
/// mark them as free gain.
///
/// Don't confuse this with [`Crd`]. A Crd is
/// there to request one or multiple capabilities
/// at once from the kernel. This structure is only
/// there to manage capability selectors inside a
/// userspace app under Hedron where capabilities
/// can be attached to.
///
/// This structure needs a dynamic allocator.
#[derive(Debug, Ord, Eq, Copy, Clone)]
pub struct CapSelRange {
    base: CapSel,
    offset: u8,
}

impl CapSelRange {
    /// Lowest valid capability selector.
    pub fn base(&self) -> CapSel {
        self.base
    }

    /// Highest possible capability selector.
    pub fn max_sel(&self) -> CapSel {
        self.base + (self.offset as u64 - 1)
    }
}

// required for .get() on a HashSet
impl Borrow<CapSel> for CapSelRange {
    fn borrow(&self) -> &CapSel {
        &self.base
    }
}

impl PartialEq for CapSelRange {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
    }
}

impl PartialOrd for CapSelRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.base < other.base {
            Ordering::Less
        } else if self.base == other.base {
            Ordering::Equal
        } else {
            Ordering::Greater
        })
    }
}

#[derive(Debug)]
pub struct CapSelRangeGuard<'a> {
    // copy is cheap, we hold a copy
    range: CapSelRange,
    manager: &'a CapSelManager,
}

impl<'a> CapSelRangeGuard<'a> {
    fn new(range: CapSelRange, manager: &'a CapSelManager) -> Self {
        Self { range, manager }
    }
}

impl<'a> Drop for CapSelRangeGuard<'a> {
    fn drop(&mut self) {
        let mut lock = self.manager.ranges.lock();
        let map = lock.as_mut().unwrap();
        map.remove(&self.range.base);
    }
}

impl<'a> Deref for CapSelRangeGuard<'a> {
    type Target = CapSelRange;

    fn deref(&self) -> &Self::Target {
        &self.range
    }
}

#[derive(Debug)]
pub struct CapSelManager {
    /// The minimum value for `base` in
    /// `CapSelRange`.
    start_val: SimpleMutex<Option<CapSel>>,
    ranges: SimpleMutex<Option<BTreeSet<CapSelRange>>>,
}

impl CapSelManager {
    pub const fn new() -> Self {
        Self {
            start_val: SimpleMutex::new(None),
            ranges: SimpleMutex::new(None),
        }
    }

    pub fn init(&self, val: CapSel) {
        let mut start_val = self.start_val.lock();
        if start_val.is_some() {
            panic!("already initialized!");
        }
        start_val.replace(val);

        let mut ranges = self.ranges.lock();
        ranges.replace(BTreeSet::new());
    }

    pub fn request(&self, count: u8) -> CapSelRangeGuard {
        assert!(count > 0, "count starts at one for one capability selector");
        let base = self.find_base(count);
        let new_range = CapSelRange {
            base,
            offset: count,
        };
        let mut lock = self.ranges.lock();
        let map = lock.as_mut().unwrap();
        map.insert(new_range.clone());
        CapSelRangeGuard::new(new_range, self)
    }

    /// Finds the next free base where we can occupy
    /// `offset`-capability selectors from.
    fn find_base(&self, offset: u8) -> CapSel {
        let lock = self.start_val.lock();
        let min_val = lock.unwrap();

        let mut new_base = min_val;

        // iterate until we found a new base; increment new base at each run
        'outer: loop {
            // we check if the new base is
            for i in (new_base)..(new_base + offset as CapSel) {
                if self.cap_sel_in_occopied_ranges(i) {
                    // not efficient but works
                    new_base += 1;
                    continue 'outer;
                }
            }
            break;
        }

        new_base
    }

    fn cap_sel_in_occopied_ranges(&self, target: CapSel) -> bool {
        let lock = self.ranges.lock();
        let map = lock.as_ref().unwrap();
        for x in map {
            let lower = x.base;
            let upper = lower + x.offset as CapSel;
            if target >= lower && target < upper {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::hrstd::capability::CapSelManager;

    #[test]
    fn test_cap_sel_range() {
        static CAP_SEL_MNGR: CapSelManager = CapSelManager::new();
        CAP_SEL_MNGR.init(32);
        let cap_r_1 = CAP_SEL_MNGR.request(1);
        let cap_r_2 = CAP_SEL_MNGR.request(4);
        let cap_r_3 = CAP_SEL_MNGR.request(2);
        let cap_r_4 = CAP_SEL_MNGR.request(3);
        assert_eq!(32, cap_r_1.base());
        assert_eq!(32, cap_r_1.max_sel());

        assert_eq!(33, cap_r_2.base());
        assert_eq!(36, cap_r_2.max_sel());

        assert_eq!(37, cap_r_3.base());
        assert_eq!(38, cap_r_3.max_sel());

        assert_eq!(39, cap_r_4.base());
        assert_eq!(41, cap_r_4.max_sel());

        {
            let cap_sel_range = CAP_SEL_MNGR.request(3);
            assert_eq!(42, cap_sel_range.base());
            assert_eq!(44, cap_sel_range.max_sel());
        }

        // other cap sel range already dropped
        {
            let cap_sel_range = CAP_SEL_MNGR.request(2);
            assert_eq!(42, cap_sel_range.base());
            assert_eq!(43, cap_sel_range.max_sel());
        }
    }
}
