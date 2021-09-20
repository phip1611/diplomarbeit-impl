//! Event Selector Base describes the offset into
//! occorunces of a specific domain:
//! - exceptions
//! - vmx intercepts
//! - svm intercepts

use bitflags::bitflags;

/// Types of events.
/// WARNING: Don't cast this directly to
/// u64, since it is 128 bytes long!
#[derive(Debug, Copy, Clone)]
pub enum EventBase {
    Exception(ExceptionEventBase),
    // not relevant for my thesis
    Vmx(),
    // not relevant for my thesis
    Svm(),
}

impl EventBase {
    pub fn val(self) -> u64 {
        match self {
            EventBase::Exception(ev) => ev.bits(),
            EventBase::Vmx() => 0,
            EventBase::Svm() => 0,
        }
    }
}

bitflags! {
    /// x86-exceptions.
    /// See https://wiki.osdev.org/Exceptions
    pub struct ExceptionEventBase: u64 {
        /// Divided by Zero error
        const DE = 0x00;
        /// Debug
        const DB = 0x01;
        /// Breakpoint
        const BP = 0x03;
        /// Overflow
        const OF = 0x04;
        /// Bound Range Exceeded
        const BR = 0x05;
        /// Invalid Opcode
        const UD = 0x06;
        /// Device Not Available
        const NM = 0x07;
        /// Double Fault.
        const DF = 0x08;
        /// Invalid TSS
        const TS = 0x0a;
        /// Segment Not Present
        const NP = 0x0b;
        /// Stack Segment Fault
        const SS = 0x0c;
        /// General Protection fault,
        /// i.e. I/O port violation or bad memory access.
        const GP = 0x0d;
        /// Page Fault.
        const PF = 0x0e;
        /// x87 Floating Point Exception
        const MF = 0x10;
        /// Alignment Check
        const AC = 0x11;
        /// Machine Check
        const MC = 0x12;
        /// SIMD Floating Point Exception
        const XM = 0x13;

        /// Hedron-Specific: Startup of an EC
        /// TODO talk with julian
        const STARTUP = 0x1e;

        /// Hedron-Specific
        /// TODO talk with julian
        const RECALL = 0x1f;
    }
}

#[cfg(test)]
mod tests {
    use crate::hedron::event_base::{
        EventBase,
        ExceptionEventBase,
    };
    use core::mem::{
        size_of,
        size_of_val,
    };

    #[test]
    fn test() {
        println!(
            "sizeof ExceptionEventBase: {}",
            size_of::<ExceptionEventBase>()
        );
        println!("sizeof EventBase: {}", size_of::<EventBase>());
        let val = EventBase::Exception(ExceptionEventBase::all());
        println!("sizeof EventBase(Exception): {}", size_of_val(&val));
    }
}
