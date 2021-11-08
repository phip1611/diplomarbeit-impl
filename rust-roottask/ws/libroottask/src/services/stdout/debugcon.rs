use crate::io_port::request_io_port;
use core::fmt::Write;
use libhrstd::libhedron::capability::CapSel;
use x86::io::outb;

/// Logger that uses I/O port 0xe9.
/// See https://phip1611.de/blog/how-to-use-qemus-debugcon-feature-and-write-to-a-file/
///
/// **There should only be one instance of this!**
#[derive(Debug)]
pub(super) struct DebugconWriter {}

impl DebugconWriter {
    const DEBUGCON_PORT: u16 = 0xe9;

    pub const fn new() -> Self {
        DebugconWriter {}
    }

    /// Initializes the debugcon logger for the roottask.
    /// Requests access to the 0xe9 I/O port via syscall.
    pub fn init(&mut self, root_pd_sel: CapSel) {
        request_io_port(root_pd_sel, Self::DEBUGCON_PORT).unwrap();
    }
}

impl Write for DebugconWriter {
    /// Writes the data to the I/O port.
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        msg.bytes().for_each(|b| unsafe {
            outb(Self::DEBUGCON_PORT, b);
        });
        Ok(())
    }
}
