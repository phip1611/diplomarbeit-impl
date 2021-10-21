//! Module for [`PortalIdentifier`].

use crate::libhedron::event_offset::VMExceptionEventOffset;
use crate::process::{
    ProcessId,
    NUM_PROCESSES,
};
use core::fmt::{
    Debug,
    Formatter,
};
use libhedron::consts::NUM_VM_EXC;
use libhedron::event_offset::ExceptionEventOffset;

/// Encodes information that are passed to portal callbacks. Each portal can get
/// a 64-bit value argument when it's invoked. This struct helps to encode
/// payload, the exception and the pid (process id/protection domain), that created
/// the exception.
#[derive(Copy, Clone)]
pub struct PortalIdentifier(u64);

impl PortalIdentifier {
    // max: 256 vCPU exceptions
    const EXC_BITMASK: u64 = 0xff00_0000_0000_0000;
    const EXC_BITSHIFT: u64 = 64 - 8;
    // max 2^16 processes, see NUM_PROCESSES
    const PID_BITMASK: u64 = 0x00ff_ff00_0000_0000;
    const PID_BITSHIFT: u64 = Self::EXC_BITSHIFT - 16;
    const PAYLOAD_BITMASK: u64 = 0x0000_00ff_ffff_ffff;

    /// Creates a new portal identifier.
    pub fn new(exc: u64, pid: ProcessId, payload: u64) -> Self {
        assert!(exc < NUM_VM_EXC as u64, "exception doesn't fit");
        assert!(pid < NUM_PROCESSES, "pid doesn't fit");
        assert!(payload < 2_u64.pow(40), "payload doesn't fit");

        let mut val = 0;
        val |= (exc << Self::EXC_BITSHIFT) & Self::EXC_BITMASK;
        val |= (pid << Self::PID_BITSHIFT) & Self::PID_BITMASK;
        val |= payload & Self::PAYLOAD_BITMASK;
        Self(val)
    }

    /// Returns the raw, encoded u64 value.
    pub const fn val(self) -> u64 {
        self.0
    }

    /// Returns the exception as [`ExceptionEventOffset`].
    pub fn exc(self) -> ExceptionEventOffset {
        ((self.val() & Self::EXC_BITMASK) >> Self::EXC_BITSHIFT).into()
    }

    /// Returns the exception as [`VMExceptionEventOffset`].
    pub fn exc_vmi(self) -> VMExceptionEventOffset {
        todo!()
    }

    /// Returns the payload of the process ID. It is 24 bit long.
    pub const fn pid(self) -> ProcessId {
        (self.val() & Self::PID_BITMASK) >> Self::PID_BITSHIFT
    }

    /// Returns the payload of the [`PortalIdentifier`]. It is 24 bit long.
    pub const fn payload(self) -> u64 {
        self.val() & Self::PAYLOAD_BITMASK
    }
}

impl Debug for PortalIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PortalIdentifier")
            .field("exc", &self.exc())
            // TODO
            // .field("exc_vmi", &self.exc_vmi())
            .field("pid", &self.pid())
            .field("payload", &self.payload())
            // hack: print as hex
            .field("val", &(self.val() as *const u64))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::libhedron::event_offset::ExceptionEventOffset;
    use crate::portal_identifier::PortalIdentifier;

    #[test]
    fn test_portal_identifier() {
        let portal_id = PortalIdentifier::new(
            ExceptionEventOffset::HedronGlobalEcStartup.val(),
            0xffff,
            0xffffffffff,
        );
        assert_eq!(ExceptionEventOffset::HedronGlobalEcStartup, portal_id.exc());
        assert_eq!(0xffff, portal_id.pid());
        assert_eq!(0xffffffffff, portal_id.payload());

        let portal_id =
            PortalIdentifier::new(ExceptionEventOffset::HedronGlobalEcStartup.val(), 1, 0);
        assert_eq!(ExceptionEventOffset::HedronGlobalEcStartup, portal_id.exc());
        assert_eq!(1, portal_id.pid());
        assert_eq!(0, portal_id.payload());
    }
}
