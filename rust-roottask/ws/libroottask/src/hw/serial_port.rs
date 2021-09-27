//! Serial Ports via x86 I/O ports. Also called COM ports.
//! <https://wiki.osdev.org/Serial_Ports>
//!
//! Its a wrapper around an UART bus.
//! Interesting infos can also be found under the name `16550_UART`

use x86::io::{
    inb,
    outb,
};

/// x86 puts the COM1 port at this address specified by the chipset.
/// For example here: <https://www.intel.com/content/dam/www/public/us/en/documents/datasheets/7-series-chipset-pch-datasheet.pdf>
/// Must most likelky in every chipset datasheet.
pub const COM1_IO_PORT: u16 = 0x3f8;

/// Offset to the IO port base address to
/// get access to specific registers.
///
/// <https://wiki.osdev.org/Serial_Ports>
#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum ComRegisterPortOffset {
    /// Reading this registers read from the Receive buffer. Writing to this register writes to the Transmit buffer.
    DataRegister = 0,
    InterruptEnable = 1,
    InterruptIdentification = 2,
    LineControlRegister = 3,
    ModemControlRegister = 4,
    LineStatusRegister = 5,
    ModemStatusRegister = 6,
    ScratchRegister = 7,
}

impl ComRegisterPortOffset {
    fn val(self) -> u16 {
        self as u16
    }
}

/// Initializes the legacy x86 I/O port serial line,
/// also known as COM1. It uses I/O port [`COM1_IO_PORT`].
///
/// Make sure you have the capability for the port first.
pub fn init_serial() -> Result<(), ()> {
    const PORT: u16 = COM1_IO_PORT;
    use ComRegisterPortOffset::*;

    // unfortunately I can't find reference for the
    // magic bits here. Legacy tutorials for even more
    // legacy functionality.

    unsafe {
        // Disable all interrupts
        outb(PORT + InterruptEnable.val(), 0);
        // Enable DLAB (set baud rate divisor)
        outb(PORT + LineControlRegister.val(), 0x80);
        // Set divisor to 3 (lo byte) 38400 baud
        outb(PORT + DataRegister.val(), 0x03);
        // (hi byte)
        outb(PORT + InterruptEnable.val(), 0x00);
        // 8 bits, no parity, one stop bit
        outb(PORT + LineControlRegister.val(), 0x03);
        // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT + InterruptIdentification.val(), 0xc7);
        // IRQs enabled, RTS (ready to send)/DSR (Data Set Ready) set
        outb(PORT + ModemControlRegister.val(), 0x0b);

        // Set in loopback mode, test the serial chip
        outb(PORT + ModemControlRegister.val(), 0x1e);
        // Test serial chip (send byte 0xAE and check if serial returns same byte)
        outb(PORT + DataRegister.val(), 0xae);

        // Check if serial is faulty (i.e: not same byte as sent)
        if inb(PORT + DataRegister.val()) != 0xAE {
            Err(())
        } else {
            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            outb(PORT + 4, 0x0F);
            Ok(())
        }
    }
}

/// Sends data via the serial connection.
/// Execute [`init_serial`] first!
pub fn snd_serial(data: &[u8]) {
    const PORT: u16 = COM1_IO_PORT;
    use ComRegisterPortOffset::*;
    for byte in data {
        unsafe {
            outb(PORT + DataRegister.val(), *byte);
        }
    }
}
