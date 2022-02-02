//! Serial logger.

use crate::io_port::request_io_ports;
use core::fmt::{
    Debug,
    Formatter,
    Write,
};
use libhrstd::libhedron::{
    CapSel,
    CrdPortIO,
};
use uart_16550::SerialPort;

/// I/O Port on x86 platforms for the COM1 port/the serial device.
/// The I/O port connects the program to a uart16550 chip on the
/// chipset that handles the actual data transfer.
const COM1_IO_PORT: u16 = 0x3f8;

/// Logger that uses I/O port 0x3f8. See `serial_port.rs`.
///
/// **There should only be one instance of this!**
pub(super) struct SerialWriter(Option<uart_16550::SerialPort>);

impl Debug for SerialWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SerialWriter")
            .field(
                self.0
                    .as_ref()
                    .map(|_| &"Some(I/O Port 0x3f8)")
                    .unwrap_or(&"None"),
            )
            .finish()
    }
}

impl SerialWriter {
    pub fn new() -> Self {
        Self(None)
    }

    /// Initializes the serial logger for the roottask.
    /// Requests access to the necessary I/O ports.
    pub fn init(&mut self, root_pd_sel: CapSel) -> Result<(), ()> {
        // order 3: 2^3 = 8 => we need ports [port..port+8]
        request_io_ports(root_pd_sel, CrdPortIO::new(COM1_IO_PORT, 3)).map_err(|_| ())?;
        let mut port = unsafe { SerialPort::new(COM1_IO_PORT) };
        port.init();
        self.0.replace(port);
        Ok(())
    }
}

impl Write for SerialWriter {
    /// Writes the data to the I/O port.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        let _ = self.0.as_mut().unwrap().write_str(msg);
        Ok(())
    }
}
