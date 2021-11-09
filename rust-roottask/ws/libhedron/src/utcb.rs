//! Module for [`Utcb`] and sub structs including utility functions to efficiently work
//! with the UTCB.
//!
//! To serialize/deserialize arbitrary user data into the UTCB, this crate uses `postcard`
//! as implementation for `serde`. It is mandatory, that this happens without heap allocations,
//! because in native Hedron-apps we need a portal call to allocate memory, therefore we must
//! avoid the chicken-egg problem!

use crate::mem::PAGE_SIZE;
use crate::mtd::Mtd;
use arrayvec::ArrayString;
use core::fmt::{
    Debug,
    Formatter,
};
use core::mem::{
    size_of,
    size_of_val,
};
use serde::{
    Deserialize,
    Serialize,
};

/// Capacity in bytes of the UTCB Data area.
pub const UTCB_DATA_CAPACITY: usize = PAGE_SIZE - size_of::<UtcbHead>();
/// Capacity count of untyped items in UTCB Data area.
pub const UNTYPED_ITEM_CAPACITY: usize = UTCB_DATA_CAPACITY / size_of::<UntypedItem>();
/// Capacity count for typed items in UTCB Data area.
pub const TYPED_ITEM_CAPACITY: usize = UTCB_DATA_CAPACITY / size_of::<TypedItem>();

#[derive(Clone, Debug)]
pub enum UtcbError {
    /// Indicates that the payload is larger than [`UTCB_DATA_CAPACITY`].
    PayloadTooLarge,
    /// Indicates that there are more untyped items than [`UNTYPED_ITEM_CAPACITY`].
    TooManyUntypedItems,
    /// Indicates that there are more typed items than [`TYPED_ITEM_CAPACITY`].
    TooManyTypedItems,
    /// Indicates that there was an error during deserialization.
    DeserializeError(postcard::Error),
    /// No data, when data was expected.
    NoData,
}

/// User Thread Control Block (UTCB). An execution context uses it's UTCB for
/// IPC and Exception handling. An UTCB is page-aligned and one page in size.
/// Consists of [`UtcbHead`] and [`UtcbData`].
///
/// **The UTCB is always allocated from Kernel heap, when an EC is created.**
/// Hence, it is not the responsibility of the Roottask or so to allocate
/// physical memory for it and delegate the mem caps to the PD of the EC.
///
/// # IPC
/// * transfer typed (NOVA-way for capability translation and delegation items
/// * transfer untyped items (= arbitrary, context-specific data)
/// # Exception Handling (and Answering)
/// * See [`UtcbDataException`].
#[repr(C, align(4096))]
pub struct Utcb {
    head: UtcbHead,
    data: UtcbData,
}

impl Utcb {
    /// A UTCB is never constructed by userland but always allocated from Kernel heap.
    #[cfg(test)]
    pub const fn new() -> Self {
        Self {
            head: UtcbHead::new(),
            data: UtcbData::new(),
        }
    }

    /// Returns a pointer to self.
    pub const fn self_ptr(&self) -> *const Utcb {
        self as *const _
    }

    /// Returns the page number of the UTCB. Panics if the UTCB is not page-aligned.
    pub fn page_num(&self) -> u64 {
        let ptr = self.self_ptr();
        let page_addr = ptr as usize;
        assert_eq!(page_addr % PAGE_SIZE, 0, "must be page aligned!");
        (page_addr / PAGE_SIZE) as u64
    }

    /// Number of untyped items, alias arbitrary payload.
    pub fn untyped_items_count(&self) -> u16 {
        self.head.items as u16
    }

    /// Number of typed items.
    pub fn typed_items_count(&self) -> u16 {
        (self.head.items >> 16) as u16
    }

    /// Returns all available untyped items as slice. The application must
    /// parse the data by itself. The microhypervisor transfers untyped items from the beginning of
    /// the UTCB data area upwards.
    pub fn untyped_items(&self) -> &[u64] {
        &self.data.untyped_items()[0..self.untyped_items_count() as usize]
    }

    /// Sets the number of untyped items.
    fn set_number_untyped_items(&mut self, count: u16) -> Result<(), UtcbError> {
        if count as usize > UNTYPED_ITEM_CAPACITY {
            Err(UtcbError::TooManyUntypedItems)
        } else {
            let typed_items = self.typed_items_count() as u64;
            self.head.items = (typed_items << 16) | count as u64;
            Ok(())
        }
    }

    /// Sets the number of typed items.
    fn set_number_typed_items(&mut self, count: u16) -> Result<(), UtcbError> {
        if count as usize > UNTYPED_ITEM_CAPACITY {
            Err(UtcbError::TooManyTypedItems)
        } else {
            let untyped_items = self.untyped_items_count() as u64;
            self.head.items = (count as u64) << 16 | untyped_items;
            Ok(())
        }
    }

    /// Returns all available typed items as slice.
    /// The microhypervisor transfers typed items from the end of the UTCB data area downwards.
    /// Each typed item occupies two words.
    pub fn typed_items(&self) -> &[TypedItem] {
        // typed items are at end of array
        let end_i = UNTYPED_ITEM_CAPACITY - 1;
        let begin_i = end_i - self.typed_items_count() as usize;
        &self.data.typed_items()[begin_i..]
    }

    /// Loads data from the UTCB, that was stored using [`store_data`].
    /// Returns a new, owned copy. Doesn't require heap allocations.
    pub fn load_data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, UtcbError> {
        if self.untyped_items_count() == 0 {
            return Err(UtcbError::NoData);
        }

        // postcard itself already encodes slices with their length properly

        let res = postcard::from_bytes(self.data.bytes())
            .map_err(|err| UtcbError::DeserializeError(err))?;

        Ok(res)
    }

    /// Puts arbitrary data into the UTCB using `serde` + `bincode`. It's a wrapper around
    /// the "untyped item"-fature of the UTCB.
    /// Note that size is limited to [`UTCB_DATA_CAPACITY`].
    /// Ignores/overwrite any typed items in the UTCB.
    ///
    /// Doesn't require heap allocations.
    pub fn store_data<T: Serialize>(&mut self, data: &T) -> Result<(), UtcbError> {
        if size_of_val(data) == 0 {
            return Err(UtcbError::NoData);
        }

        let serialized_bytes = postcard::to_slice(data, self.data.bytes_mut())
            .map_err(|_err| UtcbError::PayloadTooLarge)?;

        let untyped_item_size = size_of::<UntypedItem>();
        let required_untyped_items = if serialized_bytes.len() % untyped_item_size == 0 {
            serialized_bytes.len() / untyped_item_size
        } else {
            (serialized_bytes.len() / untyped_item_size) + 1
        };

        self.set_number_untyped_items(required_untyped_items as u16)?;
        self.set_number_typed_items(0)?;

        Ok(())
    }

    /// Returns the data as reference to [`UtcbDataException`].
    pub fn exception_data(&self) -> &UtcbDataException {
        self.data.exception_data()
    }

    /// Returns the data as mutable reference to [`UtcbDataException`].
    pub fn exception_data_mut(&mut self) -> &mut UtcbDataException {
        self.data.exception_data_mut()
    }
}

impl Debug for Utcb {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        let mut name = arrayvec::ArrayString::<64>::new();
        write!(&mut name, "Utcb (@ {:?})", &(self as *const Self))?;
        f.debug_struct(name.as_str())
            .field("head", &self.head)
            .field("data (typed items)", &self.typed_items_count())
            .field("data (untyped items)", &self.untyped_items_count())
            .finish()
    }
}

/// User Thread Control Block. Depending on the context this contains:
/// * typed items (for the legacy capability translate and delegate calls)
/// * untyped items (arbitrary data)
/// * exception or event data
#[repr(C)]
pub union UtcbData {
    /// Raw byte accessor. Starts at the same location as `untyped items`, i.e. both grow upwards.
    bytes: [u8; UTCB_DATA_CAPACITY],
    /// Used to transfer arbitrary data. The buffer is only filled with the count of items,
    /// that is defined in the header. Untyped items start from the beginning of the Utcb data
    /// area upwards.
    untyped_items: [u64; UNTYPED_ITEM_CAPACITY],
    /// Required for Delegate and Translate IPC calls. The buffer is only filled with the count of
    /// items, that is defined in the header. Typed items start from the end of the Utcb data
    /// area downwards.
    typed_items: [TypedItem; TYPED_ITEM_CAPACITY],
    /// See [`UtcbDataException`].
    exception_data: UtcbDataException,
}

#[allow(dead_code)]
impl UtcbData {
    /// Constructor.
    pub const fn new() -> Self {
        // Initializes the union with zeroes only.
        Self {
            bytes: [0; UTCB_DATA_CAPACITY],
        }
    }

    fn bytes(&self) -> &[u8; UTCB_DATA_CAPACITY] {
        unsafe { &self.bytes }
    }

    fn bytes_mut(&mut self) -> &mut [u8; UTCB_DATA_CAPACITY] {
        unsafe { &mut self.bytes }
    }

    fn untyped_items(&self) -> &[u64; UNTYPED_ITEM_CAPACITY] {
        unsafe { &self.untyped_items }
    }

    fn untyped_items_mut(&mut self) -> &mut [u64; UNTYPED_ITEM_CAPACITY] {
        unsafe { &mut self.untyped_items }
    }

    fn typed_items(&self) -> &[TypedItem; TYPED_ITEM_CAPACITY] {
        unsafe { &self.typed_items }
    }

    fn typed_items_mut(&mut self) -> &mut [TypedItem; TYPED_ITEM_CAPACITY] {
        unsafe { &mut self.typed_items }
    }

    fn exception_data(&self) -> &UtcbDataException {
        unsafe { &self.exception_data }
    }

    fn exception_data_mut(&mut self) -> &mut UtcbDataException {
        unsafe { &mut self.exception_data }
    }
}

impl Debug for UtcbData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        let mut buf = ArrayString::<64>::new();
        let non_null_bytes = unsafe { self.bytes.iter().filter(|x| **x != 0).count() };
        write!(&mut buf, "{} non-null bytes in union", non_null_bytes)?;
        write!(f, "UtcbData(")?;
        write!(f, "{}", buf)?;
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct UtcbDataItems([u64; PAGE_SIZE - size_of::<UtcbHead>()]);

/// Payload structure of [`UtcbData`] if a portal gets called after an event (exception or VM exit).
/// What data is filled here depends on the [`super::mtd::Mtd`] that is attached to the portal.
///
/// It is also used as payload for the REPLY syscall after an exception. According to the
/// MTD, the registers will be set.
#[derive(Copy, Clone)]
// this is copy because this is a limitation for unions in Rust currently
#[repr(C)]
pub struct UtcbDataException {
    pub mtd: Mtd,
    pub inst_len: u64,
    pub rip: u64,
    pub rflags: u64,
    pub intr_state: u32,
    pub actv_state: u32,
    pub intr_info: u32,
    pub intr_error: u32,
    pub vect_info: u32,
    pub vect_error: u32,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub qual: [u64; 2],
    pub ctrl: [u32; 2],
    pub xrc0: u64,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub pdpte: [u64; 4],
    pub cr8: u64,
    pub efer: u64,
    pub pat: u64,
    pub star: u64,
    pub lstar: u64,
    pub fmask: u64,
    pub kernel_gs_base: u64,
    pub dr7: u64,
    pub sysenter_cs: u64,
    pub sysenter_rsp: u64,
    pub sysenter_rip: u64,
    pub es: UtcbSegment,
    pub cs: UtcbSegment,
    pub ss: UtcbSegment,
    pub ds: UtcbSegment,
    pub fs: UtcbSegment,
    pub gs: UtcbSegment,
    pub ld: UtcbSegment,
    pub tr: UtcbSegment,
    pub gd: UtcbSegment,
    pub id: UtcbSegment,
    pub tsc_val: u64,
    pub tsc_off: u64,
    pub tsc_aux: u32,
    pub exc_bitmap: u32,
    pub tpr_threshold: u32,
    _reserved2: u32,

    pub eoi_bitmap: [u64; 4],

    pub vintr_status: u16,
    _reserved_array: [u16; 3],

    pub cr0_mon: u64,
    pub cr4_mon: u64,
    pub spec_ctrl: u64,
    pub tsc_timeout: u64,
}

impl Debug for UtcbDataException {
    fn fmt(&self, f: &mut Formatter<'_>) -> serde::__private::fmt::Result {
        f.debug_struct("UtcbDataException")
            .field("mtd", &self.mtd)
            .field("inst_len", &self.inst_len)
            .field("rip", &(self.rip as *const u64))
            .field("rflags", &(self.rflags as *const u64))
            .field("intr_state", &(self.intr_state as *const u32))
            .field("actv_state", &(self.actv_state as *const u32))
            .field("intr_info", &(self.intr_info as *const u32))
            .field("intr_error", &(self.intr_error as *const u32))
            .field("vect_info", &(self.vect_info as *const u32))
            .field("vect_error", &(self.vect_error as *const u32))
            .field("rax", &(self.rax as *const u64))
            .field("rcx", &(self.rcx as *const u64))
            .field("rdx", &(self.rdx as *const u64))
            .field("rbx", &(self.rbx as *const u64))
            .field("rsp", &(self.rsp as *const u64))
            .field("rbp", &(self.rbp as *const u64))
            .field("rsi", &(self.rsi as *const u64))
            .field("rdi", &(self.rdi as *const u64))
            .field("r8", &(self.r8 as *const u64))
            .field("r9", &(self.r9 as *const u64))
            .field("r10", &(self.r10 as *const u64))
            .field("r11", &(self.r11 as *const u64))
            .field("r12", &(self.r12 as *const u64))
            .field("r13", &(self.r13 as *const u64))
            .field("r14", &(self.r14 as *const u64))
            .field("r15", &(self.r15 as *const u64))
            .field("qual", &self.qual)
            .field("ctrl", &self.ctrl)
            .field("xrc0", &(self.xrc0 as *const u64))
            .field("cr0", &(self.cr0 as *const u64))
            .field("cr2", &(self.cr2 as *const u64))
            .field("cr3", &(self.cr3 as *const u64))
            .field("cr4", &(self.cr4 as *const u64))
            .field("pdpte", &self.pdpte)
            .field("cr8", &(self.cr8 as *const u64))
            .field("efer", &(self.efer as *const u64))
            .field("pat", &(self.pat as *const u64))
            .field("star", &(self.star as *const u64))
            .field("lstar", &(self.lstar as *const u64))
            .field("fmask", &(self.fmask as *const u64))
            .field("kernel_gs_base", &(self.kernel_gs_base as *const u64))
            .field("dr7", &(self.dr7 as *const u64))
            .field("sysenter_cs", &(self.sysenter_cs as *const u64))
            .field("sysenter_rsp", &(self.sysenter_rsp as *const u64))
            .field("sysenter_rip", &(self.sysenter_rip as *const u64))
            .field("es", &self.es)
            .field("cs", &self.cs)
            .field("ss", &self.ss)
            .field("ds", &self.ds)
            .field("fs", &self.fs)
            .field("gs", &self.gs)
            .field("ld", &self.ld)
            .field("tr", &self.tr)
            .field("gd", &self.gd)
            .field("id", &self.id)
            .field("tsc_val", &(self.tsc_val as *const u64))
            .field("tsc_off", &(self.tsc_off as *const u64))
            .field("tsc_aux", &(self.tsc_aux as *const u32))
            .field("exc_bitmap", &(self.exc_bitmap as *const u32))
            .field("tpr_threshold", &(self.tpr_threshold as *const u32))
            .field("eoi_bitmap", &self.eoi_bitmap)
            .field("vintr_status", &(self.vintr_status as *const u16))
            .field("cr0_mon", &(self.cr0_mon as *const u64))
            .field("cr4_mon", &(self.cr4_mon as *const u64))
            .field("spec_ctrl", &(self.spec_ctrl as *const u64))
            .field("tsc_timeout", &(self.tsc_timeout as *const u64))
            .finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct UtcbHead {
    /// Number of typed items. The IPC sender
    /// fills this value.
    pub items: u64,
    /// CRD for capability translation. NOVA-feature that we don't use.
    _xlt: u64,
    /// CRD for capability delegation. NOVA-feature that we don't use (see dedicated delegate syscall)
    _dlt: u64,
    /// This field is never written by the Microhypervisor and can be used to store thread-local data.
    pub tls: u64,
}

impl UtcbHead {
    /// Constructor.
    pub const fn new() -> Self {
        Self {
            items: 0,
            _xlt: 0,
            _dlt: 0,
            tls: 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct UtcbSegment {
    pub sel: u16,
    pub ar: u16,
    pub limit: u32,
    pub base: u64,
}

pub type UntypedItem = u64;

/// Typed item for delegation or translate capability operations in NOVA. Not used anymore
/// in favor of more expressive, dedicated syscalls. Stands here only for completeness in typings.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct TypedItem {
    a: u64,
    b: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;
    use core::mem::size_of_val;

    /// Tests if the sizes of the structs have an equal size to the size
    /// in Hedron. I printed the sizeof() values in Hedron to easily get this value.
    #[test]
    fn test_sizes() {
        assert_eq!(
            size_of::<UtcbHead>(),
            32,
            "UtcbHead must be as big as inside Hedron"
        );
        assert_eq!(
            size_of::<UtcbSegment>(),
            16,
            "UtcbHead must be as big as inside Hedron"
        );
        assert_eq!(
            size_of::<UtcbDataException>(),
            632,
            "UtcbDataException must be as big as inside Hedron"
        );
        assert_eq!(
            size_of::<UtcbData>(),
            PAGE_SIZE - size_of::<UtcbHead>(),
            "UtcbData must be as big as inside Hedron"
        );
        assert_eq!(
            size_of::<Utcb>(),
            PAGE_SIZE,
            "Utcb must be a page size long"
        );

        // test that all UtcbDataUnion-Fields have the same size
        let utcb = UtcbData::new();
        unsafe {
            /*assert_eq!(
                size_of_val(&utcb.bytes),
                size_of_val(&utcb.exception_data),
            );*/
            assert_eq!(size_of_val(&utcb.bytes), size_of_val(&utcb.untyped_items));
            assert_eq!(size_of_val(&utcb.bytes), size_of_val(&utcb.typed_items));
            assert_eq!(size_of_val(&utcb.bytes), size_of::<UtcbData>());
        }
    }

    /// Tests to store and load arbitrary data types from and to the untyped item section of the UTCB.
    #[test]
    fn test_store_load_utcb() {
        let mut utcb = Utcb::new();
        assert_eq!(
            size_of_val(&utcb),
            PAGE_SIZE,
            "Utcb must be a page size long"
        );
        let array = [1_u64, 3, 3, 7];
        utcb.store_data(&array).unwrap();

        assert_eq!(utcb.untyped_items_count(), 4);
        assert_eq!(utcb.typed_items_count(), 0);

        let copy = utcb.load_data::<[u64; 4]>().unwrap();
        assert_eq!(array, copy);

        let msg = "foobar hello world lorem ipsum".repeat(135);
        utcb.store_data(&msg.as_str()).unwrap();
        let copy = utcb.load_data::<&str>().unwrap();
        assert_eq!(copy, msg);
    }
}
