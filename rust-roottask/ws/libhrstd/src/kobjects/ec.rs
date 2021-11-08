use crate::cap_space::user::UserAppCapSpace;
use crate::kobjects::{
    PdObject,
    PtObject,
    ScObject,
};
use crate::libhedron::capability::{
    CapSel,
    CrdObjEC,
};
use crate::libhedron::syscall::create_ec::{
    create_global_ec,
    create_local_ec,
};
use crate::libhedron::syscall::pd_ctrl::{
    pd_ctrl_delegate,
    DelegateFlags,
};
use crate::libhedron::utcb::Utcb;
use crate::util::global_counter::GlobalIncrementingCounter;
use alloc::collections::BTreeSet;
use alloc::rc::{
    Rc,
    Weak,
};
use core::cell::{
    Ref,
    RefCell,
    RefMut,
};
use core::cmp::Ordering;
use libhedron::capability::ECCapPermissions;
use libhedron::mem::PAGE_SIZE;

pub static EC_IDENTIFIER_COUNTER: GlobalIncrementingCounter = GlobalIncrementingCounter::new();

/// Object that wraps around a local EC kernel object with convenient runtime
/// data and methods.
///
/// Relies on the layout defined in [`UserAppCapSpace`].
#[derive(Debug)]
pub struct LocalEcObject {
    /// Unique ID for local EC objects.
    id: u64,
    pd: Weak<PdObject>,
    // CapSel to the EC inside the cap space of the executing PD.
    ec_sel: CapSel,
    stack_top_ptr: u64,
    utcb_addr: u64,
    // a local EC owns all its portals
    portals: RefCell<BTreeSet<Rc<PtObject>>>,
}

impl LocalEcObject {
    /// Like [`Self::new`] but with a `create_local_ec` syscall.
    pub fn create(
        ec_sel: CapSel,
        pd_obj: &Rc<PdObject>,
        stack_top_ptr: u64,
        utcb_addr: u64,
    ) -> Rc<Self> {
        let obj = Self::new(ec_sel, pd_obj, stack_top_ptr, utcb_addr);
        create_local_ec(
            ec_sel,
            pd_obj.cap_sel(),
            stack_top_ptr,
            // 0 is used as event base in all PDs by convention
            0,
            0,
            obj.utcb_page_num(),
        )
        .unwrap();
        obj
    }

    /// Creates a new object without a syscall. Assumes that
    /// the object already lives in the cap space of the calling PD.
    /// Attaches itself to the corresponding [`PdObject`] automatically and
    /// returns a copy of self.
    pub fn new(
        ec_sel: CapSel,
        pd_obj: &Rc<PdObject>,
        stack_top_ptr: u64,
        utcb_addr: u64,
    ) -> Rc<Self> {
        assert!(utcb_addr > 0);
        assert_eq!(utcb_addr % PAGE_SIZE as u64, 0);
        assert!(stack_top_ptr > 0);
        let obj = Self {
            pd: Rc::downgrade(&pd_obj),
            id: EC_IDENTIFIER_COUNTER.next(),
            ec_sel,
            stack_top_ptr,
            utcb_addr,
            portals: RefCell::new(BTreeSet::new()),
        };
        let obj = Rc::new(obj);
        pd_obj.attach_local_ec(obj.clone());
        obj
    }

    pub fn pd(&self) -> Rc<PdObject> {
        self.pd.upgrade().unwrap()
    }
    pub fn ec_sel(&self) -> CapSel {
        self.ec_sel
    }
    pub fn stack_top_ptr(&self) -> u64 {
        self.stack_top_ptr
    }
    pub fn utcb_addr(&self) -> u64 {
        self.utcb_addr
    }
    pub fn utcb(&self) -> &Utcb {
        unsafe { (self.utcb_addr as *const Utcb).as_ref().unwrap() }
    }
    pub fn utcb_mut(&self) -> &mut Utcb {
        unsafe { (self.utcb_addr as *mut Utcb).as_mut().unwrap() }
    }
    pub fn utcb_page_num(&self) -> u64 {
        self.utcb_addr / PAGE_SIZE as u64
    }

    pub fn add_portal(&self, pt: Rc<PtObject>) {
        let _ = self.portals.borrow_mut().insert(pt);
    }

    pub fn portals(&self) -> Ref<'_, BTreeSet<Rc<PtObject>>> {
        self.portals.borrow()
    }

    pub fn portals_mut(&self) -> RefMut<'_, BTreeSet<Rc<PtObject>>> {
        self.portals.borrow_mut()
    }

    /// Returns the unique ID of this local EC.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl PartialOrd<Self> for LocalEcObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq<Self> for LocalEcObject {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for LocalEcObject {}

impl Ord for LocalEcObject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Drop for LocalEcObject {
    fn drop(&mut self) {
        // todo detach from PDobject
        log::warn!("LocalEcObject dropped: capability revoke not implemented yet");
    }
}

/// Object that wraps around a global EC kernel object with convenient runtime
/// data and methods.
///
/// Relies on the layout defined in [`UserAppCapSpace`].
#[derive(Debug)]
pub struct GlobalEcObject {
    pd: Weak<PdObject>,
    sc: RefCell<Option<Rc<ScObject>>>,
    // CapSel to the EC inside the cap space of the executing PD.
    ec_sel: CapSel,
    // the top of the stack ptr for this global EC
    stack_top_ptr: u64,
    utcb_addr: u64,
}

impl GlobalEcObject {
    /// Like [`Self::new`] but with a `create_global_ec` syscall.
    /// Delegates the capability to the new EC into the target PD.
    pub fn create(
        ec_sel: CapSel,
        pd_obj: &Rc<PdObject>,
        utcb_addr: u64,
        stack_top_ptr: u64,
    ) -> Rc<Self> {
        let obj = Self::new(ec_sel, pd_obj, utcb_addr, stack_top_ptr);
        create_global_ec(
            ec_sel,
            pd_obj.cap_sel(),
            // 0 is used as event base in all PDs by convention
            0,
            0,
            obj.utcb_page_num(),
        )
        .unwrap();
        log::debug!("WAH");
        dbg!(pd_obj.parent().unwrap().cap_sel());
        dbg!(pd_obj.cap_sel());
        dbg!(ec_sel);
        pd_ctrl_delegate(
            pd_obj.parent().unwrap().cap_sel(),
            pd_obj.cap_sel(),
            CrdObjEC::new(ec_sel, 0, ECCapPermissions::empty()),
            CrdObjEC::new(UserAppCapSpace::Ec.val(), 0, ECCapPermissions::empty()),
            DelegateFlags::default(),
        )
        .unwrap();
        obj
    }

    /// Creates a new object without a syscall. Assumes that
    /// the object already lives in the cap space of the calling PD.
    /// Attaches itself to the corresponding [`PdObject`] automatically and
    /// returns a copy of self.
    pub fn new(
        ec_sel: CapSel,
        pd_obj: &Rc<PdObject>,
        utcb_addr: u64,
        stack_top_ptr: u64,
    ) -> Rc<Self> {
        assert!(utcb_addr > 0);
        assert_eq!(utcb_addr % PAGE_SIZE as u64, 0);
        let obj = Self {
            pd: Rc::downgrade(&pd_obj),
            ec_sel,
            utcb_addr,
            sc: RefCell::new(None),
            stack_top_ptr,
        };
        let obj = Rc::new(obj);
        pd_obj.attach_global_ec(obj.clone());
        obj
    }

    /// Returns the owning [`PdObject`].
    pub fn pd(&self) -> Rc<PdObject> {
        self.pd.upgrade().unwrap()
    }
    pub fn ec_sel(&self) -> CapSel {
        self.ec_sel
    }
    pub fn utcb_addr(&self) -> u64 {
        self.utcb_addr
    }
    pub fn utcb_page_num(&self) -> u64 {
        self.utcb_addr / PAGE_SIZE as u64
    }

    /// Returns a reference to the owned scheduling context, if (already) present.
    pub fn sc(&self) -> Ref<'_, Option<Rc<ScObject>>> {
        self.sc.borrow()
    }

    /// Returns a mutable reference to the owned scheduling context, if (already) present.
    pub fn sc_mut(&self) -> RefMut<'_, Option<Rc<ScObject>>> {
        self.sc.borrow_mut()
    }

    /// Attaches a SC to this global EC.
    pub fn attach_sc(&self, sc: Rc<ScObject>) {
        assert!(self.sc.borrow().is_none(), "already has SC!");
        self.sc.borrow_mut().replace(sc);
    }

    /// Returns the initial value of `rsp`.
    pub fn stack_top_ptr(&self) -> u64 {
        self.stack_top_ptr
    }
}

impl Drop for GlobalEcObject {
    fn drop(&mut self) {
        // todo detach from PDobject
        log::warn!("GlobalEcObject dropped: capability revoke not implemented yet");
    }
}
