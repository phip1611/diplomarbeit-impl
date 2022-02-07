//! Generic Address Structure (5.2.3.1)
//!
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)] // packed is important to match size from Hedron code!
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

#[cfg(test)]
mod tests {
    use crate::acpi_gas::{
        AcpiGas,
        Asid,
    };
    use core::mem::size_of;

    #[test]
    fn test_size_as_in_hedron() {
        assert_eq!(
            size_of::<Asid>(),
            1,
            "Asid must be as large as inside Hedron code"
        );
        assert_eq!(
            size_of::<AcpiGas>(),
            12,
            "AcpiGas must be as large as inside Hedron code"
        );
    }
}
