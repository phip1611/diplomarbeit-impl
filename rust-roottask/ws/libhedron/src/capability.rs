//! Relevant typings to access or request capabilities in/from the kernel.
//!
//! The most relevant items of this file are [`CapSel`] and the following specialisations
//! of [`Crd`]s.
//! - [`CrdNull`]
//! - [`CrdMem`]
//! - [`CrdPortIO`]
//! - [`CrdObjEC`]
//! - [`CrdObjSC`]
//! - [`CrdObjSM`]
//! - [`CrdObjPD`]
//! - [`CrdObjPT`]
//!
//! This module seems a bit overkill and over-engineered, but makes it unable to create invalid
//! Crds. Furthermore, it puts all knowledge about Crds from Hedron into Code.

use core::fmt::{
    Debug,
    Formatter,
};
use core::marker::PhantomData;
use enum_iterator::IntoEnumIterator;

/// Generic capability selector. It indexes into the capability space
/// of the protection domain, similar to a file descriptor in UNIX.
///
/// The application need to keep track what `CapSel` refers to what
/// capability. Similar to `int cfg_file_fd = open("foo.json")`.
pub type CapSel = u64;

/// Refers to a Null capability. See [`Crd`] for generic details.
pub type CrdNull = Crd<NullCapPermissions, (), ()>;
/// CRD used to refer to memory (page) capabilities. See [`Crd`] for generic details.
pub type CrdMem = Crd<MemCapPermissions, CrdTypeMem, ()>;
/// CRD used to refer to x86 Port I/O capabilities. See [`Crd`] for generic details.
pub type CrdPortIO = Crd<PortIOCapPermissions, CrdTypePortIO, ()>;
/// CRD used to refer to capabilities for EC objects. See [`Crd`] for generic details.
pub type CrdObjEC = Crd<ECCapPermissions, CrdTypeObject, CrdTypeObjectEC>;
/// CRD used to refer to capabilities for SC objects. See [`Crd`] for generic details.
pub type CrdObjSC = Crd<SCCapPermissions, CrdTypeObject, CrdTypeObjectSC>;
/// CRD used to refer to capabilities for SM objects. See [`Crd`] for generic details.
pub type CrdObjSM = Crd<SMCapPermissions, CrdTypeObject, CrdTypeObjectSM>;
/// CRD used to refer to capabilities for PD objects. See [`Crd`] for generic details.
pub type CrdObjPD = Crd<PDCapPermissions, CrdTypeObject, CrdTypeObjectPD>;
/// CRD used to refer to capabilities for PT objects. See [`Crd`] for generic details.
pub type CrdObjPT = Crd<PTCapPermissions, CrdTypeObject, CrdTypeObjectPT>;

/// Highest possible order for a [`Crd`]. A order has exactly 5 bits.
pub const MAX_CRD_ORDER: u8 = 0x1f;
/// Highest possible base for a [`Crd`]. A order has exactly 52 bits.
pub const MAX_CRD_BASE: u64 = 0x000f_ffff_ffff_ffff;

/// Defines the kind of capabilities inside the capability
/// space of a PD inside the kernel. First two bits of [`Crd`].
#[derive(Debug, Copy, Clone, IntoEnumIterator, PartialEq)]
#[repr(u8)]
pub enum CrdKind {
    /// Null capability. Default value of each index in the capability space
    /// for each PD in the kernel. Usually not needed in userspace.
    CrdKindNull = 0,
    /// Capability refers to memory, therefore a page or a range
    /// of memory pages in the (virtual) address space.
    CrdKindMemory = 1,
    /// Capability refers to a x86 I/O port.
    CrdKindPortIo = 2,
    /// Capability refers to a kernel object. The context, i.e. what kernel
    /// object depends on, gets derived from the syscall.
    CrdKindObject = 3,
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

/// A **C***apability ***R***ange ***D***escriptor* is used by the Hedron syscall
/// interface to describe what kind of capabilities should be created, revoked, or
/// delegated. **There are multiple kinds of CRDs, all depending on the [`CrdKind`]
/// and the context of the syscall.**
///
/// With a `Crd` you refer to a base, which is for example the number of the page
/// or the number of the I/O port. With the order, you can create a selector range
/// from the base to index `base..(base + 2^offset)`.
///
/// Don't use the raw Crd-type directly but rather [`CrdMem`], [`CrdPortIO`],
/// [`CrdObjEC`], [`CrdObjSC`], [`CrdObjSM`], [`CrdObjPD`], and [`CrdObjPT`].
pub struct Crd<Permissions, Specialization, ObjectSpecialization> {
    /// Contains the raw bits used to encode the CRD, according to the spec.
    val: u64,
    // zero size type; gone after compilation; required for generic types
    _zst1: PhantomData<Permissions>,
    _zst2: PhantomData<Specialization>,
    _zst3: PhantomData<ObjectSpecialization>,
}

/// Helper type for [`Crd`] constructors. Unsigned integer with 52 bits.
/// Only used internally to assert certain conditions. Not publicly
/// exported to not further increase complexity of the public API.
///
/// Invalid variants panics. This is so, because an invalid input is not
/// considered as tolerable runtime behaviour but as hard programming error.
#[derive(Copy, Clone)]
struct UI52Bit(u64);

impl From<u64> for UI52Bit {
    /// from-constructor. Invalid variants panics. This is so, because an invalid input is not
    /// considered as tolerable runtime behaviour but as hard programming error.
    fn from(value: u64) -> Self {
        if value > MAX_CRD_BASE {
            panic!(
                "Maximum of 52 bits allowed! The value {} is bigger than {}",
                value, MAX_CRD_BASE
            );
        }
        Self(value)
    }
}

impl UI52Bit {
    /// Returns the raw value of the base. The value is valid.
    pub fn val(self) -> u64 {
        self.0
    }
}

impl Debug for UI52Bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.val()))
    }
}

/// Helper type for [`Crd`] constructors. Unsigned integer with 5 bits.
/// Only used internally to assert certain conditions. Not publicly
/// exported to not further increase complexity of the public API.
///
/// Invalid variants panics. This is so, because an invalid input is not
/// considered as tolerable runtime behaviour but as hard programming error.
#[derive(Copy, Clone)]
struct UI5Bit(u8);

impl From<u8> for UI5Bit {
    /// from-constructor. Invalid variants panics. This is so, because an invalid input is not
    /// considered as tolerable runtime behaviour but as hard programming error.
    fn from(value: u8) -> Self {
        if value > MAX_CRD_ORDER {
            panic!(
                "Maximum of 5 bits allowed! The value {} is bigger than {}",
                value, MAX_CRD_ORDER
            );
        }
        Self(value)
    }
}

impl From<u64> for UI5Bit {
    // wrapper around the u8 from constructor
    fn from(value: u64) -> Self {
        Self::from(value as u8)
    }
}

impl UI5Bit {
    /// Returns the raw value of the order. The value is valid.
    pub fn val(self) -> u8 {
        self.0
    }
}

impl Debug for UI5Bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.val()))
    }
}

/// Enables specialisation for generic [`Crd`].
#[derive(Debug, Copy, Clone)]
pub struct CrdTypeNull;
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

impl<Permissions, Specialization, ObjectSpecialization>
    Crd<Permissions, Specialization, ObjectSpecialization>
{
    // kind is encoded in 2 bits
    const KIND_BITMASK: u64 = 0b11;
    // permission is encoded in 5 bits
    const PERMISSIONS_BITMASK: u64 = 0b111_1100;
    const PERMISSIONS_LEFT_SHIFT: u64 = 2;
    // order is encoded in 5 bits
    const ORDER_BITMASK: u64 = 0b1111_1000_0000;
    const ORDER_LEFT_SHIFT: u64 = 7;
    // bits 63..12 => 64-12=52 bits for base
    const BASE_BITMASK: u64 = !0xfff;
    const BASE_LEFT_SHIFT: u64 = 12;

    /// Constructs a new, unvalidated Crd from a u64 value.
    fn new_from_val(val: u64) -> Self {
        Self {
            val,
            _zst1: PhantomData::default(),
            _zst2: PhantomData::default(),
            _zst3: PhantomData::default(),
        }
    }

    /// Generic constructor. Can be used by the specialisations with stronger typed arguments
    /// or arguments with context-aware names.
    fn new_generic(kind: CrdKind, base: UI52Bit, order: UI5Bit, permissions: UI5Bit) -> Self {
        let mut val = 0;
        val |= kind.val() as u64 & Self::KIND_BITMASK;
        val |= ((permissions.val() as u64) << Self::PERMISSIONS_LEFT_SHIFT)
            & Self::PERMISSIONS_BITMASK;
        val |= ((order.val() as u64) << Self::ORDER_LEFT_SHIFT) & Self::ORDER_BITMASK;
        val |= (base.val() << Self::BASE_LEFT_SHIFT) & Self::BASE_BITMASK;
        Self {
            val,
            _zst1: PhantomData::default(),
            _zst2: PhantomData::default(),
            _zst3: PhantomData::default(),
        }
    }

    /// Returns the `Crd` as encoded u64 value. This is used as transfer type to the kernel.
    /// All properties are encoded at their corresponding bitshift-offset.
    pub fn val(self) -> u64 {
        self.val
    }

    /// Returns the [`CrdKind`] of this [`Crd`].
    pub fn kind(self) -> CrdKind {
        CrdKind::from((self.val & Self::KIND_BITMASK) as u8)
    }

    /// Returns the order of this [`Crd`]. `2^order` defines the range.
    pub fn order(self) -> u8 {
        ((self.val & Self::ORDER_BITMASK) >> Self::ORDER_LEFT_SHIFT) as u8
    }

    /// Returns the base of this [`Crd`]. The base can refer to the number
    /// of the I/O port or the (virtual) page number in memory. Depends
    /// on the Crd specialisation.
    pub fn base(self) -> u64 {
        (self.val & Self::BASE_BITMASK) >> Self::BASE_LEFT_SHIFT
    }

    /// Returns the generic permissions, i.e. untyped.
    /// Internal API.
    fn gen_permissions(self) -> u8 {
        ((self.val & Self::PERMISSIONS_BITMASK) >> Self::PERMISSIONS_LEFT_SHIFT) as u8
    }
}

impl<Permissions, Specialization, ObjectSpecialization> Clone
    for Crd<Permissions, Specialization, ObjectSpecialization>
{
    fn clone(&self) -> Self {
        Self::new_from_val(self.val)
    }
}

impl<Permissions, Specialization, ObjectSpecialization> Copy
    for Crd<Permissions, Specialization, ObjectSpecialization>
{
}

// Common getter for all permissions
impl<Permissions, Specialization, ObjectSpecialization>
    Crd<Permissions, Specialization, ObjectSpecialization>
where
    Permissions: CrdPermissions,
{
    /// Returns the [`CrdPermissions`]-type of this [`Crd`]. Depends on the context.
    pub fn permissions(self) -> Permissions {
        Permissions::from(self.gen_permissions())
    }
}

// Default trait
impl<Permissions, Specialization, ObjectSpecialization> Default
    for Crd<Permissions, Specialization, ObjectSpecialization>
where
    Permissions: CrdPermissions,
{
    fn default() -> Self {
        Self::new_generic(
            CrdKind::CrdKindNull,
            0.into(),
            0_u8.into(),
            Permissions::default().val().into(),
        )
    }
}

// PartialEq trait
impl<Permissions, Specialization, ObjectSpecialization> PartialEq
    for Crd<Permissions, Specialization, ObjectSpecialization>
{
    fn eq(&self, other: &Self) -> bool {
        // Todo make copy self.val() == other.val() ? why doen't it work?!
        self.val == other.val
    }
}

impl CrdNull {
    /// Creates the CRD to request or alternate memory mappings (like permissions).
    pub fn new() -> Self {
        Self::default()
    }
}

impl Debug for CrdNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("CrdNull").field(&0).finish()
    }
}

impl CrdMem {
    /// Creates the CRD to request or alternate memory mappings (like permissions).
    pub fn new(memory_page_num: u64, order: u8, permissions: MemCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindMemory,
            memory_page_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdMem {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdMem")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (page)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdPortIO {
    /// Creates the CRD to request read/write access to one or more I/O ports.
    pub fn new(port: u16, order: u8) -> Self {
        Self::new_generic(
            CrdKind::CrdKindPortIo,
            (port as u64).into(),
            order.into(),
            PortIOCapPermissions::READ_WRITE.val().into(),
        )
    }
}

impl Debug for CrdPortIO {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdPortIO")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (port)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdObjPD {
    /// Creates a new CRD for a portal object capability. This CRD is
    /// of kind [`CrdKind::CrdKindObject`]. Therefore, it only refers
    /// to a PD, if it is used in the right context, i.e. correct syscall.
    pub fn new(pd_num: u64, order: u8, permissions: PDCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindObject,
            pd_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdObjPD {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdObjPD")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (pd)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdObjSM {
    /// Creates a new CRD for a semaphore object capability. This CRD is
    /// of kind [`CrdKind::CrdKindObject`]. Therefore, it only refers
    /// to a SM, if it is used in the right context, i.e. correct syscall.
    pub fn new(sm_num: u64, order: u8, permissions: SMCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindObject,
            sm_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdObjSM {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdObjSM")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (sm)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdObjEC {
    /// Creates a new CRD for a execution context object capability. This CRD is
    /// of kind [`CrdKind::CrdKindObject`]. Therefore, it only refers
    /// to a EC, if it is used in the right context, i.e. correct syscall.
    pub fn new(ec_num: u64, order: u8, permissions: ECCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindObject,
            ec_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdObjEC {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdObjEC")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (ec)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdObjSC {
    /// Creates a new CRD for a scheduling context object capability. This CRD is
    /// of kind [`CrdKind::CrdKindObject`]. Therefore, it only refers
    /// to a SC, if it is used in the right context, i.e. correct syscall.
    pub fn new(sc_num: u64, order: u8, permissions: SCCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindObject,
            sc_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdObjSC {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdObjSC")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (sc)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

impl CrdObjPT {
    /// Creates a new CRD for a portal object capability. This CRD is
    /// of kind [`CrdKind::CrdKindObject`]. Therefore, it only refers
    /// to a PT, if it is used in the right context, i.e. correct syscall.
    pub fn new(pt_num: u64, order: u8, permissions: PTCapPermissions) -> Self {
        Self::new_generic(
            CrdKind::CrdKindObject,
            pt_num.into(),
            order.into(),
            permissions.val().into(),
        )
    }
}

impl Debug for CrdObjPT {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CrdObjPT")
            // trick: print as pointer => print as hex
            .field("val (u64)", &(self.val as *const u64))
            .field("kind", &self.kind())
            .field("base (pt)", &self.base())
            .field("order", &self.order())
            .field("2^order", &2_u64.pow(self.order() as u32))
            .field("permissions", &self.permissions())
            .finish()
    }
}

/// Shared trait for all permission implementations.
pub trait CrdPermissions: From<u8> + Into<u8> + Default {
    /// Returns a raw unsigned integer with the permission bits.
    fn val(self) -> u8 {
        self.into()
    }
}

/// Helper macro for bits.
macro_rules! bit {
    ($num: literal) => {
        1 << $num
    };
}

bitflags::bitflags! {
    /// Permissions for a null capability. Only used to better fit into
    /// the type structure of this module.
    pub struct NullCapPermissions: u8 {
    }
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
        /// The target PD can execute the `ec_ctrl`-syscall on the given execution context.
        const EC_CTRL = bit!(0);
        /// The target PD can execute the `create_sc`-syscall on the given execution context.
        const CREATE_SC = bit!(2);
        /// The target PD can execute the `create_pt`-syscall on the given execution context.
        const CREATE_PT = bit!(3);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `SC` object.
    pub struct SCCapPermissions: u8 {
        /// The target PD can execute the `sc_ctrl`-syscall on the given scheduling context.
        const SC_CTRL = bit!(0);
    }
}

bitflags::bitflags! {
    /// Permissions of a capability for a `PT` object.
    pub struct PTCapPermissions: u8 {
        /// The target PD can execute the `pt_ctrl`-syscall on the given portal.
        const PT_CTRL = bit!(0);
        /// The target PD can execute the `call`-syscall on the given portal.
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

/// Helper struct to remove lots of boilerplate code implementations.
macro_rules! impl_permission_traits {
    ($name: ident) => {
        impl $name {
            /// Returns the raw unsigned integer of the permission bits.
            pub fn val(self) -> u8 {
                self.into()
            }
        }

        impl From<u8> for $name {
            fn from(val: u8) -> Self {
                Self::from_bits(val).unwrap()
            }
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                self.bits()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::from(0)
            }
        }

        impl CrdPermissions for $name {}
    };
}

impl_permission_traits!(NullCapPermissions);
impl_permission_traits!(MemCapPermissions);
impl_permission_traits!(PortIOCapPermissions);
impl_permission_traits!(PTCapPermissions);
impl_permission_traits!(PDCapPermissions);
impl_permission_traits!(ECCapPermissions);
impl_permission_traits!(SCCapPermissions);
impl_permission_traits!(SMCapPermissions);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{
        CrdMem,
        CrdObjPD,
        CrdPortIO,
    };
    use core::mem::size_of;

    #[test]
    fn test_size() {
        assert_eq!(8, size_of::<CrdNull>());
        assert_eq!(8, size_of::<CrdMem>());
        assert_eq!(8, size_of::<CrdPortIO>());
        assert_eq!(8, size_of::<CrdObjPD>());
        assert_eq!(8, size_of::<CrdObjSM>());
        assert_eq!(8, size_of::<CrdObjPT>());
        assert_eq!(8, size_of::<CrdObjEC>());
        assert_eq!(8, size_of::<CrdObjSC>());
    }

    #[test]
    fn test_equal() {
        let crd = CrdNull::new();
        assert_eq!(crd, crd);
        let crd = CrdMem::new(5, 0, MemCapPermissions::all());
        assert_eq!(crd, crd);
        let crd = CrdPortIO::new(0xe9, 4);
        assert_eq!(crd, crd);
        let crd = CrdObjEC::new(7, 4, ECCapPermissions::all());
        assert_eq!(crd, crd);
        let crd = CrdObjSC::new(7, 4, SCCapPermissions::all());
        assert_eq!(crd, crd);
        let crd = CrdObjSM::new(7, 4, SMCapPermissions::all());
        assert_eq!(crd, crd);
        let crd = CrdObjPT::new(7, 4, PTCapPermissions::all());
        assert_eq!(crd, crd);
        let crd = CrdObjPD::new(7, 4, PDCapPermissions::all());
        assert_eq!(crd, crd);
    }

    #[test]
    fn test_copy() {
        let crd = CrdNull::new();
        let _val = { crd.val() };
        let _crd2 = crd;
        let _crd3 = crd.clone();
    }

    #[test]
    fn test_debug_print() {
        println!("{:?}", CrdNull::new());
        println!("{:?}", CrdMem::new(512, 0, MemCapPermissions::all()));
        println!("{:?}", CrdPortIO::new(0xe9, 4));
        println!("{:?}", CrdObjEC::new(5, 0x1f, ECCapPermissions::all()));
        println!("{:?}", CrdObjSC::new(5, 0, SCCapPermissions::all()));
        println!("{:?}", CrdObjSM::new(5, 0, SMCapPermissions::all()));
        println!("{:?}", CrdObjPT::new(5, 0, PTCapPermissions::all()));
        println!("{:?}", CrdObjPD::new(5, 0, PDCapPermissions::all()));
    }

    #[test]
    fn test_bits() {
        let base = MAX_CRD_BASE;
        let order = MAX_CRD_ORDER;
        let crd = CrdMem::new(base, order, MemCapPermissions::all());

        assert_eq!(CrdKind::CrdKindMemory, crd.kind());
        assert_eq!(MemCapPermissions::all(), crd.permissions());
        assert_eq!(base, crd.base());
        assert_eq!(order, crd.order());

        let val = crd.val();
        let expected_kind_bits = 0b01;
        let expected_permission_bits = 0b00111 << 2;

        // important, cast order to u64; otherwise value gets truncated
        let expected_order_bits = (order as u64) << 7;
        let expected_base_bits = base << 12;
        assert_eq!(expected_kind_bits, val & 0x3, "kind bits must match");
        assert_eq!(
            expected_permission_bits,
            val & CrdNull::PERMISSIONS_BITMASK,
            "permission bits must match"
        );
        assert_eq!(
            expected_order_bits as u64,
            val & CrdNull::ORDER_BITMASK,
            "order bits must match"
        );
        assert_eq!(
            expected_base_bits,
            val & CrdNull::BASE_BITMASK,
            "base bits must match"
        );
    }
}
