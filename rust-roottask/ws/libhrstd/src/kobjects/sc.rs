use crate::cap_space::user::UserAppCapSpace;
use crate::kobjects::GlobalEcObject;
use crate::libhedron::capability::CrdObjSC;
use crate::libhedron::ipc_serde::__private::Formatter;
use crate::libhedron::syscall::pd_ctrl::DelegateFlags;
use alloc::rc::{
    Rc,
    Weak,
};
use core::fmt::Debug;
use libhedron::capability::{
    CapSel,
    SCCapPermissions,
};
use libhedron::qpd::Qpd;

/// Object that wraps around a global EC kernel object with convenient runtime
/// data and methods.
///
/// Relies on the layout defined in [`UserAppCapSpace`].
pub struct ScObject {
    cap_sel: CapSel,
    gl_ec: Weak<GlobalEcObject>,
    qpd: Option<Qpd>,
}

impl ScObject {
    /// Like [`Self::new`] but executes a `create_sc` syscall first.
    /// Furthermore, it delegates the capability to the new PD into the new PD.
    pub fn create(cap_sel: CapSel, gl_ec: &Rc<GlobalEcObject>, qpd: Qpd) -> Rc<Self> {
        // the PD where this SC was created
        let parent_pd_sel = gl_ec.pd().parent().expect("must have a parent").cap_sel();
        // the PD where the global EC exists in, that this SC belongs to
        let target_pd_sel = gl_ec.pd().cap_sel();

        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = libhedron::syscall::create_sc::sys_create_sc;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_create_sc;
        syscall_fn(cap_sel, parent_pd_sel, gl_ec.ec_sel(), qpd).unwrap();

        #[cfg(not(feature = "foreign_rust_rt"))]
        let syscall_fn = libhedron::syscall::pd_ctrl::sys_pd_ctrl_delegate;
        #[cfg(feature = "foreign_rust_rt")]
        let syscall_fn = crate::rt::hybrid_rt::syscalls::sys_hybrid_pd_ctrl_delegate;
        // install SC cap in new PD at well-known place
        syscall_fn(
            parent_pd_sel,
            target_pd_sel,
            CrdObjSC::new(cap_sel, 0, SCCapPermissions::empty()),
            CrdObjSC::new(UserAppCapSpace::Sc.val(), 0, SCCapPermissions::empty()),
            DelegateFlags::new(false, false, false, false, 0),
        )
        .unwrap();
        Self::new(cap_sel, gl_ec, Some(qpd))
    }

    /// Only creates the object, assuming that the object is valid inside
    /// the capability space of the caller.
    pub fn new(cap_sel: CapSel, gl_ec: &Rc<GlobalEcObject>, qpd: Option<Qpd>) -> Rc<Self> {
        let obj = Rc::new(Self {
            cap_sel,
            gl_ec: Rc::downgrade(gl_ec),
            qpd,
        });
        gl_ec.attach_sc(obj.clone());
        obj
    }

    pub fn cap_sel(&self) -> CapSel {
        self.cap_sel
    }

    /// Returns the owning [`GlobalEcObject`].
    pub fn gl_ec(&self) -> Rc<GlobalEcObject> {
        self.gl_ec.upgrade().unwrap()
    }
    pub fn qpd(&self) -> Option<Qpd> {
        self.qpd
    }
}

impl Debug for ScObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScObject")
            .field("cap_sel", &self.cap_sel)
            .field("qpd", &self.qpd)
            .field("ec_sel", &self.gl_ec().ec_sel())
            .field("pd_sel", &self.gl_ec().pd().cap_sel())
            .finish()
    }
}

impl Drop for ScObject {
    fn drop(&mut self) {
        log::warn!("ScObject dropped: capability revoke not implemented yet");
        // todo detach from PDobject
    }
}
