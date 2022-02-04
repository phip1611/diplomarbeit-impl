use crate::kobjects::PdObject;
use alloc::rc::{
    Rc,
    Weak,
};
use libhedron::syscall::SmCtrlZeroCounterStrategy;
use libhedron::CapSel;

/// A convenient wrapper around the Semaphore (SM) kernel object.
#[derive(Debug)]
pub struct SmObject {
    sel: CapSel,
    owning_pd: Weak<PdObject>,
}

impl SmObject {
    const DEFAULT_COUNTER: u64 = 0;

    /// Wrapper around [`SmObject::new`] that creates the object in the capability space
    /// of the kernel.
    pub fn create(sel: CapSel, owning_pd: &Rc<PdObject>) -> Rc<Self> {
        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = crate::libhedron::syscall::sys_create_sm;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_create_sm;

        syscall_fn(sel, owning_pd.cap_sel(), Self::DEFAULT_COUNTER).unwrap();

        Self::new(sel, owning_pd)
    }

    /// Creates a new SmObject but assumes it already exists inside the capability space
    /// of the kernel.
    pub fn new(sel: CapSel, owning_pd: &Rc<PdObject>) -> Rc<Self> {
        let sm = Rc::new(Self {
            sel,
            owning_pd: Rc::downgrade(owning_pd),
        });

        // TODO attach SM to PD Object

        sm
    }

    /// Performs a "semaphore up" operation.
    pub fn sem_up(&self) {
        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = crate::libhedron::syscall::sys_sm_up;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_sm_up;

        syscall_fn(self.sel).unwrap();
    }

    /// Performs a "semaphore down" operation.
    pub fn sem_down(&self) {
        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = crate::libhedron::syscall::sys_sm_down;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_sm_down;

        syscall_fn(self.sel, SmCtrlZeroCounterStrategy::Decrement, None).unwrap();
    }

    pub fn sel(&self) -> CapSel {
        self.sel
    }

    pub fn owning_pd(&self) -> &Weak<PdObject> {
        &self.owning_pd
    }
}

impl Drop for SmObject {
    fn drop(&mut self) {
        log::debug!("SMObject: drop not implemented yet. TODO!");
    }
}
