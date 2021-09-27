//! Generic Address Structure (5.2.3.1)
//!
#[derive(Debug)]
#[repr(C)]
pub struct AcpiGas {
    asid: Asid,
    /// Register Size in bits
    bits: u8,
    /// Register offset
    offset: u8,
    /// Access size
    access: u8,
    /// Register Address
    addr: u64,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Asid {
    MEMORY = 0x0,
    IO = 0x1,
    PciConfig = 0x2,
    EC = 0x3,
    SMBUS = 0x4,
    FIXED = 0x7f,
}
