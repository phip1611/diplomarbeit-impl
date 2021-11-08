use crate::kobjects::{
    LocalEcObject,
    PdObject,
};
use crate::libhedron::mtd::Mtd;
use crate::service_ids::ServiceId;
use crate::util::global_counter::GlobalIncrementingCounter;
use alloc::rc::{
    Rc,
    Weak,
};
use core::cell::RefCell;
use core::cmp::Ordering;
use core::fmt::Debug;
use libhedron::capability::CapSel;
use libhedron::mem::PAGE_SIZE;
use libhedron::syscall::create_pt::create_pt;
use libhedron::syscall::pt_ctrl::pt_ctrl;
use libhedron::utcb::Utcb;

/// Type for a function, that is the entry from a function call.
/// This function should wrap a [`PtObjCallbackFn`].
pub type PtEntryFn = fn(PortalIdentifier) -> !;

/// A globally, unique value associated with [`crate::kobjects::PtObject`].
/// This gets passed as argument into the callback function of a portals can
/// be used inside to lookup portals. The identifiers shall be issued by
/// [`PORTAL_IDENTIFIER_COUNTER`].
pub type PortalIdentifier = u64;

/// Counter that issues globally unique [`PortalIdentifier`] values.
pub static PORTAL_IDENTIFIER_COUNTER: GlobalIncrementingCounter = GlobalIncrementingCounter::new();

/// Holds contextual information about a [`PtObject`]. This helps the callback
/// to better understand, what portal was called and why **if multiple portals are
/// multiplexed through the same callback entry function**.
#[derive(Debug)]
pub enum PtCtx {
    /// Portal is responsible for handling error exceptions. The payload contains the
    /// exception offset (Starting by 0). See also NUM_EXC and ExceptionEventOffset.
    Exception(u64),
    /// Portal call is a service call.
    Service(ServiceId),
}

impl PtCtx {
    /// Returns the err code.
    pub fn exc(&self) -> u64 {
        match self {
            PtCtx::Exception(err) => *err,
            _ => panic!("invalid variant"),
        }
    }
    /// Returns the service id.
    pub fn service_id(&self) -> ServiceId {
        match self {
            PtCtx::Service(id) => *id,
            _ => panic!("invalid variant"),
        }
    }

    pub fn is_exception_pt(&self) -> bool {
        match self {
            PtCtx::Exception(_) => true,
            PtCtx::Service(_) => false,
        }
    }

    pub fn is_service_pt(&self) -> bool {
        !self.is_exception_pt()
    }
}

/// Object that wraps around a kernel PT object with convenient runtime
/// data and methods. A PT may be created in a src PD only to be used in
/// a dest PD, for example as exception handler, that identifies the
/// source PD. In this case the `delegated_to_pd` fiels contains the target PD.
///
/// Relies on the layout defined in [`UserAppCapSpace`].
#[derive(Debug)]
pub struct PtObject {
    cap_sel: CapSel,
    local_ec: Weak<LocalEcObject>,
    portal_id: PortalIdentifier,
    mtd: Mtd,
    ctx: PtCtx,
    delegated_to_pd: RefCell<Option<Weak<PdObject>>>,
}

impl PtObject {
    /// Like [`Self::new`] but executes a `create_pt` syscall first.
    pub fn create(
        pt_sel: CapSel,
        local_ec: &Rc<LocalEcObject>,
        mtd: Mtd,
        portal_entry_fn: PtEntryFn,
        ctx: PtCtx,
    ) -> Rc<Self> {
        log::trace!("created PT with sel={}", pt_sel);
        create_pt(
            pt_sel,
            Self::pd_sel(local_ec),
            local_ec.ec_sel(),
            mtd,
            portal_entry_fn as *const u64,
        )
        .unwrap();
        let portal_id = PORTAL_IDENTIFIER_COUNTER.next();
        pt_ctrl(pt_sel, portal_id).unwrap();
        Self::new(pt_sel, local_ec, mtd, portal_id, ctx)
    }

    /// Only creates the object, assuming that the object is valid
    /// inside the capability space of the caller.
    ///
    /// Attaches itself to the corresponding [`LocalEcObject`] automatically
    /// and returns a copy of self.
    pub fn new(
        pt_sel: CapSel,
        local_ec: &Rc<LocalEcObject>,
        mtd: Mtd,
        portal_id: PortalIdentifier,
        ctx: PtCtx,
    ) -> Rc<Self> {
        let obj = Rc::new(Self {
            cap_sel: pt_sel,
            local_ec: Rc::downgrade(local_ec),
            portal_id,
            mtd,
            ctx,
            delegated_to_pd: RefCell::new(None),
        });
        local_ec.add_portal(obj.clone());
        obj
    }

    /// Returns the PD sel of the PD this PT belongs to.
    fn pd_sel(local_ec: &Rc<LocalEcObject>) -> CapSel {
        local_ec.pd().cap_sel()
    }

    /// Returns the cap selector of this PT.
    pub fn cap_sel(&self) -> CapSel {
        self.cap_sel
    }
    pub fn local_ec(&self) -> Rc<LocalEcObject> {
        self.local_ec.upgrade().unwrap()
    }
    pub fn portal_id(&self) -> PortalIdentifier {
        self.portal_id
    }
    pub fn mtd(&self) -> Mtd {
        self.mtd
    }

    /// Returns the top of the stack address from the corresponding local EC.
    pub fn stack_top(&self) -> u64 {
        self.local_ec.upgrade().unwrap().stack_top_ptr()
    }

    /// Returns a mutable reference to the corresponding Utcb.
    pub fn utcb_mut(&self) -> &mut Utcb {
        let utcb_addr = self.local_ec.upgrade().unwrap().utcb_page_num() * PAGE_SIZE as u64;
        unsafe { (utcb_addr as *mut Utcb).as_mut().unwrap() }
    }

    /// Returns an owned copy, which means possible locks around `&self` can be dropped
    /// while this is still in use.
    pub fn ctx(&self) -> &PtCtx {
        &self.ctx
    }

    /// Store the PD object where the PT was delegated to inside the PT.
    pub fn attach_delegated_to_pd(&self, delegated_to_pd: &Rc<PdObject>) {
        assert!(
            self.delegated_to_pd.borrow().is_none(),
            "can only delegate a portal once!"
        );
        self.delegated_to_pd
            .borrow_mut()
            .replace(Rc::downgrade(delegated_to_pd));
    }

    /// Get the value of `delegated_to_pd`.
    pub fn delegated_to_pd(&self) -> Option<Rc<PdObject>> {
        if let Some(x) = &*self.delegated_to_pd.borrow() {
            Some(x.upgrade().expect("must still be valid!"))
        } else {
            None
        }
    }
}

impl PartialOrd<Self> for PtObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cap_sel.partial_cmp(&other.cap_sel)
    }
}

impl PartialEq<Self> for PtObject {
    fn eq(&self, other: &Self) -> bool {
        self.cap_sel.eq(&other.cap_sel)
    }
}

impl Eq for PtObject {}

impl Ord for PtObject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cap_sel.cmp(&other.cap_sel())
    }
}

impl Drop for PtObject {
    fn drop(&mut self) {
        log::warn!("PtObject dropped: capability revoke not implemented yet");
    }
}
