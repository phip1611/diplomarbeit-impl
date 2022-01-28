use crate::cap_space::user::UserAppCapSpace;
use crate::kobjects::{
    GlobalEcObject,
    LocalEcObject,
    PortalIdentifier,
    PtObject,
};
use crate::libhedron::syscall::DelegateFlags;
use crate::libhedron::{
    CrdObjPD,
    PDCapPermissions,
};
use crate::process::consts::{
    ProcessId,
    ROOTTASK_PROCESS_PID,
};
use alloc::collections::BTreeSet;
use alloc::rc::{
    Rc,
    Weak,
};
use alloc::vec::Vec;
use core::cell::{
    Ref,
    RefCell,
    RefMut,
};
use libhedron::{
    CapSel,
    PTCapPermissions,
};

/// Object that wraps around a kernel PD object with convenient runtime
/// data and methods. This is the base for all user processes.
///
/// Relies on the layout defined in [`UserAppCapSpace`].
#[derive(Debug)]
pub struct PdObject {
    /// The ID of the process, that the PD belongs to (1:1 mapping).
    pid: ProcessId,
    // The capability selector to the parent PD object inside the PD where this object lives in
    // (either the creator PD or the PD where this was delegated to).
    parent: Option<Weak<Self>>,
    // The capability selector of this object inside the PD where this object lives in
    // (either the creator PD or the PD where this was delegated to).
    cap_sel: CapSel,
    // A PD can have one local EC
    // (pragmatic shortcut in my work; later a vec or so)
    local_ecs: RefCell<BTreeSet<Rc<LocalEcObject>>>,
    // A PD can have one global EC
    // (pragmatic shortcut in my work; later a vec or so)
    global_ec: RefCell<Option<Rc<GlobalEcObject>>>,
    // All portals that were delegated to this portal, for example exception portals.
    // I think it's correct to use Rc here. Weak doesn't work (not `Ord`) and as long as
    // the Rc is not cyclic, everything is fine.
    delegated_pts: RefCell<BTreeSet<Rc<PtObject>>>,
}

impl PdObject {
    /// Like [`Self::new`] but executes a `create_pd` syscall first.
    /// Furthermore, it delegates the capability to the new PD into the new PD.
    ///
    /// # Parameters
    /// * `pid` [`ProcessId`] that this PD belongs to
    /// * `parent` Parent PD
    /// * `cap_sel` Capability selector in the Cap Space of the owning PD
    /// * `foreign_syscall_base` Each CPU has a dedicated PT that handles syscalls. Base + CPU
    ///                          equals the capability selector of the PT.
    pub fn create(
        pid: ProcessId,
        parent: &Rc<Self>,
        cap_sel: CapSel,
        foreign_syscall_base: Option<CapSel>,
    ) -> Rc<Self> {
        log::trace!(
            "Creating PD: pid={}, cap_sel={}, parent_pd_sel={}, foreign_syscall_base={:?}",
            pid,
            cap_sel,
            parent.cap_sel,
            foreign_syscall_base,
        );

        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = libhedron::syscall::sys_create_pd;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_create_pd;
        syscall_fn(false, cap_sel, parent.cap_sel, foreign_syscall_base).unwrap();

        log::trace!(
            "Delegating new PD from PD={} to PD={} at index {}",
            parent.cap_sel,
            cap_sel,
            UserAppCapSpace::Pd.val()
        );
        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = libhedron::syscall::sys_pd_ctrl_delegate;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_pd_ctrl_delegate;
        let perms = PDCapPermissions::CREATE_EC
            | PDCapPermissions::CREATE_PD
            | PDCapPermissions::CREATE_PT
            | PDCapPermissions::CREATE_SC;
        syscall_fn(
            parent.cap_sel,
            cap_sel,
            CrdObjPD::new(cap_sel, 0, perms),
            CrdObjPD::new(UserAppCapSpace::Pd.val(), 0, perms),
            DelegateFlags::new(false, false, false, false, 0),
        )
        .unwrap();
        Self::new(pid, Some(parent), cap_sel)
    }

    /// Only creates the object, assuming that the object is valid inside
    /// the capability space of the caller.
    pub fn new(pid: ProcessId, parent: Option<&Rc<Self>>, cap_sel: CapSel) -> Rc<Self> {
        Rc::new(Self {
            pid,
            parent: parent.map(|x| Rc::downgrade(x)),
            cap_sel,
            local_ecs: RefCell::new(BTreeSet::new()),
            global_ec: RefCell::new(None),
            delegated_pts: RefCell::new(BTreeSet::new()),
        })
    }

    /// Like [`Self::new`] but with well-known default parameters.
    /// Can be called in user processes to get a correct initial "self"
    /// [`PdObject`] without invoking any syscalls.
    pub fn self_in_user_cap_space(pid: CapSel) -> Rc<Self> {
        Self::new(pid, None, UserAppCapSpace::Pd.val())
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }
    pub fn parent(&self) -> Option<Rc<Self>> {
        self.parent.as_ref().map(|x| x.upgrade()).flatten()
    }
    pub fn cap_sel(&self) -> CapSel {
        self.cap_sel
    }

    pub fn local_ecs(&self) -> Ref<'_, BTreeSet<Rc<LocalEcObject>>> {
        self.local_ecs.borrow()
    }

    pub fn global_ec(&self) -> Ref<'_, Option<Rc<GlobalEcObject>>> {
        self.global_ec.borrow()
    }

    pub fn local_ecs_mut(&self) -> RefMut<'_, BTreeSet<Rc<LocalEcObject>>> {
        self.local_ecs.borrow_mut()
    }

    pub fn global_ec_mut(&self) -> RefMut<'_, Option<Rc<GlobalEcObject>>> {
        self.global_ec.borrow_mut()
    }

    /// Adds a [`LocalEcObject`] to the PD.
    pub fn attach_local_ec(&self, local_ec: Rc<LocalEcObject>) {
        self.local_ecs.borrow_mut().insert(local_ec);
    }

    /// Adds a [`GlobalEcObject`] to the PD.
    pub fn attach_global_ec(&self, global_ec: Rc<GlobalEcObject>) {
        assert!(
            self.global_ec.borrow().is_none(),
            "has already global ec obj"
        );
        self.global_ec.borrow_mut().replace(global_ec);
    }

    /// Returns all delegated PTs of this PD.
    pub fn delegated_pts(&self) -> Ref<BTreeSet<Rc<PtObject>>> {
        self.delegated_pts.borrow()
    }

    /// Attaches a delegated PT to this PD.
    pub fn attach_delegated_pt(&self, pt: Rc<PtObject>) {
        self.delegated_pts.borrow_mut().insert(pt);
    }

    /// Iterator over all portals from the PD.
    pub fn portals(&self) -> Vec<Rc<PtObject>> {
        let local_ecs = self.local_ecs.borrow();
        local_ecs
            .iter()
            .map(|x| x.clone())
            // Q&D
            .flat_map(|x| x.portals().iter().map(|x| x.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    }

    /// Lookup for a portal by its unique ID across all local ECs of
    /// the given portal.
    pub fn lookup_portal(&self, pid: PortalIdentifier) -> Option<Rc<PtObject>> {
        for ec in self.local_ecs.borrow().iter() {
            for pt in ec.portals().iter() {
                if pt.portal_id() == pid {
                    return Some(pt.clone());
                }
            }
        }
        None
    }
}

impl Drop for PdObject {
    fn drop(&mut self) {
        if self.pid == ROOTTASK_PROCESS_PID {
            log::warn!("trying to drop the roottask PD - is this intended?!");
        }
        log::warn!("PDObject dropped: capability revoke not implemented yet");
    }
}
