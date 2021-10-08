//! Module for [`Utcb`] and sub structs.

use crate::mem::PAGE_SIZE;
use core::fmt::{
    Debug,
    Formatter,
};
use core::mem::size_of;

/// Capacity in bytes of the UTCB Data area.
const DATA_CAPACITY: usize = PAGE_SIZE - size_of::<UtcbHead>();
/// Capacity count of untyped items in UTCB Data area.
const UNTYPED_ITEM_CAPACITY: usize = DATA_CAPACITY / size_of::<UntypedItem>();
/// Capacity count for typed items in UTCB Data area.
const TYPED_ITEM_CAPACITY: usize = DATA_CAPACITY / size_of::<TypedItem>();

#[derive(Copy, Clone, Debug)]
pub enum UtcbError {
    /// Indicates that the payload is larger than [`DATA_CAPACITY`].
    PayloadTooLarge,
    /// Indicates that there are more untyped items than [`UNTYPED_ITEM_CAPACITY`].
    TooManyUntypedItems,
    /// Indicates that there are more typed items than [`TYPED_ITEM_CAPACITY`].
    TooManyTypedItems,
}

/// User Thread Control Block (UTCB). An execution context uses its UTCB to send or receive
/// messages (IPC), ~~to transfer typed items during capability delegation~~ (not used anymore,
/// see dedicated delegate syscall), and to get information after an exception.
///
/// An UTCB is never constructed but only reused and refilled.
///
/// Consists of [`UtcbHead`] and [`UtcbData`].
#[derive(Debug)]
#[repr(C, align(4096))]
pub struct Utcb {
    head: UtcbHead,
    data: UtcbData,
}

impl Utcb {
    /// Constructor for testing.
    #[cfg(test)]
    fn new() -> Self {
        Self {
            head: UtcbHead::new(),
            data: UtcbData::new(),
        }
    }

    /// Number of untyped items, alias arbitrary payload.
    pub fn number_untyped_items(&self) -> u16 {
        self.head.items as u16
    }

    /// Number of untyped items.
    pub fn number_typed_items(&self) -> u16 {
        (self.head.items >> 16) as u16
    }

    /// Returns the pointer to the beginning of the data area of the UTCB.
    /// The microhypervisor transfers untyped items from the here upwards.
    /// Each untyped item is 64 bit (one word) long.
    pub fn utcb_data_begin(&self) -> *const u8 {
        (&self.data) as *const UtcbData as *const u8
    }

    /// Returns all available untyped items as slice. The application must
    /// parse the data by itself. The microhypervisor transfers untyped items from the beginning of
    /// the UTCB data area upwards.
    pub fn untyped_items(&self) -> &[u64] {
        unsafe { &self.data.untyped_items[0..self.number_untyped_items() as usize] }
    }

    /// Sets the number of untyped items.
    fn set_number_untyped_items(&mut self, count: u16) -> Result<(), UtcbError> {
        if count as usize > UNTYPED_ITEM_CAPACITY {
            Err(UtcbError::TooManyUntypedItems)
        } else {
            let typed_items = self.number_typed_items() as u64;
            self.head.items = (typed_items << 16) | count as u64;
            Ok(())
        }
    }

    /// Sets the number of typed items.
    fn set_number_typed_items(&mut self, count: u16) -> Result<(), UtcbError> {
        if count as usize > UNTYPED_ITEM_CAPACITY {
            Err(UtcbError::TooManyTypedItems)
        } else {
            let untyped_items = self.number_untyped_items() as u64;
            self.head.items = (count as u64) << 16 | untyped_items;
            Ok(())
        }
    }

    /// Returns all available typed items as slice.
    /// The microhypervisor transfers typed items from the end of the UTCB data area downwards.
    /// Each typed item occupies two words.
    pub fn typed_items(&self) -> &[TypedItem] {
        // typed items are at end of array
        let end_i = unsafe { self.data.untyped_items.len() } - 1;
        let begin_i = end_i - self.number_typed_items() as usize;
        unsafe { &self.data.typed_items[begin_i..] }
    }

    /// Interprets the bytes in the "untyped items" area as a type `T` and
    /// returns a reference to it.
    pub fn load_data<T: Sized>(&self) -> Result<&T, UtcbError> {
        let required_byte_count = size_of::<T>();
        let size_untyped_item = size_of::<UntypedItem>();
        let avaiable_byte_count = self.number_untyped_items() as usize * size_untyped_item;
        if required_byte_count < avaiable_byte_count {
            log::warn!(
                "required_byte_count({}) < available_byte_count({})",
                required_byte_count,
                avaiable_byte_count
            );
            return Err(UtcbError::PayloadTooLarge);
        }
        let ptr = unsafe { self.data.untyped_items.as_ptr() as *const u8 as *const T };
        Ok(unsafe { ptr.as_ref().unwrap() })
    }

    /// Copies the bytes of T into the UTCB, if enough space is available. Overwrites any
    /// typed items, if the data is large enough.
    pub fn store_data<T: Sized>(&mut self, data: T) -> Result<(), UtcbError> {
        let required_size = size_of::<T>();
        let untyped_item_size = size_of::<UntypedItem>();
        if required_size > DATA_CAPACITY {
            Err(UtcbError::PayloadTooLarge)
        } else {
            let required_untyped_items = if required_size % untyped_item_size == 0 {
                required_size / untyped_item_size
            } else {
                (required_size / untyped_item_size) + 1
            };

            self.set_number_untyped_items(required_untyped_items as u16)?;
            self.set_number_typed_items(0)?;
            unsafe {
                core::ptr::write(self.data.untyped_items.as_mut_ptr() as *mut T, data);
            }
            Ok(())
        }
    }

    /// TODO naming
    pub fn exception_data(&self) -> &UtcbDataException {
        unsafe { &self.data.exception_data }
    }
}

/// User Thread Control Block.
/// Data-Buffer for IPC.
#[repr(C)]
union UtcbData {
    /// Used to transfer arbitrary data. The buffer is only filled with the count of items,
    /// that is defined in the header. Untyped items start from the beginning of the Utcb data
    /// area upwards.
    untyped_items: [u64; UNTYPED_ITEM_CAPACITY],
    /// Required for Delegate and Translate IPC calls. The buffer is only filled with the count of
    /// items, that is defined in the header. Typed items start from the end of the Utcb data
    /// area downwards.
    typed_items: [TypedItem; TYPED_ITEM_CAPACITY],
    /// TODO filled during exceptions or specific VM events?!
    exception_data: UtcbDataException,
}

impl UtcbData {
    /// Constructor for tests.
    #[cfg(test)]
    fn new() -> Self {
        Self {
            untyped_items: [0; UNTYPED_ITEM_CAPACITY],
        }
    }
}

impl Debug for UtcbData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UtcbData (Union)")
            .field("#exception_data", unsafe { &self.exception_data })
            .field("#items", &"<array>")
            .finish()
    }
}

#[derive(Debug)]
pub struct UtcbDataItems([u64; PAGE_SIZE - size_of::<UtcbHead>()]);

// this is copy because this is a limitation for unions in Rust currently
// TODO naming?!
#[derive(Debug, Copy, Clone)]
pub struct UtcbDataException {
    pub mtd: u64,
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
    /// Constructor for tests.
    #[cfg(test)]
    fn new() -> Self {
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
    }

    #[test]
    fn test_store_load_utcb() {
        let mut utcb = Utcb::new();
        assert_eq!(
            size_of_val(&utcb),
            PAGE_SIZE,
            "Utcb must be a page size long"
        );
        let array = [1_u64, 3, 3, 7];
        utcb.store_data(array).unwrap();

        assert_eq!(utcb.number_untyped_items(), 4);
        assert_eq!(utcb.number_typed_items(), 0);

        let copy = utcb.load_data::<[u64; 4]>().unwrap();
        assert_eq!(&array, copy);

        let large_array = [0_u8; PAGE_SIZE];
        assert!(
            utcb.store_data(large_array).is_err(),
            "data too big for utcb"
        );
    }
}
