use libhrstd::libhedron::capability::CapSel;
use libroottask::io_port::request_io_port;
use x86::io::outb;

/// Logger that uses I/O port 0xe9.
/// See https://phip1611.de/blog/how-to-use-qemus-debugcon-feature-and-write-to-a-file/
#[derive(Debug)]
pub struct DebugconLogger;

impl DebugconLogger {
    const DEBUGCON_PORT: u16 = 0xe9;

    /// Initializes the debugcon logger for the roottask.
    /// Requests access to the 0xe9 I/O port via syscall.
    pub fn init(root_pd_sel: CapSel) {
        request_io_port(root_pd_sel, Self::DEBUGCON_PORT).unwrap();
    }

    /// Writes the data to the I/O port.
    pub fn write(&self, msg: &str) -> core::fmt::Result {
        msg.bytes().for_each(|b| unsafe {
            outb(Self::DEBUGCON_PORT, b);
        });
        Ok(())
    }
}
