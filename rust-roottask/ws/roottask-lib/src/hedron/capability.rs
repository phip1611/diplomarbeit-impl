use core::marker::PhantomData;
use core::mem::transmute;

/// Generic capability selector. Similar to a file
/// descriptor in UNIX. It indexes into the capability
/// space of the protection domain.
///
/// The application need to keep track what cap sel
/// refers to what. Similar to `int cfg_file = open("foo.json")`.
pub type CapSel = u64;

/// CRD used in situations where capabilities are referred in general inside
/// the capability space of a PD.
///
/// Used when creating PDs. See [`create_pd`]
pub type CrdGeneric = Crd<(), (), ()>;
/// CRD used to refer to memory (page) capabilities.
pub type CrdMem = Crd<MemCapPermissions, CrdTypeMem, ()>;
/// CRD used to refer to x86 Port I/O capabilities.
pub type CrdPortIO = Crd<PortIOCapPermissions, CrdTypePortIO, ()>;
/// CRD used to refer to capabilities for EC objects.
pub type CrdObjEC = Crd<ECCapPermissions, CrdTypeObject, CrdTypeObjectEC>;
/// CRD used to refer to capabilities for SC objects.
pub type CrdObjSC = Crd<SCCapPermissions, CrdTypeObject, CrdTypeObjectSC>;
/// CRD used to refer to capabilities for SM objects.
pub type CrdObjSM = Crd<SMCapPermissions, CrdTypeObject, CrdTypeObjectSM>;
/// CRD used to refer to capabilities for PD objects.
pub type CrdObjPD = Crd<PDCapPermissions, CrdTypeObject, CrdTypeObjectPD>;
/// CRD used to refer to capabilities for PT objects.
pub type CrdObjPT = Crd<PTCapPermissions, CrdTypeObject, CrdTypeObjectPT>;

/// Defines the kind of capabilities inside the capability
/// space of a PD inside the kernel.
#[derive(Debug, Copy, Clone, IntoEnumIterator)]
#[repr(u8)]
pub enum CrdKind {
    NullCrd = 0,
    MemoryCrd = 1,
    PortIoCrd = 2,
    ObjectCrd = 3,
}

impl CrdKind {
    /// Returns the raw unsigned integer value.
    pub fn val(self) -> u8 {
        self as u8
    }
}

impl From<u8> for CrdKind {
    /// Creates a CrdKind from an unsigned integer value.
    /// Panics if value is invalid.
    fn from(val: u8) -> Self {
        // generated during compile time; probably not recognized by IDE
        for variant in Self::into_enum_iter() {
            if variant.val() == val {
                return variant;
            }
        }
        panic!("invalid variant! id={}", val);
    }
}

/// Abstraction over different CRD versions.
/// Don't use directly. The size of this type is 8 byte.
#[derive(Debug, Copy, Clone)]
pub struct Crd<Permissions, Specialization, ObjectSpecialization> {
    val: u64,
    // zero size type; gone after compilation
    _zst1: PhantomData<Permissions>,
    _zst2: PhantomData<Specialization>,
    _zst3: PhantomData<ObjectSpecialization>,
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
impl<Permissions, Specialization, ObjectSpecialization> Crd<Permissions, Specialization, ObjectSpecialization> {
    const KIND_BITMASK: u64 = 0b11;
    const BASE_BITMASK: u64 = 0b111100;
    const BASE_LEFT_SHIFT: u64 = 12;
    const ORDER_BITMASK: u64 = 0b111_1000_0000;
    const ORDER_LEFT_SHIFT: u64 = 7;
    const PERMISSIONS_BITMASK: u64 = 0b111_1000_0000;
    const PERMISSIONS_LEFT_SHIFT: u64 = 2;

    /// Used to create generic Crds. Only use this if really necessary.
    /// Better use a more type-safe version.
    pub fn new_generic(kind: CrdKind, base: u64, order: u64, permissions: u64) -> Self {
        let mut this = 0;
        this |= kind & 0b11;
        this |= (base << Self::BASE_LEFT_SHIFT) & Self::BASE_BITMASK;
        this |= (order << Self::ORDER_LEFT_SHIFT) & Self::ORDER_BITMASK;
        this |= (permissions << Self::PERMISSIONS_LEFT_SHIFT) & Self::PERMISSIONS_BITMASK;
        Self {
            val: this,
            _zst1: PhantomData::default(),
            _zst2: PhantomData::default(),
            _zst3: PhantomData::default(),
        }
    }

    /// Returns the encoded u64 CRC. This is used as transfer type to the kernel. All properties
    /// are encoded at their corresponding bitshift-offset.
    pub fn val(self) -> u64 {
        self.val
    }
    /// Returns the kind
    pub fn kind(self) -> CrdKind {
        unsafe {
            // CrdKind is represented as u8, therefore valid
            transmute((self.val & 0b11) as u8)
        }
    }
    pub fn order(self) -> u8 {
        ((self.val & Self::ORDER_BITMASK) >> Self::ORDER_LEFT_SHIFT) as u8
    }

    pub fn base(self) -> u16 {
        ((self.val & Self::BASE_BITMASK) >> Self::BASE_LEFT_SHIFT) as u16
    }

    /// Returns the generic permissions, i.e. untyped.
    /// Better use a type-safe approach.
    pub fn gen_permissions(self) -> u8 {
        ((self.val & Self::PERMISSIONS_BITMASK) >> Self::PERMISSIONS_LEFT_SHIFT) as u8
    }
}

impl CrdMem {
    pub fn permissions(self) -> MemCapPermissions {
        MemCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

impl CrdPortIO {
    /// Creates the CRC to request read/write access to one or more I/O ports.
    pub fn new(port: u16, order: u16) -> Self {
        let mut base = 0_u64;
        base |= CrdKind::PortIoCrd as u64 & Self::KIND_BITMASK;
        // todo should this ever be initialized to false? Maybe for cap revoke?!
        base |= (PortIOCapPermissions::READ_WRITE.bits() as u64) << Self::PERMISSIONS_LEFT_SHIFT;
        base |= (port as u64) << Self::BASE_LEFT_SHIFT;
        base |= (order as u64) << Self::ORDER_LEFT_SHIFT;
        Self {
            val: base,
            // phantom data, not needed
            _zst1: PhantomData::default(),
            _zst2: PhantomData::<CrdTypePortIO>::default(),
            _zst3: PhantomData::default(),
        }
    }

    /// Creates the CRC to request read/write access to one or more I/O ports.
    pub fn permissions(self) -> PortIOCapPermissions {
        PortIOCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

impl CrdObjPD {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> PDCapPermissions {
        PDCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

impl CrdObjSM {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> SMCapPermissions {
        SMCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

impl CrdObjEC {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> ECCapPermissions {
        ECCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

impl CrdObjSC {
    /// Permission specific to ObjectSpecialization
    pub fn permissions(self) -> SCCapPermissions {
        SCCapPermissions::from_bits(self.gen_permissions()).unwrap()
    }
}

/// Helper macro for bits.
macro_rules! bit {
    ($num: literal) => {1 << $num};
}

bitflags::bitflags! {
    /// Permissions of a capability for a memory page.
    pub struct MemCapPermissions: u8 {
        const READ = bit!(0);
        const WRITE = bit!(1);
        const EXECUTE = bit!(2);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a x86 I/O port.
    pub struct PortIOCapPermissions: u8 {
        const READ_WRITE = bit!(0);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `PD` object.
    pub struct PDCapPermissions: u8 {
        /// The target PD can execute the `create_pd`-syscall.
        const CREATE_PD = bit!(0);
        /// The target PD can execute the `create_ec`-syscall.
        const CREATE_EC = bit!(1);
        /// The target PD can execute the `create_sc`-syscall.
        const CREATE_SC = bit!(2);
        /// The target PD can execute the `create_pt`-syscall.
        const CREATE_PT = bit!(3);
        /// The target PD can execute the `create_sm`-syscall.
        const CREATE_SM = bit!(4);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `EC` object.
    pub struct ECCapPermissions: u8 {
        /// The target PD can execute the `ec_ctrl`-syscall.
        const EC_CTRL = bit!(0);
        /// The target PD can execute the `create_sc`-syscall.
        const CREATE_SC = bit!(2);
        /// The target PD can execute the `create_pt`-syscall.
        const CREATE_PT = bit!(3);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `SC` object.
    pub struct SCCapPermissions: u8 {
        /// The target PD can execute the `sm_ctrl`-syscall.
        const SM_CTRL = bit!(0);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `PT` object.
    pub struct PTCapPermissions: u8 {
        /// The target PD can execute the `pt_ctrl`-syscall.
        const PT_CTRL = bit!(0);
        /// The target PD can execute the `pt_ctrl`-syscall.
        const CALL = bit!(1);
    }
}


bitflags::bitflags! {
    /// Permissions of a capability for a `SM` object.
    pub struct SMCapPermissions: u8 {
        /// The target PD can execute the `UP`-operation via the `sm_ctrl`-syscall.
        const UP = bit!(0);
        /// The target PD can execute the `DOWN`-operation via the `sm_ctrl`-syscall.
        const DOWN = bit!(1);
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

    fn test_bits() {

    }
}
