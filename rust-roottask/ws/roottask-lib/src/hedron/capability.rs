use core::mem::transmute;

/// Generic capability selector. Similar to a file
/// descriptor in UNIX.
pub type CapSel = u64;

pub type CrdMem = Crd<CrdTypeMem, ()>;
pub type CrdPortIO = Crd<CrdTypePortIO, ()>;
pub type CrdObjEC = Crd<CrdTypeMem, CrdTypeObjectEC>;
pub type CrdObjSC = Crd<CrdTypeMem, CrdTypeObjectSC>;
pub type CrdObjSM = Crd<CrdTypeMem, CrdTypeObjectSM>;
pub type CrdObjPD = Crd<CrdTypeMem, CrdTypeObjectPD>;
pub type CrdObjPT = Crd<CrdTypeMem, CrdTypeObjectPT>;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CrdKind {
    NullCrd = 0,
    MemoryCrd = 1,
    PortIoCrd = 2,
    ObjectCrd = 3,
}

/// Abstraction over different CRD versions.
/// Don't use directly. The size of this type is 8 byte.
#[derive(Debug, Copy, Clone)]
pub struct Crd<Specialization, ObjectSpecialization> {
    val: u64,
    // zero size type; gone after compilation
    _zst1: Specialization,
    _zst2: ObjectSpecialization,
}

/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeMem;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypePortIO;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObject;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObjectPT;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObjectPD;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObjectSM;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObjectSC;
/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeObjectEC;

// generic/common/shared implementation accross all Crds
impl<Specialization, ObjectSpecialization> Crd<Specialization, ObjectSpecialization> {
    const KIND_BITMASK: u64 = 0b11;
    const BASE_BITMASK: u64 = 0b111100;
    const BASE_LEFT_SHIFT: u64 = 12;
    const ORDER_BITMASK: u64 = 0b111_1000_0000;
    const ORDER_LEFT_SHIFT: u64 = 7;
    const PERMISSIONS_BITMASK: u64 = 0b111_1000_0000;
    const PERMISSIONS_LEFT_SHIFT: u64 = 2;

    pub fn val(self) -> u64 {
        self.val
    }
    pub fn kind(self) -> CrdKind {
        unsafe {
            // CrdKind is represented as u8, therefore valid
            transmute((self.val & 0b11) as u8)
        }
    }
    pub fn order(self) -> u8 {
        ((self.val & Self::ORDER_BITMASK) >> 7) as u8
    }
    pub fn base(self) -> u16 {
        ((self.val & Self::BASE_BITMASK) >> 12) as u16
    }
    fn gen_permissions(self) -> u8 {
        ((self.val & Self::PERMISSIONS_BITMASK) >> 2) as u8
    }
}

impl CrdMem {
    pub fn permissions(self) -> MemCapPermissions {
        MemCapPermissions(self.gen_permissions())
    }
}

impl CrdPortIO {
    pub fn new(port: u16, order: u16) -> Self {
        let mut base = 0_u64;
        base |= CrdKind::PortIoCrd as u64 & Self::KIND_BITMASK;
        base |= (PortIOCapPermissions::new(true).val() as u64) << Self::PERMISSIONS_LEFT_SHIFT;
        base |= (port as u64) << Self::BASE_LEFT_SHIFT;
        base |= (order as u64) << Self::ORDER_LEFT_SHIFT;
        Self {
            val: base,
            // phantom data, not needed
            _zst1: CrdTypePortIO,
            _zst2: (),
        }
    }

    pub fn permissions(self) -> PortIOCapPermissions {
        PortIOCapPermissions(self.gen_permissions())
    }
}

impl CrdObjPD {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> PDCapPermissions {
        PDCapPermissions(self.gen_permissions())
    }
}

impl CrdObjSM {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> SMCapPermissions {
        SMCapPermissions(self.gen_permissions())
    }
}

impl CrdObjEC {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> ECCapPermissions {
        ECCapPermissions(self.gen_permissions())
    }
}

impl CrdObjSC {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> SCCapPermissions {
        SCCapPermissions(self.gen_permissions())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemCapPermissions(u8);
impl MemCapPermissions {
    pub fn read(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
    pub fn write(self) -> bool {
        (self.0 & (1 << 1)) != 0
    }
    pub fn execute(self) -> bool {
        (self.0 & (1 << 2)) != 0
    }
}
#[derive(Debug, Copy, Clone)]
pub struct PortIOCapPermissions(u8);
impl PortIOCapPermissions {
    pub fn new(can_read_write: bool) -> Self {
        let mut base = 0;
        if can_read_write {
            base |= 1;
        }
        Self(base)
    }
    pub fn read_write(self) -> bool {
        (self.0 & 1) != 0
    }
    pub fn val(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PDCapPermissions(u8);
impl PDCapPermissions {
    pub fn pd(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
    pub fn ec(self) -> bool {
        (self.0 & (1 << 1)) != 0
    }
    pub fn sc(self) -> bool {
        (self.0 & (1 << 2)) != 0
    }
    pub fn pt(self) -> bool {
        (self.0 & (1 << 3)) != 0
    }
    pub fn sm(self) -> bool {
        (self.0 & (1 << 4)) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ECCapPermissions(u8);
impl ECCapPermissions {
    /// If `ec_ctrl` is set, the `ec_ctrl` system call is permitted.
    pub fn ec_ctrl(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
    /// If `create_sc` is set, `create_sc` is allowed to bind a scheduling context.
    pub fn create_sc(self) -> bool {
        (self.0 & (1 << 2)) != 0
    }
    /// if `create_pt` is set, `create_pt` can bind a portal.
    pub fn create_pt(self) -> bool {
        (self.0 & (1 << 3)) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SCCapPermissions(u8);
impl SCCapPermissions {
    /// If `sc_ctrl` is set, the `sc_ctrl` system call is permitted.
    pub fn sc_ctrl(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
}

/// Portal permissions.
#[derive(Debug, Copy, Clone)]
pub struct PTCapPermissions(u8);
impl PTCapPermissions {
    /// If `pt_ctrl` is set, the `pt_ctrl` system call is permitted.
    pub fn pt_ctrl(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
    /// If `call` is set, the portal can be traversed using `call`.
    pub fn call(self) -> bool {
        (self.0 & (1 << 1)) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SMCapPermissions(u8);
impl SMCapPermissions {
    /// If `up` is set, the `sm_ctrl` system call is permitted to do an "up" operation.
    pub fn up(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }
    /// If `down` is set, the `sm_ctrl` system call is permitted to do a "down" operation.
    pub fn down(self) -> bool {
        (self.0 & (1 << 1)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hedron::capability::{
        CrdMem,
        CrdObjPD,
        CrdPortIO,
    };
    use core::mem::size_of;

    #[test]
    fn test_size() {
        assert_eq!(8, size_of::<CrdMem>());
        assert_eq!(8, size_of::<CrdPortIO>());
        assert_eq!(8, size_of::<CrdObjPD>());
        assert_eq!(8, size_of::<CrdObjSM>());
        assert_eq!(8, size_of::<CrdObjPT>());
        assert_eq!(8, size_of::<CrdObjEC>());
        assert_eq!(8, size_of::<CrdObjSC>());
    }
}
