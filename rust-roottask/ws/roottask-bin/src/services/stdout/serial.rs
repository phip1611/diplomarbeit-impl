//! Serial logger.

use core::fmt::{
    Debug,
    Write,
};
use libhrstd::libhedron::capability::{
    CapSel,
    CrdPortIO,
};
use libroottask::hw::serial_port::{
    init_serial,
    snd_serial,
    COM1_IO_PORT,
};
use libroottask::io_port::request_io_ports;

/// Logger that uses I/O port 0x3f8. See `serial_port.rs`.
///
/// **There should only be one instance of this!**
#[derive(Debug)]
pub(super) struct SerialWriter;

impl SerialWriter {
    /// Initializes the serial logger for the roottask.
    /// Requests access to the necessary I/O ports.
    pub fn init(&self, root_pd_sel: CapSel) -> Result<(), ()> {
        // order 3: 2^3 = 8 => we need ports [port..port+8]
        request_io_ports(root_pd_sel, CrdPortIO::new(COM1_IO_PORT, 3)).map_err(|_| ())?;
        init_serial()
    }
}

impl Write for SerialWriter {
    /// Writes the data to the I/O port.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        snd_serial(msg.as_bytes());
        Ok(())
    }
}
