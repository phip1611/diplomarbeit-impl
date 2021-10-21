//! Typings for Message Transfer Descriptors.

use bitflags::bitflags;

bitflags! {
    /// The Message Transfer Descriptor (MTD) is an architecture-specific
    /// bitfield that controls the contents of an exception or intercept message.
    /// This is the data that is the payload of the [`super::utcb::Utcb`] in case
    /// of an exception.
    ///
    /// The MTD is provided by the portal associated with the event (for example
    /// exception or VM exit) and conveyed to the receiver as part of the exception
    /// or intercept message. For each bit set to 1, the microhypervisor transfers
    /// the architectural state associated with that bit either to/from the
    /// respective fields of the UTCB data area or directly in architectural registers.
    pub struct Mtd: u64 {
        // took everything 1:1 from Hedro: mtd.hpp

        /// Stands for registers `rAx, rBx, rCx, rDx`.
        const GPR_ACDB = 1 << 0;
        /// Stands for registers `rBp, rSi, rDi`.
        const GPR_BSD = 1 << 1;
        /// Stands for register `rSP`.
        const RSP = 1 << 2;
        /// Include the instruction pointer in exception messages.
        const RIP_LEN = 1 << 3;
        const RFLAGS = 1 << 4;
        const DS_ES = 1 << 5;
        const FS_GS = 1 << 6;
        const CS_SS = 1 << 7;
        const TR  = 1 << 8;
        const LDTR = 1 << 9;
        const GDTR = 1 << 10;
        const IDTR = 1 << 11;
        const CR  = 1 << 12;
        const DR  = 1 << 13;
        const SYSENTER = 1 << 14;
        const QUAL = 1 << 15;
        const CTRL = 1 << 16;
        const INJ = 1 << 17;
        const STA = 1 << 18;
        const TSC = 1 << 19;
        const EFER_PAT = 1 << 20;
        const PDPTE = 1 << 21;
        const GPR_R8_R15 = 1 << 22;
        const SYSCALL_SWAPGS = 1 << 23;
        const TSC_TIMEOUT = 1 << 24;

        const VINTR = 1 << 26;
        const EOI = 1 << 27;
        const TPR = 1 << 28;

        const TLB = 1 << 30;
        const FPU = 1 << 31;

        const NONE = 0;

        // the first 24 bits are default
        // I took this from mtd.hpp in supernova-core
        const DEFAULT = 0xffffff;
    }
}
