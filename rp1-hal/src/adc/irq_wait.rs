//! ADC0's one-shot FIFO source for the shared device IRQ waiter.

use super::*;
use crate::{
    driver_irq::{self, CoreState, Instance, Limit, Managed, PerCore, Source},
    timer::{WaitConfig, WaitTimer},
};
use core::marker::PhantomData;
use rp1_rt::interrupt::{Binding, BlockIrq, Slot};

const ADC: u8 = 0;
const ADC_IRQ: u8 = crate::pac::Interrupt::ADC_FIFO as u8;
const FIFO_IRQ: u32 = 1;
type Line<const CORE: u8> = Binding<BlockIrq<{ rp1_proto::BlockId::Adc as u16 }>, ADC_IRQ, CORE>;

/// Capability carried by parts when the declaration reserves ADC_FIFO for the HAL.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

impl<const CORE: u8> Parts<Irq<ADC, CORE>> {
    /// # Safety
    /// Only the generated resource constructor may consume this line. It must
    /// install the matching ADC_FIFO handler on CORE.
    #[doc(hidden)]
    pub unsafe fn __new(line: Line<CORE>) -> Self {
        StateSlots::bind::<CORE>(line.into_line());
        Self {
            _irq: Irq(PhantomData),
            _not_sync: PhantomData,
        }
    }
}

struct FifoSource {
    armed: bool,
}

static PROC0: CoreState<FifoSource, ADC_IRQ, 0> =
    Slot::new(Managed::new(FifoSource { armed: false }));
static PROC1: CoreState<FifoSource, ADC_IRQ, 1> =
    Slot::new(Managed::new(FifoSource { armed: false }));
type StateSlots = PerCore<FifoSource, ADC_IRQ>;

impl Instance<ADC_IRQ> for FifoSource {
    const PROC0: &'static CoreState<Self, ADC_IRQ, 0> = &PROC0;
    const PROC1: &'static CoreState<Self, ADC_IRQ, 1> = &PROC1;
}

impl From<driver_irq::Error> for Error {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::IrqUnavailable,
        }
    }
}

// Only the thread has a request. The ISR uses this same source with no access
// to its channel or state, and leaves FIFO contents and error flags for the thread.
struct WaitIo<'a, I> {
    regs: I,
    request: Option<(&'a mut State, u8)>,
    prepared: bool,
}

impl<I: Registers> WaitIo<'_, I> {
    fn probe(&mut self) -> Option<Result<u16, Error>> {
        let (state, _) = self.request.as_mut()?;
        if !self.prepared {
            if let Err(error) = state.prepare(&mut self.regs) {
                return Some(Err(error));
            }
            self.prepared = true;
        }
        match **state {
            State::Idle => None,
            _ => state.sample(&mut self.regs).transpose(),
        }
    }
}

// The waiter has masked INTE. READY and an empty FIFO qualify the retired
// generation; the next launch programs its own channel.
fn retire_idle_timeout(state: &mut State, io: &mut impl Registers) -> bool {
    let cs = io.read(CS);
    if cs & (CS_POWERED | CS_IDLE_MASK | CS_ERR_STICKY) != CS_POWERED | CS_READY
        || io.read(INTF) & FIFO_IRQ != 0
        || io.read(FCS) & (FCS_LEVEL | FCS_FLAGS | FCS_EMPTY) != FCS_EMPTY
    {
        return false;
    }
    *state = State::Idle;
    true
}

impl<I: Registers> Source<WaitIo<'_, I>> for FifoSource {
    type Error = Error;

    fn is_armed(&self) -> bool {
        self.armed
    }

    fn arm(&mut self, io: &mut WaitIo<'_, I>, mask: u32) -> Result<bool, Error> {
        if self.armed {
            return Ok(false);
        }
        let Some((state, channel)) = io.request.as_mut() else {
            return Ok(false);
        };
        if **state != State::Idle {
            return Ok(false);
        }
        self.armed = true;
        // Start in arm so the FIFO interrupt is enabled before START_ONCE.
        io.regs.write_relaxed(INTE, mask);
        state.launch(&mut io.regs, *channel);
        Ok(true)
    }

    fn service(&mut self, io: &mut WaitIo<'_, I>) -> u32 {
        if io.regs.read(INTS) & FIFO_IRQ == 0 {
            return 0;
        }
        let forced = io.regs.read(INTF) & FIFO_IRQ != 0;
        if forced {
            // A force bypasses INTE. Clear it even when no conversion is armed.
            io.regs.write(INTF, 0);
        }
        let pending = self.armed && (!forced || io.regs.read(INTS) & FIFO_IRQ != 0);
        if pending {
            // The event signal completes the mask write before exception return.
            io.regs.write_relaxed(INTE, 0);
            self.armed = false;
        } else if !self.armed {
            io.regs.write(INTE, 0);
        }
        u32::from(pending)
    }

    fn retire(&mut self, io: &mut WaitIo<'_, I>) {
        if self.armed {
            // The alarm has already finished, so complete the mask here.
            io.regs.write(INTE, 0);
            self.armed = false;
        }
    }
}

/// # Safety
/// Called only by the generated ADC handler on CORE for instance N.
/// Returns the serviced FIFO bit for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    const { assert!(N == ADC) };
    // The HAL never asserts ADC reset, and RESET_DONE cannot indicate readiness.
    StateSlots::on_interrupt::<CORE, _>(&mut WaitIo {
        regs: Mmio,
        request: None,
        prepared: false,
    })
}

/// Synchronous ADC access using ADC_FIFO, a dedicated wait alarm and WFE.
pub struct Blocking<'a, const CORE: u8> {
    adc: &'a mut Adc<Irq<ADC, CORE>>,
    timer: &'a mut WaitTimer,
    config: WaitConfig,
}

impl<const CORE: u8> Adc<Irq<ADC, CORE>> {
    /// Borrow the ADC for IRQ-backed synchronous conversions.
    pub fn blocking<'a>(&'a mut self, timer: &'a mut WaitTimer) -> Blocking<'a, CORE> {
        Blocking {
            adc: self,
            timer,
            config: WaitConfig::new(),
        }
    }
}

impl<const CORE: u8> Blocking<'_, CORE> {
    /// Override the default 1 ms timeout used by [`Self::read`].
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    /// Convert one sample with the configured timeout.
    pub fn read<C: Channel>(&mut self, _channel: C) -> Result<u16, Error> {
        let end = Deadline::after(self.config.timeout_or(TIMEOUT))?;
        self.read_with(C::SELECT, Limit::Relative(end))
    }

    /// Convert one sample by an absolute deadline. A timeout retires the IRQ
    /// and attempts to discard the sample. Check [`Adc::is_recovering`] to see
    /// whether [`Adc::try_cancel`] still needs to run.
    pub fn read_until<C: Channel>(&mut self, _channel: C, end: Deadline) -> Result<u16, Error> {
        self.read_with(C::SELECT, Limit::Absolute(end))
    }

    #[rp1_rt::isram]
    fn read_with(&mut self, channel: u8, limit: Limit) -> Result<u16, Error> {
        let mut io = WaitIo {
            regs: Mmio,
            request: Some((&mut self.adc.state, channel)),
            prepared: false,
        };
        let result =
            StateSlots::wait_for::<CORE, _, _>(&mut io, self.timer, limit, FIFO_IRQ, WaitIo::probe);
        if io.prepared
            && matches!(self.adc.state, State::Converting(_))
            && (result != Err(Error::Timeout)
                || !retire_idle_timeout(&mut self.adc.state, &mut Mmio))
        {
            let _ = self.adc.try_cancel();
        }
        result
    }

    /// Convert one temperature sample, in millidegrees Celsius.
    pub fn read_temperature(&mut self) -> Result<i32, Error> {
        self.read(TEMPERATURE).map(temperature)
    }
}

#[cfg(test)]
#[path = "irq_tests.rs"]
mod tests;
