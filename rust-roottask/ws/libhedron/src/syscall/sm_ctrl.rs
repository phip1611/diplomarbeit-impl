//! [`sys_sm_up`] and [`sys_sm_down`].

use crate::capability::CapSel;
use crate::consts::NUM_CAP_SEL;
use crate::syscall::{
    hedron_syscall_1,
    hedron_syscall_3,
    SyscallNum,
};
use crate::syscall::{
    SyscallError,
    SyscallResult,
};
use alloc::string::ToString;

/// Performs an "up" syscall on the semaphore.
///
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_sm_up(sm_sel: CapSel) -> SyscallResult {
    if sm_sel >= NUM_CAP_SEL {
        return Err(SyscallError::ClientArgumentError(
            "Argument `pt_sel` is too big".to_string(),
        ));
    }

    let mut arg1 = 0;
    arg1 |= SyscallNum::SmCtrl.val();
    // no effect; is already null; optimized by compiler
    arg1 |= SmCtrlOperation::Up.val() << 8;
    arg1 |= sm_sel << 12;

    unsafe {
        hedron_syscall_1(arg1)
            .map(|_x| ())
            .map_err(|e| SyscallError::HedronStatusError(e.0))
    }
}

/// Performs an "up" syscall on the semaphore.
///
/// # Parameters
/// * `sm_sel` Selector for the SM object
/// * `counter_strategy` [`SmCtrlZeroCounterStrategy`]
/// * `tsc_timeout` to enable tick-triggered up-operations.
/// # Safety
/// * This function may change the systems functionality in an unintended way,
///   if the arguments are illegal or wrong.
/// * This function is not allowed to panic.
/// * This function is strictly required to never produce any side effect system calls! Therefore,
///   also no log::trace()-stuff or similar. Otherwise, the current implementation of hybrid
///   foreign system calls will fail.
#[inline]
pub fn sys_sm_down(
    sm_sel: CapSel,
    counter_strategy: SmCtrlZeroCounterStrategy,
    tsc_timeout: Option<u64>,
) -> SyscallResult {
    if sm_sel >= NUM_CAP_SEL {
        return Err(SyscallError::ClientArgumentError(
            "Argument `pt_sel` is too big".to_string(),
        ));
    }
    if let Some(timeout) = tsc_timeout {
        if timeout == 0 {
            return Err(SyscallError::ClientArgumentError(
                "Argument `timeout` is zero.".to_string(),
            ));
        }
    }

    let mut arg1 = 0;
    arg1 |= SyscallNum::SmCtrl.val();
    arg1 |= SmCtrlOperation::Down.val() << 8;
    arg1 |= counter_strategy.val() << 9;
    arg1 |= sm_sel << 12;

    let mut arg2 = 0;
    let mut arg3 = 0;

    if let Some(timeout) = tsc_timeout {
        arg2 |= timeout >> 32;
        arg3 |= timeout & 0xffffffff;
    }

    unsafe {
        hedron_syscall_3(arg1, arg2, arg3)
            .map(|_x| ())
            .map_err(|e| SyscallError::HedronStatusError(e.0))
    }
}

/// Types of SM CTRL options.
#[derive(Debug, Copy, Clone)]
#[repr(u64)]
enum SmCtrlOperation {
    /// Performs an "up" operation on the semaphore.
    ///
    /// The up operation releases an execution context blocked on the semaphore if one exists, otherwise it
    /// increments the counter.
    Up = 0,
    /// Performs an "down" operation on the semaphore.
    ///
    /// The down operation blocks the calling execution context if the semaphore counter is zero, otherwise
    /// the counter is decremented or set to zero, depending on the setting of the ZC bit. A non-zero timeout
    /// value can be used to abort this operation at the moment the TSC reaches the specified value.
    Down = 1,
}

impl SmCtrlOperation {
    pub const fn val(self) -> u64 {
        self as u64
    }
}

/// Describes the strategy for the semaphore counter, when `down` operations are executed.
#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum SmCtrlZeroCounterStrategy {
    Decrement = 0,
    SetToZero = 1,
}

impl SmCtrlZeroCounterStrategy {
    pub const fn val(self) -> u64 {
        self as u64
    }
}
