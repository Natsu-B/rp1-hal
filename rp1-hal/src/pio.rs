#![doc = include_str!("../../docs/pio.md")]

mod fifo;
mod irq_wait;
mod program;
mod sm;

#[cfg(feature = "pio-asm")]
pub use ::pio::{pio_asm, pio_file};
pub use fifo::{Buffers, Polling, Rx, Tx, WaitError};
#[cfg(test)]
pub(crate) use irq_wait::FifoSource;
pub use irq_wait::{__bind_irq, __on_interrupt, Blocking, Irq};
pub use pio_core::Program;
pub use program::{InstallError, InstalledProgram};
pub use sm::{
    BuildError, InitialOutput, PIOBuilder, Pins, Running, ShiftDirection, StateMachine, Stopped,
    SyncStart,
};

use crate::{Error, FreeError, TIMER_RAWL, clocks::Clocks, read, write};
use core::{cell::Cell, marker::PhantomData};

const RESET: u32 = 1 << 27;
const RESET_SET: usize = 0x4001_6000;
const RESET_CLEAR: usize = 0x4001_7000;
const RESET_DONE: usize = 0x4001_4018;
const RESET_TIMEOUT_US: u32 = 1_000;

const BASE: usize = 0xf000_0000;
const CTRL: usize = BASE;
const FSTAT: usize = BASE + 4;
const FDEBUG: usize = BASE + 8;
const FLEVEL: usize = BASE + 0x0c;
const FLEVEL2: usize = BASE + 0x10;
const IRQ: usize = BASE + 0x34;
const IRQ0_INTE: usize = BASE + 0x150;
const IRQ0_INTF: usize = BASE + 0x154;
const IRQ1_INTE: usize = BASE + 0x15c;
const IRQ1_INTF: usize = BASE + 0x160;

pub(crate) const fn sm_reg(n: u8, offset: usize) -> usize {
    BASE + 0xcc + 0x20 * n as usize + offset
}

/// The PIO declaration and its exact set of SM tokens.
pub struct PioParts<const MASK: u8, S> {
    sms: S,
    reset: Reset,
    local: PhantomData<*mut ()>,
}

impl<const MASK: u8, S> PioParts<MASK, S> {
    pub const SM_MASK: u8 = MASK;

    /// # Safety
    /// Call once on the declaring core, with exclusive ownership of PIO and its reset bit.
    /// Supply one token for each SM in MASK.
    #[doc(hidden)]
    pub const unsafe fn __new(sms: S) -> Self {
        Self {
            sms,
            reset: Reset::Fresh,
            local: PhantomData,
        }
    }
}

/// An unconfigured bank-0 pad reserved by the core's PIO declaration.
pub struct PioPin<const N: u8>(PhantomData<*mut ()>);

impl<const N: u8> PioPin<N> {
    /// # Safety
    /// Only the declaration may issue this token, once on the core that reserves N for PIO.
    #[doc(hidden)]
    pub const unsafe fn __new() -> Self {
        assert!(N < 28);
        Self(PhantomData)
    }
}

/// An unconfigured state machine. A declaration issues each selected SM once.
pub struct UninitStateMachine<const N: u8, I = ()>(PhantomData<(*mut (), I)>);

impl<const N: u8, I> UninitStateMachine<N, I> {
    /// # Safety
    /// Only the declaration may issue this token, once for the PIO owner and SM N.
    /// It must install and bind the handlers for any IRQ capability in I.
    #[doc(hidden)]
    pub const unsafe fn __new() -> Self {
        assert!(N < 4);
        Self(PhantomData)
    }
}

/// The instruction memory owner. Dropped programs retain their allocation.
pub struct PIO {
    clocks: Clocks,
    used: Cell<u32>,
    active: [Cell<u32>; 4],
    local: PhantomData<*mut ()>,
}

mod sealed {
    pub trait Parts {}
    impl<const MASK: u8, S> Parts for super::PioParts<MASK, S> {}
}

pub trait PIOExt: sealed::Parts + Sized {
    type StateMachines;

    /// Initialize PIO. Retry with the returned parts on `Error::NotReady`.
    fn split(self, clocks: &Clocks) -> Result<(PIO, Self::StateMachines), FreeError<Self, Error>>;

    /// Initialize PIO, polling the 1 MHz timer. Each reset transition has a 1 ms deadline.
    /// On failure, returns the parts and the clock or timeout error.
    fn split_wait(
        mut self,
        clocks: &Clocks,
    ) -> Result<(PIO, Self::StateMachines), FreeError<Self, Error>> {
        loop {
            match self.split(clocks) {
                Err(error) if *error.error() == Error::NotReady => self = error.into_driver(),
                result => return result,
            }
            core::hint::spin_loop();
        }
    }
}

impl<const MASK: u8, S> PIOExt for PioParts<MASK, S> {
    type StateMachines = S;

    fn split(mut self, clocks: &Clocks) -> Result<(PIO, S), FreeError<Self, Error>> {
        if !clocks.sys_current_strict() {
            return Err(FreeError::new(self, Error::Precondition));
        }
        if let Err(error) = self.reset.step(read, write) {
            return Err(FreeError::new(self, error));
        }
        write(CTRL, 0);
        // The reset also clears FIFO state and the DMA request enables.
        for register in [IRQ0_INTE, IRQ0_INTF, IRQ1_INTE, IRQ1_INTF] {
            write(register, 0);
        }
        let pio = PIO {
            clocks: *clocks,
            used: Cell::new(0),
            active: [const { Cell::new(0) }; 4],
            local: PhantomData,
        };
        Ok((pio, self.sms))
    }
}

#[derive(Clone, Copy)]
enum Reset {
    Fresh,
    Asserted(u32),
    Released(u32),
    Failed,
}

impl Reset {
    fn step(
        &mut self,
        mut read: impl FnMut(usize) -> u32,
        mut write: impl FnMut(usize, u32),
    ) -> Result<(), Error> {
        if matches!(self, Self::Fresh) {
            let start = read(TIMER_RAWL);
            write(RESET_SET, RESET);
            *self = Self::Asserted(start);
        }
        if let Self::Asserted(start) = *self {
            let done = read(RESET_DONE) & RESET != 0;
            self.check_deadline(read(TIMER_RAWL).wrapping_sub(start))?;
            if done {
                return Err(Error::NotReady);
            }
            let start = read(TIMER_RAWL);
            write(RESET_CLEAR, RESET);
            *self = Self::Released(start);
        }
        if let Self::Released(start) = *self {
            let done = read(RESET_DONE) & RESET != 0;
            if let Err(error) = self.check_deadline(read(TIMER_RAWL).wrapping_sub(start)) {
                write(RESET_SET, RESET);
                return Err(error);
            }
            return if done { Ok(()) } else { Err(Error::NotReady) };
        }
        Err(Error::Timeout)
    }

    fn check_deadline(&mut self, elapsed: u32) -> Result<(), Error> {
        if elapsed < RESET_TIMEOUT_US {
            Ok(())
        } else {
            *self = Self::Failed;
            Err(Error::Timeout)
        }
    }
}

#[inline(never)]
fn control(enable: u8, disable: u8, restart: u8, clk_restart: u8) {
    rp1_rt::interrupt::__primask_free(|| {
        let enabled = (read(CTRL) as u8 & 0xf & !disable) | enable;
        write(
            CTRL,
            u32::from(enabled) | u32::from(restart) << 4 | u32::from(clk_restart) << 8,
        );
    });
}

#[cfg(test)]
mod tests;
