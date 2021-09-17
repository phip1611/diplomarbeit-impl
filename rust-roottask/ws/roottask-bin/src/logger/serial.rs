//! Serial logger.

use roottask_lib::hedron::capability::{
    CapSel,
    CrdPortIO,
};
use roottask_lib::hrstd::io_port::request_io_ports;
use roottask_lib::hw::serial_port::{
    init_serial,
    snd_serial,
    COM1_IO_PORT,
};

/// Logger that uses I/O port 0x3f8. See serial_port.rs.
#[derive(Debug)]
pub struct SerialLogger;

impl SerialLogger {
    /// Initializes the serial logger for the roottask.
    /// Requests access to the necessary I/O ports.
    pub fn init(root_pd_sel: CapSel) -> Result<(), ()> {
        // order 3: 2^3 = 8 => we need ports [port..port+8]
        request_io_ports(root_pd_sel, CrdPortIO::new(COM1_IO_PORT, 3)).map_err(|_| ())?;
        init_serial()
    }

    /// Writes the data to the I/O port.
    pub fn write(&self, msg: &str) -> core::fmt::Result {
        snd_serial(msg.as_bytes());
        Ok(())
    }
}
