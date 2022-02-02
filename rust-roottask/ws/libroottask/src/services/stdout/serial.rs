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
    HIP,
};
use uart_16550::SerialPort;

/// Logger that uses I/O port 0x3f8. See `serial_port.rs`.
///
/// **There should only be one instance of this!**
pub(super) struct SerialWriter {
    port_base: u16,
    port: Option<uart_16550::SerialPort>,
}

impl Debug for SerialWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SerialWriter")
            .field("port_base", &self.port_base)
            .field(
                "port",
                if self.port.is_some() {
                    &"initialized"
                } else {
                    &"not initialized"
                },
            )
            .finish()
    }
}

impl SerialWriter {
    pub fn new(hip: &HIP) -> Self {
        Self {
            port_base: hip.serial_port(),
            port: None,
        }
    }

    /// Initializes the serial logger for the roottask.
    /// Requests access to the necessary I/O ports.
    pub fn init(&mut self, root_pd_sel: CapSel) -> Result<(), ()> {
        // order 3: 2^3 = 8 => we need ports [port..port+8]
        request_io_ports(root_pd_sel, CrdPortIO::new(self.port_base, 3)).map_err(|_| ())?;
        let mut port = unsafe { SerialPort::new(self.port_base) };
        port.init();
        self.port.replace(port);
        Ok(())
    }
}

impl Write for SerialWriter {
    /// Writes the data to the I/O port.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        let _ = self.port.as_mut().unwrap().write_str(msg);
        Ok(())
    }
}
